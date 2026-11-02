# `normest` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-11-02 (batch N6, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/normest_observations.json`](observations/normest_observations.json)

## Summary

Return an estimate of the 2-norm of a matrix that agreed with norm(full(S)) to a small relative difference, and, as a second output, an integer-valued count that grew as the tolerance argument decreased.

## Confirmed invocation forms

- `nrm = normest(S)` — 2-norm estimate (COBRA form, S sparse). *(e.g. case `one-out`)*
- `[nrm, cnt] = normest(S)` — Estimate and count. *(e.g. case `sparse-stoich`)*
- `[nrm, cnt] = normest(S, tol)` — With a tolerance argument. *(e.g. case `tol-fine`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — nrm is a full (non-sparse) scalar, double for double or sparse input and single for single input; cnt is double. *(cases: `sparse-stoich`, `identity`, `single`)*
2. **Confirmed** — With the default tolerance, nrm agreed with norm(full(S)) to a relative difference of at most 2.3e-7 for every observed matrix (sparse stoichiometric-style, dense square and wide, vectors, identity, complex, single, [3 0; 0 4]). *(cases: `sparse-stoich`, `dense-square`, `dense-wide`, `one-out`, `row-vector`, `column-vector`, `identity`, `complex`, `single`)*
3. **Confirmed** — For the sparse 4x5 stoichiometric-style matrix (COBRA form normest(S)), nrm = 2 to rounding, equal to norm(full(S)). *(cases: `sparse-stoich`)*
4. **Confirmed** — For the same 3x3 matrix, tol = 1e-1 gave a larger relative difference from norm(full(S)) than the default, and tol = 1e-12 a smaller one. *(cases: `tol-coarse`, `tol-fine`, `dense-square`)*
5. **Confirmed** — An all-zero matrix, full or sparse, gives nrm = 0; a 0x0 input gives 0. *(cases: `zero`, `sparse-zero`, `empty`)*
6. **Confirmed** — A scalar gives its absolute value (-5 gave 5); a matrix containing NaN gives NaN. *(cases: `scalar`, `with-nan`)*

## Edge cases (observed)

- *(non-normative)* char input did not error: normest('ab') returned 137.8876, equal to the recorded norm(double('ab')). *(cases: `char-input`)*
- *(non-normative)* The counts (2, 7, 3, 1, 2, 2, 5, 3; 2 for tol 1e-1 and 15 for tol 1e-12; 0 for all-zero input) and the exact relative differences (1.0e-3, 5.3e-14 and 1.2e-7 for the tol cases) are recorded but not required. *(cases: `sparse-stoich`, `dense-square`, `dense-wide`, `row-vector`, `column-vector`, `identity`, `complex`, `single`, `tol-coarse`, `tol-fine`, `zero`, `sparse-zero`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| S is an integer class (int32). | `MATLAB:mtimes:integerNotSupported` | `err-int` |
| S has more than two dimensions. | `MATLAB:transpose:NDArray` | `err-3d` |
| No input is given. | `MATLAB:minrhs` | `err-no-arg` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `sparse-stoich` | `struct` | `[1, 1]` | nrm=2; cnt=2; class_nrm='double'; class_cnt='double'; issparse_nrm=false; norm_full=1.9999999999999998; rel_diff=1.1102230246251568e-16 |
| `dense-square` | `struct` | `[1, 1]` | nrm=4.7320502633041261; cnt=7; class_nrm='double'; class_cnt='double'; issparse_nrm=false; norm_full=4.7320508075688776; rel_diff=1.1501667535335275e-07 |
| `dense-wide` | `struct` | `[1, 1]` | nrm=5.5885191427415482; cnt=3; class_nrm='double'; class_cnt='double'; issparse_nrm=false; norm_full=5.5885191430045058; rel_diff=4.7053190464464668e-11 |
| `one-out` | `struct` | `[1, 1]` | nrm=3.9999991190559907; class_nrm='double'; norm_full=4; rel_diff=2.2023600232401463e-07 |
| `tol-coarse` | `struct` | `[1, 1]` | nrm=4.7271139464407828; cnt=2; class_nrm='double'; class_cnt='double'; issparse_nrm=false; norm_full=4.7320508075688776; rel_diff=0.0010432815134187512 |
| `tol-fine` | `struct` | `[1, 1]` | nrm=4.7320508075686245; cnt=15; class_nrm='double'; class_cnt='double'; issparse_nrm=false; norm_full=4.7320508075688776; rel_diff=5.349284272469241e-14 |
| `row-vector` | `struct` | `[1, 1]` | nrm=13; cnt=1; class_nrm='double'; class_cnt='double'; issparse_nrm=false; norm_full=13; rel_diff=0 |
| `column-vector` | `struct` | `[1, 1]` | nrm=13; cnt=2; class_nrm='double'; class_cnt='double'; issparse_nrm=false; norm_full=13; rel_diff=0 |
| `identity` | `struct` | `[1, 1]` | nrm=1; cnt=2; class_nrm='double'; class_cnt='double'; issparse_nrm=false; norm_full=1; rel_diff=0 |
| `zero` | `struct` | `[1, 1]` | nrm=0; cnt=0; class_nrm='double'; class_cnt='double'; issparse_nrm=false; norm_full=0; rel_diff=0 |
| `sparse-zero` | `struct` | `[1, 1]` | nrm=0; cnt=0; class_nrm='double'; class_cnt='double'; issparse_nrm=false; norm_full=0; rel_diff=0 |
| `complex` | `struct` | `[1, 1]` | nrm=2.288245606417354; cnt=5; class_nrm='double'; class_cnt='double'; issparse_nrm=false; norm_full=2.2882456112707374; rel_diff=2.121006350448628e-09 |
| `single` | `struct` | `[1, 1]` | nrm=5.4649858474731445; cnt=3; class_nrm='single'; class_cnt='double'; issparse_nrm=false; norm_full=5.4649857042190426; rel_diff=2.6213078984559241e-08 |
| `scalar` | `double` | `[1, 1]` | 5 |
| `empty` | `double` | `[1, 1]` | 0 |
| `with-nan` | `double` | `[1, 1]` | NaN |
| `char-input` | `struct` | `[1, 1]` | nrm=137.88763541376724; class_nrm='double'; norm_full=137.88763541376724; rel_diff=0 |
| `err-int` | — | — | error `MATLAB:mtimes:integerNotSupported` |
| `err-3d` | — | — | error `MATLAB:transpose:NDArray` |
| `err-no-arg` | — | — | error `MATLAB:minrhs` |

## Unresolved questions

- `normest.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `normest.q-exactness` — nrm is an estimate; implementations should be compared to norm(full(S)) within a relative tolerance, not digit for digit, and cnt should not be required to match.
- `normest.q-default-tol` — The default tolerance value was not determined; only the resulting accuracy was observed.
- `normest.q-char` — char input is accepted and computed on character codes; whether implementations must accept it is a human-review question.
- `normest.q-error-ids` — The integer and N-D identifiers name other functions (mtimes, transpose); whether an implementation must reproduce them is a human-review question (constitution 8.2).
