# Validation Report — 065-graphics-n5 (`fplot`, `rectangle`, `validatecolor` 0.2.0) — 2026-10-31

Environment: Linux x86_64; `HDF5_DIR=~/anaconda3`; test binaries run
through the shared cargo runner wrapper. The target directory was shared
with the other N5 agents (builds waited on the build lock). MATLAB was not
invoked.

## Freshness (T0 and finish)

In `../matlab-interface-spec` after `git fetch -q origin`: `HEAD` =
`origin/main` = `1c91aa1` (the pin), at start and at finish.
`git diff --stat 1c91aa1 origin/main --
specifications/{fplot,rectangle,validatecolor}` is empty and `git status
--short` shows nothing for these three functions. The 12 imported files
match `SHA256SUMS` and the spec repository byte for byte. The import is
current.

## Upstream behaviour recorded before / during implementation (T1)

- Axis-limit queries in automatic mode return `[NaN NaN]` (RunMat does not
  derive limits from data on query), so `fplot` and `rectangle` set limits
  explicitly (FR-065-10, FR-065-27).
- `get(line, 'Color')` reports a colour name (`'r'`) for the eight named
  colours (`color_to_short_name` in `core/properties.rs`): D-065-7.
- `get(axes, 'Children')` lists the title, subtitle and label objects, not
  plot objects; `allchild(ax)` lists plot objects (spec 058): D-065-8.
- `Tensor::try_from(Value::Num)` fails; scalar inputs are converted
  explicitly.
- `PatchPlot::new` rejects a face without area (zero width or height):
  such rectangles are drawn as an outline line.
- Numeric graphics handles support dot access through `get`; handle
  objects store dot-access properties (tiledlayout precedent, D-051-7).

## Test-first evidence (T4)

The three modules were registered and each builtin body began with an
early `return Err("F65-STUB")` (marker `F65-STUB`, since removed):
`cargo test -p runmat-runtime --lib -- fplot_tests rectangle_tests validatecolor_tests`
→ **1 passed, 41 failed, 2 ignored**. The pass is
`rectangle_tests::unresolved_choice_curved_outline`, which calls the
geometry helper directly; every other test failed on the stub.

## Commands and results (T8)

- First run with the implementation: 30 passed, 12 failed. Fixes: scalar
  `Num`/`Int` conversion in all three files; tests read line colours
  through `validatecolor` (D-065-7) and plot objects through `allchild`
  (D-065-8); zero-area rectangles drawn as an outline line. One further
  run: 41 passed, 1 failed (the zero-area patch), fixed as above.
- Final: `cargo test -p runmat-runtime --lib -- fplot_tests rectangle_tests validatecolor_tests`
  → **42 passed, 0 failed, 3 ignored** (`fplot_tests` 15 + 1 ignored,
  `rectangle_tests` 15 + 2 ignored, `validatecolor_tests` 12). The ignored
  tests are the deviation tests `deviation_color_reported_as_rgb_row`
  (D-065-7), `deviation_get_on_rectangle_object` (D-065-4) and
  `deviation_axes_child_is_rectangle` (D-065-5).
- Surrounding suite `cargo test -p runmat-runtime --lib -- builtins::plotting`
  → **1188 passed, 1 failed, 5 ignored**. The failure is the known
  pre-existing
  `contourf::tests::contourf_accepts_target_axes_and_does_not_alias_integer_z_as_a_handle`
  (axes-handle form under parallel runs), unrelated; the 5 ignored are this
  feature's 3 deviation tests and spec 058's 2.
- `cargo clippy -p runmat-runtime --lib --tests`: **no findings in this
  feature's files** (one `type_complexity` lint in `rectangle.rs` was fixed
  and the feature tests re-run, 42/0/3). The command still exits 101
  because of `cloned_ref_to_slice_refs` lints in other agents' in-progress
  files (`graph/domain/reordernodes.rs`, `math/rounding/rat.rs`).
- `rustfmt --edition 2021 --check` on the three new files: clean.
  `ops/mod.rs` (hand-written) only gained three `#[path]` module
  declarations. `cargo fmt --all` was not run.

## Traceability (claim → FR → test → result) (T9)

| Claim (0.2.0) | FR | Test(s) | Result |
|---|---|---|---|
| fplot.signature-primary, fplot.object | FR-065-01 | `observed_default_line` | PASS (`Type` `'line'`, D-065-1) |
| fplot.default-interval | FR-065-02 | `observed_default_line`, `observed_default_xlim` | PASS |
| fplot.interval | FR-065-03 | `observed_interval_and_values` | PASS (max error 0) |
| fplot.cobra-form | FR-065-04 | `observed_cobra_name_value_form`, `observed_cobra_form_without_output` | PASS (Color via validatecolor, D-065-7; children via allchild, D-065-8) |
| fplot.linespec | FR-065-05 | `observed_linespec`, `observed_default_line` | PASS |
| fplot.ax | FR-065-06 | `observed_axes_argument` | PASS |
| fplot.warning-char-fn | FR-065-07 | `observed_char_function_warns_and_draws`, `deviation_char_function_without_evaluator` | PASS (test eval hook, D-065-3; `lastwarn` two-output form checked) |
| fplot.error-decreasing | FR-065-08 | `observed_decreasing_interval_errors` | PASS |
| fplot.error-bad-property | FR-065-08 | `observed_bad_property_errors` | PASS |
| fplot.numeric (non-normative) | FR-065-09 | `observed_numeric_constant` | PASS |
| fplot.default-color (non-normative) | FR-065-10 | — | not asserted (theme choice) |
| (choices) | FR-065-10 | `unresolved_choice_sampling_and_arguments`, `unresolved_choice_evaluation_failure`, `unresolved_choice_hold_keeps_lines` | PASS |
| rectangle.signature-primary | FR-065-20 | `observed_default_object`, `registered_through_dispatcher` | PASS (handle object, D-065-4) |
| rectangle.defaults | FR-065-21 | `observed_default_object` | PASS |
| rectangle.cobra-form | FR-065-22 | `observed_cobra_form`, `observed_cobra_limits_in_fresh_axes` | PASS |
| rectangle.curvature | FR-065-23 | `observed_curvature_stored_as_given` | PASS |
| rectangle.colors | FR-065-24 | `observed_face_color_name_stored_as_rgb` | PASS |
| rectangle.parent | FR-065-25 | `observed_no_output_adds_one_object`, `observed_axes_argument` | PASS (child is the drawing patch, D-065-5) |
| rectangle.error-negative-width | FR-065-26 | `observed_negative_width_errors` | PASS |
| rectangle.error-position-length | FR-065-26 | `observed_position_length_errors` | PASS |
| rectangle.error-curvature-range | FR-065-26 | `observed_curvature_range_errors` | PASS |
| rectangle.error-bad-property | FR-065-26 | `observed_bad_property_errors` | PASS |
| rectangle.default-edge-color (non-normative) | FR-065-21 | — | not asserted (choice `[0 0 0]`) |
| (choices) | FR-065-27 | `unresolved_choice_invalid_values`, `unresolved_choice_curved_outline`, `unresolved_choice_limits_grow_and_objects_kept` | PASS |
| validatecolor.signature-primary, validatecolor.cobra-form | FR-065-30 | `observed_short_name_and_cobra_any`, `registered_through_dispatcher` | PASS |
| validatecolor.names | FR-065-31 | `observed_names` | PASS |
| validatecolor.triplets | FR-065-32 | `observed_triplets` | PASS (exact) |
| validatecolor.hex | FR-065-33 | `observed_hex_codes` | PASS (exact k/255) |
| validatecolor.multiple | FR-065-34 | `observed_multiple_and_one` | PASS |
| validatecolor.error-matrix-without-multiple | FR-065-35 | `observed_matrix_without_multiple_errors` | PASS |
| validatecolor.error-unknown-name | FR-065-35 | `observed_unknown_name_errors` | PASS |
| validatecolor.error-out-of-range | FR-065-35 | `observed_out_of_range_errors` | PASS |
| validatecolor.error-two-components | FR-065-35 | `observed_triplet_shape_errors` | PASS |
| validatecolor.error-bad-option | FR-065-35 | `observed_bad_option_errors` | PASS |
| (choices) | FR-065-36 | `unresolved_choice_unobserved_forms` | PASS |

## Deviations

- **D-065-1: FunctionLine class.** `fplot` returns a numeric line handle
  (`Type` `'line'`); RunMat has no FunctionLine object kind.
- **D-065-2: sampling.** 201 fixed points, evaluated once; no adaptive
  refinement and no re-evaluation on zoom or interval change.
- **D-065-3: char expressions.** Evaluated as `@(x) <expr>` through the
  host expression evaluator (`interaction::current_eval_hook`); the
  independent variable is always `x`. Without an evaluator (e.g. a bare
  runtime) the warning is issued and `RunMat:fplot:UnsupportedOption` is
  raised. Not verified end to end through the VM (tests use a test hook).
- **D-065-4: Rectangle object.** A handle object of class
  `matlab.graphics.primitive.Rectangle` read with dot access; `get`/`set`
  on it are not wired (needs upstream `get.rs`/`set.rs` changes), and
  changing a property after creation does not redraw.
- **D-065-5: axes children.** The rectangle's drawing (a patch, or an
  outline line for zero-area rectangles) is the axes' child, not the
  Rectangle object, so the observed `class(c)`/`c.Position` of the child
  are not reproduced.
- **D-065-6: LineStyle rendering.** Stored; patch edges render solid
  (`PatchPlot` has no line style). Curvature is rendered (rounded
  polygon).
- **D-065-7: line Color property.** RunMat's `get(line,'Color')` reports
  `'r'` for pure red (upstream `color_to_short_name`), not `[1 0 0]`; the
  stored colour is `[1 0 0]`, checked through `validatecolor`.
- **D-065-8: axes `Children`.** RunMat's axes `Children` lists the title,
  subtitle and label objects; plot objects are listed by `allchild`.

## Questions returned to the human reviewer

- `rectangle.q-error-ids`: `RunMat:hg:…` identifiers reproduced as observed.
- `validatecolor.q-error-ids`: `RunMat:unrecognizedStringChoice` reproduced.
- `fplot.q-theme`/`rectangle.q-theme`: LineWidth 0.5 asserted (stated
  normatively); colours are documented choices.

## Not verified headlessly

- Rendered appearance (curved corners, fills, line widths) — tests run with
  rendering disabled and assert stored properties and plot data only.
- No end-to-end run through the MATLAB-syntax VM; dot access on the
  Rectangle object was checked through `object::resolve::load_member`.
