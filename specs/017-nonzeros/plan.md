# Implementation Plan — 017-nonzeros

## Constitution Check

- I Specification-first: requirements cite approved claims — PASS
- II Clean-room: sources = imported export + RunMat code only; no MATLAB — PASS
- III Independence: trivial compaction scan over RunMat's own column-major
  dense and CSC sparse representations; no MATLAB algorithm inferred — PASS
- IV Provenance: `provenance.md` complete; commit pin
  `e46587ae1b78237e0f43cd55f678de01a28495a4` (Tier B batch 1) — PASS
- V Test-first: observed-case tests authored with the module and verified
  against the export before validation sign-off — PASS
- VI Traceability: FR ↔ claim ↔ test mapping in `validation.md` — PASS
- VII Minimal scope: one builtin, one new file + one `mod.rs` line — PASS
- VIII Human gates: full pipeline authorised for Tier B batch 1 — PASS
- IX Compatibility: follows `math/sparse` conventions (descriptor, GPU/fusion
  specs, `runtime_builtin` macro, inline tests) — PASS
- X Licensing: no external implementation sources — PASS

## Affected crates and files

| Item | Path |
|---|---|
| `nonzeros` | `crates/runmat-runtime/src/builtins/math/sparse/nonzeros.rs` (new) |
| Registration | `crates/runmat-runtime/src/builtins/math/sparse/mod.rs` (`pub(crate) mod nonzeros;` + provenance header line) |
| Docs metadata | descriptor + `#[runtime_builtin]` doc fields (feeds `docs/builtins` tooling) |

Exemplars: `math/sparse/full.rs` (sparse handling, descriptor, GPU/fusion
specs, error builders, inline test module), `math/sparse/spdiags.rs`
(`scalar_f64` reuse, gather of GPU inputs, `is_stored`-style zero test),
`array/indexing/find.rs` (column-vector type resolver,
`tensor_into_value` / `complex_tensor_into_value` normalisation).

The builtin is placed in `math/sparse` because that category centrally
handles `SparseTensor` inputs; the dense path is a plain column-major scan.

## Design decisions

- Single argument form only (`v = nonzeros(A)`, FR-017-01); the macro
  signature `async fn nonzeros_builtin(value: Value)` enforces arity.
- Dense path: filter `Tensor.data` (column-major storage) with
  `is_nonzero(v) = v.is_nan() || v != 0.0`; N-D tensors use the same scan
  (documented choice).
- Sparse path: RunMat CSC stores columns in order with rows sorted within
  each column, so `SparseTensor.values` is already column-major; filter out
  explicitly stored zeros defensively (documented choice).
- Result materialisation: `Tensor::new(kept, vec![n, 1])` then house
  normalisation `tensor_into_value` (n = 1 → `Value::Num`, the observed 1×1
  double; n = 0 → 0×1 empty double tensor, FR-017-04).
- Unobserved kinds (FR-017-05): scalar-likes via the shared `scalar_f64`
  helper; `LogicalArray` → double; complex kept complex via
  `complex_tensor_into_value`; GPU inputs gathered with
  `gpu_helpers::gather_value_async` (GPU spec: `GatherImmediately`);
  everything else errors `RunMat:nonzeros:InvalidInput`.
- Type resolver: `column_vector_type()` (`Type::Tensor` with shape
  `[None, Some(1)]`), as for `find`.
- Descriptor: output mode `Fixed` (single output); errors
  `RunMat:nonzeros:InvalidInput`, `RunMat:nonzeros:Internal`.
- Fusion spec: not fusible (data-dependent output length).

## Test plan (inline `#[cfg(test)]`, per house convention)

Normative `observed_*` (cite FR/claims in comments): the four export cases
`row`, `matrix`, `col-major` (order-discriminating), `sparse` with exact
value/shape assertions.
`summary_derived_*`: multi-column sparse matrix in column-major order (the
approved rule statement covers sparse inputs; the observed sparse case has a
single nonzero).
`unresolved_choice_*` (documented choices, non-normative): all-zero input →
0×1 empty (dense and sparse); scalars; NaN kept / negative zero dropped;
logical array → double; complex tensor/scalar; N-D column-major; non-numeric
input errors. Plus a type-resolver unit test.

## Risks & rollback

- Risk: sibling features touch shared `mod.rs` files concurrently — the
  change here is a single `mod` line; conflicts are line-additive.
- Risk: WASM registry regeneration — handled by the macro `builtin_path`;
  verified with `cargo check -p runmat-runtime`.
- Rollback: delete `nonzeros.rs` and the `mod.rs` line (no other file is
  modified).

## Verification commands

`cargo check -p runmat-runtime` ·
`RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::math::sparse::nonzeros` ·
`cargo fmt --all -- --check`
