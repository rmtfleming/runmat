# Implementation Plan — 056-table-variables

## Constitution Check

- I Specification-first: FR-056-01..33 cite the 41 approved normative
  claims; FR-056-40..53 are tagged unresolved or documented choice. PASS.
- II Clean-room: inputs are the imported 0.2.0 exports and RunMat code only.
  MATLAB not invoked. PASS.
- III Independence: key renaming, a plain transpose, per-variable function
  application and delegation to existing conversion builtins, over RunMat's
  own table model. PASS.
- IV Provenance: `provenance.md` with both pins and freshness checks. PASS.
- V Test-first: all tests written and run against registered stubs
  (failing) before the implementation. PASS.
- VII Minimal scope: five new files plus `mod` lines in the hand-written
  `table/builtins.rs`. F34 `variable_editing.rs` is not touched; its
  selection helpers are private to that file, so the needed parts are
  re-written in a small shared support file. PASS.
- IX Compatibility: `#[runtime_builtin]` with descriptors and
  `accel = "cpu"`, reusing `table_from_columns`, `table_public_properties`,
  `sync_table_properties`, `MappedCallable`, `call_builtin_async`,
  `crate::warnings::emit` and `crate::output_count`. No integer-capability
  metadata. PASS.

## Affected files

| Item | Path |
|---|---|
| Shared support (error helper, table input, selection) | `crates/runmat-runtime/src/builtins/table/builtins/variable_support.rs` |
| `renamevars` + tests | `crates/runmat-runtime/src/builtins/table/builtins/renamevars.rs` |
| `rows2vars` + tests | `crates/runmat-runtime/src/builtins/table/builtins/rows2vars.rs` |
| `varfun` + tests | `crates/runmat-runtime/src/builtins/table/builtins/varfun.rs` |
| `convertvars` + tests | `crates/runmat-runtime/src/builtins/table/builtins/convertvars.rs` |
| Registration | five `mod` lines in `crates/runmat-runtime/src/builtins/table/builtins.rs` |

## Design

- Shared: `Ctx { name, errors }` with `error(code_suffix, detail)` looking
  up the identifier in the builtin's descriptor table; `table_input`
  (table / timetable → UnsupportedOption / other → InvalidInput);
  `output_guard` (more than one output → TooManyOutputs); `select` (names,
  string array, cellstr, indices, mask → positions in listed order,
  duplicates kept for the caller to judge).
- `renamevars`: select old positions; parse new names (char, string,
  cellstr); count check; empty check; RenameDim/Unknown resolution for a
  single unknown name; build the final name list and check uniqueness;
  rebuild the variable store with renamed keys and update `VariableNames`
  (all other properties untouched).
- `rows2vars`: parse options; resolve the source; check columns, then
  classes (FR-056-45); names (row names, source values, or `VarN`), rule
  (FR-056-46), uniqueness (FR-056-47), warning via `crate::warnings::emit`;
  build `OriginalVariableNames` and one column per input row;
  `table_from_columns`.
- `varfun`: validate arity and `func`; derive the name; `MappedCallable`
  invoke per variable (error → FunFailed); heights must agree;
  `table_from_columns`.
- `convertvars`: select; parse type (text → name table; handle →
  `MappedCallable`); convert each selected variable; replace it in the
  variable store (properties untouched).

## Tests

Module names `renamevars_tests`, `rows2vars_tests`, `varfun_tests`,
`convertvars_tests`; categories `observed_*` (one or more per observation
case), `rejects_*`, `unresolved_*`, `documented_choice_*`. Output-count
tests call the builtin function directly under
`crate::output_count::push_output_count`. Anonymous-function cases use a
closure named `anonymous#N` with an installed semantic invoker (as the
`cellfun` callback tests do).

## Risks & rollback

- Results depend on upstream builtins (`string`, `categorical`, `int8`,
  `upper`, `mean`, `std`, `cumsum`, `sum`, `max`, `table2array`). If one
  disagrees with an observation a D-056-n deviation is recorded; no
  upstream change.
- The generated wasm registry is not edited; wasm registration waits for
  the next module-composition regeneration (as F34/F44/F52).
- Rollback: delete the five files and their `mod` lines.

## Verification commands

```
export HDF5_DIR=/home/rfleming/anaconda3
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=<scratchpad>/run.sh
cargo test -p runmat-runtime --lib -- renamevars_tests rows2vars_tests varfun_tests convertvars_tests
cargo test -p runmat-runtime --lib -- builtins::table
rustfmt --edition 2021 --check <the five files>
cargo clippy -p runmat-runtime --lib --tests --message-format short
```
