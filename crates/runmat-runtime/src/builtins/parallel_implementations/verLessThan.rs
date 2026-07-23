// Parallel clean-room implementation of `verLessThan` (independently developed against the
// matlab-interface-spec black-box specs). 0-6-0 ships the DEFAULT implementation; this copy is
// retained unregistered for differential cross-validation. Original path: crates/runmat-runtime/src/builtins/introspection/verLessThan.rs
//! RunMat-honest `verLessThan` builtin (MATLAB-compatible contract).
//!
//! Clean-room provenance: specs/033-environment-identity (spec `verLessThan`
//! 0.2.0, claims verLessThan.signature-primary, verLessThan.output-class,
//! verLessThan.logical-compare).
//!
//! CONTRACT vs IDENTITY (Constitution Principle II / honesty): the approved
//! export records MATLAB's *own* version-dependent truth —
//! `verLessThan('matlab','99.0')` => `true` and
//! `verLessThan('matlab','1.0')` => `false` (because MATLAB is version 26.x).
//! Those specific truth values are **non-normative** here. RunMat honours only
//! the observable *contract* (output class `logical`, a 1×1 scalar, computed
//! by a correct semantic version comparison) and compares against RunMat's
//! **own** version (`0.5.6`). Consequently `verLessThan('matlab','1.0')`
//! returns `true` for RunMat (0.5.6 < 1.0) — a deliberate divergence from
//! MATLAB's observed `false`, which reflected MATLAB's identity, not RunMat's.

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
use super::version::{option_text, RUNMAT_VERSION};
use crate::{build_runtime_error, BuiltinResult, RuntimeError};

const BUILTIN_NAME: &str = "verLessThan";

pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "verLessThan",
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
    notes: "Runtime identity comparison; no tensor inputs and no GPU execution path.",
};

pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "verLessThan",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Version comparison executed on the host; never fusible.",
};

const VERLESSTHAN_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "tf",
    ty: BuiltinParamType::LogicalArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "True when RunMat's version is older than the given version string.",
}];

const VERLESSTHAN_INPUTS: [BuiltinParamDescriptor; 2] = [
    BuiltinParamDescriptor {
        name: "toolbox",
        ty: BuiltinParamType::StringScalar,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Component name; RunMat maps every name to its own version.",
    },
    BuiltinParamDescriptor {
        name: "version",
        ty: BuiltinParamType::StringScalar,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Dotted version string to compare against.",
    },
];

const VERLESSTHAN_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "tf = verLessThan(toolbox, version)",
    inputs: &VERLESSTHAN_INPUTS,
    outputs: &VERLESSTHAN_OUTPUT,
}];

const VERLESSTHAN_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.VERLESSTHAN.INVALID_INPUT",
    identifier: Some("RunMat:verLessThan:InvalidInput"),
    when: "The toolbox or version argument is missing or not text.",
    message: "verLessThan: expected verLessThan(toolbox, version) with two text arguments",
};

const VERLESSTHAN_ERROR_INVALID_VERSION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.VERLESSTHAN.INVALID_VERSION",
    identifier: Some("RunMat:verLessThan:InvalidVersion"),
    when: "The version argument is not a dotted numeric version string.",
    message: "verLessThan: version must be a dotted numeric string such as '1.2.3'",
};

const VERLESSTHAN_ERRORS: [BuiltinErrorDescriptor; 2] = [
    VERLESSTHAN_ERROR_INVALID_INPUT,
    VERLESSTHAN_ERROR_INVALID_VERSION,
];

pub const VERLESSTHAN_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &VERLESSTHAN_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &VERLESSTHAN_ERRORS,
};

fn verlessthan_error(error: &'static BuiltinErrorDescriptor) -> RuntimeError {
    let mut builder = build_runtime_error(error.message).with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

/// Parse a dotted version string into numeric components. Each component's
/// leading ASCII digits are taken (so `'0.5.6'` and `'1.0'` parse cleanly);
/// a component with no leading digit contributes `0`. Returns `None` when the
/// string is empty or has no digits at all.
fn parse_version(text: &str) -> Option<Vec<u64>> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut components = Vec::new();
    let mut saw_digit = false;
    for part in trimmed.split('.') {
        let digits: String = part.chars().take_while(|c| c.is_ascii_digit()).collect();
        if digits.is_empty() {
            components.push(0);
        } else {
            saw_digit = true;
            components.push(digits.parse::<u64>().unwrap_or(0));
        }
    }
    if saw_digit {
        Some(components)
    } else {
        None
    }
}

/// Semantic "less than" over dotted numeric versions: pads the shorter with
/// zeros and compares component by component. Independent implementation
/// (Constitution Principle III) — no MATLAB internal algorithm is imitated.
pub(crate) fn version_less_than(current: &[u64], target: &[u64]) -> bool {
    let len = current.len().max(target.len());
    for i in 0..len {
        let a = current.get(i).copied().unwrap_or(0);
        let b = target.get(i).copied().unwrap_or(0);
        if a != b {
            return a < b;
        }
    }
    false
}

/// RunMat's own version compared against `target`. The `toolbox` name is a
/// documented mapping: RunMat exposes a single version, so every component
/// name (including 'matlab') maps to RunMat's own version — RunMat reports its
/// honest identity for any queried component.
fn dispatch_verlessthan(args: &[Value]) -> BuiltinResult<Value> {
    if args.len() != 2 {
        return Err(verlessthan_error(&VERLESSTHAN_ERROR_INVALID_INPUT));
    }
    let (Some(_toolbox), Some(target_text)) = (option_text(&args[0]), option_text(&args[1])) else {
        return Err(verlessthan_error(&VERLESSTHAN_ERROR_INVALID_INPUT));
    };
    let Some(target) = parse_version(&target_text) else {
        return Err(verlessthan_error(&VERLESSTHAN_ERROR_INVALID_VERSION));
    };
    let current = parse_version(RUNMAT_VERSION).unwrap_or_default();
    Ok(Value::Bool(version_less_than(&current, &target)))
}

fn verlessthan_type(_args: &[Type], _context: &ResolveContext) -> Type {
    Type::Bool
}

async fn verlessthan_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    dispatch_verlessthan(&args)
}

#[cfg(all(test, feature = "parallel_xval"))]
pub(crate) mod tests {
    use super::*;
    use futures::executor::block_on;
    use runmat_builtins::CharArray;

    fn run_vlt(toolbox: &str, version: &str) -> BuiltinResult<Value> {
        block_on(super::verlessthan_builtin(vec![
            Value::CharArray(CharArray::new_row(toolbox)),
            Value::CharArray(CharArray::new_row(version)),
        ]))
    }

    fn as_bool(value: Value) -> bool {
        match value {
            Value::Bool(b) => b,
            other => panic!("expected logical scalar, got {other:?}"),
        }
    }

    // CONTRACT [FR-033-05; verLessThan.output-class, verLessThan.logical-compare]:
    // returns a 1×1 logical, comparing RunMat's OWN version (0.5.6) to the
    // target. 0.5.6 < 99.0 is true. (This aligns with MATLAB's observed
    // 'older' case only incidentally; the assertion is RunMat's own truth.)
    #[test]
    fn contract_older_than_high_version_is_true() {
        let value = run_vlt("matlab", "99.0").expect("verLessThan");
        assert!(matches!(value, Value::Bool(_)), "class must be logical");
        assert!(as_bool(value));
    }

    // CONTRACT [FR-033-05]: the DIVERGENCE from MATLAB's observed identity.
    // MATLAB returned `false` for verLessThan('matlab','1.0') because MATLAB is
    // 26.x; RunMat is 0.5.6, so 0.5.6 < 1.0 is TRUE. RunMat asserts its own
    // honest result; MATLAB's observed value is NON-NORMATIVE and NOT asserted.
    #[test]
    fn contract_runmat_version_below_one_is_true_diverging_from_matlab() {
        assert!(as_bool(run_vlt("matlab", "1.0").expect("verLessThan")));
    }

    // CONTRACT [FR-033-05]: correct comparisons against RunMat's own 0.5.6.
    #[test]
    fn contract_comparisons_reflect_runmat_version() {
        // Equal → not less than.
        assert!(!as_bool(run_vlt("matlab", RUNMAT_VERSION).expect("eq")));
        // 0.5.6 < 0.0.1 is false.
        assert!(!as_bool(run_vlt("matlab", "0.0.1").expect("low")));
        // 0.5.6 < 0.6 is true (0.5 < 0.6).
        assert!(as_bool(run_vlt("matlab", "0.6").expect("minor")));
        // 0.5.6 < 0.5 is false (0.5.6 > 0.5.0).
        assert!(!as_bool(run_vlt("matlab", "0.5").expect("shorter")));
    }

    // documented_choice: the toolbox name is a documented mapping — every name
    // maps to RunMat's single version, so the result is name-independent.
    #[test]
    fn documented_choice_toolbox_name_maps_to_runmat_version() {
        let a = as_bool(run_vlt("matlab", "0.6").expect("matlab"));
        let b = as_bool(run_vlt("runmat", "0.6").expect("runmat"));
        let c = as_bool(run_vlt("anything", "0.6").expect("anything"));
        assert_eq!(a, b);
        assert_eq!(b, c);
    }

    // Pure-comparator unit checks (Constitution Principle III, independent).
    #[test]
    fn documented_choice_version_less_than_semantics() {
        assert!(version_less_than(&[0, 5, 6], &[99, 0]));
        assert!(version_less_than(&[0, 5, 6], &[1, 0]));
        assert!(!version_less_than(&[0, 5, 6], &[0, 5, 6]));
        assert!(!version_less_than(&[1, 0], &[0, 9, 9]));
        assert!(version_less_than(&[1, 2], &[1, 2, 1]));
    }

    // documented_choice: malformed inputs raise stable identifiers.
    #[test]
    fn documented_choice_invalid_inputs_error() {
        let err = block_on(super::verlessthan_builtin(vec![
            Value::Num(1.0),
            Value::Num(2.0),
        ]))
        .expect_err("non-text");
        assert_eq!(err.identifier(), Some("RunMat:verLessThan:InvalidInput"));
        let err = block_on(super::verlessthan_builtin(vec![Value::CharArray(
            CharArray::new_row("matlab"),
        )]))
        .expect_err("arity");
        assert_eq!(err.identifier(), Some("RunMat:verLessThan:InvalidInput"));
        let err = run_vlt("matlab", "not-a-version").expect_err("bad version");
        assert_eq!(err.identifier(), Some("RunMat:verLessThan:InvalidVersion"));
    }

    #[test]
    fn verlessthan_is_registered() {
        assert!(runmat_builtins::builtin_function_by_name("verLessThan").is_some());
    }

    #[test]
    fn descriptor_and_spec_metadata_consistent() {
        assert_eq!(VERLESSTHAN_DESCRIPTOR.signatures.len(), 1);
        assert!(matches!(
            VERLESSTHAN_DESCRIPTOR.output_mode,
            BuiltinOutputMode::Fixed
        ));
        assert_eq!(VERLESSTHAN_DESCRIPTOR.errors.len(), 2);
        assert_eq!(GPU_SPEC.name, "verLessThan");
        assert_eq!(FUSION_SPEC.name, "verLessThan");
    }

    #[test]
    fn verlessthan_type_is_bool() {
        assert_eq!(
            verlessthan_type(&[], &ResolveContext::new(Vec::new())),
            Type::Bool
        );
    }
}
