# Implementation Plan — 053-environment

## Constitution Check

- I Specification-first: FR-053-01..21 cite the 26 approved claims.
  FR-053-22..27 are tagged as documented choices or unresolved. PASS.
- II Clean-room: the only inputs are the imported exports and RunMat code.
  MATLAB was not invoked. PASS.
- III Independence: `cfg!` platform tests, a release-name parser and
  comparison against RunMat's own pin, delegation to RunMat's `cd` and
  `clock`. PASS.
- IV Provenance: `provenance.md` pins N3 to `1156c97` and N4 to `fc9f804`,
  freshness-checked; all 26 claim ids verified. PASS.
- V Test-first: tests are written before the bodies; the failing run is
  recorded in `validation.md`. PASS.
- VII Minimal scope: four new files. Three `mod` lines go into
  **hand-written** parents (`io/repl_fs/environment/mod.rs` ×2,
  `io/repl_fs/cd/mod.rs`). `date` needs one hand-added `mod date_builtin;`
  line in the **generated** `datetime/mod.rs` (no hand-written parent in
  the datetime area exists; reported in `validation.md`). No other generated
  file is edited. PASS.
- IX Compatibility: `#[runtime_builtin]` with an inline descriptor,
  `accel = "cpu"` where applicable; values are `Value::Bool` and
  `Value::CharArray`; `chdir` and `date` reuse existing builtins. No
  integer-capability metadata. PASS.

## Placement

| Builtin | File | Parent |
|---|---|---|
| `isunix`, `ismac` | `crates/runmat-runtime/src/builtins/io/repl_fs/environment/platform.rs` | `environment/mod.rs` (hand-written; next to `getenv`, `setenv`) |
| `isMATLABReleaseOlderThan` | `crates/runmat-runtime/src/builtins/io/repl_fs/environment/release.rs` | `environment/mod.rs` (hand-written) |
| `chdir` | `crates/runmat-runtime/src/builtins/io/repl_fs/cd/chdir.rs` | `cd/mod.rs` (hand-written; child of the `cd` module so it can call `cd`'s private `execute::run` and `input::path`) |
| `date` | `crates/runmat-runtime/src/builtins/datetime/date_builtin.rs` | `datetime/mod.rs` (**generated**; one hand-added line) |

## Design

### isunix / ismac
1. More than one requested output → `TooManyOutputs`.
2. Any argument → `RunMat:maxrhs`.
3. `Value::Bool(cfg!(unix))` / `Value::Bool(cfg!(target_os = "macos"))`.

### isMATLABReleaseOlderThan
1. Outputs ≤ 1. No input → `RunMat:minrhs`; more than one →
   `UnsupportedOption`.
2. Text scalar: char row (1×n, including 0×0 empty), `Value::String`, or a
   1-element `StringArray`; else `RunMat:validators:mustBeTextScalar`.
3. Parse `[Rr]` + 4 ASCII digits + `[aAbB]`, exact length 6; else
   `invalidReleaseInput`.
4. Parse the pin the same way; result `pin < requested` comparing
   `(year, half)`.

### chdir
1. Outputs ≤ 1; more than one input → `RunMat:chdir:InvalidInput`.
2. No input → `cd` query.
3. One input: if it is not a char row / string scalar → `RunMat:string`.
   Expand it with `cd`'s `input::path` (empty text keeps
   `RunMat:cd:EmptyFolder`); `runmat_filesystem::metadata_async` on the
   expanded path; missing or not a directory → `RunMat:cd:NonExistentFolder`.
4. Call `cd`'s `execute::run([folder])`; its result is the previous folder
   as a char row.

### date
1. Outputs ≤ 1. Any argument → `RunMat:TooManyInputs`.
2. `call_builtin_async("clock", [])` → 1×6 tensor; take year, month, day.
3. Format `format!("{day:02}-{MON}-{year:04}")` with the English month table;
   return `CharArray::new_row`.

## Test plan

Inline `#[cfg(test)]` modules `isunix_tests`, `ismac_tests`,
`is_matlab_release_older_than_tests`, `chdir_tests` and `date_tests`,
calling through `crate::call_builtin_async` (output-count tests call the
builtin function directly under `push_output_count`).

- One `observed_*` test per observed case (29). Platform tests that assert
  Linux values are `#[cfg(target_os = "linux")]`; the documented-choice
  test checks the `cfg!` mapping on any host.
- `chdir_tests` hold `REPL_FS_TEST_LOCK` (the lock the existing `cd` tests
  use) and a guard that restores the start folder; folders come from
  `tempfile::tempdir()`, never the real temp folder.
- `date_tests` call `clock` before and after `date` and accept either
  date, so a midnight crossing cannot fail the test.
- `documented_choice_*` and `unresolved_*` tests cover FR-053-22..27.

## Risks & rollback

- The release truth table depends on the compatibility pin. The observed
  tests assert the pin is `R2026a` first, so a pin change shows up as one
  clear failure.
- `chdir` changes process state; it shares the repl_fs lock with `cd`.
- Rollback: delete the four files and their `mod` lines.

## Verification commands

- `cargo test -p runmat-runtime --lib -- isunix_tests ismac_tests is_matlab_release_older_than_tests chdir_tests date_tests`
- `cargo test -p runmat-runtime --lib -- builtins::io::repl_fs::cd builtins::io::repl_fs::environment builtins::datetime` (regressions)
- `rustfmt --edition 2021 --check <files>`
- `cargo clippy -p runmat-runtime --lib --tests --message-format short`,
  filtered to the four files
