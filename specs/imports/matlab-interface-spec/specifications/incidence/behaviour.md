# `incidence` — behaviour specification

- **Spec version:** 0.3.0
- **Status:** approved 2026-10-29 (Tier B widening W2, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/incidence_observations.json`](observations/incidence_observations.json)

## Summary

Return the sparse node-by-edge incidence matrix of a graph or digraph.

## Confirmed invocation forms

- `I = incidence(G)` — Incidence matrix of graph or digraph G. *(e.g. case `basic`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — The output is a sparse double matrix, for both graph and digraph input. *(cases: `sparse-class`, `directed-sparse-class`, `empty-graph`, `nodes-no-edges`)*
2. **Confirmed** — The matrix has one row per node and one column per edge, isolated nodes included (graph(1,2,1,4) gave 4x1; three nodes with no edges gave 3x0; the empty graph gave 0x0). *(cases: `sparse-class`, `isolated-node`, `nodes-no-edges`, `empty-graph`)*
3. **Confirmed** — Each column holds -1 in the row of the edge's first end node and +1 in the row of its second end node, for both graph and digraph: digraph edge 3->1 gave +1 at node 1 and -1 at node 3; graph(3,1) stored its edge as EndNodes [1 3] and gave -1 at node 1 and +1 at node 3. *(cases: `basic`, `directed`, `directed-reverse`, `undirected-reversed-input`, `reversed-input-table`)*
4. **Confirmed** — An ordinary call issued no warning. *(cases: `warn-call`)*
5. **Confirmed** — Columns follow the graph's stored edge order (G.Edges.EndNodes): graph([2 1],[3 2]) stored [1 2; 2 3] and digraph([3 1],[1 2]) stored [1 2; 3 1], and the columns matched those orders. *(cases: `edge-order`, `edge-order-table`, `directed-edge-order`, `directed-edge-order-table`)*
6. **Confirmed** — Parallel edges each get their own column (two edges 1-2 gave two identical columns). *(cases: `multigraph`)*
7. **Confirmed** — Edge weights do not affect the entries, which stay -1 and +1. *(cases: `weighted`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| The graph or digraph contains a self-loop. | `MATLAB:graphfun:incidence:SelfLoops` | `self-loop-graph`, `self-loop-digraph` |
| The input is a numeric matrix rather than a graph object. | `MATLAB:UndefinedFunction` | `err-matrix` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `basic` | `double` | `[3, 2]` | [-1 0;1 -1;0 1] |
| `sparse-class` | `double` | `[3, 2]` | sparse([1;2;2;3], [1;1;2;2], [-1;1;-1;1], 3, 2) |
| `directed` | `double` | `[3, 2]` | [-1 0;1 -1;0 1] |
| `directed-sparse-class` | `double` | `[3, 2]` | sparse([1;2;2;3], [1;1;2;2], [-1;1;-1;1], 3, 2) |
| `reversed-input-table` | `double` | `[1, 2]` | [1 3] |
| `warn-call` | `struct` | `[1, 1]` | value=[-1 0;1 -1;0 1]; warning_id='' |
| `directed-reverse` | `double` | `[3, 1]` | [1;0;-1] |
| `undirected-reversed-input` | `double` | `[3, 1]` | [-1;0;1] |
| `edge-order` | `double` | `[3, 2]` | [-1 0;1 -1;0 1] |
| `edge-order-table` | `double` | `[2, 2]` | [1 2;2 3] |
| `directed-edge-order` | `double` | `[3, 2]` | [-1 1;1 0;0 -1] |
| `directed-edge-order-table` | `double` | `[2, 2]` | [1 2;3 1] |
| `self-loop-graph` | — | — | error `MATLAB:graphfun:incidence:SelfLoops` |
| `self-loop-digraph` | — | — | error `MATLAB:graphfun:incidence:SelfLoops` |
| `multigraph` | `double` | `[2, 2]` | [-1 -1;1 1] |
| `weighted` | `double` | `[3, 2]` | [-1 0;1 -1;0 1] |
| `isolated-node` | `double` | `[4, 1]` | [-1;1;0;0] |
| `empty-graph` | `double` | `[0, 0]` | sparse([], [], [], 0, 0) |
| `nodes-no-edges` | `double` | `[3, 0]` | sparse(zeros(0,1), zeros(0,1), zeros(0,1), 3, 0) |
| `err-matrix` | — | — | error `MATLAB:UndefinedFunction` |

## Unresolved questions

- `incidence.q-warning-coverage` — Warnings were captured only in the warn-* cases; other cases (including error and edge cases) were not checked for warnings, so an empty warnings list is not a complete statement.
- `incidence.q-named-nodes` — Graphs with named nodes or extra edge variables were not exercised.

Resolved in 0.3.0: `incidence.q-shape`, `incidence.q-directed`.
