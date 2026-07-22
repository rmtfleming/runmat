//! MATLAB-compatible `nonzeros` builtin for RunMat.
//!
//! Clean-room provenance: specs/017-nonzeros (spec `nonzeros` 0.2.0,
//! claims nonzeros.signature-primary, nonzeros.output-class,
//! nonzeros.column-major-order).
//!
//! Returns the nonzero elements of the input as a double column vector in
//! column-major order. Dense tensors are traversed in storage order (RunMat
//! tensors are column-major); sparse CSC storage already enumerates values
//! column-by-column with rows sorted, so its value array is read in order.
//! Behaviour on inputs beyond the observed dense-double and sparse cases
//! (empty result shape, scalars, logical, complex, NaN, N-D) follows
//! documented independent choices — see specs/017-nonzeros/spec.md.

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    ComplexTensor, ResolveContext, Tensor, Type, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::array::type_resolvers::column_vector_type;
use crate::builtins::common::random_args::complex_tensor_into_value;
use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ScalarType, ShapeRequirements,
};
use crate::builtins::common::{gpu_helpers, tensor};
use crate::{build_runtime_error, BuiltinResult, RuntimeError};

use super::scalar_f64;

#[runmat_macros::register_gpu_spec(builtin_path = "crate::builtins::math::sparse::nonzeros")]
pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "nonzeros",
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
    notes: "Sparse values are host-resident CSC; GPU tensors are gathered before the host-side nonzero scan.",
};

#[runmat_macros::register_fusion_spec(builtin_path = "crate::builtins::math::sparse::nonzeros")]
pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "nonzeros",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Data-dependent output length (compaction); not fusible.",
};

const NONZEROS_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "v",
    ty: BuiltinParamType::NumericArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Column vector of the nonzero elements in column-major order.",
}];

const NONZEROS_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "A",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Numeric array (full or sparse).",
}];

const NONZEROS_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "v = nonzeros(A)",
    inputs: &NONZEROS_INPUTS,
    outputs: &NONZEROS_OUTPUT,
}];

const NONZEROS_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.NONZEROS.INVALID_INPUT",
    identifier: Some("RunMat:nonzeros:InvalidInput"),
    when: "The input is not a numeric, logical, or sparse array.",
    message: "nonzeros: input must be a numeric array (full or sparse)",
};

const NONZEROS_ERROR_INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.NONZEROS.INTERNAL",
    identifier: Some("RunMat:nonzeros:Internal"),
    when: "Result materialisation fails internally.",
    message: "nonzeros: internal error",
};

const NONZEROS_ERRORS: [BuiltinErrorDescriptor; 2] =
    [NONZEROS_ERROR_INVALID_INPUT, NONZEROS_ERROR_INTERNAL];

pub const NONZEROS_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &NONZEROS_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &NONZEROS_ERRORS,
};

#[runtime_builtin(
    name = "nonzeros",
    category = "math/sparse",
    summary = "Return the nonzero elements of an array as a column vector in column-major order.",
    keywords = "nonzeros,nonzero,sparse,column,vector,find",
    accel = "custom",
    type_resolver(nonzeros_type),
    descriptor(crate::builtins::math::sparse::nonzeros::NONZEROS_DESCRIPTOR),
    builtin_path = "crate::builtins::math::sparse::nonzeros"
)]
async fn nonzeros_builtin(value: Value) -> BuiltinResult<Value> {
    let gathered = gpu_helpers::gather_value_async(&value).await?;
    match gathered {
        // Dense data is stored column-major; retaining nonzero entries in
        // storage order yields the observed column-major result
        // [nonzeros.column-major-order]. N-D tensors traverse the same
        // storage order (documented choice; only 2-D inputs are observed).
        Value::Tensor(t) => real_column_result(t.data.iter().copied().filter(|&v| is_nonzero(v))),
        // CSC storage enumerates columns in order with rows sorted inside
        // each column, so the value array is already in column-major order
        // [nonzeros.column-major-order]. Explicitly stored zeros are skipped
        // defensively (documented choice; MATLAB sparse does not store them).
        Value::SparseTensor(sparse) => {
            real_column_result(sparse.values.iter().copied().filter(|&v| is_nonzero(v)))
        }
        // Logical arrays are unobserved input classes: accepted and returned
        // as double, consistent with nonzeros.output-class on observed
        // inputs (documented choice).
        Value::LogicalArray(logical) => {
            real_column_result(logical.data.iter().filter(|&&b| b != 0).map(|_| 1.0))
        }
        // Complex inputs are unobserved: kept complex, an element counting
        // as nonzero when its real or imaginary part is nonzero or NaN
        // (documented choice).
        Value::ComplexTensor(ct) => complex_column_result(
            ct.data
                .iter()
                .copied()
                .filter(|&(re, im)| is_nonzero(re) || is_nonzero(im))
                .collect(),
        ),
        Value::Complex(re, im) => complex_column_result(if is_nonzero(re) || is_nonzero(im) {
            vec![(re, im)]
        } else {
            Vec::new()
        }),
        // Real scalar-like inputs (Num, Int, Bool, 1x1 tensors) are treated
        // as 1x1 arrays (documented choice; scalars are unobserved).
        ref scalar if scalar_f64(scalar).is_some() => {
            let v = scalar_f64(scalar).expect("scalar");
            real_column_result(if is_nonzero(v) { Some(v) } else { None }.into_iter())
        }
        other => Err(invalid_input(format!(
            "nonzeros: input must be a numeric array (full or sparse), got {other:?}"
        ))),
    }
}

fn nonzeros_type(_args: &[Type], _context: &ResolveContext) -> Type {
    column_vector_type()
}

/// An element is kept when it differs from zero; NaN counts as nonzero
/// (documented choice; NaN inputs are unobserved). Negative zero compares
/// equal to zero and is dropped.
fn is_nonzero(value: f64) -> bool {
    value.is_nan() || value != 0.0
}

/// Materialise kept real elements as an n-by-1 double column vector. A single
/// element becomes `Value::Num` (RunMat's 1x1 double, matching the observed
/// 1x1 sparse case); no elements yield a 0x1 empty double (documented choice;
/// the all-zero case is unobserved).
fn real_column_result(kept: impl Iterator<Item = f64>) -> BuiltinResult<Value> {
    let data: Vec<f64> = kept.collect();
    let len = data.len();
    let column = Tensor::new(data, vec![len, 1]).map_err(internal_error)?;
    Ok(tensor::tensor_into_value(column))
}

/// Complex analogue of `real_column_result`; an empty result falls back to a
/// 0x1 empty double (documented choice).
fn complex_column_result(kept: Vec<(f64, f64)>) -> BuiltinResult<Value> {
    if kept.is_empty() {
        return real_column_result(std::iter::empty());
    }
    let len = kept.len();
    let column = ComplexTensor::new(kept, vec![len, 1]).map_err(internal_error)?;
    Ok(complex_tensor_into_value(column))
}

fn invalid_input(message: String) -> RuntimeError {
    build_runtime_error(message)
        .with_builtin("nonzeros")
        .with_identifier(NONZEROS_ERROR_INVALID_INPUT.identifier.expect("identifier"))
        .build()
}

fn internal_error(detail: String) -> RuntimeError {
    build_runtime_error(format!("nonzeros: {detail}"))
        .with_builtin("nonzeros")
        .with_identifier(NONZEROS_ERROR_INTERNAL.identifier.expect("identifier"))
        .build()
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use futures::executor::block_on;
    use runmat_builtins::{CharArray, LogicalArray, SparseTensor};

    fn run_nonzeros(value: Value) -> Value {
        block_on(super::nonzeros_builtin(value)).expect("nonzeros")
    }

    fn tensor(data: Vec<f64>, shape: Vec<usize>) -> Value {
        Value::Tensor(Tensor::new(data, shape).unwrap())
    }

    fn expect_tensor(value: Value) -> Tensor {
        match value {
            Value::Tensor(t) => t,
            other => panic!("expected dense tensor, got {other:?}"),
        }
    }

    // Normative [FR-017-01..03; nonzeros.signature-primary,
    // nonzeros.output-class, nonzeros.column-major-order] — observed case
    // `row`: nonzeros([0 1 0 2]) => [1;2] (2x1 double).
    #[test]
    fn observed_row_vector_nonzeros() {
        let v = expect_tensor(run_nonzeros(tensor(vec![0.0, 1.0, 0.0, 2.0], vec![1, 4])));
        assert_eq!(v.shape, vec![2, 1]);
        assert_eq!(v.data, vec![1.0, 2.0]);
    }

    // Normative [FR-017-02, FR-017-03; nonzeros.output-class,
    // nonzeros.column-major-order] — observed case `matrix`:
    // nonzeros([1 2; 0 4]) => [1;2;4] (3x1 double).
    #[test]
    fn observed_matrix_nonzeros() {
        // Column-major data for [1 2; 0 4].
        let v = expect_tensor(run_nonzeros(tensor(vec![1.0, 0.0, 2.0, 4.0], vec![2, 2])));
        assert_eq!(v.shape, vec![3, 1]);
        assert_eq!(v.data, vec![1.0, 2.0, 4.0]);
    }

    // Normative [FR-017-03; nonzeros.column-major-order] — observed case
    // `col-major`: nonzeros([1 2; 3 0]) => [1;3;2] (3x1 double). Column-major
    // traversal gives 1,3,2,0; a row-major traversal would give [1;2;3], so
    // this case discriminates the two orderings.
    #[test]
    fn observed_discriminating_column_major_order() {
        // Column-major data for [1 2; 3 0].
        let v = expect_tensor(run_nonzeros(tensor(vec![1.0, 3.0, 2.0, 0.0], vec![2, 2])));
        assert_eq!(v.shape, vec![3, 1]);
        assert_eq!(v.data, vec![1.0, 3.0, 2.0]);
    }

    // Normative [FR-017-02, FR-017-03; nonzeros.output-class,
    // nonzeros.column-major-order] — observed case `sparse`:
    // nonzeros(sparse([0 3 0])) => 3 (1x1 double, issparse=false).
    // `Value::Num` is RunMat's 1x1 double.
    #[test]
    fn observed_sparse_vector_nonzeros() {
        let sparse = SparseTensor::new(1, 3, vec![0, 0, 1, 1], vec![0], vec![3.0]).unwrap();
        assert_eq!(run_nonzeros(Value::SparseTensor(sparse)), Value::Num(3.0));
    }

    // summary_derived [FR-017-03; nonzeros.column-major-order]: the approved
    // rule statement says the column-major order holds for sparse inputs as
    // well; the observed sparse case has a single nonzero, so a multi-column
    // sparse matrix is exercised here without an exact backing observation.
    // Sparse form of [1 2; 3 0] => [1;3;2].
    #[test]
    fn summary_derived_sparse_matrix_column_major() {
        let sparse =
            SparseTensor::new(2, 2, vec![0, 2, 3], vec![0, 1, 0], vec![1.0, 3.0, 2.0]).unwrap();
        let v = expect_tensor(run_nonzeros(Value::SparseTensor(sparse)));
        assert_eq!(v.shape, vec![3, 1]);
        assert_eq!(v.data, vec![1.0, 3.0, 2.0]);
    }

    // unresolved_choice [FR-017-04]: an input with no nonzero elements yields
    // a 0x1 empty double (documented choice; unobserved).
    #[test]
    fn unresolved_choice_all_zero_input_yields_0x1_empty() {
        let v = expect_tensor(run_nonzeros(tensor(vec![0.0, 0.0, 0.0, 0.0], vec![2, 2])));
        assert_eq!(v.shape, vec![0, 1]);
        assert!(v.data.is_empty());

        let sparse = expect_tensor(run_nonzeros(Value::SparseTensor(SparseTensor::zeros(2, 2))));
        assert_eq!(sparse.shape, vec![0, 1]);
        assert!(sparse.data.is_empty());
    }

    // unresolved_choice [FR-017-05]: real scalar-like inputs are treated as
    // 1x1 arrays; logical scalars return double (documented choices).
    #[test]
    fn unresolved_choice_scalar_inputs() {
        assert_eq!(run_nonzeros(Value::Num(5.0)), Value::Num(5.0));
        assert_eq!(run_nonzeros(Value::Bool(true)), Value::Num(1.0));
        let empty = expect_tensor(run_nonzeros(Value::Num(0.0)));
        assert_eq!(empty.shape, vec![0, 1]);
    }

    // unresolved_choice [FR-017-05]: NaN counts as nonzero and negative zero
    // as zero (documented choices; both unobserved).
    #[test]
    fn unresolved_choice_nan_kept_negative_zero_dropped() {
        let v = expect_tensor(run_nonzeros(tensor(
            vec![0.0, f64::NAN, -0.0, 2.0],
            vec![1, 4],
        )));
        assert_eq!(v.shape, vec![2, 1]);
        assert!(v.data[0].is_nan());
        assert_eq!(v.data[1], 2.0);
    }

    // unresolved_choice [FR-017-05]: logical arrays are accepted and return
    // double values (documented choice; unobserved input class).
    #[test]
    fn unresolved_choice_logical_array_returns_double() {
        let logical = LogicalArray::new(vec![0, 1, 1, 0], vec![2, 2]).unwrap();
        let v = expect_tensor(run_nonzeros(Value::LogicalArray(logical)));
        assert_eq!(v.shape, vec![2, 1]);
        assert_eq!(v.data, vec![1.0, 1.0]);
    }

    // unresolved_choice [FR-017-05]: complex inputs keep their complex values
    // in column-major order; an element is nonzero when either part is
    // (documented choice; unobserved input class).
    #[test]
    fn unresolved_choice_complex_nonzeros_column_major() {
        let ct = ComplexTensor::new(vec![(0.0, 0.0), (1.0, 2.0), (0.0, -3.0)], vec![1, 3]).unwrap();
        match run_nonzeros(Value::ComplexTensor(ct)) {
            Value::ComplexTensor(v) => {
                assert_eq!(v.shape, vec![2, 1]);
                assert_eq!(v.data, vec![(1.0, 2.0), (0.0, -3.0)]);
            }
            other => panic!("expected complex tensor, got {other:?}"),
        }
        assert_eq!(
            run_nonzeros(Value::Complex(0.0, 4.0)),
            Value::Complex(0.0, 4.0)
        );
        let empty = expect_tensor(run_nonzeros(Value::Complex(0.0, 0.0)));
        assert_eq!(empty.shape, vec![0, 1]);
    }

    // unresolved_choice [FR-017-05]: N-D tensors traverse the same
    // column-major storage order (documented choice; unobserved).
    #[test]
    fn unresolved_choice_nd_input_column_major() {
        let v = expect_tensor(run_nonzeros(tensor(
            vec![0.0, 1.0, 2.0, 0.0, 0.0, 3.0, 4.0, 0.0],
            vec![2, 2, 2],
        )));
        assert_eq!(v.shape, vec![4, 1]);
        assert_eq!(v.data, vec![1.0, 2.0, 3.0, 4.0]);
    }

    // unresolved_choice [FR-017-05]: non-numeric inputs are rejected with the
    // stable InvalidInput identifier (documented choice; unobserved).
    #[test]
    fn unresolved_choice_non_numeric_input_errors() {
        let err = block_on(super::nonzeros_builtin(Value::CharArray(
            CharArray::new_row("abc"),
        )))
        .expect_err("error");
        assert_eq!(err.identifier(), Some("RunMat:nonzeros:InvalidInput"));
    }

    #[test]
    fn nonzeros_type_is_column_vector() {
        assert_eq!(
            nonzeros_type(
                &[Type::Tensor { shape: None }],
                &ResolveContext::new(Vec::new()),
            ),
            Type::Tensor {
                shape: Some(vec![None, Some(1)])
            }
        );
    }
}
