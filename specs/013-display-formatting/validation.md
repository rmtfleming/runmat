# Validation Report — 013-display-formatting

**Date**: 2026-07-22. Implementation per Gate-approved plan.md/tasks.md, with
the recorded human decision applied: `display(X, name)` is REJECTED with a
clear error (no invented header formatting).

## Files changed

| File | Change |
|---|---|
| `crates/runmat-runtime/src/builtins/io/display.rs` | NEW — builtin, pure formatter, 12 inline tests |
| `crates/runmat-runtime/src/builtins/io/mod.rs` | +1 line: `pub mod display;` |

`disp.rs` and all shared formatting are untouched (verified by `git diff` and
by the disp regression run below).

## Verification commands (actual runs, this repository)

1. `cargo check -p runmat-runtime` — `Finished 'dev' profile ... in 13.92s`
   (clean, no warnings for this module).
2. `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::io::display`
   — `test result: ok. 12 passed; 0 failed; 0 ignored; ... 6622 filtered out`
3. No-disp-regression:
   `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::io::disp::`
   — `test result: ok. 13 passed; 0 failed; 0 ignored; ... 6642 filtered out`
4. `cargo fmt -p runmat-runtime` then `cargo fmt --all -- --check` — clean
   (exit 0, no diff output).

## Traceability (claim → FR → test → result)

All tests live in `crates/runmat-runtime/src/builtins/io/display.rs`
(`builtins::io::display::tests`). Console-capture tests drain the
thread-local buffer (`console::reset_thread_buffer` /
`take_thread_buffer`) — in-process, no MATLAB, no evalc.

| Claim | FR | Test | Result |
|---|---|---|---|
| display.signature-primary | FR-013-01 | `display_descriptor_signatures_cover_primary_form`; all capture tests invoke `display(X)` | ok |
| display.formatted-text (`scalar-text`) | FR-013-03 | `display_scalar_text_matches_observed` (pure, byte-exact `"    42\n\n"`) | ok |
| display.formatted-text (`scalar-text`) | FR-013-02, FR-013-03 | `display_scalar_console_capture_matches_observed` | ok |
| display.formatted-text (`vector-text`) | FR-013-04 | `display_vector_text_matches_observed` (pure, byte-exact `"     1     2     3\n\n"`) | ok |
| display.formatted-text (`vector-text`) | FR-013-02, FR-013-04 | `display_vector_console_capture_matches_observed` | ok |
| display.formatted-text (`char-text`) | FR-013-05 | `display_char_text_matches_observed` (pure, byte-exact `"hi\n"`) | ok |
| display.formatted-text (`char-text`) | FR-013-02, FR-013-05 | `display_char_console_capture_matches_observed` | ok |
| (sink/gather convention, documented choice under `display.q-return`) | FR-013-02 | `display_accepts_gpu_tensor` (gather → observed-derived layout; 0x0 sink return) | ok |
| (documented choice — export form `pending`) | FR-013-06 | `display_rejects_second_argument_unresolved_choice` | ok |
| (documented choice — header absent from export) | FR-013-06 | `display_emits_no_name_header_unresolved_choice` | ok |
| (documented choice — unobserved kinds) | FR-013-06 | `display_struct_delegates_to_disp_with_trailing_blank_unresolved_choice` | ok |
| (documented choice — unobserved kinds) | FR-013-06 | `display_int_delegates_to_disp_unresolved_choice` | ok |

Success criteria: SC-013-1 (byte-exact, both layers) — met; SC-013-2
(registered via `runtime_builtin` macro, module test filter passes) — met;
SC-013-3 (every FR cited by ≥1 test, `unresolved_choice` tags in test names)
— met; SC-013-4 (disp module passes unmodified, disp.rs untouched) — met.

## Unresolved questions carried (unchanged, non-normative)

`display.q-side-effect`, `display.q-overload`, `display.q-return`; plus the
feature-level documented choices in spec.md (no name header; two-argument
form rejected; delegation rendering for unobserved kinds; default spacing
hard-coded).

## Deviations from plan

- Delegation mechanism: plan.md offered making `disp::format_for_disp`
  `pub(crate)` if delegation needed it. Because `disp.rs` was frozen at
  implementation time, delegation instead goes through the public
  dispatcher: `crate::call_builtin_async("disp", ...)` followed by one
  recorded `"\n"` (precedent: `builtins/acceleration/gpu/arrayfun.rs`).
  Observable choice is identical (disp rendering + trailing blank line);
  disp.rs is untouched. Consequence: `format_display_text` returns
  `Option<String>` (`None` = delegate) rather than always a `String`.
- Test count: 12 (plan's listed cases plus an int-delegation case and the
  descriptor check).

## Constitution notes

- No MATLAB was invoked; all expected strings come from the approved export
  observations; all test outcomes above are from actual `cargo test` runs.
- Findings recorded at planning time stand: RunMat's `disp` numeric layout
  (2-space column separator; no scalar indent) likely diverges from the
  observed `display` layout — a spec-side `disp` observation is recommended;
  out of scope for this feature and deliberately not patched.
- Publication (commit/push/PR) NOT performed; separate approval required.
