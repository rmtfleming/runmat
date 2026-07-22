//! MATLAB-compatible `colamd` builtin for RunMat.
//!
//! Clean-room provenance: specs/027-sparse-ordering (spec `colamd` 0.2.0,
//! claims colamd.signature-primary, colamd.output-class,
//! colamd.permutation-vector). The approved export asserts only that the
//! result is a class-`double`, 1×n row vector that is a permutation of 1..n;
//! it deliberately does NOT assert the specific permutation values (the
//! observed `[2 1 3]` is algorithm-dependent — see colamd.q-values). The
//! ordering produced here is therefore an INDEPENDENT choice, not a
//! reproduction of MATLAB / SuiteSparse COLAMD.
//!
//! # External algorithm source (Constitution Principle X)
//!
//! Fill-reducing heuristic: the classic **minimum-degree** ordering applied to
//! the column-intersection graph of the sparse matrix (the graph whose
//! vertices are the columns and in which two columns are adjacent iff they
//! share a nonzero row — equivalently the sparsity graph of `Aᵀ·A`). This is
//! the Markowitz / minimum-degree scheme, independently implemented from the
//! published descriptions:
//!
//! - C. A. Tinney and J. W. Walker, "Direct solutions of sparse network
//!   equations by optimally ordered triangular factorization," Proceedings of
//!   the IEEE, vol. 55, no. 11, pp. 1801–1809, 1967 (minimum-degree / "scheme
//!   2" ordering).
//! - A. George and J. W. H. Liu, "Computer Solution of Large Sparse Positive
//!   Definite Systems," Prentice-Hall, 1981 (minimum-degree algorithm; column
//!   intersection / normal-equations graph).
//!
//! This is an exact minimum-degree elimination on an explicitly formed
//! intersection graph. It is deliberately DISTINCT from SuiteSparse COLAMD,
//! which never forms `Aᵀ·A` and instead uses approximate external degrees with
//! supernode absorption; no MathWorks or SuiteSparse source, prose, or example
//! was consulted. Licence: both sources are published algorithm descriptions
//! (journal / textbook); the Rust code below is an original implementation and
//! carries no third-party licence obligation.

use std::collections::HashSet;

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
    ResolveContext, Tensor, Type, Value,
};
use runmat_macros::runtime_builtin;

use crate::builtins::common::gpu_helpers;
use crate::builtins::common::spec::{
    BroadcastSemantics, BuiltinFusionSpec, BuiltinGpuSpec, ConstantStrategy, GpuOpKind,
    ReductionNaN, ResidencyPolicy, ScalarType, ShapeRequirements,
};
use crate::{build_runtime_error, BuiltinResult, RuntimeError};

use super::matrix_pattern;

#[runmat_macros::register_gpu_spec(builtin_path = "crate::builtins::math::sparse::colamd")]
pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "colamd",
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
    notes: "Graph ordering runs on the host over CSC structure; GPU tensor inputs are gathered before the pattern scan.",
};

#[runmat_macros::register_fusion_spec(builtin_path = "crate::builtins::math::sparse::colamd")]
pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "colamd",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Data-dependent combinatorial ordering; not fusible.",
};

const COLAMD_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "p",
    ty: BuiltinParamType::NumericArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Column permutation row vector (1×n, a permutation of 1..n).",
}];

const COLAMD_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "S",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Sparse or full matrix.",
}];

const COLAMD_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "p = colamd(S)",
    inputs: &COLAMD_INPUTS,
    outputs: &COLAMD_OUTPUT,
}];

const COLAMD_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.COLAMD.INVALID_INPUT",
    identifier: Some("RunMat:colamd:InvalidInput"),
    when: "The input is not a 2-D numeric, logical, or sparse matrix.",
    message: "colamd: input must be a 2-D numeric matrix (full or sparse)",
};

const COLAMD_ERROR_INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.COLAMD.INTERNAL",
    identifier: Some("RunMat:colamd:Internal"),
    when: "Result materialisation fails internally.",
    message: "colamd: internal error",
};

const COLAMD_ERRORS: [BuiltinErrorDescriptor; 2] =
    [COLAMD_ERROR_INVALID_INPUT, COLAMD_ERROR_INTERNAL];

pub const COLAMD_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &COLAMD_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &COLAMD_ERRORS,
};

#[runtime_builtin(
    name = "colamd",
    category = "math/sparse",
    summary = "Column fill-reducing permutation of a sparse matrix (minimum-degree ordering).",
    keywords = "colamd,sparse,permutation,ordering,minimum degree,fill-in",
    accel = "custom",
    type_resolver(colamd_type),
    descriptor(crate::builtins::math::sparse::colamd::COLAMD_DESCRIPTOR),
    builtin_path = "crate::builtins::math::sparse::colamd"
)]
async fn colamd_builtin(value: Value) -> BuiltinResult<Value> {
    // GPU tensors are gathered to host storage before the structural scan;
    // sparse and dense host values pass through unchanged.
    let gathered = gpu_helpers::gather_value_async(&value).await?;
    // Read the (rows, cols) and nonzero coordinates from either CSC sparse
    // storage or dense column-major storage [colamd.signature-primary].
    let (rows, cols, coords) = matrix_pattern(&gathered).map_err(invalid_input)?;
    // Column-intersection graph on the `cols` column-vertices: two columns are
    // adjacent iff some row is nonzero in both.
    let adjacency = column_intersection_graph(rows, cols, &coords);
    // Independent fill-reducing heuristic: minimum-degree elimination order
    // [colamd.permutation-vector; the specific order is non-normative,
    // colamd.q-values].
    let order = minimum_degree_order(cols, adjacency);
    permutation_value(order).map_err(internal_error)
}

fn colamd_type(_args: &[Type], _context: &ResolveContext) -> Type {
    // Output is a 1×n double row vector (n = number of columns); the length is
    // data-dependent, so only the row orientation is known statically.
    Type::Tensor {
        shape: Some(vec![Some(1), None]),
    }
}

/// Build the undirected column-intersection graph: vertex set is the `cols`
/// columns, and columns `i` and `j` are adjacent iff at least one row has a
/// nonzero in both. This is the sparsity pattern of `Aᵀ·A` with the diagonal
/// removed.
fn column_intersection_graph(
    rows: usize,
    cols: usize,
    coords: &[(usize, usize)],
) -> Vec<HashSet<usize>> {
    let mut cols_in_row: Vec<Vec<usize>> = vec![Vec::new(); rows];
    for &(r, c) in coords {
        if r < rows && c < cols {
            cols_in_row[r].push(c);
        }
    }
    let mut adjacency: Vec<HashSet<usize>> = vec![HashSet::new(); cols];
    for row_cols in &cols_in_row {
        for i in 0..row_cols.len() {
            for j in (i + 1)..row_cols.len() {
                let (a, b) = (row_cols[i], row_cols[j]);
                if a != b {
                    adjacency[a].insert(b);
                    adjacency[b].insert(a);
                }
            }
        }
    }
    adjacency
}

/// Classic minimum-degree elimination ordering (Tinney & Walker 1967; George &
/// Liu 1981). Repeatedly eliminates a remaining vertex of minimum current
/// degree — ties broken by smallest index for determinism — adding fill edges
/// that make the eliminated vertex's neighbours a clique. Returns the
/// elimination order as zero-based vertex indices.
fn minimum_degree_order(n: usize, mut adjacency: Vec<HashSet<usize>>) -> Vec<usize> {
    let mut eliminated = vec![false; n];
    let mut order = Vec::with_capacity(n);
    for _ in 0..n {
        // Pick the remaining vertex of minimum degree (adjacency sets only ever
        // hold non-eliminated neighbours, so `len()` is the current degree).
        let mut chosen = None;
        let mut best_degree = usize::MAX;
        for v in 0..n {
            if eliminated[v] {
                continue;
            }
            let degree = adjacency[v].len();
            if degree < best_degree {
                best_degree = degree;
                chosen = Some(v);
            }
        }
        let v = chosen.expect("a remaining vertex exists while order is incomplete");
        order.push(v);
        eliminated[v] = true;
        let neighbours: Vec<usize> = adjacency[v].iter().copied().collect();
        for &u in &neighbours {
            adjacency[u].remove(&v);
        }
        // Fill edges: connect every pair of the eliminated vertex's neighbours.
        for i in 0..neighbours.len() {
            for j in (i + 1)..neighbours.len() {
                let (a, b) = (neighbours[i], neighbours[j]);
                adjacency[a].insert(b);
                adjacency[b].insert(a);
            }
        }
        adjacency[v].clear();
    }
    order
}

/// Materialise a zero-based ordering as a 1×n double row vector of one-based
/// permutation indices [colamd.output-class, colamd.permutation-vector]. An
/// empty ordering yields a 1×0 empty row (empty matrix → empty permutation).
/// A 1×1 result is kept as an explicit 1×1 tensor so the output is uniformly a
/// 1×n row vector (documented choice).
fn permutation_value(order: Vec<usize>) -> Result<Value, String> {
    let n = order.len();
    let data: Vec<f64> = order.into_iter().map(|v| (v + 1) as f64).collect();
    Tensor::new(data, vec![1, n]).map(Value::Tensor)
}

fn invalid_input(detail: String) -> RuntimeError {
    build_runtime_error(format!("colamd: {detail}"))
        .with_builtin("colamd")
        .with_identifier(COLAMD_ERROR_INVALID_INPUT.identifier.expect("identifier"))
        .build()
}

fn internal_error(detail: String) -> RuntimeError {
    build_runtime_error(format!("colamd: {detail}"))
        .with_builtin("colamd")
        .with_identifier(COLAMD_ERROR_INTERNAL.identifier.expect("identifier"))
        .build()
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use futures::executor::block_on;
    use runmat_builtins::{builtin_function_by_name, CharArray, LogicalArray, SparseTensor};

    fn run_colamd(value: Value) -> Value {
        block_on(super::colamd_builtin(value)).expect("colamd")
    }

    /// Extract the (shape, values) of a permutation result regardless of the
    /// concrete `Value` wrapper, and assert it is a class-`double`, 1×n row
    /// vector that is a valid permutation of 1..n. This encodes the ONLY
    /// normative assertions [colamd.output-class, colamd.permutation-vector];
    /// the specific ordering is never asserted (colamd.q-values).
    fn assert_valid_permutation(value: &Value, n: usize) -> Vec<usize> {
        let tensor = match value {
            Value::Tensor(t) => t.clone(),
            other => panic!("expected dense double row vector, got {other:?}"),
        };
        // class double, size 1×n.
        assert_eq!(
            tensor.shape,
            vec![1, n],
            "output must be a 1×{n} row vector"
        );
        assert_eq!(tensor.data.len(), n);
        // Valid permutation of 1..n: each index appears exactly once.
        let mut seen = vec![false; n];
        let mut perm = Vec::with_capacity(n);
        for &raw in &tensor.data {
            assert_eq!(raw.fract(), 0.0, "permutation entries must be integers");
            let idx = raw as usize;
            assert!(idx >= 1 && idx <= n, "index {idx} out of range 1..{n}");
            assert!(!seen[idx - 1], "index {idx} repeated — not a permutation");
            seen[idx - 1] = true;
            perm.push(idx);
        }
        perm
    }

    fn observed_matrix() -> SparseTensor {
        // sparse([1 0 1; 0 1 0; 1 0 1]) in CSC form.
        // col0: rows {0,2}; col1: rows {1}; col2: rows {0,2}.
        SparseTensor::new(
            3,
            3,
            vec![0, 2, 3, 5],
            vec![0, 2, 1, 0, 2],
            vec![1.0, 1.0, 1.0, 1.0, 1.0],
        )
        .unwrap()
    }

    // Normative [FR-027-01, FR-027-02; colamd.output-class,
    // colamd.permutation-vector] — the ONLY asserted properties on the observed
    // input `sparse([1 0 1; 0 1 0; 1 0 1])` are class double, size 1×3, valid
    // permutation of 1..3. The specific values are NOT asserted here.
    #[test]
    fn observed_input_returns_valid_permutation() {
        let result = run_colamd(Value::SparseTensor(observed_matrix()));
        assert_valid_permutation(&result, 3);
    }

    // documented_choice [FR-027-04; colamd.q-values] — NON-NORMATIVE record of
    // THIS implementation's minimum-degree ordering on the observed input. The
    // observed MATLAB value `[2 1 3]` is algorithm-dependent and is not a
    // requirement; this test documents our output for traceability only. (It
    // happens to coincide with `[2 1 3]` here, but that coincidence is not
    // relied upon.)
    #[test]
    fn documented_choice_ordering_on_observed_input() {
        let result = run_colamd(Value::SparseTensor(observed_matrix()));
        let Value::Tensor(t) = result else {
            panic!("expected tensor");
        };
        assert_eq!(t.data, vec![2.0, 1.0, 3.0]);
    }

    // Normative [colamd.permutation-vector] — a dense matrix is accepted by
    // reading its structure; the result is still a valid permutation.
    #[test]
    fn dense_input_is_accepted_and_valid() {
        // [1 0 1; 0 1 0; 1 0 1] column-major.
        let dense = Tensor::new(
            vec![1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0],
            vec![3, 3],
        )
        .unwrap();
        let result = run_colamd(Value::Tensor(dense));
        assert_valid_permutation(&result, 3);
    }

    // Normative [colamd.permutation-vector] — a larger, rectangular, less
    // structured pattern still yields a valid permutation of 1..cols.
    #[test]
    fn rectangular_input_valid_permutation() {
        // 4×5 pattern (rows can exceed cols; colamd permutes the 5 columns).
        // col0:{0,3} col1:{1} col2:{0,2} col3:{2,3} col4:{1,3}
        let sparse = SparseTensor::new(
            4,
            5,
            vec![0, 2, 3, 5, 7, 9],
            vec![0, 3, 1, 0, 2, 2, 3, 1, 3],
            vec![1.0; 9],
        )
        .unwrap();
        let result = run_colamd(Value::SparseTensor(sparse));
        assert_valid_permutation(&result, 5);
    }

    // Normative [colamd.permutation-vector] — empty matrix → empty permutation
    // (1×0 double).
    #[test]
    fn empty_matrix_yields_empty_permutation() {
        let result = run_colamd(Value::SparseTensor(SparseTensor::zeros(0, 0)));
        assert_valid_permutation(&result, 0);
    }

    // documented_choice [FR-027-04] — a 1×1 input yields the single-element
    // permutation [1] as a 1×1 row (documented shape choice).
    #[test]
    fn documented_choice_scalar_input() {
        let result = run_colamd(Value::Num(7.0));
        let perm = assert_valid_permutation(&result, 1);
        assert_eq!(perm, vec![1]);
    }

    // documented_choice [FR-027-05] — a logical matrix is accepted as a
    // structural pattern (unobserved input class; documented choice).
    #[test]
    fn documented_choice_logical_input_accepted() {
        let logical = LogicalArray::new(vec![1, 0, 0, 1], vec![2, 2]).unwrap();
        let result = run_colamd(Value::LogicalArray(logical));
        assert_valid_permutation(&result, 2);
    }

    // documented_choice [FR-027-05] — non-matrix inputs are rejected with the
    // stable InvalidInput identifier (documented choice; unobserved).
    #[test]
    fn documented_choice_non_matrix_input_errors() {
        let err = block_on(super::colamd_builtin(Value::CharArray(CharArray::new_row(
            "ab",
        ))))
        .expect_err("error");
        assert_eq!(err.identifier(), Some("RunMat:colamd:InvalidInput"));
    }

    // descriptor / macro registration test — the builtin is discoverable via
    // the compile-time inventory registry with its documented signature.
    #[test]
    fn descriptor_is_registered() {
        let builtin = builtin_function_by_name("colamd").expect("colamd builtin registered");
        let descriptor = builtin.descriptor.expect("descriptor present");
        assert!(descriptor
            .signatures
            .iter()
            .any(|sig| sig.label == "p = colamd(S)"));
    }

    // GPU spec / fusion spec descriptor test — sparse ordering is a host-only
    // custom op and is not fusible.
    #[test]
    fn gpu_and_fusion_specs_are_custom_and_non_fusible() {
        assert_eq!(GPU_SPEC.name, "colamd");
        assert!(matches!(GPU_SPEC.op_kind, GpuOpKind::Custom("sparse")));
        assert_eq!(FUSION_SPEC.name, "colamd");
        assert!(FUSION_SPEC.elementwise.is_none());
        assert!(FUSION_SPEC.reduction.is_none());
    }

    // type-resolver test — output type is a 1×n double row vector.
    #[test]
    fn type_resolver_is_row_vector() {
        let ty = colamd_type(&[Type::tensor()], &ResolveContext::new(Vec::new()));
        assert_eq!(
            ty,
            Type::Tensor {
                shape: Some(vec![Some(1), None])
            }
        );
    }
}
