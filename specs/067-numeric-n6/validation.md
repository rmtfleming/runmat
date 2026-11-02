# Validation — 067-numeric-n6

Date: 2026-11-02. MATLAB was not invoked. All commands ran with
`HDF5_DIR=/home/rfleming/anaconda3` and the shared runner script
(`CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER`).

## Files

| File | Change |
|---|---|
| `crates/runmat-runtime/src/builtins/math/linalg/solve/normest.rs` | new (`normest` + `normest_tests`) |
| `crates/runmat-runtime/src/builtins/math/optim/lsqnonneg.rs` | new (`lsqnonneg` + `lsqnonneg_tests`) |
| `crates/runmat-runtime/src/builtins/math/linalg/solve/mod.rs` | hand-written: `pub mod normest;` |
| `crates/runmat-runtime/src/builtins/math/optim/mod.rs` | hand-written: `pub mod lsqnonneg;` |

No generated parent `mod.rs` was touched.

## Analysis (B3)

- Every normative claim (10 normest + 14 lsqnonneg = 24) maps to an FR, and
  every FR to at least one test (tables below). The four non-normative
  claims (`normest.char`, `normest.iteration-count`, `lsqnonneg.nonunique`,
  `lsqnonneg.iterations`) map to FR-067-11, 12, 34, 35; none is asserted as
  MATLAB behaviour beyond what was observed.
- No orphan tests: each `observed_*` test cites its case(s) and claim(s);
  each `documented_choice_*`/`unresolved_*` test cites its FR or D.
- No task without a requirement (`tasks.md`). Scope: two new files and two
  module lines in hand-written parents; no upstream code changed.
- Maintainer dispositions applied: normest accuracy only (tolerance
  comparisons with RunMat's `norm`, `cnt` not compared); lsqnonneg
  `output.algorithm`/`message` present with RunMat values; non-unique case
  asserts objective and KKT; `q-error-ids` raised under `RunMat:`.
- D-067-2 evidence for the default `tol = 1e-8`: an independent numpy
  re-run of the same iteration (scratch only, not part of the repository)
  gives, on `[3 0; 0 4]`, relative difference 3.9e-7 with `tol = 1e-6`
  (above the observed bound 2.3e-7) and 3.9e-9 with `tol = 1e-8`.

## Test-first record (Principle V)

1. Tests and descriptors were written with the bodies; each builtin body
   (normest, lsqnonneg, and web of F68, run together) began with
   `if true { return Err(<fn>_error(&INTERNAL, "stub")); } // STUB`.
2. Failing run, `cargo test -p runmat-runtime --lib -- normest_tests
   lsqnonneg_tests web_tests`:
   `test result: FAILED. 0 passed; 64 failed; 1 ignored` (13703 filtered);
   normest_tests 23 failed, lsqnonneg_tests 22 failed (web_tests 19 failed,
   1 ignored).
3. Stub lines removed, `rustfmt --edition 2021` applied. Passing run, same
   command: `test result: ok. 64 passed; 0 failed; 1 ignored` —
   normest_tests 23 (19 observed tests covering 20 cases), lsqnonneg_tests
   22 (18 observed). The ignored test is F68's deviation test.

## Surrounding suites

- `cargo test -p runmat-runtime --lib -- builtins::math::linalg
  builtins::math::optim builtins::io::http`: **1215 passed, 0 failed,
  1 ignored** (the F68 deviation test).

## Lint and format

- `rustfmt --edition 2021 --check` on both files: clean.
- `cargo clippy -p runmat-runtime --lib --tests --message-format short`:
  the first run reported one `manual_contains` lint in
  `lsqnonneg.rs` (test code); fixed (`all.x.contains(&0.0)`). Second run:
  **exit 0, no diagnostics** (lib and tests, whole crate). Feature tests
  re-run after the fix: `64 passed; 0 failed; 1 ignored`.

## Claim-id check

`grep claim_id` over the imported `provenance.yaml` files: normest 12,
lsqnonneg 16 = 28 ids, all cited in `provenance.md` and below.

## Traceability (claim → FR → test → result)

| claim | FR | test(s) (`normest_tests::` / `lsqnonneg_tests::`) | result |
|---|---|---|---|
| normest.signature-primary | FR-067-01 | `observed_sparse_stoich`, `observed_one_out`, `observed_tol_coarse_and_fine` | pass |
| normest.output-class | FR-067-02 | `observed_sparse_stoich`, `observed_identity`, `observed_single` (single 1×1), all `call2` tests (`cnt` double) | pass |
| normest.default-accuracy | FR-067-03 | `observed_dense_square`, `observed_dense_wide`, `observed_one_out`, `observed_row_vector`, `observed_column_vector`, `observed_identity`, `observed_complex`, `observed_single`, `observed_sparse_stoich` (≤ 2.3e-7 vs RunMat `norm`) | pass |
| normest.cobra-form | FR-067-04 | `observed_sparse_stoich` (`|nrm-2| ≤ 1e-12`) | pass |
| normest.tol | FR-067-05 | `observed_tol_coarse_and_fine` (cases `tol-coarse`, `tol-fine`, `dense-square`) | pass |
| normest.zero | FR-067-06 | `observed_zero`, `observed_sparse_zero`, `observed_empty` | pass |
| normest.scalar-nan | FR-067-07 | `observed_scalar`, `observed_with_nan` | pass |
| normest.char (non-normative) | FR-067-11 | `observed_char_input` | pass |
| normest.iteration-count (non-normative) | FR-067-12 | `documented_choice_count`, ordering in `observed_tol_coarse_and_fine` | pass |
| normest.error-int | FR-067-08 | `observed_err_int` | pass |
| normest.error-3d | FR-067-09 | `observed_err_3d` | pass |
| normest.error-no-arg | FR-067-10 | `observed_err_no_arg` | pass |
| lsqnonneg.signature-primary | FR-067-20 | `observed_cobra_zero_rhs`, `observed_cobra_one_out`, `observed_options_tolx` | pass |
| lsqnonneg.output-class | FR-067-21 | `observed_interior`, `observed_sparse` (`column()` asserts full double n×1) | pass |
| lsqnonneg.cobra-form | FR-067-22 | `observed_cobra_zero_rhs`, `observed_cobra_one_out` | pass |
| lsqnonneg.solutions | FR-067-23 | `observed_interior`, `observed_bound_active`, `observed_all_negative`, `observed_one_out`, `observed_overdetermined` | pass |
| lsqnonneg.residual | FR-067-24 | `observed_bound_active`, `observed_all_negative`, `observed_overdetermined`, `assert_consistent` | pass |
| lsqnonneg.lambda | FR-067-25 | same, plus `observed_rank_deficient`, `observed_sparse`, `observed_empty_rows` | pass |
| lsqnonneg.output-struct | FR-067-26 | `assert_consistent` (exitflag 1, fields `iterations, algorithm, message`), `observed_empty_rows` | pass |
| lsqnonneg.options | FR-067-27 | `observed_options_tolx`, `observed_options_display` (RunMat `optimset`) | pass |
| lsqnonneg.empty | FR-067-28 | `observed_empty_rows` | pass |
| lsqnonneg.nonunique (non-normative) | FR-067-34 | `observed_rank_deficient` (objective + KKT only), `documented_choice_kkt_larger` | pass |
| lsqnonneg.iterations (non-normative) | FR-067-35 | `assert_consistent` (non-negative integer only) | pass |
| lsqnonneg.error-single | FR-067-29 | `observed_err_single`, `observed_err_char` | pass |
| lsqnonneg.error-complex | FR-067-30 | `observed_err_complex` | pass |
| lsqnonneg.error-size | FR-067-31 | `observed_err_size` | pass |
| lsqnonneg.error-row-d | FR-067-32 | `observed_err_row_d` | pass |
| lsqnonneg.error-one-arg | FR-067-33 | `observed_err_one_arg`, `documented_choice_unsupported` (no input) | pass |

Documented choices and unresolved points:

| FR | test(s) | result |
|---|---|---|
| FR-067-13 (D-067-2) | `normest_tests::documented_choice_larger_sparse_and_null_start` (40×60 sparse within 2.3e-7; null-space start `[1 -1]`) | pass |
| FR-067-14 | `normest_tests::documented_choice_unsupported_and_outputs` | pass |
| FR-067-15 | sparse path exercised by the sparse tests (CSC, never densified, by construction); gather: no GPU provider in unit tests | by construction |
| FR-067-16 | `normest_tests::unresolved_no_warning` | pass |
| FR-067-36, 37, 38 | `lsqnonneg_tests::documented_choice_unsupported` | pass |
| FR-067-39 | `lsqnonneg_tests::documented_choice_output_struct` | pass |
| FR-067-40 | sparse densified (`observed_sparse`); gather by construction | pass / by construction |
| FR-067-41 | `lsqnonneg_tests::unresolved_no_warning` | pass |

## Deviations

- **D-067-6** (performance, not behaviour): `lsqnonneg` densifies sparse
  `C`. No test is ignored for this.

No behavioural deviation. The `q-error-ids` questions are flagged to the
maintainer under D-067-1.

## Freshness check (finish)

2026-11-02, `../matlab-interface-spec` after `git fetch origin`:
HEAD = origin/main = `20825f8`. `git log e364206..origin/main` over
`specifications/{normest,lsqnonneg,web}` lists nothing; `git status --short`
for those paths is clean (no drafts). The imported specs are current.
