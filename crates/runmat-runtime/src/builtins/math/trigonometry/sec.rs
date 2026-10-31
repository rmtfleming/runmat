//! Clean-room `sec` (elementwise secant of floating-point input).
//!
//! Clean-room feature `specs/055-numeric-n3n4/`. Normative behaviour comes
//! only from the approved export `sec` 0.2.0 (see that feature's
//! `provenance.md`). Observed `MATLAB:*` identifiers are raised as
//! `RunMat:*` (D-055-1). Unobserved classes raise
//! `RunMat:sec:UnsupportedOption` (FR-055-30).
//!
//! Algorithm: the textbook identity `sec(z) = 1 / cos(z)`, evaluated in
//! double precision (real `f64::cos`, complex `num_complex` `cos`).

use num_complex::Complex64;
use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{ComplexTensor, NumericDType, Tensor, Value};

use crate::builtins::common::map_control_flow_with_builtin;
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const NAME: &str = "sec";

// ---------------------------------------------------------------------------
// Descriptor
// ---------------------------------------------------------------------------

const SEC_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "X",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Double or single array, real or complex.",
}];

const SEC_OUTPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "Y",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Secant of each element, same size and class as X.",
}];

const SEC_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "Y = sec(X)",
    inputs: &SEC_INPUTS,
    outputs: &SEC_OUTPUTS,
}];

const MUST_BE_FLOAT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SEC.MUST_BE_FLOAT",
    identifier: Some("RunMat:math:mustBeFloat"),
    when: "X is logical, of an integer class, or char.",
    message: "sec: X must be double or single",
};

const MINRHS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SEC.MINRHS",
    identifier: Some("RunMat:minrhs"),
    when: "No input is given.",
    message: "sec: not enough input arguments",
};

const INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SEC.INVALID_INPUT",
    identifier: Some("RunMat:sec:InvalidInput"),
    when: "More than one input is given.",
    message: "sec: expected sec(X)",
};

const UNSUPPORTED_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SEC.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:sec:UnsupportedOption"),
    when: "Unobserved classes of X: sparse, string, cell, struct and others.",
    message: "sec: this form is not yet specified",
};

const TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SEC.TOO_MANY_OUTPUTS",
    identifier: Some("RunMat:sec:TooManyOutputs"),
    when: "More than one output was requested.",
    message: "sec: too many output arguments",
};

const INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SEC.INTERNAL",
    identifier: Some("RunMat:sec:Internal"),
    when: "The runtime cannot assemble the result.",
    message: "sec: internal runtime failure",
};

const SEC_ERRORS: [BuiltinErrorDescriptor; 6] = [
    MUST_BE_FLOAT,
    MINRHS,
    INVALID_INPUT,
    UNSUPPORTED_OPTION,
    TOO_MANY_OUTPUTS,
    INTERNAL,
];

pub const SEC_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &SEC_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &SEC_ERRORS,
};

fn sec_error(
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
    name = "sec",
    category = "math/trigonometry",
    summary = "Secant of each element of a double or single array.",
    keywords = "sec,secant,trigonometry,elementwise",
    accel = "cpu",
    descriptor(crate::builtins::math::trigonometry::sec::SEC_DESCRIPTOR),
    builtin_path = "crate::builtins::math::trigonometry::sec"
)]
pub(crate) async fn sec_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    // FR-055-29
    if let Some(count) = crate::output_count::current_output_count() {
        if count > 1 {
            return Err(sec_error(
                &TOO_MANY_OUTPUTS,
                format!("sec returns one output, {count} were requested"),
            ));
        }
    }
    // FR-055-08
    if args.is_empty() {
        return Err(sec_error(&MINRHS, "not enough input arguments"));
    }
    if args.len() > 1 {
        return Err(sec_error(&INVALID_INPUT, "expected sec(X)"));
    }
    // FR-055-31: gpuArray input is gathered; the result is a host array.
    let x = gather_if_needed_async(&args[0])
        .await
        .map_err(|err| map_control_flow_with_builtin(err, NAME))?;
    evaluate(x)
}

fn real(x: f64) -> f64 {
    1.0 / x.cos()
}

fn complex(re: f64, im: f64) -> (f64, f64) {
    let y = Complex64::new(1.0, 0.0) / Complex64::new(re, im).cos();
    (y.re, y.im)
}

fn not_float() -> RuntimeError {
    sec_error(
        &MUST_BE_FLOAT,
        "X must be double or single, not logical, integer or char",
    )
}

fn evaluate(x: Value) -> BuiltinResult<Value> {
    match x {
        // FR-055-01..04
        Value::Num(v) => Ok(Value::Num(real(v))),
        Value::Tensor(t) => {
            if t.integer_storage().is_some() {
                return Err(not_float());
            }
            match t.numeric_dtype() {
                NumericDType::F64 => {
                    let data = t.materialize_f64().into_iter().map(real).collect();
                    Tensor::new(data, t.shape.clone())
                        .map(Value::Tensor)
                        .map_err(|err| sec_error(&INTERNAL, err))
                }
                // FR-055-06: single in, single out (computed in double).
                NumericDType::F32 => {
                    let data = t
                        .materialize_f64()
                        .into_iter()
                        .map(|v| real(v) as f32)
                        .collect();
                    Tensor::from_f32(data, t.shape.clone())
                        .map(Value::Tensor)
                        .map_err(|err| sec_error(&INTERNAL, err))
                }
                _ => Err(not_float()),
            }
        }
        // FR-055-05
        Value::Complex(re, im) => {
            let (re, im) = complex(re, im);
            Ok(Value::Complex(re, im))
        }
        Value::ComplexTensor(t) => match t.numeric_dtype() {
            NumericDType::F64 if t.integer_storage().is_none() => {
                let data = t
                    .materialize_f64()
                    .into_iter()
                    .map(|(re, im)| complex(re, im))
                    .collect();
                ComplexTensor::new(data, t.shape.clone())
                    .map(Value::ComplexTensor)
                    .map_err(|err| sec_error(&INTERNAL, err))
            }
            NumericDType::F32 if t.integer_storage().is_none() => {
                let data = t
                    .materialize_f64()
                    .into_iter()
                    .map(|(re, im)| {
                        let (re, im) = complex(re, im);
                        (re as f32, im as f32)
                    })
                    .collect();
                ComplexTensor::from_f32(data, t.shape.clone())
                    .map(Value::ComplexTensor)
                    .map_err(|err| sec_error(&INTERNAL, err))
            }
            _ => Err(not_float()),
        },
        // FR-055-07
        Value::Bool(_) | Value::LogicalArray(_) | Value::Int(_) | Value::CharArray(_) => {
            Err(not_float())
        }
        // FR-055-30
        _ => Err(sec_error(
            &UNSUPPORTED_OPTION,
            "X must be a dense double or single array",
        )),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
// Observed values are copied verbatim from the imported observation record.
#[allow(clippy::excessive_precision, clippy::approx_constant)]
mod sec_tests {
    use super::sec_builtin;
    use crate::BuiltinResult;
    use futures::executor::block_on;
    use runmat_value::{
        CharArray, ComplexTensor, IntValue, IntegerStorage, LogicalArray, NumericDType, Tensor,
        Value,
    };
    use std::f64::consts::PI;

    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("sec", args))
    }

    fn n(x: f64) -> Value {
        Value::Num(x)
    }

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

    const MUST_BE_FLOAT: &str = "RunMat:math:mustBeFloat";
    const UNSUPPORTED: &str = "RunMat:sec:UnsupportedOption";

    // --- Observed (normative): one test per observed case --------------------

    /// case `zero` — sec.signature-primary, sec.values
    #[test]
    fn observed_zero() {
        assert_double(call(&[n(0.0)]).unwrap(), &[1, 1], &[1.0], 1e-12);
    }

    /// case `pi3` — sec.values
    #[test]
    fn observed_pi3() {
        assert_double(
            call(&[n(PI / 3.0)]).unwrap(),
            &[1, 1],
            &[1.9999999999999996],
            1e-12,
        );
    }

    /// case `pi` — sec.values
    #[test]
    fn observed_pi() {
        assert_double(call(&[n(PI)]).unwrap(), &[1, 1], &[-1.0], 1e-12);
    }

    /// case `negative` — sec.values
    #[test]
    fn observed_negative() {
        assert_double(
            call(&[n(-PI / 4.0)]).unwrap(),
            &[1, 1],
            &[1.4142135623730949],
            1e-12,
        );
    }

    /// case `pole` — sec.pole: finite and large.
    #[test]
    fn observed_pole() {
        let (_, _, got) = real(&call(&[n(PI / 2.0)]).unwrap());
        assert!(got[0].is_finite(), "sec(pi/2) must be finite");
        assert!(got[0] > 1e15, "sec(pi/2) must be large, got {}", got[0]);
        // The observed value, to a relative tolerance.
        assert!((got[0] - 16331239353195370.0).abs() / 16331239353195370.0 < 1e-12);
    }

    /// case `matrix` — sec.shape
    #[test]
    fn observed_matrix() {
        // column-major [0 pi; pi/3 -pi/3]
        let x = Value::Tensor(Tensor::new(vec![0.0, PI / 3.0, PI, -PI / 3.0], vec![2, 2]).unwrap());
        assert_double(
            call(&[x]).unwrap(),
            &[2, 2],
            &[1.0, 1.9999999999999996, -1.0, 1.9999999999999996],
            1e-12,
        );
    }

    /// case `nan` — sec.nonfinite
    #[test]
    fn observed_nan() {
        assert_double(call(&[n(f64::NAN)]).unwrap(), &[1, 1], &[f64::NAN], 0.0);
    }

    /// case `inf` — sec.nonfinite
    #[test]
    fn observed_inf() {
        assert_double(
            call(&[n(f64::INFINITY)]).unwrap(),
            &[1, 1],
            &[f64::NAN],
            0.0,
        );
    }

    /// case `complex` — sec.complex
    #[test]
    fn observed_complex() {
        match call(&[Value::Complex(1.0, 1.0)]).unwrap() {
            Value::Complex(re, im) => {
                assert!((re - 0.49833703055518686).abs() < 1e-12, "re {re}");
                assert!((im - 0.59108384172104511).abs() < 1e-12, "im {im}");
            }
            other => panic!("expected a complex scalar, got {other:?}"),
        }
    }

    /// case `single` — sec.single
    #[test]
    fn observed_single() {
        let x = Value::Tensor(Tensor::from_f32(vec![1.0], vec![1, 1]).unwrap());
        let (dtype, shape, values) = real(&call(&[x]).unwrap());
        assert_eq!(dtype, NumericDType::F32);
        assert_eq!(shape, vec![1, 1]);
        assert!((values[0] - 1.8508157730102539).abs() < 1e-6);
    }

    /// case `empty` — sec.shape
    #[test]
    fn observed_empty() {
        let x = Value::Tensor(Tensor::new(vec![], vec![0, 0]).unwrap());
        assert_double(call(&[x]).unwrap(), &[0, 0], &[], 0.0);
    }

    /// case `err-logical` — sec.error-logical
    #[test]
    fn observed_err_logical() {
        assert_eq!(identifier(call(&[Value::Bool(true)])), MUST_BE_FLOAT);
    }

    /// case `err-int` — sec.error-logical
    #[test]
    fn observed_err_int() {
        assert_eq!(
            identifier(call(&[Value::Int(IntValue::I8(1))])),
            MUST_BE_FLOAT
        );
    }

    /// case `err-char` — sec.error-logical
    #[test]
    fn observed_err_char() {
        assert_eq!(
            identifier(call(&[Value::CharArray(CharArray::new_row("a"))])),
            MUST_BE_FLOAT
        );
    }

    /// case `err-none` — sec.error-none
    #[test]
    fn observed_err_none() {
        assert_eq!(identifier(call(&[])), "RunMat:minrhs");
    }

    // --- Documented choices --------------------------------------------------

    /// FR-055-07 extended: integer/logical arrays of any class.
    #[test]
    fn documented_choice_array_classes() {
        let ints = Value::Tensor(
            Tensor::new_integer(IntegerStorage::U16(vec![1, 2]), vec![1, 2]).unwrap(),
        );
        assert_eq!(identifier(call(&[ints])), MUST_BE_FLOAT);
        let mask = Value::LogicalArray(LogicalArray::new(vec![1, 0], vec![1, 2]).unwrap());
        assert_eq!(identifier(call(&[mask])), MUST_BE_FLOAT);
    }

    /// FR-055-30: unobserved classes are unsupported.
    #[test]
    fn documented_choice_unsupported_classes() {
        assert_eq!(
            identifier(call(&[Value::String("a".to_string())])),
            UNSUPPORTED
        );
    }

    /// FR-055-32: complex arrays keep shape; complex single stays single.
    #[test]
    fn documented_choice_complex_arrays() {
        let z = Value::ComplexTensor(
            ComplexTensor::new(vec![(1.0, 1.0), (0.0, 0.0)], vec![1, 2]).unwrap(),
        );
        let Value::ComplexTensor(out) = call(&[z]).unwrap() else {
            panic!("expected a complex tensor");
        };
        assert_eq!(out.shape, vec![1, 2]);
        let values = out.materialize_f64();
        assert!((values[0].0 - 0.49833703055518686).abs() < 1e-12);
        assert_eq!(values[1], (1.0, 0.0));
        let zs =
            Value::ComplexTensor(ComplexTensor::from_f32(vec![(1.0, 1.0)], vec![1, 1]).unwrap());
        let Value::ComplexTensor(out) = call(&[zs]).unwrap() else {
            panic!("expected a complex tensor");
        };
        assert_eq!(out.numeric_dtype(), NumericDType::F32);
    }

    /// FR-055-29: arity and output count.
    #[test]
    fn documented_choice_arity_and_outputs() {
        assert_eq!(
            identifier(call(&[n(1.0), n(2.0)])),
            "RunMat:sec:InvalidInput"
        );
        let _guard = crate::output_count::push_output_count(Some(2));
        let err = block_on(sec_builtin(vec![n(1.0)])).unwrap_err();
        assert_eq!(err.identifier(), Some("RunMat:sec:TooManyOutputs"));
    }

    /// FR-055-33 (unresolved, sec.q-warning-coverage): no warning.
    #[test]
    fn unresolved_no_warning() {
        assert!(call(&[n(1.0)]).is_ok());
    }
}
