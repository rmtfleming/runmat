# `ver` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/ver_observations.json`](observations/ver_observations.json)

## Summary

Return information about installed MathWorks products as a structure array; a product name returns the entry for that product.

## Confirmed invocation forms

- `s = ver(product)` — exercised in the experiment (case `matlab`).

Additional forms are proposed but not yet exercised:
- `s = ver` — pending.

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — the output class is `struct` for the observed inputs. *(cases: `matlab`)*
2. **Confirmed** — Returns a struct; ver('MATLAB') returned a 1-by-1 struct describing the product. The field names and contents are installation-dependent. *(cases: `matlab`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `matlab` | `struct` | `[1, 1]` | `<struct [1 1]>` |

## Unresolved questions

- `ver.q-install-dependent` — Contents depend on the installation; record only the structure/type.
- `ver.q-fields` — Confirm the field names of the returned structure.
