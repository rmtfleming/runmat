# Feature Specification: `incidence`, `isisomorphic` — graph structure

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`036-graph-structure`; export `SPECIFY_FEATURE=036-graph-structure`)

**Created**: 2026-10-29 (rewritten the same day for spec 0.3.0)

**Status**: Batch C2 — implemented from the approved 0.3.0 exports.
Gates 1–3 pre-approved by the maintainer. Choices recorded against 0.2.0
under decision D1 = B are **superseded** wherever 0.3.0 resolves the point.

**Input**: Approved Tier B widening W2 exports `incidence` 0.3.0 and
`isisomorphic` 0.3.0 (see `provenance.md`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Incidence matrix of a graph or digraph (Priority: P1)

A RunMat user calling `I = incidence(G)` on a `graph` or `digraph` gets a
sparse `double` matrix with one row per node and one column per edge, −1 at
each edge's first end node and +1 at its second.

**Acceptance Scenarios** (normative):

1. `incidence(graph([1 2], [2 3]))` and `incidence(digraph([1 2], [2 3]))`
   each equal `sparse([1;2;2;3], [1;1;2;2], [-1;1;-1;1], 3, 2)`.
   [incidence.signature-primary, incidence.output-class, incidence.shape,
   incidence.signs]
2. `incidence(digraph(3, 1))` is `[1;0;-1]` (as a sparse 3×1).
   [incidence.signs]
3. Two parallel edges 1–2 give two identical columns `[-1;1]`.
   [incidence.multigraph]
4. Weights `[5 7]` on the path 1–2–3 do not change the entries.
   [incidence.weights-ignored]
5. Four nodes with one edge 1–2 give a 4×1 matrix `[-1;1;0;0]`; the empty
   graph gives 0×0; three nodes and no edges give 3×0. [incidence.shape]
6. Column `j` corresponds to row `j` of the graph's stored `G.Edges`.
   [incidence.edge-order]
7. A self-loop in a graph or digraph raises `RunMat:incidence:SelfLoops`.
   [incidence.error-self-loop-graph]
8. A numeric-matrix input raises `RunMat:incidence:UndefinedFunction`.
   [incidence.error-matrix]

### User Story 2 - Isomorphism test (Priority: P1)

A RunMat user calling `tf = isisomorphic(G1, G2)` gets a 1×1 logical that
says whether the two graphs (or two digraphs) are isomorphic.

**Acceptance Scenarios** (normative):

1. True for identical graphs, relabelled graphs (undirected, directed, with
   a self-loop, a multigraph), weights 5 vs 7, and two empty graphs.
   [isisomorphic.signature-primary, isisomorphic.output-class,
   isisomorphic.true-cases, isisomorphic.weights-ignored]
2. False for path vs star, different node counts (including isolated
   nodes), different edge counts, self-loop vs none, self-loop vs parallel
   edge, multigraph vs simple graph. [isisomorphic.false-cases]
3. Digraph chains 1→2→3 and 1→2←3 are not isomorphic.
   [isisomorphic.direction]
4. A graph with a digraph raises `RunMat:isisomorphic:ClassMismatch`.
   [isisomorphic.error-mixed]
5. Numeric-matrix inputs raise `RunMat:isisomorphic:UndefinedFunction`.
   [isisomorphic.error-matrix]

### User Story 3 - Unresolved forms fail clearly (Priority: P2)

1. `isisomorphic(G1, G2, Name, Value...)` and `[tf, P] = isisomorphic(...)`
   raise `RunMat:isisomorphic:UnsupportedOption`. *(isisomorphic.q-options,
   isisomorphic.q-second-output)*
2. `incidence(G, extra)` raises `RunMat:incidence:UnsupportedOption`.
   *(U-036-1)*

## Requirements *(mandatory)*

### Functional Requirements

- **FR-036-01** `incidence(G)`, for `G` a `graph` or `digraph`, MUST return
  a sparse matrix of class `double` [incidence.signature-primary,
  incidence.output-class] with one row per node (isolated nodes included)
  and one column per edge [incidence.shape].
- **FR-036-02** Column `j` MUST describe edge `j` in the graph's stored edge
  order (row `j` of `G.Edges.EndNodes`) [incidence.edge-order], holding −1
  in the row of `EndNodes(j,1)` and +1 in the row of `EndNodes(j,2)`, for
  both graph and digraph [incidence.signs]. Parallel edges each get their
  own column [incidence.multigraph]. Edge weights do not affect the entries
  [incidence.weights-ignored]. No warning is issued [incidence.no-warning].
- **FR-036-03** A graph or digraph containing a self-loop MUST raise
  `RunMat:incidence:SelfLoops` [incidence.error-self-loop-graph; observed
  identifier mapped into the RunMat namespace].
- **FR-036-04** A non-graph input (e.g. a numeric matrix) MUST raise
  `RunMat:incidence:UndefinedFunction` [incidence.error-matrix; observed
  identifier mapped into the RunMat namespace].
- **FR-036-05** `isisomorphic(G1, G2)` MUST return a 1×1 `logical`
  [isisomorphic.signature-primary, isisomorphic.output-class]: `true` exactly
  when some bijection of nodes maps the edge multiset of `G1` onto that of
  `G2` (self-loops and parallel edges compared by multiplicity), `false`
  otherwise [isisomorphic.true-cases, isisomorphic.false-cases]. For two
  digraphs edge direction is preserved [isisomorphic.direction]. Edge
  weights (and node names) are not compared [isisomorphic.weights-ignored].
  No warning is issued [isisomorphic.no-warning].
- **FR-036-06** A graph together with a digraph MUST raise
  `RunMat:isisomorphic:ClassMismatch` [isisomorphic.error-mixed].
- **FR-036-07** A non-graph input MUST raise
  `RunMat:isisomorphic:UndefinedFunction` [isisomorphic.error-matrix].
- **FR-036-08** (unresolved — isisomorphic.q-options,
  isisomorphic.q-second-output) More than two inputs, or more than one
  requested output, MUST raise `RunMat:isisomorphic:UnsupportedOption`.
- **FR-036-09** (documented choices — U-036-1, U-036-6) `incidence` with
  more than one input raises `RunMat:incidence:UnsupportedOption`;
  `incidence` with no input, `isisomorphic` with fewer than two inputs, or a
  malformed graph object raise `RunMat:<fn>:InvalidInput`.
- **FR-036-10** (documented choice — incidence.q-named-nodes) Graphs with
  named nodes are accepted; rows follow node order, exactly as for unnamed
  nodes. (RunMat's graph objects carry no extra edge variables besides
  `Weight`, which FR-036-02 already ignores.)
- **FR-036-11** (algorithm, documented) `isisomorphic` uses a backtracking
  search over node bijections, pruned by node count, edge count and sorted
  per-node (out-degree, in-degree, self-loop) invariants.

### Key Entities

- Input: `Value::Object` of class `graph` or `digraph` (upstream graph
  subsystem), read with upstream's `graph_data` (node count and stored edge
  list in `G.Edges` order).
- `incidence` output: `Value::SparseTensor` (real `double`).
- `isisomorphic` output: `Value::Bool`.

## Success Criteria *(mandatory)*

- **SC-036-1**: every observed case reproduces in a normative test, except
  the three listed under deviation D-036-1 (validation.md), which depend on
  the upstream graph constructor's edge order.
- **SC-036-2**: both builtins are registered through `#[runtime_builtin]`,
  and `cargo test -p runmat-runtime --lib -- graph_structure_tests` passes.
- **SC-036-3**: every FR maps to at least one test (validation.md
  traceability table).
- **SC-036-4**: `cargo test -p runmat-runtime --lib -- builtins::graph`
  shows no regressions.

## Unresolved behaviour (non-normative; returned to the specification side)

| id | question | disposition |
|---|---|---|
| incidence.q-warning-coverage | warnings checked only in `warn-call` | no warning is emitted by the implementation (by construction) |
| incidence.q-named-nodes | named nodes / extra edge variables | documented choice FR-036-10 |
| isisomorphic.q-warning-coverage | as above | as above |
| isisomorphic.q-options | name-value options | `UnsupportedOption` (FR-036-08) |
| isisomorphic.q-second-output | node permutation output | `UnsupportedOption` (FR-036-08) |
| U-036-1 (local) | extra arguments to `incidence` | `UnsupportedOption` |
| U-036-6 (local) | missing inputs / malformed graph object | `InvalidInput` |

Superseded 0.2.0-era items: incidence.q-shape and incidence.q-directed
(resolved by 0.3.0: digraphs supported), isisomorphic.q-nonisomorphic
(resolved), U-036-2 (self-loops now `SelfLoops`), U-036-3 (multiplicities
now observed), U-036-4 (mixed classes now `ClassMismatch`), U-036-5 (edge
order now normative; see D-036-1).

## Known deviation

- **D-036-1** The observed values of cases `edge-order`,
  `directed-edge-order` and `undirected-reversed-input` rely on the graph
  constructor storing undirected edges with the smaller node first and
  sorting edges (observed `G.Edges.EndNodes` `[1 2; 2 3]`, `[1 2; 3 1]`,
  `[1 3]`). RunMat's upstream `graph`/`digraph` keep the input order and
  endpoints. `incidence` follows RunMat's stored `G.Edges` (the normative
  relation in incidence.edge-order / incidence.signs), so for unsorted input
  its columns differ from the observed values. Fixing this belongs to the
  graph constructor, outside this feature's scope; flagged to the
  maintainer.
- **D-036-2** The observed `graph(1, 2, 1, 4)` (node-count argument) is not
  accepted by RunMat's upstream graph constructor; tests use the equivalent
  adjacency-matrix construction.

## Assumptions

- Upstream's graph subsystem supplies the graph types and `graph_data`;
  upstream's `SparseTensor` supplies the sparse output. No new runtime
  types are needed.
