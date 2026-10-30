# Implementation Plan — 047-balance (`balance` 0.2.0)

## Constitution Check

- I Specification-first: FR-047-01..07 cite the 9 normative claims;
  FR-047-08..14 are tagged documented choice or unresolved. PASS.
- II Clean-room: imported export plus RunMat code only; MATLAB not invoked.
  PASS.
- III Independence: Parlett & Reinsch (1969) balancing from the published
  description. Exact values are not imitated (non-normative). PASS.
- IV Provenance: pinned to `c65e238`, freshness-checked, 10 claim ids and 4
  question ids verified. PASS.
- V Test-first: tests run first against a module with no builtin; the failing
  run is recorded in validation.md. PASS.
- VII Minimal scope: one new file plus `pub mod balance;` in the
  hand-written `math/linalg/factor/mod.rs`. PASS.
- IX Compatibility: `#[runtime_builtin]` with a descriptor
  (`ByRequestedOutputCount`), `accel = "cpu"`, GPU inputs gathered, no
  integer-capability metadata. PASS.

## Affected files

| Item | Path |
|---|---|
| Builtin (new) | `crates/runmat-runtime/src/builtins/math/linalg/factor/balance.rs` |
| Registration | `pub mod balance;` in `crates/runmat-runtime/src/builtins/math/linalg/factor/mod.rs` |

## Design

1. Output count > 3 → `TooManyOutputs`. Second argument: `'noperm'`
   (case-insensitive char row or string scalar), else `UnsupportedOption`;
   a third argument → `UnsupportedOption`.
2. Gather to the host. Sparse → `RunMat:balance:noBalanceSparse`; complex →
   `UnsupportedOption`; non-float classes and N-D → `InvalidInput`;
   rows ≠ cols → `RunMat:square`.
3. Work on a row-major-agnostic dense `Vec<f64>` (column-major, n×n),
   computing single input in double (power-of-two scaling is exact).
4. Permutation phase (skipped for `'noperm'`): with active range
   `[low, high]`, repeatedly find a row whose off-diagonal entries in
   columns `low..=high` are all zero, swap it (row and column) to `high`, and
   shrink `high`. Then repeatedly find a column whose off-diagonal entries in
   rows `low..=high` are all zero, swap it to `low`, and grow `low`. Each
   swap is a full symmetric row/column exchange and is also applied to the
   column permutation `perm`.
5. Scaling phase over `low..=high`: for each `i`, compute the off-diagonal
   column and row 1-norms `c` and `r` within the active block. Skip rows
   with zero or non-finite norms. Find the power of two `f` that brings
   `c·f²` closest to `r` (radix 2: multiply `c` by 4 while `c < r/2`, divide
   while `c ≥ 2r`). Apply it only if `(c + r)/f < 0.95 (c + r)` (in the
   scaled norms), scaling column `i` by `f` and row `i` by `1/f`, with
   `s[i] *= f`. Repeat sweeps until no change; an iteration cap guarantees
   termination.
6. Outputs: `T = I(:, perm) * diag(s)`, `B` the transformed matrix,
   `S = s` (column, input class), `P = perm + 1` (column, double). One
   output → `B`; two → `[T, B]`; three → `[S, P, B]`; zero → empty list.

## Test plan (inline `#[cfg(test)] mod balance_tests`)

- One `observed_*` test per observed case (11).
- Similarity checked in the test by forming `T\A*T` through RunMat's
  `mldivide`/`mtimes` builtins, eigenvalue preservation through `eig`.
  Power-of-two check: every diagonal entry `d` satisfies
  `log2(d)` integral and `T` has no off-diagonal nonzeros.
- `documented_choice_*`: recorded values for the observed inputs, `'noperm'`
  on a permutable matrix, `T = I(:,P)*diag(S)` on a permuted case, single
  outputs, empty input, input classes, option text, `NaN`/`Inf`
  termination, a random larger matrix (similarity and powers of two).
- `unresolved_*`: complex input, other options.
- `TooManyOutputs` called directly under `push_output_count`.

## Risks & rollback

- O(n²) per sweep; acceptable.
- Rollback: delete `balance.rs` and its `mod` line.

## Verification commands

`cargo test -p runmat-runtime --lib -- balance_tests` ·
`cargo test -p runmat-runtime --lib -- factor::svd` (no regressions) ·
`rustfmt --edition 2021 --check …/factor/balance.rs` ·
`cargo clippy -p runmat-runtime --lib --tests --message-format short | grep balance.rs`
