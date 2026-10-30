# Feature Specification: `validateattributes` — class and attribute validation

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`038-validateattributes`; export `SPECIFY_FEATURE=038-validateattributes`)

**Created**: 2026-10-29 (rewritten 2026-10-29 for spec 0.3.0)

**Status**: Batch C2, resumed after the W2 widening. Implemented from the
approved export `validateattributes` **0.3.0**. The earlier 0.2.0 scope
("class + `'scalar'` only") is **superseded** by maintainer decision (a).
Decision D2 = B is **retained** (decision (b), below). Gates 1–3
pre-approved by the maintainer; no commit.

**Input**: Approved Tier B export `validateattributes` 0.3.0 (see
`provenance.md`).

## Maintainer decisions recorded in this feature

- **(a)** Implement every attribute and invocation form 0.3.0 confirms.
- **(b) D2 = B, identifier mapping.** Every error raised by RunMat uses
  `RunMat:validateattributes:<suffix>`, where `<suffix>` is the **last
  component** of the observed identifier. The observed `MATLAB:…`
  identifiers, including the `funcName` middle component
  (`MATLAB:myfun:expectedScalar`), are **non-normative**. `funcName`,
  `varName` and `argIndex` may change only the message text, never the
  identifier. Mapping table: see "Identifier mapping" below.
- **(c)** Points 0.3.0 leaves unresolved raise
  `RunMat:validateattributes:UnsupportedOption` or are labelled documented
  choices (see "Unresolved behaviour").

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Valid input passes silently (Priority: P1)

`validateattributes(A, classes, attributes)` returns without error, without
output and without a warning when `A` satisfies the classes and attributes.

**Acceptance Scenarios** (normative): the observed pass cases `pass`,
`pass-empty-attrs`, `pass-class-list`, `pass-class-char`,
`pass-class-logical`, `pass-class-cell`, `pass-class-struct`,
`pass-class-fh`, `pass-row`, `pass-positive`, `pass-nonnegative`,
`pass-integer-double`, `pass-increasing`, `pass-nondecreasing`, `pass-gt`,
`pass-ge`, `pass-numel`, `pass-size-nan`, `pass-extra-args` all return with
no error and no warning. [validateattributes.signature-primary,
validateattributes.pass-silent, validateattributes.class-list,
validateattributes.empty-attributes, validateattributes.attribute-semantics]

### User Story 2 - Each failing check raises its identifier (Priority: P1)

**Acceptance Scenarios** (normative that the error is raised; identifier per
decision (b)): every observed failing case raises
`RunMat:validateattributes:<suffix>` as listed in the identifier mapping
(cases `empty-class-list`, `fail-type`, `fail-class-list`, `fail-scalar` …
`fail-ndims`, `fail-first-of-two`, `fail-second-of-two`, `fail-extra-args`,
`fail-extra-varname`, `returns-nothing`).
[validateattributes.error-*, validateattributes.each-attribute-checked,
validateattributes.funcname-identifier]

### User Story 3 - Unknown attribute warns (Priority: P2)

`validateattributes(1, {'numeric'}, {'notanattribute'})` returns without
error and emits the warning `RunMat:validateattributes:attributeNotFound`.
[validateattributes.warning-unknown-attribute]

### User Story 4 - Unspecified options fail clearly (Priority: P2)

Every point 0.3.0 leaves unresolved, and every argument form it does not
confirm, raises `RunMat:validateattributes:UnsupportedOption` unless listed
as a documented choice (see "Unresolved behaviour").

## Identifier mapping (decision (b))

| Observed identifier (non-normative) | RunMat identifier | Condition |
|---|---|---|
| `MATLAB:invalidType`, `MATLAB:myfun:invalidType` | `RunMat:validateattributes:invalidType` | class not in list; empty class list |
| `MATLAB:expectedScalar`, `MATLAB:myfun:expectedScalar` | `…:expectedScalar` | `'scalar'` |
| `MATLAB:expectedVector` | `…:expectedVector` | `'vector'` |
| `MATLAB:expectedRow` | `…:expectedRow` | `'row'` |
| `MATLAB:expectedColumn` | `…:expectedColumn` | `'column'` |
| `MATLAB:expected2D` | `…:expected2D` | `'2d'` |
| `MATLAB:expectedNonempty` | `…:expectedNonempty` | `'nonempty'` |
| `MATLAB:expectedSquare` | `…:expectedSquare` | `'square'` |
| `MATLAB:expectedPositive` | `…:expectedPositive` | `'positive'` |
| `MATLAB:expectedNonnegative` | `…:expectedNonnegative` | `'nonnegative'` |
| `MATLAB:expectedInteger` | `…:expectedInteger` | `'integer'` |
| `MATLAB:expectedFinite` | `…:expectedFinite` | `'finite'` |
| `MATLAB:expectedNonNaN` | `…:expectedNonNaN` | `'nonnan'` |
| `MATLAB:expectedReal` | `…:expectedReal` | `'real'` |
| `MATLAB:expectedNonZero` | `…:expectedNonZero` | `'nonzero'` |
| `MATLAB:expectedNonsparse` | `…:expectedNonsparse` | `'nonsparse'` |
| `MATLAB:expectedBinary` | `…:expectedBinary` | `'binary'` |
| `MATLAB:expectedEven` | `…:expectedEven` | `'even'` |
| `MATLAB:expectedOdd` | `…:expectedOdd` | `'odd'` |
| `MATLAB:expectedIncreasing` | `…:expectedIncreasing` | `'increasing'` |
| `MATLAB:expectedDecreasing` | `…:expectedDecreasing` | `'decreasing'` |
| `MATLAB:expectedNonincreasing` | `…:expectedNonincreasing` | `'nonincreasing'` |
| *(not observed)* | `…:expectedNondecreasing` | `'nondecreasing'` (documented choice, U-038-7) |
| `MATLAB:notGreater` | `…:notGreater` | `'>', b` |
| `MATLAB:notGreaterEqual` | `…:notGreaterEqual` | `'>=', b` |
| `MATLAB:notLess` | `…:notLess` | `'<', b` |
| `MATLAB:notLessEqual` | `…:notLessEqual` | `'<=', b` |
| `MATLAB:incorrectNumel` | `…:incorrectNumel` | `'numel', n` |
| `MATLAB:incorrectSize` | `…:incorrectSize` | `'size', sz` |
| `MATLAB:incorrectNumcols` | `…:incorrectNumcols` | `'ncols', n` |
| `MATLAB:incorrectNumrows` | `…:incorrectNumrows` | `'nrows', n` |
| `MATLAB:incorrectNumdims` | `…:incorrectNumdims` | `'ndims', n` |
| `MATLAB:maxlhs` | `…:maxlhs` | an output is requested |
| warning `MATLAB:validateattributes:attributeNotFound` | warning `RunMat:validateattributes:attributeNotFound` | unrecognised attribute name |
| *(RunMat only)* | `…:UnsupportedOption` | unresolved point / unconfirmed form |
| *(RunMat only)* | `…:InvalidArgument` | malformed call |

## Requirements *(mandatory)*

### Functional Requirements

Normative (from 0.3.0 claims):

- **FR-038-01** The three confirmed forms `validateattributes(A, classes,
  attributes)`, `…, funcName, varName)` and `…, funcName, varName,
  argIndex)` MUST be accepted; on success they return no output, no error
  and no warning [validateattributes.signature-primary,
  validateattributes.pass-silent].
- **FR-038-02** `A` passes the class check when its class (RunMat's
  `class()`) equals any listed class name; observed for `single` in
  `{'double','single'}` and for `char`, `logical`, `cell`, `struct` and
  `function_handle` [validateattributes.class-list]. A class not in the
  list MUST raise `invalidType` [validateattributes.error-fail-type]. An
  **empty class list** MUST raise `invalidType`
  [validateattributes.error-empty-class-list].
- **FR-038-03** An empty attribute list imposes no checks
  [validateattributes.empty-attributes].
- **FR-038-04** Every listed attribute is enforced
  [validateattributes.each-attribute-checked]; each failing attribute
  raises the RunMat identifier in the mapping table
  [validateattributes.error-fail-scalar … validateattributes.error-fail-ndims].
- **FR-038-05** Attribute semantics confirmed: `'integer'` accepts
  integer-valued doubles; `'nonnegative'` accepts zero; `'nondecreasing'`
  accepts repeated values and `'increasing'` does not; `'>'` is strict and
  `'>='` inclusive (likewise `'<'` strict: `fail-lt` at equality); NaN in a
  `'size'` vector matches any extent
  [validateattributes.attribute-semantics, validateattributes.error-fail-lt].
- **FR-038-06** Requesting an output MUST raise `maxlhs`
  [validateattributes.error-returns-nothing]. Implemented through
  `current_output_count()`. Known gap: through the dispatcher, a retry of
  another registered implementation loses the requested count, so the error
  is not yet raised end-to-end (validation.md D-1).
- **FR-038-07** With `funcName` given, the failing check still raises the
  same RunMat identifier (decision (b)); `funcName`, `varName`, `argIndex`
  appear only in the message text [validateattributes.funcname-identifier,
  validateattributes.error-fail-extra-args,
  validateattributes.error-fail-extra-varname].
- **FR-038-08** An unrecognised attribute name MUST emit the warning
  `RunMat:validateattributes:attributeNotFound` (through RunMat's warning
  facility, `crate::warnings::emit`) and the call returns without error
  [validateattributes.warning-unknown-attribute].

Documented choices (independently derived; not MATLAB-conformance claims):

- **FR-038-09** Shape attributes use RunMat's `size()`: `scalar` all
  extents 1; `vector` 2-D with one extent equal to 1; `row` 2-D with 1 row;
  `column` 2-D with 1 column; `2d` `ndims == 2`; `square` 2-D with equal
  extents; `nonempty` `numel > 0`; `nonsparse` not a sparse value. `numel`,
  `nrows` (`size(A,1)`), `ncols` (`size(A,2)`), `ndims` compare exactly;
  `size` compares element-wise after padding both vectors with trailing 1s,
  NaN matching any extent.
- **FR-038-10** Value attributes (`positive`, `nonnegative`, `integer`,
  `finite`, `nonnan`, `real`, `nonzero`, `binary`, `even`, `odd`, the four
  monotonic attributes, `>`, `>=`, `<`, `<=`) apply to numeric classes and,
  as a documented choice, to `logical` (values 0/1). Integer classes are
  compared exactly. NaN fails every ordering and value test except
  `nonzero` and `real`; `finite` fails on NaN and ±Inf; `integer` fails on
  ±Inf; `even`/`odd` fail on non-integers; empty `A` passes every value
  attribute.
- **FR-038-11** Monotonic attributes compare consecutive elements of a
  vector, scalar or empty `A`; `nondecreasing` failure raises
  `expectedNondecreasing` (U-038-7).
- **FR-038-12** Complex `A`: `real` fails; `finite`, `nonnan`, `nonzero`
  test both parts; every other value attribute is `UnsupportedOption`.
  Shape attributes are unaffected.
- **FR-038-13** Check order: requested outputs (`maxlhs`), then argument
  parsing (`InvalidArgument`/`UnsupportedOption`), then class, then
  attributes in list order; the first failure is reported (q-check-order).
  Unknown-attribute warnings are emitted when reached in that order.
- **FR-038-14** Argument forms: 3–6 arguments, else `InvalidArgument`.
  `classes`: cell of text (confirmed) or, as a documented choice, a string
  array; a bare char vector is `UnsupportedOption`; other types or non-text
  cell entries are `InvalidArgument`. `'numeric'` matches `double`,
  `single` and the eight integer classes (documented choice; only `double`
  observed); categories `'float'` and `'integer'` are `UnsupportedOption`.
  `attributes`: a cell array; text that is not a cell is
  `UnsupportedOption`; other types `InvalidArgument`. `funcName` and
  `varName` are text, `argIndex` a positive integer-valued real scalar;
  any other form is `UnsupportedOption`.
- **FR-038-15** Value arguments: `>`, `>=`, `<`, `<=` take a real numeric
  (or logical) scalar; `numel`, `nrows`, `ncols`, `ndims` take a
  non-negative integer-valued real scalar; `size` takes a real numeric
  vector of non-negative integers or NaN. Any other value form is
  `UnsupportedOption`; a missing value is `InvalidArgument`.
- **FR-038-16** GPU-resident inputs are classified by RunMat's `class()`
  and `size()` and gathered for value attributes.

## Unresolved behaviour (non-normative; returned to the specification side)

| id | question | disposition |
|---|---|---|
| validateattributes.q-check-order | which identifier when several attributes fail | documented choice: first failing attribute in list order (FR-038-13) |
| validateattributes.q-more-attributes | value attributes on non-numeric classes | `UnsupportedOption` for char, string, cell, struct, function handle, object etc.; `logical` documented choice (FR-038-10) |
| validateattributes.q-more-attributes | `'diag'`, `'scalartext'` (named as unexercised) | `UnsupportedOption` |
| validateattributes.q-more-attributes | `'decreasing'` pass cases | documented choice: strict decrease passes (FR-038-11) |
| q-warning-coverage (manifest) | the warnings list is not complete | only `attributeNotFound` is emitted; no other warning is invented |
| U-038-1 (local) | `funcName`/`varName`/`argIndex` message effect | message text only; forms per FR-038-14 |
| U-038-2 (local) | bare char `classes`; non-cell text `attributes` | `UnsupportedOption` |
| U-038-3 (local) | categories `'float'`, `'integer'` | `UnsupportedOption` |
| U-038-4 (local) | case variants of known attribute names (e.g. `'Scalar'`) | `UnsupportedOption` (not silently warned and skipped) |
| U-038-5 (local) | non-text entry where an attribute name is expected | `UnsupportedOption` |
| U-038-6 (local) | monotonic attributes on matrices; complex value attributes other than real/finite/nonnan/nonzero; integer-complex values | `UnsupportedOption` |
| U-038-7 (local) | identifier for a `'nondecreasing'` failure | documented choice `expectedNondecreasing` |
| U-038-8 (local) | malformed attribute values (FR-038-15) | `UnsupportedOption` |

## Success Criteria *(mandatory)*

- **SC-038-1**: all 58 observed cases reproduce (pass silently, warn, or
  raise the mapped RunMat identifier) in normative tests.
- **SC-038-2**: the builtin is registered through `#[runtime_builtin]`, and
  `cargo test -p runmat-runtime --lib -- validateattributes` passes.
- **SC-038-3**: every FR maps to at least one test (validation.md).

## Assumptions

- `class_name_for_value`, `value_dimensions` and the warning facility are
  RunMat's own; no new runtime types are needed.
