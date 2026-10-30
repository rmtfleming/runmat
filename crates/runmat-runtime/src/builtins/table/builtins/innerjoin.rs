//! Clean-room `innerjoin` (table inner join).
//!
//! Clean-room feature `specs/044-table-stack-innerjoin/`. Normative behaviour
//! comes only from the approved export `innerjoin` 0.2.0 (see that feature's
//! `provenance.md`).
//!
//! Several helpers below (`single_name`, `KeySpec`, `key_spec`,
//! `resolve_keys`, `Atom`, `key_atoms`, `normalize_variable`,
//! `is_simple_identifier`, `input_suffixes`, `make_names_unique`) are copied
//! from the clean-room F35 `joins.rs`, whose items are private to that file.
//! The duplication is deliberate: `joins.rs` is not edited by this feature.

use super::*;
use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::IntegerStorage;

use crate::{build_runtime_error, RuntimeError};

const NAME: &str = "innerjoin";

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

const TLEFT: BuiltinParamDescriptor = param("Tleft", BuiltinParamType::Any, "Left table.");
const TRIGHT: BuiltinParamDescriptor = param("Tright", BuiltinParamType::Any, "Right table.");

const INNERJOIN_OUTPUT: [BuiltinParamDescriptor; 1] =
    [param("T", BuiltinParamType::Any, "Joined table.")];

const BASIC_INPUTS: [BuiltinParamDescriptor; 2] = [TLEFT, TRIGHT];
const OPTION_INPUTS: [BuiltinParamDescriptor; 4] = [
    TLEFT,
    TRIGHT,
    param(
        "Name",
        BuiltinParamType::StringScalar,
        "'Keys', 'LeftKeys', 'RightKeys' or 'LeftVariables'.",
    ),
    param(
        "Value",
        BuiltinParamType::Any,
        "Variable name(s) or index(es).",
    ),
];

const INNERJOIN_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "T = innerjoin(Tleft, Tright)",
        inputs: &BASIC_INPUTS,
        outputs: &INNERJOIN_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "T = innerjoin(Tleft, Tright, Name, Value)",
        inputs: &OPTION_INPUTS,
        outputs: &INNERJOIN_OUTPUT,
    },
];

const INNERJOIN_ERRORS: [BuiltinErrorDescriptor; 6] = [
    BuiltinErrorDescriptor {
        code: "RM.INNERJOIN.INVALID_INPUT",
        identifier: Some("RunMat:innerjoin:InvalidInput"),
        when: "An input to join is not a table.",
        message: "innerjoin: expected two tables",
    },
    BuiltinErrorDescriptor {
        code: "RM.INNERJOIN.INVALID_ARGUMENT",
        identifier: Some("RunMat:innerjoin:InvalidArgument"),
        when: "A name-value argument is malformed.",
        message: "innerjoin: invalid argument",
    },
    BuiltinErrorDescriptor {
        code: "RM.INNERJOIN.UNSUPPORTED_OPTION",
        identifier: Some("RunMat:innerjoin:UnsupportedOption"),
        when: "The call uses a form, option or data that is not yet specified for RunMat.",
        message: "innerjoin: option not yet specified",
    },
    BuiltinErrorDescriptor {
        code: "RM.INNERJOIN.UNKNOWN_VARIABLE",
        identifier: Some("RunMat:innerjoin:UnknownVariable"),
        when: "A key or LeftVariables entry names a variable that does not exist.",
        message: "innerjoin: unknown variable",
    },
    BuiltinErrorDescriptor {
        code: "RM.INNERJOIN.CANT_INFER_KEY",
        identifier: Some("RunMat:table:join:CantInferKey"),
        when: "No variable name is common to both tables and no key options are given.",
        message: "innerjoin: cannot infer key variables",
    },
    BuiltinErrorDescriptor {
        code: "RM.INNERJOIN.CONFLICTING_INPUTS",
        identifier: Some("RunMat:innerjoin:ConflictingInputs"),
        when: "'Keys' is combined with 'LeftKeys' or 'RightKeys'.",
        message: "innerjoin: conflicting key options",
    },
];

const INVALID_INPUT: usize = 0;
const INVALID_ARGUMENT: usize = 1;
const UNSUPPORTED_OPTION: usize = 2;
const UNKNOWN_VARIABLE: usize = 3;
const CANT_INFER_KEY: usize = 4;
const CONFLICTING_INPUTS: usize = 5;

pub const INNERJOIN_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &INNERJOIN_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &INNERJOIN_ERRORS,
};

fn join_error(kind: usize, detail: impl Into<String>) -> RuntimeError {
    let mut builder = build_runtime_error(format!("{NAME}: {}", detail.into())).with_builtin(NAME);
    if let Some(identifier) = INNERJOIN_ERRORS[kind].identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

fn unsupported(detail: impl Into<String>) -> RuntimeError {
    join_error(UNSUPPORTED_OPTION, detail)
}

fn invalid_argument(detail: impl Into<String>) -> RuntimeError {
    join_error(INVALID_ARGUMENT, detail)
}

fn class_label(value: &Value) -> String {
    crate::builtins::introspection::class::class_name_for_value(value)
}

// ---------------------------------------------------------------------------
// Options (FR-044-35)
// ---------------------------------------------------------------------------

struct Options {
    keys: Option<Value>,
    left_keys: Option<Value>,
    right_keys: Option<Value>,
    left_variables: Option<Value>,
}

/// Copied from F35 `joins.rs`.
fn single_name(value: &Value) -> Option<String> {
    match value {
        Value::CharArray(chars) if chars.rows <= 1 => Some(chars.data.iter().collect()),
        Value::String(text) => Some(text.clone()),
        Value::StringArray(array) if array.data.len() == 1 => Some(array.data[0].clone()),
        _ => None,
    }
}

fn parse_options(rest: Vec<Value>) -> BuiltinResult<Options> {
    if !rest.len().is_multiple_of(2) {
        return Err(invalid_argument("name-value arguments must come in pairs"));
    }
    let mut options = Options {
        keys: None,
        left_keys: None,
        right_keys: None,
        left_variables: None,
    };
    let mut args = rest.into_iter();
    while let (Some(name), Some(value)) = (args.next(), args.next()) {
        let Some(option) = single_name(&name) else {
            return Err(invalid_argument("expected an option name"));
        };
        match option.to_ascii_lowercase().as_str() {
            "keys" => options.keys = Some(value),
            "leftkeys" => options.left_keys = Some(value),
            "rightkeys" => options.right_keys = Some(value),
            "leftvariables" => options.left_variables = Some(value),
            // Not in the export (FR-044-32).
            "rightvariables" => {
                return Err(unsupported(
                    "the 'RightVariables' option is not yet supported",
                ))
            }
            _ => return Err(invalid_argument(format!("unknown option '{option}'"))),
        }
    }
    Ok(options)
}

// ---------------------------------------------------------------------------
// Keys (copied from F35 `joins.rs`)
// ---------------------------------------------------------------------------

enum KeySpec {
    Names(Vec<String>),
    Indices(Vec<usize>),
}

fn key_spec(option: &str, value: &Value, allow_empty: bool) -> BuiltinResult<KeySpec> {
    if let Some(name) = single_name(value) {
        return Ok(KeySpec::Names(vec![name]));
    }
    let spec = match value {
        Value::Cell(cell) => KeySpec::Names(
            cell.data
                .iter()
                .map(|item| {
                    single_name(item).ok_or_else(|| {
                        invalid_argument(format!("'{option}' must list variable names"))
                    })
                })
                .collect::<BuiltinResult<_>>()?,
        ),
        Value::StringArray(array) => KeySpec::Names(array.data.clone()),
        Value::Bool(_) | Value::LogicalArray(_) => {
            return Err(unsupported(format!(
                "a logical '{option}' selector is not yet supported"
            )))
        }
        Value::Num(_) | Value::Int(_) | Value::Tensor(_) => {
            let numbers = match value {
                Value::Num(number) => vec![*number],
                Value::Int(int) => vec![int.to_f64()],
                Value::Tensor(tensor) => tensor.materialize_f64(),
                _ => unreachable!("numeric selector"),
            };
            KeySpec::Indices(
                numbers
                    .into_iter()
                    .map(|number| {
                        if number >= 1.0 && number.fract() == 0.0 && number.is_finite() {
                            Ok(number as usize - 1)
                        } else {
                            Err(invalid_argument(format!(
                                "'{option}' indices must be positive integers"
                            )))
                        }
                    })
                    .collect::<BuiltinResult<_>>()?,
            )
        }
        _ => {
            return Err(invalid_argument(format!(
                "'{option}' must be variable names or indices"
            )))
        }
    };
    let empty = match &spec {
        KeySpec::Names(names) => names.is_empty(),
        KeySpec::Indices(indices) => indices.is_empty(),
    };
    if empty && !allow_empty {
        return Err(invalid_argument(format!("'{option}' must not be empty")));
    }
    Ok(spec)
}

fn resolve_keys(option: &str, spec: &KeySpec, names: &[String]) -> BuiltinResult<Vec<usize>> {
    let positions: Vec<usize> = match spec {
        KeySpec::Names(keys) => keys
            .iter()
            .map(|key| {
                names.iter().position(|name| name == key).ok_or_else(|| {
                    join_error(
                        UNKNOWN_VARIABLE,
                        format!("'{option}': table has no variable named '{key}'"),
                    )
                })
            })
            .collect::<BuiltinResult<_>>()?,
        KeySpec::Indices(indices) => indices
            .iter()
            .map(|&index| {
                if index < names.len() {
                    Ok(index)
                } else {
                    Err(join_error(
                        UNKNOWN_VARIABLE,
                        format!("'{option}': table has no variable {}", index + 1),
                    ))
                }
            })
            .collect::<BuiltinResult<_>>()?,
    };
    let mut seen = HashSet::new();
    if positions.iter().any(|position| !seen.insert(*position)) {
        return Err(invalid_argument(format!(
            "'{option}' lists a variable more than once"
        )));
    }
    Ok(positions)
}

/// Key pairs `(left position, right position)` (FR-044-13, FR-044-18).
fn key_pairs(
    options: &Options,
    left_names: &[String],
    right_names: &[String],
) -> BuiltinResult<Vec<(usize, usize)>> {
    if options.keys.is_some() && (options.left_keys.is_some() || options.right_keys.is_some()) {
        return Err(join_error(
            CONFLICTING_INPUTS,
            "'Keys' cannot be combined with 'LeftKeys' or 'RightKeys'",
        ));
    }
    if let Some(keys) = &options.keys {
        let spec = key_spec("Keys", keys, false)?;
        let left = resolve_keys("Keys", &spec, left_names)?;
        let right = resolve_keys("Keys", &spec, right_names)?;
        for (&l, &r) in left.iter().zip(&right) {
            if left_names[l] != right_names[r] {
                return Err(invalid_argument(format!(
                    "'Keys' index {} names '{}' in the left table but '{}' in the right table",
                    l + 1,
                    left_names[l],
                    right_names[r]
                )));
            }
        }
        return Ok(left.into_iter().zip(right).collect());
    }
    match (&options.left_keys, &options.right_keys) {
        (Some(lk), Some(rk)) => {
            let left = resolve_keys("LeftKeys", &key_spec("LeftKeys", lk, false)?, left_names)?;
            let right = resolve_keys("RightKeys", &key_spec("RightKeys", rk, false)?, right_names)?;
            if left.len() != right.len() {
                return Err(invalid_argument(
                    "'LeftKeys' and 'RightKeys' must name the same number of variables",
                ));
            }
            Ok(left.into_iter().zip(right).collect())
        }
        (Some(_), None) | (None, Some(_)) => Err(invalid_argument(
            "'LeftKeys' and 'RightKeys' must be given together",
        )),
        (None, None) => {
            let pairs: Vec<(usize, usize)> = left_names
                .iter()
                .enumerate()
                .filter_map(|(l, name)| {
                    right_names
                        .iter()
                        .position(|candidate| candidate == name)
                        .map(|r| (l, r))
                })
                .collect();
            if pairs.is_empty() {
                return Err(join_error(
                    CANT_INFER_KEY,
                    "the tables have no variable name in common; specify the keys",
                ));
            }
            Ok(pairs)
        }
    }
}

/// One key value (copied from F35 `joins.rs`).
#[derive(Clone, Debug)]
enum Atom {
    Float(f64),
    Int(i128),
    Text(String),
}

impl Atom {
    fn rank(&self) -> u8 {
        match self {
            Atom::Float(_) => 0,
            Atom::Int(_) => 1,
            Atom::Text(_) => 2,
        }
    }
}

impl Ord for Atom {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Atom::Float(a), Atom::Float(b)) => a.total_cmp(b),
            (Atom::Int(a), Atom::Int(b)) => a.cmp(b),
            (Atom::Text(a), Atom::Text(b)) => a.cmp(b),
            _ => self.rank().cmp(&other.rank()),
        }
    }
}

impl PartialOrd for Atom {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Atom {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Atom {}

fn is_single_column(shape: &[usize]) -> bool {
    shape.len() <= 2 && (shape.get(1).copied().unwrap_or(1) == 1 || shape.first() == Some(&0))
}

fn integer_atoms(storage: &IntegerStorage) -> Vec<Atom> {
    fn widen<T: Copy + Into<i128>>(values: &[T]) -> Vec<Atom> {
        values.iter().map(|v| Atom::Int((*v).into())).collect()
    }
    match storage {
        IntegerStorage::I8(v) => widen(v),
        IntegerStorage::I16(v) => widen(v),
        IntegerStorage::I32(v) => widen(v),
        IntegerStorage::I64(v) => widen(v),
        IntegerStorage::U8(v) => widen(v),
        IntegerStorage::U16(v) => widen(v),
        IntegerStorage::U32(v) => widen(v),
        IntegerStorage::U64(v) => widen(v),
    }
}

/// Per-row key atoms for one key variable (FR-044-33).
fn key_atoms(name: &str, value: &Value) -> BuiltinResult<Vec<Atom>> {
    let missing = |what: &str| {
        unsupported(format!(
            "key variable '{name}' contains {what}; missing key values are not yet supported"
        ))
    };
    let atoms = match value {
        Value::Tensor(tensor) if is_single_column(&tensor.shape) => {
            match tensor.integer_storage() {
                Some(storage) => integer_atoms(storage),
                None => tensor
                    .materialize_f64()
                    .into_iter()
                    .map(|v| {
                        if v.is_nan() {
                            Err(missing("NaN"))
                        } else {
                            // -0 and +0 are the same key.
                            Ok(Atom::Float(if v == 0.0 { 0.0 } else { v }))
                        }
                    })
                    .collect::<BuiltinResult<_>>()?,
            }
        }
        Value::LogicalArray(array) if is_single_column(&array.shape) => array
            .data
            .iter()
            .map(|v| Atom::Int(i128::from(*v != 0)))
            .collect(),
        Value::StringArray(array) if is_single_column(&array.shape) => array
            .data
            .iter()
            .map(|text| {
                if crate::builtins::strings::common::is_missing_string(text) {
                    Err(missing("<missing>"))
                } else {
                    Ok(Atom::Text(text.clone()))
                }
            })
            .collect::<BuiltinResult<_>>()?,
        Value::Cell(cell) if is_single_column(&cell.shape) => cell
            .data
            .iter()
            .map(|item| match item {
                Value::CharArray(chars) if chars.rows <= 1 => {
                    Ok(Atom::Text(chars.data.iter().collect()))
                }
                _ => Err(unsupported(format!(
                    "key variable '{name}' is a cell array that is not a cellstr column"
                ))),
            })
            .collect::<BuiltinResult<_>>()?,
        other => {
            return Err(unsupported(format!(
                "key variable '{name}' of class {} (or with several columns) is not yet supported",
                class_label(other)
            )))
        }
    };
    Ok(atoms)
}

// ---------------------------------------------------------------------------
// Names (copied from F35 `joins.rs`)
// ---------------------------------------------------------------------------

fn is_simple_identifier(text: &str) -> bool {
    let mut chars = text.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_alphabetic() || first == '_')
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

/// Suffixes for clashing names (FR-044-15, FR-044-34): the caller's argument
/// names when the recorded call site can be trusted to be this call,
/// otherwise `left`/`right`.
fn input_suffixes(arg_count: usize) -> (String, String) {
    let fallback = ("left".to_string(), "right".to_string());
    let Some(callsite) = crate::callsite::current_callsite() else {
        return fallback;
    };
    if callsite.arg_spans.len() != arg_count {
        return fallback;
    }
    let name = |index| {
        crate::callsite::arg_text(index)
            .map(|text| text.trim().to_string())
            .filter(|text| is_simple_identifier(text))
    };
    match (name(0), name(1)) {
        (Some(left), Some(right)) if left != right => (left, right),
        _ => fallback,
    }
}

/// Case-sensitive uniqueness: later duplicates get `_2`, `_3`, ... (FR-044-36).
fn make_names_unique(names: Vec<String>) -> Vec<String> {
    let originals: HashSet<String> = names.iter().cloned().collect();
    let mut used = HashSet::new();
    let mut out = Vec::with_capacity(names.len());
    for name in names {
        let mut candidate = name.clone();
        let mut counter = 1usize;
        while used.contains(&candidate) || (counter > 1 && originals.contains(&candidate)) {
            counter += 1;
            candidate = format!("{name}_{counter}");
        }
        used.insert(candidate.clone());
        out.push(candidate);
    }
    out
}

// ---------------------------------------------------------------------------
// Builtin
// ---------------------------------------------------------------------------

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

async fn table_input(value: Value, side: &str) -> BuiltinResult<ObjectInstance> {
    let host = gather_if_needed_async(&value)
        .await
        .map_err(map_control_flow)?;
    match host {
        Value::Object(object) if object.class_name.is(TABLE_CLASS) => Ok(object),
        Value::Object(object) if object.class_name.is(TIMETABLE_CLASS) => {
            Err(unsupported("timetable inputs are not yet supported"))
        }
        other => Err(join_error(
            INVALID_INPUT,
            format!("{side} input must be a table, got {}", class_label(&other)),
        )),
    }
}

async fn table_columns(object: &ObjectInstance) -> BuiltinResult<(Vec<String>, Vec<Value>)> {
    let variables = table_variables(object)?;
    let mut names = Vec::with_capacity(variables.fields.len());
    let mut columns = Vec::with_capacity(variables.fields.len());
    for (name, value) in variables.fields {
        let host = gather_if_needed_async(&value)
            .await
            .map_err(map_control_flow)?;
        names.push(name);
        columns.push(normalize_variable(host)?);
    }
    Ok((names, columns))
}

#[runtime_builtin(
    name = "innerjoin",
    category = "table",
    summary = "Join two tables on key variables, keeping only rows whose keys appear in both.",
    keywords = "innerjoin,join,table,merge,keys,inner join",
    accel = "cpu",
    descriptor(crate::builtins::table::builtins::innerjoin::INNERJOIN_DESCRIPTOR),
    builtin_path = "crate::builtins::table::builtins"
)]
pub(crate) async fn innerjoin_builtin(
    left: Value,
    right: Value,
    rest: Vec<Value>,
) -> BuiltinResult<Value> {
    let arg_count = 2 + rest.len();
    let left = table_input(left, "left").await?;
    let right = table_input(right, "right").await?;
    // innerjoin.q-index-outputs
    if crate::output_count::current_output_count().is_some_and(|count| count > 1) {
        return Err(unsupported(
            "the row-index outputs [T, ileft, iright] are not yet supported",
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
    let options = parse_options(host_rest)?;

    let (left_names, left_columns) = table_columns(&left).await?;
    let (right_names, right_columns) = table_columns(&right).await?;
    let pairs = key_pairs(&options, &left_names, &right_names)?;

    // Key atoms per pair, with class checks (FR-044-33).
    let mut left_atoms = Vec::with_capacity(pairs.len());
    let mut right_atoms = Vec::with_capacity(pairs.len());
    for &(l, r) in &pairs {
        let (lcol, rcol) = (&left_columns[l], &right_columns[r]);
        let (lclass, rclass) = (class_label(lcol), class_label(rcol));
        if lclass != rclass {
            return Err(unsupported(format!(
                "joining key '{}' ({lclass}) with key '{}' ({rclass}) is not yet supported",
                left_names[l], right_names[r]
            )));
        }
        left_atoms.push(key_atoms(&left_names[l], lcol)?);
        right_atoms.push(key_atoms(&right_names[r], rcol)?);
    }
    let left_height = table_height(&left)?;
    let right_height = table_height(&right)?;
    let tuple = |atoms: &[Vec<Atom>], row: usize| -> Vec<Atom> {
        atoms.iter().map(|column| column[row].clone()).collect()
    };
    let left_keys: Vec<Vec<Atom>> = (0..left_height).map(|i| tuple(&left_atoms, i)).collect();
    let right_keys: Vec<Vec<Atom>> = (0..right_height).map(|j| tuple(&right_atoms, j)).collect();

    // Equi-join (FR-044-11, FR-044-14; U-044-9).
    let mut right_rows: BTreeMap<&[Atom], Vec<usize>> = BTreeMap::new();
    for (j, key) in right_keys.iter().enumerate() {
        right_rows.entry(key.as_slice()).or_default().push(j);
    }
    let mut left_counts: BTreeMap<&[Atom], usize> = BTreeMap::new();
    for key in &left_keys {
        *left_counts.entry(key.as_slice()).or_default() += 1;
    }
    // innerjoin.q-many-to-many
    if right_rows
        .iter()
        .any(|(key, rows)| rows.len() > 1 && left_counts.get(key).is_some_and(|n| *n > 1))
    {
        return Err(unsupported(
            "a key value that occurs more than once in both tables is not yet supported",
        ));
    }
    let mut rows: Vec<(&[Atom], usize, usize)> = Vec::new();
    for (i, key) in left_keys.iter().enumerate() {
        if let Some(matches) = right_rows.get(key.as_slice()) {
            for &j in matches {
                rows.push((key.as_slice(), i, j));
            }
        }
    }
    rows.sort_by(|a, b| a.0.cmp(b.0)); // stable: left order for equal keys

    // Layout (FR-044-12, FR-044-17; U-044-7, U-044-8).
    let selected_left: Vec<usize> = match &options.left_variables {
        None => (0..left_names.len()).collect(),
        Some(value) => resolve_keys(
            "LeftVariables",
            &key_spec("LeftVariables", value, true)?,
            &left_names,
        )?,
    };
    let mut sources: Vec<(String, bool, usize)> = Vec::new(); // (name, from left, column)
    for &(l, _) in &pairs {
        if selected_left.contains(&l) {
            sources.push((left_names[l].clone(), true, l));
        }
    }
    for (l, name) in left_names.iter().enumerate() {
        if selected_left.contains(&l) && !pairs.iter().any(|&(pl, _)| pl == l) {
            sources.push((name.clone(), true, l));
        }
    }
    for (r, name) in right_names.iter().enumerate() {
        if !pairs.iter().any(|&(_, pr)| pr == r) {
            sources.push((name.clone(), false, r));
        }
    }
    if sources.is_empty() {
        // D-034-1: a table with rows but no variables cannot be represented.
        return Err(unsupported(
            "a join result with no variables is not yet supported",
        ));
    }

    // Names (FR-044-15, FR-044-34, FR-044-36).
    let left_side: HashSet<&str> = sources
        .iter()
        .filter(|(_, is_left, _)| *is_left)
        .map(|(name, _, _)| name.as_str())
        .collect();
    let right_side: HashSet<&str> = sources
        .iter()
        .filter(|(_, is_left, _)| !*is_left)
        .map(|(name, _, _)| name.as_str())
        .collect();
    let clashing: HashSet<String> = left_side
        .intersection(&right_side)
        .map(|name| name.to_string())
        .collect();
    let (left_suffix, right_suffix) = if clashing.is_empty() {
        (String::new(), String::new())
    } else {
        input_suffixes(arg_count)
    };
    let out_names = make_names_unique(
        sources
            .iter()
            .map(|(name, is_left, _)| {
                if clashing.contains(name) {
                    let suffix = if *is_left {
                        &left_suffix
                    } else {
                        &right_suffix
                    };
                    format!("{name}_{suffix}")
                } else {
                    name.clone()
                }
            })
            .collect(),
    );

    // Values.
    let left_picks: Vec<usize> = rows.iter().map(|(_, i, _)| *i).collect();
    let right_picks: Vec<usize> = rows.iter().map(|(_, _, j)| *j).collect();
    let mut out_columns = Vec::with_capacity(sources.len());
    for (_, is_left, column) in &sources {
        out_columns.push(if *is_left {
            select_rows(&left_columns[*column], &left_picks)?
        } else {
            select_rows(&right_columns[*column], &right_picks)?
        });
    }
    table_from_columns(out_names, out_columns)
}

#[cfg(test)]
mod table_innerjoin_tests {
    use super::*;
    use futures::executor::block_on;
    use runmat_types::{SourceId, Span};
    use runmat_value::{CellArray, CharArray, IntegerStorage, LogicalArray, StringArray, Tensor};

    const NAN: f64 = f64::NAN;

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

    fn table(names: &[&str], columns: Vec<Value>) -> Value {
        table_from_columns(names.iter().map(|n| n.to_string()).collect(), columns).unwrap()
    }

    fn kv(names: [&str; 2], keys: &[f64], values: &[f64]) -> Value {
        table(&names, vec![col(keys), col(values)])
    }

    /// The observation set-up tables (left keys {1,2,4}, right keys {4,1,3}).
    fn l_table() -> Value {
        kv(["k", "a"], &[1.0, 2.0, 4.0], &[10.0, 20.0, 40.0])
    }

    fn r_table() -> Value {
        kv(["k", "b"], &[4.0, 1.0, 3.0], &[400.0, 100.0, 300.0])
    }

    // ---- Calls ------------------------------------------------------------

    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("innerjoin", args))
    }

    /// Call `innerjoin` as if from the source line `source`, so the call site
    /// records the text of each argument the way the interpreter does
    /// (helper copied from the F35 `outerjoin` tests).
    fn call_at(source: &str, args: &[Value]) -> BuiltinResult<Value> {
        let open = source.find('(').expect("call has '('");
        let close = source.rfind(')').expect("call has ')'");
        let mut spans = Vec::new();
        let mut start = open + 1;
        for (offset, ch) in source[open + 1..close].char_indices() {
            if ch == ',' {
                spans.push(Span {
                    start,
                    end: open + 1 + offset,
                });
                start = open + 1 + offset + 1;
            }
        }
        spans.push(Span { start, end: close });
        let _catalog = crate::source_context::replace_source_catalog(vec![(
            SourceId(9044),
            "/tmp/innerjoin_caller.m".to_string(),
            source.to_string(),
        )]);
        let _callsite = crate::callsite::push_callsite(Some(SourceId(9044)), Some(spans));
        call(args)
    }

    fn identifier(err: crate::RuntimeError) -> String {
        err.identifier().unwrap_or("<none>").to_string()
    }

    fn err_id(args: &[Value]) -> String {
        identifier(call(args).expect_err("innerjoin should fail"))
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
        assert_eq!(
            format!("{got:?}"),
            format!("{expected:?}"),
            "values of {name}"
        );
        assert_eq!(value_row_count(&column).unwrap(), expected.len());
    }

    /// Class table, names in order, size, and every column's values.
    fn assert_observed(value: &Value, expected: &[(&str, &[f64])]) {
        assert_eq!(class_of(value), "table");
        let expected_names: Vec<&str> = expected.iter().map(|(n, _)| *n).collect();
        assert_eq!(names(value), expected_names);
        let rows = expected[0].1.len();
        assert_eq!(size(value), (rows, expected.len()));
        for (name, values) in expected {
            assert_doubles(value, name, values);
        }
    }

    // ---- Observed (normative): innerjoin 0.2.0 cases ----------------------

    /// [innerjoin.signature-primary, innerjoin.output-class, innerjoin.rows,
    /// innerjoin.single-key] Case `basic`.
    #[test]
    fn observed_basic() {
        let out = call(&[l_table(), r_table()]).expect("innerjoin");
        assert_observed(
            &out,
            &[
                ("k", &[1.0, 4.0]),
                ("a", &[10.0, 40.0]),
                ("b", &[100.0, 400.0]),
            ],
        );
    }

    /// [innerjoin.keys, innerjoin.single-key] Case `keys`.
    #[test]
    fn observed_keys() {
        let out = call(&[l_table(), r_table(), text("Keys"), text("k")]).expect("innerjoin");
        assert_observed(
            &out,
            &[
                ("k", &[1.0, 4.0]),
                ("a", &[10.0, 40.0]),
                ("b", &[100.0, 400.0]),
            ],
        );
    }

    /// [innerjoin.keys] Case `leftright-keys`.
    #[test]
    fn observed_leftright_keys() {
        let out = call(&[
            l_table(),
            kv(["id", "b"], &[4.0, 1.0], &[400.0, 100.0]),
            text("LeftKeys"),
            text("k"),
            text("RightKeys"),
            text("id"),
        ])
        .expect("innerjoin");
        assert_observed(
            &out,
            &[
                ("k", &[1.0, 4.0]),
                ("a", &[10.0, 40.0]),
                ("b", &[100.0, 400.0]),
            ],
        );
    }

    /// [innerjoin.row-order] Case `unsorted-left`.
    #[test]
    fn observed_unsorted_left() {
        let out = call(&[
            kv(["k", "a"], &[4.0, 1.0, 2.0], &[40.0, 10.0, 20.0]),
            r_table(),
        ])
        .expect("innerjoin");
        assert_observed(
            &out,
            &[
                ("k", &[1.0, 4.0]),
                ("a", &[10.0, 40.0]),
                ("b", &[100.0, 400.0]),
            ],
        );
    }

    /// [innerjoin.row-order] Case `unsorted-both`.
    #[test]
    fn observed_unsorted_both() {
        let out = call(&[
            kv(["k", "a"], &[3.0, 1.0, 2.0], &[30.0, 10.0, 20.0]),
            kv(["k", "b"], &[2.0, 3.0, 1.0], &[200.0, 300.0, 100.0]),
        ])
        .expect("innerjoin");
        assert_observed(
            &out,
            &[
                ("k", &[1.0, 2.0, 3.0]),
                ("a", &[10.0, 20.0, 30.0]),
                ("b", &[100.0, 200.0, 300.0]),
            ],
        );
    }

    /// [innerjoin.duplicates] Case `dup-left`.
    #[test]
    fn observed_duplicate_left_keys_keep_left_order() {
        let out = call(&[
            kv(["k", "a"], &[1.0, 1.0, 4.0], &[11.0, 10.0, 40.0]),
            r_table(),
        ])
        .expect("innerjoin");
        assert_observed(
            &out,
            &[
                ("k", &[1.0, 1.0, 4.0]),
                ("a", &[11.0, 10.0, 40.0]),
                ("b", &[100.0, 100.0, 400.0]),
            ],
        );
    }

    fn v_left() -> Value {
        kv(["k", "v"], &[1.0, 2.0], &[10.0, 20.0])
    }

    fn v_right() -> Value {
        kv(["k", "v"], &[2.0, 1.0], &[200.0, 100.0])
    }

    /// [innerjoin.name-suffix] Case `same-nonkey`: expression inputs.
    #[test]
    fn observed_same_nonkey_unnamed() {
        let expected: [(&str, &[f64]); 3] = [
            ("k", &[1.0, 2.0]),
            ("v_left", &[10.0, 20.0]),
            ("v_right", &[100.0, 200.0]),
        ];
        let args = [v_left(), v_right(), text("Keys"), text("k")];
        let out = call_at(
            "innerjoin(table([1; 2], [10; 20], 'VariableNames', {'k';'v'}), \
             table([2; 1], [200; 100], 'VariableNames', {'k';'v'}), 'Keys', 'k')",
            &args,
        );
        // The comma-splitting helper cannot split nested argument lists, so
        // the recorded span count differs from the call's: fallback too.
        assert_observed(&out.expect("innerjoin"), &expected);
        let out = call_at("innerjoin(tl(), tr(), 'Keys', 'k')", &args).expect("innerjoin");
        assert_observed(&out, &expected);
        let out = call(&args).expect("innerjoin");
        assert_observed(&out, &expected);
    }

    /// [innerjoin.name-suffix] Case `same-nonkey-named`.
    #[test]
    fn observed_same_nonkey_named() {
        let out = call_at(
            "innerjoin(Lw, Rw, 'Keys', 'k')",
            &[v_left(), v_right(), text("Keys"), text("k")],
        )
        .expect("innerjoin");
        assert_observed(
            &out,
            &[
                ("k", &[1.0, 2.0]),
                ("v_Lw", &[10.0, 20.0]),
                ("v_Rw", &[100.0, 200.0]),
            ],
        );
    }

    /// [innerjoin.no-match, innerjoin.output-class] Case `no-match`.
    #[test]
    fn observed_no_match() {
        let out = call(&[
            kv(["k", "a"], &[1.0, 2.0], &[10.0, 20.0]),
            kv(["k", "b"], &[3.0, 4.0], &[30.0, 40.0]),
        ])
        .expect("innerjoin");
        assert_eq!(class_of(&out), "table");
        assert_eq!(names(&out), ["k", "a", "b"]);
        assert_eq!(size(&out), (0, 3));
        for name in ["k", "a", "b"] {
            let column = var(&out, name);
            assert_eq!(class_of(&column), "double");
            match column {
                Value::Tensor(t) => assert_eq!(t.shape, [0, 1], "{name}"),
                other => panic!("{other:?}"),
            }
        }
    }

    /// [innerjoin.left-variables] Case `left-variables`.
    #[test]
    fn observed_left_variables() {
        let out =
            call(&[l_table(), r_table(), text("LeftVariables"), text("k")]).expect("innerjoin");
        assert_observed(&out, &[("k", &[1.0, 4.0]), ("b", &[100.0, 400.0])]);
    }

    /// [innerjoin.error-no-common] Case `err-no-common`.
    #[test]
    fn observed_error_no_common_variable() {
        let id = err_id(&[
            table(&["p"], vec![col(&[1.0])]),
            table(&["q"], vec![col(&[1.0])]),
        ]);
        assert_eq!(id, "RunMat:table:join:CantInferKey");
    }

    // ---- Unresolved questions (FR-044-23, 24, 34) -------------------------

    /// innerjoin.q-index-outputs.
    #[test]
    fn unresolved_index_outputs_unsupported() {
        for count in [2, 3] {
            let _outputs = crate::output_count::push_output_count(Some(count));
            let err = block_on(innerjoin_builtin(l_table(), r_table(), Vec::new()))
                .expect_err("index outputs");
            assert_eq!(
                identifier(err),
                "RunMat:innerjoin:UnsupportedOption",
                "nargout {count}"
            );
        }
        let _outputs = crate::output_count::push_output_count(Some(1));
        assert!(block_on(innerjoin_builtin(l_table(), r_table(), Vec::new())).is_ok());
    }

    /// innerjoin.q-many-to-many.
    #[test]
    fn unresolved_many_to_many_unsupported() {
        let id = err_id(&[
            kv(["k", "a"], &[1.0, 1.0], &[10.0, 11.0]),
            kv(["k", "b"], &[1.0, 1.0], &[100.0, 101.0]),
        ]);
        assert_eq!(id, "RunMat:innerjoin:UnsupportedOption");
    }

    /// innerjoin.q-name-suffix-context: no usable caller names give
    /// _left/_right.
    #[test]
    fn unresolved_name_suffix_falls_back_to_left_right() {
        let expected = ["k", "v_left", "v_right"];
        let args = [v_left(), v_right(), text("Keys"), text("k")];
        let out = call_at("innerjoin(tables{1}, R, 'Keys', 'k')", &args).unwrap();
        assert_eq!(names(&out), expected, "expression argument");
        let out = call_at("innerjoin(T, T, 'Keys', 'k')", &args).unwrap();
        assert_eq!(names(&out), expected, "same name twice");
        let out = call_at("feval(fn, L, R, 'Keys', 'k')", &args).unwrap();
        assert_eq!(names(&out), expected, "call site of another call");
        let out = call_at("innerjoin(L, R, 'Keys', 'k')", &args).unwrap();
        assert_eq!(names(&out), ["k", "v_L", "v_R"], "named inputs");
    }

    // ---- Documented choices (FR-044-31..36) -------------------------------

    /// U-044-7: keys first (key order), then the other left variables in
    /// table order, then the right non-key variables.
    #[test]
    fn documented_choice_key_placement() {
        let left = table(&["a", "k"], vec![col(&[10.0, 20.0]), col(&[2.0, 1.0])]);
        let right = table(&["b", "k"], vec![col(&[100.0, 200.0]), col(&[1.0, 2.0])]);
        let out = call(&[left, right]).expect("innerjoin");
        assert_observed(
            &out,
            &[
                ("k", &[1.0, 2.0]),
                ("a", &[20.0, 10.0]),
                ("b", &[100.0, 200.0]),
            ],
        );
        // Two keys: rows sort by the first key then the second.
        let left = table(
            &["k1", "k2", "a"],
            vec![
                col(&[2.0, 1.0, 1.0]),
                col(&[1.0, 2.0, 1.0]),
                col(&[1.0, 2.0, 3.0]),
            ],
        );
        let right = table(
            &["k2", "k1", "b"],
            vec![
                col(&[1.0, 2.0, 1.0]),
                col(&[1.0, 1.0, 2.0]),
                col(&[7.0, 8.0, 9.0]),
            ],
        );
        let out = call(&[left, right]).expect("innerjoin");
        assert_observed(
            &out,
            &[
                ("k1", &[1.0, 1.0, 2.0]),
                ("k2", &[1.0, 2.0, 1.0]),
                ("a", &[3.0, 2.0, 1.0]),
                ("b", &[7.0, 8.0, 9.0]),
            ],
        );
    }

    /// U-044-8: LeftVariables forms; RightVariables is unsupported.
    #[test]
    fn documented_choice_left_variables() {
        let out =
            call(&[l_table(), r_table(), text("LeftVariables"), text("a")]).expect("innerjoin");
        assert_observed(&out, &[("a", &[10.0, 40.0]), ("b", &[100.0, 400.0])]);
        let out = call(&[l_table(), r_table(), text("leftvariables"), Value::Num(2.0)])
            .expect("innerjoin");
        assert_eq!(names(&out), ["a", "b"]);
        assert_eq!(
            err_id(&[l_table(), r_table(), text("LeftVariables"), text("nope")]),
            "RunMat:innerjoin:UnknownVariable"
        );
        assert_eq!(
            err_id(&[l_table(), r_table(), text("RightVariables"), text("b")]),
            "RunMat:innerjoin:UnsupportedOption"
        );
        // A selection that would leave no variable at all (D-034-1).
        assert_eq!(
            err_id(&[
                kv(["k", "a"], &[1.0, 2.0], &[1.0, 2.0]),
                table(&["k"], vec![col(&[1.0])]),
                text("LeftVariables"),
                Value::Cell(CellArray::new(Vec::new(), 0, 0).unwrap()),
            ]),
            "RunMat:innerjoin:UnsupportedOption"
        );
    }

    /// U-044-9: key classes, right-only duplicates, NaN keys.
    #[test]
    fn documented_choice_key_classes() {
        let out = call(&[
            table(&["k", "a"], vec![string_col(&["y", "x"]), col(&[1.0, 2.0])]),
            table(&["k", "b"], vec![string_col(&["x", "y"]), col(&[3.0, 4.0])]),
        ])
        .expect("string keys");
        match var(&out, "k") {
            Value::StringArray(array) => assert_eq!(array.data, ["x", "y"]),
            other => panic!("{other:?}"),
        }
        assert_doubles(&out, "a", &[2.0, 1.0]);
        let out = call(&[
            table(
                &["k", "a"],
                vec![cellstr_col(&["y", "x"]), col(&[1.0, 2.0])],
            ),
            table(
                &["k", "b"],
                vec![cellstr_col(&["x", "z"]), col(&[3.0, 4.0])],
            ),
        ])
        .expect("cellstr keys");
        assert_doubles(&out, "a", &[2.0]);
        let out = call(&[
            table(&["k", "a"], vec![int32_col(&[3, 1]), col(&[1.0, 2.0])]),
            table(&["k", "b"], vec![int32_col(&[1, 3]), col(&[3.0, 4.0])]),
        ])
        .expect("int32 keys");
        assert_eq!(class_of(&var(&out, "k")), "int32");
        assert_doubles(&out, "a", &[2.0, 1.0]);
        let out = call(&[
            table(
                &["k", "a"],
                vec![logical_col(&[true, false]), col(&[1.0, 2.0])],
            ),
            table(&["k", "b"], vec![logical_col(&[true]), col(&[3.0])]),
        ])
        .expect("logical keys");
        assert_doubles(&out, "a", &[1.0]);
        // Right-only duplicates follow right-input order.
        let out = call(&[
            kv(["k", "a"], &[1.0], &[10.0]),
            kv(["k", "b"], &[1.0, 1.0], &[101.0, 100.0]),
        ])
        .expect("right duplicates");
        assert_observed(
            &out,
            &[
                ("k", &[1.0, 1.0]),
                ("a", &[10.0, 10.0]),
                ("b", &[101.0, 100.0]),
            ],
        );
        // Unsupported key classes and missing keys.
        let unsupported = "RunMat:innerjoin:UnsupportedOption";
        assert_eq!(
            err_id(&[
                kv(["k", "a"], &[1.0], &[1.0]),
                table(&["k", "b"], vec![int32_col(&[1]), col(&[1.0])]),
            ]),
            unsupported
        );
        assert_eq!(
            err_id(&[kv(["k", "a"], &[NAN, 1.0], &[1.0, 2.0]), r_table()]),
            unsupported
        );
        assert_eq!(
            err_id(&[
                table(&["k", "a"], vec![string_col(&["<missing>"]), col(&[1.0])]),
                table(&["k", "b"], vec![string_col(&["x"]), col(&[1.0])]),
            ]),
            unsupported
        );
    }

    /// U-044-10: option forms and malformed options.
    #[test]
    fn rejects_malformed_options() {
        let invalid = "RunMat:innerjoin:InvalidArgument";
        let out = call(&[
            l_table(),
            r_table(),
            Value::String("keys".into()),
            Value::Num(1.0),
        ])
        .expect("index key");
        assert_eq!(names(&out), ["k", "a", "b"]);
        assert_eq!(err_id(&[l_table(), r_table(), text("Keys")]), invalid);
        assert_eq!(
            err_id(&[l_table(), r_table(), text("Bogus"), text("k")]),
            invalid
        );
        assert_eq!(
            err_id(&[l_table(), r_table(), text("LeftKeys"), text("k")]),
            invalid
        );
        assert_eq!(
            err_id(&[l_table(), r_table(), text("Keys"), Value::Num(2.0)]),
            invalid
        );
        assert_eq!(
            err_id(&[
                l_table(),
                r_table(),
                text("Keys"),
                text("k"),
                text("LeftKeys"),
                text("k")
            ]),
            "RunMat:innerjoin:ConflictingInputs"
        );
        assert_eq!(
            err_id(&[l_table(), r_table(), text("Keys"), text("zz")]),
            "RunMat:innerjoin:UnknownVariable"
        );
    }

    /// U-044-11: non-table inputs; leftover collisions get _2.
    #[test]
    fn rejects_non_table_inputs_and_uniquifies_names() {
        assert_eq!(
            err_id(&[col(&[1.0]), r_table()]),
            "RunMat:innerjoin:InvalidInput"
        );
        assert_eq!(
            err_id(&[l_table(), Value::Num(1.0)]),
            "RunMat:innerjoin:InvalidInput"
        );
        // Left has `v_left` already: the suffixed `v` collides with it.
        let left = table(
            &["k", "v", "v_left"],
            vec![col(&[1.0]), col(&[2.0]), col(&[3.0])],
        );
        let right = kv(["k", "v"], &[1.0], &[4.0]);
        let out = call(&[left, right, text("Keys"), text("k")]).expect("innerjoin");
        assert_eq!(names(&out), ["k", "v_left", "v_left_2", "v_right"]);
    }
}
