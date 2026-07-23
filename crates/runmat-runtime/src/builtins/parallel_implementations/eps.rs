// Parallel clean-room implementation of `eps` as a FUNCTION (spec 028).
// 0-6-0 provides `eps` as a primitive CONSTANT (the default); this function form is retained
// unregistered for cross-validation. Original path: math/elementwise/eps.rs
//! MATLAB-compatible `eps` builtin for RunMat.
//!
//! Clean-room provenance: specs/028-eps-function (spec `eps` 0.2.0, claims
//! eps.signature-primary, eps.spacing-constants). `eps` was previously a
//! runtime constant (`Value::Num(f64::EPSILON)`); this feature replaces that
//! constant with a zero-or-one argument function so that `eps('single')` (and
//! the documented `eps(x)` spacing form) can be expressed. Bare `eps`
//! continues to evaluate because the HIR lowering resolves a bare identifier
//! that is not a variable/constant to a zero-argument builtin call (see
//! `runmat-hir` `lower_expr_semantic`, `is_builtin` path).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    NumericDType, ResolveContext, Tensor, Type, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::random_args::keyword_of;
use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ScalarType, ShapeRequirements,
};
use crate::builtins::common::tensor;
use crate::builtins::math::type_resolvers::numeric_unary_type;
use crate::{build_runtime_error, BuiltinResult, RuntimeError};

pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "eps",
    op_kind: GpuOpKind::Custom("metadata"),
    supported_precisions: &[ScalarType::F32, ScalarType::F64],
    broadcast: BroadcastSemantics::None,
    provider_hooks: &[],
    constant_strategy: ConstantStrategy::InlineLiteral,
    residency: ResidencyPolicy::GatherImmediately,
    nan_mode: ReductionNaN::Include,
    two_pass_threshold: None,
    workgroup_size: None,
    accepts_nan_mode: false,
    notes: "Host-only floating-point spacing query; no GPU provider hooks. gpuArray inputs are rejected rather than gathered.",
};

pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "eps",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Spacing query executes on the host; planners treat it as a scalar/metadata query.",
};

const BUILTIN_NAME: &str = "eps";

const EPS_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "d",
    ty: BuiltinParamType::NumericArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Floating-point spacing (relative accuracy).",
}];

const EPS_INPUTS_NONE: [BuiltinParamDescriptor; 0] = [];

const EPS_INPUTS_X: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "x",
    ty: BuiltinParamType::NumericArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Floating-point value or array whose spacing is measured.",
}];

const EPS_INPUTS_CLASSNAME: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "classname",
    ty: BuiltinParamType::StringScalar,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Floating-point class name, 'double' or 'single'.",
}];

const EPS_SIGNATURES: [BuiltinSignatureDescriptor; 3] = [
    BuiltinSignatureDescriptor {
        label: "d = eps",
        inputs: &EPS_INPUTS_NONE,
        outputs: &EPS_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "d = eps(x)",
        inputs: &EPS_INPUTS_X,
        outputs: &EPS_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "d = eps(classname)",
        inputs: &EPS_INPUTS_CLASSNAME,
        outputs: &EPS_OUTPUT,
    },
];

const EPS_ERROR_TOO_MANY_INPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.EPS.TOO_MANY_INPUTS",
    identifier: Some("RunMat:eps:TooManyInputs"),
    when: "More than one positional argument is provided.",
    message: "eps: too many input arguments",
};

const EPS_ERROR_INVALID_CLASS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.EPS.INVALID_CLASS",
    identifier: Some("RunMat:eps:InvalidClass"),
    when: "A class name other than 'double' or 'single' is supplied.",
    message: "eps: class must be 'double' or 'single'",
};

const EPS_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.EPS.INVALID_INPUT",
    identifier: Some("RunMat:eps:InvalidInput"),
    when: "The argument is not a floating-point value or a supported class name.",
    message: "eps: invalid input",
};

const EPS_ERRORS: [BuiltinErrorDescriptor; 3] = [
    EPS_ERROR_TOO_MANY_INPUTS,
    EPS_ERROR_INVALID_CLASS,
    EPS_ERROR_INVALID_INPUT,
];

pub const EPS_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &EPS_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &EPS_ERRORS,
};

fn eps_error_with_detail(
    error: &'static BuiltinErrorDescriptor,
    detail: impl std::fmt::Display,
) -> RuntimeError {
    let mut builder =
        build_runtime_error(format!("{}: {}", error.message, detail)).with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

fn eps_error(error: &'static BuiltinErrorDescriptor) -> RuntimeError {
    let mut builder = build_runtime_error(error.message).with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

fn eps_type(args: &[Type], context: &ResolveContext) -> Type {
    if args.is_empty() {
        // Bare `eps` reproduces the previous constant's double scalar type.
        Type::Num
    } else {
        numeric_unary_type(args, context)
    }
}

async fn eps_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    match args.len() {
        // Observed: bare `eps` is the double spacing near 1.0.
        0 => Ok(Value::Num(f64::EPSILON)),
        1 => {
            let arg = &args[0];
            if let Some(classname) = keyword_of(arg) {
                eps_for_classname(&classname)
            } else {
                eps_for_value(arg)
            }
        }
        _ => Err(eps_error(&EPS_ERROR_TOO_MANY_INPUTS)),
    }
}

/// `eps('double')` / `eps('single')`. The `'single'` result is returned as a
/// 1x1 single-class tensor so `class(eps('single'))` reports `single`
/// (`Value::Num` would report `double`, and a collapsed 1-element tensor would
/// too — the spacing constant must keep its `NumericDType::F32` tag).
fn eps_for_classname(classname: &str) -> BuiltinResult<Value> {
    match classname {
        // Observed exact: eps('single') == f32::EPSILON == 1.1920928955078125e-07.
        "single" => single_scalar(f32::EPSILON as f64),
        // Documented choice: eps('double') matches bare `eps`.
        "double" => Ok(Value::Num(f64::EPSILON)),
        other => Err(eps_error_with_detail(&EPS_ERROR_INVALID_CLASS, other)),
    }
}

/// `eps(x)` for numeric `x` (documented choice; the value form is unobserved
/// in the approved export). Returns the positive distance from `|x|` to the
/// next larger floating-point number of the same class, computed with the
/// IEEE `nextUp` operation elementwise. Class and shape follow the input.
fn eps_for_value(value: &Value) -> BuiltinResult<Value> {
    match value {
        Value::Num(n) => Ok(Value::Num(eps_ulp_f64(*n))),
        Value::Complex(re, im) => Ok(Value::Num(eps_ulp_f64(re.hypot(*im)))),
        Value::Tensor(t) => match t.dtype {
            NumericDType::F64 => {
                let data: Vec<f64> = t.data.iter().map(|&v| eps_ulp_f64(v)).collect();
                let out = Tensor::new(data, t.shape.clone())
                    .map_err(|e| eps_error_with_detail(&EPS_ERROR_INVALID_INPUT, e))?;
                Ok(tensor::tensor_into_value(out))
            }
            NumericDType::F32 => {
                let data: Vec<f64> = t.data.iter().map(|&v| eps_ulp_f32(v)).collect();
                // Keep as a tensor (never collapse) so the single class survives.
                let out = Tensor::new_with_dtype(data, t.shape.clone(), NumericDType::F32)
                    .map_err(|e| eps_error_with_detail(&EPS_ERROR_INVALID_INPUT, e))?;
                Ok(Value::Tensor(out))
            }
            NumericDType::U8 | NumericDType::U16 | NumericDType::U32 => Err(eps_error_with_detail(
                &EPS_ERROR_INVALID_INPUT,
                format!(
                    "input class '{}' is not floating-point",
                    t.dtype.class_name()
                ),
            )),
        },
        Value::ComplexTensor(ct) => {
            let data: Vec<f64> = ct
                .data
                .iter()
                .map(|&(re, im)| eps_ulp_f64(re.hypot(im)))
                .collect();
            let out = Tensor::new(data, ct.shape.clone())
                .map_err(|e| eps_error_with_detail(&EPS_ERROR_INVALID_INPUT, e))?;
            Ok(tensor::tensor_into_value(out))
        }
        Value::GpuTensor(_) => Err(eps_error_with_detail(
            &EPS_ERROR_INVALID_INPUT,
            "gpuArray input is not supported",
        )),
        other => Err(eps_error_with_detail(
            &EPS_ERROR_INVALID_INPUT,
            format!("expected a single or double value, got {other:?}"),
        )),
    }
}

fn single_scalar(value: f64) -> BuiltinResult<Value> {
    let tensor = Tensor::from_f32(vec![value as f32], vec![1, 1])
        .map_err(|e| eps_error_with_detail(&EPS_ERROR_INVALID_INPUT, e))?;
    Ok(Value::Tensor(tensor))
}

/// Double-precision spacing at `x`: `nextUp(|x|) - |x|`.
///
/// Edge cases follow IEEE `nextUp` directly (documented choice): `eps(0)` is
/// the smallest positive subnormal, `eps(realmax)` and `eps(Inf)` are `Inf`
/// and `NaN` respectively, and `eps(NaN)` is `NaN`.
fn eps_ulp_f64(x: f64) -> f64 {
    let ax = x.abs();
    ax.next_up() - ax
}

/// Single-precision spacing at `x`, computed in `f32` and widened for host
/// storage (RunMat keeps single values as `f64`-backed `NumericDType::F32`).
fn eps_ulp_f32(x: f64) -> f64 {
    let ax = (x as f32).abs();
    (ax.next_up() - ax) as f64
}

#[cfg(all(test, feature = "parallel_xval"))]
pub(crate) mod tests {
    use super::*;
    use crate::builtins::introspection::class::class_name_for_value;
    use futures::executor::block_on;
    use runmat_builtins::{ComplexTensor, IntValue};

    fn eps_call(args: Vec<Value>) -> BuiltinResult<Value> {
        block_on(super::eps_builtin(args))
    }

    // Bare-`eps` resolution evidence. The HIR lowering (`runmat-hir`
    // `lower_expr_semantic`) resolves a bare identifier that is not a
    // variable/constant to a zero-argument builtin call iff `is_builtin(name)`
    // is true, which consults exactly these two registries. After the
    // constant->function migration, `eps` must be a builtin and NOT a constant
    // so bare `eps` dispatches as `eps()` instead of loading a constant.
    #[test]
    fn eps_is_registered_as_builtin_and_not_constant() {
        assert!(
            runmat_builtins::builtin_function_by_name("eps").is_some(),
            "eps must be registered as a builtin function"
        );
        assert!(
            runmat_builtins::constants().iter().all(|c| c.name != "eps"),
            "eps must NOT be registered as a constant after the migration"
        );
        // The other numeric constants must remain intact (pi/inf/nan/...).
        let constant_names: Vec<&str> = runmat_builtins::constants()
            .iter()
            .map(|c| c.name)
            .collect();
        for kept in ["pi", "inf", "Inf", "nan", "NaN", "i", "j"] {
            assert!(
                constant_names.contains(&kept),
                "constant '{kept}' must remain registered"
            );
        }
    }

    // Descriptor / macro coverage.
    #[test]
    fn eps_descriptor_signatures_cover_core_forms() {
        let labels: Vec<&str> = EPS_DESCRIPTOR
            .signatures
            .iter()
            .map(|sig| sig.label)
            .collect();
        assert!(labels.contains(&"d = eps"));
        assert!(labels.contains(&"d = eps(x)"));
        assert!(labels.contains(&"d = eps(classname)"));
    }

    // Observed [eps.signature-primary, eps.spacing-constants]:
    // bare `eps` -> double 2.2204460492503131e-16, class double, size 1x1.
    #[test]
    fn observed_bare_eps_is_double_epsilon() {
        let result = eps_call(Vec::new()).expect("eps");
        assert_eq!(result, Value::Num(f64::EPSILON));
        assert_eq!(f64::EPSILON, 2.2204460492503131e-16);
        assert_eq!(class_name_for_value(&result), "double");
    }

    // Observed [eps.spacing-constants]:
    // eps('single') -> single 1.1920928955078125e-07, class single, size 1x1.
    #[test]
    fn observed_eps_single_is_f32_epsilon() {
        let result = eps_call(vec![Value::from("single")]).expect("eps single");
        match &result {
            Value::Tensor(t) => {
                assert_eq!(t.dtype, NumericDType::F32);
                assert_eq!(t.shape, vec![1, 1]);
                assert_eq!(t.data, vec![1.1920928955078125e-07]);
                assert_eq!(f32::EPSILON as f64, 1.1920928955078125e-07);
            }
            other => panic!("expected single tensor, got {other:?}"),
        }
        assert_eq!(class_name_for_value(&result), "single");
    }

    // Documented choice (unobserved): eps('double') matches bare eps.
    #[test]
    fn unresolved_choice_eps_double_classname() {
        let result = eps_call(vec![Value::from("double")]).expect("eps double");
        assert_eq!(result, Value::Num(f64::EPSILON));
    }

    // Documented choice: classname matching is case-insensitive (keyword_of).
    #[test]
    fn unresolved_choice_eps_single_classname_case_insensitive() {
        let result = eps_call(vec![Value::from("SINGLE")]).expect("eps SINGLE");
        assert_eq!(class_name_for_value(&result), "single");
    }

    // Documented choice: eps(1) == eps (spacing near 1.0).
    #[test]
    fn unresolved_choice_eps_of_one_equals_epsilon() {
        let result = eps_call(vec![Value::Num(1.0)]).expect("eps(1)");
        assert_eq!(result, Value::Num(f64::EPSILON));
    }

    // Documented choice: eps(x) is the nextUp spacing; at 2.0 that is 2^-51.
    #[test]
    fn unresolved_choice_eps_of_two_is_ulp() {
        let result = eps_call(vec![Value::Num(2.0)]).expect("eps(2)");
        assert_eq!(result, Value::Num(2.0f64.powi(-51)));
    }

    // Documented choice: eps uses |x|, so negative inputs match their magnitude.
    #[test]
    fn unresolved_choice_eps_negative_uses_magnitude() {
        let neg = eps_call(vec![Value::Num(-1.0)]).expect("eps(-1)");
        assert_eq!(neg, Value::Num(f64::EPSILON));
    }

    // Documented choice: eps(0) is the smallest positive subnormal double.
    #[test]
    fn unresolved_choice_eps_of_zero_is_smallest_subnormal() {
        let result = eps_call(vec![Value::Num(0.0)]).expect("eps(0)");
        assert_eq!(result, Value::Num(f64::from_bits(1)));
    }

    // Documented choice: eps(Inf) and eps(NaN) are NaN (IEEE nextUp edges).
    #[test]
    fn unresolved_choice_eps_of_inf_and_nan_are_nan() {
        for x in [f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
            match eps_call(vec![Value::Num(x)]).expect("eps") {
                Value::Num(n) => assert!(n.is_nan(), "expected NaN for eps({x}), got {n}"),
                other => panic!("expected scalar, got {other:?}"),
            }
        }
    }

    // Documented choice: eps(array) is elementwise and preserves shape/class.
    #[test]
    fn unresolved_choice_eps_double_array_preserves_shape() {
        let input = Tensor::new(vec![1.0, 2.0, 4.0, 8.0], vec![2, 2]).unwrap();
        let result = eps_call(vec![Value::Tensor(input)]).expect("eps array");
        match result {
            Value::Tensor(t) => {
                assert_eq!(t.dtype, NumericDType::F64);
                assert_eq!(t.shape, vec![2, 2]);
                assert_eq!(
                    t.data,
                    vec![
                        f64::EPSILON,
                        2.0f64.powi(-51),
                        2.0f64.powi(-50),
                        2.0f64.powi(-49),
                    ]
                );
            }
            other => panic!("expected tensor, got {other:?}"),
        }
    }

    // Documented choice: eps of a single array stays single class.
    #[test]
    fn unresolved_choice_eps_single_array_is_single() {
        let input = Tensor::from_f32(vec![1.0f32, 2.0f32], vec![1, 2]).unwrap();
        let result = eps_call(vec![Value::Tensor(input)]).expect("eps single array");
        match &result {
            Value::Tensor(t) => {
                assert_eq!(t.dtype, NumericDType::F32);
                assert_eq!(t.shape, vec![1, 2]);
                assert_eq!(t.data, vec![f32::EPSILON as f64, (2.0f32.powi(-22)) as f64]);
            }
            other => panic!("expected single tensor, got {other:?}"),
        }
        assert_eq!(class_name_for_value(&result), "single");
    }

    // Documented choice: complex input measures spacing at |x|.
    #[test]
    fn unresolved_choice_eps_complex_uses_magnitude() {
        let result = eps_call(vec![Value::Complex(3.0, 4.0)]).expect("eps complex");
        assert_eq!(result, Value::Num(eps_ulp_f64(5.0)));
    }

    // Documented choice: complex tensor collapses to real spacing per element.
    #[test]
    fn unresolved_choice_eps_complex_tensor_is_real() {
        let ct = ComplexTensor::new(vec![(3.0, 4.0), (0.0, 1.0)], vec![1, 2]).unwrap();
        let result = eps_call(vec![Value::ComplexTensor(ct)]).expect("eps complex tensor");
        match result {
            Value::Tensor(t) => {
                assert_eq!(t.shape, vec![1, 2]);
                assert_eq!(t.data, vec![eps_ulp_f64(5.0), eps_ulp_f64(1.0)]);
            }
            other => panic!("expected real tensor, got {other:?}"),
        }
    }

    // Error: unsupported class name.
    #[test]
    fn unresolved_choice_eps_invalid_class_errors() {
        let err = eps_call(vec![Value::from("int8")]).expect_err("expected error");
        assert_eq!(err.identifier(), EPS_ERROR_INVALID_CLASS.identifier);
    }

    // Error: too many inputs.
    #[test]
    fn unresolved_choice_eps_too_many_args_errors() {
        let err = eps_call(vec![Value::Num(1.0), Value::Num(2.0)]).expect_err("expected error");
        assert_eq!(err.identifier(), EPS_ERROR_TOO_MANY_INPUTS.identifier);
    }

    // Error: non-floating-point numeric class (documented choice: reject).
    #[test]
    fn unresolved_choice_eps_integer_input_errors() {
        let err = eps_call(vec![Value::Int(IntValue::I32(4))]).expect_err("expected error");
        assert_eq!(err.identifier(), EPS_ERROR_INVALID_INPUT.identifier);
    }
}
