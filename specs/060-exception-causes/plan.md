# Implementation Plan — 060-exception-causes

## Constitution Check

- I Specification-first: FR-060-01..08 cite the approved `addCause` claims
  and `MException.addcause` / `throw.cause`. FR-060-09..11 are tagged as
  local requirements, documented choices or unresolved. PASS.
- II Clean-room: inputs are the imported exports and RunMat code only.
  MATLAB was not invoked. PASS.
- III Independence: causes are a plain `Vec` on the exception value; the
  thrown error carries them as its `source`. PASS.
- IV Provenance: pinned to `fc9f804` (addCause), `8f72edb` (MException),
  `1156c97` (throw); freshness-checked at start and finish. PASS.
- V Test-first: tests written first, run against a stubbed `addCause`, the
  failing run recorded in `validation.md`. PASS.
- VII Minimal scope: the maintainer approved a core change (cause list on
  the value type, codec, error propagation). It is kept to: one field and
  one helper and a marker error type (`ExceptionCauses`) in
  `runmat_value::exception`; two helpers in
  `runmat-runtime/src/runtime_error.rs`; recursive mapping in the
  value codec (wire format unchanged — `ExceptionValue.causes` already
  existed); `rethrow` routed through the shared helper; the F49
  `Value::MException` arms of `object/resolve.rs` and `object/dispatch.rs`;
  one guarded branch in `object/protocol/path_execution/dotted_invoke.rs`
  (the interpreter lowers `e.addCause(c)` as a dotted-invoke subscript
  path, not a method call).
  `runmat-async::RuntimeError` is not changed (it cannot depend on
  `runmat-value`; its existing `source` slot is used). PASS.
- IX Compatibility: `MException::new` keeps its signature; only three
  struct literals in the workspace needed the new field (value codec decode
  and its test, all in `runmat-runtime`). No integer-capability metadata.
  PASS.

## Placement

| Item | File |
|---|---|
| `causes` field, `with_cause`, `ExceptionCauses` error marker | `crates/runmat-value/src/exception.rs` (+ re-export in `lib.rs`) |
| re-export of `ExceptionCauses`, `error_from_exception`, causes in `exception_from_error` | `crates/runmat-runtime/src/runtime_error.rs` |
| codec encode / decode | `crates/runmat-runtime/src/execution/value_codec/{encode,decode}.rs` (+ test in `mod.rs`) |
| `rethrow` keeps causes | `crates/runmat-runtime/src/lib.rs` (`rethrow_builtin`) |
| `addCause` builtin, `throw_error` | `crates/runmat-runtime/src/builtins/diagnostics/exceptions_warnings.rs` |
| `ME.cause` | `crates/runmat-runtime/src/object/resolve.rs` (MException member arm) |
| `ME.addCause(c)` | `crates/runmat-runtime/src/object/dispatch.rs` (MException method arm) |
| `ME.addCause(c)` / `ME.throw()` in compiled code | `crates/runmat-runtime/src/object/protocol/path_execution/dotted_invoke.rs` (MException short-cut to the dispatch arm) |
| VM try/catch coverage | `crates/runmat-vm/tests/exception_causes.rs` (new) |

## Design

1. `addCause(ME, cause)`: at most one output; exactly two inputs
   (`InvalidInput`); first not an `MException` → `UndefinedFunction`
   (U-060-1); cause not an `MException` → `RunMat:class:RequireClass`;
   otherwise `ME.clone().with_cause(cause.clone())`.
2. `throw` / `ME.throw` / `rethrow` build the error with
   `error_from_exception`: identifier + message, and when there are causes
   `with_source(ExceptionCauses(causes))`. Executors decorate errors in
   place (span, call frames), so the source reaches `catch`.
3. `exception_from_error` downcasts `error.source` to `ExceptionCauses`
   and sets `causes`.
4. `ME.cause` → n×1 cell of `Value::MException`.

## Test strategy

Module `exceptions_warnings_tests`: `observed_add_cause_*` for each
observed case, `rejects_add_cause_non_exception_cause`,
`unresolved_choice_add_cause_inputs`, `unresolved_choice_rethrow_keeps_causes`,
`causes_round_trip_through_value_codec`. The F49 deviation tests
(`deviation_add_cause_is_unsupported`, ignored
`deviation_observed_add_cause_and_thrown_cause`) are replaced. VM-level:
`crates/runmat-vm/tests/exception_causes.rs` runs MATLAB source through the
interpreter (`throw` → `catch` with causes, `e.addCause(c)`).

## Risks

- Uncaught errors with causes now have a `source`, so report handlers that
  walk the source chain print "caused by: <ids>" (U-060-3).
