# Validation — 050-numeric-conversion

Date: 2026-10-30. MATLAB was not invoked. All commands ran with
`HDF5_DIR=/home/rfleming/anaconda3` and the shared runner script.

## Files

| File | Change |
|---|---|
| `crates/runmat-runtime/src/builtins/math/elementwise/roots/nthroot.rs` | new (`nthroot` + `nthroot_tests`) |
| `crates/runmat-runtime/src/builtins/math/elementwise/logarithms/reallog.rs` | new (`reallog` + `reallog_tests`) |
| `crates/runmat-runtime/src/builtins/math/elementwise/integer_conversions/cast.rs` | new (`cast` + `cast_tests`) |
| `…/roots/mod.rs`, `…/logarithms/mod.rs`, `…/integer_conversions/mod.rs` | one `pub(crate) mod` line each; all three parents are hand-written (no "@generated" header) |

No generated parent was edited.

## Test-first record (Principle V)

1. Tests and descriptors written first; each builtin body began with a stub
   `return Err(…"stub")` line.
2. Failing run, `cargo test -p runmat-runtime --lib -- nthroot_tests
   reallog_tests cast_tests`:
   `test result: FAILED. 0 passed; 66 failed; 0 ignored` (13039 filtered).
3. Stubs removed (bodies implemented). Passing run, same command:
   `test result: ok. 66 passed; 0 failed; 0 ignored`. After the clippy
   fix (type alias in `nthroot.rs`) the re-run is again `66 passed; 0
   failed`.

Counts: nthroot 23 tests (14 observed), reallog 17 (12 observed), cast 26
(16 observed).

## Surrounding suites

`cargo test -p runmat-runtime --lib -- builtins::math::elementwise
builtin::registry compatibility::tests`: **789 passed, 7 failed**. The 7
failures are the known pre-existing ones and none involves these builtins
(no mention of `nthroot`/`reallog`/`cast` in their output):

- `builtin::registry::tests::canonical_catalog_and_runtime_bindings_join_one_to_one`
  (legacy `cellfun` authority);
- six `compatibility::tests` (`core_operator_extensions_reject_before_semantic_dispatch_in_matlab_mode`
  and the five integer-metadata tests).

## Lint and format

- `rustfmt --edition 2021 --check` on the three new files: clean.
- `cargo clippy -p runmat-runtime --lib --tests --message-format short`:
  the first run flagged `clippy::type_complexity` in `nthroot.rs`
  (`pair_up` return type); fixed with a `type Paired` alias. A second run
  flagged test-only lints: `excessive_precision`/`approx_constant` on the
  observed literals (kept verbatim; scoped `#[allow]` on the two test
  modules) and `cloned_ref_to_slice_refs` in `cast_tests` (now
  `std::slice::from_ref`). Final run: **no diagnostics in the F50 files**.
  The crate's `--tests` clippy build still stops on one error in another
  in-progress N2 file (`containers/map/conformance_tests.rs`), not in F50.
- Feature tests re-run after the lint fixes: `66 passed; 0 failed`.

## Claim-id check

`grep claim_id` over the imported `provenance.yaml` files: nthroot 11,
reallog 7, cast 9 = 27 ids, all cited in `provenance.md` and below.

## Traceability (claim → FR → test → result)

| claim | FR | test(s) | result |
|---|---|---|---|
| nthroot.signature-primary | FR-050-01 | `nthroot_tests::observed_cube_neg` | pass |
| nthroot.output-class | FR-050-02 | `observed_cube_neg`, `observed_single`, `documented_choice_single_n_double_x` | pass |
| nthroot.negative-odd | FR-050-03 | `observed_cube_neg`, `observed_vector` | pass |
| nthroot.even-root | FR-050-03 | `observed_fourth` | pass |
| nthroot.elementwise-n | FR-050-04 | `observed_vector_n` | pass |
| nthroot.specials | FR-050-05 | `observed_specials` | pass |
| nthroot.n-values | FR-050-06 | `observed_n_one`, `observed_n_negative`, `observed_noninteger_n` | pass |
| nthroot.exact | FR-050-07 | `observed_exactness`, `documented_choice_perfect_powers_exact` | pass |
| nthroot.error-even-negative | FR-050-08 | `observed_err_even_negative`, `observed_err_noninteger_n_neg` | pass |
| nthroot.error-complex | FR-050-09 | `observed_err_complex` | pass |
| nthroot.error-size | FR-050-10 | `observed_err_size` | pass |
| reallog.signature-primary | FR-050-11 | `reallog_tests::observed_one` | pass |
| reallog.values | FR-050-11 | `observed_one`, `observed_e`, `observed_vector` | pass |
| reallog.specials | FR-050-12 | `observed_zero`, `observed_inf`, `observed_nan` | pass |
| reallog.shape-class | FR-050-13 | `observed_matrix`, `observed_single`, `observed_empty` | pass |
| reallog.error-negative | FR-050-14 | `observed_err_negative`, `documented_choice_negative_in_array` | pass |
| reallog.error-complex | FR-050-14 | `observed_err_complex`, `documented_choice_array_classes` | pass |
| reallog.error-int | FR-050-14 | `observed_err_int`, `documented_choice_array_classes` | pass |
| cast.signature-primary | FR-050-15 | `cast_tests::observed_round` | pass |
| cast.rounding | FR-050-16 | `observed_round` | pass |
| cast.saturation | FR-050-16 | `observed_saturate`, `observed_int_to_int`, `observed_like` | pass |
| cast.nan-inf | FR-050-16 | `observed_nan_inf` | pass |
| cast.other-classes | FR-050-17 | `observed_to_single`, `observed_char_to_double`, `observed_logical_to_double`, `observed_to_logical`, `observed_to_char` | pass |
| cast.like | FR-050-15, FR-050-18 | `observed_like`, `observed_like_complex` (class + value only, D-050-3) | pass |
| cast.shape | FR-050-19 | `observed_shape`, `observed_same_class` | pass |
| cast.error-bad-type | FR-050-20 | `observed_err_bad_type`, `observed_err_struct_type` | pass |
| cast.error-cell | FR-050-20 | `observed_err_cell` | pass |

Documented choices and unresolved points:

| FR | test(s) | result |
|---|---|---|
| FR-050-21 | `nthroot_tests::documented_choice_arity`, `nthroot_tests::documented_choice_too_many_outputs`, `reallog_tests::documented_choice_arity_and_outputs`, `cast_tests::documented_choice_arity_and_outputs` | pass |
| FR-050-22 | `documented_choice_scalar_x_array_n`, `documented_choice_matrix_shapes`, `documented_choice_expansion_and_mismatch` | pass |
| FR-050-23 | `nthroot_tests::documented_choice_unsupported_classes`, `documented_choice_single_n_double_x` | pass |
| FR-050-24 | `documented_choice_edge_cases`, `documented_choice_perfect_powers_exact` | pass |
| FR-050-25 | `reallog_tests::documented_choice_unsupported_classes` | pass |
| FR-050-26 | by construction (`gather_if_needed_async`); no GPU provider in the unit-test environment | not separately tested |
| FR-050-27 | `documented_choice_all_supported_classes`, `documented_choice_text_forms`, `documented_choice_other_class_names`, `documented_choice_unsupported_forms`, `documented_choice_class_checked_first` | pass |
| FR-050-28 | `documented_choice_like_unsupported_prototype`, `observed_like_complex` | pass |
| FR-050-29 | `documented_choice_complex_source_delegated` | pass |
| FR-050-30 | `unresolved_no_warning` (×3) | pass |

## Deviations

None. D-050-3 (`cast(x,'like',complex)`) is a documented choice, not a
deviation: RunMat follows the claim text (real result), and the
disagreement between that claim and the observation record's
`isreal: false` is returned to the specification side.

## Freshness check (finish)

2026-10-30, `../matlab-interface-spec` after `git fetch -q origin`:
HEAD = origin/main = `1156c97`. `git diff --name-only 8f72edb HEAD` and the
same against `origin/main` over `specifications/{nthroot,reallog,cast}`
returned nothing; `git status --short` for those paths was clean. The
imported specs are current.
