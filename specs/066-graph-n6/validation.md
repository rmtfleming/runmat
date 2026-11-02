# Validation Report — 066-graph-n6 (specs 0.2.0) — 2026-11-02

Environment: Linux x86_64; `HDF5_DIR=~/anaconda3`; test binaries run
through the cargo runner wrapper. The target directory was shared with
other agents working on other features (F67/F68). MATLAB was not invoked.

## Freshness check (T0, T8)

In `../matlab-interface-spec`, after `git fetch -q origin` (rc 0), at start
and finish: `HEAD` = `origin/main` = `20825f8`, one commit after the pin
`e364206` (a COBRA dependency CSV, not touching these specs);
`git log e364206..origin/main -- specifications/{subgraph,findnode}
experiments/{subgraph,findnode}` empty; `git status --short` for those
directories empty (no drafts); `diff -rq` of the imported copies against
the source tree reports no difference. The imported 0.2.0 copies are
current.

## Test-first evidence (T2–T4)

`subgraph.rs` and `findnode.rs` were first written with both builtins
registered as stubs (after gathering their arguments each returned
`RunMat:<fn>:Internal` "not yet implemented") and the complete test
modules. `cargo test -p runmat-runtime --lib -- graph_n6` →
**2 passed, 24 failed, 1 ignored** (the two passes are the set-up checks
`observed_subgraph_source_graph_setup` and `observed_findnode_node_names`,
which do not call the new builtins; ignored: the `deviation_*` test).

## Commands and results (T7)

- First run after the implementation: 25 passed, 1 failed, 1 ignored. The
  failure was a test-authoring error in `unresolved_choice_findnode_query_types`:
  a `StringArray` payload is column-major, so `["a","c","b","d"]` with
  shape 2×2 is `["a" "b"; "c" "d"]`, whose column-major reading is
  `[1;3;2;4]` (as for the observed 2×2 cell). The expectation was
  corrected; the implementation was not changed.
- `cargo test -p runmat-runtime --lib -- graph_n6`: **26 passed, 0 failed,
  1 ignored**.
- `cargo test -p runmat-runtime --lib -- --ignored graph_n6`:
  `deviation_subgraph_names_end_nodes_cell` fails as documented (named
  `EndNodes` is a string array, not a cell; D-066-1).
- `cargo test -p runmat-runtime --lib -- builtins::graph`: **168 passed,
  0 failed, 7 ignored** (ignored: D-036-1, D-043-1, D-059-1, D-059-2,
  D-062-1 ×2, D-066-1). No regressions.
- `rustfmt --edition 2021 --check` on `subgraph.rs`, `findnode.rs`,
  `mod.rs`: clean.
- `cargo clippy -p runmat-runtime --lib --tests --message-format short`:
  see "Clippy" below.

## Identifier change re-run

After the identifier change (code, tests and documents updated; no
behaviour other than the identifier strings changed):

- `rustfmt --edition 2021 --check` on `subgraph.rs`, `findnode.rs`,
  `mod.rs`: clean.
- `cargo test -p runmat-runtime --lib -- graph_n6`: **26 passed, 0 failed,
  1 ignored**.
- `cargo test -p runmat-runtime --lib -- builtins::graph`: **168 passed,
  0 failed, 7 ignored**.

The command outputs earlier in this report predate the change; they are
unchanged by it apart from the identifier strings asserted.

## Clippy

`cargo clippy -p runmat-runtime --lib --tests --message-format short`,
run twice: the lib target finished with no findings. The lib-test target
reported exactly one finding, in another agent's in-progress file
(`math/optim/lsqnonneg.rs`, feature 067, `iter().any()` →
`contains()`), which stops the build. Clippy lints the whole crate in one
pass, so the same run reported **no finding in `subgraph.rs`,
`findnode.rs` or `mod.rs`** (lib or tests). That file was not edited here.

## Traceability (claim → FR → test → result) (T8)

| Claim / basis | FR | Test(s) | Result |
|---|---|---|---|
| subgraph.signature-primary, output-class | FR-066-01 | `observed_subgraph_logical_mask`, `observed_subgraph_class_and_undirected` | PASS |
| subgraph.node-order | FR-066-02 | `observed_subgraph_logical_mask`, `_logical_column`, `_index_column`, `_index_unsorted`, `_names` | PASS |
| subgraph.node-table | FR-066-03 | `observed_subgraph_source_graph_setup` (D-066-2 set-up), `_logical_mask`, `_index_unsorted`, `_names` | PASS |
| subgraph.induced-edges | FR-066-04 | `observed_subgraph_logical_mask`, `_logical_column`, `_index_column`, `_index_unsorted`, `_weighted` | PASS |
| subgraph.edge-order | FR-066-05 | `observed_subgraph_edge_order`, `_logical_column`, `_index_unsorted`, `_multigraph` | PASS |
| subgraph.multigraph, isolated | FR-066-06 | `observed_subgraph_multigraph`, `observed_subgraph_isolated_node` | PASS |
| subgraph.empty | FR-066-07 | `observed_subgraph_empty` (`all-false`, `empty-index`) | PASS |
| subgraph.names | FR-066-08 | `observed_subgraph_names` (text); cell class: `deviation_subgraph_names_end_nodes_cell` | PASS / ignored (D-066-1) |
| subgraph.error-repeat (`MATLAB:graphfun:subgraph:InvalidInd` → `RunMat:subgraph:InvalidInd`), error-range (`MATLAB:graphfun:digraph:InvalidNodeID` → `RunMat:subgraph:InvalidNodeID`), error-unknown-name (`MATLAB:graphfun:digraph:UnknownNodeName` → `RunMat:subgraph:UnknownNodeName`) | FR-066-09 | `rejects_subgraph_errors` | PASS |
| subgraph.error-one-arg (`MATLAB:minrhs` → `RunMat:minrhs`) | FR-066-10 | `rejects_subgraph_errors` | PASS |
| findnode.signature-primary, cobra-form, output-class | FR-066-11, FR-066-12 | `observed_findnode_node_names` (set-up), `observed_findnode_cobra_cell_row`, all `observed_findnode_*` (class checked via `class`) | PASS |
| findnode.names | FR-066-12 | `observed_findnode_names`, `observed_findnode_undirected` | PASS |
| findnode.not-found | FR-066-13 | `observed_findnode_not_found` | PASS |
| findnode.numeric | FR-066-14 | `observed_findnode_numeric` | PASS |
| findnode.error-unnamed-char (`MATLAB:graphfun:findnode:NoNames` → `RunMat:findnode:NoNames`), error-logical (`…:findnode:ArgType` → `RunMat:findnode:ArgType`), error-numeric-zero (`…:findnode:PosInt` → `RunMat:findnode:PosInt`), error-one-arg (`MATLAB:minrhs` → `RunMat:minrhs`) | FR-066-15..17 | `rejects_findnode_errors` | PASS |
| U-066-1, U-066-2, U-066-3 | FR-066-18 | `unresolved_choice_subgraph_inputs`, `unresolved_choice_findnode_inputs` (direct calls under `push_output_count`) | PASS |
| subgraph.q-undirected-ids, U-066-4; U-066-6 | FR-066-18 | `unresolved_choice_subgraph_node_validation` | PASS |
| subgraph.q-tie-order, U-066-5 | FR-066-18 | `unresolved_choice_subgraph_tie_order` | PASS |
| U-066-7 | FR-066-18 | `unresolved_choice_subgraph_index_and_result` | PASS |
| U-066-8 | FR-066-18 | `unresolved_choice_findnode_inputs` | PASS |
| findnode.q-query-types, U-066-9 | FR-066-18 | `unresolved_choice_findnode_query_types` | PASS |
| *.q-warning-coverage | FR-066-19 | no warning path exists | not tested (warnings) |

Every FR maps to at least one test; every test cites a claim or a
documented choice. All 45 observed cases (20 + 25) are exercised.

## Unresolved-question dispositions

| id | disposition |
|---|---|
| subgraph.q-warning-coverage, findnode.q-warning-coverage | no warnings emitted |
| subgraph.q-tie-order | U-066-5: stable (stored order) |
| subgraph.q-undirected-ids | U-066-4: same identifiers for graph inputs |
| findnode.q-query-types | U-066-9 |

## Deviations

- **D-066-1** (as D-062-2): named `Nodes.Name` / `Edges.EndNodes` are
  string arrays in RunMat (observed cell); case `names` compared as text;
  ignored test kept. `findnode`'s COBRA form answers the same for the
  cell row and RunMat's string row.
- **D-066-2** (as D-059-3/D-062-2): no node-table or edge-table
  constructor; `Met`/`Atom`/`rxns` attached by rebuilding the tables.
- D-036-1/D-043-1/D-062-1 (edges stored in input order) does not affect any
  observed value here, since `subgraph` sorts its output. D-036-2 is not
  needed by any case.

## Notes

- Error identifiers: first implemented with the full observed identifier
  under `RunMat:` (e.g. `RunMat:graphfun:digraph:InvalidNodeID`); changed
  on maintainer decision (2026-11-02) to the graph-family convention of
  features 036–062, `RunMat:<fn>:<last component>` (mapping in `spec.md`
  and the traceability table); `RunMat:minrhs` unchanged; graph inputs
  use the same identifiers (U-066-4). Re-verification: see
  "Identifier change re-run" below.
- `mod findnode;` and `mod subgraph;` were added to the hand-written
  `graph/domain/mod.rs`. No generated parent `mod.rs` was edited; the
  generated `graph/mod.rs` wasm re-exports were not edited (as F36–F62).
- No visibility change was needed; `paths.rs`, `reordernodes.rs` and
  upstream graph code are unchanged.
- Integer-capability metadata was not declared.
- No MathWorks material was consulted; MATLAB was not invoked.
  Independent legal review MAY still be required.
