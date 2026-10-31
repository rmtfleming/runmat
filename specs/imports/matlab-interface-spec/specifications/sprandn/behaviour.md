# `sprandn` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N5, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/sprandn_observations.json`](observations/sprandn_observations.json)

## Summary

Return a random sparse double matrix of a given size with nonzeros of both signs, up to a given density, or with the sparsity pattern of a given matrix.

## Confirmed invocation forms

- `R = sprandn(m, n, density)` — Random m-by-n sparse matrix (COBRA form). *(e.g. case `cobra-5x6`)*
- `R = sprandn(S)` — Same pattern as S, random values. *(e.g. case `pattern`)*
- `R = sprandn(m, n, density, rc)` — Four-argument form with a reciprocal condition number. *(e.g. case `rc`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — The result is a real, sparse double of size m-by-n with finite nonzeros of both signs; with the same rng seed the result is identical. *(cases: `cobra-5x6`, `cobra-30x30`, `density-one`)*
2. **Confirmed** — The nonzero count did not exceed density*m*n in the observed cases (14 for 5x6 at 0.6; 229 for 30x30 at 0.3; 11 for 4x5 at density 1). *(cases: `cobra-5x6`, `cobra-30x30`, `density-one`)*
3. **Confirmed** — Density 0 gave an all-zero sparse matrix, and a zero-row size gave an empty sparse 0x3. *(cases: `density-zero`, `empty-dims`)*
4. **Confirmed** — sprandn(S) returned a sparse matrix of the size of S with exactly its nonzero pattern. *(cases: `pattern`)*
5. **Confirmed** — sprandn(4, 4, 0.5, 0.1) returned a 4x4 sparse double whose 2-norm condition number was 10. *(cases: `rc`)*

## Edge cases (observed)

- *(non-normative)* Out-of-range densities did not error: 1.5 gave a 3x3 sparse matrix with 7 nonzeros, and -0.1 an all-zero 3x3 sparse matrix. *(cases: `density-above-one`, `density-negative`)*
- *(non-normative)* Inference (not observed): the nonzero values and their positions depend on the random generator. *(cases: `cobra-5x6`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| m is not an integer. | `MATLAB:RandStream:randi:invalidLimits` | `err-noninteger` |
| Only m and n are given. | `MATLAB:sprandn:TwoInputs` | `err-two-args` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `cobra-5x6` | `struct` | `[1, 1]` | class='double'; size=[5 6]; issparse=true; isreal=true; nnz=14; all_finite=true; has_negative=true; has_positive=true; repeatable=true |
| `cobra-30x30` | `struct` | `[1, 1]` | class='double'; size=[30 30]; issparse=true; isreal=true; nnz=229; all_finite=true; has_negative=true; has_positive=true; repeatable=true |
| `density-zero` | `struct` | `[1, 1]` | class='double'; size=[4 5]; issparse=true; isreal=true; nnz=0; all_finite=true; has_negative=false; has_positive=false; repeatable=true |
| `density-one` | `struct` | `[1, 1]` | class='double'; size=[4 5]; issparse=true; isreal=true; nnz=11; all_finite=true; has_negative=true; has_positive=true; repeatable=true |
| `empty-dims` | `struct` | `[1, 1]` | class='double'; size=[0 3]; issparse=true; isreal=true; nnz=0; all_finite=true; has_negative=false; has_positive=false; repeatable=true |
| `pattern` | `struct` | `[1, 1]` | class='double'; size=[3 3]; issparse=true; same_pattern=true |
| `rc` | `struct` | `[1, 1]` | class='double'; size=[4 4]; issparse=true; cond=10 |
| `density-above-one` | `struct` | `[1, 1]` | class='double'; size=[3 3]; issparse=true; isreal=true; nnz=7; all_finite=true; has_negative=true; has_positive=true; repeatable=true |
| `density-negative` | `struct` | `[1, 1]` | class='double'; size=[3 3]; issparse=true; isreal=true; nnz=0; all_finite=true; has_negative=false; has_positive=false; repeatable=true |
| `err-noninteger` | — | — | error `MATLAB:RandStream:randi:invalidLimits` |
| `err-two-args` | — | — | error `MATLAB:sprandn:TwoInputs` |

## Unresolved questions

- `sprandn.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `sprandn.q-error-ids` — The non-integer identifier names another component (RandStream/randi); whether an implementation must reproduce it is a human-review question (constitution 8.2).
- `sprandn.q-values` — Whether implementations must match the oracle values for a given seed, or only the observed structure, class and repeatability, is a human-review question (constitution 8.2).
- `sprandn.q-density` — The exact relation between density and the nonzero count was not determined; only upper bounds were observed.
