# Validation Report — 008-date-formatting — 2026-07-22

## Commands and results

- `cargo check -p runmat-runtime` — clean (`Finished dev profile ... in 1m 31s`)
- `cargo fmt --all -- --check` — clean (exit 0, no diff)
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::datetime::datestr`
  — **10 passed, 0 failed** (`test result: ok. 10 passed; 0 failed; 0 ignored`)

All three commands were re-run on the final (formatted) tree; the test run
listed all ten `builtins::datetime::datestr::tests::*` tests individually as
`ok`.

## Traceability (claim → FR → test → result)

| Claim | FR | Test | Result |
|---|---|---|---|
| datestr.signature-primary, datestr.output-class, datestr.char-format | FR-008-01, FR-008-02 | `datestr::tests::observed_serial_default_format` (738885 → '30-Dec-2022', char 1×11) | PASS |
| datestr.output-class, datestr.char-format | FR-008-03, FR-008-04 | `datestr::tests::observed_serial_explicit_format` (738885, 'yyyy-mm-dd' → '2022-12-30', char 1×10) | PASS |
| datestr.signature-primary | FR-008-01 | `datestr::tests::observed_serial_from_scalar_tensor` | PASS |
| unresolved choices (epoch, time tokens, rounding, string format, errors) | FR-008-05 | `unresolved_choice_epoch_anchoring`, `unresolved_choice_time_of_day_tokens`, `unresolved_choice_second_rounding_carries`, `unresolved_choice_string_format_argument`, `unresolved_choice_unknown_token_errors`, `unresolved_choice_invalid_date_inputs_error`, `unresolved_choice_argument_count_errors` | PASS (non-normative) |

Both observed cases in the approved export reproduce exactly (value, class
char via `Value::CharArray` row vector, sizes 1×11 and 1×10).

## Deviations and notes

- Date conversion is an independent Rust implementation of the
  public-domain civil-from-days algorithm; the serial epoch constant
  (719529 = 1970-01-01) is derived solely from the observed case
  `serial-default` (738885 → 30-Dec-2022). Recorded in `provenance.md`
  (Principle X).
- Supported format tokens: `yyyy`, `mm`, `dd`, `mmm` (exercised by the
  observed cases / default format) plus documented-choice `HH`, `MM`, `SS`.
  Unknown alphabetic tokens error (`RunMat:datestr:InvalidFormat`) rather
  than being guessed.
- Date text, date vectors, and non-scalar inputs (listed `pending` in the
  export, no observations) are rejected with `RunMat:datestr:InvalidInput`
  — documented independent choice, not asserted MATLAB-conformant.
- Registration is via the `#[runtime_builtin]` inventory macro;
  `datestr.rs` is the first submodule of the `datetime` category
  (one-line `pub mod datestr;` addition to `builtins/datetime/mod.rs`).
- Environment note: parallel batch agents (features 005–009) share this
  working tree; one intermediate test compile failed on another feature's
  in-progress file (`strings/search/strmatch.rs`). The final verification
  runs above completed cleanly after the tree settled.

## Unresolved (returned to specification side)

`datestr.q-deprecated` (legacy/not-recommended notice: RunMat emits none),
`datestr.q-locale` (default-format locale dependence: English Jan..Dec
chosen), plus new observation requests: format-token grammar beyond
yyyy/mm/dd/mmm, fractional serial numbers / time of day, non-scalar and
text/date-vector inputs.
