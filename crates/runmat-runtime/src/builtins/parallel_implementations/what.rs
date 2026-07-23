// Parallel clean-room implementation of `what` (independently developed against the
// matlab-interface-spec black-box specs). 0-6-0 ships the DEFAULT implementation; this copy is
// retained unregistered for differential cross-validation. Original path: crates/runmat-runtime/src/builtins/io/repl_fs/what.rs
//! MATLAB-compatible `what` builtin for RunMat.
//!
//! Clean-room feature `specs/032-object-introspection/` (SpecKit id
//! `032-object-introspection`). Normative behaviour comes exclusively from the
//! approved Tier B batch-3 export `what` 0.2.0 (claims `what.signature-primary`,
//! `what.output-class`, `what.folder-struct`; source commit pin
//! `1b019121bb747e63d69a3eb005534cb60669e5f4`).
//!
//! Placement: `io/repl_fs/` alongside `dir`/`ls` — `what` consults the
//! filesystem through RunMat's runtime filesystem layer (`runmat_filesystem`),
//! never through MATLAB, so it belongs with the other REPL filesystem
//! builtins rather than in `introspection/`.
//!
//! Gate-ratified framing: the export observed `what(tempdir)` as a 1x1 struct;
//! the field NAMES and file CONTENTS are folder-dependent and unresolved in
//! the export (`what.q-fields`, `what.q-content`). RunMat returns a scalar
//! struct with the documented field set (`m`, `mat`, `mex`, `mlx`, `mlapp`,
//! `classes`, `packages`, `path`); the presence of those fields and the
//! struct class/size are asserted, but specific listed contents are
//! environment-specific. See `specs/032-object-introspection/spec.md`.

use std::path::{Path, PathBuf};

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    CharArray, StructValue, Value,
};
use runmat_filesystem as vfs;
use runmat_macros::runtime_builtin;

use crate::builtins::common::fs::{expand_user_path, path_to_string};
use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ShapeRequirements,
};
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "what",
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
    notes: "Host-only filesystem builtin. Providers do not participate; GPU-resident inputs are gathered to host memory.",
};

pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "what",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "I/O builtins do not participate in fusion plans; metadata registered for completeness.",
};

const BUILTIN_NAME: &str = "what";

const WHAT_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "s",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "1x1 struct describing the MATLAB-related files in the folder.",
}];

const WHAT_INPUTS_NONE: [BuiltinParamDescriptor; 0] = [];
const WHAT_INPUTS_FOLDER: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "folder",
    ty: BuiltinParamType::StringScalar,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Folder path to inspect.",
}];

const WHAT_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "s = what",
        inputs: &WHAT_INPUTS_NONE,
        outputs: &WHAT_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "s = what(folder)",
        inputs: &WHAT_INPUTS_FOLDER,
        outputs: &WHAT_OUTPUT,
    },
];

const WHAT_ERROR_TOO_MANY_INPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.WHAT.TOO_MANY_INPUTS",
    identifier: Some("RunMat:what:TooManyInputs"),
    when: "More than one input argument is provided.",
    message: "what: too many input arguments",
};

const WHAT_ERROR_FOLDER_ARG: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.WHAT.FOLDER_ARG",
    identifier: Some("RunMat:what:FolderArg"),
    when: "Folder argument is not a character vector or string scalar.",
    message: "what: folder must be a character vector or string scalar",
};

const WHAT_ERROR_ACCESS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.WHAT.ACCESS",
    identifier: Some("RunMat:what:Access"),
    when: "The requested folder cannot be read.",
    message: "what: unable to read folder",
};

const WHAT_ERRORS: [BuiltinErrorDescriptor; 3] = [
    WHAT_ERROR_TOO_MANY_INPUTS,
    WHAT_ERROR_FOLDER_ARG,
    WHAT_ERROR_ACCESS,
];

pub const WHAT_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &WHAT_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &WHAT_ERRORS,
};

fn what_error(error: &'static BuiltinErrorDescriptor) -> RuntimeError {
    let mut builder = build_runtime_error(error.message).with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

fn what_error_detail(
    error: &'static BuiltinErrorDescriptor,
    detail: impl AsRef<str>,
) -> RuntimeError {
    let mut builder = build_runtime_error(format!("{} ({})", error.message, detail.as_ref()))
        .with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

fn scalar_text(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::StringArray(array) if array.data.len() == 1 => Some(array.data[0].clone()),
        Value::CharArray(chars) if chars.rows <= 1 => {
            Some(chars.data.iter().collect::<String>().trim_end().to_string())
        }
        _ => None,
    }
}

#[derive(Default)]
struct WhatListing {
    m: Vec<String>,
    mat: Vec<String>,
    mex: Vec<String>,
    mlx: Vec<String>,
    mlapp: Vec<String>,
    classes: Vec<String>,
    packages: Vec<String>,
}

/// Classify a directory entry by name/kind into the documented buckets.
///
/// Classification is a documented RunMat choice (the export does not fix the
/// field mapping): files by extension; `@Name` folders are classes, `+name`
/// folders are packages.
fn classify(listing: &mut WhatListing, name: &str, is_dir: bool) {
    if is_dir {
        if let Some(class) = name.strip_prefix('@') {
            if !class.is_empty() {
                listing.classes.push(class.to_string());
            }
        } else if let Some(pkg) = name.strip_prefix('+') {
            if !pkg.is_empty() {
                listing.packages.push(pkg.to_string());
            }
        }
        return;
    }
    let Some((_, ext)) = name.rsplit_once('.') else {
        return;
    };
    let ext_lower = ext.to_ascii_lowercase();
    match ext_lower.as_str() {
        "m" => listing.m.push(name.to_string()),
        "mat" => listing.mat.push(name.to_string()),
        "mlx" => listing.mlx.push(name.to_string()),
        "mlapp" => listing.mlapp.push(name.to_string()),
        _ if ext_lower.starts_with("mex") => listing.mex.push(name.to_string()),
        _ => {}
    }
}

async fn scan_folder(folder: &Path) -> BuiltinResult<WhatListing> {
    let mut listing = WhatListing::default();
    let entries = vfs::read_dir_async(folder)
        .await
        .map_err(|err| what_error_detail(&WHAT_ERROR_ACCESS, err.to_string()))?;
    for entry in entries {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == "." || name == ".." {
            continue;
        }
        let is_dir = match vfs::metadata_async(&entry.path()).await {
            Ok(meta) => meta.is_dir(),
            Err(_) => false,
        };
        classify(&mut listing, &name, is_dir);
    }
    // Ascending order within each bucket (documented choice; the export does
    // not fix ordering).
    for bucket in [
        &mut listing.m,
        &mut listing.mat,
        &mut listing.mex,
        &mut listing.mlx,
        &mut listing.mlapp,
        &mut listing.classes,
        &mut listing.packages,
    ] {
        bucket.sort();
    }
    Ok(listing)
}

fn column_cell(names: Vec<String>) -> BuiltinResult<Value> {
    let rows = names.len();
    let cells: Vec<Value> = names
        .into_iter()
        .map(|name| Value::CharArray(CharArray::new_row(&name)))
        .collect();
    crate::make_cell(cells, rows, 1).map_err(|detail| what_error_detail(&WHAT_ERROR_ACCESS, detail))
}

fn build_struct(path: &str, listing: WhatListing) -> BuiltinResult<Value> {
    let mut st = StructValue::new();
    st.insert("m", column_cell(listing.m)?);
    st.insert("mat", column_cell(listing.mat)?);
    st.insert("mex", column_cell(listing.mex)?);
    st.insert("mlx", column_cell(listing.mlx)?);
    st.insert("mlapp", column_cell(listing.mlapp)?);
    st.insert("classes", column_cell(listing.classes)?);
    st.insert("packages", column_cell(listing.packages)?);
    st.insert("path", Value::CharArray(CharArray::new_row(path)));
    Ok(Value::Struct(st))
}

async fn resolve_folder(text: &str) -> BuiltinResult<PathBuf> {
    let trimmed = text.trim();
    let expanded = if trimmed.is_empty() {
        vfs::current_dir()
            .map_err(|err| what_error_detail(&WHAT_ERROR_ACCESS, err.to_string()))?
            .to_string_lossy()
            .into_owned()
    } else {
        expand_user_path(trimmed, BUILTIN_NAME)
            .map_err(|detail| what_error_detail(&WHAT_ERROR_ACCESS, detail))?
    };
    let path = PathBuf::from(&expanded);
    let absolute = if super::support::is_rooted_path(&path) {
        path
    } else {
        vfs::current_dir()
            .map_err(|err| what_error_detail(&WHAT_ERROR_ACCESS, err.to_string()))?
            .join(path)
    };
    Ok(vfs::canonicalize_async(&absolute).await.unwrap_or(absolute))
}

async fn what_builtin(args: Vec<Value>) -> crate::BuiltinResult<Value> {
    let folder_text = match args.len() {
        0 => String::new(),
        1 => {
            let gathered = gather_if_needed_async(&args[0])
                .await
                .map_err(|_| what_error(&WHAT_ERROR_FOLDER_ARG))?;
            scalar_text(&gathered).ok_or_else(|| what_error(&WHAT_ERROR_FOLDER_ARG))?
        }
        _ => return Err(what_error(&WHAT_ERROR_TOO_MANY_INPUTS)),
    };
    let folder = resolve_folder(&folder_text).await?;
    let listing = scan_folder(&folder).await?;
    build_struct(&path_to_string(&folder), listing)
}

// ---------------------------------------------------------------------------
// Tests — use the runtime filesystem layer against real temp folders; never
// MATLAB (Constitution II). Tiers: `observed_*` reproduce the export's struct
// class/size and documented field presence; `unresolved_choice_*` cover
// documented independent choices (field set, classification, empty folder) and
// the non-assertion of specific listed contents.
// ---------------------------------------------------------------------------

#[cfg(all(test, feature = "parallel_xval"))]
pub(crate) mod tests {
    use super::super::REPL_FS_TEST_LOCK;
    use super::*;
    use runmat_filesystem::File;
    use tempfile::tempdir;

    /// Documented field set (independent RunMat choice; the export leaves field
    /// names unresolved — `what.q-fields`).
    const WHAT_FIELDS: [&str; 8] = [
        "m", "mat", "mex", "mlx", "mlapp", "classes", "packages", "path",
    ];

    fn what_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
        futures::executor::block_on(super::what_builtin(args))
    }

    fn field_char(st: &StructValue, name: &str) -> String {
        match st.fields.get(name) {
            Some(Value::CharArray(ca)) => ca.data.iter().collect(),
            other => panic!("field '{name}' is not a char array: {other:?}"),
        }
    }

    fn field_cell_names(st: &StructValue, name: &str) -> Vec<String> {
        match st.fields.get(name) {
            Some(Value::Cell(cell)) => {
                assert_eq!(cell.cols, 1, "field '{name}' must be a column cell");
                cell.data
                    .iter()
                    .map(|entry| match entry {
                        Value::CharArray(ca) => ca.data.iter().collect(),
                        other => panic!("field '{name}' element is not char: {other:?}"),
                    })
                    .collect()
            }
            other => panic!("field '{name}' is not a cell: {other:?}"),
        }
    }

    /// Observed case `tempdir`: `what(folder)` returns a 1x1 struct.
    /// [FR-032-03; what.output-class, what.folder-struct]. The specific listed
    /// contents are environment-specific and are NOT asserted here.
    #[test]
    fn observed_what_returns_scalar_struct() {
        let _lock = REPL_FS_TEST_LOCK
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let dir = tempdir().expect("tempdir");
        let value = what_builtin(vec![Value::from(dir.path().to_string_lossy().to_string())])
            .expect("what(folder)");
        match value {
            // A RunMat scalar struct is inherently 1x1.
            Value::Struct(_) => {}
            other => panic!("expected 1x1 struct, got {other:?}"),
        }
    }

    /// Documented field set: the returned struct exposes every documented
    /// field. [FR-032-03; what.q-fields carried as a documented choice].
    #[test]
    fn observed_what_struct_has_documented_fields() {
        let _lock = REPL_FS_TEST_LOCK
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let dir = tempdir().expect("tempdir");
        let Value::Struct(st) =
            what_builtin(vec![Value::from(dir.path().to_string_lossy().to_string())])
                .expect("what(folder)")
        else {
            panic!("expected struct");
        };
        for field in WHAT_FIELDS {
            assert!(st.fields.contains_key(field), "missing field '{field}'");
        }
        // `path` is a char row; the file-list fields are column cells.
        assert!(!field_char(&st, "path").is_empty());
        for field in ["m", "mat", "mex", "mlx", "mlapp", "classes", "packages"] {
            let _ = field_cell_names(&st, field);
        }
    }

    /// Documented classification (unobserved contents): `.m`/`.mat`/`.mlx`
    /// files, `@Class` and `+pkg` folders land in the right buckets. Contents
    /// are environment-specific in general; here the folder is fully
    /// controlled so the mapping choice can be exercised.
    #[test]
    fn unresolved_choice_classifies_files_and_folders() {
        let _lock = REPL_FS_TEST_LOCK
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let dir = tempdir().expect("tempdir");
        File::create(dir.path().join("alpha.m")).expect("m file");
        File::create(dir.path().join("beta.m")).expect("m file");
        File::create(dir.path().join("data.mat")).expect("mat file");
        File::create(dir.path().join("live.mlx")).expect("mlx file");
        File::create(dir.path().join("readme.txt")).expect("txt file");
        futures::executor::block_on(vfs::create_dir_async(dir.path().join("@Widget")))
            .expect("class folder");
        futures::executor::block_on(vfs::create_dir_async(dir.path().join("+mypkg")))
            .expect("package folder");
        futures::executor::block_on(vfs::create_dir_async(dir.path().join("plain")))
            .expect("plain folder");

        let Value::Struct(st) =
            what_builtin(vec![Value::from(dir.path().to_string_lossy().to_string())])
                .expect("what(folder)")
        else {
            panic!("expected struct");
        };
        assert_eq!(
            field_cell_names(&st, "m"),
            vec!["alpha.m".to_string(), "beta.m".to_string()]
        );
        assert_eq!(field_cell_names(&st, "mat"), vec!["data.mat".to_string()]);
        assert_eq!(field_cell_names(&st, "mlx"), vec!["live.mlx".to_string()]);
        assert_eq!(field_cell_names(&st, "classes"), vec!["Widget".to_string()]);
        assert_eq!(field_cell_names(&st, "packages"), vec!["mypkg".to_string()]);
        // A plain folder and a .txt file are not classified.
        assert!(field_cell_names(&st, "mlapp").is_empty());
    }

    /// Documented choice: an empty folder yields a struct whose file lists are
    /// all 0-by-1 cells (empty, but present).
    #[test]
    fn unresolved_choice_empty_folder_lists_are_empty() {
        let _lock = REPL_FS_TEST_LOCK
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let dir = tempdir().expect("tempdir");
        let Value::Struct(st) =
            what_builtin(vec![Value::from(dir.path().to_string_lossy().to_string())])
                .expect("what(folder)")
        else {
            panic!("expected struct");
        };
        for field in ["m", "mat", "mex", "mlx", "mlapp", "classes", "packages"] {
            assert!(
                field_cell_names(&st, field).is_empty(),
                "field '{field}' should be empty"
            );
        }
    }

    /// Documented choice: the no-argument form (unobserved) inspects the
    /// current directory and still returns a scalar struct.
    #[test]
    fn unresolved_choice_no_argument_scans_current_directory() {
        let _lock = REPL_FS_TEST_LOCK
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let value = what_builtin(Vec::new()).expect("what()");
        match value {
            Value::Struct(st) => assert!(st.fields.contains_key("path")),
            other => panic!("expected struct, got {other:?}"),
        }
    }

    /// Documented choice: more than one argument errors loudly.
    #[test]
    fn unresolved_choice_too_many_inputs_errors() {
        let err = what_builtin(vec![Value::from("a"), Value::from("b")]).unwrap_err();
        assert_eq!(
            err.identifier().unwrap_or("<none>"),
            "RunMat:what:TooManyInputs"
        );
    }

    /// Documented choice: a non-text folder argument errors.
    #[test]
    fn unresolved_choice_non_text_argument_errors() {
        let err = what_builtin(vec![Value::Num(3.0)]).unwrap_err();
        assert_eq!(
            err.identifier().unwrap_or("<none>"),
            "RunMat:what:FolderArg"
        );
    }

    /// Descriptor advertises the observed primary signature `s = what(folder)`.
    #[test]
    fn descriptor_exposes_primary_signature() {
        let labels: Vec<&str> = WHAT_DESCRIPTOR
            .signatures
            .iter()
            .map(|sig| sig.label)
            .collect();
        assert!(labels.contains(&"s = what(folder)"));
    }

    /// Macro registration: the builtin is discoverable by name.
    #[test]
    fn registration_discoverable_by_name() {
        assert!(runmat_builtins::builtin_function_by_name("what").is_some());
    }

    /// GPU/fusion metadata is registered under the builtin's own name.
    #[test]
    fn gpu_and_fusion_specs_match_name() {
        assert_eq!(GPU_SPEC.name, "what");
        assert_eq!(FUSION_SPEC.name, "what");
    }
}
