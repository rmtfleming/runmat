# Feature Specification: `outerjoin` — join two tables on key variables

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`035-outerjoin`; export `SPECIFY_FEATURE=035-outerjoin`)

**Created**: 2026-10-29

**Status**: Blocker B-035-1 lifted by the maintainer (follow-up decision in
`specs/001-clean-room-matlab-builtins/selection-proposal-tierc1.md`). Gates
1–3 pre-approved; B1→B5 run straight through, stopping before any commit.

**Input**: Approved export `outerjoin` 0.3.0 (IMPORT-MANIFEST Batch 7,
source `5d595da`; see `provenance.md`).

## User Scenarios & Testing *(mandatory)*

Unless stated otherwise, `L = table([1;2;4],[10;20;40],'VariableNames',{'k','a'})`
and `R = table([1;3;4],[100;300;400],'VariableNames',{'k','b'})`, both held
in workspace variables named `L` and `R`.

### User Story 1 - Full outer join on the common variables (Priority: P1)

**Acceptance Scenarios** (normative):

1. `outerjoin(L, R)` returns a 4×4 `table` with variables `k_L, a, k_R, b`
   holding `[1;2;NaN;4]`, `[10;20;NaN;40]`, `[1;NaN;3;4]`,
   `[100;NaN;300;400]`. (`default-full`, `type-full`) [outerjoin.key-inference,
   outerjoin.type-rows, outerjoin.row-order, outerjoin.column-order,
   outerjoin.name-suffix, outerjoin.fill-values]
2. Two 2-row tables sharing one key value, passed as expressions rather than
   named variables, give a 3×4 table `k_left, a, k_right, b`. (`basic`)
   [outerjoin.signature-primary, outerjoin.output-class,
   outerjoin.table-shape, outerjoin.name-suffix]
3. Unsorted inputs (`Lu`, `Ru`) give the same rows in ascending key order,
   names `k_Lu`, `k_Ru`. (`unsorted`) [outerjoin.row-order]

### User Story 2 - Choose the keys (Priority: P1)

1. `'Keys','k'` and `'Keys',1` give the same result as story 1.
   (`keys-name`, `keys-index`) [outerjoin.keys-option]
2. `'LeftKeys','k','RightKeys','id'` against `R2` (key named `id`) gives
   `k, a, id, b` with no suffixes. (`leftright-keys`) [outerjoin.keys-option,
   outerjoin.name-suffix]
3. Two common variables `k1, k2` are both keys; rows sort by `k1` then
   `k2`, also when both inputs are unsorted. (`multi-key`,
   `multi-key-unsorted`) [outerjoin.key-inference, outerjoin.row-order]
4. `'Keys'` combined with `'LeftKeys'` fails with
   `RunMat:outerjoin:ConflictingInputs`. (`err-keys-and-leftkeys`)
   [outerjoin.error-keys-and-leftkeys]
5. Tables with no common variable name and no key options fail with
   `RunMat:outerjoin:CantInferKey`. (`err-no-common`)
   [outerjoin.error-no-common]

### User Story 3 - Merge the keys (Priority: P1)

1. `'MergeKeys',true` gives `k, a, b`, with `k = [1;2;3;4]` (no fill in the
   key). (`mergekeys`, `unsorted-merge`) [outerjoin.mergekeys]
2. Merging differently named keys `k`/`id` produces `k_id`.
   (`leftright-keys-merge`) [outerjoin.mergekeys]
3. When the left key is the second variable, the merged key keeps that
   position (`a, k, b`); without merging it is `a, k_Lk2, k_R, b`.
   (`key-second-column`, `key-second-column-nomerge`)
   [outerjoin.column-order, outerjoin.name-suffix]
4. Two left rows with the same key each give an output row, in left-input
   order. (`dup-keys`, `dup-keys-desc`) [outerjoin.duplicate-keys]

### User Story 4 - Join type and fill values (Priority: P1)

1. `'Type','left'` keeps keys 1,2,4; `'right'` keeps 1,3,4; `'inner'` keeps
   1,4 and holds a single key variable (`k, a, b`). (`type-left`,
   `type-right`, `type-left-merge`, `type-inner`) [outerjoin.type-rows,
   outerjoin.inner-key-single]
2. An unrecognised type fails with `RunMat:outerjoin:InvalidType`.
   (`err-bad-type`) [outerjoin.error-bad-type]
3. Unmatched cells are filled with NaN (double, single), 0 (int32), false
   (logical), `''` (cellstr) and `<missing>` (string). (`fill-types`)
   [outerjoin.fill-values]

### User Story 5 - Unspecified behaviour fails clearly (Priority: P2)

Every question the export leaves open raises
`RunMat:outerjoin:UnsupportedOption` or follows a documented RunMat choice
(FR-035-13..22). None of these is asserted as MATLAB behaviour.

## Requirements *(mandatory)*

### Functional Requirements (normative)

- **FR-035-01** `T = outerjoin(Tleft, Tright)` MUST return a `table`
  [outerjoin.signature-primary, outerjoin.output-class]; two 2-row tables
  sharing one key value give a 3×4 table [outerjoin.table-shape].
- **FR-035-02** Without key options, the keys MUST be the variables whose
  names appear in both tables [outerjoin.key-inference]. If there are none,
  the call MUST fail with `RunMat:outerjoin:CantInferKey`
  [outerjoin.error-no-common].
- **FR-035-03** `'Keys'` (one name, or a numeric index) MUST select the
  same keys as inference would; `'LeftKeys'`/`'RightKeys'` MUST join on
  differently named keys [outerjoin.keys-option]. `'Keys'` given together
  with `'LeftKeys'` MUST fail with `RunMat:outerjoin:ConflictingInputs`
  [outerjoin.error-keys-and-leftkeys].
- **FR-035-04** `'Type'` MUST select the row set: `'full'` (default) keeps
  matched rows plus unmatched rows of both tables; `'left'` adds only
  unmatched left rows; `'right'` only unmatched right rows; `'inner'` only
  matched rows [outerjoin.type-rows]. Any other type MUST fail with
  `RunMat:outerjoin:InvalidType` [outerjoin.error-bad-type].
- **FR-035-05** Output rows MUST be in ascending key order whatever the
  input order; with several keys, ordered by the first key, then the second
  [outerjoin.row-order].
- **FR-035-06** Each left row whose key matches MUST produce its own output
  row; rows with equal keys keep left-input order [outerjoin.duplicate-keys].
- **FR-035-07** For `'full'`, `'left'` and `'right'`, the output variables
  MUST be the left table's variables in order, then the right table's;
  with `'MergeKeys'`, each merged key takes the position of the left key and
  the right key variable is dropped [outerjoin.column-order].
- **FR-035-08** With `'MergeKeys',true`, each key pair MUST become one
  variable holding the key value from whichever table supplied the row; it
  is named after the left key if both keys have the same name, else
  `<leftname>_<rightname>` [outerjoin.mergekeys].
- **FR-035-09** Without `'MergeKeys'`, a variable name present in both
  tables (key or non-key) MUST be made unique by appending `_` plus the
  caller's variable name of each input; when the inputs are not named
  variables, the suffixes MUST be `_left` and `_right`. Differently named
  keys keep their names [outerjoin.name-suffix]. How RunMat recovers caller
  names is FR-035-17.
- **FR-035-10** Cells with no matching row MUST be filled with NaN for
  double and single, 0 for int32, false for logical, `''` for cellstr and
  `<missing>` for string [outerjoin.fill-values].
- **FR-035-11** With `'Type','inner'` and one common key (MergeKeys not
  given), the output MUST hold a single key variable, `k, a, b`
  [outerjoin.inner-key-single].
- **FR-035-12** Errors carry RunMat identifiers: the export's three
  observed error conditions map to `RunMat:outerjoin:CantInferKey`,
  `RunMat:outerjoin:InvalidType` and `RunMat:outerjoin:ConflictingInputs`
  (FR-035-02..04).

### Unresolved questions (non-normative for MATLAB; normative for RunMat)

- **FR-035-13** (`outerjoin.q-index-outputs`) A call requesting more than
  one output (`[T, ileft, iright] = outerjoin(...)`) MUST raise
  `RunMat:outerjoin:UnsupportedOption`.
- **FR-035-14** (`outerjoin.q-select-variables`) `'LeftVariables'` and
  `'RightVariables'` MUST raise `RunMat:outerjoin:UnsupportedOption`.
- **FR-035-15** (`outerjoin.q-many-to-many`) A key value that occurs more
  than once in both tables, a NaN key and a `<missing>` string key MUST
  raise `RunMat:outerjoin:UnsupportedOption`.
- **FR-035-16** (`outerjoin.q-key-types`) A key pair whose two variables
  have different classes (for example double with int32, or string with
  cellstr) MUST raise `RunMat:outerjoin:UnsupportedOption`. So MUST a key
  variable that is not a single column of a supported class (FR-035-18).
- **FR-035-17** (`outerjoin.q-name-suffix-context`; documented choice) The
  suffix name of each input is the source text of that argument at the call
  site, which the interpreter records for every builtin call (the mechanism
  behind `display` labels and `inputname`). It is used only when (a) a call
  site is recorded, (b) it has exactly as many arguments as this call, (c)
  both table arguments are plain identifiers, and (d) the two identifiers
  differ. Otherwise the suffixes are `_left`/`_right`, the export's
  observed suffixes for inputs that are not named variables. Condition (b)
  rejects call sites of an enclosing call (for example `feval`). Condition
  (d) avoids identical output names for `outerjoin(T, T)`; that case is
  unobserved. The interpreter cannot tell a plain identifier that names a
  function from one that names a variable, so a bare function name is
  treated as a variable name.

### Documented choices (unobserved; RunMat behaviour only)

- **FR-035-18** U-035-1: a key value duplicated only in the right table
  gives one row per matching right row, in right-input order. U-035-2: keys
  of the same class are supported for every real numeric class, logical,
  string and cellstr (single column). They match exactly and sort
  ascending; text sorts by Unicode code point. Other key classes raise
  `UnsupportedOption` (FR-035-16).
- **FR-035-19** U-035-3: a variable that needs filling is filled
  element-wise per FR-035-10 if it is numeric (any real class; integers
  with 0), logical, string or a one-column cellstr, including multi-column
  numeric, logical and string variables. Any other class (char, complex,
  datetime, categorical, nested tables, non-cellstr cells, …) raises
  `UnsupportedOption`, but only when a fill is actually needed. Variables
  copied without fill keep their class.
- **FR-035-20** Naming. U-035-4: `'inner'` with differently named keys
  needs `'MergeKeys',true` (named per FR-035-08); without it the call raises
  `UnsupportedOption`. `'inner'` with same-named keys gives the single key
  variable (FR-035-11) for every number of keys. U-035-5: with
  `'MergeKeys'`, clashing non-key names get the FR-035-09 suffixes as well.
  U-035-9: a name that still collides after suffixing is made unique by
  appending `_2`, `_3`, … to the later variable.
- **FR-035-21** U-035-6: the output carries RunMat's default table
  properties. Row names, units, descriptions, `Description` and `UserData`
  of the inputs are not carried. U-035-7: a non-table input raises
  `RunMat:outerjoin:InvalidInput`; a timetable input raises
  `UnsupportedOption`.
- **FR-035-22** U-035-8: option names and `'Type'` values are matched
  case-insensitively. Text may be a char row or a string scalar. Key lists
  may be a name, a cell or string array of names, or numeric indices (a
  `'Keys'` index must name the same variable in both tables). The following
  raise `RunMat:outerjoin:InvalidArgument`: a dangling option, an unknown
  option, an empty or repeated key list, mismatched `LeftKeys`/`RightKeys`
  counts, `LeftKeys` without `RightKeys` (or the reverse), and a
  `'MergeKeys'` that is not a logical or 0/1 scalar. `'Keys'` with
  `'RightKeys'` raises `ConflictingInputs`. A logical key mask raises
  `UnsupportedOption`. An unknown key name or an out-of-range index raises
  `UnknownVariable`. A non-text `'Type'` raises `InvalidType`. When an
  option repeats, the last value wins.

### Key Entities

- Input/output: `Value::Object` of class `table` (upstream table
  subsystem); variables stored in the ordered internal variables struct.

## Success Criteria *(mandatory)*

- **SC-035-1**: all 25 observation cases reproduce exactly in normative
  tests: class, size, variable names in order, and every value, including
  fills.
- **SC-035-2**: `outerjoin` is registered through `#[runtime_builtin]`;
  `cargo test -p runmat-runtime --lib -- table_outerjoin` passes and the
  `builtins::table` suite stays green.
- **SC-035-3**: every FR maps to at least one test (validation.md).

## Unresolved behaviour (returned to the specification side)

| id | question | disposition |
|---|---|---|
| outerjoin.q-index-outputs | `ileft`/`iright` outputs | `UnsupportedOption` (FR-035-13) |
| outerjoin.q-select-variables | LeftVariables/RightVariables | `UnsupportedOption` (FR-035-14) |
| outerjoin.q-many-to-many | duplicates on both sides; NaN/missing keys | `UnsupportedOption` (FR-035-15) |
| outerjoin.q-key-types | keys of different classes | `UnsupportedOption` (FR-035-16) |
| outerjoin.q-name-suffix-context | caller-name suffixes | documented choice: call-site text, else `_left`/`_right` (FR-035-17) |
| U-035-1 (local) | right-only duplicate keys | documented choice (FR-035-18) |
| U-035-2 (local) | non-double key classes | documented choice (FR-035-18) |
| U-035-3 (local) | fill for other classes | documented choice / `UnsupportedOption` (FR-035-19) |
| U-035-4 (local) | inner join, differently named keys | `UnsupportedOption` without MergeKeys (FR-035-20) |
| U-035-5 (local) | non-key clash with MergeKeys | documented choice (FR-035-20) |
| U-035-6 (local) | output table properties | documented choice: defaults (FR-035-21) |
| U-035-7 (local) | timetable inputs | `UnsupportedOption` (FR-035-21) |
| U-035-8 (local) | option-parsing details | documented choice (FR-035-22) |
| U-035-9 (local) | residual name collisions | documented choice `_2`, `_3`, … (FR-035-20) |
