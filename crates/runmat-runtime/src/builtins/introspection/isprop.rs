//! Clean-room `isprop` (does an object have a property of a given name).
//!
//! Clean-room feature `specs/046-isprop/`. Normative behaviour comes only from
//! the approved export `isprop` 0.2.0 (see that feature's `provenance.md`).
//! Unobserved points raise `RunMat:isprop:UnsupportedOption` or are
//! documented choices in the feature spec.

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{LogicalArray, Value};

use crate::builtins::common::map_control_flow_with_builtin;
use crate::builtins::introspection::properties::public_property_names;
use crate::builtins::plotting::properties::{get_properties, resolve_plot_handle, PlotHandle};
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const NAME: &str = "isprop";

/// Properties RunMat exposes on an MException value (field access).
const MEXCEPTION_PROPERTIES: [&str; 3] = ["identifier", "message", "stack"];

// ---------------------------------------------------------------------------
// Descriptor
// ---------------------------------------------------------------------------

const ISPROP_INPUTS: [BuiltinParamDescriptor; 2] = [
    BuiltinParamDescriptor {
        name: "obj",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Object, graphics handle (or array of handles), or any other value.",
    },
    BuiltinParamDescriptor {
        name: "name",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Property name (char row vector or string scalar).",
    },
];

const ISPROP_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "tf",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Logical: true where obj has the named property.",
}];

const ISPROP_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "tf = isprop(obj, name)",
    inputs: &ISPROP_INPUTS,
    outputs: &ISPROP_OUTPUT,
}];

const NOT_ENOUGH_INPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ISPROP.NOT_ENOUGH_INPUTS",
    identifier: Some("RunMat:isprop:NotEnoughInputs"),
    when: "Fewer than two arguments are given.",
    message: "isprop: not enough input arguments",
};

const TOO_MANY_INPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ISPROP.TOO_MANY_INPUTS",
    identifier: Some("RunMat:isprop:TooManyInputs"),
    when: "More than two arguments are given.",
    message: "isprop: too many input arguments",
};

const UNSUPPORTED_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ISPROP.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:isprop:UnsupportedOption"),
    when: "Unobserved forms: non-text name, class references, and a non-graphics object whose property matches only when case is ignored.",
    message: "isprop: this form is not yet specified",
};

const INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ISPROP.INTERNAL",
    identifier: Some("RunMat:isprop:Internal"),
    when: "The runtime cannot assemble the result.",
    message: "isprop: internal runtime failure",
};

const ISPROP_ERRORS: [BuiltinErrorDescriptor; 4] = [
    NOT_ENOUGH_INPUTS,
    TOO_MANY_INPUTS,
    UNSUPPORTED_OPTION,
    INTERNAL,
];

pub const ISPROP_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &ISPROP_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ISPROP_ERRORS,
};

fn isprop_error(
    descriptor: &'static BuiltinErrorDescriptor,
    detail: impl Into<String>,
) -> RuntimeError {
    let mut builder = build_runtime_error(format!("{NAME}: {}", detail.into())).with_builtin(NAME);
    if let Some(identifier) = descriptor.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

// ---------------------------------------------------------------------------
// Builtin
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "isprop",
    category = "introspection",
    summary = "True where an object has a property of the given name.",
    keywords = "isprop,property,object,class,introspection,graphics handle",
    descriptor(crate::builtins::introspection::isprop::ISPROP_DESCRIPTOR),
    builtin_path = "crate::builtins::introspection::isprop"
)]
async fn isprop_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    if args.len() < 2 {
        return Err(isprop_error(
            &NOT_ENOUGH_INPUTS,
            "expected isprop(obj, name)",
        ));
    }
    if args.len() > 2 {
        return Err(isprop_error(&TOO_MANY_INPUTS, "expected isprop(obj, name)"));
    }
    // isprop.string-name; other name forms: FR-046-07.
    let name = match &args[1] {
        Value::CharArray(chars) if chars.rows <= 1 => chars.data.iter().collect::<String>(),
        Value::String(text) => text.clone(),
        Value::StringArray(array) if array.data.len() == 1 => array.data[0].clone(),
        _ => {
            return Err(isprop_error(
                &UNSUPPORTED_OPTION,
                "name must be a char row vector or string scalar",
            ))
        }
    };
    let obj = gather_if_needed_async(&args[0])
        .await
        .map_err(|err| map_control_flow_with_builtin(err, NAME))?;

    match &obj {
        // isprop.non-objects: tables (and timetables, FR-046-09) give false.
        _ if crate::builtins::table::is_table_value(&obj) => Ok(Value::Bool(false)),
        // isprop.objects: class objects (FR-046-08, FR-046-11).
        Value::Object(object) => {
            let names = public_property_names(object.class_name.as_ref());
            Ok(Value::Bool(lookup_exact(&names, &name)?))
        }
        Value::HandleObject(handle) => {
            let names = public_property_names(handle.class_name.as_ref());
            Ok(Value::Bool(lookup_exact(&names, &name)?))
        }
        // FR-046-10: object arrays give a same-shape logical.
        Value::ObjectArray(array) => {
            let names = public_property_names(array.class_name().as_ref());
            let answer = lookup_exact(&names, &name)?;
            let shape = array.shape().to_vec();
            let count = shape.iter().product::<usize>();
            LogicalArray::new(vec![answer as u8; count], shape)
                .map(Value::LogicalArray)
                .map_err(|err| isprop_error(&INTERNAL, err))
        }
        // isprop.objects (MException 'message').
        Value::MException(_) => {
            let names: Vec<String> = MEXCEPTION_PROPERTIES
                .iter()
                .map(|s| s.to_string())
                .collect();
            Ok(Value::Bool(lookup_exact(&names, &name)?))
        }
        Value::ClassRef(_) => Err(isprop_error(
            &UNSUPPORTED_OPTION,
            "class references are not yet specified",
        )),
        // Graphics handles (isprop.objects, isprop.case-insensitive-graphics,
        // isprop.arrays); other numbers give false (isprop.non-objects).
        Value::Num(x) => Ok(Value::Bool(
            graphics_handles(&[*x]).is_some_and(|h| graphics_has(&h[0], &name)),
        )),
        Value::Tensor(tensor)
            if tensor.integer_storage().is_none()
                && tensor.numeric_dtype() == runmat_value::NumericDType::F64 =>
        {
            let values = tensor.materialize_f64();
            match graphics_handles(&values) {
                Some(handles) if !handles.is_empty() => {
                    let data = handles
                        .iter()
                        .map(|handle| graphics_has(handle, &name) as u8)
                        .collect();
                    LogicalArray::new(data, tensor.shape.clone())
                        .map(Value::LogicalArray)
                        .map_err(|err| isprop_error(&INTERNAL, err))
                }
                _ => Ok(Value::Bool(false)),
            }
        }
        // isprop.non-objects and FR-046-09.
        _ => Ok(Value::Bool(false)),
    }
}

/// Exact-case match gives true; a case-only match is unresolved
/// (isprop.q-case-objects, FR-046-08); otherwise false.
fn lookup_exact(names: &[String], name: &str) -> BuiltinResult<bool> {
    if names.iter().any(|candidate| candidate == name) {
        return Ok(true);
    }
    if names
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case(name))
    {
        return Err(isprop_error(
            &UNSUPPORTED_OPTION,
            format!("property '{name}' differs from a property only in case; case handling for this object is not yet specified"),
        ));
    }
    Ok(false)
}

/// Every element resolved as a live plot handle, or `None`.
fn graphics_handles(values: &[f64]) -> Option<Vec<PlotHandle>> {
    values
        .iter()
        .map(|&x| resolve_plot_handle(&Value::Num(x), NAME).ok())
        .collect()
}

/// RunMat's `get(h, name)` succeeds (case-insensitive property lookup).
fn graphics_has(handle: &PlotHandle, name: &str) -> bool {
    !name.is_empty() && get_properties(handle.clone(), Some(name), NAME).is_ok()
}

#[cfg(test)]
mod isprop_tests {
    use crate::builtins::plotting::tests::{ensure_plot_test_env, lock_plot_registry};
    use crate::class_registry::{RuntimeClass, RuntimeProperty};
    use crate::BuiltinResult;
    use futures::executor::block_on;
    use runmat_types::MemberAccess;
    use runmat_value::{
        CellArray, CharArray, MException, ObjectArray, ObjectInstance, StringArray, Tensor, Value,
    };
    use std::collections::HashMap;

    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("isprop", args))
    }

    fn builtin(name: &str, args: &[Value]) -> Value {
        block_on(crate::call_builtin_async(name, args)).expect(name)
    }

    fn chars(s: &str) -> Value {
        Value::CharArray(CharArray::new_row(s))
    }

    fn isprop(obj: Value, name: &str) -> Value {
        call(&[obj, chars(name)]).expect("isprop")
    }

    fn assert_scalar(value: Value, expected: bool) {
        match value {
            Value::Bool(b) => assert_eq!(b, expected),
            Value::LogicalArray(la) => {
                assert_eq!(la.shape, vec![1, 1]);
                assert_eq!(la.data.as_slice(), &[expected as u8]);
            }
            other => panic!("expected a logical scalar, got {other:?}"),
        }
    }

    fn identifier(result: BuiltinResult<Value>) -> String {
        match result {
            Ok(value) => panic!("expected an error, got {value:?}"),
            Err(err) => err.identifier().unwrap_or("<none>").to_string(),
        }
    }

    const UNSUPPORTED: &str = "RunMat:isprop:UnsupportedOption";

    /// Run `body` with an invisible figure that is closed afterwards.
    fn with_figure(body: impl FnOnce(f64)) {
        let _guard = lock_plot_registry();
        ensure_plot_test_env();
        let fig = crate::builtins::plotting::figure::figure_builtin(vec![
            Value::Num(4646.0),
            Value::String("Visible".into()),
            Value::String("off".into()),
        ])
        .expect("figure");
        body(fig);
        let _ = block_on(crate::call_builtin_async("close", &[Value::Num(fig)]));
    }

    fn mexception() -> Value {
        Value::MException(MException::new("a:b".into(), "c".into()))
    }

    // --- Observed (normative): one test per observed case --------------------

    /// case `figure-yes` — isprop.signature-primary, isprop.output-class, isprop.objects
    #[test]
    fn observed_figure_has_visible() {
        with_figure(|fig| assert_scalar(isprop(Value::Num(fig), "Visible"), true));
    }

    /// case `figure-no` — isprop.objects
    #[test]
    fn observed_figure_lacks_unknown() {
        with_figure(|fig| assert_scalar(isprop(Value::Num(fig), "NotAProp"), false));
    }

    /// case `figure-case` — isprop.case-insensitive-graphics
    #[test]
    fn observed_figure_name_case_insensitive() {
        with_figure(|fig| assert_scalar(isprop(Value::Num(fig), "visible"), true));
    }

    /// case `struct-field` — isprop.non-objects
    #[test]
    fn observed_struct_field_is_not_a_property() {
        let s = builtin("struct", &[chars("a"), Value::Num(1.0)]);
        assert_scalar(isprop(s, "a"), false);
    }

    /// case `table-props` — isprop.non-objects
    #[test]
    fn observed_table_properties_is_not_a_property() {
        let t = builtin("table", &[Value::Num(1.0)]);
        assert_scalar(isprop(t, "Properties"), false);
    }

    /// case `table-var` — isprop.non-objects
    #[test]
    fn observed_table_variable_is_not_a_property() {
        let names = Value::Cell(CellArray::new(vec![chars("x")], 1, 1).unwrap());
        let t = builtin("table", &[Value::Num(1.0), chars("VariableNames"), names]);
        assert_scalar(isprop(t, "x"), false);
    }

    /// case `numeric` — isprop.non-objects, isprop.output-class
    #[test]
    fn observed_number_is_false() {
        assert_scalar(isprop(Value::Num(5.0), "a"), false);
    }

    /// case `object-yes` — isprop.objects
    #[test]
    fn observed_mexception_has_message() {
        assert_scalar(isprop(mexception(), "message"), true);
    }

    /// case `object-no` — isprop.objects
    #[test]
    fn observed_mexception_lacks_unknown() {
        assert_scalar(isprop(mexception(), "nope"), false);
    }

    /// case `handle-array` — isprop.arrays
    #[test]
    fn observed_handle_array_gives_logical_row() {
        with_figure(|_fig| {
            let a1 = crate::builtins::plotting::subplot::subplot_builtin(
                Value::Num(1.0),
                Value::Num(2.0),
                Value::Num(1.0),
            )
            .expect("axes 1");
            let a2 = crate::builtins::plotting::subplot::subplot_builtin(
                Value::Num(1.0),
                Value::Num(2.0),
                Value::Num(2.0),
            )
            .expect("axes 2");
            let handles = Value::Tensor(Tensor::new(vec![a1, a2], vec![1, 2]).unwrap());
            match isprop(handles, "Visible") {
                Value::LogicalArray(la) => {
                    assert_eq!(la.shape, vec![1, 2]);
                    assert_eq!(la.data.as_slice(), &[1, 1]);
                }
                other => panic!("expected a 1x2 logical, got {other:?}"),
            }
        });
    }

    /// case `string-name` — isprop.string-name
    #[test]
    fn observed_string_name_accepted() {
        let ex = mexception();
        assert_scalar(call(&[ex, Value::String("message".into())]).unwrap(), true);
    }

    // --- Documented choices (FR-046-07..11) and unresolved points ------------

    fn register_class(name: &str, props: &[(&str, MemberAccess)], parent: Option<&str>) {
        let mut properties = HashMap::new();
        for (prop, access) in props {
            properties.insert(
                (*prop).into(),
                RuntimeProperty {
                    name: (*prop).into(),
                    is_static: false,
                    is_constant: false,
                    is_dependent: false,
                    get_access: *access,
                    set_access: MemberAccess::Public,
                    default_value: None,
                },
            );
        }
        crate::class_registry::register_class(RuntimeClass {
            name: name.into(),
            parent: parent.map(Into::into),
            properties,
            methods: HashMap::new(),
        });
    }

    /// FR-046-07
    #[test]
    fn documented_choice_arity_and_name_forms() {
        assert_eq!(
            identifier(call(&[Value::Num(5.0)])),
            "RunMat:isprop:NotEnoughInputs"
        );
        assert_eq!(
            identifier(call(&[Value::Num(5.0), chars("a"), chars("b")])),
            "RunMat:isprop:TooManyInputs"
        );
        assert_eq!(
            identifier(call(&[mexception(), Value::Num(1.0)])),
            UNSUPPORTED
        );
        let two =
            Value::StringArray(StringArray::new(vec!["a".into(), "b".into()], vec![1, 2]).unwrap());
        assert_eq!(identifier(call(&[mexception(), two])), UNSUPPORTED);
        let one = Value::StringArray(StringArray::new(vec!["message".into()], vec![1, 1]).unwrap());
        assert_scalar(call(&[mexception(), one]).unwrap(), true);
        assert_scalar(isprop(mexception(), ""), false);
    }

    /// FR-046-08 (unresolved, isprop.q-case-objects)
    #[test]
    fn unresolved_case_mismatch_on_objects_is_unsupported() {
        assert_eq!(
            identifier(call(&[mexception(), chars("Message")])),
            UNSUPPORTED
        );
        register_class(
            "runmat.unittest.IspropCase",
            &[("Alpha", MemberAccess::Public)],
            None,
        );
        let obj = Value::Object(ObjectInstance::new("runmat.unittest.IspropCase"));
        assert_scalar(isprop(obj.clone(), "Alpha"), true);
        // Asserted on the builtin itself: after this error the upstream
        // dispatcher retries the duplicate registration, which answers false
        // for a class object (dispatcher quirk; see validation.md).
        let direct = block_on(super::isprop_builtin(vec![obj.clone(), chars("alpha")]));
        assert_eq!(identifier(direct), UNSUPPORTED);
        assert_scalar(isprop(obj, "alpha"), false);
    }

    /// FR-046-09
    #[test]
    fn documented_choice_non_objects_are_false() {
        for value in [
            chars("Visible"),
            Value::String("message".into()),
            Value::Cell(CellArray::new(vec![Value::Num(1.0)], 1, 1).unwrap()),
            Value::Bool(true),
            Value::FunctionHandle("sin".into()),
            Value::Tensor(Tensor::new(Vec::new(), vec![0, 0]).unwrap()),
            Value::Tensor(Tensor::new(vec![-7.0, 1.5], vec![1, 2]).unwrap()),
        ] {
            assert_scalar(isprop(value.clone(), "Visible"), false);
        }
        let tt = block_on(crate::call_builtin_async(
            "timetable",
            &[builtin("seconds", &[Value::Num(1.0)]), Value::Num(2.0)],
        ));
        if let Ok(tt) = tt {
            assert_scalar(isprop(tt, "Properties"), false);
        }
    }

    /// FR-046-10, FR-046-11
    #[test]
    fn documented_choice_class_objects_and_arrays() {
        register_class(
            "runmat.unittest.IspropBase",
            &[
                ("Inherited", MemberAccess::Public),
                ("Hidden", MemberAccess::Private),
            ],
            None,
        );
        register_class(
            "runmat.unittest.IspropChild",
            &[("Own", MemberAccess::Public)],
            Some("runmat.unittest.IspropBase"),
        );
        let obj = Value::Object(ObjectInstance::new("runmat.unittest.IspropChild"));
        assert_scalar(isprop(obj.clone(), "Own"), true);
        assert_scalar(isprop(obj.clone(), "Inherited"), true);
        assert_scalar(isprop(obj.clone(), "Hidden"), false);
        assert_scalar(isprop(obj.clone(), "Missing"), false);
        // Unregistered class gives false.
        let stray = Value::Object(ObjectInstance::new("runmat.unittest.IspropNotRegistered"));
        assert_scalar(isprop(stray, "Own"), false);
        // Object array: same-shape logical.
        let array = Value::ObjectArray(
            ObjectArray::new(
                "runmat.unittest.IspropChild",
                vec![obj.clone(), obj.clone(), obj],
                vec![3, 1],
            )
            .unwrap(),
        );
        match isprop(array, "Own") {
            Value::LogicalArray(la) => {
                assert_eq!(la.shape, vec![3, 1]);
                assert_eq!(la.data.as_slice(), &[1, 1, 1]);
            }
            other => panic!("expected logical array, got {other:?}"),
        }
        // Class reference is unsupported.
        assert_eq!(
            identifier(call(&[
                Value::ClassRef("runmat.unittest.IspropChild".into()),
                chars("Own")
            ])),
            UNSUPPORTED
        );
    }

    /// FR-046-12 (unresolved, isprop.q-warning-coverage)
    #[test]
    fn unresolved_no_warning_emitted() {
        crate::warning_store::reset();
        let _ = isprop(mexception(), "message");
        assert!(crate::warning_store::take_all().is_empty());
    }
}
