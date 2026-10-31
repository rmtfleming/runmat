# Feature Specification: types, structs and text — `isfloat`, `isnat`, `struct2cell`, `strvcat`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`054-types-structs-text`; export `SPECIFY_FEATURE=054-types-structs-text`)

**Created**: 2026-10-30

**Status**: Batches N3+N4, feature F54. Gates G1–G3 pre-approved by the
maintainer (`specs/001-clean-room-matlab-builtins/selection-proposal-n3n4.md`).

**Input**: Approved exports `isfloat`, `struct2cell` and `strvcat` **0.2.0**
(N3, IMPORT-MANIFEST Batch 11, `1156c97`) and `isnat` **0.2.0** (N4,
IMPORT-MANIFEST Batch 13, `fc9f804`). See `provenance.md`.

Family rationale (Principle VII): four small value-level helpers from the
same batches — two class/element predicates (`isfloat`, `isnat`) and two
container conversions (`struct2cell`, `strvcat`). They share no state. Each
has its own file, tests and requirement block.

## Decisions

- **D-054-1 Error identifier namespace.** Observed `MATLAB:x:y` identifiers
  are raised as `RunMat:x:y`: `RunMat:minrhs`, `RunMat:maxrhs`,
  `RunMat:character:InvalidInput` and `RunMat:invalidConversion`.
  Identifiers that were not observed (`UnsupportedOption`, `TooManyOutputs`,
  `InvalidInput`, `UndefinedFunction`, `Internal`) use `RunMat:<fn>:<name>`.
- **D-054-2 Dispatch-level errors are non-normative.** `struct2cell` of a
  non-struct and `isnat` of a non-datetime were observed to fail with
  `MATLAB:UndefinedFunction` at dispatch level (claims
  `struct2cell.non-struct`, `isnat.non-datetime`, `normative: false`).
  RunMat raises `RunMat:struct2cell:UndefinedFunction` /
  `RunMat:isnat:UndefinedFunction` from the builtin (documented choice); the
  identifier is not presented as MATLAB's.
- **D-054-3 `isfloat` reuses RunMat's class names.** `isfloat(A)` is true
  exactly when RunMat's `class(A)` (`class_name_for_value`) is `double` or
  `single`.
- **D-054-4 `isnat` reads RunMat's datetime storage.** A RunMat datetime is
  an object holding a tensor of serial date numbers. A NaT element is stored
  as `NaN`. `isnat` reads the serials through
  `crate::builtins::datetime::serials_from_datetime_value` and returns
  `isnan` of them, with the same shape.
- **D-054-8 `strvcat` reuses `char`.** Each accepted input (and each cell
  element) is turned into a char matrix by RunMat's `char` builtin;
  `strvcat` then drops empty inputs, stacks the rows and pads them with
  blanks.
- **D-054-5 Unobserved points** raise `RunMat:<fn>:UnsupportedOption` or are
  labelled documented choices below. Neither kind is presented as MATLAB
  behaviour.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Is the value floating point? (Priority: P1)

1. `isfloat(1)`, `isfloat(single(1))`, `isfloat(1+2i)`, `isfloat(NaN)`,
   `isfloat([])`, `isfloat(sparse(1))`, `isfloat(ones(2,3))` → logical
   `true` 1×1 (`double`, `single`, `complex`, `nan`, `empty`, `sparse`,
   `matrix`). [isfloat.signature-primary, isfloat.true-cases]
2. `isfloat(int8(1))`, `isfloat(uint64(1))`, `isfloat(true)`,
   `isfloat('a')`, `isfloat("a")`, `isfloat({1})`, `isfloat(struct('a',1))`,
   `isfloat(@sin)` → logical `false` 1×1 (`int8`, `uint64`, `logical`,
   `char`, `string`, `cell`, `struct`, `handle`). [isfloat.false-cases]
3. `isfloat()` raises `RunMat:minrhs` (`err-none`); `isfloat(1,2)` raises
   `RunMat:maxrhs` (`err-two`). [isfloat.error-none, isfloat.error-two]

### User Story 2 - Which datetime elements are NaT? (Priority: P1)

1. `isnat(NaT)` → `true` 1×1 (`nat`); `isnat(datetime(2020,1,2))` → `false`
   (`valid`); `isnat(datetime('NaT'))` → `true` (`from-text-nat`).
   [isnat.signature-primary, isnat.elementwise]
2. A 1×3 datetime `[valid NaT valid]` → logical `[false true false]` 1×3
   (`array`); a 2×2 `[valid NaT; valid valid]` → `[false true; false false]`
   (`matrix`). [isnat.elementwise]
3. A 0×1 datetime → 0×1 logical (`empty`). [isnat.empty]

### User Story 3 - Struct to cell (Priority: P1)

1. `struct2cell(struct('b',1,'a','x','c',{{2}}))` → 3×1 cell `{1; 'x'; {2}}`
   in field-definition order (`scalar`); a matrix value and a nested struct
   are kept whole (`matrix-value`, `nested`). [struct2cell.signature-primary,
   struct2cell.scalar]
2. Sizes: 1×2 struct with 2 fields → 2×1×2 (`array-size`); 2×1 with 1 field
   → 1×2 (`column-array-size`); 2×2 with 1 field → 1×2×2
   (`matrix-array-size`); 0×0 with 2 fields → 2×0×0 (`empty-array`).
   [struct2cell.array-size]
3. For `struct('a',{1,2},'b',{3,4})`, `C(:)` is `{1;3;2;4}` (`array-size`).
   [struct2cell.array-order]
4. `struct2cell(struct())` → 0×1 cell (`no-fields`). [struct2cell.no-fields]

### User Story 4 - Stack text vertically (Priority: P1)

1. `strvcat('a','bcd','ef')` → `['a  ';'bcd';'ef ']`, 3×3 (`pad`,
   `pad-size`); `strvcat('a  ','b')` → `['a  ';'b  ']` (`trailing-blanks`).
   [strvcat.signature-primary, strvcat.pad]
2. `size(strvcat('a','','b'))` → `[2 1]` (`skip-empty`);
   `size(strvcat({'a','','b'}))` → `[2 1]` (`cell-empty`).
   [strvcat.skip-empty]
3. `strvcat({'ab','c'})` → `['ab';'c ']` (`cell`). [strvcat.cell]
4. `strvcat(['ab';'cd'],'e')` → `['ab';'cd';'e ']` (`matrix-arg`).
   [strvcat.matrix-arg]
5. `strvcat(65,'b')` → `['A';'b']` (`numeric`). [strvcat.numeric]
6. `strvcat("ab","c")` → `['ab';'c ']` (`string`). [strvcat.string]
7. `strvcat()` and `strvcat('','')` → 0×0 char (`no-args`, `all-empty`).
   [strvcat.empty]
8. `strvcat('abc')` → `'abc'` 1×3 (`one`). [strvcat.one]
9. `strvcat('x',{'yy','z'})` raises `RunMat:character:InvalidInput`
   (`err-mixed`). [strvcat.error-mixed]
10. `strvcat(struct('a',1))` raises `RunMat:invalidConversion`
    (`err-struct`). [strvcat.error-struct]

## Requirements *(mandatory)*

### Functional Requirements — `isfloat` (normative)

- **FR-054-01** `tf = isfloat(A)` MUST be accepted and return a logical
  scalar [isfloat.signature-primary].
- **FR-054-02** The result is `true` for double and single values, including
  complex, NaN, empty, sparse and matrix inputs (always a scalar result)
  [isfloat.true-cases].
- **FR-054-03** The result is `false` for int8, uint64, logical, char,
  string, cell (even holding a double), struct and function handle
  [isfloat.false-cases].
- **FR-054-04** No input MUST raise `RunMat:minrhs` [isfloat.error-none];
  two inputs MUST raise `RunMat:maxrhs` [isfloat.error-two].

### Functional Requirements — `isnat` (normative)

- **FR-054-05** `tf = isnat(D)` MUST be accepted [isnat.signature-primary].
- **FR-054-06** Elementwise: a NaT element (including one made by
  `datetime('NaT')`) gives `true`, a valid datetime `false`; the result is a
  logical array of the shape of `D` (1×1, 1×3 and 2×2 observed)
  [isnat.elementwise].
- **FR-054-07** A 0×1 datetime gives a 0×1 logical [isnat.empty].

### Functional Requirements — `struct2cell` (normative)

- **FR-054-08** `C = struct2cell(S)` MUST be accepted
  [struct2cell.signature-primary].
- **FR-054-09** A scalar struct gives an n×1 cell of its field values in
  field-definition order; values are kept whole [struct2cell.scalar].
- **FR-054-10** The size is `[numfields, size(S)]` with trailing singleton
  dimensions dropped (at least two dimensions remain)
  [struct2cell.array-size].
- **FR-054-11** In column-major order the fields vary fastest:
  `C(f, i) = S(i).(field f)` [struct2cell.array-order].
- **FR-054-12** A scalar struct with no fields gives a 0×1 cell
  [struct2cell.no-fields].

### Functional Requirements — `strvcat` (normative)

- **FR-054-13** `S = strvcat(t1, t2, ...)` and `S = strvcat(c)` MUST be
  accepted and return a char matrix with one row per non-empty text row,
  padded with trailing blanks to the longest row; trailing blanks of an
  input are kept [strvcat.signature-primary, strvcat.pad].
- **FR-054-14** Empty char inputs, also inside a cell, are skipped
  [strvcat.skip-empty].
- **FR-054-15** A cell array of char gives one row per element
  [strvcat.cell].
- **FR-054-16** A multi-row char input contributes each of its rows
  [strvcat.matrix-arg].
- **FR-054-17** A numeric input is taken as character codes
  [strvcat.numeric].
- **FR-054-18** String scalars are accepted and give char rows
  [strvcat.string].
- **FR-054-19** No inputs, or only empty inputs, give a 0×0 char
  [strvcat.empty].
- **FR-054-20** A single char row is returned unchanged [strvcat.one].
- **FR-054-21** A char input mixed with a cell input MUST raise
  `RunMat:character:InvalidInput` [strvcat.error-mixed].
- **FR-054-22** A struct input MUST raise `RunMat:invalidConversion`
  [strvcat.error-struct].

### Functional Requirements (documented choices and unresolved points)

- **FR-054-23** (documented choice, all four) More than one requested output
  raises `RunMat:<fn>:TooManyOutputs`. `isnat` and `struct2cell` take
  exactly one input; any other count raises `RunMat:<fn>:InvalidInput`.
- **FR-054-24** (documented choice, isfloat) Classes not observed follow
  RunMat's `class` (D-054-3): every integer class, objects, datetime and
  the other non-float classes give `false`; a complex `single` gives
  `true`. An explicit `gpuArray` value reports class `gpuArray` in RunMat,
  so `isfloat` of it gives `false`; a device-resident value that RunMat
  moved implicitly reports its underlying class.
- **FR-054-25** (documented choice, isnat; D-054-2) A non-datetime input
  (double, duration, char, …) raises `RunMat:isnat:UndefinedFunction`.
- **FR-054-26** (documented choice, struct2cell; D-054-2) A non-struct input
  (cell, double, …) raises `RunMat:struct2cell:UndefinedFunction`. An empty
  struct array of any shape gives a cell of shape `[numfields, size(S)]`
  (trailing singletons dropped).
- **FR-054-27** (documented choice, strvcat) Inputs not covered by the
  observations. Each non-cell input, and each cell element, is converted
  with RunMat's `char` builtin on its own (D-054-8); an input whose
  conversion has no elements is skipped, and the rows of every other input
  are kept. Hence:
  - empty string scalars `""` and empty numeric inputs are skipped like
    empty char;
  - a string array contributes one row per element (column-major), as
    `char` gives;
  - numeric inputs (double, single, integer classes) are converted as by
    `char`, including its errors (`RunMat:char:*`) for non-finite codes and
    complex or more-than-2-D inputs;
  - a cell element that is not char or a string scalar raises
    `RunMat:strvcat:UnsupportedOption`; cell elements are taken in
    column-major order;
  - several cell inputs together are accepted; any cell input together with
    a non-cell input raises `RunMat:character:InvalidInput` (extension of
    `err-mixed`), checked before any conversion;
  - logical inputs raise `RunMat:strvcat:UnsupportedOption`;
  - a struct array (as well as a scalar struct) raises
    `RunMat:invalidConversion`; every other class (function handle,
    object, …) raises `RunMat:strvcat:UnsupportedOption`;
  - gpuArray inputs are gathered first.
- **FR-054-28** (unresolved, `*.q-warning-coverage` for all four) None of
  the four functions emits a warning.

## Deviations (upstream gaps)

- **D-054-6 No `NaT` builtin in RunMat.** The observed cases `nat`, `array`
  and `matrix` call `NaT`, and `empty` calls `datetime.empty(0,1)`. RunMat
  has neither, and adding them belongs to the datetime subsystem (outside
  this feature's scope). The tests build the same values through RunMat's
  own datetime constructor (`datetime_object_from_serial_tensor`, a `NaN`
  serial for NaT) and assert the observed `isnat` results on them.
- **D-054-7 `datetime('NaT')`.** If RunMat's `datetime` constructor rejects
  the text `'NaT'`, the `from-text-nat` case cannot be reproduced end to
  end; its test is then kept `#[ignore]`d, and the `isnat` side is covered
  by the internally built NaT. The B4 result is recorded in
  `validation.md`.

## Success Criteria *(mandatory)*

- **SC-054-1**: all 50 observed cases reproduce (class, size, value or
  error identifier), except where D-054-6/7 require the input to be built
  internally: isfloat 17, isnat 9 (3 non-normative), struct2cell 10 (2
  non-normative), strvcat 14 (`pad-size`, `skip-empty` and `cell-empty`
  are checked through the result size).
- **SC-054-2**: the four builtins are registered through `#[runtime_builtin]`
  and `cargo test -p runmat-runtime --lib -- isfloat_tests isnat_tests
  struct2cell_tests strvcat_tests` passes.
- **SC-054-3**: every FR maps to at least one test (`validation.md`).

## Unresolved behaviour (returned to the specification side)

| id | disposition |
|---|---|
| *.q-warning-coverage (4) | no warning emitted (FR-054-28) |
| struct2cell.non-struct, isnat.non-datetime (non-normative claims) | `RunMat:<fn>:UndefinedFunction` (D-054-2) |
