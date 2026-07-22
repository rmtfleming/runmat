# Implementation Plan — 024-histc

## Constitution Check

- I Specification-first: requirements cite approved claims `histc.*`; the
  generalised rule and unobserved surfaces are marked documented choices — PASS
- II Clean-room: sources = imported export + RunMat code (`histcounts`,
  `bounds`) only; no MATLAB invoked — PASS
- III Independence: binning derived from RunMat `Tensor`/`Value` and a
  `partition_point` search, not any MATLAB internal algorithm — PASS
- IV Provenance: `provenance.md` complete; commit pin `4445294` recorded — PASS
- V Test-first: `observed_*` and `unresolved_choice_*` tests authored with the
  builtin; every reported result comes from a real `cargo test` run — PASS
- VII Minimal scope: one builtin (`histc`), no unrelated refactoring — PASS
- IX Compatibility: mirrors `stats/hist/histcounts` (descriptor, GPU/fusion
  specs, `#[runtime_builtin]`, inline tests, `type_resolvers`) — PASS

## Affected crates and files

| Item | Path |
|---|---|
| `histc` builtin | `crates/runmat-runtime/src/builtins/stats/hist/histc.rs` (new) |
| Registration | add `pub mod histc;` to `crates/runmat-runtime/src/builtins/stats/hist/mod.rs` |
| Type resolver | add `histc_type` to `crates/runmat-runtime/src/builtins/stats/type_resolvers.rs` |

Exemplars: `stats/hist/histcounts.rs` (binning, GPU gather, descriptor,
output-count handling, inline tests) and `math/reduction/bounds.rs`
(structure, clean-room header, `observed_*`/`summary_derived`/
`unresolved_choice_*` test tiers).

## Design decisions

- **Bin rule** (`bin_of`): reject `NaN`; reject `v < edges[0]` and
  `v > edges[last]`; otherwise the bin index is
  `edges.partition_point(|e| *e <= v) - 1`. Because the predicate uses `<=`,
  `v == edges[last]` lands in the final bin — reproducing the observed
  last-edge behaviour — while `v` in `[edges[k], edges[k+1])` lands in bin `k`.
- **Operating dimension**: default = first non-singleton dimension (so the
  observed `1x4` row yields a `1x3` row); overridable by an optional 1-based
  `dim` third argument. The counts are computed independently along that
  dimension using column-major strides; the dimension's size becomes
  `length(edges)` and all other dimensions are preserved.
- **Second output** `idx`: same shape as the input; 1-based bin index per
  element, `0` if uncounted. Emitted only when `nargout >= 2`.
- **GPU**: a `Value::GpuTensor` input is gathered to host via
  `gpu_helpers::gather_tensor_async` (matching `histcounts`); `register_gpu_spec`
  and `register_fusion_spec` describe it as a host-materialising sink.
- **Errors**: non-monotonic or `NaN` edges → `RunMat:histc:InvalidArgument`;
  missing `edges` or too many args → same identifier.
- Output mode `ByRequestedOutputCount`; `type_resolver(histc_type)`.

## Test plan (inline `#[cfg(test)]`, house convention)

- `observed_*` (normative; cite `histc.signature-primary`,
  `histc.output-class`, `histc.bin-counts`): the exact observed case
  `histc([1 2 2 3], [1 2 3])` → `[1 2 1]`, `1x3`, `Value::Tensor` (double).
- `unresolved_choice_*` (non-normative documented choices, FR-024-04..07):
  column-vector orientation; value below first edge dropped; value above last
  edge dropped; `NaN` dropped; matrix column-wise; explicit `dim`; second
  output bin indices; non-monotonic-edges error.
- Infra: GPU round-trip via `test_support::with_test_provider`; descriptor
  labels; `histc_type` shape inference.

## Risks & rollback

- Risk: concurrent sibling features cause unrelated compile errors — per task
  guidance, wait and re-run rather than edit other files.
- Risk: N-D `dim` stride arithmetic — covered by explicit matrix/`dim` tests.
- Rollback: delete `histc.rs`, the `mod` line, and `histc_type` (no existing
  behaviour modified).

## Verification commands

`cargo check -p runmat-runtime` ·
`RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::stats::hist::histc` ·
`cargo fmt` then `cargo fmt --all -- --check`
