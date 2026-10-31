//! `pcolor` builtin: flat-shaded, zero-height surface coloured by C (spec 058).
//!
//! Normative behaviour comes from the approved `pcolor` 0.2.0 export (claims
//! `pcolor.output-class`, `pcolor.ax-arg`, `pcolor.c-only`, `pcolor.xyc`,
//! `pcolor.current-axes`, `pcolor.view`, `pcolor.face`, `pcolor.error-size`,
//! `pcolor.error-two-args`). The surface is RunMat's own `SurfacePlot` with
//! `flatten_z` (drawn at zero height, coloured by C). The default edge
//! colour is a documented choice (`pcolor.edgecolor` is non-normative). See
//! `specs/058-graphics-n3n4/`.

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_plot::plots::{ColorMap, NumericPlotData, ShadingMode, SurfacePlot};
use runmat_value::{Tensor, Value};

use crate::builtins::plotting::op_common::{apply_axes_target, split_leading_axes_handle};
use crate::builtins::plotting::state::{self, render_active_plot, FigureHandle, PlotRenderOptions};
use crate::builtins::plotting::type_resolvers::handle_scalar_type;
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const BUILTIN_NAME: &str = "pcolor";

/// View after `pcolor`: seen from above [pcolor.view].
const PCOLOR_VIEW: (f32, f32) = (0.0, 90.0);

const PCOLOR_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "h",
    ty: BuiltinParamType::NumericScalar,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Handle of the created surface object.",
}];

const PCOLOR_INPUT_C: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "C",
    ty: BuiltinParamType::NumericArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Numeric matrix of colour values.",
};
const PCOLOR_INPUT_AX: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "ax",
    ty: BuiltinParamType::NumericScalar,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Axes to draw into.",
};
const PCOLOR_INPUT_X: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "X",
    ty: BuiltinParamType::NumericArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Vector of length size(C,2) or matrix the size of C.",
};
const PCOLOR_INPUT_Y: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "Y",
    ty: BuiltinParamType::NumericArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Vector of length size(C,1) or matrix the size of C.",
};

const PCOLOR_INPUTS_C: [BuiltinParamDescriptor; 1] = [PCOLOR_INPUT_C];
const PCOLOR_INPUTS_XYC: [BuiltinParamDescriptor; 3] =
    [PCOLOR_INPUT_X, PCOLOR_INPUT_Y, PCOLOR_INPUT_C];
const PCOLOR_INPUTS_AX_C: [BuiltinParamDescriptor; 2] = [PCOLOR_INPUT_AX, PCOLOR_INPUT_C];

const PCOLOR_SIGNATURES: [BuiltinSignatureDescriptor; 3] = [
    BuiltinSignatureDescriptor {
        label: "h = pcolor(C)",
        inputs: &PCOLOR_INPUTS_C,
        outputs: &PCOLOR_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "h = pcolor(X, Y, C)",
        inputs: &PCOLOR_INPUTS_XYC,
        outputs: &PCOLOR_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "h = pcolor(ax, C)",
        inputs: &PCOLOR_INPUTS_AX_C,
        outputs: &PCOLOR_OUTPUT,
    },
];

const PCOLOR_ERROR_SIZE: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.PCOLOR.INPUT_SIZE_MISMATCH",
    identifier: Some("RunMat:pcolor:InputSizeMismatch"),
    when: "X or Y does not match the columns or rows of C.",
    message: "pcolor: input size mismatch",
};
const PCOLOR_ERROR_TWO_ARGS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.PCOLOR.BAD_SUBSCRIPT",
    identifier: Some("RunMat:badsubscript"),
    when: "Two numeric arguments are given and the first is not an axes.",
    message: "pcolor: two numeric arguments are not a valid call",
};
const PCOLOR_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.PCOLOR.INVALID_INPUT",
    identifier: Some("RunMat:pcolor:InvalidInput"),
    when: "An input is missing, not a real numeric array, empty or N-D.",
    message: "pcolor: invalid input",
};
const PCOLOR_ERROR_UNSUPPORTED_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.PCOLOR.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:pcolor:UnsupportedOption"),
    when: "A form the approved specification does not cover (extra or name-value arguments, complex C, C with fewer than two rows or columns).",
    message: "pcolor: unsupported option",
};
const PCOLOR_ERROR_INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.PCOLOR.INTERNAL",
    identifier: Some("RunMat:pcolor:Internal"),
    when: "An underlying plotting operation fails.",
    message: "pcolor: internal plotting operation failed",
};
const PCOLOR_ERRORS: [BuiltinErrorDescriptor; 5] = [
    PCOLOR_ERROR_SIZE,
    PCOLOR_ERROR_TWO_ARGS,
    PCOLOR_ERROR_INVALID_INPUT,
    PCOLOR_ERROR_UNSUPPORTED_OPTION,
    PCOLOR_ERROR_INTERNAL,
];

pub const PCOLOR_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &PCOLOR_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &PCOLOR_ERRORS,
};

#[runtime_builtin(
    name = "pcolor",
    category = "plotting",
    summary = "Pseudocolor plot: a flat surface at zero height coloured by C.",
    keywords = "pcolor,pseudocolor,surface,plotting",
    sink = true,
    suppress_auto_output = true,
    type_resolver(handle_scalar_type),
    descriptor(crate::builtins::plotting::ops::pcolor::PCOLOR_DESCRIPTOR),
    builtin_path = "crate::builtins::plotting::ops::pcolor"
)]
pub async fn pcolor_builtin(args: Vec<Value>) -> BuiltinResult<f64> {
    let mut gathered = Vec::with_capacity(args.len());
    for value in &args {
        gathered.push(gather_if_needed_async(value).await.map_err(internal)?);
    }
    if gathered.is_empty() {
        return Err(pcolor_error(&PCOLOR_ERROR_INVALID_INPUT, "C is required"));
    }
    let (target, rest) = split_leading_axes_handle(gathered, BUILTIN_NAME)
        .map_err(|err| pcolor_error(&PCOLOR_ERROR_INVALID_INPUT, err.message()))?;

    if rest.iter().skip(1).any(is_text) {
        return Err(pcolor_error(
            &PCOLOR_ERROR_UNSUPPORTED_OPTION,
            "name-value and text arguments are not supported",
        ));
    }
    let (x, y, c) = match rest.len() {
        1 => {
            let c = real_matrix(&rest[0], "C")?;
            let (rows, cols) = (c.rows(), c.cols());
            let x = Tensor::new((1..=cols).map(|v| v as f64).collect(), vec![1, cols])
                .map_err(|e| pcolor_error(&PCOLOR_ERROR_INTERNAL, e))?;
            let y = Tensor::new((1..=rows).map(|v| v as f64).collect(), vec![rows, 1])
                .map_err(|e| pcolor_error(&PCOLOR_ERROR_INTERNAL, e))?;
            (x, y, c)
        }
        2 if target.is_none() => {
            return Err(pcolor_error(
                &PCOLOR_ERROR_TWO_ARGS,
                "use pcolor(C), pcolor(X, Y, C) or pcolor(ax, C)",
            ));
        }
        3 => (
            real_matrix(&rest[0], "X")?,
            real_matrix(&rest[1], "Y")?,
            real_matrix(&rest[2], "C")?,
        ),
        _ => {
            return Err(pcolor_error(
                &PCOLOR_ERROR_UNSUPPORTED_OPTION,
                "only pcolor(C), pcolor(X, Y, C) and pcolor(ax, ...) are supported",
            ));
        }
    };

    let surface = build_pcolor_surface(&x, &y, &c)?;
    apply_axes_target(target, BUILTIN_NAME).map_err(internal)?;
    let (handle, figure, axes) = render_surface_plot(BUILTIN_NAME, surface).map_err(internal)?;
    state::set_view_for_axes(figure, axes, PCOLOR_VIEW.0, PCOLOR_VIEW.1)
        .map_err(|err| pcolor_error(&PCOLOR_ERROR_INTERNAL, err.to_string()))?;
    Ok(handle)
}

/// The pcolor surface: geometry from X/Y, colour source C drawn at zero
/// height (`flatten_z`), flat faces, edges shown (documented choice).
/// Reported `XData`/`YData` keep the given shapes; `ZData` is zeros.
fn build_pcolor_surface(x: &Tensor, y: &Tensor, c: &Tensor) -> BuiltinResult<SurfacePlot> {
    let (rows, cols) = (c.rows(), c.cols());
    if rows < 2 || cols < 2 {
        return Err(pcolor_error(
            &PCOLOR_ERROR_UNSUPPORTED_OPTION,
            "C must have at least two rows and two columns",
        ));
    }
    let x_grid = coordinate_grid(x, rows, cols, Axis::X)?;
    let y_grid = coordinate_grid(y, rows, cols, Axis::Y)?;
    let c_grid = columns(&c.materialize_f64(), rows, cols);
    let mut surface = SurfacePlot::from_coordinate_grids(x_grid, y_grid, c_grid)
        .map_err(|e| pcolor_error(&PCOLOR_ERROR_INTERNAL, e))?;
    surface.colormap = ColorMap::Parula;
    surface.shading_mode = ShadingMode::Flat;
    surface.wireframe = true;
    surface.flatten_z = true;
    surface.lighting_enabled = false;
    surface.set_source_data(
        Some(plot_data(x)?),
        Some(plot_data(y)?),
        Some(
            NumericPlotData::from_f64(vec![0.0; rows * cols], vec![rows, cols])
                .map_err(|e| pcolor_error(&PCOLOR_ERROR_INTERNAL, e))?,
        ),
    );
    Ok(surface)
}

#[derive(Clone, Copy, PartialEq)]
enum Axis {
    X,
    Y,
}

/// Expand X (varies along columns) or Y (varies along rows) to a full grid
/// in `[column][row]` layout. Vectors must match the columns (X) or rows (Y)
/// of C; matrices must be the size of C.
fn coordinate_grid(
    t: &Tensor,
    rows: usize,
    cols: usize,
    axis: Axis,
) -> BuiltinResult<Vec<Vec<f64>>> {
    let data = t.materialize_f64();
    let name = if axis == Axis::X { "X" } else { "Y" };
    if t.rows() == rows && t.cols() == cols {
        return Ok(columns(&data, rows, cols));
    }
    if is_vector(t) {
        let expected = if axis == Axis::X { cols } else { rows };
        if data.len() != expected {
            return Err(pcolor_error(
                &PCOLOR_ERROR_SIZE,
                format!(
                    "{name} has {} elements but C has {expected} {}",
                    data.len(),
                    if axis == Axis::X { "columns" } else { "rows" }
                ),
            ));
        }
        return Ok((0..cols)
            .map(|j| {
                (0..rows)
                    .map(|i| if axis == Axis::X { data[j] } else { data[i] })
                    .collect()
            })
            .collect());
    }
    Err(pcolor_error(
        &PCOLOR_ERROR_SIZE,
        format!("{name} is {}x{} but C is {rows}x{cols}", t.rows(), t.cols()),
    ))
}

fn is_text(value: &Value) -> bool {
    matches!(
        value,
        Value::CharArray(_) | Value::String(_) | Value::StringArray(_)
    )
}

fn is_vector(t: &Tensor) -> bool {
    t.shape.len() <= 2 && (t.rows() == 1 || t.cols() == 1)
}

/// Column-major `rows x cols` data as `[column][row]`.
pub(super) fn columns(data: &[f64], rows: usize, cols: usize) -> Vec<Vec<f64>> {
    (0..cols)
        .map(|j| data[j * rows..(j + 1) * rows].to_vec())
        .collect()
}

fn plot_data(t: &Tensor) -> BuiltinResult<NumericPlotData> {
    NumericPlotData::from_f64(t.materialize_f64().to_vec(), vec![t.rows(), t.cols()])
        .map_err(|e| pcolor_error(&PCOLOR_ERROR_INTERNAL, e))
}

/// A real, non-empty 2-D numeric (or logical) array as a double tensor.
fn real_matrix(value: &Value, name: &str) -> BuiltinResult<Tensor> {
    match value {
        Value::Complex(_, _) | Value::ComplexTensor(_) => Err(pcolor_error(
            &PCOLOR_ERROR_UNSUPPORTED_OPTION,
            format!("complex {name} is not supported"),
        )),
        Value::Num(_)
        | Value::Int(_)
        | Value::Bool(_)
        | Value::Tensor(_)
        | Value::LogicalArray(_) => {
            let t = Tensor::try_from(value)
                .map_err(|e| pcolor_error(&PCOLOR_ERROR_INVALID_INPUT, &e))?;
            if t.shape.iter().skip(2).any(|&d| d != 1) {
                return Err(pcolor_error(
                    &PCOLOR_ERROR_INVALID_INPUT,
                    format!("{name} must be 2-D"),
                ));
            }
            if t.is_empty() {
                return Err(pcolor_error(
                    &PCOLOR_ERROR_INVALID_INPUT,
                    format!("{name} must not be empty"),
                ));
            }
            Ok(t)
        }
        _ => Err(pcolor_error(
            &PCOLOR_ERROR_INVALID_INPUT,
            format!("{name} must be a real numeric array"),
        )),
    }
}

/// Add `surface` to the current axes (respecting `hold`), register its
/// handle and return `(handle, figure, axes index)`. Shared with `surfl`.
pub(super) fn render_surface_plot(
    builtin: &'static str,
    surface: SurfacePlot,
) -> BuiltinResult<(f64, FigureHandle, usize)> {
    let mut surface = Some(surface);
    let slot = std::rc::Rc::new(std::cell::RefCell::new(None));
    let slot_in = std::rc::Rc::clone(&slot);
    let figure = crate::builtins::plotting::current_figure_handle();
    let opts = PlotRenderOptions {
        title: "",
        x_label: "",
        y_label: "",
        axis_equal: false,
        ..Default::default()
    };
    let result = render_active_plot(builtin, opts, move |fig, axes| {
        let surface = surface.take().expect("surface consumed once");
        let plot_index = fig.add_surface_plot_on_axes(surface, axes);
        *slot_in.borrow_mut() = Some((axes, plot_index));
        Ok(())
    });
    let Some((axes, plot_index)) = *slot.borrow() else {
        return Err(result
            .err()
            .unwrap_or_else(|| build_runtime_error("surface was not added").build()));
    };
    let handle = state::register_surface_handle(figure, axes, plot_index);
    if let Err(err) = result {
        let lower = err.to_string().to_lowercase();
        if !(lower.contains("plotting is unavailable") || lower.contains("non-main thread")) {
            return Err(err);
        }
    }
    Ok((handle, figure, axes))
}

fn internal(err: RuntimeError) -> RuntimeError {
    if err.identifier().is_some() {
        return err;
    }
    pcolor_error(&PCOLOR_ERROR_INTERNAL, err.message())
}

fn pcolor_error(error: &'static BuiltinErrorDescriptor, detail: impl AsRef<str>) -> RuntimeError {
    let mut builder = build_runtime_error(format!("{}: {}", error.message, detail.as_ref()))
        .with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

#[cfg(test)]
pub(super) mod pcolor_tests {
    use super::pcolor_builtin;
    use crate::builtins::plotting::state::{self, clear_figure, reset_hold_state_for_run};
    use crate::builtins::plotting::tests::{ensure_plot_test_env, lock_plot_registry};
    use crate::{BuiltinResult, RuntimeError};
    use futures::executor::block_on;
    use runmat_plot::plots::figure::PlotElement;
    use runmat_plot::plots::SurfacePlot;
    use runmat_value::{CharArray, ComplexTensor, Tensor, Value};

    pub(crate) fn setup() -> crate::builtins::plotting::state::PlotTestLockGuard {
        let guard = lock_plot_registry();
        ensure_plot_test_env();
        reset_hold_state_for_run();
        call("figure", &[]).expect("figure");
        let _ = clear_figure(None);
        guard
    }

    pub(crate) fn call(name: &str, args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async(name, args))
    }

    fn direct(args: &[Value]) -> BuiltinResult<f64> {
        block_on(pcolor_builtin(args.to_vec()))
    }

    pub(crate) fn text(s: &str) -> Value {
        Value::CharArray(CharArray::new_row(s))
    }

    /// Matrix from row-major literal rows.
    pub(crate) fn matrix(rows: &[&[f64]]) -> Value {
        let m = rows.len();
        let n = rows[0].len();
        let mut data = Vec::with_capacity(m * n);
        for j in 0..n {
            for row in rows {
                data.push(row[j]);
            }
        }
        Value::Tensor(Tensor::new(data, vec![m, n]).expect("matrix"))
    }

    pub(crate) fn shaped(value: &Value) -> (Vec<usize>, Vec<f64>) {
        match value {
            Value::Num(v) => (vec![1, 1], vec![*v]),
            other => {
                let t = Tensor::try_from(other).expect("numeric");
                (vec![t.rows(), t.cols()], t.materialize_f64().to_vec())
            }
        }
    }

    pub(crate) fn prop(handle: f64, name: &str) -> Value {
        call("get", &[Value::Num(handle), text(name)]).expect("get")
    }

    pub(crate) fn string(value: Value) -> String {
        match value {
            Value::String(s) => s,
            Value::CharArray(ca) => ca.data.iter().collect(),
            other => panic!("expected text, got {other:?}"),
        }
    }

    pub(crate) fn identifier(err: &RuntimeError) -> String {
        err.identifier().unwrap_or_default().to_string()
    }

    /// The SurfacePlot behind a surface handle (headless data check).
    pub(crate) fn surface_of(handle: f64) -> SurfacePlot {
        let parent = shaped(&prop(handle, "Parent")).1[0];
        let resolved =
            crate::builtins::plotting::properties::resolve_plot_handle(&Value::Num(parent), "test")
                .expect("parent axes");
        let crate::builtins::plotting::properties::PlotHandle::Axes(figure, _) = resolved else {
            panic!("parent is not an axes");
        };
        let fig = state::clone_figure(figure).expect("figure");
        fig.plots()
            .filter_map(|p| match p {
                PlotElement::Surface(s) => Some(s.clone()),
                _ => None,
            })
            .last()
            .expect("a surface plot")
    }

    fn view_of(ax: &Value) -> Vec<f64> {
        shaped(&call("view", std::slice::from_ref(ax)).expect("view")).1
    }

    const C23: [&[f64]; 2] = [&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]];

    // ---- observed (normative) ----

    /// [pcolor.output-class, pcolor.c-only] case `c-only`.
    #[test]
    fn observed_c_only_data() {
        let _guard = setup();
        let h = direct(&[matrix(&C23)]).unwrap();
        assert_eq!(string(prop(h, "Type")), "surface");
        assert_eq!(shaped(&prop(h, "XData")), (vec![1, 3], vec![1.0, 2.0, 3.0]));
        assert_eq!(shaped(&prop(h, "YData")), (vec![2, 1], vec![1.0, 2.0]));
        assert_eq!(shaped(&prop(h, "ZData")), (vec![2, 3], vec![0.0; 6]));
        // CData: the colour source of the surface is C (D-058-6).
        let s = surface_of(h);
        assert!(s.flatten_z, "drawn at zero height");
        assert_eq!(
            s.z_data.clone().expect("colour data"),
            vec![vec![1.0, 4.0], vec![2.0, 5.0], vec![3.0, 6.0]]
        );
    }

    /// [pcolor.ax-arg] case `ax-arg`.
    #[test]
    fn observed_axes_argument() {
        let _guard = setup();
        let ax = call("axes", &[]).expect("axes");
        let h = direct(&[ax.clone(), matrix(&C23)]).unwrap();
        assert_eq!(prop(h, "Parent"), ax);
        assert_eq!(
            surface_of(h).z_data.expect("colour data"),
            vec![vec![1.0, 4.0], vec![2.0, 5.0], vec![3.0, 6.0]]
        );
    }

    /// [pcolor.xyc] case `xyc-vectors`.
    #[test]
    fn observed_vector_coordinates_kept_as_given() {
        let _guard = setup();
        let x = matrix(&[&[10.0, 20.0, 30.0]]);
        let y = matrix(&[&[1.0, 2.0]]);
        let h = direct(&[x, y, matrix(&C23)]).unwrap();
        assert_eq!(string(prop(h, "Type")), "surface");
        assert_eq!(
            shaped(&prop(h, "XData")),
            (vec![1, 3], vec![10.0, 20.0, 30.0])
        );
        assert_eq!(shaped(&prop(h, "YData")), (vec![1, 2], vec![1.0, 2.0]));
        assert_eq!(shaped(&prop(h, "ZData")), (vec![2, 3], vec![0.0; 6]));
        assert_eq!(
            surface_of(h).z_data.expect("colour data"),
            vec![vec![1.0, 4.0], vec![2.0, 5.0], vec![3.0, 6.0]]
        );
    }

    /// [pcolor.xyc] case `xyc-matrices`.
    #[test]
    fn observed_matrix_coordinates_kept_as_given() {
        let _guard = setup();
        let x = matrix(&[&[10.0, 20.0, 30.0], &[10.0, 20.0, 30.0]]);
        let y = matrix(&[&[1.0, 1.0, 1.0], &[2.0, 2.0, 2.0]]);
        let h = direct(&[x.clone(), y.clone(), matrix(&C23)]).unwrap();
        assert_eq!(shaped(&prop(h, "XData")), shaped(&x));
        assert_eq!(shaped(&prop(h, "YData")), shaped(&y));
        assert_eq!(shaped(&prop(h, "ZData")), (vec![2, 3], vec![0.0; 6]));
    }

    /// [pcolor.current-axes] case `no-ax-arg`.
    #[test]
    fn observed_draws_into_current_axes() {
        let _guard = setup();
        let h = direct(&[matrix(&[
            &[8.0, 1.0, 6.0],
            &[3.0, 5.0, 7.0],
            &[4.0, 9.0, 2.0],
        ])])
        .unwrap();
        let gca = call("gca", &[]).expect("gca");
        assert_eq!(prop(h, "Parent"), gca);
        assert_eq!(string(prop(h, "Type")), "surface");
    }

    /// [pcolor.view] case `view`.
    #[test]
    fn observed_view_from_above() {
        let _guard = setup();
        let h = direct(&[matrix(&[
            &[8.0, 1.0, 6.0],
            &[3.0, 5.0, 7.0],
            &[4.0, 9.0, 2.0],
        ])])
        .unwrap();
        assert_eq!(view_of(&prop(h, "Parent")), vec![0.0, 90.0]);
    }

    /// [pcolor.face] case `edge-face`; EdgeColor is a documented choice
    /// (`pcolor.edgecolor` non-normative).
    #[test]
    fn observed_face_color_flat_and_edge_choice() {
        let _guard = setup();
        let h = direct(&[matrix(&[
            &[8.0, 1.0, 6.0],
            &[3.0, 5.0, 7.0],
            &[4.0, 9.0, 2.0],
        ])])
        .unwrap();
        assert_eq!(string(prop(h, "FaceColor")), "flat");
        // Documented choice: edges are drawn; RunMat reports them as 'flat'.
        assert_eq!(string(prop(h, "EdgeColor")), "flat");
        assert!(surface_of(h).wireframe);
    }

    /// [pcolor.error-size] case `err-size`.
    #[test]
    fn observed_size_mismatch_errors() {
        let _guard = setup();
        let err =
            direct(&[matrix(&[&[1.0, 2.0]]), matrix(&[&[1.0, 2.0]]), matrix(&C23)]).unwrap_err();
        assert_eq!(identifier(&err), "RunMat:pcolor:InputSizeMismatch");
    }

    /// [pcolor.error-two-args] case `err-two-args` (pcolor.q-two-arg-id).
    #[test]
    fn observed_two_numeric_arguments_error() {
        let _guard = setup();
        let err =
            direct(&[matrix(&[&[1.0, 2.0]]), matrix(&[&[1.0, 2.0], &[3.0, 4.0]])]).unwrap_err();
        assert_eq!(identifier(&err), "RunMat:badsubscript");
    }

    // ---- documented choices for unobserved forms ----

    /// (choice) Y length mismatch and matrix-shape mismatch use the observed
    /// size identifier; extra/name-value arguments, complex C and C with one
    /// row or column are unsupported; missing or non-numeric input is invalid.
    /// Name-value arguments are unsupported.
    #[test]
    fn unresolved_choice_unobserved_forms() {
        let _guard = setup();
        let err = direct(&[
            matrix(&[&[1.0, 2.0, 3.0]]),
            matrix(&[&[1.0, 2.0, 3.0]]),
            matrix(&C23),
        ])
        .unwrap_err();
        assert_eq!(identifier(&err), "RunMat:pcolor:InputSizeMismatch");
        let err = direct(&[
            matrix(&[&[1.0, 2.0], &[3.0, 4.0]]),
            matrix(&[&[1.0, 2.0]]),
            matrix(&C23),
        ])
        .unwrap_err();
        assert_eq!(identifier(&err), "RunMat:pcolor:InputSizeMismatch");
        let err = direct(&[matrix(&C23), text("EdgeColor"), text("none")]).unwrap_err();
        assert_eq!(identifier(&err), "RunMat:pcolor:UnsupportedOption");
        let err = direct(&[
            matrix(&[&[1.0, 2.0, 3.0]]),
            matrix(&[&[1.0, 2.0]]),
            matrix(&C23),
            text("x"),
        ])
        .unwrap_err();
        assert_eq!(identifier(&err), "RunMat:pcolor:UnsupportedOption");
        let complex = Value::ComplexTensor(
            ComplexTensor::new(vec![(1.0, 1.0); 4], vec![2, 2]).expect("complex"),
        );
        let err = direct(&[complex]).unwrap_err();
        assert_eq!(identifier(&err), "RunMat:pcolor:UnsupportedOption");
        let err = direct(&[matrix(&[&[1.0, 2.0, 3.0]])]).unwrap_err();
        assert_eq!(identifier(&err), "RunMat:pcolor:UnsupportedOption");
        let err = direct(&[]).unwrap_err();
        assert_eq!(identifier(&err), "RunMat:pcolor:InvalidInput");
        let err = direct(&[text("abc")]).unwrap_err();
        assert_eq!(identifier(&err), "RunMat:pcolor:InvalidInput");
    }

    /// Dispatcher path: callable by name and returns the surface handle.
    #[test]
    fn registered_through_dispatcher() {
        let _guard = setup();
        let h = call("pcolor", &[matrix(&C23)]).expect("pcolor");
        let h = shaped(&h).1[0];
        assert_eq!(string(prop(h, "Type")), "surface");
    }
}
