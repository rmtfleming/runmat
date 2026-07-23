// Parallel clean-room implementation of `system` (independently developed against the
// matlab-interface-spec black-box specs). 0-6-0 ships the DEFAULT implementation; this copy is
// retained unregistered for differential cross-validation. Original path: crates/runmat-runtime/src/builtins/io/repl_fs/system.rs
//! MATLAB-compatible `system` builtin for RunMat.
//!
//! Clean-room provenance: specs/011-system-invocation (spec `system` 0.2.0,
//! claims system.signature-primary, system.status-and-output). Everything
//! beyond the observed cases (shell selection, stderr, environment, echo on
//! statement calls, cmdout encoding, signal-terminated children, non-Unix
//! platforms) is a documented independent choice recorded in the feature
//! spec; see `system.q-side-effect` / `system.q-outputs`.

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

pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "system",
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
    notes: "Spawns an external process; no GPU execution path.",
};

pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "system",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Not fusible; external side effect.",
};

const SYSTEM_OUTPUTS: [BuiltinParamDescriptor; 2] = [
    BuiltinParamDescriptor {
        name: "status",
        ty: BuiltinParamType::NumericScalar,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Numeric exit status of the command (0 for the observed successful command).",
    },
    BuiltinParamDescriptor {
        name: "cmdout",
        ty: BuiltinParamType::StringScalar,
        arity: BuiltinParamArity::Optional,
        default: None,
        description:
            "Captured standard output as a char row vector, including the trailing newline.",
    },
];

const SYSTEM_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "command",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Operating-system command text to execute.",
}];

const SYSTEM_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "[status, cmdout] = system(command)",
    inputs: &SYSTEM_INPUTS,
    outputs: &SYSTEM_OUTPUTS,
}];

const SYSTEM_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SYSTEM.INVALID_INPUT",
    identifier: Some("RunMat:system:InvalidInput"),
    when: "The command is not a character vector or string scalar.",
    message: "system: command must be a character vector or string scalar",
};

const SYSTEM_ERROR_PLATFORM_UNSUPPORTED: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SYSTEM.PLATFORM_UNSUPPORTED",
    identifier: Some("RunMat:system:PlatformUnsupported"),
    when: "The current platform cannot execute operating-system commands (non-Unix targets, including wasm and Windows, per the recorded gate decision).",
    message: "system: executing operating-system commands is not supported on this platform",
};

const SYSTEM_ERROR_SPAWN_FAILED: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SYSTEM.SPAWN_FAILED",
    identifier: Some("RunMat:system:SpawnFailed"),
    when: "The command interpreter (/bin/sh) could not be started.",
    message: "system: failed to start the command interpreter",
};

const SYSTEM_ERRORS: [BuiltinErrorDescriptor; 3] = [
    SYSTEM_ERROR_INVALID_INPUT,
    SYSTEM_ERROR_PLATFORM_UNSUPPORTED,
    SYSTEM_ERROR_SPAWN_FAILED,
];

pub const SYSTEM_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &SYSTEM_SIGNATURES,
    output_mode: BuiltinOutputMode::ByRequestedOutputCount,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &SYSTEM_ERRORS,
};

async fn system_builtin(command: Value) -> BuiltinResult<Value> {
    let command_text = match &command {
        Value::CharArray(chars) if chars.rows <= 1 => chars.data.iter().collect::<String>(),
        Value::String(text) => text.clone(),
        Value::StringArray(StringArray { data, .. }) if data.len() == 1 => data[0].clone(),
        _ => return Err(invalid_input()),
    };

    let (status, stdout_text) = run_command(&command_text)?;

    let requested = crate::output_count::current_output_count();
    // Statement calls (no requested output) forward captured stdout to the
    // console — recorded gate decision; unobserved in the export, tagged
    // unresolved_choice in tests.
    if requested == Some(0) && !stdout_text.is_empty() {
        crate::console::record_console_output(
            crate::console::ConsoleStream::Stdout,
            stdout_text.clone(),
        );
    }

    let outputs = vec![Value::Num(status), cmdout_value(&stdout_text)];
    if let Some(out_count) = requested {
        return Ok(crate::output_count::output_list_with_padding(
            out_count, outputs,
        ));
    }
    Ok(outputs.into_iter().next().expect("system outputs"))
}

/// Execution model (documented choices under `system.q-side-effect`): the
/// command line is interpreted by `/bin/sh -c` on Unix targets; stdout is
/// captured; stderr passes through to the host process; stdin is closed.
/// Non-Unix targets (wasm, Windows) report a clear unsupported error per the
/// recorded gate decision — no `cmd /C` path.
#[cfg(unix)]
fn run_command(command: &str) -> Result<(f64, String), RuntimeError> {
    use std::process::{Command, Stdio};

    let output = Command::new("/bin/sh")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|_| spawn_failed())?;

    let stdout_text = String::from_utf8_lossy(&output.stdout).into_owned();
    Ok((exit_status_value(&output.status), stdout_text))
}

#[cfg(not(unix))]
fn run_command(_command: &str) -> Result<(f64, String), RuntimeError> {
    Err(platform_unsupported())
}

/// Exit-code mapping: pass the child's exit code through numerically; a
/// signal-terminated child maps to `128 + signal` (Unix convention,
/// documented choice under `system.q-outputs`).
#[cfg(unix)]
fn exit_status_value(status: &std::process::ExitStatus) -> f64 {
    if let Some(code) = status.code() {
        return f64::from(code);
    }
    use std::os::unix::process::ExitStatusExt;
    match status.signal() {
        Some(signal) => f64::from(128 + signal),
        None => -1.0,
    }
}

/// Captured stdout as a char row vector (lossy UTF-8, documented choice);
/// empty stdout is a 0x0 char array (documented choice).
fn cmdout_value(text: &str) -> Value {
    if text.is_empty() {
        Value::CharArray(CharArray::new(Vec::new(), 0, 0).expect("empty char"))
    } else {
        Value::CharArray(CharArray::new_row(text))
    }
}

fn build_error(descriptor: &'static BuiltinErrorDescriptor) -> RuntimeError {
    build_runtime_error(descriptor.message)
        .with_builtin("system")
        .with_identifier(descriptor.identifier.expect("identifier"))
        .build()
}

fn invalid_input() -> RuntimeError {
    build_error(&SYSTEM_ERROR_INVALID_INPUT)
}

#[cfg(unix)]
fn spawn_failed() -> RuntimeError {
    build_error(&SYSTEM_ERROR_SPAWN_FAILED)
}

#[cfg(not(unix))]
fn platform_unsupported() -> RuntimeError {
    build_error(&SYSTEM_ERROR_PLATFORM_UNSUPPORTED)
}

// SANDBOX / TEST-ISOLATION (plan.md, binding): tests spawn ONLY `true`,
// `false`, and `echo hello` — fixed literals, no shell metacharacters, no
// environment mutation, no filesystem writes, no network. Process-spawning
// tests are gated #[cfg(unix)] (export platform GLNXA64).
#[cfg(all(test, feature = "parallel_xval"))]
pub(crate) mod tests {
    use super::*;
    use futures::executor::block_on;

    fn run_system(command: &str) -> BuiltinResult<Value> {
        block_on(super::system_builtin(Value::CharArray(CharArray::new_row(
            command,
        ))))
    }

    #[cfg(unix)]
    fn assert_char(value: &Value, expected: &str, rows: usize, cols: usize) {
        match value {
            Value::CharArray(chars) => {
                assert_eq!(chars.data.iter().collect::<String>(), expected);
                assert_eq!((chars.rows, chars.cols), (rows, cols));
            }
            other => panic!("expected char array, got {other:?}"),
        }
    }

    // Normative [FR-011-01, FR-011-02; system.signature-primary,
    // system.status-and-output] — observed case `status-success`:
    // status = system('true') => 0 (double 1x1).
    #[cfg(unix)]
    #[test]
    fn observed_status_success() {
        assert_eq!(run_system("true").expect("system"), Value::Num(0.0));
    }

    // Normative [FR-011-03; system.status-and-output] — observed case
    // `status-failure`: status = system('false') => 1.
    #[cfg(unix)]
    #[test]
    fn observed_status_failure() {
        assert_eq!(run_system("false").expect("system"), Value::Num(1.0));
    }

    // Normative [FR-011-04; system.status-and-output] — observed case
    // `captured-output`: [status, cmdout] = system('echo hello') =>
    // cmdout is 1x6 char 'hello' + newline, status 0.
    #[cfg(unix)]
    #[test]
    fn observed_captured_output() {
        let _guard = crate::output_count::push_output_count(Some(2));
        let result = run_system("echo hello").expect("system");
        match result {
            Value::OutputList(values) => {
                assert_eq!(values.len(), 2);
                assert_eq!(values[0], Value::Num(0.0));
                assert_char(&values[1], "hello\n", 1, 6);
            }
            other => panic!("expected output list, got {other:?}"),
        }
    }

    // unresolved_choice [FR-011-05]: empty stdout => 0x0 char cmdout
    // (documented choice; unobserved in the export).
    #[cfg(unix)]
    #[test]
    fn unresolved_choice_empty_stdout_two_outputs() {
        let _guard = crate::output_count::push_output_count(Some(2));
        let result = run_system("true").expect("system");
        match result {
            Value::OutputList(values) => {
                assert_eq!(values[0], Value::Num(0.0));
                assert_char(&values[1], "", 0, 0);
            }
            other => panic!("expected output list, got {other:?}"),
        }
    }

    // unresolved_choice [FR-011-05]: statement calls (no requested output)
    // forward captured stdout to the console — recorded gate decision.
    #[cfg(unix)]
    #[test]
    fn unresolved_choice_statement_call_echoes_stdout() {
        use crate::console::{reset_thread_buffer, take_thread_buffer, ConsoleStream};

        reset_thread_buffer();
        let _guard = crate::output_count::push_output_count(Some(0));
        let result = run_system("echo hello").expect("system");
        assert_eq!(result, Value::OutputList(Vec::new()));

        let entries = take_thread_buffer();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].stream, ConsoleStream::Stdout);
        assert_eq!(entries[0].text, "hello\n");
    }

    // unresolved_choice [FR-011-05]: non-text command input errors
    // (RunMat:system:InvalidInput); no process is spawned.
    #[test]
    fn invalid_input_errors() {
        let err = block_on(super::system_builtin(Value::Num(5.0))).expect_err("error");
        assert!(err.to_string().contains("system"));
    }

    // unresolved_choice [FR-011-05]: non-Unix targets report a clear
    // unsupported error (recorded gate decision; no cmd /C path).
    #[cfg(not(unix))]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn unsupported_platform_errors() {
        let err = run_system("true").expect_err("error");
        assert!(err.to_string().contains("not supported"));
    }

    // [SC-011-2] `system` is registered and discoverable by name; no
    // process is spawned.
    #[test]
    fn system_builtin_is_registered() {
        assert!(runmat_builtins::builtin_function_by_name("system").is_some());
    }
}
