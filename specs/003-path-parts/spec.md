# Feature Specification: Path Parts — `fileparts`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id: `003-path-parts`; export `SPECIFY_FEATURE=003-path-parts`)

**Created**: 2026-07-22

**Status**: Batch 1 — combined review pending

**Input**: Approved Tier A export `fileparts` 0.2.0 (see `provenance.md`).

## User Scenarios & Testing

### User Story 1 - Split a path into parts (Priority: P1)

`[filepath, name, ext] = fileparts(filename)` splits a path; the directory
part matches observed behaviour exactly.

**Acceptance Scenarios** (observed, normative — first output only):

1. **Given** `'/a/b/c.txt'`-shaped input (case `full-path`), **When** called,
   **Then** output 1 is char `'/a/b'` (1×4). [fileparts.first-output-dir,
   fileparts.output-class]
2. **Given** a bare file name (case `name-only`), **Then** output 1 is empty
   char `''` with size 0×0. [fileparts.first-output-dir]
3. **Given** a trailing separator (case `trailing-sep`, `'/x/y/…'` form),
   **Then** the trailing separator is removed: `'/x/y'`.
   [fileparts.first-output-dir]
4. **Given** a name without extension (case `no-ext`), **Then** output 1 is
   `''` (0×0). [fileparts.first-output-dir]

## Requirements

- **FR-003-01** `fileparts` MUST accept one path argument and support the
  three-output form `[filepath, name, ext] = fileparts(filename)`
  [fileparts.signature-primary].
- **FR-003-02** Outputs MUST be class char for char input
  [fileparts.output-class]; observed empty results have size 0×0.
- **FR-003-03** Output 1 (directory part) MUST reproduce all four observed
  cases, including empty-for-bare-name and trailing-separator removal
  [fileparts.first-output-dir].
- **FR-003-04** (summary-derived) Outputs 2 and 3 SHOULD be the file name
  (without extension) and the extension respectively, per the approved
  summary ("directory, file name, and extension parts"); their exact
  values/edge cases are unobserved (`fileparts.q-outputs`) — tests tagged
  `summary_derived`, choices documented.
- **FR-003-05** Platform/separator handling beyond the observed `/` cases is
  unresolved (`fileparts.q-platform`); implementation documents its choice
  (POSIX `/` plus `\` on Windows targets) without conformance claims.

## Success Criteria

- **SC-003-1**: The four observed cases reproduce exactly (value, class,
  shape) in normative tests.
- **SC-003-2**: Three-output requests succeed via
  `ByRequestedOutputCount`; single-output requests return the directory part
  only, matching the observation harness's recording of output 1.
- **SC-003-3**: Full FR→test traceability.

## Unresolved behaviour (implemented by documented choice)

- `fileparts.q-outputs`: name/ext split rules (dotfiles, multiple dots,
  empty ext). Choice: last-dot split of the final path component; leading-dot
  names (`.bashrc`) treated as name with no extension; documented in code.
- `fileparts.q-platform`: separator handling on non-POSIX platforms. Choice:
  `/` always; `\` recognised additionally on Windows builds.
- String (`"…"`) input: accepted, returns string outputs — RunMat convention
  alignment, unobserved, tagged.

## Assumptions

- Lives beside `fullfile.rs` in `io/repl_fs` (its inverse operation).
- Pure text manipulation; no filesystem access.
