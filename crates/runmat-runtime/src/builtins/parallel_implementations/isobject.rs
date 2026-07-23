// Parallel clean-room implementation of `isobject` (independently developed against the
// matlab-interface-spec black-box specs). 0-6-0 ships the DEFAULT implementation; this copy is
// retained unregistered for differential cross-validation. Original path: crates/runmat-runtime/src/builtins/introspection/isobject.rs
//! MATLAB-compatible `isobject` builtin with GPU-aware semantics for RunMat.
//!
//! Clean-room provenance: specs/022-legacy-predicates (spec `isobject` 0.2.0,
//! claims isobject.signature-primary, isobject.output-class,
//! isobject.logical-object-test). Object/handle-object value kinds report
//! true; all fundamental (numeric/char/cell/…) kinds report false.

use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ShapeRequirements,
};
use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    ResolveContext, Type, Value,
};
use runmat_macros::runtime_builtin;

use crate::BuiltinResult;

pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "isobject",
    op_kind: GpuOpKind::Custom("metadata"),
    supported_precisions: &[],
    broadcast: BroadcastSemantics::None,
    provider_hooks: &[],
    constant_strategy: ConstantStrategy::InlineLiteral,
    residency: ResidencyPolicy::InheritInputs,
    nan_mode: ReductionNaN::Include,
    two_pass_threshold: None,
    workgroup_size: None,
    accepts_nan_mode: false,
    notes:
        "Runs entirely on the host and inspects value kind; gpuArray inputs return logical false.",
};

pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "isobject",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Metadata-only predicate that does not participate in fusion planning.",
};

const ISOBJECT_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "tf",
    ty: BuiltinParamType::LogicalArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "True when the input is an object instance.",
}];

const ISOBJECT_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "A",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Value to test.",
}];

const ISOBJECT_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "tf = isobject(A)",
    inputs: &ISOBJECT_INPUTS,
    outputs: &ISOBJECT_OUTPUT,
}];

const ISOBJECT_ERRORS: [BuiltinErrorDescriptor; 0] = [];

pub const ISOBJECT_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &ISOBJECT_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ISOBJECT_ERRORS,
};

fn isobject_builtin(value: Value) -> BuiltinResult<Value> {
    Ok(Value::Bool(isobject_value(&value)))
}

fn bool_scalar_type(_: &[Type], _context: &ResolveContext) -> Type {
    Type::Bool
}

/// True for object-instance value kinds. `Value::Object` (value-class
/// instance) and `Value::HandleObject` (handle-class instance, e.g. the
/// feature-015 `inputParser`) are objects; every fundamental kind
/// (numeric/char/cell/struct/string/logical/function-handle/GPU) is not.
///
/// Other object-adjacent kinds (`Value::Listener`, `Value::MException`,
/// `Value::ClassRef`) are unobserved in the approved export; they
/// conservatively report `false` as a documented independent choice rather
/// than an invented MATLAB-conformance claim (see isobject.q-fundamental).
fn isobject_value(value: &Value) -> bool {
    matches!(value, Value::Object(_) | Value::HandleObject(_))
}

#[cfg(all(test, feature = "parallel_xval"))]
pub(crate) mod tests {
    use super::*;
    use crate::builtins::common::test_support;
    #[cfg(feature = "wgpu")]
    use runmat_accelerate::backend::wgpu::provider::{register_wgpu_provider, WgpuProviderOptions};
    use runmat_accelerate_api::HostTensorView;
    use runmat_builtins::{CellArray, CharArray, HandleRef, ObjectInstance, StructValue, Tensor};

    fn handle_object(class_name: &str) -> Value {
        let target = runmat_gc::gc_allocate(Value::Num(0.0)).expect("gc allocation");
        Value::HandleObject(HandleRef {
            class_name: class_name.to_string(),
            target,
            valid: true,
        })
    }

    // Normative [FR-022-01; isobject.logical-object-test, isobject.output-class]
    // Observed case `object`: isobject(inputParser) => true. RunMat's
    // inputParser (feature 015) is a Value::HandleObject.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn observed_object_instance_is_true() {
        let result = isobject_builtin(handle_object("inputParser")).expect("isobject");
        assert_eq!(result, Value::Bool(true));
    }

    // Normative — observed case `numeric`: isobject(5) => false.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn observed_numeric_is_false() {
        assert_eq!(
            isobject_builtin(Value::Num(5.0)).expect("isobject"),
            Value::Bool(false)
        );
    }

    // Normative — observed case `char`: isobject('a') => false.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn observed_char_is_false() {
        assert_eq!(
            isobject_builtin(Value::CharArray(CharArray::new_row("a"))).expect("isobject"),
            Value::Bool(false)
        );
    }

    // Normative — observed case `cell`: isobject({1}) => false.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn observed_cell_is_false() {
        let cell = CellArray::new(vec![Value::Num(1.0)], 1, 1).expect("cell");
        assert_eq!(
            isobject_builtin(Value::Cell(cell)).expect("isobject"),
            Value::Bool(false)
        );
    }

    // unresolved_choice [isobject.q-fundamental]: a value-class instance
    // (Value::Object) is an object kind too; reports true. Unobserved input
    // class in the approved export.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_value_object_is_true() {
        let obj = Value::Object(ObjectInstance::new("Point".to_string()));
        assert_eq!(isobject_builtin(obj).expect("isobject"), Value::Bool(true));
    }

    // unresolved_choice [isobject.q-fundamental]: struct, string and
    // function-handle fundamental kinds report false (unobserved).
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_fundamental_kinds_are_false() {
        let mut st = StructValue::new();
        st.insert("field", Value::Num(1.0));
        assert_eq!(
            isobject_builtin(Value::Struct(st)).expect("isobject"),
            Value::Bool(false)
        );
        assert_eq!(
            isobject_builtin(Value::String("hi".to_string())).expect("isobject"),
            Value::Bool(false)
        );
        assert_eq!(
            isobject_builtin(Value::FunctionHandle("sin".to_string())).expect("isobject"),
            Value::Bool(false)
        );
    }

    // unresolved_choice: GPU tensors are numeric, not objects → false.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_gpu_tensor_is_false() {
        test_support::with_test_provider(|provider| {
            let tensor = Tensor::new(vec![1.0, 2.0], vec![2, 1]).expect("tensor");
            let view = HostTensorView {
                data: &tensor.data,
                shape: &tensor.shape,
            };
            let handle = provider.upload(&view).expect("upload");
            let result = isobject_builtin(Value::GpuTensor(handle)).expect("isobject");
            assert_eq!(result, Value::Bool(false));
        });
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    #[cfg(feature = "wgpu")]
    fn isobject_wgpu_numeric_returns_false() {
        let _ = register_wgpu_provider(WgpuProviderOptions::default());
        let provider = runmat_accelerate_api::provider().expect("wgpu provider");
        let data = vec![0.0, 1.0];
        let shape = vec![2, 1];
        let view = HostTensorView {
            data: &data,
            shape: &shape,
        };
        let handle = provider.upload(&view).expect("upload to GPU");
        let result = isobject_builtin(Value::GpuTensor(handle)).expect("isobject");
        assert_eq!(result, Value::Bool(false));
    }
}
