//! Clean-room `reallog` (natural logarithm of non-negative real input).
//!
//! Clean-room feature `specs/050-numeric-conversion/`. Normative behaviour
//! comes only from the approved export `reallog` 0.2.0 (see that feature's
//! `provenance.md`). Observed `MATLAB:*` identifiers are raised as
//! `RunMat:*` (decision D-050-1). Unobserved classes raise
//! `RunMat:reallog:UnsupportedOption` (FR-050-25).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{NumericDType, Tensor, Value};

use crate::builtins::common::map_control_flow_with_builtin;
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const NAME: &str = "reallog";

// ---------------------------------------------------------------------------
// Descriptor
// ---------------------------------------------------------------------------

const REALLOG_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "x",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Real non-negative double or single array.",
}];

const REALLOG_OUTPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "y",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Natural logarithm, same size and class as x.",
}];

const REALLOG_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "y = reallog(x)",
    inputs: &REALLOG_INPUTS,
    outputs: &REALLOG_OUTPUTS,
}];

const COMPLEX_RESULT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.REALLOG.COMPLEX_RESULT",
    identifier: Some("RunMat:reallog:complexResult"),
    when: "x contains a negative value.",
    message: "reallog: x must be non-negative",
};

const MUST_BE_REAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.REALLOG.MUST_BE_REAL",
    identifier: Some("RunMat:validators:mustBeReal"),
    when: "x is complex.",
    message: "reallog: x must be real",
};

const MUST_BE_FLOAT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.REALLOG.MUST_BE_FLOAT",
    identifier: Some("RunMat:math:mustBeFloat"),
    when: "x is of an integer class.",
    message: "reallog: x must be double or single",
};

const INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.REALLOG.INVALID_INPUT",
    identifier: Some("RunMat:reallog:InvalidInput"),
    when: "The call does not have exactly one input.",
    message: "reallog: expected reallog(x)",
};

const UNSUPPORTED_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.REALLOG.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:reallog:UnsupportedOption"),
    when: "Unobserved classes of x: logical, char, sparse and others.",
    message: "reallog: this form is not yet specified",
};

const TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.REALLOG.TOO_MANY_OUTPUTS",
    identifier: Some("RunMat:reallog:TooManyOutputs"),
    when: "More than one output was requested.",
    message: "reallog: too many output arguments",
};

const INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.REALLOG.INTERNAL",
    identifier: Some("RunMat:reallog:Internal"),
    when: "The runtime cannot assemble the result.",
    message: "reallog: internal runtime failure",
};

const REALLOG_ERRORS: [BuiltinErrorDescriptor; 7] = [
    COMPLEX_RESULT,
    MUST_BE_REAL,
    MUST_BE_FLOAT,
    INVALID_INPUT,
    UNSUPPORTED_OPTION,
    TOO_MANY_OUTPUTS,
    INTERNAL,
];

pub const REALLOG_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &REALLOG_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &REALLOG_ERRORS,
};

fn reallog_error(
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
    name = "reallog",
    category = "math/elementwise",
    summary = "Natural logarithm of non-negative real input; errors instead of returning complex.",
    keywords = "reallog,log,natural logarithm,real,elementwise",
    accel = "cpu",
    descriptor(crate::builtins::math::elementwise::logarithms::reallog::REALLOG_DESCRIPTOR),
    builtin_path = "crate::builtins::math::elementwise::logarithms::reallog"
)]
pub(crate) async fn reallog_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    // FR-050-21
    if let Some(count) = crate::output_count::current_output_count() {
        if count > 1 {
            return Err(reallog_error(
                &TOO_MANY_OUTPUTS,
                format!("reallog returns one output, {count} were requested"),
            ));
        }
    }
    if args.len() != 1 {
        return Err(reallog_error(&INVALID_INPUT, "expected reallog(x)"));
    }
    // FR-050-26: gpuArray input is gathered; the result is a host array.
    let x = gather_if_needed_async(&args[0])
        .await
        .map_err(|err| map_control_flow_with_builtin(err, NAME))?;
    let (data, shape, single) = operand(&x)?;
    // FR-050-14
    if data.iter().any(|&v| v < 0.0) {
        return Err(reallog_error(
            &COMPLEX_RESULT,
            "x contains a negative value, so the result would be complex",
        ));
    }
    // FR-050-11..13
    let out: Vec<f64> = data.into_iter().map(f64::ln).collect();
    if single {
        return Tensor::from_f32(out.into_iter().map(|v| v as f32).collect(), shape)
            .map(Value::Tensor)
            .map_err(|err| reallog_error(&INTERNAL, err));
    }
    if out.len() == 1 && shape.iter().all(|&d| d == 1) {
        return Ok(Value::Num(out[0]));
    }
    Tensor::new(out, shape)
        .map(Value::Tensor)
        .map_err(|err| reallog_error(&INTERNAL, err))
}

/// Read x as (values, shape, is_single) after the class checks of
/// FR-050-14 and FR-050-25.
fn operand(value: &Value) -> BuiltinResult<(Vec<f64>, Vec<usize>, bool)> {
    match value {
        Value::Num(v) => Ok((vec![*v], vec![1, 1], false)),
        Value::Tensor(t) => match t.numeric_dtype() {
            NumericDType::F64 => Ok((t.materialize_f64(), t.shape.clone(), false)),
            NumericDType::F32 => Ok((t.materialize_f64(), t.shape.clone(), true)),
            _ => Err(reallog_error(
                &MUST_BE_FLOAT,
                "x must be double or single, not an integer class",
            )),
        },
        Value::Int(_) => Err(reallog_error(
            &MUST_BE_FLOAT,
            "x must be double or single, not an integer class",
        )),
        Value::Complex(_, _) | Value::ComplexTensor(_) => {
            Err(reallog_error(&MUST_BE_REAL, "x must not be complex"))
        }
        _ => Err(reallog_error(
            &UNSUPPORTED_OPTION,
            "x must be a real double or single array",
        )),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
// Observed values are copied verbatim from the imported observation record.
#[allow(clippy::excessive_precision, clippy::approx_constant)]
mod reallog_tests {
    use super::reallog_builtin;
    use crate::BuiltinResult;
    use futures::executor::block_on;
    use runmat_value::{
        CharArray, ComplexTensor, IntValue, IntegerStorage, LogicalArray, NumericDType, Tensor,
        Value,
    };

    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("reallog", args))
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

    const UNSUPPORTED: &str = "RunMat:reallog:UnsupportedOption";

    // --- Observed (normative): one test per observed case --------------------

    /// case `one` — reallog.signature-primary, reallog.values
    #[test]
    fn observed_one() {
        assert_double(call(&[n(1.0)]).unwrap(), &[1, 1], &[0.0], 0.0);
    }

    /// case `e` — reallog.values
    #[test]
    fn observed_e() {
        assert_double(call(&[n(2f64.exp())]).unwrap(), &[1, 1], &[2.0], 1e-12);
    }

    /// case `vector` — reallog.values
    #[test]
    fn observed_vector() {
        let x = Value::Tensor(Tensor::new(vec![1.0, 10.0, 100.0], vec![1, 3]).unwrap());
        assert_double(
            call(&[x]).unwrap(),
            &[1, 3],
            &[0.0, 2.3025850929940459, 4.6051701859880918],
            1e-12,
        );
    }

    /// case `zero` — reallog.specials
    #[test]
    fn observed_zero() {
        assert_double(call(&[n(0.0)]).unwrap(), &[1, 1], &[f64::NEG_INFINITY], 0.0);
    }

    /// case `inf` — reallog.specials
    #[test]
    fn observed_inf() {
        assert_double(
            call(&[n(f64::INFINITY)]).unwrap(),
            &[1, 1],
            &[f64::INFINITY],
            0.0,
        );
    }

    /// case `nan` — reallog.specials
    #[test]
    fn observed_nan() {
        assert_double(call(&[n(f64::NAN)]).unwrap(), &[1, 1], &[f64::NAN], 0.0);
    }

    /// case `matrix` — reallog.shape-class
    #[test]
    fn observed_matrix() {
        // column-major [1 2; 3 4]
        let x = Value::Tensor(Tensor::new(vec![1.0, 3.0, 2.0, 4.0], vec![2, 2]).unwrap());
        assert_double(
            call(&[x]).unwrap(),
            &[2, 2],
            &[
                0.0,
                1.0986122886681098,
                0.69314718055994529,
                1.3862943611198906,
            ],
            1e-12,
        );
    }

    /// case `single` — reallog.shape-class
    #[test]
    fn observed_single() {
        let x = Value::Tensor(Tensor::from_f32(vec![4.0], vec![1, 1]).unwrap());
        let (dtype, shape, values) = real(&call(&[x]).unwrap());
        assert_eq!(dtype, NumericDType::F32);
        assert_eq!(shape, vec![1, 1]);
        assert_eq!(values, vec![1.3862943649291992]);
    }

    /// case `empty` — reallog.shape-class
    #[test]
    fn observed_empty() {
        let x = Value::Tensor(Tensor::new(vec![], vec![0, 0]).unwrap());
        assert_double(call(&[x]).unwrap(), &[0, 0], &[], 0.0);
    }

    /// case `err-negative` — reallog.error-negative
    #[test]
    fn observed_err_negative() {
        assert_eq!(identifier(call(&[n(-1.0)])), "RunMat:reallog:complexResult");
    }

    /// case `err-complex` — reallog.error-complex
    #[test]
    fn observed_err_complex() {
        assert_eq!(
            identifier(call(&[Value::Complex(1.0, 1.0)])),
            "RunMat:validators:mustBeReal"
        );
    }

    /// case `err-int` — reallog.error-int
    #[test]
    fn observed_err_int() {
        assert_eq!(
            identifier(call(&[Value::Int(IntValue::I8(4))])),
            "RunMat:math:mustBeFloat"
        );
    }

    // --- Documented choices (FR-050-21, 25) ----------------------------------

    /// FR-050-14 extended: integer-class arrays and complex arrays.
    #[test]
    fn documented_choice_array_classes() {
        let ints =
            Value::Tensor(Tensor::new_integer(IntegerStorage::U8(vec![1, 2]), vec![1, 2]).unwrap());
        assert_eq!(identifier(call(&[ints])), "RunMat:math:mustBeFloat");
        let z = Value::ComplexTensor(
            ComplexTensor::new(vec![(1.0, 0.0), (2.0, 1.0)], vec![1, 2]).unwrap(),
        );
        assert_eq!(identifier(call(&[z])), "RunMat:validators:mustBeReal");
    }

    /// FR-050-14: one negative element anywhere fails the whole call.
    #[test]
    fn documented_choice_negative_in_array() {
        let x = Value::Tensor(Tensor::new(vec![1.0, -2.0, 3.0], vec![1, 3]).unwrap());
        assert_eq!(identifier(call(&[x])), "RunMat:reallog:complexResult");
    }

    /// FR-050-25: logical and char are unsupported.
    #[test]
    fn documented_choice_unsupported_classes() {
        assert_eq!(identifier(call(&[Value::Bool(true)])), UNSUPPORTED);
        let mask = Value::LogicalArray(LogicalArray::new(vec![1, 0], vec![1, 2]).unwrap());
        assert_eq!(identifier(call(&[mask])), UNSUPPORTED);
        assert_eq!(
            identifier(call(&[Value::CharArray(CharArray::new_row("a"))])),
            UNSUPPORTED
        );
    }

    /// FR-050-21: arity and output count.
    #[test]
    fn documented_choice_arity_and_outputs() {
        assert_eq!(identifier(call(&[])), "RunMat:reallog:InvalidInput");
        assert_eq!(
            identifier(call(&[n(1.0), n(2.0)])),
            "RunMat:reallog:InvalidInput"
        );
        let _guard = crate::output_count::push_output_count(Some(2));
        let err = block_on(reallog_builtin(vec![n(1.0)])).unwrap_err();
        assert_eq!(err.identifier(), Some("RunMat:reallog:TooManyOutputs"));
    }

    /// FR-050-30 (unresolved, reallog.q-warning-coverage): no warning.
    #[test]
    fn unresolved_no_warning() {
        assert!(call(&[n(2.0)]).is_ok());
    }
}
