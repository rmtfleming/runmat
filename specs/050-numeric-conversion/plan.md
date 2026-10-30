# Implementation Plan — 050-numeric-conversion

## Constitution Check

- I Specification-first: FR-050-01..20 cite the 27 approved claims.
  FR-050-21..30 are tagged as documented choices or unresolved. PASS.
- II Clean-room: the only inputs are the imported exports and RunMat code.
  MATLAB was not invoked. PASS.
- III Independence: a power with one Newton correction, `ln` after a domain
  check, and delegation to RunMat's own conversion builtins. PASS.
- IV Provenance: `provenance.md` is pinned to `8f72edb` and was
  freshness-checked. All 27 claim ids were verified. PASS.
- V Test-first: tests are written before the bodies, and the failing run is
  recorded in `validation.md`. PASS.
- VII Minimal scope: three new files, plus one `pub(crate) mod` line each in
  three **hand-written** parents (`math/elementwise/roots/mod.rs`,
  `math/elementwise/logarithms/mod.rs`,
  `math/elementwise/integer_conversions/mod.rs`; none carries the
  "@generated … Do not edit" header). The generated
  `math/elementwise/mod.rs` is not edited. PASS.
- IX Compatibility:
  - Registration uses `#[runtime_builtin]` with a descriptor and
    `accel = "cpu"`.
  - `nthroot`/`reallog` gather inputs with `gather_if_needed_async`;
    `cast` gathers only `p` and leaves `A` to the delegated builtin.
  - Tensor storage is read through `numeric_dtype()` and
    `materialize_f64()`; results are built with `Tensor::new` /
    `Tensor::from_f32`.
  - No integer-capability metadata is added. PASS.

## Placement

| Builtin | File | Parent (hand-written) |
|---|---|---|
| `nthroot` | `crates/runmat-runtime/src/builtins/math/elementwise/roots/nthroot.rs` | `roots/mod.rs` (next to `sqrt`, `realsqrt`) |
| `reallog` | `crates/runmat-runtime/src/builtins/math/elementwise/logarithms/reallog.rs` | `logarithms/mod.rs` (next to `log`) |
| `cast` | `crates/runmat-runtime/src/builtins/math/elementwise/integer_conversions/cast.rs` | `integer_conversions/mod.rs` (next to the `int8`…`uint64` builtins it delegates to) |

## Design

### nthroot
1. Arity 2, at most one output. Gather both inputs.
2. `x`: complex → `ComplexInput`; double/single scalar or dense tensor →
   `(values, shape, is_single)`; anything else → `UnsupportedOption`.
3. `n`: real double/single scalar or tensor, else `UnsupportedOption`.
4. Sizes: equal, or either side has one element → OK; broadcast-compatible
   but unequal → `UnsupportedOption`; else `RunMat:sizeDimensionsMustMatch`.
5. Each element: `x < 0` needs `n` finite, integral and odd, else
   `NegXNotOddIntegerN`. Root `r = |x|^(1/n)`; if `r` is finite and non-zero
   and `n` finite and non-zero, apply `r -= (r^n - |x|) / (n r^(n-1))` when
   the step is finite. Restore the sign for `x < 0`. `n = 1` returns `x`.
6. Output in the class of `x` (`Value::Num` for a double scalar).

### reallog
1. Arity 1, at most one output. Gather.
2. Complex → `mustBeReal`; integer class (scalar or tensor) → `mustBeFloat`;
   logical/char/sparse/other → `UnsupportedOption`.
3. Any element `< 0` → `complexResult`. Otherwise `ln` elementwise, same
   shape and class.

### cast
1. Arity 2 or 3, at most one output.
2. Target class:
   - 2 args: `newclass` text (char row / string scalar), else
     `UnsupportedOption`;
   - 3 args: second arg must be text equal to `like` (any case), else
     `UnsupportedOption`; gather `p`, take `class(p)`.
   - The name must be one of the 12 supported classes, else
     `UnsupportedClass`.
3. `A`: cell → `RunMat:invalidConversion`; numeric, logical, char, sparse,
   complex or gpuArray → allowed; anything else → `UnsupportedOption`.
4. Return `call_builtin_async(target, [A])`.

## Test plan

Inline `#[cfg(test)]` modules `nthroot_tests`, `reallog_tests` and
`cast_tests`, calling through `crate::call_builtin_async` (output-count
tests call the builtin function directly under `push_output_count`).

- Each observed case has one `observed_*` test: 14 + 12 + 16.
- `documented_choice_*` and `unresolved_*` tests cover FR-050-21..30.

## Risks & rollback

- No generated parent is touched.
- `cast` depends on the existing conversion builtins; any upstream change in
  their rounding or saturation shows up in the `observed_*` cast tests.
- Rollback: delete the three files and their `pub(crate) mod` lines.

## Verification commands

- `cargo test -p runmat-runtime --lib -- nthroot_tests reallog_tests cast_tests`
- `cargo test -p runmat-runtime --lib -- builtins::math::elementwise::roots builtins::math::elementwise::logarithms builtins::math::elementwise::integer_conversions` (regressions)
- `rustfmt --edition 2021 --check <files>`
- `cargo clippy -p runmat-runtime --lib --tests --message-format short`,
  filtered to the three files
