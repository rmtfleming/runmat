# Feature Specification: `rmnode`, `addedge`, `rmedge`, `laplacian` — graph editing

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`043-graph-editing`; export `SPECIFY_FEATURE=043-graph-editing`)

**Created**: 2026-10-30

**Status**: Batch N1 (F43). Gates G1–G3 pre-approved by the maintainer
(`specs/001-clean-room-matlab-builtins/selection-proposal-n1.md`).

**Input**: Approved exports `rmnode`, `addedge`, `rmedge`, `laplacian`
0.2.0 (see `provenance.md`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Remove nodes (Priority: P1)

`H = rmnode(G, nodeIDs)` returns a new graph or digraph without the given
nodes and without every edge that touches them.

**Acceptance Scenarios** (normative):

1. `rmnode(graph([1 2 3], [2 3 4]), 2)` has 3 nodes and one edge `[2 3]`;
   with `[1 4]` it has 2 nodes and edge `[1 2]`. [rmnode.signature-primary,
   rmnode.removes]
2. The result is a `graph` for a graph and a `digraph` for a digraph;
   `rmnode(digraph([1 2 3], [2 3 1]), 1)` has 2 nodes and edge `[1 2]`.
   [rmnode.class]
3. Removing `'b'` from named nodes a, b, c (edges a–b, b–c, c–a) leaves
   nodes a, c and one edge a–c. [rmnode.named]
4. Removing node 3 from the path 1–2–3–4 with weights `[10 20 30]` leaves
   edge `[1 2]` with weight 10. [rmnode.weights]
5. An empty list returns the graph unchanged; removing every node gives
   0 nodes and 0 edges (`EndNodes` 0×2). [rmnode.empty-list, rmnode.all]
6. Removing an isolated node reduces the node count and keeps the edges.
   [rmnode.isolated]

### User Story 2 - Add edges (Priority: P1)

`H = addedge(G, s, t)` and `H = addedge(G, s, t, w)` return a new graph with
the edges `s(i)–t(i)` added.

**Acceptance Scenarios** (normative):

1. Adding 1–3 to the path 1–2–3 gives `[1 2; 1 3; 2 3]`; adding `[3 1]`,
   `[1 2]` gives `[1 2; 1 2; 1 3; 2 3]` (a parallel edge is created).
   [addedge.signature-primary, addedge.edge-order, addedge.duplicates]
2. Adding 3→1 and 2→1 to the digraph 1→2 gives `[1 2; 2 1; 3 1]`.
   [addedge.digraph-order]
3. Edge 2–5 on a 2-node graph gives 5 nodes; edge 1–2 on the empty graph
   gives 2 nodes. [addedge.new-nodes]
4. Adding `[2 1]`–`[3 3]` with weights `[8 9]` to 1–2 (weight 7) gives
   weights `[7; 9; 8]` for `[1 2; 1 3; 2 3]`; a scalar weight 5 applies to
   every new edge. [addedge.weights]
5. Adding `'b'`–`'c'` to the named graph a–b creates node `c`.
   [addedge.named]
6. A self-loop 2–2 can be added. [addedge.self-loop]
7. `s = [1 2]`, `t = 3` adds 1–3 and 2–3. [addedge.scalar-expansion]
8. A weight for a graph without weights raises `RunMat:addedge:NoWeights`;
   `s` and `t` of different lengths raise `RunMat:addedge:EqualNumel`.
   [addedge.error-weight-unweighted, addedge.error-length]

### User Story 3 - Remove edges (Priority: P1)

`H = rmedge(G, s, t)` removes the edges between `s(i)` and `t(i)`;
`H = rmedge(G, idx)` removes edges by index.

**Acceptance Scenarios** (normative):

1. Removing 2–3 (or 3–2) from the path 1–2–3–4 leaves `[1 2; 3 4]` and
   4 nodes. [rmedge.signature-primary, rmedge.by-endpoints]
2. `rmedge(G, [1 3])` on the same path leaves `[2 3]`. [rmedge.by-index]
3. Both parallel 1–2 edges are removed. [rmedge.multigraph]
4. On the digraph 1→2→3, removing 2→1 changes nothing; removing 1→2
   leaves `[2 3]`. [rmedge.digraph-direction]
5. Endpoints with no edge leave the graph unchanged without error.
   [rmedge.absent]
6. Nodes are never removed; remaining edges keep their weights.
   [rmedge.keeps-nodes, rmedge.weights]
7. An edge index beyond the edge count raises `RunMat:rmedge:EdgeBounds`; a
   node index beyond the node count raises `RunMat:rmedge:InvalidNodeID`.
   [rmedge.error-index-range, rmedge.error-node-range]

### User Story 4 - Graph Laplacian (Priority: P1)

`L = laplacian(G)` returns the sparse Laplacian of an undirected simple
graph.

**Acceptance Scenarios** (normative):

1. The path 1–2–3 gives `sparse([1;2;1;2;3;2;3], [1;1;2;2;2;3;3],
   [1;-1;-1;2;-1;-1;1], 3, 3)`. [laplacian.signature-primary,
   laplacian.output-class, laplacian.values]
2. A triangle gives 2 on the diagonal and −1 elsewhere; an isolated node
   gives a zero row and column. [laplacian.values]
3. Weights `[5 7]` give the same matrix as the unweighted path.
   [laplacian.weights-ignored]
4. The empty graph gives a 0×0 sparse matrix. [laplacian.empty]
5. A self-loop raises `RunMat:laplacian:SelfLoops`; parallel edges raise
   `RunMat:laplacian:Multigraph`; a digraph raises
   `RunMat:laplacian:OnlyGraphSupported`. [laplacian.error-self-loop,
   laplacian.error-multigraph, laplacian.error-digraph]

### User Story 5 - Unresolved forms fail clearly (Priority: P2)

Every form the exports do not observe either raises
`RunMat:<fn>:UnsupportedOption` or follows a documented choice (FR-043-20,
FR-043-21), tagged `unresolved_choice_*` in tests.

## Requirements *(mandatory)*

### Functional Requirements

#### rmnode

- **FR-043-01** `rmnode(G, nodeIDs)` MUST return a new object with the
  listed nodes and every edge touching them removed; remaining nodes are
  renumbered consecutively in their original order [rmnode.signature-primary,
  rmnode.removes]. Removing an isolated node only reduces the node count
  [rmnode.isolated].
- **FR-043-02** The result MUST keep the input class (`graph`/`digraph`)
  [rmnode.class].
- **FR-043-03** Node names MUST be accepted as identifiers; the remaining
  names are kept [rmnode.named].
- **FR-043-04** Remaining edges MUST keep their weights [rmnode.weights].
- **FR-043-05** An empty list MUST return the graph unchanged; removing every
  node MUST give 0 nodes and 0 edges [rmnode.empty-list, rmnode.all].
- **FR-043-06** (documented choice — rmnode.q-ignored-ids, backed by the
  non-normative observation rmnode.ignored-ids) A positive integer index
  beyond the node count, or a name that is not a node of a named graph, is
  ignored without error. Both kinds were observed, so the observations
  settle the choice; it is still labelled a choice, not a MATLAB
  requirement.

#### addedge

- **FR-043-07** `addedge(G, s, t)` and `addedge(G, s, t, w)` MUST return a
  new object of the same class with the edges added
  [addedge.signature-primary]. The resulting edge list (existing plus new
  edges) MUST be sorted by end nodes, with undirected edges stored smaller
  node first [addedge.edge-order]; digraph edges keep their direction and
  are sorted by source then target [addedge.digraph-order]. Among edges
  with equal end nodes, existing edges stay before new ones and new ones
  keep their input order (stable sort; inference, not observed).
- **FR-043-08** An existing edge added again MUST create a parallel edge
  [addedge.duplicates]; a self-loop MUST be accepted [addedge.self-loop].
- **FR-043-09** An index beyond the node count MUST create the missing
  nodes, including on the empty graph [addedge.new-nodes]; a new name on a
  named graph MUST create a node with that name [addedge.named].
- **FR-043-10** Weights MUST follow their edges through the sort; a scalar
  weight applies to every new edge [addedge.weights].
- **FR-043-11** A scalar `t` MUST expand against a vector `s`
  [addedge.scalar-expansion].
- **FR-043-12** A weight argument for a graph without a `Weight` edge
  variable MUST raise `RunMat:addedge:NoWeights`
  [addedge.error-weight-unweighted]; `s` and `t` of different lengths
  (neither scalar) MUST raise `RunMat:addedge:EqualNumel`
  [addedge.error-length].

#### rmedge

- **FR-043-13** `rmedge(G, s, t)` MUST remove every edge between `s(i)` and
  `t(i)` (all parallel edges); for graphs endpoint order does not matter,
  for digraphs only `s(i)→t(i)` matches [rmedge.signature-primary,
  rmedge.by-endpoints, rmedge.multigraph, rmedge.digraph-direction]. Pairs
  with no edge are not an error [rmedge.absent].
- **FR-043-14** `rmedge(G, idx)` MUST remove the edges at the given
  positions of the stored edge list [rmedge.by-index].
- **FR-043-15** Nodes MUST NOT be removed and remaining edges keep their
  weights [rmedge.keeps-nodes, rmedge.weights].
- **FR-043-16** An edge index beyond the edge count MUST raise
  `RunMat:rmedge:EdgeBounds` [rmedge.error-index-range]; a node index beyond
  the node count MUST raise `RunMat:rmedge:InvalidNodeID`
  [rmedge.error-node-range].

#### laplacian

- **FR-043-17** `laplacian(G)` MUST return a sparse `double`
  numnodes×numnodes matrix [laplacian.signature-primary,
  laplacian.output-class] equal to `D − A` (node degrees on the diagonal,
  −1 for each adjacent pair, isolated nodes give a zero row and column)
  [laplacian.values]; weights are ignored [laplacian.weights-ignored]; the
  empty graph gives 0×0 [laplacian.empty].
- **FR-043-18** A self-loop MUST raise `RunMat:laplacian:SelfLoops`;
  parallel edges `RunMat:laplacian:Multigraph`; a digraph
  `RunMat:laplacian:OnlyGraphSupported` [laplacian.error-self-loop,
  laplacian.error-multigraph, laplacian.error-digraph].

#### Common

- **FR-043-19** Results are new graph objects built with upstream's
  `graph_object`, so `G.Edges`, `G.Nodes`, `numnodes`, `numedges` stay
  consistent; the input object is not modified.
- **FR-043-20** (documented choices, local ids)
  - **U-043-1** A first input that is not a `graph`/`digraph` raises
    `RunMat:<fn>:UndefinedFunction` (for `laplacian` this matches the
    non-normative observation laplacian.numeric-input; for the other three
    it is a choice).
  - **U-043-2** Missing required inputs raise `RunMat:<fn>:InvalidInput`;
    extra inputs or unobserved forms (e.g. a non-numeric fourth `addedge`
    argument, more than two inputs to `rmnode`, more than one to
    `laplacian`) raise `RunMat:<fn>:UnsupportedOption`.
  - **U-043-3** More than one requested output raises
    `RunMat:<fn>:TooManyOutputs`.
  - **U-043-4** `rmnode`: a numeric identifier that is not a positive
    integer raises `RunMat:rmnode:InvalidInput`; names on a graph without
    node names raise `RunMat:rmnode:UnsupportedOption`.
  - **U-043-5** `addedge`: a scalar `s` also expands against a vector `t`;
    on a weighted graph, edges added without `w` get weight 1; `w` with a
    length other than 1 or the number of new edges raises
    `RunMat:addedge:EqualNumel`; weights must be finite and nonnegative
    (RunMat's graph-constructor rule) else `RunMat:addedge:InvalidInput`;
    node indices must be positive integers else `RunMat:addedge:InvalidInput`;
    names on an unnamed graph that already has nodes, or an index beyond the
    node count of a named graph, raise `RunMat:addedge:UnsupportedOption`;
    names on the empty unnamed graph create a named graph; new names are
    appended in order of first appearance (`s(1)`, `t(1)`, `s(2)`, …).
  - **U-043-6** `rmedge`: names are accepted as endpoints on named graphs;
    an unknown name, a name on an unnamed graph, or a numeric node id that
    is not a positive integer raises `RunMat:rmedge:InvalidNodeID`; an edge
    index that is not a positive integer raises `RunMat:rmedge:EdgeBounds`;
    a scalar `s` or `t` expands as for `addedge`; other length mismatches
    raise `RunMat:rmedge:InvalidInput`; a non-numeric edge index raises
    `RunMat:rmedge:InvalidInput`.
  - **U-043-7** `rmnode` and `rmedge` keep the remaining edges in their
    stored order (the exports state no ordering for them).
  - **U-043-8** `laplacian`: the self-loop check precedes the multigraph
    check; named-node graphs are accepted (node order).
  - **U-043-9** Weight representation follows upstream `graph_object`: a
    `Weight` edge variable is present exactly when some weight differs from
    1. "Graph without weights" (FR-043-12) means no `Weight` variable.
- **FR-043-21** (unresolved, returned to the specification side)
  `*.q-warning-coverage`: the implementation emits no warnings.
  `rmnode.q-graph-variables`, `addedge.q-graph-variables`,
  `rmedge.q-graph-variables`: RunMat graph objects carry only `EndNodes`,
  `Weight` and `Name`/`Index`, so there are no other variables to keep;
  nothing is asserted about MATLAB behaviour for them.

### Key Entities

- Input: `Value::Object` of class `graph`/`digraph`, read with upstream's
  `graph_data`.
- Output of `rmnode`/`addedge`/`rmedge`: a new `Value::Object` from
  upstream's `graph_object`. Output of `laplacian`: `Value::SparseTensor`
  (real `double`, CSC).

## Success Criteria *(mandatory)*

- **SC-043-1** Every observed case reproduces in a normative test, except
  values that depend on the upstream constructor's edge order (D-043-1),
  which are kept in an `#[ignore]`d test.
- **SC-043-2** The four builtins are registered through
  `#[runtime_builtin]`; `cargo test -p runmat-runtime --lib --
  graph_editing_tests` passes.
- **SC-043-3** Every FR maps to at least one test (validation.md).
- **SC-043-4** `cargo test -p runmat-runtime --lib -- builtins::graph` shows
  no regressions.

## Unresolved behaviour (non-normative)

| id | disposition |
|---|---|
| rmnode.q-ignored-ids | documented choice FR-043-06 (ignore), settled by cases `out-of-range` and `unknown-name` |
| rmnode.q-warning-coverage, addedge.q-warning-coverage, rmedge.q-warning-coverage, laplacian.q-warning-coverage | no warnings emitted (FR-043-21) |
| rmnode.q-graph-variables, addedge.q-graph-variables, rmedge.q-graph-variables | not applicable to RunMat's graph representation (FR-043-21) |
| U-043-1 … U-043-9 (local) | FR-043-20 |

## Known deviations

- **D-043-1** (same cause as D-036-1) RunMat's `graph` constructor keeps
  edges in input order and endpoints as given; the observed constructor
  canonicalises them. rmnode cases `named` (observed `EndNodes` `[a c]`;
  RunMat stores `c–a`) and `unknown-name` (observed `[a b; a b]`; RunMat
  `[a b; b a]`) differ in endpoint order only. Node counts, edge counts,
  names and edge sets match. The observed values are kept in an ignored
  test. `addedge` is unaffected: it sorts its result (FR-043-07).
- **D-043-2** (same cause as D-036-2) The observed `graph(1, 2, 1, 3)`
  (node-count argument) is not accepted by RunMat's constructor; tests use
  the equivalent adjacency matrix. That graph has unit weights, so RunMat
  stores no `Weight` variable (U-043-9) where the observation reports
  `Weight = 1` (rmnode case `isolated`); the weight value itself (1) is
  preserved.

## Assumptions

- Upstream's graph subsystem supplies `GraphData`, `graph_data` and
  `graph_object`; upstream's `SparseTensor` supplies the sparse output.
