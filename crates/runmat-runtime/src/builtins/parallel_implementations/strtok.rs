// Parallel clean-room implementation of `strtok` (independently developed against the
// matlab-interface-spec black-box specs). 0-6-0 ships the DEFAULT implementation; this copy is
// retained unregistered for differential cross-validation. Original path: crates/runmat-runtime/src/builtins/strings/search/strtok.rs
//! MATLAB-compatible `strtok` builtin for RunMat.
//!
//! Clean-room provenance: specs/006-string-search (spec `strtok` 0.2.0,
//! claims strtok.signature-primary, strtok.output-class,
//! strtok.leading-token; the second output (remainder) is summary-derived
//! and its delimiter handling is a documented independent choice under
//! strtok.q-remainder).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    CharArray, StringArray, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ShapeRequirements,
};
use crate::{build_runtime_error, BuiltinResult, RuntimeError};

pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "strtok",
    op_kind: GpuOpKind::Custom("string-search"),
    supported_precisions: &[],
    broadcast: BroadcastSemantics::None,
    provider_hooks: &[],
    constant_strategy: ConstantStrategy::InlineLiteral,
    residency: ResidencyPolicy::GatherImmediately,
    nan_mode: ReductionNaN::Include,
    two_pass_threshold: None,
    workgroup_size: None,
    accepts_nan_mode: false,
    notes: "Pure text manipulation; no GPU execution path.",
};

pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "strtok",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Not fusible; text manipulation only.",
};

const BUILTIN_NAME: &str = "strtok";

const STRTOK_OUTPUTS: [BuiltinParamDescriptor; 2] = [
    BuiltinParamDescriptor {
        name: "token",
        ty: BuiltinParamType::StringScalar,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "First token after skipping leading delimiters.",
    },
    BuiltinParamDescriptor {
        name: "remain",
        ty: BuiltinParamType::StringScalar,
        arity: BuiltinParamArity::Optional,
        default: None,
        description: "Remainder of the text, starting at the delimiter that ended the token.",
    },
];

const STRTOK_INPUTS_BASE: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "str",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Input text (char row vector or string scalar).",
}];

const STRTOK_INPUTS_DELIM: [BuiltinParamDescriptor; 2] = [
    BuiltinParamDescriptor {
        name: "str",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Input text (char row vector or string scalar).",
    },
    BuiltinParamDescriptor {
        name: "delimiters",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Optional,
        default: None,
        description: "Set of delimiter characters (char row vector or string scalar).",
    },
];

const STRTOK_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "[token, remain] = strtok(str)",
        inputs: &STRTOK_INPUTS_BASE,
        outputs: &STRTOK_OUTPUTS,
    },
    BuiltinSignatureDescriptor {
        label: "[token, remain] = strtok(str, delimiters)",
        inputs: &STRTOK_INPUTS_DELIM,
        outputs: &STRTOK_OUTPUTS,
    },
];

const STRTOK_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.STRTOK.INVALID_INPUT",
    identifier: Some("RunMat:strtok:InvalidInput"),
    when: "The input is not a char row vector or string scalar.",
    message: "strtok: input must be a character vector or string scalar",
};

const STRTOK_ERROR_INVALID_DELIMITER: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.STRTOK.INVALID_DELIMITER",
    identifier: Some("RunMat:strtok:InvalidDelimiter"),
    when: "The delimiter argument is not a char row vector or string scalar.",
    message: "strtok: delimiters must be a character vector or string scalar",
};

const STRTOK_ERRORS: [BuiltinErrorDescriptor; 2] =
    [STRTOK_ERROR_INVALID_INPUT, STRTOK_ERROR_INVALID_DELIMITER];

pub const STRTOK_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &STRTOK_SIGNATURES,
    output_mode: BuiltinOutputMode::ByRequestedOutputCount,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &STRTOK_ERRORS,
};

fn strtok_error(error: &'static BuiltinErrorDescriptor) -> RuntimeError {
    let mut builder = build_runtime_error(error.message).with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

async fn strtok_builtin(text: Value, rest: Vec<Value>) -> BuiltinResult<Value> {
    let (text, as_string) = match &text {
        Value::CharArray(chars) if chars.rows <= 1 => {
            (chars.data.iter().collect::<String>(), false)
        }
        Value::String(text) => (text.clone(), true),
        Value::StringArray(StringArray { data, .. }) if data.len() == 1 => (data[0].clone(), true),
        _ => return Err(strtok_error(&STRTOK_ERROR_INVALID_INPUT)),
    };

    let delimiters = match rest.as_slice() {
        [] => None,
        [value] => Some(extract_delimiters(value)?),
        _ => return Err(strtok_error(&STRTOK_ERROR_INVALID_DELIMITER)),
    };

    let (token, remain) = split_token(&text, delimiters.as_deref());
    let outputs = vec![
        text_value(&token, as_string),
        text_value(&remain, as_string),
    ];
    if let Some(out_count) = crate::output_count::current_output_count() {
        return Ok(crate::output_count::output_list_with_padding(
            out_count, outputs,
        ));
    }
    Ok(outputs.into_iter().next().expect("strtok outputs"))
}

fn extract_delimiters(value: &Value) -> BuiltinResult<Vec<char>> {
    match value {
        Value::CharArray(chars) if chars.rows <= 1 => Ok(chars.data.clone()),
        Value::String(text) => Ok(text.chars().collect()),
        Value::StringArray(StringArray { data, .. }) if data.len() == 1 => {
            Ok(data[0].chars().collect())
        }
        _ => Err(strtok_error(&STRTOK_ERROR_INVALID_DELIMITER)),
    }
}

/// Split off the leading token: skip leading delimiter characters, collect
/// token characters up to the next delimiter, and return the rest of the
/// text (starting at that delimiter) as the remainder.
///
/// Choices on points the export leaves open: the default delimiter set is
/// whitespace via Rust's `char::is_whitespace` (observed only for the space
/// character); the remainder includes the delimiter that terminated the
/// token (strtok.q-remainder); when the input is empty or all delimiters,
/// both outputs are empty (strtok.q-empty).
fn split_token(input: &str, delimiters: Option<&[char]>) -> (String, String) {
    let is_delimiter = |c: char| match delimiters {
        Some(set) => set.contains(&c),
        None => c.is_whitespace(),
    };

    let chars: Vec<char> = input.chars().collect();
    let start = chars
        .iter()
        .position(|&c| !is_delimiter(c))
        .unwrap_or(chars.len());
    let end = chars[start..]
        .iter()
        .position(|&c| is_delimiter(c))
        .map(|offset| start + offset)
        .unwrap_or(chars.len());

    let token = chars[start..end].iter().collect();
    let remain = chars[end..].iter().collect();
    (token, remain)
}

fn text_value(text: &str, as_string: bool) -> Value {
    if as_string {
        Value::String(text.to_string())
    } else if text.is_empty() {
        Value::CharArray(CharArray::new(Vec::new(), 0, 0).expect("empty char"))
    } else {
        Value::CharArray(CharArray::new_row(text))
    }
}

#[cfg(all(test, feature = "parallel_xval"))]
pub(crate) mod tests {
    use super::*;
    use futures::executor::block_on;

    fn run_strtok(text: Value, rest: Vec<Value>) -> Value {
        block_on(super::strtok_builtin(text, rest)).expect("strtok")
    }

    fn run_strtok_char(text: &str, rest: Vec<Value>) -> Value {
        run_strtok(Value::CharArray(CharArray::new_row(text)), rest)
    }

    fn assert_char(value: &Value, expected: &str, rows: usize, cols: usize) {
        match value {
            Value::CharArray(chars) => {
                assert_eq!(chars.data.iter().collect::<String>(), expected);
                assert_eq!((chars.rows, chars.cols), (rows, cols));
            }
            other => panic!("expected char array, got {other:?}"),
        }
    }

    // Normative [FR-006-07; strtok.signature-primary, strtok.output-class,
    // strtok.leading-token] — observed case `default`:
    // strtok('hello world') => 'hello' (char, 1x5).
    #[test]
    fn observed_default_whitespace_token() {
        assert_char(&run_strtok_char("hello world", Vec::new()), "hello", 1, 5);
    }

    // Normative [FR-006-08; strtok.output-class, strtok.leading-token] —
    // observed case `custom-delim`: strtok('a,b,c', ',') => 'a' (char, 1x1).
    #[test]
    fn observed_custom_delimiter_token() {
        assert_char(
            &run_strtok_char("a,b,c", vec![Value::CharArray(CharArray::new_row(","))]),
            "a",
            1,
            1,
        );
    }

    // Normative [FR-006-07; strtok.output-class, strtok.leading-token] —
    // observed case `leading-space`: strtok('   lead trail') => 'lead'
    // (char, 1x4; leading delimiters skipped).
    #[test]
    fn observed_leading_delimiters_skipped() {
        assert_char(&run_strtok_char("   lead trail", Vec::new()), "lead", 1, 4);
    }

    // summary_derived [FR-006-09]: the approved summary states a second
    // output returns the remainder. The remainder starting at (and
    // including) the terminating delimiter is the documented choice under
    // strtok.q-remainder.
    #[test]
    fn summary_derived_two_output_remainder() {
        let _guard = crate::output_count::push_output_count(Some(2));
        let result = run_strtok_char("hello world", Vec::new());
        match result {
            Value::OutputList(outputs) => {
                assert_eq!(outputs.len(), 2);
                assert_char(&outputs[0], "hello", 1, 5);
                assert_char(&outputs[1], " world", 1, 6);
            }
            other => panic!("expected output list, got {other:?}"),
        }
    }

    // summary_derived [FR-006-09]: remainder with a custom delimiter set.
    #[test]
    fn summary_derived_custom_delimiter_remainder() {
        let (token, remain) = split_token("a,b,c", Some(&[',']));
        assert_eq!((token.as_str(), remain.as_str()), ("a", ",b,c"));
    }

    // unresolved_choice [FR-006-10; strtok.q-empty]: empty and all-delimiter
    // inputs yield an empty token and empty remainder (documented choice).
    #[test]
    fn unresolved_choice_empty_and_all_delimiter_inputs() {
        let (token, remain) = split_token("", None);
        assert_eq!((token.as_str(), remain.as_str()), ("", ""));

        let (token, remain) = split_token("   ", None);
        assert_eq!((token.as_str(), remain.as_str()), ("", ""));

        let result = run_strtok_char("   ", Vec::new());
        assert_char(&result, "", 0, 0);
    }

    // unresolved_choice [FR-006-10]: string scalar input returns string
    // outputs (unobserved input class; documented choice mirroring
    // fileparts).
    #[test]
    fn unresolved_choice_string_input_returns_string() {
        let result = run_strtok(Value::String("hello world".into()), Vec::new());
        assert_eq!(result, Value::String("hello".into()));
    }

    // unresolved_choice [FR-006-10]: tab and newline count as default
    // whitespace delimiters (documented choice; only ' ' observed).
    #[test]
    fn unresolved_choice_other_whitespace_delimits() {
        let (token, remain) = split_token("ab\tcd", None);
        assert_eq!((token.as_str(), remain.as_str()), ("ab", "\tcd"));

        let (token, remain) = split_token("\nab cd", None);
        assert_eq!((token.as_str(), remain.as_str()), ("ab", " cd"));
    }

    #[test]
    fn invalid_input_errors() {
        let err = block_on(super::strtok_builtin(Value::Num(5.0), Vec::new())).expect_err("error");
        assert!(err.to_string().contains("strtok"));
    }

    #[test]
    fn invalid_delimiter_errors() {
        let err = block_on(super::strtok_builtin(
            Value::CharArray(CharArray::new_row("a,b")),
            vec![Value::Num(1.0)],
        ))
        .expect_err("error");
        assert!(err.to_string().contains("delimiters"));
    }
}
