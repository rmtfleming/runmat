# Implementation Plan — 036-graph-structure (spec 0.3.0)

## Constitution Check

- I Specification-first: FR-036-01..07 cite the 19 approved 0.3.0 claims;
  FR-036-08..11 are tagged unresolved or documented choice. PASS.
- II Clean-room: inputs are the imported exports, RunMat code and the
  mathematical definition of graph isomorphism; MATLAB not invoked. PASS.
- III Independence: `incidence` is a direct construction from upstream's
  stored edge list; `isisomorphic` is a textbook backtracking search. PASS.
- IV Provenance: `provenance.md` pinned to `8614c5b`, freshness-checked,
  claim ids verified. PASS.
- V Test-first: the tests were rewritten to 0.3.0 and run against the
  earlier (0.2.0-era) implementation, failing 7 of 17, before the
  implementation was changed. PASS.
- VII Minimal scope: one new file plus one `mod` line in the hand-written
  `graph/domain/mod.rs`; no change to upstream graph code (D-036-1 is
  reported, not fixed). PASS.
- IX Compatibility: lives under `builtins/graph/domain/`, registered via
  `#[runtime_builtin]` with descriptors and `accel = "cpu"`, using
  upstream's `graph_data`. Integer-capability metadata deliberately omitted
  (it ties a builtin to the generated `docs/builtins/meta.json`). PASS.

## Affected files

| Item | Path |
|---|---|
| Builtins + tests (new) | `crates/runmat-runtime/src/builtins/graph/domain/structure.rs` |
| Registration | `mod structure;` in `crates/runmat-runtime/src/builtins/graph/domain/mod.rs` (hand-written; the generated `graph/mod.rs` is not touched) |

## Design

- `graph_input(name, value)`: gather; a value that is not a `graph`/`digraph`
  object → `UndefinedFunction`; a malformed graph object (rejected by
  `graph_data`) → `InvalidInput`.
- `incidence(args...)`: no args → `InvalidInput`; more than one →
  `UnsupportedOption`; any self-loop → `SelfLoops`. Graphs and digraphs are
  treated alike. CSC built directly: column `j` = stored edge `j`, two
  sorted row indices, −1 at `edge.source` (EndNodes(j,1)), +1 at
  `edge.target` (EndNodes(j,2)). Returns `Value::SparseTensor`.
- `isisomorphic(args...)`: fewer than 2 → `InvalidInput`; either input not a
  graph → `UndefinedFunction`; more than 2 → `UnsupportedOption`; graph vs
  digraph → `ClassMismatch`; more than one requested output →
  `UnsupportedOption` (descriptor uses `ByRequestedOutputCount`).
  - Multiplicity matrix `M[u][v]` (undirected edges counted both ways, a
    self-loop adds 1 on the diagonal).
  - Quick rejects: node count, edge count, sorted per-node invariants
    `(out, in, loops)`.
  - Search order: greedy, most links into already-ordered nodes first.
  - Backtracking with consistency checks against all mapped nodes.
- Error descriptors per builtin: `InvalidInput`, `UnsupportedOption`,
  `Internal`, `UndefinedFunction`, plus `SelfLoops` (incidence) /
  `ClassMismatch` (isisomorphic), all as `RunMat:<fn>:<Id>`.

## Test plan (inline `#[cfg(test)] mod graph_structure_tests`)

Tests call builtins through `crate::call_builtin_async`
(`call_builtin_async_with_outputs` for the second-output case).

- `observed_*`: every observed case group (incidence graph/digraph/
  multigraph/weights/shapes/stored edge order; isisomorphic true and false
  case groups).
- `rejects_*`: SelfLoops, UndefinedFunction (both builtins), ClassMismatch.
- `unresolved_choice_*`: named nodes, extra args/arity, options, second
  output, search cases (C6 vs two triangles etc.).
- `deviation_*` (`#[ignore]`, D-036-1): the observed absolute values for
  unsorted input, kept as an executable record of the gap.

## Risks & rollback

- D-036-1: RunMat's graph constructor does not canonicalise edge order;
  `incidence` follows the stored order. Reported to the maintainer.
- The generated `graph/mod.rs` re-exports wasm registration helpers and is
  not edited; the wasm registry for the two builtins waits for the next
  module-composition regeneration.
- Worst-case isomorphism search is exponential; fine for the sizes in scope.
- Rollback: delete `structure.rs` and its `mod` line.

## Verification commands

`cargo test -p runmat-runtime --lib -- graph_structure_tests` ·
`cargo test -p runmat-runtime --lib -- builtins::graph` ·
`rustfmt --edition 2021 --check <new file>` ·
`cargo clippy -p runmat-runtime --lib --tests --message-format short` (filtered to the new file)
