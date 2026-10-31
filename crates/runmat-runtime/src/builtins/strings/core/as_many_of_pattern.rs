//! Clean-room `asManyOfPattern` (greedy repetition of a text pattern).
//!
//! Clean-room feature `specs/057-text-parsing/`. Normative behaviour comes
//! only from the approved export `asManyOfPattern` 0.2.0 (see that feature's
//! `provenance.md`). Observed `MATLAB:*` identifiers are raised as
//! `RunMat:*` (D-057-1).
//!
//! RunMat represents a `pattern` as a host object of class `pattern` holding
//! a regular expression (`strings::core::compat::pattern_object`). This
//! builtin wraps the regex of `p` (text is escaped literally) in a greedy
//! quantifier: `(?:p)*`, `(?:p){min,}` or `(?:p){min,max}`. Where empty
//! matches appear (`asManyOfPattern.q-empty-match`) is non-normative; RunMat
//! inherits the leftmost-first, non-overlapping iteration of the `regex`
//! crate (documented choice FR-057-27).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{NumericDType, Value};

use crate::builtins::common::map_control_flow_with_builtin;
use crate::builtins::strings::core::compat::{pattern_object, pattern_regex};
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const NAME: &str = "asManyOfPattern";

// ---------------------------------------------------------------------------
// Descriptor
// ---------------------------------------------------------------------------

const fn param(
    name: &'static str,
    arity: BuiltinParamArity,
    description: &'static str,
) -> BuiltinParamDescriptor {
    BuiltinParamDescriptor {
        name,
        ty: BuiltinParamType::Any,
        arity,
        default: None,
        description,
    }
}

const IN_P: BuiltinParamDescriptor = param(
    "p",
    BuiltinParamArity::Required,
    "pattern object, string scalar or char row vector.",
);
const IN_MIN: BuiltinParamDescriptor = param(
    "min",
    BuiltinParamArity::Required,
    "Minimum number of repetitions (non-negative integer).",
);
const IN_MAX: BuiltinParamDescriptor = param(
    "max",
    BuiltinParamArity::Required,
    "Maximum number of repetitions (integer not less than min, or Inf).",
);
const OUT_PAT: BuiltinParamDescriptor =
    param("pat", BuiltinParamArity::Required, "pattern object.");

const INPUTS_P: [BuiltinParamDescriptor; 1] = [IN_P];
const INPUTS_P_MIN: [BuiltinParamDescriptor; 2] = [IN_P, IN_MIN];
const INPUTS_P_MIN_MAX: [BuiltinParamDescriptor; 3] = [IN_P, IN_MIN, IN_MAX];
const OUTPUTS: [BuiltinParamDescriptor; 1] = [OUT_PAT];

const SIGNATURES: [BuiltinSignatureDescriptor; 3] = [
    BuiltinSignatureDescriptor {
        label: "pat = asManyOfPattern(p)",
        inputs: &INPUTS_P,
        outputs: &OUTPUTS,
    },
    BuiltinSignatureDescriptor {
        label: "pat = asManyOfPattern(p, min)",
        inputs: &INPUTS_P_MIN,
        outputs: &OUTPUTS,
    },
    BuiltinSignatureDescriptor {
        label: "pat = asManyOfPattern(p, min, max)",
        inputs: &INPUTS_P_MIN_MAX,
        outputs: &OUTPUTS,
    },
];

const INVALID_LIMIT_RANGE: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.AS_MANY_OF_PATTERN.INVALID_LIMIT_RANGE",
    identifier: Some("RunMat:pattern:InvalidLimitRange"),
    when: "min is greater than max.",
    message: "asManyOfPattern: min must not exceed max",
};

const MUST_BE_NONNEGATIVE_INTEGER: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.AS_MANY_OF_PATTERN.MUST_BE_NONNEGATIVE_INTEGER",
    identifier: Some("RunMat:pattern:MustBeNonnegativeInteger"),
    when: "min or max is a negative, non-integer or NaN numeric scalar.",
    message: "asManyOfPattern: counts must be non-negative integers",
};

const MUST_BE_TEXT_OR_PATTERN: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.AS_MANY_OF_PATTERN.MUST_BE_TEXT_OR_PATTERN",
    identifier: Some("RunMat:pattern:MustBeTextOrPattern"),
    when: "p is numeric.",
    message: "asManyOfPattern: p must be text or a pattern",
};

const INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.AS_MANY_OF_PATTERN.INVALID_INPUT",
    identifier: Some("RunMat:asManyOfPattern:InvalidInput"),
    when: "The call does not have one to three inputs.",
    message: "asManyOfPattern: expected asManyOfPattern(p, min, max)",
};

const UNSUPPORTED_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.AS_MANY_OF_PATTERN.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:asManyOfPattern:UnsupportedOption"),
    when: "Unobserved inputs: non-text, non-numeric p (logical, cell, ...), text arrays, non-numeric or non-scalar counts.",
    message: "asManyOfPattern: this form is not yet specified",
};

const TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.AS_MANY_OF_PATTERN.TOO_MANY_OUTPUTS",
    identifier: Some("RunMat:asManyOfPattern:TooManyOutputs"),
    when: "More than one output was requested.",
    message: "asManyOfPattern: too many output arguments",
};

const ERRORS: [BuiltinErrorDescriptor; 6] = [
    INVALID_LIMIT_RANGE,
    MUST_BE_NONNEGATIVE_INTEGER,
    MUST_BE_TEXT_OR_PATTERN,
    INVALID_INPUT,
    UNSUPPORTED_OPTION,
    TOO_MANY_OUTPUTS,
];

pub const AS_MANY_OF_PATTERN_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ERRORS,
};

fn amop_error(
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

#[runtime_builtin(
    name = "asManyOfPattern",
    category = "strings/pattern",
    summary = "Pattern matching as many consecutive repetitions of a pattern as possible.",
    keywords = "asManyOfPattern,pattern,repetition,greedy,text",
    accel = "metadata",
    descriptor(crate::builtins::strings::core::as_many_of_pattern::AS_MANY_OF_PATTERN_DESCRIPTOR),
    builtin_path = "crate::builtins::strings::core::as_many_of_pattern"
)]
pub(crate) async fn as_many_of_pattern_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    // FR-057-29
    if let Some(count) = crate::output_count::current_output_count() {
        if count > 1 {
            return Err(amop_error(
                &TOO_MANY_OUTPUTS,
                format!("asManyOfPattern returns one output, {count} were requested"),
            ));
        }
    }
    if args.is_empty() || args.len() > 3 {
        return Err(amop_error(
            &INVALID_INPUT,
            "expected asManyOfPattern(p), (p, min) or (p, min, max)",
        ));
    }
    let mut host = Vec::with_capacity(args.len());
    for value in &args {
        host.push(
            gather_if_needed_async(value)
                .await
                .map_err(|err| map_control_flow_with_builtin(err, NAME))?,
        );
    }
    let inner = inner_regex(&host[0])?;
    let quantifier = match (host.get(1), host.get(2)) {
        // FR-057-16: zero or more.
        (None, _) => "*".to_string(),
        // FR-057-14
        (Some(min), None) => format!("{{{},}}", count(min, false)?.unwrap_or(0)),
        // FR-057-17, FR-057-19, FR-057-21
        (Some(min), Some(max)) => {
            let min = count(min, false)?.unwrap_or(0);
            match count(max, true)? {
                None => format!("{{{min},}}"),
                Some(max) if min > max => {
                    return Err(amop_error(
                        &INVALID_LIMIT_RANGE,
                        format!("min ({min}) is greater than max ({max})"),
                    ))
                }
                Some(max) => format!("{{{min},{max}}}"),
            }
        }
    };
    // FR-057-13
    Ok(pattern_object(&format!("(?:{inner}){quantifier}")))
}

fn is_numeric(value: &Value) -> bool {
    matches!(
        value,
        Value::Num(_) | Value::Int(_) | Value::Tensor(_) | Value::Complex(_, _)
    ) || matches!(value, Value::ComplexTensor(_) | Value::SparseTensor(_))
}

/// FR-057-18, FR-057-22, FR-057-30.
fn inner_regex(p: &Value) -> BuiltinResult<String> {
    match p {
        Value::Object(_) => pattern_regex(p, NAME)
            .map_err(|err| amop_error(&UNSUPPORTED_OPTION, err.message().to_string())),
        Value::String(_) => pattern_regex(p, NAME)
            .map_err(|err| amop_error(&UNSUPPORTED_OPTION, err.message().to_string())),
        Value::StringArray(array) if array.data.len() == 1 => pattern_regex(p, NAME)
            .map_err(|err| amop_error(&UNSUPPORTED_OPTION, err.message().to_string())),
        Value::CharArray(chars) if chars.rows <= 1 => pattern_regex(p, NAME)
            .map_err(|err| amop_error(&UNSUPPORTED_OPTION, err.message().to_string())),
        other if is_numeric(other) => Err(amop_error(
            &MUST_BE_TEXT_OR_PATTERN,
            "p must be text or a pattern, not numeric",
        )),
        _ => Err(amop_error(
            &UNSUPPORTED_OPTION,
            "p must be a pattern, a string scalar or a char row vector",
        )),
    }
}

/// A count argument. `Ok(None)` means unbounded (`+Inf`, allowed only for
/// max: documented choice FR-057-28).
fn count(value: &Value, allow_inf: bool) -> BuiltinResult<Option<usize>> {
    let x = match value {
        Value::Num(v) => *v,
        Value::Int(i) => i.to_f64(),
        Value::Tensor(t)
            if t.shape.iter().product::<usize>() == 1
                && (t.integer_storage().is_some()
                    || matches!(t.numeric_dtype(), NumericDType::F64 | NumericDType::F32)) =>
        {
            t.materialize_f64()[0]
        }
        _ => {
            return Err(amop_error(
                &UNSUPPORTED_OPTION,
                "counts must be real numeric scalars",
            ))
        }
    };
    if allow_inf && x == f64::INFINITY {
        return Ok(None);
    }
    if !x.is_finite() || x < 0.0 || x.fract() != 0.0 || x > u32::MAX as f64 {
        return Err(amop_error(
            &MUST_BE_NONNEGATIVE_INTEGER,
            format!("count {x} is not a non-negative integer"),
        ));
    }
    Ok(Some(x as usize))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod as_many_of_pattern_tests {
    use super::as_many_of_pattern_builtin;
    use crate::builtins::strings::core::compat::pattern_regex;
    use crate::BuiltinResult;
    use futures::executor::block_on;
    use runmat_value::{CharArray, LogicalArray, StringArray, Value};

    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("asManyOfPattern", args))
    }

    fn builtin(name: &str, args: &[Value]) -> Value {
        block_on(crate::call_builtin_async(name, args)).unwrap()
    }

    fn s(text: &str) -> Value {
        Value::String(text.to_string())
    }

    fn strings(items: &[&str]) -> Value {
        Value::StringArray(
            StringArray::new(
                items.iter().map(|t| t.to_string()).collect(),
                vec![1, items.len()],
            )
            .unwrap(),
        )
    }

    /// Test stand-in for `extract` (absent upstream, D-057-3): every
    /// leftmost-first, non-overlapping match of the pattern's regex.
    fn extract(text: &str, pat: &Value) -> Vec<String> {
        let regex = regex::Regex::new(&pattern_regex(pat, "test").unwrap()).unwrap();
        regex
            .find_iter(text)
            .map(|m| m.as_str().to_string())
            .collect()
    }

    fn logical(value: Value) -> Vec<u8> {
        match value {
            Value::LogicalArray(array) => array.data.to_vec(),
            Value::Bool(b) => vec![u8::from(b)],
            other => panic!("expected logical, got {other:?}"),
        }
    }

    /// Stand-in for `alphanumericsPattern | "_"` (both absent upstream,
    /// D-057-3): one or more letters or digits, or an underscore.
    fn alnum_or_underscore() -> Value {
        builtin("regexpPattern", &[s(r"[\p{Alphabetic}\p{N}]+|_")])
    }

    fn cobra() -> Value {
        call(&[alnum_or_underscore(), Value::Num(1.0)]).unwrap()
    }

    fn identifier(result: BuiltinResult<Value>) -> String {
        match result {
            Ok(value) => panic!("expected an error, got {value:?}"),
            Err(err) => err.identifier().unwrap_or("<none>").to_string(),
        }
    }

    // --- Observed (normative): one test per observed case --------------------

    /// case `class` — asManyOfPattern.signature-primary, asManyOfPattern.class
    #[test]
    fn observed_class() {
        let class = builtin("class", &[cobra()]);
        let text = match class {
            Value::CharArray(chars) => chars.data.iter().collect::<String>(),
            Value::String(text) => text,
            other => panic!("unexpected class result {other:?}"),
        };
        assert_eq!(text, "pattern");
    }

    /// case `cobra-extract` — asManyOfPattern.cobra-pattern
    #[test]
    fn observed_cobra_extract() {
        assert_eq!(extract("ab_1 cd-e", &cobra()), vec!["ab_1", "cd", "e"]);
    }

    /// case `min1-no-empty` — asManyOfPattern.cobra-pattern
    #[test]
    fn observed_min1_no_empty() {
        assert!(extract("--", &cobra()).is_empty());
    }

    /// case `matches` — asManyOfPattern.cobra-pattern
    #[test]
    fn observed_matches() {
        let out = builtin("matches", &[strings(&["ab_1", "ab 1", ""]), cobra()]);
        assert_eq!(logical(out), vec![1, 0, 0]);
    }

    /// case `contains` — asManyOfPattern.cobra-pattern
    #[test]
    fn observed_contains() {
        let out = builtin("contains", &[strings(&["--", "-a-"]), cobra()]);
        assert_eq!(logical(out), vec![0, 1]);
    }

    /// case `default-min` — asManyOfPattern.greedy
    #[test]
    fn observed_default_min() {
        let letter = builtin("lettersPattern", &[Value::Num(1.0)]);
        let pat = call(&[letter]).unwrap();
        assert_eq!(extract("ab cd", &pat), vec!["ab", "cd"]);
    }

    /// case `min-max` — asManyOfPattern.min-max
    #[test]
    fn observed_min_max() {
        let digit = builtin("digitsPattern", &[Value::Num(1.0)]);
        let pat = call(&[digit, Value::Num(2.0), Value::Num(3.0)]).unwrap();
        assert_eq!(extract("1 22 333 4444", &pat), vec!["22", "333", "444"]);
    }

    /// case `string-input-min` — asManyOfPattern.text-pattern
    #[test]
    fn observed_string_input_min() {
        let pat = call(&[s("a"), Value::Num(2.0)]).unwrap();
        assert_eq!(extract("a aa aaa", &pat), vec!["aa", "aaa"]);
    }

    /// case `char-input` — asManyOfPattern.text-pattern
    #[test]
    fn observed_char_input() {
        let pat = call(&[Value::CharArray(CharArray::new_row("a")), Value::Num(2.0)]).unwrap();
        assert_eq!(extract("a aa aaa", &pat), vec!["aa", "aaa"]);
    }

    /// case `err-min-gt-max` — asManyOfPattern.error-min-gt-max
    #[test]
    fn observed_err_min_gt_max() {
        assert_eq!(
            identifier(call(&[s("a"), Value::Num(3.0), Value::Num(2.0)])),
            "RunMat:pattern:InvalidLimitRange"
        );
    }

    /// case `err-negative` — asManyOfPattern.error-negative
    #[test]
    fn observed_err_negative() {
        assert_eq!(
            identifier(call(&[s("a"), Value::Num(-1.0)])),
            "RunMat:pattern:MustBeNonnegativeInteger"
        );
    }

    /// case `err-numeric` — asManyOfPattern.error-numeric
    #[test]
    fn observed_err_numeric() {
        assert_eq!(
            identifier(call(&[Value::Num(5.0)])),
            "RunMat:pattern:MustBeTextOrPattern"
        );
    }

    // --- Non-normative observations (asManyOfPattern.q-empty-match) ----------

    /// cases `string-input`, `default-min-string-letters` —
    /// asManyOfPattern.empty-matches (non-normative). FR-057-27 documented
    /// choice: empty matches follow the regex crate's iteration; for these
    /// two inputs that coincides with the observed outputs, but this is not
    /// claimed as MATLAB's rule.
    #[test]
    fn documented_choice_empty_matches() {
        let pat = call(&[s("a")]).unwrap();
        assert_eq!(extract("aaab", &pat), vec!["aaa", ""]);
        let pat = call(&[s("b")]).unwrap();
        assert_eq!(extract("ab cd", &pat), vec!["", "b", "", "", ""]);
    }

    // --- Documented choices --------------------------------------------------

    /// FR-057-18: text is matched literally (regex metacharacters escaped).
    #[test]
    fn documented_choice_text_is_literal() {
        let pat = call(&[s("a.")]).unwrap();
        assert_eq!(extract("a.a.ab", &pat), vec!["a.a.", "", ""]);
    }

    /// FR-057-28: max = Inf is unbounded; zero counts are allowed; non-integer
    /// and NaN counts raise MustBeNonnegativeInteger.
    #[test]
    fn documented_choice_counts() {
        let pat = call(&[s("a"), Value::Num(2.0), Value::Num(f64::INFINITY)]).unwrap();
        assert_eq!(extract("a aaaa", &pat), vec!["aaaa"]);
        let pat = call(&[s("a"), Value::Num(0.0), Value::Num(0.0)]).unwrap();
        assert!(extract("aa", &pat).iter().all(String::is_empty));
        for bad in [1.5, f64::NAN, f64::INFINITY] {
            assert_eq!(
                identifier(call(&[s("a"), Value::Num(bad)])),
                "RunMat:pattern:MustBeNonnegativeInteger"
            );
        }
        assert_eq!(
            identifier(call(&[s("a"), Value::Num(1.0), Value::Num(-2.0)])),
            "RunMat:pattern:MustBeNonnegativeInteger"
        );
    }

    /// FR-057-30: unobserved inputs.
    #[test]
    fn documented_choice_unsupported() {
        const U: &str = "RunMat:asManyOfPattern:UnsupportedOption";
        assert_eq!(identifier(call(&[Value::Bool(true)])), U);
        let mask = Value::LogicalArray(LogicalArray::new(vec![1], vec![1, 1]).unwrap());
        assert_eq!(identifier(call(&[mask])), U);
        assert_eq!(identifier(call(&[strings(&["a", "b"])])), U);
        assert_eq!(identifier(call(&[s("a"), s("2")])), U);
    }

    /// FR-057-29: arity and output count.
    #[test]
    fn documented_choice_arity_and_outputs() {
        assert_eq!(identifier(call(&[])), "RunMat:asManyOfPattern:InvalidInput");
        assert_eq!(
            identifier(call(&[
                s("a"),
                Value::Num(1.0),
                Value::Num(2.0),
                Value::Num(3.0)
            ])),
            "RunMat:asManyOfPattern:InvalidInput"
        );
        let _guard = crate::output_count::push_output_count(Some(2));
        let err = block_on(as_many_of_pattern_builtin(vec![s("a")])).unwrap_err();
        assert_eq!(
            err.identifier(),
            Some("RunMat:asManyOfPattern:TooManyOutputs")
        );
    }

    /// FR-057-31 (unresolved, asManyOfPattern.q-warning-coverage): no warning.
    #[test]
    fn unresolved_no_warning() {
        assert!(call(&[s("a")]).is_ok());
    }
}
