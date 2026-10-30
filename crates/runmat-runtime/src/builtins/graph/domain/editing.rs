//! Clean-room `rmnode`, `addedge`, `rmedge` and `laplacian` (graph editing).
//!
//! Clean-room feature `specs/043-graph-editing/`. Normative behaviour comes
//! only from the approved exports `rmnode`, `addedge`, `rmedge` and
//! `laplacian` 0.2.0 (see that feature's `provenance.md`). Graph objects are
//! values: every builtin here returns a new object built with upstream's
//! `graph_object`, so `G.Edges`, `G.Nodes` and `NumNodes` stay consistent.

use super::construction::{graph_data, graph_object};
use super::parsing::{numeric_f64_vector, parse_string_vector};
use super::*;
use runmat_value::SparseTensor;

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

const fn optional(name: &'static str, description: &'static str) -> BuiltinParamDescriptor {
    BuiltinParamDescriptor {
        name,
        ty: BuiltinParamType::Any,
        arity: BuiltinParamArity::Optional,
        default: None,
        description,
    }
}

const P_GRAPH: BuiltinParamDescriptor = param("G", "Graph or digraph object.");
const P_RESULT: BuiltinParamDescriptor = param("H", "New graph or digraph of the same class.");

const RMNODE_INPUTS: [BuiltinParamDescriptor; 2] =
    [P_GRAPH, param("nodeIDs", "Node indices or node names.")];
const RMNODE_OUTPUTS: [BuiltinParamDescriptor; 1] = [P_RESULT];
const RMNODE_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "H = rmnode(G, nodeIDs)",
    inputs: &RMNODE_INPUTS,
    outputs: &RMNODE_OUTPUTS,
}];

const ADDEDGE_INPUTS: [BuiltinParamDescriptor; 3] = [
    P_GRAPH,
    param("s", "Source node indices or names."),
    param("t", "Target node indices or names."),
];
const ADDEDGE_WEIGHTED_INPUTS: [BuiltinParamDescriptor; 4] = [
    P_GRAPH,
    param("s", "Source node indices or names."),
    param("t", "Target node indices or names."),
    param("w", "Edge weights (scalar or one per new edge)."),
];
const ADDEDGE_OUTPUTS: [BuiltinParamDescriptor; 1] = [P_RESULT];
const ADDEDGE_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "H = addedge(G, s, t)",
        inputs: &ADDEDGE_INPUTS,
        outputs: &ADDEDGE_OUTPUTS,
    },
    BuiltinSignatureDescriptor {
        label: "H = addedge(G, s, t, w)",
        inputs: &ADDEDGE_WEIGHTED_INPUTS,
        outputs: &ADDEDGE_OUTPUTS,
    },
];

const RMEDGE_INPUTS: [BuiltinParamDescriptor; 3] = [
    P_GRAPH,
    param(
        "s",
        "Source nodes, or edge indices in the two-argument form.",
    ),
    optional("t", "Target nodes."),
];
const RMEDGE_OUTPUTS: [BuiltinParamDescriptor; 1] = [P_RESULT];
const RMEDGE_SIGNATURES: [BuiltinSignatureDescriptor; 2] = [
    BuiltinSignatureDescriptor {
        label: "H = rmedge(G, s, t)",
        inputs: &RMEDGE_INPUTS,
        outputs: &RMEDGE_OUTPUTS,
    },
    BuiltinSignatureDescriptor {
        label: "H = rmedge(G, idx)",
        inputs: &RMEDGE_INPUTS,
        outputs: &RMEDGE_OUTPUTS,
    },
];

const LAPLACIAN_INPUTS: [BuiltinParamDescriptor; 1] = [param(
    "G",
    "Undirected graph without self-loops or parallel edges.",
)];
const LAPLACIAN_OUTPUTS: [BuiltinParamDescriptor; 1] =
    [param("L", "Sparse double numnodes-by-numnodes Laplacian.")];
const LAPLACIAN_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "L = laplacian(G)",
    inputs: &LAPLACIAN_INPUTS,
    outputs: &LAPLACIAN_OUTPUTS,
}];

macro_rules! edit_error {
    ($fn:literal, $prefix:literal, $code:literal, $id:literal, $when:literal) => {
        BuiltinErrorDescriptor {
            code: concat!("RM.", $prefix, ".", $code),
            identifier: Some(concat!("RunMat:", $fn, ":", $id)),
            when: $when,
            message: concat!($fn, ": ", $when),
        }
    };
}

/// Common errors (U-043-1..3) followed by the builtin's specific ones.
macro_rules! edit_errors {
    (
        $name:ident, $fn:literal, $prefix:literal;
        $(($code:literal, $id:literal, $when:literal)),* $(,)?
    ) => {
        const $name: &[BuiltinErrorDescriptor] = &[
            edit_error!($fn, $prefix, "INVALID_INPUT", "InvalidInput",
                "A required input is missing or an input is malformed."),
            edit_error!($fn, $prefix, "UNSUPPORTED_OPTION", "UnsupportedOption",
                "The call uses a form or option that is not yet specified for RunMat."),
            edit_error!($fn, $prefix, "INTERNAL", "Internal",
                "RunMat cannot build the result value."),
            edit_error!($fn, $prefix, "UNDEFINED_FUNCTION", "UndefinedFunction",
                "The first input is not a graph or digraph object."),
            edit_error!($fn, $prefix, "TOO_MANY_OUTPUTS", "TooManyOutputs",
                "More than one output was requested."),
            $(edit_error!($fn, $prefix, $code, $id, $when),)*
        ];
    };
}

edit_errors!(RMNODE_ERRORS, "rmnode", "RMNODE";);
edit_errors!(
    ADDEDGE_ERRORS, "addedge", "ADDEDGE";
    ("NO_WEIGHTS", "NoWeights", "Weights are given for a graph without edge weights."),
    ("EQUAL_NUMEL", "EqualNumel", "s, t (or w) have incompatible lengths."),
);
edit_errors!(
    RMEDGE_ERRORS, "rmedge", "RMEDGE";
    ("EDGE_BOUNDS", "EdgeBounds", "An edge index is not between 1 and numedges."),
    ("INVALID_NODE_ID", "InvalidNodeID", "A node identifier does not name a node of the graph."),
);
edit_errors!(
    LAPLACIAN_ERRORS, "laplacian", "LAPLACIAN";
    ("SELF_LOOPS", "SelfLoops", "The graph contains a self-loop."),
    ("MULTIGRAPH", "Multigraph", "The graph contains parallel edges."),
    ("ONLY_GRAPH_SUPPORTED", "OnlyGraphSupported", "The input is a digraph."),
);

pub const RMNODE_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &RMNODE_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: RMNODE_ERRORS,
};
pub const ADDEDGE_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &ADDEDGE_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: ADDEDGE_ERRORS,
};
pub const RMEDGE_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &RMEDGE_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: RMEDGE_ERRORS,
};
pub const LAPLACIAN_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &LAPLACIAN_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: LAPLACIAN_ERRORS,
};

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Raise `RunMat:<builtin>:<id>` from the builtin's descriptor table.
fn edit_error(
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

/// U-043-3: these builtins return one output.
fn check_output_count(
    builtin: &'static str,
    errors: &'static [BuiltinErrorDescriptor],
) -> BuiltinResult<()> {
    match crate::output_count::current_output_count() {
        Some(count) if count > 1 => Err(edit_error(
            builtin,
            errors,
            "TooManyOutputs",
            format!("{builtin} returns one output, {count} were requested"),
        )),
        _ => Ok(()),
    }
}

/// Read the graph argument. Anything that is not a `graph`/`digraph` object
/// is `UndefinedFunction` (U-043-1); a malformed object is `InvalidInput`.
/// Also reports whether `G.Edges` has a `Weight` variable (U-043-9).
fn graph_input(
    builtin: &'static str,
    errors: &'static [BuiltinErrorDescriptor],
    value: &Value,
) -> BuiltinResult<(GraphData, bool)> {
    let object = match value {
        Value::Object(object)
            if object.class_name == GRAPH_CLASS || object.class_name == DIGRAPH_CLASS =>
        {
            object
        }
        _ => {
            return Err(edit_error(
                builtin,
                errors,
                "UndefinedFunction",
                "undefined for input that is not a graph or digraph object",
            ))
        }
    };
    let graph = graph_data(value, builtin).map_err(|err| {
        edit_error(
            builtin,
            errors,
            "InvalidInput",
            format!("invalid graph object: {}", err.message()),
        )
    })?;
    let weighted = match object.properties.get("Edges") {
        Some(Value::Object(edges)) => table_variables(edges)
            .map(|vars| vars.fields.contains_key("Weight"))
            .unwrap_or(false),
        _ => false,
    };
    Ok((graph, weighted))
}

/// Build the result object (FR-043-19).
fn result_object(
    builtin: &'static str,
    errors: &'static [BuiltinErrorDescriptor],
    graph: GraphData,
) -> BuiltinResult<Value> {
    graph_object(graph).map_err(|err| edit_error(builtin, errors, "Internal", err.message()))
}

/// A node reference as given by the caller.
#[derive(Clone, Debug)]
enum NodeRef {
    Index(f64),
    Name(String),
}

fn is_numeric(value: &Value) -> bool {
    matches!(
        value,
        Value::Num(_) | Value::Int(_) | Value::Bool(_) | Value::Tensor(_) | Value::LogicalArray(_)
    )
}

/// Numeric values are indices; string, character and cell values are names.
fn node_refs(builtin: &'static str, value: &Value) -> Option<Vec<NodeRef>> {
    match value {
        Value::String(_) | Value::StringArray(_) | Value::CharArray(_) | Value::Cell(_) => {
            parse_string_vector(value)
                .ok()
                .map(|labels| labels.into_iter().map(NodeRef::Name).collect())
        }
        _ if is_numeric(value) => numeric_f64_vector(value, builtin)
            .ok()
            .map(|values| values.into_iter().map(NodeRef::Index).collect()),
        _ => None,
    }
}

/// A one-based positive integer, if `value` is one.
fn positive_index(value: f64) -> Option<usize> {
    (value.is_finite() && value >= 1.0 && value.fract() == 0.0 && value < 9.007_199_254_740_992e15)
        .then_some(value as usize)
}

/// Pairwise `s`/`t`, expanding a scalar side (addedge.scalar-expansion;
/// scalar `s` is U-043-5). `None` when the lengths are incompatible.
fn expand_pairs<T: Clone>(s: Vec<T>, t: Vec<T>) -> Option<(Vec<T>, Vec<T>)> {
    if s.len() == t.len() {
        Some((s, t))
    } else if t.len() == 1 {
        let fill = vec![t[0].clone(); s.len()];
        Some((s, fill))
    } else if s.len() == 1 {
        let fill = vec![s[0].clone(); t.len()];
        Some((fill, t))
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// rmnode
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "rmnode",
    category = "graph",
    summary = "Remove nodes and their incident edges from a graph or digraph.",
    keywords = "rmnode,graph,digraph,remove,node,edit",
    accel = "cpu",
    type_resolver(any_type),
    descriptor(crate::builtins::graph::domain::editing::RMNODE_DESCRIPTOR),
    builtin_path = "crate::builtins::graph"
)]
pub(super) async fn rmnode_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    const NAME: &str = "rmnode";
    const ERRORS: &[BuiltinErrorDescriptor] = RMNODE_ERRORS;
    let args = gather_values(args).await?;
    check_output_count(NAME, ERRORS)?;
    let Some(first) = args.first() else {
        return Err(edit_error(
            NAME,
            ERRORS,
            "InvalidInput",
            "expected rmnode(G, nodeIDs)",
        ));
    };
    let (graph, _) = graph_input(NAME, ERRORS, first)?;
    if args.len() < 2 {
        return Err(edit_error(
            NAME,
            ERRORS,
            "InvalidInput",
            "expected rmnode(G, nodeIDs)",
        ));
    }
    if args.len() > 2 {
        // U-043-2
        return Err(edit_error(
            NAME,
            ERRORS,
            "UnsupportedOption",
            "only rmnode(G, nodeIDs) is specified",
        ));
    }
    let refs = node_refs(NAME, &args[1]).ok_or_else(|| {
        edit_error(
            NAME,
            ERRORS,
            "InvalidInput",
            "node identifiers must be numeric indices or node names",
        )
    })?;
    let mut remove = vec![false; graph.node_count];
    for node in refs {
        match node {
            NodeRef::Index(value) => {
                // U-043-4
                let index = positive_index(value).ok_or_else(|| {
                    edit_error(
                        NAME,
                        ERRORS,
                        "InvalidInput",
                        "node indices must be positive integers",
                    )
                })?;
                // FR-043-06: an index beyond numnodes is ignored.
                if let Some(slot) = remove.get_mut(index - 1) {
                    *slot = true;
                }
            }
            NodeRef::Name(label) => {
                let Some(names) = &graph.node_names else {
                    // U-043-4
                    return Err(edit_error(
                        NAME,
                        ERRORS,
                        "UnsupportedOption",
                        "node names given for a graph without node names",
                    ));
                };
                // FR-043-06: an unknown name is ignored.
                if let Some(position) = names.iter().position(|name| *name == label) {
                    remove[position] = true;
                }
            }
        }
    }
    // rmnode.removes: renumber the remaining nodes in their original order.
    let mut renumber = vec![None; graph.node_count];
    let mut kept = 0usize;
    for (node, removed) in remove.iter().enumerate() {
        if !removed {
            renumber[node] = Some(kept);
            kept += 1;
        }
    }
    // rmnode.weights, U-043-7: remaining edges keep weight and stored order.
    let edges = graph
        .edges
        .iter()
        .filter_map(|edge| {
            Some(Edge {
                source: renumber[edge.source]?,
                target: renumber[edge.target]?,
                weight: edge.weight,
            })
        })
        .collect();
    // rmnode.named: remaining names are kept.
    let node_names = graph.node_names.map(|names| {
        names
            .into_iter()
            .zip(&remove)
            .filter(|(_, removed)| !**removed)
            .map(|(name, _)| name)
            .collect()
    });
    result_object(
        NAME,
        ERRORS,
        GraphData {
            directed: graph.directed,
            node_names,
            edges,
            node_count: kept,
        },
    )
}

// ---------------------------------------------------------------------------
// addedge
// ---------------------------------------------------------------------------

/// Resolve one `addedge` endpoint, growing the node set (addedge.new-nodes,
/// addedge.named, U-043-5). Errors are `(identifier, detail)`.
fn resolve_new_endpoint(
    node: &NodeRef,
    names: &mut Option<Vec<String>>,
    node_count: &mut usize,
) -> Result<usize, (&'static str, &'static str)> {
    match node {
        NodeRef::Index(value) => {
            let index = positive_index(*value)
                .ok_or(("InvalidInput", "node indices must be positive integers"))?;
            if names.is_some() && index > *node_count {
                return Err((
                    "UnsupportedOption",
                    "a numeric index beyond numnodes cannot create a named node",
                ));
            }
            *node_count = (*node_count).max(index);
            Ok(index - 1)
        }
        NodeRef::Name(label) => {
            let names = names.as_mut().ok_or((
                "UnsupportedOption",
                "node names given for a graph without node names",
            ))?;
            if let Some(position) = names.iter().position(|name| name == label) {
                return Ok(position);
            }
            names.push(label.clone());
            *node_count = names.len();
            Ok(names.len() - 1)
        }
    }
}

#[runtime_builtin(
    name = "addedge",
    category = "graph",
    summary = "Add edges (and any new nodes) to a graph or digraph.",
    keywords = "addedge,graph,digraph,add,edge,edit",
    accel = "cpu",
    type_resolver(any_type),
    descriptor(crate::builtins::graph::domain::editing::ADDEDGE_DESCRIPTOR),
    builtin_path = "crate::builtins::graph"
)]
pub(super) async fn addedge_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    const NAME: &str = "addedge";
    const ERRORS: &[BuiltinErrorDescriptor] = ADDEDGE_ERRORS;
    let args = gather_values(args).await?;
    check_output_count(NAME, ERRORS)?;
    let Some(first) = args.first() else {
        return Err(edit_error(
            NAME,
            ERRORS,
            "InvalidInput",
            "expected addedge(G, s, t)",
        ));
    };
    let (graph, weighted) = graph_input(NAME, ERRORS, first)?;
    if args.len() < 3 {
        return Err(edit_error(
            NAME,
            ERRORS,
            "InvalidInput",
            "expected addedge(G, s, t)",
        ));
    }
    if args.len() > 4 {
        // U-043-2
        return Err(edit_error(
            NAME,
            ERRORS,
            "UnsupportedOption",
            "only addedge(G, s, t) and addedge(G, s, t, w) are specified",
        ));
    }
    let endpoints = |value: &Value| {
        node_refs(NAME, value).ok_or_else(|| {
            edit_error(
                NAME,
                ERRORS,
                "InvalidInput",
                "s and t must be numeric node indices or node names",
            )
        })
    };
    let (sources, targets) = expand_pairs(endpoints(&args[1])?, endpoints(&args[2])?)
        // addedge.error-length
        .ok_or_else(|| {
            edit_error(
                NAME,
                ERRORS,
                "EqualNumel",
                "s and t must have the same number of elements",
            )
        })?;
    let count = sources.len();
    let new_weights = match args.get(3) {
        // U-043-5: unweighted additions get weight 1.
        None => vec![1.0; count],
        Some(value) if !is_numeric(value) => {
            // U-043-2: edge tables and other forms are not specified.
            return Err(edit_error(
                NAME,
                ERRORS,
                "UnsupportedOption",
                "the fourth argument must be numeric edge weights",
            ));
        }
        Some(value) => {
            if !weighted {
                // addedge.error-weight-unweighted
                return Err(edit_error(
                    NAME,
                    ERRORS,
                    "NoWeights",
                    "weights cannot be added to a graph without edge weights",
                ));
            }
            let values = numeric_f64_vector(value, NAME)
                .map_err(|err| edit_error(NAME, ERRORS, "InvalidInput", err.message()))?;
            // addedge.weights: a scalar applies to every new edge; U-043-5.
            let values = match values.len() {
                1 => vec![values[0]; count],
                len if len == count => values,
                _ => {
                    return Err(edit_error(
                        NAME,
                        ERRORS,
                        "EqualNumel",
                        "w must be a scalar or have one value per new edge",
                    ))
                }
            };
            if values.iter().any(|w| !w.is_finite() || *w < 0.0) {
                return Err(edit_error(
                    NAME,
                    ERRORS,
                    "InvalidInput",
                    "edge weights must be finite nonnegative values",
                ));
            }
            values
        }
    };

    let mut names = graph.node_names.clone();
    let has_name = sources
        .iter()
        .chain(&targets)
        .any(|node| matches!(node, NodeRef::Name(_)));
    let has_index = sources
        .iter()
        .chain(&targets)
        .any(|node| matches!(node, NodeRef::Index(_)));
    if has_name && names.is_none() && graph.node_count == 0 && !has_index {
        // U-043-5: names on the empty unnamed graph create a named graph.
        names = Some(Vec::new());
    }
    let mut node_count = graph.node_count;
    let mut edges = graph.edges;
    for ((source, target), weight) in sources.iter().zip(&targets).zip(new_weights) {
        let mut resolve = |node: &NodeRef| {
            resolve_new_endpoint(node, &mut names, &mut node_count)
                .map_err(|(id, detail)| edit_error(NAME, ERRORS, id, detail))
        };
        let source = resolve(source)?;
        let target = resolve(target)?;
        edges.push(Edge {
            source,
            target,
            weight,
        });
    }
    // addedge.edge-order / addedge.digraph-order: undirected edges smaller
    // node first; stable sort by (source, target), so weights follow their
    // edges (addedge.weights) and equal pairs keep their order.
    if !graph.directed {
        for edge in &mut edges {
            if edge.source > edge.target {
                std::mem::swap(&mut edge.source, &mut edge.target);
            }
        }
    }
    edges.sort_by_key(|edge| (edge.source, edge.target));
    result_object(
        NAME,
        ERRORS,
        GraphData {
            directed: graph.directed,
            node_names: names,
            edges,
            node_count,
        },
    )
}

// ---------------------------------------------------------------------------
// rmedge
// ---------------------------------------------------------------------------

/// Resolve an existing node for `rmedge` (rmedge.error-node-range, U-043-6).
fn resolve_existing_node(graph: &GraphData, node: &NodeRef) -> Option<usize> {
    match node {
        NodeRef::Index(value) => positive_index(*value)
            .filter(|&index| index <= graph.node_count)
            .map(|index| index - 1),
        NodeRef::Name(label) => graph
            .node_names
            .as_ref()?
            .iter()
            .position(|name| name == label),
    }
}

#[runtime_builtin(
    name = "rmedge",
    category = "graph",
    summary = "Remove edges from a graph or digraph, keeping its nodes.",
    keywords = "rmedge,graph,digraph,remove,edge,edit",
    accel = "cpu",
    type_resolver(any_type),
    descriptor(crate::builtins::graph::domain::editing::RMEDGE_DESCRIPTOR),
    builtin_path = "crate::builtins::graph"
)]
pub(super) async fn rmedge_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    const NAME: &str = "rmedge";
    const ERRORS: &[BuiltinErrorDescriptor] = RMEDGE_ERRORS;
    let args = gather_values(args).await?;
    check_output_count(NAME, ERRORS)?;
    let Some(first) = args.first() else {
        return Err(edit_error(
            NAME,
            ERRORS,
            "InvalidInput",
            "expected rmedge(G, s, t) or rmedge(G, idx)",
        ));
    };
    let (graph, _) = graph_input(NAME, ERRORS, first)?;
    if args.len() < 2 {
        return Err(edit_error(
            NAME,
            ERRORS,
            "InvalidInput",
            "expected rmedge(G, s, t) or rmedge(G, idx)",
        ));
    }
    if args.len() > 3 {
        // U-043-2
        return Err(edit_error(
            NAME,
            ERRORS,
            "UnsupportedOption",
            "only rmedge(G, s, t) and rmedge(G, idx) are specified",
        ));
    }
    let edge_count = graph.edges.len();
    let mut remove = vec![false; edge_count];
    if args.len() == 2 {
        // rmedge.by-index
        if !is_numeric(&args[1]) {
            // U-043-6
            return Err(edit_error(
                NAME,
                ERRORS,
                "InvalidInput",
                "edge indices must be numeric",
            ));
        }
        let values = numeric_f64_vector(&args[1], NAME)
            .map_err(|err| edit_error(NAME, ERRORS, "InvalidInput", err.message()))?;
        for value in values {
            // rmedge.error-index-range; U-043-6 for non-integers.
            let index = positive_index(value)
                .filter(|&index| index <= edge_count)
                .ok_or_else(|| {
                    edit_error(
                        NAME,
                        ERRORS,
                        "EdgeBounds",
                        format!("edge indices must be integers between 1 and {edge_count}"),
                    )
                })?;
            remove[index - 1] = true;
        }
    } else {
        // rmedge.by-endpoints
        let endpoints = |value: &Value| {
            node_refs(NAME, value).ok_or_else(|| {
                edit_error(
                    NAME,
                    ERRORS,
                    "InvalidInput",
                    "s and t must be numeric node indices or node names",
                )
            })
        };
        let (sources, targets) = expand_pairs(endpoints(&args[1])?, endpoints(&args[2])?)
            .ok_or_else(|| {
                // U-043-6
                edit_error(
                    NAME,
                    ERRORS,
                    "InvalidInput",
                    "s and t must have the same number of elements",
                )
            })?;
        let resolve = |node: &NodeRef| {
            resolve_existing_node(&graph, node).ok_or_else(|| {
                // rmedge.error-node-range; U-043-6 for names.
                edit_error(
                    NAME,
                    ERRORS,
                    "InvalidNodeID",
                    format!("{node:?} is not a node of the graph"),
                )
            })
        };
        let mut pairs = Vec::with_capacity(sources.len());
        for (source, target) in sources.iter().zip(&targets) {
            pairs.push((resolve(source)?, resolve(target)?));
        }
        // rmedge.multigraph, rmedge.digraph-direction, rmedge.absent
        for (slot, edge) in remove.iter_mut().zip(&graph.edges) {
            *slot = pairs.iter().any(|&(source, target)| {
                (edge.source == source && edge.target == target)
                    || (!graph.directed && edge.source == target && edge.target == source)
            });
        }
    }
    // rmedge.keeps-nodes, rmedge.weights, U-043-7
    let edges = graph
        .edges
        .into_iter()
        .zip(&remove)
        .filter(|(_, removed)| !**removed)
        .map(|(edge, _)| edge)
        .collect();
    result_object(
        NAME,
        ERRORS,
        GraphData {
            directed: graph.directed,
            node_names: graph.node_names,
            edges,
            node_count: graph.node_count,
        },
    )
}

// ---------------------------------------------------------------------------
// laplacian
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "laplacian",
    category = "graph",
    summary = "Return the sparse Laplacian matrix of an undirected simple graph.",
    keywords = "laplacian,graph,degree,adjacency,sparse,matrix",
    accel = "cpu",
    type_resolver(any_type),
    descriptor(crate::builtins::graph::domain::editing::LAPLACIAN_DESCRIPTOR),
    builtin_path = "crate::builtins::graph"
)]
pub(super) async fn laplacian_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    const NAME: &str = "laplacian";
    const ERRORS: &[BuiltinErrorDescriptor] = LAPLACIAN_ERRORS;
    let args = gather_values(args).await?;
    check_output_count(NAME, ERRORS)?;
    let Some(first) = args.first() else {
        return Err(edit_error(
            NAME,
            ERRORS,
            "InvalidInput",
            "expected a graph input",
        ));
    };
    let (graph, _) = graph_input(NAME, ERRORS, first)?;
    if args.len() > 1 {
        // U-043-2
        return Err(edit_error(
            NAME,
            ERRORS,
            "UnsupportedOption",
            "only laplacian(G) is specified",
        ));
    }
    if graph.directed {
        // laplacian.error-digraph
        return Err(edit_error(
            NAME,
            ERRORS,
            "OnlyGraphSupported",
            "the Laplacian is defined only for undirected graphs",
        ));
    }
    if graph.edges.iter().any(|edge| edge.source == edge.target) {
        // laplacian.error-self-loop (checked first, U-043-8)
        return Err(edit_error(
            NAME,
            ERRORS,
            "SelfLoops",
            "the Laplacian is not defined for graphs with self-loops",
        ));
    }
    let n = graph.node_count;
    let mut neighbours: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut seen = std::collections::HashSet::with_capacity(graph.edges.len());
    for edge in &graph.edges {
        let pair = (edge.source.min(edge.target), edge.source.max(edge.target));
        if !seen.insert(pair) {
            // laplacian.error-multigraph
            return Err(edit_error(
                NAME,
                ERRORS,
                "Multigraph",
                "the Laplacian is not defined for graphs with parallel edges",
            ));
        }
        neighbours[edge.source].push(edge.target);
        neighbours[edge.target].push(edge.source);
    }
    // laplacian.values / laplacian.weights-ignored: L = D - A with the
    // unweighted adjacency; isolated nodes give empty columns.
    let mut col_ptrs = Vec::with_capacity(n + 1);
    let mut row_indices = Vec::new();
    let mut values = Vec::new();
    col_ptrs.push(0);
    for (column, adjacent) in neighbours.iter().enumerate() {
        let mut entries: Vec<(usize, f64)> = adjacent.iter().map(|&row| (row, -1.0)).collect();
        if !adjacent.is_empty() {
            entries.push((column, adjacent.len() as f64));
        }
        entries.sort_by_key(|&(row, _)| row);
        for (row, value) in entries {
            row_indices.push(row);
            values.push(value);
        }
        col_ptrs.push(row_indices.len());
    }
    // laplacian.output-class, laplacian.empty
    let matrix = SparseTensor::new(n, n, col_ptrs, row_indices, values)
        .map_err(|err| edit_error(NAME, ERRORS, "Internal", err))?;
    Ok(Value::SparseTensor(matrix))
}

#[cfg(test)]
mod graph_editing_tests {
    use super::*;
    use futures::executor::block_on;
    use runmat_value::CharArray;

    // ---- helpers ----------------------------------------------------------

    fn row(values: &[f64]) -> Value {
        Value::Tensor(Tensor::new(values.to_vec(), vec![1, values.len()]).unwrap())
    }

    fn empty() -> Value {
        Value::Tensor(Tensor::new(Vec::new(), vec![0, 0]).unwrap())
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

    fn chr(text: &str) -> Value {
        Value::CharArray(CharArray::new_row(text))
    }

    fn num(value: f64) -> Value {
        Value::Num(value)
    }

    fn call(name: &str, args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async(name, args))
    }

    fn graph(s: &[f64], t: &[f64]) -> Value {
        call("graph", &[row(s), row(t)]).expect("graph")
    }

    fn graph_w(s: &[f64], t: &[f64], w: &[f64]) -> Value {
        call("graph", &[row(s), row(t), row(w)]).expect("weighted graph")
    }

    fn digraph(s: &[f64], t: &[f64]) -> Value {
        call("digraph", &[row(s), row(t)]).expect("digraph")
    }

    fn named_graph(s: &[&str], t: &[&str]) -> Value {
        call("graph", &[names(s), names(t)]).expect("named graph")
    }

    /// `n` nodes with one undirected edge 1-2 (adjacency form). Stands in for
    /// the observed `graph(1, 2, 1, n)` (D-043-2).
    fn edge_plus_isolated(n: usize) -> Value {
        let mut data = vec![0.0; n * n];
        data[1] = 1.0;
        data[n] = 1.0;
        call("graph", &[matrix(n, n, data)]).expect("graph")
    }

    fn path4() -> Value {
        graph(&[1.0, 2.0, 3.0], &[2.0, 3.0, 4.0])
    }

    fn identifier(err: RuntimeError) -> String {
        err.identifier().unwrap_or("<none>").to_string()
    }

    fn err_id(name: &str, args: &[Value]) -> String {
        identifier(call(name, args).expect_err("expected an error"))
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

    /// Numeric `G.Edges.EndNodes` rows.
    fn ends(g: &Value) -> Vec<[f64; 2]> {
        let vars = edge_vars(g);
        let Some(Value::Tensor(t)) = vars.fields.get("EndNodes") else {
            panic!(
                "expected numeric EndNodes, got {:?}",
                vars.fields.get("EndNodes")
            );
        };
        assert_eq!(t.cols, 2, "EndNodes is m-by-2");
        let d = t.materialize_f64();
        (0..t.rows).map(|r| [d[r], d[r + t.rows]]).collect()
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

    fn weights(g: &Value) -> Option<Vec<f64>> {
        match edge_vars(g).fields.get("Weight") {
            Some(Value::Tensor(t)) => Some(t.materialize_f64()),
            Some(Value::Num(n)) => Some(vec![*n]),
            None => None,
            Some(other) => panic!("unexpected Weight {other:?}"),
        }
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

    fn pairs(rows: &[[f64; 2]]) -> Vec<[f64; 2]> {
        rows.to_vec()
    }

    fn unordered(rows: &[[String; 2]]) -> Vec<[String; 2]> {
        let mut out: Vec<[String; 2]> = rows
            .iter()
            .map(|[a, b]| {
                if a <= b {
                    [a.clone(), b.clone()]
                } else {
                    [b.clone(), a.clone()]
                }
            })
            .collect();
        out.sort();
        out
    }

    fn s2(a: &str, b: &str) -> [String; 2] {
        [a.to_string(), b.to_string()]
    }

    fn dense(value: &Value) -> (Vec<usize>, Vec<f64>) {
        let Value::SparseTensor(matrix) = value else {
            panic!("expected sparse result, got {value:?}");
        };
        let full = matrix.to_dense().unwrap();
        (full.shape.clone(), full.materialize_f64())
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

    // ---- rmnode: observed (normative) -------------------------------------

    /// rmnode.signature-primary, rmnode.removes, rmnode.class (cases `one`,
    /// `several`).
    #[test]
    fn observed_rmnode_removes_and_renumbers() {
        let g = path4();
        let h = call("rmnode", &[g.clone(), num(2.0)]).unwrap();
        assert_eq!(class(&h), "graph");
        assert_eq!(counts(&h), (3, 1));
        assert_eq!(ends(&h), pairs(&[[2.0, 3.0]]));
        let h = call("rmnode", &[g.clone(), row(&[1.0, 4.0])]).unwrap();
        assert_eq!(counts(&h), (2, 1));
        assert_eq!(ends(&h), pairs(&[[1.0, 2.0]]));
        // input value unchanged (FR-043-19)
        assert_eq!(counts(&g), (4, 3));
    }

    /// rmnode.class (case `digraph`).
    #[test]
    fn observed_rmnode_digraph() {
        let h = call(
            "rmnode",
            &[digraph(&[1.0, 2.0, 3.0], &[2.0, 3.0, 1.0]), num(1.0)],
        )
        .unwrap();
        assert_eq!(class(&h), "digraph");
        assert_eq!(counts(&h), (2, 1));
        assert_eq!(ends(&h), pairs(&[[1.0, 2.0]]));
    }

    /// rmnode.named (case `named`). Endpoint order within the edge: D-043-1.
    #[test]
    fn observed_rmnode_named() {
        let g = named_graph(&["a", "b", "c"], &["b", "c", "a"]);
        let h = call("rmnode", &[g, chr("b")]).unwrap();
        assert_eq!(class(&h), "graph");
        assert_eq!(counts(&h), (2, 1));
        assert_eq!(node_names(&h), Some(vec!["a".into(), "c".into()]));
        assert_eq!(unordered(&named_ends(&h)), vec![s2("a", "c")]);
    }

    /// rmnode.weights (case `weighted`).
    #[test]
    fn observed_rmnode_weights() {
        let g = graph_w(&[1.0, 2.0, 3.0], &[2.0, 3.0, 4.0], &[10.0, 20.0, 30.0]);
        let h = call("rmnode", &[g, num(3.0)]).unwrap();
        assert_eq!(counts(&h), (3, 1));
        assert_eq!(ends(&h), pairs(&[[1.0, 2.0]]));
        assert_eq!(weights(&h), Some(vec![10.0]));
    }

    /// rmnode.isolated, rmnode.weights (case `isolated`; constructor form and
    /// unit-weight representation: D-043-2).
    #[test]
    fn observed_rmnode_isolated() {
        let h = call("rmnode", &[edge_plus_isolated(3), num(3.0)]).unwrap();
        assert_eq!(counts(&h), (2, 1));
        assert_eq!(ends(&h), pairs(&[[1.0, 2.0]]));
        assert!(weights(&h).is_none_or(|w| w == vec![1.0]));
    }

    /// rmnode.empty-list, rmnode.all (cases `empty-list`, `all`).
    #[test]
    fn observed_rmnode_empty_list_and_all() {
        let h = call("rmnode", &[path4(), empty()]).unwrap();
        assert_eq!(counts(&h), (4, 3));
        assert_eq!(ends(&h), pairs(&[[1.0, 2.0], [2.0, 3.0], [3.0, 4.0]]));
        let h = call("rmnode", &[path4(), row(&[1.0, 2.0, 3.0, 4.0])]).unwrap();
        assert_eq!(class(&h), "graph");
        assert_eq!(counts(&h), (0, 0));
        assert_eq!(ends(&h), Vec::<[f64; 2]>::new());
    }

    // ---- rmnode: documented choices ---------------------------------------

    /// FR-043-06 (rmnode.q-ignored-ids; non-normative rmnode.ignored-ids,
    /// cases `out-of-range`, `unknown-name`): ignored without error.
    #[test]
    fn unresolved_choice_rmnode_ignores_unknown_ids() {
        let h = call("rmnode", &[path4(), num(7.0)]).unwrap();
        assert_eq!(counts(&h), (4, 3));
        assert_eq!(ends(&h), pairs(&[[1.0, 2.0], [2.0, 3.0], [3.0, 4.0]]));
        let g = named_graph(&["a", "b"], &["b", "a"]);
        let h = call("rmnode", &[g.clone(), chr("z")]).unwrap();
        assert_eq!(counts(&h), (2, 2));
        assert_eq!(node_names(&h), Some(vec!["a".into(), "b".into()]));
        assert_eq!(named_ends(&h), named_ends(&g));
        // mixed known and unknown
        let h = call("rmnode", &[path4(), row(&[9.0, 1.0])]).unwrap();
        assert_eq!(counts(&h), (3, 2));
        // named graph, removal by index (FR-043-03 with U-043-4)
        let g = named_graph(&["a", "b", "c"], &["b", "c", "a"]);
        let h = call("rmnode", &[g, num(2.0)]).unwrap();
        assert_eq!(node_names(&h), Some(vec!["a".into(), "c".into()]));
    }

    /// U-043-1, U-043-2, U-043-4.
    #[test]
    fn unresolved_choice_rmnode_invalid_inputs() {
        assert_eq!(
            err_id("rmnode", &[path4(), num(0.0)]),
            "RunMat:rmnode:InvalidInput"
        );
        assert_eq!(
            err_id("rmnode", &[path4(), num(1.5)]),
            "RunMat:rmnode:InvalidInput"
        );
        assert_eq!(
            err_id("rmnode", &[path4(), chr("a")]),
            "RunMat:rmnode:UnsupportedOption"
        );
        assert_eq!(
            err_id("rmnode", &[path4(), num(1.0), num(2.0)]),
            "RunMat:rmnode:UnsupportedOption"
        );
        assert_eq!(err_id("rmnode", &[path4()]), "RunMat:rmnode:InvalidInput");
        assert_eq!(
            err_id("rmnode", &[matrix(2, 2, vec![0.0; 4]), num(1.0)]),
            "RunMat:rmnode:UndefinedFunction"
        );
    }

    // ---- addedge: observed (normative) ------------------------------------

    /// addedge.signature-primary, addedge.edge-order, addedge.duplicates
    /// (cases `one`, `order`).
    #[test]
    fn observed_addedge_sorted_edge_list() {
        let path3 = graph(&[1.0, 2.0], &[2.0, 3.0]);
        let h = call("addedge", &[path3.clone(), num(1.0), num(3.0)]).unwrap();
        assert_eq!(class(&h), "graph");
        assert_eq!(counts(&h), (3, 3));
        assert_eq!(ends(&h), pairs(&[[1.0, 2.0], [1.0, 3.0], [2.0, 3.0]]));
        let h = call("addedge", &[path3, row(&[3.0, 1.0]), row(&[1.0, 2.0])]).unwrap();
        assert_eq!(counts(&h), (3, 4));
        assert_eq!(
            ends(&h),
            pairs(&[[1.0, 2.0], [1.0, 2.0], [1.0, 3.0], [2.0, 3.0]])
        );
    }

    /// addedge.digraph-order (case `digraph`).
    #[test]
    fn observed_addedge_digraph_order() {
        let h = call(
            "addedge",
            &[digraph(&[1.0], &[2.0]), row(&[3.0, 2.0]), row(&[1.0, 1.0])],
        )
        .unwrap();
        assert_eq!(class(&h), "digraph");
        assert_eq!(counts(&h), (3, 3));
        assert_eq!(ends(&h), pairs(&[[1.0, 2.0], [2.0, 1.0], [3.0, 1.0]]));
    }

    /// addedge.new-nodes (cases `new-node`, `empty-graph`).
    #[test]
    fn observed_addedge_new_nodes() {
        let h = call("addedge", &[graph(&[1.0], &[2.0]), num(2.0), num(5.0)]).unwrap();
        assert_eq!(counts(&h), (5, 2));
        assert_eq!(ends(&h), pairs(&[[1.0, 2.0], [2.0, 5.0]]));
        let h = call(
            "addedge",
            &[call("graph", &[]).unwrap(), num(1.0), num(2.0)],
        )
        .unwrap();
        assert_eq!(class(&h), "graph");
        assert_eq!(counts(&h), (2, 1));
        assert_eq!(ends(&h), pairs(&[[1.0, 2.0]]));
    }

    /// addedge.weights (cases `weighted`, `weight-scalar`).
    #[test]
    fn observed_addedge_weights() {
        let g = call("graph", &[num(1.0), num(2.0), num(7.0)]).unwrap();
        let h = call(
            "addedge",
            &[
                g.clone(),
                row(&[2.0, 1.0]),
                row(&[3.0, 3.0]),
                row(&[8.0, 9.0]),
            ],
        )
        .unwrap();
        assert_eq!(counts(&h), (3, 3));
        assert_eq!(ends(&h), pairs(&[[1.0, 2.0], [1.0, 3.0], [2.0, 3.0]]));
        assert_eq!(weights(&h), Some(vec![7.0, 9.0, 8.0]));
        let h = call(
            "addedge",
            &[g, row(&[2.0, 3.0]), row(&[3.0, 4.0]), num(5.0)],
        )
        .unwrap();
        assert_eq!(counts(&h), (4, 3));
        assert_eq!(ends(&h), pairs(&[[1.0, 2.0], [2.0, 3.0], [3.0, 4.0]]));
        assert_eq!(weights(&h), Some(vec![7.0, 5.0, 5.0]));
    }

    /// addedge.named (case `named`).
    #[test]
    fn observed_addedge_named() {
        let g = named_graph(&["a"], &["b"]);
        let h = call("addedge", &[g, chr("b"), chr("c")]).unwrap();
        assert_eq!(counts(&h), (3, 2));
        assert_eq!(
            node_names(&h),
            Some(vec!["a".into(), "b".into(), "c".into()])
        );
        assert_eq!(named_ends(&h), vec![s2("a", "b"), s2("b", "c")]);
    }

    /// addedge.self-loop (case `self-loop`).
    #[test]
    fn observed_addedge_self_loop() {
        let h = call("addedge", &[graph(&[1.0], &[2.0]), num(2.0), num(2.0)]).unwrap();
        assert_eq!(counts(&h), (2, 2));
        assert_eq!(ends(&h), pairs(&[[1.0, 2.0], [2.0, 2.0]]));
    }

    /// addedge.scalar-expansion (case `scalar-expansion`).
    #[test]
    fn observed_addedge_scalar_expansion() {
        let h = call(
            "addedge",
            &[graph(&[1.0], &[2.0]), row(&[1.0, 2.0]), num(3.0)],
        )
        .unwrap();
        assert_eq!(counts(&h), (3, 3));
        assert_eq!(ends(&h), pairs(&[[1.0, 2.0], [1.0, 3.0], [2.0, 3.0]]));
    }

    /// addedge.error-weight-unweighted (case `err-weight-unweighted`).
    #[test]
    fn rejects_addedge_weight_on_unweighted_graph() {
        assert_eq!(
            err_id(
                "addedge",
                &[graph(&[1.0], &[2.0]), num(2.0), num(3.0), num(4.0)]
            ),
            "RunMat:addedge:NoWeights"
        );
    }

    /// addedge.error-length (case `err-length`).
    #[test]
    fn rejects_addedge_length_mismatch() {
        assert_eq!(
            err_id(
                "addedge",
                &[
                    graph(&[1.0], &[2.0]),
                    row(&[1.0, 2.0]),
                    row(&[3.0, 4.0, 5.0])
                ]
            ),
            "RunMat:addedge:EqualNumel"
        );
    }

    // ---- addedge: documented choices --------------------------------------

    /// U-043-5: scalar s, weighted graph without w, weight length, unsorted
    /// input graph is sorted by addedge (FR-043-07).
    #[test]
    fn unresolved_choice_addedge_expansion_and_weights() {
        let h = call(
            "addedge",
            &[graph(&[1.0], &[2.0]), num(3.0), row(&[1.0, 2.0])],
        )
        .unwrap();
        assert_eq!(ends(&h), pairs(&[[1.0, 2.0], [1.0, 3.0], [2.0, 3.0]]));
        let g = call("graph", &[num(1.0), num(2.0), num(7.0)]).unwrap();
        let h = call("addedge", &[g.clone(), num(2.0), num(3.0)]).unwrap();
        assert_eq!(weights(&h), Some(vec![7.0, 1.0]));
        assert_eq!(
            err_id(
                "addedge",
                &[
                    g.clone(),
                    row(&[2.0, 3.0]),
                    row(&[3.0, 4.0]),
                    row(&[1.0, 2.0, 3.0])
                ]
            ),
            "RunMat:addedge:EqualNumel"
        );
        assert_eq!(
            err_id("addedge", &[g, num(2.0), num(3.0), num(-1.0)]),
            "RunMat:addedge:InvalidInput"
        );
        let h = call("addedge", &[graph(&[3.0], &[1.0]), num(2.0), num(1.0)]).unwrap();
        assert_eq!(ends(&h), pairs(&[[1.0, 2.0], [1.0, 3.0]]));
    }

    /// U-043-5: names on unnamed graphs, indices on named graphs.
    #[test]
    fn unresolved_choice_addedge_names_and_indices() {
        let h = call(
            "addedge",
            &[call("graph", &[]).unwrap(), chr("x"), chr("y")],
        )
        .unwrap();
        assert_eq!(node_names(&h), Some(vec!["x".into(), "y".into()]));
        assert_eq!(named_ends(&h), vec![s2("x", "y")]);
        assert_eq!(
            err_id("addedge", &[graph(&[1.0], &[2.0]), chr("a"), chr("b")]),
            "RunMat:addedge:UnsupportedOption"
        );
        let named = named_graph(&["a"], &["b"]);
        let h = call("addedge", &[named.clone(), num(2.0), num(1.0)]).unwrap();
        assert_eq!(named_ends(&h), vec![s2("a", "b"), s2("a", "b")]);
        assert_eq!(
            err_id("addedge", &[named, num(1.0), num(3.0)]),
            "RunMat:addedge:UnsupportedOption"
        );
        assert_eq!(
            err_id("addedge", &[graph(&[1.0], &[2.0]), num(0.0), num(1.0)]),
            "RunMat:addedge:InvalidInput"
        );
    }

    /// U-043-1, U-043-2.
    #[test]
    fn unresolved_choice_addedge_arity_and_forms() {
        let g = graph(&[1.0], &[2.0]);
        assert_eq!(
            err_id("addedge", &[g.clone(), num(1.0)]),
            "RunMat:addedge:InvalidInput"
        );
        assert_eq!(
            err_id(
                "addedge",
                &[g.clone(), num(1.0), num(2.0), num(3.0), num(4.0)]
            ),
            "RunMat:addedge:UnsupportedOption"
        );
        let weighted = call("graph", &[num(1.0), num(2.0), num(7.0)]).unwrap();
        assert_eq!(
            err_id("addedge", &[weighted, num(1.0), num(2.0), chr("Weight")]),
            "RunMat:addedge:UnsupportedOption"
        );
        assert_eq!(
            err_id("addedge", &[num(1.0), num(1.0), num(2.0)]),
            "RunMat:addedge:UndefinedFunction"
        );
    }

    // ---- rmedge: observed (normative) -------------------------------------

    /// rmedge.signature-primary, rmedge.by-endpoints, rmedge.keeps-nodes
    /// (cases `endpoints`, `reversed`, `keeps-nodes`).
    #[test]
    fn observed_rmedge_by_endpoints() {
        for (s, t) in [(2.0, 3.0), (3.0, 2.0)] {
            let h = call("rmedge", &[path4(), num(s), num(t)]).unwrap();
            assert_eq!(class(&h), "graph");
            assert_eq!(counts(&h), (4, 2));
            assert_eq!(ends(&h), pairs(&[[1.0, 2.0], [3.0, 4.0]]));
        }
        let h = call("rmedge", &[graph(&[1.0], &[2.0]), num(1.0), num(2.0)]).unwrap();
        assert_eq!(counts(&h), (2, 0));
        assert_eq!(ends(&h), Vec::<[f64; 2]>::new());
    }

    /// rmedge.by-index (case `index`).
    #[test]
    fn observed_rmedge_by_index() {
        let h = call("rmedge", &[path4(), row(&[1.0, 3.0])]).unwrap();
        assert_eq!(counts(&h), (4, 1));
        assert_eq!(ends(&h), pairs(&[[2.0, 3.0]]));
    }

    /// rmedge.multigraph (case `multigraph`).
    #[test]
    fn observed_rmedge_multigraph() {
        let g = graph(&[1.0, 1.0, 2.0], &[2.0, 2.0, 3.0]);
        let h = call("rmedge", &[g, num(1.0), num(2.0)]).unwrap();
        assert_eq!(counts(&h), (3, 1));
        assert_eq!(ends(&h), pairs(&[[2.0, 3.0]]));
    }

    /// rmedge.digraph-direction, rmedge.absent (cases `digraph-direction`,
    /// `digraph`).
    #[test]
    fn observed_rmedge_digraph_direction() {
        let g = digraph(&[1.0, 2.0], &[2.0, 3.0]);
        let h = call("rmedge", &[g.clone(), num(2.0), num(1.0)]).unwrap();
        assert_eq!(class(&h), "digraph");
        assert_eq!(counts(&h), (3, 2));
        assert_eq!(ends(&h), pairs(&[[1.0, 2.0], [2.0, 3.0]]));
        let h = call("rmedge", &[g, num(1.0), num(2.0)]).unwrap();
        assert_eq!(class(&h), "digraph");
        assert_eq!(counts(&h), (3, 1));
        assert_eq!(ends(&h), pairs(&[[2.0, 3.0]]));
    }

    /// rmedge.absent (case `absent`).
    #[test]
    fn observed_rmedge_absent() {
        let h = call("rmedge", &[path4(), num(1.0), num(4.0)]).unwrap();
        assert_eq!(counts(&h), (4, 3));
        assert_eq!(ends(&h), pairs(&[[1.0, 2.0], [2.0, 3.0], [3.0, 4.0]]));
    }

    /// rmedge.weights (case `weighted`).
    #[test]
    fn observed_rmedge_weights() {
        let g = graph_w(&[1.0, 2.0], &[2.0, 3.0], &[5.0, 6.0]);
        let h = call("rmedge", &[g, num(1.0), num(2.0)]).unwrap();
        assert_eq!(counts(&h), (3, 1));
        assert_eq!(ends(&h), pairs(&[[2.0, 3.0]]));
        assert_eq!(weights(&h), Some(vec![6.0]));
    }

    /// rmedge.error-index-range (case `err-index-range`).
    #[test]
    fn rejects_rmedge_edge_index_out_of_range() {
        assert_eq!(
            err_id("rmedge", &[path4(), num(9.0)]),
            "RunMat:rmedge:EdgeBounds"
        );
    }

    /// rmedge.error-node-range (case `err-node-range`).
    #[test]
    fn rejects_rmedge_node_out_of_range() {
        assert_eq!(
            err_id("rmedge", &[path4(), num(1.0), num(9.0)]),
            "RunMat:rmedge:InvalidNodeID"
        );
    }

    // ---- rmedge: documented choices ---------------------------------------

    /// U-043-6.
    #[test]
    fn unresolved_choice_rmedge_names_and_invalid_ids() {
        let g = named_graph(&["a", "b"], &["b", "c"]);
        let h = call("rmedge", &[g.clone(), chr("c"), chr("b")]).unwrap();
        assert_eq!(counts(&h), (3, 1));
        assert_eq!(named_ends(&h), vec![s2("a", "b")]);
        assert_eq!(
            err_id("rmedge", &[g, chr("a"), chr("z")]),
            "RunMat:rmedge:InvalidNodeID"
        );
        assert_eq!(
            err_id("rmedge", &[path4(), chr("a"), chr("b")]),
            "RunMat:rmedge:InvalidNodeID"
        );
        assert_eq!(
            err_id("rmedge", &[path4(), num(0.0), num(1.0)]),
            "RunMat:rmedge:InvalidNodeID"
        );
        assert_eq!(
            err_id("rmedge", &[path4(), num(0.0)]),
            "RunMat:rmedge:EdgeBounds"
        );
        assert_eq!(
            err_id("rmedge", &[path4(), num(1.5)]),
            "RunMat:rmedge:EdgeBounds"
        );
        assert_eq!(
            err_id("rmedge", &[path4(), chr("a")]),
            "RunMat:rmedge:InvalidInput"
        );
    }

    /// U-043-6 expansion, U-043-1, U-043-2.
    #[test]
    fn unresolved_choice_rmedge_expansion_and_arity() {
        let star = graph(&[1.0, 1.0, 1.0], &[2.0, 3.0, 4.0]);
        let h = call("rmedge", &[star.clone(), num(1.0), row(&[2.0, 4.0])]).unwrap();
        assert_eq!(ends(&h), pairs(&[[1.0, 3.0]]));
        assert_eq!(
            err_id(
                "rmedge",
                &[star.clone(), row(&[1.0, 1.0]), row(&[2.0, 3.0, 4.0])]
            ),
            "RunMat:rmedge:InvalidInput"
        );
        let h = call("rmedge", &[star.clone(), empty()]).unwrap();
        assert_eq!(counts(&h), (4, 3));
        assert_eq!(
            err_id("rmedge", std::slice::from_ref(&star)),
            "RunMat:rmedge:InvalidInput"
        );
        assert_eq!(
            err_id("rmedge", &[star, num(1.0), num(2.0), num(3.0)]),
            "RunMat:rmedge:UnsupportedOption"
        );
        assert_eq!(
            err_id("rmedge", &[num(1.0), num(1.0)]),
            "RunMat:rmedge:UndefinedFunction"
        );
    }

    // ---- laplacian: observed (normative) ----------------------------------

    /// laplacian.signature-primary, laplacian.output-class, laplacian.values
    /// (cases `path`, `class`).
    #[test]
    fn observed_laplacian_path_matches_export() {
        let l = call("laplacian", &[graph(&[1.0, 2.0], &[2.0, 3.0])]).unwrap();
        // sparse([1;2;1;2;3;2;3], [1;1;2;2;2;3;3], [1;-1;-1;2;-1;-1;1], 3, 3)
        let expected = sparse(
            3,
            3,
            vec![0, 2, 5, 7],
            vec![0, 1, 0, 1, 2, 1, 2],
            vec![1.0, -1.0, -1.0, 2.0, -1.0, -1.0, 1.0],
        );
        assert_eq!(l, expected);
        assert_eq!(
            dense(&l),
            (
                vec![3, 3],
                vec![1.0, -1.0, 0.0, -1.0, 2.0, -1.0, 0.0, -1.0, 1.0]
            )
        );
    }

    /// laplacian.values (cases `triangle`, `isolated`).
    #[test]
    fn observed_laplacian_triangle_and_isolated() {
        let l = call("laplacian", &[graph(&[1.0, 2.0, 3.0], &[2.0, 3.0, 1.0])]).unwrap();
        assert_eq!(
            dense(&l),
            (
                vec![3, 3],
                vec![2.0, -1.0, -1.0, -1.0, 2.0, -1.0, -1.0, -1.0, 2.0]
            )
        );
        let l = call("laplacian", &[edge_plus_isolated(3)]).unwrap();
        assert_eq!(
            dense(&l),
            (
                vec![3, 3],
                vec![1.0, -1.0, 0.0, -1.0, 1.0, 0.0, 0.0, 0.0, 0.0]
            )
        );
    }

    /// laplacian.weights-ignored (case `weighted`).
    #[test]
    fn observed_laplacian_weights_ignored() {
        let l = call(
            "laplacian",
            &[graph_w(&[1.0, 2.0], &[2.0, 3.0], &[5.0, 7.0])],
        )
        .unwrap();
        assert_eq!(
            dense(&l),
            (
                vec![3, 3],
                vec![1.0, -1.0, 0.0, -1.0, 2.0, -1.0, 0.0, -1.0, 1.0]
            )
        );
    }

    /// laplacian.empty, laplacian.output-class (case `empty`).
    #[test]
    fn observed_laplacian_empty() {
        let l = call("laplacian", &[call("graph", &[]).unwrap()]).unwrap();
        assert_eq!(l, sparse(0, 0, vec![0], vec![], vec![]));
    }

    /// laplacian.error-self-loop, laplacian.error-multigraph,
    /// laplacian.error-digraph (cases `err-self-loop`, `err-multigraph`,
    /// `err-digraph`).
    #[test]
    fn rejects_laplacian_non_simple_and_digraph() {
        assert_eq!(
            err_id("laplacian", &[graph(&[1.0, 1.0], &[1.0, 2.0])]),
            "RunMat:laplacian:SelfLoops"
        );
        assert_eq!(
            err_id("laplacian", &[graph(&[1.0, 1.0], &[2.0, 2.0])]),
            "RunMat:laplacian:Multigraph"
        );
        // reversed parallel edge is still parallel in a graph
        assert_eq!(
            err_id("laplacian", &[graph(&[1.0, 2.0], &[2.0, 1.0])]),
            "RunMat:laplacian:Multigraph"
        );
        assert_eq!(
            err_id("laplacian", &[digraph(&[1.0, 2.0], &[2.0, 3.0])]),
            "RunMat:laplacian:OnlyGraphSupported"
        );
    }

    /// U-043-1 (laplacian.numeric-input, non-normative), U-043-2, U-043-8.
    #[test]
    fn unresolved_choice_laplacian_inputs() {
        let eye2 = matrix(2, 2, vec![1.0, 0.0, 0.0, 1.0]);
        assert_eq!(
            err_id("laplacian", &[eye2]),
            "RunMat:laplacian:UndefinedFunction"
        );
        assert_eq!(err_id("laplacian", &[]), "RunMat:laplacian:InvalidInput");
        let g = graph(&[1.0, 2.0], &[2.0, 3.0]);
        assert_eq!(
            err_id("laplacian", &[g, num(1.0)]),
            "RunMat:laplacian:UnsupportedOption"
        );
        // self-loop reported before parallel edges
        assert_eq!(
            err_id("laplacian", &[graph(&[1.0, 1.0, 1.0], &[1.0, 2.0, 2.0])]),
            "RunMat:laplacian:SelfLoops"
        );
        let named = named_graph(&["a", "b"], &["b", "c"]);
        let l = call("laplacian", &[named]).unwrap();
        assert_eq!(
            dense(&l).1,
            vec![1.0, -1.0, 0.0, -1.0, 2.0, -1.0, 0.0, -1.0, 1.0]
        );
    }

    // ---- common: output count (U-043-3) -----------------------------------

    /// U-043-3: called directly under `push_output_count` (dispatcher quirk).
    #[test]
    fn unresolved_choice_too_many_outputs() {
        let g = path4();
        let _guard = crate::output_count::push_output_count(Some(2));
        let err = block_on(rmnode_builtin(vec![g.clone(), num(1.0)])).unwrap_err();
        assert_eq!(identifier(err), "RunMat:rmnode:TooManyOutputs");
        let err = block_on(addedge_builtin(vec![g.clone(), num(1.0), num(3.0)])).unwrap_err();
        assert_eq!(identifier(err), "RunMat:addedge:TooManyOutputs");
        let err = block_on(rmedge_builtin(vec![g.clone(), num(1.0)])).unwrap_err();
        assert_eq!(identifier(err), "RunMat:rmedge:TooManyOutputs");
        let err = block_on(laplacian_builtin(vec![g])).unwrap_err();
        assert_eq!(identifier(err), "RunMat:laplacian:TooManyOutputs");
    }

    /// One requested output is accepted (direct call).
    #[test]
    fn unresolved_choice_single_output_direct_call() {
        let _guard = crate::output_count::push_output_count(Some(1));
        let h = block_on(rmnode_builtin(vec![path4(), num(4.0)])).unwrap();
        assert_eq!(counts(&h), (3, 2));
    }

    // ---- deviations ---------------------------------------------------------

    /// D-043-1: the observed endpoint order of rmnode cases `named` and
    /// `unknown-name` relies on the constructor canonicalising edges, which
    /// RunMat's `graph` does not do (same cause as D-036-1).
    #[test]
    #[ignore = "D-043-1: RunMat graph() does not canonicalise edge order"]
    fn deviation_rmnode_observed_endpoint_order() {
        let g = named_graph(&["a", "b", "c"], &["b", "c", "a"]);
        let h = call("rmnode", &[g, chr("b")]).unwrap();
        assert_eq!(named_ends(&h), vec![s2("a", "c")]);
        let g = named_graph(&["a", "b"], &["b", "a"]);
        let h = call("rmnode", &[g, chr("z")]).unwrap();
        assert_eq!(named_ends(&h), vec![s2("a", "b"), s2("a", "b")]);
    }
}
