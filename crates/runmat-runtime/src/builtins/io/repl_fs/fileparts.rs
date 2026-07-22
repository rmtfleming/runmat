//! MATLAB-compatible `fileparts` builtin for RunMat.
//!
//! Clean-room provenance: specs/003-path-parts (spec `fileparts` 0.2.0,
//! claims fileparts.signature-primary, fileparts.output-class,
//! fileparts.first-output-dir; name/ext outputs are summary-derived).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    CharArray, StringArray, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ShapeRequirements,
};
use crate::{build_runtime_error, BuiltinResult, RuntimeError};

#[runmat_macros::register_gpu_spec(builtin_path = "crate::builtins::io::repl_fs::fileparts")]
pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "fileparts",
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
    notes: "Pure path-text manipulation; no GPU execution path.",
};

#[runmat_macros::register_fusion_spec(builtin_path = "crate::builtins::io::repl_fs::fileparts")]
pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "fileparts",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Not fusible; text manipulation only.",
};

const FILEPARTS_OUTPUTS: [BuiltinParamDescriptor; 3] = [
    BuiltinParamDescriptor {
        name: "filepath",
        ty: BuiltinParamType::StringScalar,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Directory part; empty for a bare file name.",
    },
    BuiltinParamDescriptor {
        name: "name",
        ty: BuiltinParamType::StringScalar,
        arity: BuiltinParamArity::Optional,
        default: None,
        description: "File name without extension.",
    },
    BuiltinParamDescriptor {
        name: "ext",
        ty: BuiltinParamType::StringScalar,
        arity: BuiltinParamArity::Optional,
        default: None,
        description: "Extension including the leading dot, or empty.",
    },
];

const FILEPARTS_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "filename",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Path text to split.",
}];

const FILEPARTS_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "[filepath, name, ext] = fileparts(filename)",
    inputs: &FILEPARTS_INPUTS,
    outputs: &FILEPARTS_OUTPUTS,
}];

const FILEPARTS_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.FILEPARTS.INVALID_INPUT",
    identifier: Some("RunMat:fileparts:InvalidInput"),
    when: "The input is not path text (char row vector or string scalar).",
    message: "fileparts: input must be a character vector or string scalar",
};

const FILEPARTS_ERRORS: [BuiltinErrorDescriptor; 1] = [FILEPARTS_ERROR_INVALID_INPUT];

pub const FILEPARTS_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &FILEPARTS_SIGNATURES,
    output_mode: BuiltinOutputMode::ByRequestedOutputCount,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &FILEPARTS_ERRORS,
};

#[runtime_builtin(
    name = "fileparts",
    category = "io/repl_fs",
    summary = "Split a file path into directory, name, and extension parts.",
    keywords = "fileparts,path,directory,extension,filename",
    descriptor(crate::builtins::io::repl_fs::fileparts::FILEPARTS_DESCRIPTOR),
    builtin_path = "crate::builtins::io::repl_fs::fileparts"
)]
async fn fileparts_builtin(filename: Value) -> BuiltinResult<Value> {
    let (text, as_string) = match &filename {
        Value::CharArray(chars) if chars.rows <= 1 => {
            (chars.data.iter().collect::<String>(), false)
        }
        Value::String(text) => (text.clone(), true),
        Value::StringArray(StringArray { data, .. }) if data.len() == 1 => (data[0].clone(), true),
        _ => return Err(invalid_input()),
    };

    let (dir, name, ext) = split_path(&text);
    let outputs = vec![
        text_value(&dir, as_string),
        text_value(&name, as_string),
        text_value(&ext, as_string),
    ];
    if let Some(out_count) = crate::output_count::current_output_count() {
        return Ok(crate::output_count::output_list_with_padding(
            out_count, outputs,
        ));
    }
    Ok(outputs.into_iter().next().expect("fileparts outputs"))
}

fn invalid_input() -> RuntimeError {
    build_runtime_error(FILEPARTS_ERROR_INVALID_INPUT.message)
        .with_builtin("fileparts")
        .with_identifier(
            FILEPARTS_ERROR_INVALID_INPUT
                .identifier
                .expect("identifier"),
        )
        .build()
}

/// Separators: `/` always; `\` additionally on Windows builds
/// (fileparts.q-platform choice). Empty parts are 0x0 char, matching the
/// observed empty results.
fn is_separator(c: char) -> bool {
    c == '/' || (cfg!(windows) && c == '\\')
}

/// Split rules beyond the observed directory-part cases are the documented
/// fileparts.q-outputs choice: last-separator split, then last-dot split of
/// the final component with leading-dot components treated as extensionless.
fn split_path(input: &str) -> (String, String, String) {
    let (dir, rest) = match input.rfind(is_separator) {
        Some(idx) => {
            let dir = if idx == 0 { "/" } else { &input[..idx] };
            (dir.to_string(), &input[idx + 1..])
        }
        None => (String::new(), input),
    };

    let (name, ext) = match rest.rfind('.') {
        Some(dot) if dot > 0 => (rest[..dot].to_string(), rest[dot..].to_string()),
        _ => (rest.to_string(), String::new()),
    };
    (dir, name, ext)
}

fn text_value(text: &str, as_string: bool) -> Value {
    if as_string {
        Value::String(text.to_string())
    } else if text.is_empty() {
        Value::CharArray(CharArray::new(Vec::new(), 0, 0).expect("empty char"))
    } else {
        Value::CharArray(CharArray::new_row(text))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use futures::executor::block_on;

    fn run_fileparts(text: &str) -> Value {
        block_on(super::fileparts_builtin(Value::CharArray(
            CharArray::new_row(text),
        )))
        .expect("fileparts")
    }

    fn assert_char(value: &Value, expected: &str, rows: usize, cols: usize) {
        match value {
            Value::CharArray(chars) => {
                assert_eq!(chars.data.iter().collect::<String>(), expected);
                assert_eq!((chars.rows, chars.cols), (rows, cols));
            }
            other => panic!("expected char array, got {other:?}"),
        }
    }

    // Normative [FR-003-03; fileparts.first-output-dir, fileparts.output-class]
    // — observed case `full-path`: fileparts('/a/b/c.txt') => '/a/b' (1x4).
    #[test]
    fn observed_full_path_dir() {
        assert_char(&run_fileparts("/a/b/c.txt"), "/a/b", 1, 4);
    }

    // Normative — observed case `name-only`: fileparts('file.m') => '' (0x0).
    #[test]
    fn observed_bare_name_dir_is_empty() {
        assert_char(&run_fileparts("file.m"), "", 0, 0);
    }

    // Normative — observed case `trailing-sep`: fileparts('/x/y/') => '/x/y'.
    #[test]
    fn observed_trailing_separator_removed() {
        assert_char(&run_fileparts("/x/y/"), "/x/y", 1, 4);
    }

    // Normative — observed case `no-ext`: fileparts('noext') => '' (0x0).
    #[test]
    fn observed_no_extension_dir_is_empty() {
        assert_char(&run_fileparts("noext"), "", 0, 0);
    }

    // summary_derived [FR-003-04; fileparts.q-outputs]: name/ext outputs per
    // the approved summary; exact values are the documented choice.
    #[test]
    fn summary_derived_three_output_split() {
        let (dir, name, ext) = split_path("/a/b/c.txt");
        assert_eq!(
            (dir.as_str(), name.as_str(), ext.as_str()),
            ("/a/b", "c", ".txt")
        );

        let (dir, name, ext) = split_path("/x/y/");
        assert_eq!(
            (dir.as_str(), name.as_str(), ext.as_str()),
            ("/x/y", "", "")
        );

        let (dir, name, ext) = split_path("file.m");
        assert_eq!(
            (dir.as_str(), name.as_str(), ext.as_str()),
            ("", "file", ".m")
        );
    }

    // unresolved_choice [FR-003-05]: dotfiles, multi-dot names, root paths.
    #[test]
    fn unresolved_choice_edge_splits() {
        let (dir, name, ext) = split_path(".bashrc");
        assert_eq!(
            (dir.as_str(), name.as_str(), ext.as_str()),
            ("", ".bashrc", "")
        );

        let (dir, name, ext) = split_path("archive.tar.gz");
        assert_eq!(
            (dir.as_str(), name.as_str(), ext.as_str()),
            ("", "archive.tar", ".gz")
        );

        let (dir, name, ext) = split_path("/a");
        assert_eq!((dir.as_str(), name.as_str(), ext.as_str()), ("/", "a", ""));
    }

    // unresolved_choice: string input returns string outputs.
    #[test]
    fn unresolved_choice_string_input_returns_string() {
        let result =
            block_on(super::fileparts_builtin(Value::String("/a/b/c.txt".into()))).expect("ok");
        assert_eq!(result, Value::String("/a/b".into()));
    }

    #[test]
    fn invalid_input_errors() {
        let err = block_on(super::fileparts_builtin(Value::Num(5.0))).expect_err("error");
        assert!(err.to_string().contains("fileparts"));
    }
}
