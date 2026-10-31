# `allpaths` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N4, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/allpaths_observations.json`](observations/allpaths_observations.json)

## Summary

Return the paths between two nodes of a graph or digraph as a cell column of node sequences (no node repeated in any observed path).

## Confirmed invocation forms

- `P = allpaths(G, s, t)` — All paths from s to t (COBRA form). *(e.g. case `cycle`)*
- `[P, E] = allpaths(G, s, t)` — Also the edge indices of each path. *(e.g. case `edgepaths`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — On the 4-cycle 1-2-3-4-1, allpaths(G, 1, 3) gave {[1 2 3]; [1 4 3]}; on a path 1-2-3 it gave {[1 2 3]}. *(cases: `cycle`, `path`)*
2. **Confirmed** — Digraphs follow edge direction: with edges 1->2, 2->3, 1->3 the result was {[1 2 3]; [1 3]}; from 3 to 1 on 1->2->3 it was a 0x1 cell. *(cases: `digraph`, `digraph-reverse`)*
3. **Confirmed** — When s equals t (node 2 of the 4-cycle, the one case tried) the result was {2}, a single one-node path. *(cases: `same-node`)*
4. **Confirmed** — When t was in another component (the one case tried) the result was a 0x1 cell. *(cases: `unreachable`)*
5. **Confirmed** — With named nodes each path is a cellstr row of names ({'a','b','c'}; {'a','c'}). *(cases: `names`)*
6. **Confirmed** — On the 4-cycle 1-2-3-4-1, the second output was {[1 3]; [2 4]} (edge indices; the edge table itself was not recorded). *(cases: `edgepaths`)*

## Edge cases (observed)

- *(non-normative)* Observed: a numeric first argument errored with MATLAB:UndefinedFunction (dispatch level; not required). *(cases: `err-not-graph`)*
- *(non-normative)* Observed path order: on the observed graphs, paths through lower-numbered next nodes came first; whether this order is required is not established. *(cases: `cycle`, `digraph`, `names`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| A node index is out of range. | `MATLAB:graphfun:graph:InvalidNodeID` | `err-node-range` |
| A node name is unknown. | `MATLAB:graphfun:graph:UnknownNodeName` | `err-unknown-name` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `path` | `cell` | `[1, 1]` | ['[1 2 3]'] |
| `cycle` | `cell` | `[2, 1]` | ['[1 2 3]', '[1 4 3]'] |
| `digraph` | `cell` | `[2, 1]` | ['[1 2 3]', '[1 3]'] |
| `digraph-reverse` | `cell` | `[0, 1]` | [] |
| `same-node` | `cell` | `[1, 1]` | ['2'] |
| `unreachable` | `cell` | `[0, 1]` | [] |
| `names` | `cell` | `[2, 1]` | [["'a'", "'b'", "'c'"], ["'a'", "'c'"]] |
| `edgepaths` | `struct` | `[1, 1]` | paths=["[1 2 3]", "[1 4 3]"]; edgepaths=["[1 3]", "[2 4]"] |
| `err-node-range` | — | — | error `MATLAB:graphfun:graph:InvalidNodeID` |
| `err-unknown-name` | — | — | error `MATLAB:graphfun:graph:UnknownNodeName` |
| `err-not-graph` | — | — | error `MATLAB:UndefinedFunction` |

## Unresolved questions

- `allpaths.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `allpaths.q-more` — Calls with more than three arguments, multigraphs and self-loops were not exercised.
