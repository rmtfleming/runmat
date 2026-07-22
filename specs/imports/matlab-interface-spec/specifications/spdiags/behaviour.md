# `spdiags` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/spdiags_observations.json`](observations/spdiags_observations.json)

## Summary

Extract or construct the diagonals of a sparse matrix; the several call forms either read diagonals from a matrix or build a matrix from supplied diagonals.

## Confirmed invocation form

- `B = spdiags(A)` — exercised in the experiment (case `extract`).

Additional forms are proposed but not yet exercised:
- `A = spdiags(B, d, m, n)` — pending.

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `double` for the observed inputs. *(cases: `extract`, `construct`)*
2. **Confirmed** — The extraction form spdiags(A) returns the nonzero diagonals as the columns of a full matrix; the construction form spdiags(B,d,m,n) returns a sparse matrix of the requested size. *(cases: `extract`, `construct`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `extract` | `double` | `[3, 2]` | `[1 0;3 2;5 4]` |
| `construct` | `double` | `[3, 3]` | `sparse([1;2;3], [1;2;3], [1;2;3], 3, 3)` |

## Unresolved questions

- `spdiags.q-forms` — spdiags has multiple call forms with different output meanings; specify each form separately.
- `spdiags.q-outputs` — Specify the multi-output extraction form [B, d] = spdiags(A).
