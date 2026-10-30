# `isprop` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-29 (batch N1, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/isprop_observations.json`](observations/isprop_observations.json)

## Summary

Return whether an object has a property of the given name.

## Confirmed invocation forms

- `tf = isprop(obj, name)` — True where obj has property `name`. *(e.g. case `figure-yes`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — The output is logical. *(cases: `figure-yes`, `numeric`)*
2. **Confirmed** — Objects with the named property give true (figure 'Visible'; MException 'message'), others false. *(cases: `figure-yes`, `figure-no`, `object-yes`, `object-no`)*
3. **Confirmed** — For a figure, the property name matched regardless of case ('visible' gave true). *(cases: `figure-case`)*
4. **Confirmed** — A struct (with its field name), tables (with a variable name and with 'Properties') and a number gave false. *(cases: `struct-field`, `table-props`, `table-var`, `numeric`)*
5. **Confirmed** — An array of two graphics handles gave a 1x2 logical. *(cases: `handle-array`)*
6. **Confirmed** — A string scalar name is accepted. *(cases: `string-name`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `figure-yes` | `logical` | `[1, 1]` | true |
| `figure-no` | `logical` | `[1, 1]` | false |
| `figure-case` | `logical` | `[1, 1]` | true |
| `struct-field` | `logical` | `[1, 1]` | false |
| `table-props` | `logical` | `[1, 1]` | false |
| `table-var` | `logical` | `[1, 1]` | false |
| `numeric` | `logical` | `[1, 1]` | false |
| `object-yes` | `logical` | `[1, 1]` | true |
| `object-no` | `logical` | `[1, 1]` | false |
| `handle-array` | `logical` | `[1, 2]` | [true true] |
| `string-name` | `logical` | `[1, 1]` | true |

## Unresolved questions

- `isprop.q-warning-coverage` — Warnings were not captured by this harness; an empty warnings list is not a complete statement.
- `isprop.q-case-objects` — Case-insensitive matching was observed for graphics objects only.
