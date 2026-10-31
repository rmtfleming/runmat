# Validation — 054-types-structs-text

Date: 2026-10-30. MATLAB was not invoked. All commands ran with
`HDF5_DIR=/home/rfleming/anaconda3` and the shared runner script.

## Files

| File | Change |
|---|---|
| `crates/runmat-runtime/src/builtins/logical/tests/isfloat.rs` | new (`isfloat` + `isfloat_tests`) |
| `crates/runmat-runtime/src/builtins/logical/tests/isnat.rs` | new (`isnat` + `isnat_tests`) |
| `crates/runmat-runtime/src/builtins/structs/core/struct2cell.rs` | new (`struct2cell` + `struct2cell_tests`) |
| `crates/runmat-runtime/src/builtins/strings/core/strvcat.rs` | new (`strvcat` + `strvcat_tests`) |
| `…/logical/tests/mod.rs`, `…/structs/core/mod.rs`, `…/strings/core/mod.rs` | one `pub mod` line per builtin; all three parents are hand-written |

No generated parent was edited. (`strings/core/mod.rs` also has a
`pub mod as_many_of_pattern;` line from another parallel agent; it is not
part of this feature.)

## B3 analysis (cross-artifact)

- Claims → FRs: all 26 claim ids (`grep claim_id`: isfloat 5,
  struct2cell 6, strvcat 11, isnat 4) are cited in `provenance.md`. The 24
  normative ones map to FR-054-01..22. The two `normative: false` ones
  (`struct2cell.non-struct`, `isnat.non-datetime`) back only documented
  choices (FR-054-25, 26).
- FRs → tests: every FR-054-01..28 has at least one test (table below); no
  orphan test.
- Placement: hand-written parents only; `isnat` reads datetime storage
  through the `pub(crate)` re-exports of `crate::builtins::datetime`, so the
  generated `datetime/mod.rs` is not touched by this feature.
- One known upstream gap was planned for (D-054-6/7) and confirmed in B4
  (see Deviations).

## Test-first record (Principle V)

1. Tests and descriptors were written first, and every builtin body started
   with a stub `return Err(…"stub")`.
2. Failing run, `cargo test -p runmat-runtime --lib -- isunix_tests
   ismac_tests is_matlab_release_older_than_tests chdir_tests date_tests
   isfloat_tests isnat_tests struct2cell_tests strvcat_tests` (F53 and F54
   together): `test result: FAILED. 2 passed; 104 failed`. F54 part: 59
   of 60 failed; the one that passed is
   `struct2cell_tests::documented_choice_shape_rule`, which tests the pure
   shape helper only.
3. Stubs removed. The first passing run had 1 failure:
   `isnat_tests::observed_from_text_nat`, because RunMat's `datetime('NaT')`
   raises `RunMat:datetime:InvalidInput` (D-054-7). That test is now
   `#[ignore]`d with the deviation id.
4. Final run, same command: `test result: ok. 105 passed; 0 failed; 1
   ignored`. F54 part: isfloat 20, isnat 8 (+1 ignored), struct2cell 12,
   strvcat 19.

## Surrounding suites

`cargo test -p runmat-runtime --lib -- builtins::io::repl_fs::cd
builtins::io::repl_fs::environment builtins::datetime
builtins::logical::tests builtins::structs builtins::strings::core`:
**696 passed, 0 failed, 1 ignored** (`observed_from_text_nat`).

## Lint and format

- `rustfmt --edition 2021 --check` on the four new files: clean.
- `cargo clippy -p runmat-runtime --lib --tests --message-format short`:
  see "Clippy" at the end of this file.

## Identifiers

Observed, renamespaced: `RunMat:minrhs`, `RunMat:maxrhs`,
`RunMat:character:InvalidInput`, `RunMat:invalidConversion`.
Documented choices: `RunMat:isnat:UndefinedFunction`,
`RunMat:struct2cell:UndefinedFunction`, `RunMat:isnat:InvalidInput`,
`RunMat:struct2cell:InvalidInput`, `RunMat:strvcat:UnsupportedOption`,
`RunMat:<fn>:TooManyOutputs` (all four), `RunMat:<fn>:Internal`
(isnat, struct2cell, strvcat). `strvcat` passes through `char`'s
`RunMat:char:*` errors for numeric inputs `char` rejects.

## Traceability (claim → FR → test → result)

| claim | FR | test(s) | result |
|---|---|---|---|
| isfloat.signature-primary | FR-054-01 | `isfloat_tests::observed_double` | pass |
| isfloat.true-cases | FR-054-02 | `observed_double`, `observed_single`, `observed_complex`, `observed_nan`, `observed_empty`, `observed_sparse`, `observed_matrix` | pass |
| isfloat.false-cases | FR-054-03 | `observed_int8`, `observed_uint64`, `observed_logical`, `observed_char`, `observed_string`, `observed_cell`, `observed_struct`, `observed_handle` | pass |
| isfloat.error-none, isfloat.error-two | FR-054-04 | `observed_err_none`, `observed_err_two` | pass |
| isnat.signature-primary | FR-054-05 | `isnat_tests::observed_nat` | pass |
| isnat.elementwise | FR-054-06 | `observed_nat`, `observed_valid`, `observed_array`, `observed_matrix`; `observed_from_text_nat` | pass; **ignored** (D-054-7) |
| isnat.empty | FR-054-07 | `observed_empty` | pass |
| struct2cell.signature-primary, struct2cell.scalar | FR-054-08, 09 | `struct2cell_tests::observed_scalar`, `observed_nested`, `observed_matrix_value` | pass |
| struct2cell.array-size | FR-054-10 | `observed_array_size`, `observed_column_array_size`, `observed_matrix_array_size`, `observed_empty_array` | pass |
| struct2cell.array-order | FR-054-11 | `observed_array_size` | pass |
| struct2cell.no-fields | FR-054-12 | `observed_no_fields` | pass |
| strvcat.signature-primary, strvcat.pad | FR-054-13 | `strvcat_tests::observed_pad`, `observed_pad_size`, `observed_trailing_blanks` | pass |
| strvcat.skip-empty | FR-054-14 | `observed_skip_empty`, `observed_cell_empty` | pass |
| strvcat.cell | FR-054-15 | `observed_cell` | pass |
| strvcat.matrix-arg | FR-054-16 | `observed_matrix_arg` | pass |
| strvcat.numeric | FR-054-17 | `observed_numeric` | pass |
| strvcat.string | FR-054-18 | `observed_string` | pass |
| strvcat.empty | FR-054-19 | `observed_no_args`, `observed_all_empty` | pass |
| strvcat.one | FR-054-20 | `observed_one` | pass |
| strvcat.error-mixed | FR-054-21 | `observed_err_mixed` | pass |
| strvcat.error-struct | FR-054-22 | `observed_err_struct` | pass |
| (documented choice) | FR-054-23 | `documented_choice_too_many_outputs` (isfloat, strvcat), `documented_choice_arity` (isnat, struct2cell) | pass |
| (documented choice) | FR-054-24 | `isfloat_tests::documented_choice_other_classes` | pass |
| isnat.non-datetime (non-normative) | FR-054-25 | `isnat_tests::documented_choice_non_datetime` | pass |
| struct2cell.non-struct (non-normative) | FR-054-26 | `struct2cell_tests::documented_choice_non_struct`, `documented_choice_shape_rule` | pass |
| (documented choice) | FR-054-27 | `strvcat_tests::documented_choice_conversions`, `documented_choice_cells`, `documented_choice_other_classes` | pass |
| (unresolved) | FR-054-28 | `unresolved_no_warning` (all four) | pass |

SC-054-1: of 50 observed cases, 45 normative cases have `observed_*` tests
(44 pass, `from-text-nat` ignored); the 5 non-normative error cases are
covered by documented-choice tests.

## Deviations

- **D-054-6** RunMat has no `NaT` builtin and no `datetime.empty`. The
  `nat`, `array`, `matrix` and `empty` cases build their datetime inputs
  with RunMat's `datetime_object_from_serial_tensor` (NaN serial = NaT).
  Valid datetimes come from RunMat's own `datetime(y,m,d)`.
- **D-054-7** RunMat's `datetime('NaT')` raises
  `RunMat:datetime:InvalidInput` ("unable to parse date/time text 'NaT'").
  `isnat_tests::observed_from_text_nat` is `#[ignore]`d; fixing it is a
  datetime-constructor change outside this feature.

## Freshness (finish)

2026-10-30, after `git fetch -q origin` in `../matlab-interface-spec`:
HEAD = origin/main = `1c91aa1`. `git log 1156c97..origin/main` over
`specifications/{isfloat,struct2cell,strvcat}` and `git log
fc9f804..origin/main` (and `..HEAD`) over `specifications/isnat` are
empty; `git status --short -- specifications` is clean; `diff -rq` between
the imported directories and the spec repo is empty.

## Clippy

`cargo clippy -p runmat-runtime --lib --tests --message-format short`: the
first complete run flagged one lint in an F53 file
(`release.rs`, `manual_range_contains` in a test helper), fixed with
`Range::contains`. Final run: **no diagnostics in any F53 or F54 file**.
The crate's `--tests` clippy build still stops on 4 errors in other agents'
in-progress files (`graph/domain/isomorphism.rs` ×2, `graph/domain/paths.rs`,
`table/builtins/convertvars.rs`), not in this feature.
