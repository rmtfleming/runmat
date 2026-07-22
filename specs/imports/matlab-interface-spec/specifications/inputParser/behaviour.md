# `inputParser` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/inputParser_observations.json`](observations/inputParser_observations.json)

## Summary

Construct an input-parsing object used to validate and assign function arguments (positional, optional, and name-value).

## Confirmed invocation form

- `p = inputParser` — exercised in the experiment (case `construct-class`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — After parse, parsed arguments are exposed as fields of the Results struct (a required argument appears as Results.x = 5; a name-value parameter as Results.tol = 0.2 when supplied and 0.1 when omitted); a constructed parser has class inputParser. *(cases: `construct-class`, `results-class`, `required-value`, `param-value`, `param-default`)*
2. **Confirmed** — Parameters left at their default value are listed in the UsingDefaults property, returned as a cell array. *(cases: `using-defaults`, `param-default`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `construct-class` | `char` | `[1, 11]` | `'inputParser'` |
| `results-class` | `char` | `[1, 6]` | `'struct'` |
| `required-value` | `double` | `[1, 1]` | `5` |
| `param-value` | `double` | `[1, 1]` | `0.20000000000000001` |
| `param-default` | `double` | `[1, 1]` | `0.10000000000000001` |
| `using-defaults` | `cell` | `[1, 1]` | `<cell [1 1]>` |

## Unresolved questions

- `inputParser.q-class-semantics` — inputParser is a handle class with methods (addRequired, addOptional, addParameter, parse) and properties (Results, Unmatched, UsingDefaults). Its interface is not a simple function; scope how to specify class members.
- `inputParser.q-methods` — Enumerate and specify the observable behaviour of the parser methods separately.
