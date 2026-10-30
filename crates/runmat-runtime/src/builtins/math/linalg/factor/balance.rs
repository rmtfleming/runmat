//! Clean-room `balance` (feature `specs/047-balance/`).
//!
//! Normative behaviour comes only from the approved export `balance` 0.2.0
//! (see that feature's `provenance.md`): `T\A*T = B`, eigenvalues preserved,
//! power-of-two diagonal scaling, `[S, P, B]` with `S = diag(T)` and
//! `P = (1:n)'` when nothing is permuted, unchanged output for an already
//! balanced matrix, input class kept, `RunMat:square` and
//! `RunMat:balance:noBalanceSparse` errors. Exact balanced values are not
//! normative (`balance.q-exact-values`).
//!
//! Algorithm (Parlett & Reinsch, Numer. Math. 13, 1969): isolate eigenvalues
//! by symmetric row/column permutations, then scale the remaining block by
//! powers of two until each row and column have comparable off-diagonal
//! 1-norms. Written from the published description.

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{NumericDType, Tensor, Value};

use crate::builtins::common::{map_control_flow_with_builtin, tensor};
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const NAME: &str = "balance";

/// Safety cap on scaling sweeps; the 0.95 test converges long before this
/// for finite input.
const MAX_SWEEPS: usize = 10_000;

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
    "Full square real matrix (double or single).",
);
const IN_OPT: BuiltinParamDescriptor = param(
    "opt",
    BuiltinParamArity::Required,
    "'noperm' to scale without permuting.",
);
const OUT_B: BuiltinParamDescriptor = param("B", BuiltinParamArity::Required, "Balanced matrix.");
const OUT_T: BuiltinParamDescriptor = param(
    "T",
    BuiltinParamArity::Required,
    "Transformation with B = T\\A*T.",
);
const OUT_S: BuiltinParamDescriptor =
    param("S", BuiltinParamArity::Required, "Scaling column vector.");
const OUT_P: BuiltinParamDescriptor = param(
    "P",
    BuiltinParamArity::Required,
    "Permutation column vector.",
);

const INPUTS_A: [BuiltinParamDescriptor; 1] = [IN_A];
const INPUTS_A_OPT: [BuiltinParamDescriptor; 2] = [IN_A, IN_OPT];
const OUTPUTS_B: [BuiltinParamDescriptor; 1] = [OUT_B];
const OUTPUTS_TB: [BuiltinParamDescriptor; 2] = [OUT_T, OUT_B];
const OUTPUTS_SPB: [BuiltinParamDescriptor; 3] = [OUT_S, OUT_P, OUT_B];

const BALANCE_SIGNATURES: [BuiltinSignatureDescriptor; 6] = [
    BuiltinSignatureDescriptor {
        label: "B = balance(A)",
        inputs: &INPUTS_A,
        outputs: &OUTPUTS_B,
    },
    BuiltinSignatureDescriptor {
        label: "[T, B] = balance(A)",
        inputs: &INPUTS_A,
        outputs: &OUTPUTS_TB,
    },
    BuiltinSignatureDescriptor {
        label: "[S, P, B] = balance(A)",
        inputs: &INPUTS_A,
        outputs: &OUTPUTS_SPB,
    },
    BuiltinSignatureDescriptor {
        label: "B = balance(A, 'noperm')",
        inputs: &INPUTS_A_OPT,
        outputs: &OUTPUTS_B,
    },
    BuiltinSignatureDescriptor {
        label: "[T, B] = balance(A, 'noperm')",
        inputs: &INPUTS_A_OPT,
        outputs: &OUTPUTS_TB,
    },
    BuiltinSignatureDescriptor {
        label: "[S, P, B] = balance(A, 'noperm')",
        inputs: &INPUTS_A_OPT,
        outputs: &OUTPUTS_SPB,
    },
];

const SQUARE: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.BALANCE.SQUARE",
    identifier: Some("RunMat:square"),
    when: "A is not square.",
    message: "balance: A must be square",
};

const NO_SPARSE: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.BALANCE.NO_SPARSE",
    identifier: Some("RunMat:balance:noBalanceSparse"),
    when: "A is sparse.",
    message: "balance: sparse matrices are not supported",
};

const INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.BALANCE.INVALID_INPUT",
    identifier: Some("RunMat:balance:InvalidInput"),
    when: "A is not a real double or single 2-D array.",
    message: "balance: A must be a real double or single matrix",
};

const UNSUPPORTED_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.BALANCE.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:balance:UnsupportedOption"),
    when: "Options other than 'noperm', extra arguments, or complex input (not yet specified).",
    message: "balance: option not yet specified",
};

const TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.BALANCE.TOO_MANY_OUTPUTS",
    identifier: Some("RunMat:balance:TooManyOutputs"),
    when: "More than three outputs are requested.",
    message: "balance: at most three outputs are supported",
};

const INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.BALANCE.INTERNAL",
    identifier: Some("RunMat:balance:Internal"),
    when: "The runtime cannot assemble the result.",
    message: "balance: internal runtime failure",
};

const BALANCE_ERRORS: [BuiltinErrorDescriptor; 6] = [
    SQUARE,
    NO_SPARSE,
    INVALID_INPUT,
    UNSUPPORTED_OPTION,
    TOO_MANY_OUTPUTS,
    INTERNAL,
];

pub const BALANCE_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &BALANCE_SIGNATURES,
    output_mode: BuiltinOutputMode::ByRequestedOutputCount,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &BALANCE_ERRORS,
};

fn balance_error(descriptor: &BuiltinErrorDescriptor, detail: impl Into<String>) -> RuntimeError {
    build_runtime_error(format!("{NAME}: {}", detail.into()))
        .with_builtin(NAME)
        .with_identifier(descriptor.identifier.expect("identifier"))
        .build()
}

// ---------------------------------------------------------------------------
// Builtin
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "balance",
    category = "math/linalg/factor",
    summary = "Balance a square matrix by a power-of-two diagonal similarity (with optional permutation) to improve eigenvalue accuracy.",
    keywords = "balance,balancing,eigenvalues,scaling,similarity,permutation,noperm",
    accel = "cpu",
    descriptor(crate::builtins::math::linalg::factor::balance::BALANCE_DESCRIPTOR),
    builtin_path = "crate::builtins::math::linalg::factor::balance"
)]
async fn balance_builtin(a: Value, rest: Vec<Value>) -> BuiltinResult<Value> {
    let out_count = crate::output_count::current_output_count();
    if out_count.is_some_and(|count| count > 3) {
        return Err(balance_error(
            &TOO_MANY_OUTPUTS,
            "at most three outputs ([S, P, B]) are supported",
        ));
    }
    let permute = match rest.as_slice() {
        [] => true,
        [option] => {
            // balance.noperm; other text is unspecified.
            if option_text(&gather(option.clone()).await?).as_deref() == Some("noperm") {
                false
            } else {
                return Err(balance_error(
                    &UNSUPPORTED_OPTION,
                    "the only supported option is 'noperm'",
                ));
            }
        }
        _ => {
            return Err(balance_error(
                &UNSUPPORTED_OPTION,
                "at most one option ('noperm') is supported",
            ))
        }
    };

    let input = real_square(gather(a).await?)?;
    let n = input.n;
    let result = balance_matrix(input.data, n, permute);

    let make = |data: Vec<f64>, shape: Vec<usize>, single: bool| -> BuiltinResult<Value> {
        let tensor = if single {
            Tensor::from_f32(data.into_iter().map(|x| x as f32).collect(), shape)
        } else {
            Tensor::new(data, shape)
        };
        tensor
            .map(tensor::tensor_into_value)
            .map_err(|e| balance_error(&INTERNAL, e))
    };
    let single = input.single;
    let b = make(result.b, vec![n, n], single)?;
    match out_count {
        None => Ok(b),
        Some(0) => Ok(Value::OutputList(Vec::new())),
        Some(1) => Ok(Value::OutputList(vec![b])),
        Some(2) => {
            // T = I(:, P) * diag(S) (documented choice, FR-047-09).
            let mut t = vec![0.0; n * n];
            for i in 0..n {
                t[result.perm[i] + i * n] = result.scale[i];
            }
            Ok(Value::OutputList(vec![make(t, vec![n, n], single)?, b]))
        }
        Some(_) => {
            // balance.three-outputs: column S (input class), column P (double).
            let p: Vec<f64> = result.perm.iter().map(|&i| (i + 1) as f64).collect();
            Ok(Value::OutputList(vec![
                make(result.scale, vec![n, 1], single)?,
                make(p, vec![n, 1], false)?,
                b,
            ]))
        }
    }
}

async fn gather(value: Value) -> BuiltinResult<Value> {
    gather_if_needed_async(&value)
        .await
        .map_err(|err| map_control_flow_with_builtin(err, NAME))
}

fn option_text(value: &Value) -> Option<String> {
    match value {
        Value::CharArray(chars) if chars.rows <= 1 => {
            Some(chars.data.iter().collect::<String>().to_lowercase())
        }
        Value::String(text) => Some(text.to_lowercase()),
        Value::StringArray(array) if array.data.len() == 1 => Some(array.data[0].to_lowercase()),
        _ => None,
    }
}

struct SquareInput {
    data: Vec<f64>,
    n: usize,
    single: bool,
}

/// Dense real square double/single input (FR-047-07, FR-047-11, FR-047-12).
fn real_square(a: Value) -> BuiltinResult<SquareInput> {
    let (data, shape, single) = match a {
        Value::Num(x) => (vec![x], vec![1, 1], false),
        Value::Tensor(t) => {
            if t.integer_storage().is_some() {
                return Err(invalid_class(&Value::Tensor(t)));
            }
            let single = match t.numeric_dtype() {
                NumericDType::F64 => false,
                NumericDType::F32 => true,
                _ => return Err(invalid_class(&Value::Tensor(t))),
            };
            (t.materialize_f64(), t.shape.clone(), single)
        }
        // balance.error-sparse
        Value::SparseTensor(_) => {
            return Err(balance_error(
                &NO_SPARSE,
                "sparse matrices are not supported; use full(A)",
            ))
        }
        // balance.q-complex
        Value::Complex(_, _) | Value::ComplexTensor(_) => {
            return Err(balance_error(
                &UNSUPPORTED_OPTION,
                "complex input is not yet specified (balance.q-complex)",
            ))
        }
        other => return Err(invalid_class(&other)),
    };
    if shape.len() > 2 && shape[2..].iter().product::<usize>() != 1 {
        return Err(balance_error(&INVALID_INPUT, "A must be a 2-D matrix"));
    }
    let rows = shape.first().copied().unwrap_or(1);
    let cols = shape.get(1).copied().unwrap_or(1);
    if rows != cols {
        // balance.error-nonsquare
        return Err(balance_error(&SQUARE, "A must be square"));
    }
    Ok(SquareInput {
        data,
        n: rows,
        single,
    })
}

fn invalid_class(value: &Value) -> RuntimeError {
    balance_error(
        &INVALID_INPUT,
        format!(
            "A must be a real double or single matrix, got {}",
            crate::builtins::introspection::class::class_name_for_value(value)
        ),
    )
}

struct Balanced {
    /// Balanced matrix, column-major n×n.
    b: Vec<f64>,
    /// `perm[i]`: row of the nonzero in column `i` of `T` (zero-based).
    perm: Vec<usize>,
    /// `scale[i]`: value of that nonzero (a power of two).
    scale: Vec<f64>,
}

/// Balance a column-major n×n matrix (Parlett & Reinsch 1969).
fn balance_matrix(mut a: Vec<f64>, n: usize, permute: bool) -> Balanced {
    let mut perm: Vec<usize> = (0..n).collect();
    let mut scale = vec![1.0; n];
    if n == 0 {
        return Balanced { b: a, perm, scale };
    }
    let at = |a: &[f64], r: usize, c: usize| a[r + c * n];

    // Symmetric exchange of rows/columns j and m: A <- P'AP, T <- T*P.
    let swap = |a: &mut Vec<f64>, perm: &mut Vec<usize>, j: usize, m: usize| {
        if j == m {
            return;
        }
        for r in 0..n {
            a.swap(r + j * n, r + m * n);
        }
        for c in 0..n {
            a.swap(j + c * n, m + c * n);
        }
        perm.swap(j, m);
    };

    let mut low = 0usize;
    let mut high = n - 1;
    if permute {
        // Rows with no off-diagonal entries in the active columns isolate an
        // eigenvalue: move them to the bottom.
        loop {
            let found = (0..=high)
                .rev()
                .find(|&j| (0..=high).all(|i| i == j || at(&a, j, i) == 0.0));
            let Some(j) = found else { break };
            swap(&mut a, &mut perm, j, high);
            if high == 0 {
                break;
            }
            high -= 1;
        }
        // Columns with no off-diagonal entries in the active rows: move them
        // to the left.
        while low <= high {
            let found = (low..=high).find(|&j| (low..=high).all(|i| i == j || at(&a, i, j) == 0.0));
            let Some(j) = found else { break };
            swap(&mut a, &mut perm, j, low);
            low += 1;
        }
    }

    // Power-of-two scaling of the active block [low, high].
    if low <= high {
        for _ in 0..MAX_SWEEPS {
            let mut changed = false;
            for i in low..=high {
                let mut c = 0.0f64;
                let mut r = 0.0f64;
                for j in low..=high {
                    if j != i {
                        c += at(&a, j, i).abs();
                        r += at(&a, i, j).abs();
                    }
                }
                // Zero or non-finite norms are left alone (FR-047-12).
                if c == 0.0 || r == 0.0 || !c.is_finite() || !r.is_finite() {
                    continue;
                }
                let total = c + r;
                let mut f = 1.0f64;
                // Track c·f² against r.
                let mut cf = c;
                while cf < r / 2.0 {
                    f *= 2.0;
                    cf *= 4.0;
                }
                while cf >= r * 2.0 {
                    f /= 2.0;
                    cf /= 4.0;
                }
                // New column norm c·f plus new row norm r/f = (c·f² + r)/f.
                if (cf + r) / f < 0.95 * total {
                    changed = true;
                    scale[i] *= f;
                    for c_idx in 0..n {
                        a[i + c_idx * n] /= f;
                    }
                    for r_idx in 0..n {
                        a[r_idx + i * n] *= f;
                    }
                }
            }
            if !changed {
                break;
            }
        }
    }
    Balanced { b: a, perm, scale }
}

#[cfg(test)]
mod balance_tests {
    use futures::executor::block_on;
    use runmat_value::{
        CellArray, CharArray, ComplexTensor, IntValue, LogicalArray, NumericDType, SparseTensor,
        Tensor, Value,
    };

    use crate::BuiltinResult;

    // --- helpers -----------------------------------------------------------

    /// The badly scaled matrix used by most observed cases (row-major).
    const A0: [f64; 9] = [1.0, 100.0, 10000.0, 0.01, 1.0, 100.0, 0.0001, 0.01, 1.0];
    /// The lower-triangular matrix of case `permutation` (row-major).
    const LOWER: [f64; 9] = [1.0, 0.0, 0.0, 2.0, 3.0, 0.0, 4.0, 5.0, 6.0];

    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("balance", args))
    }

    fn call_outputs(args: &[Value], outputs: usize) -> BuiltinResult<Vec<Value>> {
        match block_on(crate::call_builtin_async_with_outputs(
            "balance", args, outputs,
        ))? {
            Value::OutputList(list) => Ok(list),
            other => Ok(vec![other]),
        }
    }

    /// Registered implementation called directly with an output count (the
    /// dispatcher re-invokes a failed builtin without one).
    fn call_direct(args: &[Value], outputs: usize) -> BuiltinResult<Value> {
        let builtin = runmat_builtins::builtin_function_by_name("balance")
            .unwrap_or_else(|| panic!("UndefinedFunction: balance is not registered"));
        let _guard = crate::output_count::push_output_count(Some(outputs));
        block_on((builtin.implementation)(args))
    }

    fn identifier(result: BuiltinResult<Value>) -> String {
        match result {
            Ok(value) => panic!("expected an error, got {value:?}"),
            Err(err) => err.identifier().unwrap_or("<none>").to_string(),
        }
    }

    fn assert_no_warnings<T>(f: impl FnOnce() -> T) -> T {
        crate::warning_store::reset();
        let out = f();
        let warnings = crate::warning_store::take_all();
        assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");
        out
    }

    fn matrix(n: usize, row_major: &[f64]) -> Value {
        let mut data = vec![0.0; n * n];
        for r in 0..n {
            for c in 0..n {
                data[r + c * n] = row_major[r * n + c];
            }
        }
        Value::Tensor(Tensor::new(data, vec![n, n]).unwrap())
    }

    fn single_matrix(n: usize, row_major: &[f64]) -> Value {
        let Value::Tensor(t) = matrix(n, row_major) else {
            unreachable!()
        };
        let data = t.materialize_f64().into_iter().map(|x| x as f32).collect();
        Value::Tensor(Tensor::from_f32(data, vec![n, n]).unwrap())
    }

    /// (dtype, shape, column-major data) of a dense real result.
    fn dense(value: &Value) -> (NumericDType, Vec<usize>, Vec<f64>) {
        match value {
            Value::Num(n) => (NumericDType::F64, vec![1, 1], vec![*n]),
            Value::Tensor(t) => {
                assert!(t.integer_storage().is_none(), "expected float storage");
                (t.numeric_dtype(), t.shape.clone(), t.materialize_f64())
            }
            other => panic!("expected a full real array, got {other:?}"),
        }
    }

    fn double_matrix(value: &Value, n: usize) -> Vec<f64> {
        let (dtype, shape, data) = dense(value);
        assert_eq!(dtype, NumericDType::F64, "class double");
        assert_eq!(shape, vec![n, n]);
        data
    }

    fn row_major_to_col(n: usize, row_major: &[f64]) -> Vec<f64> {
        let (_, _, data) = dense(&matrix(n, row_major));
        data
    }

    fn assert_close(actual: &[f64], expected: &[f64], tol: f64) {
        assert_eq!(actual.len(), expected.len(), "{actual:?} vs {expected:?}");
        for (a, e) in actual.iter().zip(expected) {
            assert!((a - e).abs() <= tol, "{actual:?} vs {expected:?}");
        }
    }

    /// `T\A*T` through RunMat's own `mtimes` and `mldivide`.
    fn similarity(t: &Value, a: &Value) -> Vec<f64> {
        let at = block_on(crate::call_builtin_async("mtimes", &[a.clone(), t.clone()])).unwrap();
        let tat = block_on(crate::call_builtin_async("mldivide", &[t.clone(), at])).unwrap();
        dense(&tat).2
    }

    /// Asserts `T\A*T = B` within `tol`·max|A|.
    fn assert_similarity(t: &Value, a: &Value, b: &Value, tol: f64) {
        let (_, _, a_data) = dense(a);
        let scale = a_data.iter().fold(1.0f64, |m, x| m.max(x.abs()));
        assert_close(&similarity(t, a), &dense(b).2, tol * scale);
    }

    /// Eigenvalues (re, im) through RunMat's `eig`.
    fn eigenvalues(a: &Value) -> Vec<(f64, f64)> {
        match block_on(crate::call_builtin_async("eig", std::slice::from_ref(a))).unwrap() {
            Value::Num(n) => vec![(n, 0.0)],
            Value::Tensor(t) => t.materialize_f64().into_iter().map(|x| (x, 0.0)).collect(),
            Value::ComplexTensor(t) => t.materialize_f64(),
            Value::Complex(re, im) => vec![(re, im)],
            other => panic!("unexpected eig result {other:?}"),
        }
    }

    /// Every eigenvalue of each matrix is within `tol` of one of the other's.
    fn assert_same_eigenvalues(a: &Value, b: &Value, tol: f64) {
        let (ea, eb) = (eigenvalues(a), eigenvalues(b));
        assert_eq!(ea.len(), eb.len());
        for (x, y) in [(&ea, &eb), (&eb, &ea)] {
            for &(re, im) in x.iter() {
                let nearest = y
                    .iter()
                    .map(|&(r2, i2)| (re - r2).hypot(im - i2))
                    .fold(f64::INFINITY, f64::min);
                assert!(nearest <= tol, "{ea:?} vs {eb:?}");
            }
        }
    }

    /// Asserts `T` is diagonal with powers of two on its diagonal; returns
    /// the diagonal.
    fn assert_power_of_two_diagonal(t: &[f64], n: usize) -> Vec<f64> {
        let mut diag = Vec::with_capacity(n);
        for c in 0..n {
            for r in 0..n {
                let v = t[r + c * n];
                if r == c {
                    assert!(v > 0.0 && v.log2().fract() == 0.0, "{v} is not 2^k");
                    diag.push(v);
                } else {
                    assert_eq!(v, 0.0, "T is not diagonal: {t:?}");
                }
            }
        }
        diag
    }

    // --- Observed (normative): one test per observed case --------------------

    /// case `single-output` — balance.signature-primary, balance.output-class
    #[test]
    fn observed_single_output() {
        let b = assert_no_warnings(|| call(&[matrix(3, &A0)])).expect("balance");
        let data = double_matrix(&b, 3);
        assert!(data.iter().all(|x| x.is_finite()));
    }

    /// case `two-outputs` — balance.similarity, balance.power-of-two
    #[test]
    fn observed_two_outputs() {
        let a = matrix(3, &A0);
        let out =
            assert_no_warnings(|| call_outputs(std::slice::from_ref(&a), 2)).expect("balance");
        assert_eq!(out.len(), 2);
        let t = double_matrix(&out[0], 3);
        double_matrix(&out[1], 3);
        assert_similarity(&out[0], &a, &out[1], 1e-12);
        assert_power_of_two_diagonal(&t, 3);
        // B is the single-output result.
        assert_eq!(out[1], call(&[a]).unwrap());
    }

    /// case `eig-preserved` — balance.similarity
    #[test]
    fn observed_eig_preserved() {
        let a = matrix(3, &A0);
        let b = call(std::slice::from_ref(&a)).expect("balance");
        assert_same_eigenvalues(&a, &b, 1e-10);
    }

    /// case `balanced` — balance.already-balanced
    #[test]
    fn observed_balanced() {
        let a = matrix(2, &[1.0, 2.0, 3.0, 4.0]);
        let b = call(&[a]).expect("balance");
        assert_eq!(double_matrix(&b, 2), vec![1.0, 3.0, 2.0, 4.0]);
    }

    /// case `noperm` — balance.noperm
    #[test]
    fn observed_noperm() {
        let a = matrix(3, &A0);
        let b = assert_no_warnings(|| {
            call(&[a.clone(), Value::CharArray(CharArray::new_row("noperm"))])
        })
        .expect("balance");
        double_matrix(&b, 3);
        assert_eq!(b, call(&[a]).unwrap());
    }

    /// case `permutation` — balance.similarity
    #[test]
    fn observed_permutation() {
        let a = matrix(3, &LOWER);
        let out = call_outputs(std::slice::from_ref(&a), 2).expect("balance");
        double_matrix(&out[0], 3);
        double_matrix(&out[1], 3);
        assert_similarity(&out[0], &a, &out[1], 1e-12);
        assert_same_eigenvalues(&a, &out[1], 1e-10);
    }

    /// case `three-outputs` — balance.three-outputs
    #[test]
    fn observed_three_outputs() {
        let a = matrix(3, &A0);
        let out = call_outputs(std::slice::from_ref(&a), 3).expect("balance");
        assert_eq!(out.len(), 3);
        let (s_type, s_shape, s) = dense(&out[0]);
        let (p_type, p_shape, p) = dense(&out[1]);
        assert_eq!((s_type, s_shape), (NumericDType::F64, vec![3, 1]));
        assert_eq!((p_type, p_shape), (NumericDType::F64, vec![3, 1]));
        assert_eq!(p, vec![1.0, 2.0, 3.0]);
        let tb = call_outputs(std::slice::from_ref(&a), 2).unwrap();
        let t = double_matrix(&tb[0], 3);
        assert_eq!(s, assert_power_of_two_diagonal(&t, 3));
        assert_eq!(out[2], tb[1]);
    }

    /// case `scalar` — balance.already-balanced
    #[test]
    fn observed_scalar() {
        let b = call(&[Value::Num(5.0)]).expect("balance");
        assert_eq!(dense(&b), (NumericDType::F64, vec![1, 1], vec![5.0]));
    }

    /// case `single-class` — balance.output-class
    #[test]
    fn observed_single_class() {
        let b = call(&[single_matrix(3, &A0)]).expect("balance");
        let (dtype, shape, _) = dense(&b);
        assert_eq!(dtype, NumericDType::F32);
        assert_eq!(shape, vec![3, 3]);
    }

    /// case `err-nonsquare` — balance.error-nonsquare
    #[test]
    fn observed_nonsquare_errors() {
        let a = Value::Tensor(Tensor::new(vec![1.0; 6], vec![2, 3]).unwrap());
        assert_eq!(identifier(call(&[a])), "RunMat:square");
    }

    /// case `err-sparse` — balance.error-sparse
    #[test]
    fn observed_sparse_errors() {
        let a = SparseTensor::new(2, 2, vec![0, 1, 2], vec![0, 1], vec![1.0, 1.0]).unwrap();
        assert_eq!(
            identifier(call(&[Value::SparseTensor(a)])),
            "RunMat:balance:noBalanceSparse"
        );
    }

    // --- Documented choices ---------------------------------------------------

    /// FR-047-08 (balance.q-exact-values; balance.observed-values is
    /// non-normative): RunMat's own results, recorded for traceability only.
    /// For A0, B coincides with the reference values, while T differs from the
    /// reference by a constant factor (2^11, 2^5, 2^-2 here). For the
    /// lower-triangular matrix, T is the reversal permutation.
    #[test]
    fn documented_choice_values_on_observed_inputs() {
        let out = call_outputs(&[matrix(3, &A0)], 2).unwrap();
        assert_power_of_two_diagonal(&double_matrix(&out[0], 3), 3);
        assert_eq!(
            double_matrix(&out[0], 3),
            row_major_to_col(3, &[2048.0, 0.0, 0.0, 0.0, 32.0, 0.0, 0.0, 0.0, 0.25])
        );
        assert_close(
            &double_matrix(&out[1], 3),
            &row_major_to_col(
                3,
                &[
                    1.0,
                    1.5625,
                    1.220703125,
                    0.64,
                    1.0,
                    0.78125,
                    0.8192,
                    1.28,
                    1.0,
                ],
            ),
            1e-12,
        );
        let out = call_outputs(&[matrix(3, &LOWER)], 2).unwrap();
        assert_eq!(
            double_matrix(&out[0], 3),
            row_major_to_col(3, &[0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0])
        );
        assert_eq!(
            double_matrix(&out[1], 3),
            row_major_to_col(3, &[6.0, 5.0, 4.0, 0.0, 3.0, 2.0, 0.0, 0.0, 1.0])
        );
    }

    /// FR-047-09: T = I(:,P)*diag(S) on a permuted case.
    #[test]
    fn documented_choice_t_from_s_and_p() {
        let a = matrix(3, &LOWER);
        let sp = call_outputs(std::slice::from_ref(&a), 3).unwrap();
        let (_, _, s) = dense(&sp[0]);
        let (_, p_shape, p) = dense(&sp[1]);
        assert_eq!(p_shape, vec![3, 1]);
        assert_eq!(p, vec![3.0, 2.0, 1.0]);
        let t = double_matrix(&call_outputs(&[a], 2).unwrap()[0], 3);
        let mut expected = vec![0.0; 9];
        for i in 0..3 {
            expected[(p[i] as usize - 1) + i * 3] = s[i];
        }
        assert_eq!(t, expected);
    }

    /// FR-047-10 (balance.q-noperm-permuting): 'noperm' on a matrix the
    /// default permutes applies scaling only (diagonal T, P = 1:n). Option
    /// text is case-insensitive and may be a string scalar.
    #[test]
    fn documented_choice_noperm_on_permutable_matrix() {
        let a = matrix(3, &LOWER);
        let out = call_outputs(&[a.clone(), Value::String("NoPerm".into())], 3).unwrap();
        assert_eq!(dense(&out[1]).2, vec![1.0, 2.0, 3.0]);
        let tb = call_outputs(
            &[a.clone(), Value::CharArray(CharArray::new_row("noperm"))],
            2,
        )
        .unwrap();
        assert_power_of_two_diagonal(&double_matrix(&tb[0], 3), 3);
        assert_similarity(&tb[0], &a, &tb[1], 1e-12);
    }

    /// FR-047-01/09: single input gives single T, B and S; P is double.
    #[test]
    fn documented_choice_single_outputs() {
        let tb = call_outputs(&[single_matrix(3, &A0)], 2).unwrap();
        assert_eq!(dense(&tb[0]).0, NumericDType::F32);
        assert_eq!(dense(&tb[1]).0, NumericDType::F32);
        let spb = call_outputs(&[single_matrix(3, &A0)], 3).unwrap();
        assert_eq!(dense(&spb[0]).0, NumericDType::F32);
        assert_eq!(dense(&spb[1]).0, NumericDType::F64);
        assert_eq!(dense(&spb[2]).0, NumericDType::F32);
        // Single B equals the double result rounded to single.
        let b64 = double_matrix(&call(&[matrix(3, &A0)]).unwrap(), 3);
        let b32 = dense(&tb[1]).2;
        let rounded: Vec<f64> = b64.iter().map(|&x| x as f32 as f64).collect();
        assert_eq!(b32, rounded);
    }

    /// FR-047-02/03 on a larger, badly scaled random matrix: similarity holds,
    /// T is a generalized permutation with power-of-two entries, eigenvalues
    /// are preserved, and the off-diagonal norm does not grow.
    #[test]
    fn documented_choice_random_badly_scaled_matrix() {
        let n = 8;
        let mut state: u64 = 7;
        let mut row_major = vec![0.0; n * n];
        for r in 0..n {
            for c in 0..n {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let u = ((state >> 11) as f64) / ((1u64 << 53) as f64);
                row_major[r * n + c] = (u - 0.5) * 10f64.powi(r as i32 - c as i32);
            }
        }
        let a = matrix(n, &row_major);
        let out = call_outputs(std::slice::from_ref(&a), 2).unwrap();
        let t = double_matrix(&out[0], n);
        assert_similarity(&out[0], &a, &out[1], 1e-12);
        for c in 0..n {
            let nonzero: Vec<f64> = (0..n).map(|r| t[r + c * n]).filter(|&v| v != 0.0).collect();
            assert_eq!(nonzero.len(), 1, "column {c} of T");
            assert_eq!(nonzero[0].log2().fract(), 0.0);
        }
        let off_norm = |data: &[f64]| -> f64 {
            let mut s = 0.0;
            for c in 0..n {
                for r in 0..n {
                    if r != c {
                        s += data[r + c * n].abs();
                    }
                }
            }
            s
        };
        assert!(off_norm(&dense(&out[1]).2) <= off_norm(&dense(&a).2));
        assert_same_eigenvalues(&a, &out[1], 1e-6);
    }

    /// FR-047-12: empty input gives empty outputs.
    #[test]
    fn documented_choice_empty_input() {
        let a = Value::Tensor(Tensor::new(Vec::new(), vec![0, 0]).unwrap());
        assert_eq!(
            dense(&call(std::slice::from_ref(&a)).unwrap()).1,
            vec![0, 0]
        );
        let tb = call_outputs(std::slice::from_ref(&a), 2).unwrap();
        assert_eq!(dense(&tb[0]).1, vec![0, 0]);
        let spb = call_outputs(&[a], 3).unwrap();
        assert_eq!(dense(&spb[0]).1, vec![0, 1]);
        assert_eq!(dense(&spb[1]).1, vec![0, 1]);
        assert_eq!(dense(&spb[2]).1, vec![0, 0]);
    }

    /// FR-047-12: NaN and Inf entries terminate and keep the shape.
    #[test]
    fn documented_choice_non_finite_terminates() {
        for bad in [f64::NAN, f64::INFINITY] {
            let a = matrix(3, &[1.0, bad, 1e6, 1e-6, 1.0, 2.0, 3.0, 1e-8, 1.0]);
            let b = call(&[a]).expect("balance");
            assert_eq!(dense(&b).1, vec![3, 3]);
        }
    }

    /// FR-047-12: non-float classes and N-D arrays raise InvalidInput.
    #[test]
    fn documented_choice_invalid_inputs() {
        for bad in [
            Value::LogicalArray(LogicalArray::new(vec![1, 0, 0, 1], vec![2, 2]).unwrap()),
            Value::Bool(true),
            Value::Int(IntValue::I32(3)),
            Value::CharArray(CharArray::new_row("ab")),
            Value::String("ab".into()),
            Value::Cell(CellArray::new(vec![Value::Num(1.0)], 1, 1).unwrap()),
            Value::Tensor(Tensor::new(vec![1.0; 8], vec![2, 2, 2]).unwrap()),
        ] {
            assert_eq!(
                identifier(call(std::slice::from_ref(&bad))),
                "RunMat:balance:InvalidInput",
                "{bad:?}"
            );
        }
    }

    /// FR-047-13: more than three outputs raise TooManyOutputs.
    #[test]
    fn documented_choice_too_many_outputs() {
        let result = call_direct(&[matrix(3, &A0)], 4);
        assert_eq!(identifier(result), "RunMat:balance:TooManyOutputs");
    }

    // --- Unresolved -----------------------------------------------------------

    /// FR-047-11 (balance.q-complex): complex input is unsupported.
    #[test]
    fn unresolved_complex_unsupported() {
        let a = Value::ComplexTensor(
            ComplexTensor::new(
                vec![(1.0, 1.0), (0.0, 0.0), (2.0, 0.0), (1.0, 0.0)],
                vec![2, 2],
            )
            .unwrap(),
        );
        assert_eq!(identifier(call(&[a])), "RunMat:balance:UnsupportedOption");
        assert_eq!(
            identifier(call(&[Value::Complex(1.0, 2.0)])),
            "RunMat:balance:UnsupportedOption"
        );
    }

    /// FR-047-10: options other than 'noperm' and extra arguments are
    /// unsupported.
    #[test]
    fn unresolved_other_options_unsupported() {
        let a = matrix(3, &A0);
        for args in [
            vec![a.clone(), Value::CharArray(CharArray::new_row("perm"))],
            vec![a.clone(), Value::Num(1.0)],
            vec![
                a.clone(),
                Value::CharArray(CharArray::new_row("noperm")),
                Value::Num(1.0),
            ],
        ] {
            assert_eq!(
                identifier(call(&args)),
                "RunMat:balance:UnsupportedOption",
                "{args:?}"
            );
        }
    }
}
