//! `findall` builtin: find graphics objects under a handle, including
//! objects such as axes labels (spec 058).
//!
//! Normative behaviour comes from the approved `findall` 0.2.0 export
//! (claims `findall.type`, `findall.hidden`, `findall.property`,
//! `findall.set-pattern`, `findall.self`, `findall.no-match`,
//! `findall.deleted`, `findall.error-not-handle`). RunMat has no
//! `HandleVisibility`, so every object is visible to the search. The
//! object tree used here is shared with `allchild`. See
//! `specs/058-graphics-n3n4/`.

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{Tensor, Value};

use crate::builtins::plotting::properties::{get_properties, resolve_plot_handle, PlotHandle};
use crate::builtins::plotting::state::{self, FigureHandle, PlotObjectKind};
use crate::builtins::plotting::type_resolvers::handle_array_type;
use crate::{build_runtime_error, BuiltinResult, RuntimeError};

const BUILTIN_NAME: &str = "findall";

const FINDALL_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "h",
    ty: BuiltinParamType::NumericArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Column of matching graphics handles (0x0 when nothing matches).",
}];

const FINDALL_INPUT_OBJ: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "obj",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Graphics handle (or handle array) to search under.",
};
const FINDALL_INPUTS_OBJ: [BuiltinParamDescriptor; 1] = [FINDALL_INPUT_OBJ];
const FINDALL_INPUTS_TYPE: [BuiltinParamDescriptor; 3] = [
    FINDALL_INPUT_OBJ,
    BuiltinParamDescriptor {
        name: "'Type'",
        ty: BuiltinParamType::StringScalar,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "The keyword 'Type'.",
    },
    BuiltinParamDescriptor {
        name: "t",
        ty: BuiltinParamType::StringScalar,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Object type to keep.",
    },
];
const FINDALL_INPUTS_PROPERTY: [BuiltinParamDescriptor; 3] = [
    FINDALL_INPUT_OBJ,
    BuiltinParamDescriptor {
        name: "'-property'",
        ty: BuiltinParamType::StringScalar,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "The keyword '-property'.",
    },
    BuiltinParamDescriptor {
        name: "name",
        ty: BuiltinParamType::StringScalar,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Property the objects must have.",
    },
];

const FINDALL_SIGNATURES: [BuiltinSignatureDescriptor; 3] = [
    BuiltinSignatureDescriptor {
        label: "h = findall(obj)",
        inputs: &FINDALL_INPUTS_OBJ,
        outputs: &FINDALL_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "h = findall(obj, 'Type', t)",
        inputs: &FINDALL_INPUTS_TYPE,
        outputs: &FINDALL_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "h = findall(obj, '-property', name)",
        inputs: &FINDALL_INPUTS_PROPERTY,
        outputs: &FINDALL_OUTPUT,
    },
];

const FINDALL_ERROR_INVALID_HANDLES: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.FINDALL.INVALID_HANDLES",
    identifier: Some("RunMat:findall:InvalidHandles"),
    when: "The first argument is not a graphics handle (non-integer, negative or non-finite number, or a non-numeric value).",
    message: "findall: invalid handles",
};
const FINDALL_ERROR_MINRHS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.FINDALL.MINRHS",
    identifier: Some("RunMat:minrhs"),
    when: "No input argument is given.",
    message: "findall: not enough input arguments",
};
const FINDALL_ERROR_UNSUPPORTED_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.FINDALL.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:findall:UnsupportedOption"),
    when: "A query other than 'Type', value and '-property', name pairs (not covered by the approved specification).",
    message: "findall: unsupported option",
};
const FINDALL_ERRORS: [BuiltinErrorDescriptor; 3] = [
    FINDALL_ERROR_INVALID_HANDLES,
    FINDALL_ERROR_MINRHS,
    FINDALL_ERROR_UNSUPPORTED_OPTION,
];

pub const FINDALL_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &FINDALL_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &FINDALL_ERRORS,
};

#[runtime_builtin(
    name = "findall",
    category = "plotting",
    summary = "Find all graphics objects under a handle, including label objects.",
    keywords = "findall,findobj,graphics,handle,plotting",
    suppress_auto_output = true,
    type_resolver(handle_array_type),
    descriptor(crate::builtins::plotting::ops::findall::FINDALL_DESCRIPTOR),
    builtin_path = "crate::builtins::plotting::ops::findall"
)]
pub fn findall_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    let Some(first) = args.first() else {
        return Err(findall_error(
            &FINDALL_ERROR_MINRHS,
            "a graphics handle is required",
        ));
    };
    let roots = root_handles(first)?;
    let filters = parse_filters(&args[1..])?;
    let mut found = Vec::new();
    for root in roots {
        let Some(root) = root else { continue };
        collect(root, &filters, &mut found);
    }
    Ok(handle_column(found))
}

enum Filter {
    Type(String),
    Property(String),
}

fn parse_filters(args: &[Value]) -> BuiltinResult<Vec<Filter>> {
    if !args.len().is_multiple_of(2) {
        return Err(findall_error(
            &FINDALL_ERROR_UNSUPPORTED_OPTION,
            "queries must be 'Type', value or '-property', name pairs",
        ));
    }
    args.chunks(2)
        .map(|pair| {
            let key = text_scalar(&pair[0]).map(|k| k.to_ascii_lowercase());
            let value = text_scalar(&pair[1]);
            match (key.as_deref(), value) {
                (Some("type"), Some(t)) => Ok(Filter::Type(t.trim().to_ascii_lowercase())),
                (Some("-property"), Some(name)) => Ok(Filter::Property(name.trim().to_string())),
                _ => Err(findall_error(
                    &FINDALL_ERROR_UNSUPPORTED_OPTION,
                    "only 'Type', value and '-property', name queries are supported",
                )),
            }
        })
        .collect()
}

/// Resolve the search roots. Live handles resolve to `Some`. A positive
/// integer that does not name a live object is treated as a deleted handle
/// and yields `None` (deviation D-058-1: RunMat numeric handles carry no
/// deleted state). Anything else is not a graphics handle.
fn root_handles(value: &Value) -> BuiltinResult<Vec<Option<(f64, PlotHandle)>>> {
    let numbers: Vec<f64> = match value {
        Value::Num(v) => vec![*v],
        Value::Int(i) => vec![i.to_f64()],
        Value::Tensor(t) => t.materialize_f64().to_vec(),
        _ => {
            return Err(findall_error(
                &FINDALL_ERROR_INVALID_HANDLES,
                "the first argument must be a graphics handle",
            ))
        }
    };
    numbers
        .into_iter()
        .map(|h| match resolve_live(h) {
            Some(resolved) => Ok(Some((h, resolved))),
            None if h.is_finite() && h > 0.0 && h.fract() == 0.0 => Ok(None),
            None => Err(findall_error(
                &FINDALL_ERROR_INVALID_HANDLES,
                format!("{h} is not a graphics handle"),
            )),
        })
        .collect()
}

/// A live object named by `handle`. RunMat handles are non-negative
/// integers; other numbers never name an object.
pub(crate) fn resolve_live(handle: f64) -> Option<PlotHandle> {
    if !handle.is_finite() || handle < 0.0 || handle.fract() != 0.0 {
        return None;
    }
    resolve_plot_handle(&Value::Num(handle), BUILTIN_NAME).ok()
}

/// Depth-first, pre-order: the object itself, then its children and its
/// label objects, recursively.
fn collect(node: (f64, PlotHandle), filters: &[Filter], out: &mut Vec<f64>) {
    let (handle, resolved) = node;
    if filters.iter().all(|f| matches(&resolved, f)) {
        out.push(handle);
    }
    let mut next = graphics_children(&resolved);
    next.extend(label_objects(&resolved));
    for child in next {
        if let Some(r) = resolve_live(child) {
            collect((child, r), filters, out);
        }
    }
}

fn matches(handle: &PlotHandle, filter: &Filter) -> bool {
    match filter {
        Filter::Type(t) => object_type(handle).is_some_and(|ty| ty.eq_ignore_ascii_case(t)),
        Filter::Property(name) => get_properties(handle.clone(), Some(name), BUILTIN_NAME).is_ok(),
    }
}

pub(crate) fn object_type(handle: &PlotHandle) -> Option<String> {
    match get_properties(handle.clone(), Some("Type"), BUILTIN_NAME).ok()? {
        Value::String(s) => Some(s),
        Value::CharArray(ca) => Some(ca.data.iter().collect()),
        _ => None,
    }
}

/// Children in the parent/child tree (the `allchild` set): figures of the
/// root; axes (and a figure-level title) of a figure; plot objects of an
/// axes. Every other object has no children.
pub(crate) fn graphics_children(handle: &PlotHandle) -> Vec<f64> {
    match handle {
        PlotHandle::Root => state::root_figure_handles()
            .into_iter()
            .map(|figure| figure.as_u32() as f64)
            .collect(),
        PlotHandle::Figure(figure) => {
            let mut children = state::axes_handles_for_figure(*figure).unwrap_or_default();
            if state::figure_has_sg_title(*figure) {
                children.push(state::encode_plot_object_handle(
                    *figure,
                    0,
                    PlotObjectKind::SuperTitle,
                ));
            }
            children
        }
        PlotHandle::Axes(figure, axes) => state::plot_child_handles_for_axes(*figure, *axes),
        _ => Vec::new(),
    }
}

/// Text objects held by an axes (title, subtitle and axis labels with
/// non-empty text). They are not children in the `allchild` sense but are found
/// by `findall` [findall.hidden]. Legends are not searched: RunMat enables
/// a legend object on every axes with plot entries by default, which no
/// observation shows (documented choice).
fn label_objects(handle: &PlotHandle) -> Vec<f64> {
    let PlotHandle::Axes(figure, axes) = handle else {
        return Vec::new();
    };
    let (figure, axes): (FigureHandle, usize) = (*figure, *axes);
    let Ok(meta) = state::axes_metadata_snapshot(figure, axes) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut push = |text: &Option<String>, kind: PlotObjectKind| {
        if text.as_deref().is_some_and(|t| !t.is_empty()) {
            out.push(state::encode_plot_object_handle(figure, axes, kind));
        }
    };
    push(&meta.title, PlotObjectKind::Title);
    push(&meta.subtitle, PlotObjectKind::Subtitle);
    push(&meta.x_label, PlotObjectKind::XLabel);
    push(&meta.y_label, PlotObjectKind::YLabel);
    push(&meta.z_label, PlotObjectKind::ZLabel);
    out
}

/// Handles as a column, or a 0x0 empty array (deviation D-058-3: RunMat's
/// empty handle array is a 0x0 double, not a GraphicsPlaceholder array).
pub(crate) fn handle_column(handles: Vec<f64>) -> Value {
    let len = handles.len();
    if len == 0 {
        return Value::Tensor(Tensor::new(Vec::new(), vec![0, 0]).expect("empty handles"));
    }
    Value::Tensor(Tensor::new(handles, vec![len, 1]).expect("handle column"))
}

fn text_scalar(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::StringArray(sa) if sa.data.len() == 1 => Some(sa.data[0].clone()),
        Value::CharArray(ca) if ca.rows == 1 => Some(ca.data.iter().collect()),
        _ => None,
    }
}

fn findall_error(error: &'static BuiltinErrorDescriptor, detail: impl AsRef<str>) -> RuntimeError {
    let mut builder = build_runtime_error(format!("{}: {}", error.message, detail.as_ref()))
        .with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

#[cfg(test)]
mod findall_tests {
    use super::findall_builtin;
    use crate::builtins::plotting::ops::pcolor::pcolor_tests::{
        call, identifier, matrix, prop, setup, shaped, string, text,
    };
    use crate::BuiltinResult;
    use runmat_value::{StructValue, Value};

    fn direct(args: &[Value]) -> BuiltinResult<Value> {
        findall_builtin(args.to_vec())
    }

    fn handles(value: &Value) -> Vec<f64> {
        shaped(value).1
    }

    fn types(found: &Value) -> Vec<String> {
        let mut out: Vec<String> = handles(found)
            .into_iter()
            .map(|h| string(prop(h, "Type")))
            .collect();
        out.sort();
        out.dedup();
        out
    }

    /// The observed scene: a figure with two lines (hold on). RunMat's
    /// `plot` sets default axis labels 'X' and 'Y', which the observed scene
    /// does not have; they are blanked so the scene matches (D-058-9).
    fn two_lines() -> Value {
        let f = call("gcf", &[]).expect("gcf");
        call("plot", &[matrix(&[&[1.0, 2.0, 3.0]])]).expect("plot");
        call("hold", &[text("on")]).expect("hold");
        call("plot", &[matrix(&[&[3.0, 2.0, 1.0]])]).expect("plot");
        call("xlabel", &[text("")]).expect("xlabel");
        call("ylabel", &[text("")]).expect("ylabel");
        f
    }

    // ---- observed (normative) ----

    /// [findall.type] cases `type-line`, `type-axes`.
    #[test]
    fn observed_type_filter() {
        let _guard = setup();
        let f = two_lines();
        let lines = direct(&[f.clone(), text("Type"), text("line")]).unwrap();
        assert_eq!(shaped(&lines).0, vec![2, 1]);
        assert_eq!(types(&lines), vec!["line"]);
        let axes = direct(&[f, text("Type"), text("axes")]).unwrap();
        assert_eq!(shaped(&axes).0, vec![1, 1]);
        assert_eq!(handles(&axes)[0], shaped(&call("gca", &[]).unwrap()).1[0]);
    }

    /// [findall.hidden] case `type-text-hidden`: the title and xlabel are
    /// found. (RunMat's `findobj` finds them as well: D-058-2.)
    #[test]
    fn observed_finds_title_and_label_text() {
        let _guard = setup();
        let f = two_lines();
        call("title", &[text("t")]).expect("title");
        call("xlabel", &[text("x")]).expect("xlabel");
        let found = direct(&[f, text("Type"), text("text")]).unwrap();
        assert_eq!(handles(&found).len(), 2);
    }

    /// [findall.property] case `property-fontsize`.
    #[test]
    fn observed_property_filter() {
        let _guard = setup();
        let f = two_lines();
        call("title", &[text("t")]).expect("title");
        call("xlabel", &[text("x")]).expect("xlabel");
        call("text", &[Value::Num(1.0), Value::Num(1.0), text("a")]).expect("text");
        let found = direct(&[f, text("-property"), text("FontSize")]).unwrap();
        assert_eq!(types(&found), vec!["axes", "text"]);
    }

    /// [findall.set-pattern] case `set-fontsize`. RunMat's `set` takes one
    /// handle at a time (deviation D-058-8), so each found handle is set.
    #[test]
    fn observed_set_pattern() {
        let _guard = setup();
        let f = two_lines();
        call("title", &[text("t")]).expect("title");
        let t = call("text", &[Value::Num(1.0), Value::Num(1.0), text("a")]).expect("text");
        let found = direct(&[f, text("-property"), text("FontSize")]).unwrap();
        for h in handles(&found) {
            call("set", &[Value::Num(h), text("FontSize"), Value::Num(12.0)]).expect("set");
        }
        let ax = call("gca", &[]).unwrap();
        assert_eq!(
            shaped(&call("get", &[ax, text("FontSize")]).unwrap()).1,
            vec![12.0]
        );
        assert_eq!(
            shaped(&call("get", &[t, text("FontSize")]).unwrap()).1,
            vec![12.0]
        );
    }

    /// [findall.set-pattern] exactly: `set` on the whole handle column.
    /// Not supported by RunMat's `set` (deviation D-058-8).
    #[test]
    #[ignore = "D-058-8: RunMat's set does not accept handle arrays"]
    fn deviation_set_on_handle_array() {
        let _guard = setup();
        let f = two_lines();
        call("text", &[Value::Num(1.0), Value::Num(1.0), text("a")]).expect("text");
        let found = direct(&[f, text("-property"), text("FontSize")]).unwrap();
        call("set", &[found, text("FontSize"), Value::Num(12.0)]).expect("set");
        let ax = call("gca", &[]).unwrap();
        assert_eq!(
            shaped(&call("get", &[ax, text("FontSize")]).unwrap()).1,
            vec![12.0]
        );
    }

    /// [findall.self] case `self`.
    #[test]
    fn observed_includes_the_object_itself_first() {
        let _guard = setup();
        let f = two_lines();
        let found = direct(std::slice::from_ref(&f)).unwrap();
        let hs = handles(&found);
        assert_eq!(shaped(&found).0, vec![hs.len(), 1]);
        assert_eq!(hs[0], shaped(&f).1[0]);
        assert_eq!(string(prop(hs[0], "Type")), "figure");
    }

    /// [findall.no-match] case `no-match` (0x0 representation: D-058-3).
    #[test]
    fn observed_no_match_is_empty() {
        let _guard = setup();
        let f = two_lines();
        let found = direct(&[f, text("Type"), text("surface")]).unwrap();
        assert_eq!(shaped(&found).0, vec![0, 0]);
    }

    /// [findall.deleted] case `deleted`.
    #[test]
    fn observed_deleted_figure_is_empty() {
        let _guard = setup();
        let f = call("figure", &[]).expect("figure");
        call("close", std::slice::from_ref(&f)).expect("close");
        let found = direct(&[f]).unwrap();
        assert_eq!(shaped(&found).0, vec![0, 0]);
    }

    /// [findall.error-not-handle] for numbers RunMat can tell are not
    /// handles (non-integer, negative).
    #[test]
    fn observed_non_handle_number_errors() {
        let _guard = setup();
        for bad in [1.5, -2.0, f64::NAN] {
            let err = direct(&[Value::Num(bad)]).unwrap_err();
            assert_eq!(identifier(&err), "RunMat:findall:InvalidHandles");
        }
    }

    /// [findall.error-not-handle] case `err-not-handle` exactly
    /// (`findall(123456)`). Not reproduced: deviation D-058-1.
    #[test]
    #[ignore = "D-058-1: a positive integer that is not live is treated as a deleted handle"]
    fn deviation_integer_non_handle_errors() {
        let _guard = setup();
        let err = direct(&[Value::Num(123456.0)]).unwrap_err();
        assert_eq!(identifier(&err), "RunMat:findall:InvalidHandles");
    }

    // ---- documented choices ----

    /// (choice) Rulers and legends are not part of the search; Type/-property queries
    /// combine with AND; other queries are unsupported; non-numeric first
    /// arguments are not handles; no arguments raise minrhs; handle arrays
    /// are searched in order.
    #[test]
    fn unresolved_choice_queries_and_inputs() {
        let _guard = setup();
        let f = two_lines();
        let all = types(&direct(std::slice::from_ref(&f)).unwrap());
        assert!(
            !all.iter().any(|t| t.contains("ruler") || t == "legend"),
            "{all:?}"
        );
        call("title", &[text("t")]).expect("title");
        let both = direct(&[
            f.clone(),
            text("Type"),
            text("text"),
            text("-property"),
            text("FontSize"),
        ])
        .unwrap();
        assert_eq!(handles(&both).len(), 1);
        let err = direct(&[f.clone(), text("Tag"), text("x")]).unwrap_err();
        assert_eq!(identifier(&err), "RunMat:findall:UnsupportedOption");
        let err = direct(&[f.clone(), text("-depth")]).unwrap_err();
        assert_eq!(identifier(&err), "RunMat:findall:UnsupportedOption");
        let err = direct(&[text("a")]).unwrap_err();
        assert_eq!(identifier(&err), "RunMat:findall:InvalidHandles");
        let err = direct(&[Value::Struct(StructValue::new())]).unwrap_err();
        assert_eq!(identifier(&err), "RunMat:findall:InvalidHandles");
        let err = direct(&[]).unwrap_err();
        assert_eq!(identifier(&err), "RunMat:minrhs");
        let ax = call("gca", &[]).unwrap();
        let pair = matrix(&[&[shaped(&f).1[0], shaped(&ax).1[0]]]);
        let found = direct(&[pair, text("Type"), text("axes")]).unwrap();
        assert_eq!(handles(&found).len(), 2, "axes found under both roots");
    }

    /// Dispatcher path.
    #[test]
    fn registered_through_dispatcher() {
        let _guard = setup();
        let f = two_lines();
        let found = call("findall", &[f, text("Type"), text("line")]).expect("findall");
        assert_eq!(handles(&found).len(), 2);
    }
}
