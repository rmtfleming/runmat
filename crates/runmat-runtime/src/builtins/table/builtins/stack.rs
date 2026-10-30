//! Clean-room `stack` (wide-to-long table reshape).
//!
//! Clean-room feature `specs/044-table-stack-innerjoin/`. Normative behaviour
//! comes only from the approved export `stack` 0.2.0 (see that feature's
//! `provenance.md`).

use super::*;
use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{IntegerStorage, NumericStorage};

use crate::{build_runtime_error, RuntimeError};

const NAME: &str = "stack";

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

const T_INPUT: BuiltinParamDescriptor = param("T", BuiltinParamType::Any, "Table.");
const VARS_INPUT: BuiltinParamDescriptor = param(
    "vars",
    BuiltinParamType::Any,
    "Variables to stack: names (char, string or cellstr) or indices.",
);

const STACK_OUTPUT: [BuiltinParamDescriptor; 1] =
    [param("S", BuiltinParamType::Any, "Stacked table.")];

const BASIC_INPUTS: [BuiltinParamDescriptor; 2] = [T_INPUT, VARS_INPUT];
const OPTION_INPUTS: [BuiltinParamDescriptor; 4] = [
    T_INPUT,
    VARS_INPUT,
    param(
        "Name",
        BuiltinParamType::StringScalar,
        "'NewDataVariableName', 'IndexVariableName' or 'ConstantVariables'.",
    ),
    param("Value", BuiltinParamType::Any, "Option value."),
];

const STACK_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "S = stack(T, vars)",
        inputs: &BASIC_INPUTS,
        outputs: &STACK_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "S = stack(T, vars, Name, Value)",
        inputs: &OPTION_INPUTS,
        outputs: &STACK_OUTPUT,
    },
];

const STACK_ERRORS: [BuiltinErrorDescriptor; 5] = [
    BuiltinErrorDescriptor {
        code: "RM.STACK.INVALID_INPUT",
        identifier: Some("RunMat:stack:InvalidInput"),
        when: "The first input is not a table.",
        message: "stack: expected a table",
    },
    BuiltinErrorDescriptor {
        code: "RM.STACK.INVALID_ARGUMENT",
        identifier: Some("RunMat:stack:InvalidArgument"),
        when: "A variable list, option or output name is malformed.",
        message: "stack: invalid argument",
    },
    BuiltinErrorDescriptor {
        code: "RM.STACK.UNSUPPORTED_OPTION",
        identifier: Some("RunMat:stack:UnsupportedOption"),
        when: "The call uses a form, option or data that is not yet specified for RunMat.",
        message: "stack: option not yet specified",
    },
    BuiltinErrorDescriptor {
        code: "RM.STACK.UNRECOGNIZED_VAR_NAME",
        identifier: Some("RunMat:table:UnrecognizedVarName"),
        when: "A variable name is not in the table.",
        message: "stack: unrecognized variable name",
    },
    BuiltinErrorDescriptor {
        code: "RM.STACK.INTERLEAVING_FAILED",
        identifier: Some("RunMat:table:stack:InterleavingDataVarsFailed"),
        when: "The stacked variables are a double and a cell variable.",
        message: "stack: cannot interleave the data variables",
    },
];

const INVALID_INPUT: usize = 0;
const INVALID_ARGUMENT: usize = 1;
const UNSUPPORTED_OPTION: usize = 2;
const UNRECOGNIZED_VAR_NAME: usize = 3;
const INTERLEAVING_FAILED: usize = 4;

pub const STACK_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &STACK_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &STACK_ERRORS,
};

fn stack_error(kind: usize, detail: impl Into<String>) -> RuntimeError {
    let mut builder = build_runtime_error(format!("{NAME}: {}", detail.into())).with_builtin(NAME);
    if let Some(identifier) = STACK_ERRORS[kind].identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

fn unsupported(detail: impl Into<String>) -> RuntimeError {
    stack_error(UNSUPPORTED_OPTION, detail)
}

fn invalid_argument(detail: impl Into<String>) -> RuntimeError {
    stack_error(INVALID_ARGUMENT, detail)
}

fn class_label(value: &Value) -> String {
    crate::builtins::introspection::class::class_name_for_value(value)
}

// ---------------------------------------------------------------------------
// Variable lists and options
// ---------------------------------------------------------------------------

/// The text of a single name: a char row, a string scalar, or a
/// one-element string array.
fn single_name(value: &Value) -> Option<String> {
    match value {
        Value::CharArray(chars) if chars.rows <= 1 => Some(chars.data.iter().collect()),
        Value::String(text) => Some(text.clone()),
        Value::StringArray(array) if array.data.len() == 1 => Some(array.data[0].clone()),
        _ => None,
    }
}

enum Selector {
    Names(Vec<String>),
    Indices(Vec<usize>),
}

/// Parse a variable list (FR-044-04, FR-044-21, FR-044-30).
fn selector(what: &str, value: &Value) -> BuiltinResult<Selector> {
    if let Some(name) = single_name(value) {
        return Ok(Selector::Names(vec![name]));
    }
    match value {
        Value::Cell(cell) => cell
            .data
            .iter()
            .map(|item| match item {
                // stack.q-groups
                Value::Cell(_) => Err(unsupported(
                    "stacking several groups of variables is not yet supported",
                )),
                other => single_name(other)
                    .ok_or_else(|| invalid_argument(format!("{what} must list variable names"))),
            })
            .collect::<BuiltinResult<_>>()
            .map(Selector::Names),
        Value::StringArray(array) => Ok(Selector::Names(array.data.clone())),
        Value::Bool(_) | Value::LogicalArray(_) => Err(unsupported(format!(
            "a logical {what} selector is not yet supported"
        ))),
        Value::Num(_) | Value::Int(_) | Value::Tensor(_) => {
            let numbers = match value {
                Value::Num(number) => vec![*number],
                Value::Int(int) => vec![int.to_f64()],
                Value::Tensor(tensor) => tensor.materialize_f64(),
                _ => unreachable!("numeric selector"),
            };
            numbers
                .into_iter()
                .map(|number| {
                    if number >= 1.0 && number.fract() == 0.0 && number.is_finite() {
                        Ok(number as usize - 1)
                    } else {
                        Err(invalid_argument(format!(
                            "{what} indices must be positive integers"
                        )))
                    }
                })
                .collect::<BuiltinResult<_>>()
                .map(Selector::Indices)
        }
        _ => Err(invalid_argument(format!(
            "{what} must be variable names or indices"
        ))),
    }
}

/// Positions of a parsed list in the table, in listing order.
fn resolve(what: &str, selector: &Selector, names: &[String]) -> BuiltinResult<Vec<usize>> {
    let positions: Vec<usize> = match selector {
        Selector::Names(list) => list
            .iter()
            .map(|wanted| {
                names.iter().position(|name| name == wanted).ok_or_else(|| {
                    stack_error(
                        UNRECOGNIZED_VAR_NAME,
                        format!("table has no variable named '{wanted}'"),
                    )
                })
            })
            .collect::<BuiltinResult<_>>()?,
        Selector::Indices(indices) => indices
            .iter()
            .map(|&index| {
                if index < names.len() {
                    Ok(index)
                } else {
                    Err(invalid_argument(format!(
                        "{what} index {} exceeds the number of table variables",
                        index + 1
                    )))
                }
            })
            .collect::<BuiltinResult<_>>()?,
    };
    let mut seen = HashSet::new();
    if positions.iter().any(|position| !seen.insert(*position)) {
        return Err(invalid_argument(format!(
            "{what} lists a variable more than once"
        )));
    }
    Ok(positions)
}

struct Options {
    data_name: Option<String>,
    index_name: Option<String>,
    constant: Option<Value>,
}

fn parse_options(rest: Vec<Value>) -> BuiltinResult<Options> {
    if !rest.len().is_multiple_of(2) {
        return Err(invalid_argument("name-value arguments must come in pairs"));
    }
    let mut options = Options {
        data_name: None,
        index_name: None,
        constant: None,
    };
    let mut args = rest.into_iter();
    while let (Some(name), Some(value)) = (args.next(), args.next()) {
        let Some(option) = single_name(&name) else {
            return Err(invalid_argument("expected an option name"));
        };
        let text = |label: &str| {
            single_name(&value)
                .filter(|text| !text.is_empty())
                .ok_or_else(|| invalid_argument(format!("'{label}' must be a nonempty name")))
        };
        match option.to_ascii_lowercase().as_str() {
            "newdatavariablename" => options.data_name = Some(text("NewDataVariableName")?),
            "indexvariablename" => options.index_name = Some(text("IndexVariableName")?),
            "constantvariables" => options.constant = Some(value),
            _ => return Err(invalid_argument(format!("unknown option '{option}'"))),
        }
    }
    Ok(options)
}

// ---------------------------------------------------------------------------
// Interleaving
// ---------------------------------------------------------------------------

/// Scalars become one-element arrays so every variable has a row layout.
fn normalize_variable(value: Value) -> BuiltinResult<Value> {
    Ok(match value {
        Value::Num(number) => {
            Value::Tensor(Tensor::new(vec![number], vec![1, 1]).map_err(invalid_variable)?)
        }
        Value::Bool(flag) => Value::LogicalArray(
            LogicalArray::new(vec![u8::from(flag)], vec![1, 1]).map_err(invalid_variable)?,
        ),
        Value::Int(int) => Value::Tensor(
            Tensor::new_integer(IntegerStorage::from_scalar(int), vec![1, 1])
                .map_err(invalid_variable)?,
        ),
        Value::String(text) => {
            Value::StringArray(StringArray::new(vec![text], vec![1, 1]).map_err(invalid_variable)?)
        }
        other => other,
    })
}

fn two_d(shape: &[usize]) -> Option<(usize, usize)> {
    match shape {
        [] => Some((1, 1)),
        [rows] => Some((*rows, 1)),
        [rows, cols] => Some((*rows, *cols)),
        _ => None,
    }
}

/// Output row `i * k + v` is row `i` of source `v`; column-major data.
fn interleave<T: Clone>(sources: &[&[T]], rows: usize, cols: usize) -> Vec<T> {
    let mut out = Vec::with_capacity(sources.len() * rows * cols);
    for col in 0..cols {
        for row in 0..rows {
            for source in sources {
                out.push(source[row + col * rows].clone());
            }
        }
    }
    out
}

fn interleave_numeric(tensors: &[&Tensor], rows: usize, cols: usize) -> Option<NumericStorage> {
    let storages: Vec<NumericStorage> = tensors
        .iter()
        .map(|tensor| (*tensor).clone().into_numeric_storage().ok())
        .collect::<Option<_>>()?;
    macro_rules! variant {
        ($($v:ident),*) => {
            match &storages[0] {
                $(NumericStorage::$v(_) => {
                    let slices: Vec<&[_]> = storages
                        .iter()
                        .map(|storage| match storage {
                            NumericStorage::$v(data) => Some(data.as_slice()),
                            _ => None,
                        })
                        .collect::<Option<_>>()?;
                    NumericStorage::$v(interleave(&slices, rows, cols))
                })*
            }
        };
    }
    Some(variant!(F64, F32, I8, I16, I32, I64, U8, U16, U32, U64))
}

/// Build the stacked data variable (FR-044-01, FR-044-07, FR-044-28).
fn stack_data(names: &[&str], columns: &[&Value], rows: usize) -> BuiltinResult<Value> {
    let classes: Vec<String> = columns.iter().map(|value| class_label(value)).collect();
    if classes.iter().any(|class| *class != classes[0]) {
        if classes.iter().any(|class| class == "double")
            && classes.iter().any(|class| class == "cell")
        {
            return Err(stack_error(
                INTERLEAVING_FAILED,
                "the stacked variables cannot be interleaved (double and cell)",
            ));
        }
        return Err(unsupported(format!(
            "stacking variables of different classes ({}) is not yet supported",
            classes.join(", ")
        )));
    }
    let not_supported = || {
        unsupported(format!(
            "stacking variables of class {} is not yet supported",
            classes[0]
        ))
    };
    let shapes: Vec<(usize, usize)> = columns
        .iter()
        .map(|value| match value {
            Value::Tensor(t) => two_d(&t.shape),
            Value::LogicalArray(a) => two_d(&a.shape),
            Value::StringArray(a) => two_d(&a.shape),
            Value::Cell(c) => Some((c.rows, c.cols)),
            _ => None,
        })
        .collect::<Option<_>>()
        .ok_or_else(not_supported)?;
    let cols = shapes[0].1;
    if shapes.iter().any(|&(r, c)| r != rows || c != cols) {
        return Err(unsupported(format!(
            "stacking variables {} with different numbers of columns is not yet supported",
            names.join(", ")
        )));
    }
    let shape = vec![rows * columns.len(), cols];
    match columns[0] {
        Value::Tensor(_) => {
            let tensors: Vec<&Tensor> = columns
                .iter()
                .map(|value| match value {
                    Value::Tensor(t) => t,
                    _ => unreachable!("same class"),
                })
                .collect();
            let storage = interleave_numeric(&tensors, rows, cols).ok_or_else(not_supported)?;
            Tensor::from_numeric_storage(storage, shape)
                .map(Value::Tensor)
                .map_err(invalid_variable)
        }
        Value::LogicalArray(_) => {
            let slices: Vec<&[u8]> = columns
                .iter()
                .map(|value| match value {
                    Value::LogicalArray(a) => a.data.as_slice(),
                    _ => unreachable!("same class"),
                })
                .collect();
            LogicalArray::new(interleave(&slices, rows, cols), shape)
                .map(Value::LogicalArray)
                .map_err(invalid_variable)
        }
        Value::StringArray(_) => {
            let slices: Vec<&[String]> = columns
                .iter()
                .map(|value| match value {
                    Value::StringArray(a) => a.data.as_slice(),
                    _ => unreachable!("same class"),
                })
                .collect();
            StringArray::new(interleave(&slices, rows, cols), shape)
                .map(Value::StringArray)
                .map_err(invalid_variable)
        }
        Value::Cell(_) => {
            // Cell data is row-major; interleave per (row, column).
            let mut data = Vec::with_capacity(rows * columns.len() * cols);
            for row in 0..rows {
                for value in columns {
                    let Value::Cell(cell) = value else {
                        unreachable!("same class")
                    };
                    for col in 0..cols {
                        data.push(cell.get(row, col).map_err(invalid_index)?);
                    }
                }
            }
            CellArray::new(data, rows * columns.len(), cols)
                .map(Value::Cell)
                .map_err(invalid_variable)
        }
        _ => Err(not_supported()),
    }
}

/// The categorical indicator (FR-044-03, FR-044-25).
fn indicator(stacked: &[String], rows: usize) -> BuiltinResult<Value> {
    let k = stacked.len();
    let codes: Vec<f64> = (0..rows * k).map(|r| (r % k + 1) as f64).collect();
    let mut object = ObjectInstance::new(CATEGORICAL_CLASS.to_string());
    object.properties.insert(
        "Codes".to_string(),
        Value::Tensor(Tensor::new(codes, vec![rows * k, 1]).map_err(invalid_variable)?),
    );
    object.properties.insert(
        "Categories".to_string(),
        Value::StringArray(
            StringArray::new(stacked.to_vec(), vec![1, k]).map_err(invalid_variable)?,
        ),
    );
    object
        .properties
        .insert("Ordinal".to_string(), Value::Bool(false));
    object
        .properties
        .insert("Protected".to_string(), Value::Bool(false));
    Ok(Value::Object(object))
}

// ---------------------------------------------------------------------------
// Builtin
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "stack",
    category = "table",
    summary = "Stack table variables into one variable (wide to long) with an indicator variable.",
    keywords = "stack,table,reshape,wide to long,unpivot,indicator",
    accel = "cpu",
    descriptor(crate::builtins::table::builtins::stack::STACK_DESCRIPTOR),
    builtin_path = "crate::builtins::table::builtins"
)]
pub(crate) async fn stack_builtin(table: Value, rest: Vec<Value>) -> BuiltinResult<Value> {
    let table = match gather_if_needed_async(&table)
        .await
        .map_err(map_control_flow)?
    {
        Value::Object(object) if object.class_name.is(TABLE_CLASS) => object,
        Value::Object(object) if object.class_name.is(TIMETABLE_CLASS) => {
            return Err(unsupported("timetable inputs are not yet supported"))
        }
        other => {
            return Err(stack_error(
                INVALID_INPUT,
                format!("first input must be a table, got {}", class_label(&other)),
            ))
        }
    };
    // stack.q-second-output
    if crate::output_count::current_output_count().is_some_and(|count| count > 1) {
        return Err(unsupported(
            "the row-index output [S, iT] is not yet supported",
        ));
    }
    let mut host_rest = Vec::with_capacity(rest.len());
    for value in rest {
        host_rest.push(
            gather_if_needed_async(&value)
                .await
                .map_err(map_control_flow)?,
        );
    }
    let mut host_rest = host_rest.into_iter();
    let vars = host_rest
        .next()
        .ok_or_else(|| invalid_argument("expected the variables to stack"))?;
    let options = parse_options(host_rest.collect())?;

    let rows = table_height(&table)?;
    if rows > 0 && selected_row_names(&table, &[0])?.is_some() {
        return Err(unsupported("tables with row names are not yet supported"));
    }
    let variables = table_variables(&table)?;
    let mut names = Vec::with_capacity(variables.fields.len());
    let mut columns = Vec::with_capacity(variables.fields.len());
    for (name, value) in variables.fields {
        let host = gather_if_needed_async(&value)
            .await
            .map_err(map_control_flow)?;
        names.push(name);
        columns.push(normalize_variable(host)?);
    }

    let stacked = resolve("vars", &selector("vars", &vars)?, &names)?;
    if stacked.is_empty() {
        return Err(invalid_argument("vars must name at least one variable"));
    }
    let carried: Vec<usize> = match &options.constant {
        None => (0..names.len()).filter(|i| !stacked.contains(i)).collect(),
        Some(value) => {
            let list = match value {
                Value::Cell(cell) if cell.data.is_empty() => Vec::new(),
                other => resolve(
                    "ConstantVariables",
                    &selector("ConstantVariables", other)?,
                    &names,
                )?,
            };
            if list.iter().any(|i| stacked.contains(i)) {
                return Err(invalid_argument(
                    "ConstantVariables must not list a stacked variable",
                ));
            }
            let mut list = list;
            list.sort_unstable();
            list
        }
    };

    let stacked_names: Vec<String> = stacked.iter().map(|&i| names[i].clone()).collect();
    let joined = stacked_names.join("_");
    let index_name = options
        .index_name
        .unwrap_or_else(|| format!("{joined}_Indicator"));
    let data_name = options.data_name.unwrap_or(joined);

    let k = stacked.len();
    let repeat_rows: Vec<usize> = (0..rows).flat_map(|i| std::iter::repeat_n(i, k)).collect();
    let mut out_names = Vec::with_capacity(carried.len() + 2);
    let mut out_columns = Vec::with_capacity(carried.len() + 2);
    for &i in &carried {
        out_names.push(names[i].clone());
        out_columns.push(select_rows(&columns[i], &repeat_rows)?);
    }
    out_names.push(index_name);
    out_columns.push(indicator(&stacked_names, rows)?);
    out_names.push(data_name);
    let refs: Vec<&str> = stacked_names.iter().map(String::as_str).collect();
    let data_columns: Vec<&Value> = stacked.iter().map(|&i| &columns[i]).collect();
    out_columns.push(stack_data(&refs, &data_columns, rows)?);

    let mut seen = HashSet::new();
    if let Some(duplicate) = out_names.iter().find(|name| !seen.insert(name.as_str())) {
        return Err(invalid_argument(format!(
            "output variable name '{duplicate}' is used more than once"
        )));
    }
    table_from_columns(out_names, out_columns)
}

#[cfg(test)]
mod table_stack_tests {
    use super::*;
    use futures::executor::block_on;
    use runmat_value::{CellArray, CharArray, IntegerStorage, LogicalArray, StringArray, Tensor};

    // ---- Builders ---------------------------------------------------------

    fn col(values: &[f64]) -> Value {
        Value::Tensor(Tensor::new(values.to_vec(), vec![values.len(), 1]).unwrap())
    }

    fn int32_col(values: &[i32]) -> Value {
        Value::Tensor(
            Tensor::new_integer(IntegerStorage::I32(values.to_vec()), vec![values.len(), 1])
                .unwrap(),
        )
    }

    fn logical_col(values: &[bool]) -> Value {
        Value::LogicalArray(
            LogicalArray::new(
                values.iter().map(|b| u8::from(*b)).collect(),
                vec![values.len(), 1],
            )
            .unwrap(),
        )
    }

    fn cellstr_col(values: &[&str]) -> Value {
        Value::Cell(
            CellArray::new(values.iter().map(|s| text(s)).collect(), values.len(), 1).unwrap(),
        )
    }

    fn string_col(values: &[&str]) -> Value {
        Value::StringArray(
            StringArray::new(
                values.iter().map(|s| s.to_string()).collect(),
                vec![values.len(), 1],
            )
            .unwrap(),
        )
    }

    fn text(s: &str) -> Value {
        Value::CharArray(CharArray::new_row(s))
    }

    fn cellstr(values: &[&str]) -> Value {
        Value::Cell(
            CellArray::new(values.iter().map(|s| text(s)).collect(), 1, values.len()).unwrap(),
        )
    }

    fn empty_cell() -> Value {
        Value::Cell(CellArray::new(Vec::new(), 0, 0).unwrap())
    }

    fn row(values: &[f64]) -> Value {
        Value::Tensor(Tensor::new(values.to_vec(), vec![1, values.len()]).unwrap())
    }

    fn table(names: &[&str], columns: Vec<Value>) -> Value {
        table_from_columns(names.iter().map(|n| n.to_string()).collect(), columns).unwrap()
    }

    /// The observation set-up table `id, x, y`.
    fn t_table() -> Value {
        table(
            &["id", "x", "y"],
            vec![col(&[1.0, 2.0]), col(&[10.0, 20.0]), col(&[30.0, 40.0])],
        )
    }

    // ---- Calls ------------------------------------------------------------

    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("stack", args))
    }

    fn identifier(err: crate::RuntimeError) -> String {
        err.identifier().unwrap_or("<none>").to_string()
    }

    fn err_id(args: &[Value]) -> String {
        identifier(call(args).expect_err("stack should fail"))
    }

    // ---- Inspection -------------------------------------------------------

    fn table_object(value: &Value) -> ObjectInstance {
        match value {
            Value::Object(object) if object.class_name.is(TABLE_CLASS) => object.clone(),
            other => panic!("expected table object, got {other:?}"),
        }
    }

    fn names(value: &Value) -> Vec<String> {
        table_variable_names_from_object(&table_object(value)).unwrap()
    }

    fn size(value: &Value) -> (usize, usize) {
        let object = table_object(value);
        (
            table_height(&object).unwrap(),
            table_width(&object).unwrap(),
        )
    }

    fn var(value: &Value, name: &str) -> Value {
        table_variables(&table_object(value))
            .unwrap()
            .fields
            .get(name)
            .cloned()
            .unwrap_or_else(|| panic!("no variable {name}"))
    }

    fn class_of(value: &Value) -> String {
        crate::builtins::introspection::class::class_name_for_value(value)
    }

    fn assert_doubles(value: &Value, name: &str, expected: &[f64]) {
        let column = var(value, name);
        assert_eq!(class_of(&column), "double", "class of {name}");
        let got = match &column {
            Value::Tensor(t) => t.materialize_f64(),
            Value::Num(n) => vec![*n],
            other => panic!("{name}: expected double column, got {other:?}"),
        };
        assert_eq!(got, expected, "values of {name}");
        assert_eq!(value_row_count(&column).unwrap(), expected.len());
    }

    /// Values (per row) and categories of a categorical variable.
    fn categorical(value: &Value, name: &str) -> (Vec<String>, Vec<String>) {
        let column = var(value, name);
        assert_eq!(class_of(&column), "categorical", "class of {name}");
        let Value::Object(object) = &column else {
            panic!("categorical object expected");
        };
        let rows = value_row_count(&column).unwrap();
        let labels = (0..rows)
            .map(|row| crate::builtins::table::categorical_label_at(object, row).unwrap())
            .collect();
        let categories = match object.properties.get("Categories") {
            Some(Value::StringArray(array)) => array.data.clone(),
            other => panic!("categories: {other:?}"),
        };
        (labels, categories)
    }

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|s| s.to_string()).collect()
    }

    fn assert_indicator(value: &Value, name: &str, labels: &[&str], categories: &[&str]) {
        let (got_labels, got_categories) = categorical(value, name);
        assert_eq!(got_labels, strings(labels), "indicator values of {name}");
        assert_eq!(
            got_categories,
            strings(categories),
            "indicator categories of {name}"
        );
    }

    // ---- Observed (normative): stack 0.2.0 cases --------------------------

    /// [stack.signature-primary, stack.layout, stack.default-names,
    /// stack.indicator] Case `basic`.
    #[test]
    fn observed_basic() {
        let out = call(&[t_table(), cellstr(&["x", "y"])]).expect("stack");
        assert_eq!(class_of(&out), "table");
        assert_eq!(names(&out), ["id", "x_y_Indicator", "x_y"]);
        assert_eq!(size(&out), (4, 3));
        assert_doubles(&out, "id", &[1.0, 1.0, 2.0, 2.0]);
        assert_indicator(&out, "x_y_Indicator", &["x", "y", "x", "y"], &["x", "y"]);
        assert_doubles(&out, "x_y", &[10.0, 30.0, 20.0, 40.0]);
    }

    /// [stack.default-names, stack.indicator] Case `indicator`: names, the
    /// indicator class and its string values.
    #[test]
    fn observed_indicator() {
        let out = call(&[t_table(), cellstr(&["x", "y"])]).expect("stack");
        assert_eq!(names(&out), ["id", "x_y_Indicator", "x_y"]);
        let indicator = var(&out, "x_y_Indicator");
        assert_eq!(class_of(&indicator), "categorical");
        let (labels, _) = categorical(&out, "x_y_Indicator");
        assert_eq!(labels, strings(&["x", "y", "x", "y"]));
    }

    /// [stack.by-index, stack.layout] Case `by-index`.
    #[test]
    fn observed_by_index() {
        let out = call(&[t_table(), row(&[2.0, 3.0])]).expect("stack");
        let by_name = call(&[t_table(), cellstr(&["x", "y"])]).expect("stack");
        assert_eq!(names(&out), ["id", "x_y_Indicator", "x_y"]);
        assert_eq!(size(&out), (4, 3));
        assert_doubles(&out, "id", &[1.0, 1.0, 2.0, 2.0]);
        assert_indicator(&out, "x_y_Indicator", &["x", "y", "x", "y"], &["x", "y"]);
        assert_doubles(&out, "x_y", &[10.0, 30.0, 20.0, 40.0]);
        assert_eq!(names(&out), names(&by_name));
    }

    /// [stack.options] Case `new-names`.
    #[test]
    fn observed_new_names() {
        let out = call(&[
            t_table(),
            cellstr(&["x", "y"]),
            text("NewDataVariableName"),
            text("val"),
            text("IndexVariableName"),
            text("which"),
        ])
        .expect("stack");
        assert_eq!(names(&out), ["id", "which", "val"]);
        assert_eq!(size(&out), (4, 3));
        assert_doubles(&out, "id", &[1.0, 1.0, 2.0, 2.0]);
        assert_indicator(&out, "which", &["x", "y", "x", "y"], &["x", "y"]);
        assert_doubles(&out, "val", &[10.0, 30.0, 20.0, 40.0]);
    }

    /// [stack.options] Case `constant`: `'ConstantVariables',{}`.
    #[test]
    fn observed_constant_none() {
        let out = call(&[
            t_table(),
            cellstr(&["x", "y"]),
            text("ConstantVariables"),
            empty_cell(),
        ])
        .expect("stack");
        assert_eq!(names(&out), ["x_y_Indicator", "x_y"]);
        assert_eq!(size(&out), (4, 2));
        assert_indicator(&out, "x_y_Indicator", &["x", "y", "x", "y"], &["x", "y"]);
        assert_doubles(&out, "x_y", &[10.0, 30.0, 20.0, 40.0]);
    }

    /// [stack.layout, stack.default-names] Case `three-vars`.
    #[test]
    fn observed_three_vars() {
        let t = table(
            &["id", "x", "y", "z"],
            vec![
                col(&[1.0, 2.0]),
                col(&[10.0, 20.0]),
                col(&[30.0, 40.0]),
                col(&[50.0, 60.0]),
            ],
        );
        let out = call(&[t, cellstr(&["x", "y", "z"])]).expect("stack");
        assert_eq!(names(&out), ["id", "x_y_z_Indicator", "x_y_z"]);
        assert_eq!(size(&out), (6, 3));
        assert_doubles(&out, "id", &[1.0, 1.0, 1.0, 2.0, 2.0, 2.0]);
        assert_indicator(
            &out,
            "x_y_z_Indicator",
            &["x", "y", "z", "x", "y", "z"],
            &["x", "y", "z"],
        );
        assert_doubles(&out, "x_y_z", &[10.0, 30.0, 50.0, 20.0, 40.0, 60.0]);
    }

    /// [stack.single-var] Case `one-var`.
    #[test]
    fn observed_one_var() {
        let out = call(&[t_table(), text("x")]).expect("stack");
        assert_eq!(names(&out), ["id", "y", "x_Indicator", "x"]);
        assert_eq!(size(&out), (2, 4));
        assert_doubles(&out, "id", &[1.0, 2.0]);
        assert_doubles(&out, "y", &[30.0, 40.0]);
        assert_indicator(&out, "x_Indicator", &["x", "x"], &["x"]);
        assert_doubles(&out, "x", &[10.0, 20.0]);
    }

    /// [stack.error-unknown] Case `err-unknown`.
    #[test]
    fn observed_error_unknown_variable() {
        let id = err_id(&[t_table(), cellstr(&["x", "nope"])]);
        assert_eq!(id, "RunMat:table:UnrecognizedVarName");
    }

    /// [stack.error-class-mix] Case `err-class-mix`: double with cell.
    #[test]
    fn observed_error_class_mix() {
        let t = table(
            &["x", "y"],
            vec![col(&[1.0, 2.0]), cellstr_col(&["a", "b"])],
        );
        let id = err_id(&[t, cellstr(&["x", "y"])]);
        assert_eq!(id, "RunMat:table:stack:InterleavingDataVarsFailed");
    }

    // ---- Unresolved questions (FR-044-20..22) -----------------------------

    /// stack.q-second-output: a second output is not specified.
    #[test]
    fn unresolved_second_output_unsupported() {
        let _outputs = crate::output_count::push_output_count(Some(2));
        let err = block_on(stack_builtin(t_table(), vec![cellstr(&["x", "y"])]))
            .expect_err("second output");
        assert_eq!(identifier(err), "RunMat:stack:UnsupportedOption");
        drop(_outputs);
        let _outputs = crate::output_count::push_output_count(Some(1));
        assert!(block_on(stack_builtin(t_table(), vec![cellstr(&["x", "y"])])).is_ok());
    }

    /// stack.q-groups: a cell of cells (several groups).
    #[test]
    fn unresolved_groups_unsupported() {
        let groups =
            Value::Cell(CellArray::new(vec![cellstr(&["x"]), cellstr(&["y"])], 1, 2).unwrap());
        assert_eq!(
            err_id(&[t_table(), groups]),
            "RunMat:stack:UnsupportedOption"
        );
    }

    // ---- Documented choices (FR-044-25..30) -------------------------------

    /// U-044-1: categories follow the listing order of the stacked names.
    #[test]
    fn documented_choice_categories_in_listing_order() {
        let out = call(&[t_table(), cellstr(&["y", "x"])]).expect("stack");
        assert_eq!(names(&out), ["id", "y_x_Indicator", "y_x"]);
        assert_indicator(&out, "y_x_Indicator", &["y", "x", "y", "x"], &["y", "x"]);
        assert_doubles(&out, "y_x", &[30.0, 10.0, 40.0, 20.0]);
    }

    /// U-044-2: NewDataVariableName alone keeps the default indicator name.
    #[test]
    fn documented_choice_indicator_name_with_new_data_name() {
        let out = call(&[
            t_table(),
            cellstr(&["x", "y"]),
            text("NewDataVariableName"),
            text("val"),
        ])
        .expect("stack");
        assert_eq!(names(&out), ["id", "x_y_Indicator", "val"]);
        let out = call(&[
            t_table(),
            cellstr(&["x", "y"]),
            text("indexvariablename"),
            Value::String("w".to_string()),
        ])
        .expect("stack");
        assert_eq!(names(&out), ["id", "w", "x_y"]);
    }

    /// U-044-3: ConstantVariables lists carry exactly those variables, in
    /// table order.
    #[test]
    fn documented_choice_constant_variables() {
        let t = table(
            &["a", "x", "b", "y"],
            vec![
                col(&[1.0, 2.0]),
                col(&[10.0, 20.0]),
                col(&[5.0, 6.0]),
                col(&[30.0, 40.0]),
            ],
        );
        let out = call(&[
            t.clone(),
            cellstr(&["x", "y"]),
            text("ConstantVariables"),
            cellstr(&["b", "a"]),
        ])
        .expect("stack");
        assert_eq!(names(&out), ["a", "b", "x_y_Indicator", "x_y"]);
        assert_doubles(&out, "a", &[1.0, 1.0, 2.0, 2.0]);
        assert_doubles(&out, "b", &[5.0, 5.0, 6.0, 6.0]);
        let out = call(&[
            t.clone(),
            cellstr(&["x", "y"]),
            text("ConstantVariables"),
            Value::Num(3.0),
        ])
        .expect("stack");
        assert_eq!(names(&out), ["b", "x_y_Indicator", "x_y"]);
        assert_eq!(
            err_id(&[
                t.clone(),
                cellstr(&["x", "y"]),
                text("ConstantVariables"),
                text("x"),
            ]),
            "RunMat:stack:InvalidArgument"
        );
        assert_eq!(
            err_id(&[
                t,
                cellstr(&["x", "y"]),
                text("ConstantVariables"),
                text("nope"),
            ]),
            "RunMat:table:UnrecognizedVarName"
        );
    }

    /// U-044-4: same-class stacking for int32, logical, string and cellstr,
    /// and multi-column numeric variables.
    #[test]
    fn documented_choice_same_class_interleave() {
        let t = table(&["x", "y"], vec![int32_col(&[1, 2]), int32_col(&[3, 4])]);
        let out = call(&[t, cellstr(&["x", "y"])]).expect("int32");
        let data = var(&out, "x_y");
        assert_eq!(class_of(&data), "int32");
        match &data {
            Value::Tensor(tensor) => assert_eq!(tensor.materialize_f64(), [1.0, 3.0, 2.0, 4.0]),
            other => panic!("{other:?}"),
        }

        let t = table(
            &["x", "y"],
            vec![logical_col(&[true, false]), logical_col(&[false, true])],
        );
        let out = call(&[t, cellstr(&["x", "y"])]).expect("logical");
        match var(&out, "x_y") {
            Value::LogicalArray(array) => assert_eq!(array.data.to_vec(), [1, 0, 0, 1]),
            other => panic!("{other:?}"),
        }

        let t = table(
            &["x", "y"],
            vec![string_col(&["a", "b"]), string_col(&["c", "d"])],
        );
        let out = call(&[t, cellstr(&["x", "y"])]).expect("string");
        match var(&out, "x_y") {
            Value::StringArray(array) => assert_eq!(array.data, strings(&["a", "c", "b", "d"])),
            other => panic!("{other:?}"),
        }

        let t = table(
            &["x", "y"],
            vec![cellstr_col(&["a", "b"]), cellstr_col(&["c", "d"])],
        );
        let out = call(&[t, cellstr(&["x", "y"])]).expect("cellstr");
        match var(&out, "x_y") {
            Value::Cell(cell) => {
                assert_eq!((cell.rows, cell.cols), (4, 1));
                let got: Vec<String> = cell
                    .data
                    .iter()
                    .map(|item| match item {
                        Value::CharArray(chars) => chars.data.iter().collect(),
                        other => panic!("{other:?}"),
                    })
                    .collect();
                assert_eq!(got, strings(&["a", "c", "b", "d"]));
            }
            other => panic!("{other:?}"),
        }

        let two = |a: [f64; 4]| Value::Tensor(Tensor::new(a.to_vec(), vec![2, 2]).unwrap());
        let t = table(
            &["x", "y"],
            vec![two([1.0, 2.0, 3.0, 4.0]), two([5.0, 6.0, 7.0, 8.0])],
        );
        let out = call(&[t, cellstr(&["x", "y"])]).expect("two columns");
        match var(&out, "x_y") {
            Value::Tensor(tensor) => {
                assert_eq!(tensor.shape, [4, 2]);
                assert_eq!(
                    tensor.materialize_f64(),
                    [1.0, 5.0, 2.0, 6.0, 3.0, 7.0, 4.0, 8.0]
                );
            }
            other => panic!("{other:?}"),
        }
    }

    /// U-044-4: unobserved class mixes are UnsupportedOption, not the
    /// observed InterleavingDataVarsFailed.
    #[test]
    fn documented_choice_unobserved_class_mixes_unsupported() {
        let mixes = vec![
            vec![col(&[1.0, 2.0]), int32_col(&[1, 2])],
            vec![col(&[1.0, 2.0]), logical_col(&[true, false])],
            vec![string_col(&["a", "b"]), cellstr_col(&["c", "d"])],
            vec![int32_col(&[1, 2]), cellstr_col(&["c", "d"])],
        ];
        for columns in mixes {
            let t = table(&["x", "y"], columns);
            assert_eq!(
                err_id(&[t, cellstr(&["x", "y"])]),
                "RunMat:stack:UnsupportedOption"
            );
        }
        let chars = Value::CharArray(CharArray::new("ab".chars().collect(), 2, 1).unwrap());
        let t = table(&["x", "y"], vec![chars.clone(), chars]);
        assert_eq!(
            err_id(&[t, cellstr(&["x", "y"])]),
            "RunMat:stack:UnsupportedOption"
        );
        let two = Value::Tensor(Tensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap());
        let t = table(&["x", "y"], vec![two, col(&[1.0, 2.0])]);
        assert_eq!(
            err_id(&[t, cellstr(&["x", "y"])]),
            "RunMat:stack:UnsupportedOption"
        );
    }

    /// U-044-5: input validation.
    #[test]
    fn rejects_non_table_inputs() {
        assert_eq!(
            err_id(&[col(&[1.0, 2.0]), text("x")]),
            "RunMat:stack:InvalidInput"
        );
    }

    /// U-044-6: vars and option forms.
    #[test]
    fn rejects_malformed_arguments() {
        let invalid = "RunMat:stack:InvalidArgument";
        // String array and string scalar forms are accepted.
        let out = call(&[t_table(), string_col(&["x", "y"])]).expect("string array");
        assert_eq!(names(&out), ["id", "x_y_Indicator", "x_y"]);
        let out = call(&[t_table(), Value::String("x".to_string())]).expect("string");
        assert_eq!(names(&out), ["id", "y", "x_Indicator", "x"]);
        // Malformed vars.
        assert_eq!(err_id(&[t_table(), empty_cell()]), invalid);
        assert_eq!(err_id(&[t_table(), cellstr(&["x", "x"])]), invalid);
        assert_eq!(err_id(&[t_table(), row(&[2.0, 4.0])]), invalid);
        assert_eq!(err_id(&[t_table(), row(&[1.5])]), invalid);
        assert_eq!(
            err_id(&[
                t_table(),
                Value::LogicalArray(LogicalArray::new(vec![0, 1, 1], vec![1, 3]).unwrap())
            ]),
            "RunMat:stack:UnsupportedOption"
        );
        // Options.
        assert_eq!(
            err_id(&[t_table(), text("x"), text("IndexVariableName")]),
            invalid
        );
        assert_eq!(
            err_id(&[t_table(), text("x"), text("Bogus"), text("v")]),
            invalid
        );
        assert_eq!(
            err_id(&[
                t_table(),
                text("x"),
                text("NewDataVariableName"),
                col(&[1.0])
            ]),
            invalid
        );
        // Colliding output names.
        assert_eq!(
            err_id(&[
                t_table(),
                text("x"),
                text("NewDataVariableName"),
                text("id")
            ]),
            invalid
        );
    }

    /// U-044-5: carried variables of other classes repeat row-wise; an empty
    /// table stacks to an empty table.
    #[test]
    fn documented_choice_carried_classes_and_empty() {
        let t = table(
            &["name", "x", "y"],
            vec![string_col(&["p", "q"]), col(&[1.0, 2.0]), col(&[3.0, 4.0])],
        );
        let out = call(&[t, cellstr(&["x", "y"])]).expect("stack");
        match var(&out, "name") {
            Value::StringArray(array) => assert_eq!(array.data, strings(&["p", "p", "q", "q"])),
            other => panic!("{other:?}"),
        }
        let t = table(&["id", "x", "y"], vec![col(&[]), col(&[]), col(&[])]);
        let out = call(&[t, cellstr(&["x", "y"])]).expect("empty");
        assert_eq!(size(&out), (0, 3));
        assert_eq!(names(&out), ["id", "x_y_Indicator", "x_y"]);
    }
}
