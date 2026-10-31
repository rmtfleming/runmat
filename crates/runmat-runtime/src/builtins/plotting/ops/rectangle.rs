//! `rectangle` builtin: draw a rectangle (optionally with curved corners)
//! and return a Rectangle object (spec 065).
//!
//! Normative behaviour comes from the approved `rectangle` 0.2.0 export
//! (claims `rectangle.signature-primary`, `rectangle.defaults`,
//! `rectangle.cobra-form`, `rectangle.curvature`, `rectangle.colors`,
//! `rectangle.parent` and the four `rectangle.error-*` claims). RunMat has
//! no Rectangle graphics kind, so the returned value is a handle object of
//! class `matlab.graphics.primitive.Rectangle` whose properties are read
//! with dot access (D-065-4), and the shape is drawn as a RunMat patch
//! (D-065-5). Colours are parsed by `validatecolor`'s single-colour parser.
//! The default EdgeColor is a documented choice (`rectangle.q-theme`). See
//! `specs/065-graphics-n5/`.

use glam::{Vec3, Vec4};
use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_plot::plots::{LinePlot, LineStyle, PatchEdgeColorMode, PatchFaceColorMode, PatchPlot};
use runmat_value::{CharArray, HandleRef, ObjectInstance, Tensor, Value};

use crate::builtins::plotting::op_common::apply_axes_target;
use crate::builtins::plotting::properties::{resolve_plot_handle, PlotHandle};
use crate::builtins::plotting::state::{
    self, append_active_plot, axis_limits_snapshot, current_axes_state, encode_axes_handle,
    plot_child_handles_for_axes, PlotRenderOptions,
};
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

use super::validatecolor::parse_single_color;

const BUILTIN_NAME: &str = "rectangle";

/// Class name of the returned object [rectangle.signature-primary].
pub const RECTANGLE_CLASS_NAME: runmat_types::StaticClassIdentity =
    runmat_types::StaticClassIdentity::new("matlab.graphics.primitive.Rectangle");

/// Defaults [rectangle.defaults]; EdgeColor is a documented choice.
const DEFAULT_POSITION: [f64; 4] = [0.0, 0.0, 1.0, 1.0];
const DEFAULT_LINE_WIDTH: f64 = 0.5;
const DEFAULT_EDGE_COLOR: [f64; 3] = [0.0, 0.0, 0.0];
/// Segments per curved corner (documented choice).
const CORNER_SEGMENTS: usize = 16;

const RECT_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "h",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Rectangle object (properties read with dot access).",
}];
const RECT_INPUT_AX: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "ax",
    ty: BuiltinParamType::AxesHandle,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Axes to draw into.",
};
const RECT_INPUT_PROPS: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "Name, Value",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Variadic,
    default: None,
    description: "Position, Curvature, FaceColor, EdgeColor, LineWidth, LineStyle.",
};
const RECT_INPUTS: [BuiltinParamDescriptor; 1] = [RECT_INPUT_PROPS];
const RECT_INPUTS_AX: [BuiltinParamDescriptor; 2] = [RECT_INPUT_AX, RECT_INPUT_PROPS];
const RECT_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "h = rectangle(Name, Value, ...)",
        inputs: &RECT_INPUTS,
        outputs: &RECT_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "h = rectangle(ax, Name, Value, ...)",
        inputs: &RECT_INPUTS_AX,
        outputs: &RECT_OUTPUT,
    },
];

const RECT_ERROR_DIMENSIONS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.RECTANGLE.DIMENSIONS_OUTSIDE_RANGE",
    identifier: Some("RunMat:hg:set_chck:DimensionsOutsideRange"),
    when: "Position has a negative width or height, or a non-finite element.",
    message: "rectangle: Position width and height must be non-negative and finite",
};
const RECT_ERROR_POSITION_LENGTH: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.RECTANGLE.EXPECTED_4_ELEMENT_VECTOR",
    identifier: Some("RunMat:hg:dt_conv:Matrix_to_matlabRect:Expected4ElementVector"),
    when: "Position does not have four elements.",
    message: "rectangle: Position must have four elements [x y w h]",
};
const RECT_ERROR_CURVATURE: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.RECTANGLE.CURVATURE_RANGE",
    identifier: Some("RunMat:hg:shaped_arrays:CurvatureRange"),
    when: "A Curvature value is outside [0, 1].",
    message: "rectangle: Curvature values must be in the range [0, 1]",
};
const RECT_ERROR_BAD_PROPERTY: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.RECTANGLE.INVALID_PROPERTY",
    identifier: Some("RunMat:hg:InvalidProperty"),
    when: "A name is not a rectangle property.",
    message: "rectangle: unrecognized property",
};
const RECT_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.RECTANGLE.INVALID_INPUT",
    identifier: Some("RunMat:rectangle:InvalidInput"),
    when: "A property value is invalid, a value is missing, or a name is not text.",
    message: "rectangle: invalid input",
};
const RECT_ERROR_INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.RECTANGLE.INTERNAL",
    identifier: Some("RunMat:rectangle:Internal"),
    when: "An underlying plotting operation fails.",
    message: "rectangle: internal plotting operation failed",
};
const RECT_ERRORS: [BuiltinErrorDescriptor; 6] = [
    RECT_ERROR_DIMENSIONS,
    RECT_ERROR_POSITION_LENGTH,
    RECT_ERROR_CURVATURE,
    RECT_ERROR_BAD_PROPERTY,
    RECT_ERROR_INVALID_INPUT,
    RECT_ERROR_INTERNAL,
];

pub const RECTANGLE_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &RECT_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &RECT_ERRORS,
};

/// A colour property: an RGB triple or `'none'`.
#[derive(Clone, Copy, Debug, PartialEq)]
enum ColorSpec {
    None,
    Rgb([f64; 3]),
}

#[derive(Clone, Debug)]
struct RectangleProps {
    position: [f64; 4],
    /// As given: one value or two.
    curvature: Vec<f64>,
    face_color: ColorSpec,
    edge_color: ColorSpec,
    line_width: f64,
    line_style: String,
}

impl Default for RectangleProps {
    fn default() -> Self {
        Self {
            position: DEFAULT_POSITION,
            curvature: vec![0.0, 0.0],
            face_color: ColorSpec::None,
            edge_color: ColorSpec::Rgb(DEFAULT_EDGE_COLOR),
            line_width: DEFAULT_LINE_WIDTH,
            line_style: "-".to_string(),
        }
    }
}

#[runtime_builtin(
    name = "rectangle",
    category = "plotting",
    summary = "Draw a rectangle (optionally with curved corners) in the current axes.",
    keywords = "rectangle,shape,primitive,curvature,plotting",
    sink = true,
    suppress_auto_output = true,
    descriptor(crate::builtins::plotting::ops::rectangle::RECTANGLE_DESCRIPTOR),
    builtin_path = "crate::builtins::plotting::ops::rectangle"
)]
pub async fn rectangle_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    let mut args_g = Vec::with_capacity(args.len());
    for value in &args {
        args_g.push(gather_if_needed_async(value).await.map_err(internal)?);
    }
    // A leading axes handle [rectangle.parent].
    let mut target = None;
    if let Some(first) = args_g.first() {
        if let Ok(PlotHandle::Axes(figure, axes)) = resolve_plot_handle(first, BUILTIN_NAME) {
            target = Some((figure, axes));
            args_g.remove(0);
        }
    }
    let props = parse_props(&args_g)?;
    let drawing = build_drawing(&props)?;

    apply_axes_target(target, BUILTIN_NAME).map_err(internal)?;
    let axes_state = current_axes_state();
    let (figure, axes) = (axes_state.handle, axes_state.active_index);
    let had_children = !plot_child_handles_for_axes(figure, axes).is_empty();
    let previous_limits = axis_limits_snapshot();
    render_drawing(drawing)?;
    update_limits(figure, axes, &props, had_children, previous_limits)?;
    rectangle_object(&props, encode_axes_handle(figure, axes))
}

fn parse_props(args: &[Value]) -> BuiltinResult<RectangleProps> {
    let mut props = RectangleProps::default();
    let mut i = 0;
    while i < args.len() {
        let Some(name) = text_of(&args[i]) else {
            return Err(rect_error(
                &RECT_ERROR_INVALID_INPUT,
                "expected a property name",
            ));
        };
        let key = name.trim().to_ascii_lowercase();
        if !matches!(
            key.as_str(),
            "position" | "curvature" | "facecolor" | "edgecolor" | "linewidth" | "linestyle"
        ) {
            return Err(rect_error(
                &RECT_ERROR_BAD_PROPERTY,
                format!("'{name}' is not a rectangle property"),
            ));
        }
        let Some(value) = args.get(i + 1) else {
            return Err(rect_error(
                &RECT_ERROR_INVALID_INPUT,
                format!("property '{name}' has no value"),
            ));
        };
        match key.as_str() {
            "position" => props.position = parse_position(value)?,
            "curvature" => props.curvature = parse_curvature(value)?,
            "facecolor" => props.face_color = parse_color(value, "FaceColor")?,
            "edgecolor" => props.edge_color = parse_color(value, "EdgeColor")?,
            "linewidth" => {
                let width = numeric_values(value)
                    .filter(|v| v.len() == 1 && v[0].is_finite() && v[0] > 0.0)
                    .ok_or_else(|| {
                        rect_error(
                            &RECT_ERROR_INVALID_INPUT,
                            "LineWidth must be a positive finite scalar",
                        )
                    })?;
                props.line_width = width[0];
            }
            _ => {
                let style = text_of(value)
                    .map(|s| s.trim().to_string())
                    .filter(|s| matches!(s.as_str(), "-" | "--" | ":" | "-." | "none"))
                    .ok_or_else(|| {
                        rect_error(
                            &RECT_ERROR_INVALID_INPUT,
                            "LineStyle must be '-', '--', ':', '-.' or 'none'",
                        )
                    })?;
                props.line_style = style;
            }
        }
        i += 2;
    }
    Ok(props)
}

fn parse_position(value: &Value) -> BuiltinResult<[f64; 4]> {
    let v = numeric_values(value)
        .ok_or_else(|| rect_error(&RECT_ERROR_INVALID_INPUT, "Position must be numeric"))?;
    if v.len() != 4 {
        return Err(rect_error(
            &RECT_ERROR_POSITION_LENGTH,
            format!("got {} element(s)", v.len()),
        ));
    }
    if v.iter().any(|x| !x.is_finite()) || v[2] < 0.0 || v[3] < 0.0 {
        return Err(rect_error(
            &RECT_ERROR_DIMENSIONS,
            format!("got [{} {} {} {}]", v[0], v[1], v[2], v[3]),
        ));
    }
    Ok([v[0], v[1], v[2], v[3]])
}

fn parse_curvature(value: &Value) -> BuiltinResult<Vec<f64>> {
    let v = numeric_values(value)
        .filter(|v| v.len() == 1 || v.len() == 2)
        .ok_or_else(|| {
            rect_error(
                &RECT_ERROR_INVALID_INPUT,
                "Curvature must be a numeric scalar or two-element vector",
            )
        })?;
    if v.iter().any(|c| !(0.0..=1.0).contains(c)) {
        return Err(rect_error(&RECT_ERROR_CURVATURE, format!("got {v:?}")));
    }
    Ok(v)
}

fn parse_color(value: &Value, name: &str) -> BuiltinResult<ColorSpec> {
    if let Some(text) = text_of(value) {
        if text.trim().eq_ignore_ascii_case("none") {
            return Ok(ColorSpec::None);
        }
    }
    parse_single_color(value)
        .map(ColorSpec::Rgb)
        .map_err(|err| {
            rect_error(
                &RECT_ERROR_INVALID_INPUT,
                format!("invalid {name}: {}", err.message()),
            )
        })
}

/// Outline polygon: corners are quarter-ellipses with radii `cx*w/2` and
/// `cy*h/2` (documented choice), counter-clockwise from the lower right.
fn outline(props: &RectangleProps) -> Vec<(f64, f64)> {
    let [x, y, w, h] = props.position;
    let (cx, cy) = match props.curvature.as_slice() {
        [c] => (*c, *c),
        [a, b] => (*a, *b),
        _ => (0.0, 0.0),
    };
    let (rx, ry) = (cx * w / 2.0, cy * h / 2.0);
    if rx <= 0.0 || ry <= 0.0 {
        return vec![(x, y), (x + w, y), (x + w, y + h), (x, y + h)];
    }
    // Corner centres and the start angle of each quarter arc.
    let corners = [
        (x + w - rx, y + ry, -std::f64::consts::FRAC_PI_2),
        (x + w - rx, y + h - ry, 0.0),
        (x + rx, y + h - ry, std::f64::consts::FRAC_PI_2),
        (x + rx, y + ry, std::f64::consts::PI),
    ];
    let mut points = Vec::with_capacity(4 * (CORNER_SEGMENTS + 1));
    for (ccx, ccy, start) in corners {
        for k in 0..=CORNER_SEGMENTS {
            let t = start + std::f64::consts::FRAC_PI_2 * (k as f64) / (CORNER_SEGMENTS as f64);
            let p = (ccx + rx * t.cos(), ccy + ry * t.sin());
            // Full curvature makes neighbouring arcs share an end point.
            let repeated = points.last().is_some_and(|q: &(f64, f64)| {
                (q.0 - p.0).abs() < 1e-12 && (q.1 - p.1).abs() < 1e-12
            });
            if !repeated {
                points.push(p);
            }
        }
    }
    if points.len() > 1 {
        let (first, last) = (points[0], points[points.len() - 1]);
        if (first.0 - last.0).abs() < 1e-12 && (first.1 - last.1).abs() < 1e-12 {
            points.pop();
        }
    }
    points
}

/// How the rectangle is drawn: a patch, or — when the width or height is
/// zero, which a patch face cannot represent — a closed outline line
/// (documented choice).
enum Drawing {
    Patch(PatchPlot),
    Outline(LinePlot),
}

fn build_drawing(props: &RectangleProps) -> BuiltinResult<Drawing> {
    let [_, _, w, h] = props.position;
    if w > 0.0 && h > 0.0 {
        return build_patch(props).map(Drawing::Patch);
    }
    let mut points = outline(props);
    points.push(points[0]);
    let (xs, ys): (Vec<f64>, Vec<f64>) = points.into_iter().unzip();
    let style = match props.line_style.as_str() {
        "--" => LineStyle::Dashed,
        ":" => LineStyle::Dotted,
        "-." => LineStyle::DashDot,
        "none" => LineStyle::None,
        _ => LineStyle::Solid,
    };
    let (color, style) = match props.edge_color {
        ColorSpec::Rgb(c) => (vec4(c), style),
        ColorSpec::None => (vec4([0.0, 0.0, 0.0]), LineStyle::None),
    };
    let line = LinePlot::new(xs, ys)
        .map_err(|e| rect_error(&RECT_ERROR_INTERNAL, e))?
        .with_style(color, props.line_width as f32, style);
    Ok(Drawing::Outline(line))
}

fn build_patch(props: &RectangleProps) -> BuiltinResult<PatchPlot> {
    let points = outline(props);
    let vertices: Vec<Vec3> = points
        .iter()
        .map(|&(px, py)| Vec3::new(px as f32, py as f32, 0.0))
        .collect();
    let faces = vec![(0..vertices.len()).collect::<Vec<_>>()];
    let mut patch =
        PatchPlot::new(vertices, faces).map_err(|e| rect_error(&RECT_ERROR_INTERNAL, e))?;
    match props.face_color {
        ColorSpec::None => patch.set_face_color_mode(PatchFaceColorMode::None),
        ColorSpec::Rgb(c) => {
            patch.set_face_color(vec4(c));
            patch.set_face_color_mode(PatchFaceColorMode::Color);
        }
    }
    match props.edge_color {
        ColorSpec::None => patch.set_edge_color_mode(PatchEdgeColorMode::None),
        ColorSpec::Rgb(c) => {
            patch.set_edge_color(vec4(c));
            patch.set_edge_color_mode(PatchEdgeColorMode::Color);
        }
    }
    if props.line_style == "none" {
        patch.set_edge_color_mode(PatchEdgeColorMode::None);
    }
    patch.set_line_width(props.line_width as f32);
    Ok(patch)
}

fn vec4(c: [f64; 3]) -> Vec4 {
    Vec4::new(c[0] as f32, c[1] as f32, c[2] as f32, 1.0)
}

/// Add the patch to the current axes without clearing it (documented
/// choice: a primitive).
fn render_drawing(drawing: Drawing) -> BuiltinResult<()> {
    let mut drawing = Some(drawing);
    let slot = std::rc::Rc::new(std::cell::RefCell::new(None));
    let slot_in = std::rc::Rc::clone(&slot);
    let figure = crate::builtins::plotting::current_figure_handle();
    let opts = PlotRenderOptions {
        title: "",
        x_label: "",
        y_label: "",
        ..Default::default()
    };
    let result = append_active_plot(BUILTIN_NAME, opts, move |fig, axes| {
        let (plot_index, is_patch) = match drawing.take().expect("drawing consumed once") {
            Drawing::Patch(patch) => (fig.add_patch_plot_on_axes(patch, axes), true),
            Drawing::Outline(line) => (fig.add_line_plot_on_axes(line, axes), false),
        };
        *slot_in.borrow_mut() = Some((axes, plot_index, is_patch));
        Ok(())
    });
    let Some((axes, plot_index, is_patch)) = *slot.borrow() else {
        return Err(result
            .err()
            .map(internal)
            .unwrap_or_else(|| rect_error(&RECT_ERROR_INTERNAL, "the rectangle was not added")));
    };
    if is_patch {
        state::register_patch_handle(figure, axes, plot_index);
    } else {
        state::register_line_handle(figure, axes, plot_index);
    }
    if let Err(err) = result {
        let lower = err.to_string().to_lowercase();
        if !(lower.contains("plotting is unavailable") || lower.contains("non-main thread")) {
            return Err(internal(err));
        }
    }
    Ok(())
}

/// Optional axis limits `(lo, hi)`.
type Limits = Option<(f64, f64)>;

/// Axis limits after drawing (documented choice; FR-065-22): in axes that
/// were empty with automatic limits, the limits become the rectangle's
/// extent [rectangle.cobra-form]; limits that are already set grow to
/// include it; automatic limits of axes with other content are left alone.
fn update_limits(
    figure: state::FigureHandle,
    axes: usize,
    props: &RectangleProps,
    had_children: bool,
    previous: (Limits, Limits),
) -> BuiltinResult<()> {
    let [x, y, w, h] = props.position;
    let grow = |prev: Option<(f64, f64)>, lo: f64, hi: f64| -> Option<(f64, f64)> {
        let (lo, hi) = match prev {
            Some((a, b)) => (a.min(lo), b.max(hi)),
            None if had_children => return None,
            None => (lo, hi),
        };
        (lo < hi).then_some((lo, hi))
    };
    let new_x = grow(previous.0, x, x + w).or(previous.0);
    let new_y = grow(previous.1, y, y + h).or(previous.1);
    if new_x != previous.0 || new_y != previous.1 {
        state::set_axis_limits_for_axes(figure, axes, new_x, new_y)
            .map_err(|err| rect_error(&RECT_ERROR_INTERNAL, err.to_string()))?;
    }
    Ok(())
}

/// The Rectangle handle object (dot-access properties; D-065-4).
fn rectangle_object(props: &RectangleProps, parent: f64) -> BuiltinResult<Value> {
    let mut object = ObjectInstance::new(RECTANGLE_CLASS_NAME.to_string());
    let p = &mut object.properties;
    p.insert("Type".to_string(), char_row("rectangle"));
    p.insert("Parent".to_string(), Value::Num(parent));
    p.insert("Position".to_string(), row(&props.position)?);
    p.insert("Curvature".to_string(), row(&props.curvature)?);
    p.insert("FaceColor".to_string(), color_value(props.face_color)?);
    p.insert("EdgeColor".to_string(), color_value(props.edge_color)?);
    p.insert("LineWidth".to_string(), Value::Num(props.line_width));
    p.insert("LineStyle".to_string(), char_row(&props.line_style));
    let target = runmat_gc::gc_allocate(Value::Object(object))
        .map_err(|err| rect_error(&RECT_ERROR_INTERNAL, err.to_string()))?;
    Ok(Value::HandleObject(HandleRef {
        class_name: RECTANGLE_CLASS_NAME.into(),
        target,
        valid: true,
    }))
}

/// Read a property of a Rectangle object (tests and internal use).
#[allow(dead_code)]
pub(crate) fn rectangle_property(value: &Value, name: &str) -> BuiltinResult<Value> {
    let Value::HandleObject(handle) = value else {
        return Err(rect_error(
            &RECT_ERROR_INVALID_INPUT,
            "expected a Rectangle object",
        ));
    };
    runmat_gc::gc_with_value(&handle.target, |v| match v {
        Value::Object(object) => object.properties.get(name).cloned(),
        _ => None,
    })
    .map_err(|err| rect_error(&RECT_ERROR_INTERNAL, err.to_string()))?
    .ok_or_else(|| rect_error(&RECT_ERROR_INVALID_INPUT, format!("no property '{name}'")))
}

fn color_value(color: ColorSpec) -> BuiltinResult<Value> {
    match color {
        ColorSpec::None => Ok(char_row("none")),
        ColorSpec::Rgb(c) => row(&c),
    }
}

fn row(data: &[f64]) -> BuiltinResult<Value> {
    if data.len() == 1 {
        return Ok(Value::Num(data[0]));
    }
    Tensor::new(data.to_vec(), vec![1, data.len()])
        .map(Value::Tensor)
        .map_err(|e| rect_error(&RECT_ERROR_INTERNAL, e))
}

fn char_row(text: &str) -> Value {
    Value::CharArray(CharArray::new_row(text))
}

fn numeric_values(value: &Value) -> Option<Vec<f64>> {
    match value {
        Value::Num(v) => Some(vec![*v]),
        Value::Int(i) => Some(vec![i.to_f64()]),
        Value::Tensor(_) => Tensor::try_from(value)
            .ok()
            .map(|t| t.materialize_f64().to_vec()),
        _ => None,
    }
}

fn text_of(value: &Value) -> Option<String> {
    match value {
        Value::CharArray(ca) if ca.rows <= 1 => Some(ca.data.iter().collect()),
        Value::String(s) => Some(s.clone()),
        Value::StringArray(sa) if sa.data.len() == 1 => Some(sa.data[0].clone()),
        _ => None,
    }
}

fn internal(err: RuntimeError) -> RuntimeError {
    if err.identifier().is_some() {
        return err;
    }
    rect_error(&RECT_ERROR_INTERNAL, err.message())
}

fn rect_error(error: &'static BuiltinErrorDescriptor, detail: impl AsRef<str>) -> RuntimeError {
    let mut builder = build_runtime_error(format!("{}: {}", error.message, detail.as_ref()))
        .with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

/// Spec 065 (`rectangle` 0.2.0). `observed_*` tests reproduce confirmed
/// observations; `unresolved_choice_*` and `deviation_*` tests pin RunMat's
/// documented choices and are not claims about MATLAB.
#[cfg(test)]
mod rectangle_tests {
    use super::{outline, rectangle_builtin, rectangle_property, RectangleProps};
    use crate::builtins::plotting::state::{clear_figure, reset_hold_state_for_run};
    use crate::builtins::plotting::tests::{ensure_plot_test_env, lock_plot_registry};
    use crate::{BuiltinResult, RuntimeError};
    use futures::executor::block_on;
    use runmat_value::{CharArray, Tensor, Value};

    fn setup() -> crate::builtins::plotting::state::PlotTestLockGuard {
        let guard = lock_plot_registry();
        ensure_plot_test_env();
        reset_hold_state_for_run();
        call("figure", &[]).expect("figure");
        let _ = clear_figure(None);
        guard
    }

    fn call(name: &str, args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async(name, args))
    }

    fn direct(args: &[Value]) -> BuiltinResult<Value> {
        block_on(rectangle_builtin(args.to_vec()))
    }

    fn text(s: &str) -> Value {
        Value::CharArray(CharArray::new_row(s))
    }

    fn row(data: &[f64]) -> Value {
        Value::Tensor(Tensor::new(data.to_vec(), vec![1, data.len()]).expect("row"))
    }

    fn numbers(value: &Value) -> (Vec<usize>, Vec<f64>) {
        match value {
            Value::Num(v) => (vec![1, 1], vec![*v]),
            other => {
                let t = Tensor::try_from(other).expect("numeric");
                (vec![t.rows(), t.cols()], t.materialize_f64().to_vec())
            }
        }
    }

    fn string(value: Value) -> String {
        match value {
            Value::String(s) => s,
            Value::CharArray(ca) => ca.data.iter().collect(),
            other => panic!("expected text, got {other:?}"),
        }
    }

    /// Dot access as the VM performs it (`h.Name`).
    fn member(h: &Value, name: &str) -> Value {
        block_on(crate::object::resolve::load_member(
            h.clone(),
            name.to_string(),
            false,
            None,
        ))
        .expect("member")
    }

    fn identifier(err: &RuntimeError) -> String {
        err.identifier().unwrap_or_default().to_string()
    }

    fn class_of(value: &Value) -> String {
        string(call("class", std::slice::from_ref(value)).expect("class"))
    }

    // ---- observed (normative) ----

    /// [rectangle.signature-primary, rectangle.defaults] case `default`.
    #[test]
    fn observed_default_object() {
        let _guard = setup();
        let h = direct(&[]).unwrap();
        assert_eq!(class_of(&h), "matlab.graphics.primitive.Rectangle");
        assert_eq!(
            numbers(&member(&h, "Position")),
            (vec![1, 4], vec![0.0, 0.0, 1.0, 1.0])
        );
        assert_eq!(
            numbers(&member(&h, "Curvature")),
            (vec![1, 2], vec![0.0, 0.0])
        );
        assert_eq!(string(member(&h, "FaceColor")), "none");
        assert_eq!(numbers(&member(&h, "LineWidth")).1, vec![0.5]);
        assert_eq!(string(member(&h, "LineStyle")), "-");
        // Default EdgeColor is a documented choice (rectangle.q-theme).
    }

    /// [rectangle.cobra-form] case `cobra-form`.
    #[test]
    fn observed_cobra_form() {
        let _guard = setup();
        let h = direct(&[
            text("Position"),
            row(&[0.0, 0.0, 40.0, 0.1]),
            text("EdgeColor"),
            text("black"),
            text("LineWidth"),
            Value::Num(2.0),
            text("LineStyle"),
            text("-."),
        ])
        .unwrap();
        assert_eq!(class_of(&h), "matlab.graphics.primitive.Rectangle");
        assert_eq!(
            numbers(&member(&h, "Position")).1,
            vec![0.0, 0.0, 40.0, 0.1]
        );
        assert_eq!(numbers(&member(&h, "Curvature")).1, vec![0.0, 0.0]);
        assert_eq!(string(member(&h, "FaceColor")), "none");
        assert_eq!(numbers(&member(&h, "EdgeColor")).1, vec![0.0, 0.0, 0.0]);
        assert_eq!(numbers(&member(&h, "LineWidth")).1, vec![2.0]);
        assert_eq!(string(member(&h, "LineStyle")), "-.");
    }

    /// [rectangle.cobra-form] case `cobra-limits` (fresh axes).
    #[test]
    fn observed_cobra_limits_in_fresh_axes() {
        let _guard = setup();
        call(
            "rectangle",
            &[
                text("Position"),
                row(&[0.0, 0.0, 40.0, 0.1]),
                text("EdgeColor"),
                text("black"),
                text("LineWidth"),
                Value::Num(2.0),
                text("LineStyle"),
                text("-."),
            ],
        )
        .expect("rectangle");
        assert_eq!(numbers(&call("xlim", &[]).unwrap()).1, vec![0.0, 40.0]);
        assert_eq!(numbers(&call("ylim", &[]).unwrap()).1, vec![0.0, 0.1]);
    }

    /// [rectangle.curvature] cases `curvature-scalar`, `curvature-pair`.
    #[test]
    fn observed_curvature_stored_as_given() {
        let _guard = setup();
        let h = direct(&[
            text("Position"),
            row(&[1.0, 2.0, 3.0, 4.0]),
            text("Curvature"),
            Value::Num(1.0),
        ])
        .unwrap();
        assert_eq!(numbers(&member(&h, "Curvature")), (vec![1, 1], vec![1.0]));
        assert_eq!(numbers(&member(&h, "Position")).1, vec![1.0, 2.0, 3.0, 4.0]);
        let h = direct(&[
            text("Position"),
            row(&[1.0, 2.0, 3.0, 4.0]),
            text("Curvature"),
            row(&[0.5, 0.2]),
        ])
        .unwrap();
        assert_eq!(
            numbers(&member(&h, "Curvature")),
            (vec![1, 2], vec![0.5, 0.2])
        );
    }

    /// [rectangle.colors] case `face-color`.
    #[test]
    fn observed_face_color_name_stored_as_rgb() {
        let _guard = setup();
        let h = direct(&[text("FaceColor"), text("r")]).unwrap();
        assert_eq!(
            numbers(&member(&h, "FaceColor")),
            (vec![1, 3], vec![1.0, 0.0, 0.0])
        );
        assert_eq!(numbers(&member(&h, "Position")).1, vec![0.0, 0.0, 1.0, 1.0]);
    }

    /// [rectangle.parent] case `no-output`: one object is added to the
    /// current axes (it appears as the drawing patch: D-065-5).
    #[test]
    fn observed_no_output_adds_one_object() {
        let _guard = setup();
        call("rectangle", &[text("Position"), row(&[1.0, 1.0, 2.0, 2.0])]).expect("rectangle");
        let gca = call("gca", &[]).expect("gca");
        let children = numbers(&call("allchild", &[gca]).expect("children")).1;
        assert_eq!(children.len(), 1);
    }

    /// [rectangle.parent] case `ax-arg`, and Parent = gca without `ax`.
    #[test]
    fn observed_axes_argument() {
        let _guard = setup();
        let ax = call("axes", &[]).expect("axes");
        let h = direct(&[ax.clone(), text("Position"), row(&[0.0, 0.0, 1.0, 1.0])]).unwrap();
        assert_eq!(member(&h, "Parent"), ax);
        let h = direct(&[]).unwrap();
        assert_eq!(member(&h, "Parent"), call("gca", &[]).unwrap());
    }

    /// [rectangle.error-negative-width] case `err-negative-width`.
    #[test]
    fn observed_negative_width_errors() {
        let _guard = setup();
        let err = direct(&[text("Position"), row(&[0.0, 0.0, -1.0, 1.0])]).unwrap_err();
        assert_eq!(
            identifier(&err),
            "RunMat:hg:set_chck:DimensionsOutsideRange"
        );
    }

    /// [rectangle.error-position-length] case `err-position-length`.
    #[test]
    fn observed_position_length_errors() {
        let _guard = setup();
        let err = direct(&[text("Position"), row(&[0.0, 0.0, 1.0])]).unwrap_err();
        assert_eq!(
            identifier(&err),
            "RunMat:hg:dt_conv:Matrix_to_matlabRect:Expected4ElementVector"
        );
    }

    /// [rectangle.error-curvature-range] case `err-curvature-range`.
    #[test]
    fn observed_curvature_range_errors() {
        let _guard = setup();
        let err = direct(&[text("Curvature"), row(&[2.0, 0.0])]).unwrap_err();
        assert_eq!(identifier(&err), "RunMat:hg:shaped_arrays:CurvatureRange");
    }

    /// [rectangle.error-bad-property] case `err-bad-property`.
    #[test]
    fn observed_bad_property_errors() {
        let _guard = setup();
        let err = direct(&[text("NotAProperty"), Value::Num(1.0)]).unwrap_err();
        assert_eq!(identifier(&err), "RunMat:hg:InvalidProperty");
    }

    // ---- documented choices and deviations ----

    /// (choice) Unobserved invalid values and forms (FR-065-27).
    #[test]
    fn unresolved_choice_invalid_values() {
        let _guard = setup();
        let id = |args: &[Value]| identifier(&direct(args).unwrap_err());
        assert_eq!(
            id(&[text("Position"), row(&[0.0, 0.0, 1.0, -1.0])]),
            "RunMat:hg:set_chck:DimensionsOutsideRange"
        );
        assert_eq!(
            id(&[text("Position"), row(&[0.0, f64::NAN, 1.0, 1.0])]),
            "RunMat:hg:set_chck:DimensionsOutsideRange"
        );
        assert_eq!(
            id(&[text("Curvature"), Value::Num(-0.5)]),
            "RunMat:hg:shaped_arrays:CurvatureRange"
        );
        assert_eq!(
            id(&[text("Curvature"), row(&[0.1, 0.2, 0.3])]),
            "RunMat:rectangle:InvalidInput"
        );
        assert_eq!(
            id(&[text("Position"), text("x")]),
            "RunMat:rectangle:InvalidInput"
        );
        assert_eq!(
            id(&[text("LineWidth"), Value::Num(0.0)]),
            "RunMat:rectangle:InvalidInput"
        );
        assert_eq!(
            id(&[text("LineStyle"), text("==")]),
            "RunMat:rectangle:InvalidInput"
        );
        assert_eq!(
            id(&[text("FaceColor"), text("nocolour")]),
            "RunMat:rectangle:InvalidInput"
        );
        assert_eq!(id(&[text("Position")]), "RunMat:rectangle:InvalidInput");
        assert_eq!(
            id(&[Value::Num(3.0), Value::Num(1.0)]),
            "RunMat:rectangle:InvalidInput"
        );
        // Accepted: case-insensitive names, hex and triplet colours, 'none'.
        let h = direct(&[
            text("facecolor"),
            text("#00FF00"),
            text("EDGECOLOR"),
            text("none"),
            text("Position"),
            row(&[0.0, 0.0, 0.0, 2.0]),
        ])
        .unwrap();
        assert_eq!(
            numbers(&rectangle_property(&h, "FaceColor").unwrap()).1,
            vec![0.0, 1.0, 0.0]
        );
        assert_eq!(string(rectangle_property(&h, "EdgeColor").unwrap()), "none");
        assert_eq!(string(rectangle_property(&h, "Type").unwrap()), "rectangle");
    }

    /// (choice) Rounded-corner geometry: full curvature of a square is a
    /// closed curve inside the square touching its edge midpoints.
    #[test]
    fn unresolved_choice_curved_outline() {
        let square = RectangleProps {
            position: [0.0, 0.0, 2.0, 2.0],
            curvature: vec![1.0],
            ..Default::default()
        };
        let points = outline(&square);
        assert!(points.len() > 4);
        for (x, y) in &points {
            let r = ((x - 1.0).powi(2) + (y - 1.0).powi(2)).sqrt();
            assert!((r - 1.0).abs() < 1e-12, "on the inscribed circle");
        }
        let plain = RectangleProps::default();
        assert_eq!(
            outline(&plain),
            vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]
        );
    }

    /// (choice) A second rectangle grows limits set by the first; existing
    /// plot objects are kept (no clearing without `hold`).
    #[test]
    fn unresolved_choice_limits_grow_and_objects_kept() {
        let _guard = setup();
        direct(&[text("Position"), row(&[0.0, 0.0, 1.0, 1.0])]).unwrap();
        direct(&[text("Position"), row(&[2.0, -1.0, 1.0, 1.0])]).unwrap();
        assert_eq!(numbers(&call("xlim", &[]).unwrap()).1, vec![0.0, 3.0]);
        assert_eq!(numbers(&call("ylim", &[]).unwrap()).1, vec![-1.0, 1.0]);
        let gca = call("gca", &[]).expect("gca");
        let children = numbers(&call("allchild", &[gca]).expect("children")).1;
        assert_eq!(children.len(), 2);
    }

    /// D-065-4: `get(h, 'Position')` on the Rectangle object is not wired.
    #[test]
    #[ignore = "D-065-4: get/set on Rectangle objects needs upstream get.rs/set.rs support"]
    fn deviation_get_on_rectangle_object() {
        let _guard = setup();
        let h = direct(&[]).unwrap();
        let pos = call("get", &[h, text("Position")]).expect("get");
        assert_eq!(numbers(&pos).1, vec![0.0, 0.0, 1.0, 1.0]);
    }

    /// D-065-5: the axes' child is the drawing patch, not the Rectangle.
    #[test]
    #[ignore = "D-065-5: axes Children list the patch (Type 'patch'), not the Rectangle object"]
    fn deviation_axes_child_is_rectangle() {
        let _guard = setup();
        call("rectangle", &[text("Position"), row(&[1.0, 1.0, 2.0, 2.0])]).expect("rectangle");
        let gca = call("gca", &[]).expect("gca");
        let children = call("allchild", &[gca]).expect("children");
        assert_eq!(class_of(&children), "matlab.graphics.primitive.Rectangle");
    }

    /// Dispatcher path.
    #[test]
    fn registered_through_dispatcher() {
        let _guard = setup();
        let h = call("rectangle", &[text("FaceColor"), text("b")]).expect("rectangle");
        assert_eq!(class_of(&h), "matlab.graphics.primitive.Rectangle");
    }
}
