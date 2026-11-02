# Feature Specification: `subgraph`, `findnode` — induced subgraphs and node lookup (N6)

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`066-graph-n6`; export `SPECIFY_FEATURE=066-graph-n6`)

**Created**: 2026-11-02

**Status**: Batch N6 (F66). Gates G1–G3 pre-approved by the maintainer
(`specs/001-clean-room-matlab-builtins/selection-proposal-n6.md`).

**Input**: Approved exports `subgraph` 0.2.0 and `findnode` 0.2.0
(N6, `e364206`, IMPORT-MANIFEST Batch 15) (see `provenance.md`).

Family rationale (Principle VII): both builtins are small operations on
RunMat's existing `graph`/`digraph` objects used by COBRA (`subgraph` with a
logical mask or index vector, `findnode` with a row of `G.Edges.EndNodes`).
They reuse the graph-input helpers of F59 (`paths.rs`) and the
renumber-and-sort / table-row handling of `reordernodes` (F62).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Induced subgraph (Priority: P1)

`H = subgraph(G, nodeIDs)` (COBRA forms `subgraph(ATG, mask)`,
`subgraph(BG, idx)`) returns the graph induced by the selected nodes.

**Acceptance Scenarios** (normative). `G` is the 4-cycle
`digraph([1 2 3 4], [2 3 4 1])` with node variables `Met` `{'a';'b';'c';'d'}`,
`Atom` `[1;2;3;4]` and edge variable `rxns` `{'r1';…;'r4'}` (set-up:
D-066-2):

1. Mask `[true true true false]` gives nodes `a, b, c` (`Atom [1;2;3]`) and
   edges `r1, r2` as `[1 2; 2 3]`. [subgraph.signature-primary,
   subgraph.node-order, subgraph.node-table, subgraph.induced-edges]
2. Column mask `[1;0;1;1]` gives `a, c, d` and `r3, r4` as `[2 3; 3 1]`;
   indices `[2;3]` give `b, c` and `r2` as `[1 2]`. [subgraph.node-order,
   subgraph.induced-edges]
3. Indices `[3 1 2]` give `c, a, b` (`Atom [3;1;2]`) and `r1, r2` as
   `[2 3; 3 1]`. [subgraph.node-order, subgraph.induced-edges]
4. Indices `[4 1 2]` give `d, a, b` and edges `r4, r1` as `[1 2; 2 3]`
   (sorted by renumbered end nodes, not G's order). [subgraph.edge-order]
5. An all-false mask or `[]` gives 0 nodes and 0 edges keeping the variable
   names `Met, Atom` / `EndNodes, rxns` (0×1, 0×2). [subgraph.empty]
6. `subgraph(digraph({'a','b','c'}, {'b','c','a'}), {'c','a'})` gives
   `Name` `c, a` and end nodes `{'c','a'}` by name. [subgraph.names]
   (class of the name columns: D-066-1)
7. Weighted `digraph([1 2 1], [2 3 3], [5 6 7])`, nodes `[1 2]`: one edge
   `[1 2]`, `Weight 5`. Parallel edges and self-loops are kept
   (`[1 1; 1 2; 1 2]`). A kept node with no kept edge stays (0×2 end
   nodes). `H` has the class of `G`. [subgraph.induced-edges,
   subgraph.multigraph, subgraph.isolated, subgraph.output-class]
8. A repeated index or a mask of the wrong length raises
   `RunMat:subgraph:InvalidInd`; an index beyond the node count
   `RunMat:subgraph:InvalidNodeID`; an unknown name
   `RunMat:subgraph:UnknownNodeName`; `subgraph(G)`
   `RunMat:minrhs`. [subgraph.error-*]

### User Story 2 - Node indices from names or IDs (Priority: P1)

`idx = findnode(G, nodeID)` returns a `double` column of node indices.

**Acceptance Scenarios** (normative). `G = digraph({'a','b','c'},
{'b','c','d'})`:

1. The cell row `{'b','c'}` (a row of `G.Edges.EndNodes`) gives `[2;3]`.
   [findnode.signature-primary, findnode.cobra-form]
2. `'c'` → `3`; `["d" "a"]` → `[4;1]`; `{'b';'a'}` → `[2;1]`;
   `{'a' 'b'; 'c' 'd'}` → `[1;3;2;4]` (column-major); an undirected named
   graph behaves the same. [findnode.names, findnode.output-class]
3. An unknown name (also one differing only in case) gives 0 in its
   position; repeated names give repeated indices; `{}` gives 0×1.
   [findnode.not-found]
4. Positive integer IDs come back as a column (`[3 1]` → `[3;1]`) on named
   and unnamed graphs; an ID beyond the node count gives 0.
   [findnode.numeric]
5. A name on an unnamed graph raises `RunMat:findnode:NoNames`; a
   logical, a cell holding a number or a 2-row char matrix
   `RunMat:findnode:ArgType`; `0`, `-1`, `1.5`
   `RunMat:findnode:PosInt`; `findnode(G)` `RunMat:minrhs`.
   [findnode.error-*]

### User Story 3 - Unresolved forms fail clearly (Priority: P2)

Every form the exports do not observe either raises a `RunMat:` error or
follows a documented choice (FR-066-18), tagged `unresolved_choice_*` in
tests.

## Requirements *(mandatory)*

### Functional Requirements

#### subgraph

- **FR-066-01** `subgraph(G, nodeIDs)` MUST return an object of the class
  of `G` [subgraph.signature-primary, subgraph.output-class] whose node
  count is the number of selected nodes.
- **FR-066-02** A logical mask MUST keep the selected nodes in their
  original order; indices or names MUST keep them in the order given
  [subgraph.node-order].
- **FR-066-03** Every `G.Nodes` variable MUST be carried over for the
  selected rows, keeping its name, position and class
  [subgraph.node-table].
- **FR-066-04** `H` MUST contain exactly the edges of `G` with both end
  nodes selected, renumbered to positions in `H` (for a `graph`, smaller
  end first), and every other `Edges` variable (e.g. `rxns`, `Weight`) MUST
  follow its edge [subgraph.induced-edges].
- **FR-066-05** `H.Edges` MUST be sorted by the renumbered `EndNodes` rows
  (ascending `(first end, second end)`), not kept in G's edge order
  [subgraph.edge-order].
- **FR-066-06** Parallel edges and self-loops between selected nodes MUST be
  kept [subgraph.multigraph]; a selected node with no kept edge MUST stay
  [subgraph.isolated].
- **FR-066-07** An all-false mask or an empty index vector MUST give 0 nodes
  and 0 edges keeping the node and edge variable names [subgraph.empty].
- **FR-066-08** With named nodes, `nodeIDs` MAY be names, and `H`'s
  end nodes MUST be reported by name [subgraph.names] (class: D-066-1).
- **FR-066-09** A repeated node or a mask with other than one element per
  node MUST raise `RunMat:subgraph:InvalidInd`; an index beyond the
  node count `RunMat:subgraph:InvalidNodeID`; an unknown name
  `RunMat:subgraph:UnknownNodeName` (observed on digraphs; graph inputs:
  U-066-4)
  [subgraph.error-repeat, subgraph.error-range,
  subgraph.error-unknown-name].
- **FR-066-10** `subgraph(G)` MUST raise `RunMat:minrhs`
  [subgraph.error-one-arg].

#### findnode

- **FR-066-11** `findnode(G, nodeID)` MUST return a `double` column with one
  element per queried node [findnode.signature-primary,
  findnode.output-class].
- **FR-066-12** Names (char row, string array, cell of char of any 2-D
  shape, read in column-major order) MUST be matched exactly to node
  positions, on `graph` and `digraph` [findnode.cobra-form,
  findnode.names].
- **FR-066-13** A name that is not a node (including a case variant) MUST
  give 0 in its position; repeated names MUST give repeated indices; `{}`
  MUST give 0×1 [findnode.not-found].
- **FR-066-14** Positive integer IDs MUST be returned as a column of the
  same values on named and unnamed graphs, 0 for an ID beyond the node
  count [findnode.numeric].
- **FR-066-15** A char or string name on a graph without node names MUST
  raise `RunMat:findnode:NoNames` [findnode.error-unnamed-char].
- **FR-066-16** A logical `nodeID`, a cell holding a number or a 2-row char
  matrix MUST raise `RunMat:findnode:ArgType`; a numeric ID that is
  0, negative or non-integer `RunMat:findnode:PosInt`
  [findnode.error-logical, findnode.error-numeric-zero].
- **FR-066-17** `findnode(G)` MUST raise `RunMat:minrhs`
  [findnode.error-one-arg].

#### Common

- **FR-066-18** (documented choices, local ids; not MATLAB requirements)
  - **U-066-1** Both builtins: a first input that is not a
    `graph`/`digraph` raises `RunMat:<fn>:UndefinedFunction`
    (dispatch-level, non-normative; as U-059-1); a malformed graph object
    `RunMat:<fn>:InvalidInput`; no input at all `RunMat:minrhs`.
  - **U-066-2** Both builtins: more than one requested output raises
    `RunMat:<fn>:TooManyOutputs`; zero requested outputs return an empty
    output list.
  - **U-066-3** Both builtins: more than two inputs raise
    `RunMat:<fn>:UnsupportedOption`. `subgraph`: a `nodeIDs` that is
    neither logical, numeric nor text raises
    `RunMat:subgraph:InvalidInput`.
  - **U-066-4** (`subgraph.q-undirected-ids`) A `graph` input raises the
    same identifiers as a `digraph` (`RunMat:subgraph:InvalidNodeID`,
    `RunMat:subgraph:UnknownNodeName`, `RunMat:subgraph:InvalidInd`), as
    U-062-4.
  - **U-066-5** (`subgraph.q-tie-order`) Edges with equal renumbered end
    nodes (parallel edges) keep their stored relative order (stable sort).
  - **U-066-6** `subgraph` node validation: each entry is checked first — a
    non-positive, non-integer, NaN, Inf or too-large index raises
    `…:InvalidNodeID`; a name that is not a node name, or any name on a
    graph without names, raises `…:UnknownNodeName` — then repeats raise
    `InvalidInd` (repeated names too). Numeric indices of any shape are
    read in column-major order and are accepted on named graphs. A logical
    mask must be a vector (or scalar) with one element per node, otherwise
    `InvalidInd`.
  - **U-066-7** `subgraph`: RunMat's synthetic `Index` node column of
    unnamed graphs is renumbered `1..k`; `NumNodes` is set to `k`; any
    other object property is kept; a table variable whose class RunMat's
    row selection does not support raises
    `RunMat:subgraph:UnsupportedOption`.
  - **U-066-8** `findnode`: as U-066-1..3 (no `InvalidInput` for
    `nodeID`; malformed `nodeID` values are `ArgType`, U-066-9).
  - **U-066-9** (`findnode.q-query-types`) `findnode` further query forms:
    a scalar logical, a struct or any other non-numeric, non-text class
    raises `ArgType`; NaN and Inf raise `PosInt`; an empty numeric gives
    0×1; integer classes and numeric matrices (column-major) are accepted;
    a string matrix is read column-major; a cell may hold string scalars;
    a cell element that is not text raises `ArgType` before the name
    check; any text query on a graph without names (cells, including
    `{}`) raises `NoNames`; the empty char `''` is one name.
- **FR-066-19** (unresolved, returned to the specification side)
  `*.q-warning-coverage`: no warnings are emitted.
  `subgraph.q-undirected-ids`, `subgraph.q-tie-order`,
  `findnode.q-query-types`: handled by documented choices U-066-4, U-066-5,
  U-066-9; nothing is asserted about MATLAB behaviour for them.

### Error identifiers

The exports record identifiers in the MATLAB namespace. RunMat raises the
final component in its own per-function namespace (same mapping as
features 036, 043, 059 and 062; maintainer decision 2026-11-02 for this
feature), `MATLAB:minrhs` as `RunMat:minrhs`:

| observed (export) | raised by RunMat |
|---|---|
| `MATLAB:graphfun:subgraph:InvalidInd` | `RunMat:subgraph:InvalidInd` |
| `MATLAB:graphfun:digraph:InvalidNodeID` (from subgraph) | `RunMat:subgraph:InvalidNodeID` |
| `MATLAB:graphfun:digraph:UnknownNodeName` (from subgraph) | `RunMat:subgraph:UnknownNodeName` |
| `MATLAB:graphfun:findnode:NoNames` | `RunMat:findnode:NoNames` |
| `MATLAB:graphfun:findnode:ArgType` | `RunMat:findnode:ArgType` |
| `MATLAB:graphfun:findnode:PosInt` | `RunMat:findnode:PosInt` |
| `MATLAB:minrhs` (both) | `RunMat:minrhs` |

Local choices use `RunMat:<fn>:<Id>`.

### Key Entities

- Input: `Value::Object` of class `graph`/`digraph`, read with upstream's
  `graph_data` through F59's `graph_input`; `Nodes` / `Edges` tables read
  with `table_variables` and row-selected with the table subsystem's
  `select_rows`.
- Outputs: graph objects (`subgraph`), `double` columns (`findnode`).

## Success Criteria *(mandatory)*

- **SC-066-1** Every observed case of both exports reproduces in a test;
  values that depend on RunMat's column classes (D-066-1) are compared as
  text, with the observed class kept in an `#[ignore]`d test.
- **SC-066-2** Both builtins are registered through `#[runtime_builtin]`;
  `cargo test -p runmat-runtime --lib -- graph_n6` passes.
- **SC-066-3** Every FR maps to at least one test (validation.md).
- **SC-066-4** `cargo test -p runmat-runtime --lib -- builtins::graph` shows
  no regressions.

## Unresolved behaviour (non-normative)

| id | disposition |
|---|---|
| subgraph.q-tie-order | U-066-5 (stable: stored order) |
| subgraph.q-undirected-ids | U-066-4 (same identifiers for graph inputs) |
| findnode.q-query-types | U-066-9 |
| subgraph.q-warning-coverage, findnode.q-warning-coverage | no warnings emitted (FR-066-19) |

## Known deviations

- **D-066-1** (as D-062-2) RunMat's constructor stores `Nodes.Name` and
  named `Edges.EndNodes` as string arrays (observed: cell). `subgraph`
  carries `Name` with its class and builds named `EndNodes` the way the
  upstream graph subsystem stores them (string array), because the
  subsystem's `graph_data` does not read a cell `EndNodes`; case `names`
  is compared as text. The observed `findnode` COBRA form (a 1×2 cell row
  of `G.Edges.EndNodes`) is reproduced with that cell; the RunMat row is a
  1×2 string array, which `findnode` answers identically.
- **D-066-2** (as D-059-3/D-062-2) RunMat's graph constructors create no
  node variables other than `Name` (and the synthetic `Index`) and accept
  neither a node table nor an edge table argument; the `source-graph`
  set-up (`Met`, `Atom`, `rxns`) is tested on an object whose `Nodes` and
  `Edges` tables were rebuilt with those columns.
- **D-036-1/D-043-1/D-062-1** (edges stored in input order) does not change
  any observed `subgraph` value, because `subgraph` sorts its output
  (FR-066-05); in case `multigraph` RunMat stores `G`'s edges as
  `[1 2; 1 2; 1 1; 2 3]` and `H` still has `[1 1; 1 2; 1 2]`. `findnode`
  does not depend on edge order.
- **D-036-2** (`graph(s, t, w, n)` not accepted) is not needed by any case.

## Assumptions

- Upstream's graph subsystem supplies `GraphData`, `graph_data`,
  `edges_table`, `nodes_table`; F59's `paths.rs` supplies `graph_input`
  and `double` (already `pub(super)`, unchanged).
