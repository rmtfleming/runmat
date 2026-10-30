# `throw` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N3, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/throw_observations.json`](observations/throw_observations.json)

## Summary

Throw an MException object as an error; the caught exception carries its identifier, message and causes.

## Confirmed invocation forms

- `throw(ME)` — Throw ME. *(e.g. case `direct`)*
- `ME.throw` — Dot-call syntax. *(e.g. case `dot`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — The raised error has the identifier of ME (n3:demo); case direct errors by design, as throwing is the purpose of the function. *(cases: `direct`, `caught`)*
2. **Confirmed** — The caught exception is an MException with the same identifier and message and the same number of causes. *(cases: `caught`, `dot`, `cause`)*
3. **Confirmed** — A message built from a format and arguments is caught already formatted ('value 3 of x'); '100%% sure' with no arguments gave '100% sure'. *(cases: `formatted`, `percent`)*
4. **Confirmed** — Throwing an error that was raised by error('n3:inner', 'inner') and caught keeps its identifier and message. *(cases: `rethrow-caught`)*
5. **Confirmed** — A cause added with addCause is present on the caught exception (one cause, identifier n3:in). *(cases: `cause`)*
6. **Confirmed** — An exception with an empty identifier is thrown and caught with identifier ''. *(cases: `no-id`)*

## Edge cases (observed)

- *(non-normative)* Observed: a struct or char argument errored with MATLAB:UndefinedFunction (dispatch level; not required). *(cases: `err-struct`, `err-char`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `direct` | — | — | error `n3:demo` |
| `caught` | `struct` | `[1, 1]` | threw=true; class='MException'; identifier='n3:demo'; message='hello'; num_causes=0 |
| `formatted` | `struct` | `[1, 1]` | threw=true; class='MException'; identifier='n3:fmt'; message='value 3 of x'; num_causes=0 |
| `dot` | `struct` | `[1, 1]` | threw=true; class='MException'; identifier='n3:dot'; message='dot'; num_causes=0 |
| `rethrow-caught` | `struct` | `[1, 1]` | threw=true; class='MException'; identifier='n3:inner'; message='inner'; num_causes=0 |
| `cause` | `struct` | `[1, 1]` | threw=true; class='MException'; identifier='n3:outer'; message='outer'; num_causes=1; cause_identifier='n3:in' |
| `no-id` | `struct` | `[1, 1]` | threw=true; class='MException'; identifier=''; message='no id'; num_causes=0 |
| `percent` | `struct` | `[1, 1]` | threw=true; class='MException'; identifier='n3:pct'; message='100% sure'; num_causes=0 |
| `err-struct` | — | — | error `MATLAB:UndefinedFunction` |
| `err-char` | — | — | error `MATLAB:UndefinedFunction` |

## Unresolved questions

- `throw.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `throw.q-stack` — The stack of the thrown exception was not recorded.
