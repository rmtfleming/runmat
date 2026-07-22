# `urlread` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved (observed behaviour confirmed)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/urlread_observations.json`](observations/urlread_observations.json)

## Summary

Read the contents at a URL and return the result as text.

## Confirmed invocation forms

- `s = urlread(url)` — exercised in the experiment (case `file-url`).

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — Returns the resource contents as a char row vector (a local file:// URL returned 'hello urlread'). External http/https behaviour was not exercised. *(cases: `file-url`)*

## Observed results

| case | class | size | value |
|------|-------|------|-------|
| `file-url` | `char` | `[1, 13]` | `'hello urlread'` |

## Unresolved questions

- `urlread.q-network` — urlread performs network I/O; experiments require explicit approval and a stable, permissible endpoint. Not exercised here.
- `urlread.q-deprecated` — Does urlread emit a runtime deprecation notice, and what replacement does it indicate? Not exercised.
