# Validation Report — 042-sparse-orderings (`amd`, `etree`, `symbfact` 0.2.0) — 2026-10-30

Environment: Linux x86_64; `HDF5_DIR=~/anaconda3`; test binaries run through
a cargo runner wrapper that puts anaconda's `lib/` on `LD_LIBRARY_PATH`. The
target directory is shared with three other agents. One test-first run
waited on the build lock, and one was blocked by a compile error in another
agent's file (`introspection/isprop.rs`). I did not edit that file; I waited
and retried. MATLAB was not invoked.

## Freshness

- Start of work: in `../matlab-interface-spec` (read-only),
  `git fetch -q --all`; `git diff --name-only c65e238 HEAD -- specifications/{amd,etree,symbfact,balance}`
  and the same against `origin/main` were empty; `git status --short` on
  those directories was clean; HEAD = origin/main = `8f72edb`.
- End of work: the same commands (fetch exit 0) were again empty and clean;
  HEAD = origin/main = `8f72edb119511f95884fb63f550e6c88682c327b`. The import
  is current.
- All 25 cited claim ids and 7 question ids were checked against the
  imported `provenance.yaml` / `interface.yaml` with a script; all present.

## Test-first evidence (T4)

The four files contained only their `#[cfg(test)]` modules (and
`symbolic.rs` only its test-support module) and were registered so the tests
compiled.

`cargo test -p runmat-runtime --lib -- amd_tests etree_tests symbfact_tests balance_tests`
→ `test result: FAILED. 0 passed; 73 failed` (14 `amd`, 22 `etree`, 15
`symbfact`, and 22 `balance` from feature 047, which ran in the same
command). Every failure was the missing builtin:

- panics on `Undefined function: amd|etree|symbfact|balance`;
- panics on `UndefinedFunction: <name> is not registered` (the direct-call
  helper);
- or assertions whose left side was the identifier `RunMat:UndefinedFunction`.

Before the run, two test-side corrections were made, both before any
implementation existed:

- A private-field access in the 047 test helper was replaced by
  `ComplexTensor::materialize_f64()`.
- The recorded `amd` ordering for the arrow matrix (a non-normative
  documented-choice value) was corrected from `[2 3 4 1]` to `[2 3 1 4]` after
  re-tracing the tie-breaking rule (ties go to the smallest index).

## Commands and results (T9)

- `cargo test -p runmat-runtime --lib -- amd_tests etree_tests symbfact_tests balance_tests`
  → `test result: ok. 73 passed; 0 failed` (51 for this feature). This was
  the first run after implementation.
- `cargo test -p runmat-runtime --lib -- math::sparse factor::svd` →
  `test result: ok. 129 passed; 0 failed` (includes `colamd`, `dissect`,
  `spalloc`, `svd`, `svds` and the new modules).
- `cargo test -p runmat-runtime --lib -- builtin::registry:: compatibility::`
  → 22 passed, 7 failed. The 7 are the known pre-existing failures: 6
  `compatibility::tests::*` integer-metadata tests and
  `builtin::registry::tests::canonical_catalog_and_runtime_bindings_join_one_to_one`.
  None of the failure output names `amd`, `etree`, `symbfact` or `balance`.
- `rustfmt --edition 2021 --check` on the four files → clean.
- Clippy: `cargo clippy -p runmat-runtime --lib --tests --message-format short` -> no diagnostics in this feature's files. The first runs could not reach them: another agent's `introspection/isprop.rs` failed to lint. Once they did, clippy reported 13 `clone` -> `std::slice::from_ref` lints (`cloned_ref_to_slice_refs`) in test code (11 in `balance.rs`, 1 each in `amd.rs` and `etree.rs`). I fixed them and the rerun showed none. The overall clippy exit is still 101 because of 3 errors in another agent's `graph/domain/editing.rs`, which I did not touch. After the fix, the feature tests (73 passed) and `math::sparse factor::svd` (129 passed) were rerun green.
- No release string appears in the new source files (grep).

## Traceability (claim → FR → test → result) (T10)

| Claim / basis | FR | Test(s) | Result |
|---|---|---|---|
| amd.signature-primary, amd.output-class, amd.permutation (perm) | FR-042-01/02 | `amd_tests::observed_perm` | PASS |
| amd.permutation (arrow) | FR-042-02 | `amd_tests::observed_arrow` | PASS |
| amd.dense-input | FR-042-03 | `amd_tests::observed_dense_input_matches_sparse` | PASS |
| amd.output-class (class) | FR-042-01 | `amd_tests::observed_class` | PASS |
| amd.output-class, amd.permutation (diag) | FR-042-01/02 | `amd_tests::observed_diag` | PASS |
| amd.trivial-sizes | FR-042-04 | `amd_tests::observed_empty`, `amd_tests::observed_scalar` | PASS |
| amd.error-nonsquare | FR-042-05 | `amd_tests::observed_nonsquare_errors` | PASS |
| documented choice (amd.q-ordering; amd.specific-orderings non-normative) | FR-042-15 | `amd_tests::documented_choice_orderings_on_observed_inputs`, `…_larger_patterns_dense_equals_sparse`, `…_arrow_dense_node_not_first` | PASS |
| unresolved amd.q-more-args | FR-042-16 | `amd_tests::unresolved_more_args_unsupported` | PASS |
| documented choice: input classes | FR-042-20 | `amd_tests::documented_choice_input_classes` | PASS |
| documented choice: outputs | FR-042-21 | `amd_tests::documented_choice_too_many_outputs` | PASS |
| etree.signature-primary, etree.output-class, etree.parent-vector | FR-042-06 | `etree_tests::observed_tridiagonal`, `observed_arrow`, `observed_diag`, `observed_pentadiag` | PASS |
| etree.postorder | FR-042-09 | `etree_tests::observed_postorder`, `observed_postorder_arrow` | PASS |
| etree.postorder, etree.branching | FR-042-06/09 | `etree_tests::observed_postorder_branching` | PASS |
| etree.col | FR-042-08 | `etree_tests::observed_col` | PASS |
| etree.sym-default | FR-042-06 | `etree_tests::observed_sym` | PASS |
| etree.dense-input | FR-042-07 | `etree_tests::observed_dense_input` | PASS |
| etree.upper-triangle | FR-042-07 | `etree_tests::observed_upper_only`, `observed_lower_only` | PASS |
| etree.empty | FR-042-10 | `etree_tests::observed_empty` | PASS |
| etree.error-nonsquare | FR-042-10 | `etree_tests::observed_nonsquare_errors` | PASS |
| documented choice: random patterns vs Boolean-elimination oracle | FR-042-06/07/09 | `etree_tests::documented_choice_random_patterns_match_oracle` | PASS |
| documented choice: 'col' vs A'*A oracle; empty shapes | FR-042-08/10 | `etree_tests::documented_choice_col_matches_ata_oracle`, `…_col_empty_shapes` | PASS |
| unresolved etree.q-postorder-general | FR-042-17 | `etree_tests::documented_choice_postorder_sibling_order` | PASS |
| documented choice: option text, other options | FR-042-18 | `etree_tests::documented_choice_option_text_forms`, `…_other_options_unsupported` | PASS |
| documented choice: inputs, outputs | FR-042-20/21 | `etree_tests::documented_choice_input_classes`, `…_too_many_outputs` | PASS |
| symbfact.signature-primary, symbfact.output-class, symbfact.counts | FR-042-11 | `symbfact_tests::observed_tridiagonal`, `observed_arrow`, `observed_arrow_reversed`, `observed_diag`, `observed_class` | PASS |
| symbfact.outputs | FR-042-12 | `symbfact_tests::observed_outputs` | PASS |
| symbfact.col | FR-042-14 | `symbfact_tests::observed_col` | PASS |
| symbfact.dense-input | FR-042-13 | `symbfact_tests::observed_dense_input` | PASS |
| symbfact.error-nonsquare | FR-042-14 | `symbfact_tests::observed_nonsquare_errors` | PASS |
| documented choice: all outputs vs oracle; parent/post equal etree | FR-042-11/12/13/19 | `symbfact_tests::documented_choice_random_patterns_match_oracle` | PASS |
| documented choice: 'col' vs A'*A oracle | FR-042-14 | `symbfact_tests::documented_choice_col_matches_ata_oracle` | PASS |
| documented choice: h, empty | FR-042-19 | `symbfact_tests::documented_choice_height_and_empty` | PASS |
| documented choice: outputs, option text, inputs | FR-042-18/20/21 | `symbfact_tests::documented_choice_output_counts`, `…_option_text_and_inputs` | PASS |
| unresolved symbfact.q-more-options | FR-042-18 | `symbfact_tests::unresolved_other_options_unsupported` | PASS |
| unresolved *.q-warning-coverage | FR-042-22 | every `observed_*` call runs under `assert_no_warnings` | PASS |

All 31 observed cases (8 + 14 + 9) are covered. Every FR has at least one
test, and there are no orphan tests.

## Recorded non-normative values

RunMat's `amd` orderings are `[1 2 3 4 5]` (5×5 corner matrix), `[2 3 1 4]`
(arrow), `[1 2 3]` (diagonal) and `[1 2 3 4]` (tridiagonal). The reference
values in `amd.specific-orderings` differ for the first two. That is
permitted: the claim is `normative: false`, and the maintainer decision makes
the exact ordering non-normative.

## Deviations and open items

- **Too-many-outputs errors** are tested by calling the registered
  implementation directly under `push_output_count`, because the dispatcher
  re-runs a failed builtin without the output count (upstream quirk, also
  recorded for 039-svds). A real `[a,b,c] = etree(...)` from the VM was not
  verified end-to-end.
- **Error identifiers** use the RunMat namespace (`RunMat:square`,
  `RunMat:etree:InvalidInput`, `RunMat:symbfact:InvalidInput`), following
  the established rule.
- **`amd` is exact minimum degree**, not approximate minimum degree. The
  explicit clique update costs O(n · fill) and may be slow on large dense
  patterns. The ordering is non-normative, so it can be replaced later
  without changing any normative test.
- **`symbfact` uses the upper triangle only** (documented choice, consistent
  with `etree`). `'sym'` is rejected for `symbfact`
  (`symbfact.q-more-options`).
- **Not reused**: `sparse::matrix_pattern`. Its dense path re-materialises
  the data for every element (quadratic cost) and it rejects complex input.
  `symbolic.rs` has its own reader. `matrix_pattern` was left unchanged
  (not this feature's file).
- The WASM builtin registry was not regenerated.
