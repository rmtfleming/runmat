# Feature Specification: `MException`, `throw`, `addCause`, `lastwarn` — exceptions and warnings

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`049-exceptions-warnings`; export `SPECIFY_FEATURE=049-exceptions-warnings`)

**Created**: 2026-10-30

**Status**: Batch N2 (F49). Gates G1–G3 pre-approved by the maintainer
(`specs/001-clean-room-matlab-builtins/selection-proposal-n2.md`).

**Input**: Approved exports `MException` 0.2.0 and `lastwarn` 0.2.0 (N2,
`8f72edb`) and `throw` 0.2.0 (N3, `1156c97`, imported for this feature).
See `provenance.md`.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Construct exceptions (Priority: P1)

`ME = MException(errID, msg, A1, ...)` creates an exception value.

**Acceptance Scenarios** (normative):

1. `MException('a:b', 'value %d', 5)` has class `MException`, identifier
   `'a:b'`, message `'value 5'`, a 0×1 stack and no causes; `'100%% sure'`
   gives `'100% sure'`; `'a:b:c'` is accepted. [MException.signature-primary,
   MException.output-class, MException.properties, MException.identifier-form]
2. `'nocolon'` and `'a:'` raise `RunMat:MException:badIdentifier`; an
   identifier alone raises `RunMat:minrhs`. [MException.error-bad-id,
   MException.error-no-message]

### User Story 2 - Throw and rethrow (Priority: P1)

`throw(ME)` / `ME.throw` raise the exception; `catch` receives an
`MException` with the same identifier and message.

**Acceptance Scenarios** (normative):

1. `throw(MException('n3:demo','hello'))` raises an error with identifier
   `n3:demo`; the caught exception has identifier `n3:demo`, message
   `hello`, class `MException`. [throw.signature-primary, throw.identifier,
   throw.caught, MException.throw-catch]
2. `ME.throw` behaves the same. [throw.caught (case dot)]
3. Formatted and `%%` messages are caught already formatted.
   [throw.formatted, MException.throw-catch]
4. Throwing (or rethrowing) an error caught from `error('id','msg')` keeps
   identifier and message. [throw.rethrow, MException.rethrow]
5. An empty identifier is thrown and caught as `''`. [throw.no-id]

### User Story 3 - Causes (Priority: P2)

`ME2 = addCause(ME, cause)` returns an exception with the added cause; the
cause survives `throw`. [MException.addcause, throw.cause] — **not met**,
see D-049-1.

### User Story 4 - Last warning (Priority: P1)

`[msg, id] = lastwarn`, `lastwarn(msg, id)`, `[msg, id] = lastwarn(new,
newId)`.

**Acceptance Scenarios** (normative):

1. After `warning('t:id','hello %d',3)`, `[msg,id] = lastwarn` gives
   `'hello 3'`, `'t:id'`; one output gives the message.
   [lastwarn.signature-primary, lastwarn.after-warning]
2. `lastwarn(msg,id)` stores both; `lastwarn(msg)` stores msg and `''`;
   `lastwarn('')` clears both. [lastwarn.set]
3. A setting call with outputs returns the previous values, then stores the
   new ones. [lastwarn.set-return]
4. A warning whose identifier is turned off still updates `lastwarn`.
   [lastwarn.disabled]
5. A warning without an identifier gives `id = ''`. [lastwarn.no-id]
6. After clearing, both outputs are 0×0 char. [lastwarn.empty-class]

## Requirements *(mandatory)*

### Functional Requirements

#### MException

- **FR-049-01** `MException(errID, msg, A1, ...)` MUST return an
  `MException` value whose identifier is `errID` and whose message is `msg`
  formatted with the remaining arguments (RunMat's shared `format_variadic`,
  also with no arguments, so `%%` becomes `%`); a new exception has a 0×1
  stack and no causes (`ME.cause` is an empty 0×1 cell)
  [MException.signature-primary, MException.output-class,
  MException.properties].
- **FR-049-02** Identifiers with two or three non-empty `:`-separated
  components MUST be accepted [MException.identifier-form].
- **FR-049-03** An identifier with no `:` or with an empty component MUST
  raise `RunMat:MException:badIdentifier` [MException.error-bad-id]; a call
  with only an identifier MUST raise `RunMat:minrhs`
  [MException.error-no-message].

#### throw / rethrow

- **FR-049-04** `throw(ME)` and `ME.throw` (also `ME.throw()`) MUST raise an
  error whose identifier is `ME.identifier` (including `''`) and whose
  message is `ME.message`; the exception RunMat's `catch` builds from it
  (`runtime_error::exception_from_error`) MUST have class `MException` and
  the same identifier and message [throw.signature-primary,
  throw.identifier, throw.caught, throw.formatted, throw.no-id,
  MException.throw-catch].
- **FR-049-05** Throwing or rethrowing an exception caught from
  `error(id, msg)` MUST keep its identifier and message [throw.rethrow,
  MException.rethrow] (upstream `rethrow`, unchanged).

#### addCause / causes

- **FR-049-06** `addCause(ME, cause)` returning an exception with one more
  cause, and the cause surviving `throw` [MException.addcause, throw.cause],
  are **not met** (D-049-1). `addCause` raises
  `RunMat:addCause:UnsupportedOption` instead of silently dropping the cause.

#### lastwarn

- **FR-049-07** `[msg, id] = lastwarn` MUST return the last warning's
  formatted message and identifier; with one output, the message
  [lastwarn.signature-primary, lastwarn.after-warning].
- **FR-049-08** `lastwarn(msg, id)` MUST store both; `lastwarn(msg)` MUST
  store `msg` with an empty identifier; `lastwarn('')` MUST clear both
  [lastwarn.set].
- **FR-049-09** A setting call with outputs MUST return the previous values
  and then store the new ones [lastwarn.set-return].
- **FR-049-10** A warning suppressed by `warning('off', id)` MUST still
  update `lastwarn` [lastwarn.disabled].
- **FR-049-11** A warning without an identifier MUST leave an empty
  identifier [lastwarn.no-id].
- **FR-049-12** Empty values MUST be returned as 0×0 char
  [lastwarn.empty-class].

#### Documented choices (local ids)

- **FR-049-13**
  - **U-049-1** `MException`: the empty identifier `''` is accepted (backed
    by the observed `throw` case `no-id`, which constructs
    `MException('', 'no id')`); only the component structure is checked (no
    character-set rule); a non-text identifier or message raises
    `RunMat:MException:InvalidInput`; more than one output raises
    `RunMat:MException:TooManyOutputs`.
  - **U-049-2** `throw` of a non-`MException` raises
    `RunMat:throw:UndefinedFunction` (the observed `UndefinedFunction` is
    dispatch level and non-normative, throw.non-mexception); a call with
    other than one input raises `RunMat:throw:InvalidInput`.
  - **U-049-3** `addCause` with inputs other than two `MException`s raises
    `RunMat:addCause:InvalidInput`.
  - **U-049-4** `lastwarn`: non-text inputs raise
    `RunMat:lastwarn:InvalidInput`; more than two inputs
    `RunMat:lastwarn:UnsupportedOption`; more than two outputs
    `RunMat:lastwarn:TooManyOutputs`; with no output count (statement form)
    the message is returned.
  - **U-049-5** RunMat records identifier-less warnings under the warning
    builtin's default identifier `RunMat:warning`; `lastwarn` reports that
    identifier as `''` (FR-049-11). An explicit `warning('RunMat:warning',
    …)` is therefore also reported as `''`.
  - **U-049-6** `lastwarn` keeps its own state in the session warning
    policy (`WarningPolicy::lastwarn`), separate from upstream's
    `last_warning` (used by `warning('query'|'off', 'last')`), so upstream
    warning behaviour (suppressed warnings do not change `'last'`) is
    unchanged. `lastwarn(msg, id)` does not change the `'last'` target.
- **FR-049-14** (unresolved, returned to the specification side)
  `MException.q-stack`, `throw.q-stack`: stack contents are not asserted
  (RunMat's new exceptions have an empty stack).
  `*.q-warning-coverage`: these builtins emit no warnings.

### Key Entities

- `Value::MException` (`runmat_value::MException { identifier, message,
  stack }`) — no cause list.
- `RuntimeError` with identifier and message; `exception_from_error` builds
  the caught `MException`.
- Session warning policy (`crates/runmat-runtime/src/warnings/policy.rs`).

## Success Criteria *(mandatory)*

- **SC-049-1** Every observed case reproduces in a normative test, except
  the cause cases (D-049-1), kept in an `#[ignore]`d test.
- **SC-049-2** The four builtins are registered through `#[runtime_builtin]`;
  `cargo test -p runmat-runtime --lib -- exceptions_warnings_tests` passes.
- **SC-049-3** Every FR maps to at least one test (validation.md).
- **SC-049-4** `cargo test -p runmat-runtime --lib -- builtins::diagnostics
  warnings::` shows no regressions.

## Unresolved behaviour (non-normative)

| id | disposition |
|---|---|
| MException.q-stack, throw.q-stack | FR-049-14 |
| MException.q-warning-coverage, throw.q-warning-coverage, lastwarn.q-warning-coverage | FR-049-14 |
| throw.non-mexception (non-normative observation) | U-049-2 |
| U-049-1 … U-049-6 (local) | FR-049-13 |

## Known deviations

- **D-049-1** Exception causes. `runmat_value::MException` has no cause
  list, the value codec rejects causes ("nested exception causes are not
  represented by the current runtime MException"), and a thrown
  `RuntimeError` carries only identifier and message into `catch`.
  Supporting `addCause` and `throw.cause` needs a cause field on the value
  type (crate `runmat-value`), codec support, and a way to carry the cause
  list through `RuntimeError` to `exception_from_error` — an architecture
  change outside this feature. `addCause` raises
  `RunMat:addCause:UnsupportedOption`; `ME.cause` reads as an empty 0×1
  cell (true for every RunMat exception). Observed behaviour kept in the
  ignored test `deviation_observed_add_cause_and_thrown_cause`.
- **D-049-2** `ME.throw` dispatch was added at the runtime object layer
  (`object::resolve` member read and `object::dispatch` method call on a
  `Value::MException`); whether the VM compiles a bare `ME.throw;` statement
  into one of these routes is not exercised by runtime-level tests.

## Assumptions

- `try`/`catch` builds the caught exception with
  `runtime_error::exception_from_error` (VM `interpreter/dispatch/
  exceptions.rs`), so runtime tests apply that conversion to the error
  returned by `throw`.
