# Tasks — 064-file-io-n5

The order is normative (test-first). Every task cites requirements, and a
task is ticked only once the work is actually done.

- [x] **T0** Freshness check of the source spec repository (read-only): no
  change to `specifications/{fscanf,urlwrite}` since `1c91aa1`; no drafts;
  imports byte-identical. [Principle IV]
- [x] **T1** B1: write `spec.md` and `provenance.md` (claim ids verified).
  [FR-064-01..26]
- [x] **T2** B2: write `plan.md` and this file. [all]
- [x] **T3** B3: cross-artifact analysis (recorded in `validation.md`,
  "Analysis"). [Principle VI]
- [x] **T4** Survey reuse: `sscanf` scanner in `strings/core/compat.rs`
  (made `pub(crate)`, D-064-2); fopen registry; `urlread`'s transport use
  and the loopback test pattern of `webread`. [FR-064-01..07, 24]
- [x] **T5** Write the `fscanf` tests: 16 `observed_*` tests plus
  documented-choice tests. [FR-064-01..10, 20..23]
- [x] **T6** Write the `urlwrite` tests: `observed_*` tests for the 9 cases
  plus documented-choice tests (loopback http). [FR-064-11..16, 24, 25]
- [x] **T7** Register the modules (hand-written `io/filetext/mod.rs`,
  `io/http/mod.rs`) with stub bodies. Run the filter and record the failing
  run. [Principle V]
- [x] **T8** Implement `fscanf.rs`. [FR-064-01..10, 20..23]
- [x] **T9** Implement `urlwrite.rs`. [FR-064-11..16, 24, 25]
- [x] **T10** Run the verification commands: feature tests, the
  filetext/http/strings regressions, clippy and rustfmt; record in
  `validation.md`. [SC-064-1..3]
- [x] **T11** Write the traceability table and the closing freshness check
  in `validation.md`. [SC-064-3; Principle VI]
