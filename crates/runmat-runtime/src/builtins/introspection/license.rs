//! RunMat-honest `license` builtin (MATLAB-compatible contract).
//!
//! Clean-room provenance: specs/033-environment-identity (spec `license`
//! 0.2.0, claims license.signature-primary, license.output-class,
//! license.test-numeric).
//!
//! CONTRACT vs IDENTITY (Constitution Principle II / honesty): the approved
//! export records that MATLAB's `license('test', feature)` returns the double
//! `1` when the queried feature is licensed on that MATLAB installation. That
//! value is MATLAB's *own* licensing state. RunMat honours the observable
//! *contract* (output class `double`, a 1×1 scalar; `1` == "available") with
//! RunMat's **own** honest truth: RunMat is open-source with no license
//! manager, so every feature is unconditionally available. `1` here is
//! RunMat's factual state, not a copy of MATLAB's licensing.

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    CellArray, CharArray, ResolveContext, Type, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ShapeRequirements,
};
use crate::builtins::parallel_implementations::version::option_text;
use crate::{build_runtime_error, BuiltinResult, RuntimeError};

const BUILTIN_NAME: &str = "license";

/// Documented sentinel returned by the no-argument `license` form (MATLAB's
/// license-number form is install-specific and unresolved in the export).
/// RunMat has no license number, so it returns its own identity string.
const RUNMAT_LICENSE_ID: &str = "RunMat";

#[runmat_macros::register_gpu_spec(builtin_path = "crate::builtins::introspection::license")]
pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "license",
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
    notes: "Runtime licensing query; no tensor inputs and no GPU execution path.",
};

#[runmat_macros::register_fusion_spec(builtin_path = "crate::builtins::introspection::license")]
pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "license",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Licensing query executed on the host; never fusible.",
};

const LICENSE_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "result",
    ty: BuiltinParamType::NumericScalar,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Double 1 when the queried feature is available (always, in RunMat).",
}];

const LICENSE_TEST_INPUTS: [BuiltinParamDescriptor; 2] = [
    BuiltinParamDescriptor {
        name: "action",
        ty: BuiltinParamType::StringScalar,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "The action, e.g. 'test'.",
    },
    BuiltinParamDescriptor {
        name: "feature",
        ty: BuiltinParamType::StringScalar,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Feature/product name.",
    },
];

const LICENSE_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "result = license('test', feature)",
        inputs: &LICENSE_TEST_INPUTS,
        outputs: &LICENSE_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "s = license",
        inputs: &[],
        outputs: &LICENSE_OUTPUT,
    },
];

const LICENSE_ERROR_INVALID_USAGE: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.LICENSE.INVALID_USAGE",
    identifier: Some("RunMat:license:InvalidUsage"),
    when: "The action/feature arguments do not match a supported license form.",
    message: "license: unsupported arguments; expected license, license('inuse'), or license('test'|'checkout'|'checkin', feature)",
};

const LICENSE_ERRORS: [BuiltinErrorDescriptor; 1] = [LICENSE_ERROR_INVALID_USAGE];

pub const LICENSE_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &LICENSE_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &LICENSE_ERRORS,
};

fn license_error(error: &'static BuiltinErrorDescriptor) -> RuntimeError {
    let mut builder = build_runtime_error(error.message).with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

fn char_row(text: &str) -> Value {
    Value::CharArray(CharArray::new_row(text))
}

/// Empty struct array (0×1 cell) — RunMat's "nothing checked out" result for
/// `license('inuse')`, since there is no license manager.
fn empty_struct_array() -> BuiltinResult<Value> {
    CellArray::new(Vec::new(), 0, 1)
        .map(Value::Cell)
        .map_err(|e| {
            build_runtime_error(format!("license: {e}"))
                .with_builtin(BUILTIN_NAME)
                .build()
        })
}

fn dispatch_license(args: &[Value]) -> BuiltinResult<Value> {
    match args {
        // No-argument form: RunMat's license-number stand-in (documented).
        [] => Ok(char_row(RUNMAT_LICENSE_ID)),
        // Single action.
        [action] => match option_text(action) {
            Some(text) if text.eq_ignore_ascii_case("inuse") => empty_struct_array(),
            _ => Err(license_error(&LICENSE_ERROR_INVALID_USAGE)),
        },
        // Action + feature.
        [action, feature] => {
            let (Some(action), Some(_feature)) = (option_text(action), option_text(feature)) else {
                return Err(license_error(&LICENSE_ERROR_INVALID_USAGE));
            };
            // 'test' is the observed contract form (double 1 == available).
            // 'checkout'/'checkin' are documented choices that always succeed
            // because RunMat is unlicensed/open-source.
            if action.eq_ignore_ascii_case("test")
                || action.eq_ignore_ascii_case("checkout")
                || action.eq_ignore_ascii_case("checkin")
            {
                Ok(Value::Num(1.0))
            } else {
                Err(license_error(&LICENSE_ERROR_INVALID_USAGE))
            }
        }
        _ => Err(license_error(&LICENSE_ERROR_INVALID_USAGE)),
    }
}

fn license_type(_args: &[Type], _context: &ResolveContext) -> Type {
    Type::Num
}

#[runtime_builtin(
    name = "license",
    category = "introspection",
    summary = "Query RunMat licensing; features are always available (open-source).",
    keywords = "license,feature,test,available,runmat,identity",
    examples = "tf = license('test', 'MATLAB');",
    type_resolver(license_type),
    descriptor(crate::builtins::introspection::license::LICENSE_DESCRIPTOR),
    builtin_path = "crate::builtins::introspection::license"
)]
async fn license_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    dispatch_license(&args)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use futures::executor::block_on;

    fn run_license(args: Vec<Value>) -> BuiltinResult<Value> {
        block_on(super::license_builtin(args))
    }

    fn test_feature(feature: &str) -> BuiltinResult<Value> {
        run_license(vec![
            Value::CharArray(CharArray::new_row("test")),
            Value::CharArray(CharArray::new_row(feature)),
        ])
    }

    // CONTRACT [FR-033-06; license.output-class, license.test-numeric]:
    // license('test', feature) returns the double 1 (class double, 1×1). This
    // is RunMat's own honest state: open-source, no license manager, every
    // feature available. The value 1 matches the observed shape/class contract
    // and RunMat's factual availability, not a copy of MATLAB's licensing.
    #[test]
    fn contract_test_returns_double_one() {
        assert_eq!(test_feature("MATLAB").expect("license"), Value::Num(1.0));
    }

    // CONTRACT [FR-033-06]: availability is unconditional — any feature name
    // (RunMat has no per-feature licensing) yields double 1.
    #[test]
    fn contract_test_any_feature_is_available() {
        assert_eq!(
            test_feature("Symbolic_Toolbox").expect("l"),
            Value::Num(1.0)
        );
        assert_eq!(test_feature("nonexistent").expect("l"), Value::Num(1.0));
    }

    // documented_choice [FR-033-07]: no-argument form returns RunMat's own
    // license-number stand-in (a char row), not MATLAB's install number.
    #[test]
    fn documented_choice_no_arg_returns_runmat_identity() {
        let value = run_license(Vec::new()).expect("license");
        match value {
            Value::CharArray(chars) => {
                assert_eq!(chars.data.iter().collect::<String>(), "RunMat");
                assert_eq!(chars.rows, 1);
            }
            other => panic!("expected char row, got {other:?}"),
        }
    }

    // documented_choice [FR-033-07]: license('inuse') → empty struct array,
    // because RunMat checks out nothing.
    #[test]
    fn documented_choice_inuse_is_empty_struct_array() {
        let value =
            run_license(vec![Value::CharArray(CharArray::new_row("inuse"))]).expect("license");
        assert!(matches!(value, Value::Cell(cell) if cell.data.is_empty()));
    }

    // documented_choice [FR-033-07]: checkout/checkin always succeed (double 1)
    // because RunMat is open-source.
    #[test]
    fn documented_choice_checkout_checkin_succeed() {
        let checkout = run_license(vec![
            Value::CharArray(CharArray::new_row("checkout")),
            Value::CharArray(CharArray::new_row("MATLAB")),
        ])
        .expect("checkout");
        assert_eq!(checkout, Value::Num(1.0));
    }

    // documented_choice: unsupported usage raises a stable identifier.
    #[test]
    fn documented_choice_invalid_usage_errors() {
        let err = run_license(vec![Value::CharArray(CharArray::new_row("bogus"))])
            .expect_err("bad single action");
        assert_eq!(err.identifier(), Some("RunMat:license:InvalidUsage"));
        let err = run_license(vec![
            Value::CharArray(CharArray::new_row("frobnicate")),
            Value::CharArray(CharArray::new_row("MATLAB")),
        ])
        .expect_err("bad action");
        assert_eq!(err.identifier(), Some("RunMat:license:InvalidUsage"));
        let err = run_license(vec![Value::Num(1.0), Value::Num(2.0)]).expect_err("non-text");
        assert_eq!(err.identifier(), Some("RunMat:license:InvalidUsage"));
    }

    #[test]
    fn license_is_registered() {
        assert!(runmat_builtins::builtin_function_by_name("license").is_some());
    }

    #[test]
    fn descriptor_and_spec_metadata_consistent() {
        assert_eq!(LICENSE_DESCRIPTOR.signatures.len(), 2);
        assert!(matches!(
            LICENSE_DESCRIPTOR.output_mode,
            BuiltinOutputMode::Fixed
        ));
        assert_eq!(LICENSE_DESCRIPTOR.errors.len(), 1);
        assert_eq!(GPU_SPEC.name, "license");
        assert_eq!(FUSION_SPEC.name, "license");
    }

    #[test]
    fn license_type_is_num() {
        assert_eq!(
            license_type(&[], &ResolveContext::new(Vec::new())),
            Type::Num
        );
    }
}
