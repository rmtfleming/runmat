# Feature Specification: `isomorphism`, `allpaths`, `inedges` — graph queries (N3/N4)

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`059-graph-n3n4`; export `SPECIFY_FEATURE=059-graph-n3n4`)

**Created**: 2026-10-30

**Status**: Batches N3+N4 (F59). Gates G1–G3 pre-approved by the maintainer
(`specs/001-clean-room-matlab-builtins/selection-proposal-n3n4.md`).

**Input**: Approved exports `isomorphism` 0.2.0 (N3, `1156c97`), `allpaths`
0.2.0 and `inedges` 0.2.0 (N4, `fc9f804`) (see `provenance.md`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Node permutation between isomorphic graphs (Priority: P1)

`P = isomorphism(G1, G2)` returns a permutation mapping the nodes of `G1`
onto those of `G2`, or an empty array when the graphs are not isomorphic;
`[P, edgeperm] = isomorphism(G1, G2)` also returns an edge permutation.

**Acceptance Scenarios** (normative):

1. For `graph([1 2 3 4 3 5], [2 3 4 5 5 6])` and
   `graph([6 5 4 3 4 2], [5 4 3 2 2 1])` (either argument order) `P` is the
   6×1 double `[6 5 4 3 2 1]'`, the only valid permutation.
   [isomorphism.signature-primary, isomorphism.output-class,
   isomorphism.orientation]
2. `P` is oriented so that `G2` with its nodes reordered by `P` has the
   adjacency of `G1`, i.e. `A1(i, j) == A2(P(i), P(j))`; for the digraphs
   `1→2→3` and `3→1→2`, `P = [3 1 2]'`. [isomorphism.orientation]
3. A path compared with itself gives a 3×1 double valid permutation
   (which one is not required). [isomorphism.output-class,
   isomorphism.non-unique (non-normative)]
4. Path vs star, different node counts, and digraphs differing in edge
   direction give a 0×0 double. [isomorphism.not-isomorphic]
5. Two empty graphs give a 1×0 double. [isomorphism.empty-graphs]
6. `[P, edgeperm]` for the pair in (1) gives a 6×1 double `edgeperm`
   `[1 2 3 4 5 6]'`. [isomorphism.edgeperm]
7. With `'NodeVariables', 'Met'` a path labelled `[10 20 30]` against its
   reversal labelled `[30 20 10]` gives `[3 2 1]'`; labels `[10 30 20]`
   give an empty result. [isomorphism.node-variables]
8. With `'EdgeVariables', 'Weight'` the path with weights `[5 7]` against
   the one with `[7 5]` gives `[3 2 1]'`. [isomorphism.edge-variables]
9. A graph with a digraph raises `RunMat:isomorphism:ClassMismatch`; a
   `NodeVariables` name that the first graph lacks raises
   `RunMat:isomorphism:NodeVarNotInFirstGraph`. [isomorphism.error-mixed,
   isomorphism.error-unknown-nodevar]

### User Story 2 - All paths between two nodes (Priority: P1)

`P = allpaths(G, s, t)` returns every path from `s` to `t` (no repeated
node) as a cell column; `[P, E] = allpaths(G, s, t)` also returns the edge
indices of each path.

**Acceptance Scenarios** (normative):

1. The path 1–2–3 gives `{[1 2 3]}`; the 4-cycle 1–2–3–4–1 from 1 to 3
   gives the set `{[1 2 3]; [1 4 3]}`. [allpaths.signature-primary,
   allpaths.paths]
2. The digraph `1→2, 2→3, 1→3` gives the set `{[1 2 3]; [1 3]}`; from 3 to
   1 on `1→2→3` the result is a 0×1 cell. [allpaths.digraph]
3. `s == t` gives `{s}`, one one-node path. [allpaths.same-node]
4. A target in another component gives a 0×1 cell. [allpaths.unreachable]
5. Named nodes give paths as cellstr rows (`{'a','b','c'}`, `{'a','c'}`).
   [allpaths.names]
6. On the 4-cycle with edges stored `[1 2; 1 4; 2 3; 3 4]` the second
   output is `{[1 3]; [2 4]}`, paired with `{[1 2 3]; [1 4 3]}`.
   [allpaths.edgepaths] (stored edge order: D-059-2)
7. An index out of range raises `RunMat:allpaths:InvalidNodeID`; an unknown
   name raises `RunMat:allpaths:UnknownNodeName`.
   [allpaths.error-node-range, allpaths.error-unknown-name]

### User Story 3 - In-edges of a digraph node (Priority: P1)

`eid = inedges(G, node)` returns the indices of the edges entering `node`;
`[eid, nid] = inedges(G, node)` also returns their source nodes.

**Acceptance Scenarios** (normative):

1. Edges `1→3, 2→3, 3→4`: `inedges(G, 3)` gives `eid = [1;2]`,
   `nid = [1;2]`. [inedges.signature-primary, inedges.outputs]
2. In-edges are listed in stored edge order with their sources (edges
   stored `1→2, 3→2, 4→2` give `eid = [1;2;3]`, `nid = [1;3;4]`).
   [inedges.order] (stored edge order: D-059-1)
3. A node with no in-edges gives 0×1 double `eid` and 0×1 `nid`.
   [inedges.none]
4. Named nodes give `nid` as a 2×1 cell of names (`{'a';'b'}`) and `eid` a
   2×1 double. [inedges.names]
5. A self-loop counts as an in-edge. [inedges.selfloop]
6. An undirected graph raises `RunMat:inedges:NoInEdgesUndir`; an index
   out of range `RunMat:inedges:InvalidNodeID`; more than one node
   `RunMat:inedges:InvalidNodeScalar`. [inedges.error-undirected,
   inedges.error-node-range, inedges.error-two-nodes]

### User Story 4 - Unresolved forms fail clearly (Priority: P2)

Every form the exports do not observe either raises
`RunMat:<fn>:UnsupportedOption` or follows a documented choice (FR-059-17),
tagged `unresolved_choice_*` in tests.

## Requirements *(mandatory)*

### Functional Requirements

#### isomorphism

- **FR-059-01** `isomorphism(G1, G2)` MUST return a `double` column vector
  `P` with `numnodes` entries when the graphs are isomorphic
  [isomorphism.signature-primary, isomorphism.output-class].
- **FR-059-02** `P` MUST satisfy `A1(i, j) == A2(P(i), P(j))` for all
  `i, j` (edge multiplicities, edge direction for digraphs), i.e.
  reordering `G2` by `P` gives the adjacency of `G1`
  [isomorphism.orientation]. When the valid permutation is unique (cases
  `asym`, `asym-swapped`, `digraph`, `nodevars-match`, `edgevars`) the
  exact observed value follows.
- **FR-059-03** Non-isomorphic inputs (including different node counts and
  differing edge direction) MUST give a 0×0 `double`
  [isomorphism.not-isomorphic]; two graphs with no nodes MUST give a 1×0
  `double` [isomorphism.empty-graphs].
- **FR-059-04** `[P, edgeperm]` MUST return `edgeperm` as a `double`
  column with `numedges` entries [isomorphism.edgeperm]. Its definition
  (U-059-5) reproduces the observed `[1..6]'`.
- **FR-059-05** `'NodeVariables', names` MUST restrict the result to
  permutations that map every node to a node with equal values of the
  named node variables; no such permutation gives an empty result
  [isomorphism.node-variables]. `'EdgeVariables', names` MUST likewise
  require each edge to be mapped to an edge with equal values
  [isomorphism.edge-variables].
- **FR-059-06** A `graph` with a `digraph` (either order) MUST raise
  `RunMat:isomorphism:ClassMismatch` [isomorphism.error-mixed]; a
  `NodeVariables` name absent from the first graph MUST raise
  `RunMat:isomorphism:NodeVarNotInFirstGraph`
  [isomorphism.error-unknown-nodevar].

#### allpaths

- **FR-059-07** `allpaths(G, s, t)` MUST return an n×1 cell whose elements
  are the node sequences (1×k `double` rows) of all paths from `s` to `t`
  without a repeated node; the set of paths is normative, their order is
  not [allpaths.signature-primary, allpaths.paths, allpaths.order
  (non-normative)].
- **FR-059-08** Digraphs MUST follow edge direction; no path gives a 0×1
  cell [allpaths.digraph, allpaths.unreachable].
- **FR-059-09** `s == t` MUST give a 1×1 cell holding the one-node path
  [allpaths.same-node].
- **FR-059-10** On graphs with node names each path MUST be a 1×k cellstr
  row of names, and `s`/`t` MAY be given as names [allpaths.names].
- **FR-059-11** `[P, E] = allpaths(G, s, t)` MUST return `E` as an n×1 cell
  of 1×(k−1) `double` rows of stored edge indices, `E{i}` being the edges
  of `P{i}` [allpaths.edgepaths].
- **FR-059-12** An out-of-range index MUST raise
  `RunMat:allpaths:InvalidNodeID`; an unknown name
  `RunMat:allpaths:UnknownNodeName` [allpaths.error-node-range,
  allpaths.error-unknown-name].

#### inedges

- **FR-059-13** `inedges(G, node)` on a digraph MUST return the indices of
  the stored edges whose target is `node`, ascending, as a k×1 `double`
  column; `[eid, nid]` MUST also return the source of each such edge, in
  the same order [inedges.signature-primary, inedges.outputs,
  inedges.order].
- **FR-059-14** No in-edges MUST give 0×1 `double` `eid` and 0×1 `nid`
  [inedges.none]; a self-loop MUST count as an in-edge [inedges.selfloop].
- **FR-059-15** On graphs with node names `nid` MUST be a k×1 cell of
  names (char rows) and `node` MAY be a name [inedges.names].
- **FR-059-16** An undirected graph MUST raise
  `RunMat:inedges:NoInEdgesUndir` [inedges.error-undirected]; an
  out-of-range index `RunMat:inedges:InvalidNodeID`
  [inedges.error-node-range]; more than one node
  `RunMat:inedges:InvalidNodeScalar` [inedges.error-two-nodes].

#### Common

- **FR-059-17** (documented choices, local ids; not MATLAB requirements)
  - **U-059-1** A first input (for `isomorphism`, either input) that is
    not a `graph`/`digraph` raises `RunMat:<fn>:UndefinedFunction`
    (matches the non-normative observations isomorphism.non-graph and
    allpaths.non-graph; a choice for `inedges`).
  - **U-059-2** Missing required inputs raise `RunMat:<fn>:InvalidInput`;
    extra positional inputs (`allpaths` with more than 3, `inedges` with
    more than 2) and unknown `isomorphism` option names raise
    `RunMat:<fn>:UnsupportedOption` (allpaths.q-more).
  - **U-059-3** More than two requested outputs raise
    `RunMat:<fn>:TooManyOutputs`.
  - **U-059-4** `isomorphism` search and options:
    (a) when several permutations are valid the first one found by a
    deterministic backtracking search is returned (isomorphism.non-unique);
    (b) edge weights are ignored unless named in `EdgeVariables`;
    (c) parallel edges and self-loops are matched as multisets
    (isomorphism.q-more);
    (d) option names are case-insensitive, an option value is one name or a
    list of names (all must match), a repeated option replaces the earlier
    one, unpaired or non-text option names raise `InvalidInput`;
    (e) a `NodeVariables` name present in `G1` but absent from `G2`, and an
    `EdgeVariables` name absent from either graph, raise `InvalidInput`;
    (f) RunMat's synthetic `Index` node column of unnamed graphs is not a
    node variable (`NodeVarNotInFirstGraph`); `EndNodes` is not accepted as
    an edge variable (`InvalidInput`); `'Weight'` is always available and
    reads the stored weights (unit weights when the `Edges` table has no
    `Weight` column, U-043-9);
    (g) supported variable types: real numeric or logical columns (one row
    per node/edge, any number of columns), string arrays, and cells of text
    or numeric scalars; other types raise `UnsupportedOption`; `NaN` never
    equals anything;
    (h) all option checks happen before the node-count comparison.
  - **U-059-5** `edgeperm(k)` is the index of the `G2` edge onto which `G1`
    edge `k` is mapped by `P` (parallel edges paired lowest index first).
    When not isomorphic `edgeperm` is 0×0; for two empty graphs 1×0
    (mirroring `P`).
  - **U-059-6** `allpaths`: paths are enumerated depth-first taking
    neighbours in increasing node index (matches the non-normative
    observed order); on a multigraph each node sequence is reported once,
    with the lowest-index parallel edge in `E`; self-loops are never part
    of a path; for `s == t`, `E` is `{zeros(1, 0)}`; a non-scalar `s`/`t`
    raises `InvalidInput`; a non-positive or non-integer index raises
    `InvalidNodeID`; a name on a graph without names raises
    `UnknownNodeName`.
  - **U-059-7** `inedges`: an undirected graph raises `NoInEdgesUndir` for
    any output count (observed with two); an unknown name, or a name on a
    graph without names, raises `UnknownNodeName`; a non-positive or
    non-integer index raises `InvalidNodeID`; an empty node list raises
    `InvalidNodeScalar`; `nid` on a named graph with no in-edges is a 0×1
    cell; the graph-class check precedes node validation.
- **FR-059-18** (unresolved, returned to the specification side)
  `*.q-warning-coverage`: no warnings are emitted. `isomorphism.q-more`,
  `allpaths.q-more`: handled by documented choices U-059-4(c), U-059-2,
  U-059-6; nothing is asserted about MATLAB behaviour for them.

### Key Entities

- Input: `Value::Object` of class `graph`/`digraph`, read with upstream's
  `graph_data`; node/edge variables are read from the object's `Nodes` /
  `Edges` tables.
- Outputs: `double` tensors, `Value::Cell` columns (paths, names).

## Success Criteria *(mandatory)*

- **SC-059-1** Every observed case reproduces in a normative test, except
  values that depend on the upstream constructor's edge order
  (D-059-1, D-059-2), which are kept in `#[ignore]`d tests, with the same
  behaviour asserted on an equivalently stored graph.
- **SC-059-2** The three builtins are registered through
  `#[runtime_builtin]`; `cargo test -p runmat-runtime --lib -- graph_n3n4`
  passes.
- **SC-059-3** Every FR maps to at least one test (validation.md).
- **SC-059-4** `cargo test -p runmat-runtime --lib -- builtins::graph` shows
  no regressions.

## Unresolved behaviour (non-normative)

| id | disposition |
|---|---|
| isomorphism.non-unique | U-059-4(a); tests assert validity, not the exact permutation, where several exist |
| allpaths.order | U-059-6; tests compare path sets |
| isomorphism.non-graph, allpaths.non-graph | U-059-1 |
| isomorphism.q-more | U-059-4(c), U-059-4(d) |
| allpaths.q-more | U-059-2, U-059-6 |
| *.q-warning-coverage | no warnings emitted (FR-059-18) |

## Known deviations

- **D-059-1** (same cause as D-036-1/D-043-1) RunMat's `graph`/`digraph`
  constructor keeps edges in input order; the observed constructor sorts
  them. For `inedges(digraph([4 1 3], [2 2 2]), 2)` RunMat stores
  `4→2, 1→2, 3→2` and returns `eid = [1;2;3]` (matches) but
  `nid = [4;1;3]` (observed `[1;3;4]`). The relation eid ↔ stored edge is
  the same; the observed value is reproduced on `digraph([1 3 4], [2 2 2])`
  and kept in an ignored test.
- **D-059-2** (same cause) For `allpaths(graph([1 2 3 1], [2 3 4 4]), 1, 3)`
  RunMat stores edge 4 as `1–4` after `3–4` and returns
  `E = {[1 2]; [4 3]}` (observed `{[1 3]; [2 4]}`); the node paths match.
  The observed value is reproduced on `graph([1 1 2 3], [2 4 3 4])` and
  kept in an ignored test.
- **D-059-3** RunMat's graph constructors create no node variables other
  than `Name` (and the synthetic `Index`). `isomorphism` reads any variable
  present in the object's `Nodes` table; the `Met` cases are tested on
  objects whose `Nodes` table was rebuilt with a `Met` column (the observed
  set-up assigned the variable after construction; whether RunMat's
  language-level `G.Nodes.Met = …` assignment works is not part of this
  feature).

## Assumptions

- Upstream's graph subsystem supplies `GraphData`, `graph_data`,
  `graph_object` and the table helpers; `graph()` with no arguments gives
  the empty graph.
