# `isomorphism` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N3, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/isomorphism_observations.json`](observations/isomorphism_observations.json)

## Summary

Return a node permutation mapping one graph onto another when they are isomorphic, and an empty array otherwise.

## Confirmed invocation forms

- `P = isomorphism(G1, G2)` — Node permutation, or empty. *(e.g. case `asym`)*
- `[P, edgeperm] = isomorphism(G1, G2)` — Also an edge permutation. *(e.g. case `edgeperm`)*
- `P = isomorphism(G1, G2, Name, Value)` — NodeVariables / EdgeVariables must also match (COBRA form). *(e.g. case `nodevars-match`)*

## Name-value arguments

- `NodeVariables` — Node table variable whose values must be preserved by the mapping (one variable observed).
- `EdgeVariables` — Edge table variable whose values must be preserved ('Weight' observed).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — P is a double column vector with numnodes entries. *(cases: `asym`, `identical`, `digraph`)*
2. **Confirmed** — reordernodes(G2, P) had the same adjacency as G1 in every case where this was checked; for the digraph case (1->2->3 vs 3->1->2, P = [3 1 2]) the other direction, reordernodes(G1, P) = G2, did not hold. *(cases: `asym`, `asym-swapped`, `identical`, `digraph`, `edgevars`)*
3. **Confirmed** — Non-isomorphic graphs (path vs star, different node counts, digraphs differing in edge direction) give a 0x0 double. *(cases: `not-iso`, `node-count`, `digraph-direction`)*
4. **Confirmed** — Two empty graphs give a 1x0 double. *(cases: `empty-graphs`)*
5. **Confirmed** — [P, edgeperm] returns a double column with numedges entries ([1..6]' for the asymmetric 6-node pair). *(cases: `edgeperm`)*
6. **Confirmed** — With NodeVariables 'Met', a mapping that preserves Met is returned ([3 2 1] for a path whose labels are reversed); when no mapping preserves the labels the result is empty. *(cases: `nodevars-match`, `nodevars-mismatch`)*
7. **Confirmed** — With EdgeVariables 'Weight', a path with weights [5 7] and one with [7 5] gave P = [3 2 1], the mapping that pairs equal weights (the weights themselves were not compared by the harness). *(cases: `edgevars`)*

## Edge cases (observed)

- *(non-normative)* Observed: numeric inputs errored with MATLAB:UndefinedFunction (dispatch level; not required). *(cases: `err-not-graph`)*
- *(non-normative)* Inference (not observed): when several permutations are valid, which one is returned should not be required. The path 1-2-3 compared with itself has two valid permutations and [1 2 3] was returned; the asymmetric 6-node pair has exactly one. *(cases: `identical`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| One input is a graph and the other a digraph. | `MATLAB:graphfun:isomorphism:ClassMismatch` | `err-mixed` |
| NodeVariables names a variable that neither graph has. | `MATLAB:graphfun:isomorphism:NodeVarNotInFirstGraph` | `err-unknown-nodevar` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `asym` | `struct` | `[1, 1]` | P=[6 5 4 3 2 1]; class='double'; size=[6 1]; reorder_G2_by_P_equals_G1=true; reorder_G1_by_P_equals_G2=true |
| `asym-swapped` | `struct` | `[1, 1]` | P=[6 5 4 3 2 1]; class='double'; size=[6 1]; reorder_G2_by_P_equals_G1=true; reorder_G1_by_P_equals_G2=true |
| `identical` | `struct` | `[1, 1]` | P=[1 2 3]; class='double'; size=[3 1]; reorder_G2_by_P_equals_G1=true; reorder_G1_by_P_equals_G2=true |
| `not-iso` | `struct` | `[1, 1]` | class='double'; size=[0 0] |
| `node-count` | `struct` | `[1, 1]` | class='double'; size=[0 0] |
| `digraph` | `struct` | `[1, 1]` | P=[3 1 2]; class='double'; size=[3 1]; reorder_G2_by_P_equals_G1=true; reorder_G1_by_P_equals_G2=false |
| `digraph-direction` | `struct` | `[1, 1]` | class='double'; size=[0 0] |
| `edgeperm` | `struct` | `[1, 1]` | P=[6 5 4 3 2 1]; edgeperm=[1 2 3 4 5 6]; edgeperm_class='double'; edgeperm_size=[6 1] |
| `nodevars-match` | `struct` | `[1, 1]` | P=[3 2 1]; size=[3 1] |
| `nodevars-mismatch` | `struct` | `[1, 1]` | P=[]; size=[0 0] |
| `edgevars` | `struct` | `[1, 1]` | P=[3 2 1]; class='double'; size=[3 1]; reorder_G2_by_P_equals_G1=true; reorder_G1_by_P_equals_G2=true |
| `empty-graphs` | `struct` | `[1, 1]` | class='double'; size=[1 0] |
| `err-mixed` | — | — | error `MATLAB:graphfun:isomorphism:ClassMismatch` |
| `err-not-graph` | — | — | error `MATLAB:UndefinedFunction` |
| `err-unknown-nodevar` | — | — | error `MATLAB:graphfun:isomorphism:NodeVarNotInFirstGraph` |

## Unresolved questions

- `isomorphism.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `isomorphism.q-more` — Multigraphs, self-loops and multiple node/edge variables were not exercised.
