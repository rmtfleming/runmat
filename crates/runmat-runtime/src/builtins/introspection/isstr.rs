//! MATLAB-compatible `isstr` builtin with GPU-aware semantics for RunMat.
//!
//! Clean-room provenance: specs/022-legacy-predicates (spec `isstr` 0.2.0,
//! claims isstr.signature-primary, isstr.output-class, isstr.logical-char-test).
//! `isstr` is the legacy equivalent of `ischar`: it mirrors `ischar`'s
//! semantics exactly — a character array reports true; everything else,
//! including a double-quoted string scalar, reports false.

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

#[runmat_macros::register_gpu_spec(builtin_path = "crate::builtins::introspection::isstr")]
pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "isstr",
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
    notes: "Runs entirely on the host and inspects value metadata; gpuArray inputs return logical false.",
};

#[runmat_macros::register_fusion_spec(builtin_path = "crate::builtins::introspection::isstr")]
pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "isstr",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Metadata-only predicate that does not participate in fusion planning.",
};

const ISSTR_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "tf",
    ty: BuiltinParamType::LogicalArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "True when input is a character array.",
}];

const ISSTR_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "A",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Value to test.",
}];

const ISSTR_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "tf = isstr(A)",
    inputs: &ISSTR_INPUTS,
    outputs: &ISSTR_OUTPUT,
}];

const ISSTR_ERRORS: [BuiltinErrorDescriptor; 0] = [];

pub const ISSTR_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &ISSTR_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ISSTR_ERRORS,
};

#[runtime_builtin(
    name = "isstr",
    category = "introspection",
    summary = "Return true when a value is a character array (legacy equivalent of ischar).",
    keywords = "isstr,char array,ischar,type checking,introspection",
    accel = "metadata",
    type_resolver(bool_scalar_type),
    descriptor(crate::builtins::introspection::isstr::ISSTR_DESCRIPTOR),
    builtin_path = "crate::builtins::introspection::isstr"
)]
fn isstr_builtin(value: Value) -> BuiltinResult<Value> {
    // Legacy alias of ischar: char array => true, everything else (including
    // string scalars/arrays) => false.
    Ok(Value::Bool(matches!(value, Value::CharArray(_))))
}

fn bool_scalar_type(_: &[Type], _context: &ResolveContext) -> Type {
    Type::Bool
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::builtins::common::test_support;
    #[cfg(feature = "wgpu")]
    use runmat_accelerate::backend::wgpu::provider::{register_wgpu_provider, WgpuProviderOptions};
    use runmat_accelerate_api::HostTensorView;
    use runmat_builtins::{CellArray, CharArray, LogicalArray, StringArray, Tensor};

    // Normative [FR-022-02; isstr.logical-char-test, isstr.output-class]
    // Observed case `char`: isstr('abc') => true.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn observed_char_is_true() {
        let result = isstr_builtin(Value::CharArray(CharArray::new_row("abc"))).expect("isstr");
        assert_eq!(result, Value::Bool(true));
    }

    // Normative — observed case `numeric`: isstr(5) => false.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn observed_numeric_is_false() {
        assert_eq!(
            isstr_builtin(Value::Num(5.0)).expect("isstr"),
            Value::Bool(false)
        );
    }

    // Normative — observed case `string`: isstr("hi") => false. A
    // double-quoted string scalar is NOT treated as char (matches ischar).
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn observed_string_is_false() {
        assert_eq!(
            isstr_builtin(Value::String("hi".to_string())).expect("isstr"),
            Value::Bool(false)
        );
        let array = Value::StringArray(
            StringArray::new(vec!["a".to_string(), "b".to_string()], vec![1, 2]).expect("strings"),
        );
        assert_eq!(isstr_builtin(array).expect("isstr"), Value::Bool(false));
    }

    // Normative — observed case `cell`: isstr({'a'}) => false.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn observed_cell_is_false() {
        let cell =
            CellArray::new(vec![Value::CharArray(CharArray::new_row("a"))], 1, 1).expect("cell");
        assert_eq!(
            isstr_builtin(Value::Cell(cell)).expect("isstr"),
            Value::Bool(false)
        );
    }

    // unresolved_choice: 2-D char matrix and empty char array report true,
    // mirroring ischar (unobserved shapes in the approved export).
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_char_matrix_and_empty_are_true() {
        let matrix = CharArray::new(vec!['a', 'b', 'c', 'd', 'e', 'f'], 2, 3).expect("char matrix");
        assert_eq!(
            isstr_builtin(Value::CharArray(matrix)).expect("isstr"),
            Value::Bool(true)
        );
        let empty = CharArray::new(Vec::new(), 0, 0).expect("empty char array");
        assert_eq!(
            isstr_builtin(Value::CharArray(empty)).expect("isstr"),
            Value::Bool(true)
        );
    }

    // unresolved_choice: logical values report false (mirrors ischar).
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_logical_is_false() {
        assert_eq!(
            isstr_builtin(Value::Bool(true)).expect("isstr"),
            Value::Bool(false)
        );
        let logical_array =
            Value::LogicalArray(LogicalArray::new(vec![1u8], vec![1, 1]).expect("logical array"));
        assert_eq!(
            isstr_builtin(logical_array).expect("isstr"),
            Value::Bool(false)
        );
    }

    // unresolved_choice: GPU tensors are numeric, not char → false.
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
            let result = isstr_builtin(Value::GpuTensor(handle)).expect("isstr");
            assert_eq!(result, Value::Bool(false));
        });
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    #[cfg(feature = "wgpu")]
    fn isstr_wgpu_numeric_returns_false() {
        let _ = register_wgpu_provider(WgpuProviderOptions::default());
        let provider = runmat_accelerate_api::provider().expect("wgpu provider");
        let data = vec![0.0, 1.0];
        let shape = vec![2, 1];
        let view = HostTensorView {
            data: &data,
            shape: &shape,
        };
        let handle = provider.upload(&view).expect("upload to GPU");
        let result = isstr_builtin(Value::GpuTensor(handle)).expect("isstr");
        assert_eq!(result, Value::Bool(false));
    }
}
