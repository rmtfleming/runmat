# Feature Specification: numeric N3/N4 — `sec`, `orth`, `condest`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`055-numeric-n3n4`; export `SPECIFY_FEATURE=055-numeric-n3n4`)

**Created**: 2026-10-30

**Status**: Batches N3 + N4, feature F55. Gates G1–G3 pre-approved by the
maintainer (`specs/001-clean-room-matlab-builtins/selection-proposal-n3n4.md`).

**Input**: Approved exports `sec` and `orth` **0.2.0** (N3, source
`1156c97`) and `condest` **0.2.0** (N4, source `fc9f804`); see
`provenance.md`.

Family rationale (Principle VII): three small dense numeric builtins from
the COBRA backlog. `sec` is an elementwise trigonometric function; `orth`
and `condest` are matrix functions built on standard factorisations (SVD,
LU). They share no state. Each has its own file, tests and requirement
block.

## Decisions

- **D-055-1 Error identifier namespace.** Observed `MATLAB:x:y`
  identifiers are raised as `RunMat:x:y` (maintainer rule). This gives
  `RunMat:math:mustBeFloat`, `RunMat:minrhs`, `RunMat:svd:noSVDsparse`,
  `RunMat:svd:inputType`, `RunMat:condest:NonSquareMatrix` and
  `RunMat:lu:inputType`. The `svd`/`lu` identifiers name another function;
  whether they must be reproduced is open on the spec side
  (`orth.q-error-ids`, `condest.q-error-ids`). RunMat follows the
  observation and flags the question to the maintainer. Identifiers that
  were not observed (`InvalidInput`, `UnsupportedOption`, `TooManyOutputs`,
  `Internal`) use `RunMat:<fn>:<name>`.
- **D-055-2 Placement of `sec`.** `sec` lives next to `cos`/`tan` in
  `math/trigonometry/`. That parent `mod.rs` is generated, so one line
  (`pub(crate) mod sec;`) is hand-added and reported.
- **D-055-3 `condest` computes the exact 1-norm condition number.** The
  "estimate" claim (`condest.estimate`) is a non-normative inference.
  RunMat returns `norm(A,1) * norm(inv(A),1)` with the inverse from an LU
  factorisation. This meets every observed value, and the `vs-cond`
  agreement holds by construction. It is a documented choice, not a claim
  about how MATLAB computes the value.
- **D-055-4 Unobserved points** raise `RunMat:<fn>:UnsupportedOption` or are
  labelled documented choices below. Neither kind is presented as MATLAB
  behaviour.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Secant (Priority: P1)

**Acceptance Scenarios** (normative, one per observed case):

1. `sec(0)` → `1`; `sec(pi)` → `-1`; `sec(pi/3)` → `1.9999999999999996`;
   `sec(-pi/4)` → `1.4142135623730949`, within 1e-12 (`zero`, `pi`, `pi3`,
   `negative`). [sec.signature-primary, sec.values]
2. `sec(pi/2)` is finite and large (`pole`). [sec.pole]
3. `sec([0 pi; pi/3 -pi/3])` keeps 2×2; `sec([])` → 0×0 (`matrix`,
   `empty`). [sec.shape]
4. `sec(NaN)` and `sec(Inf)` → `NaN` (`nan`, `inf`). [sec.nonfinite]
5. `sec(1+1i)` → `0.49833703055518686+0.59108384172104511i` (`complex`).
   [sec.complex]
6. `sec(single(1))` is single (`single`). [sec.single]
7. `sec(true)`, `sec(int8(1))`, `sec('a')` raise `RunMat:math:mustBeFloat`
   (`err-logical`, `err-int`, `err-char`). [sec.error-logical]
8. `sec()` raises `RunMat:minrhs` (`err-none`). [sec.error-none]

### User Story 2 - Orthonormal range basis (Priority: P1)

**Acceptance Scenarios** (normative; the basis is never compared entry by
entry, see FR-055-17):

1. `orth(magic(3))` is 3×3; `orth([1 2 3;2 4 6;1 0 1])` 3×2;
   `orth(ones(3))` 3×1; `orth(zeros(3))` 3×0; `orth([1 0;0 1;1 1;2 0])` 4×2;
   `orth([1 2 3 4;0 1 0 1])` 2×2 (`full-rank`, `rank2`, `rank1`, `zero`,
   `tall`, `wide`). [orth.signature-primary, orth.size-rank]
2. For those inputs, and for `single(magic(3))` and `[1 1i;1i 1]`,
   `norm(Q'*Q - I)` and `norm(A - Q*(Q'*A))` are below 1.2e-14 (double)
   or 5e-6 (single). [orth.orthonormal]
3. Single input gives single output; complex double input gives a
   (complex) double result (`single`, `complex`). [orth.class]
4. `orth([])` → 0×0; `orth(zeros(3,0))` → 3×0 (`empty`, `empty-3x0`).
   [orth.empty]
5. `orth(diag([1 1e-3 0]), 1e-2)` is 3×1 with span residual 1e-3
   (`second-arg`). [orth.second-arg]
6. Sparse `A` raises `RunMat:svd:noSVDsparse`; `int8` or char `A` raises
   `RunMat:svd:inputType` (`err-sparse`, `err-int`, `err-char`).
   [orth.error-sparse, orth.error-int]
7. *(non-normative)* `orth(diag([2 0 5]))` is 3×2 and spans e1 and e3
   (`diag-abs`); column order and signs are not asserted. [orth.diag]

### User Story 3 - Condition number (Priority: P1)

**Acceptance Scenarios** (normative):

1. `condest(speye(4))` → `1`; `condest(sparse(diag([1 10 100])))` → `100`;
   `condest([2 0;0 8])` → `4`; `condest(sparse(5))` → `1` (`identity`,
   `diag`, `dense`, `scalar`). [condest.signature-primary, condest.values]
2. A 4×4 sparse tridiagonal matrix agrees with `cond(full(A),1)`;
   `sparse([1 2 0;0 1 3;4 0 1])` gives 3.4 within 1e-12 (`vs-cond`,
   `vs-cond-unsym`). [condest.vs-cond]
3. `condest(sparse([1 2;2 4]))` and `condest(sparse(3,3))` → `Inf`
   (`singular`, `zero`). [condest.singular]
4. `condest(sparse(0,0))` → `0` (`empty`). [condest.empty]
5. `[c, v] = condest(sparse(diag([1 10 100])))` gives `c = 100`, `v` 3×1,
   `norm(v,1) = 1`, `norm(A*v,1) = 1` (`two-out`). [condest.two-out]
6. `condest(sparse(ones(2,3)))` raises `RunMat:condest:NonSquareMatrix`;
   `condest(['ab';'cd'])` raises `RunMat:lu:inputType` (`err-nonsquare`,
   `err-char`). [condest.error-nonsquare, condest.error-char]

## Requirements *(mandatory)*

### Functional Requirements — `sec` (normative)

- **FR-055-01** `Y = sec(X)` MUST be accepted and return the secant of each
  element [sec.signature-primary, sec.values].
- **FR-055-02** `sec(pi/2)` MUST be finite and large [sec.pole].
- **FR-055-03** The output has the size of the input, including empty
  [sec.shape].
- **FR-055-04** `NaN` and `Inf` give `NaN` [sec.nonfinite].
- **FR-055-05** Complex input gives a complex result [sec.complex].
- **FR-055-06** Single input gives a single result [sec.single].
- **FR-055-07** Logical, integer-class and char `X` MUST raise
  `RunMat:math:mustBeFloat` [sec.error-logical].
- **FR-055-08** A call with no input MUST raise `RunMat:minrhs`
  [sec.error-none].

### Functional Requirements — `orth` (normative)

- **FR-055-09** `Q = orth(A)` MUST be accepted; `Q` has `size(A,1)` rows and
  as many columns as the rank of `A` [orth.signature-primary,
  orth.size-rank].
- **FR-055-10** `Q` has orthonormal columns spanning range(A):
  `norm(Q'*Q - I)` and `norm(A - Q*(Q'*A))` at rounding level (below
  1.2e-14 double, 5e-6 single), also for complex input [orth.orthonormal].
- **FR-055-11** `A` may be a full double or single matrix, real or complex
  [orth.signature-primary, orth.class].
- **FR-055-12** Single input gives single output; complex double input gives
  a double (complex) result [orth.class].
- **FR-055-13** `orth([])` is 0×0 and `orth(zeros(3,0))` is 3×0
  [orth.empty].
- **FR-055-14** `Q = orth(A, t)` MUST be accepted;
  `orth(diag([1 1e-3 0]), 1e-2)` is 3×1 with span residual 1e-3
  [orth.second-arg].
- **FR-055-15** Sparse `A` MUST raise `RunMat:svd:noSVDsparse`
  [orth.error-sparse].
- **FR-055-16** Integer-class or char `A` MUST raise `RunMat:svd:inputType`
  [orth.error-int].
- **FR-055-17** (non-normative) Basis entries, column order and signs are
  not required (`orth.diag`, `orth.basis-not-unique`); tests assert only
  shape, class, orthonormality, span and, for `diag-abs`, which coordinate
  axes are spanned.

### Functional Requirements — `condest` (normative)

- **FR-055-18** `c = condest(A)` and `[c, v] = condest(A)` MUST be accepted
  for a square sparse or full double matrix; `c` is a double scalar
  [condest.signature-primary, condest.two-out].
- **FR-055-19** The observed values MUST be reproduced: `speye(4)` → 1,
  sparse `diag([1 10 100])` → 100, `[2 0;0 8]` → 4, `sparse(5)` → 1
  [condest.values].
- **FR-055-20** On the observed small matrices `c` MUST agree with
  `cond(full(A),1)` (exactly for the tridiagonal case, within 1e-15 relative
  rounding for the unsymmetric case; RunMat tests 1e-14 relative and 1e-12
  absolute against 3.4) [condest.vs-cond].
- **FR-055-21** A singular or all-zero matrix gives `Inf`
  [condest.singular].
- **FR-055-22** A 0×0 input gives `0` [condest.empty].
- **FR-055-23** `v` is a column of length `size(A,1)`; for sparse
  `diag([1 10 100])`, `norm(v,1) = 1` and `norm(A*v,1) = 1`
  [condest.two-out].
- **FR-055-24** Non-square `A` MUST raise `RunMat:condest:NonSquareMatrix`
  [condest.error-nonsquare].
- **FR-055-25** A square char matrix MUST raise `RunMat:lu:inputType`
  [condest.error-char].

### Functional Requirements (documented choices and unresolved points)

- **FR-055-26** (documented choice, condest) For an invertible `A`, `v` is
  the column of `inv(A)` with the largest 1-norm scaled to unit 1-norm, so
  `norm(A*v,1) = 1/norm(inv(A),1)`. For a singular `A` (`c = Inf`), `v` is
  a right singular vector of the smallest singular value, scaled to unit
  1-norm. For 0×0 `A`, `v` is 0×1. The class of `A` is checked before its
  shape.
- **FR-055-27** (documented choice, orth, `orth.q-default-tol`) The default
  rank threshold is `max(m,n) * eps(class) * sigma_max`; singular values
  above it are kept.
- **FR-055-28** (documented choice, orth, `orth.q-second-arg`) `t` is an
  absolute threshold (`sigma > t` is kept). It must be a finite
  non-negative real double/single scalar, else
  `RunMat:orth:UnsupportedOption`.
- **FR-055-29** (documented choice, all three) Arity and outputs: `sec`
  takes one input (more → `RunMat:sec:InvalidInput`); `orth` one or two
  (else `RunMat:orth:InvalidInput`); `condest` one (none →
  `RunMat:condest:InvalidInput`, extra arguments →
  `RunMat:condest:UnsupportedOption`). More outputs than `sec`/`orth`'s one
  or `condest`'s two raise `RunMat:<fn>:TooManyOutputs`. N-D `A` raises
  `RunMat:orth:InvalidInput` / `RunMat:condest:InvalidInput`.
- **FR-055-30** (documented choice, all three) Unobserved classes raise
  `RunMat:<fn>:UnsupportedOption`: `sec` — sparse, string and other
  non-numeric classes; `orth` — logical, `A` with NaN/Inf, other classes;
  `condest` — single, complex, integer, logical, NaN/Inf entries, other
  classes.
- **FR-055-31** (documented choice, all three) gpuArray inputs are gathered
  and the result is a host array.
- **FR-055-32** (documented choice, sec) Complex arrays keep their shape;
  complex single stays complex single; a complex input always gives a
  complex result (no demotion when the imaginary part is zero). Single is
  computed in double and rounded.
- **FR-055-33** (unresolved, `*.q-warning-coverage` for all three) None of
  the three functions emits a warning.

## Success Criteria *(mandatory)*

- **SC-055-1**: all observed cases reproduce (class, size, value or error
  identifier, or the structural properties for `orth`): 15 for `sec`, 15 for
  `orth`, 12 for `condest`.
- **SC-055-2**: the three builtins are registered through
  `#[runtime_builtin]`, and `cargo test -p runmat-runtime --lib --
  sec_tests orth_tests condest_tests` passes.
- **SC-055-3**: every FR maps to at least one test (`validation.md`).

## Unresolved behaviour (returned to the specification side)

| id | disposition |
|---|---|
| sec.q-warning-coverage, orth.q-warning-coverage, condest.q-warning-coverage | no warning emitted (FR-055-33) |
| orth.q-default-tol | documented choice FR-055-27 |
| orth.q-second-arg | documented choice FR-055-28 (absolute) |
| orth.q-error-ids, condest.q-error-ids | observed identifiers reproduced in the RunMat namespace (D-055-1); flagged to maintainer |
| condest.q-larger, condest.q-exactness, condest.estimate | exact value computed (D-055-3); tests use a tolerance against `cond(A,1)` |
| orth.basis-not-unique, orth.diag | non-normative; structural assertions only (FR-055-17) |
