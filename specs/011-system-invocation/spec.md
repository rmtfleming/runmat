# Feature Specification: OS Command Invocation — `system`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`011-system-invocation`; export `SPECIFY_FEATURE=011-system-invocation`)

**Created**: 2026-07-22

**Status**: Batch 3 (F11) — combined review pending (Gate 1)

**Input**: Approved export `system` 0.2.0 (see `provenance.md`; selection
approved as F11 in
`specs/001-clean-room-matlab-builtins/selection-proposal-batch3.md`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Run an OS command and branch on its exit status (Priority: P1)

RunMat users call `status = system(command)` to run an operating-system
command and receive its numeric exit status, matching the approved observed
behaviour: a successful command yields `0`, the observed failing command
yielded `1`.

**Independent Test**: run each observed status case from the export through
the builtin and compare value, class and shape.

**Acceptance Scenarios** (from observed results, normative):

1. **Given** the command `'true'`, **When** `status = system('true')` is
   evaluated, **Then** `status` is a numeric (double) 1×1 scalar equal to
   `0`. [system.signature-primary, system.status-and-output; case
   `status-success`]
2. **Given** the command `'false'`, **When** `status = system('false')` is
   evaluated, **Then** `status` is a numeric (double) 1×1 scalar equal to
   `1`. [system.status-and-output; case `status-failure`]

### User Story 2 - Capture a command's standard output (Priority: P1)

Users call `[status, cmdout] = system(command)` to additionally capture the
command's standard output as text.

**Acceptance Scenarios**:

1. **Given** the command `'echo hello'`, **When**
   `[status, cmdout] = system('echo hello')` is evaluated, **Then** `cmdout`
   is a char row vector of size 1×6 containing `'hello'` followed by a
   trailing newline (`char(10)`), and `status` is `0`.
   [system.status-and-output; case `captured-output`]

## Requirements *(mandatory)*

### Functional Requirements

- **FR-011-01** `system` MUST accept the primary invocation form
  `status = system(command)`, where `command` is command text.
  [system.signature-primary]
- **FR-011-02** The first output `status` MUST be a numeric (double) 1×1
  scalar carrying the command's exit status: `0` for the observed successful
  command (`true`). [system.status-and-output; case `status-success`]
- **FR-011-03** The observed failing command (`false`) MUST yield
  `status == 1`. The export states the full range of nonzero failure codes
  was not explored; behaviour is normative only for the observed
  0/1 passthrough, and general exit-code passthrough is the documented
  choice consistent with it. [system.status-and-output; case
  `status-failure`]
- **FR-011-04** The two-output form `[status, cmdout] = system(command)`
  MUST be accepted, and `cmdout` MUST capture the command's standard output
  as a char row vector including the trailing newline: `system('echo
  hello')` yields a 1×6 char `'hello'` + `char(10)`.
  [system.status-and-output; case `captured-output`]
- **FR-011-05** Behaviour not covered by the export — shell selection,
  stderr handling, environment inheritance, echo-to-console when no output
  is requested, `cmdout` text encoding, the range of nonzero exit codes,
  signal-terminated processes, non-text `command` inputs, Windows behaviour,
  and WASM/unsupported platforms — MUST be handled by documented independent
  choice (see Unresolved behaviour) and MUST NOT be asserted as
  MATLAB-conformant.

### Key Entities

- `status`: `Value::Num` (RunMat's double scalar).
- `cmdout`: `Value::CharArray` row vector (empty stdout representation is a
  documented choice).
- Process execution: RunMat's own process spawn via Rust `std::process`;
  never through MATLAB.

## Success Criteria *(mandatory)*

- **SC-011-1**: Every observed case in the export (`status-success`,
  `status-failure`, `captured-output`) reproduces exactly (value, class,
  size) in normative tests on a Unix host.
- **SC-011-2**: `system` is registered and discoverable via
  `builtin_function_by_name`; `cargo test -p runmat-runtime` passes for its
  test module.
- **SC-011-3**: Traceability: each FR cites ≥1 claim id; each normative test
  cites ≥1 FR.

## Unresolved behaviour (non-normative; implemented by documented choice)

The export confirms only the three observed cases; everything below is a
documented independent choice, tagged `unresolved_choice` in tests, and
revisable if the specification side later observes it.

- **Shell selection** [system.q-side-effect]: the export does not say how
  the command line is interpreted. Choice: on Unix targets, run via
  `/bin/sh -c <command>`; on Windows targets, `cmd /C <command>`
  (untested, out of observed scope — export platform is GLNXA64).
- **stderr**: unobserved. Choice: standard error is not captured into
  `cmdout`; it passes through to the host process's stderr.
- **Environment**: unobserved. Choice: the child inherits RunMat's
  environment and current working directory.
- **Echo to console with no requested output** (`system(cmd)` as a
  statement): unobserved. Choice: captured stdout is forwarded to RunMat's
  console stdout stream when fewer than two outputs are requested; exact
  policy is finalised at Gate 2 (fallback: no echo in v1).
- **`cmdout` encoding** [system.q-outputs]: unobserved. Choice: captured
  bytes are decoded as UTF-8 (lossy) into a char row vector.
- **Nonzero exit codes** [system.q-outputs]: only `1` observed. Choice:
  pass the child's exit code through numerically; a signal-terminated child
  (no exit code) maps to `128 + signal` (Unix convention).
- **Empty stdout**: unobserved. Choice: `cmdout` is an empty (0×0) char
  array.
- **Non-text `command` input**: unobserved. Choice: error
  `RunMat:system:InvalidInput` for inputs that are not a char row vector or
  string scalar.
- **WASM / platforms without process spawning**: out of scope of the export.
  Choice: a clear error (`RunMat:system:PlatformUnsupported`); `system`
  never silently no-ops.

## Sandbox and test isolation (carried from selection proposal F11)

- Tests spawn only `true`, `false`, and `echo hello` — fixed string
  literals; benign, platform-portable, no side effects.
- No shell metacharacters, pipes, redirection, environment mutation,
  filesystem writes, or network access in any test.
- Observed-case tests are gated `#[cfg(unix)]` (export platform GLNXA64);
  no test asserts Windows or WASM behaviour beyond the unsupported error.

## Assumptions

- RunMat's existing multi-output machinery
  (`BuiltinOutputMode::ByRequestedOutputCount` + `crate::output_count`)
  supports the `[status, cmdout]` form; exemplar
  `crates/runmat-runtime/src/builtins/io/repl_fs/fileparts.rs`.
- `std::process::Command` is available on native targets (already used in
  the workspace); no new dependency is required.
