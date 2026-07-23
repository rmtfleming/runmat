//! MATLAB-compatible `properties` builtin for RunMat.
//!
//! Clean-room feature `specs/032-object-introspection/` (SpecKit id
//! `032-object-introspection`). Normative behaviour comes exclusively from the
//! approved Tier B batch-3 export `properties` 0.2.0 (claims
//! `properties.signature-primary`, `properties.cellstr-names`; source commit
//! pin `1b019121bb747e63d69a3eb005534cb60669e5f4`).
//!
//! Gate-ratified framing: the export observed `properties(inputParser)` as a
//! 9x1 cellstr, but the specific member list and count are
//! MATLAB-object-specific and are NOT asserted. The normative claim is that
//! `properties` returns an N-by-1 cell array of char vectors naming the
//! actual RunMat object's (public) properties. For RunMat's `inputParser`
//! (feature 015) those are its registered `PropertyDef`s (`Results`,
//! `UsingDefaults`). See `specs/032-object-introspection/spec.md` "Value
//! non-assertion".

use std::collections::BTreeSet;
use std::collections::HashSet;

use runmat_builtins::{
    Access, BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    CharArray, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ShapeRequirements,
};
use crate::{build_runtime_error, BuiltinResult, RuntimeError};

#[runmat_macros::register_gpu_spec(builtin_path = "crate::builtins::introspection::properties")]
pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "properties",
    op_kind: GpuOpKind::Custom("introspection"),
    supported_precisions: &[],
    broadcast: BroadcastSemantics::None,
    provider_hooks: &[],
    constant_strategy: ConstantStrategy::InlineLiteral,
    residency: ResidencyPolicy::InheritInputs,
    nan_mode: ReductionNaN::Include,
    two_pass_threshold: None,
    workgroup_size: None,
    accepts_nan_mode: false,
    notes: "Host-only class introspection; providers do not participate. The property list comes from the class registry, never from device memory.",
};

#[runmat_macros::register_fusion_spec(builtin_path = "crate::builtins::introspection::properties")]
pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "properties",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Not eligible for fusion; properties executes on the host and returns a cell array.",
};

const BUILTIN_NAME: &str = "properties";

const PROPERTIES_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "c",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "N-by-1 cell array of property-name character vectors.",
}];

const PROPERTIES_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "A",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Object, class reference, or class-name text.",
}];

const PROPERTIES_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "c = properties(A)",
    inputs: &PROPERTIES_INPUTS,
    outputs: &PROPERTIES_OUTPUT,
}];

const PROPERTIES_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.PROPERTIES.INVALID_INPUT",
    identifier: Some("RunMat:properties:InvalidInput"),
    when: "Input is neither an object, a class reference, nor class-name text.",
    message: "properties: expected an object or a class name",
};

const PROPERTIES_ERROR_UNKNOWN_CLASS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.PROPERTIES.UNKNOWN_CLASS",
    identifier: Some("RunMat:properties:UnknownClass"),
    when: "The supplied class name does not name a registered class.",
    message: "properties: no registered class matches the supplied name",
};

const PROPERTIES_ERROR_INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.PROPERTIES.INTERNAL",
    identifier: Some("RunMat:properties:InternalError"),
    when: "Building the output cell array fails.",
    message: "properties: internal error",
};

const PROPERTIES_ERRORS: [BuiltinErrorDescriptor; 3] = [
    PROPERTIES_ERROR_INVALID_INPUT,
    PROPERTIES_ERROR_UNKNOWN_CLASS,
    PROPERTIES_ERROR_INTERNAL,
];

pub const PROPERTIES_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &PROPERTIES_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &PROPERTIES_ERRORS,
};

fn properties_error(error: &'static BuiltinErrorDescriptor) -> RuntimeError {
    let mut builder = build_runtime_error(error.message).with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

/// Resolve the class name that a `properties` argument refers to.
///
/// Objects, handle objects, and class references resolve to their class; a
/// char row / string scalar is interpreted as a class name (the class-name
/// text form is unobserved in the export and is a documented RunMat choice).
pub(crate) fn class_name_of_argument(value: &Value) -> Option<String> {
    match value {
        Value::Object(obj) => Some(obj.class_name.clone()),
        Value::HandleObject(handle) if !handle.class_name.is_empty() => {
            Some(handle.class_name.clone())
        }
        Value::ClassRef(name) => Some(name.clone()),
        Value::CharArray(ca) if ca.rows == 1 => Some(ca.data.iter().collect()),
        Value::String(s) => Some(s.clone()),
        Value::StringArray(sa) if sa.data.len() == 1 => Some(sa.data[0].clone()),
        _ => None,
    }
}

/// Collect the public property names declared for `class_name` and its
/// ancestors. Names are de-duplicated and returned in ascending order (a
/// documented RunMat choice: declaration order is unresolved in the export).
pub(crate) fn public_property_names(class_name: &str) -> Vec<String> {
    let mut names: BTreeSet<String> = BTreeSet::new();
    let mut current = Some(class_name.to_string());
    let mut visited: HashSet<String> = HashSet::new();
    while let Some(name) = current {
        if !visited.insert(name.clone()) {
            break;
        }
        let Some(class_def) = runmat_builtins::get_class(&name) else {
            break;
        };
        for (prop_name, prop) in &class_def.properties {
            if prop.get_access == Access::Public {
                names.insert(prop_name.clone());
            }
        }
        current = class_def.parent.clone();
    }
    names.into_iter().collect()
}

fn names_to_column_cell(names: Vec<String>) -> BuiltinResult<Value> {
    let rows = names.len();
    let cells: Vec<Value> = names
        .into_iter()
        .map(|name| Value::CharArray(CharArray::new_row(&name)))
        .collect();
    crate::make_cell(cells, rows, 1).map_err(|_| properties_error(&PROPERTIES_ERROR_INTERNAL))
}

#[runtime_builtin(
    name = "properties",
    category = "introspection",
    summary = "List the public property names of an object or class as a column cell array of char vectors.",
    keywords = "properties,object,class,introspection,property names,cellstr",
    descriptor(crate::builtins::introspection::properties::PROPERTIES_DESCRIPTOR),
    builtin_path = "crate::builtins::introspection::properties"
)]
fn properties_builtin(value: Value) -> crate::BuiltinResult<Value> {
    let Some(class_name) = class_name_of_argument(&value) else {
        return Err(properties_error(&PROPERTIES_ERROR_INVALID_INPUT));
    };
    // An object/handle/classref always resolves to a class name; text is only
    // valid if it names a registered class.
    let is_instance = matches!(
        value,
        Value::Object(_) | Value::HandleObject(_) | Value::ClassRef(_)
    );
    if !is_instance && runmat_builtins::get_class(&class_name).is_none() {
        return Err(properties_error(&PROPERTIES_ERROR_UNKNOWN_CLASS));
    }
    names_to_column_cell(public_property_names(&class_name))
}

// ---------------------------------------------------------------------------
// Tests — pure in-process; no filesystem, no network, no MATLAB
// (Constitution II). Tiers: `observed_*` reproduce the export's observed
// shape/class (using RunMat's own `inputParser`); `unresolved_choice_*` cover
// documented independent choices (empty member set, class-name text form) and
// the explicit non-assertion of the member count.
// ---------------------------------------------------------------------------

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use runmat_builtins::{Access, ClassDef, ObjectInstance, PropertyDef, Value};
    use std::collections::HashMap;

    fn run(value: Value) -> BuiltinResult<Value> {
        properties_builtin(value)
    }

    fn cell_names(value: &Value) -> Vec<String> {
        match value {
            Value::Cell(cell) => cell
                .data
                .iter()
                .map(|entry| match entry {
                    Value::CharArray(ca) => ca.data.iter().collect(),
                    other => panic!("expected char cell element, got {other:?}"),
                })
                .collect(),
            other => panic!("expected cell array, got {other:?}"),
        }
    }

    fn assert_is_cellstr_column(value: &Value) {
        let Value::Cell(cell) = value else {
            panic!("expected cell array, got {value:?}");
        };
        // N-by-1 shape: a single column.
        assert_eq!(cell.cols, 1, "expected a single column");
        assert_eq!(cell.rows, cell.data.len());
        // iscellstr: every element is a char vector (row).
        for entry in &cell.data {
            match entry {
                Value::CharArray(ca) => assert!(ca.rows <= 1, "expected char row vector element"),
                other => panic!("expected char cell element, got {other:?}"),
            }
        }
    }

    fn prop(name: &str, get_access: Access) -> PropertyDef {
        PropertyDef {
            name: name.to_string(),
            is_static: false,
            is_constant: false,
            is_dependent: false,
            get_access,
            set_access: Access::Public,
            default_value: None,
        }
    }

    fn register_test_class(name: &str, props: Vec<PropertyDef>) {
        let mut properties = HashMap::new();
        for p in props {
            properties.insert(p.name.clone(), p);
        }
        runmat_builtins::register_class(ClassDef {
            name: name.to_string(),
            parent: None,
            properties,
            methods: HashMap::new(),
        });
    }

    fn make_input_parser() -> Value {
        // Construct through the registry so the inputParser class is registered
        // on this thread (feature 015). No MATLAB is involved.
        crate::call_builtin("inputParser", &[]).expect("construct inputParser")
    }

    /// Observed case `object` + `elem-type`: `properties(inputParser)` returns
    /// a cell whose elements are char vectors (iscellstr true) and whose shape
    /// is N-by-1. [FR-032-01; properties.signature-primary,
    /// properties.cellstr-names / properties.output-class]. The member COUNT
    /// (MATLAB observed 9) is object-specific and deliberately not asserted.
    #[test]
    fn observed_input_parser_properties_is_cellstr_column() {
        let p = make_input_parser();
        let result = run(p).expect("properties(inputParser)");
        assert_is_cellstr_column(&result);
        // RunMat's inputParser declares Results and UsingDefaults (feature 015).
        let names = cell_names(&result);
        assert!(names.contains(&"Results".to_string()), "names: {names:?}");
        assert!(
            names.contains(&"UsingDefaults".to_string()),
            "names: {names:?}"
        );
    }

    /// Non-assertion guard (value non-assertion): the observed MATLAB count of
    /// 9 is NOT claimed. We assert only class cell + column shape +
    /// all-elements-char, never `len == 9`.
    #[test]
    fn unresolved_choice_property_count_not_asserted() {
        let p = make_input_parser();
        let result = run(p).expect("properties(inputParser)");
        assert_is_cellstr_column(&result);
        let names = cell_names(&result);
        // Explicitly assert the count is what RunMat's object actually has, not
        // MATLAB's 9. This documents the non-assertion boundary.
        assert_ne!(names.len(), 9, "RunMat's inputParser is not MATLAB's");
    }

    /// Documented choice: a class with no public properties yields a 0-by-1
    /// cell (empty cellstr column), not an error.
    #[test]
    fn unresolved_choice_no_properties_is_empty_column() {
        register_test_class("runmat.unittest.PropsEmpty", Vec::new());
        let result = run(Value::ClassRef("runmat.unittest.PropsEmpty".to_string()))
            .expect("properties(empty class)");
        match result {
            Value::Cell(cell) => {
                assert_eq!(cell.rows, 0);
                assert_eq!(cell.cols, 1);
            }
            other => panic!("expected cell array, got {other:?}"),
        }
    }

    /// Documented choice: the class-name text form (unobserved in the export)
    /// resolves a registered class and returns its sorted public properties.
    #[test]
    fn unresolved_choice_class_name_text_form() {
        register_test_class(
            "runmat.unittest.PropsAB",
            vec![prop("Beta", Access::Public), prop("Alpha", Access::Public)],
        );
        let result = run(Value::from("runmat.unittest.PropsAB")).expect("properties('...')");
        assert_is_cellstr_column(&result);
        assert_eq!(
            cell_names(&result),
            vec!["Alpha".to_string(), "Beta".to_string()]
        );
    }

    /// Documented choice: non-public properties are excluded from the public
    /// property list (private-get properties are not listed).
    #[test]
    fn unresolved_choice_private_properties_excluded() {
        register_test_class(
            "runmat.unittest.PropsMixed",
            vec![
                prop("Visible", Access::Public),
                prop("Hidden", Access::Private),
            ],
        );
        let result = run(Value::ClassRef("runmat.unittest.PropsMixed".to_string()))
            .expect("properties(mixed)");
        assert_eq!(cell_names(&result), vec!["Visible".to_string()]);
    }

    /// Documented choice: an unregistered class-name text errors loudly rather
    /// than returning an empty list.
    #[test]
    fn unresolved_choice_unknown_class_name_errors() {
        let err = run(Value::from("runmat.unittest.NoSuchClass")).unwrap_err();
        assert_eq!(
            err.identifier().unwrap_or("<none>"),
            "RunMat:properties:UnknownClass"
        );
    }

    /// Documented choice: a plain numeric value is not a class or object;
    /// RunMat rejects it (MATLAB error wording is unobserved).
    #[test]
    fn unresolved_choice_non_object_input_errors() {
        let err = run(Value::Num(5.0)).unwrap_err();
        assert_eq!(
            err.identifier().unwrap_or("<none>"),
            "RunMat:properties:InvalidInput"
        );
    }

    /// Descriptor advertises the observed primary signature `c = properties(A)`.
    #[test]
    fn descriptor_exposes_primary_signature() {
        let labels: Vec<&str> = PROPERTIES_DESCRIPTOR
            .signatures
            .iter()
            .map(|sig| sig.label)
            .collect();
        assert!(labels.contains(&"c = properties(A)"));
    }

    /// Macro registration: the builtin is discoverable by name.
    #[test]
    fn registration_discoverable_by_name() {
        assert!(runmat_builtins::builtin_function_by_name("properties").is_some());
    }

    /// GPU/fusion metadata is registered under the builtin's own name.
    #[test]
    fn gpu_and_fusion_specs_match_name() {
        assert_eq!(GPU_SPEC.name, "properties");
        assert_eq!(FUSION_SPEC.name, "properties");
    }
}
