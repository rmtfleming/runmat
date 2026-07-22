# Tasks — 027-sparse-ordering

Order is normative (test-first). Every task cites requirements.

- [x] **T1** Inspect `SparseTensor`/`Tensor` value models and `math/sparse`
  conventions (`full`, `speye`, `nonzeros`); confirm the shared pattern-reader
  approach. [FR-027-01] (no code changes)
- [x] **T2** Add the shared `matrix_pattern`/`reject_nd` reader to
  `math/sparse/mod.rs` and register `mod colamd; mod dissect;`. [FR-027-01,
  FR-027-05]
- [x] **T3** Implement `colamd.rs`: column-intersection graph + minimum-degree
  ordering; descriptor, GPU/fusion specs, `#[runtime_builtin]`, type resolver,
  error builders; cite Principle X sources in the module doc. [FR-027-01,
  FR-027-02, FR-027-04, FR-027-05]
- [x] **T4** Implement `dissect.rs`: symmetric graph + recursive nested
  dissection (level-structure bisection, pseudo-peripheral root); descriptor,
  GPU/fusion specs, macro, type resolver, errors; cite Principle X sources.
  [FR-027-01, FR-027-03, FR-027-04, FR-027-05]
- [x] **T5** Author tests for both: normative valid-permutation tests (observed,
  dense, larger/rectangular, empty) asserting ONLY class double, size 1×n,
  valid permutation — NOT the specific values; `documented_choice` tests
  (ordering on observed input, scalar, logical, non-square, non-matrix
  rejection); descriptor/GPU/fusion/type-resolver tests. [SC-027-1..3]
- [x] **T6** Run verification commands; record output in `validation.md`.
  [SC-027-2]
- [x] **T7** Traceability table in `validation.md` (claim → FR → test →
  result); make the non-assertion of specific permutation values explicit.
  [SC-027-3; Principle VI]
