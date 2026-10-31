//! Clean-room `chdir` (change the current folder; return the previous one).
//!
//! Clean-room feature `specs/053-environment/`. Normative behaviour comes
//! only from the approved export `chdir` 0.2.0 (see that feature's
//! `provenance.md`). After its own argument checks, `chdir` calls RunMat's
//! `cd` implementation (decision D-053-4). Observed `MATLAB:*` identifiers
//! are raised as `RunMat:*` (D-053-1).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::Value;

use crate::{build_runtime_error, BuiltinResult, RuntimeError};

const NAME: &str = "chdir";

// ---------------------------------------------------------------------------
// Descriptor
// ---------------------------------------------------------------------------

const FOLDER_INPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "folder",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Target folder as a char row or string scalar (absolute or relative).",
}];

const OLD_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "old",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Folder that was current before the call, as a char row.",
}];

const CURRENT_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "d",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Current folder as a char row.",
}];

const SIGNATURES: [BuiltinSignatureDescriptor; 3] = [
    BuiltinSignatureDescriptor {
        label: "chdir(folder)",
        inputs: &FOLDER_INPUT,
        outputs: &[],
    },
    BuiltinSignatureDescriptor {
        label: "old = chdir(folder)",
        inputs: &FOLDER_INPUT,
        outputs: &OLD_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "d = chdir",
        inputs: &[],
        outputs: &CURRENT_OUTPUT,
    },
];

const NON_EXISTENT_FOLDER: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.CHDIR.NON_EXISTENT_FOLDER",
    identifier: Some("RunMat:cd:NonExistentFolder"),
    when: "The folder does not exist (or is not a folder).",
    message: "chdir: folder does not exist",
};

const NOT_TEXT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.CHDIR.STRING",
    identifier: Some("RunMat:string"),
    when: "The folder argument is not a char row or string scalar.",
    message: "chdir: folder must be a character vector or string scalar",
};

const INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.CHDIR.INVALID_INPUT",
    identifier: Some("RunMat:chdir:InvalidInput"),
    when: "More than one input is given.",
    message: "chdir: expected chdir or chdir(folder)",
};

const TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.CHDIR.TOO_MANY_OUTPUTS",
    identifier: Some("RunMat:chdir:TooManyOutputs"),
    when: "More than one output was requested.",
    message: "chdir: too many output arguments",
};

const ERRORS: [BuiltinErrorDescriptor; 4] = [
    NON_EXISTENT_FOLDER,
    NOT_TEXT,
    INVALID_INPUT,
    TOO_MANY_OUTPUTS,
];

pub const CHDIR_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ERRORS,
};

fn chdir_error(
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
    name = "chdir",
    category = "io/repl_fs",
    summary = "Change the current folder and return the previous one; with no input, return the current folder.",
    keywords = "chdir,cd,current folder,working directory",
    descriptor(crate::builtins::io::repl_fs::cd::chdir::CHDIR_DESCRIPTOR),
    builtin_path = "crate::builtins::io::repl_fs::cd::chdir"
)]
pub(crate) async fn chdir_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    // FR-053-23
    if let Some(count) = crate::output_count::current_output_count() {
        if count > 1 {
            return Err(chdir_error(
                &TOO_MANY_OUTPUTS,
                format!("chdir returns one output, {count} were requested"),
            ));
        }
    }
    match args.len() {
        // FR-053-15, FR-053-25 (display form): the current folder.
        0 => super::execute::run(Vec::new()).await,
        1 => {
            let folder = args.into_iter().next().expect("one chdir argument");
            // FR-053-17, FR-053-25
            if !is_text_scalar(&folder) {
                return Err(chdir_error(
                    &NOT_TEXT,
                    "folder must be a character vector or string scalar",
                ));
            }
            // Empty text keeps cd's RunMat:cd:EmptyFolder (FR-053-25).
            let (raw, target) = super::input::path(folder.clone())?;
            // FR-053-16, FR-053-25
            let is_folder = runmat_filesystem::metadata_async(&target)
                .await
                .map(|meta| meta.is_dir())
                .unwrap_or(false);
            if !is_folder {
                return Err(chdir_error(
                    &NON_EXISTENT_FOLDER,
                    format!("'{raw}' is not an existing folder"),
                ));
            }
            // FR-053-11..14: cd changes folder and returns the previous one.
            super::execute::run(vec![folder]).await
        }
        _ => Err(chdir_error(&INVALID_INPUT, "chdir takes at most one input")),
    }
}

fn is_text_scalar(value: &Value) -> bool {
    match value {
        Value::String(_) => true,
        Value::StringArray(array) => array.data.len() == 1,
        Value::CharArray(chars) => chars.rows == 1 || chars.data.is_empty(),
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod chdir_tests {
    use super::chdir_builtin;
    use crate::builtins::io::repl_fs::REPL_FS_TEST_LOCK;
    use crate::BuiltinResult;
    use futures::executor::block_on;
    use runmat_value::{CellArray, CharArray, Value};
    use std::path::{Path, PathBuf};
    use std::sync::MutexGuard;

    /// Holds the repl_fs test lock (shared with the `cd` tests) and restores
    /// the start folder on drop.
    struct Fixture {
        start: PathBuf,
        _lock: MutexGuard<'static, ()>,
    }

    impl Fixture {
        fn new() -> Self {
            let lock = REPL_FS_TEST_LOCK
                .lock()
                .unwrap_or_else(|value| value.into_inner());
            Self {
                start: std::env::current_dir().expect("current directory"),
                _lock: lock,
            }
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.start);
        }
    }

    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("chdir", args))
    }

    fn chars(text: &str) -> Value {
        Value::CharArray(CharArray::new_row(text))
    }

    fn path_text(path: &Path) -> String {
        path.to_string_lossy().into_owned()
    }

    fn canonical(path: &Path) -> PathBuf {
        std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
    }

    fn cwd() -> PathBuf {
        canonical(&std::env::current_dir().expect("current directory"))
    }

    /// A 1-by-n char row; returns its text.
    fn char_row(value: &Value) -> String {
        match value {
            Value::CharArray(c) => {
                assert_eq!(c.rows, 1, "expected a char row");
                assert_eq!(c.shape, vec![1, c.data.len()]);
                c.data.iter().collect()
            }
            other => panic!("expected a char row, got {other:?}"),
        }
    }

    fn identifier(result: BuiltinResult<Value>) -> String {
        match result {
            Ok(value) => panic!("expected an error, got {value:?}"),
            Err(err) => err.identifier().unwrap_or("<none>").to_string(),
        }
    }

    // --- Observed (normative): one test per observed case --------------------
    // A fresh temporary folder stands in for `tempdir`.

    /// case `changes` — chdir.signature-primary, chdir.changes
    #[test]
    fn observed_changes() {
        let _fx = Fixture::new();
        let target = tempfile::tempdir().unwrap();
        call(&[chars(&path_text(target.path()))]).unwrap();
        assert_eq!(cwd(), canonical(target.path()));
    }

    /// case `trailing-sep` — chdir.trailing-sep
    #[test]
    fn observed_trailing_sep() {
        let _fx = Fixture::new();
        let target = tempfile::tempdir().unwrap();
        let plain = path_text(target.path());
        let with_sep = format!("{plain}{}", std::path::MAIN_SEPARATOR);
        call(&[chars(&with_sep)]).unwrap();
        let first = cwd();
        call(&[chars("..")]).unwrap();
        call(&[chars(&plain)]).unwrap();
        assert_eq!(first, cwd());
        assert_eq!(first, canonical(target.path()));
    }

    /// case `output-old` — chdir.output-old
    #[test]
    fn observed_output_old() {
        let _fx = Fixture::new();
        let start = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        call(&[chars(&path_text(start.path()))]).unwrap();
        let before = std::env::current_dir().unwrap();
        assert_ne!(canonical(&before), canonical(target.path()));
        let old = call(&[chars(&path_text(target.path()))]).unwrap();
        let old = char_row(&old);
        assert_eq!(canonical(Path::new(&old)), canonical(&before));
        assert_eq!(cwd(), canonical(target.path()));
    }

    /// case `no-arg-output` — chdir.no-arg
    #[test]
    fn observed_no_arg_output() {
        let _fx = Fixture::new();
        let before = std::env::current_dir().unwrap();
        let d = call(&[]).unwrap();
        let d = char_row(&d);
        let pwd = block_on(crate::call_builtin_async("pwd", &[])).unwrap();
        assert_eq!(d, char_row(&pwd));
        assert_eq!(canonical(Path::new(&d)), canonical(&before));
        assert_eq!(std::env::current_dir().unwrap(), before);
    }

    /// case `relative` — chdir.relative
    #[test]
    fn observed_relative() {
        let _fx = Fixture::new();
        let root = tempfile::tempdir().unwrap();
        let child = root.path().join("child");
        std::fs::create_dir(&child).unwrap();
        call(&[chars(&path_text(&child))]).unwrap();
        call(&[chars("..")]).unwrap();
        assert_eq!(cwd(), canonical(root.path()));
    }

    /// case `string` — chdir.changes (string-scalar target)
    #[test]
    fn observed_string() {
        let _fx = Fixture::new();
        let target = tempfile::tempdir().unwrap();
        call(&[Value::String(path_text(target.path()))]).unwrap();
        assert_eq!(cwd(), canonical(target.path()));
    }

    /// case `err-missing` — chdir.error-missing
    #[test]
    fn observed_err_missing() {
        let _fx = Fixture::new();
        let root = tempfile::tempdir().unwrap();
        let missing = root.path().join("no_such_folder_n3");
        let before = std::env::current_dir().unwrap();
        assert_eq!(
            identifier(call(&[chars(&path_text(&missing))])),
            "RunMat:cd:NonExistentFolder"
        );
        assert_eq!(std::env::current_dir().unwrap(), before);
    }

    /// case `err-numeric` — chdir.error-numeric
    #[test]
    fn observed_err_numeric() {
        let _fx = Fixture::new();
        assert_eq!(identifier(call(&[Value::Num(5.0)])), "RunMat:string");
    }

    // --- Documented choices ---------------------------------------------------

    /// FR-053-25 (documented choice): other non-text inputs raise RunMat:string.
    #[test]
    fn documented_choice_non_text_inputs() {
        let _fx = Fixture::new();
        let cell = Value::Cell(CellArray::new(vec![Value::from(".")], 1, 1).unwrap());
        for value in [Value::Bool(true), cell] {
            assert_eq!(identifier(call(&[value])), "RunMat:string");
        }
    }

    /// FR-053-25 (documented choice): empty text keeps cd's EmptyFolder; an
    /// existing file is not a folder.
    #[test]
    fn documented_choice_empty_and_file() {
        let _fx = Fixture::new();
        assert_eq!(identifier(call(&[chars("")])), "RunMat:cd:EmptyFolder");
        let root = tempfile::tempdir().unwrap();
        let file = root.path().join("file.txt");
        std::fs::write(&file, b"x").unwrap();
        assert_eq!(
            identifier(call(&[chars(&path_text(&file))])),
            "RunMat:cd:NonExistentFolder"
        );
    }

    /// FR-053-23 (documented choice): arity.
    #[test]
    fn documented_choice_arity() {
        let _fx = Fixture::new();
        assert_eq!(
            identifier(call(&[chars("."), chars(".")])),
            "RunMat:chdir:InvalidInput"
        );
        let _guard = crate::output_count::push_output_count(Some(2));
        let err = block_on(chdir_builtin(Vec::new())).unwrap_err();
        assert_eq!(err.identifier(), Some("RunMat:chdir:TooManyOutputs"));
    }

    /// FR-053-27 (unresolved, chdir.q-warning-coverage): result only, no warning.
    #[test]
    fn unresolved_no_warning() {
        let _fx = Fixture::new();
        assert!(matches!(call(&[]).unwrap(), Value::CharArray(_)));
    }
}
