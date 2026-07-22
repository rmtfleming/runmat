# Feature Specification: URL Fetch — `urlread`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id: `012-url-fetch`; export `SPECIFY_FEATURE=012-url-fetch`)

**Created**: 2026-07-22

**Status**: Batch 3 (F12) — B1 authored, review pending

**Input**: Approved batch-2 export `urlread` 0.2.0 (`status: approved`,
packaged export `runmat-export-2026-07-22`; see `provenance.md`). Selection
and ordering approved in
`specs/001-clean-room-matlab-builtins/selection-proposal-batch3.md` (F12).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Read the contents at a URL as text (Priority: P1)

RunMat users calling `s = urlread(url)` get the contents of the resource at
`url` back as a char row vector, matching the approved observed behaviour.

**Independent Test**: reproduce the export's single observed case with a
local temporary file: write `hello urlread` to a temp file, call
`urlread('file://<tempfile>')`, and compare value, class and shape.

**Acceptance Scenarios** (from observed results, normative):

1. **Given** a local file reachable via a `file://` URL whose contents are
   `hello urlread`, **When** `urlread` is called with that URL, **Then** the
   result is `'hello urlread'`, class `char`, size 1×13 (char row vector).
   [urlread.signature-primary, urlread.returns-char]

### User Story 2 - Fetch over HTTP/HTTPS (Priority: P2, summary-derived)

The approved export's summary describes `urlread` as reading the contents at
a URL and returning the result as text, but **no http/https case was
exercised** (`urlread.q-network`). RunMat serves http/https URLs through its
existing HTTP transport and returns the body as a char row vector. This is
summary-derived behaviour: it MUST NOT be asserted as MATLAB-conformant, and
it MUST NOT be exercised against any network endpoint in this feature's
tests (see Success Criteria SC-012-4).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-012-01** `urlread` MUST accept the invocation form `s = urlread(url)`
  with one required URL text argument (char row vector or string scalar)
  [urlread.signature-primary].
- **FR-012-02** For a `file://` URL naming an existing readable local file,
  `urlread` MUST return the file contents as a char row vector: the observed
  case (file containing `hello urlread`) MUST reproduce exactly as `char`,
  size 1×13, value `'hello urlread'` [urlread.returns-char].
- **FR-012-03** (summary-derived) For http/https URLs, `urlread` SHOULD
  fetch the resource through RunMat's existing HTTP transport and return the
  body as a char row vector, consistent with the approved summary ("read the
  contents at a URL and return the result as text"). External http/https
  behaviour was not exercised by the export (`urlread.q-network`); no test
  may contact any network endpoint, and conformance is not claimed.
- **FR-012-04** Behaviour the export does not cover MUST be handled by
  documented independent choice (see Unresolved behaviour) and MUST NOT be
  asserted as MATLAB-conformant. This includes at minimum: error conditions
  and identifiers (invalid URL text, unsupported scheme, missing/unreadable
  file, transport failure), any second output, deprecation warnings, charset
  handling, timeouts, and non-file/non-http schemes.

### Key Entities

- Result: `Value::CharArray` built as a row vector (RunMat's char row), the
  same representation `webread` uses for text responses.
- `file://` reads go through RunMat's runtime filesystem layer
  (`runmat-filesystem`), never through the HTTP transport and never through
  MATLAB.
- http/https reads delegate to the existing `io/http/transport.rs`
  request/response machinery shared with `webread`/`webwrite`.

## Success Criteria *(mandatory)*

- **SC-012-1**: The export's observed case (`file-url`) reproduces exactly
  (value `'hello urlread'`, class char, size 1×13) in a normative test using
  a self-cleaning temporary file.
- **SC-012-2**: `urlread` is registered and discoverable via
  `builtin_function_by_name`; `cargo test -p runmat-runtime urlread` passes.
- **SC-012-3**: Traceability: each FR cites ≥1 claim id (or is tagged
  summary-derived/independent choice); each normative test cites ≥1 FR.
- **SC-012-4**: Test isolation: NO test in this feature performs network
  access of any kind — no external endpoints and no loopback servers. All
  I/O in tests is `file://` over temporary files that self-clean. The
  http/https code path is covered only by (a) pure-function unit tests that
  perform no I/O and (b) the pre-existing loopback tests of the shared
  transport in `webread.rs`/`webwrite.rs`.

## Unresolved behaviour (non-normative; implemented by documented choice)

- `urlread.q-network` (carried from export): http/https fetching was not
  exercised. Choice: delegate GET requests to the existing transport and
  return the decoded body as a char row (FR-012-03, summary-derived).
- `urlread.q-deprecated` (carried from export): whether `urlread` emits a
  runtime deprecation notice is unknown. Choice: RunMat emits no warning;
  revisit if the specification side resolves the question.
- Second output: the export's interface lists exactly one output (`s`). Any
  multi-output form (e.g. a status flag) is absent from the approved export
  and MUST NOT be implemented from memory. Choice: single output only; a
  request for more outputs fails via RunMat's standard fixed-output
  machinery. Candidate follow-up question for the specification side.
- Error behaviour: the export records no error cases (`errors: []`). Choice:
  RunMat-native error identifiers under `RunMat:urlread:*` for invalid URL
  text, unsupported scheme, unreadable file, and transport failure; no
  MATLAB message/identifier compatibility is claimed.
- Charset/decoding: unobserved. Choice: http/https bodies decode via the
  transport's existing Content-Type/BOM-aware text decoding; `file://` bytes
  decode BOM-aware with UTF-8 (lossy) fallback.
- Schemes other than `file`, `http`, `https`: unobserved. Choice: rejected
  with a clear error.
- Empty-file and empty-URL inputs: unobserved. Choice: empty file yields an
  empty char row (0-length); empty URL text errors.

## Assumptions

- The existing `io/http/transport.rs` request machinery (used by `webread`)
  is reusable from a sibling module without modification; `urlread` adds no
  new transport behaviour.
- `runmat-filesystem` provides byte reads (`read`/`read_async`) as already
  used by `io/filetext/fileread.rs`, so native and WASM builds route file
  access consistently through the provider abstraction.
- No new crate dependencies are required (`url`, `encoding_rs`, `reqwest`
  are already in the runtime's dependency tree).
