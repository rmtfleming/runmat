//! Clean-room `date` (current date as `DD-Mmm-YYYY` text).
//!
//! Clean-room feature `specs/053-environment/`. Normative behaviour comes
//! only from the approved export `date` 0.2.0 (see that feature's
//! `provenance.md`). The date is taken from RunMat's `clock` builtin, so it
//! agrees with `clock` by construction (decision D-053-5). Observed
//! `MATLAB:*` identifiers are raised as `RunMat:*` (D-053-1).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_value::{CharArray, Value};

use crate::{build_runtime_error, BuiltinResult, RuntimeError};

const NAME: &str = "date";

/// FR-053-26 (documented choice): English abbreviations for every month.
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

// ---------------------------------------------------------------------------
// Descriptor
// ---------------------------------------------------------------------------

const OUTPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "d",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Current date as a 1x11 char row, DD-Mmm-YYYY.",
}];

const SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "d = date",
    inputs: &[],
    outputs: &OUTPUTS,
}];

const TOO_MANY_INPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.DATE.TOO_MANY_INPUTS",
    identifier: Some("RunMat:TooManyInputs"),
    when: "An input argument is given.",
    message: "date: too many input arguments",
};

const TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.DATE.TOO_MANY_OUTPUTS",
    identifier: Some("RunMat:date:TooManyOutputs"),
    when: "More than one output was requested.",
    message: "date: too many output arguments",
};

const INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.DATE.INTERNAL",
    identifier: Some("RunMat:date:Internal"),
    when: "The clock reading cannot be turned into a date.",
    message: "date: internal runtime failure",
};

const ERRORS: [BuiltinErrorDescriptor; 3] = [TOO_MANY_INPUTS, TOO_MANY_OUTPUTS, INTERNAL];

pub const DATE_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ERRORS,
};

fn date_error(
    descriptor: &'static BuiltinErrorDescriptor,
    detail: impl Into<String>,
) -> RuntimeError {
    let mut builder = build_runtime_error(format!("{NAME}: {}", detail.into())).with_builtin(NAME);
    if let Some(identifier) = descriptor.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

// ---------------------------------------------------------------------------
// Builtin
// ---------------------------------------------------------------------------

#[runmat_macros::runtime_builtin(
    name = "date",
    category = "datetime",
    summary = "Return the current local date as a DD-Mmm-YYYY char row.",
    keywords = "date,current date,clock,today",
    descriptor(crate::builtins::datetime::date_builtin::DATE_DESCRIPTOR),
    builtin_path = "crate::builtins::datetime::date_builtin"
)]
pub(crate) async fn date_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    // FR-053-23
    if let Some(count) = crate::output_count::current_output_count() {
        if count > 1 {
            return Err(date_error(
                &TOO_MANY_OUTPUTS,
                format!("date returns one output, {count} were requested"),
            ));
        }
    }
    // FR-053-21
    if !args.is_empty() {
        return Err(date_error(
            &TOO_MANY_INPUTS,
            "date takes no input arguments",
        ));
    }
    // FR-053-20, D-053-5: read the local date from clock.
    let clock = crate::call_builtin_async("clock", &[]).await?;
    let fields = match &clock {
        Value::Tensor(t) if t.len() >= 3 => {
            let data = t.materialize_f64();
            [data[0], data[1], data[2]]
        }
        other => {
            return Err(date_error(
                &INTERNAL,
                format!("unexpected clock value {other:?}"),
            ))
        }
    };
    let text = format_date(fields[0], fields[1], fields[2])
        .ok_or_else(|| date_error(&INTERNAL, "clock returned an invalid date"))?;
    // FR-053-18, FR-053-19
    Ok(Value::CharArray(CharArray::new_row(&text)))
}

/// `DD-Mmm-YYYY` with a zero-padded day (FR-053-19, FR-053-26).
pub(crate) fn format_date(year: f64, month: f64, day: f64) -> Option<String> {
    let valid = |v: f64| v.is_finite() && v.fract() == 0.0;
    if !valid(year) || !valid(month) || !valid(day) {
        return None;
    }
    if !(1.0..=12.0).contains(&month)
        || !(1.0..=31.0).contains(&day)
        || !(0.0..=9999.0).contains(&year)
    {
        return None;
    }
    Some(format!(
        "{:02}-{}-{:04}",
        day as u32,
        MONTHS[month as usize - 1],
        year as u32
    ))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod date_tests {
    use super::{date_builtin, format_date, MONTHS};
    use crate::BuiltinResult;
    use futures::executor::block_on;
    use runmat_value::Value;

    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("date", args))
    }

    fn clock_ymd() -> (u32, u32, u32) {
        match block_on(crate::call_builtin_async("clock", &[])).unwrap() {
            Value::Tensor(t) => {
                let d = t.materialize_f64();
                (d[0] as u32, d[1] as u32, d[2] as u32)
            }
            other => panic!("unexpected clock value {other:?}"),
        }
    }

    fn date_text() -> String {
        match call(&[]).unwrap() {
            Value::CharArray(c) => {
                assert_eq!(c.shape, vec![1, 11]);
                c.data.iter().collect()
            }
            other => panic!("expected a char row, got {other:?}"),
        }
    }

    /// `^\d{2}-[A-Z][a-z]{2}-\d{4}$`, written out without a regex engine.
    fn matches_pattern(text: &str) -> bool {
        let b: Vec<char> = text.chars().collect();
        b.len() == 11
            && b[0].is_ascii_digit()
            && b[1].is_ascii_digit()
            && b[2] == '-'
            && b[3].is_ascii_uppercase()
            && b[4].is_ascii_lowercase()
            && b[5].is_ascii_lowercase()
            && b[6] == '-'
            && b[7..].iter().all(char::is_ascii_digit)
    }

    // --- Observed (normative): one test per observed case --------------------

    /// case `class-shape` — date.signature-primary, date.output-class
    #[test]
    fn observed_class_shape() {
        match call(&[]).unwrap() {
            Value::CharArray(c) => {
                assert_eq!(c.shape, vec![1, 11]);
                assert_eq!(c.rows, 1);
                assert_eq!(c.data.len(), 11);
            }
            other => panic!("expected a char row, got {other:?}"),
        }
    }

    /// case `pattern` — date.format
    #[test]
    fn observed_pattern() {
        let text = date_text();
        assert!(matches_pattern(&text), "{text:?}");
    }

    /// case `agrees-clock` — date.agrees-clock (either side of a midnight)
    #[test]
    fn observed_agrees_clock() {
        let before = clock_ymd();
        let text = date_text();
        let after = clock_ymd();
        let parse = |text: &str| {
            let day: u32 = text[0..2].parse().unwrap();
            let month = MONTHS.iter().position(|m| *m == &text[3..6]).unwrap() as u32 + 1;
            let year: u32 = text[7..11].parse().unwrap();
            (year, month, day)
        };
        let got = parse(&text);
        assert!(
            got == before || got == after,
            "{text} vs {before:?}/{after:?}"
        );
        if got.1 == 10 {
            assert_eq!(&text[3..6], "Oct");
        }
    }

    /// case `leading-zero` — date.format (two-character day field)
    #[test]
    fn observed_leading_zero() {
        let text = date_text();
        assert_eq!(text.find('-'), Some(2), "{text:?}");
    }

    /// case `err-arg` — date.error-arg
    #[test]
    fn observed_err_arg() {
        let err = call(&[Value::Num(1.0)]).unwrap_err();
        assert_eq!(err.identifier(), Some("RunMat:TooManyInputs"));
    }

    // --- Documented choices ---------------------------------------------------

    /// FR-053-26 (documented choice): zero-padded day; English month names.
    #[test]
    fn documented_choice_format_rules() {
        assert_eq!(
            format_date(2026.0, 10.0, 30.0).as_deref(),
            Some("30-Oct-2026")
        );
        assert_eq!(
            format_date(2027.0, 1.0, 5.0).as_deref(),
            Some("05-Jan-2027")
        );
        assert_eq!(
            format_date(2024.0, 2.0, 29.0).as_deref(),
            Some("29-Feb-2024")
        );
        let all: Vec<String> = (1..=12)
            .map(|m| format_date(2026.0, m as f64, 1.0).unwrap()[3..6].to_string())
            .collect();
        assert_eq!(all, MONTHS);
        assert_eq!(format_date(2026.0, 13.0, 1.0), None);
        assert_eq!(format_date(f64::NAN, 1.0, 1.0), None);
    }

    /// FR-053-23 (documented choice): more than one output.
    #[test]
    fn documented_choice_too_many_outputs() {
        let _guard = crate::output_count::push_output_count(Some(2));
        let err = block_on(date_builtin(Vec::new())).unwrap_err();
        assert_eq!(err.identifier(), Some("RunMat:date:TooManyOutputs"));
    }

    /// FR-053-27 (unresolved, date.q-warning-coverage): result only, no warning.
    #[test]
    fn unresolved_no_warning() {
        assert!(matches!(call(&[]).unwrap(), Value::CharArray(_)));
    }
}
