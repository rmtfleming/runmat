# Tasks — 012-url-fetch

Order is normative (test-first). Every task cites requirements. No task may
introduce network access into tests (plan: SANDBOX/TEST-ISOLATION; SC-012-4).

- [x] **T1** Substrate inspection (no code changes): confirm
  `transport::{send_request, decode_body_as_text, header_value, HttpRequest,
  HttpMethod, HEADER_CONTENT_TYPE}` visibility from a sibling module of
  `io/http/`; confirm `runmat_filesystem::read_async` availability (pattern
  in `io/filetext/fileread.rs`); check `Url::to_file_path()` behaviour for
  percent-encoded paths and under `wasm32`; record the wasm `file://`
  decision from the plan. [FR-012-02, FR-012-03, FR-012-04]
- [x] **T2** Author failing normative tests in the new file's test module
  (red step; compile gated by T4): observed case
  `urlread_file_url_returns_observed_char_row` — temp file containing
  `hello urlread`, `file://` URL, expect char row 1×13 exact value; temp
  file self-cleans; descriptor/registration assertions. [FR-012-01,
  FR-012-02; SC-012-1, SC-012-2]
- [x] **T3** Author failing choice/routing tests: `unresolved_choice`-tagged
  cases (empty file → 1×0 char; missing file → FileRead error; empty/invalid
  URL → InvalidUrl; `ftp://` → UnsupportedScheme) and
  `summary_derived`-tagged pure routing tests for `route_for` (http/https →
  transport route, file → filesystem route; NO request sent, NO server
  spawned). [FR-012-03, FR-012-04; SC-012-4]
- [x] **T4** Implement `crates/runmat-runtime/src/builtins/io/http/urlread.rs`:
  descriptor (single signature `s = urlread(url)`, Fixed output), error
  descriptors (`RM.URLREAD.*` / `RunMat:urlread:*`), GPU + fusion specs
  (sink), `route_for`, `file://` read via `runmat_filesystem::read_async` +
  `decode_body_as_text(bytes, None)` + `CharArray::new_row`, http/https GET
  via `transport::send_request` with body always decoded as text; register
  via `#[runtime_builtin]`, add `pub mod urlread;` to `io/http/mod.rs`, add
  `urlread_type` resolver. [FR-012-01, FR-012-02, FR-012-03, FR-012-04]
- [x] **T5** WASM handling per T1 decision: explicit `file://` behaviour
  (provider-mapped read or clear FileRead error) under `wasm32`; verify
  native build unaffected (`cargo check -p runmat-runtime`). [FR-012-04]
- [x] **T6** Isolation guard: grep the new file's `#[cfg(test)]` module for
  `TcpListener`, `spawn_server`, `http://`, `https://` usage in executed
  fetches — must be absent (pure routing tests may name schemes in parsed
  URLs only, with no request sent); record the check output. [SC-012-4]
- [x] **T7** Run verification commands (`cargo fmt --all -- --check`,
  `cargo check -p runmat-runtime`, `RUST_TEST_THREADS=1 cargo test -p
  runmat-runtime urlread`); record real output in `validation.md` — no
  fabricated results (Principle V). [SC-012-1, SC-012-2]
- [x] **T8** Traceability table in `validation.md` (claim → FR → test →
  result), including summary-derived and unresolved_choice tags; note that
  http-path conformance is NOT claimed (`urlread.q-network` open).
  [SC-012-3; Principle VI]
