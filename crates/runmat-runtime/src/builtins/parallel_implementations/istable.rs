// Parallel clean-room implementation of `istable` (independently developed against the
// matlab-interface-spec black-box specs). 0-6-0 ships the DEFAULT implementation; this copy is
// retained unregistered for differential cross-validation. Original path: crates/runmat-runtime/src/builtins/logical/tests/istable.rs
//! MATLAB-compatible `istable` builtin for RunMat.
//!
//! Clean-room provenance: specs/016-shape-storage-predicates (spec
//! `istable` 0.2.0, claims istable.signature-primary, istable.output-class,
//! istable.logical-table-test).
//!
//! Gate-ratified decision: RunMat's runtime value model has no table type
//! (Blocker B-010-1, `specs/010-table-conversion/spec.md`), so this builtin
//! returns false for every value kind — correct for every constructible
//! input. The observed true-case (istable.logical-table-test, case `table`:
//! `istable(array2table([1 2; 3 4])) => true`) is unreachable until a table
//! type exists; this feature returns to planning when one lands.

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
    name: "istable",
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
    notes: "GPU tensors are never tables; the predicate answers from value kind without gathering.",
};

pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "istable",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Type check executed outside fusion; planners treat it as a scalar metadata query.",
};

const ISTABLE_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "tf",
    ty: BuiltinParamType::LogicalArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "True when the input is a table.",
}];

const ISTABLE_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "A",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Value to test.",
}];

const ISTABLE_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "tf = istable(A)",
    inputs: &ISTABLE_INPUTS,
    outputs: &ISTABLE_OUTPUT,
}];

const ISTABLE_ERRORS: [BuiltinErrorDescriptor; 0] = [];

pub const ISTABLE_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &ISTABLE_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ISTABLE_ERRORS,
};

async fn istable_builtin(value: Value) -> BuiltinResult<Value> {
    Ok(Value::Bool(istable_value(&value)))
}

fn bool_scalar_type(_: &[Type], _context: &ResolveContext) -> Type {
    Type::Bool
}

/// No RunMat value kind is a table (Blocker B-010-1), so every input
/// reports false — matching the observed false cases
/// (istable.logical-table-test, cases `numeric`, `cell`). The observed
/// true-case (case `table`) is unreachable until a table type exists; see
/// the module-level documented note.
fn istable_value(_value: &Value) -> bool {
    // No variant of `Value` represents a table today.
    false
}

#[cfg(all(test, feature = "parallel_xval"))]
pub(crate) mod tests {
    use super::*;
    use crate::builtins::common::test_support;
    use futures::executor::block_on;
    use runmat_builtins::{CellArray, CharArray, SparseTensor, StructValue, Tensor};

    fn run_istable(value: Value) -> Value {
        block_on(super::istable_builtin(value)).expect("istable")
    }

    // Normative [FR-016-05; istable.logical-table-test,
    // istable.output-class] — observed case `numeric`:
    // istable([1 2; 3 4]) => false.
    #[test]
    fn observed_numeric_matrix_is_false() {
        let matrix = Tensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
        assert_eq!(run_istable(Value::Tensor(matrix)), Value::Bool(false));
    }

    // Normative — observed case `cell`: istable({1, 2}) => false.
    #[test]
    fn observed_cell_is_false() {
        let cell = CellArray::new(vec![Value::Num(1.0), Value::Num(2.0)], 1, 2).unwrap();
        assert_eq!(run_istable(Value::Cell(cell)), Value::Bool(false));
    }

    // Documented note — observed case `table`
    // (istable(array2table([1 2; 3 4])) => true) is UNREACHABLE: RunMat has
    // no table type (Blocker B-010-1), so no test can construct it. The
    // gate-ratified always-false implementation is correct for every
    // constructible input; this comment records the observed basis.

    // unresolved_choice: inputs beyond the observed cases are unobserved;
    // every constructible RunMat kind reports false (no table kind exists).
    #[test]
    fn unresolved_choice_every_constructible_kind_is_false() {
        assert_eq!(run_istable(Value::Num(5.0)), Value::Bool(false));
        assert_eq!(run_istable(Value::Bool(true)), Value::Bool(false));
        assert_eq!(
            run_istable(Value::CharArray(CharArray::new_row("abc"))),
            Value::Bool(false)
        );
        assert_eq!(
            run_istable(Value::String("text".to_string())),
            Value::Bool(false)
        );
        let mut st = StructValue::new();
        st.insert("a", Value::Num(1.0));
        assert_eq!(run_istable(Value::Struct(st)), Value::Bool(false));
        let sparse = SparseTensor::zeros(2, 2);
        assert_eq!(run_istable(Value::SparseTensor(sparse)), Value::Bool(false));
    }

    // unresolved_choice: GPU handles unobserved; dense device storage is
    // never a table and reports false without gathering.
    #[test]
    fn unresolved_choice_gpu_tensor_is_false_without_gather() {
        test_support::with_test_provider(|provider| {
            let tensor = Tensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
            let view = runmat_accelerate_api::HostTensorView {
                data: &tensor.data,
                shape: &tensor.shape,
            };
            let handle = provider.upload(&view).expect("upload");
            assert_eq!(run_istable(Value::GpuTensor(handle)), Value::Bool(false));
        });
    }
}
