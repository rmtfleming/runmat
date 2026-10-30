//! Clean-room `addvars`, `removevars`, `movevars` (table variable editing).
//!
//! Clean-room feature `specs/034-table-variable-editing/`. Normative behaviour
//! comes only from the approved exports `addvars`, `removevars`, `movevars`
//! 0.3.0 (see that feature's `provenance.md`). Questions those exports leave
//! open either raise `RunMat:<fn>:UnsupportedOption` or follow a documented
//! choice listed in the feature's `spec.md`.

use super::*;
use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;

use crate::{build_runtime_error, RuntimeError};
use BuiltinParamArity::{Required, Variadic};

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

const TABLE_OUTPUT: [BuiltinParamDescriptor; 1] = [param(
    "T2",
    BuiltinParamType::Any,
    Required,
    "Table with the edited variable list.",
)];
const TABLE_INPUT: BuiltinParamDescriptor =
    param("T", BuiltinParamType::Any, Required, "Input table.");
const BEFORE_KEY: BuiltinParamDescriptor = param(
    "'Before'",
    BuiltinParamType::PropertyName,
    Required,
    "Place relative to the start of `location`.",
);
const AFTER_KEY: BuiltinParamDescriptor = param(
    "'After'",
    BuiltinParamType::PropertyName,
    Required,
    "Place relative to the end of `location`.",
);
const LOCATION: BuiltinParamDescriptor = param(
    "location",
    BuiltinParamType::Any,
    Required,
    "An existing variable name or a numeric variable index.",
);
const SELECTION_DOC: &str =
    "Variables: a name, string array or cellstr of names, numeric indices, or a logical mask.";

const ADDVARS_DATA: BuiltinParamDescriptor = param(
    "var1, ..., varN",
    BuiltinParamType::Any,
    Variadic,
    "Data for each new variable; each must have one row per table row.",
);
const ADDVARS_NAMES_KEY: BuiltinParamDescriptor = param(
    "'NewVariableNames'",
    BuiltinParamType::PropertyName,
    Required,
    "Name the new variables explicitly.",
);
const ADDVARS_NAMES: BuiltinParamDescriptor = param(
    "names",
    BuiltinParamType::Any,
    Required,
    "One name per new variable: char row, string scalar or array, or cellstr.",
);
const ADDVARS_INPUTS_APPEND: [BuiltinParamDescriptor; 2] = [TABLE_INPUT, ADDVARS_DATA];
const ADDVARS_INPUTS_BEFORE: [BuiltinParamDescriptor; 4] =
    [TABLE_INPUT, ADDVARS_DATA, BEFORE_KEY, LOCATION];
const ADDVARS_INPUTS_AFTER: [BuiltinParamDescriptor; 4] =
    [TABLE_INPUT, ADDVARS_DATA, AFTER_KEY, LOCATION];
const ADDVARS_INPUTS_NAMES: [BuiltinParamDescriptor; 4] =
    [TABLE_INPUT, ADDVARS_DATA, ADDVARS_NAMES_KEY, ADDVARS_NAMES];
const ADDVARS_SIGNATURES: [BuiltinSignatureDescriptor; 4] = [
    BuiltinSignatureDescriptor {
        label: "T2 = addvars(T, var1, ..., varN)",
        inputs: &ADDVARS_INPUTS_APPEND,
        outputs: &TABLE_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "T2 = addvars(T, var1, ..., varN, 'Before', location)",
        inputs: &ADDVARS_INPUTS_BEFORE,
        outputs: &TABLE_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "T2 = addvars(T, var1, ..., varN, 'After', location)",
        inputs: &ADDVARS_INPUTS_AFTER,
        outputs: &TABLE_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "T2 = addvars(___, 'NewVariableNames', names)",
        inputs: &ADDVARS_INPUTS_NAMES,
        outputs: &TABLE_OUTPUT,
    },
];

const REMOVEVARS_INPUTS: [BuiltinParamDescriptor; 2] = [
    TABLE_INPUT,
    param("vars", BuiltinParamType::Any, Required, SELECTION_DOC),
];
const REMOVEVARS_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "T2 = removevars(T, vars)",
    inputs: &REMOVEVARS_INPUTS,
    outputs: &TABLE_OUTPUT,
}];

const MOVEVARS_VARS: BuiltinParamDescriptor =
    param("vars", BuiltinParamType::Any, Required, SELECTION_DOC);
const MOVEVARS_INPUTS_BEFORE: [BuiltinParamDescriptor; 4] =
    [TABLE_INPUT, MOVEVARS_VARS, BEFORE_KEY, LOCATION];
const MOVEVARS_INPUTS_AFTER: [BuiltinParamDescriptor; 4] =
    [TABLE_INPUT, MOVEVARS_VARS, AFTER_KEY, LOCATION];
const MOVEVARS_INPUTS_END: [BuiltinParamDescriptor; 2] = [TABLE_INPUT, MOVEVARS_VARS];
const MOVEVARS_SIGNATURES: [BuiltinSignatureDescriptor; 3] = [
    BuiltinSignatureDescriptor {
        label: "T2 = movevars(T, vars, 'Before', location)",
        inputs: &MOVEVARS_INPUTS_BEFORE,
        outputs: &TABLE_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "T2 = movevars(T, vars, 'After', location)",
        inputs: &MOVEVARS_INPUTS_AFTER,
        outputs: &TABLE_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "T2 = movevars(T, vars)",
        inputs: &MOVEVARS_INPUTS_END,
        outputs: &TABLE_OUTPUT,
    },
];

/// Error kinds. The table-level identifiers are the confirmed identifiers
/// of the approved exports with the `MATLAB:` namespace mapped to RunMat's
/// (FR-034-11).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    InvalidInput,
    InvalidArgument,
    UnsupportedOption,
    UnrecognizedVarName,
    VarIndexOutOfRange,
    BadSubscript,
    RowDimensionMismatch,
    DuplicateVarNames,
    IncorrectNumberOfVarNames,
    BeforeAndAfter,
}

impl Kind {
    fn code_suffix(self) -> &'static str {
        match self {
            Kind::InvalidInput => "INVALID_INPUT",
            Kind::InvalidArgument => "INVALID_ARGUMENT",
            Kind::UnsupportedOption => "UNSUPPORTED_OPTION",
            Kind::UnrecognizedVarName => "UNRECOGNIZED_VAR_NAME",
            Kind::VarIndexOutOfRange => "VAR_INDEX_OUT_OF_RANGE",
            Kind::BadSubscript => "BAD_SUBSCRIPT",
            Kind::RowDimensionMismatch => "ROW_DIMENSION_MISMATCH",
            Kind::DuplicateVarNames => "DUPLICATE_VAR_NAMES",
            Kind::IncorrectNumberOfVarNames => "INCORRECT_NUMBER_OF_VAR_NAMES",
            Kind::BeforeAndAfter => "BEFORE_AND_AFTER",
        }
    }
}

macro_rules! error_entry {
    (InvalidInput, $prefix:literal, $id:literal) => {
        BuiltinErrorDescriptor {
            code: concat!("RM.", $prefix, ".INVALID_INPUT"),
            identifier: Some(concat!("RunMat:", $id, ":InvalidInput")),
            when: "The first argument is not a table.",
            message: concat!($id, ": expected a table"),
        }
    };
    (InvalidArgument, $prefix:literal, $id:literal) => {
        BuiltinErrorDescriptor {
            code: concat!("RM.", $prefix, ".INVALID_ARGUMENT"),
            identifier: Some(concat!("RunMat:", $id, ":InvalidArgument")),
            when: "An argument has an unrecognized form or an option is malformed.",
            message: concat!($id, ": invalid argument"),
        }
    };
    (UnsupportedOption, $prefix:literal, $id:literal) => {
        BuiltinErrorDescriptor {
            code: concat!("RM.", $prefix, ".UNSUPPORTED_OPTION"),
            identifier: Some(concat!("RunMat:", $id, ":UnsupportedOption")),
            when: "The call uses a form the approved specification leaves unresolved.",
            message: concat!($id, ": option not yet specified"),
        }
    };
    (UnrecognizedVarName, $prefix:literal, $id:literal) => {
        BuiltinErrorDescriptor {
            code: concat!("RM.", $prefix, ".UNRECOGNIZED_VAR_NAME"),
            identifier: Some("RunMat:table:UnrecognizedVarName"),
            when: "A variable name or location does not name a variable of the table.",
            message: concat!($id, ": unrecognized table variable name"),
        }
    };
    (VarIndexOutOfRange, $prefix:literal, $id:literal) => {
        BuiltinErrorDescriptor {
            code: concat!("RM.", $prefix, ".VAR_INDEX_OUT_OF_RANGE"),
            identifier: Some("RunMat:table:VarIndexOutOfRange"),
            when: "A numeric variable index exceeds the number of variables.",
            message: concat!($id, ": variable index out of range"),
        }
    };
    (BadSubscript, $prefix:literal, $id:literal) => {
        BuiltinErrorDescriptor {
            code: concat!("RM.", $prefix, ".BAD_SUBSCRIPT"),
            identifier: Some("RunMat:badsubscript"),
            when: "A numeric variable index is zero, negative or not an integer.",
            message: concat!($id, ": invalid variable index"),
        }
    };
    (RowDimensionMismatch, $prefix:literal, $id:literal) => {
        BuiltinErrorDescriptor {
            code: concat!("RM.", $prefix, ".ROW_DIMENSION_MISMATCH"),
            identifier: Some("RunMat:table:RowDimensionMismatch"),
            when: "New variable data does not have one row per table row.",
            message: concat!($id, ": variable height does not match the table"),
        }
    };
    (DuplicateVarNames, $prefix:literal, $id:literal) => {
        BuiltinErrorDescriptor {
            code: concat!("RM.", $prefix, ".DUPLICATE_VAR_NAMES"),
            identifier: Some("RunMat:table:DuplicateVarNames"),
            when: "A new variable name duplicates another variable name.",
            message: concat!($id, ": duplicate variable name"),
        }
    };
    (IncorrectNumberOfVarNames, $prefix:literal, $id:literal) => {
        BuiltinErrorDescriptor {
            code: concat!("RM.", $prefix, ".INCORRECT_NUMBER_OF_VAR_NAMES"),
            identifier: Some("RunMat:table:addmovevars:IncorrectNumberOfVarNames"),
            when: "'NewVariableNames' does not give exactly one name per new variable.",
            message: concat!($id, ": wrong number of new variable names"),
        }
    };
    (BeforeAndAfter, $prefix:literal, $id:literal) => {
        BuiltinErrorDescriptor {
            code: concat!("RM.", $prefix, ".BEFORE_AND_AFTER"),
            identifier: Some("RunMat:table:addmovevars:BeforeAndAfter"),
            when: "Both 'Before' and 'After' are supplied.",
            message: concat!($id, ": 'Before' and 'After' cannot both be given"),
        }
    };
}

macro_rules! error_set {
    ($errors:ident, $prefix:literal, $id:literal, [$($kind:ident),* $(,)?]) => {
        const $errors: &[BuiltinErrorDescriptor] = &[$(error_entry!($kind, $prefix, $id)),*];
    };
}

error_set!(
    ADDVARS_ERRORS,
    "ADDVARS",
    "addvars",
    [
        InvalidInput,
        InvalidArgument,
        UnsupportedOption,
        UnrecognizedVarName,
        VarIndexOutOfRange,
        BadSubscript,
        RowDimensionMismatch,
        DuplicateVarNames,
        IncorrectNumberOfVarNames,
        BeforeAndAfter,
    ]
);
error_set!(
    REMOVEVARS_ERRORS,
    "REMOVEVARS",
    "removevars",
    [
        InvalidInput,
        InvalidArgument,
        UnsupportedOption,
        UnrecognizedVarName,
        VarIndexOutOfRange,
        BadSubscript,
    ]
);
error_set!(
    MOVEVARS_ERRORS,
    "MOVEVARS",
    "movevars",
    [
        InvalidInput,
        InvalidArgument,
        UnsupportedOption,
        UnrecognizedVarName,
        VarIndexOutOfRange,
        BadSubscript,
        BeforeAndAfter,
    ]
);

pub const ADDVARS_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &ADDVARS_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: ADDVARS_ERRORS,
};
pub const REMOVEVARS_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &REMOVEVARS_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: REMOVEVARS_ERRORS,
};
pub const MOVEVARS_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &MOVEVARS_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: MOVEVARS_ERRORS,
};

// ---------------------------------------------------------------------------
// Shared core
// ---------------------------------------------------------------------------

/// The builtin being served: its name and its error descriptors.
#[derive(Clone, Copy)]
struct Ctx {
    name: &'static str,
    errors: &'static [BuiltinErrorDescriptor],
}

const ADDVARS: Ctx = Ctx {
    name: "addvars",
    errors: ADDVARS_ERRORS,
};
const REMOVEVARS: Ctx = Ctx {
    name: "removevars",
    errors: REMOVEVARS_ERRORS,
};
const MOVEVARS: Ctx = Ctx {
    name: "movevars",
    errors: MOVEVARS_ERRORS,
};

impl Ctx {
    fn error(self, kind: Kind, detail: impl Into<String>) -> RuntimeError {
        let mut builder = build_runtime_error(format!("{}: {}", self.name, detail.into()))
            .with_builtin(self.name);
        let suffix = kind.code_suffix();
        let descriptor = self
            .errors
            .iter()
            .find(|descriptor| descriptor.code.rsplit('.').next() == Some(suffix));
        debug_assert!(descriptor.is_some(), "{} lacks {kind:?}", self.name);
        if let Some(identifier) = descriptor.and_then(|descriptor| descriptor.identifier) {
            builder = builder.with_identifier(identifier);
        }
        builder.build()
    }
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

/// Real numeric values of a numeric argument, or `None` if it is not numeric.
fn numeric_values(value: &Value) -> Option<Vec<f64>> {
    match value {
        Value::Num(n) => Some(vec![*n]),
        Value::Int(i) => Some(vec![i.to_f64()]),
        Value::Tensor(tensor) => {
            let count: usize = tensor.shape.iter().product();
            Some(
                (0..count)
                    .map(|idx| crate::builtins::common::tensor::tensor_value_f64(tensor, idx))
                    .collect(),
            )
        }
        _ => None,
    }
}

async fn gathered(value: &Value) -> BuiltinResult<Value> {
    gather_if_needed_async(value)
        .await
        .map_err(map_control_flow)
}

async fn table_input(ctx: Ctx, value: Value) -> BuiltinResult<ObjectInstance> {
    match gathered(&value).await? {
        Value::Object(object) if object.class_name.is(TABLE_CLASS) => Ok(object),
        Value::Object(object) if object.class_name.is(TIMETABLE_CLASS) => Err(ctx.error(
            Kind::UnsupportedOption,
            "timetable inputs are not yet specified (U-034-3)",
        )),
        other => Err(ctx.error(
            Kind::InvalidInput,
            format!("expected a table, got {}", value_class_label(&other)),
        )),
    }
}

fn value_class_label(value: &Value) -> String {
    crate::builtins::introspection::class::class_name_for_value(value)
}

fn position_of(ctx: Ctx, names: &[String], name: &str) -> BuiltinResult<usize> {
    names
        .iter()
        .position(|candidate| candidate == name)
        .ok_or_else(|| {
            ctx.error(
                Kind::UnrecognizedVarName,
                format!("table has no variable named '{name}'"),
            )
        })
}

/// Validate a 1-based numeric variable index against `count` variables.
fn index_position(ctx: Ctx, raw: f64, count: usize) -> BuiltinResult<usize> {
    if !raw.is_finite() || raw < 1.0 || raw.fract() != 0.0 {
        return Err(ctx.error(
            Kind::BadSubscript,
            format!("variable index {raw} must be a positive integer"),
        ));
    }
    if raw > count as f64 {
        return Err(ctx.error(
            Kind::VarIndexOutOfRange,
            format!("variable index {raw} exceeds the {count} table variables"),
        ));
    }
    Ok(raw as usize - 1)
}

/// Resolve a variable selection (removevars `vars`, movevars `vars`) to
/// 0-based positions, in listed order for names and indices and in table
/// order for a logical mask; repeated entries are kept once.
async fn select_variables(ctx: Ctx, value: &Value, names: &[String]) -> BuiltinResult<Vec<usize>> {
    let value = gathered(value).await?;
    let mut positions = Vec::new();
    let bits: Option<Vec<bool>> = match &value {
        Value::Bool(bit) => Some(vec![*bit]),
        Value::LogicalArray(array) => Some(array.data.as_slice().iter().map(|b| *b != 0).collect()),
        _ => None,
    };
    if let Some(bits) = bits {
        if bits.len() != names.len() {
            // removevars.q-short-mask / removevars.q-logical-longer
            return Err(ctx.error(
                Kind::UnsupportedOption,
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
    match &value {
        Value::Cell(cell) => {
            for item in &cell.data {
                let name = single_name(item).ok_or_else(|| {
                    ctx.error(
                        Kind::InvalidArgument,
                        "a cell selection must contain only variable names",
                    )
                })?;
                positions.push(position_of(ctx, names, &name)?);
            }
        }
        Value::StringArray(array) => {
            for name in &array.data {
                positions.push(position_of(ctx, names, name)?);
            }
        }
        Value::CharArray(chars) if chars.rows > 1 => {
            return Err(ctx.error(
                Kind::UnsupportedOption,
                "a multi-row char array of names is not yet specified",
            ));
        }
        other => {
            if let Some(name) = single_name(other) {
                positions.push(position_of(ctx, names, &name)?);
            } else if let Some(indices) = numeric_values(other) {
                for raw in indices {
                    positions.push(index_position(ctx, raw, names.len())?);
                }
            } else {
                return Err(ctx.error(
                    Kind::InvalidArgument,
                    format!(
                        "cannot select table variables with a {}",
                        value_class_label(other)
                    ),
                ));
            }
        }
    }
    let mut seen = vec![false; names.len()];
    positions.retain(|idx| !std::mem::replace(&mut seen[*idx], true));
    Ok(positions)
}

/// Where edited variables go, relative to the variables that stay.
#[derive(Clone, Copy)]
enum Placement {
    End,
    Before(usize),
    After(usize),
}

/// Resolve a 'Before'/'After' location against the input table.
async fn location_position(
    ctx: Ctx,
    value: &Value,
    names: &[String],
    before: bool,
) -> BuiltinResult<usize> {
    let value = gathered(value).await?;
    if let Some(name) = single_name(&value) {
        return position_of(ctx, names, &name);
    }
    match numeric_values(&value) {
        Some(values) if values.len() == 1 => {
            let raw = values[0];
            if before && raw == names.len() as f64 + 1.0 {
                // addvars.q-before-past-end (and its movevars analogue)
                return Err(ctx.error(
                    Kind::UnsupportedOption,
                    "'Before' with an index one past the last variable is not yet specified",
                ));
            }
            index_position(ctx, raw, names.len())
        }
        _ => Err(ctx.error(
            Kind::UnsupportedOption,
            "a location must be one variable name or one numeric index",
        )),
    }
}

/// Parsed trailing name-value options.
#[derive(Default)]
struct Options {
    before: Option<Value>,
    after: Option<Value>,
    new_names: Option<Value>,
}

/// Index of the first option keyword in `args` at or after `from`, given the
/// recognised keywords. Everything from there on must be name-value pairs.
fn option_start(args: &[Value], from: usize, keywords: &[&str]) -> usize {
    (from..args.len())
        .find(|idx| {
            single_name(&args[*idx]).is_some_and(|text| {
                keywords
                    .iter()
                    .any(|keyword| keyword.eq_ignore_ascii_case(&text))
            })
        })
        .unwrap_or(args.len())
}

fn parse_options(ctx: Ctx, pairs: &[Value], keywords: &[&str]) -> BuiltinResult<Options> {
    if !pairs.len().is_multiple_of(2) {
        return Err(ctx.error(Kind::InvalidArgument, "options must be name-value pairs"));
    }
    let mut options = Options::default();
    for pair in pairs.chunks(2) {
        let key = single_name(&pair[0])
            .map(|text| text.to_ascii_lowercase())
            .filter(|text| keywords.iter().any(|k| k.eq_ignore_ascii_case(text)))
            .ok_or_else(|| ctx.error(Kind::InvalidArgument, "unrecognized option name"))?;
        let slot = match key.as_str() {
            "before" => &mut options.before,
            "after" => &mut options.after,
            _ => &mut options.new_names,
        };
        if slot.replace(pair[1].clone()).is_some() {
            return Err(ctx.error(
                Kind::UnsupportedOption,
                format!("repeating the '{key}' option is not yet specified"),
            ));
        }
    }
    Ok(options)
}

async fn placement(ctx: Ctx, options: &Options, names: &[String]) -> BuiltinResult<Placement> {
    match (&options.before, &options.after) {
        (Some(_), Some(_)) => Err(ctx.error(
            Kind::BeforeAndAfter,
            "'Before' and 'After' cannot both be given",
        )),
        (Some(location), None) => Ok(Placement::Before(
            location_position(ctx, location, names, true).await?,
        )),
        (None, Some(location)) => Ok(Placement::After(
            location_position(ctx, location, names, false).await?,
        )),
        (None, None) => Ok(Placement::End),
    }
}

/// Insertion point in a list of `len` kept variables, for an anchor given by
/// its position `anchor_at` in that list.
fn insertion_index(placement: Placement, anchor_at: impl Fn(usize) -> usize, len: usize) -> usize {
    match placement {
        Placement::End => len,
        Placement::Before(anchor) => anchor_at(anchor),
        Placement::After(anchor) => anchor_at(anchor) + 1,
    }
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
// addvars naming helpers
// ---------------------------------------------------------------------------

fn is_plain_identifier(text: &str) -> bool {
    let mut chars = text.chars();
    chars
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

/// True when an argument's source text could expand to several values (cell
/// or struct-field comma lists), which would shift later arguments.
fn may_expand(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    chars.iter().enumerate().any(|(idx, ch)| match ch {
        '{' => true,
        '.' => chars
            .get(idx + 1)
            .is_some_and(|next| next.is_ascii_alphabetic() || *next == '_' || *next == '('),
        _ => false,
    })
}

/// Caller variable names of the data arguments, read from the builtin
/// callsite the VM records (the same mechanism `display` uses for its
/// label). addvars.default-name-input; addvars.q-input-name-scope is a
/// documented choice: only a plain identifier counts, and the callsite is
/// trusted only when its argument count matches this call and no earlier
/// argument could have expanded to several values.
fn caller_names(total_args: usize, data_count: usize) -> Vec<Option<String>> {
    let aligned = crate::callsite::current_callsite()
        .is_some_and(|callsite| callsite.arg_spans.len() == total_args);
    let mut out = Vec::with_capacity(data_count);
    let mut shifted = false;
    for arg in 0..=data_count {
        let text = if aligned {
            crate::callsite::arg_text(arg).map(|text| text.trim().to_string())
        } else {
            None
        };
        if arg > 0 {
            out.push(
                text.as_ref()
                    .filter(|text| !shifted && is_plain_identifier(text))
                    .cloned(),
            );
        }
        shifted |= text.is_some_and(|text| may_expand(&text));
    }
    out
}

fn names_list(ctx: Ctx, value: &Value) -> BuiltinResult<Vec<String>> {
    if let Some(name) = single_name(value) {
        return Ok(vec![name]);
    }
    let list = match value {
        Value::StringArray(array) => Some(array.data.clone()),
        Value::Cell(cell) => cell.data.iter().map(single_name).collect(),
        _ => None,
    };
    list.ok_or_else(|| {
        ctx.error(
            Kind::InvalidArgument,
            "'NewVariableNames' must be text or a cell array of names",
        )
    })
}

// ---------------------------------------------------------------------------
// Builtins
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "addvars",
    category = "table",
    summary = "Insert new variables into a table.",
    keywords = "addvars,table,variables,append,insert,column",
    accel = "cpu",
    descriptor(crate::builtins::table::builtins::variable_editing::ADDVARS_DESCRIPTOR),
    builtin_path = "crate::builtins::table::builtins"
)]
pub(crate) async fn addvars_builtin(
    table: Value,
    var: Value,
    rest: Vec<Value>,
) -> BuiltinResult<Value> {
    let ctx = ADDVARS;
    let total_args = 2 + rest.len();
    let object = table_input(ctx, table).await?;
    let mut args = Vec::with_capacity(1 + rest.len());
    args.push(var);
    args.extend(rest);
    let keywords = ["Before", "After", "NewVariableNames"];
    let split = option_start(&args, 1, &keywords);
    let options = parse_options(ctx, &args[split..], &keywords)?;
    args.truncate(split);

    let names = table_variable_names_from_object(&object)?;
    let place = placement(ctx, &options, &names).await?;

    let mut data = Vec::with_capacity(args.len());
    for arg in &args {
        data.push(gathered(arg).await?);
    }
    let height = if table_width(&object)? > 0 {
        table_height(&object)?
    } else {
        // A table with no variables has no height of its own (FR-034-08).
        value_row_count(&data[0])?
    };
    for value in &data {
        let rows = value_row_count(value)?;
        if rows != height {
            return Err(ctx.error(
                Kind::RowDimensionMismatch,
                format!("new variable has {rows} rows but the table has {height}"),
            ));
        }
    }

    let insert_at = insertion_index(place, |anchor| anchor, names.len());
    let new_names: Vec<String> = match &options.new_names {
        Some(value) => {
            let given = names_list(ctx, &gathered(value).await?)?;
            if given.len() != data.len() {
                return Err(ctx.error(
                    Kind::IncorrectNumberOfVarNames,
                    format!(
                        "{} names given for {} new variables",
                        given.len(),
                        data.len()
                    ),
                ));
            }
            for (idx, name) in given.iter().enumerate() {
                if make_valid_variable_name(name, 1) != *name {
                    return Err(ctx.error(
                        Kind::InvalidArgument,
                        format!("'{name}' is not a valid variable name"),
                    ));
                }
                if names.contains(name) || given[..idx].contains(name) {
                    return Err(ctx.error(
                        Kind::DuplicateVarNames,
                        format!("duplicate variable name '{name}'"),
                    ));
                }
            }
            given
        }
        None => {
            // addvars.default-name-input, else addvars.default-name-position;
            // a colliding default is made unique (addvars.q-default-name-collision).
            let callers = caller_names(total_args, data.len());
            let mut taken = names.clone();
            let mut chosen = Vec::with_capacity(data.len());
            for (offset, caller) in callers.into_iter().enumerate() {
                let candidate = caller.unwrap_or_else(|| format!("Var{}", insert_at + offset + 1));
                let mut candidates = taken.clone();
                candidates.push(candidate);
                let name = make_unique_names(candidates)
                    .pop()
                    .expect("candidate list is non-empty");
                taken.push(name.clone());
                chosen.push(name);
            }
            chosen
        }
    };

    let mut slots: Vec<Slot> = (0..names.len()).map(Slot::Existing).collect();
    let new_slots = new_names
        .into_iter()
        .zip(data)
        .map(|(name, value)| Slot::New(name, value));
    slots.splice(insert_at..insert_at, new_slots);
    rebuild(object, slots)
}

#[runtime_builtin(
    name = "removevars",
    category = "table",
    summary = "Remove variables from a table.",
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
    let ctx = REMOVEVARS;
    let object = table_input(ctx, table).await?;
    if !rest.is_empty() {
        return Err(ctx.error(
            Kind::InvalidArgument,
            "removevars takes a table and one variable selection",
        ));
    }
    let names = table_variable_names_from_object(&object)?;
    let removed = select_variables(ctx, &vars, &names).await?;
    let kept: Vec<usize> = (0..names.len())
        .filter(|idx| !removed.contains(idx))
        .collect();
    if kept.is_empty() && table_height(&object)? > 0 {
        // Deviation D-034-1 (removevars.remove-all).
        return Err(ctx.error(
            Kind::UnsupportedOption,
            "removing every variable of a table with rows is not supported: RunMat tables cannot hold rows without variables (D-034-1)",
        ));
    }
    rebuild(object, kept.into_iter().map(Slot::Existing).collect())
}

#[runtime_builtin(
    name = "movevars",
    category = "table",
    summary = "Move table variables before or after another variable, or to the end.",
    keywords = "movevars,table,variables,move,reorder,column",
    accel = "cpu",
    descriptor(crate::builtins::table::builtins::variable_editing::MOVEVARS_DESCRIPTOR),
    builtin_path = "crate::builtins::table::builtins"
)]
pub(crate) async fn movevars_builtin(table: Value, rest: Vec<Value>) -> BuiltinResult<Value> {
    let ctx = MOVEVARS;
    let object = table_input(ctx, table).await?;
    let Some((vars, pairs)) = rest.split_first() else {
        return Err(ctx.error(
            Kind::InvalidArgument,
            "movevars needs the variables to move",
        ));
    };
    let options = parse_options(ctx, pairs, &["Before", "After"])?;
    let names = table_variable_names_from_object(&object)?;
    let place = placement(ctx, &options, &names).await?;
    let moving = select_variables(ctx, vars, &names).await?;

    let anchor = match place {
        Placement::Before(anchor) | Placement::After(anchor) => Some(anchor),
        Placement::End => None,
    };
    if let Some(anchor) = anchor.filter(|anchor| moving.contains(anchor)) {
        if moving.len() == 1 {
            // movevars.self-location: unchanged.
            debug_assert_eq!(moving[0], anchor);
            return rebuild(object, (0..names.len()).map(Slot::Existing).collect());
        }
        // movevars.q-location-in-vars
        return Err(ctx.error(
            Kind::UnsupportedOption,
            "moving several variables relative to one of themselves is not yet specified",
        ));
    }

    let mut order: Vec<usize> = (0..names.len())
        .filter(|idx| !moving.contains(idx))
        .collect();
    let insert_at = insertion_index(
        place,
        |anchor| {
            order
                .iter()
                .position(|idx| *idx == anchor)
                .expect("anchor is not being moved")
        },
        order.len(),
    );
    order.splice(insert_at..insert_at, moving);
    rebuild(object, order.into_iter().map(Slot::Existing).collect())
}

#[cfg(test)]
mod table_variable_editing_tests {
    use super::*;
    use futures::executor::block_on;
    use runmat_types::{SourceId, Span};
    use runmat_value::{CellArray, CharArray, LogicalArray, StringArray, Tensor};

    // RunMat spellings of the confirmed MATLAB identifiers (the `MATLAB:`
    // namespace maps to RunMat's error namespace; see spec.md FR-034-11).
    const BEFORE_AND_AFTER: &str = "RunMat:table:addmovevars:BeforeAndAfter";
    const ROW_MISMATCH: &str = "RunMat:table:RowDimensionMismatch";
    const DUPLICATE_NAMES: &str = "RunMat:table:DuplicateVarNames";
    const NAME_COUNT: &str = "RunMat:table:addmovevars:IncorrectNumberOfVarNames";
    const UNRECOGNIZED: &str = "RunMat:table:UnrecognizedVarName";
    const OUT_OF_RANGE: &str = "RunMat:table:VarIndexOutOfRange";
    const BAD_SUBSCRIPT: &str = "RunMat:badsubscript";

    fn column(values: &[f64]) -> Value {
        Value::Tensor(Tensor::new(values.to_vec(), vec![values.len(), 1]).unwrap())
    }

    /// 1-row table whose k-th variable holds k.
    fn row_table(names: &[&str]) -> Value {
        let columns = (0..names.len())
            .map(|idx| Value::Num(idx as f64 + 1.0))
            .collect();
        table_from_columns(names.iter().map(|n| n.to_string()).collect(), columns).unwrap()
    }

    /// The observed 2x2 table `a = [1;3]`, `b = [2;4]`.
    fn ab_table() -> Value {
        table_from_columns(
            vec!["a".into(), "b".into()],
            vec![column(&[1.0, 3.0]), column(&[2.0, 4.0])],
        )
        .unwrap()
    }

    /// The observed 2x4 table `a..d = [1 2 3 4; 5 6 7 8]`.
    fn abcd_table() -> Value {
        table_from_columns(
            vec!["a".into(), "b".into(), "c".into(), "d".into()],
            vec![
                column(&[1.0, 5.0]),
                column(&[2.0, 6.0]),
                column(&[3.0, 7.0]),
                column(&[4.0, 8.0]),
            ],
        )
        .unwrap()
    }

    fn text(s: &str) -> Value {
        Value::CharArray(CharArray::new_row(s))
    }

    fn cellstr(items: &[&str]) -> Value {
        Value::Cell(
            CellArray::new(items.iter().map(|s| text(s)).collect(), 1, items.len()).unwrap(),
        )
    }

    fn strings(items: &[&str]) -> Value {
        Value::StringArray(
            StringArray::new(
                items.iter().map(|s| s.to_string()).collect(),
                vec![1, items.len()],
            )
            .unwrap(),
        )
    }

    fn row(values: &[f64]) -> Value {
        Value::Tensor(Tensor::new(values.to_vec(), vec![1, values.len()]).unwrap())
    }

    fn mask(bits: &[bool]) -> Value {
        Value::LogicalArray(
            LogicalArray::new(bits.iter().map(|b| *b as u8).collect(), vec![1, bits.len()])
                .unwrap(),
        )
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

    fn joined(value: &Value) -> String {
        names(value).join(",")
    }

    fn size(value: &Value) -> (usize, usize) {
        let object = table_object(value.clone());
        (
            table_height(&object).unwrap(),
            table_width(&object).unwrap(),
        )
    }

    fn variable(value: &Value, name: &str) -> Value {
        table_variables(&table_object(value.clone()))
            .unwrap()
            .fields
            .get(name)
            .cloned()
            .unwrap_or_else(|| panic!("no variable {name}"))
    }

    fn identifier(err: crate::RuntimeError) -> String {
        err.identifier().unwrap_or("<none>").to_string()
    }

    fn err_id(name: &str, args: &[Value]) -> String {
        match call(name, args) {
            Ok(value) => panic!("{name} unexpectedly succeeded: {value:?}"),
            Err(err) => identifier(err),
        }
    }

    fn ok(name: &str, args: &[Value]) -> Value {
        call(name, args).unwrap_or_else(|err| {
            panic!("{name} failed: {} ({:?})", err.message(), err.identifier())
        })
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

    /// Run `body` with a builtin callsite whose argument texts are `args`,
    /// as the VM records for `out = addvars(<args>);`.
    fn with_callsite<R>(id: usize, args: &[&str], body: impl FnOnce() -> R) -> R {
        let source = format!("out = addvars({});", args.join(", "));
        let _catalog = crate::source_context::replace_source_catalog(vec![(
            SourceId(id),
            "/tmp/f34_callsite.m".to_string(),
            source.clone(),
        )]);
        let mut spans = Vec::new();
        let mut from = "out = addvars(".len();
        for arg in args {
            let start = from + source[from..].find(arg).expect("arg present");
            spans.push(Span {
                start,
                end: start + arg.len(),
            });
            from = start + arg.len();
        }
        let _callsite = crate::callsite::push_callsite(Some(SourceId(id)), Some(spans));
        body()
    }

    // ---- addvars: observed (normative) ----------------------------------

    /// [addvars.signature-primary, addvars.output-class, addvars.appends-variable]
    /// Case `append`.
    #[test]
    fn observed_addvars_append_matches_export() {
        let t = table_from_columns(vec!["a".into()], vec![column(&[1.0, 2.0])]).unwrap();
        let out = ok("addvars", &[t, column(&[3.0, 4.0])]);
        assert_eq!(size(&out), (2, 2));
        assert_eq!(joined(&out), "a,Var2");
        assert_eq!(variable(&out, "a"), column(&[1.0, 2.0]));
        assert_eq!(variable(&out, "Var2"), column(&[3.0, 4.0]));
    }

    /// [addvars.appends-variable, addvars.default-name-position] Case `append-full`.
    #[test]
    fn observed_addvars_append_full_matches_export() {
        let out = ok("addvars", &[ab_table(), column(&[5.0, 6.0])]);
        assert_eq!(size(&out), (2, 3));
        assert_eq!(joined(&out), "a,b,Var3");
        assert_eq!(variable(&out, "a"), column(&[1.0, 3.0]));
        assert_eq!(variable(&out, "b"), column(&[2.0, 4.0]));
        assert_eq!(variable(&out, "Var3"), column(&[5.0, 6.0]));
    }

    /// [addvars.before, addvars.after, addvars.default-name-position]
    /// Cases `before-name`, `after-name`, `before-index`, `after-index`.
    #[test]
    fn observed_addvars_before_and_after_match_export() {
        for (key, location, expected, new_name) in [
            ("Before", text("a"), "Var1,a,b", "Var1"),
            ("After", text("a"), "a,Var2,b", "Var2"),
            ("Before", Value::Num(2.0), "a,Var2,b", "Var2"),
            ("After", Value::Num(2.0), "a,b,Var3", "Var3"),
        ] {
            let out = ok(
                "addvars",
                &[ab_table(), column(&[5.0, 6.0]), text(key), location],
            );
            assert_eq!(size(&out), (2, 3));
            assert_eq!(joined(&out), expected, "{key}");
            assert_eq!(variable(&out, new_name), column(&[5.0, 6.0]));
            assert_eq!(variable(&out, "a"), column(&[1.0, 3.0]));
        }
    }

    /// [addvars.new-names] Cases `newname-char`, `newname-cell`,
    /// `newname-string`, `names-and-before`.
    #[test]
    fn observed_addvars_new_variable_names_match_export() {
        for names_arg in [text("z"), cellstr(&["z"]), Value::String("z".into())] {
            let out = ok(
                "addvars",
                &[
                    ab_table(),
                    column(&[5.0, 6.0]),
                    text("NewVariableNames"),
                    names_arg,
                ],
            );
            assert_eq!(joined(&out), "a,b,z");
            assert_eq!(variable(&out, "z"), column(&[5.0, 6.0]));
        }
        let out = ok(
            "addvars",
            &[
                ab_table(),
                column(&[5.0, 6.0]),
                text("Before"),
                text("b"),
                text("NewVariableNames"),
                text("z"),
            ],
        );
        assert_eq!(joined(&out), "a,z,b");
    }

    /// [addvars.multiple-variables, addvars.default-name-position,
    /// addvars.new-names] Cases `multi-default`, `multi-names`.
    #[test]
    fn observed_addvars_multiple_variables_match_export() {
        let out = ok(
            "addvars",
            &[ab_table(), column(&[5.0, 6.0]), column(&[7.0, 8.0])],
        );
        assert_eq!(size(&out), (2, 4));
        assert_eq!(joined(&out), "a,b,Var3,Var4");
        assert_eq!(variable(&out, "Var3"), column(&[5.0, 6.0]));
        assert_eq!(variable(&out, "Var4"), column(&[7.0, 8.0]));

        let out = ok(
            "addvars",
            &[
                ab_table(),
                column(&[5.0, 6.0]),
                column(&[7.0, 8.0]),
                text("NewVariableNames"),
                cellstr(&["y", "z"]),
            ],
        );
        assert_eq!(joined(&out), "a,b,y,z");
        assert_eq!(variable(&out, "y"), column(&[5.0, 6.0]));
        assert_eq!(variable(&out, "z"), column(&[7.0, 8.0]));
    }

    /// [addvars.data-preserved] Cases `multicol`, `cellstr-var`.
    #[test]
    fn observed_addvars_data_preserved_matches_export() {
        let matrix = Value::Tensor(Tensor::new(vec![5.0, 6.0, 7.0, 8.0], vec![2, 2]).unwrap());
        let out = ok(
            "addvars",
            &[
                ab_table(),
                matrix.clone(),
                text("NewVariableNames"),
                text("z"),
            ],
        );
        assert_eq!(size(&out), (2, 3));
        assert_eq!(variable(&out, "z"), matrix);

        let cells = Value::Cell(CellArray::new(vec![text("p"), text("q")], 2, 1).unwrap());
        let out = ok(
            "addvars",
            &[
                ab_table(),
                cells.clone(),
                text("NewVariableNames"),
                text("z"),
            ],
        );
        assert_eq!(size(&out), (2, 3));
        assert_eq!(variable(&out, "z"), cells);
    }

    /// [addvars.default-name-input] Case `name-from-input`: the data
    /// argument is the caller's variable `x`.
    #[test]
    fn observed_addvars_name_from_input_matches_export() {
        let out = with_callsite(3401, &["tbl", "x"], || {
            ok("addvars", &[ab_table(), column(&[5.0, 6.0])])
        });
        assert_eq!(size(&out), (2, 3));
        assert_eq!(joined(&out), "a,b,x");
        assert_eq!(variable(&out, "x"), column(&[5.0, 6.0]));
    }

    /// [addvars.error-before-and-after, addvars.error-rows,
    /// addvars.error-dup-name, addvars.error-names-count,
    /// addvars.error-bad-location, addvars.error-index-range]
    #[test]
    fn observed_addvars_errors_match_export() {
        let v = || column(&[5.0, 6.0]);
        let cases: Vec<(Vec<Value>, &str)> = vec![
            (
                vec![text("Before"), text("a"), text("After"), text("b")],
                BEFORE_AND_AFTER,
            ),
            (vec![text("NewVariableNames"), text("a")], DUPLICATE_NAMES),
            (vec![text("Before"), text("nope")], UNRECOGNIZED),
            (vec![text("After"), text("nope")], UNRECOGNIZED),
            (vec![text("After"), Value::Num(3.0)], OUT_OF_RANGE),
            (
                vec![text("NewVariableNames"), cellstr(&["y", "z"])],
                NAME_COUNT,
            ),
        ];
        for (extra, expected) in cases {
            let mut args = vec![ab_table(), v()];
            args.extend(extra);
            assert_eq!(err_id("addvars", &args), expected);
        }
        // err-names-count: two variables, one name
        assert_eq!(
            err_id(
                "addvars",
                &[
                    ab_table(),
                    v(),
                    column(&[7.0, 8.0]),
                    text("NewVariableNames"),
                    text("z"),
                ],
            ),
            NAME_COUNT
        );
        // err-rows / err-rows-fewer
        assert_eq!(
            err_id("addvars", &[ab_table(), column(&[5.0, 6.0, 7.0])]),
            ROW_MISMATCH
        );
        assert_eq!(
            err_id("addvars", &[ab_table(), Value::Num(5.0)]),
            ROW_MISMATCH
        );
    }

    // ---- removevars: observed (normative) -------------------------------

    /// [removevars.signature-primary, removevars.output-class, removevars.removes-variable]
    /// Cases `by-name`, `name-full`, `string-scalar`.
    #[test]
    fn observed_removevars_by_name_matches_export() {
        let out = ok("removevars", &[row_table(&["a", "b", "c"]), text("b")]);
        assert_eq!(size(&out), (1, 2));
        assert_eq!(joined(&out), "a,c");

        let out = ok("removevars", &[abcd_table(), text("b")]);
        assert_eq!(size(&out), (2, 3));
        assert_eq!(joined(&out), "a,c,d");
        assert_eq!(variable(&out, "c"), column(&[3.0, 7.0]));

        let out = ok("removevars", &[abcd_table(), Value::String("d".into())]);
        assert_eq!(joined(&out), "a,b,c");
    }

    /// [removevars.by-index, removevars.by-logical] Cases `index`,
    /// `index-vector`, `index-unordered`, `logical`.
    #[test]
    fn observed_removevars_by_index_and_mask_match_export() {
        let out = ok("removevars", &[abcd_table(), Value::Num(2.0)]);
        assert_eq!(size(&out), (2, 3));
        assert_eq!(joined(&out), "a,c,d");
        for selector in [
            row(&[1.0, 3.0]),
            row(&[3.0, 1.0]),
            mask(&[true, false, true, false]),
        ] {
            let out = ok("removevars", &[abcd_table(), selector]);
            assert_eq!(size(&out), (2, 2));
            assert_eq!(joined(&out), "b,d");
            assert_eq!(variable(&out, "d"), column(&[4.0, 8.0]));
        }
    }

    /// [removevars.multiple-names] Cases `cellstr`, `string-array`, `dup-names`.
    #[test]
    fn observed_removevars_multiple_names_match_export() {
        for selector in [cellstr(&["a", "c"]), strings(&["a", "c"])] {
            let out = ok("removevars", &[abcd_table(), selector]);
            assert_eq!(joined(&out), "b,d");
        }
        let out = ok("removevars", &[abcd_table(), cellstr(&["a", "a"])]);
        assert_eq!(size(&out), (2, 3));
        assert_eq!(joined(&out), "b,c,d");
    }

    /// [removevars.empty-selection] Cases `empty-cell`, `empty-index`.
    #[test]
    fn observed_removevars_empty_selection_matches_export() {
        let empty_cell = Value::Cell(CellArray::new(Vec::new(), 0, 0).unwrap());
        let empty_index = Value::Tensor(Tensor::new(Vec::new(), vec![0, 0]).unwrap());
        for selector in [empty_cell, empty_index] {
            let out = ok("removevars", &[abcd_table(), selector]);
            assert_eq!(size(&out), (2, 4));
            assert_eq!(joined(&out), "a,b,c,d");
        }
    }

    /// [removevars.error-unknown, removevars.error-index-range,
    /// removevars.error-index-zero]
    #[test]
    fn observed_removevars_errors_match_export() {
        assert_eq!(
            err_id("removevars", &[abcd_table(), text("nope")]),
            UNRECOGNIZED
        );
        assert_eq!(
            err_id("removevars", &[abcd_table(), Value::Num(5.0)]),
            OUT_OF_RANGE
        );
        for bad in [0.0, -1.0, 1.5] {
            assert_eq!(
                err_id("removevars", &[abcd_table(), Value::Num(bad)]),
                BAD_SUBSCRIPT
            );
        }
    }

    // ---- movevars: observed (normative) ---------------------------------

    /// [movevars.signature-primary, movevars.output-class, movevars.moves-variable]
    /// Case `before`.
    #[test]
    fn observed_movevars_before_matches_export() {
        let out = ok(
            "movevars",
            &[
                row_table(&["a", "b", "c"]),
                text("c"),
                text("Before"),
                text("a"),
            ],
        );
        assert_eq!(size(&out), (1, 3));
        assert_eq!(joined(&out), "c,a,b");
    }

    fn moved(args: Vec<Value>) -> String {
        let mut all = vec![row_table(&["a", "b", "c", "d"])];
        all.extend(args);
        joined(&ok("movevars", &all))
    }

    /// [movevars.after, movevars.numeric-location] Cases `after-name`,
    /// `after-last`, `before-index`, `after-index`.
    #[test]
    fn observed_movevars_after_and_numeric_location_match_export() {
        assert_eq!(moved(vec![text("a"), text("After"), text("c")]), "b,c,a,d");
        assert_eq!(moved(vec![text("a"), text("After"), text("d")]), "b,c,d,a");
        assert_eq!(
            moved(vec![text("d"), text("Before"), Value::Num(2.0)]),
            "a,d,b,c"
        );
        assert_eq!(
            moved(vec![text("a"), text("After"), Value::Num(3.0)]),
            "b,c,a,d"
        );
    }

    /// [movevars.numeric-vars, movevars.multiple-order] Cases `vars-index`,
    /// `vars-logical`, `multi-order`, `multi-after`.
    #[test]
    fn observed_movevars_selectors_and_order_match_export() {
        assert_eq!(
            moved(vec![Value::Num(4.0), text("Before"), Value::Num(1.0)]),
            "d,a,b,c"
        );
        assert_eq!(
            moved(vec![
                mask(&[false, true, false, true]),
                text("Before"),
                Value::Num(1.0)
            ]),
            "b,d,a,c"
        );
        assert_eq!(
            moved(vec![cellstr(&["d", "b"]), text("Before"), text("a")]),
            "d,b,a,c"
        );
        assert_eq!(
            moved(vec![cellstr(&["a", "b"]), text("After"), text("c")]),
            "c,a,b,d"
        );
    }

    /// [movevars.self-location] Cases `self`, `self-after`.
    #[test]
    fn observed_movevars_self_location_matches_export() {
        assert_eq!(moved(vec![text("b"), text("Before"), text("b")]), "a,b,c,d");
        assert_eq!(moved(vec![text("b"), text("After"), text("b")]), "a,b,c,d");
    }

    /// [movevars.default-end] Cases `no-location`, `no-location-multi`.
    #[test]
    fn observed_movevars_default_end_matches_export() {
        assert_eq!(moved(vec![text("a")]), "b,c,d,a");
        assert_eq!(moved(vec![cellstr(&["b", "a"])]), "c,d,b,a");
    }

    /// [movevars.contents-follow] Case `contents`.
    #[test]
    fn observed_movevars_contents_follow_matches_export() {
        let t = table_from_columns(
            vec!["a".into(), "b".into(), "c".into()],
            vec![
                column(&[1.0, 4.0]),
                column(&[2.0, 5.0]),
                column(&[3.0, 6.0]),
            ],
        )
        .unwrap();
        let out = ok("movevars", &[t, text("a"), text("After"), text("c")]);
        assert_eq!(size(&out), (2, 3));
        assert_eq!(joined(&out), "b,c,a");
        assert_eq!(variable(&out, "b"), column(&[2.0, 5.0]));
        assert_eq!(variable(&out, "c"), column(&[3.0, 6.0]));
        assert_eq!(variable(&out, "a"), column(&[1.0, 4.0]));
    }

    /// [movevars.error-before-and-after, movevars.error-unknown-var,
    /// movevars.error-unknown-location, movevars.error-index-range]
    #[test]
    fn observed_movevars_errors_match_export() {
        let t = || row_table(&["a", "b", "c", "d"]);
        let cases: Vec<(Vec<Value>, &str)> = vec![
            (
                vec![
                    text("a"),
                    text("Before"),
                    text("b"),
                    text("After"),
                    text("c"),
                ],
                BEFORE_AND_AFTER,
            ),
            (vec![text("nope"), text("Before"), text("a")], UNRECOGNIZED),
            (vec![text("a"), text("After"), text("nope")], UNRECOGNIZED),
            (
                vec![text("a"), text("After"), Value::Num(5.0)],
                OUT_OF_RANGE,
            ),
        ];
        for (rest, expected) in cases {
            let mut args = vec![t()];
            args.extend(rest);
            assert_eq!(err_id("movevars", &args), expected);
        }
    }

    // ---- Unresolved questions (explicit RunMat errors) -------------------

    /// addvars.q-before-past-end: the observed non-erroring `'Before', n+1`
    /// is non-normative and awaiting review, so RunMat refuses it.
    #[test]
    fn unresolved_addvars_before_past_end_is_unsupported() {
        assert_eq!(
            err_id(
                "addvars",
                &[
                    ab_table(),
                    column(&[5.0, 6.0]),
                    text("Before"),
                    Value::Num(3.0)
                ],
            ),
            "RunMat:addvars:UnsupportedOption"
        );
    }

    /// removevars.q-short-mask and removevars.q-logical-longer.
    #[test]
    fn unresolved_removevars_mask_length_mismatch_is_unsupported() {
        for selector in [
            mask(&[true, false]),
            mask(&[true, false, false, false, true]),
        ] {
            assert_eq!(
                err_id("removevars", &[abcd_table(), selector]),
                "RunMat:removevars:UnsupportedOption"
            );
        }
    }

    /// movevars.q-location-in-vars: several variables moved relative to one
    /// of themselves.
    #[test]
    fn unresolved_movevars_location_in_vars_is_unsupported() {
        assert_eq!(
            err_id(
                "movevars",
                &[
                    row_table(&["a", "b", "c", "d"]),
                    cellstr(&["a", "c"]),
                    text("Before"),
                    text("c"),
                ],
            ),
            "RunMat:movevars:UnsupportedOption"
        );
    }

    // ---- Documented choices ---------------------------------------------

    /// addvars.q-default-name-collision (documented choice): a default name
    /// that already exists is made unique; existing names never change.
    #[test]
    fn unresolved_choice_addvars_default_name_is_unique_var_k() {
        let clash = table_from_columns(vec!["Var2".into()], vec![column(&[1.0, 2.0])]).unwrap();
        let out = ok("addvars", &[clash, column(&[3.0, 4.0])]);
        let out_names = names(&out);
        assert_eq!(out_names[0], "Var2");
        assert_eq!(out_names.len(), 2);
        assert_ne!(out_names[1], "Var2");
    }

    /// addvars.q-input-name-scope (documented choice): only a plain caller
    /// variable name is used; any other expression, or a callsite whose
    /// arguments do not line up with the call, gives the position name.
    #[test]
    fn unresolved_choice_addvars_input_name_scope() {
        for (id, args) in [
            (3402, vec!["tbl", "s.val"]),
            (3403, vec!["tbl", "w(:,1)"]),
            (3404, vec!["tbl", "c{:}", "extra"]),
        ] {
            let out = with_callsite(id, &args, || {
                ok("addvars", &[ab_table(), column(&[5.0, 6.0])])
            });
            assert_eq!(joined(&out), "a,b,Var3", "{args:?}");
        }
        // Several plain variables each keep their caller name; explicit
        // NewVariableNames win over caller names.
        let out = with_callsite(3405, &["tbl", "p", "q"], || {
            ok(
                "addvars",
                &[ab_table(), column(&[5.0, 6.0]), column(&[7.0, 8.0])],
            )
        });
        assert_eq!(joined(&out), "a,b,p,q");
        let out = with_callsite(3406, &["tbl", "p", "'NewVariableNames'", "'z'"], || {
            ok(
                "addvars",
                &[
                    ab_table(),
                    column(&[5.0, 6.0]),
                    text("NewVariableNames"),
                    text("z"),
                ],
            )
        });
        assert_eq!(joined(&out), "a,b,z");
    }

    /// Deviation D-034-1: RunMat tables have no row count of their own, so a
    /// table with rows and no variables (removevars.remove-all, size [2 0])
    /// cannot be represented; RunMat refuses instead of returning 0x0.
    #[test]
    fn deviation_removevars_remove_all_with_rows_is_unsupported() {
        let all = row(&[1.0, 2.0, 3.0, 4.0]);
        assert_eq!(
            err_id("removevars", &[abcd_table(), all]),
            "RunMat:removevars:UnsupportedOption"
        );
    }

    #[test]
    fn unresolved_choice_metadata_follows_variables() {
        let t = with_units(row_table(&["a", "b", "c"]), &["m", "s", "kg"]);
        let moved = ok(
            "movevars",
            &[t.clone(), text("c"), text("before"), text("a")],
        );
        assert_eq!(
            string_list_property(&moved, VARIABLE_UNITS),
            ["kg", "m", "s"]
        );
        assert_eq!(
            string_list_property(&moved, VARIABLE_NAMES),
            ["c", "a", "b"]
        );

        let removed = ok("removevars", &[t.clone(), text("b")]);
        assert_eq!(string_list_property(&removed, VARIABLE_UNITS), ["m", "kg"]);

        let added = ok("addvars", &[t, Value::Num(9.0), text("Before"), text("a")]);
        assert_eq!(
            string_list_property(&added, VARIABLE_UNITS),
            ["", "m", "s", "kg"]
        );
        let props = table_public_properties(&table_object(added)).unwrap();
        assert_eq!(
            props.fields.get(DESCRIPTION),
            Some(&Value::String("demo".into()))
        );
    }

    #[test]
    fn accepts_string_scalar_names() {
        let out = ok(
            "removevars",
            &[row_table(&["a", "b", "c"]), Value::String("a".into())],
        );
        assert_eq!(names(&out), ["b", "c"]);
        let out = ok(
            "movevars",
            &[
                row_table(&["a", "b", "c"]),
                Value::String("a".into()),
                Value::String("Before".into()),
                Value::String("c".into()),
            ],
        );
        assert_eq!(names(&out), ["b", "a", "c"]);
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
            assert_eq!(err_id(name, &args), format!("RunMat:{name}:InvalidInput"));
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
        assert_eq!(
            err_id("removevars", &[tt, text("a")]),
            "RunMat:removevars:UnsupportedOption"
        );
    }

    #[test]
    fn rejects_malformed_name_value_arguments() {
        // unknown option name after a recognised pair
        assert_eq!(
            err_id(
                "movevars",
                &[
                    row_table(&["a", "b"]),
                    text("a"),
                    text("Sideways"),
                    text("b"),
                ],
            ),
            "RunMat:movevars:InvalidArgument"
        );
        // missing option value
        assert_eq!(
            err_id(
                "movevars",
                &[row_table(&["a", "b"]), text("a"), text("Before")]
            ),
            "RunMat:movevars:InvalidArgument"
        );
        // NewVariableNames is not a movevars option
        assert_eq!(
            err_id(
                "movevars",
                &[
                    row_table(&["a", "b"]),
                    text("a"),
                    text("NewVariableNames"),
                    text("z"),
                ],
            ),
            "RunMat:movevars:InvalidArgument"
        );
        // repeated option (unobserved)
        assert_eq!(
            err_id(
                "addvars",
                &[
                    ab_table(),
                    column(&[5.0, 6.0]),
                    text("Before"),
                    text("a"),
                    text("Before"),
                    text("b"),
                ],
            ),
            "RunMat:addvars:UnsupportedOption"
        );
    }
}
