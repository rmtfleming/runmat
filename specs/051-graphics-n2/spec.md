# Feature Specification: `hsv`, `alpha`, `tiledlayout` — graphics builtins (batch N2)

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`051-graphics-n2`; export `SPECIFY_FEATURE=051-graphics-n2`)

**Created**: 2026-10-30

**Status**: Batch N2, feature F51. Implemented from the approved `hsv`,
`alpha` and `tiledlayout` **0.2.0** exports (revision `8f72edb`). Gates 1–3
were pre-approved by the maintainer for N2
(`specs/001-clean-room-matlab-builtins/selection-proposal-n2.md`).
Unresolved points get `UnsupportedOption` or a labelled documented choice.

**Input**: Approved exports `hsv`, `alpha`, `tiledlayout` 0.2.0 (see
`provenance.md`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - hsv colormap array (Priority: P1)

`c = hsv(n)` returns an n-by-3 double matrix of RGB rows whose hues are
evenly spaced around the colour wheel, starting at red.

**Acceptance Scenarios** (normative):

1. `hsv(6)` → `[1 0 0;1 1 0;0 1 0;0 1 1;0 0 1;1 0 1]`; `hsv(4)` →
   `[1 0 0;0.5 1 0;0 1 1;0.5 0 1]`; `hsv(1)` → `[1 0 0]`; class `double`.
   [hsv.values, hsv.output-class]
2. `hsv(64)` is 64-by-3 with values in [0, 1], first row `[1 0 0]`, last
   row `[1 0 0.09375]`. [hsv.range]
3. `hsv(0)` is 0-by-3. [hsv.zero]
4. `hsv` with no figure open is 256-by-3. [hsv.default]
5. `hsv(-1)` → `RunMat:validators:mustBeNonnegative`; `hsv(2.5)` →
   `RunMat:validators:mustBeInteger`. [hsv.error-negative, hsv.error-fraction]

### User Story 2 - alpha sets face transparency (Priority: P1)

`alpha(value)` sets `FaceAlpha` of every surface and patch in the current
axes; `alpha(obj, value)` sets it on `obj`. No output is returned.

1. After `surf(magic(3))`, `alpha(0.5)` → the surface's `FaceAlpha` is 0.5;
   on a patch, `alpha(0.4)` → 0.4; `alpha(s, 0.3)` → 0.3. [alpha.scalar]
2. `alpha('clear')` → 0; `alpha('opaque')` → 1. [alpha.keywords]
3. `EdgeAlpha` stays 1 after `alpha(0.5)`. [alpha.edge-untouched]
4. With only a line in the axes, `alpha(0.5)` succeeds. [alpha.line-only]
5. `a = alpha(0.5)` errors. [alpha.no-output, alpha.error-output-arg]
6. `alpha('nope')` → `RunMat:alpha:UnknownAlphaValue`.
   [alpha.error-bad-keyword]

### User Story 3 - tiledlayout creates a layout object (Priority: P1)

`t = tiledlayout(m, n)`, `tiledlayout('flow')`, `tiledlayout(parent, m, n)`
and the `TileSpacing`/`Padding` options return a layout object.

1. `tiledlayout(2,3)` → class `matlab.graphics.layout.TiledChartLayout`,
   `GridSize [2 3]`, `TileArrangement 'fixed'`, `TileSpacing` and `Padding`
   `'loose'`, no children. [tiledlayout.output-class, tiledlayout.grid]
2. Each added tile adds one child axes. [tiledlayout.tiles]
3. `'flow'` → `TileArrangement 'flow'`, `GridSize [1 1]` when empty, `[2 2]`
   after three tiles. [tiledlayout.flow]
4. `'TileSpacing','compact','Padding','tight'` are stored as given.
   [tiledlayout.options]
5. `tiledlayout(f,1,1)` → `Parent` is `f`. [tiledlayout.parent]
6. `tiledlayout(3)` → `RunMat:narginchk:notEnoughInputs`; `tiledlayout(0,2)`
   → `RunMat:tiledchartlayout:InvalidGridSize`; `TileSpacing 'nope'` →
   `RunMat:datatypes:InvalidEnumValue`. [tiledlayout.error-*]

### User Story 4 - unobserved forms fail clearly (Priority: P2)

Forms the exports do not cover, or that RunMat cannot represent, raise
`RunMat:<fn>:UnsupportedOption` or a labelled documented-choice error.

## Requirements *(mandatory)*

### hsv

- **FR-051-01** `hsv(n)` MUST return an n-by-3 double matrix
  [hsv.output-class]. Row `i` (0-based) MUST be the RGB colour of hue
  `i/n` at full saturation and value, which reproduces the observed
  `hsv(6)`, `hsv(4)`, `hsv(1)` and `hsv(64)` rows [hsv.values, hsv.range].
  RunMat computes it with the textbook HSV-to-RGB conversion (independently
  derived; the observations fix the hue spacing `i/n`).
- **FR-051-02** `hsv(0)` MUST return a 0-by-3 double matrix [hsv.zero].
- **FR-051-03** `hsv` with no argument and no figure open MUST return
  256-by-3 [hsv.default].
- **FR-051-04** A negative `n` MUST raise
  `RunMat:validators:mustBeNonnegative` [hsv.error-negative]; a
  non-integer `n` MUST raise `RunMat:validators:mustBeInteger`
  [hsv.error-fraction].
- **FR-051-05** (documented choice; hsv.q-default-with-figure) With a
  figure open, the default length is the current axes' colormap length, the
  rule of RunMat's colormap-array family (`parula`, `jet`, …).
- **FR-051-06** (documented choice) Integer-class scalars are accepted as
  lengths. A negative value (including `-1.5`, `-Inf`) raises
  `mustBeNonnegative`; `NaN`, `+Inf` and other non-integers raise
  `mustBeInteger`. A non-numeric or non-scalar length, a length above
  RunMat's 1 000 000 guard, or more than one argument raises
  `RunMat:hsv:InvalidInput`.
- **FR-051-07** (scope) `colormap('hsv')` is not part of the `hsv` export
  and is not added. `runmat_plot` has no hsv `ColorMap` variant (its own
  tests assert that), so `hsv` is a hand-written sibling of the
  `define_colormap_builtin!` family in `colormap_arrays.rs`, not a macro
  instance.

### alpha

- **FR-051-10** `alpha(value)` MUST set `FaceAlpha` of every surface and
  patch object in the current axes to the numeric `value`; `alpha(obj,
  value)` MUST set it on a surface or patch `obj` [alpha.scalar].
- **FR-051-11** `'clear'` MUST set `FaceAlpha` 0 and `'opaque'` MUST set 1
  [alpha.keywords].
- **FR-051-12** `alpha` MUST NOT change `EdgeAlpha` [alpha.edge-untouched].
- **FR-051-13** Axes without surfaces or patches (for example only a line)
  MUST be accepted without error and left unchanged [alpha.line-only].
- **FR-051-14** `alpha` returns no output; requesting one MUST raise
  `RunMat:alpha:TooManyOutputs` [alpha.no-output, alpha.error-output-arg].
  The observed identifier is `MATLAB:TooManyOutputs`; RunMat keeps the
  `RunMat:<fn>:TooManyOutputs` convention used by `spy` and
  `exportgraphics` (deviation D-051-5).
- **FR-051-15** Text other than `'clear'`, `'opaque'`, `'flat'` and
  `'interp'` MUST raise `RunMat:alpha:UnknownAlphaValue`
  [alpha.error-bad-keyword].
- **FR-051-16** (deviation D-051-2) `'flat'` and `'interp'` are confirmed to
  set `FaceAlpha` to those keywords [alpha.keywords]. RunMat stores
  `FaceAlpha` as a number and has no alpha-data model, so these keywords
  MUST raise `RunMat:alpha:UnsupportedOption` and leave objects unchanged.
- **FR-051-17** (documented choices; alpha.q-more-forms) Keywords are
  case-insensitive and may be char rows or string scalars. An axes handle
  as `obj` applies the current-axes rule to that axes. A numeric value that
  is not a real finite scalar in [0, 1], an `obj` of any other kind (line,
  figure, text, …), or more than two arguments raises
  `RunMat:alpha:UnsupportedOption`. No arguments, or an `obj` that is not a
  valid graphics handle, raises `RunMat:alpha:InvalidInput`. Only surface
  and patch objects are affected (bars, areas, histograms are not).

### tiledlayout

- **FR-051-20** `tiledlayout(m, n)`, `tiledlayout('flow')`,
  `tiledlayout(parent, m, n)` and these with trailing `Name, Value` pairs
  MUST return a handle object whose class is
  `matlab.graphics.layout.TiledChartLayout` [tiledlayout.output-class].
- **FR-051-21** A new fixed layout MUST have `GridSize [m n]`,
  `TileArrangement 'fixed'`, `TileSpacing 'loose'`, `Padding 'loose'` and
  no children [tiledlayout.grid]. Text properties are char rows; `GridSize`
  is a 1-by-2 double row; `Children` is a column of axes handles (0-by-1
  when empty).
- **FR-051-22** Each tile added to a layout MUST add exactly one child axes
  and make it the current axes [tiledlayout.tiles]. Tiles are added by the
  internal `add_tile`, which places tile `k` at subplot position `k` of the
  layout's grid in the parent figure (RunMat's subplot machinery).
- **FR-051-23** `'flow'` MUST give `TileArrangement 'flow'`, `GridSize
  [1 1]` when empty and `[2 2]` after three tiles [tiledlayout.flow].
- **FR-051-24** `TileSpacing` `'loose'`/`'compact'` and `Padding`
  `'loose'`/`'tight'` MUST be stored as given [tiledlayout.options].
- **FR-051-25** A figure passed first MUST become the layout's `Parent`, and
  its tiles go to that figure [tiledlayout.parent]. `Parent` is RunMat's
  numeric figure handle (deviation D-051-6).
- **FR-051-26** A single numeric argument MUST raise
  `RunMat:narginchk:notEnoughInputs` [tiledlayout.error-one-arg]; a zero
  grid dimension MUST raise `RunMat:tiledchartlayout:InvalidGridSize`
  [tiledlayout.error-zero]; an unrecognised `TileSpacing` value MUST raise
  `RunMat:datatypes:InvalidEnumValue` [tiledlayout.error-spacing].
- **FR-051-27** (documented choices) Flow growth for `k` tiles uses
  `cols = ceil(sqrt(k))`, `rows = ceil(k / cols)` (minimum `[1 1]`); this
  reproduces both observed points (tiledlayout.q-flow-growth). Option names
  and values are case-insensitive and stored lower-case. `TileSpacing` and
  `Padding` accept only the observed values; any other value raises
  `RunMat:datatypes:InvalidEnumValue` (MATLAB may accept more values; not
  observed). Negative, fractional and non-finite grid sizes raise
  `InvalidGridSize`. No arguments, or `m` without `n` before options, raises
  `notEnoughInputs`. `tiledlayout` does not clear or select the figure, and
  creates no axes until a tile is added.
- **FR-051-28** (unobserved forms) An arrangement text other than `'flow'`,
  an option name other than `TileSpacing`/`Padding`, or a tile beyond the
  `m*n` tiles of a fixed grid raises `RunMat:tiledlayout:UnsupportedOption`.
  Unpaired or non-text option names, or a parent that is not an existing
  figure, raise `RunMat:tiledlayout:InvalidInput`.

## Success Criteria *(mandatory)*

- **SC-051-1**: every observed case in the three 0.2.0 exports that RunMat
  can represent reproduces in a normative test.
- **SC-051-2**: the three builtins are registered through
  `#[runtime_builtin]`, and `cargo test -p runmat-runtime --lib --
  hsv_tests alpha_tests tiledlayout_tests` passes.
- **SC-051-3**: every FR maps to at least one test, and every deviation is
  listed in `validation.md`.

## Unresolved behaviour and deviations

| id | question | disposition |
|---|---|---|
| hsv.q-default-with-figure | default length with a figure open | documented choice: current colormap length (FR-051-05) |
| hsv.q-warning-coverage, alpha.q-warning-coverage, tiledlayout.q-warning-coverage | warnings not captured | no warnings are raised |
| alpha.q-more-forms | other values, objects, argument counts | `UnsupportedOption` / documented choices (FR-051-17) |
| tiledlayout.q-flow-growth | flow grid for other tile counts | documented rule (FR-051-27) |
| D-051-1 (deviation) | `FaceAlpha` read-back precision | RunMat stores `FaceAlpha` as f32: `alpha(s,0.3)` reads back `0.30000001192…`, not the double `0.3`; tests use a 1e-6 tolerance |
| D-051-2 (deviation) | `'flat'` / `'interp'` keywords (alpha.keywords) | `RunMat:alpha:UnsupportedOption`: no keyword or alpha-data storage for `FaceAlpha` upstream |
| D-051-3 (deviation) | `EdgeAlpha` of a surface (alpha.edge-untouched) | RunMat surfaces have no `EdgeAlpha` property; the rule is tested on a patch, whose `EdgeAlpha` stays 1 |
| D-051-4 (deviation) | `nexttile` (tiledlayout.tiles, tiledlayout.flow after tiles) | no approved `nexttile` spec exists, so no public builtin; the tile rule is implemented and tested through the internal `add_tile` |
| D-051-5 (deviation) | identifier of the output-request error | `RunMat:alpha:TooManyOutputs` |
| D-051-6 (deviation) | class of `Parent` (`matlab.ui.Figure`) | RunMat figures are numeric handles; `Parent` is the figure number (class `double`) |
| D-051-7 (deviation) | layout property access | properties are read with dot access (`t.GridSize`); `get(t, …)`/`set(t, …)` are not wired for layouts (would need upstream `get`/`set` changes) |
| D-051-8 (deviation) | layout rendering | `TileSpacing`/`Padding` are stored only; tiles use the subplot grid's spacing |

## Assumptions

- Upstream plotting supplies `surf`, `patch`, `plot`, `get`, `set`, `gca`,
  `figure`, the subplot grid (`configure_subplot`), handle encoding, and
  the headless test environment. `runmat_plot` already carries surface and
  patch face alpha into the vertex colours, and patch draws switch to
  blending below 1, so `alpha` only stores the property; how it looks when
  rendered is not verified headlessly. No upstream plotting file other than `colormap_arrays.rs`
  (one new sibling builtin) and the hand-written `ops/mod.rs` is edited.
