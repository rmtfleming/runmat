# `lsqnonneg` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-11-02 (batch N6, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/lsqnonneg_observations.json`](observations/lsqnonneg_observations.json)

## Summary

Return a vector x >= 0 for a real double C and d (in the observed cases, the nonnegative least-squares solution where it could be checked by hand), and optionally resnorm = ||d - C*x||^2, the residual d - C*x, an exit flag, an output structure and lambda = C'*(d - C*x).

## Confirmed invocation forms

- `x = lsqnonneg(C, d)` — x >= 0 (COBRA form, d = zeros(m, 1)). *(e.g. case `one-out`)*
- `[x, resnorm, residual, exitflag, output, lambda] = lsqnonneg(C, d)` — All six outputs. *(e.g. case `bound-active`)*
- `[x, ...] = lsqnonneg(C, d, options)` — With an optimset options structure. *(e.g. case `options-display`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — x is a full double column vector, also for sparse C. *(cases: `interior`, `sparse`)*
2. **Confirmed** — With d = zeros(3, 1) (COBRA form) x was the 4x1 zero vector with one or six outputs; resnorm 0, residual zeros, exitflag 1 and lambda zeros. *(cases: `cobra-zero-rhs`, `cobra-one-out`)*
3. **Confirmed** — eye(2) with d = [1; 2] gave x = [1; 2]; with d = [1; -2] gave x = [1; 0]; eye(3) with d = [-1; -2; -3] gave x = 0; [2 0; 0 4] with [2; 4] gave [1; 1]; [1 0; 0 1; 1 1] with [1; 2; -5] gave x = 0. *(cases: `interior`, `bound-active`, `all-negative`, `one-out`, `overdetermined`)*
4. **Confirmed** — residual = d - C*x and resnorm = norm(residual)^2 (e.g. [0; -2] and 4 for the bound-active case; [1; 2; -5] and 30 for the overdetermined case). *(cases: `bound-active`, `all-negative`, `overdetermined`)*
5. **Confirmed** — lambda equalled the recorded C'*(d - C*x) exactly in every case; it was 0 for components with x > 0 and <= 0 for components held at 0 (bound-active [0; -2]; overdetermined [-4; -3]), except for rounding-level values (1.3e-15) in the rank-deficient case. *(cases: `interior`, `bound-active`, `all-negative`, `overdetermined`, `rank-deficient`, `sparse`, `empty-rows`)*
6. **Confirmed** — exitflag was 1 and output had a field iterations in every case that recorded them; no warning was recorded. *(cases: `cobra-zero-rhs`, `interior`, `bound-active`, `overdetermined`, `rank-deficient`, `sparse`, `empty-rows`)*
7. **Confirmed** — optimset('TolX', 1e-3) and optimset('Display', 'off') were accepted and gave the same results as without options for the same problems. *(cases: `options-tolx`, `options-display`, `overdetermined`, `bound-active`)*
8. **Confirmed** — A 0x2 C with a 0x1 d gave x = [0; 0], resnorm 0 and a 0x1 residual. *(cases: `empty-rows`)*

## Edge cases (observed)

- *(non-normative)* For [1 1; 1 1] with d = [2; 2] the solution is not unique; x = [2; 0] (to rounding) was returned. *(cases: `rank-deficient`)*
- *(non-normative)* output.iterations (0, 2, 1, 0, 0, 1), output.algorithm ('active-set') and the presence of a message field are recorded but not required. *(cases: `cobra-zero-rhs`, `interior`, `bound-active`, `all-negative`, `overdetermined`, `rank-deficient`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| C and d are both single, or C is char. | `MATLAB:lsqnonneg:NonDoubleInput` | `err-single`, `err-char` |
| C is complex. | `MATLAB:lsqnonneg:ComplexCorD` | `err-complex` |
| d has a different number of rows from C. | `MATLAB:sizeDimensionsMustMatch` | `err-size` |
| d is a row vector. | `MATLAB:dimagree` | `err-row-d` |
| Only C is given. | `MATLAB:lsqnonneg:InputArg` | `err-one-arg` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `cobra-zero-rhs` | `struct` | `[1, 1]` | x=[0;0;0;0]; class_x='double'; issparse_x=false; resnorm=0; residual=[0;0;0]; exitflag=1; output_fields=["'iterations'", "'algorithm'", "'message'"]; output_iterations=0; output_algorithm='active-set'; lambda=[0;0;0;0]; ct_residual=[0;0;0;0]; check_resnorm=0; warning_id='' |
| `cobra-one-out` | `double` | `[4, 1]` | [0;0;0;0] |
| `interior` | `struct` | `[1, 1]` | x=[1;2]; class_x='double'; issparse_x=false; resnorm=0; residual=[0;0]; exitflag=1; output_fields=["'iterations'", "'algorithm'", "'message'"]; output_iterations=2; output_algorithm='active-set'; lambda=[0;0]; ct_residual=[0;0]; check_resnorm=0; warning_id='' |
| `bound-active` | `struct` | `[1, 1]` | x=[1;0]; class_x='double'; issparse_x=false; resnorm=4; residual=[0;-2]; exitflag=1; output_fields=["'iterations'", "'algorithm'", "'message'"]; output_iterations=1; output_algorithm='active-set'; lambda=[0;-2]; ct_residual=[0;-2]; check_resnorm=4; warning_id='' |
| `all-negative` | `struct` | `[1, 1]` | x=[0;0;0]; class_x='double'; issparse_x=false; resnorm=14; residual=[-1;-2;-3]; exitflag=1; output_fields=["'iterations'", "'algorithm'", "'message'"]; output_iterations=0; output_algorithm='active-set'; lambda=[-1;-2;-3]; ct_residual=[-1;-2;-3]; check_resnorm=14; warning_id='' |
| `overdetermined` | `struct` | `[1, 1]` | x=[0;0]; class_x='double'; issparse_x=false; resnorm=30; residual=[1;2;-5]; exitflag=1; output_fields=["'iterations'", "'algorithm'", "'message'"]; output_iterations=0; output_algorithm='active-set'; lambda=[-4;-3]; ct_residual=[-4;-3]; check_resnorm=30; warning_id='' |
| `rank-deficient` | `struct` | `[1, 1]` | x=[1.9999999999999993;0]; class_x='double'; issparse_x=false; resnorm=8.8746851837363828e-31; residual=[6.6613381477509392e-16;6.6613381477509392e-16]; exitflag=1; output_fields=["'iterations'", "'algorithm'", "'message'"]; output_iterations=1; output_algorithm='active-set'; lambda=[1.3322676295501878e-15;1.3322676295501878e-15]; ct_residual=[1.3322676295501878e-15;1.3322676295501878e-15]; check_resnorm=8.8746851837363811e-31; warning_id='' |
| `one-out` | `double` | `[2, 1]` | [1;1] |
| `options-tolx` | `struct` | `[1, 1]` | x=[0;0]; class_x='double'; issparse_x=false; resnorm=30; residual=[1;2;-5]; exitflag=1; output_fields=["'iterations'", "'algorithm'", "'message'"]; output_iterations=0; output_algorithm='active-set'; lambda=[-4;-3]; ct_residual=[-4;-3]; check_resnorm=30; warning_id='' |
| `options-display` | `struct` | `[1, 1]` | x=[1;0]; class_x='double'; issparse_x=false; resnorm=4; residual=[0;-2]; exitflag=1; output_fields=["'iterations'", "'algorithm'", "'message'"]; output_iterations=1; output_algorithm='active-set'; lambda=[0;-2]; ct_residual=[0;-2]; check_resnorm=4; warning_id='' |
| `sparse` | `struct` | `[1, 1]` | x=[0;0]; class_x='double'; issparse_x=false; resnorm=30; residual=[1;2;-5]; exitflag=1; output_fields=["'iterations'", "'algorithm'", "'message'"]; output_iterations=0; output_algorithm='active-set'; lambda=[-4;-3]; ct_residual=[-4;-3]; check_resnorm=30; warning_id='' |
| `err-single` | — | — | error `MATLAB:lsqnonneg:NonDoubleInput` |
| `empty-rows` | `struct` | `[1, 1]` | x=[0;0]; class_x='double'; issparse_x=false; resnorm=0; residual=zeros(0,1); exitflag=1; output_fields=["'iterations'", "'algorithm'", "'message'"]; output_iterations=0; output_algorithm='active-set'; lambda=[0;0]; ct_residual=[0;0]; check_resnorm=0; warning_id='' |
| `err-size` | — | — | error `MATLAB:sizeDimensionsMustMatch` |
| `err-complex` | — | — | error `MATLAB:lsqnonneg:ComplexCorD` |
| `err-char` | — | — | error `MATLAB:lsqnonneg:NonDoubleInput` |
| `err-one-arg` | — | — | error `MATLAB:lsqnonneg:InputArg` |
| `err-row-d` | — | — | error `MATLAB:dimagree` |

## Unresolved questions

- `lsqnonneg.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `lsqnonneg.q-exactness` — Results are floating-point; implementations should be compared to a tolerance, not digit for digit.
- `lsqnonneg.q-output-struct` — Must implementations reproduce output.algorithm and output.message? Their contents are implementation-specific and message text was not recorded.
- `lsqnonneg.q-optimality` — Optimality was checked by hand only for small cases; a general optimality criterion was not observed.
- `lsqnonneg.q-nonunique` — Which solution is returned when it is not unique is not required by this spec.
- `lsqnonneg.q-options` — Only TolX and Display were exercised; other options, a third x0 argument and exitflag values other than 1 were not observed.
- `lsqnonneg.q-error-ids` — The size-mismatch identifiers are generic (sizeDimensionsMustMatch, dimagree); whether an implementation must reproduce them is a human-review question (constitution 8.2).
