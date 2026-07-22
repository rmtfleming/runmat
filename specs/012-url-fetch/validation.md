# Validation Report — 012-url-fetch

Date: 2026-07-22. Implementation approved at the batch gate (documented
choices ratified, including omission of any historical status output).
All results below are from actual runs in this repository; no MATLAB was
invoked at any point.

## Files changed

- `crates/runmat-runtime/src/builtins/io/http/urlread.rs` (new — builtin,
  descriptor, error descriptors, GPU/fusion specs, `urlread_type` resolver,
  inline test module)
- `crates/runmat-runtime/src/builtins/io/http/mod.rs` (one line:
  `pub mod urlread;`)
- No other source files touched; `transport.rs`, `webread.rs`,
  `webwrite.rs`, `runmat-filesystem` unchanged. No new dependencies.

## Verification commands (actual output)

- `cargo check -p runmat-runtime` →
  `Finished 'dev' profile [unoptimized + debuginfo] target(s) in 4m 10s`
  (no warnings or errors for urlread).
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::io::http::urlread` →
  `test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 6644 filtered out; finished in 0.01s`
- `rustfmt --edition 2021 --check` on the two changed files → clean.
- `cargo fmt --all -- --check` → exit 0, zero diffs (whole tree clean at
  time of run; sibling features were implementing concurrently).

## Isolation guard (SC-012-4, plan SANDBOX/TEST-ISOLATION)

`grep -n "TcpListener\|spawn_server" urlread.rs` → no matches (PASS).
`grep -n "http://\|https://" urlread.rs` → 3 matches, all non-fetching:
line 49 (descriptor description text) and lines 534–535 (`Url::parse`
inputs to the pure `route_for` classification test; no request constructed
or sent). All test I/O uses self-cleaning `file://` temp files under
`std::env::temp_dir()`. The http/https delegation path is exercised only by
the shared transport's pre-existing loopback tests in
`webread.rs`/`webwrite.rs`; urlread adds no new transport behaviour.

## Traceability (claim → FR → test → result)

| Claim / basis | FR | Test (`builtins::io::http::urlread::tests::`) | Result |
|---|---|---|---|
| urlread.signature-primary (public-interface-fact) | FR-012-01 | `urlread_registered_with_single_signature` | ok |
| urlread.signature-primary + urlread.returns-char (black-box-observation, case `file-url`) | FR-012-01, FR-012-02 | `urlread_file_url_returns_observed_char_row` (exact: `'hello urlread'`, char, 1×13) | ok |
| Approved summary text ("…return the result as text") — summary-derived | FR-012-03 | `urlread_route_for_maps_schemes_summary_derived` (pure routing; no request) | ok |
| Independent choice (unresolved: empty file) | FR-012-04 | `urlread_empty_file_returns_empty_char_row_unresolved_choice` (1×0 char) | ok |
| Independent choice (unresolved: percent-encoded file path) | FR-012-04 | `urlread_file_url_with_space_resolves_percent_encoding_unresolved_choice` | ok |
| Independent choice (unresolved: missing file) | FR-012-04 | `urlread_missing_file_errors_unresolved_choice` | ok |
| Independent choice (unresolved: empty URL) | FR-012-04 | `urlread_rejects_empty_url_unresolved_choice` | ok |
| Independent choice (unresolved: non-text argument) | FR-012-04 | `urlread_rejects_non_text_url_argument_unresolved_choice` | ok |
| Independent choice (unresolved: unparseable URL) | FR-012-04 | `urlread_rejects_unparseable_url_unresolved_choice` | ok |
| Independent choice (unresolved: unsupported scheme) | FR-012-04 | `urlread_rejects_unsupported_scheme_unresolved_choice` | ok |
| Implementation support (wasm file-path fallback helper, pure) | FR-012-04 | `urlread_percent_decode_path_handles_escapes` | ok |

Success criteria: SC-012-1 (observed case exact) PASS; SC-012-2
(registration via `builtin_function_by_name` + test module green) PASS;
SC-012-3 (every FR cited by ≥1 test, every test cites an FR) PASS;
SC-012-4 (no network in any test) PASS via guard above.

## Conformance boundaries (unchanged from spec)

- http/https fetching is summary-derived; MATLAB conformance is NOT claimed
  (`urlread.q-network` open on the specification side).
- No deprecation notice is emitted (`urlread.q-deprecated` open).
- Single output only; error identifiers (`RunMat:urlread:*`), charset
  handling, scheme set, and timeout are documented independent choices.

## Deviations from plan/tasks

1. `urlread_type` lives in `urlread.rs` (referenced by macro path) instead
   of `io/type_resolvers.rs` as planned — deliberate, to keep all edits
   inside this feature's own file plus the single `mod.rs` line while four
   sibling features implement concurrently in the same tree. Functionally
   identical (`Type::String`, matching `fileread_type`).
2. Strict red-step (T2/T3 failing-test run before T4) was not separately
   recorded: tests live in the implementation file's inline `#[cfg(test)]`
   module per house convention, so they compile only with the builtin body
   present (this compile-gating was anticipated in tasks.md T2). Tests were
   authored with the implementation and all 11 passed on first full run.
3. Formatting was applied per-file (`rustfmt` on the two changed files)
   rather than `cargo fmt -p runmat-runtime`, to avoid rewriting sibling
   agents' in-progress files; the global `cargo fmt --all -- --check` was
   then confirmed clean.

Constitution notes: Principle V satisfied (all reported outcomes are from
real runs above); Principle X — no new external code or dependencies were
introduced; independent legal review MAY still be required per constitution.
No commit/push/PR performed (publication approval is separate).
