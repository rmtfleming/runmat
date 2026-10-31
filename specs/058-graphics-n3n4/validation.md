# Validation Report — 058-graphics-n3n4 (`findall`, `ishghandle`, `allchild`, `pcolor`, `surfl` 0.2.0) — 2026-10-30

Environment: Linux x86_64; `HDF5_DIR=~/anaconda3`; test binaries run
through the shared cargo runner wrapper. The target directory was shared
with other N3/N4 agents; several builds waited on the build lock or were
blocked by other agents' in-progress files (for example
`strings/core/as_many_of_pattern.rs`) and were retried unchanged. MATLAB was
not invoked.

## Freshness (T0 and finish)

In `../matlab-interface-spec` after `git fetch -q origin`: `HEAD` =
`origin/main` = `c3305a5` (tools-only commit after the `fc9f804` N4 pin).
`git diff --stat 1156c97 origin/main -- specifications/{findall,ishghandle}`
and `git diff --stat fc9f804 origin/main --
specifications/{allchild,pcolor,surfl}` are empty; the spec repo's
untracked drafts are under `experiments/` for other functions only. The
20 imported files match `SHA256SUMS` and the spec repository byte for byte.
At finish: `HEAD` = `origin/main` = `1c91aa1` (batch N5, new specs for
other functions). The same two `git diff --stat` commands against the pins
are still empty and `git status --short` shows nothing for these five
directories: no newer or draft version exists. The import is current.

## Upstream behaviour recorded before implementation (T1)

A throwaway probe test (removed) showed: graphics handles are integers
(figure `2`, axes `2097152`, plot objects `1099511627776+`, label objects
`33554433+`); `get(h,'FontSize')` succeeds on axes, text, title/label text
and legend objects and fails on figures, lines and rulers; `set` rejects a
handle array ("expected plotting handle"); RunMat's `plot` sets default
axis labels `'X'` and `'Y'`; every axes with plot entries has a legend
object (`legend_enabled` defaults to true); `resolve_plot_handle(1.5)`
succeeds (non-integers are rounded); `groot` returns `0`.

## Test-first evidence (T4)

The five modules were registered and every builtin body began with an
early `return Err("stub")` (marker `F58-STUB`, since removed):
`cargo test -p runmat-runtime --lib -- findall_tests ishghandle_tests allchild_tests pcolor_tests surfl_tests`
→ **2 passed, 45 failed, 2 ignored**. The two passes are
`surfl_tests::observed_cdata_is_not_z` and
`surfl_tests::unresolved_choice_flat_surface_uniform_reflectance`, which
call the reflectance helper directly rather than the builtin. Every other
test failed on the stub.

## Commands and results (T10)

- First run with the implementation: 43 passed, 4 failed. Fixes:
  `resolve_live` and `ishghandle` now treat non-integer numbers as
  non-handles (RunMat's resolver rounds them); label objects count only
  with non-empty text, and the findall test scene blanks RunMat's default
  `'X'`/`'Y'` labels (D-058-9); `pcolor`/`surfl` no longer pass default
  axis labels to the renderer.
- Final: `cargo test -p runmat-runtime --lib -- findall_tests ishghandle_tests allchild_tests pcolor_tests surfl_tests`
  → **47 passed, 0 failed, 2 ignored**. The ignored tests are the deviation tests
  `deviation_integer_non_handle_errors` (D-058-1) and
  `deviation_set_on_handle_array` (D-058-8).
- Surrounding suite `cargo test -p runmat-runtime --lib -- builtins::plotting`
  → **1145 passed, 2 failed, 2 ignored**. The failures are the known
  pre-existing `contour::tests::contour_accepts_leading_axes_handle_but_not_typed_integer_handle_aliases`
  and `contourf::tests::contourf_accepts_target_axes_and_does_not_alias_integer_z_as_a_handle`
  (axes-handle forms), unrelated to this feature; the 2 ignored are this
  feature's deviation tests.
- `cargo clippy -p runmat-runtime --lib --tests`: **no findings in this feature's files**
  (lints found on the way — `manual_is_multiple_of`,
  `unnecessary_to_owned`, `cloned_ref_to_slice_refs` — were fixed and the
  feature tests re-run, 47/0/2). The command still exits 101 because of
  lints in other agents' in-progress files (`graph/domain/isomorphism.rs`,
  `graph/domain/paths.rs`, `io/repl_fs/environment/release.rs`,
  `table/builtins/convertvars.rs`).
- `rustfmt --edition 2021 --check` on the five new files: clean.
  `ops/mod.rs` only gained five `#[path]` module declarations. `cargo fmt --all` was not run.

Per module (final run): `findall_tests` 10 + 2 ignored, `ishghandle_tests`
7, `allchild_tests` 8, `pcolor_tests` 11, `surfl_tests` 11.

## Traceability (claim → FR → test → result) (T11)

| Claim (0.2.0) | FR | Test(s) | Result |
|---|---|---|---|
| findall.signature-primary, findall.self | FR-058-01 | `observed_includes_the_object_itself_first` | PASS |
| findall.type | FR-058-02 | `observed_type_filter` | PASS |
| findall.hidden | FR-058-03 | `observed_finds_title_and_label_text` | PASS (scene: D-058-9; findobj: D-058-2) |
| findall.property | FR-058-04 | `observed_property_filter` | PASS |
| findall.set-pattern | FR-058-05 | `observed_set_pattern`; `deviation_set_on_handle_array` | PASS per handle; array form ignored (D-058-8) |
| findall.no-match | FR-058-06 | `observed_no_match_is_empty` | PASS (0x0 double, D-058-3) |
| findall.deleted | FR-058-07 | `observed_deleted_figure_is_empty` | PASS |
| findall.error-not-handle | FR-058-08 | `observed_non_handle_number_errors`; `deviation_integer_non_handle_errors` | PASS for 1.5, -2, NaN; `123456` ignored (D-058-1) |
| (choices) | FR-058-09 | `unresolved_choice_queries_and_inputs`, `registered_through_dispatcher` | PASS |
| ishghandle.signature-primary, ishghandle.live | FR-058-10 | `observed_live_handles_are_true` | PASS |
| ishghandle.type | FR-058-11 | `observed_type_argument` | PASS |
| ishghandle.false-cases | FR-058-12 | `observed_false_cases` | PASS |
| ishghandle.empty | FR-058-13 | `observed_empty_gives_0x0_logical` | PASS |
| ishghandle.error-none | FR-058-14 | `observed_no_input_errors_minrhs` | PASS |
| (choices) | FR-058-15 | `unresolved_choice_text_and_argument_handling`, `registered_through_dispatcher` | PASS |
| allchild.root | FR-058-20 | `observed_root_lists_new_invisible_figure` | PASS |
| allchild.figure | FR-058-21 | `observed_figure_children_include_axes` | PASS (no annotation pane, D-058-4) |
| allchild.axes | FR-058-22 | `observed_axes_children_are_plot_objects` | PASS |
| allchild.leaf | FR-058-23 | `observed_leaf_has_no_children` | PASS (0x0, D-058-3) |
| allchild.array | FR-058-24 | `observed_array_input_gives_cell_column` | PASS |
| allchild.error-deleted | FR-058-25 | `observed_invalid_handles_error` | PASS |
| (choices) | FR-058-26 | `unresolved_choice_argument_handling`, `registered_through_dispatcher` | PASS |
| pcolor.output-class | FR-058-30 | `observed_c_only_data`, `observed_vector_coordinates_kept_as_given`, `observed_draws_into_current_axes`, `registered_through_dispatcher` | PASS (`Type` `'surface'`, D-058-5) |
| pcolor.c-only | FR-058-31 | `observed_c_only_data` | PASS (colour source checked, D-058-6) |
| pcolor.xyc | FR-058-32 | `observed_vector_coordinates_kept_as_given`, `observed_matrix_coordinates_kept_as_given` | PASS |
| pcolor.ax-arg, pcolor.current-axes | FR-058-33 | `observed_axes_argument`, `observed_draws_into_current_axes` | PASS |
| pcolor.view | FR-058-34 | `observed_view_from_above` | PASS |
| pcolor.face (+ edgecolor choice) | FR-058-35, 38 | `observed_face_color_flat_and_edge_choice` | PASS |
| pcolor.error-size | FR-058-36 | `observed_size_mismatch_errors` | PASS |
| pcolor.error-two-args | FR-058-37 | `observed_two_numeric_arguments_error` | PASS |
| (choices) | FR-058-38 | `unresolved_choice_unobserved_forms` | PASS |
| surfl.output-class, surfl.data | FR-058-40, 41 | `observed_z_only`, `observed_vector_and_matrix_coordinates_expand_to_grids`, `registered_through_dispatcher` | PASS (`Type` `'surface'`, D-058-5) |
| surfl.cdata | FR-058-42 | `observed_cdata_is_not_z` (+ colour-grid size in `observed_z_only`) | PASS; numeric range not reproduced (D-058-7) |
| surfl.view, surfl.face | FR-058-43 | `observed_view`, `observed_face_color_flat_and_edge_choice` | PASS |
| surfl.error-z-2x3, error-size, error-char | FR-058-44 | `observed_two_row_z_errors`, `observed_x_length_mismatch_errors`, `observed_single_char_argument_errors` | PASS |
| (choices) | FR-058-45 | `unresolved_choice_unobserved_forms` | PASS |
| (independent model) | FR-058-46 | `observed_cdata_is_not_z`, `unresolved_choice_flat_surface_uniform_reflectance` | PASS |

Every FR has at least one test, and every test maps to an FR or deviation.

## Deviations

- **D-058-1: deleted vs never-valid numeric handles.** RunMat handles are
  plain numbers with no deleted state, so `findall` cannot tell a closed
  figure from a number that was never a handle. Positive integers that do
  not name a live object are treated as deleted handles (empty result,
  satisfying `findall.deleted`); `findall(123456)` therefore returns empty
  instead of raising `InvalidHandles`. Non-integer, negative and
  non-finite numbers and non-numeric values do raise it. `allchild` (whose
  deleted and non-handle cases share one error) and `ishghandle` are
  unaffected. Fixing it needs a deleted-handle record in the plotting
  registry.
- **D-058-2: `findobj` finds label text.** RunMat has no
  `HandleVisibility`; `findobj` (upstream, outside this feature) already
  returns title and label objects. The `findall` requirement is met; the
  observed `findobj` count of 0 is not reproduced.
- **D-058-3: empty results.** Empty handle results are 0x0 `double`, the
  representation of RunMat's `findobj`/`gobjects`, not
  `matlab.graphics.GraphicsPlaceholder`.
- **D-058-4: no annotation pane.** `allchild(fig)` lists the axes (and an
  `sgtitle` text when set); RunMat has no annotation-pane object.
- **D-058-5: surface class names.** Handles are numeric; both `pcolor`
  (`matlab.graphics.primitive.Surface`) and `surfl`
  (`matlab.graphics.chart.primitive.Surface`) return objects of `Type`
  `'surface'`.
- **D-058-6: no `CData` property.** RunMat's surface `get` has no `CData`.
  `pcolor`'s colour source C (drawn at zero height through `flatten_z`)
  and `surfl`'s per-vertex colour grid are checked on the stored
  `SurfacePlot`. Exposing `CData` needs a change to `plotting/core/
  properties.rs` (`get_surface_property`), outside this feature.
- **D-058-7: surfl colour values.** How MATLAB computes the colours is not
  established (surfl.q-cdata); RunMat uses its own reflectance model
  (FR-058-46). The observed range 0.3548–0.7787 for `peaks(10)` is not
  reproduced or targeted.
- **D-058-8: `set` on handle arrays.** RunMat's `set` accepts one handle,
  so `set(findall(...), 'FontSize', 12)` must loop. Needs an upstream
  `set` change.
- **D-058-9: default axis labels.** RunMat's `plot` sets axis labels `'X'`
  and `'Y'`, which are then text objects that `findall` finds. The
  findall tests blank them to reproduce the observed scene; `pcolor` and
  `surfl` do not set default labels.

## Flagged for human review

- `pcolor.q-two-arg-id` and `surfl.q-error-ids`: the generic or
  foreign-function identifiers `RunMat:badsubscript`,
  `RunMat:xyzchk:lengthXDoesNotMatchNumColumnsZ` and
  `RunMat:narginchk:notEnoughInputs` are reproduced as observed.

## Not verified headlessly

- Rendered appearance (flat zero-height pcolor surface, surfl shading,
  edges) is not checked: tests run with rendering disabled and assert
  object types, properties and stored plot data.
- No end-to-end run through the MATLAB-syntax VM; the dispatcher path is
  exercised by `registered_through_dispatcher` in each module.
