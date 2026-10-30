# Implementation Plan — 039-svds (spec 0.3.0)

## Constitution Check

- I Specification-first: FR-039-01..07 cite the 14 approved 0.3.0 claims.
  FR-039-08..16 are tagged documented choice or unresolved (with q-ids).
  PASS.
- II Clean-room: inputs are the imported export plus RunMat code only;
  MATLAB not invoked. PASS.
- III Independence: the subset is selected from a full economy SVD (textbook
  definition) computed by upstream `svd`. No iterative-solver internals are
  imitated. PASS.
- IV Provenance: `provenance.md` pinned to `8614c5b`, freshness-checked,
  with all 14 claim ids verified. PASS.
- V Test-first: the 0.3.0 tests were written and run against a test-only
  module (the 0.2.0-era implementation was removed) and failed with
  `UndefinedFunction` before the implementation was added. PASS.
- VII Minimal scope: one new file plus one `pub mod svds;` line in the
  hand-written `math/linalg/factor/mod.rs`; `svd.rs` unchanged. PASS.
- IX Compatibility: `builtins/math/linalg/factor/`, `#[runtime_builtin]`
  with a descriptor (`ByRequestedOutputCount`), `accel = "cpu"`, GPU inputs
  gathered. No integer-capability metadata (it ties a builtin to the
  generated docs catalog, which this fork cannot regenerate). PASS.

## Affected files

| Item | Path |
|---|---|
| Builtin (new) | `crates/runmat-runtime/src/builtins/math/linalg/factor/svds.rs` |
| Registration | `pub mod svds;` in `crates/runmat-runtime/src/builtins/math/linalg/factor/mod.rs` (hand-written parent) |

## Design

1. Output count: more than 3 → `TooManyOutputs`. More than two arguments
   after `A` → `UnsupportedOption` (q-options).
2. Gather `A`, `k` and `sigma` to the host.
3. `dense_input(A)`:
   - Real double or single, dense or sparse, is converted to a dense double
     tensor; the result remembers whether `A` was single.
   - Complex (dense or sparse) → `UnsupportedOption` (q-options).
   - Char and other non-floating values → `InvalidAFloat`.
   - Arrays with more than 2-D → `InvalidInput`.
4. `k`:
   - When omitted: `min(6, min(m, n))`.
   - When given: a finite non-negative integer scalar (else `InvalidK`),
     clipped to `min(m, n)`.
5. `sigma`:
   - `'largest'`/`'smallest'` (case-insensitive, char or string) or a
     finite real numeric scalar.
   - Anything else → `UnsupportedOption`.
6. Call `svd::evaluate(dense, [])`, or with `"econ"` when two or more outputs
   are requested. Values come back descending.
7. Selection:
   - largest: `0..k`.
   - smallest: `p-k..p`.
   - numeric: the k nearest by `|s − σ|`, ties to the lower index (larger
     value).
   - Selected indices are then sorted ascending, so values come out largest
     first.
8. Outputs:
   - `s` is a full `[k 1]` column, converted to single when `A` was single.
   - With two or more outputs: `U(:, idx)`, `diag(s)`, `V(:, idx)`,
     truncated to the requested count.
   - Requested count 0 → empty output list (as `svd`).

## Test plan (inline `#[cfg(test)] mod svds_tests`, via `crate::call_builtin_async[_with_outputs]`)

- One `observed_*` test per observed case (19 cases in 17 tests: the two
  `warn-*` cases share tests with `k-too-large` and an ordinary call).
  Warnings are checked through the thread-local `warning_store`.
- `documented_choice_*`: default-count rule, sigma text forms and order,
  numeric-sigma ties, two outputs and `k = 0` factors and the 4-output error,
  single factors, `k` validation, non-float inputs.
- `unresolved_options_and_complex_are_unsupported` (q-options).

## Risks & rollback

- `svd::evaluate` gates logical/integer inputs behind compatibility
  extensions. `svds` rejects those classes first, so its behaviour does not
  depend on the compatibility mode.
- Dense SVD is O(mn·min(m,n)). Large sparse inputs are densified (D1 = B
  carried forward; acceptable for the COBRA call sites).
- Rollback: delete `svds.rs` and its `mod` line.

## Verification commands

`cargo test -p runmat-runtime --lib -- svds_tests` ·
`cargo test -p runmat-runtime --lib -- factor::svd::` (no regressions) ·
`rustfmt --edition 2021 --check <file>` ·
`cargo clippy -p runmat-runtime --lib --tests --message-format short | grep svds.rs`
