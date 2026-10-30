//! Clean-room `validatestring` (match text against a list of valid options).
//!
//! Clean-room feature `specs/045-text-conversion/`. Normative behaviour comes
//! only from the approved export `validatestring` 0.2.0 (see that feature's
//! `provenance.md`). Errors use `RunMat:validatestring:<observed suffix>`
//! (decision D-045-1, the `validateattributes` convention); funcName and
//! varName change only the message.

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{CharArray, Value};

use crate::builtins::common::map_control_flow_with_builtin;
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const NAME: &str = "validatestring";

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

const TXT_PARAM: BuiltinParamDescriptor = param(
    "txt",
    BuiltinParamArity::Required,
    "Text to match (char row vector or string scalar).",
);
const LIST_PARAM: BuiltinParamDescriptor = param(
    "validStrings",
    BuiltinParamArity::Required,
    "Cell array of char row vectors, or a string array, of valid options.",
);
const FUNC_NAME_PARAM: BuiltinParamDescriptor = param(
    "funcName",
    BuiltinParamArity::Required,
    "Function name shown in error messages.",
);
const VAR_NAME_PARAM: BuiltinParamDescriptor = param(
    "varName",
    BuiltinParamArity::Required,
    "Variable name shown in error messages.",
);

const INPUTS_2: [BuiltinParamDescriptor; 2] = [TXT_PARAM, LIST_PARAM];
const INPUTS_4: [BuiltinParamDescriptor; 4] =
    [TXT_PARAM, LIST_PARAM, FUNC_NAME_PARAM, VAR_NAME_PARAM];

const OUTPUT: [BuiltinParamDescriptor; 1] = [param(
    "str",
    BuiltinParamArity::Required,
    "The matched option as listed: char for a cell list, string for a string-array list.",
)];

const VALIDATESTRING_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "str = validatestring(txt, validStrings)",
        inputs: &INPUTS_2,
        outputs: &OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "str = validatestring(txt, validStrings, funcName, varName)",
        inputs: &INPUTS_4,
        outputs: &OUTPUT,
    },
];

const fn error(
    code: &'static str,
    identifier: &'static str,
    when: &'static str,
) -> BuiltinErrorDescriptor {
    BuiltinErrorDescriptor {
        code,
        identifier: Some(identifier),
        when,
        message: when,
    }
}

const VALIDATESTRING_ERRORS: &[BuiltinErrorDescriptor] = &[
    error(
        "RM.VALIDATESTRING.AMBIGUOUS",
        "RunMat:validatestring:ambiguousStringChoice",
        "The text is a prefix of several options and none of them is a prefix of all the others.",
    ),
    error(
        "RM.VALIDATESTRING.UNRECOGNIZED",
        "RunMat:validatestring:unrecognizedStringChoice",
        "No option matches, or the text is empty or not a char row / string scalar.",
    ),
    error(
        "RM.VALIDATESTRING.INVALID_STRING_LIST",
        "RunMat:validatestring:invalidStringList",
        "The list is neither a cell array of char row vectors nor a string array.",
    ),
    error(
        "RM.VALIDATESTRING.NOT_ENOUGH_INPUTS",
        "RunMat:validatestring:NotEnoughInputs",
        "Fewer than two arguments are given.",
    ),
    error(
        "RM.VALIDATESTRING.UNSUPPORTED_OPTION",
        "RunMat:validatestring:UnsupportedOption",
        "Unobserved forms: more than four arguments, non-text funcName or varName.",
    ),
];

pub const VALIDATESTRING_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &VALIDATESTRING_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: VALIDATESTRING_ERRORS,
};

fn vs_error(suffix: &str, detail: impl Into<String>) -> RuntimeError {
    build_runtime_error(format!("{NAME}: {}", detail.into()))
        .with_builtin(NAME)
        .with_identifier(format!("RunMat:validatestring:{suffix}"))
        .build()
}

// ---------------------------------------------------------------------------
// Builtin
// ---------------------------------------------------------------------------

/// Class of the result, decided by the list (validatestring.output-type).
#[derive(Clone, Copy, PartialEq)]
enum ListKind {
    Char,
    String,
}

#[runtime_builtin(
    name = "validatestring",
    category = "strings/core",
    summary = "Match text against a list of valid options (case-insensitive, unique prefixes allowed) and return the option.",
    keywords = "validatestring,validate,option,choice,prefix,match,argument validation",
    accel = "cpu",
    descriptor(crate::builtins::strings::core::validatestring::VALIDATESTRING_DESCRIPTOR),
    builtin_path = "crate::builtins::strings::core::validatestring"
)]
async fn validatestring_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    if args.len() < 2 {
        return Err(vs_error(
            "NotEnoughInputs",
            "expected validatestring(txt, validStrings)",
        ));
    }
    // validatestring.q-more-args
    if args.len() > 4 {
        return Err(vs_error(
            "UnsupportedOption",
            "more than four input arguments are not yet supported",
        ));
    }
    let mut host = Vec::with_capacity(args.len());
    for value in &args {
        host.push(
            gather_if_needed_async(value)
                .await
                .map_err(|err| map_control_flow_with_builtin(err, NAME))?,
        );
    }
    let context = message_context(&host[2..])?;
    let (kind, options) = parse_list(&host[1])?;
    let unrecognized = |shown: &str| {
        vs_error(
            "unrecognizedStringChoice",
            format!(
                "{context} {shown}does not match any valid option: {}",
                options.join(", ")
            ),
        )
    };
    // validatestring.error-nomatch: empty or non-text input.
    let Some(text) = text_scalar(&host[0]) else {
        return Err(unrecognized(""));
    };
    if text.is_empty() {
        return Err(unrecognized(""));
    }
    let folded = text.to_lowercase();
    let matches: Vec<(&String, String)> = options
        .iter()
        .map(|option| (option, option.to_lowercase()))
        .filter(|(_, lower)| lower.starts_with(&folded))
        .collect();
    // First shortest match; ties keep list order (FR-045-27).
    let Some(best) = matches
        .iter()
        .min_by_key(|(_, lower)| lower.chars().count())
    else {
        return Err(unrecognized(&format!("'{text}' ")));
    };
    // validatestring.nested-options / validatestring.error-ambiguous
    if !matches.iter().all(|(_, lower)| lower.starts_with(&best.1)) {
        let candidates: Vec<&str> = matches.iter().map(|(o, _)| o.as_str()).collect();
        return Err(vs_error(
            "ambiguousStringChoice",
            format!(
                "{context} '{text}' matches more than one valid option: {}",
                candidates.join(", ")
            ),
        ));
    }
    let chosen = best.0.clone();
    Ok(match kind {
        ListKind::Char => Value::CharArray(CharArray::new_row(&chosen)),
        ListKind::String => Value::String(chosen),
    })
}

fn text_scalar(value: &Value) -> Option<String> {
    match value {
        Value::CharArray(chars) if chars.rows <= 1 => Some(chars.data.iter().collect()),
        Value::String(text) => Some(text.clone()),
        Value::StringArray(array) if array.data.len() == 1 => Some(array.data[0].clone()),
        _ => None,
    }
}

/// The options and result class (validatestring.output-type,
/// validatestring.error-cell-of-strings, FR-045-27).
fn parse_list(value: &Value) -> BuiltinResult<(ListKind, Vec<String>)> {
    let invalid = || {
        vs_error(
            "invalidStringList",
            "validStrings must be a cell array of char row vectors or a string array",
        )
    };
    match value {
        Value::Cell(cell) => {
            let mut options = Vec::with_capacity(cell.data.len());
            for element in &cell.data {
                match element {
                    Value::CharArray(chars) if chars.rows <= 1 => {
                        options.push(chars.data.iter().collect())
                    }
                    _ => return Err(invalid()),
                }
            }
            Ok((ListKind::Char, options))
        }
        Value::String(text) => Ok((ListKind::String, vec![text.clone()])),
        Value::StringArray(array) => Ok((ListKind::String, array.data.clone())),
        _ => Err(invalid()),
    }
}

/// Message prefix from funcName and varName (message text only, D-045-1).
fn message_context(trailing: &[Value]) -> BuiltinResult<String> {
    let bad = |what: &str| {
        vs_error(
            "UnsupportedOption",
            format!("this form of the {what} argument is not yet supported"),
        )
    };
    let function = match trailing.first() {
        Some(value) => text_scalar(value).ok_or_else(|| bad("funcName"))?,
        None => String::new(),
    };
    let variable = match trailing.get(1) {
        Some(value) => text_scalar(value).ok_or_else(|| bad("varName"))?,
        None => String::new(),
    };
    let mut context = if variable.is_empty() {
        "input".to_string()
    } else {
        format!("input '{variable}'")
    };
    if !function.is_empty() {
        context.push_str(&format!(" to {function}"));
    }
    Ok(context)
}

#[cfg(test)]
mod validatestring_tests {
    use crate::BuiltinResult;
    use futures::executor::block_on;
    use runmat_value::{CellArray, CharArray, StringArray, Value};

    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("validatestring", args))
    }

    fn chars(s: &str) -> Value {
        Value::CharArray(CharArray::new_row(s))
    }

    fn string(s: &str) -> Value {
        Value::String(s.into())
    }

    fn cellstr(items: &[&str]) -> Value {
        let data: Vec<Value> = items.iter().map(|s| chars(s)).collect();
        Value::Cell(CellArray::new(data, 1, items.len()).unwrap())
    }

    fn string_row(items: &[&str]) -> Value {
        Value::StringArray(
            StringArray::new(
                items.iter().map(|s| s.to_string()).collect(),
                vec![1, items.len()],
            )
            .unwrap(),
        )
    }

    fn fruit() -> Value {
        cellstr(&["apple", "banana"])
    }

    fn assert_char(value: Value, expected: &str) {
        match value {
            Value::CharArray(ca) => {
                assert_eq!((ca.rows, ca.cols), (1, expected.chars().count()));
                assert_eq!(ca.data.iter().collect::<String>(), expected);
            }
            other => panic!("expected char {expected:?}, got {other:?}"),
        }
    }

    fn assert_string(value: Value, expected: &str) {
        match value {
            Value::String(s) => assert_eq!(s, expected),
            Value::StringArray(sa) if sa.data.len() == 1 => {
                assert_eq!(sa.shape, vec![1, 1]);
                assert_eq!(sa.data[0], expected);
            }
            other => panic!("expected string {expected:?}, got {other:?}"),
        }
    }

    fn error(result: BuiltinResult<Value>) -> (String, String) {
        match result {
            Ok(value) => panic!("expected an error, got {value:?}"),
            Err(err) => (
                err.identifier().unwrap_or("<none>").to_string(),
                err.message().to_string(),
            ),
        }
    }

    fn identifier(result: BuiltinResult<Value>) -> String {
        error(result).0
    }

    const AMBIGUOUS: &str = "RunMat:validatestring:ambiguousStringChoice";
    const UNRECOGNIZED: &str = "RunMat:validatestring:unrecognizedStringChoice";
    const INVALID_LIST: &str = "RunMat:validatestring:invalidStringList";
    const UNSUPPORTED: &str = "RunMat:validatestring:UnsupportedOption";

    // --- Observed (normative): one test per observed case --------------------

    /// case `exact` — validatestring.signature-primary, validatestring.exact,
    /// validatestring.output-type
    #[test]
    fn observed_exact() {
        assert_char(call(&[chars("apple"), fruit()]).unwrap(), "apple");
    }

    /// case `prefix` — validatestring.prefix
    #[test]
    fn observed_prefix() {
        assert_char(call(&[chars("ban"), fruit()]).unwrap(), "banana");
    }

    /// case `case` — validatestring.case-insensitive
    #[test]
    fn observed_case_insensitive() {
        assert_char(call(&[chars("APP"), fruit()]).unwrap(), "apple");
    }

    /// case `err-cell-of-strings` — validatestring.error-cell-of-strings
    #[test]
    fn observed_cell_of_strings_errors() {
        let list =
            Value::Cell(CellArray::new(vec![string("apple"), string("banana")], 1, 2).unwrap());
        assert_eq!(identifier(call(&[string("ban"), list])), INVALID_LIST);
    }

    /// case `string-input` — validatestring.output-type
    #[test]
    fn observed_string_input_with_cellstr_list() {
        assert_char(call(&[string("ban"), fruit()]).unwrap(), "banana");
    }

    /// case `string-list` — validatestring.output-type
    #[test]
    fn observed_string_list() {
        assert_string(
            call(&[string("ap"), string_row(&["apple", "banana"])]).unwrap(),
            "apple",
        );
    }

    /// case `exact-over-prefix` — validatestring.nested-options
    #[test]
    fn observed_exact_over_prefix() {
        assert_char(
            call(&[chars("app"), cellstr(&["app", "apple"])]).unwrap(),
            "app",
        );
    }

    /// case `prefix-of-several-nested` — validatestring.nested-options
    #[test]
    fn observed_prefix_of_nested_options() {
        assert_char(
            call(&[chars("ap"), cellstr(&["app", "apple"])]).unwrap(),
            "app",
        );
    }

    /// case `err-ambiguous` — validatestring.error-ambiguous
    #[test]
    fn observed_ambiguous_errors() {
        assert_eq!(
            identifier(call(&[chars("b"), cellstr(&["banana", "berry"])])),
            AMBIGUOUS
        );
    }

    /// case `err-nomatch` — validatestring.error-nomatch
    #[test]
    fn observed_no_match_errors() {
        assert_eq!(identifier(call(&[chars("kiwi"), fruit()])), UNRECOGNIZED);
    }

    /// case `err-funcname` — validatestring.error-funcname (identifier per D-045-1)
    #[test]
    fn observed_funcname_keeps_runmat_identifier() {
        let (id, message) = error(call(&[
            chars("kiwi"),
            fruit(),
            chars("myfun"),
            chars("fruit"),
        ]));
        assert_eq!(id, UNRECOGNIZED);
        assert!(!id.starts_with("MATLAB:"));
        assert!(message.contains("myfun"), "{message}");
        assert!(message.contains("fruit"), "{message}");
    }

    /// case `err-nonchar` — validatestring.error-nomatch
    #[test]
    fn observed_numeric_input_errors() {
        assert_eq!(identifier(call(&[Value::Num(5.0), fruit()])), UNRECOGNIZED);
    }

    /// case `err-empty-input` — validatestring.error-nomatch
    #[test]
    fn observed_empty_input_errors() {
        assert_eq!(identifier(call(&[chars(""), fruit()])), UNRECOGNIZED);
    }

    // --- Documented choices (FR-045-25..27) and unresolved points ------------

    /// FR-045-25
    #[test]
    fn documented_choice_arity_and_names() {
        assert_eq!(
            identifier(call(&[chars("a")])),
            "RunMat:validatestring:NotEnoughInputs"
        );
        // funcName alone (3 arguments) is accepted.
        assert_char(
            call(&[chars("ban"), fruit(), chars("myfun")]).unwrap(),
            "banana",
        );
        let (id, message) = error(call(&[chars("kiwi"), fruit(), string("myfun")]));
        assert_eq!(id, UNRECOGNIZED);
        assert!(message.contains("myfun"));
        assert_eq!(
            identifier(call(&[chars("ban"), fruit(), Value::Num(1.0)])),
            UNSUPPORTED
        );
        assert_eq!(
            identifier(call(&[chars("ban"), fruit(), chars("f"), Value::Num(1.0)])),
            UNSUPPORTED
        );
    }

    /// FR-045-26 (unresolved, validatestring.q-more-args)
    #[test]
    fn unresolved_more_than_four_arguments_are_unsupported() {
        assert_eq!(
            identifier(call(&[
                chars("ban"),
                fruit(),
                chars("f"),
                chars("v"),
                Value::Num(2.0)
            ])),
            UNSUPPORTED
        );
    }

    /// FR-045-27
    #[test]
    fn documented_choice_lists_text_and_ties() {
        // List checked before text.
        let bad_list = Value::Cell(CellArray::new(vec![Value::Num(1.0)], 1, 1).unwrap());
        assert_eq!(identifier(call(&[Value::Num(5.0), bad_list])), INVALID_LIST);
        assert_eq!(
            identifier(call(&[chars("a"), chars("apple")])),
            INVALID_LIST
        );
        assert_eq!(
            identifier(call(&[chars("a"), Value::Num(1.0)])),
            INVALID_LIST
        );
        let mixed =
            Value::Cell(CellArray::new(vec![chars("apple"), string("berry")], 1, 2).unwrap());
        assert_eq!(identifier(call(&[chars("a"), mixed])), INVALID_LIST);
        // Empty lists match nothing.
        let empty_cell = Value::Cell(CellArray::new(Vec::new(), 0, 0).unwrap());
        assert_eq!(identifier(call(&[chars("a"), empty_cell])), UNRECOGNIZED);
        let empty_strings = Value::StringArray(StringArray::new(Vec::new(), vec![0, 0]).unwrap());
        assert_eq!(identifier(call(&[chars("a"), empty_strings])), UNRECOGNIZED);
        // Non-scalar text is unrecognised.
        let block = Value::CharArray(CharArray::new("abcd".chars().collect(), 2, 2).unwrap());
        assert_eq!(identifier(call(&[block, fruit()])), UNRECOGNIZED);
        assert_eq!(
            identifier(call(&[string_row(&["a", "b"]), fruit()])),
            UNRECOGNIZED
        );
        assert_eq!(
            identifier(call(&[cellstr(&["apple"]), fruit()])),
            UNRECOGNIZED
        );
        // Ties: first listed wins; nested chains of three resolve to the shortest.
        assert_char(
            call(&[chars("app"), cellstr(&["App", "app"])]).unwrap(),
            "App",
        );
        assert_char(
            call(&[chars("ap"), cellstr(&["apply", "app", "apple"])]).unwrap(),
            "app",
        );
        assert_eq!(
            identifier(call(&[chars("ap"), cellstr(&["app", "apple", "apricot"])])),
            AMBIGUOUS
        );
        // String scalar list behaves like a one-element string array.
        assert_string(call(&[chars("b"), string("banana")]).unwrap(), "banana");
    }

    /// FR-045-28 (unresolved, validatestring.q-warning-coverage)
    #[test]
    fn unresolved_no_warning_emitted() {
        crate::warning_store::reset();
        let _ = call(&[chars("ban"), fruit()]).unwrap();
        assert!(crate::warning_store::take_all().is_empty());
    }
}
