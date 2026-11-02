//! Clean-room `subgraph` (the graph induced by a set of nodes).
//!
//! Clean-room feature `specs/066-graph-n6/`. Normative behaviour comes only
//! from the approved export `subgraph` 0.2.0 (see that feature's
//! `provenance.md`). The selected nodes keep their `Nodes` table rows; every
//! edge with both end nodes selected is kept with its `Edges` table row,
//! renumbered, and `H.Edges` is sorted by the renumbered end nodes
//! (FR-066-01..10).

use super::construction::edges_table;
use super::paths::{double, graph_input};
use super::*;

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

const SUBGRAPH_INPUTS: [BuiltinParamDescriptor; 2] = [
    param("G", "Graph or digraph object."),
    param(
        "nodeIDs",
        "Logical mask with one element per node, node indices, or node names.",
    ),
];
const SUBGRAPH_OUTPUTS: [BuiltinParamDescriptor; 1] = [param(
    "H",
    "Graph of the class of G induced by the selected nodes.",
)];
const SUBGRAPH_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "H = subgraph(G, nodeIDs)",
    inputs: &SUBGRAPH_INPUTS,
    outputs: &SUBGRAPH_OUTPUTS,
}];

macro_rules! subgraph_error {
    ($code:literal, $id:literal, $when:literal) => {
        BuiltinErrorDescriptor {
            code: concat!("RM.SUBGRAPH.", $code),
            identifier: Some($id),
            when: $when,
            message: concat!("subgraph: ", $when),
        }
    };
}

/// Local choices (U-066-1..3; index 2 is `Internal`, the fallback of
/// `graph_input`'s `query_error`), then the observed identifiers raised as
/// `RunMat:subgraph:<last component>` (graph-family convention, F36-F62);
/// graph inputs use the same identifiers (U-066-4).
const SUBGRAPH_ERRORS: &[BuiltinErrorDescriptor] = &[
    subgraph_error!(
        "INVALID_INPUT",
        "RunMat:subgraph:InvalidInput",
        "An input is malformed."
    ),
    subgraph_error!(
        "UNSUPPORTED_OPTION",
        "RunMat:subgraph:UnsupportedOption",
        "The call uses a form or option that is not yet specified for RunMat."
    ),
    subgraph_error!(
        "INTERNAL",
        "RunMat:subgraph:Internal",
        "RunMat cannot build the result value."
    ),
    subgraph_error!(
        "UNDEFINED_FUNCTION",
        "RunMat:subgraph:UndefinedFunction",
        "The first input is not a graph or digraph object."
    ),
    subgraph_error!(
        "TOO_MANY_OUTPUTS",
        "RunMat:subgraph:TooManyOutputs",
        "More than one output was requested."
    ),
    subgraph_error!("MINRHS", "RunMat:minrhs", "Only G was given."),
    subgraph_error!(
        "INVALID_IND",
        "RunMat:subgraph:InvalidInd",
        "A node is repeated or the logical mask does not have one element per node."
    ),
    subgraph_error!(
        "INVALID_NODE_ID",
        "RunMat:subgraph:InvalidNodeID",
        "A node index is not a node of the graph."
    ),
    subgraph_error!(
        "UNKNOWN_NODE_NAME",
        "RunMat:subgraph:UnknownNodeName",
        "A node name is not a node of the graph."
    ),
];

pub const SUBGRAPH_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &SUBGRAPH_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: SUBGRAPH_ERRORS,
};

const NAME: &str = "subgraph";
const ERRORS: &[BuiltinErrorDescriptor] = SUBGRAPH_ERRORS;

/// Raise the descriptor whose identifier is exactly `identifier`.
fn fail(identifier: &str, detail: impl Into<String>) -> RuntimeError {
    let descriptor = ERRORS
        .iter()
        .find(|d| d.identifier == Some(identifier))
        .or_else(|| ERRORS.get(2))
        .cloned()
        .unwrap_or(ERROR_INTERNAL);
    error_from_descriptor(NAME, descriptor, format!("{NAME}: {}", detail.into()))
}

// ---------------------------------------------------------------------------
// subgraph
// ---------------------------------------------------------------------------

fn invalid_ind(detail: &str) -> RuntimeError {
    fail("RunMat:subgraph:InvalidInd", detail)
}

/// `nodeIDs` as zero-based nodes of `G`, in the order of `H`.
fn parse_node_ids(graph: &GraphData, value: &Value) -> BuiltinResult<Vec<usize>> {
    let n = graph.node_count;
    let selected: Vec<usize> = match value {
        // subgraph.node-order: a mask keeps the original order.
        Value::Bool(_) | Value::LogicalArray(_) => {
            let (flags, is_vector): (Vec<bool>, bool) = match value {
                Value::Bool(b) => (vec![*b], true),
                Value::LogicalArray(array) => {
                    let non_singleton = array.shape.iter().filter(|&&d| d != 1).count();
                    (
                        array.data.iter().map(|&v| v != 0).collect(),
                        non_singleton <= 1,
                    )
                }
                _ => unreachable!(),
            };
            // subgraph.error-repeat (mask length); U-066-6 (vector shape).
            if flags.len() != n || !is_vector {
                return Err(invalid_ind(&format!(
                    "the logical mask must have one element per node ({n})"
                )));
            }
            flags
                .iter()
                .enumerate()
                .filter_map(|(k, &keep)| keep.then_some(k))
                .collect()
        }
        Value::Num(_) | Value::Int(_) | Value::Tensor(_) => {
            let values = super::parsing::numeric_f64_vector(value, NAME)
                .map_err(|_| fail("RunMat:subgraph:InvalidInput", "invalid node indices"))?;
            values
                .iter()
                .map(|&x| {
                    // subgraph.error-range; U-066-6
                    if x.is_finite() && x >= 1.0 && x.fract() == 0.0 && x <= n as f64 {
                        Ok(x as usize - 1)
                    } else {
                        Err(fail(
                            "RunMat:subgraph:InvalidNodeID",
                            "a node index is not a node of the graph",
                        ))
                    }
                })
                .collect::<BuiltinResult<_>>()?
        }
        Value::String(_) | Value::StringArray(_) | Value::CharArray(_) | Value::Cell(_) => {
            let labels = super::parsing::parse_string_vector(value)
                .map_err(|_| fail("RunMat:subgraph:InvalidInput", "invalid node names"))?;
            labels
                .iter()
                .map(|label| {
                    // subgraph.error-unknown-name; U-066-6 (unnamed graphs)
                    graph
                        .node_names
                        .as_ref()
                        .and_then(|names| names.iter().position(|name| name == label))
                        .ok_or_else(|| {
                            fail(
                                "RunMat:subgraph:UnknownNodeName",
                                format!("'{label}' is not a node name of the graph"),
                            )
                        })
                })
                .collect::<BuiltinResult<_>>()?
        }
        _ => {
            return Err(fail(
                "RunMat:subgraph:InvalidInput",
                "nodeIDs must be a logical mask, node indices or node names",
            ))
        }
    };
    // subgraph.error-repeat (repeated node)
    let mut seen = vec![false; n];
    for &node in &selected {
        if std::mem::replace(&mut seen[node], true) {
            return Err(invalid_ind("a node is selected more than once"));
        }
    }
    Ok(selected)
}

/// The variables of a table-valued property of the graph object, in order.
fn table_columns(object: &ObjectInstance, property: &str) -> Option<Vec<(String, Value)>> {
    match object.properties.get(property) {
        Some(Value::Object(table)) => table_variables(table)
            .ok()
            .map(|vars| vars.fields.into_iter().collect()),
        _ => None,
    }
}

fn rebuild_table(columns: Vec<(String, Value)>) -> BuiltinResult<Value> {
    let (names, values): (Vec<String>, Vec<Value>) = columns.into_iter().unzip();
    table_from_columns(names, values).map_err(|err| fail("RunMat:subgraph:Internal", err.message()))
}

fn rows_of(value: &Value, rows: &[usize]) -> BuiltinResult<Value> {
    crate::builtins::table::select_rows(value, rows).map_err(|err| {
        fail(
            "RunMat:subgraph:UnsupportedOption",
            format!("cannot select rows of a table variable: {}", err.message()),
        )
    })
}

#[runtime_builtin(
    name = "subgraph",
    category = "graph",
    summary = "Return the subgraph of a graph or digraph induced by a set of nodes.",
    keywords = "subgraph,graph,digraph,induced,nodes,select",
    accel = "cpu",
    type_resolver(any_type),
    descriptor(crate::builtins::graph::domain::subgraph::SUBGRAPH_DESCRIPTOR),
    builtin_path = "crate::builtins::graph"
)]
pub(super) async fn subgraph_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    let args = gather_values(args).await?;
    // U-066-2
    let count = crate::output_count::current_output_count();
    if let Some(requested) = count.filter(|&c| c > 1) {
        return Err(fail(
            "RunMat:subgraph:TooManyOutputs",
            format!("subgraph returns one output, {requested} were requested"),
        ));
    }
    let Some(first) = args.first() else {
        return Err(fail("RunMat:minrhs", "not enough input arguments"));
    };
    // U-066-1
    let graph = graph_input(NAME, ERRORS, first)?;
    let Value::Object(source) = first else {
        return Err(fail("RunMat:subgraph:Internal", "graph object expected"));
    };
    // subgraph.error-one-arg
    if args.len() < 2 {
        return Err(fail("RunMat:minrhs", "not enough input arguments"));
    }
    // U-066-3
    if args.len() > 2 {
        return Err(fail(
            "RunMat:subgraph:UnsupportedOption",
            "only subgraph(G, nodeIDs) is specified",
        ));
    }
    let selected = parse_node_ids(&graph, &args[1])?;
    let mut new_of_old: Vec<Option<usize>> = vec![None; graph.node_count];
    for (new, &old) in selected.iter().enumerate() {
        new_of_old[old] = Some(new);
    }
    // subgraph.induced-edges, subgraph.multigraph: every edge with both ends
    // selected, renumbered (graphs: smaller end first). subgraph.edge-order:
    // sorted by the renumbered end nodes; ties keep the stored order
    // (U-066-5).
    let mut kept: Vec<(usize, Edge)> = graph
        .edges
        .iter()
        .enumerate()
        .filter_map(|(k, edge)| {
            let (mut s, mut t) = (new_of_old[edge.source]?, new_of_old[edge.target]?);
            if !graph.directed && s > t {
                std::mem::swap(&mut s, &mut t);
            }
            Some((
                k,
                Edge {
                    source: s,
                    target: t,
                    weight: edge.weight,
                },
            ))
        })
        .collect();
    kept.sort_by_key(|(_, edge)| (edge.source, edge.target));
    let edge_rows: Vec<usize> = kept.iter().map(|(k, _)| *k).collect();
    let k = selected.len();
    let induced = GraphData {
        directed: graph.directed,
        node_names: graph
            .node_names
            .as_ref()
            .map(|names| selected.iter().map(|&old| names[old].clone()).collect()),
        edges: kept.into_iter().map(|(_, edge)| edge).collect(),
        node_count: k,
    };

    // Edges table: EndNodes from the renumbered edges (subgraph.names: by
    // name on named graphs); every other variable follows its edge
    // (subgraph.induced-edges); names kept when empty (subgraph.empty).
    let fresh_edges =
        edges_table(&induced).map_err(|err| fail("RunMat:subgraph:Internal", err.message()))?;
    let Value::Object(fresh_edges_object) = &fresh_edges else {
        return Err(fail("RunMat:subgraph:Internal", "edge table expected"));
    };
    let fresh_end_nodes = table_variables(fresh_edges_object)
        .ok()
        .and_then(|vars| vars.fields.get("EndNodes").cloned())
        .ok_or_else(|| fail("RunMat:subgraph:Internal", "edge table without EndNodes"))?;
    let edges_value = match table_columns(source, "Edges") {
        Some(columns) if columns.iter().any(|(name, _)| name == "EndNodes") => {
            let mut out = Vec::with_capacity(columns.len());
            for (name, value) in columns {
                let value = if name == "EndNodes" {
                    fresh_end_nodes.clone()
                } else {
                    rows_of(&value, &edge_rows)?
                };
                out.push((name, value));
            }
            rebuild_table(out)?
        }
        _ => fresh_edges.clone(),
    };

    // Nodes table: every variable follows its node (subgraph.node-table);
    // RunMat's synthetic `Index` column of unnamed graphs is renumbered 1..k
    // (U-066-7).
    let nodes_value = match table_columns(source, "Nodes") {
        Some(columns) if !columns.is_empty() => {
            let mut out = Vec::with_capacity(columns.len());
            for (name, value) in columns {
                let value = if name == "Index" && graph.node_names.is_none() {
                    double(NAME, ERRORS, (1..=k).map(|i| i as f64).collect(), k, 1)?
                } else {
                    rows_of(&value, &selected)?
                };
                out.push((name, value));
            }
            rebuild_table(out)?
        }
        _ => super::construction::nodes_table(&induced)
            .map_err(|err| fail("RunMat:subgraph:Internal", err.message()))?,
    };

    // subgraph.output-class: H is a copy of G with the new node count and
    // tables.
    let mut object = source.clone();
    object
        .properties
        .insert(NUM_NODES_PROPERTY.to_string(), Value::Num(k as f64));
    object.properties.insert("Edges".to_string(), edges_value);
    object.properties.insert("Nodes".to_string(), nodes_value);
    let h = Value::Object(object);
    Ok(match count {
        None => h,
        Some(0) => Value::OutputList(Vec::new()),
        Some(_) => Value::OutputList(vec![h]),
    })
}

#[cfg(test)]
mod graph_n6_subgraph_tests {
    use super::super::construction::graph_data;
    use super::super::parsing::parse_string_vector;
    use super::*;
    use futures::executor::block_on;
    use runmat_value::{CellArray, CharArray, LogicalArray};

    // ---- helpers ----------------------------------------------------------

    fn row(values: &[f64]) -> Value {
        Value::Tensor(Tensor::new(values.to_vec(), vec![1, values.len()]).unwrap())
    }

    fn col(values: &[f64]) -> Value {
        Value::Tensor(Tensor::new(values.to_vec(), vec![values.len(), 1]).unwrap())
    }

    fn empty() -> Value {
        Value::Tensor(Tensor::new(Vec::new(), vec![0, 0]).unwrap())
    }

    fn mask(values: &[bool], rows: usize, cols: usize) -> Value {
        Value::LogicalArray(
            LogicalArray::new(values.iter().map(|&b| b as u8).collect(), vec![rows, cols]).unwrap(),
        )
    }

    fn chr(text: &str) -> Value {
        Value::CharArray(CharArray::new_row(text))
    }

    fn cellstr(values: &[&str], rows: usize, cols: usize) -> Value {
        Value::Cell(CellArray::new(values.iter().map(|s| chr(s)).collect(), rows, cols).unwrap())
    }

    fn call(name: &str, args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async(name, args))
    }

    fn digraph(s: &[f64], t: &[f64]) -> Value {
        call("digraph", &[row(s), row(t)]).expect("digraph")
    }

    fn graph(s: &[f64], t: &[f64]) -> Value {
        call("graph", &[row(s), row(t)]).expect("graph")
    }

    fn identifier(err: RuntimeError) -> String {
        err.identifier().unwrap_or("<none>").to_string()
    }

    fn err_id(args: &[Value]) -> String {
        identifier(call("subgraph", args).expect_err("expected an error"))
    }

    fn sub(g: &Value, ids: Value) -> Value {
        call("subgraph", &[g.clone(), ids]).expect("subgraph")
    }

    fn boxed(args: Vec<Value>) -> futures::future::LocalBoxFuture<'static, BuiltinResult<Value>> {
        Box::pin(subgraph_builtin(args))
    }

    fn multi(args: &[Value], count: usize) -> BuiltinResult<Vec<Value>> {
        let _guard = crate::output_count::push_output_count(Some(count));
        match block_on(boxed(args.to_vec()))? {
            Value::OutputList(values) => Ok(values),
            other => panic!("expected output list, got {other:?}"),
        }
    }

    fn numeric(value: &Value) -> (usize, usize, Vec<f64>) {
        match value {
            Value::Num(n) => (1, 1, vec![*n]),
            Value::Tensor(t) => (t.rows, t.cols, tensor::tensor_values_f64(t)),
            other => panic!("expected double, got {other:?}"),
        }
    }

    fn object(g: &Value) -> &ObjectInstance {
        match g {
            Value::Object(o) => o,
            other => panic!("expected object, got {other:?}"),
        }
    }

    fn class(g: &Value) -> String {
        object(g).class_name.to_string()
    }

    fn numnodes(g: &Value) -> f64 {
        numeric(&call("numnodes", std::slice::from_ref(g)).unwrap()).2[0]
    }

    /// Variables of `G.Nodes` or `G.Edges`, in order.
    fn vars(g: &Value, property: &str) -> Vec<(String, Value)> {
        let Some(Value::Object(table)) = object(g).properties.get(property) else {
            panic!("missing {property} table");
        };
        table_variables(table).unwrap().fields.into_iter().collect()
    }

    fn var_names(g: &Value, property: &str) -> Vec<String> {
        vars(g, property).into_iter().map(|(n, _)| n).collect()
    }

    fn var(g: &Value, property: &str, name: &str) -> Value {
        vars(g, property)
            .into_iter()
            .find(|(n, _)| n == name)
            .unwrap_or_else(|| panic!("no variable {name} in {property}"))
            .1
    }

    /// Stored end nodes (1-based), as read back by RunMat's graph subsystem.
    fn ends(g: &Value) -> Vec<(usize, usize)> {
        graph_data(g, "test")
            .unwrap()
            .edges
            .iter()
            .map(|e| (e.source + 1, e.target + 1))
            .collect()
    }

    /// `EndNodes` of an unnamed graph as (rows, cols, column-major data).
    fn end_nodes(g: &Value) -> (usize, usize, Vec<f64>) {
        numeric(&var(g, "Edges", "EndNodes"))
    }

    fn texts(value: &Value) -> Vec<String> {
        parse_string_vector(value).unwrap()
    }

    /// A cell column: (rows, cols, texts).
    fn cell_column(value: &Value) -> (usize, usize, Vec<String>) {
        let Value::Cell(cell) = value else {
            panic!("expected a cell, got {value:?}");
        };
        (cell.rows, cell.cols, texts(value))
    }

    fn with_table(g: &Value, property: &str, names: &[&str], columns: Vec<Value>) -> Value {
        let mut g = g.clone();
        let Value::Object(object) = &mut g else {
            panic!("expected graph object");
        };
        let table =
            table_from_columns(names.iter().map(|s| s.to_string()).collect(), columns).unwrap();
        object.properties.insert(property.to_string(), table);
        g
    }

    /// Case `source-graph`: the 4-cycle with node variables Met and Atom and
    /// edge variable rxns. RunMat's constructor takes no node or edge table
    /// (D-059-3/D-062-2), so both tables are rebuilt with those columns.
    fn source_graph() -> Value {
        let g = digraph(&[1.0, 2.0, 3.0, 4.0], &[2.0, 3.0, 4.0, 1.0]);
        let end_nodes = Value::Tensor(
            Tensor::new(vec![1.0, 2.0, 3.0, 4.0, 2.0, 3.0, 4.0, 1.0], vec![4, 2]).unwrap(),
        );
        let g = with_table(
            &g,
            "Edges",
            &["EndNodes", "rxns"],
            vec![end_nodes, cellstr(&["r1", "r2", "r3", "r4"], 4, 1)],
        );
        with_table(
            &g,
            "Nodes",
            &["Met", "Atom"],
            vec![
                cellstr(&["a", "b", "c", "d"], 4, 1),
                col(&[1.0, 2.0, 3.0, 4.0]),
            ],
        )
    }

    /// Assert H's node table (Met, Atom) and edge table (EndNodes, rxns).
    fn assert_met_graph(h: &Value, met: &[&str], atom: &[f64], ends_cm: &[f64], rxns: &[&str]) {
        assert_eq!(class(h), "digraph");
        assert_eq!(numnodes(h), met.len() as f64);
        assert_eq!(var_names(h, "Nodes"), vec!["Met", "Atom"]);
        let (r, c, m) = cell_column(&var(h, "Nodes", "Met"));
        assert_eq!((r, c), (met.len(), 1));
        assert_eq!(m, met);
        assert_eq!(
            numeric(&var(h, "Nodes", "Atom")),
            (atom.len(), 1, atom.to_vec())
        );
        assert_eq!(var_names(h, "Edges"), vec!["EndNodes", "rxns"]);
        assert_eq!(end_nodes(h), (rxns.len(), 2, ends_cm.to_vec()));
        let (r, c, x) = cell_column(&var(h, "Edges", "rxns"));
        assert_eq!((r, c), (rxns.len(), 1));
        assert_eq!(x, rxns);
    }

    // ---- observed (normative) ---------------------------------------------

    /// Set-up check for case `source-graph`.
    #[test]
    fn observed_subgraph_source_graph_setup() {
        let g = source_graph();
        assert_met_graph(
            &g,
            &["a", "b", "c", "d"],
            &[1.0, 2.0, 3.0, 4.0],
            &[1.0, 2.0, 3.0, 4.0, 2.0, 3.0, 4.0, 1.0],
            &["r1", "r2", "r3", "r4"],
        );
    }

    /// subgraph.signature-primary, node-order, node-table, induced-edges
    /// (case `logical-mask`, COBRA form).
    #[test]
    fn observed_subgraph_logical_mask() {
        let h = sub(&source_graph(), mask(&[true, true, true, false], 1, 4));
        assert_met_graph(
            &h,
            &["a", "b", "c"],
            &[1.0, 2.0, 3.0],
            &[1.0, 2.0, 2.0, 3.0],
            &["r1", "r2"],
        );
        assert_eq!(ends(&h), vec![(1, 2), (2, 3)]);
    }

    /// subgraph.node-order, induced-edges, edge-order (case `logical-column`).
    #[test]
    fn observed_subgraph_logical_column() {
        let h = sub(&source_graph(), mask(&[true, false, true, true], 4, 1));
        assert_met_graph(
            &h,
            &["a", "c", "d"],
            &[1.0, 3.0, 4.0],
            &[2.0, 3.0, 3.0, 1.0],
            &["r3", "r4"],
        );
    }

    /// subgraph.node-order, induced-edges (case `index-column`, COBRA form).
    #[test]
    fn observed_subgraph_index_column() {
        let h = sub(&source_graph(), col(&[2.0, 3.0]));
        assert_met_graph(&h, &["b", "c"], &[2.0, 3.0], &[1.0, 2.0], &["r2"]);
    }

    /// subgraph.node-order, node-table, induced-edges, edge-order (case
    /// `index-unsorted`).
    #[test]
    fn observed_subgraph_index_unsorted() {
        let h = sub(&source_graph(), row(&[3.0, 1.0, 2.0]));
        assert_met_graph(
            &h,
            &["c", "a", "b"],
            &[3.0, 1.0, 2.0],
            &[2.0, 3.0, 3.0, 1.0],
            &["r1", "r2"],
        );
    }

    /// subgraph.edge-order (case `edge-order`): sorted by renumbered end
    /// nodes, not G's edge order.
    #[test]
    fn observed_subgraph_edge_order() {
        let h = sub(&source_graph(), row(&[4.0, 1.0, 2.0]));
        assert_met_graph(
            &h,
            &["d", "a", "b"],
            &[4.0, 1.0, 2.0],
            &[1.0, 2.0, 2.0, 3.0],
            &["r4", "r1"],
        );
        assert_eq!(ends(&h), vec![(1, 2), (2, 3)]);
    }

    /// subgraph.empty (cases `all-false`, `empty-index`).
    #[test]
    fn observed_subgraph_empty() {
        for ids in [mask(&[false; 4], 1, 4), empty()] {
            let h = sub(&source_graph(), ids);
            assert_met_graph(&h, &[], &[], &[], &[]);
            assert!(ends(&h).is_empty());
        }
    }

    /// subgraph.names, node-order (case `names`). Name and EndNodes are text
    /// (RunMat stores them as string arrays: D-066-1).
    #[test]
    fn observed_subgraph_names() {
        let g = call(
            "digraph",
            &[
                cellstr(&["a", "b", "c"], 1, 3),
                cellstr(&["b", "c", "a"], 1, 3),
            ],
        )
        .unwrap();
        let h = sub(&g, cellstr(&["c", "a"], 1, 2));
        assert_eq!(numnodes(&h), 2.0);
        assert_eq!(var_names(&h, "Nodes"), vec!["Name"]);
        assert_eq!(texts(&var(&h, "Nodes", "Name")), vec!["c", "a"]);
        assert_eq!(var_names(&h, "Edges"), vec!["EndNodes"]);
        let end = var(&h, "Edges", "EndNodes");
        assert_eq!(texts(&end), vec!["c", "a"]);
        assert_eq!(ends(&h), vec![(1, 2)]);
    }

    /// D-066-1: the observed EndNodes of a named subgraph is a cell array.
    #[test]
    #[ignore = "D-066-1 (as D-062-2): RunMat stores named EndNodes as a string array"]
    fn deviation_subgraph_names_end_nodes_cell() {
        let g = call(
            "digraph",
            &[
                cellstr(&["a", "b", "c"], 1, 3),
                cellstr(&["b", "c", "a"], 1, 3),
            ],
        )
        .unwrap();
        let h = sub(&g, cellstr(&["c", "a"], 1, 2));
        assert!(matches!(var(&h, "Edges", "EndNodes"), Value::Cell(_)));
    }

    /// subgraph.induced-edges (case `weighted`).
    #[test]
    fn observed_subgraph_weighted() {
        let g = call(
            "digraph",
            &[
                row(&[1.0, 2.0, 1.0]),
                row(&[2.0, 3.0, 3.0]),
                row(&[5.0, 6.0, 7.0]),
            ],
        )
        .unwrap();
        let h = sub(&g, row(&[1.0, 2.0]));
        assert_eq!(class(&h), "digraph");
        assert_eq!(numnodes(&h), 2.0);
        assert_eq!(var_names(&h, "Edges"), vec!["EndNodes", "Weight"]);
        assert_eq!(end_nodes(&h), (1, 2, vec![1.0, 2.0]));
        assert_eq!(numeric(&var(&h, "Edges", "Weight")), (1, 1, vec![5.0]));
    }

    /// subgraph.multigraph, edge-order (case `multigraph`).
    #[test]
    fn observed_subgraph_multigraph() {
        let h = sub(
            &digraph(&[1.0, 1.0, 1.0, 2.0], &[2.0, 2.0, 1.0, 3.0]),
            row(&[1.0, 2.0]),
        );
        assert_eq!(numnodes(&h), 2.0);
        assert_eq!(end_nodes(&h), (3, 2, vec![1.0, 1.0, 1.0, 1.0, 2.0, 2.0]));
    }

    /// subgraph.output-class (cases `undirected`, `class`).
    #[test]
    fn observed_subgraph_class_and_undirected() {
        let h = sub(&graph(&[1.0, 2.0, 3.0], &[2.0, 3.0, 1.0]), row(&[2.0, 3.0]));
        assert_eq!(class(&h), "graph");
        assert_eq!(numnodes(&h), 2.0);
        assert_eq!(end_nodes(&h), (1, 2, vec![1.0, 2.0]));
        let h = sub(&digraph(&[1.0, 2.0], &[2.0, 3.0]), row(&[1.0, 2.0]));
        assert_eq!(class(&h), "digraph");
    }

    /// subgraph.isolated (case `isolated-node`).
    #[test]
    fn observed_subgraph_isolated_node() {
        let h = sub(&digraph(&[1.0, 2.0], &[2.0, 3.0]), row(&[1.0, 3.0]));
        assert_eq!(numnodes(&h), 2.0);
        assert_eq!(end_nodes(&h), (0, 2, vec![]));
    }

    /// subgraph.error-repeat, error-range, error-unknown-name, error-one-arg
    /// (cases `err-repeat`, `err-mask-length`, `err-mask-longer`,
    /// `err-range`, `err-unknown-name`, `err-one-arg`).
    #[test]
    fn rejects_subgraph_errors() {
        let g = digraph(&[1.0, 2.0], &[2.0, 3.0]);
        assert_eq!(
            err_id(&[g.clone(), row(&[1.0, 1.0, 2.0])]),
            "RunMat:subgraph:InvalidInd"
        );
        assert_eq!(
            err_id(&[g.clone(), mask(&[true, false], 1, 2)]),
            "RunMat:subgraph:InvalidInd"
        );
        assert_eq!(
            err_id(&[g.clone(), mask(&[true, true, false, false], 1, 4)]),
            "RunMat:subgraph:InvalidInd"
        );
        assert_eq!(
            err_id(&[g.clone(), row(&[1.0, 4.0])]),
            "RunMat:subgraph:InvalidNodeID"
        );
        let named = call("digraph", &[cellstr(&["a"], 1, 1), cellstr(&["b"], 1, 1)]).unwrap();
        assert_eq!(
            err_id(&[named, cellstr(&["z"], 1, 1)]),
            "RunMat:subgraph:UnknownNodeName"
        );
        assert_eq!(err_id(&[g]), "RunMat:minrhs");
    }

    // ---- documented choices (non-normative) -------------------------------

    /// U-066-5 (subgraph.q-tie-order): parallel edges keep their stored
    /// relative order (stable sort); weights follow their edges.
    #[test]
    fn unresolved_choice_subgraph_tie_order() {
        let g = call(
            "digraph",
            &[
                row(&[2.0, 1.0, 2.0, 3.0]),
                row(&[1.0, 2.0, 1.0, 1.0]),
                row(&[5.0, 6.0, 7.0, 8.0]),
            ],
        )
        .unwrap();
        let h = sub(&g, row(&[1.0, 2.0]));
        assert_eq!(ends(&h), vec![(1, 2), (2, 1), (2, 1)]);
        assert_eq!(numeric(&var(&h, "Edges", "Weight")).2, vec![6.0, 5.0, 7.0]);
        let h = sub(&g, row(&[2.0, 1.0]));
        assert_eq!(ends(&h), vec![(1, 2), (1, 2), (2, 1)]);
        assert_eq!(numeric(&var(&h, "Edges", "Weight")).2, vec![5.0, 7.0, 6.0]);
    }

    /// U-066-1..3: non-graph input, arity, output count, malformed nodeIDs.
    #[test]
    fn unresolved_choice_subgraph_inputs() {
        let g = digraph(&[1.0, 2.0], &[2.0, 3.0]);
        assert_eq!(
            err_id(&[row(&[1.0]), row(&[1.0])]),
            "RunMat:subgraph:UndefinedFunction"
        );
        assert_eq!(
            identifier(block_on(boxed(Vec::new())).unwrap_err()),
            "RunMat:minrhs"
        );
        assert_eq!(
            err_id(&[g.clone(), row(&[1.0]), chr("x")]),
            "RunMat:subgraph:UnsupportedOption"
        );
        assert_eq!(
            err_id(&[g.clone(), Value::Struct(runmat_value::StructValue::new())]),
            "RunMat:subgraph:InvalidInput"
        );
        let err = multi(&[g.clone(), row(&[1.0])], 2).unwrap_err();
        assert_eq!(identifier(err), "RunMat:subgraph:TooManyOutputs");
        let one = multi(&[g.clone(), row(&[1.0])], 1).unwrap();
        assert_eq!(one.len(), 1);
        assert_eq!(class(&one[0]), "digraph");
        assert!(multi(&[g, row(&[1.0])], 0).unwrap().is_empty());
    }

    /// U-066-4 (subgraph.q-undirected-ids) and U-066-6: node validation.
    #[test]
    fn unresolved_choice_subgraph_node_validation() {
        let g = digraph(&[1.0, 2.0], &[2.0, 3.0]);
        for bad in [0.0, -1.0, 1.5, f64::NAN, f64::INFINITY] {
            assert_eq!(
                err_id(&[g.clone(), row(&[1.0, bad])]),
                "RunMat:subgraph:InvalidNodeID"
            );
        }
        // Range is checked before repeats.
        assert_eq!(
            err_id(&[g.clone(), row(&[1.0, 1.0, 4.0])]),
            "RunMat:subgraph:InvalidNodeID"
        );
        // A logical mask must be a vector.
        let four = digraph(&[1.0, 3.0], &[2.0, 4.0]);
        assert_eq!(
            err_id(&[four.clone(), mask(&[true, true, false, true], 2, 2)]),
            "RunMat:subgraph:InvalidInd"
        );
        // Names on an unnamed graph; repeated names.
        assert_eq!(
            err_id(&[g.clone(), chr("a")]),
            "RunMat:subgraph:UnknownNodeName"
        );
        let named = call(
            "digraph",
            &[cellstr(&["a", "b"], 1, 2), cellstr(&["b", "c"], 1, 2)],
        )
        .unwrap();
        assert_eq!(
            err_id(&[named.clone(), cellstr(&["a", "a"], 1, 2)]),
            "RunMat:subgraph:InvalidInd"
        );
        // Indices, a char name and a string array on a named graph.
        let h = sub(&named, col(&[3.0, 1.0]));
        assert_eq!(texts(&var(&h, "Nodes", "Name")), vec!["c", "a"]);
        let h = sub(&named, chr("b"));
        assert_eq!(texts(&var(&h, "Nodes", "Name")), vec!["b"]);
        let strings =
            Value::StringArray(StringArray::new(vec!["b".into(), "c".into()], vec![1, 2]).unwrap());
        let h = sub(&named, strings);
        assert_eq!(ends(&h), vec![(1, 2)]);
        // graph inputs use the same identifiers.
        let u = graph(&[1.0, 2.0], &[2.0, 3.0]);
        assert_eq!(
            err_id(&[u.clone(), row(&[5.0])]),
            "RunMat:subgraph:InvalidNodeID"
        );
        assert_eq!(
            err_id(&[u.clone(), chr("q")]),
            "RunMat:subgraph:UnknownNodeName"
        );
        assert_eq!(err_id(&[u, row(&[2.0, 2.0])]), "RunMat:subgraph:InvalidInd");
    }

    /// U-066-7: the synthetic Index column is renumbered; NumNodes is the
    /// new node count; H works with the other graph builtins; a scalar
    /// logical mask on a one-node graph.
    #[test]
    fn unresolved_choice_subgraph_index_and_result() {
        let g = digraph(&[1.0, 2.0, 3.0], &[2.0, 3.0, 1.0]);
        let h = sub(&g, row(&[3.0, 1.0]));
        assert_eq!(var_names(&h, "Nodes"), vec!["Index"]);
        assert_eq!(numeric(&var(&h, "Nodes", "Index")).2, vec![1.0, 2.0]);
        assert_eq!(
            object(&h).properties.get("NumNodes"),
            Some(&Value::Num(2.0))
        );
        assert_eq!(ends(&h), vec![(1, 2)]);
        let e = numeric(&call("numedges", std::slice::from_ref(&h)).unwrap()).2;
        assert_eq!(e, vec![1.0]);
        let one = digraph(&[1.0], &[1.0]);
        let h = sub(&one, Value::Bool(true));
        assert_eq!(ends(&h), vec![(1, 1)]);
        let h = sub(&one, Value::Bool(false));
        assert_eq!(numnodes(&h), 0.0);
    }
}
