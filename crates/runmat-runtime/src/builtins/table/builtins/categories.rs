//! Clean-room `categories` (list the categories of a categorical array).
//!
//! Clean-room feature `specs/052-data-n2/`. Normative behaviour comes only
//! from the approved export `categories` 0.2.0 (see that feature's
//! `provenance.md`). The category list and its order are those stored by
//! RunMat's `categorical` constructor; this builtin reports them.

use super::*;
use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;

use crate::{build_runtime_error, RuntimeError};

const NAME: &str = "categories";

// ---------------------------------------------------------------------------
// Descriptors
// ---------------------------------------------------------------------------

const CATEGORIES_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "A",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Categorical array.",
}];
const CATEGORIES_OUTPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "C",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "n-by-1 cell array of char vectors (0-by-0 when there are no categories).",
}];
const CATEGORIES_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "C = categories(A)",
    inputs: &CATEGORIES_INPUTS,
    outputs: &CATEGORIES_OUTPUTS,
}];

const CATEGORIES_ERRORS: [BuiltinErrorDescriptor; 5] = [
    BuiltinErrorDescriptor {
        code: "RM.CATEGORIES.INVALID_INPUT",
        identifier: Some("RunMat:categories:InvalidInput"),
        when: "The categorical input is missing or malformed.",
        message: "categories: invalid input",
    },
    BuiltinErrorDescriptor {
        code: "RM.CATEGORIES.UNSUPPORTED_OPTION",
        identifier: Some("RunMat:categories:UnsupportedOption"),
        when: "The call uses a form that is not yet specified for RunMat.",
        message: "categories: form not yet specified",
    },
    BuiltinErrorDescriptor {
        code: "RM.CATEGORIES.INTERNAL",
        identifier: Some("RunMat:categories:Internal"),
        when: "RunMat cannot build the result value.",
        message: "categories: internal error",
    },
    BuiltinErrorDescriptor {
        code: "RM.CATEGORIES.UNDEFINED_FUNCTION",
        identifier: Some("RunMat:categories:UndefinedFunction"),
        when: "The input is not a categorical array.",
        message: "categories: undefined for non-categorical input",
    },
    BuiltinErrorDescriptor {
        code: "RM.CATEGORIES.TOO_MANY_OUTPUTS",
        identifier: Some("RunMat:categories:TooManyOutputs"),
        when: "More than one output was requested.",
        message: "categories: too many outputs",
    },
];

const INVALID_INPUT: usize = 0;
const UNSUPPORTED_OPTION: usize = 1;
const INTERNAL: usize = 2;
const UNDEFINED_FUNCTION: usize = 3;
const TOO_MANY_OUTPUTS: usize = 4;

pub const CATEGORIES_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &CATEGORIES_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &CATEGORIES_ERRORS,
};

fn categories_error(kind: usize, detail: impl Into<String>) -> RuntimeError {
    let mut builder = build_runtime_error(format!("{NAME}: {}", detail.into())).with_builtin(NAME);
    if let Some(identifier) = CATEGORIES_ERRORS[kind].identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

// ---------------------------------------------------------------------------
// Builtin
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "categories",
    category = "table",
    summary = "Return the categories of a categorical array as a cell column of char vectors.",
    keywords = "categories,categorical,levels,table",
    accel = "cpu",
    descriptor(crate::builtins::table::builtins::categories::CATEGORIES_DESCRIPTOR),
    builtin_path = "crate::builtins::table::builtins"
)]
pub(crate) async fn categories_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    // U-052-1: one output.
    if let Some(count) = crate::output_count::current_output_count() {
        if count > 1 {
            return Err(categories_error(
                TOO_MANY_OUTPUTS,
                format!("categories returns one output, {count} were requested"),
            ));
        }
    }
    let args = gather_values(&args).await?;
    match args.len() {
        0 => return Err(categories_error(INVALID_INPUT, "expected categories(A)")),
        1 => {}
        _ => {
            // U-052-3
            return Err(categories_error(
                UNSUPPORTED_OPTION,
                "only categories(A) is specified",
            ));
        }
    }
    // U-052-3 (matches the non-normative categories.non-categorical).
    let object = match &args[0] {
        Value::Object(object) if object.is_class(CATEGORICAL_CLASS) => object,
        _ => {
            return Err(categories_error(
                UNDEFINED_FUNCTION,
                "undefined for input that is not a categorical array",
            ))
        }
    };
    let names = categorical_categories(object)
        .map_err(|err| categories_error(INVALID_INPUT, err.message()))?;
    // categories.output-class: n-by-1 cell of char rows, in stored order
    // (categories.sorted-default, categories.explicit-order,
    // categories.numeric, categories.missing are fixed by the constructor).
    // categories.empty / U-052-3: no categories gives 0-by-0.
    let rows = names.len();
    let cols = usize::from(rows > 0);
    let data = names
        .iter()
        .map(|name| Value::CharArray(CharArray::new_row(name)))
        .collect();
    CellArray::new(data, rows, cols)
        .map(Value::Cell)
        .map_err(|err| categories_error(INTERNAL, err))
}

#[cfg(test)]
mod categories_tests {
    use super::*;
    use futures::executor::block_on;

    // ---- helpers ----------------------------------------------------------

    fn chr(text: &str) -> Value {
        Value::CharArray(CharArray::new_row(text))
    }

    fn cellstr(values: &[&str]) -> Value {
        Value::Cell(
            CellArray::new(values.iter().map(|s| chr(s)).collect(), 1, values.len()).unwrap(),
        )
    }

    fn call(name: &str, args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async(name, args))
    }

    fn identifier(err: RuntimeError) -> String {
        err.identifier().unwrap_or("<none>").to_string()
    }

    fn categorical(args: &[Value]) -> Value {
        call("categorical", args).expect("categorical")
    }

    /// Shape and char contents of a cell result; panics unless every
    /// element is a char row vector.
    fn cell_chars(value: &Value) -> (Vec<usize>, Vec<String>) {
        let Value::Cell(cell) = value else {
            panic!("expected cell result, got {value:?}");
        };
        let texts = cell
            .data
            .iter()
            .map(|entry| match entry {
                Value::CharArray(chars) => {
                    assert_eq!(chars.rows, 1, "char row vector");
                    chars.data.iter().collect()
                }
                other => panic!("expected char element, got {other:?}"),
            })
            .collect();
        (cell.shape.clone(), texts)
    }

    fn categories(a: Value) -> (Vec<usize>, Vec<String>) {
        cell_chars(&call("categories", &[a]).expect("categories"))
    }

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|s| s.to_string()).collect()
    }

    // ---- observed (normative) ---------------------------------------------

    /// categories.signature-primary, categories.output-class,
    /// categories.sorted-default (case `from-cellstr`).
    #[test]
    fn observed_categories_from_cellstr_sorted() {
        let c = categorical(&[cellstr(&["b", "a", "c", "a"])]);
        assert_eq!(categories(c), (vec![3, 1], strings(&["a", "b", "c"])));
    }

    /// categories.sorted-default (cases `case`, `string-src`).
    #[test]
    fn observed_categories_case_and_string_source() {
        let c = categorical(&[cellstr(&["b", "B", "a"])]);
        assert_eq!(categories(c), (vec![3, 1], strings(&["B", "a", "b"])));
        let s = Value::StringArray(StringArray::new(strings(&["y", "x"]), vec![1, 2]).unwrap());
        assert_eq!(
            categories(categorical(&[s])),
            (vec![2, 1], strings(&["x", "y"]))
        );
    }

    /// categories.explicit-order (cases `explicit`, `ordinal`).
    #[test]
    fn observed_categories_explicit_order() {
        let c = categorical(&[cellstr(&["b", "a"]), cellstr(&["c", "b", "a"])]);
        assert_eq!(categories(c), (vec![3, 1], strings(&["c", "b", "a"])));
        let c = categorical(&[
            cellstr(&["lo", "hi"]),
            cellstr(&["lo", "hi"]),
            chr("Ordinal"),
            Value::Bool(true),
        ]);
        assert_eq!(categories(c), (vec![2, 1], strings(&["lo", "hi"])));
    }

    /// categories.numeric, categories.output-class (case `numeric`).
    #[test]
    fn observed_categories_numeric_source() {
        let data = Value::Tensor(Tensor::new(vec![3.0, 1.0, 2.0, 1.0], vec![1, 4]).unwrap());
        let c = categorical(&[data]);
        assert_eq!(categories(c), (vec![3, 1], strings(&["1", "2", "3"])));
    }

    /// categories.missing (case `missing`).
    #[test]
    fn observed_categories_empty_text_not_a_category() {
        let c = categorical(&[cellstr(&["a", ""])]);
        assert_eq!(categories(c), (vec![1, 1], strings(&["a"])));
    }

    /// categories.empty (case `empty`).
    #[test]
    fn observed_categories_empty_categorical() {
        let empty = Value::Cell(CellArray::new(Vec::new(), 0, 0).unwrap());
        let c = categorical(&[empty]);
        assert_eq!(categories(c), (vec![0, 0], Vec::new()));
    }

    // ---- documented choices (U-052-1, U-052-3) ------------------------------

    /// U-052-3 (matches the non-normative categories.non-categorical).
    #[test]
    fn unresolved_choice_categories_non_categorical() {
        for bad in [cellstr(&["a", "b"]), Value::Num(1.0), chr("a")] {
            let err = call("categories", &[bad]).unwrap_err();
            assert_eq!(identifier(err), "RunMat:categories:UndefinedFunction");
        }
    }

    /// U-052-1, U-052-3: arity.
    #[test]
    fn unresolved_choice_categories_arity() {
        let err = call("categories", &[]).unwrap_err();
        assert_eq!(identifier(err), "RunMat:categories:InvalidInput");
        let c = categorical(&[cellstr(&["a"])]);
        let err = call("categories", &[c, chr("x")]).unwrap_err();
        assert_eq!(identifier(err), "RunMat:categories:UnsupportedOption");
    }

    /// U-052-3: no categories gives 0x0 even for a non-empty categorical;
    /// categories are reported for a categorical with no elements.
    #[test]
    fn unresolved_choice_categories_shapes() {
        let c = categorical(&[cellstr(&[""])]);
        assert_eq!(categories(c), (vec![0, 0], Vec::new()));
        let empty = Value::Cell(CellArray::new(Vec::new(), 0, 0).unwrap());
        let c = categorical(&[empty, cellstr(&["p", "q"])]);
        assert_eq!(categories(c), (vec![2, 1], strings(&["p", "q"])));
    }

    /// U-052-1: called directly under `push_output_count` (dispatcher quirk).
    #[test]
    fn unresolved_choice_categories_output_count() {
        let c = categorical(&[cellstr(&["a"])]);
        {
            let _guard = crate::output_count::push_output_count(Some(2));
            let err = block_on(categories_builtin(vec![c.clone()])).unwrap_err();
            assert_eq!(identifier(err), "RunMat:categories:TooManyOutputs");
        }
        let _guard = crate::output_count::push_output_count(Some(1));
        let out = block_on(categories_builtin(vec![c])).unwrap();
        assert_eq!(cell_chars(&out), (vec![1, 1], strings(&["a"])));
    }
}
