# Validation Report — 034-table-variable-editing — widened to 0.3.0 — 2026-10-29

Environment: Linux x86_64; `HDF5_DIR=~/anaconda3`; test binaries run through
a cargo runner wrapper. The target directory was shared with a parallel
agent (outerjoin, C2 batch). MATLAB was not invoked.

The batch C1 (0.2.0) report is superseded; it is in git history at
`69a30fe2c` (15/15 tests then).

## Freshness check (spec source, read-only)

In `matlab-interface-spec`, at start and at finish, after `git fetch -q --all`:
`git diff --name-only 5d595da HEAD -- specifications/{addvars,removevars,movevars}`,
the same against `origin/main`, and
`git status --short -- specifications/{addvars,removevars,movevars}` all
printed nothing. Source HEAD = origin/main was `8614c5b` at start and
`c65e238` at finish (later commits touch only other specs). The import pin `5d595da` is current for these
three specs.

## Test-first evidence (W4)

The rewritten test module (31 tests) was run against the unchanged 0.2.0-era
implementation:
`cargo test -p runmat-runtime --lib -- table_variable_editing` →
**12 passed, 19 failed**. Every failure was the old code refusing a newly
confirmed form or using a 0.2.0 identifier:
14× `RunMat:{addvars,movevars,removevars}:UnsupportedOption` raised for a
now-confirmed form (Before/After, NewVariableNames, several variables,
index/mask/list selections, 2-argument movevars); expected-identifier
mismatches `RunMat:addvars:UnsupportedOption`/`RunMat:movevars:UnsupportedOption`
vs `RunMat:table:addmovevars:BeforeAndAfter`, `RunMat:removevars:UnknownVariable`
vs `RunMat:table:UnrecognizedVarName`, `RunMat:movevars:UnsupportedOption` vs
`InvalidArgument`; and `a,b,Var3` vs `a,b,x` (caller name not read). The 12
passing were unchanged 0.2.0 behaviour (`append`, `by-name`, `before`,
non-table, timetable, string scalars, Var<k> collision), plus tests whose
expected error the old code happened to raise already (`UnsupportedOption`
for before-past-end, mask length, location-in-vars, remove-all).

Failing before implementation: `observed_addvars_before_and_after_match_export`,
`observed_addvars_data_preserved_matches_export`,
`observed_addvars_errors_match_export`,
`observed_addvars_multiple_variables_match_export`,
`observed_addvars_name_from_input_matches_export`,
`observed_addvars_new_variable_names_match_export`,
`observed_movevars_after_and_numeric_location_match_export`,
`observed_movevars_contents_follow_matches_export`,
`observed_movevars_default_end_matches_export`,
`observed_movevars_errors_match_export`,
`observed_movevars_selectors_and_order_match_export`,
`observed_movevars_self_location_matches_export`,
`observed_removevars_by_index_and_mask_match_export`,
`observed_removevars_empty_selection_matches_export`,
`observed_removevars_errors_match_export`,
`observed_removevars_multiple_names_match_export`,
`rejects_malformed_name_value_arguments`,
`unresolved_choice_addvars_input_name_scope`,
`unresolved_choice_metadata_follows_variables`.

## Commands and results (W6)

- `cargo test -p runmat-runtime --lib -- table_variable_editing`:
  **31 passed, 0 failed** (was 15/15 at 0.2.0).
- `cargo test -p runmat-runtime --lib -- builtins::table`: **239 passed,
  1 failed**. The only failure is
  `joins::table_outerjoin_tests::unresolved_index_outputs_unsupported`, in
  the parallel agent's in-progress `outerjoin` file (not touched here). All
  table tests outside that file are green. Baseline before this work:
  187/187, which included the 15 F34 tests. The suite grew with this
  feature's tests and the parallel outerjoin tests.
- `rustfmt --edition 2021 --check variable_editing.rs`: clean.
- `cargo clippy -p runmat-runtime --lib --tests --message-format short | grep variable_editing`:
  no findings in this file after fixing one lint (`manual_is_multiple_of`). Remaining clippy errors are in other agents' files
  (`validateattributes.rs`, `exportgraphics.rs`, `spy.rs`, `findstr.rs`,
  `joins.rs`). Those errors stop the lib before the `--tests` lint pass,
  so test-code lints were not reached.

## Traceability (claim → FR → test)

| Claim(s) | FR | Test(s) | Result |
|---|---|---|---|
| addvars.signature-primary, output-class, appends-variable, default-name-position | FR-034-01, 04 | `observed_addvars_append_matches_export`, `observed_addvars_append_full_matches_export` | PASS |
| addvars.before, addvars.after | FR-034-02 | `observed_addvars_before_and_after_match_export` | PASS |
| addvars.new-names | FR-034-03 | `observed_addvars_new_variable_names_match_export`, `observed_addvars_multiple_variables_match_export` | PASS |
| addvars.multiple-variables | FR-034-01 | `observed_addvars_multiple_variables_match_export` | PASS |
| addvars.data-preserved | FR-034-01 | `observed_addvars_data_preserved_matches_export` | PASS |
| addvars.default-name-input | FR-034-04 | `observed_addvars_name_from_input_matches_export` | PASS |
| addvars.error-* (6 claims, 9 cases) | FR-034-05, 11 | `observed_addvars_errors_match_export` | PASS |
| removevars.signature-primary, output-class, removes-variable | FR-034-06 | `observed_removevars_by_name_matches_export` | PASS |
| removevars.by-index, by-logical | FR-034-06 | `observed_removevars_by_index_and_mask_match_export` | PASS |
| removevars.multiple-names | FR-034-06 | `observed_removevars_multiple_names_match_export` | PASS |
| removevars.empty-selection | FR-034-06 | `observed_removevars_empty_selection_matches_export` | PASS |
| removevars.error-* (3 claims, 5 cases) | FR-034-07, 11 | `observed_removevars_errors_match_export` | PASS |
| removevars.remove-all (deviation D-034-1) | FR-034-10 | `deviation_removevars_remove_all_with_rows_is_unsupported` | PASS (asserts the refusal, not the normative [2 0]) |
| movevars.signature-primary, output-class, moves-variable | FR-034-08 | `observed_movevars_before_matches_export` | PASS |
| movevars.after, numeric-location | FR-034-08 | `observed_movevars_after_and_numeric_location_match_export` | PASS |
| movevars.numeric-vars, multiple-order | FR-034-08 | `observed_movevars_selectors_and_order_match_export` | PASS |
| movevars.self-location | FR-034-08 | `observed_movevars_self_location_matches_export` | PASS |
| movevars.default-end | FR-034-08 | `observed_movevars_default_end_matches_export` | PASS |
| movevars.contents-follow | FR-034-08 | `observed_movevars_contents_follow_matches_export` | PASS |
| movevars.error-* (4 claims) | FR-034-09, 11 | `observed_movevars_errors_match_export` | PASS |
| addvars.q-before-past-end | FR-034-12 | `unresolved_addvars_before_past_end_is_unsupported` | PASS |
| removevars.q-short-mask, q-logical-longer | FR-034-12 | `unresolved_removevars_mask_length_mismatch_is_unsupported` | PASS |
| movevars.q-location-in-vars | FR-034-12 | `unresolved_movevars_location_in_vars_is_unsupported` | PASS |
| addvars.q-default-name-collision | FR-034-13 | `unresolved_choice_addvars_default_name_is_unique_var_k` | PASS |
| addvars.q-input-name-scope | FR-034-14 | `unresolved_choice_addvars_input_name_scope` | PASS |
| documented choices | FR-034-15 | `rejects_malformed_name_value_arguments` | PASS |
| documented choices (kept) | FR-034-16 | `rejects_non_table_inputs`, `rejects_timetable_inputs_as_unsupported`, `accepts_string_scalar_names`, `unresolved_choice_metadata_follows_variables` | PASS |

Every observed case except `all` (D-034-1) and the two non-normative cases
(`before-past-end`, `logical-short`, which have their own refusal tests) is
reproduced. No orphan tests.

## Deviations and open items

- **D-034-1** `removevars.remove-all` (normative `[2 0]`) is refused with
  `RunMat:removevars:UnsupportedOption`. The upstream table type has no row
  count of its own, so supporting this needs a table-subsystem change
  outside this feature's scope.
- **Caller names**: `addvars.default-name-input` is implemented through the
  VM's builtin callsite record (`crate::callsite`), the mechanism `display`
  uses. The test simulates the callsite (`replace_source_catalog` +
  `push_callsite`), the same way `display`'s own tests do. It has not been
  exercised end-to-end through the MATLAB-syntax VM. Calls without a
  callsite (from Rust, `feval`, misaligned comma lists) use the position
  name.
- Error identifiers are the confirmed ones in RunMat's namespace
  (FR-034-11), not the literal `MATLAB:` strings.
- The WASM builtin registry was not regenerated (wasm32 target not
  installed), the same deferred item as earlier batches.
