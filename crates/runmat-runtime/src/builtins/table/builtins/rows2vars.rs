//! Clean-room `rows2vars` (transpose a table).
//!
//! Clean-room feature `specs/056-table-variables/`. Normative behaviour
//! comes only from the approved export `rows2vars` 0.2.0 (see that
//! feature's `provenance.md`). Questions the export leaves open raise
//! `RunMat:rows2vars:UnsupportedOption` or follow a documented choice
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

const fn param(
    name: &'static str,
    ty: BuiltinParamType,
    arity: BuiltinParamArity,
    description: &'static str,
) -> BuiltinParamDescriptor {
    BuiltinParamDescriptor {
        name,
        ty,
        arity,
        default: None,
        description,
    }
}

const TABLE_INPUT: BuiltinParamDescriptor = param(
    "T",
    BuiltinParamType::Any,
    BuiltinParamArity::Required,
    "Input table; every variable must have one column.",
);
const ROWS2VARS_INPUTS: [BuiltinParamDescriptor; 1] = [TABLE_INPUT];
const ROWS2VARS_INPUTS_NV: [BuiltinParamDescriptor; 2] = [
    TABLE_INPUT,
    param(
        "Name, Value",
        BuiltinParamType::Any,
        BuiltinParamArity::Variadic,
        "'VariableNamesSource' (a variable name or index) and 'VariableNamingRule' ('modify' or 'preserve').",
    ),
];
const ROWS2VARS_OUTPUTS: [BuiltinParamDescriptor; 1] = [param(
    "T2",
    BuiltinParamType::Any,
    BuiltinParamArity::Required,
    "Transposed table starting with OriginalVariableNames.",
)];
const ROWS2VARS_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "T2 = rows2vars(T)",
        inputs: &ROWS2VARS_INPUTS,
        outputs: &ROWS2VARS_OUTPUTS,
    },
    BuiltinSignatureDescriptor {
        label: "T2 = rows2vars(T, Name, Value)",
        inputs: &ROWS2VARS_INPUTS_NV,
        outputs: &ROWS2VARS_OUTPUTS,
    },
];

const ROWS2VARS_ERRORS: [BuiltinErrorDescriptor; 10] = [
    BuiltinErrorDescriptor {
        code: "RM.ROWS2VARS.INVALID_INPUT",
        identifier: Some("RunMat:rows2vars:InvalidInput"),
        when: "The first argument is not a table.",
        message: "rows2vars: expected a table",
    },
    BuiltinErrorDescriptor {
        code: "RM.ROWS2VARS.INVALID_ARGUMENT",
        identifier: Some("RunMat:rows2vars:InvalidArgument"),
        when: "An option is dangling, repeated or malformed.",
        message: "rows2vars: invalid argument",
    },
    BuiltinErrorDescriptor {
        code: "RM.ROWS2VARS.UNSUPPORTED_OPTION",
        identifier: Some("RunMat:rows2vars:UnsupportedOption"),
        when: "The call uses a form the approved specification leaves unresolved.",
        message: "rows2vars: form not yet specified",
    },
    BuiltinErrorDescriptor {
        code: "RM.ROWS2VARS.TOO_MANY_OUTPUTS",
        identifier: Some("RunMat:rows2vars:TooManyOutputs"),
        when: "More than one output was requested.",
        message: "rows2vars: too many outputs",
    },
    BuiltinErrorDescriptor {
        code: "RM.ROWS2VARS.INTERNAL",
        identifier: Some("RunMat:rows2vars:Internal"),
        when: "RunMat cannot build the result value.",
        message: "rows2vars: internal error",
    },
    BuiltinErrorDescriptor {
        code: "RM.ROWS2VARS.UNRECOGNIZED_VAR_NAME",
        identifier: Some("RunMat:table:UnrecognizedVarName"),
        when: "VariableNamesSource names an absent variable.",
        message: "rows2vars: unrecognized table variable name",
    },
    BuiltinErrorDescriptor {
        code: "RM.ROWS2VARS.VAR_INDEX_OUT_OF_RANGE",
        identifier: Some("RunMat:table:VarIndexOutOfRange"),
        when: "VariableNamesSource is an index past the last variable.",
        message: "rows2vars: variable index out of range",
    },
    BuiltinErrorDescriptor {
        code: "RM.ROWS2VARS.BAD_SUBSCRIPT",
        identifier: Some("RunMat:badsubscript"),
        when: "VariableNamesSource is an index that is not a positive integer.",
        message: "rows2vars: invalid variable index",
    },
    BuiltinErrorDescriptor {
        code: "RM.ROWS2VARS.MULTICOLUMN",
        identifier: Some("RunMat:table:rows2vars:CannotTransposeMulticolumnVar"),
        when: "A variable has more than one column.",
        message: "rows2vars: cannot transpose a multi-column variable",
    },
    BuiltinErrorDescriptor {
        code: "RM.ROWS2VARS.BAD_RULE",
        identifier: Some("RunMat:rows2vars:unrecognizedStringChoice"),
        when: "VariableNamingRule has an unrecognised value.",
        message: "rows2vars: unrecognized VariableNamingRule",
    },
];

const MULTICOLUMN: &str = "MULTICOLUMN";
const BAD_RULE: &str = "BAD_RULE";

/// Warning issued when names were modified (FR-056-13).
const MODIFIED_WARNING: &str = "RunMat:table:ModifiedVarnamesRows2Vars";

pub const ROWS2VARS_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &ROWS2VARS_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ROWS2VARS_ERRORS,
};

const CTX: Ctx = Ctx {
    name: "rows2vars",
    errors: &ROWS2VARS_ERRORS,
};

// ---------------------------------------------------------------------------
// Builtin
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "rows2vars",
    category = "table",
    summary = "Transpose a table so that rows become variables.",
    keywords = "rows2vars,table,transpose,reorient,variables",
    accel = "cpu",
    descriptor(crate::builtins::table::builtins::rows2vars::ROWS2VARS_DESCRIPTOR),
    builtin_path = "crate::builtins::table::builtins"
)]
pub(crate) async fn rows2vars_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    CTX.check_outputs()?;
    let Some((table, rest)) = args.split_first() else {
        return Err(CTX.error(INVALID_ARGUMENT, "expected rows2vars(T, ...)"));
    };
    let object = CTX.table_input(table).await?;
    let options = parse_options(rest).await?;
    let (names, columns) = columns_of(&object)?;
    let height = table_height(&object)?;

    // FR-056-12: the names source, left out of the transposed variables.
    let source = match &options.source {
        Some(value) => Some(source_position(value, &names)?),
        None => None,
    };
    let transposed: Vec<usize> = (0..names.len()).filter(|i| Some(*i) != source).collect();

    // FR-056-17 first, then FR-056-45.
    for &idx in &transposed {
        if column_count(&columns[idx]) != 1 {
            return Err(CTX.error(
                MULTICOLUMN,
                format!("variable '{}' has more than one column", names[idx]),
            ));
        }
    }
    let kind = result_kind(&transposed, &columns, &names)?;

    // New variable names (FR-056-11, FR-056-12).
    let raw_names: Vec<String> = match source {
        Some(idx) => {
            if transposed.is_empty() && height > 0 {
                return Err(CTX.error(
                    UNSUPPORTED_OPTION,
                    "a table whose only variable is the names source is not yet specified",
                ));
            }
            source_names(&columns[idx], &names[idx])?
        }
        None => {
            let row_names = row_names_of(&object)?;
            if row_names.len() == height && height > 0 {
                row_names
            } else {
                (1..=height).map(|i| format!("Var{i}")).collect()
            }
        }
    };
    if raw_names.iter().any(|name| name.trim().is_empty()) {
        return Err(CTX.error(
            UNSUPPORTED_OPTION,
            "an empty new variable name is not yet specified",
        ));
    }
    let mut modified = false;
    let valid_names: Vec<String> = raw_names
        .into_iter()
        .map(|name| {
            if options.preserve {
                return name;
            }
            let valid = make_valid(&name);
            modified |= valid != name;
            valid
        })
        .collect();
    if modified {
        crate::warnings::emit(crate::warnings::WarningRequest {
            builtin: "rows2vars",
            identifier: MODIFIED_WARNING,
            message: "rows2vars: variable names were modified to make them valid identifiers",
            show_identifier: false,
        })?;
    }
    let mut out_names = vec![ORIGINAL_NAMES.to_string()];
    for name in valid_names {
        let unique = unique_name(&name, &out_names);
        out_names.push(unique);
    }

    // OriginalVariableNames, then one variable per input row.
    let original = transposed
        .iter()
        .map(|idx| Value::CharArray(CharArray::new_row(&names[*idx])))
        .collect::<Vec<_>>();
    let count = transposed.len();
    let mut out_columns = vec![Value::Cell(
        CellArray::new(original, count, 1).map_err(|err| CTX.error(INTERNAL, err))?,
    )];
    for row in 0..height {
        let cells = transposed
            .iter()
            .map(|idx| element(&columns[*idx], row))
            .collect::<BuiltinResult<Vec<_>>>()?;
        out_columns.push(assemble(kind, cells, count)?);
    }
    table_from_columns(out_names, out_columns)
}

const ORIGINAL_NAMES: &str = "OriginalVariableNames";

#[derive(Default)]
struct Options {
    source: Option<Value>,
    preserve: bool,
}

/// FR-056-48: case-insensitive name-value options.
async fn parse_options(rest: &[Value]) -> BuiltinResult<Options> {
    if !rest.len().is_multiple_of(2) {
        return Err(CTX.error(INVALID_ARGUMENT, "options must be name-value pairs"));
    }
    let mut options = Options::default();
    let mut seen_source = false;
    let mut seen_rule = false;
    for pair in rest.chunks(2) {
        let key = single_name(&gathered(&pair[0]).await?)
            .ok_or_else(|| CTX.error(INVALID_ARGUMENT, "option names must be text"))?;
        let value = gathered(&pair[1]).await?;
        if key.eq_ignore_ascii_case("VariableNamesSource") {
            if std::mem::replace(&mut seen_source, true) {
                return Err(CTX.error(INVALID_ARGUMENT, "'VariableNamesSource' is repeated"));
            }
            options.source = Some(value);
        } else if key.eq_ignore_ascii_case("VariableNamingRule") {
            if std::mem::replace(&mut seen_rule, true) {
                return Err(CTX.error(INVALID_ARGUMENT, "'VariableNamingRule' is repeated"));
            }
            let rule = single_name(&value).unwrap_or_default();
            options.preserve = if rule.eq_ignore_ascii_case("preserve") {
                true
            } else if rule.eq_ignore_ascii_case("modify") {
                false
            } else {
                return Err(CTX.error(
                    BAD_RULE,
                    "'VariableNamingRule' must be 'modify' or 'preserve'",
                ));
            };
        } else {
            return Err(CTX.error(
                UNSUPPORTED_OPTION,
                format!("option '{key}' is not yet specified"),
            ));
        }
    }
    Ok(options)
}

/// FR-056-12 / FR-056-48: one name or one positive integer index.
fn source_position(value: &Value, names: &[String]) -> BuiltinResult<usize> {
    if let Some(name) = single_name(value) {
        return CTX.position_of(names, &name);
    }
    match numeric_values(value) {
        Some(values) if values.len() == 1 => CTX.index_position(values[0], names.len()),
        _ => Err(CTX.error(
            UNSUPPORTED_OPTION,
            "'VariableNamesSource' must be one variable name or index",
        )),
    }
}

/// Names held by the source variable: a cellstr or string column.
fn source_names(value: &Value, name: &str) -> BuiltinResult<Vec<String>> {
    let names = match value {
        Value::Cell(cell) if cell.shape.len() == 2 && cell.shape[1] == 1 => cell
            .data
            .iter()
            .map(single_name)
            .collect::<Option<Vec<_>>>(),
        Value::StringArray(array) if array.shape.len() == 2 && array.shape[1] == 1 => {
            Some(array.data.clone())
        }
        _ => None,
    };
    names.ok_or_else(|| {
        CTX.error(
            UNSUPPORTED_OPTION,
            format!("a names source '{name}' that is not a cellstr or string column is not yet specified"),
        )
    })
}

fn row_names_of(object: &ObjectInstance) -> BuiltinResult<Vec<String>> {
    let props = table_public_properties(object)?;
    Ok(match props.fields.get(ROW_NAMES) {
        Some(Value::StringArray(array)) => array.data.clone(),
        Some(Value::Cell(cell)) => cell.data.iter().filter_map(single_name).collect(),
        _ => Vec::new(),
    })
}

/// Number of columns of a variable (all dimensions after the first).
fn column_count(value: &Value) -> usize {
    let shape: Vec<usize> = match value {
        Value::Tensor(tensor) => tensor.shape.clone(),
        Value::ComplexTensor(tensor) => tensor.shape.clone(),
        Value::LogicalArray(array) => array.shape.clone(),
        Value::StringArray(array) => array.shape.clone(),
        Value::Cell(cell) => cell.shape.clone(),
        Value::CharArray(chars) => vec![chars.rows, chars.cols],
        _ => vec![1, 1],
    };
    shape.iter().skip(1).product()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Double,
    Logical,
    Text,
    Cell,
}

/// FR-056-15 / FR-056-45: the class of the new variables.
fn result_kind(transposed: &[usize], columns: &[Value], names: &[String]) -> BuiltinResult<Kind> {
    let mut kinds = Vec::new();
    for &idx in transposed {
        let kind = match &columns[idx] {
            Value::Num(_) => Kind::Double,
            Value::Tensor(_) if class_label(&columns[idx]) == "double" => Kind::Double,
            Value::Bool(_) | Value::LogicalArray(_) => Kind::Logical,
            Value::StringArray(_) | Value::String(_) => Kind::Text,
            Value::Cell(_) => Kind::Cell,
            other => {
                return Err(CTX.error(
                    UNSUPPORTED_OPTION,
                    format!(
                        "transposing variable '{}' of class {} is not yet specified",
                        names[idx],
                        class_label(other)
                    ),
                ))
            }
        };
        if !kinds.contains(&kind) {
            kinds.push(kind);
        }
    }
    match kinds.as_slice() {
        [] => Ok(Kind::Double),
        [kind] => Ok(*kind),
        [_, _] if kinds.contains(&Kind::Double) && kinds.contains(&Kind::Cell) => Ok(Kind::Cell),
        _ => Err(CTX.error(
            UNSUPPORTED_OPTION,
            "transposing this mix of variable classes is not yet specified",
        )),
    }
}

/// Element `row` of a single-column variable, as a scalar value (a cell
/// variable gives its content).
fn element(value: &Value, row: usize) -> BuiltinResult<Value> {
    match value {
        Value::Num(n) => Ok(Value::Num(*n)),
        Value::Bool(b) => Ok(Value::Bool(*b)),
        Value::String(text) => Ok(Value::String(text.clone())),
        Value::Tensor(tensor) => Ok(Value::Num(
            crate::builtins::common::tensor::tensor_value_f64(tensor, row),
        )),
        Value::LogicalArray(array) => Ok(Value::Bool(array.data.as_slice()[row] != 0)),
        Value::StringArray(array) => Ok(Value::String(array.data[row].clone())),
        Value::Cell(cell) => Ok(cell.data[row].clone()),
        other => Err(CTX.error(
            INTERNAL,
            format!("unexpected variable value {}", class_label(other)),
        )),
    }
}

/// An m-by-1 column of `kind` from scalar elements.
fn assemble(kind: Kind, cells: Vec<Value>, count: usize) -> BuiltinResult<Value> {
    let internal = |err: String| CTX.error(INTERNAL, err);
    match kind {
        Kind::Double => {
            let data = cells
                .iter()
                .map(|v| match v {
                    Value::Num(n) => *n,
                    _ => f64::NAN,
                })
                .collect();
            Tensor::new(data, vec![count, 1])
                .map(Value::Tensor)
                .map_err(|e| internal(e.to_string()))
        }
        Kind::Logical => {
            let data = cells
                .iter()
                .map(|v| u8::from(matches!(v, Value::Bool(true))))
                .collect();
            LogicalArray::new(data, vec![count, 1])
                .map(Value::LogicalArray)
                .map_err(|e| internal(e.to_string()))
        }
        Kind::Text => {
            let data = cells
                .into_iter()
                .map(|v| match v {
                    Value::String(text) => text,
                    _ => String::new(),
                })
                .collect();
            StringArray::new(data, vec![count, 1])
                .map(Value::StringArray)
                .map_err(|e| internal(e.to_string()))
        }
        Kind::Cell => CellArray::new(cells, count, 1)
            .map(Value::Cell)
            .map_err(|e| internal(e.to_string())),
    }
}

/// FR-056-46 (documented choice, U-056-7).
fn make_valid(raw: &str) -> String {
    let mut out = String::new();
    let mut after_space = false;
    for ch in raw.trim().chars() {
        if ch.is_whitespace() {
            after_space = true;
            continue;
        }
        let ch = if after_space {
            ch.to_ascii_uppercase()
        } else {
            ch
        };
        after_space = false;
        out.push(if ch.is_ascii_alphanumeric() || ch == '_' {
            ch
        } else {
            '_'
        });
    }
    if !out.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) {
        out.insert(0, 'x');
    }
    out
}

/// FR-056-14 / FR-056-47: `name`, or `name_1`, `name_2`, … if taken.
fn unique_name(name: &str, taken: &[String]) -> String {
    if !taken.iter().any(|t| t == name) {
        return name.to_string();
    }
    (1..)
        .map(|k| format!("{name}_{k}"))
        .find(|candidate| !taken.iter().any(|t| t == candidate))
        .expect("an unused suffix exists")
}

#[cfg(test)]
mod rows2vars_tests {
    use super::super::variable_support::test_helpers::*;
    use super::*;
    use futures::executor::block_on;

    const OVN: &str = "OriginalVariableNames";

    /// `table([1;2],[3;4],'VariableNames',{'x','y'})`.
    fn xy() -> Value {
        table(vec![column(&[1.0, 2.0]), column(&[3.0, 4.0])], &["x", "y"])
    }

    /// `table({'k1';'k2'},[1;2],[3;4],'VariableNames',{'id','x','y'})`.
    fn keyed(keys: &[&str]) -> Value {
        table(
            vec![cellcol(keys), column(&[1.0, 2.0]), column(&[3.0, 4.0])],
            &["id", "x", "y"],
        )
    }

    /// `table({'a b';'1c'},[1;2],'VariableNames',{'id','x'})`.
    fn odd_names(keys: &[&str]) -> Value {
        table(vec![cellcol(keys), column(&[1.0, 2.0])], &["id", "x"])
    }

    fn assert_ovn(out: &Value, expected: &[&str]) {
        let value = variable(out, OVN);
        let Value::Cell(cell) = &value else {
            panic!("OriginalVariableNames must be a cell, got {value:?}");
        };
        assert_eq!(cell.shape, vec![expected.len(), 1]);
        assert_eq!(cell_texts(&value), sv(expected));
    }

    fn assert_column(out: &Value, name: &str, values: &[f64]) {
        assert_eq!(class_of(&variable(out, name)), "double");
        assert_eq!(
            doubles(&variable(out, name)),
            (vec![values.len(), 1], values.to_vec())
        );
    }

    fn warning_ids() -> Vec<String> {
        crate::warning_store::take_all()
            .into_iter()
            .map(|w| w.identifier)
            .collect()
    }

    // ---- observed (normative) ---------------------------------------------

    /// rows2vars.signature-primary, rows2vars.layout,
    /// rows2vars.default-names (case `basic`).
    #[test]
    fn observed_basic() {
        let out = ok("rows2vars", &[xy()]);
        assert_eq!(names(&out), sv(&[OVN, "Var1", "Var2"]));
        assert_eq!(size(&out), (2, 3));
        assert_ovn(&out, &["x", "y"]);
        assert_column(&out, "Var1", &[1.0, 3.0]);
        assert_column(&out, "Var2", &[2.0, 4.0]);
    }

    /// rows2vars.default-names (case `rownames`).
    #[test]
    fn observed_row_names_name_variables() {
        let t = table_rows(
            vec![column(&[1.0, 2.0]), column(&[3.0, 4.0])],
            &["x", "y"],
            &["r1", "r2"],
        );
        let out = ok("rows2vars", &[t]);
        assert_eq!(names(&out), sv(&[OVN, "r1", "r2"]));
        assert_ovn(&out, &["x", "y"]);
        assert_column(&out, "r1", &[1.0, 3.0]);
        assert_column(&out, "r2", &[2.0, 4.0]);
    }

    /// rows2vars.default-names (case `one-row`).
    #[test]
    fn observed_one_row() {
        let t = table(vec![Value::Num(1.0), Value::Num(2.0)], &["x", "y"]);
        let out = ok("rows2vars", &[t]);
        assert_eq!(names(&out), sv(&[OVN, "Var1"]));
        assert_eq!(size(&out), (2, 2));
        assert_column(&out, "Var1", &[1.0, 2.0]);
    }

    /// rows2vars.names-source (cases `source-name`, `source-index`,
    /// `source-string`).
    #[test]
    fn observed_names_source_forms() {
        let forms = [
            (text("VariableNamesSource"), text("id")),
            (text("VariableNamesSource"), Value::Num(1.0)),
            (
                Value::String("VariableNamesSource".into()),
                Value::String("id".into()),
            ),
        ];
        for (key, value) in forms {
            let out = ok("rows2vars", &[keyed(&["k1", "k2"]), key, value]);
            assert_eq!(names(&out), sv(&[OVN, "k1", "k2"]));
            assert_ovn(&out, &["x", "y"]);
            assert_column(&out, "k1", &[1.0, 3.0]);
            assert_column(&out, "k2", &[2.0, 4.0]);
        }
    }

    /// rows2vars.naming-modify, rows2vars.warning-warn-naming-default
    /// (cases `naming-default`, `warn-naming-default`).
    #[test]
    fn observed_naming_default_modifies_and_warns() {
        crate::warning_store::reset();
        let out = ok(
            "rows2vars",
            &[
                odd_names(&["a b", "1c"]),
                text("VariableNamesSource"),
                text("id"),
            ],
        );
        assert_eq!(names(&out), sv(&[OVN, "aB", "x1c"]));
        assert_eq!(size(&out), (1, 3));
        assert_ovn(&out, &["x"]);
        assert_column(&out, "aB", &[1.0]);
        assert_column(&out, "x1c", &[2.0]);
        assert_eq!(
            warning_ids(),
            sv(&["RunMat:table:ModifiedVarnamesRows2Vars"])
        );
    }

    /// rows2vars.naming-modify (case `naming-modify`).
    #[test]
    fn observed_naming_modify() {
        let out = ok(
            "rows2vars",
            &[
                odd_names(&["a b", "1c"]),
                text("VariableNamesSource"),
                text("id"),
                text("VariableNamingRule"),
                text("modify"),
            ],
        );
        assert_eq!(names(&out), sv(&[OVN, "aB", "x1c"]));
    }

    /// rows2vars.naming-preserve (case `naming-preserve`).
    #[test]
    fn observed_naming_preserve() {
        crate::warning_store::reset();
        let out = ok(
            "rows2vars",
            &[
                odd_names(&["a b", "1c"]),
                text("VariableNamesSource"),
                text("id"),
                text("VariableNamingRule"),
                text("preserve"),
            ],
        );
        assert_eq!(names(&out), sv(&[OVN, "a b", "1c"]));
        assert_column(&out, "a b", &[1.0]);
        assert_column(&out, "1c", &[2.0]);
        assert!(warning_ids().is_empty());
    }

    /// rows2vars.duplicate-source (case `dup-source`).
    #[test]
    fn observed_duplicate_source_made_unique() {
        let out = ok(
            "rows2vars",
            &[
                odd_names(&["k", "k"]),
                text("VariableNamesSource"),
                text("id"),
            ],
        );
        assert_eq!(names(&out), sv(&[OVN, "k", "k_1"]));
        assert_column(&out, "k", &[1.0]);
        assert_column(&out, "k_1", &[2.0]);
    }

    /// rows2vars.mixed (case `mixed`).
    #[test]
    fn observed_mixed_classes_give_cells() {
        let t = table(vec![column(&[1.0, 2.0]), cellcol(&["p", "q"])], &["n", "s"]);
        let out = ok("rows2vars", &[t]);
        assert_eq!(names(&out), sv(&[OVN, "Var1", "Var2"]));
        assert_ovn(&out, &["n", "s"]);
        for (name, number, word) in [("Var1", 1.0, "p"), ("Var2", 2.0, "q")] {
            let Value::Cell(cell) = variable(&out, name) else {
                panic!("{name} must be a cell");
            };
            assert_eq!(cell.shape, vec![2, 1]);
            assert_eq!(doubles(&cell.data[0]).1, vec![number]);
            assert_eq!(single_name(&cell.data[1]).as_deref(), Some(word));
            assert!(matches!(cell.data[1], Value::CharArray(_)));
        }
    }

    /// rows2vars.empty-rows (case `empty-rows`).
    #[test]
    fn observed_empty_rows() {
        let empty = Value::Tensor(runmat_value::Tensor::new(Vec::new(), vec![0, 1]).unwrap());
        let t = table(vec![empty.clone(), empty], &["x", "y"]);
        let out = ok("rows2vars", &[t]);
        assert_eq!(names(&out), sv(&[OVN]));
        assert_eq!(size(&out), (2, 1));
        assert_ovn(&out, &["x", "y"]);
    }

    /// rows2vars.error-multicol (case `err-multicol`).
    #[test]
    fn observed_error_multicolumn() {
        let m =
            Value::Tensor(runmat_value::Tensor::new(vec![1.0, 3.0, 2.0, 4.0], vec![2, 2]).unwrap());
        let t = table(vec![m], &["m"]);
        assert_eq!(
            err_id("rows2vars", &[t]),
            "RunMat:table:rows2vars:CannotTransposeMulticolumnVar"
        );
    }

    /// rows2vars.error-unknown-source (case `err-unknown-source`).
    #[test]
    fn observed_error_unknown_source() {
        assert_eq!(
            err_id(
                "rows2vars",
                &[
                    keyed(&["k1", "k2"]),
                    text("VariableNamesSource"),
                    text("nope")
                ]
            ),
            "RunMat:table:UnrecognizedVarName"
        );
    }

    /// rows2vars.error-bad-rule (case `err-bad-rule`).
    #[test]
    fn observed_error_bad_rule() {
        assert_eq!(
            err_id(
                "rows2vars",
                &[
                    keyed(&["k1", "k2"]),
                    text("VariableNamesSource"),
                    text("id"),
                    text("VariableNamingRule"),
                    text("nope"),
                ]
            ),
            "RunMat:rows2vars:unrecognizedStringChoice"
        );
    }

    // ---- documented choices (FR-056-40, 45..48) ----------------------------

    /// U-056-1 (rows2vars.non-table is non-normative).
    #[test]
    fn rejects_non_table() {
        let m =
            Value::Tensor(runmat_value::Tensor::new(vec![1.0, 3.0, 2.0, 4.0], vec![2, 2]).unwrap());
        assert_eq!(err_id("rows2vars", &[m]), "RunMat:rows2vars:InvalidInput");
        assert_eq!(err_id("rows2vars", &[]), "RunMat:rows2vars:InvalidArgument");
    }

    /// U-056-1: one output (called directly; the dispatcher retries).
    #[test]
    fn unresolved_too_many_outputs() {
        let _guard = crate::output_count::push_output_count(Some(2));
        let err = block_on(rows2vars_builtin(vec![xy()])).unwrap_err();
        assert_eq!(err.identifier(), Some("RunMat:rows2vars:TooManyOutputs"));
    }

    /// U-056-6: homogeneous classes keep their class; other mixes unsupported.
    #[test]
    fn documented_choice_classes() {
        let col = |a: &str, b: &str| {
            Value::StringArray(runmat_value::StringArray::new(sv(&[a, b]), vec![2, 1]).unwrap())
        };
        let out = ok(
            "rows2vars",
            &[table(vec![col("a", "b"), col("c", "d")], &["s", "t"])],
        );
        assert_eq!(class_of(&variable(&out, "Var1")), "string");
        assert_eq!(string_texts(&variable(&out, "Var1")), sv(&["a", "c"]));
        let out = ok(
            "rows2vars",
            &[table(
                vec![cellcol(&["a", "b"]), cellcol(&["c", "d"])],
                &["s", "t"],
            )],
        );
        assert_eq!(cell_texts(&variable(&out, "Var2")), sv(&["b", "d"]));
        let logical = |bits: &[bool]| {
            Value::LogicalArray(
                runmat_value::LogicalArray::new(
                    bits.iter().map(|b| *b as u8).collect(),
                    vec![bits.len(), 1],
                )
                .unwrap(),
            )
        };
        let out = ok(
            "rows2vars",
            &[table(
                vec![logical(&[true, false]), logical(&[false, false])],
                &["p", "q"],
            )],
        );
        assert_eq!(class_of(&variable(&out, "Var1")), "logical");
        let mixed = table(
            vec![column(&[1.0, 2.0]), logical(&[true, false])],
            &["n", "b"],
        );
        assert_eq!(
            err_id("rows2vars", &[mixed]),
            "RunMat:rows2vars:UnsupportedOption"
        );
        let ints = ok("int8", &[column(&[1.0, 2.0])]);
        let t = table(vec![ints], &["i"]);
        assert_eq!(
            err_id("rows2vars", &[t]),
            "RunMat:rows2vars:UnsupportedOption"
        );
    }

    /// U-056-7: the name rule on row names and other characters; the
    /// warning under 'modify'; empty names unsupported.
    #[test]
    fn documented_choice_name_rule() {
        crate::warning_store::reset();
        let out = ok(
            "rows2vars",
            &[
                odd_names(&[" lead", "a-b c"]),
                text("VariableNamesSource"),
                text("id"),
                text("variablenamingrule"),
                text("MODIFY"),
            ],
        );
        assert_eq!(names(&out), sv(&[OVN, "lead", "a_bC"]));
        assert_eq!(
            warning_ids(),
            sv(&["RunMat:table:ModifiedVarnamesRows2Vars"])
        );
        let t = table_rows(vec![column(&[1.0, 2.0])], &["x"], &["r 1", "ok"]);
        let out = ok("rows2vars", &[t]);
        assert_eq!(names(&out), sv(&[OVN, "r1", "ok"]));
        crate::warning_store::reset();
        assert_eq!(
            err_id(
                "rows2vars",
                &[
                    odd_names(&["", "b"]),
                    text("VariableNamesSource"),
                    text("id")
                ]
            ),
            "RunMat:rows2vars:UnsupportedOption"
        );
        let _ = warning_ids();
    }

    /// U-056-8: clash with OriginalVariableNames; no row names on output.
    #[test]
    fn documented_choice_uniqueness_and_row_names() {
        let out = ok(
            "rows2vars",
            &[
                odd_names(&[OVN, "z"]),
                text("VariableNamesSource"),
                text("id"),
            ],
        );
        assert_eq!(names(&out), sv(&[OVN, "OriginalVariableNames_1", "z"]));
        assert!(property_strings(&out, ROW_NAMES).is_empty());
    }

    /// U-056-9: option validation and source validation.
    #[test]
    fn documented_choice_options() {
        let t = keyed(&["k1", "k2"]);
        let unsupported = "RunMat:rows2vars:UnsupportedOption";
        let invalid = "RunMat:rows2vars:InvalidArgument";
        assert_eq!(
            err_id("rows2vars", &[t.clone(), text("DataVariables"), text("x")]),
            unsupported
        );
        assert_eq!(
            err_id("rows2vars", &[t.clone(), text("VariableNamesSource")]),
            invalid
        );
        assert_eq!(
            err_id(
                "rows2vars",
                &[
                    t.clone(),
                    text("VariableNamesSource"),
                    text("id"),
                    text("VariableNamesSource"),
                    text("id"),
                ]
            ),
            invalid
        );
        assert_eq!(
            err_id(
                "rows2vars",
                &[t.clone(), text("VariableNamesSource"), Value::Num(4.0)]
            ),
            "RunMat:table:VarIndexOutOfRange"
        );
        assert_eq!(
            err_id(
                "rows2vars",
                &[t.clone(), text("VariableNamesSource"), Value::Num(1.5)]
            ),
            "RunMat:badsubscript"
        );
        // A numeric source variable is not yet specified.
        assert_eq!(
            err_id("rows2vars", &[t, text("VariableNamesSource"), text("x")]),
            unsupported
        );
        // The source as the only variable, with rows.
        let only = table(vec![cellcol(&["a", "b"])], &["id"]);
        assert_eq!(
            err_id(
                "rows2vars",
                &[only, text("VariableNamesSource"), text("id")]
            ),
            unsupported
        );
    }
}
