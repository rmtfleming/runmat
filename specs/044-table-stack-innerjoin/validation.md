# Validation Report — 044-table-stack-innerjoin — 2026-10-30

Environment: Linux x86_64; `HDF5_DIR=~/anaconda3`; test binaries run through
the scratchpad cargo runner wrapper. MATLAB was not invoked.

## Spec freshness (T0, T7)

In `matlab-interface-spec`, at start and again at finish: `git fetch -q --all`,
then `git diff --name-only c65e238 HEAD -- specifications/{stack,innerjoin}`,
the same against `origin/main`, and `git status --short` for both dirs were
all empty. At finish source HEAD = origin/main = `8f72edb` (newer than the pin,
but with no change under either directory). The imported directories are
byte-identical to the source (`diff -rq`).

## Test-first evidence (T4)

The test modules were written first. At that point `stack.rs` and
`innerjoin.rs` held only the tests plus an unregistered placeholder function,
which returned an error so that the direct-call output-count tests compiled.
`cargo test -p runmat-runtime --lib -- table_stack_tests table_innerjoin_tests`
gave **0 passed, 38 failed**. 36 failures reported `RunMat:UndefinedFunction`
(the builtins were not registered). The 2 direct-call tests
(`unresolved_second_output_unsupported`, `unresolved_index_outputs_unsupported`)
got the placeholder error instead of `UnsupportedOption`. The first attempt to
run was blocked by an unrelated compile error in a parallel agent's file
(`graph/domain/editing.rs`), and the run was retried until that file compiled.

After the implementation, the first run gave 35 passed and 3 failed. All
three were test defects, now fixed. Two name-suffix tests omitted
`'Keys','k'`, so `v` was inferred as a second key and no name clashed. One
test compared tables through `Debug` text, whose `HashMap` field order is not
stable. The implementation did not change.

## Commands and results (T7)

- `cargo test -p runmat-runtime --lib -- table_stack_tests table_innerjoin_tests`:
  **38 passed, 0 failed** (19 stack, 19 innerjoin).
- `cargo test -p runmat-runtime --lib -- builtins::table`: **278 passed,
  0 failed** (240 before this feature + 38).
- `cargo clippy -p runmat-runtime --lib --tests --message-format short`: no
  diagnostics in `stack.rs`, `innerjoin.rs` or `table/builtins.rs`. The run
  exits 101 only because of 3 `unnecessary_to_owned` errors in a parallel
  agent's `introspection/isprop.rs` (not touched).
- `rustfmt --edition 2021 --check` on both files: clean.

## Traceability (T8)

| claim | FR | test(s) | result |
|---|---|---|---|
| stack.signature-primary | FR-044-01 | `observed_basic` | pass |
| stack.layout | FR-044-01 | `observed_basic`, `observed_by_index`, `observed_three_vars` | pass |
| stack.default-names | FR-044-02 | `observed_basic`, `observed_indicator`, `observed_three_vars` | pass |
| stack.indicator | FR-044-03 | `observed_basic`, `observed_indicator` | pass |
| stack.by-index | FR-044-04 | `observed_by_index` | pass |
| stack.options | FR-044-05 | `observed_new_names`, `observed_constant_none` | pass |
| stack.single-var | FR-044-06 | `observed_one_var` | pass |
| stack.error-unknown | FR-044-07 | `observed_error_unknown_variable` | pass |
| stack.error-class-mix | FR-044-07 | `observed_error_class_mix` | pass |
| innerjoin.signature-primary, innerjoin.output-class | FR-044-10 | `observed_basic`, `observed_no_match` | pass |
| innerjoin.rows | FR-044-11 | `observed_basic` | pass |
| innerjoin.row-order | FR-044-11 | `observed_unsorted_left`, `observed_unsorted_both` | pass |
| innerjoin.single-key | FR-044-12 | `observed_basic`, `observed_keys` | pass |
| innerjoin.keys | FR-044-13 | `observed_keys`, `observed_leftright_keys` | pass |
| innerjoin.duplicates | FR-044-14 | `observed_duplicate_left_keys_keep_left_order` | pass |
| innerjoin.name-suffix | FR-044-15 | `observed_same_nonkey_unnamed`, `observed_same_nonkey_named` | pass |
| innerjoin.no-match | FR-044-16 | `observed_no_match` | pass |
| innerjoin.left-variables | FR-044-17 | `observed_left_variables` | pass |
| innerjoin.error-no-common | FR-044-18 | `observed_error_no_common_variable` | pass |
| stack.q-second-output | FR-044-20 | `unresolved_second_output_unsupported` | pass |
| stack.q-groups | FR-044-21 | `unresolved_groups_unsupported` | pass |
| innerjoin.q-index-outputs | FR-044-23 | `unresolved_index_outputs_unsupported` | pass |
| innerjoin.q-many-to-many | FR-044-24 | `unresolved_many_to_many_unsupported` | pass |
| innerjoin.q-name-suffix-context | FR-044-34 | `unresolved_name_suffix_falls_back_to_left_right`, `observed_same_nonkey_*` | pass |
| stack/innerjoin.q-warning-coverage | FR-044-22 | none (no warnings emitted) | n/a |
| U-044-1 | FR-044-25 | `documented_choice_categories_in_listing_order` | pass |
| U-044-2 | FR-044-26 | `documented_choice_indicator_name_with_new_data_name` | pass |
| U-044-3 | FR-044-27 | `documented_choice_constant_variables` | pass |
| U-044-4 | FR-044-28 | `documented_choice_same_class_interleave`, `documented_choice_unobserved_class_mixes_unsupported` | pass |
| U-044-5 | FR-044-29 | `rejects_non_table_inputs`, `documented_choice_carried_classes_and_empty` | pass |
| U-044-6 | FR-044-30 | `rejects_malformed_arguments` | pass |
| U-044-7 | FR-044-31 | `documented_choice_key_placement` | pass |
| U-044-8 | FR-044-32 | `documented_choice_left_variables` | pass |
| U-044-9 | FR-044-33 | `documented_choice_key_classes` | pass |
| U-044-10 | FR-044-35 | `rejects_malformed_options` | pass |
| U-044-11 | FR-044-36 | `rejects_non_table_inputs_and_uniquifies_names` | pass |

All 20 observation cases (9 stack, 11 innerjoin) are reproduced. Each test
checks variable names, order, size, class and values, and the categorical
values and categories for the indicator.

## Deviations and items for review

- **Name suffixes (flagged, FR-044-34).** RunMat reads the caller's argument
  text from `crate::callsite` and uses it only when the call site matches
  this call and both table arguments are distinct plain identifiers. In every
  other case the suffixes are `_left`/`_right`. This includes a call made
  through `feval`, the same variable twice, and calls with no recorded call
  site, such as a direct Rust call. A bare function name used as an argument
  is indistinguishable from a variable name. This is the same mechanism and
  limitation as F35.
- **Key placement (U-044-7).** Keys come first, in key order. This reads
  `innerjoin.single-key` literally. It is unobserved when the key is not the
  first left variable. F35 `outerjoin` with `MergeKeys` keeps the merged key at
  the left key's position, so the two builtins may differ for such tables.
  This needs a spec-side observation.
- **Indicator name with only NewDataVariableName (U-044-2)** and **indicator
  category order (U-044-1)** are documented choices, unobserved.
- **Class mixes (U-044-4).** `InterleavingDataVarsFailed` is raised only for the
  observed double + cell mix. Every other mix is `UnsupportedOption`.
- **Unknown innerjoin key or LeftVariables** use `RunMat:innerjoin:UnknownVariable`
  (unobserved; F35 convention), whereas stack's unknown names use the observed
  `RunMat:table:UnrecognizedVarName`.
- **D-034-1.** A `LeftVariables` selection that would leave no output
  variable raises `UnsupportedOption`. Otherwise the gap does not arise.
- **Code duplication.** The innerjoin key, option and name helpers are copied
  from `joins.rs`, which was not edited. This is noted in the file header and
  in `provenance.md`.
