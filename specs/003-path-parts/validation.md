# Validation Report — 003-path-parts — 2026-07-22

## Commands and results

- `cargo check -p runmat-runtime` — clean
- `cargo fmt --all -- --check` — clean
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::io::repl_fs::fileparts` — **8 passed, 0 failed**

## Traceability

| Claim | FR | Test | Result |
|---|---|---|---|
| fileparts.first-output-dir, output-class ('/a/b/c.txt' → '/a/b' 1×4) | FR-003-03 | `observed_full_path_dir` | PASS |
| fileparts.first-output-dir ('file.m' → '' 0×0) | FR-003-03 | `observed_bare_name_dir_is_empty` | PASS |
| fileparts.first-output-dir ('/x/y/' → '/x/y') | FR-003-03 | `observed_trailing_separator_removed` | PASS |
| fileparts.first-output-dir ('noext' → '' 0×0) | FR-003-03 | `observed_no_extension_dir_is_empty` | PASS |
| (summary) name/ext outputs | FR-003-04 | `summary_derived_three_output_split` | PASS (summary-derived) |
| unresolved choices (dotfiles, multi-dot, root, string input) | FR-003-05 | `unresolved_choice_edge_splits`, `unresolved_choice_string_input_returns_string` | PASS (non-normative) |
| input validation | — | `invalid_input_errors` | PASS |

All four observed cases reproduce exactly including empty-char shape 0×0.
Multi-output via `BuiltinOutputMode::ByRequestedOutputCount`; nargout==1
returns the directory part, matching the observation harness's first-output
recording.

## Deviations and notes

- Documented choices: last-separator/last-dot split; leading-dot names have
  no extension; root-relative `'/a'` yields dir `'/'`; `\` recognised only on
  Windows builds.

## Unresolved (returned to specification side)

`fileparts.q-outputs` (observations for outputs 2–3), `fileparts.q-platform`.
