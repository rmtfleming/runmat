# Feature Specification: numeric N6 — `normest`, `lsqnonneg`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`067-numeric-n6`; export `SPECIFY_FEATURE=067-numeric-n6`)

**Created**: 2026-11-02

**Status**: Batch N6, feature F67. Gates G1–G3 pre-approved by the
maintainer (`specs/001-clean-room-matlab-builtins/selection-proposal-n6.md`).

**Input**: Approved exports `normest` **0.2.0** and `lsqnonneg` **0.2.0**
(N6, source `e364206`); see `provenance.md`.

Family rationale (Principle VII): two numeric builtins from the COBRA
backlog. COBRA calls `normest(S)` on a sparse stoichiometric matrix and
`lsqnonneg(C, d)` for nonnegative least squares. They share no state. Each
has its own file, tests and requirement block.

## Decisions

- **D-067-1 Error identifier namespace.** Observed `MATLAB:x:y`
  identifiers are raised as `RunMat:x:y` (maintainer rule):
  `RunMat:mtimes:integerNotSupported`, `RunMat:transpose:NDArray`,
  `RunMat:minrhs`, `RunMat:lsqnonneg:NonDoubleInput`,
  `RunMat:lsqnonneg:ComplexCorD`, `RunMat:sizeDimensionsMustMatch`,
  `RunMat:dimagree`, `RunMat:lsqnonneg:InputArg`. `mtimes`, `transpose`,
  `sizeDimensionsMustMatch` and `dimagree` name other functions or generic
  checks; whether they must be reproduced is open on the spec side
  (`normest.q-error-ids`, `lsqnonneg.q-error-ids`). RunMat follows the
  observation (maintainer disposition: "raised as observed under
  `RunMat:`"). Identifiers that were not observed (`UnsupportedOption`,
  `InvalidInput`, `TooManyOutputs`, `Internal`) use `RunMat:<fn>:<name>`.
- **D-067-2 `normest` algorithm and default tolerance (documented choice
  for `normest.q-exactness`, `normest.q-default-tol`; maintainer
  disposition "accuracy only").** RunMat applies the power method to `S'*S`
  (Golub & Van Loan, *Matrix Computations*, 4th ed., §7.3.1, §8.6) as one
  product with `S` and one with `S'` per step; sparse input stays in
  compressed-column form (never densified). The start vector holds the
  column sums of `|S|` (the column of largest norm if `S` maps that vector
  to zero). The estimate is `||S*x||` for unit `x`; iteration stops when two
  successive estimates differ by at most `tol` times the current one, or
  after 10000 steps (no warning). The **default `tol` is `1e-8`**: with the
  stopping rule above, `1e-6` would not guarantee the observed accuracy
  bound (2.3e-7) on slowly converging inputs such as `[3 0; 0 4]`. `cnt` is
  the number of steps; it is not compared with the observed counts.
- **D-067-3 `normest` unobserved inputs.** Char input is accepted and
  computed on its character codes (following the non-normative observation
  `normest.char`; `normest.q-char` remains open). Logical input, strings and
  other non-numeric classes, an invalid `tol` (not a positive finite real
  scalar) and more than two arguments raise
  `RunMat:normest:UnsupportedOption`. A matrix with `Inf` and no `NaN`
  gives `Inf`.
- **D-067-4 `lsqnonneg` algorithm (documented choice; `lsqnonneg.q-optimality`,
  `lsqnonneg.q-nonunique`).** The active-set NNLS algorithm of Lawson &
  Hanson, *Solving Least Squares Problems* (1974), ch. 23; each passive-set
  subproblem is solved as a minimum-norm least-squares problem by nalgebra's
  SVD. The optimality threshold on the gradient `C'*(d - C*x)` is `TolX`
  when given, else `10*eps*max(m,n)*norm(C,1)*norm(d,inf)`. Where the
  minimiser is not unique, the one returned is whatever the algorithm
  reaches; tests assert the objective and the KKT conditions, not `x`
  (maintainer disposition).
- **D-067-5 `lsqnonneg` output structure (documented choice for
  `lsqnonneg.q-output-struct`).** Fields in order `iterations` (outer
  iterations, double), `algorithm` (`'active-set'`, a description of the
  method) and `message` (RunMat's own sentence). `exitflag` is 1 when the
  optimality conditions hold, 0 if the iteration limit `3*n + 10` is
  reached (not observed).
- **D-067-6 `lsqnonneg` densifies sparse `C`** (performance, not
  behaviour): memory `O(m*n)`. No sparse NNLS solver exists in RunMat.
- **D-067-7 Placement.** `normest` in `math/linalg/solve/` (hand-written
  `mod.rs`) next to `norm` and `condest`; `lsqnonneg` in `math/optim/`
  (hand-written `mod.rs`) next to `lsqnonlin`, reading `optimset` /
  `optimoptions` structures. No generated parent is touched.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Estimate a matrix 2-norm (Priority: P1)

**Acceptance Scenarios** (normative, one per observed case; values compared
with RunMat's own `norm(full(S))` within the observed bound 2.3e-7, not digit
for digit, `normest.q-exactness`):

1. `[nrm, cnt] = normest(S)` with the sparse 4×5 stoichiometric matrix →
   full double `nrm = 2` (to rounding), double `cnt` (`sparse-stoich`).
   [normest.signature-primary, normest.cobra-form, normest.output-class]
2. Dense 3×3 `[4 1 0; 1 3 1; 0 1 2]`, dense 2×4, `[3 0; 0 4]` (one output),
   `[3 4 12]`, `[3; 4; 12]`, `speye(5)`, complex `[1 1i; 2 0]` agree with
   `norm(full(S))` within 2.3e-7 (`dense-square`, `dense-wide`, `one-out`,
   `row-vector`, `column-vector`, `identity`, `complex`).
   [normest.default-accuracy, normest.output-class]
3. `single([1 2; 3 4])` → single scalar within the single tolerance
   (`single`). [normest.output-class, normest.default-accuracy]
4. On the dense 3×3 matrix, `tol = 1e-1` gives a larger relative difference
   than the default, `tol = 1e-12` a smaller one (`tol-coarse`, `tol-fine`).
   [normest.tol]
5. `zeros(3,2)`, `sparse(3,3)` and `[]` → 0 (`zero`, `sparse-zero`,
   `empty`). [normest.zero]
6. `normest(-5)` → 5; `normest([1 NaN; 0 1])` → NaN (`scalar`,
   `with-nan`). [normest.scalar-nan]
7. `int32` input → `RunMat:mtimes:integerNotSupported`; `ones(2,2,2)` →
   `RunMat:transpose:NDArray`; no input → `RunMat:minrhs` (`err-int`,
   `err-3d`, `err-no-arg`). [normest.error-int, normest.error-3d,
   normest.error-no-arg]
8. *(non-normative)* `normest('ab')` → `norm(double('ab'))`
   (`char-input`). [normest.char]

### User Story 2 - Nonnegative least squares (Priority: P1)

**Acceptance Scenarios** (normative, one per observed case; tolerance
comparisons, `lsqnonneg.q-exactness`):

1. COBRA form: `lsqnonneg(C, zeros(3,1))` with the 3×4 COBRA matrix → 4×1
   zero vector with one or six outputs; `resnorm 0`, residual zeros,
   `exitflag 1`, lambda zeros (`cobra-zero-rhs`, `cobra-one-out`).
   [lsqnonneg.signature-primary, lsqnonneg.cobra-form]
2. `eye(2)`, `[1; 2]` → `[1; 2]`; `eye(2)`, `[1; -2]` → `[1; 0]`, residual
   `[0; -2]`, resnorm 4, lambda `[0; -2]`; `eye(3)`, `[-1; -2; -3]` → 0,
   resnorm 14; `[2 0; 0 4]`, `[2; 4]` → `[1; 1]`; `[1 0; 0 1; 1 1]`,
   `[1; 2; -5]` → 0, residual `[1; 2; -5]`, resnorm 30, lambda `[-4; -3]`
   (`interior`, `bound-active`, `all-negative`, `one-out`,
   `overdetermined`). [lsqnonneg.solutions, lsqnonneg.residual,
   lsqnonneg.lambda]
3. `x` is a full double column, also for sparse `C` (`interior`, `sparse`).
   [lsqnonneg.output-class]
4. `exitflag = 1` and `output.iterations` exists in every case
   [lsqnonneg.output-struct].
5. `optimset('TolX', 1e-3)` and `optimset('Display', 'off')` are accepted and
   change nothing on the observed problems (`options-tolx`,
   `options-display`). [lsqnonneg.options]
6. `zeros(0,2)`, `zeros(0,1)` → `x = [0; 0]`, resnorm 0, 0×1 residual
   (`empty-rows`). [lsqnonneg.empty]
7. *(non-normative)* `[1 1; 1 1]`, `[2; 2]`: the minimiser is not unique;
   RunMat's result has `resnorm ≈ 0` and satisfies the KKT conditions
   (`rank-deficient`). [lsqnonneg.nonunique, lsqnonneg.lambda]
8. Errors: single `C` and `d`, or char `C` →
   `RunMat:lsqnonneg:NonDoubleInput`; complex `C` →
   `RunMat:lsqnonneg:ComplexCorD`; a row mismatch →
   `RunMat:sizeDimensionsMustMatch`; a row-vector `d` → `RunMat:dimagree`;
   `C` only → `RunMat:lsqnonneg:InputArg` (`err-single`, `err-char`,
   `err-complex`, `err-size`, `err-row-d`, `err-one-arg`).

## Requirements *(mandatory)*

### Functional Requirements — `normest` (normative)

- **FR-067-01** `nrm = normest(S)`, `[nrm, cnt] = normest(S)` and
  `[nrm, cnt] = normest(S, tol)` MUST be accepted [normest.signature-primary].
- **FR-067-02** `nrm` is a full scalar, double for double or sparse input
  and single for single input; `cnt` is double [normest.output-class].
- **FR-067-03** With the default tolerance, `nrm` agrees with
  `norm(full(S))` to a relative difference of at most 2.3e-7 for the
  observed matrices [normest.default-accuracy].
- **FR-067-04** The sparse 4×5 COBRA matrix gives `nrm = 2` to rounding
  [normest.cobra-form].
- **FR-067-05** On the observed 3×3 matrix, `tol = 1e-1` gives a larger and
  `tol = 1e-12` a smaller relative difference than the default
  [normest.tol].
- **FR-067-06** All-zero full or sparse input gives 0; `0×0` gives 0
  [normest.zero].
- **FR-067-07** A scalar gives its absolute value; a matrix containing NaN
  gives NaN [normest.scalar-nan].
- **FR-067-08** Integer-class `S` MUST raise
  `RunMat:mtimes:integerNotSupported` [normest.error-int].
- **FR-067-09** `S` with more than two dimensions MUST raise
  `RunMat:transpose:NDArray` [normest.error-3d].
- **FR-067-10** No input MUST raise `RunMat:minrhs` [normest.error-no-arg].

### Functional Requirements — `normest` (non-normative and documented choices)

- **FR-067-11** (non-normative `normest.char`; documented choice D-067-3)
  Char input is accepted and computed on its character codes.
- **FR-067-12** (non-normative `normest.iteration-count`) `cnt` is RunMat's
  step count: 0 for zero/empty/non-finite input, positive otherwise, and it
  grows as `tol` decreases on the observed matrix. Values are not compared
  with the observed counts.
- **FR-067-13** (documented choice D-067-2) Power method on `S'*S`, default
  `tol = 1e-8`, at most 10000 steps.
- **FR-067-14** (documented choice D-067-3) Logical or non-numeric `S`, an
  invalid `tol`, and more than two arguments raise
  `RunMat:normest:UnsupportedOption`; more than two outputs raise
  `RunMat:normest:TooManyOutputs`; `Inf` (without NaN) gives `Inf`.
- **FR-067-15** (documented choice) Sparse input is not densified; gpuArray
  input is gathered.
- **FR-067-16** (unresolved, `normest.q-warning-coverage`) No warning is
  emitted.

### Functional Requirements — `lsqnonneg` (normative)

- **FR-067-20** `x = lsqnonneg(C, d)`, the six-output form and the
  `options` form MUST be accepted [lsqnonneg.signature-primary].
- **FR-067-21** `x` is a full double column vector, also for sparse `C`
  [lsqnonneg.output-class].
- **FR-067-22** The COBRA form `lsqnonneg(C, zeros(3,1))` gives the zero
  vector, resnorm 0, zero residual, exitflag 1, zero lambda
  [lsqnonneg.cobra-form].
- **FR-067-23** The observed solutions are reproduced (User Story 2,
  scenario 2) [lsqnonneg.solutions].
- **FR-067-24** `residual = d - C*x` and `resnorm = norm(residual)^2`
  [lsqnonneg.residual].
- **FR-067-25** `lambda = C'*(d - C*x)`; 0 for components with `x > 0`,
  `<= 0` for components at 0, up to rounding [lsqnonneg.lambda].
- **FR-067-26** `exitflag = 1` and `output` has a field `iterations`
  [lsqnonneg.output-struct].
- **FR-067-27** `optimset('TolX', 1e-3)` and `optimset('Display', 'off')`
  are accepted and give the same results on the observed problems
  [lsqnonneg.options].
- **FR-067-28** `C` 0×2, `d` 0×1 gives `x = [0; 0]`, resnorm 0 and a 0×1
  residual [lsqnonneg.empty].
- **FR-067-29** Single `C` and `d`, or char `C`, MUST raise
  `RunMat:lsqnonneg:NonDoubleInput` [lsqnonneg.error-single].
- **FR-067-30** Complex `C` MUST raise `RunMat:lsqnonneg:ComplexCorD`
  [lsqnonneg.error-complex].
- **FR-067-31** A `d` with a different number of rows from `C` MUST raise
  `RunMat:sizeDimensionsMustMatch` [lsqnonneg.error-size].
- **FR-067-32** A row-vector `d` MUST raise `RunMat:dimagree`
  [lsqnonneg.error-row-d].
- **FR-067-33** A call with only `C` MUST raise `RunMat:lsqnonneg:InputArg`
  [lsqnonneg.error-one-arg]. (No input raises the same identifier:
  documented choice.)

### Functional Requirements — `lsqnonneg` (non-normative and documented choices)

- **FR-067-34** (non-normative `lsqnonneg.nonunique`; D-067-4) Where the
  minimiser is not unique, any `x >= 0` with the minimal objective that
  satisfies the KKT conditions is acceptable; RunMat's is the one the
  Lawson–Hanson iteration reaches.
- **FR-067-35** (non-normative `lsqnonneg.iterations`) `output.iterations`
  is RunMat's outer-iteration count; values are not compared.
- **FR-067-36** (documented choice) Mixed double/single, integer, logical
  and other classes raise `RunMat:lsqnonneg:NonDoubleInput` (complex
  `d` raises `ComplexCorD`); N-D `C` or `d` raises
  `RunMat:lsqnonneg:InvalidInput`; NaN/Inf entries raise
  `RunMat:lsqnonneg:UnsupportedOption`.
- **FR-067-37** (documented choice for `lsqnonneg.q-options`) Options are a
  structure (from `optimset` or `optimoptions`) or `[]`. `TolX` (alias
  `StepTolerance`, finite `>= 0`) is the optimality threshold; `Display`
  `'off'`, `'none'`, `'final'`, `'notify'`, `'iter'` is accepted and nothing
  is printed; `Solver` is ignored; empty fields are ignored. Any other
  non-empty field, another `Display` value, a non-structure third argument
  (e.g. an `x0` vector) and more than three arguments raise
  `RunMat:lsqnonneg:UnsupportedOption`.
- **FR-067-38** (documented choice D-067-5) More than six outputs raise
  `RunMat:lsqnonneg:TooManyOutputs`; `exitflag` is 0 at the iteration
  limit.
- **FR-067-39** (documented choice D-067-5, `lsqnonneg.q-output-struct`)
  `output.algorithm = 'active-set'`, `output.message` is RunMat's sentence.
- **FR-067-40** (documented choice) gpuArray inputs are gathered; sparse `C`
  is densified (D-067-6).
- **FR-067-41** (unresolved, `lsqnonneg.q-warning-coverage`) No warning is
  emitted.

## Success Criteria *(mandatory)*

- **SC-067-1**: all observed cases reproduce (20 for `normest`, 18 for
  `lsqnonneg`): class, size, value within tolerance, or error identifier.
- **SC-067-2**: both builtins are registered through `#[runtime_builtin]`,
  and `cargo test -p runmat-runtime --lib -- normest_tests lsqnonneg_tests`
  passes.
- **SC-067-3**: every FR maps to at least one test (`validation.md`).

## Unresolved behaviour (returned to the specification side)

| id | disposition |
|---|---|
| normest.q-warning-coverage, lsqnonneg.q-warning-coverage | no warning emitted (FR-067-16/41) |
| normest.q-exactness | tolerance comparisons with RunMat's `norm(full(S))`; `cnt` not compared (FR-067-03, 12) |
| normest.q-default-tol | default `tol = 1e-8` (D-067-2) |
| normest.q-char | char accepted on codes (D-067-3) |
| normest.q-error-ids, lsqnonneg.q-error-ids | observed identifiers under `RunMat:` (D-067-1); flagged to maintainer |
| lsqnonneg.q-exactness | tolerance comparisons |
| lsqnonneg.q-output-struct | fields present; values RunMat's (D-067-5) |
| lsqnonneg.q-optimality | KKT conditions asserted on larger problems (D-067-4) |
| lsqnonneg.q-nonunique | objective and KKT asserted, not `x` (FR-067-34) |
| lsqnonneg.q-options | FR-067-37 |
