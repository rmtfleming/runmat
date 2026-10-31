# Implementation Plan — 065-graphics-n5 (`fplot`, `rectangle`, `validatecolor` 0.2.0)

## Constitution Check

- I Specification-first: FR-065-01..08, 20..26, 30..35 cite normative 0.2.0
  claims; FR-065-09 implements a non-normative observation as observed;
  FR-065-10, 27, 36 are tagged documented choices. PASS.
- II Clean-room: inputs are the imported exports plus RunMat code only;
  MATLAB not invoked. PASS.
- III Independence: `fplot` samples a fixed grid and reuses RunMat's
  `plot` line construction (`build_line_plot_for_builtin`), line-style
  parser and colour order; `rectangle` draws a RunMat `PatchPlot` whose
  rounded corners are an independently chosen quarter-ellipse geometry;
  `validatecolor` reuses RunMat's plotting colour table
  (`style::color_from_name_or_token`). PASS.
- IV Provenance: `provenance.md` pinned to `1c91aa1`; freshness checked at
  start and finish. PASS.
- V Test-first: tests written and run against stub builtins before the
  implementation was enabled (validation.md). PASS.
- VII Minimal scope: three new files and three module lines in the
  hand-written `ops/mod.rs`. No upstream file is edited (gaps are
  deviations). PASS.
- IX Compatibility: `#[runtime_builtin]` with descriptors and error
  descriptors; `fplot`/`rectangle` are sinks with `suppress_auto_output`;
  no integer-capability metadata. PASS.

## Affected files

| Item | Path |
|---|---|
| `fplot` | `crates/runmat-runtime/src/builtins/plotting/ops/fplot.rs` |
| `rectangle` | `crates/runmat-runtime/src/builtins/plotting/ops/rectangle.rs` |
| `validatecolor` + shared single-colour parser (`pub(crate) parse_single_color`) | `crates/runmat-runtime/src/builtins/plotting/ops/validatecolor.rs` |
| Registration | `#[path]` lines in the hand-written `crates/runmat-runtime/src/builtins/plotting/ops/mod.rs` |

Reused without modification: `plot::build_line_plot_for_builtin`
(`pub(super)`, visible to `plotting::ops`), `style::parse_line_style_args`,
`style::color_from_name_or_token` (already `pub(crate)`),
`state::{render_active_plot, append_active_plot, register_line_handle,
register_patch_handle, set_axis_limits_for_axes, axis_limits_snapshot,
plot_child_handles_for_axes, line_color_for_axes_series_index,
next_line_color_for_axes}`, `warnings::emit`,
`interaction::current_eval_hook`. No visibility change was needed.

## Design

### fplot

1. Gather. If ≥ 2 arguments and the first is an axes handle → target.
2. f: function value → callable; char/string → emit
   `RunMat:fplot:StringFunctionsDeprecated`, then evaluate `@(x) <expr>`
   with the host eval hook (none → `UnsupportedOption`); real scalar →
   constant; empty → `InvalidInput`; else `UnsupportedOption`.
3. Second argument: function → `UnsupportedOption` (parametric); numeric →
   interval (2 elements, finite, increasing else `RangeVector`; other
   counts `UnsupportedOption`).
4. Style arguments: names must be in the accepted list (else
   `InvalidParameter`); the first may be a LineSpec. Parse with
   `parse_line_style_args(generic("fplot"))` (errors → `InvalidInput`).
   Defaults: LineWidth 0.5, LineStyle `-`, colour from the axes colour
   order (`next_line_color_for_axes` with hold, series-0 colour without).
5. x = 201 points (`lo + (hi-lo)·i/200`, last = `hi`); y = vectorised call
   or per-point fallback.
6. Select axes; `render_active_plot` (clears unless hold), add the line,
   `register_line_handle`; set x-limits to the interval.

### rectangle

1. Gather; split a leading axes handle. Parse name-value pairs
   (case-insensitive; unknown → `RunMat:hg:InvalidProperty`).
2. Validate Position (4 elements → else `Expected4ElementVector`; w,h ≥ 0
   and finite → else `DimensionsOutsideRange`), Curvature (1–2 elements in
   [0, 1] → else `CurvatureRange`/`InvalidInput`), colours via
   `validatecolor::parse_single_color` or `'none'`, LineWidth, LineStyle.
3. Outline polygon (rounded corners), `PatchPlot` with face/edge modes,
   colours and width; `append_active_plot` (no clearing);
   `register_patch_handle`.
4. Limits: empty axes in auto mode → rectangle extent; set limits → grown
   to include it; auto limits with other content → unchanged.
5. Return a `HandleObject` of class `matlab.graphics.primitive.Rectangle`
   with properties Type, Parent, Position, Curvature, FaceColor,
   EdgeColor, LineWidth, LineStyle.

### validatecolor

1. 0 args → `RunMat:minrhs`; > 2 → `RunMat:maxrhs`; option must be text
   `one`/`multiple` (case-insensitive) else
   `RunMat:unrecognizedStringChoice`.
2. Cell / string array → one colour per element; numeric → triplet or (with
   `multiple`) m-by-3 rows; m-by-3 with m > 1 without `multiple` →
   `MultipleColors`.
3. Text → `color_from_name_or_token` (+ trim), components recovered as
   exact `k/255`; unknown → `InvalidColorString`. Float components outside
   [0, 1] or NaN → `OutOfRange`; uint8 /255, uint16 /65535; other classes
   → `RunMat:validatecolor:UnsupportedOption`; wrong element count →
   `InvalidTripletShape`.
4. Return an m-by-3 double.

## Test plan

- `fplot_tests`: `observed_default_line`, `observed_default_xlim`,
  `observed_interval_and_values`, `observed_cobra_name_value_form`,
  `observed_cobra_form_without_output`, `observed_linespec`,
  `observed_axes_argument`, `observed_char_function_warns_and_draws`,
  `observed_numeric_constant`, `observed_decreasing_interval_errors`,
  `observed_bad_property_errors`, `unresolved_choice_sampling_and_arguments`,
  `unresolved_choice_evaluation_failure`, `unresolved_choice_hold_keeps_lines`,
  `deviation_char_function_without_evaluator`.
- `rectangle_tests`: `observed_default_object`, `observed_cobra_form`,
  `observed_cobra_limits_in_fresh_axes`, `observed_curvature_stored_as_given`,
  `observed_face_color_name_stored_as_rgb`,
  `observed_no_output_adds_one_object`, `observed_axes_argument`,
  `observed_negative_width_errors`, `observed_position_length_errors`,
  `observed_curvature_range_errors`, `observed_bad_property_errors`,
  `unresolved_choice_invalid_values`, `unresolved_choice_curved_outline`,
  `unresolved_choice_limits_grow_and_objects_kept`,
  `deviation_get_on_rectangle_object` (#[ignore], D-065-4),
  `deviation_axes_child_is_rectangle` (#[ignore], D-065-5),
  `registered_through_dispatcher`.
- `validatecolor_tests`: `observed_short_name_and_cobra_any`,
  `observed_names`, `observed_triplets`, `observed_hex_codes`,
  `observed_multiple_and_one`, `observed_matrix_without_multiple_errors`,
  `observed_unknown_name_errors`, `observed_out_of_range_errors`,
  `observed_triplet_shape_errors`, `observed_bad_option_errors`,
  `unresolved_choice_unobserved_forms`, `registered_through_dispatcher`.

Rendering is disabled in tests (`ensure_plot_test_env`); tests assert
object properties and stored data, not pixels. Function handles in tests
are served by a test semantic-function invoker; the char-expression test
installs a test eval hook.

## Verification commands

`cargo test -p runmat-runtime --lib -- fplot_tests rectangle_tests validatecolor_tests` ·
`cargo test -p runmat-runtime --lib -- builtins::plotting` ·
`rustfmt --edition 2021 --check <files>` ·
`cargo clippy -p runmat-runtime --lib --tests`
