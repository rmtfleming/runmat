# Feature Specification: `svds` — a subset of singular values and vectors

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`039-svds`; export `SPECIFY_FEATURE=039-svds`)

**Created**: 2026-10-29 (revised the same day for spec 0.3.0)

**Status**: Batch C2 — implemented from the approved 0.3.0 export (maintainer
decision 2026-10-29: 0.3.0 supersedes the earlier D1 "mixed" choices wherever
it resolves a point). Gates 1–3 pre-approved by the maintainer.

**Input**: Approved Tier B export `svds` **0.3.0** (see `provenance.md`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - The k largest singular values (Priority: P1)

`s = svds(A, k)` returns a full column of the `k` largest singular values,
largest first.

**Acceptance Scenarios** (normative, one per observed case):

1. `svds(sparse([2 0; 0 3]), 2)` → full double `[3; 2]` (case `k`).
   [svds.signature-primary, svds.output-class, svds.largest]
2. `svds(diag([4 1 3]), 2)` → `[4; 3]` (case `dense`). [svds.output-class,
   svds.largest]
3. `svds(sparse(diag([1 5 2 4])), 4)` → `[5; 4; 2; 1]` (case `order`).
   [svds.largest]
4. `svds(sparse([1 0 0; 0 2 0]), 2)` → `[2; 1]` (case `rectangular`).
   [svds.rectangular]
5. `svds(sparse(3, 3), 2)` → `[0; 0]` (case `zero-matrix`).
   [svds.zero-matrix]
6. `svds(single(diag([4 1 3])), 2)` → single `[4; 3]` (case `single`).
   [svds.output-class]

### User Story 2 - Default count and k edge cases (Priority: P1)

1. `svds(sparse(diag(1:8)))` → `[8;7;6;5;4;3]`; `svds(sparse([2 0; 0 3]))`
   → `[3; 2]` (cases `default-8`, `default-2`). [svds.default-count]
2. `svds(sparse([2 0; 0 3]), 3)` → `[3; 2]`, with no error and no warning
   (cases `k-too-large`, `warn-k-too-large`). [svds.k-exceeds]
3. `svds(sparse([2 0; 0 3]), 0)` → `zeros(0, 1)` double (case `k-zero`).
   [svds.k-zero]
4. An ordinary call issues no warning (case `warn-k`). [svds.no-warning]

### User Story 3 - Selecting by sigma (Priority: P2)

1. `svds(sparse(diag([4 1 3])), 1, 'smallest')` → `1` (case `smallest`).
   [svds.smallest]
2. `svds(sparse(diag([4 1 3])), 2, 'largest')` → `[4; 3]` (case
   `largest`). [svds.largest]
3. `svds(sparse(diag([4 1 3])), 1, 2.5)` → `3` (case `sigma-numeric`).
   [svds.sigma-numeric]

### User Story 4 - Singular vectors (Priority: P2)

1. `[U,S,V] = svds(sparse(diag([4 1 3])), 2)` → `U` 3×2, `S` 2×2, `V` 3×2,
   all full; `diag(S) = [4; 3]`; orthonormal columns; `A*V = U*S`
   (case `usv-shapes`). [svds.usv]
2. `[U,S,V] = svds(sparse([1 0 0; 0 2 0]), 2)` → `U` 2×2, `S` 2×2, `V` 3×2,
   `diag(S) = [2; 1]` (case `usv-rect`). [svds.usv]

### User Story 5 - Errors (Priority: P2)

1. `svds('abc', 1)` raises `RunMat:svds:InvalidAFloat` (case `err-char`).
   [svds.error-char]
2. `svds(sparse([2 0; 0 3]), -1)` raises `RunMat:svds:InvalidK` (case
   `err-k-negative`). [svds.error-k-negative]

## Requirements *(mandatory)*

### Functional Requirements (normative)

- **FR-039-01** `svds(A)`, `svds(A, k)` and `svds(A, k, sigma)` MUST be
  accepted. The single output MUST be a full (non-sparse) column vector of
  class double for double input, sparse or full, and of class single for
  single input [svds.signature-primary, svds.output-class].
- **FR-039-02** With `k` given (and with `sigma = 'largest'`), the `k`
  largest singular values MUST be returned in descending order
  [svds.largest]. Rectangular input MUST be accepted [svds.rectangular]. An
  all-zero matrix MUST give zeros [svds.zero-matrix].
- **FR-039-03** Without `k`, an 8×8 matrix MUST give its 6 largest values
  and a 2×2 matrix both of its values [svds.default-count].
- **FR-039-04** `k > min(size(A))` MUST be clipped to `min(size(A))`
  silently, with no error and no warning [svds.k-exceeds]. `k = 0` MUST
  return a `0×1` double [svds.k-zero]. An ordinary call MUST issue no
  warning [svds.no-warning].
- **FR-039-05** `sigma = 'smallest'` MUST return the smallest singular
  values [svds.smallest]. A numeric `sigma` MUST return the singular values
  closest to it [svds.sigma-numeric].
- **FR-039-06** `[U, S, V] = svds(...)` MUST return full `U` (m×k), `S`
  (k×k, diagonal) and `V` (n×k); `diag(S)` equals the single-output values;
  `U` and `V` have orthonormal columns; `A*V = U*S` within 1e-10
  [svds.usv].
- **FR-039-07** A char `A` MUST raise `RunMat:svds:InvalidAFloat`
  [svds.error-char]. A negative `k` MUST raise `RunMat:svds:InvalidK`
  [svds.error-k-negative]. The export records the identifiers
  `MATLAB:svds:InvalidAFloat` / `MATLAB:svds:InvalidK`. RunMat keeps the
  observed suffix in its own namespace, following the batch C2 decision D2 = B
  precedent (see the Decisions section).

### Functional Requirements (documented choices and unresolved points)

- **FR-039-08** (documented choice, svds.q-default-rule) The default count
  is `min(6, min(size(A)))`. This is the simplest rule consistent with
  both observations. It also applies to `[U,S,V] = svds(A)`.
- **FR-039-09** (documented choice) Selected values are always returned
  largest first. This is observed for `'largest'` and chosen for `'smallest'`
  with k > 1 and for numeric sigma. Text sigma is matched case-insensitively
  and may be a char row or a string scalar.
- **FR-039-10** (documented choice, svds.q-sigma-numeric) Numeric sigma
  selects the `k` values with the smallest `|s − sigma|`. A tie goes to the
  larger value.
- **FR-039-11** (documented choice) `k` MUST be a real numeric scalar
  holding a finite non-negative integer (typed integers accepted).
  Fractional, NaN, Inf, non-scalar, empty, logical and text `k` raise
  `RunMat:svds:InvalidK`, the same error as the observed negative case.
- **FR-039-12** (documented choice) `A` must be a real double or single
  matrix, dense or sparse. Logical, integer, string, cell, struct and other
  non-floating values raise `RunMat:svds:InvalidAFloat`, as char does. An
  array with more than two non-singleton dimensions raises
  `RunMat:svds:InvalidInput`. GPU inputs are gathered to the host.
- **FR-039-13** (documented choice) Two outputs return `[U, S]`. More than
  three outputs raise `RunMat:svds:TooManyOutputs`. With `k = 0` the
  factors are `m×0`, `0×0` and `n×0`. Single input gives single factors.
- **FR-039-14** (unresolved, svds.q-options) Any argument after `sigma`,
  an options struct, a text sigma other than `'largest'`/`'smallest'`, a
  non-scalar or non-finite numeric sigma, and complex `A` MUST raise
  `RunMat:svds:UnsupportedOption`.
- **FR-039-15** (unresolved, svds.q-vector-signs) Singular-vector signs
  are whatever upstream `svd` produces. Tests check only shapes,
  orthonormality and `A*V = U*S`.
- **FR-039-16** (unresolved, svds.q-warning-coverage) No `svds` code path
  emits a warning. Tests check this for the two `warn-*` cases only.

## Decisions

- **Error identifier namespace.** The batch C2 decision D2 = B (recorded in
  `specs/001-clean-room-matlab-builtins/selection-proposal-tierc2.md`) put
  `validateattributes` errors in the `RunMat:` namespace and treated the
  observed `MATLAB:` identifier as non-normative. This feature applies the
  same rule: `RunMat:svds:InvalidAFloat`, `RunMat:svds:InvalidK`. **Flag for
  maintainer review**: if the maintainer prefers emitting the observed
  `MATLAB:` identifiers, only the two descriptor constants change.

## Success Criteria *(mandatory)*

- **SC-039-1**: all 19 observed cases reproduce (class, size, value or
  error), each in its own `observed_*` test.
- **SC-039-2**: `svds` is registered through `#[runtime_builtin]`, and
  `cargo test -p runmat-runtime --lib -- svds_tests` passes.
- **SC-039-3**: every FR maps to at least one test (validation.md).

## Unresolved behaviour (returned to the specification side)

| id | question | disposition |
|---|---|---|
| svds.q-warning-coverage | warnings checked only in `warn-*` cases | no warnings emitted (FR-039-16) |
| svds.q-default-rule | general default-count rule | documented choice `min(6, min(size(A)))` (FR-039-08) |
| svds.q-sigma-numeric | ties / several values near the target | documented choice: nearest, ties to larger, largest first (FR-039-10) |
| svds.q-options | options struct, name-value settings, complex input | `UnsupportedOption` (FR-039-14) |
| svds.q-vector-signs | singular-vector signs | not asserted (FR-039-15) |

Resolved by 0.3.0 and superseding the 0.2.0 dispositions: `svds.q-default`
(was `UnsupportedOption`) and `svds.q-vectors` (was `UnsupportedOption`).
Also superseded: the 0.2.0-era choices "k > min(size) is an error", "k = 0 is
an error", "single precision is unsupported" and "complex input is
supported" (the latter is now `UnsupportedOption` under svds.q-options).

## Assumptions

- Upstream `svd` (`math/linalg/factor/svd.rs`) returns singular values in
  descending order with matching economy-size `U`/`V`, and is reused
  unchanged.
