//! MATLAB-compatible `methods` builtin for RunMat.
//!
//! Clean-room feature `specs/032-object-introspection/` (SpecKit id
//! `032-object-introspection`). Normative behaviour comes exclusively from the
//! approved Tier B batch-3 export `methods` 0.2.0 (claims
//! `methods.signature-primary`, `methods.cellstr-names`; source commit pin
//! `1b019121bb747e63d69a3eb005534cb60669e5f4`).
//!
//! Gate-ratified framing: the export observed `methods(inputParser)` as a
//! 19x1 cellstr, but the specific member list and count are
//! MATLAB-object-specific and are NOT asserted. The normative claim is that
//! `methods` returns an N-by-1 cell array of char vectors naming the actual
//! RunMat object's (public) methods. For RunMat's `inputParser` (feature 015)
//! those are `addRequired`, `addParameter`, `parse`. See
//! `specs/032-object-introspection/spec.md` "Value non-assertion".

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
use crate::builtins::introspection::properties::class_name_of_argument;
use crate::{build_runtime_error, BuiltinResult, RuntimeError};

#[runmat_macros::register_gpu_spec(builtin_path = "crate::builtins::introspection::methods")]
pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "methods",
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
    notes: "Host-only class introspection; providers do not participate. The method list comes from the class registry, never from device memory.",
};

#[runmat_macros::register_fusion_spec(builtin_path = "crate::builtins::introspection::methods")]
pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "methods",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Not eligible for fusion; methods executes on the host and returns a cell array.",
};

const BUILTIN_NAME: &str = "methods";

const METHODS_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "c",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "N-by-1 cell array of method-name character vectors.",
}];

const METHODS_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "A",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Object, class reference, or class-name text.",
}];

const METHODS_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "c = methods(A)",
    inputs: &METHODS_INPUTS,
    outputs: &METHODS_OUTPUT,
}];

const METHODS_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.METHODS.INVALID_INPUT",
    identifier: Some("RunMat:methods:InvalidInput"),
    when: "Input is neither an object, a class reference, nor class-name text.",
    message: "methods: expected an object or a class name",
};

const METHODS_ERROR_UNKNOWN_CLASS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.METHODS.UNKNOWN_CLASS",
    identifier: Some("RunMat:methods:UnknownClass"),
    when: "The supplied class name does not name a registered class.",
    message: "methods: no registered class matches the supplied name",
};

const METHODS_ERROR_INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.METHODS.INTERNAL",
    identifier: Some("RunMat:methods:InternalError"),
    when: "Building the output cell array fails.",
    message: "methods: internal error",
};

const METHODS_ERRORS: [BuiltinErrorDescriptor; 3] = [
    METHODS_ERROR_INVALID_INPUT,
    METHODS_ERROR_UNKNOWN_CLASS,
    METHODS_ERROR_INTERNAL,
];

pub const METHODS_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &METHODS_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &METHODS_ERRORS,
};

fn methods_error(error: &'static BuiltinErrorDescriptor) -> RuntimeError {
    let mut builder = build_runtime_error(error.message).with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

/// Collect the public method names declared for `class_name` and its
/// ancestors. Names are de-duplicated and returned in ascending order (a
/// documented RunMat choice: declaration order is unresolved in the export).
pub(crate) fn public_method_names(class_name: &str) -> Vec<String> {
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
        for (method_name, method) in &class_def.methods {
            if method.access == Access::Public {
                names.insert(method_name.clone());
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
    crate::make_cell(cells, rows, 1).map_err(|_| methods_error(&METHODS_ERROR_INTERNAL))
}

#[runtime_builtin(
    name = "methods",
    category = "introspection",
    summary = "List the public method names of an object or class as a column cell array of char vectors.",
    keywords = "methods,object,class,introspection,method names,cellstr",
    descriptor(crate::builtins::introspection::methods::METHODS_DESCRIPTOR),
    builtin_path = "crate::builtins::introspection::methods"
)]
fn methods_builtin(value: Value) -> crate::BuiltinResult<Value> {
    let Some(class_name) = class_name_of_argument(&value) else {
        return Err(methods_error(&METHODS_ERROR_INVALID_INPUT));
    };
    let is_instance = matches!(
        value,
        Value::Object(_) | Value::HandleObject(_) | Value::ClassRef(_)
    );
    if !is_instance && runmat_builtins::get_class(&class_name).is_none() {
        return Err(methods_error(&METHODS_ERROR_UNKNOWN_CLASS));
    }
    names_to_column_cell(public_method_names(&class_name))
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
    use runmat_builtins::{Access, ClassDef, MethodDef, Value};
    use std::collections::HashMap;

    fn run(value: Value) -> BuiltinResult<Value> {
        methods_builtin(value)
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
        assert_eq!(cell.cols, 1, "expected a single column");
        assert_eq!(cell.rows, cell.data.len());
        for entry in &cell.data {
            match entry {
                Value::CharArray(ca) => assert!(ca.rows <= 1, "expected char row vector element"),
                other => panic!("expected char cell element, got {other:?}"),
            }
        }
    }

    fn method(name: &str, access: Access) -> MethodDef {
        MethodDef {
            name: name.to_string(),
            is_static: false,
            is_abstract: false,
            is_sealed: false,
            access,
            function_name: format!("test.{name}"),
            implicit_class_argument: None,
        }
    }

    fn register_test_class(name: &str, methods: Vec<MethodDef>) {
        let mut method_map = HashMap::new();
        for m in methods {
            method_map.insert(m.name.clone(), m);
        }
        runmat_builtins::register_class(ClassDef {
            name: name.to_string(),
            parent: None,
            properties: HashMap::new(),
            methods: method_map,
        });
    }

    fn make_input_parser() -> Value {
        crate::call_builtin("inputParser", &[]).expect("construct inputParser")
    }

    /// Observed case `object` + `elem-type`: `methods(inputParser)` returns a
    /// cell whose elements are char vectors (iscellstr true) and whose shape is
    /// N-by-1. [FR-032-02; methods.signature-primary, methods.cellstr-names].
    /// The member COUNT (MATLAB observed 19) is object-specific and
    /// deliberately not asserted.
    #[test]
    fn observed_input_parser_methods_is_cellstr_column() {
        let p = make_input_parser();
        let result = run(p).expect("methods(inputParser)");
        assert_is_cellstr_column(&result);
        // RunMat's inputParser declares addRequired, addParameter, parse
        // (feature 015).
        let names = cell_names(&result);
        for expected in ["addRequired", "addParameter", "parse"] {
            assert!(
                names.contains(&expected.to_string()),
                "missing {expected}; names: {names:?}"
            );
        }
    }

    /// Non-assertion guard (value non-assertion): the observed MATLAB count of
    /// 19 is NOT claimed. We assert only class cell + column shape +
    /// all-elements-char, never `len == 19`.
    #[test]
    fn unresolved_choice_method_count_not_asserted() {
        let p = make_input_parser();
        let result = run(p).expect("methods(inputParser)");
        assert_is_cellstr_column(&result);
        let names = cell_names(&result);
        assert_ne!(names.len(), 19, "RunMat's inputParser is not MATLAB's");
    }

    /// Documented choice: a class with no public methods yields a 0-by-1 cell.
    #[test]
    fn unresolved_choice_no_methods_is_empty_column() {
        register_test_class("runmat.unittest.MethodsEmpty", Vec::new());
        let result = run(Value::ClassRef("runmat.unittest.MethodsEmpty".to_string()))
            .expect("methods(empty class)");
        match result {
            Value::Cell(cell) => {
                assert_eq!(cell.rows, 0);
                assert_eq!(cell.cols, 1);
            }
            other => panic!("expected cell array, got {other:?}"),
        }
    }

    /// Documented choice: the class-name text form (unobserved) resolves a
    /// registered class and returns its sorted public methods.
    #[test]
    fn unresolved_choice_class_name_text_form() {
        register_test_class(
            "runmat.unittest.MethodsAB",
            vec![
                method("run", Access::Public),
                method("build", Access::Public),
            ],
        );
        let result = run(Value::from("runmat.unittest.MethodsAB")).expect("methods('...')");
        assert_is_cellstr_column(&result);
        assert_eq!(
            cell_names(&result),
            vec!["build".to_string(), "run".to_string()]
        );
    }

    /// Documented choice: non-public methods are excluded from the public
    /// method list.
    #[test]
    fn unresolved_choice_private_methods_excluded() {
        register_test_class(
            "runmat.unittest.MethodsMixed",
            vec![
                method("open", Access::Public),
                method("secret", Access::Private),
            ],
        );
        let result = run(Value::ClassRef("runmat.unittest.MethodsMixed".to_string()))
            .expect("methods(mixed)");
        assert_eq!(cell_names(&result), vec!["open".to_string()]);
    }

    /// Documented choice: an unregistered class-name text errors loudly.
    #[test]
    fn unresolved_choice_unknown_class_name_errors() {
        let err = run(Value::from("runmat.unittest.NoSuchClass")).unwrap_err();
        assert_eq!(
            err.identifier().unwrap_or("<none>"),
            "RunMat:methods:UnknownClass"
        );
    }

    /// Documented choice: a plain numeric value is rejected.
    #[test]
    fn unresolved_choice_non_object_input_errors() {
        let err = run(Value::Num(5.0)).unwrap_err();
        assert_eq!(
            err.identifier().unwrap_or("<none>"),
            "RunMat:methods:InvalidInput"
        );
    }

    /// Descriptor advertises the observed primary signature `c = methods(A)`.
    #[test]
    fn descriptor_exposes_primary_signature() {
        let labels: Vec<&str> = METHODS_DESCRIPTOR
            .signatures
            .iter()
            .map(|sig| sig.label)
            .collect();
        assert!(labels.contains(&"c = methods(A)"));
    }

    /// Macro registration: the builtin is discoverable by name.
    #[test]
    fn registration_discoverable_by_name() {
        assert!(runmat_builtins::builtin_function_by_name("methods").is_some());
    }

    /// GPU/fusion metadata is registered under the builtin's own name.
    #[test]
    fn gpu_and_fusion_specs_match_name() {
        assert_eq!(GPU_SPEC.name, "methods");
        assert_eq!(FUSION_SPEC.name, "methods");
    }
}
