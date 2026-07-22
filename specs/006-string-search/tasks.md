# Tasks — 006-string-search

Order is normative (test-first). Every task cites requirements.

- [x] **T1** Inspect `strings/search` exemplars (`contains.rs`,
  `startswith.rs`, `text_utils.rs`, `strfind.rs`) and the `fileparts.rs`
  multi-output pattern; confirm text-collection and output-count mappings.
  [FR-006-01..10] (no code changes)
- [x] **T2** Author tests for the observed `matches` cases (`in-set`,
  `scalar`, `ignore-case`) plus documented-choice and error-path tests in
  the new file's test module (red step; compile gated by T4).
  [FR-006-01..03; SC-006-1]
- [x] **T3** Author tests for the observed `strmatch` cases (`prefix`,
  `exact`, `no-match`) and `strtok` cases (`default`, `custom-delim`,
  `leading-space`), plus `summary_derived` remainder tests and
  `unresolved_choice` tests. [FR-006-04..10; SC-006-1]
- [x] **T4** Implement `matches.rs` + registration (descriptor, GPU/fusion
  specs, macro, `mod.rs` entry). [FR-006-01..03]
- [x] **T5** Implement `strmatch.rs` + registration. [FR-006-04..06]
- [x] **T6** Implement `strtok.rs` + registration (multi-output via
  `output_count`). [FR-006-07..10]
- [x] **T7** Run verification commands; record output in `validation.md`.
  [SC-006-1..3]
- [x] **T8** Traceability table in `validation.md` (claim → FR → test →
  result). [SC-006-3; Principle VI]
