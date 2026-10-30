# Tasks — 050-numeric-conversion

The order is normative (test-first). Every task cites requirements, and a
task is ticked only once the work is actually done.

- [x] **T0** Freshness check of the source spec repository (read-only):
  HEAD = origin/main = `1156c97`, with no diff since `8f72edb` and no
  working-tree change under the three spec directories. [Principle IV]
- [x] **T1** B1: write `spec.md` and `provenance.md` (pinned to `8f72edb`;
  claim ids verified). [FR-050-01..32]
- [x] **T2** B2: write `plan.md` and this file. [all]
- [x] **T3** Write the `nthroot` tests: 14 `observed_*` tests plus the
  documented-choice and unresolved tests. [FR-050-01..10, 21..24, 26, 30]
- [x] **T4** Write the `reallog` tests: 12 `observed_*` tests plus the
  documented-choice and unresolved tests. [FR-050-11..14, 21, 25, 26, 30]
- [x] **T5** Write the `cast` tests: 16 `observed_*` tests plus the
  documented-choice and unresolved tests. [FR-050-15..21, 27..30]
- [x] **T6** Register the modules (`pub(crate) mod` lines in the hand-written
  `roots/mod.rs`, `logarithms/mod.rs`, `integer_conversions/mod.rs`) with
  stub bodies. Run the filter and record the failing run. [Principle V]
- [x] **T7** Implement `nthroot.rs`. [FR-050-01..10, 21..24, 26]
- [x] **T8** Implement `reallog.rs`. [FR-050-11..14, 21, 25, 26]
- [x] **T9** Implement `cast.rs`. [FR-050-15..21, 27..29]
- [x] **T10** Run the verification commands: feature tests, the elementwise
  regressions, clippy and rustfmt. Record the results in `validation.md`.
  [SC-050-1..3]
- [x] **T11** Write the traceability table (claim → FR → test → result) and
  the closing freshness check in `validation.md`. [SC-050-3; Principle VI]

## cast 0.2.1 rebase (2026-10-30)

- [x] **T12** Freshness check: spec repo HEAD = origin/main = `c80cbcc`, no
  later change to `specifications/cast`, no draft. [Principle IV]
- [x] **T13** Update `spec.md`, `provenance.md` (cast pinned to `c80cbcc`),
  `plan.md` and this file; close D-050-3. [FR-050-18, 28, 29, 31, 32]
- [x] **T14** Tests first: rewrite `observed_like_complex` (complex `2+0i`),
  add `observed_like_complex_parts`, `observed_like_complex_int_src`,
  `observed_like_complex_single`, `observed_like_complex_zero_imag`,
  `observed_like_real_from_complex`, and the documented-choice test for
  complex integer / complex sparse `p`. Run `cast_tests` and record the
  failing run. [FR-050-28, 31, 32]
- [x] **T15** Implement the complex lift in `cast.rs`. [FR-050-28, 31, 32]
- [x] **T16** Run `cast_tests`, the `builtins::math::elementwise` suite,
  rustfmt, and clippy `--lib --tests` with and without
  `--features parallel_xval`; update `validation.md`. [SC-050-1..3]
