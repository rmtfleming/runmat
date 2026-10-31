//! Clean-room `strread` (read formatted fields from a char row vector).
//!
//! Clean-room feature `specs/057-text-parsing/`. Normative behaviour comes
//! only from the approved export `strread` 0.2.0 (see that feature's
//! `provenance.md`). Observed `MATLAB:*` identifiers are raised as
//! `RunMat:*` (D-057-1); the identifiers name `dataread`, which is the
//! open spec-side question `strread.q-error-ids`.
//!
//! Only the observed conversions (`%s`, `%d`, `%f`) and the observed option
//! (`'delimiter'`, one character) are implemented; everything else raises
//! `RunMat:strread:UnsupportedOption`. The `textread` parser of spec 052
//! has no public items and raises `textread` identifiers, so this module has
//! its own small tokenizer (D-057-2).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{CellArray, CharArray, Tensor, Value};

use crate::builtins::common::map_control_flow_with_builtin;
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const NAME: &str = "strread";

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

const IN_STR: BuiltinParamDescriptor =
    param("str", BuiltinParamArity::Required, "char row vector.");
const IN_FORMAT: BuiltinParamDescriptor = param(
    "format",
    BuiltinParamArity::Required,
    "Format of %s, %d and %f conversions.",
);
const IN_OPTIONS: BuiltinParamDescriptor = param(
    "'delimiter', d",
    BuiltinParamArity::Variadic,
    "Single-character field delimiter.",
);
const OUT_A: BuiltinParamDescriptor = param(
    "A",
    BuiltinParamArity::Required,
    "Row of doubles (no format), or one column per conversion.",
);
const OUT_REST: BuiltinParamDescriptor = param(
    "B, ...",
    BuiltinParamArity::Variadic,
    "Further columns, one per conversion.",
);

const INPUTS_STR: [BuiltinParamDescriptor; 1] = [IN_STR];
const INPUTS_FORMAT: [BuiltinParamDescriptor; 2] = [IN_STR, IN_FORMAT];
const INPUTS_OPTIONS: [BuiltinParamDescriptor; 3] = [IN_STR, IN_FORMAT, IN_OPTIONS];
const OUTPUTS_A: [BuiltinParamDescriptor; 1] = [OUT_A];
const OUTPUTS_MANY: [BuiltinParamDescriptor; 2] = [OUT_A, OUT_REST];

const SIGNATURES: [BuiltinSignatureDescriptor; 3] = [
    BuiltinSignatureDescriptor {
        label: "A = strread(str)",
        inputs: &INPUTS_STR,
        outputs: &OUTPUTS_A,
    },
    BuiltinSignatureDescriptor {
        label: "A = strread(str, format)",
        inputs: &INPUTS_FORMAT,
        outputs: &OUTPUTS_A,
    },
    BuiltinSignatureDescriptor {
        label: "[A, B, ...] = strread(str, format, 'delimiter', d)",
        inputs: &INPUTS_OPTIONS,
        outputs: &OUTPUTS_MANY,
    },
];

const SECOND_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.STRREAD.SECOND_INPUT_FILENAME_OR_STRING",
    identifier: Some("RunMat:textio:dataread:SecondInputFilenameOrString"),
    when: "The first argument is numeric.",
    message: "strread: the first argument must be text",
};

const TROUBLE_READING: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.STRREAD.TROUBLE_READING",
    identifier: Some("RunMat:textio:dataread:TroubleReading"),
    when: "A field cannot be read with the given numeric conversion.",
    message: "strread: a field cannot be read as a number",
};

const INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.STRREAD.INVALID_INPUT",
    identifier: Some("RunMat:strread:InvalidInput"),
    when: "No input is given, or an option name has no value.",
    message: "strread: expected strread(str, format, 'delimiter', d)",
};

const UNSUPPORTED_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.STRREAD.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:strread:UnsupportedOption"),
    when: "Unobserved forms: other conversions, literals, repeat counts, other options, multi-character delimiters, empty fields, non-char text.",
    message: "strread: this form is not yet specified",
};

const TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.STRREAD.TOO_MANY_OUTPUTS",
    identifier: Some("RunMat:strread:TooManyOutputs"),
    when: "More outputs were requested than the format has conversions.",
    message: "strread: too many output arguments",
};

const INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.STRREAD.INTERNAL",
    identifier: Some("RunMat:strread:Internal"),
    when: "The runtime cannot assemble the result.",
    message: "strread: internal runtime failure",
};

const ERRORS: [BuiltinErrorDescriptor; 6] = [
    SECOND_INPUT,
    TROUBLE_READING,
    INVALID_INPUT,
    UNSUPPORTED_OPTION,
    TOO_MANY_OUTPUTS,
    INTERNAL,
];

pub const STRREAD_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ERRORS,
};

fn strread_error(
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
    strread_error(&UNSUPPORTED_OPTION, detail)
}

// ---------------------------------------------------------------------------
// Builtin
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "strread",
    category = "io/filetext",
    summary = "Read formatted fields from text into one output per conversion.",
    keywords = "strread,parse,text,format,fields,delimiter",
    accel = "cpu",
    descriptor(crate::builtins::io::filetext::strread::STRREAD_DESCRIPTOR),
    builtin_path = "crate::builtins::io::filetext::strread"
)]
pub(crate) async fn strread_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    let mut host = Vec::with_capacity(args.len());
    for value in &args {
        host.push(
            gather_if_needed_async(value)
                .await
                .map_err(|err| map_control_flow_with_builtin(err, NAME))?,
        );
    }
    let Some(first) = host.first() else {
        return Err(strread_error(&INVALID_INPUT, "expected strread(str, ...)"));
    };
    let text = input_text(first)?;
    let fields = match host.get(1) {
        None => None,
        Some(format) => Some(parse_format(&char_row(format, "the format")?)?),
    };
    let delimiter = parse_options(host.get(2..).unwrap_or(&[]))?;

    // FR-057-29: requested outputs against available conversions.
    let available = fields.as_ref().map_or(1, Vec::len);
    let requested = crate::output_count::current_output_count();
    if let Some(count) = requested {
        if count > available {
            return Err(strread_error(
                &TOO_MANY_OUTPUTS,
                format!("{count} outputs requested, the format has {available} conversions"),
            ));
        }
    }

    let tokens = tokens(&text, delimiter)?;
    let mut outputs = match &fields {
        // FR-057-01
        None => vec![default_row(&tokens)?],
        // FR-057-02..06
        Some(fields) => read_fields(&tokens, fields)?,
    };
    match requested {
        Some(0) => Ok(Value::OutputList(Vec::new())),
        Some(count) => {
            outputs.truncate(count);
            Ok(Value::OutputList(outputs))
        }
        None => Ok(outputs.swap_remove(0)),
    }
}

// ---------------------------------------------------------------------------
// Arguments
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Field {
    /// `%s`: column cell of char.
    Str,
    /// `%d`: double column from integer fields.
    Int,
    /// `%f`: double column.
    Float,
}

/// A char row vector (or `''`) as a Rust string.
fn char_row(value: &Value, what: &str) -> BuiltinResult<String> {
    match value {
        Value::CharArray(chars) if chars.rows <= 1 => Ok(chars.data.iter().collect()),
        _ => Err(unsupported(format!(
            "{what} must be a char row vector (other text forms are not yet specified)"
        ))),
    }
}

/// FR-057-07 (strread.error-numeric), FR-057-30.
fn input_text(value: &Value) -> BuiltinResult<String> {
    match value {
        Value::Num(_)
        | Value::Int(_)
        | Value::Tensor(_)
        | Value::Complex(_, _)
        | Value::ComplexTensor(_) => Err(strread_error(
            &SECOND_INPUT,
            "the first argument must be text, not numeric",
        )),
        other => char_row(other, "str"),
    }
}

/// FR-057-26: `%s`, `%d` and `%f`, optionally separated by whitespace.
fn parse_format(format: &str) -> BuiltinResult<Vec<Field>> {
    let chars: Vec<char> = format.chars().collect();
    let mut fields = Vec::new();
    let mut idx = 0;
    while idx < chars.len() {
        let ch = chars[idx];
        if ch.is_whitespace() {
            idx += 1;
            continue;
        }
        if ch != '%' {
            return Err(unsupported(format!(
                "format '{format}': literal text '{ch}' is not yet specified"
            )));
        }
        let field = match chars.get(idx + 1) {
            Some('s') => Field::Str,
            Some('d') => Field::Int,
            Some('f') => Field::Float,
            _ => {
                let rest: String = chars[idx..].iter().take(4).collect();
                return Err(unsupported(format!(
                    "format '{format}': the conversion starting '{rest}' is not yet specified"
                )));
            }
        };
        fields.push(field);
        idx += 2;
    }
    if fields.is_empty() {
        return Err(unsupported(format!("format '{format}' has no conversions")));
    }
    Ok(fields)
}

/// FR-057-05, FR-057-26: only `'delimiter'` with one character.
fn parse_options(rest: &[Value]) -> BuiltinResult<Option<char>> {
    let mut delimiter = None;
    let mut idx = 0;
    while idx < rest.len() {
        let name = match &rest[idx] {
            Value::CharArray(_) => char_row(&rest[idx], "an option name")?,
            Value::Num(_) | Value::Int(_) | Value::Tensor(_) => {
                return Err(unsupported("a repeat count argument is not yet specified"))
            }
            _ => return Err(unsupported("option names must be char row vectors")),
        };
        if !name.eq_ignore_ascii_case("delimiter") {
            return Err(unsupported(format!("option '{name}' is not yet specified")));
        }
        let Some(value) = rest.get(idx + 1) else {
            return Err(strread_error(
                &INVALID_INPUT,
                format!("option '{name}' has no value"),
            ));
        };
        let text = char_row(value, "the delimiter")?;
        let mut chars = text.chars();
        match (chars.next(), chars.next()) {
            (Some(ch), None) if !ch.is_whitespace() => delimiter = Some(ch),
            _ => {
                return Err(unsupported(
                    "only a single non-whitespace delimiter character is specified",
                ))
            }
        }
        idx += 2;
    }
    Ok(delimiter)
}

// ---------------------------------------------------------------------------
// Reading
// ---------------------------------------------------------------------------

/// FR-057-05: whitespace-separated fields, or fields split only at the
/// delimiter and trimmed of surrounding whitespace.
fn tokens(text: &str, delimiter: Option<char>) -> BuiltinResult<Vec<&str>> {
    match delimiter {
        None => Ok(text.split_whitespace().collect()),
        Some(delimiter) => {
            if text.trim().is_empty() {
                return Ok(Vec::new());
            }
            text.split(delimiter)
                .map(|field| {
                    let field = field.trim();
                    if field.is_empty() {
                        Err(unsupported(
                            "empty fields between delimiters are not yet specified",
                        ))
                    } else {
                        Ok(field)
                    }
                })
                .collect()
        }
    }
}

fn trouble(token: &str, conversion: &str) -> RuntimeError {
    strread_error(
        &TROUBLE_READING,
        format!("'{token}' cannot be read with {conversion}"),
    )
}

fn parse_float(token: &str, conversion: &str) -> BuiltinResult<f64> {
    token.parse::<f64>().map_err(|_| trouble(token, conversion))
}

/// FR-057-03, FR-057-08: an optionally signed digit string; other numbers
/// are not yet specified for `%d`.
fn parse_int(token: &str) -> BuiltinResult<f64> {
    let digits = token.strip_prefix(['+', '-']).unwrap_or(token);
    if !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()) {
        return parse_float(token, "%d");
    }
    match token.parse::<f64>() {
        Ok(_) => Err(unsupported(format!(
            "'{token}' is not an integer; %d of such fields is not yet specified"
        ))),
        Err(_) => Err(trouble(token, "%d")),
    }
}

/// FR-057-01, FR-057-25: no format reads a row of doubles.
fn default_row(tokens: &[&str]) -> BuiltinResult<Value> {
    let values = tokens
        .iter()
        .map(|token| parse_float(token, "the default format"))
        .collect::<BuiltinResult<Vec<f64>>>()?;
    let cols = values.len();
    Tensor::new(values, vec![1, cols])
        .map(Value::Tensor)
        .map_err(|err| strread_error(&INTERNAL, err))
}

/// FR-057-02..04, FR-057-06: fields assigned to conversions in cyclic
/// order, one column per conversion.
fn read_fields(tokens: &[&str], fields: &[Field]) -> BuiltinResult<Vec<Value>> {
    let mut outputs = Vec::with_capacity(fields.len());
    for (position, field) in fields.iter().enumerate() {
        let column: Vec<&str> = tokens
            .iter()
            .skip(position)
            .step_by(fields.len())
            .copied()
            .collect();
        let rows = column.len();
        let value = match field {
            Field::Str => Value::Cell(
                CellArray::new(
                    column
                        .into_iter()
                        .map(|token| Value::CharArray(CharArray::new_row(token)))
                        .collect(),
                    rows,
                    1,
                )
                .map_err(|err| strread_error(&INTERNAL, err))?,
            ),
            Field::Int | Field::Float => {
                let values = column
                    .into_iter()
                    .map(|token| {
                        if *field == Field::Int {
                            parse_int(token)
                        } else {
                            parse_float(token, "%f")
                        }
                    })
                    .collect::<BuiltinResult<Vec<f64>>>()?;
                Value::Tensor(
                    Tensor::new(values, vec![rows, 1])
                        .map_err(|err| strread_error(&INTERNAL, err))?,
                )
            }
        };
        outputs.push(value);
    }
    Ok(outputs)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod strread_tests {
    use super::strread_builtin;
    use crate::BuiltinResult;
    use futures::executor::block_on;
    use runmat_value::{CharArray, Tensor, Value};

    fn chr(text: &str) -> Value {
        Value::CharArray(CharArray::new_row(text))
    }

    fn empty_chr() -> Value {
        Value::CharArray(CharArray::new(Vec::new(), 0, 0).unwrap())
    }

    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("strread", args))
    }

    /// Call with `count` requested outputs.
    fn call_n(args: Vec<Value>, count: usize) -> BuiltinResult<Vec<Value>> {
        let _guard = crate::output_count::push_output_count(Some(count));
        match block_on(strread_builtin(args))? {
            Value::OutputList(outs) => Ok(outs),
            other => panic!("expected an output list, got {other:?}"),
        }
    }

    fn doubles(value: &Value) -> (Vec<usize>, Vec<f64>) {
        match value {
            Value::Tensor(t) => (t.shape.clone(), t.materialize_f64()),
            Value::Num(v) => (vec![1, 1], vec![*v]),
            other => panic!("expected a double array, got {other:?}"),
        }
    }

    fn cellstr(value: &Value) -> (Vec<usize>, Vec<String>) {
        match value {
            Value::Cell(cell) => (
                vec![cell.rows, cell.cols],
                cell.data
                    .iter()
                    .map(|item| match item {
                        Value::CharArray(chars) => {
                            assert!(chars.rows == 1, "expected a char row");
                            chars.data.iter().collect()
                        }
                        other => panic!("expected char, got {other:?}"),
                    })
                    .collect(),
            ),
            other => panic!("expected a cell array, got {other:?}"),
        }
    }

    fn identifier(result: BuiltinResult<Value>) -> String {
        match result {
            Ok(value) => panic!("expected an error, got {value:?}"),
            Err(err) => err.identifier().unwrap_or("<none>").to_string(),
        }
    }

    const UNSUPPORTED: &str = "RunMat:strread:UnsupportedOption";

    // --- Observed (normative): one test per observed case --------------------

    /// case `default` — strread.signature-primary, strread.default
    #[test]
    fn observed_default() {
        let out = call(&[chr("1 2 3")]).unwrap();
        assert_eq!(doubles(&out), (vec![1, 3], vec![1.0, 2.0, 3.0]));
    }

    /// case `s` — strread.s
    #[test]
    fn observed_s() {
        let out = call(&[chr("ab cd ef"), chr("%s")]).unwrap();
        assert_eq!(
            cellstr(&out),
            (vec![3, 1], vec!["ab".into(), "cd".into(), "ef".into()])
        );
    }

    /// case `d` — strread.numeric
    #[test]
    fn observed_d() {
        let out = call(&[chr("4 5 6"), chr("%d")]).unwrap();
        assert_eq!(doubles(&out), (vec![3, 1], vec![4.0, 5.0, 6.0]));
    }

    /// case `f` — strread.numeric
    #[test]
    fn observed_f() {
        let out = call(&[chr("1.5 2.25"), chr("%f")]).unwrap();
        assert_eq!(doubles(&out), (vec![2, 1], vec![1.5, 2.25]));
    }

    /// case `two-out` — strread.cyclic
    #[test]
    fn observed_two_out() {
        let outs = call_n(
            vec![chr("x!y!z!w"), chr("%s%s"), chr("delimiter"), chr("!")],
            2,
        )
        .unwrap();
        assert_eq!(
            cellstr(&outs[0]),
            (vec![2, 1], vec!["x".into(), "z".into()])
        );
        assert_eq!(
            cellstr(&outs[1]),
            (vec![2, 1], vec!["y".into(), "w".into()])
        );
    }

    /// case `two-out-mixed` — strread.cyclic
    #[test]
    fn observed_two_out_mixed() {
        let outs = call_n(vec![chr("p 1 q 2"), chr("%s%d")], 2).unwrap();
        assert_eq!(
            cellstr(&outs[0]),
            (vec![2, 1], vec!["p".into(), "q".into()])
        );
        assert_eq!(doubles(&outs[1]), (vec![2, 1], vec![1.0, 2.0]));
    }

    /// case `two-out-uneven` — strread.cyclic
    #[test]
    fn observed_two_out_uneven() {
        let outs = call_n(vec![chr("1 2 3"), chr("%d%d")], 2).unwrap();
        assert_eq!(doubles(&outs[0]), (vec![2, 1], vec![1.0, 3.0]));
        assert_eq!(doubles(&outs[1]), (vec![1, 1], vec![2.0]));
    }

    /// case `delimiter-comma` — strread.delimiter
    #[test]
    fn observed_delimiter_comma() {
        let out = call(&[chr("a,b,c"), chr("%s"), chr("delimiter"), chr(",")]).unwrap();
        assert_eq!(
            cellstr(&out),
            (vec![3, 1], vec!["a".into(), "b".into(), "c".into()])
        );
    }

    /// case `delimiter-spaces` — strread.delimiter
    #[test]
    fn observed_delimiter_spaces() {
        let out = call(&[chr("a b, c d"), chr("%s"), chr("delimiter"), chr(",")]).unwrap();
        assert_eq!(
            cellstr(&out),
            (vec![2, 1], vec!["a b".into(), "c d".into()])
        );
    }

    /// case `num2str-form` — strread.s. The input is the text that RunMat's
    /// own num2str gives for [10 2 300].
    #[test]
    fn observed_num2str_form() {
        let row = Value::Tensor(Tensor::new(vec![10.0, 2.0, 300.0], vec![1, 3]).unwrap());
        let text = block_on(crate::call_builtin_async("num2str", &[row])).unwrap();
        let out = call(&[text, chr("%s")]).unwrap();
        assert_eq!(
            cellstr(&out),
            (vec![3, 1], vec!["10".into(), "2".into(), "300".into()])
        );
    }

    /// case `empty` — strread.empty
    #[test]
    fn observed_empty() {
        let out = call(&[empty_chr(), chr("%s")]).unwrap();
        assert_eq!(cellstr(&out), (vec![0, 1], vec![]));
    }

    /// case `orientation` — strread.numeric
    #[test]
    fn observed_orientation() {
        let out = call(&[chr("7 8"), chr("%d")]).unwrap();
        assert_eq!(doubles(&out), (vec![2, 1], vec![7.0, 8.0]));
    }

    /// case `warn` — strread.no-warning: the call succeeds with [1 2] and
    /// strread emits no warning (no warning path exists in this module).
    #[test]
    fn observed_warn() {
        let out = call(&[chr("1 2")]).unwrap();
        assert_eq!(doubles(&out), (vec![1, 2], vec![1.0, 2.0]));
    }

    /// case `err-numeric` — strread.error-numeric
    #[test]
    fn observed_err_numeric() {
        assert_eq!(
            identifier(call(&[Value::Num(5.0)])),
            "RunMat:textio:dataread:SecondInputFilenameOrString"
        );
    }

    /// case `err-bad-format` — strread.error-bad-format
    #[test]
    fn observed_err_bad_format() {
        assert_eq!(
            identifier(call(&[chr("a b"), chr("%d")])),
            "RunMat:textio:dataread:TroubleReading"
        );
    }

    // --- Documented choices --------------------------------------------------

    /// FR-057-25: empty text with no format or a numeric conversion.
    #[test]
    fn documented_choice_empty_numeric() {
        assert_eq!(
            doubles(&call(&[empty_chr()]).unwrap()),
            (vec![1, 0], vec![])
        );
        assert_eq!(
            doubles(&call(&[empty_chr(), chr("%f")]).unwrap()),
            (vec![0, 1], vec![])
        );
        // the no-format form reads like %f
        assert_eq!(
            identifier(call(&[chr("1 x")])),
            "RunMat:textio:dataread:TroubleReading"
        );
    }

    /// FR-057-26: whitespace in the format, %f with integers, %d with a sign.
    #[test]
    fn documented_choice_format_details() {
        let outs = call_n(vec![chr("a 1.5 b -2"), chr("%s %f")], 2).unwrap();
        assert_eq!(doubles(&outs[1]), (vec![2, 1], vec![1.5, -2.0]));
        assert_eq!(
            doubles(&call(&[chr("-3 +4"), chr("%d")]).unwrap()),
            (vec![2, 1], vec![-3.0, 4.0])
        );
        // fewer outputs than conversions: the first ones are returned
        let outs = call_n(vec![chr("p 1"), chr("%s%d")], 1).unwrap();
        assert_eq!(outs.len(), 1);
    }

    /// FR-057-30: unobserved forms.
    #[test]
    fn documented_choice_unsupported() {
        let cases: Vec<Vec<Value>> = vec![
            vec![chr("1 2"), chr("%u")],
            vec![chr("1 2"), chr("%*s")],
            vec![chr("1 2"), chr("x%d")],
            vec![chr("1 2"), chr("%5d")],
            vec![chr("1 2"), chr("%d"), Value::Num(1.0)],
            vec![chr("1 2"), chr("%d"), chr("headerlines"), Value::Num(1.0)],
            vec![chr("1,,2"), chr("%d"), chr("delimiter"), chr(",")],
            vec![chr("1;2"), chr("%d"), chr("delimiter"), chr(",;")],
            vec![chr("1.5"), chr("%d")],
            vec![Value::String("1 2".into())],
            vec![Value::Bool(true)],
            vec![chr("1 2"), Value::String("%d".into())],
        ];
        for args in cases {
            assert_eq!(identifier(call(&args)), UNSUPPORTED, "args {args:?}");
        }
    }

    /// FR-057-29: arity and output count.
    #[test]
    fn documented_choice_arity_and_outputs() {
        assert_eq!(identifier(call(&[])), "RunMat:strread:InvalidInput");
        assert_eq!(
            identifier(call(&[chr("a"), chr("%s"), chr("delimiter")])),
            "RunMat:strread:InvalidInput"
        );
        let err = call_n(vec![chr("1 2"), chr("%d")], 2).unwrap_err();
        assert_eq!(err.identifier(), Some("RunMat:strread:TooManyOutputs"));
        let err = call_n(vec![chr("1 2")], 2).unwrap_err();
        assert_eq!(err.identifier(), Some("RunMat:strread:TooManyOutputs"));
    }
}
