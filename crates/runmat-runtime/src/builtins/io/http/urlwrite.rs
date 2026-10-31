//! Clean-room `urlwrite` (save the content at a URL to a file).
//!
//! Clean-room feature `specs/064-file-io-n5/`. Normative behaviour comes only
//! from the approved export `urlwrite` 0.2.0 (pin `1c91aa1`), which was
//! observed with local `file://` URLs and an unknown protocol only. The
//! http/https path is a **labelled documented choice, not observed
//! behaviour** (maintainer disposition of `urlwrite.q-network`): it uses the
//! shared transport in `io/http/transport.rs`, as `urlread` and `webread` do.

use std::path::{Path, PathBuf};
use std::time::Duration;

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_filesystem as fs;
use runmat_macros::runtime_builtin;
use runmat_value::{CharArray, Value};
use url::Url;

use super::transport::{self, HttpMethod, HttpRequest};
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const NAME: &str = "urlwrite";
const DEFAULT_TIMEOUT_SECONDS: f64 = 60.0;
const DEFAULT_USER_AGENT: &str = "RunMat urlwrite/0.0";

// ---------------------------------------------------------------------------
// Descriptors
// ---------------------------------------------------------------------------

const fn param(
    name: &'static str,
    ty: BuiltinParamType,
    description: &'static str,
) -> BuiltinParamDescriptor {
    BuiltinParamDescriptor {
        name,
        ty,
        arity: BuiltinParamArity::Required,
        default: None,
        description,
    }
}

const P_URL: BuiltinParamDescriptor = param(
    "url",
    BuiltinParamType::StringScalar,
    "URL text (file://; http:// and https:// as a documented choice).",
);
const P_FILENAME: BuiltinParamDescriptor = param(
    "filename",
    BuiltinParamType::StringScalar,
    "Destination file path.",
);
const P_F: BuiltinParamDescriptor = param(
    "f",
    BuiltinParamType::Any,
    "char path of the written file; empty char on failure with two outputs.",
);
const P_STATUS: BuiltinParamDescriptor = param(
    "status",
    BuiltinParamType::Any,
    "double 1 on success, 0 on failure.",
);

const INPUTS: [BuiltinParamDescriptor; 2] = [P_URL, P_FILENAME];
const ONE_OUTPUT: [BuiltinParamDescriptor; 1] = [P_F];
const TWO_OUTPUTS: [BuiltinParamDescriptor; 2] = [P_F, P_STATUS];

const URLWRITE_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "f = urlwrite(url, filename)",
        inputs: &INPUTS,
        outputs: &ONE_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "[f, status] = urlwrite(url, filename)",
        inputs: &INPUTS,
        outputs: &TWO_OUTPUTS,
    },
];

const URLWRITE_ERRORS: [BuiltinErrorDescriptor; 8] = [
    BuiltinErrorDescriptor {
        code: "RM.URLWRITE.MINRHS",
        identifier: Some("RunMat:minrhs"),
        when: "The URL or the file name is missing.",
        message: "urlwrite: not enough input arguments",
    },
    BuiltinErrorDescriptor {
        code: "RM.URLWRITE.FILE_NOT_FOUND",
        identifier: Some("RunMat:urlwrite:FileNotFound"),
        when: "With one output, the file:// source cannot be read.",
        message: "urlwrite: file not found",
    },
    BuiltinErrorDescriptor {
        code: "RM.URLWRITE.INVALID_OUTPUT_LOCATION",
        identifier: Some("RunMat:urlwrite:InvalidOutputLocation"),
        when: "The destination folder does not exist or the file cannot be written.",
        message: "urlwrite: invalid output location",
    },
    BuiltinErrorDescriptor {
        code: "RM.URLWRITE.INVALID_URL",
        identifier: Some("RunMat:urlwrite:InvalidUrl"),
        when: "With one output, the URL cannot be parsed or its protocol is unknown.",
        message: "urlwrite: invalid URL",
    },
    BuiltinErrorDescriptor {
        code: "RM.URLWRITE.CONNECTION_FAILED",
        identifier: Some("RunMat:urlwrite:ConnectionFailed"),
        when: "With one output, the http/https transfer fails (documented choice).",
        message: "urlwrite: connection failed",
    },
    BuiltinErrorDescriptor {
        code: "RM.URLWRITE.INVALID_INPUT",
        identifier: Some("RunMat:urlwrite:InvalidInput"),
        when: "The URL or the file name is not text.",
        message: "urlwrite: invalid input",
    },
    BuiltinErrorDescriptor {
        code: "RM.URLWRITE.UNSUPPORTED_OPTION",
        identifier: Some("RunMat:urlwrite:UnsupportedOption"),
        when: "Arguments beyond url and filename were given.",
        message: "urlwrite: form not yet specified",
    },
    BuiltinErrorDescriptor {
        code: "RM.URLWRITE.TOO_MANY_OUTPUTS",
        identifier: Some("RunMat:urlwrite:TooManyOutputs"),
        when: "More than two outputs were requested.",
        message: "urlwrite: too many outputs",
    },
];

const MINRHS: usize = 0;
const FILE_NOT_FOUND: usize = 1;
const INVALID_OUTPUT_LOCATION: usize = 2;
const INVALID_URL: usize = 3;
const CONNECTION_FAILED: usize = 4;
const INVALID_INPUT: usize = 5;
const UNSUPPORTED_OPTION: usize = 6;
const TOO_MANY_OUTPUTS: usize = 7;

pub const URLWRITE_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &URLWRITE_SIGNATURES,
    output_mode: BuiltinOutputMode::ByRequestedOutputCount,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &URLWRITE_ERRORS,
};

fn urlwrite_error(kind: usize, detail: impl Into<String>) -> RuntimeError {
    let mut builder = build_runtime_error(format!("{NAME}: {}", detail.into())).with_builtin(NAME);
    if let Some(identifier) = URLWRITE_ERRORS[kind].identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

// ---------------------------------------------------------------------------
// Builtin
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "urlwrite",
    category = "io/http",
    summary = "Save the content at a URL to a file and return the file path.",
    keywords = "urlwrite,url,download,save,file url,http",
    accel = "sink",
    descriptor(crate::builtins::io::http::urlwrite::URLWRITE_DESCRIPTOR),
    builtin_path = "crate::builtins::io::http::urlwrite"
)]
async fn urlwrite_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    let requested = crate::output_count::current_output_count();
    if requested.is_some_and(|count| count > 2) {
        return Err(urlwrite_error(
            TOO_MANY_OUTPUTS,
            "at most two outputs (f, status) are supported",
        ));
    }
    let with_status = requested == Some(2);
    let mut gathered = Vec::with_capacity(args.len());
    for value in &args {
        gathered.push(gather_if_needed_async(value).await?);
    }
    // urlwrite.error-one-arg (FR-064-16).
    if gathered.len() < 2 {
        return Err(urlwrite_error(MINRHS, "expected urlwrite(url, filename)"));
    }
    // urlwrite.q-options (FR-064-25).
    if gathered.len() > 2 {
        return Err(urlwrite_error(
            UNSUPPORTED_OPTION,
            "arguments beyond url and filename are not supported",
        ));
    }
    let url_text = text_arg(&gathered[0])
        .ok_or_else(|| urlwrite_error(INVALID_INPUT, "the URL must be a char vector or string"))?;
    let filename = text_arg(&gathered[1])
        .filter(|name| !name.is_empty())
        .ok_or_else(|| {
            urlwrite_error(
                INVALID_INPUT,
                "the file name must be a nonempty char vector or string",
            )
        })?;

    // urlwrite.error-bad-dir (FR-064-14): raised with one or two outputs,
    // checked before the source (FR-064-25 ordering choice).
    let destination = PathBuf::from(&filename);
    check_destination_folder(&destination).await?;

    let outcome = fetch(&url_text).await;
    let result = match outcome {
        Ok(bytes) => {
            fs::write_async(&destination, &bytes).await.map_err(|err| {
                urlwrite_error(
                    INVALID_OUTPUT_LOCATION,
                    format!("cannot write '{filename}': {err}"),
                )
            })?;
            Ok(())
        }
        Err(err) => Err(err),
    };

    let outputs = match result {
        // urlwrite.success (FR-064-11).
        Ok(()) => vec![
            Value::CharArray(CharArray::new_row(&filename)),
            Value::Num(1.0),
        ],
        // urlwrite.status-failure (FR-064-12): no error with two outputs.
        Err(_) if with_status => vec![empty_char(), Value::Num(0.0)],
        // urlwrite.error-missing-one-out / error-protocol (FR-064-13, 15).
        Err(err) => return Err(err),
    };
    match requested {
        Some(0) => Ok(Value::OutputList(Vec::new())),
        Some(count) => Ok(Value::OutputList(outputs.into_iter().take(count).collect())),
        None => Ok(outputs.into_iter().next().expect("f output")),
    }
}

fn text_arg(value: &Value) -> Option<String> {
    match value {
        Value::CharArray(chars) if chars.rows <= 1 => Some(chars.data.iter().collect()),
        Value::String(text) => Some(text.clone()),
        Value::StringArray(array) if array.data.len() == 1 => Some(array.data[0].clone()),
        _ => None,
    }
}

fn empty_char() -> Value {
    Value::CharArray(CharArray::new(Vec::new(), 0, 0).expect("empty char"))
}

async fn check_destination_folder(destination: &Path) -> BuiltinResult<()> {
    let folder = match destination.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    };
    match fs::metadata_async(&folder).await {
        Ok(meta) if meta.is_dir() => Ok(()),
        _ => Err(urlwrite_error(
            INVALID_OUTPUT_LOCATION,
            format!("the folder '{}' does not exist", folder.display()),
        )),
    }
}

/// Scheme routing decision, pure so it is testable without I/O.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Route {
    File,
    Http,
    Unknown,
}

fn route_for(url: &Url) -> Route {
    match url.scheme() {
        "file" => Route::File,
        "http" | "https" => Route::Http,
        _ => Route::Unknown,
    }
}

async fn fetch(url_text: &str) -> BuiltinResult<Vec<u8>> {
    let trimmed = url_text.trim();
    // An unparseable URL is treated like an unknown protocol (FR-064-25).
    let url = Url::parse(trimmed)
        .map_err(|err| urlwrite_error(INVALID_URL, format!("invalid URL '{trimmed}': {err}")))?;
    match route_for(&url) {
        Route::File => read_file_url(&url).await,
        Route::Http => fetch_http(url),
        Route::Unknown => Err(urlwrite_error(
            INVALID_URL,
            format!("unknown protocol '{}'", url.scheme()),
        )),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn file_url_to_path(url: &Url) -> Option<PathBuf> {
    url.to_file_path().ok()
}

#[cfg(target_arch = "wasm32")]
fn file_url_to_path(url: &Url) -> Option<PathBuf> {
    Some(PathBuf::from(url.path()))
}

/// urlwrite.success / urlwrite.error-missing-one-out.
async fn read_file_url(url: &Url) -> BuiltinResult<Vec<u8>> {
    let not_found = |detail: String| urlwrite_error(FILE_NOT_FOUND, detail);
    let path = file_url_to_path(url)
        .ok_or_else(|| not_found(format!("cannot resolve file URL '{url}'")))?;
    fs::read_async(&path)
        .await
        .map_err(|err| not_found(format!("cannot read '{}': {err}", path.display())))
}

/// Documented choice (urlwrite.q-network, maintainer): http/https through the
/// shared transport; the body bytes are written unchanged.
fn fetch_http(url: Url) -> BuiltinResult<Vec<u8>> {
    let request = HttpRequest {
        url,
        method: HttpMethod::Get,
        headers: Vec::new(),
        body: None,
        timeout: Duration::from_secs_f64(DEFAULT_TIMEOUT_SECONDS),
        user_agent: DEFAULT_USER_AGENT.to_string(),
    };
    transport::send_request(&request)
        .map(|response| response.body)
        .map_err(|err| {
            urlwrite_error(
                CONNECTION_FAILED,
                err.message_with_prefix("transfer failed"),
            )
        })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

// Every test is network-free: file:// temp files, and for http a loopback
// listener on 127.0.0.1 (the pattern used by the webread tests).
#[cfg(test)]
mod urlwrite_tests {
    use super::*;
    use futures::executor::block_on;

    fn chr(text: &str) -> Value {
        Value::CharArray(CharArray::new_row(text))
    }

    fn call_n(args: Vec<Value>, nargout: usize) -> BuiltinResult<Vec<Value>> {
        let _guard = crate::output_count::push_output_count(Some(nargout));
        match block_on(urlwrite_builtin(args))? {
            Value::OutputList(values) => Ok(values),
            other => panic!("expected an output list, got {other:?}"),
        }
    }

    fn identifier(err: RuntimeError) -> String {
        err.identifier().unwrap_or("<none>").to_string()
    }

    fn path_text(path: &Path) -> String {
        path.to_str().expect("utf-8 temp path").to_string()
    }

    /// `['file://' src]` as in the observations.
    fn file_url(path: &Path) -> String {
        format!("file://{}", path_text(path))
    }

    struct Setup {
        _dir: tempfile::TempDir,
        src: PathBuf,
        dst: PathBuf,
        missing: PathBuf,
    }

    fn setup() -> Setup {
        let dir = tempfile::tempdir().expect("temp dir");
        let src = dir.path().join("src.txt");
        std::fs::write(&src, "abc 123").expect("write source");
        Setup {
            src,
            dst: dir.path().join("dst.txt"),
            missing: dir.path().join("missing.txt"),
            _dir: dir,
        }
    }

    fn assert_char(value: &Value, text: &str) {
        match value {
            Value::CharArray(ca) => assert_eq!(ca.data.iter().collect::<String>(), text),
            other => panic!("expected char, got {other:?}"),
        }
    }

    fn assert_empty_char(value: &Value) {
        match value {
            Value::CharArray(ca) => assert!(ca.data.is_empty(), "f must be empty"),
            other => panic!("expected char, got {other:?}"),
        }
    }

    // ---- observed cases (normative) ----------------------------------------

    // file-url [urlwrite.signature-primary, urlwrite.success; FR-064-11]
    #[test]
    fn observed_file_url() {
        let s = setup();
        let dst = path_text(&s.dst);
        let f = block_on(crate::call_builtin_async(
            "urlwrite",
            &[chr(&file_url(&s.src)), chr(&dst)],
        ))
        .expect("urlwrite");
        assert_char(&f, &dst);
        assert_eq!(std::fs::read_to_string(&s.dst).unwrap(), "abc 123");
    }

    // file-url-status [urlwrite.success; FR-064-11]
    #[test]
    fn observed_file_url_status() {
        let s = setup();
        let dst = path_text(&s.dst);
        let out = call_n(vec![chr(&file_url(&s.src)), chr(&dst)], 2).expect("urlwrite");
        assert_char(&out[0], &dst);
        assert_eq!(out[1], Value::Num(1.0));
        assert_eq!(std::fs::read_to_string(&s.dst).unwrap(), "abc 123");
    }

    // missing-two-out [urlwrite.status-failure; FR-064-12]
    #[test]
    fn observed_missing_two_out() {
        let s = setup();
        let out = call_n(vec![chr(&file_url(&s.missing)), chr(&path_text(&s.dst))], 2)
            .expect("no error with two outputs");
        assert_empty_char(&out[0]);
        assert_eq!(out[1], Value::Num(0.0));
        assert!(!s.dst.exists());
    }

    // err-missing-one-out [urlwrite.error-missing-one-out; FR-064-13]
    #[test]
    fn observed_err_missing_one_out() {
        let s = setup();
        let err = call_n(vec![chr(&file_url(&s.missing)), chr(&path_text(&s.dst))], 1)
            .expect_err("missing source");
        assert_eq!(identifier(err), "RunMat:urlwrite:FileNotFound");
        assert!(!s.dst.exists());
    }

    // err-bad-dir, err-bad-dir-two-out [urlwrite.error-bad-dir; FR-064-14]
    #[test]
    fn observed_err_bad_dir() {
        let s = setup();
        let bad = s.dst.with_file_name("no_such_folder").join("x.txt");
        for nargout in [1, 2] {
            let err = call_n(vec![chr(&file_url(&s.src)), chr(&path_text(&bad))], nargout)
                .expect_err("bad dir");
            assert_eq!(identifier(err), "RunMat:urlwrite:InvalidOutputLocation");
        }
    }

    // err-protocol [urlwrite.error-protocol; FR-064-15]
    #[test]
    fn observed_err_protocol() {
        let s = setup();
        let err = call_n(vec![chr("nosuchprotocol://x"), chr(&path_text(&s.dst))], 1)
            .expect_err("unknown protocol");
        assert_eq!(identifier(err), "RunMat:urlwrite:InvalidUrl");
    }

    // protocol-two-out [urlwrite.status-failure; FR-064-12]
    #[test]
    fn observed_protocol_two_out() {
        let s = setup();
        let out = call_n(vec![chr("nosuchprotocol://x"), chr(&path_text(&s.dst))], 2)
            .expect("no error with two outputs");
        assert_empty_char(&out[0]);
        assert_eq!(out[1], Value::Num(0.0));
        assert!(!s.dst.exists());
    }

    // err-one-arg [urlwrite.error-one-arg; FR-064-16]
    #[test]
    fn observed_err_one_arg() {
        let err = call_n(vec![chr("file:///tmp")], 1).expect_err("one arg");
        assert_eq!(identifier(err), "RunMat:minrhs");
    }

    // ---- documented choices (non-normative) --------------------------------

    /// One-shot loopback HTTP server; returns its base URL.
    #[cfg(not(target_arch = "wasm32"))]
    fn spawn_server(status_line: &'static str, body: &'static [u8]) -> String {
        use std::io::{Read as _, Write as _};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind loopback");
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 1024];
                let _ = stream.read(&mut buf);
                let head = format!(
                    "HTTP/1.1 {status_line}\r\nContent-Length: {}\r\nContent-Type: application/octet-stream\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(head.as_bytes());
                let _ = stream.write_all(body);
            }
        });
        format!("http://{addr}")
    }

    // FR-064-24: http through the shared transport (loopback only).
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn documented_choice_http_loopback() {
        let s = setup();
        let base = spawn_server("200 OK", b"\x00bin\xffdata");
        let dst = path_text(&s.dst);
        let out = call_n(vec![chr(&format!("{base}/file.bin")), chr(&dst)], 2).expect("http");
        assert_char(&out[0], &dst);
        assert_eq!(out[1], Value::Num(1.0));
        assert_eq!(std::fs::read(&s.dst).unwrap(), b"\x00bin\xffdata");
    }

    // FR-064-24: http failures: status 0 with two outputs, error with one.
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn documented_choice_http_failure() {
        let s = setup();
        let dst = path_text(&s.dst);
        let base = spawn_server("404 Not Found", b"");
        let out = call_n(vec![chr(&format!("{base}/x")), chr(&dst)], 2).expect("status 0");
        assert_empty_char(&out[0]);
        assert_eq!(out[1], Value::Num(0.0));
        assert!(!s.dst.exists());

        let base = spawn_server("500 Internal Server Error", b"");
        let err = call_n(vec![chr(&format!("{base}/x")), chr(&dst)], 1).expect_err("one out");
        assert_eq!(identifier(err), "RunMat:urlwrite:ConnectionFailed");
    }

    // FR-064-24: routing classification (pure, no request).
    #[test]
    fn documented_choice_routing() {
        let route = |text: &str| route_for(&Url::parse(text).unwrap());
        assert_eq!(route("http://example.invalid/x"), Route::Http);
        assert_eq!(route("https://example.invalid/x"), Route::Http);
        assert_eq!(route("file:///tmp/x"), Route::File);
        assert_eq!(route("ftp://example.invalid/x"), Route::Unknown);
    }

    // FR-064-25: argument forms that were not observed.
    #[test]
    fn documented_choice_arguments() {
        let s = setup();
        let src = chr(&file_url(&s.src));
        let dst = chr(&path_text(&s.dst));
        let id = |args: Vec<Value>, nargout: usize| {
            identifier(call_n(args, nargout).expect_err("expected an error"))
        };
        assert_eq!(id(vec![], 1), "RunMat:minrhs");
        assert_eq!(
            id(vec![src.clone(), dst.clone(), chr("get")], 1),
            "RunMat:urlwrite:UnsupportedOption"
        );
        assert_eq!(
            id(vec![Value::Num(1.0), dst.clone()], 1),
            "RunMat:urlwrite:InvalidInput"
        );
        assert_eq!(
            id(vec![src.clone(), chr("")], 1),
            "RunMat:urlwrite:InvalidInput"
        );
        assert_eq!(
            id(vec![src.clone(), dst.clone()], 3),
            "RunMat:urlwrite:TooManyOutputs"
        );
        assert_eq!(
            id(vec![chr("not a url"), dst.clone()], 1),
            "RunMat:urlwrite:InvalidUrl"
        );
        // Destination is checked before the source.
        let bad = s.dst.with_file_name("nope").join("x.txt");
        assert_eq!(
            id(vec![chr(&file_url(&s.missing)), chr(&path_text(&bad))], 2),
            "RunMat:urlwrite:InvalidOutputLocation"
        );
        // A string-scalar URL and file name are accepted.
        let out = call_n(
            vec![
                Value::String(file_url(&s.src)),
                Value::String(path_text(&s.dst)),
            ],
            1,
        )
        .expect("string inputs");
        assert_char(&out[0], &path_text(&s.dst));
    }

    // FR-064-25: a destination that is a folder cannot be written.
    #[test]
    fn documented_choice_destination_is_folder() {
        let s = setup();
        let folder = s.dst.with_file_name("sub");
        std::fs::create_dir(&folder).unwrap();
        let err = call_n(vec![chr(&file_url(&s.src)), chr(&path_text(&folder))], 2)
            .expect_err("folder destination");
        assert_eq!(identifier(err), "RunMat:urlwrite:InvalidOutputLocation");
    }
}
