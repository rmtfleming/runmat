# Validation Report — 060-exception-causes (addCause 0.2.0) — 2026-10-30/31

Environment: Linux x86_64; `HDF5_DIR=~/anaconda3`; test binaries run
through the cargo runner wrapper. The target directory was shared with other
agents (several build attempts waited out their in-progress stub files).
MATLAB was not invoked.

## Freshness check (T0, finish)

In `../matlab-interface-spec`, after `git fetch -q origin` (rc 0):

- Start: HEAD = origin/main = `c3305a5`; `git log fc9f804..origin/main` over
  `specifications/{addCause,MException,throw}` empty; no draft or
  experiment for these functions in `git status --short`.
- Finish (2026-10-31 00:13 UTC): HEAD = origin/main = `1c91aa1`;
  `git log fc9f804..HEAD` and `..origin/main` over the three directories:
  0 commits; `git status --short` over them and `experiments/addCause`:
  clean.

## Test-first evidence (T4)

Tests were written with `add_cause` stubbed to return
`RunMat:addCause:Internal` "not yet implemented" (the core type field and
codec mapping already compiled). Run
`cargo test -p runmat-runtime --lib -- exceptions_warnings_tests`:
**22 passed, 10 failed, 0 ignored**. The 10 failures were every F60 test;
the 22 passes were the unchanged F49 tests.

First run after implementing: **43 passed, 2 failed** (filter
`exceptions_warnings_tests value_codec runtime_error`). The two failures
(`observed_add_cause_survives_throw_and_catch`,
`unresolved_choice_rethrow_keeps_causes`) showed the causes present on the
thrown error's `source` but not recovered by the downcast: the
`ExceptionCauses` type was defined in `runmat-runtime`, and in the lib-test
binary the dispatched builtin used a second copy of that crate, so the type
ids differed. Fix: the marker type moved to `runmat-value`
(`runmat_value::ExceptionCauses`, re-exported from `runtime_error`).

The first VM run had **1 failure** (`method_syntax_and_value_semantics`:
"Reference to non-existent field 'addCause'"): compiled `e.addCause(c)` is
a dotted-invoke subscript path, not a method call. Fix: a guarded
MException branch in `object/protocol/path_execution/dotted_invoke.rs`.

## Core changes (all files outside the F49 builtin file)

| file | change | reason |
|---|---|---|
| `crates/runmat-value/src/exception.rs` | `MException.causes: Vec<MException>` (empty from `MException::new`); `MException::with_cause`; new `ExceptionCauses(Vec<MException>)` with `Display` + `Error` | cause list on the value type; one type identity for the downcast |
| `crates/runmat-value/src/lib.rs` | `pub use exception::{ExceptionCauses, MException};` | export |
| `crates/runmat-runtime/src/runtime_error.rs` | `exception_from_error` restores causes from `error.source` (downcast to `ExceptionCauses`); new `error_from_exception(&MException)` (identifier + message + `with_source(ExceptionCauses)` when there are causes); `pub use runmat_value::ExceptionCauses` | causes survive throw → catch (FR-060-05) |
| `crates/runmat-runtime/src/lib.rs` | `rethrow_builtin` uses `error_from_exception` | U-060-2 |
| `crates/runmat-runtime/src/execution/value_codec/decode.rs` | causes decoded recursively (the "nested exception causes are not represented" rejection removed) | FR-060-09 |
| `crates/runmat-runtime/src/execution/value_codec/encode.rs` | causes encoded recursively (`exception_payload`) | FR-060-09 |
| `crates/runmat-runtime/src/execution/value_codec/mod.rs` | existing round-trip test's exception literal gains nested causes | FR-060-09 |
| `crates/runmat-runtime/src/object/resolve.rs` | MException `cause` member → n×1 cell of the causes | FR-060-02 |
| `crates/runmat-runtime/src/object/dispatch.rs` | MException method arm `addCause` with one argument | FR-060-07 |
| `crates/runmat-runtime/src/object/protocol/path_execution/dotted_invoke.rs` | `ME.addCause(...)` / `ME.throw(...)` on a `Value::MException` routed to the dispatch arms | FR-060-07 in compiled code; also exercises `e.throw()` (D-049-2 update) |
| `crates/runmat-vm/tests/exception_causes.rs` (new) | 4 interpreter tests | FR-060-04..07, U-060-2 |

The wire format (`runmat_execution::value::ExceptionValue.causes`) was not
changed. `runmat-async::RuntimeError` was not changed (its `source` slot is
used). `getfield(ME,'cause')` (builtin special-object path) was not changed.
No generated parent `mod.rs` was edited.

## Commands and results (T6)

- `cargo test -p runmat-runtime --lib -- exceptions_warnings_tests`:
  **32 passed, 0 failed, 0 ignored** (22 F49 + 10 F60; the F49 ignored test
  is gone).
- `RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib --
  exceptions_warnings_tests builtins::diagnostics object::
  execution::value_codec runtime_error warnings::` (after the final change):
  **193 passed, 0 failed, 0 ignored**.
- `cargo test -p runmat-value`: **92 passed, 0 failed**.
- `cargo test -p runmat-vm --test exceptions --test exception_causes`:
  **6 + 4 passed, 0 failed** (upstream `exceptions.rs` unchanged and
  passing).
- `cargo clippy -p runmat-value --lib --tests`: no findings.
  `cargo clippy -p runmat-runtime --lib --tests`: no findings (Finished).
  `cargo clippy -p runmat-vm --tests`: no findings (Finished).
- `rustfmt --edition 2021 --check` on every changed file: clean
  (`lib.rs` was checked as a file; its diffs belong to other agents'
  child modules and were not touched).

## Traceability (claim → FR → test → result)

| Claim / basis | FR | Test(s) | Result |
|---|---|---|---|
| addCause.signature-primary, addCause.append, MException.addcause | FR-060-01, FR-060-02 | `observed_add_cause_appends_in_order` | PASS |
| addCause.nested | FR-060-03 | `observed_add_cause_nested_causes_kept` | PASS |
| addCause.value-semantics, MException.addcause | FR-060-04 | `observed_add_cause_value_semantics`; VM `method_syntax_and_value_semantics` | PASS |
| addCause.thrown, throw.cause | FR-060-05 | `observed_add_cause_survives_throw_and_catch` (throw, `ME.throw`); VM `thrown_causes_reach_catch`, `method_throw_carries_causes` | PASS |
| addCause.caught-cause | FR-060-06 | `observed_add_cause_caught_error_as_cause`; VM `caught_error_as_cause_then_rethrown` | PASS |
| addCause.method-syntax | FR-060-07 | `observed_add_cause_method_syntax`; VM `method_syntax_and_value_semantics` | PASS |
| addCause.error-char-cause | FR-060-08 | `rejects_add_cause_non_exception_cause` | PASS |
| local (codec) | FR-060-09 | `causes_round_trip_through_value_codec`; `execution::value_codec::tests::stable_immutable_runtime_forms_round_trip_exactly` (struct with nested-cause exception) | PASS |
| addCause.non-exception (non-normative), U-060-1 | FR-060-10 | `unresolved_choice_add_cause_inputs` | PASS |
| U-060-2 (rethrow) | FR-060-10 | `unresolved_choice_rethrow_keeps_causes`; VM `caught_error_as_cause_then_rethrown` | PASS |
| U-060-3 (uncaught display) | FR-060-10 | not asserted | documented |
| addCause.q-warning-coverage | FR-060-11 | not asserted | not tested |
| MException.properties (0×1 without causes) | FR-060-02 | `observed_add_cause_value_semantics` | PASS |

Every FR maps to at least one test.

## Error identifiers

`RunMat:class:RequireClass` (observed `MATLAB:class:RequireClass`),
`RunMat:addCause:UndefinedFunction` (U-060-1),
`RunMat:addCause:InvalidInput`, `RunMat:addCause:TooManyOutputs`.
`RunMat:addCause:UnsupportedOption` is no longer raised.

## Deviations

None for F60. D-049-1 is closed (see
`specs/049-exceptions-warnings/validation.md`); D-049-2 updated (the
`e.throw()` form now runs through the VM; the bare `ME.throw;` statement is
still not exercised).

## Notes

- No hand-added lines in generated parents.
- Integer-capability metadata was not declared.
- No MathWorks material was consulted; MATLAB was not invoked.
