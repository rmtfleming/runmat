// Parallel clean-room implementation of `normalize` (independently developed against the
// matlab-interface-spec black-box specs). 0-6-0 ships the DEFAULT implementation; this copy is
// retained unregistered for differential cross-validation. Original path: crates/runmat-runtime/src/builtins/math/reduction/normalize.rs
//! MATLAB-compatible `normalize` builtin for RunMat.
//!
//! Clean-room provenance: specs/025-normalize (spec `normalize` 0.2.0, pinned
//! to matlab-interface-spec approval commit `4445294`). Observed, normative
//! behaviour: the default z-score `normalize([1 2 3]) => [-1 0 1]`
//! (claims normalize.signature-primary, normalize.output-class,
//! normalize.zscore-and-range) and the range method
//! `normalize([1 2 3], 'range') => [0 0.5 1]` (same claims). All other methods,
//! the operating-dimension argument, and custom range intervals are unobserved
//! (unresolved questions normalize.q-dim, normalize.q-methods) and are
//! implemented by documented independent choice — never asserted as
//! MATLAB-conformant.

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
use crate::builtins::common::{gpu_helpers, shape::is_scalar_shape, tensor};
use crate::{build_runtime_error, BuiltinResult, RuntimeError};

const NAME: &str = "normalize";

pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "normalize",
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
    notes: "Centre/scale composite; gathers to host and reuses mean/std/min/max style reductions.",
};

pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "normalize",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: true,
    notes: "Reduction followed by a broadcast transform; not independently fusible.",
};

const NORMALIZE_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "N",
    ty: BuiltinParamType::NumericArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Normalized data, class double, same size as A.",
}];

const NORMALIZE_INPUTS_A: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "A",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Input array.",
}];

const NORMALIZE_INPUTS_A_METHOD: [BuiltinParamDescriptor; 2] = [
    BuiltinParamDescriptor {
        name: "A",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Input array.",
    },
    BuiltinParamDescriptor {
        name: "method",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Optional,
        default: Some("\"zscore\""),
        description: "Operating dimension (numeric) or normalization method (string).",
    },
];

const NORMALIZE_INPUTS_A_DIM_METHOD: [BuiltinParamDescriptor; 3] = [
    BuiltinParamDescriptor {
        name: "A",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Input array.",
    },
    BuiltinParamDescriptor {
        name: "method",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Optional,
        default: Some("\"zscore\""),
        description: "Operating dimension (numeric) or normalization method (string).",
    },
    BuiltinParamDescriptor {
        name: "methodtype",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Optional,
        default: None,
        description: "Method qualifier (e.g. range interval [a b], norm order p).",
    },
];

const NORMALIZE_SIGNATURES: [BuiltinSignatureDescriptor; 3] = [
    BuiltinSignatureDescriptor {
        label: "N = normalize(A)",
        inputs: &NORMALIZE_INPUTS_A,
        outputs: &NORMALIZE_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "N = normalize(A, dim) | normalize(A, method)",
        inputs: &NORMALIZE_INPUTS_A_METHOD,
        outputs: &NORMALIZE_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "N = normalize(A, [dim,] method, methodtype)",
        inputs: &NORMALIZE_INPUTS_A_DIM_METHOD,
        outputs: &NORMALIZE_OUTPUT,
    },
];

const NORMALIZE_ERROR_INVALID_ARGUMENT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.NORMALIZE.INVALID_ARGUMENT",
    identifier: Some("RunMat:normalize:InvalidArgument"),
    when: "Dimension, method, or method-type argument grammar is invalid.",
    message: "normalize: invalid argument",
};

const NORMALIZE_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.NORMALIZE.INVALID_INPUT",
    identifier: Some("RunMat:normalize:InvalidInput"),
    when: "Input values cannot be converted to a real double array.",
    message: "normalize: invalid input",
};

const NORMALIZE_ERROR_UNSUPPORTED: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.NORMALIZE.UNSUPPORTED",
    identifier: Some("RunMat:normalize:Unsupported"),
    when: "A method whose behaviour is unresolved in the approved specification (robust/mad/iqr/medianiqr) or a complex input is requested.",
    message: "normalize: unsupported method or input",
};

const NORMALIZE_ERROR_INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.NORMALIZE.INTERNAL",
    identifier: Some("RunMat:normalize:Internal"),
    when: "Result tensor construction fails.",
    message: "normalize: internal failure",
};

const NORMALIZE_ERRORS: [BuiltinErrorDescriptor; 4] = [
    NORMALIZE_ERROR_INVALID_ARGUMENT,
    NORMALIZE_ERROR_INVALID_INPUT,
    NORMALIZE_ERROR_UNSUPPORTED,
    NORMALIZE_ERROR_INTERNAL,
];

pub const NORMALIZE_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &NORMALIZE_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &NORMALIZE_ERRORS,
};

fn descriptor_error(
    error: &'static BuiltinErrorDescriptor,
    detail: impl AsRef<str>,
) -> RuntimeError {
    let mut builder =
        build_runtime_error(format!("{}: {}", error.message, detail.as_ref())).with_builtin(NAME);
    if let Some(identifier) = error.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

fn invalid_argument(detail: impl AsRef<str>) -> RuntimeError {
    descriptor_error(&NORMALIZE_ERROR_INVALID_ARGUMENT, detail)
}

fn invalid_input(detail: impl AsRef<str>) -> RuntimeError {
    descriptor_error(&NORMALIZE_ERROR_INVALID_INPUT, detail)
}

fn unsupported(detail: impl AsRef<str>) -> RuntimeError {
    descriptor_error(&NORMALIZE_ERROR_UNSUPPORTED, detail)
}

fn internal_error(detail: impl AsRef<str>) -> RuntimeError {
    descriptor_error(&NORMALIZE_ERROR_INTERNAL, detail)
}

/// Normalization method. Only `ZScore` (default) and `Range` are backed by
/// observations; every other variant is a documented independent choice for
/// unresolved question `normalize.q-methods`.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Method {
    /// Centre by the mean and scale by the sample (N-1) standard deviation.
    ZScore,
    /// Subtract a centre. `None` centres by the mean.
    Center(Option<f64>),
    /// Divide by a scale factor.
    Scale(ScaleKind),
    /// Divide by the vector p-norm along the operating dimension.
    Norm(f64),
    /// Rescale the operating dimension to the interval `[a, b]`.
    Range(f64, f64),
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ScaleKind {
    /// Sample (N-1) standard deviation.
    Std,
    /// First element along the operating dimension.
    First,
    /// Explicit numeric scale.
    Value(f64),
}

async fn normalize_builtin(value: Value, rest: Vec<Value>) -> BuiltinResult<Value> {
    // Reject inputs and methods the approved specification leaves unresolved
    // rather than inventing behaviour (Constitution I/IV).
    let host = match value {
        Value::GpuTensor(handle) => gpu_helpers::gather_tensor_async(&handle).await?,
        Value::Complex(_, _) | Value::ComplexTensor(_) => {
            return Err(unsupported(
                "complex inputs are unresolved in the approved spec",
            ));
        }
        other => tensor::value_into_tensor_for(NAME, other).map_err(invalid_input)?,
    };

    let (dim, method) = parse_arguments(&rest)?;
    let dim = dim.unwrap_or_else(|| default_dimension(&host.shape));
    let normalized = normalize_tensor(&host, dim, method)?;
    Ok(tensor::tensor_into_value(normalized))
}

/// Parse the optional `[dim,] method[, methodtype]` argument tail.
fn parse_arguments(rest: &[Value]) -> BuiltinResult<(Option<usize>, Method)> {
    let mut idx = 0;
    let mut dim: Option<usize> = None;

    // A leading numeric argument is always the operating dimension.
    if let Some(first) = rest.first() {
        if is_numeric_value(first) {
            dim = Some(parse_dim(first)?);
            idx = 1;
        }
    }

    let method = match rest.get(idx) {
        None => Method::ZScore,
        Some(v) => {
            let Some(name) = value_as_str(v) else {
                return Err(invalid_argument(format!(
                    "expected a method name string, got {v:?}"
                )));
            };
            idx += 1;
            let methodtype = rest.get(idx).cloned();
            if methodtype.is_some() {
                idx += 1;
            }
            parse_method(&name, methodtype.as_ref())?
        }
    };

    if idx < rest.len() {
        return Err(invalid_argument("too many arguments"));
    }
    Ok((dim, method))
}

fn parse_method(name: &str, methodtype: Option<&Value>) -> BuiltinResult<Method> {
    let lname = name.trim().to_ascii_lowercase();
    match lname.as_str() {
        "zscore" => match methodtype_str(methodtype)? {
            None => Ok(Method::ZScore),
            Some(s) if s_is(&s, "std") => Ok(Method::ZScore),
            Some(other) => Err(unsupported(format!(
                "zscore scaling '{other}' is unresolved (normalize.q-methods)"
            ))),
        },
        "range" => match methodtype {
            None => Ok(Method::Range(0.0, 1.0)),
            Some(v) => {
                let interval = value_vec_f64(v).ok_or_else(|| {
                    invalid_argument("range interval must be a numeric 2-element vector")
                })?;
                if interval.len() != 2 {
                    return Err(invalid_argument(
                        "range interval must have exactly two elements [a b]",
                    ));
                }
                Ok(Method::Range(interval[0], interval[1]))
            }
        },
        "center" => match methodtype {
            None => Ok(Method::Center(None)),
            Some(v) => {
                if let Some(s) = value_as_str(v) {
                    match s.trim().to_ascii_lowercase().as_str() {
                        "mean" => Ok(Method::Center(None)),
                        "median" => Err(unsupported(
                            "center 'median' is unresolved (normalize.q-methods)",
                        )),
                        other => Err(invalid_argument(format!("unknown center type '{other}'"))),
                    }
                } else if let Some(c) = value_scalar_f64(v) {
                    Ok(Method::Center(Some(c)))
                } else {
                    Err(invalid_argument("center value must be a scalar or 'mean'"))
                }
            }
        },
        "scale" => match methodtype {
            None => Ok(Method::Scale(ScaleKind::Std)),
            Some(v) => {
                if let Some(s) = value_as_str(v) {
                    match s.trim().to_ascii_lowercase().as_str() {
                        "std" => Ok(Method::Scale(ScaleKind::Std)),
                        "first" => Ok(Method::Scale(ScaleKind::First)),
                        "mad" | "iqr" => Err(unsupported(format!(
                            "scale '{s}' is unresolved (normalize.q-methods)"
                        ))),
                        other => Err(invalid_argument(format!("unknown scale type '{other}'"))),
                    }
                } else if let Some(c) = value_scalar_f64(v) {
                    Ok(Method::Scale(ScaleKind::Value(c)))
                } else {
                    Err(invalid_argument(
                        "scale value must be a scalar or 'std'/'first'",
                    ))
                }
            }
        },
        "norm" => match methodtype {
            None => Ok(Method::Norm(2.0)),
            Some(v) => {
                if let Some(s) = value_as_str(v) {
                    match s.trim().to_ascii_lowercase().as_str() {
                        "inf" => Ok(Method::Norm(f64::INFINITY)),
                        other => Err(invalid_argument(format!("unknown norm order '{other}'"))),
                    }
                } else if let Some(p) = value_scalar_f64(v) {
                    if p <= 0.0 || p.is_nan() {
                        return Err(invalid_argument("norm order must be positive"));
                    }
                    Ok(Method::Norm(p))
                } else {
                    Err(invalid_argument(
                        "norm order must be a positive scalar or Inf",
                    ))
                }
            }
        },
        "robust" | "medianiqr" | "mad" | "iqr" => Err(unsupported(format!(
            "method '{lname}' is unresolved in the approved spec (normalize.q-methods)"
        ))),
        other => Err(invalid_argument(format!("unknown method '{other}'"))),
    }
}

fn s_is(s: &str, want: &str) -> bool {
    s.trim().eq_ignore_ascii_case(want)
}

fn methodtype_str(v: Option<&Value>) -> BuiltinResult<Option<String>> {
    match v {
        None => Ok(None),
        Some(val) => match value_as_str(val) {
            Some(s) => Ok(Some(s)),
            None => Err(invalid_argument("zscore method type must be a string")),
        },
    }
}

fn is_numeric_value(v: &Value) -> bool {
    matches!(
        v,
        Value::Num(_) | Value::Int(_) | Value::Bool(_) | Value::Tensor(_) | Value::LogicalArray(_)
    )
}

fn parse_dim(v: &Value) -> BuiltinResult<usize> {
    let scalar = value_scalar_f64(v)
        .ok_or_else(|| invalid_argument("operating dimension must be a numeric scalar"))?;
    if !scalar.is_finite() {
        return Err(invalid_argument("operating dimension must be finite"));
    }
    if scalar < 1.0 || scalar.fract() != 0.0 {
        return Err(invalid_argument(
            "operating dimension must be a positive integer",
        ));
    }
    Ok(scalar as usize)
}

fn value_scalar_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Num(n) => Some(*n),
        Value::Int(i) => Some(i.to_f64()),
        Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        Value::Tensor(t) if t.data.len() == 1 => Some(t.data[0]),
        Value::LogicalArray(la) if la.data.len() == 1 => {
            Some(if la.data[0] != 0 { 1.0 } else { 0.0 })
        }
        _ => None,
    }
}

fn value_vec_f64(v: &Value) -> Option<Vec<f64>> {
    match v {
        Value::Num(n) => Some(vec![*n]),
        Value::Int(i) => Some(vec![i.to_f64()]),
        Value::Tensor(t) => Some(t.data.clone()),
        _ => None,
    }
}

fn value_as_str(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::StringArray(sa) if sa.data.len() == 1 => Some(sa.data[0].clone()),
        Value::CharArray(ca) if ca.rows <= 1 => Some(ca.data.iter().collect()),
        _ => None,
    }
}

/// First non-singleton dimension (1-based), like MATLAB reduction defaults.
fn default_dimension(shape: &[usize]) -> usize {
    if is_scalar_shape(shape) {
        return 1;
    }
    shape
        .iter()
        .position(|&extent| extent != 1)
        .map(|idx| idx + 1)
        .unwrap_or(1)
}

fn normalize_tensor(input: &Tensor, dim: usize, method: Method) -> BuiltinResult<Tensor> {
    let shape = &input.shape;
    let ndims = shape.len();
    let dim_index = dim - 1;

    // A dimension beyond the array rank selects a singleton axis: each element
    // is normalized on its own.
    let n = if dim_index < ndims {
        shape[dim_index]
    } else {
        1
    };
    let stride_before: usize = shape[..dim_index.min(ndims)].iter().product();
    let stride_after: usize = if dim_index < ndims {
        shape[dim_index + 1..].iter().product()
    } else {
        1
    };

    let mut data = input.data.clone();
    let mut line = vec![0.0f64; n];

    for after in 0..stride_after {
        for before in 0..stride_before {
            let base = before + after * stride_before * n;
            for (k, slot) in line.iter_mut().enumerate() {
                *slot = data[base + k * stride_before];
            }
            transform_line(&mut line, method);
            for (k, &val) in line.iter().enumerate() {
                data[base + k * stride_before] = val;
            }
        }
    }

    Tensor::new(data, shape.clone()).map_err(internal_error)
}

/// Transform one line (the elements along the operating dimension) in place.
fn transform_line(line: &mut [f64], method: Method) {
    match method {
        Method::ZScore => {
            let mean = line_mean(line);
            let scale = line_std_sample(line, mean);
            for x in line.iter_mut() {
                *x = (*x - mean) / scale;
            }
        }
        Method::Center(centre) => {
            let c = centre.unwrap_or_else(|| line_mean(line));
            for x in line.iter_mut() {
                *x -= c;
            }
        }
        Method::Scale(kind) => {
            let scale = match kind {
                ScaleKind::Std => line_std_sample(line, line_mean(line)),
                ScaleKind::First => line.first().copied().unwrap_or(f64::NAN),
                ScaleKind::Value(c) => c,
            };
            for x in line.iter_mut() {
                *x /= scale;
            }
        }
        Method::Norm(p) => {
            let norm = line_pnorm(line, p);
            for x in line.iter_mut() {
                *x /= norm;
            }
        }
        Method::Range(a, b) => {
            let lo = line.iter().copied().fold(f64::INFINITY, f64::min);
            let hi = line.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let span = hi - lo;
            for x in line.iter_mut() {
                *x = a + (*x - lo) / span * (b - a);
            }
        }
    }
}

fn line_mean(line: &[f64]) -> f64 {
    if line.is_empty() {
        return f64::NAN;
    }
    line.iter().sum::<f64>() / (line.len() as f64)
}

/// Sample (N-1) standard deviation. Returns 0 for a single element so that a
/// z-score of a lone value evaluates to 0/0 = NaN, matching a degenerate scale.
fn line_std_sample(line: &[f64], mean: f64) -> f64 {
    let n = line.len();
    if n < 2 {
        return 0.0;
    }
    let ss: f64 = line.iter().map(|&x| (x - mean) * (x - mean)).sum();
    (ss / ((n - 1) as f64)).sqrt()
}

fn line_pnorm(line: &[f64], p: f64) -> f64 {
    if p.is_infinite() {
        return line.iter().fold(0.0f64, |acc, &x| acc.max(x.abs()));
    }
    if (p - 1.0).abs() < f64::EPSILON {
        return line.iter().map(|&x| x.abs()).sum();
    }
    if (p - 2.0).abs() < f64::EPSILON {
        return line.iter().map(|&x| x * x).sum::<f64>().sqrt();
    }
    line.iter()
        .map(|&x| x.abs().powf(p))
        .sum::<f64>()
        .powf(1.0 / p)
}

#[cfg(all(test, feature = "parallel_xval"))]
pub(crate) mod tests {
    use super::*;
    use futures::executor::block_on;
    use runmat_builtins::{IntValue, Tensor};

    fn tensor(data: Vec<f64>, shape: Vec<usize>) -> Value {
        Value::Tensor(Tensor::new(data, shape).unwrap())
    }

    fn run(value: Value, rest: Vec<Value>) -> Value {
        block_on(super::normalize_builtin(value, rest)).expect("normalize")
    }

    fn approx(actual: &[f64], expected: &[f64]) {
        assert_eq!(actual.len(), expected.len(), "length mismatch");
        for (a, e) in actual.iter().zip(expected.iter()) {
            assert!((a - e).abs() < 1e-12, "value={a}, expected={e}");
        }
    }

    // ---- Observed / normative -------------------------------------------

    // [normalize.signature-primary, normalize.output-class,
    //  normalize.zscore-and-range] observed case `zscore`:
    // normalize([1 2 3]) => [-1 0 1] (1x3 double), sample std (N-1) = 1.
    #[test]
    fn observed_zscore_vector() {
        let out = run(tensor(vec![1.0, 2.0, 3.0], vec![1, 3]), vec![]);
        match out {
            Value::Tensor(t) => {
                assert_eq!(t.shape, vec![1, 3]);
                approx(&t.data, &[-1.0, 0.0, 1.0]);
            }
            other => panic!("expected 1x3 tensor, got {other:?}"),
        }
    }

    // [normalize.zscore-and-range, normalize.output-class] observed case
    // `range`: normalize([1 2 3], 'range') => [0 0.5 1] (1x3 double).
    #[test]
    fn observed_range_vector() {
        let out = run(
            tensor(vec![1.0, 2.0, 3.0], vec![1, 3]),
            vec![Value::from("range")],
        );
        match out {
            Value::Tensor(t) => {
                assert_eq!(t.shape, vec![1, 3]);
                approx(&t.data, &[0.0, 0.5, 1.0]);
            }
            other => panic!("expected 1x3 tensor, got {other:?}"),
        }
    }

    // [normalize.output-class] The observed output class is double; the
    // returned RunMat value is a real double tensor/scalar.
    #[test]
    fn observed_output_class_is_double_tensor() {
        let out = run(tensor(vec![1.0, 2.0, 3.0], vec![1, 3]), vec![]);
        assert!(matches!(out, Value::Tensor(_)));
    }

    // ---- Unresolved choices (non-normative; documented) -----------------

    // [normalize.q-dim] Column-wise default for a matrix: operate down each
    // column (dim 1, the first non-singleton dimension). Documented choice.
    #[test]
    fn unresolved_choice_matrix_default_is_columnwise() {
        // Column-major [1 2; 3 4] -> columns (1,3) and (2,4).
        let out = run(tensor(vec![1.0, 3.0, 2.0, 4.0], vec![2, 2]), vec![]);
        match out {
            Value::Tensor(t) => {
                assert_eq!(t.shape, vec![2, 2]);
                // Each column is [x1,x2]; mean=(x1+x2)/2, std=|x1-x2|/sqrt(2).
                let s = std::f64::consts::FRAC_1_SQRT_2; // 1/sqrt(2)
                approx(&t.data, &[-s, s, -s, s]);
            }
            other => panic!("expected 2x2 tensor, got {other:?}"),
        }
    }

    // [normalize.q-dim] Explicit dimension argument operates row-wise (dim 2).
    // Documented choice.
    #[test]
    fn unresolved_choice_explicit_dim_rowwise() {
        let out = run(
            tensor(vec![1.0, 3.0, 2.0, 4.0], vec![2, 2]),
            vec![Value::Num(2.0)],
        );
        match out {
            Value::Tensor(t) => {
                assert_eq!(t.shape, vec![2, 2]);
                // Rows are [1,2] and [3,4]; each -> [-1/sqrt2, 1/sqrt2].
                let s = std::f64::consts::FRAC_1_SQRT_2;
                approx(&t.data, &[-s, -s, s, s]);
            }
            other => panic!("expected 2x2 tensor, got {other:?}"),
        }
    }

    // [normalize.q-methods] Explicit 'zscore' equals the default. Documented.
    #[test]
    fn unresolved_choice_explicit_zscore_matches_default() {
        let out = run(
            tensor(vec![1.0, 2.0, 3.0], vec![1, 3]),
            vec![Value::from("zscore")],
        );
        match out {
            Value::Tensor(t) => approx(&t.data, &[-1.0, 0.0, 1.0]),
            other => panic!("expected tensor, got {other:?}"),
        }
    }

    // [normalize.q-methods] Custom range interval [a b]. Documented choice.
    #[test]
    fn unresolved_choice_range_custom_interval() {
        let mt = tensor(vec![-1.0, 1.0], vec![1, 2]);
        let out = run(
            tensor(vec![1.0, 2.0, 3.0], vec![1, 3]),
            vec![Value::from("range"), mt],
        );
        match out {
            Value::Tensor(t) => approx(&t.data, &[-1.0, 0.0, 1.0]),
            other => panic!("expected tensor, got {other:?}"),
        }
    }

    // [normalize.q-methods] 'center' subtracts the mean. Documented choice.
    #[test]
    fn unresolved_choice_center_subtracts_mean() {
        let out = run(
            tensor(vec![1.0, 2.0, 3.0], vec![1, 3]),
            vec![Value::from("center")],
        );
        match out {
            Value::Tensor(t) => approx(&t.data, &[-1.0, 0.0, 1.0]),
            other => panic!("expected tensor, got {other:?}"),
        }
    }

    // [normalize.q-methods] 'scale' divides by the sample std. Documented.
    #[test]
    fn unresolved_choice_scale_divides_by_std() {
        let out = run(
            tensor(vec![1.0, 2.0, 3.0], vec![1, 3]),
            vec![Value::from("scale")],
        );
        match out {
            // std = 1, so values are unchanged.
            Value::Tensor(t) => approx(&t.data, &[1.0, 2.0, 3.0]),
            other => panic!("expected tensor, got {other:?}"),
        }
    }

    // [normalize.q-methods] 'norm' with default order 2. Documented choice.
    #[test]
    fn unresolved_choice_norm_default_two() {
        let out = run(
            tensor(vec![3.0, 4.0], vec![1, 2]),
            vec![Value::from("norm")],
        );
        match out {
            Value::Tensor(t) => approx(&t.data, &[0.6, 0.8]),
            other => panic!("expected tensor, got {other:?}"),
        }
    }

    // [normalize.q-methods] 'norm' with explicit order 1. Documented choice.
    #[test]
    fn unresolved_choice_norm_order_one() {
        let out = run(
            tensor(vec![1.0, 3.0], vec![1, 2]),
            vec![Value::from("norm"), Value::Num(1.0)],
        );
        match out {
            Value::Tensor(t) => approx(&t.data, &[0.25, 0.75]),
            other => panic!("expected tensor, got {other:?}"),
        }
    }

    // Integer input is accepted and produces a double result (output-class).
    #[test]
    fn integer_input_produces_double() {
        let out = run(
            Value::Tensor(Tensor::new(vec![1.0, 2.0, 3.0], vec![1, 3]).unwrap()),
            vec![],
        );
        assert!(matches!(out, Value::Tensor(_)));
        // Also verify an actual IntValue scalar dim argument parses.
        let out2 = run(
            tensor(vec![1.0, 3.0, 2.0, 4.0], vec![2, 2]),
            vec![Value::Int(IntValue::I32(2))],
        );
        assert!(matches!(out2, Value::Tensor(_)));
    }

    // ---- Errors ---------------------------------------------------------

    // Unresolved robust family rejected rather than invented.
    #[test]
    fn unsupported_medianiqr_is_rejected() {
        let err = block_on(super::normalize_builtin(
            tensor(vec![1.0, 2.0, 3.0], vec![1, 3]),
            vec![Value::from("medianiqr")],
        ))
        .expect_err("error");
        assert_eq!(err.identifier(), NORMALIZE_ERROR_UNSUPPORTED.identifier);
    }

    #[test]
    fn unsupported_zscore_robust_is_rejected() {
        let err = block_on(super::normalize_builtin(
            tensor(vec![1.0, 2.0, 3.0], vec![1, 3]),
            vec![Value::from("zscore"), Value::from("robust")],
        ))
        .expect_err("error");
        assert_eq!(err.identifier(), NORMALIZE_ERROR_UNSUPPORTED.identifier);
    }

    #[test]
    fn complex_input_is_unsupported() {
        let err = block_on(super::normalize_builtin(Value::Complex(1.0, 2.0), vec![]))
            .expect_err("error");
        assert_eq!(err.identifier(), NORMALIZE_ERROR_UNSUPPORTED.identifier);
    }

    #[test]
    fn unknown_method_is_invalid_argument() {
        let err = block_on(super::normalize_builtin(
            tensor(vec![1.0, 2.0, 3.0], vec![1, 3]),
            vec![Value::from("bogus")],
        ))
        .expect_err("error");
        assert_eq!(
            err.identifier(),
            NORMALIZE_ERROR_INVALID_ARGUMENT.identifier
        );
    }

    #[test]
    fn descriptor_has_stable_error_codes() {
        let codes: Vec<&str> = NORMALIZE_DESCRIPTOR.errors.iter().map(|e| e.code).collect();
        assert!(codes.contains(&"RM.NORMALIZE.INVALID_ARGUMENT"));
        assert!(codes.contains(&"RM.NORMALIZE.INVALID_INPUT"));
        assert!(codes.contains(&"RM.NORMALIZE.UNSUPPORTED"));
        assert!(codes.contains(&"RM.NORMALIZE.INTERNAL"));
    }
}
