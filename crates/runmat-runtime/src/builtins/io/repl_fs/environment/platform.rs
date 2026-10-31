//! Clean-room `isunix` and `ismac` (host platform tests).
//!
//! Clean-room feature `specs/053-environment/`. Normative behaviour comes
//! only from the approved exports `isunix` and `ismac` 0.2.0 (see that
//! feature's `provenance.md`), observed on a Linux host only. RunMat reports
//! the platform it was compiled for (decision D-053-2); the values on other
//! hosts are the documented choice FR-053-22. Observed `MATLAB:*`
//! identifiers are raised as `RunMat:*` (D-053-1).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::Value;

use crate::{build_runtime_error, BuiltinResult, RuntimeError};

// ---------------------------------------------------------------------------
// Descriptors
// ---------------------------------------------------------------------------

const TF_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "tf",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Logical scalar.",
}];

const ISUNIX_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "tf = isunix",
    inputs: &[],
    outputs: &TF_OUTPUT,
}];

const ISMAC_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "tf = ismac",
    inputs: &[],
    outputs: &TF_OUTPUT,
}];

const ISUNIX_MAXRHS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ISUNIX.MAXRHS",
    identifier: Some("RunMat:maxrhs"),
    when: "An input argument is given.",
    message: "isunix: too many input arguments",
};

const ISUNIX_TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ISUNIX.TOO_MANY_OUTPUTS",
    identifier: Some("RunMat:isunix:TooManyOutputs"),
    when: "More than one output was requested.",
    message: "isunix: too many output arguments",
};

const ISMAC_MAXRHS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ISMAC.MAXRHS",
    identifier: Some("RunMat:maxrhs"),
    when: "An input argument is given.",
    message: "ismac: too many input arguments",
};

const ISMAC_TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ISMAC.TOO_MANY_OUTPUTS",
    identifier: Some("RunMat:ismac:TooManyOutputs"),
    when: "More than one output was requested.",
    message: "ismac: too many output arguments",
};

const ISUNIX_ERRORS: [BuiltinErrorDescriptor; 2] = [ISUNIX_MAXRHS, ISUNIX_TOO_MANY_OUTPUTS];
const ISMAC_ERRORS: [BuiltinErrorDescriptor; 2] = [ISMAC_MAXRHS, ISMAC_TOO_MANY_OUTPUTS];

pub const ISUNIX_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &ISUNIX_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ISUNIX_ERRORS,
};

pub const ISMAC_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &ISMAC_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ISMAC_ERRORS,
};

fn platform_error(
    name: &'static str,
    descriptor: &'static BuiltinErrorDescriptor,
    detail: impl Into<String>,
) -> RuntimeError {
    let mut builder = build_runtime_error(format!("{name}: {}", detail.into())).with_builtin(name);
    if let Some(identifier) = descriptor.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

/// FR-053-04 and FR-053-23: no inputs, at most one output.
fn check_call(
    name: &'static str,
    args: &[Value],
    maxrhs: &'static BuiltinErrorDescriptor,
    too_many_outputs: &'static BuiltinErrorDescriptor,
) -> BuiltinResult<()> {
    if let Some(count) = crate::output_count::current_output_count() {
        if count > 1 {
            return Err(platform_error(
                name,
                too_many_outputs,
                format!("{name} returns one output, {count} were requested"),
            ));
        }
    }
    if !args.is_empty() {
        return Err(platform_error(
            name,
            maxrhs,
            format!("{name} takes no input arguments"),
        ));
    }
    Ok(())
}

/// FR-053-22: `isunix` is true on every Unix-family compile target.
pub(crate) const HOST_IS_UNIX: bool = cfg!(unix);
/// FR-053-22: `ismac` is true only on macOS.
pub(crate) const HOST_IS_MAC: bool = cfg!(target_os = "macos");

// ---------------------------------------------------------------------------
// Builtins
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "isunix",
    category = "io/repl_fs",
    summary = "True when RunMat runs on a Unix-family platform (Linux, macOS).",
    keywords = "isunix,platform,linux,unix,operating system",
    accel = "cpu",
    descriptor(crate::builtins::io::repl_fs::environment::platform::ISUNIX_DESCRIPTOR),
    builtin_path = "crate::builtins::io::repl_fs::environment::platform"
)]
pub(crate) async fn isunix_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    check_call("isunix", &args, &ISUNIX_MAXRHS, &ISUNIX_TOO_MANY_OUTPUTS)?;
    // FR-053-01, FR-053-03, FR-053-22
    Ok(Value::Bool(HOST_IS_UNIX))
}

#[runtime_builtin(
    name = "ismac",
    category = "io/repl_fs",
    summary = "True when RunMat runs on macOS.",
    keywords = "ismac,platform,macos,operating system",
    accel = "cpu",
    descriptor(crate::builtins::io::repl_fs::environment::platform::ISMAC_DESCRIPTOR),
    builtin_path = "crate::builtins::io::repl_fs::environment::platform"
)]
pub(crate) async fn ismac_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    check_call("ismac", &args, &ISMAC_MAXRHS, &ISMAC_TOO_MANY_OUTPUTS)?;
    // FR-053-02, FR-053-03, FR-053-22
    Ok(Value::Bool(HOST_IS_MAC))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
fn call(name: &str, args: &[Value]) -> BuiltinResult<Value> {
    futures::executor::block_on(crate::call_builtin_async(name, args))
}

#[cfg(test)]
fn identifier(result: BuiltinResult<Value>) -> String {
    match result {
        Ok(value) => panic!("expected an error, got {value:?}"),
        Err(err) => err.identifier().unwrap_or("<none>").to_string(),
    }
}

#[cfg(test)]
mod isunix_tests {
    use super::{call, identifier, isunix_builtin};
    use futures::executor::block_on;
    use runmat_value::Value;

    // --- Observed (normative on a Linux host) --------------------------------

    /// case `value` — isunix.signature-primary, isunix.value
    #[cfg(target_os = "linux")]
    #[test]
    fn observed_value() {
        assert_eq!(call("isunix", &[]).unwrap(), Value::Bool(true));
    }

    /// case `consistency` — isunix.value (isunix true, ismac false)
    #[cfg(target_os = "linux")]
    #[test]
    fn observed_consistency() {
        assert_eq!(call("isunix", &[]).unwrap(), Value::Bool(true));
        assert_eq!(call("ismac", &[]).unwrap(), Value::Bool(false));
    }

    /// case `cobra-linux-test` — isunix.cobra-test (`isunix && ~ismac`)
    #[cfg(target_os = "linux")]
    #[test]
    fn observed_cobra_linux_test() {
        let unix = call("isunix", &[]).unwrap() == Value::Bool(true);
        let mac = call("ismac", &[]).unwrap() == Value::Bool(true);
        assert!(unix && !mac);
    }

    /// case `err-arg` — isunix.error-arg
    #[test]
    fn observed_err_arg() {
        assert_eq!(
            identifier(call("isunix", &[Value::Num(1.0)])),
            "RunMat:maxrhs"
        );
    }

    // --- Documented choices ---------------------------------------------------

    /// FR-053-22 (documented choice): true on every Unix-family target.
    #[test]
    fn documented_choice_host_mapping() {
        assert_eq!(call("isunix", &[]).unwrap(), Value::Bool(cfg!(unix)));
    }

    /// FR-053-23 (documented choice): more than one output.
    #[test]
    fn documented_choice_too_many_outputs() {
        let _guard = crate::output_count::push_output_count(Some(2));
        let err = block_on(isunix_builtin(Vec::new())).unwrap_err();
        assert_eq!(err.identifier(), Some("RunMat:isunix:TooManyOutputs"));
    }

    /// FR-053-27 (unresolved, isunix.q-warning-coverage): result only, no warning.
    #[test]
    fn unresolved_no_warning() {
        assert!(matches!(call("isunix", &[]).unwrap(), Value::Bool(_)));
    }
}

#[cfg(test)]
mod ismac_tests {
    use super::{call, identifier, ismac_builtin};
    use futures::executor::block_on;
    use runmat_value::Value;

    /// case `value` — ismac.signature-primary, ismac.value
    #[cfg(target_os = "linux")]
    #[test]
    fn observed_value() {
        assert_eq!(call("ismac", &[]).unwrap(), Value::Bool(false));
    }

    /// case `err-arg` — ismac.error-arg
    #[test]
    fn observed_err_arg() {
        assert_eq!(
            identifier(call("ismac", &[Value::from("x")])),
            "RunMat:maxrhs"
        );
    }

    /// FR-053-22 (documented choice): true only on macOS.
    #[test]
    fn documented_choice_host_mapping() {
        assert_eq!(
            call("ismac", &[]).unwrap(),
            Value::Bool(cfg!(target_os = "macos"))
        );
    }

    /// FR-053-23 (documented choice): more than one output.
    #[test]
    fn documented_choice_too_many_outputs() {
        let _guard = crate::output_count::push_output_count(Some(2));
        let err = block_on(ismac_builtin(Vec::new())).unwrap_err();
        assert_eq!(err.identifier(), Some("RunMat:ismac:TooManyOutputs"));
    }
}
