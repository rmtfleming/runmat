# `web` — behaviour specification

- **Spec version:** 0.2.0
- **Status:** approved 2026-11-02 (batch N6, new spec, human-loop review)
- **Applicable releases:** R2026a (26.1.0.3276743, GLNXA64)
- **Backing observations:** [`observations/web_observations.json`](observations/web_observations.json)

## Summary

Direct a page to the MATLAB web browser or, with '-browser', the system browser (display inferred, not checked by the experiment), returning a status of 0 and, for the MATLAB browser, a handle accepted by close. Observed with a local file:// page only.

## Confirmed invocation forms

- `web(url, '-browser')` — System browser, no output (COBRA form). *(e.g. case `browser-cobra`)*
- `[stat, h] = web(url, '-new', '-noaddressbox', '-notoolbar')` — MATLAB browser with the COBRA options; COBRA calls it with no output, which was not observed (only two outputs). *(e.g. case `cobra-new`)*
- `[stat, h] = web(url)` — MATLAB browser. *(e.g. case `plain`)*
- `stat = web(url, ...)` — Status only. *(e.g. case `one-out`)*
- `[stat, h] = web()` — No input. *(e.g. case `no-input`)*

## Required behaviour (confirmed)

Each item is backed by the cited black-box observation(s).
1. **Confirmed** — Opening in the MATLAB browser, with no option, '-new', '-notoolbar' or the COBRA combination '-new', '-noaddressbox', '-notoolbar', returned stat = 0 (double) and a non-empty handle h that close(h) accepted without error. *(cases: `plain`, `cobra-new`, `new`, `notoolbar`)*
2. **Confirmed** — A string scalar URL, a plain path without file://, and no input at all were each accepted, with stat = 0 and a closable handle. *(cases: `string-url`, `plain-path`, `no-input`)*
3. **Confirmed** — With '-browser', the call completed with no output; stat was 0 with one or two outputs, and with two outputs h was an empty double. *(cases: `browser-cobra`, `browser-stat`, `browser-two-out`)*
4. **Confirmed** — With one output, stat was 0 for the MATLAB browser. *(cases: `one-out`)*

## Edge cases (observed)

- *(non-normative)* A file:// URL to a file that does not exist did not error and gave stat = 0, the same as an existing file. *(cases: `missing-file`)*
- *(non-normative)* An unrecognised option ('-nosuchoption') did not error and gave stat = 0 and a closable handle. *(cases: `unknown-option`)*
- *(non-normative)* The MATLAB-browser handle's class was 'com.mathworks.mde.webbrowser.WebBrowser'; the class name is recorded only and not required. *(cases: `plain`)*

## Errors (confirmed)

| condition | identifier | case |
|---|---|---|
| url is numeric. | `MATLAB:string:MustBeCharCellArrayOrString` | `err-numeric` |

## Observed results

| case | class | size | value / error |
|------|-------|------|-------|
| `plain` | `struct` | `[1, 1]` | nout=2; stat=0; stat_class='double'; h_class='com.mathworks.mde.webbrowser.WebBrowser'; h_isempty=false; closed=true; completed=true |
| `cobra-new` | `struct` | `[1, 1]` | nout=2; stat=0; stat_class='double'; h_class='com.mathworks.mde.webbrowser.WebBrowser'; h_isempty=false; closed=true; completed=true |
| `new` | `struct` | `[1, 1]` | nout=2; stat=0; stat_class='double'; h_class='com.mathworks.mde.webbrowser.WebBrowser'; h_isempty=false; closed=true; completed=true |
| `notoolbar` | `struct` | `[1, 1]` | nout=2; stat=0; stat_class='double'; h_class='com.mathworks.mde.webbrowser.WebBrowser'; h_isempty=false; closed=true; completed=true |
| `one-out` | `struct` | `[1, 1]` | nout=1; stat=0; stat_class='double'; h_class='double'; h_isempty=true; closed=false; completed=true |
| `string-url` | `struct` | `[1, 1]` | nout=2; stat=0; stat_class='double'; h_class='com.mathworks.mde.webbrowser.WebBrowser'; h_isempty=false; closed=true; completed=true |
| `plain-path` | `struct` | `[1, 1]` | nout=2; stat=0; stat_class='double'; h_class='com.mathworks.mde.webbrowser.WebBrowser'; h_isempty=false; closed=true; completed=true |
| `missing-file` | `struct` | `[1, 1]` | nout=2; stat=0; stat_class='double'; h_class='com.mathworks.mde.webbrowser.WebBrowser'; h_isempty=false; closed=true; completed=true |
| `no-input` | `struct` | `[1, 1]` | nout=2; stat=0; stat_class='double'; h_class='com.mathworks.mde.webbrowser.WebBrowser'; h_isempty=false; closed=true; completed=true |
| `browser-cobra` | `struct` | `[1, 1]` | nout=0; stat=[]; stat_class=''; h_class=''; h_isempty=true; closed=false; completed=true |
| `browser-stat` | `struct` | `[1, 1]` | nout=1; stat=0; stat_class='double'; h_class='double'; h_isempty=true; closed=false; completed=true |
| `browser-two-out` | `struct` | `[1, 1]` | nout=2; stat=0; stat_class='double'; h_class='double'; h_isempty=true; closed=false; completed=true |
| `err-numeric` | — | — | error `MATLAB:string:MustBeCharCellArrayOrString` |
| `unknown-option` | `struct` | `[1, 1]` | nout=2; stat=0; stat_class='double'; h_class='com.mathworks.mde.webbrowser.WebBrowser'; h_isempty=false; closed=true; completed=true |

## Unresolved questions

- `web.q-warning-coverage` — Warnings were not captured except where a case reports a lastwarn identifier; an empty warnings list is not a complete statement.
- `web.q-network` — No network URL was contacted, so behaviour for COBRA's http URLs is unobserved.
- `web.q-display` — Whether and how the page was displayed was not checked by the experiment; only the outputs were recorded.
- `web.q-stat` — stat was 0 in every case, including a missing file; whether any input gives a non-zero status was not observed.
- `web.q-handle` — Must implementations return a handle that close() accepts? The observed handle is a desktop object.
- `web.q-cobra-nargout0` — The MATLAB-browser COBRA form with no output, and '-noaddressbox' alone, were not observed.
- `web.q-harness-h` — In the one-output cases the recorded h fields come from the harness (h = []), not from web.
- `web.q-headless` — Behaviour without a desktop display (as for a headless implementation) was not observed.
- `web.q-error-ids` — The numeric-input identifier is a generic string validator; whether an implementation must reproduce it is a human-review question (constitution 8.2).

Resolved in 0.2.0: `web.q-browser`, `web.q-outputs`, `web.q-observations-pending`.
