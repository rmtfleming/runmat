# Validation Report — 009-sparse-construction — 2026-07-22

## Commands and results

- `cargo check -p runmat-runtime` — clean (1m 57s)
- `cargo fmt --all -- --check` — clean
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::math::sparse` — **20 passed, 0 failed** (6573 filtered out)

## Traceability (claim → FR → test → result)

All tests below live in
`crates/runmat-runtime/src/builtins/math/sparse/{full,speye,spdiags}.rs`.

| Claim | FR | Test | Result |
|---|---|---|---|
| full.signature-primary, output-class, full-preserves-values | FR-009-01 | `full::tests::observed_sparse_matrix_to_dense`, `observed_sparse_vector_to_dense`, `observed_empty_sparse_to_empty_dense` | PASS |
| full.full-preserves-values (already-full) | FR-009-02 | `full::tests::observed_dense_input_passes_through` | PASS |
| unresolved choices (full.q-class) | FR-009-03 | `full::tests::unresolved_choice_other_full_kinds_pass_through`, `internal_error_on_dimension_overflow` | PASS (non-normative) |
| speye.signature-primary, output-class, sparse-identity | FR-009-04, FR-009-05 | `speye::tests::observed_square_identity`, `observed_rectangular_identity`, `observed_size_vector_identity`, `observed_zero_size_identity` | PASS |
| unresolved choices (speye.q-class) | FR-009-06 | `speye::tests::unresolved_choice_invalid_sizes_error`, `too_many_arguments_error` | PASS (non-normative) |
| spdiags.signature-primary, output-class, extract-vs-construct (extract) | FR-009-07 | `spdiags::tests::observed_extract_dense_diagonals`; `summary_derived_extract_from_sparse_input` (summary-derived) | PASS |
| spdiags.extract-vs-construct (construct) | FR-009-08 | `spdiags::tests::observed_construct_single_diagonal` | PASS |
| (summary) second output d = diagonal indices | FR-009-09 | `spdiags::tests::summary_derived_second_output_diagonal_indices` | PASS (summary-derived) |
| unresolved choices (spdiags.q-forms, diagonal rule) | FR-009-10 | `spdiags::tests::unresolved_choice_subdiagonal_padding`, `unresolved_choice_wide_matrix_round_trip`, `unresolved_choice_other_forms_error`, `construct_size_mismatch_errors` | PASS (non-normative) |

All 10 observed cases across the three exports reproduce exactly (value,
class double, shape, sparse/dense storage kind via `Value::SparseTensor` /
`Value::Tensor`, per RunMat's value model).

## Deviations and notes

- RunMat already ships a `sparse` builtin
  (`crates/runmat-runtime/src/builtins/array/creation/sparse.rs`). Feature
  009 does not call it: `full`/`speye`/`spdiags` construct and consume
  `SparseTensor` values directly, and test expectations translate the
  exports' `sparse(i, j, v, m, n)` value notation into equivalent CSC
  literals.
- The spdiags diagonal-numbering/padding rule is an independent documented
  choice (spec FR-009-10) verified against the observed extract case
  element-for-element and by a construct/extract round trip; only the
  observed cases are claimed MATLAB-conformant.
- `SparseTensor` is real-valued f64, so `full.q-complex` cannot arise on
  this substrate; carried unresolved for the specification side.
- Registration is via the `#[runtime_builtin]` inventory macro
  (compile-time, same mechanism as existing builtins); shared size/scalar
  parsing helpers live in `math/sparse/mod.rs`.

## Unresolved (returned to specification side)

`full.q-class`, `full.q-complex`, `speye.q-class`, `spdiags.q-forms`,
`spdiags.q-outputs` (observation for the `[B, d]` extraction form and the
remaining call forms).
