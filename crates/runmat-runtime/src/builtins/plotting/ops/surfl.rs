//! `surfl` builtin: 3-D surface coloured by a reflectance (shading) map
//! (spec 058).
//!
//! Normative behaviour comes from the approved `surfl` 0.2.0 export (claims
//! `surfl.output-class`, `surfl.data`, `surfl.cdata`, `surfl.view`,
//! `surfl.face`, `surfl.error-z-2x3`, `surfl.error-size`, `surfl.error-char`).
//! How MATLAB computes the colour data was not established
//! (`surfl.q-cdata`), so RunMat uses its own, independently chosen
//! reflectance model: Lambert diffuse plus Blinn-Phong specular terms from a
//! light placed relative to the viewer (`surfl_reflectance`). The default
//! edge colour is a documented choice (`surfl.edgecolor` is non-normative).
//! See `specs/058-graphics-n3n4/`.

use glam::Vec4;
use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_plot::plots::{ColorMap, NumericPlotData, ShadingMode, SurfacePlot};
use runmat_value::{Tensor, Value};

use super::pcolor::{columns, render_surface_plot};
use crate::builtins::plotting::state;
use crate::builtins::plotting::type_resolvers::handle_scalar_type;
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const BUILTIN_NAME: &str = "surfl";

/// View after `surfl` [surfl.view].
pub(crate) const SURFL_VIEW: (f64, f64) = (-37.5, 30.0);

/// RunMat's reflectance model (independently chosen, not MATLAB's): ambient,
/// diffuse and specular weights, specular exponent, and the light's offset
/// from the viewing direction in degrees (azimuth, elevation).
const AMBIENT: f64 = 0.25;
const DIFFUSE: f64 = 0.6;
const SPECULAR: f64 = 0.15;
const SHININESS: i32 = 8;
const LIGHT_OFFSET_DEG: (f64, f64) = (45.0, 15.0);

const SURFL_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "h",
    ty: BuiltinParamType::NumericScalar,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Handle of the created surface object.",
}];

const SURFL_INPUT_Z: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "Z",
    ty: BuiltinParamType::NumericArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Numeric matrix of heights with at least 3 rows.",
};
const SURFL_INPUTS_Z: [BuiltinParamDescriptor; 1] = [SURFL_INPUT_Z];
const SURFL_INPUTS_XYZ: [BuiltinParamDescriptor; 3] = [
    BuiltinParamDescriptor {
        name: "X",
        ty: BuiltinParamType::NumericArray,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Vector of length size(Z,2) or matrix the size of Z.",
    },
    BuiltinParamDescriptor {
        name: "Y",
        ty: BuiltinParamType::NumericArray,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Vector of length size(Z,1) or matrix the size of Z.",
    },
    SURFL_INPUT_Z,
];

const SURFL_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "h = surfl(Z)",
        inputs: &SURFL_INPUTS_Z,
        outputs: &SURFL_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "h = surfl(X, Y, Z)",
        inputs: &SURFL_INPUTS_XYZ,
        outputs: &SURFL_OUTPUT,
    },
];

const SURFL_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SURFL.INVALID_INPUT",
    identifier: Some("RunMat:surfl:InvalidInput"),
    when: "Z has fewer than 3 rows; or (RunMat choice) an input is not a real, non-empty 2-D numeric array.",
    message: "surfl: invalid input",
};
const SURFL_ERROR_X_LENGTH: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SURFL.X_LENGTH",
    identifier: Some("RunMat:xyzchk:lengthXDoesNotMatchNumColumnsZ"),
    when: "Vector X does not have one element per column of Z.",
    message: "surfl: the length of X must match the number of columns of Z",
};
const SURFL_ERROR_Y_LENGTH: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SURFL.Y_LENGTH",
    identifier: Some("RunMat:xyzchk:lengthYDoesNotMatchNumRowsZ"),
    when: "(RunMat choice) Vector Y does not have one element per row of Z.",
    message: "surfl: the length of Y must match the number of rows of Z",
};
const SURFL_ERROR_SIZE: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SURFL.INPUT_SIZE_MISMATCH",
    identifier: Some("RunMat:surfl:InputSizeMismatch"),
    when: "(RunMat choice) Matrix X or Y is not the size of Z.",
    message: "surfl: X and Y matrices must be the size of Z",
};
const SURFL_ERROR_NOT_ENOUGH: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SURFL.NOT_ENOUGH_INPUTS",
    identifier: Some("RunMat:narginchk:notEnoughInputs"),
    when: "A single char argument is given, or (RunMat choice) no argument or a single non-numeric argument.",
    message: "surfl: not enough input arguments",
};
const SURFL_ERROR_UNSUPPORTED_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SURFL.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:surfl:UnsupportedOption"),
    when: "A form the approved specification does not cover (2 or more than 3 arguments, complex input, Z with fewer than 3 columns).",
    message: "surfl: unsupported option",
};
const SURFL_ERROR_INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SURFL.INTERNAL",
    identifier: Some("RunMat:surfl:Internal"),
    when: "An underlying plotting operation fails.",
    message: "surfl: internal plotting operation failed",
};
const SURFL_ERRORS: [BuiltinErrorDescriptor; 7] = [
    SURFL_ERROR_INVALID_INPUT,
    SURFL_ERROR_X_LENGTH,
    SURFL_ERROR_Y_LENGTH,
    SURFL_ERROR_SIZE,
    SURFL_ERROR_NOT_ENOUGH,
    SURFL_ERROR_UNSUPPORTED_OPTION,
    SURFL_ERROR_INTERNAL,
];

pub const SURFL_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &SURFL_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &SURFL_ERRORS,
};

#[runtime_builtin(
    name = "surfl",
    category = "plotting",
    summary = "3-D surface plot coloured by a lighting reflectance map.",
    keywords = "surfl,surface,lighting,shaded,plotting",
    sink = true,
    suppress_auto_output = true,
    type_resolver(handle_scalar_type),
    descriptor(crate::builtins::plotting::ops::surfl::SURFL_DESCRIPTOR),
    builtin_path = "crate::builtins::plotting::ops::surfl"
)]
pub async fn surfl_builtin(args: Vec<Value>) -> BuiltinResult<f64> {
    let mut gathered = Vec::with_capacity(args.len());
    for value in &args {
        gathered.push(gather_if_needed_async(value).await.map_err(internal)?);
    }
    let (x, y, z) = match gathered.len() {
        0 => {
            return Err(surfl_error(&SURFL_ERROR_NOT_ENOUGH, "Z is required"));
        }
        1 => {
            if !is_numeric(&gathered[0]) {
                return Err(surfl_error(
                    &SURFL_ERROR_NOT_ENOUGH,
                    "Z must be a numeric matrix",
                ));
            }
            let z = real_matrix(&gathered[0], "Z")?;
            let (rows, cols) = (z.rows(), z.cols());
            let x: Vec<f64> = (1..=cols).map(|v| v as f64).collect();
            let y: Vec<f64> = (1..=rows).map(|v| v as f64).collect();
            check_size(&z)?;
            (
                expand(&x, rows, cols, true),
                expand(&y, rows, cols, false),
                z,
            )
        }
        3 => {
            let x = real_matrix(&gathered[0], "X")?;
            let y = real_matrix(&gathered[1], "Y")?;
            let z = real_matrix(&gathered[2], "Z")?;
            check_size(&z)?;
            let (rows, cols) = (z.rows(), z.cols());
            (
                coordinate_grid(&x, rows, cols, true)?,
                coordinate_grid(&y, rows, cols, false)?,
                z,
            )
        }
        _ => {
            return Err(surfl_error(
                &SURFL_ERROR_UNSUPPORTED_OPTION,
                "only surfl(Z) and surfl(X, Y, Z) are supported",
            ));
        }
    };

    let (rows, cols) = (z.rows(), z.cols());
    let z_data = z.materialize_f64();
    let reflectance = surfl_reflectance(&x, &y, &z_data, rows, cols, SURFL_VIEW);
    let surface = build_surfl_surface(&x, &y, &z_data, &reflectance, rows, cols)?;
    let (handle, figure, axes) = render_surface_plot(BUILTIN_NAME, surface).map_err(internal)?;
    state::set_view_for_axes(figure, axes, SURFL_VIEW.0 as f32, SURFL_VIEW.1 as f32)
        .map_err(|err| surfl_error(&SURFL_ERROR_INTERNAL, err.to_string()))?;
    Ok(handle)
}

/// [surfl.error-z-2x3] Z needs at least 3 rows; fewer than 3 columns was not
/// observed (`surfl.q-min-size`) and is unsupported.
fn check_size(z: &Tensor) -> BuiltinResult<()> {
    if z.rows() < 3 {
        return Err(surfl_error(
            &SURFL_ERROR_INVALID_INPUT,
            "Z must have at least 3 rows",
        ));
    }
    if z.cols() < 3 {
        return Err(surfl_error(
            &SURFL_ERROR_UNSUPPORTED_OPTION,
            "Z with fewer than 3 columns is not supported",
        ));
    }
    Ok(())
}

fn build_surfl_surface(
    x: &[f64],
    y: &[f64],
    z: &[f64],
    reflectance: &[f64],
    rows: usize,
    cols: usize,
) -> BuiltinResult<SurfacePlot> {
    let mut surface = SurfacePlot::from_coordinate_grids(
        columns(x, rows, cols),
        columns(y, rows, cols),
        columns(z, rows, cols),
    )
    .map_err(|e| surfl_error(&SURFL_ERROR_INTERNAL, e))?;
    surface.colormap = ColorMap::Parula;
    surface.shading_mode = ShadingMode::Flat;
    surface.wireframe = true;
    surface.lighting_enabled = false;
    surface.color_grid = Some(reflectance_colors(reflectance, rows, cols));
    let data = |values: &[f64]| {
        NumericPlotData::from_f64(values.to_vec(), vec![rows, cols])
            .map_err(|e| surfl_error(&SURFL_ERROR_INTERNAL, e))
    };
    surface.set_source_data(Some(data(x)?), Some(data(y)?), Some(data(z)?));
    Ok(surface)
}

/// Map reflectance through the colormap with automatic limits, as a
/// `[column][row]` RGBA grid.
fn reflectance_colors(reflectance: &[f64], rows: usize, cols: usize) -> Vec<Vec<Vec4>> {
    let (lo, hi) = reflectance
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &v| {
            (lo.min(v), hi.max(v))
        });
    let span = hi - lo;
    let colormap = ColorMap::Parula;
    (0..cols)
        .map(|j| {
            (0..rows)
                .map(|i| {
                    let v = reflectance[i + j * rows];
                    let t = if span > 0.0 { (v - lo) / span } else { 0.5 };
                    colormap.map_value(t as f32).extend(1.0)
                })
                .collect()
        })
        .collect()
}

/// RunMat's reflectance map for a surface given column-major `rows x cols`
/// coordinate grids, seen from `view` = (azimuth, elevation) in degrees.
///
/// Coordinates are first scaled to unit range per axis. Normals come from
/// central differences (one-sided at the border) and face the viewer. The
/// light sits at the viewer's direction offset by `LIGHT_OFFSET_DEG`. Each
/// value is `AMBIENT + DIFFUSE * max(0, n.l) + SPECULAR * max(0, n.h)^SHININESS`,
/// clamped to [0, 1]. This is an independently chosen textbook model; it is
/// not claimed to reproduce MATLAB's values (`surfl.q-cdata`).
pub(crate) fn surfl_reflectance(
    x: &[f64],
    y: &[f64],
    z: &[f64],
    rows: usize,
    cols: usize,
    view: (f64, f64),
) -> Vec<f64> {
    let scale = |v: &[f64]| -> Vec<f64> {
        let (lo, hi) = v
            .iter()
            .filter(|a| a.is_finite())
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &a| {
                (lo.min(a), hi.max(a))
            });
        let span = if hi > lo { hi - lo } else { 1.0 };
        v.iter().map(|a| (a - lo) / span).collect()
    };
    let (xs, ys, zs) = (scale(x), scale(y), scale(z));
    let point = |i: usize, j: usize| {
        let k = i + j * rows;
        [xs[k], ys[k], zs[k]]
    };
    let diff = |a: [f64; 3], b: [f64; 3]| [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    let viewer = direction(view.0, view.1);
    let light = direction(view.0 + LIGHT_OFFSET_DEG.0, view.1 + LIGHT_OFFSET_DEG.1);
    let half = normalize([
        viewer[0] + light[0],
        viewer[1] + light[1],
        viewer[2] + light[2],
    ]);
    let mut out = Vec::with_capacity(rows * cols);
    for j in 0..cols {
        for i in 0..rows {
            let (j0, j1) = (j.saturating_sub(1), (j + 1).min(cols - 1));
            let (i0, i1) = (i.saturating_sub(1), (i + 1).min(rows - 1));
            let along_cols = diff(point(i, j1), point(i, j0));
            let along_rows = diff(point(i1, j), point(i0, j));
            let mut n = normalize(cross(along_cols, along_rows));
            if !n.iter().all(|v| v.is_finite()) || n == [0.0; 3] {
                n = [0.0, 0.0, 1.0];
            }
            if dot(n, viewer) < 0.0 {
                n = [-n[0], -n[1], -n[2]];
            }
            let d = dot(n, light).max(0.0);
            let s = dot(n, half).max(0.0).powi(SHININESS);
            let r = AMBIENT + DIFFUSE * d + SPECULAR * s;
            out.push(if r.is_finite() {
                r.clamp(0.0, 1.0)
            } else {
                AMBIENT
            });
        }
    }
    out
}

/// Unit vector pointing from the origin towards a viewer at azimuth `az`
/// (degrees, counter-clockwise from the negative y axis) and elevation `el`.
fn direction(az: f64, el: f64) -> [f64; 3] {
    let (az, el) = (az.to_radians(), el.to_radians());
    [el.cos() * az.sin(), -el.cos() * az.cos(), el.sin()]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalize(a: [f64; 3]) -> [f64; 3] {
    let len = dot(a, a).sqrt();
    if len > 0.0 {
        [a[0] / len, a[1] / len, a[2] / len]
    } else {
        [0.0; 3]
    }
}

/// Expand a coordinate vector to a column-major `rows x cols` grid: X varies
/// along columns, Y along rows.
fn expand(v: &[f64], rows: usize, cols: usize, is_x: bool) -> Vec<f64> {
    let mut out = Vec::with_capacity(rows * cols);
    for j in 0..cols {
        for i in 0..rows {
            out.push(if is_x { v[j] } else { v[i] });
        }
    }
    out
}

fn coordinate_grid(t: &Tensor, rows: usize, cols: usize, is_x: bool) -> BuiltinResult<Vec<f64>> {
    let data = t.materialize_f64();
    if t.rows() == rows && t.cols() == cols {
        return Ok(data);
    }
    if t.rows() == 1 || t.cols() == 1 {
        let (expected, error) = if is_x {
            (cols, &SURFL_ERROR_X_LENGTH)
        } else {
            (rows, &SURFL_ERROR_Y_LENGTH)
        };
        if data.len() != expected {
            return Err(surfl_error(
                error,
                format!("got {} elements, expected {expected}", data.len()),
            ));
        }
        return Ok(expand(&data, rows, cols, is_x));
    }
    Err(surfl_error(
        &SURFL_ERROR_SIZE,
        format!(
            "{} is {}x{} but Z is {rows}x{cols}",
            if is_x { "X" } else { "Y" },
            t.rows(),
            t.cols()
        ),
    ))
}

fn is_numeric(value: &Value) -> bool {
    matches!(
        value,
        Value::Num(_)
            | Value::Int(_)
            | Value::Bool(_)
            | Value::Tensor(_)
            | Value::LogicalArray(_)
            | Value::Complex(_, _)
            | Value::ComplexTensor(_)
    )
}

fn real_matrix(value: &Value, name: &str) -> BuiltinResult<Tensor> {
    match value {
        Value::Complex(_, _) | Value::ComplexTensor(_) => Err(surfl_error(
            &SURFL_ERROR_UNSUPPORTED_OPTION,
            format!("complex {name} is not supported"),
        )),
        v if is_numeric(v) => {
            let t = Tensor::try_from(v).map_err(|e| surfl_error(&SURFL_ERROR_INVALID_INPUT, &e))?;
            if t.shape.iter().skip(2).any(|&d| d != 1) || t.is_empty() {
                return Err(surfl_error(
                    &SURFL_ERROR_INVALID_INPUT,
                    format!("{name} must be a non-empty 2-D array"),
                ));
            }
            Ok(t)
        }
        _ => Err(surfl_error(
            &SURFL_ERROR_INVALID_INPUT,
            format!("{name} must be a real numeric array"),
        )),
    }
}

fn internal(err: RuntimeError) -> RuntimeError {
    if err.identifier().is_some() {
        return err;
    }
    surfl_error(&SURFL_ERROR_INTERNAL, err.message())
}

fn surfl_error(error: &'static BuiltinErrorDescriptor, detail: impl AsRef<str>) -> RuntimeError {
    let mut builder = build_runtime_error(format!("{}: {}", error.message, detail.as_ref()))
        .with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

#[cfg(test)]
mod surfl_tests {
    use super::{surfl_builtin, surfl_reflectance, SURFL_VIEW};
    use crate::builtins::plotting::ops::pcolor::pcolor_tests::{
        call, identifier, matrix, prop, setup, shaped, string, surface_of, text,
    };
    use crate::BuiltinResult;
    use futures::executor::block_on;
    use runmat_value::{ComplexTensor, Tensor, Value};

    fn direct(args: &[Value]) -> BuiltinResult<f64> {
        block_on(surfl_builtin(args.to_vec()))
    }

    const MAGIC3: [&[f64]; 3] = [&[8.0, 1.0, 6.0], &[3.0, 5.0, 7.0], &[4.0, 9.0, 2.0]];

    fn row(v: &[f64]) -> Value {
        matrix(&[v])
    }

    /// Common observed checks: Type, full XData/YData grids, ZData, CData size.
    fn assert_surface(h: f64, x: &Value, y: &Value, z: &Value) {
        assert_eq!(string(prop(h, "Type")), "surface");
        assert_eq!(shaped(&prop(h, "XData")), shaped(x));
        assert_eq!(shaped(&prop(h, "YData")), shaped(y));
        assert_eq!(shaped(&prop(h, "ZData")), shaped(z));
        let (zshape, _) = shaped(z);
        let grid = surface_of(h).color_grid.expect("CData colour grid");
        assert_eq!(grid.len(), zshape[1], "CData columns");
        assert!(grid.iter().all(|c| c.len() == zshape[0]), "CData rows");
    }

    // ---- observed (normative) ----

    /// [surfl.output-class, surfl.data] case `z-only`.
    #[test]
    fn observed_z_only() {
        let _guard = setup();
        let h = direct(&[matrix(&MAGIC3)]).unwrap();
        assert_surface(
            h,
            &matrix(&[&[1.0, 2.0, 3.0], &[1.0, 2.0, 3.0], &[1.0, 2.0, 3.0]]),
            &matrix(&[&[1.0, 1.0, 1.0], &[2.0, 2.0, 2.0], &[3.0, 3.0, 3.0]]),
            &matrix(&MAGIC3),
        );
    }

    /// [surfl.data] cases `xyz-vectors-3x3`, `xyz-vectors-4x3`,
    /// `xyz-matrices-3x3`.
    #[test]
    fn observed_vector_and_matrix_coordinates_expand_to_grids() {
        let _guard = setup();
        let x_grid3 = matrix(&[&[1.0, 2.0, 3.0], &[1.0, 2.0, 3.0], &[1.0, 2.0, 3.0]]);
        let y_grid3 = matrix(&[
            &[10.0, 10.0, 10.0],
            &[20.0, 20.0, 20.0],
            &[30.0, 30.0, 30.0],
        ]);
        let h = direct(&[
            row(&[1.0, 2.0, 3.0]),
            row(&[10.0, 20.0, 30.0]),
            matrix(&MAGIC3),
        ])
        .unwrap();
        assert_surface(h, &x_grid3, &y_grid3, &matrix(&MAGIC3));

        let z43 = matrix(&[
            &[1.0, 5.0, 9.0],
            &[2.0, 6.0, 10.0],
            &[3.0, 7.0, 11.0],
            &[4.0, 8.0, 12.0],
        ]);
        let h = direct(&[
            row(&[1.0, 2.0, 3.0]),
            row(&[10.0, 20.0, 30.0, 40.0]),
            z43.clone(),
        ])
        .unwrap();
        assert_surface(
            h,
            &matrix(&[
                &[1.0, 2.0, 3.0],
                &[1.0, 2.0, 3.0],
                &[1.0, 2.0, 3.0],
                &[1.0, 2.0, 3.0],
            ]),
            &matrix(&[
                &[10.0, 10.0, 10.0],
                &[20.0, 20.0, 20.0],
                &[30.0, 30.0, 30.0],
                &[40.0, 40.0, 40.0],
            ]),
            &z43,
        );

        let h = direct(&[x_grid3.clone(), y_grid3.clone(), matrix(&MAGIC3)]).unwrap();
        assert_surface(h, &x_grid3, &y_grid3, &matrix(&MAGIC3));
    }

    /// [surfl.cdata] case `cdata-range`: CData is a reflectance map, not Z.
    /// The observed numeric range for one MATLAB input is not reproduced
    /// (deviation D-058-7); RunMat's values lie in [0, 1] and vary.
    #[test]
    fn observed_cdata_is_not_z() {
        let (rows, cols) = (10, 10);
        let mut x = Vec::new();
        let mut y = Vec::new();
        let mut z = Vec::new();
        for j in 0..cols {
            for i in 0..rows {
                let (u, v) = (-3.0 + 6.0 * j as f64 / 9.0, -3.0 + 6.0 * i as f64 / 9.0);
                x.push(u);
                y.push(v);
                z.push((u * 0.8).sin() * (v * 0.6).cos() * 3.0);
            }
        }
        let r = surfl_reflectance(&x, &y, &z, rows, cols, SURFL_VIEW);
        assert_eq!(r.len(), rows * cols);
        assert!(r.iter().all(|v| (0.0..=1.0).contains(v)), "in [0, 1]");
        assert_ne!(r, z, "CData is not Z");
        let (lo, hi) = r
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), &v| {
                (a.min(v), b.max(v))
            });
        assert!(hi - lo > 0.05, "reflectance varies over a curved surface");
    }

    /// [surfl.view] case `view`.
    #[test]
    fn observed_view() {
        let _guard = setup();
        let h = direct(&[matrix(&MAGIC3)]).unwrap();
        let view = shaped(&call("view", &[prop(h, "Parent")]).expect("view")).1;
        assert_eq!(view, vec![-37.5, 30.0]);
    }

    /// [surfl.face] case `face`; EdgeColor is a documented choice.
    #[test]
    fn observed_face_color_flat_and_edge_choice() {
        let _guard = setup();
        let h = direct(&[matrix(&MAGIC3)]).unwrap();
        assert_eq!(string(prop(h, "FaceColor")), "flat");
        assert_eq!(string(prop(h, "EdgeColor")), "flat");
    }

    /// [surfl.error-z-2x3] cases `err-z-2x3`, `err-xyz-vectors-2row`,
    /// `err-xyz-matrices-2row`.
    #[test]
    fn observed_two_row_z_errors() {
        let _guard = setup();
        let z = matrix(&[&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]]);
        let err = direct(std::slice::from_ref(&z)).unwrap_err();
        assert_eq!(identifier(&err), "RunMat:surfl:InvalidInput");
        let err = direct(&[row(&[1.0, 2.0, 3.0]), row(&[10.0, 20.0]), z.clone()]).unwrap_err();
        assert_eq!(identifier(&err), "RunMat:surfl:InvalidInput");
        let err = direct(&[
            matrix(&[&[1.0, 2.0, 3.0], &[1.0, 2.0, 3.0]]),
            matrix(&[&[10.0, 10.0, 10.0], &[20.0, 20.0, 20.0]]),
            z,
        ])
        .unwrap_err();
        assert_eq!(identifier(&err), "RunMat:surfl:InvalidInput");
    }

    /// [surfl.error-size] case `err-size`.
    #[test]
    fn observed_x_length_mismatch_errors() {
        let _guard = setup();
        let err =
            direct(&[row(&[1.0, 2.0]), row(&[10.0, 20.0, 30.0]), matrix(&MAGIC3)]).unwrap_err();
        assert_eq!(
            identifier(&err),
            "RunMat:xyzchk:lengthXDoesNotMatchNumColumnsZ"
        );
    }

    /// [surfl.error-char] case `err-char`.
    #[test]
    fn observed_single_char_argument_errors() {
        let _guard = setup();
        let err = direct(&[text("abc")]).unwrap_err();
        assert_eq!(identifier(&err), "RunMat:narginchk:notEnoughInputs");
    }

    // ---- documented choices for unobserved forms ----

    /// (choice) Y length and matrix-shape mismatches, no arguments, two or
    /// four arguments, complex Z and Z with fewer than 3 columns.
    #[test]
    fn unresolved_choice_unobserved_forms() {
        let _guard = setup();
        let err =
            direct(&[row(&[1.0, 2.0, 3.0]), row(&[10.0, 20.0]), matrix(&MAGIC3)]).unwrap_err();
        assert_eq!(
            identifier(&err),
            "RunMat:xyzchk:lengthYDoesNotMatchNumRowsZ"
        );
        let err = direct(&[
            matrix(&[&[1.0, 2.0], &[1.0, 2.0]]),
            row(&[10.0, 20.0, 30.0]),
            matrix(&MAGIC3),
        ])
        .unwrap_err();
        assert_eq!(identifier(&err), "RunMat:surfl:InputSizeMismatch");
        let err = direct(&[]).unwrap_err();
        assert_eq!(identifier(&err), "RunMat:narginchk:notEnoughInputs");
        let err = direct(&[matrix(&MAGIC3), row(&[1.0, 1.0, 1.0])]).unwrap_err();
        assert_eq!(identifier(&err), "RunMat:surfl:UnsupportedOption");
        let complex = Value::ComplexTensor(
            ComplexTensor::new(vec![(1.0, 1.0); 9], vec![3, 3]).expect("complex"),
        );
        let err = direct(&[complex]).unwrap_err();
        assert_eq!(identifier(&err), "RunMat:surfl:UnsupportedOption");
        let z32 =
            Value::Tensor(Tensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![3, 2]).unwrap());
        let err = direct(&[z32]).unwrap_err();
        assert_eq!(identifier(&err), "RunMat:surfl:UnsupportedOption");
    }

    /// (choice) The reflectance model is deterministic and flat surfaces are
    /// uniformly lit.
    #[test]
    fn unresolved_choice_flat_surface_uniform_reflectance() {
        let x = vec![1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 3.0, 3.0, 3.0];
        let y = vec![1.0, 2.0, 3.0, 1.0, 2.0, 3.0, 1.0, 2.0, 3.0];
        let z = vec![5.0; 9];
        let r = surfl_reflectance(&x, &y, &z, 3, 3, SURFL_VIEW);
        assert!(r.iter().all(|v| (v - r[0]).abs() < 1e-12));
        assert_eq!(r, surfl_reflectance(&x, &y, &z, 3, 3, SURFL_VIEW));
    }

    /// Dispatcher path: callable by name.
    #[test]
    fn registered_through_dispatcher() {
        let _guard = setup();
        let h = call("surfl", &[matrix(&MAGIC3)]).expect("surfl");
        assert_eq!(string(prop(shaped(&h).1[0], "Type")), "surface");
    }
}
