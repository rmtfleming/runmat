//! Clean-room `findstr` (start indices of the shorter argument in the longer).
//!
//! Clean-room feature `specs/037-findstr/`. Normative behaviour comes only
//! from the approved export `findstr` 0.3.0 (see that feature's
//! `provenance.md`). Error identifiers use the RunMat namespace with the
//! observed suffixes (batch C2 decision D2 = B precedent).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{NumericDType, Tensor, Value};

use crate::builtins::common::map_control_flow_with_builtin;
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const NAME: &str = "findstr";

// ---------------------------------------------------------------------------
// Descriptor
// ---------------------------------------------------------------------------

const FINDSTR_INPUTS: [BuiltinParamDescriptor; 2] = [
    BuiltinParamDescriptor {
        name: "s1",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Char row vector, string scalar, or double row vector.",
    },
    BuiltinParamDescriptor {
        name: "s2",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Char row vector, string scalar, or double row vector.",
    },
];

const FINDSTR_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "k",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Double row of 1-based start indices of the shorter argument in the longer, or a 0x0 empty double.",
}];

const FINDSTR_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "k = findstr(s1, s2)",
    inputs: &FINDSTR_INPUTS,
    outputs: &FINDSTR_OUTPUT,
}];

const MULTI_ROW_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.FINDSTR.MULTI_ROW_INPUT",
    identifier: Some("RunMat:findstr:MultiRowInput"),
    when: "An argument has more than one row (char or numeric).",
    message: "findstr: inputs must be row vectors",
};

const INPUTS_MUST_BE_CHAR_ARRAYS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.FINDSTR.INPUTS_MUST_BE_CHAR_ARRAYS",
    identifier: Some("RunMat:findstr:InputsMustBeCharArrays"),
    when: "An argument is a cell, struct, multi-element string array or other non-text value.",
    message: "findstr: inputs must be character arrays",
};

const NOT_ENOUGH_INPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.FINDSTR.NOT_ENOUGH_INPUTS",
    identifier: Some("RunMat:findstr:NotEnoughInputs"),
    when: "Fewer than two arguments are given.",
    message: "findstr: not enough input arguments",
};

const TOO_MANY_INPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.FINDSTR.TOO_MANY_INPUTS",
    identifier: Some("RunMat:findstr:TooManyInputs"),
    when: "More than two arguments are given.",
    message: "findstr: too many input arguments",
};

const UNSUPPORTED_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.FINDSTR.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:findstr:UnsupportedOption"),
    when: "Unobserved argument classes: mixed text/numeric, logical, integer, single or complex.",
    message: "findstr: argument class not yet specified",
};

const INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.FINDSTR.INTERNAL",
    identifier: Some("RunMat:findstr:Internal"),
    when: "The runtime cannot assemble the result.",
    message: "findstr: internal runtime failure",
};

const FINDSTR_ERRORS: [BuiltinErrorDescriptor; 6] = [
    MULTI_ROW_INPUT,
    INPUTS_MUST_BE_CHAR_ARRAYS,
    NOT_ENOUGH_INPUTS,
    TOO_MANY_INPUTS,
    UNSUPPORTED_OPTION,
    INTERNAL,
];

pub const FINDSTR_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &FINDSTR_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &FINDSTR_ERRORS,
};

fn findstr_error(
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

/// One argument as a sequence of comparable elements.
enum Sequence {
    Text(Vec<char>),
    Numbers(Vec<f64>),
}

impl Sequence {
    fn len(&self) -> usize {
        match self {
            Sequence::Text(chars) => chars.len(),
            Sequence::Numbers(numbers) => numbers.len(),
        }
    }
}

#[runtime_builtin(
    name = "findstr",
    category = "strings/search",
    summary = "Return the start indices of the shorter of two texts within the longer.",
    keywords = "findstr,substring,index,positions,string search",
    accel = "cpu",
    descriptor(crate::builtins::strings::search::findstr::FINDSTR_DESCRIPTOR),
    builtin_path = "crate::builtins::strings::search::findstr"
)]
async fn findstr_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    if args.len() < 2 {
        // findstr.error-one-arg
        return Err(findstr_error(
            &NOT_ENOUGH_INPUTS,
            "two input arguments are required",
        ));
    }
    if args.len() > 2 {
        return Err(findstr_error(
            &TOO_MANY_INPUTS,
            "only findstr(s1, s2) is supported",
        ));
    }
    let mut sequences = Vec::with_capacity(2);
    for (value, label) in args.iter().zip(["s1", "s2"]) {
        let host = gather_if_needed_async(value)
            .await
            .map_err(|err| map_control_flow_with_builtin(err, NAME))?;
        sequences.push(sequence_of(&host, label)?);
    }
    let second = sequences.pop().expect("two sequences");
    let first = sequences.pop().expect("two sequences");

    // findstr.order-independent: search the shorter within the longer.
    let (haystack, needle) = if second.len() > first.len() {
        (second, first)
    } else {
        (first, second)
    };
    let starts = match (&haystack, &needle) {
        (Sequence::Text(h), Sequence::Text(n)) => start_indices(h, n),
        (Sequence::Numbers(h), Sequence::Numbers(n)) => start_indices(h, n),
        _ => {
            return Err(findstr_error(
                &UNSUPPORTED_OPTION,
                "mixing text and numeric arguments is not yet specified",
            ))
        }
    };
    // findstr.no-match: no occurrence (or an empty argument) gives 0x0.
    let shape = if starts.is_empty() {
        vec![0, 0]
    } else {
        vec![1, starts.len()]
    };
    Tensor::new(starts, shape)
        .map(Value::Tensor)
        .map_err(|err| findstr_error(&INTERNAL, err))
}

fn sequence_of(value: &Value, label: &str) -> BuiltinResult<Sequence> {
    let multi_row = || findstr_error(&MULTI_ROW_INPUT, format!("{label} must be a row vector"));
    let unsupported = |what: &str| {
        findstr_error(
            &UNSUPPORTED_OPTION,
            format!("{label}: {what} arguments are not yet specified"),
        )
    };
    match value {
        // findstr.error-column-char
        Value::CharArray(chars) if chars.rows > 1 => Err(multi_row()),
        Value::CharArray(chars) => Ok(Sequence::Text(chars.data.clone())),
        // findstr.string-scalar
        Value::String(text) => Ok(Sequence::Text(text.chars().collect())),
        Value::StringArray(array) if array.data.len() == 1 => {
            Ok(Sequence::Text(array.data[0].chars().collect()))
        }
        // findstr.numeric (double row vectors)
        Value::Num(n) => Ok(Sequence::Numbers(vec![*n])),
        Value::Tensor(t) => {
            if t.integer_storage().is_some() {
                return Err(unsupported("integer"));
            }
            if t.numeric_dtype() != NumericDType::F64 {
                return Err(unsupported("single"));
            }
            let is_empty = t.is_empty();
            let rows = t.shape.first().copied().unwrap_or(1);
            let extra = t.shape.iter().skip(2).any(|&dim| dim != 1);
            if !is_empty && (rows != 1 || extra) {
                return Err(multi_row());
            }
            Ok(Sequence::Numbers(t.materialize_f64()))
        }
        Value::Int(_) => Err(unsupported("integer")),
        Value::Bool(_) | Value::LogicalArray(_) => Err(unsupported("logical")),
        Value::Complex(_, _) | Value::ComplexTensor(_) => Err(unsupported("complex")),
        // findstr.error-cell; other classes follow it (documented choice).
        other => Err(findstr_error(
            &INPUTS_MUST_BE_CHAR_ARRAYS,
            format!(
                "{label} must be a character array, got {}",
                crate::builtins::introspection::class::class_name_for_value(other)
            ),
        )),
    }
}

/// Every 1-based start of `needle` in `haystack`, overlaps included
/// (findstr.start-indices, findstr.overlapping). An empty needle has no
/// matches (findstr.no-match).
fn start_indices<T: PartialEq>(haystack: &[T], needle: &[T]) -> Vec<f64> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return Vec::new();
    }
    haystack
        .windows(needle.len())
        .enumerate()
        .filter(|(_, window)| *window == needle)
        .map(|(start, _)| (start + 1) as f64)
        .collect()
}

#[cfg(test)]
mod findstr_tests {
    use crate::BuiltinResult;
    use futures::executor::block_on;
    use runmat_value::{
        CellArray, CharArray, IntValue, LogicalArray, NumericDType, StringArray, Tensor, Value,
    };

    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("findstr", args))
    }

    fn chars(s: &str) -> Value {
        Value::CharArray(CharArray::new_row(s))
    }

    fn empty_char() -> Value {
        Value::CharArray(CharArray::new(Vec::new(), 0, 0).unwrap())
    }

    fn row(values: &[f64]) -> Value {
        Value::Tensor(Tensor::new(values.to_vec(), vec![1, values.len()]).unwrap())
    }

    fn identifier(result: BuiltinResult<Value>) -> String {
        match result {
            Ok(value) => panic!("expected an error, got {value:?}"),
            Err(err) => err.identifier().unwrap_or("<none>").to_string(),
        }
    }

    /// (shape, data) of a double result; a scalar counts as 1x1.
    fn double(value: Value) -> (Vec<usize>, Vec<f64>) {
        match value {
            Value::Tensor(t) => {
                assert!(t.integer_storage().is_none(), "expected double storage");
                assert_eq!(
                    t.numeric_dtype(),
                    NumericDType::F64,
                    "expected class double"
                );
                (t.shape.clone(), t.materialize_f64())
            }
            Value::Num(n) => (vec![1, 1], vec![n]),
            other => panic!("expected a double result, got {other:?}"),
        }
    }

    fn assert_row(value: Value, expected: &[f64]) {
        let (shape, data) = double(value);
        assert_eq!(shape, vec![1, expected.len()]);
        assert_eq!(data, expected);
    }

    fn assert_empty_0x0(value: Value) {
        let (shape, data) = double(value);
        assert_eq!(shape, vec![0, 0]);
        assert!(data.is_empty());
    }

    // --- Observed (normative): one test per observed case --------------------

    /// case `occurrences` — findstr.signature-primary, findstr.output-class,
    /// findstr.start-indices
    #[test]
    fn observed_occurrences() {
        assert_row(
            call(&[chars("abcabc"), chars("bc")]).expect("findstr"),
            &[2.0, 5.0],
        );
    }

    /// case `swapped` — findstr.order-independent
    #[test]
    fn observed_swapped() {
        assert_row(call(&[chars("bc"), chars("abcabc")]).unwrap(), &[2.0, 5.0]);
    }

    /// case `overlapping` — findstr.overlapping
    #[test]
    fn observed_overlapping() {
        assert_row(
            call(&[chars("aaaa"), chars("aa")]).unwrap(),
            &[1.0, 2.0, 3.0],
        );
    }

    /// case `no-match` — findstr.no-match, findstr.output-class
    #[test]
    fn observed_no_match() {
        assert_empty_0x0(call(&[chars("abc"), chars("x")]).unwrap());
    }

    /// case `equal-length-match` — findstr.equal-length
    #[test]
    fn observed_equal_length_match() {
        assert_row(call(&[chars("ab"), chars("ab")]).unwrap(), &[1.0]);
    }

    /// case `equal-length-nomatch` — findstr.equal-length, findstr.no-match
    #[test]
    fn observed_equal_length_nomatch() {
        assert_empty_0x0(call(&[chars("ab"), chars("ba")]).unwrap());
    }

    /// case `empty-pattern` — findstr.no-match
    #[test]
    fn observed_empty_pattern() {
        assert_empty_0x0(call(&[chars("abc"), empty_char()]).unwrap());
    }

    /// case `both-empty` — findstr.no-match
    #[test]
    fn observed_both_empty() {
        assert_empty_0x0(call(&[empty_char(), empty_char()]).unwrap());
    }

    /// case `empty-first` — findstr.no-match
    #[test]
    fn observed_empty_first() {
        assert_empty_0x0(call(&[empty_char(), chars("abc")]).unwrap());
    }

    /// case `warn-call` — findstr.no-warning
    #[test]
    fn observed_no_warning() {
        crate::warning_store::reset();
        let k = call(&[chars("abcabc"), chars("bc")]).unwrap();
        let warnings = crate::warning_store::take_all();
        assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");
        assert_row(k, &[2.0, 5.0]);
    }

    /// case `case` — findstr.case-sensitive, findstr.no-match
    #[test]
    fn observed_case_sensitive() {
        assert_empty_0x0(call(&[chars("ABC"), chars("b")]).unwrap());
    }

    /// case `single-char` — findstr.start-indices
    #[test]
    fn observed_single_char() {
        assert_row(
            call(&[chars("banana"), chars("a")]).unwrap(),
            &[2.0, 4.0, 6.0],
        );
    }

    /// case `string-scalar` — findstr.string-scalar
    #[test]
    fn observed_string_scalar() {
        assert_row(
            call(&[Value::String("abcabc".into()), Value::String("bc".into())]).unwrap(),
            &[2.0, 5.0],
        );
    }

    /// case `numeric` — findstr.numeric, findstr.output-class
    #[test]
    fn observed_numeric() {
        assert_row(
            call(&[row(&[1.0, 2.0, 3.0, 1.0, 2.0]), row(&[1.0, 2.0])]).unwrap(),
            &[1.0, 4.0],
        );
    }

    /// case `column-char` — findstr.error-column-char (RunMat namespace)
    #[test]
    fn observed_multi_row_char_errors() {
        let column = Value::CharArray(CharArray::new(vec!['a', 'b', 'a'], 3, 1).unwrap());
        assert_eq!(
            identifier(call(&[column, chars("a")])),
            "RunMat:findstr:MultiRowInput"
        );
    }

    /// case `err-cell` — findstr.error-cell (RunMat namespace)
    #[test]
    fn observed_cell_errors() {
        let cell = Value::Cell(CellArray::new(vec![chars("abc")], 1, 1).unwrap());
        assert_eq!(
            identifier(call(&[cell, chars("b")])),
            "RunMat:findstr:InputsMustBeCharArrays"
        );
    }

    /// case `err-one-arg` — findstr.error-one-arg (RunMat namespace)
    #[test]
    fn observed_one_argument_errors() {
        assert_eq!(
            identifier(call(&[chars("abc")])),
            "RunMat:findstr:NotEnoughInputs"
        );
    }

    // --- Documented choices -------------------------------------------------

    /// String scalars and char rows mix; a one-element string array counts as
    /// a string scalar; multi-row char errors in either position.
    #[test]
    fn documented_choice_text_forms() {
        assert_row(
            call(&[Value::String("abcabc".into()), chars("bc")]).unwrap(),
            &[2.0, 5.0],
        );
        let one = Value::StringArray(StringArray::new(vec!["bc".into()], vec![1, 1]).unwrap());
        assert_row(call(&[chars("abcabc"), one]).unwrap(), &[2.0, 5.0]);
        let block = Value::CharArray(CharArray::new("abcd".chars().collect(), 2, 2).unwrap());
        assert_eq!(
            identifier(call(&[chars("abcabc"), block])),
            "RunMat:findstr:MultiRowInput"
        );
    }

    /// Numeric forms: scalars, empty [], swapped order, multi-row numeric.
    #[test]
    fn documented_choice_numeric_forms() {
        assert_row(
            call(&[row(&[1.0, 2.0]), row(&[1.0, 2.0, 3.0, 1.0, 2.0])]).unwrap(),
            &[1.0, 4.0],
        );
        assert_row(
            call(&[row(&[5.0, 1.0, 5.0]), Value::Num(5.0)]).unwrap(),
            &[1.0, 3.0],
        );
        let empty = Value::Tensor(Tensor::new(Vec::new(), vec![0, 0]).unwrap());
        assert_empty_0x0(call(&[row(&[1.0, 2.0]), empty]).unwrap());
        // NaN never equals NaN.
        assert_empty_0x0(call(&[row(&[f64::NAN, 1.0]), Value::Num(f64::NAN)]).unwrap());
        let matrix = Value::Tensor(Tensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap());
        assert_eq!(
            identifier(call(&[matrix, Value::Num(1.0)])),
            "RunMat:findstr:MultiRowInput"
        );
    }

    /// Other non-text classes are rejected like cells; extra arguments error.
    #[test]
    fn documented_choice_other_inputs_and_arity() {
        let bad = [
            Value::Struct(runmat_value::StructValue::new()),
            Value::StringArray(StringArray::new(vec!["a".into(), "b".into()], vec![1, 2]).unwrap()),
        ];
        for value in bad {
            assert_eq!(
                identifier(call(&[value.clone(), chars("a")])),
                "RunMat:findstr:InputsMustBeCharArrays",
                "{value:?}"
            );
        }
        assert_eq!(
            identifier(call(&[chars("abcabc"), chars("bc"), chars("x")])),
            "RunMat:findstr:TooManyInputs"
        );
    }

    // --- Unresolved / unobserved input classes --------------------------------

    /// Mixed text/numeric and logical, integer, single or complex numeric
    /// arguments are not observed.
    #[test]
    fn unresolved_unobserved_classes_are_unsupported() {
        let cases = [
            vec![chars("abc"), row(&[98.0])],
            vec![
                Value::LogicalArray(LogicalArray::new(vec![1, 0], vec![1, 2]).unwrap()),
                Value::Bool(true),
            ],
            vec![row(&[1.0, 2.0]), Value::Int(IntValue::I32(1))],
            vec![
                Value::Tensor(Tensor::from_f32(vec![1.0, 2.0], vec![1, 2]).unwrap()),
                row(&[1.0]),
            ],
            vec![row(&[1.0, 2.0]), Value::Complex(1.0, 1.0)],
        ];
        for args in cases {
            assert_eq!(
                identifier(call(&args)),
                "RunMat:findstr:UnsupportedOption",
                "{args:?}"
            );
        }
    }
}
