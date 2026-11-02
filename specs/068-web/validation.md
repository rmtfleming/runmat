# Validation — 068-web

Date: 2026-11-02. MATLAB was not invoked. No test opened a browser or
contacted the network: every test installs a recording launcher, and the
default launcher is a no-op under `cfg(test)`. All commands ran with
`HDF5_DIR=/home/rfleming/anaconda3` and the shared runner script.

## Files

| File | Change |
|---|---|
| `crates/runmat-runtime/src/builtins/io/http/web.rs` | new (`web`, launcher hook `set_web_launcher`, `web_tests`) |
| `crates/runmat-runtime/src/builtins/io/http/mod.rs` | hand-written: `pub mod web;` |

No generated parent `mod.rs` was touched.

## Analysis (B3)

- Every normative claim (6) maps to an FR and a test; the three
  non-normative claims map to FR-068-07..09. The one part of a normative
  claim RunMat does not meet (`close(h)` accepting the handle, in
  `web.matlab-browser`) is recorded as deviation D-068-4 with an
  `#[ignore]`d test.
- No orphan tests; no task without a requirement. Scope: one new file and
  one module line; no upstream code changed.
- Maintainer dispositions applied: observed `file://` outputs reproduced;
  launching goes through an injectable launcher, never in tests; headless
  behaviour a labelled choice; `q-error-ids` raised under `RunMat:`.

## Test-first record (Principle V)

1. `web_builtin` began with `if true { return Err(web_error(&INTERNAL,
   "stub")); } // STUB` (run together with F67).
2. Failing run, `cargo test -p runmat-runtime --lib -- normest_tests
   lsqnonneg_tests web_tests`: `test result: FAILED. 0 passed; 64 failed;
   1 ignored`; web_tests 19 failed, 1 ignored.
3. Stub removed. Passing run: `test result: ok. 64 passed; 0 failed;
   1 ignored`; web_tests 19 passed (14 observed), 1 ignored
   (`deviation_close_handle`).

## Surrounding suites

- `cargo test -p runmat-runtime --lib -- builtins::math::linalg
  builtins::math::optim builtins::io::http`: **1215 passed, 0 failed,
  1 ignored**.

## Lint and format

- `rustfmt --edition 2021 --check` on `web.rs`: clean.
- `cargo clippy -p runmat-runtime --lib --tests`: exit 0, no diagnostics.

## Claim-id check

`grep claim_id` over the imported `provenance.yaml`: 9 ids, all cited.

## Traceability (claim → FR → test → result)

| claim | FR | test(s) (`web_tests::`) | result |
|---|---|---|---|
| web.signature-primary | FR-068-01 | `observed_browser_cobra` | pass |
| web.matlab-browser | FR-068-02 | `observed_plain`, `observed_cobra_new`, `observed_new`, `observed_notoolbar` (stat 0 double, non-empty handle) | pass |
| web.matlab-browser (`close(h)`) | D-068-4 | `deviation_close_handle` | ignored (deviation) |
| web.inputs | FR-068-03 | `observed_string_url`, `observed_plain_path`, `observed_no_input` | pass |
| web.system-browser | FR-068-01, 04 | `observed_browser_cobra`, `observed_browser_stat`, `observed_browser_two_out` (`h = []` double) | pass |
| web.one-out | FR-068-05 | `observed_one_out` | pass |
| web.error-numeric | FR-068-06 | `observed_err_numeric` | pass |
| web.missing-file (non-normative) | FR-068-07 | `observed_missing_file` | pass |
| web.unknown-option (non-normative) | FR-068-08 | `observed_unknown_option` | pass |
| web.handle-class (non-normative) | FR-068-09 | `assert_handle` in the MATLAB-browser tests (`Type = 'runmat.web.Browser'`) | pass |

| FR | test(s) | result |
|---|---|---|
| FR-068-10 | `documented_choice_launch_failure_and_statement` | pass |
| FR-068-11, 12 | `documented_choice_outputs_and_text_forms` | pass |
| FR-068-13 | `documented_choice_network_url_passed_through`, `documented_choice_default_launcher_inert_in_tests`, every observed test (request recorded with its target) | pass |
| FR-068-14 | `observed_plain_path` | pass |
| FR-068-15 | `unresolved_no_warning` | pass |

## Deviations

- **D-068-4** `web.q-handle` / `web.matlab-browser`: the handle is a RunMat
  structure, and `close(h)` raises an error in RunMat (the `close`
  dispatcher accepts figures and network objects only). Supporting it needs
  a change to `builtins/close` (upstream subsystem), not made here. Test
  `web_tests::deviation_close_handle` is `#[ignore]`d.

The default launcher (D-068-2) is not exercised by any test, by design;
its real-browser path is untested in CI.

## Freshness check (finish)

2026-11-02, `../matlab-interface-spec` after `git fetch origin`:
HEAD = origin/main = `20825f8`; nothing under `specifications/web` changed
since `e364206` and there are no drafts. The imported spec is current.
