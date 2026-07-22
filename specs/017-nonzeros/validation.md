# Validation Report — 017-nonzeros — 2026-07-22

## Commands and results

- `cargo check -p runmat-runtime` — clean (Finished `dev` profile, 1m 12s)
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::math::sparse::nonzeros`
  — **13 passed, 0 failed** (re-run after `cargo fmt`; identical result)
- `cargo fmt` then `cargo fmt --all -- --check` — **clean** on the final
  run. (An intermediate workspace-wide check transiently failed on sibling
  features' files — `logical/tests/istable.rs`, `math/elementwise/psi.rs` —
  being implemented concurrently in the shared tree; this feature's files
  were verified clean throughout with `rustfmt --check --edition 2021`, and
  the workspace-wide check passed once the siblings formatted; see
  Deviations.)
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::math::sparse`
  (whole category, including this feature) — **33 passed, 0 failed**

## Traceability (claim → FR → test → result)

| Claim | FR | Test (`builtins::math::sparse::nonzeros::tests::`) | Result |
|---|---|---|---|
| nonzeros.signature-primary, output-class, column-major-order | FR-017-01..03 | `observed_row_vector_nonzeros` | PASS |
| nonzeros.output-class, column-major-order | FR-017-02..03 | `observed_matrix_nonzeros` | PASS |
| nonzeros.column-major-order | FR-017-03 | `observed_discriminating_column_major_order` | PASS |
| nonzeros.output-class, column-major-order | FR-017-02..03 | `observed_sparse_vector_nonzeros` | PASS |
| nonzeros.column-major-order (rule statement: "works for … sparse inputs") | FR-017-03 | `summary_derived_sparse_matrix_column_major` | PASS (claim-statement-derived; no exact backing observation) |
| documented choices | FR-017-04 | `unresolved_choice_all_zero_input_yields_0x1_empty` | PASS (non-normative) |
| documented choices | FR-017-05 | `unresolved_choice_scalar_inputs`, `unresolved_choice_nan_kept_negative_zero_dropped`, `unresolved_choice_logical_array_returns_double`, `unresolved_choice_complex_nonzeros_column_major`, `unresolved_choice_nd_input_column_major`, `unresolved_choice_non_numeric_input_errors` | PASS (non-normative) |
| (registration/type metadata) | SC-017-2 | `nonzeros_type_is_column_vector` | PASS |

## Observed cases — actual outputs

All four observed cases from the export reproduce exactly under RunMat's
value model (`Value::Num` ≡ 1×1 double; `Value::Tensor` shape `[n, 1]` ≡
n×1 double):

| Export case | Input (RunMat) | Expected (export) | Actual (test-asserted) |
|---|---|---|---|
| `row` | `Tensor [0,1,0,2]`, shape `[1,4]` | `[1;2]`, double, `[2,1]` | `Tensor data [1,2]`, shape `[2,1]` — PASS |
| `matrix` | `Tensor [1,0,2,4]`, shape `[2,2]` (col-major `[1 2;0 4]`) | `[1;2;4]`, double, `[3,1]` | `Tensor data [1,2,4]`, shape `[3,1]` — PASS |
| `col-major` | `Tensor [1,3,2,0]`, shape `[2,2]` (col-major `[1 2;3 0]`) | `[1;3;2]`, double, `[3,1]` | `Tensor data [1,3,2]`, shape `[3,1]` — PASS |
| `sparse` | `SparseTensor 1×3`, `col_ptrs [0,0,1,1]`, `values [3]` | `3`, double, `[1,1]`, issparse=false | `Value::Num(3.0)` — PASS |

## Deviations and notes

- No deviations from the approved export. The order-discriminating case
  (`col-major`) asserts the exact `[1;3;2]` sequence, ruling out a row-major
  implementation.
- The workspace-wide `cargo fmt --all -- --check` was run while five sibling
  features were being implemented concurrently in the same tree; failures it
  reported were exclusively in sibling-owned files. This feature's two files
  pass `rustfmt --check` individually.
- Sparse path reads `SparseTensor.values` directly (CSC invariants make it
  column-major); no densification occurs.
- Registration via the `#[runtime_builtin]` inventory macro
  (`builtin_path = "crate::builtins::math::sparse::nonzeros"`), same
  mechanism as the existing `math/sparse` builtins.

## Unresolved (returned to specification side)

The export carries no unresolved questions. Candidates for future
observation: all-zero input result shape (0×1), logical/integer/complex
input classes, NaN handling, N-D inputs, non-numeric input error identity
(see spec.md, Unresolved behaviour).
