// Parallel clean-room implementation of `isrow` (independently developed against the
// matlab-interface-spec black-box specs). 0-6-0 ships the DEFAULT implementation; this copy is
// retained unregistered for differential cross-validation. Original path: crates/runmat-runtime/src/builtins/logical/tests/isrow.rs
//! MATLAB-compatible `isrow` builtin for RunMat.
//!
//! Clean-room provenance: specs/016-shape-storage-predicates (spec `isrow`
//! 0.2.0, claims isrow.signature-primary, isrow.output-class,
//! isrow.logical-row-test).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    ResolveContext, Type, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::shape::value_dimensions;
use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ScalarType, ShapeRequirements,
};
use crate::BuiltinResult;

pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "isrow",
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
    notes: "Answers from GPU handle shape metadata; gathers only when a provider omits shape.",
};

pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "isrow",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Shape check executed outside fusion; planners treat it as a scalar metadata query.",
};

const ISROW_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "tf",
    ty: BuiltinParamType::LogicalArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "True when the input is a row vector (1-by-N, including scalars).",
}];

const ISROW_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "A",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Value to test.",
}];

const ISROW_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "tf = isrow(A)",
    inputs: &ISROW_INPUTS,
    outputs: &ISROW_OUTPUT,
}];

const ISROW_ERRORS: [BuiltinErrorDescriptor; 0] = [];

pub const ISROW_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &ISROW_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ISROW_ERRORS,
};

async fn isrow_builtin(value: Value) -> BuiltinResult<Value> {
    Ok(Value::Bool(isrow_value(&value).await?))
}

fn bool_scalar_type(_: &[Type], _context: &ResolveContext) -> Type {
    Type::Bool
}

/// A value is a row when its MATLAB-visible shape is 2-D with a first
/// dimension of 1; scalars normalise to `[1, 1]` and therefore count as
/// rows (observed). N-D shapes (ndims > 2) report false and `1x0` empties
/// report true — both unobserved input classes handled by documented
/// independent choice (isrow.q-empty carried unresolved).
async fn isrow_value(value: &Value) -> BuiltinResult<bool> {
    let dims = value_dimensions(value).await?;
    Ok(dims.len() == 2 && dims[0] == 1)
}

#[cfg(all(test, feature = "parallel_xval"))]
pub(crate) mod tests {
    use super::*;
    use crate::builtins::common::test_support;
    use futures::executor::block_on;
    use runmat_builtins::Tensor;

    fn run_isrow(value: Value) -> Value {
        block_on(super::isrow_builtin(value)).expect("isrow")
    }

    // Normative [FR-016-01; isrow.logical-row-test, isrow.output-class]
    // Observed case `row`: isrow([1 2 3]) => true.
    #[test]
    fn observed_row_vector_is_true() {
        let row = Tensor::new(vec![1.0, 2.0, 3.0], vec![1, 3]).unwrap();
        assert_eq!(run_isrow(Value::Tensor(row)), Value::Bool(true));
    }

    // Normative — observed case `column`: isrow([1; 2; 3]) => false.
    #[test]
    fn observed_column_vector_is_false() {
        let column = Tensor::new(vec![1.0, 2.0, 3.0], vec![3, 1]).unwrap();
        assert_eq!(run_isrow(Value::Tensor(column)), Value::Bool(false));
    }

    // Normative — observed case `scalar`: isrow(5) => true.
    #[test]
    fn observed_scalar_is_true() {
        assert_eq!(run_isrow(Value::Num(5.0)), Value::Bool(true));
    }

    // Normative — observed case `matrix`: isrow([1 2; 3 4]) => false.
    #[test]
    fn observed_matrix_is_false() {
        let matrix = Tensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
        assert_eq!(run_isrow(Value::Tensor(matrix)), Value::Bool(false));
    }

    // unresolved_choice: empty shapes unobserved (isrow.q-empty); the size
    // rule is applied uniformly — 1x0 is a row, 0x1 and 0x0 are not.
    #[test]
    fn unresolved_choice_empty_shapes_follow_size_rule() {
        let empty_row = Tensor::new(Vec::new(), vec![1, 0]).unwrap();
        let empty_column = Tensor::new(Vec::new(), vec![0, 1]).unwrap();
        let empty = Tensor::new(Vec::new(), vec![0, 0]).unwrap();
        assert_eq!(run_isrow(Value::Tensor(empty_row)), Value::Bool(true));
        assert_eq!(run_isrow(Value::Tensor(empty_column)), Value::Bool(false));
        assert_eq!(run_isrow(Value::Tensor(empty)), Value::Bool(false));
    }

    // unresolved_choice: N-D inputs unobserved; ndims > 2 reports false
    // (trailing singleton dimensions are not collapsed, as in isvector).
    #[test]
    fn unresolved_choice_nd_array_is_false() {
        let cube = Tensor::new(vec![0.0; 3], vec![1, 3, 1]).unwrap();
        assert_eq!(run_isrow(Value::Tensor(cube)), Value::Bool(false));
    }

    // unresolved_choice: GPU handles unobserved; the predicate answers from
    // handle shape metadata without gathering.
    #[test]
    fn unresolved_choice_gpu_tensor_uses_handle_shape() {
        test_support::with_test_provider(|provider| {
            let row = Tensor::new(vec![1.0, 2.0, 3.0], vec![1, 3]).unwrap();
            let matrix = Tensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
            let row_view = runmat_accelerate_api::HostTensorView {
                data: &row.data,
                shape: &row.shape,
            };
            let matrix_view = runmat_accelerate_api::HostTensorView {
                data: &matrix.data,
                shape: &matrix.shape,
            };
            let row_handle = provider.upload(&row_view).expect("upload row");
            let matrix_handle = provider.upload(&matrix_view).expect("upload matrix");
            assert_eq!(run_isrow(Value::GpuTensor(row_handle)), Value::Bool(true));
            assert_eq!(
                run_isrow(Value::GpuTensor(matrix_handle)),
                Value::Bool(false)
            );
        });
    }
}
