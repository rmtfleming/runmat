# Tasks — 025-normalize

Order is normative (test-first). Every task cites requirements.

- [x] **T1** Confirm reduction-line stride math and sample-std (N−1) approach
  against `mean.rs`/`std.rs`; confirm output-class handling. [FR-025-02,
  FR-025-07] (no code changes)
- [x] **T2** Author observed/normative tests: z-score vector `[1 2 3]→[-1 0 1]`,
  range vector `→[0 0.5 1]`, output class double. [FR-025-01, FR-025-02,
  FR-025-03; SC-025-1]
- [x] **T3** Author documented-choice tests (`unresolved_choice_*`): matrix
  column-wise default, explicit `dim` row-wise, explicit `zscore`, custom
  range interval, `center`, `scale`, `norm` (p=2, p=1), integer dim.
  [FR-025-04, FR-025-05]
- [x] **T4** Author error tests: `medianiqr`, `zscore,robust`, complex →
  `UNSUPPORTED`; unknown method → `INVALID_ARGUMENT`; stable error codes.
  [FR-025-06]
- [x] **T5** Implement `normalize.rs`: descriptor, GPU/fusion specs, argument
  parser, line transform for zscore/range/center/scale/norm, error builders.
  [FR-025-01..07]
- [x] **T6** Register via `pub(crate) mod normalize;` in `math/reduction/mod.rs`.
  [SC-025-2]
- [x] **T7** Run verification commands; record output in `validation.md`.
  [SC-025-2]
- [x] **T8** Traceability table in `validation.md` (claim/question → FR → test
  → result). [SC-025-3; Principle VI]
