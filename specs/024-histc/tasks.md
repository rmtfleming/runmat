# Tasks — 024-histc

Order is normative (test-first). Every task cites requirements.

- [ ] **T1** Read exemplars (`stats/hist/histcounts.rs`,
  `math/reduction/bounds.rs`) and confirm the `Tensor`/`Value`,
  `gpu_helpers`, `tensor`, and `output_count` helper APIs. [FR-024-01] (no code)
- [ ] **T2** Add `histc_type` to `stats/type_resolvers.rs` (output length =
  `length(edges)`, orientation from input). [FR-024-01, FR-024-05]
- [ ] **T3** Author the observed normative test for
  `histc([1 2 2 3], [1 2 3]) → [1 2 1]` (`1x3`, double). [FR-024-01..03;
  histc.signature-primary, histc.output-class, histc.bin-counts; SC-024-1]
- [ ] **T4** Author `unresolved_choice_*` tests: column orientation,
  below-first-edge dropped, above-last-edge dropped, NaN dropped, matrix
  column-wise, explicit `dim`, second output bin indices, non-monotonic-edges
  error. [FR-024-04..07]
- [ ] **T5** Implement `histc.rs`: bin rule, dim/stride binning, second output,
  GPU gather, descriptor, GPU/fusion specs, `#[runtime_builtin]`. [FR-024-01..07]
- [ ] **T6** Register `pub mod histc;` in `stats/hist/mod.rs`. [SC-024-2]
- [ ] **T7** Add GPU round-trip, descriptor, and `histc_type` infra tests.
  [FR-024-01, FR-024-05]
- [ ] **T8** Run verification commands; record actual output in
  `validation.md`. [SC-024-1..2]
- [ ] **T9** Traceability table in `validation.md` (claim → FR → test →
  result). [SC-024-3; Principle VI]
