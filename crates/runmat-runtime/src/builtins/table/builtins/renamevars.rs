//! Clean-room `renamevars` (rename table variables).
//!
//! Clean-room feature `specs/056-table-variables/`. Normative behaviour
//! comes only from the approved export `renamevars` 0.2.0 (see that
//! feature's `provenance.md`). Questions the export leaves open raise
//! `RunMat:renamevars:UnsupportedOption` or follow a documented choice
//! listed in the feature's `spec.md`.

use super::variable_support::*;
use super::*;
use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;

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

const RENAMEVARS_INPUTS: [BuiltinParamDescriptor; 3] = [
    param("T", "Input table."),
    param(
        "vars",
        "Variables: a name, cellstr or string array of names, numeric indices, or a logical mask.",
    ),
    param(
        "newNames",
        "One new name per selected variable: char row, cellstr or string array.",
    ),
];
const RENAMEVARS_OUTPUTS: [BuiltinParamDescriptor; 1] =
    [param("T2", "Table with the selected variables renamed.")];
const RENAMEVARS_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "T2 = renamevars(T, vars, newNames)",
    inputs: &RENAMEVARS_INPUTS,
    outputs: &RENAMEVARS_OUTPUTS,
}];

const RENAMEVARS_ERRORS: [BuiltinErrorDescriptor; 12] = [
    BuiltinErrorDescriptor {
        code: "RM.RENAMEVARS.INVALID_INPUT",
        identifier: Some("RunMat:renamevars:InvalidInput"),
        when: "The first argument is not a table.",
        message: "renamevars: expected a table",
    },
    BuiltinErrorDescriptor {
        code: "RM.RENAMEVARS.INVALID_ARGUMENT",
        identifier: Some("RunMat:renamevars:InvalidArgument"),
        when: "An argument has an unrecognised form, or a variable is selected twice.",
        message: "renamevars: invalid argument",
    },
    BuiltinErrorDescriptor {
        code: "RM.RENAMEVARS.UNSUPPORTED_OPTION",
        identifier: Some("RunMat:renamevars:UnsupportedOption"),
        when: "The call uses a form the approved specification leaves unresolved.",
        message: "renamevars: form not yet specified",
    },
    BuiltinErrorDescriptor {
        code: "RM.RENAMEVARS.TOO_MANY_OUTPUTS",
        identifier: Some("RunMat:renamevars:TooManyOutputs"),
        when: "More than one output was requested.",
        message: "renamevars: too many outputs",
    },
    BuiltinErrorDescriptor {
        code: "RM.RENAMEVARS.INTERNAL",
        identifier: Some("RunMat:renamevars:Internal"),
        when: "RunMat cannot build the result value.",
        message: "renamevars: internal error",
    },
    BuiltinErrorDescriptor {
        code: "RM.RENAMEVARS.UNRECOGNIZED_VAR_NAME",
        identifier: Some("RunMat:table:UnrecognizedVarName"),
        when: "An old name is not a variable of the table.",
        message: "renamevars: unrecognized table variable name",
    },
    BuiltinErrorDescriptor {
        code: "RM.RENAMEVARS.VAR_INDEX_OUT_OF_RANGE",
        identifier: Some("RunMat:table:VarIndexOutOfRange"),
        when: "A numeric index is past the last variable.",
        message: "renamevars: variable index out of range",
    },
    BuiltinErrorDescriptor {
        code: "RM.RENAMEVARS.BAD_SUBSCRIPT",
        identifier: Some("RunMat:badsubscript"),
        when: "A numeric index is zero, negative or not an integer.",
        message: "renamevars: invalid variable index",
    },
    BuiltinErrorDescriptor {
        code: "RM.RENAMEVARS.DUPLICATE_VAR_NAMES",
        identifier: Some("RunMat:table:DuplicateVarNames"),
        when: "A new name equals another variable name.",
        message: "renamevars: duplicate variable name",
    },
    BuiltinErrorDescriptor {
        code: "RM.RENAMEVARS.NUM_NAMES_MISMATCH",
        identifier: Some("RunMat:table:renamevars:NumNamesMismatch"),
        when: "The numbers of old and new names differ.",
        message: "renamevars: numbers of old and new names differ",
    },
    BuiltinErrorDescriptor {
        code: "RM.RENAMEVARS.INVALID_VAR_NAMES",
        identifier: Some("RunMat:table:InvalidVarNames"),
        when: "A new name is empty.",
        message: "renamevars: invalid variable name",
    },
    BuiltinErrorDescriptor {
        code: "RM.RENAMEVARS.RENAME_DIM",
        identifier: Some("RunMat:table:renamevars:RenameDim"),
        when: "The old name is the row dimension name of a table with row names.",
        message: "renamevars: cannot rename a dimension",
    },
];

const NUM_NAMES_MISMATCH: &str = "NUM_NAMES_MISMATCH";
const DUPLICATE_VAR_NAMES: &str = "DUPLICATE_VAR_NAMES";
const INVALID_VAR_NAMES: &str = "INVALID_VAR_NAMES";
const RENAME_DIM: &str = "RENAME_DIM";

pub const RENAMEVARS_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &RENAMEVARS_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &RENAMEVARS_ERRORS,
};

const CTX: Ctx = Ctx {
    name: "renamevars",
    errors: &RENAMEVARS_ERRORS,
};

// ---------------------------------------------------------------------------
// Builtin
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "renamevars",
    category = "table",
    summary = "Rename selected variables of a table.",
    keywords = "renamevars,table,variables,rename,names",
    accel = "cpu",
    descriptor(crate::builtins::table::builtins::renamevars::RENAMEVARS_DESCRIPTOR),
    builtin_path = "crate::builtins::table::builtins"
)]
pub(crate) async fn renamevars_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    CTX.check_outputs()?;
    let [table, vars, new_names] = args.as_slice() else {
        return Err(CTX.error(INVALID_ARGUMENT, "expected renamevars(T, vars, newNames)"));
    };
    let object = CTX.table_input(table).await?;
    let (names, columns) = columns_of(&object)?;

    let vars = gathered(vars).await?;
    let selected = match CTX.select(&vars, &names).await {
        Ok(selected) => selected,
        Err(err) => {
            // renamevars.error-dim-Row: the row dimension name is not a variable.
            if err.identifier() == Some("RunMat:table:UnrecognizedVarName") {
                if let Some(name) = single_name(&vars) {
                    if is_row_dimension(&object, &name)? {
                        return Err(CTX.error(
                            RENAME_DIM,
                            format!("'{name}' names the row dimension, not a variable"),
                        ));
                    }
                }
            }
            return Err(err);
        }
    };
    let mut seen = vec![false; names.len()];
    for &position in &selected {
        if std::mem::replace(&mut seen[position], true) {
            return Err(CTX.error(
                INVALID_ARGUMENT,
                format!("variable '{}' is selected more than once", names[position]),
            ));
        }
    }

    let new_names = gathered(new_names).await?;
    let given = name_list(&new_names).ok_or_else(|| {
        CTX.error(
            INVALID_ARGUMENT,
            "new names must be text, a string array or a cellstr",
        )
    })?;
    if given.len() != selected.len() {
        return Err(CTX.error(
            NUM_NAMES_MISMATCH,
            format!(
                "{} new names given for {} selected variables",
                given.len(),
                selected.len()
            ),
        ));
    }
    if given.iter().any(|name| name.trim().is_empty()) {
        return Err(CTX.error(INVALID_VAR_NAMES, "a new variable name is empty"));
    }

    let mut renamed = names.clone();
    for (&position, name) in selected.iter().zip(given) {
        renamed[position] = name;
    }
    for (idx, name) in renamed.iter().enumerate() {
        if renamed[..idx].contains(name) {
            return Err(CTX.error(
                DUPLICATE_VAR_NAMES,
                format!("duplicate variable name '{name}'"),
            ));
        }
    }
    with_variables(object, renamed, columns)
}

/// renamevars.error-dim-Row / U-056-4: `name` is the row dimension of a
/// table that has row names.
fn is_row_dimension(object: &ObjectInstance, name: &str) -> BuiltinResult<bool> {
    let props = table_public_properties(object)?;
    let has_row_names = match props.fields.get(ROW_NAMES) {
        Some(Value::StringArray(array)) => !array.data.is_empty(),
        Some(Value::Cell(cell)) => !cell.data.is_empty(),
        _ => false,
    };
    if !has_row_names {
        return Ok(false);
    }
    let dimension = match props.fields.get(DIMENSION_NAMES) {
        Some(Value::StringArray(array)) => array.data.first().cloned(),
        Some(Value::Cell(cell)) => cell.data.first().and_then(single_name),
        _ => None,
    };
    Ok(name == "Row" || dimension.as_deref() == Some(name))
}

#[cfg(test)]
mod renamevars_tests {
    use super::super::variable_support::test_helpers::*;
    use super::*;
    use futures::executor::block_on;

    const DUPLICATE: &str = "RunMat:table:DuplicateVarNames";
    const UNRECOGNIZED: &str = "RunMat:table:UnrecognizedVarName";
    const MISMATCH: &str = "RunMat:table:renamevars:NumNamesMismatch";
    const INVALID_NAMES: &str = "RunMat:table:InvalidVarNames";
    const OUT_OF_RANGE: &str = "RunMat:table:VarIndexOutOfRange";
    const RENAME_DIM_ID: &str = "RunMat:table:renamevars:RenameDim";

    /// The observed `table([1;2],[3;4],[5;6],'VariableNames',{'a','b','c'})`.
    fn abc() -> Value {
        table(
            vec![
                column(&[1.0, 2.0]),
                column(&[3.0, 4.0]),
                column(&[5.0, 6.0]),
            ],
            &["a", "b", "c"],
        )
    }

    fn rename(vars: Value, new_names: Value) -> Value {
        ok("renamevars", &[abc(), vars, new_names])
    }

    /// Names in order, and the data still in place.
    fn assert_layout(result: &Value, expected: &[&str]) {
        assert_eq!(names(result), sv(expected));
        assert_eq!(size(result), (2, 3));
        let data = [[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]];
        for (name, values) in expected.iter().zip(data) {
            assert_eq!(
                doubles(&variable(result, name)),
                (vec![2, 1], values.to_vec())
            );
        }
    }

    // ---- observed (normative) ---------------------------------------------

    /// renamevars.signature-primary, renamevars.selection,
    /// renamevars.keeps-data (case `one`).
    #[test]
    fn observed_one_by_name() {
        assert_layout(&rename(text("b"), text("B")), &["a", "B", "c"]);
    }

    /// renamevars.selection, renamevars.keeps-data (case `many`).
    #[test]
    fn observed_many_by_cellstr() {
        let out = rename(cellstr(&["a", "c"]), cellstr(&["x", "z"]));
        assert_layout(&out, &["x", "b", "z"]);
    }

    /// renamevars.selection (cases `index`, `index-many`).
    #[test]
    fn observed_by_index_and_index_vector() {
        assert_layout(&rename(Value::Num(2.0), text("B")), &["a", "B", "c"]);
        let out = rename(row(&[3.0, 1.0]), cellstr(&["Z", "X"]));
        assert_layout(&out, &["X", "b", "Z"]);
    }

    /// renamevars.selection (case `logical`).
    #[test]
    fn observed_by_logical_mask() {
        let out = rename(mask(&[false, true, false]), text("B"));
        assert_layout(&out, &["a", "B", "c"]);
    }

    /// renamevars.selection (case `string`).
    #[test]
    fn observed_by_string_arrays() {
        let out = rename(strings(&["a", "b"]), strings(&["p", "q"]));
        assert_layout(&out, &["p", "q", "c"]);
    }

    /// renamevars.same-name (case `same-name`).
    #[test]
    fn observed_same_name_unchanged() {
        assert_layout(&rename(text("a"), text("a")), &["a", "b", "c"]);
    }

    /// renamevars.swap (case `swap`).
    #[test]
    fn observed_swap() {
        let out = rename(cellstr(&["a", "b"]), cellstr(&["b", "a"]));
        assert_layout(&out, &["b", "a", "c"]);
    }

    /// renamevars.keeps-data (case `rownames`).
    #[test]
    fn observed_row_names_kept() {
        let t = table_rows(vec![column(&[1.0, 2.0])], &["a"], &["r1", "r2"]);
        let out = ok("renamevars", &[t, text("a"), text("b")]);
        assert_eq!(names(&out), sv(&["b"]));
        assert_eq!(doubles(&variable(&out, "b")), (vec![2, 1], vec![1.0, 2.0]));
        assert_eq!(property_strings(&out, ROW_NAMES), sv(&["r1", "r2"]));
    }

    /// renamevars.non-identifier (case `non-identifier`).
    #[test]
    fn observed_non_identifier_name_accepted() {
        assert_layout(&rename(text("a"), text("my var")), &["my var", "b", "c"]);
    }

    /// renamevars.error-duplicate (case `err-duplicate`).
    #[test]
    fn observed_error_duplicate() {
        assert_eq!(
            err_id("renamevars", &[abc(), text("a"), text("b")]),
            DUPLICATE
        );
    }

    /// renamevars.error-unknown (case `err-unknown`).
    #[test]
    fn observed_error_unknown() {
        assert_eq!(
            err_id("renamevars", &[abc(), text("nope"), text("x")]),
            UNRECOGNIZED
        );
    }

    /// renamevars.error-count (case `err-count`).
    #[test]
    fn observed_error_count() {
        assert_eq!(
            err_id(
                "renamevars",
                &[abc(), cellstr(&["a", "b"]), cellstr(&["x"])]
            ),
            MISMATCH
        );
    }

    /// renamevars.error-empty-name (case `err-empty-name`).
    #[test]
    fn observed_error_empty_name() {
        assert_eq!(
            err_id("renamevars", &[abc(), text("a"), text("")]),
            INVALID_NAMES
        );
    }

    /// renamevars.error-index-range (case `err-index-range`).
    #[test]
    fn observed_error_index_range() {
        assert_eq!(
            err_id("renamevars", &[abc(), Value::Num(4.0), text("x")]),
            OUT_OF_RANGE
        );
    }

    /// renamevars.error-dim-Row (case `err-dim-Row`).
    #[test]
    fn observed_error_rename_row_dimension() {
        let t = table_rows(vec![column(&[1.0, 2.0])], &["a"], &["r1", "r2"]);
        assert_eq!(
            err_id("renamevars", &[t, text("Row"), text("ID")]),
            RENAME_DIM_ID
        );
    }

    // ---- documented choices (FR-056-40..44) --------------------------------

    /// U-056-1 (renamevars.non-table is non-normative), arity.
    #[test]
    fn rejects_non_table_and_bad_arity() {
        let s = ok("struct", &[text("a"), Value::Num(1.0)]);
        assert_eq!(
            err_id("renamevars", &[s, text("a"), text("b")]),
            "RunMat:renamevars:InvalidInput"
        );
        assert_eq!(
            err_id("renamevars", &[abc(), text("a")]),
            "RunMat:renamevars:InvalidArgument"
        );
        assert_eq!(
            err_id("renamevars", &[abc(), text("a"), text("b"), text("c")]),
            "RunMat:renamevars:InvalidArgument"
        );
    }

    /// U-056-1: one output (called directly; the dispatcher retries).
    #[test]
    fn unresolved_too_many_outputs() {
        let _guard = crate::output_count::push_output_count(Some(2));
        let err = block_on(renamevars_builtin(vec![abc(), text("a"), text("x")])).unwrap_err();
        assert_eq!(err.identifier(), Some("RunMat:renamevars:TooManyOutputs"));
    }

    /// U-056-4 (renamevars.q-row-variable): a variable named `Row` is
    /// renamed; `'Row'` without row names is an unknown name.
    #[test]
    fn documented_choice_row_variable_and_row_without_row_names() {
        let t = table(vec![column(&[1.0, 2.0])], &["Row"]);
        let out = ok("renamevars", &[t, text("Row"), text("ID")]);
        assert_eq!(names(&out), sv(&["ID"]));
        assert_eq!(
            err_id("renamevars", &[abc(), text("Row"), text("ID")]),
            UNRECOGNIZED
        );
    }

    /// U-056-5: equal new names, repeated selection, blank names, empty
    /// selection, units follow their variables.
    #[test]
    fn documented_choice_name_rules() {
        assert_eq!(
            err_id(
                "renamevars",
                &[abc(), cellstr(&["a", "b"]), cellstr(&["z", "z"])]
            ),
            DUPLICATE
        );
        assert_eq!(
            err_id(
                "renamevars",
                &[abc(), cellstr(&["a", "a"]), cellstr(&["x", "y"])]
            ),
            "RunMat:renamevars:InvalidArgument"
        );
        assert_eq!(
            err_id("renamevars", &[abc(), text("a"), text("   ")]),
            INVALID_NAMES
        );
        let empty = Value::Cell(runmat_value::CellArray::new(Vec::new(), 0, 0).unwrap());
        let out = ok("renamevars", &[abc(), empty.clone(), empty]);
        assert_layout(&out, &["a", "b", "c"]);
    }

    /// U-056-3: selector validation as F34.
    #[test]
    fn documented_choice_selector_validation() {
        assert_eq!(
            err_id("renamevars", &[abc(), Value::Num(0.0), text("x")]),
            "RunMat:badsubscript"
        );
        assert_eq!(
            err_id("renamevars", &[abc(), mask(&[true, false]), text("x")]),
            "RunMat:renamevars:UnsupportedOption"
        );
        assert_eq!(
            err_id("renamevars", &[abc(), text("a"), Value::Num(3.0)]),
            "RunMat:renamevars:InvalidArgument"
        );
    }

    /// U-056-5: other properties (units) stay with their variables.
    #[test]
    fn documented_choice_properties_kept() {
        let t = ok(
            "renamevars",
            &[abc(), cellstr(&["a", "b"]), cellstr(&["b", "a"])],
        );
        let mut object = object(&t);
        let mut props = table_public_properties(&object).unwrap();
        props.insert(
            VARIABLE_UNITS,
            Value::StringArray(
                runmat_value::StringArray::new(sv(&["m", "s", "kg"]), vec![1, 3]).unwrap(),
            ),
        );
        sync_table_properties(&mut object, props);
        let out = ok("renamevars", &[Value::Object(object), text("c"), text("w")]);
        assert_eq!(names(&out), sv(&["b", "a", "w"]));
        assert_eq!(
            property_strings(&out, VARIABLE_UNITS),
            sv(&["m", "s", "kg"])
        );
    }
}
