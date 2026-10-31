# Validation — 055-numeric-n3n4

Date: 2026-10-30. MATLAB was not invoked. All commands ran with
`HDF5_DIR=/home/rfleming/anaconda3` and the shared runner script
(`CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER`).

## Files

| File | Change |
|---|---|
| `crates/runmat-runtime/src/builtins/math/trigonometry/sec.rs` | new (`sec` + `sec_tests`) |
| `crates/runmat-runtime/src/builtins/math/linalg/factor/orth.rs` | new (`orth` + `orth_tests`) |
| `crates/runmat-runtime/src/builtins/math/linalg/solve/condest.rs` | new (`condest` + `condest_tests`) |
| `crates/runmat-runtime/src/builtins/math/trigonometry/mod.rs` | **generated parent**: one hand-added `pub(crate) mod sec;` line (D-055-2) |
| `crates/runmat-runtime/src/builtins/math/linalg/factor/mod.rs` | hand-written: `pub mod orth;` |
| `crates/runmat-runtime/src/builtins/math/linalg/solve/mod.rs` | hand-written: `pub mod condest;` |

## Analysis (B3)

- Every normative claim (26) maps to an FR, and every FR to at least one
  test (table below). The two non-normative claims (`orth.diag`,
  `orth.basis-not-unique`) map to FR-055-17, and `condest.estimate` maps to
  D-055-3. None of them is asserted as a requirement.
- No test is an orphan: each `observed_*` test cites its case and claim, and
  each `documented_choice_*`/`unresolved_*` test cites its FR.
- No task is without a requirement (`tasks.md`). Scope: three new files and
  three module lines; no upstream code was changed.
- Placement risk: `math/trigonometry/mod.rs` is generated, so the hand-added
  line must be listed in
  `builtins/parallel_implementations/README.md` ("Generated parent
  modules"). That shared file was left to the coordinator.

## Test-first record (Principle V)

1. Tests and descriptors were written in the same file as the bodies. Each
   builtin body then began with a stub line
   `if true { return Err(<fn>_error(&INTERNAL, "stub")); } // STUB`.
2. Failing run, `cargo test -p runmat-runtime --lib -- sec_tests orth_tests
   condest_tests strread_tests as_many_of_pattern_tests` (shared with F57):
   `test result: FAILED. 0 passed; 94 failed; 0 ignored` (13408 filtered).
   For F55, sec_tests 20 failed, orth_tests 20 failed, condest_tests 17
   failed.
3. The stub lines were removed. Passing run, same command:
   `test result: ok. 94 passed; 0 failed; 0 ignored`. For F55, sec_tests 20
   (15 observed), orth_tests 20 (15 observed) and condest_tests 17
   (12 observed) passed.

## Surrounding suites

- `cargo test -p runmat-runtime --lib -- builtins::math::trigonometry
  builtins::math::linalg builtins::strings builtins::io::filetext`:
  **2581 passed, 0 failed**.
- `cargo test -p runmat-runtime --lib -- builtin::registry
  compatibility::tests`: **5 passed, 8 failed**. Seven are the known
  pre-existing failures (registry one-to-one, `core_operator_extensions…`
  and the five integer-metadata tests). The eighth,
  `compatibility_version_pin_is_centralized_in_the_repository`, points at
  `builtins/io/repl_fs/environment/release.rs`, another agent's
  in-progress F53 file. None of the failures mentions an F55 builtin.

## Lint and format

- `rustfmt --edition 2021 --check` on the three new files: clean.
- `cargo clippy -p runmat-runtime --lib --tests --message-format short`:
  **no diagnostics in the F55 files**. The run still ends with exit 101,
  because of errors in other agents' in-progress files
  (`plotting/ops/findall.rs`, `pcolor.rs`, `surfl.rs`). Since the lib target
  fails, cargo does not reach the `--tests` clippy pass, so the F55 test
  modules were not checked by this first run (see the second run). The test modules carry a scoped
  `#[allow(clippy::excessive_precision, …)]` for the verbatim observed
  literals.

- Second clippy run, with only the two lints raised in the other agents'
  plotting files allowed on the command line
  (`-- -A clippy::manual_is_multiple_of -A clippy::unnecessary_to_owned`)
  so the `--tests` pass is reached: the lib test target was checked and
  reported 4 errors, all in other agents' in-progress files
  (`graph/domain/isomorphism.rs`, `graph/domain/paths.rs`,
  `table/builtins/convertvars.rs`). There were **no diagnostics in the F55
  files, lib or tests**.

## Claim-id check

`grep claim_id` over the imported `provenance.yaml` files: sec 9, orth 10,
condest 9 = 28 ids, all cited in `provenance.md` and below.

## Traceability (claim → FR → test → result)

| claim | FR | test(s) | result |
|---|---|---|---|
| sec.signature-primary | FR-055-01 | `sec_tests::observed_zero` | pass |
| sec.values | FR-055-01 | `observed_zero`, `observed_pi3`, `observed_pi`, `observed_negative` | pass |
| sec.pole | FR-055-02 | `observed_pole` | pass |
| sec.shape | FR-055-03 | `observed_matrix`, `observed_empty` | pass |
| sec.nonfinite | FR-055-04 | `observed_nan`, `observed_inf` | pass |
| sec.complex | FR-055-05 | `observed_complex` | pass |
| sec.single | FR-055-06 | `observed_single` | pass |
| sec.error-logical | FR-055-07 | `observed_err_logical`, `observed_err_int`, `observed_err_char`, `documented_choice_array_classes` | pass |
| sec.error-none | FR-055-08 | `observed_err_none` | pass |
| orth.signature-primary | FR-055-09, 11 | `orth_tests::observed_full_rank` | pass |
| orth.size-rank | FR-055-09 | `observed_full_rank`, `observed_rank2`, `observed_rank1`, `observed_zero`, `observed_tall`, `observed_wide` | pass |
| orth.orthonormal | FR-055-10 | the six above plus `observed_single`, `observed_complex` | pass |
| orth.class | FR-055-11, 12 | `observed_single`, `observed_complex` | pass |
| orth.empty | FR-055-13 | `observed_empty`, `observed_empty_3x0` | pass |
| orth.second-arg | FR-055-14 | `observed_second_arg` | pass |
| orth.error-sparse | FR-055-15 | `observed_err_sparse` | pass |
| orth.error-int | FR-055-16 | `observed_err_int`, `observed_err_char` | pass |
| orth.diag (non-normative) | FR-055-17 | `observed_diag_abs` (shape, span and e2 ⟂ range only) | pass |
| orth.basis-not-unique (non-normative) | FR-055-17 | all `orth_tests` assert structure only | pass |
| condest.signature-primary | FR-055-18 | `condest_tests::observed_identity` | pass |
| condest.values | FR-055-19 | `observed_identity`, `observed_diag`, `observed_dense`, `observed_scalar` | pass |
| condest.vs-cond | FR-055-20 | `observed_vs_cond`, `observed_vs_cond_unsym` | pass |
| condest.singular | FR-055-21 | `observed_singular`, `observed_zero` | pass |
| condest.empty | FR-055-22 | `observed_empty` | pass |
| condest.two-out | FR-055-18, 23 | `observed_two_out` | pass |
| condest.estimate (non-normative) | D-055-3 | `documented_choice_exact_value_larger` | pass |
| condest.error-nonsquare | FR-055-24 | `observed_err_nonsquare` | pass |
| condest.error-char | FR-055-25 | `observed_err_char` | pass |

Documented choices and unresolved points:

| FR | test(s) | result |
|---|---|---|
| FR-055-26 | `condest_tests::documented_choice_v_singular_and_empty` | pass |
| FR-055-27 | `orth_tests::documented_choice_default_threshold` | pass |
| FR-055-28 | `orth_tests::documented_choice_absolute_threshold`, `documented_choice_unsupported` | pass |
| FR-055-29 | `sec_tests::documented_choice_arity_and_outputs`, `orth_tests::documented_choice_arity_and_outputs`, `condest_tests::documented_choice_arity_and_outputs` | pass |
| FR-055-30 | `sec_tests::documented_choice_unsupported_classes`, `orth_tests::documented_choice_unsupported`, `condest_tests::documented_choice_unsupported` | pass |
| FR-055-31 | by construction (`gather_if_needed_async`); no GPU provider used in these unit tests | not separately tested |
| FR-055-32 | `sec_tests::documented_choice_complex_arrays` | pass |
| FR-055-33 | `unresolved_no_warning` (×3) | pass |

Notes on the observed cases:

- `vs-cond`: the observation does not record the tridiagonal matrix.
  The test uses tridiag(−1, 2, −1), whose cond1 is 12, and checks agreement
  with RunMat's `cond(A, 1)` (relative 1e-14). The observed value
  2.7272727272727271 is not asserted, because its input is unknown.
- `orth` residuals are Frobenius norms, which bound the 2-norm in the
  observation record from above. The thresholds are therefore no weaker
  than the observed ones.

## Deviations

None. The open identifier questions (`orth.q-error-ids`,
`condest.q-error-ids`) are flagged to the maintainer under D-055-1.

## Freshness check (finish)

2026-10-30, `../matlab-interface-spec` after `git fetch -q origin`:
HEAD = origin/main = `1c91aa1` (new commits since the start, none touching
these specs). `git log 1156c97..origin/main` over `specifications/{sec,orth,strread}`
and `git log fc9f804..origin/main` over `specifications/{condest,asManyOfPattern}`
list nothing; `git status --short` for those spec and experiment paths is
clean (no drafts). The imported specs are current.
