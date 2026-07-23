//! MATLAB-compatible `inputParser` builtin class.
//!
//! Clean-room feature `specs/015-input-parsing/` (SpecKit id 015-input-parsing).
//! Normative behaviour comes exclusively from the approved Tier A export
//! `inputParser` 0.2.0 (claims `inputParser.signature-primary`,
//! `inputParser.results-fields`, `inputParser.using-defaults`; commit pin
//! `b054f3ad809ceee28529a4dd7a6e28d2b6b434ef`). Everything outside the
//! observed surface (validators, `addOptional`, `KeepUnmatched`,
//! `StructExpand`, case-insensitive matching, `Unmatched`, error wording)
//! is an explicit RunMat error or a documented independent choice — see
//! `specs/015-input-parsing/spec.md` "Unresolved behaviour".
//!
//! Design follows the in-tree `containers.Map` exemplar
//! (`builtins/containers/map/containers.map.rs`): lazy per-thread class
//! registration with dotted-method builtins, a constructor returning
//! `Value::HandleObject`, and state mutation through the GC target via
//! `runmat_gc::gc_with_value_mut` (the write-back story from
//! `specs/015-input-parsing/plan.md`; no side registry).

use std::cell::Cell;
use std::collections::HashMap;

use runmat_builtins::{
    Access, BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    CharArray, ClassDef, HandleRef, MethodDef, ObjectInstance, PropertyDef, StructValue, Value,
};
use runmat_macros::runtime_builtin;

use crate::{BuiltinResult, RuntimeError};

const CLASS_NAME: &str = "inputParser";
const BUILTIN_CONSTRUCTOR: &str = "inputParser";
const BUILTIN_ADD_REQUIRED: &str = "inputParser.addRequired";
const BUILTIN_ADD_PARAMETER: &str = "inputParser.addParameter";
const BUILTIN_PARSE: &str = "inputParser.parse";

const PROP_RESULTS: &str = "Results";
const PROP_USING_DEFAULTS: &str = "UsingDefaults";
/// Internal schema properties (registration order matters; kept as row
/// cells). Prefixed to avoid collision with user-visible names; their
/// visibility through generic field access is a documented cosmetic leak
/// (plan.md, machinery caveats).
const PROP_REQUIRED: &str = "__ip_required__";
const PROP_PARAM_NAMES: &str = "__ip_param_names__";
const PROP_PARAM_DEFAULTS: &str = "__ip_param_defaults__";

// ---------------------------------------------------------------------------
// Error descriptors (identifiers are RunMat's own wording; the export
// records no MATLAB error conditions — FR-015-08).
// ---------------------------------------------------------------------------

const IP_ERROR_TOO_MANY_INPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.INPUTPARSER.TOO_MANY_INPUTS",
    identifier: Some("RunMat:inputParser:TooManyInputs"),
    when: "The constructor is called with input arguments.",
    message: "inputParser: constructor takes no input arguments",
};

const IP_ERROR_INVALID_RECEIVER: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.INPUTPARSER.INVALID_RECEIVER",
    identifier: Some("RunMat:inputParser:InvalidReceiver"),
    when: "The first argument is not a valid inputParser handle.",
    message: "inputParser: expected a valid inputParser object",
};

const IP_ERROR_INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.INPUTPARSER.INTERNAL",
    identifier: Some("RunMat:inputParser:Internal"),
    when: "The parser's internal storage has an unexpected shape.",
    message: "inputParser: internal storage error",
};

const IP_ERROR_WRONG_ARITY: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.INPUTPARSER.WRONG_ARITY",
    identifier: Some("RunMat:inputParser:WrongArity"),
    when: "A registration method is called with too few arguments.",
    message: "inputParser: missing required method argument",
};

const IP_ERROR_VALIDATORS_UNSUPPORTED: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.INPUTPARSER.VALIDATORS_UNSUPPORTED",
    identifier: Some("RunMat:inputParser:ValidatorsUnsupported"),
    when: "A validation function (or other extra argument) is supplied to addRequired/addParameter.",
    message: "inputParser: validation functions are not supported by RunMat yet (unresolved in the approved specification); remove the extra argument",
};

const IP_ERROR_INVALID_NAME: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.INPUTPARSER.INVALID_NAME",
    identifier: Some("RunMat:inputParser:InvalidName"),
    when: "An argument name is not non-empty text.",
    message: "inputParser: argument name must be a non-empty character vector or string scalar",
};

const IP_ERROR_DUPLICATE_NAME: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.INPUTPARSER.DUPLICATE_NAME",
    identifier: Some("RunMat:inputParser:DuplicateName"),
    when: "An argument name is registered twice on the same parser.",
    message: "inputParser: argument name is already registered",
};

const IP_ERROR_MISSING_REQUIRED: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.INPUTPARSER.MISSING_REQUIRED",
    identifier: Some("RunMat:inputParser:MissingRequired"),
    when: "parse is called with fewer positional inputs than registered required arguments.",
    message: "inputParser: a required argument was not supplied",
};

const IP_ERROR_NAME_VALUE_MISMATCH: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.INPUTPARSER.NAME_VALUE_MISMATCH",
    identifier: Some("RunMat:inputParser:NameValueMismatch"),
    when: "Inputs after the required positionals do not form name-value pairs.",
    message: "inputParser: arguments after the required positionals must be name-value pairs",
};

const IP_ERROR_PARAMETER_NAME_INVALID: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.INPUTPARSER.PARAMETER_NAME_INVALID",
    identifier: Some("RunMat:inputParser:ParameterNameInvalid"),
    when: "A name position in the name-value tail is not text.",
    message: "inputParser: parameter name must be a character vector or string scalar",
};

const IP_ERROR_UNKNOWN_PARAMETER: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.INPUTPARSER.UNKNOWN_PARAMETER",
    identifier: Some("RunMat:inputParser:UnknownParameter"),
    when: "A supplied parameter name matches no registered parameter (matching is exact and case-sensitive; independent RunMat choice).",
    message: "inputParser: unknown parameter name",
};

const IP_ERROR_DUPLICATE_PARAMETER: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.INPUTPARSER.DUPLICATE_PARAMETER",
    identifier: Some("RunMat:inputParser:DuplicateParameter"),
    when: "The same parameter name is supplied more than once in one parse call.",
    message: "inputParser: parameter supplied more than once",
};

const IP_ERRORS: [BuiltinErrorDescriptor; 11] = [
    IP_ERROR_TOO_MANY_INPUTS,
    IP_ERROR_INVALID_RECEIVER,
    IP_ERROR_INTERNAL,
    IP_ERROR_WRONG_ARITY,
    IP_ERROR_VALIDATORS_UNSUPPORTED,
    IP_ERROR_INVALID_NAME,
    IP_ERROR_DUPLICATE_NAME,
    IP_ERROR_MISSING_REQUIRED,
    IP_ERROR_NAME_VALUE_MISMATCH,
    IP_ERROR_PARAMETER_NAME_INVALID,
    IP_ERROR_UNKNOWN_PARAMETER,
];

// ---------------------------------------------------------------------------
// Descriptors
// ---------------------------------------------------------------------------

const PARSER_VALUE_PARAM: BuiltinParamDescriptor = BuiltinParamDescriptor {
    name: "p",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "inputParser object.",
};

const CONSTRUCTOR_OUTPUT: [BuiltinParamDescriptor; 1] = [PARSER_VALUE_PARAM];

const CONSTRUCTOR_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "p = inputParser",
    inputs: &[],
    outputs: &CONSTRUCTOR_OUTPUT,
}];

pub const INPUT_PARSER_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &CONSTRUCTOR_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &IP_ERRORS,
};

const ADD_REQUIRED_INPUTS: [BuiltinParamDescriptor; 2] = [
    PARSER_VALUE_PARAM,
    BuiltinParamDescriptor {
        name: "argName",
        ty: BuiltinParamType::StringScalar,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Name of the required positional argument.",
    },
];

const ADD_REQUIRED_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "addRequired(p, argName)",
    inputs: &ADD_REQUIRED_INPUTS,
    outputs: &CONSTRUCTOR_OUTPUT,
}];

pub const INPUT_PARSER_ADD_REQUIRED_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &ADD_REQUIRED_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &IP_ERRORS,
};

const ADD_PARAMETER_INPUTS: [BuiltinParamDescriptor; 3] = [
    PARSER_VALUE_PARAM,
    BuiltinParamDescriptor {
        name: "paramName",
        ty: BuiltinParamType::StringScalar,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Name of the name-value parameter.",
    },
    BuiltinParamDescriptor {
        name: "defaultVal",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Default value used when the parameter is omitted at parse time.",
    },
];

const ADD_PARAMETER_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "addParameter(p, paramName, defaultVal)",
    inputs: &ADD_PARAMETER_INPUTS,
    outputs: &CONSTRUCTOR_OUTPUT,
}];

pub const INPUT_PARSER_ADD_PARAMETER_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &ADD_PARAMETER_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &IP_ERRORS,
};

const PARSE_INPUTS: [BuiltinParamDescriptor; 2] = [
    PARSER_VALUE_PARAM,
    BuiltinParamDescriptor {
        name: "varargin",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Variadic,
        default: None,
        description: "Required positional values followed by name-value pairs.",
    },
];

const PARSE_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "parse(p, varargin)",
    inputs: &PARSE_INPUTS,
    outputs: &CONSTRUCTOR_OUTPUT,
}];

pub const INPUT_PARSER_PARSE_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &PARSE_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &IP_ERRORS,
};

// ---------------------------------------------------------------------------
// Class registration (containers.Map pattern: lazy, per-thread)
// ---------------------------------------------------------------------------

thread_local! {
    static INPUT_PARSER_CLASS_REGISTERED: Cell<bool> = const { Cell::new(false) };
}

fn ensure_input_parser_class_registered() {
    INPUT_PARSER_CLASS_REGISTERED.with(|registered| {
        if registered.get() {
            return;
        }
        let mut properties = HashMap::new();
        for name in [PROP_RESULTS, PROP_USING_DEFAULTS] {
            properties.insert(
                name.to_string(),
                PropertyDef {
                    name: name.to_string(),
                    is_static: false,
                    is_constant: false,
                    is_dependent: false,
                    get_access: Access::Public,
                    set_access: Access::Private,
                    default_value: None,
                },
            );
        }
        let mut methods = HashMap::new();
        // `addOptional` is deliberately NOT registered: its behaviour is
        // unresolved in the approved export (FR-015-08).
        for (name, function_name) in [
            ("addRequired", BUILTIN_ADD_REQUIRED),
            ("addParameter", BUILTIN_ADD_PARAMETER),
            ("parse", BUILTIN_PARSE),
        ] {
            methods.insert(
                name.to_string(),
                MethodDef {
                    name: name.to_string(),
                    is_static: false,
                    is_abstract: false,
                    is_sealed: false,
                    access: Access::Public,
                    function_name: function_name.to_string(),
                    implicit_class_argument: None,
                },
            );
        }
        runmat_builtins::register_class(ClassDef {
            name: CLASS_NAME.to_string(),
            parent: None,
            properties,
            methods,
        });
        registered.set(true);
    });
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn ip_error(error: &'static BuiltinErrorDescriptor, builtin: &'static str) -> RuntimeError {
    crate::runtime_descriptor_error(builtin, error)
}

fn ip_error_detail(
    error: &'static BuiltinErrorDescriptor,
    builtin: &'static str,
    detail: impl AsRef<str>,
) -> RuntimeError {
    crate::runtime_descriptor_error_with_detail(builtin, error, detail)
}

fn ip_internal(builtin: &'static str, detail: impl AsRef<str>) -> RuntimeError {
    ip_error_detail(&IP_ERROR_INTERNAL, builtin, detail)
}

fn row_cell(values: Vec<Value>, builtin: &'static str) -> BuiltinResult<Value> {
    let cols = values.len();
    crate::make_cell_with_shape(values, vec![1, cols])
        .map_err(|e| ip_internal(builtin, format!("cell construction failed: {e}")))
}

fn char_row(text: &str) -> Value {
    Value::CharArray(CharArray::new_row(text))
}

/// Extract text from a char row vector, string scalar, or 1x1 string array.
fn text_scalar(value: &Value) -> Option<String> {
    match value {
        Value::CharArray(ca) if ca.rows == 1 => Some(ca.data.iter().collect()),
        Value::String(s) => Some(s.clone()),
        Value::StringArray(sa) if sa.data.len() == 1 => Some(sa.data[0].clone()),
        _ => None,
    }
}

fn argument_name(value: &Value, builtin: &'static str) -> BuiltinResult<String> {
    let Some(name) = text_scalar(value) else {
        return Err(ip_error_detail(
            &IP_ERROR_INVALID_NAME,
            builtin,
            format!("got {value:?}"),
        ));
    };
    if name.is_empty() {
        return Err(ip_error_detail(
            &IP_ERROR_INVALID_NAME,
            builtin,
            "name must not be empty",
        ));
    }
    Ok(name)
}

/// Validate the receiver and return a clone of its handle (cheap: shared GC
/// target).
fn parser_handle(value: &Value, builtin: &'static str) -> BuiltinResult<HandleRef> {
    match value {
        Value::HandleObject(handle) => {
            if handle.class_name != CLASS_NAME {
                return Err(ip_error_detail(
                    &IP_ERROR_INVALID_RECEIVER,
                    builtin,
                    format!(
                        "expected an {} object, got class '{}'",
                        CLASS_NAME, handle.class_name
                    ),
                ));
            }
            if !crate::is_handle_valid(handle) {
                return Err(ip_error_detail(
                    &IP_ERROR_INVALID_RECEIVER,
                    builtin,
                    "handle is invalid or deleted",
                ));
            }
            Ok(handle.clone())
        }
        other => Err(ip_error_detail(
            &IP_ERROR_INVALID_RECEIVER,
            builtin,
            format!("expected an {CLASS_NAME} object, got {other:?}"),
        )),
    }
}

/// Snapshot the underlying object state (clone; mutation goes through
/// `write_parser_props`).
fn parser_object(handle: &HandleRef, builtin: &'static str) -> BuiltinResult<ObjectInstance> {
    match runmat_gc::gc_clone_value(&handle.target) {
        Ok(Value::Object(obj)) if obj.class_name == CLASS_NAME => Ok(obj),
        Ok(other) => Err(ip_internal(
            builtin,
            format!("unexpected handle storage shape {other:?}"),
        )),
        Err(e) => Err(ip_internal(builtin, format!("invalid handle target: {e}"))),
    }
}

/// The state-mutation story from plan.md: write through the shared GC target
/// with `gc_with_value_mut` (setfield's `assign_into_handle` precedent), so
/// every alias of the handle observes the update.
fn write_parser_props(
    handle: &HandleRef,
    builtin: &'static str,
    updates: Vec<(&'static str, Value)>,
) -> BuiltinResult<()> {
    runmat_gc::gc_with_value_mut(&handle.target, |target| -> BuiltinResult<()> {
        let Value::Object(obj) = target else {
            return Err(ip_internal(builtin, "handle storage is not an object"));
        };
        for (name, value) in updates {
            runmat_gc::gc_record_handle_write(&handle.target, &value);
            obj.properties.insert(name.to_string(), value);
        }
        Ok(())
    })
    .map_err(|e| ip_internal(builtin, format!("invalid handle target: {e}")))?
}

fn schema_cell(
    obj: &ObjectInstance,
    prop: &'static str,
    builtin: &'static str,
) -> BuiltinResult<Vec<Value>> {
    match obj.properties.get(prop) {
        Some(Value::Cell(cell)) => Ok(cell.data.to_vec()),
        other => Err(ip_internal(
            builtin,
            format!("schema property '{prop}' has unexpected shape {other:?}"),
        )),
    }
}

fn schema_names(
    obj: &ObjectInstance,
    prop: &'static str,
    builtin: &'static str,
) -> BuiltinResult<Vec<String>> {
    schema_cell(obj, prop, builtin)?
        .iter()
        .map(|value| {
            text_scalar(value).ok_or_else(|| {
                ip_internal(
                    builtin,
                    format!("schema property '{prop}' holds a non-text entry {value:?}"),
                )
            })
        })
        .collect()
}

fn ensure_name_unregistered(
    obj: &ObjectInstance,
    name: &str,
    builtin: &'static str,
) -> BuiltinResult<()> {
    let required = schema_names(obj, PROP_REQUIRED, builtin)?;
    let params = schema_names(obj, PROP_PARAM_NAMES, builtin)?;
    if required.iter().any(|n| n == name) || params.iter().any(|n| n == name) {
        return Err(ip_error_detail(
            &IP_ERROR_DUPLICATE_NAME,
            builtin,
            format!("'{name}'"),
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Builtins
// ---------------------------------------------------------------------------

/// `p = inputParser` — claim inputParser.signature-primary (FR-015-01).
#[runtime_builtin(
    name = "inputParser",
    category = "introspection",
    summary = "Construct an argument-parsing object with required and name-value parameter definitions.",
    keywords = "inputParser,arguments,parse,name-value,required,parameter,defaults",
    descriptor(crate::builtins::introspection::input_parser::INPUT_PARSER_DESCRIPTOR),
    builtin_path = "crate::builtins::introspection::input_parser"
)]
fn input_parser_builtin(args: Vec<Value>) -> crate::BuiltinResult<Value> {
    if !args.is_empty() {
        // Unobserved input form; explicit RunMat error (FR-015-08).
        return Err(ip_error_detail(
            &IP_ERROR_TOO_MANY_INPUTS,
            BUILTIN_CONSTRUCTOR,
            format!("got {} input(s)", args.len()),
        ));
    }
    ensure_input_parser_class_registered();
    let mut obj = ObjectInstance::new(CLASS_NAME.to_string());
    // Pre-parse values are unobserved; documented independent choices
    // (spec.md): Results = empty struct, UsingDefaults = 1x0 cell.
    obj.properties
        .insert(PROP_RESULTS.to_string(), Value::Struct(StructValue::new()));
    obj.properties.insert(
        PROP_USING_DEFAULTS.to_string(),
        row_cell(Vec::new(), BUILTIN_CONSTRUCTOR)?,
    );
    for prop in [PROP_REQUIRED, PROP_PARAM_NAMES, PROP_PARAM_DEFAULTS] {
        obj.properties
            .insert(prop.to_string(), row_cell(Vec::new(), BUILTIN_CONSTRUCTOR)?);
    }
    let gc = runmat_gc::gc_allocate(Value::Object(obj))
        .map_err(|e| ip_internal(BUILTIN_CONSTRUCTOR, format!("gc allocation failed: {e}")))?;
    Ok(Value::HandleObject(HandleRef {
        class_name: CLASS_NAME.to_string(),
        target: gc,
        valid: true,
    }))
}

/// `addRequired(p, argName)` — claim inputParser.results-fields (FR-015-02).
#[runtime_builtin(
    name = "inputParser.addRequired",
    descriptor(crate::builtins::introspection::input_parser::INPUT_PARSER_ADD_REQUIRED_DESCRIPTOR),
    builtin_path = "crate::builtins::introspection::input_parser"
)]
fn input_parser_add_required(parser: Value, rest: Vec<Value>) -> crate::BuiltinResult<Value> {
    let handle = parser_handle(&parser, BUILTIN_ADD_REQUIRED)?;
    let Some((name_arg, extra)) = rest.split_first() else {
        return Err(ip_error_detail(
            &IP_ERROR_WRONG_ARITY,
            BUILTIN_ADD_REQUIRED,
            "expected an argument name",
        ));
    };
    if !extra.is_empty() {
        // Validation functions are unresolved in the approved export;
        // explicit rejection instead of silent acceptance (FR-015-08).
        return Err(ip_error(
            &IP_ERROR_VALIDATORS_UNSUPPORTED,
            BUILTIN_ADD_REQUIRED,
        ));
    }
    let name = argument_name(name_arg, BUILTIN_ADD_REQUIRED)?;
    let obj = parser_object(&handle, BUILTIN_ADD_REQUIRED)?;
    ensure_name_unregistered(&obj, &name, BUILTIN_ADD_REQUIRED)?;
    let mut required = schema_cell(&obj, PROP_REQUIRED, BUILTIN_ADD_REQUIRED)?;
    required.push(char_row(&name));
    let updated = row_cell(required, BUILTIN_ADD_REQUIRED)?;
    write_parser_props(
        &handle,
        BUILTIN_ADD_REQUIRED,
        vec![(PROP_REQUIRED, updated)],
    )?;
    Ok(parser)
}

/// `addParameter(p, paramName, defaultVal)` — claim
/// inputParser.results-fields (FR-015-03, FR-015-04).
#[runtime_builtin(
    name = "inputParser.addParameter",
    descriptor(
        crate::builtins::introspection::input_parser::INPUT_PARSER_ADD_PARAMETER_DESCRIPTOR
    ),
    builtin_path = "crate::builtins::introspection::input_parser"
)]
fn input_parser_add_parameter(parser: Value, rest: Vec<Value>) -> crate::BuiltinResult<Value> {
    let handle = parser_handle(&parser, BUILTIN_ADD_PARAMETER)?;
    if rest.len() < 2 {
        return Err(ip_error_detail(
            &IP_ERROR_WRONG_ARITY,
            BUILTIN_ADD_PARAMETER,
            "expected a parameter name and a default value",
        ));
    }
    if rest.len() > 2 {
        return Err(ip_error(
            &IP_ERROR_VALIDATORS_UNSUPPORTED,
            BUILTIN_ADD_PARAMETER,
        ));
    }
    let name = argument_name(&rest[0], BUILTIN_ADD_PARAMETER)?;
    let default = rest[1].clone();
    let obj = parser_object(&handle, BUILTIN_ADD_PARAMETER)?;
    ensure_name_unregistered(&obj, &name, BUILTIN_ADD_PARAMETER)?;
    let mut names = schema_cell(&obj, PROP_PARAM_NAMES, BUILTIN_ADD_PARAMETER)?;
    let mut defaults = schema_cell(&obj, PROP_PARAM_DEFAULTS, BUILTIN_ADD_PARAMETER)?;
    names.push(char_row(&name));
    defaults.push(default);
    let names_cell = row_cell(names, BUILTIN_ADD_PARAMETER)?;
    let defaults_cell = row_cell(defaults, BUILTIN_ADD_PARAMETER)?;
    write_parser_props(
        &handle,
        BUILTIN_ADD_PARAMETER,
        vec![
            (PROP_PARAM_NAMES, names_cell),
            (PROP_PARAM_DEFAULTS, defaults_cell),
        ],
    )?;
    Ok(parser)
}

/// `parse(p, varargin)` — claims inputParser.results-fields,
/// inputParser.using-defaults (FR-015-02..06).
#[runtime_builtin(
    name = "inputParser.parse",
    descriptor(crate::builtins::introspection::input_parser::INPUT_PARSER_PARSE_DESCRIPTOR),
    builtin_path = "crate::builtins::introspection::input_parser"
)]
fn input_parser_parse(parser: Value, rest: Vec<Value>) -> crate::BuiltinResult<Value> {
    let handle = parser_handle(&parser, BUILTIN_PARSE)?;
    let obj = parser_object(&handle, BUILTIN_PARSE)?;
    let required = schema_names(&obj, PROP_REQUIRED, BUILTIN_PARSE)?;
    let param_names = schema_names(&obj, PROP_PARAM_NAMES, BUILTIN_PARSE)?;
    let param_defaults = schema_cell(&obj, PROP_PARAM_DEFAULTS, BUILTIN_PARSE)?;

    let n_required = required.len();
    if rest.len() < n_required {
        return Err(ip_error_detail(
            &IP_ERROR_MISSING_REQUIRED,
            BUILTIN_PARSE,
            format!(
                "required argument '{}' was not supplied",
                required[rest.len()]
            ),
        ));
    }
    let (positional, name_value) = rest.split_at(n_required);
    if name_value.len() % 2 != 0 {
        return Err(ip_error_detail(
            &IP_ERROR_NAME_VALUE_MISMATCH,
            BUILTIN_PARSE,
            format!("{} trailing input(s)", name_value.len()),
        ));
    }
    let mut supplied: Vec<(String, Value)> = Vec::with_capacity(name_value.len() / 2);
    for pair in name_value.chunks_exact(2) {
        let Some(name) = text_scalar(&pair[0]) else {
            return Err(ip_error_detail(
                &IP_ERROR_PARAMETER_NAME_INVALID,
                BUILTIN_PARSE,
                format!("got {:?}", pair[0]),
            ));
        };
        // Exact, case-sensitive matching: documented independent choice
        // (case sensitivity is unresolved in the approved export).
        if !param_names.iter().any(|p| p == &name) {
            return Err(ip_error_detail(
                &IP_ERROR_UNKNOWN_PARAMETER,
                BUILTIN_PARSE,
                format!("'{name}'"),
            ));
        }
        if supplied.iter().any(|(n, _)| n == &name) {
            return Err(ip_error_detail(
                &IP_ERROR_DUPLICATE_PARAMETER,
                BUILTIN_PARSE,
                format!("'{name}'"),
            ));
        }
        supplied.push((name, pair[1].clone()));
    }

    // Results: required in registration order, then parameters in
    // registration order (field order beyond the observed surface is a
    // RunMat choice; StructValue preserves insertion order).
    let mut results = StructValue::new();
    for (name, value) in required.iter().zip(positional.iter()) {
        results.insert(name.clone(), value.clone());
    }
    let mut using_defaults: Vec<Value> = Vec::new();
    for (name, default) in param_names.iter().zip(param_defaults.iter()) {
        if let Some((_, value)) = supplied.iter().find(|(n, _)| n == name) {
            results.insert(name.clone(), value.clone());
        } else {
            results.insert(name.clone(), default.clone());
            // Claim inputParser.using-defaults: defaulted parameters are
            // listed; element type char row is a documented choice.
            using_defaults.push(char_row(name));
        }
    }
    let using_defaults_cell = row_cell(using_defaults, BUILTIN_PARSE)?;
    write_parser_props(
        &handle,
        BUILTIN_PARSE,
        vec![
            (PROP_RESULTS, Value::Struct(results)),
            (PROP_USING_DEFAULTS, using_defaults_cell),
        ],
    )?;
    Ok(parser)
}

// ---------------------------------------------------------------------------
// Tests — pure in-process; no filesystem, no network, no MATLAB
// (Constitution II; SC-015-4). Tier naming: `normative_*` (observed cases,
// claim-cited), `dual_syntax_*` (FR-015-07 dispatch coverage),
// `unresolved_choice_*` (documented independent choices, never asserted as
// MATLAB-conformant). No summary-derived tier exists for this feature.
// ---------------------------------------------------------------------------

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::builtins::introspection::class::class_name_for_value;
    use futures::executor::block_on;

    fn construct() -> Value {
        input_parser_builtin(Vec::new()).expect("inputParser constructor")
    }

    fn add_required(parser: &Value, rest: Vec<Value>) -> BuiltinResult<Value> {
        input_parser_add_required(parser.clone(), rest)
    }

    fn add_parameter(parser: &Value, rest: Vec<Value>) -> BuiltinResult<Value> {
        input_parser_add_parameter(parser.clone(), rest)
    }

    fn parse(parser: &Value, rest: Vec<Value>) -> BuiltinResult<Value> {
        input_parser_parse(parser.clone(), rest)
    }

    fn property(parser: &Value, name: &str) -> Value {
        crate::call_builtin(
            "getfield",
            &[parser.clone(), Value::String(name.to_string())],
        )
        .unwrap_or_else(|e| panic!("getfield {name}: {e:?}"))
    }

    fn results_field(parser: &Value, field: &str) -> Value {
        match property(parser, PROP_RESULTS) {
            Value::Struct(st) => st
                .fields
                .get(field)
                .unwrap_or_else(|| panic!("Results has no field '{field}'"))
                .clone(),
            other => panic!("Results is not a struct: {other:?}"),
        }
    }

    fn identifier_of(err: RuntimeError) -> String {
        err.identifier().unwrap_or("<none>").to_string()
    }

    /// Observed case `construct-class`: class(inputParser) -> 'inputParser'
    /// (char 1x11). [FR-015-01; inputParser.signature-primary,
    /// inputParser.results-fields] Note: the char-vs-string representation
    /// of `class`'s own return value is RunMat's pre-existing convention
    /// (shared by all builtins); the normative content and length are
    /// asserted here.
    #[test]
    fn normative_construct_class_is_inputparser() {
        let p = construct();
        let name = class_name_for_value(&p);
        assert_eq!(name, "inputParser");
        assert_eq!(name.chars().count(), 11); // observed size [1, 11]
    }

    /// Observed cases `results-class` + `required-value`:
    /// addRequired('x') + parse(5) -> class(Results) == 'struct',
    /// Results.x == 5 (double 1x1). [FR-015-02; inputParser.results-fields]
    #[test]
    fn normative_required_chain_results_struct_and_value() {
        let p = construct();
        add_required(&p, vec![char_row("x")]).expect("addRequired");
        parse(&p, vec![Value::Num(5.0)]).expect("parse");
        let results = property(&p, PROP_RESULTS);
        let class_name = class_name_for_value(&results);
        assert_eq!(class_name, "struct");
        assert_eq!(class_name.chars().count(), 6); // observed size [1, 6]
        assert_eq!(results_field(&p, "x"), Value::Num(5.0));
    }

    /// Observed case `param-value`: addParameter('tol', 0.1) +
    /// parse('tol', 0.2) -> Results.tol == 0.2 (double 1x1; observation
    /// renders the double nearest 0.2 as 0.20000000000000001).
    /// [FR-015-03; inputParser.results-fields]
    #[test]
    fn normative_param_supplied_overrides_default() {
        let p = construct();
        add_parameter(&p, vec![char_row("tol"), Value::Num(0.1)]).expect("addParameter");
        parse(&p, vec![char_row("tol"), Value::Num(0.2)]).expect("parse");
        assert_eq!(results_field(&p, "tol"), Value::Num(0.2));
    }

    /// Observed case `param-default`: addParameter('tol', 0.1) + parse() ->
    /// Results.tol == 0.1. [FR-015-04; inputParser.results-fields]
    #[test]
    fn normative_param_default_used_when_omitted() {
        let p = construct();
        add_parameter(&p, vec![char_row("tol"), Value::Num(0.1)]).expect("addParameter");
        parse(&p, Vec::new()).expect("parse");
        assert_eq!(results_field(&p, "tol"), Value::Num(0.1));
    }

    /// Observed case `using-defaults`: with one parameter left at its
    /// default, UsingDefaults is a 1x1 cell. [FR-015-05;
    /// inputParser.using-defaults]
    #[test]
    fn normative_using_defaults_cell_1x1() {
        let p = construct();
        add_parameter(&p, vec![char_row("tol"), Value::Num(0.1)]).expect("addParameter");
        parse(&p, Vec::new()).expect("parse");
        match property(&p, PROP_USING_DEFAULTS) {
            Value::Cell(cell) => {
                assert_eq!(cell.rows, 1);
                assert_eq!(cell.cols, 1);
            }
            other => panic!("UsingDefaults is not a cell: {other:?}"),
        }
    }

    /// FR-015-07: method-call syntax through the object dispatch machinery
    /// (`call_method` -> lookup_method -> "inputParser.parse" builtin).
    #[test]
    fn dual_syntax_method_call_via_call_method() {
        let p = construct();
        block_on(
            crate::builtins::introspection::call_method::dispatch_call_method(
                p.clone(),
                "addParameter".to_string(),
                vec![char_row("tol"), Value::Num(0.1)],
            ),
        )
        .expect("p.addParameter('tol', 0.1)");
        block_on(
            crate::builtins::introspection::call_method::dispatch_call_method(
                p.clone(),
                "parse".to_string(),
                vec![char_row("tol"), Value::Num(0.2)],
            ),
        )
        .expect("p.parse('tol', 0.2)");
        assert_eq!(results_field(&p, "tol"), Value::Num(0.2));
    }

    /// FR-015-07: function-call syntax through the dispatcher's
    /// registered-instance-method fallback (`parse(p, 5)` with no top-level
    /// `parse` builtin registered).
    #[test]
    fn dual_syntax_function_call_via_dispatcher() {
        let p = construct();
        crate::call_builtin("addRequired", &[p.clone(), char_row("x")])
            .expect("addRequired(p, 'x')");
        crate::call_builtin("parse", &[p.clone(), Value::Num(5.0)]).expect("parse(p, 5)");
        assert_eq!(results_field(&p, "x"), Value::Num(5.0));
    }

    /// SC-015-2: class and dotted-method builtins are discoverable.
    #[test]
    fn registration_discoverable_class_and_builtins() {
        let _p = construct();
        let class_def = runmat_builtins::get_class(CLASS_NAME).expect("class registered");
        for method in ["addRequired", "addParameter", "parse"] {
            assert!(class_def.methods.contains_key(method), "missing {method}");
        }
        assert!(!class_def.methods.contains_key("addOptional"));
        for builtin in [
            BUILTIN_CONSTRUCTOR,
            BUILTIN_ADD_REQUIRED,
            BUILTIN_ADD_PARAMETER,
            BUILTIN_PARSE,
        ] {
            assert!(
                runmat_builtins::builtin_function_by_name(builtin).is_some(),
                "builtin '{builtin}' not registered"
            );
        }
    }

    /// FR-015-06 (documented choice side): aliases share the GC target, so
    /// registrations/parses through one alias are visible through another.
    /// Aliasing itself is unobserved in the export (`unresolved_choice`).
    #[test]
    fn unresolved_choice_alias_sees_mutations() {
        let p = construct();
        let alias = p.clone();
        add_parameter(&p, vec![char_row("tol"), Value::Num(0.1)]).expect("addParameter");
        parse(&alias, Vec::new()).expect("parse via alias");
        assert_eq!(results_field(&p, "tol"), Value::Num(0.1));
    }

    /// Documented choice: UsingDefaults elements are the registered names as
    /// char row vectors ("listed" per approved behaviour text; element type
    /// unobserved).
    #[test]
    fn unresolved_choice_using_defaults_lists_name_as_char() {
        let p = construct();
        add_parameter(&p, vec![char_row("tol"), Value::Num(0.1)]).expect("addParameter");
        parse(&p, Vec::new()).expect("parse");
        match property(&p, PROP_USING_DEFAULTS) {
            Value::Cell(cell) => {
                assert_eq!(cell.data.len(), 1);
                assert_eq!(cell.data[0], char_row("tol"));
            }
            other => panic!("UsingDefaults is not a cell: {other:?}"),
        }
    }

    /// Documented choice: pre-parse Results is an empty struct and
    /// UsingDefaults a 1x0 cell (unobserved in the export).
    #[test]
    fn unresolved_choice_pre_parse_properties() {
        let p = construct();
        match property(&p, PROP_RESULTS) {
            Value::Struct(st) => assert!(st.fields.is_empty()),
            other => panic!("Results is not a struct: {other:?}"),
        }
        match property(&p, PROP_USING_DEFAULTS) {
            Value::Cell(cell) => {
                assert_eq!(cell.rows, 1);
                assert_eq!(cell.cols, 0);
            }
            other => panic!("UsingDefaults is not a cell: {other:?}"),
        }
    }

    /// FR-015-08: validation functions are unresolved in the export; a third
    /// argument is rejected loudly, never silently ignored.
    #[test]
    fn unresolved_choice_validator_argument_rejected() {
        let p = construct();
        let err = add_required(&p, vec![char_row("x"), Value::Num(1.0)]).unwrap_err();
        assert_eq!(
            identifier_of(err),
            "RunMat:inputParser:ValidatorsUnsupported"
        );
        let err =
            add_parameter(&p, vec![char_row("tol"), Value::Num(0.1), Value::Num(1.0)]).unwrap_err();
        assert_eq!(
            identifier_of(err),
            "RunMat:inputParser:ValidatorsUnsupported"
        );
    }

    /// FR-015-08: unknown parameter names error (RunMat wording).
    #[test]
    fn unresolved_choice_unknown_parameter_errors() {
        let p = construct();
        add_parameter(&p, vec![char_row("tol"), Value::Num(0.1)]).expect("addParameter");
        let err = parse(&p, vec![char_row("bad"), Value::Num(1.0)]).unwrap_err();
        assert_eq!(identifier_of(err), "RunMat:inputParser:UnknownParameter");
    }

    /// FR-015-08: parameter matching is exact and case-sensitive
    /// (documented choice; case sensitivity is unresolved in the export).
    #[test]
    fn unresolved_choice_case_sensitive_parameter_names() {
        let p = construct();
        add_parameter(&p, vec![char_row("tol"), Value::Num(0.1)]).expect("addParameter");
        let err = parse(&p, vec![char_row("Tol"), Value::Num(0.2)]).unwrap_err();
        assert_eq!(identifier_of(err), "RunMat:inputParser:UnknownParameter");
    }

    /// FR-015-08: missing required positional errors.
    #[test]
    fn unresolved_choice_missing_required_errors() {
        let p = construct();
        add_required(&p, vec![char_row("x")]).expect("addRequired");
        let err = parse(&p, Vec::new()).unwrap_err();
        assert_eq!(identifier_of(err), "RunMat:inputParser:MissingRequired");
    }

    /// FR-015-08: a dangling name with no value errors.
    #[test]
    fn unresolved_choice_dangling_name_value_errors() {
        let p = construct();
        add_parameter(&p, vec![char_row("tol"), Value::Num(0.1)]).expect("addParameter");
        let err = parse(&p, vec![char_row("tol")]).unwrap_err();
        assert_eq!(identifier_of(err), "RunMat:inputParser:NameValueMismatch");
    }

    /// FR-015-08: registering the same name twice errors.
    #[test]
    fn unresolved_choice_duplicate_registration_rejected() {
        let p = construct();
        add_parameter(&p, vec![char_row("tol"), Value::Num(0.1)]).expect("addParameter");
        let err = add_parameter(&p, vec![char_row("tol"), Value::Num(0.2)]).unwrap_err();
        assert_eq!(identifier_of(err), "RunMat:inputParser:DuplicateName");
        let err = add_required(&p, vec![char_row("tol")]).unwrap_err();
        assert_eq!(identifier_of(err), "RunMat:inputParser:DuplicateName");
    }

    /// FR-015-08: supplying one parameter twice in a single parse errors.
    #[test]
    fn unresolved_choice_duplicate_parameter_in_parse_rejected() {
        let p = construct();
        add_parameter(&p, vec![char_row("tol"), Value::Num(0.1)]).expect("addParameter");
        let err = parse(
            &p,
            vec![
                char_row("tol"),
                Value::Num(0.2),
                char_row("tol"),
                Value::Num(0.3),
            ],
        )
        .unwrap_err();
        assert_eq!(identifier_of(err), "RunMat:inputParser:DuplicateParameter");
    }

    /// FR-015-08: the constructor takes no inputs; extras error.
    #[test]
    fn unresolved_choice_constructor_rejects_inputs() {
        let err = input_parser_builtin(vec![Value::Num(1.0)]).unwrap_err();
        assert_eq!(identifier_of(err), "RunMat:inputParser:TooManyInputs");
    }

    /// FR-015-02..05 combined chain: required + supplied parameter + defaulted
    /// parameter in one parse; UsingDefaults lists only the defaulted one.
    #[test]
    fn normative_combined_chain_required_and_parameters() {
        let p = construct();
        add_required(&p, vec![char_row("x")]).expect("addRequired");
        add_parameter(&p, vec![char_row("tol"), Value::Num(0.1)]).expect("addParameter tol");
        add_parameter(&p, vec![char_row("verbose"), Value::Bool(false)])
            .expect("addParameter verbose");
        parse(
            &p,
            vec![Value::Num(5.0), char_row("verbose"), Value::Bool(true)],
        )
        .expect("parse");
        assert_eq!(results_field(&p, "x"), Value::Num(5.0));
        assert_eq!(results_field(&p, "tol"), Value::Num(0.1));
        assert_eq!(results_field(&p, "verbose"), Value::Bool(true));
        match property(&p, PROP_USING_DEFAULTS) {
            Value::Cell(cell) => {
                assert_eq!((cell.rows, cell.cols), (1, 1));
                assert_eq!(cell.data[0], char_row("tol"));
            }
            other => panic!("UsingDefaults is not a cell: {other:?}"),
        }
    }
}
