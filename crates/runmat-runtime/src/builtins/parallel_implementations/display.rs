// Parallel clean-room implementation of `display` (independently developed against the
// matlab-interface-spec black-box specs). 0-6-0 ships the DEFAULT implementation; this copy is
// retained unregistered for differential cross-validation. Original path: crates/runmat-runtime/src/builtins/io/display.rs
//! MATLAB-compatible `display` builtin (feature 013-display-formatting).
//!
//! Behavioural source: approved clean-room export `display` 0.2.0
//! (`specs/013-display-formatting/provenance.md`). The normative surface is
//! the byte-exact command-window text for the three observed cases:
//! a real scalar (`"    42\n\n"`), a real row vector
//! (`"     1     2     3\n\n"`, contiguous width-6 right-aligned fields)
//! and a char row (`"hi\n"`, no quotes, no indent, no trailing blank line).
//! All other value kinds are rendered by documented independent choice:
//! they delegate to the `disp` builtin's rendering followed by one blank
//! line. The two-argument form `display(X, name)` is rejected (recorded
//! human decision at the feature gate; the form is unapproved in the
//! export and no header formatting may be invented).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    CharArray, Tensor, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ShapeRequirements,
};
use crate::builtins::strings::common::char_row_to_string;
use crate::console::{record_console_output, ConsoleStream};
use crate::gather_if_needed_async;

pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "display",
    op_kind: GpuOpKind::Custom("sink"),
    supported_precisions: &[],
    broadcast: BroadcastSemantics::None,
    provider_hooks: &[],
    constant_strategy: ConstantStrategy::InlineLiteral,
    residency: ResidencyPolicy::GatherImmediately,
    nan_mode: ReductionNaN::Include,
    two_pass_threshold: None,
    workgroup_size: None,
    accepts_nan_mode: false,
    notes: "Always formats on the CPU; GPU tensors are gathered via the active provider before display.",
};

pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "display",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Side-effecting sink; excluded from fusion planning.",
};

/// Minimum field width (in characters) for the numeric display layout.
/// Derived from the observed cases: `42` renders as a 6-character
/// right-aligned field and `[1 2 3]` as three contiguous 6-character
/// right-aligned fields with no separator between them.
const NUMERIC_MIN_FIELD_WIDTH: usize = 6;

const DISPLAY_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "ans",
    ty: BuiltinParamType::NumericArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Empty matrix placeholder returned by sink invocation.",
}];
const DISPLAY_INPUTS_VALUE: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "X",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Value to display in the Command Window.",
}];
const DISPLAY_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "display(X)",
    inputs: &DISPLAY_INPUTS_VALUE,
    outputs: &DISPLAY_OUTPUT,
}];
const DISPLAY_ERROR_NAME_UNSUPPORTED: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.DISPLAY.ARG_UNSUPPORTED",
    identifier: None,
    when: "A second argument is passed to display.",
    message: "display: the two-argument form is not yet specified",
};
const DISPLAY_ERROR_GATHER: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.DISPLAY.GATHER",
    identifier: None,
    when: "Input value cannot be gathered onto the host for rendering.",
    message: "display: failed to gather value for display",
};
const DISPLAY_ERROR_RENDER: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.DISPLAY.RENDER",
    identifier: None,
    when: "Delegated rendering of an unobserved value kind fails.",
    message: "display: failed to render value",
};
const DISPLAY_ERRORS: [BuiltinErrorDescriptor; 3] = [
    DISPLAY_ERROR_NAME_UNSUPPORTED,
    DISPLAY_ERROR_GATHER,
    DISPLAY_ERROR_RENDER,
];
pub const DISPLAY_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &DISPLAY_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &DISPLAY_ERRORS,
};

fn display_error(error: &'static BuiltinErrorDescriptor) -> crate::RuntimeError {
    display_error_with(error, error.message)
}

fn display_error_with(
    error: &'static BuiltinErrorDescriptor,
    message: impl Into<String>,
) -> crate::RuntimeError {
    let mut builder = crate::build_runtime_error(message).with_builtin("display");
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

async fn display_builtin(value: Value, rest: Vec<Value>) -> crate::BuiltinResult<Value> {
    if !rest.is_empty() {
        // FR-013-06 / recorded gate decision: display(X, name) is unapproved
        // in the export; reject rather than invent header formatting.
        return Err(display_error(&DISPLAY_ERROR_NAME_UNSUPPORTED));
    }

    let host_value = gather_if_needed_async(&value)
        .await
        .map_err(|e| display_error_with(&DISPLAY_ERROR_GATHER, format!("display: {e}")))?;

    match format_display_text(&host_value) {
        Some(text) => record_console_output(ConsoleStream::Stdout, text),
        None => {
            // Documented independent choice (spec Unresolved behaviour):
            // unobserved value kinds delegate to disp's rendering (which
            // records its text ending in exactly one newline) followed by
            // one blank line. disp itself is not modified by this feature.
            crate::call_builtin_async("disp", std::slice::from_ref(&host_value))
                .await
                .map_err(|e| display_error_with(&DISPLAY_ERROR_RENDER, format!("display: {e}")))?;
            record_console_output(ConsoleStream::Stdout, "\n");
        }
    }

    Ok(empty_return_value())
}

/// Pure formatter for the value kinds whose `display` text is derived from
/// the approved observations. Returns the COMPLETE emitted text, including
/// every trailing newline, so tests can assert byte equality directly.
/// Returns `None` for kinds outside the observed-derived surface; those
/// delegate to `disp` rendering plus a trailing blank line (documented
/// choice, non-normative).
pub(crate) fn format_display_text(value: &Value) -> Option<String> {
    match value {
        // Observed case `scalar-text`: display(42) -> "    42\n\n".
        Value::Num(n) => Some(scalar_text(&format!("{}", Value::Num(*n)))),
        // Real double tensors generalise the observed layout. 1x1 tensors
        // take the scalar path; 2-D tensors take the grid path (observed
        // case `vector-text` for 1x3). Empty and N-D tensors delegate.
        Value::Tensor(tensor) => {
            if tensor.data.len() == 1 {
                return Some(scalar_text(&format!("{}", Value::Num(tensor.data[0]))));
            }
            let dims = canonical_dims(&tensor.shape);
            if tensor.data.is_empty() || dims.len() > 2 || dims.contains(&0) {
                return None;
            }
            let rows = dims[0];
            let cols = dims.get(1).copied().unwrap_or(1);
            Some(grid_text(rows, cols, |r, c| {
                format!("{}", Value::Num(tensor.data[r + c * rows]))
            }))
        }
        // Observed case `char-text`: display('hi') -> "hi\n" — no quotes,
        // no indent, no trailing blank line. Multi-row and empty char
        // arrays are unobserved; they reuse the same rule (one line per
        // row, single trailing newline) as a documented choice.
        Value::CharArray(array) => Some(char_text(array)),
        _ => None,
    }
}

/// Right-align a scalar field to the minimum numeric width and append the
/// content newline plus the trailing blank line: `"    42\n\n"`.
fn scalar_text(field: &str) -> String {
    let width = field.chars().count().max(NUMERIC_MIN_FIELD_WIDTH);
    format!("{field:>width$}\n\n")
}

/// Render a 2-D grid as contiguous right-aligned fields (minimum width 6,
/// no separator between fields), one line per row, then a blank line:
/// `"     1     2     3\n\n"`.
fn grid_text<F>(rows: usize, cols: usize, mut field_at: F) -> String
where
    F: FnMut(usize, usize) -> String,
{
    let mut grid = vec![vec![String::new(); cols]; rows];
    let mut widths = vec![NUMERIC_MIN_FIELD_WIDTH; cols];
    for (c, width) in widths.iter_mut().enumerate() {
        for (r, row) in grid.iter_mut().enumerate() {
            let field = field_at(r, c);
            let len = field.chars().count();
            if len > *width {
                *width = len;
            }
            row[c] = field;
        }
    }

    let mut text = String::new();
    for row in &grid {
        for (c, field) in row.iter().enumerate() {
            let width = widths[c];
            text.push_str(&format!("{field:>width$}"));
        }
        text.push('\n');
    }
    text.push('\n');
    text
}

/// Char arrays: one bare line per row, single trailing newline, no blank
/// line.
fn char_text(array: &CharArray) -> String {
    if array.rows == 0 || array.cols == 0 {
        return "\n".to_string();
    }
    let mut text = String::new();
    for row in 0..array.rows {
        text.push_str(&char_row_to_string(array, row));
        text.push('\n');
    }
    text
}

fn canonical_dims(shape: &[usize]) -> Vec<usize> {
    match shape.len() {
        0 => vec![1, 1],
        1 => vec![1, shape[0]],
        _ => shape.to_vec(),
    }
}

fn empty_return_value() -> Value {
    Value::Tensor(Tensor::zeros(vec![0, 0]))
}

#[cfg(all(test, feature = "parallel_xval"))]
pub(crate) mod tests {
    use super::*;
    use crate::builtins::common::test_support;
    use runmat_builtins::{IntValue, StructValue, Tensor};

    fn captured_display(value: Value) -> String {
        crate::console::reset_thread_buffer();
        futures::executor::block_on(display_builtin(value, Vec::new())).expect("display");
        crate::console::take_thread_buffer()
            .into_iter()
            .map(|entry| entry.text)
            .collect()
    }

    #[test]
    fn display_descriptor_signatures_cover_primary_form() {
        let labels: Vec<&str> = DISPLAY_DESCRIPTOR
            .signatures
            .iter()
            .map(|sig| sig.label)
            .collect();
        assert!(labels.contains(&"display(X)"));
    }

    // FR-013-03 [display.formatted-text / scalar-text]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn display_scalar_text_matches_observed() {
        assert_eq!(
            format_display_text(&Value::Num(42.0)),
            Some("    42\n\n".to_string())
        );
    }

    // FR-013-04 [display.formatted-text / vector-text]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn display_vector_text_matches_observed() {
        let tensor = Tensor::new(vec![1.0, 2.0, 3.0], vec![1, 3]).expect("tensor");
        assert_eq!(
            format_display_text(&Value::Tensor(tensor)),
            Some("     1     2     3\n\n".to_string())
        );
    }

    // FR-013-05 [display.formatted-text / char-text]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn display_char_text_matches_observed() {
        let array = CharArray::new_row("hi");
        assert_eq!(
            format_display_text(&Value::CharArray(array)),
            Some("hi\n".to_string())
        );
    }

    // FR-013-02 + FR-013-03: end-to-end console capture, byte-exact.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn display_scalar_console_capture_matches_observed() {
        assert_eq!(captured_display(Value::Num(42.0)), "    42\n\n");
    }

    // FR-013-02 + FR-013-04: end-to-end console capture, byte-exact.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn display_vector_console_capture_matches_observed() {
        let tensor = Tensor::new(vec![1.0, 2.0, 3.0], vec![1, 3]).expect("tensor");
        assert_eq!(
            captured_display(Value::Tensor(tensor)),
            "     1     2     3\n\n"
        );
    }

    // FR-013-02 + FR-013-05: end-to-end console capture, byte-exact.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn display_char_console_capture_matches_observed() {
        let array = CharArray::new_row("hi");
        assert_eq!(captured_display(Value::CharArray(array)), "hi\n");
    }

    // FR-013-06, unresolved_choice: the two-argument form is unapproved in
    // the export and rejected per the recorded gate decision.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn display_rejects_second_argument_unresolved_choice() {
        let err = futures::executor::block_on(display_builtin(
            Value::Num(1.0),
            vec![Value::String("x".into())],
        ))
        .expect_err("expected error");
        assert!(err.contains("two-argument form"));
    }

    // FR-013-06, unresolved_choice: no variable-name header is ever emitted
    // (all observed cases pass literals and show no header).
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn display_emits_no_name_header_unresolved_choice() {
        let text = captured_display(Value::Num(42.0));
        assert!(!text.contains('='));
    }

    // FR-013-06, unresolved_choice: unobserved kinds delegate to disp's
    // rendering plus one trailing blank line.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn display_struct_delegates_to_disp_with_trailing_blank_unresolved_choice() {
        let mut sv = StructValue::new();
        sv.insert("msg", Value::String("ok".into()));
        assert_eq!(captured_display(Value::Struct(sv)), "    msg: \"ok\"\n\n");
    }

    // FR-013-06, unresolved_choice: integer scalars are unobserved and take
    // the delegation path (disp rendering + blank line).
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn display_int_delegates_to_disp_unresolved_choice() {
        assert_eq!(format_display_text(&Value::Int(IntValue::I32(7))), None);
        assert_eq!(captured_display(Value::Int(IntValue::I32(7))), "7\n\n");
    }

    // FR-013-02: GPU tensors are gathered before rendering; the gathered
    // 1x2 double takes the observed-derived grid layout.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn display_accepts_gpu_tensor() {
        test_support::with_test_provider(|provider| {
            let tensor = Tensor::new(vec![1.0, 2.0], vec![1, 2]).expect("tensor");
            let view = runmat_accelerate_api::HostTensorView {
                data: &tensor.data,
                shape: &tensor.shape,
            };
            let handle = provider.upload(&view).expect("upload");
            crate::console::reset_thread_buffer();
            let result =
                futures::executor::block_on(display_builtin(Value::GpuTensor(handle), Vec::new()))
                    .expect("display should succeed");
            assert_eq!(result, Value::Tensor(Tensor::zeros(vec![0, 0])));
            let text: String = crate::console::take_thread_buffer()
                .into_iter()
                .map(|entry| entry.text)
                .collect();
            assert_eq!(text, "     1     2\n\n");
        });
    }
}
