//! Clean-room `lsqminnorm` (minimum-norm least-squares solution).
//!
//! Clean-room feature `specs/061-sparse-linalg-n5/`. Normative behaviour
//! comes only from the approved export `lsqminnorm` 0.2.0 (see that feature's
//! `provenance.md`): without `tol` (or with a small `tol`) the result agrees
//! with `pinv(full(A))*b`; sparse `A` gives a full result; the COBRA form
//! `lsqminnorm(S', b)` with sparse rank-deficient `S` works; all-zero and
//! empty `A` give zeros; single in, single out; `tol = 0.5` on
//! `[2 1; 1 0.2]` gives a different (truncated) result; and the error
//! identifiers (raised as `RunMat:*`, D-061-1).
//!
//! Algorithms (textbook numerical linear algebra, Golub & Van Loan, "Matrix
//! Computations", ch. 5):
//! - default path (D-061-2): the minimum-norm least-squares solution from a
//!   thin SVD, `x = V * pinv(Sigma) * U' * b`, keeping singular values above
//!   RunMat's `pinv` default threshold `max(m,n) * eps(max sigma)`;
//! - `tol` path (D-061-3, documented choice for the unresolved
//!   `lsqminnorm.q-tol-rule`): a Householder QR factorisation with column
//!   pivoting, `A*P = Q*R`, truncated at the numerical rank `r` = number of
//!   leading `|R(k,k)| > tol`, followed by the minimum-norm solution of the
//!   full-row-rank system `R(1:r,:) * z = (Q'*b)(1:r,:)`, and `x = P*z`.

use nalgebra::{linalg::SVD, ComplexField, DMatrix};
use num_complex::Complex64;
use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{ComplexTensor, NumericDType, Tensor, Value};

use crate::builtins::common::linalg::svd_default_tolerance;
use crate::builtins::common::map_control_flow_with_builtin;
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const NAME: &str = "lsqminnorm";

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
    "Floating-point matrix, full or sparse, real or complex.",
);
const IN_B: BuiltinParamDescriptor = param(
    "b",
    BuiltinParamArity::Required,
    "Right-hand side with as many rows as A; several columns allowed.",
);
const IN_TOL: BuiltinParamDescriptor = param(
    "tol",
    BuiltinParamArity::Required,
    "Absolute rank tolerance on the pivoted-QR diagonal.",
);
const OUT_X: BuiltinParamDescriptor = param(
    "x",
    BuiltinParamArity::Required,
    "Full columns(A)-by-columns(b) minimum-norm least-squares solution.",
);

const INPUTS_AB: [BuiltinParamDescriptor; 2] = [IN_A, IN_B];
const INPUTS_AB_TOL: [BuiltinParamDescriptor; 3] = [IN_A, IN_B, IN_TOL];
const OUTPUTS_X: [BuiltinParamDescriptor; 1] = [OUT_X];

const LSQMINNORM_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "x = lsqminnorm(A, b)",
        inputs: &INPUTS_AB,
        outputs: &OUTPUTS_X,
    },
    BuiltinSignatureDescriptor {
        label: "x = lsqminnorm(A, b, tol)",
        inputs: &INPUTS_AB_TOL,
        outputs: &OUTPUTS_X,
    },
];

const NOT_ENOUGH_INPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.LSQMINNORM.NOT_ENOUGH_INPUTS",
    identifier: Some("RunMat:narginchk:notEnoughInputs"),
    when: "Fewer than two inputs are given.",
    message: "lsqminnorm: not enough input arguments",
};

const INVALID_A: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.LSQMINNORM.INVALID_A",
    identifier: Some("RunMat:lsqminnorm:InvalidA"),
    when: "A is char or an integer class.",
    message: "lsqminnorm: A must be a floating-point matrix",
};

const ROW_MISMATCH: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.LSQMINNORM.ROW_MISMATCH",
    identifier: Some("RunMat:decomposition:mldivide"),
    when: "b has a different number of rows from A.",
    message: "lsqminnorm: A and b must have the same number of rows",
};

const INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.LSQMINNORM.INVALID_INPUT",
    identifier: Some("RunMat:lsqminnorm:InvalidInput"),
    when: "A or b is not 2-D.",
    message: "lsqminnorm: A and b must be 2-D matrices",
};

const UNSUPPORTED_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.LSQMINNORM.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:lsqminnorm:UnsupportedOption"),
    when: "Unobserved inputs: extra arguments, logical or non-numeric A, non-float b, NaN/Inf entries, a tol that is not a finite non-negative real scalar.",
    message: "lsqminnorm: this form is not yet specified",
};

const TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.LSQMINNORM.TOO_MANY_OUTPUTS",
    identifier: Some("RunMat:lsqminnorm:TooManyOutputs"),
    when: "More than one output was requested.",
    message: "lsqminnorm: too many output arguments",
};

const INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.LSQMINNORM.INTERNAL",
    identifier: Some("RunMat:lsqminnorm:Internal"),
    when: "The runtime cannot assemble the result.",
    message: "lsqminnorm: internal runtime failure",
};

const LSQMINNORM_ERRORS: [BuiltinErrorDescriptor; 7] = [
    NOT_ENOUGH_INPUTS,
    INVALID_A,
    ROW_MISMATCH,
    INVALID_INPUT,
    UNSUPPORTED_OPTION,
    TOO_MANY_OUTPUTS,
    INTERNAL,
];

pub const LSQMINNORM_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &LSQMINNORM_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &LSQMINNORM_ERRORS,
};

fn lsq_error(
    descriptor: &'static BuiltinErrorDescriptor,
    detail: impl Into<String>,
) -> RuntimeError {
    let mut builder = build_runtime_error(format!("{NAME}: {}", detail.into())).with_builtin(NAME);
    if let Some(identifier) = descriptor.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

fn unsupported(detail: &str) -> RuntimeError {
    lsq_error(&UNSUPPORTED_OPTION, detail)
}

// ---------------------------------------------------------------------------
// Builtin
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "lsqminnorm",
    category = "math/linalg/solve",
    summary = "Minimum-norm least-squares solution of A*x = b.",
    keywords = "lsqminnorm,least squares,minimum norm,pinv,rank deficient,sparse",
    accel = "cpu",
    descriptor(crate::builtins::math::linalg::solve::lsqminnorm::LSQMINNORM_DESCRIPTOR),
    builtin_path = "crate::builtins::math::linalg::solve::lsqminnorm"
)]
pub(crate) async fn lsqminnorm_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    let out_count = crate::output_count::current_output_count();
    // FR-061-12
    if out_count.is_some_and(|count| count > 1) {
        return Err(lsq_error(&TOO_MANY_OUTPUTS, "only one output is supported"));
    }
    // FR-061-11 (lsqminnorm.error-one-arg); zero inputs: documented choice.
    if args.len() < 2 {
        return Err(lsq_error(
            &NOT_ENOUGH_INPUTS,
            "expected lsqminnorm(A, b) or lsqminnorm(A, b, tol)",
        ));
    }
    if args.len() > 3 {
        return Err(unsupported("arguments after tol are not yet specified"));
    }
    // FR-061-13: gpuArray inputs are gathered.
    let mut gathered = Vec::with_capacity(args.len());
    for arg in &args {
        gathered.push(
            gather_if_needed_async(arg)
                .await
                .map_err(|err| map_control_flow_with_builtin(err, NAME))?,
        );
    }
    let a = dense_input(&gathered[0], Role::A)?;
    let b = dense_input(&gathered[1], Role::B)?;
    // FR-061-09 (lsqminnorm.error-rows)
    if a.rows != b.rows {
        return Err(lsq_error(
            &ROW_MISMATCH,
            format!("A has {} rows but b has {} rows", a.rows, b.rows),
        ));
    }
    let tol = match gathered.get(2) {
        Some(value) => Some(parse_tol(value)?),
        None => None,
    };
    let single = a.single || b.single;
    let complex = a.complex || b.complex;
    let (n, p) = (a.cols, b.cols);
    let x: Vec<Complex64> = if a.rows == 0 || n == 0 || p == 0 {
        // FR-061-04, FR-061-06: nothing to fit, x = zeros(n, p).
        vec![Complex64::new(0.0, 0.0); n * p]
    } else if complex {
        let am = DMatrix::from_column_slice(a.rows, n, &a.data);
        let bm = DMatrix::from_column_slice(b.rows, p, &b.data);
        solve(am, bm, tol)?.iter().copied().collect()
    } else {
        let re = |d: &[Complex64]| d.iter().map(|z| z.re).collect::<Vec<f64>>();
        let am = DMatrix::from_column_slice(a.rows, n, &re(&a.data));
        let bm = DMatrix::from_column_slice(b.rows, p, &re(&b.data));
        solve(am, bm, tol)?
            .iter()
            .map(|v| Complex64::new(*v, 0.0))
            .collect()
    };
    let x = assemble(x, n, p, single, complex)?;
    match out_count {
        None => Ok(x),
        Some(0) => Ok(Value::OutputList(Vec::new())),
        Some(_) => Ok(Value::OutputList(vec![x])),
    }
}

// ---------------------------------------------------------------------------
// Inputs
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum Role {
    A,
    B,
}

struct Dense {
    data: Vec<Complex64>,
    rows: usize,
    cols: usize,
    single: bool,
    complex: bool,
}

fn float_flag(dtype: NumericDType, role: Role) -> BuiltinResult<bool> {
    match dtype {
        NumericDType::F64 => Ok(false),
        NumericDType::F32 => Ok(true),
        _ => Err(class_error(role)),
    }
}

/// FR-061-10 for `A`; FR-061-14 (documented choice) for `b`.
fn class_error(role: Role) -> RuntimeError {
    match role {
        Role::A => lsq_error(
            &INVALID_A,
            "A must be double or single, not char or integer",
        ),
        Role::B => unsupported("b must be a double or single matrix"),
    }
}

fn real_data(values: Vec<f64>) -> Vec<Complex64> {
    values.into_iter().map(|v| Complex64::new(v, 0.0)).collect()
}

fn complex_data(values: Vec<(f64, f64)>) -> Vec<Complex64> {
    values
        .into_iter()
        .map(|(re, im)| Complex64::new(re, im))
        .collect()
}

/// FR-061-01..03, FR-061-10, FR-061-14: a 2-D floating matrix, full or
/// sparse, real or complex, read column-major into complex storage.
fn dense_input(value: &Value, role: Role) -> BuiltinResult<Dense> {
    let (data, shape, single, complex) = match value {
        Value::Num(v) => (vec![Complex64::new(*v, 0.0)], vec![1, 1], false, false),
        Value::Complex(re, im) => (vec![Complex64::new(*re, *im)], vec![1, 1], false, true),
        Value::Tensor(t) => {
            if t.integer_storage().is_some() {
                return Err(class_error(role));
            }
            let single = float_flag(t.numeric_dtype(), role)?;
            (real_data(t.materialize_f64()), t.shape.clone(), single, false)
        }
        Value::ComplexTensor(t) => {
            if t.integer_storage().is_some() {
                return Err(class_error(role));
            }
            let single = float_flag(t.numeric_dtype(), role)?;
            (complex_data(t.materialize_f64()), t.shape.clone(), single, true)
        }
        Value::SparseTensor(s) => {
            if s.is_logical() {
                return Err(unsupported("logical A or b is not yet specified"));
            }
            if s.integer_storage().is_some() {
                return Err(class_error(role));
            }
            let single = float_flag(s.numeric_dtype().unwrap_or(NumericDType::F64), role)?;
            if s.is_complex() {
                let dense = s
                    .to_dense_complex()
                    .map_err(|err| lsq_error(&INTERNAL, err))?;
                (
                    complex_data(dense.materialize_f64()),
                    vec![s.rows, s.cols],
                    single,
                    true,
                )
            } else {
                let dense = s.to_dense().map_err(|err| lsq_error(&INTERNAL, err))?;
                (
                    real_data(dense.materialize_f64()),
                    vec![s.rows, s.cols],
                    single,
                    false,
                )
            }
        }
        Value::Int(_) | Value::CharArray(_) => return Err(class_error(role)),
        _ => {
            return Err(unsupported(
                "A and b must be double or single matrices (logical and other classes are not yet specified)",
            ))
        }
    };
    if shape.iter().skip(2).any(|&dim| dim != 1) {
        return Err(lsq_error(&INVALID_INPUT, "A and b must be 2-D matrices"));
    }
    if data.iter().any(|z| !z.re.is_finite() || !z.im.is_finite()) {
        return Err(unsupported("NaN or Inf entries are not yet specified"));
    }
    Ok(Dense {
        data,
        rows: shape.first().copied().unwrap_or(1),
        cols: shape.get(1).copied().unwrap_or(1),
        single,
        complex,
    })
}

/// FR-061-15 (documented choice): a finite non-negative real scalar.
fn parse_tol(value: &Value) -> BuiltinResult<f64> {
    let tol = match value {
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
    tol.filter(|v| v.is_finite() && *v >= 0.0)
        .ok_or_else(|| unsupported("tol must be a finite non-negative real scalar"))
}

// ---------------------------------------------------------------------------
// Solvers (generic over real and complex double)
// ---------------------------------------------------------------------------

fn solve<T>(a: DMatrix<T>, b: DMatrix<T>, tol: Option<f64>) -> BuiltinResult<DMatrix<T>>
where
    T: ComplexField<RealField = f64> + Copy,
{
    match tol {
        None => svd_min_norm(a, &b, None),
        Some(tol) => pivoted_qr_min_norm(a, b, tol),
    }
}

/// D-061-2: x = V * pinv(Sigma) * U' * b with the singular values above
/// `threshold` (default: RunMat's `pinv` threshold `max(m,n)*eps(max sigma)`).
fn svd_min_norm<T>(
    a: DMatrix<T>,
    b: &DMatrix<T>,
    threshold: Option<f64>,
) -> BuiltinResult<DMatrix<T>>
where
    T: ComplexField<RealField = f64> + Copy,
{
    let (m, n) = a.shape();
    let svd = SVD::new(a, true, true);
    let u = svd
        .u
        .ok_or_else(|| lsq_error(&INTERNAL, "the SVD returned no left vectors"))?;
    let v_t = svd
        .v_t
        .ok_or_else(|| lsq_error(&INTERNAL, "the SVD returned no right vectors"))?;
    let sigma: Vec<f64> = svd.singular_values.iter().copied().collect();
    let tol = threshold.unwrap_or_else(|| svd_default_tolerance(&sigma, m, n));
    let mut x = DMatrix::<T>::zeros(n, b.ncols());
    for (k, &s) in sigma.iter().enumerate() {
        if s <= tol || s == 0.0 {
            continue;
        }
        // coefficient row: (u_k^H b) / s
        let coeff = u.column(k).adjoint() * b;
        let scale = T::from_real(1.0 / s);
        for j in 0..b.ncols() {
            let c = coeff[(0, j)] * scale;
            for i in 0..n {
                // v_k = conj(row k of V^H)
                x[(i, j)] += v_t[(k, i)].conjugate() * c;
            }
        }
    }
    Ok(x)
}

/// D-061-3 (documented choice for `lsqminnorm.q-tol-rule`): Householder QR
/// with column pivoting, truncated at the leading `|R(k,k)| > tol`, then the
/// minimum-norm solution of the truncated full-row-rank system.
fn pivoted_qr_min_norm<T>(
    mut a: DMatrix<T>,
    mut b: DMatrix<T>,
    tol: f64,
) -> BuiltinResult<DMatrix<T>>
where
    T: ComplexField<RealField = f64> + Copy,
{
    let (m, n) = a.shape();
    let p = b.ncols();
    let mut perm: Vec<usize> = (0..n).collect();
    let mut rank = 0usize;
    for k in 0..m.min(n) {
        // Pivot: the remaining column of largest 2-norm (rows k..m).
        let mut best = k;
        let mut best_norm = -1.0f64;
        for j in k..n {
            let norm = (k..m)
                .map(|i| a[(i, j)].modulus_squared())
                .sum::<f64>()
                .sqrt();
            if norm > best_norm {
                best = j;
                best_norm = norm;
            }
        }
        if best != k {
            a.swap_columns(k, best);
            perm.swap(k, best);
        }
        if best_norm <= tol || best_norm == 0.0 {
            break;
        }
        // Householder reflector mapping a(k..m, k) to alpha * e1.
        let x0 = a[(k, k)];
        let phase = if x0.modulus() == 0.0 {
            T::one()
        } else {
            x0 * T::from_real(1.0 / x0.modulus())
        };
        let alpha = -(phase * T::from_real(best_norm));
        let mut v: Vec<T> = (k..m).map(|i| a[(i, k)]).collect();
        v[0] -= alpha;
        let v_norm2: f64 = v.iter().map(|z| z.modulus_squared()).sum();
        if v_norm2 > 0.0 {
            let factor = T::from_real(2.0 / v_norm2);
            for j in (k + 1)..n {
                let w = (k..m).fold(T::zero(), |acc, i| acc + v[i - k].conjugate() * a[(i, j)]);
                let w = w * factor;
                for i in k..m {
                    a[(i, j)] -= v[i - k] * w;
                }
            }
            for j in 0..p {
                let w = (k..m).fold(T::zero(), |acc, i| acc + v[i - k].conjugate() * b[(i, j)]);
                let w = w * factor;
                for i in k..m {
                    b[(i, j)] -= v[i - k] * w;
                }
            }
        }
        a[(k, k)] = alpha;
        for i in (k + 1)..m {
            a[(i, k)] = T::zero();
        }
        rank = k + 1;
    }
    let mut x = DMatrix::<T>::zeros(n, p);
    if rank == 0 {
        return Ok(x);
    }
    // R1 = R(1:r, :) is r-by-n with full row rank; z = pinv(R1) * c1.
    let r1 = a.rows(0, rank).into_owned();
    let c1 = b.rows(0, rank).into_owned();
    let z = svd_min_norm(r1, &c1, Some(0.0))?;
    for (j, &col) in perm.iter().enumerate() {
        for q in 0..p {
            x[(col, q)] = z[(j, q)];
        }
    }
    Ok(x)
}

// ---------------------------------------------------------------------------
// Output
// ---------------------------------------------------------------------------

/// FR-061-02, FR-061-05, FR-061-16: a full matrix of the floating class of
/// the inputs; complex when either input is complex.
fn assemble(
    x: Vec<Complex64>,
    n: usize,
    p: usize,
    single: bool,
    complex: bool,
) -> BuiltinResult<Value> {
    let shape = vec![n, p];
    if complex {
        let tensor = if single {
            ComplexTensor::from_f32(
                x.iter().map(|z| (z.re as f32, z.im as f32)).collect(),
                shape,
            )
        } else {
            ComplexTensor::new(x.iter().map(|z| (z.re, z.im)).collect(), shape)
        };
        return tensor
            .map(Value::ComplexTensor)
            .map_err(|err| lsq_error(&INTERNAL, err));
    }
    let tensor = if single {
        Tensor::from_f32(x.iter().map(|z| z.re as f32).collect(), shape)
    } else {
        Tensor::new(x.iter().map(|z| z.re).collect(), shape)
    };
    tensor
        .map(Value::Tensor)
        .map_err(|err| lsq_error(&INTERNAL, err))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::excessive_precision)]
mod lsqminnorm_tests {
    use super::lsqminnorm_builtin;
    use crate::BuiltinResult;
    use futures::executor::block_on;
    use runmat_value::{
        CharArray, ComplexTensor, IntegerStorage, LogicalArray, NumericDType, SparseTensor, Tensor,
        Value,
    };

    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("lsqminnorm", args))
    }

    /// Row-major literal to a column-major dense double matrix.
    fn mat(rows: usize, cols: usize, row_major: &[f64]) -> Value {
        let mut data = vec![0.0; rows * cols];
        for r in 0..rows {
            for c in 0..cols {
                data[r + c * rows] = row_major[r * cols + c];
            }
        }
        Value::Tensor(Tensor::new(data, vec![rows, cols]).unwrap())
    }

    fn col(values: &[f64]) -> Value {
        mat(values.len(), 1, values)
    }

    /// Sparse matrix from a row-major dense literal.
    fn sparse(rows: usize, cols: usize, row_major: &[f64]) -> Value {
        let mut col_ptrs = vec![0];
        let mut row_idx = Vec::new();
        let mut values = Vec::new();
        for c in 0..cols {
            for r in 0..rows {
                let v = row_major[r * cols + c];
                if v != 0.0 {
                    row_idx.push(r);
                    values.push(v);
                }
            }
            col_ptrs.push(row_idx.len());
        }
        Value::SparseTensor(SparseTensor::new(rows, cols, col_ptrs, row_idx, values).unwrap())
    }

    /// (shape, column-major data, dtype) of a real full result.
    fn real(value: &Value) -> (Vec<usize>, Vec<f64>, NumericDType) {
        match value {
            Value::Tensor(t) => (t.shape.clone(), t.materialize_f64(), t.numeric_dtype()),
            Value::Num(v) => (vec![1, 1], vec![*v], NumericDType::F64),
            other => panic!("expected a full real matrix, got {other:?}"),
        }
    }

    fn close(actual: &[f64], expected: &[f64], tol: f64) {
        assert_eq!(actual.len(), expected.len(), "{actual:?} vs {expected:?}");
        for (a, e) in actual.iter().zip(expected) {
            assert!((a - e).abs() <= tol, "{actual:?} vs {expected:?}");
        }
    }

    /// pinv(full(A)) * b computed with RunMat's own pinv and mtimes.
    fn pinv_times(a: Value, b: Value) -> Value {
        let full = block_on(crate::call_builtin_async("full", &[a])).unwrap();
        let pinv = block_on(crate::call_builtin_async("pinv", &[full])).unwrap();
        block_on(crate::call_builtin_async("mtimes", &[pinv, b])).unwrap()
    }

    /// Observed values and the agreement with pinv(A)*b (2e-15 observed;
    /// RunMat tests 1e-12, `lsqminnorm.q-exactness`).
    fn check(a: Value, b: Value, shape: &[usize], expected_col_major: &[f64]) {
        let x = call(&[a.clone(), b.clone()]).unwrap();
        let (got_shape, data, dtype) = real(&x);
        assert_eq!(got_shape, shape);
        assert_eq!(dtype, NumericDType::F64);
        close(&data, expected_col_major, 1e-12);
        let (_, reference, _) = real(&pinv_times(a, b));
        close(&data, &reference, 1e-12);
    }

    fn identifier(result: BuiltinResult<Value>) -> String {
        match result {
            Ok(value) => panic!("expected an error, got {value:?}"),
            Err(err) => err.identifier().unwrap_or("<none>").to_string(),
        }
    }

    // --- Observed (normative): one test per observed case --------------------

    /// case `square-full-rank` — lsqminnorm.signature-primary, lsqminnorm.vs-pinv
    #[test]
    fn observed_square_full_rank() {
        check(
            mat(2, 2, &[2.0, 1.0, 1.0, 3.0]),
            col(&[3.0, 5.0]),
            &[2, 1],
            &[0.79999999999999938, 1.4000000000000001],
        );
    }

    /// case `overdetermined` — lsqminnorm.vs-pinv
    #[test]
    fn observed_overdetermined() {
        check(
            mat(3, 2, &[1.0, 0.0, 0.0, 1.0, 1.0, 1.0]),
            col(&[1.0, 2.0, 4.0]),
            &[2, 1],
            &[1.3333333333333328, 2.3333333333333339],
        );
    }

    /// case `underdetermined` — lsqminnorm.signature-primary, lsqminnorm.vs-pinv
    #[test]
    fn observed_underdetermined() {
        check(
            mat(2, 3, &[1.0, 1.0, 0.0, 0.0, 1.0, 1.0]),
            col(&[2.0, 3.0]),
            &[3, 1],
            &[0.3333333333333327, 1.6666666666666665, 1.3333333333333337],
        );
    }

    /// case `rank-deficient` — lsqminnorm.vs-pinv (no warning)
    #[test]
    fn observed_rank_deficient() {
        check(
            mat(3, 2, &[1.0, 2.0, 2.0, 4.0, 3.0, 6.0]),
            col(&[1.0, 2.0, 3.0]),
            &[2, 1],
            &[0.2, 0.4],
        );
    }

    /// case `sparse-transposed` — lsqminnorm.cobra-form, lsqminnorm.full-output:
    /// the COBRA call `lsqminnorm(S', b)` with sparse rank-deficient S, the
    /// transpose taken with RunMat's own `ctranspose`.
    #[test]
    fn observed_sparse_transposed() {
        let s = sparse(3, 3, &[1.0, -1.0, 0.0, 0.0, 1.0, -1.0, 1.0, 0.0, -1.0]);
        let st = block_on(crate::call_builtin_async("ctranspose", &[s])).unwrap();
        assert!(matches!(st, Value::SparseTensor(_)));
        let b = col(&[1.0, 2.0, 3.0]);
        check(
            st.clone(),
            b.clone(),
            &[3, 1],
            &[
                -0.33333333333333348,
                -0.33333333333333359,
                -0.66666666666666696,
            ],
        );
        // The residual of the inconsistent system is the observed 3.4641.
        let (_, x, _) = real(&call(&[st, b]).unwrap());
        let a = [[1.0, 0.0, 1.0], [-1.0, 1.0, 0.0], [0.0, -1.0, -1.0]];
        let rhs = [1.0, 2.0, 3.0];
        let residual: f64 = (0..3)
            .map(|i| {
                let ax: f64 = (0..3).map(|j| a[i][j] * x[j]).sum();
                (ax - rhs[i]).powi(2)
            })
            .sum::<f64>()
            .sqrt();
        assert!((residual - 3.4641016151377544).abs() < 1e-12, "{residual}");
    }

    /// case `sparse-wide` — lsqminnorm.vs-pinv, lsqminnorm.full-output
    #[test]
    fn observed_sparse_wide() {
        check(
            sparse(2, 4, &[1.0, 0.0, 2.0, 0.0, 0.0, 3.0, 0.0, 1.0]),
            col(&[1.0, 1.0]),
            &[4, 1],
            &[
                0.2,
                0.3000000000000001,
                0.39999999999999997,
                0.10000000000000002,
            ],
        );
    }

    /// case `multi-rhs` — lsqminnorm.vs-pinv
    #[test]
    fn observed_multi_rhs() {
        check(
            mat(2, 3, &[1.0, 1.0, 0.0, 0.0, 1.0, 1.0]),
            mat(2, 2, &[2.0, 1.0, 3.0, 0.0]),
            &[3, 2],
            &[
                0.3333333333333327,
                1.6666666666666665,
                1.3333333333333337,
                0.66666666666666641,
                0.33333333333333326,
                -0.33333333333333337,
            ],
        );
    }

    /// case `zero-matrix` — lsqminnorm.zero, lsqminnorm.vs-pinv
    #[test]
    fn observed_zero_matrix() {
        check(
            mat(2, 3, &[0.0; 6]),
            col(&[1.0, 1.0]),
            &[3, 1],
            &[0.0, 0.0, 0.0],
        );
    }

    /// case `tol-arg` — lsqminnorm.tol: tol = 0.5 differs from pinv(A)*b. The
    /// value is the D-061-3 documented choice (pivoted QR truncated at rank 1),
    /// which reproduces the observed numbers; the rule itself is unresolved
    /// (`lsqminnorm.tol-rule`, non-normative).
    #[test]
    fn observed_tol_arg() {
        let a = mat(2, 2, &[2.0, 1.0, 1.0, 0.2]);
        let b = col(&[1.0, 1.0]);
        let (shape, x, _) = real(&call(&[a.clone(), b.clone(), Value::Num(0.5)]).unwrap());
        assert_eq!(shape, vec![2, 1]);
        let (_, reference, _) = real(&pinv_times(a, b));
        close(
            &reference,
            &[1.3333333333333333, -1.6666666666666667],
            1e-12,
        );
        let distance = ((x[0] - reference[0]).powi(2) + (x[1] - reference[1]).powi(2)).sqrt();
        assert!((distance - 2.0625098730592994).abs() < 1e-10, "{distance}");
        close(&x, &[0.50268096514745331, 0.22117962466487934], 1e-12);
    }

    /// case `tol-small` — lsqminnorm.tol, lsqminnorm.vs-pinv
    #[test]
    fn observed_tol_small() {
        let a = mat(2, 2, &[2.0, 1.0, 1.0, 0.2]);
        let b = col(&[1.0, 1.0]);
        let (shape, x, dtype) = real(&call(&[a.clone(), b.clone(), Value::Num(1e-3)]).unwrap());
        assert_eq!((shape, dtype), (vec![2, 1], NumericDType::F64));
        close(&x, &[1.333333333333333, -1.6666666666666667], 1e-12);
        let (_, reference, _) = real(&pinv_times(a, b));
        close(&x, &reference, 1e-12);
    }

    /// case `single` — lsqminnorm.single
    #[test]
    fn observed_single() {
        let a = Value::Tensor(Tensor::from_f32(vec![2.0, 1.0, 1.0, 3.0], vec![2, 2]).unwrap());
        let b = Value::Tensor(Tensor::from_f32(vec![3.0, 5.0], vec![2, 1]).unwrap());
        let (shape, x, dtype) = real(&call(&[a, b]).unwrap());
        assert_eq!((shape, dtype), (vec![2, 1], NumericDType::F32));
        close(&x, &[0.8, 1.4], 1e-6);
    }

    /// case `complex` — lsqminnorm.vs-pinv (complex input)
    #[test]
    fn observed_complex() {
        // [1 1i; 0 1] column-major
        let a = Value::ComplexTensor(
            ComplexTensor::new(
                vec![(1.0, 0.0), (0.0, 0.0), (0.0, 1.0), (1.0, 0.0)],
                vec![2, 2],
            )
            .unwrap(),
        );
        let b = Value::ComplexTensor(
            ComplexTensor::new(vec![(1.0, 0.0), (0.0, 1.0)], vec![2, 1]).unwrap(),
        );
        match call(&[a, b]).unwrap() {
            Value::ComplexTensor(t) => {
                assert_eq!(t.shape, vec![2, 1]);
                assert_eq!(t.numeric_dtype(), NumericDType::F64);
                let x = t.materialize_f64();
                let expected = [(2.0, 0.0), (0.0, 1.0)];
                for (got, want) in x.iter().zip(expected) {
                    assert!((got.0 - want.0).abs() < 1e-12 && (got.1 - want.1).abs() < 1e-12);
                }
            }
            other => panic!("expected a complex result, got {other:?}"),
        }
    }

    /// case `empty` — lsqminnorm.empty
    #[test]
    fn observed_empty() {
        let a = Value::Tensor(Tensor::new(vec![], vec![0, 3]).unwrap());
        let b = Value::Tensor(Tensor::new(vec![], vec![0, 1]).unwrap());
        let (shape, x, dtype) = real(&call(&[a, b]).unwrap());
        assert_eq!((shape, dtype), (vec![3, 1], NumericDType::F64));
        assert_eq!(x, vec![0.0; 3]);
    }

    /// case `err-rows` — lsqminnorm.error-rows
    #[test]
    fn observed_err_rows() {
        assert_eq!(
            identifier(call(&[
                mat(2, 2, &[1.0, 0.0, 0.0, 1.0]),
                col(&[1.0, 2.0, 3.0])
            ])),
            "RunMat:decomposition:mldivide"
        );
    }

    /// case `err-char` — lsqminnorm.error-char
    #[test]
    fn observed_err_char() {
        assert_eq!(
            identifier(call(&[
                Value::CharArray(CharArray::new_row("ab")),
                col(&[1.0, 2.0])
            ])),
            "RunMat:lsqminnorm:InvalidA"
        );
    }

    /// case `err-int` — lsqminnorm.error-char
    #[test]
    fn observed_err_int() {
        let a = Value::Tensor(
            Tensor::new_integer(IntegerStorage::I32(vec![1, 0, 0, 1]), vec![2, 2]).unwrap(),
        );
        assert_eq!(
            identifier(call(&[a, col(&[1.0, 2.0])])),
            "RunMat:lsqminnorm:InvalidA"
        );
    }

    /// case `err-one-arg` — lsqminnorm.error-one-arg
    #[test]
    fn observed_err_one_arg() {
        assert_eq!(
            identifier(call(&[mat(2, 2, &[1.0, 0.0, 0.0, 1.0])])),
            "RunMat:narginchk:notEnoughInputs"
        );
    }

    // --- Documented choices and unresolved points ----------------------------

    /// D-061-3: the tol path keeps every component whose pivoted-QR diagonal
    /// exceeds tol; for a full-rank matrix it equals the unique solution, and
    /// a tol above every diagonal gives zeros.
    #[test]
    fn documented_choice_tol_rule() {
        let a = mat(3, 2, &[1.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
        let b = col(&[1.0, 2.0, 4.0]);
        let (_, x, _) = real(&call(&[a.clone(), b.clone(), Value::Num(1e-8)]).unwrap());
        close(&x, &[1.3333333333333333, 2.3333333333333333], 1e-12);
        let (_, x, _) = real(&call(&[a.clone(), b.clone(), Value::Num(10.0)]).unwrap());
        assert_eq!(x, vec![0.0, 0.0]);
        // A small tol on a rank-deficient matrix drops the rounding-level pivot.
        let r = mat(3, 2, &[1.0, 2.0, 2.0, 4.0, 3.0, 6.0]);
        let (_, x, _) = real(&call(&[r, col(&[1.0, 2.0, 3.0]), Value::Num(1e-10)]).unwrap());
        // A rank-1 system: the pivoted-QR truncation has the same range and
        // row space as A, so it gives the minimum-norm solution [0.2; 0.4].
        close(&x, &[0.2, 0.4], 1e-12);
        // Sparse input also takes the tol path.
        let s = sparse(2, 4, &[1.0, 0.0, 2.0, 0.0, 0.0, 3.0, 0.0, 1.0]);
        let (_, x, _) = real(&call(&[s, col(&[1.0, 1.0]), Value::Num(1e-6)]).unwrap());
        close(&x, &[0.2, 0.3, 0.4, 0.1], 1e-12);
    }

    /// FR-061-14, FR-061-15: invalid or unobserved arguments.
    #[test]
    fn documented_choice_unsupported() {
        const U: &str = "RunMat:lsqminnorm:UnsupportedOption";
        let a = mat(2, 2, &[1.0, 0.0, 0.0, 1.0]);
        let b = col(&[1.0, 2.0]);
        for tol in [
            Value::Num(-1.0),
            Value::Num(f64::NAN),
            Value::Num(f64::INFINITY),
            Value::Tensor(Tensor::new(vec![], vec![0, 0]).unwrap()),
            Value::CharArray(CharArray::new_row("x")),
        ] {
            assert_eq!(identifier(call(&[a.clone(), b.clone(), tol])), U);
        }
        assert_eq!(
            identifier(call(&[
                a.clone(),
                b.clone(),
                Value::Num(1.0),
                Value::Num(1.0)
            ])),
            U
        );
        let logical = Value::LogicalArray(LogicalArray::new(vec![1, 0, 0, 1], vec![2, 2]).unwrap());
        assert_eq!(identifier(call(&[logical, b.clone()])), U);
        assert_eq!(
            identifier(call(&[
                a.clone(),
                Value::CharArray(CharArray::new_row("ab"))
            ])),
            U
        );
        assert_eq!(identifier(call(&[mat(1, 1, &[f64::NAN]), col(&[1.0])])), U);
        let nd = Value::Tensor(Tensor::new(vec![1.0; 8], vec![2, 2, 2]).unwrap());
        assert_eq!(identifier(call(&[nd, b])), "RunMat:lsqminnorm:InvalidInput");
    }

    /// FR-061-11, FR-061-12: arity and output count.
    #[test]
    fn documented_choice_arity_and_outputs() {
        assert_eq!(identifier(call(&[])), "RunMat:narginchk:notEnoughInputs");
        let _guard = crate::output_count::push_output_count(Some(2));
        let err = block_on(lsqminnorm_builtin(vec![Value::Num(1.0), Value::Num(1.0)])).unwrap_err();
        assert_eq!(err.identifier(), Some("RunMat:lsqminnorm:TooManyOutputs"));
    }

    /// FR-061-16: mixed classes and complex right-hand sides.
    #[test]
    fn documented_choice_mixed_classes() {
        let a = mat(2, 2, &[2.0, 1.0, 1.0, 3.0]);
        let b32 = Value::Tensor(Tensor::from_f32(vec![3.0, 5.0], vec![2, 1]).unwrap());
        let (_, x, dtype) = real(&call(&[a.clone(), b32]).unwrap());
        assert_eq!(dtype, NumericDType::F32);
        close(&x, &[0.8, 1.4], 1e-6);
        let bz = Value::ComplexTensor(
            ComplexTensor::new(vec![(3.0, 0.0), (5.0, 0.0)], vec![2, 1]).unwrap(),
        );
        assert!(matches!(call(&[a, bz]).unwrap(), Value::ComplexTensor(_)));
        // A scalar system.
        let (_, x, _) = real(&call(&[Value::Num(4.0), Value::Num(2.0)]).unwrap());
        close(&x, &[0.5], 1e-15);
    }

    /// `lsqminnorm.q-warning-coverage` (unresolved): no warning is emitted,
    /// also for rank-deficient input.
    #[test]
    fn unresolved_no_warning() {
        let r = mat(3, 2, &[1.0, 2.0, 2.0, 4.0, 3.0, 6.0]);
        assert!(call(&[r, col(&[1.0, 2.0, 3.0])]).is_ok());
    }
}
