# Validation Report — 062-graph-n5 (specs 0.2.0) — 2026-10-31

Environment: Linux x86_64; `HDF5_DIR=~/anaconda3`; test binaries run
through the cargo runner wrapper. The target directory was shared with
other agents working on other features. MATLAB was not invoked.

## Freshness check (T0, T8)

In `../matlab-interface-spec`, after `git fetch -q origin` (rc 0), at start
and finish: `HEAD` = `origin/main` = `1c91aa1` (the pin);
`git log 1c91aa1..origin/main -- specifications/{outedges,reordernodes}
experiments/{outedges,reordernodes}` empty; `git status --short` for those
directories empty (no drafts); `diff -rq` of the imported copies against
the source tree reports no difference. The imported 0.2.0 copies are
current.

## Test-first evidence (T2–T4)

`outedges.rs` and `reordernodes.rs` were first written with both builtins
registered as stubs (after gathering their arguments each returned
`RunMat:<fn>:Internal` "not yet implemented") and the complete test
modules (built together with feature 063's stub).
`cargo test -p runmat-runtime --lib -- graph_n5 rat_tests` →
**0 passed, 38 failed, 2 ignored** (25 of the failures are this feature's;
ignored: the two `deviation_*` tests).

## Commands and results (T7)

- `cargo test -p runmat-runtime --lib -- graph_n5 rat_tests` after the
  implementation: **38 passed, 0 failed, 2 ignored** (this feature: 25
  passed, 2 ignored).
- `cargo test -p runmat-runtime --lib -- --ignored graph_n5`: both
  deviation tests fail as documented —
  `deviation_outedges_observed_order_for_unsorted_input`:
  `left: (2, 1, [3.0, 2.0])`, `right: (2, 1, [2.0, 3.0])` (D-062-1);
  `deviation_reordernodes_idx_for_adjacency_input`: `left: [1.0, 2.0, 3.0]`,
  `right: [3.0, 1.0, 2.0]` (D-062-1).
- `cargo test -p runmat-runtime --lib -- builtins::graph`: **142 passed,
  0 failed, 6 ignored** (ignored: D-036-1, D-043-1, D-059-1, D-059-2,
  D-062-1 ×2). No regressions.
- `rustfmt --edition 2021 --check` on `outedges.rs`, `reordernodes.rs`,
  `paths.rs`: clean.
- `cargo clippy -p runmat-runtime --lib --tests --message-format short`:
  see "Clippy" below.

## Clippy

`cargo clippy -p runmat-runtime --lib --tests --message-format short`: the
first run stopped on a finding in another agent's in-progress file
(`plotting/ops/rectangle.rs`, lib target; no lib finding in this feature).
The second run reported one test-code finding here
(`reordernodes.rs`: `&[g.clone()]` → `std::slice::from_ref(&g)`) and one in
feature 063; both were fixed. The final run **finished with no findings**
(lib and tests), and the feature tests were re-run (38 passed, 2 ignored,
together with 063).

## Traceability (claim → FR → test → result) (T8)

| Claim / basis | FR | Test(s) | Result |
|---|---|---|---|
| outedges.signature-primary, outedges.order | FR-062-01, FR-062-02 | `observed_outedges_two_outputs` (sorted-storage input), `observed_outedges_order_follows_stored_edges` (observed input, relation), `observed_outedges_one_output`; observed `nid` on the observed input: `deviation_outedges_observed_order_for_unsorted_input` | PASS / ignored (D-062-1) |
| outedges.empty | FR-062-03 | `observed_outedges_no_out_edges`, `observed_outedges_isolated_node` (D-036-2 set-up) | PASS |
| outedges.names | FR-062-04 | `observed_outedges_names_char`, `observed_outedges_names_string`, `observed_outedges_names_index` | PASS |
| outedges.multi | FR-062-05 | `observed_outedges_multigraph`, `observed_outedges_self_loop` | PASS |
| outedges.undirected | FR-062-06 | `observed_outedges_undirected` | PASS |
| outedges.error-range, error-vector, error-unknown-name | FR-062-07 | `rejects_outedges_errors` | PASS |
| reordernodes.signature-primary, reordernodes.nodes | FR-062-08, FR-062-09 | `observed_reordernodes_met_graph_nodes` (D-062-2 set-up) | PASS |
| reordernodes.edges | FR-062-10, FR-062-11 | `observed_reordernodes_met_graph_nodes`, `observed_reordernodes_idx`, `observed_reordernodes_idx_relation_for_adjacency_input`; observed `idx` on the adjacency-built input: `deviation_reordernodes_idx_for_adjacency_input` | PASS / ignored (D-062-1) |
| reordernodes.weights | FR-062-10, FR-062-11 | `observed_reordernodes_weights` | PASS |
| reordernodes.undirected | FR-062-10 | `observed_reordernodes_undirected` | PASS |
| reordernodes.names | FR-062-09, FR-062-12 | `observed_reordernodes_names` | PASS |
| reordernodes.identity | FR-062-08 | `observed_reordernodes_identity_and_class` | PASS |
| reordernodes.error-repeat, error-range | FR-062-13 | `rejects_reordernodes_errors` | PASS |
| U-062-1, U-062-2, U-062-3 | FR-062-14 | `unresolved_choice_outedges_inputs`, `unresolved_choice_reordernodes_inputs` (direct calls under `push_output_count`) | PASS |
| outedges.q-undirected-ids, U-062-4 | FR-062-14 | `unresolved_choice_outedges_node_validation` | PASS |
| U-062-5 | FR-062-14 | `unresolved_choice_outedges_self_loop_and_empty_names` | PASS |
| U-062-6 | FR-062-14 | `unresolved_choice_reordernodes_inputs` | PASS |
| U-062-7 | FR-062-14 | `unresolved_choice_reordernodes_order_validation` | PASS |
| U-062-8, U-062-9 | FR-062-14 | `unresolved_choice_reordernodes_ties_index_and_empty` | PASS |
| *.q-warning-coverage | FR-062-15 | no warning path exists | not tested (warnings) |

Every FR maps to at least one test; every test cites a claim or a
documented choice.

## Unresolved-question dispositions

| id | disposition |
|---|---|
| outedges.q-warning-coverage, reordernodes.q-warning-coverage | no warnings emitted |
| outedges.q-undirected-ids | U-062-4: same identifiers for graph inputs |

## Deviations

- **D-062-1** (same cause as D-036-1/D-043-1/D-059-1): `outedges` case
  `two-out` — `nid` follows RunMat's stored (input) edge order (`[3;2]` vs
  observed `[2;3]`); `eid` matches. `reordernodes` case `idx-out` on the
  adjacency-built graph — `idx = [1;2;3]` (observed `[3;1;2]`) because
  `digraph(A)` stores edges column by column; `H` matches. Ignored tests
  kept; observed values reproduced on sorted-storage graphs.
- **D-062-2** (as D-059-3): node variables `Met`/`Atom` attached by
  rebuilding the `Nodes` table; RunMat's constructor stores `Name` and
  named `EndNodes` as string arrays (observed cell) — text compared.
- **D-036-2**: `digraph(s, t, [], 4)` not accepted; `isolated` uses the
  adjacency form.

## Notes

- Nine F59 helpers in `paths.rs` were widened from private to
  `pub(super)` (`query_error`, `graph_input`, `check_output_count`,
  `outputs`, `NodeArg`, `resolve_node`, `double`, `cell`, `name_value`);
  no behaviour change (F59 tests pass within `builtins::graph`).
- `mod outedges;` and `mod reordernodes;` were added to the hand-written
  `graph/domain/mod.rs`. No generated parent `mod.rs` was edited; the
  generated `graph/mod.rs` wasm re-exports were not edited (as F36/F43/F59).
- `structure.rs`, `editing.rs`, `addnode.rs`, `isomorphism.rs` unchanged.
- Integer-capability metadata was not declared.
- No MathWorks material was consulted; MATLAB was not invoked.
  Independent legal review MAY still be required.
