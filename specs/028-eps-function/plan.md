# Implementation Plan — 028-eps-function

## Constitution Check

- I Specification-first: FR-028-01/02 cite approved claims; all other forms
  are marked documented-choice against `eps.q-value-form` — PASS
- II Clean-room: sources = imported export + RunMat code only; no MATLAB
  invoked; `eps(x)` uses the public IEEE `nextUp` technique — PASS
- III Independence: spacing derived from IEEE arithmetic + RunMat `Value`
  model, not a reconstructed MATLAB internal — PASS
- IV Provenance: `provenance.md` complete; revision pinned to source commit
  `1b01912` — PASS
- V Test-first: acceptance criteria fixed before implementation; observed vs
  documented-choice tests separated — PASS
- VII Minimal scope: one builtin + the single constant removal it requires; no
  unrelated refactoring — PASS
- IX Compatibility: follows the `math/elementwise/` builtin conventions
  (descriptor, GPU/fusion spec, `#[runtime_builtin]`, `Value` model) — PASS

## Constant → function migration (binding gate decision)

RunMat registered `eps` as a constant in
`crates/runmat-runtime/src/builtins/constants/mod.rs`:
`register_constant!("eps", Value::Num(f64::EPSILON), …)`. A constant cannot
express `eps('single')` or `eps(x)`. The plan:

1. Remove **only** that one `register_constant!("eps", …)` entry (leave `pi`,
   `sqrt2`, `i`, `j`, `inf`/`Inf`, `nan`/`NaN`, `true`, `false` intact); leave
   a comment pointing to the new function.
2. Add `#[runtime_builtin(name = "eps", category = "math/elementwise", …)]` in
   `crates/runmat-runtime/src/builtins/math/elementwise/eps.rs`, registered via
   `pub(crate) mod eps;` in `math/elementwise/mod.rs`.

### Bare-`eps` resolution — key risk and verification result

Risk: with `eps` no longer a constant, bare `eps` (no parentheses) must still
evaluate. Investigation of `crates/runmat-hir/src/lowering/ctx.rs`
(`lower_expr_semantic`, `AstExpr::Ident` arm) shows the resolution order for a
bare identifier is:

1. local variable binding, else
2. external/workspace binding that is a constant, else
3. registered **constant** (`runmat_builtins::constants()`), else
4. imported static property, else
5. **builtin call** when `is_builtin(name)` is true — builds a `Call` with
   empty args and `bare_identifier = true` (the same path `pwd`, `tic`, … use),
   else
6. `undefined variable` error.

`is_builtin` consults `runmat_builtins::builtin_functions()`. So once `eps` is
a builtin and no longer a constant, bare `eps` lowers to a zero-argument
`eps()` call. **Verified two ways:**

- Registry test (`eps_is_registered_as_builtin_and_not_constant`):
  `builtin_function_by_name("eps").is_some()` and `constants()` contains no
  `eps`, while `pi/inf/Inf/nan/NaN/i/j` remain.
- End-to-end (temporary `runmat-core` integration test, run then removed):
  `d = eps` → `Num(f64::EPSILON)`; `eps(2)` → `2^-51`;
  `class(eps('single'))` → `"single"`; `eps('single')` →
  `Tensor{ data:[1.1920928955078125e-7], shape:[1,1], dtype:F32 }`.

Result: **bare `eps` resolves correctly — not a blocker.**

## Affected crates and files

| Item | Path |
|---|---|
| `eps` builtin | `crates/runmat-runtime/src/builtins/math/elementwise/eps.rs` (new) |
| Registration | `pub(crate) mod eps;` in `math/elementwise/mod.rs` |
| Constant removal | `crates/runmat-runtime/src/builtins/constants/mod.rs` (delete the `eps` entry only; add explanatory comment) |

Exemplars: `math/elementwise/gamma.rs` and `single.rs` (elementwise builtin
structure, descriptor/spec, single-class representation); `io/repl_fs/pwd.rs`
(zero-arg `Vec<Value>` signature); `logical/tests/iscell.rs` (metadata-style,
no GPU provider hook).

## Design decisions

- Signature `async fn eps_builtin(args: Vec<Value>)` — fully variadic so the
  zero-arg (bare), one-value, and one-classname forms all dispatch. (A
  `(Value, Vec<Value>)` signature would reject the zero-arg bare form.)
- Dispatch: 0 args → `Num(f64::EPSILON)`; 1 arg with `keyword_of(...)` present
  → class-name path; else numeric-value path; >1 arg → `TooManyInputs`.
- `eps('single')` → 1×1 `NumericDType::F32` tensor built with `Tensor::from_f32`
  and returned as `Value::Tensor` **without** collapsing to `Num` (collapse or
  `Num` would report class `double`).
- `eps(x)` spacing: `nextUp(|x|) − |x|`, computed in the input's precision
  (`f64` for double, `f32` widened for single). Rust 1.90 provides
  `f64::next_up` / `f32::next_up` (stable since 1.86).
- Arrays preserve shape; `F64` results may collapse a 1-element tensor to `Num`
  (double convention), `F32` results always stay `Value::Tensor` to keep the
  single class.
- Complex input → spacing at `|x|` (real result). Integer/logical/gpuArray →
  `InvalidInput`. Unknown class name → `InvalidClass`.
- `type_resolver` `eps_type`: `Type::Num` for zero args (matches the old
  constant's scalar type); otherwise delegate to `numeric_unary_type`.
- No GPU/fusion kernel: `GPU_SPEC`/`FUSION_SPEC` registered as
  `Custom("metadata")` with no provider hooks (host-only), `accel = "metadata"`.

## Test plan (inline `#[cfg(test)]`, house convention)

- `observed_*` (normative, cite claims): bare `eps` exact double; `eps('single')`
  exact single value + class.
- Registry/resolution: `eps` is a builtin and not a constant (bare-`eps`
  evidence).
- `unresolved_choice_*` (documented, non-normative): `eps('double')`,
  case-insensitive class name, `eps(1)`/`eps(2)` ulp, negative, `eps(0)`
  subnormal, `eps(Inf)/eps(NaN)` → NaN, double array shape, single array class,
  complex magnitude, complex tensor, invalid class error, too-many-args error,
  integer-input error.
- Descriptor: three signature labels present.

## Risks & rollback

- Risk: single scalar representation — mitigated by returning a 1×1 F32 tensor
  and asserting `class_name_for_value == "single"` in tests.
- Risk: WASM builtin registry becomes stale (fingerprint changes when a source
  file is added). Native `cargo check`/tests skip that validation
  (`build.rs` returns early when `target_arch != wasm32`); WASM builds require
  `scripts/regenerate-wasm-registry.sh`, deferred to the batch-level
  regeneration (five sibling features add builtins concurrently). Noted in
  `validation.md`.
- Rollback: delete `eps.rs` + its `mod` line and restore the
  `register_constant!("eps", …)` entry.

## Verification commands

`cargo check -p runmat-runtime` ·
`RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::math::elementwise::eps` ·
`RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- eps` (swap safety) ·
`cargo fmt --all -- --check` · `cargo clippy -p runmat-runtime --lib`
