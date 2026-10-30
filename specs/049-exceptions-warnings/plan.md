# Implementation Plan — 049-exceptions-warnings (specs 0.2.0)

## Constitution Check

- I Specification-first: FR-049-01..12 cite the 23 approved normative
  claims (two not met, D-049-1); U-049-1..6 are labelled documented choices;
  FR-049-14 carries the unresolved questions. PASS.
- II Clean-room: inputs are the imported exports and RunMat's own exception
  and warning machinery; MATLAB not invoked. PASS.
- III Independence: values and errors are RunMat's own types. PASS.
- IV Provenance: pinned to `8f72edb` (MException, lastwarn) and `1156c97`
  (throw), freshness-checked. PASS.
- V Test-first: the four builtins are registered as stubs, the tests run and
  fail, and the failing run is recorded before the implementation. PASS.
- VII Minimal scope: one new file; one hand-added `pub mod` line in the
  generated `diagnostics/mod.rs`; small additive edits in
  `warnings/policy.rs` (a `lastwarn` state with getter/setter),
  `warnings/emission.rs` (record every emitted warning into that state),
  `object/resolve.rs` and `object/dispatch.rs` (`ME.cause`, `ME.throw`).
  The cause architecture gap is documented (D-049-1), not refactored. PASS.
- IX Compatibility: `#[runtime_builtin]` with descriptors in
  `builtins/diagnostics/`; `Value::MException`; `crate::warnings`. No
  `integer_capabilities`. PASS.

## Affected files

| Item | Path |
|---|---|
| Builtins + tests (new) | `crates/runmat-runtime/src/builtins/diagnostics/exceptions_warnings.rs` |
| Registration (generated parent, one hand-added line) | `crates/runmat-runtime/src/builtins/diagnostics/mod.rs`: `pub mod exceptions_warnings;` |
| lastwarn state | `crates/runmat-runtime/src/warnings/policy.rs` (field `lastwarn`, `lastwarn()`, `set_lastwarn()`) |
| lastwarn recording | `crates/runmat-runtime/src/warnings/emission.rs` (one line in `emit`) |
| `ME.cause`, `ME.throw` | `crates/runmat-runtime/src/object/resolve.rs` (MException member arm) |
| `ME.throw()` | `crates/runmat-runtime/src/object/dispatch.rs` (MException method-call arm) |

## Design

- `MException(id, fmt, args…)`: output-count guard; fewer than two inputs →
  `RunMat:minrhs`; text inputs; identifier `''` or ≥2 non-empty components
  else `badIdentifier`; message via `format_variadic`; returns
  `Value::MException(MException::new(id, msg))`.
- `throw(ME)`: returns `Err(RuntimeError{identifier: ME.identifier (also
  ''), message: ME.message})` (`throw_error`, shared with the `ME.throw`
  routes). Non-MException → `RunMat:throw:UndefinedFunction`.
- `addCause`: explicit `RunMat:addCause:UnsupportedOption` (D-049-1).
- `lastwarn`: reads `WarningPolicy::lastwarn`; setting forms store
  `Some(RuntimeWarning{message, identifier})` or `None` for `''`/`''`;
  outputs (previous values) by requested output count; `RunMat:warning` is
  reported as `''`; empty values are 0×0 char. `emission::emit` records
  every warning (also suppressed ones) into `lastwarn`.

## Test plan (`exceptions_warnings_tests`)

- `observed_*` for every observed case of the three exports (catch emulated
  with `runtime_error::exception_from_error`); `rejects_*` for
  badIdentifier / minrhs; `unresolved_choice_*` for U-049-1..4;
  `deviation_*` for D-049-1 (one active test pinning the explicit error,
  one `#[ignore]`d with the observed behaviour); registration.

## Risks & rollback

- Upstream edits are additive (new match arms, new field); upstream
  `warning` behaviour is unchanged (U-049-6).
- Rollback: delete the new file and its `mod` line; revert the four small
  upstream edits.

## Verification commands

```
cargo test -p runmat-runtime --lib -- exceptions_warnings_tests
cargo test -p runmat-runtime --lib -- builtins::diagnostics warnings:: object::
rustfmt --edition 2021 --check <changed files>
cargo clippy -p runmat-runtime --lib --tests --message-format short
```

## Analysis (B3)

- Every normative claim id of the three imported `provenance.yaml` files is
  cited by an FR (23/23); the non-normative throw.non-mexception backs
  U-049-2 only.
- FR-049-06 is explicitly *not met* and paired with D-049-1 and an ignored
  test; this is surfaced to the reviewer rather than silently resolved.
- Every FR has a planned test; every task cites an FR; no orphans.
- Identifier mapping consistent across spec, provenance and tests.
- Risk flagged for the reviewer: D-049-1 needs an architecture decision
  (cause list on `runmat_value::MException` and through `RuntimeError`).
