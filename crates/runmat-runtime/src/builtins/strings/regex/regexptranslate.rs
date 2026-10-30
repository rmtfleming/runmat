//! Clean-room `regexptranslate` (escape text or translate wildcards into a
//! regular-expression pattern).
//!
//! Clean-room feature `specs/045-text-conversion/`. Normative behaviour comes
//! only from the approved export `regexptranslate` 0.2.0 (see that feature's
//! `provenance.md`). Observed `MATLAB:x:y` identifiers are raised as
//! `RunMat:x:y` (decision D-045-1); unobserved points raise
//! `RunMat:regexptranslate:UnsupportedOption` or are documented choices.

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{CellArray, CharArray, StringArray, Value};

use crate::builtins::common::map_control_flow_with_builtin;
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const NAME: &str = "regexptranslate";

// ---------------------------------------------------------------------------
// Descriptor
// ---------------------------------------------------------------------------

const REGEXPTRANSLATE_INPUTS: [BuiltinParamDescriptor; 2] = [
    BuiltinParamDescriptor {
        name: "op",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "'escape' or 'wildcard'.",
    },
    BuiltinParamDescriptor {
        name: "s",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Char row vector, string array, or cell array of char row vectors.",
    },
];

const REGEXPTRANSLATE_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "p",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Translated text of the same kind as s (char, string or cell).",
}];

const REGEXPTRANSLATE_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "p = regexptranslate(op, s)",
    inputs: &REGEXPTRANSLATE_INPUTS,
    outputs: &REGEXPTRANSLATE_OUTPUT,
}];

const INVALID_OPTIONS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.REGEXPTRANSLATE.INVALID_OPTIONS",
    identifier: Some("RunMat:REGEXP:invalidOptions"),
    when: "op is text that is not a recognised operation.",
    message: "regexptranslate: unrecognised operation",
};

const NOT_ENOUGH_INPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.REGEXPTRANSLATE.NOT_ENOUGH_INPUTS",
    identifier: Some("RunMat:regexptranslate:NotEnoughInputs"),
    when: "Fewer than two arguments are given.",
    message: "regexptranslate: not enough input arguments",
};

const TOO_MANY_INPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.REGEXPTRANSLATE.TOO_MANY_INPUTS",
    identifier: Some("RunMat:regexptranslate:TooManyInputs"),
    when: "More than two arguments are given.",
    message: "regexptranslate: too many input arguments",
};

const UNSUPPORTED_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.REGEXPTRANSLATE.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:regexptranslate:UnsupportedOption"),
    when: "Unobserved forms: case variants of the operations, non-text op, multi-row char, non-char cell elements, other classes.",
    message: "regexptranslate: this form is not yet specified",
};

const INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.REGEXPTRANSLATE.INTERNAL",
    identifier: Some("RunMat:regexptranslate:Internal"),
    when: "The runtime cannot assemble the result.",
    message: "regexptranslate: internal runtime failure",
};

const REGEXPTRANSLATE_ERRORS: [BuiltinErrorDescriptor; 5] = [
    INVALID_OPTIONS,
    NOT_ENOUGH_INPUTS,
    TOO_MANY_INPUTS,
    UNSUPPORTED_OPTION,
    INTERNAL,
];

pub const REGEXPTRANSLATE_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &REGEXPTRANSLATE_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &REGEXPTRANSLATE_ERRORS,
};

fn rt_error(
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

#[derive(Clone, Copy)]
enum Operation {
    Escape,
    Wildcard,
}

#[runtime_builtin(
    name = "regexptranslate",
    category = "strings/regex",
    summary = "Escape regular-expression special characters or translate wildcards into a pattern.",
    keywords = "regexptranslate,regexp,escape,wildcard,pattern",
    accel = "cpu",
    descriptor(crate::builtins::strings::regex::regexptranslate::REGEXPTRANSLATE_DESCRIPTOR),
    builtin_path = "crate::builtins::strings::regex::regexptranslate"
)]
async fn regexptranslate_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    if args.len() < 2 {
        return Err(rt_error(
            &NOT_ENOUGH_INPUTS,
            "expected regexptranslate(op, s)",
        ));
    }
    if args.len() > 2 {
        return Err(rt_error(
            &TOO_MANY_INPUTS,
            "expected regexptranslate(op, s)",
        ));
    }
    let mut host = Vec::with_capacity(2);
    for value in &args {
        host.push(
            gather_if_needed_async(value)
                .await
                .map_err(|err| map_control_flow_with_builtin(err, NAME))?,
        );
    }
    let op = parse_operation(&host[0])?;
    translate_value(op, &host[1])
}

fn text_scalar(value: &Value) -> Option<String> {
    match value {
        Value::CharArray(chars) if chars.rows <= 1 => Some(chars.data.iter().collect()),
        Value::String(text) => Some(text.clone()),
        Value::StringArray(array) if array.data.len() == 1 => Some(array.data[0].clone()),
        _ => None,
    }
}

fn parse_operation(value: &Value) -> BuiltinResult<Operation> {
    let Some(op) = text_scalar(value) else {
        return Err(rt_error(
            &UNSUPPORTED_OPTION,
            "op must be a char row vector or string scalar",
        ));
    };
    match op.as_str() {
        "escape" => Ok(Operation::Escape),
        "wildcard" => Ok(Operation::Wildcard),
        // regexptranslate.q-other-ops: case variants are not specified.
        other if other.eq_ignore_ascii_case("escape") || other.eq_ignore_ascii_case("wildcard") => {
            Err(rt_error(
                &UNSUPPORTED_OPTION,
                format!("operation '{other}' (case variant) is not yet specified"),
            ))
        }
        // regexptranslate.error-op
        other => Err(rt_error(
            &INVALID_OPTIONS,
            format!("'{other}' is not a recognised operation"),
        )),
    }
}

/// Characters that 'escape' prefixes with a backslash (regexptranslate.escape).
const ESCAPED: &str = ".$^*+?()[]{}|\\-";

fn translate_text(op: Operation, text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 2);
    for ch in text.chars() {
        match op {
            Operation::Escape => {
                if ESCAPED.contains(ch) {
                    out.push('\\');
                }
                out.push(ch);
            }
            // regexptranslate.wildcard; other characters are copied (FR-045-19).
            Operation::Wildcard => match ch {
                '*' => out.push_str(".*"),
                '?' => out.push('.'),
                '.' => out.push_str("\\."),
                other => out.push(other),
            },
        }
    }
    out
}

/// Char result; empty text gives 0x0 (regexptranslate.types, FR-045-20).
fn char_value(text: &str) -> Value {
    if text.is_empty() {
        Value::CharArray(CharArray::new(Vec::new(), 0, 0).expect("0x0 char"))
    } else {
        Value::CharArray(CharArray::new_row(text))
    }
}

fn translate_value(op: Operation, value: &Value) -> BuiltinResult<Value> {
    let unsupported = |what: &str| {
        rt_error(
            &UNSUPPORTED_OPTION,
            format!("{what} input is not yet specified"),
        )
    };
    match value {
        Value::CharArray(chars) if chars.rows <= 1 => {
            let text: String = chars.data.iter().collect();
            Ok(char_value(&translate_text(op, &text)))
        }
        Value::CharArray(_) => Err(unsupported("multi-row char")),
        Value::String(text) => Ok(Value::String(translate_text(op, text))),
        Value::StringArray(array) => {
            let data = array
                .data
                .iter()
                .map(|text| translate_text(op, text))
                .collect();
            StringArray::new(data, array.shape.clone())
                .map(Value::StringArray)
                .map_err(|err| rt_error(&INTERNAL, err))
        }
        Value::Cell(cell) => {
            let mut data = Vec::with_capacity(cell.data.len());
            for element in &cell.data {
                match element {
                    Value::CharArray(chars) if chars.rows <= 1 => {
                        let text: String = chars.data.iter().collect();
                        data.push(char_value(&translate_text(op, &text)));
                    }
                    _ => return Err(unsupported("a cell element that is not a char row vector")),
                }
            }
            CellArray::new_with_shape(data, cell.shape.clone())
                .map(Value::Cell)
                .map_err(|err| rt_error(&INTERNAL, err))
        }
        other => Err(unsupported(&format!(
            "class {}",
            crate::builtins::introspection::class::class_name_for_value(other)
        ))),
    }
}

#[cfg(test)]
mod regexptranslate_tests {
    use crate::BuiltinResult;
    use futures::executor::block_on;
    use runmat_value::{CellArray, CharArray, StringArray, Value};

    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("regexptranslate", args))
    }

    fn chars(s: &str) -> Value {
        Value::CharArray(CharArray::new_row(s))
    }

    fn empty_char() -> Value {
        Value::CharArray(CharArray::new(Vec::new(), 0, 0).unwrap())
    }

    fn translate(op: &str, s: &str) -> Value {
        call(&[chars(op), chars(s)]).expect("regexptranslate")
    }

    /// (rows, cols, text) of a char result.
    fn char_parts(value: &Value) -> (usize, usize, String) {
        match value {
            Value::CharArray(ca) => (ca.rows, ca.cols, ca.data.iter().collect()),
            other => panic!("expected char, got {other:?}"),
        }
    }

    fn assert_char_row(value: &Value, expected: &str) {
        let (rows, cols, text) = char_parts(value);
        assert_eq!(text, expected);
        assert_eq!((rows, cols), (1, expected.chars().count()));
    }

    fn identifier(result: BuiltinResult<Value>) -> String {
        match result {
            Ok(value) => panic!("expected an error, got {value:?}"),
            Err(err) => err.identifier().unwrap_or("<none>").to_string(),
        }
    }

    const UNSUPPORTED: &str = "RunMat:regexptranslate:UnsupportedOption";

    // --- Observed (normative): one test per observed case --------------------

    /// case `escape-dot` — regexptranslate.signature-primary, regexptranslate.escape
    #[test]
    fn observed_escape_dot() {
        assert_char_row(&translate("escape", "a.b"), "a\\.b");
    }

    /// case `escape-specials` — regexptranslate.escape
    #[test]
    fn observed_escape_specials() {
        let out = translate("escape", "$^*+?()[]{}|\\");
        assert_char_row(&out, "\\$\\^\\*\\+\\?\\(\\)\\[\\]\\{\\}\\|\\\\");
        assert_eq!(char_parts(&out).1, 26);
    }

    /// case `escape-plain` — regexptranslate.escape
    #[test]
    fn observed_escape_plain() {
        assert_char_row(&translate("escape", "abc"), "abc");
    }

    /// case `escape-others` — regexptranslate.escape
    #[test]
    fn observed_escape_others() {
        let out = translate("escape", "-<>#/ ,:=!@");
        assert_char_row(&out, "\\-<>#/ ,:=!@");
        assert_eq!(char_parts(&out).1, 12);
    }

    /// case `escape-whitespace` — regexptranslate.escape-whitespace
    #[test]
    fn observed_escape_whitespace() {
        assert_char_row(&translate("escape", "a\tb\nc"), "a\tb\nc");
    }

    /// case `wildcard-star` — regexptranslate.wildcard
    #[test]
    fn observed_wildcard_star() {
        assert_char_row(&translate("wildcard", "*.m"), ".*\\.m");
    }

    /// case `wildcard-question` — regexptranslate.wildcard
    #[test]
    fn observed_wildcard_question() {
        assert_char_row(&translate("wildcard", "a?c"), "a.c");
    }

    /// case `wildcard-dot` — regexptranslate.wildcard
    #[test]
    fn observed_wildcard_dot() {
        assert_char_row(&translate("wildcard", "a.b"), "a\\.b");
    }

    /// case `wildcard-use` — regexptranslate.wildcard-use
    #[test]
    fn observed_wildcard_use() {
        let (_, _, pattern) = char_parts(&translate("wildcard", "*.m"));
        let anchored = format!("^{pattern}$");
        let hit = block_on(crate::call_builtin_async(
            "regexp",
            &[chars("file.m"), chars(&anchored), chars("once")],
        ))
        .expect("regexp");
        let found = match hit {
            Value::Num(n) => n == 1.0,
            Value::Tensor(t) => !t.is_empty(),
            other => panic!("unexpected regexp result {other:?}"),
        };
        assert!(found, "anchored wildcard pattern must match 'file.m'");
        let miss = block_on(crate::call_builtin_async(
            "regexp",
            &[chars("file.c"), chars(&anchored), chars("once")],
        ))
        .expect("regexp");
        assert!(
            matches!(&miss, Value::Tensor(t) if t.is_empty()),
            "'file.c' must not match, got {miss:?}"
        );
    }

    /// case `string-input` — regexptranslate.types
    #[test]
    fn observed_string_input() {
        let out = call(&[Value::String("escape".into()), Value::String("a.b".into())])
            .expect("regexptranslate");
        match out {
            Value::String(s) => assert_eq!(s, "a\\.b"),
            Value::StringArray(sa) => {
                assert_eq!(sa.shape, vec![1, 1]);
                assert_eq!(sa.data, vec!["a\\.b".to_string()]);
            }
            other => panic!("expected a string scalar, got {other:?}"),
        }
    }

    /// case `cell-input` — regexptranslate.types
    #[test]
    fn observed_cell_input() {
        let input = Value::Cell(CellArray::new(vec![chars("a.b"), chars("c+d")], 1, 2).unwrap());
        let out = call(&[chars("escape"), input]).expect("regexptranslate");
        let Value::Cell(cell) = out else {
            panic!("expected a cell, got {out:?}");
        };
        assert_eq!(cell.shape, vec![1, 2]);
        assert_char_row(&cell.data[0], "a\\.b");
        assert_char_row(&cell.data[1], "c\\+d");
    }

    /// case `empty` — regexptranslate.types
    #[test]
    fn observed_empty() {
        let out = call(&[chars("escape"), empty_char()]).expect("regexptranslate");
        assert_eq!(char_parts(&out), (0, 0, String::new()));
    }

    /// case `err-op` — regexptranslate.error-op
    #[test]
    fn observed_unknown_operation_errors() {
        assert_eq!(
            identifier(call(&[chars("nope"), chars("a")])),
            "RunMat:REGEXP:invalidOptions"
        );
    }

    // --- Documented choices (FR-045-18..20) and unresolved points ------------

    /// FR-045-18
    #[test]
    fn documented_choice_arity_and_op_forms() {
        assert_eq!(
            identifier(call(&[chars("escape")])),
            "RunMat:regexptranslate:NotEnoughInputs"
        );
        assert_eq!(
            identifier(call(&[chars("escape"), chars("a"), chars("b")])),
            "RunMat:regexptranslate:TooManyInputs"
        );
        assert_eq!(
            identifier(call(&[chars("other"), chars("a")])),
            "RunMat:REGEXP:invalidOptions"
        );
        assert_eq!(
            identifier(call(&[Value::Num(1.0), chars("a")])),
            UNSUPPORTED
        );
        // A one-element string array counts as a string scalar op.
        let op = Value::StringArray(StringArray::new(vec!["escape".into()], vec![1, 1]).unwrap());
        assert_char_row(&call(&[op, chars("a.b")]).unwrap(), "a\\.b");
    }

    /// FR-045-18 (unresolved, regexptranslate.q-other-ops)
    #[test]
    fn unresolved_case_variant_ops_are_unsupported() {
        for op in ["ESCAPE", "Wildcard", "Escape"] {
            assert_eq!(
                identifier(call(&[chars(op), chars("a")])),
                UNSUPPORTED,
                "{op}"
            );
        }
    }

    /// FR-045-19
    #[test]
    fn documented_choice_unlisted_characters_are_copied() {
        assert_char_row(&translate("escape", "09_~%'\";&é"), "09_~%'\";&é");
        assert_char_row(&translate("wildcard", "a+b(c)$"), "a+b(c)$");
        assert_char_row(&translate("wildcard", "**"), ".*.*");
    }

    /// FR-045-20
    #[test]
    fn documented_choice_shapes_and_unsupported_inputs() {
        // 1x0 char gives 0x0 char.
        let row0 = Value::CharArray(CharArray::new(Vec::new(), 1, 0).unwrap());
        assert_eq!(
            char_parts(&call(&[chars("escape"), row0]).unwrap()),
            (0, 0, String::new())
        );
        // Empty cell element gives 0x0 char; cell shape kept.
        let input = Value::Cell(CellArray::new(vec![empty_char(), chars("x.")], 2, 1).unwrap());
        let Value::Cell(cell) = call(&[chars("wildcard"), input]).unwrap() else {
            panic!("expected cell");
        };
        assert_eq!(cell.shape, vec![2, 1]);
        assert_eq!(char_parts(&cell.data[0]), (0, 0, String::new()));
        assert_char_row(&cell.data[1], "x\\.");
        // String array element-wise with shape kept.
        let strings = Value::StringArray(
            StringArray::new(vec!["a.".into(), "*".into()], vec![2, 1]).unwrap(),
        );
        let Value::StringArray(sa) = call(&[chars("wildcard"), strings]).unwrap() else {
            panic!("expected string array");
        };
        assert_eq!(sa.shape, vec![2, 1]);
        assert_eq!(sa.data, vec!["a\\.".to_string(), ".*".to_string()]);
        // Unsupported subjects.
        let block = Value::CharArray(CharArray::new("abcd".chars().collect(), 2, 2).unwrap());
        assert_eq!(identifier(call(&[chars("escape"), block])), UNSUPPORTED);
        let bad_cell = Value::Cell(CellArray::new(vec![Value::Num(1.0)], 1, 1).unwrap());
        assert_eq!(identifier(call(&[chars("escape"), bad_cell])), UNSUPPORTED);
        assert_eq!(
            identifier(call(&[chars("escape"), Value::Num(3.0)])),
            UNSUPPORTED
        );
    }

    /// FR-045-28 (unresolved, regexptranslate.q-warning-coverage)
    #[test]
    fn unresolved_no_warning_emitted() {
        crate::warning_store::reset();
        let _ = translate("escape", "a.b");
        assert!(crate::warning_store::take_all().is_empty());
    }
}
