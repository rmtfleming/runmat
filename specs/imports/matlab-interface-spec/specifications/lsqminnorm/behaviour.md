# `lsqminnorm` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N5, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/lsqminnorm_observations.json`](observations/lsqminnorm_observations.json)

## Summary

Return a least-squares solution x of A*x = b with the smallest norm; for the observed systems it agreed with pinv(A)*b.

## Confirmed invocation forms

- `x = lsqminnorm(A, b)` — Minimum-norm least-squares solution (COBRA form, with A = S'). *(e.g. case `underdetermined`)*
- `x = lsqminnorm(A, b, tol)` — With a tolerance argument. *(e.g. case `tol-arg`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — Without tol, or with tol = 1e-3, x agreed with pinv(full(A))*b to within 2e-15 (double) for square nonsingular, overdetermined, underdetermined, rank-deficient, all-zero, complex, several right-hand-side and sparse inputs; no warning was recorded for the rank-deficient cases. *(cases: `square-full-rank`, `overdetermined`, `underdetermined`, `rank-deficient`, `sparse-transposed`, `sparse-wide`, `multi-rhs`, `zero-matrix`, `complex`, `tol-small`)*
2. **Confirmed** — For sparse A the result was a full (non-sparse) double. *(cases: `sparse-transposed`, `sparse-wide`)*
3. **Confirmed** — The COBRA form with a sparse, rank-deficient A passed transposed (`lsqminnorm(S', b)`) returned the minimum-norm least-squares solution of the inconsistent system (equal to pinv(A)*b). *(cases: `sparse-transposed`)*
4. **Confirmed** — An all-zero A gave x = 0 of size columns(A) by 1. *(cases: `zero-matrix`)*
5. **Confirmed** — single inputs gave a single result (agreement with pinv to about 3e-7). *(cases: `single`)*
6. **Confirmed** — A 0x3 A with a 0x1 b gave a 3x1 zero vector. *(cases: `empty`)*
7. **Confirmed** — With tol = 0.5 for the 2x2 matrix [2 1; 1 0.2], the result differed from pinv(A)*b ([0.5027; 0.2212] instead of [1.3333; -1.6667]); with tol = 1e-3 it agreed with pinv(A)*b. *(cases: `tol-arg`, `tol-small`)*

## Edge cases (observed)

- *(non-normative)* Inference (computed from the observed values, not observed directly): the tol = 0.5 result is not the SVD truncation at rank 1 ([0.4932; 0.2197]); it equals, to the printed digits, the minimum-norm solution after truncating a column-pivoted QR factorization of A at rank 1 (|R(1,1)| = 2.236 > tol > |R(2,2)| = 0.268), i.e. [3; 1.32]/5.968. *(cases: `tol-arg`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| b has a different number of rows from A. | `MATLAB:decomposition:mldivide` | `err-rows` |
| A is char or an integer class. | `MATLAB:lsqminnorm:InvalidA` | `err-char`, `err-int` |
| Only A is given. | `MATLAB:narginchk:notEnoughInputs` | `err-one-arg` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `square-full-rank` | `struct` | `[1, 1]` | x=[0.79999999999999938;1.4000000000000001]; class='double'; issparse=false; residual_norm=8.8817841970012523e-16; x_norm=1.6124515496597098; distance_to_pinv=7.0216669371534024e-16; warning_id='' |
| `overdetermined` | `struct` | `[1, 1]` | x=[1.3333333333333328;2.3333333333333339]; class='double'; issparse=false; residual_norm=0.57735026918962562; x_norm=2.6874192494328502; distance_to_pinv=4.9650683064945462e-16; warning_id='' |
| `underdetermined` | `struct` | `[1, 1]` | x=[0.3333333333333327;1.6666666666666665;1.3333333333333337]; class='double'; issparse=false; residual_norm=8.8817841970012523e-16; x_norm=2.1602468994692869; distance_to_pinv=3.3306690738754696e-16; warning_id='' |
| `rank-deficient` | `struct` | `[1, 1]` | x=[0.20000000000000001;0.40000000000000002]; class='double'; issparse=false; residual_norm=4.4408920985006262e-16; x_norm=0.44721359549995798; distance_to_pinv=1.2412670766236366e-16; warning_id='' |
| `sparse-transposed` | `struct` | `[1, 1]` | x=[-0.33333333333333348;-0.33333333333333359;-0.66666666666666696]; class='double'; issparse=false; residual_norm=3.4641016151377544; x_norm=0.81649658092772648; distance_to_pinv=5.6610488670036757e-16; warning_id='' |
| `sparse-wide` | `struct` | `[1, 1]` | x=[0.20000000000000001;0.3000000000000001;0.39999999999999997;0.10000000000000002]; class='double'; issparse=false; residual_norm=4.4408920985006262e-16; x_norm=0.54772255750516619; distance_to_pinv=1.0007415106216802e-16; warning_id='' |
| `multi-rhs` | `struct` | `[1, 1]` | x=[0.3333333333333327 0.66666666666666641;1.6666666666666665 0.33333333333333326;1.3333333333333337 -0.33333333333333337]; class='double'; issparse=false; residual_norm=9.5504995767854724e-16; x_norm=2.3094010767585029; distance_to_pinv=3.6821932062951477e-16; warning_id='' |
| `zero-matrix` | `struct` | `[1, 1]` | x=[0;0;0]; class='double'; issparse=false; residual_norm=1.4142135623730951; x_norm=0; distance_to_pinv=0; warning_id='' |
| `tol-arg` | `struct` | `[1, 1]` | x=[0.50268096514745331;0.22117962466487934]; class='double'; issparse=false; residual_norm=0.50656231661858508; x_norm=0.54918901945365961; distance_to_pinv=2.0625098730592994; warning_id='' |
| `tol-small` | `struct` | `[1, 1]` | x=[1.333333333333333;-1.6666666666666667]; class='double'; issparse=false; residual_norm=7.4476024597418194e-16; x_norm=2.1343747458109492; distance_to_pinv=1.1957467920563633e-15; warning_id='' |
| `single` | `struct` | `[1, 1]` | x=[0.79999983310699463;1.4000002145767212]; class='single'; issparse=false; residual_norm=4.9151249237271077e-07; x_norm=1.6124516531623776; distance_to_pinv=2.7183900495392789e-07; warning_id='' |
| `complex` | `struct` | `[1, 1]` | x=[2.0000000000000004-3.1401849173675508e-16i;1.5700924586837754e-16+1i]; class='double'; issparse=false; residual_norm=4.9650683064945462e-16; x_norm=2.2360679774997902; distance_to_pinv=5.6610488670036767e-16; warning_id='' |
| `empty` | `double` | `[3, 1]` | [0;0;0] |
| `err-rows` | — | — | error `MATLAB:decomposition:mldivide` |
| `err-char` | — | — | error `MATLAB:lsqminnorm:InvalidA` |
| `err-int` | — | — | error `MATLAB:lsqminnorm:InvalidA` |
| `err-one-arg` | — | — | error `MATLAB:narginchk:notEnoughInputs` |

## Unresolved questions

- `lsqminnorm.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `lsqminnorm.q-tol-rule` — How tol selects the retained components, and which factorization is truncated, was inferred from one case only; the exact rule is unresolved.
- `lsqminnorm.q-error-ids` — The row-mismatch identifier names another function (decomposition/mldivide); whether an implementation must reproduce it is a human-review question (constitution 8.2).
- `lsqminnorm.q-exactness` — Results agree with pinv(A)*b to rounding only; implementations should be compared to a tolerance, not digit for digit.
