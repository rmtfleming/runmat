# `rmedge` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-29 (batch N1, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/rmedge_observations.json`](observations/rmedge_observations.json)

## Summary

Return a graph or digraph with the specified edges removed; nodes are kept.

## Confirmed invocation forms

- `H = rmedge(G, s, t)` — Remove edges between s(i) and t(i). *(e.g. case `endpoints`)*
- `H = rmedge(G, idx)` — Remove edges by index into the edge list. *(e.g. case `index`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — Removing by endpoints deletes the matching edge; for graphs the endpoint order does not matter (2,3 and 3,2 both removed edge 2-3). *(cases: `endpoints`, `reversed`)*
2. **Confirmed** — With two arguments the second is a vector of edge indices ([1 3] on path 1-2-3-4 left edge [2 3]). *(cases: `index`)*
3. **Confirmed** — Removing by endpoints in a multigraph removes every parallel edge between them (two 1-2 edges both removed). *(cases: `multigraph`)*
4. **Confirmed** — For digraphs direction matters: removing 2->1 when only 1->2 exists left the graph unchanged; removing 1->2 removed it. *(cases: `digraph-direction`, `digraph`)*
5. **Confirmed** — Endpoints with no edge between them leave the graph unchanged, without error. *(cases: `absent`, `digraph-direction`)*
6. **Confirmed** — Nodes were not removed, including a node that lost its last edge. *(cases: `keeps-nodes`, `endpoints`)*
7. **Confirmed** — Remaining edges keep their weights. *(cases: `weighted`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| An edge index exceeds numedges. | `MATLAB:graphfun:findedge:EdgeBounds` | `err-index-range` |
| A node index exceeds numnodes. | `MATLAB:graphfun:graph:InvalidNodeID` | `err-node-range` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `endpoints` | `struct` | `[1, 1]` | class='graph'; numnodes=4; numedges=2; EndNodes=[1 2;3 4] |
| `reversed` | `struct` | `[1, 1]` | class='graph'; numnodes=4; numedges=2; EndNodes=[1 2;3 4] |
| `index` | `struct` | `[1, 1]` | class='graph'; numnodes=4; numedges=1; EndNodes=[2 3] |
| `multigraph` | `struct` | `[1, 1]` | class='graph'; numnodes=3; numedges=1; EndNodes=[2 3] |
| `digraph-direction` | `struct` | `[1, 1]` | class='digraph'; numnodes=3; numedges=2; EndNodes=[1 2;2 3] |
| `digraph` | `struct` | `[1, 1]` | class='digraph'; numnodes=3; numedges=1; EndNodes=[2 3] |
| `absent` | `struct` | `[1, 1]` | class='graph'; numnodes=4; numedges=3; EndNodes=[1 2;2 3;3 4] |
| `keeps-nodes` | `struct` | `[1, 1]` | class='graph'; numnodes=2; numedges=0; EndNodes=zeros(0,2) |
| `weighted` | `struct` | `[1, 1]` | class='graph'; numnodes=3; numedges=1; EndNodes=[2 3]; Weight=6 |
| `err-index-range` | — | — | error `MATLAB:graphfun:findedge:EdgeBounds` |
| `err-node-range` | — | — | error `MATLAB:graphfun:graph:InvalidNodeID` |

## Unresolved questions

- `rmedge.q-warning-coverage` — Warnings were not captured by this harness; an empty warnings list is not a complete statement.
- `rmedge.q-graph-variables` — Graphs with extra node or edge variables (beyond Name and Weight) were not exercised.
