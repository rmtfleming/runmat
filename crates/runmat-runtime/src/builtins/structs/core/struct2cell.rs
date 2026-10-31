//! Clean-room `struct2cell` (struct array to cell array, fields first).
//!
//! Clean-room feature `specs/054-types-structs-text/`. Normative behaviour
//! comes only from the approved export `struct2cell` 0.2.0 (see that
//! feature's `provenance.md`). A non-struct input raises
//! `RunMat:struct2cell:UndefinedFunction` (D-054-2, documented choice).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{CellArray, Value};

use crate::{build_runtime_error, BuiltinResult, RuntimeError};

const NAME: &str = "struct2cell";

// ---------------------------------------------------------------------------
// Descriptor
// ---------------------------------------------------------------------------

const INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "S",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Scalar struct or struct array.",
}];

const OUTPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "C",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Cell array of size [numfields, size(S)] (trailing singletons dropped).",
}];

const SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "C = struct2cell(S)",
    inputs: &INPUTS,
    outputs: &OUTPUTS,
}];

const UNDEFINED_FUNCTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.STRUCT2CELL.UNDEFINED_FUNCTION",
    identifier: Some("RunMat:struct2cell:UndefinedFunction"),
    when: "The input is not a struct.",
    message: "struct2cell: input must be a struct",
};

const INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.STRUCT2CELL.INVALID_INPUT",
    identifier: Some("RunMat:struct2cell:InvalidInput"),
    when: "The call does not have exactly one input.",
    message: "struct2cell: expected struct2cell(S)",
};

const TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.STRUCT2CELL.TOO_MANY_OUTPUTS",
    identifier: Some("RunMat:struct2cell:TooManyOutputs"),
    when: "More than one output was requested.",
    message: "struct2cell: too many output arguments",
};

const INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.STRUCT2CELL.INTERNAL",
    identifier: Some("RunMat:struct2cell:Internal"),
    when: "The result cell cannot be built.",
    message: "struct2cell: internal runtime failure",
};

const ERRORS: [BuiltinErrorDescriptor; 4] = [
    UNDEFINED_FUNCTION,
    INVALID_INPUT,
    TOO_MANY_OUTPUTS,
    INTERNAL,
];

pub const STRUCT2CELL_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ERRORS,
};

fn struct2cell_error(
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
    name = "struct2cell",
    category = "structs/core",
    summary = "Convert a struct array to a cell array whose first dimension runs over the fields.",
    keywords = "struct2cell,struct,cell,fields,convert",
    accel = "cpu",
    descriptor(crate::builtins::structs::core::struct2cell::STRUCT2CELL_DESCRIPTOR),
    builtin_path = "crate::builtins::structs::core::struct2cell"
)]
pub(crate) async fn struct2cell_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    // FR-054-23
    if let Some(count) = crate::output_count::current_output_count() {
        if count > 1 {
            return Err(struct2cell_error(
                &TOO_MANY_OUTPUTS,
                format!("struct2cell returns one output, {count} were requested"),
            ));
        }
    }
    let [value] = args.as_slice() else {
        return Err(struct2cell_error(
            &INVALID_INPUT,
            "struct2cell takes exactly one input",
        ));
    };
    // (field columns, struct shape); each column is column-major over S.
    let (columns, shape): (Vec<Vec<Value>>, Vec<usize>) = match value {
        // FR-054-09, FR-054-12
        Value::Struct(s) => (
            s.fields.values().map(|v| vec![v.clone()]).collect(),
            vec![1, 1],
        ),
        // FR-054-10, FR-054-11, FR-054-26
        Value::StructArray(array) => {
            let mut columns = Vec::new();
            for name in array.field_names() {
                let values = array.field_values(name).ok_or_else(|| {
                    struct2cell_error(&INTERNAL, format!("field '{name}' has no values"))
                })?;
                columns.push(values.to_vec());
            }
            (columns, array.shape().to_vec())
        }
        // FR-054-26 (D-054-2)
        _ => {
            return Err(struct2cell_error(
                &UNDEFINED_FUNCTION,
                "struct2cell is defined for structs only",
            ))
        }
    };
    let fields = columns.len();
    let elements: usize = shape.iter().product();
    // FR-054-11: fields vary fastest.
    let mut data = Vec::with_capacity(fields * elements);
    for element in 0..elements {
        for column in &columns {
            data.push(column[element].clone());
        }
    }
    let out_shape = result_shape(fields, &shape);
    CellArray::from_column_major(data, out_shape)
        .map(Value::Cell)
        .map_err(|err| struct2cell_error(&INTERNAL, err))
}

/// `[numfields, size(S)]` with trailing singleton dimensions dropped while
/// more than two dimensions remain (FR-054-10).
fn result_shape(fields: usize, shape: &[usize]) -> Vec<usize> {
    let mut out = Vec::with_capacity(shape.len() + 1);
    out.push(fields);
    out.extend_from_slice(shape);
    while out.len() > 2 && out.last() == Some(&1) {
        out.pop();
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod struct2cell_tests {
    use super::{result_shape, struct2cell_builtin};
    use crate::BuiltinResult;
    use futures::executor::block_on;
    use runmat_value::{CellArray, CharArray, StructValue, Tensor, Value};

    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("struct2cell", args))
    }

    fn builtin(name: &str, args: &[Value]) -> Value {
        block_on(crate::call_builtin_async(name, args)).unwrap()
    }

    fn chars(text: &str) -> Value {
        Value::CharArray(CharArray::new_row(text))
    }

    /// A 1-by-n cell (for struct('name', {...}) arguments).
    fn row_cell(values: Vec<Value>) -> Value {
        let n = values.len();
        Value::Cell(CellArray::new(values, 1, n).unwrap())
    }

    fn col_cell(values: Vec<Value>) -> Value {
        let n = values.len();
        Value::Cell(CellArray::new(values, n, 1).unwrap())
    }

    /// (shape, column-major values) of a cell result.
    fn cell(value: Value) -> (Vec<usize>, Vec<Value>) {
        match value {
            Value::Cell(c) => (c.shape.clone(), c.to_column_major()),
            other => panic!("expected a cell, got {other:?}"),
        }
    }

    fn identifier(result: BuiltinResult<Value>) -> String {
        match result {
            Ok(value) => panic!("expected an error, got {value:?}"),
            Err(err) => err.identifier().unwrap_or("<none>").to_string(),
        }
    }

    // --- Observed (normative): one test per observed case --------------------

    /// case `scalar` — struct2cell.signature-primary, struct2cell.scalar
    #[test]
    fn observed_scalar() {
        let inner = row_cell(vec![Value::Num(2.0)]);
        // struct('b', 1, 'a', 'x', 'c', {{2}})
        let s = builtin(
            "struct",
            &[
                chars("b"),
                Value::Num(1.0),
                chars("a"),
                chars("x"),
                chars("c"),
                row_cell(vec![inner.clone()]),
            ],
        );
        let (shape, values) = cell(call(&[s]).unwrap());
        assert_eq!(shape, vec![3, 1]);
        assert_eq!(values, vec![Value::Num(1.0), chars("x"), inner]);
    }

    /// case `array-size` — struct2cell.array-size, struct2cell.array-order
    #[test]
    fn observed_array_size() {
        // struct('a', {1, 2}, 'b', {3, 4})
        let s = builtin(
            "struct",
            &[
                chars("a"),
                row_cell(vec![Value::Num(1.0), Value::Num(2.0)]),
                chars("b"),
                row_cell(vec![Value::Num(3.0), Value::Num(4.0)]),
            ],
        );
        let (shape, values) = cell(call(&[s]).unwrap());
        assert_eq!(shape, vec![2, 1, 2]);
        assert_eq!(
            values,
            vec![
                Value::Num(1.0),
                Value::Num(3.0),
                Value::Num(2.0),
                Value::Num(4.0)
            ]
        );
    }

    /// case `column-array-size` — struct2cell.array-size
    #[test]
    fn observed_column_array_size() {
        // struct('a', {1; 2})
        let s = builtin(
            "struct",
            &[chars("a"), col_cell(vec![Value::Num(1.0), Value::Num(2.0)])],
        );
        let (shape, values) = cell(call(&[s]).unwrap());
        assert_eq!(shape, vec![1, 2]);
        assert_eq!(values, vec![Value::Num(1.0), Value::Num(2.0)]);
    }

    /// case `matrix-array-size` — struct2cell.array-size
    #[test]
    fn observed_matrix_array_size() {
        // repmat(struct('a', 5), 2, 2)
        let one = builtin("struct", &[chars("a"), Value::Num(5.0)]);
        let s = builtin("repmat", &[one, Value::Num(2.0), Value::Num(2.0)]);
        let (shape, values) = cell(call(&[s]).unwrap());
        assert_eq!(shape, vec![1, 2, 2]);
        assert_eq!(values, vec![Value::Num(5.0); 4]);
    }

    /// case `empty-array` — struct2cell.array-size
    #[test]
    fn observed_empty_array() {
        // struct('a', {}, 'b', {})
        let empty = Value::Cell(CellArray::new(Vec::new(), 0, 0).unwrap());
        let s = builtin("struct", &[chars("a"), empty.clone(), chars("b"), empty]);
        let (shape, values) = cell(call(&[s]).unwrap());
        assert_eq!(shape, vec![2, 0, 0]);
        assert!(values.is_empty());
    }

    /// case `no-fields` — struct2cell.no-fields
    #[test]
    fn observed_no_fields() {
        let s = builtin("struct", &[]);
        let (shape, values) = cell(call(&[s]).unwrap());
        assert_eq!(shape, vec![0, 1]);
        assert!(values.is_empty());
    }

    /// case `nested` — struct2cell.scalar (a nested struct is kept whole)
    #[test]
    fn observed_nested() {
        let inner = builtin("struct", &[chars("x"), Value::Num(1.0)]);
        let s = builtin("struct", &[chars("s"), inner.clone()]);
        let (shape, values) = cell(call(&[s]).unwrap());
        assert_eq!(shape, vec![1, 1]);
        assert_eq!(values, vec![inner]);
    }

    /// case `matrix-value` — struct2cell.scalar (a matrix is kept whole)
    #[test]
    fn observed_matrix_value() {
        // [1 2; 3 4], column-major
        let m = Value::Tensor(Tensor::new(vec![1.0, 3.0, 2.0, 4.0], vec![2, 2]).unwrap());
        let mut fields = StructValue::new();
        fields.fields.insert("m".into(), m.clone());
        let (shape, values) = cell(call(&[Value::Struct(fields)]).unwrap());
        assert_eq!(shape, vec![1, 1]);
        assert_eq!(values, vec![m]);
    }

    // --- Documented choices ---------------------------------------------------

    /// FR-054-26 (documented choice; cases `err-cell`, `err-double` are
    /// non-normative): non-struct input.
    #[test]
    fn documented_choice_non_struct() {
        for value in [row_cell(vec![Value::Num(1.0)]), Value::Num(1.0)] {
            assert_eq!(
                identifier(call(&[value])),
                "RunMat:struct2cell:UndefinedFunction"
            );
        }
    }

    /// FR-054-26 (documented choice): shape rule for other struct shapes.
    #[test]
    fn documented_choice_shape_rule() {
        assert_eq!(result_shape(3, &[1, 1]), vec![3, 1]);
        assert_eq!(result_shape(2, &[1, 0]), vec![2, 1, 0]);
        assert_eq!(result_shape(2, &[0, 1]), vec![2, 0]);
        assert_eq!(result_shape(1, &[2, 3, 1]), vec![1, 2, 3]);
    }

    /// FR-054-23 (documented choice): arity and outputs.
    #[test]
    fn documented_choice_arity() {
        assert_eq!(identifier(call(&[])), "RunMat:struct2cell:InvalidInput");
        let s = builtin("struct", &[]);
        assert_eq!(
            identifier(call(&[s.clone(), s.clone()])),
            "RunMat:struct2cell:InvalidInput"
        );
        let _guard = crate::output_count::push_output_count(Some(2));
        let err = block_on(struct2cell_builtin(vec![s])).unwrap_err();
        assert_eq!(err.identifier(), Some("RunMat:struct2cell:TooManyOutputs"));
    }

    /// FR-054-28 (unresolved, struct2cell.q-warning-coverage): result only.
    #[test]
    fn unresolved_no_warning() {
        let s = builtin("struct", &[]);
        assert!(matches!(call(&[s]).unwrap(), Value::Cell(_)));
    }
}
