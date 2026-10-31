# Validation — 061-sparse-linalg-n5

Date: 2026-10-31. MATLAB was not invoked. All commands ran with
`HDF5_DIR=/home/rfleming/anaconda3` and the shared runner script
(`CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER`).

## Files

| File | Change |
|---|---|
| `crates/runmat-runtime/src/builtins/math/linalg/solve/lsqminnorm.rs` | new (`lsqminnorm` + `lsqminnorm_tests`) |
| `crates/runmat-runtime/src/builtins/math/sparse/sprank.rs` | new (`sprank` + `sprank_tests`) |
| `crates/runmat-runtime/src/builtins/array/creation/sparse/sprandn.rs` | new (`sprandn` + `sprandn_tests`) |
| `crates/runmat-runtime/src/builtins/math/linalg/solve/mod.rs` | hand-written: `pub mod lsqminnorm;` |
| `crates/runmat-runtime/src/builtins/math/sparse/mod.rs` | hand-written: `pub(crate) mod sprank;` |
| `crates/runmat-runtime/src/builtins/array/creation/sparse/mod.rs` | hand-written: `pub(crate) mod sprandn;` (no visibility change to the `sprand` helpers; the child module uses them as a descendant) |

No generated parent `mod.rs` was touched.

## Analysis (B3)

- Every normative claim (27) maps to an FR, and every FR to at least one
  test (table below). The three non-normative claims
  (`lsqminnorm.tol-rule`, `sprandn.density-out-of-range`,
  `sprandn.values`) map to FR-061-08 / D-061-3, FR-061-36 and
  FR-061-37 / D-061-5. None is asserted as MATLAB behaviour.
- No test is an orphan: each `observed_*` test cites its case and claim,
  and each `documented_choice_*`/`unresolved_*` test cites its FR or D.
- No task is without a requirement (`tasks.md`). Scope: three new files and
  three module lines in hand-written parents; no upstream code changed.
- Maintainer dispositions applied: `lsqminnorm.q-tol-rule` → D-061-3
  (pivoted QR truncated at the numerical rank; the observed `tol-arg`
  value is reproduced to 1e-12); `sprandn.q-values` → structure, class,
  bound and seeded repeatability only; `q-error-ids` → observed ids under
  `RunMat:` (D-061-1).

## Test-first record (Principle V)

1. Tests and descriptors were written in the same file as the bodies. Each
   builtin body then began with a stub line
   `if true { return Err(<fn>_error(&…, "stub")); } // STUB`.
2. Failing run, `cargo test -p runmat-runtime --lib -- lsqminnorm_tests
   sprank_tests sprandn_tests`:
   `test result: FAILED. 1 passed; 53 failed; 0 ignored` (13621 filtered).
   The one pass is `sprank_tests::documented_choice_matching_algorithm`,
   which calls the matching helper directly rather than the builtin.
3. The stub lines were removed. Passing run, same command:
   `test result: ok. 54 passed; 0 failed; 0 ignored` — lsqminnorm_tests 22
   (17 observed), sprank_tests 18 (14 observed), sprandn_tests 14
   (11 observed).

## Surrounding suites

- `cargo test -p runmat-runtime --lib -- builtins::math::linalg
  builtins::array::creation::sparse builtins::math::sparse`:
  **1038 passed, 0 failed**.

## Lint and format

- `rustfmt --edition 2021 --check` on the three new files: clean (applied
  with `rustfmt --edition 2021 <files>` after the stubs were removed).
- `cargo clippy -p runmat-runtime --lib --tests --message-format short`:
  exit 101 on one `type_complexity` error in another agent's in-progress
  file (`plotting/ops/rectangle.rs`); no diagnostics in F61 files. A
  second run with `-- -A clippy::type_complexity` reached the lib test
  target and reported 2 errors, both in other agents' files
  (`graph/domain/reordernodes.rs`, `math/rounding/rat.rs`). There were
  **no diagnostics in the F61 files, lib or tests**.

## Claim-id check

`grep claim_id` over the imported `provenance.yaml` files: lsqminnorm 12,
sprandn 10, sprank 8 = 30 ids, all cited in `provenance.md` and below.

## Traceability (claim → FR → test → result)

| claim | FR | test(s) | result |
|---|---|---|---|
| lsqminnorm.signature-primary | FR-061-01 | `lsqminnorm_tests::observed_square_full_rank`, `observed_underdetermined` | pass |
| lsqminnorm.vs-pinv | FR-061-01, 07 | `observed_square_full_rank`, `observed_overdetermined`, `observed_underdetermined`, `observed_rank_deficient`, `observed_sparse_transposed`, `observed_sparse_wide`, `observed_multi_rhs`, `observed_zero_matrix`, `observed_complex`, `observed_tol_small` | pass |
| lsqminnorm.full-output | FR-061-02 | `observed_sparse_transposed`, `observed_sparse_wide` | pass |
| lsqminnorm.cobra-form | FR-061-03 | `observed_sparse_transposed` (`S'` via RunMat `ctranspose` on sparse `S`; residual 3.4641) | pass |
| lsqminnorm.zero | FR-061-04 | `observed_zero_matrix` | pass |
| lsqminnorm.single | FR-061-05 | `observed_single` | pass |
| lsqminnorm.empty | FR-061-06 | `observed_empty` | pass |
| lsqminnorm.tol | FR-061-07 | `observed_tol_arg`, `observed_tol_small` | pass |
| lsqminnorm.tol-rule (non-normative) | FR-061-08, D-061-3 | `observed_tol_arg` (value reproduced), `documented_choice_tol_rule` | pass |
| lsqminnorm.error-rows | FR-061-09 | `observed_err_rows` | pass |
| lsqminnorm.error-char | FR-061-10 | `observed_err_char`, `observed_err_int` | pass |
| lsqminnorm.error-one-arg | FR-061-11 | `observed_err_one_arg`, `documented_choice_arity_and_outputs` | pass |
| sprank.signature-primary | FR-061-18 | `sprank_tests::observed_identity` | pass |
| sprank.structural | FR-061-19 | `observed_cancelling` (RunMat `rank` gives 1) | pass |
| sprank.matching | FR-061-20 | `observed_identity`, `observed_zero_row`, `observed_matching_limited`, `observed_wide`, `observed_tall` | pass |
| sprank.zero-empty | FR-061-21 | `observed_zero_matrix`, `observed_empty` | pass |
| sprank.inputs | FR-061-22 | `observed_dense`, `observed_logical`, `observed_complex` | pass |
| sprank.class | FR-061-23 | `observed_identity`, `observed_dense` (assert `Value::Num`) | pass |
| sprank.error-char | FR-061-24 | `observed_err_char`, `observed_err_cell` | pass |
| sprank.error-no-arg | FR-061-25 | `observed_err_no_arg` | pass |
| sprandn.signature-primary | FR-061-29 | `sprandn_tests::observed_cobra_5x6` | pass |
| sprandn.structure | FR-061-29 | `observed_cobra_5x6`, `observed_cobra_30x30`, `observed_density_one` (class, size, finite, both signs, `rng(7)` repeat) | pass |
| sprandn.nnz-bound | FR-061-30 | same three | pass |
| sprandn.zero-density | FR-061-31 | `observed_density_zero`, `observed_empty_dims` | pass |
| sprandn.pattern | FR-061-32 | `observed_pattern` | pass |
| sprandn.rc | FR-061-33 | `observed_rc` (cond = 10 within 1e-10) | pass |
| sprandn.density-out-of-range (non-normative) | FR-061-36 | `observed_density_above_one`, `observed_density_negative` | pass |
| sprandn.values (non-normative) | FR-061-37, D-061-5 | no value asserted; `documented_choice_count_and_stream` | pass |
| sprandn.error-noninteger | FR-061-34 | `observed_err_noninteger` | pass |
| sprandn.error-two-args | FR-061-35 | `observed_err_two_args` | pass |

Documented choices and unresolved points:

| FR | test(s) | result |
|---|---|---|
| FR-061-12 | `lsqminnorm_tests::documented_choice_arity_and_outputs` | pass |
| FR-061-13, 28, 39 | by construction (gather); no GPU provider in these unit tests | not separately tested |
| FR-061-14, 15 | `lsqminnorm_tests::documented_choice_unsupported` | pass |
| FR-061-16 | `lsqminnorm_tests::documented_choice_mixed_classes` | pass |
| FR-061-26 | `sprank_tests::documented_choice_input_forms` | pass |
| FR-061-27 | `sprank_tests::documented_choice_unsupported_and_outputs` | pass |
| FR-061-28 (algorithm) | `sprank_tests::documented_choice_matching_algorithm` (greedy-trap pattern; 200000×200000 bidiagonal) | pass |
| FR-061-36, 37 | `sprandn_tests::documented_choice_count_and_stream`, the two density observations | pass |
| FR-061-38 | `sprandn_tests::documented_choice_invalid_and_unsupported` | pass |
| FR-061-17 / 40 | `unresolved_no_warning` (×3) | pass |

Notes on the observed cases:

- `lsqminnorm` values are compared with the recorded numbers and with
  RunMat's own `pinv(full(A))*b` at 1e-12 (1e-6 for single), per
  `lsqminnorm.q-exactness`; the observed 2e-15 is not asserted digit for
  digit.
- `sprandn` counts in the observations (14, 229, 11, 7) are not asserted;
  only the upper bound is (`sprandn.q-density`). RunMat's counts are
  `floor(density*m*n)` (D-061-5).

## Deviations

- **D-061-4** (performance, not behaviour): `lsqminnorm` densifies sparse
  input before the SVD/QR, so memory is `O(m*n)` and time
  `O(m*n*min(m,n))`. Large genome-scale COBRA models may be slow. No test
  is ignored for this.

The open identifier questions (`*.q-error-ids`) are flagged to the
maintainer under D-061-1.

## Freshness check (finish)

2026-10-31, `../matlab-interface-spec` after `git fetch origin`:
HEAD = origin/main = `1c91aa1` (the pin). `git log 1c91aa1..origin/main`
over `specifications/{lsqminnorm,sprandn,sprank}` lists nothing; `git
status --short` for those paths is clean (no drafts). The imported specs
are current.
