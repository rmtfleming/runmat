//! Clean-room `sprandn` (sparse normally distributed random matrix).
//!
//! Clean-room feature `specs/061-sparse-linalg-n5/`. Normative behaviour
//! comes only from the approved export `sprandn` 0.2.0 (see that feature's
//! `provenance.md`): a real sparse double `m`-by-`n` result with finite
//! nonzeros of both signs, repeatable for the same `rng` seed; at most
//! `density*m*n` nonzeros; zero density and zero-row sizes give empty
//! sparse results; `sprandn(S)` keeps exactly the pattern of `S`;
//! `sprandn(m, n, density, rc)` has 2-norm condition number `1/rc`; and the
//! error identifiers (raised as `RunMat:*`, D-061-1). The values themselves
//! are not specified (`sprandn.q-values`): RunMat's values come from its own
//! generator and are never compared with oracle values.
//!
//! Construction (D-061-5): the nonzero count is `floor(density*m*n)`, the
//! positions a uniform sample without replacement and the values standard
//! normal draws, all from RunMat's global random stream (the one `rng`
//! seeds). The `rc` form reuses the `sprand` construction in the parent
//! module: a diagonal with geometrically spaced singular values `1 .. rc`
//! spread by random plane rotations, which preserve the singular values.

use std::collections::BTreeMap;

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{NumericDType, SparseTensor, Value};

use crate::builtins::common::{gpu_helpers, random};
use crate::{build_runtime_error, BuiltinResult, RuntimeError};

const NAME: &str = "sprandn";

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
    "Matrix whose nonzero pattern is kept.",
);
const IN_M: BuiltinParamDescriptor = param(
    "m",
    BuiltinParamArity::Required,
    "Number of rows (nonnegative integer).",
);
const IN_N: BuiltinParamDescriptor = param(
    "n",
    BuiltinParamArity::Required,
    "Number of columns (nonnegative integer).",
);
const IN_DENSITY: BuiltinParamDescriptor = param(
    "density",
    BuiltinParamArity::Required,
    "Target fraction of nonzeros (clamped to [0, 1]).",
);
const IN_RC: BuiltinParamDescriptor = param(
    "rc",
    BuiltinParamArity::Required,
    "Reciprocal 2-norm condition number (scalar in [0, 1]).",
);
const OUT_R: BuiltinParamDescriptor = param(
    "R",
    BuiltinParamArity::Required,
    "Sparse real double matrix.",
);

const INPUTS_S: [BuiltinParamDescriptor; 1] = [IN_S];
const INPUTS_MND: [BuiltinParamDescriptor; 3] = [IN_M, IN_N, IN_DENSITY];
const INPUTS_MND_RC: [BuiltinParamDescriptor; 4] = [IN_M, IN_N, IN_DENSITY, IN_RC];
const OUTPUTS_R: [BuiltinParamDescriptor; 1] = [OUT_R];

const SPRANDN_SIGNATURES: [BuiltinSignatureDescriptor; 3] = [
    BuiltinSignatureDescriptor {
        label: "R = sprandn(m, n, density)",
        inputs: &INPUTS_MND,
        outputs: &OUTPUTS_R,
    },
    BuiltinSignatureDescriptor {
        label: "R = sprandn(S)",
        inputs: &INPUTS_S,
        outputs: &OUTPUTS_R,
    },
    BuiltinSignatureDescriptor {
        label: "R = sprandn(m, n, density, rc)",
        inputs: &INPUTS_MND_RC,
        outputs: &OUTPUTS_R,
    },
];

const INVALID_LIMITS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SPRANDN.INVALID_LIMITS",
    identifier: Some("RunMat:RandStream:randi:invalidLimits"),
    when: "m or n is not a nonnegative integer.",
    message: "sprandn: m and n must be nonnegative integers",
};

const TWO_INPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SPRANDN.TWO_INPUTS",
    identifier: Some("RunMat:sprandn:TwoInputs"),
    when: "Only m and n are given.",
    message: "sprandn: a density is required with m and n",
};

const INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SPRANDN.INVALID_INPUT",
    identifier: Some("RunMat:sprandn:InvalidInput"),
    when: "No input is given, or S is not a 2-D numeric or logical matrix.",
    message: "sprandn: expected sprandn(S), sprandn(m,n,density) or sprandn(m,n,density,rc)",
};

const UNSUPPORTED_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SPRANDN.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:sprandn:UnsupportedOption"),
    when: "Unobserved inputs: more than four arguments, typename, non-double sizes, NaN or non-scalar density, vector or out-of-range rc, sizes too large for bounded storage.",
    message: "sprandn: this form is not yet specified",
};

const TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SPRANDN.TOO_MANY_OUTPUTS",
    identifier: Some("RunMat:sprandn:TooManyOutputs"),
    when: "More than one output was requested.",
    message: "sprandn: too many output arguments",
};

const INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SPRANDN.INTERNAL",
    identifier: Some("RunMat:sprandn:Internal"),
    when: "The runtime cannot assemble the result.",
    message: "sprandn: internal runtime failure",
};

const SPRANDN_ERRORS: [BuiltinErrorDescriptor; 6] = [
    INVALID_LIMITS,
    TWO_INPUTS,
    INVALID_INPUT,
    UNSUPPORTED_OPTION,
    TOO_MANY_OUTPUTS,
    INTERNAL,
];

pub const SPRANDN_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &SPRANDN_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &SPRANDN_ERRORS,
};

fn sprandn_error(
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
    sprandn_error(&UNSUPPORTED_OPTION, detail)
}

/// Re-label an error from the shared `sprand` helpers as a `sprandn` error.
fn relabel(err: RuntimeError) -> RuntimeError {
    unsupported(err.message().replace("sprand:", "").trim().to_string())
}

#[runtime_builtin(
    name = "sprandn",
    category = "array/creation",
    summary = "Create a sparse normally distributed random matrix.",
    keywords = "sprandn,sparse,random,normal,density",
    accel = "cpu",
    descriptor(crate::builtins::array::creation::sparse::sprandn::SPRANDN_DESCRIPTOR),
    builtin_path = "crate::builtins::array::creation::sparse::sprandn"
)]
pub(crate) async fn sprandn_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    // FR-061-38
    if crate::output_count::current_output_count().is_some_and(|count| count > 1) {
        return Err(sprandn_error(
            &TOO_MANY_OUTPUTS,
            "only one output is supported",
        ));
    }
    // FR-061-39: gpuArray inputs are gathered.
    let mut gathered = Vec::with_capacity(args.len());
    for arg in &args {
        gathered.push(gpu_helpers::gather_value_async(arg).await?);
    }
    let result = match gathered.len() {
        0 => return Err(sprandn_error(&INVALID_INPUT, "expected at least one input")),
        1 => pattern_form(gathered.pop().expect("one argument"))?,
        // FR-061-35 (sprandn.error-two-args)
        2 => {
            return Err(sprandn_error(
                &TWO_INPUTS,
                "sprandn(m, n) needs a density: use sprandn(m, n, density)",
            ))
        }
        3 | 4 => {
            let rows = size_arg(&gathered[0], "m")?;
            let cols = size_arg(&gathered[1], "n")?;
            let density = density_arg(&gathered[2])?;
            if gathered.len() == 4 {
                rc_form(rows, cols, density, &gathered[3])?
            } else {
                density_form(rows, cols, density)?
            }
        }
        _ => {
            return Err(unsupported(
                "more than four arguments are not yet specified",
            ))
        }
    };
    Ok(Value::SparseTensor(result))
}

/// Scalar real double value, or `None`.
fn real_double_scalar(value: &Value) -> Option<f64> {
    match value {
        Value::Num(v) => Some(*v),
        Value::Tensor(t)
            if t.integer_storage().is_none()
                && t.numeric_dtype() == NumericDType::F64
                && t.shape.iter().product::<usize>() == 1 =>
        {
            Some(t.materialize_f64()[0])
        }
        _ => None,
    }
}

/// FR-061-34 (sprandn.error-noninteger); FR-061-38 for non-double classes.
fn size_arg(value: &Value, name: &str) -> BuiltinResult<usize> {
    let raw = real_double_scalar(value)
        .ok_or_else(|| unsupported(format!("{name} must be a real double scalar")))?;
    if !raw.is_finite() || raw < 0.0 || raw.fract() != 0.0 {
        return Err(sprandn_error(
            &INVALID_LIMITS,
            format!("{name} must be a nonnegative integer, got {raw}"),
        ));
    }
    if raw > crate::builtins::math::sparse::max_usize_cast_value() {
        return Err(unsupported(format!("{name} is too large")));
    }
    Ok(raw as usize)
}

/// FR-061-36 (documented choice for the non-normative
/// `sprandn.density-out-of-range`): out-of-range densities are clamped to
/// [0, 1] without an error; NaN and non-scalar densities are unsupported.
fn density_arg(value: &Value) -> BuiltinResult<f64> {
    let density = real_double_scalar(value)
        .filter(|d| !d.is_nan())
        .ok_or_else(|| unsupported("density must be a real double scalar"))?;
    Ok(density.clamp(0.0, 1.0))
}

/// D-061-5: floor(density*m*n), never above the observed bound.
fn target_count(rows: usize, cols: usize, density: f64) -> BuiltinResult<(usize, usize)> {
    let total = rows
        .checked_mul(cols)
        .ok_or_else(|| unsupported("matrix dimensions overflow"))?;
    let target = ((total as f64) * density).floor().clamp(0.0, total as f64) as usize;
    if target > super::SPARSE_HELPER_DENSE_INPUT_LIMIT {
        return Err(unsupported(format!(
            "{target} stored entries exceed the safe threshold"
        )));
    }
    Ok((total, target))
}

/// FR-061-29..31: `sprandn(m, n, density)`.
fn density_form(rows: usize, cols: usize, density: f64) -> BuiltinResult<SparseTensor> {
    let (total, target) = target_count(rows, cols, density)?;
    if target == 0 {
        return Ok(SparseTensor::zeros(rows, cols));
    }
    let positions = super::sample_unique_positions(total, target).map_err(relabel)?;
    let values = random::generate_normal(target, NAME)?;
    let mut entries = BTreeMap::new();
    for (linear, value) in positions.into_iter().zip(values) {
        entries.insert((linear / rows, linear % rows), value);
    }
    super::sparse_from_entries(rows, cols, entries, NAME)
        .map_err(|err| sprandn_error(&INTERNAL, err.message().to_string()))
}

/// FR-061-32 (sprandn.pattern): exactly the pattern of `S`, new values.
fn pattern_form(value: Value) -> BuiltinResult<SparseTensor> {
    let pattern = match value {
        Value::SparseTensor(_)
        | Value::Tensor(_)
        | Value::LogicalArray(_)
        | Value::ComplexTensor(_)
        | Value::Num(_)
        | Value::Bool(_)
        | Value::Complex(_, _) => super::sparse_pattern_from_value(value)
            .map_err(|err| sprandn_error(&INVALID_INPUT, err.message().to_string()))?,
        _ => {
            return Err(sprandn_error(
                &INVALID_INPUT,
                "S must be a 2-D numeric or logical matrix",
            ))
        }
    };
    let mut values = random::generate_normal(pattern.nnz(), NAME)?;
    // A draw of exactly zero (probability zero) would break the pattern.
    for v in &mut values {
        if *v == 0.0 {
            *v = f64::MIN_POSITIVE;
        }
    }
    SparseTensor::new(
        pattern.rows,
        pattern.cols,
        pattern.col_ptrs,
        pattern.row_indices,
        values,
    )
    .map_err(|err| sprandn_error(&INTERNAL, err))
}

/// FR-061-33 (sprandn.rc): a scalar rc in [0, 1]; the condition-number
/// construction is the parent module's `sprand` one (D-061-5).
fn rc_form(rows: usize, cols: usize, density: f64, rc: &Value) -> BuiltinResult<SparseTensor> {
    let rc = real_double_scalar(rc)
        .filter(|v| v.is_finite() && (0.0..=1.0).contains(v))
        .ok_or_else(|| unsupported("rc must be a real double scalar in [0, 1]"))?;
    let rank_limit = rows.min(cols);
    let profile = super::scalar_rcond_singular_profile(rank_limit, rc);
    super::sprand_from_condition_profile(rows, cols, density, &profile).map_err(relabel)
}

#[cfg(test)]
mod sprandn_tests {
    use super::sprandn_builtin;
    use crate::builtins::common::random;
    use crate::BuiltinResult;
    use futures::executor::block_on;
    use nalgebra::DMatrix;
    use runmat_value::{CharArray, IntValue, NumericDType, SparseTensor, Tensor, Value};

    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("sprandn", args))
    }

    fn rng(seed: f64) {
        block_on(crate::call_builtin_async("rng", &[Value::Num(seed)])).unwrap();
    }

    fn nums(values: &[f64]) -> Vec<Value> {
        values.iter().map(|v| Value::Num(*v)).collect()
    }

    fn sparse_of(value: Value) -> SparseTensor {
        match value {
            Value::SparseTensor(s) => s,
            other => panic!("expected a sparse matrix, got {other:?}"),
        }
    }

    /// Structure: real sparse double of the given size, finite values.
    fn structure(s: &SparseTensor, rows: usize, cols: usize) -> Vec<f64> {
        assert_eq!((s.rows, s.cols), (rows, cols));
        assert!(!s.is_complex() && !s.is_logical());
        assert_eq!(s.numeric_dtype(), Some(NumericDType::F64));
        let values = s.materialize_real_f64().unwrap();
        assert_eq!(values.len(), s.nnz());
        assert!(values.iter().all(|v| v.is_finite() && *v != 0.0));
        values
    }

    fn same(a: &SparseTensor, b: &SparseTensor) -> bool {
        a.col_ptrs.to_vec() == b.col_ptrs.to_vec()
            && a.row_indices.to_vec() == b.row_indices.to_vec()
            && a.materialize_real_f64().unwrap() == b.materialize_real_f64().unwrap()
    }

    fn identifier(result: BuiltinResult<Value>) -> String {
        match result {
            Ok(value) => panic!("expected an error, got {value:?}"),
            Err(err) => err.identifier().unwrap_or("<none>").to_string(),
        }
    }

    /// Seeded form: structure, nnz bound, both signs and repeatability.
    fn seeded(rows: usize, cols: usize, density: f64) -> SparseTensor {
        let args = nums(&[rows as f64, cols as f64, density]);
        rng(7.0);
        let first = sparse_of(call(&args).unwrap());
        rng(7.0);
        let second = sparse_of(call(&args).unwrap());
        assert!(same(&first, &second), "same seed must repeat");
        let values = structure(&first, rows, cols);
        assert!(first.nnz() as f64 <= density * (rows * cols) as f64);
        assert!(values.iter().any(|v| *v < 0.0) && values.iter().any(|v| *v > 0.0));
        first
    }

    // --- Observed (normative): one test per observed case --------------------

    /// case `cobra-5x6` — sprandn.signature-primary, sprandn.structure,
    /// sprandn.nnz-bound (values are never compared, sprandn.q-values)
    #[test]
    fn observed_cobra_5x6() {
        let _guard = random::test_guard();
        seeded(5, 6, 0.6);
    }

    /// case `cobra-30x30` — sprandn.structure, sprandn.nnz-bound
    #[test]
    fn observed_cobra_30x30() {
        let _guard = random::test_guard();
        seeded(30, 30, 0.3);
    }

    /// case `density-zero` — sprandn.zero-density
    #[test]
    fn observed_density_zero() {
        let _guard = random::test_guard();
        let s = sparse_of(call(&nums(&[4.0, 5.0, 0.0])).unwrap());
        structure(&s, 4, 5);
        assert_eq!(s.nnz(), 0);
    }

    /// case `density-one` — sprandn.structure, sprandn.nnz-bound
    #[test]
    fn observed_density_one() {
        let _guard = random::test_guard();
        seeded(4, 5, 1.0);
    }

    /// case `empty-dims` — sprandn.zero-density
    #[test]
    fn observed_empty_dims() {
        let _guard = random::test_guard();
        let s = sparse_of(call(&nums(&[0.0, 3.0, 0.5])).unwrap());
        structure(&s, 0, 3);
        assert_eq!(s.nnz(), 0);
    }

    /// case `pattern` — sprandn.pattern
    #[test]
    fn observed_pattern() {
        let _guard = random::test_guard();
        // sparse([1 3], [2 1], [5 6], 3, 3): (1,2) = 5 and (3,1) = 6.
        let s = SparseTensor::new(3, 3, vec![0, 1, 2, 2], vec![2, 0], vec![6.0, 5.0]).unwrap();
        let r = sparse_of(call(&[Value::SparseTensor(s.clone())]).unwrap());
        structure(&r, 3, 3);
        assert_eq!(r.col_ptrs.to_vec(), s.col_ptrs.to_vec());
        assert_eq!(r.row_indices.to_vec(), s.row_indices.to_vec());
    }

    /// case `rc` — sprandn.rc: cond(full(R)) = 1/rc = 10.
    #[test]
    fn observed_rc() {
        let _guard = random::test_guard();
        rng(7.0);
        let r = sparse_of(call(&nums(&[4.0, 4.0, 0.5, 0.1])).unwrap());
        structure(&r, 4, 4);
        let dense = r.to_dense().unwrap().materialize_f64();
        let sigma = DMatrix::from_column_slice(4, 4, &dense).singular_values();
        let cond = sigma.max() / sigma.min();
        assert!((cond - 10.0).abs() < 1e-10, "cond = {cond}");
    }

    /// case `density-above-one` — sprandn.density-out-of-range (non-normative:
    /// no error; the count is a documented choice, FR-061-36)
    #[test]
    fn observed_density_above_one() {
        let _guard = random::test_guard();
        let s = sparse_of(call(&nums(&[3.0, 3.0, 1.5])).unwrap());
        structure(&s, 3, 3);
        assert!(s.nnz() <= 9);
    }

    /// case `density-negative` — sprandn.density-out-of-range (non-normative)
    #[test]
    fn observed_density_negative() {
        let _guard = random::test_guard();
        let s = sparse_of(call(&nums(&[3.0, 3.0, -0.1])).unwrap());
        structure(&s, 3, 3);
        assert_eq!(s.nnz(), 0);
    }

    /// case `err-noninteger` — sprandn.error-noninteger
    #[test]
    fn observed_err_noninteger() {
        assert_eq!(
            identifier(call(&nums(&[2.5, 3.0, 0.5]))),
            "RunMat:RandStream:randi:invalidLimits"
        );
    }

    /// case `err-two-args` — sprandn.error-two-args
    #[test]
    fn observed_err_two_args() {
        assert_eq!(
            identifier(call(&nums(&[3.0, 3.0]))),
            "RunMat:sprandn:TwoInputs"
        );
    }

    // --- Documented choices and unresolved points ----------------------------

    /// D-061-5: the count is floor(density*m*n); the values follow the
    /// global stream (different seeds give different matrices).
    #[test]
    fn documented_choice_count_and_stream() {
        let _guard = random::test_guard();
        rng(1.0);
        let a = sparse_of(call(&nums(&[3.0, 3.0, 0.5])).unwrap());
        assert_eq!(a.nnz(), 4);
        let b = sparse_of(call(&nums(&[3.0, 3.0, 0.5])).unwrap());
        assert!(!same(&a, &b), "consecutive calls draw new values");
        let full = sparse_of(call(&nums(&[4.0, 5.0, 1.0])).unwrap());
        assert_eq!(full.nnz(), 20);
        // sprandn(S) for a full S keeps its pattern.
        let dense = Tensor::new(vec![0.0, 2.0, 3.0, 0.0], vec![2, 2]).unwrap();
        let r = sparse_of(call(&[Value::Tensor(dense)]).unwrap());
        assert_eq!(r.row_indices.to_vec(), vec![1, 0]);
    }

    /// FR-061-34, FR-061-38: other invalid or unobserved inputs.
    #[test]
    fn documented_choice_invalid_and_unsupported() {
        const U: &str = "RunMat:sprandn:UnsupportedOption";
        const L: &str = "RunMat:RandStream:randi:invalidLimits";
        assert_eq!(identifier(call(&nums(&[3.0, -1.0, 0.5]))), L);
        assert_eq!(identifier(call(&nums(&[f64::NAN, 3.0, 0.5]))), L);
        assert_eq!(identifier(call(&nums(&[3.0, 3.0, f64::NAN]))), U);
        assert_eq!(identifier(call(&nums(&[3.0, 3.0, 0.5, 2.0]))), U);
        assert_eq!(identifier(call(&nums(&[3.0, 3.0, 0.5, 0.1, 1.0]))), U);
        let typename = Value::CharArray(CharArray::new_row("single"));
        assert_eq!(
            identifier(call(&[
                Value::Num(3.0),
                Value::Num(3.0),
                Value::Num(0.5),
                typename
            ])),
            U
        );
        assert_eq!(
            identifier(call(&[
                Value::Int(IntValue::I32(3)),
                Value::Num(3.0),
                Value::Num(0.5)
            ])),
            U
        );
        assert_eq!(identifier(call(&[])), "RunMat:sprandn:InvalidInput");
        assert_eq!(
            identifier(call(&[Value::CharArray(CharArray::new_row("ab"))])),
            "RunMat:sprandn:InvalidInput"
        );
        let _guard = crate::output_count::push_output_count(Some(2));
        let err = block_on(sprandn_builtin(nums(&[2.0, 2.0, 0.5]))).unwrap_err();
        assert_eq!(err.identifier(), Some("RunMat:sprandn:TooManyOutputs"));
    }

    /// `sprandn.q-warning-coverage` (unresolved): no warning is emitted.
    #[test]
    fn unresolved_no_warning() {
        let _guard = random::test_guard();
        assert!(call(&nums(&[2.0, 2.0, 0.5])).is_ok());
    }
}
