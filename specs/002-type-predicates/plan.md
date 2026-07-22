# Implementation Plan — 002-type-predicates

## Constitution Check

- I Specification-first: requirements cite approved claims — PASS
- II Clean-room: sources = imported export + RunMat code only; no MATLAB — PASS
- III Independence: predicates derived from RunMat `Value` kinds — PASS
- IV Provenance: `provenance.md` complete; interim pin waiver recorded — PASS
- V Test-first: tests authored before builtin bodies (tasks T1–T3 before T4–T6) — PASS
- VII Minimal scope: three tightly coupled predicates, no refactoring — PASS
- IX Compatibility: follows `logical/tests/` and `io/repl_fs/` conventions — PASS

## Affected crates and files

| Item | Path |
|---|---|
| `iscell` | `crates/runmat-runtime/src/builtins/logical/tests/iscell.rs` (new) |
| `isstruct` | `crates/runmat-runtime/src/builtins/logical/tests/isstruct.rs` (new) |
| `isfile` | `crates/runmat-runtime/src/builtins/io/repl_fs/isfile.rs` (new) |
| Registration | add `mod` entries in `logical/tests/mod.rs`(or `logical/mod.rs` tree) and `io/repl_fs/mod.rs` |
| Docs metadata | descriptor + `#[runtime_builtin]` doc fields (feeds `docs/builtins` tooling) |

Exemplar: `logical/tests/isnumeric.rs` (descriptor, GPU spec, error builder,
inline test module). `isfile` exemplar additionally: `io/repl_fs/exist.rs`
(filesystem access pattern; respects the runtime filesystem provider).

## Design decisions

- `iscell_value(&Value) -> bool`: `matches!(v, Value::Cell(_))`. GPU handles:
  gather NOT required — a GPU tensor is not a cell/struct; return false
  directly (metadata-only, mirrors `isnumeric` accel = "metadata" approach
  without provider call).
- `isstruct_value`: `matches!(v, Value::Struct(_))`. (RunMat represents
  struct arrays via `Value::Struct` — verify at implementation; if struct
  arrays are a distinct kind, include it. Requirement is kind-level, from
  isstruct.logical-struct-test: scalar struct AND struct array → true.)
- `isfile`: accept `Value::CharArray` / `Value::String` path text; resolve
  via the same filesystem abstraction as `exist.rs`; regular file → true,
  missing → false, folder → false (documented choice, unresolved
  isfile.q-folder). Output `Value::Bool`.
- Output mode `Fixed`, type_resolver `bool_scalar_type` for all three.

## Test plan (inline `#[cfg(test)]`, per house convention)

Normative (cite FR/claims in test names or comments):
- iscell: cases cell-array, empty-cell → true; numeric-scalar, char-row,
  struct → false.
- isstruct: struct, struct-array → true; numeric, cell → false.
- isfile: missing path → false; class/shape checks.
Summary-derived (tagged `summary_derived` in name): isfile existing temp file
→ true (create in std::env::temp_dir, remove after).
Unresolved-tracking (tagged `unresolved_choice`): isfile on a folder → false.

## Risks & rollback

- Risk: struct-array representation differs from assumption — resolve during
  T1 by inspecting `Value`; requirement is unaffected (kind-level).
- Risk: WASM build needs registry regeneration — handled by macro
  `builtin_path`; verify with `cargo check -p runmat-runtime`.
- Rollback: delete the three new files + mod entries (no existing file is
  modified beyond `mod.rs` line additions).

## Verification commands

`cargo fmt --all -- --check` · `cargo check -p runmat-runtime` ·
`RUST_TEST_THREADS=1 cargo test -p runmat-runtime iscell isstruct isfile`
