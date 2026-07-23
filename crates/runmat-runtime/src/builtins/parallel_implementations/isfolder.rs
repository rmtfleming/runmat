// Parallel clean-room implementation of `isfolder` (independently developed against the
// matlab-interface-spec black-box specs). 0-6-0 ships the DEFAULT implementation; this copy is
// retained unregistered for differential cross-validation. Original path: crates/runmat-runtime/src/builtins/io/repl_fs/isfolder.rs
//! MATLAB-compatible `isfolder` builtin for RunMat.
//!
//! Clean-room provenance: specs/022-legacy-predicates (spec `isfolder` 0.2.0,
//! claims isfolder.signature-primary, isfolder.output-class,
//! isfolder.logical-folder-test). Mirrors `isfile` but tests for a directory
//! via `runmat_filesystem::metadata_async(...).is_dir()`.

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    ResolveContext, StringArray, Type, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ShapeRequirements,
};
use crate::BuiltinResult;

pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "isfolder",
    op_kind: GpuOpKind::Custom("io"),
    supported_precisions: &[],
    broadcast: BroadcastSemantics::None,
    provider_hooks: &[],
    constant_strategy: ConstantStrategy::InlineLiteral,
    residency: ResidencyPolicy::GatherImmediately,
    nan_mode: ReductionNaN::Include,
    two_pass_threshold: None,
    workgroup_size: None,
    accepts_nan_mode: false,
    notes: "Filesystem predicate; no GPU execution path.",
};

pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "isfolder",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Not fusible; performs filesystem I/O.",
};

const ISFOLDER_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "tf",
    ty: BuiltinParamType::LogicalArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "True when the path refers to an existing folder.",
}];

const ISFOLDER_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "path",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Path text to test.",
}];

const ISFOLDER_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "tf = isfolder(path)",
    inputs: &ISFOLDER_INPUTS,
    outputs: &ISFOLDER_OUTPUT,
}];

const ISFOLDER_ERRORS: [BuiltinErrorDescriptor; 0] = [];

pub const ISFOLDER_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &ISFOLDER_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ISFOLDER_ERRORS,
};

async fn isfolder_builtin(path: Value) -> BuiltinResult<Value> {
    folder_predicate(path).await
}

/// Shared folder-existence predicate. `isfolder` and its legacy alias `isdir`
/// both delegate here so there is a single source of truth for the behaviour.
pub(crate) async fn folder_predicate(path: Value) -> BuiltinResult<Value> {
    let Some(text) = path_text(&path) else {
        // Non-text inputs are unobserved in the approved spec; a predicate
        // answer (false) is the documented independent choice.
        return Ok(Value::Bool(false));
    };
    if text.is_empty() {
        return Ok(Value::Bool(false));
    }
    let is_dir = match runmat_filesystem::metadata_async(&text).await {
        Ok(meta) => meta.is_dir(),
        Err(_) => false,
    };
    Ok(Value::Bool(is_dir))
}

fn bool_scalar_type(_: &[Type], _context: &ResolveContext) -> Type {
    Type::Bool
}

fn path_text(value: &Value) -> Option<String> {
    match value {
        Value::CharArray(chars) if chars.rows <= 1 => Some(chars.data.iter().collect()),
        Value::String(text) => Some(text.clone()),
        Value::StringArray(StringArray { data, .. }) if data.len() == 1 => Some(data[0].clone()),
        _ => None,
    }
}

#[cfg(all(test, feature = "parallel_xval"))]
pub(crate) mod tests {
    use super::*;
    use futures::executor::block_on;
    use runmat_builtins::CharArray;
    use std::fs;

    fn run_isfolder(value: Value) -> Value {
        block_on(super::isfolder_builtin(value)).expect("isfolder")
    }

    fn char_value(text: &str) -> Value {
        Value::CharArray(CharArray::new_row(text))
    }

    // Normative [FR-022-03; isfolder.logical-folder-test, isfolder.output-class]
    // Observed case `existing`: isfolder(tempdir) => true.
    #[test]
    fn observed_existing_folder_is_true() {
        let dir = std::env::temp_dir();
        assert_eq!(
            run_isfolder(char_value(dir.to_str().unwrap())),
            Value::Bool(true)
        );
    }

    // Normative [FR-022-03; isfolder.logical-folder-test, isfolder.output-class]
    // Observed case `missing`: isfolder([tempname '.nope']) => false.
    #[test]
    fn observed_missing_path_is_false() {
        let path = std::env::temp_dir().join("runmat_isfolder_observed_missing.nope");
        let _ = fs::remove_dir_all(&path);
        let _ = fs::remove_file(&path);
        assert_eq!(
            run_isfolder(char_value(path.to_str().unwrap())),
            Value::Bool(false)
        );
    }

    // unresolved_choice [FR-022-05; isfolder.q-file]: a regular file reports
    // false (folder != file), pending spec-side confirmation.
    #[test]
    fn unresolved_choice_file_is_false() {
        let path = std::env::temp_dir().join("runmat_isfolder_unresolved_file.txt");
        fs::write(&path, b"runmat").expect("create temp file");
        let result = run_isfolder(char_value(path.to_str().unwrap()));
        let _ = fs::remove_file(&path);
        assert_eq!(result, Value::Bool(false));
    }

    // unresolved_choice [FR-022-05]: non-text input reports false.
    #[test]
    fn unresolved_choice_non_text_input_is_false() {
        assert_eq!(run_isfolder(Value::Num(5.0)), Value::Bool(false));
    }
}
