//! Clean-room `orth` (orthonormal basis for the range of a matrix).
//!
//! Clean-room feature `specs/055-numeric-n3n4/`. Normative behaviour comes
//! only from the approved export `orth` 0.2.0 (see that feature's
//! `provenance.md`): `size(A,1)`-by-rank output with orthonormal columns
//! spanning range(A), class kept, empty shapes, the second-argument
//! threshold case, and the `svd` error identifiers (raised as `RunMat:*`,
//! D-055-1). The basis itself is not unique (`orth.basis-not-unique`,
//! non-normative), so only size, orthonormality and span are required.
//!
//! Algorithm (textbook numerical linear algebra, e.g. Golub & Van Loan,
//! "Matrix Computations", ch. 2 and 5): the leading left singular vectors of
//! a thin SVD (nalgebra), one per singular value above the rank threshold.

use nalgebra::{linalg::SVD, DMatrix};
use num_complex::Complex64;
use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{ComplexTensor, NumericDType, Tensor, Value};

use crate::builtins::common::map_control_flow_with_builtin;
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const NAME: &str = "orth";

// ---------------------------------------------------------------------------
// Descriptor
// ---------------------------------------------------------------------------

const fn param(
    name: &'static str,
    arity: BuiltinParamArity,
    description: &'static str,
) -> BuiltinParamDescriptor {
    BuiltinParamDescriptor {
        name,
        ty: BuiltinParamType::Any,
        arity,
        default: None,
        description,
    }
}

const IN_A: BuiltinParamDescriptor = param(
    "A",
    BuiltinParamArity::Required,
    "Full double or single matrix, real or complex.",
);
const IN_T: BuiltinParamDescriptor = param(
    "t",
    BuiltinParamArity::Required,
    "Singular-value threshold (documented choice: absolute).",
);
const OUT_Q: BuiltinParamDescriptor = param(
    "Q",
    BuiltinParamArity::Required,
    "Orthonormal basis of range(A), size(A,1)-by-rank.",
);

const ORTH_INPUTS_A: [BuiltinParamDescriptor; 1] = [IN_A];
const ORTH_INPUTS_A_T: [BuiltinParamDescriptor; 2] = [IN_A, IN_T];
const ORTH_OUTPUTS: [BuiltinParamDescriptor; 1] = [OUT_Q];

const ORTH_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "Q = orth(A)",
        inputs: &ORTH_INPUTS_A,
        outputs: &ORTH_OUTPUTS,
    },
    BuiltinSignatureDescriptor {
        label: "Q = orth(A, t)",
        inputs: &ORTH_INPUTS_A_T,
        outputs: &ORTH_OUTPUTS,
    },
];

const NO_SVD_SPARSE: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ORTH.NO_SVD_SPARSE",
    identifier: Some("RunMat:svd:noSVDsparse"),
    when: "A is sparse.",
    message: "orth: A must be a full matrix",
};

const INPUT_TYPE: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ORTH.INPUT_TYPE",
    identifier: Some("RunMat:svd:inputType"),
    when: "A is an integer or char array.",
    message: "orth: A must be double or single",
};

const INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ORTH.INVALID_INPUT",
    identifier: Some("RunMat:orth:InvalidInput"),
    when: "The call does not have one or two inputs, or A is not 2-D.",
    message: "orth: expected orth(A) or orth(A, t)",
};

const UNSUPPORTED_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ORTH.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:orth:UnsupportedOption"),
    when: "Unobserved inputs: logical A, non-finite A, a t that is not a finite non-negative real scalar, other classes.",
    message: "orth: this form is not yet specified",
};

const TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ORTH.TOO_MANY_OUTPUTS",
    identifier: Some("RunMat:orth:TooManyOutputs"),
    when: "More than one output was requested.",
    message: "orth: too many output arguments",
};

const INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ORTH.INTERNAL",
    identifier: Some("RunMat:orth:Internal"),
    when: "The runtime cannot assemble the result.",
    message: "orth: internal runtime failure",
};

const ORTH_ERRORS: [BuiltinErrorDescriptor; 6] = [
    NO_SVD_SPARSE,
    INPUT_TYPE,
    INVALID_INPUT,
    UNSUPPORTED_OPTION,
    TOO_MANY_OUTPUTS,
    INTERNAL,
];

pub const ORTH_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &ORTH_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ORTH_ERRORS,
};

fn orth_error(
    descriptor: &'static BuiltinErrorDescriptor,
    detail: impl Into<String>,
) -> RuntimeError {
    let mut builder = build_runtime_error(format!("{NAME}: {}", detail.into())).with_builtin(NAME);
    if let Some(identifier) = descriptor.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

// ---------------------------------------------------------------------------
// Builtin
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "orth",
    category = "math/linalg/factor",
    summary = "Orthonormal basis for the range (column space) of a matrix.",
    keywords = "orth,orthonormal basis,range,column space,svd,rank",
    accel = "cpu",
    descriptor(crate::builtins::math::linalg::factor::orth::ORTH_DESCRIPTOR),
    builtin_path = "crate::builtins::math::linalg::factor::orth"
)]
pub(crate) async fn orth_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    // FR-055-29
    if let Some(count) = crate::output_count::current_output_count() {
        if count > 1 {
            return Err(orth_error(
                &TOO_MANY_OUTPUTS,
                format!("orth returns one output, {count} were requested"),
            ));
        }
    }
    if args.is_empty() || args.len() > 2 {
        return Err(orth_error(&INVALID_INPUT, "expected orth(A) or orth(A, t)"));
    }
    // FR-055-31: gpuArray input is gathered; the result is a host array.
    let mut host = Vec::with_capacity(args.len());
    for value in &args {
        host.push(
            gather_if_needed_async(value)
                .await
                .map_err(|err| map_control_flow_with_builtin(err, NAME))?,
        );
    }
    let input = dense_input(&host[0])?;
    let threshold = match host.get(1) {
        None => None,
        Some(t) => Some(parse_threshold(t)?),
    };
    basis(input, threshold)
}

struct DenseInput {
    data: Vec<Complex64>,
    rows: usize,
    cols: usize,
    single: bool,
    complex: bool,
}

fn input_type() -> RuntimeError {
    orth_error(
        &INPUT_TYPE,
        "A must be double or single, not integer or char",
    )
}

fn shape2(shape: &[usize]) -> BuiltinResult<(usize, usize)> {
    if shape.iter().skip(2).any(|&dim| dim != 1) {
        return Err(orth_error(&INVALID_INPUT, "A must be a 2-D matrix"));
    }
    Ok((
        shape.first().copied().unwrap_or(1),
        shape.get(1).copied().unwrap_or(1),
    ))
}

/// FR-055-11, FR-055-15, FR-055-16, FR-055-30.
fn dense_input(a: &Value) -> BuiltinResult<DenseInput> {
    let (data, shape, single, complex): (Vec<Complex64>, Vec<usize>, bool, bool) = match a {
        Value::Num(v) => (vec![Complex64::new(*v, 0.0)], vec![1, 1], false, false),
        Value::Tensor(t) => {
            if t.integer_storage().is_some() {
                return Err(input_type());
            }
            let single = match t.numeric_dtype() {
                NumericDType::F64 => false,
                NumericDType::F32 => true,
                _ => return Err(input_type()),
            };
            (
                t.materialize_f64()
                    .into_iter()
                    .map(|v| Complex64::new(v, 0.0))
                    .collect(),
                t.shape.clone(),
                single,
                false,
            )
        }
        Value::Complex(re, im) => (vec![Complex64::new(*re, *im)], vec![1, 1], false, true),
        Value::ComplexTensor(t) => {
            if t.integer_storage().is_some() {
                return Err(input_type());
            }
            let single = match t.numeric_dtype() {
                NumericDType::F64 => false,
                NumericDType::F32 => true,
                _ => return Err(input_type()),
            };
            (
                t.materialize_f64()
                    .into_iter()
                    .map(|(re, im)| Complex64::new(re, im))
                    .collect(),
                t.shape.clone(),
                single,
                true,
            )
        }
        Value::SparseTensor(_) => {
            return Err(orth_error(&NO_SVD_SPARSE, "A must be a full matrix"))
        }
        Value::Int(_) | Value::CharArray(_) => return Err(input_type()),
        _ => {
            return Err(orth_error(
                &UNSUPPORTED_OPTION,
                "A must be a full double or single matrix",
            ))
        }
    };
    let (rows, cols) = shape2(&shape)?;
    if data.iter().any(|z| !z.re.is_finite() || !z.im.is_finite()) {
        return Err(orth_error(
            &UNSUPPORTED_OPTION,
            "A with NaN or Inf entries is not yet specified",
        ));
    }
    Ok(DenseInput {
        data,
        rows,
        cols,
        single,
        complex,
    })
}

/// FR-055-14, FR-055-28: a finite non-negative real scalar.
fn parse_threshold(t: &Value) -> BuiltinResult<f64> {
    let value = match t {
        Value::Num(v) => Some(*v),
        Value::Tensor(t)
            if t.integer_storage().is_none()
                && t.shape.iter().product::<usize>() == 1
                && matches!(t.numeric_dtype(), NumericDType::F64 | NumericDType::F32) =>
        {
            Some(t.materialize_f64()[0])
        }
        _ => None,
    };
    value.filter(|v| v.is_finite() && *v >= 0.0).ok_or_else(|| {
        orth_error(
            &UNSUPPORTED_OPTION,
            "t must be a finite non-negative real double or single scalar",
        )
    })
}

fn basis(input: DenseInput, threshold: Option<f64>) -> BuiltinResult<Value> {
    let DenseInput {
        data,
        rows,
        cols,
        single,
        complex,
    } = input;
    // FR-055-13: empty input gives size(A,1)-by-0.
    let (q, rank) = if rows == 0 || cols == 0 {
        (Vec::new(), 0)
    } else {
        let matrix = DMatrix::from_column_slice(rows, cols, &data);
        let svd = SVD::new(matrix, true, false);
        let u = svd
            .u
            .ok_or_else(|| orth_error(&INTERNAL, "the SVD returned no left vectors"))?;
        let sigma: Vec<f64> = svd.singular_values.iter().copied().collect();
        let mut order: Vec<usize> = (0..sigma.len()).collect();
        order.sort_by(|&i, &j| sigma[j].total_cmp(&sigma[i]));
        let largest = order.first().map_or(0.0, |&i| sigma[i]);
        // FR-055-27 (documented choice, orth.q-default-tol): the usual
        // numerical-rank threshold max(m,n) * eps(class) * largest sigma.
        // FR-055-28 (documented choice, orth.q-second-arg): t is absolute.
        let eps = if single {
            f64::from(f32::EPSILON)
        } else {
            f64::EPSILON
        };
        let tol = threshold.unwrap_or(rows.max(cols) as f64 * eps * largest);
        let kept: Vec<usize> = order.into_iter().filter(|&i| sigma[i] > tol).collect();
        let mut q = Vec::with_capacity(rows * kept.len());
        for &col in &kept {
            q.extend((0..rows).map(|row| u[(row, col)]));
        }
        (q, kept.len())
    };
    let shape = vec![rows, rank];
    // FR-055-12: class kept; complex input gives a complex result.
    if complex {
        let tensor = if single {
            ComplexTensor::from_f32(
                q.iter().map(|z| (z.re as f32, z.im as f32)).collect(),
                shape,
            )
        } else {
            ComplexTensor::new(q.iter().map(|z| (z.re, z.im)).collect(), shape)
        };
        return tensor
            .map(Value::ComplexTensor)
            .map_err(|err| orth_error(&INTERNAL, err));
    }
    let tensor = if single {
        Tensor::from_f32(q.iter().map(|z| z.re as f32).collect(), shape)
    } else {
        Tensor::new(q.iter().map(|z| z.re).collect(), shape)
    };
    tensor
        .map(Value::Tensor)
        .map_err(|err| orth_error(&INTERNAL, err))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod orth_tests {
    use super::orth_builtin;
    use crate::BuiltinResult;
    use futures::executor::block_on;
    use num_complex::Complex64;
    use runmat_value::{
        CharArray, ComplexTensor, IntegerStorage, LogicalArray, NumericDType, SparseTensor, Tensor,
        Value,
    };

    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("orth", args))
    }

    /// Row-major literal to a column-major double matrix value.
    fn mat(rows: usize, cols: usize, row_major: &[f64]) -> Value {
        let mut data = vec![0.0; rows * cols];
        for r in 0..rows {
            for c in 0..cols {
                data[r + c * rows] = row_major[r * cols + c];
            }
        }
        Value::Tensor(Tensor::new(data, vec![rows, cols]).unwrap())
    }

    fn identifier(result: BuiltinResult<Value>) -> String {
        match result {
            Ok(value) => panic!("expected an error, got {value:?}"),
            Err(err) => err.identifier().unwrap_or("<none>").to_string(),
        }
    }

    struct Dense {
        rows: usize,
        cols: usize,
        data: Vec<Complex64>,
        dtype: NumericDType,
        complex: bool,
    }

    fn dense(value: &Value) -> Dense {
        match value {
            Value::Num(v) => Dense {
                rows: 1,
                cols: 1,
                data: vec![Complex64::new(*v, 0.0)],
                dtype: NumericDType::F64,
                complex: false,
            },
            Value::Tensor(t) => Dense {
                rows: t.shape[0],
                cols: t.shape[1],
                data: t
                    .materialize_f64()
                    .into_iter()
                    .map(|v| Complex64::new(v, 0.0))
                    .collect(),
                dtype: t.numeric_dtype(),
                complex: false,
            },
            Value::ComplexTensor(t) => Dense {
                rows: t.shape[0],
                cols: t.shape[1],
                data: t
                    .materialize_f64()
                    .into_iter()
                    .map(|(re, im)| Complex64::new(re, im))
                    .collect(),
                dtype: t.numeric_dtype(),
                complex: true,
            },
            other => panic!("expected a dense matrix, got {other:?}"),
        }
    }

    fn at(m: &Dense, r: usize, c: usize) -> Complex64 {
        m.data[r + c * m.rows]
    }

    /// Frobenius norms of Q'*Q - I and A - Q*(Q'*A) (Frobenius bounds the
    /// 2-norm used in the observation record from above).
    fn residuals(a: &Dense, q: &Dense) -> (f64, f64) {
        let k = q.cols;
        let mut ortho = 0.0;
        for i in 0..k {
            for j in 0..k {
                let mut s = Complex64::new(0.0, 0.0);
                for r in 0..q.rows {
                    s += at(q, r, i).conj() * at(q, r, j);
                }
                if i == j {
                    s -= 1.0;
                }
                ortho += s.norm_sqr();
            }
        }
        let mut span = 0.0;
        for c in 0..a.cols {
            // coefficients Q' * a(:,c)
            let coef: Vec<Complex64> = (0..k)
                .map(|i| (0..a.rows).map(|r| at(q, r, i).conj() * at(a, r, c)).sum())
                .collect();
            for r in 0..a.rows {
                let proj: Complex64 = (0..k).map(|i| at(q, r, i) * coef[i]).sum();
                span += (at(a, r, c) - proj).norm_sqr();
            }
        }
        (ortho.sqrt(), span.sqrt())
    }

    /// Size, class, orthonormality and span; never exact basis entries
    /// (orth.basis-not-unique is non-normative).
    fn check(a: Value, rows: usize, cols: usize, dtype: NumericDType, tol: f64) -> Dense {
        let q = dense(&call(std::slice::from_ref(&a)).unwrap());
        assert_eq!((q.rows, q.cols), (rows, cols), "size of Q");
        assert_eq!(q.dtype, dtype, "class of Q");
        let (ortho, span) = residuals(&dense(&a), &q);
        assert!(ortho < tol, "norm(Q'*Q - I) = {ortho}");
        assert!(span < tol, "norm(A - Q*Q'*A) = {span}");
        q
    }

    fn magic3() -> Value {
        mat(3, 3, &[8.0, 1.0, 6.0, 3.0, 5.0, 7.0, 4.0, 9.0, 2.0])
    }

    // --- Observed (normative): one test per observed case --------------------

    /// case `full-rank` — orth.signature-primary, orth.size-rank, orth.orthonormal
    #[test]
    fn observed_full_rank() {
        check(magic3(), 3, 3, NumericDType::F64, 1.2e-14);
    }

    /// case `rank2` — orth.size-rank, orth.orthonormal
    #[test]
    fn observed_rank2() {
        let a = mat(3, 3, &[1.0, 2.0, 3.0, 2.0, 4.0, 6.0, 1.0, 0.0, 1.0]);
        check(a, 3, 2, NumericDType::F64, 1.2e-14);
    }

    /// case `rank1` — orth.size-rank, orth.orthonormal
    #[test]
    fn observed_rank1() {
        check(mat(3, 3, &[1.0; 9]), 3, 1, NumericDType::F64, 1.2e-14);
    }

    /// case `zero` — orth.size-rank
    #[test]
    fn observed_zero() {
        check(mat(3, 3, &[0.0; 9]), 3, 0, NumericDType::F64, 1.2e-14);
    }

    /// case `tall` — orth.size-rank, orth.orthonormal
    #[test]
    fn observed_tall() {
        let a = mat(4, 2, &[1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 0.0]);
        check(a, 4, 2, NumericDType::F64, 1.2e-14);
    }

    /// case `wide` — orth.size-rank, orth.orthonormal
    #[test]
    fn observed_wide() {
        let a = mat(2, 4, &[1.0, 2.0, 3.0, 4.0, 0.0, 1.0, 0.0, 1.0]);
        check(a, 2, 2, NumericDType::F64, 1.2e-14);
    }

    /// case `diag-abs` — orth.diag (non-normative: only the shape and the
    /// spanned coordinate axes are asserted, not column order or signs).
    #[test]
    fn observed_diag_abs() {
        let a = mat(3, 3, &[2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 5.0]);
        let q = check(a, 3, 2, NumericDType::F64, 1.2e-14);
        for c in 0..2 {
            assert!(at(&q, 1, c).norm() < 1e-14, "e2 is not in range(A)");
        }
    }

    /// case `empty` — orth.empty
    #[test]
    fn observed_empty() {
        let q = dense(&call(&[Value::Tensor(Tensor::new(vec![], vec![0, 0]).unwrap())]).unwrap());
        assert_eq!((q.rows, q.cols, q.dtype), (0, 0, NumericDType::F64));
    }

    /// case `empty-3x0` — orth.empty
    #[test]
    fn observed_empty_3x0() {
        let q = dense(&call(&[Value::Tensor(Tensor::new(vec![], vec![3, 0]).unwrap())]).unwrap());
        assert_eq!((q.rows, q.cols, q.dtype), (3, 0, NumericDType::F64));
    }

    /// case `single` — orth.class, orth.orthonormal
    #[test]
    fn observed_single() {
        let a = Value::Tensor(
            Tensor::from_f32(
                vec![8.0, 3.0, 4.0, 1.0, 5.0, 9.0, 6.0, 7.0, 2.0],
                vec![3, 3],
            )
            .unwrap(),
        );
        check(a, 3, 3, NumericDType::F32, 5e-6);
    }

    /// case `complex` — orth.class, orth.orthonormal
    #[test]
    fn observed_complex() {
        // [1 1i; 1i 1], column-major
        let a = Value::ComplexTensor(
            ComplexTensor::new(
                vec![(1.0, 0.0), (0.0, 1.0), (0.0, 1.0), (1.0, 0.0)],
                vec![2, 2],
            )
            .unwrap(),
        );
        let q = check(a, 2, 2, NumericDType::F64, 1.2e-14);
        assert!(q.complex, "complex input gives a complex double result");
    }

    /// case `second-arg` — orth.second-arg
    #[test]
    fn observed_second_arg() {
        let a = mat(3, 3, &[1.0, 0.0, 0.0, 0.0, 1e-3, 0.0, 0.0, 0.0, 0.0]);
        let q = dense(&call(&[a.clone(), Value::Num(1e-2)]).unwrap());
        assert_eq!((q.rows, q.cols), (3, 1));
        let (ortho, span) = residuals(&dense(&a), &q);
        assert!(ortho < 1.2e-14, "norm(Q'*Q - I) = {ortho}");
        // The 1e-3 direction is dropped: span residual 1e-3.
        assert!((span - 1e-3).abs() < 1e-15, "span residual {span}");
    }

    /// case `err-sparse` — orth.error-sparse
    #[test]
    fn observed_err_sparse() {
        let s = SparseTensor::new(2, 2, vec![0, 1, 2], vec![0, 1], vec![3.0, 4.0]).unwrap();
        assert_eq!(
            identifier(call(&[Value::SparseTensor(s)])),
            "RunMat:svd:noSVDsparse"
        );
    }

    /// case `err-int` — orth.error-int
    #[test]
    fn observed_err_int() {
        let a = Value::Tensor(
            Tensor::new_integer(
                IntegerStorage::I8(vec![8, 3, 4, 1, 5, 9, 6, 7, 2]),
                vec![3, 3],
            )
            .unwrap(),
        );
        assert_eq!(identifier(call(&[a])), "RunMat:svd:inputType");
    }

    /// case `err-char` — orth.error-int
    #[test]
    fn observed_err_char() {
        assert_eq!(
            identifier(call(&[Value::CharArray(CharArray::new_row("abc"))])),
            "RunMat:svd:inputType"
        );
    }

    // --- Documented choices --------------------------------------------------

    /// FR-055-27: default threshold separates clearly distinct singular
    /// values; nearly dependent columns at rounding level are dropped.
    #[test]
    fn documented_choice_default_threshold() {
        let a = mat(2, 2, &[1.0, 1.0, 1.0, 1.0 + 1e-17]);
        check(a, 2, 1, NumericDType::F64, 1.2e-14);
        check(
            mat(2, 2, &[1.0, 0.0, 0.0, 1e-8]),
            2,
            2,
            NumericDType::F64,
            1.2e-14,
        );
    }

    /// FR-055-28: t is an absolute threshold.
    #[test]
    fn documented_choice_absolute_threshold() {
        let a = mat(2, 2, &[100.0, 0.0, 0.0, 2.0]);
        let q = dense(&call(&[a.clone(), Value::Num(1.0)]).unwrap());
        assert_eq!(q.cols, 2);
        let q = dense(&call(&[a, Value::Num(3.0)]).unwrap());
        assert_eq!(q.cols, 1);
    }

    /// FR-055-30: unobserved inputs.
    #[test]
    fn documented_choice_unsupported() {
        const U: &str = "RunMat:orth:UnsupportedOption";
        let mask = Value::LogicalArray(LogicalArray::new(vec![1, 0], vec![1, 2]).unwrap());
        assert_eq!(identifier(call(&[mask])), U);
        assert_eq!(identifier(call(&[Value::Num(f64::NAN)])), U);
        assert_eq!(identifier(call(&[magic3(), Value::Num(-1.0)])), U);
        assert_eq!(identifier(call(&[magic3(), Value::String("x".into())])), U);
    }

    /// FR-055-29: arity and output count.
    #[test]
    fn documented_choice_arity_and_outputs() {
        assert_eq!(identifier(call(&[])), "RunMat:orth:InvalidInput");
        assert_eq!(
            identifier(call(&[magic3(), Value::Num(1.0), Value::Num(1.0)])),
            "RunMat:orth:InvalidInput"
        );
        let _guard = crate::output_count::push_output_count(Some(2));
        let err = block_on(orth_builtin(vec![magic3()])).unwrap_err();
        assert_eq!(err.identifier(), Some("RunMat:orth:TooManyOutputs"));
    }

    /// FR-055-33 (unresolved, orth.q-warning-coverage): no warning.
    #[test]
    fn unresolved_no_warning() {
        assert!(call(&[magic3()]).is_ok());
    }
}
