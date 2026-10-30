//! Clean-room `incidence` and `isisomorphic` (graph structure).
//!
//! Clean-room feature `specs/036-graph-structure/`. Normative behaviour comes
//! only from the approved exports `incidence` and `isisomorphic` 0.2.0 (see
//! that feature's `provenance.md`).

use super::construction::graph_data;
use super::*;
use runmat_value::SparseTensor;

// ---------------------------------------------------------------------------
// Descriptors
// ---------------------------------------------------------------------------

const INCIDENCE_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "G",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Undirected graph object.",
}];
const INCIDENCE_OUTPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "I",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Sparse node-by-edge incidence matrix.",
}];
const INCIDENCE_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "I = incidence(G)",
    inputs: &INCIDENCE_INPUTS,
    outputs: &INCIDENCE_OUTPUTS,
}];

const ISISOMORPHIC_INPUTS: [BuiltinParamDescriptor; 2] = [
    BuiltinParamDescriptor {
        name: "G1",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "First graph or digraph object.",
    },
    BuiltinParamDescriptor {
        name: "G2",
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description: "Second graph or digraph object (same class as G1).",
    },
];
const ISISOMORPHIC_OUTPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "tf",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Logical scalar: true when G1 and G2 are isomorphic.",
}];
const ISISOMORPHIC_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "tf = isisomorphic(G1, G2)",
    inputs: &ISISOMORPHIC_INPUTS,
    outputs: &ISISOMORPHIC_OUTPUTS,
}];

macro_rules! structure_errors {
    (
        $errors:ident, $prefix:literal, $id:literal,
        $specific_code:literal, $specific_id:literal, $specific_when:literal, $specific_message:literal
    ) => {
        const $errors: [BuiltinErrorDescriptor; 5] = [
            BuiltinErrorDescriptor {
                code: concat!("RM.", $prefix, ".INVALID_INPUT"),
                identifier: Some(concat!("RunMat:", $id, ":InvalidInput")),
                when: "A required graph input is missing or the graph object is malformed.",
                message: concat!($id, ": invalid graph input"),
            },
            BuiltinErrorDescriptor {
                code: concat!("RM.", $prefix, ".UNSUPPORTED_OPTION"),
                identifier: Some(concat!("RunMat:", $id, ":UnsupportedOption")),
                when: "The call uses a form or option that is not yet specified for RunMat.",
                message: concat!($id, ": option not yet specified"),
            },
            BuiltinErrorDescriptor {
                code: concat!("RM.", $prefix, ".INTERNAL"),
                identifier: Some(concat!("RunMat:", $id, ":Internal")),
                when: "RunMat cannot build the result value.",
                message: concat!($id, ": internal error"),
            },
            BuiltinErrorDescriptor {
                code: concat!("RM.", $prefix, ".UNDEFINED_FUNCTION"),
                identifier: Some(concat!("RunMat:", $id, ":UndefinedFunction")),
                when: "An input is not a graph or digraph object (e.g. a numeric matrix).",
                message: concat!($id, ": undefined for non-graph input"),
            },
            BuiltinErrorDescriptor {
                code: concat!("RM.", $prefix, ".", $specific_code),
                identifier: Some(concat!("RunMat:", $id, ":", $specific_id)),
                when: $specific_when,
                message: concat!($id, ": ", $specific_message),
            },
        ];
    };
}
structure_errors!(
    INCIDENCE_ERRORS,
    "INCIDENCE",
    "incidence",
    "SELF_LOOPS",
    "SelfLoops",
    "The graph or digraph contains a self-loop.",
    "graph contains self-loops"
);
structure_errors!(
    ISISOMORPHIC_ERRORS,
    "ISISOMORPHIC",
    "isisomorphic",
    "CLASS_MISMATCH",
    "ClassMismatch",
    "One input is a graph and the other a digraph.",
    "inputs must both be graphs or both be digraphs"
);

const INVALID_INPUT: usize = 0;
const UNSUPPORTED_OPTION: usize = 1;
const INTERNAL: usize = 2;
const UNDEFINED_FUNCTION: usize = 3;
/// `SelfLoops` for `incidence`, `ClassMismatch` for `isisomorphic`.
const SPECIFIC: usize = 4;

pub const INCIDENCE_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &INCIDENCE_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &INCIDENCE_ERRORS,
};
pub const ISISOMORPHIC_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &ISISOMORPHIC_SIGNATURES,
    output_mode: BuiltinOutputMode::ByRequestedOutputCount,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ISISOMORPHIC_ERRORS,
};

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn structure_error(
    builtin: &'static str,
    errors: &'static [BuiltinErrorDescriptor; 5],
    kind: usize,
    detail: impl Into<String>,
) -> RuntimeError {
    error_from_descriptor(
        builtin,
        errors[kind].clone(),
        format!("{builtin}: {}", detail.into()),
    )
}

/// Read a graph or digraph argument; anything else is `UndefinedFunction`
/// (incidence.error-matrix, isisomorphic.error-matrix).
fn graph_input(
    builtin: &'static str,
    errors: &'static [BuiltinErrorDescriptor; 5],
    value: &Value,
) -> BuiltinResult<GraphData> {
    let is_graph = matches!(
        value,
        Value::Object(object)
            if object.class_name == GRAPH_CLASS || object.class_name == DIGRAPH_CLASS
    );
    if !is_graph {
        return Err(structure_error(
            builtin,
            errors,
            UNDEFINED_FUNCTION,
            "undefined for input that is not a graph or digraph object",
        ));
    }
    graph_data(value, builtin).map_err(|err| {
        structure_error(
            builtin,
            errors,
            INVALID_INPUT,
            format!("invalid graph object: {}", err.message()),
        )
    })
}

// ---------------------------------------------------------------------------
// incidence
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "incidence",
    category = "graph",
    summary = "Return the sparse node-by-edge incidence matrix of a graph or digraph.",
    keywords = "incidence,graph,edges,nodes,sparse,matrix",
    accel = "cpu",
    type_resolver(any_type),
    descriptor(crate::builtins::graph::domain::structure::INCIDENCE_DESCRIPTOR),
    builtin_path = "crate::builtins::graph"
)]
pub(super) async fn incidence_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    const NAME: &str = "incidence";
    let args = gather_values(args).await?;
    let Some(first) = args.first() else {
        return Err(structure_error(
            NAME,
            &INCIDENCE_ERRORS,
            INVALID_INPUT,
            "expected a graph input",
        ));
    };
    let graph = graph_input(NAME, &INCIDENCE_ERRORS, first)?;
    if args.len() > 1 {
        // U-036-1
        return Err(structure_error(
            NAME,
            &INCIDENCE_ERRORS,
            UNSUPPORTED_OPTION,
            "only incidence(G) is specified",
        ));
    }
    if graph.edges.iter().any(|edge| edge.source == edge.target) {
        // incidence.error-self-loop-graph
        return Err(structure_error(
            NAME,
            &INCIDENCE_ERRORS,
            SPECIFIC,
            "the incidence matrix is not defined for graphs with self-loops",
        ));
    }
    // incidence.signs, incidence.edge-order, incidence.weights-ignored:
    // column j is stored edge j (graph or digraph); -1 at its first end node,
    // +1 at its second. Row indices are sorted within each column (CSC).
    let edge_count = graph.edges.len();
    let mut col_ptrs = Vec::with_capacity(edge_count + 1);
    let mut row_indices = Vec::with_capacity(2 * edge_count);
    let mut values = Vec::with_capacity(2 * edge_count);
    col_ptrs.push(0);
    for edge in &graph.edges {
        let (low, high) = (edge.source.min(edge.target), edge.source.max(edge.target));
        for node in [low, high] {
            row_indices.push(node);
            values.push(if node == edge.source { -1.0 } else { 1.0 });
        }
        col_ptrs.push(row_indices.len());
    }
    let matrix = SparseTensor::new(graph.node_count, edge_count, col_ptrs, row_indices, values)
        .map_err(|err| structure_error(NAME, &INCIDENCE_ERRORS, INTERNAL, err))?;
    Ok(Value::SparseTensor(matrix))
}

// ---------------------------------------------------------------------------
// isisomorphic
// ---------------------------------------------------------------------------

/// Edge multiplicities: `counts[u * n + v]` is the number of edges from `u`
/// to `v`. Undirected edges count in both directions; a self-loop adds one
/// to the diagonal.
struct Multigraph {
    n: usize,
    counts: Vec<u32>,
}

impl Multigraph {
    fn new(graph: &GraphData) -> Self {
        let n = graph.node_count;
        let mut counts = vec![0u32; n * n];
        for edge in &graph.edges {
            counts[edge.source * n + edge.target] += 1;
            if !graph.directed && edge.source != edge.target {
                counts[edge.target * n + edge.source] += 1;
            }
        }
        Self { n, counts }
    }

    fn at(&self, u: usize, v: usize) -> u32 {
        self.counts[u * self.n + v]
    }

    /// Per-node invariant `(out-links, in-links, self-loops)`, each excluding
    /// self-loops from the first two.
    fn invariants(&self) -> Vec<(u32, u32, u32)> {
        (0..self.n)
            .map(|u| {
                let loops = self.at(u, u);
                let out: u32 = (0..self.n).map(|v| self.at(u, v)).sum::<u32>() - loops;
                let inc: u32 = (0..self.n).map(|v| self.at(v, u)).sum::<u32>() - loops;
                (out, inc, loops)
            })
            .collect()
    }

    fn links(&self, u: usize, v: usize) -> u32 {
        self.at(u, v) + self.at(v, u)
    }
}

/// Search order: start at the most connected node, then repeatedly take the
/// node with the most links into the nodes already ordered (ties: more
/// links overall, then lower index). Keeps the partial map constrained.
fn search_order(graph: &Multigraph, invariants: &[(u32, u32, u32)]) -> Vec<usize> {
    let n = graph.n;
    let total = |u: usize| {
        let (out, inc, loops) = invariants[u];
        out + inc + loops
    };
    let mut ordered = Vec::with_capacity(n);
    let mut placed = vec![false; n];
    let mut attached = vec![0u32; n];
    for _ in 0..n {
        let next = (0..n)
            .filter(|&u| !placed[u])
            .max_by(|&a, &b| {
                (attached[a], total(a))
                    .cmp(&(attached[b], total(b)))
                    .then(b.cmp(&a))
            })
            .expect("an unplaced node remains");
        placed[next] = true;
        ordered.push(next);
        for (u, slot) in attached.iter_mut().enumerate() {
            *slot += graph.links(next, u);
        }
    }
    ordered
}

struct Matcher<'a> {
    g1: &'a Multigraph,
    g2: &'a Multigraph,
    inv1: &'a [(u32, u32, u32)],
    inv2: &'a [(u32, u32, u32)],
    order: Vec<usize>,
    map: Vec<Option<usize>>,
    used: Vec<bool>,
}

impl Matcher<'_> {
    fn feasible(&self, depth: usize, v: usize, w: usize) -> bool {
        if self.used[w] || self.inv1[v] != self.inv2[w] || self.g1.at(v, v) != self.g2.at(w, w) {
            return false;
        }
        self.order[..depth].iter().all(|&u| {
            let mapped = self.map[u].expect("ordered prefix is mapped");
            self.g1.at(v, u) == self.g2.at(w, mapped) && self.g1.at(u, v) == self.g2.at(mapped, w)
        })
    }

    fn extend(&mut self, depth: usize) -> bool {
        if depth == self.order.len() {
            return true;
        }
        let v = self.order[depth];
        for w in 0..self.g2.n {
            if self.feasible(depth, v, w) {
                self.map[v] = Some(w);
                self.used[w] = true;
                if self.extend(depth + 1) {
                    return true;
                }
                self.map[v] = None;
                self.used[w] = false;
            }
        }
        false
    }
}

/// Multigraph isomorphism by backtracking with invariant pruning
/// (FR-036-06, FR-036-07).
fn graphs_isomorphic(a: &GraphData, b: &GraphData) -> bool {
    if a.node_count != b.node_count || a.edges.len() != b.edges.len() {
        return false;
    }
    let g1 = Multigraph::new(a);
    let g2 = Multigraph::new(b);
    let inv1 = g1.invariants();
    let inv2 = g2.invariants();
    let mut sorted1 = inv1.clone();
    let mut sorted2 = inv2.clone();
    sorted1.sort_unstable();
    sorted2.sort_unstable();
    if sorted1 != sorted2 {
        return false;
    }
    let order = search_order(&g1, &inv1);
    let mut matcher = Matcher {
        g1: &g1,
        g2: &g2,
        inv1: &inv1,
        inv2: &inv2,
        order,
        map: vec![None; g1.n],
        used: vec![false; g2.n],
    };
    matcher.extend(0)
}

fn logical_type(_args: &[Type], _ctx: &ResolveContext) -> Type {
    Type::Bool
}

#[runtime_builtin(
    name = "isisomorphic",
    category = "graph",
    summary = "Determine whether two graphs are isomorphic.",
    keywords = "isisomorphic,graph,digraph,isomorphism,structure",
    accel = "cpu",
    type_resolver(logical_type),
    descriptor(crate::builtins::graph::domain::structure::ISISOMORPHIC_DESCRIPTOR),
    builtin_path = "crate::builtins::graph"
)]
pub(super) async fn isisomorphic_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    const NAME: &str = "isisomorphic";
    let args = gather_values(args).await?;
    if args.len() < 2 {
        return Err(structure_error(
            NAME,
            &ISISOMORPHIC_ERRORS,
            INVALID_INPUT,
            "expected two graph inputs",
        ));
    }
    let first = graph_input(NAME, &ISISOMORPHIC_ERRORS, &args[0])?;
    let second = graph_input(NAME, &ISISOMORPHIC_ERRORS, &args[1])?;
    if args.len() > 2 {
        // isisomorphic.q-options
        return Err(structure_error(
            NAME,
            &ISISOMORPHIC_ERRORS,
            UNSUPPORTED_OPTION,
            "only isisomorphic(G1, G2) is specified",
        ));
    }
    if first.directed != second.directed {
        // isisomorphic.error-mixed
        return Err(structure_error(
            NAME,
            &ISISOMORPHIC_ERRORS,
            SPECIFIC,
            "inputs must both be graphs or both be digraphs",
        ));
    }
    let result = Value::Bool(graphs_isomorphic(&first, &second));
    match crate::output_count::current_output_count() {
        Some(0) => Ok(Value::OutputList(Vec::new())),
        Some(1) => Ok(Value::OutputList(vec![result])),
        // isisomorphic.q-second-output
        Some(_) => Err(structure_error(
            NAME,
            &ISISOMORPHIC_ERRORS,
            UNSUPPORTED_OPTION,
            "a second output (node permutation) is not yet supported",
        )),
        None => Ok(result),
    }
}

#[cfg(test)]
mod graph_structure_tests {
    use super::*;
    use futures::executor::block_on;
    use runmat_value::SparseTensor;

    fn row(values: &[f64]) -> Value {
        Value::Tensor(Tensor::new(values.to_vec(), vec![1, values.len()]).unwrap())
    }

    fn matrix(rows: usize, cols: usize, data: Vec<f64>) -> Value {
        Value::Tensor(Tensor::new(data, vec![rows, cols]).unwrap())
    }

    fn names(values: &[&str]) -> Value {
        Value::StringArray(
            StringArray::new(
                values.iter().map(|s| s.to_string()).collect(),
                vec![1, values.len()],
            )
            .unwrap(),
        )
    }

    fn call(name: &str, args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async(name, args))
    }

    fn graph(s: &[f64], t: &[f64]) -> Value {
        call("graph", &[row(s), row(t)]).expect("graph")
    }

    fn digraph(s: &[f64], t: &[f64]) -> Value {
        call("digraph", &[row(s), row(t)]).expect("digraph")
    }

    /// `n` nodes with one undirected edge 1-2 (adjacency form). Stands in for
    /// the observed `graph(1, 2, 1, n)`, whose node-count argument RunMat's
    /// graph constructor does not accept (see validation.md deviations).
    fn edge_plus_isolated(n: usize) -> Value {
        let mut data = vec![0.0; n * n];
        data[1] = 1.0; // (2,1)
        data[n] = 1.0; // (1,2)
        call("graph", &[matrix(n, n, data)]).expect("graph")
    }

    fn identifier(err: RuntimeError) -> String {
        err.identifier().unwrap_or("<none>").to_string()
    }

    fn sparse(
        rows: usize,
        cols: usize,
        col_ptrs: Vec<usize>,
        row_indices: Vec<usize>,
        values: Vec<f64>,
    ) -> Value {
        Value::SparseTensor(SparseTensor::new(rows, cols, col_ptrs, row_indices, values).unwrap())
    }

    /// Dense column-major contents of a sparse result.
    fn dense(value: &Value) -> (Vec<usize>, Vec<f64>) {
        let Value::SparseTensor(matrix) = value else {
            panic!("expected sparse result, got {value:?}");
        };
        let full = matrix.to_dense().unwrap();
        (full.shape.clone(), full.materialize_f64())
    }

    fn incidence(g: Value) -> Value {
        call("incidence", &[g]).expect("incidence")
    }

    /// RunMat's stored `G.Edges.EndNodes` as zero-based pairs.
    fn stored_end_nodes(g: &Value) -> Vec<(usize, usize)> {
        let Value::Object(object) = g else {
            panic!("expected graph object");
        };
        let Some(Value::Object(edges)) = object.properties.get("Edges") else {
            panic!("expected Edges table");
        };
        let vars = table_variables(edges).unwrap();
        let Some(Value::Tensor(end_nodes)) = vars.fields.get("EndNodes") else {
            panic!("expected numeric EndNodes");
        };
        let data = end_nodes.materialize_f64();
        (0..end_nodes.rows)
            .map(|r| (data[r] as usize - 1, data[r + end_nodes.rows] as usize - 1))
            .collect()
    }

    fn iso(a: &Value, b: &Value) -> Value {
        call("isisomorphic", &[a.clone(), b.clone()]).expect("isisomorphic")
    }

    // ---- incidence: observed (normative) ---------------------------------

    /// incidence.signature-primary, incidence.output-class, incidence.shape,
    /// incidence.signs (cases `sparse-class`, `basic`, `warn-call`).
    #[test]
    fn observed_incidence_graph_matches_export() {
        let result = incidence(graph(&[1.0, 2.0], &[2.0, 3.0]));
        let Value::SparseTensor(ref m) = result else {
            panic!("expected sparse result, got {result:?}");
        };
        assert!(m.as_f64_slice().is_some(), "class double");
        // sparse([1;2;2;3], [1;1;2;2], [-1;1;-1;1], 3, 2)
        let expected = sparse(
            3,
            2,
            vec![0, 2, 4],
            vec![0, 1, 1, 2],
            vec![-1.0, 1.0, -1.0, 1.0],
        );
        assert_eq!(result, expected);
    }

    /// incidence.output-class, incidence.signs (cases `directed`,
    /// `directed-sparse-class`, `directed-reverse`).
    #[test]
    fn observed_incidence_digraph_matches_export() {
        let result = incidence(digraph(&[1.0, 2.0], &[2.0, 3.0]));
        let expected = sparse(
            3,
            2,
            vec![0, 2, 4],
            vec![0, 1, 1, 2],
            vec![-1.0, 1.0, -1.0, 1.0],
        );
        assert_eq!(result, expected);
        // digraph(3, 1): edge 3->1 gives [1;0;-1]
        let result = incidence(digraph(&[3.0], &[1.0]));
        assert_eq!(dense(&result), (vec![3, 1], vec![1.0, 0.0, -1.0]));
    }

    /// incidence.multigraph (case `multigraph`).
    #[test]
    fn observed_incidence_multigraph_matches_export() {
        let result = incidence(graph(&[1.0, 1.0], &[2.0, 2.0]));
        assert_eq!(dense(&result), (vec![2, 2], vec![-1.0, 1.0, -1.0, 1.0]));
    }

    /// incidence.weights-ignored (case `weighted`).
    #[test]
    fn observed_incidence_weights_ignored() {
        let g = call(
            "graph",
            &[row(&[1.0, 2.0]), row(&[2.0, 3.0]), row(&[5.0, 7.0])],
        )
        .unwrap();
        let result = incidence(g);
        assert_eq!(
            dense(&result),
            (vec![3, 2], vec![-1.0, 1.0, 0.0, 0.0, -1.0, 1.0])
        );
    }

    /// incidence.shape, incidence.output-class (cases `isolated-node`,
    /// `empty-graph`, `nodes-no-edges`).
    #[test]
    fn observed_incidence_shapes_match_export() {
        let result = incidence(edge_plus_isolated(4));
        assert_eq!(dense(&result), (vec![4, 1], vec![-1.0, 1.0, 0.0, 0.0]));
        let empty = call("graph", &[]).unwrap();
        assert_eq!(incidence(empty), sparse(0, 0, vec![0], vec![], vec![]));
        let no_edges = call("graph", &[matrix(3, 3, vec![0.0; 9])]).unwrap();
        assert_eq!(incidence(no_edges), sparse(3, 0, vec![0], vec![], vec![]));
    }

    /// incidence.edge-order, incidence.signs: columns follow the graph's
    /// stored edge order, -1 at EndNodes(j,1) and +1 at EndNodes(j,2)
    /// (cases `edge-order`, `directed-edge-order` and their `-table` cases,
    /// checked against RunMat's own stored order).
    #[test]
    fn observed_incidence_columns_follow_stored_edge_order() {
        for g in [
            graph(&[2.0, 1.0], &[3.0, 2.0]),
            digraph(&[3.0, 1.0], &[1.0, 2.0]),
            graph(&[3.0], &[1.0]),
        ] {
            let pairs = stored_end_nodes(&g);
            let (shape, data) = dense(&incidence(g));
            assert_eq!(shape, vec![3, pairs.len()]);
            for (j, (first, second)) in pairs.into_iter().enumerate() {
                let mut column = vec![0.0; 3];
                column[first] = -1.0;
                column[second] = 1.0;
                assert_eq!(&data[3 * j..3 * j + 3], column.as_slice(), "column {j}");
            }
        }
    }

    /// Deviation D-036-1: the observed values for unsorted input depend on
    /// the graph constructor canonicalising and sorting edges (observed
    /// `G.Edges.EndNodes` of `[1 2; 2 3]`, `[1 2; 3 1]` and `[1 3]`).
    /// RunMat's `graph`/`digraph` keep the input order and endpoints, which
    /// is outside this feature's scope.
    #[test]
    #[ignore = "D-036-1: RunMat graph()/digraph() do not canonicalise edge order"]
    fn deviation_incidence_observed_values_for_unsorted_input() {
        // case `edge-order`
        let result = incidence(graph(&[2.0, 1.0], &[3.0, 2.0]));
        assert_eq!(dense(&result).1, vec![-1.0, 1.0, 0.0, 0.0, -1.0, 1.0]);
        // case `directed-edge-order`
        let result = incidence(digraph(&[3.0, 1.0], &[1.0, 2.0]));
        assert_eq!(dense(&result).1, vec![-1.0, 1.0, 0.0, 1.0, 0.0, -1.0]);
        // case `undirected-reversed-input`
        let result = incidence(graph(&[3.0], &[1.0]));
        assert_eq!(dense(&result).1, vec![-1.0, 0.0, 1.0]);
    }

    /// incidence.error-self-loop-graph (cases `self-loop-graph`,
    /// `self-loop-digraph`), RunMat namespace.
    #[test]
    fn rejects_incidence_self_loops() {
        for g in [
            graph(&[1.0, 1.0], &[1.0, 2.0]),
            digraph(&[1.0, 1.0], &[1.0, 2.0]),
        ] {
            let err = call("incidence", &[g]).unwrap_err();
            assert_eq!(identifier(err), "RunMat:incidence:SelfLoops");
        }
    }

    /// incidence.error-matrix (case `err-matrix`), RunMat namespace.
    #[test]
    fn rejects_incidence_matrix_input() {
        let magic3 = matrix(3, 3, vec![8.0, 3.0, 4.0, 1.0, 5.0, 9.0, 6.0, 7.0, 2.0]);
        for value in [magic3, Value::Num(1.0), names(&["a"])] {
            let err = call("incidence", &[value]).unwrap_err();
            assert_eq!(identifier(err), "RunMat:incidence:UndefinedFunction");
        }
    }

    /// Documented choice (incidence.q-named-nodes): named nodes give the same
    /// matrix, rows in node order.
    #[test]
    fn unresolved_choice_incidence_named_nodes() {
        let named = call("graph", &[names(&["a", "b"]), names(&["b", "c"])]).unwrap();
        let expected = sparse(
            3,
            2,
            vec![0, 2, 4],
            vec![0, 1, 1, 2],
            vec![-1.0, 1.0, -1.0, 1.0],
        );
        assert_eq!(incidence(named), expected);
    }

    /// Documented choice (U-036-1): only incidence(G) is specified.
    #[test]
    fn unresolved_choice_incidence_extra_arguments_and_arity() {
        let g = graph(&[1.0, 2.0], &[2.0, 3.0]);
        let err = call("incidence", &[g, Value::Num(1.0)]).unwrap_err();
        assert_eq!(identifier(err), "RunMat:incidence:UnsupportedOption");
        let err = call("incidence", &[]).unwrap_err();
        assert_eq!(identifier(err), "RunMat:incidence:InvalidInput");
    }

    // ---- isisomorphic: observed (normative) ------------------------------

    /// isisomorphic.signature-primary, isisomorphic.output-class,
    /// isisomorphic.true-cases, isisomorphic.weights-ignored (cases
    /// `identical`, `relabelled`, `warn-call`, `directed-relabelled`,
    /// `self-loop`, `multigraph-relabelled`, `weights-ignored`, `empty`).
    #[test]
    fn observed_isisomorphic_true_cases_match_export() {
        let path = graph(&[1.0, 2.0], &[2.0, 3.0]);
        let cases = [
            (path.clone(), graph(&[1.0, 2.0], &[2.0, 3.0])),
            (path.clone(), graph(&[2.0, 1.0], &[1.0, 3.0])),
            (
                digraph(&[1.0, 2.0], &[2.0, 3.0]),
                digraph(&[3.0, 2.0], &[2.0, 1.0]),
            ),
            (
                graph(&[1.0, 1.0], &[1.0, 2.0]),
                graph(&[2.0, 2.0], &[2.0, 1.0]),
            ),
            (
                graph(&[1.0, 1.0, 2.0], &[2.0, 2.0, 3.0]),
                graph(&[3.0, 3.0, 2.0], &[2.0, 2.0, 1.0]),
            ),
            (
                call(
                    "graph",
                    &[Value::Num(1.0), Value::Num(2.0), Value::Num(5.0)],
                )
                .unwrap(),
                call(
                    "graph",
                    &[Value::Num(1.0), Value::Num(2.0), Value::Num(7.0)],
                )
                .unwrap(),
            ),
            (call("graph", &[]).unwrap(), call("graph", &[]).unwrap()),
        ];
        for (idx, (a, b)) in cases.iter().enumerate() {
            assert_eq!(iso(a, b), Value::Bool(true), "true case {idx}");
        }
    }

    /// isisomorphic.output-class, isisomorphic.false-cases,
    /// isisomorphic.direction (cases `path-vs-star`, `node-count`,
    /// `edge-count`, `self-loop-vs-none`, `self-loop-vs-parallel`,
    /// `multigraph`, `isolated`, `directed-different`).
    #[test]
    fn observed_isisomorphic_false_cases_match_export() {
        let path = graph(&[1.0, 2.0], &[2.0, 3.0]);
        let self_loop = graph(&[1.0, 1.0], &[1.0, 2.0]);
        let cases = [
            (
                graph(&[1.0, 2.0, 3.0], &[2.0, 3.0, 4.0]),
                graph(&[1.0, 1.0, 1.0], &[2.0, 3.0, 4.0]),
            ),
            (path.clone(), graph(&[1.0, 2.0, 3.0], &[2.0, 3.0, 4.0])),
            (path.clone(), graph(&[1.0, 2.0, 1.0], &[2.0, 3.0, 3.0])),
            (self_loop.clone(), path.clone()),
            (self_loop.clone(), graph(&[1.0, 1.0], &[2.0, 2.0])),
            (
                graph(&[1.0, 1.0, 2.0], &[2.0, 2.0, 3.0]),
                graph(&[1.0, 2.0, 3.0], &[2.0, 3.0, 1.0]),
            ),
            (edge_plus_isolated(3), edge_plus_isolated(4)),
            (
                digraph(&[1.0, 2.0], &[2.0, 3.0]),
                digraph(&[1.0, 3.0], &[2.0, 2.0]),
            ),
        ];
        for (idx, (a, b)) in cases.iter().enumerate() {
            assert_eq!(iso(a, b), Value::Bool(false), "false case {idx}");
        }
    }

    /// isisomorphic.error-mixed (case `err-mixed`), RunMat namespace.
    #[test]
    fn rejects_isisomorphic_graph_with_digraph() {
        let g = graph(&[1.0, 2.0], &[2.0, 3.0]);
        let d = digraph(&[1.0, 2.0], &[2.0, 3.0]);
        for (a, b) in [(&g, &d), (&d, &g)] {
            let err = call("isisomorphic", &[a.clone(), b.clone()]).unwrap_err();
            assert_eq!(identifier(err), "RunMat:isisomorphic:ClassMismatch");
        }
    }

    /// isisomorphic.error-matrix (case `err-matrix`), RunMat namespace.
    #[test]
    fn rejects_isisomorphic_matrix_inputs() {
        let eye2 = matrix(2, 2, vec![1.0, 0.0, 0.0, 1.0]);
        let g = graph(&[1.0, 2.0], &[2.0, 3.0]);
        for (a, b) in [
            (eye2.clone(), eye2.clone()),
            (g.clone(), Value::Num(1.0)),
            (eye2, g),
        ] {
            let err = call("isisomorphic", &[a, b]).unwrap_err();
            assert_eq!(identifier(err), "RunMat:isisomorphic:UndefinedFunction");
        }
    }

    /// Unresolved isisomorphic.q-options and isisomorphic.q-second-output.
    #[test]
    fn unresolved_choice_isisomorphic_options_and_second_output() {
        let g = graph(&[1.0, 2.0], &[2.0, 3.0]);
        let err = call(
            "isisomorphic",
            &[
                g.clone(),
                g.clone(),
                Value::String("EdgeVariables".into()),
                Value::String("Weight".into()),
            ],
        )
        .unwrap_err();
        assert_eq!(identifier(err), "RunMat:isisomorphic:UnsupportedOption");
        // Same pattern as upstream's treelayout output-count test.
        let _guard = crate::output_count::push_output_count(Some(2));
        let err = block_on(isisomorphic_builtin(vec![g.clone(), g])).unwrap_err();
        assert_eq!(identifier(err), "RunMat:isisomorphic:UnsupportedOption");
    }

    /// Documented choice (U-036-6): fewer than two inputs.
    #[test]
    fn unresolved_choice_isisomorphic_arity() {
        let g = graph(&[1.0, 2.0], &[2.0, 3.0]);
        let err = call("isisomorphic", &[g]).unwrap_err();
        assert_eq!(identifier(err), "RunMat:isisomorphic:InvalidInput");
        let err = call("isisomorphic", &[]).unwrap_err();
        assert_eq!(identifier(err), "RunMat:isisomorphic:InvalidInput");
    }

    /// Documented choice (isomorphism by definition, multigraph
    /// multiplicities): equal degree sequences that are not isomorphic,
    /// larger relabellings, and node names ignored.
    #[test]
    fn unresolved_choice_isisomorphic_search_cases() {
        let c6 = graph(
            &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            &[2.0, 3.0, 4.0, 5.0, 6.0, 1.0],
        );
        let two_triangles = graph(
            &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            &[2.0, 3.0, 1.0, 5.0, 6.0, 4.0],
        );
        assert_eq!(iso(&c6, &two_triangles), Value::Bool(false));
        let c6_perm = graph(
            &[4.0, 1.0, 6.0, 2.0, 5.0, 3.0],
            &[1.0, 6.0, 2.0, 5.0, 3.0, 4.0],
        );
        assert_eq!(iso(&c6, &c6_perm), Value::Bool(true));
        // same degree sequence, different multiplicities
        let double_plus_single = graph(&[1.0, 1.0, 3.0], &[2.0, 2.0, 4.0]);
        let path4 = graph(&[1.0, 2.0, 3.0], &[2.0, 3.0, 4.0]);
        assert_eq!(iso(&double_plus_single, &path4), Value::Bool(false));
        let path = graph(&[1.0, 2.0], &[2.0, 3.0]);
        let named = call("graph", &[names(&["x", "y"]), names(&["y", "z"])]).unwrap();
        assert_eq!(iso(&path, &named), Value::Bool(true));
    }
}
