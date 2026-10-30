# Validation Report — 035-outerjoin — 2026-10-29

Environment: Linux x86_64; `HDF5_DIR=~/anaconda3`; test binaries run through
the scratchpad cargo runner wrapper (anaconda `lib/` only for the test
process). MATLAB was not invoked.

## Spec freshness (T0, T7)

In `matlab-interface-spec`, at start and again at finish: `git fetch -q --all`,
then `git diff --name-only 5d595da HEAD -- specifications/outerjoin`, the
same against `origin/main`, and `git status --short -- specifications/outerjoin`
were all empty (source HEAD = origin/main = `c65e238` at finish). The four
imported files are byte-identical to the source (`diff -q`).

## Test-first evidence (T4)

The test module was written before `joins.rs` contained any builtin; only
the `mod joins;` line was present.
`cargo test -p runmat-runtime --lib -- table_outerjoin` gave **0 passed,
37 failed**. Every failure body inspected reported
`RunMat:UndefinedFunction` ("Undefined function: outerjoin"); for example
`observed_type_left_merge`, `observed_leftright_keys` and
`unresolved_select_variables_unsupported` (left: `"RunMat:UndefinedFunction"`).
The builtin was absent. A second capture of the full log was blocked by an
unrelated compile error in the parallel F34 file and was not repeated.

## Commands and results (T7)

- `cargo test -p runmat-runtime --lib -- table_outerjoin`: **37 passed,
  0 failed**.
  - Observed (25 cases): `observed_basic`, `observed_default_full`,
    `observed_type_full`, `observed_keys_name_and_index`,
    `observed_mergekeys`, `observed_type_left`, `observed_type_right`,
    `observed_type_left_merge`, `observed_type_inner`,
    `observed_leftright_keys`, `observed_leftright_keys_merge`,
    `observed_unsorted`, `observed_unsorted_merge`,
    `observed_duplicate_left_keys_keep_left_order`,
    `observed_same_nonkey_name`, `observed_fill_types`,
    `observed_multi_key`, `observed_key_second_column`,
    `observed_key_second_column_nomerge`,
    `observed_error_no_common_variable`, `observed_error_bad_type`,
    `observed_error_keys_and_leftkeys`.
  - Unresolved: `unresolved_index_outputs_unsupported`,
    `unresolved_select_variables_unsupported`,
    `unresolved_many_to_many_and_missing_keys_unsupported`,
    `unresolved_key_types_unsupported`,
    `unresolved_name_suffix_falls_back_to_left_right`.
  - Documented choices: `documented_choice_right_duplicates_follow_right_order`,
    `documented_choice_non_double_keys`,
    `documented_choice_unspecified_fill_classes`,
    `documented_choice_inner_with_different_key_names`,
    `documented_choice_name_clashes`,
    `documented_choice_default_output_properties`,
    `documented_choice_option_forms`.
  - Validation: `rejects_non_table_and_timetable_inputs`,
    `rejects_malformed_options`, `inner_join_without_matches_is_empty`.
- `cargo test -p runmat-runtime --lib -- builtins::table`: **240 passed,
  0 failed**. This includes the 37 above and the parallel F34 tests, with
  no regressions.
- `cargo clippy -p runmat-runtime --lib --tests --message-format short`:
  see "Clippy" below.
- `rustfmt --edition 2021 --check` on `joins.rs` and `table/builtins.rs`:
  clean. `cargo fmt --all` was deliberately not run, so the parallel
  agent's files were left untouched.

### Clippy

- First run: one lint in `joins.rs` (`manual implementation of
  .is_multiple_of()` at the option-pair parity check). Fixed
  (`!rest.len().is_multiple_of(2)`).
- Final run: **no diagnostics in `joins.rs`**. The lib check still stops
  with one error in an unrelated, untracked work-in-progress file,
  `builtins/plotting/ops/exportgraphics.rs:288` (same lint, not part of
  this feature and not touched). Earlier runs also flagged
  `common/validateattributes.rs`. Because of that failure, the `--tests`
  clippy target (which would lint the `joins.rs` test module) was not
  reached. Clippy on the test module is therefore **unverified**; the test
  module compiles and runs without warnings under `cargo test`.
- After the lint fix: `table_outerjoin` 37 passed, 0 failed (re-run).

## Traceability (claim → FR → test → result) (T8)

| Claim / basis | FR | Test(s) | Result |
|---|---|---|---|
| outerjoin.signature-primary, output-class, table-shape | FR-035-01 | `observed_basic`, `observed_fill_types`, `observed_mergekeys` | PASS |
| outerjoin.key-inference, error-no-common | FR-035-02 | `observed_default_full`, `observed_multi_key`, `observed_error_no_common_variable` | PASS |
| outerjoin.keys-option, error-keys-and-leftkeys | FR-035-03 | `observed_keys_name_and_index`, `observed_leftright_keys`, `observed_error_keys_and_leftkeys` | PASS |
| outerjoin.type-rows, error-bad-type | FR-035-04 | `observed_type_full/left/right/inner`, `observed_error_bad_type` | PASS |
| outerjoin.row-order | FR-035-05 | `observed_unsorted`, `observed_unsorted_merge`, `observed_multi_key` | PASS |
| outerjoin.duplicate-keys | FR-035-06 | `observed_duplicate_left_keys_keep_left_order` | PASS |
| outerjoin.column-order | FR-035-07 | `observed_key_second_column`, `observed_key_second_column_nomerge`, `observed_type_left`, `observed_multi_key` | PASS |
| outerjoin.mergekeys | FR-035-08 | `observed_mergekeys`, `observed_type_left_merge`, `observed_leftright_keys_merge` | PASS |
| outerjoin.name-suffix | FR-035-09 | `observed_default_full`, `observed_unsorted`, `observed_same_nonkey_name`, `observed_basic`, `observed_key_second_column_nomerge` | PASS |
| outerjoin.fill-values | FR-035-10 | `observed_fill_types`, `observed_default_full`, `observed_type_left` | PASS |
| outerjoin.inner-key-single | FR-035-11 | `observed_type_inner` | PASS |
| error identifiers | FR-035-12 | the three `observed_error_*` tests | PASS |
| unresolved: q-index-outputs | FR-035-13 | `unresolved_index_outputs_unsupported` | PASS (direct call; see open item 1) |
| unresolved: q-select-variables | FR-035-14 | `unresolved_select_variables_unsupported` | PASS |
| unresolved: q-many-to-many | FR-035-15 | `unresolved_many_to_many_and_missing_keys_unsupported` | PASS |
| unresolved: q-key-types | FR-035-16 | `unresolved_key_types_unsupported` | PASS |
| unresolved: q-name-suffix-context | FR-035-17 | `unresolved_name_suffix_falls_back_to_left_right` plus the call-site observed tests | PASS |
| U-035-1, U-035-2 | FR-035-18 | `documented_choice_right_duplicates_follow_right_order`, `documented_choice_non_double_keys` | PASS |
| U-035-3 | FR-035-19 | `documented_choice_unspecified_fill_classes` | PASS |
| U-035-4, -5, -9 | FR-035-20 | `documented_choice_inner_with_different_key_names`, `documented_choice_name_clashes` | PASS |
| U-035-6, -7 | FR-035-21 | `documented_choice_default_output_properties`, `rejects_non_table_and_timetable_inputs` | PASS |
| U-035-8 | FR-035-22 | `documented_choice_option_forms`, `rejects_malformed_options`, `inner_join_without_matches_is_empty` | PASS |

Every FR has at least one test and every test maps to an FR. Every claim id
cited was checked against the imported `provenance.yaml` (16 claims, all
present).

## Deviations and open items

1. **Index outputs through the dispatcher (FR-035-13).** The builtin
   raises `UnsupportedOption` when `crate::output_count` reports more than
   one requested output, which the test checks by calling the builtin
   directly under `push_output_count(Some(2|3))`. That is how other
   multi-output tests in this crate do it. However, an earlier version of the test called
   `crate::call_builtin_async_with_outputs("outerjoin", .., 2)`, and the
   dispatcher invoked the builtin twice: first with `Some(2)` (which raised
   the error), then a second time at `dispatcher.rs:365` seeing no output
   count (which succeeded and returned the table). The root cause lies in
   the dispatcher/context machinery and was not investigated further (out of
   scope). So `[T, il, ir] = outerjoin(...)` through the full interpreter
   is **not verified** to raise `UnsupportedOption`. It may instead
   surface as the VM's generic too-many-outputs error for a fixed-output
   builtin, or something else. This needs a follow-up check.
2. **Caller-name suffixes (FR-035-17).** These are implemented from the
   interpreter's per-call call-site record (`crate::callsite::current_callsite`
   and `arg_text`), the same mechanism behind `display` labels and
   `inputname`. The tests simulate the record exactly as the existing
   `display` tests do. They have not been checked end-to-end through the
   MATLAB-syntax VM.
3. The tests call the registered builtin through `call_builtin_async`;
   there is no VM-level script test.
4. The WASM builtin registry was not regenerated, the same deferred item
   as earlier batches.
5. `outerjoin.q-name-suffix-context` stays flagged for human review. RunMat
   reproduces the observed caller-name suffixes when it can recover them
   and otherwise falls back to the observed `_left`/`_right`.
