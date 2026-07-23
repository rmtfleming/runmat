//! MATLAB-compatible `computer` builtin for RunMat.
//!
//! Clean-room provenance: specs/019-computer (spec `computer` 0.2.0, claims
//! computer.signature-primary, computer.output-class,
//! computer.platform-strings).
//!
//! Observed behaviour (normative, captured on Linux x86_64 / GLNXA64):
//! `computer` => `'GLNXA64'` (1x7 char) and `computer('arch')` =>
//! `'glnxa64'` (1x7 char). The identifier pairs for other platforms, the
//! option-handling edge cases, and the error paths are documented
//! independent choices; the `[str, maxsize, endian]` multi-output form is
//! unresolved in the approved export (computer.q-multi-output) and omitted.

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    CharArray, ResolveContext, Type, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ShapeRequirements,
};
use crate::{build_runtime_error, BuiltinResult, RuntimeError};

const BUILTIN_NAME: &str = "computer";

#[runmat_macros::register_gpu_spec(builtin_path = "crate::builtins::introspection::computer")]
pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "computer",
    op_kind: GpuOpKind::Custom("introspection"),
    supported_precisions: &[],
    broadcast: BroadcastSemantics::None,
    provider_hooks: &[],
    constant_strategy: ConstantStrategy::InlineLiteral,
    residency: ResidencyPolicy::GatherImmediately,
    nan_mode: ReductionNaN::Include,
    two_pass_threshold: None,
    workgroup_size: None,
    accepts_nan_mode: false,
    notes: "Host platform metadata query; no tensor inputs and no GPU execution path.",
};

#[runmat_macros::register_fusion_spec(builtin_path = "crate::builtins::introspection::computer")]
pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "computer",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Metadata query executed on the host; never fusible.",
};

const COMPUTER_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "str",
    ty: BuiltinParamType::StringScalar,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Platform identifier as a char row vector.",
}];

const COMPUTER_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "option",
    ty: BuiltinParamType::StringScalar,
    arity: BuiltinParamArity::Optional,
    default: None,
    description: "`'arch'` selects the lower-case architecture string.",
}];

const COMPUTER_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "str = computer",
        inputs: &[],
        outputs: &COMPUTER_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "str = computer('arch')",
        inputs: &COMPUTER_INPUTS,
        outputs: &COMPUTER_OUTPUT,
    },
];

const COMPUTER_ERROR_INVALID_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.COMPUTER.INVALID_OPTION",
    identifier: Some("RunMat:computer:InvalidOption"),
    when: "The option argument is not the text 'arch'.",
    message: "computer: option must be 'arch'",
};

const COMPUTER_ERROR_TOO_MANY_INPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.COMPUTER.TOO_MANY_INPUTS",
    identifier: Some("RunMat:computer:TooManyInputs"),
    when: "More than one input argument is provided.",
    message: "computer: too many input arguments",
};

const COMPUTER_ERRORS: [BuiltinErrorDescriptor; 2] = [
    COMPUTER_ERROR_INVALID_OPTION,
    COMPUTER_ERROR_TOO_MANY_INPUTS,
];

pub const COMPUTER_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &COMPUTER_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &COMPUTER_ERRORS,
};

fn computer_error(error: &'static BuiltinErrorDescriptor) -> RuntimeError {
    let mut builder = build_runtime_error(error.message).with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

/// Map a host OS/architecture pair to the (upper-case platform identifier,
/// lower-case architecture string) pair.
///
/// Only the `("linux", "x86_64")` row is observed and normative
/// [computer.platform-strings]. The macOS and Windows rows are documented
/// independent choices (unobserved), and any other combination falls back to
/// a synthesized `OS-ARCH` pair by documented choice so the result is always
/// a non-empty char row vector (see specs/019-computer/spec.md, FR-019-03).
pub(crate) fn platform_strings(os: &str, arch: &str) -> (String, String) {
    match (os, arch) {
        ("linux", "x86_64") => ("GLNXA64".to_string(), "glnxa64".to_string()),
        ("macos", "aarch64") => ("MACA64".to_string(), "maca64".to_string()),
        ("macos", "x86_64") => ("MACI64".to_string(), "maci64".to_string()),
        ("windows", "x86_64") => ("PCWIN64".to_string(), "win64".to_string()),
        (os, arch) => {
            let combined = format!("{os}-{arch}");
            (combined.to_ascii_uppercase(), combined.to_ascii_lowercase())
        }
    }
}

fn host_platform_strings() -> (String, String) {
    platform_strings(std::env::consts::OS, std::env::consts::ARCH)
}

fn char_row(text: &str) -> Value {
    Value::CharArray(CharArray::new_row(text))
}

fn option_text(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::StringArray(array) if array.data.len() == 1 => Some(array.data[0].clone()),
        Value::CharArray(array) if array.rows == 1 => Some(array.data.iter().collect()),
        _ => None,
    }
}

/// Dispatch over the raw argument list. Option handling beyond the observed
/// exact `'arch'` text (case-insensitivity, string scalars, rejection of
/// anything else) is the documented FR-019-04 choice.
fn dispatch_computer(args: &[Value]) -> BuiltinResult<Value> {
    if args.len() > 1 {
        return Err(computer_error(&COMPUTER_ERROR_TOO_MANY_INPUTS));
    }
    let (default_id, arch_id) = host_platform_strings();
    let Some(option) = args.first() else {
        return Ok(char_row(&default_id));
    };
    match option_text(option) {
        Some(text) if text.eq_ignore_ascii_case("arch") => Ok(char_row(&arch_id)),
        _ => Err(computer_error(&COMPUTER_ERROR_INVALID_OPTION)),
    }
}

fn computer_type(_args: &[Type], _context: &ResolveContext) -> Type {
    Type::String
}

#[runtime_builtin(
    name = "computer",
    category = "introspection",
    summary = "Return the platform identifier for the machine RunMat is running on.",
    keywords = "computer,platform,architecture,arch,system,host",
    examples = "id = computer(); arch = computer('arch');",
    type_resolver(computer_type),
    descriptor(crate::builtins::introspection::computer::COMPUTER_DESCRIPTOR),
    builtin_path = "crate::builtins::introspection::computer"
)]
async fn computer_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    dispatch_computer(&args)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use futures::executor::block_on;

    fn run_computer(args: Vec<Value>) -> BuiltinResult<Value> {
        block_on(super::computer_builtin(args))
    }

    fn assert_char_row(value: &Value, expected: &str) {
        match value {
            Value::CharArray(chars) => {
                assert_eq!(chars.data.iter().collect::<String>(), expected);
                assert_eq!((chars.rows, chars.cols), (1, expected.chars().count()));
            }
            other => panic!("expected char row vector, got {other:?}"),
        }
    }

    // Normative [FR-019-01, FR-019-02; computer.platform-strings,
    // computer.output-class] — observed cases `default` ('GLNXA64', 1x7) and
    // `arch` ('glnxa64', 1x7) on GLNXA64. Host independence: the observed
    // pair is asserted through the pure mapping with the observed platform
    // inputs (linux/x86_64) rather than by querying the live host.
    #[test]
    fn observed_default_linux_x86_64_maps_to_glnxa64() {
        let (upper, lower) = platform_strings("linux", "x86_64");
        assert_eq!(upper, "GLNXA64");
        assert_eq!(lower, "glnxa64");
    }

    // Normative [FR-019-01, FR-019-02] — end-to-end reproduction of both
    // observed cases; compiled only on the observed platform (Linux x86_64).
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    #[test]
    fn observed_live_builtin_glnxa64_pair() {
        let value = run_computer(Vec::new()).expect("computer");
        assert_char_row(&value, "GLNXA64");
        let value = run_computer(vec![Value::CharArray(CharArray::new_row("arch"))])
            .expect("computer('arch')");
        assert_char_row(&value, "glnxa64");
    }

    // Normative [FR-019-01, FR-019-02; computer.output-class] — on any host
    // both forms return a char row vector consistent with the mapping.
    #[test]
    fn observed_output_class_char_row_vector() {
        let (upper, lower) = super::host_platform_strings();
        assert_char_row(&run_computer(Vec::new()).expect("computer"), &upper);
        assert_char_row(
            &run_computer(vec![Value::String("arch".to_string())]).expect("computer('arch')"),
            &lower,
        );
    }

    // unresolved_choice [FR-019-03]: identifier pairs for non-observed
    // platforms are documented choices, not observations.
    #[test]
    fn unresolved_choice_documented_platform_pairs() {
        assert_eq!(
            platform_strings("macos", "aarch64"),
            ("MACA64".to_string(), "maca64".to_string())
        );
        assert_eq!(
            platform_strings("macos", "x86_64"),
            ("MACI64".to_string(), "maci64".to_string())
        );
        assert_eq!(
            platform_strings("windows", "x86_64"),
            ("PCWIN64".to_string(), "win64".to_string())
        );
    }

    // unresolved_choice [FR-019-03]: unmapped platforms synthesize an
    // OS-ARCH pair so the output stays a non-empty char row vector.
    #[test]
    fn unresolved_choice_unmapped_platform_fallback() {
        assert_eq!(
            platform_strings("linux", "aarch64"),
            ("LINUX-AARCH64".to_string(), "linux-aarch64".to_string())
        );
        assert_eq!(
            platform_strings("wasm", "wasm32"),
            ("WASM-WASM32".to_string(), "wasm-wasm32".to_string())
        );
    }

    // unresolved_choice [FR-019-04]: only exact 'arch' (char) is observed;
    // case-insensitive matching is the documented choice.
    #[test]
    fn unresolved_choice_arch_option_case_insensitive() {
        let (_, lower) = super::host_platform_strings();
        let value = run_computer(vec![Value::CharArray(CharArray::new_row("ARCH"))])
            .expect("computer('ARCH')");
        assert_char_row(&value, &lower);
    }

    // unresolved_choice [FR-019-04]: unrecognized options (e.g. 'version',
    // never observed) are rejected with a stable identifier.
    #[test]
    fn unresolved_choice_unrecognized_option_errors() {
        let err = run_computer(vec![Value::CharArray(CharArray::new_row("version"))])
            .expect_err("computer('version') should be rejected");
        assert_eq!(err.identifier(), Some("RunMat:computer:InvalidOption"));
    }

    // unresolved_choice [FR-019-04]: non-text options are rejected.
    #[test]
    fn unresolved_choice_non_text_option_errors() {
        let err = run_computer(vec![Value::Num(1.0)]).expect_err("computer(1) should be rejected");
        assert_eq!(err.identifier(), Some("RunMat:computer:InvalidOption"));
    }

    // unresolved_choice [FR-019-04]: more than one argument is rejected.
    #[test]
    fn unresolved_choice_too_many_inputs_errors() {
        let err = run_computer(vec![
            Value::String("arch".to_string()),
            Value::String("arch".to_string()),
        ])
        .expect_err("computer('arch','arch') should be rejected");
        assert_eq!(err.identifier(), Some("RunMat:computer:TooManyInputs"));
    }

    // Registration is discoverable through the builtin registry (SC-019-2).
    #[test]
    fn computer_is_registered() {
        assert!(runmat_builtins::builtin_function_by_name("computer").is_some());
    }

    // Descriptor and acceleration metadata stay consistent [FR-019-05:
    // Fixed single output; multi-output form omitted per
    // computer.q-multi-output].
    #[test]
    fn descriptor_and_spec_metadata_consistent() {
        assert_eq!(COMPUTER_DESCRIPTOR.signatures.len(), 2);
        assert!(matches!(
            COMPUTER_DESCRIPTOR.output_mode,
            BuiltinOutputMode::Fixed
        ));
        assert_eq!(COMPUTER_DESCRIPTOR.errors.len(), 2);
        assert_eq!(GPU_SPEC.name, "computer");
        assert_eq!(FUSION_SPEC.name, "computer");
    }

    #[test]
    fn computer_type_is_string() {
        assert_eq!(
            computer_type(&[], &ResolveContext::new(Vec::new())),
            Type::String
        );
    }
}
