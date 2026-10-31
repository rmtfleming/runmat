# Tasks — 053-environment

The order is normative (test-first). Every task cites requirements, and a
task is ticked only once the work is actually done.

- [x] **T0** Freshness check of the source spec repository (read-only):
  no change under the five spec directories since their pins (`1156c97`,
  `fc9f804`) and no working-tree draft. [Principle IV]
- [x] **T1** B1: write `spec.md` and `provenance.md`; verify claim ids.
  [FR-053-01..27]
- [x] **T2** B2: write `plan.md` and this file. [all]
- [x] **T3** B3: cross-artifact analysis (claims ↔ FRs ↔ planned tests,
  placement, generated parents). [Principle VI]
- [x] **T4** Write `isunix_tests` and `ismac_tests`: 6 `observed_*` tests
  plus documented-choice tests. [FR-053-01..04, 22, 23, 27]
- [x] **T5** Write `is_matlab_release_older_than_tests`: 10 `observed_*`
  tests plus documented-choice tests. [FR-053-05..10, 23, 24, 27]
- [x] **T6** Write `chdir_tests`: 8 `observed_*` tests plus
  documented-choice tests, under `REPL_FS_TEST_LOCK` with folder restore.
  [FR-053-11..17, 23, 25, 27]
- [x] **T7** Write `date_tests`: 5 `observed_*` tests plus
  documented-choice tests. [FR-053-18..21, 23, 26, 27]
- [x] **T8** Register the modules with stub bodies; run the filter and
  record the failing run. [Principle V]
- [x] **T9** Implement `platform.rs`. [FR-053-01..04, 22, 23]
- [x] **T10** Implement `release.rs`. [FR-053-05..10, 23, 24]
- [x] **T11** Implement `chdir.rs`. [FR-053-11..17, 23, 25]
- [x] **T12** Implement `date_builtin.rs`. [FR-053-18..21, 23, 26]
- [x] **T13** Run the verification commands (feature tests, regressions,
  clippy, rustfmt); record results in `validation.md`. [SC-053-1..3]
- [x] **T14** Write the traceability table (claim → FR → test → result) and
  the closing freshness check in `validation.md`. [SC-053-3; Principle VI]
