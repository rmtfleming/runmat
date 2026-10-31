//! Clean-room `reordernodes` (permute the nodes of a graph or digraph).
//!
//! Clean-room feature `specs/062-graph-n5/`. Normative behaviour comes only
//! from the approved export `reordernodes` 0.2.0 (see that feature's
//! `provenance.md`). Every `Nodes` table variable is permuted with the
//! nodes; every `Edges` table variable is permuted with the edges, and
//! `EndNodes` is renumbered (FR-062-08..12).

use super::construction::edges_table;
use super::parsing::{numeric_f64_vector, parse_string_vector};
use super::paths::{check_output_count, double, graph_input, outputs, query_error};
use super::*;
use crate::builtins::table::select_rows;

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

const REORDERNODES_INPUTS: [BuiltinParamDescriptor; 2] = [
    param("G", "Graph or digraph object."),
    param(
        "order",
        "Permutation of the node indices, or all node names in the new order.",
    ),
];
const REORDERNODES_OUTPUT_H: BuiltinParamDescriptor =
    param("H", "Graph of the class of G with its nodes reordered.");
const REORDERNODES_OUTPUTS: [BuiltinParamDescriptor; 1] = [REORDERNODES_OUTPUT_H];
const REORDERNODES_OUTPUTS_TWO: [BuiltinParamDescriptor; 2] = [
    REORDERNODES_OUTPUT_H,
    param("idx", "Column: row k of H.Edges is row idx(k) of G.Edges."),
];
const REORDERNODES_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "H = reordernodes(G, order)",
        inputs: &REORDERNODES_INPUTS,
        outputs: &REORDERNODES_OUTPUTS,
    },
    BuiltinSignatureDescriptor {
        label: "[H, idx] = reordernodes(G, order)",
        inputs: &REORDERNODES_INPUTS,
        outputs: &REORDERNODES_OUTPUTS_TWO,
    },
];

macro_rules! reorder_error {
    ($code:literal, $id:literal, $when:literal) => {
        BuiltinErrorDescriptor {
            code: concat!("RM.REORDERNODES.", $code),
            identifier: Some(concat!("RunMat:reordernodes:", $id)),
            when: $when,
            message: concat!("reordernodes: ", $when),
        }
    };
}

/// Common errors (U-062-6; index 2 is `Internal`, the fallback of
/// `query_error`) followed by the observed ones and U-062-7.
const REORDERNODES_ERRORS: &[BuiltinErrorDescriptor] = &[
    reorder_error!(
        "INVALID_INPUT",
        "InvalidInput",
        "A required input is missing or an input is malformed."
    ),
    reorder_error!(
        "UNSUPPORTED_OPTION",
        "UnsupportedOption",
        "The call uses a form or option that is not yet specified for RunMat."
    ),
    reorder_error!(
        "INTERNAL",
        "Internal",
        "RunMat cannot build the result value."
    ),
    reorder_error!(
        "UNDEFINED_FUNCTION",
        "UndefinedFunction",
        "The first input is not a graph or digraph object."
    ),
    reorder_error!(
        "TOO_MANY_OUTPUTS",
        "TooManyOutputs",
        "More than two outputs were requested."
    ),
    reorder_error!(
        "INVALID_ORDER",
        "InvalidOrder",
        "The order repeats or omits nodes."
    ),
    reorder_error!(
        "INVALID_NODE_ID",
        "InvalidNodeID",
        "The order contains an index that is not a node of the graph."
    ),
    reorder_error!(
        "UNKNOWN_NODE_NAME",
        "UnknownNodeName",
        "The order contains a name that is not a node of the graph."
    ),
];

pub const REORDERNODES_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &REORDERNODES_SIGNATURES,
    output_mode: BuiltinOutputMode::ByRequestedOutputCount,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: REORDERNODES_ERRORS,
};

const NAME: &str = "reordernodes";
const ERRORS: &[BuiltinErrorDescriptor] = REORDERNODES_ERRORS;

fn fail(id: &str, detail: impl Into<String>) -> RuntimeError {
    query_error(NAME, ERRORS, id, detail)
}

// ---------------------------------------------------------------------------
// reordernodes
// ---------------------------------------------------------------------------

/// `order` as zero-based old node indices, new position k holding old node
/// `order[k]`. Each entry is validated first (InvalidNodeID /
/// UnknownNodeName), then the whole list (InvalidOrder) (U-062-7).
fn parse_order(graph: &GraphData, value: &Value) -> BuiltinResult<Vec<usize>> {
    let n = graph.node_count;
    let order: Vec<usize> = match value {
        Value::String(_) | Value::StringArray(_) | Value::CharArray(_) | Value::Cell(_) => {
            // reordernodes.names
            let labels = parse_string_vector(value)
                .map_err(|_| fail("InvalidInput", "order must be indices or node names"))?;
            let names = graph.node_names.as_ref();
            labels
                .iter()
                .map(|label| {
                    names
                        .and_then(|names| names.iter().position(|name| name == label))
                        .ok_or_else(|| {
                            fail(
                                "UnknownNodeName",
                                format!("'{label}' is not a node name of the graph"),
                            )
                        })
                })
                .collect::<BuiltinResult<_>>()?
        }
        Value::Num(_)
        | Value::Int(_)
        | Value::Bool(_)
        | Value::Tensor(_)
        | Value::LogicalArray(_) => {
            let values = numeric_f64_vector(value, NAME)
                .map_err(|_| fail("InvalidInput", "order must be indices or node names"))?;
            values
                .iter()
                .map(|&x| {
                    // reordernodes.error-range
                    if x.is_finite() && x >= 1.0 && x.fract() == 0.0 && x <= n as f64 {
                        Ok(x as usize - 1)
                    } else {
                        Err(fail(
                            "InvalidNodeID",
                            "order contains an index that is not a node of the graph",
                        ))
                    }
                })
                .collect::<BuiltinResult<_>>()?
        }
        _ => return Err(fail("InvalidInput", "order must be indices or node names")),
    };
    // reordernodes.error-repeat (repeats and omissions)
    let mut seen = vec![false; n];
    for &node in &order {
        if std::mem::replace(&mut seen[node], true) {
            return Err(fail("InvalidOrder", "order repeats a node"));
        }
    }
    if order.len() != n {
        return Err(fail(
            "InvalidOrder",
            format!("order must list all {n} nodes exactly once"),
        ));
    }
    Ok(order)
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
    table_from_columns(names, values).map_err(|err| fail("Internal", err.message()))
}

fn rows_of(value: &Value, rows: &[usize]) -> BuiltinResult<Value> {
    select_rows(value, rows).map_err(|err| {
        fail(
            "UnsupportedOption",
            format!("cannot reorder a table variable: {}", err.message()),
        )
    })
}

#[runtime_builtin(
    name = "reordernodes",
    category = "graph",
    summary = "Reorder the nodes of a graph or digraph, carrying node and edge data along.",
    keywords = "reordernodes,graph,digraph,permute,nodes,order",
    accel = "cpu",
    type_resolver(any_type),
    descriptor(crate::builtins::graph::domain::reordernodes::REORDERNODES_DESCRIPTOR),
    builtin_path = "crate::builtins::graph"
)]
pub(super) async fn reordernodes_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    let args = gather_values(args).await?;
    let count = check_output_count(NAME, ERRORS)?;
    let Some(first) = args.first() else {
        return Err(fail("InvalidInput", "expected reordernodes(G, order)"));
    };
    // U-062-6
    let graph = graph_input(NAME, ERRORS, first)?;
    let Value::Object(source) = first else {
        return Err(fail("Internal", "graph object expected"));
    };
    if args.len() < 2 {
        return Err(fail("InvalidInput", "expected reordernodes(G, order)"));
    }
    if args.len() > 2 {
        return Err(fail(
            "UnsupportedOption",
            "only reordernodes(G, order) is specified",
        ));
    }
    let order = parse_order(&graph, &args[1])?;
    // reordernodes.nodes: node k of H is node order(k) of G.
    let mut new_of_old = vec![0usize; graph.node_count];
    for (new, &old) in order.iter().enumerate() {
        new_of_old[old] = new;
    }
    // reordernodes.edges: renumber, smaller end first for graphs
    // (reordernodes.undirected), ascending end nodes; ties keep the stored
    // order (U-062-8).
    let mut renumbered: Vec<(usize, Edge)> = graph
        .edges
        .iter()
        .enumerate()
        .map(|(k, edge)| {
            let (mut s, mut t) = (new_of_old[edge.source], new_of_old[edge.target]);
            if !graph.directed && s > t {
                std::mem::swap(&mut s, &mut t);
            }
            (
                k,
                Edge {
                    source: s,
                    target: t,
                    weight: edge.weight,
                },
            )
        })
        .collect();
    renumbered.sort_by_key(|(_, edge)| (edge.source, edge.target));
    let idx: Vec<usize> = renumbered.iter().map(|(k, _)| *k).collect();
    let reordered = GraphData {
        directed: graph.directed,
        node_names: graph
            .node_names
            .as_ref()
            .map(|names| order.iter().map(|&old| names[old].clone()).collect()),
        edges: renumbered.into_iter().map(|(_, edge)| edge).collect(),
        node_count: graph.node_count,
    };

    // Edges table: EndNodes from the renumbered edges; every other variable
    // (Weight included, reordernodes.weights) follows its edge.
    let fresh_edges = edges_table(&reordered).map_err(|err| fail("Internal", err.message()))?;
    let Value::Object(fresh_edges_object) = &fresh_edges else {
        return Err(fail("Internal", "edge table expected"));
    };
    let fresh_end_nodes = table_variables(fresh_edges_object)
        .ok()
        .and_then(|vars| vars.fields.get("EndNodes").cloned())
        .ok_or_else(|| fail("Internal", "edge table without EndNodes"))?;
    let edges_value = match table_columns(source, "Edges") {
        Some(columns) if columns.iter().any(|(name, _)| name == "EndNodes") => {
            let mut out = Vec::with_capacity(columns.len());
            for (name, value) in columns {
                let value = if name == "EndNodes" {
                    fresh_end_nodes.clone()
                } else {
                    rows_of(&value, &idx)?
                };
                out.push((name, value));
            }
            rebuild_table(out)?
        }
        _ => fresh_edges.clone(),
    };

    // Nodes table: every variable follows its node; RunMat's synthetic
    // `Index` column of unnamed graphs is renumbered 1..n (U-062-9).
    let nodes_value = match table_columns(source, "Nodes") {
        Some(columns) if !columns.is_empty() => {
            let mut out = Vec::with_capacity(columns.len());
            for (name, value) in columns {
                let value = if name == "Index" && graph.node_names.is_none() {
                    double(
                        NAME,
                        ERRORS,
                        (1..=graph.node_count).map(|i| i as f64).collect(),
                        graph.node_count,
                        1,
                    )?
                } else {
                    rows_of(&value, &order)?
                };
                out.push((name, value));
            }
            rebuild_table(out)?
        }
        _ => super::construction::nodes_table(&reordered)
            .map_err(|err| fail("Internal", err.message()))?,
    };

    // reordernodes.identity (class): H is a copy of G with new tables.
    let mut object = source.clone();
    object.properties.insert("Edges".to_string(), edges_value);
    object.properties.insert("Nodes".to_string(), nodes_value);
    let h = Value::Object(object);
    // reordernodes.edges: idx as a column (0x1 without edges).
    let k = idx.len();
    outputs(count, h, || {
        double(
            NAME,
            ERRORS,
            idx.iter().map(|&e| (e + 1) as f64).collect(),
            k,
            1,
        )
    })
}

#[cfg(test)]
mod graph_n5_reordernodes_tests {
    use super::super::construction::graph_data;
    use super::*;
    use futures::executor::block_on;
    use runmat_value::{CellArray, CharArray};

    // ---- helpers ----------------------------------------------------------

    fn row(values: &[f64]) -> Value {
        Value::Tensor(Tensor::new(values.to_vec(), vec![1, values.len()]).unwrap())
    }

    fn col(values: &[f64]) -> Value {
        Value::Tensor(Tensor::new(values.to_vec(), vec![values.len(), 1]).unwrap())
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

    fn graph(s: &[f64], t: &[f64]) -> Value {
        call("graph", &[row(s), row(t)]).expect("graph")
    }

    fn digraph(s: &[f64], t: &[f64]) -> Value {
        call("digraph", &[row(s), row(t)]).expect("digraph")
    }

    fn identifier(err: RuntimeError) -> String {
        err.identifier().unwrap_or("<none>").to_string()
    }

    fn err_id(args: &[Value]) -> String {
        identifier(call("reordernodes", args).expect_err("expected an error"))
    }

    fn boxed(args: Vec<Value>) -> futures::future::LocalBoxFuture<'static, BuiltinResult<Value>> {
        Box::pin(reordernodes_builtin(args))
    }

    fn multi(args: &[Value], count: usize) -> BuiltinResult<Vec<Value>> {
        let _guard = crate::output_count::push_output_count(Some(count));
        match block_on(boxed(args.to_vec()))? {
            Value::OutputList(values) => Ok(values),
            other => panic!("expected output list, got {other:?}"),
        }
    }

    /// [H, idx] = reordernodes(args...).
    fn reorder2(args: &[Value]) -> (Value, Vec<f64>) {
        let out = multi(args, 2).expect("reordernodes");
        assert_eq!(out.len(), 2);
        let (rows, cols, idx) = numeric(&out[1]);
        assert_eq!((rows, cols), (idx.len(), 1), "idx must be a column");
        (out[0].clone(), idx)
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

    /// Variables of `G.Nodes` or `G.Edges`, in order.
    fn vars(g: &Value, property: &str) -> Vec<(String, Value)> {
        let Some(Value::Object(table)) = object(g).properties.get(property) else {
            panic!("missing {property} table");
        };
        table_variables(table).unwrap().fields.into_iter().collect()
    }

    fn var(g: &Value, property: &str, name: &str) -> Value {
        vars(g, property)
            .into_iter()
            .find(|(n, _)| n == name)
            .unwrap_or_else(|| panic!("no variable {name} in {property}"))
            .1
    }

    /// Stored end nodes (1-based).
    fn ends(g: &Value) -> Vec<(usize, usize)> {
        graph_data(g, "test")
            .unwrap()
            .edges
            .iter()
            .map(|e| (e.source + 1, e.target + 1))
            .collect()
    }

    /// Text values of a cellstr or string column.
    fn texts(value: &Value) -> Vec<String> {
        parse_string_vector(value).unwrap()
    }

    /// Replace the `Nodes` table (D-062-2: RunMat's constructors create no
    /// custom node variables).
    fn with_nodes(g: &Value, names: &[&str], columns: Vec<Value>) -> Value {
        let mut g = g.clone();
        let Value::Object(object) = &mut g else {
            panic!("expected graph object");
        };
        let table =
            table_from_columns(names.iter().map(|s| s.to_string()).collect(), columns).unwrap();
        object.properties.insert("Nodes".to_string(), table);
        g
    }

    /// Case `met-graph-before`: the cycle 1->2->3->1 with node variables Met
    /// and Atom, stored in the observed (sorted) edge order.
    fn met_graph() -> Value {
        with_nodes(
            &digraph(&[1.0, 2.0, 3.0], &[2.0, 3.0, 1.0]),
            &["Met", "Atom"],
            vec![cellstr(&["m1", "m2", "m1"], 3, 1), col(&[1.0, 2.0, 3.0])],
        )
    }

    // ---- observed (normative) ---------------------------------------------

    /// reordernodes.signature-primary, reordernodes.nodes (cases
    /// `met-graph-before`, `met-graph`, COBRA form).
    #[test]
    fn observed_reordernodes_met_graph_nodes() {
        let g = met_graph();
        assert_eq!(ends(&g), vec![(1, 2), (2, 3), (3, 1)]);
        let h = call("reordernodes", &[g, row(&[3.0, 1.0, 2.0])]).unwrap();
        let nodes = vars(&h, "Nodes");
        let names: Vec<&str> = nodes.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, vec!["Met", "Atom"]);
        let Value::Cell(met) = &nodes[0].1 else {
            panic!("Met must stay a cell, got {:?}", nodes[0].1);
        };
        assert_eq!((met.rows, met.cols), (3, 1));
        assert_eq!(texts(&nodes[0].1), vec!["m1", "m1", "m2"]);
        assert_eq!(numeric(&nodes[1].1), (3, 1, vec![3.0, 1.0, 2.0]));
        // reordernodes.edges (case `met-graph`)
        assert_eq!(ends(&h), vec![(1, 2), (2, 3), (3, 1)]);
        assert_eq!(
            numeric(&var(&h, "Edges", "EndNodes")).2,
            vec![1.0, 2.0, 3.0, 2.0, 3.0, 1.0]
        );
    }

    /// reordernodes.edges (case `idx-out`).
    #[test]
    fn observed_reordernodes_idx() {
        let (h, idx) = reorder2(&[met_graph(), row(&[3.0, 1.0, 2.0])]);
        assert_eq!(idx, vec![3.0, 1.0, 2.0]);
        assert_eq!(ends(&h), vec![(1, 2), (2, 3), (3, 1)]);
    }

    /// D-062-1: the observed set-up builds the graph from an adjacency
    /// matrix, which RunMat stores column by column ([3 1; 1 2; 2 3]), so
    /// idx refers to that order. Fails until the constructor sorts edges.
    #[test]
    #[ignore = "D-062-1: RunMat's digraph(A) stores edges in column order, not sorted"]
    fn deviation_reordernodes_idx_for_adjacency_input() {
        let a = Tensor::new(
            vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            vec![3, 3],
        )
        .unwrap();
        let g = call("digraph", &[Value::Tensor(a)]).unwrap();
        let (_, idx) = reorder2(&[g, row(&[3.0, 1.0, 2.0])]);
        assert_eq!(idx, vec![3.0, 1.0, 2.0]);
    }

    /// The idx relation holds for the adjacency-built graph (D-062-1):
    /// row k of H.Edges is row idx(k) of G.Edges, renumbered.
    #[test]
    fn observed_reordernodes_idx_relation_for_adjacency_input() {
        let a = Tensor::new(
            vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            vec![3, 3],
        )
        .unwrap();
        let g = call("digraph", &[Value::Tensor(a)]).unwrap();
        let order = [3usize, 1, 2];
        let (h, idx) = reorder2(&[g.clone(), row(&[3.0, 1.0, 2.0])]);
        let (ge, he) = (ends(&g), ends(&h));
        assert_eq!(he, vec![(1, 2), (2, 3), (3, 1)]);
        let new_of = |old: usize| order.iter().position(|&o| o == old).unwrap() + 1;
        for (k, &i) in idx.iter().enumerate() {
            let (s, t) = ge[i as usize - 1];
            assert_eq!(he[k], (new_of(s), new_of(t)));
        }
    }

    /// reordernodes.weights (case `weighted`).
    #[test]
    fn observed_reordernodes_weights() {
        let g = call(
            "digraph",
            &[row(&[1.0, 2.0]), row(&[2.0, 3.0]), row(&[7.0, 9.0])],
        )
        .unwrap();
        let (h, idx) = reorder2(&[g, row(&[2.0, 3.0, 1.0])]);
        assert_eq!(ends(&h), vec![(1, 2), (3, 1)]);
        assert_eq!(numeric(&var(&h, "Edges", "Weight")), (2, 1, vec![9.0, 7.0]));
        assert_eq!(idx, vec![2.0, 1.0]);
    }

    /// reordernodes.undirected (case `undirected`).
    #[test]
    fn observed_reordernodes_undirected() {
        let h = call(
            "reordernodes",
            &[graph(&[1.0, 2.0], &[2.0, 3.0]), row(&[3.0, 2.0, 1.0])],
        )
        .unwrap();
        assert_eq!(class(&h), "graph");
        assert_eq!(ends(&h), vec![(1, 2), (2, 3)]);
        assert_eq!(
            numeric(&var(&h, "Edges", "EndNodes")).2,
            vec![1.0, 2.0, 2.0, 3.0]
        );
    }

    /// reordernodes.names (case `names`): order given as names.
    #[test]
    fn observed_reordernodes_names() {
        let g = call(
            "digraph",
            &[cellstr(&["a", "b"], 1, 2), cellstr(&["b", "c"], 1, 2)],
        )
        .unwrap();
        let h = call("reordernodes", &[g, cellstr(&["c", "a", "b"], 1, 3)]).unwrap();
        let nodes = vars(&h, "Nodes");
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].0, "Name");
        assert_eq!(texts(&nodes[0].1), vec!["c", "a", "b"]);
        // End nodes by name, column-major [a b; b c].
        assert_eq!(
            texts(&var(&h, "Edges", "EndNodes")),
            vec!["a", "b", "b", "c"]
        );
    }

    /// reordernodes.identity (cases `identity`, `class`).
    #[test]
    fn observed_reordernodes_identity_and_class() {
        let g = digraph(&[1.0, 2.0], &[2.0, 3.0]);
        let (h, idx) = reorder2(&[g.clone(), row(&[1.0, 2.0, 3.0])]);
        assert_eq!(ends(&h), vec![(1, 2), (2, 3)]);
        assert_eq!(idx, vec![1.0, 2.0]);
        let h = call("reordernodes", &[g, row(&[3.0, 2.0, 1.0])]).unwrap();
        assert_eq!(class(&h), "digraph");
    }

    /// reordernodes.error-repeat (cases `err-repeat`, `err-short`),
    /// reordernodes.error-range (case `err-range`).
    #[test]
    fn rejects_reordernodes_errors() {
        let g = digraph(&[1.0, 2.0], &[2.0, 3.0]);
        assert_eq!(
            err_id(&[g.clone(), row(&[1.0, 1.0, 2.0])]),
            "RunMat:reordernodes:InvalidOrder"
        );
        assert_eq!(
            err_id(&[g.clone(), row(&[2.0, 1.0])]),
            "RunMat:reordernodes:InvalidOrder"
        );
        assert_eq!(
            err_id(&[g, row(&[1.0, 2.0, 4.0])]),
            "RunMat:reordernodes:InvalidNodeID"
        );
    }

    // ---- documented choices (non-normative) -------------------------------

    /// U-062-6: non-graph input, arity, output count.
    #[test]
    fn unresolved_choice_reordernodes_inputs() {
        let g = digraph(&[1.0, 2.0], &[2.0, 3.0]);
        assert_eq!(
            err_id(&[row(&[1.0]), row(&[1.0])]),
            "RunMat:reordernodes:UndefinedFunction"
        );
        assert_eq!(
            err_id(std::slice::from_ref(&g)),
            "RunMat:reordernodes:InvalidInput"
        );
        assert_eq!(
            identifier(block_on(boxed(Vec::new())).unwrap_err()),
            "RunMat:reordernodes:InvalidInput"
        );
        assert_eq!(
            err_id(&[g.clone(), row(&[1.0, 2.0, 3.0]), chr("x")]),
            "RunMat:reordernodes:UnsupportedOption"
        );
        let err = multi(&[g.clone(), row(&[1.0, 2.0, 3.0])], 3).unwrap_err();
        assert_eq!(identifier(err), "RunMat:reordernodes:TooManyOutputs");
        assert!(multi(&[g.clone(), row(&[1.0, 2.0, 3.0])], 0)
            .unwrap()
            .is_empty());
        let one = multi(&[g, row(&[1.0, 2.0, 3.0])], 1).unwrap();
        assert_eq!(one.len(), 1);
    }

    /// U-062-7: order validation.
    #[test]
    fn unresolved_choice_reordernodes_order_validation() {
        let g = digraph(&[1.0, 2.0], &[2.0, 3.0]);
        for bad in [[0.0, 1.0, 2.0], [1.5, 2.0, 3.0], [f64::NAN, 1.0, 2.0]] {
            assert_eq!(
                err_id(&[g.clone(), row(&bad)]),
                "RunMat:reordernodes:InvalidNodeID"
            );
        }
        // Range before completeness: a too-long order with an extra index.
        assert_eq!(
            err_id(&[g.clone(), row(&[1.0, 2.0, 3.0, 4.0])]),
            "RunMat:reordernodes:InvalidNodeID"
        );
        assert_eq!(
            err_id(&[g.clone(), row(&[1.0, 2.0, 3.0, 3.0])]),
            "RunMat:reordernodes:InvalidOrder"
        );
        assert_eq!(
            err_id(&[g.clone(), row(&[])]),
            "RunMat:reordernodes:InvalidOrder"
        );
        // Names on an unnamed graph, unknown names, repeated names.
        assert_eq!(
            err_id(&[g.clone(), cellstr(&["a", "b", "c"], 1, 3)]),
            "RunMat:reordernodes:UnknownNodeName"
        );
        let named = call(
            "digraph",
            &[cellstr(&["a", "b"], 1, 2), cellstr(&["b", "c"], 1, 2)],
        )
        .unwrap();
        assert_eq!(
            err_id(&[named.clone(), cellstr(&["a", "b", "z"], 1, 3)]),
            "RunMat:reordernodes:UnknownNodeName"
        );
        assert_eq!(
            err_id(&[named.clone(), cellstr(&["a", "a", "b"], 1, 3)]),
            "RunMat:reordernodes:InvalidOrder"
        );
        // Indices on a named graph, and a column order, are accepted.
        let h = call("reordernodes", &[named, col(&[3.0, 1.0, 2.0])]).unwrap();
        assert_eq!(texts(&var(&h, "Nodes", "Name")), vec!["c", "a", "b"]);
        assert_eq!(
            err_id(&[g, Value::Struct(runmat_value::StructValue::new())]),
            "RunMat:reordernodes:InvalidInput"
        );
    }

    /// U-062-8, U-062-9: parallel edges keep their stored order; the
    /// synthetic Index column is renumbered; other edge variables follow
    /// their edges; the empty graph.
    #[test]
    fn unresolved_choice_reordernodes_ties_index_and_empty() {
        let g = call(
            "digraph",
            &[
                row(&[2.0, 1.0, 2.0]),
                row(&[1.0, 2.0, 1.0]),
                row(&[5.0, 6.0, 7.0]),
            ],
        )
        .unwrap();
        let (h, idx) = reorder2(&[g, row(&[2.0, 1.0])]);
        assert_eq!(ends(&h), vec![(1, 2), (1, 2), (2, 1)]);
        assert_eq!(idx, vec![1.0, 3.0, 2.0]);
        assert_eq!(numeric(&var(&h, "Edges", "Weight")).2, vec![5.0, 7.0, 6.0]);
        assert_eq!(numeric(&var(&h, "Nodes", "Index")).2, vec![1.0, 2.0]);

        let mut g = digraph(&[1.0, 2.0], &[2.0, 3.0]);
        if let Value::Object(object) = &mut g {
            let table = table_from_columns(
                vec!["EndNodes".to_string(), "Code".to_string()],
                vec![
                    Value::Tensor(Tensor::new(vec![1.0, 2.0, 2.0, 3.0], vec![2, 2]).unwrap()),
                    cellstr(&["x", "y"], 2, 1),
                ],
            )
            .unwrap();
            object.properties.insert("Edges".to_string(), table);
        }
        let (h, idx) = reorder2(&[g, row(&[3.0, 2.0, 1.0])]);
        // 1->2 becomes 3->2, 2->3 becomes 2->1: sorted [2 1; 3 2].
        assert_eq!(ends(&h), vec![(2, 1), (3, 2)]);
        assert_eq!(idx, vec![2.0, 1.0]);
        assert_eq!(texts(&var(&h, "Edges", "Code")), vec!["y", "x"]);

        let empty = call("graph", &[]).unwrap();
        let (h, idx) = reorder2(&[empty, row(&[])]);
        assert_eq!(class(&h), "graph");
        assert!(idx.is_empty());
        assert!(ends(&h).is_empty());
    }
}
