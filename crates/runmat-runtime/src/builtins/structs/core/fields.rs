//! MATLAB-compatible `fields` builtin for RunMat.
//!
//! Clean-room provenance: specs/005-struct-cell-access (spec `fields` 0.2.0,
//! claims fields.signature-primary, fields.output-class,
//! fields.cell-fieldnames). The approved summary describes `fields` as a
//! legacy equivalent of `fieldnames`; the implementation delegates to the
//! registered `fieldnames` builtin (same pattern as `bounds` → `min`/`max`).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ShapeRequirements,
};
use crate::builtins::structs::type_resolvers::fieldnames_type;
use crate::BuiltinResult;

#[runmat_macros::register_gpu_spec(builtin_path = "crate::builtins::structs::core::fields")]
pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "fields",
    op_kind: GpuOpKind::Custom("fieldnames"),
    supported_precisions: &[],
    broadcast: BroadcastSemantics::None,
    provider_hooks: &[],
    constant_strategy: ConstantStrategy::InlineLiteral,
    residency: ResidencyPolicy::InheritInputs,
    nan_mode: ReductionNaN::Include,
    two_pass_threshold: None,
    workgroup_size: None,
    accepts_nan_mode: false,
    notes: "Host-only introspection via the fieldnames delegate; providers do not participate.",
};

#[runmat_macros::register_fusion_spec(builtin_path = "crate::builtins::structs::core::fields")]
pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "fields",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes:
        "Fusion planner treats fields as a host inspector; it terminates any pending fusion group.",
};

const FIELDS_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "c",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Cell array of field-name character vectors.",
}];

const FIELDS_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "s",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Structure array.",
}];

const FIELDS_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "c = fields(s)",
    inputs: &FIELDS_INPUTS,
    outputs: &FIELDS_OUTPUT,
}];

const FIELDS_ERROR_INVALID_TARGET: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.FIELDS.INVALID_TARGET",
    identifier: Some("RunMat:fieldnames:InvalidTarget"),
    when: "Input is not a struct, struct array, or object value; raised by the delegated fieldnames builtin (error inputs are unobserved in the approved spec).",
    message: "fieldnames: expected struct, struct array, or object",
};

const FIELDS_ERRORS: [BuiltinErrorDescriptor; 1] = [FIELDS_ERROR_INVALID_TARGET];

pub const FIELDS_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &FIELDS_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &FIELDS_ERRORS,
};

#[runtime_builtin(
    name = "fields",
    category = "structs/core",
    summary = "Return struct field names as a cell array of character vectors (legacy equivalent of fieldnames).",
    keywords = "fields,fieldnames,struct,introspection",
    type_resolver(fieldnames_type),
    descriptor(crate::builtins::structs::core::fields::FIELDS_DESCRIPTOR),
    builtin_path = "crate::builtins::structs::core::fields"
)]
async fn fields_builtin(value: Value) -> BuiltinResult<Value> {
    crate::call_builtin_async("fieldnames", &[value]).await
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use futures::executor::block_on;
    use runmat_builtins::{CellArray, StructValue};

    fn run_fields(value: Value) -> BuiltinResult<Value> {
        block_on(super::fields_builtin(value))
    }

    fn cell_strings(cell: &CellArray) -> Vec<String> {
        cell.data
            .iter()
            .map(|v| match v {
                Value::CharArray(ca) => ca.data.iter().collect(),
                other => panic!("expected character array cell element, got {other:?}"),
            })
            .collect()
    }

    // Normative [FR-005-01; fields.signature-primary, fields.output-class,
    // fields.cell-fieldnames] — observed case `two-fields`:
    // fields(struct('a',1,'b',2)) => 2x1 cell of field-name char vectors.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn observed_two_field_struct_yields_2x1_cell() {
        let mut st = StructValue::new();
        st.insert("a", Value::Num(1.0));
        st.insert("b", Value::Num(2.0));
        let Value::Cell(cell) = run_fields(Value::Struct(st)).expect("fields") else {
            panic!("expected cell array result");
        };
        assert_eq!(cell.rows, 2);
        assert_eq!(cell.cols, 1);
        assert_eq!(cell_strings(&cell), vec!["a".to_string(), "b".to_string()]);
    }

    // Normative [FR-005-01; fields.output-class, fields.cell-fieldnames] —
    // observed case `empty-struct`: fields(struct()) => 0x1 cell.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn observed_empty_struct_yields_0x1_cell() {
        let Value::Cell(cell) = run_fields(Value::Struct(StructValue::new())).expect("fields")
        else {
            panic!("expected cell array result");
        };
        assert_eq!(cell.rows, 0);
        assert_eq!(cell.cols, 1);
        assert!(cell.data.is_empty());
    }

    // unresolved_choice [FR-005-03; fields.q-alias]: non-struct inputs are
    // unobserved in the approved spec; the fieldnames delegate rejects them.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_non_struct_input_errors() {
        let err = run_fields(Value::Num(5.0)).unwrap_err();
        assert!(
            err.to_string()
                .contains("expected struct, struct array, or object"),
            "unexpected error message: {err}"
        );
    }

    // unresolved_choice [FR-005-03; fields.q-alias]: struct arrays are
    // unobserved for fields; the delegate returns the sorted union of field
    // names, identical to fieldnames.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unresolved_choice_struct_array_matches_fieldnames() {
        let mut first = StructValue::new();
        first.insert("name", Value::Num(1.0));
        let mut second = StructValue::new();
        second.insert("id", Value::Num(2.0));
        let array =
            CellArray::new(vec![Value::Struct(first), Value::Struct(second)], 1, 2).unwrap();
        let Value::Cell(cell) = run_fields(Value::Cell(array)).expect("fields") else {
            panic!("expected cell array result");
        };
        assert_eq!(cell.rows, 2);
        assert_eq!(cell.cols, 1);
        assert_eq!(
            cell_strings(&cell),
            vec!["id".to_string(), "name".to_string()]
        );
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn descriptor_signature_covers_fields_form() {
        let labels: Vec<&str> = FIELDS_DESCRIPTOR
            .signatures
            .iter()
            .map(|sig| sig.label)
            .collect();
        assert_eq!(labels, vec!["c = fields(s)"]);
    }
}
