# Feature Specification: `spy` — plot the sparsity pattern of a matrix

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`040-spy`; export `SPECIFY_FEATURE=040-spy`)

**Created**: 2026-10-29 (revised the same day for spec 0.3.0)

**Status**: Batch C2 — implemented from the approved `spy` **0.3.0** export.
FR-010 decision D1 = B (mixed); Gates 1–3 pre-approved by the maintainer.
The maintainer's resume instruction: implement every form and option 0.3.0
confirms; default sizes and colours are non-normative (keep RunMat
defaults); points 0.3.0 leaves unresolved get `UnsupportedOption` or a
labelled documented choice. The 0.2.0-based draft of this file is
superseded.

**Input**: Approved Tier B export `spy` 0.3.0 (see `provenance.md`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Draw the nonzero pattern (Priority: P1)

`spy(A)` draws one marker-only line in the current axes at the nonzero
positions of `A` (column on x, row on y), sets the axes limits to the matrix
extent with equal data units, labels the x axis `nz = <count>`, and returns
no output.

**Acceptance Scenarios** (normative):

1. `spy(sparse([1 0;0 1]))` → the current axes gains children; no output.
   [spy.void-plot]
2. `h = spy(A)` errors. [spy.void-plot, spy.error-output-arg]
3. `spy(sparse([1 0 1;0 1 0]))` → exactly one line, `LineStyle` `'none'`,
   with markers; `XData` `[1 2 3]`, `YData` `[1 2 1]`; `XLim` `[0 4]`,
   `YLim` `[0 3]`; equal first two `DataAspectRatio` entries; x label
   `'nz = 3'`. [spy.one-line, spy.positions, spy.axes, spy.nz-label,
   spy.default-marker]
4. `spy(sparse([1;0;0;1]))` → `XLim` `[0 2]`, `YLim` `[0 5]`. [spy.axes]
5. `spy(sparse(3,3))` → one line with `NaN` coordinates, `XLim`/`YLim`
   `[0 4]`, label `'nz = 0'`. [spy.all-zero, spy.axes, spy.nz-label]
6. `spy([0 7;3 0])`, `spy(logical([1 0;1 1]))`, `spy('ab')` plot the
   nonzeros (`[1 2]/[2 1]`, `[1 1 2]/[1 2 2]`, `[1 2]/[1 1]`).
   [spy.input-types, spy.positions]

### User Story 2 - Style arguments (Priority: P1)

1. `spy(A,'r*')` → marker `'*'`, colour red. `spy(A,'g+',12)` → marker
   `'+'`, colour green, `MarkerSize` 12. [spy.linespec, spy.markersize]
2. `spy(A,20)` → `MarkerSize` 20 with the default marker. [spy.markersize]

### User Story 3 - Unresolved and unobserved forms fail clearly (Priority: P2)

1. Complex `A` → `RunMat:spy:UnsupportedOption` (spy.q-large).
2. A LineSpec containing a line style (`'r-'`, `'b:o'`), the reversed order
   `spy(A,12,'r*')`, an axes target `spy(ax,A)`, more than three arguments,
   or name-value pairs → `RunMat:spy:UnsupportedOption` (unobserved,
   U-040-2).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-040-01** `spy(A, …)` MUST draw into the current axes and return no
  output value; requesting an output MUST raise
  `RunMat:spy:TooManyOutputs` [spy.void-plot, spy.error-output-arg]. The
  observed identifier is in the MATLAB namespace; RunMat keeps its own
  `RunMat:<fn>:TooManyOutputs` convention (as `display`, `diary`), so the
  error condition is normative and the identifier text is not (deviation
  D-040-3, same policy as batch C2 decision D2).
- **FR-040-02** The axes MUST receive exactly one line object with markers
  and `LineStyle` `'none'` [spy.one-line]. It is drawn through RunMat's
  `plot`, so it follows the `hold` state (replaces content unless hold is
  on).
- **FR-040-03** `XData` MUST be the 1-based column indices and `YData` the
  1-based row indices of the nonzeros, in column-major order
  [spy.positions].
- **FR-040-04** For an m-by-n `A`, `XLim` MUST be `[0 n+1]` and `YLim`
  `[0 m+1]`, and the first two `DataAspectRatio` entries MUST be equal
  (RunMat sets `[1 1 1]`) [spy.axes]. The claim's `YDir 'reverse'` part is
  **not implemented**: RunMat axes have no direction model (the `axis ij`
  token is accepted as a no-op upstream). Deviation D-040-1.
- **FR-040-05** The x-axis label MUST be `'nz = '` followed by the number
  of nonzeros [spy.nz-label].
- **FR-040-06** With no style arguments the marker MUST be `'.'`
  [spy.default-marker]. RunMat passes the `'.'` token to `plot`; RunMat's
  line model stores it as a point-sized circle and `get(h,'Marker')`
  reports `'o'`. Deviation D-040-2.
- **FR-040-07** A LineSpec argument MUST set marker and colour
  [spy.linespec]. Accepted characters: markers `o + * . x ^ v < > s d p h`,
  colours `r g b c m y k w`. With a colour but no marker, the marker is `'.'`
  (documented choice).
- **FR-040-08** A numeric marker-size argument MUST set `MarkerSize`, alone
  or after a LineSpec [spy.markersize]. It must be a real, finite, positive
  scalar.
- **FR-040-09** An all-zero (or empty) `A` MUST still produce one line, with
  `NaN` coordinates [spy.all-zero].
- **FR-040-10** Full numeric (real), logical and char inputs MUST be
  accepted, and their nonzero elements plotted [spy.input-types]. Sparse
  real/logical inputs are accepted too (the observed `plots`/`points`
  cases use `sparse`). gpuArray inputs are gathered.
- **FR-040-11** (documented choice; spy.default-style-values is
  non-normative; spy.q-default-markersize) Default marker size and colour
  are RunMat's own `plot` defaults, independent of the matrix size.
- **FR-040-12** (unresolved — spy.q-large) Complex `A` MUST raise
  `RunMat:spy:UnsupportedOption`. Large matrices get no special handling
  (documented choice).
- **FR-040-13** (unobserved — U-040-2) A LineSpec with a line-style token
  (`-`, `--`, `:`, `-.`), a marker size before a LineSpec, a non-scalar
  numeric second argument (e.g. `spy(ax, A)`), a non-numeric third
  argument, or more than three arguments MUST raise
  `RunMat:spy:UnsupportedOption`.
- **FR-040-14** (documented choice) No arguments; text-string, cell,
  struct, object or other non-numeric `A`; `A` with more than two
  dimensions; an empty LineSpec or one with an unrecognised character; and
  a non-positive, non-finite or non-real marker size MUST raise
  `RunMat:spy:InvalidInput`.

## Success Criteria *(mandatory)*

- **SC-040-1**: every observed case in the 0.3.0 export that RunMat can
  represent reproduces in a normative test.
- **SC-040-2**: `spy` is registered through `#[runtime_builtin]`, and
  `cargo test -p runmat-runtime --lib -- spy_tests` passes.
- **SC-040-3**: every FR maps to at least one test, and every deviation is
  listed in validation.md.

## Unresolved behaviour and deviations

| id | question | disposition |
|---|---|---|
| spy.q-default-markersize | default size vs matrix size | documented choice: RunMat `plot` default, constant |
| spy.q-large | large and complex input | complex → `UnsupportedOption`; large: no special handling |
| U-040-2 (local) | line-style LineSpecs, reversed order, `spy(ax,A)`, name-value | `UnsupportedOption` |
| D-040-1 (deviation) | `YDir 'reverse'` (part of spy.axes) | not implemented: no axis-direction model in RunMat |
| D-040-2 (deviation) | `Marker` reads `'.'` (spy.default-marker) | RunMat reports `'o'` (point-sized circle) |
| D-040-3 (deviation) | identifier of the output-request error | `RunMat:spy:TooManyOutputs` |
| D-040-4 (deviation) | "axes Children became non-empty" proxy (spy.void-plot) | RunMat's axes `Children` lists only the axes' fixed text/legend objects, never plot lines; the new line is observed with `findobj(gca,'Type','line')` |

## Assumptions

- Upstream plotting supplies `plot`, `xlim`, `ylim`, `daspect`, `xlabel`,
  `gca`, `get`, `find` and the headless test environment. No upstream file
  is edited.
