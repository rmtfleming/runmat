// Parallel clean-room implementation of `version` (independently developed against the
// matlab-interface-spec black-box specs). 0-6-0 ships the DEFAULT implementation; this copy is
// retained unregistered for differential cross-validation. Original path: crates/runmat-runtime/src/builtins/introspection/version.rs
//! RunMat-honest `version` builtin (MATLAB-compatible contract).
//!
//! Clean-room provenance: specs/033-environment-identity (spec `version`
//! 0.2.0, claims version.signature-primary, version.output-class,
//! version.version-strings).
//!
//! CONTRACT vs IDENTITY (Constitution Principle II / honesty): the approved
//! export records MATLAB's *own* observed identity — `version` =>
//! `'26.1.0.3276743 (R2026a) Update 3'` and `version('-release')` =>
//! `'2026a'`. Those strings are MATLAB's identity and are **non-normative**
//! here: RunMat is not MATLAB, and returning them verbatim would be factually
//! false. RunMat honours only the observable *contract* (output class `char`,
//! a char row vector; a `-release` substring form) while returning its **own**
//! honest identity: the workspace version `env!("CARGO_PKG_VERSION")`
//! (`0.5.6`) and a RunMat-defined release tag. The specific string values are
//! RunMat's, not MATLAB's.

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

const BUILTIN_NAME: &str = "version";

/// RunMat's own workspace version, taken from the crate metadata at compile
/// time (`0.5.6`). This is RunMat's honest identity, not MATLAB's.
pub(crate) const RUNMAT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// RunMat-defined release tag returned by `version('-release')`. This is a
/// documented independent choice signalling the MATLAB release RunMat targets
/// for compatibility; it is deliberately NOT MATLAB's own release string
/// (`'2026a'`), which would be an impersonation.
pub(crate) const RUNMAT_RELEASE: &str = "R2026a-compat";

pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "version",
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
    notes: "Runtime identity query; no tensor inputs and no GPU execution path.",
};

pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "version",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Identity query executed on the host; never fusible.",
};

const VERSION_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "v",
    ty: BuiltinParamType::StringScalar,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "RunMat version text as a char row vector.",
}];

const VERSION_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "option",
    ty: BuiltinParamType::StringScalar,
    arity: BuiltinParamArity::Optional,
    default: None,
    description: "`'-release'` selects the RunMat release tag substring.",
}];

const VERSION_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "v = version",
        inputs: &[],
        outputs: &VERSION_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "v = version('-release')",
        inputs: &VERSION_INPUTS,
        outputs: &VERSION_OUTPUT,
    },
];

const VERSION_ERROR_INVALID_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.VERSION.INVALID_OPTION",
    identifier: Some("RunMat:version:InvalidOption"),
    when: "The option argument is not the recognized text '-release'.",
    message: "version: option must be '-release'",
};

const VERSION_ERROR_TOO_MANY_INPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.VERSION.TOO_MANY_INPUTS",
    identifier: Some("RunMat:version:TooManyInputs"),
    when: "More than one input argument is provided.",
    message: "version: too many input arguments",
};

const VERSION_ERRORS: [BuiltinErrorDescriptor; 2] =
    [VERSION_ERROR_INVALID_OPTION, VERSION_ERROR_TOO_MANY_INPUTS];

pub const VERSION_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &VERSION_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &VERSION_ERRORS,
};

fn version_error(error: &'static BuiltinErrorDescriptor) -> RuntimeError {
    let mut builder = build_runtime_error(error.message).with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

fn char_row(text: &str) -> Value {
    Value::CharArray(CharArray::new_row(text))
}

pub(crate) fn option_text(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::StringArray(array) if array.data.len() == 1 => Some(array.data[0].clone()),
        Value::CharArray(array) if array.rows == 1 => Some(array.data.iter().collect()),
        _ => None,
    }
}

/// RunMat's honest full version string, e.g. `'0.5.6 (RunMat)'`. The
/// parenthetical marker mirrors the observed MATLAB shape (a version number
/// followed by a parenthesised tag) without reproducing MATLAB's content.
pub(crate) fn runmat_version_string() -> String {
    format!("{RUNMAT_VERSION} (RunMat)")
}

/// Dispatch over the raw argument list. Only the no-argument form and the
/// `'-release'` option are recognized; other options (`'-date'`, `'-java'`,
/// … which are unresolved in the export) raise a stable identifier by
/// documented choice, mirroring `computer`.
fn dispatch_version(args: &[Value]) -> BuiltinResult<Value> {
    if args.len() > 1 {
        return Err(version_error(&VERSION_ERROR_TOO_MANY_INPUTS));
    }
    let Some(option) = args.first() else {
        return Ok(char_row(&runmat_version_string()));
    };
    match option_text(option) {
        Some(text) if text.eq_ignore_ascii_case("-release") => Ok(char_row(RUNMAT_RELEASE)),
        _ => Err(version_error(&VERSION_ERROR_INVALID_OPTION)),
    }
}

fn version_type(_args: &[Type], _context: &ResolveContext) -> Type {
    Type::String
}

async fn version_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    dispatch_version(&args)
}

#[cfg(all(test, feature = "parallel_xval"))]
pub(crate) mod tests {
    use super::*;
    use futures::executor::block_on;

    fn run_version(args: Vec<Value>) -> BuiltinResult<Value> {
        block_on(super::version_builtin(args))
    }

    fn char_row_text(value: &Value) -> String {
        match value {
            Value::CharArray(chars) => {
                assert_eq!(chars.rows, 1, "expected a char row vector");
                chars.data.iter().collect()
            }
            other => panic!("expected char row vector, got {other:?}"),
        }
    }

    // CONTRACT [FR-033-01; version.output-class, version.version-strings]:
    // `version` returns a char row vector holding RunMat's OWN version. The
    // string embeds the actual crate version (`env!("CARGO_PKG_VERSION")`),
    // never MATLAB's observed value. Only the class/shape and the RunMat-value
    // are asserted.
    #[test]
    fn contract_no_arg_returns_runmat_char_version() {
        let value = run_version(Vec::new()).expect("version");
        let text = char_row_text(&value);
        assert!(
            text.contains(RUNMAT_VERSION),
            "version() {text:?} must contain the crate version {RUNMAT_VERSION:?}"
        );
        assert!(
            text.contains("RunMat"),
            "version() {text:?} must self-identify as RunMat"
        );
        // Honesty guard: RunMat MUST NOT emit MATLAB's observed identity.
        assert!(!text.contains("R2026a) Update"));
    }

    // CONTRACT [FR-033-02; version.output-class]: `version('-release')`
    // returns a char row vector (the observed class/shape). The value is
    // RunMat's release tag, a documented choice — NOT MATLAB's '2026a'.
    #[test]
    fn contract_release_option_returns_char_release_tag() {
        let value = run_version(vec![Value::CharArray(CharArray::new_row("-release"))])
            .expect("version('-release')");
        let text = char_row_text(&value);
        assert_eq!(text, RUNMAT_RELEASE);
        assert_ne!(text, "2026a", "must not reproduce MATLAB's release string");
    }

    // documented_choice [FR-033-02]: the option is matched case-insensitively
    // and accepts a string scalar (never observed; RunMat's choice).
    #[test]
    fn documented_choice_release_option_case_insensitive_and_string() {
        let value =
            run_version(vec![Value::String("-RELEASE".to_string())]).expect("version('-RELEASE')");
        assert_eq!(char_row_text(&value), RUNMAT_RELEASE);
    }

    // documented_choice [FR-033-02]: unrecognized options (e.g. '-date',
    // unresolved in the export) are rejected with a stable identifier.
    #[test]
    fn documented_choice_unrecognized_option_errors() {
        let err = run_version(vec![Value::CharArray(CharArray::new_row("-date"))])
            .expect_err("version('-date') should be rejected");
        assert_eq!(err.identifier(), Some("RunMat:version:InvalidOption"));
    }

    // documented_choice [FR-033-02]: non-text option is rejected.
    #[test]
    fn documented_choice_non_text_option_errors() {
        let err = run_version(vec![Value::Num(1.0)]).expect_err("version(1) should be rejected");
        assert_eq!(err.identifier(), Some("RunMat:version:InvalidOption"));
    }

    // documented_choice [FR-033-02]: more than one argument is rejected.
    #[test]
    fn documented_choice_too_many_inputs_errors() {
        let err = run_version(vec![
            Value::String("-release".to_string()),
            Value::String("-release".to_string()),
        ])
        .expect_err("version('-release','-release') should be rejected");
        assert_eq!(err.identifier(), Some("RunMat:version:TooManyInputs"));
    }

    // Registration is discoverable through the builtin registry.
    #[test]
    fn version_is_registered() {
        assert!(runmat_builtins::builtin_function_by_name("version").is_some());
    }

    // Descriptor and acceleration metadata stay consistent.
    #[test]
    fn descriptor_and_spec_metadata_consistent() {
        assert_eq!(VERSION_DESCRIPTOR.signatures.len(), 2);
        assert!(matches!(
            VERSION_DESCRIPTOR.output_mode,
            BuiltinOutputMode::Fixed
        ));
        assert_eq!(VERSION_DESCRIPTOR.errors.len(), 2);
        assert_eq!(GPU_SPEC.name, "version");
        assert_eq!(FUSION_SPEC.name, "version");
    }

    #[test]
    fn version_type_is_string() {
        assert_eq!(
            version_type(&[], &ResolveContext::new(Vec::new())),
            Type::String
        );
    }
}
