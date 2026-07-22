# Validation Report — 011-system-invocation

Date: 2026-07-22. Host: Linux x86_64 (Unix; observed-case tests run
natively). Implementation approved at the batch gate with two recorded
human decisions: (1) Windows returns the clear unsupported error — no
`cmd /C` path; Unix uses `/bin/sh -c`; (2) statement calls (no requested
output) forward captured stdout to the console via
`crate::console::record_console_output`, tagged `unresolved_choice`.

## Files

| Item | Path |
|---|---|
| Builtin (new) | `crates/runmat-runtime/src/builtins/io/repl_fs/system.rs` |
| Registration (1 line) | `crates/runmat-runtime/src/builtins/io/repl_fs/mod.rs` (`pub mod system;`) |

No other source file was modified by this feature.

## Verification commands (actual runs)

`cargo check -p runmat-runtime`:

```
Checking runmat-runtime v0.5.6 (…/crates/runmat-runtime)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 15.62s
```

`RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::io::repl_fs::system`:

```
running 7 tests
test builtins::io::repl_fs::system::tests::invalid_input_errors ... ok
test builtins::io::repl_fs::system::tests::observed_captured_output ... ok
test builtins::io::repl_fs::system::tests::observed_status_failure ... ok
test builtins::io::repl_fs::system::tests::observed_status_success ... ok
test builtins::io::repl_fs::system::tests::system_builtin_is_registered ... ok
test builtins::io::repl_fs::system::tests::unresolved_choice_empty_stdout_two_outputs ... ok
test builtins::io::repl_fs::system::tests::unresolved_choice_statement_call_echoes_stdout ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 6648 filtered out; finished in 0.01s
```

`cargo fmt -p runmat-runtime` applied, then `cargo fmt --all -- --check`
exited 0 (clean).

An eighth test, `unsupported_platform_errors`, is compile-gated
`#[cfg(not(unix))]` and does not run on this host; it asserts the
Windows/wasm unsupported error per the recorded gate decision.

## Traceability (claim → FR → test → result)

| claim_id (category) | FR | test | result |
|---|---|---|---|
| system.signature-primary (public-interface-fact) | FR-011-01 | `observed_status_success` | PASS |
| system.status-and-output (black-box-observation; case `status-success`) | FR-011-02 | `observed_status_success` | PASS |
| system.status-and-output (case `status-failure`) | FR-011-03 | `observed_status_failure` | PASS |
| system.status-and-output (case `captured-output`) | FR-011-04 | `observed_captured_output` | PASS |
| — (documented choice; empty stdout) | FR-011-05 | `unresolved_choice_empty_stdout_two_outputs` | PASS |
| — (documented choice; gate decision 2, statement-call echo) | FR-011-05 | `unresolved_choice_statement_call_echoes_stdout` | PASS |
| — (documented choice; invalid input) | FR-011-05 | `invalid_input_errors` | PASS |
| — (documented choice; gate decision 1, non-Unix unsupported) | FR-011-05 | `unsupported_platform_errors` | compile-gated (not runnable on this host) |
| — (registration) | SC-011-2 | `system_builtin_is_registered` | PASS |

Success criteria: SC-011-1 (three observed cases reproduce exactly —
value, class, size) — MET on Unix. SC-011-2 (registered, discoverable via
`builtin_function_by_name`; test module passes) — MET. SC-011-3 (each FR
cites ≥1 claim id; each normative test cites ≥1 FR in its comment) — MET.

## Statement-call semantics note (T1 finding)

`crate::output_count::current_output_count()` returns `Some(0)` for
statement calls dispatched through
`call_builtin_async_with_outputs` (the JIT passes the real requested
output count — `crates/runmat-turbine/src/lib.rs` expanded-outputs call
site); the echo triggers exactly on `Some(0)`. The legacy
`call_builtin_async` path pushes `None` and returns `status` only, with no
echo (documented; matches the `fileparts.rs` single-output convention).

## Sandbox compliance

Tests spawned only `/bin/sh -c` with `true`, `false`, `echo hello` (fixed
literals; no metacharacters, no environment mutation, no filesystem
writes, no network). Process-spawning tests are `#[cfg(unix)]`-gated.
`stdin` is closed for the child; stderr passes through (no test asserts
stderr). MATLAB was not invoked at any point.

## Deviations from plan.md

1. Windows `cmd /C` path (planned as a documented choice) was REMOVED per
   recorded gate decision 1: all non-Unix targets (including Windows and
   wasm) return `RunMat:system:PlatformUnsupported`. The single
   `#[cfg(unix)]` / `#[cfg(not(unix))]` split implements this.
2. Echo choice finalised per recorded gate decision 2 (forward stdout on
   statement calls); implemented and tested as planned Option A, with the
   trigger narrowed to `current_output_count() == Some(0)` after the T1
   inspection.
3. Added `RunMat:system:SpawnFailed` (`RM.SYSTEM.SPAWN_FAILED`) for a
   failed `/bin/sh` spawn — error-path completeness within FR-011-05's
   documented-choice scope (untestable without breaking sandbox rules; not
   exercised by tests).
4. Added the `system_builtin_is_registered` test to evidence SC-011-2
   directly.

Unresolved questions carried (unchanged): `system.q-side-effect`,
`system.q-outputs`. Per constitution Principle X, independent legal review
may still be required; no external implementation sources were used
(standard library only).
