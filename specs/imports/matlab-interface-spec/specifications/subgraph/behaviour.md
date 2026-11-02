# `subgraph` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-11-02 (batch N6, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/subgraph_observations.json`](observations/subgraph_observations.json)

## Summary

Return the graph induced by a set of nodes: the selected nodes with their node-table rows, and every edge whose two end nodes are both selected, renumbered to the new node positions.

## Confirmed invocation forms

- `H = subgraph(G, nodeIDs)` — Induced subgraph for a logical mask (COBRA form subgraph(ATG, mask)), a vector of indices (COBRA form subgraph(BG, idx)) or a cell of names. *(e.g. case `logical-mask`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — H has the class of G: digraph for a digraph and graph for a graph. *(cases: `class`, `undirected`)*
2. **Confirmed** — A logical mask keeps the selected nodes in their original order; indices or names keep them in the order given ([3 1 2] gave node-table rows c, a, b; {'c','a'} gave c, a). *(cases: `logical-mask`, `logical-column`, `index-column`, `index-unsorted`, `names`)*
3. **Confirmed** — All node-table variables (Met, Atom; or Name) are carried over for the selected rows. *(cases: `source-graph`, `logical-mask`, `index-unsorted`, `names`)*
4. **Confirmed** — H contains exactly the edges of G with both end nodes selected, with EndNodes renumbered to positions in H and edge-table variables (rxns, Weight) carried over. For the 4-cycle with edges r1..r4, the mask [1 1 1 0] kept r1, r2 as [1 2; 2 3]; [1 0 1 1] kept r3, r4 as [2 3; 3 1]; [3 1 2] kept r1, r2 as [2 3; 3 1]. *(cases: `source-graph`, `logical-mask`, `logical-column`, `index-column`, `index-unsorted`, `weighted`)*
5. **Confirmed** — The edge table of H is sorted by its renumbered EndNodes rows, not kept in G's edge order: indices [4 1 2] gave nodes d, a, b and edges r4, r1 as [1 2; 2 3] (G's order would give r1, r4 as [2 3; 1 2]); this is the one case that separates the two orders. *(cases: `edge-order`, `logical-column`, `index-unsorted`, `multigraph`)*
6. **Confirmed** — Parallel edges and self-loops between selected nodes are kept (EndNodes [1 1; 1 2; 1 2]). *(cases: `multigraph`)*
7. **Confirmed** — A selected node with no kept edge stays in H (2 nodes, 0x2 EndNodes). *(cases: `isolated-node`)*
8. **Confirmed** — An all-false mask or an empty index vector gives a graph with 0 nodes and 0 edges that keeps the node and edge variable names. *(cases: `all-false`, `empty-index`)*
9. **Confirmed** — With named nodes, EndNodes of H is a cell array of names. *(cases: `names`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| A node index is repeated, or a logical mask is shorter or longer than the number of nodes. | `MATLAB:graphfun:subgraph:InvalidInd` | `err-repeat`, `err-mask-length`, `err-mask-longer` |
| A node index exceeds the number of nodes (digraph). | `MATLAB:graphfun:digraph:InvalidNodeID` | `err-range` |
| A name is not a node name (digraph). | `MATLAB:graphfun:digraph:UnknownNodeName` | `err-unknown-name` |
| Only G is given. | `MATLAB:minrhs` | `err-one-arg` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `source-graph` | `struct` | `[1, 1]` | class='digraph'; numnodes=4; nodes={'variable_names': ['Met', 'Atom'], 'variables': [{'name': 'Met', 'class': 'cell', 'size': [4, 1], 'value': ["'a'", "'b'", "'c'", "'d'"]}, {'name': 'Atom', 'class': 'double', 'size': [4, 1], 'value': '[1;2;3;4]'}]}; edges={'variable_names': ['EndNodes', 'rxns'], 'variables': [{'name': 'EndNodes', 'class': 'double', 'size': [4, 2], 'value': '[1 2;2 3;3 4;4 1]'}, {'name': 'rxns', 'class': 'cell', 'size': [4, 1], 'value': ["'r1'", "'r2'", "'r3'", "'r4'"]}]} |
| `logical-mask` | `struct` | `[1, 1]` | class='digraph'; numnodes=3; nodes={'variable_names': ['Met', 'Atom'], 'variables': [{'name': 'Met', 'class': 'cell', 'size': [3, 1], 'value': ["'a'", "'b'", "'c'"]}, {'name': 'Atom', 'class': 'double', 'size': [3, 1], 'value': '[1;2;3]'}]}; edges={'variable_names': ['EndNodes', 'rxns'], 'variables': [{'name': 'EndNodes', 'class': 'double', 'size': [2, 2], 'value': '[1 2;2 3]'}, {'name': 'rxns', 'class': 'cell', 'size': [2, 1], 'value': ["'r1'", "'r2'"]}]} |
| `logical-column` | `struct` | `[1, 1]` | class='digraph'; numnodes=3; nodes={'variable_names': ['Met', 'Atom'], 'variables': [{'name': 'Met', 'class': 'cell', 'size': [3, 1], 'value': ["'a'", "'c'", "'d'"]}, {'name': 'Atom', 'class': 'double', 'size': [3, 1], 'value': '[1;3;4]'}]}; edges={'variable_names': ['EndNodes', 'rxns'], 'variables': [{'name': 'EndNodes', 'class': 'double', 'size': [2, 2], 'value': '[2 3;3 1]'}, {'name': 'rxns', 'class': 'cell', 'size': [2, 1], 'value': ["'r3'", "'r4'"]}]} |
| `index-column` | `struct` | `[1, 1]` | class='digraph'; numnodes=2; nodes={'variable_names': ['Met', 'Atom'], 'variables': [{'name': 'Met', 'class': 'cell', 'size': [2, 1], 'value': ["'b'", "'c'"]}, {'name': 'Atom', 'class': 'double', 'size': [2, 1], 'value': '[2;3]'}]}; edges={'variable_names': ['EndNodes', 'rxns'], 'variables': [{'name': 'EndNodes', 'class': 'double', 'size': [1, 2], 'value': '[1 2]'}, {'name': 'rxns', 'class': 'cell', 'size': [1, 1], 'value': ["'r2'"]}]} |
| `index-unsorted` | `struct` | `[1, 1]` | class='digraph'; numnodes=3; nodes={'variable_names': ['Met', 'Atom'], 'variables': [{'name': 'Met', 'class': 'cell', 'size': [3, 1], 'value': ["'c'", "'a'", "'b'"]}, {'name': 'Atom', 'class': 'double', 'size': [3, 1], 'value': '[3;1;2]'}]}; edges={'variable_names': ['EndNodes', 'rxns'], 'variables': [{'name': 'EndNodes', 'class': 'double', 'size': [2, 2], 'value': '[2 3;3 1]'}, {'name': 'rxns', 'class': 'cell', 'size': [2, 1], 'value': ["'r1'", "'r2'"]}]} |
| `edge-order` | `struct` | `[1, 1]` | class='digraph'; numnodes=3; nodes={'variable_names': ['Met', 'Atom'], 'variables': [{'name': 'Met', 'class': 'cell', 'size': [3, 1], 'value': ["'d'", "'a'", "'b'"]}, {'name': 'Atom', 'class': 'double', 'size': [3, 1], 'value': '[4;1;2]'}]}; edges={'variable_names': ['EndNodes', 'rxns'], 'variables': [{'name': 'EndNodes', 'class': 'double', 'size': [2, 2], 'value': '[1 2;2 3]'}, {'name': 'rxns', 'class': 'cell', 'size': [2, 1], 'value': ["'r4'", "'r1'"]}]} |
| `err-mask-longer` | — | — | error `MATLAB:graphfun:subgraph:InvalidInd` |
| `all-false` | `struct` | `[1, 1]` | class='digraph'; numnodes=0; nodes={'variable_names': ['Met', 'Atom'], 'variables': [{'name': 'Met', 'class': 'cell', 'size': [0, 1], 'value': []}, {'name': 'Atom', 'class': 'double', 'size': [0, 1], 'value': 'zeros(0,1)'}]}; edges={'variable_names': ['EndNodes', 'rxns'], 'variables': [{'name': 'EndNodes', 'class': 'double', 'size': [0, 2], 'value': 'zeros(0,2)'}, {'name': 'rxns', 'class': 'cell', 'size': [0, 1], 'value': []}]} |
| `empty-index` | `struct` | `[1, 1]` | class='digraph'; numnodes=0; nodes={'variable_names': ['Met', 'Atom'], 'variables': [{'name': 'Met', 'class': 'cell', 'size': [0, 1], 'value': []}, {'name': 'Atom', 'class': 'double', 'size': [0, 1], 'value': 'zeros(0,1)'}]}; edges={'variable_names': ['EndNodes', 'rxns'], 'variables': [{'name': 'EndNodes', 'class': 'double', 'size': [0, 2], 'value': 'zeros(0,2)'}, {'name': 'rxns', 'class': 'cell', 'size': [0, 1], 'value': []}]} |
| `names` | `struct` | `[1, 1]` | class='digraph'; numnodes=2; nodes={'variable_names': ['Name'], 'variables': [{'name': 'Name', 'class': 'cell', 'size': [2, 1], 'value': ["'c'", "'a'"]}]}; edges={'variable_names': ['EndNodes'], 'variables': [{'name': 'EndNodes', 'class': 'cell', 'size': [1, 2], 'value': ["'c'", "'a'"]}]} |
| `weighted` | `struct` | `[1, 1]` | class='digraph'; numnodes=2; nodes={'variable_names': [], 'variables': []}; edges={'variable_names': ['EndNodes', 'Weight'], 'variables': [{'name': 'EndNodes', 'class': 'double', 'size': [1, 2], 'value': '[1 2]'}, {'name': 'Weight', 'class': 'double', 'size': [1, 1], 'value': '5'}]} |
| `multigraph` | `struct` | `[1, 1]` | class='digraph'; numnodes=2; nodes={'variable_names': [], 'variables': []}; edges={'variable_names': ['EndNodes'], 'variables': [{'name': 'EndNodes', 'class': 'double', 'size': [3, 2], 'value': '[1 1;1 2;1 2]'}]} |
| `undirected` | `struct` | `[1, 1]` | class='graph'; numnodes=2; nodes={'variable_names': [], 'variables': []}; edges={'variable_names': ['EndNodes'], 'variables': [{'name': 'EndNodes', 'class': 'double', 'size': [1, 2], 'value': '[1 2]'}]} |
| `class` | `char` | `[1, 7]` | 'digraph' |
| `isolated-node` | `struct` | `[1, 1]` | class='digraph'; numnodes=2; nodes={'variable_names': [], 'variables': []}; edges={'variable_names': ['EndNodes'], 'variables': [{'name': 'EndNodes', 'class': 'double', 'size': [0, 2], 'value': 'zeros(0,2)'}]} |
| `err-repeat` | — | — | error `MATLAB:graphfun:subgraph:InvalidInd` |
| `err-range` | — | — | error `MATLAB:graphfun:digraph:InvalidNodeID` |
| `err-mask-length` | — | — | error `MATLAB:graphfun:subgraph:InvalidInd` |
| `err-unknown-name` | — | — | error `MATLAB:graphfun:digraph:UnknownNodeName` |
| `err-one-arg` | — | — | error `MATLAB:minrhs` |

## Unresolved questions

- `subgraph.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `subgraph.q-tie-order` — The order of parallel edges, which tie after sorting, was not established; the sorted order rests on one case where it differs from G's order.
- `subgraph.q-undirected-ids` — Only digraph error identifiers were observed; identifiers for graph inputs were not exercised.
