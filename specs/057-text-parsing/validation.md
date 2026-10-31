# Validation — 057-text-parsing

Date: 2026-10-30. MATLAB was not invoked. All commands ran with
`HDF5_DIR=/home/rfleming/anaconda3` and the shared runner script.

## Files

| File | Change |
|---|---|
| `crates/runmat-runtime/src/builtins/io/filetext/strread.rs` | new (`strread` + `strread_tests`) |
| `crates/runmat-runtime/src/builtins/strings/core/as_many_of_pattern.rs` | new (`asManyOfPattern` + `as_many_of_pattern_tests`) |
| `crates/runmat-runtime/src/builtins/io/filetext/mod.rs` | hand-written: `pub mod strread;` |
| `crates/runmat-runtime/src/builtins/strings/core/mod.rs` | hand-written: `pub mod as_many_of_pattern;` |

No generated parent was edited. `textread.rs` was not edited (D-057-2).

## Analysis (B3)

- Every normative claim (19) maps to an FR, and every FR to at least one
  test (table below). The non-normative `asManyOfPattern.empty-matches` maps
  to the labelled documented choice FR-057-27.
- No orphan tests. No task is without a requirement. FR numbers 10–12, 15
  and 20 are intentionally unused.
- Reuse survey: `textread.rs` has no public parser items, and its errors
  are `RunMat:textread:*`, so `strread` has its own tokenizer. The pattern
  representation (`compat::pattern_object` / `pattern_regex`) is reused
  unchanged.

## Test-first record (Principle V)

1. Tests and descriptors were written in the same file as the bodies, and
   each builtin body began with a `// STUB` early `return Err(...)`.
2. Failing run (shared with F55): `cargo test -p runmat-runtime --lib --
   sec_tests orth_tests condest_tests strread_tests
   as_many_of_pattern_tests` gave `test result: FAILED. 0 passed; 94
   failed`. For F57, strread_tests 19 failed and as_many_of_pattern_tests
   18 failed.
3. The stubs were removed. Passing run, same command: `test result: ok. 94
   passed; 0 failed`. For F57, strread_tests 19 passed (15 observed) and
   as_many_of_pattern_tests 18 passed (12 observed).

## Surrounding suites

- `cargo test -p runmat-runtime --lib -- builtins::math::trigonometry
  builtins::math::linalg builtins::strings builtins::io::filetext`:
  **2581 passed, 0 failed** (the `strings` and `io::filetext` suites include
  `textread_tests` and every pattern builtin's tests).
- `builtin::registry compatibility::tests`: 5 passed and 8 failed. These are
  the 7 known pre-existing failures plus
  `compatibility_version_pin_is_centralized_in_the_repository`, which
  points at another agent's in-progress `repl_fs/environment/release.rs`.
  None involves an F57 builtin.

## Lint and format

- `rustfmt --edition 2021 --check` on both new files: clean.
- `cargo clippy -p runmat-runtime --lib --tests`: **no diagnostics in the
  F57 files** for the lib target. The run ends with exit 101 because of
  errors in other agents' in-progress files (`plotting/ops/findall.rs`,
  `pcolor.rs`, `surfl.rs`). The `--tests` clippy pass is therefore not
  reached, and the test modules were not checked by this first run (see the second run).

- Second clippy run, with only the two lints raised in the other agents'
  plotting files allowed on the command line
  (`-- -A clippy::manual_is_multiple_of -A clippy::unnecessary_to_owned`)
  so the `--tests` pass is reached: the lib test target was checked and
  reported 4 errors, all in other agents' in-progress files
  (`graph/domain/isomorphism.rs`, `graph/domain/paths.rs`,
  `table/builtins/convertvars.rs`). There were **no diagnostics in the F57
  files, lib or tests**.

## Claim-id check

`grep claim_id`: strread 10, asManyOfPattern 10 = 20 ids, all cited in
`provenance.md`.

## Traceability (claim → FR → test → result)

| claim | FR | test(s) | result |
|---|---|---|---|
| strread.signature-primary | FR-057-01 | `strread_tests::observed_default` | pass |
| strread.default | FR-057-01 | `observed_default` | pass |
| strread.s | FR-057-02 | `observed_s`, `observed_num2str_form` | pass |
| strread.numeric | FR-057-03 | `observed_d`, `observed_f`, `observed_orientation` | pass |
| strread.cyclic | FR-057-04 | `observed_two_out`, `observed_two_out_mixed`, `observed_two_out_uneven` | pass |
| strread.delimiter | FR-057-05 | `observed_delimiter_comma`, `observed_delimiter_spaces` | pass |
| strread.empty | FR-057-06 | `observed_empty` | pass |
| strread.error-numeric | FR-057-07 | `observed_err_numeric` | pass |
| strread.error-bad-format | FR-057-08 | `observed_err_bad_format` | pass |
| strread.no-warning | FR-057-09 | `observed_warn` | pass |
| asManyOfPattern.signature-primary | FR-057-13 | `as_many_of_pattern_tests::observed_class` | pass |
| asManyOfPattern.class | FR-057-13 | `observed_class` (via RunMat `class`) | pass |
| asManyOfPattern.cobra-pattern | FR-057-14 | `observed_cobra_extract`, `observed_min1_no_empty`, `observed_matches` (RunMat `matches`), `observed_contains` (RunMat `contains`) | pass (D-057-3 stand-ins) |
| asManyOfPattern.greedy | FR-057-16 | `observed_default_min` (RunMat `lettersPattern`) | pass |
| asManyOfPattern.min-max | FR-057-17 | `observed_min_max` (RunMat `digitsPattern`) | pass |
| asManyOfPattern.text-pattern | FR-057-18 | `observed_string_input_min`, `observed_char_input` | pass |
| asManyOfPattern.empty-matches (non-normative) | FR-057-27 | `documented_choice_empty_matches` | pass |
| asManyOfPattern.error-min-gt-max | FR-057-19 | `observed_err_min_gt_max` | pass |
| asManyOfPattern.error-negative | FR-057-21 | `observed_err_negative`, `documented_choice_counts` | pass |
| asManyOfPattern.error-numeric | FR-057-22 | `observed_err_numeric` | pass |

Documented choices and unresolved points:

| FR | test(s) | result |
|---|---|---|
| FR-057-25 | `strread_tests::documented_choice_empty_numeric` | pass |
| FR-057-26 | `strread_tests::documented_choice_format_details`, `documented_choice_unsupported` | pass |
| FR-057-27 | `as_many_of_pattern_tests::documented_choice_empty_matches`, `documented_choice_text_is_literal` | pass |
| FR-057-28 | `documented_choice_counts` | pass |
| FR-057-29 | `strread_tests::documented_choice_arity_and_outputs`, `as_many_of_pattern_tests::documented_choice_arity_and_outputs` | pass |
| FR-057-30 | `strread_tests::documented_choice_unsupported`, `as_many_of_pattern_tests::documented_choice_unsupported` | pass |
| FR-057-31 | `as_many_of_pattern_tests::unresolved_no_warning`; `strread_tests::observed_warn` | pass |

## Deviations

- **D-057-3 (upstream gap, documented only).** RunMat has no `extract`
  builtin, no `alphanumericsPattern` and no `|` operator on pattern
  objects. The observed `asManyOfPattern` cases that use them are checked at
  regex level. A test helper takes the leftmost-first, non-overlapping
  matches of the pattern's regex (standing in for `extract`), and
  `regexpPattern("[\p{Alphabetic}\p{N}]+|_")` stands in for
  `alphanumericsPattern | "_"`. Adding those functions is out of scope.
  When upstream gains them, the tests should switch to the real builtins.
- **D-057-2 (reuse not possible).** `textread.rs` exposes no parser items,
  so `strread` does not reuse them; `textread.rs` is unchanged.

The open identifier question `strread.q-error-ids` is flagged to the
maintainer under D-057-1.

## Freshness check (finish)

2026-10-30, `../matlab-interface-spec` after `git fetch -q origin`:
HEAD = origin/main = `1c91aa1` (new commits since the start, none touching
these specs). `git log 1156c97..origin/main` over `specifications/{sec,orth,strread}`
and `git log fc9f804..origin/main` over `specifications/{condest,asManyOfPattern}`
list nothing; `git status --short` for those spec and experiment paths is
clean (no drafts). The imported specs are current.
