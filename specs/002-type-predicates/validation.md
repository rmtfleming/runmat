# Validation Report — 002-type-predicates — 2026-07-22

## Commands and results

- `cargo check -p runmat-runtime` — clean (1m 11s)
- `cargo fmt --all -- --check` — clean
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::logical::tests::iscell builtins::logical::tests::isstruct builtins::io::repl_fs::isfile` — **12 passed, 0 failed**

## Traceability (claim → FR → test → result)

| Claim | FR | Test | Result |
|---|---|---|---|
| iscell.signature-primary, output-class, logical-cell-test | FR-002-01 | `iscell::tests::observed_cell_array_is_true`, `observed_empty_cell_is_true`, `observed_non_cell_inputs_are_false` | PASS |
| isstruct.signature-primary, output-class, logical-struct-test | FR-002-02 | `isstruct::tests::observed_scalar_struct_is_true`, `observed_struct_array_is_true`, `observed_non_struct_inputs_are_false` | PASS |
| isfile.signature-primary, output-class, missing-false | FR-002-03 | `isfile::tests::observed_missing_path_is_false` | PASS |
| (summary) existing→true | FR-002-04 | `isfile::tests::summary_derived_existing_file_is_true` | PASS (summary-derived) |
| unresolved choices | FR-002-05 | `iscell::…::unresolved_choice_struct_array_representation_is_false`, `isstruct::…::unresolved_choice_empty_cell_is_false`, `isfile::…::unresolved_choice_folder_is_false`, `unresolved_choice_non_text_input_is_false` | PASS (non-normative) |

All 11 observed cases across the three exports reproduce exactly (value,
class logical via `Value::Bool` ≡ 1×1 logical, per RunMat's value model).

## Deviations and notes

- Struct arrays share the cell representation in RunMat
  (`Value::Cell` of all-struct elements); `iscell` reports false and
  `isstruct` true for that representation — documented independent choice on
  unobserved input classes, keeping the predicates mutually exclusive.
- Registration is via the `#[runtime_builtin]` inventory macro (compile-time,
  same mechanism as all 662 existing builtins); name-dispatch through the
  registry is additionally exercised end-to-end by `bounds` delegating to
  `min`/`max` (feature 004).

## Unresolved (returned to specification side)

`iscell.q-output`, `isstruct.q-output`, `isfile.q-existing` (observation for
existing file), `isfile.q-folder`.
