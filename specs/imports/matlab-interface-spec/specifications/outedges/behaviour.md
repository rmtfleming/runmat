# `outedges` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N5, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/outedges_observations.json`](observations/outedges_observations.json)

## Summary

Return the indices of the edges leaving a node and, as a second output, the nodes those edges reach.

## Confirmed invocation forms

- `[eid, nid] = outedges(G, node)` — Edge indices and successor nodes (COBRA form). *(e.g. case `two-out`)*
- `eid = outedges(G, node)` — Edge indices only. *(e.g. case `one-out`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — For G = digraph([1 1 2 3], [3 2 3 1]), whose edge table is [1 2; 1 3; 2 3; 3 1], node 1 gave eid = [1; 2] and nid = [2; 3], and node 3 gave eid = 4: eid lists edge-table rows in ascending order and nid the matching target nodes. *(cases: `edge-table`, `two-out`, `one-out`)*
2. **Confirmed** — A node with no out-edges, including an isolated node, gave 0x1 double eid and nid. *(cases: `no-out`, `isolated`)*
3. **Confirmed** — With named nodes, eid is double; nid was a cell array of char for a char query, a string array for a string query, and double indices for an index query. *(cases: `names-char`, `names-string`, `names-index`)*
4. **Confirmed** — Parallel edges each appear (nid [2; 2; 3]) and a self-loop is listed with the node itself as nid. *(cases: `multigraph`, `self-loop`)*
5. **Confirmed** — For an undirected graph([1 2], [2 3]), node 2 gave eid = [1; 2] and nid = [1; 3] (both incident edges). *(cases: `undirected`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| The node index exceeds the number of nodes. | `MATLAB:graphfun:digraph:InvalidNodeID` | `err-range` |
| More than one node is given. | `MATLAB:graphfun:graphbuiltin:InvalidNodeScalar` | `err-vector` |
| The name is not a node name. | `MATLAB:graphfun:digraph:UnknownNodeName` | `err-unknown-name` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `edge-table` | `double` | `[4, 2]` | [1 2;1 3;2 3;3 1] |
| `two-out` | `struct` | `[1, 1]` | eid=[1;2]; nid=[2;3]; class_eid='double'; class_nid='double'; size_eid=[2 1]; size_nid=[2 1] |
| `one-out` | `double` | `[1, 1]` | 4 |
| `no-out` | `struct` | `[1, 1]` | eid=zeros(0,1); nid=zeros(0,1); class_eid='double'; class_nid='double'; size_eid=[0 1]; size_nid=[0 1] |
| `isolated` | `struct` | `[1, 1]` | eid=zeros(0,1); nid=zeros(0,1); class_eid='double'; class_nid='double'; size_eid=[0 1]; size_nid=[0 1] |
| `names-char` | `struct` | `[1, 1]` | eid=[1;2]; nid=["'b'", "'c'"]; class_eid='double'; class_nid='cell'; size_eid=[2 1]; size_nid=[2 1] |
| `names-string` | `struct` | `[1, 1]` | eid=[1;2]; nid=["b";"c"]; class_eid='double'; class_nid='string'; size_eid=[2 1]; size_nid=[2 1] |
| `names-index` | `struct` | `[1, 1]` | eid=[1;2]; nid=[2;3]; class_eid='double'; class_nid='double'; size_eid=[2 1]; size_nid=[2 1] |
| `multigraph` | `struct` | `[1, 1]` | eid=[1;2;3]; nid=[2;2;3]; class_eid='double'; class_nid='double'; size_eid=[3 1]; size_nid=[3 1] |
| `self-loop` | `struct` | `[1, 1]` | eid=[1;2]; nid=[1;2]; class_eid='double'; class_nid='double'; size_eid=[2 1]; size_nid=[2 1] |
| `undirected` | `struct` | `[1, 1]` | eid=[1;2]; nid=[1;3]; class_eid='double'; class_nid='double'; size_eid=[2 1]; size_nid=[2 1] |
| `err-range` | — | — | error `MATLAB:graphfun:digraph:InvalidNodeID` |
| `err-vector` | — | — | error `MATLAB:graphfun:graphbuiltin:InvalidNodeScalar` |
| `err-unknown-name` | — | — | error `MATLAB:graphfun:digraph:UnknownNodeName` |

## Unresolved questions

- `outedges.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `outedges.q-undirected-ids` — Only digraph error identifiers were observed; identifiers for graph inputs were not exercised.
