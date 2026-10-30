# Feature Specification: numeric conversion — `nthroot`, `reallog`, `cast`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`050-numeric-conversion`; export `SPECIFY_FEATURE=050-numeric-conversion`)

**Created**: 2026-10-30

**Status**: Batch N2, feature F50. Gates G1–G3 pre-approved by the maintainer
(`specs/001-clean-room-matlab-builtins/selection-proposal-n2.md`).

**Input**: Approved N2 exports `nthroot`, `reallog` and `cast`, all
**0.2.0** (see `provenance.md`).

Family rationale (Principle VII): three small elementwise numeric helpers
from the same approved batch. `nthroot` and `reallog` are the real-only
counterparts of `power`/`log`; `cast` converts between numeric, logical and
char classes. They share no state. Each has its own file, tests and
requirement block.

## Decisions

- **D-050-1 Error identifier namespace.** Observed `MATLAB:x:y` identifiers
  are raised as `RunMat:x:y` (maintainer rule). This gives
  `RunMat:nthroot:NegXNotOddIntegerN`, `RunMat:nthroot:ComplexInput`,
  `RunMat:sizeDimensionsMustMatch`, `RunMat:reallog:complexResult`,
  `RunMat:validators:mustBeReal`, `RunMat:math:mustBeFloat`,
  `RunMat:cast:UnsupportedClass` and `RunMat:invalidConversion`.
  Identifiers that were not observed (`InvalidInput`, `UnsupportedOption`,
  `TooManyOutputs`, `Internal`) use `RunMat:<fn>:<name>`.
- **D-050-2 `cast` reuses RunMat's conversion builtins.** `cast(A, cls)`
  validates `cls` and `A`, then calls the registered conversion builtin of
  that name (`double`, `single`, `int8` … `uint64`, `logical`, `char`).
  Rounding, saturation, NaN/Inf handling and GPU residency of `A` are those
  of the existing builtins; nothing is re-implemented.
- **D-050-3 `cast(x,'like',complex)` (flagged, `cast.q-like-complexity`).**
  The normative claim `cast.like` says the result is a *real* double `2`.
  The imported observation record for case `like-complex` carries
  `isreal: false` for the same result. The two artefacts disagree, and the
  spec side leaves the question open. RunMat follows the claim text: `'like'`
  takes only the class of `p`, and a real `A` gives a real result. Tests
  check only class (`double`) and value (`2`), which both readings agree on.
  The disagreement is returned to the specification side.
- **D-050-4 Unobserved points** raise `RunMat:<fn>:UnsupportedOption` or are
  labelled documented choices below. Neither kind is presented as MATLAB
  behaviour.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Real n-th roots (Priority: P1)

**Acceptance Scenarios** (normative, one per observed case):

1. `nthroot(-8,3)` → double `-2` (`cube-neg`). [nthroot.signature-primary,
   nthroot.output-class, nthroot.negative-odd]
2. `nthroot(16,4)` → `2` within 1e-12 (`fourth`). [nthroot.even-root]
3. `nthroot([1 8 27 -64],3)` → `[1 2 3 -4]` 1×4 (`vector`).
   [nthroot.negative-odd]
4. `nthroot([4 27 16],[2 3 4])` → `[2 3 2]` (`vector-n`).
   [nthroot.elementwise-n]
5. `nthroot([0 Inf -Inf NaN],3)` → `[0 Inf -Inf NaN]` (`specials`).
   [nthroot.specials]
6. `nthroot(-5,1)` → `-5`; `nthroot(8,-3)` → `0.5`; `nthroot(8,2.5)` →
   `2.2973967099940702` (`n-one`, `n-negative`, `noninteger-n`).
   [nthroot.n-values]
7. `nthroot(single(27),3)` → single `3` (`single`). [nthroot.output-class]
8. `nthroot(1000,3) == 10` is true (`exactness`). [nthroot.exact]
9. `nthroot(-16,4)` and `nthroot(-8,2.5)` raise
   `RunMat:nthroot:NegXNotOddIntegerN` (`err-even-negative`,
   `err-noninteger-n-neg`). [nthroot.error-even-negative]
10. `nthroot(1i,3)` raises `RunMat:nthroot:ComplexInput` (`err-complex`).
    [nthroot.error-complex]
11. `nthroot([1 2 3],[2 3])` raises `RunMat:sizeDimensionsMustMatch`
    (`err-size`). [nthroot.error-size]

### User Story 2 - Real-only logarithm (Priority: P1)

1. `reallog(1)` → `0`; `reallog(exp(2))` → `2`; `reallog([1 10 100])` →
   `[0 log(10) log(100)]` within 1e-12 (`one`, `e`, `vector`).
   [reallog.signature-primary, reallog.values]
2. `reallog(0)` → `-Inf`; `reallog(Inf)` → `Inf`; `reallog(NaN)` → `NaN`
   (`zero`, `inf`, `nan`). [reallog.specials]
3. `reallog([1 2;3 4])` keeps 2×2; `reallog(single(4))` is single;
   `reallog([])` → 0×0 double (`matrix`, `single`, `empty`).
   [reallog.shape-class]
4. `reallog(-1)` raises `RunMat:reallog:complexResult` (`err-negative`).
   [reallog.error-negative]
5. `reallog(1+1i)` raises `RunMat:validators:mustBeReal` (`err-complex`).
   [reallog.error-complex]
6. `reallog(int8(4))` raises `RunMat:math:mustBeFloat` (`err-int`).
   [reallog.error-int]

### User Story 3 - Convert between classes (Priority: P1)

1. `cast([1.5 2.5 -1.5],'int32')` → int32 `[2 3 -2]` (`round`).
   [cast.signature-primary, cast.rounding]
2. `cast([-1 300 255.6],'uint8')` → `[0 255 255]`;
   `cast(int16([-200 5 200]),'int8')` → `[-128 5 127]` (`saturate`,
   `int-to-int`). [cast.saturation]
3. `cast([NaN Inf -Inf],'int8')` → `[0 127 -128]` (`nan-inf`).
   [cast.nan-inf]
4. `cast(1/3,'single')` → single `0.3333333432674408`;
   `cast('ab','double')` → `[97 98]`; `cast([true false],'double')` →
   `[1 0]`; `cast([0 2],'logical')` → `[false true]`; `cast([72 105],'char')`
   → `'Hi'` (`to-single`, `char-to-double`, `logical-to-double`,
   `to-logical`, `to-char`). [cast.other-classes]
5. `cast(70000,'like',uint16(1))` → uint16 `65535` (`like`);
   `cast(2,'like',1i)` → double `2` (`like-complex`, D-050-3).
   [cast.like, cast.saturation]
6. `cast([1 2;3 4],'uint8')` keeps 2×2; `cast(int8(5),'int8')` → int8 `5`
   (`shape`, `same-class`). [cast.shape]
7. `cast(1,'notaclass')` and `cast(1,'struct')` raise
   `RunMat:cast:UnsupportedClass` (`err-bad-type`, `err-struct-type`).
   [cast.error-bad-type]
8. `cast({1},'double')` raises `RunMat:invalidConversion` (`err-cell`).
   [cast.error-cell]

## Requirements *(mandatory)*

### Functional Requirements — `nthroot` (normative)

- **FR-050-01** `y = nthroot(x, n)` MUST be accepted
  [nthroot.signature-primary].
- **FR-050-02** The result is real, with the class of `x` (double; single
  for single `x`) [nthroot.output-class].
- **FR-050-03** Odd integer roots of negative values are real and negative
  [nthroot.negative-odd]; even roots of positive values are the positive
  real root [nthroot.even-root].
- **FR-050-04** `n` may be an array of the same size as `x`, applied
  elementwise [nthroot.elementwise-n].
- **FR-050-05** `0`, `Inf`, `-Inf`, `NaN` with `n = 3` give `0`, `Inf`,
  `-Inf`, `NaN` [nthroot.specials].
- **FR-050-06** `n = 1` returns `x`; negative `n` gives the reciprocal
  root; non-integer `n` with non-negative `x` gives `x^(1/n)`
  [nthroot.n-values].
- **FR-050-07** `nthroot(1000,3)` MUST equal `10` exactly
  [nthroot.exact].
- **FR-050-08** A negative `x` with `n` not an odd integer MUST raise
  `RunMat:nthroot:NegXNotOddIntegerN` [nthroot.error-even-negative].
- **FR-050-09** Complex `x` MUST raise `RunMat:nthroot:ComplexInput`
  [nthroot.error-complex].
- **FR-050-10** Incompatible sizes of `x` and `n` MUST raise
  `RunMat:sizeDimensionsMustMatch` [nthroot.error-size].

### Functional Requirements — `reallog` (normative)

- **FR-050-11** `y = reallog(x)` MUST be accepted and return the natural
  logarithm [reallog.signature-primary, reallog.values].
- **FR-050-12** `0` → `-Inf`, `Inf` → `Inf`, `NaN` → `NaN`
  [reallog.specials].
- **FR-050-13** Shape is kept; single gives single; `[]` gives `[]`
  [reallog.shape-class].
- **FR-050-14** A negative element MUST raise
  `RunMat:reallog:complexResult` [reallog.error-negative]; complex input
  MUST raise `RunMat:validators:mustBeReal` [reallog.error-complex];
  integer-class input MUST raise `RunMat:math:mustBeFloat`
  [reallog.error-int].

### Functional Requirements — `cast` (normative)

- **FR-050-15** `B = cast(A, newclass)` and `B = cast(A, 'like', p)` MUST be
  accepted [cast.signature-primary, cast.like].
- **FR-050-16** Conversion to an integer class rounds to nearest with halves
  away from zero [cast.rounding], saturates out-of-range values
  [cast.saturation], and maps NaN → 0, Inf → intmax, -Inf → intmin
  [cast.nan-inf].
- **FR-050-17** double → single rounds to single precision; char → double
  gives character codes; logical → double gives 1/0; double → logical gives
  nonzero → true; double → char gives characters [cast.other-classes].
- **FR-050-18** `'like' p` converts to the class of `p`. A complex double
  `p` gives a double result with the value of `A` (D-050-3) [cast.like].
- **FR-050-19** The shape is kept; casting to the same class leaves the
  value unchanged [cast.shape].
- **FR-050-20** An unknown class name, or `'struct'`, MUST raise
  `RunMat:cast:UnsupportedClass` [cast.error-bad-type]. A cell `A` MUST
  raise `RunMat:invalidConversion` [cast.error-cell].

### Functional Requirements (documented choices and unresolved points)

- **FR-050-21** (documented choice, all three) Arity: `nthroot` takes
  exactly 2 inputs, `reallog` exactly 1, `cast` 2 or 3; anything else raises
  `RunMat:<fn>:InvalidInput`. More than one requested output raises
  `RunMat:<fn>:TooManyOutputs`.
- **FR-050-22** (documented choice, nthroot) Size rules beyond the observed
  cases:
  - equal shapes, or a scalar on either side, are accepted (a scalar `x`
    with an array `n` gives the shape of `n`);
  - shapes that differ but would be compatible under implicit expansion
    (e.g. row against column) raise `RunMat:nthroot:UnsupportedOption`;
  - all other mismatches raise `RunMat:sizeDimensionsMustMatch`
    (extension of `err-size`).
- **FR-050-23** (documented choice, nthroot) Classes:
  - `x` must be double or single (scalar or dense array); integer, logical,
    char, sparse and other classes raise `RunMat:nthroot:UnsupportedOption`;
  - `n` must be real double or single; any other class (including complex
    `n`) raises `RunMat:nthroot:UnsupportedOption`;
  - the result class follows `x` only (FR-050-02), also for single `n`.
- **FR-050-24** (documented choice, nthroot) Algorithm and edge cases:
  `x^(1/n)` on `|x|`, followed by one Newton correction step when the root
  is finite and non-zero; the sign of `x` is restored for odd roots. `n = 0`,
  `n = ±Inf` and `n = NaN` with non-negative `x` follow that arithmetic; with
  negative `x` they raise `NegXNotOddIntegerN` (not an odd integer). Empty
  `x` gives an empty result of the same shape.
- **FR-050-25** (documented choice, reallog) Logical, char, sparse and other
  classes raise `RunMat:reallog:UnsupportedOption`.
- **FR-050-26** (documented choice, nthroot/reallog) gpuArray inputs are
  gathered and the result is a host array.
- **FR-050-27** (documented choice, cast) Argument validation:
  - `newclass` (and the `'like'` keyword) may be a char row or a string
    scalar; class names are matched exactly (lower case), so `'INT8'` raises
    `UnsupportedClass`; `'like'` is matched case-insensitively;
  - every class name outside `double single int8 int16 int32 int64 uint8
    uint16 uint32 uint64 logical char` raises `RunMat:cast:UnsupportedClass`
    (extension of `err-bad-type`/`err-struct-type`);
  - a non-text `newclass`, or a 3-argument call whose second argument is not
    `'like'`, raises `RunMat:cast:UnsupportedOption`;
  - the class is checked before `A`;
  - `A` of class string, struct, object, function handle or other
    non-numeric/non-logical/non-char classes raises
    `RunMat:cast:UnsupportedOption`.
- **FR-050-28** (documented choice, cast) `'like' p`: only the class of `p`
  is used. Sparsity, complexity and GPU residency of `p` are not
  propagated; `p` is gathered before its class is read. A `p` whose class is
  not in the supported list raises `RunMat:cast:UnsupportedClass`.
- **FR-050-29** (documented choice, cast) Behaviour not covered by the
  observations — complex `A`, sparse `A`, gpuArray `A`, errors such as
  `logical(NaN)` — is whatever the delegated conversion builtin does
  (D-050-2), including its error identifiers.
- **FR-050-30** (unresolved, `*.q-warning-coverage` for all three) None of
  the three functions emits a warning.

## Success Criteria *(mandatory)*

- **SC-050-1**: all 42 observed cases reproduce (class, size, value or
  error identifier): 14 for nthroot, 12 for reallog and 16 for cast.
- **SC-050-2**: the three builtins are registered through
  `#[runtime_builtin]`, and `cargo test -p runmat-runtime --lib --
  nthroot_tests reallog_tests cast_tests` passes.
- **SC-050-3**: every FR maps to at least one test (`validation.md`).

## Unresolved behaviour (returned to the specification side)

| id | disposition |
|---|---|
| nthroot.q-warning-coverage | no warning emitted (FR-050-30) |
| reallog.q-warning-coverage | no warning emitted (FR-050-30) |
| cast.q-warning-coverage | no warning emitted (FR-050-30) |
| cast.q-like-complexity (flagged) | real result, per the claim text; the observation record's `isreal: false` contradicts the claim and is reported back (D-050-3) |
