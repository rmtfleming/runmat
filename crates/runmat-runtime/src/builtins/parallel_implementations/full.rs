// Parallel clean-room implementation of `full` (independently developed against the
// matlab-interface-spec black-box specs). 0-6-0 ships the DEFAULT implementation; this copy is
// retained unregistered for differential cross-validation. Original path: crates/runmat-runtime/src/builtins/math/sparse/full.rs
//! MATLAB-compatible `full` builtin for RunMat.
//!
//! Clean-room provenance: specs/009-sparse-construction (spec `full` 0.2.0,
//! claims full.signature-primary, full.output-class,
//! full.full-preserves-values).

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
use crate::{build_runtime_error, BuiltinResult, RuntimeError};

pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "full",
    op_kind: GpuOpKind::Custom("sparse"),
    supported_precisions: &[ScalarType::F32, ScalarType::F64],
    broadcast: BroadcastSemantics::None,
    provider_hooks: &[],
    constant_strategy: ConstantStrategy::InlineLiteral,
    residency: ResidencyPolicy::GatherImmediately,
    nan_mode: ReductionNaN::Include,
    two_pass_threshold: None,
    workgroup_size: None,
    accepts_nan_mode: false,
    notes: "Sparse values are host-resident CSC; densification runs on host. Dense inputs (including GPU tensors) pass through unchanged without gathering.",
};

pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "full",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Storage-representation change; not fusible.",
};

const FULL_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "B",
    ty: BuiltinParamType::NumericArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Full-storage array with the same values as the input.",
}];

const FULL_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "S",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Sparse or full array.",
}];

const FULL_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "B = full(S)",
    inputs: &FULL_INPUTS,
    outputs: &FULL_OUTPUT,
}];

const FULL_ERROR_INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.FULL.INTERNAL",
    identifier: Some("RunMat:full:Internal"),
    when: "Densification fails (dimension overflow or allocation failure).",
    message: "full: densification failed",
};

const FULL_ERRORS: [BuiltinErrorDescriptor; 1] = [FULL_ERROR_INTERNAL];

pub const FULL_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &FULL_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &FULL_ERRORS,
};

async fn full_builtin(value: Value) -> BuiltinResult<Value> {
    match value {
        // Sparse input: expand to a dense double tensor (column-major),
        // preserving values; empty sparse yields a 0x0 dense array
        // [full.full-preserves-values, full.output-class].
        Value::SparseTensor(sparse) => sparse.to_dense().map(Value::Tensor).map_err(internal_error),
        // Already-full input: returned unchanged [full.full-preserves-values,
        // observed case `already-full`]. Non-sparse kinds beyond the observed
        // dense double matrix (scalars, logicals, GPU tensors, ...) pass
        // through by documented independent choice (full.q-class carried
        // unresolved); complex sparse does not exist in RunMat's value model
        // (full.q-complex moot on this substrate).
        other => Ok(other),
    }
}

fn full_type(args: &[Type], _context: &ResolveContext) -> Type {
    // Sparse values are typed `Type::Tensor` in RunMat, and non-sparse inputs
    // pass through unchanged, so the output type mirrors the input type.
    args.first().cloned().unwrap_or_else(Type::tensor)
}

fn internal_error(detail: String) -> RuntimeError {
    build_runtime_error(format!("full: {detail}"))
        .with_builtin("full")
        .with_identifier(FULL_ERROR_INTERNAL.identifier.expect("identifier"))
        .build()
}

#[cfg(all(test, feature = "parallel_xval"))]
pub(crate) mod tests {
    use super::*;
    use futures::executor::block_on;
    use runmat_builtins::{SparseTensor, Tensor};

    fn run_full(value: Value) -> Value {
        block_on(super::full_builtin(value)).expect("full")
    }

    fn expect_tensor(value: Value) -> Tensor {
        match value {
            Value::Tensor(t) => t,
            other => panic!("expected dense tensor, got {other:?}"),
        }
    }

    // Normative [FR-009-01; full.full-preserves-values, full.output-class] —
    // observed case `from-sparse-matrix`: full(sparse([1 0; 0 2])) =>
    // [1 0;0 2] (2x2 double, issparse=false).
    #[test]
    fn observed_sparse_matrix_to_dense() {
        let sparse = SparseTensor::new(2, 2, vec![0, 1, 2], vec![0, 1], vec![1.0, 2.0]).unwrap();
        let dense = expect_tensor(run_full(Value::SparseTensor(sparse)));
        assert_eq!(dense.shape, vec![2, 2]);
        // Column-major layout of [1 0;0 2].
        assert_eq!(dense.data, vec![1.0, 0.0, 0.0, 2.0]);
    }

    // Normative — observed case `from-sparse-vector`: full(sparse([0 3 0]))
    // => [0 3 0] (1x3 double).
    #[test]
    fn observed_sparse_vector_to_dense() {
        let sparse = SparseTensor::new(1, 3, vec![0, 0, 1, 1], vec![0], vec![3.0]).unwrap();
        let dense = expect_tensor(run_full(Value::SparseTensor(sparse)));
        assert_eq!(dense.shape, vec![1, 3]);
        assert_eq!(dense.data, vec![0.0, 3.0, 0.0]);
    }

    // Normative — observed case `already-full`: full([1 2; 3 4]) =>
    // [1 2;3 4] unchanged (2x2 double).
    #[test]
    fn observed_dense_input_passes_through() {
        // Column-major data for [1 2; 3 4].
        let dense = Tensor::new(vec![1.0, 3.0, 2.0, 4.0], vec![2, 2]).unwrap();
        let result = run_full(Value::Tensor(dense.clone()));
        assert_eq!(result, Value::Tensor(dense));
    }

    // Normative — observed case `empty-sparse`: full(sparse([])) => []
    // (0x0 double).
    #[test]
    fn observed_empty_sparse_to_empty_dense() {
        let dense = expect_tensor(run_full(Value::SparseTensor(SparseTensor::zeros(0, 0))));
        assert_eq!(dense.shape, vec![0, 0]);
        assert!(dense.data.is_empty());
    }

    // unresolved_choice [FR-009-03; full.q-class]: non-sparse kinds beyond the
    // observed dense double matrix pass through unchanged (documented
    // independent choice; not asserted as MATLAB-conformant).
    #[test]
    fn unresolved_choice_other_full_kinds_pass_through() {
        assert_eq!(run_full(Value::Num(5.0)), Value::Num(5.0));
        assert_eq!(run_full(Value::Bool(true)), Value::Bool(true));
    }

    #[test]
    fn internal_error_on_dimension_overflow() {
        let sparse = SparseTensor {
            rows: usize::MAX,
            cols: 2,
            col_ptrs: vec![0, 0, 0],
            row_indices: Vec::new(),
            values: Vec::new(),
        };
        let err = block_on(super::full_builtin(Value::SparseTensor(sparse))).expect_err("error");
        assert_eq!(err.identifier(), Some("RunMat:full:Internal"));
    }
}
