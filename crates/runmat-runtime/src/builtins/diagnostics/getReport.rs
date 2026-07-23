//! MATLAB-compatible `getReport` builtin for RunMat.
//!
//! Clean-room provenance: specs/031-error-report (spec `getReport` 0.2.0,
//! claims getReport.signature-primary, getReport.output-class,
//! getReport.char-report).
//!
//! Observed behaviour (normative, captured on R2026a / GLNXA64):
//! `getReport(MException('x:y','oops'))` => `'oops'` (1×4 char). The observed
//! report of a plain `MException` is exactly its message text, returned as a
//! char row vector. The fuller MATLAB report — identifier formatting, a stack
//! trace, and the `'extended'`/`'basic'`/`hyperlinks` options — is unresolved
//! in the approved export (`getReport.q-options`) and is NOT implemented: the
//! minimal observed behaviour is message-only (see specs/031-error-report).

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

const BUILTIN_NAME: &str = "getReport";

#[runmat_macros::register_gpu_spec(builtin_path = "crate::builtins::diagnostics::get_report")]
pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "getReport",
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
    notes: "Reads text off an MException value; no tensor inputs and no GPU execution path.",
};

#[runmat_macros::register_fusion_spec(builtin_path = "crate::builtins::diagnostics::get_report")]
pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "getReport",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Exception-report query executed on the host; never fusible.",
};

const GET_REPORT_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "report",
    ty: BuiltinParamType::StringScalar,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Char report text describing the exception.",
}];

const GET_REPORT_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "ME",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "An MException object.",
}];

const GET_REPORT_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "report = getReport(ME)",
    inputs: &GET_REPORT_INPUTS,
    outputs: &GET_REPORT_OUTPUT,
}];

const GET_REPORT_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.GETREPORT.INVALID_INPUT",
    identifier: Some("RunMat:getReport:InvalidInput"),
    when: "The input argument is not an MException object.",
    message: "getReport: input must be an MException object",
};

const GET_REPORT_ERRORS: [BuiltinErrorDescriptor; 1] = [GET_REPORT_ERROR_INVALID_INPUT];

pub const GET_REPORT_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &GET_REPORT_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &GET_REPORT_ERRORS,
};

fn get_report_error(error: &'static BuiltinErrorDescriptor) -> RuntimeError {
    let mut builder = build_runtime_error(error.message).with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

fn char_row(text: &str) -> Value {
    Value::CharArray(CharArray::new_row(text))
}

fn get_report_type(_args: &[Type], _context: &ResolveContext) -> Type {
    Type::String
}

/// Return the exception's report text.
///
/// The observed report of a plain `MException` is exactly its message text
/// [getReport.char-report], returned as a char row vector
/// [getReport.output-class]. Identifier formatting, stack-trace rendering, and
/// the `'extended'`/`'basic'`/hyperlink options are unresolved in the approved
/// export (`getReport.q-options`); the message-only report is the minimal
/// observed behaviour and is applied to every `MException` by documented
/// choice (FR-031-03). A non-`MException` input is rejected with a stable
/// identifier — an unobserved error path, documented choice (FR-031-04).
async fn get_report_builtin(me: Value) -> BuiltinResult<Value> {
    match me {
        Value::MException(mex) => Ok(char_row(&mex.message)),
        _ => Err(get_report_error(&GET_REPORT_ERROR_INVALID_INPUT)),
    }
}

#[runtime_builtin(
    name = "getReport",
    category = "diagnostics",
    summary = "Return the char report text of an MException object.",
    keywords = "getReport,MException,exception,error,report,message",
    examples = "report = getReport(ME);",
    type_resolver(get_report_type),
    descriptor(crate::builtins::diagnostics::get_report::GET_REPORT_DESCRIPTOR),
    builtin_path = "crate::builtins::diagnostics::get_report"
)]
async fn get_report_builtin_registered(me: Value) -> BuiltinResult<Value> {
    get_report_builtin(me).await
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use futures::executor::block_on;
    use runmat_builtins::MException;

    fn run_get_report(me: Value) -> BuiltinResult<Value> {
        block_on(super::get_report_builtin(me))
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

    fn exception(identifier: &str, message: &str) -> Value {
        Value::MException(MException::new(identifier.to_string(), message.to_string()))
    }

    // Normative [FR-031-01, FR-031-02; getReport.signature-primary,
    // getReport.char-report, getReport.output-class] — observed case `basic`:
    // getReport(MException('x:y','oops')) => 'oops', class char, size 1×4.
    #[test]
    fn observed_message_report_is_char_row() {
        let value = run_get_report(exception("x:y", "oops")).expect("getReport");
        assert_char_row(&value, "oops");
    }

    // Normative [FR-031-02; getReport.output-class] — the report is a char row
    // vector; its length tracks the message text.
    #[test]
    fn observed_output_class_char_row() {
        let value = run_get_report(exception("pkg:id", "something failed")).expect("getReport");
        assert_char_row(&value, "something failed");
    }

    // unresolved_choice [FR-031-03; getReport.q-options] — the observed report
    // is message-only. The identifier is NOT included, and a populated stack is
    // NOT rendered (stack-trace / identifier formatting and the
    // 'extended'/'basic'/hyperlink options are unobserved; documented choice to
    // return the message text alone).
    #[test]
    fn unresolved_choice_report_is_message_only() {
        let mut mex = MException::new("some:identifier".to_string(), "boom".to_string());
        mex.stack.push("frame_a".to_string());
        mex.stack.push("frame_b".to_string());
        let value = run_get_report(Value::MException(mex)).expect("getReport");
        assert_char_row(&value, "boom");
    }

    // unresolved_choice [FR-031-03] — an empty message yields an empty (1×0)
    // char row; unobserved input, documented choice.
    #[test]
    fn unresolved_choice_empty_message() {
        let value = run_get_report(exception("x:y", "")).expect("getReport");
        assert_char_row(&value, "");
    }

    // unresolved_choice [FR-031-04] — a non-MException input is rejected with a
    // stable identifier; the error path is unobserved (documented choice).
    #[test]
    fn unresolved_choice_non_mexception_input_errors() {
        let err = run_get_report(Value::Num(1.0)).expect_err("non-MException should be rejected");
        assert_eq!(err.identifier(), Some("RunMat:getReport:InvalidInput"));
    }

    // Registration is discoverable through the builtin registry (SC-031-2).
    #[test]
    fn get_report_is_registered() {
        assert!(runmat_builtins::builtin_function_by_name("getReport").is_some());
    }

    // Type resolver reports the char/string producer type (SC-031-2).
    #[test]
    fn get_report_type_is_string() {
        assert_eq!(
            get_report_type(&[], &ResolveContext::new(Vec::new())),
            Type::String
        );
    }

    // Descriptor and acceleration metadata stay consistent [FR-031-05: Fixed
    // single output; extended-format options omitted per getReport.q-options].
    #[test]
    fn descriptor_and_spec_metadata_consistent() {
        assert_eq!(GET_REPORT_DESCRIPTOR.signatures.len(), 1);
        assert!(matches!(
            GET_REPORT_DESCRIPTOR.output_mode,
            BuiltinOutputMode::Fixed
        ));
        assert_eq!(GET_REPORT_DESCRIPTOR.errors.len(), 1);
        assert_eq!(GPU_SPEC.name, "getReport");
        assert_eq!(FUSION_SPEC.name, "getReport");
    }
}
