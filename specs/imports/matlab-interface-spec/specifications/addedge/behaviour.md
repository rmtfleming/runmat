# `addedge` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-29 (batch N1, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/addedge_observations.json`](observations/addedge_observations.json)

## Summary

Return a graph or digraph with new edges added, creating nodes as needed.

## Confirmed invocation forms

- `H = addedge(G, s, t)` — Add edges s(i)-t(i). *(e.g. case `one`)*
- `H = addedge(G, s, t, w)` — Add weighted edges. *(e.g. case `weighted`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — In every observed case the resulting edge list was sorted by end nodes, with undirected edges stored smaller node first: adding 1-3 to path 1-2-3 gave [1 2; 1 3; 2 3]; adding 3-1 and 1-2 gave [1 2; 1 2; 1 3; 2 3]. *(cases: `one`, `order`)*
2. **Confirmed** — Digraph edges kept their direction and the list was sorted by source then target: adding 3->1 and 2->1 to 1->2 gave [1 2; 2 1; 3 1]. *(cases: `digraph`)*
3. **Confirmed** — Adding an edge that already exists creates a parallel edge (multigraph). *(cases: `order`)*
4. **Confirmed** — An endpoint beyond numnodes creates the missing nodes (edge 2-5 on a 2-node graph gave 5 nodes); adding to an empty graph creates the nodes. *(cases: `new-node`, `empty-graph`)*
5. **Confirmed** — Weights follow their edges after sorting (adding 2-3 weight 8 and 1-3 weight 9 to 1-2 weight 7 gave weights [7; 9; 8] for edges [1 2; 1 3; 2 3]); a scalar weight applies to every new edge. *(cases: `weighted`, `weight-scalar`)*
6. **Confirmed** — Named endpoints are accepted, and a new name creates a node with that name. *(cases: `named`)*
7. **Confirmed** — A self-loop can be added. *(cases: `self-loop`)*
8. **Confirmed** — A scalar t expands against a vector s (s = [1 2], t = 3 added edges 1-3 and 2-3). *(cases: `scalar-expansion`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| A weight is given for a graph without weights. | `MATLAB:graphfun:addedge:NoWeights` | `err-weight-unweighted` |
| s and t are vectors of different lengths. | `MATLAB:graphfun:graphbuiltin:EqualNumel` | `err-length` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `one` | `struct` | `[1, 1]` | class='graph'; numnodes=3; numedges=3; EndNodes=[1 2;1 3;2 3] |
| `order` | `struct` | `[1, 1]` | class='graph'; numnodes=3; numedges=4; EndNodes=[1 2;1 2;1 3;2 3] |
| `new-node` | `struct` | `[1, 1]` | class='graph'; numnodes=5; numedges=2; EndNodes=[1 2;2 5] |
| `weighted` | `struct` | `[1, 1]` | class='graph'; numnodes=3; numedges=3; EndNodes=[1 2;1 3;2 3]; Weight=[7;9;8] |
| `weight-scalar` | `struct` | `[1, 1]` | class='graph'; numnodes=4; numedges=3; EndNodes=[1 2;2 3;3 4]; Weight=[7;5;5] |
| `digraph` | `struct` | `[1, 1]` | class='digraph'; numnodes=3; numedges=3; EndNodes=[1 2;2 1;3 1] |
| `named` | `struct` | `[1, 1]` | class='graph'; numnodes=3; numedges=2; EndNodes=["'a'", "'b'", "'b'", "'c'"]; NodeNames=["'a'", "'b'", "'c'"] |
| `self-loop` | `struct` | `[1, 1]` | class='graph'; numnodes=2; numedges=2; EndNodes=[1 2;2 2] |
| `empty-graph` | `struct` | `[1, 1]` | class='graph'; numnodes=2; numedges=1; EndNodes=[1 2] |
| `err-weight-unweighted` | — | — | error `MATLAB:graphfun:addedge:NoWeights` |
| `scalar-expansion` | `struct` | `[1, 1]` | class='graph'; numnodes=3; numedges=3; EndNodes=[1 2;1 3;2 3] |
| `err-length` | — | — | error `MATLAB:graphfun:graphbuiltin:EqualNumel` |

## Unresolved questions

- `addedge.q-warning-coverage` — Warnings were not captured by this harness; an empty warnings list is not a complete statement.
- `addedge.q-graph-variables` — Graphs with extra node or edge variables (beyond Name and Weight) were not exercised.
