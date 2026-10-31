//! Clean-room `isfloat` (is the value of class double or single?).
//!
//! Clean-room feature `specs/054-types-structs-text/`. Normative behaviour
//! comes only from the approved export `isfloat` 0.2.0 (see that feature's
//! `provenance.md`). The class test uses RunMat's own `class` name of the
//! value (decision D-054-3). Observed `MATLAB:*` identifiers are raised as
//! `RunMat:*` (D-054-1).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::Value;

use crate::{build_runtime_error, BuiltinResult, RuntimeError};

const NAME: &str = "isfloat";

// ---------------------------------------------------------------------------
// Descriptor
// ---------------------------------------------------------------------------

const INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "A",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Any value.",
}];

const OUTPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "tf",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Logical scalar: true when A is double or single.",
}];

const SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "tf = isfloat(A)",
    inputs: &INPUTS,
    outputs: &OUTPUTS,
}];

const MINRHS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ISFLOAT.MINRHS",
    identifier: Some("RunMat:minrhs"),
    when: "No input is given.",
    message: "isfloat: not enough input arguments",
};

const MAXRHS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ISFLOAT.MAXRHS",
    identifier: Some("RunMat:maxrhs"),
    when: "More than one input is given.",
    message: "isfloat: too many input arguments",
};

const TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ISFLOAT.TOO_MANY_OUTPUTS",
    identifier: Some("RunMat:isfloat:TooManyOutputs"),
    when: "More than one output was requested.",
    message: "isfloat: too many output arguments",
};

const ERRORS: [BuiltinErrorDescriptor; 3] = [MINRHS, MAXRHS, TOO_MANY_OUTPUTS];

pub const ISFLOAT_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ERRORS,
};

fn isfloat_error(
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
    name = "isfloat",
    category = "logical/tests",
    summary = "True when the value is of class double or single (real, complex, sparse or empty).",
    keywords = "isfloat,floating point,double,single,class test",
    accel = "cpu",
    descriptor(crate::builtins::logical::tests::isfloat::ISFLOAT_DESCRIPTOR),
    builtin_path = "crate::builtins::logical::tests::isfloat"
)]
pub(crate) async fn isfloat_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    // FR-054-23
    if let Some(count) = crate::output_count::current_output_count() {
        if count > 1 {
            return Err(isfloat_error(
                &TOO_MANY_OUTPUTS,
                format!("isfloat returns one output, {count} were requested"),
            ));
        }
    }
    // FR-054-04
    let value = match args.as_slice() {
        [] => return Err(isfloat_error(&MINRHS, "isfloat requires one input")),
        [value] => value,
        _ => return Err(isfloat_error(&MAXRHS, "isfloat takes one input")),
    };
    // FR-054-01..03, FR-054-24 (D-054-3)
    let class = crate::builtins::introspection::class::class_name_for_value(value);
    Ok(Value::Bool(matches!(class.as_str(), "double" | "single")))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod isfloat_tests {
    use super::isfloat_builtin;
    use crate::BuiltinResult;
    use futures::executor::block_on;
    use runmat_value::{CellArray, CharArray, ComplexTensor, IntValue, StructValue, Tensor, Value};

    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("isfloat", args))
    }

    fn builtin(name: &str, args: &[Value]) -> Value {
        block_on(crate::call_builtin_async(name, args)).unwrap()
    }

    fn isfloat(value: Value) -> Value {
        call(&[value]).unwrap()
    }

    fn identifier(result: BuiltinResult<Value>) -> String {
        match result {
            Ok(value) => panic!("expected an error, got {value:?}"),
            Err(err) => err.identifier().unwrap_or("<none>").to_string(),
        }
    }

    const T: Value = Value::Bool(true);
    const F: Value = Value::Bool(false);

    // --- Observed (normative): one test per observed case --------------------

    /// case `double` — isfloat.signature-primary, isfloat.true-cases
    #[test]
    fn observed_double() {
        assert_eq!(isfloat(Value::Num(1.0)), T);
    }

    /// case `single` — isfloat.true-cases
    #[test]
    fn observed_single() {
        assert_eq!(isfloat(builtin("single", &[Value::Num(1.0)])), T);
    }

    /// case `complex` — isfloat.true-cases
    #[test]
    fn observed_complex() {
        assert_eq!(isfloat(Value::Complex(1.0, 2.0)), T);
    }

    /// case `nan` — isfloat.true-cases
    #[test]
    fn observed_nan() {
        assert_eq!(isfloat(Value::Num(f64::NAN)), T);
    }

    /// case `empty` — isfloat.true-cases
    #[test]
    fn observed_empty() {
        let empty = Value::Tensor(Tensor::new(Vec::new(), vec![0, 0]).unwrap());
        assert_eq!(isfloat(empty), T);
    }

    /// case `sparse` — isfloat.true-cases
    #[test]
    fn observed_sparse() {
        let sparse = builtin("sparse", &[Value::Num(1.0)]);
        assert!(matches!(sparse, Value::SparseTensor(_)));
        assert_eq!(isfloat(sparse), T);
    }

    /// case `matrix` — isfloat.true-cases (scalar result for a 2x3 input)
    #[test]
    fn observed_matrix() {
        let ones = Value::Tensor(Tensor::new(vec![1.0; 6], vec![2, 3]).unwrap());
        assert_eq!(isfloat(ones), T);
    }

    /// case `int8` — isfloat.false-cases
    #[test]
    fn observed_int8() {
        assert_eq!(isfloat(Value::Int(IntValue::I8(1))), F);
    }

    /// case `uint64` — isfloat.false-cases
    #[test]
    fn observed_uint64() {
        assert_eq!(isfloat(Value::Int(IntValue::U64(1))), F);
    }

    /// case `logical` — isfloat.false-cases
    #[test]
    fn observed_logical() {
        assert_eq!(isfloat(Value::Bool(true)), F);
    }

    /// case `char` — isfloat.false-cases
    #[test]
    fn observed_char() {
        assert_eq!(isfloat(Value::CharArray(CharArray::new_row("a"))), F);
    }

    /// case `string` — isfloat.false-cases
    #[test]
    fn observed_string() {
        assert_eq!(isfloat(Value::String("a".into())), F);
    }

    /// case `cell` — isfloat.false-cases (a cell holding a double)
    #[test]
    fn observed_cell() {
        let cell = Value::Cell(CellArray::new(vec![Value::Num(1.0)], 1, 1).unwrap());
        assert_eq!(isfloat(cell), F);
    }

    /// case `struct` — isfloat.false-cases
    #[test]
    fn observed_struct() {
        let mut s = StructValue::new();
        s.fields.insert("a".into(), Value::Num(1.0));
        assert_eq!(isfloat(Value::Struct(s)), F);
    }

    /// case `handle` — isfloat.false-cases
    #[test]
    fn observed_handle() {
        assert_eq!(isfloat(Value::FunctionHandle("sin".into())), F);
    }

    /// case `err-none` — isfloat.error-none
    #[test]
    fn observed_err_none() {
        assert_eq!(identifier(call(&[])), "RunMat:minrhs");
    }

    /// case `err-two` — isfloat.error-two
    #[test]
    fn observed_err_two() {
        assert_eq!(
            identifier(call(&[Value::Num(1.0), Value::Num(2.0)])),
            "RunMat:maxrhs"
        );
    }

    // --- Documented choices ---------------------------------------------------

    /// FR-054-24 (documented choice): other classes follow RunMat's `class`.
    #[test]
    fn documented_choice_other_classes() {
        let single_complex = ComplexTensor::new(vec![(1.0, 2.0)], vec![1, 1]).unwrap();
        let single_complex = builtin("single", &[Value::ComplexTensor(single_complex)]);
        assert_eq!(isfloat(single_complex), T);
        for value in [
            Value::Int(IntValue::I16(1)),
            Value::Int(IntValue::U8(1)),
            Value::Int(IntValue::I64(1)),
            builtin(
                "datetime",
                &[Value::Num(2020.0), Value::Num(1.0), Value::Num(2.0)],
            ),
        ] {
            assert_eq!(isfloat(value), F);
        }
    }

    /// FR-054-23 (documented choice): more than one output.
    #[test]
    fn documented_choice_too_many_outputs() {
        let _guard = crate::output_count::push_output_count(Some(2));
        let err = block_on(isfloat_builtin(vec![Value::Num(1.0)])).unwrap_err();
        assert_eq!(err.identifier(), Some("RunMat:isfloat:TooManyOutputs"));
    }

    /// FR-054-28 (unresolved, isfloat.q-warning-coverage): result only.
    #[test]
    fn unresolved_no_warning() {
        assert_eq!(isfloat(Value::Num(1.0)), T);
    }
}
