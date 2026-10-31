//! `fplot` builtin: plot a function of x over an interval as a line
//! (spec 065).
//!
//! Normative behaviour comes from the approved `fplot` 0.2.0 export (claims
//! `fplot.signature-primary`, `fplot.object`, `fplot.default-interval`,
//! `fplot.interval`, `fplot.cobra-form`, `fplot.linespec`, `fplot.ax`,
//! `fplot.error-decreasing`, `fplot.error-bad-property`,
//! `fplot.warning-char-fn`). The function is sampled on a fixed grid (a
//! documented choice: `fplot.q-points` is unresolved) and drawn as a RunMat
//! line object, using the same line construction as `plot`. Default colours
//! follow the axes colour order (`fplot.q-theme`). See
//! `specs/065-graphics-n5/`.

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_plot::plots::LineStyle;
use runmat_value::{Tensor, Value};

use crate::builtins::math::optim::common::call_function;
use crate::builtins::plotting::op_common::apply_axes_target;
use crate::builtins::plotting::properties::{resolve_plot_handle, PlotHandle};
use crate::builtins::plotting::state::{
    self, current_axes_state, current_hold_enabled, line_color_for_axes_series_index,
    next_line_color_for_axes, PlotRenderOptions,
};
use crate::builtins::plotting::style::{parse_line_style_args, LineStyleParseOptions};
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const BUILTIN_NAME: &str = "fplot";

/// Default interval [fplot.default-interval].
const DEFAULT_INTERVAL: (f64, f64) = (-5.0, 5.0);
/// Number of sample points (documented choice; fplot.q-points).
pub(crate) const FPLOT_POINTS: usize = 201;
/// Default line width [fplot.linespec].
const DEFAULT_LINE_WIDTH: f32 = 0.5;

/// Name-value names accepted by RunMat's fplot line (documented choice).
const LINE_PROPERTIES: [&str; 8] = [
    "linewidth",
    "color",
    "linestyle",
    "marker",
    "markersize",
    "markeredgecolor",
    "markerfacecolor",
    "displayname",
];

const FPLOT_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "h",
    ty: BuiltinParamType::NumericScalar,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Handle of the created line object.",
}];
const FPLOT_INPUT_AX: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "ax",
    ty: BuiltinParamType::AxesHandle,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Axes to draw into.",
};
const FPLOT_INPUT_F: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "f",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description:
        "Function handle of one variable (a numeric constant or char expression is also accepted).",
};
const FPLOT_INPUT_INTERVAL: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "interval",
    ty: BuiltinParamType::NumericArray,
    arity: BuiltinParamArity::Optional,
    default: Some("[-5 5]"),
    description: "Increasing two-element interval [xmin xmax].",
};
const FPLOT_INPUT_STYLE: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "LineSpec, Name, Value",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Variadic,
    default: None,
    description: "Optional LineSpec followed by line name-value pairs (LineWidth, Color, ...).",
};
const FPLOT_INPUTS: [BuiltinParamDescriptor; 3] =
    [FPLOT_INPUT_F, FPLOT_INPUT_INTERVAL, FPLOT_INPUT_STYLE];
const FPLOT_INPUTS_AX: [BuiltinParamDescriptor; 4] = [
    FPLOT_INPUT_AX,
    FPLOT_INPUT_F,
    FPLOT_INPUT_INTERVAL,
    FPLOT_INPUT_STYLE,
];
const FPLOT_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "h = fplot(f, interval, LineSpec, Name, Value)",
        inputs: &FPLOT_INPUTS,
        outputs: &FPLOT_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "h = fplot(ax, ___)",
        inputs: &FPLOT_INPUTS_AX,
        outputs: &FPLOT_OUTPUT,
    },
];

const FPLOT_ERROR_RANGE: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.FPLOT.RANGE_VECTOR",
    identifier: Some("RunMat:fplot:RangeVector"),
    when: "The interval is not increasing (or not finite).",
    message: "fplot: the interval must be an increasing two-element vector",
};
const FPLOT_ERROR_BAD_PARAMETER: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.FPLOT.INVALID_PARAMETER",
    identifier: Some("RunMat:fplot:InvalidParameter"),
    when: "A name-value name is not a line property, or a text argument is neither a property nor a LineSpec.",
    message: "fplot: invalid parameter",
};
const FPLOT_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.FPLOT.INVALID_INPUT",
    identifier: Some("RunMat:fplot:InvalidInput"),
    when:
        "The function is missing or empty, a property has no value, or a property value is invalid.",
    message: "fplot: invalid input",
};
const FPLOT_ERROR_EVALUATION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.FPLOT.EVALUATION_FAILED",
    identifier: Some("RunMat:fplot:EvaluationFailed"),
    when: "The function does not return a real numeric value for a sample point.",
    message: "fplot: function evaluation failed",
};
const FPLOT_ERROR_UNSUPPORTED: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.FPLOT.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:fplot:UnsupportedOption"),
    when: "A form the approved specification does not cover (parametric curves, other interval shapes, non-scalar numeric f, char expressions without a host evaluator).",
    message: "fplot: unsupported option",
};
const FPLOT_ERROR_INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.FPLOT.INTERNAL",
    identifier: Some("RunMat:fplot:Internal"),
    when: "An underlying plotting operation fails.",
    message: "fplot: internal plotting operation failed",
};
const FPLOT_ERRORS: [BuiltinErrorDescriptor; 6] = [
    FPLOT_ERROR_RANGE,
    FPLOT_ERROR_BAD_PARAMETER,
    FPLOT_ERROR_INVALID_INPUT,
    FPLOT_ERROR_EVALUATION,
    FPLOT_ERROR_UNSUPPORTED,
    FPLOT_ERROR_INTERNAL,
];

pub const FPLOT_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &FPLOT_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &FPLOT_ERRORS,
};

/// Warning for a char/string function [fplot.warning-char-fn].
const STRING_FUNCTION_WARNING: &str = "RunMat:fplot:StringFunctionsDeprecated";

#[runtime_builtin(
    name = "fplot",
    category = "plotting",
    summary = "Plot a function of x over an interval.",
    keywords = "fplot,function plot,line,plotting",
    sink = true,
    suppress_auto_output = true,
    type_resolver(crate::builtins::plotting::type_resolvers::handle_scalar_type),
    descriptor(crate::builtins::plotting::ops::fplot::FPLOT_DESCRIPTOR),
    builtin_path = "crate::builtins::plotting::ops::fplot"
)]
pub async fn fplot_builtin(args: Vec<Value>) -> BuiltinResult<f64> {
    let mut gathered = Vec::with_capacity(args.len());
    for value in &args {
        gathered.push(gather_if_needed_async(value).await.map_err(internal)?);
    }
    // A leading axes handle, when more arguments follow [fplot.ax].
    let mut target = None;
    let mut args = gathered;
    if args.len() >= 2 {
        if let Ok(PlotHandle::Axes(figure, axes)) = resolve_plot_handle(&args[0], BUILTIN_NAME) {
            target = Some((figure, axes));
            args.remove(0);
        }
    }
    let Some(f) = args.first().cloned() else {
        return Err(fplot_error(
            &FPLOT_ERROR_INVALID_INPUT,
            "a function is required",
        ));
    };
    let function = parse_function(&f).await?;

    let mut idx = 1;
    let mut interval = DEFAULT_INTERVAL;
    if let Some(next) = args.get(1) {
        if is_function_value(next) {
            return Err(fplot_error(
                &FPLOT_ERROR_UNSUPPORTED,
                "parametric curves fplot(xt, yt) are not supported",
            ));
        }
        if is_numeric(next) {
            interval = parse_interval(next)?;
            idx = 2;
        }
    }
    let style_args = validate_style_args(&args[idx..])?;
    let parsed = parse_line_style_args(&style_args, &LineStyleParseOptions::generic(BUILTIN_NAME))
        .map_err(|err| fplot_error(&FPLOT_ERROR_INVALID_INPUT, err.message()))?;
    let width_explicit = style_args
        .iter()
        .filter_map(text_of)
        .any(|t| t.trim().eq_ignore_ascii_case("linewidth"));

    let x = sample_points(interval);
    let y = evaluate(&function, &x).await?;

    apply_axes_target(target, BUILTIN_NAME).map_err(internal)?;
    let axes = current_axes_state().active_index;
    let mut appearance = parsed.appearance.clone();
    if !width_explicit {
        appearance.line_width = DEFAULT_LINE_WIDTH;
    }
    if !parsed.line_style_explicit {
        appearance.line_style = LineStyle::Solid;
    }
    if !parsed.color_explicit {
        appearance.color = if current_hold_enabled() {
            next_line_color_for_axes(axes)
        } else {
            line_color_for_axes_series_index(axes, 0)
        };
    }
    let label = parsed.label.clone().unwrap_or_default();
    let mut line = crate::builtins::plotting::plot::build_line_plot_for_builtin(
        BUILTIN_NAME,
        x,
        y,
        &label,
        &appearance,
    )
    .map_err(|err| fplot_error(&FPLOT_ERROR_INVALID_INPUT, err.message()))?;
    if parsed.label.is_none() {
        line.label = None;
    }

    let (handle, figure, axes) = render_line(line)?;
    state::set_axis_limits_for_axes(figure, axes, Some(interval), None)
        .map_err(|err| fplot_error(&FPLOT_ERROR_INTERNAL, err.to_string()))?;
    Ok(handle)
}

/// What is being plotted.
enum PlotFunction {
    Callable(Value),
    Constant(f64),
}

async fn parse_function(f: &Value) -> BuiltinResult<PlotFunction> {
    if is_function_value(f) {
        return Ok(PlotFunction::Callable(f.clone()));
    }
    if let Some(expr) = text_of(f) {
        // [fplot.warning-char-fn]: the plot is still drawn.
        crate::warnings::emit(crate::warnings::WarningRequest {
            builtin: BUILTIN_NAME,
            identifier: STRING_FUNCTION_WARNING,
            message: "fplot: character and string functions are deprecated; use a function handle",
            show_identifier: true,
        })?;
        return expression_function(&expr).await.map(PlotFunction::Callable);
    }
    match f {
        Value::Num(v) => Ok(PlotFunction::Constant(*v)),
        Value::Int(i) => Ok(PlotFunction::Constant(i.to_f64())),
        Value::Bool(b) => Ok(PlotFunction::Constant(if *b { 1.0 } else { 0.0 })),
        Value::Tensor(t) if t.is_empty() => Err(fplot_error(
            &FPLOT_ERROR_INVALID_INPUT,
            "the function must not be empty",
        )),
        Value::Tensor(t) if t.len() == 1 => Ok(PlotFunction::Constant(t.materialize_f64()[0])),
        _ => Err(fplot_error(
            &FPLOT_ERROR_UNSUPPORTED,
            "f must be a function handle, a real numeric scalar or a char expression",
        )),
    }
}

/// Turn a char expression into a callable `@(x) <expr>` through the host
/// expression evaluator (documented choice: the variable is `x`; D-065-3).
async fn expression_function(expr: &str) -> BuiltinResult<Value> {
    let Some(hook) = crate::interaction::current_eval_hook() else {
        return Err(fplot_error(
            &FPLOT_ERROR_UNSUPPORTED,
            "char expressions need the host expression evaluator, which is not available",
        ));
    };
    let value = hook(format!("@(x) {}", expr.trim()))
        .await
        .map_err(|err| fplot_error(&FPLOT_ERROR_INVALID_INPUT, err.message()))?;
    if !is_function_value(&value) {
        return Err(fplot_error(
            &FPLOT_ERROR_INVALID_INPUT,
            "the expression did not evaluate to a function",
        ));
    }
    Ok(value)
}

fn parse_interval(value: &Value) -> BuiltinResult<(f64, f64)> {
    let d = match value {
        Value::Num(v) => vec![*v],
        Value::Int(i) => vec![i.to_f64()],
        Value::Bool(b) => vec![f64::from(u8::from(*b))],
        _ => Tensor::try_from(value)
            .map_err(|e| fplot_error(&FPLOT_ERROR_INVALID_INPUT, e))?
            .materialize_f64()
            .to_vec(),
    };
    if d.len() != 2 {
        return Err(fplot_error(
            &FPLOT_ERROR_UNSUPPORTED,
            format!(
                "only a two-element interval is supported, got {} elements",
                d.len()
            ),
        ));
    }
    let (lo, hi) = (d[0], d[1]);
    if !(lo.is_finite() && hi.is_finite() && lo < hi) {
        return Err(fplot_error(&FPLOT_ERROR_RANGE, format!("got [{lo} {hi}]")));
    }
    Ok((lo, hi))
}

/// Check names (and the optional leading LineSpec) before handing the
/// arguments to RunMat's line-style parser [fplot.error-bad-property].
fn validate_style_args(args: &[Value]) -> BuiltinResult<Vec<Value>> {
    let mut i = 0;
    while i < args.len() {
        let Some(name) = text_of(&args[i]) else {
            return Err(fplot_error(
                &FPLOT_ERROR_BAD_PARAMETER,
                "expected a property name",
            ));
        };
        let lower = name.trim().to_ascii_lowercase();
        if LINE_PROPERTIES.contains(&lower.as_str()) {
            if i + 1 >= args.len() {
                return Err(fplot_error(
                    &FPLOT_ERROR_INVALID_INPUT,
                    format!("property '{name}' has no value"),
                ));
            }
            i += 2;
            continue;
        }
        if i == 0 && is_line_spec(&name) {
            i += 1;
            continue;
        }
        return Err(fplot_error(
            &FPLOT_ERROR_BAD_PARAMETER,
            format!("'{name}' is not a line property"),
        ));
    }
    Ok(args.to_vec())
}

/// A LineSpec: at most one line style, marker and colour token.
fn is_line_spec(text: &str) -> bool {
    let t = text.trim();
    if t.is_empty() {
        return false;
    }
    let (mut style, mut marker, mut color) = (0, 0, 0);
    let mut chars = t.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '-' => {
                if matches!(chars.peek(), Some('-') | Some('.')) {
                    chars.next();
                }
                style += 1;
            }
            ':' => style += 1,
            'o' | '+' | '*' | '.' | 'x' | '^' | 'v' | '<' | '>' | 's' | 'd' | 'p' | 'h' => {
                marker += 1
            }
            'r' | 'g' | 'b' | 'c' | 'm' | 'y' | 'k' | 'w' => color += 1,
            _ => return false,
        }
    }
    style <= 1 && marker <= 1 && color <= 1
}

/// `FPLOT_POINTS` equally spaced points from `lo` to `hi`, both included.
pub(crate) fn sample_points((lo, hi): (f64, f64)) -> Vec<f64> {
    let n = FPLOT_POINTS;
    let mut x: Vec<f64> = (0..n)
        .map(|i| lo + (hi - lo) * (i as f64) / ((n - 1) as f64))
        .collect();
    x[n - 1] = hi;
    x
}

/// Evaluate on the whole row first; fall back to one call per point.
async fn evaluate(function: &PlotFunction, x: &[f64]) -> BuiltinResult<Vec<f64>> {
    let f = match function {
        PlotFunction::Constant(v) => return Ok(vec![*v; x.len()]),
        PlotFunction::Callable(f) => f,
    };
    let row = Tensor::new(x.to_vec(), vec![1, x.len()])
        .map_err(|e| fplot_error(&FPLOT_ERROR_INTERNAL, e))?;
    if let Ok(value) = call_function(f, vec![Value::Tensor(row)]).await {
        if let Some(values) = real_values(&value).await {
            if values.len() == x.len() {
                return Ok(values);
            }
        }
    }
    let mut y = Vec::with_capacity(x.len());
    for &xi in x {
        let value = call_function(f, vec![Value::Num(xi)])
            .await
            .map_err(|err| fplot_error(&FPLOT_ERROR_EVALUATION, err.message()))?;
        match real_values(&value).await {
            Some(v) if v.len() == 1 => y.push(v[0]),
            _ => {
                return Err(fplot_error(
                    &FPLOT_ERROR_EVALUATION,
                    format!("the function must return a real scalar at x = {xi}"),
                ))
            }
        }
    }
    Ok(y)
}

async fn real_values(value: &Value) -> Option<Vec<f64>> {
    let value = gather_if_needed_async(value).await.ok()?;
    match &value {
        Value::Num(v) => Some(vec![*v]),
        Value::Int(i) => Some(vec![i.to_f64()]),
        Value::Bool(b) => Some(vec![if *b { 1.0 } else { 0.0 }]),
        Value::Tensor(t) => Some(t.materialize_f64().to_vec()),
        Value::LogicalArray(l) => Some(l.data.iter().map(|&b| f64::from(b != 0)).collect()),
        _ => None,
    }
}

/// Add the line to the current axes (cleared unless `hold` is on),
/// register its handle and return `(handle, figure, axes index)`.
fn render_line(
    line: runmat_plot::plots::LinePlot,
) -> BuiltinResult<(f64, state::FigureHandle, usize)> {
    let mut line = Some(line);
    let slot = std::rc::Rc::new(std::cell::RefCell::new(None));
    let slot_in = std::rc::Rc::clone(&slot);
    let figure = crate::builtins::plotting::current_figure_handle();
    let opts = PlotRenderOptions {
        title: "",
        x_label: "",
        y_label: "",
        ..Default::default()
    };
    let apply = move |fig: &mut runmat_plot::plots::Figure, axes: usize| {
        let line = line.take().expect("line consumed once");
        let plot_index = fig.add_line_plot_on_axes(line, axes);
        *slot_in.borrow_mut() = Some((axes, plot_index));
        Ok(())
    };
    let result = state::render_active_plot(BUILTIN_NAME, opts, apply);
    let Some((axes, plot_index)) = *slot.borrow() else {
        return Err(result
            .err()
            .map(internal)
            .unwrap_or_else(|| fplot_error(&FPLOT_ERROR_INTERNAL, "the line was not added")));
    };
    let handle = state::register_line_handle(figure, axes, plot_index);
    if let Err(err) = result {
        let lower = err.to_string().to_lowercase();
        if !(lower.contains("plotting is unavailable") || lower.contains("non-main thread")) {
            return Err(internal(err));
        }
    }
    Ok((handle, figure, axes))
}

fn is_function_value(value: &Value) -> bool {
    matches!(
        value,
        Value::FunctionHandle(_)
            | Value::ExternalFunctionHandle(_)
            | Value::MethodFunctionHandle(_)
            | Value::BoundFunctionHandle { .. }
            | Value::Closure(_)
    )
}

fn is_numeric(value: &Value) -> bool {
    matches!(
        value,
        Value::Num(_) | Value::Int(_) | Value::Tensor(_) | Value::Bool(_) | Value::LogicalArray(_)
    )
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
    fplot_error(&FPLOT_ERROR_INTERNAL, err.message())
}

fn fplot_error(error: &'static BuiltinErrorDescriptor, detail: impl AsRef<str>) -> RuntimeError {
    let mut builder = build_runtime_error(format!("{}: {}", error.message, detail.as_ref()))
        .with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

/// Spec 065 (`fplot` 0.2.0). `observed_*` tests reproduce confirmed
/// observations; `unresolved_choice_*` and `deviation_*` tests pin RunMat's
/// documented choices and are not claims about MATLAB.
#[cfg(test)]
pub(super) mod fplot_tests {
    use super::{fplot_builtin, FPLOT_POINTS};
    use crate::builtins::plotting::state::{clear_figure, reset_hold_state_for_run};
    use crate::builtins::plotting::tests::{ensure_plot_test_env, lock_plot_registry};
    use crate::{BuiltinResult, RuntimeError};
    use futures::executor::block_on;
    use runmat_value::{CharArray, Tensor, Value};
    use std::sync::Arc;

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

    fn direct(args: &[Value]) -> BuiltinResult<f64> {
        block_on(fplot_builtin(args.to_vec()))
    }

    fn text(s: &str) -> Value {
        Value::CharArray(CharArray::new_row(s))
    }

    fn row(data: &[f64]) -> Value {
        Value::Tensor(Tensor::new(data.to_vec(), vec![1, data.len()]).expect("row"))
    }

    fn numbers(value: &Value) -> Vec<f64> {
        match value {
            Value::Num(v) => vec![*v],
            other => Tensor::try_from(other)
                .expect("numeric")
                .materialize_f64()
                .to_vec(),
        }
    }

    fn prop(handle: f64, name: &str) -> Value {
        call("get", &[Value::Num(handle), text(name)]).expect("get")
    }

    fn string(value: Value) -> String {
        match value {
            Value::String(s) => s,
            Value::CharArray(ca) => ca.data.iter().collect(),
            other => panic!("expected text, got {other:?}"),
        }
    }

    fn identifier(err: &RuntimeError) -> String {
        err.identifier().unwrap_or_default().to_string()
    }

    /// A test function `@(x) g(x)`, applied elementwise to numeric input.
    fn with_function(
        g: impl Fn(f64) -> f64 + Send + Sync + 'static,
    ) -> crate::user_functions::FunctionInvokerGuard {
        let g = Arc::new(g);
        crate::user_functions::install_semantic_function_invoker(Some(Arc::new(
            move |_function, args, _requested_outputs| {
                let g = Arc::clone(&g);
                let out = match &args[0] {
                    Value::Num(v) => Value::Num(g(*v)),
                    Value::Tensor(t) => Value::Tensor(
                        Tensor::new(
                            t.materialize_f64().iter().map(|v| g(*v)).collect(),
                            t.shape.clone(),
                        )
                        .expect("tensor"),
                    ),
                    other => panic!("unexpected argument {other:?}"),
                };
                Box::pin(async move { crate::sequence::single_value_sequence(out) })
            },
        )))
    }

    fn handle_value() -> Value {
        Value::BoundFunctionHandle {
            name: "fplot_test_fn".into(),
            function: 1,
        }
    }

    fn children_of_gca() -> Vec<f64> {
        let gca = call("gca", &[]).expect("gca");
        // RunMat's axes `Children` lists title/label objects; plot objects
        // are listed by `allchild` (D-065-8).
        numbers(&call("allchild", &[gca]).expect("allchild"))
    }

    /// The line colour as RGB. RunMat's line `Color` property reports a
    /// colour name for the eight named colours (D-065-7), so it is
    /// normalised through `validatecolor`.
    fn rgb_of(h: f64) -> Vec<f64> {
        numbers(&call("validatecolor", &[prop(h, "Color")]).expect("validatecolor"))
    }

    fn assert_line_basics(h: f64, lo: f64, hi: f64) {
        assert_eq!(string(prop(h, "Type")), "line");
        let x = numbers(&prop(h, "XData"));
        assert!(x.len() > 2, "more than two points");
        assert_eq!(x.first().copied(), Some(lo));
        assert_eq!(x.last().copied(), Some(hi));
        let gca = call("gca", &[]).expect("gca");
        assert_eq!(prop(h, "Parent"), gca);
    }

    // ---- observed (normative) ----

    /// [fplot.signature-primary, fplot.object, fplot.default-interval,
    /// fplot.linespec defaults] case `default`.
    #[test]
    fn observed_default_line() {
        let _guard = setup();
        let _f = with_function(|x| x);
        let h = direct(&[handle_value()]).unwrap();
        assert_line_basics(h, -5.0, 5.0);
        assert_eq!(numbers(&prop(h, "LineWidth")), vec![0.5]);
        assert_eq!(string(prop(h, "LineStyle")), "-");
        // Default Color is a documented choice (fplot.q-theme): not asserted.
    }

    /// [fplot.default-interval] case `xlim-default`.
    #[test]
    fn observed_default_xlim() {
        let _guard = setup();
        let _f = with_function(|x| x);
        direct(&[handle_value()]).unwrap();
        let xlim = call("xlim", &[]).expect("xlim");
        assert_eq!(numbers(&xlim), vec![-5.0, 5.0]);
    }

    /// [fplot.interval] cases `interval`, `values`.
    #[test]
    fn observed_interval_and_values() {
        let _guard = setup();
        let _f = with_function(|x| x * x);
        let h = direct(&[handle_value(), row(&[0.0, 2.0])]).unwrap();
        assert_line_basics(h, 0.0, 2.0);
        let x = numbers(&prop(h, "XData"));
        let y = numbers(&prop(h, "YData"));
        let max_err = x
            .iter()
            .zip(&y)
            .map(|(a, b)| (b - a.powi(2)).abs())
            .fold(0.0, f64::max);
        assert_eq!(max_err, 0.0);
    }

    /// [fplot.cobra-form] case `cobra-nv`.
    #[test]
    fn observed_cobra_name_value_form() {
        let _guard = setup();
        let _f = with_function(|x| x);
        let h = direct(&[
            handle_value(),
            text("LineWidth"),
            Value::Num(1.5),
            text("Color"),
            text("r"),
        ])
        .unwrap();
        assert_line_basics(h, -5.0, 5.0);
        assert_eq!(numbers(&prop(h, "LineWidth")), vec![1.5]);
        assert_eq!(rgb_of(h), vec![1.0, 0.0, 0.0]);
        assert_eq!(string(prop(h, "LineStyle")), "-");
    }

    /// [fplot.cobra-form] case `cobra-no-output`: through the dispatcher,
    /// the axes then hold exactly one such line.
    #[test]
    fn observed_cobra_form_without_output() {
        let _guard = setup();
        let _f = with_function(|x| x);
        call(
            "fplot",
            &[
                handle_value(),
                text("LineWidth"),
                Value::Num(1.5),
                text("Color"),
                text("r"),
            ],
        )
        .expect("fplot");
        let children = children_of_gca();
        assert_eq!(children.len(), 1);
        let c = children[0];
        assert_eq!(string(prop(c, "Type")), "line");
        assert_eq!(numbers(&prop(c, "LineWidth")), vec![1.5]);
        assert_eq!(rgb_of(c), vec![1.0, 0.0, 0.0]);
    }

    /// [fplot.linespec] case `linespec`.
    #[test]
    fn observed_linespec() {
        let _guard = setup();
        let _f = with_function(|x| x);
        let h = direct(&[handle_value(), text("r--")]).unwrap();
        assert_line_basics(h, -5.0, 5.0);
        assert_eq!(rgb_of(h), vec![1.0, 0.0, 0.0]);
        assert_eq!(string(prop(h, "LineStyle")), "--");
        assert_eq!(numbers(&prop(h, "LineWidth")), vec![0.5]);
    }

    /// [fplot.ax] case `ax-arg`.
    #[test]
    fn observed_axes_argument() {
        let _guard = setup();
        let _f = with_function(|x| x);
        let ax = call("axes", &[]).expect("axes");
        let h = direct(&[ax.clone(), handle_value()]).unwrap();
        assert_eq!(prop(h, "Parent"), ax);
    }

    /// [fplot.warning-char-fn] case `char-fn`: drawn, and `lastwarn`
    /// reports the warning. The expression goes through the host evaluator
    /// (here a test hook returning a test function; D-065-3).
    #[test]
    fn observed_char_function_warns_and_draws() {
        let _guard = setup();
        let _f = with_function(f64::sin);
        let seen = Arc::new(std::sync::Mutex::new(String::new()));
        let seen_in = Arc::clone(&seen);
        let _hook = crate::interaction::replace_eval_hook(Some(Arc::new(move |text: String| {
            *seen_in.lock().unwrap() = text;
            Box::pin(async move {
                Ok(Value::BoundFunctionHandle {
                    name: "fplot_test_fn".into(),
                    function: 1,
                })
            })
        })));
        crate::warnings::with_policy(|p| p.set_lastwarn(None));
        let h = direct(&[text("sin(x)")]).unwrap();
        assert_eq!(seen.lock().unwrap().as_str(), "@(x) sin(x)");
        assert_line_basics(h, -5.0, 5.0);
        let y = numbers(&prop(h, "YData"));
        assert_eq!(y[0], (-5.0f64).sin());
        let last = crate::warnings::with_policy(|p| p.lastwarn()).expect("a warning");
        assert_eq!(last.identifier, "RunMat:fplot:StringFunctionsDeprecated");
        // lastwarn's two-output form reports the identifier.
        let _out = crate::output_count::push_output_count(Some(2));
        let pair = block_on(
            crate::builtins::diagnostics::exceptions_warnings::lastwarn_builtin(Vec::new()),
        )
        .expect("lastwarn");
        let Value::OutputList(items) = pair else {
            panic!("expected two outputs");
        };
        assert_eq!(
            string(items[1].clone()),
            "RunMat:fplot:StringFunctionsDeprecated"
        );
    }

    /// [fplot.numeric] (non-normative) case `numeric-constant`.
    #[test]
    fn observed_numeric_constant() {
        let _guard = setup();
        let h = direct(&[Value::Num(5.0)]).unwrap();
        assert_line_basics(h, -5.0, 5.0);
        assert!(numbers(&prop(h, "YData")).iter().all(|&v| v == 5.0));
    }

    /// [fplot.error-decreasing] case `err-decreasing`.
    #[test]
    fn observed_decreasing_interval_errors() {
        let _guard = setup();
        let _f = with_function(|x| x);
        let err = direct(&[handle_value(), row(&[2.0, 1.0])]).unwrap_err();
        assert_eq!(identifier(&err), "RunMat:fplot:RangeVector");
    }

    /// [fplot.error-bad-property] case `err-bad-property`.
    #[test]
    fn observed_bad_property_errors() {
        let _guard = setup();
        let _f = with_function(|x| x);
        let err = direct(&[handle_value(), text("NotAProperty"), Value::Num(1.0)]).unwrap_err();
        assert_eq!(identifier(&err), "RunMat:fplot:InvalidParameter");
    }

    // ---- documented choices for unobserved forms ----

    /// (choice) Sampling grid, evaluation fallback, interval and argument
    /// errors (fplot.q-points; FR-065-10).
    #[test]
    fn unresolved_choice_sampling_and_arguments() {
        let _guard = setup();
        // A function that is not vectorised: the row call returns a
        // scalar, so each point is evaluated separately.
        let _f = crate::user_functions::install_semantic_function_invoker(Some(Arc::new(
            move |_function, args, _requested_outputs| {
                let out = match &args[0] {
                    Value::Num(v) => Value::Num(v * 3.0),
                    _ => Value::Num(0.0),
                };
                Box::pin(async move { crate::sequence::single_value_sequence(out) })
            },
        )));
        let h = direct(&[handle_value(), row(&[0.0, 1.0])]).unwrap();
        let x = numbers(&prop(h, "XData"));
        let y = numbers(&prop(h, "YData"));
        assert_eq!(x.len(), FPLOT_POINTS);
        assert_eq!(x[100], 0.5);
        assert_eq!(y[FPLOT_POINTS - 1], 3.0);
        let id = |args: &[Value]| identifier(&direct(args).unwrap_err());
        assert_eq!(
            id(&[handle_value(), row(&[1.0, 1.0])]),
            "RunMat:fplot:RangeVector"
        );
        assert_eq!(
            id(&[handle_value(), row(&[0.0, f64::INFINITY])]),
            "RunMat:fplot:RangeVector"
        );
        assert_eq!(
            id(&[handle_value(), row(&[0.0, 1.0, 2.0, 3.0])]),
            "RunMat:fplot:UnsupportedOption"
        );
        assert_eq!(
            id(&[handle_value(), handle_value()]),
            "RunMat:fplot:UnsupportedOption"
        );
        assert_eq!(id(&[]), "RunMat:fplot:InvalidInput");
        assert_eq!(id(&[row(&[1.0, 2.0])]), "RunMat:fplot:UnsupportedOption");
        assert_eq!(
            id(&[handle_value(), text("LineWidth")]),
            "RunMat:fplot:InvalidInput"
        );
        assert_eq!(
            id(&[handle_value(), text("LineWidth"), Value::Num(-1.0)]),
            "RunMat:fplot:InvalidInput"
        );
        assert_eq!(
            id(&[handle_value(), text("zz")]),
            "RunMat:fplot:InvalidParameter"
        );
        assert_eq!(
            id(&[
                handle_value(),
                Value::Num(3.0),
                Value::Num(1.0),
                Value::Num(2.0)
            ]),
            "RunMat:fplot:UnsupportedOption"
        );
    }

    /// (choice) A non-real function result raises EvaluationFailed.
    #[test]
    fn unresolved_choice_evaluation_failure() {
        let _guard = setup();
        let _f = crate::user_functions::install_semantic_function_invoker(Some(Arc::new(
            move |_function, _args, _requested_outputs| {
                Box::pin(async move {
                    crate::sequence::single_value_sequence(Value::String("no".into()))
                })
            },
        )));
        let err = direct(&[handle_value()]).unwrap_err();
        assert_eq!(identifier(&err), "RunMat:fplot:EvaluationFailed");
    }

    /// D-065-7: `get(h, 'Color')` should be the RGB row [1 0 0].
    #[test]
    #[ignore = "D-065-7: RunMat's line Color property reports 'r' for pure red (upstream properties.rs)"]
    fn deviation_color_reported_as_rgb_row() {
        let _guard = setup();
        let _f = with_function(|x| x);
        let h = direct(&[handle_value(), text("Color"), text("r")]).unwrap();
        assert_eq!(numbers(&prop(h, "Color")), vec![1.0, 0.0, 0.0]);
    }

    /// D-065-3: without a host evaluator a char expression warns and then
    /// raises UnsupportedOption.
    #[test]
    fn deviation_char_function_without_evaluator() {
        let _guard = setup();
        let _hook = crate::interaction::replace_eval_hook(None);
        crate::warnings::with_policy(|p| p.set_lastwarn(None));
        let err = direct(&[text("sin(x)")]).unwrap_err();
        assert_eq!(identifier(&err), "RunMat:fplot:UnsupportedOption");
        let last = crate::warnings::with_policy(|p| p.lastwarn()).expect("a warning");
        assert_eq!(last.identifier, "RunMat:fplot:StringFunctionsDeprecated");
    }

    /// (choice) `hold on` keeps earlier objects and advances the colour order.
    #[test]
    fn unresolved_choice_hold_keeps_lines() {
        let _guard = setup();
        let _f = with_function(|x| x);
        direct(&[handle_value()]).unwrap();
        call("hold", &[text("on")]).expect("hold");
        direct(&[handle_value(), row(&[0.0, 1.0])]).unwrap();
        assert_eq!(children_of_gca().len(), 2);
        call("hold", &[text("off")]).expect("hold");
        direct(&[handle_value()]).unwrap();
        assert_eq!(children_of_gca().len(), 1);
    }
}
