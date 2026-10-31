//! Clean-room `MException`, `throw`, `addCause` and `lastwarn`.
//!
//! Clean-room features `specs/049-exceptions-warnings/` and
//! `specs/060-exception-causes/`. Normative behaviour comes only from the
//! approved exports `MException` 0.2.0 and `lastwarn` 0.2.0
//! (matlab-interface-spec `8f72edb`), `throw` 0.2.0 (`1156c97`) and
//! `addCause` 0.2.0 (`fc9f804`); see those features' `provenance.md`.
//! Exceptions are upstream `Value::MException` values with a cause list; a
//! thrown exception is a `RuntimeError` carrying the exception's identifier,
//! message and causes (`runtime_error::error_from_exception`), which
//! `try`/`catch` turns back into an `MException` through
//! `runtime_error::exception_from_error`.

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    ResolveContext, Type,
};
use runmat_macros::runtime_builtin;
use runmat_value::{CharArray, MException, Value};

use crate::builtins::common::format::format_variadic;
use crate::warning_store::RuntimeWarning;
use crate::{build_runtime_error, BuiltinResult, RuntimeError};

// ---------------------------------------------------------------------------
// Descriptors
// ---------------------------------------------------------------------------

const fn param(name: &'static str, description: &'static str) -> BuiltinParamDescriptor {
    BuiltinParamDescriptor {
        name,
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description,
    }
}

const fn variadic(name: &'static str, description: &'static str) -> BuiltinParamDescriptor {
    BuiltinParamDescriptor {
        name,
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Variadic,
        default: None,
        description,
    }
}

const P_ME: BuiltinParamDescriptor = param("ME", "An MException object.");

const MEXCEPTION_INPUTS: [BuiltinParamDescriptor; 3] = [
    param(
        "errID",
        "Identifier: '' or two or more ':'-separated non-empty components.",
    ),
    param("msg", "Message text or format."),
    variadic("A", "Values formatted into the message."),
];
const MEXCEPTION_OUTPUTS: [BuiltinParamDescriptor; 1] = [P_ME];
const MEXCEPTION_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "ME = MException(errID, msg, A1, ...)",
    inputs: &MEXCEPTION_INPUTS,
    outputs: &MEXCEPTION_OUTPUTS,
}];

const THROW_INPUTS: [BuiltinParamDescriptor; 1] = [P_ME];
const THROW_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "throw(ME)",
    inputs: &THROW_INPUTS,
    outputs: &[],
}];

const ADD_CAUSE_INPUTS: [BuiltinParamDescriptor; 2] = [
    P_ME,
    param("causeException", "An MException to add as a cause."),
];
const ADD_CAUSE_OUTPUTS: [BuiltinParamDescriptor; 1] = [P_ME];
const ADD_CAUSE_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "ME = addCause(ME, causeException)",
    inputs: &ADD_CAUSE_INPUTS,
    outputs: &ADD_CAUSE_OUTPUTS,
}];

const LASTWARN_OUTPUTS: [BuiltinParamDescriptor; 2] = [
    param("msg", "Last warning message (char)."),
    param("id", "Last warning identifier (char)."),
];
const LASTWARN_SET_INPUTS: [BuiltinParamDescriptor; 2] = [
    param("newMsg", "Message to store."),
    BuiltinParamDescriptor {
        name: "newId",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Optional,
        default: None,
        description: "Identifier to store ('' when omitted).",
    },
];
const LASTWARN_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "[msg, id] = lastwarn",
        inputs: &[],
        outputs: &LASTWARN_OUTPUTS,
    },
    BuiltinSignatureDescriptor {
        label: "[msg, id] = lastwarn(newMsg, newId)",
        inputs: &LASTWARN_SET_INPUTS,
        outputs: &LASTWARN_OUTPUTS,
    },
];

macro_rules! err {
    ($fn:literal, $code:literal, $id:literal, $when:literal) => {
        BuiltinErrorDescriptor {
            code: concat!("RM.", $code),
            identifier: Some($id),
            when: $when,
            message: concat!($fn, ": ", $when),
        }
    };
}

const MEXCEPTION_BAD_IDENTIFIER: BuiltinErrorDescriptor = err!(
    "MException",
    "MEXCEPTION.BAD_IDENTIFIER",
    "RunMat:MException:badIdentifier",
    "the identifier must be empty or have two or more non-empty ':'-separated components"
);
const MEXCEPTION_MINRHS: BuiltinErrorDescriptor = err!(
    "MException",
    "MEXCEPTION.MINRHS",
    "RunMat:minrhs",
    "not enough input arguments (an identifier and a message are required)"
);
const MEXCEPTION_INVALID_INPUT: BuiltinErrorDescriptor = err!(
    "MException",
    "MEXCEPTION.INVALID_INPUT",
    "RunMat:MException:InvalidInput",
    "the identifier and the message must be text"
);
const MEXCEPTION_TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = err!(
    "MException",
    "MEXCEPTION.TOO_MANY_OUTPUTS",
    "RunMat:MException:TooManyOutputs",
    "too many output arguments"
);
const MEXCEPTION_ERRORS: [BuiltinErrorDescriptor; 4] = [
    MEXCEPTION_BAD_IDENTIFIER,
    MEXCEPTION_MINRHS,
    MEXCEPTION_INVALID_INPUT,
    MEXCEPTION_TOO_MANY_OUTPUTS,
];

const THROW_UNDEFINED: BuiltinErrorDescriptor = err!(
    "throw",
    "THROW.UNDEFINED_FUNCTION",
    "RunMat:throw:UndefinedFunction",
    "the input must be an MException object"
);
const THROW_INVALID_INPUT: BuiltinErrorDescriptor = err!(
    "throw",
    "THROW.INVALID_INPUT",
    "RunMat:throw:InvalidInput",
    "throw expects exactly one MException input"
);
const THROW_ERRORS: [BuiltinErrorDescriptor; 2] = [THROW_UNDEFINED, THROW_INVALID_INPUT];

const ADD_CAUSE_REQUIRE_CLASS: BuiltinErrorDescriptor = err!(
    "addCause",
    "ADDCAUSE.REQUIRE_CLASS",
    "RunMat:class:RequireClass",
    "the cause must be an MException object"
);
const ADD_CAUSE_UNDEFINED: BuiltinErrorDescriptor = err!(
    "addCause",
    "ADDCAUSE.UNDEFINED_FUNCTION",
    "RunMat:addCause:UndefinedFunction",
    "the first input must be an MException object"
);
const ADD_CAUSE_INVALID_INPUT: BuiltinErrorDescriptor = err!(
    "addCause",
    "ADDCAUSE.INVALID_INPUT",
    "RunMat:addCause:InvalidInput",
    "addCause expects exactly two inputs (an MException and its cause)"
);
const ADD_CAUSE_TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = err!(
    "addCause",
    "ADDCAUSE.TOO_MANY_OUTPUTS",
    "RunMat:addCause:TooManyOutputs",
    "too many output arguments"
);
const ADD_CAUSE_ERRORS: [BuiltinErrorDescriptor; 4] = [
    ADD_CAUSE_REQUIRE_CLASS,
    ADD_CAUSE_UNDEFINED,
    ADD_CAUSE_INVALID_INPUT,
    ADD_CAUSE_TOO_MANY_OUTPUTS,
];

const LASTWARN_INVALID_INPUT: BuiltinErrorDescriptor = err!(
    "lastwarn",
    "LASTWARN.INVALID_INPUT",
    "RunMat:lastwarn:InvalidInput",
    "the message and the identifier must be text"
);
const LASTWARN_UNSUPPORTED: BuiltinErrorDescriptor = err!(
    "lastwarn",
    "LASTWARN.UNSUPPORTED",
    "RunMat:lastwarn:UnsupportedOption",
    "at most two inputs (message, identifier) are supported"
);
const LASTWARN_TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = err!(
    "lastwarn",
    "LASTWARN.TOO_MANY_OUTPUTS",
    "RunMat:lastwarn:TooManyOutputs",
    "too many output arguments"
);
const LASTWARN_ERRORS: [BuiltinErrorDescriptor; 3] = [
    LASTWARN_INVALID_INPUT,
    LASTWARN_UNSUPPORTED,
    LASTWARN_TOO_MANY_OUTPUTS,
];

pub const MEXCEPTION_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &MEXCEPTION_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &MEXCEPTION_ERRORS,
};
pub const THROW_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &THROW_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &THROW_ERRORS,
};
pub const ADD_CAUSE_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &ADD_CAUSE_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ADD_CAUSE_ERRORS,
};
pub const LASTWARN_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &LASTWARN_SIGNATURES,
    output_mode: BuiltinOutputMode::ByRequestedOutputCount,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &LASTWARN_ERRORS,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn fail(builtin: &'static str, error: &'static BuiltinErrorDescriptor) -> RuntimeError {
    let mut builder = build_runtime_error(error.message).with_builtin(builtin);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

fn unknown_type(_args: &[Type], _ctx: &ResolveContext) -> Type {
    Type::Unknown
}

fn lastwarn_type(_args: &[Type], _ctx: &ResolveContext) -> Type {
    Type::String
}

fn text(value: &Value) -> Option<String> {
    match value {
        Value::CharArray(ca) if ca.rows <= 1 => Some(ca.data.iter().collect()),
        Value::String(s) => Some(s.clone()),
        Value::StringArray(sa) if sa.data.len() == 1 => Some(sa.data[0].clone()),
        _ => None,
    }
}

/// A char row, or a 0×0 char for the empty text [lastwarn.class].
fn char_value(text: &str) -> Value {
    if text.is_empty() {
        Value::CharArray(CharArray::new(Vec::new(), 0, 0).expect("0x0 char"))
    } else {
        Value::CharArray(CharArray::new_row(text))
    }
}

fn ensure_single_output(
    builtin: &'static str,
    error: &'static BuiltinErrorDescriptor,
) -> BuiltinResult<()> {
    match crate::output_count::current_output_count() {
        Some(n) if n > 1 => Err(fail(builtin, error)),
        _ => Ok(()),
    }
}

/// `''` or at least two components, none empty [MException.identifier-rule,
/// MException.error-bad-id; throw.no-id].
fn identifier_is_valid(identifier: &str) -> bool {
    identifier.is_empty() || {
        let parts: Vec<&str> = identifier.split(':').collect();
        parts.len() >= 2 && parts.iter().all(|part| !part.is_empty())
    }
}

/// The error raised by `throw(ME)` and `ME.throw`: it carries the
/// exception's identifier (also `''`), message and causes, so the caught
/// exception has the same identifier, message and causes [throw.raises,
/// throw.caught, throw.cause, addCause.thrown].
pub(crate) fn throw_error(exception: &MException) -> RuntimeError {
    crate::runtime_error::error_from_exception(exception)
}

/// `addCause(ME, cause)` and `ME.addCause(cause)`: a copy of `ME` with
/// `cause` (and its own causes) appended; `ME` itself is unchanged
/// [addCause.append, addCause.nested, addCause.value-semantics,
/// addCause.method-syntax, MException.addcause].
pub(crate) fn add_cause(exception: &Value, cause: &Value) -> BuiltinResult<Value> {
    const NAME: &str = "addCause";
    match (exception, cause) {
        (Value::MException(exception), Value::MException(cause)) => Ok(Value::MException(
            exception.clone().with_cause(cause.clone()),
        )),
        // Non-normative (dispatch level, addCause.non-exception): U-060-1.
        (other, _) if !matches!(other, Value::MException(_)) => {
            Err(fail(NAME, &ADD_CAUSE_UNDEFINED))
        }
        // [addCause.error-char-cause]
        _ => Err(fail(NAME, &ADD_CAUSE_REQUIRE_CLASS)),
    }
}

/// Identifier reported by `lastwarn`: RunMat stores identifier-less warnings
/// under the warning builtin's default identifier, which `lastwarn` reports
/// as `''` (U-049-5) [lastwarn.no-id].
const WARNING_DEFAULT_IDENTIFIER: &str = "RunMat:warning";

fn reported_identifier(identifier: &str) -> &str {
    if identifier == WARNING_DEFAULT_IDENTIFIER {
        ""
    } else {
        identifier
    }
}

// ---------------------------------------------------------------------------
// MException
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "MException",
    category = "diagnostics",
    summary = "Create an exception object with an identifier and a formatted message.",
    keywords = "MException,exception,error,identifier,message",
    accel = "metadata",
    type_resolver(unknown_type),
    descriptor(crate::builtins::diagnostics::exceptions_warnings::MEXCEPTION_DESCRIPTOR),
    builtin_path = "crate::builtins::diagnostics::exceptions_warnings"
)]
pub(crate) async fn mexception_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    const NAME: &str = "MException";
    ensure_single_output(NAME, &MEXCEPTION_TOO_MANY_OUTPUTS)?;
    if args.len() < 2 {
        return Err(fail(NAME, &MEXCEPTION_MINRHS));
    }
    let args = gather(args).await?;
    let identifier = text(&args[0]).ok_or_else(|| fail(NAME, &MEXCEPTION_INVALID_INPUT))?;
    if !identifier_is_valid(&identifier) {
        return Err(fail(NAME, &MEXCEPTION_BAD_IDENTIFIER));
    }
    let format = text(&args[1]).ok_or_else(|| fail(NAME, &MEXCEPTION_INVALID_INPUT))?;
    let message = format_variadic(&format, &args[2..])?;
    Ok(Value::MException(MException::new(identifier, message)))
}

async fn gather(args: Vec<Value>) -> BuiltinResult<Vec<Value>> {
    let mut out = Vec::with_capacity(args.len());
    for value in &args {
        out.push(crate::gather_if_needed_async(value).await?);
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// throw / addCause
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "throw",
    category = "diagnostics",
    summary = "Throw an MException object as an error.",
    keywords = "throw,MException,exception,error",
    accel = "metadata",
    type_resolver(unknown_type),
    descriptor(crate::builtins::diagnostics::exceptions_warnings::THROW_DESCRIPTOR),
    builtin_path = "crate::builtins::diagnostics::exceptions_warnings"
)]
pub(crate) async fn throw_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    const NAME: &str = "throw";
    match args.as_slice() {
        [Value::MException(exception)] => Err(throw_error(exception)),
        [_] => Err(fail(NAME, &THROW_UNDEFINED)),
        _ => Err(fail(NAME, &THROW_INVALID_INPUT)),
    }
}

#[runtime_builtin(
    name = "addCause",
    category = "diagnostics",
    summary = "Return a copy of an MException with another MException appended to its causes.",
    keywords = "addCause,MException,cause,exception",
    accel = "metadata",
    type_resolver(unknown_type),
    descriptor(crate::builtins::diagnostics::exceptions_warnings::ADD_CAUSE_DESCRIPTOR),
    builtin_path = "crate::builtins::diagnostics::exceptions_warnings"
)]
pub(crate) async fn add_cause_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    const NAME: &str = "addCause";
    ensure_single_output(NAME, &ADD_CAUSE_TOO_MANY_OUTPUTS)?;
    match args.as_slice() {
        [exception, cause] => add_cause(exception, cause),
        _ => Err(fail(NAME, &ADD_CAUSE_INVALID_INPUT)),
    }
}

// ---------------------------------------------------------------------------
// lastwarn
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "lastwarn",
    category = "diagnostics",
    summary = "Query or set the message and identifier of the most recent warning.",
    keywords = "lastwarn,warning,last,message,identifier",
    accel = "metadata",
    type_resolver(lastwarn_type),
    descriptor(crate::builtins::diagnostics::exceptions_warnings::LASTWARN_DESCRIPTOR),
    builtin_path = "crate::builtins::diagnostics::exceptions_warnings"
)]
pub(crate) async fn lastwarn_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    const NAME: &str = "lastwarn";
    let out_count = crate::output_count::current_output_count();
    if matches!(out_count, Some(n) if n > 2) {
        return Err(fail(NAME, &LASTWARN_TOO_MANY_OUTPUTS));
    }
    if args.len() > 2 {
        return Err(fail(NAME, &LASTWARN_UNSUPPORTED));
    }
    let args = gather(args).await?;
    let mut texts = Vec::with_capacity(args.len());
    for value in &args {
        texts.push(text(value).ok_or_else(|| fail(NAME, &LASTWARN_INVALID_INPUT))?);
    }
    let previous = crate::warnings::with_policy(|policy| policy.lastwarn());
    if let Some(message) = texts.first() {
        // lastwarn(msg) stores msg with an empty identifier; lastwarn('')
        // clears both [lastwarn.set, lastwarn.set-msg-only, lastwarn.reset].
        let identifier = texts.get(1).cloned().unwrap_or_default();
        let stored = if message.is_empty() && identifier.is_empty() {
            None
        } else {
            Some(RuntimeWarning {
                identifier,
                message: message.clone(),
            })
        };
        crate::warnings::with_policy(|policy| policy.set_lastwarn(stored));
    }
    // Outputs report the state before any setting call [lastwarn.set-return].
    let (message, identifier) = previous
        .map(|warning| {
            let id = reported_identifier(&warning.identifier).to_string();
            (warning.message, id)
        })
        .unwrap_or_default();
    match out_count {
        Some(0) => Ok(Value::OutputList(Vec::new())),
        Some(2) => Ok(Value::OutputList(vec![
            char_value(&message),
            char_value(&identifier),
        ])),
        _ => Ok(char_value(&message)),
    }
}

#[cfg(test)]
mod exceptions_warnings_tests {
    use super::*;
    use futures::executor::block_on;

    fn call(name: &str, args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async(name, args))
    }

    fn ch(text: &str) -> Value {
        Value::CharArray(CharArray::new_row(text))
    }

    fn mexception(args: &[Value]) -> MException {
        match call("MException", args).expect("MException") {
            Value::MException(me) => me,
            other => panic!("expected MException, got {other:?}"),
        }
    }

    /// What a `catch` sees: the runtime's error-to-exception conversion.
    fn caught(err: RuntimeError) -> MException {
        crate::runtime_error::exception_from_error(&err)
    }

    fn text_of(value: &Value) -> String {
        match value {
            Value::CharArray(ca) => ca.data.iter().collect(),
            Value::String(s) => s.clone(),
            other => panic!("expected text, got {other:?}"),
        }
    }

    fn member(value: &Value, name: &str) -> BuiltinResult<Value> {
        block_on(crate::object::resolve::load_member(
            value.clone(),
            name.to_string(),
            false,
            None,
        ))
    }

    fn class_of(value: &Value) -> String {
        text_of(&call("class", std::slice::from_ref(value)).expect("class"))
    }

    fn reset_warnings() {
        crate::warnings::with_policy(|policy| policy.reset());
    }

    fn lastwarn_with_outputs(args: Vec<Value>, outputs: usize) -> BuiltinResult<Value> {
        let _guard = crate::output_count::push_output_count(Some(outputs));
        block_on(lastwarn_builtin(args))
    }

    fn lastwarn_pair() -> (Value, Value) {
        match lastwarn_with_outputs(Vec::new(), 2).expect("lastwarn") {
            Value::OutputList(mut values) => {
                let id = values.pop().expect("id");
                let msg = values.pop().expect("msg");
                (msg, id)
            }
            other => panic!("expected two outputs, got {other:?}"),
        }
    }

    fn lastwarn_texts() -> (String, String) {
        let (msg, id) = lastwarn_pair();
        (text_of(&msg), text_of(&id))
    }

    // ------------------------------------------------------------ MException

    #[test]
    fn observed_mexception_construct_formats_message() {
        // MException.signature-primary/-format, class, fields (case construct)
        let me = mexception(&[ch("a:b"), ch("value %d"), Value::Num(5.0)]);
        assert_eq!(me.identifier, "a:b");
        assert_eq!(me.message, "value 5");
        let value = Value::MException(me);
        assert_eq!(class_of(&value), "MException");
        let stack = member(&value, "stack").expect("stack");
        match stack {
            Value::Cell(cell) => assert_eq!((cell.rows, cell.cols), (0, 1)),
            other => panic!("expected 0x1 stack, got {other:?}"),
        }
    }

    #[test]
    fn observed_mexception_plain_percent_three_part() {
        // cases no-format, percent, three-part-id
        let me = mexception(&[ch("x:y"), ch("plain")]);
        assert_eq!(
            (me.identifier.as_str(), me.message.as_str()),
            ("x:y", "plain")
        );
        let me = mexception(&[ch("x:y"), ch("100%% sure")]);
        assert_eq!(me.message, "100% sure");
        let me = mexception(&[ch("a:b:c"), ch("m")]);
        assert_eq!(
            (me.identifier.as_str(), me.message.as_str()),
            ("a:b:c", "m")
        );
    }

    #[test]
    fn observed_mexception_member_reads() {
        // identifier and message are readable members (case construct)
        let value = Value::MException(mexception(&[ch("a:b"), ch("value %d"), Value::Num(5.0)]));
        assert_eq!(text_of(&member(&value, "identifier").unwrap()), "a:b");
        assert_eq!(text_of(&member(&value, "message").unwrap()), "value 5");
    }

    #[test]
    fn observed_mexception_throw_catch() {
        // case throw-catch: throw(MException('t:h','boom %s','x'))
        let me = Value::MException(mexception(&[ch("t:h"), ch("boom %s"), ch("x")]));
        let err = call("throw", &[me]).expect_err("throws");
        let e = caught(err);
        assert_eq!(
            (e.identifier.as_str(), e.message.as_str()),
            ("t:h", "boom x")
        );
        assert_eq!(class_of(&Value::MException(e)), "MException");
    }

    #[test]
    fn observed_mexception_rethrow_preserves() {
        // case rethrow: error('r:t','orig') caught, then rethrow(E)
        let err = call("error", &[ch("r:t"), ch("orig")]).expect_err("error");
        let e = caught(err);
        let err2 = call("rethrow", &[Value::MException(e)]).expect_err("rethrow");
        let e2 = caught(err2);
        assert_eq!(
            (e2.identifier.as_str(), e2.message.as_str()),
            ("r:t", "orig")
        );
    }

    #[test]
    fn rejects_mexception_bad_identifier() {
        // MException.error-bad-id (cases err-bad-id, err-empty-id-part)
        for id in ["nocolon", "a:"] {
            let err = call("MException", &[ch(id), ch("m")]).expect_err("bad id");
            assert_eq!(
                err.identifier(),
                Some("RunMat:MException:badIdentifier"),
                "{id}"
            );
        }
    }

    #[test]
    fn rejects_mexception_identifier_only() {
        // MException.error-minrhs (case err-no-message)
        let err = call("MException", &[ch("a:b")]).expect_err("minrhs");
        assert_eq!(err.identifier(), Some("RunMat:minrhs"));
    }

    #[test]
    fn unresolved_choice_mexception_inputs() {
        // U-049-1: empty identifier accepted (backed by throw case no-id);
        // a leading empty component is rejected; non-text inputs are
        // InvalidInput; more than one output is TooManyOutputs.
        assert_eq!(mexception(&[ch(""), ch("no id")]).identifier, "");
        let err = call("MException", &[ch(":b"), ch("m")]).expect_err(":b");
        assert_eq!(err.identifier(), Some("RunMat:MException:badIdentifier"));
        let err = call("MException", &[Value::Num(1.0), ch("m")]).expect_err("num id");
        assert_eq!(err.identifier(), Some("RunMat:MException:InvalidInput"));
        let _guard = crate::output_count::push_output_count(Some(2));
        let err = block_on(mexception_builtin(vec![ch("a:b"), ch("m")])).expect_err("2 outs");
        assert_eq!(err.identifier(), Some("RunMat:MException:TooManyOutputs"));
    }

    // ------------------------------------------------------------ throw

    #[test]
    fn observed_throw_direct_and_caught() {
        // throw.raises, throw.caught (cases direct, caught)
        let me = Value::MException(mexception(&[ch("n3:demo"), ch("hello")]));
        let err = call("throw", &[me]).expect_err("throws");
        assert_eq!(err.identifier(), Some("n3:demo"));
        let e = caught(err);
        assert_eq!(
            (e.identifier.as_str(), e.message.as_str()),
            ("n3:demo", "hello")
        );
    }

    #[test]
    fn observed_throw_formatted_and_percent() {
        // throw.formatted (cases formatted, percent)
        let me = Value::MException(mexception(&[
            ch("n3:fmt"),
            ch("value %d of %s"),
            Value::Num(3.0),
            ch("x"),
        ]));
        let e = caught(call("throw", &[me]).expect_err("throws"));
        assert_eq!(e.message, "value 3 of x");
        let me = Value::MException(mexception(&[ch("n3:pct"), ch("100%% sure")]));
        let e = caught(call("throw", &[me]).expect_err("throws"));
        assert_eq!(
            (e.identifier.as_str(), e.message.as_str()),
            ("n3:pct", "100% sure")
        );
    }

    #[test]
    fn observed_throw_dot_syntax() {
        // throw.dot-syntax (case dot): ME.throw and ME.throw()
        let me = Value::MException(mexception(&[ch("n3:dot"), ch("dot")]));
        let e = caught(member(&me, "throw").expect_err("ME.throw throws"));
        assert_eq!(
            (e.identifier.as_str(), e.message.as_str()),
            ("n3:dot", "dot")
        );
        let err = block_on(
            crate::object::dispatch::call_method_or_member_index_named_with_outputs(
                me,
                "throw".to_string(),
                Vec::new(),
                0,
                None,
            ),
        )
        .expect_err("ME.throw() throws");
        let e = caught(err);
        assert_eq!(
            (e.identifier.as_str(), e.message.as_str()),
            ("n3:dot", "dot")
        );
    }

    #[test]
    fn observed_throw_of_caught_error() {
        // throw.rethrow-caught: error('n3:inner','inner') caught, then throw(e1)
        let e1 = caught(call("error", &[ch("n3:inner"), ch("inner")]).expect_err("error"));
        let e2 = caught(call("throw", &[Value::MException(e1)]).expect_err("throw"));
        assert_eq!(
            (e2.identifier.as_str(), e2.message.as_str()),
            ("n3:inner", "inner")
        );
    }

    #[test]
    fn observed_throw_empty_identifier() {
        // throw.no-id (case no-id)
        let me = Value::MException(mexception(&[ch(""), ch("no id")]));
        let e = caught(call("throw", &[me]).expect_err("throws"));
        assert_eq!((e.identifier.as_str(), e.message.as_str()), ("", "no id"));
    }

    #[test]
    fn unresolved_choice_throw_non_exception() {
        // throw.non-exception (non-normative, dispatch level): U-049-2
        let err = call("throw", &[ch("n3 text")]).expect_err("char");
        assert_eq!(err.identifier(), Some("RunMat:throw:UndefinedFunction"));
        let mut st = runmat_value::StructValue::new();
        st.fields.insert("identifier".into(), ch("n3:s"));
        st.fields.insert("message".into(), ch("m"));
        let err = call("throw", &[Value::Struct(st)]).expect_err("struct");
        assert_eq!(err.identifier(), Some("RunMat:throw:UndefinedFunction"));
    }

    // ------------------------------------------------------------ addCause / causes (spec 060)

    fn me(id: &str, msg: &str) -> Value {
        Value::MException(mexception(&[ch(id), ch(msg)]))
    }

    fn add_cause_call(me: Value, cause: Value) -> BuiltinResult<Value> {
        call("addCause", &[me, cause])
    }

    /// `ME.cause` as (rows, cols, cause identifiers); asserts class cell.
    fn causes_of(value: &Value) -> (usize, usize, Vec<String>) {
        match member(value, "cause").expect("cause") {
            Value::Cell(cell) => {
                let ids = cell
                    .data
                    .iter()
                    .map(|v| match v {
                        Value::MException(c) => c.identifier.clone(),
                        other => panic!("expected MException cause, got {other:?}"),
                    })
                    .collect();
                (cell.rows, cell.cols, ids)
            }
            other => panic!("expected cause cell, got {other:?}"),
        }
    }

    fn identifier_of(value: &Value) -> String {
        text_of(&member(value, "identifier").expect("identifier"))
    }

    #[test]
    fn observed_add_cause_appends_in_order() {
        // addCause.signature-primary, addCause.append (cases one, two);
        // MException.addcause (returned object has 1 cause 'c:d')
        let outer = me("n4:outer", "o");
        let one = add_cause_call(outer.clone(), me("n4:c1", "c")).expect("one");
        assert_eq!(class_of(&one), "MException");
        assert_eq!(identifier_of(&one), "n4:outer");
        assert_eq!(causes_of(&one), (1, 1, vec!["n4:c1".to_string()]));
        let cause_cell = member(&one, "cause").expect("cause");
        assert_eq!(class_of(&cause_cell), "cell");
        let two = add_cause_call(one, me("n4:c2", "c")).expect("two");
        assert_eq!(identifier_of(&two), "n4:outer");
        assert_eq!(
            causes_of(&two),
            (2, 1, vec!["n4:c1".to_string(), "n4:c2".to_string()])
        );
        let top = add_cause_call(me("a:b", "top"), me("c:d", "inner")).expect("addcause");
        assert_eq!(causes_of(&top).2, vec!["c:d".to_string()]);
    }

    #[test]
    fn observed_add_cause_nested_causes_kept() {
        // addCause.nested (case nested)
        let inner = add_cause_call(me("n4:mid", "m"), me("n4:deep", "d")).expect("inner");
        let e = add_cause_call(me("n4:outer", "o"), inner).expect("outer");
        let cause = match member(&e, "cause").expect("cause") {
            Value::Cell(cell) => cell.data[0].clone(),
            other => panic!("expected cell, got {other:?}"),
        };
        assert_eq!(identifier_of(&cause), "n4:mid");
        assert_eq!(causes_of(&cause).2, vec!["n4:deep".to_string()]);
    }

    #[test]
    fn observed_add_cause_value_semantics() {
        // addCause.value-semantics (case value-semantics); MException.addcause
        // (the original object still has 0 causes)
        let e = me("n4:o", "o");
        let _discarded = add_cause_call(e.clone(), me("n4:c", "c")).expect("addCause");
        assert_eq!(causes_of(&e), (0, 1, Vec::new()));
    }

    #[test]
    fn observed_add_cause_survives_throw_and_catch() {
        // addCause.thrown (case thrown); throw.cause (case cause)
        let e = add_cause_call(me("n4:outer", "o"), me("n4:c1", "c")).expect("addCause");
        let x = Value::MException(caught(call("throw", &[e]).expect_err("throws")));
        assert_eq!(class_of(&x), "MException");
        assert_eq!(identifier_of(&x), "n4:outer");
        assert_eq!(causes_of(&x), (1, 1, vec!["n4:c1".to_string()]));
        let e = add_cause_call(me("n3:outer", "outer"), me("n3:in", "in")).expect("addCause");
        let x = Value::MException(caught(call("throw", &[e]).expect_err("throws")));
        assert_eq!(causes_of(&x).2, vec!["n3:in".to_string()]);
        // ME.throw and ME.throw() carry the causes too.
        let e = add_cause_call(me("n4:dot", "d"), me("n4:c", "c")).expect("addCause");
        let x = Value::MException(caught(member(&e, "throw").expect_err("ME.throw")));
        assert_eq!(causes_of(&x).2, vec!["n4:c".to_string()]);
    }

    #[test]
    fn observed_add_cause_caught_error_as_cause() {
        // addCause.caught-cause (case caught-builtin-cause)
        let raised = caught(call("error", &[ch("n4:raised"), ch("r")]).expect_err("error"));
        let e = add_cause_call(me("n4:outer", "o"), Value::MException(raised)).expect("addCause");
        assert_eq!(identifier_of(&e), "n4:outer");
        assert_eq!(causes_of(&e), (1, 1, vec!["n4:raised".to_string()]));
    }

    #[test]
    fn observed_add_cause_method_syntax() {
        // addCause.method-syntax (case method-syntax): e = e.addCause(c)
        let e = me("n4:o", "o");
        let sequence = block_on(
            crate::object::dispatch::call_method_or_member_index_named_with_outputs(
                e,
                "addCause".to_string(),
                vec![me("n4:c", "c")],
                1,
                None,
            ),
        )
        .expect("ME.addCause(c)");
        let mut values = sequence.into_values();
        assert_eq!(values.len(), 1);
        let e = values.pop().expect("value");
        assert_eq!(identifier_of(&e), "n4:o");
        assert_eq!(causes_of(&e), (1, 1, vec!["n4:c".to_string()]));
    }

    #[test]
    fn rejects_add_cause_non_exception_cause() {
        // addCause.error-char-cause (cases err-char-cause, err-struct-cause)
        let err = add_cause_call(me("n4:o", "o"), ch("text")).expect_err("char");
        assert_eq!(err.identifier(), Some("RunMat:class:RequireClass"));
        let mut st = runmat_value::StructValue::new();
        st.fields.insert("identifier".into(), ch("a:b"));
        let err = add_cause_call(me("n4:o", "o"), Value::Struct(st)).expect_err("struct");
        assert_eq!(err.identifier(), Some("RunMat:class:RequireClass"));
    }

    #[test]
    fn unresolved_choice_add_cause_inputs() {
        // addCause.non-exception (non-normative, dispatch level) and argument
        // count / output count: U-060-1.
        let mut st = runmat_value::StructValue::new();
        st.fields.insert("a".into(), Value::Num(1.0));
        let err = add_cause_call(Value::Struct(st), me("n4:c", "c")).expect_err("struct ME");
        assert_eq!(err.identifier(), Some("RunMat:addCause:UndefinedFunction"));
        let err = call("addCause", &[me("n4:o", "o")]).expect_err("one input");
        assert_eq!(err.identifier(), Some("RunMat:addCause:InvalidInput"));
        let err = call(
            "addCause",
            &[me("n4:o", "o"), me("n4:c", "c"), me("n4:d", "d")],
        )
        .expect_err("three inputs");
        assert_eq!(err.identifier(), Some("RunMat:addCause:InvalidInput"));
        let _guard = crate::output_count::push_output_count(Some(2));
        let err = block_on(add_cause_builtin(vec![me("n4:o", "o"), me("n4:c", "c")]))
            .expect_err("2 outs");
        assert_eq!(err.identifier(), Some("RunMat:addCause:TooManyOutputs"));
    }

    #[test]
    fn unresolved_choice_rethrow_keeps_causes() {
        // U-060-2: rethrow(ME) of an exception with causes keeps them.
        let e = add_cause_call(me("n4:outer", "o"), me("n4:c1", "c")).expect("addCause");
        let x = Value::MException(caught(call("rethrow", &[e]).expect_err("rethrow")));
        assert_eq!(identifier_of(&x), "n4:outer");
        assert_eq!(causes_of(&x).2, vec!["n4:c1".to_string()]);
    }

    #[test]
    fn causes_round_trip_through_value_codec() {
        // FR-060-07: the portable value codec keeps (nested) causes.
        let inner = add_cause_call(me("n4:mid", "m"), me("n4:deep", "d")).expect("inner");
        let e = add_cause_call(me("n4:outer", "o"), inner).expect("outer");
        let payload = crate::execution::value_codec::encode_inline_value(&e).expect("encode");
        let decoded = crate::execution::value_codec::decode_inline_value(&payload).expect("decode");
        assert_eq!(decoded, e);
    }

    // ------------------------------------------------------------ lastwarn

    #[test]
    fn observed_lastwarn_after_warning() {
        // lastwarn.after-warning, lastwarn.single-output
        reset_warnings();
        call("warning", &[ch("t:id"), ch("hello %d"), Value::Num(3.0)]).expect("warning");
        assert_eq!(
            lastwarn_texts(),
            ("hello 3".to_string(), "t:id".to_string())
        );
        call("warning", &[ch("t:id2"), ch("second")]).expect("warning");
        let single = lastwarn_with_outputs(Vec::new(), 1).expect("one output");
        assert_eq!(single, Value::CharArray(CharArray::new_row("second")));
        reset_warnings();
    }

    #[test]
    fn observed_lastwarn_set_forms() {
        // lastwarn.set, lastwarn.set-msg-only, lastwarn.reset
        reset_warnings();
        lastwarn_with_outputs(vec![ch("m"), ch("x:y")], 0).expect("set");
        assert_eq!(lastwarn_texts(), ("m".to_string(), "x:y".to_string()));
        lastwarn_with_outputs(vec![ch("prev"), ch("p:rev")], 0).expect("set");
        lastwarn_with_outputs(vec![ch("only")], 0).expect("set msg");
        assert_eq!(lastwarn_texts(), ("only".to_string(), String::new()));
        call("warning", &[ch("t:id"), ch("x")]).expect("warning");
        lastwarn_with_outputs(vec![ch("")], 0).expect("clear");
        assert_eq!(lastwarn_texts(), (String::new(), String::new()));
        reset_warnings();
    }

    #[test]
    fn observed_lastwarn_set_returns_previous() {
        // lastwarn.set-return
        reset_warnings();
        lastwarn_with_outputs(vec![ch("old"), ch("o:ld")], 0).expect("set");
        let (m, i) = match lastwarn_with_outputs(vec![ch("new"), ch("n:ew")], 2).expect("set") {
            Value::OutputList(values) => (text_of(&values[0]), text_of(&values[1])),
            other => panic!("expected two outputs, got {other:?}"),
        };
        assert_eq!((m.as_str(), i.as_str()), ("old", "o:ld"));
        assert_eq!(lastwarn_texts(), ("new".to_string(), "n:ew".to_string()));
        reset_warnings();
    }

    #[test]
    fn observed_lastwarn_disabled_warning_still_recorded() {
        // lastwarn.disabled
        reset_warnings();
        call("warning", &[ch("off"), ch("q:off")]).expect("off");
        lastwarn_with_outputs(vec![ch(""), ch("")], 0).expect("clear");
        call("warning", &[ch("q:off"), ch("hidden")]).expect("warning");
        assert_eq!(
            lastwarn_texts(),
            ("hidden".to_string(), "q:off".to_string())
        );
        reset_warnings();
    }

    #[test]
    fn observed_lastwarn_no_identifier() {
        // lastwarn.no-id
        reset_warnings();
        lastwarn_with_outputs(vec![ch(""), ch("z:z")], 0).expect("set");
        call("warning", &[ch("plain text")]).expect("warning");
        assert_eq!(lastwarn_texts(), ("plain text".to_string(), String::new()));
        reset_warnings();
    }

    #[test]
    fn observed_lastwarn_cleared_outputs_are_0x0_char() {
        // lastwarn.class
        reset_warnings();
        lastwarn_with_outputs(vec![ch(""), ch("")], 0).expect("clear");
        let (msg, id) = lastwarn_pair();
        for value in [msg, id] {
            match value {
                Value::CharArray(ca) => assert_eq!((ca.rows, ca.cols), (0, 0)),
                other => panic!("expected 0x0 char, got {other:?}"),
            }
        }
        reset_warnings();
    }

    #[test]
    fn unresolved_choice_lastwarn_inputs_and_outputs() {
        // U-049-4: non-text input, more than two inputs, more than two outputs.
        reset_warnings();
        let err = lastwarn_with_outputs(vec![Value::Num(1.0)], 0).expect_err("num");
        assert_eq!(err.identifier(), Some("RunMat:lastwarn:InvalidInput"));
        let err = lastwarn_with_outputs(vec![ch("a"), ch("b:c"), ch("d")], 0).expect_err("3 in");
        assert_eq!(err.identifier(), Some("RunMat:lastwarn:UnsupportedOption"));
        let err = lastwarn_with_outputs(Vec::new(), 3).expect_err("3 out");
        assert_eq!(err.identifier(), Some("RunMat:lastwarn:TooManyOutputs"));
        // Through the dispatcher with no output count: the message only.
        assert_eq!(
            call("lastwarn", &[]).expect("lastwarn"),
            Value::CharArray(CharArray::new(Vec::new(), 0, 0).unwrap())
        );
    }

    #[test]
    fn registration_discoverable_by_name() {
        for name in ["MException", "throw", "addCause", "lastwarn"] {
            assert!(
                runmat_builtins::builtin_function_by_name(name).is_some(),
                "{name}"
            );
        }
    }
}
