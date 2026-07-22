//! MATLAB-compatible `matches` builtin for RunMat.
//!
//! Clean-room provenance: specs/006-string-search (spec `matches` 0.2.0,
//! claims matches.signature-primary, matches.output-class,
//! matches.scalar-membership; array-first-argument shape is a documented
//! independent choice).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::map_control_flow_with_builtin;
use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ShapeRequirements,
};
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

use super::text_utils::{logical_result, parse_ignore_case, TextCollection, TextElement};
use crate::builtins::strings::type_resolvers::logical_text_match_type;

#[runmat_macros::register_gpu_spec(builtin_path = "crate::builtins::strings::search::matches")]
pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "matches",
    op_kind: GpuOpKind::Custom("string-search"),
    supported_precisions: &[],
    broadcast: BroadcastSemantics::Matlab,
    provider_hooks: &[],
    constant_strategy: ConstantStrategy::InlineLiteral,
    residency: ResidencyPolicy::GatherImmediately,
    nan_mode: ReductionNaN::Include,
    two_pass_threshold: None,
    workgroup_size: None,
    accepts_nan_mode: false,
    notes: "Executes entirely on the host; inputs are gathered from the GPU before performing equality checks.",
};

#[runmat_macros::register_fusion_spec(builtin_path = "crate::builtins::strings::search::matches")]
pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "matches",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Text operation; not eligible for fusion and materialises host logical results.",
};

const BUILTIN_NAME: &str = "matches";

const MATCHES_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "tf",
    ty: BuiltinParamType::LogicalArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Logical result indicating whether each text element equals a pattern.",
}];

const MATCHES_INPUTS_BASE: [BuiltinParamDescriptor; 2] = [
    BuiltinParamDescriptor {
        name: "str",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Text input (string/char/cell/string array).",
    },
    BuiltinParamDescriptor {
        name: "pat",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Pattern text (string/char/cell/string array).",
    },
];

const MATCHES_INPUTS_IGNORE_CASE_PAIR: [BuiltinParamDescriptor; 4] = [
    BuiltinParamDescriptor {
        name: "str",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Text input (string/char/cell/string array).",
    },
    BuiltinParamDescriptor {
        name: "pat",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Pattern text (string/char/cell/string array).",
    },
    BuiltinParamDescriptor {
        name: "name",
        ty: BuiltinParamType::StringScalar,
        arity: BuiltinParamArity::Required,
        default: Some("\"IgnoreCase\""),
        description: "Option name (`\"IgnoreCase\"`).",
    },
    BuiltinParamDescriptor {
        name: "value",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Option value for `\"IgnoreCase\"`.",
    },
];

const MATCHES_INPUTS_OPTION_PAIRS: [BuiltinParamDescriptor; 3] = [
    BuiltinParamDescriptor {
        name: "str",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Text input (string/char/cell/string array).",
    },
    BuiltinParamDescriptor {
        name: "pat",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Pattern text (string/char/cell/string array).",
    },
    BuiltinParamDescriptor {
        name: "nameValuePairs...",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Variadic,
        default: None,
        description: "Name-value option pairs (`\"IgnoreCase\"`, value).",
    },
];

const MATCHES_SIGNATURES: [BuiltinSignatureDescriptor; 3] = [
    BuiltinSignatureDescriptor {
        label: "tf = matches(str, pat)",
        inputs: &MATCHES_INPUTS_BASE,
        outputs: &MATCHES_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "tf = matches(str, pat, \"IgnoreCase\", value)",
        inputs: &MATCHES_INPUTS_IGNORE_CASE_PAIR,
        outputs: &MATCHES_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "tf = matches(str, pat, nameValuePairs...)",
        inputs: &MATCHES_INPUTS_OPTION_PAIRS,
        outputs: &MATCHES_OUTPUT,
    },
];

const MATCHES_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.MATCHES.INVALID_INPUT",
    identifier: Some("RunMat:matches:InvalidInput"),
    when: "Text or pattern input is not a supported text container.",
    message: "matches: text and pattern inputs must be text values",
};

const MATCHES_ERROR_INVALID_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.MATCHES.INVALID_OPTION",
    identifier: Some("RunMat:matches:InvalidOption"),
    when: "IgnoreCase option arguments are invalid or malformed.",
    message: "matches: invalid option arguments",
};

const MATCHES_ERROR_INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.MATCHES.INTERNAL",
    identifier: Some("RunMat:matches:InternalError"),
    when: "Internal logical result assembly failed.",
    message: "matches: internal error",
};

const MATCHES_ERRORS: [BuiltinErrorDescriptor; 3] = [
    MATCHES_ERROR_INVALID_INPUT,
    MATCHES_ERROR_INVALID_OPTION,
    MATCHES_ERROR_INTERNAL,
];

pub const MATCHES_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &MATCHES_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &MATCHES_ERRORS,
};

fn matches_error_with_message(
    message: impl Into<String>,
    error: &'static BuiltinErrorDescriptor,
) -> RuntimeError {
    let mut builder = build_runtime_error(message).with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

fn remap_matches_flow(err: RuntimeError) -> RuntimeError {
    map_control_flow_with_builtin(err, BUILTIN_NAME)
}

#[runtime_builtin(
    name = "matches",
    category = "strings/search",
    summary = "Test whether text inputs equal a pattern or any of a set of patterns.",
    keywords = "matches,equality,text,ignorecase,search",
    accel = "sink",
    type_resolver(logical_text_match_type),
    descriptor(crate::builtins::strings::search::matches::MATCHES_DESCRIPTOR),
    builtin_path = "crate::builtins::strings::search::matches"
)]
async fn matches_builtin(
    text: Value,
    pattern: Value,
    rest: Vec<Value>,
) -> crate::BuiltinResult<Value> {
    let text = gather_if_needed_async(&text)
        .await
        .map_err(remap_matches_flow)?;
    let pattern = gather_if_needed_async(&pattern)
        .await
        .map_err(remap_matches_flow)?;
    let ignore_case = parse_ignore_case(BUILTIN_NAME, &rest).map_err(|err| {
        matches_error_with_message(err.message().to_string(), &MATCHES_ERROR_INVALID_OPTION)
    })?;
    let subject = TextCollection::from_subject(BUILTIN_NAME, text).map_err(|err| {
        matches_error_with_message(err.message().to_string(), &MATCHES_ERROR_INVALID_INPUT)
    })?;
    let patterns = TextCollection::from_pattern(BUILTIN_NAME, pattern).map_err(|err| {
        matches_error_with_message(err.message().to_string(), &MATCHES_ERROR_INVALID_INPUT)
    })?;
    evaluate_matches(&subject, &patterns, ignore_case)
}

/// The observed rule (matches.scalar-membership) covers scalar first
/// arguments: the result is true when the subject equals any of the given
/// patterns. Generalising element-wise over an array first argument (result
/// shaped like `str`, each element tested against the whole pattern set) is a
/// documented independent choice consistent with the observed membership
/// semantics; it is not asserted as MATLAB-conformant.
fn evaluate_matches(
    subject: &TextCollection,
    patterns: &TextCollection,
    ignore_case: bool,
) -> BuiltinResult<Value> {
    let pattern_lower = if ignore_case {
        Some(patterns.lowercased())
    } else {
        None
    };

    let mut data = Vec::with_capacity(subject.elements.len());
    for element in &subject.elements {
        let value = match element {
            TextElement::Missing => false,
            TextElement::Text(text) => {
                let lowered_subject = if ignore_case {
                    Some(text.to_lowercase())
                } else {
                    None
                };
                patterns
                    .elements
                    .iter()
                    .enumerate()
                    .any(|(idx, pattern)| match pattern {
                        TextElement::Missing => false,
                        TextElement::Text(pattern) => {
                            if ignore_case {
                                let lowered_pattern = pattern_lower
                                    .as_ref()
                                    .and_then(|vec| vec[idx].as_deref())
                                    .expect("lowercase pattern available");
                                lowered_subject.as_deref() == Some(lowered_pattern)
                            } else {
                                text == pattern
                            }
                        }
                    })
            }
        };
        data.push(if value { 1 } else { 0 });
    }
    logical_result(BUILTIN_NAME, data, subject.shape.clone()).map_err(|err| {
        matches_error_with_message(err.message().to_string(), &MATCHES_ERROR_INTERNAL)
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use runmat_builtins::{CellArray, CharArray, LogicalArray, ResolveContext, StringArray, Type};

    fn run_matches(text: Value, pattern: Value, rest: Vec<Value>) -> BuiltinResult<Value> {
        futures::executor::block_on(matches_builtin(text, pattern, rest))
    }

    // Normative [FR-006-01; matches.signature-primary, matches.output-class,
    // matches.scalar-membership] — observed case `in-set`:
    // matches("hello", ["hello","world"]) => logical 1x1 true.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn observed_in_set_membership_is_true() {
        let patterns = StringArray::new(vec!["hello".into(), "world".into()], vec![1, 2]).unwrap();
        let result = run_matches(
            Value::String("hello".into()),
            Value::StringArray(patterns),
            Vec::new(),
        )
        .expect("matches");
        assert_eq!(result, Value::Bool(true));
    }

    // Normative [FR-006-01; matches.output-class, matches.scalar-membership]
    // — observed case `scalar`: matches('abc', 'abc') => logical 1x1 true.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn observed_scalar_char_equality_is_true() {
        let result = run_matches(
            Value::CharArray(CharArray::new_row("abc")),
            Value::CharArray(CharArray::new_row("abc")),
            Vec::new(),
        )
        .expect("matches");
        assert_eq!(result, Value::Bool(true));
    }

    // Normative [FR-006-02; matches.output-class, matches.scalar-membership]
    // — observed case `ignore-case`:
    // matches("Cat", "cat", 'IgnoreCase', true) => logical 1x1 true.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn observed_ignore_case_option_is_true() {
        let result = run_matches(
            Value::String("Cat".into()),
            Value::String("cat".into()),
            vec![Value::String("IgnoreCase".into()), Value::Bool(true)],
        )
        .expect("matches");
        assert_eq!(result, Value::Bool(true));
    }

    // unresolved_choice [FR-006-03]: non-equal subject yields false; exact
    // (not substring) equality distinguishes matches from contains.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_non_member_is_false() {
        let patterns = StringArray::new(vec!["hello".into(), "world".into()], vec![1, 2]).unwrap();
        let result = run_matches(
            Value::String("hell".into()),
            Value::StringArray(patterns),
            Vec::new(),
        )
        .expect("matches");
        assert_eq!(result, Value::Bool(false));
    }

    // unresolved_choice [FR-006-03]: case-sensitive by default.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_case_sensitive_default_is_false() {
        let result = run_matches(
            Value::String("Cat".into()),
            Value::String("cat".into()),
            Vec::new(),
        )
        .expect("matches");
        assert_eq!(result, Value::Bool(false));
    }

    // unresolved_choice [FR-006-03]: array first argument yields a logical
    // array shaped like the first argument, each element tested against the
    // whole pattern set (documented choice; unobserved in the export).
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_array_first_argument_shapes_result() {
        let subjects = StringArray::new(
            vec!["alpha".into(), "beta".into(), "gamma".into()],
            vec![3, 1],
        )
        .unwrap();
        let patterns = StringArray::new(vec!["beta".into(), "gamma".into()], vec![1, 2]).unwrap();
        let result = run_matches(
            Value::StringArray(subjects),
            Value::StringArray(patterns),
            Vec::new(),
        )
        .expect("matches");
        let expected = LogicalArray::new(vec![0, 1, 1], vec![3, 1]).unwrap();
        assert_eq!(result, Value::LogicalArray(expected));
    }

    // unresolved_choice [FR-006-03]: cell array of char vectors as patterns.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_cell_patterns_membership() {
        let cell = CellArray::new(vec![Value::from("red"), Value::from("green")], 1, 2).unwrap();
        let result = run_matches(Value::String("green".into()), Value::Cell(cell), Vec::new())
            .expect("matches");
        assert_eq!(result, Value::Bool(true));
    }

    // unresolved_choice [FR-006-03]: missing string subjects compare false.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_missing_subject_is_false() {
        let array = StringArray::new(vec!["<missing>".into()], vec![1, 1]).unwrap();
        let result = run_matches(
            Value::StringArray(array),
            Value::String("<missing>".into()),
            Vec::new(),
        )
        .expect("matches");
        assert_eq!(result, Value::Bool(false));
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn invalid_option_name_errors() {
        let err = run_matches(
            Value::String("foo".into()),
            Value::String("foo".into()),
            vec![Value::String("IgnoreCases".into()), Value::Bool(true)],
        )
        .unwrap_err();
        assert!(err.to_string().contains("unknown option"));
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn invalid_subject_type_errors() {
        let err = run_matches(Value::Num(1.0), Value::String("a".into()), Vec::new()).unwrap_err();
        assert!(err.to_string().contains("first argument must be text"));
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn invalid_pattern_type_errors() {
        let err =
            run_matches(Value::String("foo".into()), Value::Num(1.0), Vec::new()).unwrap_err();
        assert!(err.to_string().contains("pattern must be text"));
    }

    #[test]
    fn matches_type_is_logical_match() {
        assert_eq!(
            logical_text_match_type(
                &[Type::String, Type::String],
                &ResolveContext::new(Vec::new()),
            ),
            Type::Bool
        );
    }
}
