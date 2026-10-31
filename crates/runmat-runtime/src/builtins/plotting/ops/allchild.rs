//! `allchild` builtin: children of graphics objects (spec 058).
//!
//! Normative behaviour comes from the approved `allchild` 0.2.0 export
//! (claims `allchild.root`, `allchild.figure`, `allchild.axes`,
//! `allchild.leaf`, `allchild.array`, `allchild.error-deleted`). The object
//! tree is the one `findall` searches (`findall::graphics_children`). RunMat
//! has no annotation-pane object (deviation D-058-4). See
//! `specs/058-graphics-n3n4/`.

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{CellArray, Value};

use super::findall::{graphics_children, handle_column, resolve_live};
use crate::{build_runtime_error, BuiltinResult, RuntimeError};

const BUILTIN_NAME: &str = "allchild";

const ALLCHILD_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "obj",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Graphics handle or array of handles; 0 for the root.",
}];
const ALLCHILD_OUTPUT_ONE: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "h",
    ty: BuiltinParamType::NumericArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Column of child handles (0x0 when there are none).",
}];
const ALLCHILD_OUTPUT_MANY: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "c",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Cell column with one child column per input handle.",
}];

const ALLCHILD_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "h = allchild(obj)",
        inputs: &ALLCHILD_INPUTS,
        outputs: &ALLCHILD_OUTPUT_ONE,
    },
    BuiltinSignatureDescriptor {
        label: "c = allchild([obj1 obj2])",
        inputs: &ALLCHILD_INPUTS,
        outputs: &ALLCHILD_OUTPUT_MANY,
    },
];

const ALLCHILD_ERROR_INVALID_HANDLES: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ALLCHILD.INVALID_HANDLES",
    identifier: Some("RunMat:allchild:InvalidHandles"),
    when: "An input is a deleted handle, a number that is not a handle, or a non-numeric value.",
    message: "allchild: invalid handles",
};
const ALLCHILD_ERROR_MINRHS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ALLCHILD.MINRHS",
    identifier: Some("RunMat:minrhs"),
    when: "No input argument is given.",
    message: "allchild: not enough input arguments",
};
const ALLCHILD_ERROR_UNSUPPORTED_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ALLCHILD.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:allchild:UnsupportedOption"),
    when: "More than one input argument is given (not covered by the approved specification).",
    message: "allchild: unsupported option",
};
const ALLCHILD_ERRORS: [BuiltinErrorDescriptor; 3] = [
    ALLCHILD_ERROR_INVALID_HANDLES,
    ALLCHILD_ERROR_MINRHS,
    ALLCHILD_ERROR_UNSUPPORTED_OPTION,
];

pub const ALLCHILD_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &ALLCHILD_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ALLCHILD_ERRORS,
};

#[runtime_builtin(
    name = "allchild",
    category = "plotting",
    summary = "Children of graphics objects.",
    keywords = "allchild,children,graphics,handle,plotting",
    suppress_auto_output = true,
    descriptor(crate::builtins::plotting::ops::allchild::ALLCHILD_DESCRIPTOR),
    builtin_path = "crate::builtins::plotting::ops::allchild"
)]
pub fn allchild_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    let obj = match args.len() {
        0 => {
            return Err(allchild_error(
                &ALLCHILD_ERROR_MINRHS,
                "a graphics handle is required",
            ))
        }
        1 => &args[0],
        _ => {
            return Err(allchild_error(
                &ALLCHILD_ERROR_UNSUPPORTED_OPTION,
                "only one input argument is supported",
            ))
        }
    };
    let handles: Vec<f64> = match obj {
        Value::Num(v) => vec![*v],
        Value::Int(i) => vec![i.to_f64()],
        Value::Tensor(t) => t.materialize_f64().to_vec(),
        _ => {
            return Err(allchild_error(
                &ALLCHILD_ERROR_INVALID_HANDLES,
                "the input must be a graphics handle",
            ))
        }
    };
    let mut results = Vec::with_capacity(handles.len());
    for handle in &handles {
        let resolved = resolve_live(*handle).ok_or_else(|| {
            allchild_error(
                &ALLCHILD_ERROR_INVALID_HANDLES,
                format!("{handle} is not a valid graphics handle"),
            )
        })?;
        results.push(handle_column(graphics_children(&resolved)));
    }
    match results.len() {
        0 => Ok(handle_column(Vec::new())),
        1 => Ok(results.pop().expect("one result")),
        n => Ok(Value::Cell(CellArray::new(results, n, 1).map_err(|e| {
            allchild_error(&ALLCHILD_ERROR_INVALID_HANDLES, e)
        })?)),
    }
}

fn allchild_error(error: &'static BuiltinErrorDescriptor, detail: impl AsRef<str>) -> RuntimeError {
    let mut builder = build_runtime_error(format!("{}: {}", error.message, detail.as_ref()))
        .with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

#[cfg(test)]
mod allchild_tests {
    use super::allchild_builtin;
    use crate::builtins::plotting::ops::pcolor::pcolor_tests::{
        call, identifier, matrix, prop, setup, shaped, string, text,
    };
    use crate::BuiltinResult;
    use runmat_value::Value;

    fn direct(args: &[Value]) -> BuiltinResult<Value> {
        allchild_builtin(args.to_vec())
    }

    fn types(found: &Value) -> Vec<String> {
        let mut out: Vec<String> = shaped(found)
            .1
            .into_iter()
            .map(|h| string(prop(h, "Type")))
            .collect();
        out.sort();
        out
    }

    // ---- observed (normative) ----

    /// [allchild.root] case `root-has-figure`.
    #[test]
    fn observed_root_lists_new_invisible_figure() {
        let _guard = setup();
        let f = call("figure", &[text("Visible"), text("off")]).expect("figure");
        let found = direct(&[Value::Num(0.0)]).unwrap();
        let (shape, hs) = shaped(&found);
        assert_eq!(shape, vec![hs.len(), 1], "column");
        assert!(hs.contains(&shaped(&f).1[0]));
        assert_eq!(string(prop(shaped(&f).1[0], "Type")), "figure");
    }

    /// [allchild.figure] case `figure`: the axes is a child. The observed
    /// annotation pane does not exist in RunMat (deviation D-058-4).
    #[test]
    fn observed_figure_children_include_axes() {
        let _guard = setup();
        call("plot", &[matrix(&[&[1.0, 2.0, 3.0]])]).expect("plot");
        let f = call("gcf", &[]).unwrap();
        let found = direct(&[f]).unwrap();
        assert_eq!(types(&found), vec!["axes"]);
    }

    /// [allchild.axes] case `axes-vs-children`.
    #[test]
    fn observed_axes_children_are_plot_objects() {
        let _guard = setup();
        call("plot", &[matrix(&[&[1.0, 2.0, 3.0]])]).expect("plot");
        call("title", &[text("t")]).expect("title");
        let ax = call("gca", &[]).unwrap();
        let found = direct(&[ax]).unwrap();
        assert_eq!(types(&found), vec!["line"]);
    }

    /// [allchild.leaf] case `leaf` (0x0 representation: D-058-3).
    #[test]
    fn observed_leaf_has_no_children() {
        let _guard = setup();
        let ln = call("plot", &[matrix(&[&[1.0, 2.0, 3.0]])]).expect("plot");
        assert_eq!(shaped(&direct(&[ln]).unwrap()).0, vec![0, 0]);
    }

    /// [allchild.array] case `array`.
    #[test]
    fn observed_array_input_gives_cell_column() {
        let _guard = setup();
        let f1 = shaped(&call("figure", &[]).unwrap()).1[0];
        let f2 = shaped(&call("figure", &[]).unwrap()).1[0];
        match direct(&[matrix(&[&[f1, f2]])]).unwrap() {
            Value::Cell(cell) => {
                assert_eq!((cell.rows, cell.cols), (2, 1));
            }
            other => panic!("expected a cell, got {other:?}"),
        }
    }

    /// [allchild.error-deleted] cases `err-deleted`, `err-not-handle`,
    /// `err-char`.
    #[test]
    fn observed_invalid_handles_error() {
        let _guard = setup();
        let f = call("figure", &[]).expect("figure");
        call("close", std::slice::from_ref(&f)).expect("close");
        for bad in [f, Value::Num(123456.0), text("a")] {
            let err = direct(&[bad]).unwrap_err();
            assert_eq!(identifier(&err), "RunMat:allchild:InvalidHandles");
        }
    }

    // ---- documented choices ----

    /// (choice) No input raises minrhs; extra inputs are unsupported; an
    /// empty input gives a 0x0 result.
    #[test]
    fn unresolved_choice_argument_handling() {
        let _guard = setup();
        assert_eq!(identifier(&direct(&[]).unwrap_err()), "RunMat:minrhs");
        let err = direct(&[Value::Num(0.0), Value::Num(0.0)]).unwrap_err();
        assert_eq!(identifier(&err), "RunMat:allchild:UnsupportedOption");
        let empty = Value::Tensor(runmat_value::Tensor::new(Vec::new(), vec![0, 0]).unwrap());
        assert_eq!(shaped(&direct(&[empty]).unwrap()).0, vec![0, 0]);
    }

    /// Dispatcher path.
    #[test]
    fn registered_through_dispatcher() {
        let _guard = setup();
        let found = call("allchild", &[Value::Num(0.0)]).expect("allchild");
        assert!(!shaped(&found).1.is_empty());
    }
}
