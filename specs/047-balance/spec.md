# Feature Specification: `balance` — diagonal similarity scaling for eigenvalue accuracy

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`047-balance`; export `SPECIFY_FEATURE=047-balance`)

**Created**: 2026-10-30

**Status**: Batch N1 (F47). Gates G1–G3 pre-approved by the maintainer
(`specs/001-clean-room-matlab-builtins/selection-proposal-n1.md`).

**Input**: Approved export `balance` 0.2.0 (see `provenance.md`).

Throughout, `A0 = [1 100 10000; 0.01 1 100; 0.0001 0.01 1]` (the badly scaled
matrix used by most observed cases).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Balanced matrix and transformation (Priority: P1)

**Acceptance Scenarios** (normative, one per observed case):

1. `B = balance(A0)` → a 3×3 double (case `single-output`).
   [balance.signature-primary, balance.output-class]
2. `[T, B] = balance(A0)` → `T\A0*T = B` within 1e-12; `T` is diagonal with
   powers of two on its diagonal (case `two-outputs`). [balance.similarity,
   balance.power-of-two]
3. `balance(A0)` has the same eigenvalues as `A0` within 1e-10 (case
   `eig-preserved`). [balance.similarity]
4. `[T, B] = balance([1 0 0; 2 3 0; 4 5 6])` → `T\A*T = B` within 1e-12
   (case `permutation`). [balance.similarity]
5. `[S, P, B] = balance(A0)` → `S` a column equal to `diag(T)`, `P = [1;2;3]`,
   `B` as in scenario 1 (case `three-outputs`). [balance.three-outputs]

### User Story 2 - Trivial and option cases (Priority: P1)

1. `balance([1 2; 3 4])` → `[1 2; 3 4]` unchanged (case `balanced`);
   `balance(5)` → `5` (case `scalar`). [balance.already-balanced]
2. `balance(A0, 'noperm')` equals `balance(A0)` (case `noperm`).
   [balance.noperm]
3. `class(balance(single(A0)))` → `'single'` (case `single-class`).
   [balance.output-class]

### User Story 3 - Errors (Priority: P2)

1. `balance(ones(2, 3))` raises `RunMat:square` (case `err-nonsquare`).
   [balance.error-nonsquare]
2. `balance(speye(2))` raises `RunMat:balance:noBalanceSparse` (case
   `err-sparse`). [balance.error-sparse]

## Requirements *(mandatory)*

### Functional Requirements (normative)

- **FR-047-01** `B = balance(A)`, `[T, B] = balance(A)`,
  `[S, P, B] = balance(A)` and each of these with `'noperm'` MUST be accepted
  [balance.signature-primary]. Outputs MUST keep the input class: double for
  double, single for single [balance.output-class].
- **FR-047-02** `[T, B] = balance(A)` MUST satisfy `T\A*T = B` within 1e-12
  (relative to the size of `A`'s entries) and preserve the eigenvalues
  within 1e-10, both for a matrix that needs no permutation and for one
  that does [balance.similarity].
- **FR-047-03** For `A0`, `T` MUST be diagonal with powers of two on its
  diagonal [balance.power-of-two]. The exact exponents are not asserted
  (balance.q-exact-values; maintainer decision).
- **FR-047-04** `[S, P, B] = balance(A)` MUST return a column `S` and a
  column `P`. With no permutation, `S = diag(T)` and `P = (1:n)'`
  [balance.three-outputs].
- **FR-047-05** An already balanced matrix (`[1 2; 3 4]`) and a scalar MUST
  be returned unchanged [balance.already-balanced].
- **FR-047-06** `'noperm'` MUST give the same `B` as the default for a matrix
  that needs no permutation [balance.noperm].
- **FR-047-07** A non-square `A` MUST raise `RunMat:square`
  [balance.error-nonsquare]. A sparse `A` MUST raise
  `RunMat:balance:noBalanceSparse` [balance.error-sparse].

### Functional Requirements (documented choices and unresolved points)

- **FR-047-08** (documented choice; balance.observed-values is
  `normative: false`, balance.q-exact-values) Algorithm: first isolate
  eigenvalues by symmetric row/column permutations, then scale rows and
  columns of the remaining block by powers of two until the row and column
  1-norms (diagonal excluded) agree within the classical 0.95 criterion
  (Parlett & Reinsch). The exact `B` and `T` values are not asserted. The
  values RunMat produces for the observed inputs are recorded in a
  `documented_choice_*` test and in validation.md.
- **FR-047-09** (documented choice) `T = I(:, P) * diag(S)`: column `i` of `T`
  is `S(i)` times unit vector `P(i)`. This reduces to `S = diag(T)`,
  `P = (1:n)'` when no permutation is applied (FR-047-04). `S` has the input
  class; `P` is double.
- **FR-047-10** (unresolved, balance.q-noperm-permuting) `'noperm'` on a
  matrix that the default would permute applies scaling only: `P = (1:n)'`
  and `T` is diagonal. The option text may be a char row or a string scalar
  and is matched case-insensitively. Any other option, or a third argument,
  raises `RunMat:balance:UnsupportedOption`.
- **FR-047-11** (unresolved, balance.q-complex) Complex `A` MUST raise
  `RunMat:balance:UnsupportedOption`.
- **FR-047-12** (documented choice) `A` must be a real double or single
  matrix. Logical, integer, char, string, cell and struct values raise
  `RunMat:balance:InvalidInput`; arrays with more than two non-singleton
  dimensions also raise `RunMat:balance:InvalidInput`. A 0×0 `A` returns
  0×0 `B` and `T`, and 0×1 `S` and `P`. Rows or columns with non-finite
  norms are left unscaled, so `NaN`/`Inf` input terminates. GPU inputs are
  gathered to the host.
- **FR-047-13** (documented choice) More than three outputs raise
  `RunMat:balance:TooManyOutputs`.
- **FR-047-14** (unresolved, balance.q-warning-coverage) No code path emits a
  warning.

## Decisions

- **Error identifier namespace.** `MATLAB:square` → `RunMat:square`;
  `MATLAB:balance:noBalanceSparse` → `RunMat:balance:noBalanceSparse` (same
  rule as previous batches).
- **Exact balanced values are non-normative** (maintainer decision in
  `selection-proposal-n1.md`); tests assert similarity, structure and
  power-of-two scaling.

## Success Criteria *(mandatory)*

- **SC-047-1**: all 11 observed cases have an `observed_*` test (class, size,
  normative property or error).
- **SC-047-2**: `balance` is registered through `#[runtime_builtin]`, and
  `cargo test -p runmat-runtime --lib -- balance_tests` passes.
- **SC-047-3**: every FR maps to at least one test (validation.md).

## Unresolved behaviour (returned to the specification side)

| id | disposition |
|---|---|
| balance.q-warning-coverage | no warnings emitted (FR-047-14) |
| balance.q-exact-values | not asserted; algorithm documented (FR-047-08) |
| balance.q-noperm-permuting | documented choice: scaling only (FR-047-10) |
| balance.q-complex | `RunMat:balance:UnsupportedOption` (FR-047-11) |
