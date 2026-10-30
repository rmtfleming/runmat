//! `spy` builtin: plot the nonzero pattern of a matrix (spec 040).
//!
//! Normative behaviour comes from the approved `spy` 0.3.0 export (claims
//! `spy.void-plot`, `spy.one-line`, `spy.positions`, `spy.axes`,
//! `spy.nz-label`, `spy.default-marker`, `spy.linespec`, `spy.markersize`,
//! `spy.all-zero`, `spy.input-types`, `spy.error-output-arg`). Default
//! marker size and colour are RunMat's own plot defaults (documented
//! choice); see `specs/040-spy/spec.md`.

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{CharArray, Tensor, Value};

use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const BUILTIN_NAME: &str = "spy";
const MARKER_TOKENS: &str = "o+*.x^v<>sdph";
const COLOR_TOKENS: &str = "rgbcmykw";
const LINE_STYLE_TOKENS: &str = "-:";

const SPY_INPUT_A: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "A",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Matrix whose nonzero pattern is plotted (numeric, logical, char or sparse).",
};
const SPY_INPUT_LINESPEC: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "LineSpec",
    ty: BuiltinParamType::StringScalar,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Marker and colour characters, e.g. 'r*'.",
};
const SPY_INPUT_MARKERSIZE: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "markersize",
    ty: BuiltinParamType::NumericScalar,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Positive marker size.",
};
const SPY_INPUTS_A: [BuiltinParamDescriptor; 1] = [SPY_INPUT_A];
const SPY_INPUTS_A_SPEC: [BuiltinParamDescriptor; 2] = [SPY_INPUT_A, SPY_INPUT_LINESPEC];
const SPY_INPUTS_A_SIZE: [BuiltinParamDescriptor; 2] = [SPY_INPUT_A, SPY_INPUT_MARKERSIZE];
const SPY_INPUTS_A_SPEC_SIZE: [BuiltinParamDescriptor; 3] =
    [SPY_INPUT_A, SPY_INPUT_LINESPEC, SPY_INPUT_MARKERSIZE];
const SPY_SIGNATURES: [BuiltinSignatureDescriptor; 4] = [
    BuiltinSignatureDescriptor {
        label: "spy(A)",
        inputs: &SPY_INPUTS_A,
        outputs: &[],
    },
    BuiltinSignatureDescriptor {
        label: "spy(A, LineSpec)",
        inputs: &SPY_INPUTS_A_SPEC,
        outputs: &[],
    },
    BuiltinSignatureDescriptor {
        label: "spy(A, markersize)",
        inputs: &SPY_INPUTS_A_SIZE,
        outputs: &[],
    },
    BuiltinSignatureDescriptor {
        label: "spy(A, LineSpec, markersize)",
        inputs: &SPY_INPUTS_A_SPEC_SIZE,
        outputs: &[],
    },
];

const SPY_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SPY.INVALID_INPUT",
    identifier: Some("RunMat:spy:InvalidInput"),
    when: "A is missing, non-numeric or N-D, or a LineSpec/marker size is malformed.",
    message: "spy: invalid input",
};
const SPY_ERROR_UNSUPPORTED_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SPY.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:spy:UnsupportedOption"),
    when: "The call uses a form or input the approved specification leaves unresolved (complex input, line styles, axes targets, extra arguments).",
    message: "spy: unsupported option",
};
const SPY_ERROR_TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SPY.TOO_MANY_OUTPUTS",
    identifier: Some("RunMat:spy:TooManyOutputs"),
    when: "An output argument is requested from spy.",
    message: "spy does not return output arguments",
};
const SPY_ERROR_INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SPY.INTERNAL",
    identifier: Some("RunMat:spy:Internal"),
    when: "An underlying plotting operation fails.",
    message: "spy: internal plotting operation failed",
};
const SPY_ERRORS: [BuiltinErrorDescriptor; 4] = [
    SPY_ERROR_INVALID_INPUT,
    SPY_ERROR_UNSUPPORTED_OPTION,
    SPY_ERROR_TOO_MANY_OUTPUTS,
    SPY_ERROR_INTERNAL,
];

pub const SPY_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &SPY_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &SPY_ERRORS,
};

#[derive(Debug, Default)]
struct SpyStyle {
    color: Option<char>,
    marker: Option<char>,
    marker_size: Option<f64>,
}

impl SpyStyle {
    fn plot_spec(&self) -> String {
        let mut spec = String::new();
        if let Some(color) = self.color {
            spec.push(color);
        }
        spec.push(self.marker.unwrap_or('.'));
        spec
    }
}

#[runtime_builtin(
    name = "spy",
    category = "plotting",
    summary = "Plot the sparsity pattern of a matrix.",
    keywords = "spy,sparsity,pattern,nonzero,plotting",
    sink = true,
    suppress_auto_output = true,
    descriptor(crate::builtins::plotting::ops::spy::SPY_DESCRIPTOR),
    builtin_path = "crate::builtins::plotting::ops::spy"
)]
pub async fn spy_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    if matches!(crate::output_count::current_output_count(), Some(n) if n > 0) {
        return Err(spy_error(
            &SPY_ERROR_TOO_MANY_OUTPUTS,
            "no output value is returned",
        ));
    }
    let mut args = args.into_iter();
    let Some(a) = args.next() else {
        return Err(spy_error(
            &SPY_ERROR_INVALID_INPUT,
            "spy requires a matrix argument A",
        ));
    };
    let mut rest = Vec::new();
    for value in args {
        rest.push(gather_if_needed_async(&value).await.map_err(internal)?);
    }
    let style = parse_style(&rest)?;
    let a = gather_if_needed_async(&a).await.map_err(internal)?;
    let a = normalize_input(a)?;

    let (rows, cols) = matrix_dims(&a)?;

    // `find` is evaluated directly (not through the dispatcher) so the row
    // and column outputs do not depend on the caller's requested output count.
    let found = crate::builtins::array::indexing::find::evaluate(a, &[]).await?;
    let row_idx = numbers(&found.row_value()?)?;
    let col_idx = numbers(&found.column_value()?)?;
    let nnz = row_idx.len();
    let (x, y) = if nnz == 0 {
        (vec![f64::NAN], vec![f64::NAN])
    } else {
        (col_idx, row_idx)
    };

    let mut plot_args = vec![
        row_vector(x)?,
        row_vector(y)?,
        Value::CharArray(CharArray::new_row(&style.plot_spec())),
    ];
    if let Some(size) = style.marker_size {
        plot_args.push(Value::CharArray(CharArray::new_row("MarkerSize")));
        plot_args.push(Value::Num(size));
    }
    call("plot", &plot_args).await?;
    call("xlim", &[row_vector(vec![0.0, cols + 1.0])?]).await?;
    call("ylim", &[row_vector(vec![0.0, rows + 1.0])?]).await?;
    // daspect's setter form must be called with zero requested outputs.
    crate::call_builtin_async_with_outputs("daspect", &[row_vector(vec![1.0, 1.0, 1.0])?], 0)
        .await
        .map_err(internal)?;
    call(
        "xlabel",
        &[Value::CharArray(CharArray::new_row(&format!("nz = {nnz}")))],
    )
    .await?;
    Ok(Value::OutputList(Vec::new()))
}

/// Rows and columns of a 2-D input; trailing singleton dimensions are ignored.
fn matrix_dims(a: &Value) -> BuiltinResult<(f64, f64)> {
    let shape: Vec<usize> = match a {
        Value::Tensor(t) => t.shape.clone(),
        Value::LogicalArray(l) => l.shape.clone(),
        Value::SparseTensor(s) => vec![s.rows, s.cols],
        _ => vec![1, 1],
    };
    let mut shape = match shape.len() {
        0 => vec![1, 1],
        1 => vec![shape[0], 1],
        _ => shape,
    };
    while shape.len() > 2 && shape.last() == Some(&1) {
        shape.pop();
    }
    if shape.len() > 2 {
        return Err(spy_error(
            &SPY_ERROR_INVALID_INPUT,
            "A must be a 2-D matrix",
        ));
    }
    Ok((shape[0] as f64, shape[1] as f64))
}

fn parse_style(rest: &[Value]) -> BuiltinResult<SpyStyle> {
    let mut style = SpyStyle::default();
    match rest {
        [] => {}
        [first] => {
            if let Some(spec) = text(first) {
                parse_linespec(&spec, &mut style)?;
            } else if is_numeric(first) {
                style.marker_size = Some(marker_size(first)?);
            } else {
                return Err(spy_error(
                    &SPY_ERROR_INVALID_INPUT,
                    "the second argument must be a LineSpec or a marker size",
                ));
            }
        }
        [first, second] => {
            let Some(spec) = text(first) else {
                return Err(spy_error(
                    &SPY_ERROR_UNSUPPORTED_OPTION,
                    "only spy(A, LineSpec, markersize) is supported with three arguments",
                ));
            };
            if !is_numeric(second) {
                return Err(spy_error(
                    &SPY_ERROR_UNSUPPORTED_OPTION,
                    "the third argument must be a numeric marker size; name-value options are not supported",
                ));
            }
            parse_linespec(&spec, &mut style)?;
            style.marker_size = Some(marker_size(second)?);
        }
        _ => {
            return Err(spy_error(
                &SPY_ERROR_UNSUPPORTED_OPTION,
                "spy accepts at most A, a LineSpec and a marker size",
            ))
        }
    }
    Ok(style)
}

fn parse_linespec(spec: &str, style: &mut SpyStyle) -> BuiltinResult<()> {
    if spec.is_empty() {
        return Err(spy_error(
            &SPY_ERROR_INVALID_INPUT,
            "LineSpec must not be empty",
        ));
    }
    // Line-style tokens are unobserved for spy: reject before anything else.
    if spec.chars().any(|ch| LINE_STYLE_TOKENS.contains(ch)) {
        return Err(spy_error(
            &SPY_ERROR_UNSUPPORTED_OPTION,
            format!("line styles in LineSpec '{spec}' are not supported"),
        ));
    }
    for ch in spec.chars() {
        if MARKER_TOKENS.contains(ch) {
            if style.marker.replace(ch).is_some() {
                return Err(spy_error(
                    &SPY_ERROR_INVALID_INPUT,
                    format!("LineSpec '{spec}' names more than one marker"),
                ));
            }
        } else if COLOR_TOKENS.contains(ch) {
            if style.color.replace(ch).is_some() {
                return Err(spy_error(
                    &SPY_ERROR_INVALID_INPUT,
                    format!("LineSpec '{spec}' names more than one colour"),
                ));
            }
        } else {
            return Err(spy_error(
                &SPY_ERROR_INVALID_INPUT,
                format!("unrecognised LineSpec character '{ch}' in '{spec}'"),
            ));
        }
    }
    Ok(())
}

fn marker_size(value: &Value) -> BuiltinResult<f64> {
    let size = match value {
        Value::Num(v) => Some(*v),
        Value::Int(i) => Some(i.to_f64()),
        Value::Tensor(t) if t.len() == 1 => {
            Some(crate::builtins::common::tensor::tensor_value_f64(t, 0))
        }
        Value::Tensor(_) => {
            return Err(spy_error(
                &SPY_ERROR_UNSUPPORTED_OPTION,
                "a non-scalar numeric second argument (such as spy(ax, A)) is not supported",
            ))
        }
        _ => None,
    };
    match size {
        Some(v) if v.is_finite() && v > 0.0 => Ok(v),
        _ => Err(spy_error(
            &SPY_ERROR_INVALID_INPUT,
            "marker size must be a real, finite, positive scalar",
        )),
    }
}

fn is_numeric(value: &Value) -> bool {
    matches!(
        value,
        Value::Num(_)
            | Value::Int(_)
            | Value::Tensor(_)
            | Value::Complex(_, _)
            | Value::ComplexTensor(_)
    )
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

/// Accept real numeric, logical, char and sparse real/logical input; char
/// becomes its code points. Complex input is unresolved (spy.q-large).
fn normalize_input(a: Value) -> BuiltinResult<Value> {
    match a {
        Value::Num(_) | Value::Int(_) | Value::Bool(_) | Value::LogicalArray(_) => Ok(a),
        Value::Tensor(_) => Ok(a),
        Value::SparseTensor(ref s) if s.is_complex() => Err(complex_unsupported()),
        Value::SparseTensor(_) => Ok(a),
        Value::Complex(_, _) | Value::ComplexTensor(_) => Err(complex_unsupported()),
        Value::CharArray(ca) => {
            if ca.shape.len() > 2 {
                return Err(spy_error(
                    &SPY_ERROR_INVALID_INPUT,
                    "A must be a 2-D matrix",
                ));
            }
            let (rows, cols) = (ca.rows, ca.cols);
            let mut data = vec![0.0; rows * cols];
            for r in 0..rows {
                for c in 0..cols {
                    data[r + c * rows] = ca.data[r * cols + c] as u32 as f64;
                }
            }
            Tensor::new(data, vec![rows, cols])
                .map(Value::Tensor)
                .map_err(|err| spy_error(&SPY_ERROR_INTERNAL, err))
        }
        other => Err(spy_error(
            &SPY_ERROR_INVALID_INPUT,
            format!(
                "A must be numeric, logical or char, got {}",
                value_kind(&other)
            ),
        )),
    }
}

fn complex_unsupported() -> RuntimeError {
    spy_error(
        &SPY_ERROR_UNSUPPORTED_OPTION,
        "complex input is not supported (unresolved in the approved specification)",
    )
}

fn value_kind(value: &Value) -> &'static str {
    match value {
        Value::String(_) | Value::StringArray(_) => "string",
        Value::Cell(_) => "cell",
        Value::Struct(_) | Value::StructArray(_) => "struct",
        Value::Object(_) | Value::ObjectArray(_) | Value::HandleObject(_) => "object",
        _ => "an unsupported value",
    }
}

fn row_vector(data: Vec<f64>) -> BuiltinResult<Value> {
    let n = data.len();
    Tensor::new(data, vec![1, n])
        .map(Value::Tensor)
        .map_err(|err| spy_error(&SPY_ERROR_INTERNAL, err))
}

fn numbers(value: &Value) -> BuiltinResult<Vec<f64>> {
    match value {
        Value::Num(v) => Ok(vec![*v]),
        Value::Tensor(t) => Ok(t.materialize_f64().to_vec()),
        other => Tensor::try_from(other)
            .map(|t| t.materialize_f64().to_vec())
            .map_err(|err| spy_error(&SPY_ERROR_INTERNAL, format!("{err:?}"))),
    }
}

async fn call(name: &str, args: &[Value]) -> BuiltinResult<Value> {
    crate::call_builtin_async(name, args)
        .await
        .map_err(internal)
}

fn internal(err: RuntimeError) -> RuntimeError {
    let message = format!("{}: {}", SPY_ERROR_INTERNAL.message, err.message());
    build_runtime_error(message)
        .with_builtin(BUILTIN_NAME)
        .with_identifier(SPY_ERROR_INTERNAL.identifier.unwrap_or_default())
        .with_source(err)
        .build()
}

fn spy_error(error: &'static BuiltinErrorDescriptor, detail: impl AsRef<str>) -> RuntimeError {
    let mut builder = build_runtime_error(format!("{}: {}", error.message, detail.as_ref()))
        .with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

#[cfg(test)]
mod spy_tests {
    use crate::builtins::plotting::state::{clear_figure, reset_hold_state_for_run};
    use crate::builtins::plotting::tests::{ensure_plot_test_env, lock_plot_registry};
    use crate::{BuiltinResult, RuntimeError};
    use futures::executor::block_on;
    use runmat_value::{CharArray, ComplexTensor, LogicalArray, Tensor, Value};

    fn setup() -> crate::builtins::plotting::state::PlotTestLockGuard {
        let guard = lock_plot_registry();
        ensure_plot_test_env();
        reset_hold_state_for_run();
        // A new figure makes the current axes exist and start empty.
        call("figure", &[]).expect("figure");
        let _ = clear_figure(None);
        guard
    }

    /// Error paths call the builtin function directly. Through the
    /// dispatcher, a failing builtin is re-run because every builtin is
    /// registered twice in this build, and the re-run does not see the
    /// requested output count (see validation.md, finding F-1).
    fn direct(args: &[Value]) -> BuiltinResult<Value> {
        block_on(super::spy_builtin(args.to_vec()))
    }

    fn fresh() {
        let _ = clear_figure(None);
    }

    fn call(name: &str, args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async(name, args))
    }

    fn matrix(rows: usize, cols: usize, column_major: &[f64]) -> Value {
        Value::Tensor(Tensor::new(column_major.to_vec(), vec![rows, cols]).expect("matrix"))
    }

    fn sparse(rows: usize, cols: usize, column_major: &[f64]) -> Value {
        let a = call("sparse", &[matrix(rows, cols, column_major)]).expect("sparse");
        assert!(matches!(a, Value::SparseTensor(_)));
        a
    }

    fn text(s: &str) -> Value {
        Value::CharArray(CharArray::new_row(s))
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

    fn gca() -> Value {
        call("gca", &[]).expect("gca")
    }

    /// Line objects in the current axes. RunMat's axes `Children` property
    /// lists only the axes' text/legend objects, not plot lines, so the
    /// "axes has content" proxy is observed with `findobj` (validation.md,
    /// deviation D-040-4).
    fn axes_children() -> Vec<f64> {
        let found = call("findobj", &[gca(), text("Type"), text("line")]).expect("findobj");
        match found {
            Value::Tensor(t) if t.materialize_f64().is_empty() => Vec::new(),
            other => numbers(&other),
        }
    }

    fn only_child() -> Value {
        let children = axes_children();
        assert_eq!(children.len(), 1, "expected one axes child: {children:?}");
        Value::Num(children[0])
    }

    fn prop(handle: &Value, name: &str) -> Value {
        call("get", &[handle.clone(), text(name)]).expect("get property")
    }

    fn string(value: Value) -> String {
        match value {
            Value::String(s) => s,
            Value::CharArray(ca) => ca.data.iter().collect(),
            other => panic!("expected text, got {other:?}"),
        }
    }

    fn xlabel_text() -> String {
        let label = prop(&gca(), "XLabel");
        string(prop(&label, "String"))
    }

    fn identifier(err: &RuntimeError) -> String {
        err.identifier().unwrap_or_default().to_string()
    }

    fn spy(args: &[Value]) {
        let out = call("spy", args).expect("spy");
        assert_eq!(out, Value::OutputList(Vec::new()), "spy returns no output");
    }

    fn xy(child: &Value) -> (Vec<f64>, Vec<f64>) {
        (
            numbers(&prop(child, "XData")),
            numbers(&prop(child, "YData")),
        )
    }

    // ---- observed (normative) ----

    /// [spy.void-plot] case `plots`.
    #[test]
    fn observed_plots_adds_axes_children() {
        let _guard = setup();
        let before = axes_children().len();
        spy(&[sparse(2, 2, &[1.0, 0.0, 0.0, 1.0])]);
        assert!(axes_children().len() > before, "axes must gain content");
    }

    /// [spy.void-plot, spy.error-output-arg] case `output-arg`.
    #[test]
    fn observed_output_request_errors() {
        let _guard = setup();
        let a = sparse(2, 2, &[1.0, 0.0, 0.0, 1.0]);
        let _outputs = crate::output_count::push_output_count(Some(1));
        let err = direct(&[a]).expect_err("requesting an output must fail");
        assert_eq!(identifier(&err), "RunMat:spy:TooManyOutputs");
    }

    /// [spy.one-line, spy.positions, spy.default-marker] case `points`.
    #[test]
    fn observed_points_one_marker_only_line() {
        let _guard = setup();
        // [1 0 1; 0 1 0]
        spy(&[sparse(2, 3, &[1.0, 0.0, 0.0, 1.0, 1.0, 0.0])]);
        let child = only_child();
        assert_eq!(prop(&child, "Type"), Value::String("line".into()));
        assert_eq!(prop(&child, "LineStyle"), Value::String("none".into()));
        // RunMat's line model stores the '.' marker as a point-sized circle
        // and reports it as 'o' (see validation.md, deviation D-040-2).
        assert_ne!(prop(&child, "Marker"), Value::String("none".into()));
        assert_eq!(xy(&child), (vec![1.0, 2.0, 3.0], vec![1.0, 2.0, 1.0]));
    }

    /// [spy.axes] cases `points`, `tall`, `all-zero`: limits and equal
    /// data units. (YDir 'reverse' is deviation D-040-1.)
    #[test]
    fn observed_axes_limits_and_equal_units() {
        let _guard = setup();
        for (a, xlim, ylim) in [
            (
                sparse(2, 3, &[1.0, 0.0, 0.0, 1.0, 1.0, 0.0]),
                [0.0, 4.0],
                [0.0, 3.0],
            ),
            (sparse(4, 1, &[1.0, 0.0, 0.0, 1.0]), [0.0, 2.0], [0.0, 5.0]),
            (sparse(3, 3, &[0.0; 9]), [0.0, 4.0], [0.0, 4.0]),
        ] {
            fresh();
            spy(&[a]);
            let ax = gca();
            assert_eq!(numbers(&prop(&ax, "XLim")), xlim.to_vec());
            assert_eq!(numbers(&prop(&ax, "YLim")), ylim.to_vec());
            let dar = numbers(&prop(&ax, "DataAspectRatio"));
            assert_eq!(dar[0], dar[1], "x and y data units must be equal");
        }
    }

    /// [spy.nz-label] cases `points`, `dense`, `all-zero`.
    #[test]
    fn observed_nz_label() {
        let _guard = setup();
        for (a, label) in [
            (sparse(2, 3, &[1.0, 0.0, 0.0, 1.0, 1.0, 0.0]), "nz = 3"),
            (matrix(2, 2, &[0.0, 3.0, 7.0, 0.0]), "nz = 2"),
            (sparse(3, 3, &[0.0; 9]), "nz = 0"),
        ] {
            fresh();
            spy(&[a]);
            assert_eq!(xlabel_text(), label);
        }
    }

    /// [spy.all-zero, spy.one-line] case `all-zero`.
    #[test]
    fn observed_all_zero_gives_one_nan_line() {
        let _guard = setup();
        spy(&[sparse(3, 3, &[0.0; 9])]);
        let child = only_child();
        assert_eq!(prop(&child, "LineStyle"), Value::String("none".into()));
        let (x, y) = xy(&child);
        assert!(x.len() == 1 && x[0].is_nan(), "XData {x:?}");
        assert!(y.len() == 1 && y[0].is_nan(), "YData {y:?}");
    }

    /// [spy.linespec, spy.markersize] cases `linespec`,
    /// `linespec-markersize`.
    #[test]
    fn observed_linespec_sets_marker_and_colour() {
        let _guard = setup();
        let a = || sparse(2, 2, &[1.0, 0.0, 0.0, 1.0]);
        spy(&[a(), text("r*")]);
        let child = only_child();
        assert_eq!(prop(&child, "Marker"), Value::String("*".into()));
        assert_eq!(prop(&child, "Color"), Value::String("r".into()));
        assert_eq!(prop(&child, "LineStyle"), Value::String("none".into()));
        assert_eq!(xy(&child), (vec![1.0, 2.0], vec![1.0, 2.0]));

        fresh();
        spy(&[a(), text("g+"), Value::Num(12.0)]);
        let child = only_child();
        assert_eq!(prop(&child, "Marker"), Value::String("+".into()));
        assert_eq!(prop(&child, "Color"), Value::String("g".into()));
        assert_eq!(prop(&child, "MarkerSize"), Value::Num(12.0));
        assert_eq!(prop(&child, "LineStyle"), Value::String("none".into()));
    }

    /// [spy.markersize, spy.default-marker] case `markersize`.
    #[test]
    fn observed_markersize_sets_marker_size() {
        let _guard = setup();
        spy(&[sparse(2, 2, &[1.0, 0.0, 0.0, 1.0]), Value::Num(20.0)]);
        let child = only_child();
        assert_eq!(prop(&child, "MarkerSize"), Value::Num(20.0));
        assert_eq!(prop(&child, "LineStyle"), Value::String("none".into()));
        assert_eq!(xy(&child), (vec![1.0, 2.0], vec![1.0, 2.0]));
    }

    /// [spy.input-types, spy.positions] cases `dense`, `logical`, `char`.
    #[test]
    fn observed_dense_logical_char_inputs() {
        let _guard = setup();
        // dense [0 7; 3 0]
        spy(&[matrix(2, 2, &[0.0, 3.0, 7.0, 0.0])]);
        assert_eq!(xy(&only_child()), (vec![1.0, 2.0], vec![2.0, 1.0]));

        fresh();
        // logical([1 0; 1 1])
        let logical =
            Value::LogicalArray(LogicalArray::new(vec![1, 1, 0, 1], vec![2, 2]).expect("logical"));
        spy(&[logical]);
        assert_eq!(
            xy(&only_child()),
            (vec![1.0, 1.0, 2.0], vec![1.0, 2.0, 2.0])
        );

        fresh();
        spy(&[text("ab")]);
        assert_eq!(xy(&only_child()), (vec![1.0, 2.0], vec![1.0, 1.0]));
        assert_eq!(numbers(&prop(&gca(), "YLim")), vec![0.0, 2.0]);
        assert_eq!(numbers(&prop(&gca(), "XLim")), vec![0.0, 3.0]);
    }

    // ---- unresolved (spy.q-large) and unobserved forms ----

    /// spy.q-large: complex input is unresolved.
    #[test]
    fn unresolved_choice_complex_input_unsupported() {
        let _guard = setup();
        let before = axes_children();
        let z = Value::ComplexTensor(
            ComplexTensor::new(vec![(1.0, 1.0), (0.0, 0.0)], vec![1, 2]).expect("complex"),
        );
        let err = direct(&[z]).expect_err("complex must fail");
        assert_eq!(identifier(&err), "RunMat:spy:UnsupportedOption");
        assert_eq!(axes_children(), before, "nothing may be drawn");
    }

    /// Unobserved forms: LineSpec with a line style, reversed argument
    /// order, an axes target, extra or name-value arguments.
    #[test]
    fn unresolved_choice_unobserved_forms_unsupported() {
        let _guard = setup();
        let before = axes_children();
        let a = matrix(2, 2, &[1.0, 0.0, 0.0, 1.0]);
        let cases: Vec<Vec<Value>> = vec![
            vec![a.clone(), text("r-")],
            vec![a.clone(), text("b:o")],
            vec![a.clone(), Value::Num(12.0), text("r*")],
            vec![gca(), a.clone()],
            vec![a.clone(), text("r*"), Value::Num(5.0), Value::Num(1.0)],
            vec![a.clone(), text("Color"), text("r")],
        ];
        for args in cases {
            let err = direct(&args).expect_err("unobserved form must fail");
            assert_eq!(
                identifier(&err),
                "RunMat:spy:UnsupportedOption",
                "args {args:?}"
            );
        }
        assert_eq!(axes_children(), before, "nothing may be drawn");
    }

    // ---- validation ----

    /// Non-numeric inputs are rejected.
    #[test]
    fn rejects_non_numeric_inputs() {
        let _guard = setup();
        for input in [
            Value::String("abc".into()),
            Value::Cell(runmat_value::CellArray::new(vec![Value::Num(1.0)], 1, 1).unwrap()),
            Value::Struct(runmat_value::StructValue::new()),
        ] {
            let err = direct(&[input]).expect_err("non-numeric must fail");
            assert_eq!(identifier(&err), "RunMat:spy:InvalidInput");
        }
    }

    /// N-D arrays and a missing argument are rejected.
    #[test]
    fn rejects_nd_and_missing_input() {
        let _guard = setup();
        let nd = Value::Tensor(Tensor::new(vec![1.0; 8], vec![2, 2, 2]).expect("nd"));
        let err = direct(&[nd]).expect_err("N-D must fail");
        assert_eq!(identifier(&err), "RunMat:spy:InvalidInput");
        let err = direct(&[]).expect_err("missing A must fail");
        assert_eq!(identifier(&err), "RunMat:spy:InvalidInput");
    }

    /// Invalid LineSpec characters and marker sizes are rejected.
    #[test]
    fn rejects_invalid_style_arguments() {
        let _guard = setup();
        let a = matrix(2, 2, &[1.0, 0.0, 0.0, 1.0]);
        for style in [
            vec![text("q")],
            vec![text("")],
            vec![Value::Num(-1.0)],
            vec![Value::Num(0.0)],
            vec![Value::Num(f64::NAN)],
            vec![text("r*"), Value::Num(f64::INFINITY)],
        ] {
            let mut args = vec![a.clone()];
            args.extend(style);
            let err = direct(&args).expect_err("invalid style must fail");
            assert_eq!(identifier(&err), "RunMat:spy:InvalidInput", "args {args:?}");
        }
    }
}
