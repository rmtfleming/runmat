# Feature Specification: Platform Identifier — `computer`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`019-computer`; export `SPECIFY_FEATURE=019-computer`)

**Created**: 2026-07-22

**Status**: Tier B batch 1 — implementation authorised (FR-010 gate decision
recorded 2026-07-22 in `specs/001-clean-room-matlab-builtins/selection-proposal-tierb1.md`)

**Input**: Approved Tier B export `computer` 0.2.0 (see `provenance.md`;
import pinned at source commit `e46587ae1b78237e0f43cd55f678de01a28495a4`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Identify the current platform (Priority: P1)

RunMat users calling `computer` get a char row vector naming the platform the
runtime is executing on; `computer('arch')` gives the short lower-case
architecture string. On the observed platform (Linux x86_64) the values are
`'GLNXA64'` and `'glnxa64'`, each 1×7 char.

**Independent Test**: run both observed cases from the export through the
platform-mapping function (and, on a Linux x86_64 host, through the live
builtin) and compare value, class and shape.

**Acceptance Scenarios** (from observed results, normative):

1. **Given** the observed platform inputs (OS `linux`, architecture
   `x86_64`), **When** `computer` is called with no argument, **Then** the
   result is `'GLNXA64'`, class char, size 1×7.
   [computer.platform-strings, computer.output-class]
2. **Given** the same platform, **When** `computer('arch')` is called,
   **Then** the result is `'glnxa64'`, class char, size 1×7.
   [computer.platform-strings, computer.output-class]

## Requirements *(mandatory)*

### Functional Requirements

- **FR-019-01** `computer` MUST accept the zero-argument form
  `str = computer` [computer.signature-primary] and return class char
  [computer.output-class] as a row vector holding the upper-case platform
  identifier; on the observed platform (Linux x86_64) the value MUST be
  `'GLNXA64'` (1×7) [computer.platform-strings].
- **FR-019-02** `computer('arch')` MUST be accepted (confirmed invocation
  form) and return the lower-case architecture string as a char row vector;
  on the observed platform the value MUST be `'glnxa64'` (1×7)
  [computer.platform-strings, computer.output-class].
- **FR-019-03** The platform identifier MUST be derived from the host OS and
  CPU architecture (Rust `std::env::consts::{OS, ARCH}`) through a pure
  mapping. Only the `linux`/`x86_64` → `GLNXA64`/`glnxa64` pair is observed
  and normative. The remaining standard MATLAB pairs are documented
  independent choices, NOT asserted MATLAB-conformant: `macos`/`aarch64` →
  `MACA64`/`maca64`; `macos`/`x86_64` → `MACI64`/`maci64`;
  `windows`/`x86_64` → `PCWIN64`/`win64`. Any other OS/architecture
  combination falls back to a synthesized `OS-ARCH` identifier
  (upper/lower case) by documented choice (see Unresolved behaviour).
- **FR-019-04** Option handling beyond the observed exact text `'arch'` is a
  documented independent choice: the option is accepted as char row vector
  or string scalar, matched ASCII case-insensitively; any other option value
  raises `RunMat:computer:InvalidOption`; a non-text option raises the same
  identifier; more than one argument raises `RunMat:computer:TooManyInputs`.
- **FR-019-05** The multi-output form `[str, maxsize, endian] = computer` is
  unresolved in the approved export (`computer.q-multi-output`, not
  exercised) and MUST be omitted: the builtin declares a Fixed single
  output.

### Key Entities

- Result: `Value::CharArray` row vector (RunMat's char row vector — the
  observed class for both cases).
- Platform source: Rust `std::env::consts::OS` / `std::env::consts::ARCH`
  compile-time constants; no process, environment-variable, or MATLAB query.

## Success Criteria *(mandatory)*

- **SC-019-1**: Both observed cases in the export reproduce exactly (value,
  class char, size 1×7) in normative tests. Host independence: the observed
  pair is asserted through the pure mapping function with `linux`/`x86_64`
  inputs; an additional end-to-end test of the live builtin is compiled only
  on Linux x86_64 hosts (the observed platform).
- **SC-019-2**: `computer` is registered and discoverable via
  `builtin_function_by_name`; `cargo test -p runmat-runtime` passes for its
  test module.
- **SC-019-3**: Traceability: each FR cites ≥1 claim id (or is explicitly a
  documented choice); each normative test cites ≥1 FR.

## Unresolved behaviour (non-normative; implemented by documented choice)

- `computer.q-multi-output`: the `[str, maxsize, endian]` form is not
  exercised in the export. Choice: omit (Fixed single output); revisit when
  the specification side observes it.
- Non-observed platforms: only GLNXA64 is observed. Choice: standard MATLAB
  identifier pairs for macOS arm64/x86_64 and Windows x86_64 (FR-019-03);
  unmapped platforms (e.g. Linux aarch64, wasm32) synthesize
  `OS-ARCH`/`os-arch` so the result is always a non-empty char row vector.
- Option matching: only exact `'arch'` (char) is observed. Choice:
  case-insensitive match, char or string scalar accepted; anything else is
  an error with a stable identifier (FR-019-04).
- A `'version'`-style option is neither observed nor listed in the export;
  it is treated as an unrecognized option (error), not implemented.

## Assumptions

- RunMat's value model already provides `CharArray` row vectors; no new
  runtime types are required.
- `std::env::consts::{OS, ARCH}` identify the host adequately for the
  observed platform and the documented-choice platforms.
