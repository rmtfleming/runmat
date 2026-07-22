//! MATLAB-compatible `saveas` builtin for saving figures to image files.
//!
//! Clean-room feature 014-figure-export. Normative behaviour (approved
//! export, claims `saveas.signature-primary` and `saveas.creates-file`):
//! `saveas(fig, filename)` creates the target file with a nonzero byte
//! size and produces no visible output. Format selection, the
//! three-argument form and error cases are documented independent choices
//! (unresolved `saveas.q-formats`); they are not asserted as
//! MATLAB-conformant. The builtin is a thin adapter over the existing
//! `print` export machinery (`render_png` + `write_bytes`); it introduces
//! no new rendering.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ShapeRequirements,
};
use crate::builtins::plotting::type_resolvers::bool_type;
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

use super::op_common::handles::handle_from_scalar;
use super::print::{render_png, write_bytes};
use super::state::FigureHandle;

#[runmat_macros::register_gpu_spec(builtin_path = "crate::builtins::plotting::saveas")]
pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "saveas",
    op_kind: GpuOpKind::Custom("figure-export"),
    supported_precisions: &[],
    broadcast: BroadcastSemantics::None,
    provider_hooks: &[],
    constant_strategy: ConstantStrategy::InlineLiteral,
    residency: ResidencyPolicy::GatherImmediately,
    nan_mode: ReductionNaN::Include,
    two_pass_threshold: None,
    workgroup_size: None,
    accepts_nan_mode: false,
    notes: "Exports a figure through the shared plotting renderer. Handle and text arguments are gathered; figure content may still render through the shared WGPU plotting path.",
};

#[runmat_macros::register_fusion_spec(builtin_path = "crate::builtins::plotting::saveas")]
pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "saveas",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "saveas performs figure export I/O and terminates fusion graphs.",
};

const BUILTIN_NAME: &str = "saveas";
/// Default export dimensions, mirroring `print`'s screen-resolution export.
const DEFAULT_WIDTH: u32 = 800;
const DEFAULT_HEIGHT: u32 = 600;

const SAVEAS_OUTPUT_OK: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "ok",
    ty: BuiltinParamType::LogicalArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Internal status; saveas suppresses automatic display — the observable effect is the written file.",
}];
const SAVEAS_INPUT_FIG: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "fig",
    ty: BuiltinParamType::NumericScalar,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Figure handle (numeric scalar as returned by figure/gcf).",
};
const SAVEAS_INPUT_FILENAME: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "filename",
    ty: BuiltinParamType::StringScalar,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Output file name; '.png' is appended when no extension is given.",
};
const SAVEAS_INPUT_FORMATTYPE: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "formattype",
    ty: BuiltinParamType::StringScalar,
    arity: BuiltinParamArity::Optional,
    default: None,
    description: "Optional explicit format; RunMat currently supports 'png'.",
};
const SAVEAS_INPUTS_TWO: [BuiltinParamDescriptor; 2] = [SAVEAS_INPUT_FIG, SAVEAS_INPUT_FILENAME];
const SAVEAS_INPUTS_THREE: [BuiltinParamDescriptor; 3] = [
    SAVEAS_INPUT_FIG,
    SAVEAS_INPUT_FILENAME,
    SAVEAS_INPUT_FORMATTYPE,
];
const SAVEAS_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "saveas(fig, filename)",
        inputs: &SAVEAS_INPUTS_TWO,
        outputs: &SAVEAS_OUTPUT_OK,
    },
    BuiltinSignatureDescriptor {
        label: "saveas(fig, filename, formattype)",
        inputs: &SAVEAS_INPUTS_THREE,
        outputs: &SAVEAS_OUTPUT_OK,
    },
];

const SAVEAS_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SAVEAS.INVALID_INPUT",
    identifier: Some("RunMat:saveas:InvalidInput"),
    when: "Arguments are missing, malformed, or cannot be interpreted as a figure handle plus filename.",
    message: "saveas: invalid input arguments",
};
const SAVEAS_ERROR_UNSUPPORTED_FORMAT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SAVEAS.UNSUPPORTED_FORMAT",
    identifier: Some("RunMat:saveas:UnsupportedFormat"),
    when: "The requested output format or file extension is not supported by the active exporter.",
    message: "saveas: unsupported output format",
};
const SAVEAS_ERROR_RENDER: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SAVEAS.RENDER",
    identifier: Some("RunMat:saveas:RenderFailed"),
    when: "The figure renderer fails while serializing the figure (including builds without plot-core, and handles that name no live figure).",
    message: "saveas: figure export failed",
};
const SAVEAS_ERROR_IO: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SAVEAS.IO",
    identifier: Some("RunMat:saveas:IoFailure"),
    when: "The exported bytes cannot be written to the target file.",
    message: "saveas: file I/O failed",
};
const SAVEAS_ERROR_INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SAVEAS.INTERNAL",
    identifier: None,
    when: "Internal runtime control-flow or conversion fails.",
    message: "saveas: internal error",
};
const SAVEAS_ERRORS: [BuiltinErrorDescriptor; 5] = [
    SAVEAS_ERROR_INVALID_INPUT,
    SAVEAS_ERROR_UNSUPPORTED_FORMAT,
    SAVEAS_ERROR_RENDER,
    SAVEAS_ERROR_IO,
    SAVEAS_ERROR_INTERNAL,
];

pub const SAVEAS_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &SAVEAS_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &SAVEAS_ERRORS,
};

#[runtime_builtin(
    name = "saveas",
    category = "plotting",
    summary = "Save a figure to an image file.",
    keywords = "saveas,plotting,figure export,png,save figure",
    sink = true,
    suppress_auto_output = true,
    accel = "metadata",
    type_resolver(bool_type),
    descriptor(crate::builtins::plotting::saveas::SAVEAS_DESCRIPTOR),
    builtin_path = "crate::builtins::plotting::saveas"
)]
pub async fn saveas_builtin(args: Vec<Value>) -> BuiltinResult<bool> {
    let args = gather_values(&args).await?;
    if args.len() < 2 || args.len() > 3 {
        return Err(saveas_error_with_detail(
            &SAVEAS_ERROR_INVALID_INPUT,
            format!(
                "expected saveas(fig, filename) or saveas(fig, filename, formattype), got {} argument(s)",
                args.len()
            ),
        ));
    }

    let figure = figure_handle_value(&args[0])?;
    let filename = text_arg(&args[1], "filename")?;
    let formattype = match args.get(2) {
        Some(value) => Some(text_arg(value, "formattype")?),
        None => None,
    };

    if filename.trim().is_empty() {
        return Err(saveas_error_with_detail(
            &SAVEAS_ERROR_INVALID_INPUT,
            "filename must be nonempty",
        ));
    }
    if filename.contains('\0') {
        return Err(saveas_error_with_detail(
            &SAVEAS_ERROR_INVALID_INPUT,
            "filename must not contain NUL bytes",
        ));
    }

    let path = resolve_output_path(&filename, formattype.as_deref())?;
    let bytes = render_figure(figure).await?;
    write_output(&path, &bytes).await?;
    Ok(true)
}

async fn gather_values(values: &[Value]) -> BuiltinResult<Vec<Value>> {
    let mut out = Vec::with_capacity(values.len());
    for value in values {
        out.push(
            gather_if_needed_async(value)
                .await
                .map_err(map_control_flow)?,
        );
    }
    Ok(out)
}

/// The approved invocation form requires an explicit figure handle
/// (`saveas(fig, filename)`); there is no implicit current-figure form.
fn figure_handle_value(value: &Value) -> BuiltinResult<FigureHandle> {
    match value {
        Value::Num(v) => handle_from_scalar(*v, BUILTIN_NAME),
        Value::Int(i) => handle_from_scalar(i.to_f64(), BUILTIN_NAME),
        Value::Tensor(tensor) if tensor.data.len() == 1 => {
            handle_from_scalar(tensor.data[0], BUILTIN_NAME)
        }
        other => Err(saveas_error_with_detail(
            &SAVEAS_ERROR_INVALID_INPUT,
            format!(
                "expected a figure handle (numeric scalar) as the first argument, got {other:?}"
            ),
        )),
    }
}

fn text_arg(value: &Value, what: &str) -> BuiltinResult<String> {
    match value {
        Value::String(s) => Ok(s.clone()),
        Value::CharArray(ca) if ca.rows == 1 => Ok(ca.data.iter().collect()),
        Value::CharArray(_) => Err(saveas_error_with_detail(
            &SAVEAS_ERROR_INVALID_INPUT,
            format!("{what} must be a 1-by-N character vector or string scalar"),
        )),
        Value::StringArray(sa) if sa.data.len() == 1 => Ok(sa.data[0].clone()),
        Value::StringArray(_) => Err(saveas_error_with_detail(
            &SAVEAS_ERROR_INVALID_INPUT,
            format!("{what} must be a scalar string"),
        )),
        other => Err(saveas_error_with_detail(
            &SAVEAS_ERROR_INVALID_INPUT,
            format!("expected text {what}, got {other:?}"),
        )),
    }
}

/// Documented independent choice (unresolved `saveas.q-formats`): RunMat
/// currently writes PNG only. A `.png` extension (case-insensitive) is
/// kept; a missing extension gets `.png` appended to the final path
/// component; any other extension or format type is rejected. Not
/// asserted as MATLAB-conformant.
fn resolve_output_path(filename: &str, formattype: Option<&str>) -> BuiltinResult<PathBuf> {
    if let Some(format) = formattype {
        let trimmed = format.trim();
        if !trimmed.eq_ignore_ascii_case("png") {
            return Err(saveas_error_with_detail(
                &SAVEAS_ERROR_UNSUPPORTED_FORMAT,
                format!("format '{trimmed}' is not available yet; RunMat currently supports 'png'"),
            ));
        }
    }

    let path = Path::new(filename);
    match path.extension() {
        Some(ext)
            if ext
                .to_str()
                .map(|ext| ext.eq_ignore_ascii_case("png"))
                .unwrap_or(false) =>
        {
            Ok(path.to_path_buf())
        }
        Some(ext) => Err(saveas_error_with_detail(
            &SAVEAS_ERROR_UNSUPPORTED_FORMAT,
            format!(
                "output extension '.{}' is not available yet; RunMat currently writes PNG ('.png')",
                ext.to_string_lossy()
            ),
        )),
        None => {
            let Some(file_name) = path.file_name() else {
                return Err(saveas_error_with_detail(
                    &SAVEAS_ERROR_INVALID_INPUT,
                    "filename must include a file name before appending '.png'",
                ));
            };
            let mut file_name = OsString::from(file_name);
            file_name.push(".png");
            Ok(path.with_file_name(file_name))
        }
    }
}

/// Renders through the shared `print` export path (`render_figure_snapshot`
/// under `plot-core`; the same "plot-core support is not enabled in this
/// build" failure otherwise). Errors are re-attributed to `saveas`.
async fn render_figure(handle: FigureHandle) -> BuiltinResult<Vec<u8>> {
    render_png(handle, DEFAULT_WIDTH, DEFAULT_HEIGHT)
        .await
        .map_err(|err| {
            saveas_error_with_source(
                format!("{}: {}", SAVEAS_ERROR_RENDER.message, err.message()),
                &SAVEAS_ERROR_RENDER,
                err,
            )
        })
}

/// Writes through the shared atomic temp-file-then-rename path used by
/// `print`. Errors are re-attributed to `saveas`.
async fn write_output(path: &Path, payload: &[u8]) -> BuiltinResult<()> {
    write_bytes(path, payload).await.map_err(|err| {
        saveas_error_with_source(
            format!("{}: {}", SAVEAS_ERROR_IO.message, err.message()),
            &SAVEAS_ERROR_IO,
            err,
        )
    })
}

fn saveas_error_with_detail(
    error: &'static BuiltinErrorDescriptor,
    detail: impl AsRef<str>,
) -> RuntimeError {
    saveas_error_with_message(format!("{}: {}", error.message, detail.as_ref()), error)
}

fn saveas_error_with_message(
    message: impl Into<String>,
    error: &'static BuiltinErrorDescriptor,
) -> RuntimeError {
    let mut builder = build_runtime_error(message).with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

fn saveas_error_with_source(
    message: impl Into<String>,
    error: &'static BuiltinErrorDescriptor,
    source: impl std::error::Error + Send + Sync + 'static,
) -> RuntimeError {
    let mut builder = build_runtime_error(message)
        .with_builtin(BUILTIN_NAME)
        .with_source(source);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

fn map_control_flow(err: RuntimeError) -> RuntimeError {
    saveas_error_with_source(
        format!("{}: {}", SAVEAS_ERROR_INTERNAL.message, err.message()),
        &SAVEAS_ERROR_INTERNAL,
        err,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtins::plotting::tests::{ensure_plot_test_env, lock_plot_registry};

    fn setup() -> crate::builtins::plotting::state::PlotTestLockGuard {
        let guard = lock_plot_registry();
        ensure_plot_test_env();
        crate::builtins::plotting::state::reset_hold_state_for_run();
        let _ = crate::builtins::plotting::state::clear_figure(None);
        guard
    }

    fn unique_temp_path(stem: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let thread = std::thread::current();
        let thread_name = thread.name().unwrap_or("thread");
        let safe_thread_name: String = thread_name
            .chars()
            .map(|ch| match ch {
                '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
                ch if ch.is_control() => '_',
                ch => ch,
            })
            .collect();
        path.push(format!(
            "runmat_saveas_test_{}_{}_{}",
            stem,
            std::process::id(),
            safe_thread_name
        ));
        path
    }

    #[cfg(feature = "plot-core")]
    mod rendering {
        use super::*;
        use crate::builtins::plotting::figure::figure_builtin;
        use crate::builtins::plotting::plot::plot_builtin;
        use futures::executor::block_on;
        use runmat_builtins::{NumericDType, Tensor};

        fn tensor(data: &[f64]) -> Tensor {
            Tensor {
                data: data.to_vec(),
                shape: vec![data.len()],
                rows: data.len(),
                cols: 1,
                dtype: NumericDType::F64,
            }
        }

        fn plotted_figure_handle() -> f64 {
            let handle = figure_builtin(vec![]).expect("figure");
            block_on(plot_builtin(vec![
                Value::Tensor(tensor(&[0.0, 1.0, 2.0, 3.0])),
                Value::Tensor(tensor(&[0.0, 1.0, 4.0, 9.0])),
            ]))
            .expect("plot");
            handle
        }

        /// Normative [FR-014-01, FR-014-02; claims saveas.signature-primary,
        /// saveas.creates-file; observed cases file-created, file-nonempty].
        /// The observed byte count (12312) is environment-specific and is
        /// deliberately NOT asserted; the normative claims are existence and
        /// nonzero size. Temp output self-cleans in success and failure paths.
        #[test]
        fn saveas_creates_file_and_is_nonempty() {
            let _guard = setup();
            let handle = plotted_figure_handle();

            let output = unique_temp_path("basic").with_extension("png");
            let _ = std::fs::remove_file(&output);
            let result = block_on(saveas_builtin(vec![
                Value::Num(handle),
                Value::String(output.to_string_lossy().into_owned()),
            ]));
            let metadata = std::fs::metadata(&output);
            let _ = std::fs::remove_file(&output);

            result.expect("saveas(fig, filename) succeeds");
            let metadata = metadata.expect("target file exists after saveas (case file-created)");
            assert!(
                metadata.len() > 0,
                "target file has nonzero byte size (case file-nonempty)"
            );
        }

        /// Normative [FR-014-01]: the figure handle argument is accepted in
        /// RunMat's numeric-scalar handle forms (here: 1-element tensor).
        #[test]
        fn saveas_accepts_numeric_figure_handle() {
            let _guard = setup();
            let handle = plotted_figure_handle();

            let output = unique_temp_path("tensor_handle").with_extension("png");
            let _ = std::fs::remove_file(&output);
            let result = block_on(saveas_builtin(vec![
                Value::Tensor(tensor(&[handle])),
                Value::String(output.to_string_lossy().into_owned()),
            ]));
            let exists = output.exists();
            let _ = std::fs::remove_file(&output);

            result.expect("saveas accepts a 1-element tensor handle");
            assert!(exists, "file created for tensor-form handle");
        }

        /// Documented choice, tagged unresolved (saveas.q-formats)
        /// [FR-014-04]: a '.png' target yields PNG-encoded bytes. This is an
        /// implementation-consistency check, not a MATLAB-conformance claim.
        #[test]
        fn saveas_writes_png_bytes_unresolved_choice() {
            let _guard = setup();
            let handle = plotted_figure_handle();

            let output = unique_temp_path("magic").with_extension("png");
            let _ = std::fs::remove_file(&output);
            let result = block_on(saveas_builtin(vec![
                Value::Num(handle),
                Value::String(output.to_string_lossy().into_owned()),
            ]));
            let bytes = std::fs::read(&output);
            let _ = std::fs::remove_file(&output);

            result.expect("saveas");
            let bytes = bytes.expect("read exported file");
            assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
        }

        /// Documented choice, tagged unresolved (export lists no error cases)
        /// [FR-014-04]: a positive handle that names no live figure errors and
        /// creates no file.
        #[test]
        fn saveas_nonexistent_figure_errors_unresolved_choice() {
            let _guard = setup();
            let output = unique_temp_path("nonexistent").with_extension("png");
            let _ = std::fs::remove_file(&output);
            let err = block_on(saveas_builtin(vec![
                Value::Num(4_000_000.0),
                Value::String(output.to_string_lossy().into_owned()),
            ]))
            .expect_err("handle without a live figure must error");
            let exists = output.exists();
            let _ = std::fs::remove_file(&output);

            assert!(
                err.message().contains("does not exist"),
                "{}",
                err.message()
            );
            assert!(!exists, "no file may be created on error");
        }
    }

    // No `#[cfg(not(feature = "plot-core"))]` test exists: `builtins::plotting`
    // (and therefore `saveas`, like `print`) is only compiled under the
    // `plot-core` feature (`builtins/mod.rs`), so plot-core-off builds have no
    // saveas builtin at all — matching `print`. Recorded as a deviation from
    // the plan's T3 in `specs/014-figure-export/validation.md`.

    /// Normative [FR-014-01, FR-014-03]: descriptor covers the approved
    /// two-argument form (and the documented-choice three-argument form);
    /// the builtin is a sink with suppressed auto display, modelling
    /// "saveas returns no output; the effect is the written file".
    #[test]
    fn saveas_descriptor_covers_approved_signature() {
        let labels: Vec<&str> = SAVEAS_DESCRIPTOR
            .signatures
            .iter()
            .map(|sig| sig.label)
            .collect();
        assert!(labels.contains(&"saveas(fig, filename)"));
        assert!(labels.contains(&"saveas(fig, filename, formattype)"));
        assert!(
            runmat_builtins::builtin_function_by_name("saveas").is_some(),
            "saveas must be registered and discoverable [SC-014-2]"
        );
    }

    /// Documented choice, tagged unresolved (saveas.q-formats) [FR-014-04]:
    /// a missing extension gets '.png' appended to the final path component.
    #[test]
    fn saveas_appends_png_when_extension_missing_unresolved_choice() {
        let path = resolve_output_path("exports/report", None).expect("resolve");
        assert_eq!(path, PathBuf::from("exports/report.png"));
        let path = resolve_output_path("Report.PNG", None).expect("resolve");
        assert_eq!(path, PathBuf::from("Report.PNG"));
    }

    /// Documented choice, tagged unresolved (saveas.q-formats) [FR-014-04]:
    /// non-PNG extensions are rejected with the UnsupportedFormat error.
    #[test]
    fn saveas_rejects_unsupported_extension_unresolved_choice() {
        let err = resolve_output_path("figure.svg", None).expect_err("svg unsupported");
        assert!(
            err.message().contains("not available yet"),
            "{}",
            err.message()
        );
    }

    /// Documented choice, tagged unresolved (3-arg form pending in export)
    /// [FR-014-04]: formattype 'png' (case-insensitive) is accepted; other
    /// format types are rejected.
    #[test]
    fn saveas_formattype_png_only_unresolved_choice() {
        let path = resolve_output_path("figure.png", Some("png")).expect("png accepted");
        assert_eq!(path, PathBuf::from("figure.png"));
        let path = resolve_output_path("figure", Some("PNG")).expect("case-insensitive");
        assert_eq!(path, PathBuf::from("figure.png"));
        let err = resolve_output_path("figure.png", Some("jpeg")).expect_err("jpeg unsupported");
        assert!(
            err.message().contains("currently supports 'png'"),
            "{}",
            err.message()
        );
    }

    /// Documented choice, tagged unresolved (export lists no error cases)
    /// [FR-014-04]: invalid handle scalars and non-handle first arguments
    /// error before any file is touched.
    #[test]
    fn saveas_invalid_handle_errors_unresolved_choice() {
        let _guard = setup();
        let err = futures::executor::block_on(saveas_builtin(vec![
            Value::Num(0.0),
            Value::from("x.png"),
        ]))
        .expect_err("zero handle");
        assert!(
            err.message().contains("must be positive"),
            "{}",
            err.message()
        );

        let err = futures::executor::block_on(saveas_builtin(vec![
            Value::Num(f64::NAN),
            Value::from("x.png"),
        ]))
        .expect_err("NaN handle");
        assert!(
            err.message().contains("must be finite"),
            "{}",
            err.message()
        );

        let err = futures::executor::block_on(saveas_builtin(vec![
            Value::from("not-a-handle"),
            Value::from("x.png"),
        ]))
        .expect_err("text is not a figure handle (explicit-handle-required)");
        assert!(err.message().contains("figure handle"), "{}", err.message());
    }

    /// Documented choice, tagged unresolved (export lists no error cases)
    /// [FR-014-04]: wrong arity and non-text filenames error.
    #[test]
    fn saveas_invalid_arguments_error_unresolved_choice() {
        let _guard = setup();
        let err = futures::executor::block_on(saveas_builtin(vec![Value::Num(1.0)]))
            .expect_err("missing filename");
        assert!(err.message().contains("argument"), "{}", err.message());

        let err =
            futures::executor::block_on(saveas_builtin(vec![Value::Num(1.0), Value::Num(2.0)]))
                .expect_err("numeric filename");
        assert!(err.message().contains("filename"), "{}", err.message());

        let err =
            futures::executor::block_on(saveas_builtin(vec![Value::Num(1.0), Value::from("")]))
                .expect_err("empty filename");
        assert!(err.message().contains("nonempty"), "{}", err.message());
    }
}
