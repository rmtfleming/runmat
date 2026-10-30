//! `exportgraphics` builtin: write an axes' or figure's graphics to a file
//! (spec 041).
//!
//! Normative behaviour comes from the approved `exportgraphics` 0.3.0 export
//! (format by extension, axes/figure targets, `Resolution`, `ContentType`,
//! `BackgroundColor`, `Append`, overwrite, and the observed error and warning
//! conditions). RunMat renders figures as raster images only, so the vector
//! formats (pdf, eps) are `UnsupportedOption`. Default pixel size is RunMat's
//! own (documented choice); see `specs/041-exportgraphics/spec.md`.

use std::path::Path;

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::Value;

use crate::builtins::plotting::properties::PlotHandle;
use crate::builtins::plotting::FigureHandle;
use crate::warnings::{self, WarningRequest};
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const BUILTIN_NAME: &str = "exportgraphics";
/// RunMat's reference export size (the `print` default: 800x600 at 150).
const BASE_WIDTH: f64 = 800.0;
const BASE_HEIGHT: f64 = 600.0;
const BASE_RESOLUTION: f64 = 150.0;
const MAX_EXPORT_DIMENSION: f64 = 8192.0;
const JPEG_QUALITY: u8 = 90;

const EG_INPUT_OBJ: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "obj",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Axes or figure handle.",
};
const EG_INPUT_FILENAME: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "filename",
    ty: BuiltinParamType::StringScalar,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Output file; the extension selects the format (png, jpg, jpeg, tif, tiff, gif).",
};
const EG_INPUT_OPTIONS: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "Name, Value",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Variadic,
    default: None,
    description: "Resolution, ContentType, BackgroundColor ('none'), Append (false).",
};
const EG_INPUTS: [BuiltinParamDescriptor; 2] = [EG_INPUT_OBJ, EG_INPUT_FILENAME];
const EG_INPUTS_OPTIONS: [BuiltinParamDescriptor; 3] =
    [EG_INPUT_OBJ, EG_INPUT_FILENAME, EG_INPUT_OPTIONS];
const EG_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "exportgraphics(obj, filename)",
        inputs: &EG_INPUTS,
        outputs: &[],
    },
    BuiltinSignatureDescriptor {
        label: "exportgraphics(obj, filename, Name, Value)",
        inputs: &EG_INPUTS_OPTIONS,
        outputs: &[],
    },
];

const fn eg_error(
    code: &'static str,
    identifier: &'static str,
    when: &'static str,
    message: &'static str,
) -> BuiltinErrorDescriptor {
    BuiltinErrorDescriptor {
        code,
        identifier: Some(identifier),
        when,
        message,
    }
}

const ERR_INVALID_INPUT: BuiltinErrorDescriptor = eg_error(
    "RM.EXPORTGRAPHICS.INVALID_INPUT",
    "RunMat:exportgraphics:InvalidInput",
    "Arguments are missing or malformed.",
    "exportgraphics: invalid input",
);
const ERR_UNSUPPORTED_OPTION: BuiltinErrorDescriptor = eg_error(
    "RM.EXPORTGRAPHICS.UNSUPPORTED_OPTION",
    "RunMat:exportgraphics:UnsupportedOption",
    "A format, option or target RunMat cannot honour (vector formats, unobserved options or handle kinds).",
    "exportgraphics: unsupported option",
);
const ERR_TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = eg_error(
    "RM.EXPORTGRAPHICS.TOO_MANY_OUTPUTS",
    "RunMat:exportgraphics:TooManyOutputs",
    "An output argument is requested.",
    "exportgraphics does not return output arguments",
);
const ERR_NOT_GRAPHICS: BuiltinErrorDescriptor = eg_error(
    "RM.EXPORTGRAPHICS.NOT_GRAPHICS",
    "RunMat:exportgraphics:GraphicsHandleMustBeFirst",
    "The first argument is not a graphics handle.",
    "exportgraphics: the first argument must be an axes or figure handle",
);
const ERR_UNKNOWN_FORMAT: BuiltinErrorDescriptor = eg_error(
    "RM.EXPORTGRAPHICS.UNKNOWN_FORMAT",
    "RunMat:exportgraphics:InvalidFileFormatForExport",
    "The extension is not a recognised export format.",
    "exportgraphics: unrecognised file format",
);
const ERR_NO_EXTENSION: BuiltinErrorDescriptor = eg_error(
    "RM.EXPORTGRAPHICS.NO_EXTENSION",
    "RunMat:exportgraphics:OutputFileNeedsNameAndExtension",
    "The filename has no extension.",
    "exportgraphics: the output file needs a name and an extension",
);
const ERR_CANNOT_CREATE: BuiltinErrorDescriptor = eg_error(
    "RM.EXPORTGRAPHICS.CANNOT_CREATE",
    "RunMat:exportgraphics:CannotCreateOutputFile",
    "The output file cannot be written (for example, its folder does not exist).",
    "exportgraphics: cannot create the output file",
);
const ERR_APPEND: BuiltinErrorDescriptor = eg_error(
    "RM.EXPORTGRAPHICS.APPEND",
    "RunMat:exportgraphics:UnsupportedAppendValue",
    "Append is true for an image-format target.",
    "exportgraphics: Append is not supported for this format",
);
const ERR_RENDER: BuiltinErrorDescriptor = eg_error(
    "RM.EXPORTGRAPHICS.RENDER",
    "RunMat:exportgraphics:RenderFailed",
    "The figure renderer or image encoder fails.",
    "exportgraphics: rendering failed",
);
const EG_ERRORS: [BuiltinErrorDescriptor; 9] = [
    ERR_INVALID_INPUT,
    ERR_UNSUPPORTED_OPTION,
    ERR_TOO_MANY_OUTPUTS,
    ERR_NOT_GRAPHICS,
    ERR_UNKNOWN_FORMAT,
    ERR_NO_EXTENSION,
    ERR_CANNOT_CREATE,
    ERR_APPEND,
    ERR_RENDER,
];

pub const EXPORTGRAPHICS_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &EG_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &EG_ERRORS,
};

const WARN_VECTOR_ON_IMAGE: &str = "RunMat:exportgraphics:ImageOutputIgnoresVectorContentType";
const WARN_TRANSPARENT_BACKGROUND: &str =
    "RunMat:exportgraphics:ReplacingTransparentBackgroundWithDefaultColor";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RasterFormat {
    Png,
    Jpeg,
    Tiff,
    Gif,
}

#[derive(Debug, Default)]
struct ExportOptions {
    resolution: Option<f64>,
    vector_content: bool,
    background_none: bool,
    append: bool,
}

#[runtime_builtin(
    name = "exportgraphics",
    category = "plotting",
    summary = "Save the contents of an axes or figure to an image file.",
    keywords = "exportgraphics,export,figure,axes,png,jpeg,tiff,gif,save",
    sink = true,
    suppress_auto_output = true,
    accel = "metadata",
    descriptor(crate::builtins::plotting::ops::exportgraphics::EXPORTGRAPHICS_DESCRIPTOR),
    builtin_path = "crate::builtins::plotting::ops::exportgraphics"
)]
pub async fn exportgraphics_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    if matches!(crate::output_count::current_output_count(), Some(n) if n > 0) {
        return Err(eg_err(&ERR_TOO_MANY_OUTPUTS, "no output value is returned"));
    }
    if args.len() < 2 {
        return Err(eg_err(
            &ERR_INVALID_INPUT,
            "expected exportgraphics(obj, filename, ...)",
        ));
    }
    let mut gathered = Vec::with_capacity(args.len());
    for value in &args {
        gathered.push(gather_if_needed_async(value).await.map_err(|err| {
            let message = err.message().to_string();
            eg_err_with_source(&ERR_INVALID_INPUT, message, err)
        })?);
    }

    let figure = target_figure(&gathered[0])?;
    let filename = text(&gathered[1]).ok_or_else(|| {
        eg_err(
            &ERR_INVALID_INPUT,
            "filename must be a character vector or string scalar",
        )
    })?;
    if filename.is_empty() || filename.contains('\0') {
        return Err(eg_err(
            &ERR_INVALID_INPUT,
            "filename must be non-empty and must not contain NUL characters",
        ));
    }
    let options = parse_options(&gathered[2..])?;
    let format = format_for(&filename)?;
    if options.append {
        return Err(eg_err(
            &ERR_APPEND,
            "Append true is not supported for image formats",
        ));
    }
    let (width, height) = pixel_size(options.resolution)?;

    let png = render_png(figure, width, height).await?;
    let bytes = encode(png, format)?;
    runmat_filesystem::write_async(Path::new(&filename), &bytes)
        .await
        .map_err(|err| {
            eg_err_with_source(
                &ERR_CANNOT_CREATE,
                format!("unable to write '{filename}': {err}"),
                err,
            )
        })?;

    if options.vector_content {
        warn(
            WARN_VECTOR_ON_IMAGE,
            "exportgraphics: ContentType 'vector' is ignored for image formats; an image was written.",
        )?;
    }
    if options.background_none {
        warn(
            WARN_TRANSPARENT_BACKGROUND,
            "exportgraphics: transparent background is not supported for image formats; the default background colour was used.",
        )?;
    }
    Ok(Value::OutputList(Vec::new()))
}

fn target_figure(value: &Value) -> BuiltinResult<FigureHandle> {
    let is_scalar_number = match value {
        Value::Num(_) | Value::Int(_) => true,
        Value::Tensor(t) => t.len() == 1,
        _ => false,
    };
    if !is_scalar_number {
        return Err(eg_err(
            &ERR_NOT_GRAPHICS,
            "expected an axes or figure handle",
        ));
    }
    let handle =
        crate::builtins::plotting::resolve_plot_handle(value, BUILTIN_NAME).map_err(|err| {
            let message = err.message().to_string();
            eg_err_with_source(&ERR_NOT_GRAPHICS, message, err)
        })?;
    match handle {
        PlotHandle::Figure(figure) => Ok(figure),
        PlotHandle::Axes(figure, _) => Ok(figure),
        PlotHandle::Root => Err(eg_err(
            &ERR_UNSUPPORTED_OPTION,
            "exporting the root object is not supported",
        )),
        _ => Err(eg_err(
            &ERR_UNSUPPORTED_OPTION,
            "only axes and figure handles are supported as export targets",
        )),
    }
}

fn parse_options(rest: &[Value]) -> BuiltinResult<ExportOptions> {
    if !rest.len().is_multiple_of(2) {
        return Err(eg_err(
            &ERR_INVALID_INPUT,
            "options must be given as name-value pairs",
        ));
    }
    let mut options = ExportOptions::default();
    for pair in rest.chunks(2) {
        let Some(name) = text(&pair[0]) else {
            return Err(eg_err(&ERR_INVALID_INPUT, "option names must be text"));
        };
        let value = &pair[1];
        match name.to_ascii_lowercase().as_str() {
            "resolution" => {
                let resolution = scalar(value)
                    .filter(|r| r.is_finite() && *r > 0.0)
                    .ok_or_else(|| {
                        eg_err(
                            &ERR_INVALID_INPUT,
                            "Resolution must be a positive finite scalar",
                        )
                    })?;
                options.resolution = Some(resolution);
            }
            "contenttype" => {
                let content = text(value).map(|s| s.to_ascii_lowercase());
                match content.as_deref() {
                    Some("image") => options.vector_content = false,
                    Some("vector") => options.vector_content = true,
                    _ => {
                        return Err(eg_err(
                            &ERR_INVALID_INPUT,
                            "ContentType must be 'image' or 'vector'",
                        ))
                    }
                }
            }
            "backgroundcolor" => match text(value).map(|s| s.to_ascii_lowercase()).as_deref() {
                Some("none") => options.background_none = true,
                _ => {
                    return Err(eg_err(
                        &ERR_UNSUPPORTED_OPTION,
                        "only BackgroundColor 'none' is supported",
                    ))
                }
            },
            "append" => {
                options.append = match value {
                    Value::Bool(b) => *b,
                    other => match scalar(other) {
                        Some(v) if v == 0.0 || v == 1.0 => v != 0.0,
                        _ => {
                            return Err(eg_err(
                                &ERR_INVALID_INPUT,
                                "Append must be a logical scalar",
                            ))
                        }
                    },
                };
            }
            other => {
                return Err(eg_err(
                    &ERR_UNSUPPORTED_OPTION,
                    format!("option '{other}' is not supported"),
                ))
            }
        }
    }
    Ok(options)
}

fn format_for(filename: &str) -> BuiltinResult<RasterFormat> {
    let Some(ext) = Path::new(filename).extension().and_then(|e| e.to_str()) else {
        return Err(eg_err(
            &ERR_NO_EXTENSION,
            format!("'{filename}' has no file extension"),
        ));
    };
    match ext.to_ascii_lowercase().as_str() {
        "png" => Ok(RasterFormat::Png),
        "jpg" | "jpeg" => Ok(RasterFormat::Jpeg),
        "tif" | "tiff" => Ok(RasterFormat::Tiff),
        "gif" => Ok(RasterFormat::Gif),
        "pdf" | "eps" | "svg" | "emf" => Err(eg_err(
            &ERR_UNSUPPORTED_OPTION,
            format!("'.{ext}' output is not available; RunMat exports png, jpg, tif and gif"),
        )),
        _ => Err(eg_err(
            &ERR_UNKNOWN_FORMAT,
            format!("'.{ext}' is not a recognised export format"),
        )),
    }
}

fn pixel_size(resolution: Option<f64>) -> BuiltinResult<(u32, u32)> {
    let scale = resolution.unwrap_or(BASE_RESOLUTION) / BASE_RESOLUTION;
    let width = (BASE_WIDTH * scale).round();
    let height = (BASE_HEIGHT * scale).round();
    if width < 1.0 || height < 1.0 || width > MAX_EXPORT_DIMENSION || height > MAX_EXPORT_DIMENSION
    {
        return Err(eg_err(
            &ERR_INVALID_INPUT,
            format!("Resolution would produce an unsupported image size ({width}x{height} px)"),
        ));
    }
    Ok((width as u32, height as u32))
}

#[cfg(feature = "plot-core")]
async fn render_png(figure: FigureHandle, width: u32, height: u32) -> BuiltinResult<Vec<u8>> {
    crate::builtins::plotting::render_figure_snapshot(figure, width, height, None)
        .await
        .map_err(|err| {
            let message = err.message().to_string();
            eg_err_with_source(&ERR_RENDER, message, err)
        })
}

#[cfg(not(feature = "plot-core"))]
async fn render_png(_figure: FigureHandle, _width: u32, _height: u32) -> BuiltinResult<Vec<u8>> {
    Err(eg_err(
        &ERR_RENDER,
        "plot-core support is not enabled in this build",
    ))
}

fn encode(png: Vec<u8>, format: RasterFormat) -> BuiltinResult<Vec<u8>> {
    let output = match format {
        RasterFormat::Png => return Ok(png),
        RasterFormat::Jpeg => image::ImageOutputFormat::Jpeg(JPEG_QUALITY),
        RasterFormat::Tiff => image::ImageOutputFormat::Tiff,
        RasterFormat::Gif => image::ImageOutputFormat::Gif,
    };
    let decoded = image::load_from_memory_with_format(&png, image::ImageFormat::Png)
        .map_err(|err| eg_err_with_source(&ERR_RENDER, err.to_string(), err))?;
    let image = match format {
        // JPEG has no alpha channel.
        RasterFormat::Jpeg => image::DynamicImage::ImageRgb8(decoded.to_rgb8()),
        _ => decoded,
    };
    let mut cursor = std::io::Cursor::new(Vec::new());
    image
        .write_to(&mut cursor, output)
        .map_err(|err| eg_err_with_source(&ERR_RENDER, err.to_string(), err))?;
    Ok(cursor.into_inner())
}

fn warn(identifier: &str, message: &str) -> BuiltinResult<()> {
    warnings::emit(WarningRequest {
        builtin: BUILTIN_NAME,
        identifier,
        message,
        show_identifier: false,
    })
}

fn text(value: &Value) -> Option<String> {
    match value {
        Value::CharArray(ca) if ca.rows == 1 || ca.data.is_empty() => {
            Some(ca.data.iter().collect())
        }
        Value::String(s) => Some(s.clone()),
        Value::StringArray(sa) if sa.data.len() == 1 => Some(sa.data[0].clone()),
        _ => None,
    }
}

fn scalar(value: &Value) -> Option<f64> {
    match value {
        Value::Num(v) => Some(*v),
        Value::Int(i) => Some(i.to_f64()),
        Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        Value::Tensor(t) if t.len() == 1 => {
            Some(crate::builtins::common::tensor::tensor_value_f64(t, 0))
        }
        _ => None,
    }
}

fn eg_err(error: &'static BuiltinErrorDescriptor, detail: impl AsRef<str>) -> RuntimeError {
    let mut builder = build_runtime_error(format!("{}: {}", error.message, detail.as_ref()))
        .with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

fn eg_err_with_source(
    error: &'static BuiltinErrorDescriptor,
    detail: impl AsRef<str>,
    source: impl std::error::Error + Send + Sync + 'static,
) -> RuntimeError {
    let mut builder = build_runtime_error(format!("{}: {}", error.message, detail.as_ref()))
        .with_builtin(BUILTIN_NAME)
        .with_source(source);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

#[cfg(test)]
mod exportgraphics_tests {
    use crate::builtins::plotting::state::{clear_figure, reset_hold_state_for_run};
    use crate::builtins::plotting::tests::{ensure_plot_test_env, lock_plot_registry};
    use crate::{BuiltinResult, RuntimeError};
    use futures::executor::block_on;
    use runmat_value::{CharArray, Tensor, Value};
    use std::path::Path;

    const PNG_SIGNATURE: &[u8] = &[137, 80, 78, 71, 13, 10, 26, 10];
    const JPEG_SIGNATURE: &[u8] = &[255, 216, 255];
    const TIFF_LE_SIGNATURE: &[u8] = &[73, 73, 42, 0];
    const GIF_SIGNATURE: &[u8] = b"GIF89a";

    fn setup() -> crate::builtins::plotting::state::PlotTestLockGuard {
        let guard = lock_plot_registry();
        ensure_plot_test_env();
        reset_hold_state_for_run();
        call("figure", &[]).expect("figure");
        let _ = clear_figure(None);
        crate::warning_store::reset();
        guard
    }

    /// Error paths call the builtin function directly; see validation.md,
    /// finding F-1 (dispatcher re-runs a failing builtin).
    fn direct(args: &[Value]) -> BuiltinResult<Value> {
        block_on(super::exportgraphics_builtin(args.to_vec()))
    }

    fn call(name: &str, args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async(name, args))
    }

    fn text(s: &str) -> Value {
        Value::CharArray(CharArray::new_row(s))
    }

    fn path_text(path: &Path) -> Value {
        text(&path.to_string_lossy())
    }

    fn row(data: &[f64]) -> Value {
        Value::Tensor(Tensor::new(data.to_vec(), vec![1, data.len()]).expect("row"))
    }

    fn draw_line() {
        call("plot", &[row(&[1.0, 2.0, 3.0])]).expect("plot");
    }

    fn gca() -> Value {
        call("gca", &[]).expect("gca")
    }

    fn identifier(err: &RuntimeError) -> String {
        err.identifier().unwrap_or_default().to_string()
    }

    fn isfile(path: &Path) -> bool {
        match call("isfile", &[path_text(path)]).expect("isfile") {
            Value::Bool(b) => b,
            other => panic!("isfile returned {other:?}"),
        }
    }

    fn export(args: &[Value]) {
        let out = call("exportgraphics", args).expect("exportgraphics");
        assert_eq!(out, Value::OutputList(Vec::new()), "no output value");
    }

    fn starts_with(path: &Path, signature: &[u8]) -> bool {
        std::fs::read(path)
            .expect("read exported file")
            .starts_with(signature)
    }

    fn png_size(path: &Path) -> (u32, u32) {
        let bytes = std::fs::read(path).expect("read png");
        // IHDR width/height are big-endian u32 at offsets 16 and 20.
        let w = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
        let h = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
        (w, h)
    }

    fn warning_ids() -> Vec<String> {
        crate::warning_store::take_all()
            .into_iter()
            .map(|w| w.identifier)
            .collect()
    }

    // ---- observed (normative) ----

    /// [exportgraphics.void-file-write, exportgraphics.targets,
    /// exportgraphics.format-by-extension] case `file-created`.
    #[test]
    fn observed_file_created_from_axes() {
        let _guard = setup();
        draw_line();
        let dir = tempfile::tempdir().expect("tempdir");
        let f = dir.path().join("out.png");
        assert!(!isfile(&f));
        export(&[gca(), path_text(&f)]);
        assert!(isfile(&f));
        assert!(starts_with(&f, PNG_SIGNATURE));
        assert!(warning_ids().is_empty());
    }

    /// [exportgraphics.void-file-write, exportgraphics.error-output-arg]
    /// case `output-arg`.
    #[test]
    fn observed_output_request_errors() {
        let _guard = setup();
        draw_line();
        let dir = tempfile::tempdir().expect("tempdir");
        let f = dir.path().join("y.png");
        let args = [gca(), path_text(&f)];
        let _outputs = crate::output_count::push_output_count(Some(1));
        let err = direct(&args).expect_err("requesting an output must fail");
        assert_eq!(identifier(&err), "RunMat:exportgraphics:TooManyOutputs");
    }

    /// [exportgraphics.targets] case `target-figure`.
    #[test]
    fn observed_figure_target() {
        let _guard = setup();
        draw_line();
        let dir = tempfile::tempdir().expect("tempdir");
        let f = dir.path().join("fig.png");
        let fig = call("gcf", &[]).expect("gcf");
        export(&[fig, Value::String(f.to_string_lossy().into())]);
        assert!(starts_with(&f, PNG_SIGNATURE));
    }

    /// [exportgraphics.format-by-extension] cases `fmt-upper`, `fmt-jpg`,
    /// `fmt-jpeg`, `fmt-tif`, `fmt-tiff`, `fmt-gif` (raster formats).
    #[test]
    fn observed_raster_format_by_extension() {
        let _guard = setup();
        draw_line();
        let dir = tempfile::tempdir().expect("tempdir");
        for (name, signature) in [
            ("a.PNG", PNG_SIGNATURE),
            ("a.jpg", JPEG_SIGNATURE),
            ("b.jpeg", JPEG_SIGNATURE),
            ("a.tif", TIFF_LE_SIGNATURE),
            ("b.tiff", TIFF_LE_SIGNATURE),
            ("a.gif", GIF_SIGNATURE),
            ("c.JPG", JPEG_SIGNATURE),
        ] {
            let f = dir.path().join(name);
            export(&[gca(), path_text(&f)]);
            assert!(starts_with(&f, signature), "{name} has the wrong signature");
        }
        // the filename is used exactly as given
        assert!(dir.path().join("a.PNG").is_file());
    }

    /// [exportgraphics.resolution] cases `res-100`, `res-200`.
    #[test]
    fn observed_resolution_scales_pixels() {
        let _guard = setup();
        draw_line();
        let dir = tempfile::tempdir().expect("tempdir");
        let f100 = dir.path().join("r100.png");
        let f200 = dir.path().join("r200.png");
        export(&[
            gca(),
            path_text(&f100),
            text("Resolution"),
            Value::Num(100.0),
        ]);
        export(&[
            gca(),
            path_text(&f200),
            text("Resolution"),
            Value::Num(200.0),
        ]);
        let (w1, h1) = png_size(&f100);
        let (w2, h2) = png_size(&f200);
        let rw = w2 as f64 / w1 as f64;
        let rh = h2 as f64 / h1 as f64;
        assert!((rw - 2.0).abs() < 0.05, "width ratio {rw}");
        assert!((rh - 2.0).abs() < 0.05, "height ratio {rh}");
    }

    /// [exportgraphics.warning-content-png-vector] cases
    /// `content-png-vector`, `content-jpg-vector`.
    #[test]
    fn observed_vector_content_type_on_image_warns_and_writes() {
        let _guard = setup();
        draw_line();
        let dir = tempfile::tempdir().expect("tempdir");
        for name in ["v.png", "v.jpg"] {
            let f = dir.path().join(name);
            export(&[gca(), path_text(&f), text("ContentType"), text("vector")]);
            assert!(isfile(&f));
            assert_eq!(
                warning_ids(),
                vec!["RunMat:exportgraphics:ImageOutputIgnoresVectorContentType".to_string()]
            );
        }
        let f = dir.path().join("i.png");
        export(&[gca(), path_text(&f), text("ContentType"), text("image")]);
        assert!(isfile(&f));
        assert!(warning_ids().is_empty());
    }

    /// [exportgraphics.warning-background] case `background`.
    #[test]
    fn observed_background_none_warns_and_writes() {
        let _guard = setup();
        draw_line();
        let dir = tempfile::tempdir().expect("tempdir");
        let f = dir.path().join("bg.png");
        export(&[gca(), path_text(&f), text("BackgroundColor"), text("none")]);
        assert!(starts_with(&f, PNG_SIGNATURE));
        assert_eq!(
            warning_ids(),
            vec![
                "RunMat:exportgraphics:ReplacingTransparentBackgroundWithDefaultColor".to_string()
            ]
        );
    }

    /// [exportgraphics.error-append-png] case `append-png`; Append false is
    /// accepted.
    #[test]
    fn observed_append_true_with_png_errors() {
        let _guard = setup();
        draw_line();
        let dir = tempfile::tempdir().expect("tempdir");
        let f = dir.path().join("ap.png");
        export(&[gca(), path_text(&f)]);
        let before = std::fs::read(&f).unwrap();
        let err = direct(&[gca(), path_text(&f), text("Append"), Value::Bool(true)])
            .expect_err("Append true with png must fail");
        assert_eq!(
            identifier(&err),
            "RunMat:exportgraphics:UnsupportedAppendValue"
        );
        assert_eq!(std::fs::read(&f).unwrap(), before, "file unchanged");
        export(&[gca(), path_text(&f), text("Append"), Value::Bool(false)]);
    }

    /// [exportgraphics.overwrite] case `overwrite`.
    #[test]
    fn observed_overwrite_replaces_file() {
        let _guard = setup();
        draw_line();
        let dir = tempfile::tempdir().expect("tempdir");
        let f = dir.path().join("ow.png");
        export(&[gca(), path_text(&f)]);
        let b1 = std::fs::read(&f).unwrap();
        let _ = clear_figure(None);
        call("plot", &[row(&[3.0, 1.0, 2.0])]).expect("plot");
        export(&[gca(), path_text(&f)]);
        assert!(isfile(&f));
        assert_ne!(std::fs::read(&f).unwrap(), b1, "bytes must change");
    }

    /// [exportgraphics.error-ext-unknown] case `err-ext-unknown`.
    #[test]
    fn observed_unknown_extension_errors() {
        let _guard = setup();
        draw_line();
        let dir = tempfile::tempdir().expect("tempdir");
        let f = dir.path().join("x.xyz");
        let err = direct(&[gca(), path_text(&f)]).expect_err("unknown ext");
        assert_eq!(
            identifier(&err),
            "RunMat:exportgraphics:InvalidFileFormatForExport"
        );
        assert!(!f.exists());
    }

    /// [exportgraphics.error-ext-none] case `err-ext-none`.
    #[test]
    fn observed_missing_extension_errors() {
        let _guard = setup();
        draw_line();
        let dir = tempfile::tempdir().expect("tempdir");
        let f = dir.path().join("noext");
        let err = direct(&[gca(), path_text(&f)]).expect_err("no ext");
        assert_eq!(
            identifier(&err),
            "RunMat:exportgraphics:OutputFileNeedsNameAndExtension"
        );
        assert!(!f.exists());
    }

    /// [exportgraphics.error-bad-folder] case `err-bad-folder`.
    #[test]
    fn observed_missing_folder_errors() {
        let _guard = setup();
        draw_line();
        let dir = tempfile::tempdir().expect("tempdir");
        let f = dir.path().join("no_such").join("sub").join("x.png");
        let err = direct(&[gca(), path_text(&f)]).expect_err("bad folder");
        assert_eq!(
            identifier(&err),
            "RunMat:exportgraphics:CannotCreateOutputFile"
        );
        assert!(!f.exists());
    }

    /// [exportgraphics.error-not-graphics] case `err-not-graphics`.
    #[test]
    fn observed_non_graphics_first_argument_errors() {
        let _guard = setup();
        let dir = tempfile::tempdir().expect("tempdir");
        let f = dir.path().join("ng.png");
        for obj in [Value::Num(987_654.0), text("gca"), row(&[1.0, 2.0])] {
            let err = direct(&[obj, path_text(&f)]).expect_err("not a graphics handle");
            assert_eq!(
                identifier(&err),
                "RunMat:exportgraphics:GraphicsHandleMustBeFirst"
            );
        }
        assert!(!f.exists());
    }

    // ---- unresolved / beyond RunMat's renderer ----

    /// pdf and eps are confirmed formats (format-by-extension,
    /// content-type-pdf, append-pdf) but RunMat has no vector renderer;
    /// emf and svg are exportgraphics.q-more-formats.
    #[test]
    fn unresolved_choice_vector_formats_unsupported() {
        let _guard = setup();
        draw_line();
        let dir = tempfile::tempdir().expect("tempdir");
        for name in ["x.pdf", "x.PDF", "x.eps", "x.svg", "x.emf"] {
            let f = dir.path().join(name);
            let err = direct(&[gca(), path_text(&f)]).expect_err("vector format must fail");
            assert_eq!(
                identifier(&err),
                "RunMat:exportgraphics:UnsupportedOption",
                "{name}"
            );
            assert!(!f.exists(), "{name} must not be written");
        }
    }

    /// exportgraphics.q-more-formats (Padding, Colorspace, ...), unobserved
    /// BackgroundColor values, and non-figure/axes graphics handles.
    #[test]
    fn unresolved_choice_unobserved_options_unsupported() {
        let _guard = setup();
        let line = call("plot", &[row(&[1.0, 2.0, 3.0])]).expect("plot");
        let dir = tempfile::tempdir().expect("tempdir");
        let f = dir.path().join("opt.png");
        for extra in [
            vec![text("Padding"), Value::Num(10.0)],
            vec![text("Colorspace"), text("gray")],
            vec![text("BackgroundColor"), text("white")],
            vec![text("BackgroundColor"), row(&[1.0, 1.0, 1.0])],
        ] {
            let mut args = vec![gca(), path_text(&f)];
            args.extend(extra);
            let err = direct(&args).expect_err("option must fail");
            assert_eq!(identifier(&err), "RunMat:exportgraphics:UnsupportedOption");
        }
        for obj in [line, Value::Num(0.0)] {
            let err = direct(&[obj, path_text(&f)]).expect_err("handle kind");
            assert_eq!(identifier(&err), "RunMat:exportgraphics:UnsupportedOption");
        }
        assert!(!f.exists());
    }

    // ---- validation ----

    /// Malformed filenames and option values.
    #[test]
    fn rejects_invalid_arguments() {
        let _guard = setup();
        draw_line();
        let dir = tempfile::tempdir().expect("tempdir");
        let f = dir.path().join("bad.png");
        let p = path_text(&f);
        let cases: Vec<Vec<Value>> = vec![
            vec![gca()],
            vec![gca(), Value::Num(3.0)],
            vec![gca(), text("")],
            vec![gca(), p.clone(), text("Resolution")],
            vec![gca(), p.clone(), text("Resolution"), Value::Num(-5.0)],
            vec![gca(), p.clone(), text("Resolution"), text("high")],
            vec![gca(), p.clone(), text("ContentType"), text("auto3d")],
            vec![gca(), p.clone(), text("Append"), text("yes")],
            vec![gca(), p.clone(), Value::Num(1.0), Value::Num(2.0)],
        ];
        for args in cases {
            let err = direct(&args).expect_err("invalid arguments");
            assert_eq!(
                identifier(&err),
                "RunMat:exportgraphics:InvalidInput",
                "args {args:?}"
            );
        }
        assert!(!f.exists());
    }
}
