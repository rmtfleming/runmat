# Implementation Plan — 012-url-fetch

## Constitution Check

- I Specification-first: all normative requirements cite approved claims
  (`urlread.signature-primary`, `urlread.returns-char`); everything else is
  tagged summary-derived or independent choice — PASS
- II Clean-room: sources = batch-2 packaged export + RunMat code only; no
  MATLAB invocation anywhere (including tests) — PASS
- III Independence: implementation composes RunMat's own transport and
  filesystem layers; no MATLAB-internal imitation — PASS
- IV Provenance: `provenance.md` complete with authoritative commit pin
  `b054f3ad…` (batch-2; no waiver needed) — PASS
- V Test-first: observed-case and routing tests authored before the builtin
  body (T2–T3 before T4) — PASS
- VI Traceability: FR ↔ test mapping enforced in tasks; validation.md table
  at T7 — PASS
- VII Minimal scope: one builtin, thin adapter, no transport or filesystem
  refactoring — PASS
- VIII Human gates: this plan stops at Gate 2; no implementation before
  Gate 3 approval — PASS
- IX Compatibility: lives under `builtins/io/http/`, registered via
  `runtime_builtin` macro, follows `webread.rs`/`fileread.rs` conventions
  (descriptor, GPU/fusion specs, error descriptors, inline tests) — PASS
- X Licensing: no new dependencies; reuses in-tree `url`, `encoding_rs`,
  `reqwest` already reviewed for `webread` — PASS

## Affected crates and files

| Item | Path |
|---|---|
| `urlread` builtin | `crates/runmat-runtime/src/builtins/io/http/urlread.rs` (new) |
| Registration | add `pub mod urlread;` to `crates/runmat-runtime/src/builtins/io/http/mod.rs` |
| Type resolver | add `urlread_type` (char-row result, mirrors `fileread_type`) to `crates/runmat-runtime/src/builtins/io/type_resolvers.rs` |
| Docs metadata | descriptor + `#[runtime_builtin]` doc fields (feeds `docs/builtins` tooling) |

Exemplars: `io/http/webread.rs` (descriptor/error-descriptor/GPU-fusion-spec
shape, transport use, `CharArray::new_row` text output);
`io/filetext/fileread.rs` (filesystem read via `runmat_filesystem` provider,
`fs::read_async(path).await`).

No changes to `transport.rs`, `webread.rs`, `webwrite.rs`, or
`runmat-filesystem`.

## Design decisions

- **Signature**: `s = urlread(url)`; single required text argument (char row
  or string scalar, gathered via `gather_if_needed_async` like `webread`).
  Output mode `Fixed` with one output — the export lists exactly one output,
  so multi-output requests fail via the standard fixed-output machinery
  (documented choice; see spec Unresolved). [FR-012-01, FR-012-04]
- **URL parsing and routing**: parse with `url::Url::parse`; a pure
  function `route_for(url: &Url) -> Route { File | Http | Unsupported }`
  performs scheme classification so routing is unit-testable with no I/O.
  Empty/invalid URL text → `RunMat:urlread:InvalidUrl`; schemes other than
  `file`/`http`/`https` → `RunMat:urlread:UnsupportedScheme`. [FR-012-04]
- **`file://` path (normative, serves the observed case)**: convert via
  `Url::to_file_path()`; read bytes with `runmat_filesystem::read_async`
  (the exact pattern of `fileread.rs`), so the filesystem provider
  abstraction governs native and WASM alike. The HTTP transport is NOT
  involved. Decode bytes with the transport's `decode_body_as_text(bytes,
  None)` (BOM-aware, UTF-8 with lossy fallback — documented charset choice);
  return `Value::CharArray(CharArray::new_row(&text))`. Read failure →
  `RunMat:urlread:FileRead`. [FR-012-02]
- **http/https path (summary-derived)**: delegate to the existing transport
  exactly as `webread` does — build
  `HttpRequest { url, method: HttpMethod::Get, headers: vec![], body: None,
  timeout, user_agent }` and call `super::transport::send_request` (the
  `pub(crate)` entry point `webread.rs` imports from the sibling module).
  Timeout 60 s and user agent `"RunMat urlread/0.0"` mirror `webread`
  defaults (documented choices). The body is ALWAYS decoded as text via
  `decode_body_as_text(&response.body, content_type_header)` and returned as
  a char row — no JSON sniffing, no binary tensor branch (urlread returns
  text; this is the deliberate difference from `webread`). Transport failure
  (connect/timeout/HTTP status) → `RunMat:urlread:Transport`, wrapping the
  `TransportError` message. [FR-012-03, FR-012-04]
- **No deprecation warning** is emitted (`urlread.q-deprecated` unresolved;
  documented choice). [FR-012-04]
- **Error identifiers** are RunMat-native (`RM.URLREAD.*` codes,
  `RunMat:urlread:*` identifiers) — the export records no error behaviour,
  so no MATLAB error compatibility is claimed. [FR-012-04]
- **Accel/fusion**: `accel = "sink"`, GPU spec `Custom("url-fetch")` with
  `GatherImmediately` residency and a fusion-terminating fusion spec,
  mirroring `webread` (I/O never runs on accelerators).
- **WASM**: http/https uses the transport's existing XHR implementation
  unchanged. For `file://`, `Url::to_file_path()` semantics off native
  targets are an implementation-time check (T1): if conversion is not
  dependable under `wasm32`, the wasm build maps the URL path component onto
  the `runmat_filesystem` provider path space, or returns a clear
  `RunMat:urlread:FileRead` error — either way behaviour is explicit, never
  silent. [FR-012-04]

## SANDBOX / TEST-ISOLATION (binding for this feature)

- Tests use `file://` URLs over temporary files ONLY, created under
  `std::env::temp_dir()` (via the existing test-support fs helpers used by
  `fileread.rs` tests) and removed before the test returns (self-cleaning).
- **NO network access in any test** — no external endpoints AND no loopback
  (`TcpListener`) servers, even though sibling `webread.rs` tests use
  loopback. The http/https code path in `urlread` is a thin delegation that
  adds no new transport behaviour; its coverage relies on:
  (a) the existing loopback tests of the shared transport in
  `io/http/webread.rs` and `io/http/webwrite.rs` (request construction,
  status/timeout/connect errors, charset decoding via
  `decode_body_as_text`), and
  (b) urlread's own pure-function tests (scheme routing, request-parameter
  construction) that perform no I/O.
- Review guard: a task (T6) greps the new file's test module for
  `TcpListener`/`http://`/`https://` fetches to confirm the constraint
  before validation is recorded.

## Test plan (inline `#[cfg(test)]`, per house convention)

Normative (cite FR/claims in test names or comments):
- `urlread_file_url_returns_observed_char_row` — write `hello urlread` to a
  temp file, build its `file://` URL, expect `Value::CharArray`, size 1×13,
  exact value. [FR-012-01, FR-012-02; SC-012-1]
- Descriptor/registration checks (signature label `s = urlread(url)`;
  discoverable via `builtin_function_by_name`). [FR-012-01; SC-012-2]

Unresolved-tracking (tagged `unresolved_choice` in name):
- empty temp file → 1×0 char row; missing file → `FileRead` error;
  non-text-scalar or empty URL argument → `InvalidUrl`/`InvalidArgument`
  error; `ftp://` scheme → `UnsupportedScheme` error. [FR-012-04]

Summary-derived, no I/O (tagged `summary_derived`):
- `route_for` maps `http`/`https` URLs to the transport route and `file`
  to the filesystem route (pure function — no request is sent). [FR-012-03]

## Risks & rollback

- Risk: `Url::to_file_path()` behaviour for percent-encoded paths and on
  `wasm32` — resolved at T1 by inspection; fallback documented above.
- Risk: `decode_body_as_text` is `pub(crate)` in `transport.rs` and already
  visible from sibling modules (as `webread.rs` proves); if visibility were
  ever narrowed, urlread would need its own decoder — no change planned now.
- Risk: divergence temptation — implementing the historical multi-output or
  error-message behaviour from memory is PROHIBITED; anything beyond the
  export goes through the Unresolved list. Scope changes return to Gate 1.
- Rollback: delete `urlread.rs`, the one-line `mod` entry, and the
  `urlread_type` resolver; no existing behaviour is touched.

## Verification commands

`cargo fmt --all -- --check` · `cargo check -p runmat-runtime` ·
`RUST_TEST_THREADS=1 cargo test -p runmat-runtime urlread`
