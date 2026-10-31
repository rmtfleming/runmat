# Feature Specification: `renamevars`, `rows2vars`, `varfun`, `convertvars` — table variable transforms

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`056-table-variables`; export `SPECIFY_FEATURE=056-table-variables`)

**Created**: 2026-10-30

**Status**: Selected in batches N3+N4 (F56,
`specs/001-clean-room-matlab-builtins/selection-proposal-n3n4.md`). Gates
G1–G3 pre-approved; B1→B5 run straight through, stopping before any commit.

**Input**: Approved exports `renamevars` 0.2.0 and `rows2vars` 0.2.0
(IMPORT-MANIFEST Batch 11, source `1156c97`), `varfun` 0.2.0 and
`convertvars` 0.2.0 (IMPORT-MANIFEST Batch 13, source `fc9f804`). See
`provenance.md`.

## User Scenarios & Testing *(mandatory)*

Unless stated otherwise `T = table([1;2],[3;4],[5;6],'VariableNames',{'a','b','c'})`
for `renamevars`; `P = table({'a';'b'},[1;2],{'x';'y'},'VariableNames',{'p','n','q'})`
for `convertvars`; `X = table([1;2;3],[4;6;8],'VariableNames',{'x','y'})` for
`varfun`. Bracketed names are claim ids; parenthesised names are
observation cases.

### User Story 1 - Rename table variables (P1)

1. `renamevars(T,'b','B')` gives `a, B, c` with unchanged data (`one`);
   `{'a','c'}`→`{'x','z'}` gives `x, b, z` (`many`); index `2`, index vector
   `[3 1]` with `{'Z','X'}` (gives `X, b, Z`), logical mask
   `[false true false]` and string arrays `["a" "b"]`→`["p" "q"]` work
   (`index`, `index-many`, `logical`, `string`). [renamevars.signature-primary,
   renamevars.selection, renamevars.keeps-data]
2. Row names are kept (`rownames`); renaming to the same name is a no-op
   (`same-name`); swapping `{'a','b'}`→`{'b','a'}` works (`swap`); the new
   name `'my var'` is accepted unchanged (`non-identifier`).
3. Errors: duplicate name, unknown name, count mismatch, empty name, index
   past the end, `'Row'` on a table with row names (`err-*`).

### User Story 2 - Transpose a table (P1)

1. `rows2vars(table([1;2],[3;4],'VariableNames',{'x','y'}))` gives
   `OriginalVariableNames={'x';'y'}`, `Var1=[1;3]`, `Var2=[2;4]` (`basic`);
   row names name the new variables (`rownames`); `VariableNamesSource` by
   name, index or string takes the names from that variable (`source-*`).
2. Invalid names are modified under the default/`'modify'` rule (`'a b'`→`aB`,
   `'1c'`→`x1c`) with warning `RunMat:table:ModifiedVarnamesRows2Vars`, and
   kept under `'preserve'`; duplicates become `k, k_1`; mixed double/cellstr
   variables give cell columns; a 0-row table gives only
   `OriginalVariableNames`.
3. Errors: multi-column variable, unknown source, bad naming rule.

### User Story 3 - Apply a function to each variable (P1)

1. `varfun(@mean,X)` gives a 1×2 table `mean_x=2, mean_y=6`; `@std`,
   `@cumsum`, `@max` and an anonymous function (`Fun_x`, `Fun_y`, 3 rows)
   follow the same naming; a 0-row table with `@sum` gives `sum_x=0`.
2. Errors: function fails on a variable (`FunFailed`); numeric first
   argument (`InvalidFunction`).

### User Story 4 - Convert table variables (P1)

1. `convertvars(P,'p','string')` converts `p` to string and leaves `n`, `q`;
   selection by string array, index and mask works; names with spaces work;
   `'categorical'`, `'int8'`, `'double'` (no change), numeric→`'string'`
   and a function handle (`@upper`) work.
2. Errors: unknown variable, unknown type name, failing conversion.

### User Story 5 - Unspecified behaviour fails clearly (P2)

Each open question raises `RunMat:<fn>:UnsupportedOption` or follows a
documented RunMat choice (FR-056-40..). None is asserted as MATLAB behaviour.

## Requirements *(mandatory)*

Observed `MATLAB:<rest>` identifiers are raised as `RunMat:<rest>` (the
mapping of F34 FR-034-11). Message text is not reproduced.

### `renamevars` (normative)

- **FR-056-01** `T2 = renamevars(T, vars, newNames)` MUST return a table of
  the same size [renamevars.signature-primary].
- **FR-056-02** `vars` MUST accept a char name, cellstr, string array,
  numeric index, index vector (paired with `newNames` in the given order)
  and a logical mask [renamevars.selection].
- **FR-056-03** Variable order, data and row names MUST be unchanged
  [renamevars.keeps-data]; renaming to the current name MUST leave the table
  unchanged [renamevars.same-name]; names MAY be swapped in one call
  [renamevars.swap]; a new name that is not a valid identifier MUST be
  accepted unchanged [renamevars.non-identifier].
- **FR-056-04** Errors: a new name equal to another existing variable name →
  `RunMat:table:DuplicateVarNames` [renamevars.error-duplicate]; an old name
  that is not a variable → `RunMat:table:UnrecognizedVarName`
  [renamevars.error-unknown]; different numbers of old and new names →
  `RunMat:table:renamevars:NumNamesMismatch` [renamevars.error-count]; an
  empty new name → `RunMat:table:InvalidVarNames`
  [renamevars.error-empty-name]; a numeric index past the last variable →
  `RunMat:table:VarIndexOutOfRange` [renamevars.error-index-range]; old name
  `'Row'` on a table with row names and no such variable →
  `RunMat:table:renamevars:RenameDim` [renamevars.error-dim-Row].

### `rows2vars` (normative)

- **FR-056-10** `T2 = rows2vars(T)` and `T2 = rows2vars(T, Name, Value)` MUST
  return a table [rows2vars.signature-primary]. The first variable
  `OriginalVariableNames` is a cell column of the original names; then one
  variable per original row holding that row's values
  [rows2vars.layout].
- **FR-056-11** Without row names or `VariableNamesSource` the new variables
  are `Var1, Var2, …`; with row names they take the row names
  [rows2vars.default-names].
- **FR-056-12** `VariableNamesSource` by name or index (char or string option
  name and value) MUST take the names from that variable, which is not
  transposed [rows2vars.names-source].
- **FR-056-13** Under the default rule or `'modify'`, invalid names MUST be
  made valid (`'a b'`→`aB`, `'1c'`→`x1c`) [rows2vars.naming-modify], and
  under the default rule warning `RunMat:table:ModifiedVarnamesRows2Vars`
  MUST be issued [rows2vars.warning-warn-naming-default]. Under
  `'preserve'` names are kept [rows2vars.naming-preserve].
- **FR-056-14** Duplicate source values MUST be made unique (`k`, `k_1`)
  [rows2vars.duplicate-source].
- **FR-056-15** When the transposed variables are double and cellstr, every
  new variable MUST be a cell column [rows2vars.mixed].
- **FR-056-16** A 0-row table MUST give a table with only
  `OriginalVariableNames`, one row per original variable
  [rows2vars.empty-rows].
- **FR-056-17** Errors: a variable with more than one column →
  `RunMat:table:rows2vars:CannotTransposeMulticolumnVar`
  [rows2vars.error-multicol]; a source naming an absent variable →
  `RunMat:table:UnrecognizedVarName` [rows2vars.error-unknown-source]; an
  unrecognised `VariableNamingRule` → `RunMat:rows2vars:unrecognizedStringChoice`
  [rows2vars.error-bad-rule].

### `varfun` (normative)

- **FR-056-20** `R = varfun(func, T)` MUST return a table
  [varfun.signature-primary] whose variables are named
  `<function name>_<variable name>`; an anonymous function gives
  `Fun_<variable name>` [varfun.names].
- **FR-056-21** A scalar result gives one row; a column result gives that
  many rows [varfun.rows]. `table2array(varfun(@mean,X))'` is `[2;6]`, and
  `varfun(@mean, X([1 3],2:end))` gives `mean_y = 6` [varfun.cobra-pattern].
- **FR-056-22** A table with row names and a scalar-valued function gives a
  one-row result `mean_x` [varfun.rownames]; a 0-row table with `@sum` gives
  `sum_x = 0` [varfun.empty-rows].
- **FR-056-23** Errors: `func` fails on a variable →
  `RunMat:table:varfun:FunFailed` [varfun.error-cellstr]; a numeric first
  argument → `RunMat:table:varfun:InvalidFunction`
  [varfun.error-not-function].

### `convertvars` (normative)

- **FR-056-30** `T2 = convertvars(T, vars, type)` and
  `T2 = convertvars(T, vars, func)` MUST return a table
  [convertvars.signature-primary]. `vars` by name, string array of names,
  numeric index or logical mask; other variables unchanged
  [convertvars.selection]; names containing spaces work
  [convertvars.cobra-form].
- **FR-056-31** cellstr→`'string'` gives string; cellstr→`'categorical'`
  gives categorical (categories `a, b`); double→`'string'` gives `"1","2"`;
  double→`'int8'` gives int8; double→`'double'` leaves the table unchanged
  [convertvars.types].
- **FR-056-32** A function handle MUST be applied to the variable
  (`@upper` on `{'a';'b'}` gives `{'A';'B'}`, still cell)
  [convertvars.function].
- **FR-056-33** Errors: unknown variable → `RunMat:table:UnrecognizedVarName`
  [convertvars.error-unknown]; unrecognised type name →
  `RunMat:table:convertvars:InvalidTextType` [convertvars.error-bad-type];
  failing conversion (cellstr→`'double'`) →
  `RunMat:table:convertvars:VariableTypesConversionFailed`
  [convertvars.error-cell-to-double].

### Non-normative observations

`renamevars.non-table`, `rows2vars.non-table`, `varfun.non-table`,
`convertvars.non-table` (dispatch-level `MATLAB:UndefinedFunction`) are not
requirements. RunMat raises `RunMat:<fn>:InvalidInput` for a non-table
(FR-056-40).

### Unresolved questions and documented choices (RunMat behaviour only)

Common to all four:

- **FR-056-40** U-056-1: the table input must be a `table`
  (`RunMat:<fn>:InvalidInput`); a `timetable` raises
  `RunMat:<fn>:UnsupportedOption` (`renamevars.q-timetable`,
  `rows2vars.q-more`). More than one requested output raises
  `RunMat:<fn>:TooManyOutputs`. Wrong argument counts raise
  `RunMat:<fn>:InvalidArgument`.
- **FR-056-41** U-056-2 (`*.q-warning-coverage`): only the
  `rows2vars` naming warning of FR-056-13 is emitted; no other warning
  behaviour is asserted.
- **FR-056-42** U-056-3 (variable selection, renamevars and convertvars):
  as F34 — a cellstr selection is accepted for `convertvars`; an index
  that is zero, negative or fractional raises `RunMat:badsubscript`; a
  logical mask whose length differs from the width, or a multi-row char
  array, raises `UnsupportedOption`; any other selector class raises
  `InvalidArgument`.

`renamevars`:

- **FR-056-43** U-056-4 (`renamevars.q-row-variable`, documented choice): a
  variable literally named `Row` is renamed like any other variable. The
  `RenameDim` error is raised only when the old name is not a variable, the
  table has row names, and the old name is `'Row'` or the table's first
  `DimensionNames` entry. `'Row'` on a table without row names is an
  unknown name (`UnrecognizedVarName`).
- **FR-056-44** U-056-5: two new names that are equal are also
  `DuplicateVarNames`; a variable selected more than once raises
  `RunMat:renamevars:InvalidArgument`; a whitespace-only new name is
  `InvalidVarNames`; an empty selection with empty names returns `T`
  unchanged. Units and descriptions stay with their variables.

`rows2vars`:

- **FR-056-45** U-056-6 (classes, `rows2vars.q-more`): the transposed
  variables must all be real double, all logical, all string, or all cell
  (result of the same class), or a mix of exactly real double and cell (cell
  result, FR-056-15). Any other class or mix raises
  `RunMat:rows2vars:UnsupportedOption`. The multi-column check
  (FR-056-17) applies first, to every transposed variable.
- **FR-056-46** U-056-7 (name making, documented choice derived from the two
  observed cases): trim outer whitespace; remove each inner whitespace run
  and upper-case the following letter; replace every other character that
  is not a letter, digit or underscore with `_`; prefix `x` when the result
  does not start with a letter. The rule is applied to row names and to
  source values alike. The warning is issued under `'modify'` too. An empty
  name raises `UnsupportedOption` under either rule.
- **FR-056-47** U-056-8 (`rows2vars.q-rownames-dup-warning`): duplicates
  (including a clash with `OriginalVariableNames`) are suffixed `_1`, `_2`,
  … (case-sensitive), under both rules; no duplicate warning; the output
  has no row names. `VariableNamesSource` wins over row names.
- **FR-056-48** U-056-9 (options): option names and the rule value are
  case-insensitive; an option other than `VariableNamesSource` /
  `VariableNamingRule` raises `UnsupportedOption`; a dangling option or a
  repeated one raises `InvalidArgument`. The source must be one name or one
  positive integer index (`RunMat:table:VarIndexOutOfRange` past the end,
  `RunMat:badsubscript` for a bad index) and its values must be a cellstr
  or string column; other source classes raise `UnsupportedOption`. A
  table whose only variable is the source, with rows, raises
  `UnsupportedOption`.

`varfun`:

- **FR-056-49** U-056-10 (`varfun.q-more-args`): any call with more than two
  arguments (including name-value options such as `InputVariables`,
  `OutputFormat`, `GroupingVariables`, none of which was observed) raises
  `RunMat:varfun:UnsupportedOption`.
- **FR-056-50** U-056-11 (function argument): numeric and logical first
  arguments raise `InvalidFunction` (FR-056-23); text (a function name)
  raises `UnsupportedOption`; any other value raises
  `RunMat:varfun:InvalidArgument`. The function name of a handle is its
  text after any `@` and after the last `.`; a closure whose name is not a
  valid identifier (anonymous closures) gives `Fun`.
- **FR-056-51** U-056-12 (results): results of different heights raise
  `RunMat:varfun:UnsupportedOption`. The output has no row names (the
  observation records no row names, `varfun.rownames`). A table with no
  variables gives an empty table with no variables.

`convertvars`:

- **FR-056-52** U-056-13 (type names): `double`, `single`, `int8`…`uint64`,
  `logical`, `string`, `categorical` (case-sensitive) are converted by
  calling RunMat's builtin of that name. `char`, `cell`, `cellstr`,
  `datetime`, `duration`, `calendarDuration`, `struct`, `table`,
  `timetable` raise `RunMat:convertvars:UnsupportedOption`; any other text
  is `InvalidTextType` (FR-056-33). A non-text, non-handle type argument
  raises `RunMat:convertvars:InvalidArgument`.
- **FR-056-53** U-056-14 (failures): any error from the conversion builtin,
  a cell variable converted to a numeric or logical class, or a converted
  value with a different number of rows, is
  `VariableTypesConversionFailed` (the confirmed condition, "the conversion
  fails"). For the function form an error raised by the function
  propagates unchanged, and a result with a different number of rows raises
  `RunMat:convertvars:UnsupportedOption`. A variable selected twice is
  converted once. Row names, units and descriptions are kept.

## Success Criteria

- **SC-056-1** Every observation case of the four exports (17 renamevars,
  17 rows2vars, 12 varfun, 14 convertvars) is reproduced by a test, except
  the four non-normative `err-not-table` cases (covered by FR-056-40
  tests).
- **SC-056-2** The `builtins::table` suite stays green apart from the
  known pre-existing failures; clippy and rustfmt clean on the new files.
- **SC-056-3** `validation.md` traces every claim → FR → test → result.

## Known upstream gap

D-034-1 (RunMat's table representation derives height from its first
variable, so a table with rows and no variables cannot be represented)
arises nowhere as a wrong result: `renamevars` and `convertvars` keep the
variable list; every `rows2vars` output has `OriginalVariableNames`; a
`varfun` result has one variable per input variable, and an input with no
variables has no rows. (The `rows2vars` case where the source is the only
variable is representable but leaves the class of the new 0-row variables
unobserved; it raises `UnsupportedOption`, FR-056-48.)
