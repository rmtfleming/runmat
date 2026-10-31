//! Clean-room `strvcat` (stack text rows into a blank-padded char matrix,
//! skipping empty inputs).
//!
//! Clean-room feature `specs/054-types-structs-text/`. Normative behaviour
//! comes only from the approved export `strvcat` 0.2.0 (see that feature's
//! `provenance.md`). Each input is converted by RunMat's own `char` builtin
//! (decision D-054-8). Observed `MATLAB:*` identifiers are raised as
//! `RunMat:*` (D-054-1).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{CharArray, Value};

use crate::builtins::common::map_control_flow_with_builtin;
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const NAME: &str = "strvcat";

// ---------------------------------------------------------------------------
// Descriptor
// ---------------------------------------------------------------------------

const TEXT_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "t",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Variadic,
    default: None,
    description: "Char arrays, string scalars or numeric character codes.",
}];

const CELL_INPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "c",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Cell array of char.",
}];

const OUTPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "S",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Char matrix, one blank-padded row per non-empty text row.",
}];

const SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "S = strvcat(t1, t2, ...)",
        inputs: &TEXT_INPUTS,
        outputs: &OUTPUTS,
    },
    BuiltinSignatureDescriptor {
        label: "S = strvcat(c)",
        inputs: &CELL_INPUT,
        outputs: &OUTPUTS,
    },
];

const MIXED_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.STRVCAT.MIXED_INPUT",
    identifier: Some("RunMat:character:InvalidInput"),
    when: "A cell input is mixed with a non-cell input.",
    message: "strvcat: cell and non-cell inputs cannot be mixed",
};

const INVALID_CONVERSION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.STRVCAT.INVALID_CONVERSION",
    identifier: Some("RunMat:invalidConversion"),
    when: "An input is a struct.",
    message: "strvcat: cannot convert a struct to char",
};

const UNSUPPORTED_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.STRVCAT.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:strvcat:UnsupportedOption"),
    when: "Unobserved input classes: logical, other non-text classes, non-text cell elements.",
    message: "strvcat: this form is not yet specified",
};

const TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.STRVCAT.TOO_MANY_OUTPUTS",
    identifier: Some("RunMat:strvcat:TooManyOutputs"),
    when: "More than one output was requested.",
    message: "strvcat: too many output arguments",
};

const INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.STRVCAT.INTERNAL",
    identifier: Some("RunMat:strvcat:Internal"),
    when: "The result cannot be built.",
    message: "strvcat: internal runtime failure",
};

const ERRORS: [BuiltinErrorDescriptor; 5] = [
    MIXED_INPUT,
    INVALID_CONVERSION,
    UNSUPPORTED_OPTION,
    TOO_MANY_OUTPUTS,
    INTERNAL,
];

pub const STRVCAT_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ERRORS,
};

fn strvcat_error(
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
    name = "strvcat",
    category = "strings/core",
    summary = "Stack text vertically into a blank-padded char matrix, skipping empty inputs.",
    keywords = "strvcat,char,vertical concatenation,pad,text rows",
    accel = "cpu",
    descriptor(crate::builtins::strings::core::strvcat::STRVCAT_DESCRIPTOR),
    builtin_path = "crate::builtins::strings::core::strvcat"
)]
pub(crate) async fn strvcat_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    // FR-054-23
    if let Some(count) = crate::output_count::current_output_count() {
        if count > 1 {
            return Err(strvcat_error(
                &TOO_MANY_OUTPUTS,
                format!("strvcat returns one output, {count} were requested"),
            ));
        }
    }
    // FR-054-27: gpuArray inputs are gathered first.
    let mut inputs = Vec::with_capacity(args.len());
    for arg in &args {
        inputs.push(
            gather_if_needed_async(arg)
                .await
                .map_err(|err| map_control_flow_with_builtin(err, NAME))?,
        );
    }
    // FR-054-21, FR-054-27: a cell may not be mixed with a non-cell.
    let cells = inputs
        .iter()
        .filter(|v| matches!(v, Value::Cell(_)))
        .count();
    if cells > 0 && cells < inputs.len() {
        return Err(strvcat_error(
            &MIXED_INPUT,
            "cell inputs cannot be combined with other inputs",
        ));
    }
    let mut rows: Vec<Vec<char>> = Vec::new();
    for input in &inputs {
        match input {
            // FR-054-15, FR-054-14, FR-054-27
            Value::Cell(cell) => {
                for element in cell.iter_column_major() {
                    if !matches!(element, Value::CharArray(_) | Value::String(_)) {
                        return Err(strvcat_error(
                            &UNSUPPORTED_OPTION,
                            "cell elements must be char arrays or string scalars",
                        ));
                    }
                    append_rows(&mut rows, element).await?;
                }
            }
            // FR-054-22, FR-054-27
            Value::Struct(_) | Value::StructArray(_) => {
                return Err(strvcat_error(
                    &INVALID_CONVERSION,
                    "a struct cannot be converted to char",
                ))
            }
            // FR-054-13, FR-054-16..18, FR-054-27
            Value::CharArray(_)
            | Value::String(_)
            | Value::StringArray(_)
            | Value::Num(_)
            | Value::Int(_)
            | Value::Tensor(_)
            | Value::Complex(_, _)
            | Value::ComplexTensor(_)
            | Value::SparseTensor(_) => append_rows(&mut rows, input).await?,
            // FR-054-27: logical and every other class.
            _ => {
                return Err(strvcat_error(
                    &UNSUPPORTED_OPTION,
                    "inputs must be text or numeric character codes",
                ))
            }
        }
    }
    // FR-054-13, FR-054-19
    let width = rows.iter().map(Vec::len).max().unwrap_or(0);
    let height = rows.len();
    if height == 0 {
        return CharArray::new(Vec::new(), 0, 0)
            .map(Value::CharArray)
            .map_err(|err| strvcat_error(&INTERNAL, err));
    }
    let mut data = Vec::with_capacity(height * width);
    for mut row in rows {
        row.resize(width, ' ');
        data.extend(row);
    }
    CharArray::new(data, height, width)
        .map(Value::CharArray)
        .map_err(|err| strvcat_error(&INTERNAL, err))
}

/// Convert one input with RunMat's `char` (D-054-8) and append its rows,
/// unless the conversion has no elements (FR-054-14, FR-054-27).
async fn append_rows(rows: &mut Vec<Vec<char>>, value: &Value) -> BuiltinResult<()> {
    let converted = crate::call_builtin_async("char", std::slice::from_ref(value)).await?;
    let Value::CharArray(chars) = converted else {
        return Err(strvcat_error(&INTERNAL, "char did not return a char array"));
    };
    if chars.data.is_empty() {
        return Ok(());
    }
    // CharArray data is row-major within its 2-D page.
    for r in 0..chars.rows {
        rows.push(chars.data[r * chars.cols..(r + 1) * chars.cols].to_vec());
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod strvcat_tests {
    use super::strvcat_builtin;
    use crate::BuiltinResult;
    use futures::executor::block_on;
    use runmat_value::{CellArray, CharArray, IntValue, StringArray, StructValue, Tensor, Value};

    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("strvcat", args))
    }

    fn chars(text: &str) -> Value {
        Value::CharArray(CharArray::new_row(text))
    }

    fn cell_row(values: Vec<Value>) -> Value {
        let n = values.len();
        Value::Cell(CellArray::new(values, 1, n).unwrap())
    }

    /// (shape, rows) of a char result.
    fn rows(value: Value) -> (Vec<usize>, Vec<String>) {
        match value {
            Value::CharArray(c) => {
                let rows = (0..c.rows)
                    .map(|r| c.data[r * c.cols..(r + 1) * c.cols].iter().collect())
                    .collect();
                (c.shape.clone(), rows)
            }
            other => panic!("expected a char result, got {other:?}"),
        }
    }

    fn identifier(result: BuiltinResult<Value>) -> String {
        match result {
            Ok(value) => panic!("expected an error, got {value:?}"),
            Err(err) => err.identifier().unwrap_or("<none>").to_string(),
        }
    }

    const UNSUPPORTED: &str = "RunMat:strvcat:UnsupportedOption";

    // --- Observed (normative): one test per observed case --------------------

    /// case `pad` — strvcat.signature-primary, strvcat.pad
    #[test]
    fn observed_pad() {
        let out = call(&[chars("a"), chars("bcd"), chars("ef")]).unwrap();
        assert_eq!(
            rows(out),
            (vec![3, 3], vec!["a  ".into(), "bcd".into(), "ef ".into()])
        );
    }

    /// case `pad-size` — strvcat.pad (size [3 3])
    #[test]
    fn observed_pad_size() {
        let (shape, _) = rows(call(&[chars("a"), chars("bcd"), chars("ef")]).unwrap());
        assert_eq!(shape, vec![3, 3]);
    }

    /// case `skip-empty` — strvcat.skip-empty (size [2 1])
    #[test]
    fn observed_skip_empty() {
        let empty = Value::CharArray(CharArray::new(Vec::new(), 0, 0).unwrap());
        let (shape, r) = rows(call(&[chars("a"), empty, chars("b")]).unwrap());
        assert_eq!(shape, vec![2, 1]);
        assert_eq!(r, vec!["a".to_string(), "b".to_string()]);
    }

    /// case `cell` — strvcat.cell
    #[test]
    fn observed_cell() {
        let out = call(&[cell_row(vec![chars("ab"), chars("c")])]).unwrap();
        assert_eq!(rows(out), (vec![2, 2], vec!["ab".into(), "c ".into()]));
    }

    /// case `cell-empty` — strvcat.skip-empty (inside a cell; size [2 1])
    #[test]
    fn observed_cell_empty() {
        let empty = Value::CharArray(CharArray::new(Vec::new(), 0, 0).unwrap());
        let out = call(&[cell_row(vec![chars("a"), empty, chars("b")])]).unwrap();
        assert_eq!(rows(out).0, vec![2, 1]);
    }

    /// case `err-mixed` — strvcat.error-mixed
    #[test]
    fn observed_err_mixed() {
        let cell = cell_row(vec![chars("yy"), chars("z")]);
        assert_eq!(
            identifier(call(&[chars("x"), cell])),
            "RunMat:character:InvalidInput"
        );
    }

    /// case `numeric` — strvcat.numeric
    #[test]
    fn observed_numeric() {
        let out = call(&[Value::Num(65.0), chars("b")]).unwrap();
        assert_eq!(rows(out), (vec![2, 1], vec!["A".into(), "b".into()]));
    }

    /// case `matrix-arg` — strvcat.matrix-arg
    #[test]
    fn observed_matrix_arg() {
        let m = Value::CharArray(CharArray::new("abcd".chars().collect(), 2, 2).unwrap());
        let out = call(&[m, chars("e")]).unwrap();
        assert_eq!(
            rows(out),
            (vec![3, 2], vec!["ab".into(), "cd".into(), "e ".into()])
        );
    }

    /// case `one` — strvcat.one
    #[test]
    fn observed_one() {
        let out = call(&[chars("abc")]).unwrap();
        assert_eq!(rows(out), (vec![1, 3], vec!["abc".into()]));
    }

    /// case `all-empty` — strvcat.empty
    #[test]
    fn observed_all_empty() {
        let empty = Value::CharArray(CharArray::new(Vec::new(), 0, 0).unwrap());
        let out = call(&[empty.clone(), empty]).unwrap();
        assert_eq!(rows(out), (vec![0, 0], Vec::new()));
    }

    /// case `no-args` — strvcat.empty
    #[test]
    fn observed_no_args() {
        assert_eq!(rows(call(&[]).unwrap()), (vec![0, 0], Vec::new()));
    }

    /// case `string` — strvcat.string
    #[test]
    fn observed_string() {
        let out = call(&[Value::String("ab".into()), Value::String("c".into())]).unwrap();
        assert_eq!(rows(out), (vec![2, 2], vec!["ab".into(), "c ".into()]));
    }

    /// case `trailing-blanks` — strvcat.pad (input blanks kept)
    #[test]
    fn observed_trailing_blanks() {
        let out = call(&[chars("a  "), chars("b")]).unwrap();
        assert_eq!(rows(out), (vec![2, 3], vec!["a  ".into(), "b  ".into()]));
    }

    /// case `err-struct` — strvcat.error-struct
    #[test]
    fn observed_err_struct() {
        let mut s = StructValue::new();
        s.fields.insert("a".into(), Value::Num(1.0));
        assert_eq!(
            identifier(call(&[Value::Struct(s)])),
            "RunMat:invalidConversion"
        );
    }

    // --- Documented choices ---------------------------------------------------

    /// FR-054-27 (documented choice): empty string / numeric skipped; string
    /// arrays and integer codes converted as by `char`.
    #[test]
    fn documented_choice_conversions() {
        let empty_num = Value::Tensor(Tensor::new(Vec::new(), vec![0, 0]).unwrap());
        let out = call(&[Value::String(String::new()), empty_num, chars("x")]).unwrap();
        assert_eq!(rows(out), (vec![1, 1], vec!["x".into()]));

        let strings = StringArray::new(vec!["ab".into(), "c".into()], vec![1, 2]).unwrap();
        let out = call(&[Value::StringArray(strings), Value::Int(IntValue::U8(66))]).unwrap();
        assert_eq!(
            rows(out),
            (vec![3, 2], vec!["ab".into(), "c ".into(), "B ".into()])
        );

        let codes = Value::Tensor(Tensor::new(vec![72.0, 105.0], vec![1, 2]).unwrap());
        assert_eq!(rows(call(&[codes]).unwrap()).1, vec!["Hi".to_string()]);
    }

    /// FR-054-27 (documented choice): several cells; cell elements in
    /// column-major order; a non-text cell element.
    #[test]
    fn documented_choice_cells() {
        let a = cell_row(vec![chars("a")]);
        let col = Value::Cell(CellArray::new(vec![chars("b"), chars("cc")], 2, 1).unwrap());
        let out = call(&[a, col]).unwrap();
        assert_eq!(
            rows(out),
            (vec![3, 2], vec!["a ".into(), "b ".into(), "cc".into()])
        );
        let bad = cell_row(vec![chars("a"), Value::Num(65.0)]);
        assert_eq!(identifier(call(&[bad])), UNSUPPORTED);
    }

    /// FR-054-27 (documented choice): logical and other classes.
    #[test]
    fn documented_choice_other_classes() {
        for value in [Value::Bool(true), Value::FunctionHandle("sin".into())] {
            assert_eq!(identifier(call(&[value])), UNSUPPORTED);
        }
    }

    /// FR-054-23 (documented choice): more than one output.
    #[test]
    fn documented_choice_too_many_outputs() {
        let _guard = crate::output_count::push_output_count(Some(2));
        let err = block_on(strvcat_builtin(vec![chars("a")])).unwrap_err();
        assert_eq!(err.identifier(), Some("RunMat:strvcat:TooManyOutputs"));
    }

    /// FR-054-28 (unresolved, strvcat.q-warning-coverage): result only.
    #[test]
    fn unresolved_no_warning() {
        assert!(matches!(call(&[chars("a")]).unwrap(), Value::CharArray(_)));
    }
}
