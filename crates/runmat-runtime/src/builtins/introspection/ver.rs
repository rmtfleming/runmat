//! RunMat-honest `ver` builtin (MATLAB-compatible contract).
//!
//! Clean-room provenance: specs/033-environment-identity (spec `ver` 0.2.0,
//! claims ver.signature-primary, ver.output-class, ver.product-struct).
//!
//! CONTRACT vs IDENTITY (Constitution Principle II / honesty): the approved
//! export records that MATLAB's `ver('MATLAB')` returns a 1×1 `struct`
//! describing the MATLAB product; the field names and contents are explicitly
//! install-dependent and unresolved (ver.q-fields, ver.q-install-dependent).
//! RunMat honours only the observable *contract* (output class `struct`) and
//! populates it with RunMat's **own** identity. It does NOT fabricate a MATLAB
//! product entry: `ver('MATLAB')` returns an empty struct array by documented
//! choice, because RunMat is not MATLAB.

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    CellArray, CharArray, ResolveContext, StructValue, Type, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ShapeRequirements,
};
use crate::builtins::parallel_implementations::version::{option_text, RUNMAT_RELEASE, RUNMAT_VERSION};
use crate::{build_runtime_error, BuiltinResult, RuntimeError};

const BUILTIN_NAME: &str = "ver";

/// RunMat product name reported in the `Name` field of the `ver` struct.
const RUNMAT_NAME: &str = "RunMat";

/// Documented release date for the `Date` field of the `ver` struct — the
/// actual v0.5.6 release date from RunMat's own history (a documented choice;
/// RunMat does not expose a per-product build timestamp). This is RunMat's own
/// datum, not a MATLAB build date.
const RUNMAT_DATE: &str = "2026-06-28";

#[runmat_macros::register_gpu_spec(builtin_path = "crate::builtins::introspection::ver")]
pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "ver",
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

#[runmat_macros::register_fusion_spec(builtin_path = "crate::builtins::introspection::ver")]
pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "ver",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Identity query executed on the host; never fusible.",
};

const VER_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "s",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Struct describing RunMat (fields Name, Version, Release, Date).",
}];

const VER_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "product",
    ty: BuiltinParamType::StringScalar,
    arity: BuiltinParamArity::Optional,
    default: None,
    description: "Optional product name; 'RunMat' returns the RunMat entry.",
}];

const VER_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "s = ver",
        inputs: &[],
        outputs: &VER_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "s = ver(product)",
        inputs: &VER_INPUTS,
        outputs: &VER_OUTPUT,
    },
];

const VER_ERROR_TOO_MANY_INPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.VER.TOO_MANY_INPUTS",
    identifier: Some("RunMat:ver:TooManyInputs"),
    when: "More than one input argument is provided.",
    message: "ver: too many input arguments",
};

const VER_ERROR_INVALID_PRODUCT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.VER.INVALID_PRODUCT",
    identifier: Some("RunMat:ver:InvalidProduct"),
    when: "The product argument is not text.",
    message: "ver: product name must be a char row vector or string scalar",
};

const VER_ERRORS: [BuiltinErrorDescriptor; 2] =
    [VER_ERROR_TOO_MANY_INPUTS, VER_ERROR_INVALID_PRODUCT];

pub const VER_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &VER_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &VER_ERRORS,
};

fn ver_error(error: &'static BuiltinErrorDescriptor) -> RuntimeError {
    let mut builder = build_runtime_error(error.message).with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

fn char_row(text: &str) -> Value {
    Value::CharArray(CharArray::new_row(text))
}

/// Build the 1×1 struct describing RunMat itself (the observed contract class
/// is `struct`). Every field value is RunMat's own honest identity.
pub(crate) fn runmat_ver_struct() -> Value {
    let mut st = StructValue::new();
    st.insert("Name", char_row(RUNMAT_NAME));
    st.insert("Version", char_row(RUNMAT_VERSION));
    st.insert("Release", char_row(RUNMAT_RELEASE));
    st.insert("Date", char_row(RUNMAT_DATE));
    Value::Struct(st)
}

/// An empty struct array (RunMat's representation is an empty 0×1 cell, as
/// `dir` uses for "no matches"). Returned for products RunMat does not
/// provide — including `ver('MATLAB')` — rather than fabricating an entry.
fn empty_struct_array() -> BuiltinResult<Value> {
    CellArray::new(Vec::new(), 0, 1)
        .map(Value::Cell)
        .map_err(|e| {
            build_runtime_error(format!("ver: {e}"))
                .with_builtin(BUILTIN_NAME)
                .build()
        })
}

fn dispatch_ver(args: &[Value]) -> BuiltinResult<Value> {
    if args.len() > 1 {
        return Err(ver_error(&VER_ERROR_TOO_MANY_INPUTS));
    }
    let Some(product) = args.first() else {
        // No argument: describe RunMat itself.
        return Ok(runmat_ver_struct());
    };
    match option_text(product) {
        Some(name) if name.eq_ignore_ascii_case(RUNMAT_NAME) => Ok(runmat_ver_struct()),
        // Any other product (including 'MATLAB'): RunMat does not provide it,
        // so return an empty struct array by documented choice.
        Some(_) => empty_struct_array(),
        None => Err(ver_error(&VER_ERROR_INVALID_PRODUCT)),
    }
}

fn ver_type(_args: &[Type], _context: &ResolveContext) -> Type {
    Type::Struct { known_fields: None }
}

#[runtime_builtin(
    name = "ver",
    category = "introspection",
    summary = "Return a struct describing the RunMat runtime (Name, Version, Release, Date).",
    keywords = "ver,version,product,runmat,identity,toolbox",
    examples = "s = ver(); s = ver('RunMat');",
    type_resolver(ver_type),
    descriptor(crate::builtins::introspection::ver::VER_DESCRIPTOR),
    builtin_path = "crate::builtins::introspection::ver"
)]
async fn ver_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    dispatch_ver(&args)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use futures::executor::block_on;

    fn run_ver(args: Vec<Value>) -> BuiltinResult<Value> {
        block_on(super::ver_builtin(args))
    }

    fn field_char(st: &StructValue, name: &str) -> String {
        match st.fields.get(name) {
            Some(Value::CharArray(chars)) => chars.data.iter().collect(),
            other => panic!("field {name:?} not a char row: {other:?}"),
        }
    }

    // CONTRACT [FR-033-03; ver.output-class, ver.product-struct]: the
    // no-argument form returns class `struct` (a 1×1 struct — the observed
    // contract). The fields are RunMat's own identity (field names/contents
    // are unresolved in the export, so they are RunMat's documented choice).
    #[test]
    fn contract_no_arg_returns_runmat_struct() {
        let value = run_ver(Vec::new()).expect("ver");
        let Value::Struct(st) = value else {
            panic!("ver() must return a struct");
        };
        assert_eq!(field_char(&st, "Name"), "RunMat");
        assert_eq!(field_char(&st, "Version"), RUNMAT_VERSION);
        assert_eq!(field_char(&st, "Release"), RUNMAT_RELEASE);
        assert_eq!(field_char(&st, "Date"), RUNMAT_DATE);
    }

    // CONTRACT [FR-033-03]: ver('RunMat') returns the same RunMat struct;
    // asserts class `struct` with RunMat's honest Version, never MATLAB's.
    #[test]
    fn contract_runmat_product_returns_struct() {
        let value = run_ver(vec![Value::CharArray(CharArray::new_row("RunMat"))]).expect("ver");
        let Value::Struct(st) = value else {
            panic!("ver('RunMat') must return a struct");
        };
        assert_eq!(field_char(&st, "Version"), RUNMAT_VERSION);
        assert_ne!(
            field_char(&st, "Version"),
            "26.1.0.3276743 (R2026a) Update 3"
        );
    }

    // documented_choice [FR-033-04]: RunMat is not MATLAB, so ver('MATLAB')
    // yields an empty struct array (0×1 cell) rather than a fabricated MATLAB
    // product entry. The MATLAB-value claim is deliberately NOT asserted.
    #[test]
    fn documented_choice_matlab_product_is_empty_struct_array() {
        let value = run_ver(vec![Value::CharArray(CharArray::new_row("MATLAB"))]).expect("ver");
        match value {
            Value::Cell(cell) => assert_eq!(cell.data.len(), 0),
            other => panic!("expected empty struct array, got {other:?}"),
        }
    }

    // documented_choice [FR-033-04]: an unknown product is likewise an empty
    // struct array; product matching is case-insensitive.
    #[test]
    fn documented_choice_unknown_product_is_empty_and_case_insensitive() {
        let unknown = run_ver(vec![Value::String("NoSuchToolbox".to_string())]).expect("ver");
        assert!(matches!(unknown, Value::Cell(cell) if cell.data.is_empty()));
        let lower = run_ver(vec![Value::String("runmat".to_string())]).expect("ver");
        assert!(matches!(lower, Value::Struct(_)));
    }

    // documented_choice: non-text product and too-many-args errors.
    #[test]
    fn documented_choice_invalid_inputs_error() {
        let err = run_ver(vec![Value::Num(1.0)]).expect_err("ver(1) should be rejected");
        assert_eq!(err.identifier(), Some("RunMat:ver:InvalidProduct"));
        let err = run_ver(vec![
            Value::String("RunMat".to_string()),
            Value::String("RunMat".to_string()),
        ])
        .expect_err("too many inputs");
        assert_eq!(err.identifier(), Some("RunMat:ver:TooManyInputs"));
    }

    #[test]
    fn ver_is_registered() {
        assert!(runmat_builtins::builtin_function_by_name("ver").is_some());
    }

    #[test]
    fn descriptor_and_spec_metadata_consistent() {
        assert_eq!(VER_DESCRIPTOR.signatures.len(), 2);
        assert!(matches!(
            VER_DESCRIPTOR.output_mode,
            BuiltinOutputMode::Fixed
        ));
        assert_eq!(VER_DESCRIPTOR.errors.len(), 2);
        assert_eq!(GPU_SPEC.name, "ver");
        assert_eq!(FUSION_SPEC.name, "ver");
    }

    #[test]
    fn ver_type_is_struct() {
        assert!(matches!(
            ver_type(&[], &ResolveContext::new(Vec::new())),
            Type::Struct { .. }
        ));
    }
}
