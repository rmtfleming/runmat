//! Clean-room `condest` (1-norm condition number of a square matrix).
//!
//! Clean-room feature `specs/055-numeric-n3n4/`. Normative behaviour comes
//! only from the approved export `condest` 0.2.0 (see that feature's
//! `provenance.md`): the observed values, agreement with `cond(A, 1)` on the
//! small observed matrices, `Inf` for singular input, `0` for `0x0`, the
//! two-output `[c, v]` norms, and the error identifiers (raised as
//! `RunMat:*`, D-055-1). That `condest` is only an estimate is a
//! non-normative inference (`condest.estimate`).
//!
//! Algorithm (documented choice, D-055-3): RunMat computes the 1-norm
//! condition number exactly, `norm(A,1) * norm(inv(A),1)`, with the inverse
//! from an LU factorisation with partial pivoting (nalgebra). `v` is the
//! column of `inv(A)` with the largest 1-norm, scaled to unit 1-norm, so
//! `norm(A*v,1) = 1 / norm(inv(A),1)` (textbook identity, Higham,
//! "Accuracy and Stability of Numerical Algorithms", ch. 15).

use nalgebra::{linalg::SVD, DMatrix};
use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{NumericDType, Tensor, Value};

use crate::builtins::common::map_control_flow_with_builtin;
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const NAME: &str = "condest";

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
    "Square real double matrix, sparse or full.",
);
const OUT_C: BuiltinParamDescriptor = param(
    "c",
    BuiltinParamArity::Required,
    "1-norm condition number (double scalar).",
);
const OUT_V: BuiltinParamDescriptor = param(
    "v",
    BuiltinParamArity::Optional,
    "Column vector with norm(v,1) = 1 and small norm(A*v,1).",
);

const CONDEST_INPUTS: [BuiltinParamDescriptor; 1] = [IN_A];
const CONDEST_OUTPUTS_C: [BuiltinParamDescriptor; 1] = [OUT_C];
const CONDEST_OUTPUTS_CV: [BuiltinParamDescriptor; 2] = [OUT_C, OUT_V];

const CONDEST_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "c = condest(A)",
        inputs: &CONDEST_INPUTS,
        outputs: &CONDEST_OUTPUTS_C,
    },
    BuiltinSignatureDescriptor {
        label: "[c, v] = condest(A)",
        inputs: &CONDEST_INPUTS,
        outputs: &CONDEST_OUTPUTS_CV,
    },
];

const NON_SQUARE: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.CONDEST.NON_SQUARE",
    identifier: Some("RunMat:condest:NonSquareMatrix"),
    when: "A is not square.",
    message: "condest: A must be a square matrix",
};

const LU_INPUT_TYPE: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.CONDEST.LU_INPUT_TYPE",
    identifier: Some("RunMat:lu:inputType"),
    when: "A is a char matrix.",
    message: "condest: A must be a numeric matrix",
};

const INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.CONDEST.INVALID_INPUT",
    identifier: Some("RunMat:condest:InvalidInput"),
    when: "No input is given, or A is not 2-D.",
    message: "condest: expected condest(A)",
};

const UNSUPPORTED_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.CONDEST.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:condest:UnsupportedOption"),
    when: "Unobserved inputs: extra arguments, single, complex, integer, logical, non-finite entries, other classes.",
    message: "condest: this form is not yet specified",
};

const TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.CONDEST.TOO_MANY_OUTPUTS",
    identifier: Some("RunMat:condest:TooManyOutputs"),
    when: "More than two outputs were requested.",
    message: "condest: too many output arguments",
};

const INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.CONDEST.INTERNAL",
    identifier: Some("RunMat:condest:Internal"),
    when: "The runtime cannot assemble the result.",
    message: "condest: internal runtime failure",
};

const CONDEST_ERRORS: [BuiltinErrorDescriptor; 6] = [
    NON_SQUARE,
    LU_INPUT_TYPE,
    INVALID_INPUT,
    UNSUPPORTED_OPTION,
    TOO_MANY_OUTPUTS,
    INTERNAL,
];

pub const CONDEST_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &CONDEST_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &CONDEST_ERRORS,
};

fn condest_error(
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
    name = "condest",
    category = "math/linalg/solve",
    summary = "1-norm condition number of a square matrix, with a vector v.",
    keywords = "condest,condition number,1-norm,cond,sparse",
    accel = "cpu",
    descriptor(crate::builtins::math::linalg::solve::condest::CONDEST_DESCRIPTOR),
    builtin_path = "crate::builtins::math::linalg::solve::condest"
)]
pub(crate) async fn condest_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    let out_count = crate::output_count::current_output_count();
    // FR-055-29
    if out_count.is_some_and(|count| count > 2) {
        return Err(condest_error(
            &TOO_MANY_OUTPUTS,
            "at most two outputs ([c, v]) are supported",
        ));
    }
    if args.is_empty() {
        return Err(condest_error(&INVALID_INPUT, "expected condest(A)"));
    }
    if args.len() > 1 {
        return Err(condest_error(
            &UNSUPPORTED_OPTION,
            "arguments after A are not yet specified",
        ));
    }
    // FR-055-31: gpuArray input is gathered.
    let a = gather_if_needed_async(&args[0])
        .await
        .map_err(|err| map_control_flow_with_builtin(err, NAME))?;
    let (data, n) = square_input(&a)?;
    let (c, v) = condition(&data, n)?;
    let c = Value::Num(c);
    match out_count {
        None => Ok(c),
        Some(0) => Ok(Value::OutputList(Vec::new())),
        Some(1) => Ok(Value::OutputList(vec![c])),
        Some(_) => {
            let v = Tensor::new(v, vec![n, 1])
                .map(Value::Tensor)
                .map_err(|err| condest_error(&INTERNAL, err))?;
            Ok(Value::OutputList(vec![c, v]))
        }
    }
}

fn unsupported(detail: &str) -> RuntimeError {
    condest_error(&UNSUPPORTED_OPTION, detail)
}

/// FR-055-24, FR-055-25, FR-055-30: a square real double matrix, full or
/// sparse, as column-major data and its order.
fn square_input(a: &Value) -> BuiltinResult<(Vec<f64>, usize)> {
    let (data, shape) = match a {
        Value::Num(v) => (vec![*v], vec![1, 1]),
        Value::Tensor(t) => {
            if t.integer_storage().is_some() || t.numeric_dtype() != NumericDType::F64 {
                return Err(unsupported("A must be a double matrix"));
            }
            (t.materialize_f64(), t.shape.clone())
        }
        Value::SparseTensor(s) => {
            if s.is_complex()
                || s.is_logical()
                || s.integer_storage().is_some()
                || s.numeric_dtype() != Some(NumericDType::F64)
            {
                return Err(unsupported("A must be a real double matrix"));
            }
            let dense = s.to_dense().map_err(|err| condest_error(&INTERNAL, err))?;
            (dense.materialize_f64(), vec![s.rows, s.cols])
        }
        // FR-055-25 (condest.error-char)
        Value::CharArray(_) => {
            return Err(condest_error(&LU_INPUT_TYPE, "A must not be char"));
        }
        _ => return Err(unsupported("A must be a real double matrix")),
    };
    if shape.iter().skip(2).any(|&dim| dim != 1) {
        return Err(condest_error(&INVALID_INPUT, "A must be a 2-D matrix"));
    }
    let rows = shape.first().copied().unwrap_or(1);
    let cols = shape.get(1).copied().unwrap_or(1);
    // FR-055-24 (condest.error-nonsquare)
    if rows != cols {
        return Err(condest_error(
            &NON_SQUARE,
            format!("A is {rows}x{cols}, not square"),
        ));
    }
    if data.iter().any(|v| !v.is_finite()) {
        return Err(unsupported(
            "A with NaN or Inf entries is not yet specified",
        ));
    }
    Ok((data, rows))
}

fn column_sums(m: &DMatrix<f64>) -> Vec<f64> {
    m.column_iter()
        .map(|col| col.iter().map(|v| v.abs()).sum())
        .collect()
}

fn max_index(values: &[f64]) -> (usize, f64) {
    values.iter().copied().enumerate().fold(
        (0, 0.0),
        |best, (i, v)| if v > best.1 { (i, v) } else { best },
    )
}

/// FR-055-19..23.
fn condition(data: &[f64], n: usize) -> BuiltinResult<(f64, Vec<f64>)> {
    // FR-055-22 (condest.empty)
    if n == 0 {
        return Ok((0.0, Vec::new()));
    }
    let a = DMatrix::from_column_slice(n, n, data);
    let (_, norm_a) = max_index(&column_sums(&a));
    match a.clone().lu().try_inverse() {
        Some(inv) => {
            let sums = column_sums(&inv);
            let (j, norm_inv) = max_index(&sums);
            let v = inv.column(j).iter().map(|x| x / norm_inv).collect();
            Ok((norm_a * norm_inv, v))
        }
        // FR-055-21 (condest.singular). v: documented choice FR-055-26, a
        // right singular vector of the smallest singular value, unit 1-norm.
        None => {
            let svd = SVD::new(a, false, true);
            let v_t = svd
                .v_t
                .ok_or_else(|| condest_error(&INTERNAL, "the SVD returned no right vectors"))?;
            let sigma: Vec<f64> = svd.singular_values.iter().copied().collect();
            let k = (0..sigma.len())
                .min_by(|&i, &j| sigma[i].total_cmp(&sigma[j]))
                .unwrap_or(0);
            let row: Vec<f64> = v_t.row(k).iter().copied().collect();
            let norm: f64 = row.iter().map(|x| x.abs()).sum();
            let v = if norm > 0.0 {
                row.iter().map(|x| x / norm).collect()
            } else {
                row
            };
            Ok((f64::INFINITY, v))
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::excessive_precision)]
mod condest_tests {
    use super::condest_builtin;
    use crate::BuiltinResult;
    use futures::executor::block_on;
    use runmat_value::{CharArray, ComplexTensor, SparseTensor, Tensor, Value};

    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("condest", args))
    }

    fn call2(a: Value) -> (f64, Vec<usize>, Vec<f64>) {
        let _guard = crate::output_count::push_output_count(Some(2));
        match block_on(condest_builtin(vec![a])).unwrap() {
            Value::OutputList(mut outs) => {
                assert_eq!(outs.len(), 2);
                let v = outs.pop().unwrap();
                let c = scalar(&outs.pop().unwrap());
                match v {
                    Value::Tensor(t) => (c, t.shape.clone(), t.materialize_f64()),
                    other => panic!("expected a double column v, got {other:?}"),
                }
            }
            other => panic!("expected two outputs, got {other:?}"),
        }
    }

    fn scalar(value: &Value) -> f64 {
        match value {
            Value::Num(v) => *v,
            other => panic!("expected a double scalar, got {other:?}"),
        }
    }

    /// Row-major literal to a column-major dense matrix.
    fn dense(n: usize, row_major: &[f64]) -> Tensor {
        let mut data = vec![0.0; n * n];
        for r in 0..n {
            for c in 0..n {
                data[r + c * n] = row_major[r * n + c];
            }
        }
        Tensor::new(data, vec![n, n]).unwrap()
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

    fn diag(values: &[f64]) -> Vec<f64> {
        let n = values.len();
        let mut m = vec![0.0; n * n];
        for (i, v) in values.iter().enumerate() {
            m[i * n + i] = *v;
        }
        m
    }

    fn cond1(t: Tensor) -> f64 {
        scalar(
            &block_on(crate::call_builtin_async(
                "cond",
                &[Value::Tensor(t), Value::Num(1.0)],
            ))
            .unwrap(),
        )
    }

    fn identifier(result: BuiltinResult<Value>) -> String {
        match result {
            Ok(value) => panic!("expected an error, got {value:?}"),
            Err(err) => err.identifier().unwrap_or("<none>").to_string(),
        }
    }

    // --- Observed (normative): one test per observed case --------------------

    /// case `identity` — condest.signature-primary, condest.values
    #[test]
    fn observed_identity() {
        assert_eq!(
            scalar(&call(&[sparse(4, 4, &diag(&[1.0; 4]))]).unwrap()),
            1.0
        );
    }

    /// case `diag` — condest.values
    #[test]
    fn observed_diag() {
        let a = sparse(3, 3, &diag(&[1.0, 10.0, 100.0]));
        assert_eq!(scalar(&call(&[a]).unwrap()), 100.0);
    }

    /// case `vs-cond` — condest.vs-cond: a 4x4 sparse tridiagonal matrix
    /// agrees with cond(full(A), 1). The observed matrix is not recorded, so
    /// tridiag(-1, 2, -1) is used (cond1 = 12).
    #[test]
    fn observed_vs_cond() {
        let rows = [
            2.0, -1.0, 0.0, 0.0, -1.0, 2.0, -1.0, 0.0, 0.0, -1.0, 2.0, -1.0, 0.0, 0.0, -1.0, 2.0,
        ];
        let c = scalar(&call(&[sparse(4, 4, &rows)]).unwrap());
        let reference = cond1(dense(4, &rows));
        assert!(
            (c - reference).abs() <= 1e-14 * reference,
            "{c} vs {reference}"
        );
        assert!((c - 12.0).abs() < 1e-12);
    }

    /// case `vs-cond-unsym` — condest.vs-cond
    #[test]
    fn observed_vs_cond_unsym() {
        let rows = [1.0, 2.0, 0.0, 0.0, 1.0, 3.0, 4.0, 0.0, 1.0];
        let c = scalar(&call(&[sparse(3, 3, &rows)]).unwrap());
        let reference = cond1(dense(3, &rows));
        assert!(
            (c - reference).abs() <= 1e-14 * reference,
            "{c} vs {reference}"
        );
        assert!((c - 3.4000000000000004).abs() < 1e-12, "{c}");
    }

    /// case `dense` — condest.values (full input)
    #[test]
    fn observed_dense() {
        let a = Value::Tensor(dense(2, &[2.0, 0.0, 0.0, 8.0]));
        assert_eq!(scalar(&call(&[a]).unwrap()), 4.0);
    }

    /// case `singular` — condest.singular
    #[test]
    fn observed_singular() {
        let a = sparse(2, 2, &[1.0, 2.0, 2.0, 4.0]);
        assert_eq!(scalar(&call(&[a]).unwrap()), f64::INFINITY);
    }

    /// case `zero` — condest.singular
    #[test]
    fn observed_zero() {
        assert_eq!(
            scalar(&call(&[sparse(3, 3, &[0.0; 9])]).unwrap()),
            f64::INFINITY
        );
    }

    /// case `two-out` — condest.two-out
    #[test]
    fn observed_two_out() {
        let (c, shape, v) = call2(sparse(3, 3, &diag(&[1.0, 10.0, 100.0])));
        assert_eq!(c, 100.0);
        assert_eq!(shape, vec![3, 1]);
        let norm_v: f64 = v.iter().map(|x| x.abs()).sum();
        assert!((norm_v - 1.0).abs() < 1e-15);
        let av = [v[0], 10.0 * v[1], 100.0 * v[2]];
        let norm_av: f64 = av.iter().map(|x| x.abs()).sum();
        assert!((norm_av - 1.0).abs() < 1e-15, "norm(A*v,1) = {norm_av}");
    }

    /// case `scalar` — condest.values
    #[test]
    fn observed_scalar() {
        assert_eq!(scalar(&call(&[sparse(1, 1, &[5.0])]).unwrap()), 1.0);
    }

    /// case `empty` — condest.empty
    #[test]
    fn observed_empty() {
        let a = Value::SparseTensor(SparseTensor::new(0, 0, vec![0], vec![], vec![]).unwrap());
        assert_eq!(scalar(&call(&[a]).unwrap()), 0.0);
    }

    /// case `err-nonsquare` — condest.error-nonsquare
    #[test]
    fn observed_err_nonsquare() {
        assert_eq!(
            identifier(call(&[sparse(2, 3, &[1.0; 6])])),
            "RunMat:condest:NonSquareMatrix"
        );
    }

    /// case `err-char` — condest.error-char
    #[test]
    fn observed_err_char() {
        let chars = CharArray::new(vec!['a', 'c', 'b', 'd'], 2, 2).unwrap();
        assert_eq!(
            identifier(call(&[Value::CharArray(chars)])),
            "RunMat:lu:inputType"
        );
    }

    // --- Documented choices --------------------------------------------------

    /// D-055-3: the value is the exact cond(A,1) also on a larger matrix
    /// (condest.estimate and condest.q-larger are non-normative).
    #[test]
    fn documented_choice_exact_value_larger() {
        let n = 6;
        let mut rows = vec![0.0; n * n];
        for r in 0..n {
            for c in 0..n {
                rows[r * n + c] = 1.0 / ((r + c + 1) as f64) + if r == c { 1.0 } else { 0.0 };
            }
        }
        let c = scalar(&call(&[Value::Tensor(dense(n, &rows))]).unwrap());
        let reference = cond1(dense(n, &rows));
        assert!(
            (c - reference).abs() <= 1e-10 * reference,
            "{c} vs {reference}"
        );
    }

    /// FR-055-26: v of a singular matrix is a unit-1-norm null vector, and
    /// v of an empty matrix is 0x1.
    #[test]
    fn documented_choice_v_singular_and_empty() {
        let (c, shape, v) = call2(sparse(2, 2, &[1.0, 2.0, 2.0, 4.0]));
        assert_eq!(c, f64::INFINITY);
        assert_eq!(shape, vec![2, 1]);
        let norm_v: f64 = v.iter().map(|x| x.abs()).sum();
        assert!((norm_v - 1.0).abs() < 1e-15);
        assert!((v[0] + 2.0 * v[1]).abs() < 1e-14);
        let empty = Value::SparseTensor(SparseTensor::new(0, 0, vec![0], vec![], vec![]).unwrap());
        let (c, shape, _) = call2(empty);
        assert_eq!((c, shape), (0.0, vec![0, 1]));
    }

    /// FR-055-30: unobserved inputs.
    #[test]
    fn documented_choice_unsupported() {
        const U: &str = "RunMat:condest:UnsupportedOption";
        let single = Value::Tensor(Tensor::from_f32(vec![1.0], vec![1, 1]).unwrap());
        assert_eq!(identifier(call(&[single])), U);
        let z = Value::ComplexTensor(ComplexTensor::new(vec![(1.0, 1.0)], vec![1, 1]).unwrap());
        assert_eq!(identifier(call(&[z])), U);
        assert_eq!(identifier(call(&[Value::Bool(true)])), U);
        assert_eq!(identifier(call(&[Value::Num(f64::NAN)])), U);
        assert_eq!(identifier(call(&[Value::Num(2.0), Value::Num(2.0)])), U);
    }

    /// FR-055-29: arity and output count.
    #[test]
    fn documented_choice_arity_and_outputs() {
        assert_eq!(identifier(call(&[])), "RunMat:condest:InvalidInput");
        let _guard = crate::output_count::push_output_count(Some(3));
        let err = block_on(condest_builtin(vec![Value::Num(1.0)])).unwrap_err();
        assert_eq!(err.identifier(), Some("RunMat:condest:TooManyOutputs"));
    }

    /// FR-055-33 (unresolved, condest.q-warning-coverage): no warning.
    #[test]
    fn unresolved_no_warning() {
        assert!(call(&[Value::Num(2.0)]).is_ok());
    }
}
