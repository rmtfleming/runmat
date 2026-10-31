# Implementation Plan — 061-sparse-linalg-n5

## Constitution Check

- I Specification-first: FR-061-01..07, 09..11, 18..25 and 29..35 cite the
  27 approved normative claims; FR-061-08, 36, 37 record the three
  non-normative claims; the remaining FRs are tagged documented choices or
  unresolved. PASS.
- II Clean-room: the only inputs are the imported exports and RunMat code.
  MATLAB was not invoked. PASS.
- III Independence: SVD minimum-norm solution, Householder QR with column
  pivoting, Hopcroft–Karp matching, RunMat's own random stream; all
  textbook or existing RunMat code. PASS.
- IV Provenance: `provenance.md` pins `1c91aa1`, freshness-checked, 30
  claim ids verified. PASS.
- V Test-first: tests are written with stub bodies (`return Err(stub)`),
  the failing run is recorded in `validation.md`, then the stubs are
  removed. PASS.
- VII Minimal scope: three new files plus one module line in each of three
  hand-written parents. No generated parent and no upstream code changed;
  the `sprand` helpers are reused from a child module without changing
  their visibility. PASS.
- IX Compatibility: `#[runtime_builtin]` with descriptors and
  `accel = "cpu"`; inputs gathered (`gather_if_needed_async`,
  `gpu_helpers::gather_value_async`); results built with `Tensor`,
  `ComplexTensor`, `SparseTensor`; nalgebra for the SVD; random values
  through `builtins::common::random`. No integer-capability metadata.
  PASS.

## Placement

| Builtin | File | Parent |
|---|---|---|
| `lsqminnorm` | `crates/runmat-runtime/src/builtins/math/linalg/solve/lsqminnorm.rs` | `math/linalg/solve/mod.rs` (hand-written) next to `pinv`, `rank`, `null` |
| `sprank` | `crates/runmat-runtime/src/builtins/math/sparse/sprank.rs` | `math/sparse/mod.rs` (hand-written) next to `amd`, `etree` |
| `sprandn` | `crates/runmat-runtime/src/builtins/array/creation/sparse/sprandn.rs` | `array/creation/sparse/mod.rs` (hand-written) next to `sprand` |

## Design

### lsqminnorm
1. At most one output; fewer than two inputs → `RunMat:narginchk:notEnoughInputs`;
   more than three → `UnsupportedOption`. Gather.
2. `A`, `b`: `Num`, `Complex`, F64/F32 `Tensor`/`ComplexTensor`, sparse
   real/complex (densified, D-061-4). Char or integer `A` → `InvalidA`;
   char/integer `b`, logical, other classes, NaN/Inf → `UnsupportedOption`;
   N-D → `InvalidInput`.
3. Row mismatch → `RunMat:decomposition:mldivide`. `tol`: finite
   non-negative real scalar.
4. Empty `A` or `b` → `zeros(n, p)`. Otherwise real (`f64`) or complex
   (`Complex64`) path, generic over nalgebra `ComplexField`:
   default → SVD minimum-norm solution (D-061-2); `tol` → pivoted QR
   truncated at `|R(k,k)| > tol`, minimum-norm solve of `R(1:r,:)`,
   un-permute (D-061-3).
5. Output: full `Tensor`/`ComplexTensor`, single if either input single.

### sprank
1. At most one output; no input → `RunMat:minrhs`; extra input →
   `UnsupportedOption`. Gather.
2. Pattern as per-column row lists: sparse (stored entries), dense
   numeric/complex/logical (nonzeros, NaN counts), scalars. Char/cell →
   `RunMat:math:mustBeFloatOrLogical`; integer, string, others →
   `UnsupportedOption`; N-D → `InvalidInput`.
3. Hopcroft–Karp maximum matching (BFS layers, iterative DFS). Result
   `Value::Num`.

### sprandn
1. At most one output. Gather. Arity: 0 → `InvalidInput`; 1 → pattern;
   2 → `TwoInputs`; 3/4 → sizes, density, `rc`; more → `UnsupportedOption`.
2. Sizes: real double scalars; non-integer/negative/non-finite →
   `RunMat:RandStream:randi:invalidLimits`. Density: real double scalar,
   clamped to [0, 1]; NaN → `UnsupportedOption`.
3. Density form: `floor(density*m*n)` unique positions
   (`sample_unique_positions`) and `generate_normal` values →
   `sparse_from_entries`. Pattern form: `sparse_pattern_from_value` plus
   normal values. `rc` form: scalar `rc` in [0, 1] →
   `scalar_rcond_singular_profile` + `sprand_from_condition_profile`.

## Test plan

Inline `#[cfg(test)]` modules `lsqminnorm_tests`, `sprank_tests`,
`sprandn_tests`, calling through `crate::call_builtin_async` (output-count
tests call the builtin function directly under `push_output_count`).

- Each observed case has one `observed_*` test: 17 + 14 + 11.
- `lsqminnorm` values are compared with the observation and with RunMat's
  own `pinv(full(A))*b` to 1e-12 (1e-6 single). The COBRA case builds `S'`
  with RunMat's `ctranspose` on a sparse `S`.
- `sprandn` tests seed with RunMat's `rng(7)` builtin under
  `random::test_guard()`, and assert structure, class, bound, sign mix and
  repeatability only; the `rc` case computes the condition number with
  nalgebra's singular values.
- `documented_choice_*` and `unresolved_*` tests cover the remaining FRs.

## Risks & rollback

- `lsqminnorm` densifies sparse input (D-061-4); large COBRA models may be
  slow or memory-bound.
- `sprandn` depends on private helpers of `array/creation/sparse/mod.rs`;
  if they are renamed upstream the child module must follow.
- Rollback: delete the three files and their module lines.

## Verification commands

- `cargo test -p runmat-runtime --lib -- lsqminnorm_tests sprank_tests sprandn_tests`
- `cargo test -p runmat-runtime --lib -- builtins::math::linalg builtins::array::creation::sparse builtins::math::sparse` (regressions)
- `rustfmt --edition 2021 --check <files>`
- `cargo clippy -p runmat-runtime --lib --tests`, filtered to the three files
