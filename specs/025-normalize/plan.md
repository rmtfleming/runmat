# Implementation Plan — 025-normalize

## Constitution Check

- I Specification-first: FRs cite approved claims / unresolved questions — PASS
- II Clean-room: sources = imported export + RunMat code only; no MATLAB — PASS
- III Independence: computed from standard mean/std/min/max/norm definitions
  and RunMat's own tensor machinery; no MATLAB-internal imitation — PASS
- IV Provenance: `provenance.md` complete; revision pinned to commit `4445294` — PASS
- V Test-first: acceptance criteria and observed/unresolved tests defined
  before/with the implementation body — PASS
- VII Minimal scope: one builtin, one new file + one `mod.rs` line — PASS
- IX Compatibility: mirrors `math/reduction` siblings (`mean`, `std`,
  `bounds`): descriptor, GPU/fusion specs, `#[runtime_builtin]` macro — PASS
- X Licensing: no MathWorks source/docs used — PASS

## Affected crates and files

| Item | Path |
|---|---|
| `normalize` builtin | `crates/runmat-runtime/src/builtins/math/reduction/normalize.rs` (new) |
| Registration | `crates/runmat-runtime/src/builtins/math/reduction/mod.rs` (add `pub(crate) mod normalize;`) |
| Docs metadata | descriptor + `#[runtime_builtin]` doc fields (feeds `docs/builtins` tooling) |

Exemplars: `math/reduction/mean.rs` and `std.rs` (reduction approach, sample
std with N−1 denominator, host/GPU gather, output-class handling);
`math/reduction/bounds.rs` (compact composite-reduction structure: descriptor,
GPU/fusion spec, error builders, inline `#[cfg(test)]` module).

## Design decisions

- Single async builtin `normalize(A, rest)`. Input is gathered to a real host
  double `Tensor` (`gpu_helpers::gather_tensor_async` for GPU;
  `tensor::value_into_tensor_for` otherwise). Complex → `RM.NORMALIZE.UNSUPPORTED`.
- Argument grammar `[dim,] method[, methodtype]`: a leading numeric scalar is
  the operating dimension; a string is the method; a trailing value qualifies
  the method (range interval `[a b]`, norm order `p`, center/scale type).
- Operating dimension: explicit `dim`, else first non-singleton dimension.
  Reduction/broadcast done in one pass over lines along the operating
  dimension using column-major strides (same layout math as `mean`/`std`).
- Methods:
  - `zscore` (default): `(x − mean) / std_sample` (N−1). **Observed.**
  - `range` `[a b]` (default `[0 1]`): `a + (x − min)/(max − min)·(b − a)`.
    **Observed** for `[0 1]`; interval unobserved.
  - `center` (mean | numeric), `scale` (std | first | numeric), `norm`
    (p, default 2, `Inf`): documented choices (normalize.q-methods).
  - `robust`/`medianiqr`/`mad`/`iqr`, `zscore,robust`: rejected (unresolved).
- Output is `tensor::tensor_into_value` (scalar → `Value::Num`), class double.
- Output mode `Fixed`; no `type_resolver` (same-shape, mirrors `bounds`).

## Test plan (inline `#[cfg(test)]`, per house convention)

- Normative (`observed_*`, cite claims): z-score vector, range vector, output
  class double.
- Documented choices (`unresolved_choice_*`, cite normalize.q-dim /
  normalize.q-methods): matrix column-wise default, explicit `dim` row-wise,
  explicit `zscore`, custom range interval, `center`, `scale`, `norm` orders,
  integer-input dim argument.
- Errors: `medianiqr`/`zscore,robust`/complex → `RM.NORMALIZE.UNSUPPORTED`;
  unknown method → `RM.NORMALIZE.INVALID_ARGUMENT`; descriptor error codes
  stable.

## Risks & rollback

- Risk: a sibling in the same batch introduces an unrelated compile error —
  re-run `cargo check` rather than editing other files.
- Risk: sample-vs-population std ambiguity — resolved by the observed
  `[1 2 3] → [-1 0 1]` (variance `2/2`), forcing N−1; asserted in a normative
  test.
- Rollback: delete `normalize.rs` and the single `mod.rs` line (no existing
  file otherwise modified).

## Verification commands

`cargo check -p runmat-runtime` ·
`RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::math::reduction::normalize` ·
`cargo fmt --all -- --check`
