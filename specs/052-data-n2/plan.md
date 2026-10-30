# Implementation Plan — 052-data-n2 (specs 0.2.0)

## Constitution Check

- I Specification-first: FR-052-01..15 cite the 23 approved normative
  claims; U-052-1..9 are labelled documented choices; FR-052-18 carries the
  unresolved questions. PASS.
- II Clean-room: inputs are the imported exports and RunMat's own graph,
  categorical and file-system code; MATLAB not invoked. PASS.
- III Independence: node-list append; property read; a small tokenizer.
  PASS.
- IV Provenance: `provenance.md` pinned to `8f72edb`, freshness-checked,
  claim ids verified against the imported `provenance.yaml`. PASS.
- V Test-first: tests written first against registered stubs; the failing
  run is recorded in `validation.md` before the implementation. PASS.
- VII Minimal scope: three new files plus one `mod` line in each of three
  hand-written parents; no upstream code changed. PASS.
- IX Compatibility: `#[runtime_builtin]` with descriptors and `accel =
  "cpu"`, upstream `graph_data`/`graph_object`, `categorical_categories`,
  `runmat_filesystem`. No `integer_capabilities`/`integer_audit`. PASS.

## Affected files

| Item | Path |
|---|---|
| `addnode` + tests (new) | `crates/runmat-runtime/src/builtins/graph/domain/addnode.rs` |
| Registration | `mod addnode;` in hand-written `crates/runmat-runtime/src/builtins/graph/domain/mod.rs` |
| `categories` + tests (new) | `crates/runmat-runtime/src/builtins/table/builtins/categories.rs` |
| Registration | `mod categories;` in hand-written `crates/runmat-runtime/src/builtins/table/builtins.rs` |
| `textread` + tests (new) | `crates/runmat-runtime/src/builtins/io/filetext/textread.rs` |
| Registration | `pub mod textread;` in hand-written `crates/runmat-runtime/src/builtins/io/filetext/mod.rs` |

No generated parent `mod.rs` is touched.

## Design

- `addnode`: gather; output-count guard; graph check (`graph_data`);
  classify the second input: numeric scalar → count (negative, NaN, Inf or
  fractional → `InvalidNrNodes`; named graph → `UnsupportedOption`);
  text (`parse_string_vector`) → names (empty name → `UnsupportedOption`;
  unnamed non-empty graph → generate `Node1..NodeN`; duplicates against
  existing names or within the list → `NonUniqueNames`); else
  `UnsupportedOption`. Edges copied unchanged; `graph_object`. Its own small
  error helper (F43's helpers in `editing.rs` are private to that file).
- `categories`: gather; output-count guard; one input; categorical object →
  `categorical_categories`; build an n×1 `Cell` of `CharArray` rows (0×0
  when n = 0).
- `textread`: gather; parse filename; format (char/string) or none;
  name/value options (`headerlines`, `delimiter`); read bytes via
  `runmat_filesystem::read_async` (error → `FileNotFound`); decode UTF-8
  (lossy); drop header lines; tokenize (whitespace, or delimiter + line
  ends with trimming); assign tokens cyclically to fields; build columns;
  return per U-052-8 (`OutputList` when a count is requested, padded never —
  more than available is `TooManyOutputs`). Default format: per-line
  numeric rows → column-major matrix.

## Test plan (inline `#[cfg(test)]` modules `addnode_tests`, `categories_tests`, `textread_tests`)

- `observed_*`: every observed case of the three exports (textread cases
  write their own files into a `tempfile::tempdir()`).
- `rejects_*`: NonUniqueNames, InvalidNrNodes, FileNotFound.
- `unresolved_choice_*`: U-052-1..9 (output counts tested by calling the
  builtin function directly under `push_output_count`).

## Risks & rollback

- `categories` order depends on upstream's `categorical` constructor; if it
  disagrees with an observation, a D-052-n deviation is recorded (no
  upstream change).
- The generated `graph/mod.rs`/`io/mod.rs` wasm re-exports are not edited;
  wasm registration waits for the next module-composition regeneration.
- Rollback: delete the three files and their `mod` lines.

## Verification commands

```
export HDF5_DIR=/home/rfleming/anaconda3
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=<scratchpad>/run.sh
cargo test -p runmat-runtime --lib -- addnode_tests categories_tests textread_tests
cargo test -p runmat-runtime --lib -- builtins::graph builtins::table builtins::io
rustfmt --edition 2021 --check <the three files>
cargo clippy -p runmat-runtime --lib --tests --message-format short
```
