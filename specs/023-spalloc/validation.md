# Validation Report — 023-spalloc — 2026-07-22

## Commands and results

- `cargo check -p runmat-runtime` — clean (finished in 16.54s; exit 0).
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::math::sparse::spalloc`
  — **8 passed, 0 failed, 0 ignored** (6854 filtered out). Actual output:

  ```
  running 8 tests
  test builtins::math::sparse::spalloc::tests::observed_basic_all_zero_sparse ... ok
  test builtins::math::sparse::spalloc::tests::spalloc_type_is_tensor ... ok
  test builtins::math::sparse::spalloc::tests::unresolved_choice_invalid_dimensions_error ... ok
  test builtins::math::sparse::spalloc::tests::unresolved_choice_invalid_nz_error ... ok
  test builtins::math::sparse::spalloc::tests::unresolved_choice_nz_capacity_does_not_change_result ... ok
  test builtins::math::sparse::spalloc::tests::unresolved_choice_rectangular_and_empty_sizes ... ok
  test builtins::math::sparse::spalloc::tests::unresolved_choice_scalar_like_argument_kinds ... ok
  test builtins::math::sparse::spalloc::tests::unresolved_choice_wrong_argument_count_error ... ok

  test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 6854 filtered out
  ```

- `cargo fmt` — idempotent (exit 0, no changes). `cargo fmt --all -- --check`
  — **clean (exit 0)**. `spalloc.rs` was clean throughout
  (`rustfmt --edition 2021 --check` exit 0). During implementation the
  workspace check transiently reported diffs only in **sibling** batch-4 files
  being implemented concurrently (`math/reduction/normalize.rs`,
  `math/sparse/colamd.rs`, `math/sparse/dissect.rs`, `stats/hist/histc.rs`);
  per the concurrent-implementation protocol those files were not edited, and
  the workspace check is now clean after the siblings settled.

## Traceability (claim → FR → test → result)

| Claim | FR | Test | Result |
|---|---|---|---|
| spalloc.signature-primary | FR-023-01 | `observed_basic_all_zero_sparse`; `unresolved_choice_wrong_argument_count_error` | PASS |
| spalloc.output-class | FR-023-02 | `observed_basic_all_zero_sparse` (Value::SparseTensor ≡ sparse double) | PASS |
| spalloc.all-zero-sparse | FR-023-03 | `observed_basic_all_zero_sparse` (size [3,3], nnz 0) | PASS |
| (documented choice) spalloc.q-nz-capacity | FR-023-04 | `unresolved_choice_nz_capacity_does_not_change_result`; `unresolved_choice_invalid_nz_error` | PASS (non-normative) |
| (documented choice) spalloc.q-dims | FR-023-04 | `unresolved_choice_rectangular_and_empty_sizes`; `unresolved_choice_scalar_like_argument_kinds` | PASS (non-normative) |
| (documented choice) spalloc.q-invalid-dims | FR-023-05 | `unresolved_choice_invalid_dimensions_error` | PASS (non-normative) |
| (documented choice) spalloc.q-arity | FR-023-05 | `unresolved_choice_wrong_argument_count_error` | PASS (non-normative) |
| — | (type resolver) | `spalloc_type_is_tensor` | PASS |

The single observed export case reproduces exactly: value
`sparse(zeros(0,1), zeros(0,1), zeros(0,1), 3, 3)` ≡ `SparseTensor::zeros(3, 3)`,
class `double` (RunMat sparse double), size `[3, 3]`, `issparse` true, `nnz`
`0`.

## Deviations and notes

- `nz` is validated as a nonnegative integer but has no effect on the result:
  RunMat's compressed-sparse-column storage has no separate reserved capacity,
  so the capacity hint cannot change the observable content (nnz stays 0). This
  is a documented independent choice, since the export observed only `nz = 4`.
- Registration is via the compile-time `#[runtime_builtin]` inventory macro
  (same mechanism as all existing builtins) plus a `pub(crate) mod spalloc;`
  line in `math/sparse/mod.rs`; no existing file is otherwise modified.
- No external algorithm source was needed (Principle X): the result is the
  empty-CSC constructor `SparseTensor::zeros`. No MathWorks source,
  documentation, or examples were consulted; MATLAB was not invoked.

## Unresolved (returned to specification side)

`spalloc.q-nz-capacity` (effect of nz beyond the observed value 4),
`spalloc.q-dims` (sizes beyond the observed 3×3), `spalloc.q-invalid-dims`
(non-integer / negative / non-scalar dimensions), `spalloc.q-arity` (argument
counts other than the observed three-argument form).
