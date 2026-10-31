# Feature Specification: `findall`, `ishghandle`, `allchild`, `pcolor`, `surfl` — graphics builtins (batches N3+N4)

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`058-graphics-n3n4`; export `SPECIFY_FEATURE=058-graphics-n3n4`)

**Created**: 2026-10-30

**Status**: Batches N3+N4, feature F58. Implemented from the approved
`findall` and `ishghandle` **0.2.0** exports (N3, revision `1156c97`) and
the approved `allchild`, `pcolor` and `surfl` **0.2.0** exports (N4,
revision `fc9f804`). Gates 1–3 were pre-approved by the maintainer for N3
and N4 (`specs/001-clean-room-matlab-builtins/selection-proposal-n3n4.md`).
Unresolved points get `UnsupportedOption` or a labelled documented choice.

**Input**: Approved exports `findall`, `ishghandle`, `allchild`, `pcolor`,
`surfl` 0.2.0 (see `provenance.md`).

## RunMat graphics model (context for every requirement)

RunMat graphics handles are numbers (`double`), not handle objects. The
object tree is root (0) → figures → axes → plot objects; titles, axis
labels and legends are objects held by an axes; rulers (`XAxis`, `YAxis`)
are objects reachable through axes properties. RunMat has no
`HandleVisibility` property, no `GraphicsPlaceholder` class, no annotation
pane, and its surface objects have no `CData` property (colour comes from Z
through the colormap, from a flattened colour source, or from a per-vertex
colour grid). The deviations below follow from this model.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - findall finds every object, including labels (P1)

`h = findall(obj)` returns `obj` and every object under it as a column;
`'Type', t` and `'-property', name` filter it. The COBRA pattern
`set(findall(fig, '-property', 'FontSize'), 'FontSize', 12)` works.

1. Figure with two lines: `'Type','line'` → 2x1; `'Type','axes'` → the one
   axes. [findall.type]
2. After `title` and `xlabel`, `'Type','text'` finds 2 objects.
   [findall.hidden]
3. With lines, title, label and a `text` object, `'-property','FontSize'`
   finds objects of Type axes and text only. [findall.property]
4. The `set` pattern sets FontSize 12 on the axes and the text object.
   [findall.set-pattern]
5. `findall(fig)` includes `fig`, first. [findall.self]
6. No match → empty 0x0. [findall.no-match]
7. A deleted figure handle → empty 0x0, no error. [findall.deleted]
8. A number that is not a handle → `RunMat:findall:InvalidHandles`.
   [findall.error-not-handle]

### User Story 2 - ishghandle tests handles elementwise (P1)

1. True for live figure, axes and line handles, elementwise for `[f ax
   ln]`, for `double(f)`, for `0` and for `groot`. [ishghandle.live]
2. `ishghandle(f,'figure')`, `ishghandle(ax,'axes')` true;
   `ishghandle(ax,'figure')` false. [ishghandle.type]
3. False for a deleted figure, `123456`, `'a'`, a struct, a graph object;
   a 2x2 non-integer matrix gives a 2x2 false. [ishghandle.false-cases]
4. `ishghandle([])` → 0x0 logical. [ishghandle.empty]
5. `ishghandle()` → `RunMat:minrhs`. [ishghandle.error-none]

### User Story 3 - allchild lists children (P1)

1. `allchild(0)` is a column that includes a new invisible figure.
   [allchild.root]
2. For a figure with one axes the result includes the axes (MATLAB also
   lists an annotation pane: D-058-4). [allchild.figure]
3. For axes with one line and a title, the result is the one line.
   [allchild.axes]
4. For a line, the result is empty 0x0. [allchild.leaf]
5. For `[fig1 fig2]` the result is a 2x1 cell. [allchild.array]
6. A deleted handle, `123456` or `'a'` → `RunMat:allchild:InvalidHandles`.
   [allchild.error-deleted]

### User Story 4 - pcolor draws a flat coloured surface (P1)

1. `h = pcolor(C)` for 2x3 C → surface with XData `[1 2 3]`, YData
   `[1;2]`, ZData `zeros(2,3)`, colour data C. [pcolor.output-class,
   pcolor.c-only]
2. `pcolor(x, y, C)`: vectors kept as given; meshgrid matrices kept as
   given; ZData zeros. [pcolor.xyc]
3. `pcolor(ax, C)` draws into `ax`; without `ax`, into `gca`.
   [pcolor.ax-arg, pcolor.current-axes]
4. The axes view becomes `[0 90]`; FaceColor is `'flat'`. [pcolor.view,
   pcolor.face]
5. Mismatched x/y → `RunMat:pcolor:InputSizeMismatch`;
   `pcolor([1 2], [1 2; 3 4])` → `RunMat:badsubscript`.
   [pcolor.error-size, pcolor.error-two-args]

### User Story 5 - surfl draws a lit surface (P1)

1. `h = surfl(Z)`, `surfl(x, y, Z)` (vectors or matrices) → surface with
   full XData/YData grids, ZData Z, colour data of Z's size that is not Z.
   [surfl.output-class, surfl.data, surfl.cdata]
2. View `[-37.5 30]`; FaceColor `'flat'`. [surfl.view, surfl.face]
3. Z with 2 rows → `RunMat:surfl:InvalidInput` (3+ rows work); x length ≠
   columns → `RunMat:xyzchk:lengthXDoesNotMatchNumColumnsZ`;
   `surfl('abc')` → `RunMat:narginchk:notEnoughInputs`.
   [surfl.error-z-2x3, surfl.error-size, surfl.error-char]

### User Story 6 - unobserved forms fail clearly (P2)

Forms the exports do not cover raise `RunMat:<fn>:UnsupportedOption` or a
labelled documented-choice error.

## Requirements *(mandatory)*

### findall

- **FR-058-01** `findall(obj)` MUST return a column holding `obj` first and
  then every object under it [findall.signature-primary, findall.self].
  The search is depth-first pre-order over the `allchild` tree (FR-058-21)
  plus, for each axes, its title, subtitle and x/y/z labels whose text is
  non-empty. Legends and rulers are not searched (documented choice: RunMat
  enables a legend object on every axes with plot entries by default, and
  rulers carry no `FontSize`; neither appears in any observation).
- **FR-058-02** `'Type', t` MUST keep only objects whose `Type` equals `t`
  (case-insensitive) [findall.type].
- **FR-058-03** Axes title and label text objects MUST be found
  [findall.hidden]. RunMat has no `HandleVisibility`, so RunMat's `findobj`
  finds them too (deviation D-058-2; `findobj` is outside this feature).
- **FR-058-04** `'-property', name` MUST keep only objects for which
  `get(h, name)` succeeds [findall.property]. Rulers are not searched
  (documented choice consistent with the observation that only axes and
  text objects carry `FontSize` in the result).
- **FR-058-05** Setting `FontSize` 12 on the objects found by
  `findall(fig,'-property','FontSize')` MUST set the axes and text font
  sizes [findall.set-pattern]. RunMat's `set` accepts one handle at a time,
  so the one-call COBRA form `set(handles, ...)` is not available
  (deviation D-058-8; `set` is outside this feature).
- **FR-058-06** No match MUST give an empty result [findall.no-match]: a
  0x0 double (deviation D-058-3).
- **FR-058-07** A deleted figure handle MUST give the same empty result
  without error [findall.deleted]. RunMat treats any positive integer that
  does not name a live object as a deleted handle (deviation D-058-1).
- **FR-058-08** A number that is not a graphics handle MUST raise
  `RunMat:findall:InvalidHandles` [findall.error-not-handle]. Implemented
  for non-integer, negative and non-finite numbers and for non-numeric
  values; the observed `findall(123456)` returns empty (D-058-1).
- **FR-058-09** (documented choices) Several queries combine with AND. Any
  other query (`Name, Value` pairs other than `'Type'`, `'-depth'`,
  `'-regexp'`, …) raises `RunMat:findall:UnsupportedOption`. No argument
  raises `RunMat:minrhs`. A handle array is searched root by root, results
  concatenated. Order beyond the first element is RunMat's tree order
  (findall.q-order).

### ishghandle

- **FR-058-10** `ishghandle(h)` MUST return a logical of `h`'s size, true
  where `h` is a live graphics handle: figure, axes, line handles, the root
  `0` and `groot`, `double(f)` of a live figure, elementwise for arrays
  [ishghandle.live].
- **FR-058-11** `ishghandle(h, type)` MUST also require the object's Type
  to equal `type` [ishghandle.type].
- **FR-058-12** A deleted figure, a non-handle number, char, struct, a
  graph object MUST give false; a non-integer numeric matrix MUST give a
  false matrix of its size [ishghandle.false-cases].
- **FR-058-13** Empty input MUST give a 0x0 logical [ishghandle.empty].
- **FR-058-14** No input MUST raise `RunMat:minrhs` [ishghandle.error-none].
- **FR-058-15** (documented choices) Non-integer, negative and non-finite
  numbers are false (RunMat handles are non-negative integers); integer
  values are judged by RunMat's `isgraphics` predicate (any value naming a
  live object is true: ishghandle.q-numeric-handles). Every non-numeric class is false, shaped
  like the input for char, logical, string and cell arrays (scalar false
  otherwise). A non-text `type` raises `RunMat:ishghandle:InvalidInput`;
  more than two arguments raise `RunMat:ishghandle:UnsupportedOption`.

### allchild

- **FR-058-20** `allchild(0)` MUST return a column of the open figures,
  including invisible ones [allchild.root].
- **FR-058-21** `allchild(fig)` MUST include the figure's axes
  [allchild.figure] (RunMat also lists a figure-level `sgtitle` text when
  set). The observed annotation pane is not represented (D-058-4).
- **FR-058-22** `allchild(ax)` MUST return the plot objects of the axes and
  not its title [allchild.axes].
- **FR-058-23** For an object without children (a line) the result MUST be
  empty, 0x0 (D-058-3) [allchild.leaf].
- **FR-058-24** For an array of n > 1 handles the result MUST be an n-by-1
  cell, one child column per handle [allchild.array].
- **FR-058-25** A deleted handle, a number that is not a handle, or char
  MUST raise `RunMat:allchild:InvalidHandles` [allchild.error-deleted].
- **FR-058-26** (documented choices) No argument raises `RunMat:minrhs`;
  more than one raises `RunMat:allchild:UnsupportedOption`; an empty input
  gives 0x0. Order is RunMat's registration order (allchild.q-order).

### pcolor

- **FR-058-30** `pcolor` MUST create and return a surface object (`Type`
  `'surface'`; class: D-058-5) [pcolor.output-class].
- **FR-058-31** `pcolor(C)` for m-by-n C MUST give `XData = 1:n` (1-by-n),
  `YData = (1:m)'` (m-by-1), `ZData = zeros(m,n)` and colour data C
  [pcolor.c-only]. Colour data is checked on the surface's colour source
  (`get(h,'CData')` is not available: D-058-6).
- **FR-058-32** `pcolor(X, Y, C)` MUST keep vector X/Y with their given
  shapes and matrix X/Y as given; `ZData` zeros the size of C; colour C
  [pcolor.xyc].
- **FR-058-33** `pcolor(ax, C)` MUST draw into `ax` (`Parent == ax`)
  [pcolor.ax-arg]; otherwise into the current axes [pcolor.current-axes].
- **FR-058-34** The axes view MUST be `[0 90]` afterwards [pcolor.view].
- **FR-058-35** `FaceColor` MUST be `'flat'` [pcolor.face].
- **FR-058-36** X or Y not matching C MUST raise
  `RunMat:pcolor:InputSizeMismatch` [pcolor.error-size].
- **FR-058-37** Two numeric arguments whose first is not an axes MUST raise
  `RunMat:badsubscript` [pcolor.error-two-args]. The generic identifier is
  reproduced as observed; whether it must be is open
  (pcolor.q-two-arg-id, returned to the human reviewer).
- **FR-058-38** (documented choices) Edges are drawn; RunMat's surface
  model reports `EdgeColor` `'flat'` (the observed dark-grey RGB is
  non-normative: pcolor.edgecolor). Colours use RunMat's default
  `parula` colormap. The view is set even with `hold on`. Text or
  name-value arguments, more than 3 data arguments, complex C, and C with
  fewer than 2 rows or columns raise `RunMat:pcolor:UnsupportedOption`.
  Missing, empty, N-D or non-numeric inputs raise
  `RunMat:pcolor:InvalidInput`. A Y-length or matrix-shape mismatch uses
  `InputSizeMismatch`.

### surfl

- **FR-058-40** `surfl(Z)` and `surfl(X, Y, Z)` MUST create and return one
  surface object (`Type` `'surface'`; class: D-058-5) [surfl.output-class].
- **FR-058-41** `ZData` MUST be Z; `XData`/`YData` MUST be full m-by-n
  grids (from `1:n`, `1:m` for `surfl(Z)`, expanded from vectors, or the
  given matrices); the colour data MUST have Z's size [surfl.data].
- **FR-058-42** The colour data MUST NOT be Z [surfl.cdata]. RunMat
  computes an independently chosen reflectance map in [0, 1] (FR-058-46);
  the observed numeric range for MATLAB's `peaks(10)` is not reproduced
  (D-058-7, surfl.q-cdata).
- **FR-058-43** The axes view MUST be `[-37.5 30]` [surfl.view];
  `FaceColor` MUST be `'flat'` [surfl.face].
- **FR-058-44** Z with fewer than 3 rows MUST raise
  `RunMat:surfl:InvalidInput`; 3 and 4 rows MUST work
  [surfl.error-z-2x3]. Vector X whose length differs from the columns of Z
  MUST raise `RunMat:xyzchk:lengthXDoesNotMatchNumColumnsZ`
  [surfl.error-size]. A single char argument MUST raise
  `RunMat:narginchk:notEnoughInputs` [surfl.error-char]. Identifiers that
  name other functions are reproduced as observed (surfl.q-error-ids,
  returned to the human reviewer).
- **FR-058-45** (documented choices) Z with fewer than 3 columns, two or
  more than three arguments, and complex input raise
  `RunMat:surfl:UnsupportedOption` (surfl.q-min-size). No argument or a
  single non-numeric argument raises `RunMat:narginchk:notEnoughInputs`
  (surfl.q-char-error). A Y-length mismatch raises
  `RunMat:xyzchk:lengthYDoesNotMatchNumRowsZ` (by analogy, unobserved); a
  matrix X/Y not the size of Z raises `RunMat:surfl:InputSizeMismatch`.
  Edges are drawn (EdgeColor `'flat'`; surfl.edgecolor non-normative).
- **FR-058-46** (independent algorithm) Reflectance: coordinates scaled to
  unit range per axis; normals from central differences facing the viewer;
  light at the viewing direction offset by +45° azimuth and +15°
  elevation; value `0.25 + 0.6·max(0,n·l) + 0.15·max(0,n·h)^8` clamped to
  [0, 1]; colours map it through `parula` with automatic limits. This is a
  textbook Lambert/Blinn-Phong model chosen for RunMat.

## Success Criteria

- **SC-058-1** Every normative claim of the five exports maps to an FR and
  a passing test, or to a listed deviation with a reason.
- **SC-058-2** The surrounding `builtins::plotting` suite shows no new
  failures.
- **SC-058-3** `validation.md` records the failing (stub) run, the passing
  run, clippy and rustfmt results, and the freshness checks.

## Deviations (summary; details in `validation.md`)

- D-058-1 deleted vs never-valid numeric handles (findall).
- D-058-2 `findobj` also finds label text (no `HandleVisibility`).
- D-058-3 empty handle arrays are 0x0 double, not `GraphicsPlaceholder`.
- D-058-4 no annotation pane among a figure's children.
- D-058-5 surface class names are not representable (numeric handles,
  `Type` `'surface'` for both pcolor and surfl).
- D-058-6 no `CData` property on RunMat surfaces.
- D-058-7 surfl colour values come from RunMat's own reflectance model.
- D-058-8 RunMat's `set` does not accept a handle array.
- D-058-9 RunMat's `plot` sets default axis labels 'X'/'Y' (test scenes
  blank them).
