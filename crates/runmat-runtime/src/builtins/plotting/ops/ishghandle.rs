//! `ishghandle` builtin: elementwise test for valid graphics handles (spec 058).
//!
//! Normative behaviour comes from the approved `ishghandle` 0.2.0 export
//! (claims `ishghandle.live`, `ishghandle.type`, `ishghandle.false-cases`,
//! `ishghandle.empty`, `ishghandle.error-none`). Numeric and handle inputs
//! are tested with RunMat's graphics-handle predicate (`isgraphics`);
//! non-numeric inputs are never handles. See `specs/058-graphics-n3n4/`.

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{LogicalArray, Value};

use crate::builtins::plotting::type_resolvers::handle_logical_type;
use crate::{build_runtime_error, BuiltinResult, RuntimeError};

const BUILTIN_NAME: &str = "ishghandle";

const ISHGHANDLE_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "tf",
    ty: BuiltinParamType::LogicalArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Logical array the size of h: true where h is a valid graphics handle.",
}];

const ISHGHANDLE_INPUT_H: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "h",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Any value.",
};

const ISHGHANDLE_INPUTS_H: [BuiltinParamDescriptor; 1] = [ISHGHANDLE_INPUT_H];

const ISHGHANDLE_INPUTS_H_TYPE: [BuiltinParamDescriptor; 2] = [
    ISHGHANDLE_INPUT_H,
    BuiltinParamDescriptor {
        name: "type",
        ty: BuiltinParamType::StringScalar,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Required graphics object Type (for example 'figure' or 'axes').",
    },
];

const ISHGHANDLE_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "tf = ishghandle(h)",
        inputs: &ISHGHANDLE_INPUTS_H,
        outputs: &ISHGHANDLE_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "tf = ishghandle(h, type)",
        inputs: &ISHGHANDLE_INPUTS_H_TYPE,
        outputs: &ISHGHANDLE_OUTPUT,
    },
];

const ISHGHANDLE_ERROR_MINRHS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ISHGHANDLE.MINRHS",
    identifier: Some("RunMat:minrhs"),
    when: "No input argument is given.",
    message: "ishghandle: not enough input arguments",
};
const ISHGHANDLE_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ISHGHANDLE.INVALID_INPUT",
    identifier: Some("RunMat:ishghandle:InvalidInput"),
    when: "The type argument is not a text scalar.",
    message: "ishghandle: invalid input",
};
const ISHGHANDLE_ERROR_UNSUPPORTED_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ISHGHANDLE.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:ishghandle:UnsupportedOption"),
    when: "More than two input arguments are given (not covered by the approved specification).",
    message: "ishghandle: unsupported option",
};
const ISHGHANDLE_ERRORS: [BuiltinErrorDescriptor; 3] = [
    ISHGHANDLE_ERROR_MINRHS,
    ISHGHANDLE_ERROR_INVALID_INPUT,
    ISHGHANDLE_ERROR_UNSUPPORTED_OPTION,
];

pub const ISHGHANDLE_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &ISHGHANDLE_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ISHGHANDLE_ERRORS,
};

#[runtime_builtin(
    name = "ishghandle",
    category = "plotting",
    summary = "True for valid graphics handles, optionally of a given type.",
    keywords = "ishghandle,graphics,handle,isgraphics,plotting",
    suppress_auto_output = true,
    type_resolver(handle_logical_type),
    descriptor(crate::builtins::plotting::ops::ishghandle::ISHGHANDLE_DESCRIPTOR),
    builtin_path = "crate::builtins::plotting::ops::ishghandle"
)]
pub fn ishghandle_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    match args.len() {
        0 => Err(ishghandle_error(
            &ISHGHANDLE_ERROR_MINRHS,
            "a value to test is required",
        )),
        1 | 2 => {
            if let Some(kind) = args.get(1) {
                if text_scalar(kind).is_none() {
                    return Err(ishghandle_error(
                        &ISHGHANDLE_ERROR_INVALID_INPUT,
                        "type must be a character row or string scalar",
                    ));
                }
            }
            if !is_handle_candidate(&args[0]) {
                return Ok(false_like(&args[0]));
            }
            match &args[0] {
                Value::Num(v) => Ok(Value::Bool(element_is_handle(*v, args.get(1))?)),
                Value::Tensor(t) if t.integer_storage().is_none() => {
                    let data = t
                        .materialize_f64()
                        .iter()
                        .map(|&v| element_is_handle(v, args.get(1)).map(u8::from))
                        .collect::<BuiltinResult<Vec<u8>>>()?;
                    Ok(Value::LogicalArray(
                        LogicalArray::new(data, t.shape.clone()).expect("logical shape"),
                    ))
                }
                _ => crate::builtins::plotting::isgraphics::isgraphics_builtin(args),
            }
        }
        _ => Err(ishghandle_error(
            &ISHGHANDLE_ERROR_UNSUPPORTED_OPTION,
            "at most two input arguments are supported",
        )),
    }
}

/// One double element: RunMat handles are non-negative integers, so any
/// other number is false; integers are judged by `isgraphics`.
fn element_is_handle(value: f64, kind: Option<&Value>) -> BuiltinResult<bool> {
    if !value.is_finite() || value < 0.0 || value.fract() != 0.0 {
        return Ok(false);
    }
    let mut args = vec![Value::Num(value)];
    args.extend(kind.cloned());
    match crate::builtins::plotting::isgraphics::isgraphics_builtin(args)? {
        Value::Bool(b) => Ok(b),
        Value::LogicalArray(la) => Ok(la.data.first().is_some_and(|&b| b != 0)),
        _ => Ok(false),
    }
}

/// Values that can name a RunMat graphics handle: numeric scalars and arrays
/// (RunMat graphics handles are numbers). Every other class is never a
/// handle.
fn is_handle_candidate(value: &Value) -> bool {
    matches!(
        value,
        Value::Num(_) | Value::Int(_) | Value::Tensor(_) | Value::GpuTensor(_)
    )
}

/// A false result shaped like the input: a scalar false for scalar-like
/// values, otherwise a false logical array of the input's size.
fn false_like(value: &Value) -> Value {
    let shape = match value {
        Value::CharArray(chars) => vec![chars.rows, chars.cols],
        Value::LogicalArray(logical) => logical.shape.clone(),
        Value::StringArray(strings) => strings.shape.clone(),
        Value::Cell(cell) => vec![cell.rows, cell.cols],
        _ => vec![1, 1],
    };
    let len = shape.iter().product::<usize>();
    if shape == [1, 1] {
        Value::Bool(false)
    } else {
        Value::LogicalArray(LogicalArray::new(vec![0; len], shape).expect("logical shape"))
    }
}

fn text_scalar(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::StringArray(sa) if sa.data.len() == 1 => Some(sa.data[0].clone()),
        Value::CharArray(ca) if ca.rows == 1 => Some(ca.data.iter().collect()),
        _ => None,
    }
}

fn ishghandle_error(
    error: &'static BuiltinErrorDescriptor,
    detail: impl AsRef<str>,
) -> RuntimeError {
    let mut builder = build_runtime_error(format!("{}: {}", error.message, detail.as_ref()))
        .with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

#[cfg(test)]
mod ishghandle_tests {
    use super::ishghandle_builtin;
    use crate::builtins::plotting::state::reset_hold_state_for_run;
    use crate::builtins::plotting::tests::{ensure_plot_test_env, lock_plot_registry};
    use crate::BuiltinResult;
    use futures::executor::block_on;
    use runmat_value::{CharArray, StructValue, Tensor, Value};

    fn setup() -> crate::builtins::plotting::state::PlotTestLockGuard {
        let guard = lock_plot_registry();
        ensure_plot_test_env();
        reset_hold_state_for_run();
        guard
    }

    fn call(name: &str, args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async(name, args))
    }

    fn text(s: &str) -> Value {
        Value::CharArray(CharArray::new_row(s))
    }

    fn direct(args: &[Value]) -> BuiltinResult<Value> {
        ishghandle_builtin(args.to_vec())
    }

    fn row(values: &[f64]) -> Value {
        Value::Tensor(Tensor::new(values.to_vec(), vec![1, values.len()]).expect("row"))
    }

    /// Logical result as (shape, data).
    fn logical(value: Value) -> (Vec<usize>, Vec<u8>) {
        match value {
            Value::Bool(b) => (vec![1, 1], vec![u8::from(b)]),
            Value::LogicalArray(la) => (la.shape.clone(), la.data.to_vec()),
            other => panic!("expected logical, got {other:?}"),
        }
    }

    fn scalar(value: Value) -> bool {
        let (shape, data) = logical(value);
        assert_eq!(shape, vec![1, 1]);
        data[0] != 0
    }

    /// A live figure with one axes holding one line: (f, ax, ln).
    fn figure_axes_line() -> (Value, Value, Value) {
        let f = call("figure", &[]).expect("figure");
        let ln = call("plot", &[row(&[1.0, 2.0, 3.0])]).expect("plot");
        let ax = call("gca", &[]).expect("gca");
        (f, ax, ln)
    }

    fn num(value: &Value) -> f64 {
        match value {
            Value::Num(n) => *n,
            other => panic!("expected numeric handle, got {other:?}"),
        }
    }

    // ---- observed (normative) ----

    /// [ishghandle.live] cases `figure`, `line`, `array`, `double-of-figure`,
    /// `root`, `groot`.
    #[test]
    fn observed_live_handles_are_true() {
        let _guard = setup();
        let (f, ax, ln) = figure_axes_line();
        assert!(scalar(direct(std::slice::from_ref(&f)).unwrap()));
        assert!(scalar(direct(std::slice::from_ref(&ax)).unwrap()));
        assert!(scalar(direct(std::slice::from_ref(&ln)).unwrap()));
        let (shape, data) = logical(direct(&[row(&[num(&f), num(&ax), num(&ln)])]).unwrap());
        assert_eq!(shape, vec![1, 3]);
        assert_eq!(data, vec![1, 1, 1]);
        let as_double = call("double", std::slice::from_ref(&f)).expect("double");
        assert!(scalar(direct(&[as_double]).unwrap()));
        assert!(scalar(direct(&[Value::Num(0.0)]).unwrap()));
        let root = call("groot", &[]).expect("groot");
        assert!(scalar(direct(&[root]).unwrap()));
    }

    /// [ishghandle.type] cases `figure-type`, `axes-type`, `axes-as-figure`.
    #[test]
    fn observed_type_argument() {
        let _guard = setup();
        let (f, ax, _ln) = figure_axes_line();
        assert!(scalar(direct(&[f, text("figure")]).unwrap()));
        assert!(scalar(direct(&[ax.clone(), text("axes")]).unwrap()));
        assert!(!scalar(direct(&[ax, text("figure")]).unwrap()));
    }

    /// [ishghandle.false-cases] cases `deleted`, `number`, `char`, `struct`,
    /// `graph`, `matrix-number`.
    #[test]
    fn observed_false_cases() {
        let _guard = setup();
        let f = call("figure", &[]).expect("figure");
        call("close", std::slice::from_ref(&f)).expect("close");
        assert!(!scalar(direct(&[f]).unwrap()));
        assert!(!scalar(direct(&[Value::Num(123456.0)]).unwrap()));
        assert!(!scalar(direct(&[text("a")]).unwrap()));
        let mut st = StructValue::new();
        st.insert("a", Value::Num(1.0));
        assert!(!scalar(direct(&[Value::Struct(st)]).unwrap()));
        let g = call("graph", &[Value::Num(1.0), Value::Num(2.0)]).expect("graph");
        assert!(!scalar(direct(&[g]).unwrap()));
        let m = Value::Tensor(Tensor::new(vec![1.5, 3.5, 2.5, 4.5], vec![2, 2]).unwrap());
        let (shape, data) = logical(direct(&[m]).unwrap());
        assert_eq!(shape, vec![2, 2]);
        assert_eq!(data, vec![0, 0, 0, 0]);
    }

    /// [ishghandle.empty] case `empty`.
    #[test]
    fn observed_empty_gives_0x0_logical() {
        let _guard = setup();
        let empty = Value::Tensor(Tensor::new(Vec::new(), vec![0, 0]).unwrap());
        let out = direct(&[empty]).unwrap();
        match out {
            Value::LogicalArray(la) => assert_eq!(la.shape, vec![0, 0]),
            other => panic!("expected 0x0 logical, got {other:?}"),
        }
    }

    /// [ishghandle.error-none] case `err-none`.
    #[test]
    fn observed_no_input_errors_minrhs() {
        let err = direct(&[]).unwrap_err();
        assert_eq!(err.identifier(), Some("RunMat:minrhs"));
    }

    // ---- documented choices for unobserved forms ----

    /// (choice) A char row tests false elementwise; a non-text type errors;
    /// more than two arguments are unsupported.
    #[test]
    fn unresolved_choice_text_and_argument_handling() {
        let _guard = setup();
        let (shape, data) = logical(direct(&[text("abc")]).unwrap());
        assert_eq!(shape, vec![1, 3]);
        assert_eq!(data, vec![0, 0, 0]);
        let err = direct(&[Value::Num(0.0), Value::Num(1.0)]).unwrap_err();
        assert_eq!(err.identifier(), Some("RunMat:ishghandle:InvalidInput"));
        let err = direct(&[Value::Num(0.0), text("root"), text("x")]).unwrap_err();
        assert_eq!(
            err.identifier(),
            Some("RunMat:ishghandle:UnsupportedOption")
        );
    }

    /// Dispatcher path: the builtin is registered and callable by name.
    #[test]
    fn registered_through_dispatcher() {
        let _guard = setup();
        assert!(scalar(call("ishghandle", &[Value::Num(0.0)]).unwrap()));
    }
}
