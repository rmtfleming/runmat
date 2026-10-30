# Feature Specification: `keys`, `isKey`, `remove` — containers.Map conformance

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`048-containers-map-conformance`; export
`SPECIFY_FEATURE=048-containers-map-conformance`)

**Created**: 2026-10-30

**Status**: Batch N2 (F48). Gates G1–G3 pre-approved by the maintainer
(`specs/001-clean-room-matlab-builtins/selection-proposal-n2.md`).

**Input**: Approved exports `keys`, `isKey`, `remove` 0.2.0 (see
`provenance.md`).

**Nature of the feature**: conformance work on upstream code. The three
functions already exist as the `containers.Map` methods
`containers.Map.keys`, `containers.Map.isKey` and `containers.Map.remove` in
`crates/runmat-runtime/src/builtins/containers/map/containers.map.rs`. This
feature adds spec-derived tests through RunMat's public dispatch routes and
makes minimal in-place fixes where upstream diverges from a normative claim.
The upstream file is not restructured.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - List keys (Priority: P1)

`k = keys(M)` (and `M.keys`) returns the keys of a map as a sorted row cell.

**Acceptance Scenarios** (normative):

1. `keys(containers.Map({'b','a','c'}, {1,2,3}))` is a 1×3 cell
   `{'a','b','c'}`; for numeric keys inserted as `[3 1 2]` it is
   `{1,2,3}`; for an empty map it is a 1×0 cell. [keys.signature-primary,
   keys.output-class, keys.sorted]
2. Keys `b,B,a,A` come back as `A,B,a,b`. [keys.sorted]
3. `values(M)` is in the same order as `keys(M)` (map b→1, a→2, c→3 gives
   values `2,1,3`). [keys.aligned-with-values]
4. `M.keys` gives the same result as `keys(M)`. [keys.method-syntax]

### User Story 2 - Test membership (Priority: P1)

`tf = isKey(M, key)` returns logical membership.

**Acceptance Scenarios** (normative):

1. Present key → `true`, absent → `false`, `'A'` in a map with `'a'` →
   `false`; the result is logical. [isKey.signature-primary,
   isKey.output-class, isKey.membership]
2. The 2×2 cell `{'a','z';'c','b'}` gives `[true false; true true]`.
   [isKey.cell-query]
3. Numeric keys: `2` → true, `5` → false, `{1, 4}` → `[true false]`.
   [isKey.numeric]
4. A char key on a numeric-keyed map, and `'a'` on an empty map, give
   `false` without error. [isKey.lenient]
5. The string scalar `"a"` matches the char key `'a'`. [isKey.string-key]

### User Story 3 - Remove keys (Priority: P1)

`M = remove(M, keySet)` removes keys; the map is a handle.

**Acceptance Scenarios** (normative):

1. Removing `'a'`, `{'a','c'}`, the numeric key `1`, or every key removes
   exactly those keys; the original map variable shows the same keys and
   `Count` as the returned map; the returned value is a `containers.Map`.
   [remove.signature-primary, remove.removes, remove.handle,
   remove.returns-map]
2. Removing an absent key (also a char key on a numeric map) completes,
   keeps every key, and issues warning `RunMat:Containers:Map:NoKeyToRemove`.
   [remove.warning-absent]
3. With `{'a','z'}` the present key `'a'` is still removed and the warning
   is issued. [remove.present-removed-with-absent, remove.warning-absent]

## Requirements *(mandatory)*

### Functional Requirements

- **FR-048-01** `keys(M)` MUST return a 1×n cell of the keys (1×0 for an
  empty map) [keys.signature-primary, keys.output-class].
- **FR-048-02** Keys MUST be in sorted order: char keys by character code
  (upper case before lower case), numeric keys ascending [keys.sorted].
- **FR-048-03** `values(M)` MUST be in the order of `keys(M)`
  [keys.aligned-with-values].
- **FR-048-04** `M.keys` (and `M.keys()`) MUST give the same result as
  `keys(M)` [keys.method-syntax].
- **FR-048-05** `isKey(M, key)` MUST return logical `true`/`false`;
  matching is case-sensitive [isKey.signature-primary, isKey.output-class,
  isKey.membership].
- **FR-048-06** A cell array of keys MUST give a logical array of the same
  size, element (i,j) answering cell element (i,j) [isKey.cell-query].
- **FR-048-07** Numeric keys MUST work the same way [isKey.numeric].
- **FR-048-08** A char key on a numeric-keyed map, and any key on an empty
  (char-keyed) map, MUST give `false` without error [isKey.lenient]. RunMat
  applies this to every text-vs-numeric category mismatch (U-048-3).
- **FR-048-09** A string scalar key MUST match the equal char key
  [isKey.string-key].
- **FR-048-10** `remove(M, keySet)` MUST remove one key, a cell of keys, a
  numeric key, or every key [remove.signature-primary, remove.removes].
- **FR-048-11** The map MUST be modified in place (the original variable
  sees the change) and returned; the result is a `containers.Map`
  [remove.handle, remove.returns-map].
- **FR-048-12** A key that is not in the map (including a char key on a
  numeric map) MUST NOT raise an error: the call completes, every present key in
  `keySet` is removed, and a warning with identifier
  `RunMat:Containers:Map:NoKeyToRemove` is issued [remove.warning-absent,
  remove.present-removed-with-absent].
- **FR-048-13** (documented choices, local ids)
  - **U-048-1** `keys` of a non-map input errors through RunMat's dispatcher
    (no `keys` function exists for other classes). The observed
    `UndefinedFunction` is dispatch level and non-normative (keys.non-map);
    the RunMat identifier is not asserted.
  - **U-048-2** `remove` issues one `NoKeyToRemove` warning per call, after
    all present keys have been removed, however many keys were absent. The
    warning text is RunMat's own wording. It goes through RunMat's warning
    policy, so `warning('off', id)` silences it and `warning('error', id)`
    turns it into an error with the same identifier.
  - **U-048-3** `isKey` and `remove`: a *category* mismatch — a text key
    (char, string) on a numeric map, or a numeric/logical key on a char map —
    counts as "not present" (`false` / NoKeyToRemove warning), also inside a
    cell query. A mismatch between numeric classes (e.g. a `double` key on a
    `uint64` map) keeps upstream's KeyType error, as do keys upstream rejects
    as invalid (a non-finite double, a non-integer for an integer key type);
    upstream test `integer_key_methods_reject_class_mismatch_and_binary64_aliases`
    pins that and is unchanged.
- **FR-048-14** (unresolved, returned to the specification side)
  `keys.q-warning-coverage`, `isKey.q-warning-coverage`,
  `remove.q-warning-coverage`: no warnings are emitted except FR-048-12.

### Key Entities

- `containers.Map` handle objects (`Value::HandleObject`) owned by the
  upstream map registry (`MapStore`).

## Success Criteria *(mandatory)*

- **SC-048-1** Every observed case of the three exports reproduces in a
  normative test.
- **SC-048-2** `cargo test -p runmat-runtime --lib -- map_conformance_tests`
  passes; the upstream `containers_map` test module still passes (with the
  one upstream test that asserted the old remove-missing-key error updated
  to FR-048-12).
- **SC-048-3** Every FR maps to at least one test (validation.md).

## Unresolved behaviour (non-normative)

| id | disposition |
|---|---|
| keys.q-warning-coverage, isKey.q-warning-coverage, remove.q-warning-coverage | FR-048-14 |
| keys.non-map (non-normative observation) | U-048-1 |
| U-048-1 … U-048-3 (local) | FR-048-13 |

## Known deviations

- **D-048-1** (upstream dispatcher) When `remove` (or any `containers.Map`
  method) fails while called by name through
  `call_builtin_async("remove", …)`, the dispatcher's implementation loop
  invokes `containers.Map.remove` a second time (two registered
  implementations of the same name are tried in turn after an error), and the
  second invocation reports `RunMat:containers.Map:Internal` "internal
  storage not found", masking the first error. Observed with the
  `warning('error', 'RunMat:Containers:Map:NoKeyToRemove')` case of U-048-2
  (and, before the fixes, with upstream's own `MissingKey` error). The cause
  is in upstream dispatch/registration, not in the map code; it is not fixed
  here. Kept as the ignored test
  `deviation_remove_warning_error_mode_through_dispatcher`. No normative
  claim is affected (the default warning mode never errors).

## Assumptions

- The function-call forms `keys(M)`, `isKey(M,k)`, `remove(M,k)` reach the
  upstream methods through the dispatcher's registered-instance-method lookup
  (`containers.Map` class registry entries); the member forms go through
  `object::resolve::load_member` / `object::dispatch::
  call_method_or_member_index_named_with_outputs`. Tests use these routes.
