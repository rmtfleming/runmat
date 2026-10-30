# Validation Report — 043-graph-editing (specs 0.2.0) — 2026-10-30

Environment: Linux x86_64; `HDF5_DIR=~/anaconda3`; test binaries run
through the cargo runner wrapper (anaconda `lib/` on `LD_LIBRARY_PATH` for
the test process only). The target directory was shared with other agents
working on other features. MATLAB was not invoked.

## Freshness check (T0, T10)

In `../matlab-interface-spec`, after `git fetch -q --all`:
`git diff --name-only c65e238 HEAD -- specifications/{rmnode,addedge,rmedge,laplacian}`
and the same against `origin/main` were empty; `git status --short` on the
four directories was clean. Checked at the start and again at the end of the
work (final check 2026-10-30: `git fetch -q --all` rc 0; `HEAD` = `origin/main` = `8f72edb`, which is later than the pin but changes none of the four spec directories — both diffs empty, status clean; the imported 0.2.0 copies are current).

## Test-first evidence (T2–T4)

`editing.rs` was first written with the four builtins registered as stubs
(each gathered its arguments, applied the output-count check, then returned
`RunMat:<fn>:Internal` "not yet implemented") and the complete test module.
`cargo test -p runmat-runtime --lib -- graph_editing_tests` →
**1 passed, 37 failed, 1 ignored**. The one pass was
`unresolved_choice_too_many_outputs`, because the stubs already contained
the output-count guard; every behavioural test failed. (The first attempts
could not compile because of other agents' in-progress files —
`introspection/isprop.rs`, `math/linalg/factor/balance.rs` — so the run was
taken once the shared build compiled.)

## Commands and results (T9)

- `cargo test -p runmat-runtime --lib -- graph_editing_tests` after the
  implementation: **38 passed, 0 failed, 1 ignored** (ignored:
  `deviation_rmnode_observed_endpoint_order`, D-043-1).
- `cargo test -p runmat-runtime --lib -- builtins::graph`: **70 passed,
  0 failed, 2 ignored** (15 upstream graph tests, 17 feature-036 tests,
  38 feature-043 tests; ignored: D-036-1 and D-043-1). No regressions.
- `cargo test -p runmat-runtime --lib -- --ignored
  deviation_rmnode_observed_endpoint_order`: fails as documented —
  `left: [["c", "a"]]`, `right: [["a", "c"]]` (RunMat keeps the
  constructor's stored endpoint order `c–a`).
- `rustfmt --edition 2021 --check .../graph/domain/editing.rs`: clean.
- `cargo clippy -p runmat-runtime --lib --tests --message-format short`:
  the first complete run (once other agents' lib-target lint errors in
  `introspection/isprop.rs` had cleared) reported 3 findings, all
  `clippy::cloned_ref_to_slice_refs` in this feature's test helpers
  (`&[g.clone()]`, `&[star.clone()]`). They were fixed with
  `std::slice::from_ref`; the re-run finished with **no findings anywhere
  in the crate** (`Finished dev profile`), lib and tests targets. The
  feature tests and the graph suite were re-run after the fix with the same
  results (graph suite 70 passed, 0 failed, 2 ignored).

## Traceability (claim → FR → test → result) (T10)

| Claim / basis | FR | Test(s) | Result |
|---|---|---|---|
| rmnode.signature-primary, rmnode.removes | FR-043-01 | `observed_rmnode_removes_and_renumbers` | PASS |
| rmnode.isolated | FR-043-01 | `observed_rmnode_isolated` (adjacency form, D-043-2) | PASS |
| rmnode.class | FR-043-02 | `observed_rmnode_removes_and_renumbers`, `observed_rmnode_digraph` | PASS |
| rmnode.named | FR-043-03 | `observed_rmnode_named` (edge as unordered pair); exact endpoint order: `deviation_rmnode_observed_endpoint_order` | PASS / ignored (D-043-1) |
| rmnode.weights | FR-043-04 | `observed_rmnode_weights`, `observed_rmnode_isolated` | PASS |
| rmnode.empty-list, rmnode.all | FR-043-05 | `observed_rmnode_empty_list_and_all` | PASS |
| rmnode.ignored-ids (non-normative), rmnode.q-ignored-ids | FR-043-06 | `unresolved_choice_rmnode_ignores_unknown_ids`; exact `unknown-name` endpoints: `deviation_rmnode_observed_endpoint_order` | PASS / ignored (D-043-1) |
| addedge.signature-primary, addedge.edge-order | FR-043-07 | `observed_addedge_sorted_edge_list`, `observed_addedge_scalar_expansion` | PASS |
| addedge.digraph-order | FR-043-07 | `observed_addedge_digraph_order` | PASS |
| addedge.duplicates | FR-043-08 | `observed_addedge_sorted_edge_list` (case `order`) | PASS |
| addedge.self-loop | FR-043-08 | `observed_addedge_self_loop` | PASS |
| addedge.new-nodes | FR-043-09 | `observed_addedge_new_nodes` | PASS |
| addedge.named | FR-043-09 | `observed_addedge_named` | PASS |
| addedge.weights | FR-043-10 | `observed_addedge_weights` | PASS |
| addedge.scalar-expansion | FR-043-11 | `observed_addedge_scalar_expansion` | PASS |
| addedge.error-weight-unweighted | FR-043-12 | `rejects_addedge_weight_on_unweighted_graph` | PASS |
| addedge.error-length | FR-043-12 | `rejects_addedge_length_mismatch` | PASS |
| rmedge.signature-primary, rmedge.by-endpoints | FR-043-13 | `observed_rmedge_by_endpoints` | PASS |
| rmedge.multigraph | FR-043-13 | `observed_rmedge_multigraph` | PASS |
| rmedge.digraph-direction | FR-043-13 | `observed_rmedge_digraph_direction` | PASS |
| rmedge.absent | FR-043-13 | `observed_rmedge_absent`, `observed_rmedge_digraph_direction` | PASS |
| rmedge.by-index | FR-043-14 | `observed_rmedge_by_index` | PASS |
| rmedge.keeps-nodes | FR-043-15 | `observed_rmedge_by_endpoints` (case `keeps-nodes`) | PASS |
| rmedge.weights | FR-043-15 | `observed_rmedge_weights` | PASS |
| rmedge.error-index-range | FR-043-16 | `rejects_rmedge_edge_index_out_of_range` | PASS |
| rmedge.error-node-range | FR-043-16 | `rejects_rmedge_node_out_of_range` | PASS |
| laplacian.signature-primary, laplacian.output-class, laplacian.values | FR-043-17 | `observed_laplacian_path_matches_export` (exact sparse CSC), `observed_laplacian_triangle_and_isolated` | PASS |
| laplacian.weights-ignored | FR-043-17 | `observed_laplacian_weights_ignored` | PASS |
| laplacian.empty | FR-043-17 | `observed_laplacian_empty` | PASS |
| laplacian.error-self-loop, laplacian.error-multigraph, laplacian.error-digraph | FR-043-18 | `rejects_laplacian_non_simple_and_digraph` | PASS |
| (design) result is a new object, counts consistent | FR-043-19 | every `observed_*` test checks `numnodes`/`numedges` through the public builtins; `observed_rmnode_removes_and_renumbers` checks the input is unchanged | PASS |
| laplacian.numeric-input (non-normative), U-043-1 | FR-043-20 | `unresolved_choice_laplacian_inputs`, `unresolved_choice_rmnode_invalid_inputs`, `unresolved_choice_addedge_arity_and_forms`, `unresolved_choice_rmedge_expansion_and_arity` | PASS |
| U-043-2 | FR-043-20 | same four tests | PASS |
| U-043-3 | FR-043-20 | `unresolved_choice_too_many_outputs`, `unresolved_choice_single_output_direct_call` | PASS |
| U-043-4 | FR-043-20 | `unresolved_choice_rmnode_invalid_inputs`, `unresolved_choice_rmnode_ignores_unknown_ids` | PASS |
| U-043-5 | FR-043-20 | `unresolved_choice_addedge_expansion_and_weights`, `unresolved_choice_addedge_names_and_indices` | PASS |
| U-043-6 | FR-043-20 | `unresolved_choice_rmedge_names_and_invalid_ids`, `unresolved_choice_rmedge_expansion_and_arity` | PASS |
| U-043-7 | FR-043-20 | `observed_rmnode_*`, `observed_rmedge_*` (stored order kept) | PASS |
| U-043-8 | FR-043-20 | `unresolved_choice_laplacian_inputs` | PASS |
| U-043-9 | FR-043-20 | `observed_rmnode_isolated`, `unresolved_choice_addedge_expansion_and_weights` | PASS |
| *.q-warning-coverage, *.q-graph-variables | FR-043-21 | no warning path exists in the implementation; RunMat graphs have no extra variables (not tested) | not tested |

Every FR maps to at least one test; every test cites a claim or a
documented choice (no orphan tests).

## Unresolved-question dispositions

| id | disposition |
|---|---|
| rmnode.q-ignored-ids | documented choice FR-043-06: ignore (settled by observed cases `out-of-range`, `unknown-name`) |
| rmnode/addedge/rmedge/laplacian .q-warning-coverage | no warnings emitted |
| rmnode/addedge/rmedge .q-graph-variables | not applicable to RunMat's graph representation |

## Deviations

- **D-043-1** (same cause as D-036-1): RunMat's constructor keeps input edge
  order and endpoints. Observed endpoint order of rmnode cases `named` and
  `unknown-name` is not reproduced; counts, names and edge sets are. Kept as
  the ignored test `deviation_rmnode_observed_endpoint_order` (fails today
  with `[["c","a"]]` vs `[["a","c"]]`). Upstream fix belongs to the graph
  constructor; not attempted here.
- **D-043-2** (same cause as D-036-2): the observed `graph(1, 2, 1, 3)` is
  not accepted by RunMat's constructor; tests use the adjacency form. Unit
  weights are stored without a `Weight` variable (U-043-9), where the
  observation reports `Weight = 1`.

## Notes

- Integer-capability metadata was not declared (as instructed).
- The generated `graph/mod.rs` wasm re-exports were not edited; wasm
  registration of the four builtins waits for the next module-composition
  regeneration (same as feature 036).
- No MathWorks material was consulted; MATLAB was not invoked.
  Independent legal review MAY still be required.
