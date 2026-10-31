# Feature Specification: `addCause` and exception causes

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`060-exception-causes`; export `SPECIFY_FEATURE=060-exception-causes`)

**Created**: 2026-10-30

**Status**: Batch N4 (F60). Gates G1–G3 pre-approved by the maintainer
(`specs/001-clean-room-matlab-builtins/selection-proposal-n3n4.md`,
"Implement cause support", superseding the D-049-1 "document only"
decision).

**Input**: Approved export `addCause` 0.2.0 (N4, `fc9f804`), plus the two
cause claims of the N2/N3 exports already imported for F49:
`MException.addcause` (`MException` 0.2.0, `8f72edb`) and `throw.cause`
(`throw` 0.2.0, `1156c97`). See `provenance.md`.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Add causes (Priority: P1)

`ME2 = addCause(ME, cause)` returns a copy of `ME` with `cause` appended.

**Acceptance Scenarios** (normative):

1. `e = addCause(MException('n4:outer','o'), MException('n4:c1','c'))` is an
   `MException` with identifier `'n4:outer'`; `e.cause` is a 1×1 cell
   holding the `'n4:c1'` exception. Adding `'n4:c2'` gives a 2×1 cell with
   `'n4:c1'` first. [addCause.signature-primary, addCause.append,
   MException.addcause]
2. A cause that has its own causes keeps them. [addCause.nested]
3. `addCause(e, c)` without assignment leaves `e` with 0 causes.
   [addCause.value-semantics, MException.addcause]
4. An exception raised by `error('n4:raised','r')` and caught can be added
   as a cause. [addCause.caught-cause]
5. `e = e.addCause(c)` gives the same result. [addCause.method-syntax]
6. A char or struct cause raises `RunMat:class:RequireClass`.
   [addCause.error-char-cause]

### User Story 2 - Causes survive throw (Priority: P1)

`try throw(addCause(outer, c1)); catch x; end` gives `x` with identifier
`'n4:outer'` and `x.cause` a 1×1 cell holding `'n4:c1'`.
[addCause.thrown, throw.cause]

## Requirements *(mandatory)*

### Functional Requirements

- **FR-060-01** `addCause(ME, cause)` with two `MException` values MUST
  return an `MException` with `ME`'s identifier, message and stack and
  `cause` appended after `ME`'s existing causes. [addCause.signature-primary,
  addCause.append, MException.addcause]
- **FR-060-02** `ME.cause` MUST be an n×1 cell (class `cell`) of the causes
  in the order they were added; 0×1 without causes. [addCause.append,
  MException.properties]
- **FR-060-03** A cause's own causes MUST be kept. [addCause.nested]
- **FR-060-04** `addCause` MUST NOT change its input (value semantics).
  [addCause.value-semantics, MException.addcause]
- **FR-060-05** `throw(ME)`, `ME.throw` and `ME.throw()` MUST carry the
  causes to the exception received by `catch`. [addCause.thrown,
  throw.cause]
- **FR-060-06** A caught exception (from `error`) MUST be accepted as a
  cause. [addCause.caught-cause]
- **FR-060-07** `ME.addCause(cause)` MUST give the same result as
  `addCause(ME, cause)`. [addCause.method-syntax]
- **FR-060-08** A cause that is not an `MException` MUST raise
  `RunMat:class:RequireClass`. [addCause.error-char-cause]
- **FR-060-09** (documented choice, core) The value codec MUST round-trip
  causes recursively (previously it rejected them), so exceptions with
  causes can cross execution boundaries. Local requirement; no claim.
- **FR-060-10** (documented choices, non-normative)
  - **U-060-1** First argument not an `MException` (observed only as a
    dispatch-level `MATLAB:UndefinedFunction`, non-normative
    [addCause.non-exception]) raises `RunMat:addCause:UndefinedFunction`
    (mirrors U-049-2 for `throw`). A call with other than two inputs raises
    `RunMat:addCause:InvalidInput`; more than one output raises
    `RunMat:addCause:TooManyOutputs`.
  - **U-060-2** `rethrow(ME)` also keeps the causes (not observed; follows
    from `rethrow` preserving the exception [MException.rethrow]).
  - **U-060-3** An uncaught error with causes reports them through the
    error's `source` chain ("caused by: <ids>"); the display of uncaught
    causes was not observed.
- **FR-060-11** (unresolved, returned to the specification side)
  `addCause.q-warning-coverage`: `addCause` emits no warnings; not asserted.

### Key Entities

- `runmat_value::MException { identifier, message, stack, causes }` — new
  `causes: Vec<MException>` field (empty from `MException::new`), plus
  `MException::with_cause`.
- `runmat_value::ExceptionCauses` (re-exported from `runtime_error`) — the
  cause list carried as a thrown `RuntimeError`'s `source`;
  `runtime_error::error_from_exception` builds the thrown error and
  `exception_from_error` restores the causes at `catch`. It lives in
  `runmat-value` because a downcast needs one type identity across crates
  (a type defined in `runmat-runtime` did not downcast in the lib-test
  binary, where builtins are dispatched through a second copy of the crate).
- Value codec `ExceptionValue.causes` (already in the wire format of
  `runmat-execution`), now filled and read recursively.

## Success Criteria *(mandatory)*

- **SC-060-1** Every observed `addCause` case and the two N2/N3 cause cases
  reproduce in normative tests (the dispatch-level `err-not-exception` case
  in a documented-choice test).
- **SC-060-2** `cargo test -p runmat-runtime --lib -- exceptions_warnings_tests`
  passes with no ignored test; D-049-1 is closed.
- **SC-060-3** No regressions in `builtins::diagnostics`, `object::`,
  `execution::value_codec`, `runmat-value`, and the VM `exceptions` tests.
- **SC-060-4** Every FR maps to at least one test (validation.md).

## Unresolved behaviour (non-normative)

| id | disposition |
|---|---|
| addCause.non-exception (non-normative observation) | U-060-1 |
| addCause.q-warning-coverage | FR-060-11 |
| U-060-1 … U-060-3 (local) | FR-060-10 |

## Known deviations

None. (`getfield(ME, 'cause')`, the `getfield` builtin's special-object
path, is not changed and still lists only `message`, `identifier`,
`stack`; `ME.cause` member access is the specified form.)

## Assumptions

- Applicable release R2026a as recorded in the imported specs.
- Supersedes FR-049-06 / D-049-1 of `specs/049-exceptions-warnings/`.
