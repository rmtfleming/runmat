//! MATLAB-compatible `int2str` builtin for RunMat.
//!
//! Clean-room provenance: specs/007-numeric-string-conversion (spec `int2str`
//! 0.2.0, claims int2str.signature-primary, int2str.output-class,
//! int2str.rounding-and-char).
//!
//! Observed behaviour (normative): rounds each element to the nearest integer
//! with ties away from zero (`int2str(3.7)` => `'4'`, `int2str(-2.5)` =>
//! `'-3'`) and returns a char array whose rows mirror the input rows;
//! multi-element inputs are space-separated (`int2str([1.2 3.8 5.5])` =>
//! `'1  4  6'`, 1x7). Column alignment beyond the observed single-digit cases
//! is an independent choice (unresolved question int2str.q-spacing):
//! right-aligned columns joined by two spaces, all rows equal width.

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    CharArray, Tensor, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::map_control_flow_with_builtin;
use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ShapeRequirements,
};
use crate::builtins::common::tensor;
use crate::builtins::strings::type_resolvers::string_scalar_type;
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const BUILTIN_NAME: &str = "int2str";

#[runmat_macros::register_gpu_spec(builtin_path = "crate::builtins::strings::core::int2str")]
pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "int2str",
    op_kind: GpuOpKind::Custom("conversion"),
    supported_precisions: &[],
    broadcast: BroadcastSemantics::None,
    provider_hooks: &[],
    constant_strategy: ConstantStrategy::InlineLiteral,
    residency: ResidencyPolicy::GatherImmediately,
    nan_mode: ReductionNaN::Include,
    two_pass_threshold: None,
    workgroup_size: None,
    accepts_nan_mode: false,
    notes: "Always gathers GPU data to host memory before rounding and formatting integer text.",
};

#[runmat_macros::register_fusion_spec(builtin_path = "crate::builtins::strings::core::int2str")]
pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "int2str",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes:
        "Conversion builtin; not eligible for fusion and always materialises host character arrays.",
};

const INT2STR_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "str",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Character array containing the rounded integer values.",
}];

const INT2STR_INPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "X",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Numeric or logical scalar/vector/matrix input.",
}];

const INT2STR_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "str = int2str(X)",
    inputs: &INT2STR_INPUT,
    outputs: &INT2STR_OUTPUT,
}];

const INT2STR_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.INT2STR.INVALID_INPUT",
    identifier: Some("RunMat:int2str:InvalidInput"),
    when: "Input value is not a supported numeric/logical scalar, vector, or matrix.",
    message: "int2str: unsupported input type",
};

const INT2STR_ERROR_INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.INT2STR.INTERNAL",
    identifier: Some("RunMat:int2str:InternalError"),
    when: "Internal char-array assembly failed.",
    message: "int2str: internal error",
};

const INT2STR_ERRORS: [BuiltinErrorDescriptor; 2] =
    [INT2STR_ERROR_INVALID_INPUT, INT2STR_ERROR_INTERNAL];

pub const INT2STR_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &INT2STR_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &INT2STR_ERRORS,
};

fn int2str_error(error: &'static BuiltinErrorDescriptor) -> RuntimeError {
    int2str_error_with_message(error.message, error)
}

fn int2str_error_with_message(
    message: impl Into<String>,
    error: &'static BuiltinErrorDescriptor,
) -> RuntimeError {
    let mut builder = build_runtime_error(message).with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

fn remap_int2str_flow(err: RuntimeError) -> RuntimeError {
    map_control_flow_with_builtin(err, BUILTIN_NAME)
}

#[runtime_builtin(
    name = "int2str",
    category = "strings/core",
    summary = "Round numeric values to the nearest integers and convert them to a character array.",
    keywords = "int2str,integer to string,round,format",
    examples = "txt = int2str([1.2 3.8 5.5]);",
    type_resolver(string_scalar_type),
    descriptor(crate::builtins::strings::core::int2str::INT2STR_DESCRIPTOR),
    builtin_path = "crate::builtins::strings::core::int2str"
)]
async fn int2str_builtin(value: Value) -> BuiltinResult<Value> {
    let gathered = gather_if_needed_async(&value)
        .await
        .map_err(remap_int2str_flow)?;
    let (data, rows, cols) = extract_real_data(gathered)?;
    let char_array = format_rounded_matrix(&data, rows, cols)?;
    Ok(Value::CharArray(char_array))
}

/// Extract real numeric data (column-major) plus its 2-D extent. Complex,
/// text, cell, and struct inputs are rejected with a documented independent
/// choice: the approved export only observes real numeric inputs.
fn extract_real_data(value: Value) -> BuiltinResult<(Vec<f64>, usize, usize)> {
    match value {
        Value::Num(n) => Ok((vec![n], 1, 1)),
        Value::Int(i) => Ok((vec![i.to_f64()], 1, 1)),
        Value::Bool(b) => Ok((vec![if b { 1.0 } else { 0.0 }], 1, 1)),
        Value::Tensor(t) => tensor_to_real_data(t),
        Value::LogicalArray(la) => {
            let tensor = tensor::logical_to_tensor(&la)
                .map_err(|_| int2str_error(&INT2STR_ERROR_INVALID_INPUT))?;
            tensor_to_real_data(tensor)
        }
        other => Err(int2str_error_with_message(
            format!(
                "{} {:?}; expected real numeric or logical values",
                INT2STR_ERROR_INVALID_INPUT.message, other
            ),
            &INT2STR_ERROR_INVALID_INPUT,
        )),
    }
}

fn tensor_to_real_data(tensor: Tensor) -> BuiltinResult<(Vec<f64>, usize, usize)> {
    if tensor.shape.len() > 2 {
        return Err(int2str_error_with_message(
            "int2str: input must be scalar, vector, or 2-D matrix",
            &INT2STR_ERROR_INVALID_INPUT,
        ));
    }
    let rows = tensor.rows();
    let cols = tensor.cols();
    Ok((tensor.data, rows, cols))
}

/// Round each element (ties away from zero, per int2str.rounding-and-char),
/// format per column, and join columns with two spaces. Columns are
/// right-aligned to the widest entry so every row has equal width; the
/// observed 1x7 (`'1  4  6'`) and 2x4 (`'1  3'` / `'4  4'`) extents follow
/// from this layout.
fn format_rounded_matrix(data: &[f64], rows: usize, cols: usize) -> BuiltinResult<CharArray> {
    if rows == 0 || cols == 0 {
        return CharArray::new(Vec::new(), 0, 0)
            .map_err(|_| int2str_error(&INT2STR_ERROR_INTERNAL));
    }

    let mut texts = vec![String::new(); rows * cols];
    let mut col_widths = vec![0usize; cols];
    for (col, col_width) in col_widths.iter_mut().enumerate() {
        for row in 0..rows {
            let idx = row + col * rows;
            let text = format_rounded_value(data.get(idx).copied().unwrap_or(0.0));
            let width = text.chars().count();
            if width > *col_width {
                *col_width = width;
            }
            texts[idx] = text;
        }
    }

    let width: usize = col_widths.iter().sum::<usize>() + 2 * (cols - 1);
    let mut chars = Vec::with_capacity(rows * width);
    for row in 0..rows {
        for (col, col_width) in col_widths.iter().enumerate() {
            if col > 0 {
                chars.extend([' ', ' ']);
            }
            let text = &texts[row + col * rows];
            let pad = col_width.saturating_sub(text.chars().count());
            chars.extend(std::iter::repeat_n(' ', pad));
            chars.extend(text.chars());
        }
    }

    CharArray::new(chars, rows, width).map_err(|_| int2str_error(&INT2STR_ERROR_INTERNAL))
}

/// Round to the nearest integer with ties away from zero (Rust's
/// `f64::round` semantics match the observed -2.5 -> -3 and 5.5 -> 6).
/// Non-finite values keep word spellings (independent choice; the approved
/// export records no NaN/Inf observations).
fn format_rounded_value(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    }
    if value.is_infinite() {
        return if value.is_sign_negative() {
            "-Inf".to_string()
        } else {
            "Inf".to_string()
        };
    }
    let rounded = value.round();
    if rounded == 0.0 {
        return "0".to_string();
    }
    format!("{rounded:.0}")
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::builtins::common::test_support;
    use runmat_builtins::{IntValue, LogicalArray, ResolveContext, Type};

    fn run_int2str(value: Value) -> Value {
        futures::executor::block_on(super::int2str_builtin(value)).expect("int2str")
    }

    fn char_rows(ca: &CharArray) -> Vec<String> {
        ca.data
            .chunks(ca.cols.max(1))
            .map(|chunk| chunk.iter().collect())
            .collect()
    }

    // Normative [FR-007-01, FR-007-02; int2str.signature-primary,
    // int2str.output-class, int2str.rounding-and-char]
    // Observed case `scalar`: int2str(3.7) => '4' (char, 1x1).
    #[test]
    fn observed_scalar_rounds_to_nearest_integer() {
        let out = run_int2str(Value::Num(3.7));
        match out {
            Value::CharArray(ca) => {
                assert_eq!((ca.rows, ca.cols), (1, 1));
                assert_eq!(ca.data.iter().collect::<String>(), "4");
            }
            other => panic!("expected char array, got {other:?}"),
        }
    }

    // Normative [FR-007-02; int2str.rounding-and-char, int2str.output-class]
    // Observed case `negative`: int2str(-2.5) => '-3' (char, 1x2) — ties
    // round away from zero.
    #[test]
    fn observed_negative_half_rounds_away_from_zero() {
        let out = run_int2str(Value::Num(-2.5));
        match out {
            Value::CharArray(ca) => {
                assert_eq!((ca.rows, ca.cols), (1, 2));
                assert_eq!(ca.data.iter().collect::<String>(), "-3");
            }
            other => panic!("expected char array, got {other:?}"),
        }
    }

    // Normative [FR-007-02, FR-007-03; int2str.rounding-and-char]
    // Observed case `vector`: int2str([1.2 3.8 5.5]) => '1  4  6' (char, 1x7).
    #[test]
    fn observed_vector_elements_joined_with_two_spaces() {
        let tensor = Tensor::new(vec![1.2, 3.8, 5.5], vec![1, 3]).expect("tensor");
        let out = run_int2str(Value::Tensor(tensor));
        match out {
            Value::CharArray(ca) => {
                assert_eq!((ca.rows, ca.cols), (1, 7));
                assert_eq!(ca.data.iter().collect::<String>(), "1  4  6");
            }
            other => panic!("expected char array, got {other:?}"),
        }
    }

    // Normative [FR-007-02, FR-007-03; int2str.rounding-and-char,
    // int2str.output-class]
    // Observed case `matrix`: int2str([1.1 2.9; 3.5 4.4]) => ['1  3';'4  4']
    // (char, 2x4) — output rows mirror input rows.
    #[test]
    fn observed_matrix_rows_mirror_input_rows() {
        // Column-major data for [1.1 2.9; 3.5 4.4].
        let tensor = Tensor::new(vec![1.1, 3.5, 2.9, 4.4], vec![2, 2]).expect("tensor");
        let out = run_int2str(Value::Tensor(tensor));
        match out {
            Value::CharArray(ca) => {
                assert_eq!((ca.rows, ca.cols), (2, 4));
                assert_eq!(char_rows(&ca), vec!["1  3".to_string(), "4  4".to_string()]);
            }
            other => panic!("expected char array, got {other:?}"),
        }
    }

    // unresolved_choice (int2str.q-spacing): multi-width column alignment is
    // unobserved; the documented choice is right-aligned columns joined by
    // two spaces, so all rows share one width.
    #[test]
    fn unresolved_choice_columns_right_aligned_two_space_separator() {
        // Column-major data for [5 123; 42 7].
        let tensor = Tensor::new(vec![5.0, 42.0, 123.0, 7.0], vec![2, 2]).expect("tensor");
        let out = run_int2str(Value::Tensor(tensor));
        match out {
            Value::CharArray(ca) => {
                assert_eq!((ca.rows, ca.cols), (2, 7));
                assert_eq!(
                    char_rows(&ca),
                    vec![" 5  123".to_string(), "42    7".to_string()]
                );
            }
            other => panic!("expected char array, got {other:?}"),
        }
    }

    // unresolved_choice: NaN/Inf inputs are unobserved in the approved
    // export; documented choice keeps word spellings.
    #[test]
    fn unresolved_choice_nonfinite_values_keep_word_spellings() {
        let tensor = Tensor::new(vec![f64::NAN, f64::INFINITY, f64::NEG_INFINITY], vec![1, 3])
            .expect("tensor");
        let out = run_int2str(Value::Tensor(tensor));
        match out {
            Value::CharArray(ca) => {
                assert_eq!(ca.rows, 1);
                assert_eq!(ca.data.iter().collect::<String>(), "NaN  Inf  -Inf");
            }
            other => panic!("expected char array, got {other:?}"),
        }
    }

    // unresolved_choice: empty input unobserved; documented choice returns a
    // 0x0 char array.
    #[test]
    fn unresolved_choice_empty_input_returns_empty_char() {
        let tensor = Tensor::new(Vec::new(), vec![0, 0]).expect("tensor");
        let out = run_int2str(Value::Tensor(tensor));
        match out {
            Value::CharArray(ca) => {
                assert_eq!((ca.rows, ca.cols), (0, 0));
                assert!(ca.data.is_empty());
            }
            other => panic!("expected char array, got {other:?}"),
        }
    }

    // unresolved_choice: integer-typed and logical inputs are unobserved;
    // documented choice treats them like their numeric values.
    #[test]
    fn unresolved_choice_integer_and_logical_inputs_format_numerically() {
        let out = run_int2str(Value::Int(IntValue::I32(7)));
        match out {
            Value::CharArray(ca) => assert_eq!(ca.data.iter().collect::<String>(), "7"),
            other => panic!("expected char array, got {other:?}"),
        }

        let logical = LogicalArray::new(vec![1, 0, 1], vec![1, 3]).expect("logical");
        let out = run_int2str(Value::LogicalArray(logical));
        match out {
            Value::CharArray(ca) => assert_eq!(ca.data.iter().collect::<String>(), "1  0  1"),
            other => panic!("expected char array, got {other:?}"),
        }
    }

    // unresolved_choice: non-numeric input classes are unobserved; documented
    // choice rejects them with a stable identifier.
    #[test]
    fn unresolved_choice_non_numeric_input_errors() {
        let err =
            futures::executor::block_on(super::int2str_builtin(Value::String("hello".to_string())))
                .expect_err("int2str should reject text input");
        assert_eq!(err.identifier(), Some("RunMat:int2str:InvalidInput"));
    }

    // GPU inputs are gathered to the host before formatting (Principle IX;
    // residency policy GatherImmediately).
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn int2str_gpu_tensor_roundtrip() {
        test_support::with_test_provider(|provider| {
            let tensor = Tensor::new(vec![1.2, 3.8], vec![1, 2]).expect("tensor");
            let view = runmat_accelerate_api::HostTensorView {
                data: &tensor.data,
                shape: &tensor.shape,
            };
            let handle = provider.upload(&view).expect("upload");
            let out = run_int2str(Value::GpuTensor(handle));
            match out {
                Value::CharArray(ca) => {
                    assert_eq!(ca.data.iter().collect::<String>(), "1  4");
                }
                other => panic!("expected char array, got {other:?}"),
            }
        });
    }

    // Registration is discoverable through the builtin registry (SC-007-2).
    #[test]
    fn int2str_is_registered() {
        assert!(runmat_builtins::builtin_function_by_name("int2str").is_some());
    }

    #[test]
    fn int2str_type_is_string_scalar() {
        assert_eq!(
            string_scalar_type(&[Type::Num], &ResolveContext::new(Vec::new())),
            Type::String
        );
    }
}
