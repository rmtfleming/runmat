//! MATLAB-compatible `isstruct` builtin for RunMat.
//!
//! Clean-room provenance: specs/002-type-predicates (spec `isstruct` 0.2.0,
//! claims isstruct.signature-primary, isstruct.output-class,
//! isstruct.logical-struct-test).

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

#[runmat_macros::register_gpu_spec(builtin_path = "crate::builtins::logical::tests::isstruct")]
pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "isstruct",
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
    notes:
        "GPU tensors are never structs; the predicate answers from value kind without gathering.",
};

#[runmat_macros::register_fusion_spec(builtin_path = "crate::builtins::logical::tests::isstruct")]
pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "isstruct",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Type check executed outside fusion; planners treat it as a scalar metadata query.",
};

const ISSTRUCT_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "tf",
    ty: BuiltinParamType::LogicalArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "True when the input is a structure (scalar or array).",
}];

const ISSTRUCT_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "A",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Value to test.",
}];

const ISSTRUCT_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "tf = isstruct(A)",
    inputs: &ISSTRUCT_INPUTS,
    outputs: &ISSTRUCT_OUTPUT,
}];

const ISSTRUCT_ERRORS: [BuiltinErrorDescriptor; 0] = [];

pub const ISSTRUCT_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &ISSTRUCT_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ISSTRUCT_ERRORS,
};

#[runtime_builtin(
    name = "isstruct",
    category = "logical/tests",
    summary = "Return true when a value is a structure array.",
    keywords = "isstruct,struct,type,predicate",
    accel = "metadata",
    type_resolver(bool_scalar_type),
    descriptor(crate::builtins::logical::tests::isstruct::ISSTRUCT_DESCRIPTOR),
    builtin_path = "crate::builtins::logical::tests::isstruct"
)]
async fn isstruct_builtin(value: Value) -> BuiltinResult<Value> {
    Ok(Value::Bool(isstruct_value(&value)))
}

fn bool_scalar_type(_: &[Type], _context: &ResolveContext) -> Type {
    Type::Bool
}

/// Struct arrays are cells whose elements are all structs (see
/// `structs::core::isfield::classify_struct`); a non-empty such cell reports
/// true. The empty cell reports false so the observed `iscell({}) == true`
/// stays exclusive with this predicate (empty struct-array input is
/// unobserved in the approved spec — documented independent choice).
fn isstruct_value(value: &Value) -> bool {
    match value {
        Value::Struct(_) => true,
        Value::Cell(cell) => {
            !cell.data.is_empty() && cell.data.iter().all(|v| matches!(v, Value::Struct(_)))
        }
        _ => false,
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use futures::executor::block_on;
    use runmat_builtins::{CellArray, StructValue};

    fn run_isstruct(value: Value) -> Value {
        block_on(super::isstruct_builtin(value)).expect("isstruct")
    }

    fn scalar_struct() -> Value {
        let mut st = StructValue::new();
        st.insert("a", Value::Num(1.0));
        Value::Struct(st)
    }

    // Normative [FR-002-02; isstruct.logical-struct-test,
    // isstruct.output-class] — observed case `struct`: isstruct(struct('a',1)).
    #[test]
    fn observed_scalar_struct_is_true() {
        assert_eq!(run_isstruct(scalar_struct()), Value::Bool(true));
    }

    // Normative — observed case `struct-array`: isstruct(struct('a',{1,2})).
    // RunMat represents struct arrays as cells whose elements are all structs.
    #[test]
    fn observed_struct_array_is_true() {
        let cell = CellArray::new(vec![scalar_struct(), scalar_struct()], 1, 2).unwrap();
        assert_eq!(run_isstruct(Value::Cell(cell)), Value::Bool(true));
    }

    // Normative — observed cases `numeric`, `cell` ({1}).
    #[test]
    fn observed_non_struct_inputs_are_false() {
        assert_eq!(run_isstruct(Value::Num(5.0)), Value::Bool(false));
        let cell = CellArray::new(vec![Value::Num(1.0)], 1, 1).unwrap();
        assert_eq!(run_isstruct(Value::Cell(cell)), Value::Bool(false));
    }

    // unresolved_choice: empty cell reports false (empty struct-array input
    // unobserved; keeps exclusivity with observed iscell({}) == true).
    #[test]
    fn unresolved_choice_empty_cell_is_false() {
        let cell = CellArray::new(vec![], 0, 0).unwrap();
        assert_eq!(run_isstruct(Value::Cell(cell)), Value::Bool(false));
    }
}
