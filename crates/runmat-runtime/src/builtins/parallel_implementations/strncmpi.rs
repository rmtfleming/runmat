// Parallel clean-room implementation of `strncmpi` (independently developed against the
// matlab-interface-spec black-box specs). 0-6-0 ships the DEFAULT implementation; this copy is
// retained unregistered for differential cross-validation. Original path: crates/runmat-runtime/src/builtins/strings/core/strncmpi.rs
//! MATLAB-compatible `strncmpi` builtin for RunMat (case-insensitive prefix
//! comparison).
//!
//! Clean-room implementation for SpecKit feature `020-strncmpi` from the
//! approved `strncmpi` 0.2.0 export. Structure and array semantics mirror
//! `strncmp`; character comparison applies the lowercase fold used by
//! `strcmpi`.

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::broadcast::{broadcast_index, broadcast_shapes, compute_strides};
use crate::builtins::common::map_control_flow_with_builtin;
use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ShapeRequirements,
};
use crate::builtins::common::tensor;
use crate::builtins::strings::search::text_utils::{logical_result, TextCollection, TextElement};
use crate::builtins::strings::type_resolvers::logical_text_match_type;
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const FN_NAME: &str = "strncmpi";

const STRNCMPI_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "tf",
    ty: BuiltinParamType::LogicalArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Logical case-insensitive prefix-comparison result.",
}];

const STRNCMPI_INPUTS: [BuiltinParamDescriptor; 3] = [
    BuiltinParamDescriptor {
        name: "A",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "First text input (string/char/cell/string array).",
    },
    BuiltinParamDescriptor {
        name: "B",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Second text input (string/char/cell/string array).",
    },
    BuiltinParamDescriptor {
        name: "N",
        ty: BuiltinParamType::IntegerScalar,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Prefix length to compare.",
    },
];

const STRNCMPI_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "tf = strncmpi(A, B, N)",
    inputs: &STRNCMPI_INPUTS,
    outputs: &STRNCMPI_OUTPUT,
}];

const STRNCMPI_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.STRNCMPI.INVALID_INPUT",
    identifier: Some("RunMat:strncmpi:InvalidInput"),
    when: "At least one text input is not a supported text container.",
    message: "strncmpi: text inputs must be string/char/cell/string-array values",
};

const STRNCMPI_ERROR_SHAPE_MISMATCH: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.STRNCMPI.SHAPE_MISMATCH",
    identifier: Some("RunMat:strncmpi:ShapeMismatch"),
    when: "Text inputs are not broadcast-compatible.",
    message: "strncmpi: input sizes are not broadcast-compatible",
};

const STRNCMPI_ERROR_INVALID_PREFIX_LENGTH: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.STRNCMPI.INVALID_PREFIX_LENGTH",
    identifier: Some("RunMat:strncmpi:InvalidPrefixLength"),
    when: "Prefix length argument is not a finite nonnegative integer scalar.",
    message: "strncmpi: prefix length must be a finite nonnegative integer scalar",
};

const STRNCMPI_ERROR_INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.STRNCMPI.INTERNAL",
    identifier: Some("RunMat:strncmpi:InternalError"),
    when: "Internal logical result assembly failed.",
    message: "strncmpi: internal error",
};

const STRNCMPI_ERRORS: [BuiltinErrorDescriptor; 4] = [
    STRNCMPI_ERROR_INVALID_INPUT,
    STRNCMPI_ERROR_SHAPE_MISMATCH,
    STRNCMPI_ERROR_INVALID_PREFIX_LENGTH,
    STRNCMPI_ERROR_INTERNAL,
];

pub const STRNCMPI_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &STRNCMPI_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &STRNCMPI_ERRORS,
};

pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "strncmpi",
    op_kind: GpuOpKind::Custom("string-prefix-compare-fold"),
    supported_precisions: &[],
    broadcast: BroadcastSemantics::Matlab,
    provider_hooks: &[],
    constant_strategy: ConstantStrategy::InlineLiteral,
    residency: ResidencyPolicy::GatherImmediately,
    nan_mode: ReductionNaN::Include,
    two_pass_threshold: None,
    workgroup_size: None,
    accepts_nan_mode: false,
    notes: "Performs host-side case-folded prefix comparisons; GPU inputs are gathered before evaluation.",
};

pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "strncmpi",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Produces logical host results and is not eligible for GPU fusion.",
};

fn strncmpi_error(error: &'static BuiltinErrorDescriptor) -> RuntimeError {
    strncmpi_error_with_message(error.message, error)
}

fn strncmpi_error_with_message(
    message: impl Into<String>,
    error: &'static BuiltinErrorDescriptor,
) -> RuntimeError {
    let mut builder = build_runtime_error(message).with_builtin(FN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

fn remap_strncmpi_flow(err: RuntimeError) -> RuntimeError {
    map_control_flow_with_builtin(err, FN_NAME)
}

async fn strncmpi_builtin(a: Value, b: Value, n: Value) -> crate::BuiltinResult<Value> {
    let a = gather_if_needed_async(&a)
        .await
        .map_err(remap_strncmpi_flow)?;
    let b = gather_if_needed_async(&b)
        .await
        .map_err(remap_strncmpi_flow)?;
    let n = gather_if_needed_async(&n)
        .await
        .map_err(remap_strncmpi_flow)?;

    let limit = parse_prefix_length(n)?;
    let left = TextCollection::from_argument(FN_NAME, a, "first argument")
        .map_err(|_| strncmpi_error(&STRNCMPI_ERROR_INVALID_INPUT))?;
    let right = TextCollection::from_argument(FN_NAME, b, "second argument")
        .map_err(|_| strncmpi_error(&STRNCMPI_ERROR_INVALID_INPUT))?;
    evaluate_strncmpi(&left, &right, limit)
}

fn evaluate_strncmpi(
    left: &TextCollection,
    right: &TextCollection,
    limit: usize,
) -> BuiltinResult<Value> {
    let shape = broadcast_shapes(FN_NAME, &left.shape, &right.shape)
        .map_err(|_| strncmpi_error(&STRNCMPI_ERROR_SHAPE_MISMATCH))?;
    let total = tensor::element_count(&shape);
    if total == 0 {
        return logical_result(FN_NAME, Vec::new(), shape)
            .map_err(|_| strncmpi_error(&STRNCMPI_ERROR_INTERNAL));
    }

    let left_strides = compute_strides(&left.shape);
    let right_strides = compute_strides(&right.shape);
    let mut data = Vec::with_capacity(total);

    for linear in 0..total {
        let li = broadcast_index(linear, &shape, &left.shape, &left_strides);
        let ri = broadcast_index(linear, &shape, &right.shape, &right_strides);
        let equal = if limit == 0 {
            true
        } else {
            match (&left.elements[li], &right.elements[ri]) {
                (TextElement::Missing, _) | (_, TextElement::Missing) => false,
                (TextElement::Text(lhs), TextElement::Text(rhs)) => {
                    prefix_equal_ignore_case(lhs, rhs, limit)
                }
            }
        };
        data.push(if equal { 1 } else { 0 });
    }

    logical_result(FN_NAME, data, shape).map_err(|_| strncmpi_error(&STRNCMPI_ERROR_INTERNAL))
}

fn prefix_equal_ignore_case(lhs: &str, rhs: &str, limit: usize) -> bool {
    if limit == 0 {
        return true;
    }
    let mut lhs_iter = lhs.chars();
    let mut rhs_iter = rhs.chars();
    let mut compared = 0usize;

    while compared < limit {
        let left_char = lhs_iter.next();
        let right_char = rhs_iter.next();
        match (left_char, right_char) {
            (Some(lc), Some(rc)) => {
                if !chars_equal_ignore_case(lc, rc) {
                    return false;
                }
            }
            (None, Some(_)) | (Some(_), None) => {
                return false;
            }
            (None, None) => {
                return true;
            }
        }
        compared += 1;
    }

    true
}

/// Case fold consistent with `strcmpi`, which lowercases whole texts via
/// `str::to_lowercase`; here the same Unicode lowercase mapping is applied
/// per character.
fn chars_equal_ignore_case(lc: char, rc: char) -> bool {
    lc == rc || lc.to_lowercase().eq(rc.to_lowercase())
}

fn parse_prefix_length(value: Value) -> BuiltinResult<usize> {
    match value {
        Value::Int(i) => {
            let raw = i.to_i64();
            if raw < 0 {
                return Err(strncmpi_error(&STRNCMPI_ERROR_INVALID_PREFIX_LENGTH));
            }
            Ok(raw as usize)
        }
        Value::Num(n) => parse_prefix_length_from_float(n),
        Value::Bool(b) => Ok(if b { 1 } else { 0 }),
        Value::Tensor(tensor) => {
            if tensor.data.len() != 1 {
                return Err(strncmpi_error(&STRNCMPI_ERROR_INVALID_PREFIX_LENGTH));
            }
            parse_prefix_length_from_float(tensor.data[0])
        }
        Value::LogicalArray(array) => {
            if array.data.len() != 1 {
                return Err(strncmpi_error(&STRNCMPI_ERROR_INVALID_PREFIX_LENGTH));
            }
            Ok(if array.data[0] != 0 { 1 } else { 0 })
        }
        _ => Err(strncmpi_error(&STRNCMPI_ERROR_INVALID_PREFIX_LENGTH)),
    }
}

fn parse_prefix_length_from_float(value: f64) -> BuiltinResult<usize> {
    if !value.is_finite() {
        return Err(strncmpi_error(&STRNCMPI_ERROR_INVALID_PREFIX_LENGTH));
    }
    if value < 0.0 {
        return Err(strncmpi_error(&STRNCMPI_ERROR_INVALID_PREFIX_LENGTH));
    }
    let rounded = value.round();
    if (rounded - value).abs() > f64::EPSILON {
        return Err(strncmpi_error(&STRNCMPI_ERROR_INVALID_PREFIX_LENGTH));
    }
    if rounded > (usize::MAX as f64) {
        return Err(strncmpi_error(&STRNCMPI_ERROR_INVALID_PREFIX_LENGTH));
    }
    Ok(rounded as usize)
}

#[cfg(all(test, feature = "parallel_xval"))]
pub(crate) mod tests {
    use super::*;
    #[cfg(feature = "wgpu")]
    use runmat_accelerate_api::AccelProvider;
    use runmat_builtins::{
        CellArray, CharArray, IntValue, LogicalArray, ResolveContext, StringArray, Tensor, Type,
    };

    fn strncmpi_builtin(a: Value, b: Value, n: Value) -> BuiltinResult<Value> {
        futures::executor::block_on(super::strncmpi_builtin(a, b, n))
    }

    fn error_message(err: crate::RuntimeError) -> String {
        err.to_string()
    }

    // --- Observed tier: normative, reproduces the approved export cases ---

    /// Export case `match`: strncmpi('Hello', 'help', 3) -> 1x1 logical true.
    /// [strncmpi.signature-primary, strncmpi.output-class,
    /// strncmpi.case-insensitive-prefix; FR-020-01..03]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn observed_match_first_three_characters_ignoring_case_true() {
        let result = strncmpi_builtin(
            Value::String("Hello".into()),
            Value::String("help".into()),
            Value::Int(IntValue::I32(3)),
        )
        .expect("strncmpi");
        assert_eq!(result, Value::Bool(true));
    }

    /// Export case `prefix-equal`: strncmpi('abc', 'abd', 2) -> 1x1 logical
    /// true. [strncmpi.output-class, strncmpi.case-insensitive-prefix;
    /// FR-020-02, FR-020-03]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn observed_prefix_equal_first_two_characters_true() {
        let result = strncmpi_builtin(
            Value::String("abc".into()),
            Value::String("abd".into()),
            Value::Int(IntValue::I32(2)),
        )
        .expect("strncmpi");
        assert_eq!(result, Value::Bool(true));
    }

    /// Export case `no-match`: strncmpi('abc', 'xyz', 1) -> 1x1 logical
    /// false. [strncmpi.output-class, strncmpi.case-insensitive-prefix;
    /// FR-020-02, FR-020-03]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn observed_no_match_first_character_false() {
        let result = strncmpi_builtin(
            Value::String("abc".into()),
            Value::String("xyz".into()),
            Value::Int(IntValue::I32(1)),
        )
        .expect("strncmpi");
        assert_eq!(result, Value::Bool(false));
    }

    // --- Unresolved-choice tier: documented independent choices mirroring
    // RunMat's strncmp/strcmpi; not asserted as MATLAB-conformant.
    // [FR-020-04; strncmpi.q-arrays where noted] ---

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_mismatch_within_prefix_false() {
        let result = strncmpi_builtin(
            Value::String("RunMat".into()),
            Value::String("runway".into()),
            Value::Int(IntValue::I32(4)),
        )
        .expect("strncmpi");
        assert_eq!(result, Value::Bool(false));
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_shorter_string_within_prefix_false() {
        let result = strncmpi_builtin(
            Value::String("cat".into()),
            Value::String("CATER".into()),
            Value::Int(IntValue::I32(4)),
        )
        .expect("strncmpi");
        assert_eq!(result, Value::Bool(false));
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_both_exhausted_before_limit_true() {
        let result = strncmpi_builtin(
            Value::String("cat".into()),
            Value::String("CAT".into()),
            Value::Int(IntValue::I32(10)),
        )
        .expect("strncmpi");
        assert_eq!(result, Value::Bool(true));
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_zero_length_always_true() {
        let result = strncmpi_builtin(
            Value::String("alpha".into()),
            Value::String("omega".into()),
            Value::Num(0.0),
        )
        .expect("strncmpi");
        assert_eq!(result, Value::Bool(true));
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_prefix_length_bool_true_compares_first_character() {
        let result = strncmpi_builtin(
            Value::String("Alpha".into()),
            Value::String("array".into()),
            Value::Bool(true),
        )
        .expect("strncmpi");
        assert_eq!(result, Value::Bool(true));
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_prefix_length_bool_false_treated_as_zero() {
        let result = strncmpi_builtin(
            Value::String("alpha".into()),
            Value::String("omega".into()),
            Value::Bool(false),
        )
        .expect("strncmpi");
        assert_eq!(result, Value::Bool(true));
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_prefix_length_logical_array_scalar() {
        let logical = LogicalArray::new(vec![1], vec![1]).unwrap();
        let result = strncmpi_builtin(
            Value::String("Beta".into()),
            Value::String("theta".into()),
            Value::LogicalArray(logical),
        )
        .expect("strncmpi");
        assert_eq!(result, Value::Bool(false));
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_prefix_length_tensor_scalar_double() {
        let limit = Tensor::new(vec![2.0], vec![1, 1]).unwrap();
        let result = strncmpi_builtin(
            Value::String("GAMMA".into()),
            Value::String("gamut".into()),
            Value::Tensor(limit),
        )
        .expect("strncmpi");
        assert_eq!(result, Value::Bool(true));
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_char_array_rows_casefold() {
        let chars = CharArray::new(
            vec![
                'C', 'A', 'T', ' ', ' ', 'c', 'a', 'm', 'e', 'l', 'C', 'o', 'w', ' ', ' ',
            ],
            3,
            5,
        )
        .unwrap();
        let result = strncmpi_builtin(
            Value::CharArray(chars),
            Value::String("ca".into()),
            Value::Int(IntValue::I32(2)),
        )
        .expect("strncmpi");
        let expected = LogicalArray::new(vec![1, 1, 0], vec![3, 1]).unwrap();
        assert_eq!(result, Value::LogicalArray(expected));
    }

    /// [strncmpi.q-arrays] Cell-array behaviour is unobserved in the export;
    /// element-wise broadcasting mirrors strncmp with case folding.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_cell_arrays_broadcast_casefold() {
        let left = CellArray::new(
            vec![
                Value::from("Red"),
                Value::from("GREEN"),
                Value::from("blue"),
            ],
            1,
            3,
        )
        .unwrap();
        let right = CellArray::new(
            vec![
                Value::from("rose"),
                Value::from("gray"),
                Value::from("BLACK"),
            ],
            1,
            3,
        )
        .unwrap();
        let result = strncmpi_builtin(
            Value::Cell(left),
            Value::Cell(right),
            Value::Int(IntValue::I32(2)),
        )
        .expect("strncmpi");
        let expected = LogicalArray::new(vec![0, 1, 1], vec![1, 3]).unwrap();
        assert_eq!(result, Value::LogicalArray(expected));
    }

    /// [strncmpi.q-arrays] String-array behaviour is unobserved in the
    /// export; broadcasting mirrors strncmp with case folding.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_string_array_broadcast_scalar_casefold() {
        let strings = StringArray::new(
            vec!["North".into(), "south".into(), "east".into()],
            vec![1, 3],
        )
        .unwrap();
        let result = strncmpi_builtin(
            Value::StringArray(strings),
            Value::String("NO".into()),
            Value::Int(IntValue::I32(2)),
        )
        .expect("strncmpi");
        let expected = LogicalArray::new(vec![1, 0, 0], vec![1, 3]).unwrap();
        assert_eq!(result, Value::LogicalArray(expected));
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_missing_string_false_when_prefix_positive() {
        let strings =
            StringArray::new(vec!["<missing>".into(), "Value".into()], vec![1, 2]).unwrap();
        let result = strncmpi_builtin(
            Value::StringArray(strings),
            Value::String("VAL".into()),
            Value::Int(IntValue::I32(3)),
        )
        .expect("strncmpi");
        let expected = LogicalArray::new(vec![0, 1], vec![1, 2]).unwrap();
        assert_eq!(result, Value::LogicalArray(expected));
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_missing_zero_length_true() {
        let strings = StringArray::new(vec!["<missing>".into()], vec![1, 1]).unwrap();
        let result = strncmpi_builtin(
            Value::StringArray(strings),
            Value::String("anything".into()),
            Value::Int(IntValue::I32(0)),
        )
        .expect("strncmpi");
        assert_eq!(result, Value::Bool(true));
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_size_mismatch_error() {
        let left = StringArray::new(vec!["a".into(), "b".into()], vec![2, 1]).unwrap();
        let right = StringArray::new(vec!["a".into(), "b".into(), "c".into()], vec![3, 1]).unwrap();
        let err = error_message(
            strncmpi_builtin(
                Value::StringArray(left),
                Value::StringArray(right),
                Value::Int(IntValue::I32(1)),
            )
            .expect_err("size mismatch"),
        );
        assert!(err.contains(STRNCMPI_ERROR_SHAPE_MISMATCH.message));
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_invalid_length_type_errors() {
        let err = error_message(
            strncmpi_builtin(
                Value::String("abc".into()),
                Value::String("abc".into()),
                Value::String("3".into()),
            )
            .expect_err("invalid prefix length"),
        );
        assert!(err.contains("prefix length"));
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_negative_length_errors() {
        let err = error_message(
            strncmpi_builtin(
                Value::String("abc".into()),
                Value::String("abc".into()),
                Value::Num(-1.0),
            )
            .expect_err("negative length"),
        );
        assert!(err.to_ascii_lowercase().contains("nonnegative"));
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_invalid_argument_type_errors() {
        let err = error_message(
            strncmpi_builtin(
                Value::Num(1.0),
                Value::String("a".into()),
                Value::Int(IntValue::I32(1)),
            )
            .expect_err("invalid type"),
        );
        assert!(err.contains(STRNCMPI_ERROR_INVALID_INPUT.message));
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    #[cfg(feature = "wgpu")]
    fn strncmpi_prefix_length_from_gpu_tensor() {
        use runmat_accelerate::backend::wgpu::provider::{
            register_wgpu_provider, WgpuProviderOptions,
        };
        use runmat_accelerate_api::HostTensorView;

        let provider = match register_wgpu_provider(WgpuProviderOptions::default()) {
            Ok(provider) => provider,
            Err(_) => return,
        };
        let tensor = Tensor::new(vec![3.0], vec![1, 1]).unwrap();
        let view = HostTensorView {
            data: &tensor.data,
            shape: &tensor.shape,
        };
        let handle = provider.upload(&view).expect("upload prefix length to GPU");
        let result = strncmpi_builtin(
            Value::String("Delta".into()),
            Value::String("dELuge".into()),
            Value::GpuTensor(handle.clone()),
        )
        .expect("strncmpi");
        assert_eq!(result, Value::Bool(true));
        let _ = provider.free(&handle);
    }

    #[test]
    fn strncmpi_type_is_logical_match() {
        assert_eq!(
            logical_text_match_type(
                &[Type::String, Type::String],
                &ResolveContext::new(Vec::new()),
            ),
            Type::Bool
        );
    }
}
