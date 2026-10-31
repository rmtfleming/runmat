# Validation — 064-file-io-n5

Date: 2026-10-31. MATLAB was not invoked. All commands ran with
`HDF5_DIR=/home/rfleming/anaconda3` and the shared runner script. No test
contacts the network: http tests use a loopback listener on 127.0.0.1.

## Files

| File | Change |
|---|---|
| `crates/runmat-runtime/src/builtins/io/filetext/fscanf.rs` | new (`fscanf` + `fscanf_tests`) |
| `crates/runmat-runtime/src/builtins/io/http/urlwrite.rs` | new (`urlwrite` + `urlwrite_tests`) |
| `crates/runmat-runtime/src/builtins/io/filetext/mod.rs` | hand-written: `pub mod fscanf;` |
| `crates/runmat-runtime/src/builtins/io/http/mod.rs` | hand-written: `pub mod urlwrite;` |
| `crates/runmat-runtime/src/builtins/strings/core/compat.rs` | visibility only (D-064-2): `ScanKind`, `IntegerRadix`, `ScanValue`, `ScanToken`, `parse_scan_format`, `scan_one`, `skip_whitespace` made `pub(crate)`; no body change |

No generated parent was edited.

## Analysis (B3)

- Every normative claim (17) maps to an FR, and every FR to at least one
  test (table below). No orphan tests; no task without a requirement.
  FR-064-17..19 are intentionally unused.
- Reuse survey: the `sscanf` scanner is reused through `pub(crate)` items
  (D-064-2); `sscanf_scan` is not reused, because it counts values, returns
  double only and has no `[m Inf]` shape. The fid comes from the `fopen`
  registry. `urlwrite` uses the shared transport like `urlread`.
- `fscanf.q-mixed` and `urlwrite.q-network` follow the maintainer
  dispositions in `selection-proposal-n5.md`.

## Test-first record (Principle V)

1. Tests, descriptors and bodies were written in the same files. Each
   builtin body began with a `// STUB` early `return Err(...)`.
2. Failing run: `cargo test -p runmat-runtime --lib -- fscanf_tests
   urlwrite_tests` gave `test result: FAILED. 2 passed; 33 failed`.
   fscanf_tests: 22 failed. urlwrite_tests: 11 failed. The 2 tests that
   passed were `documented_choice_routing` (pure routing helper) and
   `observed_err_one_arg`, which passed because the stub's error happened to
   use the `RunMat:minrhs` descriptor.
3. The stubs were removed. Passing run, same command: `test result: ok. 35
   passed; 0 failed`. fscanf_tests: 22 passed (17 observed tests for 16
   cases). urlwrite_tests: 13 passed (8 observed tests for 9 cases).

## Surrounding suites

- `cargo test -p runmat-runtime --lib -- builtins::io::filetext
  builtins::io::http builtins::strings::core`: **701 passed, 0 failed**
  (includes `sscanf`, `fgetl`, `fread`, `urlread`, `webread`).

## Lint and format

- `rustfmt --edition 2021 --check` on both new files: clean.
- `cargo clippy -p runmat-runtime --lib --tests`: exit 101 on the lib
  target because of `clippy::type_complexity` in another agent's in-progress
  `plotting/ops/rectangle.rs`. Second run with
  `-- -A clippy::type_complexity` reached the lib test target. It reported 2
  errors, both in other agents' in-progress files
  (`graph/domain/reordernodes.rs`, `math/rounding/rat.rs`). There were **no
  diagnostics in the F64 files or in `compat.rs`**, lib or tests.

## Claim-id check

`grep claim_id`: fscanf 10, urlwrite 7 = 17 ids, all cited in
`provenance.md`.

## Traceability (claim → FR → test → result)

| claim | FR | test(s) | result |
|---|---|---|---|
| fscanf.signature-primary | FR-064-01, 10 | `fscanf_tests::observed_c_all`, `observed_c_all_one_output` | pass |
| fscanf.c | FR-064-01 | `observed_c_all`, `observed_c_trailing_newline` | pass |
| fscanf.s | FR-064-02 | `observed_s_skip` | pass |
| fscanf.numeric | FR-064-03 | `observed_d_ints`, `observed_f_reals`, `observed_hex` | pass |
| fscanf.size | FR-064-04 | `observed_size_matrix`, `observed_size_scalar` | pass |
| fscanf.literal-stop | FR-064-05 | `observed_literal`, `observed_partial`, `observed_mismatch` | pass |
| fscanf.empty | FR-064-06 | `observed_empty_c`, `observed_empty_d` | pass |
| fscanf.mixed | FR-064-07 | `observed_mixed_d_s` | pass |
| fscanf.error-bad-fid | FR-064-08 | `observed_err_bad_fid` | pass |
| fscanf.error-no-format | FR-064-09 | `observed_err_no_format` | pass |
| urlwrite.signature-primary | FR-064-11 | `urlwrite_tests::observed_file_url` (dispatcher) | pass |
| urlwrite.success | FR-064-11 | `observed_file_url`, `observed_file_url_status` | pass |
| urlwrite.status-failure | FR-064-12 | `observed_missing_two_out`, `observed_protocol_two_out` | pass |
| urlwrite.error-missing-one-out | FR-064-13 | `observed_err_missing_one_out` | pass |
| urlwrite.error-bad-dir | FR-064-14 | `observed_err_bad_dir` (cases `err-bad-dir`, `err-bad-dir-two-out`) | pass |
| urlwrite.error-protocol | FR-064-15 | `observed_err_protocol` | pass |
| urlwrite.error-one-arg | FR-064-16 | `observed_err_one_arg` | pass |

Documented choices and unresolved points:

| FR | test(s) | result |
|---|---|---|
| FR-064-20 | `fscanf_tests::documented_choice_other_conversions` | pass |
| FR-064-21 | `documented_choice_size_shapes` | pass |
| FR-064-22 | `documented_choice_file_position` | pass |
| FR-064-23 | `documented_choice_arguments`, `documented_choice_write_only_fid` | pass |
| FR-064-24 (not observed; maintainer disposition) | `urlwrite_tests::documented_choice_http_loopback`, `documented_choice_http_failure`, `documented_choice_routing` | pass |
| FR-064-25 | `documented_choice_arguments`, `documented_choice_destination_is_folder` | pass |
| FR-064-26 | no warning path exists in either file; `observed_file_url` covers the observed no-warning case | by construction |

## Deviations

None. No upstream gap blocked an observed case.

## Freshness check (finish)

2026-10-31, `../matlab-interface-spec` after `git fetch -q origin`:
HEAD = origin/main = `1c91aa1`. `git log 1c91aa1..origin/main --
specifications/{fscanf,urlwrite}` lists nothing; `git status --short` for
those spec and experiment paths is clean (no drafts). The imported specs are
current.
