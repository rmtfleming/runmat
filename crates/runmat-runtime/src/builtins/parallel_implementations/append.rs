// Parallel clean-room implementation of `append` (independently developed against the
// matlab-interface-spec black-box specs). 0-6-0 ships the DEFAULT implementation; this copy is
// retained unregistered for differential cross-validation. Original path: crates/runmat-runtime/src/builtins/strings/transform/append.rs
//! MATLAB-compatible `append` builtin with GPU-aware semantics for RunMat.
//!
//! `append` (R2020b+) concatenates text pieces in order. Unlike `strcat`, it
//! does **not** trim trailing whitespace from character-array inputs. Observed
//! behaviour (clean-room export `append` 0.2.0, R2026a):
//! - `append("a","b") -> "ab"` (string scalar, class `string`, size `[1 1]`)
//! - `append("x","y","z") -> "xyz"` (string scalar)
//! - `append('a','b') -> 'ab'` (char row, class `char`, size `[1 2]`)
//!
//! Output-class rule (claim `append.concatenates-text`): if **any** argument is
//! a string the result is `string`; if **all** arguments are char the result is
//! a char row. Behaviour for string arrays (element-wise), trailing whitespace,
//! and non-text inputs is unobserved and handled by documented independent
//! choice (see `specs/026-append/spec.md`).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    CharArray, StringArray, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::broadcast::{broadcast_index, broadcast_shapes, compute_strides};
use crate::builtins::common::map_control_flow_with_builtin;
use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ShapeRequirements,
};
use crate::builtins::strings::common::char_row_to_string_slice;
use crate::builtins::strings::type_resolvers::text_concat_type;
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "append",
    op_kind: GpuOpKind::Custom("string-transform"),
    supported_precisions: &[],
    broadcast: BroadcastSemantics::Matlab,
    provider_hooks: &[],
    constant_strategy: ConstantStrategy::InlineLiteral,
    residency: ResidencyPolicy::GatherImmediately,
    nan_mode: ReductionNaN::Include,
    two_pass_threshold: None,
    workgroup_size: None,
    accepts_nan_mode: false,
    notes: "Executes on the CPU and preserves trailing whitespace; GPU inputs are gathered before concatenation.",
};

pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "append",
    shape: ShapeRequirements::BroadcastCompatible,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "String concatenation runs on the host and is not eligible for fusion.",
};

const BUILTIN_NAME: &str = "append";

const APPEND_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "str",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Combined text: a string when any input is a string, otherwise a char row.",
}];

const APPEND_INPUTS: [BuiltinParamDescriptor; 2] = [
    BuiltinParamDescriptor {
        name: "str1",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "First text input (string or character array).",
    },
    BuiltinParamDescriptor {
        name: "str2",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Variadic,
        default: None,
        description: "Additional text inputs appended in order.",
    },
];

const APPEND_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "str = append(str1, str2, ...)",
    inputs: &APPEND_INPUTS,
    outputs: &APPEND_OUTPUT,
}];

const APPEND_ERROR_NOT_ENOUGH_INPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.APPEND.NOT_ENOUGH_INPUTS",
    identifier: Some("RunMat:append:NotEnoughInputs"),
    when: "No arguments are supplied.",
    message: "append: not enough input arguments",
};

const APPEND_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.APPEND.INVALID_INPUT",
    identifier: Some("RunMat:append:InvalidInput"),
    when: "An input is not a string or character array.",
    message: "append: inputs must be strings or character arrays",
};

const APPEND_ERROR_SIZE_MISMATCH: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.APPEND.SIZE_MISMATCH",
    identifier: Some("RunMat:append:SizeMismatch"),
    when: "Input shapes are not broadcast-compatible.",
    message: "append: array sizes are not compatible for broadcasting",
};

const APPEND_ERROR_INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.APPEND.INTERNAL",
    identifier: Some("RunMat:append:InternalError"),
    when: "Internal output container construction failed.",
    message: "append: internal error",
};

const APPEND_ERRORS: [BuiltinErrorDescriptor; 4] = [
    APPEND_ERROR_NOT_ENOUGH_INPUTS,
    APPEND_ERROR_INVALID_INPUT,
    APPEND_ERROR_SIZE_MISMATCH,
    APPEND_ERROR_INTERNAL,
];

pub const APPEND_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &APPEND_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &APPEND_ERRORS,
};

fn map_flow(err: RuntimeError) -> RuntimeError {
    map_control_flow_with_builtin(err, BUILTIN_NAME)
}

fn append_error_with_message(
    message: impl Into<String>,
    error: &'static BuiltinErrorDescriptor,
) -> RuntimeError {
    let mut builder = build_runtime_error(message).with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

fn append_error(error: &'static BuiltinErrorDescriptor) -> RuntimeError {
    append_error_with_message(error.message, error)
}

/// The MATLAB text class of a single input operand. `append` only recognises
/// the two classes it was observed to accept.
#[derive(Clone, Copy, PartialEq, Eq)]
enum OperandKind {
    String,
    Char,
}

/// One text operand normalised into a flat list of pieces plus its shape.
struct TextOperand {
    data: Vec<String>,
    shape: Vec<usize>,
    strides: Vec<usize>,
    kind: OperandKind,
}

impl TextOperand {
    fn from_value(value: Value) -> BuiltinResult<Self> {
        match value {
            Value::String(s) => Ok(Self::from_string_scalar(s)),
            Value::StringArray(sa) => Ok(Self::from_string_array(sa)),
            Value::CharArray(ca) => Ok(Self::from_char_array(&ca)),
            _ => Err(append_error(&APPEND_ERROR_INVALID_INPUT)),
        }
    }

    fn from_string_scalar(text: String) -> Self {
        Self {
            data: vec![text],
            shape: vec![1, 1],
            strides: compute_strides(&[1, 1]),
            kind: OperandKind::String,
        }
    }

    fn from_string_array(array: StringArray) -> Self {
        let shape = array.shape.clone();
        let strides = compute_strides(&shape);
        Self {
            data: array.data,
            shape,
            strides,
            kind: OperandKind::String,
        }
    }

    /// A character array is treated as one text piece per row, matching how
    /// RunMat's text-concatenation builtins model char operands. Unlike
    /// `strcat`, trailing whitespace is preserved (documented independent
    /// choice; unobserved).
    fn from_char_array(array: &CharArray) -> Self {
        let rows = array.rows;
        let cols = array.cols;
        let mut elements = Vec::with_capacity(rows);
        for row in 0..rows {
            elements.push(char_row_to_string_slice(&array.data, cols, row));
        }
        let shape = vec![rows, 1];
        let strides = compute_strides(&shape);
        Self {
            data: elements,
            shape,
            strides,
            kind: OperandKind::Char,
        }
    }
}

/// The container class of the result, resolved from the input operand classes.
#[derive(Clone, Copy, PartialEq, Eq)]
enum OutputKind {
    Char,
    String,
}

impl OutputKind {
    /// Any string input forces a string result; otherwise all-char stays char.
    fn update(self, operand_kind: OperandKind) -> Self {
        match (self, operand_kind) {
            (_, OperandKind::String) => OutputKind::String,
            (OutputKind::String, _) => OutputKind::String,
            _ => OutputKind::Char,
        }
    }
}

async fn append_builtin(rest: Vec<Value>) -> BuiltinResult<Value> {
    if rest.is_empty() {
        return Err(append_error(&APPEND_ERROR_NOT_ENOUGH_INPUTS));
    }

    let mut operands = Vec::with_capacity(rest.len());
    let mut output_kind = OutputKind::Char;

    for value in rest {
        let gathered = gather_if_needed_async(&value).await.map_err(map_flow)?;
        let operand = TextOperand::from_value(gathered)?;
        output_kind = output_kind.update(operand.kind);
        operands.push(operand);
    }

    let mut output_shape = operands
        .first()
        .map(|op| op.shape.clone())
        .unwrap_or_else(|| vec![1, 1]);
    for operand in operands.iter().skip(1) {
        output_shape =
            broadcast_shapes(BUILTIN_NAME, &output_shape, &operand.shape).map_err(|e| {
                append_error_with_message(
                    format!("{}: {e}", APPEND_ERROR_SIZE_MISMATCH.message),
                    &APPEND_ERROR_SIZE_MISMATCH,
                )
            })?;
    }

    let total_len: usize = output_shape.iter().product();
    let mut concatenated = Vec::with_capacity(total_len);

    for linear in 0..total_len {
        let mut buffer = String::new();
        for operand in &operands {
            let idx = broadcast_index(linear, &output_shape, &operand.shape, &operand.strides);
            buffer.push_str(&operand.data[idx]);
        }
        concatenated.push(buffer);
    }

    match output_kind {
        OutputKind::String => build_string_output(concatenated, &output_shape),
        OutputKind::Char => build_char_output(concatenated),
    }
}

fn build_string_output(data: Vec<String>, shape: &[usize]) -> BuiltinResult<Value> {
    if data.is_empty() {
        let array = StringArray::new(data, shape.to_vec()).map_err(|e| {
            append_error_with_message(format!("{BUILTIN_NAME}: {e}"), &APPEND_ERROR_INTERNAL)
        })?;
        return Ok(Value::StringArray(array));
    }

    let is_scalar = shape.is_empty() || shape.iter().all(|&dim| dim == 1);
    if is_scalar {
        return Ok(Value::String(data[0].clone()));
    }

    let array = StringArray::new(data, shape.to_vec()).map_err(|e| {
        append_error_with_message(format!("{BUILTIN_NAME}: {e}"), &APPEND_ERROR_INTERNAL)
    })?;
    Ok(Value::StringArray(array))
}

fn build_char_output(data: Vec<String>) -> BuiltinResult<Value> {
    let rows = data.len();
    if rows == 0 {
        let array = CharArray::new(Vec::new(), 0, 0).map_err(|e| {
            append_error_with_message(format!("{BUILTIN_NAME}: {e}"), &APPEND_ERROR_INTERNAL)
        })?;
        return Ok(Value::CharArray(array));
    }

    let max_cols = data.iter().map(|s| s.chars().count()).max().unwrap_or(0);
    let mut chars = Vec::with_capacity(rows * max_cols);
    for text in data {
        let mut row_chars: Vec<char> = text.chars().collect();
        if row_chars.len() < max_cols {
            row_chars.resize(max_cols, ' ');
        }
        chars.extend(row_chars.into_iter());
    }
    let array = CharArray::new(chars, rows, max_cols).map_err(|e| {
        append_error_with_message(format!("{BUILTIN_NAME}: {e}"), &APPEND_ERROR_INTERNAL)
    })?;
    Ok(Value::CharArray(array))
}

#[cfg(all(test, feature = "parallel_xval"))]
pub(crate) mod tests {
    use super::*;
    #[cfg(feature = "wgpu")]
    use crate::builtins::common::test_support;
    #[cfg(feature = "wgpu")]
    use runmat_builtins::Tensor;
    use runmat_builtins::{CharArray, IntValue, ResolveContext, StringArray, Type};

    fn run_append(rest: Vec<Value>) -> BuiltinResult<Value> {
        futures::executor::block_on(append_builtin(rest))
    }

    // ---- observed (normative) --------------------------------------------
    // These reproduce the exact black-box cases from the approved export
    // `append` 0.2.0 (claims: append.signature-primary, append.concatenates-text).

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn observed_two_strings_return_string_scalar() {
        // append("a","b") -> "ab" (class string, size [1 1])
        let result =
            run_append(vec![Value::String("a".into()), Value::String("b".into())]).expect("append");
        assert_eq!(result, Value::String("ab".into()));
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn observed_three_strings_return_string_scalar() {
        // append("x","y","z") -> "xyz" (class string, size [1 1])
        let result = run_append(vec![
            Value::String("x".into()),
            Value::String("y".into()),
            Value::String("z".into()),
        ])
        .expect("append");
        assert_eq!(result, Value::String("xyz".into()));
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn observed_two_chars_return_char_row() {
        // append('a','b') -> 'ab' (class char, size [1 2])
        let first = CharArray::new_row("a");
        let second = CharArray::new_row("b");
        let result =
            run_append(vec![Value::CharArray(first), Value::CharArray(second)]).expect("append");
        match result {
            Value::CharArray(ca) => {
                assert_eq!(ca.rows, 1);
                assert_eq!(ca.cols, 2);
                assert_eq!(ca.data, vec!['a', 'b']);
            }
            other => panic!("expected char array, got {other:?}"),
        }
    }

    // ---- documented independent choices (non-normative; unobserved) ------

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_char_inputs_preserve_trailing_whitespace() {
        // Distinguishes append from strcat: 'hi ' + 'there' keeps the space.
        // Unobserved for append; documented choice (append.q-types adjacent).
        let first = CharArray::new_row("hi ");
        let second = CharArray::new_row("there");
        let result =
            run_append(vec![Value::CharArray(first), Value::CharArray(second)]).expect("append");
        match result {
            Value::CharArray(ca) => {
                assert_eq!(ca.rows, 1);
                assert_eq!(ca.cols, 8);
                assert_eq!(ca.data, "hi there".chars().collect::<Vec<char>>());
            }
            other => panic!("expected char array, got {other:?}"),
        }
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_mixed_char_and_string_returns_string() {
        // Output-class rule (append.concatenates-text): any string -> string.
        let result = run_append(vec![
            Value::CharArray(CharArray::new_row("run")),
            Value::String("mat".into()),
        ])
        .expect("append");
        assert_eq!(result, Value::String("runmat".into()));
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_single_string_argument_returns_itself() {
        // Zero-length variadic tail: append("solo") -> "solo". Unobserved.
        let result = run_append(vec![Value::String("solo".into())]).expect("append");
        assert_eq!(result, Value::String("solo".into()));
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_string_array_concatenates_element_wise() {
        // String arrays combine element-wise with scalar broadcasting.
        let array = StringArray::new(vec!["core".into(), "runtime".into()], vec![1, 2])
            .expect("string array");
        let result = run_append(vec![
            Value::String("runmat-".into()),
            Value::StringArray(array),
        ])
        .expect("append");
        match result {
            Value::StringArray(sa) => {
                assert_eq!(sa.shape, vec![1, 2]);
                assert_eq!(
                    sa.data,
                    vec![String::from("runmat-core"), String::from("runmat-runtime")]
                );
            }
            other => panic!("expected string array, got {other:?}"),
        }
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_empty_string_array_returns_empty_array() {
        let empty = StringArray::new(Vec::<String>::new(), vec![0, 2]).expect("string array");
        let result = run_append(vec![
            Value::StringArray(empty),
            Value::String("prefix".into()),
        ])
        .expect("append");
        match result {
            Value::StringArray(sa) => {
                assert_eq!(sa.shape, vec![0, 2]);
                assert!(sa.data.is_empty());
            }
            other => panic!("expected empty string array, got {other:?}"),
        }
    }

    // ---- error conditions (documented choices) ---------------------------

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_no_arguments_errors() {
        let err = run_append(Vec::new()).expect_err("expected error");
        assert_eq!(err.to_string(), APPEND_ERROR_NOT_ENOUGH_INPUTS.message);
        assert_eq!(
            err.identifier.as_deref(),
            APPEND_ERROR_NOT_ENOUGH_INPUTS.identifier
        );
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_non_text_input_errors() {
        let err = run_append(vec![Value::Int(IntValue::I32(4))]).expect_err("expected error");
        assert_eq!(err.to_string(), APPEND_ERROR_INVALID_INPUT.message);
        assert_eq!(
            err.identifier.as_deref(),
            APPEND_ERROR_INVALID_INPUT.identifier
        );
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_mismatched_sizes_error() {
        let left = CharArray::new(vec!['A', 'B'], 2, 1).expect("char");
        let right = CharArray::new(vec!['C', 'D', 'E'], 3, 1).expect("char");
        let err = run_append(vec![Value::CharArray(left), Value::CharArray(right)])
            .expect_err("expected broadcast error");
        assert!(err
            .to_string()
            .starts_with(APPEND_ERROR_SIZE_MISMATCH.message));
        assert_eq!(
            err.identifier.as_deref(),
            APPEND_ERROR_SIZE_MISMATCH.identifier
        );
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn append_type_resolver_reports_string_for_string_input() {
        assert_eq!(
            text_concat_type(&[Type::String], &ResolveContext::new(Vec::new())),
            Type::String
        );
    }

    #[cfg(feature = "wgpu")]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn append_gpu_operand_still_errors_on_type() {
        test_support::with_test_provider(|provider| {
            let tensor = Tensor::new(vec![1.0, 2.0], vec![1, 2]).expect("tensor");
            let view = runmat_accelerate_api::HostTensorView {
                data: &tensor.data,
                shape: &tensor.shape,
            };
            let handle = provider.upload(&view).expect("upload");
            let err = run_append(vec![Value::GpuTensor(handle)]).expect_err("expected error");
            assert_eq!(err.to_string(), APPEND_ERROR_INVALID_INPUT.message);
        });
    }
}
