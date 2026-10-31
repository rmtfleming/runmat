# `urlwrite` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-10-30 (batch N5, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/urlwrite_observations.json`](observations/urlwrite_observations.json)

## Summary

Save the content at a URL to a file and return the destination path; with two outputs, a missing file:// source or an unknown protocol gives status 0 instead of an error. Observed with local file:// URLs only.

## Confirmed invocation forms

- `f = urlwrite(url, filename)` — Save to filename and return its path (COBRA form). *(e.g. case `file-url`)*
- `[f, status] = urlwrite(url, filename)` — Also return a status. *(e.g. case `file-url-status`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — For a readable file:// URL the destination file was written with the same contents and f was the char destination path given; with two outputs status was double 1. No warning was recorded. *(cases: `file-url`, `file-url-status`)*
2. **Confirmed** — With two outputs, a missing source file or an unknown protocol did not error: f was empty char, status was 0 and no destination file was created. *(cases: `missing-two-out`, `protocol-two-out`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| With one output, the file:// source does not exist. | `MATLAB:urlwrite:FileNotFound` | `err-missing-one-out` |
| The destination folder does not exist (one or two outputs). | `MATLAB:urlwrite:InvalidOutputLocation` | `err-bad-dir`, `err-bad-dir-two-out` |
| With one output, the URL protocol is unknown. | `MATLAB:urlwrite:InvalidUrl` | `err-protocol` |
| Only the URL is given. | `MATLAB:minrhs` | `err-one-arg` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `file-url` | `struct` | `[1, 1]` | class_f='char'; f_is_dst=true; f_is_empty=false; status=[]; class_status='double'; dst_exists=true; contents='abc 123'; warning_id='' |
| `file-url-status` | `struct` | `[1, 1]` | class_f='char'; f_is_dst=true; f_is_empty=false; status=1; class_status='double'; dst_exists=true; contents='abc 123'; warning_id='' |
| `missing-two-out` | `struct` | `[1, 1]` | class_f='char'; f_is_dst=false; f_is_empty=true; status=0; class_status='double'; dst_exists=false; contents=''; warning_id='' |
| `err-missing-one-out` | — | — | error `MATLAB:urlwrite:FileNotFound` |
| `err-bad-dir` | — | — | error `MATLAB:urlwrite:InvalidOutputLocation` |
| `err-bad-dir-two-out` | — | — | error `MATLAB:urlwrite:InvalidOutputLocation` |
| `err-protocol` | — | — | error `MATLAB:urlwrite:InvalidUrl` |
| `protocol-two-out` | `struct` | `[1, 1]` | class_f='char'; f_is_empty=true; status=0; dst_exists=false |
| `err-one-arg` | — | — | error `MATLAB:minrhs` |

## Unresolved questions

- `urlwrite.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `urlwrite.q-network` — No network URL was contacted (http/https were not exercised), so download behaviour, as in COBRA's https call, is unobserved.
- `urlwrite.q-options` — Arguments beyond url and filename were not exercised.
