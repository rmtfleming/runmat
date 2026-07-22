# Implementation Plan — 032-object-introspection

## Constitution Check

- I Specification-first: every FR cites an approved claim id — PASS
- II Clean-room: sources = imported exports + RunMat code only; no MATLAB
  process/MCP/Engine used in any step (tests scan real temp folders through
  `runmat_filesystem`) — PASS
- III Independence: member lists derived from RunMat's own class registry;
  `what` classification is an independent RunMat scheme — PASS
- IV Provenance: `provenance.md` complete; commit pin
  `1b019121bb747e63d69a3eb005534cb60669e5f4` recorded — PASS
- V Test-first: acceptance criteria + tiered tests authored alongside the
  builtins; shape/class + non-assertion + unresolved-choice coverage — PASS
- VI Traceability: FR→test map in `validation.md` — PASS
- VII Minimal scope: three tightly coupled introspection builtins; no
  unrelated refactoring — PASS
- IX Compatibility: `properties`/`methods` follow the `introspection/`
  descriptor + GPU/fusion spec conventions (exemplars `class.rs`,
  `fieldnames.rs`); `what` follows `io/repl_fs/dir.rs` — PASS
- X Licensing: no external implementation code imported — PASS

## Affected crates and files

| Item | Path |
|---|---|
| `properties` | `crates/runmat-runtime/src/builtins/introspection/properties.rs` (new) |
| `methods` | `crates/runmat-runtime/src/builtins/introspection/methods.rs` (new) |
| `what` | `crates/runmat-runtime/src/builtins/io/repl_fs/what.rs` (new) |
| Registration | `pub mod` entries in `introspection/mod.rs` and `io/repl_fs/mod.rs` |
| Docs metadata | descriptor + `#[runtime_builtin]` doc fields (feeds `docs/builtins`) |

**`what` placement — justification**: `what` scans the filesystem, exactly as
`dir` and `ls` do, through `runmat_filesystem`. It belongs in `io/repl_fs/`
next to those builtins, not in `introspection/` (which is class-registry
introspection). This keeps the filesystem-provider access pattern in one
place and reuses `expand_user_path`/`path_to_string`/`is_rooted_path`.

## Design decisions

- **Shared resolver**: `properties::class_name_of_argument(&Value)` maps
  `Object`/`HandleObject`/`ClassRef` to their class name and a char/string
  scalar to a class name (documented text form). `methods.rs` reuses it.
- **Member gathering** walks the `ClassDef` parent chain via
  `runmat_builtins::get_class`, collecting members whose
  `get_access`/`access` is `Public` into a `BTreeSet<String>` (dedup +
  ascending order, a documented choice). Output = N×1 cell of char rows via
  `crate::make_cell(names, rows, 1)` (the `fieldnames` cellstr-column
  pattern).
- **Non-assertion enforced in code + tests**: the observed shape test asserts
  only `cell` class + column + all-char; a dedicated
  `unresolved_choice_*_count_not_asserted` test documents that MATLAB's 9/19
  are not claimed.
- **`what`**: accept a char/string folder (or no argument → cwd), resolve via
  `expand_user_path` + `canonicalize_async`, scan with `read_dir_async`,
  classify entries (extension → file buckets; `@`/`+` folders → classes /
  packages), and build a 1×1 `Value::Struct` with fields `m`, `mat`, `mex`,
  `mlx`, `mlapp`, `classes`, `packages` (N×1 cellstr columns) and `path`
  (char row). Async builtin like `dir`.
- Output mode `Fixed` for all three; no `type_resolver` (matches
  `input_parser` sibling — the outputs are cell/struct, not scalar-typed).
- GPU/fusion specs registered host-only (`Custom("introspection")` /
  `Custom("io")`), mirroring `class.rs` / `dir.rs`; introspection never
  touches device memory.

## Test plan (inline `#[cfg(test)]`, house convention)

Tiers (per the constitution + task):

- `observed_*` — reproduce the export shape/class using RunMat's `inputParser`
  (constructed via `crate::call_builtin("inputParser", &[])`) for
  properties/methods, and a real temp folder for `what`.
- `unresolved_choice_*` — empty/no-member classes → 0×1 column; class-name
  text form; private-member exclusion; unknown-class + non-object errors;
  count-non-assertion (`assert_ne!(len, 9|19)`); `what` classification,
  empty-folder, no-argument, arity/type errors.
- Descriptor / registration / GPU-fusion-name sanity tests.

## Risks & rollback

- Risk: `inputParser` class not registered on the test thread → mitigated by
  constructing it through the registry inside each test (registers lazily);
  `RUST_TEST_THREADS=1` keeps the thread-local registry shared.
- Risk: filesystem tests racing on cwd → guarded by `REPL_FS_TEST_LOCK`
  (shared with `dir`/`ls`).
- Rollback: delete the three new files + the three `mod.rs` line additions;
  no existing file is otherwise modified.

## Verification commands

`cargo check -p runmat-runtime` ·
`RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::introspection::properties builtins::introspection::methods what` ·
`cargo fmt --all -- --check` · `cargo clippy -p runmat-runtime --lib`
