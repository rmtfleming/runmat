# Implementation Plan — 067-numeric-n6

## Constitution Check

- I Specification-first: FR-067-01..10 and 20..33 cite the 24 approved
  normative claims; FR-067-11, 12, 34, 35 record the four non-normative
  claims; the remaining FRs are tagged documented choices or unresolved.
  PASS.
- II Clean-room: the only inputs are the imported exports and RunMat code.
  MATLAB was not invoked. PASS.
- III Independence: power method on `S'*S` and Lawson–Hanson NNLS, both
  textbook algorithms; nalgebra SVD. PASS.
- IV Provenance: `provenance.md` pins `e364206`, freshness-checked, 28 claim
  ids verified. PASS.
- V Test-first: tests written with a stub line at the top of each builtin
  body; the failing run is recorded in `validation.md`, then the stubs are
  removed. PASS.
- VII Minimal scope: two new files plus one module line in each of two
  hand-written parents. No upstream code changed. PASS.
- IX Compatibility: `#[runtime_builtin]` with descriptors and
  `accel = "cpu"`; inputs gathered (`gather_if_needed_async`); results built
  with `Tensor`, `StructValue`, `CharArray`. No integer-capability metadata.
  PASS.

## Placement

| Builtin | File | Parent |
|---|---|---|
| `normest` | `crates/runmat-runtime/src/builtins/math/linalg/solve/normest.rs` | `math/linalg/solve/mod.rs` (hand-written) |
| `lsqnonneg` | `crates/runmat-runtime/src/builtins/math/optim/lsqnonneg.rs` | `math/optim/mod.rs` (hand-written) |

## Design

### normest
1. At most two outputs; no input → `RunMat:minrhs`; more than two inputs →
   `UnsupportedOption`. Gather.
2. `S`: `Num`, `Complex`, F64/F32 `Tensor`/`ComplexTensor`, sparse real or
   complex (kept as CSC), char (codes). Integer → 
   `RunMat:mtimes:integerNotSupported`; N-D → `RunMat:transpose:NDArray`;
   logical/other → `UnsupportedOption`.
3. `tol`: positive finite real scalar, default `1e-8` (D-067-2).
4. Empty → 0; NaN → NaN; Inf → Inf; else power method with products `S*x`,
   `S'*y` over stored entries; zero start vector → 0, `cnt = 0`.
5. `nrm` double `Num` or 1×1 single `Tensor`; `cnt` `Num`.

### lsqnonneg
1. At most six outputs; fewer than two inputs → `InputArg`; more than three
   → `UnsupportedOption`. Gather.
2. `C`, `d`: real double, full or sparse (densified). Complex →
   `ComplexCorD`; single/char/integer/logical/other → `NonDoubleInput`;
   N-D → `InvalidInput`. `d` not a column → `RunMat:dimagree`; row mismatch
   → `RunMat:sizeDimensionsMustMatch`; NaN/Inf → `UnsupportedOption`.
3. Options: structure or `[]`; TolX/StepTolerance, Display, Solver; others
   non-empty → `UnsupportedOption`.
4. Lawson–Hanson NNLS with SVD subproblems; outer iteration cap `3n+10`.
5. Outputs: `x` (n×1), `resnorm`, `residual` (m×1), `exitflag`, `output`
   struct (`iterations`, `algorithm`, `message`), `lambda` (n×1), truncated
   to the requested count.

## Test plan

Inline `#[cfg(test)]` modules `normest_tests` and `lsqnonneg_tests`, calling
through `crate::call_builtin_async`; multi-output and output-count tests call
the builtin function directly under `push_output_count`.

- One `observed_*` test per observed case (20 normest, 18 lsqnonneg; the two
  tol cases share one test with `dense-square`, so 19 normest tests).
- normest values are compared with RunMat's own `norm` of the full matrix,
  within 2.3e-7 (the observed bound); the tol ordering is checked against
  the default run. `cnt` is only checked to be a non-negative integer.
- lsqnonneg checks values within 1e-14 where the observation fixes them, and
  the residual/lambda/resnorm relations and KKT conditions independently.
  The rank-deficient case asserts objective and KKT only.
- `documented_choice_*` and `unresolved_*` tests cover the remaining FRs.

## Risks & rollback

- normest convergence is linear in `(sigma2/sigma1)^2`; near-equal leading
  singular values need many steps (cap 10000).
- lsqnonneg densifies `C` (D-067-6).
- Rollback: delete the two files and their module lines.

## Verification commands

- `cargo test -p runmat-runtime --lib -- normest_tests lsqnonneg_tests`
- `cargo test -p runmat-runtime --lib -- builtins::math::linalg builtins::math::optim`
- `rustfmt --edition 2021 --check <files>`
- `cargo clippy -p runmat-runtime --lib --tests`, filtered to the two files
