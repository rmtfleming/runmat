//! `tiledlayout` builtin: create a tiled chart layout in a figure
//! (spec 051-graphics-n2).
//!
//! Normative behaviour comes from the approved `tiledlayout` 0.2.0 export:
//! fixed `m`-by-`n` and `'flow'` arrangements, the `TileSpacing` and
//! `Padding` options, an explicit figure parent, the layout's observed
//! properties, and the observed errors. The layout is a RunMat handle object
//! of class `matlab.graphics.layout.TiledChartLayout` whose properties are
//! read with dot access. Tiles are placed on the figure's existing subplot
//! grid by [`add_tile`]; there is no public `nexttile` builtin because no
//! approved `nexttile` specification exists (deviation D-051-4). See
//! `specs/051-graphics-n2/spec.md`.

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{CharArray, HandleRef, ObjectInstance, Tensor, Value};

use crate::builtins::plotting::properties::{resolve_plot_handle, PlotHandle};
use crate::builtins::plotting::state::{
    configure_subplot, current_axes_state, encode_axes_handle, select_current_figure_if_exists,
};
use crate::builtins::plotting::FigureHandle;
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

/// RunMat guard on each grid dimension.
const MAX_GRID_DIMENSION: usize = 10_000;

const BUILTIN_NAME: &str = "tiledlayout";

/// Class name of the returned layout object.
pub const LAYOUT_CLASS_NAME: runmat_types::StaticClassIdentity =
    runmat_types::StaticClassIdentity::new("matlab.graphics.layout.TiledChartLayout");

const TL_INPUT_PARENT: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "parent",
    ty: BuiltinParamType::NumericScalar,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Figure handle that receives the layout.",
};
const TL_INPUT_M: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "m",
    ty: BuiltinParamType::SizeArg,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Number of tile rows (positive integer).",
};
const TL_INPUT_N: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "n",
    ty: BuiltinParamType::SizeArg,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Number of tile columns (positive integer).",
};
const TL_INPUT_FLOW: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "arrangement",
    ty: BuiltinParamType::StringScalar,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "'flow': the grid grows as tiles are added.",
};
const TL_INPUT_OPTIONS: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "Name, Value",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Variadic,
    default: None,
    description: "TileSpacing ('loose', 'compact') and Padding ('loose', 'tight').",
};
const TL_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "t",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "TiledChartLayout handle object.",
}];
const TL_INPUTS_GRID: [BuiltinParamDescriptor; 3] = [TL_INPUT_M, TL_INPUT_N, TL_INPUT_OPTIONS];
const TL_INPUTS_FLOW: [BuiltinParamDescriptor; 2] = [TL_INPUT_FLOW, TL_INPUT_OPTIONS];
const TL_INPUTS_PARENT: [BuiltinParamDescriptor; 4] =
    [TL_INPUT_PARENT, TL_INPUT_M, TL_INPUT_N, TL_INPUT_OPTIONS];
const TL_SIGNATURES: [BuiltinSignatureDescriptor; 3] = [
    BuiltinSignatureDescriptor {
        label: "t = tiledlayout(m, n, Name, Value)",
        inputs: &TL_INPUTS_GRID,
        outputs: &TL_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "t = tiledlayout('flow', Name, Value)",
        inputs: &TL_INPUTS_FLOW,
        outputs: &TL_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "t = tiledlayout(parent, m, n, Name, Value)",
        inputs: &TL_INPUTS_PARENT,
        outputs: &TL_OUTPUT,
    },
];

const fn tl_error_descriptor(
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

const ERR_NOT_ENOUGH_INPUTS: BuiltinErrorDescriptor = tl_error_descriptor(
    "RM.TILEDLAYOUT.NOT_ENOUGH_INPUTS",
    "RunMat:narginchk:notEnoughInputs",
    "No arguments, or a row count without a column count.",
    "tiledlayout: not enough input arguments",
);
const ERR_INVALID_GRID: BuiltinErrorDescriptor = tl_error_descriptor(
    "RM.TILEDLAYOUT.INVALID_GRID",
    "RunMat:tiledchartlayout:InvalidGridSize",
    "A grid dimension is zero, negative, non-integer or non-finite.",
    "tiledlayout: grid dimensions must be positive integers",
);
const ERR_INVALID_ENUM: BuiltinErrorDescriptor = tl_error_descriptor(
    "RM.TILEDLAYOUT.INVALID_ENUM",
    "RunMat:datatypes:InvalidEnumValue",
    "A TileSpacing or Padding value is not one RunMat recognises.",
    "tiledlayout: invalid option value",
);
const ERR_INVALID_INPUT: BuiltinErrorDescriptor = tl_error_descriptor(
    "RM.TILEDLAYOUT.INVALID_INPUT",
    "RunMat:tiledlayout:InvalidInput",
    "Name-value arguments are unpaired or not text, or the parent is not a figure.",
    "tiledlayout: invalid input",
);
const ERR_UNSUPPORTED_OPTION: BuiltinErrorDescriptor = tl_error_descriptor(
    "RM.TILEDLAYOUT.UNSUPPORTED_OPTION",
    "RunMat:tiledlayout:UnsupportedOption",
    "An arrangement other than m-by-n or 'flow', an option other than TileSpacing/Padding, or a tile beyond a fixed grid.",
    "tiledlayout: unsupported option",
);
const ERR_INTERNAL: BuiltinErrorDescriptor = tl_error_descriptor(
    "RM.TILEDLAYOUT.INTERNAL",
    "RunMat:tiledlayout:Internal",
    "The layout object or figure state cannot be updated.",
    "tiledlayout: internal error",
);
const TL_ERRORS: [BuiltinErrorDescriptor; 6] = [
    ERR_NOT_ENOUGH_INPUTS,
    ERR_INVALID_GRID,
    ERR_INVALID_ENUM,
    ERR_INVALID_INPUT,
    ERR_UNSUPPORTED_OPTION,
    ERR_INTERNAL,
];

pub const TILEDLAYOUT_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &TL_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &TL_ERRORS,
};

#[runtime_builtin(
    name = "tiledlayout",
    category = "plotting",
    summary = "Create a tiled chart layout for arranging axes in a grid.",
    keywords = "tiledlayout,tiles,layout,grid,axes,plotting",
    descriptor(crate::builtins::plotting::ops::tiledlayout::TILEDLAYOUT_DESCRIPTOR),
    builtin_path = "crate::builtins::plotting::ops::tiledlayout"
)]
pub async fn tiledlayout_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    let mut gathered = Vec::with_capacity(args.len());
    for value in &args {
        gathered.push(
            gather_if_needed_async(value)
                .await
                .map_err(|err| tl_error(&ERR_INTERNAL, err.message()))?,
        );
    }
    let spec = parse_layout_args(&gathered)?;
    let figure = match spec.parent {
        Some(figure) => figure,
        None => current_axes_state().handle,
    };
    let (grid, arrangement) = match spec.arrangement {
        Arrangement::Fixed(rows, cols) => ([rows, cols], "fixed"),
        Arrangement::Flow => ([1, 1], "flow"),
    };
    let mut object = ObjectInstance::new(LAYOUT_CLASS_NAME.to_string());
    object
        .properties
        .insert("GridSize".to_string(), grid_value(grid));
    object
        .properties
        .insert("TileArrangement".to_string(), char_row(arrangement));
    object
        .properties
        .insert("TileSpacing".to_string(), char_row(&spec.tile_spacing));
    object
        .properties
        .insert("Padding".to_string(), char_row(&spec.padding));
    object
        .properties
        .insert("Children".to_string(), handles_column(&[]));
    object
        .properties
        .insert("Parent".to_string(), Value::Num(figure.as_u32() as f64));
    let target = runmat_gc::gc_allocate(Value::Object(object))
        .map_err(|err| tl_error(&ERR_INTERNAL, err.to_string()))?;
    Ok(Value::HandleObject(HandleRef {
        class_name: LAYOUT_CLASS_NAME.into(),
        target,
        valid: true,
    }))
}

#[derive(Debug, Clone, Copy)]
enum Arrangement {
    Fixed(usize, usize),
    Flow,
}

#[derive(Debug)]
struct LayoutSpec {
    parent: Option<FigureHandle>,
    arrangement: Arrangement,
    tile_spacing: String,
    padding: String,
}

fn numeric_scalar(value: &Value) -> Option<f64> {
    match value {
        Value::Num(v) => Some(*v),
        Value::Int(i) => Some(i.to_f64()),
        Value::Tensor(t) => {
            let data = t.materialize_f64();
            (data.len() == 1).then(|| data[0])
        }
        _ => None,
    }
}

fn text_scalar(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::CharArray(c) if c.rows == 1 => Some(c.data.iter().collect()),
        Value::StringArray(sa) if sa.data.len() == 1 => Some(sa.data[0].clone()),
        _ => None,
    }
}

fn parse_layout_args(args: &[Value]) -> BuiltinResult<LayoutSpec> {
    if args.is_empty() {
        return Err(tl_error(
            &ERR_NOT_ENOUGH_INPUTS,
            "expected tiledlayout(m, n) or tiledlayout('flow')",
        ));
    }
    let is_num = |idx: usize| args.get(idx).and_then(numeric_scalar).is_some();
    let is_flow = |idx: usize| {
        args.get(idx)
            .and_then(text_scalar)
            .is_some_and(|t| t.trim().eq_ignore_ascii_case("flow"))
    };
    // A leading figure: three leading numbers, or a number before 'flow'.
    let has_parent = (is_num(0) && is_num(1) && is_num(2)) || (is_num(0) && is_flow(1));
    let mut idx = 0;
    let parent = if has_parent {
        idx = 1;
        Some(parent_figure(&args[0])?)
    } else {
        None
    };
    let arrangement = if let Some(text) = text_scalar(&args[idx]) {
        if !text.trim().eq_ignore_ascii_case("flow") {
            return Err(tl_error(
                &ERR_UNSUPPORTED_OPTION,
                format!("arrangement '{text}' is not supported; use m, n or 'flow'"),
            ));
        }
        idx += 1;
        Arrangement::Flow
    } else if let Some(m) = numeric_scalar(&args[idx]) {
        let Some(n) = args.get(idx + 1).and_then(numeric_scalar) else {
            return Err(tl_error(
                &ERR_NOT_ENOUGH_INPUTS,
                "the number of columns n is missing",
            ));
        };
        idx += 2;
        Arrangement::Fixed(grid_dimension(m)?, grid_dimension(n)?)
    } else {
        return Err(tl_error(
            &ERR_INVALID_INPUT,
            "expected grid dimensions m, n or 'flow'",
        ));
    };

    let mut tile_spacing = "loose".to_string();
    let mut padding = "loose".to_string();
    let rest = &args[idx..];
    if !rest.len().is_multiple_of(2) {
        return Err(tl_error(
            &ERR_INVALID_INPUT,
            "name-value arguments must come in pairs",
        ));
    }
    for pair in rest.chunks(2) {
        let Some(name) = text_scalar(&pair[0]) else {
            return Err(tl_error(&ERR_INVALID_INPUT, "option names must be text"));
        };
        let value = text_scalar(&pair[1]).map(|v| v.trim().to_ascii_lowercase());
        match name.trim().to_ascii_lowercase().as_str() {
            "tilespacing" => {
                tile_spacing = enum_value(value, &["loose", "compact"], "TileSpacing")?;
            }
            "padding" => {
                padding = enum_value(value, &["loose", "tight"], "Padding")?;
            }
            _ => {
                return Err(tl_error(
                    &ERR_UNSUPPORTED_OPTION,
                    format!("option '{name}' is not supported"),
                ))
            }
        }
    }
    Ok(LayoutSpec {
        parent,
        arrangement,
        tile_spacing,
        padding,
    })
}

fn enum_value(value: Option<String>, allowed: &[&str], property: &str) -> BuiltinResult<String> {
    match value {
        Some(v) if allowed.contains(&v.as_str()) => Ok(v),
        _ => Err(tl_error(
            &ERR_INVALID_ENUM,
            format!("{property} must be one of: {}", allowed.join(", ")),
        )),
    }
}

fn grid_dimension(value: f64) -> BuiltinResult<usize> {
    if !value.is_finite() || value < 1.0 || value.fract() != 0.0 {
        return Err(tl_error(&ERR_INVALID_GRID, format!("got {value}")));
    }
    if value > MAX_GRID_DIMENSION as f64 {
        return Err(tl_error(&ERR_INVALID_GRID, "grid dimension is too large"));
    }
    Ok(value as usize)
}

fn parent_figure(value: &Value) -> BuiltinResult<FigureHandle> {
    match resolve_plot_handle(value, BUILTIN_NAME) {
        Ok(PlotHandle::Figure(handle)) => Ok(handle),
        _ => Err(tl_error(
            &ERR_INVALID_INPUT,
            "the parent must be an existing figure",
        )),
    }
}

fn grid_value(grid: [usize; 2]) -> Value {
    Value::Tensor(
        Tensor::new(vec![grid[0] as f64, grid[1] as f64], vec![1, 2]).expect("1x2 grid size"),
    )
}

fn handles_column(handles: &[f64]) -> Value {
    Value::Tensor(Tensor::new(handles.to_vec(), vec![handles.len(), 1]).expect("handle column"))
}

fn char_row(text: &str) -> Value {
    Value::CharArray(CharArray::new_row(text))
}

/// Flow grid for `k` tiles: `cols = ceil(sqrt(k))`, `rows = ceil(k / cols)`
/// (documented choice; observed at k = 0 and k = 3 only).
#[allow(dead_code)]
fn flow_grid(k: usize) -> [usize; 2] {
    if k <= 1 {
        return [1, 1];
    }
    let mut cols = (k as f64).sqrt().ceil() as usize;
    while cols * cols < k {
        cols += 1;
    }
    [k.div_ceil(cols), cols]
}

/// Add one tile (axes) to `layout` and make it the current axes. Returns the
/// new axes handle. This is the layout model a future `nexttile` builtin
/// would call; it is not exposed to MATLAB code (deviation D-051-4).
#[allow(dead_code)]
pub(crate) fn add_tile(layout: &Value) -> BuiltinResult<f64> {
    let handle = layout_handle(layout)
        .ok_or_else(|| tl_error(&ERR_INVALID_INPUT, "expected a TiledChartLayout object"))?;
    let read = |name: &str| layout_property(layout, name);
    let numbers = |value: Value| -> Vec<f64> {
        match value {
            Value::Num(v) => vec![v],
            Value::Tensor(t) => t.materialize_f64().to_vec(),
            _ => Vec::new(),
        }
    };
    let grid = numbers(read("GridSize")?);
    let mut children = numbers(read("Children")?);
    let arrangement = match read("TileArrangement")? {
        Value::CharArray(c) => c.data.iter().collect::<String>(),
        _ => String::new(),
    };
    let parent = numbers(read("Parent")?)
        .first()
        .copied()
        .ok_or_else(|| tl_error(&ERR_INTERNAL, "layout has no parent figure"))?;
    let figure = FigureHandle::from(parent as u32);
    let k = children.len() + 1;
    let [rows, cols] = if arrangement == "flow" {
        flow_grid(k)
    } else {
        let (rows, cols) = (grid[0] as usize, grid[1] as usize);
        if k > rows * cols {
            return Err(tl_error(
                &ERR_UNSUPPORTED_OPTION,
                format!("a {rows}-by-{cols} layout has no room for tile {k}"),
            ));
        }
        [rows, cols]
    };
    select_current_figure_if_exists(figure)
        .map_err(|err| tl_error(&ERR_INTERNAL, err.to_string()))?;
    configure_subplot(rows, cols, k - 1).map_err(|err| tl_error(&ERR_INTERNAL, err.to_string()))?;
    let axes = encode_axes_handle(figure, k - 1);
    children.push(axes);
    runmat_gc::gc_with_value_mut(&handle.target, |target| {
        if let Value::Object(object) = target {
            object
                .properties
                .insert("GridSize".to_string(), grid_value([rows, cols]));
            object
                .properties
                .insert("Children".to_string(), handles_column(&children));
        }
    })
    .map_err(|err| tl_error(&ERR_INTERNAL, err.to_string()))?;
    Ok(axes)
}

/// Read a property of a layout object (dot-access storage).
#[allow(dead_code)]
pub(crate) fn layout_property(layout: &Value, name: &str) -> BuiltinResult<Value> {
    let handle = layout_handle(layout)
        .ok_or_else(|| tl_error(&ERR_INVALID_INPUT, "expected a TiledChartLayout object"))?;
    runmat_gc::gc_with_value(&handle.target, |value| match value {
        Value::Object(object) => object.properties.get(name).cloned(),
        _ => None,
    })
    .map_err(|err| tl_error(&ERR_INTERNAL, err.to_string()))?
    .ok_or_else(|| tl_error(&ERR_INVALID_INPUT, format!("no layout property '{name}'")))
}

#[allow(dead_code)]
fn layout_handle(value: &Value) -> Option<&HandleRef> {
    match value {
        Value::HandleObject(handle)
            if handle.class_name == LAYOUT_CLASS_NAME && crate::is_handle_valid(handle) =>
        {
            Some(handle)
        }
        _ => None,
    }
}

fn tl_error(error: &'static BuiltinErrorDescriptor, detail: impl AsRef<str>) -> RuntimeError {
    let mut builder = build_runtime_error(format!("{}: {}", error.message, detail.as_ref()))
        .with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

/// Spec 051 (`tiledlayout` 0.2.0). `observed_*` tests reproduce confirmed
/// observations; `unresolved_choice_*`, `deviation_*` and `rejects_*` tests
/// pin RunMat's documented choices and are not claims about MATLAB.
#[cfg(test)]
mod tiledlayout_tests {
    use super::{add_tile, layout_property};
    use crate::builtins::plotting::state::{
        clear_figure, current_figure_handle, decode_axes_handle, reset_hold_state_for_run,
    };
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
        block_on(super::tiledlayout_builtin(args.to_vec()))
    }

    fn text(s: &str) -> Value {
        Value::CharArray(CharArray::new_row(s))
    }

    fn identifier(err: &RuntimeError) -> String {
        err.identifier().unwrap_or_default().to_string()
    }

    fn numbers(value: &Value) -> Vec<f64> {
        match value {
            Value::Num(v) => vec![*v],
            other => Tensor::try_from(other)
                .expect("numeric value")
                .materialize_f64()
                .to_vec(),
        }
    }

    fn chars(value: &Value) -> String {
        match value {
            Value::CharArray(c) => c.data.iter().collect(),
            other => panic!("expected char row, got {other:?}"),
        }
    }

    fn grid(t: &Value) -> Vec<f64> {
        numbers(&layout_property(t, "GridSize").expect("GridSize"))
    }

    fn n_children(t: &Value) -> usize {
        numbers(&layout_property(t, "Children").expect("Children")).len()
    }

    /// Observed property set for a layout: class, GridSize, TileSpacing,
    /// Padding, TileArrangement, number of children.
    fn assert_layout(
        t: &Value,
        grid_size: [f64; 2],
        spacing: &str,
        padding: &str,
        arrangement: &str,
        children: usize,
    ) {
        let class = call("class", std::slice::from_ref(t)).expect("class");
        let class = match class {
            Value::String(s) => s,
            other => chars(&other),
        };
        assert_eq!(class, "matlab.graphics.layout.TiledChartLayout");
        assert_eq!(grid(t), grid_size.to_vec());
        assert_eq!(
            chars(&layout_property(t, "TileSpacing").expect("TileSpacing")),
            spacing
        );
        assert_eq!(
            chars(&layout_property(t, "Padding").expect("Padding")),
            padding
        );
        assert_eq!(
            chars(&layout_property(t, "TileArrangement").expect("TileArrangement")),
            arrangement
        );
        assert_eq!(n_children(t), children);
    }

    #[test]
    fn observed_grid_layout_properties() {
        let _g = setup();
        let t = call("tiledlayout", &[Value::Num(2.0), Value::Num(3.0)]).expect("tiledlayout(2,3)");
        assert_layout(&t, [2.0, 3.0], "loose", "loose", "fixed", 0);
    }

    #[test]
    fn deviation_properties_read_through_field_access() {
        // D-051-7: properties are stored on the handle object and read by
        // member access (`t.GridSize`, `load_member`), not by `get`.
        let _g = setup();
        let t = call("tiledlayout", &[Value::Num(2.0), Value::Num(3.0)]).expect("tiledlayout");
        let grid = block_on(crate::object::resolve::load_member(
            t.clone(),
            "GridSize".to_string(),
            false,
            None,
        ))
        .expect("t.GridSize");
        assert_eq!(numbers(&grid), vec![2.0, 3.0]);
    }

    #[test]
    fn observed_tiles_add_one_child_each() {
        let _g = setup();
        let t = call("tiledlayout", &[Value::Num(2.0), Value::Num(2.0)]).expect("tiledlayout(2,2)");
        let first = add_tile(&t).expect("tile 1");
        let second = add_tile(&t).expect("tile 2");
        assert_ne!(first, second);
        assert_layout(&t, [2.0, 2.0], "loose", "loose", "fixed", 2);
        // The newest tile is the current axes.
        let gca = numbers(&call("gca", &[]).expect("gca"))[0];
        assert_eq!(gca, second);
        assert_eq!(
            numbers(&layout_property(&t, "Children").expect("Children")),
            vec![first, second]
        );
    }

    #[test]
    fn observed_flow_layout_and_growth() {
        let _g = setup();
        let t = call("tiledlayout", &[text("flow")]).expect("tiledlayout('flow')");
        assert_layout(&t, [1.0, 1.0], "loose", "loose", "flow", 0);
        for _ in 0..3 {
            add_tile(&t).expect("flow tile");
        }
        assert_layout(&t, [2.0, 2.0], "loose", "loose", "flow", 3);
    }

    #[test]
    fn observed_spacing_and_padding_stored() {
        let _g = setup();
        let t = call(
            "tiledlayout",
            &[
                Value::Num(1.0),
                Value::Num(2.0),
                text("TileSpacing"),
                text("compact"),
                text("Padding"),
                text("tight"),
            ],
        )
        .expect("tiledlayout with options");
        assert_layout(&t, [1.0, 2.0], "compact", "tight", "fixed", 0);
    }

    #[test]
    fn observed_figure_parent() {
        let _g = setup();
        let f = call("figure", &[]).expect("figure");
        let other = call("figure", &[]).expect("another figure");
        assert_ne!(numbers(&f), numbers(&other));
        let t = call(
            "tiledlayout",
            &[f.clone(), Value::Num(1.0), Value::Num(1.0)],
        )
        .expect("tiledlayout(f,1,1)");
        assert_eq!(
            numbers(&layout_property(&t, "Parent").expect("Parent")),
            numbers(&f)
        );
        // A tile of this layout goes to the parent figure.
        let ax = add_tile(&t).expect("tile");
        let (fig, _) = decode_axes_handle(ax).expect("axes handle");
        assert_eq!(fig.as_u32() as f64, numbers(&f)[0]);
    }

    #[test]
    fn observed_single_numeric_argument_errors() {
        let _g = setup();
        let err = direct(&[Value::Num(3.0)]).expect_err("tiledlayout(3)");
        assert_eq!(identifier(&err), "RunMat:narginchk:notEnoughInputs");
    }

    #[test]
    fn observed_zero_grid_dimension_errors() {
        let _g = setup();
        let err = direct(&[Value::Num(0.0), Value::Num(2.0)]).expect_err("tiledlayout(0,2)");
        assert_eq!(identifier(&err), "RunMat:tiledchartlayout:InvalidGridSize");
    }

    #[test]
    fn observed_invalid_tile_spacing_errors() {
        let _g = setup();
        let err = direct(&[
            Value::Num(1.0),
            Value::Num(1.0),
            text("TileSpacing"),
            text("nope"),
        ])
        .expect_err("TileSpacing 'nope'");
        assert_eq!(identifier(&err), "RunMat:datatypes:InvalidEnumValue");
    }

    #[test]
    fn unresolved_choice_flow_growth_rule() {
        // tiledlayout.q-flow-growth: RunMat uses cols = ceil(sqrt(k)),
        // rows = ceil(k / cols); only k = 0 and k = 3 are observed.
        let _g = setup();
        let t = call("tiledlayout", &[text("flow")]).expect("flow");
        let mut sizes = Vec::new();
        for _ in 0..5 {
            add_tile(&t).expect("tile");
            sizes.push(grid(&t));
        }
        assert_eq!(
            sizes,
            vec![
                vec![1.0, 1.0],
                vec![1.0, 2.0],
                vec![2.0, 2.0],
                vec![2.0, 2.0],
                vec![2.0, 3.0],
            ]
        );
    }

    #[test]
    fn unresolved_choice_option_values_and_names() {
        let _g = setup();
        // Case-insensitive names and values; values stored lower-case.
        let t = call(
            "tiledlayout",
            &[
                text("flow"),
                Value::String("tilespacing".into()),
                Value::String("COMPACT".into()),
            ],
        )
        .expect("flow with options");
        assert_layout(&t, [1.0, 1.0], "compact", "loose", "flow", 0);
        // Padding values other than the observed ones are rejected.
        let err = direct(&[
            Value::Num(1.0),
            Value::Num(1.0),
            text("Padding"),
            text("nope"),
        ])
        .expect_err("Padding 'nope'");
        assert_eq!(identifier(&err), "RunMat:datatypes:InvalidEnumValue");
        // Negative, fractional and NaN grid sizes use the zero-size error.
        for (m, n) in [(-1.0, 2.0), (1.5, 2.0), (2.0, f64::NAN)] {
            let err = direct(&[Value::Num(m), Value::Num(n)]).expect_err("bad grid");
            assert_eq!(identifier(&err), "RunMat:tiledchartlayout:InvalidGridSize");
        }
        // No arguments, or m without n before options.
        let err = direct(&[]).expect_err("tiledlayout()");
        assert_eq!(identifier(&err), "RunMat:narginchk:notEnoughInputs");
        let err =
            direct(&[Value::Num(2.0), text("Padding"), text("tight")]).expect_err("m without n");
        assert_eq!(identifier(&err), "RunMat:narginchk:notEnoughInputs");
    }

    #[test]
    fn unresolved_choice_unobserved_forms_unsupported() {
        let _g = setup();
        for args in [
            vec![text("vertical")],
            vec![
                Value::Num(1.0),
                Value::Num(1.0),
                text("Units"),
                text("pixels"),
            ],
        ] {
            let err = direct(&args).expect_err("unsupported");
            assert_eq!(identifier(&err), "RunMat:tiledlayout:UnsupportedOption");
        }
        // A fixed grid has no room beyond m*n tiles.
        let t = call("tiledlayout", &[Value::Num(1.0), Value::Num(1.0)]).expect("1x1");
        add_tile(&t).expect("tile 1");
        let err = add_tile(&t).expect_err("tile 2 of 1x1");
        assert_eq!(identifier(&err), "RunMat:tiledlayout:UnsupportedOption");
    }

    #[test]
    fn rejects_malformed_inputs() {
        let _g = setup();
        let current = current_figure_handle().as_u32() as f64;
        for args in [
            vec![Value::Num(1.0), Value::Num(1.0), text("TileSpacing")],
            vec![
                Value::Num(1.0),
                Value::Num(1.0),
                Value::Num(3.0),
                Value::Num(4.0),
            ],
            vec![
                Value::Num(current + 9999.0),
                Value::Num(1.0),
                Value::Num(1.0),
            ],
        ] {
            let err = direct(&args).expect_err("malformed");
            assert_eq!(identifier(&err), "RunMat:tiledlayout:InvalidInput");
        }
        let err = add_tile(&Value::Num(1.0)).expect_err("not a layout");
        assert_eq!(identifier(&err), "RunMat:tiledlayout:InvalidInput");
    }
}
