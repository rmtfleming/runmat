//! MATLAB-compatible `strmatch` builtin for RunMat (legacy prefix matcher).
//!
//! Clean-room provenance: specs/006-string-search (spec `strmatch` 0.2.0,
//! claims strmatch.signature-primary, strmatch.output-class,
//! strmatch.index-vector).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    ResolveContext, StringArray, Tensor, Type, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ShapeRequirements,
};
use crate::{build_runtime_error, BuiltinResult, RuntimeError};

use super::text_utils::{TextCollection, TextElement};

#[runmat_macros::register_gpu_spec(builtin_path = "crate::builtins::strings::search::strmatch")]
pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "strmatch",
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
    notes: "Executes entirely on the host; text inputs never reside on the GPU.",
};

#[runmat_macros::register_fusion_spec(builtin_path = "crate::builtins::strings::search::strmatch")]
pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "strmatch",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Text operation; not eligible for fusion and materialises host numeric results.",
};

const BUILTIN_NAME: &str = "strmatch";

const STRMATCH_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "idx",
    ty: BuiltinParamType::NumericArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Column vector of 1-based indices of matching rows (double).",
}];

const STRMATCH_INPUTS_BASE: [BuiltinParamDescriptor; 2] = [
    BuiltinParamDescriptor {
        name: "str",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Prefix (or exact) text to look for (char row vector or string scalar).",
    },
    BuiltinParamDescriptor {
        name: "arr",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Array of text rows to search (cellstr, char matrix, or string array).",
    },
];

const STRMATCH_INPUTS_EXACT: [BuiltinParamDescriptor; 3] = [
    BuiltinParamDescriptor {
        name: "str",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Prefix (or exact) text to look for (char row vector or string scalar).",
    },
    BuiltinParamDescriptor {
        name: "arr",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Array of text rows to search (cellstr, char matrix, or string array).",
    },
    BuiltinParamDescriptor {
        name: "flag",
        ty: BuiltinParamType::StringScalar,
        arity: BuiltinParamArity::Required,
        default: Some("'exact'"),
        description: "Match mode flag; `'exact'` requires full equality.",
    },
];

const STRMATCH_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "idx = strmatch(str, arr)",
        inputs: &STRMATCH_INPUTS_BASE,
        outputs: &STRMATCH_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "idx = strmatch(str, arr, 'exact')",
        inputs: &STRMATCH_INPUTS_EXACT,
        outputs: &STRMATCH_OUTPUT,
    },
];

const STRMATCH_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.STRMATCH.INVALID_INPUT",
    identifier: Some("RunMat:strmatch:InvalidInput"),
    when: "Pattern or search array is not a supported text container.",
    message: "strmatch: inputs must be text values",
};

const STRMATCH_ERROR_INVALID_FLAG: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.STRMATCH.INVALID_FLAG",
    identifier: Some("RunMat:strmatch:InvalidFlag"),
    when: "The third argument is not the 'exact' flag.",
    message: "strmatch: invalid match flag; supported flag is 'exact'",
};

const STRMATCH_ERROR_INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.STRMATCH.INTERNAL",
    identifier: Some("RunMat:strmatch:InternalError"),
    when: "Internal index-vector assembly failed.",
    message: "strmatch: internal error",
};

const STRMATCH_ERRORS: [BuiltinErrorDescriptor; 3] = [
    STRMATCH_ERROR_INVALID_INPUT,
    STRMATCH_ERROR_INVALID_FLAG,
    STRMATCH_ERROR_INTERNAL,
];

pub const STRMATCH_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &STRMATCH_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &STRMATCH_ERRORS,
};

fn strmatch_error_with_message(
    message: impl Into<String>,
    error: &'static BuiltinErrorDescriptor,
) -> RuntimeError {
    let mut builder = build_runtime_error(message).with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

fn strmatch_index_type(_args: &[Type], _context: &ResolveContext) -> Type {
    Type::tensor()
}

#[runtime_builtin(
    name = "strmatch",
    category = "strings/search",
    summary = "Return indices of text rows that begin with (or exactly equal) a pattern.",
    keywords = "strmatch,prefix,index,legacy,string search",
    type_resolver(strmatch_index_type),
    descriptor(crate::builtins::strings::search::strmatch::STRMATCH_DESCRIPTOR),
    builtin_path = "crate::builtins::strings::search::strmatch"
)]
async fn strmatch_builtin(pattern: Value, array: Value, rest: Vec<Value>) -> BuiltinResult<Value> {
    let exact = parse_exact_flag(&rest)?;
    let pattern = extract_pattern_text(&pattern)?;
    let entries =
        TextCollection::from_argument(BUILTIN_NAME, array, "second argument").map_err(|err| {
            strmatch_error_with_message(err.message().to_string(), &STRMATCH_ERROR_INVALID_INPUT)
        })?;

    let mut indices = Vec::new();
    for (idx, element) in entries.elements.iter().enumerate() {
        let is_match = match element {
            TextElement::Missing => false,
            TextElement::Text(entry) => {
                if exact {
                    entry == &pattern
                } else {
                    entry.starts_with(pattern.as_str())
                }
            }
        };
        if is_match {
            indices.push((idx + 1) as f64);
        }
    }

    let count = indices.len();
    Tensor::new(indices, vec![count, 1])
        .map(Value::Tensor)
        .map_err(|e| {
            strmatch_error_with_message(format!("{BUILTIN_NAME}: {e}"), &STRMATCH_ERROR_INTERNAL)
        })
}

/// The pattern must be a single piece of text (char row vector, string
/// scalar, or 1-element string array). Multi-row patterns are unobserved in
/// the export and are rejected (documented independent choice).
fn extract_pattern_text(value: &Value) -> BuiltinResult<String> {
    match value {
        Value::CharArray(chars) if chars.rows <= 1 => Ok(chars.data.iter().collect()),
        Value::String(text) => Ok(text.clone()),
        Value::StringArray(StringArray { data, .. }) if data.len() == 1 => Ok(data[0].clone()),
        _ => Err(strmatch_error_with_message(
            format!("{BUILTIN_NAME}: first argument must be a character vector or string scalar"),
            &STRMATCH_ERROR_INVALID_INPUT,
        )),
    }
}

/// Only the observed `'exact'` flag is accepted; any other third argument is
/// rejected (flag values beyond `'exact'` are unobserved in the export —
/// documented independent choice). The flag is matched case-insensitively.
fn parse_exact_flag(rest: &[Value]) -> BuiltinResult<bool> {
    match rest {
        [] => Ok(false),
        [flag] => {
            let text = match flag {
                Value::CharArray(chars) if chars.rows <= 1 => chars.data.iter().collect::<String>(),
                Value::String(text) => text.clone(),
                Value::StringArray(StringArray { data, .. }) if data.len() == 1 => data[0].clone(),
                _ => {
                    return Err(strmatch_error_with_message(
                        STRMATCH_ERROR_INVALID_FLAG.message,
                        &STRMATCH_ERROR_INVALID_FLAG,
                    ))
                }
            };
            if text.eq_ignore_ascii_case("exact") {
                Ok(true)
            } else {
                Err(strmatch_error_with_message(
                    format!(
                        "{BUILTIN_NAME}: invalid match flag '{text}'; supported flag is 'exact'"
                    ),
                    &STRMATCH_ERROR_INVALID_FLAG,
                ))
            }
        }
        _ => Err(strmatch_error_with_message(
            format!("{BUILTIN_NAME}: too many input arguments"),
            &STRMATCH_ERROR_INVALID_FLAG,
        )),
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use futures::executor::block_on;
    use runmat_builtins::{CellArray, CharArray};

    fn run_strmatch(pattern: Value, array: Value, rest: Vec<Value>) -> BuiltinResult<Value> {
        block_on(strmatch_builtin(pattern, array, rest))
    }

    fn cellstr_column(entries: &[&str]) -> Value {
        let data = entries.iter().map(|s| Value::from(*s)).collect::<Vec<_>>();
        let rows = entries.len();
        Value::Cell(CellArray::new(data, rows, 1).unwrap())
    }

    fn assert_double_column(value: &Value, expected: &[f64]) {
        match value {
            Value::Tensor(tensor) => {
                assert_eq!(tensor.data, expected);
                assert_eq!(tensor.shape, vec![expected.len(), 1]);
            }
            other => panic!("expected double column vector, got {other:?}"),
        }
    }

    // Normative [FR-006-04; strmatch.signature-primary,
    // strmatch.output-class, strmatch.index-vector] — observed case
    // `prefix`: strmatch('ap', {'apple';'apricot';'banana'}) => [1;2]
    // (double, 2x1).
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn observed_prefix_match_indices() {
        let result = run_strmatch(
            Value::CharArray(CharArray::new_row("ap")),
            cellstr_column(&["apple", "apricot", "banana"]),
            Vec::new(),
        )
        .expect("strmatch");
        assert_double_column(&result, &[1.0, 2.0]);
    }

    // Normative [FR-006-05; strmatch.output-class, strmatch.index-vector] —
    // observed case `exact`:
    // strmatch('apple', {'apple';'apple pie'}, 'exact') => 1 (double, 1x1).
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn observed_exact_match_index() {
        let result = run_strmatch(
            Value::CharArray(CharArray::new_row("apple")),
            cellstr_column(&["apple", "apple pie"]),
            vec![Value::CharArray(CharArray::new_row("exact"))],
        )
        .expect("strmatch");
        assert_double_column(&result, &[1.0]);
    }

    // Normative [FR-006-04; strmatch.output-class, strmatch.index-vector] —
    // observed case `no-match`: strmatch('zz', {'apple';'banana'}) =>
    // zeros(0,1) (double, 0x1).
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn observed_no_match_is_zero_by_one_empty() {
        let result = run_strmatch(
            Value::CharArray(CharArray::new_row("zz")),
            cellstr_column(&["apple", "banana"]),
            Vec::new(),
        )
        .expect("strmatch");
        assert_double_column(&result, &[]);
    }

    // unresolved_choice [FR-006-06]: char-matrix search arrays match by row
    // (unobserved input class in the export; documented choice).
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_char_matrix_rows() {
        let matrix = CharArray::new("catcowdog".chars().collect(), 3, 3).unwrap();
        let result = run_strmatch(
            Value::CharArray(CharArray::new_row("c")),
            Value::CharArray(matrix),
            Vec::new(),
        )
        .expect("strmatch");
        assert_double_column(&result, &[1.0, 2.0]);
    }

    // unresolved_choice [FR-006-06]: string-array search arrays use linear
    // (column-major) element order (unobserved; documented choice).
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_string_array_entries() {
        let entries = StringArray::new(
            vec!["apple".into(), "apricot".into(), "banana".into()],
            vec![3, 1],
        )
        .unwrap();
        let result = run_strmatch(
            Value::String("ap".into()),
            Value::StringArray(entries),
            Vec::new(),
        )
        .expect("strmatch");
        assert_double_column(&result, &[1.0, 2.0]);
    }

    // unresolved_choice [FR-006-06]: exact mode rejects prefix-only matches.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_exact_excludes_prefix_only() {
        let result = run_strmatch(
            Value::CharArray(CharArray::new_row("ap")),
            cellstr_column(&["apple", "apricot"]),
            vec![Value::CharArray(CharArray::new_row("exact"))],
        )
        .expect("strmatch");
        assert_double_column(&result, &[]);
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn invalid_flag_errors() {
        let err = run_strmatch(
            Value::CharArray(CharArray::new_row("ap")),
            cellstr_column(&["apple"]),
            vec![Value::CharArray(CharArray::new_row("prefix"))],
        )
        .unwrap_err();
        assert!(err.to_string().contains("invalid match flag"));
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn invalid_pattern_type_errors() {
        let err =
            run_strmatch(Value::Num(1.0), cellstr_column(&["apple"]), Vec::new()).unwrap_err();
        assert!(err
            .to_string()
            .contains("first argument must be a character vector or string scalar"));
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn invalid_array_type_errors() {
        let err = run_strmatch(
            Value::CharArray(CharArray::new_row("ap")),
            Value::Num(1.0),
            Vec::new(),
        )
        .unwrap_err();
        assert!(err.to_string().contains("second argument must be text"));
    }

    #[test]
    fn strmatch_type_is_tensor() {
        assert_eq!(
            strmatch_index_type(&[Type::String], &ResolveContext::new(Vec::new())),
            Type::tensor()
        );
    }
}
