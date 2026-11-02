//! Clean-room `normest` (estimate of the matrix 2-norm).
//!
//! Clean-room feature `specs/067-numeric-n6/`. Normative behaviour comes only
//! from the approved export `normest` 0.2.0 (pin `e364206`, see that
//! feature's `provenance.md`): the output classes, agreement with
//! `norm(full(S))` within the observed relative difference, the effect of the
//! tolerance argument, the zero/empty/scalar/NaN values and the error
//! identifiers (raised as `RunMat:*`, D-067-1). The count `cnt` and the
//! default tolerance are not specified (`normest.q-exactness`,
//! `normest.q-default-tol`).
//!
//! Algorithm (documented choice, D-067-2): the power method applied to
//! `S'*S` (Golub & Van Loan, *Matrix Computations*, 4th ed., §7.3.1 and
//! §8.6), carried out as one product with `S` and one with `S'` per step, so
//! sparse input is never densified. The estimate after a step is `||S*x||`
//! for a unit vector `x`; iteration stops once two successive estimates
//! differ by at most `tol` times the current one. The start vector holds the
//! column sums of `|S|`; if `S` maps it to zero, the column of largest norm
//! is used instead. `cnt` is the number of steps taken.

use num_complex::Complex64;
use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{NumericDType, Tensor, Value};

use crate::builtins::common::map_control_flow_with_builtin;
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const NAME: &str = "normest";

/// D-067-2: default relative tolerance (documented choice, `normest.q-default-tol`).
pub(crate) const DEFAULT_TOL: f64 = 1.0e-8;
/// D-067-2: iteration cap (documented choice; no warning is raised).
const MAX_STEPS: usize = 10_000;

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

const IN_S: BuiltinParamDescriptor = param(
    "S",
    BuiltinParamArity::Required,
    "2-D floating-point matrix, full or sparse, real or complex.",
);
const IN_TOL: BuiltinParamDescriptor = param(
    "tol",
    BuiltinParamArity::Optional,
    "Positive relative tolerance (RunMat default 1e-8).",
);
const OUT_NRM: BuiltinParamDescriptor = param(
    "nrm",
    BuiltinParamArity::Required,
    "Estimate of norm(S), a full scalar of the floating class of S.",
);
const OUT_CNT: BuiltinParamDescriptor = param(
    "cnt",
    BuiltinParamArity::Optional,
    "Number of power-method steps taken (double).",
);

const INPUTS_S: [BuiltinParamDescriptor; 1] = [IN_S];
const INPUTS_S_TOL: [BuiltinParamDescriptor; 2] = [IN_S, IN_TOL];
const OUTPUTS_NRM: [BuiltinParamDescriptor; 1] = [OUT_NRM];
const OUTPUTS_NRM_CNT: [BuiltinParamDescriptor; 2] = [OUT_NRM, OUT_CNT];

const NORMEST_SIGNATURES: [BuiltinSignatureDescriptor; 3] = [
    BuiltinSignatureDescriptor {
        label: "nrm = normest(S)",
        inputs: &INPUTS_S,
        outputs: &OUTPUTS_NRM,
    },
    BuiltinSignatureDescriptor {
        label: "[nrm, cnt] = normest(S)",
        inputs: &INPUTS_S,
        outputs: &OUTPUTS_NRM_CNT,
    },
    BuiltinSignatureDescriptor {
        label: "[nrm, cnt] = normest(S, tol)",
        inputs: &INPUTS_S_TOL,
        outputs: &OUTPUTS_NRM_CNT,
    },
];

const INTEGER_NOT_SUPPORTED: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.NORMEST.INTEGER_NOT_SUPPORTED",
    identifier: Some("RunMat:mtimes:integerNotSupported"),
    when: "S is of an integer class.",
    message: "normest: integer matrices are not supported",
};

const ND_ARRAY: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.NORMEST.ND_ARRAY",
    identifier: Some("RunMat:transpose:NDArray"),
    when: "S has more than two dimensions.",
    message: "normest: S must be a 2-D matrix",
};

const MINRHS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.NORMEST.MINRHS",
    identifier: Some("RunMat:minrhs"),
    when: "No input is given.",
    message: "normest: not enough input arguments",
};

const UNSUPPORTED_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.NORMEST.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:normest:UnsupportedOption"),
    when: "Unobserved inputs: logical or non-numeric S, an invalid tol, more than two arguments.",
    message: "normest: this form is not yet specified",
};

const TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.NORMEST.TOO_MANY_OUTPUTS",
    identifier: Some("RunMat:normest:TooManyOutputs"),
    when: "More than two outputs were requested.",
    message: "normest: too many output arguments",
};

const INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.NORMEST.INTERNAL",
    identifier: Some("RunMat:normest:Internal"),
    when: "The runtime cannot assemble the result.",
    message: "normest: internal runtime failure",
};

const NORMEST_ERRORS: [BuiltinErrorDescriptor; 6] = [
    INTEGER_NOT_SUPPORTED,
    ND_ARRAY,
    MINRHS,
    UNSUPPORTED_OPTION,
    TOO_MANY_OUTPUTS,
    INTERNAL,
];

pub const NORMEST_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &NORMEST_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &NORMEST_ERRORS,
};

fn normest_error(
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
    normest_error(&UNSUPPORTED_OPTION, detail)
}

// ---------------------------------------------------------------------------
// Builtin
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "normest",
    category = "math/linalg/solve",
    summary = "Estimate of the 2-norm of a matrix, by the power method on S'*S.",
    keywords = "normest,2-norm,norm estimate,power method,sparse",
    accel = "cpu",
    descriptor(crate::builtins::math::linalg::solve::normest::NORMEST_DESCRIPTOR),
    builtin_path = "crate::builtins::math::linalg::solve::normest"
)]
pub(crate) async fn normest_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    let out_count = crate::output_count::current_output_count();
    // FR-067-14
    if out_count.is_some_and(|count| count > 2) {
        return Err(normest_error(
            &TOO_MANY_OUTPUTS,
            "at most two outputs ([nrm, cnt]) are supported",
        ));
    }
    // FR-067-10 (normest.error-no-arg)
    if args.is_empty() {
        return Err(normest_error(&MINRHS, "expected normest(S)"));
    }
    if args.len() > 2 {
        return Err(unsupported(
            "arguments after S and tol are not yet specified",
        ));
    }
    // FR-067-15: gpuArray input is gathered.
    let mut gathered = Vec::with_capacity(args.len());
    for arg in &args {
        gathered.push(
            gather_if_needed_async(arg)
                .await
                .map_err(|err| map_control_flow_with_builtin(err, NAME))?,
        );
    }
    let matrix = Operator::from_value(&gathered[0])?;
    let tol = match gathered.get(1) {
        Some(value) => parse_tol(value)?,
        None => DEFAULT_TOL,
    };
    let (estimate, steps) = matrix.estimate(tol);
    // FR-067-02 (normest.output-class)
    let nrm = if matrix.single {
        Tensor::from_f32(vec![estimate as f32], vec![1, 1])
            .map(Value::Tensor)
            .map_err(|err| normest_error(&INTERNAL, err))?
    } else {
        Value::Num(estimate)
    };
    match out_count {
        None => Ok(nrm),
        Some(0) => Ok(Value::OutputList(Vec::new())),
        Some(1) => Ok(Value::OutputList(vec![nrm])),
        Some(_) => Ok(Value::OutputList(vec![nrm, Value::Num(steps as f64)])),
    }
}

/// FR-067-14 (documented choice): a positive finite real double or single scalar.
fn parse_tol(value: &Value) -> BuiltinResult<f64> {
    let tol = match value {
        Value::Num(v) => *v,
        Value::Tensor(t)
            if t.integer_storage().is_none()
                && matches!(t.numeric_dtype(), NumericDType::F64 | NumericDType::F32)
                && t.len() == 1 =>
        {
            t.materialize_f64()[0]
        }
        _ => return Err(unsupported("tol must be a real double or single scalar")),
    };
    if !(tol.is_finite() && tol > 0.0) {
        return Err(unsupported("tol must be positive and finite"));
    }
    Ok(tol)
}

// ---------------------------------------------------------------------------
// Matrix operator: full or sparse, without densifying sparse input
// ---------------------------------------------------------------------------

enum Storage {
    /// Column-major dense entries.
    Dense(Vec<Complex64>),
    /// Compressed sparse columns.
    Sparse {
        col_ptrs: Vec<usize>,
        row_indices: Vec<usize>,
        values: Vec<Complex64>,
    },
}

struct Operator {
    rows: usize,
    cols: usize,
    storage: Storage,
    single: bool,
}

fn real_entries(values: Vec<f64>) -> Vec<Complex64> {
    values.into_iter().map(|v| Complex64::new(v, 0.0)).collect()
}

fn complex_entries(values: Vec<(f64, f64)>) -> Vec<Complex64> {
    values
        .into_iter()
        .map(|(re, im)| Complex64::new(re, im))
        .collect()
}

/// FR-067-08, FR-067-14: the floating class flag of S (`true` for single).
fn float_class(dtype: NumericDType) -> BuiltinResult<bool> {
    match dtype {
        NumericDType::F64 => Ok(false),
        NumericDType::F32 => Ok(true),
        _ => Err(normest_error(
            &INTEGER_NOT_SUPPORTED,
            "S must be double or single",
        )),
    }
}

fn check_2d(shape: &[usize]) -> BuiltinResult<(usize, usize)> {
    // FR-067-09 (normest.error-3d)
    if shape.iter().skip(2).any(|&dim| dim != 1) {
        return Err(normest_error(&ND_ARRAY, "S must be a 2-D matrix"));
    }
    Ok((
        shape.first().copied().unwrap_or(1),
        shape.get(1).copied().unwrap_or(1),
    ))
}

impl Operator {
    fn dense(data: Vec<Complex64>, shape: &[usize], single: bool) -> BuiltinResult<Self> {
        let (rows, cols) = check_2d(shape)?;
        Ok(Self {
            rows,
            cols,
            storage: Storage::Dense(data),
            single,
        })
    }

    fn from_value(value: &Value) -> BuiltinResult<Self> {
        match value {
            Value::Num(v) => Self::dense(vec![Complex64::new(*v, 0.0)], &[1, 1], false),
            Value::Complex(re, im) => Self::dense(vec![Complex64::new(*re, *im)], &[1, 1], false),
            // FR-067-08 (normest.error-int)
            Value::Int(_) => Err(normest_error(
                &INTEGER_NOT_SUPPORTED,
                "integer matrices are not supported",
            )),
            Value::Tensor(t) => {
                if t.integer_storage().is_some() {
                    return Err(normest_error(
                        &INTEGER_NOT_SUPPORTED,
                        "integer matrices are not supported",
                    ));
                }
                let single = float_class(t.numeric_dtype())?;
                Self::dense(real_entries(t.materialize_f64()), &t.shape, single)
            }
            Value::ComplexTensor(t) => {
                if t.integer_storage().is_some() {
                    return Err(normest_error(
                        &INTEGER_NOT_SUPPORTED,
                        "integer matrices are not supported",
                    ));
                }
                let single = float_class(t.numeric_dtype())?;
                Self::dense(complex_entries(t.materialize_f64()), &t.shape, single)
            }
            // FR-067-11 (non-normative normest.char; documented choice): the
            // character codes are used.
            Value::CharArray(chars) => {
                let shape = chars.shape().to_vec();
                let codes = chars
                    .to_column_major()
                    .into_iter()
                    .map(|c| Complex64::new(c as u32 as f64, 0.0))
                    .collect();
                Self::dense(codes, &shape, false)
            }
            Value::SparseTensor(s) => {
                if s.is_logical() {
                    return Err(unsupported("logical S is not yet specified"));
                }
                if s.integer_storage().is_some() {
                    return Err(normest_error(
                        &INTEGER_NOT_SUPPORTED,
                        "integer matrices are not supported",
                    ));
                }
                let single = float_class(s.numeric_dtype().unwrap_or(NumericDType::F64))?;
                let values = if s.is_complex() {
                    complex_entries(
                        s.materialize_complex_f64()
                            .map_err(|err| normest_error(&INTERNAL, err))?,
                    )
                } else {
                    real_entries(
                        s.materialize_real_f64()
                            .map_err(|err| normest_error(&INTERNAL, err))?,
                    )
                };
                Ok(Self {
                    rows: s.rows,
                    cols: s.cols,
                    storage: Storage::Sparse {
                        col_ptrs: s.col_ptrs.to_vec(),
                        row_indices: s.row_indices.to_vec(),
                        values,
                    },
                    single,
                })
            }
            _ => Err(unsupported(
                "S must be a double or single matrix (logical and other classes are not yet specified)",
            )),
        }
    }

    /// Visit every stored entry as (row, column, value).
    fn for_each(&self, mut f: impl FnMut(usize, usize, Complex64)) {
        match &self.storage {
            Storage::Dense(data) => {
                for j in 0..self.cols {
                    for i in 0..self.rows {
                        f(i, j, data[i + j * self.rows]);
                    }
                }
            }
            Storage::Sparse {
                col_ptrs,
                row_indices,
                values,
            } => {
                for j in 0..self.cols {
                    for k in col_ptrs[j]..col_ptrs[j + 1] {
                        f(row_indices[k], j, values[k]);
                    }
                }
            }
        }
    }

    /// `S * x`.
    fn apply(&self, x: &[Complex64]) -> Vec<Complex64> {
        let mut y = vec![Complex64::new(0.0, 0.0); self.rows];
        self.for_each(|i, j, a| y[i] += a * x[j]);
        y
    }

    /// `S' * y` (conjugate transpose).
    fn apply_adjoint(&self, y: &[Complex64]) -> Vec<Complex64> {
        let mut x = vec![Complex64::new(0.0, 0.0); self.cols];
        self.for_each(|i, j, a| x[j] += a.conj() * y[i]);
        x
    }

    /// FR-067-03..07, D-067-2: returns (estimate, steps).
    fn estimate(&self, tol: f64) -> (f64, usize) {
        // FR-067-06 (normest.zero): empty input.
        if self.rows == 0 || self.cols == 0 {
            return (0.0, 0);
        }
        // FR-067-07 (normest.scalar-nan); Inf without NaN gives Inf (documented choice).
        let mut has_nan = false;
        let mut has_inf = false;
        self.for_each(|_, _, a| {
            has_nan |= a.re.is_nan() || a.im.is_nan();
            has_inf |= a.re.is_infinite() || a.im.is_infinite();
        });
        if has_nan {
            return (f64::NAN, 0);
        }
        if has_inf {
            return (f64::INFINITY, 0);
        }
        // Start vector: column sums of |S|.
        let mut x = vec![Complex64::new(0.0, 0.0); self.cols];
        let mut col_norms = vec![0.0f64; self.cols];
        self.for_each(|_, j, a| {
            x[j] += Complex64::new(a.norm(), 0.0);
            col_norms[j] += a.norm_sqr();
        });
        let mut e = norm2(&x);
        // FR-067-06 (normest.zero): all-zero input.
        if e == 0.0 {
            return (0.0, 0);
        }
        scale(&mut x, 1.0 / e);
        let mut previous = 0.0;
        let mut steps = 0usize;
        while (e - previous).abs() > tol * e && steps < MAX_STEPS {
            previous = e;
            let mut sx = self.apply(&x);
            let mut sx_norm = norm2(&sx);
            if sx_norm == 0.0 {
                // The start vector lies in the null space: use the column of
                // largest norm (S times it is that nonzero column).
                let k = (0..self.cols)
                    .max_by(|&a, &b| col_norms[a].total_cmp(&col_norms[b]))
                    .unwrap_or(0);
                x = vec![Complex64::new(0.0, 0.0); self.cols];
                x[k] = Complex64::new(1.0, 0.0);
                sx = self.apply(&x);
                sx_norm = norm2(&sx);
            }
            e = sx_norm;
            x = self.apply_adjoint(&sx);
            let x_norm = norm2(&x);
            steps += 1;
            if x_norm == 0.0 {
                break;
            }
            scale(&mut x, 1.0 / x_norm);
        }
        (e, steps)
    }
}

/// 2-norm with scaling against overflow.
fn norm2(v: &[Complex64]) -> f64 {
    let largest = v.iter().map(|z| z.norm()).fold(0.0f64, f64::max);
    if largest == 0.0 || !largest.is_finite() {
        return largest;
    }
    let sum: f64 = v.iter().map(|z| (z / largest).norm_sqr()).sum();
    largest * sum.sqrt()
}

fn scale(v: &mut [Complex64], factor: f64) {
    for z in v.iter_mut() {
        *z *= factor;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::excessive_precision)]
mod normest_tests {
    use super::normest_builtin;
    use crate::BuiltinResult;
    use futures::executor::block_on;
    use runmat_value::{
        CharArray, ComplexTensor, IntegerStorage, LogicalArray, NumericDType, SparseTensor, Tensor,
        Value,
    };

    /// `normest.default-accuracy`: the largest observed relative difference.
    const OBSERVED_BOUND: f64 = 2.3e-7;

    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("normest", args))
    }

    /// `[nrm, cnt] = normest(args...)`, called directly with two outputs.
    fn call2(args: Vec<Value>) -> (Value, Value) {
        let _guard = crate::output_count::push_output_count(Some(2));
        match block_on(normest_builtin(args)).unwrap() {
            Value::OutputList(mut outs) => {
                assert_eq!(outs.len(), 2);
                let cnt = outs.pop().unwrap();
                let nrm = outs.pop().unwrap();
                (nrm, cnt)
            }
            other => panic!("expected two outputs, got {other:?}"),
        }
    }

    fn scalar(value: &Value) -> f64 {
        match value {
            Value::Num(v) => *v,
            other => panic!("expected a full double scalar, got {other:?}"),
        }
    }

    /// The count is a full double holding a non-negative integer.
    fn count(value: &Value) -> f64 {
        let c = scalar(value);
        assert!(c >= 0.0 && c.fract() == 0.0, "cnt = {c}");
        c
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

    /// RunMat's own `norm(full(S))` (2-norm).
    fn norm_full(value: Value) -> f64 {
        scalar(&block_on(crate::call_builtin_async("norm", &[value])).unwrap())
    }

    fn rel_diff(estimate: f64, reference: f64) -> f64 {
        if reference == 0.0 {
            estimate.abs()
        } else {
            (estimate - reference).abs() / reference
        }
    }

    fn identifier(result: BuiltinResult<Value>) -> String {
        match result {
            Ok(value) => panic!("expected an error, got {value:?}"),
            Err(err) => err.identifier().unwrap_or("<none>").to_string(),
        }
    }

    const STOICH: [f64; 20] = [
        -1.0, 0.0, 1.0, 0.0, 0.0, 1.0, -1.0, 0.0, 0.0, 1.0, 0.0, 1.0, -1.0, -1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, -1.0,
    ];
    const SQUARE: [f64; 9] = [4.0, 1.0, 0.0, 1.0, 3.0, 1.0, 0.0, 1.0, 2.0];

    /// Asserts the two-output form: full double nrm within the observed
    /// bound of RunMat's norm(full(S)), integer-valued double cnt.
    fn assert_accurate(input: Value, dense: Value) -> f64 {
        let (nrm, cnt) = call2(vec![input]);
        let estimate = scalar(&nrm);
        count(&cnt);
        let reference = norm_full(dense);
        let diff = rel_diff(estimate, reference);
        assert!(diff <= OBSERVED_BOUND, "{estimate} vs {reference}: {diff}");
        estimate
    }

    // --- Observed (normative): one test per observed case --------------------

    /// case `sparse-stoich` — normest.signature-primary, normest.cobra-form,
    /// normest.output-class, normest.default-accuracy
    #[test]
    fn observed_sparse_stoich() {
        let estimate = assert_accurate(sparse(4, 5, &STOICH), mat(4, 5, &STOICH));
        assert!((estimate - 2.0).abs() <= 1e-12, "{estimate}");
    }

    /// case `dense-square` — normest.default-accuracy
    #[test]
    fn observed_dense_square() {
        let estimate = assert_accurate(mat(3, 3, &SQUARE), mat(3, 3, &SQUARE));
        assert!((estimate - 4.7320508075688776).abs() <= 4.7320508075688776 * OBSERVED_BOUND);
    }

    /// case `dense-wide` — normest.default-accuracy
    #[test]
    fn observed_dense_wide() {
        let a = [1.0, 2.0, 3.0, 4.0, 0.0, 1.0, 0.0, 1.0];
        let estimate = assert_accurate(mat(2, 4, &a), mat(2, 4, &a));
        assert!((estimate - 5.5885191430045058).abs() <= 5.6 * OBSERVED_BOUND);
    }

    /// case `one-out` — normest.signature-primary, normest.default-accuracy
    #[test]
    fn observed_one_out() {
        let estimate = scalar(&call(&[mat(2, 2, &[3.0, 0.0, 0.0, 4.0])]).unwrap());
        assert!(rel_diff(estimate, 4.0) <= OBSERVED_BOUND, "{estimate}");
        // One requested output.
        let _guard = crate::output_count::push_output_count(Some(1));
        match block_on(normest_builtin(vec![mat(2, 2, &[3.0, 0.0, 0.0, 4.0])])).unwrap() {
            Value::OutputList(outs) => {
                assert_eq!(outs.len(), 1);
                assert!(rel_diff(scalar(&outs[0]), 4.0) <= OBSERVED_BOUND);
            }
            other => panic!("expected one output, got {other:?}"),
        }
    }

    /// cases `tol-coarse`, `tol-fine`, `dense-square` — normest.tol
    #[test]
    fn observed_tol_coarse_and_fine() {
        let reference = norm_full(mat(3, 3, &SQUARE));
        let default = rel_diff(scalar(&call(&[mat(3, 3, &SQUARE)]).unwrap()), reference);
        let (coarse, coarse_cnt) = call2(vec![mat(3, 3, &SQUARE), Value::Num(1e-1)]);
        let (fine, fine_cnt) = call2(vec![mat(3, 3, &SQUARE), Value::Num(1e-12)]);
        let coarse = rel_diff(scalar(&coarse), reference);
        let fine = rel_diff(scalar(&fine), reference);
        assert!(coarse > default, "tol 1e-1: {coarse} vs default {default}");
        assert!(fine < default, "tol 1e-12: {fine} vs default {default}");
        // The count grows as tol decreases (normest.iteration-count is
        // non-normative; only the ordering is checked here).
        assert!(count(&coarse_cnt) < count(&fine_cnt));
    }

    /// case `row-vector` — normest.default-accuracy
    #[test]
    fn observed_row_vector() {
        let v = [3.0, 4.0, 12.0];
        let estimate = assert_accurate(mat(1, 3, &v), mat(1, 3, &v));
        assert!(rel_diff(estimate, 13.0) <= 1e-15, "{estimate}");
    }

    /// case `column-vector` — normest.default-accuracy
    #[test]
    fn observed_column_vector() {
        let v = [3.0, 4.0, 12.0];
        let estimate = assert_accurate(mat(3, 1, &v), mat(3, 1, &v));
        assert!(rel_diff(estimate, 13.0) <= 1e-15, "{estimate}");
    }

    /// case `identity` — normest.output-class, normest.default-accuracy
    #[test]
    fn observed_identity() {
        let mut eye = vec![0.0; 25];
        for i in 0..5 {
            eye[i * 5 + i] = 1.0;
        }
        let estimate = assert_accurate(sparse(5, 5, &eye), mat(5, 5, &eye));
        assert!((estimate - 1.0).abs() <= 1e-15);
    }

    /// case `zero` — normest.zero
    #[test]
    fn observed_zero() {
        let (nrm, cnt) = call2(vec![mat(3, 2, &[0.0; 6])]);
        assert_eq!(scalar(&nrm), 0.0);
        count(&cnt);
    }

    /// case `sparse-zero` — normest.zero
    #[test]
    fn observed_sparse_zero() {
        let (nrm, cnt) = call2(vec![sparse(3, 3, &[0.0; 9])]);
        assert_eq!(scalar(&nrm), 0.0);
        count(&cnt);
    }

    /// case `complex` — normest.default-accuracy
    #[test]
    fn observed_complex() {
        // [1 1i; 2 0], column-major.
        let z = ComplexTensor::new(
            vec![(1.0, 0.0), (2.0, 0.0), (0.0, 1.0), (0.0, 0.0)],
            vec![2, 2],
        )
        .unwrap();
        let estimate = assert_accurate(Value::ComplexTensor(z.clone()), Value::ComplexTensor(z));
        assert!(rel_diff(estimate, 2.2882456112707374) <= OBSERVED_BOUND);
    }

    /// case `single` — normest.output-class, normest.default-accuracy
    #[test]
    fn observed_single() {
        // single([1 2; 3 4]), column-major.
        let s = Tensor::from_f32(vec![1.0, 3.0, 2.0, 4.0], vec![2, 2]).unwrap();
        let (nrm, cnt) = call2(vec![Value::Tensor(s)]);
        count(&cnt);
        match nrm {
            Value::Tensor(t) => {
                assert_eq!(t.numeric_dtype(), NumericDType::F32);
                assert_eq!(t.shape, vec![1, 1]);
                let estimate = t.materialize_f64()[0];
                // Observation tolerance 1e-4 for single.
                assert!(rel_diff(estimate, 5.4649857042190426) <= 1e-6, "{estimate}");
            }
            other => panic!("expected a single scalar, got {other:?}"),
        }
    }

    /// case `scalar` — normest.scalar-nan
    #[test]
    fn observed_scalar() {
        assert_eq!(scalar(&call(&[Value::Num(-5.0)]).unwrap()), 5.0);
    }

    /// case `empty` — normest.zero
    #[test]
    fn observed_empty() {
        let empty = Value::Tensor(Tensor::new(vec![], vec![0, 0]).unwrap());
        assert_eq!(scalar(&call(&[empty]).unwrap()), 0.0);
    }

    /// case `with-nan` — normest.scalar-nan
    #[test]
    fn observed_with_nan() {
        let a = mat(2, 2, &[1.0, f64::NAN, 0.0, 1.0]);
        assert!(scalar(&call(&[a]).unwrap()).is_nan());
    }

    /// case `char-input` — normest.char (non-normative; FR-067-11 documented
    /// choice: char is accepted and computed on its codes)
    #[test]
    fn observed_char_input() {
        let chars = Value::CharArray(CharArray::new_row("ab"));
        let estimate = scalar(&call(&[chars]).unwrap());
        let reference = norm_full(mat(1, 2, &[97.0, 98.0]));
        assert!(rel_diff(estimate, reference) <= 1e-15, "{estimate}");
        assert!(rel_diff(estimate, 137.88763541376724) <= 1e-15);
    }

    /// case `err-int` — normest.error-int
    #[test]
    fn observed_err_int() {
        let ints = Tensor::new_integer(IntegerStorage::I32(vec![1, 3, 2, 4]), vec![2, 2]).unwrap();
        assert_eq!(
            identifier(call(&[Value::Tensor(ints)])),
            "RunMat:mtimes:integerNotSupported"
        );
    }

    /// case `err-3d` — normest.error-3d
    #[test]
    fn observed_err_3d() {
        let ones = Value::Tensor(Tensor::new(vec![1.0; 8], vec![2, 2, 2]).unwrap());
        assert_eq!(identifier(call(&[ones])), "RunMat:transpose:NDArray");
    }

    /// case `err-no-arg` — normest.error-no-arg
    #[test]
    fn observed_err_no_arg() {
        assert_eq!(identifier(call(&[])), "RunMat:minrhs");
    }

    // --- Documented choices --------------------------------------------------

    /// FR-067-13 / D-067-2: accuracy on a larger sparse stoichiometric-style
    /// matrix without densifying (relative difference within the default
    /// tolerance's reach), and a start vector in the null space.
    #[test]
    fn documented_choice_larger_sparse_and_null_start() {
        let (m, n) = (40, 60);
        let mut rows = vec![0.0; m * n];
        for j in 0..n {
            rows[(j % m) * n + j] = -1.0;
            rows[((j * 7 + 3) % m) * n + j] = 1.0 + (j % 3) as f64;
        }
        let estimate = scalar(&call(&[sparse(m, n, &rows)]).unwrap());
        let reference = norm_full(mat(m, n, &rows));
        assert!(rel_diff(estimate, reference) <= OBSERVED_BOUND);
        // [1 -1]: the column-sum start vector [1; 1] is in the null space.
        let estimate = scalar(&call(&[mat(1, 2, &[1.0, -1.0])]).unwrap());
        assert!(rel_diff(estimate, 2f64.sqrt()) <= 1e-15, "{estimate}");
    }

    /// FR-067-14: unobserved inputs, invalid tol, arity and output count; Inf.
    #[test]
    fn documented_choice_unsupported_and_outputs() {
        const U: &str = "RunMat:normest:UnsupportedOption";
        assert_eq!(identifier(call(&[Value::Bool(true)])), U);
        let logical = LogicalArray::new(vec![1, 0, 0, 1], vec![2, 2]).unwrap();
        assert_eq!(identifier(call(&[Value::LogicalArray(logical)])), U);
        assert_eq!(identifier(call(&[Value::from("ab")])), U);
        assert_eq!(identifier(call(&[Value::Num(1.0), Value::Num(0.0)])), U);
        assert_eq!(identifier(call(&[Value::Num(1.0), Value::Num(-1.0)])), U);
        assert_eq!(
            identifier(call(&[Value::Num(1.0), Value::Num(f64::NAN)])),
            U
        );
        assert_eq!(
            identifier(call(&[Value::Num(1.0), mat(1, 2, &[1e-3, 1e-3])])),
            U
        );
        assert_eq!(
            identifier(call(&[Value::Num(1.0), Value::Num(1e-6), Value::Num(1.0)])),
            U
        );
        assert_eq!(
            identifier(call(&[Value::Int(runmat_value::IntValue::I32(3))])),
            "RunMat:mtimes:integerNotSupported"
        );
        assert_eq!(
            scalar(&call(&[mat(1, 2, &[1.0, f64::INFINITY])]).unwrap()),
            f64::INFINITY
        );
        let _guard = crate::output_count::push_output_count(Some(3));
        let err = block_on(normest_builtin(vec![Value::Num(1.0)])).unwrap_err();
        assert_eq!(err.identifier(), Some("RunMat:normest:TooManyOutputs"));
    }

    /// FR-067-12 (non-normative normest.iteration-count): cnt is 0 for
    /// all-zero input and positive otherwise; its value is not compared with
    /// the observed counts.
    #[test]
    fn documented_choice_count() {
        let (_, cnt) = call2(vec![mat(3, 2, &[0.0; 6])]);
        assert_eq!(count(&cnt), 0.0);
        let (_, cnt) = call2(vec![mat(3, 3, &SQUARE)]);
        assert!(count(&cnt) >= 1.0);
    }

    /// FR-067-16 (unresolved, normest.q-warning-coverage): no warning.
    #[test]
    fn unresolved_no_warning() {
        assert!(call(&[Value::Num(2.0)]).is_ok());
    }
}
