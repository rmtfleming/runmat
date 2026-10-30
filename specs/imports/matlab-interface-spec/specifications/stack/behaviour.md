# `stack` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-29 (batch N1, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/stack_observations.json`](observations/stack_observations.json)

## Summary

Stack several table variables into one variable (wide to long), adding an indicator variable that records the source variable.

## Confirmed invocation forms

- `S = stack(T, vars)` — Stack the variables `vars` of table T. *(e.g. case `basic`)*
- `S = stack(T, vars, Name, Value)` — With output naming or carried-variable options. *(e.g. case `new-names`)*

## Name-value arguments

- `NewDataVariableName` — Name of the stacked data variable.
- `IndexVariableName` — Name of the indicator variable.
- `ConstantVariables` — Variables carried into every output row; {} carries none.

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — Each input row becomes one output row per stacked variable, in input-row order then variable order; the other variables are repeated: id [1;2], x [10;20], y [30;40] gave id [1;1;2;2] and data [10;30;20;40]. *(cases: `basic`, `by-index`, `three-vars`)*
2. **Confirmed** — By default the data variable is named by joining the stacked names with '_' (x_y; x_y_z for three) and the indicator variable is that name plus '_Indicator'; output order is the carried variables, then the indicator, then the data variable. *(cases: `basic`, `indicator`, `three-vars`)*
3. **Confirmed** — The indicator variable is categorical; its values name the source variable of each row (x, y, x, y) and its categories are the stacked variable names. *(cases: `basic`, `indicator`)*
4. **Confirmed** — vars given as indices gave the same result as names. *(cases: `by-index`, `basic`)*
5. **Confirmed** — NewDataVariableName and IndexVariableName rename the data and indicator variables; ConstantVariables {} carries no other variables. *(cases: `new-names`, `constant`)*
6. **Confirmed** — Stacking one variable keeps the remaining non-stacked variables (in order), followed by '<name>_Indicator' and the data variable named after the stacked variable. *(cases: `one-var`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| A stacked variable name is not in the table. | `MATLAB:table:UnrecognizedVarName` | `err-unknown` |
| The stacked variables have incompatible classes (double and cell). | `MATLAB:table:stack:InterleavingDataVarsFailed` | `err-class-mix` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `basic` | `table` | `[4, 3]` | id:double=[1;1;2;2]; x_y_Indicator:categorical={'categorical_values': ['x', 'y', 'x', 'y'], 'categories': ['x', 'y']}; x_y:double=[10;30;20;40] |
| `indicator` | `struct` | `[1, 1]` | names=["'id'", "'x_y_Indicator'", "'x_y'"]; class='categorical'; values=["'x'", "'y'", "'x'", "'y'"] |
| `by-index` | `table` | `[4, 3]` | id:double=[1;1;2;2]; x_y_Indicator:categorical={'categorical_values': ['x', 'y', 'x', 'y'], 'categories': ['x', 'y']}; x_y:double=[10;30;20;40] |
| `new-names` | `table` | `[4, 3]` | id:double=[1;1;2;2]; which:categorical={'categorical_values': ['x', 'y', 'x', 'y'], 'categories': ['x', 'y']}; val:double=[10;30;20;40] |
| `constant` | `table` | `[4, 2]` | x_y_Indicator:categorical={'categorical_values': ['x', 'y', 'x', 'y'], 'categories': ['x', 'y']}; x_y:double=[10;30;20;40] |
| `three-vars` | `table` | `[6, 3]` | id:double=[1;1;1;2;2;2]; x_y_z_Indicator:categorical={'categorical_values': ['x', 'y', 'z', 'x', 'y', 'z'], 'categories': ['x', 'y', 'z']}; x_y_z:double=[10;30;50;20;40;60] |
| `one-var` | `table` | `[2, 4]` | id:double=[1;2]; y:double=[30;40]; x_Indicator:categorical={'categorical_values': ['x', 'x'], 'categories': ['x']}; x:double=[10;20] |
| `err-unknown` | — | — | error `MATLAB:table:UnrecognizedVarName` |
| `err-class-mix` | — | — | error `MATLAB:table:stack:InterleavingDataVarsFailed` |

## Unresolved questions

- `stack.q-warning-coverage` — Warnings were not captured by this harness; an empty warnings list is not a complete statement.
- `stack.q-second-output` — The second output (row indices into T) was not exercised.
- `stack.q-groups` — Stacking several groups at once (a cell of cells) was not exercised.
