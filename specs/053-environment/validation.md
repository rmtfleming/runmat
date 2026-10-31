# Validation — 053-environment

Date: 2026-10-30. MATLAB was not invoked. All commands ran with
`HDF5_DIR=/home/rfleming/anaconda3` and the shared runner script.

## Files

| File | Change |
|---|---|
| `crates/runmat-runtime/src/builtins/io/repl_fs/environment/platform.rs` | new (`isunix`, `ismac` + `isunix_tests`, `ismac_tests`) |
| `crates/runmat-runtime/src/builtins/io/repl_fs/environment/release.rs` | new (`isMATLABReleaseOlderThan` + `is_matlab_release_older_than_tests`) |
| `crates/runmat-runtime/src/builtins/io/repl_fs/cd/chdir.rs` | new (`chdir` + `chdir_tests`) |
| `crates/runmat-runtime/src/builtins/datetime/date_builtin.rs` | new (`date` + `date_tests`) |
| `…/io/repl_fs/environment/mod.rs` (hand-written) | `pub(crate) mod platform;`, `pub(crate) mod release;` |
| `…/io/repl_fs/cd/mod.rs` (hand-written) | `mod chdir;` |
| `…/datetime/mod.rs` (**generated**) | one hand-added line `pub(crate) mod date_builtin;` (no hand-written parent exists in the datetime area) |

## B3 analysis (cross-artifact)

- Claims → FRs: all 26 claim ids (`grep claim_id`: isunix 4, ismac 3,
  isMATLABReleaseOlderThan 6, chdir 8, date 5) are cited in
  `provenance.md` and map to FR-053-01..21. No orphan claim.
- FRs → tests: every FR-053-01..27 has at least one test (table below).
  Every test cites a case id or an FR; no orphan test.
- Unresolved questions: all 12 have a disposition in `spec.md`.
- Placement: three hand-written parents and one generated parent (one
  line, reported here). No other file outside the feature was edited.
- No inconsistency between spec, plan and tasks was found.

## Test-first record (Principle V)

1. Tests and descriptors were written first, and every builtin body started
   with a stub `return Err(…"stub")`.
2. Failing run, `cargo test -p runmat-runtime --lib -- isunix_tests
   ismac_tests is_matlab_release_older_than_tests chdir_tests date_tests
   isfloat_tests isnat_tests struct2cell_tests strvcat_tests` (F53 and F54
   together): `test result: FAILED. 2 passed; 104 failed`. F53 part: 45
   of 46 failed; the one that passed is `date_tests::documented_choice_format_rules`,
   which tests the pure formatter helper only.
3. Stubs removed. Passing run, same command: `test result: ok. 105 passed;
   0 failed; 1 ignored` (the ignored test belongs to F54). F53 part: 46
   passed — isunix 7, ismac 4, isMATLABReleaseOlderThan 15, chdir 12,
   date 8.

## Surrounding suites

`cargo test -p runmat-runtime --lib -- builtins::io::repl_fs::cd
builtins::io::repl_fs::environment builtins::datetime
builtins::logical::tests builtins::structs builtins::strings::core`:
**696 passed, 0 failed, 1 ignored** (the F54 ignored test). This includes
the existing `cd` tests, which share `REPL_FS_TEST_LOCK` with `chdir_tests`.

## Lint and format

- `rustfmt --edition 2021 --check` on the four new files: clean.
- `cargo clippy -p runmat-runtime --lib --tests --message-format short`:
  see "Clippy" at the end of this file.

## Identifiers

`RunMat:maxrhs`, `RunMat:minrhs`,
`RunMat:isMATLABReleaseOlderThan:invalidReleaseInput`,
`RunMat:validators:mustBeTextScalar`, `RunMat:cd:NonExistentFolder`,
`RunMat:string`, `RunMat:TooManyInputs` (observed, renamespaced);
`RunMat:isMATLABReleaseOlderThan:UnsupportedOption`,
`RunMat:chdir:InvalidInput`, `RunMat:<fn>:TooManyOutputs` for all five,
`RunMat:isMATLABReleaseOlderThan:Internal`, `RunMat:date:Internal`
(documented choices). `chdir` also passes through `cd`'s
`RunMat:cd:EmptyFolder` and `RunMat:cd:ChangeFailed`.

## Traceability (claim → FR → test → result)

Platform `observed_*` tests are compiled on Linux only (the observed
platform); this run was on Linux.

| claim | FR | test(s) | result |
|---|---|---|---|
| isunix.signature-primary, isunix.value | FR-053-01 | `isunix_tests::observed_value`, `observed_consistency` | pass |
| ismac.signature-primary, ismac.value | FR-053-02 | `ismac_tests::observed_value`, `isunix_tests::observed_consistency` | pass |
| isunix.cobra-test | FR-053-03 | `isunix_tests::observed_cobra_linux_test` | pass |
| isunix.error-arg, ismac.error-arg | FR-053-04 | `isunix_tests::observed_err_arg`, `ismac_tests::observed_err_arg` | pass |
| isMATLABReleaseOlderThan.signature-primary | FR-053-05 | `is_matlab_release_older_than_tests::observed_older` | pass |
| isMATLABReleaseOlderThan.comparison | FR-053-06 | `observed_older`, `observed_same`, `observed_later_b`, `observed_later_year`, `observed_much_older`, `observed_string` | pass |
| isMATLABReleaseOlderThan.text | FR-053-07 | `observed_string`, `observed_lowercase` | pass |
| isMATLABReleaseOlderThan.error-bad-name | FR-053-08 | `observed_err_bad_name` | pass |
| isMATLABReleaseOlderThan.error-numeric | FR-053-09 | `observed_err_numeric` | pass |
| isMATLABReleaseOlderThan.error-none | FR-053-10 | `observed_err_none` | pass |
| chdir.signature-primary, chdir.changes | FR-053-11 | `chdir_tests::observed_changes`, `observed_string` | pass |
| chdir.trailing-sep | FR-053-12 | `observed_trailing_sep` | pass |
| chdir.relative | FR-053-13 | `observed_relative` | pass |
| chdir.output-old | FR-053-14 | `observed_output_old` | pass |
| chdir.no-arg | FR-053-15 | `observed_no_arg_output` | pass |
| chdir.error-missing | FR-053-16 | `observed_err_missing` | pass |
| chdir.error-numeric | FR-053-17 | `observed_err_numeric` | pass |
| date.signature-primary, date.output-class | FR-053-18 | `date_tests::observed_class_shape` | pass |
| date.format | FR-053-19 | `observed_pattern`, `observed_leading_zero` | pass |
| date.agrees-clock | FR-053-20 | `observed_agrees_clock` | pass |
| date.error-arg | FR-053-21 | `observed_err_arg` | pass |
| (documented choice) | FR-053-22 | `isunix_tests::documented_choice_host_mapping`, `ismac_tests::documented_choice_host_mapping` | pass |
| (documented choice) | FR-053-23 | `documented_choice_too_many_outputs` (isunix, ismac, date), `is_matlab_release_older_than_tests::documented_choice_arity`, `chdir_tests::documented_choice_arity` | pass |
| (documented choice) | FR-053-24 | `documented_choice_grammar`, `documented_choice_non_text_scalar`, `documented_choice_pin_is_not_older_than_itself` | pass |
| (documented choice) | FR-053-25 | `chdir_tests::documented_choice_non_text_inputs`, `documented_choice_empty_and_file` | pass |
| (documented choice) | FR-053-26 | `date_tests::documented_choice_format_rules` | pass |
| (unresolved) | FR-053-27 | `unresolved_no_warning` (isunix, isMATLABReleaseOlderThan, chdir, date) | pass |

SC-053-1: 29 observed cases → 29 `observed_*` tests, all pass on Linux.

## Unresolved-question dispositions

As in `spec.md`: platform → host `cfg!` (FR-053-22); release → compatibility
pin, more inputs `UnsupportedOption`; `chdir.q-display` → returns the
folder; date zero-pad/months/locale → FR-053-26; warnings → none.

## Deviations

None. `chdir` tests use a fresh `tempfile` folder in place of MATLAB's
`tempdir`, hold `REPL_FS_TEST_LOCK` and restore the start folder on drop.

## Freshness (finish)

2026-10-30, after `git fetch -q origin` in `../matlab-interface-spec`:
HEAD = origin/main = `1c91aa1`. `git log 1156c97..origin/main` over
`specifications/{chdir,date}` and `git log fc9f804..origin/main` (and
`..HEAD`) over `specifications/{isunix,ismac,isMATLABReleaseOlderThan}`
are empty; `git status --short -- specifications` is clean; `diff -rq`
between the imported directories and the spec repo is empty. The imported
specs are current.

## Clippy

`cargo clippy -p runmat-runtime --lib --tests --message-format short`: the
first complete run flagged one lint in an F53 file
(`release.rs`, `manual_range_contains` in a test helper), fixed with
`Range::contains`. Final run: **no diagnostics in any F53 or F54 file**.
The crate's `--tests` clippy build still stops on 4 errors in other agents'
in-progress files (`graph/domain/isomorphism.rs` ×2, `graph/domain/paths.rs`,
`table/builtins/convertvars.rs`), not in this feature.

## Compatibility-pin test

The coordinator reported that `release.rs` broke
`compatibility::tests::compatibility_version_pin_is_centralized_in_the_repository`
(the release pin may appear only in `compatibility.rs`). The tests had the
pin release written as literals. They now read it from
`crate::compatibility::MATLAB_COMPATIBILITY_RELEASE`: `same` uses the pin
itself, `later-b` (and the upper-case `B` grammar check) the pin with its
half letter replaced, and a guard `assert_observed_table_applies` checks that
the pin is an `a` release in `R2022a..R2027a` (the range where the observed
truth table holds) before the observed comparisons run. No release literal
equal to the pin remains in F53 or F54 code. Re-run with the feature tests:
`compatibility_version_pin_is_centralized_in_the_repository ... ok`, and
`test result: ok. 106 passed; 0 failed; 1 ignored` (105 feature tests + the
pin test; 1 ignored is F54's `observed_from_text_nat`).
