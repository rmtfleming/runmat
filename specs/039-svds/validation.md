# Validation Report — 039-svds (spec 0.3.0) — 2026-10-29

Environment: Linux x86_64; `HDF5_DIR=~/anaconda3`; test binaries run through
a cargo runner wrapper that puts anaconda's `lib/` on `LD_LIBRARY_PATH`. The
target directory is shared with four other agents, so several runs waited on
the build lock or were blocked by compile errors in other agents' files
(`table/builtins/variable_editing.rs`, `plotting/ops/{spy,exportgraphics}.rs`).
I did not edit those files; I waited and retried. MATLAB was not invoked.

## Freshness (T0)

In `../matlab-interface-spec` (read-only): `git fetch -q --all` (exit 0).
`git rev-parse HEAD origin/main` gave `8614c5b…` for both.
`git diff --name-only 8614c5b HEAD -- specifications/svds specifications/findstr`
returned nothing, as did the same diff against `origin/main`.
`git status --short` on those paths was clean. Result: the import is current.

## History

A first draft was written against the 0.2.0 import (pin `104fb5f`). Its
test-first run (0 passed / 11 failed, all `RunMat:UndefinedFunction`) is
superseded. Implementation was halted by the maintainer, and the feature was
redone from the 0.3.0 import below.

## Test-first evidence (T4)

The 0.2.0-era implementation was removed. `svds.rs` contained only the 0.3.0
tests, and `pub mod svds;` was re-added so the tests compile.

`cargo test -p runmat-runtime --lib -- svds_tests findstr_tests` →
**0 passed, 47 failed** (26 svds, 21 findstr). Every failure was
`RunMat:UndefinedFunction`: 37 tests panicked on "Undefined function: svds" or
"…findstr", and 10 reported identifier `RunMat:UndefinedFunction`.

## Commands and results (T7)

- `cargo test -p runmat-runtime --lib -- svds_tests findstr_tests` →
  **47 passed, 0 failed** (26 svds, 21 findstr).
  - The first post-implementation run gave 46/1. The failure was
    `documented_choice_two_outputs_and_k_zero_vectors`, explained under
    Deviations; the test was adjusted and the rerun is green.
- `cargo test -p runmat-runtime --lib -- factor::svd::` → 21 passed,
  0 failed (upstream `svd` unchanged, no regression).
- `cargo test -p runmat-runtime --lib -- compatibility::` → 6 failed. These
  are the six known upstream integer-metadata failures (`integer_audit_*`,
  `integer_capability_metadata_*`, `core_operator_extensions_*`,
  `integer_extensions_are_declared_*`, `generated_builtin_catalog_*` ×2).
  `svds` declares no integer metadata, so this feature does not affect them.
- `cargo test -p runmat-runtime --lib -- builtin::registry::` → 1 failed
  (`canonical_catalog_and_runtime_bindings_join_one_to_one`). It reports
  76 identity errors, all for unrelated builtins (`cell`, `cellfun`,
  `addpath`, …). None names `svds` or `findstr`, so this failure is not caused
  by this feature. I did not check whether it fails on a pristine tree.
- `cargo clippy -p runmat-runtime --lib --tests --message-format short` →
  finished with no errors and no diagnostics in `svds.rs`.
- `rustfmt --edition 2021 --check …/factor/svds.rs` → clean.
- Before any of this, I checked every cited `svds.*` claim id against the
  imported `provenance.yaml`. All 14 are present. Only the superseded 0.2.0
  q-ids `svds.q-default` and `svds.q-vectors` are absent, and they are cited
  only as resolved.

## Traceability (claim → FR → test → result) (T8)

| Claim / basis | FR | Test(s) | Result |
|---|---|---|---|
| svds.signature-primary, svds.output-class (k) | FR-039-01 | `observed_k` | PASS |
| svds.output-class (dense), svds.largest (dense) | FR-039-01/02 | `observed_dense` | PASS |
| svds.output-class (single) | FR-039-01 | `observed_single_input_gives_single` | PASS |
| svds.largest (order, largest) | FR-039-02 | `observed_order_descending`, `observed_largest` | PASS |
| svds.rectangular | FR-039-02 | `observed_rectangular` | PASS |
| svds.zero-matrix | FR-039-02 | `observed_zero_matrix` | PASS |
| svds.default-count | FR-039-03 | `observed_default_8`, `observed_default_2` | PASS |
| svds.k-exceeds (k-too-large, warn-k-too-large) | FR-039-04 | `observed_k_too_large_is_clipped_without_warning` | PASS |
| svds.no-warning (warn-k) | FR-039-04 | `observed_ordinary_call_issues_no_warning` | PASS |
| svds.k-zero | FR-039-04 | `observed_k_zero_is_0x1_double` | PASS |
| svds.smallest | FR-039-05 | `observed_smallest` | PASS |
| svds.sigma-numeric | FR-039-05 | `observed_sigma_numeric` | PASS |
| svds.usv (usv-shapes, usv-rect) | FR-039-06 | `observed_usv_shapes`, `observed_usv_rect` | PASS |
| svds.error-char | FR-039-07 | `observed_char_input_errors` | PASS |
| svds.error-k-negative | FR-039-07 | `observed_negative_k_errors` | PASS |
| documented choice (svds.q-default-rule) | FR-039-08 | `documented_choice_default_count_rule` | PASS |
| documented choice: sigma text forms, order | FR-039-09 | `documented_choice_sigma_text_forms_and_order` | PASS |
| documented choice (svds.q-sigma-numeric) | FR-039-10 | `documented_choice_sigma_numeric_ties_and_order` | PASS |
| documented choice: k validation | FR-039-11 | `documented_choice_k_validation` | PASS |
| documented choice: non-float A, N-D A | FR-039-12 | `documented_choice_non_float_inputs_rejected` | PASS |
| documented choice: 2 outputs, k=0 factors, >3 outputs, single factors | FR-039-13 | `documented_choice_two_outputs_and_k_zero_vectors`, `documented_choice_single_vectors_are_single` | PASS |
| unresolved svds.q-options | FR-039-14 | `unresolved_options_and_complex_are_unsupported` | PASS |
| unresolved svds.q-vector-signs | FR-039-15 | (signs deliberately not asserted; `observed_usv_*` check shapes, orthonormality, A·V = U·S) | PASS |
| unresolved svds.q-warning-coverage | FR-039-16 | the two `warn-*` tests above | PASS |

All 19 observed cases are covered. There are no orphan tests, and every FR
has at least one test.

## Deviations and open items

- **Dispatcher re-invocation with more than three outputs.** When `svds`
  errors under `call_builtin_async_with_outputs(…, 4)`, the dispatcher
  invoked the builtin a second time with no requested output count, and that
  call succeeded. A temporary debug print showed `Some(4)` followed by
  `None`, and a backtrace put both calls in `call_builtin_async_impl`'s
  implementation loop.
  - The `TooManyOutputs` check is therefore tested on the builtin itself,
    with `push_output_count(Some(4))`, which is the pattern sibling builtins
    use.
  - Whether a real `[a,b,c,d] = svds(…)` from the VM raises the error was
    not verified.
  - The debug code was removed.
  - This is upstream dispatcher behaviour, worth a separate look.
- **Error identifiers** use the `RunMat:svds:` namespace with the observed
  suffixes, following the D2 = B precedent. Flagged for maintainer review.
- **Not verified end-to-end through the VM.** The tests call the registered
  builtin through the runtime dispatcher.
- **Performance.** Sparse input is densified before the SVD.
- The WASM builtin registry was not regenerated (wasm32 target not
  installed).
