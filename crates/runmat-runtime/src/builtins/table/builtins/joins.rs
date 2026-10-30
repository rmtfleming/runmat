//! Clean-room `outerjoin` (table joins).
//!
//! Clean-room feature `specs/035-outerjoin/`. Normative behaviour comes only
//! from the approved export `outerjoin` 0.3.0 (see that feature's
//! `provenance.md`). The export's unresolved questions raise
//! `RunMat:outerjoin:UnsupportedOption` or follow a documented RunMat choice
//! (spec.md FR-035-13..22).

use super::*;
use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{IntegerStorage, NumericStorage};

use crate::{build_runtime_error, RuntimeError};

const NAME: &str = "outerjoin";

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

const OUTERJOIN_OUTPUT: [BuiltinParamDescriptor; 1] =
    [param("T", BuiltinParamType::Any, "Joined table.")];

const BASIC_INPUTS: [BuiltinParamDescriptor; 2] = [TLEFT, TRIGHT];
const KEYS_INPUTS: [BuiltinParamDescriptor; 4] = [
    TLEFT,
    TRIGHT,
    param("'Keys'", BuiltinParamType::StringScalar, "Option name."),
    param(
        "keys",
        BuiltinParamType::Any,
        "Key variable(s) common to both tables, by name or index.",
    ),
];
const LEFT_RIGHT_KEYS_INPUTS: [BuiltinParamDescriptor; 6] = [
    TLEFT,
    TRIGHT,
    param("'LeftKeys'", BuiltinParamType::StringScalar, "Option name."),
    param("lk", BuiltinParamType::Any, "Key variable(s) of Tleft."),
    param(
        "'RightKeys'",
        BuiltinParamType::StringScalar,
        "Option name.",
    ),
    param("rk", BuiltinParamType::Any, "Key variable(s) of Tright."),
];
const MERGE_KEYS_INPUTS: [BuiltinParamDescriptor; 4] = [
    TLEFT,
    TRIGHT,
    param(
        "'MergeKeys'",
        BuiltinParamType::StringScalar,
        "Option name.",
    ),
    param(
        "tf",
        BuiltinParamType::Any,
        "Logical; true merges each key pair into one variable.",
    ),
];
const TYPE_INPUTS: [BuiltinParamDescriptor; 4] = [
    TLEFT,
    TRIGHT,
    param("'Type'", BuiltinParamType::StringScalar, "Option name."),
    param(
        "type",
        BuiltinParamType::StringScalar,
        "'full' (default), 'left', 'right' or 'inner'.",
    ),
];

const OUTERJOIN_SIGNATURES: [BuiltinSignatureDescriptor; 5] = [
    BuiltinSignatureDescriptor {
        label: "T = outerjoin(Tleft, Tright)",
        inputs: &BASIC_INPUTS,
        outputs: &OUTERJOIN_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "T = outerjoin(Tleft, Tright, 'Keys', keys)",
        inputs: &KEYS_INPUTS,
        outputs: &OUTERJOIN_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "T = outerjoin(Tleft, Tright, 'LeftKeys', lk, 'RightKeys', rk)",
        inputs: &LEFT_RIGHT_KEYS_INPUTS,
        outputs: &OUTERJOIN_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "T = outerjoin(___, 'MergeKeys', tf)",
        inputs: &MERGE_KEYS_INPUTS,
        outputs: &OUTERJOIN_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "T = outerjoin(___, 'Type', type)",
        inputs: &TYPE_INPUTS,
        outputs: &OUTERJOIN_OUTPUT,
    },
];

const OUTERJOIN_ERRORS: [BuiltinErrorDescriptor; 7] = [
    BuiltinErrorDescriptor {
        code: "RM.OUTERJOIN.INVALID_INPUT",
        identifier: Some("RunMat:outerjoin:InvalidInput"),
        when: "An input to join is not a table.",
        message: "outerjoin: expected two tables",
    },
    BuiltinErrorDescriptor {
        code: "RM.OUTERJOIN.INVALID_ARGUMENT",
        identifier: Some("RunMat:outerjoin:InvalidArgument"),
        when: "A name-value argument is malformed.",
        message: "outerjoin: invalid argument",
    },
    BuiltinErrorDescriptor {
        code: "RM.OUTERJOIN.UNSUPPORTED_OPTION",
        identifier: Some("RunMat:outerjoin:UnsupportedOption"),
        when: "The call uses a form, option or data that is not yet specified for RunMat.",
        message: "outerjoin: option not yet specified",
    },
    BuiltinErrorDescriptor {
        code: "RM.OUTERJOIN.UNKNOWN_VARIABLE",
        identifier: Some("RunMat:outerjoin:UnknownVariable"),
        when: "A key names a variable that does not exist.",
        message: "outerjoin: unknown variable",
    },
    BuiltinErrorDescriptor {
        code: "RM.OUTERJOIN.CANT_INFER_KEY",
        identifier: Some("RunMat:outerjoin:CantInferKey"),
        when: "No variable name is common to both tables and no key options are given.",
        message: "outerjoin: cannot infer key variables",
    },
    BuiltinErrorDescriptor {
        code: "RM.OUTERJOIN.INVALID_TYPE",
        identifier: Some("RunMat:outerjoin:InvalidType"),
        when: "'Type' is not 'full', 'left', 'right' or 'inner'.",
        message: "outerjoin: invalid join type",
    },
    BuiltinErrorDescriptor {
        code: "RM.OUTERJOIN.CONFLICTING_INPUTS",
        identifier: Some("RunMat:outerjoin:ConflictingInputs"),
        when: "'Keys' is combined with 'LeftKeys' or 'RightKeys'.",
        message: "outerjoin: conflicting key options",
    },
];

const INVALID_INPUT: usize = 0;
const INVALID_ARGUMENT: usize = 1;
const UNSUPPORTED_OPTION: usize = 2;
const UNKNOWN_VARIABLE: usize = 3;
const CANT_INFER_KEY: usize = 4;
const INVALID_TYPE: usize = 5;
const CONFLICTING_INPUTS: usize = 6;

pub const OUTERJOIN_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &OUTERJOIN_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &OUTERJOIN_ERRORS,
};

fn join_error(kind: usize, detail: impl Into<String>) -> RuntimeError {
    let mut builder = build_runtime_error(format!("{NAME}: {}", detail.into())).with_builtin(NAME);
    if let Some(identifier) = OUTERJOIN_ERRORS[kind].identifier {
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
// Options
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum JoinType {
    Full,
    Left,
    Right,
    Inner,
}

impl JoinType {
    fn keeps_unmatched_left(self) -> bool {
        matches!(self, JoinType::Full | JoinType::Left)
    }

    fn keeps_unmatched_right(self) -> bool {
        matches!(self, JoinType::Full | JoinType::Right)
    }
}

struct Options {
    keys: Option<Value>,
    left_keys: Option<Value>,
    right_keys: Option<Value>,
    merge_keys: bool,
    join_type: JoinType,
}

/// The text of a single name: a char row vector, a string scalar, or a
/// one-element string array (FR-035-22).
fn single_name(value: &Value) -> Option<String> {
    match value {
        Value::CharArray(chars) if chars.rows <= 1 => Some(chars.data.iter().collect()),
        Value::String(text) => Some(text.clone()),
        Value::StringArray(array) if array.data.len() == 1 => Some(array.data[0].clone()),
        _ => None,
    }
}

fn parse_flag(value: &Value) -> Option<bool> {
    let number = match value {
        Value::Bool(flag) => return Some(*flag),
        Value::Num(number) => *number,
        Value::Int(int) => int.to_f64(),
        Value::LogicalArray(array) if array.data.len() == 1 => return Some(array.data[0] != 0),
        Value::Tensor(tensor) if tensor.len() == 1 => *tensor.materialize_f64().first()?,
        _ => return None,
    };
    if number == 0.0 {
        Some(false)
    } else if number == 1.0 {
        Some(true)
    } else {
        None
    }
}

fn parse_type(value: &Value) -> BuiltinResult<JoinType> {
    let text = single_name(value).ok_or_else(|| join_error(INVALID_TYPE, "'Type' must be text"))?;
    match text.to_ascii_lowercase().as_str() {
        "full" => Ok(JoinType::Full),
        "left" => Ok(JoinType::Left),
        "right" => Ok(JoinType::Right),
        "inner" => Ok(JoinType::Inner),
        _ => Err(join_error(
            INVALID_TYPE,
            format!("unrecognised join type '{text}'"),
        )),
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
        merge_keys: false,
        join_type: JoinType::Full,
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
            "mergekeys" => {
                options.merge_keys = parse_flag(&value)
                    .ok_or_else(|| invalid_argument("'MergeKeys' must be a logical scalar"))?
            }
            "type" => options.join_type = parse_type(&value)?,
            // outerjoin.q-select-variables
            "leftvariables" | "rightvariables" => {
                return Err(unsupported(format!(
                    "the '{option}' option is not yet supported"
                )))
            }
            _ => return Err(invalid_argument(format!("unknown option '{option}'"))),
        }
    }
    Ok(options)
}

// ---------------------------------------------------------------------------
// Keys
// ---------------------------------------------------------------------------

enum KeySpec {
    Names(Vec<String>),
    Indices(Vec<usize>),
}

fn key_spec(option: &str, value: &Value) -> BuiltinResult<KeySpec> {
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
    if empty {
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

/// Key pairs `(left position, right position)` (FR-035-02/03, FR-035-22).
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
        let spec = key_spec("Keys", keys)?;
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
            let left = resolve_keys("LeftKeys", &key_spec("LeftKeys", lk)?, left_names)?;
            let right = resolve_keys("RightKeys", &key_spec("RightKeys", rk)?, right_names)?;
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

/// One key value. Within a key position both tables have the same class,
/// so only like variants are compared in practice.
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

/// Per-row key atoms for one key variable (FR-035-15/16/18).
fn key_atoms(name: &str, value: &Value) -> BuiltinResult<Vec<Atom>> {
    let many_to_many = |what: &str| {
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
                            Err(many_to_many("NaN"))
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
                    Err(many_to_many("<missing>"))
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
        // outerjoin.q-key-types
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
// Column assembly
// ---------------------------------------------------------------------------

/// Where one output cell comes from.
#[derive(Clone, Copy)]
enum Pick {
    Left(usize),
    Right(usize),
    Fill,
}

/// Scalars become one-element arrays so that every variable has a row
/// layout `select_rows` and the fill path understand.
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

/// Column-major gather from two same-layout sources, with `fill` for
/// unmatched cells.
fn gather_with_fill<T: Clone>(
    left: (&[T], usize),
    right: (&[T], usize),
    cols: usize,
    picks: &[Pick],
    fill: T,
) -> Vec<T> {
    let mut out = Vec::with_capacity(picks.len() * cols);
    for col in 0..cols {
        for pick in picks {
            out.push(match pick {
                Pick::Left(row) => left.0[row + col * left.1].clone(),
                Pick::Right(row) => right.0[row + col * right.1].clone(),
                Pick::Fill => fill.clone(),
            });
        }
    }
    out
}

fn two_d(shape: &[usize]) -> Option<(usize, usize)> {
    match shape {
        [] => Some((1, 1)),
        [rows] => Some((*rows, 1)),
        [rows, cols] => Some((*rows, *cols)),
        _ => None,
    }
}

fn numeric_gather(left: &Tensor, right: &Tensor, picks: &[Pick]) -> Option<BuiltinResult<Value>> {
    let (lrows, cols) = two_d(&left.shape)?;
    let (rrows, rcols) = two_d(&right.shape)?;
    if cols != rcols {
        return None;
    }
    let ls = left.clone().into_numeric_storage().ok()?;
    let rs = right.clone().into_numeric_storage().ok()?;
    macro_rules! gather {
        ($variant:ident, $a:expr, $b:expr, $fill:expr) => {
            NumericStorage::$variant(gather_with_fill(
                (&$a, lrows),
                (&$b, rrows),
                cols,
                picks,
                $fill,
            ))
        };
    }
    let storage = match (&ls, &rs) {
        (NumericStorage::F64(a), NumericStorage::F64(b)) => gather!(F64, a, b, f64::NAN),
        (NumericStorage::F32(a), NumericStorage::F32(b)) => gather!(F32, a, b, f32::NAN),
        (NumericStorage::I8(a), NumericStorage::I8(b)) => gather!(I8, a, b, 0),
        (NumericStorage::I16(a), NumericStorage::I16(b)) => gather!(I16, a, b, 0),
        (NumericStorage::I32(a), NumericStorage::I32(b)) => gather!(I32, a, b, 0),
        (NumericStorage::I64(a), NumericStorage::I64(b)) => gather!(I64, a, b, 0),
        (NumericStorage::U8(a), NumericStorage::U8(b)) => gather!(U8, a, b, 0),
        (NumericStorage::U16(a), NumericStorage::U16(b)) => gather!(U16, a, b, 0),
        (NumericStorage::U32(a), NumericStorage::U32(b)) => gather!(U32, a, b, 0),
        (NumericStorage::U64(a), NumericStorage::U64(b)) => gather!(U64, a, b, 0),
        _ => return None,
    };
    Some(
        Tensor::from_numeric_storage(storage, vec![picks.len(), cols])
            .map(Value::Tensor)
            .map_err(invalid_variable),
    )
}

fn is_cellstr_column(cell: &CellArray) -> bool {
    is_single_column(&cell.shape)
        && cell
            .data
            .iter()
            .all(|item| matches!(item, Value::CharArray(chars) if chars.rows <= 1))
}

/// Build one output variable from `picks` (FR-035-10, FR-035-19).
fn assemble(name: &str, left: &Value, right: &Value, picks: &[Pick]) -> BuiltinResult<Value> {
    let only_left = picks.iter().all(|pick| matches!(pick, Pick::Left(_)));
    let only_right = picks.iter().all(|pick| matches!(pick, Pick::Right(_)));
    if only_left || only_right {
        let rows: Vec<usize> = picks
            .iter()
            .map(|pick| match pick {
                Pick::Left(row) | Pick::Right(row) => *row,
                Pick::Fill => unreachable!("no fill in a single-source column"),
            })
            .collect();
        return select_rows(if only_left { left } else { right }, &rows);
    }
    let filled = match (left, right) {
        (Value::Tensor(a), Value::Tensor(b)) => numeric_gather(a, b, picks),
        (Value::LogicalArray(a), Value::LogicalArray(b)) => {
            match (two_d(&a.shape), two_d(&b.shape)) {
                (Some((lrows, cols)), Some((rrows, rcols))) if cols == rcols => {
                    let data = gather_with_fill(
                        (a.data.as_slice(), lrows),
                        (b.data.as_slice(), rrows),
                        cols,
                        picks,
                        0u8,
                    );
                    Some(
                        LogicalArray::new(data, vec![picks.len(), cols])
                            .map(Value::LogicalArray)
                            .map_err(invalid_variable),
                    )
                }
                _ => None,
            }
        }
        (Value::StringArray(a), Value::StringArray(b)) => {
            match (two_d(&a.shape), two_d(&b.shape)) {
                (Some((lrows, cols)), Some((rrows, rcols))) if cols == rcols => {
                    let data = gather_with_fill(
                        (a.data.as_slice(), lrows),
                        (b.data.as_slice(), rrows),
                        cols,
                        picks,
                        "<missing>".to_string(),
                    );
                    Some(
                        StringArray::new(data, vec![picks.len(), cols])
                            .map(Value::StringArray)
                            .map_err(invalid_variable),
                    )
                }
                _ => None,
            }
        }
        (Value::Cell(a), Value::Cell(b)) if is_cellstr_column(a) && is_cellstr_column(b) => {
            let empty =
                Value::CharArray(CharArray::new(Vec::new(), 0, 0).map_err(invalid_variable)?);
            let data = gather_with_fill(
                (a.data.as_slice(), a.rows),
                (b.data.as_slice(), b.rows),
                1,
                picks,
                empty,
            );
            Some(
                CellArray::new(data, picks.len(), 1)
                    .map(Value::Cell)
                    .map_err(invalid_variable),
            )
        }
        _ => None,
    };
    filled.unwrap_or_else(|| {
        Err(unsupported(format!(
            "no fill value is specified for variable '{name}' of class {}",
            class_label(left)
        )))
    })
}

// ---------------------------------------------------------------------------
// Names
// ---------------------------------------------------------------------------

fn is_simple_identifier(text: &str) -> bool {
    let mut chars = text.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_alphabetic() || first == '_')
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

/// Suffixes for clashing names (FR-035-09, FR-035-17): the caller's
/// argument names when the interpreter's call site can be trusted to be
/// this call, otherwise `left`/`right`.
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

/// Case-sensitive uniqueness: later duplicates get `_2`, `_3`, ... (U-035-9).
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

async fn table_input(value: Value, side: &str) -> BuiltinResult<ObjectInstance> {
    let host = gather_if_needed_async(&value)
        .await
        .map_err(map_control_flow)?;
    match host {
        Value::Object(object) if object.class_name.is(TABLE_CLASS) => Ok(object),
        // U-035-7
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

/// Where an output variable's values come from.
enum Source {
    Left(usize),
    Right(usize),
    Merged(usize, usize),
}

#[runtime_builtin(
    name = "outerjoin",
    category = "table",
    summary = "Join two tables on key variables, keeping unmatched rows.",
    keywords = "outerjoin,join,table,merge,keys,left join,right join,full join",
    accel = "cpu",
    descriptor(crate::builtins::table::builtins::joins::OUTERJOIN_DESCRIPTOR),
    builtin_path = "crate::builtins::table::builtins"
)]
pub(crate) async fn outerjoin_builtin(
    left: Value,
    right: Value,
    rest: Vec<Value>,
) -> BuiltinResult<Value> {
    let arg_count = 2 + rest.len();
    let left = table_input(left, "left").await?;
    let right = table_input(right, "right").await?;
    // outerjoin.q-index-outputs
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

    // Key atoms per key pair, with class checks (outerjoin.q-key-types).
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

    // Join (FR-035-04..06; U-035-1).
    let mut right_rows: BTreeMap<&[Atom], Vec<usize>> = BTreeMap::new();
    for (j, key) in right_keys.iter().enumerate() {
        right_rows.entry(key.as_slice()).or_default().push(j);
    }
    let mut left_counts: BTreeMap<&[Atom], usize> = BTreeMap::new();
    for key in &left_keys {
        *left_counts.entry(key.as_slice()).or_default() += 1;
    }
    // outerjoin.q-many-to-many
    if right_rows
        .iter()
        .any(|(key, rows)| rows.len() > 1 && left_counts.get(key).is_some_and(|n| *n > 1))
    {
        return Err(unsupported(
            "a key value that occurs more than once in both tables is not yet supported",
        ));
    }
    let join_type = options.join_type;
    let mut matched_right = vec![false; right_height];
    let mut rows: Vec<(&[Atom], Option<usize>, Option<usize>)> = Vec::new();
    for (i, key) in left_keys.iter().enumerate() {
        match right_rows.get(key.as_slice()) {
            Some(matches) => {
                for &j in matches {
                    matched_right[j] = true;
                    rows.push((key.as_slice(), Some(i), Some(j)));
                }
            }
            None if join_type.keeps_unmatched_left() => rows.push((key.as_slice(), Some(i), None)),
            None => {}
        }
    }
    if join_type.keeps_unmatched_right() {
        for (j, key) in right_keys.iter().enumerate() {
            if !matched_right[j] {
                rows.push((key.as_slice(), None, Some(j)));
            }
        }
    }
    rows.sort_by(|a, b| a.0.cmp(b.0));

    // Variable layout (FR-035-07, FR-035-08, FR-035-11; U-035-4).
    let merge = options.merge_keys || join_type == JoinType::Inner;
    if join_type == JoinType::Inner
        && !options.merge_keys
        && pairs.iter().any(|&(l, r)| left_names[l] != right_names[r])
    {
        return Err(unsupported(
            "'Type','inner' with differently named keys requires 'MergeKeys',true",
        ));
    }
    let mut sources: Vec<(String, Source, bool)> = Vec::new(); // (name, source, left side)
    for (l, name) in left_names.iter().enumerate() {
        match pairs.iter().find(|(pl, _)| *pl == l) {
            Some(&(_, r)) if merge => {
                let merged = if right_names[r] == *name {
                    name.clone()
                } else {
                    format!("{name}_{}", right_names[r])
                };
                sources.push((merged, Source::Merged(l, r), true));
            }
            _ => sources.push((name.clone(), Source::Left(l), true)),
        }
    }
    for (r, name) in right_names.iter().enumerate() {
        if merge && pairs.iter().any(|(_, pr)| *pr == r) {
            continue;
        }
        sources.push((name.clone(), Source::Right(r), false));
    }

    // Names (FR-035-09, FR-035-17; U-035-5, U-035-9).
    let left_side: HashSet<&str> = sources
        .iter()
        .filter(|(_, _, is_left)| *is_left)
        .map(|(name, _, _)| name.as_str())
        .collect();
    let right_side: HashSet<&str> = sources
        .iter()
        .filter(|(_, _, is_left)| !*is_left)
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
            .map(|(name, _, is_left)| {
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
    let mut out_columns = Vec::with_capacity(sources.len());
    for (name, source, _) in &sources {
        let (lvalue, rvalue, picks): (&Value, &Value, Vec<Pick>) = match *source {
            Source::Left(l) => (
                &left_columns[l],
                &left_columns[l],
                rows.iter()
                    .map(|(_, i, _)| i.map_or(Pick::Fill, Pick::Left))
                    .collect(),
            ),
            Source::Right(r) => (
                &right_columns[r],
                &right_columns[r],
                rows.iter()
                    .map(|(_, _, j)| j.map_or(Pick::Fill, Pick::Right))
                    .collect(),
            ),
            Source::Merged(l, r) => (
                &left_columns[l],
                &right_columns[r],
                rows.iter()
                    .map(|(_, i, j)| match (i, j) {
                        (Some(i), _) => Pick::Left(*i),
                        (None, Some(j)) => Pick::Right(*j),
                        (None, None) => Pick::Fill,
                    })
                    .collect(),
            ),
        };
        out_columns.push(assemble(name, lvalue, rvalue, &picks)?);
    }
    table_from_columns(out_names, out_columns)
}

#[cfg(test)]
mod table_outerjoin_tests {
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

    fn single_col(values: &[f32]) -> Value {
        Value::Tensor(Tensor::from_f32(values.to_vec(), vec![values.len(), 1]).unwrap())
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

    /// Two-variable double table `names[0] = keys`, `names[1] = values`.
    fn kv(names: [&str; 2], keys: &[f64], values: &[f64]) -> Value {
        table(&names, vec![col(keys), col(values)])
    }

    /// The observation set-up tables `L` and `R` (keys {1,2,4} / {1,3,4}).
    fn l_table() -> Value {
        kv(["k", "a"], &[1.0, 2.0, 4.0], &[10.0, 20.0, 40.0])
    }

    fn r_table() -> Value {
        kv(["k", "b"], &[1.0, 3.0, 4.0], &[100.0, 300.0, 400.0])
    }

    // ---- Calls ------------------------------------------------------------

    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("outerjoin", args))
    }

    /// Call `outerjoin` as if from the source line `source` (for example
    /// `outerjoin(L, R, 'Type', 'left')`), so the call site records the
    /// text of each argument the way the interpreter does.
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
            SourceId(9035),
            "/tmp/outerjoin_caller.m".to_string(),
            source.to_string(),
        )]);
        let _callsite = crate::callsite::push_callsite(Some(SourceId(9035)), Some(spans));
        call(args)
    }

    fn identifier(err: crate::RuntimeError) -> String {
        err.identifier().unwrap_or("<none>").to_string()
    }

    fn err_id(args: &[Value]) -> String {
        identifier(call(args).expect_err("outerjoin should fail"))
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

    /// NaN-aware comparison through the Debug text of the values.
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
    }

    /// Check an all-double observed result: class table, the variable names
    /// in order, the size, and every column's values.
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

    // ---- Observed (normative): 0.3.0 cases ------------------------------

    /// [outerjoin.signature-primary, outerjoin.output-class,
    /// outerjoin.table-shape, outerjoin.name-suffix (no workspace names),
    /// outerjoin.fill-values] Case `basic`.
    #[test]
    fn observed_basic() {
        let out = call(&[
            kv(["k", "a"], &[1.0, 2.0], &[10.0, 20.0]),
            kv(["k", "b"], &[1.0, 3.0], &[100.0, 300.0]),
        ])
        .expect("outerjoin");
        assert_observed(
            &out,
            &[
                ("k_left", &[1.0, 2.0, NAN]),
                ("a", &[10.0, 20.0, NAN]),
                ("k_right", &[1.0, NAN, 3.0]),
                ("b", &[100.0, NAN, 300.0]),
            ],
        );
    }

    const FULL_LR: [(&str, &[f64]); 4] = [
        ("k_L", &[1.0, 2.0, NAN, 4.0]),
        ("a", &[10.0, 20.0, NAN, 40.0]),
        ("k_R", &[1.0, NAN, 3.0, 4.0]),
        ("b", &[100.0, NAN, 300.0, 400.0]),
    ];

    /// [outerjoin.key-inference, outerjoin.type-rows, outerjoin.row-order,
    /// outerjoin.column-order, outerjoin.name-suffix, outerjoin.fill-values]
    /// Case `default-full`.
    #[test]
    fn observed_default_full() {
        let out = call_at("outerjoin(L, R)", &[l_table(), r_table()]).expect("outerjoin");
        assert_observed(&out, &FULL_LR);
    }

    /// [outerjoin.type-rows] Case `type-full`.
    #[test]
    fn observed_type_full() {
        let out = call_at(
            "outerjoin(L, R, 'Type', 'full')",
            &[l_table(), r_table(), text("Type"), text("full")],
        )
        .expect("outerjoin");
        assert_observed(&out, &FULL_LR);
    }

    /// [outerjoin.keys-option] Cases `keys-name` and `keys-index`.
    #[test]
    fn observed_keys_name_and_index() {
        let out = call_at(
            "outerjoin(L, R, 'Keys', 'k')",
            &[l_table(), r_table(), text("Keys"), text("k")],
        )
        .expect("keys-name");
        assert_observed(&out, &FULL_LR);
        let out = call_at(
            "outerjoin(L, R, 'Keys', 1)",
            &[l_table(), r_table(), text("Keys"), Value::Num(1.0)],
        )
        .expect("keys-index");
        assert_observed(&out, &FULL_LR);
    }

    /// [outerjoin.mergekeys, outerjoin.column-order, outerjoin.output-class]
    /// Case `mergekeys`.
    #[test]
    fn observed_mergekeys() {
        let out = call_at(
            "outerjoin(L, R, 'MergeKeys', true)",
            &[l_table(), r_table(), text("MergeKeys"), Value::Bool(true)],
        )
        .expect("outerjoin");
        assert_observed(
            &out,
            &[
                ("k", &[1.0, 2.0, 3.0, 4.0]),
                ("a", &[10.0, 20.0, NAN, 40.0]),
                ("b", &[100.0, NAN, 300.0, 400.0]),
            ],
        );
    }

    /// [outerjoin.type-rows, outerjoin.column-order, outerjoin.fill-values]
    /// Case `type-left`.
    #[test]
    fn observed_type_left() {
        let out = call_at(
            "outerjoin(L, R, 'Type', 'left')",
            &[l_table(), r_table(), text("Type"), text("left")],
        )
        .expect("outerjoin");
        assert_observed(
            &out,
            &[
                ("k_L", &[1.0, 2.0, 4.0]),
                ("a", &[10.0, 20.0, 40.0]),
                ("k_R", &[1.0, NAN, 4.0]),
                ("b", &[100.0, NAN, 400.0]),
            ],
        );
    }

    /// [outerjoin.type-rows, outerjoin.column-order] Case `type-right`.
    #[test]
    fn observed_type_right() {
        let out = call_at(
            "outerjoin(L, R, 'Type', 'right')",
            &[l_table(), r_table(), text("Type"), text("right")],
        )
        .expect("outerjoin");
        assert_observed(
            &out,
            &[
                ("k_L", &[1.0, NAN, 4.0]),
                ("a", &[10.0, NAN, 40.0]),
                ("k_R", &[1.0, 3.0, 4.0]),
                ("b", &[100.0, 300.0, 400.0]),
            ],
        );
    }

    /// [outerjoin.mergekeys] Case `type-left-merge`.
    #[test]
    fn observed_type_left_merge() {
        let out = call_at(
            "outerjoin(L, R, 'Type', 'left', 'MergeKeys', true)",
            &[
                l_table(),
                r_table(),
                text("Type"),
                text("left"),
                text("MergeKeys"),
                Value::Bool(true),
            ],
        )
        .expect("outerjoin");
        assert_observed(
            &out,
            &[
                ("k", &[1.0, 2.0, 4.0]),
                ("a", &[10.0, 20.0, 40.0]),
                ("b", &[100.0, NAN, 400.0]),
            ],
        );
    }

    /// [outerjoin.type-rows, outerjoin.inner-key-single] Case `type-inner`.
    #[test]
    fn observed_type_inner() {
        let out = call_at(
            "outerjoin(L, R, 'Type', 'inner')",
            &[l_table(), r_table(), text("Type"), text("inner")],
        )
        .expect("outerjoin");
        assert_observed(
            &out,
            &[
                ("k", &[1.0, 4.0]),
                ("a", &[10.0, 40.0]),
                ("b", &[100.0, 400.0]),
            ],
        );
    }

    fn r2_table() -> Value {
        kv(["id", "b"], &[1.0, 3.0, 4.0], &[100.0, 300.0, 400.0])
    }

    /// [outerjoin.keys-option, outerjoin.name-suffix (differently named keys
    /// keep their names)] Case `leftright-keys`.
    #[test]
    fn observed_leftright_keys() {
        let out = call_at(
            "outerjoin(L, R2, 'LeftKeys', 'k', 'RightKeys', 'id')",
            &[
                l_table(),
                r2_table(),
                text("LeftKeys"),
                text("k"),
                text("RightKeys"),
                text("id"),
            ],
        )
        .expect("outerjoin");
        assert_observed(
            &out,
            &[
                ("k", &[1.0, 2.0, NAN, 4.0]),
                ("a", &[10.0, 20.0, NAN, 40.0]),
                ("id", &[1.0, NAN, 3.0, 4.0]),
                ("b", &[100.0, NAN, 300.0, 400.0]),
            ],
        );
    }

    /// [outerjoin.mergekeys (k_id), outerjoin.column-order] Case
    /// `leftright-keys-merge`.
    #[test]
    fn observed_leftright_keys_merge() {
        let out = call(&[
            l_table(),
            r2_table(),
            text("LeftKeys"),
            text("k"),
            text("RightKeys"),
            text("id"),
            text("MergeKeys"),
            Value::Bool(true),
        ])
        .expect("outerjoin");
        assert_observed(
            &out,
            &[
                ("k_id", &[1.0, 2.0, 3.0, 4.0]),
                ("a", &[10.0, 20.0, NAN, 40.0]),
                ("b", &[100.0, NAN, 300.0, 400.0]),
            ],
        );
    }

    fn lu_table() -> Value {
        kv(["k", "a"], &[4.0, 2.0, 1.0], &[40.0, 20.0, 10.0])
    }

    fn ru_table() -> Value {
        kv(["k", "b"], &[3.0, 1.0, 4.0], &[300.0, 100.0, 400.0])
    }

    /// [outerjoin.row-order, outerjoin.name-suffix] Case `unsorted`.
    #[test]
    fn observed_unsorted() {
        let out = call_at("outerjoin(Lu, Ru)", &[lu_table(), ru_table()]).expect("outerjoin");
        assert_observed(
            &out,
            &[
                ("k_Lu", &[1.0, 2.0, NAN, 4.0]),
                ("a", &[10.0, 20.0, NAN, 40.0]),
                ("k_Ru", &[1.0, NAN, 3.0, 4.0]),
                ("b", &[100.0, NAN, 300.0, 400.0]),
            ],
        );
    }

    /// [outerjoin.row-order, outerjoin.mergekeys] Case `unsorted-merge`.
    #[test]
    fn observed_unsorted_merge() {
        let out = call(&[lu_table(), ru_table(), text("MergeKeys"), Value::Bool(true)])
            .expect("outerjoin");
        assert_observed(
            &out,
            &[
                ("k", &[1.0, 2.0, 3.0, 4.0]),
                ("a", &[10.0, 20.0, NAN, 40.0]),
                ("b", &[100.0, NAN, 300.0, 400.0]),
            ],
        );
    }

    /// [outerjoin.duplicate-keys] Cases `dup-keys` and `dup-keys-desc`.
    #[test]
    fn observed_duplicate_left_keys_keep_left_order() {
        let out = call(&[
            kv(["k", "a"], &[1.0, 1.0, 2.0], &[10.0, 11.0, 20.0]),
            r_table(),
            text("MergeKeys"),
            Value::Bool(true),
        ])
        .expect("dup-keys");
        assert_observed(
            &out,
            &[
                ("k", &[1.0, 1.0, 2.0, 3.0, 4.0]),
                ("a", &[10.0, 11.0, 20.0, NAN, NAN]),
                ("b", &[100.0, 100.0, NAN, 300.0, 400.0]),
            ],
        );
        let out = call(&[
            kv(["k", "a"], &[1.0, 1.0, 2.0], &[11.0, 10.0, 20.0]),
            r_table(),
            text("MergeKeys"),
            Value::Bool(true),
        ])
        .expect("dup-keys-desc");
        assert_observed(
            &out,
            &[
                ("k", &[1.0, 1.0, 2.0, 3.0, 4.0]),
                ("a", &[11.0, 10.0, 20.0, NAN, NAN]),
                ("b", &[100.0, 100.0, NAN, 300.0, 400.0]),
            ],
        );
    }

    /// [outerjoin.name-suffix (non-key clash)] Case `same-nonkey-name`.
    #[test]
    fn observed_same_nonkey_name() {
        let out = call_at(
            "outerjoin(Ls, Rs, 'Keys', 'k')",
            &[
                kv(["k", "v"], &[1.0, 2.0], &[10.0, 20.0]),
                kv(["k", "v"], &[1.0, 3.0], &[100.0, 300.0]),
                text("Keys"),
                text("k"),
            ],
        )
        .expect("outerjoin");
        assert_observed(
            &out,
            &[
                ("k_Ls", &[1.0, 2.0, NAN]),
                ("v_Ls", &[10.0, 20.0, NAN]),
                ("k_Rs", &[1.0, NAN, 3.0]),
                ("v_Rs", &[100.0, NAN, 300.0]),
            ],
        );
    }

    /// [outerjoin.fill-values, outerjoin.output-class] Case `fill-types`:
    /// NaN for double and single, 0 for int32, false for logical, '' for
    /// cellstr, <missing> for string.
    #[test]
    fn observed_fill_types() {
        let lt = table(
            &["k", "i", "g", "c", "s", "f"],
            vec![
                col(&[1.0, 2.0]),
                int32_col(&[7, 8]),
                logical_col(&[true, false]),
                cellstr_col(&["p", "q"]),
                string_col(&["s", "t"]),
                single_col(&[1.5, 2.5]),
            ],
        );
        let rt = kv(["k", "b"], &[1.0, 3.0], &[100.0, 300.0]);
        let out = call(&[lt, rt, text("MergeKeys"), Value::Bool(true)]).expect("outerjoin");
        assert_eq!(class_of(&out), "table");
        assert_eq!(names(&out), ["k", "i", "g", "c", "s", "f", "b"]);
        assert_eq!(size(&out), (3, 7));
        assert_doubles(&out, "k", &[1.0, 2.0, 3.0]);
        assert_doubles(&out, "b", &[100.0, NAN, 300.0]);
        assert_eq!(var(&out, "i"), int32_col(&[7, 8, 0]));
        assert_eq!(var(&out, "g"), logical_col(&[true, false, false]));
        let c = var(&out, "c");
        assert_eq!(class_of(&c), "cell");
        let Value::Cell(cell) = &c else {
            panic!("expected cell")
        };
        assert_eq!((cell.rows, cell.cols), (3, 1));
        assert_eq!(cell.get(0, 0).unwrap(), text("p"));
        assert_eq!(cell.get(1, 0).unwrap(), text("q"));
        match cell.get(2, 0).unwrap() {
            Value::CharArray(chars) => assert!(chars.data.is_empty(), "fill is ''"),
            other => panic!("expected empty char fill, got {other:?}"),
        }
        assert_eq!(var(&out, "s"), string_col(&["s", "t", "<missing>"]));
        let f = var(&out, "f");
        assert_eq!(class_of(&f), "single");
        let Value::Tensor(f) = f else {
            panic!("expected single tensor")
        };
        assert_eq!(f.shape, vec![3, 1]);
        assert_eq!(format!("{:?}", f.materialize_f64()), "[1.5, 2.5, NaN]");
    }

    /// [outerjoin.key-inference, outerjoin.row-order, outerjoin.mergekeys]
    /// Cases `multi-key` and `multi-key-unsorted`.
    #[test]
    fn observed_multi_key() {
        let expected: [(&str, &[f64]); 4] = [
            ("k1", &[1.0, 1.0, 2.0, 2.0]),
            ("k2", &[5.0, 6.0, 5.0, 6.0]),
            ("a", &[10.0, 20.0, 30.0, NAN]),
            ("b", &[100.0, NAN, 200.0, 300.0]),
        ];
        let sorted = call(&[
            table(
                &["k1", "k2", "a"],
                vec![
                    col(&[1.0, 1.0, 2.0]),
                    col(&[5.0, 6.0, 5.0]),
                    col(&[10.0, 20.0, 30.0]),
                ],
            ),
            table(
                &["k1", "k2", "b"],
                vec![
                    col(&[1.0, 2.0, 2.0]),
                    col(&[5.0, 5.0, 6.0]),
                    col(&[100.0, 200.0, 300.0]),
                ],
            ),
            text("MergeKeys"),
            Value::Bool(true),
        ])
        .expect("multi-key");
        assert_observed(&sorted, &expected);
        let unsorted = call(&[
            table(
                &["k1", "k2", "a"],
                vec![
                    col(&[2.0, 1.0, 1.0]),
                    col(&[5.0, 6.0, 5.0]),
                    col(&[30.0, 20.0, 10.0]),
                ],
            ),
            table(
                &["k1", "k2", "b"],
                vec![
                    col(&[2.0, 2.0, 1.0]),
                    col(&[6.0, 5.0, 5.0]),
                    col(&[300.0, 200.0, 100.0]),
                ],
            ),
            text("MergeKeys"),
            Value::Bool(true),
        ])
        .expect("multi-key-unsorted");
        assert_observed(&unsorted, &expected);
    }

    fn lk2_table() -> Value {
        kv(["a", "k"], &[10.0, 20.0, 40.0], &[1.0, 2.0, 4.0])
    }

    /// [outerjoin.column-order, outerjoin.mergekeys] Case `key-second-column`.
    #[test]
    fn observed_key_second_column() {
        let out = call(&[lk2_table(), r_table(), text("MergeKeys"), Value::Bool(true)])
            .expect("outerjoin");
        assert_observed(
            &out,
            &[
                ("a", &[10.0, 20.0, NAN, 40.0]),
                ("k", &[1.0, 2.0, 3.0, 4.0]),
                ("b", &[100.0, NAN, 300.0, 400.0]),
            ],
        );
    }

    /// [outerjoin.column-order, outerjoin.name-suffix] Case
    /// `key-second-column-nomerge`.
    #[test]
    fn observed_key_second_column_nomerge() {
        let out = call_at("outerjoin(Lk2, R)", &[lk2_table(), r_table()]).expect("outerjoin");
        assert_observed(
            &out,
            &[
                ("a", &[10.0, 20.0, NAN, 40.0]),
                ("k_Lk2", &[1.0, 2.0, NAN, 4.0]),
                ("k_R", &[1.0, NAN, 3.0, 4.0]),
                ("b", &[100.0, NAN, 300.0, 400.0]),
            ],
        );
    }

    /// [outerjoin.error-no-common] Case `err-no-common`.
    #[test]
    fn observed_error_no_common_variable() {
        let id = err_id(&[
            kv(["p", "a"], &[1.0, 2.0], &[10.0, 20.0]),
            kv(["q", "b"], &[1.0, 3.0], &[100.0, 300.0]),
        ]);
        assert_eq!(id, "RunMat:outerjoin:CantInferKey");
    }

    /// [outerjoin.error-bad-type] Case `err-bad-type`.
    #[test]
    fn observed_error_bad_type() {
        let id = err_id(&[l_table(), r_table(), text("Type"), text("sideways")]);
        assert_eq!(id, "RunMat:outerjoin:InvalidType");
    }

    /// [outerjoin.error-keys-and-leftkeys] Case `err-keys-and-leftkeys`.
    #[test]
    fn observed_error_keys_and_leftkeys() {
        let id = err_id(&[
            l_table(),
            r_table(),
            text("Keys"),
            text("k"),
            text("LeftKeys"),
            text("k"),
        ]);
        assert_eq!(id, "RunMat:outerjoin:ConflictingInputs");
    }

    // ---- Unresolved questions (FR-035-13..17) ----------------------------

    /// outerjoin.q-index-outputs: index outputs are not specified.
    #[test]
    fn unresolved_index_outputs_unsupported() {
        // Calls the builtin directly under a requested output count, as the
        // other multi-output tests in this crate do (see validation.md for
        // the dispatcher-level observation).
        for count in [2, 3] {
            let _outputs = crate::output_count::push_output_count(Some(count));
            let err = block_on(outerjoin_builtin(l_table(), r_table(), Vec::new()))
                .expect_err("index outputs");
            assert_eq!(
                identifier(err),
                "RunMat:outerjoin:UnsupportedOption",
                "nargout {count}"
            );
        }
        let _outputs = crate::output_count::push_output_count(Some(1));
        assert!(block_on(outerjoin_builtin(l_table(), r_table(), Vec::new())).is_ok());
    }

    /// outerjoin.q-select-variables: LeftVariables / RightVariables.
    #[test]
    fn unresolved_select_variables_unsupported() {
        for option in ["LeftVariables", "RightVariables"] {
            let id = err_id(&[l_table(), r_table(), text(option), text("k")]);
            assert_eq!(id, "RunMat:outerjoin:UnsupportedOption", "{option}");
        }
    }

    /// outerjoin.q-many-to-many: a key value duplicated in both tables, a
    /// NaN key, and a missing-string key.
    #[test]
    fn unresolved_many_to_many_and_missing_keys_unsupported() {
        let id = err_id(&[
            kv(["k", "a"], &[1.0, 1.0], &[10.0, 11.0]),
            kv(["k", "b"], &[1.0, 1.0], &[100.0, 101.0]),
        ]);
        assert_eq!(id, "RunMat:outerjoin:UnsupportedOption");
        let id = err_id(&[kv(["k", "a"], &[1.0, NAN], &[10.0, 11.0]), r_table()]);
        assert_eq!(id, "RunMat:outerjoin:UnsupportedOption");
        let id = err_id(&[
            table(
                &["k", "a"],
                vec![string_col(&["x", "<missing>"]), col(&[1.0, 2.0])],
            ),
            table(&["k", "b"], vec![string_col(&["x", "y"]), col(&[3.0, 4.0])]),
        ]);
        assert_eq!(id, "RunMat:outerjoin:UnsupportedOption");
    }

    /// outerjoin.q-key-types: key pairs of different classes, and key
    /// variables of classes RunMat does not join on.
    #[test]
    fn unresolved_key_types_unsupported() {
        let id = err_id(&[
            table(&["k", "a"], vec![col(&[1.0, 2.0]), col(&[10.0, 20.0])]),
            table(&["k", "b"], vec![int32_col(&[1, 3]), col(&[100.0, 300.0])]),
        ]);
        assert_eq!(id, "RunMat:outerjoin:UnsupportedOption");
        let id = err_id(&[
            table(
                &["k", "a"],
                vec![string_col(&["x", "y"]), col(&[10.0, 20.0])],
            ),
            table(
                &["k", "b"],
                vec![cellstr_col(&["x", "z"]), col(&[100.0, 300.0])],
            ),
        ]);
        assert_eq!(id, "RunMat:outerjoin:UnsupportedOption");
        let chars = Value::CharArray(CharArray::new("xy".chars().collect(), 2, 1).unwrap());
        let id = err_id(&[
            table(&["k", "a"], vec![chars.clone(), col(&[10.0, 20.0])]),
            table(&["k", "b"], vec![chars, col(&[100.0, 300.0])]),
        ]);
        assert_eq!(id, "RunMat:outerjoin:UnsupportedOption");
        let two_columns = Value::Tensor(Tensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap());
        let id = err_id(&[
            table(&["k", "a"], vec![two_columns.clone(), col(&[10.0, 20.0])]),
            table(&["k", "b"], vec![two_columns, col(&[100.0, 300.0])]),
        ]);
        assert_eq!(id, "RunMat:outerjoin:UnsupportedOption");
    }

    /// outerjoin.q-name-suffix-context: with no recoverable argument names
    /// (no call site, expression arguments, a call site that does not belong
    /// to this call, or the same name twice) the suffixes are _left/_right.
    #[test]
    fn unresolved_name_suffix_falls_back_to_left_right() {
        let expected = ["k_left", "a", "k_right", "b"];
        let out = call(&[l_table(), r_table()]).unwrap();
        assert_eq!(names(&out), expected);
        let out = call_at("outerjoin(tables{1}, R)", &[l_table(), r_table()]).unwrap();
        assert_eq!(names(&out), expected, "expression argument");
        let out = call_at("outerjoin(T, T)", &[l_table(), r_table()]).unwrap();
        assert_eq!(names(&out), expected, "same name twice");
        let out = call_at("feval(fn, L, R)", &[l_table(), r_table()]).unwrap();
        assert_eq!(names(&out), expected, "call site of another call");
    }

    // ---- Documented choices (FR-035-18..22) ------------------------------

    /// U-035-1: a key value duplicated only in the right table gives one row
    /// per right row, in right-input order.
    #[test]
    fn documented_choice_right_duplicates_follow_right_order() {
        let out = call(&[
            kv(["k", "a"], &[1.0, 2.0], &[10.0, 20.0]),
            kv(["k", "b"], &[1.0, 3.0, 1.0], &[101.0, 300.0, 100.0]),
            text("MergeKeys"),
            Value::Bool(true),
        ])
        .unwrap();
        assert_observed(
            &out,
            &[
                ("k", &[1.0, 1.0, 2.0, 3.0]),
                ("a", &[10.0, 10.0, 20.0, NAN]),
                ("b", &[101.0, 100.0, NAN, 300.0]),
            ],
        );
    }

    /// U-035-2: same-class keys of other supported classes (int32, logical,
    /// string, cellstr) match exactly and sort ascending (text by code
    /// point).
    #[test]
    fn documented_choice_non_double_keys() {
        let out = call(&[
            table(&["k", "a"], vec![int32_col(&[3, 1]), col(&[30.0, 10.0])]),
            table(&["k", "b"], vec![int32_col(&[2, 1]), col(&[200.0, 100.0])]),
            text("MergeKeys"),
            Value::Bool(true),
        ])
        .unwrap();
        assert_eq!(var(&out, "k"), int32_col(&[1, 2, 3]));
        assert_doubles(&out, "a", &[10.0, NAN, 30.0]);

        let out = call(&[
            table(&["k", "a"], vec![string_col(&["b", "a"]), col(&[2.0, 1.0])]),
            table(
                &["k", "b"],
                vec![string_col(&["c", "a"]), col(&[30.0, 10.0])],
            ),
            text("MergeKeys"),
            Value::Bool(true),
        ])
        .unwrap();
        assert_eq!(var(&out, "k"), string_col(&["a", "b", "c"]));
        assert_doubles(&out, "b", &[10.0, NAN, 30.0]);

        let out = call(&[
            table(
                &["k", "a"],
                vec![cellstr_col(&["y", "x"]), col(&[2.0, 1.0])],
            ),
            table(&["k", "b"], vec![cellstr_col(&["x"]), col(&[10.0])]),
        ])
        .unwrap();
        assert_eq!(names(&out), ["k_left", "a", "k_right", "b"]);
        let Value::Cell(right_keys) = var(&out, "k_right") else {
            panic!("cell keys")
        };
        assert_eq!(right_keys.get(0, 0).unwrap(), text("x"));
        assert!(matches!(right_keys.get(1, 0).unwrap(), Value::CharArray(c) if c.data.is_empty()));

        let out = call(&[
            table(
                &["k", "a"],
                vec![logical_col(&[true, false]), col(&[1.0, 0.0])],
            ),
            table(&["k", "b"], vec![logical_col(&[true]), col(&[5.0])]),
            text("Type"),
            text("inner"),
        ])
        .unwrap();
        assert_eq!(var(&out, "k"), logical_col(&[true]));
    }

    /// U-035-3: a variable whose class has no specified fill value is only
    /// an error when an unmatched row actually needs a fill.
    #[test]
    fn documented_choice_unspecified_fill_classes() {
        let chars = Value::CharArray(CharArray::new("pq".chars().collect(), 2, 1).unwrap());
        let left = || table(&["k", "c"], vec![col(&[1.0, 2.0]), chars.clone()]);
        let id = err_id(&[left(), kv(["k", "b"], &[1.0, 3.0], &[100.0, 300.0])]);
        assert_eq!(id, "RunMat:outerjoin:UnsupportedOption");
        let out = call(&[
            left(),
            kv(["k", "b"], &[1.0, 3.0], &[100.0, 300.0]),
            text("Type"),
            text("left"),
        ])
        .expect("no fill needed in the left variables");
        assert_eq!(size(&out), (2, 4));
        // Multi-column numeric variables are filled element-wise.
        let wide = Value::Tensor(Tensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap());
        let out = call(&[
            kv(["k", "a"], &[1.0, 2.0], &[10.0, 20.0]),
            table(&["k", "w"], vec![col(&[1.0, 3.0]), wide]),
            text("MergeKeys"),
            Value::Bool(true),
        ])
        .unwrap();
        let Value::Tensor(w) = var(&out, "w") else {
            panic!("tensor")
        };
        assert_eq!(w.shape, vec![3, 2]);
        assert_eq!(
            format!("{:?}", w.materialize_f64()),
            format!("{:?}", [1.0, NAN, 2.0, 3.0, NAN, 4.0])
        );
    }

    /// U-035-4: 'inner' with differently named keys needs MergeKeys; with
    /// MergeKeys the merged name follows the mergekeys rule.
    #[test]
    fn documented_choice_inner_with_different_key_names() {
        let base = || {
            vec![
                l_table(),
                r2_table(),
                text("LeftKeys"),
                text("k"),
                text("RightKeys"),
                text("id"),
                text("Type"),
                text("inner"),
            ]
        };
        assert_eq!(err_id(&base()), "RunMat:outerjoin:UnsupportedOption");
        let mut args = base();
        args.extend([text("MergeKeys"), Value::Bool(true)]);
        let out = call(&args).unwrap();
        assert_observed(
            &out,
            &[
                ("k_id", &[1.0, 4.0]),
                ("a", &[10.0, 40.0]),
                ("b", &[100.0, 400.0]),
            ],
        );
    }

    /// U-035-5: with MergeKeys, clashing non-key names are suffixed like
    /// the no-merge case; U-035-9: names still colliding afterwards get
    /// _2, _3, ... (later variable renamed).
    #[test]
    fn documented_choice_name_clashes() {
        let out = call_at(
            "outerjoin(Ls, Rs, 'MergeKeys', true)",
            &[
                kv(["k", "v"], &[1.0], &[10.0]),
                kv(["k", "v"], &[1.0], &[100.0]),
                text("Keys"),
                text("k"),
                text("MergeKeys"),
                Value::Bool(true),
            ],
        );
        // The source text above has fewer arguments than the call, so the
        // call site is ignored and the fallback suffixes are used.
        assert_eq!(names(&out.unwrap()), ["k", "v_left", "v_right"]);

        let out = call(&[
            table(&["k", "k_left"], vec![col(&[1.0]), col(&[5.0])]),
            kv(["k", "b"], &[1.0], &[6.0]),
        ])
        .unwrap();
        assert_eq!(names(&out), ["k_left", "k_left_2", "k_right", "b"]);
    }

    /// U-035-6: the output carries RunMat's default table properties.
    #[test]
    fn documented_choice_default_output_properties() {
        let out = call(&[l_table(), r_table()]).unwrap();
        let props = table_public_properties(&table_object(&out)).unwrap();
        match props.fields.get(VARIABLE_UNITS) {
            Some(Value::StringArray(units)) => {
                assert_eq!(units.data, vec![String::new(); 4])
            }
            other => panic!("units: {other:?}"),
        }
    }

    /// U-035-8: option names and Type values are case-insensitive; string
    /// scalars, cell arrays and string arrays name keys.
    #[test]
    fn documented_choice_option_forms() {
        let out = call(&[
            l_table(),
            r_table(),
            Value::String("type".into()),
            Value::String("LEFT".into()),
            text("mergekeys"),
            Value::Num(1.0),
            text("keys"),
            Value::Cell(CellArray::new(vec![text("k")], 1, 1).unwrap()),
        ])
        .unwrap();
        assert_eq!(names(&out), ["k", "a", "b"]);
        assert_eq!(size(&out), (3, 3));
        let out = call(&[
            l_table(),
            r2_table(),
            text("LeftKeys"),
            Value::Num(1.0),
            text("RightKeys"),
            Value::StringArray(StringArray::new(vec!["id".into()], vec![1, 1]).unwrap()),
        ])
        .unwrap();
        assert_eq!(names(&out), ["k", "a", "id", "b"]);
    }

    // ---- Validation --------------------------------------------------------

    #[test]
    fn rejects_non_table_and_timetable_inputs() {
        assert_eq!(
            err_id(&[Value::Num(1.0), r_table()]),
            "RunMat:outerjoin:InvalidInput"
        );
        assert_eq!(
            err_id(&[l_table(), Value::Num(1.0)]),
            "RunMat:outerjoin:InvalidInput"
        );
        let tt = table_from_columns_with_class(
            TIMETABLE_CLASS,
            vec!["k".into()],
            vec![col(&[1.0])],
            None,
        )
        .unwrap();
        assert_eq!(
            err_id(&[tt, r_table()]),
            "RunMat:outerjoin:UnsupportedOption"
        );
    }

    #[test]
    fn rejects_malformed_options() {
        let with = |extra: Vec<Value>| {
            let mut args = vec![l_table(), r_table()];
            args.extend(extra);
            err_id(&args)
        };
        assert_eq!(
            with(vec![text("Type")]),
            "RunMat:outerjoin:InvalidArgument",
            "odd count"
        );
        assert_eq!(
            with(vec![text("Bogus"), Value::Num(1.0)]),
            "RunMat:outerjoin:InvalidArgument",
            "unknown option"
        );
        assert_eq!(
            with(vec![Value::Num(1.0), Value::Num(1.0)]),
            "RunMat:outerjoin:InvalidArgument",
            "non-text option name"
        );
        assert_eq!(
            with(vec![text("LeftKeys"), text("k")]),
            "RunMat:outerjoin:InvalidArgument",
            "LeftKeys without RightKeys"
        );
        assert_eq!(
            with(vec![
                text("LeftKeys"),
                text("k"),
                text("RightKeys"),
                Value::Cell(CellArray::new(vec![text("k"), text("b")], 1, 2).unwrap()),
            ]),
            "RunMat:outerjoin:InvalidArgument",
            "key count mismatch"
        );
        assert_eq!(
            with(vec![text("Keys"), text("k"), text("RightKeys"), text("k")]),
            "RunMat:outerjoin:ConflictingInputs"
        );
        assert_eq!(
            with(vec![text("Keys"), text("zz")]),
            "RunMat:outerjoin:UnknownVariable"
        );
        assert_eq!(
            with(vec![text("Keys"), Value::Num(9.0)]),
            "RunMat:outerjoin:UnknownVariable"
        );
        assert_eq!(
            with(vec![text("Keys"), Value::Num(2.0)]),
            "RunMat:outerjoin:InvalidArgument",
            "index names differ across tables"
        );
        assert_eq!(
            with(vec![text("Keys"), logical_col(&[true, false])]),
            "RunMat:outerjoin:UnsupportedOption",
            "logical key mask"
        );
        assert_eq!(
            with(vec![
                text("Keys"),
                Value::Cell(CellArray::new(Vec::new(), 1, 0).unwrap())
            ]),
            "RunMat:outerjoin:InvalidArgument",
            "empty key list"
        );
        assert_eq!(
            with(vec![
                text("Keys"),
                Value::Cell(CellArray::new(vec![text("k"), text("k")], 1, 2).unwrap())
            ]),
            "RunMat:outerjoin:InvalidArgument",
            "repeated key"
        );
        assert_eq!(
            with(vec![text("MergeKeys"), text("yes")]),
            "RunMat:outerjoin:InvalidArgument"
        );
        assert_eq!(
            with(vec![text("Type"), Value::Num(1.0)]),
            "RunMat:outerjoin:InvalidType"
        );
        assert_eq!(
            with(vec![text("Type"), text("outer")]),
            "RunMat:outerjoin:InvalidType"
        );
    }

    #[test]
    fn inner_join_without_matches_is_empty() {
        let out = call(&[
            kv(["k", "a"], &[1.0], &[10.0]),
            kv(["k", "b"], &[2.0], &[20.0]),
            text("Type"),
            text("inner"),
        ])
        .unwrap();
        assert_eq!(names(&out), ["k", "a", "b"]);
        assert_eq!(size(&out), (0, 3));
    }
}
