// Parallel clean-room implementation of `dec2bin` (independently developed against the
// matlab-interface-spec black-box specs). 0-6-0 ships the DEFAULT implementation; this copy is
// retained unregistered for differential cross-validation. Original path: crates/runmat-runtime/src/builtins/strings/core/dec2bin.rs
//! MATLAB-compatible `dec2bin` builtin for RunMat.
//!
//! Clean-room provenance: specs/018-dec2bin (spec `dec2bin` 0.2.0, claims
//! dec2bin.signature-primary, dec2bin.output-class, dec2bin.binary-char).
//!
//! Observed behaviour (normative): converts a nonnegative integer to a char
//! row of binary digits (`dec2bin(5)` => `'101'`, 1x3); an optional width
//! argument zero-pads on the left up to at least that many digits
//! (`dec2bin(5, 8)` => `'00000101'`, 1x8) and never truncates below the
//! natural width (`dec2bin(5, 2)` => `'101'`, 1x3); zero yields `'0'` (1x1).
//!
//! Unobserved input classes are handled by documented independent choice
//! (unresolved questions dec2bin.q-vector, dec2bin.q-negative): multi-element
//! inputs produce a char matrix with one row per element (column-major
//! element order) zero-padded to a common width; non-integer values round to
//! the nearest integer (ties away from zero); negative and non-finite values
//! are rejected.

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

const BUILTIN_NAME: &str = "dec2bin";

/// Largest integer that `f64` represents exactly (2^53). Inputs above this
/// are rejected because their binary digits would not be trustworthy
/// (documented independent choice; unobserved in the approved export).
const MAX_EXACT_INTEGER: f64 = 9007199254740992.0;

pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "dec2bin",
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
    notes: "Always gathers GPU data to host memory before formatting binary digit text.",
};

pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "dec2bin",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes:
        "Conversion builtin; not eligible for fusion and always materialises host character arrays.",
};

const DEC2BIN_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "str",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Character array of binary digits.",
}];

const DEC2BIN_INPUT_D: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "D",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Nonnegative integer value(s).",
}];

const DEC2BIN_INPUT_WIDTH: [BuiltinParamDescriptor; 2] = [
    BuiltinParamDescriptor {
        name: "D",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Nonnegative integer value(s).",
    },
    BuiltinParamDescriptor {
        name: "n",
        ty: BuiltinParamType::IntegerScalar,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Minimum number of binary digits (zero-pads, never truncates).",
    },
];

const DEC2BIN_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "str = dec2bin(D)",
        inputs: &DEC2BIN_INPUT_D,
        outputs: &DEC2BIN_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "str = dec2bin(D, n)",
        inputs: &DEC2BIN_INPUT_WIDTH,
        outputs: &DEC2BIN_OUTPUT,
    },
];

const DEC2BIN_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.DEC2BIN.INVALID_INPUT",
    identifier: Some("RunMat:dec2bin:InvalidInput"),
    when: "Input is not a nonnegative, finite numeric/logical value (or exceeds 2^53).",
    message: "dec2bin: input must contain finite nonnegative integers",
};

const DEC2BIN_ERROR_INVALID_WIDTH: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.DEC2BIN.INVALID_WIDTH",
    identifier: Some("RunMat:dec2bin:InvalidWidth"),
    when: "Width argument is not a finite nonnegative numeric scalar, or too many arguments.",
    message: "dec2bin: invalid width argument",
};

const DEC2BIN_ERROR_INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.DEC2BIN.INTERNAL",
    identifier: Some("RunMat:dec2bin:InternalError"),
    when: "Internal char-array assembly failed.",
    message: "dec2bin: internal error",
};

const DEC2BIN_ERRORS: [BuiltinErrorDescriptor; 3] = [
    DEC2BIN_ERROR_INVALID_INPUT,
    DEC2BIN_ERROR_INVALID_WIDTH,
    DEC2BIN_ERROR_INTERNAL,
];

pub const DEC2BIN_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &DEC2BIN_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &DEC2BIN_ERRORS,
};

fn dec2bin_error(error: &'static BuiltinErrorDescriptor) -> RuntimeError {
    dec2bin_error_with_message(error.message, error)
}

fn dec2bin_error_with_message(
    message: impl Into<String>,
    error: &'static BuiltinErrorDescriptor,
) -> RuntimeError {
    let mut builder = build_runtime_error(message).with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

fn remap_dec2bin_flow(err: RuntimeError) -> RuntimeError {
    map_control_flow_with_builtin(err, BUILTIN_NAME)
}

async fn dec2bin_builtin(value: Value, rest: Vec<Value>) -> BuiltinResult<Value> {
    let gathered = gather_if_needed_async(&value)
        .await
        .map_err(remap_dec2bin_flow)?;
    let requested_width = parse_width_args(rest).await?;
    let values = extract_nonnegative_integers(gathered)?;
    let char_array = format_binary_rows(&values, requested_width)?;
    Ok(Value::CharArray(char_array))
}

/// Resolve the optional minimum-width argument. Absent => 0 (natural width).
/// The width must be a finite nonnegative real numeric scalar; non-integer
/// widths round to the nearest integer (documented independent choice — the
/// approved export only observes integer widths 8 and 2).
async fn parse_width_args(args: Vec<Value>) -> BuiltinResult<usize> {
    if args.is_empty() {
        return Ok(0);
    }
    if args.len() > 1 {
        return Err(dec2bin_error_with_message(
            "dec2bin: too many input arguments",
            &DEC2BIN_ERROR_INVALID_WIDTH,
        ));
    }
    let arg = args.into_iter().next().expect("length checked");
    let gathered = gather_if_needed_async(&arg)
        .await
        .map_err(remap_dec2bin_flow)?;
    let raw = match gathered {
        Value::Num(n) => n,
        Value::Int(i) => i.to_f64(),
        Value::Bool(b) => {
            if b {
                1.0
            } else {
                0.0
            }
        }
        Value::Tensor(t) if t.data.len() == 1 => t.data[0],
        _ => {
            return Err(dec2bin_error(&DEC2BIN_ERROR_INVALID_WIDTH));
        }
    };
    if !raw.is_finite() || raw < 0.0 {
        return Err(dec2bin_error(&DEC2BIN_ERROR_INVALID_WIDTH));
    }
    Ok(raw.round() as usize)
}

/// Extract the input elements (column-major) as nonnegative integers.
/// Non-integer values round to the nearest integer with ties away from zero;
/// negative, non-finite, and beyond-2^53 values are rejected. All of these
/// are documented independent choices on unobserved input classes
/// (dec2bin.q-negative); only nonnegative integer scalars are observed.
fn extract_nonnegative_integers(value: Value) -> BuiltinResult<Vec<u64>> {
    match value {
        Value::Num(n) => Ok(vec![to_nonnegative_integer(n)?]),
        Value::Int(i) => Ok(vec![to_nonnegative_integer(i.to_f64())?]),
        Value::Bool(b) => Ok(vec![u64::from(b)]),
        Value::Tensor(t) => tensor_to_integers(t),
        Value::LogicalArray(la) => {
            let tensor = tensor::logical_to_tensor(&la)
                .map_err(|_| dec2bin_error(&DEC2BIN_ERROR_INVALID_INPUT))?;
            tensor_to_integers(tensor)
        }
        other => Err(dec2bin_error_with_message(
            format!("{} (got {:?})", DEC2BIN_ERROR_INVALID_INPUT.message, other),
            &DEC2BIN_ERROR_INVALID_INPUT,
        )),
    }
}

fn tensor_to_integers(tensor: Tensor) -> BuiltinResult<Vec<u64>> {
    tensor
        .data
        .iter()
        .map(|&v| to_nonnegative_integer(v))
        .collect()
}

fn to_nonnegative_integer(value: f64) -> BuiltinResult<u64> {
    if !value.is_finite() {
        return Err(dec2bin_error_with_message(
            "dec2bin: input must be finite",
            &DEC2BIN_ERROR_INVALID_INPUT,
        ));
    }
    let rounded = value.round();
    if rounded < 0.0 {
        return Err(dec2bin_error_with_message(
            "dec2bin: input must be nonnegative",
            &DEC2BIN_ERROR_INVALID_INPUT,
        ));
    }
    if rounded > MAX_EXACT_INTEGER {
        return Err(dec2bin_error_with_message(
            "dec2bin: input exceeds the largest exactly representable integer (2^53)",
            &DEC2BIN_ERROR_INVALID_INPUT,
        ));
    }
    Ok(rounded as u64)
}

/// Assemble the char output: one row per element (column-major element
/// order), every row zero-padded on the left to a common width of
/// max(requested width, widest natural width). The observed scalar cases
/// (natural '101', padded '00000101', zero '0', non-truncating width 2)
/// follow from this rule; the multi-row layout itself is a documented
/// independent choice (dec2bin.q-vector).
fn format_binary_rows(values: &[u64], requested_width: usize) -> BuiltinResult<CharArray> {
    if values.is_empty() {
        return CharArray::new(Vec::new(), 0, 0)
            .map_err(|_| dec2bin_error(&DEC2BIN_ERROR_INTERNAL));
    }

    let digits: Vec<String> = values.iter().map(|&v| format!("{v:b}")).collect();
    let natural_width = digits.iter().map(|d| d.len()).max().unwrap_or(1);
    let width = natural_width.max(requested_width).max(1);

    let mut chars = Vec::with_capacity(digits.len() * width);
    for text in &digits {
        let pad = width - text.len();
        chars.extend(std::iter::repeat_n('0', pad));
        chars.extend(text.chars());
    }
    CharArray::new(chars, digits.len(), width).map_err(|_| dec2bin_error(&DEC2BIN_ERROR_INTERNAL))
}

#[cfg(all(test, feature = "parallel_xval"))]
pub(crate) mod tests {
    use super::*;
    use crate::builtins::common::test_support;
    use runmat_builtins::{IntValue, LogicalArray, ResolveContext, Type};

    fn run_dec2bin(value: Value) -> Value {
        futures::executor::block_on(super::dec2bin_builtin(value, Vec::new())).expect("dec2bin")
    }

    fn run_dec2bin_width(value: Value, width: Value) -> Value {
        futures::executor::block_on(super::dec2bin_builtin(value, vec![width])).expect("dec2bin")
    }

    fn char_rows(ca: &CharArray) -> Vec<String> {
        ca.data
            .chunks(ca.cols.max(1))
            .map(|chunk| chunk.iter().collect())
            .collect()
    }

    // Normative [FR-018-01, FR-018-02, FR-018-03; dec2bin.signature-primary,
    // dec2bin.output-class, dec2bin.binary-char]
    // Observed case `scalar`: dec2bin(5) => '101' (char, 1x3).
    #[test]
    fn observed_scalar_five_is_101() {
        let out = run_dec2bin(Value::Num(5.0));
        match out {
            Value::CharArray(ca) => {
                assert_eq!((ca.rows, ca.cols), (1, 3));
                assert_eq!(ca.data.iter().collect::<String>(), "101");
            }
            other => panic!("expected char array, got {other:?}"),
        }
    }

    // Normative [FR-018-04; dec2bin.binary-char, dec2bin.output-class]
    // Observed case `padded`: dec2bin(5, 8) => '00000101' (char, 1x8).
    #[test]
    fn observed_padded_width_eight_zero_pads() {
        let out = run_dec2bin_width(Value::Num(5.0), Value::Num(8.0));
        match out {
            Value::CharArray(ca) => {
                assert_eq!((ca.rows, ca.cols), (1, 8));
                assert_eq!(ca.data.iter().collect::<String>(), "00000101");
            }
            other => panic!("expected char array, got {other:?}"),
        }
    }

    // Normative [FR-018-03; dec2bin.binary-char, dec2bin.output-class]
    // Observed case `zero`: dec2bin(0) => '0' (char, 1x1).
    #[test]
    fn observed_zero_is_single_zero_digit() {
        let out = run_dec2bin(Value::Num(0.0));
        match out {
            Value::CharArray(ca) => {
                assert_eq!((ca.rows, ca.cols), (1, 1));
                assert_eq!(ca.data.iter().collect::<String>(), "0");
            }
            other => panic!("expected char array, got {other:?}"),
        }
    }

    // Normative [FR-018-05; dec2bin.binary-char]
    // Observed case `width-below-natural`: dec2bin(5, 2) => '101' (char, 1x3)
    // — a width below the natural width never truncates.
    #[test]
    fn observed_width_below_natural_does_not_truncate() {
        let out = run_dec2bin_width(Value::Num(5.0), Value::Num(2.0));
        match out {
            Value::CharArray(ca) => {
                assert_eq!((ca.rows, ca.cols), (1, 3));
                assert_eq!(ca.data.iter().collect::<String>(), "101");
            }
            other => panic!("expected char array, got {other:?}"),
        }
    }

    // unresolved_choice (dec2bin.q-vector): multi-element inputs are
    // unobserved; documented choice returns one row per element
    // (column-major order), zero-padded to the widest natural width.
    #[test]
    fn unresolved_choice_vector_rows_share_common_width() {
        let tensor = Tensor::new(vec![5.0, 3.0], vec![1, 2]).expect("tensor");
        let out = run_dec2bin(Value::Tensor(tensor));
        match out {
            Value::CharArray(ca) => {
                assert_eq!((ca.rows, ca.cols), (2, 3));
                assert_eq!(char_rows(&ca), vec!["101".to_string(), "011".to_string()]);
            }
            other => panic!("expected char array, got {other:?}"),
        }
    }

    // unresolved_choice (dec2bin.q-vector): the width argument applies to
    // every row; the common width is max(requested, widest natural).
    #[test]
    fn unresolved_choice_vector_with_width_pads_every_row() {
        let tensor = Tensor::new(vec![1.0, 2.0], vec![2, 1]).expect("tensor");
        let out = run_dec2bin_width(Value::Tensor(tensor), Value::Num(4.0));
        match out {
            Value::CharArray(ca) => {
                assert_eq!((ca.rows, ca.cols), (2, 4));
                assert_eq!(char_rows(&ca), vec!["0001".to_string(), "0010".to_string()]);
            }
            other => panic!("expected char array, got {other:?}"),
        }
    }

    // unresolved_choice (dec2bin.q-negative): negative inputs are unobserved;
    // documented choice rejects them (the approved summary covers only
    // nonnegative integers).
    #[test]
    fn unresolved_choice_negative_input_errors() {
        let err = futures::executor::block_on(super::dec2bin_builtin(Value::Num(-1.0), Vec::new()))
            .expect_err("dec2bin should reject negative input");
        assert_eq!(err.identifier(), Some("RunMat:dec2bin:InvalidInput"));
    }

    // unresolved_choice (dec2bin.q-negative): non-integer inputs are
    // unobserved; documented choice rounds to the nearest integer with ties
    // away from zero (5.4 -> 5 -> '101'; 2.5 -> 3 -> '11').
    #[test]
    fn unresolved_choice_noninteger_rounds_to_nearest() {
        let out = run_dec2bin(Value::Num(5.4));
        match out {
            Value::CharArray(ca) => assert_eq!(ca.data.iter().collect::<String>(), "101"),
            other => panic!("expected char array, got {other:?}"),
        }
        let out = run_dec2bin(Value::Num(2.5));
        match out {
            Value::CharArray(ca) => assert_eq!(ca.data.iter().collect::<String>(), "11"),
            other => panic!("expected char array, got {other:?}"),
        }
    }

    // unresolved_choice: NaN/Inf inputs are unobserved; documented choice
    // rejects them because they have no binary digit representation.
    #[test]
    fn unresolved_choice_nonfinite_input_errors() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let err =
                futures::executor::block_on(super::dec2bin_builtin(Value::Num(bad), Vec::new()))
                    .expect_err("dec2bin should reject non-finite input");
            assert_eq!(err.identifier(), Some("RunMat:dec2bin:InvalidInput"));
        }
    }

    // unresolved_choice: empty input is unobserved; documented choice
    // returns a 0x0 char array.
    #[test]
    fn unresolved_choice_empty_input_returns_empty_char() {
        let tensor = Tensor::new(Vec::new(), vec![0, 0]).expect("tensor");
        let out = run_dec2bin(Value::Tensor(tensor));
        match out {
            Value::CharArray(ca) => {
                assert_eq!((ca.rows, ca.cols), (0, 0));
                assert!(ca.data.is_empty());
            }
            other => panic!("expected char array, got {other:?}"),
        }
    }

    // unresolved_choice: integer-typed and logical inputs are unobserved;
    // documented choice converts their numeric values.
    #[test]
    fn unresolved_choice_integer_and_logical_inputs_convert() {
        let out = run_dec2bin(Value::Int(IntValue::I32(6)));
        match out {
            Value::CharArray(ca) => assert_eq!(ca.data.iter().collect::<String>(), "110"),
            other => panic!("expected char array, got {other:?}"),
        }

        let logical = LogicalArray::new(vec![1], vec![1, 1]).expect("logical");
        let out = run_dec2bin(Value::LogicalArray(logical));
        match out {
            Value::CharArray(ca) => {
                assert_eq!((ca.rows, ca.cols), (1, 1));
                assert_eq!(ca.data.iter().collect::<String>(), "1");
            }
            other => panic!("expected char array, got {other:?}"),
        }
    }

    // unresolved_choice: invalid width arguments (negative, non-finite,
    // non-numeric, or too many arguments) are unobserved; documented choice
    // rejects them with a stable identifier.
    #[test]
    fn unresolved_choice_invalid_width_arguments_error() {
        let cases: Vec<Vec<Value>> = vec![
            vec![Value::Num(-2.0)],
            vec![Value::Num(f64::NAN)],
            vec![Value::String("wide".to_string())],
            vec![Value::Num(4.0), Value::Num(4.0)],
        ];
        for rest in cases {
            let err = futures::executor::block_on(super::dec2bin_builtin(Value::Num(5.0), rest))
                .expect_err("dec2bin should reject invalid width arguments");
            assert_eq!(err.identifier(), Some("RunMat:dec2bin:InvalidWidth"));
        }
    }

    // unresolved_choice: non-numeric first arguments are unobserved;
    // documented choice rejects them with a stable identifier.
    #[test]
    fn unresolved_choice_non_numeric_input_errors() {
        let err = futures::executor::block_on(super::dec2bin_builtin(
            Value::String("hello".to_string()),
            Vec::new(),
        ))
        .expect_err("dec2bin should reject text input");
        assert_eq!(err.identifier(), Some("RunMat:dec2bin:InvalidInput"));
    }

    // GPU inputs are gathered to the host before formatting (Principle IX;
    // residency policy GatherImmediately).
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn dec2bin_gpu_tensor_roundtrip() {
        test_support::with_test_provider(|provider| {
            let tensor = Tensor::new(vec![5.0], vec![1, 1]).expect("tensor");
            let view = runmat_accelerate_api::HostTensorView {
                data: &tensor.data,
                shape: &tensor.shape,
            };
            let handle = provider.upload(&view).expect("upload");
            let out = run_dec2bin_width(Value::GpuTensor(handle), Value::Num(8.0));
            match out {
                Value::CharArray(ca) => {
                    assert_eq!((ca.rows, ca.cols), (1, 8));
                    assert_eq!(ca.data.iter().collect::<String>(), "00000101");
                }
                other => panic!("expected char array, got {other:?}"),
            }
        });
    }

    // Registration is discoverable through the builtin registry (SC-018-2).
    #[test]
    fn dec2bin_is_registered() {
        assert!(runmat_builtins::builtin_function_by_name("dec2bin").is_some());
    }

    #[test]
    fn dec2bin_type_is_string_scalar() {
        assert_eq!(
            string_scalar_type(&[Type::Num], &ResolveContext::new(Vec::new())),
            Type::String
        );
    }
}
