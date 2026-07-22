# Selection Proposal (FR-010) — Tier B batch 2 — 2026-07-22

Candidates: the 12 approved specifications imported from source commit
`4445294` (see IMPORT-MANIFEST.md batch-4 section). FR-002 fully satisfied.
None exist in RunMat. Substrate confirmed: `ischar.rs`, `isfile.rs`,
`mean.rs`/`std.rs`, `histcounts.rs`, `strcat.rs`, `SparseTensor`.

## Proposed grouping

| # | Feature | Function(s) | Substrate | Scope | Status |
|---|---|---|---|---|---|
| F22 | 022-legacy-predicates | `isobject`, `isstr`, `isfolder`, `isdir` | logical/introspection + filesystem | Small | Ready |
| F23 | 023-spalloc | `spalloc` | SparseTensor (empty m×n preallocation) | Small | Ready |
| F24 | 024-histc | `histc` | stats (bin counts; alongside histcounts) | Small–medium | Ready |
| F25 | 025-normalize | `normalize` | reduction (z-score default, 'range') | Small–medium | Ready |
| F26 | 026-append | `append` | strings (string/char concatenation) | Small | Ready |
| F27 | 027-sparse-ordering | `colamd`, `dissect` | SparseTensor | Medium | **Decision needed** |
| — | (table-blocked) | `movevars`, `outerjoin` | **No table type (B-010-1)** | — | BLOCKED |

## Notes

- `isstr` is the legacy alias of `ischar`; `isdir` the legacy alias of
  `isfolder`. Each pair implemented consistently (isstr→char-kind test,
  isdir→isfolder filesystem test). `isobject` → true for object/handle-object
  value kinds (e.g. the inputParser handle from feature 015).
- `append` is string/char concatenation (R2020b+), NOT table-related despite
  the name; `append("a","b")`→`"ab"` (string), `append('a','b')`→`'ab'`
  (char). Implemented over the strcat machinery.

## F27 decision — algorithm-dependent orderings (gate)

`colamd` and `dissect` return fill-reducing permutation vectors. The
**observed permutation values are algorithm-dependent** and the spec review
**deliberately did not assert them** (only the output is a 1×n double
permutation). Principle III forbids reproducing MATLAB's internal COLAMD /
nested-dissection algorithm. Two honest options:

- **(A) Independent heuristic ordering** — implement a genuine, independently
  derived fill-reducing heuristic (e.g. a simple minimum-degree ordering for
  colamd; a basic recursive graph bisection for dissect). Returns *a* valid
  permutation, useful, but generally NOT MATLAB's exact vector — which is
  fine because values are non-normative. More work; real utility.
- **(B) Valid-permutation placeholder** — return a valid permutation
  (e.g. identity, or a trivial degree sort) satisfying only the normative
  shape/class claim, documented as a non-optimising placeholder pending a
  dedicated ordering feature. Minimal; honest; low utility.

Recommendation: **(B) for now** (keeps scope minimal and avoids shipping a
half-optimising ordering that users might mistake for COLAMD), with a
follow-up feature for real orderings if needed. Alternatively defer F27
entirely.

## Proposed order

F22 → F26 → F23 → F24 → F25 → (F27 per decision). Ascending effort.

## Decision — RECORDED 2026-07-22 (maintainer, interactive gate)

- [x] Grouping and order **APPROVED**.
- [x] **F22–F26 authorised** (isobject, isstr, isfolder, isdir, spalloc,
  histc, normalize, append).
- [x] **F27 (colamd/dissect): option A — independent heuristic ordering.**
  Implement genuine, independently-derived fill-reducing heuristics
  (e.g. minimum-degree for colamd, recursive graph bisection for dissect);
  return a valid permutation of the correct shape/class (the only normative
  claim); the specific permutation is NOT asserted MATLAB-conformant
  (Principle III). Algorithm sources recorded per Principle X.
- [x] `movevars`/`outerjoin` BLOCKED with the table cluster (010).
