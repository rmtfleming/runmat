# `inedges` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N4, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/inedges_observations.json`](observations/inedges_observations.json)

## Summary

Return the indices of the edges entering a node of a digraph and, as a second output, their source nodes.

## Confirmed invocation forms

- `eid = inedges(G, node)` — Incoming edge indices. *(e.g. case `one-out`)*
- `[eid, nid] = inedges(G, node)` — Also the source nodes (COBRA form). *(e.g. case `two-out`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — For edges 1->3, 2->3, 3->4, inedges(G, 3) gave eid [1;2] and nid [1;2]. *(cases: `one-out`, `two-out`)*
2. **Confirmed** — For edges added as 4->2, 1->2, 3->2, the result was eid [1;2;3] and nid [1;3;4] (the edge table itself was not recorded). *(cases: `order`)*
3. **Confirmed** — A node with no in-edges gives 0x1 double eid and 0x1 nid. *(cases: `none`)*
4. **Confirmed** — With named nodes nid is a 2x1 cell of names ({'a';'b'}) and eid a 2x1 double. *(cases: `names`)*
5. **Confirmed** — A self-loop counts as an in-edge (node 1 with edges 1->1 and 2->1 gave eid [1;2], nid [1;2]). *(cases: `selfloop`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| G is an undirected graph (called with two outputs). | `MATLAB:graphfun:graph:NoInEdgesUndir` | `err-undirected` |
| The node index is out of range. | `MATLAB:graphfun:digraph:InvalidNodeID` | `err-node-range` |
| More than one node is given. | `MATLAB:graphfun:graphbuiltin:InvalidNodeScalar` | `err-two-nodes` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `one-out` | `double` | `[2, 1]` | [1;2] |
| `two-out` | `struct` | `[1, 1]` | eid=[1;2]; nid=[1;2] |
| `order` | `struct` | `[1, 1]` | eid=[1;2;3]; nid=[1;3;4] |
| `none` | `struct` | `[1, 1]` | eid=zeros(0,1); nid=zeros(0,1); eid_class='double'; eid_size=[0 1]; nid_class='double'; nid_size=[0 1] |
| `names` | `struct` | `[1, 1]` | eid=[1;2]; nid=["'a'", "'b'"]; eid_class='double'; eid_size=[2 1]; nid_class='cell'; nid_size=[2 1] |
| `selfloop` | `struct` | `[1, 1]` | eid=[1;2]; nid=[1;2] |
| `err-undirected` | — | — | error `MATLAB:graphfun:graph:NoInEdgesUndir` |
| `err-node-range` | — | — | error `MATLAB:graphfun:digraph:InvalidNodeID` |
| `err-two-nodes` | — | — | error `MATLAB:graphfun:graphbuiltin:InvalidNodeScalar` |

## Unresolved questions

- `inedges.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
