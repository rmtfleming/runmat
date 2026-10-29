//! Clean-room `addvars`, `removevars`, `movevars` (table variable editing).
//!
//! Clean-room feature `specs/034-table-variable-editing/`. Normative behaviour
//! comes only from the approved exports `addvars`, `removevars`, `movevars`
//! 0.2.0 (see that feature's `provenance.md`). Under FR-010 decision D1 = A,
//! every option the exports leave unresolved raises
//! `RunMat:<fn>:UnsupportedOption` instead of guessing.

use super::*;
use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;

use crate::{build_runtime_error, RuntimeError};

// ---------------------------------------------------------------------------
// Descriptors
// ---------------------------------------------------------------------------

const fn param(
    name: &'static str,
    ty: BuiltinParamType,
    description: &'static str,
) -> BuiltinParamDescriptor {
    BuiltinParamDescriptor {
        name,
        ty,
        arity: BuiltinParamArity::Required,
        default: None,
        description,
    }
}

const TABLE_OUTPUT: [BuiltinParamDescriptor; 1] = [param(
    "T2",
    BuiltinParamType::Any,
    "Table with the edited variable list.",
)];

const ADDVARS_INPUTS: [BuiltinParamDescriptor; 2] = [
    param("T", BuiltinParamType::Any, "Input table."),
    param(
        "var",
        BuiltinParamType::Any,
        "Data for the new variable; must have one row per table row.",
    ),
];
const ADDVARS_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "T2 = addvars(T, var)",
    inputs: &ADDVARS_INPUTS,
    outputs: &TABLE_OUTPUT,
}];

const REMOVEVARS_INPUTS: [BuiltinParamDescriptor; 2] = [
    param("T", BuiltinParamType::Any, "Input table."),
    param(
        "vars",
        BuiltinParamType::StringScalar,
        "Name of the variable to remove.",
    ),
];
const REMOVEVARS_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "T2 = removevars(T, vars)",
    inputs: &REMOVEVARS_INPUTS,
    outputs: &TABLE_OUTPUT,
}];

const MOVEVARS_INPUTS: [BuiltinParamDescriptor; 4] = [
    param("T", BuiltinParamType::Any, "Input table."),
    param(
        "vars",
        BuiltinParamType::StringScalar,
        "Name of the variable to move.",
    ),
    param(
        "'Before'",
        BuiltinParamType::StringScalar,
        "Placement keyword.",
    ),
    param(
        "location",
        BuiltinParamType::StringScalar,
        "Name of the variable to place it before.",
    ),
];
const MOVEVARS_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "T2 = movevars(T, vars, 'Before', location)",
    inputs: &MOVEVARS_INPUTS,
    outputs: &TABLE_OUTPUT,
}];

macro_rules! variable_editing_errors {
    ($errors:ident, $prefix:literal, $id:literal) => {
        const $errors: [BuiltinErrorDescriptor; 5] = [
            BuiltinErrorDescriptor {
                code: concat!("RM.", $prefix, ".INVALID_INPUT"),
                identifier: Some(concat!("RunMat:", $id, ":InvalidInput")),
                when: "The first argument is not a table.",
                message: concat!($id, ": expected a table"),
            },
            BuiltinErrorDescriptor {
                code: concat!("RM.", $prefix, ".INVALID_ARGUMENT"),
                identifier: Some(concat!("RunMat:", $id, ":InvalidArgument")),
                when: "An argument has an unrecognized form.",
                message: concat!($id, ": invalid argument"),
            },
            BuiltinErrorDescriptor {
                code: concat!("RM.", $prefix, ".UNSUPPORTED_OPTION"),
                identifier: Some(concat!("RunMat:", $id, ":UnsupportedOption")),
                when: "The call uses a form or option that is not yet specified for RunMat.",
                message: concat!($id, ": option not yet specified"),
            },
            BuiltinErrorDescriptor {
                code: concat!("RM.", $prefix, ".UNKNOWN_VARIABLE"),
                identifier: Some(concat!("RunMat:", $id, ":UnknownVariable")),
                when: "A named variable does not exist in the table.",
                message: concat!($id, ": unknown variable"),
            },
            BuiltinErrorDescriptor {
                code: concat!("RM.", $prefix, ".HEIGHT_MISMATCH"),
                identifier: Some(concat!("RunMat:", $id, ":HeightMismatch")),
                when: "New variable data does not have one row per table row.",
                message: concat!($id, ": variable height does not match the table"),
            },
        ];
    };
}
variable_editing_errors!(ADDVARS_ERRORS, "ADDVARS", "addvars");
variable_editing_errors!(REMOVEVARS_ERRORS, "REMOVEVARS", "removevars");
variable_editing_errors!(MOVEVARS_ERRORS, "MOVEVARS", "movevars");

const INVALID_INPUT: usize = 0;
const INVALID_ARGUMENT: usize = 1;
const UNSUPPORTED_OPTION: usize = 2;
const UNKNOWN_VARIABLE: usize = 3;
const HEIGHT_MISMATCH: usize = 4;

pub const ADDVARS_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &ADDVARS_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ADDVARS_ERRORS,
};
pub const REMOVEVARS_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &REMOVEVARS_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &REMOVEVARS_ERRORS,
};
pub const MOVEVARS_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &MOVEVARS_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &MOVEVARS_ERRORS,
};

// ---------------------------------------------------------------------------
// Shared core
// ---------------------------------------------------------------------------

fn edit_error(
    builtin: &'static str,
    errors: &'static [BuiltinErrorDescriptor; 5],
    kind: usize,
    detail: impl Into<String>,
) -> RuntimeError {
    let descriptor = &errors[kind];
    let mut builder =
        build_runtime_error(format!("{builtin}: {}", detail.into())).with_builtin(builtin);
    if let Some(identifier) = descriptor.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

/// One entry of the edited variable list: an existing variable (by its
/// position in the source table) or a new one.
enum Slot {
    Existing(usize),
    New(String, Value),
}

/// The text of a single name: a char row vector, a string scalar, or a
/// one-element string array (FR-034-09).
fn single_name(value: &Value) -> Option<String> {
    match value {
        Value::CharArray(chars) if chars.rows <= 1 => Some(chars.data.iter().collect()),
        Value::String(text) => Some(text.clone()),
        Value::StringArray(array) if array.data.len() == 1 => Some(array.data[0].clone()),
        _ => None,
    }
}

/// Numeric or logical selectors (index / mask forms; unresolved).
fn is_index_like(value: &Value) -> bool {
    matches!(
        value,
        Value::Num(_)
            | Value::Int(_)
            | Value::Bool(_)
            | Value::Tensor(_)
            | Value::LogicalArray(_)
            | Value::GpuTensor(_)
    )
}

async fn table_input(
    builtin: &'static str,
    errors: &'static [BuiltinErrorDescriptor; 5],
    value: Value,
) -> BuiltinResult<ObjectInstance> {
    let host = gather_if_needed_async(&value)
        .await
        .map_err(map_control_flow)?;
    match host {
        Value::Object(object) if object.class_name.is(TABLE_CLASS) => Ok(object),
        Value::Object(object) if object.class_name.is(TIMETABLE_CLASS) => Err(edit_error(
            builtin,
            errors,
            UNSUPPORTED_OPTION,
            "timetable inputs are not yet specified (U-034-3)",
        )),
        other => Err(edit_error(
            builtin,
            errors,
            INVALID_INPUT,
            format!("expected a table, got {}", value_class_label(&other)),
        )),
    }
}

fn value_class_label(value: &Value) -> String {
    crate::builtins::introspection::class::class_name_for_value(value)
}

fn position_of(
    builtin: &'static str,
    errors: &'static [BuiltinErrorDescriptor; 5],
    names: &[String],
    name: &str,
) -> BuiltinResult<usize> {
    names
        .iter()
        .position(|candidate| candidate == name)
        .ok_or_else(|| {
            edit_error(
                builtin,
                errors,
                UNKNOWN_VARIABLE,
                format!("table has no variable named '{name}'"),
            )
        })
}

/// Rebuild `source` with the variables listed in `slots`. Row names,
/// Description, UserData and DimensionNames are kept; VariableUnits and
/// VariableDescriptions follow their variables (FR-034-10).
fn rebuild(source: ObjectInstance, slots: Vec<Slot>) -> BuiltinResult<Value> {
    let names = table_variable_names_from_object(&source)?;
    let variables = table_variables(&source)?;
    let mut props = table_public_properties(&source)?;
    let per_variable = |field: &str| -> Vec<String> {
        match props.fields.get(field) {
            Some(Value::StringArray(array)) if array.data.len() == names.len() => {
                array.data.clone()
            }
            _ => vec![String::new(); names.len()],
        }
    };
    let units = per_variable(VARIABLE_UNITS);
    let descriptions = per_variable(VARIABLE_DESCRIPTIONS);

    let mut out_vars = StructValue::new();
    let mut out_names = Vec::with_capacity(slots.len());
    let mut out_units = Vec::with_capacity(slots.len());
    let mut out_descriptions = Vec::with_capacity(slots.len());
    for slot in slots {
        match slot {
            Slot::Existing(idx) => {
                let name = names[idx].clone();
                let value =
                    variables.fields.get(&name).cloned().ok_or_else(|| {
                        invalid_variable(format!("table: missing variable '{name}'"))
                    })?;
                out_vars.insert(name.clone(), value);
                out_names.push(name);
                out_units.push(units[idx].clone());
                out_descriptions.push(descriptions[idx].clone());
            }
            Slot::New(name, value) => {
                out_vars.insert(name.clone(), value);
                out_names.push(name);
                out_units.push(String::new());
                out_descriptions.push(String::new());
            }
        }
    }

    let row_list = |values: Vec<String>| -> BuiltinResult<Value> {
        let len = values.len();
        StringArray::new(values, vec![1, len])
            .map(Value::StringArray)
            .map_err(|err| invalid_variable(format!("table: {err}")))
    };
    props.insert(VARIABLE_NAMES, row_list(out_names)?);
    props.insert(VARIABLE_UNITS, row_list(out_units)?);
    props.insert(VARIABLE_DESCRIPTIONS, row_list(out_descriptions)?);

    let mut object = source;
    object
        .properties
        .insert(TABLE_VARIABLES_FIELD.to_string(), Value::Struct(out_vars));
    sync_table_properties(&mut object, props);
    Ok(Value::Object(object))
}

// ---------------------------------------------------------------------------
// Builtins
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "addvars",
    category = "table",
    summary = "Append a variable to a table.",
    keywords = "addvars,table,variables,append,column",
    accel = "cpu",
    descriptor(crate::builtins::table::builtins::variable_editing::ADDVARS_DESCRIPTOR),
    builtin_path = "crate::builtins::table::builtins"
)]
pub(crate) async fn addvars_builtin(
    table: Value,
    var: Value,
    rest: Vec<Value>,
) -> BuiltinResult<Value> {
    const NAME: &str = "addvars";
    let object = table_input(NAME, &ADDVARS_ERRORS, table).await?;
    if !rest.is_empty() {
        // addvars.q-position ('Before'/'After'), addvars.q-names
        // (NewVariableNames) and several data arguments (U-034-1).
        return Err(edit_error(
            NAME,
            &ADDVARS_ERRORS,
            UNSUPPORTED_OPTION,
            "only addvars(T, var) is specified; placement options, NewVariableNames and multiple variables are not yet supported",
        ));
    }
    let var = gather_if_needed_async(&var)
        .await
        .map_err(map_control_flow)?;
    let height = table_height(&object)?;
    let var_height = value_row_count(&var)?;
    if table_width(&object)? > 0 && var_height != height {
        return Err(edit_error(
            NAME,
            &ADDVARS_ERRORS,
            HEIGHT_MISMATCH,
            format!("new variable has {var_height} rows but the table has {height}"),
        ));
    }
    let names = table_variable_names_from_object(&object)?;
    // U-034-4: default name Var<k>, made unique; existing names never change.
    let mut candidates = names.clone();
    candidates.push(format!("Var{}", names.len() + 1));
    let new_name = make_unique_names(candidates)
        .pop()
        .expect("candidate list is non-empty");
    let mut slots: Vec<Slot> = (0..names.len()).map(Slot::Existing).collect();
    slots.push(Slot::New(new_name, var));
    rebuild(object, slots)
}

#[runtime_builtin(
    name = "removevars",
    category = "table",
    summary = "Remove a named variable from a table.",
    keywords = "removevars,table,variables,remove,delete,column",
    accel = "cpu",
    descriptor(crate::builtins::table::builtins::variable_editing::REMOVEVARS_DESCRIPTOR),
    builtin_path = "crate::builtins::table::builtins"
)]
pub(crate) async fn removevars_builtin(
    table: Value,
    vars: Value,
    rest: Vec<Value>,
) -> BuiltinResult<Value> {
    const NAME: &str = "removevars";
    let object = table_input(NAME, &REMOVEVARS_ERRORS, table).await?;
    if !rest.is_empty() {
        return Err(edit_error(
            NAME,
            &REMOVEVARS_ERRORS,
            UNSUPPORTED_OPTION,
            "only removevars(T, vars) is specified",
        ));
    }
    let name = match single_name(&vars) {
        Some(name) => name,
        None if is_index_like(&vars) => {
            // removevars.q-index
            return Err(edit_error(
                NAME,
                &REMOVEVARS_ERRORS,
                UNSUPPORTED_OPTION,
                "removal by numeric index or logical mask is not yet supported",
            ));
        }
        None if matches!(vars, Value::Cell(_) | Value::StringArray(_)) => {
            // removevars.q-multiple
            return Err(edit_error(
                NAME,
                &REMOVEVARS_ERRORS,
                UNSUPPORTED_OPTION,
                "removing several variables at once is not yet supported",
            ));
        }
        None => {
            return Err(edit_error(
                NAME,
                &REMOVEVARS_ERRORS,
                INVALID_ARGUMENT,
                "vars must be a variable name",
            ))
        }
    };
    let names = table_variable_names_from_object(&object)?;
    let target = position_of(NAME, &REMOVEVARS_ERRORS, &names, &name)?;
    let slots = (0..names.len())
        .filter(|idx| *idx != target)
        .map(Slot::Existing)
        .collect();
    rebuild(object, slots)
}

#[runtime_builtin(
    name = "movevars",
    category = "table",
    summary = "Move a table variable before another variable.",
    keywords = "movevars,table,variables,move,reorder,column",
    accel = "cpu",
    descriptor(crate::builtins::table::builtins::variable_editing::MOVEVARS_DESCRIPTOR),
    builtin_path = "crate::builtins::table::builtins"
)]
pub(crate) async fn movevars_builtin(table: Value, rest: Vec<Value>) -> BuiltinResult<Value> {
    const NAME: &str = "movevars";
    let object = table_input(NAME, &MOVEVARS_ERRORS, table).await?;
    let unsupported = |detail: &str| edit_error(NAME, &MOVEVARS_ERRORS, UNSUPPORTED_OPTION, detail);
    let [vars, keyword, location] = rest.as_slice() else {
        return Err(unsupported(
            "only movevars(T, vars, 'Before', location) is specified",
        ));
    };
    match single_name(keyword)
        .map(|k| k.to_ascii_lowercase())
        .as_deref()
    {
        Some("before") => {}
        // movevars.q-after
        Some("after") => return Err(unsupported("the 'After' placement is not yet supported")),
        _ => {
            return Err(edit_error(
                NAME,
                &MOVEVARS_ERRORS,
                INVALID_ARGUMENT,
                "expected the 'Before' keyword",
            ))
        }
    }
    let Some(var_name) = single_name(vars) else {
        // U-034-2 (several variables) and index forms
        return Err(unsupported(
            "moving several variables or variables by index is not yet supported",
        ));
    };
    let Some(location_name) = single_name(location) else {
        // movevars.q-index
        return Err(unsupported("a numeric location is not yet supported"));
    };
    let names = table_variable_names_from_object(&object)?;
    let moving = position_of(NAME, &MOVEVARS_ERRORS, &names, &var_name)?;
    let anchor = position_of(NAME, &MOVEVARS_ERRORS, &names, &location_name)?;
    let mut order: Vec<usize> = (0..names.len()).filter(|idx| *idx != moving).collect();
    let insert_at = order
        .iter()
        .position(|idx| *idx == anchor)
        .unwrap_or(moving); // moving == anchor: put it back where it was
    order.insert(insert_at, moving);
    rebuild(object, order.into_iter().map(Slot::Existing).collect())
}

#[cfg(test)]
mod table_variable_editing_tests {
    use super::*;
    use futures::executor::block_on;
    use runmat_value::{CharArray, StringArray, Tensor};

    fn column(values: &[f64]) -> Value {
        Value::Tensor(Tensor::new(values.to_vec(), vec![values.len(), 1]).unwrap())
    }

    fn row_table(names: &[&str]) -> Value {
        let columns = (0..names.len())
            .map(|idx| Value::Num(idx as f64 + 1.0))
            .collect();
        table_from_columns(names.iter().map(|n| n.to_string()).collect(), columns).unwrap()
    }

    fn text(s: &str) -> Value {
        Value::CharArray(CharArray::new_row(s))
    }

    fn call(name: &str, args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async(name, args))
    }

    fn table_object(value: Value) -> ObjectInstance {
        match value {
            Value::Object(object) if object.class_name.is(TABLE_CLASS) => object,
            other => panic!("expected table object, got {other:?}"),
        }
    }

    fn names(value: &Value) -> Vec<String> {
        table_variable_names_from_object(&table_object(value.clone())).unwrap()
    }

    fn size(value: &Value) -> (usize, usize) {
        let object = table_object(value.clone());
        (
            table_height(&object).unwrap(),
            table_width(&object).unwrap(),
        )
    }

    fn identifier(err: crate::RuntimeError) -> String {
        err.identifier().unwrap_or("<none>").to_string()
    }

    fn string_list_property(value: &Value, field: &str) -> Vec<String> {
        let props = table_public_properties(&table_object(value.clone())).unwrap();
        match props.fields.get(field) {
            Some(Value::StringArray(array)) => array.data.clone(),
            other => panic!("expected string array for {field}, got {other:?}"),
        }
    }

    fn with_units(table: Value, units: &[&str]) -> Value {
        let mut object = table_object(table);
        let mut props = table_public_properties(&object).unwrap();
        props.insert(
            VARIABLE_UNITS,
            Value::StringArray(
                StringArray::new(
                    units.iter().map(|u| u.to_string()).collect(),
                    vec![1, units.len()],
                )
                .unwrap(),
            ),
        );
        props.insert(DESCRIPTION, Value::String("demo".into()));
        sync_table_properties(&mut object, props);
        Value::Object(object)
    }

    // ---- Observed (normative) -------------------------------------------

    /// [addvars.signature-primary, addvars.output-class, addvars.appends-variable]
    /// Case `append`: one-variable 2-row table plus a 2-row column gives a
    /// 2x2 table; the original variable is unchanged.
    #[test]
    fn observed_addvars_append_matches_export() {
        let t = table_from_columns(vec!["a".into()], vec![column(&[1.0, 2.0])]).unwrap();
        let out = call("addvars", &[t, column(&[3.0, 4.0])]).expect("addvars");
        assert_eq!(size(&out), (2, 2));
        let vars = table_variables(&table_object(out.clone())).unwrap();
        let values: Vec<&Value> = vars.fields.values().collect();
        assert_eq!(values[0], &column(&[1.0, 2.0]));
        assert_eq!(values[1], &column(&[3.0, 4.0]));
        assert_eq!(names(&out)[0], "a");
    }

    /// [removevars.signature-primary, removevars.output-class, removevars.removes-variable]
    /// Case `by-name`: removing 'b' from a 1x3 table a,b,c leaves a 1x2 table.
    #[test]
    fn observed_removevars_by_name_matches_export() {
        let out =
            call("removevars", &[row_table(&["a", "b", "c"]), text("b")]).expect("removevars");
        assert_eq!(size(&out), (1, 2));
        assert_eq!(names(&out), ["a", "c"]);
    }

    /// [movevars.signature-primary, movevars.moves-variable]
    /// Cases `before` / `order`: moving 'c' before 'a' gives order c,a,b.
    #[test]
    fn observed_movevars_before_matches_export() {
        let out = call(
            "movevars",
            &[
                row_table(&["a", "b", "c"]),
                text("c"),
                text("Before"),
                text("a"),
            ],
        )
        .expect("movevars");
        assert_eq!(size(&out), (1, 3));
        assert_eq!(names(&out).join(","), "c,a,b");
    }

    // ---- Unresolved dispositions (FR-034-04..06) -------------------------

    #[test]
    fn unresolved_choice_addvars_options_unsupported() {
        let t = || table_from_columns(vec!["a".into()], vec![column(&[1.0, 2.0])]).unwrap();
        let v = || column(&[3.0, 4.0]);
        for extra in [
            vec![text("Before"), text("a")],           // addvars.q-position
            vec![text("After"), Value::Num(1.0)],      // addvars.q-position
            vec![text("NewVariableNames"), text("x")], // addvars.q-names
            vec![column(&[5.0, 6.0])],                 // U-034-1
        ] {
            let mut args = vec![t(), v()];
            args.extend(extra);
            let err = call("addvars", &args).expect_err("unsupported");
            assert_eq!(identifier(err), "RunMat:addvars:UnsupportedOption");
        }
    }

    #[test]
    fn unresolved_choice_removevars_index_and_multiple_unsupported() {
        let selectors = [
            Value::Num(2.0), // removevars.q-index
            Value::LogicalArray(
                runmat_value::LogicalArray::new(vec![0, 1, 0], vec![1, 3]).unwrap(),
            ), // removevars.q-index
            Value::Cell(runmat_value::CellArray::new(vec![text("a"), text("b")], 1, 2).unwrap()), // removevars.q-multiple
            Value::StringArray(StringArray::new(vec!["a".into(), "b".into()], vec![1, 2]).unwrap()), // removevars.q-multiple
        ];
        for selector in selectors {
            let err = call("removevars", &[row_table(&["a", "b", "c"]), selector])
                .expect_err("unsupported");
            assert_eq!(identifier(err), "RunMat:removevars:UnsupportedOption");
        }
    }

    #[test]
    fn unresolved_choice_movevars_after_index_and_lists_unsupported() {
        let cases = [
            vec![text("c"), text("After"), text("a")], // movevars.q-after
            vec![text("c"), text("Before"), Value::Num(1.0)], // movevars.q-index
            vec![
                Value::Cell(
                    runmat_value::CellArray::new(vec![text("b"), text("c")], 1, 2).unwrap(),
                ),
                text("Before"),
                text("a"),
            ], // U-034-2
        ];
        for rest in cases {
            let mut args = vec![row_table(&["a", "b", "c"])];
            args.extend(rest);
            let err = call("movevars", &args).expect_err("unsupported");
            assert_eq!(identifier(err), "RunMat:movevars:UnsupportedOption");
        }
    }

    // ---- Documented choices (FR-034-07..10) ------------------------------

    /// U-034-4: the added variable is named Var<k>, made unique; existing
    /// names never change.
    #[test]
    fn unresolved_choice_addvars_default_name_is_unique_var_k() {
        let t = table_from_columns(vec!["a".into()], vec![column(&[1.0, 2.0])]).unwrap();
        let out = call("addvars", &[t, column(&[3.0, 4.0])]).unwrap();
        assert_eq!(names(&out), ["a", "Var2"]);

        let clash = table_from_columns(vec!["Var2".into()], vec![column(&[1.0, 2.0])]).unwrap();
        let out = call("addvars", &[clash, column(&[3.0, 4.0])]).unwrap();
        let out_names = names(&out);
        assert_eq!(out_names[0], "Var2");
        assert_eq!(out_names.len(), 2);
        assert_ne!(out_names[1], "Var2");
    }

    #[test]
    fn unresolved_choice_metadata_follows_variables() {
        let t = with_units(row_table(&["a", "b", "c"]), &["m", "s", "kg"]);
        let moved = call(
            "movevars",
            &[t.clone(), text("c"), text("before"), text("a")],
        )
        .unwrap();
        assert_eq!(
            string_list_property(&moved, VARIABLE_UNITS),
            ["kg", "m", "s"]
        );
        assert_eq!(
            string_list_property(&moved, VARIABLE_NAMES),
            ["c", "a", "b"]
        );

        let removed = call("removevars", &[t.clone(), text("b")]).unwrap();
        assert_eq!(string_list_property(&removed, VARIABLE_UNITS), ["m", "kg"]);

        let added = call("addvars", &[t, Value::Num(9.0)]).unwrap();
        assert_eq!(
            string_list_property(&added, VARIABLE_UNITS),
            ["m", "s", "kg", ""]
        );
        let props = table_public_properties(&table_object(added)).unwrap();
        assert_eq!(
            props.fields.get(DESCRIPTION),
            Some(&Value::String("demo".into()))
        );
    }

    #[test]
    fn unresolved_choice_move_before_itself_is_unchanged() {
        let out = call(
            "movevars",
            &[
                row_table(&["a", "b", "c"]),
                text("b"),
                text("Before"),
                text("b"),
            ],
        )
        .unwrap();
        assert_eq!(names(&out), ["a", "b", "c"]);
    }

    #[test]
    fn accepts_string_scalar_names() {
        let out = call(
            "removevars",
            &[row_table(&["a", "b", "c"]), Value::String("a".into())],
        )
        .unwrap();
        assert_eq!(names(&out), ["b", "c"]);
        let out = call(
            "movevars",
            &[
                row_table(&["a", "b", "c"]),
                Value::String("a".into()),
                Value::String("Before".into()),
                Value::String("c".into()),
            ],
        )
        .unwrap();
        assert_eq!(names(&out), ["b", "a", "c"]);
    }

    #[test]
    fn rejects_height_mismatch() {
        let t = table_from_columns(vec!["a".into()], vec![column(&[1.0, 2.0])]).unwrap();
        let err = call("addvars", &[t, column(&[1.0, 2.0, 3.0])]).expect_err("height");
        assert_eq!(identifier(err), "RunMat:addvars:HeightMismatch");
    }

    #[test]
    fn rejects_unknown_variables() {
        let err = call("removevars", &[row_table(&["a", "b"]), text("z")]).expect_err("unknown");
        assert_eq!(identifier(err), "RunMat:removevars:UnknownVariable");
        for (var, loc) in [("z", "a"), ("a", "z")] {
            let err = call(
                "movevars",
                &[row_table(&["a", "b"]), text(var), text("Before"), text(loc)],
            )
            .expect_err("unknown");
            assert_eq!(identifier(err), "RunMat:movevars:UnknownVariable");
        }
    }

    #[test]
    fn rejects_non_table_inputs() {
        for (name, args) in [
            ("addvars", vec![Value::Num(1.0), Value::Num(2.0)]),
            ("removevars", vec![Value::Num(1.0), text("a")]),
            (
                "movevars",
                vec![Value::Num(1.0), text("a"), text("Before"), text("b")],
            ),
        ] {
            let err = call(name, &args).expect_err("non-table");
            assert_eq!(identifier(err), format!("RunMat:{name}:InvalidInput"));
        }
    }

    /// U-034-3: timetable inputs are unobserved.
    #[test]
    fn rejects_timetable_inputs_as_unsupported() {
        let tt = table_from_columns_with_class(
            TIMETABLE_CLASS,
            vec!["a".into()],
            vec![column(&[1.0])],
            None,
        )
        .unwrap();
        let err = call("removevars", &[tt, text("a")]).expect_err("timetable");
        assert_eq!(identifier(err), "RunMat:removevars:UnsupportedOption");
    }

    #[test]
    fn rejects_unknown_movevars_keyword_and_arity() {
        let err = call(
            "movevars",
            &[
                row_table(&["a", "b"]),
                text("a"),
                text("Sideways"),
                text("b"),
            ],
        )
        .expect_err("keyword");
        assert_eq!(identifier(err), "RunMat:movevars:InvalidArgument");
        let err = call("movevars", &[row_table(&["a", "b"]), text("a")]).expect_err("arity");
        assert_eq!(identifier(err), "RunMat:movevars:UnsupportedOption");
    }
}
