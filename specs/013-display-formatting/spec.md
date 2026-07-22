# Feature Specification: Display Formatting — `display`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id: `013-display-formatting`; export `SPECIFY_FEATURE=013-display-formatting`)

**Created**: 2026-07-22

**Status**: Batch 3 (F13) — B1 requirements draft, Gate 1 pending

**Input**: Approved Tier A export `display` 0.2.0 (batch-2 packaged export,
commit pin `b054f3ad…34ef` — see `provenance.md`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Explicitly display a value (Priority: P1)

RunMat users calling `display(X)` get the value rendered as command-window
text, byte-identical to the approved observed renderings: numeric values in
indented, right-aligned columns followed by a blank line; char rows printed
bare (no quotes, no indent, no trailing blank line).

**Independent Test**: run each observed case through the builtin, capture the
emitted console text in-process (RunMat's console buffer — no evalc, no
MATLAB), and compare the captured bytes to the observed strings.

**Acceptance Scenarios** (from observed results, normative; strings written
with `\n` denoting the newline char):

1. **Given** the scalar `42`, **When** `display(42)` runs, **Then** the
   emitted text is exactly `"    42\n\n"` — a 6-character right-aligned field
   (4 spaces + `42`), one newline, one empty line. [display.formatted-text /
   case `scalar-text`; display.signature-primary]
2. **Given** the row vector `[1 2 3]`, **When** `display([1 2 3])` runs,
   **Then** the emitted text is exactly `"     1     2     3\n\n"` — three
   contiguous width-6 right-aligned fields (no separator between fields),
   one newline, one empty line. [display.formatted-text / case `vector-text`]
3. **Given** the char row `'hi'`, **When** `display('hi')` runs, **Then** the
   emitted text is exactly `"hi\n"` — no surrounding quotes, no leading
   indent, no trailing blank line. [display.formatted-text / case `char-text`]

Note: all three observed calls pass a *literal* (unnamed) argument and none
of the observed texts contains a variable-name header (e.g. `x =`). Header
behaviour for named variables is NOT part of the export and stays unresolved
(see Unresolved behaviour).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-013-01** `display` MUST accept the single-argument form `display(X)`
  [display.signature-primary].
- **FR-013-02** `display(X)`'s observable effect MUST be text emitted to
  RunMat's command-window/console output channel, capturable in-process for
  testing [display.formatted-text; unresolved `display.q-side-effect` scopes
  anything beyond the emitted text]. The builtin's own return value is not
  constrained by the export (`display.q-return`); RunMat uses its standard
  sink convention (0x0 empty, auto-display suppressed) as a documented
  choice.
- **FR-013-03** `display(42)` MUST emit exactly `"    42\n\n"`
  [display.formatted-text / `scalar-text`].
- **FR-013-04** `display([1 2 3])` MUST emit exactly
  `"     1     2     3\n\n"` (width-6 right-aligned fields, contiguous, no
  inter-column separator) [display.formatted-text / `vector-text`].
- **FR-013-05** `display('hi')` MUST emit exactly `"hi\n"` — char rows are
  shown without surrounding quotes, without leading whitespace, and without a
  trailing blank line [display.formatted-text / `char-text`].
- **FR-013-06** Inputs and forms not covered by the export — `display(X,
  name)`, named-variable headers, structs, cells, string arrays, complex
  values, empties, multi-row char arrays, GPU values, non-default `format`
  modes, overloaded class display — MUST be handled by documented independent
  choice (see Unresolved behaviour) and MUST NOT be asserted as
  MATLAB-conformant.

### Key Entities

- Emitted text: produced by a pure formatting function returning `String`
  (exact bytes, including all trailing newlines); the builtin writes that
  string to RunMat's console recording layer
  (`crates/runmat-runtime/src/console.rs`), which tests drain in-process.
- The builtin lives beside `disp` under
  `crates/runmat-runtime/src/builtins/io/` and follows the same sink
  conventions (GPU gather before rendering, fixed output mode).

## Success Criteria *(mandatory)*

- **SC-013-1**: Each of the three observed cases reproduces byte-for-byte:
  the pure formatter's `String` equals the observed text, and the
  console-captured text from an actual builtin invocation equals it too.
- **SC-013-2**: `display` is registered and discoverable via
  `builtin_function_by_name`; `cargo test -p runmat-runtime display` passes
  for its test module.
- **SC-013-3**: Traceability: each FR cites ≥1 claim id; each normative test
  cites ≥1 FR; unresolved-choice tests are tagged and cite the unresolved id.
- **SC-013-4**: No global change to `disp`'s existing rendering behaviour
  (its test module passes unmodified except, at most, visibility of helper
  functions).

## Unresolved behaviour (non-normative; implemented by documented choice)

Carried from the export (`behaviour.md` / `interface.yaml`):

- `display.q-side-effect`: only the evalc-captured text was observed; how the
  side effect should be specified more broadly is open. Choice: emit exactly
  the observed bytes through RunMat's console layer; nothing else.
- `display.q-overload`: per-class overloading is out of scope; RunMat
  registers a single builtin.
- `display.q-return`: return value unobserved. Choice: return RunMat's
  standard sink placeholder (0x0 double) with auto-display suppressed,
  matching `disp`.

Absent from the export (documented independent choices, tagged
`unresolved_choice` in tests):

- **Named-variable header** (`x =`): no observed case shows a header, and all
  observed calls pass literals. Choice: the builtin never prints a header
  (RunMat builtins do not receive caller variable names). If the spec side
  later observes header behaviour, this returns to Gate 1.
- **`display(X, name)`**: listed `status: pending` in the export — not
  approved. Choice: RunMat rejects a second argument with a clear error
  (`display: the two-argument form is not yet specified`), rather than
  inventing header formatting. Revisit when the form is approved.
- **Other value classes / shapes** (matrices with >1 row, non-integer values,
  complex, logical, struct, cell, string arrays, empties, multi-row char):
  unobserved. Choice: real numeric 2-D arrays generalise the observed layout
  (right-aligned fields, minimum width 6, no inter-column separator, 1 row
  per matrix row, trailing blank line); all other kinds delegate to RunMat's
  existing `disp` rendering plus a trailing blank line for non-char-like
  values. These renderings are RunMat choices, not MATLAB-conformance claims.
- **Interaction with `format` modes** (`compact`/`loose`, `long`, …): the
  observed texts were produced under MATLAB's default formatting state.
  RunMat currently stores no loose/compact spacing state (`io/format.rs`
  accepts the keywords as no-ops); choice: `display` hard-codes the observed
  default spacing (trailing blank line for numeric, none for char).

## Assumptions

- RunMat's console layer (`record_console_output` / `take_thread_buffer`)
  preserves the recorded text byte-for-byte per thread; existing tests
  (`math/optim/quad.rs`, `fzero.rs`) already rely on this.
- `runmat_builtins::format_number` renders `42.0` as `42` and small integers
  as their digit strings under the default `FormatMode::Short` (verified in
  `crates/runmat-builtins/src/lib.rs::fmt_short`), so the observed field
  contents are achievable from the existing scalar formatter plus width-6
  right-alignment.
