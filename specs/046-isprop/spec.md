# Feature Specification: `isprop` — does an object have a named property

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`046-isprop`; export `SPECIFY_FEATURE=046-isprop`)

**Created**: 2026-10-30

**Status**: Batch N1, feature F46. Gates G1–G3 pre-approved by the maintainer
(`specs/001-clean-room-matlab-builtins/selection-proposal-n1.md`).

**Input**: Approved N1 export `isprop` **0.2.0** (see `provenance.md`).

## RunMat representation notes (not MATLAB behaviour)

- In RunMat, graphics objects (figures, axes and so on) are numeric handle
  values that the plotting registry resolves (`resolve_plot_handle`). A
  value counts as a graphics object exactly when every element resolves.
  The property check is "RunMat's `get(h, name)` succeeds". `get` already
  matches property names case-insensitively.
- An MException is `Value::MException`. Its properties are the fields RunMat
  exposes through field access: `identifier`, `message` and `stack`.
- Class objects (`Value::Object`, `Value::HandleObject`) use the class
  registry (`crate::class_registry::get_class`, walking parent classes),
  with the same public-get-access rule as the clean-room `properties`
  (spec 032).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Query a property name (Priority: P1)

**Acceptance Scenarios** (normative, one per observed case):

1. A figure and `'Visible'` → logical `true` 1×1 (`figure-yes`).
   [isprop.signature-primary, isprop.output-class, isprop.objects]
2. A figure and `'NotAProp'` → `false` (`figure-no`). [isprop.objects]
3. A figure and `'visible'` → `true` (`figure-case`).
   [isprop.case-insensitive-graphics]
4. `isprop(struct('a',1),'a')`, `isprop(table(1),'Properties')`,
   `isprop(table(1,'VariableNames',{'x'}),'x')` and `isprop(5,'a')` each give
   logical `false` (`struct-field`, `table-props`, `table-var`, `numeric`).
   [isprop.non-objects, isprop.output-class]
5. `isprop(MException('a:b','c'),'message')` → `true`; with `'nope'` →
   `false` (`object-yes`, `object-no`). [isprop.objects]
6. Two axes handles in a 1×2 array with `'Visible'` → logical `[true true]`
   1×2 (`handle-array`). [isprop.arrays]
7. `isprop(MException("a:b","c"),"message")` → `true` (`string-name`).
   [isprop.string-name]

## Requirements *(mandatory)*

### Functional Requirements (normative)

- **FR-046-01** `tf = isprop(obj, name)` MUST be accepted and return logical
  [isprop.signature-primary, isprop.output-class].
- **FR-046-02** An object with a property called `name` gives `true`, and
  one without gives `false`. Observed for a figure (`Visible`) and for an
  MException (`message`) [isprop.objects].
- **FR-046-03** For graphics objects the name matches regardless of case
  [isprop.case-insensitive-graphics].
- **FR-046-04** A struct (even for its own field name), a table (for
  `'Properties'` and for a variable name) and a number MUST give `false`
  [isprop.non-objects].
- **FR-046-05** An array of graphics handles gives a logical array of the
  same size, one result per handle [isprop.arrays].
- **FR-046-06** `name` may be a char row or a string scalar
  [isprop.string-name].

### Functional Requirements (documented choices and unresolved points)

- **FR-046-07** (documented choice) Arity other than 2 raises
  `RunMat:isprop:NotEnoughInputs` or `RunMat:isprop:TooManyInputs`. A `name`
  that is not a char row or string scalar raises
  `RunMat:isprop:UnsupportedOption`.
- **FR-046-08** (unresolved, isprop.q-case-objects) For non-graphics objects
  (class objects and MException), an exact-case match gives `true`. A name
  that matches a property only when case is ignored raises
  `RunMat:isprop:UnsupportedOption`. A name that matches nothing gives
  `false`. (Known upstream dispatcher quirk: for a `Value::Object` receiver,
  `call_builtin_async` retries a duplicate registration after this error,
  and the retry answers `false`. See `validation.md`.)
- **FR-046-09** (documented choice) Non-object values give scalar `false`.
  This covers char, string, cell, logical, function handles, and numeric
  arrays whose elements are not all live graphics handles. Timetables are
  treated like tables (FR-046-04). An empty numeric array gives scalar
  `false`.
- **FR-046-10** (documented choice) An object array (`Value::ObjectArray`)
  gives a logical array of its shape. Every element has the same class, so
  every element gets the same answer (the FR-046-05 rule, extended). A class
  reference (`?Name`) raises `RunMat:isprop:UnsupportedOption`.
- **FR-046-11** (documented choice) Class properties come from the class
  registry and include inherited ones. Only properties with public get
  access count, as in `properties`. An object whose class is not registered
  gives `false`. A handle object's dynamic properties are not consulted.
- **FR-046-12** (unresolved, isprop.q-warning-coverage) No warning is
  emitted.

## Success Criteria *(mandatory)*

- **SC-046-1**: all 11 observed cases reproduce (class, size, value).
- **SC-046-2**: `isprop` is registered through `#[runtime_builtin]`, and
  `cargo test -p runmat-runtime --lib -- isprop_tests` passes.
- **SC-046-3**: every FR maps to at least one test (`validation.md`).

## Unresolved behaviour (returned to the specification side)

| id | disposition |
|---|---|
| isprop.q-warning-coverage | no warning emitted (FR-046-12) |
| isprop.q-case-objects | a non-graphics name that differs only in case → `UnsupportedOption` (FR-046-08) |
