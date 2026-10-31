//! Clean-room `convertvars` (convert selected table variables).
//!
//! Clean-room feature `specs/056-table-variables/`. Normative behaviour
//! comes only from the approved export `convertvars` 0.2.0 (see that
//! feature's `provenance.md`). Conversions are delegated to RunMat's
//! existing conversion builtins, called by name.

use super::variable_support::*;
use super::*;
use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::mapped_callable::MappedCallable;

// ---------------------------------------------------------------------------
// Descriptors
// ---------------------------------------------------------------------------

const fn param(name: &'static str, description: &'static str) -> BuiltinParamDescriptor {
    BuiltinParamDescriptor {
        name,
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description,
    }
}

const TABLE_INPUT: BuiltinParamDescriptor = param("T", "Input table.");
const VARS_INPUT: BuiltinParamDescriptor = param(
    "vars",
    "Variables: a name, string array or cellstr of names, numeric indices, or a logical mask.",
);
const CONVERTVARS_INPUTS_TYPE: [BuiltinParamDescriptor; 3] = [
    TABLE_INPUT,
    VARS_INPUT,
    param(
        "type",
        "Target class name: double, single, int8..uint64, logical, string or categorical.",
    ),
];
const CONVERTVARS_INPUTS_FUNC: [BuiltinParamDescriptor; 3] = [
    TABLE_INPUT,
    VARS_INPUT,
    param("func", "Function handle applied to each selected variable."),
];
const CONVERTVARS_OUTPUTS: [BuiltinParamDescriptor; 1] =
    [param("T2", "Table with the selected variables converted.")];
const CONVERTVARS_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "T2 = convertvars(T, vars, type)",
        inputs: &CONVERTVARS_INPUTS_TYPE,
        outputs: &CONVERTVARS_OUTPUTS,
    },
    BuiltinSignatureDescriptor {
        label: "T2 = convertvars(T, vars, func)",
        inputs: &CONVERTVARS_INPUTS_FUNC,
        outputs: &CONVERTVARS_OUTPUTS,
    },
];

const CONVERTVARS_ERRORS: [BuiltinErrorDescriptor; 10] = [
    BuiltinErrorDescriptor {
        code: "RM.CONVERTVARS.INVALID_INPUT",
        identifier: Some("RunMat:convertvars:InvalidInput"),
        when: "The first argument is not a table.",
        message: "convertvars: expected a table",
    },
    BuiltinErrorDescriptor {
        code: "RM.CONVERTVARS.INVALID_ARGUMENT",
        identifier: Some("RunMat:convertvars:InvalidArgument"),
        when: "An argument has an unrecognised form.",
        message: "convertvars: invalid argument",
    },
    BuiltinErrorDescriptor {
        code: "RM.CONVERTVARS.UNSUPPORTED_OPTION",
        identifier: Some("RunMat:convertvars:UnsupportedOption"),
        when: "The call uses a form the approved specification leaves unresolved.",
        message: "convertvars: form not yet specified",
    },
    BuiltinErrorDescriptor {
        code: "RM.CONVERTVARS.TOO_MANY_OUTPUTS",
        identifier: Some("RunMat:convertvars:TooManyOutputs"),
        when: "More than one output was requested.",
        message: "convertvars: too many outputs",
    },
    BuiltinErrorDescriptor {
        code: "RM.CONVERTVARS.INTERNAL",
        identifier: Some("RunMat:convertvars:Internal"),
        when: "RunMat cannot build the result value.",
        message: "convertvars: internal error",
    },
    BuiltinErrorDescriptor {
        code: "RM.CONVERTVARS.UNRECOGNIZED_VAR_NAME",
        identifier: Some("RunMat:table:UnrecognizedVarName"),
        when: "A selected variable is not in the table.",
        message: "convertvars: unrecognized table variable name",
    },
    BuiltinErrorDescriptor {
        code: "RM.CONVERTVARS.VAR_INDEX_OUT_OF_RANGE",
        identifier: Some("RunMat:table:VarIndexOutOfRange"),
        when: "A numeric index is past the last variable.",
        message: "convertvars: variable index out of range",
    },
    BuiltinErrorDescriptor {
        code: "RM.CONVERTVARS.BAD_SUBSCRIPT",
        identifier: Some("RunMat:badsubscript"),
        when: "A numeric index is zero, negative or not an integer.",
        message: "convertvars: invalid variable index",
    },
    BuiltinErrorDescriptor {
        code: "RM.CONVERTVARS.INVALID_TEXT_TYPE",
        identifier: Some("RunMat:table:convertvars:InvalidTextType"),
        when: "The type name is not recognised.",
        message: "convertvars: unrecognized type name",
    },
    BuiltinErrorDescriptor {
        code: "RM.CONVERTVARS.CONVERSION_FAILED",
        identifier: Some("RunMat:table:convertvars:VariableTypesConversionFailed"),
        when: "A selected variable cannot be converted to the type.",
        message: "convertvars: conversion failed",
    },
];

const INVALID_TEXT_TYPE: &str = "INVALID_TEXT_TYPE";
const CONVERSION_FAILED: &str = "CONVERSION_FAILED";

pub const CONVERTVARS_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &CONVERTVARS_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &CONVERTVARS_ERRORS,
};

const CTX: Ctx = Ctx {
    name: "convertvars",
    errors: &CONVERTVARS_ERRORS,
};

// ---------------------------------------------------------------------------
// Builtin
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "convertvars",
    category = "table",
    summary = "Convert selected table variables to a type or by a function.",
    keywords = "convertvars,table,variables,convert,type,class",
    accel = "cpu",
    descriptor(crate::builtins::table::builtins::convertvars::CONVERTVARS_DESCRIPTOR),
    builtin_path = "crate::builtins::table::builtins"
)]
pub(crate) async fn convertvars_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    CTX.check_outputs()?;
    let [table, vars, target] = args.as_slice() else {
        return Err(CTX.error(
            INVALID_ARGUMENT,
            "expected convertvars(T, vars, type) or convertvars(T, vars, func)",
        ));
    };
    let object = CTX.table_input(table).await?;
    let (names, mut columns) = columns_of(&object)?;
    let mut selected = CTX.select(vars, &names).await?;
    // U-056-14: a variable selected twice is converted once.
    let mut seen = vec![false; names.len()];
    selected.retain(|idx| !std::mem::replace(&mut seen[*idx], true));

    let conversion = parse_target(&gathered(target).await?)?;
    for idx in selected {
        let before = value_row_count(&columns[idx])?;
        let converted = match &conversion {
            Conversion::Type(type_name) => {
                convert_to(&columns[idx], type_name, &names[idx]).await?
            }
            Conversion::Function(callable) => {
                // U-056-14: the function's own error propagates.
                callable
                    .invoke(&[columns[idx].clone()])
                    .await
                    .map_err(|err| match err {
                        crate::builtins::common::mapped_callable::CallableCallError::Runtime(
                            err,
                        ) => err,
                        _ => CTX.error(INVALID_ARGUMENT, "the function could not be called"),
                    })
                    .and_then(|value| {
                        super::varfun::first_output(value).ok_or_else(|| {
                            CTX.error(UNSUPPORTED_OPTION, "the function returned no value")
                        })
                    })?
            }
        };
        if value_row_count(&converted)? != before {
            return Err(match conversion {
                Conversion::Type(_) => CTX.error(
                    CONVERSION_FAILED,
                    format!("converting '{}' changed its number of rows", names[idx]),
                ),
                Conversion::Function(_) => CTX.error(
                    UNSUPPORTED_OPTION,
                    "a function result with a different number of rows is not yet specified",
                ),
            });
        }
        columns[idx] = converted;
    }
    with_variables(object, names, columns)
}

enum Conversion {
    Type(&'static str),
    Function(MappedCallable),
}

/// Type names converted by RunMat's builtin of the same name (U-056-13).
const SUPPORTED_TYPES: [&str; 13] = [
    "double",
    "single",
    "int8",
    "int16",
    "int32",
    "int64",
    "uint8",
    "uint16",
    "uint32",
    "uint64",
    "logical",
    "string",
    "categorical",
];

/// Recognised class names that are not yet specified as targets (U-056-13).
const UNSUPPORTED_TYPES: [&str; 9] = [
    "char",
    "cell",
    "cellstr",
    "datetime",
    "duration",
    "calendarDuration",
    "struct",
    "table",
    "timetable",
];

const NUMERIC_TYPES: [&str; 11] = [
    "double", "single", "int8", "int16", "int32", "int64", "uint8", "uint16", "uint32", "uint64",
    "logical",
];

fn parse_target(target: &Value) -> BuiltinResult<Conversion> {
    if let Some(name) = single_name(target) {
        if let Some(found) = SUPPORTED_TYPES.iter().find(|t| **t == name) {
            return Ok(Conversion::Type(found));
        }
        if UNSUPPORTED_TYPES.contains(&name.as_str()) {
            return Err(CTX.error(
                UNSUPPORTED_OPTION,
                format!("conversion to '{name}' is not yet specified"),
            ));
        }
        return Err(CTX.error(
            INVALID_TEXT_TYPE,
            format!("'{name}' is not a recognised type name"),
        ));
    }
    match target {
        Value::FunctionHandle(_)
        | Value::ExternalFunctionHandle(_)
        | Value::MethodFunctionHandle(_)
        | Value::BoundFunctionHandle { .. }
        | Value::Closure(_) => MappedCallable::parse(target.clone())
            .map(Conversion::Function)
            .map_err(|err| CTX.error(INVALID_ARGUMENT, format!("invalid function: {err:?}"))),
        other => Err(CTX.error(
            INVALID_ARGUMENT,
            format!(
                "the type must be a type name or a function handle, got {}",
                class_label(other)
            ),
        )),
    }
}

/// convertvars.types / convertvars.error-cell-to-double (U-056-14).
async fn convert_to(value: &Value, type_name: &str, name: &str) -> BuiltinResult<Value> {
    let failed = |detail: String| {
        CTX.error(
            CONVERSION_FAILED,
            format!("cannot convert variable '{name}' to {type_name}: {detail}"),
        )
    };
    if matches!(value, Value::Cell(_)) && NUMERIC_TYPES.contains(&type_name) {
        return Err(failed("a cell variable has no numeric value".into()));
    }
    crate::call_builtin_async(type_name, std::slice::from_ref(value))
        .await
        .map_err(|err| failed(err.message().to_string()))
}

#[cfg(test)]
mod convertvars_tests {
    use super::super::variable_support::test_helpers::*;
    use super::*;
    use futures::executor::block_on;

    /// `table({'a';'b'},[1;2],{'x';'y'},'VariableNames',{'p','n','q'})`.
    fn pnq() -> Value {
        table(
            vec![
                cellcol(&["a", "b"]),
                column(&[1.0, 2.0]),
                cellcol(&["x", "y"]),
            ],
            &["p", "n", "q"],
        )
    }

    fn convert(vars: Value, to: Value) -> Value {
        ok("convertvars", &[pnq(), vars, to])
    }

    fn assert_strings(out: &Value, name: &str, expected: &[&str]) {
        let value = variable(out, name);
        assert_eq!(class_of(&value), "string");
        let Value::StringArray(array) = &value else {
            panic!("expected string array, got {value:?}");
        };
        assert_eq!(array.shape, vec![expected.len(), 1]);
        assert_eq!(array.data, sv(expected));
    }

    fn assert_cellstr(out: &Value, name: &str, expected: &[&str]) {
        let value = variable(out, name);
        assert_eq!(class_of(&value), "cell");
        let Value::Cell(cell) = &value else {
            unreachable!()
        };
        assert_eq!(cell.shape, vec![expected.len(), 1]);
        assert_eq!(cell_texts(&value), sv(expected));
    }

    fn assert_n_double(out: &Value) {
        let value = variable(out, "n");
        assert_eq!(class_of(&value), "double");
        assert_eq!(doubles(&value), (vec![2, 1], vec![1.0, 2.0]));
    }

    // ---- observed (normative) ---------------------------------------------

    /// convertvars.signature-primary, convertvars.selection,
    /// convertvars.types (case `string`).
    #[test]
    fn observed_cellstr_to_string() {
        let out = convert(text("p"), text("string"));
        assert_eq!(names(&out), sv(&["p", "n", "q"]));
        assert_strings(&out, "p", &["a", "b"]);
        assert_n_double(&out);
        assert_cellstr(&out, "q", &["x", "y"]);
    }

    /// convertvars.selection (case `string-array-names`).
    #[test]
    fn observed_string_array_selection() {
        let out = convert(strings(&["p", "q"]), text("string"));
        assert_strings(&out, "p", &["a", "b"]);
        assert_n_double(&out);
        assert_strings(&out, "q", &["x", "y"]);
    }

    /// convertvars.cobra-form (case `preserve-names`).
    #[test]
    fn observed_names_with_spaces() {
        let t = table_from_columns(
            sv(&["Reaction ID", "Taxon"]),
            vec![cellcol(&["a", "b"]), cellcol(&["c", "d"])],
        )
        .unwrap();
        let out = ok(
            "convertvars",
            &[t, strings(&["Reaction ID", "Taxon"]), text("string")],
        );
        assert_eq!(names(&out), sv(&["Reaction ID", "Taxon"]));
        assert_strings(&out, "Reaction ID", &["a", "b"]);
        assert_strings(&out, "Taxon", &["c", "d"]);
    }

    /// convertvars.selection (cases `index`, `mask`).
    #[test]
    fn observed_index_and_mask_selection() {
        let out = convert(Value::Num(3.0), text("string"));
        assert_cellstr(&out, "p", &["a", "b"]);
        assert_n_double(&out);
        assert_strings(&out, "q", &["x", "y"]);
        let out = convert(mask(&[true, false, false]), text("string"));
        assert_strings(&out, "p", &["a", "b"]);
        assert_cellstr(&out, "q", &["x", "y"]);
    }

    /// convertvars.types (case `categorical`).
    #[test]
    fn observed_cellstr_to_categorical() {
        let out = convert(text("p"), text("categorical"));
        let value = variable(&out, "p");
        assert_eq!(class_of(&value), "categorical");
        let categories = ok("categories", std::slice::from_ref(&value));
        assert_eq!(cell_texts(&categories), sv(&["a", "b"]));
        assert_eq!(categorical_labels(&value).unwrap(), sv(&["a", "b"]));
        assert_eq!(size(&out), (2, 3));
        assert_cellstr(&out, "q", &["x", "y"]);
    }

    /// convertvars.types (case `numeric-to-string`).
    #[test]
    fn observed_double_to_string() {
        let out = convert(text("n"), text("string"));
        assert_strings(&out, "n", &["1", "2"]);
        assert_cellstr(&out, "p", &["a", "b"]);
    }

    /// convertvars.types (case `to-int8`).
    #[test]
    fn observed_double_to_int8() {
        let out = convert(text("n"), text("int8"));
        let value = variable(&out, "n");
        assert_eq!(class_of(&value), "int8");
        assert_eq!(doubles(&value), (vec![2, 1], vec![1.0, 2.0]));
    }

    /// convertvars.types (case `same-type`).
    #[test]
    fn observed_double_to_double_unchanged() {
        let out = convert(text("n"), text("double"));
        assert_n_double(&out);
        assert_cellstr(&out, "p", &["a", "b"]);
        assert_cellstr(&out, "q", &["x", "y"]);
    }

    /// convertvars.function (case `function`).
    #[test]
    fn observed_function_handle() {
        let out = convert(text("p"), Value::FunctionHandle("upper".into()));
        assert_cellstr(&out, "p", &["A", "B"]);
        assert_n_double(&out);
        assert_cellstr(&out, "q", &["x", "y"]);
    }

    /// convertvars.error-unknown (case `err-unknown`).
    #[test]
    fn observed_error_unknown_variable() {
        assert_eq!(
            err_id("convertvars", &[pnq(), text("nope"), text("string")]),
            "RunMat:table:UnrecognizedVarName"
        );
    }

    /// convertvars.error-bad-type (case `err-bad-type`).
    #[test]
    fn observed_error_bad_type() {
        assert_eq!(
            err_id("convertvars", &[pnq(), text("p"), text("nope")]),
            "RunMat:table:convertvars:InvalidTextType"
        );
    }

    /// convertvars.error-cell-to-double (case `err-cell-to-double`).
    #[test]
    fn observed_error_cell_to_double() {
        assert_eq!(
            err_id("convertvars", &[pnq(), text("p"), text("double")]),
            "RunMat:table:convertvars:VariableTypesConversionFailed"
        );
    }

    // ---- documented choices (FR-056-40, 42, 52, 53) ------------------------

    /// U-056-1 (convertvars.non-table is non-normative), arity.
    #[test]
    fn rejects_non_table_and_arity() {
        let s = ok("struct", &[text("a"), Value::Num(1.0)]);
        assert_eq!(
            err_id("convertvars", &[s, text("a"), text("string")]),
            "RunMat:convertvars:InvalidInput"
        );
        assert_eq!(
            err_id("convertvars", &[pnq(), text("p")]),
            "RunMat:convertvars:InvalidArgument"
        );
    }

    /// U-056-1: one output (called directly; the dispatcher retries).
    #[test]
    fn unresolved_too_many_outputs() {
        let _guard = crate::output_count::push_output_count(Some(2));
        let err =
            block_on(convertvars_builtin(vec![pnq(), text("p"), text("string")])).unwrap_err();
        assert_eq!(err.identifier(), Some("RunMat:convertvars:TooManyOutputs"));
    }

    /// U-056-13: type-name table.
    #[test]
    fn documented_choice_type_names() {
        for (name, class) in [
            ("single", "single"),
            ("uint16", "uint16"),
            ("logical", "logical"),
        ] {
            let out = convert(text("n"), text(name));
            assert_eq!(class_of(&variable(&out, "n")), class);
        }
        let out = convert(text("p"), Value::String("string".into()));
        assert_strings(&out, "p", &["a", "b"]);
        for name in ["char", "cell", "cellstr", "datetime", "duration", "table"] {
            assert_eq!(
                err_id("convertvars", &[pnq(), text("n"), text(name)]),
                "RunMat:convertvars:UnsupportedOption"
            );
        }
        assert_eq!(
            err_id("convertvars", &[pnq(), text("n"), text("Double")]),
            "RunMat:table:convertvars:InvalidTextType"
        );
        assert_eq!(
            err_id("convertvars", &[pnq(), text("n"), Value::Num(1.0)]),
            "RunMat:convertvars:InvalidArgument"
        );
    }

    /// U-056-14: failures, function errors, repeated selection, cellstr
    /// selection, properties kept.
    #[test]
    fn documented_choice_failures_and_selection() {
        assert_eq!(
            err_id("convertvars", &[pnq(), text("q"), text("int8")]),
            "RunMat:table:convertvars:VariableTypesConversionFailed"
        );
        // A function that changes the height.
        assert_eq!(
            err_id(
                "convertvars",
                &[pnq(), text("n"), Value::FunctionHandle("sum".into())]
            ),
            "RunMat:convertvars:UnsupportedOption"
        );
        // A function that fails: its own error propagates.
        let err = call(
            "convertvars",
            &[
                pnq(),
                text("p"),
                Value::FunctionHandle("no_such_function_056".into()),
            ],
        )
        .unwrap_err();
        assert_ne!(
            err.identifier(),
            Some("RunMat:table:convertvars:VariableTypesConversionFailed")
        );
        let out = convert(cellstr(&["p", "p"]), text("string"));
        assert_strings(&out, "p", &["a", "b"]);
        let out = convert(row(&[1.0, 3.0]), text("string"));
        assert_strings(&out, "q", &["x", "y"]);
        let t = table_rows(vec![column(&[1.0, 2.0])], &["n"], &["r1", "r2"]);
        let out = ok("convertvars", &[t, text("n"), text("int8")]);
        assert_eq!(property_strings(&out, ROW_NAMES), sv(&["r1", "r2"]));
        assert_eq!(
            err_id("convertvars", &[pnq(), Value::Num(4.0), text("string")]),
            "RunMat:table:VarIndexOutOfRange"
        );
    }
}
