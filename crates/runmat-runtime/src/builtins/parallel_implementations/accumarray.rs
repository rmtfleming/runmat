// Parallel clean-room implementation of `accumarray` (independently developed against the
// matlab-interface-spec black-box specs). 0-6-0 ships the DEFAULT implementation; this copy is
// retained unregistered for differential cross-validation. Original path: crates/runmat-runtime/src/builtins/array/creation/accumarray.rs
//! MATLAB-compatible `accumarray` builtin for RunMat.
//!
//! Clean-room provenance: specs/030-accumarray (spec `accumarray` 0.2.0,
//! claims accumarray.signature-primary, accumarray.output-class,
//! accumarray.sum-by-subscript). Only the single-subscript column sum form is
//! observed; the `sz`/`fillval` extensions and the scalar-`vals` broadcast are
//! independent documented choices, and the custom-function (`accumarray.q-func`)
//! and sparse-output (`accumarray.q-size`) forms are carried unresolved and
//! rejected rather than invented.

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    ResolveContext, Tensor, Type, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::array::type_resolvers::column_vector_type;
use crate::builtins::common::gpu_helpers::gather_value_async;
use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ScalarType, ShapeRequirements,
};
use crate::builtins::common::tensor;
use crate::{build_runtime_error, BuiltinResult, RuntimeError};

const BUILTIN_NAME: &str = "accumarray";

pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "accumarray",
    op_kind: GpuOpKind::Custom("scatter"),
    supported_precisions: &[ScalarType::F32, ScalarType::F64],
    broadcast: BroadcastSemantics::None,
    provider_hooks: &[],
    constant_strategy: ConstantStrategy::InlineLiteral,
    residency: ResidencyPolicy::GatherImmediately,
    nan_mode: ReductionNaN::Include,
    two_pass_threshold: None,
    workgroup_size: None,
    accepts_nan_mode: false,
    notes: "Subscript scatter-add executes on the host; GPU-resident inputs are gathered before accumulation.",
};

pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "accumarray",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Data-dependent scatter over subscripts; materialises a host output and does not participate in fusion.",
};

const ACCUMARRAY_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "A",
    ty: BuiltinParamType::NumericArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Column vector of accumulated values indexed by subscript.",
}];

const ACCUMARRAY_INPUTS: [BuiltinParamDescriptor; 5] = [
    BuiltinParamDescriptor {
        name: "subs",
        ty: BuiltinParamType::NumericArray,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Column vector of positive-integer subscripts.",
    },
    BuiltinParamDescriptor {
        name: "vals",
        ty: BuiltinParamType::NumericArray,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Values to accumulate; a scalar is broadcast to every subscript.",
    },
    BuiltinParamDescriptor {
        name: "sz",
        ty: BuiltinParamType::SizeArg,
        arity: BuiltinParamArity::Optional,
        default: None,
        description: "Optional output size [m 1]; empty [] selects max(subs).",
    },
    BuiltinParamDescriptor {
        name: "fun",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Optional,
        default: None,
        description: "Accumulation function; only the default sum ([] ) is supported.",
    },
    BuiltinParamDescriptor {
        name: "fillval",
        ty: BuiltinParamType::NumericScalar,
        arity: BuiltinParamArity::Optional,
        default: None,
        description: "Scalar used for output positions that receive no subscript (default 0).",
    },
];

const ACCUMARRAY_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "A = accumarray(subs, vals) | accumarray(subs, vals, sz) | accumarray(subs, vals, sz, [], fillval)",
    inputs: &ACCUMARRAY_INPUTS,
    outputs: &ACCUMARRAY_OUTPUT,
}];

const ACCUMARRAY_ERROR_INVALID_SUBSCRIPTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ACCUMARRAY.INVALID_SUBSCRIPTS",
    identifier: Some("RunMat:accumarray:InvalidSubscripts"),
    when: "A subscript is not a real positive integer.",
    message: "accumarray: subscripts must be real positive integers",
};

const ACCUMARRAY_ERROR_SIZE_MISMATCH: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ACCUMARRAY.SIZE_MISMATCH",
    identifier: Some("RunMat:accumarray:SizeMismatch"),
    when: "vals is neither a scalar nor the same length as subs.",
    message: "accumarray: vals must be a scalar or match the number of subscripts",
};

const ACCUMARRAY_ERROR_MULTI_COLUMN: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ACCUMARRAY.MULTI_COLUMN",
    identifier: Some("RunMat:accumarray:MultiColumnUnsupported"),
    when: "subs has more than one column (N-D output form).",
    message: "accumarray: multi-column subscripts (N-D output) are not yet supported",
};

const ACCUMARRAY_ERROR_CUSTOM_FUNCTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ACCUMARRAY.CUSTOM_FUNCTION",
    identifier: Some("RunMat:accumarray:CustomFunctionUnsupported"),
    when: "A non-empty accumulation function argument is supplied.",
    message: "accumarray: custom accumulation functions are not yet supported; pass [] for the default sum",
};

const ACCUMARRAY_ERROR_SPARSE_OUTPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ACCUMARRAY.SPARSE_OUTPUT",
    identifier: Some("RunMat:accumarray:SparseOutputUnsupported"),
    when: "The issparse option (sixth argument) is supplied.",
    message: "accumarray: sparse output (issparse) is not yet supported",
};

const ACCUMARRAY_ERROR_SIZE_ARGUMENT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ACCUMARRAY.SIZE_ARGUMENT",
    identifier: Some("RunMat:accumarray:InvalidSize"),
    when: "The sz argument is not a valid [m 1] size, or is smaller than max(subs).",
    message: "accumarray: sz must be a nonnegative [m 1] size no smaller than max(subs)",
};

const ACCUMARRAY_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ACCUMARRAY.INVALID_INPUT",
    identifier: Some("RunMat:accumarray:InvalidInput"),
    when: "An argument has an unsupported type.",
    message: "accumarray: invalid input arguments",
};

const ACCUMARRAY_ERROR_INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ACCUMARRAY.INTERNAL",
    identifier: Some("RunMat:accumarray:InternalError"),
    when: "Output tensor construction fails.",
    message: "accumarray: internal error",
};

const ACCUMARRAY_ERRORS: [BuiltinErrorDescriptor; 7] = [
    ACCUMARRAY_ERROR_INVALID_SUBSCRIPTS,
    ACCUMARRAY_ERROR_SIZE_MISMATCH,
    ACCUMARRAY_ERROR_MULTI_COLUMN,
    ACCUMARRAY_ERROR_CUSTOM_FUNCTION,
    ACCUMARRAY_ERROR_SPARSE_OUTPUT,
    ACCUMARRAY_ERROR_SIZE_ARGUMENT,
    ACCUMARRAY_ERROR_INVALID_INPUT,
];

pub const ACCUMARRAY_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &ACCUMARRAY_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ACCUMARRAY_ERRORS,
};

fn accumarray_type(_: &[Type], _: &ResolveContext) -> Type {
    // The single-subscript form always yields a double column vector; its
    // length depends on max(subs)/sz and is unknown at type time.
    column_vector_type()
}

fn accumarray_error(
    error: &'static BuiltinErrorDescriptor,
    message: impl Into<String>,
) -> RuntimeError {
    let mut builder = build_runtime_error(message).with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

/// `[]` (or any zero-element numeric/logical array) selects the default for an
/// optional argument, matching MATLAB's placeholder convention.
fn is_empty_placeholder(value: &Value) -> bool {
    match value {
        Value::Tensor(t) => t.data.is_empty(),
        Value::LogicalArray(a) => a.data.is_empty(),
        _ => false,
    }
}

fn is_function_handle(value: &Value) -> bool {
    matches!(
        value,
        Value::FunctionHandle(_)
            | Value::ExternalFunctionHandle(_)
            | Value::MethodFunctionHandle(_)
            | Value::BoundFunctionHandle { .. }
            | Value::Closure(_)
    )
}

/// Coerce a real value into a 1-based positive-integer subscript.
fn coerce_subscript(value: f64) -> Option<usize> {
    if value.is_finite() && value.fract() == 0.0 && value >= 1.0 {
        Some(value as usize)
    } else {
        None
    }
}

/// Coerce a real value into a nonnegative integer output length.
fn coerce_length(value: f64) -> Option<usize> {
    if value.is_finite() && value.fract() == 0.0 && value >= 0.0 {
        Some(value as usize)
    } else {
        None
    }
}

async fn value_to_host_tensor(name_hint: &str, value: Value) -> BuiltinResult<Tensor> {
    let gathered = gather_value_async(&value).await?;
    tensor::value_into_tensor_for(name_hint, gathered)
        .map_err(|m| accumarray_error(&ACCUMARRAY_ERROR_INVALID_INPUT, m))
}

async fn accumarray_builtin(subs: Value, vals: Value, rest: Vec<Value>) -> BuiltinResult<Value> {
    // The issparse option (sixth positional argument, i.e. rest[3]) is
    // unresolved (accumarray.q-size) and rejected clearly rather than invented.
    if rest.len() > 3 {
        return Err(accumarray_error(
            &ACCUMARRAY_ERROR_SPARSE_OUTPUT,
            ACCUMARRAY_ERROR_SPARSE_OUTPUT.message,
        ));
    }

    // --- subscripts ---------------------------------------------------------
    let subs_tensor = value_to_host_tensor("accumarray", subs).await?;
    let rows = subs_tensor.shape.first().copied().unwrap_or(0);
    let total = subs_tensor.data.len();
    let cols = if rows == 0 { 0 } else { total / rows };
    if cols > 1 {
        // A row vector [1 N] or an N-by-K matrix is the multi-column (N-D)
        // form, which is unobserved and unsupported here.
        return Err(accumarray_error(
            &ACCUMARRAY_ERROR_MULTI_COLUMN,
            ACCUMARRAY_ERROR_MULTI_COLUMN.message,
        ));
    }
    let n = total;
    let mut indices: Vec<usize> = Vec::with_capacity(n);
    let mut max_index = 0usize;
    for &raw in &subs_tensor.data {
        match coerce_subscript(raw) {
            Some(idx) => {
                max_index = max_index.max(idx);
                indices.push(idx);
            }
            None => {
                return Err(accumarray_error(
                    &ACCUMARRAY_ERROR_INVALID_SUBSCRIPTS,
                    ACCUMARRAY_ERROR_INVALID_SUBSCRIPTS.message,
                ))
            }
        }
    }

    // --- values -------------------------------------------------------------
    let vals_tensor = value_to_host_tensor("accumarray", vals).await?;
    let vals_len = vals_tensor.data.len();
    let scalar_vals = vals_len == 1;
    if !scalar_vals && vals_len != n {
        return Err(accumarray_error(
            &ACCUMARRAY_ERROR_SIZE_MISMATCH,
            ACCUMARRAY_ERROR_SIZE_MISMATCH.message,
        ));
    }

    // --- optional fun (rest[1]): only the default sum ([]) is supported -----
    if let Some(fun) = rest.get(1) {
        if is_function_handle(fun) || !is_empty_placeholder(fun) {
            return Err(accumarray_error(
                &ACCUMARRAY_ERROR_CUSTOM_FUNCTION,
                ACCUMARRAY_ERROR_CUSTOM_FUNCTION.message,
            ));
        }
    }

    // --- optional sz (rest[0]): [m 1] output length -------------------------
    let out_len = match rest.first() {
        Some(sz) if !is_empty_placeholder(sz) => {
            let sz_tensor = value_to_host_tensor("accumarray", sz.clone()).await?;
            resolve_output_length(&sz_tensor, max_index)?
        }
        _ => max_index,
    };

    // --- optional fillval (rest[2]): scalar fill for empty positions --------
    let fillval = match rest.get(2) {
        Some(fv) if !is_empty_placeholder(fv) => {
            let fv_tensor = value_to_host_tensor("accumarray", fv.clone()).await?;
            if fv_tensor.data.len() != 1 {
                return Err(accumarray_error(
                    &ACCUMARRAY_ERROR_INVALID_INPUT,
                    "accumarray: fillval must be a real scalar",
                ));
            }
            fv_tensor.data[0]
        }
        _ => 0.0,
    };

    // --- scatter-add --------------------------------------------------------
    let mut acc = vec![0.0f64; out_len];
    let mut touched = vec![false; out_len];
    for (i, &idx1) in indices.iter().enumerate() {
        let contribution = if scalar_vals {
            vals_tensor.data[0]
        } else {
            vals_tensor.data[i]
        };
        let idx0 = idx1 - 1; // idx1 <= max_index <= out_len, so idx0 is in range
        acc[idx0] += contribution;
        touched[idx0] = true;
    }
    let data: Vec<f64> = (0..out_len)
        .map(|k| if touched[k] { acc[k] } else { fillval })
        .collect();

    build_output(data, out_len)
}

fn resolve_output_length(sz_tensor: &Tensor, max_index: usize) -> BuiltinResult<usize> {
    if sz_tensor.data.is_empty() {
        return Ok(max_index);
    }
    // For the single-subscript column form, sz describes an [m 1] column; the
    // first element is the length and every trailing element must be 1.
    let m = coerce_length(sz_tensor.data[0]).ok_or_else(|| {
        accumarray_error(
            &ACCUMARRAY_ERROR_SIZE_ARGUMENT,
            ACCUMARRAY_ERROR_SIZE_ARGUMENT.message,
        )
    })?;
    for &extra in &sz_tensor.data[1..] {
        if extra != 1.0 {
            return Err(accumarray_error(
                &ACCUMARRAY_ERROR_SIZE_ARGUMENT,
                ACCUMARRAY_ERROR_SIZE_ARGUMENT.message,
            ));
        }
    }
    if m < max_index {
        return Err(accumarray_error(
            &ACCUMARRAY_ERROR_SIZE_ARGUMENT,
            ACCUMARRAY_ERROR_SIZE_ARGUMENT.message,
        ));
    }
    Ok(m)
}

fn build_output(data: Vec<f64>, out_len: usize) -> BuiltinResult<Value> {
    if out_len == 1 {
        // RunMat represents a genuine 1x1 double as a scalar Num.
        return Ok(Value::Num(data[0]));
    }
    let tensor = Tensor::new(data, vec![out_len, 1]).map_err(|e| {
        accumarray_error(
            &ACCUMARRAY_ERROR_INTERNAL,
            format!("accumarray: unable to construct output: {e}"),
        )
    })?;
    Ok(Value::Tensor(tensor))
}

#[cfg(all(test, feature = "parallel_xval"))]
pub(crate) mod tests {
    use super::*;
    use futures::executor::block_on;
    use runmat_builtins::{LogicalArray, Value};

    fn tensor(data: Vec<f64>, shape: Vec<usize>) -> Value {
        Value::Tensor(Tensor::new(data, shape).unwrap())
    }

    fn run(subs: Value, vals: Value, rest: Vec<Value>) -> BuiltinResult<Value> {
        block_on(super::accumarray_builtin(subs, vals, rest))
    }

    // Normative [FR-030-01; accumarray.sum-by-subscript, accumarray.output-class]
    // Observed case `sum`: accumarray([1;2;1],[10;20;30]) => [40;20] (2x1 double).
    #[test]
    fn observed_sum_by_subscript_matches_export() {
        let out = run(
            tensor(vec![1.0, 2.0, 1.0], vec![3, 1]),
            tensor(vec![10.0, 20.0, 30.0], vec![3, 1]),
            vec![],
        )
        .expect("accumarray");
        match out {
            Value::Tensor(t) => {
                assert_eq!(t.shape, vec![2, 1]);
                assert_eq!(t.data, vec![40.0, 20.0]);
            }
            other => panic!("expected 2x1 tensor, got {other:?}"),
        }
    }

    // Normative [FR-030-01; accumarray.output-class] — the result is a real
    // double column, never a scalar collapse when length > 1.
    #[test]
    fn observed_output_is_double_column() {
        let out = run(
            tensor(vec![2.0, 2.0, 3.0], vec![3, 1]),
            tensor(vec![1.0, 1.0, 5.0], vec![3, 1]),
            vec![],
        )
        .expect("accumarray");
        match out {
            // index 1 untouched -> 0, index 2 -> 2, index 3 -> 5.
            Value::Tensor(t) => {
                assert_eq!(t.shape, vec![3, 1]);
                assert_eq!(t.data, vec![0.0, 2.0, 5.0]);
            }
            other => panic!("expected 3x1 tensor, got {other:?}"),
        }
    }

    // Documented choice (unobserved): a scalar `vals` is broadcast to every
    // subscript. Not backed by an observation; independent choice.
    #[test]
    fn unresolved_choice_scalar_vals_broadcast() {
        let out = run(
            tensor(vec![1.0, 1.0, 2.0], vec![3, 1]),
            Value::Num(5.0),
            vec![],
        )
        .expect("accumarray");
        match out {
            Value::Tensor(t) => {
                assert_eq!(t.shape, vec![2, 1]);
                assert_eq!(t.data, vec![10.0, 5.0]);
            }
            other => panic!("expected tensor, got {other:?}"),
        }
    }

    // Documented choice (unobserved; accumarray.q-size): the sz argument fixes
    // the output length; empty positions take the default fill 0.
    #[test]
    fn unresolved_choice_sz_extends_output_length() {
        let out = run(
            tensor(vec![1.0, 2.0, 1.0], vec![3, 1]),
            tensor(vec![10.0, 20.0, 30.0], vec![3, 1]),
            vec![tensor(vec![4.0, 1.0], vec![1, 2])],
        )
        .expect("accumarray");
        match out {
            Value::Tensor(t) => {
                assert_eq!(t.shape, vec![4, 1]);
                assert_eq!(t.data, vec![40.0, 20.0, 0.0, 0.0]);
            }
            other => panic!("expected tensor, got {other:?}"),
        }
    }

    // Documented choice (unobserved; accumarray.q-size): fillval is used only
    // for positions that receive no subscript, not added to accumulated ones.
    #[test]
    fn unresolved_choice_fillval_used_for_empty_positions() {
        let out = run(
            tensor(vec![1.0, 3.0], vec![2, 1]),
            tensor(vec![5.0, 7.0], vec![2, 1]),
            vec![
                tensor(vec![4.0, 1.0], vec![1, 2]),
                tensor(vec![], vec![0, 0]), // fun = [] -> default sum
                Value::Num(-1.0),           // fillval
            ],
        )
        .expect("accumarray");
        match out {
            Value::Tensor(t) => {
                assert_eq!(t.shape, vec![4, 1]);
                assert_eq!(t.data, vec![5.0, -1.0, 7.0, -1.0]);
            }
            other => panic!("expected tensor, got {other:?}"),
        }
    }

    // Documented choice: a single-element result collapses to a scalar Num, per
    // RunMat's 1x1 convention.
    #[test]
    fn unresolved_choice_single_element_output_is_scalar() {
        let out = run(Value::Num(1.0), Value::Num(42.0), vec![]).expect("accumarray");
        assert_eq!(out, Value::Num(42.0));
    }

    // Unresolved (carried): multi-column subscripts (N-D output) are rejected.
    #[test]
    fn unresolved_choice_multi_column_subs_rejected() {
        let err = run(
            tensor(vec![1.0, 2.0, 1.0, 3.0], vec![2, 2]),
            tensor(vec![10.0, 20.0], vec![2, 1]),
            vec![],
        )
        .expect_err("multi-column should be rejected");
        assert_eq!(err.identifier(), ACCUMARRAY_ERROR_MULTI_COLUMN.identifier);
    }

    // Unresolved (carried; accumarray.q-func): custom accumulation function
    // handles are rejected rather than invented.
    #[test]
    fn unresolved_choice_custom_function_handle_rejected() {
        let err = run(
            tensor(vec![1.0, 2.0, 1.0], vec![3, 1]),
            tensor(vec![10.0, 20.0, 30.0], vec![3, 1]),
            vec![
                tensor(vec![], vec![0, 0]),
                Value::FunctionHandle("max".to_string()),
            ],
        )
        .expect_err("custom function should be rejected");
        assert_eq!(
            err.identifier(),
            ACCUMARRAY_ERROR_CUSTOM_FUNCTION.identifier
        );
    }

    // Unresolved (carried; accumarray.q-size): the issparse option is rejected.
    #[test]
    fn unresolved_choice_sparse_output_option_rejected() {
        let err = run(
            tensor(vec![1.0, 2.0, 1.0], vec![3, 1]),
            tensor(vec![10.0, 20.0, 30.0], vec![3, 1]),
            vec![
                tensor(vec![2.0, 1.0], vec![1, 2]),
                tensor(vec![], vec![0, 0]),
                Value::Num(0.0),
                Value::Bool(true), // issparse
            ],
        )
        .expect_err("issparse should be rejected");
        assert_eq!(err.identifier(), ACCUMARRAY_ERROR_SPARSE_OUTPUT.identifier);
    }

    // Validation (documented choice): subscripts must be real positive integers.
    #[test]
    fn rejects_non_positive_integer_subscripts() {
        let err = run(
            tensor(vec![0.0, 1.0], vec![2, 1]),
            tensor(vec![5.0, 6.0], vec![2, 1]),
            vec![],
        )
        .expect_err("zero subscript should be rejected");
        assert_eq!(
            err.identifier(),
            ACCUMARRAY_ERROR_INVALID_SUBSCRIPTS.identifier
        );
    }

    #[test]
    fn rejects_non_integer_subscripts() {
        let err = run(
            tensor(vec![1.5, 2.0], vec![2, 1]),
            tensor(vec![5.0, 6.0], vec![2, 1]),
            vec![],
        )
        .expect_err("fractional subscript should be rejected");
        assert_eq!(
            err.identifier(),
            ACCUMARRAY_ERROR_INVALID_SUBSCRIPTS.identifier
        );
    }

    // Validation (documented choice): vals must be scalar or match subs length.
    #[test]
    fn rejects_vals_length_mismatch() {
        let err = run(
            tensor(vec![1.0, 2.0, 1.0], vec![3, 1]),
            tensor(vec![10.0, 20.0], vec![2, 1]),
            vec![],
        )
        .expect_err("vals length mismatch should be rejected");
        assert_eq!(err.identifier(), ACCUMARRAY_ERROR_SIZE_MISMATCH.identifier);
    }

    // Validation (documented choice): sz cannot be smaller than max(subs).
    #[test]
    fn rejects_size_argument_smaller_than_max_subscript() {
        let err = run(
            tensor(vec![1.0, 5.0], vec![2, 1]),
            tensor(vec![1.0, 1.0], vec![2, 1]),
            vec![tensor(vec![3.0, 1.0], vec![1, 2])],
        )
        .expect_err("undersized sz should be rejected");
        assert_eq!(err.identifier(), ACCUMARRAY_ERROR_SIZE_ARGUMENT.identifier);
    }

    // Documented choice: logical vals are coerced like numeric vals.
    #[test]
    fn accepts_logical_vals_via_coercion() {
        let logical = Value::LogicalArray(LogicalArray::new(vec![1, 0, 1], vec![3, 1]).unwrap());
        let out =
            run(tensor(vec![1.0, 2.0, 1.0], vec![3, 1]), logical, vec![]).expect("accumarray");
        match out {
            Value::Tensor(t) => {
                assert_eq!(t.shape, vec![2, 1]);
                assert_eq!(t.data, vec![2.0, 0.0]);
            }
            other => panic!("expected tensor, got {other:?}"),
        }
    }
}
