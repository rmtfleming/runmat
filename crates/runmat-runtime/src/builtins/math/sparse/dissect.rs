//! MATLAB-compatible `dissect` builtin for RunMat.
//!
//! Clean-room provenance: specs/027-sparse-ordering (spec `dissect` 0.2.0,
//! claims dissect.signature-primary, dissect.output-class,
//! dissect.permutation-vector). The approved export asserts only that the
//! result is a class-`double`, 1×n row vector that is a permutation of 1..n;
//! it deliberately does NOT assert the specific permutation values (the
//! observed `[3 1 2]` is algorithm-dependent — see dissect.q-values). The
//! ordering produced here is therefore an INDEPENDENT choice, not a
//! reproduction of MATLAB's nested dissection.
//!
//! # External algorithm source (Constitution Principle X)
//!
//! Fill-reducing heuristic: a basic **recursive nested dissection** driven by
//! level-structure graph bisection, independently implemented from the
//! published descriptions:
//!
//! - A. George, "Nested Dissection of a Regular Finite Element Mesh," SIAM
//!   Journal on Numerical Analysis, vol. 10, no. 2, pp. 345–363, 1973 (nested
//!   dissection: number the two halves first, the separator last, recurse).
//! - A. George and J. W. H. Liu, "Computer Solution of Large Sparse Positive
//!   Definite Systems," Prentice-Hall, 1981 (level structures; separators).
//! - N. E. Gibbs, W. G. Poole, and P. K. Stockmeyer, "An algorithm for
//!   reducing the bandwidth and profile of a sparse matrix," SIAM J. Numer.
//!   Anal., vol. 13, no. 2, pp. 236–250, 1976 (pseudo-peripheral start vertex).
//!
//! The separator is an entire breadth-first level of the rooted level
//! structure: because BFS levels place no edge between non-adjacent levels,
//! removing one level disconnects the lower levels from the higher ones — a
//! genuine vertex separator. This is a textbook nested-dissection scheme,
//! deliberately DISTINCT from MATLAB's METIS-based nested dissection; no
//! MathWorks or METIS source, prose, or example was consulted. Licence: the
//! sources are published algorithm descriptions (journal / textbook); the Rust
//! code below is an original implementation and carries no third-party licence
//! obligation.

use std::collections::{HashSet, VecDeque};

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

#[runmat_macros::register_gpu_spec(builtin_path = "crate::builtins::math::sparse::dissect")]
pub const GPU_SPEC: BuiltinGpuSpec = BuiltinGpuSpec {
    name: "dissect",
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
    notes: "Nested dissection runs on the host over the symmetric graph pattern; GPU tensor inputs are gathered first.",
};

#[runmat_macros::register_fusion_spec(builtin_path = "crate::builtins::math::sparse::dissect")]
pub const FUSION_SPEC: BuiltinFusionSpec = BuiltinFusionSpec {
    name: "dissect",
    shape: ShapeRequirements::Any,
    constant_strategy: ConstantStrategy::InlineLiteral,
    elementwise: None,
    reduction: None,
    emits_nan: false,
    notes: "Recursive combinatorial ordering; not fusible.",
};

const DISSECT_OUTPUT: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "p",
    ty: BuiltinParamType::NumericArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Nested-dissection permutation row vector (1×n, a permutation of 1..n).",
}];

const DISSECT_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "S",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Sparse or full matrix.",
}];

const DISSECT_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "p = dissect(S)",
    inputs: &DISSECT_INPUTS,
    outputs: &DISSECT_OUTPUT,
}];

const DISSECT_ERROR_INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.DISSECT.INVALID_INPUT",
    identifier: Some("RunMat:dissect:InvalidInput"),
    when: "The input is not a 2-D numeric, logical, or sparse matrix.",
    message: "dissect: input must be a 2-D numeric matrix (full or sparse)",
};

const DISSECT_ERROR_INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.DISSECT.INTERNAL",
    identifier: Some("RunMat:dissect:Internal"),
    when: "Result materialisation fails internally.",
    message: "dissect: internal error",
};

const DISSECT_ERRORS: [BuiltinErrorDescriptor; 2] =
    [DISSECT_ERROR_INVALID_INPUT, DISSECT_ERROR_INTERNAL];

pub const DISSECT_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &DISSECT_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &DISSECT_ERRORS,
};

#[runtime_builtin(
    name = "dissect",
    category = "math/sparse",
    summary = "Nested-dissection fill-reducing permutation of a sparse matrix.",
    keywords = "dissect,sparse,permutation,nested dissection,ordering,fill-in",
    accel = "custom",
    type_resolver(dissect_type),
    descriptor(crate::builtins::math::sparse::dissect::DISSECT_DESCRIPTOR),
    builtin_path = "crate::builtins::math::sparse::dissect"
)]
async fn dissect_builtin(value: Value) -> BuiltinResult<Value> {
    // GPU tensors are gathered to host storage before the structural scan;
    // sparse and dense host values pass through unchanged.
    let gathered = gpu_helpers::gather_value_async(&value).await?;
    let (rows, cols, coords) = matrix_pattern(&gathered).map_err(invalid_input)?;
    // Nested dissection needs a symmetric graph on the `cols` vertices
    // [dissect.signature-primary].
    let adjacency = symmetric_graph(rows, cols, &coords);
    // Independent fill-reducing heuristic: recursive nested dissection
    // [dissect.permutation-vector; the specific order is non-normative,
    // dissect.q-values].
    let order = nested_dissection_order(cols, &adjacency);
    permutation_value(order).map_err(internal_error)
}

fn dissect_type(_args: &[Type], _context: &ResolveContext) -> Type {
    // Output is a 1×n double row vector (n = number of columns); the length is
    // data-dependent, so only the row orientation is known statically.
    Type::Tensor {
        shape: Some(vec![Some(1), None]),
    }
}

/// Build the symmetric adjacency graph on the `cols` column-vertices.
///
/// - Square input (`rows == cols`): the graph of the symmetrised pattern
///   `A + Aᵀ` — vertices `i` and `j` are adjacent iff `A[i,j]` or `A[j,i]` is
///   nonzero. This is the standard graph of a (structurally) symmetric matrix.
/// - Non-square input (documented choice; unobserved): `A + Aᵀ` is undefined,
///   so we fall back to the column-intersection graph (columns adjacent iff
///   they share a nonzero row), which is symmetric on the `cols` vertices.
fn symmetric_graph(rows: usize, cols: usize, coords: &[(usize, usize)]) -> Vec<HashSet<usize>> {
    let mut adjacency: Vec<HashSet<usize>> = vec![HashSet::new(); cols];
    if rows == cols {
        for &(r, c) in coords {
            if r < cols && c < cols && r != c {
                adjacency[r].insert(c);
                adjacency[c].insert(r);
            }
        }
    } else {
        let mut cols_in_row: Vec<Vec<usize>> = vec![Vec::new(); rows];
        for &(r, c) in coords {
            if r < rows && c < cols {
                cols_in_row[r].push(c);
            }
        }
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
    }
    adjacency
}

/// Recursive nested-dissection ordering over the whole vertex set 0..n.
fn nested_dissection_order(n: usize, adjacency: &[HashSet<usize>]) -> Vec<usize> {
    let vertices: Vec<usize> = (0..n).collect();
    order_subset(&vertices, adjacency)
}

/// Order an induced subset: split into connected components, and within a
/// connected component of size ≥ 3 bisect by a level separator — number the
/// two sides recursively first, then the separator last (nested dissection).
fn order_subset(vertices: &[usize], adjacency: &[HashSet<usize>]) -> Vec<usize> {
    if vertices.len() <= 2 {
        let mut sorted = vertices.to_vec();
        sorted.sort_unstable();
        return sorted;
    }
    let members: HashSet<usize> = vertices.iter().copied().collect();
    let mut components = connected_components(vertices, &members, adjacency);
    if components.len() > 1 {
        // Deterministic order: components by their smallest vertex.
        components.sort_by_key(|component| *component.iter().min().expect("nonempty component"));
        let mut result = Vec::with_capacity(vertices.len());
        for component in &components {
            result.extend(order_subset(component, adjacency));
        }
        return result;
    }
    match bisect(vertices, &members, adjacency) {
        Some((left, right, mut separator)) => {
            let mut result = order_subset(&left, adjacency);
            result.extend(order_subset(&right, adjacency));
            separator.sort_unstable();
            result.extend(separator);
            result
        }
        // No usable separator (e.g. a clique): fall back to index order.
        None => {
            let mut sorted = vertices.to_vec();
            sorted.sort_unstable();
            sorted
        }
    }
}

/// Connected components of the subgraph induced by `members`.
fn connected_components(
    vertices: &[usize],
    members: &HashSet<usize>,
    adjacency: &[HashSet<usize>],
) -> Vec<Vec<usize>> {
    let mut visited: HashSet<usize> = HashSet::new();
    let mut components = Vec::new();
    for &start in vertices {
        if visited.contains(&start) {
            continue;
        }
        let mut stack = vec![start];
        visited.insert(start);
        let mut component = Vec::new();
        while let Some(u) = stack.pop() {
            component.push(u);
            for &w in &adjacency[u] {
                if members.contains(&w) && visited.insert(w) {
                    stack.push(w);
                }
            }
        }
        components.push(component);
    }
    components
}

/// Bisect a connected subset of size ≥ 3 using a breadth-first level structure
/// rooted at a pseudo-peripheral vertex. Returns `(left, right, separator)`
/// where `separator` is a whole BFS level whose removal disconnects `left`
/// (lower levels) from `right` (higher levels). Returns `None` when no level
/// separator leaves both sides nonempty (fewer than three levels).
fn bisect(
    vertices: &[usize],
    members: &HashSet<usize>,
    adjacency: &[HashSet<usize>],
) -> Option<(Vec<usize>, Vec<usize>, Vec<usize>)> {
    let start = pseudo_peripheral(vertices, members, adjacency);
    let levels = bfs_levels(start, members, adjacency);
    let max_level = vertices.iter().map(|v| levels[v]).max().unwrap_or(0);
    if max_level < 2 {
        return None;
    }
    let mut count_per_level = vec![0usize; max_level + 1];
    for &v in vertices {
        count_per_level[levels[&v]] += 1;
    }
    // Pick the smallest interior level whose cumulative count reaches half the
    // vertices, so the lower side gets roughly half and both sides stay
    // nonempty.
    let target = (vertices.len() / 2).max(1);
    let mut cumulative = 0usize;
    let mut separator_level = max_level - 1;
    for (level, &count) in count_per_level.iter().enumerate() {
        cumulative += count;
        if level >= 1 && level < max_level && cumulative >= target {
            separator_level = level;
            break;
        }
    }
    let mut left = Vec::new();
    let mut right = Vec::new();
    let mut separator = Vec::new();
    for &v in vertices {
        match levels[&v].cmp(&separator_level) {
            std::cmp::Ordering::Less => left.push(v),
            std::cmp::Ordering::Equal => separator.push(v),
            std::cmp::Ordering::Greater => right.push(v),
        }
    }
    if left.is_empty() || right.is_empty() {
        return None;
    }
    Some((left, right, separator))
}

/// A pseudo-peripheral start vertex (Gibbs–Poole–Stockmeyer style): start from
/// a minimum-degree vertex, then repeatedly jump to a farthest vertex of the
/// rooted level structure. A small fixed number of passes keeps it terminating
/// and deterministic (ties broken by smallest index).
fn pseudo_peripheral(
    vertices: &[usize],
    members: &HashSet<usize>,
    adjacency: &[HashSet<usize>],
) -> usize {
    let induced_degree = |v: usize| adjacency[v].iter().filter(|w| members.contains(w)).count();
    let mut start = *vertices
        .iter()
        .min_by_key(|&&v| (induced_degree(v), v))
        .expect("nonempty subset");
    for _ in 0..3 {
        let levels = bfs_levels(start, members, adjacency);
        // Farthest vertex: maximum level, ties broken by smallest index.
        let farthest = vertices
            .iter()
            .copied()
            .max_by_key(|&v| (levels[&v], std::cmp::Reverse(v)))
            .expect("nonempty subset");
        if farthest == start {
            break;
        }
        start = farthest;
    }
    start
}

/// Breadth-first level (distance) of every vertex in the connected subset from
/// `start`.
fn bfs_levels(
    start: usize,
    members: &HashSet<usize>,
    adjacency: &[HashSet<usize>],
) -> std::collections::HashMap<usize, usize> {
    let mut levels = std::collections::HashMap::new();
    let mut queue = VecDeque::new();
    levels.insert(start, 0usize);
    queue.push_back(start);
    while let Some(u) = queue.pop_front() {
        let next = levels[&u] + 1;
        // Deterministic neighbour visitation order.
        let mut neighbours: Vec<usize> = adjacency[u]
            .iter()
            .copied()
            .filter(|w| members.contains(w))
            .collect();
        neighbours.sort_unstable();
        for w in neighbours {
            if let std::collections::hash_map::Entry::Vacant(e) = levels.entry(w) {
                e.insert(next);
                queue.push_back(w);
            }
        }
    }
    levels
}

/// Materialise a zero-based ordering as a 1×n double row vector of one-based
/// permutation indices [dissect.output-class, dissect.permutation-vector]. An
/// empty ordering yields a 1×0 empty row (empty matrix → empty permutation);
/// a 1×1 result is kept as an explicit 1×1 tensor (documented choice).
fn permutation_value(order: Vec<usize>) -> Result<Value, String> {
    let n = order.len();
    let data: Vec<f64> = order.into_iter().map(|v| (v + 1) as f64).collect();
    Tensor::new(data, vec![1, n]).map(Value::Tensor)
}

fn invalid_input(detail: String) -> RuntimeError {
    build_runtime_error(format!("dissect: {detail}"))
        .with_builtin("dissect")
        .with_identifier(DISSECT_ERROR_INVALID_INPUT.identifier.expect("identifier"))
        .build()
}

fn internal_error(detail: String) -> RuntimeError {
    build_runtime_error(format!("dissect: {detail}"))
        .with_builtin("dissect")
        .with_identifier(DISSECT_ERROR_INTERNAL.identifier.expect("identifier"))
        .build()
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use futures::executor::block_on;
    use runmat_builtins::{builtin_function_by_name, CharArray, LogicalArray, SparseTensor};

    fn run_dissect(value: Value) -> Value {
        block_on(super::dissect_builtin(value)).expect("dissect")
    }

    /// Assert the result is a class-`double`, 1×n row vector that is a valid
    /// permutation of 1..n — the ONLY normative assertions
    /// [dissect.output-class, dissect.permutation-vector]. The specific
    /// ordering is never asserted (dissect.q-values).
    fn assert_valid_permutation(value: &Value, n: usize) -> Vec<usize> {
        let tensor = match value {
            Value::Tensor(t) => t.clone(),
            other => panic!("expected dense double row vector, got {other:?}"),
        };
        assert_eq!(
            tensor.shape,
            vec![1, n],
            "output must be a 1×{n} row vector"
        );
        assert_eq!(tensor.data.len(), n);
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
        // sparse([2 1 0; 1 2 1; 0 1 2]) in CSC form (symmetric tridiagonal).
        // col0: rows {0,1}; col1: rows {0,1,2}; col2: rows {1,2}.
        SparseTensor::new(
            3,
            3,
            vec![0, 2, 5, 7],
            vec![0, 1, 0, 1, 2, 1, 2],
            vec![1.0, 1.0, 1.0, 2.0, 1.0, 1.0, 2.0],
        )
        .unwrap()
    }

    // Normative [FR-027-01, FR-027-03; dissect.output-class,
    // dissect.permutation-vector] — the ONLY asserted properties on the
    // observed input `sparse([2 1 0; 1 2 1; 0 1 2])` are class double, size
    // 1×3, valid permutation of 1..3. The specific values are NOT asserted.
    #[test]
    fn observed_input_returns_valid_permutation() {
        let result = run_dissect(Value::SparseTensor(observed_matrix()));
        assert_valid_permutation(&result, 3);
    }

    // documented_choice [FR-027-04; dissect.q-values] — NON-NORMATIVE record of
    // THIS implementation's nested-dissection ordering on the observed input.
    // This implementation's level-separator scheme roots at a pseudo-peripheral
    // end of the path 0-1-2 and numbers the two ends then the middle, giving
    // `[3 1 2]`. That coincides with the observed MATLAB value here, but the
    // coincidence is incidental: the specific values are algorithm-dependent
    // and non-normative (dissect.q-values), NOT a requirement, and are recorded
    // for traceability only.
    #[test]
    fn documented_choice_ordering_on_observed_input() {
        let result = run_dissect(Value::SparseTensor(observed_matrix()));
        let Value::Tensor(t) = result else {
            panic!("expected tensor");
        };
        assert_eq!(t.data, vec![3.0, 1.0, 2.0]);
    }

    // Normative [dissect.permutation-vector] — a dense symmetric matrix is
    // accepted by reading its structure; the result is still valid.
    #[test]
    fn dense_input_is_accepted_and_valid() {
        // [2 1 0; 1 2 1; 0 1 2] column-major.
        let dense = Tensor::new(
            vec![2.0, 1.0, 0.0, 1.0, 2.0, 1.0, 0.0, 1.0, 2.0],
            vec![3, 3],
        )
        .unwrap();
        let result = run_dissect(Value::Tensor(dense));
        assert_valid_permutation(&result, 3);
    }

    // Normative [dissect.permutation-vector] — a larger symmetric pattern (a
    // 6-vertex path) is bisected recursively and yields a valid permutation.
    #[test]
    fn larger_symmetric_input_valid_permutation() {
        // Path graph 0-1-2-3-4-5 as a symmetric tridiagonal 6×6 pattern.
        let n = 6;
        let mut col_ptrs = vec![0usize];
        let mut row_indices = Vec::new();
        let mut values = Vec::new();
        for col in 0..n {
            let mut rows_here = Vec::new();
            if col > 0 {
                rows_here.push(col - 1);
            }
            rows_here.push(col);
            if col + 1 < n {
                rows_here.push(col + 1);
            }
            for r in rows_here {
                row_indices.push(r);
                values.push(1.0);
            }
            col_ptrs.push(row_indices.len());
        }
        let sparse = SparseTensor::new(n, n, col_ptrs, row_indices, values).unwrap();
        let result = run_dissect(Value::SparseTensor(sparse));
        assert_valid_permutation(&result, n);
    }

    // Normative [dissect.permutation-vector] — empty matrix → empty
    // permutation (1×0 double).
    #[test]
    fn empty_matrix_yields_empty_permutation() {
        let result = run_dissect(Value::SparseTensor(SparseTensor::zeros(0, 0)));
        assert_valid_permutation(&result, 0);
    }

    // documented_choice [FR-027-04] — a 1×1 input yields the single-element
    // permutation [1] as a 1×1 row.
    #[test]
    fn documented_choice_scalar_input() {
        let result = run_dissect(Value::Num(4.0));
        let perm = assert_valid_permutation(&result, 1);
        assert_eq!(perm, vec![1]);
    }

    // documented_choice [FR-027-05] — a non-square matrix is symmetrised via
    // the column-intersection graph (documented choice; A+Aᵀ is undefined for
    // rectangular inputs and this case is unobserved).
    #[test]
    fn documented_choice_non_square_input_accepted() {
        // 2×3 pattern; permutes the 3 columns.
        let sparse = SparseTensor::new(
            2,
            3,
            vec![0, 1, 3, 4],
            vec![0, 0, 1, 1],
            vec![1.0, 1.0, 1.0, 1.0],
        )
        .unwrap();
        let result = run_dissect(Value::SparseTensor(sparse));
        assert_valid_permutation(&result, 3);
    }

    // documented_choice [FR-027-05] — a logical matrix is accepted as a
    // structural pattern (unobserved input class; documented choice).
    #[test]
    fn documented_choice_logical_input_accepted() {
        let logical = LogicalArray::new(vec![1, 1, 1, 1], vec![2, 2]).unwrap();
        let result = run_dissect(Value::LogicalArray(logical));
        assert_valid_permutation(&result, 2);
    }

    // documented_choice [FR-027-05] — non-matrix inputs are rejected with the
    // stable InvalidInput identifier (documented choice; unobserved).
    #[test]
    fn documented_choice_non_matrix_input_errors() {
        let err = block_on(super::dissect_builtin(Value::CharArray(
            CharArray::new_row("ab"),
        )))
        .expect_err("error");
        assert_eq!(err.identifier(), Some("RunMat:dissect:InvalidInput"));
    }

    // descriptor / macro registration test — the builtin is discoverable via
    // the compile-time inventory registry with its documented signature.
    #[test]
    fn descriptor_is_registered() {
        let builtin = builtin_function_by_name("dissect").expect("dissect builtin registered");
        let descriptor = builtin.descriptor.expect("descriptor present");
        assert!(descriptor
            .signatures
            .iter()
            .any(|sig| sig.label == "p = dissect(S)"));
    }

    // GPU spec / fusion spec descriptor test — nested dissection is a host-only
    // custom op and is not fusible.
    #[test]
    fn gpu_and_fusion_specs_are_custom_and_non_fusible() {
        assert_eq!(GPU_SPEC.name, "dissect");
        assert!(matches!(GPU_SPEC.op_kind, GpuOpKind::Custom("sparse")));
        assert_eq!(FUSION_SPEC.name, "dissect");
        assert!(FUSION_SPEC.elementwise.is_none());
        assert!(FUSION_SPEC.reduction.is_none());
    }

    // type-resolver test — output type is a 1×n double row vector.
    #[test]
    fn type_resolver_is_row_vector() {
        let ty = dissect_type(&[Type::tensor()], &ResolveContext::new(Vec::new()));
        assert_eq!(
            ty,
            Type::Tensor {
                shape: Some(vec![Some(1), None])
            }
        );
    }
}
