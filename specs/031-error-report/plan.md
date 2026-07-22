# Implementation Plan — 031-error-report

## Constitution Check

- I Specification-first: requirements cite approved claims — PASS
- II Clean-room: sources = imported export + RunMat code only; no MATLAB — PASS
- III Independence: report derived from RunMat's own `MException.message`
  field, not MATLAB internals — PASS
- IV Provenance: `provenance.md` complete; commit pin `1b01912` — PASS
- V Test-first: tests authored with the builtin file; the observed case is
  reproduced exactly through the builtin — PASS
- VI Traceability: FR ↔ claim ↔ test mapping in `validation.md` — PASS
- VII Minimal scope: one builtin, no refactoring — PASS
- IX Compatibility: follows `diagnostics/` conventions (descriptor, macro
  registration, GPU/fusion metadata, inline tiered tests) — PASS

## Affected crates and files

| Item | Path |
|---|---|
| `getReport` | `crates/runmat-runtime/src/builtins/diagnostics/getReport.rs` (new) |
| Registration | add `#[path = "getReport.rs"] pub mod get_report;` to `crates/runmat-runtime/src/builtins/diagnostics/mod.rs` |
| Docs metadata | descriptor + `#[runtime_builtin]` doc fields (feeds `docs/builtins` tooling) |

Category choice: `diagnostics` — `getReport` operates on an `MException`, the
same value kind handled by the other exception builtins `throw`/`rethrow`
(`crates/runmat-runtime/src/builtins/diagnostics/`). Placing it beside
`rethrow.rs` keeps the MException-operating builtins together.

File naming: the source function is `getReport` (camelCase). The file is named
`getReport.rs` to match, and is attached with the house `#[path = "…"]`
convention (as `containers/map/containers.map.rs`) under a snake-case module
`get_report`, so the module path is `crate::builtins::diagnostics::get_report`
and no `non_snake_case` lint is raised.

Exemplars: `introspection/computer.rs` (char row vector output, type resolver,
GPU/fusion metadata-only spec, tiered `observed_*`/`unresolved_choice_*`
tests, stable error identifier), `logical/tests/iscell.rs` (single-`Value`
input, descriptor + spec structure), `diagnostics/rethrow.rs` (MException-
operating builtin placement), `structs/core/getfield.rs`
(`get_exception_field` reads `MException.message`).

## Design decisions

- Single-`Value` input `me: Value` (iscell/rethrow pattern). The wrapper
  `get_report_builtin_registered` (registered under `name = "getReport"`)
  delegates to `get_report_builtin`, which the inline tests call directly.
- `match me { Value::MException(mex) => char_row(&mex.message), _ => Err(…) }`.
  The report is `mex.message` as a `Value::CharArray` row vector
  (`CharArray::new_row`), the observed class [getReport.char-report,
  getReport.output-class].
- Message-only report for every `MException` (FR-031-03): identifier
  formatting and stack rendering are unobserved (`getReport.q-options`); a
  populated stack is ignored and an empty message yields a 1×0 char row —
  documented choices.
- Non-`MException` input → `RunMat:getReport:InvalidInput` (FR-031-04),
  documented choice; the error path is unobserved.
- Output mode `Fixed` (single output), single one-argument signature; the
  `'extended'`/`'basic'`/hyperlink options are omitted per
  `getReport.q-options` (FR-031-05).
- Type resolver: local `get_report_type` → `Type::String` (same as other char
  row vector producers, e.g. `computer`).
- GPU/fusion metadata: host-only value query; `GpuOpKind::Custom`
  ("introspection"), empty precision list, `GatherImmediately`, not fusible
  (mirrors `computer`/`iscell` metadata-only pattern). No `accel` tag — no
  real device path.

## Test plan (inline `#[cfg(test)]`, per house convention)

Normative (cite FR/claims in comments):
- `observed_message_report_is_char_row` — the observed case
  `getReport(MException('x:y','oops'))` reproduces `'oops'` as 1×4 char.
- `observed_output_class_char_row` — a general message returns a char row
  vector whose length tracks the message [getReport.output-class].

Unresolved-tracking (tagged `unresolved_choice`):
- `unresolved_choice_report_is_message_only` — an MException with a populated
  stack and a distinct identifier still reports the message text alone
  (`getReport.q-options`).
- `unresolved_choice_empty_message` — empty message → 1×0 char row.
- `unresolved_choice_non_mexception_input_errors` — a non-MException input is
  rejected with `RunMat:getReport:InvalidInput`.

Registration/metadata: registry lookup (`builtin_function_by_name`),
descriptor shape (1 signature, Fixed output, 1 error), GPU/fusion spec names,
type resolver result.

## Risks & rollback

- Risk: another concurrent Tier B feature touches `diagnostics/mod.rs` — the
  change here is a single two-line `#[path]`/`pub mod get_report;` addition;
  conflicts are trivially mergeable.
- Risk: macro/WASM registry regeneration — handled by `builtin_path` on the
  macro; verified with `cargo check -p runmat-runtime`.
- Rollback: delete `getReport.rs` and the two `mod` lines (no other file
  changes).

## Verification commands

`cargo check -p runmat-runtime` ·
`RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::diagnostics::get_report` ·
`cargo fmt` then `cargo fmt --all -- --check` ·
`cargo clippy -p runmat-runtime --lib`
