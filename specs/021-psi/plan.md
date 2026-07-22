# Implementation Plan — 021-psi

## Constitution Check

- I Specification-first: requirements cite approved claims (`psi` 0.2.0) — PASS
- II Clean-room: sources = imported export + RunMat code + public-domain
  mathematics; no MATLAB invocation anywhere — PASS
- III Independence: algorithm is the classical recurrence + Bernoulli
  asymptotic series from Abramowitz & Stegun (public domain); no MATLAB
  internals inferred — PASS
- IV Provenance: `provenance.md` complete with commit pin
  `e46587ae1b78237e0f43cd55f678de01a28495a4` and Principle X record — PASS
- V Test-first: observed/unresolved tests authored before the builtin body
  (T2 before T3) — PASS
- VI Traceability: claim → FR → test table in `validation.md` — PASS
- VII Minimal scope: one builtin, single-argument form only; polygamma
  rejected pending spec; no refactoring — PASS
- VIII Human gates: Gates 1–3 approved for the Tier B batch; publication
  (commit/push/PR) remains separately gated — PASS
- IX Compatibility: mirrors `math/elementwise/gamma.rs` (descriptor, GPU and
  fusion specs, macro registration, inline tests); `cargo fmt` clean — PASS
- X Licensing: external source is public-domain (NBS AMS 55); recorded in
  `provenance.md` — PASS

## Affected crates and files

| Item | Path |
|---|---|
| `psi` builtin | `crates/runmat-runtime/src/builtins/math/elementwise/psi.rs` (new) |
| Registration | add `pub(crate) mod psi;` to `crates/runmat-runtime/src/builtins/math/elementwise/mod.rs` |
| Docs metadata | descriptor + `#[runtime_builtin]` doc fields (feeds `docs/builtins` tooling) |

Exemplar: `math/elementwise/gamma.rs` (elementwise structure, descriptor,
GPU/fusion specs, error builders, inline test module, tensor mapping).

## Design decisions

- `digamma(x: f64) -> f64`: while `x < 10`, accumulate `-1/x` and shift
  `x += 1` (A&S 6.3.5); then apply `ln x − 1/(2x)` minus the Bernoulli
  series through the `B14` term (A&S 6.3.18), giving ~1 ulp accuracy at the
  observed points (verified ≤1.2e-16 against the export values; requirement
  is 1e-12).
- Poles (`x ≤ 0` with `x == floor(x)`) → `NaN`; `NaN` → `NaN`; `+Inf` →
  `+Inf`; `-Inf` → `NaN` (documented choices; `psi.q-poles` unresolved).
- Negative non-integers evaluate through the same recurrence (well-defined;
  documented choice, unobserved).
- Value handling mirrors `gamma`: `Num`/`Int`/`Bool` → double scalar;
  `Tensor` maps element-wise preserving shape; `LogicalArray`/`CharArray`
  promote to double tensors; `String`/`StringArray` and complex values
  error with `RunMat:psi:InvalidInput`.
- GPU: no `unary_psi` provider hook exists in `runmat-accelerate-api`, so
  `Value::GpuTensor` gathers to host and evaluates on the CPU
  (`provider_hooks: &[]`, residency `GatherImmediately`); no accel macro tag
  (per `docs/builtins/authoring.md`, tags only for real device paths).
  The accelerate API is NOT modified (minimal scope).
- Two-argument calls error with `RunMat:psi:PolygammaUnsupported`
  (psi.q-polygamma pending); three or more arguments error with
  `RunMat:psi:InvalidArgument`.
- Output mode `Fixed`; type resolver `numeric_unary_type` (same as `gamma`).

## Test plan (inline `#[cfg(test)]`, per house convention)

Normative `observed_*` (tolerance 1e-12; cite claims + FRs; release R2026a):
- scalar case: `psi(1)` → 1×1 double `-0.57721566490153231`.
- vector case: `psi([1 2 3])` → 1×3 double
  `[-0.57721566490153231 0.42278433509846747 0.92278433509846747]`.

Unresolved-tracking `unresolved_choice_*`: poles → NaN; negative
non-integer finite + recurrence identity; NaN/Inf propagation; int/logical
promotion; char promotion; complex rejected; string rejected; polygamma
form rejected; >2 args rejected.

Algorithm verification (public mathematics, not MATLAB claims): ψ(1/2) =
−γ − 2 ln 2; forward recurrence ψ(x+1) − ψ(x) = 1/x over a sweep.

Structural: descriptor signature coverage; type-resolver shape tests; GPU
gather path via `test_support::with_test_provider` matching observed values.

## Risks & rollback

- Risk: five sibling features edit `math/elementwise/mod.rs` concurrently —
  single-line insertion; re-read and re-apply on conflict; compile errors in
  other files are waited out, not edited.
- Risk: asymptotic-series accuracy — mitigated by shift threshold 10 and
  terms through B14 (validated numerically in `validation.md`).
- Rollback: delete `psi.rs` and the one `mod.rs` line (no other file is
  touched).

## Verification commands

`cargo check -p runmat-runtime` ·
`RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::math::elementwise::psi` ·
`cargo fmt` then `cargo fmt --all -- --check`
