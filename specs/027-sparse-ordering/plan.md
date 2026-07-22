# Implementation Plan — 027-sparse-ordering

## Constitution Check

- I Specification-first: normative requirements cite approved claims
  (`colamd.*`, `dissect.*`); unresolved value questions carried — PASS
- II Clean-room: sources = imported export + RunMat code + published algorithm
  literature; no MATLAB invoked; no MathWorks/SuiteSparse/METIS source or prose
  — PASS
- III Implementation independence: the exports deliberately do NOT assert the
  permutation values; orderings use independent heuristics (minimum-degree /
  nested dissection), NOT reproductions of MATLAB internals — PASS
- IV Provenance: `provenance.md` complete; commit pin `4445294`; external
  algorithm sources recorded (Principle X) — PASS
- V Test-first: acceptance criteria (class/shape/valid-permutation) defined
  before bodies; tests authored alongside the red step — PASS
- VI Traceability: each FR ↔ claim ↔ test recorded in `validation.md` — PASS
- VII Minimal scope: two tightly coupled fill-reducing ordering builtins; no
  unrelated refactoring (only additive `mod` lines + one shared helper) — PASS
- VIII Human gates: gate approved with the binding constraint "independent
  heuristic ordering; specific values non-normative" — PASS
- IX Compatibility: follows `math/sparse/{full,speye,nonzeros}.rs` conventions
  (descriptor, GPU/fusion specs, `#[runtime_builtin]`, inline tests) — PASS
- X Licensing: external sources are published algorithm descriptions (journal /
  textbook); original Rust implementation; recorded in `provenance.md`;
  independent legal review may still be required — PASS

## Affected crates and files

| Item | Path |
|---|---|
| `colamd` | `crates/runmat-runtime/src/builtins/math/sparse/colamd.rs` (new) |
| `dissect` | `crates/runmat-runtime/src/builtins/math/sparse/dissect.rs` (new) |
| Shared pattern reader + registration | `crates/runmat-runtime/src/builtins/math/sparse/mod.rs` (add `mod colamd; mod dissect;` and `matrix_pattern`/`reject_nd` helpers) |
| Docs metadata | descriptor + `#[runtime_builtin]` doc fields (feeds `docs/builtins` tooling) |

Exemplars: `math/sparse/full.rs` and `speye.rs` (SparseTensor access, GPU_SPEC,
FUSION_SPEC, descriptor, error builders, inline tests); `nonzeros.rs`
(host-side structural scan + `gather_value_async`).

## Design decisions

- **Result shape**: 1×n `Value::Tensor` of one-based indices (`f64`). Empty
  matrix → 1×0 empty row. A 1×1 result is kept as an explicit 1×1 tensor so the
  output is uniformly a 1×n row vector (documented choice). `n` = number of
  columns.
- **Pattern reader** (`matrix_pattern`, shared in `mod.rs`): reads
  `(rows, cols, coords)` of structurally nonzero `(row, col)` from CSC sparse
  storage, dense/logical column-major storage, or a scalar (1×1). N-D and
  non-matrix inputs error. `NaN` counts as nonzero (it is a stored value).
- **`colamd`**: build the column-intersection graph (pattern of `Aᵀ·A`, diagonal
  removed) on the `cols` column-vertices, then run classic minimum-degree
  elimination (min current degree, ties by smallest index; fill edges make the
  eliminated vertex's neighbours a clique). Tinney & Walker 1967; George & Liu
  1981.
- **`dissect`**: build a symmetric graph — `A+Aᵀ` pattern when square; the
  column-intersection graph when non-square (documented choice, `A+Aᵀ` is
  undefined for rectangular). Then recursive nested dissection: split into
  connected components; within a connected component of size ≥ 3, root a BFS
  level structure at a pseudo-peripheral vertex, take a whole middle level as
  the separator (removing one BFS level disconnects lower from higher levels),
  number the two sides recursively first and the separator last. George 1973;
  George & Liu 1981; Gibbs–Poole–Stockmeyer 1976.
- **GPU/fusion**: both are host-only `GpuOpKind::Custom("sparse")`, not fusible
  (data-dependent combinatorial ordering). GPU tensor inputs are gathered
  before the structural scan.
- **Type resolver**: `Type::Tensor { shape: [Some(1), None] }` (1×n row).
- **Errors**: stable `RunMat:{colamd,dissect}:InvalidInput` and `:Internal`.

## Test plan (inline `#[cfg(test)]`)

Normative (assert ONLY class double, size 1×n, valid permutation of 1..n):
observed input, dense input, rectangular/larger input, empty matrix.
`documented_choice` (non-normative): the ordering on the observed input, scalar
1×1, logical input, non-square `dissect`, non-matrix rejection.
Infrastructure: descriptor registration (`builtin_function_by_name`),
GPU/fusion spec shape, type-resolver.

## Risks & rollback

- Risk: `mod.rs` is edited concurrently by a sibling (`spalloc`); resolved by
  additive edits only and re-running `cargo check`.
- Risk: the independent heuristic could produce an invalid permutation on some
  pattern — mitigated by the valid-permutation assertions across varied inputs
  and terminating recursion (each bisection strictly shrinks the vertex set).
- Rollback: delete the two new files and revert the `mod.rs` additions (no
  existing builtin is modified).

## Verification commands

`cargo check -p runmat-runtime` ·
`RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::math::sparse::colamd builtins::math::sparse::dissect` ·
`cargo fmt` then `cargo fmt --all -- --check`
