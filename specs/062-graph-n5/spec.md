# Feature Specification: `outedges`, `reordernodes` — graph queries and node reordering (N5)

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`062-graph-n5`; export `SPECIFY_FEATURE=062-graph-n5`)

**Created**: 2026-10-31

**Status**: Batch N5 (F62). Gates G1–G3 pre-approved by the maintainer
(`specs/001-clean-room-matlab-builtins/selection-proposal-n5.md`).

**Input**: Approved exports `outedges` 0.2.0 and `reordernodes` 0.2.0
(N5, `1c91aa1`, IMPORT-MANIFEST Batch 14) (see `provenance.md`).

Family rationale (Principle VII): both builtins are small operations on
RunMat's existing `graph`/`digraph` objects, used by COBRA (`outedges` for
successor edges, `reordernodes` with a permutation from `isomorphism`).
They share the node-resolution and output helpers of `inedges` (F59).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Out-edges of a node (Priority: P1)

`eid = outedges(G, node)` returns the indices of the edges leaving `node`;
`[eid, nid] = outedges(G, node)` (COBRA form) also returns the nodes those
edges reach.

**Acceptance Scenarios** (normative):

1. For `digraph([1 1 2 3], [3 2 3 1])`, whose edge table is
   `[1 2; 1 3; 2 3; 3 1]`, node 1 gives `eid = [1;2]`, `nid = [2;3]`, and
   node 3 gives `eid = 4`. [outedges.signature-primary, outedges.order]
   (stored edge order: D-062-1)
2. A node with no out-edges, including an isolated node, gives 0×1 double
   `eid` and `nid`. [outedges.empty] (isolated-node set-up: D-036-2)
3. Named nodes: `eid` is double; `nid` is a cell of char for a char query,
   a string array for a string query, and double indices for an index
   query. [outedges.names]
4. Parallel edges each appear (`nid = [2;2;3]`) and a self-loop is listed
   with the node itself as `nid`. [outedges.multi]
5. For `graph([1 2], [2 3])` node 2 gives `eid = [1;2]`, `nid = [1;3]`
   (both incident edges). [outedges.undirected]
6. An index beyond the node count raises `RunMat:outedges:InvalidNodeID`;
   more than one node `RunMat:outedges:InvalidNodeScalar`; an unknown name
   `RunMat:outedges:UnknownNodeName`. [outedges.error-range,
   outedges.error-vector, outedges.error-unknown-name]

### User Story 2 - Reorder the nodes of a graph (Priority: P1)

`H = reordernodes(G, order)` (COBRA form) returns a graph of the class of
`G` whose node `k` is node `order(k)` of `G`, with node data and edges
carried along; `[H, idx] = reordernodes(G, order)` also returns the edge
permutation.

**Acceptance Scenarios** (normative):

1. A digraph with node variables `Met` `{'m1';'m2';'m1'}` and `Atom`
   `[1;2;3]` and order `[3 1 2]` gives `H.Nodes` `Met`
   `{'m1';'m1';'m2'}`, `Atom` `[3;1;2]`, with the variable names and
   classes kept. [reordernodes.signature-primary, reordernodes.nodes]
   (node-variable set-up: D-062-2)
2. Edges follow their end nodes, renumbered, and `H.Edges` is in ascending
   order of end nodes: for the cycle `1→2→3→1` and order `[3 1 2]` the end
   nodes stay `[1 2; 2 3; 3 1]` and `idx = [3;1;2]`. [reordernodes.edges]
   (adjacency-built set-up: D-062-1)
3. `digraph([1 2], [2 3], [7 9])` with order `[2 3 1]` gives end nodes
   `[1 2; 3 1]`, `Weight` `[9;7]`, `idx = [2;1]`. [reordernodes.weights]
4. `graph([1 2], [2 3])` with order `[3 2 1]` gives end nodes
   `[1 2; 2 3]`. [reordernodes.undirected]
5. An order given as names `{'c','a','b'}` reorders `Name` to
   `{'c';'a';'b'}`; end nodes are reported by name. [reordernodes.names]
6. The identity order leaves the edges unchanged; the output class is that
   of `G`. [reordernodes.identity]
7. A repeated or missing node raises `RunMat:reordernodes:InvalidOrder`; an
   index beyond the node count `RunMat:reordernodes:InvalidNodeID`.
   [reordernodes.error-repeat, reordernodes.error-range]

### User Story 3 - Unresolved forms fail clearly (Priority: P2)

Every form the exports do not observe either raises
`RunMat:<fn>:UnsupportedOption` / `InvalidInput` or follows a documented
choice (FR-062-14), tagged `unresolved_choice_*` in tests.

## Requirements *(mandatory)*

### Functional Requirements

#### outedges

- **FR-062-01** `outedges(G, node)` MUST return, as a k×1 `double` column,
  the indices of the stored edges leaving `node`, ascending
  [outedges.signature-primary, outedges.order]. With one output only `eid`
  is returned.
- **FR-062-02** `[eid, nid]` MUST also return the other end of each such
  edge, in the same order [outedges.order].
- **FR-062-03** No out-edges MUST give 0×1 `double` `eid` and `nid`
  [outedges.empty].
- **FR-062-04** On graphs with node names `node` MAY be a name; `nid` MUST
  be a k×1 cell of char for a char query, a k×1 string array for a string
  query and a k×1 `double` for an index query [outedges.names].
- **FR-062-05** Each parallel edge MUST appear; a self-loop MUST be listed
  with `nid` equal to `node` [outedges.multi].
- **FR-062-06** On an undirected `graph` every incident edge MUST be listed
  with `nid` the other end [outedges.undirected].
- **FR-062-07** An out-of-range index MUST raise
  `RunMat:outedges:InvalidNodeID`; more than one node
  `RunMat:outedges:InvalidNodeScalar`; an unknown name
  `RunMat:outedges:UnknownNodeName` [outedges.error-range,
  outedges.error-vector, outedges.error-unknown-name].

#### reordernodes

- **FR-062-08** `reordernodes(G, order)` MUST return an object of the class
  of `G` with the same node count, whose node `k` is node `order(k)` of
  `G` [reordernodes.signature-primary, reordernodes.identity].
- **FR-062-09** Every `G.Nodes` variable MUST be permuted with the nodes,
  keeping its name, position and class [reordernodes.nodes]; `Name` follows
  the same rule [reordernodes.names].
- **FR-062-10** Each edge MUST be renumbered through the permutation (for a
  `graph`, smaller end first) and `H.Edges` MUST be listed in ascending
  order of `(first end, second end)` [reordernodes.edges,
  reordernodes.undirected]; every other `Edges` variable (e.g. `Weight`)
  MUST follow its edge [reordernodes.weights].
- **FR-062-11** `[H, idx]` MUST return `idx` as a `double` column with
  `numedges` entries, row `k` of `H.Edges` being row `idx(k)` of `G.Edges`
  [reordernodes.edges, reordernodes.weights].
- **FR-062-12** `order` MAY be a numeric vector of node indices or a list
  of all node names (cell, string or char) [reordernodes.names].
- **FR-062-13** An `order` that repeats or omits nodes MUST raise
  `RunMat:reordernodes:InvalidOrder`; an index beyond the node count
  `RunMat:reordernodes:InvalidNodeID` [reordernodes.error-repeat,
  reordernodes.error-range].

#### Common

- **FR-062-14** (documented choices, local ids; not MATLAB requirements)
  - **U-062-1** `outedges`: a first input that is not a `graph`/`digraph`
    raises `RunMat:outedges:UndefinedFunction` (dispatch-level,
    non-normative; same choice as U-059-1).
  - **U-062-2** `outedges`: missing inputs raise `InvalidInput`; more than
    two inputs `UnsupportedOption`.
  - **U-062-3** Both builtins: more than two requested outputs raise
    `RunMat:<fn>:TooManyOutputs`; zero requested outputs return an empty
    output list (as F59).
  - **U-062-4** `outedges` node validation: identifiers are the same for
    `graph` and `digraph` (outedges.q-undirected-ids); a non-positive,
    non-integer or NaN index raises `InvalidNodeID`; an empty node list
    raises `InvalidNodeScalar`; a name on a graph without names raises
    `UnknownNodeName`; a one-element cell of a name answers like a char
    query; malformed node values raise `InvalidInput`.
  - **U-062-5** `outedges`: an undirected self-loop is listed once (one
    entry per edge); `nid` for a named-graph text query with no out-edges
    is a 0×1 cell (char query) or 0×1 string array (string query).
  - **U-062-6** `reordernodes`: non-graph first input →
    `UndefinedFunction`; missing inputs → `InvalidInput`; more than two
    inputs → `UnsupportedOption`; an `order` that is neither numeric nor
    text → `InvalidInput`.
  - **U-062-7** `reordernodes` order validation: each entry is checked
    first — a non-positive, non-integer, NaN or too-large index raises
    `InvalidNodeID`; a name that is not a node name, or any name on a
    graph without names, raises `RunMat:reordernodes:UnknownNodeName` —
    then the list as a whole (repeats, wrong length) raises
    `InvalidOrder`. Numeric `order` of any shape is read in column-major
    order; indices are accepted on named graphs.
  - **U-062-8** `reordernodes`: edges with equal renumbered end nodes
    (parallel edges) keep their stored relative order (stable sort).
  - **U-062-9** `reordernodes`: RunMat's synthetic `Index` node column of
    unnamed graphs is renumbered `1..n` rather than permuted; any other
    object property is kept; a table variable whose class RunMat's row
    selection does not support raises `UnsupportedOption`. The empty graph
    with order `[]` gives an empty graph and a 0×1 `idx`.
- **FR-062-15** (unresolved, returned to the specification side)
  `*.q-warning-coverage`: no warnings are emitted.
  `outedges.q-undirected-ids`: handled by documented choice U-062-4;
  nothing is asserted about MATLAB behaviour for it.

### Key Entities

- Input: `Value::Object` of class `graph`/`digraph`, read with upstream's
  `graph_data`; `Nodes` / `Edges` tables read with `table_variables` and
  permuted with the table subsystem's `select_rows`.
- Outputs: `double` columns, cell/string columns of names, graph objects.

## Success Criteria *(mandatory)*

- **SC-062-1** Every observed case reproduces in a normative test, except
  values that depend on the upstream constructor's edge order (D-062-1),
  which are kept in `#[ignore]`d tests, with the same behaviour asserted on
  an equivalently stored graph.
- **SC-062-2** Both builtins are registered through `#[runtime_builtin]`;
  `cargo test -p runmat-runtime --lib -- graph_n5` passes.
- **SC-062-3** Every FR maps to at least one test (validation.md).
- **SC-062-4** `cargo test -p runmat-runtime --lib -- builtins::graph` shows
  no regressions.

## Unresolved behaviour (non-normative)

| id | disposition |
|---|---|
| outedges.q-undirected-ids | U-062-4 (same identifiers for graph inputs) |
| outedges.q-warning-coverage, reordernodes.q-warning-coverage | no warnings emitted (FR-062-15) |

## Known deviations

- **D-062-1** (same cause as D-036-1/D-043-1/D-059-1) RunMat's
  `graph`/`digraph` constructor keeps edges in input order (and
  `digraph(A)` stores them column by column); the observed constructor
  sorts them. `outedges(digraph([1 1 2 3], [3 2 3 1]), 1)` stores
  `1→3, 1→2` first and returns `eid = [1;2]` (matches) but `nid = [3;2]`
  (observed `[2;3]`). For case `idx-out` (adjacency-built cycle) RunMat
  stores `[3 1; 1 2; 2 3]`, so `idx = [1;2;3]` (observed `[3;1;2]`); `H`
  itself matches because `reordernodes` sorts its output. The observed
  values are reproduced on equivalently (sorted) stored graphs and kept in
  ignored tests.
- **D-062-2** (as D-059-3) RunMat's graph constructors create no node
  variables other than `Name` (and the synthetic `Index`) and do not accept
  a node table argument; the `Met`/`Atom` cases are tested on objects whose
  `Nodes` table was rebuilt with those columns. RunMat's constructor stores
  `Nodes.Name` and named `Edges.EndNodes` as string arrays (observed: cell);
  `reordernodes` keeps each column's class, so these cases compare text.
- **D-036-2** applies: the observed `digraph([1 2], [2 1], [], 4)`
  (node-count argument) is not accepted by RunMat's upstream graph
  constructor; the `isolated` case uses the equivalent adjacency-matrix
  construction.

## Assumptions

- Upstream's graph subsystem supplies `GraphData`, `graph_data`,
  `edges_table`, `nodes_table`; F59's `paths.rs` supplies the node and
  output helpers (made `pub(super)`, behaviour unchanged).
