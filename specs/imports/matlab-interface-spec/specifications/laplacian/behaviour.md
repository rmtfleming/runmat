# `laplacian` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-29 (batch N1, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/laplacian_observations.json`](observations/laplacian_observations.json)

## Summary

Return the sparse graph Laplacian (degree matrix minus adjacency) of an undirected simple graph.

## Confirmed invocation forms

- `L = laplacian(G)` — Laplacian of graph G. *(e.g. case `path`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — The output is a sparse double numnodes-by-numnodes matrix. *(cases: `class`, `empty`)*
2. **Confirmed** — L = D - A with D the diagonal of node degrees and A the unweighted adjacency: path 1-2-3 gave [1 -1 0; -1 2 -1; 0 -1 1]; a triangle gave 2 on the diagonal and -1 elsewhere; an isolated node gives a zero row and column. *(cases: `path`, `triangle`, `isolated`)*
3. **Confirmed** — Edge weights are ignored (weights 5 and 7 gave the same matrix as the unweighted path). *(cases: `weighted`, `path`)*
4. **Confirmed** — The empty graph gives a 0x0 sparse matrix. *(cases: `empty`)*

## Edge cases (observed)

- *(non-normative)* Observed: a numeric matrix input errored with MATLAB:UndefinedFunction. Recorded as non-normative; this identifier is not required. *(cases: `err-matrix`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| The graph has a self-loop. | `MATLAB:graphfun:laplacian:SelfLoops` | `err-self-loop` |
| The graph has parallel edges. | `MATLAB:graphfun:laplacian:Multigraph` | `err-multigraph` |
| The input is a digraph. | `MATLAB:graphfun:digraph:OnlyGraphSupported` | `err-digraph` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `path` | `double` | `[3, 3]` | [1 -1 0;-1 2 -1;0 -1 1] |
| `class` | `double` | `[3, 3]` | sparse([1;2;1;2;3;2;3], [1;1;2;2;2;3;3], [1;-1;-1;2;-1;-1;1], 3, 3) |
| `weighted` | `double` | `[3, 3]` | [1 -1 0;-1 2 -1;0 -1 1] |
| `triangle` | `double` | `[3, 3]` | [2 -1 -1;-1 2 -1;-1 -1 2] |
| `isolated` | `double` | `[3, 3]` | [1 -1 0;-1 1 0;0 0 0] |
| `err-self-loop` | — | — | error `MATLAB:graphfun:laplacian:SelfLoops` |
| `err-multigraph` | — | — | error `MATLAB:graphfun:laplacian:Multigraph` |
| `empty` | `double` | `[0, 0]` | sparse([], [], [], 0, 0) |
| `err-digraph` | — | — | error `MATLAB:graphfun:digraph:OnlyGraphSupported` |
| `err-matrix` | — | — | error `MATLAB:UndefinedFunction` |

## Unresolved questions

- `laplacian.q-warning-coverage` — Warnings were not captured by this harness; an empty warnings list is not a complete statement.
