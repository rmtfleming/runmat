# `addnode` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N2, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/addnode_observations.json`](observations/addnode_observations.json)

## Summary

Return a graph or digraph with new nodes added, given as a count or as names.

## Confirmed invocation forms

- `H = addnode(G, nodeCount)` — Add unnamed nodes. *(e.g. case `count`)*
- `H = addnode(G, nodeNames)` — Add named nodes. *(e.g. case `names`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — A count adds that many nodes at the end and leaves the edges unchanged (2 nodes added to a 3-node path gave 5 nodes and the same 2 edges); 0 adds none. *(cases: `count`, `zero`, `digraph`, `empty`)*
2. **Confirmed** — Names add nodes with those names at the end (a char name or a cellstr). *(cases: `names`, `name-char`, `names-to-unnamed-empty`)*
3. **Confirmed** — Adding the name 'x' to a 2-node graph with unnamed nodes gave node names Node1, Node2, x. *(cases: `names-to-unnamed`)*
4. **Confirmed** — The result keeps the input class. *(cases: `count`, `digraph`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| A name is already used by a node. | `MATLAB:graphfun:graph:NonUniqueNames` | `err-dup-name` |
| The count is negative. | `MATLAB:graphfun:addnode:InvalidNrNodes` | `err-negative` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `count` | `struct` | `[1, 1]` | class='graph'; numnodes=5; numedges=2; EndNodes=[1 2;2 3] |
| `zero` | `struct` | `[1, 1]` | class='graph'; numnodes=3; numedges=2; EndNodes=[1 2;2 3] |
| `names` | `struct` | `[1, 1]` | class='graph'; numnodes=4; numedges=1; EndNodes=["'a'", "'b'"]; NodeNames=["'a'", "'b'", "'c'", "'d'"] |
| `name-char` | `struct` | `[1, 1]` | class='graph'; numnodes=3; numedges=1; EndNodes=["'a'", "'b'"]; NodeNames=["'a'", "'b'", "'c'"] |
| `digraph` | `struct` | `[1, 1]` | class='digraph'; numnodes=3; numedges=1; EndNodes=[1 2] |
| `empty` | `struct` | `[1, 1]` | class='graph'; numnodes=3; numedges=0; EndNodes=zeros(0,2) |
| `names-to-unnamed-empty` | `struct` | `[1, 1]` | class='graph'; numnodes=2; numedges=0; EndNodes=[]; NodeNames=["'p'", "'q'"] |
| `err-dup-name` | — | — | error `MATLAB:graphfun:graph:NonUniqueNames` |
| `names-to-unnamed` | `struct` | `[1, 1]` | class='graph'; numnodes=3; numedges=1; EndNodes=["'Node1'", "'Node2'"]; NodeNames=["'Node1'", "'Node2'", "'x'"] |
| `err-negative` | — | — | error `MATLAB:graphfun:addnode:InvalidNrNodes` |

## Unresolved questions

- `addnode.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `addnode.q-graph-variables` — Node inputs other than a count, a char name and a cellstr were not exercised.
