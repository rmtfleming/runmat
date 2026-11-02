# Implementation Plan — 066-graph-n6 (specs 0.2.0)

## Constitution Check

- I Specification-first: FR-066-01..17 cite the 24 approved normative
  claims; U-066-1..9 are labelled documented choices; FR-066-19 carries the
  unresolved questions. PASS.
- II Clean-room: inputs are the imported exports and RunMat's graph and
  table subsystems; MATLAB not invoked. PASS.
- III Independence: node selection + renumbering + stable sort; exact name
  lookup. PASS.
- IV Provenance: `provenance.md` pinned to `e364206`, freshness-checked,
  claim ids verified against the imported `provenance.yaml`. PASS.
- V Test-first: tests written first against registered stubs; the failing
  run is recorded in `validation.md` before the implementation. PASS.
- VII Minimal scope: two new files and two `mod` lines in the hand-written
  `graph/domain/mod.rs`. No visibility change was needed (`graph_input`,
  `double` in `paths.rs` and `edges_table`, `nodes_table`, `graph_data` in
  `construction.rs` are already `pub(super)`); `reordernodes.rs`'s private
  table helpers are re-stated locally (three small functions) rather than
  shared, so F62 is untouched. Upstream graph code is not changed
  (D-066-1/2 documented, not fixed). PASS.
- IX Compatibility: `builtins/graph/domain/`, `#[runtime_builtin]` with
  descriptors and `accel = "cpu"`, upstream `graph_data`/`edges_table`,
  table `select_rows`. No `integer_capabilities`. PASS.

## Affected files

| Item | Path |
|---|---|
| `subgraph` + tests (new) | `crates/runmat-runtime/src/builtins/graph/domain/subgraph.rs` |
| `findnode` + tests (new) | `crates/runmat-runtime/src/builtins/graph/domain/findnode.rs` |
| Registration | `mod findnode;`, `mod subgraph;` in the hand-written `crates/runmat-runtime/src/builtins/graph/domain/mod.rs` |

## Design

- Errors: per-builtin `BuiltinErrorDescriptor` tables. Index 0..4 are the
  local `RunMat:<fn>:{InvalidInput, UnsupportedOption, Internal,
  UndefinedFunction, TooManyOutputs}` (so F59's `graph_input`, which looks
  up by suffix, finds `UndefinedFunction`/`InvalidInput`); then
  `RunMat:minrhs` and the observed identifiers mapped to
  `RunMat:<fn>:<last component>` (graph-family convention). A local `fail`
  looks up by the exact identifier (`RunMat:minrhs` is outside the
  `<fn>` namespace).
- `subgraph`: output count ≤ 1 (U-066-2); `graph_input`; arity; parse
  `nodeIDs` → ordered zero-based old nodes (mask: original order, vector
  check; numeric: range per entry; text: name lookup per entry; then
  repeats → `InvalidInd`). Map old→new, keep edges with both ends mapped,
  renumber (graphs: smaller end first), stable sort by `(s, t)` → kept
  edge rows. Clone `G`'s object; `NumNodes = k`; `Edges` rebuilt with
  `EndNodes` from `edges_table` of the induced `GraphData` and every other
  variable row-selected; `Nodes` rebuilt with every variable row-selected
  (synthetic `Index` renumbered 1..k).
- `findnode`: output count ≤ 1; `graph_input`; arity; classify `nodeID`:
  logical → `ArgType`; numeric → `PosInt` check, IDs, 0 beyond `n`; char
  row / string / string array / cell (column-major, elements must be
  text) → names, `NoNames` without node names, exact match or 0. Result a
  k×1 double.

## Test plan (inline `#[cfg(test)]` modules `graph_n6_subgraph_tests`, `graph_n6_findnode_tests`)

- `observed_*`: every observed case of both exports (set-up cases
  `source-graph`, `node-names` as set-up checks).
- `rejects_*`: every observed error case.
- `unresolved_choice_*`: U-066-1..9.
- `deviation_*` (`#[ignore]`, D-066-1): the cell class of named
  `EndNodes`.

## Analysis (B3)

- Coverage: every normative claim maps to an FR (provenance table) and to
  at least one planned test; every U-item has an `unresolved_choice_*`
  test. No FR without a claim or a documented-choice label.
- Consistency (hand-checked against the observations):
  `edge-order` `[4 1 2]`: new ids `d=1, a=2, b=3`; `r1` (1→2) → 2→3, `r4`
  (4→1) → 1→2; `r2`, `r3` dropped; sorted `[1 2; 2 3]` = `r4, r1` ✓.
  `index-unsorted` `[3 1 2]`: `c=1, a=2, b=3`; `r1` → 2→3, `r2` (2→3) →
  3→1; sorted `[2 3; 3 1]` = `r1, r2` ✓. `logical-column` `a, c, d`:
  `r3` (3→4) → 2→3, `r4` (4→1) → 3→1 ✓. `multigraph`: stored
  `[1 2; 1 2; 1 1; 2 3]`, kept and sorted `[1 1; 1 2; 1 2]` ✓.
  `undirected` `[2 3]`: edge 2–3 → `[1 2]` ✓. `names`: `c=1, a=2`; `c→a`
  → `[1 2]`, by name `{c, a}` ✓. `findnode` `cell-matrix`: column-major
  `a, c, b, d` → `[1;3;2;4]` ✓.
- Ambiguity resolved by choice: tie order (U-066-5), `graph`-input
  identifiers (U-066-4), unobserved query forms (U-066-9).
- Identifier namespace: first implemented with the full observed
  identifier under `RunMat:` (N6 selection-proposal wording); changed on
  maintainer decision (2026-11-02) to the F36–F62 mapping
  `RunMat:<fn>:<last component>`; recorded in `spec.md`.
- No constitution conflicts; no CRITICAL findings.

## Risks & rollback

- D-066-1/2 depend on the upstream constructor; reported, not fixed.
- The generated `graph/mod.rs` wasm re-exports are not edited (as features
  036/043/059/062).
- Rollback: delete the two files and their `mod` lines.

## Verification commands

```
export HDF5_DIR=/home/rfleming/anaconda3
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=<scratchpad>/run.sh
cargo test -p runmat-runtime --lib -- graph_n6
cargo test -p runmat-runtime --lib -- --ignored graph_n6
cargo test -p runmat-runtime --lib -- builtins::graph
rustfmt --edition 2021 --check crates/runmat-runtime/src/builtins/graph/domain/{subgraph,findnode,mod}.rs
cargo clippy -p runmat-runtime --lib --tests --message-format short
```
