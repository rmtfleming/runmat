# Validation Report — 029-time-conversion — 2026-07-22

## Commands and results

- `cargo check -p runmat-runtime` — clean (finished in 17.5s)
- `cargo fmt -p runmat-runtime` then `cargo fmt --all -- --check` — clean
- `cargo clippy -p runmat-runtime --lib` — **0 warnings** (0 in the two new
  files; a transient sibling error in `introspection::version` cleared on
  re-run — no edits to sibling files)
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::datetime::etime builtins::duration`
  — **14 passed, 0 failed** (10 new: 6 `etime` + 4 `seconds`; plus 4 existing
  `duration` constructor tests still green, confirming no regression)

### `etime` / `seconds` test output

```
test builtins::datetime::etime::tests::descriptor_advertises_primary_signature ... ok
test builtins::datetime::etime::tests::observed_one_second_elapsed ... ok
test builtins::datetime::etime::tests::unresolved_choice_cross_day_span ... ok
test builtins::datetime::etime::tests::unresolved_choice_cross_year_span ... ok
test builtins::datetime::etime::tests::unresolved_choice_invalid_inputs_error ... ok
test builtins::datetime::etime::tests::unresolved_choice_negative_when_t2_before_t1 ... ok
test builtins::duration::seconds::tests::descriptor_advertises_primary_signature ... ok
test builtins::duration::seconds::tests::observed_scalar_number_becomes_duration ... ok
test builtins::duration::seconds::tests::unresolved_choice_array_input_maps_elementwise ... ok
test builtins::duration::seconds::tests::unresolved_choice_invalid_inputs_error ... ok
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 6962 filtered out
```

## Actual observed outputs reproduced

| Call | Observed (export) | RunMat result | Match |
|---|---|---|---|
| `etime([2020 1 1 0 0 1], [2020 1 1 0 0 0])` | `1`, double, 1×1 | `Value::Num(1.0)` | exact |
| `seconds(5)` | `<duration [1 1]>`, class duration, 1×1 | duration object, `__days` 1×1, magnitude 5 s (display `"5"`) | class + size exact; magnitude per semantics |

## Traceability (claim → FR → test → result)

| Claim | FR | Test | Result |
|---|---|---|---|
| etime.signature-primary, etime.output-class, etime.elapsed-seconds | FR-029-01 | `etime::tests::observed_one_second_elapsed` | PASS |
| seconds.signature-primary, seconds.output-class, seconds.to-duration | FR-029-02, FR-029-03 | `seconds::tests::observed_scalar_number_becomes_duration` | PASS |
| etime.q-negative (unresolved) | FR-029-04 | `etime::tests::unresolved_choice_negative_when_t2_before_t1`, `unresolved_choice_cross_day_span`, `unresolved_choice_cross_year_span` | PASS (non-normative) |
| etime input validation (unresolved) | FR-029-05 | `etime::tests::unresolved_choice_invalid_inputs_error` | PASS (non-normative) |
| seconds.q-from-duration / array / format (unresolved) | FR-029-06 | `seconds::tests::unresolved_choice_array_input_maps_elementwise`, `unresolved_choice_invalid_inputs_error` | PASS (non-normative) |
| descriptor advertises primary signature | SC-029-2 | `etime::tests::descriptor_advertises_primary_signature`, `seconds::tests::descriptor_advertises_primary_signature` | PASS |

Both observed cases across the two exports reproduce exactly for the
normative attributes (value/class/size for `etime`; class/size for `seconds`,
with magnitude verified by the documented semantics).

## Deviations and notes

- `etime` epoch: the civil-to-days conversion is anchored to 1970-01-01, but
  because `etime` returns `seconds(t2) − seconds(t1)` the anchor cancels; the
  observed one-second case reproduces independently of the epoch choice.
- `seconds` display `Format` is `"s"` (render in seconds) — an unobserved
  documented independent choice; the observed export value is opaque.
- `seconds` reuses the duration module's `duration_object_from_days_tensor`
  (days-based `__days` representation), identical to the `duration`
  constructor, so downstream duration arithmetic/formatting apply unchanged.
- Registration is via the compile-time `#[runtime_builtin]` inventory macro,
  the same mechanism as all existing builtins; the only edits to existing
  files are the two `pub mod` lines in `datetime/mod.rs` and
  `duration/mod.rs`.

## Unresolved (returned to specification side)

`etime.q-deprecated`, `etime.q-negative`, `seconds.q-from-duration`.
