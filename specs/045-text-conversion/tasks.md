# Tasks — 045-text-conversion

The order is normative (test-first). Every task cites requirements, and a
task is ticked only once the work is actually done.

- [x] **T0** Freshness check of the source spec repository (read-only):
  HEAD = origin/main = `8f72edb`, with no diff since `c65e238` and no
  working-tree change under the three spec directories. [Principle IV]
- [x] **T1** B1: write `spec.md` and `provenance.md` (pinned to `c65e238`;
  claim ids verified). [FR-045-01..28]
- [x] **T2** B2: write `plan.md` and this file. [all]
- [x] **T3** Write the `regexptranslate` tests: 13 `observed_*` tests plus
  the documented-choice and unresolved tests. [FR-045-01..05, 18..20, 28]
- [x] **T4** Write the `dec2base` tests: 17 `observed_*` tests plus the
  documented-choice and unresolved tests. [FR-045-06..11, 21..24, 28]
- [x] **T5** Write the `validatestring` tests: 13 `observed_*` tests plus the
  documented-choice and unresolved tests. [FR-045-12..17, 25..28]
- [x] **T6** Register test-only modules (`pub mod` lines in the hand-written
  `strings/regex/mod.rs` and `strings/core/mod.rs`). Run the filter and
  record the failing run. [Principle V]
- [x] **T7** Implement `regexptranslate.rs`. [FR-045-01..05, 18..20]
- [x] **T8** Implement `dec2base.rs`. [FR-045-06..11, 21..24]
- [x] **T9** Implement `validatestring.rs`. [FR-045-12..17, 25..27]
- [x] **T10** Run the verification commands: feature tests, the
  `builtins::strings` regressions, clippy and rustfmt. Record the results in
  `validation.md`. [SC-045-1..3]
- [x] **T11** Write the traceability table (claim → FR → test → result) and
  the closing freshness check in `validation.md`. [SC-045-3; Principle VI]
