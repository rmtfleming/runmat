//! Clean-room `dec2base` (non-negative integers as text in base 2..36).
//!
//! Clean-room feature `specs/045-text-conversion/`. Normative behaviour comes
//! only from the approved export `dec2base` 0.2.0 (see that feature's
//! `provenance.md`). Observed `MATLAB:dec2base:*` identifiers are raised as
//! `RunMat:dec2base:*` (decision D-045-1). Char bases other than the observed
//! two-character case raise `RunMat:dec2base:UnsupportedOption` (D-045-2).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{CharArray, NumericDType, Value};

use crate::builtins::common::map_control_flow_with_builtin;
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const NAME: &str = "dec2base";

/// Largest value with observed behaviour (dec2base.large); larger values are
/// unresolved (dec2base.q-large-values).
const MAX_SPECIFIED: u64 = 1 << 52;

const DIGITS: &[u8; 36] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";

// ---------------------------------------------------------------------------
// Descriptor
// ---------------------------------------------------------------------------

const D_PARAM: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "d",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Non-negative integer value(s), double or integer class.",
};
const BASE_PARAM: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "base",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Integer base from 2 to 36.",
};
const N_PARAM: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "n",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Minimum number of digits (left-padded with '0').",
};

const DEC2BASE_INPUTS_2: [BuiltinParamDescriptor; 2] = [D_PARAM, BASE_PARAM];
const DEC2BASE_INPUTS_3: [BuiltinParamDescriptor; 3] = [D_PARAM, BASE_PARAM, N_PARAM];

const DEC2BASE_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "str",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Char array with one row per element of d.",
}];

const DEC2BASE_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "str = dec2base(d, base)",
        inputs: &DEC2BASE_INPUTS_2,
        outputs: &DEC2BASE_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "str = dec2base(d, base, n)",
        inputs: &DEC2BASE_INPUTS_3,
        outputs: &DEC2BASE_OUTPUT,
    },
];

const FIRST_ARG: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.DEC2BASE.FIRST_ARG",
    identifier: Some("RunMat:dec2base:FirstArg"),
    when: "d has a negative, non-integer or non-finite element.",
    message: "dec2base: d must contain non-negative integers",
};

const SECOND_ARG: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.DEC2BASE.SECOND_ARG",
    identifier: Some("RunMat:dec2base:SecondArg"),
    when: "base is not an integer scalar from 2 to 36, or is a two-character char row.",
    message: "dec2base: base must be an integer from 2 to 36",
};

const NOT_ENOUGH_INPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.DEC2BASE.NOT_ENOUGH_INPUTS",
    identifier: Some("RunMat:dec2base:NotEnoughInputs"),
    when: "Fewer than two arguments are given.",
    message: "dec2base: not enough input arguments",
};

const TOO_MANY_INPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.DEC2BASE.TOO_MANY_INPUTS",
    identifier: Some("RunMat:dec2base:TooManyInputs"),
    when: "More than three arguments are given.",
    message: "dec2base: too many input arguments",
};

const UNSUPPORTED_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.DEC2BASE.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:dec2base:UnsupportedOption"),
    when: "Unobserved forms: char/string/logical bases (other than a two-character char row), values above 2^52, unsupported classes of d, invalid n.",
    message: "dec2base: this form is not yet specified",
};

const INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.DEC2BASE.INTERNAL",
    identifier: Some("RunMat:dec2base:Internal"),
    when: "The runtime cannot assemble the result.",
    message: "dec2base: internal runtime failure",
};

const DEC2BASE_ERRORS: [BuiltinErrorDescriptor; 6] = [
    FIRST_ARG,
    SECOND_ARG,
    NOT_ENOUGH_INPUTS,
    TOO_MANY_INPUTS,
    UNSUPPORTED_OPTION,
    INTERNAL,
];

pub const DEC2BASE_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &DEC2BASE_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &DEC2BASE_ERRORS,
};

fn d2b_error(
    descriptor: &'static BuiltinErrorDescriptor,
    detail: impl Into<String>,
) -> RuntimeError {
    let mut builder = build_runtime_error(format!("{NAME}: {}", detail.into())).with_builtin(NAME);
    if let Some(identifier) = descriptor.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

fn unsupported(detail: impl Into<String>) -> RuntimeError {
    d2b_error(&UNSUPPORTED_OPTION, detail)
}

// ---------------------------------------------------------------------------
// Builtin
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "dec2base",
    category = "strings/core",
    summary = "Convert non-negative integers to text in a base from 2 to 36.",
    keywords = "dec2base,base conversion,radix,hexadecimal,digits",
    accel = "cpu",
    descriptor(crate::builtins::strings::core::dec2base::DEC2BASE_DESCRIPTOR),
    builtin_path = "crate::builtins::strings::core::dec2base"
)]
async fn dec2base_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    if args.len() < 2 {
        return Err(d2b_error(
            &NOT_ENOUGH_INPUTS,
            "expected dec2base(d, base) or dec2base(d, base, n)",
        ));
    }
    if args.len() > 3 {
        return Err(d2b_error(
            &TOO_MANY_INPUTS,
            "expected dec2base(d, base) or dec2base(d, base, n)",
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
    let values = element_values(&host[0])?;
    let base = parse_base(&host[1])?;
    let min_digits = match host.get(2) {
        Some(n) => parse_min_digits(n)?,
        None => 0,
    };

    if values.is_empty() {
        return CharArray::new(Vec::new(), 0, 0)
            .map(Value::CharArray)
            .map_err(|err| d2b_error(&INTERNAL, err));
    }
    let digits: Vec<Vec<u8>> = values.iter().map(|&v| to_digits(v, base)).collect();
    let width = digits
        .iter()
        .map(Vec::len)
        .max()
        .unwrap_or(1)
        .max(min_digits);
    // Char payloads are row-major: one padded row per element (dec2base.vector).
    let mut data = Vec::with_capacity(digits.len() * width);
    for row in &digits {
        data.extend(std::iter::repeat_n('0', width - row.len()));
        data.extend(row.iter().map(|&b| b as char));
    }
    CharArray::new(data, digits.len(), width)
        .map(Value::CharArray)
        .map_err(|err| d2b_error(&INTERNAL, err))
}

/// Digits of `value` in `base`, most significant first; 0 gives "0"
/// (dec2base.digits, dec2base.zero). Textbook repeated division.
fn to_digits(mut value: u64, base: u64) -> Vec<u8> {
    if value == 0 {
        return vec![b'0'];
    }
    let mut out = Vec::new();
    while value > 0 {
        out.push(DIGITS[(value % base) as usize]);
        value /= base;
    }
    out.reverse();
    out
}

fn check_double(x: f64) -> BuiltinResult<u64> {
    // dec2base.error-negative (NaN/Inf: FR-045-21)
    if !x.is_finite() || x < 0.0 || x.fract() != 0.0 {
        return Err(d2b_error(
            &FIRST_ARG,
            "d must contain non-negative integers",
        ));
    }
    if x > MAX_SPECIFIED as f64 {
        return Err(unsupported("values above 2^52 are not yet specified"));
    }
    Ok(x as u64)
}

fn check_integer(x: i128) -> BuiltinResult<u64> {
    if x < 0 {
        return Err(d2b_error(
            &FIRST_ARG,
            "d must contain non-negative integers",
        ));
    }
    if x > MAX_SPECIFIED as i128 {
        return Err(unsupported("values above 2^52 are not yet specified"));
    }
    Ok(x as u64)
}

/// Elements of `d` in column-major order (FR-045-09, FR-045-21).
fn element_values(value: &Value) -> BuiltinResult<Vec<u64>> {
    match value {
        Value::Num(x) => Ok(vec![check_double(*x)?]),
        Value::Int(i) => Ok(vec![check_integer(i.to_i128())?]),
        Value::Tensor(tensor) => {
            if let Some(storage) = tensor.integer_storage() {
                return storage
                    .exact_values()
                    .iter()
                    .map(|v| check_integer(v.to_i128()))
                    .collect();
            }
            if tensor.numeric_dtype() != NumericDType::F64 {
                return Err(unsupported("single-precision d is not yet specified"));
            }
            tensor
                .materialize_f64()
                .into_iter()
                .map(check_double)
                .collect()
        }
        other => Err(unsupported(format!(
            "d of class {} is not yet specified",
            crate::builtins::introspection::class::class_name_for_value(other)
        ))),
    }
}

/// A real double or integer scalar, without range checks.
fn numeric_scalar(value: &Value) -> Option<f64> {
    match value {
        Value::Num(x) => Some(*x),
        Value::Int(i) => Some(i.to_i128() as f64),
        Value::Tensor(tensor) if tensor.shape.iter().product::<usize>() == 1 => {
            if let Some(storage) = tensor.integer_storage() {
                return storage.exact_values().first().map(|v| v.to_i128() as f64);
            }
            if tensor.numeric_dtype() != NumericDType::F64 {
                return None;
            }
            tensor.materialize_f64().first().copied()
        }
        _ => None,
    }
}

fn is_numeric(value: &Value) -> bool {
    matches!(value, Value::Num(_) | Value::Int(_) | Value::Tensor(_))
}

fn parse_base(value: &Value) -> BuiltinResult<u64> {
    let invalid = || d2b_error(&SECOND_ARG, "base must be an integer from 2 to 36");
    match value {
        // dec2base.error-base1 ('ab'); other char forms: D-045-2.
        Value::CharArray(chars) if chars.rows == 1 && chars.cols == 2 => Err(invalid()),
        Value::CharArray(_) | Value::String(_) | Value::StringArray(_) => {
            Err(unsupported("symbol-set (text) bases are not yet specified"))
        }
        _ if is_numeric(value) => {
            let Some(base) = numeric_scalar(value) else {
                if matches!(value, Value::Tensor(t) if t.integer_storage().is_none() && t.numeric_dtype() != NumericDType::F64)
                {
                    return Err(unsupported("single-precision base is not yet specified"));
                }
                return Err(invalid());
            };
            if base.fract() != 0.0 || !(2.0..=36.0).contains(&base) {
                return Err(invalid());
            }
            Ok(base as u64)
        }
        other => Err(unsupported(format!(
            "base of class {} is not yet specified",
            crate::builtins::introspection::class::class_name_for_value(other)
        ))),
    }
}

fn parse_min_digits(value: &Value) -> BuiltinResult<usize> {
    let invalid = || unsupported("n must be a non-negative integer scalar");
    if !is_numeric(value) {
        return Err(invalid());
    }
    let n = numeric_scalar(value).ok_or_else(invalid)?;
    if !n.is_finite() || n < 0.0 || n.fract() != 0.0 || n > u32::MAX as f64 {
        return Err(invalid());
    }
    Ok(n as usize)
}

#[cfg(test)]
mod dec2base_tests {
    use crate::BuiltinResult;
    use futures::executor::block_on;
    use runmat_value::{
        CharArray, IntValue, IntegerStorage, LogicalArray, StringArray, Tensor, Value,
    };

    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("dec2base", args))
    }

    fn n(x: f64) -> Value {
        Value::Num(x)
    }

    fn row(values: &[f64]) -> Value {
        Value::Tensor(Tensor::new(values.to_vec(), vec![1, values.len()]).unwrap())
    }

    fn col(values: &[f64]) -> Value {
        Value::Tensor(Tensor::new(values.to_vec(), vec![values.len(), 1]).unwrap())
    }

    fn chars(s: &str) -> Value {
        Value::CharArray(CharArray::new_row(s))
    }

    /// Rows of a char result, with its (rows, cols).
    fn char_rows(value: &Value) -> ((usize, usize), Vec<String>) {
        let Value::CharArray(ca) = value else {
            panic!("expected char, got {value:?}");
        };
        let rows = (0..ca.rows)
            .map(|r| ca.data[r * ca.cols..(r + 1) * ca.cols].iter().collect())
            .collect();
        ((ca.rows, ca.cols), rows)
    }

    fn assert_rows(value: Value, expected: &[&str]) {
        let (shape, rows) = char_rows(&value);
        let width = expected.first().map_or(0, |s| s.len());
        assert_eq!(shape, (expected.len(), width));
        assert_eq!(rows, expected);
    }

    fn identifier(result: BuiltinResult<Value>) -> String {
        match result {
            Ok(value) => panic!("expected an error, got {value:?}"),
            Err(err) => err.identifier().unwrap_or("<none>").to_string(),
        }
    }

    const FIRST: &str = "RunMat:dec2base:FirstArg";
    const SECOND: &str = "RunMat:dec2base:SecondArg";
    const UNSUPPORTED: &str = "RunMat:dec2base:UnsupportedOption";

    // --- Observed (normative): one test per observed case --------------------

    /// case `base2` — dec2base.signature-primary, dec2base.output-class, dec2base.digits
    #[test]
    fn observed_base2() {
        assert_rows(call(&[n(23.0), n(2.0)]).unwrap(), &["10111"]);
    }

    /// case `base3` — dec2base.digits
    #[test]
    fn observed_base3() {
        assert_rows(call(&[n(23.0), n(3.0)]).unwrap(), &["212"]);
    }

    /// case `base16` — dec2base.digits
    #[test]
    fn observed_base16() {
        assert_rows(call(&[n(255.0), n(16.0)]).unwrap(), &["FF"]);
    }

    /// case `base36` — dec2base.vector
    #[test]
    fn observed_base36_pair() {
        assert_rows(call(&[row(&[35.0, 36.0]), n(36.0)]).unwrap(), &["0Z", "10"]);
    }

    /// case `zero` — dec2base.zero
    #[test]
    fn observed_zero() {
        assert_rows(call(&[n(0.0), n(7.0)]).unwrap(), &["0"]);
    }

    /// case `base36-z` — dec2base.digits
    #[test]
    fn observed_base36_z() {
        assert_rows(call(&[n(35.0), n(36.0)]).unwrap(), &["Z"]);
    }

    /// case `pad` — dec2base.padding
    #[test]
    fn observed_pad() {
        assert_rows(call(&[n(5.0), n(2.0), n(8.0)]).unwrap(), &["00000101"]);
    }

    /// case `pad-short` — dec2base.padding
    #[test]
    fn observed_pad_short() {
        assert_rows(call(&[n(255.0), n(2.0), n(3.0)]).unwrap(), &["11111111"]);
    }

    /// case `vector` — dec2base.vector, dec2base.output-class
    #[test]
    fn observed_column_vector() {
        assert_rows(
            call(&[col(&[1.0, 10.0, 100.0]), n(10.0)]).unwrap(),
            &["001", "010", "100"],
        );
    }

    /// case `row-vector` — dec2base.vector
    #[test]
    fn observed_row_vector() {
        assert_rows(
            call(&[row(&[1.0, 2.0, 3.0]), n(2.0)]).unwrap(),
            &["01", "10", "11"],
        );
    }

    /// case `err-symbols` — dec2base.error-base1 (D-045-2)
    #[test]
    fn observed_two_char_symbol_base_errors() {
        assert_eq!(identifier(call(&[n(5.0), chars("ab")])), SECOND);
    }

    /// case `uint8` — dec2base.integer-class, dec2base.digits
    #[test]
    fn observed_uint8() {
        assert_rows(
            call(&[Value::Int(IntValue::U8(200)), n(16.0)]).unwrap(),
            &["C8"],
        );
    }

    /// case `large` — dec2base.large
    #[test]
    fn observed_two_pow_52() {
        assert_rows(
            call(&[n(2f64.powi(52)), n(16.0)]).unwrap(),
            &["10000000000000"],
        );
    }

    /// case `err-negative` — dec2base.error-negative
    #[test]
    fn observed_negative_errors() {
        assert_eq!(identifier(call(&[n(-1.0), n(2.0)])), FIRST);
    }

    /// case `err-fraction` — dec2base.error-negative
    #[test]
    fn observed_fraction_errors() {
        assert_eq!(identifier(call(&[n(1.5), n(2.0)])), FIRST);
    }

    /// case `err-base1` — dec2base.error-base1
    #[test]
    fn observed_base1_errors() {
        assert_eq!(identifier(call(&[n(5.0), n(1.0)])), SECOND);
    }

    /// case `err-base37` — dec2base.error-base1
    #[test]
    fn observed_base37_errors() {
        assert_eq!(identifier(call(&[n(5.0), n(37.0)])), SECOND);
    }

    // --- Documented choices (FR-045-21..24) and unresolved points ------------

    /// FR-045-21
    #[test]
    fn documented_choice_shapes_and_classes_of_d() {
        assert_eq!(
            identifier(call(&[n(1.0)])),
            "RunMat:dec2base:NotEnoughInputs"
        );
        assert_eq!(
            identifier(call(&[n(1.0), n(2.0), n(3.0), n(4.0)])),
            "RunMat:dec2base:TooManyInputs"
        );
        // Matrix: column-major element order, one row each.
        let m = Value::Tensor(Tensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap());
        assert_rows(call(&[m, n(10.0)]).unwrap(), &["1", "2", "3", "4"]);
        // Empty d gives 0x0 char.
        let empty = Value::Tensor(Tensor::new(Vec::new(), vec![0, 0]).unwrap());
        let (shape, _) = char_rows(&call(&[empty, n(2.0), n(4.0)]).unwrap());
        assert_eq!(shape, (0, 0));
        // Integer-class arrays.
        let ints = Value::Tensor(
            Tensor::new_integer(IntegerStorage::I16(vec![7, 8]), vec![1, 2]).unwrap(),
        );
        assert_rows(call(&[ints, n(8.0)]).unwrap(), &["07", "10"]);
        let neg = Value::Int(IntValue::I32(-3));
        assert_eq!(identifier(call(&[neg, n(2.0)])), FIRST);
        // NaN and Inf are not non-negative integers.
        assert_eq!(identifier(call(&[n(f64::NAN), n(2.0)])), FIRST);
        assert_eq!(identifier(call(&[n(f64::INFINITY), n(2.0)])), FIRST);
        // Unsupported classes.
        for d in [
            Value::Bool(true),
            Value::LogicalArray(LogicalArray::new(vec![1, 0], vec![1, 2]).unwrap()),
            Value::Tensor(Tensor::from_f32(vec![3.0], vec![1, 1]).unwrap()),
            Value::Complex(1.0, 1.0),
            chars("5"),
            Value::String("5".into()),
        ] {
            assert_eq!(identifier(call(&[d.clone(), n(2.0)])), UNSUPPORTED, "{d:?}");
        }
    }

    /// FR-045-22
    #[test]
    fn documented_choice_base_forms() {
        assert_rows(
            call(&[n(255.0), Value::Int(IntValue::U8(16))]).unwrap(),
            &["FF"],
        );
        assert_rows(call(&[n(10.0), n(36.0)]).unwrap(), &["A"]);
        assert_eq!(identifier(call(&[n(5.0), n(2.5)])), SECOND);
        assert_eq!(identifier(call(&[n(5.0), n(0.0)])), SECOND);
        assert_eq!(identifier(call(&[n(5.0), n(f64::NAN)])), SECOND);
        assert_eq!(identifier(call(&[n(5.0), row(&[2.0, 3.0])])), SECOND);
        assert_eq!(identifier(call(&[n(5.0), Value::Bool(true)])), UNSUPPORTED);
    }

    /// D-045-2 (flagged open question dec2base.q-char-base)
    #[test]
    fn unresolved_char_bases_are_unsupported() {
        for base in ["0123456789", "abc", "a", ""] {
            assert_eq!(
                identifier(call(&[n(5.0), chars(base)])),
                UNSUPPORTED,
                "{base}"
            );
        }
        assert_eq!(
            identifier(call(&[n(5.0), Value::String("ab".into())])),
            UNSUPPORTED
        );
        let sa = Value::StringArray(StringArray::new(vec!["ab".into()], vec![1, 1]).unwrap());
        assert_eq!(identifier(call(&[n(5.0), sa])), UNSUPPORTED);
    }

    /// FR-045-23 (unresolved, dec2base.q-large-values)
    #[test]
    fn unresolved_values_above_two_pow_52_are_unsupported() {
        assert_eq!(
            identifier(call(&[n(2f64.powi(52) + 2.0), n(16.0)])),
            UNSUPPORTED
        );
        assert_eq!(
            identifier(call(&[Value::Int(IntValue::U64(u64::MAX)), n(16.0)])),
            UNSUPPORTED
        );
        assert_rows(
            call(&[Value::Int(IntValue::U64(1 << 52)), n(16.0)]).unwrap(),
            &["10000000000000"],
        );
    }

    /// FR-045-24
    #[test]
    fn documented_choice_digit_count_forms() {
        assert_rows(call(&[n(5.0), n(2.0), n(0.0)]).unwrap(), &["101"]);
        assert_rows(
            call(&[n(5.0), n(2.0), Value::Int(IntValue::U8(4))]).unwrap(),
            &["0101"],
        );
        assert_rows(
            call(&[row(&[1.0, 2.0]), n(10.0), n(3.0)]).unwrap(),
            &["001", "002"],
        );
        for bad in [n(-1.0), n(1.5), row(&[1.0, 2.0]), chars("3"), n(f64::NAN)] {
            assert_eq!(
                identifier(call(&[n(5.0), n(2.0), bad.clone()])),
                UNSUPPORTED,
                "{bad:?}"
            );
        }
    }

    /// FR-045-28 (unresolved, dec2base.q-warning-coverage)
    #[test]
    fn unresolved_no_warning_emitted() {
        crate::warning_store::reset();
        let _ = call(&[n(23.0), n(2.0)]).unwrap();
        assert!(crate::warning_store::take_all().is_empty());
    }
}
