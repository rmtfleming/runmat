# Validation Report — 037-findstr (spec 0.3.0) — 2026-10-29

Environment: as for `specs/039-svds/validation.md` (same session, shared
target directory, runner wrapper, MATLAB not invoked).

## Freshness (T0)

In `../matlab-interface-spec` (read-only): `git fetch -q --all` (exit 0);
HEAD = origin/main = `8614c5b…`. The `git diff --name-only 8614c5b
{HEAD,origin/main} -- specifications/findstr` checks returned nothing, and
`git status --short` was clean. Result: the import is current.

## History

A first draft against 0.2.0 (pin `104fb5f`) had a test-first run of 0 passed /
9 failed (all `RunMat:UndefinedFunction`). That run is superseded. The
feature was redone from 0.3.0, which resolves `findstr.q-order`.

## Test-first evidence (T4)

The 0.2.0-era implementation was removed. `findstr.rs` contained only the
0.3.0 tests, and `pub mod findstr;` was re-added.

`cargo test -p runmat-runtime --lib -- svds_tests findstr_tests` →
**0 passed, 47 failed**, of which findstr accounts for **21**. Every failure
was `RunMat:UndefinedFunction`.

## Commands and results (T7)

- `cargo test -p runmat-runtime --lib -- svds_tests findstr_tests` →
  **47 passed, 0 failed**, including all 21 findstr tests:
  - observed (17): `observed_occurrences`, `observed_swapped`,
    `observed_overlapping`, `observed_no_match`,
    `observed_equal_length_match`, `observed_equal_length_nomatch`,
    `observed_empty_pattern`, `observed_both_empty`, `observed_empty_first`,
    `observed_no_warning`, `observed_case_sensitive`,
    `observed_single_char`, `observed_string_scalar`, `observed_numeric`,
    `observed_multi_row_char_errors`, `observed_cell_errors`,
    `observed_one_argument_errors`
  - documented choice / unresolved (4): `documented_choice_text_forms`,
    `documented_choice_numeric_forms`,
    `documented_choice_other_inputs_and_arity`,
    `unresolved_unobserved_classes_are_unsupported`
- `cargo test -p runmat-runtime --lib -- strings::search::` → 137 passed,
  0 failed (upstream `strfind`, `contains` and the other siblings show no
  regression).
- `cargo test -p runmat-runtime --lib -- compatibility::` → the six known
  upstream failures only.
- `cargo test -p runmat-runtime --lib -- builtin::registry::` → 1 failed.
  It reports 76 identity errors for unrelated builtins and none for
  `findstr` or `svds`.
- `cargo clippy -p runmat-runtime --lib --tests --message-format short`:
  - The first run flagged `findstr.rs:229` (`len() == 0` → `is_empty()`).
  - After the fix, clippy finished with no errors and no diagnostics in
    `findstr.rs`.
- `rustfmt --edition 2021 --check …/search/findstr.rs` → clean.
- I checked all 14 cited `findstr.*` claim ids against the imported
  `provenance.yaml`, and all are present. The superseded
  `findstr.q-order` is cited only as resolved.

## Traceability (claim → FR → test → result) (T8)

| Claim / basis | FR | Test(s) | Result |
|---|---|---|---|
| findstr.signature-primary, findstr.output-class | FR-037-01 | `observed_occurrences`, `observed_no_match`, `observed_numeric` | PASS |
| findstr.start-indices | FR-037-02 | `observed_occurrences`, `observed_single_char` | PASS |
| findstr.overlapping | FR-037-02 | `observed_overlapping` | PASS |
| findstr.order-independent | FR-037-03 | `observed_swapped` | PASS |
| findstr.equal-length | FR-037-03 | `observed_equal_length_match`, `observed_equal_length_nomatch` | PASS |
| findstr.no-match | FR-037-04 | `observed_no_match`, `observed_empty_pattern`, `observed_empty_first`, `observed_both_empty`, `observed_equal_length_nomatch`, `observed_case_sensitive` | PASS |
| findstr.case-sensitive | FR-037-04 | `observed_case_sensitive` | PASS |
| findstr.string-scalar | FR-037-05 | `observed_string_scalar` | PASS |
| findstr.numeric | FR-037-05 | `observed_numeric` | PASS |
| findstr.no-warning | FR-037-06 | `observed_no_warning` | PASS |
| findstr.error-column-char, findstr.error-cell, findstr.error-one-arg | FR-037-07 | `observed_multi_row_char_errors`, `observed_cell_errors`, `observed_one_argument_errors` | PASS |
| documented choice: mixed char/string, 1-element string array, char block | FR-037-08 | `documented_choice_text_forms` | PASS |
| documented choice: numeric scalar/empty/swapped/NaN/multi-row | FR-037-09 | `documented_choice_numeric_forms` | PASS |
| documented choice: struct, string array, >2 args | FR-037-10 | `documented_choice_other_inputs_and_arity` | PASS |
| unobserved classes (U-037-1) | FR-037-11 | `unresolved_unobserved_classes_are_unsupported` | PASS |
| unresolved findstr.q-deprecated, q-warning-coverage | FR-037-12 | `observed_no_warning` | PASS |

All 17 observed cases are covered. There are no orphan tests, and every FR
has at least one test.

## Deviations and open items

- **Generated parent edited.** `pub mod findstr;` was added to
  `crates/runmat-runtime/src/builtins/strings/search/mod.rs`, which is marked
  "@generated … Do not edit" (same precedent as `strmatch`). It must be
  listed in the maintainer README, and it will be dropped if the module
  composition is regenerated.
- **Error identifiers** use the `RunMat:findstr:` namespace with the observed
  suffixes, plus `NotEnoughInputs` for the observed `MATLAB:minrhs`.
  Flagged for maintainer review.
- **Not verified end-to-end through the VM.** The tests call the registered
  builtin through the runtime dispatcher.
- The WASM builtin registry was not regenerated.
