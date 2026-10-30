//! `alpha` builtin: set the face transparency of surfaces and patches
//! (spec 051-graphics-n2).
//!
//! Normative behaviour comes from the approved `alpha` 0.2.0 export: a
//! numeric value or the keywords `'clear'` (0) and `'opaque'` (1) set
//! `FaceAlpha` of every surface and patch in the current axes, or of a given
//! object; `EdgeAlpha` is left alone; axes holding only lines are accepted;
//! no output is returned. RunMat stores `FaceAlpha` as a single-precision
//! number and has no per-vertex alpha data, so the `'flat'` and `'interp'`
//! keywords are `UnsupportedOption` (deviation D-051-2); see
//! `specs/051-graphics-n2/spec.md`.

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::Value;

use crate::builtins::plotting::properties::{resolve_plot_handle, PlotHandle};
use crate::builtins::plotting::set::set_builtin;
use crate::builtins::plotting::state::{
    current_axes_state, plot_child_handle_snapshot, plot_child_handles_for_axes,
    PlotChildHandleState,
};
use crate::builtins::plotting::FigureHandle;
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const BUILTIN_NAME: &str = "alpha";

const ALPHA_INPUT_OBJ: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "obj",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Surface, patch or axes handle.",
};
const ALPHA_INPUT_VALUE: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "value",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Scalar in [0, 1], or 'clear' (0) or 'opaque' (1).",
};
const ALPHA_INPUTS_VALUE: [BuiltinParamDescriptor; 1] = [ALPHA_INPUT_VALUE];
const ALPHA_INPUTS_OBJ_VALUE: [BuiltinParamDescriptor; 2] = [ALPHA_INPUT_OBJ, ALPHA_INPUT_VALUE];
const ALPHA_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "alpha(value)",
        inputs: &ALPHA_INPUTS_VALUE,
        outputs: &[],
    },
    BuiltinSignatureDescriptor {
        label: "alpha(obj, value)",
        inputs: &ALPHA_INPUTS_OBJ_VALUE,
        outputs: &[],
    },
];

const fn alpha_error_descriptor(
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

const ERR_INVALID_INPUT: BuiltinErrorDescriptor = alpha_error_descriptor(
    "RM.ALPHA.INVALID_INPUT",
    "RunMat:alpha:InvalidInput",
    "No value is given, or the object argument is not a valid graphics handle.",
    "alpha: invalid input",
);
const ERR_UNKNOWN_VALUE: BuiltinErrorDescriptor = alpha_error_descriptor(
    "RM.ALPHA.UNKNOWN_VALUE",
    "RunMat:alpha:UnknownAlphaValue",
    "The value is text other than 'clear', 'opaque', 'flat' or 'interp'.",
    "alpha: unrecognised alpha value",
);
const ERR_UNSUPPORTED_OPTION: BuiltinErrorDescriptor = alpha_error_descriptor(
    "RM.ALPHA.UNSUPPORTED_OPTION",
    "RunMat:alpha:UnsupportedOption",
    "'flat'/'interp', a numeric value outside [0, 1] or not a real scalar, an object other than a surface, patch or axes, or more than two arguments.",
    "alpha: unsupported option",
);
const ERR_TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = alpha_error_descriptor(
    "RM.ALPHA.TOO_MANY_OUTPUTS",
    "RunMat:alpha:TooManyOutputs",
    "An output argument is requested.",
    "alpha does not return output arguments",
);
const ERR_INTERNAL: BuiltinErrorDescriptor = alpha_error_descriptor(
    "RM.ALPHA.INTERNAL",
    "RunMat:alpha:Internal",
    "Updating a graphics object fails.",
    "alpha: internal error",
);
const ALPHA_ERRORS: [BuiltinErrorDescriptor; 5] = [
    ERR_INVALID_INPUT,
    ERR_UNKNOWN_VALUE,
    ERR_UNSUPPORTED_OPTION,
    ERR_TOO_MANY_OUTPUTS,
    ERR_INTERNAL,
];

pub const ALPHA_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &ALPHA_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ALPHA_ERRORS,
};

#[runtime_builtin(
    name = "alpha",
    category = "plotting",
    summary = "Set the face transparency of surfaces and patches.",
    keywords = "alpha,transparency,facealpha,surface,patch,plotting",
    sink = true,
    suppress_auto_output = true,
    descriptor(crate::builtins::plotting::ops::alpha::ALPHA_DESCRIPTOR),
    builtin_path = "crate::builtins::plotting::ops::alpha"
)]
pub async fn alpha_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    if matches!(crate::output_count::current_output_count(), Some(n) if n > 0) {
        return Err(alpha_error(
            &ERR_TOO_MANY_OUTPUTS,
            "no output value is returned",
        ));
    }
    let mut gathered = Vec::with_capacity(args.len());
    for value in &args {
        gathered.push(
            gather_if_needed_async(value)
                .await
                .map_err(|err| alpha_error(&ERR_INTERNAL, err.message()))?,
        );
    }
    let (target, value) = match gathered.as_slice() {
        [] => {
            return Err(alpha_error(
                &ERR_INVALID_INPUT,
                "expected alpha(value) or alpha(obj, value)",
            ))
        }
        [value] => (None, value),
        [obj, value] => (Some(obj), value),
        _ => {
            return Err(alpha_error(
                &ERR_UNSUPPORTED_OPTION,
                "more than two arguments are not supported",
            ))
        }
    };
    let alpha = parse_alpha_value(value)?;
    let objects = match target {
        None => {
            let axes = current_axes_state();
            face_objects_in_axes(axes.handle, axes.active_index)
        }
        Some(obj) => target_objects(obj)?,
    };
    for handle in objects {
        set_builtin(vec![
            Value::Num(handle),
            Value::String("FaceAlpha".into()),
            Value::Num(alpha),
        ])
        .map_err(|err| alpha_error(&ERR_INTERNAL, err.message()))?;
    }
    Ok(Value::OutputList(Vec::new()))
}

/// Parse the alpha argument: a real finite scalar in [0, 1], or a keyword.
fn parse_alpha_value(value: &Value) -> BuiltinResult<f64> {
    if let Some(text) = text_scalar(value) {
        return match text.trim().to_ascii_lowercase().as_str() {
            "clear" => Ok(0.0),
            "opaque" => Ok(1.0),
            "flat" | "interp" => Err(alpha_error(
                &ERR_UNSUPPORTED_OPTION,
                format!("'{text}' needs per-vertex alpha data, which RunMat does not store"),
            )),
            _ => Err(alpha_error(
                &ERR_UNKNOWN_VALUE,
                format!("'{text}' is not a recognised alpha value"),
            )),
        };
    }
    let scalar = match value {
        Value::Num(v) => Some(*v),
        Value::Int(i) => Some(i.to_f64()),
        Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        Value::Tensor(t) => {
            let data = t.materialize_f64();
            (data.len() == 1).then(|| data[0])
        }
        _ => None,
    };
    match scalar {
        Some(v) if v.is_finite() && (0.0..=1.0).contains(&v) => Ok(v),
        _ => Err(alpha_error(
            &ERR_UNSUPPORTED_OPTION,
            "the alpha value must be a real scalar in [0, 1]",
        )),
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

/// Surface and patch handles parented to the given axes.
fn face_objects_in_axes(figure: FigureHandle, axes_index: usize) -> Vec<f64> {
    plot_child_handles_for_axes(figure, axes_index)
        .into_iter()
        .filter(|handle| {
            matches!(
                plot_child_handle_snapshot(*handle),
                Ok(PlotChildHandleState::Surface(_)) | Ok(PlotChildHandleState::Patch(_))
            )
        })
        .collect()
}

fn target_objects(obj: &Value) -> BuiltinResult<Vec<f64>> {
    let handle = resolve_plot_handle(obj, BUILTIN_NAME)
        .map_err(|err| alpha_error(&ERR_INVALID_INPUT, err.message()))?;
    match handle {
        PlotHandle::Axes(figure, axes_index) => Ok(face_objects_in_axes(figure, axes_index)),
        PlotHandle::PlotChild(id, state) => match *state {
            PlotChildHandleState::Surface(_) | PlotChildHandleState::Patch(_) => Ok(vec![id]),
            _ => Err(alpha_error(
                &ERR_UNSUPPORTED_OPTION,
                "only surface, patch and axes objects are supported",
            )),
        },
        _ => Err(alpha_error(
            &ERR_UNSUPPORTED_OPTION,
            "only surface, patch and axes objects are supported",
        )),
    }
}

fn alpha_error(error: &'static BuiltinErrorDescriptor, detail: impl AsRef<str>) -> RuntimeError {
    let mut builder = build_runtime_error(format!("{}: {}", error.message, detail.as_ref()))
        .with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

/// Spec 051 (`alpha` 0.2.0). `observed_*` tests reproduce confirmed
/// observations; `unresolved_choice_*`, `deviation_*` and `rejects_*` tests
/// pin RunMat's documented choices and are not claims about MATLAB.
#[cfg(test)]
mod alpha_tests {
    use crate::builtins::plotting::state::{clear_figure, reset_hold_state_for_run};
    use crate::builtins::plotting::tests::{ensure_plot_test_env, lock_plot_registry};
    use crate::{BuiltinResult, RuntimeError};
    use futures::executor::block_on;
    use runmat_value::{CharArray, Tensor, Value};

    /// FaceAlpha is stored as f32 upstream (deviation D-051-1).
    const F32_TOL: f64 = 1e-6;

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

    /// Error paths call the builtin directly (dispatcher retry, see the
    /// 040-spy validation finding F-1).
    fn direct(args: &[Value]) -> BuiltinResult<Value> {
        block_on(super::alpha_builtin(args.to_vec()))
    }

    fn text(s: &str) -> Value {
        Value::CharArray(CharArray::new_row(s))
    }

    fn identifier(err: &RuntimeError) -> String {
        err.identifier().unwrap_or_default().to_string()
    }

    fn magic3() -> Value {
        Value::Tensor(
            Tensor::new(
                vec![8.0, 3.0, 4.0, 1.0, 5.0, 9.0, 6.0, 7.0, 2.0],
                vec![3, 3],
            )
            .expect("magic(3)"),
        )
    }

    fn row(values: &[f64]) -> Value {
        Value::Tensor(Tensor::new(values.to_vec(), vec![1, values.len()]).expect("row"))
    }

    fn surf() -> Value {
        call("surf", &[magic3()]).expect("surf")
    }

    fn patch() -> Value {
        call(
            "patch",
            &[row(&[0.0, 1.0, 0.0]), row(&[0.0, 0.0, 1.0]), text("r")],
        )
        .expect("patch")
    }

    fn prop(h: &Value, name: &str) -> Value {
        call("get", &[h.clone(), text(name)]).expect("get")
    }

    fn num(value: &Value) -> f64 {
        match value {
            Value::Num(v) => *v,
            Value::Tensor(t) if t.materialize_f64().len() == 1 => t.materialize_f64()[0],
            other => panic!("expected a numeric scalar, got {other:?}"),
        }
    }

    fn assert_face_alpha(h: &Value, want: f64) {
        let got = num(&prop(h, "FaceAlpha"));
        assert!((got - want).abs() <= F32_TOL, "FaceAlpha {got} vs {want}");
    }

    #[test]
    fn observed_scalar_sets_surface_face_alpha() {
        let _g = setup();
        let s = surf();
        let out = call("alpha", &[Value::Num(0.5)]).expect("alpha(0.5)");
        assert_eq!(
            out,
            Value::OutputList(Vec::new()),
            "alpha returns no output"
        );
        assert_face_alpha(&s, 0.5);
    }

    #[test]
    fn observed_scalar_sets_patch_face_alpha() {
        let _g = setup();
        let p = patch();
        call("alpha", &[Value::Num(0.4)]).expect("alpha(0.4)");
        assert_face_alpha(&p, 0.4);
    }

    #[test]
    fn observed_object_form_sets_given_surface() {
        let _g = setup();
        let s = surf();
        call("alpha", &[s.clone(), Value::Num(0.3)]).expect("alpha(s, 0.3)");
        assert_face_alpha(&s, 0.3);
    }

    #[test]
    fn observed_clear_and_opaque_keywords() {
        let _g = setup();
        let s = surf();
        call("alpha", &[text("clear")]).expect("alpha('clear')");
        assert_face_alpha(&s, 0.0);
        call("alpha", &[Value::Num(0.2)]).expect("alpha(0.2)");
        call("alpha", &[text("opaque")]).expect("alpha('opaque')");
        assert_face_alpha(&s, 1.0);
    }

    #[test]
    fn observed_edge_alpha_untouched() {
        let _g = setup();
        // RunMat surfaces expose no EdgeAlpha; patches do. Observed on a
        // surface; checked here on a patch (deviation D-051-3).
        let p = patch();
        call("alpha", &[Value::Num(0.5)]).expect("alpha(0.5)");
        assert_eq!(num(&prop(&p, "EdgeAlpha")), 1.0);
        assert_face_alpha(&p, 0.5);
    }

    #[test]
    fn observed_line_only_axes_completes() {
        let _g = setup();
        let h = call("plot", &[row(&[1.0, 2.0, 3.0])]).expect("plot");
        call("alpha", &[Value::Num(0.5)]).expect("alpha with only a line");
        assert!(matches!(prop(&h, "Type"), Value::String(ref t) if t == "line"));
    }

    #[test]
    fn observed_output_request_errors() {
        let _g = setup();
        surf();
        let _outputs = crate::output_count::push_output_count(Some(1));
        let err = direct(&[Value::Num(0.5)]).expect_err("a = alpha(0.5)");
        assert_eq!(identifier(&err), "RunMat:alpha:TooManyOutputs");
    }

    #[test]
    fn observed_unknown_keyword_errors() {
        let _g = setup();
        surf();
        let err = direct(&[text("nope")]).expect_err("alpha('nope')");
        assert_eq!(identifier(&err), "RunMat:alpha:UnknownAlphaValue");
    }

    #[test]
    fn deviation_flat_and_interp_unsupported() {
        let _g = setup();
        let s = surf();
        for keyword in ["flat", "interp"] {
            let err = direct(&[text(keyword)]).expect_err(keyword);
            assert_eq!(identifier(&err), "RunMat:alpha:UnsupportedOption");
        }
        assert_face_alpha(&s, 1.0);
    }

    #[test]
    fn unresolved_choice_mixed_axes_and_axes_target() {
        let _g = setup();
        call("hold", &[text("on")]).expect("hold on");
        let s = surf();
        let p = patch();
        let l = call("plot", &[row(&[1.0, 2.0, 3.0])]).expect("plot");
        call("alpha", &[Value::Num(0.25)]).expect("alpha(0.25)");
        assert_face_alpha(&s, 0.25);
        assert_face_alpha(&p, 0.25);
        assert!(matches!(prop(&l, "Type"), Value::String(ref t) if t == "line"));
        let ax = call("gca", &[]).expect("gca");
        call("alpha", &[ax, Value::Num(0.75)]).expect("alpha(ax, 0.75)");
        assert_face_alpha(&s, 0.75);
        assert_face_alpha(&p, 0.75);
    }

    #[test]
    fn unresolved_choice_keywords_case_insensitive_and_string_scalar() {
        let _g = setup();
        let s = surf();
        call("alpha", &[Value::String("CLEAR".into())]).expect("alpha(\"CLEAR\")");
        assert_face_alpha(&s, 0.0);
    }

    #[test]
    fn unresolved_choice_unobserved_values_and_targets_unsupported() {
        let _g = setup();
        let s = surf();
        let l = call("plot", &[row(&[1.0, 2.0])]).expect("plot");
        for args in [
            vec![Value::Num(1.5)],
            vec![Value::Num(-0.1)],
            vec![Value::Num(f64::NAN)],
            vec![row(&[0.1, 0.2])],
            vec![l.clone(), Value::Num(0.5)],
            vec![s.clone(), Value::Num(0.5), Value::Num(0.5)],
        ] {
            let err = direct(&args).expect_err("unsupported alpha form");
            assert_eq!(identifier(&err), "RunMat:alpha:UnsupportedOption");
        }
    }

    #[test]
    fn rejects_missing_value_and_invalid_handle() {
        let _g = setup();
        let err = direct(&[]).expect_err("alpha()");
        assert_eq!(identifier(&err), "RunMat:alpha:InvalidInput");
        let err = direct(&[Value::Num(987654.0), Value::Num(0.5)]).expect_err("bad handle");
        assert_eq!(identifier(&err), "RunMat:alpha:InvalidInput");
    }
}
