# Validation Report — 047-balance (`balance` 0.2.0) — 2026-10-30

Environment: Linux x86_64; `HDF5_DIR=~/anaconda3`; test binaries run through
a cargo runner wrapper that puts anaconda's `lib/` on `LD_LIBRARY_PATH`. The
target directory is shared with three other agents. One run was blocked by a
compile error in another agent's file (`introspection/isprop.rs`); I waited
and retried without editing it. MATLAB was not invoked.

## Freshness

- Start of work: in `../matlab-interface-spec` (read-only),
  `git fetch -q --all`; `git diff --name-only c65e238 HEAD -- specifications/{amd,etree,symbfact,balance}`
  and the same against `origin/main` were empty; `git status --short` was
  clean; HEAD = origin/main = `8f72edb`.
- End of work: unchanged (empty diffs, clean status, HEAD = origin/main =
  `8f72edb119511f95884fb63f550e6c88682c327b`).
- All 10 cited claim ids and 4 question ids were checked against the import
  with a script; all present.

## Test-first evidence (T4)

`balance.rs` contained only `mod balance_tests`, and `pub mod balance;` was
added so it compiled.

`cargo test -p runmat-runtime --lib -- amd_tests etree_tests symbfact_tests balance_tests`
→ `test result: FAILED. 0 passed; 73 failed`, of which 22 were `balance_tests`.
Each `balance` failure was one of:

- a panic on `Undefined function: balance`;
- a panic on `UndefinedFunction: balance is not registered`;
- an assertion with left side `RunMat:UndefinedFunction`.

Before that run, one test-helper compile error was fixed: a private
`ComplexTensor` field access was replaced by `materialize_f64()`.

## Commands and results (T7)

- `cargo test -p runmat-runtime --lib -- amd_tests etree_tests symbfact_tests balance_tests`
  → `test result: ok. 73 passed; 0 failed` (22 `balance_tests`). This was
  the first run after implementation.
- `cargo test -p runmat-runtime --lib -- math::sparse factor::svd` →
  `ok. 129 passed; 0 failed`.
- `cargo test -p runmat-runtime --lib -- builtin::registry:: compatibility::`
  → 22 passed, 7 failed. These are the known pre-existing failures: 6
  `compatibility::tests::*` plus `canonical_catalog_and_runtime_bindings_join_one_to_one`.
  None of the failure output mentions `balance`.
- `rustfmt --edition 2021 --check …/factor/balance.rs` → clean.
- Clippy: `cargo clippy -p runmat-runtime --lib --tests --message-format short` -> no diagnostics in this feature's files. The first runs could not reach them: another agent's `introspection/isprop.rs` failed to lint. Once they did, clippy reported 13 `clone` -> `std::slice::from_ref` lints (`cloned_ref_to_slice_refs`) in test code (11 in `balance.rs`, 1 each in `amd.rs` and `etree.rs`). I fixed them and the rerun showed none. The overall clippy exit is still 101 because of 3 errors in another agent's `graph/domain/editing.rs`, which I did not touch. After the fix, the feature tests (73 passed) and `math::sparse factor::svd` (129 passed) were rerun green.
- No release string appears in `balance.rs` (grep).

## Traceability (claim → FR → test → result) (T8)

| Claim / basis | FR | Test(s) | Result |
|---|---|---|---|
| balance.signature-primary, balance.output-class (single-output) | FR-047-01 | `observed_single_output` | PASS |
| balance.similarity, balance.power-of-two (two-outputs) | FR-047-02/03 | `observed_two_outputs` | PASS |
| balance.similarity (eig-preserved) | FR-047-02 | `observed_eig_preserved` | PASS |
| balance.already-balanced (balanced, scalar) | FR-047-05 | `observed_balanced`, `observed_scalar` | PASS |
| balance.noperm | FR-047-06 | `observed_noperm` | PASS |
| balance.similarity (permutation) | FR-047-02 | `observed_permutation` | PASS |
| balance.three-outputs | FR-047-04 | `observed_three_outputs` | PASS |
| balance.output-class (single-class) | FR-047-01 | `observed_single_class` | PASS |
| balance.error-nonsquare | FR-047-07 | `observed_nonsquare_errors` | PASS |
| balance.error-sparse | FR-047-07 | `observed_sparse_errors` | PASS |
| documented choice (balance.q-exact-values; balance.observed-values non-normative) | FR-047-08 | `documented_choice_values_on_observed_inputs` | PASS |
| documented choice: T = I(:,P)·diag(S) | FR-047-09 | `documented_choice_t_from_s_and_p` | PASS |
| unresolved balance.q-noperm-permuting; option text | FR-047-10 | `documented_choice_noperm_on_permutable_matrix` | PASS |
| unresolved balance.q-complex | FR-047-11 | `unresolved_complex_unsupported` | PASS |
| unresolved: other options | FR-047-10 | `unresolved_other_options_unsupported` | PASS |
| documented choice: input classes, empty, non-finite | FR-047-12 | `documented_choice_invalid_inputs`, `documented_choice_empty_input`, `documented_choice_non_finite_terminates` | PASS |
| documented choice: single outputs | FR-047-01/09 | `documented_choice_single_outputs` | PASS |
| documented choice: larger random matrix (similarity, 2^k generalized permutation, eigenvalues) | FR-047-02/03 | `documented_choice_random_badly_scaled_matrix` | PASS |
| documented choice: outputs | FR-047-13 | `documented_choice_too_many_outputs` | PASS |
| unresolved balance.q-warning-coverage | FR-047-14 | `observed_single_output`, `observed_two_outputs`, `observed_noperm` run under `assert_no_warnings` | PASS |

All 11 observed cases are covered. Every FR has at least one test, and there
are no orphan tests.

## Recorded non-normative values

For `A0 = [1 100 10000; 0.01 1 100; 0.0001 0.01 1]`, RunMat gives
`B = [1 1.5625 1.220703125; 0.64 1 0.78125; 0.8192 1.28 1]`, which matches
the reference `B` within 1e-12. RunMat's `T` is `diag([2^11 2^5 2^-2])`; the
reference `T` is `diag([2^9 2^3 2^-4])`. The two differ by the constant
factor 4, which leaves `B` unchanged. For `[1 0 0; 2 3 0; 4 5 6]`, `T` is the
reversal permutation and `B = [6 5 4; 0 3 2; 0 0 1]`, as in the reference.
None of these values is asserted as normative.

## Deviations and open items

- **Too-many-outputs** is tested by calling the registered implementation
  directly under `push_output_count` (upstream dispatcher re-invocation
  quirk). It was not verified through the VM.
- **Error identifiers**: `RunMat:square` and
  `RunMat:balance:noBalanceSparse`, following the namespace rule.
- **Single input** is computed in double and rounded back. Power-of-two
  scaling is exact, so this only matters at the overflow edge.
- **Not end-to-end through the VM**: the tests call the builtin through the
  runtime dispatcher.
