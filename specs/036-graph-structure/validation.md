# Validation Report — 036-graph-structure (spec 0.3.0) — 2026-10-29

Environment: Linux x86_64; `HDF5_DIR=~/anaconda3`; test binaries run
through the cargo runner wrapper (anaconda `lib/` on `LD_LIBRARY_PATH` for
the test process only). The target directory was shared with three other
agents working on other features. MATLAB was not invoked.

## Freshness check (T0)

In `../matlab-interface-spec`, after `git fetch -q --all` (rc 0):
`HEAD` = `origin/main` = `8614c5b396851cdac1d1eb58f1cf5f7fa7f8909e`;
`git diff --name-only 8614c5b HEAD -- specifications/{incidence,isisomorphic}`
and the same against `origin/main` were empty; `git status --short` on both
directories was clean. The imported 0.3.0 copies are current.

## Test-first evidence (T4)

History: a 0.2.0-era test run (before any implementation) gave **0 passed,
15 failed**, all `RunMat:UndefinedFunction`. Work then stopped for the 0.3.0
re-specification.

0.3.0: the tests were rewritten to the new observations and run against the
earlier, unchanged implementation (which compiled):
`cargo test -p runmat-runtime --lib -- graph_structure_tests` →
**10 passed, 7 failed, 1 ignored**. The failures were the 0.3.0 changes:
`observed_incidence_digraph_matches_export` and
`observed_incidence_columns_follow_stored_edge_order` (digraphs were
rejected), `rejects_incidence_self_loops` (old id `UnsupportedOption`),
`rejects_incidence_matrix_input` / `rejects_isisomorphic_matrix_inputs`
(old id `InvalidInput`), `rejects_isisomorphic_graph_with_digraph` (old id
`UnsupportedOption`), and
`unresolved_choice_isisomorphic_options_and_second_output` (a second output
was not rejected: returned `Bool(true)`).

## Commands and results (T7)

- `cargo test -p runmat-runtime --lib -- graph_structure_tests` after the
  implementation: **17 passed, 0 failed, 1 ignored**.
  - `observed_incidence_graph_matches_export`
  - `observed_incidence_digraph_matches_export`
  - `observed_incidence_multigraph_matches_export`
  - `observed_incidence_weights_ignored`
  - `observed_incidence_shapes_match_export`
  - `observed_incidence_columns_follow_stored_edge_order`
  - `rejects_incidence_self_loops`
  - `rejects_incidence_matrix_input`
  - `unresolved_choice_incidence_named_nodes`
  - `unresolved_choice_incidence_extra_arguments_and_arity`
  - `observed_isisomorphic_true_cases_match_export`
  - `observed_isisomorphic_false_cases_match_export`
  - `rejects_isisomorphic_graph_with_digraph`
  - `rejects_isisomorphic_matrix_inputs`
  - `unresolved_choice_isisomorphic_options_and_second_output`
  - `unresolved_choice_isisomorphic_arity`
  - `unresolved_choice_isisomorphic_search_cases`
  - ignored: `deviation_incidence_observed_values_for_unsorted_input`
    (D-036-1, kept as an executable record of the gap)
- `cargo test -p runmat-runtime --lib -- builtins::graph`: **32 passed,
  0 failed, 1 ignored** (15 upstream graph tests plus the 17 above; the
  ignored one is D-036-1). No regressions.
- `rustfmt --edition 2021 --check .../graph/domain/structure.rs`: clean.
- `cargo clippy -p runmat-runtime --lib --tests --message-format short`:
  **no findings in `structure.rs` or `graph/domain/mod.rs`**. The run
  stopped with 7 errors, all in other files: `validateattributes.rs`,
  `exportgraphics.rs`, `spy.rs`, `findstr.rs` (other agents' work in
  progress), and `table/builtins/joins.rs`, `variable_editing.rs` (earlier
  batch C1). Because the lib target failed, clippy did not reach the
  `--tests` target, so the test module was not linted.

## Traceability (claim → FR → test → result) (T8)

| Claim / basis | FR | Test(s) | Result |
|---|---|---|---|
| incidence.signature-primary, incidence.output-class, incidence.shape | FR-036-01 | `observed_incidence_graph_matches_export`, `observed_incidence_digraph_matches_export`, `observed_incidence_shapes_match_export` | PASS |
| incidence.signs | FR-036-02 | `observed_incidence_graph_matches_export`, `observed_incidence_digraph_matches_export` (incl. `directed-reverse`), `observed_incidence_columns_follow_stored_edge_order` | PASS |
| incidence.edge-order | FR-036-02 | `observed_incidence_columns_follow_stored_edge_order` (relation to RunMat's stored `G.Edges`); absolute values: `deviation_incidence_observed_values_for_unsorted_input` | PASS / ignored (D-036-1) |
| incidence.multigraph | FR-036-02 | `observed_incidence_multigraph_matches_export` | PASS |
| incidence.weights-ignored | FR-036-02 | `observed_incidence_weights_ignored` | PASS |
| incidence.no-warning | FR-036-02 | by construction: the implementation has no warning path (no runtime warning-capture test) | not tested |
| incidence.error-self-loop-graph | FR-036-03 | `rejects_incidence_self_loops` | PASS |
| incidence.error-matrix | FR-036-04 | `rejects_incidence_matrix_input` | PASS |
| isisomorphic.signature-primary, isisomorphic.output-class, isisomorphic.true-cases, isisomorphic.weights-ignored | FR-036-05 | `observed_isisomorphic_true_cases_match_export` | PASS |
| isisomorphic.false-cases, isisomorphic.direction | FR-036-05 | `observed_isisomorphic_false_cases_match_export` | PASS |
| isisomorphic.no-warning | FR-036-05 | by construction (as above) | not tested |
| isisomorphic.error-mixed | FR-036-06 | `rejects_isisomorphic_graph_with_digraph` | PASS |
| isisomorphic.error-matrix | FR-036-07 | `rejects_isisomorphic_matrix_inputs` | PASS |
| unresolved: isisomorphic.q-options, isisomorphic.q-second-output | FR-036-08 | `unresolved_choice_isisomorphic_options_and_second_output` | PASS |
| documented choice U-036-1, U-036-6 | FR-036-09 | `unresolved_choice_incidence_extra_arguments_and_arity`, `unresolved_choice_isisomorphic_arity` | PASS |
| documented choice incidence.q-named-nodes | FR-036-10 | `unresolved_choice_incidence_named_nodes` | PASS |
| algorithm (definition of isomorphism) | FR-036-11 | `unresolved_choice_isisomorphic_search_cases` | PASS |

Every FR has at least one test. No orphan tests.

## Deviations and open items

- **D-036-1 (needs a maintainer decision).** Upstream RunMat `graph()` /
  `digraph()` store edges in input order with the endpoints as given. The
  observed constructor stores undirected edges with the smaller node first
  and sorts edges. `incidence` follows RunMat's stored `G.Edges`, so for
  unsorted input (cases `edge-order`, `directed-edge-order`,
  `undirected-reversed-input`) its columns differ from the observed values.
  The fix belongs in the graph constructor, which is outside this feature
  and would also change `findedge`/`G.Edges` behaviour.
- **D-036-2.** The observed `graph(1, 2, 1, n)` (node-count argument) is
  rejected by RunMat's graph constructor; tests use the equivalent
  adjacency-matrix construction.
- **Second-output test path.** Called through
  `call_builtin_async_with_outputs(..., 2)` in the unit-test harness, the
  builtin did not see the requested output count (it returned `Bool(true)`).
  The rejection is therefore tested the way upstream's `treelayout` test
  does it: push the output count and call the builtin function directly.
  Rejecting `[tf, P] = isisomorphic(...)` through the VM is not verified.
- Not verified end-to-end through the MATLAB-syntax VM; the tests call the
  registered builtins through the runtime dispatcher.
- The WASM builtin registry and the generated `graph/mod.rs` wasm
  re-exports were not regenerated (deferred, as in earlier batches).
- Unresolved questions returned to the specification side:
  `incidence.q-warning-coverage`, `incidence.q-named-nodes`,
  `isisomorphic.q-warning-coverage`, `isisomorphic.q-options`,
  `isisomorphic.q-second-output`.
