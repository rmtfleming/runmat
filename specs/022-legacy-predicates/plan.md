# Implementation Plan — 022-legacy-predicates

## Constitution Check

- I Specification-first: every requirement cites approved claims — PASS
- II Clean-room: sources = imported Tier B batch-2 export + RunMat code only;
  MATLAB not invoked — PASS
- III Independence: predicates derived from RunMat `Value` kinds and the
  existing `runmat_filesystem` abstraction — PASS
- IV Provenance: `provenance.md` complete; revision pinned to source commit
  `4445294` — PASS
- V Test-first: acceptance criteria defined before bodies; observed cases
  encoded as tests — PASS
- VII Minimal scope: four tightly coupled legacy predicates, no unrelated
  refactoring — PASS
- IX Compatibility: follows `introspection/` and `io/repl_fs/` conventions;
  `#[runtime_builtin]` registration; `Value` semantics — PASS
- X Licensing: no external implementation code imported — PASS

## Affected crates and files

| Item | Path |
|---|---|
| `isobject` | `crates/runmat-runtime/src/builtins/introspection/isobject.rs` (new) |
| `isstr` | `crates/runmat-runtime/src/builtins/introspection/isstr.rs` (new) |
| `isfolder` | `crates/runmat-runtime/src/builtins/io/repl_fs/isfolder.rs` (new) |
| `isdir` | `crates/runmat-runtime/src/builtins/io/repl_fs/isdir.rs` (new) |
| Registration | `introspection/mod.rs` (+`isobject`, +`isstr`); `io/repl_fs/mod.rs` (+`isdir`, +`isfolder`) |
| Docs metadata | descriptor + `#[runtime_builtin]` doc fields per builtin |

Exemplars:
- `isobject` — `introspection/isa.rs` (metadata predicate; object-kind
  inspection, GPU spec, descriptor, inline tests).
- `isstr` — `introspection/ischar.rs` (legacy alias; mirror `matches!(v,
  Value::CharArray(_))` exactly).
- `isfolder` — `io/repl_fs/isfile.rs` (async filesystem predicate; `path_text`
  helper; `runmat_filesystem::metadata_async` — `is_dir()` instead of
  `is_file()`).
- `isdir` — legacy alias of `isfolder`; delegates to a shared helper.

## Design decisions

- `isobject_value(&Value) -> bool`: `matches!(v, Value::Object(_) |
  Value::HandleObject(_))`. GPU handles / all fundamental kinds → false
  directly (metadata-only; `accel = "metadata"`, no provider call).
- `isstr`: sync `fn` returning `Value::Bool(matches!(v, Value::CharArray(_)))`
  — byte-for-byte semantic mirror of `ischar`. Local `bool_scalar_type`
  resolver.
- `isfolder`: async; accept `Value::CharArray` (rows ≤ 1) / `Value::String` /
  1-element `Value::StringArray` as path text; resolve via
  `runmat_filesystem::metadata_async(&text)` and answer `meta.is_dir()`;
  missing/error/empty/non-text → `false`. A shared `pub(crate) async fn
  folder_predicate(Value)` holds the logic.
- `isdir`: async; legacy alias — its builtin body calls
  `isfolder::folder_predicate(path).await`. Chosen over
  `call_builtin_async("isfolder", …)` to avoid a name-dispatch round-trip and
  keep a single, directly-tested source of truth. Documented in the file
  doc-comment.
- Output mode `Fixed`, type_resolver `bool_scalar_type` (local per file, so the
  shared `type_resolvers.rs` is not touched), for all four.
- GPU/fusion specs: `isobject`/`isstr` metadata (`InheritInputs`, no
  precisions, mirror `ischar`); `isfolder`/`isdir` I/O (`GatherImmediately`,
  mirror `isfile`).

## Test plan (inline `#[cfg(test)]`, per house convention)

Normative (cite FR/claims in test names or comments), tier prefix
`observed_*`:
- isobject: object→true; numeric, char, cell → false.
- isstr: char→true; numeric, string, cell → false.
- isfolder: existing folder (`tempdir`) → true; missing (self-cleaning path) →
  false.
- isdir: existing folder → true; missing → false.

Unresolved-tracking (tier prefix `unresolved_choice_*`):
- isobject: value-class `Value::Object` → true; struct/string/function-handle
  → false; GPU tensor → false; wgpu numeric → false (feature-gated).
- isstr: 2-D char matrix + empty char → true; logical → false; GPU tensor →
  false; wgpu numeric → false (feature-gated).
- isfolder / isdir: a regular file → false (`q-file`); non-text input → false.

## Risks & rollback

- Risk: object-kind representation differs from assumption — resolved by
  inspecting the `Value` enum (`Value::Object`, `Value::HandleObject`) and
  `input_parser.rs` (constructs `Value::HandleObject`); requirement is
  kind-level.
- Risk: five sibling features build concurrently — touch only the four new
  files and the specific `mod.rs` lines; on an unrelated compile error, wait
  and re-run rather than edit.
- Rollback: delete the four new files + the four `mod.rs` line additions (no
  existing file is otherwise modified).

## Verification commands

`cargo check -p runmat-runtime` ·
`RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::introspection::isobject builtins::introspection::isstr builtins::io::repl_fs::isfolder builtins::io::repl_fs::isdir` ·
`cargo fmt` then `cargo fmt --all -- --check`
