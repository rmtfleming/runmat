// Parallel clean-room implementation of `etime` (independently developed against the
// matlab-interface-spec black-box specs). 0-6-0 ships the DEFAULT implementation; this copy is
// retained unregistered for differential cross-validation. Original path: crates/runmat-runtime/src/builtins/datetime/etime.rs
//! MATLAB-compatible `etime` builtin for RunMat.
//!
//! Clean-room provenance: specs/029-time-conversion (spec `etime` 0.2.0,
//! claims etime.signature-primary, etime.output-class,
//! etime.elapsed-seconds). Observed case `one-second`:
//! `etime([2020 1 1 0 0 1], [2020 1 1 0 0 0])` => `1` (double, 1x1).
//!
//! Elapsed seconds are computed by converting each date vector
//! `[Y M D H MN S]` to a total-seconds count via an independently
//! implemented civil-to-days conversion (the publicly documented,
//! public-domain Hinnant algorithm — the inverse of the `civil_from_days`
//! used by `datestr`) and subtracting. Because `etime` returns a difference,
//! the choice of day epoch cancels and is immaterial. Behaviour beyond the
//! observed one-second case (negative results when t2 precedes t1, cross-day
//! and cross-year spans, fractional seconds, malformed date vectors) is a
//! documented independent choice — see specs/029-time-conversion/spec.md.

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    ResolveContext, Type, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ShapeRequirements,
};
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "etime",
    op_kind: GpuOpKind::Custom("datetime"),
    supported_precisions: &[],
    broadcast: BroadcastSemantics::None,
    provider_hooks: &[],
    constant_strategy: ConstantStrategy::InlineLiteral,
    residency: ResidencyPolicy::GatherImmediately,
    nan_mode: ReductionNaN::Include,
    two_pass_threshold: None,
    workgroup_size: None,
    accepts_nan_mode: false,
    notes: "Scalar elapsed-seconds difference over two date vectors; GPU inputs are gathered immediately.",
};

pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "etime",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Not fusible; produces a scalar double from two six-element date vectors.",
};

const ETIME_OUTPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "s",
    ty: BuiltinParamType::NumericScalar,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Elapsed seconds between the two date vectors (double).",
}];

const ETIME_INPUTS: [BuiltinParamDescriptor; 2] = [
    BuiltinParamDescriptor {
        name: "t2",
        ty: BuiltinParamType::NumericArray,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Later date vector [Y M D H MN S].",
    },
    BuiltinParamDescriptor {
        name: "t1",
        ty: BuiltinParamType::NumericArray,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Earlier date vector [Y M D H MN S].",
    },
];

const ETIME_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "s = etime(t2, t1)",
    inputs: &ETIME_INPUTS,
    outputs: &ETIME_OUTPUTS,
}];

const ETIME_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ETIME.INVALID_INPUT",
    identifier: Some("RunMat:etime:InvalidInput"),
    when: "An argument is not a finite six-element date vector [Y M D H MN S].",
    message: "etime: each argument must be a six-element date vector [Y M D H MN S]",
};

const ETIME_ERRORS: [BuiltinErrorDescriptor; 1] = [ETIME_ERROR_INVALID_INPUT];

pub const ETIME_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &ETIME_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ETIME_ERRORS,
};

const SECONDS_PER_DAY: f64 = 86_400.0;

async fn etime_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    if args.len() != 2 {
        return Err(invalid_input("etime: expected etime(t2, t1)"));
    }
    let t2 = gather_if_needed_async(&args[0])
        .await
        .map_err(|err| invalid_input(format!("etime: {}", err.message())))?;
    let t1 = gather_if_needed_async(&args[1])
        .await
        .map_err(|err| invalid_input(format!("etime: {}", err.message())))?;
    let seconds_t2 = date_vector_seconds(&t2)?;
    let seconds_t1 = date_vector_seconds(&t1)?;
    Ok(Value::Num(seconds_t2 - seconds_t1))
}

fn num_scalar_type(_: &[Type], _context: &ResolveContext) -> Type {
    Type::Num
}

fn invalid_input(message: impl Into<String>) -> RuntimeError {
    build_runtime_error(message)
        .with_builtin("etime")
        .with_identifier(ETIME_ERROR_INVALID_INPUT.identifier.expect("identifier"))
        .build()
}

/// Extract the six components of a date vector `[Y M D H MN S]`. Only a
/// six-element finite numeric vector is accepted; other shapes and classes
/// are unobserved in the approved spec, so they are rejected with a clear
/// error (documented independent choice).
fn date_vector_seconds(value: &Value) -> BuiltinResult<f64> {
    let data: Vec<f64> = match value {
        Value::Tensor(t) => t.data.clone(),
        Value::Num(n) => vec![*n],
        Value::Int(i) => vec![i.to_f64()],
        _ => return Err(invalid_input(ETIME_ERROR_INVALID_INPUT.message)),
    };
    if data.len() != 6 || data.iter().any(|v| !v.is_finite()) {
        return Err(invalid_input(ETIME_ERROR_INVALID_INPUT.message));
    }
    let (year, month, day) = (data[0], data[1], data[2]);
    let (hour, minute, second) = (data[3], data[4], data[5]);
    let days = days_from_civil(
        year.round() as i64,
        month.round() as i64,
        day.round() as i64,
    );
    Ok(days as f64 * SECONDS_PER_DAY + hour * 3600.0 + minute * 60.0 + second)
}

/// Convert a proleptic Gregorian `(year, month, day)` into a day count
/// relative to 1970-01-01. Independent implementation of the publicly
/// documented, public-domain civil-to-days algorithm (Howard Hinnant,
/// "chrono-Compatible Low-Level Date Algorithms"); the inverse of the
/// `civil_from_days` used by `datestr`. The epoch offset cancels in the
/// `etime` difference and is immaterial to the observable result.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400; // [0, 399]
    let mp = if month > 2 { month - 3 } else { month + 9 }; // [0, 11]
    let doy = (153 * mp + 2) / 5 + day - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146_097 + doe - 719_468
}

#[cfg(all(test, feature = "parallel_xval"))]
pub(crate) mod tests {
    use super::*;
    use futures::executor::block_on;
    use runmat_builtins::Tensor;

    fn date_vector(components: [f64; 6]) -> Value {
        Value::Tensor(Tensor::new(components.to_vec(), vec![1, 6]).unwrap())
    }

    fn run(args: Vec<Value>) -> Value {
        block_on(super::etime_builtin(args)).expect("etime")
    }

    fn run_err(args: Vec<Value>) -> RuntimeError {
        block_on(super::etime_builtin(args)).expect_err("expected etime error")
    }

    fn assert_scalar_double(value: &Value, expected: f64) {
        match value {
            Value::Num(n) => assert!((n - expected).abs() < 1e-9, "expected {expected}, got {n}"),
            other => panic!("expected scalar double, got {other:?}"),
        }
    }

    // Normative [FR-029-01; etime.signature-primary, etime.output-class,
    // etime.elapsed-seconds] — observed case `one-second`:
    // etime([2020 1 1 0 0 1], [2020 1 1 0 0 0]) => 1 (double, 1x1).
    #[test]
    fn observed_one_second_elapsed() {
        let value = run(vec![
            date_vector([2020.0, 1.0, 1.0, 0.0, 0.0, 1.0]),
            date_vector([2020.0, 1.0, 1.0, 0.0, 0.0, 0.0]),
        ]);
        assert_scalar_double(&value, 1.0);
    }

    // unresolved_choice [FR-029-04; etime.q-negative]: when t2 precedes t1 the
    // difference is negative (documented independent choice; unobserved).
    #[test]
    fn unresolved_choice_negative_when_t2_before_t1() {
        let value = run(vec![
            date_vector([2020.0, 1.0, 1.0, 0.0, 0.0, 0.0]),
            date_vector([2020.0, 1.0, 1.0, 0.0, 0.0, 1.0]),
        ]);
        assert_scalar_double(&value, -1.0);
    }

    // unresolved_choice [FR-029-04]: spans that cross day, month and year
    // boundaries accumulate through the civil-to-days conversion (unobserved;
    // documented choice). One full day plus one hour = 90000 seconds.
    #[test]
    fn unresolved_choice_cross_day_span() {
        let value = run(vec![
            date_vector([2020.0, 1.0, 2.0, 1.0, 0.0, 0.0]),
            date_vector([2020.0, 1.0, 1.0, 0.0, 0.0, 0.0]),
        ]);
        assert_scalar_double(&value, 90_000.0);
    }

    // unresolved_choice [FR-029-04]: a non-leap common year spans exactly
    // 365 days (2021 is not a leap year) — exercises the year rollover in the
    // civil-to-days conversion.
    #[test]
    fn unresolved_choice_cross_year_span() {
        let value = run(vec![
            date_vector([2022.0, 1.0, 1.0, 0.0, 0.0, 0.0]),
            date_vector([2021.0, 1.0, 1.0, 0.0, 0.0, 0.0]),
        ]);
        assert_scalar_double(&value, 365.0 * 86_400.0);
    }

    // unresolved_choice [FR-029-05]: malformed inputs (wrong element count,
    // non-numeric, non-finite) and wrong argument counts are rejected with a
    // clear error rather than guessed.
    #[test]
    fn unresolved_choice_invalid_inputs_error() {
        let good = date_vector([2020.0, 1.0, 1.0, 0.0, 0.0, 0.0]);

        // Wrong argument count.
        assert!(run_err(vec![good.clone()]).to_string().contains("etime"));

        // Five-element vector.
        let short =
            Value::Tensor(Tensor::new(vec![2020.0, 1.0, 1.0, 0.0, 0.0], vec![1, 5]).unwrap());
        assert!(run_err(vec![short, good.clone()])
            .to_string()
            .contains("etime"));

        // Non-finite component.
        let nan = date_vector([2020.0, 1.0, 1.0, 0.0, 0.0, f64::NAN]);
        assert!(run_err(vec![nan, good]).to_string().contains("etime"));
    }

    #[test]
    fn descriptor_advertises_primary_signature() {
        assert_eq!(ETIME_DESCRIPTOR.signatures[0].label, "s = etime(t2, t1)");
    }
}
