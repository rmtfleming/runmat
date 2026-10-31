# `rat` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N5, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/rat_observations.json`](observations/rat_observations.json)

## Summary

Return a rational approximation N./D of each element, or, with one output, a char matrix of text expansions.

## Confirmed invocation forms

- `[N, D] = rat(X)` — Numerators and denominators, elementwise (COBRA form, D used). *(e.g. case `quarter`)*
- `[N, D] = rat(X, tol)` — With a tolerance. *(e.g. case `pi-tol`)*
- `s = rat(X)` — Char expansion, one row per element. *(e.g. case `text-form`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — 0.75 gave 3/4, 1/3 gave 1/3, -0.5 gave -1/2 (the sign is on N and D is positive), 5 gave 5/1 and 0 gave 0/1. *(cases: `quarter`, `third`, `negative`, `integer`, `zero`)*
2. **Confirmed** — pi gave 355/113 with the default tolerance (error about 2.7e-7), and 0.6667 gave 6667/10000. *(cases: `pi-default`, `coeff-like`)*
3. **Confirmed** — pi with tol = 1e-2 gave 22/7. *(cases: `pi-tol`)*
4. **Confirmed** — Arrays are handled elementwise with the input shape ([0.5 0.25; 1.5 2] gave N = [1 1; 3 2], D = [2 4; 2 1]); an empty input gave empty outputs. *(cases: `matrix`, `empty`)*
5. **Confirmed** — N and D are double for double input and single for single input. *(cases: `quarter`, `single`)*
6. **Confirmed** — For real input, one output gives a char row per element: '1 + 1/(-4)' for 0.75, and a 2x10 char matrix for [0.5 1.5] whose rows are '1 + 1/(-2)' and '2 + 1/(-2)'. *(cases: `text-form`, `text-form-vector`)*

## Edge cases (observed)

- Inf and -Inf gave N = 1 and -1 with D = 0; NaN gave N = 0, D = 0. *(cases: `inf`, `nan`)*
- *(non-normative)* Complex input did not error: [N, D] = rat(1 + 2i) gave N = 1+2i, D = 1, and the one-output form gave a 3x8 char matrix. *(cases: `complex-two-out`, `complex-text-form`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| X is char or an integer class. | `MATLAB:norm:inputType` | `err-char`, `err-int` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `quarter` | `struct` | `[1, 1]` | N=3; D=4; class_N='double'; class_D='double'; abs_error=0 |
| `third` | `struct` | `[1, 1]` | N=1; D=3; class_N='double'; class_D='double'; abs_error=0 |
| `negative` | `struct` | `[1, 1]` | N=-1; D=2; class_N='double'; class_D='double'; abs_error=0 |
| `integer` | `struct` | `[1, 1]` | N=5; D=1; class_N='double'; class_D='double'; abs_error=0 |
| `zero` | `struct` | `[1, 1]` | N=0; D=1; class_N='double'; class_D='double'; abs_error=0 |
| `pi-default` | `struct` | `[1, 1]` | N=355; D=113; class_N='double'; class_D='double'; abs_error=2.667641894049666e-07 |
| `pi-tol` | `struct` | `[1, 1]` | N=22; D=7; class_N='double'; class_D='double'; abs_error=0.0012644892673496777 |
| `matrix` | `struct` | `[1, 1]` | N=[1 1;3 2]; D=[2 4;2 1]; class_N='double'; class_D='double'; abs_error=[0 0;0 0] |
| `coeff-like` | `struct` | `[1, 1]` | N=6667; D=10000; class_N='double'; class_D='double'; abs_error=0 |
| `inf` | `struct` | `[1, 1]` | N=[1 -1]; D=[0 0]; class_N='double'; class_D='double'; abs_error=[NaN NaN] |
| `nan` | `struct` | `[1, 1]` | N=0; D=0; class_N='double'; class_D='double'; abs_error=NaN |
| `single` | `struct` | `[1, 1]` | N=3; D=4; class_N='single'; class_D='single'; abs_error=0 |
| `empty` | `struct` | `[1, 1]` | N=[]; D=[]; class_N='double'; class_D='double'; abs_error=[] |
| `text-form` | `char` | `[1, 10]` | '1 + 1/(-4)' |
| `text-form-vector` | `char` | `[2, 10]` | ['1 + 1/(-2)';'2 + 1/(-2)'] |
| `complex-text-form` | `char` | `[3, 8]` | ['1       ';' +i* ...';'2       '] |
| `complex-two-out` | `struct` | `[1, 1]` | N=1+2i; D=1; class_N='double'; class_D='double'; abs_error=0 |
| `err-char` | — | — | error `MATLAB:norm:inputType` |
| `err-int` | — | — | error `MATLAB:norm:inputType` |

## Unresolved questions

- `rat.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `rat.q-error-ids` — The error identifier names another function (norm); whether an implementation must reproduce it is a human-review question (constitution 8.2).
- `rat.q-default-tol` — The default tolerance was not determined; only its results for pi and 0.6667 were observed.
- `rat.q-complex` — The complex results are recorded but not required; whether complex input should be supported is a human-review question.
