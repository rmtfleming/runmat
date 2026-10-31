# Implementation Plan — 059-graph-n3n4 (specs 0.2.0)

## Constitution Check

- I Specification-first: FR-059-01..16 cite the 28 approved normative
  claims; U-059-1..7 are labelled documented choices; FR-059-18 carries the
  unresolved questions. PASS.
- II Clean-room: inputs are the imported exports, RunMat's graph subsystem
  and textbook graph algorithms; MATLAB not invoked. PASS.
- III Independence: backtracking isomorphism with invariant pruning,
  depth-first simple-path enumeration, edge-list filter. PASS.
- IV Provenance: `provenance.md` pinned to `1156c97` (isomorphism) and
  `fc9f804` (allpaths, inedges), freshness-checked, claim ids verified
  against the imported `provenance.yaml`. PASS.
- V Test-first: tests written first against registered stubs; the failing
  run is recorded in `validation.md` before the implementation. PASS.
- VII Minimal scope: two new files plus two `mod` lines in the hand-written
  `graph/domain/mod.rs`; `structure.rs` (F36), `editing.rs` (F43),
  `addnode.rs` (F52) and upstream graph code are not changed
  (D-059-1..3 documented, not fixed). PASS.
- IX Compatibility: `builtins/graph/domain/`, `#[runtime_builtin]` with
  descriptors and `accel = "cpu"`, upstream `graph_data`, table helpers,
  `CellArray`. No `integer_capabilities`. PASS.

## Affected files

| Item | Path |
|---|---|
| `isomorphism` + tests (new) | `crates/runmat-runtime/src/builtins/graph/domain/isomorphism.rs` |
| `allpaths`, `inedges` + tests (new) | `crates/runmat-runtime/src/builtins/graph/domain/paths.rs` |
| Registration | `mod isomorphism;` and `mod paths;` in the hand-written `crates/runmat-runtime/src/builtins/graph/domain/mod.rs` |

## Design

- Errors: per-builtin `BuiltinErrorDescriptor` tables (`RunMat:<fn>:<Id>`):
  common `InvalidInput`, `UnsupportedOption`, `Internal`,
  `UndefinedFunction`, `TooManyOutputs`, then the observed identifiers.
- Output count: `None` → first output; `Some(0|1|2)` → `OutputList`;
  more → `TooManyOutputs` (U-059-3).
- `isomorphism`:
  - read both graphs (non-graph → `UndefinedFunction`), class check
    (`ClassMismatch`), parse name-value options, resolve node/edge
    variables into per-node / per-edge label ids interned across both
    graphs (`NodeVarNotInFirstGraph`, U-059-4(e–g)).
  - Labelled multigraph: per-node label, map `(u, v)` → sorted list of edge
    label ids (undirected edges stored under both orientations), per-node
    invariant (label, out-links, in-links, self-loop labels).
  - Quick rejection: node/edge counts, sorted invariant multisets.
  - Search order: most-connected first, then most links into the ordered
    prefix (ties: lower index). Iterative backtracking (no recursion depth
    limit); candidate `w` for `v` must be unused, have equal invariant, and
    equal pair lists to every mapped node in both directions. The first
    complete map is returned (U-059-4(a)).
  - `P(i) = map(i) + 1`; `edgeperm` per U-059-5; shapes per FR-059-03.
- `allpaths`: resolve `s`, `t` (U-059-6 errors); neighbour lists sorted by
  node with the lowest-index edge per neighbour, self-loops skipped,
  digraphs use out-edges; iterative DFS with an on-path mask; results as
  cells (`double` rows or cellstr rows).
- `inedges`: digraph check (`NoInEdgesUndir`), node resolution
  (`InvalidNodeScalar`, `InvalidNodeID`, `UnknownNodeName`), filter stored
  edges by target.

## Test plan (inline `#[cfg(test)]` modules `graph_n3n4_isomorphism_tests`, `graph_n3n4_paths_tests`)

Graphs are built by calling `graph`/`digraph` through
`crate::call_builtin_async`; multi-output forms through
`push_output_count` + direct call of the builtin function.

- `observed_*`: every observed case of the three exports (isomorphism:
  validity of `P` via `adjacency`, plus the exact value where unique;
  allpaths: path sets).
- `rejects_*`: ClassMismatch, NodeVarNotInFirstGraph, InvalidNodeID,
  UnknownNodeName, NoInEdgesUndir, InvalidNodeScalar.
- `unresolved_choice_*`: U-059-1..7.
- `deviation_*` (`#[ignore]`, D-059-1/2): observed values on the observed
  (unsorted) constructor input.

## Analysis (B3)

- Coverage: every normative claim maps to an FR (provenance table) and to
  at least one planned test; every U-item has an `unresolved_choice_*`
  test. No FR without a claim or a documented-choice label.
- Consistency: the edge-order deviations (D-059-1/2) and the edgeperm
  definition (U-059-5) were checked by hand against the observed graphs:
  with RunMat's stored order, U-059-5 yields the observed `[1..6]'` for the
  `edgeperm` case; `inedges` `eid` matches, `nid` does not (D-059-1);
  `allpaths` node paths match, `E` does not (D-059-2).
- Ambiguity resolved by choice: the observed `edgevars` statement says the
  weights were not compared by the harness; FR-059-05 requires only that
  the returned permutation pairs equal weights, which the unique observed
  value `[3 2 1]` satisfies.
- Risk: `isomorphism` is exponential in the worst case (inherent);
  `allpaths` output is exponential in the graph size (inherent).
- No constitution conflicts; no CRITICAL findings.

## Risks & rollback

- D-059-1/2/3 depend on the upstream constructor; reported, not fixed.
- The generated `graph/mod.rs` wasm re-exports are not edited; wasm
  registration waits for the next module-composition regeneration (as for
  features 036/043).
- Rollback: delete the two files and their `mod` lines.

## Verification commands

```
export HDF5_DIR=/home/rfleming/anaconda3
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=<scratchpad>/run.sh
cargo test -p runmat-runtime --lib -- graph_n3n4
cargo test -p runmat-runtime --lib -- builtins::graph
rustfmt --edition 2021 --check crates/runmat-runtime/src/builtins/graph/domain/{isomorphism,paths}.rs
cargo clippy -p runmat-runtime --lib --tests --message-format short
```
