# Validation Report — 059-graph-n3n4 (specs 0.2.0) — 2026-10-30

Environment: Linux x86_64; `HDF5_DIR=~/anaconda3`; test binaries run
through the cargo runner wrapper. The target directory was shared with
other agents working on other features. MATLAB was not invoked.

## Freshness check (T0, T9)

In `../matlab-interface-spec`, after `git fetch -q origin`:

- Start: `HEAD` = `origin/main` = `c3305a5`; no commit after the pins
  touches `specifications/isomorphism` (since `1156c97`) or
  `specifications/{allpaths,inedges}` (since `fc9f804`); status clean for
  those directories; imported copies byte-identical to the source tree.
- Finish: fetch rc 0; `HEAD` = `origin/main` = `1c91aa1` (later than the
  start, changes none of the three spec directories):
  `git log 1156c97..origin/main -- specifications/isomorphism`,
  `git log fc9f804..origin/main -- specifications/{allpaths,inedges}` and
  the matching `git diff --name-only … HEAD` were all empty;
  `git status --short` for the three `specifications/` and `experiments/`
  directories was empty (no drafts). The imported 0.2.0 copies are current.

## Test-first evidence (T2–T4)

`isomorphism.rs` and `paths.rs` were first written with the three builtins
registered as stubs (each gathered its arguments, then returned
`RunMat:<fn>:Internal` "not yet implemented") and the complete test
modules. `cargo test -p runmat-runtime --lib -- graph_n3n4` →
**0 passed, 34 failed, 2 ignored** (ignored: the two `deviation_*` tests).
Earlier attempts could not compile because of other agents' in-progress
files (`table/builtins/renamevars.rs`, `variable_support.rs`,
`strings/core/as_many_of_pattern.rs`); the run was taken once the shared
build compiled.

## Commands and results (T8)

- `cargo test -p runmat-runtime --lib -- graph_n3n4` after the
  implementation: **34 passed, 0 failed, 2 ignored**.
- `cargo test -p runmat-runtime --lib -- --ignored graph_n3n4`: both
  deviation tests fail as documented —
  `deviation_inedges_observed_order_for_unsorted_input`: `left: (3, 1, [4, 1, 3])`,
  `right: (3, 1, [1, 3, 4])` (D-059-1);
  `deviation_allpaths_observed_edge_paths_for_unsorted_input`:
  `E = [1 2], [4 3]` vs observed `[1 3], [2 4]` (D-059-2).
- `cargo test -p runmat-runtime --lib -- builtins::graph`: **116 passed,
  0 failed, 4 ignored** (ignored: D-036-1, D-043-1, D-059-1, D-059-2). No
  regressions.
- `rustfmt --edition 2021 --check` on both files: clean.
- `cargo clippy -p runmat-runtime --lib --tests --message-format short`:
  the first run reported one finding in this feature
  (`isomorphism.rs`: manual `% 2 != 0` → `is_multiple_of`), fixed; the
  re-run reports **no findings in this feature's files**. The lib target
  still stops on three findings in other agents' in-progress files
  (`plotting/ops/{findall,pcolor,surfl}.rs`), so the `--tests` target could
  not be linted in that run; the feature tests were re-run after the fix
  (34 passed, 2 ignored).

## Traceability (claim → FR → test → result) (T9)

| Claim / basis | FR | Test(s) | Result |
|---|---|---|---|
| isomorphism.signature-primary, output-class | FR-059-01 | `observed_isomorphism_asymmetric_pair_matches_export`, `observed_isomorphism_identical_path_is_valid_permutation`, `observed_isomorphism_digraph_orientation` | PASS |
| isomorphism.orientation | FR-059-02 | same three plus `observed_isomorphism_edge_variables` (both reorder directions checked against the observed true/false values) | PASS |
| isomorphism.not-isomorphic | FR-059-03 | `observed_isomorphism_not_isomorphic_is_0x0` | PASS |
| isomorphism.empty-graphs | FR-059-03 | `observed_isomorphism_empty_graphs_is_1x0` | PASS |
| isomorphism.edgeperm | FR-059-04 | `observed_isomorphism_edgeperm_matches_export` | PASS |
| isomorphism.node-variables | FR-059-05 | `observed_isomorphism_node_variables` (D-059-3 set-up) | PASS |
| isomorphism.edge-variables | FR-059-05 | `observed_isomorphism_edge_variables` | PASS |
| isomorphism.error-mixed | FR-059-06 | `rejects_isomorphism_graph_with_digraph` | PASS |
| isomorphism.error-unknown-nodevar | FR-059-06 | `rejects_isomorphism_unknown_node_variable` | PASS |
| allpaths.signature-primary, allpaths.paths | FR-059-07 | `observed_allpaths_path_and_cycle` (path sets) | PASS |
| allpaths.digraph, allpaths.unreachable | FR-059-08 | `observed_allpaths_digraph_direction`, `observed_allpaths_unreachable` | PASS |
| allpaths.same-node | FR-059-09 | `observed_allpaths_same_node` | PASS |
| allpaths.names | FR-059-10 | `observed_allpaths_names` | PASS |
| allpaths.edgepaths | FR-059-11 | `observed_allpaths_edge_paths` (sorted-storage input), `observed_allpaths_edge_paths_follow_stored_edges` (observed input, relation); observed absolute values: `deviation_allpaths_observed_edge_paths_for_unsorted_input` | PASS / ignored (D-059-2) |
| allpaths.error-node-range, error-unknown-name | FR-059-12 | `rejects_allpaths_invalid_nodes` | PASS |
| inedges.signature-primary, inedges.outputs | FR-059-13 | `observed_inedges_one_and_two_outputs` | PASS |
| inedges.order | FR-059-13 | `observed_inedges_order` (sorted-storage input), `observed_inedges_order_follows_stored_edges` (observed input, relation); observed `nid`: `deviation_inedges_observed_order_for_unsorted_input` | PASS / ignored (D-059-1) |
| inedges.none, inedges.selfloop | FR-059-14 | `observed_inedges_none`, `observed_inedges_self_loop` | PASS |
| inedges.names | FR-059-15 | `observed_inedges_names` | PASS |
| inedges.error-undirected, error-node-range, error-two-nodes | FR-059-16 | `rejects_inedges_errors` | PASS |
| isomorphism.non-graph, allpaths.non-graph (non-normative), U-059-1 | FR-059-17 | `unresolved_choice_isomorphism_inputs`, `unresolved_choice_allpaths_inputs`, `unresolved_choice_inedges_inputs` | PASS |
| U-059-2 | FR-059-17 | same three tests | PASS |
| U-059-3 | FR-059-17 | `unresolved_choice_isomorphism_output_count`, `unresolved_choice_allpaths_inputs`, `unresolved_choice_inedges_inputs` (direct calls under `push_output_count`) | PASS |
| isomorphism.non-unique (non-normative), U-059-4(a) | FR-059-17 | `observed_isomorphism_identical_path_is_valid_permutation`, `unresolved_choice_isomorphism_search_cases` (validity only) | PASS |
| U-059-4(b) | FR-059-17 | `unresolved_choice_isomorphism_weights_ignored_without_option` | PASS |
| U-059-4(c), U-059-5 | FR-059-17 | `unresolved_choice_isomorphism_multigraph_self_loops_and_edgeperm` | PASS |
| U-059-4(d)–(h) | FR-059-17 | `unresolved_choice_isomorphism_options` | PASS |
| allpaths.order (non-normative), U-059-6 | FR-059-17 | `unresolved_choice_allpaths_order_multigraph_and_self_loops`, `unresolved_choice_allpaths_inputs` | PASS |
| U-059-7 | FR-059-17 | `unresolved_choice_inedges_inputs` | PASS |
| *.q-warning-coverage, *.q-more | FR-059-18 | no warning path exists; q-more handled by U-059-2/4/6 tests above | PASS / not tested (warnings) |

Every FR maps to at least one test; every test cites a claim or a
documented choice.

## Unresolved-question dispositions

| id | disposition |
|---|---|
| isomorphism.q-warning-coverage, allpaths.q-warning-coverage, inedges.q-warning-coverage | no warnings emitted |
| isomorphism.q-more | U-059-4(c) multisets; U-059-4(d) several variables |
| allpaths.q-more | U-059-2 (more than three inputs → UnsupportedOption), U-059-6 |

## Deviations

- **D-059-1** (same cause as D-036-1/D-043-1): `inedges` case `order` —
  `nid` follows RunMat's stored (input) edge order (`[4;1;3]` vs observed
  `[1;3;4]`); `eid` matches. Ignored test kept.
- **D-059-2** (same cause): `allpaths` case `edgepaths` — `E` refers to
  RunMat's stored edge order (`{[1 2]; [4 3]}` vs observed
  `{[1 3]; [2 4]}`); node paths match. Ignored test kept.
- **D-059-3**: RunMat's constructors create no custom node variables;
  the `Met` cases attach the variable by rebuilding the object's `Nodes`
  table. `graph(s,t,w,n)` (D-036-2) was not needed by any case here.

## Notes

- `structure.rs` (F36) was not modified: `isomorphism` needs the map and
  node/edge labels, which F36's boolean unlabelled matcher does not
  provide, so a separate labelled matcher was written (no code copied).
- `mod isomorphism;` and `mod paths;` were added to the hand-written
  `graph/domain/mod.rs`. The generated `graph/mod.rs` wasm re-exports were
  not edited (same as features 036/043).
- Integer-capability metadata was not declared.
- No MathWorks material was consulted; MATLAB was not invoked.
  Independent legal review MAY still be required.
