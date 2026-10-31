# `reordernodes` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N5, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/reordernodes_observations.json`](observations/reordernodes_observations.json)

## Summary

Return a graph of the same class whose nodes appear in a given order, with the node table and edges carried along, and optionally the edge permutation.

## Confirmed invocation forms

- `H = reordernodes(G, order)` — Reorder the nodes (COBRA form, order from isomorphism). *(e.g. case `met-graph`)*
- `[H, idx] = reordernodes(G, order)` — Also return the edge permutation. *(e.g. case `idx-out`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — Node k of H is node order(k) of G: with node variables Met and Atom and order [3 1 2], H.Nodes became Met {'m1','m1','m2'}, Atom [3; 1; 2], keeping the variable names and classes (COBRA form). *(cases: `met-graph-before`, `met-graph`)*
2. **Confirmed** — Edges follow their end nodes, renumbered, and H.Edges is listed in ascending order of end nodes: for the cycle 1->2->3->1 and order [3 1 2] the end nodes stayed [1 2; 2 3; 3 1] and idx was [3; 1; 2]. *(cases: `met-graph`, `idx-out`)*
3. **Confirmed** — Weights move with their edges: digraph([1 2], [2 3], [7 9]) with order [2 3 1] gave end nodes [1 2; 3 1], Weight [9; 7] and idx [2; 1]. *(cases: `weighted`)*
4. **Confirmed** — An undirected graph([1 2], [2 3]) with order [3 2 1] gave end nodes [1 2; 2 3]. *(cases: `undirected`)*
5. **Confirmed** — order given as node names {'c','a','b'} reorders the Name variable to {'c';'a';'b'}; end nodes were reported by name. *(cases: `names`)*
6. **Confirmed** — The identity order leaves the edges unchanged, and the output class is that of G (digraph). *(cases: `identity`, `class`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| order repeats a node or omits nodes. | `MATLAB:graphfun:reordernodes:InvalidOrder` | `err-repeat`, `err-short` |
| order contains an index beyond the node count. | `MATLAB:graphfun:digraph:InvalidNodeID` | `err-range` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `met-graph-before` | `struct` | `[1, 1]` | nodes={'variable_names': ['Met', 'Atom'], 'variables': [{'name': 'Met', 'class': 'cell', 'size': [3, 1], 'value': ["'m1'", "'m2'", "'m1'"]}, {'name': 'Atom', 'class': 'double', 'size': [3, 1], 'value': '[1;2;3]'}]}; endnodes=[1 2;2 3;3 1] |
| `met-graph` | `struct` | `[1, 1]` | nodes={'variable_names': ['Met', 'Atom'], 'variables': [{'name': 'Met', 'class': 'cell', 'size': [3, 1], 'value': ["'m1'", "'m1'", "'m2'"]}, {'name': 'Atom', 'class': 'double', 'size': [3, 1], 'value': '[3;1;2]'}]}; endnodes=[1 2;2 3;3 1] |
| `idx-out` | `double` | `[3, 1]` | [3;1;2] |
| `weighted` | `struct` | `[1, 1]` | endnodes=[1 2;3 1]; weight=[9;7]; idx=[2;1] |
| `undirected` | `struct` | `[1, 1]` | nodes={'variable_names': [], 'variables': []}; endnodes=[1 2;2 3] |
| `names` | `struct` | `[1, 1]` | nodes={'variable_names': ['Name'], 'variables': [{'name': 'Name', 'class': 'cell', 'size': [3, 1], 'value': ["'c'", "'a'", "'b'"]}]}; endnodes=["'a'", "'b'", "'b'", "'c'"] |
| `identity` | `struct` | `[1, 1]` | nodes={'variable_names': [], 'variables': []}; endnodes=[1 2;2 3] |
| `class` | `char` | `[1, 7]` | 'digraph' |
| `err-repeat` | — | — | error `MATLAB:graphfun:reordernodes:InvalidOrder` |
| `err-short` | — | — | error `MATLAB:graphfun:reordernodes:InvalidOrder` |
| `err-range` | — | — | error `MATLAB:graphfun:digraph:InvalidNodeID` |

## Unresolved questions

- `reordernodes.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
