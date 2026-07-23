// Parallel clean-room implementation of `spdiags` (independently developed against the
// matlab-interface-spec black-box specs). 0-6-0 ships the DEFAULT implementation; this copy is
// retained unregistered for differential cross-validation. Original path: crates/runmat-runtime/src/builtins/math/sparse/spdiags.rs
//! MATLAB-compatible `spdiags` builtin for RunMat.
//!
//! Clean-room provenance: specs/009-sparse-construction (spec `spdiags`
//! 0.2.0, claims spdiags.signature-primary, spdiags.output-class,
//! spdiags.extract-vs-construct; the second extraction output is
//! summary-derived).
//!
//! Diagonal numbering rule (documented independent choice, reproduces the
//! observed cases exactly): element A(i, j) lies on diagonal d = j - i
//! (d = 0 main, d > 0 superdiagonals, d < 0 subdiagonals). The extracted
//! matrix B has min(m, n) rows and one column per nonzero diagonal, in
//! ascending d order. Column padding: for m >= n the element A(i, j) is
//! stored in row j of B (superdiagonals padded with zeros at the top,
//! subdiagonals at the bottom); for m < n it is stored in row i (the
//! opposite padding). The construction form applies the inverse mapping.

use std::collections::{BTreeMap, BTreeSet};

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    ResolveContext, SparseTensor, Tensor, Type, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::gpu_helpers;
use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ScalarType, ShapeRequirements,
};
use crate::{build_runtime_error, BuiltinResult, RuntimeError};

use super::support::{scalar_f64, size_from_f64};

pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "spdiags",
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
    notes: "Sparse matrices are host-resident CSC values; GPU inputs are gathered before diagonal extraction or construction.",
};

pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "spdiags",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Representation-changing diagonal reshuffle; not fusible.",
};

const SPDIAGS_EXTRACT_OUTPUTS: [BuiltinParamDescriptor; 2] = [
    BuiltinParamDescriptor {
        name: "B",
        ty: BuiltinParamType::NumericArray,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Nonzero diagonals of A as columns of a full matrix.",
    },
    BuiltinParamDescriptor {
        name: "d",
        ty: BuiltinParamType::NumericArray,
        arity: BuiltinParamArity::Optional,
        default: None,
        description: "Indices of the extracted diagonals (column vector).",
    },
];

const SPDIAGS_EXTRACT_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "A",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Matrix whose diagonals are read.",
}];

const SPDIAGS_CONSTRUCT_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "A",
    ty: BuiltinParamType::NumericArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Sparse m-by-n matrix built from the supplied diagonals.",
}];

const SPDIAGS_CONSTRUCT_INPUTS: [BuiltinParamDescriptor; 4] = [
    BuiltinParamDescriptor {
        name: "B",
        ty: BuiltinParamType::NumericArray,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "min(m,n)-by-p matrix of diagonal data.",
    },
    BuiltinParamDescriptor {
        name: "d",
        ty: BuiltinParamType::NumericArray,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Diagonal indices for the columns of B.",
    },
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

const SPDIAGS_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "[B, d] = spdiags(A)",
        inputs: &SPDIAGS_EXTRACT_INPUTS,
        outputs: &SPDIAGS_EXTRACT_OUTPUTS,
    },
    BuiltinSignatureDescriptor {
        label: "A = spdiags(B, d, m, n)",
        inputs: &SPDIAGS_CONSTRUCT_INPUTS,
        outputs: &SPDIAGS_CONSTRUCT_OUTPUT,
    },
];

const SPDIAGS_ERROR_INVALID_ARGUMENT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SPDIAGS.INVALID_ARGUMENT",
    identifier: Some("RunMat:spdiags:InvalidArgument"),
    when: "An unsupported call form is used (argument count other than 1 or 4).",
    message: "spdiags: only spdiags(A) and spdiags(B, d, m, n) are supported",
};

const SPDIAGS_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SPDIAGS.INVALID_INPUT",
    identifier: Some("RunMat:spdiags:InvalidInput"),
    when: "Inputs are not numeric matrices/vectors of consistent sizes.",
    message: "spdiags: invalid input",
};

const SPDIAGS_ERROR_INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SPDIAGS.INTERNAL",
    identifier: Some("RunMat:spdiags:Internal"),
    when: "Result materialisation fails internally.",
    message: "spdiags: internal error",
};

const SPDIAGS_ERRORS: [BuiltinErrorDescriptor; 3] = [
    SPDIAGS_ERROR_INVALID_ARGUMENT,
    SPDIAGS_ERROR_INVALID_INPUT,
    SPDIAGS_ERROR_INTERNAL,
];

pub const SPDIAGS_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &SPDIAGS_SIGNATURES,
    output_mode: BuiltinOutputMode::ByRequestedOutputCount,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &SPDIAGS_ERRORS,
};

async fn spdiags_builtin(value: Value, rest: Vec<Value>) -> BuiltinResult<Value> {
    let mut args = Vec::with_capacity(rest.len() + 1);
    args.push(gpu_helpers::gather_value_async(&value).await?);
    for arg in &rest {
        args.push(gpu_helpers::gather_value_async(arg).await?);
    }
    match args.len() {
        // Extraction form B = spdiags(A) [spdiags.signature-primary,
        // spdiags.extract-vs-construct].
        1 => extract_form(args.into_iter().next().expect("one argument")),
        // Construction form A = spdiags(B, d, m, n)
        // [spdiags.extract-vs-construct].
        4 => {
            let sparse = construct_form(args)?;
            let result = Value::SparseTensor(sparse);
            if let Some(out_count) = crate::output_count::current_output_count() {
                return Ok(crate::output_count::output_list_with_padding(
                    out_count,
                    vec![result],
                ));
            }
            Ok(result)
        }
        // Other call forms (spdiags(A, d), spdiags(B, d, A)) are unresolved
        // in the approved spec (spdiags.q-forms) and deliberately
        // unimplemented.
        _ => Err(invalid_argument()),
    }
}

fn spdiags_type(_args: &[Type], _context: &ResolveContext) -> Type {
    Type::tensor()
}

fn is_stored(value: f64) -> bool {
    value.is_nan() || value != 0.0
}

/// Extract all nonzero diagonals of A into a dense min(m,n)-by-p matrix,
/// diagonals sorted ascending. Second output (requested via output count)
/// is the p-by-1 vector of diagonal indices (summary-derived;
/// spdiags.q-outputs).
fn extract_form(a: Value) -> BuiltinResult<Value> {
    let (rows, cols, entries) = nonzero_entries(a)?;
    let diag_set: BTreeSet<i64> = entries
        .iter()
        .map(|&(row, col, _)| col as i64 - row as i64)
        .collect();
    let diag_index: BTreeMap<i64, usize> =
        diag_set.iter().enumerate().map(|(k, &d)| (d, k)).collect();
    let p = diag_set.len();
    let rows_b = rows.min(cols);

    let mut data = vec![0.0; rows_b.saturating_mul(p)];
    for (row, col, value) in &entries {
        let k = diag_index[&(*col as i64 - *row as i64)];
        let r = if rows >= cols { *col } else { *row };
        data[r + k * rows_b] = *value;
    }
    let b = Tensor::new(data, vec![rows_b, p]).map_err(internal_error)?;
    let d = Tensor::new(diag_set.iter().map(|&d| d as f64).collect(), vec![p, 1])
        .map_err(internal_error)?;

    let outputs = vec![Value::Tensor(b), Value::Tensor(d)];
    if let Some(out_count) = crate::output_count::current_output_count() {
        return Ok(crate::output_count::output_list_with_padding(
            out_count, outputs,
        ));
    }
    Ok(outputs.into_iter().next().expect("spdiags outputs"))
}

/// (rows, cols, entries) where each entry is (row, col, value).
type MatrixEntries = (usize, usize, Vec<(usize, usize, f64)>);

/// Enumerate the stored (row, col, value) entries of a matrix-like value in
/// column-major order, without densifying sparse inputs.
fn nonzero_entries(value: Value) -> BuiltinResult<MatrixEntries> {
    match value {
        Value::SparseTensor(sparse) => {
            let mut entries = Vec::with_capacity(sparse.nnz());
            for col in 0..sparse.cols {
                for idx in sparse.col_ptrs[col]..sparse.col_ptrs[col + 1] {
                    let v = sparse.values[idx];
                    if is_stored(v) {
                        entries.push((sparse.row_indices[idx], col, v));
                    }
                }
            }
            Ok((sparse.rows, sparse.cols, entries))
        }
        Value::Tensor(tensor) => {
            if tensor.shape.len() > 2 {
                return Err(invalid_input(format!(
                    "spdiags: input must be a 2-D matrix, got {}-D tensor",
                    tensor.shape.len()
                )));
            }
            let rows = tensor.rows();
            let cols = tensor.cols();
            let mut entries = Vec::new();
            for col in 0..cols {
                for row in 0..rows {
                    let v = tensor.data[row + col * rows];
                    if is_stored(v) {
                        entries.push((row, col, v));
                    }
                }
            }
            Ok((rows, cols, entries))
        }
        ref scalar if scalar_f64(scalar).is_some() => {
            let v = scalar_f64(scalar).expect("scalar");
            let entries = if is_stored(v) {
                vec![(0, 0, v)]
            } else {
                vec![]
            };
            Ok((1, 1, entries))
        }
        other => Err(invalid_input(format!(
            "spdiags: unsupported input {other:?}"
        ))),
    }
}

/// Build the m-by-n sparse matrix whose diagonal d(k) holds column k of B,
/// using the inverse of the extraction placement rule. Out-of-range
/// positions are ignored; zero values are not stored; a repeated diagonal
/// index lets the later column win (documented choices — only the single
/// main-diagonal case is observed).
fn construct_form(args: Vec<Value>) -> BuiltinResult<SparseTensor> {
    let mut iter = args.into_iter();
    let b_value = iter.next().expect("B");
    let d_value = iter.next().expect("d");
    let m_value = iter.next().expect("m");
    let n_value = iter.next().expect("n");

    let rows = parse_size(&m_value, "m")?;
    let cols = parse_size(&n_value, "n")?;
    let diags = diag_vector(&d_value)?;
    let b = dense_matrix(b_value)?;

    let min_dim = rows.min(cols);
    if b.rows() != min_dim || b.cols() != diags.len() {
        return Err(invalid_input(format!(
            "spdiags: B must be min(m,n)-by-length(d) ({}x{}), got {}x{}",
            min_dim,
            diags.len(),
            b.rows(),
            b.cols()
        )));
    }

    // (col, row) -> value; BTreeMap gives column-major assembly order.
    let mut entries: BTreeMap<(usize, usize), f64> = BTreeMap::new();
    for (k, &d) in diags.iter().enumerate() {
        if rows >= cols {
            for col in 0..cols {
                let row = col as i64 - d;
                if row < 0 || row >= rows as i64 {
                    continue;
                }
                entries.insert((col, row as usize), b.data[col + k * min_dim]);
            }
        } else {
            for row in 0..rows {
                let col = row as i64 + d;
                if col < 0 || col >= cols as i64 {
                    continue;
                }
                entries.insert((col as usize, row), b.data[row + k * min_dim]);
            }
        }
    }

    assemble_csc(rows, cols, &entries)
}

fn assemble_csc(
    rows: usize,
    cols: usize,
    entries: &BTreeMap<(usize, usize), f64>,
) -> BuiltinResult<SparseTensor> {
    let mut col_ptrs = Vec::with_capacity(cols.saturating_add(1));
    let mut row_indices = Vec::new();
    let mut values = Vec::new();
    col_ptrs.push(0);
    let mut current_col = 0usize;
    for (&(col, row), &value) in entries.iter() {
        if !is_stored(value) {
            continue;
        }
        while current_col < col {
            col_ptrs.push(values.len());
            current_col += 1;
        }
        row_indices.push(row);
        values.push(value);
    }
    while current_col < cols {
        col_ptrs.push(values.len());
        current_col += 1;
    }
    SparseTensor::new(rows, cols, col_ptrs, row_indices, values)
        .map_err(|err| internal_error(err.to_string()))
}

fn dense_matrix(value: Value) -> BuiltinResult<Tensor> {
    match value {
        Value::Tensor(tensor) => {
            if tensor.shape.len() > 2 {
                return Err(invalid_input(format!(
                    "spdiags: B must be a 2-D matrix, got {}-D tensor",
                    tensor.shape.len()
                )));
            }
            Ok(tensor)
        }
        Value::SparseTensor(sparse) => sparse.to_dense().map_err(internal_error),
        ref scalar if scalar_f64(scalar).is_some() => {
            Tensor::new(vec![scalar_f64(scalar).expect("scalar")], vec![1, 1])
                .map_err(internal_error)
        }
        other => Err(invalid_input(format!(
            "spdiags: B must be a numeric matrix, got {other:?}"
        ))),
    }
}

fn diag_vector(value: &Value) -> BuiltinResult<Vec<i64>> {
    let raw: Vec<f64> = match value {
        Value::Tensor(tensor) => {
            let non_unit = tensor.shape.iter().filter(|&&dim| dim != 1).count();
            if tensor.shape.len() > 2 || non_unit > 1 {
                return Err(invalid_input(
                    "spdiags: d must be a numeric vector".to_string(),
                ));
            }
            tensor.data.clone()
        }
        other => match scalar_f64(other) {
            Some(v) => vec![v],
            None => {
                return Err(invalid_input(
                    "spdiags: d must be a numeric vector".to_string(),
                ))
            }
        },
    };
    raw.into_iter()
        .map(|v| {
            if !v.is_finite() || v.fract() != 0.0 || v.abs() > i64::MAX as f64 / 2.0 {
                Err(invalid_input(format!(
                    "spdiags: diagonal indices must be integers, got {v}"
                )))
            } else {
                Ok(v as i64)
            }
        })
        .collect()
}

fn parse_size(value: &Value, name: &str) -> BuiltinResult<usize> {
    let raw = scalar_f64(value)
        .ok_or_else(|| invalid_input(format!("spdiags: {name} must be a scalar size")))?;
    size_from_f64(raw, name).map_err(|detail| invalid_input(format!("spdiags: {detail}")))
}

fn invalid_argument() -> RuntimeError {
    build_runtime_error(SPDIAGS_ERROR_INVALID_ARGUMENT.message)
        .with_builtin("spdiags")
        .with_identifier(
            SPDIAGS_ERROR_INVALID_ARGUMENT
                .identifier
                .expect("identifier"),
        )
        .build()
}

fn invalid_input(message: String) -> RuntimeError {
    build_runtime_error(message)
        .with_builtin("spdiags")
        .with_identifier(SPDIAGS_ERROR_INVALID_INPUT.identifier.expect("identifier"))
        .build()
}

fn internal_error(detail: String) -> RuntimeError {
    build_runtime_error(format!("spdiags: {detail}"))
        .with_builtin("spdiags")
        .with_identifier(SPDIAGS_ERROR_INTERNAL.identifier.expect("identifier"))
        .build()
}

#[cfg(all(test, feature = "parallel_xval"))]
pub(crate) mod tests {
    use super::*;
    use futures::executor::block_on;

    fn run_spdiags(value: Value, rest: Vec<Value>) -> Value {
        block_on(super::spdiags_builtin(value, rest)).expect("spdiags")
    }

    fn tensor(data: Vec<f64>, shape: Vec<usize>) -> Value {
        Value::Tensor(Tensor::new(data, shape).unwrap())
    }

    fn expect_tensor(value: Value) -> Tensor {
        match value {
            Value::Tensor(t) => t,
            other => panic!("expected dense tensor, got {other:?}"),
        }
    }

    fn expect_sparse(value: Value) -> SparseTensor {
        match value {
            Value::SparseTensor(sparse) => sparse,
            other => panic!("expected sparse tensor, got {other:?}"),
        }
    }

    // Normative [FR-009-07; spdiags.signature-primary,
    // spdiags.extract-vs-construct, spdiags.output-class] — observed case
    // `extract`: spdiags([1 2 0; 0 3 4; 0 0 5]) => [1 0;3 2;5 4]
    // (3x2 double, issparse=false).
    #[test]
    fn observed_extract_dense_diagonals() {
        // Column-major data for [1 2 0; 0 3 4; 0 0 5].
        let a = tensor(
            vec![1.0, 0.0, 0.0, 2.0, 3.0, 0.0, 0.0, 4.0, 5.0],
            vec![3, 3],
        );
        let b = expect_tensor(run_spdiags(a, vec![]));
        assert_eq!(b.shape, vec![3, 2]);
        // Column-major layout of [1 0;3 2;5 4].
        assert_eq!(b.data, vec![1.0, 3.0, 5.0, 0.0, 2.0, 4.0]);
    }

    // Normative [FR-009-08; spdiags.extract-vs-construct,
    // spdiags.output-class] — observed case `construct`:
    // spdiags([1; 2; 3], 0, 3, 3) => sparse([1;2;3], [1;2;3], [1;2;3], 3, 3)
    // (3x3 double, issparse=true).
    #[test]
    fn observed_construct_single_diagonal() {
        let b = tensor(vec![1.0, 2.0, 3.0], vec![3, 1]);
        let a = expect_sparse(run_spdiags(
            b,
            vec![Value::Num(0.0), Value::Num(3.0), Value::Num(3.0)],
        ));
        let expected =
            SparseTensor::new(3, 3, vec![0, 1, 2, 3], vec![0, 1, 2], vec![1.0, 2.0, 3.0]).unwrap();
        assert_eq!(a, expected);
    }

    // summary_derived [FR-009-09; spdiags.q-outputs]: the second extraction
    // output is the column vector of extracted diagonal indices, per the
    // approved summary of the extraction form; unobserved.
    #[test]
    fn summary_derived_second_output_diagonal_indices() {
        let _guard = crate::output_count::push_output_count(Some(2));
        let out = block_on(super::spdiags_builtin(
            tensor(
                vec![1.0, 0.0, 0.0, 2.0, 3.0, 0.0, 0.0, 4.0, 5.0],
                vec![3, 3],
            ),
            vec![],
        ))
        .expect("spdiags");
        drop(_guard);
        match out {
            Value::OutputList(values) => {
                assert_eq!(values.len(), 2);
                let b = match &values[0] {
                    Value::Tensor(t) => t.clone(),
                    other => panic!("expected tensor, got {other:?}"),
                };
                assert_eq!(b.data, vec![1.0, 3.0, 5.0, 0.0, 2.0, 4.0]);
                let d = match &values[1] {
                    Value::Tensor(t) => t.clone(),
                    other => panic!("expected tensor, got {other:?}"),
                };
                assert_eq!(d.shape, vec![2, 1]);
                assert_eq!(d.data, vec![0.0, 1.0]);
            }
            other => panic!("expected output list, got {other:?}"),
        }
    }

    // summary_derived [FR-009-07]: extraction from a sparse input follows
    // the same layout (the approved summary describes reading diagonals of a
    // sparse matrix; the observed extract case used a full input).
    #[test]
    fn summary_derived_extract_from_sparse_input() {
        // Sparse form of [1 2 0; 0 3 4; 0 0 5].
        let a = SparseTensor::new(
            3,
            3,
            vec![0, 1, 3, 5],
            vec![0, 0, 1, 1, 2],
            vec![1.0, 2.0, 3.0, 4.0, 5.0],
        )
        .unwrap();
        let b = expect_tensor(run_spdiags(Value::SparseTensor(a), vec![]));
        assert_eq!(b.shape, vec![3, 2]);
        assert_eq!(b.data, vec![1.0, 3.0, 5.0, 0.0, 2.0, 4.0]);
    }

    // unresolved_choice [FR-009-10]: general diagonal numbering and padding
    // rule on unobserved combinations — subdiagonals (d < 0) are padded at
    // the bottom for m >= n.
    #[test]
    fn unresolved_choice_subdiagonal_padding() {
        // Column-major data for [0 0 0; 7 0 0; 0 8 0].
        let a = tensor(
            vec![0.0, 7.0, 0.0, 0.0, 0.0, 8.0, 0.0, 0.0, 0.0],
            vec![3, 3],
        );
        let _guard = crate::output_count::push_output_count(Some(2));
        let out = block_on(super::spdiags_builtin(a, vec![])).expect("spdiags");
        drop(_guard);
        match out {
            Value::OutputList(values) => {
                let b = match &values[0] {
                    Value::Tensor(t) => t.clone(),
                    other => panic!("expected tensor, got {other:?}"),
                };
                assert_eq!(b.shape, vec![3, 1]);
                assert_eq!(b.data, vec![7.0, 8.0, 0.0]);
                let d = match &values[1] {
                    Value::Tensor(t) => t.clone(),
                    other => panic!("expected tensor, got {other:?}"),
                };
                assert_eq!(d.data, vec![-1.0]);
            }
            other => panic!("expected output list, got {other:?}"),
        }
    }

    // unresolved_choice [FR-009-10]: for m < n the placement uses the row
    // index (opposite padding); construction is the exact inverse of
    // extraction (round trip).
    #[test]
    fn unresolved_choice_wide_matrix_round_trip() {
        // Construct: B = [9; 6] on diagonal d = 1 of a 2x3 matrix.
        let b = tensor(vec![9.0, 6.0], vec![2, 1]);
        let a = expect_sparse(run_spdiags(
            b,
            vec![Value::Num(1.0), Value::Num(2.0), Value::Num(3.0)],
        ));
        // Expected: A = [0 9 0; 0 0 6].
        let expected =
            SparseTensor::new(2, 3, vec![0, 0, 1, 2], vec![0, 1], vec![9.0, 6.0]).unwrap();
        assert_eq!(a, expected);

        // Extract reproduces the original diagonal data.
        let back = expect_tensor(run_spdiags(Value::SparseTensor(a), vec![]));
        assert_eq!(back.shape, vec![2, 1]);
        assert_eq!(back.data, vec![9.0, 6.0]);
    }

    // unresolved_choice [FR-009-10; spdiags.q-forms]: other call forms are
    // deliberately unimplemented.
    #[test]
    fn unresolved_choice_other_forms_error() {
        let err = block_on(super::spdiags_builtin(
            tensor(vec![1.0], vec![1, 1]),
            vec![Value::Num(0.0)],
        ))
        .expect_err("error");
        assert_eq!(err.identifier(), Some("RunMat:spdiags:InvalidArgument"));
    }

    #[test]
    fn construct_size_mismatch_errors() {
        let b = tensor(vec![1.0, 2.0], vec![2, 1]);
        let err = block_on(super::spdiags_builtin(
            b,
            vec![Value::Num(0.0), Value::Num(3.0), Value::Num(3.0)],
        ))
        .expect_err("error");
        assert_eq!(err.identifier(), Some("RunMat:spdiags:InvalidInput"));
        assert!(err.message().contains("min(m,n)-by-length(d)"));
    }
}
