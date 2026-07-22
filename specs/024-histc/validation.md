# Validation Report — 024-histc — 2026-07-22

## Commands and results

- `cargo check -p runmat-runtime` — clean (Finished dev profile in 4m 46s)
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::stats::hist::histc::`
  — **14 passed, 0 failed, 0 ignored** (0.00s)
- `cargo fmt` then `cargo fmt --all -- --check` — clean (exit 0)

### Test list (all PASS)

```
observed_basic_counts_and_last_bin
observed_output_class_is_double
unresolved_choice_column_input_keeps_orientation
unresolved_choice_below_first_edge_dropped
unresolved_choice_above_last_edge_dropped
unresolved_choice_nan_dropped
unresolved_choice_matrix_columnwise_default
unresolved_choice_dim_argument
unresolved_choice_second_output_bin_indices
unresolved_choice_non_monotonic_edges_error
missing_edges_errors
descriptor_signature_covers_primary_form
histc_type_uses_edge_length_row
gpu_roundtrip_matches_host
```

## Actual outputs verified by tests

| Call | Result (value / shape) |
|---|---|
| `histc([1 2 2 3], [1 2 3])` (observed) | `[1 2 1]`, `1x3` double |
| `histc([1;2;2;3], [1;2;3])` | `[1;2;1]`, `3x1` |
| `histc([0 1 2 2 3], [1 2 3])` | `[1 2 1]` (0 dropped) |
| `histc([1 2 2 3 5], [1 2 3])` | `[1 2 1]` (5 dropped) |
| `histc([1 NaN 2 2 3], [1 2 3])` | `[1 2 1]` (NaN dropped) |
| `histc([1 2;2 3], [1 2 3])` | `[1 1 0; 0 1 1]`ᵀ col-major → `3x2` |
| `histc([1 2;2 3], [1 2 3], 2)` | `2x3`, rows `[1 1 0]`,`[0 1 1]` |
| `[n,idx]=histc([0 1 2 2 3],[1 2 3])` | `n=[1 2 1]`, `idx=[0 1 2 2 3]` |
| `histc([1 2], [3 2 1])` | error `RunMat:histc:EdgesNotMonotonic` |

## Traceability (claim → FR → test → result)

| Claim | FR | Test | Result |
|---|---|---|---|
| histc.signature-primary, histc.output-class, histc.bin-counts | FR-024-01, FR-024-02, FR-024-03 | `observed_basic_counts_and_last_bin` | PASS |
| histc.output-class | FR-024-01 | `observed_output_class_is_double` | PASS |
| (documented choice) | FR-024-04 | `unresolved_choice_column_input_keeps_orientation`, `unresolved_choice_below_first_edge_dropped`, `unresolved_choice_above_last_edge_dropped`, `unresolved_choice_nan_dropped` | PASS (non-normative) |
| (documented choice) | FR-024-05 | `unresolved_choice_matrix_columnwise_default`, `unresolved_choice_dim_argument`, `histc_type_uses_edge_length_row` | PASS (non-normative) |
| (documented choice) | FR-024-06 | `unresolved_choice_second_output_bin_indices` | PASS (non-normative) |
| (documented choice) | FR-024-07 | `unresolved_choice_non_monotonic_edges_error`, `missing_edges_errors` | PASS (non-normative) |
| registration/GPU | SC-024-2 | `descriptor_signature_covers_primary_form`, `gpu_roundtrip_matches_host` | PASS |

The single approved observed case (`histc([1 2 2 3], [1 2 3]) → [1 2 1]`,
class double, size `1x3`, final bin counting values equal to the last edge)
reproduces exactly. SC-024-1, SC-024-2, SC-024-3 met.

## Deviations and notes

- The generalised bin rule (bin `k` = `[edges(k), edges(k+1))`, final bin =
  values `== edges(end)`) is implemented via
  `edges.partition_point(|e| *e <= v) - 1`; the `<=` predicate makes
  `v == edges(end)` land in the final bin, matching the one observed case. The
  rule's application to values other than those in the observation is an
  independent inference, tested under `unresolved_choice_*` without conformance
  claims.
- Column/matrix orientation, the `dim` argument, the second (bin-index)
  output, out-of-range/NaN handling, and edge-monotonicity errors are
  documented independent choices (FR-024-04..07); none is asserted as
  MATLAB-conformant.
- Registration is the compile-time `#[runtime_builtin]` inventory macro (same
  mechanism as existing builtins); `register_gpu_spec`/`register_fusion_spec`
  describe `histc` as a host-materialising sink. GPU input is gathered to host
  before binning (`gpu_roundtrip_matches_host`).
- No MATLAB was invoked. No commits/pushes were made.

## Unresolved (returned to specification side)

- `histc.q-deprecated` — deprecation notice relative to `histcounts`
  (unconfirmed; no warning emitted).
- Confirmation of the documented choices in FR-024-04..07 (orientation, `dim`,
  second output, out-of-range/NaN, edge errors) requires additional approved
  observations.
