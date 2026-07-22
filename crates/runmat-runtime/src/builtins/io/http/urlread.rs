//! MATLAB-compatible `urlread` builtin for RunMat.
//!
//! Clean-room feature `012-url-fetch`. Normative behaviour comes only from
//! the approved `urlread` 0.2.0 export (observed case: a `file://` URL whose
//! file contains `hello urlread` returns that text as a 1x13 char row
//! vector). The http/https path is summary-derived: it delegates to the
//! shared transport used by `webread` and always decodes the body as text.
//! Error identifiers, charset handling, scheme coverage and the single
//! output are documented independent choices (see
//! `specs/012-url-fetch/spec.md`).

use std::path::PathBuf;
use std::time::Duration;

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    CharArray, ResolveContext, Type, Value,
};
use runmat_filesystem as fs;
use runmat_macros::runtime_builtin;
use url::Url;

use super::transport::{
    self, decode_body_as_text, header_value, HttpMethod, HttpRequest, HEADER_CONTENT_TYPE,
};
use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ShapeRequirements,
};
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const BUILTIN_NAME: &str = "urlread";
const DEFAULT_TIMEOUT_SECONDS: f64 = 60.0;
const DEFAULT_USER_AGENT: &str = "RunMat urlread/0.0";

const URLREAD_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "s",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Resource contents as a 1-by-N character vector.",
}];
const URLREAD_INPUTS_URL: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "url",
    ty: BuiltinParamType::StringScalar,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "URL text (file://, http:// or https://).",
}];
const URLREAD_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "s = urlread(url)",
    inputs: &URLREAD_INPUTS_URL,
    outputs: &URLREAD_OUTPUT,
}];

const URLREAD_ERROR_INVALID_ARGUMENT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.URLREAD.INVALID_ARGUMENT",
    identifier: Some("RunMat:urlread:InvalidArgument"),
    when: "URL argument is not a character vector or string scalar.",
    message: "urlread: invalid argument",
};
const URLREAD_ERROR_INVALID_URL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.URLREAD.INVALID_URL",
    identifier: Some("RunMat:urlread:InvalidUrl"),
    when: "URL text is empty or cannot be parsed.",
    message: "urlread: invalid URL",
};
const URLREAD_ERROR_UNSUPPORTED_SCHEME: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.URLREAD.UNSUPPORTED_SCHEME",
    identifier: Some("RunMat:urlread:UnsupportedScheme"),
    when: "URL scheme is not file, http, or https.",
    message: "urlread: unsupported URL scheme",
};
const URLREAD_ERROR_FILE_READ: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.URLREAD.FILE_READ",
    identifier: Some("RunMat:urlread:FileRead"),
    when: "A file:// URL cannot be resolved or its file cannot be read.",
    message: "urlread: file read failed",
};
const URLREAD_ERROR_TRANSPORT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.URLREAD.TRANSPORT",
    identifier: Some("RunMat:urlread:Transport"),
    when: "HTTP transport fails (connect, timeout, or HTTP status).",
    message: "urlread: transport failure",
};
const URLREAD_ERROR_FLOW: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.URLREAD.FLOW",
    identifier: Some("RunMat:urlread:Flow"),
    when: "Nested flow fails while gathering inputs.",
    message: "urlread: flow failure",
};

const URLREAD_ERRORS: [BuiltinErrorDescriptor; 6] = [
    URLREAD_ERROR_INVALID_ARGUMENT,
    URLREAD_ERROR_INVALID_URL,
    URLREAD_ERROR_UNSUPPORTED_SCHEME,
    URLREAD_ERROR_FILE_READ,
    URLREAD_ERROR_TRANSPORT,
    URLREAD_ERROR_FLOW,
];

pub const URLREAD_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &URLREAD_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &URLREAD_ERRORS,
};

#[runmat_macros::register_gpu_spec(builtin_path = "crate::builtins::io::http::urlread")]
pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "urlread",
    op_kind: GpuOpKind::Custom("url-fetch"),
    supported_precisions: &[],
    broadcast: BroadcastSemantics::None,
    provider_hooks: &[],
    constant_strategy: ConstantStrategy::InlineLiteral,
    residency: ResidencyPolicy::GatherImmediately,
    nan_mode: ReductionNaN::Include,
    two_pass_threshold: None,
    workgroup_size: None,
    accepts_nan_mode: false,
    notes: "URL fetches always execute on the CPU; gpuArray inputs are gathered eagerly.",
};

#[runmat_macros::register_fusion_spec(builtin_path = "crate::builtins::io::http::urlread")]
pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "urlread",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "urlread performs I/O and terminates fusion graphs.",
};

/// Type resolver: `urlread` always yields text (char row vector).
pub fn urlread_type(args: &[Type], ctx: &ResolveContext) -> Type {
    let _ = (args, ctx);
    Type::String
}

fn urlread_error_with(
    error: &'static BuiltinErrorDescriptor,
    message: impl Into<String>,
) -> RuntimeError {
    let mut builder = build_runtime_error(message).with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

fn urlread_error_with_source<E>(
    error: &'static BuiltinErrorDescriptor,
    message: impl Into<String>,
    source: E,
) -> RuntimeError
where
    E: std::error::Error + Send + Sync + 'static,
{
    let mut builder = build_runtime_error(message)
        .with_builtin(BUILTIN_NAME)
        .with_source(source);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

fn urlread_flow_with_context(err: RuntimeError) -> RuntimeError {
    let message = format!("urlread: {}", err.message());
    let mut builder = build_runtime_error(message)
        .with_builtin(BUILTIN_NAME)
        .with_source(err);
    if let Some(identifier) = URLREAD_ERROR_FLOW.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

/// Scheme routing decision, kept pure so it is unit-testable without I/O.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Route {
    File,
    Http,
    Unsupported,
}

fn route_for(url: &Url) -> Route {
    match url.scheme() {
        "file" => Route::File,
        "http" | "https" => Route::Http,
        _ => Route::Unsupported,
    }
}

fn expect_url_text(value: &Value) -> BuiltinResult<String> {
    match value {
        Value::String(s) => Ok(s.clone()),
        Value::CharArray(ca) if ca.rows == 1 => Ok(ca.data.iter().collect()),
        Value::StringArray(sa) if sa.data.len() == 1 => Ok(sa.data[0].clone()),
        _ => Err(urlread_error_with(
            &URLREAD_ERROR_INVALID_ARGUMENT,
            "urlread: URL must be a character vector or string scalar",
        )),
    }
}

fn char_row_value(text: &str) -> Value {
    Value::CharArray(CharArray::new_row(text))
}

/// Minimal percent-decoding for file URL paths on targets where
/// `Url::to_file_path` is unavailable (wasm32). Invalid escapes pass through
/// unchanged; the result decodes as UTF-8 with lossy fallback.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn percent_decode_path(input: &str) -> String {
    fn hex_val(byte: u8) -> Option<u8> {
        match byte {
            b'0'..=b'9' => Some(byte - b'0'),
            b'a'..=b'f' => Some(byte - b'a' + 10),
            b'A'..=b'F' => Some(byte - b'A' + 10),
            _ => None,
        }
    }
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(high), Some(low)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
                out.push((high << 4) | low);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(not(target_arch = "wasm32"))]
fn file_url_to_path(url: &Url) -> BuiltinResult<PathBuf> {
    url.to_file_path().map_err(|()| {
        urlread_error_with(
            &URLREAD_ERROR_FILE_READ,
            format!("urlread: unable to resolve file URL '{url}' to a local filesystem path"),
        )
    })
}

#[cfg(target_arch = "wasm32")]
fn file_url_to_path(url: &Url) -> BuiltinResult<PathBuf> {
    // `Url::to_file_path` is not available on wasm32-unknown-unknown; map the
    // percent-decoded URL path onto the runmat-filesystem provider path space
    // (documented independent choice, spec 012 Unresolved behaviour).
    Ok(PathBuf::from(percent_decode_path(url.path())))
}

async fn read_file_url(url: &Url) -> BuiltinResult<Value> {
    let path = file_url_to_path(url)?;
    let bytes = fs::read_async(&path).await.map_err(|err| {
        urlread_error_with_source(
            &URLREAD_ERROR_FILE_READ,
            format!("urlread: unable to read '{}': {err}", path.display()),
            err,
        )
    })?;
    // Charset is unobserved in the approved export; documented choice: reuse
    // the transport's BOM-aware text decoding (UTF-8 with lossy fallback).
    let text = decode_body_as_text(&bytes, None);
    Ok(char_row_value(&text))
}

fn fetch_http(url: Url) -> BuiltinResult<Value> {
    let request = HttpRequest {
        url,
        method: HttpMethod::Get,
        headers: Vec::new(),
        body: None,
        timeout: Duration::from_secs_f64(DEFAULT_TIMEOUT_SECONDS),
        user_agent: DEFAULT_USER_AGENT.to_string(),
    };
    let response = transport::send_request(&request).map_err(|err| {
        urlread_error_with_source(
            &URLREAD_ERROR_TRANSPORT,
            err.message_with_prefix("urlread"),
            err,
        )
    })?;
    let content_type =
        header_value(&response.headers, HEADER_CONTENT_TYPE).map(|value| value.to_string());
    // urlread returns text unconditionally (summary-derived); unlike webread
    // there is no JSON or binary branch.
    let text = decode_body_as_text(&response.body, content_type.as_deref());
    Ok(char_row_value(&text))
}

#[runtime_builtin(
    name = "urlread",
    category = "io/http",
    summary = "Fetch the contents at a URL and return them as a character row vector.",
    keywords = "urlread,url,download,fetch,http,file url,text",
    accel = "sink",
    type_resolver(crate::builtins::io::http::urlread::urlread_type),
    descriptor(crate::builtins::io::http::urlread::URLREAD_DESCRIPTOR),
    builtin_path = "crate::builtins::io::http::urlread"
)]
async fn urlread_builtin(url: Value) -> crate::BuiltinResult<Value> {
    let gathered = gather_if_needed_async(&url)
        .await
        .map_err(urlread_flow_with_context)?;
    let url_text = expect_url_text(&gathered)?;
    let trimmed = url_text.trim();
    if trimmed.is_empty() {
        return Err(urlread_error_with(
            &URLREAD_ERROR_INVALID_URL,
            "urlread: URL must not be empty",
        ));
    }
    let parsed = Url::parse(trimmed).map_err(|err| {
        urlread_error_with_source(
            &URLREAD_ERROR_INVALID_URL,
            format!("urlread: invalid URL '{trimmed}': {err}"),
            err,
        )
    })?;
    match route_for(&parsed) {
        Route::File => read_file_url(&parsed).await,
        Route::Http => fetch_http(parsed),
        Route::Unsupported => Err(urlread_error_with(
            &URLREAD_ERROR_UNSUPPORTED_SCHEME,
            format!(
                "urlread: unsupported URL scheme '{}'; supported schemes are file, http, https",
                parsed.scheme()
            ),
        )),
    }
}

// SANDBOX/TEST-ISOLATION (feature 012, SC-012-4): every test below is
// network-free. All I/O goes through self-cleaning `file://` temp files; the
// http/https delegation is covered by the shared transport's existing
// loopback tests in `webread.rs`/`webwrite.rs` plus the pure routing tests
// here (which never send a request).
#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::builtins::common::test_support;
    use runmat_time::unix_timestamp_ms;

    fn run_urlread(url: Value) -> BuiltinResult<Value> {
        futures::executor::block_on(urlread_builtin(url))
    }

    fn error_message(err: RuntimeError) -> String {
        err.message().to_string()
    }

    fn unique_path(prefix: &str) -> PathBuf {
        let millis = unix_timestamp_ms();
        let mut path = std::env::temp_dir();
        path.push(format!("runmat_{prefix}_{}_{}", std::process::id(), millis));
        path
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn file_url_for(path: &std::path::Path) -> String {
        Url::from_file_path(path)
            .expect("absolute temp path converts to file URL")
            .to_string()
    }

    #[cfg(target_arch = "wasm32")]
    fn file_url_for(path: &std::path::Path) -> String {
        format!("file://{}", path.to_string_lossy())
    }

    // FR-012-01; SC-012-2.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn urlread_registered_with_single_signature() {
        assert!(runmat_builtins::builtin_function_by_name("urlread").is_some());
        let labels: Vec<&str> = URLREAD_DESCRIPTOR
            .signatures
            .iter()
            .map(|sig| sig.label)
            .collect();
        assert_eq!(labels, vec!["s = urlread(url)"]);
    }

    // Normative observed case `file-url` (claims urlread.signature-primary,
    // urlread.returns-char). [FR-012-01, FR-012-02; SC-012-1]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn urlread_file_url_returns_observed_char_row() {
        let path = unique_path("urlread_observed");
        test_support::fs::write(&path, "hello urlread").expect("write temp file");
        let url = file_url_for(&path);

        let result = run_urlread(Value::CharArray(CharArray::new_row(&url)));
        let _ = test_support::fs::remove_file(&path);

        match result.expect("urlread on file URL") {
            Value::CharArray(ca) => {
                assert_eq!(ca.rows, 1, "result must be a char row vector");
                assert_eq!(ca.cols, 13, "observed size is 1x13");
                let text: String = ca.data.iter().collect();
                assert_eq!(text, "hello urlread");
            }
            other => panic!("expected char row vector, got {other:?}"),
        }
    }

    // unresolved_choice: empty file yields an empty char row. [FR-012-04]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn urlread_empty_file_returns_empty_char_row_unresolved_choice() {
        let path = unique_path("urlread_empty");
        test_support::fs::write(&path, "").expect("write empty temp file");
        let url = file_url_for(&path);

        let result = run_urlread(Value::from(url));
        let _ = test_support::fs::remove_file(&path);

        match result.expect("urlread on empty file URL") {
            Value::CharArray(ca) => {
                assert_eq!(ca.rows, 1);
                assert_eq!(ca.cols, 0);
            }
            other => panic!("expected empty char row, got {other:?}"),
        }
    }

    // unresolved_choice: percent-encoded file URLs resolve to local paths.
    // [FR-012-04]
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn urlread_file_url_with_space_resolves_percent_encoding_unresolved_choice() {
        let path = unique_path("urlread space");
        test_support::fs::write(&path, "spaced").expect("write temp file with space");
        let url = file_url_for(&path);
        assert!(
            url.contains("%20"),
            "expected percent-encoded space in {url}"
        );

        let result = run_urlread(Value::from(url));
        let _ = test_support::fs::remove_file(&path);

        match result.expect("urlread on percent-encoded file URL") {
            Value::CharArray(ca) => {
                let text: String = ca.data.iter().collect();
                assert_eq!(text, "spaced");
            }
            other => panic!("expected char row vector, got {other:?}"),
        }
    }

    // unresolved_choice: missing file errors with the FileRead choice.
    // [FR-012-04]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn urlread_missing_file_errors_unresolved_choice() {
        let path = unique_path("urlread_missing");
        let url = file_url_for(&path);
        let err = run_urlread(Value::from(url)).expect_err("missing file must error");
        let message = error_message(err);
        assert!(
            message.contains("unable to read"),
            "unexpected message: {message}"
        );
    }

    // unresolved_choice: empty URL text is rejected. [FR-012-04]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn urlread_rejects_empty_url_unresolved_choice() {
        let err = run_urlread(Value::from(String::new())).expect_err("empty URL must error");
        let message = error_message(err);
        assert!(
            message.contains("URL must not be empty"),
            "unexpected message: {message}"
        );
    }

    // unresolved_choice: non-text URL argument is rejected. [FR-012-04]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn urlread_rejects_non_text_url_argument_unresolved_choice() {
        let err = run_urlread(Value::Num(4.0)).expect_err("numeric URL must error");
        let message = error_message(err);
        assert!(
            message.contains("character vector or string scalar"),
            "unexpected message: {message}"
        );
    }

    // unresolved_choice: unparseable URL text is rejected. [FR-012-04]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn urlread_rejects_unparseable_url_unresolved_choice() {
        let err = run_urlread(Value::from("not a url".to_string()))
            .expect_err("unparseable URL must error");
        let message = error_message(err);
        assert!(
            message.contains("invalid URL"),
            "unexpected message: {message}"
        );
    }

    // unresolved_choice: schemes beyond file/http/https are rejected before
    // any request is attempted. [FR-012-04]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn urlread_rejects_unsupported_scheme_unresolved_choice() {
        let err = run_urlread(Value::from("ftp://example.invalid/resource".to_string()))
            .expect_err("ftp scheme must error");
        let message = error_message(err);
        assert!(
            message.contains("unsupported URL scheme 'ftp'"),
            "unexpected message: {message}"
        );
    }

    // summary_derived: routing sends http/https to the transport route and
    // file to the filesystem route. Pure classification — no request is
    // constructed or sent. [FR-012-03]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn urlread_route_for_maps_schemes_summary_derived() {
        let http = Url::parse("http://example.invalid/x").expect("parse http URL");
        let https = Url::parse("https://example.invalid/x").expect("parse https URL");
        let file = Url::parse("file:///tmp/x").expect("parse file URL");
        let ftp = Url::parse("ftp://example.invalid/x").expect("parse ftp URL");
        assert_eq!(route_for(&http), Route::Http);
        assert_eq!(route_for(&https), Route::Http);
        assert_eq!(route_for(&file), Route::File);
        assert_eq!(route_for(&ftp), Route::Unsupported);
    }

    // Helper coverage for the wasm file-path fallback (pure, no I/O).
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn urlread_percent_decode_path_handles_escapes() {
        assert_eq!(percent_decode_path("/tmp/a%20b"), "/tmp/a b");
        assert_eq!(percent_decode_path("/plain/path"), "/plain/path");
        assert_eq!(percent_decode_path("/bad%2xescape"), "/bad%2xescape");
        assert_eq!(percent_decode_path("/trailing%2"), "/trailing%2");
    }
}
