# Feature Specification: `findstr` — start indices of the shorter argument in the longer

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`037-findstr`; export `SPECIFY_FEATURE=037-findstr`)

**Created**: 2026-10-29 (revised the same day for spec 0.3.0)

**Status**: Batch C2 — implemented from the approved 0.3.0 export (maintainer
decision 2026-10-29: 0.3.0 supersedes the earlier D1 "strict" choice for
`findstr`, since argument order is now observed). Gates 1–3 pre-approved.

**Input**: Approved Tier B export `findstr` **0.3.0** (see `provenance.md`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Find every occurrence, in either order (Priority: P1)

**Acceptance Scenarios** (normative, one per observed case):

1. `findstr('abcabc', 'bc')` → double `[2 5]` (case `occurrences`).
   [findstr.signature-primary, findstr.output-class, findstr.start-indices]
2. `findstr('bc', 'abcabc')` → `[2 5]` (case `swapped`).
   [findstr.order-independent]
3. `findstr('aaaa', 'aa')` → `[1 2 3]` (case `overlapping`).
   [findstr.overlapping]
4. `findstr('banana', 'a')` → `[2 4 6]` (case `single-char`).
   [findstr.start-indices]
5. `findstr('ab', 'ab')` → `1`; `findstr('ab', 'ba')` → `[]` (cases
   `equal-length-match`, `equal-length-nomatch`). [findstr.equal-length]
6. `findstr("abcabc", "bc")` → `[2 5]` (case `string-scalar`).
   [findstr.string-scalar]
7. `findstr([1 2 3 1 2], [1 2])` → `[1 4]` (case `numeric`).
   [findstr.numeric, findstr.output-class]

### User Story 2 - Empty results (Priority: P1)

1. No occurrence, an empty argument, or both empty give a `0×0` double:
   `('abc','x')`, `('abc','')`, `('','abc')`, `('','')`, `('ABC','b')`
   (cases `no-match`, `empty-pattern`, `empty-first`, `both-empty`, `case`).
   [findstr.no-match, findstr.case-sensitive]
2. An ordinary call issues no warning (case `warn-call`).
   [findstr.no-warning]

### User Story 3 - Errors (Priority: P2)

1. A multi-row char argument raises `RunMat:findstr:MultiRowInput` (case
   `column-char`). [findstr.error-column-char]
2. A cell argument raises `RunMat:findstr:InputsMustBeCharArrays` (case
   `err-cell`). [findstr.error-cell]
3. One argument raises `RunMat:findstr:NotEnoughInputs` (case
   `err-one-arg`). [findstr.error-one-arg]

## Requirements *(mandatory)*

### Functional Requirements (normative)

- **FR-037-01** `k = findstr(s1, s2)` MUST return class double for every
  non-erroring call [findstr.signature-primary, findstr.output-class].
- **FR-037-02** The result MUST be a row of the 1-based start indices of
  every occurrence, in increasing order, overlaps included
  [findstr.start-indices, findstr.overlapping].
- **FR-037-03** The shorter argument MUST be searched for within the longer,
  regardless of argument order [findstr.order-independent]. With equal
  lengths, identical arguments give `1` and different ones give empty
  [findstr.equal-length].
- **FR-037-04** No occurrence, or one or both arguments empty, MUST give a
  `0×0` double [findstr.no-match]. Matching MUST be case-sensitive
  [findstr.case-sensitive].
- **FR-037-05** String scalars MUST behave like the equivalent char rows
  [findstr.string-scalar]. Double row vectors MUST be accepted and matched
  element-wise [findstr.numeric].
- **FR-037-06** An ordinary call MUST issue no warning [findstr.no-warning].
- **FR-037-07** A multi-row char argument MUST raise
  `RunMat:findstr:MultiRowInput` [findstr.error-column-char]. A cell argument
  MUST raise `RunMat:findstr:InputsMustBeCharArrays` [findstr.error-cell].
  One argument MUST raise `RunMat:findstr:NotEnoughInputs`
  [findstr.error-one-arg]. The export records `MATLAB:strfind:MultiRowInput`,
  `MATLAB:findstr:InputsMustBeCharArrays` and `MATLAB:minrhs`. RunMat uses its
  own namespace (see Decisions).

### Functional Requirements (documented choices and unresolved points)

- **FR-037-08** (documented choice) Char and string arguments may be mixed.
  A one-element string array counts as a string scalar. The empty `0×0`
  char is an empty argument.
- **FR-037-09** (documented choice) Numeric arguments are real double: a
  scalar, a row vector, or an empty array (treated as an empty argument). A
  double with more than one row (or extra non-singleton dimensions) raises
  `RunMat:findstr:MultiRowInput`, as multi-row char does. NaN never matches
  (IEEE equality).
- **FR-037-10** (documented choice) Struct, multi-element string array and
  other non-text, non-numeric values raise
  `RunMat:findstr:InputsMustBeCharArrays`, as cell does. More than two
  arguments raise `RunMat:findstr:TooManyInputs`.
- **FR-037-11** (unobserved; UnsupportedOption) Mixing a text argument
  with a numeric one, and logical, integer, single or complex arguments,
  MUST raise `RunMat:findstr:UnsupportedOption` (U-037-1).
- **FR-037-12** (unresolved, findstr.q-deprecated, findstr.q-warning-coverage)
  No deprecation or other warning is emitted. Only the `warn-call` case is
  asserted.

## Decisions

- **Error identifier namespace.** RunMat follows the batch C2 decision D2 = B
  precedent (`specs/001-clean-room-matlab-builtins/selection-proposal-tierc2.md`):
  `RunMat:findstr:<observed suffix>`, with `NotEnoughInputs` for the
  observed `MATLAB:minrhs` (matching RunMat's existing
  `RunMat:…:NotEnoughInputs` convention). **Flag for maintainer review.**

## Success Criteria *(mandatory)*

- **SC-037-1**: all 17 observed cases reproduce (class, size, value or
  error), each in its own `observed_*` test.
- **SC-037-2**: `findstr` is registered through `#[runtime_builtin]` and
  `cargo test -p runmat-runtime --lib -- findstr_tests` passes.
- **SC-037-3**: every FR maps to at least one test (validation.md).

## Unresolved behaviour (returned to the specification side)

| id | question | disposition |
|---|---|---|
| findstr.q-warning-coverage | warnings checked only in `warn-call` | no warnings emitted |
| findstr.q-deprecated | formal deprecation (documentation question) | no warning emitted |
| U-037-1 (local) | mixed text/numeric; logical, integer, single, complex | `UnsupportedOption` |

Resolved by 0.3.0 and superseding the 0.2.0 dispositions: `findstr.q-order`
(was `UnsupportedOption` when `s2` is longer). Also superseded: the 0.2.0-era
choices "no match gives 1×0", "numeric input is InvalidInput" and "extra
arguments are UnsupportedOption".
