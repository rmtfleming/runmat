# Implementation Plan — 011-system-invocation

## Constitution Check

- I Specification-first: requirements cite approved claims
  (`system.signature-primary`, `system.status-and-output`) — PASS
- II Clean-room: sources = imported packaged export + RunMat code only; no
  MATLAB invocation anywhere in the workflow or tests — PASS
- III Independence: execution via Rust `std::process`; no imitation of
  MATLAB internals; shell choice is an independent documented choice — PASS
- IV Provenance: `provenance.md` complete; authoritative commit pin
  `b054f3ad…` via `dist/runmat-export-2026-07-22` (batch-2) — PASS
- V Test-first: observed-case tests authored before the builtin body
  (tasks T2–T3 before T4–T6) — PASS
- VI Traceability: FR ↔ test mapping recorded in `validation.md` (T8) — PASS
- VII Minimal scope: one builtin, one new file + one `mod` line; no
  refactoring — PASS
- VIII Human gates: this plan stops at Gate 2; implementation only after
  Gate 3 approval — PASS
- IX Compatibility: follows `io/repl_fs/` conventions (descriptor, macro,
  GPU/fusion specs, inline tests); no new architecture — PASS
- X Licensing: standard library only; no external implementation source —
  PASS

## Affected crates and files

| Item | Path |
|---|---|
| `system` builtin | `crates/runmat-runtime/src/builtins/io/repl_fs/system.rs` (new) |
| Registration | `pub mod system;` added to `crates/runmat-runtime/src/builtins/io/repl_fs/mod.rs` (alphabetical, between `setenv` and `tempdir`) |
| Docs metadata | descriptor + `#[runtime_builtin]` doc fields (feeds `docs/builtins` tooling) |

Exemplars: `io/repl_fs/exist.rs` (sibling file layout, error builder,
GPU/fusion spec boilerplate for a non-GPU io builtin);
`io/repl_fs/fileparts.rs` (multi-output pattern:
`BuiltinOutputMode::ByRequestedOutputCount` in the descriptor, then
`crate::output_count::current_output_count()` +
`crate::output_count::output_list_with_padding(...)` in the body).

## Design decisions

- **Signature/descriptor**: one signature
  `[status, cmdout] = system(command)`; input `command`
  (`BuiltinParamType::Any`, validated in the body); outputs `status`
  (required) and `cmdout` (optional). `output_mode:
  BuiltinOutputMode::ByRequestedOutputCount`, completion policy `Public`.
- **Input validation** [FR-011-05]: accept `Value::CharArray` (rows ≤ 1),
  `Value::String`, and 1-element `Value::StringArray` as command text (same
  acceptance set as `fileparts`); anything else errors
  `RunMat:system:InvalidInput` (`RM.SYSTEM.INVALID_INPUT`).
- **Execution (Unix, documented choice)**: spawn
  `std::process::Command::new("/bin/sh").arg("-c").arg(command)` with stdout
  piped (captured) and stderr inherited; block on `.output()`-style
  collection of the piped stdout. Absolute `/bin/sh` avoids PATH lookup
  (sandbox-conscious). Windows (`cfg(windows)`, untested documented
  choice): `cmd /C <command>`.
- **`status`** [FR-011-02/03]: `Value::Num(code as f64)` from the child's
  exit code; signal-terminated child (no code, Unix) maps to
  `128 + signal` (documented choice, `unresolved_choice`).
- **`cmdout`** [FR-011-04]: captured stdout bytes decoded
  `String::from_utf8_lossy` (documented choice, `system.q-outputs`) into a
  `Value::CharArray` row vector including the trailing newline; empty
  stdout → 0×0 char (documented choice, matching the empty-char convention
  in `fileparts.rs`).
- **Multi-output plumbing**: build `vec![status, cmdout]`; if
  `crate::output_count::current_output_count()` is `Some(n)`, return
  `output_list_with_padding(n, outputs)`; otherwise return `status` alone —
  exactly the `fileparts.rs` shape.
- **Echo with no requested output** (unresolved, Gate 2 decision): proposed
  documented choice — when fewer than two outputs are requested, forward
  captured stdout to RunMat's console via
  `crate::console::record_console_output(ConsoleStream::Stdout, …)` so
  statement-form `system('echo hello')` shows output. Fallback if the
  reviewer prefers minimal v1: no echo (captured stdout dropped when not
  requested). The semantics of `current_output_count()` for statement calls
  must be verified in T1 before committing to either.
- **WASM / unsupported platforms** (documented choice): on
  `target_arch = "wasm32"` (and any target without process spawning) the
  body returns the error `RunMat:system:PlatformUnsupported`
  (`RM.SYSTEM.PLATFORM_UNSUPPORTED`, message
  "system: executing operating-system commands is not supported on this
  platform"). The builtin registers on all targets so the error is
  discoverable; only the execution path is `cfg`-gated.
- **GPU/fusion specs**: same non-GPU boilerplate as `fileparts.rs`
  (`GpuOpKind::Custom("io")`, no provider hooks, not fusible).

## SANDBOX / TEST-ISOLATION

Binding for all tasks in this feature (carried from selection proposal F11
and IMPORT-MANIFEST batch-2 side-effect note):

- Tests spawn ONLY `true`, `false`, and `echo hello` — fixed string
  literals baked into the test module; nothing user- or
  environment-derived.
- NO shell metacharacters in any test command: no `;`, `|`, `&`, `>`, `<`,
  backticks, `$(...)`, globs, or quoting tricks.
- No test mutates the environment, writes files, or touches the network;
  the spawned commands are side-effect-free coreutils/shell builtins.
- Observed-case tests are gated `#[cfg(unix)]` — the export platform is
  GLNXA64 and `/bin/sh`, `true`, `false`, `echo` are POSIX-portable; no
  process-spawning test runs on non-Unix or wasm targets.
- The wasm/unsupported error path is tested only as a compile-gated unit
  test (`#[cfg(target_arch = "wasm32")]`) or by direct call of the gated
  error constructor; CI does not need to spawn anything for it.
- Error-path tests (invalid input) spawn no process at all.

## Test plan (inline `#[cfg(test)]`, per house convention)

Normative, `#[cfg(unix)]` (cite FR/claims in test names or comments):
- `observed_status_success`: `system('true')` → `Value::Num(0.0)`, scalar.
  [FR-011-01, FR-011-02; case `status-success`]
- `observed_status_failure`: `system('false')` → `Value::Num(1.0)`.
  [FR-011-03; case `status-failure`]
- `observed_captured_output`: two-output request → `cmdout` char 1×6
  `"hello\n"`, `status` 0. [FR-011-04; case `captured-output`] (drive the
  two-output path the way `fileparts.rs` tests do, via the output-count
  helpers if needed).

Unresolved-tracking (tagged `unresolved_choice`, `#[cfg(unix)]` where a
process is spawned):
- empty stdout (`true` with two outputs) → 0×0 char `cmdout`.
- invalid input (`Value::Num`) → error containing `system` /
  `RunMat:system:InvalidInput` (no process spawned; not `cfg`-gated).
- unsupported-platform error text (compile-gated wasm or direct
  constructor call).

## Risks & rollback

- Risk: `current_output_count()` semantics for statement-form (zero-output)
  calls are unverified — affects the echo documented choice. Mitigation:
  T1 inspects `output_count.rs` and an interpreter call site before tests
  are written; echo decision finalised at Gate 2.
- Risk: CI sandboxes may forbid `fork/exec`. Commands used are minimal
  (`/bin/sh`, `true`, `false`, `echo`); if a CI environment still blocks
  spawning, surface at Gate 3 with an env-gated skip proposal rather than
  silently weakening tests.
- Risk: test-name filter `system` collides with unrelated test names —
  use the module path filter
  `builtins::io::repl_fs::system` when running narrowly.
- Rollback: delete `system.rs` and the single `mod` line (no other file is
  modified).

## Verification commands

`cargo fmt --all -- --check` · `cargo check -p runmat-runtime` ·
`RUST_TEST_THREADS=1 cargo test -p runmat-runtime builtins::io::repl_fs::system`
