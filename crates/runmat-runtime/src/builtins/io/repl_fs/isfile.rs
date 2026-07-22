//! MATLAB-compatible `isfile` builtin for RunMat.
//!
//! Clean-room provenance: specs/002-type-predicates (spec `isfile` 0.2.0,
//! claims isfile.signature-primary, isfile.output-class,
//! isfile.missing-false; existing-file behaviour is summary-derived).

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

#[runmat_macros::register_gpu_spec(builtin_path = "crate::builtins::io::repl_fs::isfile")]
pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "isfile",
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

#[runmat_macros::register_fusion_spec(builtin_path = "crate::builtins::io::repl_fs::isfile")]
pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "isfile",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Not fusible; performs filesystem I/O.",
};

const ISFILE_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "tf",
    ty: BuiltinParamType::LogicalArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "True when the path refers to an existing file.",
}];

const ISFILE_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "path",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Path text to test.",
}];

const ISFILE_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "tf = isfile(path)",
    inputs: &ISFILE_INPUTS,
    outputs: &ISFILE_OUTPUT,
}];

const ISFILE_ERRORS: [BuiltinErrorDescriptor; 0] = [];

pub const ISFILE_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &ISFILE_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ISFILE_ERRORS,
};

#[runtime_builtin(
    name = "isfile",
    category = "io/repl_fs",
    summary = "Return true when a path refers to an existing file.",
    keywords = "isfile,file,exists,filesystem,predicate",
    type_resolver(bool_scalar_type),
    descriptor(crate::builtins::io::repl_fs::isfile::ISFILE_DESCRIPTOR),
    builtin_path = "crate::builtins::io::repl_fs::isfile"
)]
async fn isfile_builtin(path: Value) -> BuiltinResult<Value> {
    let Some(text) = path_text(&path) else {
        // Non-text inputs are unobserved in the approved spec; a predicate
        // answer (false) is the documented independent choice.
        return Ok(Value::Bool(false));
    };
    if text.is_empty() {
        return Ok(Value::Bool(false));
    }
    let is_file = match runmat_filesystem::metadata_async(&text).await {
        Ok(meta) => meta.is_file(),
        Err(_) => false,
    };
    Ok(Value::Bool(is_file))
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

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use futures::executor::block_on;
    use runmat_builtins::CharArray;
    use std::fs;

    fn run_isfile(value: Value) -> Value {
        block_on(super::isfile_builtin(value)).expect("isfile")
    }

    fn char_value(text: &str) -> Value {
        Value::CharArray(CharArray::new_row(text))
    }

    // Normative [FR-002-03; isfile.missing-false, isfile.output-class] —
    // observed case `missing`: isfile([tempname '.nope']) => false.
    #[test]
    fn observed_missing_path_is_false() {
        let path = std::env::temp_dir().join("runmat_isfile_observed_missing.nope");
        let _ = fs::remove_file(&path);
        assert_eq!(
            run_isfile(char_value(path.to_str().unwrap())),
            Value::Bool(false)
        );
    }

    // summary_derived [FR-002-04]: existing regular file reports true per the
    // approved summary; no observation case backs this yet
    // (isfile.q-existing).
    #[test]
    fn summary_derived_existing_file_is_true() {
        let path = std::env::temp_dir().join("runmat_isfile_summary_derived_existing.txt");
        fs::write(&path, b"runmat").expect("create temp file");
        let result = run_isfile(char_value(path.to_str().unwrap()));
        let _ = fs::remove_file(&path);
        assert_eq!(result, Value::Bool(true));
    }

    // unresolved_choice [FR-002-05; isfile.q-folder]: folders report false.
    #[test]
    fn unresolved_choice_folder_is_false() {
        let dir = std::env::temp_dir();
        assert_eq!(
            run_isfile(char_value(dir.to_str().unwrap())),
            Value::Bool(false)
        );
    }

    // unresolved_choice [FR-002-05]: non-text input reports false.
    #[test]
    fn unresolved_choice_non_text_input_is_false() {
        assert_eq!(run_isfile(Value::Num(5.0)), Value::Bool(false));
    }
}
