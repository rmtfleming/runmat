//! MATLAB-compatible `bounds` builtin for RunMat.
//!
//! Clean-room provenance: specs/004-minmax-bounds (spec `bounds` 0.2.0,
//! claims bounds.signature-primary, bounds.output-class,
//! bounds.first-output-min; the second output is summary-derived).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    Tensor, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ScalarType, ShapeRequirements,
};
use crate::{build_runtime_error, BuiltinResult, RuntimeError};

#[runmat_macros::register_gpu_spec(builtin_path = "crate::builtins::math::reduction::bounds")]
pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "bounds",
    op_kind: GpuOpKind::Custom("reduction"),
    supported_precisions: &[ScalarType::F32, ScalarType::F64],
    broadcast: BroadcastSemantics::None,
    provider_hooks: &[],
    constant_strategy: ConstantStrategy::InlineLiteral,
    residency: ResidencyPolicy::GatherImmediately,
    nan_mode: ReductionNaN::Include,
    two_pass_threshold: None,
    workgroup_size: None,
    accepts_nan_mode: false,
    notes: "Delegates to the min/max builtins, which own the GPU reduction paths.",
};

#[runmat_macros::register_fusion_spec(builtin_path = "crate::builtins::math::reduction::bounds")]
pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "bounds",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Composite of min/max reductions; not independently fusible.",
};

const BOUNDS_OUTPUTS: [BuiltinParamDescriptor; 2] = [
    BuiltinParamDescriptor {
        name: "lo",
        ty: BuiltinParamType::NumericArray,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Smallest elements along the operating dimension.",
    },
    BuiltinParamDescriptor {
        name: "hi",
        ty: BuiltinParamType::NumericArray,
        arity: BuiltinParamArity::Optional,
        default: None,
        description: "Largest elements along the operating dimension.",
    },
];

const BOUNDS_INPUTS: [BuiltinParamDescriptor; 2] = [
    BuiltinParamDescriptor {
        name: "A",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Input array.",
    },
    BuiltinParamDescriptor {
        name: "dim",
        ty: BuiltinParamType::NumericScalar,
        arity: BuiltinParamArity::Optional,
        default: None,
        description: "Operating dimension.",
    },
];

const BOUNDS_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "[lo, hi] = bounds(A) | bounds(A, dim)",
    inputs: &BOUNDS_INPUTS,
    outputs: &BOUNDS_OUTPUTS,
}];

const BOUNDS_ERROR_INVALID_ARGUMENT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.BOUNDS.INVALID_ARGUMENT",
    identifier: Some("RunMat:bounds:InvalidArgument"),
    when: "More than one option argument is supplied.",
    message: "bounds: only bounds(A) and bounds(A, dim) are supported",
};

const BOUNDS_ERRORS: [BuiltinErrorDescriptor; 1] = [BOUNDS_ERROR_INVALID_ARGUMENT];

pub const BOUNDS_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &BOUNDS_SIGNATURES,
    output_mode: BuiltinOutputMode::ByRequestedOutputCount,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &BOUNDS_ERRORS,
};

#[runtime_builtin(
    name = "bounds",
    category = "math/reduction",
    summary = "Return the smallest and largest elements, optionally along a dimension.",
    keywords = "bounds,min,max,range,reduction",
    descriptor(crate::builtins::math::reduction::bounds::BOUNDS_DESCRIPTOR),
    builtin_path = "crate::builtins::math::reduction::bounds"
)]
async fn bounds_builtin(value: Value, rest: Vec<Value>) -> BuiltinResult<Value> {
    // Option strings ('omitnan'/'includenan') are unresolved in the approved
    // spec (bounds.q-nan) and deliberately unimplemented (FR-004-05).
    let args: Vec<Value> = match rest.len() {
        0 => vec![value],
        1 => {
            let placeholder = Value::Tensor(
                Tensor::new(Vec::new(), vec![0, 0]).map_err(|e| internal_error(e.to_string()))?,
            );
            vec![value, placeholder, rest[0].clone()]
        }
        _ => return Err(invalid_argument()),
    };

    let lo = crate::call_builtin_async("min", &args).await?;
    if let Some(out_count) = crate::output_count::current_output_count() {
        if out_count >= 2 {
            let hi = crate::call_builtin_async("max", &args).await?;
            return Ok(crate::output_count::output_list_with_padding(
                out_count,
                vec![lo, hi],
            ));
        }
        return Ok(crate::output_count::output_list_with_padding(
            out_count,
            vec![lo],
        ));
    }
    Ok(lo)
}

fn invalid_argument() -> RuntimeError {
    build_runtime_error(BOUNDS_ERROR_INVALID_ARGUMENT.message)
        .with_builtin("bounds")
        .with_identifier(
            BOUNDS_ERROR_INVALID_ARGUMENT
                .identifier
                .expect("identifier"),
        )
        .build()
}

fn internal_error(detail: String) -> RuntimeError {
    build_runtime_error(format!("bounds: {detail}"))
        .with_builtin("bounds")
        .build()
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use futures::executor::block_on;

    fn tensor(data: Vec<f64>, shape: Vec<usize>) -> Value {
        Value::Tensor(Tensor::new(data, shape).unwrap())
    }

    fn run_bounds(value: Value, rest: Vec<Value>) -> Value {
        block_on(super::bounds_builtin(value, rest)).expect("bounds")
    }

    // Normative [FR-004-03; bounds.first-output-min, bounds.output-class] —
    // observed case `vector`: bounds([3 1 2]) => 1 (1x1 double).
    #[test]
    fn observed_vector_first_output_is_min() {
        let result = run_bounds(tensor(vec![3.0, 1.0, 2.0], vec![1, 3]), vec![]);
        assert_eq!(result, Value::Num(1.0));
    }

    // Normative — observed case `matrix-default`: bounds([1 2; 3 4]) => [1 2]
    // (1x2 double, column-wise minimum).
    #[test]
    fn observed_matrix_default_first_output() {
        // Column-major data for [1 2; 3 4].
        let result = run_bounds(tensor(vec![1.0, 3.0, 2.0, 4.0], vec![2, 2]), vec![]);
        match result {
            Value::Tensor(t) => {
                assert_eq!(t.shape, vec![1, 2]);
                assert_eq!(t.data, vec![1.0, 2.0]);
            }
            other => panic!("expected tensor, got {other:?}"),
        }
    }

    // Normative — observed case `matrix-dim2`: bounds([1 2; 3 4], 2) => [1;3]
    // (2x1 double, row-wise minimum).
    #[test]
    fn observed_matrix_dim2_first_output() {
        let result = run_bounds(
            tensor(vec![1.0, 3.0, 2.0, 4.0], vec![2, 2]),
            vec![Value::Num(2.0)],
        );
        match result {
            Value::Tensor(t) => {
                assert_eq!(t.shape, vec![2, 1]);
                assert_eq!(t.data, vec![1.0, 3.0]);
            }
            other => panic!("expected tensor, got {other:?}"),
        }
    }

    // summary_derived [FR-004-04; bounds.q-second-output]: hi is the maximum
    // with matching shape, per the approved summary; unobserved.
    #[test]
    fn summary_derived_second_output_is_max() {
        let _guard = crate::output_count::push_output_count(Some(2));
        let out = block_on(super::bounds_builtin(
            tensor(vec![3.0, 1.0, 2.0], vec![1, 3]),
            vec![],
        ))
        .expect("bounds");
        drop(_guard);
        match out {
            Value::OutputList(values) => {
                assert_eq!(values.len(), 2);
                assert_eq!(values[0], Value::Num(1.0));
                assert_eq!(values[1], Value::Num(3.0));
            }
            other => panic!("expected output list, got {other:?}"),
        }
    }

    // unresolved_choice [FR-004-05; bounds.q-nan]: NaN semantics inherited
    // from the min/max kernels, documented without conformance claims.
    #[test]
    fn unresolved_choice_nan_inherited_from_min_kernel() {
        let result = run_bounds(tensor(vec![f64::NAN, 1.0, 2.0], vec![1, 3]), vec![]);
        // RunMat's min kernel semantics apply verbatim; assert only that the
        // call succeeds and returns a numeric scalar.
        assert!(matches!(result, Value::Num(_)));
    }

    #[test]
    fn too_many_arguments_error() {
        let err = block_on(super::bounds_builtin(
            tensor(vec![1.0], vec![1, 1]),
            vec![Value::Num(1.0), Value::Num(2.0)],
        ))
        .expect_err("error");
        assert!(err.to_string().contains("bounds"));
    }
}
