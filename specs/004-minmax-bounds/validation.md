# Validation Report — 004-minmax-bounds — 2026-07-22

## Commands and results

- `cargo check -p runmat-runtime` — clean
- `cargo fmt --all -- --check` — clean
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::math::reduction::bounds` — **6 passed, 0 failed**

## Traceability

| Claim | FR | Test | Result |
|---|---|---|---|
| bounds.first-output-min, output-class (bounds([3 1 2]) → 1) | FR-004-03 | `observed_vector_first_output_is_min` | PASS |
| bounds.first-output-min (bounds([1 2;3 4]) → [1 2] 1×2) | FR-004-03 | `observed_matrix_default_first_output` | PASS |
| bounds.first-output-min (bounds([1 2;3 4],2) → [1;3] 2×1) | FR-004-01/03 | `observed_matrix_dim2_first_output` | PASS |
| (summary) hi = largest | FR-004-04 | `summary_derived_second_output_is_max` | PASS (summary-derived) |
| no option arguments | FR-004-05 | `too_many_arguments_error` | PASS |
| NaN inherited from kernels | FR-004-05 | `unresolved_choice_nan_inherited_from_min_kernel` | PASS (non-normative) |

All three observed cases (exact recorded call inputs) reproduce exactly.

## Deviations and notes

- Implementation delegates wholly to the registered `min`/`max` builtins via
  the runtime dispatcher (`call_builtin_async`) — no new reduction algorithm
  (Principles III/IX); this also exercises registry name-dispatch end-to-end.
- No visibility changes to `min.rs`/`max.rs` were needed (the risk flagged in
  the plan did not materialise).
- `'omitnan'`/`'includenan'` deliberately rejected (unresolved bounds.q-nan).

## Unresolved (returned to specification side)

`bounds.q-second-output` (observation for hi), `bounds.q-nan`.
