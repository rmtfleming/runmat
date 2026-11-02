# `findnode` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-11-02 (batch N6, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/findnode_observations.json`](observations/findnode_observations.json)

## Summary

Return the node indices for node names (0 for a name that is not a node), or for numeric node IDs, as a double column vector.

## Confirmed invocation forms

- `idx = findnode(G, nodeID)` — Indices for names (COBRA form, nodeID a 1x2 cell row of G.Edges.EndNodes) or numeric IDs. *(e.g. case `cell-row`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — idx is a double column vector in every non-error case. *(cases: `cell-row`, `char`, `string-array`, `cell-column`, `cell-matrix`, `numeric-named`)*
2. **Confirmed** — For G = digraph({'a','b','c'}, {'b','c','d'}), the 1x2 cell row {'b','c'} of G.Edges.EndNodes gave [2; 3]. *(cases: `node-names`, `cell-row`)*
3. **Confirmed** — Names are matched to node positions: 'c' gave 3, ["d" "a"] gave [4; 1], {'b'; 'a'} gave [2; 1]; a 2x2 cell is read in column-major order ({'a' 'b'; 'c' 'd'} gave [1; 3; 2; 4]); an undirected graph behaves the same. *(cases: `char`, `string-array`, `cell-column`, `cell-matrix`, `undirected`)*
4. **Confirmed** — A name that is not a node gives 0 in its position (also for a name differing only in case); repeated names give repeated indices; an empty cell gives a 0x1 result. *(cases: `unknown`, `mixed-unknown`, `case-sensitive`, `repeated`, `empty-cell`)*
5. **Confirmed** — Positive integer IDs are returned as a column of the same indices ([3 1] gave [3; 1]) on named and unnamed graphs; an index beyond the node count gives 0 (observed on both). *(cases: `numeric-named`, `numeric-unnamed`, `numeric-range`, `numeric-range-named`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| A char or string name is given for a graph without node names. | `MATLAB:graphfun:findnode:NoNames` | `err-unnamed-char`, `err-unnamed-string` |
| nodeID is logical, a cell holding a number, or a 2-row char matrix. | `MATLAB:graphfun:findnode:ArgType` | `err-logical`, `err-cell-numeric`, `err-char-matrix` |
| A numeric ID is 0, negative or non-integer (0, -1, 1.5). | `MATLAB:graphfun:findnode:PosInt` | `err-numeric-zero`, `err-numeric-negative`, `err-numeric-fraction` |
| Only G is given. | `MATLAB:minrhs` | `err-one-arg` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `node-names` | `cell` | `[4, 1]` | ["'a'", "'b'", "'c'", "'d'"] |
| `cell-row` | `struct` | `[1, 1]` | query=["'b'", "'c'"]; idx=[2;3]; class='double'; size=[2 1] |
| `char` | `double` | `[1, 1]` | 3 |
| `string-array` | `double` | `[2, 1]` | [4;1] |
| `cell-column` | `double` | `[2, 1]` | [2;1] |
| `cell-matrix` | `double` | `[4, 1]` | [1;3;2;4] |
| `unknown` | `double` | `[1, 1]` | 0 |
| `mixed-unknown` | `double` | `[3, 1]` | [1;0;3] |
| `repeated` | `double` | `[2, 1]` | [2;2] |
| `case-sensitive` | `double` | `[1, 1]` | 0 |
| `empty-cell` | `double` | `[0, 1]` | zeros(0,1) |
| `numeric-named` | `double` | `[2, 1]` | [3;1] |
| `numeric-unnamed` | `double` | `[2, 1]` | [2;3] |
| `numeric-range` | `double` | `[1, 1]` | 0 |
| `numeric-range-named` | `double` | `[1, 1]` | 0 |
| `err-numeric-zero` | — | — | error `MATLAB:graphfun:findnode:PosInt` |
| `err-numeric-negative` | — | — | error `MATLAB:graphfun:findnode:PosInt` |
| `err-numeric-fraction` | — | — | error `MATLAB:graphfun:findnode:PosInt` |
| `err-char-matrix` | — | — | error `MATLAB:graphfun:findnode:ArgType` |
| `err-unnamed-string` | — | — | error `MATLAB:graphfun:findnode:NoNames` |
| `undirected` | `double` | `[2, 1]` | [3;1] |
| `err-unnamed-char` | — | — | error `MATLAB:graphfun:findnode:NoNames` |
| `err-logical` | — | — | error `MATLAB:graphfun:findnode:ArgType` |
| `err-one-arg` | — | — | error `MATLAB:minrhs` |
| `err-cell-numeric` | — | — | error `MATLAB:graphfun:findnode:ArgType` |

## Unresolved questions

- `findnode.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `findnode.q-query-types` — Only the query types listed were exercised; for example, categorical or a cell mixing names and empty char were not.
