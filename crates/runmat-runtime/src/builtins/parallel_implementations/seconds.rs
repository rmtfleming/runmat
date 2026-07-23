// Parallel clean-room implementation of `seconds` (independently developed against the
// matlab-interface-spec black-box specs). 0-6-0 ships the DEFAULT implementation; this copy is
// retained unregistered for differential cross-validation. Original path: crates/runmat-runtime/src/builtins/duration/seconds.rs
//! MATLAB-compatible `seconds` builtin for RunMat.
//!
//! Clean-room provenance: specs/029-time-conversion (spec `seconds` 0.2.0,
//! claims seconds.signature-primary, seconds.output-class,
//! seconds.to-duration). Observed case `to-duration`: `seconds(5)` => a
//! `duration` scalar (class duration, 1x1).
//!
//! `seconds(x)` builds a `duration` from a number of seconds using the same
//! internal representation as the `duration` constructor (days stored in the
//! `__days` property): each element is `x / 86400` days. The observed value
//! is opaque (`<duration [1 1]>`), so the internal magnitude and the chosen
//! display `Format` ("s", rendering the value in seconds) are documented
//! independent choices. The duration-to-number inverse form `seconds(d)` and
//! array inputs are unobserved — see specs/029-time-conversion/spec.md.

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    Tensor, Value,
};
use runmat_macros::runtime_builtin;

use super::duration_object_from_days_tensor;
use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ShapeRequirements,
};
use crate::builtins::common::tensor;
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "seconds",
    op_kind: GpuOpKind::Custom("duration"),
    supported_precisions: &[],
    broadcast: BroadcastSemantics::None,
    provider_hooks: &[],
    constant_strategy: ConstantStrategy::InlineLiteral,
    residency: ResidencyPolicy::GatherImmediately,
    nan_mode: ReductionNaN::Include,
    two_pass_threshold: None,
    workgroup_size: None,
    accepts_nan_mode: false,
    notes: "Converts a numeric second count into a duration object; GPU inputs are gathered immediately.",
};

pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "seconds",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Not fusible; produces a duration object rather than a numeric tensor.",
};

const SECONDS_OUTPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "d",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Duration of x seconds.",
}];

const SECONDS_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "x",
    ty: BuiltinParamType::NumericArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Number of seconds.",
}];

const SECONDS_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "d = seconds(x)",
    inputs: &SECONDS_INPUTS,
    outputs: &SECONDS_OUTPUTS,
}];

const SECONDS_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SECONDS.INVALID_INPUT",
    identifier: Some("RunMat:seconds:InvalidInput"),
    when: "The argument is not a numeric value convertible to a duration.",
    message: "seconds: input must be a numeric value",
};

const SECONDS_ERRORS: [BuiltinErrorDescriptor; 1] = [SECONDS_ERROR_INVALID_INPUT];

pub const SECONDS_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &SECONDS_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &SECONDS_ERRORS,
};

const SECONDS_PER_DAY: f64 = 86_400.0;

/// Display format for durations produced by `seconds`: render the value in
/// seconds. Unobserved (the observed value is opaque); documented choice.
const SECONDS_FORMAT: &str = "s";

async fn seconds_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    if args.len() != 1 {
        return Err(invalid_input("seconds: expected seconds(x)"));
    }
    let value = gather_if_needed_async(&args[0])
        .await
        .map_err(|err| invalid_input(format!("seconds: {}", err.message())))?;
    let source = numeric_tensor(value)?;
    let days: Vec<f64> = source.data.iter().map(|s| s / SECONDS_PER_DAY).collect();
    let shape = tensor::default_shape_for(&source.shape, days.len());
    let days_tensor =
        Tensor::new(days, shape).map_err(|err| invalid_input(format!("seconds: {err}")))?;
    duration_object_from_days_tensor(days_tensor, SECONDS_FORMAT)
}

fn invalid_input(message: impl Into<String>) -> RuntimeError {
    build_runtime_error(message)
        .with_builtin("seconds")
        .with_identifier(SECONDS_ERROR_INVALID_INPUT.identifier.expect("identifier"))
        .build()
}

/// Accept a numeric value and return it as a tensor. Non-numeric inputs
/// (including the unobserved duration-to-number form `seconds(d)`) are
/// rejected with a clear error (documented independent choice).
fn numeric_tensor(value: Value) -> BuiltinResult<Tensor> {
    match value {
        Value::Num(n) => Ok(Tensor::new(vec![n], vec![1, 1]).expect("scalar tensor")),
        Value::Int(i) => Ok(Tensor::new(vec![i.to_f64()], vec![1, 1]).expect("scalar tensor")),
        Value::Tensor(t) => Ok(t),
        _ => Err(invalid_input(SECONDS_ERROR_INVALID_INPUT.message)),
    }
}

#[cfg(all(test, feature = "parallel_xval"))]
pub(crate) mod tests {
    use super::*;
    use crate::builtins::duration::{
        duration_display_text, duration_tensor_from_duration_value, is_duration_object,
    };
    use futures::executor::block_on;

    fn run(args: Vec<Value>) -> Value {
        block_on(super::seconds_builtin(args)).expect("seconds")
    }

    fn run_err(args: Vec<Value>) -> RuntimeError {
        block_on(super::seconds_builtin(args)).expect_err("expected seconds error")
    }

    /// Total seconds stored in a (scalar or array) duration value.
    fn duration_seconds(value: &Value) -> Vec<f64> {
        duration_tensor_from_duration_value(value)
            .expect("duration days tensor")
            .data
            .iter()
            .map(|days| days * SECONDS_PER_DAY)
            .collect()
    }

    // Normative [FR-029-02, FR-029-03; seconds.signature-primary,
    // seconds.output-class, seconds.to-duration] — observed case
    // `to-duration`: seconds(5) => a duration scalar (class duration, 1x1)
    // whose magnitude is 5 seconds.
    #[test]
    fn observed_scalar_number_becomes_duration() {
        let value = run(vec![Value::Num(5.0)]);
        assert!(is_duration_object(&value), "result must be a duration");
        let days = duration_tensor_from_duration_value(&value).expect("days tensor");
        assert_eq!(days.shape, vec![1, 1], "scalar duration is 1x1");
        let seconds = duration_seconds(&value);
        assert_eq!(seconds.len(), 1);
        assert!(
            (seconds[0] - 5.0).abs() < 1e-9,
            "expected 5 seconds, got {}",
            seconds[0]
        );
        // The chosen "s" format renders the magnitude directly.
        let rendered = duration_display_text(&value)
            .expect("display")
            .expect("duration text");
        assert_eq!(rendered, "5");
    }

    // unresolved_choice [FR-029-06]: array inputs map element-wise to a
    // duration array of the same shape (unobserved; documented choice).
    #[test]
    fn unresolved_choice_array_input_maps_elementwise() {
        let input = Value::Tensor(Tensor::new(vec![1.0, 2.0, 3.0], vec![1, 3]).unwrap());
        let value = run(vec![input]);
        assert!(is_duration_object(&value));
        let days = duration_tensor_from_duration_value(&value).expect("days tensor");
        assert_eq!(days.shape, vec![1, 3]);
        assert_eq!(duration_seconds(&value), vec![1.0, 2.0, 3.0]);
    }

    // unresolved_choice [FR-029-06]: non-numeric inputs and wrong argument
    // counts are rejected with a clear error (the duration-to-number inverse
    // form is unobserved).
    #[test]
    fn unresolved_choice_invalid_inputs_error() {
        assert!(run_err(vec![]).to_string().contains("seconds"));
        assert!(run_err(vec![Value::Num(1.0), Value::Num(2.0)])
            .to_string()
            .contains("seconds"));
        assert!(run_err(vec![Value::String("nope".to_string())])
            .to_string()
            .contains("seconds"));
    }

    #[test]
    fn descriptor_advertises_primary_signature() {
        assert_eq!(SECONDS_DESCRIPTOR.signatures[0].label, "d = seconds(x)");
    }
}
