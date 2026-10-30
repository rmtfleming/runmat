# Implementation Plan — 043-graph-editing (specs 0.2.0)

## Constitution Check

- I Specification-first: FR-043-01..18 cite the 37 approved normative
  claims; FR-043-06 and U-043-1..9 are labelled documented choices; FR-043-21
  carries the unresolved questions. PASS.
- II Clean-room: inputs are the imported exports, RunMat's graph subsystem
  and textbook graph theory; MATLAB not invoked. PASS.
- III Independence: list edits on upstream `GraphData`; `L = D − A`. PASS.
- IV Provenance: `provenance.md` pinned to `c65e238`, freshness-checked,
  claim ids verified against the imported `provenance.yaml`. PASS.
- V Test-first: tests written first against registered stubs; the failing
  run is recorded in `validation.md` before the implementation. PASS.
- VII Minimal scope: one new file plus one `mod editing;` line in the
  hand-written `graph/domain/mod.rs`; no upstream graph code changed
  (D-043-1/2 documented, not fixed). PASS.
- IX Compatibility: `builtins/graph/domain/`, `#[runtime_builtin]` with
  descriptors and `accel = "cpu"`, upstream `graph_data`/`graph_object`,
  `SparseTensor`. No `integer_capabilities`/`integer_audit`. PASS.

## Affected files

| Item | Path |
|---|---|
| Builtins + tests (new) | `crates/runmat-runtime/src/builtins/graph/domain/editing.rs` |
| Registration | `mod editing;` in `crates/runmat-runtime/src/builtins/graph/domain/mod.rs` |

## Design

- Shared: `gather_values`; output-count guard (more than one requested output
  → `TooManyOutputs`, U-043-3); `graph_input` (non-graph → `UndefinedFunction`,
  malformed → `InvalidInput`); `has_weight_variable` (reads the `Edges`
  table for a `Weight` variable, U-043-9). Errors are raised from per-builtin
  `BuiltinErrorDescriptor` tables (`RunMat:<fn>:<Id>`).
- Node references: numeric (`Num`/`Int`/`Bool`/`Tensor`/`LogicalArray`) or
  text (`String`/`StringArray`/`CharArray`/`Cell`) via upstream parsing
  helpers.
- `rmnode`: keep-mask over nodes; out-of-range indices and unknown names are
  skipped (FR-043-06); edges with both ends kept are remapped in stored order;
  names filtered; `graph_object`.
- `addedge`: parse `s`, `t`; scalar expansion (either side); length check
  (`EqualNumel`); optional `w` (non-numeric → `UnsupportedOption`; no
  `Weight` variable → `NoWeights`; length 1 or m, else `EqualNumel`;
  finite nonnegative). Resolve tokens (indices grow the node count; names
  look up or append). Canonicalise undirected edges (min, max), stable sort
  by (source, target); `graph_object`.
- `rmedge`: two-argument form → edge indices in 1..=m else `EdgeBounds`;
  three-argument form → node refs in 1..=n else `InvalidNodeID`, pairwise
  with scalar expansion, remove all matching edges (either orientation for
  graphs); `graph_object`.
- `laplacian`: digraph → `OnlyGraphSupported`; self-loop → `SelfLoops`;
  repeated unordered pair → `Multigraph`; CSC built column by column with
  sorted rows: −1 for each neighbour, degree on the diagonal (no explicit
  zeros); `SparseTensor::new(n, n, …)`.

## Test plan (inline `#[cfg(test)] mod graph_editing_tests`)

Graphs are built by calling `graph`/`digraph` through
`crate::call_builtin_async`; results are inspected through `G.Edges`,
`G.Nodes`, `NumNodes` and the `numnodes`/`numedges` builtins.

- `observed_*`: every observed case group of the four exports.
- `rejects_*`: NoWeights, EqualNumel, EdgeBounds, InvalidNodeID, SelfLoops,
  Multigraph, OnlyGraphSupported.
- `unresolved_choice_*`: FR-043-06 and U-043-1..9 (output count tested by
  calling the builtin function directly under `push_output_count`).
- `deviation_*` (`#[ignore]`, D-043-1): observed endpoint order of rmnode
  cases `named` and `unknown-name`.

## Risks & rollback

- D-043-1/2 depend on the upstream constructor; reported, not fixed.
- The generated `graph/mod.rs` wasm re-exports are not edited; wasm
  registration waits for the next module-composition regeneration (as for
  feature 036).
- Rollback: delete `editing.rs` and its `mod` line.

## Verification commands

```
export HDF5_DIR=/home/rfleming/anaconda3
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=<scratchpad>/run.sh
cargo test -p runmat-runtime --lib -- graph_editing_tests
cargo test -p runmat-runtime --lib -- builtins::graph
rustfmt --edition 2021 --check crates/runmat-runtime/src/builtins/graph/domain/editing.rs
cargo clippy -p runmat-runtime --lib --tests --message-format short 2>&1 | grep editing
```
