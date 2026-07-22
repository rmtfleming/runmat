# `psi` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/psi_observations.json`](observations/psi_observations.json)

## Summary

Evaluate the psi (digamma) function of the input; a two-argument form evaluates the polygamma function of a given order.

## Confirmed invocation forms

- `Y = psi(X)` — exercised in the experiment (case `scalar`).

Additional forms are proposed but not yet exercised:
- `Y = psi(k, X)` — pending.

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `double` for the observed inputs. *(cases: `scalar`, `vector`)*
2. **Confirmed** — Returns double values element-wise; psi(1) equals the negative of the Euler-Mascheroni constant (about -0.5772). *(cases: `scalar`, `vector`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `scalar` | `double` | `[1, 1]` | `-0.57721566490153231` |
| `vector` | `double` | `[1, 3]` | `[-0.57721566490153231 0.42278433509846747 0.9227843350984674…` |

## Unresolved questions

- `psi.q-polygamma` — Specify the two-argument polygamma form psi(k, X).
- `psi.q-poles` — Behaviour at nonpositive integers (poles).
