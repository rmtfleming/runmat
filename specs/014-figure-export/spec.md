# Feature Specification: Figure Export — `saveas`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`014-figure-export`; export `SPECIFY_FEATURE=014-figure-export`)

**Created**: 2026-07-22

**Status**: Batch 3 (F14) — B1 requirements, Gate 1 review pending

**Input**: Approved Tier A export `saveas` 0.2.0 (see `provenance.md`);
selection proposal F14 row in
`specs/001-clean-room-matlab-builtins/selection-proposal-batch3.md`
(approved 2026-07-22).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Save a figure to an image file (Priority: P1)

A RunMat user who has created a figure (e.g. `fig = figure(); plot(x, y)`)
calls `saveas(fig, filename)` and afterwards the target file exists on disk
and is non-empty. The call itself produces no output value; its effect is
the written file.

**Independent Test**: build a figure headlessly in a unit test, call
`saveas` with a unique temporary path, assert the file exists with nonzero
byte size, then delete the file.

**Acceptance Scenarios** (from observed results, normative):

1. **Given** an existing figure with plotted content, **When**
   `saveas(fig, file)` is called with a writable target path, **Then** the
   target file exists afterwards (the analogue of the observed
   `isfile(f) == true`, logical 1×1) and its byte size is nonzero (the
   analogue of the observed `dir(f).bytes == 12312`, double 1×1).
   [saveas.creates-file] *Note: the observed byte count (12312, an
   off-screen line plot saved as PNG of about 12 kB on R2026a/GLNXA64) is
   environment-specific; the normative claims are file-created and
   file-non-empty, not any exact byte count. RunMat tests MUST assert
   existence and `size > 0`, and MUST NOT assert the observed byte value.*
2. **Given** the same call, **When** it completes, **Then** no value is
   displayed or returned to the user; the effect is the written file.
   [saveas.creates-file — "saveas returns no output; the effect is the
   written file"]

### User Story 2 - Save the current figure via its handle (Priority: P2)

`saveas(gcf, filename)` works identically: in RunMat a figure handle is a
numeric (double) scalar as returned by `figure`/`gcf`, and `saveas` accepts
such a handle as its first argument. [saveas.signature-primary]

**Acceptance Scenario**:

1. **Given** a current figure, **When** `saveas(gcf, file)` is called,
   **Then** the file is created non-empty, as in User Story 1.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-014-01** `saveas` MUST accept the two-argument form
  `saveas(fig, filename)`, where `fig` is a figure handle (in RunMat: the
  numeric scalar handle produced by `figure`/`gcf`) and `filename` is text
  (char row vector or string scalar). [saveas.signature-primary]
- **FR-014-02** A successful `saveas(fig, filename)` call MUST create the
  target file: afterwards the file exists and has a nonzero byte size.
  [saveas.creates-file]
- **FR-014-03** `saveas` MUST NOT produce a visible output value; the
  observable effect is the written file. (RunMat models this with its
  house convention for side-effect builtins: suppressed auto-display, sink
  builtin — see plan.) [saveas.creates-file]
- **FR-014-04** Behaviour that the approved export leaves unresolved or
  pending MUST be handled by documented independent choice (see Unresolved
  behaviour below), MUST be tagged non-normative in tests, and MUST NOT be
  asserted as MATLAB-conformant. This covers: the supported format set and
  extension-based format inference (`saveas.q-formats`), the three-argument
  `saveas(fig, filename, formattype)` form (export status: pending),
  invalid-handle and invalid-argument errors (export lists no errors), and
  the exact no-output semantics under output capture.

### Key Entities

- Figure handle: RunMat `FigureHandle` (u32 newtype) surfaced to MATLAB
  code as a numeric double scalar (`figure` returns `f64`; handle
  arguments are accepted as `Value::Num`/`Value::Int`/1-element tensor).
- Export pipeline: the existing figure-export machinery used by `print`
  (`render_figure_snapshot` → PNG bytes → atomic file write through
  `runmat-filesystem`). No new rendering is introduced.

## Success Criteria *(mandatory)*

- **SC-014-1**: Both observed cases reproduce headlessly in normative
  tests: after `saveas`, the file exists (`file-created`) and has nonzero
  size (`file-nonempty`); exact byte counts are not asserted.
- **SC-014-2**: `saveas` is registered and discoverable via
  `builtin_function_by_name`; `cargo test -p runmat-runtime saveas` passes
  with default features (which include `plot-core`).
- **SC-014-3**: Traceability: each FR cites ≥1 claim id from
  `provenance.md`; each normative test cites ≥1 FR; documented-choice tests
  are tagged (`unresolved_choice`).
- **SC-014-4**: Test isolation: every test writes only uniquely named files
  under the system temp directory and removes them before finishing, in
  both success and failure paths.

## Unresolved behaviour (non-normative; implemented by documented choice)

The approved export confirms only the two observed claims. Everything
below is a RunMat-independent documented choice, tagged `unresolved_choice`
in tests, pending spec-side resolution of `saveas.q-formats` /
`saveas.q-side-effect` and promotion of the pending interface rows:

- **Supported formats / extension inference** (`saveas.q-formats`): RunMat
  initially supports PNG output only, matching the capability of the
  existing export substrate (`print` supports `-dpng` only). A `.png`
  extension (case-insensitive) selects PNG. A filename with no extension
  gets `.png` appended to its final path component (consistent with
  RunMat's `print`). Any other extension errors with
  `RunMat:saveas:UnsupportedFormat`. None of this is claimed
  MATLAB-conformant.
- **Three-argument form** `saveas(fig, filename, formattype)` (export
  status: pending): accepted with `formattype` of `'png'` only; other
  values error with `RunMat:saveas:UnsupportedFormat`.
- **Invalid handles / arguments** (export lists no error cases): a
  non-finite, non-positive or oversized handle scalar errors through the
  existing shared handle machinery; a positive handle that names no live
  figure errors (`figure handle N does not exist` via the render path,
  surfaced under a `saveas` error identifier); wrong argument counts or
  non-text filenames error with `RunMat:saveas:InvalidInput`.
- **No-output semantics under capture** (e.g. `v = saveas(...)`): the
  export records "(none)" with status pending. RunMat follows its
  side-effect builtin convention (internal status value, suppressed
  auto-display), as `print`/`drawnow` do; whether MATLAB errors on output
  capture is unobserved and not asserted.
- **Builds without `plot-core`**: outside the export's scope. Documented
  choice: `saveas` registers but the rendering step errors, exactly
  matching `print`'s behaviour ("plot-core support is not enabled in this
  build").

## Assumptions

- The existing `print` export substrate (`render_figure_snapshot`, atomic
  `runmat-filesystem` write) is behaviourally adequate for `saveas`; no
  new rendering or file-format code is required.
- Headless test execution is possible: the plotting test environment
  (`ensure_plot_test_env` + registry lock) plus `runmat-plot`'s CPU raster
  fallback already let `print`'s unit test write a real PNG in CI; `saveas`
  tests reuse the identical pattern (see plan, Test isolation).
- Applicable release for all normative claims: R2026a (GLNXA64), per the
  export.
