# Implementation Plan — 003-path-parts

## Constitution Check
Same profile as 002 — PASS on I–V, VII, IX (single builtin, test-first,
conventions followed; provenance interim-pin waiver applies).

## Affected crates and files

| Item | Path |
|---|---|
| `fileparts` | `crates/runmat-runtime/src/builtins/io/repl_fs/fileparts.rs` (new) |
| Registration | `io/repl_fs/mod.rs` module entry |

Exemplars: `io/repl_fs/fullfile.rs` (path text handling, char/string I/O),
`math/reduction/max.rs` (`BuiltinOutputMode::ByRequestedOutputCount` +
`output_count::current_output_count()` + `output_list_with_padding`).

## Design decisions

- Pure text split, no fs access. Algorithm (independent, documented):
  1. Strip one trailing separator (observed: trailing-sep case).
  2. Directory part = text before last separator ('' when none — observed
     name-only case; 0×0 empty char).
  3. Name/ext = last component split at last '.' (leading-dot component has
     no ext) — summary-derived choice (fileparts.q-outputs).
- Output mode `ByRequestedOutputCount`, max 3 outputs; nargout==1 → dir part
  (matches observation harness which recorded output 1).
- Char input → char outputs (empty = 0×0 `CharArray`); string input → string
  outputs (tagged unobserved).

## Test plan

Normative: four observed cases exactly (`'/a/b'` 1×4, `''` 0×0 twice,
`'/x/y'` 1×4) with class checks. Summary-derived (`summary_derived` tags):
name/ext outputs for the same inputs per documented choice.
Unresolved-tracking: dotfile, multi-dot, string input.

## Risks & rollback

- Risk: RunMat empty-char shape conventions (0×0 vs 1×0) — verify against
  `CharArray` constructors during T1; observed requirement is 0×0.
- Rollback: remove file + mod entry.

## Verification

`cargo fmt --all -- --check` · `cargo check -p runmat-runtime` ·
`RUST_TEST_THREADS=1 cargo test -p runmat-runtime fileparts`
