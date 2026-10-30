//! Clean-room `nthroot` (real n-th root, elementwise).
//!
//! Clean-room feature `specs/050-numeric-conversion/`. Normative behaviour
//! comes only from the approved export `nthroot` 0.2.0 (see that feature's
//! `provenance.md`). Observed `MATLAB:*` identifiers are raised as
//! `RunMat:*` (decision D-050-1). Unobserved forms raise
//! `RunMat:nthroot:UnsupportedOption` or follow labelled documented choices
//! (FR-050-21..26).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{NumericDType, Tensor, Value};

use crate::builtins::common::map_control_flow_with_builtin;
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const NAME: &str = "nthroot";

// ---------------------------------------------------------------------------
// Descriptor
// ---------------------------------------------------------------------------

const NTHROOT_INPUTS: [BuiltinParamDescriptor; 2] = [
    BuiltinParamDescriptor {
        name: "x",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Real double or single array.",
    },
    BuiltinParamDescriptor {
        name: "n",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Real scalar, or an array the same size as x.",
    },
];

const NTHROOT_OUTPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "y",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Real n-th root, in the class of x.",
}];

const NTHROOT_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "y = nthroot(x, n)",
    inputs: &NTHROOT_INPUTS,
    outputs: &NTHROOT_OUTPUTS,
}];

const NEG_X_NOT_ODD_INTEGER_N: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.NTHROOT.NEG_X_NOT_ODD_INTEGER_N",
    identifier: Some("RunMat:nthroot:NegXNotOddIntegerN"),
    when: "An element of x is negative and the matching n is not an odd integer.",
    message: "nthroot: negative x requires an odd integer n",
};

const COMPLEX_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.NTHROOT.COMPLEX_INPUT",
    identifier: Some("RunMat:nthroot:ComplexInput"),
    when: "x is complex.",
    message: "nthroot: x must be real",
};

const SIZE_MISMATCH: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.NTHROOT.SIZE_MISMATCH",
    identifier: Some("RunMat:sizeDimensionsMustMatch"),
    when: "x and n have incompatible sizes.",
    message: "nthroot: x and n must have the same size, or one of them must be scalar",
};

const INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.NTHROOT.INVALID_INPUT",
    identifier: Some("RunMat:nthroot:InvalidInput"),
    when: "The call does not have exactly two inputs.",
    message: "nthroot: expected nthroot(x, n)",
};

const UNSUPPORTED_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.NTHROOT.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:nthroot:UnsupportedOption"),
    when: "Unobserved forms: x not double/single, n not real double/single, implicit expansion.",
    message: "nthroot: this form is not yet specified",
};

const TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.NTHROOT.TOO_MANY_OUTPUTS",
    identifier: Some("RunMat:nthroot:TooManyOutputs"),
    when: "More than one output was requested.",
    message: "nthroot: too many output arguments",
};

const INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.NTHROOT.INTERNAL",
    identifier: Some("RunMat:nthroot:Internal"),
    when: "The runtime cannot assemble the result.",
    message: "nthroot: internal runtime failure",
};

const NTHROOT_ERRORS: [BuiltinErrorDescriptor; 7] = [
    NEG_X_NOT_ODD_INTEGER_N,
    COMPLEX_INPUT,
    SIZE_MISMATCH,
    INVALID_INPUT,
    UNSUPPORTED_OPTION,
    TOO_MANY_OUTPUTS,
    INTERNAL,
];

pub const NTHROOT_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &NTHROOT_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &NTHROOT_ERRORS,
};

fn nthroot_error(
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
    name = "nthroot",
    category = "math/elementwise",
    summary = "Real n-th root of real input, elementwise.",
    keywords = "nthroot,root,cube root,real root,elementwise",
    accel = "cpu",
    descriptor(crate::builtins::math::elementwise::roots::nthroot::NTHROOT_DESCRIPTOR),
    builtin_path = "crate::builtins::math::elementwise::roots::nthroot"
)]
pub(crate) async fn nthroot_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    // FR-050-21
    if let Some(count) = crate::output_count::current_output_count() {
        if count > 1 {
            return Err(nthroot_error(
                &TOO_MANY_OUTPUTS,
                format!("nthroot returns one output, {count} were requested"),
            ));
        }
    }
    if args.len() != 2 {
        return Err(nthroot_error(&INVALID_INPUT, "expected nthroot(x, n)"));
    }
    // FR-050-26: gpuArray inputs are gathered; the result is a host array.
    let mut host = Vec::with_capacity(2);
    for value in &args {
        host.push(
            gather_if_needed_async(value)
                .await
                .map_err(|err| map_control_flow_with_builtin(err, NAME))?,
        );
    }
    let x = x_operand(&host[0])?;
    let n = n_operand(&host[1])?;
    let (shape, pairs) = pair_up(&x, &n)?;
    let mut out = Vec::with_capacity(pairs.len());
    for (xv, nv) in pairs {
        out.push(real_root(xv, nv)?);
    }
    build_result(out, shape, x.single)
}

// ---------------------------------------------------------------------------
// Operands
// ---------------------------------------------------------------------------

struct RealArray {
    data: Vec<f64>,
    shape: Vec<usize>,
    single: bool,
}

/// Read a real double/single scalar or dense tensor; `None` for any other
/// class (complex is reported separately by the callers).
fn real_array(value: &Value) -> Option<RealArray> {
    match value {
        Value::Num(v) => Some(RealArray {
            data: vec![*v],
            shape: vec![1, 1],
            single: false,
        }),
        Value::Tensor(t) => match t.numeric_dtype() {
            NumericDType::F64 | NumericDType::F32 => Some(RealArray {
                data: t.materialize_f64(),
                shape: t.shape.clone(),
                single: t.numeric_dtype() == NumericDType::F32,
            }),
            _ => None,
        },
        _ => None,
    }
}

fn x_operand(value: &Value) -> BuiltinResult<RealArray> {
    if matches!(value, Value::Complex(_, _) | Value::ComplexTensor(_)) {
        // FR-050-09
        return Err(nthroot_error(&COMPLEX_INPUT, "x must not be complex"));
    }
    // FR-050-23
    real_array(value).ok_or_else(|| {
        nthroot_error(
            &UNSUPPORTED_OPTION,
            "x must be a real double or single array",
        )
    })
}

fn n_operand(value: &Value) -> BuiltinResult<RealArray> {
    // FR-050-23 (complex n included)
    real_array(value).ok_or_else(|| {
        nthroot_error(
            &UNSUPPORTED_OPTION,
            "n must be a real double or single array",
        )
    })
}

/// Pad a shape with trailing ones to `len` dimensions.
fn padded(shape: &[usize], len: usize) -> Vec<usize> {
    let mut out = shape.to_vec();
    out.resize(len, 1);
    out
}

/// Result shape and the (x, n) element pairs.
type Paired = (Vec<usize>, Vec<(f64, f64)>);

/// Pair elements of x and n (FR-050-04, FR-050-10, FR-050-22).
fn pair_up(x: &RealArray, n: &RealArray) -> BuiltinResult<Paired> {
    let dims = x.shape.len().max(n.shape.len()).max(2);
    let xs = padded(&x.shape, dims);
    let ns = padded(&n.shape, dims);
    if xs == ns {
        let pairs = x.data.iter().copied().zip(n.data.iter().copied()).collect();
        return Ok((x.shape.clone(), pairs));
    }
    if n.data.len() == 1 {
        let nv = n.data[0];
        return Ok((x.shape.clone(), x.data.iter().map(|&v| (v, nv)).collect()));
    }
    if x.data.len() == 1 {
        let xv = x.data[0];
        return Ok((n.shape.clone(), n.data.iter().map(|&v| (xv, v)).collect()));
    }
    let expandable = xs
        .iter()
        .zip(&ns)
        .all(|(&a, &b)| a == b || a == 1 || b == 1);
    if expandable {
        Err(nthroot_error(
            &UNSUPPORTED_OPTION,
            "implicit expansion of x and n is not yet specified",
        ))
    } else {
        Err(nthroot_error(
            &SIZE_MISMATCH,
            format!("sizes {:?} and {:?} are incompatible", x.shape, n.shape),
        ))
    }
}

// ---------------------------------------------------------------------------
// Arithmetic
// ---------------------------------------------------------------------------

fn is_odd_integer(n: f64) -> bool {
    n.is_finite() && n.fract() == 0.0 && (n % 2.0).abs() == 1.0
}

/// FR-050-03, 05, 06, 08: real root of one element.
fn real_root(x: f64, n: f64) -> BuiltinResult<f64> {
    if x < 0.0 {
        if !is_odd_integer(n) {
            return Err(nthroot_error(
                &NEG_X_NOT_ODD_INTEGER_N,
                format!("x = {x} is negative and n = {n} is not an odd integer"),
            ));
        }
        return Ok(-positive_root(-x, n));
    }
    Ok(positive_root(x, n))
}

/// Root of a non-negative (or NaN) value: `x^(1/n)` refined by one Newton
/// step for `r^n = x` (FR-050-07, FR-050-24).
fn positive_root(x: f64, n: f64) -> f64 {
    if n == 1.0 {
        return x;
    }
    let r = x.powf(1.0 / n);
    if !r.is_finite() || r == 0.0 || !n.is_finite() || n == 0.0 {
        return r;
    }
    let step = (r.powf(n) - x) / (n * r.powf(n - 1.0));
    if step.is_finite() {
        r - step
    } else {
        r
    }
}

fn build_result(data: Vec<f64>, shape: Vec<usize>, single: bool) -> BuiltinResult<Value> {
    if single {
        let data32: Vec<f32> = data.into_iter().map(|v| v as f32).collect();
        return Tensor::from_f32(data32, shape)
            .map(Value::Tensor)
            .map_err(|err| nthroot_error(&INTERNAL, err));
    }
    if data.len() == 1 && shape.iter().all(|&d| d == 1) {
        return Ok(Value::Num(data[0]));
    }
    Tensor::new(data, shape)
        .map(Value::Tensor)
        .map_err(|err| nthroot_error(&INTERNAL, err))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
// Observed values are copied verbatim from the imported observation record.
#[allow(clippy::excessive_precision, clippy::approx_constant)]
mod nthroot_tests {
    use super::nthroot_builtin;
    use crate::BuiltinResult;
    use futures::executor::block_on;
    use runmat_value::{CharArray, IntValue, NumericDType, Tensor, Value};

    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("nthroot", args))
    }

    fn n(x: f64) -> Value {
        Value::Num(x)
    }

    fn row(values: &[f64]) -> Value {
        Value::Tensor(Tensor::new(values.to_vec(), vec![1, values.len()]).unwrap())
    }

    fn col(values: &[f64]) -> Value {
        Value::Tensor(Tensor::new(values.to_vec(), vec![values.len(), 1]).unwrap())
    }

    fn single(x: f32) -> Value {
        Value::Tensor(Tensor::from_f32(vec![x], vec![1, 1]).unwrap())
    }

    /// (dtype, shape, values) of a real numeric result.
    fn real(value: &Value) -> (NumericDType, Vec<usize>, Vec<f64>) {
        match value {
            Value::Num(v) => (NumericDType::F64, vec![1, 1], vec![*v]),
            Value::Tensor(t) => (t.numeric_dtype(), t.shape.clone(), t.materialize_f64()),
            other => panic!("expected a real numeric result, got {other:?}"),
        }
    }

    fn assert_double(value: Value, shape: &[usize], expected: &[f64], tol: f64) {
        let (dtype, got_shape, got) = real(&value);
        assert_eq!(dtype, NumericDType::F64);
        assert_eq!(got_shape, shape);
        assert_eq!(got.len(), expected.len());
        for (g, e) in got.iter().zip(expected) {
            if e.is_nan() {
                assert!(g.is_nan(), "expected NaN, got {g}");
            } else if e.is_infinite() {
                assert_eq!(g, e);
            } else {
                assert!((g - e).abs() <= tol, "expected {e}, got {g}");
            }
        }
    }

    fn identifier(result: BuiltinResult<Value>) -> String {
        match result {
            Ok(value) => panic!("expected an error, got {value:?}"),
            Err(err) => err.identifier().unwrap_or("<none>").to_string(),
        }
    }

    const NEG: &str = "RunMat:nthroot:NegXNotOddIntegerN";
    const UNSUPPORTED: &str = "RunMat:nthroot:UnsupportedOption";

    // --- Observed (normative): one test per observed case --------------------

    /// case `cube-neg` — nthroot.signature-primary, output-class, negative-odd
    #[test]
    fn observed_cube_neg() {
        assert_double(call(&[n(-8.0), n(3.0)]).unwrap(), &[1, 1], &[-2.0], 0.0);
    }

    /// case `fourth` — nthroot.even-root
    #[test]
    fn observed_fourth() {
        assert_double(call(&[n(16.0), n(4.0)]).unwrap(), &[1, 1], &[2.0], 1e-12);
    }

    /// case `vector` — nthroot.negative-odd
    #[test]
    fn observed_vector() {
        let y = call(&[row(&[1.0, 8.0, 27.0, -64.0]), n(3.0)]).unwrap();
        assert_double(y, &[1, 4], &[1.0, 2.0, 3.0, -4.0], 1e-12);
    }

    /// case `vector-n` — nthroot.elementwise-n
    #[test]
    fn observed_vector_n() {
        let y = call(&[row(&[4.0, 27.0, 16.0]), row(&[2.0, 3.0, 4.0])]).unwrap();
        assert_double(y, &[1, 3], &[2.0, 3.0, 2.0], 1e-12);
    }

    /// case `specials` — nthroot.specials
    #[test]
    fn observed_specials() {
        let y = call(&[
            row(&[0.0, f64::INFINITY, f64::NEG_INFINITY, f64::NAN]),
            n(3.0),
        ])
        .unwrap();
        assert_double(
            y,
            &[1, 4],
            &[0.0, f64::INFINITY, f64::NEG_INFINITY, f64::NAN],
            0.0,
        );
    }

    /// case `n-one` — nthroot.n-values
    #[test]
    fn observed_n_one() {
        assert_double(call(&[n(-5.0), n(1.0)]).unwrap(), &[1, 1], &[-5.0], 0.0);
    }

    /// case `n-negative` — nthroot.n-values
    #[test]
    fn observed_n_negative() {
        assert_double(call(&[n(8.0), n(-3.0)]).unwrap(), &[1, 1], &[0.5], 1e-15);
    }

    /// case `single` — nthroot.output-class
    #[test]
    fn observed_single() {
        let (dtype, shape, values) = real(&call(&[single(27.0), n(3.0)]).unwrap());
        assert_eq!(dtype, NumericDType::F32);
        assert_eq!(shape, vec![1, 1]);
        assert_eq!(values, vec![3.0]);
    }

    /// case `exactness` — nthroot.exact
    #[test]
    fn observed_exactness() {
        let (_, _, values) = real(&call(&[n(1000.0), n(3.0)]).unwrap());
        assert_eq!(values, vec![10.0]);
    }

    /// case `err-even-negative` — nthroot.error-even-negative
    #[test]
    fn observed_err_even_negative() {
        assert_eq!(identifier(call(&[n(-16.0), n(4.0)])), NEG);
    }

    /// case `noninteger-n` — nthroot.n-values
    #[test]
    fn observed_noninteger_n() {
        let y = call(&[n(8.0), n(2.5)]).unwrap();
        assert_double(y, &[1, 1], &[2.2973967099940702], 1e-12);
    }

    /// case `err-noninteger-n-neg` — nthroot.error-even-negative
    #[test]
    fn observed_err_noninteger_n_neg() {
        assert_eq!(identifier(call(&[n(-8.0), n(2.5)])), NEG);
    }

    /// case `err-complex` — nthroot.error-complex
    #[test]
    fn observed_err_complex() {
        assert_eq!(
            identifier(call(&[Value::Complex(0.0, 1.0), n(3.0)])),
            "RunMat:nthroot:ComplexInput"
        );
    }

    /// case `err-size` — nthroot.error-size
    #[test]
    fn observed_err_size() {
        assert_eq!(
            identifier(call(&[row(&[1.0, 2.0, 3.0]), row(&[2.0, 3.0])])),
            "RunMat:sizeDimensionsMustMatch"
        );
    }

    // --- Documented choices (FR-050-21..26) ----------------------------------

    /// FR-050-21: arity.
    #[test]
    fn documented_choice_arity() {
        assert_eq!(identifier(call(&[n(8.0)])), "RunMat:nthroot:InvalidInput");
        assert_eq!(
            identifier(call(&[n(8.0), n(3.0), n(1.0)])),
            "RunMat:nthroot:InvalidInput"
        );
    }

    /// FR-050-21: more than one output (direct call under push_output_count).
    #[test]
    fn documented_choice_too_many_outputs() {
        let _guard = crate::output_count::push_output_count(Some(2));
        let err = block_on(nthroot_builtin(vec![n(8.0), n(3.0)])).unwrap_err();
        assert_eq!(err.identifier(), Some("RunMat:nthroot:TooManyOutputs"));
    }

    /// FR-050-22: scalar x with array n takes the shape of n.
    #[test]
    fn documented_choice_scalar_x_array_n() {
        let y = call(&[n(64.0), row(&[2.0, 3.0, 6.0])]).unwrap();
        assert_double(y, &[1, 3], &[8.0, 4.0, 2.0], 1e-12);
    }

    /// FR-050-22: matrix x with scalar n keeps the shape of x; equal-shape
    /// matrices pair elementwise.
    #[test]
    fn documented_choice_matrix_shapes() {
        let x = Value::Tensor(Tensor::new(vec![1.0, 8.0, 27.0, -8.0], vec![2, 2]).unwrap());
        let y = call(&[x, n(3.0)]).unwrap();
        assert_double(y, &[2, 2], &[1.0, 2.0, 3.0, -2.0], 1e-12);
    }

    /// FR-050-22: row against column (implicit expansion) is unsupported;
    /// other mismatches are sizeDimensionsMustMatch.
    #[test]
    fn documented_choice_expansion_and_mismatch() {
        assert_eq!(
            identifier(call(&[row(&[1.0, 2.0]), col(&[2.0, 3.0])])),
            UNSUPPORTED
        );
        assert_eq!(
            identifier(call(&[col(&[1.0, 2.0, 3.0]), col(&[2.0, 3.0])])),
            "RunMat:sizeDimensionsMustMatch"
        );
    }

    /// FR-050-23: classes of x and n.
    #[test]
    fn documented_choice_unsupported_classes() {
        assert_eq!(
            identifier(call(&[Value::Int(IntValue::I8(8)), n(3.0)])),
            UNSUPPORTED
        );
        assert_eq!(identifier(call(&[Value::Bool(true), n(3.0)])), UNSUPPORTED);
        assert_eq!(
            identifier(call(&[Value::CharArray(CharArray::new_row("a")), n(3.0)])),
            UNSUPPORTED
        );
        assert_eq!(
            identifier(call(&[n(8.0), Value::Int(IntValue::I8(3))])),
            UNSUPPORTED
        );
        assert_eq!(
            identifier(call(&[n(8.0), Value::Complex(3.0, 1.0)])),
            UNSUPPORTED
        );
    }

    /// FR-050-23: single n with double x gives double (class of x).
    #[test]
    fn documented_choice_single_n_double_x() {
        let (dtype, _, values) = real(&call(&[n(27.0), single(3.0)]).unwrap());
        assert_eq!(dtype, NumericDType::F64);
        assert_eq!(values, vec![3.0]);
    }

    /// FR-050-24: negative odd n with negative x, special n, empty x.
    #[test]
    fn documented_choice_edge_cases() {
        assert_double(call(&[n(-8.0), n(-3.0)]).unwrap(), &[1, 1], &[-0.5], 1e-15);
        assert_eq!(identifier(call(&[n(-8.0), n(f64::INFINITY)])), NEG);
        assert_eq!(identifier(call(&[n(-8.0), n(f64::NAN)])), NEG);
        assert_eq!(identifier(call(&[n(-8.0), n(0.0)])), NEG);
        let empty = Value::Tensor(Tensor::new(vec![], vec![0, 3]).unwrap());
        assert_double(call(&[empty, n(3.0)]).unwrap(), &[0, 3], &[], 0.0);
    }

    /// FR-050-03/07: Newton refinement makes perfect powers exact.
    #[test]
    fn documented_choice_perfect_powers_exact() {
        for (x, k, r) in [(125.0, 3.0, 5.0), (-343.0, 3.0, -7.0), (32.0, 5.0, 2.0)] {
            let (_, _, values) = real(&call(&[n(x), n(k)]).unwrap());
            assert_eq!(values, vec![r], "nthroot({x},{k})");
        }
    }

    /// FR-050-30 (unresolved, nthroot.q-warning-coverage): no warning is
    /// emitted; the call simply succeeds.
    #[test]
    fn unresolved_no_warning() {
        assert!(call(&[n(27.0), n(3.0)]).is_ok());
    }
}
