// Parallel clean-room implementation of `iscellstr` (independently developed against the
// matlab-interface-spec black-box specs). 0-6-0 ships the DEFAULT implementation; this copy is
// retained unregistered for differential cross-validation. Original path: crates/runmat-runtime/src/builtins/logical/tests/iscellstr.rs
//! MATLAB-compatible `iscellstr` builtin for RunMat.
//!
//! Clean-room provenance: specs/016-shape-storage-predicates (spec
//! `iscellstr` 0.2.0, claims iscellstr.signature-primary,
//! iscellstr.output-class, iscellstr.logical-cellstr-test).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    ResolveContext, Type, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ScalarType, ShapeRequirements,
};
use crate::BuiltinResult;

pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "iscellstr",
    op_kind: GpuOpKind::Custom("metadata"),
    supported_precisions: &[ScalarType::F32, ScalarType::F64],
    broadcast: BroadcastSemantics::None,
    provider_hooks: &[],
    constant_strategy: ConstantStrategy::InlineLiteral,
    residency: ResidencyPolicy::GatherImmediately,
    nan_mode: ReductionNaN::Include,
    two_pass_threshold: None,
    workgroup_size: None,
    accepts_nan_mode: false,
    notes:
        "GPU tensors are never cell arrays; the predicate answers from value kind without gathering.",
};

pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "iscellstr",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Type check executed outside fusion; planners treat it as a scalar metadata query.",
};

const ISCELLSTR_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "tf",
    ty: BuiltinParamType::LogicalArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "True when the input is a cell array of character vectors.",
}];

const ISCELLSTR_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "A",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Value to test.",
}];

const ISCELLSTR_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "tf = iscellstr(A)",
    inputs: &ISCELLSTR_INPUTS,
    outputs: &ISCELLSTR_OUTPUT,
}];

const ISCELLSTR_ERRORS: [BuiltinErrorDescriptor; 0] = [];

pub const ISCELLSTR_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &ISCELLSTR_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ISCELLSTR_ERRORS,
};

async fn iscellstr_builtin(value: Value) -> BuiltinResult<Value> {
    Ok(Value::Bool(iscellstr_value(&value)))
}

fn bool_scalar_type(_: &[Type], _context: &ResolveContext) -> Type {
    Type::Bool
}

/// True when the input is a cell whose every element is char-like; the
/// empty cell `{}` vacuously satisfies the rule (observed). The element
/// test is kind-level: `Value::CharArray` (normative, matching `ischar`)
/// plus `Value::String` — RunMat's internal string-scalar text
/// representation — accepted by documented independent choice
/// (iscellstr.q-strings carried unresolved). Char-matrix elements pass the
/// kind-level test (unobserved input class, documented choice).
fn iscellstr_value(value: &Value) -> bool {
    match value {
        Value::Cell(cell) => cell
            .data
            .iter()
            .all(|element| matches!(element, Value::CharArray(_) | Value::String(_))),
        _ => false,
    }
}

#[cfg(all(test, feature = "parallel_xval"))]
pub(crate) mod tests {
    use super::*;
    use futures::executor::block_on;
    use runmat_builtins::{CellArray, CharArray, StringArray};

    fn run_iscellstr(value: Value) -> Value {
        block_on(super::iscellstr_builtin(value)).expect("iscellstr")
    }

    // Normative [FR-016-04; iscellstr.logical-cellstr-test,
    // iscellstr.output-class] — observed case `all-char`:
    // iscellstr({'a', 'bc'}) => true.
    #[test]
    fn observed_cell_of_char_vectors_is_true() {
        let cell = CellArray::new(
            vec![
                Value::CharArray(CharArray::new_row("a")),
                Value::CharArray(CharArray::new_row("bc")),
            ],
            1,
            2,
        )
        .unwrap();
        assert_eq!(run_iscellstr(Value::Cell(cell)), Value::Bool(true));
    }

    // Normative — observed case `mixed`: iscellstr({'a', 1}) => false.
    #[test]
    fn observed_mixed_cell_is_false() {
        let cell = CellArray::new(
            vec![Value::CharArray(CharArray::new_row("a")), Value::Num(1.0)],
            1,
            2,
        )
        .unwrap();
        assert_eq!(run_iscellstr(Value::Cell(cell)), Value::Bool(false));
    }

    // Normative — observed case `char`: iscellstr('abc') => false.
    #[test]
    fn observed_plain_char_vector_is_false() {
        assert_eq!(
            run_iscellstr(Value::CharArray(CharArray::new_row("abc"))),
            Value::Bool(false)
        );
    }

    // Normative — observed case `empty-cell`: iscellstr({}) => true.
    #[test]
    fn observed_empty_cell_is_true() {
        let cell = CellArray::new(vec![], 0, 0).unwrap();
        assert_eq!(run_iscellstr(Value::Cell(cell)), Value::Bool(true));
    }

    // unresolved_choice: string elements unobserved (iscellstr.q-strings);
    // RunMat's internal string-scalar text representation counts as text.
    #[test]
    fn unresolved_choice_string_scalar_elements_count_as_text() {
        let cell = CellArray::new(
            vec![
                Value::String("a".to_string()),
                Value::CharArray(CharArray::new_row("bc")),
            ],
            1,
            2,
        )
        .unwrap();
        assert_eq!(run_iscellstr(Value::Cell(cell)), Value::Bool(true));
    }

    // unresolved_choice: a string array input is not a cell and reports
    // false (unobserved input class).
    #[test]
    fn unresolved_choice_string_array_input_is_false() {
        let strings = StringArray::new(vec!["a".to_string(), "b".to_string()], vec![1, 2]).unwrap();
        assert_eq!(
            run_iscellstr(Value::StringArray(strings)),
            Value::Bool(false)
        );
    }

    // unresolved_choice: char-matrix elements unobserved; the element test
    // is kind-level (char-ness, mirroring ischar), so they report true.
    #[test]
    fn unresolved_choice_char_matrix_element_follows_kind() {
        let matrix = CharArray::new(vec!['a', 'b', 'c', 'd'], 2, 2).unwrap();
        let cell = CellArray::new(vec![Value::CharArray(matrix)], 1, 1).unwrap();
        assert_eq!(run_iscellstr(Value::Cell(cell)), Value::Bool(true));
    }

    // unresolved_choice: non-char kinds inside a cell (nested cell, bool)
    // report false, generalising the observed mixed case.
    #[test]
    fn unresolved_choice_non_char_kinds_inside_cell_are_false() {
        let nested = CellArray::new(vec![Value::CharArray(CharArray::new_row("a"))], 1, 1).unwrap();
        let cell = CellArray::new(
            vec![
                Value::Cell(nested),
                Value::CharArray(CharArray::new_row("b")),
            ],
            1,
            2,
        )
        .unwrap();
        assert_eq!(run_iscellstr(Value::Cell(cell)), Value::Bool(false));
        let bool_cell = CellArray::new(vec![Value::Bool(true)], 1, 1).unwrap();
        assert_eq!(run_iscellstr(Value::Cell(bool_cell)), Value::Bool(false));
    }
}
