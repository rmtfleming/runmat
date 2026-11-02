//! Clean-room `findnode` (node indices for node names or node IDs).
//!
//! Clean-room feature `specs/066-graph-n6/`. Normative behaviour comes only
//! from the approved export `findnode` 0.2.0 (see that feature's
//! `provenance.md`). The result is always a `double` column with one element
//! per queried node, 0 for a name that is not a node (FR-066-11..17).

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

const FINDNODE_INPUTS: [BuiltinParamDescriptor; 2] = [
    param("G", "Graph or digraph object."),
    param(
        "nodeID",
        "Node names (char, string or cellstr) or node indices.",
    ),
];
const FINDNODE_OUTPUTS: [BuiltinParamDescriptor; 1] = [param(
    "idx",
    "Double column of node indices, 0 for a name that is not a node.",
)];
const FINDNODE_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "idx = findnode(G, nodeID)",
    inputs: &FINDNODE_INPUTS,
    outputs: &FINDNODE_OUTPUTS,
}];

macro_rules! findnode_error {
    ($code:literal, $id:literal, $when:literal) => {
        BuiltinErrorDescriptor {
            code: concat!("RM.FINDNODE.", $code),
            identifier: Some($id),
            when: $when,
            message: concat!("findnode: ", $when),
        }
    };
}

/// Local choices (U-066-8..9; index 2 is `Internal`, the fallback of
/// `graph_input`'s `query_error`), then the observed identifiers raised as
/// `RunMat:findnode:<last component>` (graph-family convention, F36-F62).
const FINDNODE_ERRORS: &[BuiltinErrorDescriptor] = &[
    findnode_error!(
        "INVALID_INPUT",
        "RunMat:findnode:InvalidInput",
        "The graph object is malformed."
    ),
    findnode_error!(
        "UNSUPPORTED_OPTION",
        "RunMat:findnode:UnsupportedOption",
        "The call uses a form or option that is not yet specified for RunMat."
    ),
    findnode_error!(
        "INTERNAL",
        "RunMat:findnode:Internal",
        "RunMat cannot build the result value."
    ),
    findnode_error!(
        "UNDEFINED_FUNCTION",
        "RunMat:findnode:UndefinedFunction",
        "The first input is not a graph or digraph object."
    ),
    findnode_error!(
        "TOO_MANY_OUTPUTS",
        "RunMat:findnode:TooManyOutputs",
        "More than one output was requested."
    ),
    findnode_error!("MINRHS", "RunMat:minrhs", "Only G was given."),
    findnode_error!(
        "NO_NAMES",
        "RunMat:findnode:NoNames",
        "A node name was given for a graph without node names."
    ),
    findnode_error!(
        "ARG_TYPE",
        "RunMat:findnode:ArgType",
        "nodeID is not a list of node names or numeric node IDs."
    ),
    findnode_error!(
        "POS_INT",
        "RunMat:findnode:PosInt",
        "A numeric node ID is not a positive integer."
    ),
];

pub const FINDNODE_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &FINDNODE_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: FINDNODE_ERRORS,
};

const NAME: &str = "findnode";
const ERRORS: &[BuiltinErrorDescriptor] = FINDNODE_ERRORS;

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
// findnode
// ---------------------------------------------------------------------------

#[runtime_builtin(
    name = "findnode",
    category = "graph",
    summary = "Return the node indices of node names or node IDs in a graph or digraph.",
    keywords = "findnode,graph,digraph,node,name,index,lookup",
    accel = "cpu",
    type_resolver(numeric_type),
    descriptor(crate::builtins::graph::domain::findnode::FINDNODE_DESCRIPTOR),
    builtin_path = "crate::builtins::graph"
)]
pub(super) async fn findnode_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    let args = gather_values(args).await?;
    // U-066-8
    let count = crate::output_count::current_output_count();
    if let Some(requested) = count.filter(|&c| c > 1) {
        return Err(fail(
            "RunMat:findnode:TooManyOutputs",
            format!("findnode returns one output, {requested} were requested"),
        ));
    }
    let Some(first) = args.first() else {
        return Err(fail("RunMat:minrhs", "not enough input arguments"));
    };
    // U-066-8
    let graph = graph_input(NAME, ERRORS, first)?;
    // findnode.error-one-arg
    if args.len() < 2 {
        return Err(fail("RunMat:minrhs", "not enough input arguments"));
    }
    if args.len() > 2 {
        return Err(fail(
            "RunMat:findnode:UnsupportedOption",
            "only findnode(G, nodeID) is specified",
        ));
    }
    let idx = match query(&args[1])? {
        Query::Ids(values) => {
            // findnode.numeric: the IDs themselves, 0 beyond the node count.
            let n = graph.node_count as f64;
            values
                .into_iter()
                .map(|x| if x <= n { x } else { 0.0 })
                .collect::<Vec<f64>>()
        }
        Query::Names(labels) => {
            // findnode.error-unnamed-char; U-066-9 (cells, empty cell)
            let Some(names) = graph.node_names.as_ref() else {
                return Err(fail(
                    "RunMat:findnode:NoNames",
                    "the graph has no node names",
                ));
            };
            // findnode.names, findnode.not-found: exact (case-sensitive)
            // match, 0 when absent, one entry per query.
            labels
                .iter()
                .map(|label| {
                    names
                        .iter()
                        .position(|name| name == label)
                        .map_or(0.0, |k| (k + 1) as f64)
                })
                .collect()
        }
    };
    // findnode.output-class: a double column.
    let k = idx.len();
    let idx = double(NAME, ERRORS, idx, k, 1)?;
    Ok(match count {
        None => idx,
        Some(0) => Value::OutputList(Vec::new()),
        Some(_) => Value::OutputList(vec![idx]),
    })
}

enum Query {
    Ids(Vec<f64>),
    Names(Vec<String>),
}

fn arg_type(detail: &str) -> RuntimeError {
    fail("RunMat:findnode:ArgType", detail)
}

/// Classify and read `nodeID` in column-major order.
fn query(value: &Value) -> BuiltinResult<Query> {
    match value {
        // findnode.error-logical (U-066-9: a scalar logical too)
        Value::Bool(_) | Value::LogicalArray(_) => Err(arg_type(
            "nodeID must be node names or numeric node IDs, not logical",
        )),
        Value::Num(_) | Value::Int(_) | Value::Tensor(_) => {
            let values = super::parsing::numeric_f64_vector(value, NAME)
                .map_err(|_| arg_type("invalid numeric node IDs"))?;
            // findnode.error-numeric-zero; U-066-9 (NaN, Inf)
            if values
                .iter()
                .any(|&x| !x.is_finite() || x < 1.0 || x.fract() != 0.0)
            {
                return Err(fail(
                    "RunMat:findnode:PosInt",
                    "numeric node IDs must be positive integers",
                ));
            }
            Ok(Query::Ids(values))
        }
        Value::String(text) => Ok(Query::Names(vec![text.clone()])),
        // StringArray data is column-major.
        Value::StringArray(array) => Ok(Query::Names(array.data.clone())),
        // findnode.error-logical (2-row char matrix); U-066-9 (empty char is
        // one name).
        Value::CharArray(chars) => {
            if chars.rows > 1 || chars.shape.len() > 2 {
                return Err(arg_type("a char nodeID must be a single row"));
            }
            Ok(Query::Names(vec![chars.data.iter().collect()]))
        }
        Value::Cell(cell) => {
            // Cell payloads are row-major per 2-D page; read column-major
            // (findnode.names, case `cell-matrix`).
            let page = cell.rows * cell.cols;
            let pages = cell.data.len().checked_div(page).unwrap_or(0);
            let mut labels = Vec::with_capacity(cell.data.len());
            for p in 0..pages {
                for c in 0..cell.cols {
                    for r in 0..cell.rows {
                        let entry = &cell.data[p * page + r * cell.cols + c];
                        // findnode.error-logical (cell holding a number)
                        let text = match entry {
                            Value::CharArray(chars) if chars.rows <= 1 => {
                                chars.data.iter().collect()
                            }
                            Value::String(text) => text.clone(),
                            Value::StringArray(array) if array.data.len() == 1 => {
                                array.data[0].clone()
                            }
                            _ => {
                                return Err(arg_type(
                                    "a cell nodeID must hold only character vectors or strings",
                                ))
                            }
                        };
                        labels.push(text);
                    }
                }
            }
            Ok(Query::Names(labels))
        }
        _ => Err(arg_type("nodeID must be node names or numeric node IDs")),
    }
}

#[cfg(test)]
mod graph_n6_findnode_tests {
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

    fn chr(text: &str) -> Value {
        Value::CharArray(CharArray::new_row(text))
    }

    /// A cell from its elements in row-major (MATLAB literal) order.
    fn cellstr(values: &[&str], rows: usize, cols: usize) -> Value {
        Value::Cell(CellArray::new(values.iter().map(|s| chr(s)).collect(), rows, cols).unwrap())
    }

    fn strings(values: &[&str], rows: usize, cols: usize) -> Value {
        Value::StringArray(
            StringArray::new(
                values.iter().map(|s| s.to_string()).collect(),
                vec![rows, cols],
            )
            .unwrap(),
        )
    }

    fn call(name: &str, args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async(name, args))
    }

    fn digraph(s: &[f64], t: &[f64]) -> Value {
        call("digraph", &[row(s), row(t)]).expect("digraph")
    }

    /// The test digraph of the export: digraph({'a','b','c'}, {'b','c','d'}).
    fn abcd() -> Value {
        call(
            "digraph",
            &[
                cellstr(&["a", "b", "c"], 1, 3),
                cellstr(&["b", "c", "d"], 1, 3),
            ],
        )
        .expect("digraph")
    }

    fn unnamed() -> Value {
        digraph(&[1.0, 2.0], &[2.0, 3.0])
    }

    fn identifier(err: RuntimeError) -> String {
        err.identifier().unwrap_or("<none>").to_string()
    }

    fn err_id(args: &[Value]) -> String {
        identifier(call("findnode", args).expect_err("expected an error"))
    }

    /// findnode(G, q) as (rows, cols, values); the class must be double.
    fn find(g: &Value, q: Value) -> (usize, usize, Vec<f64>) {
        let idx = call("findnode", &[g.clone(), q]).expect("findnode");
        let class = call("class", std::slice::from_ref(&idx)).unwrap();
        assert_eq!(parse_string_vector(&class).unwrap(), vec!["double"]);
        match idx {
            Value::Num(n) => (1, 1, vec![n]),
            Value::Tensor(t) => (t.rows, t.cols, tensor::tensor_values_f64(&t)),
            other => panic!("expected double, got {other:?}"),
        }
    }

    fn column(values: &[f64]) -> (usize, usize, Vec<f64>) {
        (values.len(), 1, values.to_vec())
    }

    fn boxed(args: Vec<Value>) -> futures::future::LocalBoxFuture<'static, BuiltinResult<Value>> {
        Box::pin(findnode_builtin(args))
    }

    fn multi(args: &[Value], count: usize) -> BuiltinResult<Vec<Value>> {
        let _guard = crate::output_count::push_output_count(Some(count));
        match block_on(boxed(args.to_vec()))? {
            Value::OutputList(values) => Ok(values),
            other => panic!("expected output list, got {other:?}"),
        }
    }

    fn edge_end_nodes(g: &Value) -> Value {
        let Value::Object(object) = g else {
            panic!("expected graph");
        };
        let Some(Value::Object(table)) = object.properties.get("Edges") else {
            panic!("missing Edges");
        };
        table_variables(table).unwrap().fields["EndNodes"].clone()
    }

    // ---- observed (normative) ---------------------------------------------

    /// Set-up check for case `node-names`.
    #[test]
    fn observed_findnode_node_names() {
        let Value::Object(object) = abcd() else {
            panic!("expected graph");
        };
        let Some(Value::Object(nodes)) = object.properties.get("Nodes") else {
            panic!("missing Nodes");
        };
        let name = table_variables(nodes).unwrap().fields["Name"].clone();
        assert_eq!(
            parse_string_vector(&name).unwrap(),
            vec!["a", "b", "c", "d"]
        );
    }

    /// findnode.signature-primary, cobra-form, output-class (case
    /// `cell-row`): a 1x2 cell row {'b','c'} of G.Edges.EndNodes.
    #[test]
    fn observed_findnode_cobra_cell_row() {
        let g = abcd();
        assert_eq!(find(&g, cellstr(&["b", "c"], 1, 2)), column(&[2.0, 3.0]));
        // Row 2 of RunMat's EndNodes (a string array, D-066-1) answers the same.
        let Value::StringArray(end) = edge_end_nodes(&g) else {
            panic!("expected string EndNodes");
        };
        let row2 = vec![end.data[1].clone(), end.data[1 + end.rows].clone()];
        assert_eq!(row2, vec!["b", "c"]);
        assert_eq!(find(&g, strings(&["b", "c"], 1, 2)), column(&[2.0, 3.0]));
    }

    /// findnode.names, output-class (cases `char`, `string-array`,
    /// `cell-column`, `cell-matrix`).
    #[test]
    fn observed_findnode_names() {
        let g = abcd();
        assert_eq!(find(&g, chr("c")), column(&[3.0]));
        assert_eq!(find(&g, strings(&["d", "a"], 1, 2)), column(&[4.0, 1.0]));
        assert_eq!(find(&g, cellstr(&["b", "a"], 2, 1)), column(&[2.0, 1.0]));
        // {'a' 'b'; 'c' 'd'} is read in column-major order.
        assert_eq!(
            find(&g, cellstr(&["a", "b", "c", "d"], 2, 2)),
            column(&[1.0, 3.0, 2.0, 4.0])
        );
    }

    /// findnode.names (case `undirected`).
    #[test]
    fn observed_findnode_undirected() {
        let g = call(
            "graph",
            &[cellstr(&["x", "y"], 1, 2), cellstr(&["y", "z"], 1, 2)],
        )
        .unwrap();
        assert_eq!(find(&g, cellstr(&["z", "x"], 1, 2)), column(&[3.0, 1.0]));
    }

    /// findnode.not-found (cases `unknown`, `mixed-unknown`, `repeated`,
    /// `case-sensitive`, `empty-cell`).
    #[test]
    fn observed_findnode_not_found() {
        let g = abcd();
        assert_eq!(find(&g, chr("z")), column(&[0.0]));
        assert_eq!(
            find(&g, cellstr(&["a", "z", "c"], 1, 3)),
            column(&[1.0, 0.0, 3.0])
        );
        assert_eq!(find(&g, cellstr(&["b", "b"], 1, 2)), column(&[2.0, 2.0]));
        assert_eq!(find(&g, chr("A")), column(&[0.0]));
        assert_eq!(find(&g, cellstr(&[], 0, 0)), (0, 1, vec![]));
    }

    /// findnode.numeric (cases `numeric-named`, `numeric-unnamed`,
    /// `numeric-range`, `numeric-range-named`).
    #[test]
    fn observed_findnode_numeric() {
        assert_eq!(find(&abcd(), row(&[3.0, 1.0])), column(&[3.0, 1.0]));
        assert_eq!(find(&unnamed(), col(&[2.0, 3.0])), column(&[2.0, 3.0]));
        assert_eq!(find(&unnamed(), Value::Num(7.0)), column(&[0.0]));
        assert_eq!(find(&abcd(), Value::Num(9.0)), column(&[0.0]));
    }

    /// findnode.error-unnamed-char, error-logical, error-numeric-zero,
    /// error-one-arg (cases `err-*`).
    #[test]
    fn rejects_findnode_errors() {
        for bad in [0.0, -1.0, 1.5] {
            assert_eq!(
                err_id(&[unnamed(), Value::Num(bad)]),
                "RunMat:findnode:PosInt"
            );
        }
        let two = call("digraph", &[chr("ab"), chr("cd")]).unwrap();
        let matrix = Value::CharArray(CharArray::new(vec!['a', 'b', 'c', 'd'], 2, 2).unwrap());
        assert_eq!(err_id(&[two, matrix]), "RunMat:findnode:ArgType");
        assert_eq!(
            err_id(&[unnamed(), Value::String("a".into())]),
            "RunMat:findnode:NoNames"
        );
        assert_eq!(err_id(&[unnamed(), chr("a")]), "RunMat:findnode:NoNames");
        let logical = Value::LogicalArray(LogicalArray::new(vec![1, 0], vec![1, 2]).unwrap());
        assert_eq!(err_id(&[abcd(), logical]), "RunMat:findnode:ArgType");
        assert_eq!(err_id(&[abcd()]), "RunMat:minrhs");
        let numeric_cell = Value::Cell(CellArray::new(vec![Value::Num(1.0)], 1, 1).unwrap());
        assert_eq!(err_id(&[abcd(), numeric_cell]), "RunMat:findnode:ArgType");
    }

    // ---- documented choices (non-normative) -------------------------------

    /// U-066-8: non-graph input, arity, output count.
    #[test]
    fn unresolved_choice_findnode_inputs() {
        assert_eq!(
            err_id(&[row(&[1.0]), row(&[1.0])]),
            "RunMat:findnode:UndefinedFunction"
        );
        assert_eq!(
            identifier(block_on(boxed(Vec::new())).unwrap_err()),
            "RunMat:minrhs"
        );
        assert_eq!(
            err_id(&[abcd(), chr("a"), chr("b")]),
            "RunMat:findnode:UnsupportedOption"
        );
        let err = multi(&[abcd(), chr("a")], 2).unwrap_err();
        assert_eq!(identifier(err), "RunMat:findnode:TooManyOutputs");
        let one = multi(&[abcd(), chr("a")], 1).unwrap();
        assert_eq!(one, vec![Value::Num(1.0)]);
        assert!(multi(&[abcd(), chr("a")], 0).unwrap().is_empty());
    }

    /// U-066-9 (findnode.q-query-types): further query forms.
    #[test]
    fn unresolved_choice_findnode_query_types() {
        let g = abcd();
        // Numeric: NaN and Inf are not positive integers; empty gives 0x1;
        // integer classes and matrices (column-major) are accepted.
        for bad in [f64::NAN, f64::INFINITY] {
            assert_eq!(
                err_id(&[g.clone(), Value::Num(bad)]),
                "RunMat:findnode:PosInt"
            );
        }
        let empty = Value::Tensor(Tensor::new(Vec::new(), vec![0, 0]).unwrap());
        assert_eq!(find(&g, empty), (0, 1, vec![]));
        assert_eq!(find(&g, Value::Int(IntValue::I32(2))), column(&[2.0]));
        let m = Value::Tensor(Tensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap());
        assert_eq!(find(&g, m), column(&[1.0, 2.0, 3.0, 4.0]));
        // A scalar logical is ArgType like a logical vector.
        assert_eq!(
            err_id(&[g.clone(), Value::Bool(true)]),
            "RunMat:findnode:ArgType"
        );
        // A 2x2 string array ["a" "b"; "c" "d"] (column-major payload) is
        // read in column-major order, like the 2x2 cell.
        let s = Value::StringArray(
            StringArray::new(
                vec!["a".into(), "c".into(), "b".into(), "d".into()],
                vec![2, 2],
            )
            .unwrap(),
        );
        assert_eq!(find(&g, s), column(&[1.0, 3.0, 2.0, 4.0]));
        // A cell of string scalars works like a cellstr.
        let cs =
            Value::Cell(CellArray::new(vec![Value::String("d".into()), chr("b")], 1, 2).unwrap());
        assert_eq!(find(&g, cs), column(&[4.0, 2.0]));
        // Cells on a graph without names: NoNames (empty cell included);
        // element types are checked first.
        assert_eq!(
            err_id(&[unnamed(), cellstr(&["a"], 1, 1)]),
            "RunMat:findnode:NoNames"
        );
        assert_eq!(
            err_id(&[unnamed(), cellstr(&[], 0, 0)]),
            "RunMat:findnode:NoNames"
        );
        let numeric_cell = Value::Cell(CellArray::new(vec![Value::Num(1.0)], 1, 1).unwrap());
        assert_eq!(
            err_id(&[unnamed(), numeric_cell]),
            "RunMat:findnode:ArgType"
        );
        // Other classes are ArgType.
        assert_eq!(
            err_id(&[g, Value::Struct(runmat_value::StructValue::new())]),
            "RunMat:findnode:ArgType"
        );
    }
}
