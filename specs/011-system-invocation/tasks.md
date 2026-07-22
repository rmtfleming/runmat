# Tasks — 011-system-invocation

Order is normative (test-first). Every task cites requirements. The
SANDBOX / TEST-ISOLATION section of `plan.md` binds every task: test
commands are only `true`, `false`, `echo hello`; no shell metacharacters;
process-spawning tests gated `#[cfg(unix)]`.

- [x] **T1** Inspect `crate::output_count`
  (`current_output_count`, `output_list_with_padding`) and one interpreter
  call site to confirm multi-output and statement-form (zero-output)
  semantics; confirm the `io/repl_fs/mod.rs` registration pattern and the
  `fileparts.rs` exemplar shape. Record the echo-choice consequence for
  Gate 2. [FR-011-01, FR-011-04, FR-011-05] (no code changes)
- [x] **T2** Author failing normative tests for the three observed cases in
  the new file's `#[cfg(test)]` module (`#[cfg(unix)]`):
  `system('true')` → 0, `system('false')` → 1,
  `[status, cmdout] = system('echo hello')` → `cmdout` char 1×6
  `"hello\n"`. Tests reference the not-yet-written builtin — this is the
  red step; compilation is gated by T4. [FR-011-01..04; SC-011-1]
- [x] **T3** Author failing `unresolved_choice` and error tests: empty
  stdout → 0×0 char (`#[cfg(unix)]`); non-text input →
  `RunMat:system:InvalidInput` (no process spawn); unsupported-platform
  error text via the gated error path. [FR-011-05]
- [x] **T4** Implement `system.rs`: descriptor
  (`[status, cmdout] = system(command)`,
  `BuiltinOutputMode::ByRequestedOutputCount`), input validation, Unix
  `/bin/sh -c` execution with captured stdout, `status` as
  `Value::Num`, `cmdout` as char row vector (lossy UTF-8, trailing newline
  preserved), multi-output return via `crate::output_count`.
  [FR-011-01, FR-011-02, FR-011-03, FR-011-04]
- [x] **T5** Implement platform gating: wasm32 (and non-Unix without the
  documented `cmd /C` choice) returns `RunMat:system:PlatformUnsupported`;
  apply the Gate 2 echo decision for statement-form calls. [FR-011-05]
- [x] **T6** Register the builtin: `pub mod system;` in
  `io/repl_fs/mod.rs`; descriptor doc fields for `docs/builtins` tooling;
  confirm discovery via `builtin_function_by_name`. [FR-011-01; SC-011-2]
- [x] **T7** Run verification commands (`cargo fmt --all -- --check`,
  `cargo check -p runmat-runtime`, `RUST_TEST_THREADS=1 cargo test -p
  runmat-runtime builtins::io::repl_fs::system`); record real output in
  `validation.md` — no fabricated results. [SC-011-1, SC-011-2]
- [x] **T8** Traceability table in `validation.md` (claim → FR → test →
  result), including `unresolved_choice` tagging and the carried questions
  `system.q-side-effect` / `system.q-outputs`. [SC-011-3; Principle VI]
