//! MATLAB-compatible `spalloc` builtin for RunMat.
//!
//! Clean-room provenance: specs/023-spalloc (spec `spalloc` 0.2.0, claims
//! spalloc.signature-primary, spalloc.output-class, spalloc.all-zero-sparse).
//!
//! `spalloc(m, n, nz)` preallocates an all-zero `m`-by-`n` sparse matrix; `nz`
//! is a capacity hint only. RunMat stores sparse matrices in compressed-sparse-
//! column (CSC) form with no separate capacity, so the hint does not change the
//! observable result (`nnz` stays 0). See specs/023-spalloc for the documented
//! independent choices carried as unresolved questions.

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    ResolveContext, SparseTensor, Type, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ScalarType, ShapeRequirements,
};
use crate::{build_runtime_error, BuiltinResult, RuntimeError};

use super::{scalar_f64, size_from_f64};

#[runmat_macros::register_gpu_spec(builtin_path = "crate::builtins::math::sparse::spalloc")]
pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "spalloc",
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
    notes: "Creates a host-resident empty CSC matrix; no GPU execution path.",
};

#[runmat_macros::register_fusion_spec(builtin_path = "crate::builtins::math::sparse::spalloc")]
pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "spalloc",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Sparse constructor; not fusible.",
};

const SPALLOC_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "S",
    ty: BuiltinParamType::NumericArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "All-zero sparse matrix of the requested size.",
}];

const SPALLOC_INPUTS: [BuiltinParamDescriptor; 3] = [
    BuiltinParamDescriptor {
        name: "m",
        ty: BuiltinParamType::SizeArg,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Number of rows.",
    },
    BuiltinParamDescriptor {
        name: "n",
        ty: BuiltinParamType::SizeArg,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Number of columns.",
    },
    BuiltinParamDescriptor {
        name: "nz",
        ty: BuiltinParamType::IntegerScalar,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Reserved nonzero capacity (hint only; does not change the result).",
    },
];

const SPALLOC_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "S = spalloc(m, n, nz)",
    inputs: &SPALLOC_INPUTS,
    outputs: &SPALLOC_OUTPUT,
}];

const SPALLOC_ERROR_INVALID_ARGUMENT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SPALLOC.INVALID_ARGUMENT",
    identifier: Some("RunMat:spalloc:InvalidArgument"),
    when: "The call is not spalloc(m, n, nz) with nonnegative-integer arguments.",
    message: "spalloc: expected spalloc(m, n, nz) with nonnegative integer arguments",
};

const SPALLOC_ERRORS: [BuiltinErrorDescriptor; 1] = [SPALLOC_ERROR_INVALID_ARGUMENT];

pub const SPALLOC_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &SPALLOC_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &SPALLOC_ERRORS,
};

#[runtime_builtin(
    name = "spalloc",
    category = "math/sparse",
    summary = "Preallocate an all-zero sparse matrix of a given size (nz is a capacity hint).",
    keywords = "spalloc,sparse,allocate,preallocate,zeros",
    accel = "array_construct",
    type_resolver(spalloc_type),
    descriptor(crate::builtins::math::sparse::spalloc::SPALLOC_DESCRIPTOR),
    builtin_path = "crate::builtins::math::sparse::spalloc"
)]
async fn spalloc_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    // Confirmed invocation form: S = spalloc(m, n, nz) [spalloc.signature-primary].
    // Exactly three arguments are required; other arities are rejected as a
    // documented independent choice (unobserved) [spalloc.q-arity].
    if args.len() != 3 {
        return Err(invalid_argument());
    }
    let rows = parse_dim(&args[0], "m")?;
    let cols = parse_dim(&args[1], "n")?;
    // `nz` is a capacity hint only. RunMat's CSC representation has no separate
    // reserved capacity, so the hint cannot change the observable result
    // (nnz stays 0). It is still validated as a nonnegative integer for
    // consistency with the dimensions — a documented independent choice, since
    // the export observed only nz = 4 [spalloc.q-nz-capacity].
    let _nz = parse_dim(&args[2], "nz")?;
    // Returns an all-zero sparse matrix of the requested size (issparse is
    // true; nnz == 0) [spalloc.all-zero-sparse, spalloc.output-class].
    Ok(Value::SparseTensor(SparseTensor::zeros(rows, cols)))
}

fn spalloc_type(_args: &[Type], _context: &ResolveContext) -> Type {
    // Sparse values are typed `Type::Tensor` in RunMat's value model.
    Type::tensor()
}

/// Parse one nonnegative-integer argument from a scalar-like value. Non-integer,
/// negative, non-finite, and non-scalar inputs are rejected (documented
/// independent choice; the export observed only the integer 3-by-3-with-4 form).
fn parse_dim(value: &Value, name: &str) -> BuiltinResult<usize> {
    let raw = scalar_f64(value).ok_or_else(invalid_argument)?;
    size_from_f64(raw, name).map_err(invalid_argument_detail)
}

fn invalid_argument() -> RuntimeError {
    build_runtime_error(SPALLOC_ERROR_INVALID_ARGUMENT.message)
        .with_builtin("spalloc")
        .with_identifier(
            SPALLOC_ERROR_INVALID_ARGUMENT
                .identifier
                .expect("identifier"),
        )
        .build()
}

fn invalid_argument_detail(detail: String) -> RuntimeError {
    build_runtime_error(format!("spalloc: {detail}"))
        .with_builtin("spalloc")
        .with_identifier(
            SPALLOC_ERROR_INVALID_ARGUMENT
                .identifier
                .expect("identifier"),
        )
        .build()
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use futures::executor::block_on;
    use runmat_builtins::{IntValue, Tensor};

    fn run_spalloc(args: Vec<Value>) -> Value {
        block_on(super::spalloc_builtin(args)).expect("spalloc")
    }

    fn expect_sparse(value: Value) -> SparseTensor {
        match value {
            Value::SparseTensor(sparse) => sparse,
            other => panic!("expected sparse tensor, got {other:?}"),
        }
    }

    // Observed tier — normative, reproduces the approved export case exactly.
    //
    // Export case `basic`: spalloc(3, 3, 4) =>
    // sparse(zeros(0,1), zeros(0,1), zeros(0,1), 3, 3): a 3x3 double sparse
    // matrix, issparse=true, with no stored nonzeros (nnz == 0).
    // [spalloc.signature-primary, spalloc.output-class, spalloc.all-zero-sparse;
    // FR-023-01, FR-023-02, FR-023-03]
    #[test]
    fn observed_basic_all_zero_sparse() {
        let sparse = expect_sparse(run_spalloc(vec![
            Value::Num(3.0),
            Value::Num(3.0),
            Value::Num(4.0),
        ]));
        assert_eq!(sparse, SparseTensor::zeros(3, 3));
        assert_eq!(sparse.shape(), vec![3, 3]);
        assert_eq!(sparse.nnz(), 0);
    }

    // Unresolved-choice tier — documented independent choices, not asserted as
    // MATLAB-conformant. The export recorded a single case (spalloc(3,3,4)) and
    // no unresolved questions, so everything below is a RunMat-side choice.

    // [spalloc.q-nz-capacity; FR-023-04] The nz hint does not change the
    // observable content: different (valid) nz values yield identical results.
    #[test]
    fn unresolved_choice_nz_capacity_does_not_change_result() {
        let baseline = SparseTensor::zeros(3, 3);
        for nz in [0.0, 4.0, 100.0] {
            let sparse = expect_sparse(run_spalloc(vec![
                Value::Num(3.0),
                Value::Num(3.0),
                Value::Num(nz),
            ]));
            assert_eq!(sparse, baseline);
            assert_eq!(sparse.nnz(), 0);
        }
    }

    // [spalloc.q-dims; FR-023-04] Rectangular and empty sizes beyond the single
    // observed 3x3 case follow the same all-zero rule.
    #[test]
    fn unresolved_choice_rectangular_and_empty_sizes() {
        let rect = expect_sparse(run_spalloc(vec![
            Value::Num(2.0),
            Value::Num(5.0),
            Value::Num(3.0),
        ]));
        assert_eq!(rect, SparseTensor::zeros(2, 5));
        assert_eq!(rect.shape(), vec![2, 5]);

        let empty = expect_sparse(run_spalloc(vec![
            Value::Num(0.0),
            Value::Num(0.0),
            Value::Num(0.0),
        ]));
        assert_eq!(empty, SparseTensor::zeros(0, 0));
        assert_eq!(empty.nnz(), 0);
    }

    // [spalloc.q-dims; FR-023-04] Dimensions and the capacity hint are accepted
    // from the same scalar-like kinds RunMat's other sparse constructors accept
    // (Int, Bool, 1-element tensor), mirroring speye.
    #[test]
    fn unresolved_choice_scalar_like_argument_kinds() {
        let from_int = expect_sparse(run_spalloc(vec![
            Value::Int(IntValue::I32(3)),
            Value::Int(IntValue::I32(2)),
            Value::Int(IntValue::I32(6)),
        ]));
        assert_eq!(from_int, SparseTensor::zeros(3, 2));

        let one = Tensor::new(vec![1.0], vec![1, 1]).unwrap();
        let from_mixed = expect_sparse(run_spalloc(vec![
            Value::Bool(true),
            Value::Tensor(one),
            Value::Num(0.0),
        ]));
        assert_eq!(from_mixed, SparseTensor::zeros(1, 1));
    }

    // [spalloc.q-invalid-dims; FR-023-05] Negative, non-integer, and non-scalar
    // dimensions are rejected (documented choice; unobserved input classes).
    #[test]
    fn unresolved_choice_invalid_dimensions_error() {
        let neg = block_on(super::spalloc_builtin(vec![
            Value::Num(-1.0),
            Value::Num(3.0),
            Value::Num(4.0),
        ]))
        .expect_err("negative rows");
        assert_eq!(neg.identifier(), Some("RunMat:spalloc:InvalidArgument"));

        let frac = block_on(super::spalloc_builtin(vec![
            Value::Num(3.0),
            Value::Num(2.5),
            Value::Num(4.0),
        ]))
        .expect_err("non-integer cols");
        assert_eq!(frac.identifier(), Some("RunMat:spalloc:InvalidArgument"));

        let vec_dim = Tensor::new(vec![3.0, 3.0], vec![1, 2]).unwrap();
        let non_scalar = block_on(super::spalloc_builtin(vec![
            Value::Tensor(vec_dim),
            Value::Num(3.0),
            Value::Num(4.0),
        ]))
        .expect_err("non-scalar rows");
        assert_eq!(
            non_scalar.identifier(),
            Some("RunMat:spalloc:InvalidArgument")
        );
    }

    // [spalloc.q-nz-capacity; FR-023-05] The capacity hint is validated the same
    // way as the dimensions: negative and non-integer nz are rejected.
    #[test]
    fn unresolved_choice_invalid_nz_error() {
        let neg = block_on(super::spalloc_builtin(vec![
            Value::Num(3.0),
            Value::Num(3.0),
            Value::Num(-4.0),
        ]))
        .expect_err("negative nz");
        assert_eq!(neg.identifier(), Some("RunMat:spalloc:InvalidArgument"));

        let frac = block_on(super::spalloc_builtin(vec![
            Value::Num(3.0),
            Value::Num(3.0),
            Value::Num(1.5),
        ]))
        .expect_err("non-integer nz");
        assert_eq!(frac.identifier(), Some("RunMat:spalloc:InvalidArgument"));
    }

    // [spalloc.q-arity; FR-023-05] Only the three-argument form is accepted;
    // fewer (missing nz) or more arguments are rejected (documented choice).
    #[test]
    fn unresolved_choice_wrong_argument_count_error() {
        let missing_nz = block_on(super::spalloc_builtin(vec![
            Value::Num(3.0),
            Value::Num(3.0),
        ]))
        .expect_err("missing nz");
        assert_eq!(
            missing_nz.identifier(),
            Some("RunMat:spalloc:InvalidArgument")
        );

        let too_many = block_on(super::spalloc_builtin(vec![
            Value::Num(3.0),
            Value::Num(3.0),
            Value::Num(4.0),
            Value::Num(5.0),
        ]))
        .expect_err("too many args");
        assert_eq!(
            too_many.identifier(),
            Some("RunMat:spalloc:InvalidArgument")
        );
    }

    #[test]
    fn spalloc_type_is_tensor() {
        assert_eq!(
            spalloc_type(&[], &ResolveContext::new(Vec::new())),
            Type::tensor()
        );
    }
}
