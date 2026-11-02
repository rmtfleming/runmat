//! Clean-room `web` (direct a page to a browser).
//!
//! Clean-room feature `specs/068-web/`. Normative behaviour comes only from
//! the approved export `web` 0.2.0 (pin `e364206`), which was observed with
//! local `file://` pages only: the accepted forms and options, `stat = 0`
//! (double), a non-empty handle for the MATLAB-browser forms, an empty double
//! handle with `-browser`, no output with `nargout = 0`, and the numeric-URL
//! error identifier (raised as `RunMat:*`, D-068-1).
//!
//! Display, network URLs, headless behaviour and the handle object are not
//! specified (`web.q-display`, `web.q-network`, `web.q-headless`,
//! `web.q-handle`). RunMat's choices (D-068-2..5): RunMat has no embedded
//! browser, so every form hands the request to a launcher. The launcher is
//! injectable ([`set_web_launcher`]); the default opens the system browser
//! with the platform opener when a display is available and does nothing
//! otherwise. Unit tests never use the default: under `cfg(test)` it is a
//! no-op, and the tests install a recording launcher.

use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{CharArray, StructValue, Tensor, Value};

use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const NAME: &str = "web";

/// D-068-4: the `Type` field of the handle structure (documented choice).
pub const WEB_HANDLE_TYPE: &str = "runmat.web.Browser";

// ---------------------------------------------------------------------------
// Launcher hook
// ---------------------------------------------------------------------------

/// Where the page was directed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebTarget {
    /// The forms that address the MATLAB web browser (no `-browser`).
    RunMatBrowser,
    /// `-browser`: the system browser.
    SystemBrowser,
}

/// One `web` request, as handed to the launcher.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebRequest {
    /// The URL; a plain path is turned into a `file://` URL. Empty for `web()`.
    pub url: String,
    pub target: WebTarget,
    pub new_window: bool,
    pub no_toolbar: bool,
    pub no_address_box: bool,
    /// Unrecognised options, accepted and ignored (`web.unknown-option`).
    pub unrecognised: Vec<String>,
}

/// A launcher returns `Err` when the page could not be shown; `web` still
/// returns `stat = 0` (D-068-3).
pub type WebLauncher = Arc<dyn Fn(&WebRequest) -> Result<(), String> + Send + Sync>;

fn launcher_slot() -> &'static Mutex<Option<WebLauncher>> {
    static SLOT: OnceLock<Mutex<Option<WebLauncher>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

/// Install a launcher (or `None` for the default); returns the previous one.
pub fn set_web_launcher(launcher: Option<WebLauncher>) -> Option<WebLauncher> {
    let mut slot = launcher_slot()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    std::mem::replace(&mut *slot, launcher)
}

fn launch(request: &WebRequest) -> Result<(), String> {
    let installed = launcher_slot()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    match installed {
        Some(launcher) => launcher(request),
        None => default_launch(request),
    }
}

/// Unit tests never open a browser.
#[cfg(test)]
fn default_launch(_request: &WebRequest) -> Result<(), String> {
    Ok(())
}

/// D-068-2: the system browser through the platform opener, when a display
/// is available; nothing for `web()` or without a display (headless).
#[cfg(all(not(test), not(target_arch = "wasm32")))]
fn default_launch(request: &WebRequest) -> Result<(), String> {
    if request.url.is_empty() || headless() {
        return Ok(());
    }
    let mut command = if cfg!(target_os = "macos") {
        let mut c = std::process::Command::new("open");
        c.arg(&request.url);
        c
    } else if cfg!(target_os = "windows") {
        let mut c = std::process::Command::new("cmd");
        c.args(["/C", "start", ""]).arg(&request.url);
        c
    } else {
        let mut c = std::process::Command::new("xdg-open");
        c.arg(&request.url);
        c
    };
    command
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|err| err.to_string())
}

#[cfg(all(not(test), target_arch = "wasm32"))]
fn default_launch(_request: &WebRequest) -> Result<(), String> {
    Ok(())
}

/// D-068-2: no display server on X11/Wayland systems.
#[cfg(all(not(test), not(target_arch = "wasm32")))]
fn headless() -> bool {
    if cfg!(any(target_os = "macos", target_os = "windows")) {
        return false;
    }
    let set = |name: &str| std::env::var_os(name).is_some_and(|v| !v.is_empty());
    !(set("DISPLAY") || set("WAYLAND_DISPLAY"))
}

// ---------------------------------------------------------------------------
// Descriptor
// ---------------------------------------------------------------------------

const fn param(
    name: &'static str,
    ty: BuiltinParamType,
    arity: BuiltinParamArity,
    description: &'static str,
) -> BuiltinParamDescriptor {
    BuiltinParamDescriptor {
        name,
        ty,
        arity,
        default: None,
        description,
    }
}

const IN_URL: BuiltinParamDescriptor = param(
    "url",
    BuiltinParamType::StringScalar,
    BuiltinParamArity::Optional,
    "URL or file path (char or string scalar).",
);
const IN_OPTIONS: BuiltinParamDescriptor = param(
    "options",
    BuiltinParamType::StringScalar,
    BuiltinParamArity::Variadic,
    "'-browser', '-new', '-notoolbar', '-noaddressbox'.",
);
const OUT_STAT: BuiltinParamDescriptor = param(
    "stat",
    BuiltinParamType::Any,
    BuiltinParamArity::Optional,
    "Status, 0 (double).",
);
const OUT_H: BuiltinParamDescriptor = param(
    "h",
    BuiltinParamType::Any,
    BuiltinParamArity::Optional,
    "Handle structure; [] with '-browser'.",
);

const INPUTS_URL_OPTIONS: [BuiltinParamDescriptor; 2] = [IN_URL, IN_OPTIONS];
const INPUTS_NONE: [BuiltinParamDescriptor; 0] = [];
const OUTPUTS_NONE: [BuiltinParamDescriptor; 0] = [];
const OUTPUTS_STAT: [BuiltinParamDescriptor; 1] = [OUT_STAT];
const OUTPUTS_STAT_H: [BuiltinParamDescriptor; 2] = [OUT_STAT, OUT_H];

const WEB_SIGNATURES: [BuiltinSignatureDescriptor; 4] = [
    BuiltinSignatureDescriptor {
        label: "web(url, '-browser')",
        inputs: &INPUTS_URL_OPTIONS,
        outputs: &OUTPUTS_NONE,
    },
    BuiltinSignatureDescriptor {
        label: "[stat, h] = web(url, '-new', '-noaddressbox', '-notoolbar')",
        inputs: &INPUTS_URL_OPTIONS,
        outputs: &OUTPUTS_STAT_H,
    },
    BuiltinSignatureDescriptor {
        label: "stat = web(url, ...)",
        inputs: &INPUTS_URL_OPTIONS,
        outputs: &OUTPUTS_STAT,
    },
    BuiltinSignatureDescriptor {
        label: "[stat, h] = web()",
        inputs: &INPUTS_NONE,
        outputs: &OUTPUTS_STAT_H,
    },
];

const NOT_TEXT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.WEB.NOT_TEXT",
    identifier: Some("RunMat:string:MustBeCharCellArrayOrString"),
    when: "The URL or an option is not text (e.g. numeric).",
    message: "web: the URL and options must be character vectors or string scalars",
};

const UNSUPPORTED_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.WEB.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:web:UnsupportedOption"),
    when: "Unobserved text forms: cell arrays, multi-row char, string arrays.",
    message: "web: this form is not yet specified",
};

const TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.WEB.TOO_MANY_OUTPUTS",
    identifier: Some("RunMat:web:TooManyOutputs"),
    when: "More than two outputs were requested.",
    message: "web: too many output arguments",
};

const INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.WEB.INTERNAL",
    identifier: Some("RunMat:web:Internal"),
    when: "The runtime cannot assemble the result.",
    message: "web: internal runtime failure",
};

const WEB_ERRORS: [BuiltinErrorDescriptor; 4] =
    [NOT_TEXT, UNSUPPORTED_OPTION, TOO_MANY_OUTPUTS, INTERNAL];

pub const WEB_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &WEB_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &WEB_ERRORS,
};

fn web_error(
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
    name = "web",
    category = "io/http",
    summary = "Direct a URL or file to a web browser; returns status 0 and a handle.",
    keywords = "web,browser,url,html,open page,documentation",
    accel = "sink",
    suppress_auto_output = true,
    descriptor(crate::builtins::io::http::web::WEB_DESCRIPTOR),
    builtin_path = "crate::builtins::io::http::web"
)]
pub(crate) async fn web_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    let out_count = crate::output_count::current_output_count();
    // FR-068-11
    if out_count.is_some_and(|count| count > 2) {
        return Err(web_error(
            &TOO_MANY_OUTPUTS,
            "at most two outputs ([stat, h]) are supported",
        ));
    }
    let mut texts = Vec::with_capacity(args.len());
    for arg in &args {
        let value = gather_if_needed_async(arg).await?;
        // FR-068-06 (web.error-numeric); FR-068-12 for options and other forms.
        texts.push(text_arg(&value)?);
    }
    let request = build_request(&texts);
    // FR-068-07 (web.missing-file), FR-068-10 (D-068-3): the status does not
    // depend on whether the page could be shown.
    let _ = launch(&request);
    let stat = Value::Num(0.0);
    match out_count {
        None => Ok(stat),
        Some(0) => Ok(Value::OutputList(Vec::new())),
        Some(1) => Ok(Value::OutputList(vec![stat])),
        Some(_) => Ok(Value::OutputList(vec![stat, handle(&request)?])),
    }
}

/// FR-068-03, FR-068-06, FR-068-12.
fn text_arg(value: &Value) -> BuiltinResult<String> {
    match value {
        Value::String(s) => Ok(s.clone()),
        Value::StringArray(sa) if sa.data.len() == 1 => Ok(sa.data[0].clone()),
        Value::CharArray(chars) if chars.data.is_empty() => Ok(String::new()),
        Value::CharArray(chars) if chars.rows == 1 => Ok(chars.data.iter().collect()),
        Value::CharArray(_) | Value::StringArray(_) | Value::Cell(_) => Err(web_error(
            &UNSUPPORTED_OPTION,
            "the URL and options must be single-row text",
        )),
        _ => Err(web_error(
            &NOT_TEXT,
            "the URL and options must be character vectors or string scalars",
        )),
    }
}

/// FR-068-01..05, FR-068-08.
fn build_request(texts: &[String]) -> WebRequest {
    let url = texts.first().map(|t| normalise_url(t)).unwrap_or_default();
    let mut request = WebRequest {
        url,
        target: WebTarget::RunMatBrowser,
        new_window: false,
        no_toolbar: false,
        no_address_box: false,
        unrecognised: Vec::new(),
    };
    for option in texts.iter().skip(1) {
        match option.to_ascii_lowercase().as_str() {
            "-browser" => request.target = WebTarget::SystemBrowser,
            "-new" => request.new_window = true,
            "-notoolbar" => request.no_toolbar = true,
            "-noaddressbox" => request.no_address_box = true,
            // FR-068-08 (web.unknown-option)
            _ => request.unrecognised.push(option.clone()),
        }
    }
    request
}

/// FR-068-14 / D-068-5: text with a URL scheme is used as is; a plain path
/// becomes an absolute `file://` URL (relative to the current folder).
fn normalise_url(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if let Ok(parsed) = url::Url::parse(trimmed) {
        // A one-letter scheme is a Windows drive letter, not a URL.
        if parsed.scheme().len() > 1 {
            return trimmed.to_string();
        }
    }
    let path = Path::new(trimmed);
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        match std::env::current_dir() {
            Ok(cwd) => cwd.join(path),
            Err(_) => return trimmed.to_string(),
        }
    };
    url::Url::from_file_path(&absolute)
        .map(|u| u.to_string())
        .unwrap_or_else(|_| trimmed.to_string())
}

/// FR-068-02, FR-068-04, FR-068-09; D-068-4 (documented deviation for `web.q-handle`):
/// a 1x1 structure for the RunMat-browser forms, `[]` with `-browser`.
fn handle(request: &WebRequest) -> BuiltinResult<Value> {
    if request.target == WebTarget::SystemBrowser {
        return Tensor::new(Vec::new(), vec![0, 0])
            .map(Value::Tensor)
            .map_err(|err| web_error(&INTERNAL, err));
    }
    let mut fields = StructValue::new();
    fields.insert(
        "Type",
        Value::CharArray(CharArray::new_row(WEB_HANDLE_TYPE)),
    );
    fields.insert("URL", Value::CharArray(CharArray::new_row(&request.url)));
    Ok(Value::Struct(fields))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod web_tests {
    use super::{set_web_launcher, web_builtin, WebRequest, WebTarget, WEB_HANDLE_TYPE};
    use crate::BuiltinResult;
    use futures::executor::block_on;
    use runmat_value::{CellArray, CharArray, StringArray, Value};
    use std::sync::{Arc, Mutex, MutexGuard};

    /// Serialises the tests that replace the global launcher.
    fn lock() -> MutexGuard<'static, ()> {
        static LOCK: Mutex<()> = Mutex::new(());
        LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Installs a recording launcher for the test's lifetime; nothing is
    /// ever launched and no network is contacted.
    struct Recorder {
        requests: Arc<Mutex<Vec<WebRequest>>>,
        _guard: MutexGuard<'static, ()>,
    }

    impl Recorder {
        fn new(result: Result<(), String>) -> Self {
            let guard = lock();
            let requests: Arc<Mutex<Vec<WebRequest>>> = Arc::default();
            let sink = Arc::clone(&requests);
            set_web_launcher(Some(Arc::new(move |request: &WebRequest| {
                sink.lock().unwrap().push(request.clone());
                result.clone()
            })));
            Recorder {
                requests,
                _guard: guard,
            }
        }

        fn take(&self) -> Vec<WebRequest> {
            std::mem::take(&mut *self.requests.lock().unwrap())
        }
    }

    impl Drop for Recorder {
        fn drop(&mut self) {
            set_web_launcher(None);
        }
    }

    fn text(s: &str) -> Value {
        Value::CharArray(CharArray::new_row(s))
    }

    fn run(args: Vec<Value>, nargout: Option<usize>) -> BuiltinResult<Value> {
        let _guard = crate::output_count::push_output_count(nargout);
        block_on(web_builtin(args))
    }

    fn outputs(args: Vec<Value>, nargout: usize) -> Vec<Value> {
        match run(args, Some(nargout)).unwrap() {
            Value::OutputList(outs) => {
                assert_eq!(outs.len(), nargout);
                outs
            }
            other => panic!("expected {nargout} outputs, got {other:?}"),
        }
    }

    fn assert_stat_zero(value: &Value) {
        assert!(
            matches!(value, Value::Num(v) if *v == 0.0),
            "stat = {value:?}"
        );
    }

    /// A non-empty RunMat handle structure (D-068-4).
    fn assert_handle(value: &Value, url: &str) {
        match value {
            Value::Struct(s) => {
                let field = |name: &str| match s.fields.get(name) {
                    Some(Value::CharArray(c)) => c.data.iter().collect::<String>(),
                    other => panic!("expected char {name}, got {other:?}"),
                };
                assert_eq!(field("Type"), WEB_HANDLE_TYPE);
                assert_eq!(field("URL"), url);
            }
            other => panic!("expected a handle structure, got {other:?}"),
        }
    }

    fn assert_empty_double(value: &Value) {
        match value {
            Value::Tensor(t) => {
                assert!(t.is_empty(), "h = {t:?}");
                assert_eq!(t.numeric_dtype(), runmat_value::NumericDType::F64);
            }
            other => panic!("expected [], got {other:?}"),
        }
    }

    /// A temporary local page, as in the observations.
    fn page() -> (tempfile::TempDir, String) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("page.html");
        std::fs::write(&path, "<html><body>RunMat</body></html>").unwrap();
        let url = url::Url::from_file_path(&path).unwrap().to_string();
        (dir, url)
    }

    // --- Observed (normative): one test per observed case --------------------

    /// case `plain` — web.matlab-browser
    #[test]
    fn observed_plain() {
        let recorder = Recorder::new(Ok(()));
        let (_dir, url) = page();
        let outs = outputs(vec![text(&url)], 2);
        assert_stat_zero(&outs[0]);
        assert_handle(&outs[1], &url);
        let requests = recorder.take();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].target, WebTarget::RunMatBrowser);
        assert_eq!(requests[0].url, url);
    }

    /// case `cobra-new` — web.matlab-browser
    #[test]
    fn observed_cobra_new() {
        let recorder = Recorder::new(Ok(()));
        let (_dir, url) = page();
        let args = vec![
            text(&url),
            text("-new"),
            text("-noaddressbox"),
            text("-notoolbar"),
        ];
        let outs = outputs(args, 2);
        assert_stat_zero(&outs[0]);
        assert_handle(&outs[1], &url);
        let request = &recorder.take()[0];
        assert!(request.new_window && request.no_address_box && request.no_toolbar);
        assert_eq!(request.target, WebTarget::RunMatBrowser);
    }

    /// case `new` — web.matlab-browser
    #[test]
    fn observed_new() {
        let recorder = Recorder::new(Ok(()));
        let (_dir, url) = page();
        let outs = outputs(vec![text(&url), text("-new")], 2);
        assert_stat_zero(&outs[0]);
        assert_handle(&outs[1], &url);
        assert!(recorder.take()[0].new_window);
    }

    /// case `notoolbar` — web.matlab-browser
    #[test]
    fn observed_notoolbar() {
        let recorder = Recorder::new(Ok(()));
        let (_dir, url) = page();
        let outs = outputs(vec![text(&url), text("-notoolbar")], 2);
        assert_stat_zero(&outs[0]);
        assert_handle(&outs[1], &url);
        assert!(recorder.take()[0].no_toolbar);
    }

    /// case `one-out` — web.one-out
    #[test]
    fn observed_one_out() {
        let _recorder = Recorder::new(Ok(()));
        let (_dir, url) = page();
        let outs = outputs(vec![text(&url), text("-new")], 1);
        assert_stat_zero(&outs[0]);
    }

    /// case `string-url` — web.inputs
    #[test]
    fn observed_string_url() {
        let recorder = Recorder::new(Ok(()));
        let (_dir, url) = page();
        let outs = outputs(vec![Value::String(url.clone()), text("-new")], 2);
        assert_stat_zero(&outs[0]);
        assert_handle(&outs[1], &url);
        assert_eq!(recorder.take()[0].url, url);
        // A 1x1 string array is the same string scalar.
        let scalar = StringArray::new(vec![url.clone()], vec![1, 1]).unwrap();
        let outs = outputs(vec![Value::StringArray(scalar)], 2);
        assert_stat_zero(&outs[0]);
    }

    /// case `plain-path` — web.inputs (FR-068-14, D-068-5: the path becomes a
    /// file:// URL)
    #[test]
    fn observed_plain_path() {
        let recorder = Recorder::new(Ok(()));
        let (dir, url) = page();
        let path = dir.path().join("page.html");
        let outs = outputs(vec![text(path.to_str().unwrap()), text("-new")], 2);
        assert_stat_zero(&outs[0]);
        assert_handle(&outs[1], &url);
        assert_eq!(recorder.take()[0].url, url);
    }

    /// case `missing-file` — web.missing-file (non-normative, observed)
    #[test]
    fn observed_missing_file() {
        let recorder = Recorder::new(Ok(()));
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("missing.html");
        let url = url::Url::from_file_path(&missing).unwrap().to_string();
        let outs = outputs(vec![text(&url), text("-new")], 2);
        assert_stat_zero(&outs[0]);
        assert_handle(&outs[1], &url);
        assert_eq!(recorder.take().len(), 1);
    }

    /// case `no-input` — web.inputs
    #[test]
    fn observed_no_input() {
        let recorder = Recorder::new(Ok(()));
        let outs = outputs(vec![], 2);
        assert_stat_zero(&outs[0]);
        assert_handle(&outs[1], "");
        assert_eq!(recorder.take()[0].url, "");
    }

    /// case `browser-cobra` — web.signature-primary, web.system-browser
    #[test]
    fn observed_browser_cobra() {
        let recorder = Recorder::new(Ok(()));
        let (_dir, url) = page();
        let outs = outputs(vec![text(&url), text("-browser")], 0);
        assert!(outs.is_empty());
        let requests = recorder.take();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].target, WebTarget::SystemBrowser);
        assert_eq!(requests[0].url, url);
    }

    /// case `browser-stat` — web.system-browser
    #[test]
    fn observed_browser_stat() {
        let _recorder = Recorder::new(Ok(()));
        let (_dir, url) = page();
        let outs = outputs(vec![text(&url), text("-browser")], 1);
        assert_stat_zero(&outs[0]);
    }

    /// case `browser-two-out` — web.system-browser
    #[test]
    fn observed_browser_two_out() {
        let _recorder = Recorder::new(Ok(()));
        let (_dir, url) = page();
        let outs = outputs(vec![text(&url), text("-browser")], 2);
        assert_stat_zero(&outs[0]);
        assert_empty_double(&outs[1]);
    }

    /// case `err-numeric` — web.error-numeric
    #[test]
    fn observed_err_numeric() {
        let recorder = Recorder::new(Ok(()));
        let err = run(vec![Value::Num(42.0)], Some(0)).unwrap_err();
        assert_eq!(
            err.identifier(),
            Some("RunMat:string:MustBeCharCellArrayOrString")
        );
        assert!(recorder.take().is_empty());
    }

    /// case `unknown-option` — web.unknown-option (non-normative, observed)
    #[test]
    fn observed_unknown_option() {
        let recorder = Recorder::new(Ok(()));
        let (_dir, url) = page();
        let outs = outputs(vec![text(&url), text("-nosuchoption")], 2);
        assert_stat_zero(&outs[0]);
        assert_handle(&outs[1], &url);
        assert_eq!(recorder.take()[0].unrecognised, vec!["-nosuchoption"]);
    }

    // --- Documented choices --------------------------------------------------

    /// FR-068-10 / D-068-3 (`web.q-stat`): a launcher failure still gives
    /// stat 0; no output count (statement form) also gives 0.
    #[test]
    fn documented_choice_launch_failure_and_statement() {
        let recorder = Recorder::new(Err("no browser".to_string()));
        let (_dir, url) = page();
        let outs = outputs(vec![text(&url), text("-browser")], 1);
        assert_stat_zero(&outs[0]);
        assert_stat_zero(&run(vec![text(&url)], None).unwrap());
        assert_eq!(recorder.take().len(), 2);
    }

    /// FR-068-13 / D-068-2 (`web.q-network`): an http URL is handed to the
    /// launcher unchanged; web itself does not contact the network.
    #[test]
    fn documented_choice_network_url_passed_through() {
        let recorder = Recorder::new(Ok(()));
        let outs = outputs(
            vec![text("https://example.invalid/page"), text("-browser")],
            1,
        );
        assert_stat_zero(&outs[0]);
        assert_eq!(recorder.take()[0].url, "https://example.invalid/page");
    }

    /// FR-068-11, FR-068-12: output count and unobserved text forms.
    #[test]
    fn documented_choice_outputs_and_text_forms() {
        let recorder = Recorder::new(Ok(()));
        let err = run(vec![text("file:///x.html")], Some(3)).unwrap_err();
        assert_eq!(err.identifier(), Some("RunMat:web:TooManyOutputs"));
        let err = run(vec![text("file:///x.html"), Value::Num(1.0)], Some(1)).unwrap_err();
        assert_eq!(
            err.identifier(),
            Some("RunMat:string:MustBeCharCellArrayOrString")
        );
        let cell = CellArray::new(vec![text("file:///x.html")], 1, 1).unwrap();
        let err = run(vec![Value::Cell(cell)], Some(1)).unwrap_err();
        assert_eq!(err.identifier(), Some("RunMat:web:UnsupportedOption"));
        // Options are case-insensitive.
        outputs(vec![text("file:///x.html"), text("-BROWSER")], 1);
        let requests = recorder.take();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].target, WebTarget::SystemBrowser);
    }

    /// FR-068-13 / D-068-2: the default launcher is a no-op under test, so even without a
    /// recorder nothing is opened.
    #[test]
    fn documented_choice_default_launcher_inert_in_tests() {
        let _guard = lock();
        set_web_launcher(None);
        let outs = outputs(vec![text("file:///x.html"), text("-browser")], 1);
        assert_stat_zero(&outs[0]);
    }

    // --- Deviations ------------------------------------------------------------

    /// D-068-4 (`web.q-handle`): RunMat's `close` does not accept the web
    /// handle structure (closing would need a change to the `close`
    /// dispatcher, an upstream subsystem). Observed: close(h) succeeds.
    #[test]
    #[ignore = "D-068-4: close(h) for a web handle needs a close-dispatcher change"]
    fn deviation_close_handle() {
        let _recorder = Recorder::new(Ok(()));
        let outs = outputs(vec![text("file:///x.html")], 2);
        let result = block_on(crate::call_builtin_async("close", &[outs[1].clone()]));
        assert!(result.is_ok(), "{result:?}");
    }

    /// FR-068-15 (unresolved, web.q-warning-coverage): no warning.
    #[test]
    fn unresolved_no_warning() {
        let _recorder = Recorder::new(Ok(()));
        assert!(run(vec![text("file:///x.html")], Some(1)).is_ok());
    }
}
