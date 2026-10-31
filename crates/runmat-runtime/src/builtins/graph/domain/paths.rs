//! Clean-room `allpaths` and `inedges` (graph path and edge queries).
//!
//! Clean-room feature `specs/059-graph-n3n4/`. Normative behaviour comes only
//! from the approved exports `allpaths` and `inedges` 0.2.0 (see that
//! feature's `provenance.md`). Edge indices refer to RunMat's stored edge
//! list (`G.Edges` row order).

use super::algorithms::tensor_value;
use super::construction::graph_data;
use super::parsing::{numeric_f64_vector, parse_string_vector};
use super::*;
use runmat_value::{CellArray, CharArray};

// ---------------------------------------------------------------------------
// Descriptors
// ---------------------------------------------------------------------------

const fn param(name: &'static str, description: &'static str) -> BuiltinParamDescriptor {
    BuiltinParamDescriptor {
        name,
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Required,
        default: None,
        description,
    }
}

const P_GRAPH: BuiltinParamDescriptor = param("G", "Graph or digraph object.");

const ALLPATHS_INPUTS: [BuiltinParamDescriptor; 3] = [
    P_GRAPH,
    param("s", "Source node index or name."),
    param("t", "Target node index or name."),
];
const ALLPATHS_OUTPUT_P: BuiltinParamDescriptor = param(
    "P",
    "Cell column of paths (node index rows, or cellstr rows of names).",
);
const ALLPATHS_OUTPUTS: [BuiltinParamDescriptor; 1] = [ALLPATHS_OUTPUT_P];
const ALLPATHS_OUTPUTS_TWO: [BuiltinParamDescriptor; 2] = [
    ALLPATHS_OUTPUT_P,
    param("E", "Cell column of edge index rows, one per path."),
];
const ALLPATHS_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "P = allpaths(G, s, t)",
        inputs: &ALLPATHS_INPUTS,
        outputs: &ALLPATHS_OUTPUTS,
    },
    BuiltinSignatureDescriptor {
        label: "[P, E] = allpaths(G, s, t)",
        inputs: &ALLPATHS_INPUTS,
        outputs: &ALLPATHS_OUTPUTS_TWO,
    },
];

const INEDGES_INPUTS: [BuiltinParamDescriptor; 2] = [
    param("G", "Digraph object."),
    param("node", "One node, as index or name."),
];
const INEDGES_OUTPUT_EID: BuiltinParamDescriptor =
    param("eid", "Column of incoming edge indices (double).");
const INEDGES_OUTPUTS: [BuiltinParamDescriptor; 1] = [INEDGES_OUTPUT_EID];
const INEDGES_OUTPUTS_TWO: [BuiltinParamDescriptor; 2] = [
    INEDGES_OUTPUT_EID,
    param(
        "nid",
        "Column of source nodes (indices, or a cell of names).",
    ),
];
const INEDGES_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "eid = inedges(G, node)",
        inputs: &INEDGES_INPUTS,
        outputs: &INEDGES_OUTPUTS,
    },
    BuiltinSignatureDescriptor {
        label: "[eid, nid] = inedges(G, node)",
        inputs: &INEDGES_INPUTS,
        outputs: &INEDGES_OUTPUTS_TWO,
    },
];

macro_rules! query_error {
    ($fn:literal, $prefix:literal, $code:literal, $id:literal, $when:literal) => {
        BuiltinErrorDescriptor {
            code: concat!("RM.", $prefix, ".", $code),
            identifier: Some(concat!("RunMat:", $fn, ":", $id)),
            when: $when,
            message: concat!($fn, ": ", $when),
        }
    };
}

/// Common errors (U-059-1..3) followed by the builtin's specific ones.
macro_rules! query_errors {
    (
        $name:ident, $fn:literal, $prefix:literal;
        $(($code:literal, $id:literal, $when:literal)),* $(,)?
    ) => {
        const $name: &[BuiltinErrorDescriptor] = &[
            query_error!($fn, $prefix, "INVALID_INPUT", "InvalidInput",
                "A required input is missing or an input is malformed."),
            query_error!($fn, $prefix, "UNSUPPORTED_OPTION", "UnsupportedOption",
                "The call uses a form or option that is not yet specified for RunMat."),
            query_error!($fn, $prefix, "INTERNAL", "Internal",
                "RunMat cannot build the result value."),
            query_error!($fn, $prefix, "UNDEFINED_FUNCTION", "UndefinedFunction",
                "The first input is not a graph or digraph object."),
            query_error!($fn, $prefix, "TOO_MANY_OUTPUTS", "TooManyOutputs",
                "More than two outputs were requested."),
            $(query_error!($fn, $prefix, $code, $id, $when),)*
        ];
    };
}

query_errors!(
    ALLPATHS_ERRORS, "allpaths", "ALLPATHS";
    ("INVALID_NODE_ID", "InvalidNodeID", "A node index is not a node of the graph."),
    ("UNKNOWN_NODE_NAME", "UnknownNodeName", "A node name is not a node of the graph."),
);
query_errors!(
    INEDGES_ERRORS, "inedges", "INEDGES";
    ("NO_IN_EDGES_UNDIR", "NoInEdgesUndir", "The input is an undirected graph."),
    ("INVALID_NODE_ID", "InvalidNodeID", "The node index is not a node of the digraph."),
    ("INVALID_NODE_SCALAR", "InvalidNodeScalar", "Other than exactly one node was given."),
    ("UNKNOWN_NODE_NAME", "UnknownNodeName", "The node name is not a node of the digraph."),
);

pub const ALLPATHS_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &ALLPATHS_SIGNATURES,
    output_mode: BuiltinOutputMode::ByRequestedOutputCount,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: ALLPATHS_ERRORS,
};
pub const INEDGES_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &INEDGES_SIGNATURES,
    output_mode: BuiltinOutputMode::ByRequestedOutputCount,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: INEDGES_ERRORS,
};

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Raise `RunMat:<builtin>:<id>` from the builtin's descriptor table.
pub(super) fn query_error(
    builtin: &'static str,
    errors: &'static [BuiltinErrorDescriptor],
    id: &str,
    detail: impl Into<String>,
) -> RuntimeError {
    let suffix = format!(":{id}");
    let descriptor = errors
        .iter()
        .find(|d| d.identifier.is_some_and(|ident| ident.ends_with(&suffix)))
        .or_else(|| errors.get(2))
        .cloned()
        .unwrap_or(ERROR_INTERNAL);
    error_from_descriptor(builtin, descriptor, format!("{builtin}: {}", detail.into()))
}

/// Read the graph argument. Anything that is not a `graph`/`digraph` object
/// is `UndefinedFunction` (U-059-1); a malformed object is `InvalidInput`.
pub(super) fn graph_input(
    builtin: &'static str,
    errors: &'static [BuiltinErrorDescriptor],
    value: &Value,
) -> BuiltinResult<GraphData> {
    let is_graph = matches!(
        value,
        Value::Object(object)
            if object.class_name == GRAPH_CLASS || object.class_name == DIGRAPH_CLASS
    );
    if !is_graph {
        return Err(query_error(
            builtin,
            errors,
            "UndefinedFunction",
            "undefined for input that is not a graph or digraph object",
        ));
    }
    graph_data(value, builtin).map_err(|err| {
        query_error(
            builtin,
            errors,
            "InvalidInput",
            format!("invalid graph object: {}", err.message()),
        )
    })
}

/// U-059-3: at most two outputs.
pub(super) fn check_output_count(
    builtin: &'static str,
    errors: &'static [BuiltinErrorDescriptor],
) -> BuiltinResult<Option<usize>> {
    match crate::output_count::current_output_count() {
        Some(count) if count > 2 => Err(query_error(
            builtin,
            errors,
            "TooManyOutputs",
            format!("{builtin} returns at most two outputs, {count} were requested"),
        )),
        other => Ok(other),
    }
}

/// Package one or two outputs for the requested output count.
pub(super) fn outputs(
    count: Option<usize>,
    first: Value,
    second: impl FnOnce() -> BuiltinResult<Value>,
) -> BuiltinResult<Value> {
    Ok(match count {
        None => first,
        Some(0) => Value::OutputList(Vec::new()),
        Some(1) => Value::OutputList(vec![first]),
        Some(_) => Value::OutputList(vec![first, second()?]),
    })
}

/// Outcome of resolving one node argument.
pub(super) enum NodeArg {
    Node(usize),
    NotScalar,
    InvalidId,
    UnknownName,
    Malformed,
}

/// Text values are names, numeric values one-based indices.
pub(super) fn resolve_node(graph: &GraphData, value: &Value) -> NodeArg {
    match value {
        Value::String(_) | Value::StringArray(_) | Value::CharArray(_) | Value::Cell(_) => {
            let Ok(labels) = parse_string_vector(value) else {
                return NodeArg::Malformed;
            };
            if labels.len() != 1 {
                return NodeArg::NotScalar;
            }
            graph
                .node_names
                .as_ref()
                .and_then(|names| names.iter().position(|name| name == &labels[0]))
                .map_or(NodeArg::UnknownName, NodeArg::Node)
        }
        Value::Num(_)
        | Value::Int(_)
        | Value::Bool(_)
        | Value::Tensor(_)
        | Value::LogicalArray(_) => {
            let Ok(values) = numeric_f64_vector(value, "graph") else {
                return NodeArg::Malformed;
            };
            if values.len() != 1 {
                return NodeArg::NotScalar;
            }
            let x = values[0];
            if x.is_finite() && x >= 1.0 && x.fract() == 0.0 && x <= graph.node_count as f64 {
                NodeArg::Node(x as usize - 1)
            } else {
                NodeArg::InvalidId
            }
        }
        _ => NodeArg::Malformed,
    }
}

pub(super) fn double(
    builtin: &'static str,
    errors: &'static [BuiltinErrorDescriptor],
    data: Vec<f64>,
    rows: usize,
    cols: usize,
) -> BuiltinResult<Value> {
    tensor_value(data, vec![rows, cols], builtin)
        .map_err(|err| query_error(builtin, errors, "Internal", err.message()))
}

pub(super) fn cell(
    builtin: &'static str,
    errors: &'static [BuiltinErrorDescriptor],
    values: Vec<Value>,
    rows: usize,
    cols: usize,
) -> BuiltinResult<Value> {
    CellArray::new(values, rows, cols)
        .map(Value::Cell)
        .map_err(|err| query_error(builtin, errors, "Internal", err))
}

pub(super) fn name_value(name: &str) -> Value {
    Value::CharArray(CharArray::new_row(name))
}

// ---------------------------------------------------------------------------
// allpaths
// ---------------------------------------------------------------------------

/// Every simple path from `source` to `target` as (nodes, edges), depth
/// first with neighbours in increasing node index; one path per node
/// sequence, using the lowest-index parallel edge; self-loops skipped
/// (FR-059-07..09, U-059-6).
fn simple_paths(graph: &GraphData, source: usize, target: usize) -> Vec<(Vec<usize>, Vec<usize>)> {
    if source == target {
        return vec![(vec![source], Vec::new())];
    }
    let n = graph.node_count;
    let mut adjacency: Vec<std::collections::BTreeMap<usize, usize>> =
        vec![std::collections::BTreeMap::new(); n];
    for (k, edge) in graph.edges.iter().enumerate() {
        if edge.source == edge.target {
            continue;
        }
        adjacency[edge.source].entry(edge.target).or_insert(k);
        if !graph.directed {
            adjacency[edge.target].entry(edge.source).or_insert(k);
        }
    }
    let adjacency: Vec<Vec<(usize, usize)>> = adjacency
        .into_iter()
        .map(|map| map.into_iter().collect())
        .collect();
    let mut found = Vec::new();
    let mut on_path = vec![false; n];
    let mut nodes = vec![source];
    let mut edges: Vec<usize> = Vec::new();
    let mut cursor = vec![0usize];
    on_path[source] = true;
    while let Some(&u) = nodes.last() {
        let depth = nodes.len() - 1;
        if let Some(&(v, e)) = adjacency[u].get(cursor[depth]) {
            cursor[depth] += 1;
            if on_path[v] {
                continue;
            }
            if v == target {
                let mut path = nodes.clone();
                path.push(v);
                let mut path_edges = edges.clone();
                path_edges.push(e);
                found.push((path, path_edges));
                continue;
            }
            nodes.push(v);
            edges.push(e);
            cursor.push(0);
            on_path[v] = true;
        } else {
            on_path[u] = false;
            nodes.pop();
            edges.pop();
            cursor.pop();
        }
    }
    found
}

#[runtime_builtin(
    name = "allpaths",
    category = "graph",
    summary = "Return all paths between two nodes of a graph or digraph.",
    keywords = "allpaths,graph,digraph,paths,simple paths,enumerate",
    accel = "cpu",
    type_resolver(any_type),
    descriptor(crate::builtins::graph::domain::paths::ALLPATHS_DESCRIPTOR),
    builtin_path = "crate::builtins::graph"
)]
pub(super) async fn allpaths_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    const NAME: &str = "allpaths";
    const ERRORS: &[BuiltinErrorDescriptor] = ALLPATHS_ERRORS;
    let args = gather_values(args).await?;
    let count = check_output_count(NAME, ERRORS)?;
    let Some(first) = args.first() else {
        return Err(query_error(
            NAME,
            ERRORS,
            "InvalidInput",
            "expected allpaths(G, s, t)",
        ));
    };
    let graph = graph_input(NAME, ERRORS, first)?;
    if args.len() < 3 {
        return Err(query_error(
            NAME,
            ERRORS,
            "InvalidInput",
            "expected allpaths(G, s, t)",
        ));
    }
    if args.len() > 3 {
        // U-059-2 (allpaths.q-more)
        return Err(query_error(
            NAME,
            ERRORS,
            "UnsupportedOption",
            "only allpaths(G, s, t) is specified",
        ));
    }
    let mut ends = [0usize; 2];
    for (slot, value) in ends.iter_mut().zip(&args[1..3]) {
        *slot = match resolve_node(&graph, value) {
            NodeArg::Node(index) => index,
            // allpaths.error-node-range; U-059-6
            NodeArg::InvalidId => {
                return Err(query_error(
                    NAME,
                    ERRORS,
                    "InvalidNodeID",
                    "node index is not a node of the graph",
                ))
            }
            // allpaths.error-unknown-name; U-059-6
            NodeArg::UnknownName => {
                return Err(query_error(
                    NAME,
                    ERRORS,
                    "UnknownNodeName",
                    "node name is not a node of the graph",
                ))
            }
            NodeArg::NotScalar | NodeArg::Malformed => {
                return Err(query_error(
                    NAME,
                    ERRORS,
                    "InvalidInput",
                    "s and t must each be one node index or name",
                ))
            }
        };
    }
    let paths = simple_paths(&graph, ends[0], ends[1]);
    let count_paths = paths.len();
    let mut node_cells = Vec::with_capacity(count_paths);
    let mut edge_cells = Vec::with_capacity(count_paths);
    for (nodes, edges) in paths {
        // allpaths.paths, allpaths.names
        node_cells.push(match &graph.node_names {
            Some(names) => cell(
                NAME,
                ERRORS,
                nodes.iter().map(|&i| name_value(&names[i])).collect(),
                1,
                nodes.len(),
            )?,
            None => double(
                NAME,
                ERRORS,
                nodes.iter().map(|&i| (i + 1) as f64).collect(),
                1,
                nodes.len(),
            )?,
        });
        // allpaths.edgepaths
        edge_cells.push(double(
            NAME,
            ERRORS,
            edges.iter().map(|&k| (k + 1) as f64).collect(),
            1,
            edges.len(),
        )?);
    }
    let p = cell(NAME, ERRORS, node_cells, count_paths, 1)?;
    outputs(count, p, || cell(NAME, ERRORS, edge_cells, count_paths, 1))
}

// ---------------------------------------------------------------------------
// inedges
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "inedges",
    category = "graph",
    summary = "Return the incoming edges of a digraph node and their source nodes.",
    keywords = "inedges,digraph,edges,incoming,predecessors",
    accel = "cpu",
    type_resolver(any_type),
    descriptor(crate::builtins::graph::domain::paths::INEDGES_DESCRIPTOR),
    builtin_path = "crate::builtins::graph"
)]
pub(super) async fn inedges_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    const NAME: &str = "inedges";
    const ERRORS: &[BuiltinErrorDescriptor] = INEDGES_ERRORS;
    let args = gather_values(args).await?;
    let count = check_output_count(NAME, ERRORS)?;
    let Some(first) = args.first() else {
        return Err(query_error(
            NAME,
            ERRORS,
            "InvalidInput",
            "expected inedges(G, node)",
        ));
    };
    let graph = graph_input(NAME, ERRORS, first)?;
    if args.len() < 2 {
        return Err(query_error(
            NAME,
            ERRORS,
            "InvalidInput",
            "expected inedges(G, node)",
        ));
    }
    if args.len() > 2 {
        // U-059-2
        return Err(query_error(
            NAME,
            ERRORS,
            "UnsupportedOption",
            "only inedges(G, node) is specified",
        ));
    }
    if !graph.directed {
        // inedges.error-undirected; U-059-7 (any output count)
        return Err(query_error(
            NAME,
            ERRORS,
            "NoInEdgesUndir",
            "in-edges are only defined for digraphs",
        ));
    }
    let node = match resolve_node(&graph, &args[1]) {
        NodeArg::Node(index) => index,
        // inedges.error-node-range; U-059-7
        NodeArg::InvalidId => {
            return Err(query_error(
                NAME,
                ERRORS,
                "InvalidNodeID",
                "node index is not a node of the digraph",
            ))
        }
        // inedges.error-two-nodes; U-059-7
        NodeArg::NotScalar => {
            return Err(query_error(
                NAME,
                ERRORS,
                "InvalidNodeScalar",
                "exactly one node must be given",
            ))
        }
        // U-059-7
        NodeArg::UnknownName => {
            return Err(query_error(
                NAME,
                ERRORS,
                "UnknownNodeName",
                "node name is not a node of the digraph",
            ))
        }
        NodeArg::Malformed => {
            return Err(query_error(
                NAME,
                ERRORS,
                "InvalidInput",
                "node must be an index or a name",
            ))
        }
    };
    // inedges.outputs, inedges.order, inedges.selfloop: stored order.
    let incoming: Vec<(usize, usize)> = graph
        .edges
        .iter()
        .enumerate()
        .filter(|(_, edge)| edge.target == node)
        .map(|(k, edge)| (k, edge.source))
        .collect();
    let k = incoming.len();
    // inedges.none: 0x1
    let eid = double(
        NAME,
        ERRORS,
        incoming.iter().map(|&(e, _)| (e + 1) as f64).collect(),
        k,
        1,
    )?;
    outputs(count, eid, || match &graph.node_names {
        // inedges.names
        Some(names) => cell(
            NAME,
            ERRORS,
            incoming
                .iter()
                .map(|&(_, s)| name_value(&names[s]))
                .collect(),
            k,
            1,
        ),
        None => double(
            NAME,
            ERRORS,
            incoming.iter().map(|&(_, s)| (s + 1) as f64).collect(),
            k,
            1,
        ),
    })
}

#[cfg(test)]
mod graph_n3n4_paths_tests {
    use super::*;
    use futures::executor::block_on;

    // ---- helpers ----------------------------------------------------------

    fn row(values: &[f64]) -> Value {
        Value::Tensor(Tensor::new(values.to_vec(), vec![1, values.len()]).unwrap())
    }

    fn chr(text: &str) -> Value {
        Value::CharArray(CharArray::new_row(text))
    }

    fn num(value: f64) -> Value {
        Value::Num(value)
    }

    fn cellstr_row(values: &[&str]) -> Value {
        Value::Cell(
            CellArray::new(values.iter().map(|s| chr(s)).collect(), 1, values.len()).unwrap(),
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

    fn identifier(err: RuntimeError) -> String {
        err.identifier().unwrap_or("<none>").to_string()
    }

    fn err_id(name: &str, args: &[Value]) -> String {
        identifier(call(name, args).expect_err("expected an error"))
    }

    /// `(rows, cols, values)` of a double result (`Num` is 1x1).
    fn numeric(value: &Value) -> (usize, usize, Vec<f64>) {
        match value {
            Value::Num(n) => (1, 1, vec![*n]),
            Value::Tensor(t) => (t.rows, t.cols, tensor::tensor_values_f64(t)),
            other => panic!("expected double, got {other:?}"),
        }
    }

    fn text(value: &Value) -> String {
        match value {
            Value::CharArray(c) if c.rows == 1 => c.data.iter().collect(),
            other => panic!("expected char row, got {other:?}"),
        }
    }

    /// A path element: numeric row, or cellstr row of names.
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    enum Row {
        Nums(Vec<i64>),
        Names(Vec<String>),
    }

    fn path_row(value: &Value) -> Row {
        match value {
            Value::Cell(c) => {
                assert_eq!(c.rows, 1, "cellstr path must be a row");
                Row::Names(c.data.iter().map(text).collect())
            }
            other => {
                let (r, c, v) = numeric(other);
                assert!(r == 1 || (r, c) == (1, 0), "path must be a row: {r}x{c}");
                Row::Nums(v.into_iter().map(|x| x as i64).collect())
            }
        }
    }

    /// Cell column -> rows (asserting n x 1, 0x1 when empty).
    fn cell_column(value: &Value) -> Vec<Row> {
        let Value::Cell(cell) = value else {
            panic!("expected cell, got {value:?}");
        };
        assert_eq!(
            cell.cols, 1,
            "expected a cell column, got {}x{}",
            cell.rows, cell.cols
        );
        cell.data.iter().map(path_row).collect()
    }

    fn nums(rows: &[&[i64]]) -> Vec<Row> {
        rows.iter().map(|r| Row::Nums(r.to_vec())).collect()
    }

    fn sorted(mut rows: Vec<Row>) -> Vec<Row> {
        rows.sort();
        rows
    }

    fn allpaths(args: &[Value]) -> Vec<Row> {
        cell_column(&call("allpaths", args).expect("allpaths"))
    }

    fn multi(
        f: fn(Vec<Value>) -> futures::future::LocalBoxFuture<'static, BuiltinResult<Value>>,
        args: &[Value],
        count: usize,
    ) -> BuiltinResult<Vec<Value>> {
        let _guard = crate::output_count::push_output_count(Some(count));
        match block_on(f(args.to_vec()))? {
            Value::OutputList(values) => Ok(values),
            other => panic!("expected output list, got {other:?}"),
        }
    }

    fn allpaths_boxed(
        args: Vec<Value>,
    ) -> futures::future::LocalBoxFuture<'static, BuiltinResult<Value>> {
        Box::pin(allpaths_builtin(args))
    }

    fn inedges_boxed(
        args: Vec<Value>,
    ) -> futures::future::LocalBoxFuture<'static, BuiltinResult<Value>> {
        Box::pin(inedges_builtin(args))
    }

    /// [P, E] = allpaths(args...) as (node path, edge path) pairs.
    fn allpaths2(args: &[Value]) -> Vec<(Row, Row)> {
        let out = multi(allpaths_boxed, args, 2).expect("allpaths");
        assert_eq!(out.len(), 2);
        let (p, e) = (cell_column(&out[0]), cell_column(&out[1]));
        assert_eq!(p.len(), e.len());
        p.into_iter().zip(e).collect()
    }

    /// [eid, nid] = inedges(args...).
    fn inedges2(args: &[Value]) -> (Value, Value) {
        let out = multi(inedges_boxed, args, 2).expect("inedges");
        assert_eq!(out.len(), 2);
        (out[0].clone(), out[1].clone())
    }

    fn cycle4() -> Value {
        graph(&[1.0, 2.0, 3.0, 1.0], &[2.0, 3.0, 4.0, 4.0])
    }

    fn named(s: &[&str], t: &[&str]) -> Value {
        call("graph", &[cellstr_row(s), cellstr_row(t)]).expect("named graph")
    }

    fn named_digraph(s: &[&str], t: &[&str]) -> Value {
        call("digraph", &[cellstr_row(s), cellstr_row(t)]).expect("named digraph")
    }

    /// Stored end nodes (0-based) of a graph.
    fn stored(g: &Value) -> Vec<(usize, usize)> {
        graph_data(g, "test")
            .unwrap()
            .edges
            .iter()
            .map(|e| (e.source, e.target))
            .collect()
    }

    // ---- allpaths: observed (normative) ---------------------------------

    /// allpaths.signature-primary, allpaths.paths (cases `path`, `cycle`).
    #[test]
    fn observed_allpaths_path_and_cycle() {
        let p = allpaths(&[graph(&[1.0, 2.0], &[2.0, 3.0]), num(1.0), num(3.0)]);
        assert_eq!(p, nums(&[&[1, 2, 3]]));
        let p = allpaths(&[cycle4(), num(1.0), num(3.0)]);
        assert_eq!(sorted(p), nums(&[&[1, 2, 3], &[1, 4, 3]]));
    }

    /// allpaths.digraph (cases `digraph`, `digraph-reverse`).
    #[test]
    fn observed_allpaths_digraph_direction() {
        let d = digraph(&[1.0, 2.0, 1.0], &[2.0, 3.0, 3.0]);
        let p = allpaths(&[d, num(1.0), num(3.0)]);
        assert_eq!(sorted(p), nums(&[&[1, 2, 3], &[1, 3]]));
        let d = digraph(&[1.0, 2.0], &[2.0, 3.0]);
        let out = call("allpaths", &[d, num(3.0), num(1.0)]).unwrap();
        let Value::Cell(cell) = &out else {
            panic!("expected cell");
        };
        assert_eq!((cell.rows, cell.cols), (0, 1));
    }

    /// allpaths.same-node: {2}.
    #[test]
    fn observed_allpaths_same_node() {
        let p = allpaths(&[cycle4(), num(2.0), num(2.0)]);
        assert_eq!(p, nums(&[&[2]]));
    }

    /// allpaths.unreachable: 0x1 cell.
    #[test]
    fn observed_allpaths_unreachable() {
        let g = graph(&[1.0, 3.0], &[2.0, 4.0]);
        let out = call("allpaths", &[g, num(1.0), num(4.0)]).unwrap();
        let Value::Cell(cell) = &out else {
            panic!("expected cell");
        };
        assert_eq!((cell.rows, cell.cols), (0, 1));
    }

    /// allpaths.names: cellstr rows {'a','b','c'}; {'a','c'}.
    #[test]
    fn observed_allpaths_names() {
        let g = named(&["a", "b", "a"], &["b", "c", "c"]);
        let p = allpaths(&[g, chr("a"), chr("c")]);
        let expected = vec![
            Row::Names(vec!["a".into(), "b".into(), "c".into()]),
            Row::Names(vec!["a".into(), "c".into()]),
        ];
        assert_eq!(sorted(p), sorted(expected));
    }

    /// allpaths.edgepaths on the 4-cycle stored as [1 2; 1 4; 2 3; 3 4]
    /// (the order the observed constructor stores; D-059-2).
    #[test]
    fn observed_allpaths_edge_paths() {
        let g = graph(&[1.0, 1.0, 2.0, 3.0], &[2.0, 4.0, 3.0, 4.0]);
        let pairs = allpaths2(&[g, num(1.0), num(3.0)]);
        let mut pairs = pairs;
        pairs.sort();
        assert_eq!(
            pairs,
            vec![
                (Row::Nums(vec![1, 2, 3]), Row::Nums(vec![1, 3])),
                (Row::Nums(vec![1, 4, 3]), Row::Nums(vec![2, 4])),
            ]
        );
    }

    /// allpaths.edgepaths on the observed input: each E{i} lists the stored
    /// edges joining consecutive nodes of P{i}.
    #[test]
    fn observed_allpaths_edge_paths_follow_stored_edges() {
        let g = cycle4();
        let edges = stored(&g);
        for (path, edge_path) in allpaths2(&[g, num(1.0), num(3.0)]) {
            let (Row::Nums(path), Row::Nums(edge_path)) = (path, edge_path) else {
                panic!("numeric rows expected");
            };
            assert_eq!(edge_path.len() + 1, path.len());
            for (k, &e) in edge_path.iter().enumerate() {
                let (a, b) = edges[e as usize - 1];
                let (u, v) = (path[k] as usize - 1, path[k + 1] as usize - 1);
                assert!((a, b) == (u, v) || (a, b) == (v, u));
            }
        }
    }

    /// D-059-2: observed E for graph([1 2 3 1], [2 3 4 4]) needs a sorted
    /// constructor.
    #[test]
    #[ignore = "D-059-2: RunMat's graph constructor keeps input edge order"]
    fn deviation_allpaths_observed_edge_paths_for_unsorted_input() {
        let mut pairs = allpaths2(&[cycle4(), num(1.0), num(3.0)]);
        pairs.sort();
        assert_eq!(
            pairs,
            vec![
                (Row::Nums(vec![1, 2, 3]), Row::Nums(vec![1, 3])),
                (Row::Nums(vec![1, 4, 3]), Row::Nums(vec![2, 4])),
            ]
        );
    }

    /// allpaths.error-node-range, allpaths.error-unknown-name.
    #[test]
    fn rejects_allpaths_invalid_nodes() {
        let g = graph(&[1.0, 2.0], &[2.0, 3.0]);
        assert_eq!(
            err_id("allpaths", &[g, num(1.0), num(9.0)]),
            "RunMat:allpaths:InvalidNodeID"
        );
        let n = named(&["a"], &["b"]);
        assert_eq!(
            err_id("allpaths", &[n, chr("a"), chr("z")]),
            "RunMat:allpaths:UnknownNodeName"
        );
    }

    // ---- allpaths: documented choices ------------------------------------

    /// U-059-1..3, U-059-6 input handling.
    #[test]
    fn unresolved_choice_allpaths_inputs() {
        let g = graph(&[1.0, 2.0], &[2.0, 3.0]);
        let m = Value::Tensor(Tensor::new(vec![0.0, 1.0, 1.0, 0.0], vec![2, 2]).unwrap());
        assert_eq!(
            err_id("allpaths", &[m, num(1.0), num(2.0)]),
            "RunMat:allpaths:UndefinedFunction"
        );
        assert_eq!(
            err_id("allpaths", &[g.clone(), num(1.0)]),
            "RunMat:allpaths:InvalidInput"
        );
        assert_eq!(
            err_id("allpaths", &[g.clone(), num(1.0), num(3.0), num(5.0)]),
            "RunMat:allpaths:UnsupportedOption"
        );
        assert_eq!(
            err_id("allpaths", &[g.clone(), row(&[1.0, 2.0]), num(3.0)]),
            "RunMat:allpaths:InvalidInput"
        );
        for bad in [0.0, 1.5, -1.0] {
            assert_eq!(
                err_id("allpaths", &[g.clone(), num(bad), num(3.0)]),
                "RunMat:allpaths:InvalidNodeID"
            );
        }
        assert_eq!(
            err_id("allpaths", &[g.clone(), chr("a"), num(3.0)]),
            "RunMat:allpaths:UnknownNodeName"
        );
        // numeric indices on a named graph
        let n = named(&["a", "b"], &["b", "c"]);
        let p = allpaths(&[n, num(1.0), num(3.0)]);
        assert_eq!(p.len(), 1);
        let err = multi(allpaths_boxed, &[g, num(1.0), num(3.0)], 3).unwrap_err();
        assert_eq!(identifier(err), "RunMat:allpaths:TooManyOutputs");
    }

    /// U-059-6: order, multigraphs, self-loops, s == t edge path.
    #[test]
    fn unresolved_choice_allpaths_order_multigraph_and_self_loops() {
        // lower-numbered next node first (allpaths.order, non-normative)
        let p = allpaths(&[cycle4(), num(1.0), num(3.0)]);
        assert_eq!(p, nums(&[&[1, 2, 3], &[1, 4, 3]]));
        // parallel edges: one path, lowest-index edge; self-loop ignored
        let g = graph(&[1.0, 2.0, 1.0, 2.0], &[2.0, 2.0, 2.0, 3.0]);
        let pairs = allpaths2(&[g, num(1.0), num(3.0)]);
        assert_eq!(
            pairs,
            vec![(Row::Nums(vec![1, 2, 3]), Row::Nums(vec![1, 4]))]
        );
        // s == t: E = {zeros(1, 0)}
        let pairs = allpaths2(&[cycle4(), num(2.0), num(2.0)]);
        assert_eq!(pairs, vec![(Row::Nums(vec![2]), Row::Nums(vec![]))]);
        // single output through the output list
        let out = multi(allpaths_boxed, &[cycle4(), num(1.0), num(3.0)], 1).unwrap();
        assert_eq!(out.len(), 1);
    }

    // ---- inedges: observed (normative) -----------------------------------

    /// inedges.signature-primary, inedges.outputs (cases `one-out`,
    /// `two-out`).
    #[test]
    fn observed_inedges_one_and_two_outputs() {
        let d = digraph(&[1.0, 2.0, 3.0], &[3.0, 3.0, 4.0]);
        let eid = call("inedges", &[d.clone(), num(3.0)]).unwrap();
        assert_eq!(numeric(&eid), (2, 1, vec![1.0, 2.0]));
        let (eid, nid) = inedges2(&[d, num(3.0)]);
        assert_eq!(numeric(&eid), (2, 1, vec![1.0, 2.0]));
        assert_eq!(numeric(&nid), (2, 1, vec![1.0, 2.0]));
    }

    /// inedges.order on the edges stored as the observed constructor stores
    /// them (1->2, 3->2, 4->2; D-059-1).
    #[test]
    fn observed_inedges_order() {
        let d = digraph(&[1.0, 3.0, 4.0], &[2.0, 2.0, 2.0]);
        let (eid, nid) = inedges2(&[d, num(2.0)]);
        assert_eq!(numeric(&eid), (3, 1, vec![1.0, 2.0, 3.0]));
        assert_eq!(numeric(&nid), (3, 1, vec![1.0, 3.0, 4.0]));
    }

    /// inedges.order on the observed input: eid ascending, nid the source of
    /// each stored edge.
    #[test]
    fn observed_inedges_order_follows_stored_edges() {
        let d = digraph(&[4.0, 1.0, 3.0], &[2.0, 2.0, 2.0]);
        let edges = stored(&d);
        let (eid, nid) = inedges2(&[d, num(2.0)]);
        let (_, _, eid) = numeric(&eid);
        let (_, _, nid) = numeric(&nid);
        assert_eq!(eid, vec![1.0, 2.0, 3.0]);
        for (e, n) in eid.iter().zip(&nid) {
            assert_eq!(edges[*e as usize - 1].0 + 1, *n as usize);
        }
    }

    /// D-059-1: observed nid for digraph([4 1 3], [2 2 2]) needs a sorted
    /// constructor.
    #[test]
    #[ignore = "D-059-1: RunMat's digraph constructor keeps input edge order"]
    fn deviation_inedges_observed_order_for_unsorted_input() {
        let d = digraph(&[4.0, 1.0, 3.0], &[2.0, 2.0, 2.0]);
        let (eid, nid) = inedges2(&[d, num(2.0)]);
        assert_eq!(numeric(&eid), (3, 1, vec![1.0, 2.0, 3.0]));
        assert_eq!(numeric(&nid), (3, 1, vec![1.0, 3.0, 4.0]));
    }

    /// inedges.none: 0x1 double eid and nid.
    #[test]
    fn observed_inedges_none() {
        let d = digraph(&[1.0, 2.0], &[2.0, 3.0]);
        let (eid, nid) = inedges2(&[d, num(1.0)]);
        assert_eq!(numeric(&eid), (0, 1, vec![]));
        assert_eq!(numeric(&nid), (0, 1, vec![]));
    }

    /// inedges.names: nid a 2x1 cell {'a';'b'}, eid 2x1 double.
    #[test]
    fn observed_inedges_names() {
        let d = named_digraph(&["a", "b"], &["c", "c"]);
        let (eid, nid) = inedges2(&[d, chr("c")]);
        assert_eq!(numeric(&eid), (2, 1, vec![1.0, 2.0]));
        let Value::Cell(cell) = &nid else {
            panic!("expected cell, got {nid:?}");
        };
        assert_eq!((cell.rows, cell.cols), (2, 1));
        let names: Vec<String> = cell.data.iter().map(text).collect();
        assert_eq!(names, vec!["a", "b"]);
    }

    /// inedges.selfloop.
    #[test]
    fn observed_inedges_self_loop() {
        let d = digraph(&[1.0, 2.0], &[1.0, 1.0]);
        let (eid, nid) = inedges2(&[d, num(1.0)]);
        assert_eq!(numeric(&eid), (2, 1, vec![1.0, 2.0]));
        assert_eq!(numeric(&nid), (2, 1, vec![1.0, 2.0]));
    }

    /// inedges.error-undirected, error-node-range, error-two-nodes.
    #[test]
    fn rejects_inedges_errors() {
        let g = graph(&[1.0, 2.0], &[2.0, 3.0]);
        let err = multi(inedges_boxed, &[g, num(2.0)], 2).unwrap_err();
        assert_eq!(identifier(err), "RunMat:inedges:NoInEdgesUndir");
        let d = call("digraph", &[num(1.0), num(2.0)]).unwrap();
        assert_eq!(
            err_id("inedges", &[d, num(5.0)]),
            "RunMat:inedges:InvalidNodeID"
        );
        let d = digraph(&[1.0, 2.0], &[3.0, 3.0]);
        assert_eq!(
            err_id("inedges", &[d, row(&[1.0, 3.0])]),
            "RunMat:inedges:InvalidNodeScalar"
        );
    }

    // ---- inedges: documented choices --------------------------------------

    /// U-059-1..3, U-059-7.
    #[test]
    fn unresolved_choice_inedges_inputs() {
        let d = digraph(&[1.0, 2.0], &[3.0, 3.0]);
        let g = graph(&[1.0, 2.0], &[2.0, 3.0]);
        assert_eq!(
            err_id("inedges", &[g, num(2.0)]),
            "RunMat:inedges:NoInEdgesUndir"
        );
        let m = Value::Tensor(Tensor::new(vec![0.0, 1.0, 1.0, 0.0], vec![2, 2]).unwrap());
        assert_eq!(
            err_id("inedges", &[m, num(1.0)]),
            "RunMat:inedges:UndefinedFunction"
        );
        assert_eq!(
            err_id("inedges", std::slice::from_ref(&d)),
            "RunMat:inedges:InvalidInput"
        );
        assert_eq!(
            err_id("inedges", &[d.clone(), num(1.0), num(2.0)]),
            "RunMat:inedges:UnsupportedOption"
        );
        for bad in [0.0, 2.5] {
            assert_eq!(
                err_id("inedges", &[d.clone(), num(bad)]),
                "RunMat:inedges:InvalidNodeID"
            );
        }
        let empty = Value::Tensor(Tensor::new(vec![], vec![0, 0]).unwrap());
        assert_eq!(
            err_id("inedges", &[d.clone(), empty]),
            "RunMat:inedges:InvalidNodeScalar"
        );
        assert_eq!(
            err_id("inedges", &[d.clone(), chr("a")]),
            "RunMat:inedges:UnknownNodeName"
        );
        let n = named_digraph(&["a", "b"], &["c", "c"]);
        assert_eq!(
            err_id("inedges", &[n.clone(), chr("z")]),
            "RunMat:inedges:UnknownNodeName"
        );
        let (eid, nid) = inedges2(&[n.clone(), chr("a")]);
        assert_eq!(numeric(&eid), (0, 1, vec![]));
        let Value::Cell(cell) = &nid else {
            panic!("expected cell, got {nid:?}");
        };
        assert_eq!((cell.rows, cell.cols), (0, 1));
        // numeric index on a named digraph
        let names = graph_data(&n, "test").unwrap().node_names.unwrap();
        let c = names.iter().position(|x| x == "c").unwrap() + 1;
        let (eid, _) = inedges2(&[n, num(c as f64)]);
        assert_eq!(numeric(&eid), (2, 1, vec![1.0, 2.0]));
        let err = multi(inedges_boxed, &[d, num(3.0)], 3).unwrap_err();
        assert_eq!(identifier(err), "RunMat:inedges:TooManyOutputs");
    }
}
