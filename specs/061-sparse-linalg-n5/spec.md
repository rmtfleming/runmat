# Feature Specification: sparse linear algebra N5 — `lsqminnorm`, `sprandn`, `sprank`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`061-sparse-linalg-n5`; export `SPECIFY_FEATURE=061-sparse-linalg-n5`)

**Created**: 2026-10-31

**Status**: Batch N5, feature F61. Gates G1–G3 pre-approved by the
maintainer (`specs/001-clean-room-matlab-builtins/selection-proposal-n5.md`).

**Input**: Approved exports `lsqminnorm`, `sprandn` and `sprank` **0.2.0**
(N5, source `1c91aa1`); see `provenance.md`.

Family rationale (Principle VII): three sparse/linear-algebra builtins from
the COBRA backlog. `lsqminnorm` solves least-squares problems (COBRA calls
`lsqminnorm(S', b)` with a sparse stoichiometric matrix), `sprandn` builds
random sparse test matrices and `sprank` measures the structural rank of a
sparse pattern. They share no state. Each has its own file, tests and
requirement block.

## Decisions

- **D-061-1 Error identifier namespace.** Observed `MATLAB:x:y`
  identifiers are raised as `RunMat:x:y` (maintainer rule). This gives
  `RunMat:decomposition:mldivide`, `RunMat:lsqminnorm:InvalidA`,
  `RunMat:narginchk:notEnoughInputs`, `RunMat:RandStream:randi:invalidLimits`,
  `RunMat:sprandn:TwoInputs`, `RunMat:math:mustBeFloatOrLogical` and
  `RunMat:minrhs`. `decomposition:mldivide` and `RandStream:randi` name
  other components; whether they must be reproduced is open on the spec
  side (`lsqminnorm.q-error-ids`, `sprandn.q-error-ids`,
  `sprank.q-error-ids`). RunMat follows the observation (brief:
  "raise the observed id under `RunMat:` prefix") and flags the question to
  the maintainer. Identifiers that were not observed (`InvalidInput`,
  `UnsupportedOption`, `TooManyOutputs`, `Internal`) use
  `RunMat:<fn>:<name>`.
- **D-061-2 `lsqminnorm` default path.** Without `tol` RunMat returns the
  minimum-norm least-squares solution from a thin SVD,
  `x = V * pinv(Sigma) * U' * b`, keeping the singular values above
  RunMat's own `pinv` default threshold `max(m,n) * eps(max sigma)`
  (`svd_default_tolerance`). This is `pinv(A)*b` by construction, which is
  what the confirmed claim `lsqminnorm.vs-pinv` requires.
- **D-061-3 `lsqminnorm` tolerance path (documented choice for the
  unresolved `lsqminnorm.q-tol-rule`; maintainer disposition "follow the
  observed case").** With `tol`, RunMat computes a Householder QR
  factorisation with column pivoting `A*P = Q*R`, keeps the leading
  components whose `|R(k,k)| > tol` (an absolute threshold), and returns
  `x = P*z` with `z` the minimum-norm solution of the full-row-rank system
  `R(1:r,:) * z = (Q'*b)(1:r,:)`. On `[2 1; 1 0.2]`, `b = [1; 1]`,
  `tol = 0.5` this reproduces the observed `[0.50268096514745331;
  0.22117962466487934]` (that is `[3; 1.32]/5.968`), and with `tol = 1e-3`
  it is the unique solution `pinv(A)*b`. This is the non-normative
  inference `lsqminnorm.tol-rule` implemented as a labelled RunMat choice;
  it is not presented as MATLAB's rule.
- **D-061-4 `lsqminnorm` densifies sparse input.** Sparse `A` and `b` are
  converted to dense storage before the SVD/QR. Results meet the
  requirements, but memory is `O(m*n)` and time `O(m*n*min(m,n))`, which
  limits the size of COBRA models that can be solved in practice. This is a
  documented performance limitation, not a behavioural deviation (no
  sparse least-squares solver exists in RunMat to build on).
- **D-061-5 `sprandn` construction (documented choice; maintainer
  disposition for `sprandn.q-values`: "structure, class, density bound and
  seeded repeatability only; values not asserted").** The nonzero count is
  `floor(density*m*n)` (so the observed upper bound `density*m*n` always
  holds), the positions are a uniform sample without replacement and the
  values standard normal draws, all from RunMat's global random stream (the
  one `rng` seeds), so the same seed gives the same matrix. The `rc` form
  reuses the `sprand` condition-number construction of the parent module
  (geometric singular values `1 .. rc` on a diagonal, spread by random
  plane rotations). No test compares values with oracle values.
- **D-061-6 Unobserved points** raise `RunMat:<fn>:UnsupportedOption` or
  are labelled documented choices below. Neither kind is presented as
  MATLAB behaviour.
- **D-061-7 Placement.** `lsqminnorm` in `math/linalg/solve/` (hand-written
  `mod.rs`), `sprank` in `math/sparse/` (hand-written `mod.rs`), `sprandn`
  as a child module of the hand-written `array/creation/sparse/mod.rs`, so
  it can call the private `sprand` helpers there (`sample_unique_positions`,
  `sparse_from_entries`, `sparse_pattern_from_value`,
  `scalar_rcond_singular_profile`, `sprand_from_condition_profile`)
  without changing their visibility. No generated parent is touched.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Minimum-norm least squares (Priority: P1)

**Acceptance Scenarios** (normative, one per observed case; values within
1e-12 of the observation and of RunMat's `pinv(full(A))*b`,
`lsqminnorm.q-exactness`):

1. `lsqminnorm([2 1; 1 3], [3; 5])` → `[0.8; 1.4]`
   (`square-full-rank`). [lsqminnorm.signature-primary, lsqminnorm.vs-pinv]
2. `lsqminnorm([1 0; 0 1; 1 1], [1; 2; 4])` → `[4/3; 7/3]`
   (`overdetermined`); `lsqminnorm([1 1 0; 0 1 1], [2; 3])` →
   `[1/3; 5/3; 4/3]` (`underdetermined`); `lsqminnorm([1 2; 2 4; 3 6],
   [1; 2; 3])` → `[0.2; 0.4]` with no warning (`rank-deficient`);
   two right-hand sides `[2 1; 3 0]` → 3×2 (`multi-rhs`).
   [lsqminnorm.vs-pinv]
3. `S = sparse([1 -1 0; 0 1 -1; 1 0 -1]); lsqminnorm(S', [1; 2; 3])` →
   full double `[-1/3; -1/3; -2/3]`, residual norm 3.4641
   (`sparse-transposed`); `lsqminnorm(sparse([1 0 2 0; 0 3 0 1]), [1; 1])`
   → full `[0.2; 0.3; 0.4; 0.1]` (`sparse-wide`). [lsqminnorm.cobra-form,
   lsqminnorm.full-output, lsqminnorm.vs-pinv]
4. `lsqminnorm(zeros(2,3), [1; 1])` → `zeros(3,1)` (`zero-matrix`).
   [lsqminnorm.zero]
5. `lsqminnorm(single([2 1; 1 3]), single([3; 5]))` is single, within 1e-6
   of `[0.8; 1.4]` (`single`). [lsqminnorm.single]
6. `lsqminnorm([1 1i; 0 1], [1; 1i])` → `[2; 1i]` (`complex`).
   [lsqminnorm.vs-pinv]
7. `lsqminnorm(zeros(0,3), zeros(0,1))` → 3×1 zeros (`empty`).
   [lsqminnorm.empty]
8. `lsqminnorm([2 1; 1 0.2], [1; 1], 0.5)` → `[0.5027; 0.2212]`, at
   distance 2.0625 from `pinv(A)*b` (`tol-arg`); with `1e-3` →
   `[4/3; -5/3]` = `pinv(A)*b` (`tol-small`). [lsqminnorm.tol]
9. Row mismatch → `RunMat:decomposition:mldivide` (`err-rows`); char or
   `int32` `A` → `RunMat:lsqminnorm:InvalidA` (`err-char`, `err-int`); `A`
   only → `RunMat:narginchk:notEnoughInputs` (`err-one-arg`).
   [lsqminnorm.error-rows, lsqminnorm.error-char, lsqminnorm.error-one-arg]

### User Story 2 - Structural rank (Priority: P1)

**Acceptance Scenarios** (normative):

1. `sprank(speye(3))` → `3` (double scalar) (`identity`).
   [sprank.signature-primary, sprank.matching, sprank.class]
2. `sprank(sparse(ones(2)))` → `2` while `rank` is `1` (`cancelling`).
   [sprank.structural]
3. A zero row gives 2; `[1 1 0; 0 0 1; 0 0 1]` gives 2; the 2×3
   `[1 1 0; 0 1 1]` and the 4×2 `[1 0; 1 0; 1 0; 0 1]` give 2
   (`zero-row`, `matching-limited`, `wide`, `tall`). [sprank.matching]
4. `sprank(sparse(3,3))` and `sprank(sparse(0,0))` → `0` (`zero-matrix`,
   `empty`). [sprank.zero-empty]
5. Full `eye(2)`, sparse logical `eye(2)` and sparse `1i*eye(2)` → `2`
   (`dense`, `logical`, `complex`). [sprank.inputs, sprank.class]
6. Char or cell input → `RunMat:math:mustBeFloatOrLogical` (`err-char`,
   `err-cell`); no input → `RunMat:minrhs` (`err-no-arg`).
   [sprank.error-char, sprank.error-no-arg]

### User Story 3 - Sparse normal random matrices (Priority: P2)

**Acceptance Scenarios** (normative; values are never compared,
`sprandn.q-values`):

1. After `rng(7)`, `sprandn(5, 6, 0.6)` and `sprandn(30, 30, 0.3)` are
   real sparse double matrices of that size with finite nonzeros of both
   signs, at most `density*m*n` nonzeros, and identical when the seed is
   repeated (`cobra-5x6`, `cobra-30x30`); likewise `sprandn(4, 5, 1)`
   (`density-one`). [sprandn.signature-primary, sprandn.structure,
   sprandn.nnz-bound]
2. `sprandn(4, 5, 0)` is an all-zero sparse 4×5; `sprandn(0, 3, 0.5)` an
   empty sparse 0×3 (`density-zero`, `empty-dims`). [sprandn.zero-density]
3. `sprandn(S)` keeps exactly the pattern of `S` (`pattern`).
   [sprandn.pattern]
4. `sprandn(4, 4, 0.5, 0.1)` is a 4×4 sparse double with
   `cond(full(R)) = 10` (`rc`). [sprandn.rc]
5. `sprandn(2.5, 3, 0.5)` → `RunMat:RandStream:randi:invalidLimits`
   (`err-noninteger`); `sprandn(3, 3)` → `RunMat:sprandn:TwoInputs`
   (`err-two-args`). [sprandn.error-noninteger, sprandn.error-two-args]
6. *(non-normative)* densities 1.5 and −0.1 do not error (3×3 sparse;
   −0.1 gives no nonzeros) (`density-above-one`, `density-negative`).
   [sprandn.density-out-of-range]

## Requirements *(mandatory)*

### Functional Requirements — `lsqminnorm` (normative)

- **FR-061-01** `x = lsqminnorm(A, b)` MUST be accepted and agree with
  `pinv(full(A))*b` (tolerance, not digits) for square, over- and
  underdetermined, rank-deficient, all-zero, complex, multi-column and
  sparse inputs [lsqminnorm.signature-primary, lsqminnorm.vs-pinv].
- **FR-061-02** Sparse `A` gives a full (non-sparse) double result
  [lsqminnorm.full-output].
- **FR-061-03** The COBRA form `lsqminnorm(S', b)` with sparse
  rank-deficient `S` returns the minimum-norm least-squares solution of the
  inconsistent system [lsqminnorm.cobra-form].
- **FR-061-04** All-zero `A` gives `zeros(columns(A), columns(b))`
  [lsqminnorm.zero].
- **FR-061-05** Single inputs give a single result [lsqminnorm.single].
- **FR-061-06** `A` 0×3 with `b` 0×1 gives a 3×1 zero vector
  [lsqminnorm.empty].
- **FR-061-07** `x = lsqminnorm(A, b, tol)` MUST be accepted; on
  `[2 1; 1 0.2]`, `[1; 1]` the result with `tol = 0.5` differs from
  `pinv(A)*b` and is `[0.5027; 0.2212]`; with `tol = 1e-3` it equals
  `pinv(A)*b` [lsqminnorm.tol].
- **FR-061-08** (non-normative) The rule by which `tol` truncates is an
  inference (`lsqminnorm.tol-rule`); RunMat's rule is D-061-3.
- **FR-061-09** A row mismatch between `A` and `b` MUST raise
  `RunMat:decomposition:mldivide` [lsqminnorm.error-rows].
- **FR-061-10** Char or integer-class `A` MUST raise
  `RunMat:lsqminnorm:InvalidA` [lsqminnorm.error-char].
- **FR-061-11** A call with only `A` MUST raise
  `RunMat:narginchk:notEnoughInputs` [lsqminnorm.error-one-arg]. (A call
  with no input raises the same identifier: documented choice.)

### Functional Requirements — `lsqminnorm` (documented choices)

- **FR-061-12** More than one output raises
  `RunMat:lsqminnorm:TooManyOutputs`.
- **FR-061-13** gpuArray inputs are gathered; the result is a host array.
- **FR-061-14** Unobserved classes raise
  `RunMat:lsqminnorm:UnsupportedOption`: logical `A` or `b`, char, integer
  or other non-float `b`, other classes, `NaN`/`Inf` entries, and more than
  three arguments. N-D `A` or `b` raises `RunMat:lsqminnorm:InvalidInput`.
- **FR-061-15** `tol` must be a finite non-negative real double/single
  scalar (absolute threshold, D-061-3), else `UnsupportedOption`.
- **FR-061-16** The result is single when either input is single, and
  complex when either input is complex (no demotion when the imaginary
  part is zero). Single is computed in double and rounded.

### Functional Requirements — `sprank` (normative)

- **FR-061-18** `r = sprank(A)` MUST be accepted [sprank.signature-primary].
- **FR-061-19** Only the nonzero pattern counts: `sparse(ones(2))` gives 2
  [sprank.structural].
- **FR-061-20** `r` is the size of a maximum matching of the pattern, with
  the observed values `speye(3)` → 3, a zero row → 2,
  `[1 1 0; 0 0 1; 0 0 1]` → 2, 2×3 `[1 1 0; 0 1 1]` → 2, 4×2
  `[1 0; 1 0; 1 0; 0 1]` → 2 [sprank.matching].
- **FR-061-21** All-zero and 0×0 sparse matrices give 0
  [sprank.zero-empty].
- **FR-061-22** Full double, sparse logical and sparse complex inputs are
  accepted [sprank.inputs].
- **FR-061-23** `r` is a double scalar [sprank.class].
- **FR-061-24** Char or cell `A` MUST raise
  `RunMat:math:mustBeFloatOrLogical` [sprank.error-char].
- **FR-061-25** No input MUST raise `RunMat:minrhs` [sprank.error-no-arg].

### Functional Requirements — `sprank` (documented choices)

- **FR-061-26** Full logical, full complex, single and scalar inputs are
  accepted and read by their nonzero pattern; `NaN` counts as a nonzero;
  trailing singleton dimensions are allowed.
- **FR-061-27** Integer-class, string and other unobserved classes, and a
  second argument, raise `RunMat:sprank:UnsupportedOption`; N-D input
  raises `RunMat:sprank:InvalidInput`; more than one output raises
  `RunMat:sprank:TooManyOutputs`.
- **FR-061-28** gpuArray input is gathered. The matching is computed with
  Hopcroft–Karp and an iterative search (no recursion depth limit).

### Functional Requirements — `sprandn` (normative)

- **FR-061-29** `R = sprandn(m, n, density)` MUST be accepted and return a
  real sparse double `m`-by-`n` matrix with finite nonzeros, of both signs
  in the observed cases, identical for the same `rng` seed
  [sprandn.signature-primary, sprandn.structure].
- **FR-061-30** `nnz(R) <= density*m*n` [sprandn.nnz-bound].
- **FR-061-31** Density 0 gives an all-zero sparse matrix; a zero-row size
  gives an empty sparse 0×3 [sprandn.zero-density].
- **FR-061-32** `sprandn(S)` returns a sparse matrix of the size of `S`
  with exactly its nonzero pattern [sprandn.pattern].
- **FR-061-33** `sprandn(4, 4, 0.5, 0.1)` returns a 4×4 sparse double with
  2-norm condition number 10 [sprandn.rc].
- **FR-061-34** Non-integer `m` MUST raise
  `RunMat:RandStream:randi:invalidLimits` [sprandn.error-noninteger].
- **FR-061-35** `sprandn(m, n)` MUST raise `RunMat:sprandn:TwoInputs`
  [sprandn.error-two-args].

### Functional Requirements — `sprandn` (documented choices)

- **FR-061-36** (documented choice for the non-normative
  `sprandn.density-out-of-range`) Densities outside [0, 1] do not error;
  they are clamped to [0, 1] (so 1.5 fills the matrix and −0.1 gives no
  nonzeros). `NaN` density raises `UnsupportedOption`.
- **FR-061-37** (non-normative, `sprandn.values`, `sprandn.q-values`,
  `sprandn.q-density`) Values and positions are not specified; RunMat's
  construction is D-061-5 (count `floor(density*m*n)`).
- **FR-061-38** Negative, `NaN` or `Inf` `m`/`n` also raise
  `RunMat:RandStream:randi:invalidLimits`. No input, or an `S` that is not
  a numeric/logical matrix, raises `RunMat:sprandn:InvalidInput`. Integer
  or single `m`/`n`/`density`/`rc`, a typename, a vector or out-of-range
  `rc`, more than four arguments and sizes beyond RunMat's bounded storage
  raise `RunMat:sprandn:UnsupportedOption`. More than one output raises
  `RunMat:sprandn:TooManyOutputs`.
- **FR-061-39** gpuArray inputs are gathered.

### Functional Requirements (unresolved, all three)

- **FR-061-17 / FR-061-40** (unresolved, `*.q-warning-coverage`) None of
  the three functions emits a warning (also for rank-deficient
  `lsqminnorm` input, where none was recorded).

## Success Criteria *(mandatory)*

- **SC-061-1**: all observed cases reproduce (class, size, value or error
  identifier; structural properties for `sprandn`): 17 for `lsqminnorm`,
  14 for `sprank`, 11 for `sprandn`.
- **SC-061-2**: the three builtins are registered through
  `#[runtime_builtin]`, and `cargo test -p runmat-runtime --lib --
  lsqminnorm_tests sprank_tests sprandn_tests` passes.
- **SC-061-3**: every FR maps to at least one test (`validation.md`).

## Unresolved behaviour (returned to the specification side)

| id | disposition |
|---|---|
| lsqminnorm.q-warning-coverage, sprandn.q-warning-coverage, sprank.q-warning-coverage | no warning emitted (FR-061-17/40) |
| lsqminnorm.q-tol-rule, lsqminnorm.tol-rule | documented choice D-061-3 (pivoted QR truncated at `|R(k,k)| > tol`), reproduces the observed case |
| lsqminnorm.q-exactness | tests compare with a tolerance (1e-12 double, 1e-6 single) |
| lsqminnorm.q-error-ids, sprandn.q-error-ids, sprank.q-error-ids | observed identifiers reproduced in the RunMat namespace (D-061-1); flagged to maintainer |
| sprandn.q-values, sprandn.values | structure, class, bound and seeded repeatability only (D-061-5) |
| sprandn.q-density | count `floor(density*m*n)` (D-061-5) |
| sprandn.density-out-of-range | clamp (FR-061-36) |
