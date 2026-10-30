
# Feature Specification: `addvars`, `removevars`, `movevars` — table variable editing

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`034-table-variable-editing`; export `SPECIFY_FEATURE=034-table-variable-editing`)

**Created**: 2026-10-29 (batch C1, 0.2.0) · **Widened**: 2026-10-29 (0.3.0)

**Status**: Widened to the 0.3.0 exports (maintainer decision "Widen F34",
`specs/001-clean-room-matlab-builtins/selection-proposal-tierc1.md`, follow-up
after spec widening W1). Gates 1–3 pre-approved by the maintainer.

**Input**: Approved Tier B exports `addvars` 0.3.0, `removevars` 0.3.0,
`movevars` 0.3.0 (IMPORT-MANIFEST Batch 7, source pin `5d595da`; see
`provenance.md`).

Claim ids in square brackets are `claim_id`s from the imported
`provenance.yaml` files. Case names in backticks are `case_id`s from the
imported observations.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Add variables to a table (Priority: P1)

A RunMat user calls `addvars` to append one or more variables, or to insert
them before or after an existing variable, optionally naming them.

**Acceptance Scenarios** (normative; `T` = 2×2 table `a = [1;3]`, `b = [2;4]`
unless stated):

1. `addvars(T1, [3;4])` with one-variable `T1` (`a = [1;2]`) → `table`
   `[2 2]`, variables `a, Var2`. `addvars(T, [5;6])` → `[2 3]`, `a, b, Var3`,
   contents unchanged (`append`, `append-full`). [addvars.signature-primary,
   addvars.output-class, addvars.appends-variable, addvars.default-name-position]
2. `'Before','a'` → `Var1, a, b`; `'After','a'` → `a, Var2, b`;
   `'Before',2` → `a, Var2, b`; `'After',2` → `a, b, Var3` (`before-name`,
   `after-name`, `before-index`, `after-index`). [addvars.before,
   addvars.after, addvars.default-name-position]
3. `'NewVariableNames'` given as `'z'`, `{'z'}` or `"z"` → `a, b, z`; with
   `'Before','b'` → `a, z, b` (`newname-*`, `names-and-before`).
   [addvars.new-names]
4. Two data arguments → `a, b, Var3, Var4`; with names `{'y','z'}` →
   `a, b, y, z` (`multi-default`, `multi-names`). [addvars.multiple-variables]
5. A 2×2 matrix becomes one 2×2 variable; a 2×1 cellstr becomes one cell
   variable (`multicol`, `cellstr-var`). [addvars.data-preserved]
6. `x = [5;6]; addvars(T, x)` → `a, b, x` (`name-from-input`).
   [addvars.default-name-input]
7. Errors: both `'Before'` and `'After'` → `RunMat:table:addmovevars:BeforeAndAfter`;
   wrong row count (more or fewer) → `RunMat:table:RowDimensionMismatch`;
   `'NewVariableNames','a'` → `RunMat:table:DuplicateVarNames`; wrong number
   of names → `RunMat:table:addmovevars:IncorrectNumberOfVarNames`; unknown
   location name → `RunMat:table:UnrecognizedVarName`; `'After',3` →
   `RunMat:table:VarIndexOutOfRange`. [addvars.error-before-and-after,
   addvars.error-rows, addvars.error-dup-name, addvars.error-names-count,
   addvars.error-bad-location, addvars.error-index-range]

### User Story 2 - Remove variables (Priority: P1)

**Acceptance Scenarios** (normative; `T` = 2×4 table `a..d =
[1 2 3 4; 5 6 7 8]` unless stated):

1. `removevars(T3, 'b')` on 1×3 `a,b,c` → `[1 2]`, `a, c`; `'b'` on `T` →
   `a, c, d`; `"d"` → `a, b, c` (`by-name`, `name-full`, `string-scalar`).
   [removevars.signature-primary, removevars.output-class, removevars.removes-variable]
2. `2` → `a, c, d`; `[1 3]`, `[3 1]` and `[true false true false]` → `b, d`.
   [removevars.by-index, removevars.by-logical]
3. `{'a','c'}` and `["a","c"]` → `b, d`; `{'a','a'}` → `b, c, d`.
   [removevars.multiple-names]
4. `{}` and `[]` → `T` unchanged. [removevars.empty-selection]
5. Errors: `'nope'` → `RunMat:table:UnrecognizedVarName`; `5` →
   `RunMat:table:VarIndexOutOfRange`; `0`, `-1`, `1.5` → `RunMat:badsubscript`.
   [removevars.error-unknown, removevars.error-index-range, removevars.error-index-zero]

### User Story 3 - Move variables (Priority: P1)

**Acceptance Scenarios** (normative; `T` = 1×4 table `a..d` unless stated):

1. `movevars(T3, 'c', 'Before', 'a')` on `a,b,c` → `[1 3]`, `c, a, b`.
   [movevars.signature-primary, movevars.output-class, movevars.moves-variable]
2. `'a','After','c'` → `b, c, a, d`; `'a','After','d'` → `b, c, d, a`;
   `'d','Before',2` → `a, d, b, c`; `'a','After',3` → `b, c, a, d`.
   [movevars.after, movevars.numeric-location]
3. `4,'Before',1` → `d, a, b, c`; mask `[false true false true]`,
   `'Before',1` → `b, d, a, c`; `{'d','b'},'Before','a'` → `d, b, a, c`;
   `{'a','b'},'After','c'` → `c, a, b, d`. [movevars.numeric-vars,
   movevars.multiple-order]
4. `'b','Before','b'` and `'b','After','b'` → unchanged. [movevars.self-location]
5. `movevars(T, 'a')` → `b, c, d, a`; `movevars(T, {'b','a'})` →
   `c, d, b, a`. [movevars.default-end]
6. Contents move with their names (`contents`). [movevars.contents-follow]
7. Errors: both keywords → `RunMat:table:addmovevars:BeforeAndAfter`; unknown
   variable or location → `RunMat:table:UnrecognizedVarName`; `'After',5` →
   `RunMat:table:VarIndexOutOfRange`. [movevars.error-before-and-after,
   movevars.error-unknown-var, movevars.error-unknown-location,
   movevars.error-index-range]

### User Story 4 - Unresolved questions fail clearly (Priority: P2)

1. `addvars(T, v, 'Before', 3)` on two variables raises
   `RunMat:addvars:UnsupportedOption` (addvars.q-before-past-end).
2. A logical mask shorter or longer than the variable count raises
   `RunMat:removevars:UnsupportedOption` (removevars.q-short-mask,
   removevars.q-logical-longer).
3. `movevars(T, {'a','c'}, 'Before', 'c')` raises
   `RunMat:movevars:UnsupportedOption` (movevars.q-location-in-vars).

## Requirements *(mandatory)*

### Functional Requirements (normative, observation-backed)

- **FR-034-01** `addvars(T, var1, ..., varN)` MUST return a `table` with the
  new variables appended in argument order; existing variables keep order and
  contents and the row count is preserved. Each data argument is stored
  unchanged as one variable. [addvars.signature-primary, addvars.output-class,
  addvars.appends-variable, addvars.multiple-variables, addvars.data-preserved]
- **FR-034-02** `'Before', location` / `'After', location` MUST insert the new
  variables immediately before / after `location`, given as a variable name
  or a numeric index into `T`. [addvars.before, addvars.after]
- **FR-034-03** `'NewVariableNames', names` (char row, string scalar, string
  array or cellstr) MUST name the new variables, one name per variable;
  it combines with `'Before'`/`'After'`. [addvars.new-names]
- **FR-034-04** Without `'NewVariableNames'`, a data argument passed as a
  plain caller variable MUST take that variable's name
  [addvars.default-name-input]; any other data argument MUST be named `Var`
  followed by its column position in the output table
  [addvars.default-name-position].
- **FR-034-05** `addvars` errors (identifiers per FR-034-11): both
  `'Before'` and `'After'` → `BeforeAndAfter`; a data argument whose row count
  differs from `T` → `RowDimensionMismatch`; a given name equal to an existing
  name → `DuplicateVarNames`; name count ≠ variable count →
  `IncorrectNumberOfVarNames`; unknown location name → `UnrecognizedVarName`;
  `'After'` index > width → `VarIndexOutOfRange`. [addvars.error-*]
- **FR-034-06** `removevars(T, vars)` MUST return a `table` without the
  selected variables, the rest keeping order, contents and row count. `vars`
  may be a char name, string scalar or array, cellstr, numeric index scalar
  or vector (order irrelevant), or a logical mask with one element per
  variable. A name listed twice is removed once; an empty selection returns
  `T` unchanged. [removevars.signature-primary, removevars.output-class,
  removevars.removes-variable, removevars.by-index, removevars.by-logical,
  removevars.multiple-names, removevars.empty-selection]
- **FR-034-07** `removevars` errors: unknown name → `UnrecognizedVarName`;
  index > width → `VarIndexOutOfRange`; index zero, negative or non-integer
  → `badsubscript`. [removevars.error-unknown, removevars.error-index-range,
  removevars.error-index-zero]
- **FR-034-08** `movevars(T, vars, 'Before'|'After', location)` and
  `movevars(T, vars)` MUST move the selected variables, as one contiguous
  block, to immediately before / after `location` (name or numeric index
  into `T`), or to the end. A list of names keeps the listed order and a
  logical mask keeps table order. Moving a single variable relative to
  itself leaves `T` unchanged. Contents follow their names; variable and row
  counts are preserved. [movevars.signature-primary, movevars.output-class,
  movevars.moves-variable, movevars.after, movevars.numeric-location,
  movevars.numeric-vars, movevars.multiple-order, movevars.self-location,
  movevars.default-end, movevars.contents-follow]
- **FR-034-09** `movevars` errors: both keywords → `BeforeAndAfter`; unknown
  variable or location name → `UnrecognizedVarName`; location index > width
  → `VarIndexOutOfRange`. [movevars.error-*]
- **FR-034-10** (D-034-1, deviation) `removevars.remove-all` requires a
  `[2 0]` result. RunMat's table representation derives height from its
  first variable, so a table with rows and no variables cannot be
  represented. Removing every variable of a table with at least one row MUST
  therefore raise `RunMat:removevars:UnsupportedOption` instead of returning
  a wrong-sized table. A 0-row table may have all its variables removed.
- **FR-034-11** (identifier mapping) Each confirmed `MATLAB:<rest>`
  identifier is raised as `RunMat:<rest>` (RunMat's error namespace, the
  same mapping `runtime_error::semantic_error` applies), e.g.
  `RunMat:table:UnrecognizedVarName`, `RunMat:badsubscript`.

### Unresolved questions and documented choices

- **FR-034-12** (unresolved — explicit error) `addvars.q-before-past-end`:
  `'Before', width+1` raises `RunMat:addvars:UnsupportedOption` (the observed
  non-error is non-normative and awaiting review). `movevars` gets the same
  treatment (U-034-5). `removevars.q-short-mask`,
  `removevars.q-logical-longer`: a mask whose length differs from the
  variable count raises `RunMat:<fn>:UnsupportedOption` (also for
  `movevars`). `movevars.q-location-in-vars`: several moved variables with
  the location among them raise `RunMat:movevars:UnsupportedOption`.
- **FR-034-13** (documented choice — addvars.q-default-name-collision) A
  default name (`Var<k>` or a caller name) that already exists, or
  that an earlier new variable took, is made unique with RunMat's table
  naming helper (`make_unique_names`, which appends a suffix). Existing names
  never change.
- **FR-034-14** (documented choice — addvars.q-input-name-scope) Caller names
  are read from the builtin callsite the VM records, the mechanism `display`
  uses for its label. Only a plain identifier counts. Field, indexed and
  other expressions (`s.x`, `x(:,1)`) get the position name. The callsite is
  ignored, and position names used, when its argument count differs from the
  call's or an earlier argument could have expanded to several values
  (`c{:}`, `s.f`), and when no callsite exists (e.g. calls from Rust,
  `feval`).
- **FR-034-15** (documented choice) Other unobserved forms: a location
  index that is zero, negative or non-integer → `RunMat:badsubscript`; a
  `'Before'` index > width+1 → `VarIndexOutOfRange`; a location that is not
  one name or one number, a multi-row char array of names, and a repeated
  option → `RunMat:<fn>:UnsupportedOption`; an odd option list, an unknown
  option name (including `'NewVariableNames'` for `movevars`), or a
  non-name cell element → `RunMat:<fn>:InvalidArgument`. A given name that
  is not a valid variable name → `RunMat:addvars:InvalidArgument`. Two equal
  given names → `DuplicateVarNames`. Numeric `vars` for `movevars` keep their
  listed order. An empty `movevars` selection returns `T` unchanged once the
  location is validated. Option names match whole and ignore case.
  A text data argument equal to an option name is read as the option.
- **FR-034-16** (documented choice, kept from 0.2.0) Text arguments may be
  char rows or string scalars. A first argument that is not a table →
  `RunMat:<fn>:InvalidInput`. A `timetable` (no 0.3.0 claim) →
  `RunMat:<fn>:UnsupportedOption` (U-034-3). `addvars` on a table with no
  variables takes its height from the first data argument. Metadata: row
  names, `Description`, `UserData` and `DimensionNames` are unchanged.
  `VariableUnits` and `VariableDescriptions` follow their variables. New
  variables get empty units and descriptions.

### Key Entities

- Input/output: `Value::Object` of class `table` (upstream table
  subsystem). The ordered variable map lives in the object's internal
  variables struct; public properties live in its `Properties` struct.

## Success Criteria *(mandatory)*

- **SC-034-1**: every observed case named above reproduces exactly (class,
  size, variable order, contents, error identifier per FR-034-11), except
  `all` (D-034-1).
- **SC-034-2**: `cargo test -p runmat-runtime --lib -- table_variable_editing`
  passes; `builtins::table` stays green.
- **SC-034-3**: every FR maps to at least one test (validation.md).

## Unresolved behaviour (returned to the specification side)

| id | question | disposition |
|---|---|---|
| addvars.q-default-name-collision | default `Var<n>` already exists | documented choice: made unique (FR-034-13) |
| addvars.q-before-past-end | `'Before'`, width+1 | `UnsupportedOption` (FR-034-12) |
| addvars.q-input-name-scope | caller name for `s.x`, `x(:,1)` | documented choice: position name (FR-034-14) |
| removevars.q-logical-longer | mask longer than width | `UnsupportedOption` (FR-034-12) |
| removevars.q-short-mask | mask shorter than width | `UnsupportedOption` (FR-034-12) |
| movevars.q-location-in-vars | location among several moved variables | `UnsupportedOption` (FR-034-12) |
| U-034-3 (local) | `timetable` inputs | `UnsupportedOption` |
| U-034-5 (local) | `movevars` `'Before'`, width+1 | `UnsupportedOption` |
| D-034-1 (deviation) | `removevars.remove-all` `[N 0]` with N>0 | `UnsupportedOption` (FR-034-10) |

## Superseded 0.2.0 dispositions

Batch C1 (0.2.0, decision D1 = A) raised `UnsupportedOption` for
`addvars.q-position`, `addvars.q-names`, `removevars.q-index`,
`removevars.q-multiple`, `movevars.q-after`, `movevars.q-index` and the
local U-034-1 (several data arguments) and U-034-2 (several variables to
move). 0.3.0 resolves all of them by observation, and they are now implemented
(FR-034-01..09). Other 0.2.0 choices that 0.3.0 contradicts and that are now
reversed:

- `RunMat:<fn>:UnknownVariable` → `RunMat:table:UnrecognizedVarName`, and
  `RunMat:addvars:HeightMismatch` → `RunMat:table:RowDimensionMismatch`
  (confirmed identifiers, FR-034-11).
- U-034-4 (default name `Var<width+1>`) → the position rule and the
  caller-name rule (FR-034-04). Appending gives the same name as before.
- `movevars(T, vars)` (2 arguments) was `UnsupportedOption` and now moves to
  the end. A missing option value is now `InvalidArgument`.
- `removevars(T, vars, extra)` was `UnsupportedOption` and is now
  `InvalidArgument` (only one invocation form is confirmed).

## Assumptions

- Upstream's table subsystem supplies the table type, ordered variable
  storage, the `Properties` struct and the naming helpers. No new runtime
  types are needed. The table subsystem itself is not changed, which is
  why D-034-1 is a deviation rather than a fix.
