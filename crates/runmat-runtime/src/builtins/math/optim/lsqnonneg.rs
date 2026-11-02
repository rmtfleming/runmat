//! Clean-room `lsqnonneg` (nonnegative linear least squares).
//!
//! Clean-room feature `specs/067-numeric-n6/`. Normative behaviour comes only
//! from the approved export `lsqnonneg` 0.2.0 (pin `e364206`, see that
//! feature's `provenance.md`): x >= 0 as a full double column, the observed
//! solutions, residual/resnorm/lambda relations, exitflag 1 and an output
//! structure with `iterations`, acceptance of `optimset` TolX/Display, the
//! empty case, and the error identifiers (raised as `RunMat:*`, D-067-1).
//! `output.algorithm`/`output.message` contents and the choice among
//! non-unique minimisers are not specified (`lsqnonneg.q-output-struct`,
//! `lsqnonneg.q-nonunique`).
//!
//! Algorithm (documented choice, D-067-4): the active-set method of Lawson
//! and Hanson, *Solving Least Squares Problems* (1974), ch. 23, algorithm
//! NNLS. Each passive-set subproblem is solved as a minimum-norm least-squares
//! problem with nalgebra's SVD (Apache-2.0, already a RunMat dependency).

use nalgebra::{DMatrix, DVector};
use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{CharArray, NumericDType, StructValue, Tensor, Value};

use crate::builtins::common::map_control_flow_with_builtin;
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const NAME: &str = "lsqnonneg";

/// D-067-5: `output.algorithm` (documented choice, `lsqnonneg.q-output-struct`).
pub(crate) const ALGORITHM: &str = "active-set";
/// D-067-5: `output.message` (RunMat's own wording).
const MESSAGE_CONVERGED: &str = "lsqnonneg: optimal nonnegative least-squares solution found.";
const MESSAGE_ITERATION_LIMIT: &str =
    "lsqnonneg: iteration limit reached before the optimality conditions were met.";

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

const IN_C: BuiltinParamDescriptor = param(
    "C",
    BuiltinParamArity::Required,
    "Real double matrix, full or sparse.",
);
const IN_D: BuiltinParamDescriptor = param(
    "d",
    BuiltinParamArity::Required,
    "Real double column vector with as many rows as C.",
);
const IN_OPTIONS: BuiltinParamDescriptor = param(
    "options",
    BuiltinParamArity::Optional,
    "optimset/optimoptions structure (TolX and Display are used).",
);
const OUT_X: BuiltinParamDescriptor = param(
    "x",
    BuiltinParamArity::Required,
    "Nonnegative minimiser, full double column.",
);
const OUT_RESNORM: BuiltinParamDescriptor = param(
    "resnorm",
    BuiltinParamArity::Optional,
    "Squared 2-norm of the residual.",
);
const OUT_RESIDUAL: BuiltinParamDescriptor =
    param("residual", BuiltinParamArity::Optional, "d - C*x.");
const OUT_EXITFLAG: BuiltinParamDescriptor = param(
    "exitflag",
    BuiltinParamArity::Optional,
    "1 when converged, 0 at the iteration limit.",
);
const OUT_OUTPUT: BuiltinParamDescriptor = param(
    "output",
    BuiltinParamArity::Optional,
    "Structure with fields iterations, algorithm and message.",
);
const OUT_LAMBDA: BuiltinParamDescriptor =
    param("lambda", BuiltinParamArity::Optional, "C'*(d - C*x).");

const INPUTS_CD: [BuiltinParamDescriptor; 2] = [IN_C, IN_D];
const INPUTS_CD_OPTIONS: [BuiltinParamDescriptor; 3] = [IN_C, IN_D, IN_OPTIONS];
const OUTPUTS_X: [BuiltinParamDescriptor; 1] = [OUT_X];
const OUTPUTS_ALL: [BuiltinParamDescriptor; 6] = [
    OUT_X,
    OUT_RESNORM,
    OUT_RESIDUAL,
    OUT_EXITFLAG,
    OUT_OUTPUT,
    OUT_LAMBDA,
];

const LSQNONNEG_SIGNATURES: [BuiltinSignatureDescriptor; 3] = [
    BuiltinSignatureDescriptor {
        label: "x = lsqnonneg(C, d)",
        inputs: &INPUTS_CD,
        outputs: &OUTPUTS_X,
    },
    BuiltinSignatureDescriptor {
        label: "[x, resnorm, residual, exitflag, output, lambda] = lsqnonneg(C, d)",
        inputs: &INPUTS_CD,
        outputs: &OUTPUTS_ALL,
    },
    BuiltinSignatureDescriptor {
        label: "[x, resnorm, residual, exitflag, output, lambda] = lsqnonneg(C, d, options)",
        inputs: &INPUTS_CD_OPTIONS,
        outputs: &OUTPUTS_ALL,
    },
];

const NON_DOUBLE: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.LSQNONNEG.NON_DOUBLE",
    identifier: Some("RunMat:lsqnonneg:NonDoubleInput"),
    when: "C or d is not double (single, char, integer, logical or another class).",
    message: "lsqnonneg: C and d must be real double arrays",
};

const COMPLEX: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.LSQNONNEG.COMPLEX",
    identifier: Some("RunMat:lsqnonneg:ComplexCorD"),
    when: "C or d is complex.",
    message: "lsqnonneg: C and d must be real",
};

const SIZE_MISMATCH: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.LSQNONNEG.SIZE_MISMATCH",
    identifier: Some("RunMat:sizeDimensionsMustMatch"),
    when: "d has a different number of rows from C.",
    message: "lsqnonneg: d must have as many rows as C",
};

const DIMAGREE: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.LSQNONNEG.DIMAGREE",
    identifier: Some("RunMat:dimagree"),
    when: "d is not a column vector.",
    message: "lsqnonneg: d must be a column vector",
};

const INPUT_ARG: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.LSQNONNEG.INPUT_ARG",
    identifier: Some("RunMat:lsqnonneg:InputArg"),
    when: "Fewer than two inputs are given.",
    message: "lsqnonneg: expected lsqnonneg(C, d)",
};

const INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.LSQNONNEG.INVALID_INPUT",
    identifier: Some("RunMat:lsqnonneg:InvalidInput"),
    when: "C or d has more than two dimensions.",
    message: "lsqnonneg: C and d must be 2-D",
};

const UNSUPPORTED_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.LSQNONNEG.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:lsqnonneg:UnsupportedOption"),
    when: "Unobserved forms: options other than TolX/Display, an x0 argument, more than three arguments, NaN/Inf entries.",
    message: "lsqnonneg: this form is not yet specified",
};

const TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.LSQNONNEG.TOO_MANY_OUTPUTS",
    identifier: Some("RunMat:lsqnonneg:TooManyOutputs"),
    when: "More than six outputs were requested.",
    message: "lsqnonneg: too many output arguments",
};

const INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.LSQNONNEG.INTERNAL",
    identifier: Some("RunMat:lsqnonneg:Internal"),
    when: "The runtime cannot assemble the result.",
    message: "lsqnonneg: internal runtime failure",
};

const LSQNONNEG_ERRORS: [BuiltinErrorDescriptor; 9] = [
    NON_DOUBLE,
    COMPLEX,
    SIZE_MISMATCH,
    DIMAGREE,
    INPUT_ARG,
    INVALID_INPUT,
    UNSUPPORTED_OPTION,
    TOO_MANY_OUTPUTS,
    INTERNAL,
];

pub const LSQNONNEG_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &LSQNONNEG_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &LSQNONNEG_ERRORS,
};

fn lsqnonneg_error(
    descriptor: &'static BuiltinErrorDescriptor,
    detail: impl Into<String>,
) -> RuntimeError {
    let mut builder = build_runtime_error(format!("{NAME}: {}", detail.into())).with_builtin(NAME);
    if let Some(identifier) = descriptor.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

fn unsupported(detail: impl Into<String>) -> RuntimeError {
    lsqnonneg_error(&UNSUPPORTED_OPTION, detail)
}

// ---------------------------------------------------------------------------
// Builtin
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "lsqnonneg",
    category = "math/optim",
    summary = "Nonnegative linear least squares: minimise norm(C*x - d) subject to x >= 0.",
    keywords = "lsqnonneg,nonnegative least squares,nnls,active set,optimization",
    accel = "cpu",
    descriptor(crate::builtins::math::optim::lsqnonneg::LSQNONNEG_DESCRIPTOR),
    builtin_path = "crate::builtins::math::optim::lsqnonneg"
)]
pub(crate) async fn lsqnonneg_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    let out_count = crate::output_count::current_output_count();
    // FR-067-38
    if out_count.is_some_and(|count| count > 6) {
        return Err(lsqnonneg_error(
            &TOO_MANY_OUTPUTS,
            "at most six outputs are supported",
        ));
    }
    // FR-067-33 (lsqnonneg.error-one-arg)
    if args.len() < 2 {
        return Err(lsqnonneg_error(&INPUT_ARG, "expected lsqnonneg(C, d)"));
    }
    // FR-067-37 (lsqnonneg.q-options)
    if args.len() > 3 {
        return Err(unsupported(
            "arguments after C, d and options are not yet specified",
        ));
    }
    // FR-067-40: gpuArray inputs are gathered.
    let mut gathered = Vec::with_capacity(args.len());
    for arg in &args {
        gathered.push(
            gather_if_needed_async(arg)
                .await
                .map_err(|err| map_control_flow_with_builtin(err, NAME))?,
        );
    }
    let c = real_double(&gathered[0])?;
    let d = real_double(&gathered[1])?;
    // FR-067-32 (lsqnonneg.error-row-d), FR-067-31 (lsqnonneg.error-size)
    if d.cols != 1 {
        return Err(lsqnonneg_error(&DIMAGREE, "d must be a column vector"));
    }
    if d.rows != c.rows {
        return Err(lsqnonneg_error(
            &SIZE_MISMATCH,
            format!("C has {} rows but d has {}", c.rows, d.rows),
        ));
    }
    if c.data.iter().chain(d.data.iter()).any(|v| !v.is_finite()) {
        return Err(unsupported("NaN or Inf entries are not yet specified"));
    }
    let options = Options::parse(gathered.get(2))?;
    let solution = nnls(&c, &d.data, options.tol_x);
    assemble(&c, &d.data, solution, out_count)
}

// ---------------------------------------------------------------------------
// Inputs and options
// ---------------------------------------------------------------------------

/// A dense real double matrix, column-major.
struct Matrix {
    data: Vec<f64>,
    rows: usize,
    cols: usize,
}

impl Matrix {
    fn at(&self, i: usize, j: usize) -> f64 {
        self.data[i + j * self.rows]
    }
}

fn non_double(detail: &str) -> RuntimeError {
    lsqnonneg_error(&NON_DOUBLE, detail)
}

fn shaped(data: Vec<f64>, shape: &[usize]) -> BuiltinResult<Matrix> {
    if shape.iter().skip(2).any(|&dim| dim != 1) {
        return Err(lsqnonneg_error(&INVALID_INPUT, "C and d must be 2-D"));
    }
    Ok(Matrix {
        data,
        rows: shape.first().copied().unwrap_or(1),
        cols: shape.get(1).copied().unwrap_or(1),
    })
}

/// FR-067-21, FR-067-29, FR-067-30, FR-067-36: a real double array, full or
/// sparse (sparse is densified, D-067-6).
fn real_double(value: &Value) -> BuiltinResult<Matrix> {
    match value {
        Value::Num(v) => shaped(vec![*v], &[1, 1]),
        // FR-067-30 (lsqnonneg.error-complex)
        Value::Complex(_, _) | Value::ComplexTensor(_) => {
            Err(lsqnonneg_error(&COMPLEX, "C and d must be real"))
        }
        Value::Tensor(t) => {
            if t.integer_storage().is_some() || t.numeric_dtype() != NumericDType::F64 {
                // FR-067-29 (lsqnonneg.error-single); integer: FR-067-36
                return Err(non_double("C and d must be double"));
            }
            shaped(t.materialize_f64(), &t.shape)
        }
        Value::SparseTensor(s) => {
            if s.is_complex() {
                return Err(lsqnonneg_error(&COMPLEX, "C and d must be real"));
            }
            if s.is_logical()
                || s.integer_storage().is_some()
                || s.numeric_dtype() != Some(NumericDType::F64)
            {
                return Err(non_double("C and d must be double"));
            }
            let dense = s
                .to_dense()
                .map_err(|err| lsqnonneg_error(&INTERNAL, err))?;
            shaped(dense.materialize_f64(), &[s.rows, s.cols])
        }
        // FR-067-29 (lsqnonneg.error-char); other classes: FR-067-36
        _ => Err(non_double("C and d must be double")),
    }
}

struct Options {
    tol_x: Option<f64>,
}

fn option_text(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::StringArray(sa) if sa.data.len() == 1 => Some(sa.data[0].clone()),
        Value::CharArray(CharArray { data, rows: 1, .. }) => Some(data.iter().collect()),
        Value::CharArray(chars) if chars.data.is_empty() => Some(String::new()),
        _ => None,
    }
}

fn is_empty_value(value: &Value) -> bool {
    match value {
        Value::Tensor(t) => t.is_empty(),
        Value::CharArray(c) => c.data.is_empty(),
        Value::Cell(c) => c.data.is_empty(),
        _ => false,
    }
}

impl Options {
    /// FR-067-27 (lsqnonneg.options), FR-067-37 (documented choice for
    /// `lsqnonneg.q-options`).
    fn parse(value: Option<&Value>) -> BuiltinResult<Self> {
        let mut options = Options { tol_x: None };
        let fields: &StructValue = match value {
            None => return Ok(options),
            Some(v) if is_empty_value(v) && !matches!(v, Value::Struct(_)) => return Ok(options),
            Some(Value::Struct(s)) => s,
            Some(_) => {
                return Err(unsupported(
                    "options must be a structure from optimset or optimoptions (an x0 argument is not supported)",
                ))
            }
        };
        for (name, field) in fields.fields.iter() {
            match name.to_ascii_lowercase().as_str() {
                "tolx" | "steptolerance" => {
                    if is_empty_value(field) {
                        continue;
                    }
                    let tol = match field {
                        Value::Num(v) => *v,
                        Value::Tensor(t)
                            if t.len() == 1
                                && t.integer_storage().is_none()
                                && t.numeric_dtype() == NumericDType::F64 =>
                        {
                            t.materialize_f64()[0]
                        }
                        _ => return Err(unsupported("TolX must be a real double scalar")),
                    };
                    if !(tol.is_finite() && tol >= 0.0) {
                        return Err(unsupported("TolX must be finite and non-negative"));
                    }
                    options.tol_x = Some(tol);
                }
                "display" => {
                    if is_empty_value(field) {
                        continue;
                    }
                    let text = option_text(field)
                        .ok_or_else(|| unsupported("Display must be text"))?
                        .to_ascii_lowercase();
                    // No progress output is printed for any accepted value.
                    if !matches!(text.as_str(), "off" | "none" | "final" | "notify" | "iter") {
                        return Err(unsupported(format!(
                            "Display '{text}' is not yet specified"
                        )));
                    }
                }
                // Recorded by optimoptions; it does not affect the solver.
                "solver" => {}
                _ if is_empty_value(field) => {}
                _ => {
                    return Err(unsupported(format!(
                        "option '{name}' is not yet specified (only TolX and Display are supported)"
                    )))
                }
            }
        }
        Ok(options)
    }
}

// ---------------------------------------------------------------------------
// Lawson-Hanson NNLS
// ---------------------------------------------------------------------------

struct Solution {
    x: Vec<f64>,
    iterations: usize,
    converged: bool,
}

/// `C' * (d - C*x)`.
fn gradient(c: &Matrix, d: &[f64], x: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let mut residual = d.to_vec();
    for j in 0..c.cols {
        if x[j] != 0.0 {
            for (i, r) in residual.iter_mut().enumerate() {
                *r -= c.at(i, j) * x[j];
            }
        }
    }
    let w = (0..c.cols)
        .map(|j| (0..c.rows).map(|i| c.at(i, j) * residual[i]).sum())
        .collect();
    (residual, w)
}

/// Minimum-norm least-squares solution on the passive columns.
fn passive_solve(c: &Matrix, d: &[f64], passive: &[usize]) -> Vec<f64> {
    let mut z = vec![0.0; c.cols];
    if passive.is_empty() || c.rows == 0 {
        return z;
    }
    let sub = DMatrix::from_fn(c.rows, passive.len(), |i, k| c.at(i, passive[k]));
    let rhs = DVector::from_column_slice(d);
    let svd = sub.svd(true, true);
    let sigma_max = svd.singular_values.iter().copied().fold(0.0, f64::max);
    let eps = (c.rows.max(passive.len()) as f64) * f64::EPSILON * sigma_max;
    if let Ok(solution) = svd.solve(&rhs, eps) {
        for (k, &j) in passive.iter().enumerate() {
            z[j] = solution[k];
        }
    }
    z
}

/// FR-067-23..25, D-067-4. `tol` is the optimality threshold on the
/// gradient; the default (D-067-5) is `10 * eps * max(m, n) * norm(C, 1) *
/// norm(d, inf)`.
fn nnls(c: &Matrix, d: &[f64], tol_x: Option<f64>) -> Solution {
    let n = c.cols;
    let norm_c1 = (0..n)
        .map(|j| (0..c.rows).map(|i| c.at(i, j).abs()).sum::<f64>())
        .fold(0.0, f64::max);
    let norm_d = d.iter().map(|v| v.abs()).fold(0.0, f64::max);
    let tol = tol_x.unwrap_or(10.0 * f64::EPSILON * (c.rows.max(n) as f64) * norm_c1 * norm_d);
    let max_iterations = 3 * n.max(1) + 10;

    let mut x = vec![0.0; n];
    let mut passive = vec![false; n];
    let mut iterations = 0usize;
    let mut converged = true;
    let (_, mut w) = gradient(c, d, &x);
    loop {
        // Most positive gradient component among the active (zero) set.
        let candidate = (0..n)
            .filter(|&j| !passive[j] && w[j] > tol)
            .max_by(|&a, &b| w[a].total_cmp(&w[b]));
        let Some(t) = candidate else {
            break;
        };
        if iterations >= max_iterations {
            converged = false;
            break;
        }
        iterations += 1;
        passive[t] = true;
        loop {
            let set: Vec<usize> = (0..n).filter(|&j| passive[j]).collect();
            let z = passive_solve(c, d, &set);
            if set.iter().all(|&j| z[j] > 0.0) {
                x = z;
                break;
            }
            // Step towards z as far as feasibility allows; the component that
            // limits the step (and any other that reaches zero) leaves the
            // passive set.
            let mut limiting = None;
            let mut alpha = f64::INFINITY;
            for &j in &set {
                if z[j] <= 0.0 {
                    let step = x[j] / (x[j] - z[j]);
                    if step < alpha {
                        alpha = step;
                        limiting = Some(j);
                    }
                }
            }
            for j in 0..n {
                x[j] += alpha * (z[j] - x[j]);
            }
            if let Some(j) = limiting {
                x[j] = 0.0;
            }
            for &j in &set {
                if x[j] <= 0.0 {
                    passive[j] = false;
                    x[j] = 0.0;
                }
            }
            if !passive.iter().any(|&p| p) {
                break;
            }
        }
        for j in 0..n {
            if !passive[j] {
                x[j] = 0.0;
            }
        }
        w = gradient(c, d, &x).1;
    }
    Solution {
        x,
        iterations,
        converged,
    }
}

// ---------------------------------------------------------------------------
// Outputs
// ---------------------------------------------------------------------------

fn column(values: Vec<f64>) -> BuiltinResult<Value> {
    let len = values.len();
    Tensor::new(values, vec![len, 1])
        .map(Value::Tensor)
        .map_err(|err| lsqnonneg_error(&INTERNAL, err))
}

/// FR-067-21..26, FR-067-39, D-067-5.
fn assemble(
    c: &Matrix,
    d: &[f64],
    solution: Solution,
    out_count: Option<usize>,
) -> BuiltinResult<Value> {
    let x_value = column(solution.x.clone())?;
    let wanted = match out_count {
        None => return Ok(x_value),
        Some(count) => count,
    };
    if wanted == 0 {
        return Ok(Value::OutputList(Vec::new()));
    }
    let (residual, lambda) = gradient(c, d, &solution.x);
    let resnorm: f64 = residual.iter().map(|r| r * r).sum();
    let mut output = StructValue::new();
    output.insert("iterations", Value::Num(solution.iterations as f64));
    output.insert("algorithm", Value::CharArray(CharArray::new_row(ALGORITHM)));
    let message = if solution.converged {
        MESSAGE_CONVERGED
    } else {
        MESSAGE_ITERATION_LIMIT
    };
    output.insert("message", Value::CharArray(CharArray::new_row(message)));
    let all = vec![
        x_value,
        Value::Num(resnorm),
        column(residual)?,
        Value::Num(if solution.converged { 1.0 } else { 0.0 }),
        Value::Struct(output),
        column(lambda)?,
    ];
    Ok(Value::OutputList(all.into_iter().take(wanted).collect()))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod lsqnonneg_tests {
    use super::lsqnonneg_builtin;
    use crate::BuiltinResult;
    use futures::executor::block_on;
    use runmat_value::{
        CharArray, ComplexTensor, IntegerStorage, LogicalArray, SparseTensor, StructValue, Tensor,
        Value,
    };

    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("lsqnonneg", args))
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

    fn eye(n: usize) -> Value {
        let mut rows = vec![0.0; n * n];
        for i in 0..n {
            rows[i * n + i] = 1.0;
        }
        mat(n, n, &rows)
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

    fn optimset(name: &str, value: Value) -> Value {
        block_on(crate::call_builtin_async(
            "optimset",
            &[Value::from(name), value],
        ))
        .unwrap()
    }

    /// A full double column vector.
    fn column(value: &Value) -> Vec<f64> {
        match value {
            Value::Tensor(t) => {
                assert_eq!(t.shape.len(), 2);
                assert_eq!(t.shape[1], 1, "expected a column, got {:?}", t.shape);
                assert_eq!(t.numeric_dtype(), runmat_value::NumericDType::F64);
                t.materialize_f64()
            }
            Value::Num(v) => vec![*v],
            other => panic!("expected a full double column, got {other:?}"),
        }
    }

    fn scalar(value: &Value) -> f64 {
        match value {
            Value::Num(v) => *v,
            other => panic!("expected a double scalar, got {other:?}"),
        }
    }

    struct All {
        x: Vec<f64>,
        resnorm: f64,
        residual: Vec<f64>,
        exitflag: f64,
        output: StructValue,
        lambda: Vec<f64>,
    }

    fn call6(args: Vec<Value>) -> All {
        let _guard = crate::output_count::push_output_count(Some(6));
        match block_on(lsqnonneg_builtin(args)).unwrap() {
            Value::OutputList(outs) => {
                assert_eq!(outs.len(), 6);
                let output = match &outs[4] {
                    Value::Struct(s) => s.clone(),
                    other => panic!("expected an output struct, got {other:?}"),
                };
                All {
                    x: column(&outs[0]),
                    resnorm: scalar(&outs[1]),
                    residual: column(&outs[2]),
                    exitflag: scalar(&outs[3]),
                    output,
                    lambda: column(&outs[5]),
                }
            }
            other => panic!("expected six outputs, got {other:?}"),
        }
    }

    fn close(actual: &[f64], expected: &[f64], tol: f64) {
        assert_eq!(actual.len(), expected.len(), "{actual:?} vs {expected:?}");
        for (a, e) in actual.iter().zip(expected) {
            assert!((a - e).abs() <= tol, "{actual:?} vs {expected:?}");
        }
    }

    /// Row-major C, d -> (residual, C'*residual), computed independently.
    fn reference(
        rows: usize,
        cols: usize,
        c: &[f64],
        d: &[f64],
        x: &[f64],
    ) -> (Vec<f64>, Vec<f64>) {
        let residual: Vec<f64> = (0..rows)
            .map(|i| d[i] - (0..cols).map(|j| c[i * cols + j] * x[j]).sum::<f64>())
            .collect();
        let lambda = (0..cols)
            .map(|j| (0..rows).map(|i| c[i * cols + j] * residual[i]).sum())
            .collect();
        (residual, lambda)
    }

    /// FR-067-24..26: residual, resnorm, lambda relations; exitflag 1; output
    /// fields; KKT conditions (x >= 0, lambda <= 0, lambda = 0 where x > 0).
    fn assert_consistent(all: &All, rows: usize, cols: usize, c: &[f64], d: &[f64]) {
        let (residual, lambda) = reference(rows, cols, c, d, &all.x);
        close(&all.residual, &residual, 1e-12);
        close(&all.lambda, &lambda, 1e-12);
        let resnorm: f64 = residual.iter().map(|r| r * r).sum();
        assert!((all.resnorm - resnorm).abs() <= 1e-12 * (1.0 + resnorm));
        assert_eq!(all.exitflag, 1.0);
        let names: Vec<&str> = all.output.fields.keys().map(String::as_str).collect();
        assert_eq!(names, vec!["iterations", "algorithm", "message"]);
        let iterations = scalar(all.output.fields.get("iterations").unwrap());
        assert!(iterations >= 0.0 && iterations.fract() == 0.0);
        for (x, l) in all.x.iter().zip(&all.lambda) {
            assert!(*x >= 0.0, "x = {:?}", all.x);
            assert!(*l <= 1e-12, "lambda = {:?}", all.lambda);
            if *x > 0.0 {
                assert!(l.abs() <= 1e-12, "lambda = {:?}", all.lambda);
            }
        }
    }

    fn identifier(result: BuiltinResult<Value>) -> String {
        match result {
            Ok(value) => panic!("expected an error, got {value:?}"),
            Err(err) => err.identifier().unwrap_or("<none>").to_string(),
        }
    }

    const COBRA: [f64; 12] = [
        1.0, -1.0, 0.0, 2.0, 0.0, 1.0, -1.0, 0.0, 1.0, 0.0, 1.0, -1.0,
    ];
    const OVER: [f64; 6] = [1.0, 0.0, 0.0, 1.0, 1.0, 1.0];

    // --- Observed (normative): one test per observed case --------------------

    /// case `cobra-zero-rhs` — lsqnonneg.signature-primary,
    /// lsqnonneg.cobra-form, lsqnonneg.output-struct
    #[test]
    fn observed_cobra_zero_rhs() {
        let all = call6(vec![mat(3, 4, &COBRA), col(&[0.0; 3])]);
        assert_eq!(all.x, vec![0.0; 4]);
        assert_eq!(all.resnorm, 0.0);
        assert_eq!(all.residual, vec![0.0; 3]);
        assert_eq!(all.lambda, vec![0.0; 4]);
        assert_consistent(&all, 3, 4, &COBRA, &[0.0; 3]);
    }

    /// case `cobra-one-out` — lsqnonneg.cobra-form
    #[test]
    fn observed_cobra_one_out() {
        let x = call(&[mat(3, 4, &COBRA), col(&[0.0; 3])]).unwrap();
        assert_eq!(column(&x), vec![0.0; 4]);
    }

    /// case `interior` — lsqnonneg.solutions, lsqnonneg.output-class
    #[test]
    fn observed_interior() {
        let all = call6(vec![eye(2), col(&[1.0, 2.0])]);
        close(&all.x, &[1.0, 2.0], 1e-14);
        assert!(all.resnorm.abs() <= 1e-28);
        assert_consistent(&all, 2, 2, &[1.0, 0.0, 0.0, 1.0], &[1.0, 2.0]);
    }

    /// case `bound-active` — lsqnonneg.solutions, lsqnonneg.residual,
    /// lsqnonneg.lambda
    #[test]
    fn observed_bound_active() {
        let all = call6(vec![eye(2), col(&[1.0, -2.0])]);
        close(&all.x, &[1.0, 0.0], 1e-14);
        close(&all.residual, &[0.0, -2.0], 1e-14);
        assert!((all.resnorm - 4.0).abs() <= 1e-14);
        close(&all.lambda, &[0.0, -2.0], 1e-14);
        assert_consistent(&all, 2, 2, &[1.0, 0.0, 0.0, 1.0], &[1.0, -2.0]);
    }

    /// case `all-negative` — lsqnonneg.solutions, lsqnonneg.residual
    #[test]
    fn observed_all_negative() {
        let all = call6(vec![eye(3), col(&[-1.0, -2.0, -3.0])]);
        assert_eq!(all.x, vec![0.0; 3]);
        close(&all.residual, &[-1.0, -2.0, -3.0], 0.0);
        assert_eq!(all.resnorm, 14.0);
        close(&all.lambda, &[-1.0, -2.0, -3.0], 0.0);
    }

    /// case `overdetermined` — lsqnonneg.solutions, lsqnonneg.residual,
    /// lsqnonneg.lambda
    #[test]
    fn observed_overdetermined() {
        let all = call6(vec![mat(3, 2, &OVER), col(&[1.0, 2.0, -5.0])]);
        assert_eq!(all.x, vec![0.0, 0.0]);
        close(&all.residual, &[1.0, 2.0, -5.0], 0.0);
        assert_eq!(all.resnorm, 30.0);
        close(&all.lambda, &[-4.0, -3.0], 0.0);
        assert_consistent(&all, 3, 2, &OVER, &[1.0, 2.0, -5.0]);
    }

    /// case `rank-deficient` — lsqnonneg.lambda; lsqnonneg.nonunique is
    /// non-normative, so the objective and KKT conditions are asserted, not x.
    #[test]
    fn observed_rank_deficient() {
        let c = [1.0, 1.0, 1.0, 1.0];
        let all = call6(vec![mat(2, 2, &c), col(&[2.0, 2.0])]);
        assert!(all.resnorm <= 1e-26, "resnorm = {}", all.resnorm);
        assert!((all.x[0] + all.x[1] - 2.0).abs() <= 1e-13, "{:?}", all.x);
        for l in &all.lambda {
            assert!(l.abs() <= 1e-13);
        }
        assert_consistent(&all, 2, 2, &c, &[2.0, 2.0]);
    }

    /// case `one-out` — lsqnonneg.solutions
    #[test]
    fn observed_one_out() {
        let x = call(&[mat(2, 2, &[2.0, 0.0, 0.0, 4.0]), col(&[2.0, 4.0])]).unwrap();
        close(&column(&x), &[1.0, 1.0], 1e-14);
    }

    /// case `options-tolx` — lsqnonneg.options
    #[test]
    fn observed_options_tolx() {
        let options = optimset("TolX", Value::Num(1e-3));
        let all = call6(vec![mat(3, 2, &OVER), col(&[1.0, 2.0, -5.0]), options]);
        assert_eq!(all.x, vec![0.0, 0.0]);
        assert_eq!(all.resnorm, 30.0);
        close(&all.lambda, &[-4.0, -3.0], 0.0);
        assert_consistent(&all, 3, 2, &OVER, &[1.0, 2.0, -5.0]);
    }

    /// case `options-display` — lsqnonneg.options
    #[test]
    fn observed_options_display() {
        let options = optimset("Display", Value::from("off"));
        let all = call6(vec![eye(2), col(&[1.0, -2.0]), options]);
        close(&all.x, &[1.0, 0.0], 1e-14);
        assert!((all.resnorm - 4.0).abs() <= 1e-14);
        close(&all.lambda, &[0.0, -2.0], 1e-14);
    }

    /// case `sparse` — lsqnonneg.output-class (full x for sparse C)
    #[test]
    fn observed_sparse() {
        let all = call6(vec![sparse(3, 2, &OVER), col(&[1.0, 2.0, -5.0])]);
        assert_eq!(all.x, vec![0.0, 0.0]);
        assert_eq!(all.resnorm, 30.0);
        close(&all.lambda, &[-4.0, -3.0], 0.0);
        assert_consistent(&all, 3, 2, &OVER, &[1.0, 2.0, -5.0]);
    }

    /// case `empty-rows` — lsqnonneg.empty
    #[test]
    fn observed_empty_rows() {
        let c = Value::Tensor(Tensor::new(vec![], vec![0, 2]).unwrap());
        let d = Value::Tensor(Tensor::new(vec![], vec![0, 1]).unwrap());
        let all = call6(vec![c, d]);
        assert_eq!(all.x, vec![0.0, 0.0]);
        assert_eq!(all.resnorm, 0.0);
        assert!(all.residual.is_empty());
        assert_eq!(all.lambda, vec![0.0, 0.0]);
        assert_eq!(all.exitflag, 1.0);
    }

    /// case `err-single` — lsqnonneg.error-single
    #[test]
    fn observed_err_single() {
        let c = Value::Tensor(Tensor::from_f32(vec![1.0, 0.0, 0.0, 1.0], vec![2, 2]).unwrap());
        let d = Value::Tensor(Tensor::from_f32(vec![1.0, -2.0], vec![2, 1]).unwrap());
        assert_eq!(identifier(call(&[c, d])), "RunMat:lsqnonneg:NonDoubleInput");
    }

    /// case `err-size` — lsqnonneg.error-size
    #[test]
    fn observed_err_size() {
        assert_eq!(
            identifier(call(&[eye(2), col(&[1.0, 2.0, 3.0])])),
            "RunMat:sizeDimensionsMustMatch"
        );
    }

    /// case `err-complex` — lsqnonneg.error-complex
    #[test]
    fn observed_err_complex() {
        // [1 1i; 0 1], column-major.
        let c = ComplexTensor::new(
            vec![(1.0, 0.0), (0.0, 0.0), (0.0, 1.0), (1.0, 0.0)],
            vec![2, 2],
        )
        .unwrap();
        assert_eq!(
            identifier(call(&[Value::ComplexTensor(c), col(&[1.0, 1.0])])),
            "RunMat:lsqnonneg:ComplexCorD"
        );
    }

    /// case `err-char` — lsqnonneg.error-single (char C)
    #[test]
    fn observed_err_char() {
        let c = Value::CharArray(CharArray::new_row("ab"));
        assert_eq!(
            identifier(call(&[c, col(&[1.0, 2.0])])),
            "RunMat:lsqnonneg:NonDoubleInput"
        );
    }

    /// case `err-one-arg` — lsqnonneg.error-one-arg
    #[test]
    fn observed_err_one_arg() {
        assert_eq!(identifier(call(&[eye(2)])), "RunMat:lsqnonneg:InputArg");
    }

    /// case `err-row-d` — lsqnonneg.error-row-d
    #[test]
    fn observed_err_row_d() {
        assert_eq!(
            identifier(call(&[eye(2), mat(1, 2, &[1.0, 2.0])])),
            "RunMat:dimagree"
        );
    }

    // --- Documented choices --------------------------------------------------

    /// FR-067-34 / D-067-4: a larger problem with a mix of active and passive
    /// components satisfies the KKT conditions, and a COBRA-sized sparse
    /// stoichiometric matrix with a nonzero right-hand side is solved.
    #[test]
    fn documented_choice_kkt_larger() {
        let (m, n) = (12, 8);
        let mut c = vec![0.0; m * n];
        for i in 0..m {
            for j in 0..n {
                c[i * n + j] = (((i * 31 + j * 17) % 23) as f64 - 11.0) / 7.0;
            }
        }
        let d: Vec<f64> = (0..m).map(|i| ((i * 13) % 9) as f64 - 4.0).collect();
        let all = call6(vec![mat(m, n, &c), col(&d)]);
        assert_consistent(&all, m, n, &c, &d);
        assert!(all.x.iter().any(|&v| v > 0.0) && all.x.contains(&0.0));

        let s = [
            -1.0, 0.0, 1.0, 0.0, 0.0, 1.0, -1.0, 0.0, 0.0, 1.0, 0.0, 1.0, -1.0, -1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, -1.0,
        ];
        let d = [1.0, -0.5, 0.25, 2.0];
        let all = call6(vec![sparse(4, 5, &s), col(&d)]);
        assert_consistent(&all, 4, 5, &s, &d);
    }

    /// FR-067-36..38: unobserved classes, options, arity and outputs.
    #[test]
    fn documented_choice_unsupported() {
        const U: &str = "RunMat:lsqnonneg:UnsupportedOption";
        const N: &str = "RunMat:lsqnonneg:NonDoubleInput";
        // Mixed single d, integer, logical, cell.
        let d_single = Value::Tensor(Tensor::from_f32(vec![1.0, 1.0], vec![2, 1]).unwrap());
        assert_eq!(identifier(call(&[eye(2), d_single])), N);
        let ints = Tensor::new_integer(IntegerStorage::I32(vec![1, 0, 0, 1]), vec![2, 2]).unwrap();
        assert_eq!(
            identifier(call(&[Value::Tensor(ints), col(&[1.0, 1.0])])),
            N
        );
        let logical = LogicalArray::new(vec![1, 0, 0, 1], vec![2, 2]).unwrap();
        assert_eq!(
            identifier(call(&[Value::LogicalArray(logical), col(&[1.0, 1.0])])),
            N
        );
        assert_eq!(identifier(call(&[])), "RunMat:lsqnonneg:InputArg");
        // N-D C.
        let nd = Value::Tensor(Tensor::new(vec![1.0; 8], vec![2, 2, 2]).unwrap());
        assert_eq!(
            identifier(call(&[nd, col(&[1.0, 1.0])])),
            "RunMat:lsqnonneg:InvalidInput"
        );
        // NaN entries.
        assert_eq!(identifier(call(&[eye(2), col(&[f64::NAN, 1.0])])), U);
        // Options: unsupported field, invalid Display, x0 argument, 4 args.
        let max_iter = optimset("MaxIter", Value::Num(5.0));
        assert_eq!(identifier(call(&[eye(2), col(&[1.0, 1.0]), max_iter])), U);
        let display = optimset("Display", Value::from("loud"));
        assert_eq!(identifier(call(&[eye(2), col(&[1.0, 1.0]), display])), U);
        assert_eq!(
            identifier(call(&[eye(2), col(&[1.0, 1.0]), col(&[0.0, 0.0])])),
            U
        );
        assert_eq!(
            identifier(call(&[
                eye(2),
                col(&[1.0, 1.0]),
                Value::Tensor(Tensor::new(vec![], vec![0, 0]).unwrap()),
                Value::Num(1.0),
            ])),
            U
        );
        // [] options are accepted.
        let empty = Value::Tensor(Tensor::new(vec![], vec![0, 0]).unwrap());
        assert!(call(&[eye(2), col(&[1.0, 1.0]), empty]).is_ok());
        // Too many outputs.
        let _guard = crate::output_count::push_output_count(Some(7));
        let err = block_on(lsqnonneg_builtin(vec![eye(2), col(&[1.0, 1.0])])).unwrap_err();
        assert_eq!(err.identifier(), Some("RunMat:lsqnonneg:TooManyOutputs"));
    }

    /// FR-067-39 / D-067-5 (non-normative lsqnonneg.q-output-struct):
    /// output.algorithm and output.message are char rows with RunMat's values.
    #[test]
    fn documented_choice_output_struct() {
        let all = call6(vec![eye(2), col(&[1.0, 2.0])]);
        let text = |name: &str| match all.output.fields.get(name) {
            Some(Value::CharArray(c)) => c.data.iter().collect::<String>(),
            other => panic!("expected char {name}, got {other:?}"),
        };
        assert_eq!(text("algorithm"), super::ALGORITHM);
        assert!(!text("message").is_empty());
    }

    /// FR-067-41 (unresolved, lsqnonneg.q-warning-coverage): no warning.
    #[test]
    fn unresolved_no_warning() {
        assert!(call(&[mat(2, 2, &[1.0, 1.0, 1.0, 1.0]), col(&[2.0, 2.0])]).is_ok());
    }
}
