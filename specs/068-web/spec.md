# Feature Specification: `web`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`068-web`; export `SPECIFY_FEATURE=068-web`)

**Created**: 2026-11-02

**Status**: Batch N6, feature F68. Gates G1–G3 pre-approved by the
maintainer (`specs/001-clean-room-matlab-builtins/selection-proposal-n6.md`).

**Input**: Approved export `web` **0.2.0** (N6, source `e364206`; completed
on the spec side from a 0.1.0 draft stub that was never imported here); see
`provenance.md`.

Scope (Principle VII): one builtin. COBRA calls `web(url, '-browser')` and
`web(url, '-new', '-noaddressbox', '-notoolbar')` to show documentation
pages. The specification was observed with a local `file://` page only.

## Decisions

- **D-068-1 Error identifier namespace.** The observed
  `MATLAB:string:MustBeCharCellArrayOrString` is raised as
  `RunMat:string:MustBeCharCellArrayOrString` (maintainer rule; it names a
  generic string validator, `web.q-error-ids` is open on the spec side and
  flagged to the maintainer). Unobserved identifiers use
  `RunMat:web:<name>` (`UnsupportedOption`, `TooManyOutputs`, `Internal`).
- **D-068-2 Display, network and headless behaviour (labelled documented
  choices for `web.q-display`, `web.q-network`, `web.q-headless`; maintainer
  disposition "launching a browser goes through an injectable launcher
  (never in tests)").** RunMat has no embedded web browser. Every form —
  with or without `-browser` — builds a request (URL, target, flags) and
  hands it to a launcher. The launcher is injectable through
  `builtins::io::http::web::set_web_launcher`. The default launcher opens
  the URL in the system browser with the platform opener (`xdg-open` on
  Linux/BSD, `open` on macOS, `cmd /C start` on Windows); it does nothing for
  `web()` (empty URL), on Linux/BSD without `DISPLAY`/`WAYLAND_DISPLAY`
  (headless), and on wasm. `web` itself never contacts the network: http(s)
  URLs are passed to the launcher unchanged. Under `cfg(test)` the default
  launcher is a no-op, and the unit tests install a recording launcher; no
  test opens a browser or uses the network.
- **D-068-3 Status (documented choice for `web.q-stat`).** `stat` is 0 in
  every accepted call, including a missing file (as observed) and a launcher
  failure (not observed).
- **D-068-4 Handle (documented deviation for `web.q-handle`).** RunMat has
  no browser object. With the MATLAB-browser forms `h` is a non-empty 1×1
  structure with fields `Type = 'runmat.web.Browser'` and `URL` (the URL
  handed to the launcher). The observed class name is not reproduced
  (non-normative `web.handle-class`). The confirmed claim that `close(h)`
  accepts the handle is **not met**: RunMat's `close` dispatcher accepts
  figures and network objects only, and extending it is an upstream change
  outside this feature. Recorded as an `#[ignore]`d test.
- **D-068-5 Plain paths.** Text without a URL scheme (a one-letter scheme
  counts as a Windows drive letter) is treated as a file path, made
  absolute against the current folder and converted to a `file://` URL. The
  file need not exist.
- **D-068-6 Placement.** `crates/runmat-runtime/src/builtins/io/http/web.rs`
  next to `urlread`/`urlwrite` (hand-written `io/http/mod.rs`). Registered
  with `suppress_auto_output = true`, so the statement form shows nothing.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Show a page (Priority: P1)

**Acceptance Scenarios** (normative, one per observed case; a temporary
local HTML page, as in the observations):

1. `web(url, '-browser')` with no output completes and requests the system
   browser (`browser-cobra`). [web.signature-primary, web.system-browser]
2. `[stat, h] = web(url)`, `web(url, '-new')`, `web(url, '-notoolbar')` and
   `web(url, '-new', '-noaddressbox', '-notoolbar')` → `stat = 0` (double),
   `h` non-empty (`plain`, `new`, `notoolbar`, `cobra-new`).
   [web.matlab-browser]
3. A string-scalar URL, a plain path without `file://`, and no input →
   `stat = 0`, non-empty `h` (`string-url`, `plain-path`, `no-input`).
   [web.inputs]
4. `stat = web(url, '-browser')` → 0; `[stat, h] = web(url, '-browser')` →
   0 and `h = []` (double) (`browser-stat`, `browser-two-out`).
   [web.system-browser]
5. `stat = web(url, '-new')` → 0 (`one-out`). [web.one-out]
6. `web(42)` → `RunMat:string:MustBeCharCellArrayOrString`
   (`err-numeric`). [web.error-numeric]
7. *(non-normative)* A `file://` URL to a missing file → `stat = 0`
   (`missing-file`); an unrecognised option `'-nosuchoption'` is accepted
   (`unknown-option`). [web.missing-file, web.unknown-option]

## Requirements *(mandatory)*

### Functional Requirements (normative)

- **FR-068-01** `web(url, '-browser')` MUST be accepted with no output
  [web.signature-primary].
- **FR-068-02** The MATLAB-browser forms (no option, `-new`, `-notoolbar`,
  `-new -noaddressbox -notoolbar`) MUST return `stat = 0` (double) and a
  non-empty handle `h` [web.matlab-browser]. (`close(h)`: deviation D-068-4.)
- **FR-068-03** A string-scalar URL, a plain path and no input MUST be
  accepted with the same outputs [web.inputs].
- **FR-068-04** With `-browser`, `stat` is 0 with one or two outputs, and
  `h` is an empty double [web.system-browser].
- **FR-068-05** With one output, `stat` is 0 for the MATLAB-browser forms
  [web.one-out].
- **FR-068-06** A numeric URL MUST raise
  `RunMat:string:MustBeCharCellArrayOrString` [web.error-numeric].

### Functional Requirements (non-normative and documented choices)

- **FR-068-07** (non-normative `web.missing-file`) A missing file gives
  `stat = 0` and a handle; no existence check is made.
- **FR-068-08** (non-normative `web.unknown-option`) Unrecognised options
  are accepted and ignored (recorded in the request).
- **FR-068-09** (non-normative `web.handle-class`; D-068-4) The handle is a
  RunMat structure, not the observed desktop class.
- **FR-068-10** (documented choice D-068-3, `web.q-stat`) `stat` is 0 also
  when the launcher fails; the call with no output count returns 0 without
  display.
- **FR-068-11** (documented choice) More than two outputs raise
  `RunMat:web:TooManyOutputs`.
- **FR-068-12** (documented choice) Non-text options raise
  `RunMat:string:MustBeCharCellArrayOrString`; cell arrays, multi-row char
  and non-scalar string arrays raise `RunMat:web:UnsupportedOption`.
  Options are matched case-insensitively.
- **FR-068-13** (documented choice D-068-2, `web.q-display`,
  `web.q-network`, `web.q-headless`, `web.q-cobra-nargout0`) Every form goes
  through the injectable launcher; the default opens the system browser when
  a display is available. The COBRA MATLAB-browser form with no output
  behaves as the other forms (request launched, nothing returned).
- **FR-068-14** (documented choice D-068-5) Plain paths become absolute
  `file://` URLs.
- **FR-068-15** (unresolved, `web.q-warning-coverage`) No warning is
  emitted.

## Success Criteria *(mandatory)*

- **SC-068-1**: all 14 observed cases reproduce (outputs, classes, error
  identifier), with the handle under D-068-4.
- **SC-068-2**: `web` is registered through `#[runtime_builtin]`, and
  `cargo test -p runmat-runtime --lib -- web_tests` passes without opening a
  browser or contacting the network.
- **SC-068-3**: every FR maps to at least one test (`validation.md`).

## Unresolved behaviour (returned to the specification side)

| id | disposition |
|---|---|
| web.q-warning-coverage | no warning (FR-068-15) |
| web.q-network | URL passed to the launcher; `web` does no I/O (D-068-2) |
| web.q-display | system browser via the launcher (D-068-2) |
| web.q-stat | always 0 (D-068-3) |
| web.q-handle | structure handle; `close(h)` not supported (deviation D-068-4) |
| web.q-cobra-nargout0 | same as the other forms (FR-068-13) |
| web.q-harness-h | not applicable (the one-output forms return only `stat`) |
| web.q-headless | default launcher does nothing without a display (D-068-2) |
| web.q-error-ids | `RunMat:string:MustBeCharCellArrayOrString` (D-068-1); flagged |
