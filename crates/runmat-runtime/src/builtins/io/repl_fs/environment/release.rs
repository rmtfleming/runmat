//! Clean-room `isMATLABReleaseOlderThan` (compare the running release).
//!
//! Clean-room feature `specs/053-environment/`. Normative behaviour comes
//! only from the approved export `isMATLABReleaseOlderThan` 0.2.0 (see that
//! feature's `provenance.md`). The running release is RunMat's compatibility
//! pin `crate::compatibility::MATLAB_COMPATIBILITY_RELEASE` (decision
//! D-053-3). Observed `MATLAB:*` identifiers are raised as `RunMat:*`
//! (D-053-1).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::Value;

use crate::{build_runtime_error, BuiltinResult, RuntimeError};

const NAME: &str = "isMATLABReleaseOlderThan";

// ---------------------------------------------------------------------------
// Descriptor
// ---------------------------------------------------------------------------

const INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "release",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Release name such as 'R2022a', as a char row or string scalar.",
}];

const OUTPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "tf",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "True when the running release is older than release.",
}];

const SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "tf = isMATLABReleaseOlderThan(release)",
    inputs: &INPUTS,
    outputs: &OUTPUTS,
}];

const INVALID_RELEASE: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ISMATLABRELEASEOLDERTHAN.INVALID_RELEASE_INPUT",
    identifier: Some("RunMat:isMATLABReleaseOlderThan:invalidReleaseInput"),
    when: "The text is not a release name of the form R<year>a or R<year>b.",
    message: "isMATLABReleaseOlderThan: invalid release name",
};

const MUST_BE_TEXT_SCALAR: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ISMATLABRELEASEOLDERTHAN.MUST_BE_TEXT_SCALAR",
    identifier: Some("RunMat:validators:mustBeTextScalar"),
    when: "The input is not a char row or string scalar.",
    message: "isMATLABReleaseOlderThan: release must be a text scalar",
};

const MINRHS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ISMATLABRELEASEOLDERTHAN.MINRHS",
    identifier: Some("RunMat:minrhs"),
    when: "No input is given.",
    message: "isMATLABReleaseOlderThan: not enough input arguments",
};

const UNSUPPORTED_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ISMATLABRELEASEOLDERTHAN.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:isMATLABReleaseOlderThan:UnsupportedOption"),
    when: "More than one input is given (not yet specified).",
    message: "isMATLABReleaseOlderThan: this form is not yet specified",
};

const TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ISMATLABRELEASEOLDERTHAN.TOO_MANY_OUTPUTS",
    identifier: Some("RunMat:isMATLABReleaseOlderThan:TooManyOutputs"),
    when: "More than one output was requested.",
    message: "isMATLABReleaseOlderThan: too many output arguments",
};

const INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ISMATLABRELEASEOLDERTHAN.INTERNAL",
    identifier: Some("RunMat:isMATLABReleaseOlderThan:Internal"),
    when: "RunMat's compatibility release pin cannot be parsed.",
    message: "isMATLABReleaseOlderThan: internal runtime failure",
};

const ERRORS: [BuiltinErrorDescriptor; 6] = [
    INVALID_RELEASE,
    MUST_BE_TEXT_SCALAR,
    MINRHS,
    UNSUPPORTED_OPTION,
    TOO_MANY_OUTPUTS,
    INTERNAL,
];

pub const IS_MATLAB_RELEASE_OLDER_THAN_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ERRORS,
};

fn release_error(
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
    name = "isMATLABReleaseOlderThan",
    category = "io/repl_fs",
    summary = "True when the MATLAB release RunMat is compatible with is older than the named release.",
    keywords = "isMATLABReleaseOlderThan,release,version,compatibility",
    accel = "cpu",
    descriptor(
        crate::builtins::io::repl_fs::environment::release::IS_MATLAB_RELEASE_OLDER_THAN_DESCRIPTOR
    ),
    builtin_path = "crate::builtins::io::repl_fs::environment::release"
)]
pub(crate) async fn is_matlab_release_older_than_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    // FR-053-23
    if let Some(count) = crate::output_count::current_output_count() {
        if count > 1 {
            return Err(release_error(
                &TOO_MANY_OUTPUTS,
                format!("returns one output, {count} were requested"),
            ));
        }
    }
    // FR-053-10, FR-053-23
    let release = match args.as_slice() {
        [] => return Err(release_error(&MINRHS, "a release name is required")),
        [release] => release,
        _ => {
            return Err(release_error(
                &UNSUPPORTED_OPTION,
                "only the one-argument form is supported",
            ))
        }
    };
    // FR-053-07, FR-053-09, FR-053-24
    let text = text_scalar(release)?;
    // FR-053-08, FR-053-24
    let requested = parse_release(&text).ok_or_else(|| {
        release_error(
            &INVALID_RELEASE,
            format!("'{text}' is not a release name such as R2022a"),
        )
    })?;
    let running = parse_release(crate::compatibility::MATLAB_COMPATIBILITY_RELEASE)
        .ok_or_else(|| release_error(&INTERNAL, "the compatibility release pin is malformed"))?;
    // FR-053-05, FR-053-06
    Ok(Value::Bool(running < requested))
}

/// Text scalar: char row (or 0×0 char), string scalar (FR-053-07, FR-053-24).
fn text_scalar(value: &Value) -> BuiltinResult<String> {
    match value {
        Value::String(text) => Ok(text.clone()),
        Value::StringArray(array) if array.data.len() == 1 => Ok(array.data[0].clone()),
        Value::CharArray(chars) if chars.rows == 1 || chars.data.is_empty() => {
            Ok(chars.data.iter().collect())
        }
        _ => Err(release_error(
            &MUST_BE_TEXT_SCALAR,
            "release must be a character vector or string scalar",
        )),
    }
}

/// Parse `R<yyyy><a|b>` (FR-053-24) into an ordered `(year, half)` key, where
/// `a` = 0 and `b` = 1. The leading `R` may be lower case (FR-053-07).
pub(crate) fn parse_release(text: &str) -> Option<(u32, u8)> {
    let bytes = text.as_bytes();
    if bytes.len() != 6 || !matches!(bytes[0], b'R' | b'r') {
        return None;
    }
    let year = &text[1..5];
    if !year.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let half = match bytes[5] {
        b'a' | b'A' => 0,
        b'b' | b'B' => 1,
        _ => return None,
    };
    Some((year.parse().ok()?, half))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod is_matlab_release_older_than_tests {
    use super::{is_matlab_release_older_than_builtin, parse_release};
    use crate::BuiltinResult;
    use futures::executor::block_on;
    use runmat_value::{CellArray, CharArray, StringArray, Value};

    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("isMATLABReleaseOlderThan", args))
    }

    /// The running release: RunMat's compatibility pin (D-053-3). No
    /// release literal is written here; the pin lives only in
    /// `compatibility.rs`.
    fn pin() -> &'static str {
        crate::compatibility::MATLAB_COMPATIBILITY_RELEASE
    }

    /// The pin with its half letter replaced (e.g. the `b` release of the
    /// pin's year).
    fn pin_with_half(half: char) -> String {
        let mut text = pin().to_string();
        text.pop();
        text.push(half);
        text
    }

    /// The observed truth table holds for the pin exactly when the pin is an
    /// `a` release with R2022a <= pin < R2027a; the observed release meets
    /// this. Checked first so that a pin change fails clearly.
    fn assert_observed_table_applies() {
        let pin = parse_release(pin()).expect("compatibility pin parses");
        assert_eq!(pin.1, 0, "the observed table needs an `a` release pin");
        assert!(((2022, 0)..(2027, 0)).contains(&pin), "pin {pin:?}");
    }

    fn older(text: &str) -> Value {
        assert_observed_table_applies();
        call(&[Value::CharArray(CharArray::new_row(text))]).unwrap()
    }

    fn identifier(result: BuiltinResult<Value>) -> String {
        match result {
            Ok(value) => panic!("expected an error, got {value:?}"),
            Err(err) => err.identifier().unwrap_or("<none>").to_string(),
        }
    }

    const INVALID: &str = "RunMat:isMATLABReleaseOlderThan:invalidReleaseInput";
    const TEXT_SCALAR: &str = "RunMat:validators:mustBeTextScalar";

    // --- Observed (normative): one test per observed case --------------------

    /// case `older` — signature-primary, comparison
    #[test]
    fn observed_older() {
        assert_eq!(older("R2022a"), Value::Bool(false));
    }

    /// case `same` — comparison
    #[test]
    fn observed_same() {
        // `same`: the running release itself.
        assert_eq!(older(pin()), Value::Bool(false));
    }

    /// case `later-b` — comparison
    #[test]
    fn observed_later_b() {
        // `later-b`: the b release of the running release's year.
        assert_eq!(older(&pin_with_half('b')), Value::Bool(true));
    }

    /// case `later-year` — comparison
    #[test]
    fn observed_later_year() {
        assert_eq!(older("R2030a"), Value::Bool(true));
    }

    /// case `much-older` — comparison
    #[test]
    fn observed_much_older() {
        assert_eq!(older("R2006a"), Value::Bool(false));
    }

    /// case `string` — text, comparison
    #[test]
    fn observed_string() {
        assert_observed_table_applies();
        assert_eq!(
            call(&[Value::String("R2027a".to_string())]).unwrap(),
            Value::Bool(true)
        );
    }

    /// case `lowercase` — text
    #[test]
    fn observed_lowercase() {
        assert_eq!(older("r2022a"), Value::Bool(false));
    }

    /// case `err-bad-name` — error-bad-name
    #[test]
    fn observed_err_bad_name() {
        assert_eq!(
            identifier(call(&[Value::CharArray(CharArray::new_row("2022a"))])),
            INVALID
        );
    }

    /// case `err-numeric` — error-numeric
    #[test]
    fn observed_err_numeric() {
        assert_eq!(identifier(call(&[Value::Num(2022.0)])), TEXT_SCALAR);
    }

    /// case `err-none` — error-none
    #[test]
    fn observed_err_none() {
        assert_eq!(identifier(call(&[])), "RunMat:minrhs");
    }

    // --- Documented choices ---------------------------------------------------

    /// FR-053-24 (documented choice): release-name grammar.
    #[test]
    fn documented_choice_grammar() {
        assert_eq!(parse_release("R2022a"), Some((2022, 0)));
        assert_eq!(parse_release("r2022B"), Some((2022, 1)));
        for bad in [
            "",
            "R2022",
            "R22a",
            "R2022c",
            " R2022a",
            "R2022a ",
            "R2022a Update 3",
            "X2022a",
        ] {
            assert_eq!(parse_release(bad), None, "{bad:?}");
            assert_eq!(
                identifier(call(&[Value::CharArray(CharArray::new_row(bad))])),
                INVALID,
                "{bad:?}"
            );
        }
        // Upper-case half letter is accepted.
        assert_eq!(older(&pin_with_half('B')), Value::Bool(true));
    }

    /// FR-053-24 (documented choice): non-text and non-scalar text.
    #[test]
    fn documented_choice_non_text_scalar() {
        let matrix =
            Value::CharArray(CharArray::new("R2022aR2023a".chars().collect(), 2, 6).unwrap());
        let strings = Value::StringArray(
            StringArray::new(vec!["R2022a".into(), "R2023a".into()], vec![1, 2]).unwrap(),
        );
        let cell = Value::Cell(CellArray::new(vec![Value::from("R2022a")], 1, 1).unwrap());
        for value in [Value::Bool(true), matrix, strings, cell] {
            assert_eq!(identifier(call(&[value])), TEXT_SCALAR);
        }
    }

    /// FR-053-23 (documented choice): more than one input or output.
    #[test]
    fn documented_choice_arity() {
        assert_eq!(
            identifier(call(&[Value::from("R2022a"), Value::from("release")])),
            "RunMat:isMATLABReleaseOlderThan:UnsupportedOption"
        );
        let _guard = crate::output_count::push_output_count(Some(2));
        let err = block_on(is_matlab_release_older_than_builtin(vec![Value::from(
            "R2022a",
        )]))
        .unwrap_err();
        assert_eq!(
            err.identifier(),
            Some("RunMat:isMATLABReleaseOlderThan:TooManyOutputs")
        );
    }

    /// D-053-3: the comparison follows the compatibility pin.
    #[test]
    fn documented_choice_pin_is_not_older_than_itself() {
        let pin = crate::compatibility::MATLAB_COMPATIBILITY_RELEASE;
        assert!(parse_release(pin).is_some());
        assert_eq!(call(&[Value::from(pin)]).unwrap(), Value::Bool(false));
    }

    /// FR-053-27 (unresolved, q-warning-coverage): result only, no warning.
    #[test]
    fn unresolved_no_warning() {
        assert!(matches!(older("R2022a"), Value::Bool(_)));
    }
}
