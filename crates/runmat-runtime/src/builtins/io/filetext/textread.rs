//! Clean-room `textread` (read formatted fields from a text file).
//!
//! Clean-room feature `specs/052-data-n2/`. Normative behaviour comes only
//! from the approved export `textread` 0.2.0 (see that feature's
//! `provenance.md`). Only the observed conversions (`%f`, `%d`, `%s`, `%*s`)
//! and options (`headerlines`, `delimiter`) are supported; every other form
//! raises `RunMat:textread:UnsupportedOption` (U-052-4, U-052-6).

use std::path::PathBuf;

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{CellArray, CharArray, Tensor, Value};

use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const NAME: &str = "textread";

// ---------------------------------------------------------------------------
// Descriptors
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

const P_FILENAME: BuiltinParamDescriptor = param(
    "filename",
    BuiltinParamArity::Required,
    "Path to a text file.",
);
const P_FORMAT: BuiltinParamDescriptor = param(
    "format",
    BuiltinParamArity::Required,
    "Format with %f, %d, %s and %*s fields.",
);
const P_OPTIONS: BuiltinParamDescriptor = param(
    "options",
    BuiltinParamArity::Variadic,
    "'headerlines' or 'delimiter' name-value pairs.",
);
const P_OUTPUTS: BuiltinParamDescriptor = param(
    "A, B, ...",
    BuiltinParamArity::Variadic,
    "One column per non-skipped field: double for %f/%d, cell of char for %s.",
);
const P_MATRIX: BuiltinParamDescriptor = param(
    "A",
    BuiltinParamArity::Required,
    "Double matrix with the file's row and column layout.",
);

const FORMAT_INPUTS: [BuiltinParamDescriptor; 2] = [P_FILENAME, P_FORMAT];
const OPTION_INPUTS: [BuiltinParamDescriptor; 3] = [P_FILENAME, P_FORMAT, P_OPTIONS];
const DEFAULT_INPUTS: [BuiltinParamDescriptor; 1] = [P_FILENAME];
const FIELD_OUTPUTS: [BuiltinParamDescriptor; 1] = [P_OUTPUTS];
const MATRIX_OUTPUTS: [BuiltinParamDescriptor; 1] = [P_MATRIX];

const TEXTREAD_SIGNATURES: [BuiltinSignatureDescriptor; 3] = [
    BuiltinSignatureDescriptor {
        label: "[A, B, ...] = textread(filename, format)",
        inputs: &FORMAT_INPUTS,
        outputs: &FIELD_OUTPUTS,
    },
    BuiltinSignatureDescriptor {
        label: "[A, B, ...] = textread(filename, format, Name, Value)",
        inputs: &OPTION_INPUTS,
        outputs: &FIELD_OUTPUTS,
    },
    BuiltinSignatureDescriptor {
        label: "A = textread(filename)",
        inputs: &DEFAULT_INPUTS,
        outputs: &MATRIX_OUTPUTS,
    },
];

const TEXTREAD_ERRORS: [BuiltinErrorDescriptor; 5] = [
    BuiltinErrorDescriptor {
        code: "RM.TEXTREAD.INVALID_INPUT",
        identifier: Some("RunMat:textread:InvalidInput"),
        when: "A required input is missing or an input or option value is malformed.",
        message: "textread: invalid input",
    },
    BuiltinErrorDescriptor {
        code: "RM.TEXTREAD.UNSUPPORTED_OPTION",
        identifier: Some("RunMat:textread:UnsupportedOption"),
        when: "The format, an option or the file data uses a form that is not yet specified for RunMat.",
        message: "textread: form not yet specified",
    },
    BuiltinErrorDescriptor {
        code: "RM.TEXTREAD.INTERNAL",
        identifier: Some("RunMat:textread:Internal"),
        when: "RunMat cannot build the result values.",
        message: "textread: internal error",
    },
    BuiltinErrorDescriptor {
        code: "RM.TEXTREAD.FILE_NOT_FOUND",
        identifier: Some("RunMat:textread:FileNotFound"),
        when: "The file does not exist or cannot be read.",
        message: "textread: file not found",
    },
    BuiltinErrorDescriptor {
        code: "RM.TEXTREAD.TOO_MANY_OUTPUTS",
        identifier: Some("RunMat:textread:TooManyOutputs"),
        when: "More outputs were requested than the format has non-skipped fields.",
        message: "textread: too many outputs",
    },
];

const INVALID_INPUT: usize = 0;
const UNSUPPORTED_OPTION: usize = 1;
const INTERNAL: usize = 2;
const FILE_NOT_FOUND: usize = 3;
const TOO_MANY_OUTPUTS: usize = 4;

pub const TEXTREAD_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &TEXTREAD_SIGNATURES,
    output_mode: BuiltinOutputMode::ByRequestedOutputCount,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &TEXTREAD_ERRORS,
};

fn textread_error(kind: usize, detail: impl Into<String>) -> RuntimeError {
    let mut builder = build_runtime_error(format!("{NAME}: {}", detail.into())).with_builtin(NAME);
    if let Some(identifier) = TEXTREAD_ERRORS[kind].identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

// ---------------------------------------------------------------------------
// Builtin
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "textread",
    category = "io/filetext",
    summary = "Read formatted fields from a text file into one output per field.",
    keywords = "textread,io,text file,format,read,columns",
    accel = "cpu",
    descriptor(crate::builtins::io::filetext::textread::TEXTREAD_DESCRIPTOR),
    builtin_path = "crate::builtins::io::filetext::textread"
)]
async fn textread_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    let mut gathered = Vec::with_capacity(args.len());
    for value in &args {
        gathered.push(gather_if_needed_async(value).await?);
    }
    let Some(path_value) = gathered.first() else {
        return Err(textread_error(
            INVALID_INPUT,
            "expected textread(filename, format, ...)",
        ));
    };
    let path = text_arg(path_value)
        .filter(|text| !text.is_empty())
        .ok_or_else(|| {
            textread_error(
                INVALID_INPUT,
                "the file name must be a nonempty char vector or string scalar",
            )
        })?;
    let fields = match gathered.get(1) {
        None => None,
        Some(value) => {
            let format = text_arg(value).ok_or_else(|| {
                textread_error(
                    INVALID_INPUT,
                    "the format must be a char vector or string scalar",
                )
            })?;
            Some(parse_format(&format)?)
        }
    };
    let options = parse_options(gathered.get(2..).unwrap_or(&[]))?;

    // U-052-8: check the requested outputs before touching the file.
    let available = fields.as_ref().map_or(1, |fields| {
        fields.iter().filter(|field| **field != Field::Skip).count()
    });
    let requested = crate::output_count::current_output_count();
    if let Some(count) = requested {
        if count > available {
            return Err(textread_error(
                TOO_MANY_OUTPUTS,
                format!("{count} outputs requested, the format has {available} fields"),
            ));
        }
    }

    // textread.error-missing, U-052-9.
    let bytes = runmat_filesystem::read_async(PathBuf::from(&path))
        .await
        .map_err(|err| textread_error(FILE_NOT_FOUND, format!("cannot read '{path}': {err}")))?;
    let text = String::from_utf8_lossy(&bytes);
    // textread.options: 'headerlines' skips whole lines.
    let lines: Vec<&str> = text.split('\n').skip(options.header_lines).collect();

    let mut outputs = match &fields {
        None => vec![default_matrix(&lines)?],
        Some(fields) => read_fields(&lines, fields, options.delimiter)?,
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
    /// `%f`: double.
    Float,
    /// `%d`: double from an integer field.
    Int,
    /// `%s`: cell of char.
    Str,
    /// `%*s`: read and discarded.
    Skip,
}

struct Options {
    header_lines: usize,
    delimiter: Option<char>,
}

fn text_arg(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::StringArray(array) if array.data.len() == 1 => Some(array.data[0].clone()),
        Value::CharArray(chars) if chars.rows <= 1 => Some(chars.data.iter().collect()),
        _ => None,
    }
}

fn scalar_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Num(n) => Some(*n),
        Value::Int(i) => Some(i.to_f64()),
        Value::Tensor(t) if t.shape.iter().product::<usize>() == 1 => Some(t.materialize_f64()[0]),
        _ => None,
    }
}

/// U-052-4: `%f`, `%d`, `%s` and `%*s` separated by optional whitespace.
fn parse_format(format: &str) -> BuiltinResult<Vec<Field>> {
    let unsupported =
        |detail: String| textread_error(UNSUPPORTED_OPTION, format!("format '{format}': {detail}"));
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
            return Err(unsupported(format!("literal text '{ch}' is not supported")));
        }
        let field = match (chars.get(idx + 1), chars.get(idx + 2)) {
            (Some('f'), _) => (Field::Float, 2),
            (Some('d'), _) => (Field::Int, 2),
            (Some('s'), _) => (Field::Str, 2),
            (Some('*'), Some('s')) => (Field::Skip, 3),
            _ => {
                let rest: String = chars[idx..].iter().take(4).collect();
                return Err(unsupported(format!(
                    "the conversion starting '{rest}' is not supported"
                )));
            }
        };
        fields.push(field.0);
        idx += field.1;
    }
    if !fields.iter().any(|field| *field != Field::Skip) {
        return Err(unsupported("the format has no output fields".to_string()));
    }
    Ok(fields)
}

/// U-052-6: `headerlines` and `delimiter` name-value pairs.
fn parse_options(rest: &[Value]) -> BuiltinResult<Options> {
    let mut options = Options {
        header_lines: 0,
        delimiter: None,
    };
    let mut idx = 0;
    while idx < rest.len() {
        let Some(name) = text_arg(&rest[idx]) else {
            if idx == 0 && scalar_f64(&rest[idx]).is_some() {
                return Err(textread_error(
                    UNSUPPORTED_OPTION,
                    "a repeat count argument is not supported",
                ));
            }
            return Err(textread_error(
                INVALID_INPUT,
                "option names must be char vectors or string scalars",
            ));
        };
        let lowered = name.to_ascii_lowercase();
        if lowered != "headerlines" && lowered != "delimiter" {
            return Err(textread_error(
                UNSUPPORTED_OPTION,
                format!("option '{name}' is not supported"),
            ));
        }
        let Some(value) = rest.get(idx + 1) else {
            return Err(textread_error(
                INVALID_INPUT,
                format!("option '{name}' has no value"),
            ));
        };
        if lowered == "headerlines" {
            let lines = scalar_f64(value)
                .filter(|n| n.is_finite() && *n >= 0.0 && n.fract() == 0.0)
                .ok_or_else(|| {
                    textread_error(INVALID_INPUT, "'headerlines' must be a nonnegative integer")
                })?;
            options.header_lines = lines as usize;
        } else {
            let text = text_arg(value).ok_or_else(|| {
                textread_error(
                    INVALID_INPUT,
                    "'delimiter' must be a char vector or string scalar",
                )
            })?;
            let mut chars = text.chars();
            match (chars.next(), chars.next()) {
                (Some(ch), None) => options.delimiter = Some(ch),
                _ => {
                    return Err(textread_error(
                        UNSUPPORTED_OPTION,
                        "only a single-character delimiter is supported",
                    ))
                }
            }
        }
        idx += 2;
    }
    Ok(options)
}

// ---------------------------------------------------------------------------
// Reading
// ---------------------------------------------------------------------------

/// Fields of the file in reading order (U-052-5): whitespace-separated, or
/// split on the delimiter and at line ends, trimmed, blank lines skipped.
fn tokens<'a>(lines: &[&'a str], delimiter: Option<char>) -> BuiltinResult<Vec<&'a str>> {
    let mut out = Vec::new();
    for line in lines {
        match delimiter {
            None => out.extend(line.split_whitespace()),
            Some(delimiter) => {
                if line.trim().is_empty() {
                    continue;
                }
                for field in line.split(delimiter) {
                    let field = field.trim();
                    if field.is_empty() {
                        return Err(textread_error(
                            UNSUPPORTED_OPTION,
                            "empty fields between delimiters are not supported",
                        ));
                    }
                    out.push(field);
                }
            }
        }
    }
    Ok(out)
}

fn parse_float(token: &str) -> BuiltinResult<f64> {
    token.parse::<f64>().map_err(|_| {
        textread_error(
            UNSUPPORTED_OPTION,
            format!("'{token}' is not a number for a %f field"),
        )
    })
}

fn parse_int(token: &str) -> BuiltinResult<f64> {
    let digits = token.strip_prefix(['+', '-']).unwrap_or(token);
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err(textread_error(
            UNSUPPORTED_OPTION,
            format!("'{token}' is not an integer for a %d field"),
        ));
    }
    parse_float(token)
}

/// textread.columns, textread.types, textread.skip, textread.empty.
fn read_fields(
    lines: &[&str],
    fields: &[Field],
    delimiter: Option<char>,
) -> BuiltinResult<Vec<Value>> {
    let tokens = tokens(lines, delimiter)?;
    if tokens.len() % fields.len() != 0 {
        return Err(textread_error(
            UNSUPPORTED_OPTION,
            "the last record has fewer fields than the format",
        ));
    }
    let records = tokens.len() / fields.len();
    let mut outputs = Vec::new();
    for (position, field) in fields.iter().enumerate() {
        let column = tokens.iter().skip(position).step_by(fields.len());
        let value = match field {
            Field::Skip => continue,
            Field::Float | Field::Int => {
                let parse = if *field == Field::Float {
                    parse_float
                } else {
                    parse_int
                };
                let values = column
                    .map(|token| parse(token))
                    .collect::<BuiltinResult<Vec<f64>>>()?;
                Value::Tensor(
                    Tensor::new(values, vec![records, 1])
                        .map_err(|err| textread_error(INTERNAL, err))?,
                )
            }
            Field::Str => Value::Cell(
                CellArray::new(
                    column
                        .map(|token| Value::CharArray(CharArray::new_row(token)))
                        .collect(),
                    records,
                    1,
                )
                .map_err(|err| textread_error(INTERNAL, err))?,
            ),
        };
        outputs.push(value);
    }
    Ok(outputs)
}

/// textread.default-format, U-052-7: one matrix row per non-blank line.
fn default_matrix(lines: &[&str]) -> BuiltinResult<Value> {
    let mut rows: Vec<Vec<f64>> = Vec::new();
    for line in lines {
        let row = line
            .split_whitespace()
            .map(|token| {
                token.parse::<f64>().map_err(|_| {
                    textread_error(
                        UNSUPPORTED_OPTION,
                        format!("'{token}' is not a number (no format given)"),
                    )
                })
            })
            .collect::<BuiltinResult<Vec<f64>>>()?;
        if row.is_empty() {
            continue;
        }
        if rows.first().is_some_and(|first| first.len() != row.len()) {
            return Err(textread_error(
                UNSUPPORTED_OPTION,
                "rows with different numbers of values are not supported",
            ));
        }
        rows.push(row);
    }
    let nrows = rows.len();
    let ncols = rows.first().map_or(0, Vec::len);
    let mut data = Vec::with_capacity(nrows * ncols);
    for col in 0..ncols {
        data.extend(rows.iter().map(|row| row[col]));
    }
    Tensor::new(data, vec![nrows, ncols])
        .map(Value::Tensor)
        .map_err(|err| textread_error(INTERNAL, err))
}

#[cfg(test)]
mod textread_tests {
    use super::*;
    use futures::executor::block_on;
    use runmat_value::StringArray;
    use std::path::Path;

    // ---- helpers ----------------------------------------------------------

    /// Write `content` to `name` inside the test's own temporary directory.
    fn write(dir: &Path, name: &str, content: &str) -> Value {
        let path = dir.join(name);
        std::fs::write(&path, content).expect("write temp file");
        chr(path.to_str().expect("utf-8 temp path"))
    }

    fn chr(text: &str) -> Value {
        Value::CharArray(CharArray::new_row(text))
    }

    fn num(value: f64) -> Value {
        Value::Num(value)
    }

    fn identifier(err: RuntimeError) -> String {
        err.identifier().unwrap_or("<none>").to_string()
    }

    /// Call `textread` directly with `nargout` outputs requested.
    fn read_n(args: Vec<Value>, nargout: usize) -> BuiltinResult<Vec<Value>> {
        let _guard = crate::output_count::push_output_count(Some(nargout));
        match block_on(textread_builtin(args))? {
            Value::OutputList(values) => Ok(values),
            other => panic!("expected an output list, got {other:?}"),
        }
    }

    fn err_n(args: Vec<Value>, nargout: usize) -> String {
        identifier(read_n(args, nargout).expect_err("expected an error"))
    }

    /// Through the dispatcher, one value (no output count).
    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("textread", args))
    }

    fn err_id(args: &[Value]) -> String {
        identifier(call(args).expect_err("expected an error"))
    }

    /// Shape and data of a double result.
    fn doubles(value: &Value) -> (Vec<usize>, Vec<f64>) {
        match value {
            Value::Tensor(t) => (t.shape.clone(), t.materialize_f64()),
            Value::Num(n) => (vec![1, 1], vec![*n]),
            other => panic!("expected double result, got {other:?}"),
        }
    }

    /// Shape and char contents of a cell result.
    fn cell_chars(value: &Value) -> (Vec<usize>, Vec<String>) {
        let Value::Cell(cell) = value else {
            panic!("expected cell result, got {value:?}");
        };
        let texts = cell
            .data
            .iter()
            .map(|entry| match entry {
                Value::CharArray(chars) => {
                    assert_eq!(chars.rows, 1, "char row vector");
                    chars.data.iter().collect()
                }
                other => panic!("expected char element, got {other:?}"),
            })
            .collect();
        (cell.shape.clone(), texts)
    }

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|s| s.to_string()).collect()
    }

    fn col(values: &[f64]) -> (Vec<usize>, Vec<f64>) {
        (vec![values.len(), 1], values.to_vec())
    }

    // ---- observed (normative) ---------------------------------------------

    /// textread.signature-primary, textread.columns, textread.types,
    /// textread.no-warning (case `numeric`).
    #[test]
    fn observed_textread_numeric_columns() {
        let dir = tempfile::tempdir().unwrap();
        let file = write(dir.path(), "numeric.txt", "1 2\n3 4\n");
        crate::warning_store::reset();
        let out = read_n(vec![file, chr("%f %f")], 2).unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(doubles(&out[0]), col(&[1.0, 3.0]));
        assert_eq!(doubles(&out[1]), col(&[2.0, 4.0]));
        assert!(crate::warning_store::take_all().is_empty(), "no warning");
    }

    /// textread.columns (case `single-column`).
    #[test]
    fn observed_textread_single_column() {
        let dir = tempfile::tempdir().unwrap();
        let file = write(dir.path(), "single.txt", "7\n8\n9\n");
        let out = read_n(vec![file.clone(), chr("%f")], 1).unwrap();
        assert_eq!(doubles(&out[0]), col(&[7.0, 8.0, 9.0]));
        // Through the dispatcher (no output count): the first output.
        assert_eq!(
            doubles(&call(&[file, chr("%f")]).unwrap()),
            col(&[7.0, 8.0, 9.0])
        );
    }

    /// textread.types (case `mixed`).
    #[test]
    fn observed_textread_mixed_types() {
        let dir = tempfile::tempdir().unwrap();
        let file = write(dir.path(), "mixed.txt", "x 1\ny 2\n");
        let out = read_n(vec![file, chr("%s %d")], 2).unwrap();
        assert_eq!(cell_chars(&out[0]), (vec![2, 1], strings(&["x", "y"])));
        assert_eq!(doubles(&out[1]), col(&[1.0, 2.0]));
    }

    /// textread.options (cases `headerlines`, `delimiter`).
    #[test]
    fn observed_textread_options() {
        let dir = tempfile::tempdir().unwrap();
        let file = write(dir.path(), "header.txt", "h\n5\n6\n");
        let out = read_n(vec![file, chr("%f"), chr("headerlines"), num(1.0)], 1).unwrap();
        assert_eq!(doubles(&out[0]), col(&[5.0, 6.0]));
        let file = write(dir.path(), "comma.txt", "1,2\n3,4\n");
        let out = read_n(vec![file, chr("%f %f"), chr("delimiter"), chr(",")], 2).unwrap();
        assert_eq!(doubles(&out[0]), col(&[1.0, 3.0]));
        assert_eq!(doubles(&out[1]), col(&[2.0, 4.0]));
    }

    /// textread.skip (case `skip-field`).
    #[test]
    fn observed_textread_skip_field() {
        let dir = tempfile::tempdir().unwrap();
        let file = write(dir.path(), "skip.txt", "x 1\ny 2\n");
        let out = read_n(vec![file, chr("%*s %d")], 1).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(doubles(&out[0]), col(&[1.0, 2.0]));
    }

    /// textread.default-format (case `default-format`).
    #[test]
    fn observed_textread_default_format() {
        let dir = tempfile::tempdir().unwrap();
        let file = write(dir.path(), "matrix.txt", "1 2\n3 4\n");
        let out = read_n(vec![file], 1).unwrap();
        // Column-major [1 2; 3 4].
        assert_eq!(doubles(&out[0]), (vec![2, 2], vec![1.0, 3.0, 2.0, 4.0]));
    }

    /// textread.empty (case `empty-file`).
    #[test]
    fn observed_textread_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = write(dir.path(), "empty.txt", "");
        let out = read_n(vec![file, chr("%f")], 1).unwrap();
        assert_eq!(doubles(&out[0]), (vec![0, 1], Vec::new()));
    }

    /// textread.error-missing (case `err-missing`).
    #[test]
    fn rejects_textread_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("no_such_file_xyz.txt");
        let file = chr(missing.to_str().unwrap());
        assert_eq!(err_id(&[file, chr("%f")]), "RunMat:textread:FileNotFound");
    }

    // ---- documented choices (U-052-1, U-052-4 .. U-052-9) -------------------

    /// U-052-4: only %f, %d, %s, %*s.
    #[test]
    fn unresolved_choice_textread_format_grammar() {
        let dir = tempfile::tempdir().unwrap();
        let file = write(dir.path(), "data.txt", "1 2\n");
        for format in [
            "%u", "%5f", "%.2f", "%*d", "%q", "%c", "a%f", "%[abc]", "%", "",
        ] {
            assert_eq!(
                err_n(vec![file.clone(), chr(format)], 1),
                "RunMat:textread:UnsupportedOption",
                "format {format:?}"
            );
        }
        // Whitespace between fields is optional; a string format is accepted.
        let fmt = Value::StringArray(StringArray::new(strings(&["%f%f"]), vec![1, 1]).unwrap());
        let out = read_n(vec![file, fmt], 2).unwrap();
        assert_eq!(doubles(&out[1]), col(&[2.0]));
    }

    /// U-052-5: field data outside the specified forms.
    #[test]
    fn unresolved_choice_textread_field_data() {
        let dir = tempfile::tempdir().unwrap();
        // %f accepts signs, exponents and Inf/NaN spellings.
        let file = write(dir.path(), "floats.txt", "-1.5e1\nInf\n");
        let out = read_n(vec![file, chr("%f")], 1).unwrap();
        assert_eq!(doubles(&out[0]).1[0], -15.0);
        assert!(doubles(&out[0]).1[1].is_infinite());
        // %d accepts signed digit strings only.
        let file = write(dir.path(), "ints.txt", "-3\n+4\n");
        let out = read_n(vec![file, chr("%d")], 1).unwrap();
        assert_eq!(doubles(&out[0]), col(&[-3.0, 4.0]));
        for (name, content, format) in [
            ("bad_float.txt", "abc\n", "%f"),
            ("bad_int.txt", "1.5\n", "%d"),
            ("short.txt", "1 2\n3\n", "%f %f"),
        ] {
            let file = write(dir.path(), name, content);
            assert_eq!(
                err_n(vec![file, chr(format)], 1),
                "RunMat:textread:UnsupportedOption",
                "{name}"
            );
        }
        // Delimiter mode trims fields; an empty field is unsupported.
        let file = write(dir.path(), "spaced.txt", " 1 , 2\r\n\n3,4\n");
        let out = read_n(vec![file, chr("%f %f"), chr("delimiter"), chr(",")], 2).unwrap();
        assert_eq!(doubles(&out[0]), col(&[1.0, 3.0]));
        let file = write(dir.path(), "hole.txt", "1,,2\n");
        assert_eq!(
            err_n(vec![file, chr("%f"), chr("delimiter"), chr(",")], 1),
            "RunMat:textread:UnsupportedOption"
        );
        // An empty file with %s gives a 0x1 cell (U-052-7).
        let file = write(dir.path(), "empty.txt", "");
        let out = read_n(vec![file, chr("%s")], 1).unwrap();
        assert_eq!(cell_chars(&out[0]), (vec![0, 1], Vec::new()));
    }

    /// U-052-6: options.
    #[test]
    fn unresolved_choice_textread_options() {
        let dir = tempfile::tempdir().unwrap();
        let file = write(dir.path(), "data.txt", "h\n1\n");
        // Names are case-insensitive.
        let out = read_n(
            vec![file.clone(), chr("%f"), chr("HeaderLines"), num(1.0)],
            1,
        )
        .unwrap();
        assert_eq!(doubles(&out[0]), col(&[1.0]));
        let unsupported = [
            vec![file.clone(), chr("%f"), chr("commentstyle"), chr("shell")],
            vec![file.clone(), chr("%f"), num(2.0)],
            vec![file.clone(), chr("%f"), chr("delimiter"), chr(",;")],
        ];
        for args in unsupported {
            assert_eq!(err_n(args, 1), "RunMat:textread:UnsupportedOption");
        }
        let invalid = [
            vec![file.clone(), chr("%f"), chr("headerlines")],
            vec![file.clone(), chr("%f"), chr("headerlines"), num(-1.0)],
            vec![file.clone(), chr("%f"), chr("headerlines"), num(1.5)],
            vec![file.clone(), chr("%f"), chr("delimiter"), num(1.0)],
            vec![file.clone(), num(1.0)],
            vec![num(1.0), chr("%f")],
            vec![],
        ];
        for args in invalid {
            assert_eq!(err_n(args, 1), "RunMat:textread:InvalidInput");
        }
    }

    /// U-052-7: the format-less form.
    #[test]
    fn unresolved_choice_textread_default_format() {
        let dir = tempfile::tempdir().unwrap();
        let file = write(dir.path(), "blank.txt", "1 2 3\n\n4 5 6\n");
        let out = read_n(vec![file], 1).unwrap();
        assert_eq!(
            doubles(&out[0]),
            (vec![2, 3], vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0])
        );
        let file = write(dir.path(), "empty.txt", "");
        let out = read_n(vec![file], 1).unwrap();
        assert_eq!(doubles(&out[0]), (vec![0, 0], Vec::new()));
        for (name, content) in [("ragged.txt", "1 2\n3\n"), ("text.txt", "1 a\n")] {
            let file = write(dir.path(), name, content);
            assert_eq!(err_n(vec![file], 1), "RunMat:textread:UnsupportedOption");
        }
    }

    /// U-052-8: output counts.
    #[test]
    fn unresolved_choice_textread_output_count() {
        let dir = tempfile::tempdir().unwrap();
        let file = write(dir.path(), "numeric.txt", "1 2\n3 4\n");
        let out = read_n(vec![file.clone(), chr("%f %f")], 1).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(doubles(&out[0]), col(&[1.0, 3.0]));
        assert_eq!(
            err_n(vec![file.clone(), chr("%f %f")], 3),
            "RunMat:textread:TooManyOutputs"
        );
        assert_eq!(err_n(vec![file], 2), "RunMat:textread:TooManyOutputs");
    }

    /// U-052-9: a directory cannot be read.
    #[test]
    fn unresolved_choice_textread_unreadable_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = chr(dir.path().to_str().unwrap());
        assert_eq!(
            err_n(vec![path, chr("%f")], 1),
            "RunMat:textread:FileNotFound"
        );
    }
}
