//! MATLAB-compatible `isdir` builtin for RunMat (legacy alias of `isfolder`).
//!
//! Clean-room provenance: specs/022-legacy-predicates (spec `isdir` 0.2.0,
//! claims isdir.signature-primary, isdir.output-class,
//! isdir.logical-folder-test). `isdir` is the legacy equivalent of `isfolder`;
//! it delegates to the shared `isfolder::folder_predicate` so both builtins
//! share a single source of truth (existing folder => true, missing => false).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    ResolveContext, Type, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ShapeRequirements,
};
use crate::BuiltinResult;

#[runmat_macros::register_gpu_spec(builtin_path = "crate::builtins::io::repl_fs::isdir")]
pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "isdir",
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

#[runmat_macros::register_fusion_spec(builtin_path = "crate::builtins::io::repl_fs::isdir")]
pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "isdir",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Not fusible; performs filesystem I/O.",
};

const ISDIR_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "tf",
    ty: BuiltinParamType::LogicalArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "True when the path refers to an existing folder.",
}];

const ISDIR_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "path",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Path text to test.",
}];

const ISDIR_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "tf = isdir(path)",
    inputs: &ISDIR_INPUTS,
    outputs: &ISDIR_OUTPUT,
}];

const ISDIR_ERRORS: [BuiltinErrorDescriptor; 0] = [];

pub const ISDIR_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &ISDIR_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ISDIR_ERRORS,
};

#[runtime_builtin(
    name = "isdir",
    category = "io/repl_fs",
    summary = "Return true when a path refers to an existing folder (legacy equivalent of isfolder).",
    keywords = "isdir,dir,folder,directory,exists,filesystem,predicate",
    type_resolver(bool_scalar_type),
    descriptor(crate::builtins::io::repl_fs::isdir::ISDIR_DESCRIPTOR),
    builtin_path = "crate::builtins::io::repl_fs::isdir"
)]
async fn isdir_builtin(path: Value) -> BuiltinResult<Value> {
    // Legacy alias: identical behaviour to isfolder via the shared helper.
    crate::builtins::parallel_implementations::isfolder::folder_predicate(path).await
}

fn bool_scalar_type(_: &[Type], _context: &ResolveContext) -> Type {
    Type::Bool
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use futures::executor::block_on;
    use runmat_builtins::CharArray;
    use std::fs;

    fn run_isdir(value: Value) -> Value {
        block_on(super::isdir_builtin(value)).expect("isdir")
    }

    fn char_value(text: &str) -> Value {
        Value::CharArray(CharArray::new_row(text))
    }

    // Normative [FR-022-04; isdir.logical-folder-test, isdir.output-class]
    // Observed case `existing`: isdir(tempdir) => true.
    #[test]
    fn observed_existing_folder_is_true() {
        let dir = std::env::temp_dir();
        assert_eq!(
            run_isdir(char_value(dir.to_str().unwrap())),
            Value::Bool(true)
        );
    }

    // Normative [FR-022-04; isdir.logical-folder-test, isdir.output-class]
    // Observed case `missing`: isdir([tempname '.nope']) => false.
    #[test]
    fn observed_missing_path_is_false() {
        let path = std::env::temp_dir().join("runmat_isdir_observed_missing.nope");
        let _ = fs::remove_dir_all(&path);
        let _ = fs::remove_file(&path);
        assert_eq!(
            run_isdir(char_value(path.to_str().unwrap())),
            Value::Bool(false)
        );
    }

    // unresolved_choice [FR-022-05; isdir.q-deprecated]: legacy alias mirrors
    // isfolder — a regular file reports false (folder != file).
    #[test]
    fn unresolved_choice_file_is_false() {
        let path = std::env::temp_dir().join("runmat_isdir_unresolved_file.txt");
        fs::write(&path, b"runmat").expect("create temp file");
        let result = run_isdir(char_value(path.to_str().unwrap()));
        let _ = fs::remove_file(&path);
        assert_eq!(result, Value::Bool(false));
    }

    // unresolved_choice [FR-022-05]: non-text input reports false.
    #[test]
    fn unresolved_choice_non_text_input_is_false() {
        assert_eq!(run_isdir(Value::Num(5.0)), Value::Bool(false));
    }
}
