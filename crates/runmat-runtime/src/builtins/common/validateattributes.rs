//! Clean-room `validateattributes` (class and attribute validation).
//!
//! Clean-room feature `specs/038-validateattributes/`. Normative behaviour
//! comes only from the approved export `validateattributes` 0.3.0 (see that
//! feature's `provenance.md`). Every error uses the RunMat namespace
//! `RunMat:validateattributes:<suffix>` (maintainer decision D2 = B); points
//! the export leaves unresolved raise `RunMat:validateattributes:UnsupportedOption`
//! or are documented choices in the feature spec.

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{IntegerStorage, Value};

use crate::builtins::common::shape::{effective_rank, value_dimensions};
use crate::builtins::introspection::class::class_name_for_value;
use crate::warnings::{self, WarningRequest};
use crate::{build_runtime_error, BuiltinResult, RuntimeError};

const NAME: &str = "validateattributes";

// ---------------------------------------------------------------------------
// Descriptors
// ---------------------------------------------------------------------------

const fn param(
    name: &'static str,
    arity: BuiltinParamArity,
    description: &'static str,
) -> BuiltinParamDescriptor {
    BuiltinParamDescriptor {
        name,
        ty: BuiltinParamType::Any,
        arity,
        default: None,
        description,
    }
}

const A_PARAM: BuiltinParamDescriptor =
    param("A", BuiltinParamArity::Required, "Value to validate.");
const CLASSES_PARAM: BuiltinParamDescriptor = param(
    "classes",
    BuiltinParamArity::Required,
    "Cell array of class names; A must match one ('numeric' matches any numeric class).",
);
const ATTRIBUTES_PARAM: BuiltinParamDescriptor = param(
    "attributes",
    BuiltinParamArity::Required,
    "Cell array of attribute names, some followed by a value.",
);
const FUNC_NAME_PARAM: BuiltinParamDescriptor = param(
    "funcName",
    BuiltinParamArity::Required,
    "Function name shown in error messages.",
);
const VAR_NAME_PARAM: BuiltinParamDescriptor = param(
    "varName",
    BuiltinParamArity::Required,
    "Variable name shown in error messages.",
);
const ARG_INDEX_PARAM: BuiltinParamDescriptor = param(
    "argIndex",
    BuiltinParamArity::Required,
    "Argument position shown in error messages.",
);

const INPUTS_3: [BuiltinParamDescriptor; 3] = [A_PARAM, CLASSES_PARAM, ATTRIBUTES_PARAM];
const INPUTS_5: [BuiltinParamDescriptor; 5] = [
    A_PARAM,
    CLASSES_PARAM,
    ATTRIBUTES_PARAM,
    FUNC_NAME_PARAM,
    VAR_NAME_PARAM,
];
const INPUTS_6: [BuiltinParamDescriptor; 6] = [
    A_PARAM,
    CLASSES_PARAM,
    ATTRIBUTES_PARAM,
    FUNC_NAME_PARAM,
    VAR_NAME_PARAM,
    ARG_INDEX_PARAM,
];
const VALIDATEATTRIBUTES_SIGNATURES: [BuiltinSignatureDescriptor; 3] = [
    BuiltinSignatureDescriptor {
        label: "validateattributes(A, classes, attributes)",
        inputs: &INPUTS_3,
        outputs: &[],
    },
    BuiltinSignatureDescriptor {
        label: "validateattributes(A, classes, attributes, funcName, varName)",
        inputs: &INPUTS_5,
        outputs: &[],
    },
    BuiltinSignatureDescriptor {
        label: "validateattributes(A, classes, attributes, funcName, varName, argIndex)",
        inputs: &INPUTS_6,
        outputs: &[],
    },
];

const fn error(
    code: &'static str,
    identifier: &'static str,
    when: &'static str,
) -> BuiltinErrorDescriptor {
    BuiltinErrorDescriptor {
        code,
        identifier: Some(identifier),
        when,
        message: when,
    }
}

/// One entry per RunMat identifier (spec "Identifier mapping"; decision D2 = B).
const VALIDATEATTRIBUTES_ERRORS: &[BuiltinErrorDescriptor] = &[
    error("RM.VALIDATEATTRIBUTES.INVALID_TYPE", "RunMat:validateattributes:invalidType", "The input's class is not in the class list (an empty list matches nothing)."),
    error("RM.VALIDATEATTRIBUTES.EXPECTED_SCALAR", "RunMat:validateattributes:expectedScalar", "'scalar' and the input is not 1x1."),
    error("RM.VALIDATEATTRIBUTES.EXPECTED_VECTOR", "RunMat:validateattributes:expectedVector", "'vector' and the input is not a 2-D array with one extent equal to 1."),
    error("RM.VALIDATEATTRIBUTES.EXPECTED_ROW", "RunMat:validateattributes:expectedRow", "'row' and the input is not a single row."),
    error("RM.VALIDATEATTRIBUTES.EXPECTED_COLUMN", "RunMat:validateattributes:expectedColumn", "'column' and the input is not a single column."),
    error("RM.VALIDATEATTRIBUTES.EXPECTED_2D", "RunMat:validateattributes:expected2D", "'2d' and the input has more than two dimensions."),
    error("RM.VALIDATEATTRIBUTES.EXPECTED_NONEMPTY", "RunMat:validateattributes:expectedNonempty", "'nonempty' and the input is empty."),
    error("RM.VALIDATEATTRIBUTES.EXPECTED_SQUARE", "RunMat:validateattributes:expectedSquare", "'square' and the input is not a square matrix."),
    error("RM.VALIDATEATTRIBUTES.EXPECTED_POSITIVE", "RunMat:validateattributes:expectedPositive", "'positive' and a value is not greater than zero."),
    error("RM.VALIDATEATTRIBUTES.EXPECTED_NONNEGATIVE", "RunMat:validateattributes:expectedNonnegative", "'nonnegative' and a value is negative."),
    error("RM.VALIDATEATTRIBUTES.EXPECTED_INTEGER", "RunMat:validateattributes:expectedInteger", "'integer' and a value is not integer-valued."),
    error("RM.VALIDATEATTRIBUTES.EXPECTED_FINITE", "RunMat:validateattributes:expectedFinite", "'finite' and a value is Inf or NaN."),
    error("RM.VALIDATEATTRIBUTES.EXPECTED_NONNAN", "RunMat:validateattributes:expectedNonNaN", "'nonnan' and a value is NaN."),
    error("RM.VALIDATEATTRIBUTES.EXPECTED_REAL", "RunMat:validateattributes:expectedReal", "'real' and the input is complex."),
    error("RM.VALIDATEATTRIBUTES.EXPECTED_NONZERO", "RunMat:validateattributes:expectedNonZero", "'nonzero' and a value is zero."),
    error("RM.VALIDATEATTRIBUTES.EXPECTED_NONSPARSE", "RunMat:validateattributes:expectedNonsparse", "'nonsparse' and the input is sparse."),
    error("RM.VALIDATEATTRIBUTES.EXPECTED_BINARY", "RunMat:validateattributes:expectedBinary", "'binary' and a value is neither 0 nor 1."),
    error("RM.VALIDATEATTRIBUTES.EXPECTED_EVEN", "RunMat:validateattributes:expectedEven", "'even' and a value is not an even integer."),
    error("RM.VALIDATEATTRIBUTES.EXPECTED_ODD", "RunMat:validateattributes:expectedOdd", "'odd' and a value is not an odd integer."),
    error("RM.VALIDATEATTRIBUTES.EXPECTED_INCREASING", "RunMat:validateattributes:expectedIncreasing", "'increasing' and the values are not strictly increasing."),
    error("RM.VALIDATEATTRIBUTES.EXPECTED_DECREASING", "RunMat:validateattributes:expectedDecreasing", "'decreasing' and the values are not strictly decreasing."),
    error("RM.VALIDATEATTRIBUTES.EXPECTED_NONDECREASING", "RunMat:validateattributes:expectedNondecreasing", "'nondecreasing' and a value is below its predecessor."),
    error("RM.VALIDATEATTRIBUTES.EXPECTED_NONINCREASING", "RunMat:validateattributes:expectedNonincreasing", "'nonincreasing' and a value is above its predecessor."),
    error("RM.VALIDATEATTRIBUTES.NOT_GREATER", "RunMat:validateattributes:notGreater", "'>', b and a value is not greater than b."),
    error("RM.VALIDATEATTRIBUTES.NOT_GREATER_EQUAL", "RunMat:validateattributes:notGreaterEqual", "'>=', b and a value is below b."),
    error("RM.VALIDATEATTRIBUTES.NOT_LESS", "RunMat:validateattributes:notLess", "'<', b and a value is not less than b."),
    error("RM.VALIDATEATTRIBUTES.NOT_LESS_EQUAL", "RunMat:validateattributes:notLessEqual", "'<=', b and a value is above b."),
    error("RM.VALIDATEATTRIBUTES.INCORRECT_NUMEL", "RunMat:validateattributes:incorrectNumel", "'numel', n and the element count differs from n."),
    error("RM.VALIDATEATTRIBUTES.INCORRECT_SIZE", "RunMat:validateattributes:incorrectSize", "'size', sz and the size does not match sz."),
    error("RM.VALIDATEATTRIBUTES.INCORRECT_NUMCOLS", "RunMat:validateattributes:incorrectNumcols", "'ncols', n and the column count differs from n."),
    error("RM.VALIDATEATTRIBUTES.INCORRECT_NUMROWS", "RunMat:validateattributes:incorrectNumrows", "'nrows', n and the row count differs from n."),
    error("RM.VALIDATEATTRIBUTES.INCORRECT_NUMDIMS", "RunMat:validateattributes:incorrectNumdims", "'ndims', n and the dimension count differs from n."),
    error("RM.VALIDATEATTRIBUTES.MAXLHS", "RunMat:validateattributes:maxlhs", "An output argument is requested; validateattributes returns nothing."),
    error("RM.VALIDATEATTRIBUTES.UNSUPPORTED_OPTION", "RunMat:validateattributes:UnsupportedOption", "The call uses an attribute, value form or argument form that is not yet specified for RunMat."),
    error("RM.VALIDATEATTRIBUTES.INVALID_ARGUMENT", "RunMat:validateattributes:InvalidArgument", "The argument count or an argument type is invalid."),
];

pub const VALIDATEATTRIBUTES_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &VALIDATEATTRIBUTES_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: VALIDATEATTRIBUTES_ERRORS,
};

const WARNING_ATTRIBUTE_NOT_FOUND: &str = "RunMat:validateattributes:attributeNotFound";

fn va_error(suffix: &str, detail: impl Into<String>) -> RuntimeError {
    build_runtime_error(format!("{NAME}: {}", detail.into()))
        .with_builtin(NAME)
        .with_identifier(format!("RunMat:validateattributes:{suffix}"))
        .build()
}

fn unsupported(detail: impl Into<String>) -> RuntimeError {
    va_error("UnsupportedOption", detail)
}

fn invalid_argument(detail: impl Into<String>) -> RuntimeError {
    va_error("InvalidArgument", detail)
}

// ---------------------------------------------------------------------------
// Attributes
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
enum ShapeAttr {
    Scalar,
    Vector,
    Row,
    Column,
    TwoD,
    Nonempty,
    Square,
    Nonsparse,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ValueAttr {
    Positive,
    Nonnegative,
    Integer,
    Finite,
    NonNaN,
    Real,
    NonZero,
    Binary,
    Even,
    Odd,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Monotonic {
    Increasing,
    Decreasing,
    Nondecreasing,
    Nonincreasing,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Compare {
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Count {
    Numel,
    Nrows,
    Ncols,
    Ndims,
}

#[derive(Clone, Debug, PartialEq)]
enum Attr {
    Shape(ShapeAttr),
    Value(ValueAttr),
    Monotonic(Monotonic),
    Compare(Compare, f64),
    Count(Count, usize),
    /// Expected extents; `None` (from NaN) matches any extent.
    Size(Vec<Option<usize>>),
    Unknown(String),
}

/// Every attribute name 0.3.0 confirms (FR-038-04).
const KNOWN_NAMES: [&str; 31] = [
    "scalar",
    "vector",
    "row",
    "column",
    "2d",
    "nonempty",
    "square",
    "nonsparse",
    "positive",
    "nonnegative",
    "integer",
    "finite",
    "nonnan",
    "real",
    "nonzero",
    "binary",
    "even",
    "odd",
    "increasing",
    "decreasing",
    "nondecreasing",
    "nonincreasing",
    ">",
    ">=",
    "<",
    "<=",
    "numel",
    "nrows",
    "ncols",
    "ndims",
    "size",
];

/// Attribute names the export lists as unexercised (q-more-attributes).
const UNEXERCISED_NAMES: [&str; 2] = ["diag", "scalartext"];

fn simple_attribute(name: &str) -> Option<Attr> {
    use Attr::{Monotonic as M, Shape as S, Value as V};
    Some(match name {
        "scalar" => S(ShapeAttr::Scalar),
        "vector" => S(ShapeAttr::Vector),
        "row" => S(ShapeAttr::Row),
        "column" => S(ShapeAttr::Column),
        "2d" => S(ShapeAttr::TwoD),
        "nonempty" => S(ShapeAttr::Nonempty),
        "square" => S(ShapeAttr::Square),
        "nonsparse" => S(ShapeAttr::Nonsparse),
        "positive" => V(ValueAttr::Positive),
        "nonnegative" => V(ValueAttr::Nonnegative),
        "integer" => V(ValueAttr::Integer),
        "finite" => V(ValueAttr::Finite),
        "nonnan" => V(ValueAttr::NonNaN),
        "real" => V(ValueAttr::Real),
        "nonzero" => V(ValueAttr::NonZero),
        "binary" => V(ValueAttr::Binary),
        "even" => V(ValueAttr::Even),
        "odd" => V(ValueAttr::Odd),
        "increasing" => M(Monotonic::Increasing),
        "decreasing" => M(Monotonic::Decreasing),
        "nondecreasing" => M(Monotonic::Nondecreasing),
        "nonincreasing" => M(Monotonic::Nonincreasing),
        _ => return None,
    })
}

/// Text of a char row vector (including the empty char) or a string scalar.
fn text_scalar(value: &Value) -> Option<String> {
    match value {
        Value::CharArray(chars) if chars.rows <= 1 => Some(chars.data.iter().collect()),
        Value::String(text) => Some(text.clone()),
        Value::StringArray(array) if array.data.len() == 1 => Some(array.data[0].clone()),
        _ => None,
    }
}

fn is_text(value: &Value) -> bool {
    matches!(
        value,
        Value::CharArray(_) | Value::String(_) | Value::StringArray(_)
    )
}

/// A real numeric or logical scalar as `f64` (FR-038-15).
fn real_scalar(value: &Value) -> Option<f64> {
    match value {
        Value::Num(x) => Some(*x),
        Value::Int(int) => Some(int.to_f64()),
        Value::Bool(flag) => Some(if *flag { 1.0 } else { 0.0 }),
        Value::Tensor(tensor) if tensor.len() == 1 => tensor.materialize_f64().first().copied(),
        Value::LogicalArray(array) if array.data.len() == 1 => {
            Some(f64::from(array.data.as_slice()[0] != 0))
        }
        _ => None,
    }
}

fn count_value(value: &Value) -> Option<usize> {
    real_scalar(value)
        .filter(|x| x.is_finite() && *x >= 0.0 && x.fract() == 0.0)
        .map(|x| x as usize)
}

fn size_value(value: &Value) -> Option<Vec<Option<usize>>> {
    let values = match value {
        Value::Num(x) => vec![*x],
        Value::Int(int) => vec![int.to_f64()],
        Value::Tensor(tensor) => {
            let dims = &tensor.shape;
            let is_vector = dims.len() <= 2 && dims.iter().filter(|&&d| d != 1).count() <= 1;
            if !is_vector || tensor.is_empty() {
                return None;
            }
            tensor.materialize_f64()
        }
        _ => return None,
    };
    values
        .into_iter()
        .map(|x| {
            if x.is_nan() {
                Some(None)
            } else if x.is_finite() && x >= 0.0 && x.fract() == 0.0 {
                Some(Some(x as usize))
            } else {
                None
            }
        })
        .collect()
}

fn parse_attributes(value: &Value) -> Result<Vec<Attr>, RuntimeError> {
    let items = match value {
        Value::Cell(cell) => &cell.data,
        _ if is_text(value) => {
            return Err(unsupported(
                "a non-cell attribute list is not yet supported; use a cell array",
            ))
        }
        _ => return Err(invalid_argument("attributes must be a cell array")),
    };
    let mut attrs = Vec::new();
    let mut index = 0;
    while index < items.len() {
        let name = text_scalar(&items[index])
            .ok_or_else(|| unsupported("a non-text entry where an attribute name was expected"))?;
        index += 1;
        if let Some(attr) = simple_attribute(&name) {
            attrs.push(attr);
            continue;
        }
        let takes_value = matches!(
            name.as_str(),
            ">" | ">=" | "<" | "<=" | "numel" | "nrows" | "ncols" | "ndims" | "size"
        );
        if takes_value {
            let operand = items
                .get(index)
                .ok_or_else(|| invalid_argument(format!("attribute '{name}' needs a value")))?;
            index += 1;
            let bad_value = || {
                unsupported(format!(
                    "this value form for attribute '{name}' is not yet supported"
                ))
            };
            let attr = match name.as_str() {
                ">" | ">=" | "<" | "<=" => {
                    let bound = real_scalar(operand).ok_or_else(bad_value)?;
                    let op = match name.as_str() {
                        ">" => Compare::Greater,
                        ">=" => Compare::GreaterEqual,
                        "<" => Compare::Less,
                        _ => Compare::LessEqual,
                    };
                    Attr::Compare(op, bound)
                }
                "size" => Attr::Size(size_value(operand).ok_or_else(bad_value)?),
                _ => {
                    let count = count_value(operand).ok_or_else(bad_value)?;
                    let kind = match name.as_str() {
                        "numel" => Count::Numel,
                        "nrows" => Count::Nrows,
                        "ncols" => Count::Ncols,
                        _ => Count::Ndims,
                    };
                    Attr::Count(kind, count)
                }
            };
            attrs.push(attr);
            continue;
        }
        let lower = name.to_ascii_lowercase();
        if UNEXERCISED_NAMES.contains(&lower.as_str()) {
            return Err(unsupported(format!(
                "attribute '{name}' is not yet supported"
            )));
        }
        if KNOWN_NAMES.contains(&lower.as_str()) {
            // U-038-4: case variants of known names are not silently skipped.
            return Err(unsupported(format!(
                "attribute '{name}' differs from a known attribute only by case"
            )));
        }
        attrs.push(Attr::Unknown(name));
    }
    Ok(attrs)
}

// ---------------------------------------------------------------------------
// Classes and context
// ---------------------------------------------------------------------------

/// Numeric classes covered by the `'numeric'` category (FR-038-14).
const NUMERIC_CLASSES: [&str; 10] = [
    "double", "single", "int8", "int16", "int32", "int64", "uint8", "uint16", "uint32", "uint64",
];

fn parse_classes(value: &Value) -> Result<Vec<String>, RuntimeError> {
    let names = match value {
        Value::Cell(cell) => cell
            .data
            .iter()
            .map(|item| {
                text_scalar(item).ok_or_else(|| {
                    invalid_argument("class names must be char row vectors or string scalars")
                })
            })
            .collect::<Result<Vec<_>, _>>()?,
        Value::String(text) => vec![text.clone()],
        Value::StringArray(array) => array.data.clone(),
        Value::CharArray(_) => {
            return Err(unsupported(
                "a char vector class list is not yet supported; use a cell array",
            ))
        }
        _ => {
            return Err(invalid_argument(
                "classes must be a cell array of class names",
            ))
        }
    };
    if let Some(category) = names.iter().find(|n| *n == "float" || *n == "integer") {
        return Err(unsupported(format!(
            "class category '{category}' is not yet supported"
        )));
    }
    Ok(names)
}

fn class_matches(class: &str, names: &[String]) -> bool {
    names
        .iter()
        .any(|name| name == class || (name == "numeric" && NUMERIC_CLASSES.contains(&class)))
}

fn positive_index(value: &Value) -> Option<usize> {
    count_value(value).filter(|&n| n >= 1)
}

/// Message prefix from funcName, varName, argIndex (message text only).
fn message_context(trailing: &[Value]) -> Result<String, RuntimeError> {
    let bad = |what: &str| {
        unsupported(format!(
            "this form of the {what} argument is not yet supported"
        ))
    };
    let function = match trailing.first() {
        Some(value) => text_scalar(value).ok_or_else(|| bad("funcName"))?,
        None => String::new(),
    };
    let variable = match trailing.get(1) {
        Some(value) => text_scalar(value).ok_or_else(|| bad("varName"))?,
        None => String::new(),
    };
    let index = match trailing.get(2) {
        Some(value) => Some(positive_index(value).ok_or_else(|| bad("argIndex"))?),
        None => None,
    };
    let mut context = if variable.is_empty() {
        "input".to_string()
    } else {
        format!("input '{variable}'")
    };
    if let Some(index) = index {
        context.push_str(&format!(" (argument {index})"));
    }
    if !function.is_empty() {
        context.push_str(&format!(" to {function}"));
    }
    Ok(context)
}

// ---------------------------------------------------------------------------
// Element view
// ---------------------------------------------------------------------------

enum Elems {
    Real(Vec<f64>),
    Int(Vec<i128>),
    Complex(Vec<(f64, f64)>),
}

fn integer_values(storage: &IntegerStorage) -> Vec<i128> {
    match storage {
        IntegerStorage::I8(v) => v.iter().map(|&x| x as i128).collect(),
        IntegerStorage::I16(v) => v.iter().map(|&x| x as i128).collect(),
        IntegerStorage::I32(v) => v.iter().map(|&x| x as i128).collect(),
        IntegerStorage::I64(v) => v.iter().map(|&x| x as i128).collect(),
        IntegerStorage::U8(v) => v.iter().map(|&x| x as i128).collect(),
        IntegerStorage::U16(v) => v.iter().map(|&x| x as i128).collect(),
        IntegerStorage::U32(v) => v.iter().map(|&x| x as i128).collect(),
        IntegerStorage::U64(v) => v.iter().map(|&x| x as i128).collect(),
    }
}

/// Element values of numeric and logical inputs; `Ok(None)` for other
/// classes (q-more-attributes).
fn elements(value: &Value) -> Result<Option<Elems>, RuntimeError> {
    let elems = match value {
        Value::Num(x) => Elems::Real(vec![*x]),
        Value::Bool(flag) => Elems::Real(vec![f64::from(*flag)]),
        Value::Int(int) => Elems::Int(vec![int.to_i128()]),
        Value::Complex(re, im) => Elems::Complex(vec![(*re, *im)]),
        Value::LogicalArray(array) => Elems::Real(
            array
                .data
                .as_slice()
                .iter()
                .map(|&b| f64::from(b != 0))
                .collect(),
        ),
        Value::Tensor(tensor) => match tensor.integer_storage() {
            Some(storage) => Elems::Int(integer_values(storage)),
            None => Elems::Real(tensor.materialize_f64()),
        },
        Value::ComplexTensor(tensor) => {
            if tensor.integer_storage().is_some() {
                return Err(unsupported(
                    "value attributes on complex integer values are not yet supported",
                ));
            }
            Elems::Complex(tensor.materialize_f64())
        }
        Value::SparseTensor(sparse) => {
            if sparse.is_complex() {
                let dense = sparse.to_dense_complex().map_err(invalid_argument)?;
                Elems::Complex(dense.materialize_f64())
            } else if sparse.is_logical() {
                let dense = sparse.to_dense_logical().map_err(invalid_argument)?;
                Elems::Real(
                    dense
                        .data
                        .as_slice()
                        .iter()
                        .map(|&b| f64::from(b != 0))
                        .collect(),
                )
            } else {
                Elems::Real(
                    sparse
                        .to_dense()
                        .map_err(invalid_argument)?
                        .materialize_f64(),
                )
            }
        }
        _ => return Ok(None),
    };
    Ok(Some(elems))
}

fn is_vector_shape(dims: &[usize]) -> bool {
    dims.len() == 2 && (dims[0] == 1 || dims[1] == 1)
}

// ---------------------------------------------------------------------------
// Checks
// ---------------------------------------------------------------------------

fn check_shape(
    attr: ShapeAttr,
    value: &Value,
    dims: &[usize],
    context: &str,
) -> Result<(), RuntimeError> {
    let two_d = dims.len() == 2;
    let numel: usize = dims.iter().product();
    let (ok, suffix, need) = match attr {
        ShapeAttr::Scalar => (dims.iter().all(|&d| d == 1), "expectedScalar", "scalar"),
        ShapeAttr::Vector => (is_vector_shape(dims), "expectedVector", "a vector"),
        ShapeAttr::Row => (two_d && dims[0] == 1, "expectedRow", "a row vector"),
        ShapeAttr::Column => (two_d && dims[1] == 1, "expectedColumn", "a column vector"),
        ShapeAttr::TwoD => (two_d, "expected2D", "two-dimensional"),
        ShapeAttr::Nonempty => (numel > 0, "expectedNonempty", "nonempty"),
        ShapeAttr::Square => (
            two_d && dims[0] == dims[1],
            "expectedSquare",
            "a square matrix",
        ),
        ShapeAttr::Nonsparse => (
            !matches!(value, Value::SparseTensor(_)),
            "expectedNonsparse",
            "nonsparse",
        ),
    };
    if ok {
        Ok(())
    } else {
        Err(va_error(
            suffix,
            format!("{context} must be {need}; its size is {}", size_text(dims)),
        ))
    }
}

fn size_text(dims: &[usize]) -> String {
    dims.iter()
        .map(|d| d.to_string())
        .collect::<Vec<_>>()
        .join("x")
}

fn value_attr_info(attr: ValueAttr) -> (&'static str, &'static str) {
    match attr {
        ValueAttr::Positive => ("expectedPositive", "positive"),
        ValueAttr::Nonnegative => ("expectedNonnegative", "nonnegative"),
        ValueAttr::Integer => ("expectedInteger", "integer-valued"),
        ValueAttr::Finite => ("expectedFinite", "finite"),
        ValueAttr::NonNaN => ("expectedNonNaN", "free of NaN values"),
        ValueAttr::Real => ("expectedReal", "real"),
        ValueAttr::NonZero => ("expectedNonZero", "nonzero"),
        ValueAttr::Binary => ("expectedBinary", "binary (0 or 1)"),
        ValueAttr::Even => ("expectedEven", "even"),
        ValueAttr::Odd => ("expectedOdd", "odd"),
    }
}

fn real_value_ok(attr: ValueAttr, x: f64) -> bool {
    let integral = x.is_finite() && x.fract() == 0.0;
    match attr {
        ValueAttr::Positive => x > 0.0,
        ValueAttr::Nonnegative => x >= 0.0,
        ValueAttr::Integer => integral,
        ValueAttr::Finite => x.is_finite(),
        ValueAttr::NonNaN => !x.is_nan(),
        ValueAttr::Real => true,
        ValueAttr::NonZero => x != 0.0,
        ValueAttr::Binary => x == 0.0 || x == 1.0,
        ValueAttr::Even => integral && x % 2.0 == 0.0,
        ValueAttr::Odd => integral && x % 2.0 != 0.0,
    }
}

fn int_value_ok(attr: ValueAttr, x: i128) -> bool {
    match attr {
        ValueAttr::Positive => x > 0,
        ValueAttr::Nonnegative => x >= 0,
        ValueAttr::Integer | ValueAttr::Finite | ValueAttr::NonNaN | ValueAttr::Real => true,
        ValueAttr::NonZero => x != 0,
        ValueAttr::Binary => x == 0 || x == 1,
        ValueAttr::Even => x % 2 == 0,
        ValueAttr::Odd => x % 2 != 0,
    }
}

fn check_value(attr: ValueAttr, elems: &Elems, context: &str) -> Result<(), RuntimeError> {
    let ok = match elems {
        Elems::Real(values) => values.iter().all(|&x| real_value_ok(attr, x)),
        Elems::Int(values) => values.iter().all(|&x| int_value_ok(attr, x)),
        Elems::Complex(values) => match attr {
            ValueAttr::Real => values.is_empty(),
            ValueAttr::Finite => values
                .iter()
                .all(|(re, im)| re.is_finite() && im.is_finite()),
            ValueAttr::NonNaN => values.iter().all(|(re, im)| !re.is_nan() && !im.is_nan()),
            ValueAttr::NonZero => values.iter().all(|(re, im)| *re != 0.0 || *im != 0.0),
            _ => {
                return Err(unsupported(format!(
                    "attribute '{}' on complex values is not yet supported",
                    value_attr_info(attr).1
                )))
            }
        },
    };
    if ok {
        Ok(())
    } else {
        let (suffix, need) = value_attr_info(attr);
        Err(va_error(suffix, format!("{context} must be {need}")))
    }
}

fn check_monotonic(
    attr: Monotonic,
    elems: &Elems,
    dims: &[usize],
    context: &str,
) -> Result<(), RuntimeError> {
    let numel: usize = dims.iter().product();
    if numel > 1 && !is_vector_shape(dims) {
        return Err(unsupported(
            "monotonic attributes on non-vector inputs are not yet supported",
        ));
    }
    let pair_ok = |cmp: Option<std::cmp::Ordering>| {
        use std::cmp::Ordering::{Equal, Greater, Less};
        matches!(
            (attr, cmp),
            (Monotonic::Increasing, Some(Less))
                | (Monotonic::Decreasing, Some(Greater))
                | (Monotonic::Nondecreasing, Some(Less | Equal))
                | (Monotonic::Nonincreasing, Some(Greater | Equal))
        )
    };
    let ok = match elems {
        Elems::Real(values) => values.windows(2).all(|w| pair_ok(w[0].partial_cmp(&w[1]))),
        Elems::Int(values) => values.windows(2).all(|w| pair_ok(Some(w[0].cmp(&w[1])))),
        Elems::Complex(_) => {
            return Err(unsupported(
                "ordering attributes on complex values are not yet supported",
            ))
        }
    };
    if ok {
        return Ok(());
    }
    let (suffix, need) = match attr {
        Monotonic::Increasing => ("expectedIncreasing", "strictly increasing"),
        Monotonic::Decreasing => ("expectedDecreasing", "strictly decreasing"),
        Monotonic::Nondecreasing => ("expectedNondecreasing", "nondecreasing"),
        Monotonic::Nonincreasing => ("expectedNonincreasing", "nonincreasing"),
    };
    Err(va_error(suffix, format!("{context} must be {need}")))
}

fn check_compare(
    op: Compare,
    bound: f64,
    elems: &Elems,
    context: &str,
) -> Result<(), RuntimeError> {
    let test = |x: f64| match op {
        Compare::Greater => x > bound,
        Compare::GreaterEqual => x >= bound,
        Compare::Less => x < bound,
        Compare::LessEqual => x <= bound,
    };
    let ok = match elems {
        Elems::Real(values) => values.iter().all(|&x| test(x)),
        // Integer values are compared through binary64 (documented choice).
        Elems::Int(values) => values.iter().all(|&x| test(x as f64)),
        Elems::Complex(_) => {
            return Err(unsupported(
                "ordering attributes on complex values are not yet supported",
            ))
        }
    };
    if ok {
        return Ok(());
    }
    let (suffix, symbol) = match op {
        Compare::Greater => ("notGreater", ">"),
        Compare::GreaterEqual => ("notGreaterEqual", ">="),
        Compare::Less => ("notLess", "<"),
        Compare::LessEqual => ("notLessEqual", "<="),
    };
    Err(va_error(
        suffix,
        format!("{context} must have every value {symbol} {bound}"),
    ))
}

fn check_count(
    kind: Count,
    expected: usize,
    dims: &[usize],
    context: &str,
) -> Result<(), RuntimeError> {
    let (actual, suffix, what) = match kind {
        Count::Numel => (dims.iter().product(), "incorrectNumel", "elements"),
        Count::Nrows => (
            dims.first().copied().unwrap_or(1),
            "incorrectNumrows",
            "rows",
        ),
        Count::Ncols => (
            dims.get(1).copied().unwrap_or(1),
            "incorrectNumcols",
            "columns",
        ),
        Count::Ndims => (effective_rank(dims), "incorrectNumdims", "dimensions"),
    };
    if actual == expected {
        Ok(())
    } else {
        Err(va_error(
            suffix,
            format!("{context} must have {expected} {what}, but it has {actual}"),
        ))
    }
}

fn check_size(
    expected: &[Option<usize>],
    dims: &[usize],
    context: &str,
) -> Result<(), RuntimeError> {
    let len = expected.len().max(dims.len());
    let ok = (0..len).all(|i| {
        let actual = dims.get(i).copied().unwrap_or(1);
        match expected.get(i) {
            Some(None) => true,
            Some(Some(want)) => *want == actual,
            None => actual == 1,
        }
    });
    if ok {
        return Ok(());
    }
    let wanted = expected
        .iter()
        .map(|e| e.map_or_else(|| "N".to_string(), |n| n.to_string()))
        .collect::<Vec<_>>()
        .join("x");
    Err(va_error(
        "incorrectSize",
        format!(
            "{context} must be of size {wanted}, but its size is {}",
            size_text(dims)
        ),
    ))
}

// ---------------------------------------------------------------------------
// Builtin
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "validateattributes",
    category = "argument-validation",
    summary = "Check that an input has an allowed class and every listed attribute; error otherwise.",
    keywords = "validateattributes,validate,attributes,class,scalar,positive,size,argument validation",
    accel = "cpu",
    sink = true,
    suppress_auto_output = true,
    descriptor(crate::builtins::common::validateattributes::VALIDATEATTRIBUTES_DESCRIPTOR),
    builtin_path = "crate::builtins::common::validateattributes"
)]
async fn validateattributes_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    // FR-038-06: the function returns nothing.
    if matches!(crate::output_count::current_output_count(), Some(n) if n > 0) {
        return Err(va_error(
            "maxlhs",
            "too many output arguments; validateattributes returns nothing",
        )
        .into());
    }
    if !(3..=6).contains(&args.len()) {
        return Err(invalid_argument("expected 3 to 6 input arguments").into());
    }
    let classes = parse_classes(&args[1])?;
    let attributes = parse_attributes(&args[2])?;
    let context = message_context(&args[3..])?;

    let value = &args[0];
    let class = class_name_for_value(value);
    if !class_matches(&class, &classes) {
        let allowed = if classes.is_empty() {
            "(none)".to_string()
        } else {
            classes.join(", ")
        };
        return Err(va_error(
            "invalidType",
            format!("{context} has class {class}; allowed classes: {allowed}"),
        )
        .into());
    }

    let dims = value_dimensions(value).await?;
    let gathered = if matches!(value, Value::GpuTensor(_)) {
        crate::dispatcher::gather_if_needed_async(value).await?
    } else {
        value.clone()
    };
    let mut elems: Option<Option<Elems>> = None;
    for attr in &attributes {
        match attr {
            Attr::Shape(shape) => check_shape(*shape, &gathered, &dims, &context)?,
            Attr::Count(kind, expected) => check_count(*kind, *expected, &dims, &context)?,
            Attr::Size(expected) => check_size(expected, &dims, &context)?,
            Attr::Unknown(name) => warnings::emit(WarningRequest {
                builtin: NAME,
                identifier: WARNING_ATTRIBUTE_NOT_FOUND,
                message: &format!("{NAME}: attribute '{name}' is not recognised and was ignored"),
                show_identifier: false,
            })?,
            Attr::Value(_) | Attr::Monotonic(_) | Attr::Compare(_, _) => {
                if elems.is_none() {
                    elems = Some(elements(&gathered)?);
                }
                let Some(Some(view)) = elems.as_ref() else {
                    return Err(unsupported(format!(
                        "value attributes on class {class} are not yet supported"
                    ))
                    .into());
                };
                match attr {
                    Attr::Value(v) => check_value(*v, view, &context)?,
                    Attr::Monotonic(m) => check_monotonic(*m, view, &dims, &context)?,
                    Attr::Compare(op, bound) => check_compare(*op, *bound, view, &context)?,
                    _ => unreachable!(),
                }
            }
        }
    }
    // Validator convention: a placeholder that is never displayed.
    Ok(Value::Num(0.0))
}

#[cfg(test)]
mod validateattributes_tests {
    use crate::BuiltinResult;
    use futures::executor::block_on;
    use runmat_value::{CellArray, CharArray, IntValue, StringArray, Tensor, Value};

    const PREFIX: &str = "RunMat:validateattributes:";

    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("validateattributes", args))
    }

    fn builtin(name: &str, args: &[Value]) -> Value {
        block_on(crate::call_builtin_async(name, args)).expect(name)
    }

    /// Run a call and collect the identifiers of any warnings it emitted.
    fn run(args: &[Value]) -> (Result<(), String>, Vec<String>) {
        crate::warning_store::reset();
        let result = call(args)
            .map(|_| ())
            .map_err(|err| err.identifier().unwrap_or("<none>").to_string());
        let warnings = crate::warning_store::take_all()
            .into_iter()
            .map(|w| w.identifier)
            .collect();
        (result, warnings)
    }

    fn expect_pass(label: &str, args: &[Value]) {
        let (result, warnings) = run(args);
        assert_eq!(result, Ok(()), "{label}: expected pass");
        assert!(
            warnings.is_empty(),
            "{label}: unexpected warnings {warnings:?}"
        );
    }

    fn expect_err(label: &str, args: &[Value], suffix: &str) {
        let (result, _) = run(args);
        let expected = format!("{PREFIX}{suffix}");
        assert_eq!(result, Err(expected), "{label}");
    }

    fn unsupported(label: &str, args: &[Value]) {
        expect_err(label, args, "UnsupportedOption");
    }

    fn text(s: &str) -> Value {
        Value::CharArray(CharArray::new_row(s))
    }

    fn cell(items: Vec<Value>) -> Value {
        let n = items.len();
        Value::Cell(CellArray::new(items, 1, n).unwrap())
    }

    fn names(items: &[&str]) -> Value {
        cell(items.iter().map(|s| text(s)).collect())
    }

    fn n(x: f64) -> Value {
        Value::Num(x)
    }

    fn row(values: &[f64]) -> Value {
        Value::Tensor(Tensor::new(values.to_vec(), vec![1, values.len()]).unwrap())
    }

    fn col(values: &[f64]) -> Value {
        Value::Tensor(Tensor::new(values.to_vec(), vec![values.len(), 1]).unwrap())
    }

    fn ones(shape: &[usize]) -> Value {
        let len = shape.iter().product();
        Value::Tensor(Tensor::new(vec![1.0; len], shape.to_vec()).unwrap())
    }

    fn empty() -> Value {
        Value::Tensor(Tensor::new(vec![], vec![0, 0]).unwrap())
    }

    fn single(x: f32) -> Value {
        Value::Tensor(Tensor::from_f32(vec![x], vec![1, 1]).unwrap())
    }

    fn numeric() -> Value {
        names(&["numeric"])
    }

    fn attrs(items: Vec<Value>) -> Value {
        cell(items)
    }

    // --- Observed cases (normative; identifiers mapped per decision (b)) ---

    /// [validateattributes.signature-primary, validateattributes.pass-silent,
    /// validateattributes.class-list, validateattributes.empty-attributes,
    /// validateattributes.attribute-semantics] FR-038-01..03, FR-038-05.
    #[test]
    fn observed_pass_cases_return_silently() {
        let cases: Vec<(&str, Vec<Value>)> = vec![
            ("pass", vec![n(5.0), numeric(), names(&["scalar"])]),
            ("pass-empty-attrs", vec![n(5.0), numeric(), cell(vec![])]),
            (
                "pass-class-list",
                vec![single(1.0), names(&["double", "single"]), cell(vec![])],
            ),
            (
                "pass-class-char",
                vec![text("abc"), names(&["char"]), names(&["row"])],
            ),
            (
                "pass-class-logical",
                vec![Value::Bool(true), names(&["logical"]), names(&["scalar"])],
            ),
            (
                "pass-class-cell",
                vec![
                    cell(vec![n(1.0), n(2.0)]),
                    names(&["cell"]),
                    names(&["vector"]),
                ],
            ),
            (
                "pass-class-struct",
                vec![
                    builtin("struct", &[text("a"), n(1.0)]),
                    names(&["struct"]),
                    names(&["scalar"]),
                ],
            ),
            (
                "pass-class-fh",
                vec![
                    Value::FunctionHandle("sin".into()),
                    names(&["function_handle"]),
                    cell(vec![]),
                ],
            ),
            (
                "pass-row",
                vec![row(&[1.0, 2.0, 3.0]), numeric(), names(&["row"])],
            ),
            (
                "pass-positive",
                vec![row(&[1.0, 2.0]), numeric(), names(&["positive"])],
            ),
            (
                "pass-nonnegative",
                vec![row(&[0.0, 1.0]), numeric(), names(&["nonnegative"])],
            ),
            (
                "pass-integer-double",
                vec![n(3.0), numeric(), names(&["integer"])],
            ),
            (
                "pass-increasing",
                vec![row(&[1.0, 2.0, 3.0]), numeric(), names(&["increasing"])],
            ),
            (
                "pass-nondecreasing",
                vec![row(&[1.0, 1.0, 2.0]), numeric(), names(&["nondecreasing"])],
            ),
            (
                "pass-gt",
                vec![row(&[4.0, 5.0]), numeric(), attrs(vec![text(">"), n(3.0)])],
            ),
            (
                "pass-ge",
                vec![row(&[3.0, 5.0]), numeric(), attrs(vec![text(">="), n(3.0)])],
            ),
            (
                "pass-numel",
                vec![
                    row(&[1.0, 2.0, 3.0]),
                    numeric(),
                    attrs(vec![text("numel"), n(3.0)]),
                ],
            ),
            (
                "pass-size-nan",
                vec![
                    ones(&[2, 5]),
                    numeric(),
                    attrs(vec![text("size"), row(&[2.0, f64::NAN])]),
                ],
            ),
            (
                "pass-extra-args",
                vec![
                    n(5.0),
                    numeric(),
                    names(&["scalar"]),
                    text("myfun"),
                    text("x"),
                    n(2.0),
                ],
            ),
        ];
        for (label, args) in cases {
            expect_pass(label, &args);
        }
    }

    /// [validateattributes.error-fail-type, validateattributes.error-empty-class-list]
    /// FR-038-02.
    #[test]
    fn observed_class_failures_raise_invalid_type() {
        expect_err(
            "empty-class-list",
            &[text("x"), cell(vec![]), cell(vec![])],
            "invalidType",
        );
        expect_err(
            "fail-type",
            &[text("a"), numeric(), cell(vec![])],
            "invalidType",
        );
        expect_err(
            "fail-class-list",
            &[
                Value::Int(IntValue::I8(1)),
                names(&["double", "single"]),
                cell(vec![]),
            ],
            "invalidType",
        );
    }

    /// [validateattributes.error-fail-<attribute>, validateattributes.each-attribute-checked]
    /// FR-038-04, FR-038-05.
    #[test]
    fn observed_attribute_failures_raise_mapped_identifiers() {
        let single_attr = |a: &str| names(&[a]);
        let pair = |a: &str, v: Value| attrs(vec![text(a), v]);
        let sparse_eye = builtin("speye", &[n(2.0)]);
        let cases: Vec<(&str, Value, Value, &str)> = vec![
            (
                "fail-scalar",
                row(&[1.0, 2.0]),
                single_attr("scalar"),
                "expectedScalar",
            ),
            (
                "fail-vector",
                ones(&[2, 2]),
                single_attr("vector"),
                "expectedVector",
            ),
            (
                "fail-row",
                col(&[1.0, 2.0]),
                single_attr("row"),
                "expectedRow",
            ),
            (
                "fail-column",
                row(&[1.0, 2.0]),
                single_attr("column"),
                "expectedColumn",
            ),
            ("fail-2d", ones(&[2, 2, 2]), single_attr("2d"), "expected2D"),
            (
                "fail-nonempty",
                empty(),
                single_attr("nonempty"),
                "expectedNonempty",
            ),
            (
                "fail-square",
                ones(&[2, 3]),
                single_attr("square"),
                "expectedSquare",
            ),
            (
                "fail-positive",
                row(&[0.0, 1.0]),
                single_attr("positive"),
                "expectedPositive",
            ),
            (
                "fail-nonnegative",
                n(-1.0),
                single_attr("nonnegative"),
                "expectedNonnegative",
            ),
            (
                "fail-integer",
                n(1.5),
                single_attr("integer"),
                "expectedInteger",
            ),
            (
                "fail-finite",
                n(f64::INFINITY),
                single_attr("finite"),
                "expectedFinite",
            ),
            (
                "fail-nonnan",
                n(f64::NAN),
                single_attr("nonnan"),
                "expectedNonNaN",
            ),
            (
                "fail-real",
                Value::Complex(0.0, 1.0),
                single_attr("real"),
                "expectedReal",
            ),
            (
                "fail-nonzero",
                row(&[1.0, 0.0]),
                single_attr("nonzero"),
                "expectedNonZero",
            ),
            (
                "fail-nonsparse",
                sparse_eye,
                single_attr("nonsparse"),
                "expectedNonsparse",
            ),
            (
                "fail-binary",
                row(&[0.0, 1.0, 2.0]),
                single_attr("binary"),
                "expectedBinary",
            ),
            ("fail-even", n(3.0), single_attr("even"), "expectedEven"),
            ("fail-odd", n(2.0), single_attr("odd"), "expectedOdd"),
            (
                "fail-increasing",
                row(&[1.0, 1.0, 2.0]),
                single_attr("increasing"),
                "expectedIncreasing",
            ),
            (
                "fail-decreasing",
                row(&[1.0, 2.0]),
                single_attr("decreasing"),
                "expectedDecreasing",
            ),
            (
                "fail-nonincreasing",
                row(&[1.0, 2.0]),
                single_attr("nonincreasing"),
                "expectedNonincreasing",
            ),
            ("fail-gt", row(&[3.0, 5.0]), pair(">", n(3.0)), "notGreater"),
            ("fail-ge", n(2.0), pair(">=", n(3.0)), "notGreaterEqual"),
            ("fail-lt", n(3.0), pair("<", n(3.0)), "notLess"),
            ("fail-le", n(4.0), pair("<=", n(3.0)), "notLessEqual"),
            (
                "fail-numel",
                row(&[1.0, 2.0]),
                pair("numel", n(3.0)),
                "incorrectNumel",
            ),
            (
                "fail-size",
                ones(&[3, 5]),
                pair("size", row(&[2.0, f64::NAN])),
                "incorrectSize",
            ),
            (
                "fail-ncols",
                ones(&[2, 3]),
                pair("ncols", n(2.0)),
                "incorrectNumcols",
            ),
            (
                "fail-nrows",
                ones(&[3, 2]),
                pair("nrows", n(2.0)),
                "incorrectNumrows",
            ),
            (
                "fail-ndims",
                ones(&[2, 2]),
                pair("ndims", n(3.0)),
                "incorrectNumdims",
            ),
            (
                "fail-first-of-two",
                row(&[1.0, 2.0]),
                names(&["scalar", "positive"]),
                "expectedScalar",
            ),
            (
                "fail-second-of-two",
                n(-1.0),
                names(&["scalar", "positive"]),
                "expectedPositive",
            ),
        ];
        for (label, value, attributes, suffix) in cases {
            expect_err(label, &[value, numeric(), attributes], suffix);
        }
    }

    /// [validateattributes.funcname-identifier, validateattributes.error-fail-extra-args,
    /// validateattributes.error-fail-extra-varname] FR-038-07: funcName changes only
    /// the message, never the identifier.
    #[test]
    fn observed_funcname_cases_keep_runmat_identifier() {
        let err = call(&[
            row(&[1.0, 2.0]),
            numeric(),
            names(&["scalar"]),
            text("myfun"),
            text("x"),
            n(2.0),
        ])
        .unwrap_err();
        assert_eq!(
            err.identifier(),
            Some("RunMat:validateattributes:expectedScalar")
        );
        assert!(err.message.contains("myfun"), "{}", err.message);
        assert!(err.message.contains('x'), "{}", err.message);

        let err =
            call(&[text("a"), numeric(), cell(vec![]), text("myfun"), text("x")]).unwrap_err();
        assert_eq!(
            err.identifier(),
            Some("RunMat:validateattributes:invalidType")
        );
        assert!(err.message.contains("myfun"), "{}", err.message);
        assert!(!err.identifier().unwrap().starts_with("MATLAB:"));
    }

    /// [validateattributes.warning-unknown-attribute] FR-038-08.
    #[test]
    fn observed_unknown_attribute_warns_and_returns() {
        let (result, warnings) = run(&[n(1.0), numeric(), names(&["notanattribute"])]);
        assert_eq!(result, Ok(()));
        assert_eq!(
            warnings,
            vec!["RunMat:validateattributes:attributeNotFound".to_string()]
        );
    }

    /// [validateattributes.error-returns-nothing] FR-038-06.
    #[test]
    fn observed_requesting_an_output_raises_maxlhs() {
        // Called directly under an output-count guard (the crossvalind test
        // pattern): the dispatcher retries other registered implementations
        // after an error, and the retry does not see the requested count
        // (see validation.md, deviations).
        let call_with = |outputs: Option<usize>| {
            let _guard = crate::output_count::push_output_count(outputs);
            block_on(super::validateattributes_builtin(vec![
                n(5.0),
                numeric(),
                names(&["scalar"]),
            ]))
        };
        let err = call_with(Some(1)).unwrap_err();
        assert_eq!(err.identifier(), Some("RunMat:validateattributes:maxlhs"));
        // Zero requested outputs (a statement call) and an unknown count pass.
        call_with(Some(0)).unwrap();
        call_with(None).unwrap();
    }

    // --- Documented choices -------------------------------------------------

    /// FR-038-14: 'numeric' covers integer and single classes; logical is not
    /// numeric; class names match exactly.
    #[test]
    fn documented_choice_numeric_category_and_exact_names() {
        for value in [
            Value::Int(IntValue::I8(3)),
            Value::Int(IntValue::U64(7)),
            single(1.5),
        ] {
            expect_pass("numeric member", &[value, numeric(), names(&["scalar"])]);
        }
        expect_err(
            "logical not numeric",
            &[Value::Bool(true), numeric(), cell(vec![])],
            "invalidType",
        );
        expect_err(
            "case-sensitive class",
            &[text("a"), names(&["CHAR"]), cell(vec![])],
            "invalidType",
        );
        let classes = Value::StringArray(
            StringArray::new(vec!["char".into(), "numeric".into()], vec![1, 2]).unwrap(),
        );
        expect_pass(
            "string-array classes",
            &[n(2.0), classes.clone(), cell(vec![])],
        );
        expect_err(
            "string not char",
            &[Value::String("a".into()), classes, cell(vec![])],
            "invalidType",
        );
    }

    /// FR-038-10: logical values, NaN/Inf, integer classes, empty inputs.
    #[test]
    fn documented_choice_value_semantics() {
        let logical = builtin("logical", &[row(&[1.0, 0.0])]);
        expect_pass(
            "logical binary",
            &[logical.clone(), names(&["logical"]), names(&["binary"])],
        );
        expect_err(
            "logical positive",
            &[logical, names(&["logical"]), names(&["positive"])],
            "expectedPositive",
        );
        expect_err(
            "nan finite",
            &[n(f64::NAN), numeric(), names(&["finite"])],
            "expectedFinite",
        );
        expect_err(
            "nan positive",
            &[n(f64::NAN), numeric(), names(&["positive"])],
            "expectedPositive",
        );
        expect_pass(
            "nan nonzero",
            &[n(f64::NAN), numeric(), names(&["nonzero"])],
        );
        expect_err(
            "inf integer",
            &[n(f64::INFINITY), numeric(), names(&["integer"])],
            "expectedInteger",
        );
        expect_err(
            "2.5 even",
            &[n(2.5), numeric(), names(&["even"])],
            "expectedEven",
        );
        expect_err(
            "2.5 odd",
            &[n(2.5), numeric(), names(&["odd"])],
            "expectedOdd",
        );
        expect_pass(
            "int64 odd exact",
            &[
                Value::Int(IntValue::I64(i64::MAX)),
                numeric(),
                names(&["odd"]),
            ],
        );
        expect_err(
            "int8 negative",
            &[
                Value::Int(IntValue::I8(-1)),
                numeric(),
                names(&["nonnegative"]),
            ],
            "expectedNonnegative",
        );
        expect_pass(
            "empty passes value attributes",
            &[
                empty(),
                numeric(),
                names(&["positive", "integer", "increasing"]),
            ],
        );
        expect_pass(
            "strict decrease",
            &[row(&[3.0, 2.0]), numeric(), names(&["decreasing"])],
        );
        expect_pass(
            "column increasing",
            &[col(&[1.0, 2.0]), numeric(), names(&["increasing"])],
        );
    }

    /// FR-038-11 / U-038-7: a 'nondecreasing' failure.
    #[test]
    fn documented_choice_nondecreasing_identifier() {
        expect_err(
            "nondecreasing",
            &[row(&[2.0, 1.0]), numeric(), names(&["nondecreasing"])],
            "expectedNondecreasing",
        );
    }

    /// FR-038-12: complex values.
    #[test]
    fn documented_choice_complex_values() {
        let z = Value::Complex(1.0, 1.0);
        expect_pass(
            "complex finite",
            &[
                z.clone(),
                numeric(),
                names(&["finite", "nonnan", "nonzero"]),
            ],
        );
        expect_err(
            "complex inf",
            &[
                Value::Complex(1.0, f64::INFINITY),
                numeric(),
                names(&["finite"]),
            ],
            "expectedFinite",
        );
        expect_pass("complex scalar shape", &[z, numeric(), names(&["scalar"])]);
    }

    /// FR-038-09: shape rules for N-D, empty, and 'size' padding.
    #[test]
    fn documented_choice_shape_rules() {
        expect_pass(
            "size padding",
            &[
                ones(&[2, 3]),
                numeric(),
                attrs(vec![text("size"), row(&[2.0, 3.0, 1.0])]),
            ],
        );
        expect_err(
            "size shorter",
            &[
                ones(&[2, 3, 4]),
                numeric(),
                attrs(vec![text("size"), row(&[2.0, 3.0])]),
            ],
            "incorrectSize",
        );
        expect_pass(
            "ndims 3",
            &[
                ones(&[2, 2, 2]),
                numeric(),
                attrs(vec![text("ndims"), n(3.0)]),
            ],
        );
        expect_pass(
            "ncols nd",
            &[
                ones(&[2, 3, 4]),
                numeric(),
                attrs(vec![text("ncols"), n(3.0)]),
            ],
        );
        expect_err(
            "3-D vector",
            &[ones(&[1, 2, 2]), numeric(), names(&["vector"])],
            "expectedVector",
        );
        expect_pass("empty 2d", &[empty(), numeric(), names(&["2d", "square"])]);
        expect_pass(
            "scalar is vector",
            &[n(1.0), numeric(), names(&["vector", "row", "column"])],
        );
        expect_pass(
            "dense nonsparse",
            &[ones(&[2, 2]), numeric(), names(&["nonsparse"])],
        );
    }

    /// FR-038-13: class before attributes; attributes in list order.
    #[test]
    fn documented_choice_check_order() {
        expect_err(
            "class first",
            &[text("ab"), numeric(), names(&["scalar"])],
            "invalidType",
        );
        expect_err(
            "list order",
            &[row(&[-1.0, 2.0]), numeric(), names(&["positive", "scalar"])],
            "expectedPositive",
        );
        // An unknown attribute after a failing one is never reached.
        let (result, warnings) = run(&[
            row(&[1.0, 2.0]),
            numeric(),
            names(&["scalar", "notanattribute"]),
        ]);
        assert_eq!(result, Err(format!("{PREFIX}expectedScalar")));
        assert!(warnings.is_empty());
    }

    /// FR-038-07 / U-038-1: context arguments appear in the message only.
    #[test]
    fn documented_choice_message_context() {
        let err = call(&[
            row(&[1.0, 2.0]),
            numeric(),
            names(&["scalar"]),
            text("f"),
            Value::String("alpha".into()),
            Value::Int(IntValue::U8(4)),
        ])
        .unwrap_err();
        assert_eq!(
            err.identifier(),
            Some("RunMat:validateattributes:expectedScalar")
        );
        assert!(err.message.contains("alpha"), "{}", err.message);
        assert!(err.message.contains('4'), "{}", err.message);
        expect_pass(
            "empty context strings",
            &[n(1.0), numeric(), cell(vec![]), text(""), text("")],
        );
    }

    // --- Unresolved dispositions ---------------------------------------------

    /// q-more-attributes: value attributes on non-numeric classes.
    #[test]
    fn unresolved_choice_value_attributes_on_non_numeric_classes() {
        unsupported(
            "char nonnegative",
            &[text("a"), names(&["char"]), names(&["nonnegative"])],
        );
        unsupported(
            "cell positive",
            &[cell(vec![n(1.0)]), names(&["cell"]), names(&["positive"])],
        );
        unsupported(
            "char gt",
            &[text("a"), names(&["char"]), attrs(vec![text(">"), n(0.0)])],
        );
        // Shape attributes still apply to any class.
        expect_pass(
            "char row",
            &[text("a"), names(&["char"]), names(&["nonempty", "scalar"])],
        );
    }

    /// q-more-attributes / U-038-4 / U-038-5: named-but-unexercised attributes,
    /// case variants and non-text names.
    #[test]
    fn unresolved_choice_unexercised_attribute_names() {
        for name in ["diag", "scalartext", "Scalar", "POSITIVE"] {
            unsupported(name, &[n(1.0), numeric(), names(&[name])]);
        }
        unsupported("non-text name", &[n(1.0), numeric(), attrs(vec![n(1.0)])]);
    }

    /// U-038-3: class categories.
    #[test]
    fn unresolved_choice_float_and_integer_categories() {
        for category in ["float", "integer"] {
            unsupported(category, &[n(1.0), names(&[category]), cell(vec![])]);
        }
    }

    /// U-038-2: bare char class list; non-cell text attributes.
    #[test]
    fn unresolved_choice_unconfirmed_argument_forms() {
        unsupported("char classes", &[n(1.0), text("numeric"), cell(vec![])]);
        unsupported("char attributes", &[n(1.0), numeric(), text("scalar")]);
        unsupported(
            "string attributes",
            &[n(1.0), numeric(), Value::String("scalar".into())],
        );
    }

    /// U-038-6: monotonic attributes on matrices; complex ordering.
    #[test]
    fn unresolved_choice_matrix_monotonic_and_complex_ordering() {
        unsupported(
            "matrix increasing",
            &[ones(&[2, 2]), numeric(), names(&["increasing"])],
        );
        unsupported(
            "complex positive",
            &[Value::Complex(1.0, 1.0), numeric(), names(&["positive"])],
        );
        unsupported(
            "complex gt",
            &[
                Value::Complex(1.0, 1.0),
                numeric(),
                attrs(vec![text(">"), n(0.0)]),
            ],
        );
    }

    /// U-038-8: malformed attribute values.
    #[test]
    fn unresolved_choice_malformed_attribute_values() {
        let cases = vec![
            attrs(vec![text(">"), row(&[1.0, 2.0])]),
            attrs(vec![text(">"), text("a")]),
            attrs(vec![text("numel"), n(1.5)]),
            attrs(vec![text("numel"), n(-1.0)]),
            attrs(vec![text("ndims"), text("2")]),
            attrs(vec![text("size"), row(&[2.0, 1.5])]),
            attrs(vec![text("size"), text("ab")]),
        ];
        for attributes in cases {
            unsupported("malformed value", &[n(1.0), numeric(), attributes]);
        }
    }

    /// U-038-1: malformed trailing arguments.
    #[test]
    fn unresolved_choice_malformed_trailing_arguments() {
        let base = || vec![n(5.0), numeric(), names(&["scalar"])];
        let cases: Vec<Vec<Value>> = vec![
            vec![n(1.0)],
            vec![text("f"), n(2.0)],
            vec![text("f"), text("x"), n(0.0)],
            vec![text("f"), text("x"), n(1.5)],
            vec![text("f"), text("x"), text("2")],
        ];
        for extra in cases {
            let mut args = base();
            args.extend(extra);
            unsupported("trailing", &args);
        }
    }

    // --- Argument validation -------------------------------------------------

    /// FR-038-14 / FR-038-15: malformed calls.
    #[test]
    fn rejects_bad_arity_and_argument_types() {
        expect_err("two args", &[n(1.0), numeric()], "InvalidArgument");
        let mut seven = vec![n(1.0), numeric(), cell(vec![])];
        seven.extend([text("f"), text("x"), n(1.0), n(1.0)]);
        expect_err("seven args", &seven, "InvalidArgument");
        expect_err(
            "numeric classes",
            &[n(1.0), n(2.0), cell(vec![])],
            "InvalidArgument",
        );
        expect_err(
            "non-text class entry",
            &[n(1.0), cell(vec![n(2.0)]), cell(vec![])],
            "InvalidArgument",
        );
        expect_err(
            "numeric attributes",
            &[n(1.0), numeric(), n(1.0)],
            "InvalidArgument",
        );
        expect_err(
            "missing value",
            &[n(1.0), numeric(), names(&["numel"])],
            "InvalidArgument",
        );
    }
}
