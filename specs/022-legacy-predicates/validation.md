# Validation Report — 022-legacy-predicates — 2026-07-22

## Commands and results

- `cargo check -p runmat-runtime` — clean (finished in ~17s).
- `cargo fmt` then `cargo fmt --all -- --check` — clean (exit 0).
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::introspection::isobject builtins::introspection::isstr builtins::io::repl_fs::isfolder builtins::io::repl_fs::isdir`
  — **30 passed, 0 failed** (22 in the four new modules; the remaining 8 are
  `builtins::introspection::isstring`, matched incidentally because `isstr` is
  a prefix of `isstring`).

Per-module test counts (default features; two `#[cfg(feature = "wgpu")]` tests
in `isobject`/`isstr` are additionally present but not compiled in this run):

| Module | Tests run |
|---|---|
| `builtins::introspection::isobject` | 7 |
| `builtins::introspection::isstr` | 7 |
| `builtins::io::repl_fs::isfolder` | 4 |
| `builtins::io::repl_fs::isdir` | 4 |

## Actual observed-case outputs (reproduced exactly)

| Builtin | Observed call | Expected | Test result |
|---|---|---|---|
| isobject | `isobject(inputParser)` | `true` (1×1 logical) | PASS |
| isobject | `isobject(5)` | `false` | PASS |
| isobject | `isobject('a')` | `false` | PASS |
| isobject | `isobject({1})` | `false` | PASS |
| isstr | `isstr('abc')` | `true` | PASS |
| isstr | `isstr(5)` | `false` | PASS |
| isstr | `isstr("hi")` | `false` | PASS |
| isstr | `isstr({'a'})` | `false` | PASS |
| isfolder | `isfolder(tempdir)` | `true` | PASS |
| isfolder | `isfolder([tempname '.nope'])` | `false` | PASS |
| isdir | `isdir(tempdir)` | `true` | PASS |
| isdir | `isdir([tempname '.nope'])` | `false` | PASS |

All 12 observed cases across the four exports reproduce exactly (value; class
logical via `Value::Bool` ≡ 1×1 logical, per RunMat's value model).

## Traceability (claim → FR → test → result)

| Claim(s) | FR | Test(s) | Result |
|---|---|---|---|
| isobject.signature-primary, output-class, logical-object-test | FR-022-01 | `isobject::tests::observed_object_instance_is_true`, `observed_numeric_is_false`, `observed_char_is_false`, `observed_cell_is_false` | PASS |
| isstr.signature-primary, output-class, logical-char-test | FR-022-02 | `isstr::tests::observed_char_is_true`, `observed_numeric_is_false`, `observed_string_is_false`, `observed_cell_is_false` | PASS |
| isfolder.signature-primary, output-class, logical-folder-test | FR-022-03 | `isfolder::tests::observed_existing_folder_is_true`, `observed_missing_path_is_false` | PASS |
| isdir.signature-primary, output-class, logical-folder-test | FR-022-04 | `isdir::tests::observed_existing_folder_is_true`, `observed_missing_path_is_false` | PASS |
| unresolved choices (isobject.q-fundamental, isstr.q-deprecated, isfolder.q-file, isdir.q-deprecated) | FR-022-05 | `isobject::tests::unresolved_choice_value_object_is_true`, `unresolved_choice_fundamental_kinds_are_false`, `unresolved_choice_gpu_tensor_is_false`; `isstr::tests::unresolved_choice_char_matrix_and_empty_are_true`, `unresolved_choice_logical_is_false`, `unresolved_choice_gpu_tensor_is_false`; `isfolder::tests::unresolved_choice_file_is_false`, `unresolved_choice_non_text_input_is_false`; `isdir::tests::unresolved_choice_file_is_false`, `unresolved_choice_non_text_input_is_false` | PASS (non-normative) |

Every FR cites ≥1 claim id; every normative test cites ≥1 FR/claim in its
name/comment. No orphan tests; no requirement without a covering test.

## Deviations and notes

- `isstr` is a byte-for-byte semantic mirror of `ischar`
  (`matches!(v, Value::CharArray(_))`); a double-quoted string scalar reports
  `false`, matching the observed `isstr("hi") => false` case.
- `isdir` is implemented as a legacy alias that delegates to a shared
  `pub(crate) folder_predicate` in `isfolder.rs`, rather than via
  `call_builtin_async("isfolder", …)`. This keeps a single, directly-tested
  source of truth and avoids a name-dispatch round-trip; both builtins are
  separately registered and separately tested.
- `isobject` reports `true` for `Value::Object` and `Value::HandleObject`.
  Other object-adjacent kinds (`Value::Listener`, `Value::MException`,
  `Value::ClassRef`) are unobserved in the export and conservatively report
  `false` (documented independent choice; not asserted as MATLAB-conformant).
- `isfolder`/`isdir` treat a file as `false` (folder ≠ file) — a documented
  choice pending `isfolder.q-file`.
- Registration is via the compile-time `#[runtime_builtin]` inventory macro
  (same mechanism as all existing builtins); no central dispatch table edits.
- MATLAB was NOT invoked at any point in this repository.

## Unresolved (returned to specification side)

`isobject.q-fundamental`, `isstr.q-deprecated`, `isfolder.q-file`,
`isdir.q-deprecated`.
