# Feature Specification: `stack`, `innerjoin` — reshape and inner-join tables

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`044-table-stack-innerjoin`; export `SPECIFY_FEATURE=044-table-stack-innerjoin`)

**Created**: 2026-10-30

**Status**: Selected in batch N1 (F44,
`specs/001-clean-room-matlab-builtins/selection-proposal-n1.md`). Gates
G1–G3 pre-approved; B1→B5 run straight through, stopping before any commit.

**Input**: Approved exports `stack` 0.2.0 and `innerjoin` 0.2.0
(IMPORT-MANIFEST Batch 9, source `c65e238`; see `provenance.md`).

## User Scenarios & Testing *(mandatory)*

Unless stated otherwise, `T = table([1;2],[10;20],[30;40],'VariableNames',{'id','x','y'})`
for `stack`; for `innerjoin`, `L = table([1;2;4],[10;20;40],'VariableNames',{'k','a'})`
and `R = table([4;1;3],[400;100;300],'VariableNames',{'k','b'})`, passed as
expressions (not named workspace variables) unless a story says otherwise.

### User Story 1 - Stack variables wide to long (Priority: P1)

**Acceptance Scenarios** (normative):

1. `stack(T, {'x','y'})` returns a 4×3 `table` `id, x_y_Indicator, x_y`
   with `id = [1;1;2;2]`, indicator `categorical` values `x,y,x,y` with
   categories `{x,y}`, and `x_y = [10;30;20;40]`. (`basic`, `indicator`)
   [stack.signature-primary, stack.layout, stack.default-names,
   stack.indicator]
2. `stack(T, [2 3])` gives the same table. (`by-index`) [stack.by-index]
3. Three stacked variables give `x_y_z_Indicator` / `x_y_z`, 6 rows, data
   `[10;30;50;20;40;60]`. (`three-vars`) [stack.layout, stack.default-names]
4. `stack(T, 'x')` gives `id, y, x_Indicator, x` (2×4). (`one-var`)
   [stack.single-var]
5. `'NewDataVariableName','val','IndexVariableName','which'` gives
   `id, which, val`; `'ConstantVariables',{}` gives `x_y_Indicator, x_y`.
   (`new-names`, `constant`) [stack.options]
6. An unknown stacked variable fails with `RunMat:table:UnrecognizedVarName`;
   stacking a double with a cell variable fails with
   `RunMat:table:stack:InterleavingDataVarsFailed`. (`err-unknown`,
   `err-class-mix`) [stack.error-unknown, stack.error-class-mix]

### User Story 2 - Inner join two tables (Priority: P1)

1. `innerjoin(L, R)` returns a 2×3 `table` `k, a, b` with `[1;4]`,
   `[10;40]`, `[100;400]`. (`basic`) [innerjoin.signature-primary,
   innerjoin.output-class, innerjoin.rows, innerjoin.single-key]
2. `'Keys','k'` gives the same; `'LeftKeys','k','RightKeys','id'` against a
   right table whose key is `id` gives `k, a, b`. (`keys`, `leftright-keys`)
   [innerjoin.keys]
3. Unsorted inputs give rows in ascending key order. (`unsorted-left`,
   `unsorted-both`) [innerjoin.row-order]
4. Left keys `[1;1;4]` with `a = [11;10;40]` give `k = [1;1;4]`,
   `a = [11;10;40]`, `b = [100;100;400]`. (`dup-left`) [innerjoin.duplicates]
5. A non-key `v` in both tables gives `v_left, v_right` for expression
   inputs and `v_Lw, v_Rw` for inputs held in workspace variables `Lw`, `Rw`.
   (`same-nonkey`, `same-nonkey-named`) [innerjoin.name-suffix]
6. No common key values give a 0×3 table `k, a, b` of doubles.
   (`no-match`) [innerjoin.no-match]
7. `'LeftVariables','k'` gives `k, b`. (`left-variables`)
   [innerjoin.left-variables]
8. Tables with no common variable and no keys fail with
   `RunMat:table:join:CantInferKey`. (`err-no-common`)
   [innerjoin.error-no-common]

### User Story 3 - Unspecified behaviour fails clearly (Priority: P2)

Every question the exports leave open raises `RunMat:<fn>:UnsupportedOption`
or follows a documented RunMat choice (FR-044-20..36). None of these is
asserted as MATLAB behaviour.

## Requirements *(mandatory)*

### Functional Requirements — `stack` (normative)

- **FR-044-01** `S = stack(T, vars)` MUST return a table
  [stack.signature-primary]. Each input row MUST give one output row per
  stacked variable, in input-row order then stacked-variable order; the
  carried variables are repeated [stack.layout].
- **FR-044-02** By default the data variable MUST be named by joining the
  stacked names with `_`, the indicator variable MUST be that name plus
  `_Indicator`, and the output order MUST be: carried variables, indicator,
  data [stack.default-names].
- **FR-044-03** The indicator MUST be `categorical`; its value in each row
  names the source variable and its categories are the stacked names
  [stack.indicator].
- **FR-044-04** `vars` given as indices MUST give the same result as names
  [stack.by-index].
- **FR-044-05** `'NewDataVariableName'` and `'IndexVariableName'` MUST
  rename the data and indicator variables; `'ConstantVariables',{}` MUST
  carry no other variables [stack.options].
- **FR-044-06** Stacking one variable MUST keep the remaining variables in
  table order, followed by `<name>_Indicator` and the data variable named
  after the stacked variable [stack.single-var].
- **FR-044-07** A stacked variable name not in the table MUST fail with
  `RunMat:table:UnrecognizedVarName` [stack.error-unknown]. Stacking a
  double variable with a cell variable MUST fail with
  `RunMat:table:stack:InterleavingDataVarsFailed` [stack.error-class-mix].

### Functional Requirements — `innerjoin` (normative)

- **FR-044-10** `T = innerjoin(Tleft, Tright)` MUST return a table
  [innerjoin.signature-primary, innerjoin.output-class].
- **FR-044-11** Only rows whose key appears in both tables are kept
  [innerjoin.rows]; output rows MUST be in ascending key order whatever the
  input order [innerjoin.row-order].
- **FR-044-12** The key MUST appear once, followed by the left non-key
  variables, then the right non-key variables [innerjoin.single-key].
- **FR-044-13** `'Keys'` by name MUST give the same result as inference;
  `'LeftKeys'`/`'RightKeys'` MUST join differently named keys and the
  output keeps the left key name [innerjoin.keys].
- **FR-044-14** Each matching left row MUST produce its own row; equal keys
  keep left-input order [innerjoin.duplicates].
- **FR-044-15** A non-key name present in both tables MUST be made unique
  by appending `_` plus the caller's variable name of each input; when the
  inputs are not named variables the suffixes MUST be `_left` and `_right`
  [innerjoin.name-suffix]. The recovery mechanism is FR-044-34.
- **FR-044-16** With no common key values the result MUST be a 0-row table
  with all variables [innerjoin.no-match].
- **FR-044-17** `'LeftVariables'` MUST select the left variables in the
  output; `'LeftVariables','k'` gives `k, b` [innerjoin.left-variables].
- **FR-044-18** No common variable and no key options MUST fail with
  `RunMat:table:join:CantInferKey` [innerjoin.error-no-common].

Observed `MATLAB:` identifiers are raised in the RunMat namespace with the
same tail (`RunMat:table:UnrecognizedVarName`,
`RunMat:table:stack:InterleavingDataVarsFailed`,
`RunMat:table:join:CantInferKey`). Message text is not reproduced.

### Unresolved questions (non-normative for MATLAB; normative for RunMat)

- **FR-044-20** (`stack.q-second-output`) `[S, iT] = stack(...)` (more than
  one output) MUST raise `RunMat:stack:UnsupportedOption`.
- **FR-044-21** (`stack.q-groups`) `vars` given as a cell containing cell
  arrays (several groups) MUST raise `RunMat:stack:UnsupportedOption`.
- **FR-044-22** (`stack.q-warning-coverage`, `innerjoin.q-warning-coverage`)
  Neither builtin emits warnings. No warning behaviour is asserted.
- **FR-044-23** (`innerjoin.q-index-outputs`) More than one requested output
  MUST raise `RunMat:innerjoin:UnsupportedOption`.
- **FR-044-24** (`innerjoin.q-many-to-many`) A key value that occurs more
  than once in both tables MUST raise `RunMat:innerjoin:UnsupportedOption`.
- **FR-044-34** (`innerjoin.q-name-suffix-context`; documented choice,
  flagged for human review) Same mechanism as F35 `outerjoin` (FR-035-17):
  the suffix of each input is that argument's source text at the recorded
  call site, used only when (a) a call site is recorded, (b) it has exactly
  as many arguments as this call, (c) both table arguments are plain
  identifiers and (d) the identifiers differ. Otherwise the suffixes are
  `_left`/`_right`, the export's observed suffixes for expression inputs.

### Documented choices (unobserved; RunMat behaviour only)

`stack`:

- **FR-044-25** U-044-1: categories of the indicator are the stacked names
  in the order they were listed (the observed cases list them in sorted
  order, so both readings agree on them). The indicator is non-ordinal and
  non-protected.
- **FR-044-26** U-044-2: with `'NewDataVariableName'` but no
  `'IndexVariableName'`, the indicator name is the default joined name plus
  `_Indicator` (the rule of FR-044-02 read literally), not the new data name
  plus `_Indicator`.
- **FR-044-27** U-044-3: `'ConstantVariables'` (names, indices, string or
  cell array) carries exactly the listed variables, in table order. Listing
  a stacked variable raises `RunMat:stack:InvalidArgument`; an unknown name
  raises `RunMat:table:UnrecognizedVarName`.
- **FR-044-28** U-044-4: stacked variables of one class are interleaved for
  real numeric (same storage class), logical, string and cell variables with
  the same number of columns. Any other mix of classes (for example double
  with single, int32 or logical, or string with cellstr), differing column
  counts, and any other class (char, complex, categorical, datetime,
  duration, nested tables) raise `RunMat:stack:UnsupportedOption`. The
  observed error `RunMat:table:stack:InterleavingDataVarsFailed` is raised
  only when the stacked variables include a `double` and a `cell`
  (FR-044-07); it is not generalised to other mixes.
- **FR-044-29** U-044-5: a table with row names raises
  `RunMat:stack:UnsupportedOption`; a timetable raises `UnsupportedOption`;
  any other non-table first input raises `RunMat:stack:InvalidInput`. The
  output carries RunMat's default table properties.
- **FR-044-30** U-044-6: `vars` may be a char row, string scalar or array,
  cellstr, or positive integer indices. An empty or repeated list, an index
  out of range, or a non-integer index raises `RunMat:stack:InvalidArgument`;
  a logical mask raises `UnsupportedOption`. Option names are matched
  case-insensitively; a dangling or unknown option, or a new name that is
  not text, raises `InvalidArgument`. Output names that collide raise
  `RunMat:stack:InvalidArgument`.

`innerjoin`:

- **FR-044-31** U-044-7 (key placement): the output starts with the left
  key variables in key order (FR-044-12 read literally), then the other
  selected left variables in left-table order, then the right non-key
  variables in right-table order. The observed cases all have the key as
  the first left variable, so this is unobserved when it is not.
- **FR-044-32** U-044-8 (`LeftVariables`): accepts the key-list forms of
  FR-044-35. A key left out of `LeftVariables` does not appear in the
  output. `'RightVariables'` (not in the export) raises
  `RunMat:innerjoin:UnsupportedOption`.
- **FR-044-33** U-044-9 (key classes): as F35 FR-035-16/18 — keys of the
  same class are supported for real numeric classes, logical, string and
  cellstr single columns; a key pair of different classes, any other key
  class, a NaN key and a `<missing>` string key raise
  `RunMat:innerjoin:UnsupportedOption`. A key duplicated only in the right
  table gives one row per right row, in right-input order.
- **FR-044-35** U-044-10 (options): option names case-insensitive; key
  lists are a name, a cell or string array of names, or indices. A dangling
  or unknown option, an empty or repeated key list, mismatched
  `LeftKeys`/`RightKeys` counts or only one of them, a `'Keys'` index naming
  different variables, raise `RunMat:innerjoin:InvalidArgument`; `'Keys'`
  with `'LeftKeys'`/`'RightKeys'` raises `RunMat:innerjoin:ConflictingInputs`;
  an unknown key or `LeftVariables` name or index raises
  `RunMat:innerjoin:UnknownVariable`; a logical selector raises
  `UnsupportedOption`.
- **FR-044-36** U-044-11: inputs must be tables (`RunMat:innerjoin:InvalidInput`;
  timetables `UnsupportedOption`). The output carries RunMat's default table
  properties (row names, units, descriptions are not carried). A name that
  still collides after suffixing gets `_2`, `_3`, … (as F35 U-035-9).

## Success Criteria

- **SC-044-1** Every one of the 9 `stack` and 11 `innerjoin` observation
  cases is reproduced by a test (names, order, row order, class, values).
- **SC-044-2** `builtins::table` suite stays green; clippy clean on the new
  files.
- **SC-044-3** `validation.md` traces every claim → FR → test → result.

## Known upstream gap

D-034-1 (a table with rows but no variables cannot be represented) does not
arise: every `stack` output has at least the indicator and data variables,
and every `innerjoin` output has at least one right or left variable. A
`LeftVariables` selection that empties the left side while the right has no
non-key variables would need a variable-less table; RunMat raises
`RunMat:innerjoin:UnsupportedOption` for it.
