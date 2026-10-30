//! Clean-room `cast` (convert to a numeric, logical or char class).
//!
//! Clean-room feature `specs/050-numeric-conversion/`. Normative behaviour
//! comes only from the approved export `cast` 0.2.0 (see that feature's
//! `provenance.md`). Observed `MATLAB:*` identifiers are raised as
//! `RunMat:*` (decision D-050-1).
//!
//! `cast` only validates its arguments and then calls RunMat's registered
//! conversion builtin named by the target class (`double`, `single`,
//! `int8`…`uint64`, `logical`, `char`), so rounding, saturation and NaN/Inf
//! handling are exactly those builtins' (decision D-050-2). Spec 0.2.1:
//! `'like'` a complex double/single `p` lifts the result to complex with
//! RunMat's `complex` builtin (FR-050-31, closing D-050-3).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{NumericDType, Value};

use crate::builtins::common::map_control_flow_with_builtin;
use crate::builtins::introspection::class::class_name_for_value;
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const NAME: &str = "cast";

/// Target classes accepted by `cast` (FR-050-20, FR-050-27).
const SUPPORTED_CLASSES: [&str; 12] = [
    "double", "single", "int8", "int16", "int32", "int64", "uint8", "uint16", "uint32", "uint64",
    "logical", "char",
];

// ---------------------------------------------------------------------------
// Descriptor
// ---------------------------------------------------------------------------

const A_PARAM: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "A",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Numeric, logical or char array.",
};

const CAST_INPUTS: [BuiltinParamDescriptor; 2] = [
    A_PARAM,
    BuiltinParamDescriptor {
        name: "newclass",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Name of a numeric class, 'logical' or 'char'.",
    },
];

const CAST_LIKE_INPUTS: [BuiltinParamDescriptor; 3] = [
    A_PARAM,
    BuiltinParamDescriptor {
        name: "'like'",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "The keyword 'like'.",
    },
    BuiltinParamDescriptor {
        name: "p",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Prototype whose class is used.",
    },
];

const CAST_OUTPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "B",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Array of the target class with the shape of A.",
}];

const CAST_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "B = cast(A, newclass)",
        inputs: &CAST_INPUTS,
        outputs: &CAST_OUTPUTS,
    },
    BuiltinSignatureDescriptor {
        label: "B = cast(A, 'like', p)",
        inputs: &CAST_LIKE_INPUTS,
        outputs: &CAST_OUTPUTS,
    },
];

const UNSUPPORTED_CLASS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.CAST.UNSUPPORTED_CLASS",
    identifier: Some("RunMat:cast:UnsupportedClass"),
    when: "newclass (or the class of p) is not a numeric class, 'logical' or 'char'.",
    message: "cast: unsupported target class",
};

const INVALID_CONVERSION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.CAST.INVALID_CONVERSION",
    identifier: Some("RunMat:invalidConversion"),
    when: "A is a cell array.",
    message: "cast: cannot convert a cell array",
};

const INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.CAST.INVALID_INPUT",
    identifier: Some("RunMat:cast:InvalidInput"),
    when: "The call does not have two or three inputs.",
    message: "cast: expected cast(A, newclass) or cast(A, 'like', p)",
};

const UNSUPPORTED_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.CAST.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:cast:UnsupportedOption"),
    when: "Unobserved forms: non-text newclass, a three-argument form without 'like', or A of a non-numeric/logical/char class.",
    message: "cast: this form is not yet specified",
};

const TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.CAST.TOO_MANY_OUTPUTS",
    identifier: Some("RunMat:cast:TooManyOutputs"),
    when: "More than one output was requested.",
    message: "cast: too many output arguments",
};

const CAST_ERRORS: [BuiltinErrorDescriptor; 5] = [
    UNSUPPORTED_CLASS,
    INVALID_CONVERSION,
    INVALID_INPUT,
    UNSUPPORTED_OPTION,
    TOO_MANY_OUTPUTS,
];

pub const CAST_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &CAST_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &CAST_ERRORS,
};

fn cast_error(
    descriptor: &'static BuiltinErrorDescriptor,
    detail: impl Into<String>,
) -> RuntimeError {
    let mut builder = build_runtime_error(format!("{NAME}: {}", detail.into())).with_builtin(NAME);
    if let Some(identifier) = descriptor.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

// ---------------------------------------------------------------------------
// Builtin
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "cast",
    category = "math/elementwise",
    summary = "Convert values to a numeric, logical or char class.",
    keywords = "cast,convert,class,like,int8,uint8,single,double,logical,char",
    accel = "cpu",
    descriptor(crate::builtins::math::elementwise::integer_conversions::cast::CAST_DESCRIPTOR),
    builtin_path = "crate::builtins::math::elementwise::integer_conversions::cast"
)]
pub(crate) async fn cast_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    // FR-050-21
    if let Some(count) = crate::output_count::current_output_count() {
        if count > 1 {
            return Err(cast_error(
                &TOO_MANY_OUTPUTS,
                format!("cast returns one output, {count} were requested"),
            ));
        }
    }
    if !(2..=3).contains(&args.len()) {
        return Err(cast_error(
            &INVALID_INPUT,
            "expected cast(A, newclass) or cast(A, 'like', p)",
        ));
    }
    // FR-050-27: the target class is checked before A.
    let target = target_class(&args).await?;
    check_source(&args[0])?;
    // D-050-2 / FR-050-29: delegate to the registered conversion builtin.
    // FR-050-32: `double` keeps a complex A complex.
    let mut args = args;
    let a = args.swap_remove(0);
    let converted = crate::call_builtin_async(&target.class, &[a]).await?;
    if target.complex {
        // FR-050-31: lift to complex with RunMat's one-argument `complex`,
        // which keeps a complex value and otherwise adds a zero imaginary
        // part (Value::Complex for a double scalar, ComplexTensor otherwise).
        return crate::call_builtin_async("complex", &[converted]).await;
    }
    Ok(converted)
}

/// Target of a conversion: the class name and, for `'like'` a complex `p`,
/// whether the result must be complex (FR-050-31).
struct Target {
    class: String,
    complex: bool,
}

/// Text of a char row or string scalar.
fn text_of(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::StringArray(sa) if sa.data.len() == 1 => Some(sa.data[0].clone()),
        Value::CharArray(ca) if ca.rows <= 1 => Some(ca.data.iter().collect()),
        _ => None,
    }
}

/// Resolve the target class name (FR-050-15, 18, 20, 27, 28).
async fn target_class(args: &[Value]) -> BuiltinResult<Target> {
    let mut complex = false;
    let name = if args.len() == 2 {
        text_of(&args[1]).ok_or_else(|| {
            cast_error(
                &UNSUPPORTED_OPTION,
                "newclass must be a class name given as text",
            )
        })?
    } else {
        let keyword = text_of(&args[1]).unwrap_or_default();
        if !keyword.eq_ignore_ascii_case("like") {
            return Err(cast_error(
                &UNSUPPORTED_OPTION,
                "the three-argument form must be cast(A, 'like', p)",
            ));
        }
        // FR-050-28: p is gathered first; its class and, for dense
        // double/single p, its complexity are used.
        let prototype = gather_if_needed_async(&args[2])
            .await
            .map_err(|err| map_control_flow_with_builtin(err, NAME))?;
        let name = class_name_for_value(&prototype);
        if SUPPORTED_CLASSES.contains(&name.as_str()) {
            complex = prototype_is_complex(&prototype)?;
        }
        name
    };
    if SUPPORTED_CLASSES.contains(&name.as_str()) {
        Ok(Target {
            class: name,
            complex,
        })
    } else {
        Err(cast_error(
            &UNSUPPORTED_CLASS,
            format!("'{name}' is not a numeric class, 'logical' or 'char'"),
        ))
    }
}

/// Whether a `'like'` prototype is complex (FR-050-31). Complex double and
/// single prototypes are observed; a complex integer-class or complex sparse
/// prototype is not, and raises UnsupportedOption (FR-050-28).
fn prototype_is_complex(prototype: &Value) -> BuiltinResult<bool> {
    match prototype {
        Value::Complex(_, _) => Ok(true),
        Value::ComplexTensor(t) => match t.numeric_dtype() {
            NumericDType::F64 | NumericDType::F32 => Ok(true),
            _ => Err(cast_error(
                &UNSUPPORTED_OPTION,
                "'like' a complex integer-class prototype is not yet specified",
            )),
        },
        Value::SparseTensor(s) if s.is_complex() => Err(cast_error(
            &UNSUPPORTED_OPTION,
            "'like' a complex sparse prototype is not yet specified",
        )),
        _ => Ok(false),
    }
}

/// Accept numeric, logical and char sources (FR-050-20, FR-050-27).
fn check_source(value: &Value) -> BuiltinResult<()> {
    match value {
        Value::Num(_)
        | Value::Int(_)
        | Value::Bool(_)
        | Value::Complex(_, _)
        | Value::Tensor(_)
        | Value::ComplexTensor(_)
        | Value::SparseTensor(_)
        | Value::LogicalArray(_)
        | Value::CharArray(_)
        | Value::GpuTensor(_) => Ok(()),
        Value::Cell(_) => Err(cast_error(
            &INVALID_CONVERSION,
            "conversion from cell is not possible",
        )),
        other => Err(cast_error(
            &UNSUPPORTED_OPTION,
            format!(
                "conversion from {} is not yet specified",
                class_name_for_value(other)
            ),
        )),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod cast_tests {
    use super::cast_builtin;
    use crate::builtins::introspection::class::class_name_for_value;
    use crate::BuiltinResult;
    use futures::executor::block_on;
    use runmat_value::{
        CellArray, CharArray, IntValue, IntegerStorage, LogicalArray, StructValue, Tensor, Value,
    };

    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("cast", args))
    }

    fn n(x: f64) -> Value {
        Value::Num(x)
    }

    fn row(values: &[f64]) -> Value {
        Value::Tensor(Tensor::new(values.to_vec(), vec![1, values.len()]).unwrap())
    }

    fn chars(s: &str) -> Value {
        Value::CharArray(CharArray::new_row(s))
    }

    /// (class, shape, (re, im) pairs) of a complex result; panics on a real
    /// result, so it also checks complexity (`isreal` false).
    fn complex_parts(value: &Value) -> (String, Vec<usize>, Vec<(f64, f64)>) {
        let class = class_name_for_value(value);
        match value {
            Value::Complex(re, im) => (class, vec![1, 1], vec![(*re, *im)]),
            Value::ComplexTensor(t) => (class, t.shape.clone(), t.materialize_f64()),
            other => panic!("expected a complex result, got {other:?}"),
        }
    }

    fn check_complex(
        result: BuiltinResult<Value>,
        class: &str,
        shape: &[usize],
        expected: &[(f64, f64)],
    ) {
        let (got_class, got_shape, got) = complex_parts(&result.unwrap());
        assert_eq!(got_class, class);
        assert_eq!(got_shape, shape);
        assert_eq!(got, expected);
    }

    fn builtin(name: &str, args: &[Value]) -> Value {
        block_on(crate::call_builtin_async(name, args)).unwrap()
    }

    fn shape_of(value: &Value) -> Vec<usize> {
        match value {
            Value::Num(_) | Value::Int(_) | Value::Bool(_) | Value::Complex(_, _) => vec![1, 1],
            Value::Tensor(t) => t.shape.clone(),
            Value::LogicalArray(l) => l.shape.clone(),
            Value::CharArray(c) => vec![c.rows, c.cols],
            other => panic!("unexpected result {other:?}"),
        }
    }

    /// Values of any numeric/logical/char result, read through `double`.
    fn values_of(value: &Value) -> Vec<f64> {
        match block_on(crate::call_builtin_async(
            "double",
            std::slice::from_ref(value),
        ))
        .unwrap()
        {
            Value::Num(v) => vec![v],
            Value::Tensor(t) => t.materialize_f64(),
            other => panic!("unexpected double result {other:?}"),
        }
    }

    /// Check class, shape and values of a result.
    fn check(result: BuiltinResult<Value>, class: &str, shape: &[usize], expected: &[f64]) {
        let value = result.unwrap();
        assert_eq!(class_name_for_value(&value), class, "class of {value:?}");
        assert_eq!(shape_of(&value), shape);
        assert_eq!(values_of(&value), expected);
    }

    fn identifier(result: BuiltinResult<Value>) -> String {
        match result {
            Ok(value) => panic!("expected an error, got {value:?}"),
            Err(err) => err.identifier().unwrap_or("<none>").to_string(),
        }
    }

    const BAD_CLASS: &str = "RunMat:cast:UnsupportedClass";
    const UNSUPPORTED: &str = "RunMat:cast:UnsupportedOption";

    // --- Observed (normative): one test per observed case --------------------

    /// case `round` — cast.signature-primary, cast.rounding
    #[test]
    fn observed_round() {
        check(
            call(&[row(&[1.5, 2.5, -1.5]), chars("int32")]),
            "int32",
            &[1, 3],
            &[2.0, 3.0, -2.0],
        );
    }

    /// case `saturate` — cast.saturation
    #[test]
    fn observed_saturate() {
        check(
            call(&[row(&[-1.0, 300.0, 255.6]), chars("uint8")]),
            "uint8",
            &[1, 3],
            &[0.0, 255.0, 255.0],
        );
    }

    /// case `nan-inf` — cast.nan-inf
    #[test]
    fn observed_nan_inf() {
        check(
            call(&[
                row(&[f64::NAN, f64::INFINITY, f64::NEG_INFINITY]),
                chars("int8"),
            ]),
            "int8",
            &[1, 3],
            &[0.0, 127.0, -128.0],
        );
    }

    /// case `to-single` — cast.other-classes
    #[test]
    fn observed_to_single() {
        check(
            call(&[n(1.0 / 3.0), chars("single")]),
            "single",
            &[1, 1],
            &[0.3333333432674408],
        );
    }

    /// case `char-to-double` — cast.other-classes
    #[test]
    fn observed_char_to_double() {
        check(
            call(&[chars("ab"), chars("double")]),
            "double",
            &[1, 2],
            &[97.0, 98.0],
        );
    }

    /// case `logical-to-double` — cast.other-classes
    #[test]
    fn observed_logical_to_double() {
        let mask = Value::LogicalArray(LogicalArray::new(vec![1, 0], vec![1, 2]).unwrap());
        check(
            call(&[mask, chars("double")]),
            "double",
            &[1, 2],
            &[1.0, 0.0],
        );
    }

    /// case `int-to-int` — cast.saturation
    #[test]
    fn observed_int_to_int() {
        let a = Value::Tensor(
            Tensor::new_integer(IntegerStorage::I16(vec![-200, 5, 200]), vec![1, 3]).unwrap(),
        );
        check(
            call(&[a, chars("int8")]),
            "int8",
            &[1, 3],
            &[-128.0, 5.0, 127.0],
        );
    }

    /// case `to-logical` — cast.other-classes
    #[test]
    fn observed_to_logical() {
        check(
            call(&[row(&[0.0, 2.0]), chars("logical")]),
            "logical",
            &[1, 2],
            &[0.0, 1.0],
        );
    }

    /// case `to-char` — cast.other-classes
    #[test]
    fn observed_to_char() {
        let value = call(&[row(&[72.0, 105.0]), chars("char")]).unwrap();
        let Value::CharArray(ca) = &value else {
            panic!("expected char, got {value:?}");
        };
        assert_eq!((ca.rows, ca.cols), (1, 2));
        assert_eq!(ca.data.iter().collect::<String>(), "Hi");
    }

    /// case `like` — cast.like, cast.saturation
    #[test]
    fn observed_like() {
        check(
            call(&[n(70000.0), chars("like"), Value::Int(IntValue::U16(1))]),
            "uint16",
            &[1, 1],
            &[65535.0],
        );
    }

    /// case `like-complex` — cast.like-complex (0.2.1: complex double 2+0i)
    #[test]
    fn observed_like_complex() {
        check_complex(
            call(&[n(2.0), chars("like"), Value::Complex(0.0, 1.0)]),
            "double",
            &[1, 1],
            &[(2.0, 0.0)],
        );
    }

    /// case `like-complex-parts` — cast.like-complex (class double, isreal
    /// false, real 2, imag 0)
    #[test]
    fn observed_like_complex_parts() {
        let b = call(&[n(2.0), chars("like"), Value::Complex(0.0, 1.0)]).unwrap();
        let (class, _, parts) = complex_parts(&b);
        assert_eq!(class, "double");
        assert_eq!(parts, vec![(2.0, 0.0)]);
        let isreal = builtin("isreal", &[b]);
        assert!(matches!(isreal, Value::Bool(false)), "isreal: {isreal:?}");
    }

    /// case `like-complex-int-src` — cast.like-complex
    #[test]
    fn observed_like_complex_int_src() {
        check_complex(
            call(&[
                Value::Int(IntValue::I8(3)),
                chars("like"),
                Value::Complex(0.0, 1.0),
            ]),
            "double",
            &[1, 1],
            &[(3.0, 0.0)],
        );
    }

    /// case `like-complex-single` — cast.like-complex
    #[test]
    fn observed_like_complex_single() {
        let p = builtin("single", &[Value::Complex(0.0, 1.0)]);
        check_complex(
            call(&[n(2.0), chars("like"), p]),
            "single",
            &[1, 1],
            &[(2.0, 0.0)],
        );
    }

    /// case `like-complex-zero-imag` — cast.like-complex
    #[test]
    fn observed_like_complex_zero_imag() {
        let p = builtin("complex", &[n(1.0), n(0.0)]);
        check_complex(
            call(&[row(&[1.0, 2.0]), chars("like"), p]),
            "double",
            &[1, 2],
            &[(1.0, 0.0), (2.0, 0.0)],
        );
    }

    /// case `like-real-from-complex` — cast.like-real-p
    #[test]
    fn observed_like_real_from_complex() {
        check_complex(
            call(&[Value::Complex(1.0, 2.0), chars("like"), n(1.0)]),
            "double",
            &[1, 1],
            &[(1.0, 2.0)],
        );
    }

    /// case `shape` — cast.shape
    #[test]
    fn observed_shape() {
        let a = Value::Tensor(Tensor::new(vec![1.0, 3.0, 2.0, 4.0], vec![2, 2]).unwrap());
        check(
            call(&[a, chars("uint8")]),
            "uint8",
            &[2, 2],
            &[1.0, 3.0, 2.0, 4.0],
        );
    }

    /// case `same-class` — cast.shape
    #[test]
    fn observed_same_class() {
        check(
            call(&[Value::Int(IntValue::I8(5)), chars("int8")]),
            "int8",
            &[1, 1],
            &[5.0],
        );
    }

    /// case `err-bad-type` — cast.error-bad-type
    #[test]
    fn observed_err_bad_type() {
        assert_eq!(identifier(call(&[n(1.0), chars("notaclass")])), BAD_CLASS);
    }

    /// case `err-cell` — cast.error-cell
    #[test]
    fn observed_err_cell() {
        let cell = Value::Cell(CellArray::new(vec![n(1.0)], 1, 1).unwrap());
        assert_eq!(
            identifier(call(&[cell, chars("double")])),
            "RunMat:invalidConversion"
        );
    }

    /// case `err-struct-type` — cast.error-bad-type
    #[test]
    fn observed_err_struct_type() {
        assert_eq!(identifier(call(&[n(1.0), chars("struct")])), BAD_CLASS);
    }

    // --- Documented choices (FR-050-21, 27..29) ------------------------------

    /// FR-050-21: arity and output count.
    #[test]
    fn documented_choice_arity_and_outputs() {
        assert_eq!(identifier(call(&[n(1.0)])), "RunMat:cast:InvalidInput");
        assert_eq!(
            identifier(call(&[n(1.0), chars("like"), n(1.0), n(2.0)])),
            "RunMat:cast:InvalidInput"
        );
        let _guard = crate::output_count::push_output_count(Some(2));
        let err = block_on(cast_builtin(vec![n(1.0), chars("int8")])).unwrap_err();
        assert_eq!(err.identifier(), Some("RunMat:cast:TooManyOutputs"));
    }

    /// FR-050-27: every supported class name round-trips a small value.
    #[test]
    fn documented_choice_all_supported_classes() {
        for class in [
            "double", "single", "int8", "int16", "int32", "int64", "uint8", "uint16", "uint32",
            "uint64", "logical",
        ] {
            check(call(&[n(1.0), chars(class)]), class, &[1, 1], &[1.0]);
        }
        check(call(&[n(65.0), chars("char")]), "char", &[1, 1], &[65.0]);
    }

    /// FR-050-27: string-scalar class name; case-sensitive class names;
    /// case-insensitive 'like'.
    #[test]
    fn documented_choice_text_forms() {
        check(
            call(&[n(3.7), Value::String("int16".into())]),
            "int16",
            &[1, 1],
            &[4.0],
        );
        assert_eq!(identifier(call(&[n(1.0), chars("INT8")])), BAD_CLASS);
        check(
            call(&[n(3.0), chars("LIKE"), Value::Int(IntValue::I8(0))]),
            "int8",
            &[1, 1],
            &[3.0],
        );
    }

    /// FR-050-27: other class names outside the supported list.
    #[test]
    fn documented_choice_other_class_names() {
        for class in ["cell", "string", "function_handle", ""] {
            assert_eq!(
                identifier(call(&[n(1.0), chars(class)])),
                BAD_CLASS,
                "{class}"
            );
        }
    }

    /// FR-050-27: non-text newclass, a bad keyword, unsupported A.
    #[test]
    fn documented_choice_unsupported_forms() {
        assert_eq!(identifier(call(&[n(1.0), n(8.0)])), UNSUPPORTED);
        assert_eq!(
            identifier(call(&[n(1.0), chars("as"), n(1.0)])),
            UNSUPPORTED
        );
        assert_eq!(
            identifier(call(&[Value::String("7".into()), chars("double")])),
            UNSUPPORTED
        );
        assert_eq!(
            identifier(call(&[Value::Struct(StructValue::new()), chars("double")])),
            UNSUPPORTED
        );
    }

    /// FR-050-27: the class is checked before A.
    #[test]
    fn documented_choice_class_checked_first() {
        let cell = Value::Cell(CellArray::new(vec![n(1.0)], 1, 1).unwrap());
        assert_eq!(identifier(call(&[cell, chars("struct")])), BAD_CLASS);
    }

    /// FR-050-28: 'like' a prototype whose class is not supported.
    #[test]
    fn documented_choice_like_unsupported_prototype() {
        let cell = Value::Cell(CellArray::new(vec![n(1.0)], 1, 1).unwrap());
        assert_eq!(identifier(call(&[n(1.0), chars("like"), cell])), BAD_CLASS);
        check(
            call(&[n(2.0), chars("like"), chars("x")]),
            "char",
            &[1, 1],
            &[2.0],
        );
        let mask = Value::Bool(false);
        check(
            call(&[n(2.0), chars("like"), mask]),
            "logical",
            &[1, 1],
            &[1.0],
        );
    }

    /// FR-050-29: complex A with a named class keeps the delegated
    /// builtin's behaviour (double keeps the complex value).
    #[test]
    fn documented_choice_complex_source_delegated() {
        let value = call(&[Value::Complex(1.0, 2.0), chars("double")]).unwrap();
        assert_eq!(class_name_for_value(&value), "double");
        assert!(matches!(value, Value::Complex(1.0, 2.0)), "{value:?}");
    }

    /// FR-050-28: a complex integer-class p and a complex sparse p are not
    /// observed and raise UnsupportedOption.
    #[test]
    fn documented_choice_like_complex_integer_or_sparse_prototype() {
        let p = builtin(
            "complex",
            &[Value::Int(IntValue::I8(1)), Value::Int(IntValue::I8(1))],
        );
        assert_eq!(class_name_for_value(&p), "int8");
        assert_eq!(identifier(call(&[n(2.0), chars("like"), p])), UNSUPPORTED);
        let p = builtin("sparse", &[Value::Complex(1.0, 1.0)]);
        assert!(
            matches!(&p, Value::SparseTensor(s) if s.is_complex()),
            "{p:?}"
        );
        assert_eq!(identifier(call(&[n(2.0), chars("like"), p])), UNSUPPORTED);
    }

    /// FR-050-31: a complex p with a complex A keeps A's imaginary part, and
    /// an array A 'like' a complex single p gives a complex single array.
    #[test]
    fn documented_choice_like_complex_more_shapes() {
        check_complex(
            call(&[
                Value::Complex(1.0, 2.0),
                chars("like"),
                Value::Complex(0.0, 1.0),
            ]),
            "double",
            &[1, 1],
            &[(1.0, 2.0)],
        );
        let p = builtin("single", &[Value::Complex(0.0, 1.0)]);
        check_complex(
            call(&[row(&[1.0, 2.0]), chars("like"), p]),
            "single",
            &[1, 2],
            &[(1.0, 0.0), (2.0, 0.0)],
        );
    }

    /// FR-050-30 (unresolved, cast.q-warning-coverage): no warning.
    #[test]
    fn unresolved_no_warning() {
        assert!(call(&[n(1.0), chars("int8")]).is_ok());
    }
}
