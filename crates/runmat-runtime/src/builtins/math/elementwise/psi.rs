//! MATLAB-compatible `psi` (digamma) builtin for RunMat.
//!
//! Clean-room provenance: specs/021-psi (approved export `psi` 0.2.0, commit
//! `e46587ae1b78237e0f43cd55f678de01a28495a4` of `matlab-interface-spec`;
//! claims psi.signature-primary, psi.output-class, psi.digamma-values).
//! Behaviour left unresolved by the export (poles at non-positive integers,
//! negative/complex arguments, the two-argument polygamma form) follows
//! documented independent choices tagged `unresolved_choice_*` in tests.
//!
//! Algorithm (independent, public-domain mathematics): the classical digamma
//! evaluation via the argument-shift recurrence `psi(x+1) = psi(x) + 1/x`
//! followed by the Bernoulli-number asymptotic expansion
//! `psi(x) ~ ln x - 1/(2x) - sum_{k>=1} B_{2k} / (2k * x^{2k})`, from
//! M. Abramowitz and I. A. Stegun (eds.), "Handbook of Mathematical
//! Functions", National Bureau of Standards Applied Mathematics Series 55
//! (1964), equations 6.3.5 and 6.3.18 (a U.S. government work in the public
//! domain). The same scheme is described in J. M. Bernardo, "Algorithm
//! AS 103: Psi (Digamma) Function", Applied Statistics 25(3), 1976. The code
//! below was written from the formulas; no MathWorks material was consulted.

use runmat_accelerate_api::GpuTensorHandle;
use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    CharArray, Tensor, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ScalarType, ShapeRequirements,
};
use crate::builtins::common::{gpu_helpers, map_control_flow_with_builtin, tensor};
use crate::builtins::math::type_resolvers::numeric_unary_type;
use crate::{build_runtime_error, BuiltinResult, RuntimeError};

/// Arguments below this threshold are shifted up with the recurrence before
/// the asymptotic series is applied. With terms through `B14` the truncation
/// error at the threshold is below 1e-16 relative.
const DIGAMMA_SHIFT_THRESHOLD: f64 = 10.0;

/// Horner coefficients of the subtracted asymptotic tail
/// `sum B_{2k} / (2k * x^{2k})` (A&S 6.3.18): 1/(12x^2) - 1/(120x^4)
/// + 1/(252x^6) - 1/(240x^8) + 1/(132x^10) - 691/(32760x^12) + 1/(12x^14).
const DIGAMMA_TAIL: [f64; 7] = [
    1.0 / 12.0,
    -1.0 / 120.0,
    1.0 / 252.0,
    -1.0 / 240.0,
    1.0 / 132.0,
    -691.0 / 32760.0,
    1.0 / 12.0,
];

#[runmat_macros::register_gpu_spec(builtin_path = "crate::builtins::math::elementwise::psi")]
pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "psi",
    op_kind: GpuOpKind::Elementwise,
    supported_precisions: &[ScalarType::F32, ScalarType::F64],
    broadcast: BroadcastSemantics::Matlab,
    provider_hooks: &[],
    constant_strategy: ConstantStrategy::InlineLiteral,
    residency: ResidencyPolicy::GatherImmediately,
    nan_mode: ReductionNaN::Include,
    two_pass_threshold: None,
    workgroup_size: None,
    accepts_nan_mode: false,
    notes: "No provider hook exists for psi; GPU inputs are gathered to the host and evaluated on the CPU.",
};

#[runmat_macros::register_fusion_spec(builtin_path = "crate::builtins::math::elementwise::psi")]
pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "psi",
    shape: ShapeRequirements::BroadcastCompatible,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: true,
    notes: "Host-only evaluation; poles at non-positive integers produce NaN. Fusion falls back to the standalone host path.",
};

const BUILTIN_NAME: &str = "psi";

const PSI_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "Y",
    ty: BuiltinParamType::NumericArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Digamma-function result (class double).",
}];

const PSI_INPUTS_X: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "X",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Real numeric input.",
}];

const PSI_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "Y = psi(X)",
    inputs: &PSI_INPUTS_X,
    outputs: &PSI_OUTPUT,
}];

const PSI_ERROR_INVALID_ARGUMENT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.PSI.INVALID_ARGUMENT",
    identifier: Some("RunMat:psi:InvalidArgument"),
    when: "More arguments are supplied than any supported form accepts.",
    message: "psi: invalid argument",
};

const PSI_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.PSI.INVALID_INPUT",
    identifier: Some("RunMat:psi:InvalidInput"),
    when: "Input cannot be converted to supported real numeric forms.",
    message: "psi: invalid input",
};

const PSI_ERROR_POLYGAMMA_UNSUPPORTED: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.PSI.POLYGAMMA_UNSUPPORTED",
    identifier: Some("RunMat:psi:PolygammaUnsupported"),
    when: "The two-argument polygamma form psi(k, X) is requested.",
    message: "psi: polygamma form not supported",
};

const PSI_ERRORS: [BuiltinErrorDescriptor; 3] = [
    PSI_ERROR_INVALID_ARGUMENT,
    PSI_ERROR_INVALID_INPUT,
    PSI_ERROR_POLYGAMMA_UNSUPPORTED,
];

pub const PSI_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &PSI_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &PSI_ERRORS,
};

fn builtin_error(message: impl Into<String>) -> RuntimeError {
    build_runtime_error(message)
        .with_builtin(BUILTIN_NAME)
        .build()
}

fn psi_error_with_detail(
    error: &'static BuiltinErrorDescriptor,
    detail: impl std::fmt::Display,
) -> RuntimeError {
    let mut builder =
        build_runtime_error(format!("{}: {}", error.message, detail)).with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

#[runtime_builtin(
    name = "psi",
    category = "math/elementwise",
    summary = "Compute digamma (psi) function values element-wise.",
    keywords = "psi,digamma,polygamma,special",
    type_resolver(numeric_unary_type),
    descriptor(crate::builtins::math::elementwise::psi::PSI_DESCRIPTOR),
    builtin_path = "crate::builtins::math::elementwise::psi"
)]
async fn psi_builtin(value: Value, rest: Vec<Value>) -> BuiltinResult<Value> {
    reject_extra_args(&rest)?;
    match value {
        Value::GpuTensor(handle) => psi_gpu(handle).await,
        Value::Tensor(t) => psi_tensor(t).map(tensor::tensor_into_value),
        Value::Num(n) => Ok(Value::Num(digamma(n))),
        Value::Int(i) => Ok(Value::Num(digamma(i.to_f64()))),
        Value::Bool(b) => Ok(Value::Num(digamma(if b { 1.0 } else { 0.0 }))),
        Value::LogicalArray(logical) => {
            let t = tensor::logical_to_tensor(&logical)
                .map_err(|e| builtin_error(format!("psi: {e}")))?;
            psi_tensor(t).map(tensor::tensor_into_value)
        }
        Value::CharArray(ca) => psi_char_array(ca),
        Value::Complex(_, _) | Value::ComplexTensor(_) => Err(psi_error_with_detail(
            &PSI_ERROR_INVALID_INPUT,
            "complex input is not supported (unobserved in the approved specification)",
        )),
        Value::String(_) | Value::StringArray(_) => Err(psi_error_with_detail(
            &PSI_ERROR_INVALID_INPUT,
            "expected numeric input",
        )),
        other => Err(psi_error_with_detail(
            &PSI_ERROR_INVALID_INPUT,
            format!("unsupported input type {other:?}; expected numeric or gpuArray input"),
        )),
    }
}

/// The approved export confirms only the single-argument form `Y = psi(X)`;
/// the two-argument polygamma form is pending (unresolved psi.q-polygamma).
fn reject_extra_args(rest: &[Value]) -> BuiltinResult<()> {
    match rest.len() {
        0 => Ok(()),
        1 => Err(psi_error_with_detail(
            &PSI_ERROR_POLYGAMMA_UNSUPPORTED,
            "the two-argument polygamma form psi(k, X) is not yet specified for RunMat (unresolved psi.q-polygamma)",
        )),
        _ => Err(psi_error_with_detail(
            &PSI_ERROR_INVALID_ARGUMENT,
            "too many input arguments",
        )),
    }
}

async fn psi_gpu(handle: GpuTensorHandle) -> BuiltinResult<Value> {
    // runmat-accelerate-api exposes no `unary_psi` hook, so GPU inputs are
    // gathered to the host and evaluated on the CPU.
    let gathered = gpu_helpers::gather_value_async(&Value::GpuTensor(handle))
        .await
        .map_err(|flow| map_control_flow_with_builtin(flow, BUILTIN_NAME))?;
    match gathered {
        Value::Tensor(t) => psi_tensor(t).map(tensor::tensor_into_value),
        Value::Num(n) => Ok(Value::Num(digamma(n))),
        other => Err(psi_error_with_detail(
            &PSI_ERROR_INVALID_INPUT,
            format!("unsupported gathered gpuArray value {other:?}"),
        )),
    }
}

fn psi_tensor(t: Tensor) -> BuiltinResult<Tensor> {
    let data = t.data.iter().map(|&v| digamma(v)).collect::<Vec<_>>();
    Tensor::new(data, t.shape.clone()).map_err(|e| builtin_error(format!("psi: {e}")))
}

fn psi_char_array(ca: CharArray) -> BuiltinResult<Value> {
    let data = ca
        .data
        .iter()
        .map(|&ch| digamma(ch as u32 as f64))
        .collect::<Vec<_>>();
    let tensor = Tensor::new(data, vec![ca.rows, ca.cols])
        .map_err(|e| builtin_error(format!("psi: {e}")))?;
    Ok(tensor::tensor_into_value(tensor))
}

/// Digamma function for real arguments (see the module header for the
/// public-domain algorithm source, A&S 6.3.5 / 6.3.18).
///
/// Documented independent choices for unobserved inputs: exact non-positive
/// integers (the poles of digamma) return NaN; `+Inf` returns `+Inf`;
/// `-Inf` and NaN return NaN. Negative non-integer arguments evaluate
/// through the same recurrence, where the function is well defined.
fn digamma(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    if x.is_infinite() {
        return if x.is_sign_positive() {
            f64::INFINITY
        } else {
            f64::NAN
        };
    }
    if x <= 0.0 && x == x.floor() {
        // Poles at 0, -1, -2, ...: unresolved psi.q-poles; documented choice.
        return f64::NAN;
    }
    let mut x = x;
    let mut result = 0.0;
    // Recurrence psi(x) = psi(x + 1) - 1/x (A&S 6.3.5) shifts the argument
    // into the asymptotic regime.
    while x < DIGAMMA_SHIFT_THRESHOLD {
        result -= 1.0 / x;
        x += 1.0;
    }
    // Asymptotic expansion psi(x) ~ ln x - 1/(2x) - sum B_{2k}/(2k x^{2k})
    // (A&S 6.3.18), truncated after the B14 term.
    let inv = 1.0 / x;
    let inv2 = inv * inv;
    let mut tail = 0.0;
    for &coeff in DIGAMMA_TAIL.iter().rev() {
        tail = coeff + inv2 * tail;
    }
    result + x.ln() - 0.5 * inv - inv2 * tail
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::builtins::common::test_support;
    use futures::executor::block_on;
    use runmat_accelerate_api::HostTensorView;
    use runmat_builtins::{IntValue, ResolveContext, Type};

    /// Observed values from the approved export
    /// (`observations/psi_observations.json`, cases `scalar` and `vector`,
    /// R2026a GLNXA64). Floating-point observations; reproduced to
    /// `OBSERVED_TOLERANCE`.
    const OBSERVED_PSI_ONE: f64 = -0.577_215_664_901_532_31;
    const OBSERVED_PSI_TWO: f64 = 0.422_784_335_098_467_47;
    const OBSERVED_PSI_THREE: f64 = 0.922_784_335_098_467_47;
    const OBSERVED_TOLERANCE: f64 = 1e-12;

    /// Euler-Mascheroni constant (public mathematics; used only for
    /// algorithm-identity checks, not as an observation).
    const EULER_MASCHERONI: f64 = 0.577_215_664_901_532_9;

    fn psi_builtin(value: Value, rest: Vec<Value>) -> BuiltinResult<Value> {
        block_on(super::psi_builtin(value, rest))
    }

    fn approx_eq(a: f64, b: f64, tol: f64) {
        assert!((a - b).abs() <= tol, "expected {b}, got {a} (tol {tol})");
    }

    #[test]
    fn psi_descriptor_signature_covers_single_argument_form() {
        let labels: Vec<&str> = PSI_DESCRIPTOR
            .signatures
            .iter()
            .map(|sig| sig.label)
            .collect();
        assert_eq!(labels, vec!["Y = psi(X)"]);
    }

    #[test]
    fn psi_type_preserves_tensor_shape() {
        let out = numeric_unary_type(
            &[Type::Tensor {
                shape: Some(vec![Some(2), Some(3)]),
            }],
            &ResolveContext::new(Vec::new()),
        );
        assert_eq!(
            out,
            Type::Tensor {
                shape: Some(vec![Some(2), Some(3)])
            }
        );
    }

    #[test]
    fn psi_type_scalar_tensor_returns_num() {
        let out = numeric_unary_type(
            &[Type::Tensor {
                shape: Some(vec![Some(1), Some(1)]),
            }],
            &ResolveContext::new(Vec::new()),
        );
        assert_eq!(out, Type::Num);
    }

    // Normative [FR-021-01, FR-021-02; psi.digamma-values, psi.output-class;
    // R2026a] — observed case `scalar`: psi(1) => -0.57721566490153231,
    // class double, size 1x1.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn observed_scalar_digamma_at_one() {
        match psi_builtin(Value::Num(1.0), Vec::new()).expect("psi") {
            Value::Num(v) => approx_eq(v, OBSERVED_PSI_ONE, OBSERVED_TOLERANCE),
            other => panic!("expected double scalar result, got {other:?}"),
        }
    }

    // Normative [FR-021-01..03; psi.digamma-values, psi.output-class;
    // R2026a] — observed case `vector`: psi([1 2 3]) =>
    // [-0.57721566490153231 0.42278433509846747 0.92278433509846747],
    // class double, size 1x3.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn observed_vector_digamma_values_and_shape() {
        let input = Tensor::new(vec![1.0, 2.0, 3.0], vec![1, 3]).unwrap();
        match psi_builtin(Value::Tensor(input), Vec::new()).expect("psi") {
            Value::Tensor(t) => {
                assert_eq!(t.shape, vec![1, 3]);
                approx_eq(t.data[0], OBSERVED_PSI_ONE, OBSERVED_TOLERANCE);
                approx_eq(t.data[1], OBSERVED_PSI_TWO, OBSERVED_TOLERANCE);
                approx_eq(t.data[2], OBSERVED_PSI_THREE, OBSERVED_TOLERANCE);
            }
            other => panic!("expected double tensor result, got {other:?}"),
        }
    }

    // Algorithm identity (public mathematics, not a MATLAB claim):
    // psi(1/2) = -gamma - 2 ln 2.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn digamma_matches_half_argument_closed_form() {
        let expected = -EULER_MASCHERONI - 2.0 * std::f64::consts::LN_2;
        approx_eq(digamma(0.5), expected, 1e-13);
    }

    // Algorithm identity (public mathematics): psi(x+1) - psi(x) = 1/x.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn digamma_satisfies_forward_recurrence() {
        for &x in &[0.25, 1.75, 7.5, 42.0, 123.456, -2.25] {
            let residual = digamma(x + 1.0) - digamma(x) - 1.0 / x;
            assert!(
                residual.abs() < 1e-10,
                "recurrence residual {residual} at x={x}"
            );
        }
    }

    // unresolved_choice [FR-021-04; psi.q-poles]: exact non-positive
    // integers (digamma poles) return NaN by documented choice.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_poles_return_nan() {
        for &x in &[0.0, -1.0, -5.0, -100.0] {
            match psi_builtin(Value::Num(x), Vec::new()).expect("psi") {
                Value::Num(v) => assert!(v.is_nan(), "expected NaN at pole {x}, got {v}"),
                other => panic!("expected scalar result, got {other:?}"),
            }
        }
    }

    // unresolved_choice [FR-021-04]: negative non-integers are finite and
    // consistent with the recurrence psi(x+1) = psi(x) + 1/x.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_negative_non_integer_is_finite() {
        let value = digamma(-0.5);
        assert!(value.is_finite());
        approx_eq(value, digamma(0.5) + 2.0, 1e-10);
    }

    // unresolved_choice [FR-021-04]: NaN propagates; +Inf -> +Inf;
    // -Inf -> NaN.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_nan_and_infinity_propagation() {
        assert!(digamma(f64::NAN).is_nan());
        assert_eq!(digamma(f64::INFINITY), f64::INFINITY);
        assert!(digamma(f64::NEG_INFINITY).is_nan());
    }

    // unresolved_choice [FR-021-04]: integer input promotes to double.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_int_input_promotes_to_double() {
        match psi_builtin(Value::Int(IntValue::I32(1)), Vec::new()).expect("psi") {
            Value::Num(v) => approx_eq(v, OBSERVED_PSI_ONE, OBSERVED_TOLERANCE),
            other => panic!("expected double scalar result, got {other:?}"),
        }
    }

    // unresolved_choice [FR-021-04]: logical input promotes to double.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_logical_input_promotes_to_double() {
        match psi_builtin(Value::Bool(true), Vec::new()).expect("psi") {
            Value::Num(v) => approx_eq(v, OBSERVED_PSI_ONE, OBSERVED_TOLERANCE),
            other => panic!("expected double scalar result, got {other:?}"),
        }
    }

    // unresolved_choice [FR-021-04]: char input promotes to its code points.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_char_input_promotes_to_code_points() {
        let chars = CharArray::new("ab".chars().collect(), 1, 2).unwrap();
        match psi_builtin(Value::CharArray(chars), Vec::new()).expect("psi") {
            Value::Tensor(t) => {
                assert_eq!(t.shape, vec![1, 2]);
                approx_eq(t.data[0], digamma(97.0), 1e-12);
                approx_eq(t.data[1], digamma(98.0), 1e-12);
            }
            other => panic!("expected tensor result, got {other:?}"),
        }
    }

    // unresolved_choice [FR-021-04]: complex input is rejected.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_complex_input_errors() {
        let err = psi_builtin(Value::Complex(1.0, 1.0), Vec::new()).expect_err("expected error");
        assert_eq!(err.identifier(), PSI_ERROR_INVALID_INPUT.identifier);
        assert!(err.message().contains("complex input is not supported"));
    }

    // unresolved_choice [FR-021-04]: string input is rejected.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_string_input_errors() {
        let err = psi_builtin(Value::from("hello"), Vec::new()).expect_err("expected error");
        assert_eq!(err.identifier(), PSI_ERROR_INVALID_INPUT.identifier);
        assert!(err.message().contains("expected numeric input"));
    }

    // unresolved_choice [FR-021-05; psi.q-polygamma]: the pending
    // two-argument polygamma form errors with a stable identifier.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_polygamma_form_errors() {
        let x = Tensor::new(vec![1.0, 2.0, 3.0], vec![1, 3]).unwrap();
        let err = psi_builtin(Value::Num(0.0), vec![Value::Tensor(x)]).expect_err("expected error");
        assert_eq!(err.identifier(), PSI_ERROR_POLYGAMMA_UNSUPPORTED.identifier);
        assert!(err.message().contains("polygamma"));
    }

    // unresolved_choice [FR-021-05]: three or more arguments are rejected.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_extra_arguments_error() {
        let err = psi_builtin(Value::Num(1.0), vec![Value::Num(2.0), Value::Num(3.0)])
            .expect_err("expected error");
        assert_eq!(err.identifier(), PSI_ERROR_INVALID_ARGUMENT.identifier);
        assert!(err.message().contains("too many input arguments"));
    }

    // GPU inputs gather to the host (no provider hook); results match the
    // observed vector case within the observed tolerance.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn psi_gpu_gather_matches_observed_values() {
        test_support::with_test_provider(|provider| {
            let input = Tensor::new(vec![1.0, 2.0, 3.0], vec![1, 3]).unwrap();
            let view = HostTensorView {
                data: &input.data,
                shape: &input.shape,
            };
            let handle = provider.upload(&view).expect("upload");
            let result = psi_builtin(Value::GpuTensor(handle), Vec::new()).expect("psi");
            let gathered = test_support::gather(result).expect("gather");
            assert_eq!(gathered.shape, vec![1, 3]);
            approx_eq(gathered.data[0], OBSERVED_PSI_ONE, OBSERVED_TOLERANCE);
            approx_eq(gathered.data[1], OBSERVED_PSI_TWO, OBSERVED_TOLERANCE);
            approx_eq(gathered.data[2], OBSERVED_PSI_THREE, OBSERVED_TOLERANCE);
        });
    }
}
