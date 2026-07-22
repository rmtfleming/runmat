# Tasks — 030-accumarray

Order is normative (test-first). Every task cites requirements.

- [x] **T1** Inspect `array/` structure, `sparse.rs`/`sub2ind.rs` exemplars,
  and the `Value`/`Tensor` model; confirm placement in `array/creation`.
  [FR-030-01] (no code changes)
- [x] **T2** Author the observed normative test (`accumarray([1;2;1],
  [10;20;30]) => [40;20]`, double, `[2,1]`) plus the double-column test.
  [FR-030-01; SC-030-1]
- [x] **T3** Author documented-choice / unresolved tests: scalar-vals
  broadcast, `sz` length, `fillval`, single-element collapse, multi-column
  rejected, custom-function rejected, `issparse` rejected. [FR-030-03..07]
- [x] **T4** Author validation tests: non-positive / non-integer subscripts,
  vals length mismatch, undersized `sz`, logical-vals coercion.
  [FR-030-02, FR-030-03, FR-030-04, FR-030-08]
- [x] **T5** Implement `accumarray.rs`: descriptor, GPU/fusion specs, error
  set, subscript validation, host scatter-add with `touched` mask, `sz`/
  `fillval`/`fun`/`issparse` handling, output construction, `type_resolver`.
  [FR-030-01..08]
- [x] **T6** Register the module: add `pub(crate) mod accumarray;` to
  `array/creation/mod.rs`. [SC-030-2]
- [x] **T7** Run verification commands; record output in `validation.md`.
  [SC-030-1..3]
- [x] **T8** Build the traceability table (claim → FR → test → result) in
  `validation.md`. [SC-030-3; Principle VI]
