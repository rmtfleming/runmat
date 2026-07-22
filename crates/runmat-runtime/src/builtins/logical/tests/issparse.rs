//! MATLAB-compatible `issparse` builtin for RunMat.
//!
//! Clean-room provenance: specs/016-shape-storage-predicates (spec
//! `issparse` 0.2.0, claims issparse.signature-primary,
//! issparse.output-class, issparse.logical-sparse-test).

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

#[runmat_macros::register_gpu_spec(builtin_path = "crate::builtins::logical::tests::issparse")]
pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "issparse",
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
        "GPU tensors are dense storage; the predicate answers from value kind without gathering.",
};

#[runmat_macros::register_fusion_spec(builtin_path = "crate::builtins::logical::tests::issparse")]
pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "issparse",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Storage check executed outside fusion; planners treat it as a scalar metadata query.",
};

const ISSPARSE_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "tf",
    ty: BuiltinParamType::LogicalArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "True when the input uses sparse storage.",
}];

const ISSPARSE_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "A",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Value to test.",
}];

const ISSPARSE_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "tf = issparse(A)",
    inputs: &ISSPARSE_INPUTS,
    outputs: &ISSPARSE_OUTPUT,
}];

const ISSPARSE_ERRORS: [BuiltinErrorDescriptor; 0] = [];

pub const ISSPARSE_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &ISSPARSE_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ISSPARSE_ERRORS,
};

#[runtime_builtin(
    name = "issparse",
    category = "logical/tests",
    summary = "Return true when a value uses sparse storage.",
    keywords = "issparse,sparse,storage,type,predicate",
    accel = "metadata",
    type_resolver(bool_scalar_type),
    descriptor(crate::builtins::logical::tests::issparse::ISSPARSE_DESCRIPTOR),
    builtin_path = "crate::builtins::logical::tests::issparse"
)]
async fn issparse_builtin(value: Value) -> BuiltinResult<Value> {
    Ok(Value::Bool(issparse_value(&value)))
}

fn bool_scalar_type(_: &[Type], _context: &ResolveContext) -> Type {
    Type::Bool
}

/// `Value::SparseTensor` is RunMat's sole sparse representation; every
/// other runtime kind — including GPU tensor handles, which are dense
/// device storage — reports false without gathering (unobserved input
/// classes handled by documented independent choice; issparse.q-output
/// carried unresolved on the specification side).
fn issparse_value(value: &Value) -> bool {
    matches!(value, Value::SparseTensor(_))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::builtins::common::test_support;
    use futures::executor::block_on;
    use runmat_builtins::{CellArray, CharArray, SparseTensor, Tensor};

    fn run_issparse(value: Value) -> Value {
        block_on(super::issparse_builtin(value)).expect("issparse")
    }

    // Normative [FR-016-03; issparse.logical-sparse-test,
    // issparse.output-class] — observed case `sparse`:
    // issparse(sparse([1 0; 0 2])) => true.
    #[test]
    fn observed_sparse_matrix_is_true() {
        let sparse = SparseTensor::new(2, 2, vec![0, 1, 2], vec![0, 1], vec![1.0, 2.0]).unwrap();
        assert_eq!(run_issparse(Value::SparseTensor(sparse)), Value::Bool(true));
    }

    // Normative — observed case `full`: issparse([1 2; 3 4]) => false.
    #[test]
    fn observed_full_matrix_is_false() {
        let full = Tensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
        assert_eq!(run_issparse(Value::Tensor(full)), Value::Bool(false));
    }

    // Normative — observed case `empty-sparse`: issparse(sparse([])) => true.
    #[test]
    fn observed_empty_sparse_is_true() {
        let empty = SparseTensor::zeros(0, 0);
        assert_eq!(run_issparse(Value::SparseTensor(empty)), Value::Bool(true));
    }

    // unresolved_choice: non-array kinds unobserved; every non-sparse
    // runtime kind reports false (issparse.q-output carried unresolved).
    #[test]
    fn unresolved_choice_non_sparse_kinds_are_false() {
        assert_eq!(run_issparse(Value::Num(5.0)), Value::Bool(false));
        assert_eq!(run_issparse(Value::Bool(true)), Value::Bool(false));
        assert_eq!(
            run_issparse(Value::CharArray(CharArray::new_row("abc"))),
            Value::Bool(false)
        );
        let cell = CellArray::new(vec![Value::Num(1.0)], 1, 1).unwrap();
        assert_eq!(run_issparse(Value::Cell(cell)), Value::Bool(false));
    }

    // unresolved_choice: GPU handles unobserved; dense device storage
    // reports false from the value kind without gathering.
    #[test]
    fn unresolved_choice_gpu_tensor_is_false_without_gather() {
        test_support::with_test_provider(|provider| {
            let tensor = Tensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
            let view = runmat_accelerate_api::HostTensorView {
                data: &tensor.data,
                shape: &tensor.shape,
            };
            let handle = provider.upload(&view).expect("upload");
            assert_eq!(run_issparse(Value::GpuTensor(handle)), Value::Bool(false));
        });
    }
}
