# Feature Specification: `fplot`, `rectangle`, `validatecolor` — graphics builtins (batch N5)

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`065-graphics-n5`; export `SPECIFY_FEATURE=065-graphics-n5`)

**Created**: 2026-10-31

**Status**: Batch N5, feature F65. Implemented from the approved `fplot`,
`rectangle` and `validatecolor` **0.2.0** exports (revision `1c91aa1`,
IMPORT-MANIFEST Batch 14). Gates 1–3 were pre-approved by the maintainer
(`specs/001-clean-room-matlab-builtins/selection-proposal-n5.md`).
Unresolved points get `UnsupportedOption` or a labelled documented choice.

**Input**: Approved exports `fplot`, `rectangle`, `validatecolor` 0.2.0 (see
`provenance.md`).

## RunMat graphics model (context for every requirement)

RunMat graphics handles are numbers (`double`); there are no graphics
object classes such as `FunctionLine` or `Rectangle`. `get`/`set` and dot
access on a numeric handle go through RunMat's plot-object registry, which
has a fixed set of object kinds (line, patch, surface, …). Adding a kind is
an upstream change outside this feature. Line colours and widths are stored
as `f32`. The deviations below follow from this model.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - fplot draws a function as a line (P1)

`fplot(f)` samples a function of x over `[-5 5]` (or a given interval) and
draws one line. The COBRA form `fplot(@(x) x, 'LineWidth', 1.5, 'Color',
'r')` works without an output.

1. `h = fplot(@(x) x)` → one line in the current axes, more than two
   points, XData from -5 to 5, x-limits `[-5 5]`, LineWidth 0.5, LineStyle
   `'-'`. [fplot.object, fplot.default-interval, fplot.linespec]
2. `fplot(@(x) x.^2, [0 2])` → XData from 0 to 2 and YData equal to
   XData.^2 exactly. [fplot.interval]
3. COBRA form → LineWidth 1.5, Color `[1 0 0]`; without an output the axes
   hold one such line. [fplot.cobra-form]
4. `fplot(@(x) x, 'r--')` → Color `[1 0 0]`, LineStyle `'--'`.
   [fplot.linespec]
5. `fplot(ax, @(x) x)` draws into `ax`. [fplot.ax]
6. `fplot('sin(x)')` draws and warns `RunMat:fplot:StringFunctionsDeprecated`
   (reported by `lastwarn`). [fplot.warning-char-fn]
7. `fplot(5)` plots the constant over `[-5 5]`. [fplot.numeric, non-normative]
8. `[2 1]` → `RunMat:fplot:RangeVector`; `'NotAProperty'` →
   `RunMat:fplot:InvalidParameter`. [fplot.error-decreasing,
   fplot.error-bad-property]

### User Story 2 - rectangle draws a rectangle object (P1)

1. `h = rectangle()` → class `matlab.graphics.primitive.Rectangle`,
   Position `[0 0 1 1]`, Curvature `[0 0]`, FaceColor `'none'`, LineWidth
   0.5, LineStyle `'-'`. [rectangle.defaults]
2. COBRA form `rectangle('Position', [0, 0, 40, 0.1], 'EdgeColor',
   'black', 'LineWidth', 2, 'LineStyle', '-.')` → those values (EdgeColor
   `[0 0 0]`); fresh axes get limits x `[0 40]`, y `[0 0.1]`.
   [rectangle.cobra-form]
3. Curvature stored as given (1 stays 1; `[0.5 0.2]` stays). FaceColor
   `'r'` stored as `[1 0 0]`. [rectangle.curvature, rectangle.colors]
4. Without an output one object is added to the current axes;
   `rectangle(ax, ...)` draws into `ax`. [rectangle.parent]
5. Negative width → `RunMat:hg:set_chck:DimensionsOutsideRange`; 3-element
   Position → `RunMat:hg:dt_conv:Matrix_to_matlabRect:Expected4ElementVector`;
   Curvature above 1 → `RunMat:hg:shaped_arrays:CurvatureRange`; unknown
   name → `RunMat:hg:InvalidProperty`. [rectangle.error-*]

### User Story 3 - validatecolor normalises colours (P1)

1. `validatecolor('r')` → `[1 0 0]`; COBRA `any(validatecolor('r'))` is
   true. [validatecolor.signature-primary, validatecolor.cobra-form]
2. Names `'r'`, `'red'`, `"blue"`, `'Green'`, `'k'`. [validatecolor.names]
3. double/single triplets in [0, 1] returned as double; uint8 divided by
   255. [validatecolor.triplets]
4. `'#FF8000'` → `[1 128/255 0]`; `'#f80'` → `[1 0x88/255 0]`.
   [validatecolor.hex]
5. `'multiple'`: cell of names or m-by-3 matrix → one row each; `'one'`
   with `'r'` → `[1 0 0]`. [validatecolor.multiple]
6. Errors: m-by-3 (m > 1) without `'multiple'` →
   `RunMat:graphics:validatecolor:MultipleColors`; unknown name or
   `'none'` → `…:InvalidColorString`; component above 1 → `…:OutOfRange`;
   2 elements or a scalar → `…:InvalidTripletShape`; bad option →
   `RunMat:unrecognizedStringChoice`. [validatecolor.error-*]

### User Story 4 - unobserved forms fail clearly (P2)

Forms the exports do not cover raise `RunMat:<fn>:UnsupportedOption` or a
labelled documented-choice error.

## Requirements *(mandatory)*

### fplot

- **FR-065-01** `fplot(f, ...)` MUST create one line object in the current
  axes and return its handle; the line MUST have more than two points
  [fplot.signature-primary, fplot.object]. RunMat returns a numeric line
  handle (`Type` `'line'`); the `FunctionLine` class is not representable
  (D-065-1).
- **FR-065-02** Without an interval, XData MUST run from -5 to 5 and the
  axes x-limits MUST be `[-5 5]` [fplot.default-interval].
- **FR-065-03** `fplot(f, [xmin xmax])` MUST give XData from `xmin` to
  `xmax`, and YData MUST equal `f(XData)` exactly (checked for
  `@(x) x.^2` on `[0 2]`) [fplot.interval].
- **FR-065-04** The COBRA call `fplot(@(x) x, 'LineWidth', 1.5, 'Color',
  'r')` MUST give LineWidth 1.5, Color `[1 0 0]`, parent = current axes;
  without an output the axes MUST contain exactly that one line
  [fplot.cobra-form]. RunMat's line `Color` property reports the colour
  name `'r'` for pure red (equal to `[1 0 0]` through `validatecolor`;
  D-065-7), and plot objects of an axes are listed by `allchild`, since
  RunMat's axes `Children` lists title/label objects (D-065-8).
- **FR-065-05** LineSpec `'r--'` MUST give Color `[1 0 0]` and LineStyle
  `'--'`; the default LineWidth MUST be 0.5 and LineStyle `'-'`
  [fplot.linespec]. (0.5 is normative here although fplot.q-theme notes it
  may depend on the theme.)
- **FR-065-06** `fplot(ax, f, ...)` MUST draw into `ax` [fplot.ax].
- **FR-065-07** A char (or string) function MUST still be drawn and MUST
  issue warning `RunMat:fplot:StringFunctionsDeprecated` through RunMat's
  warning machinery, so `lastwarn` reports it [fplot.warning-char-fn].
  Evaluating the expression needs RunMat's host expression evaluator (the
  eval hook); the expression is the body of `@(x) …` (documented choice:
  the independent variable is `x`). Without an evaluator the call raises
  `RunMat:fplot:UnsupportedOption` after the warning (D-065-3).
- **FR-065-08** A decreasing interval MUST raise `RunMat:fplot:RangeVector`
  [fplot.error-decreasing]. A name that is not a line property MUST raise
  `RunMat:fplot:InvalidParameter` [fplot.error-bad-property].
- **FR-065-09** (non-normative, implemented as observed) A real numeric
  scalar `f` is plotted as a constant over the interval [fplot.numeric].
- **FR-065-10** (documented choices)
  - Sampling: 201 equally spaced points including both ends; no adaptive
    refinement and no re-sampling on zoom (fplot.q-points, D-065-2). The
    function is first called once with the 1-by-201 row of x values; if it
    returns a real numeric array with 201 elements that is used, otherwise
    it is called once per point and must return a real scalar (else
    `RunMat:fplot:EvaluationFailed`). Non-finite values are kept.
  - Default colour: the axes colour order, as for `plot` (fplot.q-theme;
    the observed `[0.066 0.443 0.745]` is non-normative and not asserted).
  - Accepted name-value names (case-insensitive): `LineWidth`, `Color`,
    `LineStyle`, `Marker`, `MarkerSize`, `MarkerEdgeColor`,
    `MarkerFaceColor`, `DisplayName`. Any other name, or a non-text value
    in a name position, raises `InvalidParameter`; a text argument
    directly after f/interval that is not one of those names is parsed as
    a LineSpec, and if it is not a valid LineSpec raises
    `InvalidParameter`. Invalid property values raise
    `RunMat:fplot:InvalidInput`.
  - Interval: a 2-element real finite vector; equal ends or non-finite
    values raise `RangeVector`; any other numeric second argument (e.g. the
    4-element form) and a second function (parametric form) raise
    `RunMat:fplot:UnsupportedOption`.
  - No argument, an empty or non-scalar numeric f, and other classes raise
    `RunMat:fplot:InvalidInput` (empty/missing) or `UnsupportedOption`
    (non-scalar numeric, complex, other classes).
  - The x-limits are set to the interval after drawing (axes limit mode
    becomes manual; FR-065-02).

### rectangle

- **FR-065-20** `rectangle(Name, Value, ...)`, `h = rectangle()` and
  `h = rectangle(ax, ...)` MUST draw one rectangle into the current (or
  given) axes and return an object of class
  `matlab.graphics.primitive.Rectangle` [rectangle.signature-primary].
  RunMat returns a handle object whose properties `Type`, `Parent`,
  `Position`, `Curvature`, `FaceColor`, `EdgeColor`, `LineWidth`,
  `LineStyle` are read with dot access (D-065-4); the drawing is a RunMat
  patch (D-065-5).
- **FR-065-21** Defaults MUST be Position `[0 0 1 1]`, Curvature `[0 0]`,
  FaceColor `'none'`, LineWidth 0.5, LineStyle `'-'` [rectangle.defaults].
  Default EdgeColor is a documented choice, `[0 0 0]` (the observed dark
  grey is non-normative: rectangle.default-edge-color, rectangle.q-theme).
- **FR-065-22** The COBRA call MUST store Position `[0 0 40 0.1]`, EdgeColor
  `[0 0 0]`, LineWidth 2, LineStyle `'-.'`; in fresh axes the limits MUST
  become x `[0 40]`, y `[0 0.1]` [rectangle.cobra-form].
- **FR-065-23** Curvature MUST be stored as given: scalar stays scalar, a
  pair stays a 1-by-2 row [rectangle.curvature].
- **FR-065-24** A colour name MUST be stored as its RGB row: FaceColor
  `'r'` → `[1 0 0]` [rectangle.colors]. Colours are parsed with
  `validatecolor`'s single-colour parser (names, hex, triplets).
- **FR-065-25** Without an output one object MUST be added to the current
  axes; with `ax` first, `Parent` MUST equal `ax` [rectangle.parent]. The
  axes' `Children` lists the RunMat patch, not the Rectangle object
  (D-065-5).
- **FR-065-26** Errors (identifiers naming the graphics property layer are
  reproduced under `RunMat:` as observed; rectangle.q-error-ids returned to
  the human reviewer): negative width →
  `RunMat:hg:set_chck:DimensionsOutsideRange`
  [rectangle.error-negative-width]; Position without 4 elements →
  `RunMat:hg:dt_conv:Matrix_to_matlabRect:Expected4ElementVector`
  [rectangle.error-position-length]; a Curvature value above 1 →
  `RunMat:hg:shaped_arrays:CurvatureRange` [rectangle.error-curvature-range];
  an unknown name → `RunMat:hg:InvalidProperty`
  [rectangle.error-bad-property].
- **FR-065-27** (documented choices) Names are case-insensitive. A negative
  height or a non-finite Position element also raises
  `DimensionsOutsideRange`; a negative Curvature value also raises
  `CurvatureRange`; Curvature with other than 1 or 2 elements, a
  non-numeric Position, a non-positive LineWidth, an unknown LineStyle (not
  `-`, `--`, `:`, `-.`, `none`), an invalid colour (EdgeColor may also be
  `'none'`), a missing value, or a non-text name raise
  `RunMat:rectangle:InvalidInput`. Rendering: the outline is a polygon
  whose corners are quarter-ellipses with radii `cx·w/2`, `cy·h/2`
  (16 segments per corner), drawn as a patch with the face and edge
  colours and LineWidth; LineStyle is stored but edges are drawn solid
  (D-065-6). A rectangle with zero width or height (no face area) is drawn
  as a closed outline line instead of a patch. With `hold off` the rectangle is added without clearing
  existing plot objects (it is a primitive, documented choice).

### validatecolor

- **FR-065-30** `validatecolor(c)` MUST return a 1-by-3 double; `'r'` →
  `[1 0 0]` and `any(validatecolor('r'))` is true
  [validatecolor.signature-primary, validatecolor.cobra-form].
- **FR-065-31** Names `'r'`/`'red'` → `[1 0 0]`, string `"blue"` → `[0 0
  1]`, `'Green'` → `[0 1 0]`, `'k'` → `[0 0 0]` [validatecolor.names].
  The name table is RunMat's plotting colour table (eight names and their
  one-letter forms; validatecolor.q-names).
- **FR-065-32** double and single triplets in [0, 1] MUST be returned as
  double unchanged; uint8 MUST be divided by 255 [validatecolor.triplets].
- **FR-065-33** `'#RRGGBB'` and `'#RGB'` hex codes MUST give component/255
  (`#f80` → `[1 0x88/255 0]`) [validatecolor.hex].
- **FR-065-34** With `'multiple'`, a cell of colours or an m-by-3 matrix
  MUST give an m-by-3 result; `'one'` behaves as the default
  [validatecolor.multiple].
- **FR-065-35** Errors: m-by-3 with m > 1 without `'multiple'` →
  `RunMat:graphics:validatecolor:MultipleColors`; unknown name or `'none'`
  → `RunMat:graphics:validatecolor:InvalidColorString`; a component above
  1 → `RunMat:graphics:validatecolor:OutOfRange`; a numeric input that is
  not a triplet (2 elements, scalar) →
  `RunMat:graphics:validatecolor:InvalidTripletShape`; a second argument
  other than `'one'`/`'multiple'` → `RunMat:unrecognizedStringChoice`
  (validatecolor.q-error-ids returned to the human reviewer)
  [validatecolor.error-*].
- **FR-065-36** (documented choices) Option text is case-insensitive.
  Negative or NaN components also raise `OutOfRange`. uint16 is divided by
  65535; other integer classes, logical, complex and non-numeric non-text
  inputs raise `RunMat:validatecolor:UnsupportedOption`. With `'multiple'`
  a string array or cell gives one row per element (each element one
  colour) and an empty numeric input gives 0-by-3; a matrix whose second
  dimension is not 3 raises `InvalidTripletShape`. Without `'multiple'` a
  1-element cell is accepted and a larger cell or string array raises
  `MultipleColors`. A multi-row char array raises `UnsupportedOption`.
  Text is trimmed. No argument → `RunMat:minrhs`; more than two →
  `RunMat:maxrhs`; a non-text option → `RunMat:unrecognizedStringChoice`.

## Success Criteria

- **SC-065-1** Every normative claim of the three exports maps to an FR and
  a passing test, or to a listed deviation with a reason.
- **SC-065-2** The surrounding `builtins::plotting` suite shows no new
  failures.
- **SC-065-3** `validation.md` records the failing (stub) run, the passing
  run, clippy and rustfmt results, and the freshness checks.

## Deviations (summary; details in `validation.md`)

- D-065-1 `fplot` returns a numeric line handle (`Type` `'line'`), not a
  `FunctionLine` object.
- D-065-2 `fplot` samples a fixed 201-point grid once (no adaptive
  sampling, no update on zoom or later `set` of the function).
- D-065-3 char-expression functions need the host expression evaluator.
- D-065-4 `rectangle` objects are handle objects read by dot access;
  `get`/`set` on them are not wired, and changing a property after creation
  does not redraw.
- D-065-5 the axes' `Children` lists the patch that draws the rectangle,
  not the Rectangle object.
- D-065-6 `rectangle` LineStyle is stored but edges render solid.
- D-065-7 RunMat's line `Color` property reports a colour name (`'r'`) for
  the eight named colours, not an RGB row.
- D-065-8 RunMat's axes `Children` property lists title/label objects;
  plot objects are listed by `allchild`.
