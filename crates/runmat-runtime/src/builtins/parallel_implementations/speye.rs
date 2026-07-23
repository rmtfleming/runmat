// Parallel clean-room implementation of `speye` (independently developed against the
// matlab-interface-spec black-box specs). 0-6-0 ships the DEFAULT implementation; this copy is
// retained unregistered for differential cross-validation. Original path: crates/runmat-runtime/src/builtins/math/sparse/speye.rs
//! MATLAB-compatible `speye` builtin for RunMat.
//!
//! Clean-room provenance: specs/009-sparse-construction (spec `speye` 0.2.0,
//! claims speye.signature-primary, speye.output-class,
//! speye.sparse-identity).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    ResolveContext, SparseTensor, Type, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ScalarType, ShapeRequirements,
};
use crate::{build_runtime_error, BuiltinResult, RuntimeError};

use super::support::{scalar_f64, size_from_f64};

pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "speye",
    op_kind: GpuOpKind::Custom("sparse"),
    supported_precisions: &[ScalarType::F32, ScalarType::F64],
    broadcast: BroadcastSemantics::None,
    provider_hooks: &[],
    constant_strategy: ConstantStrategy::InlineLiteral,
    residency: ResidencyPolicy::GatherImmediately,
    nan_mode: ReductionNaN::Include,
    two_pass_threshold: None,
    workgroup_size: None,
    accepts_nan_mode: false,
    notes: "Creates host-resident CSC identity values; no GPU execution path.",
};

pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "speye",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Sparse constructor; not fusible.",
};

const SPEYE_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "S",
    ty: BuiltinParamType::NumericArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Sparse matrix with ones on the main diagonal.",
}];

const SPEYE_INPUT_N: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "n",
    ty: BuiltinParamType::SizeArg,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Number of rows and columns.",
}];

const SPEYE_INPUT_MN: [BuiltinParamDescriptor; 2] = [
    BuiltinParamDescriptor {
        name: "m",
        ty: BuiltinParamType::SizeArg,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Number of rows.",
    },
    BuiltinParamDescriptor {
        name: "n",
        ty: BuiltinParamType::SizeArg,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Number of columns.",
    },
];

const SPEYE_INPUT_SZ: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "sz",
    ty: BuiltinParamType::SizeArg,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Two-element size vector [m n].",
}];

const SPEYE_SIGNATURES: [BuiltinSignatureDescriptor; 3] = [
    BuiltinSignatureDescriptor {
        label: "S = speye(n)",
        inputs: &SPEYE_INPUT_N,
        outputs: &SPEYE_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "S = speye(m, n)",
        inputs: &SPEYE_INPUT_MN,
        outputs: &SPEYE_OUTPUT,
    },
    BuiltinSignatureDescriptor {
        label: "S = speye(sz)",
        inputs: &SPEYE_INPUT_SZ,
        outputs: &SPEYE_OUTPUT,
    },
];

const SPEYE_ERROR_INVALID_ARGUMENT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SPEYE.INVALID_ARGUMENT",
    identifier: Some("RunMat:speye:InvalidArgument"),
    when: "Arguments are not a scalar size, an (m, n) pair, or a two-element size vector.",
    message: "speye: expected speye(n), speye(m, n), or speye([m n])",
};

const SPEYE_ERROR_INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SPEYE.INTERNAL",
    identifier: Some("RunMat:speye:Internal"),
    when: "Sparse identity materialisation fails internally.",
    message: "speye: internal error",
};

const SPEYE_ERRORS: [BuiltinErrorDescriptor; 2] =
    [SPEYE_ERROR_INVALID_ARGUMENT, SPEYE_ERROR_INTERNAL];

pub const SPEYE_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &SPEYE_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &SPEYE_ERRORS,
};

async fn speye_builtin(value: Value, rest: Vec<Value>) -> BuiltinResult<Value> {
    let (rows, cols) = match rest.len() {
        // speye(n) or speye([m n]) [speye.signature-primary,
        // speye.sparse-identity, observed cases `square`, `size-vector`,
        // `zero`].
        0 => dims_from_single(&value)?,
        // speye(m, n) [speye.sparse-identity, observed case `rectangular`].
        1 => {
            let rows = parse_size(&value, "m")?;
            let cols = parse_size(&rest[0], "n")?;
            (rows, cols)
        }
        _ => return Err(invalid_argument()),
    };
    Ok(Value::SparseTensor(sparse_identity(rows, cols)?))
}

fn speye_type(_args: &[Type], _context: &ResolveContext) -> Type {
    Type::tensor()
}

/// One argument: scalar n => n-by-n; two-element numeric vector => [m n].
/// Vectors with any other element count are rejected (documented choice;
/// only the two-element form was observed).
fn dims_from_single(value: &Value) -> BuiltinResult<(usize, usize)> {
    if let Value::Tensor(t) = value {
        if t.data.len() == 2 {
            let rows = size_from_f64(t.data[0], "m").map_err(invalid_argument_detail)?;
            let cols = size_from_f64(t.data[1], "n").map_err(invalid_argument_detail)?;
            return Ok((rows, cols));
        }
    }
    let n = parse_size(value, "n")?;
    Ok((n, n))
}

fn parse_size(value: &Value, name: &str) -> BuiltinResult<usize> {
    let raw = scalar_f64(value).ok_or_else(invalid_argument)?;
    size_from_f64(raw, name).map_err(invalid_argument_detail)
}

/// Build the m-by-n CSC identity directly: one unit entry at (i, i) for
/// i < min(m, n).
fn sparse_identity(rows: usize, cols: usize) -> BuiltinResult<SparseTensor> {
    let diag = rows.min(cols);
    let mut col_ptrs = Vec::with_capacity(cols.saturating_add(1));
    col_ptrs.push(0usize);
    for col in 0..cols {
        col_ptrs.push(diag.min(col + 1));
    }
    SparseTensor::new(rows, cols, col_ptrs, (0..diag).collect(), vec![1.0; diag])
        .map_err(|err| internal_error(err.to_string()))
}

fn invalid_argument() -> RuntimeError {
    build_runtime_error(SPEYE_ERROR_INVALID_ARGUMENT.message)
        .with_builtin("speye")
        .with_identifier(SPEYE_ERROR_INVALID_ARGUMENT.identifier.expect("identifier"))
        .build()
}

fn invalid_argument_detail(detail: String) -> RuntimeError {
    build_runtime_error(format!("speye: {detail}"))
        .with_builtin("speye")
        .with_identifier(SPEYE_ERROR_INVALID_ARGUMENT.identifier.expect("identifier"))
        .build()
}

fn internal_error(detail: String) -> RuntimeError {
    build_runtime_error(format!("speye: {detail}"))
        .with_builtin("speye")
        .with_identifier(SPEYE_ERROR_INTERNAL.identifier.expect("identifier"))
        .build()
}

#[cfg(all(test, feature = "parallel_xval"))]
pub(crate) mod tests {
    use super::*;
    use futures::executor::block_on;
    use runmat_builtins::Tensor;

    fn run_speye(value: Value, rest: Vec<Value>) -> Value {
        block_on(super::speye_builtin(value, rest)).expect("speye")
    }

    fn expect_sparse(value: Value) -> SparseTensor {
        match value {
            Value::SparseTensor(sparse) => sparse,
            other => panic!("expected sparse tensor, got {other:?}"),
        }
    }

    // Normative [FR-009-04; speye.signature-primary, speye.sparse-identity,
    // speye.output-class] — observed case `square`: speye(3) =>
    // sparse([1;2;3], [1;2;3], [1;1;1], 3, 3) (3x3 double, issparse=true).
    #[test]
    fn observed_square_identity() {
        let sparse = expect_sparse(run_speye(Value::Num(3.0), vec![]));
        let expected =
            SparseTensor::new(3, 3, vec![0, 1, 2, 3], vec![0, 1, 2], vec![1.0, 1.0, 1.0]).unwrap();
        assert_eq!(sparse, expected);
    }

    // Normative — observed case `rectangular`: speye(2, 4) =>
    // sparse([1;2], [1;2], [1;1], 2, 4) (2x4 double, issparse=true).
    #[test]
    fn observed_rectangular_identity() {
        let sparse = expect_sparse(run_speye(Value::Num(2.0), vec![Value::Num(4.0)]));
        let expected =
            SparseTensor::new(2, 4, vec![0, 1, 2, 2, 2], vec![0, 1], vec![1.0, 1.0]).unwrap();
        assert_eq!(sparse, expected);
    }

    // Normative — observed case `size-vector`: speye([3 3]) => same as
    // speye(3) (3x3 double, issparse=true).
    #[test]
    fn observed_size_vector_identity() {
        let sz = Tensor::new(vec![3.0, 3.0], vec![1, 2]).unwrap();
        let sparse = expect_sparse(run_speye(Value::Tensor(sz), vec![]));
        let expected =
            SparseTensor::new(3, 3, vec![0, 1, 2, 3], vec![0, 1, 2], vec![1.0, 1.0, 1.0]).unwrap();
        assert_eq!(sparse, expected);
    }

    // Normative — observed case `zero`: speye(0) =>
    // sparse([], [], [], 0, 0) (0x0 double, issparse=true).
    #[test]
    fn observed_zero_size_identity() {
        let sparse = expect_sparse(run_speye(Value::Num(0.0), vec![]));
        assert_eq!(sparse, SparseTensor::zeros(0, 0));
    }

    // unresolved_choice [FR-009-06; speye.q-class]: non-integer, negative,
    // and wrong-length size inputs are rejected (documented independent
    // choice; unobserved input classes).
    #[test]
    fn unresolved_choice_invalid_sizes_error() {
        let err = block_on(super::speye_builtin(Value::Num(2.5), vec![])).expect_err("error");
        assert_eq!(err.identifier(), Some("RunMat:speye:InvalidArgument"));

        let err = block_on(super::speye_builtin(Value::Num(-1.0), vec![])).expect_err("error");
        assert_eq!(err.identifier(), Some("RunMat:speye:InvalidArgument"));

        let sz = Tensor::new(vec![2.0, 2.0, 2.0], vec![1, 3]).unwrap();
        let err = block_on(super::speye_builtin(Value::Tensor(sz), vec![])).expect_err("error");
        assert_eq!(err.identifier(), Some("RunMat:speye:InvalidArgument"));
    }

    #[test]
    fn too_many_arguments_error() {
        let err = block_on(super::speye_builtin(
            Value::Num(2.0),
            vec![Value::Num(2.0), Value::Num(2.0)],
        ))
        .expect_err("error");
        assert_eq!(err.identifier(), Some("RunMat:speye:InvalidArgument"));
    }
}
