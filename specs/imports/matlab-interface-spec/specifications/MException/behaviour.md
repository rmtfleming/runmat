# `MException` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N2, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/MException_observations.json`](observations/MException_observations.json)

## Summary

Create an exception object with an identifier and a formatted message; it can be thrown, caught, rethrown and given causes.

## Confirmed invocation forms

- `ME = MException(errID, msg)` — Exception with identifier and message. *(e.g. case `no-format`)*
- `ME = MException(errID, fmt, A1, ...)` — Message formatted from fmt and values. *(e.g. case `construct`)*
- `throw(ME)` — Throw the exception. *(e.g. case `throw-catch`)*
- `ME2 = addCause(ME, cause)` — Return an exception with an added cause. *(e.g. case `addcause`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — The result has class MException. *(cases: `construct`, `throw-catch`)*
2. **Confirmed** — identifier and message hold the given identifier and the formatted message ('value %d', 5 gave 'value 5'; '100%% sure' gave '100% sure'); a new object has a 0x1 stack and no causes. *(cases: `construct`, `no-format`, `percent`)*
3. **Confirmed** — Identifiers with two or three ':'-separated components are accepted. *(cases: `construct`, `three-part-id`)*
4. **Confirmed** — throw(ME) raises it, and the caught object is an MException with the same identifier and message. *(cases: `throw-catch`)*
5. **Confirmed** — rethrow of a caught error preserves its identifier and message. *(cases: `rethrow`)*
6. **Confirmed** — addCause returns an exception whose cause list holds the added exception (1 cause, identifier 'c:d'); the original object still had 0 causes. *(cases: `addcause`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| The identifier has no ':' or has an empty component ('nocolon', 'a:'). | `MATLAB:MException:badIdentifier` | `err-bad-id`, `err-empty-id-part` |
| Only the identifier is given (a generic too-few-arguments identifier, not specific to MException). | `MATLAB:minrhs` | `err-no-message` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `construct` | `struct` | `[1, 1]` | class='MException'; identifier='a:b'; message='value 5'; stack_size=[0 1]; n_cause=0 |
| `no-format` | `struct` | `[1, 1]` | class='MException'; identifier='x:y'; message='plain'; stack_size=[0 1]; n_cause=0 |
| `percent` | `struct` | `[1, 1]` | class='MException'; identifier='x:y'; message='100% sure'; stack_size=[0 1]; n_cause=0 |
| `three-part-id` | `struct` | `[1, 1]` | class='MException'; identifier='a:b:c'; message='m'; stack_size=[0 1]; n_cause=0 |
| `throw-catch` | `struct` | `[1, 1]` | identifier='t:h'; message='boom x'; class='MException' |
| `addcause` | `struct` | `[1, 1]` | n_cause_returned=1; cause_id='c:d'; n_cause_original=0 |
| `rethrow` | `struct` | `[1, 1]` | identifier='r:t'; message='orig' |
| `err-bad-id` | — | — | error `MATLAB:MException:badIdentifier` |
| `err-empty-id-part` | — | — | error `MATLAB:MException:badIdentifier` |
| `err-no-message` | — | — | error `MATLAB:minrhs` |

## Unresolved questions

- `MException.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `MException.q-stack` — The stack contents of a thrown exception were not recorded.
