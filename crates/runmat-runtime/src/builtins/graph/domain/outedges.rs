//! Clean-room `outedges` (outgoing edges of a graph node).
//!
//! Clean-room feature `specs/062-graph-n5/`. Normative behaviour comes only
//! from the approved export `outedges` 0.2.0 (see that feature's
//! `provenance.md`). Edge indices refer to RunMat's stored edge list
//! (`G.Edges` row order, D-062-1). Shares the node-resolution and output
//! helpers of `inedges` (`paths.rs`, feature 059).

use super::paths::{
    cell, check_output_count, double, graph_input, name_value, outputs, query_error, resolve_node,
    NodeArg,
};
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

const OUTEDGES_INPUTS: [BuiltinParamDescriptor; 2] = [
    param("G", "Digraph or graph object."),
    param("node", "One node, as index or name."),
];
const OUTEDGES_OUTPUT_EID: BuiltinParamDescriptor =
    param("eid", "Column of outgoing edge indices (double).");
const OUTEDGES_OUTPUTS: [BuiltinParamDescriptor; 1] = [OUTEDGES_OUTPUT_EID];
const OUTEDGES_OUTPUTS_TWO: [BuiltinParamDescriptor; 2] = [
    OUTEDGES_OUTPUT_EID,
    param(
        "nid",
        "Column of the other end nodes (indices, a cell of names, or strings).",
    ),
];
const OUTEDGES_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "eid = outedges(G, node)",
        inputs: &OUTEDGES_INPUTS,
        outputs: &OUTEDGES_OUTPUTS,
    },
    BuiltinSignatureDescriptor {
        label: "[eid, nid] = outedges(G, node)",
        inputs: &OUTEDGES_INPUTS,
        outputs: &OUTEDGES_OUTPUTS_TWO,
    },
];

macro_rules! outedges_error {
    ($code:literal, $id:literal, $when:literal) => {
        BuiltinErrorDescriptor {
            code: concat!("RM.OUTEDGES.", $code),
            identifier: Some(concat!("RunMat:outedges:", $id)),
            when: $when,
            message: concat!("outedges: ", $when),
        }
    };
}

/// Common errors (U-062-1..3; index 2 is `Internal`, the fallback of
/// `query_error`) followed by the observed ones.
const OUTEDGES_ERRORS: &[BuiltinErrorDescriptor] = &[
    outedges_error!(
        "INVALID_INPUT",
        "InvalidInput",
        "A required input is missing or an input is malformed."
    ),
    outedges_error!(
        "UNSUPPORTED_OPTION",
        "UnsupportedOption",
        "The call uses a form or option that is not yet specified for RunMat."
    ),
    outedges_error!(
        "INTERNAL",
        "Internal",
        "RunMat cannot build the result value."
    ),
    outedges_error!(
        "UNDEFINED_FUNCTION",
        "UndefinedFunction",
        "The first input is not a graph or digraph object."
    ),
    outedges_error!(
        "TOO_MANY_OUTPUTS",
        "TooManyOutputs",
        "More than two outputs were requested."
    ),
    outedges_error!(
        "INVALID_NODE_ID",
        "InvalidNodeID",
        "The node index is not a node of the graph."
    ),
    outedges_error!(
        "INVALID_NODE_SCALAR",
        "InvalidNodeScalar",
        "Other than exactly one node was given."
    ),
    outedges_error!(
        "UNKNOWN_NODE_NAME",
        "UnknownNodeName",
        "The node name is not a node of the graph."
    ),
];

pub const OUTEDGES_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &OUTEDGES_SIGNATURES,
    output_mode: BuiltinOutputMode::ByRequestedOutputCount,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: OUTEDGES_ERRORS,
};

// ---------------------------------------------------------------------------
// outedges
// ---------------------------------------------------------------------------

/// How the node was named in the call; decides the class of `nid`
/// (outedges.names).
#[derive(Clone, Copy, PartialEq, Eq)]
enum QueryKind {
    Index,
    Char,
    String,
}

fn query_kind(value: &Value) -> QueryKind {
    match value {
        Value::String(_) | Value::StringArray(_) => QueryKind::String,
        // U-062-4: a cell holding one name answers like a char query.
        Value::CharArray(_) | Value::Cell(_) => QueryKind::Char,
        _ => QueryKind::Index,
    }
}

#[runtime_builtin(
    name = "outedges",
    category = "graph",
    summary = "Return the outgoing edges of a graph node and the nodes they reach.",
    keywords = "outedges,digraph,graph,edges,outgoing,successors",
    accel = "cpu",
    type_resolver(any_type),
    descriptor(crate::builtins::graph::domain::outedges::OUTEDGES_DESCRIPTOR),
    builtin_path = "crate::builtins::graph"
)]
pub(super) async fn outedges_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    const NAME: &str = "outedges";
    const ERRORS: &[BuiltinErrorDescriptor] = OUTEDGES_ERRORS;
    let args = gather_values(args).await?;
    let count = check_output_count(NAME, ERRORS)?;
    let Some(first) = args.first() else {
        return Err(query_error(
            NAME,
            ERRORS,
            "InvalidInput",
            "expected outedges(G, node)",
        ));
    };
    // U-062-1
    let graph = graph_input(NAME, ERRORS, first)?;
    if args.len() < 2 {
        // U-062-2
        return Err(query_error(
            NAME,
            ERRORS,
            "InvalidInput",
            "expected outedges(G, node)",
        ));
    }
    if args.len() > 2 {
        // U-062-2
        return Err(query_error(
            NAME,
            ERRORS,
            "UnsupportedOption",
            "only outedges(G, node) is specified",
        ));
    }
    let node = match resolve_node(&graph, &args[1]) {
        NodeArg::Node(index) => index,
        // outedges.error-range; U-062-4
        NodeArg::InvalidId => {
            return Err(query_error(
                NAME,
                ERRORS,
                "InvalidNodeID",
                "node index is not a node of the graph",
            ))
        }
        // outedges.error-vector; U-062-4
        NodeArg::NotScalar => {
            return Err(query_error(
                NAME,
                ERRORS,
                "InvalidNodeScalar",
                "exactly one node must be given",
            ))
        }
        // outedges.error-unknown-name; U-062-4
        NodeArg::UnknownName => {
            return Err(query_error(
                NAME,
                ERRORS,
                "UnknownNodeName",
                "node name is not a node of the graph",
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
    // outedges.order, outedges.multi: ascending stored edge index, one entry
    // per edge. outedges.undirected: every incident edge, `nid` the other
    // end; a self-loop is listed once (U-062-5).
    let outgoing: Vec<(usize, usize)> = graph
        .edges
        .iter()
        .enumerate()
        .filter_map(|(k, edge)| {
            if edge.source == node {
                Some((k, edge.target))
            } else if !graph.directed && edge.target == node {
                Some((k, edge.source))
            } else {
                None
            }
        })
        .collect();
    let k = outgoing.len();
    // outedges.empty: 0x1
    let eid = double(
        NAME,
        ERRORS,
        outgoing.iter().map(|&(e, _)| (e + 1) as f64).collect(),
        k,
        1,
    )?;
    let kind = query_kind(&args[1]);
    outputs(count, eid, || match (&graph.node_names, kind) {
        // outedges.names: char query -> cell of char.
        (Some(names), QueryKind::Char) => cell(
            NAME,
            ERRORS,
            outgoing
                .iter()
                .map(|&(_, t)| name_value(&names[t]))
                .collect(),
            k,
            1,
        ),
        // outedges.names: string query -> string array.
        (Some(names), QueryKind::String) => StringArray::new(
            outgoing.iter().map(|&(_, t)| names[t].clone()).collect(),
            vec![k, 1],
        )
        .map(Value::StringArray)
        .map_err(|err| query_error(NAME, ERRORS, "Internal", err)),
        // outedges.names (index query), outedges.order: double indices.
        _ => double(
            NAME,
            ERRORS,
            outgoing.iter().map(|&(_, t)| (t + 1) as f64).collect(),
            k,
            1,
        ),
    })
}

#[cfg(test)]
mod graph_n5_outedges_tests {
    use super::*;
    use futures::executor::block_on;
    use runmat_value::{CellArray, CharArray};

    // ---- helpers ----------------------------------------------------------

    fn row(values: &[f64]) -> Value {
        Value::Tensor(Tensor::new(values.to_vec(), vec![1, values.len()]).unwrap())
    }

    fn chr(text: &str) -> Value {
        Value::CharArray(CharArray::new_row(text))
    }

    fn string(text: &str) -> Value {
        Value::String(text.to_string())
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

    fn named_digraph(s: &[&str], t: &[&str]) -> Value {
        call("digraph", &[cellstr_row(s), cellstr_row(t)]).expect("named digraph")
    }

    fn identifier(err: RuntimeError) -> String {
        err.identifier().unwrap_or("<none>").to_string()
    }

    fn err_id(args: &[Value]) -> String {
        identifier(call("outedges", args).expect_err("expected an error"))
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

    fn boxed(args: Vec<Value>) -> futures::future::LocalBoxFuture<'static, BuiltinResult<Value>> {
        Box::pin(outedges_builtin(args))
    }

    fn multi(args: &[Value], count: usize) -> BuiltinResult<Vec<Value>> {
        let _guard = crate::output_count::push_output_count(Some(count));
        match block_on(boxed(args.to_vec()))? {
            Value::OutputList(values) => Ok(values),
            other => panic!("expected output list, got {other:?}"),
        }
    }

    /// [eid, nid] = outedges(args...).
    fn outedges2(args: &[Value]) -> (Value, Value) {
        let out = multi(args, 2).expect("outedges");
        assert_eq!(out.len(), 2);
        (out[0].clone(), out[1].clone())
    }

    fn column(values: &[f64]) -> (usize, usize, Vec<f64>) {
        (values.len(), 1, values.to_vec())
    }

    /// Stored end nodes (1-based) of a graph.
    fn stored(g: &Value) -> Vec<(usize, usize)> {
        super::super::construction::graph_data(g, "test")
            .unwrap()
            .edges
            .iter()
            .map(|e| (e.source + 1, e.target + 1))
            .collect()
    }

    /// The test digraph of case `edge-table`, stored in the observed
    /// (sorted) edge order [1 2; 1 3; 2 3; 3 1] (D-062-1).
    fn edge_table_sorted() -> Value {
        digraph(&[1.0, 1.0, 2.0, 3.0], &[2.0, 3.0, 3.0, 1.0])
    }

    /// The observed constructor call of case `edge-table`.
    fn edge_table_observed_input() -> Value {
        digraph(&[1.0, 1.0, 2.0, 3.0], &[3.0, 2.0, 3.0, 1.0])
    }

    // ---- observed (normative) ---------------------------------------------

    /// outedges.signature-primary, outedges.order (cases `edge-table`,
    /// `two-out`): eid ascending, nid the matching targets.
    #[test]
    fn observed_outedges_two_outputs() {
        let g = edge_table_sorted();
        assert_eq!(stored(&g), vec![(1, 2), (1, 3), (2, 3), (3, 1)]);
        let (eid, nid) = outedges2(&[g, num(1.0)]);
        assert_eq!(numeric(&eid), column(&[1.0, 2.0]));
        assert_eq!(numeric(&nid), column(&[2.0, 3.0]));
    }

    /// outedges.order (case `two-out` on the observed constructor input):
    /// RunMat's stored order is the input order, so eid still lists rows 1
    /// and 2 and nid their targets (D-062-1).
    #[test]
    fn observed_outedges_order_follows_stored_edges() {
        let g = edge_table_observed_input();
        let edges = stored(&g);
        let (eid, nid) = outedges2(&[g, num(1.0)]);
        let (_, _, e) = numeric(&eid);
        let (_, _, n) = numeric(&nid);
        assert_eq!(e, vec![1.0, 2.0]);
        for (row, target) in e.iter().zip(&n) {
            let (s, t) = edges[*row as usize - 1];
            assert_eq!((s, t as f64), (1, *target));
        }
    }

    /// D-062-1: the observed `nid` for the observed (unsorted) constructor
    /// input. Fails until the upstream constructor sorts edges.
    #[test]
    #[ignore = "D-062-1: RunMat's digraph constructor keeps edges in input order"]
    fn deviation_outedges_observed_order_for_unsorted_input() {
        let (eid, nid) = outedges2(&[edge_table_observed_input(), num(1.0)]);
        assert_eq!(numeric(&eid), column(&[1.0, 2.0]));
        assert_eq!(numeric(&nid), column(&[2.0, 3.0]));
    }

    /// outedges.order (case `one-out`): one output, node 3 -> edge 4.
    #[test]
    fn observed_outedges_one_output() {
        for g in [edge_table_sorted(), edge_table_observed_input()] {
            let eid = call("outedges", &[g, num(3.0)]).unwrap();
            assert_eq!(numeric(&eid), (1, 1, vec![4.0]));
        }
        let out = multi(&[edge_table_sorted(), num(3.0)], 1).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(numeric(&out[0]), (1, 1, vec![4.0]));
    }

    /// outedges.empty (case `no-out`).
    #[test]
    fn observed_outedges_no_out_edges() {
        let (eid, nid) = outedges2(&[digraph(&[1.0, 2.0], &[2.0, 3.0]), num(3.0)]);
        assert_eq!(numeric(&eid), (0, 1, vec![]));
        assert_eq!(numeric(&nid), (0, 1, vec![]));
    }

    /// outedges.empty (case `isolated`); `digraph([1 2], [2 1], [], 4)` is
    /// built as the equivalent adjacency matrix (D-036-2).
    #[test]
    fn observed_outedges_isolated_node() {
        let mut a = vec![0.0; 16];
        a[4] = 1.0; // A(1,2)
        a[1] = 1.0; // A(2,1)
        let g = call(
            "digraph",
            &[Value::Tensor(Tensor::new(a, vec![4, 4]).unwrap())],
        )
        .unwrap();
        let (eid, nid) = outedges2(&[g, num(4.0)]);
        assert_eq!(numeric(&eid), (0, 1, vec![]));
        assert_eq!(numeric(&nid), (0, 1, vec![]));
    }

    fn names_graph() -> Value {
        named_digraph(&["a", "a", "b"], &["b", "c", "c"])
    }

    /// outedges.names (case `names-char`): nid is a 2x1 cell of char.
    #[test]
    fn observed_outedges_names_char() {
        let (eid, nid) = outedges2(&[names_graph(), chr("a")]);
        assert_eq!(numeric(&eid), column(&[1.0, 2.0]));
        let Value::Cell(c) = &nid else {
            panic!("expected cell, got {nid:?}");
        };
        assert_eq!((c.rows, c.cols), (2, 1));
        let got: Vec<String> = c.data.iter().map(text).collect();
        assert_eq!(got, vec!["b", "c"]);
    }

    /// outedges.names (case `names-string`): nid is a 2x1 string array.
    #[test]
    fn observed_outedges_names_string() {
        let (eid, nid) = outedges2(&[names_graph(), string("a")]);
        assert_eq!(numeric(&eid), column(&[1.0, 2.0]));
        let Value::StringArray(s) = &nid else {
            panic!("expected string array, got {nid:?}");
        };
        assert_eq!(s.shape, vec![2, 1]);
        assert_eq!(s.data, vec!["b".to_string(), "c".to_string()]);
    }

    /// outedges.names (case `names-index`): index query gives doubles.
    #[test]
    fn observed_outedges_names_index() {
        let (eid, nid) = outedges2(&[names_graph(), num(1.0)]);
        assert_eq!(numeric(&eid), column(&[1.0, 2.0]));
        assert_eq!(numeric(&nid), column(&[2.0, 3.0]));
    }

    /// outedges.multi (case `multigraph`): each parallel edge appears.
    #[test]
    fn observed_outedges_multigraph() {
        let g = digraph(&[1.0, 1.0, 1.0], &[2.0, 2.0, 3.0]);
        let (eid, nid) = outedges2(&[g, num(1.0)]);
        assert_eq!(numeric(&eid), column(&[1.0, 2.0, 3.0]));
        assert_eq!(numeric(&nid), column(&[2.0, 2.0, 3.0]));
    }

    /// outedges.multi (case `self-loop`): the node itself is the nid.
    #[test]
    fn observed_outedges_self_loop() {
        let g = digraph(&[1.0, 1.0], &[1.0, 2.0]);
        let (eid, nid) = outedges2(&[g, num(1.0)]);
        assert_eq!(numeric(&eid), column(&[1.0, 2.0]));
        assert_eq!(numeric(&nid), column(&[1.0, 2.0]));
    }

    /// outedges.undirected (case `undirected`): both incident edges.
    #[test]
    fn observed_outedges_undirected() {
        let (eid, nid) = outedges2(&[graph(&[1.0, 2.0], &[2.0, 3.0]), num(2.0)]);
        assert_eq!(numeric(&eid), column(&[1.0, 2.0]));
        assert_eq!(numeric(&nid), column(&[1.0, 3.0]));
    }

    /// outedges.error-range, error-vector, error-unknown-name.
    #[test]
    fn rejects_outedges_errors() {
        assert_eq!(
            err_id(&[edge_table_observed_input(), num(5.0)]),
            "RunMat:outedges:InvalidNodeID"
        );
        assert_eq!(
            err_id(&[edge_table_observed_input(), row(&[1.0, 2.0])]),
            "RunMat:outedges:InvalidNodeScalar"
        );
        let g = named_digraph(&["a"], &["b"]);
        assert_eq!(err_id(&[g, chr("z")]), "RunMat:outedges:UnknownNodeName");
    }

    // ---- documented choices (non-normative) -------------------------------

    /// U-062-1..3: non-graph input, arity, output count.
    #[test]
    fn unresolved_choice_outedges_inputs() {
        assert_eq!(
            err_id(&[row(&[1.0, 2.0]), num(1.0)]),
            "RunMat:outedges:UndefinedFunction"
        );
        assert_eq!(
            err_id(&[edge_table_sorted()]),
            "RunMat:outedges:InvalidInput"
        );
        assert_eq!(
            identifier(block_on(boxed(Vec::new())).unwrap_err()),
            "RunMat:outedges:InvalidInput"
        );
        assert_eq!(
            err_id(&[edge_table_sorted(), num(1.0), num(2.0)]),
            "RunMat:outedges:UnsupportedOption"
        );
        let err = multi(&[edge_table_sorted(), num(1.0)], 3).unwrap_err();
        assert_eq!(identifier(err), "RunMat:outedges:TooManyOutputs");
        assert!(multi(&[edge_table_sorted(), num(1.0)], 0)
            .unwrap()
            .is_empty());
    }

    /// U-062-4: node validation (graph inputs use the same identifiers).
    #[test]
    fn unresolved_choice_outedges_node_validation() {
        let g = edge_table_sorted();
        for bad in [0.0, 1.5, -1.0, f64::NAN] {
            assert_eq!(
                err_id(&[g.clone(), num(bad)]),
                "RunMat:outedges:InvalidNodeID"
            );
        }
        assert_eq!(
            err_id(&[g.clone(), row(&[])]),
            "RunMat:outedges:InvalidNodeScalar"
        );
        assert_eq!(
            err_id(&[g.clone(), chr("a")]),
            "RunMat:outedges:UnknownNodeName"
        );
        let u = graph(&[1.0, 2.0], &[2.0, 3.0]);
        assert_eq!(
            err_id(&[u.clone(), num(4.0)]),
            "RunMat:outedges:InvalidNodeID"
        );
        assert_eq!(
            err_id(&[u, row(&[1.0, 2.0])]),
            "RunMat:outedges:InvalidNodeScalar"
        );
        // A one-name cell answers like a char query.
        let (_, nid) = outedges2(&[names_graph(), cellstr_row(&["b"])]);
        let Value::Cell(c) = &nid else {
            panic!("expected cell, got {nid:?}");
        };
        assert_eq!(c.data.iter().map(text).collect::<Vec<_>>(), vec!["c"]);
    }

    /// U-062-5: undirected self-loop listed once; empty name results keep
    /// the query's class.
    #[test]
    fn unresolved_choice_outedges_self_loop_and_empty_names() {
        let u = graph(&[1.0, 1.0], &[1.0, 2.0]);
        let (eid, nid) = outedges2(&[u, num(1.0)]);
        assert_eq!(numeric(&eid), column(&[1.0, 2.0]));
        assert_eq!(numeric(&nid), column(&[1.0, 2.0]));
        let (eid, nid) = outedges2(&[names_graph(), chr("c")]);
        assert_eq!(numeric(&eid), (0, 1, vec![]));
        let Value::Cell(c) = &nid else {
            panic!("expected cell, got {nid:?}");
        };
        assert_eq!((c.rows, c.cols), (0, 1));
        let (_, nid) = outedges2(&[names_graph(), string("c")]);
        let Value::StringArray(s) = &nid else {
            panic!("expected string array, got {nid:?}");
        };
        assert_eq!(s.shape, vec![0, 1]);
    }
}
