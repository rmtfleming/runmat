# `rmnode` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-29 (batch N1, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/rmnode_observations.json`](observations/rmnode_observations.json)

## Summary

Return a graph or digraph with the specified nodes and all their incident edges removed.

## Confirmed invocation forms

- `H = rmnode(G, nodeIDs)` — Remove nodes given by index or name. *(e.g. case `one`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — The nodes and every edge touching them are removed and the remaining nodes are renumbered consecutively in their original order: removing node 2 from path 1-2-3-4 left 3 nodes and the single edge [2 3] (formerly 3-4); removing [1 4] left 2 nodes and edge [1 2]. *(cases: `one`, `several`)*
2. **Confirmed** — The result keeps the input class (graph or digraph). *(cases: `one`, `digraph`)*
3. **Confirmed** — Named nodes can be removed by name; the remaining names are kept (removing 'b' from a,b,c left a and c with edge a-c). *(cases: `named`)*
4. **Confirmed** — Remaining edges keep their weights. *(cases: `weighted`, `isolated`)*
5. **Confirmed** — An empty node list returns the graph unchanged. *(cases: `empty-list`)*
6. **Confirmed** — Removing every node gives a graph with 0 nodes and 0 edges. *(cases: `all`)*
7. **Confirmed** — Removing an isolated node reduces numnodes and leaves the edges unchanged. *(cases: `isolated`)*

## Edge cases (observed)

- *(non-normative)* Observed: a node index beyond numnodes (7 on a 4-node graph) and an unknown node name ('z') did not error and returned the graph unchanged. *(cases: `out-of-range`, `unknown-name`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `one` | `struct` | `[1, 1]` | class='graph'; numnodes=3; numedges=1; EndNodes=[2 3] |
| `several` | `struct` | `[1, 1]` | class='graph'; numnodes=2; numedges=1; EndNodes=[1 2] |
| `named` | `struct` | `[1, 1]` | class='graph'; numnodes=2; numedges=1; EndNodes=["'a'", "'c'"]; NodeNames=["'a'", "'c'"] |
| `weighted` | `struct` | `[1, 1]` | class='graph'; numnodes=3; numedges=1; EndNodes=[1 2]; Weight=10 |
| `digraph` | `struct` | `[1, 1]` | class='digraph'; numnodes=2; numedges=1; EndNodes=[1 2] |
| `isolated` | `struct` | `[1, 1]` | class='graph'; numnodes=2; numedges=1; EndNodes=[1 2]; Weight=1 |
| `empty-list` | `struct` | `[1, 1]` | class='graph'; numnodes=4; numedges=3; EndNodes=[1 2;2 3;3 4] |
| `all` | `struct` | `[1, 1]` | class='graph'; numnodes=0; numedges=0; EndNodes=zeros(0,2) |
| `out-of-range` | `struct` | `[1, 1]` | class='graph'; numnodes=4; numedges=3; EndNodes=[1 2;2 3;3 4] |
| `unknown-name` | `struct` | `[1, 1]` | class='graph'; numnodes=2; numedges=2; EndNodes=["'a'", "'a'", "'b'", "'b'"]; NodeNames=["'a'", "'b'"] |

## Unresolved questions

- `rmnode.q-warning-coverage` — Warnings were not captured by this harness; an empty warnings list is not a complete statement.
- `rmnode.q-graph-variables` — Graphs with extra node or edge variables (beyond Name and Weight) were not exercised.
- `rmnode.q-ignored-ids` — Flagged for human review (constitution 8.2): whether out-of-range or unknown node identifiers should be silently ignored (cases out-of-range, unknown-name) or rejected by conforming implementations.
