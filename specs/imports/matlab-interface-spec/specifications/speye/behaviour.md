# `speye` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/speye_observations.json`](observations/speye_observations.json)

## Summary

Create a sparse matrix with ones on the main diagonal and zeros elsewhere.

## Confirmed invocation form

- `S = speye(n)` — exercised in the experiment (case `square`).

Additional forms are proposed but not yet exercised:
- `S = speye(m, n)` — pending.
- `S = speye(sz)` — pending.

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `double` for the observed inputs. *(cases: `square`, `rectangular`, `size-vector`, `zero`)*
2. **Confirmed** — Returns a sparse double matrix (issparse = true) with ones on the main diagonal; the n, (m,n), and size-vector forms were all accepted. *(cases: `square`, `rectangular`, `size-vector`, `zero`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `square` | `double` | `[3, 3]` | `sparse([1;2;3], [1;2;3], [1;1;1], 3, 3)` |
| `rectangular` | `double` | `[2, 4]` | `sparse([1;2], [1;2], [1;1], 2, 4)` |
| `size-vector` | `double` | `[3, 3]` | `sparse([1;2;3], [1;2;3], [1;1;1], 3, 3)` |
| `zero` | `double` | `[0, 0]` | `sparse([], [], [], 0, 0)` |

## Unresolved questions

- `speye.q-class` — Confirm class (double), issparse=true, and diagonal values.
