# Feature Specification: `addvars`, `removevars`, `movevars` — table variable editing

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`034-table-variable-editing`; export `SPECIFY_FEATURE=034-table-variable-editing`)

**Created**: 2026-10-29

**Status**: Batch C1 — FR-010 decision D1 = A (confirmed forms only); Gates
1–3 pre-approved by the maintainer

**Input**: Approved Tier B exports `addvars` 0.2.0, `removevars` 0.2.0,
`movevars` 0.2.0 (see `provenance.md`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Append a variable to a table (Priority: P1)

A RunMat user calling `T2 = addvars(T, var)` gets a table with `var` added as
a new last variable, with the same number of rows as `T`.

**Independent Test**: build the observed input (one-variable, two-row table
`a = [1;2]`), call `addvars(T, [3;4])`, and check the class (`table`), size
`[2 2]`, and that the original variable is unchanged.

**Acceptance Scenarios** (normative):

1. **Given** a 2×1 table with variable `a = [1;2]`, **When**
   `addvars(T, [3;4])` is called, **Then** the result is a `table` of size
   `[2 2]`; its first variable is still `a = [1;2]` and its second holds
   `[3;4]`. [addvars.signature-primary, addvars.output-class,
   addvars.appends-variable]

### User Story 2 - Remove a named variable (Priority: P1)

**Acceptance Scenarios** (normative):

1. **Given** a 1×3 table with variables `a, b, c`, **When**
   `removevars(T, 'b')` is called, **Then** the result is a `table` of size
   `[1 2]` with variables `a, c` (row count preserved).
   [removevars.signature-primary, removevars.output-class,
   removevars.removes-variable]

### User Story 3 - Move a variable before another (Priority: P1)

**Acceptance Scenarios** (normative):

1. **Given** a 1×3 table with variables `a, b, c`, **When**
   `movevars(T, 'c', 'Before', 'a')` is called, **Then** the result is a
   `table` of size `[1 3]` whose `VariableNames` are `c, a, b`, with variable
   and row counts preserved. [movevars.signature-primary,
   movevars.moves-variable]

### User Story 4 - Unspecified options fail clearly (Priority: P2)

Under FR-010 decision D1 = A, every option the approved specs leave
unresolved raises an explicit, identifiable RunMat error instead of
guessing. COBRA call sites that use those options therefore fail loudly
until the specification side observes them.

**Acceptance Scenarios** (unresolved, normative-for-RunMat only):

1. `addvars(T, var, 'Before', 'a')`, `addvars(T, var, 'After', 1)`,
   `addvars(T, var, 'NewVariableNames', 'x')` and `addvars(T, v1, v2)` raise
   `RunMat:addvars:UnsupportedOption`. *(addvars.q-position, addvars.q-names;
   U-034-1)*
2. `removevars(T, 2)`, `removevars(T, [false true false])`,
   `removevars(T, {'a','b'})` and `removevars(T, ["a" "b"])` raise
   `RunMat:removevars:UnsupportedOption`. *(removevars.q-index,
   removevars.q-multiple)*
3. `movevars(T, 'c', 'After', 'a')`, `movevars(T, 'c', 'Before', 1)` and
   `movevars(T, {'b','c'}, 'Before', 'a')` raise
   `RunMat:movevars:UnsupportedOption`. *(movevars.q-after, movevars.q-index;
   U-034-2)*

## Requirements *(mandatory)*

### Functional Requirements

- **FR-034-01** `addvars(T, var)` MUST return a `table`
  [addvars.output-class] equal to `T` with `var` appended as a new last
  variable, preserving the row count [addvars.appends-variable,
  addvars.signature-primary].
- **FR-034-02** `removevars(T, name)`, with `name` a char row vector naming
  one existing variable, MUST return a `table` [removevars.output-class]
  equal to `T` without that variable, preserving row count and the order of
  the remaining variables [removevars.removes-variable,
  removevars.signature-primary].
- **FR-034-03** `movevars(T, name, 'Before', location)`, with `name` and
  `location` each naming one existing variable, MUST return a table whose
  variable order places `name` immediately before `location`, preserving
  the variable and row counts [movevars.moves-variable,
  movevars.signature-primary].
- **FR-034-04** (unresolved — addvars.q-position, addvars.q-names, U-034-1)
  `addvars` called with anything other than exactly two arguments MUST raise
  `RunMat:addvars:UnsupportedOption`.
- **FR-034-05** (unresolved — removevars.q-index, removevars.q-multiple)
  `removevars` with a numeric or logical selector, a cell array, or a string
  array with more than one element MUST raise
  `RunMat:removevars:UnsupportedOption`.
- **FR-034-06** (unresolved — movevars.q-after, movevars.q-index, U-034-2)
  `movevars` with `'After'`, a numeric location, or a list of variables
  MUST raise `RunMat:movevars:UnsupportedOption`.
- **FR-034-07** (documented choice) Naming a variable that does not exist
  (`removevars` name, `movevars` name or location) MUST raise
  `RunMat:<fn>:UnknownVariable`. A first argument that is not a `table`
  MUST raise `RunMat:<fn>:InvalidInput`. A `timetable` input is unobserved
  (U-034-3) and MUST raise `RunMat:<fn>:UnsupportedOption`.
- **FR-034-08** (documented choice) `addvars`: `var` MUST have the same row
  count as `T`, else `RunMat:addvars:HeightMismatch`. A table with no
  variables has no height of its own, so any height is accepted. The new variable is
  named `Var<k>`, where `k` is its 1-based position, made unique against the
  existing names (RunMat's existing default-name convention, as used by
  `array2table`). The default name is unobserved (U-034-4).
- **FR-034-09** (documented choice) Text arguments MAY be char row vectors
  or string scalars (RunMat's usual text equivalence). The `'Before'`
  keyword is matched case-insensitively (RunMat's name-value convention).
- **FR-034-10** (documented choice) Table metadata is preserved: row names,
  `Description`, `UserData` and `DimensionNames` are unchanged.
  `VariableUnits` and `VariableDescriptions` move with their variables.
  An added variable gets empty units and description. Moving a variable
  before itself returns the table unchanged.

### Key Entities

- Input/output: `Value::Object` of class `table` (upstream table
  subsystem). The ordered variable map lives in the object's internal
  variables struct; public properties live in its `Properties` struct.

## Success Criteria *(mandatory)*

- **SC-034-1**: the three observed cases reproduce exactly (class, size,
  variable order) in normative tests.
- **SC-034-2**: all three builtins are registered through
  `#[runtime_builtin]`, and
  `cargo test -p runmat-runtime --lib -- table_variable_editing` passes.
- **SC-034-3**: every FR maps to at least one test, and every observed test
  cites at least one claim (validation.md traceability table).

## Unresolved behaviour (non-normative; returned to the specification side)

| id | question | disposition |
|---|---|---|
| addvars.q-position | 'Before'/'After' placement | `UnsupportedOption` |
| addvars.q-names | NewVariableNames | `UnsupportedOption` |
| removevars.q-index | numeric index / logical mask | `UnsupportedOption` |
| removevars.q-multiple | several variables at once | `UnsupportedOption` |
| movevars.q-after | 'After' placement | `UnsupportedOption` |
| movevars.q-index | numeric location | `UnsupportedOption` |
| U-034-1 (local) | `addvars` with several data arguments | `UnsupportedOption` |
| U-034-2 (local) | `movevars` with several variables | `UnsupportedOption` |
| U-034-3 (local) | `timetable` inputs | `UnsupportedOption` |
| U-034-4 (local) | default name of an added variable | documented choice `Var<k>` |

## Assumptions

- Each approved export confirms only one invocation form (cases `append`,
  `by-name`, `before`/`order`). Everything else is unobserved.
- Upstream's table subsystem supplies the table type, ordered variable
  storage, the `Properties` struct and the default-name helpers. No new
  runtime types are needed.
