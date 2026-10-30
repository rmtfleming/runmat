//! Clean-room `addnode` (append nodes to a graph or digraph).
//!
//! Clean-room feature `specs/052-data-n2/`. Normative behaviour comes only
//! from the approved export `addnode` 0.2.0 (see that feature's
//! `provenance.md`). The result is a new object built with upstream's
//! `graph_object`, so `G.Edges`, `G.Nodes` and `NumNodes` stay consistent.

use super::construction::{graph_data, graph_object};
use super::parsing::parse_string_vector;
use super::*;

const NAME: &str = "addnode";

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

const ADDNODE_COUNT_INPUTS: [BuiltinParamDescriptor; 2] = [
    param("G", "Graph or digraph object."),
    param("nodeCount", "Nonnegative number of unnamed nodes to add."),
];
const ADDNODE_NAMES_INPUTS: [BuiltinParamDescriptor; 2] = [
    param("G", "Graph or digraph object."),
    param(
        "nodeNames",
        "Names of the nodes to add (char vector or cellstr).",
    ),
];
const ADDNODE_OUTPUTS: [BuiltinParamDescriptor; 1] =
    [param("H", "New graph or digraph of the same class.")];
const ADDNODE_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "H = addnode(G, nodeCount)",
        inputs: &ADDNODE_COUNT_INPUTS,
        outputs: &ADDNODE_OUTPUTS,
    },
    BuiltinSignatureDescriptor {
        label: "H = addnode(G, nodeNames)",
        inputs: &ADDNODE_NAMES_INPUTS,
        outputs: &ADDNODE_OUTPUTS,
    },
];

macro_rules! addnode_error {
    ($code:literal, $id:literal, $when:literal) => {
        BuiltinErrorDescriptor {
            code: concat!("RM.ADDNODE.", $code),
            identifier: Some(concat!("RunMat:addnode:", $id)),
            when: $when,
            message: concat!("addnode: ", $when),
        }
    };
}

const ADDNODE_ERRORS: [BuiltinErrorDescriptor; 7] = [
    addnode_error!(
        "INVALID_INPUT",
        "InvalidInput",
        "A required input is missing or an input is malformed."
    ),
    addnode_error!(
        "UNSUPPORTED_OPTION",
        "UnsupportedOption",
        "The call uses a form or node input that is not yet specified for RunMat."
    ),
    addnode_error!(
        "INTERNAL",
        "Internal",
        "RunMat cannot build the result value."
    ),
    addnode_error!(
        "UNDEFINED_FUNCTION",
        "UndefinedFunction",
        "The first input is not a graph or digraph object."
    ),
    addnode_error!(
        "TOO_MANY_OUTPUTS",
        "TooManyOutputs",
        "More than one output was requested."
    ),
    addnode_error!(
        "NON_UNIQUE_NAMES",
        "NonUniqueNames",
        "A node name is already used by a node."
    ),
    addnode_error!(
        "INVALID_NR_NODES",
        "InvalidNrNodes",
        "The node count is not a nonnegative integer."
    ),
];

const INVALID_INPUT: usize = 0;
const UNSUPPORTED_OPTION: usize = 1;
const INTERNAL: usize = 2;
const UNDEFINED_FUNCTION: usize = 3;
const TOO_MANY_OUTPUTS: usize = 4;
const NON_UNIQUE_NAMES: usize = 5;
const INVALID_NR_NODES: usize = 6;

pub const ADDNODE_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &ADDNODE_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ADDNODE_ERRORS,
};

fn addnode_error(kind: usize, detail: impl Into<String>) -> RuntimeError {
    error_from_descriptor(
        NAME,
        ADDNODE_ERRORS[kind].clone(),
        format!("{NAME}: {}", detail.into()),
    )
}

// ---------------------------------------------------------------------------
// Builtin
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "addnode",
    category = "graph",
    summary = "Add nodes, given as a count or as names, to a graph or digraph.",
    keywords = "addnode,graph,digraph,add,node,edit",
    accel = "cpu",
    type_resolver(any_type),
    descriptor(crate::builtins::graph::domain::addnode::ADDNODE_DESCRIPTOR),
    builtin_path = "crate::builtins::graph"
)]
pub(super) async fn addnode_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    let args = gather_values(args).await?;
    // U-052-1: one output.
    if let Some(count) = crate::output_count::current_output_count() {
        if count > 1 {
            return Err(addnode_error(
                TOO_MANY_OUTPUTS,
                format!("addnode returns one output, {count} were requested"),
            ));
        }
    }
    let Some(first) = args.first() else {
        return Err(addnode_error(INVALID_INPUT, "expected addnode(G, nodes)"));
    };
    // U-052-2: anything that is not a graph/digraph object.
    let is_graph = matches!(
        first,
        Value::Object(object)
            if object.class_name == GRAPH_CLASS || object.class_name == DIGRAPH_CLASS
    );
    if !is_graph {
        return Err(addnode_error(
            UNDEFINED_FUNCTION,
            "undefined for input that is not a graph or digraph object",
        ));
    }
    let mut graph = graph_data(first, NAME).map_err(|err| {
        addnode_error(
            INVALID_INPUT,
            format!("invalid graph object: {}", err.message()),
        )
    })?;
    match args.len() {
        1 => return Err(addnode_error(INVALID_INPUT, "expected addnode(G, nodes)")),
        2 => {}
        _ => {
            return Err(addnode_error(
                UNSUPPORTED_OPTION,
                "only addnode(G, nodeCount) and addnode(G, nodeNames) are specified",
            ))
        }
    }
    match &args[1] {
        Value::Num(_) | Value::Int(_) | Value::Tensor(_) => {
            let count = node_count(&args[1])?;
            if graph.node_names.is_some() && graph.node_count > 0 {
                // U-052-2: unnamed nodes on a named graph are not specified.
                return Err(addnode_error(
                    UNSUPPORTED_OPTION,
                    "adding a node count to a graph with node names is not specified",
                ));
            }
            // addnode.count: append at the end, edges unchanged.
            graph.node_count += count;
        }
        Value::String(_) | Value::StringArray(_) | Value::CharArray(_) | Value::Cell(_) => {
            let new_names = parse_string_vector(&args[1]).map_err(|err| {
                addnode_error(UNSUPPORTED_OPTION, format!("unsupported node names: {err}"))
            })?;
            if new_names.is_empty() || new_names.iter().any(String::is_empty) {
                // U-052-2
                return Err(addnode_error(
                    UNSUPPORTED_OPTION,
                    "empty node names are not specified",
                ));
            }
            // addnode.names-to-unnamed: existing unnamed nodes become NodeK.
            let mut names = graph.node_names.take().unwrap_or_else(|| {
                (1..=graph.node_count)
                    .map(|index| format!("Node{index}"))
                    .collect()
            });
            for name in new_names {
                // addnode.error-dup-name (and U-052-2 for repeats in the list).
                if names.contains(&name) {
                    return Err(addnode_error(
                        NON_UNIQUE_NAMES,
                        format!("node name '{name}' is already used"),
                    ));
                }
                names.push(name);
            }
            // addnode.names: appended in order.
            graph.node_count = names.len();
            graph.node_names = Some(names);
        }
        _ => {
            // U-052-2 (addnode.q-graph-variables)
            return Err(addnode_error(
                UNSUPPORTED_OPTION,
                "nodes must be a count, a char vector, a string array or a cellstr",
            ));
        }
    }
    graph_object(graph).map_err(|err| addnode_error(INTERNAL, err.message()))
}

/// A nonnegative integer node count (addnode.error-negative; U-052-2 for
/// NaN, Inf and fractions). Numeric non-scalars are UnsupportedOption.
fn node_count(value: &Value) -> BuiltinResult<usize> {
    let count = match value {
        Value::Num(n) => *n,
        Value::Int(i) => i.to_f64(),
        Value::Tensor(t) if t.rows * t.cols == 1 && t.shape.iter().product::<usize>() == 1 => {
            t.materialize_f64()[0]
        }
        _ => {
            return Err(addnode_error(
                UNSUPPORTED_OPTION,
                "a numeric node input must be a scalar count",
            ))
        }
    };
    if !count.is_finite() || count < 0.0 || count.fract() != 0.0 {
        return Err(addnode_error(
            INVALID_NR_NODES,
            "the number of nodes must be a nonnegative integer",
        ));
    }
    Ok(count as usize)
}

#[cfg(test)]
mod addnode_tests {
    use super::*;
    use futures::executor::block_on;
    use runmat_value::{CellArray, CharArray};

    // ---- helpers ----------------------------------------------------------

    fn row(values: &[f64]) -> Value {
        Value::Tensor(Tensor::new(values.to_vec(), vec![1, values.len()]).unwrap())
    }

    fn num(value: f64) -> Value {
        Value::Num(value)
    }

    fn chr(text: &str) -> Value {
        Value::CharArray(CharArray::new_row(text))
    }

    fn cellstr(values: &[&str]) -> Value {
        Value::Cell(
            CellArray::new(values.iter().map(|s| chr(s)).collect(), 1, values.len()).unwrap(),
        )
    }

    fn call(name: &str, args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async(name, args))
    }

    fn identifier(err: RuntimeError) -> String {
        err.identifier().unwrap_or("<none>").to_string()
    }

    fn err_id(args: &[Value]) -> String {
        identifier(call("addnode", args).expect_err("expected an error"))
    }

    /// `graph([1 2], [2 3])` — the 3-node path of cases `count`/`zero`.
    fn path3() -> Value {
        call("graph", &[row(&[1.0, 2.0]), row(&[2.0, 3.0])]).expect("graph")
    }

    /// `graph({'a'}, {'b'})`.
    fn named_ab() -> Value {
        call("graph", &[cellstr(&["a"]), cellstr(&["b"])]).expect("named graph")
    }

    fn empty_graph() -> Value {
        call("graph", &[]).expect("graph()")
    }

    fn object(g: &Value) -> &ObjectInstance {
        let Value::Object(object) = g else {
            panic!("expected graph object, got {g:?}");
        };
        object
    }

    fn class(g: &Value) -> &'static str {
        let object = object(g);
        if object.class_name == GRAPH_CLASS {
            "graph"
        } else if object.class_name == DIGRAPH_CLASS {
            "digraph"
        } else {
            "other"
        }
    }

    fn scalar(value: Value) -> f64 {
        match value {
            Value::Num(n) => n,
            Value::Int(i) => i.to_f64(),
            Value::Tensor(t) if t.rows * t.cols == 1 => t.materialize_f64()[0],
            other => panic!("expected scalar, got {other:?}"),
        }
    }

    /// numnodes / numedges through the public builtins (consistency).
    fn counts(g: &Value) -> (usize, usize) {
        let n = scalar(call("numnodes", std::slice::from_ref(g)).unwrap()) as usize;
        let m = scalar(call("numedges", std::slice::from_ref(g)).unwrap()) as usize;
        (n, m)
    }

    fn edge_vars(g: &Value) -> runmat_value::StructValue {
        let Some(Value::Object(edges)) = object(g).properties.get("Edges") else {
            panic!("expected Edges table");
        };
        table_variables(edges).unwrap()
    }

    /// Numeric `G.Edges.EndNodes` rows and the EndNodes shape.
    fn ends(g: &Value) -> (Vec<usize>, Vec<[f64; 2]>) {
        let vars = edge_vars(g);
        let Some(Value::Tensor(t)) = vars.fields.get("EndNodes") else {
            panic!(
                "expected numeric EndNodes, got {:?}",
                vars.fields.get("EndNodes")
            );
        };
        let d = t.materialize_f64();
        (
            vec![t.rows, t.cols],
            (0..t.rows).map(|r| [d[r], d[r + t.rows]]).collect(),
        )
    }

    /// String `G.Edges.EndNodes` rows (named graphs).
    fn named_ends(g: &Value) -> Vec<[String; 2]> {
        let vars = edge_vars(g);
        let Some(Value::StringArray(a)) = vars.fields.get("EndNodes") else {
            panic!(
                "expected string EndNodes, got {:?}",
                vars.fields.get("EndNodes")
            );
        };
        (0..a.rows)
            .map(|r| [a.data[r].clone(), a.data[r + a.rows].clone()])
            .collect()
    }

    fn node_names(g: &Value) -> Option<Vec<String>> {
        let Some(Value::Object(nodes)) = object(g).properties.get("Nodes") else {
            return None;
        };
        match table_variables(nodes).unwrap().fields.get("Name") {
            Some(Value::StringArray(a)) => Some(a.data.clone()),
            None => None,
            Some(other) => panic!("unexpected Name {other:?}"),
        }
    }

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|s| s.to_string()).collect()
    }

    fn s2(a: &str, b: &str) -> [String; 2] {
        [a.to_string(), b.to_string()]
    }

    // ---- observed (normative) ---------------------------------------------

    /// addnode.signature-primary, addnode.count, addnode.class (cases
    /// `count`, `zero`).
    #[test]
    fn observed_addnode_count_and_zero() {
        let g = path3();
        let h = call("addnode", &[g.clone(), num(2.0)]).unwrap();
        assert_eq!(class(&h), "graph");
        assert_eq!(counts(&h), (5, 2));
        assert_eq!(ends(&h).1, vec![[1.0, 2.0], [2.0, 3.0]]);
        let h = call("addnode", &[g.clone(), num(0.0)]).unwrap();
        assert_eq!(counts(&h), (3, 2));
        assert_eq!(ends(&h).1, vec![[1.0, 2.0], [2.0, 3.0]]);
        // FR-052-16: the input is not modified.
        assert_eq!(counts(&g), (3, 2));
    }

    /// addnode.count, addnode.class (case `digraph`).
    #[test]
    fn observed_addnode_digraph() {
        let g = call("digraph", &[num(1.0), num(2.0)]).unwrap();
        let h = call("addnode", &[g, num(1.0)]).unwrap();
        assert_eq!(class(&h), "digraph");
        assert_eq!(counts(&h), (3, 1));
        assert_eq!(ends(&h).1, vec![[1.0, 2.0]]);
    }

    /// addnode.count (case `empty`): EndNodes stays 0-by-2.
    #[test]
    fn observed_addnode_empty_graph() {
        let h = call("addnode", &[empty_graph(), num(3.0)]).unwrap();
        assert_eq!(class(&h), "graph");
        assert_eq!(counts(&h), (3, 0));
        assert_eq!(ends(&h), (vec![0, 2], Vec::new()));
        assert_eq!(node_names(&h), None);
    }

    /// addnode.names (cases `names`, `name-char`).
    #[test]
    fn observed_addnode_names() {
        let h = call("addnode", &[named_ab(), cellstr(&["c", "d"])]).unwrap();
        assert_eq!(class(&h), "graph");
        assert_eq!(counts(&h), (4, 1));
        assert_eq!(named_ends(&h), vec![s2("a", "b")]);
        assert_eq!(node_names(&h), Some(strings(&["a", "b", "c", "d"])));
        let h = call("addnode", &[named_ab(), chr("c")]).unwrap();
        assert_eq!(counts(&h), (3, 1));
        assert_eq!(named_ends(&h), vec![s2("a", "b")]);
        assert_eq!(node_names(&h), Some(strings(&["a", "b", "c"])));
    }

    /// addnode.names (case `names-to-unnamed-empty`).
    #[test]
    fn observed_addnode_names_to_empty_graph() {
        let h = call("addnode", &[empty_graph(), cellstr(&["p", "q"])]).unwrap();
        assert_eq!(counts(&h), (2, 0));
        assert_eq!(node_names(&h), Some(strings(&["p", "q"])));
    }

    /// addnode.names-to-unnamed (case `names-to-unnamed`).
    #[test]
    fn observed_addnode_names_to_unnamed() {
        let g = call("graph", &[num(1.0), num(2.0)]).unwrap();
        let h = call("addnode", &[g, cellstr(&["x"])]).unwrap();
        assert_eq!(counts(&h), (3, 1));
        assert_eq!(named_ends(&h), vec![s2("Node1", "Node2")]);
        assert_eq!(node_names(&h), Some(strings(&["Node1", "Node2", "x"])));
    }

    /// addnode.error-dup-name (case `err-dup-name`).
    #[test]
    fn rejects_addnode_duplicate_name() {
        assert_eq!(
            err_id(&[named_ab(), chr("a")]),
            "RunMat:addnode:NonUniqueNames"
        );
    }

    /// addnode.error-negative (case `err-negative`).
    #[test]
    fn rejects_addnode_negative_count() {
        let g = call("graph", &[num(1.0), num(2.0)]).unwrap();
        assert_eq!(err_id(&[g, num(-1.0)]), "RunMat:addnode:InvalidNrNodes");
    }

    // ---- documented choices (U-052-1, U-052-2) ------------------------------

    /// U-052-2: count forms that are not nonnegative integers.
    #[test]
    fn unresolved_choice_addnode_invalid_counts() {
        for bad in [1.5, f64::NAN, f64::INFINITY] {
            assert_eq!(
                err_id(&[path3(), num(bad)]),
                "RunMat:addnode:InvalidNrNodes",
                "count {bad}"
            );
        }
        // An integer-valued 1x1 tensor and an integer value count as a count.
        let h = call("addnode", &[path3(), row(&[1.0])]).unwrap();
        assert_eq!(counts(&h), (4, 2));
        let h = call("addnode", &[path3(), Value::Int(IntValue::I32(2))]).unwrap();
        assert_eq!(counts(&h), (5, 2));
    }

    /// U-052-2: names and node inputs outside the observed forms.
    #[test]
    fn unresolved_choice_addnode_names_and_forms() {
        // Repeated within the new names.
        assert_eq!(
            err_id(&[named_ab(), cellstr(&["c", "c"])]),
            "RunMat:addnode:NonUniqueNames"
        );
        // Equal to a generated NodeK name.
        let g = call("graph", &[num(1.0), num(2.0)]).unwrap();
        assert_eq!(err_id(&[g, chr("Node2")]), "RunMat:addnode:NonUniqueNames");
        // String arrays are accepted like a cellstr.
        let names = Value::StringArray(StringArray::new(strings(&["c", "d"]), vec![1, 2]).unwrap());
        let h = call("addnode", &[named_ab(), names]).unwrap();
        assert_eq!(node_names(&h), Some(strings(&["a", "b", "c", "d"])));
        // An empty name, a count on a named graph, a numeric vector, a
        // non-text cell, extra inputs.
        assert_eq!(
            err_id(&[named_ab(), chr("")]),
            "RunMat:addnode:UnsupportedOption"
        );
        assert_eq!(
            err_id(&[named_ab(), num(1.0)]),
            "RunMat:addnode:UnsupportedOption"
        );
        assert_eq!(
            err_id(&[path3(), row(&[1.0, 2.0])]),
            "RunMat:addnode:UnsupportedOption"
        );
        let bad_cell = Value::Cell(CellArray::new(vec![num(1.0)], 1, 1).unwrap());
        assert_eq!(
            err_id(&[path3(), bad_cell]),
            "RunMat:addnode:UnsupportedOption"
        );
        assert_eq!(
            err_id(&[path3(), num(1.0), num(1.0)]),
            "RunMat:addnode:UnsupportedOption"
        );
    }

    /// U-052-1, U-052-2: missing inputs and a non-graph first input.
    #[test]
    fn unresolved_choice_addnode_arity_and_input_class() {
        assert_eq!(err_id(&[path3()]), "RunMat:addnode:InvalidInput");
        assert_eq!(err_id(&[]), "RunMat:addnode:InvalidInput");
        assert_eq!(
            err_id(&[num(3.0), num(1.0)]),
            "RunMat:addnode:UndefinedFunction"
        );
    }

    /// U-052-1: called directly under `push_output_count` (dispatcher quirk).
    #[test]
    fn unresolved_choice_addnode_output_count() {
        {
            let _guard = crate::output_count::push_output_count(Some(2));
            let err = block_on(addnode_builtin(vec![path3(), num(1.0)])).unwrap_err();
            assert_eq!(identifier(err), "RunMat:addnode:TooManyOutputs");
        }
        let _guard = crate::output_count::push_output_count(Some(1));
        let h = block_on(addnode_builtin(vec![path3(), num(1.0)])).unwrap();
        assert_eq!(counts(&h), (4, 2));
    }
}
