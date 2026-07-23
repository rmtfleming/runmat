// Parallel clean-room implementation of `datestr` (independently developed against the
// matlab-interface-spec black-box specs). 0-6-0 ships the DEFAULT implementation; this copy is
// retained unregistered for differential cross-validation. Original path: crates/runmat-runtime/src/builtins/datetime/datestr.rs
//! MATLAB-compatible `datestr` builtin for RunMat.
//!
//! Clean-room provenance: specs/008-date-formatting (spec `datestr` 0.2.0,
//! claims datestr.signature-primary, datestr.output-class,
//! datestr.char-format). Observed cases: `serial-default`
//! (`datestr(738885)` => `'30-Dec-2022'`, 1x11 char) and `serial-format`
//! (`datestr(738885, 'yyyy-mm-dd')` => `'2022-12-30'`, 1x10 char).
//!
//! Date arithmetic uses an independently implemented civil-from-days
//! conversion (the publicly documented, public-domain Hinnant algorithm),
//! with the serial-day epoch anchored so the observed cases reproduce
//! exactly. Behaviour beyond the two observed cases (time-of-day tokens,
//! fractional serial numbers, unknown tokens, non-numeric inputs) is a
//! documented independent choice — see specs/008-date-formatting/spec.md.

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    CharArray, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ShapeRequirements,
};
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "datestr",
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
    notes: "Scalar date-to-text formatting; GPU inputs are gathered immediately.",
};

pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "datestr",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Not fusible; produces character text from a scalar serial date number.",
};

const DATESTR_OUTPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "s",
    ty: BuiltinParamType::StringScalar,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Character row vector representation of the date.",
}];

const DATESTR_INPUTS: [BuiltinParamDescriptor; 2] = [
    BuiltinParamDescriptor {
        name: "t",
        ty: BuiltinParamType::NumericScalar,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Serial date number (scalar).",
    },
    BuiltinParamDescriptor {
        name: "formatOut",
        ty: BuiltinParamType::StringScalar,
        arity: BuiltinParamArity::Optional,
        default: Some("dd-mmm-yyyy"),
        description: "Output format tokens (yyyy, mm, dd, mmm, HH, MM, SS).",
    },
];

const DATESTR_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "s = datestr(t)",
        inputs: &[BuiltinParamDescriptor {
            name: "t",
            ty: BuiltinParamType::NumericScalar,
            arity: BuiltinParamArity::Required,
            default: None,
            description: "Serial date number (scalar).",
        }],
        outputs: &DATESTR_OUTPUTS,
    },
    BuiltinSignatureDescriptor {
        label: "s = datestr(t, formatOut)",
        inputs: &DATESTR_INPUTS,
        outputs: &DATESTR_OUTPUTS,
    },
];

const DATESTR_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.DATESTR.INVALID_INPUT",
    identifier: Some("RunMat:datestr:InvalidInput"),
    when: "The date input is not a finite numeric scalar serial date number.",
    message: "datestr: input must be a finite numeric scalar serial date number",
};

const DATESTR_ERROR_INVALID_FORMAT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.DATESTR.INVALID_FORMAT",
    identifier: Some("RunMat:datestr:InvalidFormat"),
    when: "The format argument is not text or contains an unsupported token.",
    message: "datestr: unsupported output format",
};

const DATESTR_ERRORS: [BuiltinErrorDescriptor; 2] =
    [DATESTR_ERROR_INVALID_INPUT, DATESTR_ERROR_INVALID_FORMAT];

pub const DATESTR_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &DATESTR_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &DATESTR_ERRORS,
};

/// Default output format: day-month-year, per observed case `serial-default`
/// (`datestr(738885)` => `'30-Dec-2022'`) [datestr.char-format].
const DEFAULT_FORMAT: &str = "dd-mmm-yyyy";

/// Serial day number of 1970-01-01 under the chosen epoch. Anchored so the
/// observed case reproduces exactly: serial 738885 => 30-Dec-2022
/// (738885 - 719529 = 19356 days after 1970-01-01). A consequence of this
/// anchoring is that serial day 1 falls on 01-Jan-0000 in the proleptic
/// Gregorian calendar (documented choice; unobserved).
const UNIX_EPOCH_SERIAL_DAY: i64 = 719_529;

const SECONDS_PER_DAY: f64 = 86_400.0;

/// English three-letter month abbreviations for the `mmm` token
/// (datestr.q-locale: locale dependence unobserved; English is the
/// documented choice).
const MONTH_ABBREV: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

async fn datestr_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(invalid_input(
            "datestr: expected datestr(t) or datestr(t, formatOut)",
        ));
    }
    let date_value = gather_if_needed_async(&args[0])
        .await
        .map_err(|err| invalid_input(format!("datestr: {}", err.message())))?;
    let serial = numeric_scalar(&date_value)?;
    let format = match args.get(1) {
        Some(value) => format_text(value)?,
        None => DEFAULT_FORMAT.to_string(),
    };
    let text = format_serial(serial, &format)?;
    Ok(Value::CharArray(CharArray::new_row(&text)))
}

fn invalid_input(message: impl Into<String>) -> RuntimeError {
    build_runtime_error(message)
        .with_builtin("datestr")
        .with_identifier(DATESTR_ERROR_INVALID_INPUT.identifier.expect("identifier"))
        .build()
}

fn invalid_format(message: impl Into<String>) -> RuntimeError {
    build_runtime_error(message)
        .with_builtin("datestr")
        .with_identifier(DATESTR_ERROR_INVALID_FORMAT.identifier.expect("identifier"))
        .build()
}

/// Only scalar numeric serial date numbers are accepted. Date vectors,
/// date text, and non-scalar arrays are listed as `pending` in the approved
/// interface and have no backing observation; rejecting them with a clear
/// error is the documented choice.
fn numeric_scalar(value: &Value) -> BuiltinResult<f64> {
    let serial = match value {
        Value::Num(n) => *n,
        Value::Int(i) => i.to_f64(),
        Value::Tensor(t) if t.data.len() == 1 => t.data[0],
        _ => return Err(invalid_input(DATESTR_ERROR_INVALID_INPUT.message)),
    };
    if !serial.is_finite() {
        return Err(invalid_input(DATESTR_ERROR_INVALID_INPUT.message));
    }
    Ok(serial)
}

fn format_text(value: &Value) -> BuiltinResult<String> {
    match value {
        Value::CharArray(chars) if chars.rows <= 1 => Ok(chars.data.iter().collect()),
        Value::String(text) => Ok(text.clone()),
        Value::StringArray(array) if array.data.len() == 1 => Ok(array.data[0].clone()),
        _ => Err(invalid_format(
            "datestr: formatOut must be a character vector or string scalar",
        )),
    }
}

/// Split a serial date number into a whole serial day and a time of day,
/// rounding the fractional day to the nearest second (fractional serials
/// are unobserved; rounding and carry are the documented choice).
fn split_serial(serial: f64) -> (i64, u32, u32, u32) {
    let mut day = serial.floor();
    let mut seconds = ((serial - day) * SECONDS_PER_DAY).round() as i64;
    if seconds >= SECONDS_PER_DAY as i64 {
        day += 1.0;
        seconds = 0;
    }
    let hour = (seconds / 3600) as u32;
    let minute = ((seconds % 3600) / 60) as u32;
    let second = (seconds % 60) as u32;
    (day as i64, hour, minute, second)
}

/// Convert a day count relative to 1970-01-01 into a proleptic Gregorian
/// (year, month, day). Independent implementation of the publicly
/// documented, public-domain civil-from-days algorithm (Howard Hinnant,
/// "chrono-Compatible Low-Level Date Algorithms").
fn civil_from_days(days_since_unix_epoch: i64) -> (i64, u32, u32) {
    let z = days_since_unix_epoch + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097); // [0, 146096]
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

/// Expand format tokens against the date components. Observed tokens:
/// `yyyy`, `mm`, `dd` (case `serial-format`) and `mmm` via the default
/// format (case `serial-default`). `HH`/`MM`/`SS` (hour/minute/second) are
/// an unobserved, documented independent choice. Any other alphabetic
/// character is rejected rather than guessed.
fn format_serial(serial: f64, format: &str) -> BuiltinResult<String> {
    let (serial_day, hour, minute, second) = split_serial(serial);
    let (year, month, day) = civil_from_days(serial_day - UNIX_EPOCH_SERIAL_DAY);
    let mut out = String::with_capacity(format.len() + 8);
    let mut i = 0usize;
    while i < format.len() {
        let rest = &format[i..];
        if rest.starts_with("yyyy") {
            out.push_str(&format!("{year:04}"));
            i += 4;
        } else if rest.starts_with("mmm") {
            out.push_str(MONTH_ABBREV[(month - 1) as usize]);
            i += 3;
        } else if rest.starts_with("mm") {
            out.push_str(&format!("{month:02}"));
            i += 2;
        } else if rest.starts_with("dd") {
            out.push_str(&format!("{day:02}"));
            i += 2;
        } else if rest.starts_with("HH") {
            out.push_str(&format!("{hour:02}"));
            i += 2;
        } else if rest.starts_with("MM") {
            out.push_str(&format!("{minute:02}"));
            i += 2;
        } else if rest.starts_with("SS") {
            out.push_str(&format!("{second:02}"));
            i += 2;
        } else {
            let c = rest.chars().next().expect("non-empty remainder");
            if c.is_ascii_alphabetic() {
                return Err(invalid_format(format!(
                    "datestr: unsupported format token starting at '{c}' in '{format}'"
                )));
            }
            out.push(c);
            i += c.len_utf8();
        }
    }
    Ok(out)
}

#[cfg(all(test, feature = "parallel_xval"))]
pub(crate) mod tests {
    use super::*;
    use futures::executor::block_on;
    use runmat_builtins::Tensor;

    fn run(args: Vec<Value>) -> Value {
        block_on(super::datestr_builtin(args)).expect("datestr")
    }

    fn run_err(args: Vec<Value>) -> RuntimeError {
        block_on(super::datestr_builtin(args)).expect_err("expected datestr error")
    }

    fn char_format(text: &str) -> Value {
        Value::CharArray(CharArray::new_row(text))
    }

    fn assert_char(value: &Value, expected: &str, rows: usize, cols: usize) {
        match value {
            Value::CharArray(chars) => {
                assert_eq!(chars.data.iter().collect::<String>(), expected);
                assert_eq!((chars.rows, chars.cols), (rows, cols));
            }
            other => panic!("expected char array, got {other:?}"),
        }
    }

    // Normative [FR-008-01, FR-008-02; datestr.signature-primary,
    // datestr.output-class, datestr.char-format] — observed case
    // `serial-default`: datestr(738885) => '30-Dec-2022' (char, 1x11).
    #[test]
    fn observed_serial_default_format() {
        assert_char(&run(vec![Value::Num(738885.0)]), "30-Dec-2022", 1, 11);
    }

    // Normative [FR-008-03; datestr.output-class, datestr.char-format] —
    // observed case `serial-format`: datestr(738885, 'yyyy-mm-dd') =>
    // '2022-12-30' (char, 1x10).
    #[test]
    fn observed_serial_explicit_format() {
        assert_char(
            &run(vec![Value::Num(738885.0), char_format("yyyy-mm-dd")]),
            "2022-12-30",
            1,
            10,
        );
    }

    // Normative [FR-008-01] — a 1x1 numeric tensor is the same observed
    // scalar serial input.
    #[test]
    fn observed_serial_from_scalar_tensor() {
        let tensor = Tensor::new(vec![738885.0], vec![1, 1]).unwrap();
        assert_char(&run(vec![Value::Tensor(tensor)]), "30-Dec-2022", 1, 11);
    }

    // unresolved_choice [FR-008-05]: epoch anchoring. The chosen epoch
    // reproduces the observed case and implies serial day 1 = 01-Jan-0000
    // and serial 719529 = 01-Jan-1970 (both unobserved).
    #[test]
    fn unresolved_choice_epoch_anchoring() {
        assert_char(&run(vec![Value::Num(1.0)]), "01-Jan-0000", 1, 11);
        assert_char(&run(vec![Value::Num(719529.0)]), "01-Jan-1970", 1, 11);
    }

    // unresolved_choice [FR-008-05]: fractional serial numbers carry a time
    // of day rendered by the HH/MM/SS tokens (unobserved; documented choice).
    #[test]
    fn unresolved_choice_time_of_day_tokens() {
        assert_char(
            &run(vec![
                Value::Num(738885.5),
                char_format("yyyy-mm-dd HH:MM:SS"),
            ]),
            "2022-12-30 12:00:00",
            1,
            19,
        );
    }

    // unresolved_choice [FR-008-05]: fractional seconds round to the nearest
    // second and carry across midnight (documented choice).
    #[test]
    fn unresolved_choice_second_rounding_carries() {
        let serial = 738885.0 + 86_399.7 / 86_400.0;
        assert_char(
            &run(vec![
                Value::Num(serial),
                char_format("dd-mmm-yyyy HH:MM:SS"),
            ]),
            "31-Dec-2022 00:00:00",
            1,
            20,
        );
    }

    // unresolved_choice [FR-008-05]: string-scalar format arguments are
    // accepted; the output stays char per datestr.output-class.
    #[test]
    fn unresolved_choice_string_format_argument() {
        assert_char(
            &run(vec![
                Value::Num(738885.0),
                Value::String("yyyy-mm-dd".to_string()),
            ]),
            "2022-12-30",
            1,
            10,
        );
    }

    // unresolved_choice [FR-008-05]: unknown alphabetic tokens are rejected
    // with a clear error instead of being guessed.
    #[test]
    fn unresolved_choice_unknown_token_errors() {
        let err = run_err(vec![Value::Num(738885.0), char_format("yyyy-QQ")]);
        assert!(err.to_string().contains("datestr"));
    }

    // unresolved_choice [FR-008-05]: non-numeric, non-scalar, and non-finite
    // date inputs are rejected (date text/date vectors are pending in the
    // approved interface, without observations).
    #[test]
    fn unresolved_choice_invalid_date_inputs_error() {
        let err = run_err(vec![char_format("30-Dec-2022")]);
        assert!(err.to_string().contains("datestr"));

        let tensor = Tensor::new(vec![738885.0, 738886.0], vec![1, 2]).unwrap();
        let err = run_err(vec![Value::Tensor(tensor)]);
        assert!(err.to_string().contains("datestr"));

        let err = run_err(vec![Value::Num(f64::NAN)]);
        assert!(err.to_string().contains("datestr"));
    }

    // unresolved_choice [FR-008-05]: argument-count validation.
    #[test]
    fn unresolved_choice_argument_count_errors() {
        let err = run_err(vec![]);
        assert!(err.to_string().contains("datestr"));

        let err = run_err(vec![
            Value::Num(738885.0),
            char_format("yyyy-mm-dd"),
            Value::Num(1.0),
        ]);
        assert!(err.to_string().contains("datestr"));
    }
}
