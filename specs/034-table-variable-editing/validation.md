# Validation Report — 034-table-variable-editing — 2026-10-29

Environment: Linux x86_64; `HDF5_DIR=~/anaconda3` (upstream now links system
HDF5); test binaries run with anaconda's `lib/` on `LD_LIBRARY_PATH` through
a cargo runner wrapper. MATLAB was not invoked.

## Test-first evidence (T4)

Before `variable_editing.rs` contained any builtin:
`cargo test -p runmat-runtime --lib -- table_variable_editing` →
**0 passed, 15 failed**. Every failure was `RunMat:UndefinedFunction`
(e.g. "Undefined function: removevars"), so the builtins were absent.

## Commands and results (T7)

- `cargo check -p runmat-runtime --lib --tests`: clean.
- `cargo test -p runmat-runtime --lib -- table_variable_editing`:
  **15 passed, 0 failed**:
  - `observed_addvars_append_matches_export`
  - `observed_removevars_by_name_matches_export`
  - `observed_movevars_before_matches_export`
  - `unresolved_choice_addvars_options_unsupported`
  - `unresolved_choice_removevars_index_and_multiple_unsupported`
  - `unresolved_choice_movevars_after_index_and_lists_unsupported`
  - `unresolved_choice_addvars_default_name_is_unique_var_k`
  - `unresolved_choice_metadata_follows_variables`
  - `unresolved_choice_move_before_itself_is_unchanged`
  - `accepts_string_scalar_names`
  - `rejects_height_mismatch`
  - `rejects_unknown_variables`
  - `rejects_non_table_inputs`
  - `rejects_timetable_inputs_as_unsupported`
  - `rejects_unknown_movevars_keyword_and_arity`
- `cargo test -p runmat-runtime --lib -- builtins::table`: **187 passed,
  0 failed** before integration and still green afterwards (no regressions
  in upstream's table suite).
- `cargo clippy -p runmat-runtime --lib --tests`: no warnings.
- `cargo fmt --all -- --check`: clean.
- `cargo test -p runmat-runtime --lib -- compatibility::`: 22 passed,
  **6 failed**. The same 6 tests fail on pristine upstream
  `RM-1064-remaining` @ `b7fc8b594` (built in a separate worktree, same
  command), so they are pre-existing upstream integer-metadata work in
  progress, not caused by this feature. This feature registers no integer
  capabilities or audits, so it does not enter those tests.
  - `compatibility_version_pin_is_centralized_in_the_repository` failed
    after the upstream merges because clean-room files mentioned the
    compatibility release. Fixed on this branch: code now derives the RunMat
    release tag from `crate::compatibility::MATLAB_COMPATIBILITY_RELEASE`,
    comments point to the imported provenance, and the test skips the
    top-level `specs/` evidence directory (fork-local exception; the imported
    specs must stay byte-identical and release-tagged). The test passes.

## Traceability (claim → FR → test → result) (T8)

| Claim / basis | FR | Test(s) | Result |
|---|---|---|---|
| addvars.signature-primary, addvars.output-class, addvars.appends-variable | FR-034-01 | `observed_addvars_append_matches_export` | PASS |
| removevars.signature-primary, removevars.output-class, removevars.removes-variable | FR-034-02 | `observed_removevars_by_name_matches_export` | PASS |
| movevars.signature-primary, movevars.moves-variable | FR-034-03 | `observed_movevars_before_matches_export` | PASS |
| unresolved: addvars.q-position, addvars.q-names, U-034-1 | FR-034-04 | `unresolved_choice_addvars_options_unsupported` | PASS |
| unresolved: removevars.q-index, removevars.q-multiple | FR-034-05 | `unresolved_choice_removevars_index_and_multiple_unsupported` | PASS |
| unresolved: movevars.q-after, movevars.q-index, U-034-2 | FR-034-06 | `unresolved_choice_movevars_after_index_and_lists_unsupported`, `rejects_unknown_movevars_keyword_and_arity` | PASS |
| documented choice: unknown names, non-table, timetable (U-034-3) | FR-034-07 | `rejects_unknown_variables`, `rejects_non_table_inputs`, `rejects_timetable_inputs_as_unsupported` | PASS |
| documented choice: height, default name (U-034-4) | FR-034-08 | `rejects_height_mismatch`, `unresolved_choice_addvars_default_name_is_unique_var_k` | PASS |
| documented choice: text forms, keyword case | FR-034-09 | `accepts_string_scalar_names`, `unresolved_choice_metadata_follows_variables` (lower-case `'before'`) | PASS |
| documented choice: metadata, self-move | FR-034-10 | `unresolved_choice_metadata_follows_variables`, `unresolved_choice_move_before_itself_is_unchanged` | PASS |

No orphan tests: every test above maps to an FR. Every FR has at least one
test.

## B3 analysis summary

- spec ↔ plan ↔ tasks consistent. All eight relied-upon claim ids verified
  against the imported `provenance.yaml` files (one mis-cited id,
  `movevars.signature-before`, was corrected to `movevars.signature-primary`
  before implementation).
- Scope (Principle VII): one new file plus one `mod` line in the
  hand-written `table/builtins.rs`. The only other source edits in this
  batch are the pin-centralisation fixes listed above, which repair a
  merge-induced test failure rather than extending scope.

## Deviations and open items

- Not verified end-to-end through the MATLAB-syntax VM; the tests call the
  registered builtins through the runtime dispatcher (`call_builtin_async`).
- The WASM builtin registry was not regenerated (wasm32 target not
  installed), the same deferred item as earlier batches.
- All unresolved claims above are requested from the specification side
  (FR-010 decision D1), so F34 can be widened in a follow-up feature.
