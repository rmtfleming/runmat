# Implementation Plan — 055-numeric-n3n4

## Constitution Check

- I Specification-first: FR-055-01..16 and 18..25 cite the 26 approved
  normative claims; FR-055-17 records the two non-normative claims;
  FR-055-26..33 are tagged documented choices or unresolved. PASS.
- II Clean-room: the only inputs are the imported exports and RunMat code.
  MATLAB was not invoked. PASS.
- III Independence: `1/cos`, thin SVD with a rank threshold, LU inverse with
  1-norms; all textbook. PASS.
- IV Provenance: `provenance.md` pins `1156c97` (sec, orth) and `fc9f804`
  (condest), freshness-checked, 28 claim ids verified. PASS.
- V Test-first: tests are written with stub bodies (`return Err(stub)`), the
  failing run is recorded in `validation.md`, then the stubs are removed.
  PASS.
- VII Minimal scope: three new files plus one module line in each of three
  parents. `math/linalg/factor/mod.rs` and `math/linalg/solve/mod.rs` are
  hand-written; `math/trigonometry/mod.rs` is generated and gets one
  hand-added `pub(crate) mod sec;` line (D-055-2, reported). No upstream
  code changed. PASS.
- IX Compatibility: `#[runtime_builtin]` with descriptors and
  `accel = "cpu"`; inputs gathered with `gather_if_needed_async`; tensors
  read through `numeric_dtype()`/`materialize_f64()`, results built with
  `Tensor::new`/`from_f32` and `ComplexTensor::new`/`from_f32`; nalgebra
  for SVD/LU as in upstream `null`/`cond`. No integer-capability metadata.
  PASS.

## Placement

| Builtin | File | Parent |
|---|---|---|
| `sec` | `crates/runmat-runtime/src/builtins/math/trigonometry/sec.rs` | `math/trigonometry/mod.rs` (generated; one hand-added line) next to `cos`, `tan` |
| `orth` | `crates/runmat-runtime/src/builtins/math/linalg/factor/orth.rs` | `math/linalg/factor/mod.rs` (hand-written) next to `svd`, `svds` |
| `condest` | `crates/runmat-runtime/src/builtins/math/linalg/solve/condest.rs` | `math/linalg/solve/mod.rs` (hand-written) next to `cond`, `rcond` |

## Design

### sec
1. At most one output; no input → `RunMat:minrhs`; more than one →
   `InvalidInput`. Gather.
2. `Num`/F64 tensor → `1/cos(x)` elementwise, same shape. F32 tensor →
   computed in double, stored as single. `Complex`/complex F64/F32 tensor →
   `1/cos(z)` with `num_complex`, complex result of the same precision.
3. Logical, integer (scalar, tensor, complex-integer), char →
   `RunMat:math:mustBeFloat`; anything else → `UnsupportedOption`.

### orth
1. At most one output; one or two inputs. Gather.
2. `A`: double/single, real/complex dense, 2-D. Sparse →
   `RunMat:svd:noSVDsparse`; integer/char → `RunMat:svd:inputType`;
   logical/other or non-finite entries → `UnsupportedOption`.
3. Empty `A` → `size(A,1)`×0. Otherwise thin SVD (nalgebra, `Complex64`
   for both real and complex), sort singular values descending, keep those
   above `t` or above `max(m,n)·eps(class)·σmax`; `Q = U(:, kept)`.
4. Output: real `Tensor` (F64/F32) for real input, `ComplexTensor`
   (F64/F32) for complex input.

### condest
1. At most two outputs; exactly one input (extra → `UnsupportedOption`).
   Gather.
2. `A`: `Num`, F64 dense tensor or real F64 sparse (densified). Char →
   `RunMat:lu:inputType` (checked before shape). Other classes and
   non-finite entries → `UnsupportedOption`. Non-square →
   `RunMat:condest:NonSquareMatrix`.
3. `n = 0` → `c = 0`, `v` 0×1. LU inverse: if singular → `c = Inf`, `v` from
   the SVD; else `c = norm(A,1)·norm(inv(A),1)` and `v` the normalised
   largest column of `inv(A)`.
4. Outputs: `c` as `Value::Num`; `[c, v]` as an `OutputList`.

## Test plan

Inline `#[cfg(test)]` modules `sec_tests`, `orth_tests`, `condest_tests`,
calling through `crate::call_builtin_async` (output-count tests call the
builtin function directly under `push_output_count`).

- Each observed case has one `observed_*` test: 15 + 15 + 12.
- `orth` tests compute `norm(Q'*Q - I)` and `norm(A - Q*(Q'*A))` (Frobenius,
  an upper bound for the 2-norm) and never compare basis entries.
- `condest` `vs-cond` tests compare with RunMat's own `cond(A, 1)`.
- `documented_choice_*` and `unresolved_*` tests cover FR-055-26..33.

## Risks & rollback

- The generated `math/trigonometry/mod.rs` may be regenerated upstream;
  the hand-added line must then be re-added.
- `orth` rank decisions near the threshold are a documented choice; only
  clearly separated singular values are tested.
- Rollback: delete the three files and their module lines.

## Verification commands

- `cargo test -p runmat-runtime --lib -- sec_tests orth_tests condest_tests`
- `cargo test -p runmat-runtime --lib -- builtins::math::trigonometry builtins::math::linalg` (regressions)
- `rustfmt --edition 2021 --check <files>`
- `cargo clippy -p runmat-runtime --lib --tests`, filtered to the three files
