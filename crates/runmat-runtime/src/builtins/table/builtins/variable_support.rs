//! Shared helpers for the clean-room `renamevars`, `rows2vars`, `varfun`
//! and `convertvars` builtins (feature `specs/056-table-variables/`).
//!
//! F34's selection helpers in `variable_editing.rs` are private to that
//! file, so the parts these builtins need are written here.

use super::*;
use runmat_builtins::BuiltinErrorDescriptor;

use crate::{build_runtime_error, RuntimeError};

/// Error-code suffixes shared by the four builtins' descriptor tables.
pub(super) const INVALID_INPUT: &str = "INVALID_INPUT";
pub(super) const INVALID_ARGUMENT: &str = "INVALID_ARGUMENT";
pub(super) const UNSUPPORTED_OPTION: &str = "UNSUPPORTED_OPTION";
pub(super) const TOO_MANY_OUTPUTS: &str = "TOO_MANY_OUTPUTS";
pub(super) const INTERNAL: &str = "INTERNAL";
pub(super) const UNRECOGNIZED_VAR_NAME: &str = "UNRECOGNIZED_VAR_NAME";
pub(super) const VAR_INDEX_OUT_OF_RANGE: &str = "VAR_INDEX_OUT_OF_RANGE";
pub(super) const BAD_SUBSCRIPT: &str = "BAD_SUBSCRIPT";

/// The builtin being served: its name and its error descriptors.
#[derive(Clone, Copy)]
pub(super) struct Ctx {
    pub(super) name: &'static str,
    pub(super) errors: &'static [BuiltinErrorDescriptor],
}

impl Ctx {
    /// An error carrying the identifier of the descriptor whose code ends
    /// in `.{suffix}`.
    pub(super) fn error(self, suffix: &str, detail: impl Into<String>) -> RuntimeError {
        let mut builder = build_runtime_error(format!("{}: {}", self.name, detail.into()))
            .with_builtin(self.name);
        let descriptor = self
            .errors
            .iter()
            .find(|descriptor| descriptor.code.rsplit('.').next() == Some(suffix));
        debug_assert!(descriptor.is_some(), "{} lacks {suffix}", self.name);
        if let Some(identifier) = descriptor.and_then(|descriptor| descriptor.identifier) {
            builder = builder.with_identifier(identifier);
        }
        builder.build()
    }

    /// U-056-1: one output.
    pub(super) fn check_outputs(self) -> BuiltinResult<()> {
        match crate::output_count::current_output_count() {
            Some(count) if count > 1 => Err(self.error(
                TOO_MANY_OUTPUTS,
                format!("returns one output, {count} were requested"),
            )),
            _ => Ok(()),
        }
    }

    /// U-056-1: a `table` input; a timetable is not yet specified.
    pub(super) async fn table_input(self, value: &Value) -> BuiltinResult<ObjectInstance> {
        match gathered(value).await? {
            Value::Object(object) if object.class_name.is(TABLE_CLASS) => Ok(object),
            Value::Object(object) if object.class_name.is(TIMETABLE_CLASS) => {
                Err(self.error(UNSUPPORTED_OPTION, "timetable inputs are not yet specified"))
            }
            other => Err(self.error(
                INVALID_INPUT,
                format!("expected a table, got {}", class_label(&other)),
            )),
        }
    }

    pub(super) fn position_of(self, names: &[String], name: &str) -> BuiltinResult<usize> {
        names
            .iter()
            .position(|candidate| candidate == name)
            .ok_or_else(|| {
                self.error(
                    UNRECOGNIZED_VAR_NAME,
                    format!("table has no variable named '{name}'"),
                )
            })
    }

    /// Validate a 1-based numeric variable index against `count` variables.
    pub(super) fn index_position(self, raw: f64, count: usize) -> BuiltinResult<usize> {
        if !raw.is_finite() || raw < 1.0 || raw.fract() != 0.0 {
            return Err(self.error(
                BAD_SUBSCRIPT,
                format!("variable index {raw} must be a positive integer"),
            ));
        }
        if raw > count as f64 {
            return Err(self.error(
                VAR_INDEX_OUT_OF_RANGE,
                format!("variable index {raw} exceeds the {count} table variables"),
            ));
        }
        Ok(raw as usize - 1)
    }

    /// Resolve a variable selection to 0-based positions: names and indices
    /// in listed order, a logical mask in table order. Repeated entries are
    /// kept; the caller decides what they mean (U-056-3).
    pub(super) async fn select(self, value: &Value, names: &[String]) -> BuiltinResult<Vec<usize>> {
        let value = gathered(value).await?;
        let bits: Option<Vec<bool>> = match &value {
            Value::Bool(bit) => Some(vec![*bit]),
            Value::LogicalArray(array) => {
                Some(array.data.as_slice().iter().map(|b| *b != 0).collect())
            }
            _ => None,
        };
        if let Some(bits) = bits {
            if bits.len() != names.len() {
                return Err(self.error(
                    UNSUPPORTED_OPTION,
                    format!(
                        "a logical mask of length {} for {} variables is not yet specified",
                        bits.len(),
                        names.len()
                    ),
                ));
            }
            return Ok(bits
                .iter()
                .enumerate()
                .filter(|(_, b)| **b)
                .map(|(i, _)| i)
                .collect());
        }
        let mut positions = Vec::new();
        match &value {
            Value::Cell(cell) => {
                for item in &cell.data {
                    let name = single_name(item).ok_or_else(|| {
                        self.error(
                            INVALID_ARGUMENT,
                            "a cell selection must contain only variable names",
                        )
                    })?;
                    positions.push(self.position_of(names, &name)?);
                }
            }
            Value::StringArray(array) => {
                for name in &array.data {
                    positions.push(self.position_of(names, name)?);
                }
            }
            Value::CharArray(chars) if chars.rows > 1 => {
                return Err(self.error(
                    UNSUPPORTED_OPTION,
                    "a multi-row char array of names is not yet specified",
                ));
            }
            other => {
                if let Some(name) = single_name(other) {
                    positions.push(self.position_of(names, &name)?);
                } else if let Some(indices) = numeric_values(other) {
                    for raw in indices {
                        positions.push(self.index_position(raw, names.len())?);
                    }
                } else {
                    return Err(self.error(
                        INVALID_ARGUMENT,
                        format!(
                            "cannot select table variables with a {}",
                            class_label(other)
                        ),
                    ));
                }
            }
        }
        Ok(positions)
    }
}

pub(super) async fn gathered(value: &Value) -> BuiltinResult<Value> {
    gather_if_needed_async(value)
        .await
        .map_err(map_control_flow)
}

pub(super) fn class_label(value: &Value) -> String {
    crate::builtins::introspection::class::class_name_for_value(value)
}

/// The text of a single name: a char row vector (or empty char), a string
/// scalar, or a one-element string array.
pub(super) fn single_name(value: &Value) -> Option<String> {
    match value {
        Value::CharArray(chars) if chars.rows <= 1 => Some(chars.data.iter().collect()),
        Value::String(text) => Some(text.clone()),
        Value::StringArray(array) if array.data.len() == 1 => Some(array.data[0].clone()),
        _ => None,
    }
}

/// A list of names: one text value, a string array or a cellstr.
pub(super) fn name_list(value: &Value) -> Option<Vec<String>> {
    if let Some(name) = single_name(value) {
        return Some(vec![name]);
    }
    match value {
        Value::StringArray(array) => Some(array.data.clone()),
        Value::Cell(cell) => cell.data.iter().map(single_name).collect(),
        _ => None,
    }
}

/// Real numeric values of a numeric argument, or `None` if it is not numeric.
pub(super) fn numeric_values(value: &Value) -> Option<Vec<f64>> {
    match value {
        Value::Num(n) => Some(vec![*n]),
        Value::Int(i) => Some(vec![i.to_f64()]),
        Value::Tensor(tensor) => Some(
            (0..tensor.len())
                .map(|idx| crate::builtins::common::tensor::tensor_value_f64(tensor, idx))
                .collect(),
        ),
        _ => None,
    }
}

/// Replace the variable store of `source` with `columns` under `names`,
/// keeping every other property (row names, units, descriptions,
/// dimension names) except `VariableNames`, which is set to `names`.
pub(super) fn with_variables(
    source: ObjectInstance,
    names: Vec<String>,
    columns: Vec<Value>,
) -> BuiltinResult<Value> {
    let mut props = table_public_properties(&source)?;
    let mut variables = StructValue::new();
    for (name, value) in names.iter().cloned().zip(columns) {
        variables.insert(name, value);
    }
    let count = names.len();
    props.insert(
        VARIABLE_NAMES,
        Value::StringArray(
            StringArray::new(names, vec![1, count])
                .map_err(|err| invalid_variable(format!("table: {err}")))?,
        ),
    );
    let mut object = source;
    object
        .properties
        .insert(TABLE_VARIABLES_FIELD.to_string(), Value::Struct(variables));
    sync_table_properties(&mut object, props);
    Ok(Value::Object(object))
}

/// The variable values of `object` in variable order.
pub(super) fn columns_of(object: &ObjectInstance) -> BuiltinResult<(Vec<String>, Vec<Value>)> {
    let variables = table_variables(object)?;
    let names: Vec<String> = variables.fields.keys().cloned().collect();
    let columns = names
        .iter()
        .map(|name| {
            variables
                .fields
                .get(name)
                .cloned()
                .unwrap_or(Value::Num(0.0))
        })
        .collect();
    Ok((names, columns))
}

/// Test helpers shared by the four test modules of this feature.
#[cfg(test)]
pub(super) mod test_helpers {
    use super::*;
    use futures::executor::block_on;
    use runmat_value::{CellArray, CharArray, LogicalArray, StringArray, Tensor};

    pub(crate) fn call(name: &str, args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async(name, args))
    }

    pub(crate) fn ok(name: &str, args: &[Value]) -> Value {
        call(name, args).unwrap_or_else(|err| {
            panic!("{name} failed: {} ({:?})", err.message(), err.identifier())
        })
    }

    pub(crate) fn err_id(name: &str, args: &[Value]) -> String {
        match call(name, args) {
            Ok(value) => panic!("{name} unexpectedly succeeded: {value:?}"),
            Err(err) => err.identifier().unwrap_or("<none>").to_string(),
        }
    }

    pub(crate) fn column(values: &[f64]) -> Value {
        Value::Tensor(Tensor::new(values.to_vec(), vec![values.len(), 1]).unwrap())
    }

    pub(crate) fn row(values: &[f64]) -> Value {
        Value::Tensor(Tensor::new(values.to_vec(), vec![1, values.len()]).unwrap())
    }

    pub(crate) fn text(s: &str) -> Value {
        Value::CharArray(CharArray::new_row(s))
    }

    /// 1-by-n cellstr.
    pub(crate) fn cellstr(items: &[&str]) -> Value {
        Value::Cell(
            CellArray::new(items.iter().map(|s| text(s)).collect(), 1, items.len()).unwrap(),
        )
    }

    /// n-by-1 cellstr.
    pub(crate) fn cellcol(items: &[&str]) -> Value {
        Value::Cell(
            CellArray::new(items.iter().map(|s| text(s)).collect(), items.len(), 1).unwrap(),
        )
    }

    /// 1-by-n string array.
    pub(crate) fn strings(items: &[&str]) -> Value {
        Value::StringArray(
            StringArray::new(
                items.iter().map(|s| s.to_string()).collect(),
                vec![1, items.len()],
            )
            .unwrap(),
        )
    }

    pub(crate) fn mask(bits: &[bool]) -> Value {
        Value::LogicalArray(
            LogicalArray::new(bits.iter().map(|b| *b as u8).collect(), vec![1, bits.len()])
                .unwrap(),
        )
    }

    /// `table(columns..., 'VariableNames', names)` through the constructor.
    pub(crate) fn table(columns: Vec<Value>, names: &[&str]) -> Value {
        let mut args = columns;
        args.push(text("VariableNames"));
        args.push(cellstr(names));
        ok("table", &args)
    }

    /// As `table`, with `'RowNames', rows`.
    pub(crate) fn table_rows(columns: Vec<Value>, names: &[&str], rows: &[&str]) -> Value {
        let mut args = columns;
        args.push(text("VariableNames"));
        args.push(cellstr(names));
        args.push(text("RowNames"));
        args.push(cellstr(rows));
        ok("table", &args)
    }

    pub(crate) fn object(value: &Value) -> ObjectInstance {
        match value {
            Value::Object(object) if object.class_name.is(TABLE_CLASS) => object.clone(),
            other => panic!("expected table object, got {other:?}"),
        }
    }

    pub(crate) fn names(value: &Value) -> Vec<String> {
        table_variable_names_from_object(&object(value)).unwrap()
    }

    pub(crate) fn size(value: &Value) -> (usize, usize) {
        let object = object(value);
        (
            table_height(&object).unwrap(),
            table_width(&object).unwrap(),
        )
    }

    pub(crate) fn variable(value: &Value, name: &str) -> Value {
        table_variables(&object(value))
            .unwrap()
            .fields
            .get(name)
            .cloned()
            .unwrap_or_else(|| panic!("no variable {name} in {:?}", names(value)))
    }

    pub(crate) fn class_of(value: &Value) -> String {
        class_label(value)
    }

    /// Values and shape of a real double column.
    pub(crate) fn doubles(value: &Value) -> (Vec<usize>, Vec<f64>) {
        match value {
            Value::Num(n) => (vec![1, 1], vec![*n]),
            Value::Tensor(tensor) => (
                tensor.shape.clone(),
                (0..tensor.len())
                    .map(|idx| crate::builtins::common::tensor::tensor_value_f64(tensor, idx))
                    .collect(),
            ),
            other => panic!("expected numeric value, got {other:?}"),
        }
    }

    /// Element texts of a cellstr, in storage order.
    pub(crate) fn cell_texts(value: &Value) -> Vec<String> {
        match value {
            Value::Cell(cell) => cell
                .data
                .iter()
                .map(|item| single_name(item).unwrap_or_else(|| panic!("non-text {item:?}")))
                .collect(),
            other => panic!("expected cell, got {other:?}"),
        }
    }

    pub(crate) fn string_texts(value: &Value) -> Vec<String> {
        match value {
            Value::StringArray(array) => array.data.clone(),
            Value::String(text) => vec![text.clone()],
            other => panic!("expected string array, got {other:?}"),
        }
    }

    pub(crate) fn property_strings(value: &Value, field: &str) -> Vec<String> {
        let props = table_public_properties(&object(value)).unwrap();
        match props.fields.get(field) {
            Some(Value::StringArray(array)) => array.data.clone(),
            other => panic!("expected string array for {field}, got {other:?}"),
        }
    }

    pub(crate) fn sv(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }
}
