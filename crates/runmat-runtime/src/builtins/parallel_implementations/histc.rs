// Parallel clean-room implementation of `histc` (independently developed against the
// matlab-interface-spec black-box specs). 0-6-0 ships the DEFAULT implementation; this copy is
// retained unregistered for differential cross-validation. Original path: crates/runmat-runtime/src/builtins/stats/hist/histc.rs
//! MATLAB-compatible legacy `histc` builtin for RunMat.
//!
//! Clean-room provenance: specs/024-histc (spec `histc` 0.2.0, source commit
//! `4445294`; claims histc.signature-primary, histc.output-class,
//! histc.bin-counts). Behaviour beyond the single observed case
//! (`histc([1 2 2 3], [1 2 3]) => [1 2 1]`) — column/matrix orientation, the
//! `dim` argument, the second (bin-index) output, out-of-range and NaN
//! handling, and edge-monotonicity errors — is documented independent choice
//! and MUST NOT be treated as MATLAB-conformant.

use runmat_accelerate_api::GpuTensorHandle;
use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    Tensor, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::gpu_helpers;
use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ProviderHook, ReductionNaN, ResidencyPolicy, ScalarType, ShapeRequirements,
};
use crate::builtins::common::tensor;
use crate::{build_runtime_error, BuiltinResult, RuntimeError};

const BUILTIN_NAME: &str = "histc";

const HISTC_OUTPUTS: [BuiltinParamDescriptor; 2] = [
    BuiltinParamDescriptor {
        name: "n",
        ty: BuiltinParamType::NumericArray,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Per-bin counts; one element per edge.",
    },
    BuiltinParamDescriptor {
        name: "idx",
        ty: BuiltinParamType::NumericArray,
        arity: BuiltinParamArity::Optional,
        default: None,
        description: "1-based bin index for each input element (0 if uncounted).",
    },
];

const HISTC_INPUTS: [BuiltinParamDescriptor; 3] = [
    BuiltinParamDescriptor {
        name: "x",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Input data values.",
    },
    BuiltinParamDescriptor {
        name: "edges",
        ty: BuiltinParamType::NumericArray,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Monotonically non-decreasing edge boundaries.",
    },
    BuiltinParamDescriptor {
        name: "dim",
        ty: BuiltinParamType::NumericScalar,
        arity: BuiltinParamArity::Optional,
        default: None,
        description: "Operating dimension (1-based).",
    },
];

const HISTC_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "[n, idx] = histc(x, edges) | histc(x, edges, dim)",
    inputs: &HISTC_INPUTS,
    outputs: &HISTC_OUTPUTS,
}];

const HISTC_ERROR_INVALID_ARGUMENT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.HISTC.INVALID_ARGUMENT",
    identifier: Some("RunMat:histc:InvalidArgument"),
    when: "Arguments are missing, too many, or the edges are not a valid vector.",
    message: "histc: invalid argument",
};

const HISTC_ERROR_EDGES_NOT_MONOTONIC: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.HISTC.EDGES_NOT_MONOTONIC",
    identifier: Some("RunMat:histc:EdgesNotMonotonic"),
    when: "The edge vector contains NaN or is not monotonically non-decreasing.",
    message: "histc: edges must be monotonically non-decreasing finite values",
};

const HISTC_ERRORS: [BuiltinErrorDescriptor; 2] = [
    HISTC_ERROR_INVALID_ARGUMENT,
    HISTC_ERROR_EDGES_NOT_MONOTONIC,
];

pub const HISTC_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &HISTC_SIGNATURES,
    output_mode: BuiltinOutputMode::ByRequestedOutputCount,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &HISTC_ERRORS,
};

fn builtin_error_with(
    error: &'static BuiltinErrorDescriptor,
    message: impl Into<String>,
) -> RuntimeError {
    let mut builder = build_runtime_error(message).with_builtin(BUILTIN_NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

fn invalid_argument(message: impl Into<String>) -> RuntimeError {
    builtin_error_with(&HISTC_ERROR_INVALID_ARGUMENT, message)
}

fn builtin_error(message: impl Into<String>) -> RuntimeError {
    build_runtime_error(message)
        .with_builtin(BUILTIN_NAME)
        .build()
}

pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "histc",
    op_kind: GpuOpKind::Custom("histc"),
    supported_precisions: &[ScalarType::F32, ScalarType::F64],
    broadcast: BroadcastSemantics::None,
    provider_hooks: &[ProviderHook::Custom("histc")],
    constant_strategy: ConstantStrategy::InlineLiteral,
    residency: ResidencyPolicy::GatherImmediately,
    nan_mode: ReductionNaN::Omit,
    two_pass_threshold: None,
    workgroup_size: None,
    accepts_nan_mode: false,
    notes: "Legacy edge-based histogram; current builds gather to host memory before binning.",
};

pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "histc",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Edge-based binning materialises counts on the host and terminates fusion chains.",
};

async fn histc_builtin(x: Value, rest: Vec<Value>) -> BuiltinResult<Value> {
    let eval = evaluate(x, &rest).await?;
    if let Some(out_count) = crate::output_count::current_output_count() {
        if out_count == 0 {
            return Ok(Value::OutputList(Vec::new()));
        }
        if out_count == 1 {
            return Ok(Value::OutputList(vec![eval.into_counts_value()]));
        }
        let (counts, idx) = eval.into_pair();
        return Ok(crate::output_count::output_list_with_padding(
            out_count,
            vec![counts, idx],
        ));
    }
    Ok(eval.into_counts_value())
}

/// Evaluate `histc` once, surfacing both the counts and the bin-index output.
pub async fn evaluate(x: Value, rest: &[Value]) -> BuiltinResult<HistcEvaluation> {
    if rest.is_empty() {
        return Err(invalid_argument("histc: requires edge boundaries (edges)"));
    }
    if rest.len() > 2 {
        return Err(invalid_argument(
            "histc: only histc(x, edges) and histc(x, edges, dim) are supported",
        ));
    }

    let edges = edges_from_value(&rest[0])?;
    validate_edges(&edges)?;
    let dim_arg = match rest.get(1) {
        Some(value) => Some(dim_from_value(value)?),
        None => None,
    };

    let tensor = match x {
        Value::GpuTensor(handle) => gather(handle).await?,
        other => tensor::value_into_tensor_for(BUILTIN_NAME, other).map_err(builtin_error)?,
    };

    histc_from_tensor(tensor, &edges, dim_arg)
}

async fn gather(handle: GpuTensorHandle) -> BuiltinResult<Tensor> {
    gpu_helpers::gather_tensor_async(&handle).await
}

fn histc_from_tensor(
    tensor: Tensor,
    edges: &[f64],
    dim_arg: Option<usize>,
) -> BuiltinResult<HistcEvaluation> {
    let mut dims = tensor.shape.clone();
    if dims.is_empty() {
        dims = vec![1, 1];
    }

    // Operating dimension (0-based). Default: first non-singleton dimension.
    let dim = match dim_arg {
        Some(d) => d - 1,
        None => default_dim(&dims),
    };
    if dim >= dims.len() {
        // A dimension beyond the array extent is singleton; extend the shape.
        dims.resize(dim + 1, 1);
    }

    let bins = edges.len();
    let in_strides = column_major_strides(&dims);

    let mut out_dims = dims.clone();
    out_dims[dim] = bins;
    let out_strides = column_major_strides(&out_dims);

    let total: usize = dims.iter().product();
    let out_total: usize = out_dims.iter().product();
    let mut counts = vec![0.0f64; out_total];
    let mut idx = vec![0.0f64; total];

    let len_d = dims[dim];
    let step_in = in_strides[dim];
    let step_out = out_strides[dim];

    for base_in in 0..total {
        // Only iterate line starts (coordinate along `dim` == 0).
        if !(base_in / step_in).is_multiple_of(len_d) {
            continue;
        }
        // Map the line start into the output layout.
        let mut base_out = 0usize;
        for (axis, &size) in dims.iter().enumerate() {
            let coord = if axis == dim {
                0
            } else {
                (base_in / in_strides[axis]) % size
            };
            base_out += coord * out_strides[axis];
        }

        for t in 0..len_d {
            let pos = base_in + t * step_in;
            let value = tensor.data[pos];
            if let Some(bin) = bin_of(value, edges) {
                counts[base_out + bin * step_out] += 1.0;
                idx[pos] = (bin + 1) as f64;
            }
        }
    }

    let counts_tensor =
        Tensor::new(counts, out_dims).map_err(|e| builtin_error(format!("histc: {e}")))?;
    let idx_tensor = Tensor::new(idx, dims).map_err(|e| builtin_error(format!("histc: {e}")))?;
    Ok(HistcEvaluation::new(counts_tensor, idx_tensor))
}

/// Bin index for `value`, or `None` when it is not counted.
///
/// Bin `k` counts `edges[k] <= v < edges[k+1]`; the final bin counts values
/// exactly equal to `edges[last]` (the observed last-edge behaviour). Because
/// the predicate uses `<=`, `partition_point` places `v == edges[last]` in the
/// final bin automatically. Values below `edges[0]`, values strictly above
/// `edges[last]`, and NaN are not counted (documented choice).
fn bin_of(value: f64, edges: &[f64]) -> Option<usize> {
    if value.is_nan() || edges.is_empty() {
        return None;
    }
    let last = edges.len() - 1;
    if value < edges[0] || value > edges[last] {
        return None;
    }
    let pp = edges.partition_point(|&edge| edge <= value);
    if pp == 0 {
        None
    } else {
        Some(pp - 1)
    }
}

fn default_dim(dims: &[usize]) -> usize {
    dims.iter().position(|&d| d != 1).unwrap_or(0)
}

fn column_major_strides(dims: &[usize]) -> Vec<usize> {
    let mut strides = vec![1usize; dims.len()];
    for axis in 1..dims.len() {
        strides[axis] = strides[axis - 1] * dims[axis - 1];
    }
    strides
}

fn validate_edges(edges: &[f64]) -> BuiltinResult<()> {
    for pair in edges.windows(2) {
        if pair[0].is_nan() || pair[1].is_nan() {
            return Err(builtin_error_with(
                &HISTC_ERROR_EDGES_NOT_MONOTONIC,
                HISTC_ERROR_EDGES_NOT_MONOTONIC.message,
            ));
        }
        if pair[1] < pair[0] {
            return Err(builtin_error_with(
                &HISTC_ERROR_EDGES_NOT_MONOTONIC,
                HISTC_ERROR_EDGES_NOT_MONOTONIC.message,
            ));
        }
    }
    if let Some(first) = edges.first() {
        if first.is_nan() {
            return Err(builtin_error_with(
                &HISTC_ERROR_EDGES_NOT_MONOTONIC,
                HISTC_ERROR_EDGES_NOT_MONOTONIC.message,
            ));
        }
    }
    Ok(())
}

fn edges_from_value(value: &Value) -> BuiltinResult<Vec<f64>> {
    let tensor = tensor::value_to_tensor(value)
        .map_err(|_| invalid_argument("histc: edges must be a numeric vector"))?;
    Ok(tensor.data)
}

fn dim_from_value(value: &Value) -> BuiltinResult<usize> {
    let scalar = match value {
        Value::Num(n) => *n,
        Value::Int(i) => i.to_f64(),
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        Value::Tensor(t) if t.data.len() == 1 => t.data[0],
        Value::LogicalArray(l) if l.data.len() == 1 => {
            if l.data[0] != 0 {
                1.0
            } else {
                0.0
            }
        }
        _ => {
            return Err(invalid_argument(
                "histc: dim must be a positive integer scalar",
            ))
        }
    };
    if !scalar.is_finite() || scalar < 1.0 || scalar.fract() != 0.0 {
        return Err(invalid_argument(
            "histc: dim must be a positive integer scalar",
        ));
    }
    Ok(scalar as usize)
}

#[derive(Clone)]
pub struct HistcEvaluation {
    counts: Tensor,
    idx: Tensor,
}

impl HistcEvaluation {
    fn new(counts: Tensor, idx: Tensor) -> Self {
        Self { counts, idx }
    }

    pub fn into_counts_value(self) -> Value {
        Value::Tensor(self.counts)
    }

    pub fn into_pair(self) -> (Value, Value) {
        (Value::Tensor(self.counts), Value::Tensor(self.idx))
    }
}

#[cfg(all(test, feature = "parallel_xval"))]
pub(crate) mod tests {
    use super::*;
    use crate::builtins::common::test_support;
    use futures::executor::block_on;
    use runmat_builtins::{ResolveContext, Tensor, Type, Value};

    fn tensor(data: Vec<f64>, shape: Vec<usize>) -> Value {
        Value::Tensor(Tensor::new(data, shape).unwrap())
    }

    fn counts(x: Value, rest: Vec<Value>) -> (Vec<f64>, Vec<usize>) {
        let eval = block_on(evaluate(x, &rest)).expect("histc");
        match eval.into_counts_value() {
            Value::Tensor(t) => (t.data, t.shape),
            other => panic!("expected tensor, got {other:?}"),
        }
    }

    // Normative [FR-024-01..03; histc.signature-primary, histc.output-class,
    // histc.bin-counts] — observed case `basic`:
    // histc([1 2 2 3], [1 2 3]) => [1 2 1] (1x3 double). bin1 [1,2)={1}=1,
    // bin2 [2,3)={2,2}=2, final bin (v == last edge 3) = {3} = 1.
    #[test]
    fn observed_basic_counts_and_last_bin() {
        let (data, shape) = counts(
            tensor(vec![1.0, 2.0, 2.0, 3.0], vec![1, 4]),
            vec![tensor(vec![1.0, 2.0, 3.0], vec![1, 3])],
        );
        assert_eq!(shape, vec![1, 3]);
        assert_eq!(data, vec![1.0, 2.0, 1.0]);
    }

    // Normative [FR-024-01; histc.output-class] — the first output is class
    // double (RunMat `Value::Tensor` of f64).
    #[test]
    fn observed_output_class_is_double() {
        let eval = block_on(evaluate(
            tensor(vec![1.0, 2.0, 2.0, 3.0], vec![1, 4]),
            &[tensor(vec![1.0, 2.0, 3.0], vec![1, 3])],
        ))
        .expect("histc");
        assert!(matches!(eval.into_counts_value(), Value::Tensor(_)));
    }

    // unresolved_choice [FR-024-04] — column-vector input keeps column
    // orientation; output length still equals length(edges). Unobserved.
    #[test]
    fn unresolved_choice_column_input_keeps_orientation() {
        let (data, shape) = counts(
            tensor(vec![1.0, 2.0, 2.0, 3.0], vec![4, 1]),
            vec![tensor(vec![1.0, 2.0, 3.0], vec![3, 1])],
        );
        assert_eq!(shape, vec![3, 1]);
        assert_eq!(data, vec![1.0, 2.0, 1.0]);
    }

    // unresolved_choice [FR-024-04] — values below edges(1) are not counted.
    // Unobserved.
    #[test]
    fn unresolved_choice_below_first_edge_dropped() {
        let (data, _) = counts(
            tensor(vec![0.0, 1.0, 2.0, 2.0, 3.0], vec![1, 5]),
            vec![tensor(vec![1.0, 2.0, 3.0], vec![1, 3])],
        );
        assert_eq!(data, vec![1.0, 2.0, 1.0]);
    }

    // unresolved_choice [FR-024-04] — values strictly above edges(end) are not
    // counted (only exact equality to the last edge lands in the final bin).
    // Unobserved.
    #[test]
    fn unresolved_choice_above_last_edge_dropped() {
        let (data, _) = counts(
            tensor(vec![1.0, 2.0, 2.0, 3.0, 5.0], vec![1, 5]),
            vec![tensor(vec![1.0, 2.0, 3.0], vec![1, 3])],
        );
        assert_eq!(data, vec![1.0, 2.0, 1.0]);
    }

    // unresolved_choice [FR-024-04] — NaN inputs are not counted. Unobserved.
    #[test]
    fn unresolved_choice_nan_dropped() {
        let (data, _) = counts(
            tensor(vec![1.0, f64::NAN, 2.0, 2.0, 3.0], vec![1, 5]),
            vec![tensor(vec![1.0, 2.0, 3.0], vec![1, 3])],
        );
        assert_eq!(data, vec![1.0, 2.0, 1.0]);
    }

    // unresolved_choice [FR-024-05] — matrix input is histogrammed column-wise
    // along the first non-singleton dimension by default. Unobserved.
    // Columns: [1;2] and [2;3]; edges [1 2 3].
    #[test]
    fn unresolved_choice_matrix_columnwise_default() {
        // Column-major [1 2; 2 3]: col0 = [1,2], col1 = [2,3].
        let (data, shape) = counts(
            tensor(vec![1.0, 2.0, 2.0, 3.0], vec![2, 2]),
            vec![tensor(vec![1.0, 2.0, 3.0], vec![1, 3])],
        );
        assert_eq!(shape, vec![3, 2]);
        // col0: bin1={1}=1, bin2={2}=1, last(none)=0 => [1,1,0]
        // col1: bin1(none)=0, bin2={2}=1, last{3}=1 => [0,1,1]
        assert_eq!(data, vec![1.0, 1.0, 0.0, 0.0, 1.0, 1.0]);
    }

    // unresolved_choice [FR-024-05] — an explicit dim argument selects the
    // operating dimension. Unobserved.
    #[test]
    fn unresolved_choice_dim_argument() {
        // [1 2; 2 3] column-major, operate along dim 2 (rows).
        let (data, shape) = counts(
            tensor(vec![1.0, 2.0, 2.0, 3.0], vec![2, 2]),
            vec![tensor(vec![1.0, 2.0, 3.0], vec![1, 3]), Value::Num(2.0)],
        );
        assert_eq!(shape, vec![2, 3]);
        // row0 = [1,2] => [1,1,0]; row1 = [2,3] => [0,1,1]
        // column-major out: col0=[1,0], col1=[1,1], col2=[0,1]
        assert_eq!(data, vec![1.0, 0.0, 1.0, 1.0, 0.0, 1.0]);
    }

    // unresolved_choice [FR-024-06] — the second output gives the 1-based bin
    // index for each element (0 if uncounted). Unobserved.
    #[test]
    fn unresolved_choice_second_output_bin_indices() {
        let _guard = crate::output_count::push_output_count(Some(2));
        let out = block_on(histc_builtin(
            tensor(vec![0.0, 1.0, 2.0, 2.0, 3.0], vec![1, 5]),
            vec![tensor(vec![1.0, 2.0, 3.0], vec![1, 3])],
        ))
        .expect("histc");
        drop(_guard);
        match out {
            Value::OutputList(values) => {
                assert_eq!(values.len(), 2);
                match &values[0] {
                    Value::Tensor(t) => assert_eq!(t.data, vec![1.0, 2.0, 1.0]),
                    other => panic!("expected counts tensor, got {other:?}"),
                }
                match &values[1] {
                    Value::Tensor(t) => {
                        // 0 (below), 1, 2, 2, 3(last bin)
                        assert_eq!(t.data, vec![0.0, 1.0, 2.0, 2.0, 3.0]);
                        assert_eq!(t.shape, vec![1, 5]);
                    }
                    other => panic!("expected idx tensor, got {other:?}"),
                }
            }
            other => panic!("expected output list, got {other:?}"),
        }
    }

    // unresolved_choice [FR-024-07] — non-monotonic edges raise an error.
    // Unobserved.
    #[test]
    fn unresolved_choice_non_monotonic_edges_error() {
        let err = block_on(evaluate(
            tensor(vec![1.0, 2.0], vec![1, 2]),
            &[tensor(vec![3.0, 2.0, 1.0], vec![1, 3])],
        ))
        .err()
        .expect("non-monotonic edges should error");
        assert_eq!(err.identifier(), HISTC_ERROR_EDGES_NOT_MONOTONIC.identifier);
    }

    #[test]
    fn missing_edges_errors() {
        let err = block_on(evaluate(tensor(vec![1.0], vec![1, 1]), &[]))
            .err()
            .expect("missing edges should error");
        assert_eq!(err.identifier(), HISTC_ERROR_INVALID_ARGUMENT.identifier);
    }

    #[test]
    fn descriptor_signature_covers_primary_form() {
        let labels: Vec<&str> = HISTC_DESCRIPTOR
            .signatures
            .iter()
            .map(|sig| sig.label)
            .collect();
        assert!(labels.iter().any(|label| label.contains("histc(x, edges)")));
    }

    #[test]
    fn histc_type_uses_edge_length_row() {
        let ctx = ResolveContext::new(Vec::new());
        let out = histc_type(
            &[
                Type::Tensor {
                    shape: Some(vec![Some(1), Some(4)]),
                },
                Type::Tensor {
                    shape: Some(vec![Some(1), Some(3)]),
                },
            ],
            &ctx,
        );
        assert_eq!(
            out,
            Type::Tensor {
                shape: Some(vec![Some(1), Some(3)])
            }
        );
    }

    // GPU input is gathered to host before binning; result matches the host
    // path for the observed case.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn gpu_roundtrip_matches_host() {
        test_support::with_test_provider(|provider| {
            let host = Tensor::new(vec![1.0, 2.0, 2.0, 3.0], vec![1, 4]).unwrap();
            let view = runmat_accelerate_api::HostTensorView {
                data: &host.data,
                shape: &host.shape,
            };
            let handle = provider.upload(&view).expect("upload");
            let (data, shape) = counts(
                Value::GpuTensor(handle),
                vec![tensor(vec![1.0, 2.0, 3.0], vec![1, 3])],
            );
            assert_eq!(shape, vec![1, 3]);
            assert_eq!(data, vec![1.0, 2.0, 1.0]);
        });
    }
}
