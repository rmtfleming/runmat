# Implementation Plan — 062-graph-n5 (specs 0.2.0)

## Constitution Check

- I Specification-first: FR-062-01..13 cite the 18 approved normative
  claims; U-062-1..9 are labelled documented choices; FR-062-15 carries the
  unresolved questions. PASS.
- II Clean-room: inputs are the imported exports and RunMat's graph and
  table subsystems; MATLAB not invoked. PASS.
- III Independence: edge-list filter; inverse permutation + stable sort.
  PASS.
- IV Provenance: `provenance.md` pinned to `1c91aa1`, freshness-checked,
  claim ids verified against the imported `provenance.yaml`. PASS.
- V Test-first: tests written first against registered stubs; the failing
  run is recorded in `validation.md` before the implementation. PASS.
- VII Minimal scope: two new files, two `mod` lines in the hand-written
  `graph/domain/mod.rs`, and nine F59 helpers in `paths.rs` widened from
  private to `pub(super)` (no behaviour change). `structure.rs`,
  `editing.rs`, `addnode.rs`, `isomorphism.rs` and upstream graph code are
  not changed (D-062-1/2 documented, not fixed). PASS.
- IX Compatibility: `builtins/graph/domain/`, `#[runtime_builtin]` with
  descriptors and `accel = "cpu"`, upstream `graph_data`/`edges_table`,
  table `select_rows`. No `integer_capabilities`. PASS.

## Affected files

| Item | Path |
|---|---|
| `outedges` + tests (new) | `crates/runmat-runtime/src/builtins/graph/domain/outedges.rs` |
| `reordernodes` + tests (new) | `crates/runmat-runtime/src/builtins/graph/domain/reordernodes.rs` |
| Registration | `mod outedges;`, `mod reordernodes;` in the hand-written `crates/runmat-runtime/src/builtins/graph/domain/mod.rs` |
| Shared helpers (visibility only) | `crates/runmat-runtime/src/builtins/graph/domain/paths.rs`: `query_error`, `graph_input`, `check_output_count`, `outputs`, `NodeArg`, `resolve_node`, `double`, `cell`, `name_value` → `pub(super)` |

## Design

- Errors: per-builtin `BuiltinErrorDescriptor` tables (`RunMat:<fn>:<Id>`):
  common `InvalidInput`, `UnsupportedOption`, `Internal`,
  `UndefinedFunction`, `TooManyOutputs`, then the observed identifiers
  (and `reordernodes:UnknownNodeName`, U-062-7). Raised through F59's
  `query_error`.
- `outedges`: mirror of `inedges`: `graph_input`, arity checks,
  `resolve_node`; select stored edges with `source == node` (plus
  `target == node` for graphs, other end as `nid`); `nid` class from the
  query value (char/cell → cell of char, string → string array, numeric →
  double).
- `reordernodes`: parse `order` (U-062-7) into old indices; inverse
  permutation; renumber each edge (graphs: smaller end first); stable sort
  by `(s, t)` → `idx`. Clone the input object; `Edges` table rebuilt with
  `EndNodes` from `edges_table` of the reordered `GraphData` and every
  other variable row-selected by `idx`; `Nodes` table rebuilt with every
  variable row-selected by `order` (synthetic `Index` renumbered, U-062-9).

## Test plan (inline `#[cfg(test)]` modules `graph_n5_outedges_tests`, `graph_n5_reordernodes_tests`)

- `observed_*`: every observed case of both exports.
- `rejects_*`: the five observed error identifiers.
- `unresolved_choice_*`: U-062-1..9.
- `deviation_*` (`#[ignore]`, D-062-1): observed values on the observed
  (unsorted / adjacency) constructor input.

## Analysis (B3)

- Coverage: every normative claim maps to an FR (provenance table) and to
  at least one planned test; every U-item has an `unresolved_choice_*`
  test. No FR without a claim or a documented-choice label.
- Consistency: hand-checked against the observations — `weighted`:
  old→new `2→1, 3→2, 1→3`; edge 1 (1→2) becomes 3→1, edge 2 (2→3) becomes
  1→2; sorted `[1 2; 3 1]`, `idx = [2;1]`, `Weight = [9;7]` ✓.
  `undirected`: `[1 2; 2 3]` after canonicalisation ✓. `names`: new ids
  `c=1, a=2, b=3`; `a→b = 2→3`, `b→c = 3→1`; sorted end nodes by name
  `[a b; b c]` ✓. `met-graph`/`idx-out` on sorted storage: `idx = [3;1;2]` ✓;
  on adjacency storage `[1;2;3]` (D-062-1). `outedges` `two-out` on the
  observed input: `nid = [3;2]` (D-062-1).
- Ambiguity resolved by choice: the export does not say how equal end-node
  pairs are ordered (U-062-8) or what happens to RunMat's synthetic
  `Index` column (U-062-9).
- No constitution conflicts; no CRITICAL findings.

## Risks & rollback

- D-062-1/2 depend on the upstream constructor; reported, not fixed.
- The generated `graph/mod.rs` wasm re-exports are not edited (as features
  036/043/059).
- Rollback: delete the two files and their `mod` lines; revert the
  `pub(super)` visibility in `paths.rs`.

## Verification commands

```
export HDF5_DIR=/home/rfleming/anaconda3
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=<scratchpad>/run.sh
cargo test -p runmat-runtime --lib -- graph_n5
cargo test -p runmat-runtime --lib -- builtins::graph
rustfmt --edition 2021 --check crates/runmat-runtime/src/builtins/graph/domain/{outedges,reordernodes,paths}.rs
cargo clippy -p runmat-runtime --lib --tests --message-format short
```
