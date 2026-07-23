// Parallel clean-room implementation of `iscell` (independently developed against the
// matlab-interface-spec black-box specs). 0-6-0 ships the DEFAULT implementation; this copy is
// retained unregistered for differential cross-validation. Original path: crates/runmat-runtime/src/builtins/logical/tests/iscell.rs
//! MATLAB-compatible `iscell` builtin for RunMat.
//!
//! Clean-room provenance: specs/002-type-predicates (spec `iscell` 0.2.0,
//! claims iscell.signature-primary, iscell.output-class,
//! iscell.logical-cell-test).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    ResolveContext, Type, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ScalarType, ShapeRequirements,
};
use crate::BuiltinResult;

pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "iscell",
    op_kind: GpuOpKind::Custom("metadata"),
    supported_precisions: &[ScalarType::F32, ScalarType::F64],
    broadcast: BroadcastSemantics::None,
    provider_hooks: &[],
    constant_strategy: ConstantStrategy::InlineLiteral,
    residency: ResidencyPolicy::GatherImmediately,
    nan_mode: ReductionNaN::Include,
    two_pass_threshold: None,
    workgroup_size: None,
    accepts_nan_mode: false,
    notes: "GPU tensors are never cell arrays; the predicate answers from value kind without gathering.",
};

pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "iscell",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Type check executed outside fusion; planners treat it as a scalar metadata query.",
};

const ISCELL_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "tf",
    ty: BuiltinParamType::LogicalArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "True when the input is a cell array.",
}];

const ISCELL_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "A",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Value to test.",
}];

const ISCELL_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "tf = iscell(A)",
    inputs: &ISCELL_INPUTS,
    outputs: &ISCELL_OUTPUT,
}];

const ISCELL_ERRORS: [BuiltinErrorDescriptor; 0] = [];

pub const ISCELL_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &ISCELL_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ISCELL_ERRORS,
};

async fn iscell_builtin(value: Value) -> BuiltinResult<Value> {
    Ok(Value::Bool(iscell_value(&value)))
}

fn bool_scalar_type(_: &[Type], _context: &ResolveContext) -> Type {
    Type::Bool
}

/// Struct arrays share the cell representation (a cell whose elements are all
/// structs — see `structs::core::isfield::classify_struct`); those report
/// false here so `iscell`/`isstruct` stay mutually exclusive. That input
/// class is unobserved in the approved spec (documented independent choice).
fn iscell_value(value: &Value) -> bool {
    match value {
        Value::Cell(cell) => {
            cell.data.is_empty() || !cell.data.iter().all(|v| matches!(v, Value::Struct(_)))
        }
        _ => false,
    }
}

#[cfg(all(test, feature = "parallel_xval"))]
pub(crate) mod tests {
    use super::*;
    use futures::executor::block_on;
    use runmat_builtins::{CellArray, CharArray, StructValue, Tensor};

    fn run_iscell(value: Value) -> Value {
        block_on(super::iscell_builtin(value)).expect("iscell")
    }

    fn scalar_struct() -> Value {
        let mut st = StructValue::new();
        st.insert("a", Value::Num(1.0));
        Value::Struct(st)
    }

    // Normative [FR-002-01; iscell.logical-cell-test, iscell.output-class]
    // Observed case `cell-array`: iscell({1, 'a', [1 2]}) => true.
    #[test]
    fn observed_cell_array_is_true() {
        let cell = CellArray::new(
            vec![
                Value::Num(1.0),
                Value::CharArray(CharArray::new_row("a")),
                Value::Tensor(Tensor::new(vec![1.0, 2.0], vec![1, 2]).unwrap()),
            ],
            1,
            3,
        )
        .unwrap();
        assert_eq!(run_iscell(Value::Cell(cell)), Value::Bool(true));
    }

    // Normative — observed case `empty-cell`: iscell({}) => true.
    #[test]
    fn observed_empty_cell_is_true() {
        let cell = CellArray::new(vec![], 0, 0).unwrap();
        assert_eq!(run_iscell(Value::Cell(cell)), Value::Bool(true));
    }

    // Normative — observed cases `numeric-scalar`, `char-row`, `struct`.
    #[test]
    fn observed_non_cell_inputs_are_false() {
        assert_eq!(run_iscell(Value::Num(5.0)), Value::Bool(false));
        assert_eq!(
            run_iscell(Value::CharArray(CharArray::new_row("abc"))),
            Value::Bool(false)
        );
        assert_eq!(run_iscell(scalar_struct()), Value::Bool(false));
    }

    // unresolved_choice: struct arrays (cell-of-structs representation)
    // report false; unobserved input class in the approved spec.
    #[test]
    fn unresolved_choice_struct_array_representation_is_false() {
        let cell = CellArray::new(vec![scalar_struct(), scalar_struct()], 1, 2).unwrap();
        assert_eq!(run_iscell(Value::Cell(cell)), Value::Bool(false));
    }
}
