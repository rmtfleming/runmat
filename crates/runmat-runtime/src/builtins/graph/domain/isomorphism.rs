//! Clean-room `isomorphism` (node permutation between isomorphic graphs).
//!
//! Clean-room feature `specs/059-graph-n3n4/`. Normative behaviour comes only
//! from the approved export `isomorphism` 0.2.0 (see that feature's
//! `provenance.md`). The search is a textbook backtracking matcher over a
//! labelled multigraph; it is independent of feature 036's unlabelled
//! `isisomorphic` search.
use super::algorithms::tensor_value;
use super::construction::graph_data;
use super::parsing::{canonical, parse_string_vector, scalar_text};
use super::*;

// ---------------------------------------------------------------------------
// Descriptors
// ---------------------------------------------------------------------------

const fn param(
    name: &'static str,
    arity: BuiltinParamArity,
    description: &'static str,
) -> BuiltinParamDescriptor {
    BuiltinParamDescriptor {
        name,
        ty: BuiltinParamType::Any,
        arity,
        default: None,
        description,
    }
}

const ISO_G1: BuiltinParamDescriptor = param(
    "G1",
    BuiltinParamArity::Required,
    "First graph or digraph object.",
);
const ISO_G2: BuiltinParamDescriptor = param(
    "G2",
    BuiltinParamArity::Required,
    "Second graph or digraph object (same class as G1).",
);
const ISO_INPUTS: [BuiltinParamDescriptor; 2] = [ISO_G1, ISO_G2];
const ISO_NV_INPUTS: [BuiltinParamDescriptor; 3] = [
    ISO_G1,
    ISO_G2,
    param(
        "Name,Value",
        BuiltinParamArity::Variadic,
        "'NodeVariables' or 'EdgeVariables' with variable names.",
    ),
];
const ISO_OUTPUT_P: BuiltinParamDescriptor = param(
    "P",
    BuiltinParamArity::Required,
    "Node permutation (double column), or empty when not isomorphic.",
);
const ISO_OUTPUTS: [BuiltinParamDescriptor; 1] = [ISO_OUTPUT_P];
const ISO_OUTPUTS_TWO: [BuiltinParamDescriptor; 2] = [
    ISO_OUTPUT_P,
    param(
        "edgeperm",
        BuiltinParamArity::Required,
        "Edge permutation (double column).",
    ),
];
const ISO_SIGNATURES: [BuiltinSignatureDescriptor; 3] = [
    BuiltinSignatureDescriptor {
        label: "P = isomorphism(G1, G2)",
        inputs: &ISO_INPUTS,
        outputs: &ISO_OUTPUTS,
    },
    BuiltinSignatureDescriptor {
        label: "[P, edgeperm] = isomorphism(G1, G2)",
        inputs: &ISO_INPUTS,
        outputs: &ISO_OUTPUTS_TWO,
    },
    BuiltinSignatureDescriptor {
        label: "P = isomorphism(G1, G2, Name, Value)",
        inputs: &ISO_NV_INPUTS,
        outputs: &ISO_OUTPUTS,
    },
];

macro_rules! iso_error {
    ($code:literal, $id:literal, $when:literal) => {
        BuiltinErrorDescriptor {
            code: concat!("RM.ISOMORPHISM.", $code),
            identifier: Some(concat!("RunMat:isomorphism:", $id)),
            when: $when,
            message: concat!("isomorphism: ", $when),
        }
    };
}

const ISO_ERRORS: &[BuiltinErrorDescriptor] = &[
    iso_error!(
        "INVALID_INPUT",
        "InvalidInput",
        "A required input is missing or an input is malformed."
    ),
    iso_error!(
        "UNSUPPORTED_OPTION",
        "UnsupportedOption",
        "The call uses a form or option that is not yet specified for RunMat."
    ),
    iso_error!(
        "INTERNAL",
        "Internal",
        "RunMat cannot build the result value."
    ),
    iso_error!(
        "UNDEFINED_FUNCTION",
        "UndefinedFunction",
        "An input is not a graph or digraph object."
    ),
    iso_error!(
        "TOO_MANY_OUTPUTS",
        "TooManyOutputs",
        "More than two outputs were requested."
    ),
    iso_error!(
        "CLASS_MISMATCH",
        "ClassMismatch",
        "One input is a graph and the other a digraph."
    ),
    iso_error!(
        "NODE_VAR_NOT_IN_FIRST_GRAPH",
        "NodeVarNotInFirstGraph",
        "A NodeVariables name is not a node variable of the first graph."
    ),
];

pub const ISOMORPHISM_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &ISO_SIGNATURES,
    output_mode: BuiltinOutputMode::ByRequestedOutputCount,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: ISO_ERRORS,
};

const NAME: &str = "isomorphism";

/// Raise `RunMat:isomorphism:<id>` from the descriptor table.
fn iso_error(id: &str, detail: impl Into<String>) -> RuntimeError {
    let suffix = format!(":{id}");
    let descriptor = ISO_ERRORS
        .iter()
        .find(|d| d.identifier.is_some_and(|ident| ident.ends_with(&suffix)))
        .cloned()
        .unwrap_or(ERROR_INTERNAL);
    error_from_descriptor(NAME, descriptor, format!("{NAME}: {}", detail.into()))
}

// ---------------------------------------------------------------------------
// Inputs and options
// ---------------------------------------------------------------------------

/// Read a graph argument; anything else is `UndefinedFunction` (U-059-1).
fn graph_input(value: &Value) -> BuiltinResult<(GraphData, &ObjectInstance)> {
    let object = match value {
        Value::Object(object)
            if object.class_name == GRAPH_CLASS || object.class_name == DIGRAPH_CLASS =>
        {
            object
        }
        _ => {
            return Err(iso_error(
                "UndefinedFunction",
                "undefined for input that is not a graph or digraph object",
            ))
        }
    };
    let graph = graph_data(value, NAME).map_err(|err| {
        iso_error(
            "InvalidInput",
            format!("invalid graph object: {}", err.message()),
        )
    })?;
    Ok((graph, object))
}

#[derive(Default)]
struct Options {
    node_vars: Vec<String>,
    edge_vars: Vec<String>,
}

/// Name-value options (isomorphism.node-variables, edge-variables;
/// U-059-4(d)).
fn parse_options(rest: &[Value]) -> BuiltinResult<Options> {
    if !rest.len().is_multiple_of(2) {
        return Err(iso_error(
            "InvalidInput",
            "name-value arguments must come in pairs",
        ));
    }
    let mut options = Options::default();
    for pair in rest.chunks(2) {
        let Some(name) = scalar_text(&pair[0]) else {
            return Err(iso_error("InvalidInput", "option names must be text"));
        };
        let vars = parse_string_vector(&pair[1]).map_err(|err| {
            iso_error(
                "InvalidInput",
                format!("'{name}' expects variable names: {err}"),
            )
        })?;
        match canonical(&name).as_str() {
            "nodevariables" => options.node_vars = vars,
            "edgevariables" => options.edge_vars = vars,
            _ => {
                return Err(iso_error(
                    "UnsupportedOption",
                    format!("option '{name}' is not supported"),
                ))
            }
        }
    }
    Ok(options)
}

// ---------------------------------------------------------------------------
// Node and edge labels (U-059-4(e)-(g))
// ---------------------------------------------------------------------------

/// One comparable value of a node/edge variable.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Atom {
    Num(u64),
    Text(String),
    /// NaN: equal to nothing (each occurrence is distinct).
    Unequal(usize),
}

#[derive(Default)]
struct Atoms {
    unequal: usize,
}

impl Atoms {
    fn num(&mut self, value: f64) -> Atom {
        if value.is_nan() {
            self.unequal += 1;
            Atom::Unequal(self.unequal)
        } else if value == 0.0 {
            Atom::Num(0.0f64.to_bits())
        } else {
            Atom::Num(value.to_bits())
        }
    }

    fn element(&mut self, value: &Value) -> Option<Atom> {
        if let Some(text) = scalar_text(value) {
            return Some(Atom::Text(text));
        }
        match value {
            Value::Num(x) => Some(self.num(*x)),
            Value::Int(i) => Some(self.num(i.to_f64())),
            Value::Bool(b) => Some(self.num(if *b { 1.0 } else { 0.0 })),
            Value::Tensor(t) if t.shape.iter().product::<usize>() == 1 => {
                Some(self.num(tensor::tensor_values_f64(t)[0]))
            }
            _ => None,
        }
    }
}

/// Split a table variable into one row of atoms per node/edge.
fn variable_rows(
    var: &str,
    value: &Value,
    rows: usize,
    atoms: &mut Atoms,
) -> BuiltinResult<Vec<Vec<Atom>>> {
    let mismatch = || {
        iso_error(
            "InvalidInput",
            format!("variable '{var}' does not have one row per node or edge"),
        )
    };
    let unsupported = || {
        iso_error(
            "UnsupportedOption",
            format!("variable '{var}' has a type that is not supported for matching"),
        )
    };
    let mut out = vec![Vec::new(); rows];
    match value {
        Value::Tensor(t) => {
            if t.rows != rows {
                return Err(mismatch());
            }
            let data = tensor::tensor_values_f64(t);
            let cols = if rows == 0 { 0 } else { data.len() / rows };
            for (r, row) in out.iter_mut().enumerate() {
                for c in 0..cols {
                    row.push(atoms.num(data[r + c * rows]));
                }
            }
        }
        Value::LogicalArray(array) => {
            if array.shape.first().copied().unwrap_or(0) != rows {
                return Err(mismatch());
            }
            let data: Vec<u8> = array.data.iter().copied().collect();
            let cols = if rows == 0 { 0 } else { data.len() / rows };
            for (r, row) in out.iter_mut().enumerate() {
                for c in 0..cols {
                    row.push(atoms.num(f64::from(data[r + c * rows] != 0)));
                }
            }
        }
        Value::StringArray(array) => {
            if array.rows != rows {
                return Err(mismatch());
            }
            for (r, row) in out.iter_mut().enumerate() {
                for c in 0..array.cols {
                    row.push(Atom::Text(array.data[r + c * rows].clone()));
                }
            }
        }
        Value::Cell(cell) => {
            if cell.rows != rows {
                return Err(mismatch());
            }
            // Cell payloads are row-major within a 2-D page.
            for (r, row) in out.iter_mut().enumerate() {
                for c in 0..cell.cols {
                    let atom = atoms
                        .element(&cell.data[r * cell.cols + c])
                        .ok_or_else(unsupported)?;
                    row.push(atom);
                }
            }
        }
        Value::Num(_) | Value::Int(_) | Value::Bool(_) if rows == 1 => {
            out[0].push(atoms.element(value).ok_or_else(unsupported)?);
        }
        Value::Num(_) | Value::Int(_) | Value::Bool(_) => return Err(mismatch()),
        _ => return Err(unsupported()),
    }
    Ok(out)
}

fn table_variable(table: Option<&Value>, var: &str) -> BuiltinResult<Option<Value>> {
    let Some(Value::Object(table)) = table else {
        return Ok(None);
    };
    let vars = table_variables(table).map_err(|err| {
        iso_error(
            "InvalidInput",
            format!("invalid graph table: {}", err.message()),
        )
    })?;
    Ok(vars.fields.get(var).cloned())
}

/// A node variable of a graph object (U-059-4(f): the synthetic `Index`
/// column of unnamed graphs is not one).
fn node_variable(
    object: &ObjectInstance,
    graph: &GraphData,
    var: &str,
) -> BuiltinResult<Option<Value>> {
    if var == "Index" && graph.node_names.is_none() {
        return Ok(None);
    }
    table_variable(object.properties.get("Nodes"), var)
}

/// Rows of an edge variable (U-059-4(f): `Weight` always reads the stored
/// weights; `EndNodes` is not an edge attribute).
fn edge_variable_rows(
    object: &ObjectInstance,
    graph: &GraphData,
    var: &str,
    atoms: &mut Atoms,
) -> BuiltinResult<Option<Vec<Vec<Atom>>>> {
    if var == "EndNodes" {
        return Err(iso_error(
            "InvalidInput",
            "'EndNodes' cannot be used as an edge variable",
        ));
    }
    if var == "Weight" {
        return Ok(Some(
            graph
                .edges
                .iter()
                .map(|edge| vec![atoms.num(edge.weight)])
                .collect(),
        ));
    }
    match table_variable(object.properties.get("Edges"), var)? {
        Some(value) => Ok(Some(variable_rows(var, &value, graph.edges.len(), atoms)?)),
        None => Ok(None),
    }
}

struct Labels {
    nodes: [Vec<u32>; 2],
    edges: [Vec<u32>; 2],
}

/// Combined node and edge labels, interned across both graphs.
fn resolve_labels(
    graphs: [(&GraphData, &ObjectInstance); 2],
    options: &Options,
) -> BuiltinResult<Labels> {
    let mut atoms = Atoms::default();
    let mut node_rows: [Vec<Vec<Atom>>; 2] = [
        vec![Vec::new(); graphs[0].0.node_count],
        vec![Vec::new(); graphs[1].0.node_count],
    ];
    for var in &options.node_vars {
        for (which, (graph, object)) in graphs.iter().enumerate() {
            let Some(value) = node_variable(object, graph, var)? else {
                return Err(if which == 0 {
                    // isomorphism.error-unknown-nodevar
                    iso_error(
                        "NodeVarNotInFirstGraph",
                        format!("'{var}' is not a node variable of the first graph"),
                    )
                } else {
                    iso_error(
                        "InvalidInput",
                        format!("'{var}' is not a node variable of the second graph"),
                    )
                });
            };
            let rows = variable_rows(var, &value, graph.node_count, &mut atoms)?;
            for (target, row) in node_rows[which].iter_mut().zip(rows) {
                target.extend(row);
            }
        }
    }
    let mut edge_rows: [Vec<Vec<Atom>>; 2] = [
        vec![Vec::new(); graphs[0].0.edges.len()],
        vec![Vec::new(); graphs[1].0.edges.len()],
    ];
    for var in &options.edge_vars {
        for (which, (graph, object)) in graphs.iter().enumerate() {
            let Some(rows) = edge_variable_rows(object, graph, var, &mut atoms)? else {
                let position = if which == 0 { "first" } else { "second" };
                return Err(iso_error(
                    "InvalidInput",
                    format!("'{var}' is not an edge variable of the {position} graph"),
                ));
            };
            for (target, row) in edge_rows[which].iter_mut().zip(rows) {
                target.extend(row);
            }
        }
    }
    let mut table: HashMap<Vec<Atom>, u32> = HashMap::new();
    let mut intern = |rows: Vec<Vec<Atom>>| -> Vec<u32> {
        rows.into_iter()
            .map(|row| {
                let next = table.len() as u32;
                *table.entry(row).or_insert(next)
            })
            .collect()
    };
    let [n1, n2] = node_rows;
    let [e1, e2] = edge_rows;
    let nodes = [intern(n1), intern(n2)];
    let edges = [intern(e1), intern(e2)];
    Ok(Labels { nodes, edges })
}

// ---------------------------------------------------------------------------
// Labelled multigraph matcher
// ---------------------------------------------------------------------------

/// `(node label, out-links, in-links, sorted self-loop labels)`; links
/// exclude self-loops. Undirected edges count as both out- and in-links.
type Invariant = (u32, u32, u32, Vec<u32>);

struct Labelled {
    n: usize,
    /// `(u, v)` -> sorted labels of the edges from `u` to `v` (undirected
    /// edges are stored under both orientations, self-loops once).
    pairs: HashMap<(usize, usize), Vec<u32>>,
    neighbours: Vec<Vec<usize>>,
    invariants: Vec<Invariant>,
}

impl Labelled {
    fn new(graph: &GraphData, node_labels: &[u32], edge_labels: &[u32]) -> Self {
        let n = graph.node_count;
        let mut pairs: HashMap<(usize, usize), Vec<u32>> = HashMap::new();
        let mut out = vec![0u32; n];
        let mut inc = vec![0u32; n];
        let mut loops = vec![Vec::new(); n];
        for (edge, &label) in graph.edges.iter().zip(edge_labels) {
            let (s, t) = (edge.source, edge.target);
            pairs.entry((s, t)).or_default().push(label);
            if s == t {
                loops[s].push(label);
                continue;
            }
            out[s] += 1;
            inc[t] += 1;
            if !graph.directed {
                pairs.entry((t, s)).or_default().push(label);
                out[t] += 1;
                inc[s] += 1;
            }
        }
        let mut neighbours = vec![Vec::new(); n];
        for (&(u, v), labels) in pairs.iter_mut() {
            labels.sort_unstable();
            if u != v {
                neighbours[u].push(v);
                neighbours[v].push(u);
            }
        }
        for list in &mut neighbours {
            list.sort_unstable();
            list.dedup();
        }
        let invariants = (0..n)
            .map(|u| {
                let mut own = std::mem::take(&mut loops[u]);
                own.sort_unstable();
                (node_labels[u], out[u], inc[u], own)
            })
            .collect();
        Self {
            n,
            pairs,
            neighbours,
            invariants,
        }
    }

    fn pair(&self, u: usize, v: usize) -> &[u32] {
        self.pairs.get(&(u, v)).map_or(&[], Vec::as_slice)
    }

    fn links(&self, u: usize, v: usize) -> u32 {
        (self.pair(u, v).len() + self.pair(v, u).len()) as u32
    }
}

/// Most connected node first, then repeatedly the node with the most links
/// into the ordered prefix (ties: more links overall, then lower index).
fn search_order(graph: &Labelled) -> Vec<usize> {
    let n = graph.n;
    let total: Vec<u32> = graph
        .invariants
        .iter()
        .map(|(_, out, inc, loops)| out + inc + loops.len() as u32)
        .collect();
    let mut ordered = Vec::with_capacity(n);
    let mut placed = vec![false; n];
    let mut attached = vec![0u32; n];
    for _ in 0..n {
        let next = (0..n)
            .filter(|&u| !placed[u])
            .max_by(|&a, &b| {
                (attached[a], total[a])
                    .cmp(&(attached[b], total[b]))
                    .then(b.cmp(&a))
            })
            .expect("an unplaced node remains");
        placed[next] = true;
        ordered.push(next);
        for &u in &graph.neighbours[next] {
            attached[u] += graph.links(next, u);
        }
    }
    ordered
}

/// Node map `G1 -> G2` such that every pair `(i, j)` carries the same edge
/// label multiset in both graphs, or `None` (FR-059-02, FR-059-05).
/// Iterative backtracking; the first complete map is returned (U-059-4(a)).
fn find_map(g1: &Labelled, g2: &Labelled, edge_counts: (usize, usize)) -> Option<Vec<usize>> {
    if g1.n != g2.n || edge_counts.0 != edge_counts.1 {
        return None;
    }
    let mut sorted1 = g1.invariants.clone();
    let mut sorted2 = g2.invariants.clone();
    sorted1.sort_unstable();
    sorted2.sort_unstable();
    if sorted1 != sorted2 {
        return None;
    }
    let n = g1.n;
    let order = search_order(g1);
    let mut map = vec![usize::MAX; n];
    let mut used = vec![false; n];
    let mut cursor = vec![0usize; n + 1];
    let mut depth = 0usize;
    let feasible = |map: &[usize], used: &[bool], depth: usize, v: usize, w: usize| {
        !used[w]
            && g1.invariants[v] == g2.invariants[w]
            && order[..depth].iter().all(|&u| {
                let m = map[u];
                g1.pair(v, u) == g2.pair(w, m) && g1.pair(u, v) == g2.pair(m, w)
            })
    };
    while depth < n {
        let v = order[depth];
        let mut chosen = None;
        while cursor[depth] < n {
            let w = cursor[depth];
            cursor[depth] += 1;
            if feasible(&map, &used, depth, v, w) {
                chosen = Some(w);
                break;
            }
        }
        match chosen {
            Some(w) => {
                map[v] = w;
                used[w] = true;
                depth += 1;
                cursor[depth] = 0;
            }
            None => {
                if depth == 0 {
                    return None;
                }
                depth -= 1;
                let u = order[depth];
                used[map[u]] = false;
                map[u] = usize::MAX;
            }
        }
    }
    Some(map)
}

/// U-059-5: `edgeperm(k)` is the `G2` edge onto which `G1` edge `k` maps
/// (parallel edges paired lowest index first).
fn edge_permutation(
    g1: &GraphData,
    g2: &GraphData,
    map: &[usize],
    labels: &Labels,
) -> BuiltinResult<Vec<f64>> {
    let directed = g1.directed;
    let key = |s: usize, t: usize, label: u32| {
        if directed {
            (s, t, label)
        } else {
            (s.min(t), s.max(t), label)
        }
    };
    let mut available: HashMap<(usize, usize, u32), Vec<usize>> = HashMap::new();
    for (j, edge) in g2.edges.iter().enumerate().rev() {
        available
            .entry(key(edge.source, edge.target, labels.edges[1][j]))
            .or_default()
            .push(j);
    }
    g1.edges
        .iter()
        .enumerate()
        .map(|(k, edge)| {
            available
                .get_mut(&key(map[edge.source], map[edge.target], labels.edges[0][k]))
                .and_then(Vec::pop)
                .map(|j| (j + 1) as f64)
                .ok_or_else(|| iso_error("Internal", "edge permutation is inconsistent"))
        })
        .collect()
}

fn double(data: Vec<f64>, rows: usize, cols: usize) -> BuiltinResult<Value> {
    tensor_value(data, vec![rows, cols], NAME).map_err(|err| iso_error("Internal", err.message()))
}

// ---------------------------------------------------------------------------
// Builtin
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "isomorphism",
    category = "graph",
    summary = "Return a node permutation mapping one graph onto an isomorphic graph.",
    keywords = "isomorphism,graph,digraph,permutation,isomorphic,structure",
    accel = "cpu",
    type_resolver(any_type),
    descriptor(crate::builtins::graph::domain::isomorphism::ISOMORPHISM_DESCRIPTOR),
    builtin_path = "crate::builtins::graph"
)]
pub(super) async fn isomorphism_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    let args = gather_values(args).await?;
    let output_count = crate::output_count::current_output_count();
    if let Some(count) = output_count.filter(|&count| count > 2) {
        // U-059-3
        return Err(iso_error(
            "TooManyOutputs",
            format!("isomorphism returns at most two outputs, {count} were requested"),
        ));
    }
    if args.len() < 2 {
        // U-059-2
        return Err(iso_error("InvalidInput", "expected two graph inputs"));
    }
    let (g1, o1) = graph_input(&args[0])?;
    let (g2, o2) = graph_input(&args[1])?;
    if g1.directed != g2.directed {
        // isomorphism.error-mixed
        return Err(iso_error(
            "ClassMismatch",
            "inputs must both be graphs or both be digraphs",
        ));
    }
    let options = parse_options(&args[2..])?;
    let labels = resolve_labels([(&g1, o1), (&g2, o2)], &options)?;
    let l1 = Labelled::new(&g1, &labels.nodes[0], &labels.edges[0]);
    let l2 = Labelled::new(&g2, &labels.nodes[1], &labels.edges[1]);
    let (p, edgeperm) = match find_map(&l1, &l2, (g1.edges.len(), g2.edges.len())) {
        // isomorphism.not-isomorphic; U-059-5
        None => (double(Vec::new(), 0, 0)?, double(Vec::new(), 0, 0)?),
        // isomorphism.empty-graphs; U-059-5
        Some(_) if g1.node_count == 0 => (double(Vec::new(), 1, 0)?, double(Vec::new(), 1, 0)?),
        Some(map) => {
            // isomorphism.output-class, isomorphism.orientation
            let p: Vec<f64> = map.iter().map(|&w| (w + 1) as f64).collect();
            let ep = edge_permutation(&g1, &g2, &map, &labels)?;
            let (n, m) = (p.len(), ep.len());
            (double(p, n, 1)?, double(ep, m, 1)?)
        }
    };
    Ok(match output_count {
        None => p,
        Some(0) => Value::OutputList(Vec::new()),
        Some(1) => Value::OutputList(vec![p]),
        // isomorphism.edgeperm
        Some(_) => Value::OutputList(vec![p, edgeperm]),
    })
}

#[cfg(test)]
mod graph_n3n4_isomorphism_tests {
    use super::*;
    use futures::executor::block_on;
    use runmat_value::{CellArray, CharArray};

    // ---- helpers ----------------------------------------------------------

    fn row(values: &[f64]) -> Value {
        Value::Tensor(Tensor::new(values.to_vec(), vec![1, values.len()]).unwrap())
    }

    fn column(values: &[f64]) -> Value {
        Value::Tensor(Tensor::new(values.to_vec(), vec![values.len(), 1]).unwrap())
    }

    fn chr(text: &str) -> Value {
        Value::CharArray(CharArray::new_row(text))
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

    fn graph_w(s: &[f64], t: &[f64], w: &[f64]) -> Value {
        call("graph", &[row(s), row(t), row(w)]).expect("weighted graph")
    }

    fn digraph(s: &[f64], t: &[f64]) -> Value {
        call("digraph", &[row(s), row(t)]).expect("digraph")
    }

    fn named_graph(s: &[&str], t: &[&str]) -> Value {
        call("graph", &[names(s), names(t)]).expect("named graph")
    }

    fn empty_graph() -> Value {
        call("graph", &[]).expect("empty graph")
    }

    fn identifier(err: RuntimeError) -> String {
        err.identifier().unwrap_or("<none>").to_string()
    }

    fn err_id(args: &[Value]) -> String {
        identifier(call(NAME, args).expect_err("expected an error"))
    }

    /// `(rows, cols, values)` of a numeric result (`Num` is 1x1).
    fn numeric(value: &Value) -> (usize, usize, Vec<f64>) {
        match value {
            Value::Num(n) => (1, 1, vec![*n]),
            Value::Tensor(t) => (t.rows, t.cols, tensor::tensor_values_f64(t)),
            other => panic!("expected double, got {other:?}"),
        }
    }

    /// P = isomorphism(args...) with one output.
    /// (rows, cols, column-major data) of a dense numeric result.
    type Dense = (usize, usize, Vec<f64>);

    fn iso(args: &[Value]) -> Dense {
        numeric(&call(NAME, args).expect("isomorphism"))
    }

    /// [P, edgeperm] = isomorphism(args...), direct call under the output count.
    fn iso2(args: &[Value]) -> (Dense, Dense) {
        let _guard = crate::output_count::push_output_count(Some(2));
        let out = block_on(isomorphism_builtin(args.to_vec())).expect("isomorphism");
        let Value::OutputList(values) = out else {
            panic!("expected an output list, got {out:?}");
        };
        assert_eq!(values.len(), 2);
        (numeric(&values[0]), numeric(&values[1]))
    }

    /// Edge multiplicity matrix (undirected edges counted both ways).
    fn multiplicities(g: &Value) -> Vec<Vec<u32>> {
        let data = graph_data(g, "test").unwrap();
        let n = data.node_count;
        let mut a = vec![vec![0u32; n]; n];
        for edge in &data.edges {
            a[edge.source][edge.target] += 1;
            if !data.directed && edge.source != edge.target {
                a[edge.target][edge.source] += 1;
            }
        }
        a
    }

    fn to_perm(p: &[f64]) -> Vec<usize> {
        p.iter().map(|&x| x as usize - 1).collect()
    }

    /// isomorphism.orientation: G2 reordered by P has the adjacency of G1.
    fn reorder_second_matches_first(g1: &Value, g2: &Value, p: &[f64]) -> bool {
        let (a1, a2) = (multiplicities(g1), multiplicities(g2));
        let p = to_perm(p);
        let n = a1.len();
        a2.len() == n && (0..n).all(|i| (0..n).all(|j| a1[i][j] == a2[p[i]][p[j]]))
    }

    /// The other direction: G1 reordered by P has the adjacency of G2.
    fn reorder_first_matches_second(g1: &Value, g2: &Value, p: &[f64]) -> bool {
        reorder_second_matches_first(g2, g1, p)
    }

    fn is_permutation(p: &[f64], n: usize) -> bool {
        let mut seen = vec![false; n];
        p.len() == n
            && p.iter().all(|&x| {
                let ok = x.fract() == 0.0 && x >= 1.0 && x <= n as f64 && !seen[x as usize - 1];
                if ok {
                    seen[x as usize - 1] = true;
                }
                ok
            })
    }

    fn asym_pair() -> (Value, Value) {
        (
            graph(
                &[1.0, 2.0, 3.0, 4.0, 3.0, 5.0],
                &[2.0, 3.0, 4.0, 5.0, 5.0, 6.0],
            ),
            graph(
                &[6.0, 5.0, 4.0, 3.0, 4.0, 2.0],
                &[5.0, 4.0, 3.0, 2.0, 2.0, 1.0],
            ),
        )
    }

    fn path3() -> Value {
        graph(&[1.0, 2.0], &[2.0, 3.0])
    }

    /// Rebuild the `Nodes` table of an unnamed graph with an extra variable
    /// (D-059-3: RunMat's constructors create no custom node variables).
    fn with_node_variable(g: &Value, var: &str, values: Value) -> Value {
        let mut g = g.clone();
        let Value::Object(object) = &mut g else {
            panic!("expected graph object");
        };
        let n = graph_data(&Value::Object(object.clone()), "test")
            .unwrap()
            .node_count;
        let index = column(&(1..=n).map(|i| i as f64).collect::<Vec<_>>());
        let table = table_from_columns(
            vec!["Index".to_string(), var.to_string()],
            vec![index, values],
        )
        .unwrap();
        object.properties.insert("Nodes".to_string(), table);
        g
    }

    fn nv(name: &str, value: &str) -> [Value; 2] {
        [chr(name), chr(value)]
    }

    // ---- observed (normative) -------------------------------------------

    /// isomorphism.signature-primary, output-class, orientation (cases
    /// `asym`, `asym-swapped`): the unique permutation, a 6x1 double.
    #[test]
    fn observed_isomorphism_asymmetric_pair_matches_export() {
        let (g1, g2) = asym_pair();
        let expected = vec![6.0, 5.0, 4.0, 3.0, 2.0, 1.0];
        let (r, c, p) = iso(&[g1.clone(), g2.clone()]);
        assert_eq!((r, c), (6, 1));
        assert_eq!(p, expected);
        assert!(reorder_second_matches_first(&g1, &g2, &p));
        assert!(reorder_first_matches_second(&g1, &g2, &p));
        let (r, c, p) = iso(&[g2.clone(), g1.clone()]);
        assert_eq!((r, c), (6, 1));
        assert_eq!(p, expected);
        assert!(reorder_second_matches_first(&g2, &g1, &p));
    }

    /// isomorphism.output-class (case `identical`); which valid permutation
    /// is returned is not required (isomorphism.non-unique).
    #[test]
    fn observed_isomorphism_identical_path_is_valid_permutation() {
        let g = path3();
        let (r, c, p) = iso(&[g.clone(), g.clone()]);
        assert_eq!((r, c), (3, 1));
        assert!(is_permutation(&p, 3));
        assert!(reorder_second_matches_first(&g, &g, &p));
    }

    /// isomorphism.orientation (case `digraph`): P = [3 1 2]; reordering G2
    /// by P gives G1, reordering G1 by P does not give G2.
    #[test]
    fn observed_isomorphism_digraph_orientation() {
        let g1 = digraph(&[1.0, 2.0], &[2.0, 3.0]);
        let g2 = digraph(&[3.0, 1.0], &[1.0, 2.0]);
        let (r, c, p) = iso(&[g1.clone(), g2.clone()]);
        assert_eq!((r, c), (3, 1));
        assert_eq!(p, vec![3.0, 1.0, 2.0]);
        assert!(reorder_second_matches_first(&g1, &g2, &p));
        assert!(!reorder_first_matches_second(&g1, &g2, &p));
    }

    /// isomorphism.not-isomorphic (cases `not-iso`, `node-count`,
    /// `digraph-direction`): 0x0 double.
    #[test]
    fn observed_isomorphism_not_isomorphic_is_0x0() {
        let path4 = graph(&[1.0, 2.0, 3.0], &[2.0, 3.0, 4.0]);
        let star = graph(&[1.0, 1.0, 1.0], &[2.0, 3.0, 4.0]);
        assert_eq!(iso(&[path4, star]), (0, 0, vec![]));
        let single = call("graph", &[Value::Num(1.0), Value::Num(2.0)]).unwrap();
        assert_eq!(iso(&[single, path3()]), (0, 0, vec![]));
        let d1 = digraph(&[1.0, 2.0], &[2.0, 3.0]);
        let d2 = digraph(&[2.0, 2.0], &[1.0, 3.0]);
        assert_eq!(iso(&[d1, d2]), (0, 0, vec![]));
    }

    /// isomorphism.empty-graphs: two empty graphs give a 1x0 double.
    #[test]
    fn observed_isomorphism_empty_graphs_is_1x0() {
        assert_eq!(iso(&[empty_graph(), empty_graph()]), (1, 0, vec![]));
    }

    /// isomorphism.edgeperm (case `edgeperm`): [1..6]' as a 6x1 double.
    #[test]
    fn observed_isomorphism_edgeperm_matches_export() {
        let (g1, g2) = asym_pair();
        let ((pr, pc, p), (er, ec, ep)) = iso2(&[g1, g2]);
        assert_eq!((pr, pc), (6, 1));
        assert_eq!(p, vec![6.0, 5.0, 4.0, 3.0, 2.0, 1.0]);
        assert_eq!((er, ec), (6, 1));
        assert_eq!(ep, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    }

    /// isomorphism.node-variables (cases `nodevars-match`,
    /// `nodevars-mismatch`).
    #[test]
    fn observed_isomorphism_node_variables() {
        let g1 = with_node_variable(&path3(), "Met", column(&[10.0, 20.0, 30.0]));
        let g2 = with_node_variable(&path3(), "Met", column(&[30.0, 20.0, 10.0]));
        let mut args = vec![g1.clone(), g2];
        args.extend(nv("NodeVariables", "Met"));
        assert_eq!(iso(&args), (3, 1, vec![3.0, 2.0, 1.0]));
        let g3 = with_node_variable(&path3(), "Met", column(&[10.0, 30.0, 20.0]));
        let mut args = vec![g1, g3];
        args.extend(nv("NodeVariables", "Met"));
        assert_eq!(iso(&args), (0, 0, vec![]));
    }

    /// isomorphism.edge-variables (case `edgevars`): P = [3 2 1].
    #[test]
    fn observed_isomorphism_edge_variables() {
        let g1 = graph_w(&[1.0, 2.0], &[2.0, 3.0], &[5.0, 7.0]);
        let g2 = graph_w(&[1.0, 2.0], &[2.0, 3.0], &[7.0, 5.0]);
        let mut args = vec![g1.clone(), g2.clone()];
        args.extend(nv("EdgeVariables", "Weight"));
        let (r, c, p) = iso(&args);
        assert_eq!((r, c), (3, 1));
        assert_eq!(p, vec![3.0, 2.0, 1.0]);
        assert!(reorder_second_matches_first(&g1, &g2, &p));
        assert!(reorder_first_matches_second(&g1, &g2, &p));
    }

    // ---- errors (normative) ---------------------------------------------

    /// isomorphism.error-mixed.
    #[test]
    fn rejects_isomorphism_graph_with_digraph() {
        let g = call("graph", &[Value::Num(1.0), Value::Num(2.0)]).unwrap();
        let d = call("digraph", &[Value::Num(1.0), Value::Num(2.0)]).unwrap();
        assert_eq!(
            err_id(&[g.clone(), d.clone()]),
            "RunMat:isomorphism:ClassMismatch"
        );
        assert_eq!(err_id(&[d, g]), "RunMat:isomorphism:ClassMismatch");
    }

    /// isomorphism.error-unknown-nodevar.
    #[test]
    fn rejects_isomorphism_unknown_node_variable() {
        let g = call("graph", &[Value::Num(1.0), Value::Num(2.0)]).unwrap();
        let mut args = vec![g.clone(), g];
        args.extend(nv("NodeVariables", "nope"));
        assert_eq!(err_id(&args), "RunMat:isomorphism:NodeVarNotInFirstGraph");
    }

    // ---- documented choices (non-normative) ----------------------------

    /// U-059-1, U-059-2 (isomorphism.non-graph is non-normative).
    #[test]
    fn unresolved_choice_isomorphism_inputs() {
        let m = Value::Tensor(Tensor::new(vec![0.0, 1.0, 1.0, 0.0], vec![2, 2]).unwrap());
        assert_eq!(
            err_id(&[m.clone(), m.clone()]),
            "RunMat:isomorphism:UndefinedFunction"
        );
        assert_eq!(
            err_id(&[path3(), m]),
            "RunMat:isomorphism:UndefinedFunction"
        );
        assert_eq!(err_id(&[]), "RunMat:isomorphism:InvalidInput");
        assert_eq!(err_id(&[path3()]), "RunMat:isomorphism:InvalidInput");
    }

    /// U-059-4(d)-(h): option parsing and variable resolution.
    #[test]
    fn unresolved_choice_isomorphism_options() {
        let g = path3();
        let met = with_node_variable(&g, "Met", column(&[1.0, 2.0, 1.0]));
        let run = |extra: Vec<Value>, a: &Value, b: &Value| {
            let mut args = vec![a.clone(), b.clone()];
            args.extend(extra);
            call(NAME, &args)
        };
        // unknown option name / unpaired / non-text name
        let err = run(nv("Foo", "Met").to_vec(), &g, &g).unwrap_err();
        assert_eq!(identifier(err), "RunMat:isomorphism:UnsupportedOption");
        let err = run(vec![chr("NodeVariables")], &g, &g).unwrap_err();
        assert_eq!(identifier(err), "RunMat:isomorphism:InvalidInput");
        let err = run(vec![Value::Num(1.0), chr("Met")], &g, &g).unwrap_err();
        assert_eq!(identifier(err), "RunMat:isomorphism:InvalidInput");
        // case-insensitive name; string-array list of names
        let (r, _, p) = numeric(&run(nv("nodevariables", "Met").to_vec(), &met, &met).unwrap());
        assert_eq!(r, 3);
        assert_eq!(p[1], 2.0);
        let list = vec![chr("NodeVariables"), names(&["Met"])];
        assert_eq!(numeric(&run(list, &met, &met).unwrap()).0, 3);
        // present in G1, absent from G2
        let err = run(nv("NodeVariables", "Met").to_vec(), &met, &g).unwrap_err();
        assert_eq!(identifier(err), "RunMat:isomorphism:InvalidInput");
        // synthetic Index column is not a node variable
        let err = run(nv("NodeVariables", "Index").to_vec(), &g, &g).unwrap_err();
        assert_eq!(identifier(err), "RunMat:isomorphism:NodeVarNotInFirstGraph");
        // edge variables: unknown -> InvalidInput; EndNodes -> InvalidInput
        let err = run(nv("EdgeVariables", "Color").to_vec(), &g, &g).unwrap_err();
        assert_eq!(identifier(err), "RunMat:isomorphism:InvalidInput");
        let err = run(nv("EdgeVariables", "EndNodes").to_vec(), &g, &g).unwrap_err();
        assert_eq!(identifier(err), "RunMat:isomorphism:InvalidInput");
        // Weight on unweighted graphs reads unit weights
        assert_eq!(
            numeric(&run(nv("EdgeVariables", "Weight").to_vec(), &g, &g).unwrap()).0,
            3
        );
        let weighted = graph_w(&[1.0, 2.0], &[2.0, 3.0], &[1.0, 2.0]);
        let err_free = run(nv("EdgeVariables", "Weight").to_vec(), &g, &weighted).unwrap();
        assert_eq!(numeric(&err_free), (0, 0, vec![]));
        // NaN never equals anything
        let nan = with_node_variable(&g, "Met", column(&[f64::NAN, 2.0, 3.0]));
        let out = run(nv("NodeVariables", "Met").to_vec(), &nan, &nan).unwrap();
        assert_eq!(numeric(&out), (0, 0, vec![]));
        // text-valued node variables; Name on named graphs
        let labels = |v: &[&str]| {
            Value::Cell(CellArray::new(v.iter().map(|s| chr(s)).collect(), v.len(), 1).unwrap())
        };
        let t1 = with_node_variable(&g, "Tag", labels(&["x", "y", "z"]));
        let t2 = with_node_variable(&g, "Tag", labels(&["z", "y", "x"]));
        let out = run(nv("NodeVariables", "Tag").to_vec(), &t1, &t2).unwrap();
        assert_eq!(numeric(&out), (3, 1, vec![3.0, 2.0, 1.0]));
        let n1 = named_graph(&["a", "b"], &["b", "c"]);
        let n2 = named_graph(&["c", "b"], &["b", "a"]);
        let out = run(nv("NodeVariables", "Name").to_vec(), &n1, &n2).unwrap();
        let (_, _, p) = numeric(&out);
        let names1 = graph_data(&n1, "test").unwrap().node_names.unwrap();
        let names2 = graph_data(&n2, "test").unwrap().node_names.unwrap();
        for (i, &target) in p.iter().enumerate() {
            assert_eq!(names1[i], names2[target as usize - 1]);
        }
        // unsupported variable type (cell holding non-scalar numerics)
        let cells = (0..3).map(|_| row(&[1.0, 2.0])).collect();
        let bad = with_node_variable(&g, "S", Value::Cell(CellArray::new(cells, 3, 1).unwrap()));
        let err = run(nv("NodeVariables", "S").to_vec(), &bad, &bad).unwrap_err();
        assert_eq!(identifier(err), "RunMat:isomorphism:UnsupportedOption");
    }

    /// U-059-4(b): weights are ignored unless named in EdgeVariables.
    #[test]
    fn unresolved_choice_isomorphism_weights_ignored_without_option() {
        let g1 = graph_w(&[1.0, 2.0], &[2.0, 3.0], &[5.0, 7.0]);
        let g2 = graph_w(&[1.0, 2.0], &[2.0, 3.0], &[1.0, 9.0]);
        let (r, _, p) = iso(&[g1.clone(), g2.clone()]);
        assert_eq!(r, 3);
        assert!(reorder_second_matches_first(&g1, &g2, &p));
    }

    /// U-059-4(c), U-059-5: multigraphs and self-loops as multisets; edgeperm.
    #[test]
    fn unresolved_choice_isomorphism_multigraph_self_loops_and_edgeperm() {
        // 1=2 (double edge), 2-3, loop at 3
        let g1 = graph(&[1.0, 1.0, 2.0, 3.0], &[2.0, 2.0, 3.0, 3.0]);
        // relabelled: 3=2 (double), 2-1, loop at 1
        let g2 = graph(&[1.0, 3.0, 2.0, 2.0], &[1.0, 2.0, 3.0, 1.0]);
        let ((r, _, p), (er, ec, ep)) = iso2(&[g1.clone(), g2.clone()]);
        assert_eq!(r, 3);
        assert!(reorder_second_matches_first(&g1, &g2, &p));
        assert_eq!((er, ec), (4, 1));
        let d1 = graph_data(&g1, "test").unwrap();
        let d2 = graph_data(&g2, "test").unwrap();
        let perm = to_perm(&p);
        let mut used = [false; 4];
        for (k, edge) in d1.edges.iter().enumerate() {
            let j = ep[k] as usize - 1;
            assert!(!used[j]);
            used[j] = true;
            let target = &d2.edges[j];
            let (a, b) = (perm[edge.source], perm[edge.target]);
            assert!(
                (target.source, target.target) == (a, b)
                    || (target.source, target.target) == (b, a)
            );
        }
        // different multiplicities: not isomorphic
        let g3 = graph(&[1.0, 1.0, 2.0, 3.0], &[2.0, 3.0, 3.0, 3.0]);
        assert_eq!(iso(&[g1, g3]), (0, 0, vec![]));
        // edgeperm shapes: not isomorphic -> 0x0; empty graphs -> 1x0
        let path4 = graph(&[1.0, 2.0, 3.0], &[2.0, 3.0, 4.0]);
        let star = graph(&[1.0, 1.0, 1.0], &[2.0, 3.0, 4.0]);
        let ((_, _, _), (er, ec, _)) = iso2(&[path4, star]);
        assert_eq!((er, ec), (0, 0));
        let ((pr, pc, _), (er, ec, _)) = iso2(&[empty_graph(), empty_graph()]);
        assert_eq!((pr, pc, er, ec), (1, 0, 1, 0));
    }

    /// U-059-4(a): search on larger graphs (valid permutation; regular
    /// non-isomorphic pair rejected).
    #[test]
    fn unresolved_choice_isomorphism_search_cases() {
        // Petersen graph and a relabelling of it
        let outer: Vec<(f64, f64)> = (0..5).map(|i| (i as f64, ((i + 1) % 5) as f64)).collect();
        let spokes: Vec<(f64, f64)> = (0..5).map(|i| (i as f64, (i + 5) as f64)).collect();
        let inner: Vec<(f64, f64)> = (0..5)
            .map(|i| ((i + 5) as f64, (((i + 2) % 5) + 5) as f64))
            .collect();
        let edges: Vec<(f64, f64)> = outer.into_iter().chain(spokes).chain(inner).collect();
        let relabel = [7.0, 2.0, 9.0, 0.0, 5.0, 3.0, 8.0, 1.0, 6.0, 4.0];
        let s: Vec<f64> = edges.iter().map(|e| e.0 + 1.0).collect();
        let t: Vec<f64> = edges.iter().map(|e| e.1 + 1.0).collect();
        let s2: Vec<f64> = edges.iter().map(|e| relabel[e.0 as usize] + 1.0).collect();
        let t2: Vec<f64> = edges.iter().map(|e| relabel[e.1 as usize] + 1.0).collect();
        let (g1, g2) = (graph(&s, &t), graph(&s2, &t2));
        let (r, _, p) = iso(&[g1.clone(), g2.clone()]);
        assert_eq!(r, 10);
        assert!(is_permutation(&p, 10));
        assert!(reorder_second_matches_first(&g1, &g2, &p));
        // 6-cycle vs two triangles (both 2-regular)
        let c6 = graph(
            &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            &[2.0, 3.0, 4.0, 5.0, 6.0, 1.0],
        );
        let triangles = graph(
            &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            &[2.0, 3.0, 1.0, 5.0, 6.0, 4.0],
        );
        assert_eq!(iso(&[c6, triangles]), (0, 0, vec![]));
    }

    /// U-059-3: more than two outputs (direct call: dispatcher quirk).
    #[test]
    fn unresolved_choice_isomorphism_output_count() {
        {
            let _guard = crate::output_count::push_output_count(Some(3));
            let err = block_on(isomorphism_builtin(vec![path3(), path3()])).unwrap_err();
            assert_eq!(identifier(err), "RunMat:isomorphism:TooManyOutputs");
        }
        let _guard = crate::output_count::push_output_count(Some(1));
        let out = block_on(isomorphism_builtin(vec![path3(), path3()])).unwrap();
        let Value::OutputList(values) = out else {
            panic!("expected output list");
        };
        assert_eq!(values.len(), 1);
    }
}
