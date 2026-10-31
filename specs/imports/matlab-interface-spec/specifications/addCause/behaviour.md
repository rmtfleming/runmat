# `addCause` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N4, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/addCause_observations.json`](observations/addCause_observations.json)

## Summary

Return a copy of an MException with another MException appended to its list of causes.

## Confirmed invocation forms

- `ME2 = addCause(ME, cause)` — Append `cause` (COBRA form err = addCause(err, ME)). *(e.g. case `one`)*
- `ME2 = ME.addCause(cause)` — Dot-call syntax. *(e.g. case `method-syntax`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — Causes are appended in call order and held in an n-by-1 cell (class cell; 1x1, then 2x1 with c1 before c2); the identifier is unchanged. *(cases: `one`, `two`)*
2. **Confirmed** — A cause may itself have causes, which are kept. *(cases: `nested`)*
3. **Confirmed** — Calling addCause without assigning the result left the original exception with 0 causes. *(cases: `value-semantics`)*
4. **Confirmed** — Causes are present on the exception caught after throw. *(cases: `thrown`)*
5. **Confirmed** — An exception raised by error() and caught can be added as a cause. *(cases: `caught-builtin-cause`)*
6. **Confirmed** — ME.addCause(c) gives the same result. *(cases: `method-syntax`)*

## Edge cases (observed)

- *(non-normative)* Observed: a struct first argument errored with MATLAB:UndefinedFunction (dispatch level; not required). *(cases: `err-not-exception`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| The cause is not an MException (char or struct). | `MATLAB:class:RequireClass` | `err-char-cause`, `err-struct-cause` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `one` | `struct` | `[1, 1]` | class='MException'; identifier='n4:outer'; cause_ids=["'n4:c1'"]; cause_class='cell'; cause_size=[1 1] |
| `two` | `struct` | `[1, 1]` | class='MException'; identifier='n4:outer'; cause_ids=["'n4:c1'", "'n4:c2'"]; cause_class='cell'; cause_size=[2 1] |
| `nested` | `struct` | `[1, 1]` | cause_id='n4:mid'; inner_cause_id='n4:deep' |
| `value-semantics` | `double` | `[1, 1]` | 0 |
| `thrown` | `struct` | `[1, 1]` | class='MException'; identifier='n4:outer'; cause_ids=["'n4:c1'"]; cause_class='cell'; cause_size=[1 1] |
| `caught-builtin-cause` | `struct` | `[1, 1]` | class='MException'; identifier='n4:outer'; cause_ids=["'n4:raised'"]; cause_class='cell'; cause_size=[1 1] |
| `method-syntax` | `struct` | `[1, 1]` | class='MException'; identifier='n4:o'; cause_ids=["'n4:c'"]; cause_class='cell'; cause_size=[1 1] |
| `err-char-cause` | — | — | error `MATLAB:class:RequireClass` |
| `err-struct-cause` | — | — | error `MATLAB:class:RequireClass` |
| `err-not-exception` | — | — | error `MATLAB:UndefinedFunction` |

## Unresolved questions

- `addCause.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
