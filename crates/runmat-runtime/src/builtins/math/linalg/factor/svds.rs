//! Clean-room `svds` (a subset of singular values and vectors).
//!
//! Clean-room feature `specs/039-svds/`. Normative behaviour comes only from
//! the approved export `svds` 0.3.0 (see that feature's `provenance.md`).
//! Values and vectors come from a dense SVD of `full(A)` (upstream `svd`,
//! economy size); the requested subset is then selected. Points the export
//! leaves open are either documented choices (tagged with their question id)
//! or raise `RunMat:svds:UnsupportedOption` (`svds.q-options`).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{NumericDType, Tensor, Value};

use crate::builtins::common::{map_control_flow_with_builtin, tensor};
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const NAME: &str = "svds";

/// Default count when `k` is omitted: min(DEFAULT_K, min(size(A)))
/// (documented choice, `svds.q-default-rule`).
const DEFAULT_K: usize = 6;

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
    "2-D real double or single matrix, dense or sparse.",
);
const IN_K: BuiltinParamDescriptor = param(
    "k",
    BuiltinParamArity::Required,
    "Number of singular values; values above min(size(A)) are clipped.",
);
const IN_SIGMA: BuiltinParamDescriptor = param(
    "sigma",
    BuiltinParamArity::Required,
    "'largest', 'smallest', or a numeric target value.",
);

const SVDS_INPUTS_A: [BuiltinParamDescriptor; 1] = [IN_A];
const SVDS_INPUTS_A_K: [BuiltinParamDescriptor; 2] = [IN_A, IN_K];
const SVDS_INPUTS_A_K_SIGMA: [BuiltinParamDescriptor; 3] = [IN_A, IN_K, IN_SIGMA];

const SVDS_OUTPUT_S: [BuiltinParamDescriptor; 1] = [param(
    "s",
    BuiltinParamArity::Required,
    "Full column of the selected singular values, largest first.",
)];
const SVDS_OUTPUT_USV: [BuiltinParamDescriptor; 3] = [
    param(
        "U",
        BuiltinParamArity::Required,
        "Left singular vectors, m-by-k.",
    ),
    param(
        "S",
        BuiltinParamArity::Required,
        "Diagonal k-by-k matrix of singular values.",
    ),
    param(
        "V",
        BuiltinParamArity::Required,
        "Right singular vectors, n-by-k.",
    ),
];

const SVDS_SIGNATURES: [BuiltinSignatureDescriptor; 6] = [
    BuiltinSignatureDescriptor {
        label: "s = svds(A)",
        inputs: &SVDS_INPUTS_A,
        outputs: &SVDS_OUTPUT_S,
    },
    BuiltinSignatureDescriptor {
        label: "s = svds(A, k)",
        inputs: &SVDS_INPUTS_A_K,
        outputs: &SVDS_OUTPUT_S,
    },
    BuiltinSignatureDescriptor {
        label: "s = svds(A, k, sigma)",
        inputs: &SVDS_INPUTS_A_K_SIGMA,
        outputs: &SVDS_OUTPUT_S,
    },
    BuiltinSignatureDescriptor {
        label: "[U, S, V] = svds(A)",
        inputs: &SVDS_INPUTS_A,
        outputs: &SVDS_OUTPUT_USV,
    },
    BuiltinSignatureDescriptor {
        label: "[U, S, V] = svds(A, k)",
        inputs: &SVDS_INPUTS_A_K,
        outputs: &SVDS_OUTPUT_USV,
    },
    BuiltinSignatureDescriptor {
        label: "[U, S, V] = svds(A, k, sigma)",
        inputs: &SVDS_INPUTS_A_K_SIGMA,
        outputs: &SVDS_OUTPUT_USV,
    },
];

const INVALID_A_FLOAT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SVDS.INVALID_A_FLOAT",
    identifier: Some("RunMat:svds:InvalidAFloat"),
    when: "A is not a floating-point (double or single) array, e.g. char.",
    message: "svds: A must be a double or single matrix",
};

const INVALID_K: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SVDS.INVALID_K",
    identifier: Some("RunMat:svds:InvalidK"),
    when: "k is negative, non-integer, non-finite, or not a numeric scalar.",
    message: "svds: k must be a non-negative integer scalar",
};

const INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SVDS.INVALID_INPUT",
    identifier: Some("RunMat:svds:InvalidInput"),
    when: "A has more than two non-singleton dimensions.",
    message: "svds: A must be a 2-D matrix",
};

const UNSUPPORTED_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SVDS.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:svds:UnsupportedOption"),
    when: "Options, other sigma values, or complex input (not yet specified for RunMat).",
    message: "svds: option not yet specified",
};

const TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SVDS.TOO_MANY_OUTPUTS",
    identifier: Some("RunMat:svds:TooManyOutputs"),
    when: "More than three outputs are requested.",
    message: "svds: at most three outputs are supported",
};

const INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SVDS.INTERNAL",
    identifier: Some("RunMat:svds:Internal"),
    when: "The runtime cannot assemble the result.",
    message: "svds: internal runtime failure",
};

const SVDS_ERRORS: [BuiltinErrorDescriptor; 6] = [
    INVALID_A_FLOAT,
    INVALID_K,
    INVALID_INPUT,
    UNSUPPORTED_OPTION,
    TOO_MANY_OUTPUTS,
    INTERNAL,
];

pub const SVDS_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &SVDS_SIGNATURES,
    output_mode: BuiltinOutputMode::ByRequestedOutputCount,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &SVDS_ERRORS,
};

fn svds_error(
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

/// Which singular values to keep.
enum Sigma {
    Largest,
    Smallest,
    Nearest(f64),
}

#[runtime_builtin(
    name = "svds",
    category = "math/linalg/factor",
    summary = "Return a subset of the singular values (and optionally vectors) of a matrix.",
    keywords = "svds,singular values,largest,smallest,sparse,svd",
    accel = "cpu",
    descriptor(crate::builtins::math::linalg::factor::svds::SVDS_DESCRIPTOR),
    builtin_path = "crate::builtins::math::linalg::factor::svds"
)]
async fn svds_builtin(a: Value, rest: Vec<Value>) -> BuiltinResult<Value> {
    let out_count = crate::output_count::current_output_count();
    if out_count.is_some_and(|count| count > 3) {
        return Err(svds_error(
            &TOO_MANY_OUTPUTS,
            "at most three outputs ([U, S, V]) are supported",
        ));
    }
    if rest.len() > 2 {
        // svds.q-options
        return Err(svds_error(
            &UNSUPPORTED_OPTION,
            "arguments after sigma (options) are not yet specified (svds.q-options)",
        ));
    }

    let a = gather(a).await?;
    let mut host_rest = Vec::with_capacity(rest.len());
    for value in rest {
        host_rest.push(gather(value).await?);
    }

    let input = dense_input(a)?;
    let limit = input.rows.min(input.cols);
    let k = match host_rest.first() {
        // svds.default-count; general rule is svds.q-default-rule.
        None => DEFAULT_K.min(limit),
        // svds.k-exceeds: a larger k is clipped silently.
        Some(k) => parse_k(k)?.min(limit),
    };
    let sigma = match host_rest.get(1) {
        None => Sigma::Largest,
        Some(sigma) => parse_sigma(sigma)?,
    };

    let want_vectors = out_count.is_some_and(|count| count >= 2);
    let options = if want_vectors {
        vec![Value::String("econ".into())]
    } else {
        Vec::new()
    };
    let eval = crate::builtins::math::linalg::factor::svd::evaluate(input.value, &options).await?;
    let values = real_data(&eval.singular_values())?;
    let selected = select(&values, k, &sigma);
    let s_values: Vec<f64> = selected.iter().map(|&i| values[i]).collect();

    match out_count {
        Some(0) => Ok(Value::OutputList(Vec::new())),
        Some(count) if count >= 2 => {
            let (u, u_rows) = matrix_data(&eval.u())?;
            let (v, v_rows) = matrix_data(&eval.v())?;
            let mut outputs = vec![
                columns(&u, u_rows, &selected, input.single)?,
                diagonal(&s_values, input.single)?,
                columns(&v, v_rows, &selected, input.single)?,
            ];
            outputs.truncate(count);
            Ok(Value::OutputList(outputs))
        }
        Some(_) => Ok(Value::OutputList(vec![column(&s_values, input.single)?])),
        None => column(&s_values, input.single),
    }
}

async fn gather(value: Value) -> BuiltinResult<Value> {
    gather_if_needed_async(&value)
        .await
        .map_err(|err| map_control_flow_with_builtin(err, NAME))
}

struct DenseInput {
    value: Value,
    rows: usize,
    cols: usize,
    single: bool,
}

/// Dense real form of `A`. Double and single are observed (svds.output-class);
/// char is rejected (svds.error-char) and other non-floating classes follow
/// it (documented choice). Complex input is unobserved (svds.q-options).
fn dense_input(a: Value) -> BuiltinResult<DenseInput> {
    let complex = || {
        svds_error(
            &UNSUPPORTED_OPTION,
            "complex input is not yet specified (svds.q-options)",
        )
    };
    let (tensor, single) = match a {
        Value::Num(n) => (
            Tensor::new(vec![n], vec![1, 1]).map_err(|e| svds_error(&INTERNAL, e))?,
            false,
        ),
        Value::Tensor(t) => {
            if t.integer_storage().is_some() {
                return Err(not_float(&Value::Tensor(t)));
            }
            match t.numeric_dtype() {
                NumericDType::F64 => (t, false),
                NumericDType::F32 => (t, true),
                _ => return Err(not_float(&Value::Tensor(t))),
            }
        }
        Value::SparseTensor(s) => {
            if s.is_complex() {
                return Err(complex());
            }
            if s.is_logical() || s.integer_storage().is_some() {
                return Err(not_float(&Value::SparseTensor(s)));
            }
            let single = match s.numeric_dtype() {
                Some(NumericDType::F64) => false,
                Some(NumericDType::F32) => true,
                _ => return Err(not_float(&Value::SparseTensor(s))),
            };
            (s.to_dense().map_err(|e| svds_error(&INTERNAL, e))?, single)
        }
        Value::Complex(_, _) | Value::ComplexTensor(_) => return Err(complex()),
        other => return Err(not_float(&other)),
    };
    if tensor.shape.iter().skip(2).any(|&dim| dim != 1) {
        return Err(svds_error(&INVALID_INPUT, "A must be a 2-D matrix"));
    }
    let rows = tensor.shape.first().copied().unwrap_or(1);
    let cols = tensor.shape.get(1).copied().unwrap_or(1);
    // Compute in double; single results are converted back at the end.
    let data = tensor.materialize_f64();
    let value =
        Value::Tensor(Tensor::new(data, vec![rows, cols]).map_err(|e| svds_error(&INTERNAL, e))?);
    Ok(DenseInput {
        value,
        rows,
        cols,
        single,
    })
}

fn not_float(value: &Value) -> RuntimeError {
    svds_error(
        &INVALID_A_FLOAT,
        format!(
            "A must be a double or single matrix, got {}",
            crate::builtins::introspection::class::class_name_for_value(value)
        ),
    )
}

/// `k`: a real numeric scalar holding a non-negative integer. Negative is
/// observed to error (svds.error-k-negative); non-integer, non-finite and
/// non-scalar values share that error (documented choice).
fn parse_k(k: &Value) -> BuiltinResult<usize> {
    let raw = match k {
        Value::Num(n) => Some(*n),
        Value::Int(i) => Some(i.to_f64()),
        Value::Tensor(t) if t.len() == 1 => t.materialize_f64().first().copied(),
        _ => None,
    };
    match raw {
        Some(x) if x.is_finite() && x.fract() == 0.0 && x >= 0.0 => Ok(x as usize),
        _ => Err(svds_error(
            &INVALID_K,
            "k must be a non-negative integer scalar",
        )),
    }
}

/// `sigma`: 'largest' / 'smallest' (matched case-insensitively, documented
/// choice) or a real numeric scalar target. Anything else is unobserved
/// (svds.q-options).
fn parse_sigma(sigma: &Value) -> BuiltinResult<Sigma> {
    let text = match sigma {
        Value::CharArray(chars) if chars.rows <= 1 => Some(chars.data.iter().collect::<String>()),
        Value::String(text) => Some(text.clone()),
        Value::StringArray(array) if array.data.len() == 1 => Some(array.data[0].clone()),
        _ => None,
    };
    if let Some(text) = text {
        return match text.to_ascii_lowercase().as_str() {
            "largest" => Ok(Sigma::Largest),
            "smallest" => Ok(Sigma::Smallest),
            _ => Err(svds_error(
                &UNSUPPORTED_OPTION,
                format!("sigma '{text}' is not yet specified (svds.q-options)"),
            )),
        };
    }
    let target = match sigma {
        Value::Num(n) => Some(*n),
        Value::Int(i) => Some(i.to_f64()),
        Value::Tensor(t) if t.len() == 1 && t.integer_storage().is_none() => {
            t.materialize_f64().first().copied()
        }
        _ => None,
    };
    match target {
        Some(x) if x.is_finite() => Ok(Sigma::Nearest(x)),
        _ => Err(svds_error(
            &UNSUPPORTED_OPTION,
            "this sigma (or options) argument is not yet specified (svds.q-options)",
        )),
    }
}

/// Indices into the descending `values` of the `k` selected singular values,
/// in increasing index order (so the values come out largest first —
/// observed for 'largest'; documented choice for the other selections).
fn select(values: &[f64], k: usize, sigma: &Sigma) -> Vec<usize> {
    let p = values.len();
    let k = k.min(p);
    match sigma {
        Sigma::Largest => (0..k).collect(),
        Sigma::Smallest => (p - k..p).collect(),
        Sigma::Nearest(target) => {
            // Nearest first; ties go to the larger value, i.e. the lower
            // index (documented choice, svds.q-sigma-numeric).
            let mut order: Vec<usize> = (0..p).collect();
            order.sort_by(|&i, &j| {
                let di = (values[i] - target).abs();
                let dj = (values[j] - target).abs();
                di.total_cmp(&dj).then(i.cmp(&j))
            });
            let mut chosen: Vec<usize> = order.into_iter().take(k).collect();
            chosen.sort_unstable();
            chosen
        }
    }
}

fn real_data(value: &Value) -> BuiltinResult<Vec<f64>> {
    match value {
        Value::Num(n) => Ok(vec![*n]),
        Value::Tensor(t) => Ok(t.materialize_f64()),
        other => Err(svds_error(
            &INTERNAL,
            format!("unexpected svd result {other:?}"),
        )),
    }
}

/// (column-major data, row count) of a real matrix result from `svd`.
fn matrix_data(value: &Value) -> BuiltinResult<(Vec<f64>, usize)> {
    match value {
        Value::Num(n) => Ok((vec![*n], 1)),
        Value::Tensor(t) => Ok((t.materialize_f64(), t.rows())),
        other => Err(svds_error(
            &INTERNAL,
            format!("unexpected svd factor {other:?}"),
        )),
    }
}

fn make_tensor(data: Vec<f64>, shape: Vec<usize>, single: bool) -> BuiltinResult<Tensor> {
    let tensor = if single {
        Tensor::from_f32(data.into_iter().map(|x| x as f32).collect(), shape)
    } else {
        Tensor::new(data, shape)
    };
    tensor.map_err(|e| svds_error(&INTERNAL, e))
}

/// The singular-value column: full, `[k 1]` (svds.output-class, svds.k-zero).
fn column(values: &[f64], single: bool) -> BuiltinResult<Value> {
    let tensor = make_tensor(values.to_vec(), vec![values.len(), 1], single)?;
    Ok(tensor::tensor_into_value(tensor))
}

fn diagonal(values: &[f64], single: bool) -> BuiltinResult<Value> {
    let k = values.len();
    let mut data = vec![0.0; k * k];
    for (i, v) in values.iter().enumerate() {
        data[i + i * k] = *v;
    }
    Ok(tensor::tensor_into_value(make_tensor(
        data,
        vec![k, k],
        single,
    )?))
}

fn columns(data: &[f64], rows: usize, selected: &[usize], single: bool) -> BuiltinResult<Value> {
    let mut out = Vec::with_capacity(rows * selected.len());
    for &col in selected {
        out.extend_from_slice(&data[col * rows..(col + 1) * rows]);
    }
    Ok(tensor::tensor_into_value(make_tensor(
        out,
        vec![rows, selected.len()],
        single,
    )?))
}

#[cfg(test)]
mod svds_tests {
    use crate::BuiltinResult;
    use futures::executor::block_on;
    use runmat_value::{
        CellArray, CharArray, ComplexTensor, IntValue, LogicalArray, NumericDType, SparseTensor,
        StructValue, Tensor, Value,
    };

    // --- helpers -----------------------------------------------------------

    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("svds", args))
    }

    fn call_outputs(args: &[Value], outputs: usize) -> BuiltinResult<Vec<Value>> {
        match block_on(crate::call_builtin_async_with_outputs(
            "svds", args, outputs,
        ))? {
            Value::OutputList(list) => Ok(list),
            other => Ok(vec![other]),
        }
    }

    fn identifier(result: BuiltinResult<Value>) -> String {
        match result {
            Ok(value) => panic!("expected an error, got {value:?}"),
            Err(err) => err.identifier().unwrap_or("<none>").to_string(),
        }
    }

    /// (dtype, shape, column-major data) of a dense real result.
    fn dense(value: &Value) -> (NumericDType, Vec<usize>, Vec<f64>) {
        match value {
            Value::Num(n) => (NumericDType::F64, vec![1, 1], vec![*n]),
            Value::Tensor(t) => {
                assert!(t.integer_storage().is_none(), "expected float storage");
                (t.numeric_dtype(), t.shape.clone(), t.materialize_f64())
            }
            other => panic!("expected a dense real array, got {other:?}"),
        }
    }

    /// Asserts a dense double column with the given values (tolerance 1e-10).
    fn assert_double_column(value: &Value, expected: &[f64]) {
        let (dtype, shape, data) = dense(value);
        assert_eq!(dtype, NumericDType::F64, "class double");
        assert_eq!(shape, vec![expected.len(), 1]);
        assert_close(&data, expected, 1e-10);
    }

    fn assert_close(actual: &[f64], expected: &[f64], tol: f64) {
        assert_eq!(actual.len(), expected.len(), "{actual:?} vs {expected:?}");
        for (a, e) in actual.iter().zip(expected) {
            assert!((a - e).abs() < tol, "{actual:?} vs {expected:?}");
        }
    }

    fn sparse_diag(values: &[f64]) -> Value {
        let n = values.len();
        Value::SparseTensor(
            SparseTensor::new(
                n,
                n,
                (0..=n).collect::<Vec<_>>(),
                (0..n).collect::<Vec<_>>(),
                values.to_vec(),
            )
            .unwrap(),
        )
    }

    fn dense_matrix(data: &[f64], rows: usize, cols: usize) -> Value {
        Value::Tensor(Tensor::new(data.to_vec(), vec![rows, cols]).unwrap())
    }

    fn dense_diag(values: &[f64]) -> Value {
        let n = values.len();
        let mut data = vec![0.0; n * n];
        for (i, v) in values.iter().enumerate() {
            data[i + i * n] = *v;
        }
        dense_matrix(&data, n, n)
    }

    /// sparse([2 0; 0 3])
    fn sparse_2x2() -> Value {
        sparse_diag(&[2.0, 3.0])
    }

    /// sparse([1 0 0; 0 2 0])
    fn sparse_rect() -> Value {
        Value::SparseTensor(
            SparseTensor::new(2, 3, vec![0, 1, 2, 2], vec![0, 1], vec![1.0, 2.0]).unwrap(),
        )
    }

    fn num(x: f64) -> Value {
        Value::Num(x)
    }

    fn text(s: &str) -> Value {
        Value::CharArray(CharArray::new_row(s))
    }

    /// Column-major product of (r x m) and (m x c).
    fn matmul(a: &[f64], r: usize, m: usize, b: &[f64], c: usize) -> Vec<f64> {
        let mut out = vec![0.0; r * c];
        for j in 0..c {
            for i in 0..r {
                out[i + j * r] = (0..m).map(|p| a[i + p * r] * b[p + j * m]).sum();
            }
        }
        out
    }

    fn transpose(a: &[f64], r: usize, c: usize) -> Vec<f64> {
        let mut out = vec![0.0; r * c];
        for j in 0..c {
            for i in 0..r {
                out[j + i * c] = a[i + j * r];
            }
        }
        out
    }

    fn identity(n: usize) -> Vec<f64> {
        let mut out = vec![0.0; n * n];
        for i in 0..n {
            out[i + i * n] = 1.0;
        }
        out
    }

    /// Checks the observed [U,S,V] facts: shapes, dense, diag(S), orthonormal
    /// columns, and A*V = U*S.
    fn assert_usv(a_dense: &[f64], m: usize, n: usize, k: usize, outputs: &[Value], diag: &[f64]) {
        assert_eq!(outputs.len(), 3);
        for out in outputs {
            assert!(!matches!(out, Value::SparseTensor(_)), "outputs are full");
        }
        let (_, u_shape, u) = dense(&outputs[0]);
        let (_, s_shape, s) = dense(&outputs[1]);
        let (_, v_shape, v) = dense(&outputs[2]);
        assert_eq!(u_shape, vec![m, k]);
        assert_eq!(s_shape, vec![k, k]);
        assert_eq!(v_shape, vec![n, k]);
        let s_diag: Vec<f64> = (0..k).map(|i| s[i + i * k]).collect();
        assert_close(&s_diag, diag, 1e-10);
        let utu = matmul(&transpose(&u, m, k), k, m, &u, k);
        assert_close(&utu, &identity(k), 1e-10);
        let vtv = matmul(&transpose(&v, n, k), k, n, &v, k);
        assert_close(&vtv, &identity(k), 1e-10);
        let av = matmul(a_dense, m, n, &v, k);
        let us = matmul(&u, m, k, &s, k);
        assert_close(&av, &us, 1e-10);
    }

    fn assert_no_warnings<T>(f: impl FnOnce() -> T) -> T {
        crate::warning_store::reset();
        let out = f();
        let warnings = crate::warning_store::take_all();
        assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");
        out
    }

    // --- Observed (normative): one test per observed case --------------------

    /// case `k` — svds.signature-primary, svds.output-class, svds.largest
    #[test]
    fn observed_k() {
        let s = call(&[sparse_2x2(), num(2.0)]).expect("svds");
        assert!(!matches!(s, Value::SparseTensor(_)), "result is full");
        assert_double_column(&s, &[3.0, 2.0]);
    }

    /// case `default-8` — svds.default-count
    #[test]
    fn observed_default_8() {
        let a = sparse_diag(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
        assert_double_column(&call(&[a]).unwrap(), &[8.0, 7.0, 6.0, 5.0, 4.0, 3.0]);
    }

    /// case `default-2` — svds.default-count
    #[test]
    fn observed_default_2() {
        assert_double_column(&call(&[sparse_2x2()]).unwrap(), &[3.0, 2.0]);
    }

    /// cases `k-too-large`, `warn-k-too-large` — svds.k-exceeds
    #[test]
    fn observed_k_too_large_is_clipped_without_warning() {
        let s = assert_no_warnings(|| call(&[sparse_2x2(), num(3.0)]).unwrap());
        assert_double_column(&s, &[3.0, 2.0]);
    }

    /// case `warn-k` — svds.no-warning
    #[test]
    fn observed_ordinary_call_issues_no_warning() {
        let s = assert_no_warnings(|| call(&[sparse_2x2(), num(2.0)]).unwrap());
        assert_double_column(&s, &[3.0, 2.0]);
    }

    /// case `k-zero` — svds.k-zero
    #[test]
    fn observed_k_zero_is_0x1_double() {
        let s = call(&[sparse_2x2(), num(0.0)]).unwrap();
        let (dtype, shape, data) = dense(&s);
        assert_eq!(dtype, NumericDType::F64);
        assert_eq!(shape, vec![0, 1]);
        assert!(data.is_empty());
    }

    /// case `dense` — svds.output-class, svds.largest
    #[test]
    fn observed_dense() {
        assert_double_column(
            &call(&[dense_diag(&[4.0, 1.0, 3.0]), num(2.0)]).unwrap(),
            &[4.0, 3.0],
        );
    }

    /// case `rectangular` — svds.rectangular
    #[test]
    fn observed_rectangular() {
        assert_double_column(&call(&[sparse_rect(), num(2.0)]).unwrap(), &[2.0, 1.0]);
    }

    /// case `smallest` — svds.smallest
    #[test]
    fn observed_smallest() {
        let a = sparse_diag(&[4.0, 1.0, 3.0]);
        assert_double_column(&call(&[a, num(1.0), text("smallest")]).unwrap(), &[1.0]);
    }

    /// case `largest` — svds.largest
    #[test]
    fn observed_largest() {
        let a = sparse_diag(&[4.0, 1.0, 3.0]);
        assert_double_column(&call(&[a, num(2.0), text("largest")]).unwrap(), &[4.0, 3.0]);
    }

    /// case `sigma-numeric` — svds.sigma-numeric
    #[test]
    fn observed_sigma_numeric() {
        let a = sparse_diag(&[4.0, 1.0, 3.0]);
        assert_double_column(&call(&[a, num(1.0), num(2.5)]).unwrap(), &[3.0]);
    }

    /// case `zero-matrix` — svds.zero-matrix
    #[test]
    fn observed_zero_matrix() {
        let a = Value::SparseTensor(
            SparseTensor::new(3, 3, vec![0, 0, 0, 0], Vec::<usize>::new(), Vec::new()).unwrap(),
        );
        assert_double_column(&call(&[a, num(2.0)]).unwrap(), &[0.0, 0.0]);
    }

    /// case `single` — svds.output-class
    #[test]
    fn observed_single_input_gives_single() {
        let a = Value::Tensor(
            Tensor::from_f32(
                vec![4.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 3.0],
                vec![3, 3],
            )
            .unwrap(),
        );
        let (dtype, shape, data) = dense(&call(&[a, num(2.0)]).unwrap());
        assert_eq!(dtype, NumericDType::F32, "class single");
        assert_eq!(shape, vec![2, 1]);
        assert_close(&data, &[4.0, 3.0], 1e-5);
    }

    /// case `order` — svds.largest
    #[test]
    fn observed_order_descending() {
        let a = sparse_diag(&[1.0, 5.0, 2.0, 4.0]);
        assert_double_column(&call(&[a, num(4.0)]).unwrap(), &[5.0, 4.0, 2.0, 1.0]);
    }

    /// case `usv-shapes` — svds.usv
    #[test]
    fn observed_usv_shapes() {
        let outputs = call_outputs(&[sparse_diag(&[4.0, 1.0, 3.0]), num(2.0)], 3).unwrap();
        let a = [4.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 3.0];
        assert_usv(&a, 3, 3, 2, &outputs, &[4.0, 3.0]);
    }

    /// case `usv-rect` — svds.usv
    #[test]
    fn observed_usv_rect() {
        let outputs = call_outputs(&[sparse_rect(), num(2.0)], 3).unwrap();
        let a = [1.0, 0.0, 0.0, 2.0, 0.0, 0.0];
        assert_usv(&a, 2, 3, 2, &outputs, &[2.0, 1.0]);
    }

    /// case `err-char` — svds.error-char (identifier in the RunMat namespace)
    #[test]
    fn observed_char_input_errors() {
        assert_eq!(
            identifier(call(&[text("abc"), num(1.0)])),
            "RunMat:svds:InvalidAFloat"
        );
    }

    /// case `err-k-negative` — svds.error-k-negative
    #[test]
    fn observed_negative_k_errors() {
        assert_eq!(
            identifier(call(&[sparse_2x2(), num(-1.0)])),
            "RunMat:svds:InvalidK"
        );
    }

    // --- Documented choices -------------------------------------------------

    /// svds.q-default-rule: default count is min(6, min(size(A))).
    #[test]
    fn documented_choice_default_count_rule() {
        let ten: Vec<f64> = (1..=10).map(f64::from).collect();
        let s = call(&[sparse_diag(&ten)]).unwrap();
        assert_double_column(&s, &[10.0, 9.0, 8.0, 7.0, 6.0, 5.0]);
        // 3x5: min(size) = 3 < 6.
        let mut data = vec![0.0; 15];
        data[0] = 1.0;
        data[1 + 3] = 2.0;
        data[2 + 6] = 3.0;
        let s = call(&[dense_matrix(&data, 3, 5)]).unwrap();
        assert_double_column(&s, &[3.0, 2.0, 1.0]);
        // The default count also applies to [U,S,V] = svds(A).
        let outputs = call_outputs(&[sparse_2x2()], 3).unwrap();
        assert_usv(&[2.0, 0.0, 0.0, 3.0], 2, 2, 2, &outputs, &[3.0, 2.0]);
    }

    /// 'smallest' with k > 1 and 'largest'/'smallest' text forms.
    #[test]
    fn documented_choice_sigma_text_forms_and_order() {
        let a = sparse_diag(&[4.0, 1.0, 3.0]);
        // Selected values are returned largest first.
        assert_double_column(
            &call(&[a.clone(), num(2.0), text("smallest")]).unwrap(),
            &[3.0, 1.0],
        );
        assert_double_column(
            &call(&[a.clone(), num(1.0), Value::String("Smallest".into())]).unwrap(),
            &[1.0],
        );
        assert_double_column(&call(&[a, num(2.0), text("LARGEST")]).unwrap(), &[4.0, 3.0]);
    }

    /// svds.q-sigma-numeric: nearest k values, ties to the larger value,
    /// returned largest first.
    #[test]
    fn documented_choice_sigma_numeric_ties_and_order() {
        let a = sparse_diag(&[4.0, 1.0, 3.0]);
        assert_double_column(
            &call(&[a.clone(), num(2.0), num(2.0)]).unwrap(),
            &[3.0, 1.0],
        );
        // 2 is equidistant from 1 and 3: the larger wins.
        assert_double_column(&call(&[a.clone(), num(1.0), num(2.0)]).unwrap(), &[3.0]);
        let outputs = call_outputs(&[a, num(1.0), num(1.2)], 3).unwrap();
        assert_usv(
            &[4.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 3.0],
            3,
            3,
            1,
            &outputs,
            &[1.0],
        );
    }

    /// Two outputs return [U, S]; k = 0 with three outputs gives empty factors.
    #[test]
    fn documented_choice_two_outputs_and_k_zero_vectors() {
        let outputs = call_outputs(&[sparse_rect(), num(2.0)], 2).unwrap();
        assert_eq!(outputs.len(), 2);
        assert_eq!(dense(&outputs[0]).1, vec![2, 2]);
        assert_eq!(dense(&outputs[1]).1, vec![2, 2]);
        let outputs = call_outputs(&[sparse_rect(), num(0.0)], 3).unwrap();
        assert_eq!(dense(&outputs[0]).1, vec![2, 0]);
        assert_eq!(dense(&outputs[1]).1, vec![0, 0]);
        assert_eq!(dense(&outputs[2]).1, vec![3, 0]);
        // More than three outputs: checked on the builtin itself with the
        // output count pushed, as sibling builtins do (the dispatcher path
        // re-invokes a failed builtin without an output count; see
        // validation.md, deviations).
        let _outputs = crate::output_count::push_output_count(Some(4));
        let result = block_on(super::svds_builtin(sparse_rect(), vec![num(1.0)]));
        assert_eq!(identifier(result), "RunMat:svds:TooManyOutputs");
    }

    /// Single-precision [U,S,V] outputs are single.
    #[test]
    fn documented_choice_single_vectors_are_single() {
        let a = Value::Tensor(Tensor::from_f32(vec![2.0, 0.0, 0.0, 3.0], vec![2, 2]).unwrap());
        let outputs = call_outputs(&[a, num(1.0)], 3).unwrap();
        for out in &outputs {
            assert_eq!(dense(out).0, NumericDType::F32);
        }
    }

    /// k must be a non-negative integer scalar; typed integers are accepted.
    #[test]
    fn documented_choice_k_validation() {
        for k in [
            num(1.5),
            num(f64::NAN),
            num(f64::INFINITY),
            dense_matrix(&[1.0, 2.0], 1, 2),
            dense_matrix(&[], 0, 0),
            text("L"),
            Value::Bool(true),
        ] {
            assert_eq!(
                identifier(call(&[sparse_2x2(), k.clone()])),
                "RunMat:svds:InvalidK",
                "k = {k:?}"
            );
        }
        assert_double_column(
            &call(&[sparse_2x2(), Value::Int(IntValue::I32(1))]).unwrap(),
            &[3.0],
        );
    }

    /// Non-floating A (logical, integer, string, cell, struct) is rejected like
    /// char; arrays with more than two dimensions are rejected as invalid.
    #[test]
    fn documented_choice_non_float_inputs_rejected() {
        for a in [
            Value::LogicalArray(LogicalArray::new(vec![1, 0, 0, 1], vec![2, 2]).unwrap()),
            Value::Bool(true),
            Value::Int(IntValue::I32(4)),
            Value::String("ab".into()),
            Value::Cell(CellArray::new(vec![num(1.0)], 1, 1).unwrap()),
            Value::Struct(StructValue::new()),
        ] {
            assert_eq!(
                identifier(call(&[a.clone(), num(1.0)])),
                "RunMat:svds:InvalidAFloat",
                "A = {a:?}"
            );
        }
        let nd = Value::Tensor(Tensor::new(vec![1.0; 8], vec![2, 2, 2]).unwrap());
        assert_eq!(
            identifier(call(&[nd, num(1.0)])),
            "RunMat:svds:InvalidInput"
        );
    }

    // --- Unresolved (svds.q-options) -----------------------------------------

    #[test]
    fn unresolved_options_and_complex_are_unsupported() {
        let a = sparse_diag(&[4.0, 1.0, 3.0]);
        for extra in [
            vec![num(1.0), text("smallestnz")],
            vec![num(1.0), Value::Struct(StructValue::new())],
            vec![num(1.0), text("largest"), Value::Struct(StructValue::new())],
            vec![num(1.0), text("largest"), text("Tolerance"), num(1e-8)],
            vec![num(1.0), dense_matrix(&[1.0, 2.0], 1, 2)],
        ] {
            let mut args = vec![a.clone()];
            args.extend(extra.clone());
            assert_eq!(
                identifier(call(&args)),
                "RunMat:svds:UnsupportedOption",
                "args = {extra:?}"
            );
        }
        let complex = Value::ComplexTensor(
            ComplexTensor::new(
                vec![(0.0, 3.0), (0.0, 0.0), (0.0, 0.0), (4.0, 0.0)],
                vec![2, 2],
            )
            .unwrap(),
        );
        assert_eq!(
            identifier(call(&[complex, num(1.0)])),
            "RunMat:svds:UnsupportedOption"
        );
    }
}
