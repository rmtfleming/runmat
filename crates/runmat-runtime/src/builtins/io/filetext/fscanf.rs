//! Clean-room `fscanf` (read formatted data from an open text file).
//!
//! Clean-room feature `specs/064-file-io-n5/`. Normative behaviour comes only
//! from the approved export `fscanf` 0.2.0 (pin `1c91aa1`, see that feature's
//! `provenance.md`). Conversions are scanned with the `sscanf` machinery in
//! `strings::core::compat` (`parse_scan_format`, `scan_one`); the read loop,
//! the conversion count, the char/double result class and the size rules are
//! specific to this file. Unobserved forms are labelled documented choices
//! (FR-064-20..23) or raise `RunMat:fscanf:UnsupportedOption`.

use std::io::{Read, Seek, SeekFrom};

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{CharArray, Tensor, Value};

use crate::builtins::io::filetext::{helpers::decode_bytes, registry};
use crate::builtins::strings::core::compat::{
    parse_scan_format, scan_one, skip_whitespace, ScanKind, ScanToken, ScanValue,
};
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const NAME: &str = "fscanf";

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

const P_FID: BuiltinParamDescriptor = param(
    "fid",
    BuiltinParamArity::Required,
    "File identifier returned by fopen.",
);
const P_FORMAT: BuiltinParamDescriptor = param(
    "format",
    BuiltinParamArity::Required,
    "Format text with conversions such as %c, %s, %d, %f, %x and literals.",
);
const P_SIZE: BuiltinParamDescriptor = param(
    "sz",
    BuiltinParamArity::Required,
    "Scalar count n or [m Inf].",
);
const P_A: BuiltinParamDescriptor = param(
    "A",
    BuiltinParamArity::Required,
    "char row for %c/%s formats; double otherwise.",
);
const P_COUNT: BuiltinParamDescriptor = param(
    "count",
    BuiltinParamArity::Required,
    "Number of successful conversions (double).",
);

const FORMAT_INPUTS: [BuiltinParamDescriptor; 2] = [P_FID, P_FORMAT];
const SIZE_INPUTS: [BuiltinParamDescriptor; 3] = [P_FID, P_FORMAT, P_SIZE];
const ONE_OUTPUT: [BuiltinParamDescriptor; 1] = [P_A];
const TWO_OUTPUTS: [BuiltinParamDescriptor; 2] = [P_A, P_COUNT];

const FSCANF_SIGNATURES: [BuiltinSignatureDescriptor; 3] = [
    BuiltinSignatureDescriptor {
        label: "A = fscanf(fid, format)",
        inputs: &FORMAT_INPUTS,
        outputs: &ONE_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "A = fscanf(fid, format, sz)",
        inputs: &SIZE_INPUTS,
        outputs: &ONE_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "[A, count] = fscanf(fid, format, ...)",
        inputs: &SIZE_INPUTS,
        outputs: &TWO_OUTPUTS,
    },
];

const FSCANF_ERRORS: [BuiltinErrorDescriptor; 6] = [
    BuiltinErrorDescriptor {
        code: "RM.FSCANF.INVALID_FID",
        identifier: Some("RunMat:FileIO:InvalidFid"),
        when: "The identifier is not a file opened for reading by fopen.",
        message: "fscanf: invalid file identifier. Use fopen to generate a valid file ID.",
    },
    BuiltinErrorDescriptor {
        code: "RM.FSCANF.MINRHS",
        identifier: Some("RunMat:minrhs"),
        when: "The file identifier or the format is missing.",
        message: "fscanf: not enough input arguments",
    },
    BuiltinErrorDescriptor {
        code: "RM.FSCANF.INVALID_INPUT",
        identifier: Some("RunMat:fscanf:InvalidInput"),
        when: "The format is not text or the size argument is malformed.",
        message: "fscanf: invalid input",
    },
    BuiltinErrorDescriptor {
        code: "RM.FSCANF.UNSUPPORTED_OPTION",
        identifier: Some("RunMat:fscanf:UnsupportedOption"),
        when: "The call uses a form that is not yet specified for RunMat.",
        message: "fscanf: form not yet specified",
    },
    BuiltinErrorDescriptor {
        code: "RM.FSCANF.TOO_MANY_OUTPUTS",
        identifier: Some("RunMat:fscanf:TooManyOutputs"),
        when: "More than two outputs were requested.",
        message: "fscanf: too many outputs",
    },
    BuiltinErrorDescriptor {
        code: "RM.FSCANF.READ_FAILED",
        identifier: Some("RunMat:fscanf:ReadFailed"),
        when: "Reading or decoding the file failed.",
        message: "fscanf: read failed",
    },
];

const INVALID_FID: usize = 0;
const MINRHS: usize = 1;
const INVALID_INPUT: usize = 2;
const UNSUPPORTED_OPTION: usize = 3;
const TOO_MANY_OUTPUTS: usize = 4;
const READ_FAILED: usize = 5;

pub const FSCANF_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &FSCANF_SIGNATURES,
    output_mode: BuiltinOutputMode::ByRequestedOutputCount,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &FSCANF_ERRORS,
};

fn fscanf_error(kind: usize, detail: impl Into<String>) -> RuntimeError {
    let mut builder = build_runtime_error(format!("{NAME}: {}", detail.into())).with_builtin(NAME);
    if let Some(identifier) = FSCANF_ERRORS[kind].identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

// ---------------------------------------------------------------------------
// Builtin
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "fscanf",
    category = "io/filetext",
    summary = "Read formatted data from an open text file.",
    keywords = "fscanf,file,io,scan,format,read",
    accel = "cpu",
    descriptor(crate::builtins::io::filetext::fscanf::FSCANF_DESCRIPTOR),
    builtin_path = "crate::builtins::io::filetext::fscanf"
)]
async fn fscanf_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    let requested = crate::output_count::current_output_count();
    if requested.is_some_and(|count| count > 2) {
        // FR-064-23: only A and count were observed.
        return Err(fscanf_error(
            TOO_MANY_OUTPUTS,
            "at most two outputs (A, count) are supported",
        ));
    }
    let mut gathered = Vec::with_capacity(args.len());
    for value in &args {
        gathered.push(gather_if_needed_async(value).await?);
    }
    // fscanf.error-no-format (FR-064-09).
    if gathered.len() < 2 {
        return Err(fscanf_error(MINRHS, "expected fscanf(fid, format, ...)"));
    }
    if gathered.len() > 3 {
        return Err(fscanf_error(
            UNSUPPORTED_OPTION,
            "only fscanf(fid, format) and fscanf(fid, format, sz) are supported",
        ));
    }
    let fid = parse_fid(&gathered[0])?;
    let format = text_arg(&gathered[1]).ok_or_else(|| {
        fscanf_error(
            INVALID_INPUT,
            "the format must be a char vector or string scalar",
        )
    })?;
    let tokens = parse_tokens(&format)?;
    let size = match gathered.get(2) {
        Some(value) => parse_size(value)?,
        None => ScanSize::All,
    };

    let text = ReadText::open(fid)?;
    let scan = scan_text(&text.text, &tokens, size.limit());
    text.finish(fid, &scan)?;

    let char_result = tokens.iter().all(|token| match token {
        ScanToken::Spec {
            kind,
            suppress: false,
            ..
        } => matches!(kind, ScanKind::Char | ScanKind::String),
        _ => true,
    });
    let value = build_result(scan.values, &size, char_result)?;
    let count = Value::Num(scan.count as f64);
    match requested {
        Some(0) => Ok(Value::OutputList(Vec::new())),
        Some(1) => Ok(Value::OutputList(vec![value])),
        Some(_) => Ok(Value::OutputList(vec![value, count])),
        None => Ok(value),
    }
}

// ---------------------------------------------------------------------------
// Arguments
// ---------------------------------------------------------------------------

fn text_arg(value: &Value) -> Option<String> {
    match value {
        Value::CharArray(chars) if chars.rows <= 1 => Some(chars.data.iter().collect()),
        Value::String(text) => Some(text.clone()),
        Value::StringArray(array) if array.data.len() == 1 => Some(array.data[0].clone()),
        _ => None,
    }
}

fn scalar_number(value: &Value) -> Option<f64> {
    match value {
        Value::Num(n) => Some(*n),
        Value::Tensor(t) => match t.as_f64_slice() {
            Some([n]) => Some(*n),
            _ => None,
        },
        _ => None,
    }
}

/// fscanf.error-bad-fid (FR-064-08) plus FR-064-23 for the other forms.
fn parse_fid(value: &Value) -> BuiltinResult<i32> {
    let invalid = || fscanf_error(INVALID_FID, "file identifier is not open for reading");
    let n = scalar_number(value).ok_or_else(invalid)?;
    if !n.is_finite() || n.fract() != 0.0 || n < 0.0 || n > i32::MAX as f64 {
        return Err(invalid());
    }
    let fid = n as i32;
    if fid < 3 {
        return Err(fscanf_error(
            UNSUPPORTED_OPTION,
            "reading the standard streams (fid 0, 1, 2) is not supported",
        ));
    }
    let info = registry::info_for(fid).ok_or_else(invalid)?;
    let permission = info.permission.to_ascii_lowercase();
    if !(permission.starts_with('r') || permission.contains('+')) {
        return Err(invalid());
    }
    Ok(fid)
}

/// Accept the conversions of the shared scanner except the 64-bit integer
/// forms (FR-064-20); a format without conversions is not specified.
fn parse_tokens(format: &str) -> BuiltinResult<Vec<ScanToken>> {
    let tokens = parse_scan_format(format)
        .map_err(|err| fscanf_error(UNSUPPORTED_OPTION, err.message().replace("sscanf: ", "")))?;
    let mut conversions = 0usize;
    for token in &tokens {
        if let ScanToken::Spec { kind, .. } = token {
            conversions += 1;
            if matches!(
                kind,
                ScanKind::SignedInteger {
                    exact_i64: true,
                    ..
                } | ScanKind::UnsignedInteger {
                    exact_u64: true,
                    ..
                }
            ) {
                return Err(fscanf_error(
                    UNSUPPORTED_OPTION,
                    "64-bit integer conversions (%ld, %lu, ...) are not supported",
                ));
            }
        }
    }
    if conversions == 0 {
        return Err(fscanf_error(
            UNSUPPORTED_OPTION,
            "a format without conversions is not supported",
        ));
    }
    Ok(tokens)
}

#[derive(Clone, Copy, Debug)]
enum ScanSize {
    /// No size or `Inf`: every value, as a column.
    All,
    /// Scalar `n`: at most n values, as a column.
    Count(usize),
    /// `[m n]` (n may be Inf): at most m*n values in m rows.
    Rows(usize, Option<usize>),
}

impl ScanSize {
    fn limit(self) -> Option<usize> {
        match self {
            ScanSize::All => None,
            ScanSize::Count(n) => Some(n),
            ScanSize::Rows(m, Some(n)) => Some(m.saturating_mul(n)),
            ScanSize::Rows(_, None) => None,
        }
    }
}

fn size_dim(value: f64) -> Option<usize> {
    (value.is_finite() && value >= 0.0 && value.fract() == 0.0 && value <= u32::MAX as f64)
        .then_some(value as usize)
}

/// fscanf.size (FR-064-04) plus FR-064-21 for the unobserved shapes.
fn parse_size(value: &Value) -> BuiltinResult<ScanSize> {
    let invalid = || {
        fscanf_error(
            INVALID_INPUT,
            "size must be a nonnegative integer n, Inf, or [m n] with n possibly Inf",
        )
    };
    let data: Vec<f64> = match value {
        Value::Num(n) => vec![*n],
        Value::Tensor(t) => t.as_f64_slice().ok_or_else(invalid)?.to_vec(),
        _ => return Err(invalid()),
    };
    match data.as_slice() {
        [n] if *n == f64::INFINITY => Ok(ScanSize::All),
        [n] => size_dim(*n).map(ScanSize::Count).ok_or_else(invalid),
        [m, n] => {
            let m = size_dim(*m).ok_or_else(invalid)?;
            if *n == f64::INFINITY {
                if m == 0 {
                    return Err(invalid());
                }
                Ok(ScanSize::Rows(m, None))
            } else {
                Ok(ScanSize::Rows(m, Some(size_dim(*n).ok_or_else(invalid)?)))
            }
        }
        _ => Err(invalid()),
    }
}

// ---------------------------------------------------------------------------
// File access
// ---------------------------------------------------------------------------

/// The rest of the file from the current position, decoded.
struct ReadText {
    start: u64,
    byte_len: usize,
    text: String,
    one_byte_chars: bool,
}

impl ReadText {
    fn open(fid: i32) -> BuiltinResult<Self> {
        let encoding = registry::info_for(fid)
            .map(|info| info.encoding)
            .unwrap_or_default();
        let handle = registry::shared_handle(fid)
            .ok_or_else(|| fscanf_error(INVALID_FID, "file identifier is not open"))?;
        let mut guard = handle
            .lock()
            .map_err(|_| fscanf_error(READ_FAILED, "file handle lock poisoned"))?;
        let file = guard
            .as_mut()
            .ok_or_else(|| fscanf_error(INVALID_FID, "file identifier is not open"))?;
        let start = file
            .stream_position()
            .map_err(|err| fscanf_error(READ_FAILED, err.to_string()))?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .map_err(|err| fscanf_error(READ_FAILED, err.to_string()))?;
        drop(guard);
        let chars = decode_bytes(&bytes, &encoding, NAME)
            .map_err(|err| fscanf_error(READ_FAILED, err.message().to_string()))?;
        let one_byte_chars = chars.len() == bytes.len();
        Ok(Self {
            start,
            byte_len: bytes.len(),
            text: chars.into_iter().collect(),
            one_byte_chars,
        })
    }

    /// FR-064-22: leave the file just after the last consumed character and
    /// record end-of-file when everything was consumed.
    fn finish(&self, fid: i32, scan: &Scan) -> BuiltinResult<()> {
        let consumed_bytes = if scan.consumed >= self.text.len() {
            self.byte_len
        } else if self.one_byte_chars {
            self.text[..scan.consumed].chars().count()
        } else if self.text.len() == self.byte_len {
            // UTF-8 decoding without replacement: string offsets are bytes.
            scan.consumed
        } else {
            self.byte_len
        };
        if let Some(handle) = registry::shared_handle(fid) {
            let mut guard = handle
                .lock()
                .map_err(|_| fscanf_error(READ_FAILED, "file handle lock poisoned"))?;
            if let Some(file) = guard.as_mut() {
                file.seek(SeekFrom::Start(self.start + consumed_bytes as u64))
                    .map_err(|err| fscanf_error(READ_FAILED, err.to_string()))?;
            }
        }
        if consumed_bytes >= self.byte_len {
            registry::mark_eof_encountered(fid);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Scanning
// ---------------------------------------------------------------------------

struct Scan {
    values: Vec<f64>,
    count: usize,
    consumed: usize,
}

/// Apply the format repeatedly until a conversion or literal fails, the text
/// ends, or `limit` values have been read (fscanf.literal-stop, FR-064-05).
/// `count` is the number of successful (non-suppressed) conversions
/// (fscanf.s, fscanf.mixed).
fn scan_text(text: &str, tokens: &[ScanToken], limit: Option<usize>) -> Scan {
    let mut values = Vec::new();
    let mut count = 0usize;
    let mut pos = 0usize;
    let mut consumed = 0usize;
    let full = |values: &Vec<f64>| limit.is_some_and(|limit| values.len() >= limit);
    'outer: while !full(&values) {
        let start = pos;
        for token in tokens {
            match token {
                ScanToken::Whitespace => pos = skip_whitespace(text, pos),
                ScanToken::Literal(ch) => {
                    if text[pos..].starts_with(*ch) {
                        pos += ch.len_utf8();
                    } else {
                        break 'outer;
                    }
                }
                ScanToken::Spec {
                    kind,
                    width,
                    suppress,
                } => {
                    let at = if matches!(kind, ScanKind::Char) {
                        pos
                    } else {
                        skip_whitespace(text, pos)
                    };
                    let Some((parsed, next)) = scan_one(text, at, *kind, *width) else {
                        break 'outer;
                    };
                    pos = next;
                    if !*suppress {
                        count += 1;
                        values.extend(parsed.into_iter().map(|value| match value {
                            ScanValue::F64(v) => v,
                            ScanValue::I64(v) => v as f64,
                            ScanValue::U64(v) => v as f64,
                        }));
                        if full(&values) {
                            consumed = pos;
                            break 'outer;
                        }
                    }
                }
            }
            consumed = pos;
        }
        if pos == start || pos >= text.len() {
            break;
        }
    }
    if let Some(limit) = limit {
        values.truncate(limit);
    }
    Scan {
        values,
        count,
        consumed,
    }
}

/// Shape the values (fscanf.numeric, fscanf.size, fscanf.empty, FR-064-21).
fn build_result(mut values: Vec<f64>, size: &ScanSize, char_result: bool) -> BuiltinResult<Value> {
    let n = values.len();
    let (rows, cols) = match size {
        _ if n == 0 => (0, 0),
        ScanSize::Rows(m, _) => {
            let m = *m;
            let cols = n.div_ceil(m);
            values.resize(m * cols, 0.0);
            (m, cols)
        }
        _ if char_result => (1, n),
        _ => (n, 1),
    };
    let internal = |err: String| fscanf_error(READ_FAILED, err);
    if char_result {
        let chars: Vec<char> = values
            .iter()
            .map(|code| char::from_u32(*code as u32).unwrap_or('\u{FFFD}'))
            .collect();
        // Column-major data into a row-major CharArray.
        let mut row_major = vec!['\0'; chars.len()];
        for (index, ch) in chars.into_iter().enumerate() {
            let (r, c) = (index % rows.max(1), index / rows.max(1));
            row_major[r * cols + c] = ch;
        }
        CharArray::new(row_major, rows, cols)
            .map(Value::CharArray)
            .map_err(internal)
    } else {
        Tensor::new(values, vec![rows, cols])
            .map(Value::Tensor)
            .map_err(internal)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod fscanf_tests {
    use super::*;
    use crate::builtins::io::filetext::fopen;
    use futures::executor::block_on;
    use std::path::Path;

    struct Fixture {
        _dir: tempfile::TempDir,
        _guard: std::sync::MutexGuard<'static, ()>,
        fid: f64,
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = registry::close(self.fid as i32);
        }
    }

    /// Write `content` to a file in a fresh temporary directory and fopen it.
    fn open_with(content: &str) -> Fixture {
        let guard = registry::test_guard();
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("data.txt");
        std::fs::write(&path, content).expect("write temp file");
        let fid = fopen_read(&path);
        Fixture {
            _dir: dir,
            _guard: guard,
            fid,
        }
    }

    fn fopen_read(path: &Path) -> f64 {
        let eval = block_on(fopen::evaluate(&[Value::from(
            path.to_string_lossy().to_string(),
        )]))
        .expect("fopen");
        let fid = eval.as_open().expect("open outputs").fid;
        assert!(fid >= 3.0);
        fid
    }

    fn chr(text: &str) -> Value {
        Value::CharArray(CharArray::new_row(text))
    }

    fn call_n(args: Vec<Value>, nargout: usize) -> BuiltinResult<Vec<Value>> {
        let _guard = crate::output_count::push_output_count(Some(nargout));
        match block_on(fscanf_builtin(args))? {
            Value::OutputList(values) => Ok(values),
            other => panic!("expected an output list, got {other:?}"),
        }
    }

    /// `[A, count] = fscanf(fid, format, extra...)`.
    fn scan2(fx: &Fixture, format: &str, extra: Vec<Value>) -> (Value, f64) {
        let mut args = vec![Value::Num(fx.fid), chr(format)];
        args.extend(extra);
        let out = call_n(args, 2).expect("fscanf");
        let count = match out[1] {
            Value::Num(n) => n,
            ref other => panic!("count must be double, got {other:?}"),
        };
        (out[0].clone(), count)
    }

    fn identifier(err: RuntimeError) -> String {
        err.identifier().unwrap_or("<none>").to_string()
    }

    fn assert_char(value: &Value, text: &str, rows: usize, cols: usize) {
        match value {
            Value::CharArray(ca) => {
                assert_eq!((ca.rows, ca.cols), (rows, cols), "char size");
                assert_eq!(ca.data.iter().collect::<String>(), text);
            }
            other => panic!("expected char, got {other:?}"),
        }
    }

    fn assert_double(value: &Value, data: &[f64], shape: &[usize]) {
        match value {
            Value::Tensor(t) => {
                assert!(t.integer_storage().is_none(), "class must be double");
                assert_eq!(t.shape, shape, "double size");
                assert_eq!(t.as_f64_slice().expect("f64 data"), data);
            }
            other => panic!("expected double tensor, got {other:?}"),
        }
    }

    // ---- observed cases (normative) ----------------------------------------

    // c-all [fscanf.signature-primary, fscanf.c; FR-064-01]
    #[test]
    fn observed_c_all() {
        let fx = open_with("ab c\nde");
        let (a, count) = scan2(&fx, "%c", vec![]);
        assert_char(&a, "ab c\nde", 1, 7);
        assert_eq!(count, 7.0);
    }

    // c-all, one-output COBRA form through the dispatcher [FR-064-01]
    #[test]
    fn observed_c_all_one_output() {
        let fx = open_with("ab c\nde");
        let a = block_on(crate::call_builtin_async(
            "fscanf",
            &[Value::Num(fx.fid), chr("%c")],
        ))
        .expect("fscanf");
        assert_char(&a, "ab c\nde", 1, 7);
    }

    // s-skip [fscanf.s; FR-064-02]
    #[test]
    fn observed_s_skip() {
        let fx = open_with("ab c\nde");
        let (a, count) = scan2(&fx, "%s", vec![]);
        assert_char(&a, "abcde", 1, 5);
        assert_eq!(count, 3.0);
    }

    // c-trailing-newline [fscanf.c; FR-064-01]
    #[test]
    fn observed_c_trailing_newline() {
        let fx = open_with("xy\n");
        let (a, count) = scan2(&fx, "%c", vec![]);
        assert_char(&a, "xy\n", 1, 3);
        assert_eq!(count, 3.0);
    }

    // d-ints [fscanf.numeric; FR-064-03]
    #[test]
    fn observed_d_ints() {
        let fx = open_with("1 2 3");
        let (a, count) = scan2(&fx, "%d", vec![]);
        assert_double(&a, &[1.0, 2.0, 3.0], &[3, 1]);
        assert_eq!(count, 3.0);
    }

    // f-reals [fscanf.numeric; FR-064-03]
    #[test]
    fn observed_f_reals() {
        let fx = open_with("1.5 -2\n3e2");
        let (a, count) = scan2(&fx, "%f", vec![]);
        assert_double(&a, &[1.5, -2.0, 300.0], &[3, 1]);
        assert_eq!(count, 3.0);
    }

    // size-matrix [fscanf.size; FR-064-04]
    #[test]
    fn observed_size_matrix() {
        let fx = open_with("1 2 3 4 5 6");
        let size = Tensor::new(vec![2.0, f64::INFINITY], vec![1, 2]).unwrap();
        let (a, count) = scan2(&fx, "%f", vec![Value::Tensor(size)]);
        assert_double(&a, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0], &[2, 3]);
        assert_eq!(count, 6.0);
    }

    // size-scalar [fscanf.size; FR-064-04]
    #[test]
    fn observed_size_scalar() {
        let fx = open_with("1 2 3");
        let (a, count) = scan2(&fx, "%d", vec![Value::Num(2.0)]);
        assert_double(&a, &[1.0, 2.0], &[2, 1]);
        assert_eq!(count, 2.0);
    }

    // literal [fscanf.literal-stop; FR-064-05]
    #[test]
    fn observed_literal() {
        let fx = open_with("1,2,3");
        let (a, count) = scan2(&fx, "%d,", vec![]);
        assert_double(&a, &[1.0, 2.0, 3.0], &[3, 1]);
        assert_eq!(count, 3.0);
    }

    // mismatch [fscanf.literal-stop; FR-064-05]
    #[test]
    fn observed_mismatch() {
        let fx = open_with("abc");
        let (a, count) = scan2(&fx, "%d", vec![]);
        assert_double(&a, &[], &[0, 0]);
        assert_eq!(count, 0.0);
    }

    // partial [fscanf.literal-stop; FR-064-05]
    #[test]
    fn observed_partial() {
        let fx = open_with("4 5 x 6");
        let (a, count) = scan2(&fx, "%d", vec![]);
        assert_double(&a, &[4.0, 5.0], &[2, 1]);
        assert_eq!(count, 2.0);
    }

    // hex [fscanf.numeric; FR-064-03]
    #[test]
    fn observed_hex() {
        let fx = open_with("1f a");
        let (a, count) = scan2(&fx, "%x", vec![]);
        assert_double(&a, &[31.0, 10.0], &[2, 1]);
        assert_eq!(count, 2.0);
    }

    // mixed-d-s [fscanf.mixed; FR-064-07] (q-mixed: reproduced, maintainer)
    #[test]
    fn observed_mixed_d_s() {
        let fx = open_with("7 ab");
        let (a, count) = scan2(&fx, "%d%s", vec![]);
        assert_double(&a, &[7.0, 97.0, 98.0], &[3, 1]);
        assert_eq!(count, 2.0);
    }

    // empty-c [fscanf.empty; FR-064-06]
    #[test]
    fn observed_empty_c() {
        let fx = open_with("");
        let (a, count) = scan2(&fx, "%c", vec![]);
        assert_char(&a, "", 0, 0);
        assert_eq!(count, 0.0);
    }

    // empty-d [fscanf.empty; FR-064-06]
    #[test]
    fn observed_empty_d() {
        let fx = open_with("");
        let (a, count) = scan2(&fx, "%d", vec![]);
        assert_double(&a, &[], &[0, 0]);
        assert_eq!(count, 0.0);
    }

    // err-bad-fid [fscanf.error-bad-fid; FR-064-08]
    #[test]
    fn observed_err_bad_fid() {
        let _guard = registry::test_guard();
        let err = call_n(vec![Value::Num(-1.0), chr("%d")], 1).expect_err("bad fid");
        assert_eq!(identifier(err), "RunMat:FileIO:InvalidFid");
    }

    // err-no-format [fscanf.error-no-format; FR-064-09]
    #[test]
    fn observed_err_no_format() {
        let fx = open_with("1 2");
        let err = call_n(vec![Value::Num(fx.fid)], 1).expect_err("no format");
        assert_eq!(identifier(err), "RunMat:minrhs");
    }

    // ---- documented choices (non-normative) --------------------------------

    // FR-064-20: other conversions of the shared scanner; unsupported forms.
    #[test]
    fn documented_choice_other_conversions() {
        let fx = open_with("1e1 2 x3");
        let (a, count) = scan2(&fx, "%g %*d x%d", vec![]);
        assert_double(&a, &[10.0, 3.0], &[2, 1]);
        assert_eq!(count, 2.0);
        drop(fx);

        for format in ["%ld", "abc", "", "%q"] {
            let fx = open_with("1 2");
            let err = call_n(vec![Value::Num(fx.fid), chr(format)], 1).expect_err(format);
            assert_eq!(
                identifier(err),
                "RunMat:fscanf:UnsupportedOption",
                "{format}"
            );
        }
    }

    // FR-064-21: unobserved size shapes.
    #[test]
    fn documented_choice_size_shapes() {
        let fx = open_with("1 2 3 4 5");
        let size = Tensor::new(vec![2.0, f64::INFINITY], vec![1, 2]).unwrap();
        let (a, count) = scan2(&fx, "%d", vec![Value::Tensor(size)]);
        assert_double(&a, &[1.0, 2.0, 3.0, 4.0, 5.0, 0.0], &[2, 3]);
        assert_eq!(count, 5.0);
        drop(fx);

        let fx = open_with("1 2 3 4 5");
        let size = Tensor::new(vec![2.0, 2.0], vec![1, 2]).unwrap();
        let (a, count) = scan2(&fx, "%d", vec![Value::Tensor(size)]);
        assert_double(&a, &[1.0, 2.0, 3.0, 4.0], &[2, 2]);
        assert_eq!(count, 4.0);
        drop(fx);

        let fx = open_with("abcdef");
        let (a, count) = scan2(&fx, "%c", vec![Value::Num(3.0)]);
        assert_char(&a, "abc", 1, 3);
        assert_eq!(count, 3.0);
        drop(fx);

        let fx = open_with("1 2");
        let (a, _) = scan2(&fx, "%d", vec![Value::Num(f64::INFINITY)]);
        assert_double(&a, &[1.0, 2.0], &[2, 1]);
        drop(fx);

        for bad in [Value::Num(-1.0), Value::Num(1.5), chr("x")] {
            let fx = open_with("1 2");
            let err = call_n(vec![Value::Num(fx.fid), chr("%d"), bad], 1).expect_err("bad size");
            assert_eq!(identifier(err), "RunMat:fscanf:InvalidInput");
        }
    }

    // FR-064-22: the file position follows the last consumed character.
    #[test]
    fn documented_choice_file_position() {
        let fx = open_with("1 2 3 4");
        let (a, _) = scan2(&fx, "%d", vec![Value::Num(2.0)]);
        assert_double(&a, &[1.0, 2.0], &[2, 1]);
        let (b, _) = scan2(&fx, "%d", vec![]);
        assert_double(&b, &[3.0, 4.0], &[2, 1]);
        assert_eq!(registry::eof_encountered(fx.fid as i32), Some(true));
        let (c, count) = scan2(&fx, "%d", vec![]);
        assert_double(&c, &[], &[0, 0]);
        assert_eq!(count, 0.0);
    }

    // FR-064-23: argument forms that were not observed.
    #[test]
    fn documented_choice_arguments() {
        let fx = open_with("1 2");
        let fid = Value::Num(fx.fid);
        let id = |args: Vec<Value>, nargout: usize| {
            identifier(call_n(args, nargout).expect_err("expected an error"))
        };
        assert_eq!(id(vec![], 1), "RunMat:minrhs");
        assert_eq!(
            id(vec![Value::Num(0.0), chr("%d")], 1),
            "RunMat:fscanf:UnsupportedOption"
        );
        assert_eq!(id(vec![chr("f"), chr("%d")], 1), "RunMat:FileIO:InvalidFid");
        assert_eq!(
            id(vec![Value::Num(9999.0), chr("%d")], 1),
            "RunMat:FileIO:InvalidFid"
        );
        assert_eq!(
            id(vec![fid.clone(), Value::Num(1.0)], 1),
            "RunMat:fscanf:InvalidInput"
        );
        assert_eq!(
            id(
                vec![fid.clone(), chr("%d"), Value::Num(1.0), Value::Num(1.0)],
                1
            ),
            "RunMat:fscanf:UnsupportedOption"
        );
        assert_eq!(id(vec![fid, chr("%d")], 3), "RunMat:fscanf:TooManyOutputs");
    }

    // FR-064-23: a file opened for writing only is not readable.
    #[test]
    fn documented_choice_write_only_fid() {
        let _guard = registry::test_guard();
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("out.txt");
        let eval = block_on(fopen::evaluate(&[
            Value::from(path.to_string_lossy().to_string()),
            chr("w"),
        ]))
        .expect("fopen w");
        let fid = eval.as_open().expect("open").fid;
        let err = call_n(vec![Value::Num(fid), chr("%d")], 1).expect_err("write-only");
        let _ = registry::close(fid as i32);
        assert_eq!(identifier(err), "RunMat:FileIO:InvalidFid");
    }
}
