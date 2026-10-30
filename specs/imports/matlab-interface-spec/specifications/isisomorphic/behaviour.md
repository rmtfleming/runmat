# `isisomorphic` — behaviour specification

- **Spec version:** 0.3.0
- **Status:** approved 2026-10-29 (Tier B widening W2, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/isisomorphic_observations.json`](observations/isisomorphic_observations.json)

## Summary

Return whether two graphs (or two digraphs) are isomorphic.

## Confirmed invocation forms

- `tf = isisomorphic(G1, G2)` — True if G1 and G2 are isomorphic. *(e.g. case `identical`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — The output is a 1x1 logical for every non-erroring observed call. *(cases: `identical`, `path-vs-star`)*
2. **Confirmed** — Returns true for identical graphs, relabelled graphs (undirected, directed, with a self-loop, and a multigraph), and two empty graphs. *(cases: `identical`, `relabelled`, `directed-relabelled`, `self-loop`, `multigraph-relabelled`, `empty`)*
3. **Confirmed** — Returns false when the structures differ: path vs star with equal node and edge counts; different node counts (including isolated nodes, and a self-loop graph vs a larger path); different edge counts; a self-loop vs a parallel edge with equal node and edge counts; a multigraph vs a simple graph with equal counts. *(cases: `path-vs-star`, `node-count`, `isolated`, `self-loop-vs-none`, `edge-count`, `self-loop-vs-parallel`, `multigraph`)*
4. **Confirmed** — An ordinary call issued no warning. *(cases: `warn-call`)*
5. **Confirmed** — For digraphs, edge direction matters: chains 1->2->3 and 1->2<-3 are not isomorphic. *(cases: `directed-different`)*
6. **Confirmed** — Edge weights are not compared by default: one edge of weight 5 vs one of weight 7 gave true. *(cases: `weights-ignored`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| One input is a graph and the other a digraph. | `MATLAB:graphfun:isomorphism:ClassMismatch` | `err-mixed` |
| The inputs are numeric matrices rather than graph objects. | `MATLAB:UndefinedFunction` | `err-matrix` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `identical` | `logical` | `[1, 1]` | true |
| `relabelled` | `logical` | `[1, 1]` | true |
| `path-vs-star` | `logical` | `[1, 1]` | false |
| `node-count` | `logical` | `[1, 1]` | false |
| `edge-count` | `logical` | `[1, 1]` | false |
| `directed-relabelled` | `logical` | `[1, 1]` | true |
| `directed-different` | `logical` | `[1, 1]` | false |
| `self-loop` | `logical` | `[1, 1]` | true |
| `self-loop-vs-none` | `logical` | `[1, 1]` | false |
| `self-loop-vs-parallel` | `logical` | `[1, 1]` | false |
| `warn-call` | `struct` | `[1, 1]` | value=true; warning_id='' |
| `multigraph` | `logical` | `[1, 1]` | false |
| `multigraph-relabelled` | `logical` | `[1, 1]` | true |
| `weights-ignored` | `logical` | `[1, 1]` | true |
| `empty` | `logical` | `[1, 1]` | true |
| `isolated` | `logical` | `[1, 1]` | false |
| `err-mixed` | — | — | error `MATLAB:graphfun:isomorphism:ClassMismatch` |
| `err-matrix` | — | — | error `MATLAB:UndefinedFunction` |

## Unresolved questions

- `isisomorphic.q-warning-coverage` — Warnings were captured only in the warn-* cases; other cases (including error and edge cases) were not checked for warnings, so an empty warnings list is not a complete statement.
- `isisomorphic.q-options` — Name-value options for comparing node or edge variables were not exercised.
- `isisomorphic.q-second-output` — A second output (a node permutation) was not exercised.

Resolved in 0.3.0: `isisomorphic.q-nonisomorphic`.
