# Implementation Plan — 063-rat (spec 0.2.0)

## Constitution Check

- I Specification-first: FR-063-01..08 cite the 9 approved normative
  claims; the complex form is supported as observed by maintainer
  disposition (non-normative, D-063-3); U-063-1..6 are labelled documented
  choices; FR-063-10 carries the unresolved questions. PASS.
- II Clean-room: input is the imported export; MATLAB not invoked. PASS.
- III Independence: textbook nearest-integer continued fractions; default
  tolerance is RunMat's own documented choice. PASS.
- IV Provenance: `provenance.md` pinned to `1c91aa1`, freshness-checked,
  claim ids verified. PASS.
- V Test-first: tests written first against a registered stub; the failing
  run is recorded in `validation.md`. PASS.
- VII Minimal scope: one new file plus one `mod` line in the hand-written
  `math/rounding/mod.rs`. PASS.
- IX Compatibility: `#[runtime_builtin]` with an in-file descriptor,
  `accel = "cpu"`, gpu inputs gathered; no `integer_capabilities`. PASS.

## Affected files

| Item | Path |
|---|---|
| `rat` + tests (new) | `crates/runmat-runtime/src/builtins/math/rounding/rat.rs` |
| Registration | `pub(crate) mod rat;` in the hand-written `crates/runmat-runtime/src/builtins/math/rounding/mod.rs` |

## Design

- Input: `Num`, `Complex`, `Tensor`/`ComplexTensor` of `F64`/`F32` (widened
  to f64; integer storage → `norm:inputType`); `Int`, `CharArray` →
  `norm:inputType`; others → `UnsupportedOption`.
- `expand(x, tol)` per element (U-063-1/2): nearest-integer partial
  quotients with convergent recurrence and stopping rule; returns `N`, `D`,
  terms, and a special text for non-finite input.
- Outputs: scalar input → `Num`/`Complex`; array → tensor of the input
  shape and precision. Complex: per-part expansion over `lcm` denominator.
- Text: rows per element (three per complex element), right-padded,
  row-major `CharArray`.

## Test plan (inline `#[cfg(test)]` module `rat_tests`)

- `observed_*` / `rejects_*`: every normative case.
- `supported_complex_as_observed`: the two complex cases.
- `unresolved_choice_*`: U-063-1..6.

## Analysis (B3)

- Coverage: every normative claim maps to an FR and a test; every U-item to
  an `unresolved_choice_*` test.
- Consistency: the expansion was checked by hand (and by a scratch
  script, outside the repository) for every observed value: `0.75` →
  terms `[1, -4]` → `3/4` and text `1 + 1/(-4)`; `0.5` → `[1, -2]`, `1.5` →
  `[2, -2]` (both observed texts); `pi` → `[3, 7, 16]` → `355/113`, error
  `2.667641894049666e-07` (≤ `1e-6·pi`), while `22/7` (error `1.26e-3`) is
  rejected by the default and accepted by `tol = 1e-2` (observed
  `0.0012644892673496777`); `0.6667` → `[1, -3, -3333]` → `6667/10000`
  (the earlier convergent `2/3` has error `3.3e-5 > 6.7e-7`); `1/3` →
  `[0, 3]`; `-0.5` → `[-1, 2]` → `-1/2`; complex `1+2i` → `1/1`, `2/1` →
  `N = 1+2i`, `D = 1`, text rows of width 8.
- Ambiguity resolved by choice: default tolerance (U-063-2), text grammar
  beyond two terms (U-063-4), complex layout for arrays (U-063-4).
- No constitution conflicts; no CRITICAL findings.

## Risks & rollback

- The default tolerance is a choice; inputs whose nearest convergents lie
  close to `1e-6·|x|` may differ from other implementations
  (non-normative, documented).
- Rollback: delete `rat.rs` and its `mod` line.

## Verification commands

```
cargo test -p runmat-runtime --lib -- rat_tests
cargo test -p runmat-runtime --lib -- builtins::math::rounding
rustfmt --edition 2021 --check crates/runmat-runtime/src/builtins/math/rounding/rat.rs
cargo clippy -p runmat-runtime --lib --tests --message-format short
```
