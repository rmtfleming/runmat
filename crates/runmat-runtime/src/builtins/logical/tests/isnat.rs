//! Clean-room `isnat` (elementwise NaT test for datetime arrays).
//!
//! Clean-room feature `specs/054-types-structs-text/`. Normative behaviour
//! comes only from the approved export `isnat` 0.2.0 (see that feature's
//! `provenance.md`). A RunMat datetime stores serial date numbers, and a NaT
//! element is stored as NaN (decision D-054-4). A non-datetime input raises
//! `RunMat:isnat:UndefinedFunction` (D-054-2, documented choice).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{LogicalArray, Value};

use crate::{build_runtime_error, BuiltinResult, RuntimeError};

const NAME: &str = "isnat";

// ---------------------------------------------------------------------------
// Descriptor
// ---------------------------------------------------------------------------

const INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "D",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "datetime array.",
}];

const OUTPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "tf",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Logical array the size of D, true where D is NaT.",
}];

const SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "tf = isnat(D)",
    inputs: &INPUTS,
    outputs: &OUTPUTS,
}];

const UNDEFINED_FUNCTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ISNAT.UNDEFINED_FUNCTION",
    identifier: Some("RunMat:isnat:UndefinedFunction"),
    when: "The input is not a datetime array.",
    message: "isnat: input must be a datetime array",
};

const INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ISNAT.INVALID_INPUT",
    identifier: Some("RunMat:isnat:InvalidInput"),
    when: "The call does not have exactly one input.",
    message: "isnat: expected isnat(D)",
};

const TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ISNAT.TOO_MANY_OUTPUTS",
    identifier: Some("RunMat:isnat:TooManyOutputs"),
    when: "More than one output was requested.",
    message: "isnat: too many output arguments",
};

const INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ISNAT.INTERNAL",
    identifier: Some("RunMat:isnat:Internal"),
    when: "The datetime storage cannot be read or the result cannot be built.",
    message: "isnat: internal runtime failure",
};

const ERRORS: [BuiltinErrorDescriptor; 4] = [
    UNDEFINED_FUNCTION,
    INVALID_INPUT,
    TOO_MANY_OUTPUTS,
    INTERNAL,
];

pub const ISNAT_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ERRORS,
};

fn isnat_error(
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
    name = "isnat",
    category = "logical/tests",
    summary = "Elementwise test for NaT (not-a-time) elements of a datetime array.",
    keywords = "isnat,NaT,datetime,missing,not a time",
    accel = "cpu",
    descriptor(crate::builtins::logical::tests::isnat::ISNAT_DESCRIPTOR),
    builtin_path = "crate::builtins::logical::tests::isnat"
)]
pub(crate) async fn isnat_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    // FR-054-23
    if let Some(count) = crate::output_count::current_output_count() {
        if count > 1 {
            return Err(isnat_error(
                &TOO_MANY_OUTPUTS,
                format!("isnat returns one output, {count} were requested"),
            ));
        }
    }
    let [value] = args.as_slice() else {
        return Err(isnat_error(&INVALID_INPUT, "isnat takes exactly one input"));
    };
    // FR-054-25 (D-054-2)
    if !crate::builtins::datetime::is_datetime_object(value) {
        return Err(isnat_error(
            &UNDEFINED_FUNCTION,
            "isnat is defined for datetime arrays only",
        ));
    }
    // FR-054-05..07 (D-054-4)
    let serials = crate::builtins::datetime::serials_from_datetime_value(value)
        .map_err(|err| isnat_error(&INTERNAL, err.message().to_string()))?;
    let flags: Vec<u8> = serials
        .materialize_f64()
        .iter()
        .map(|v| u8::from(v.is_nan()))
        .collect();
    if flags.len() == 1 && serials.shape.iter().all(|&d| d == 1) {
        return Ok(Value::Bool(flags[0] == 1));
    }
    LogicalArray::new(flags, serials.shape.clone())
        .map(Value::LogicalArray)
        .map_err(|err| isnat_error(&INTERNAL, err))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod isnat_tests {
    use super::isnat_builtin;
    use crate::BuiltinResult;
    use futures::executor::block_on;
    use runmat_value::{CharArray, Tensor, Value};

    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("isnat", args))
    }

    /// datetime(y, m, d) through RunMat's own constructor.
    fn ymd(y: f64, m: f64, d: f64) -> Value {
        block_on(crate::call_builtin_async(
            "datetime",
            &[Value::Num(y), Value::Num(m), Value::Num(d)],
        ))
        .unwrap()
    }

    fn serial(value: &Value) -> f64 {
        let t = crate::builtins::datetime::serials_from_datetime_value(value).unwrap();
        t.materialize_f64()[0]
    }

    /// A datetime array built from serial numbers; NaN is NaT (D-054-6).
    fn from_serials(serials: Vec<f64>, shape: Vec<usize>) -> Value {
        crate::builtins::datetime::datetime_object_from_serial_tensor(
            Tensor::new(serials, shape).unwrap(),
            "dd-MMM-yyyy",
        )
        .unwrap()
    }

    fn nat() -> Value {
        from_serials(vec![f64::NAN], vec![1, 1])
    }

    /// (shape, column-major flags) of a logical result.
    fn logical(value: Value) -> (Vec<usize>, Vec<bool>) {
        match value {
            Value::Bool(b) => (vec![1, 1], vec![b]),
            Value::LogicalArray(a) => (a.shape.clone(), a.data.iter().map(|&b| b != 0).collect()),
            other => panic!("expected a logical result, got {other:?}"),
        }
    }

    fn identifier(result: BuiltinResult<Value>) -> String {
        match result {
            Ok(value) => panic!("expected an error, got {value:?}"),
            Err(err) => err.identifier().unwrap_or("<none>").to_string(),
        }
    }

    const UNDEFINED: &str = "RunMat:isnat:UndefinedFunction";

    // --- Observed (normative): one test per observed case --------------------

    /// case `nat` — isnat.signature-primary, isnat.elementwise (NaT built
    /// internally, D-054-6)
    #[test]
    fn observed_nat() {
        assert_eq!(call(&[nat()]).unwrap(), Value::Bool(true));
    }

    /// case `valid` — isnat.elementwise
    #[test]
    fn observed_valid() {
        assert_eq!(call(&[ymd(2020.0, 1.0, 2.0)]).unwrap(), Value::Bool(false));
    }

    /// case `array` — isnat.elementwise ([valid NaT valid], 1x3)
    #[test]
    fn observed_array() {
        let a = serial(&ymd(2020.0, 1.0, 2.0));
        let b = serial(&ymd(2021.0, 3.0, 4.0));
        let d = from_serials(vec![a, f64::NAN, b], vec![1, 3]);
        assert_eq!(
            logical(call(&[d]).unwrap()),
            (vec![1, 3], vec![false, true, false])
        );
    }

    /// case `matrix` — isnat.elementwise ([valid NaT; valid valid], 2x2)
    #[test]
    fn observed_matrix() {
        let s11 = serial(&ymd(2020.0, 1.0, 1.0));
        let s21 = serial(&ymd(2020.0, 1.0, 2.0));
        let s22 = serial(&ymd(2020.0, 1.0, 3.0));
        // column-major: (1,1), (2,1), (1,2), (2,2)
        let d = from_serials(vec![s11, s21, f64::NAN, s22], vec![2, 2]);
        assert_eq!(
            logical(call(&[d]).unwrap()),
            (vec![2, 2], vec![false, false, true, false])
        );
    }

    /// case `empty` — isnat.empty (0x1 datetime built internally, D-054-6)
    #[test]
    fn observed_empty() {
        let d = from_serials(Vec::new(), vec![0, 1]);
        assert_eq!(logical(call(&[d]).unwrap()), (vec![0, 1], Vec::new()));
    }

    /// case `from-text-nat` — isnat.elementwise (`datetime('NaT')`, D-054-7)
    ///
    /// Ignored: RunMat's `datetime` constructor rejects the text 'NaT'
    /// (`RunMat:datetime:InvalidInput`), an upstream gap outside this
    /// feature. The `isnat` side is covered by `observed_nat`.
    #[test]
    #[ignore = "D-054-7: RunMat datetime('NaT') is not supported upstream"]
    fn observed_from_text_nat() {
        let d = block_on(crate::call_builtin_async(
            "datetime",
            &[Value::CharArray(CharArray::new_row("NaT"))],
        ))
        .expect("RunMat datetime('NaT')");
        assert_eq!(call(&[d]).unwrap(), Value::Bool(true));
    }

    // --- Documented choices ---------------------------------------------------

    /// FR-054-25 (documented choice; cases `err-double`, `err-duration`,
    /// `err-char` are non-normative): non-datetime input.
    #[test]
    fn documented_choice_non_datetime() {
        let hours = block_on(crate::call_builtin_async("hours", &[Value::Num(1.0)])).unwrap();
        for value in [
            Value::Num(f64::NAN),
            hours,
            Value::CharArray(CharArray::new_row("NaT")),
        ] {
            assert_eq!(identifier(call(&[value])), UNDEFINED);
        }
    }

    /// FR-054-23 (documented choice): arity and outputs.
    #[test]
    fn documented_choice_arity() {
        assert_eq!(identifier(call(&[])), "RunMat:isnat:InvalidInput");
        assert_eq!(
            identifier(call(&[nat(), nat()])),
            "RunMat:isnat:InvalidInput"
        );
        let _guard = crate::output_count::push_output_count(Some(2));
        let err = block_on(isnat_builtin(vec![nat()])).unwrap_err();
        assert_eq!(err.identifier(), Some("RunMat:isnat:TooManyOutputs"));
    }

    /// FR-054-28 (unresolved, isnat.q-warning-coverage): result only.
    #[test]
    fn unresolved_no_warning() {
        assert!(matches!(call(&[nat()]).unwrap(), Value::Bool(true)));
    }
}
