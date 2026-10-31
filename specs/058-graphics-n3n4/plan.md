# Implementation Plan — 058-graphics-n3n4 (`findall`, `ishghandle`, `allchild`, `pcolor`, `surfl` 0.2.0)

## Constitution Check

- I Specification-first: FR-058-01..08, 10..14, 20..25, 30..37, 40..44 cite
  normative 0.2.0 claims; FR-058-09, 15, 26, 38, 45, 46 are tagged
  documented choice or independent algorithm. PASS.
- II Clean-room: inputs are the imported exports plus RunMat code only;
  MATLAB not invoked. PASS.
- III Independence: the object tree is RunMat's own registry
  (`root_figure_handles`, `axes_handles_for_figure`,
  `plot_child_handles_for_axes`, axes metadata); `ishghandle` reuses
  RunMat's `isgraphics`; `pcolor`/`surfl` build RunMat `SurfacePlot`s;
  surfl colours use a textbook Lambert/Blinn-Phong model. PASS.
- IV Provenance: `provenance.md` pinned to `1156c97` (N3) and `fc9f804`
  (N4); freshness checked at start and finish. PASS.
- V Test-first: tests written and run against stub builtins before the
  implementation was enabled (validation.md). PASS.
- VII Minimal scope: five new files and five module lines in the
  hand-written `ops/mod.rs`. No upstream file is edited (gaps are
  deviations). PASS.
- IX Compatibility: `#[runtime_builtin]` with descriptors and error
  descriptors; plotting sinks use `sink = true`, `suppress_auto_output`;
  no integer-capability metadata. PASS.

## Affected files

| Item | Path |
|---|---|
| `findall` + shared object tree | `crates/runmat-runtime/src/builtins/plotting/ops/findall.rs` |
| `ishghandle` | `crates/runmat-runtime/src/builtins/plotting/ops/ishghandle.rs` |
| `allchild` | `crates/runmat-runtime/src/builtins/plotting/ops/allchild.rs` |
| `pcolor` + shared surface rendering helper | `crates/runmat-runtime/src/builtins/plotting/ops/pcolor.rs` |
| `surfl` + reflectance model | `crates/runmat-runtime/src/builtins/plotting/ops/surfl.rs` |
| Registration | `#[path]` lines in the hand-written `crates/runmat-runtime/src/builtins/plotting/ops/mod.rs` |

`plotting/mod.rs` is `@generated` and is not edited.

## Design

### Object tree (findall.rs, shared with allchild)

- `graphics_children(h)`: root → open figures (sorted); figure → axes
  handles (+ `sgtitle` text when set); axes → registered plot objects;
  everything else → none.
- `label_objects(ax)`: title, subtitle, x/y/z labels that are set, legend
  when shown (encoded plot-object handles).
- `resolve_live(h)`: `resolve_plot_handle` on a finite non-negative number.

### findall

1. 0 args → `RunMat:minrhs`. First arg numeric scalar/array → roots; else
   `InvalidHandles`. Each root: live → search; positive integer → deleted,
   contributes nothing (D-058-1); else `InvalidHandles`.
2. Remaining args as pairs `('Type', t)` / `('-property', name)`; anything
   else → `UnsupportedOption`.
3. Pre-order DFS over `graphics_children ∪ label_objects`; keep an object
   when every filter matches (`Type` via `get_properties(h,'Type')`,
   property via `get_properties(h, name).is_ok()`).
4. Return an n-by-1 double column or 0x0.

### ishghandle

0 args → `RunMat:minrhs`; >2 → `UnsupportedOption`; non-text type →
`InvalidInput`; non-numeric `h` → false shaped like `h`; otherwise
delegate to `isgraphics_builtin(h[, type])`.

### allchild

1 arg only (`minrhs`/`UnsupportedOption` otherwise). Numeric values; each
must be live else `InvalidHandles`. One handle → its child column; several
→ n-by-1 cell of child columns.

### pcolor

1. Gather; split a leading axes handle (`split_leading_axes_handle`).
2. Text after the first argument → `UnsupportedOption`. 1 data arg → C with
   `x = 1:n` (row), `y = (1:m)'` (column); 2 data args without axes →
   `RunMat:badsubscript`; 3 → X, Y, C; other → `UnsupportedOption`.
3. Validate (real, non-empty, 2-D; min 2x2 else `UnsupportedOption`);
   expand X/Y to grids (vector length or matrix shape else
   `InputSizeMismatch`).
4. `SurfacePlot::from_coordinate_grids(X, Y, C)` with `flatten_z`, flat
   shading, edges on, lighting off, parula; source data X, Y as given and
   ZData zeros.
5. Select the target axes, render (`render_surface_plot`, registers the
   surface handle), set view `[0 90]`.

### surfl

1. Gather. 0 args or one non-numeric → `notEnoughInputs`; 1 → Z; 3 → X, Y,
   Z; other → `UnsupportedOption`.
2. Z rows < 3 → `InvalidInput`; cols < 3 → `UnsupportedOption`. Expand X/Y
   (x length → `lengthXDoesNotMatchNumColumnsZ`, y length →
   `lengthYDoesNotMatchNumRowsZ`, matrix shape → `InputSizeMismatch`).
3. `surfl_reflectance(X, Y, Z, view = [-37.5 30])` (FR-058-46).
4. `SurfacePlot::from_coordinate_grids(X, Y, Z)`, flat shading, edges on,
   lighting off, `color_grid` = parula(reflectance, auto limits); source
   data X, Y grids and Z.
5. Render into the current axes; set view `[-37.5 30]`.

## Test plan

- `findall_tests`: `observed_type_filter`,
  `observed_finds_title_and_label_text`, `observed_property_filter`,
  `observed_set_pattern`, `deviation_set_on_handle_array` (#[ignore], D-058-8),
  `observed_includes_the_object_itself_first`,
  `observed_no_match_is_empty`, `observed_deleted_figure_is_empty`,
  `observed_non_handle_number_errors`,
  `deviation_integer_non_handle_errors` (#[ignore], D-058-1),
  `unresolved_choice_queries_and_inputs`, `registered_through_dispatcher`.
- `ishghandle_tests`: `observed_live_handles_are_true`,
  `observed_type_argument`, `observed_false_cases`,
  `observed_empty_gives_0x0_logical`, `observed_no_input_errors_minrhs`,
  `unresolved_choice_text_and_argument_handling`,
  `registered_through_dispatcher`.
- `allchild_tests`: `observed_root_lists_new_invisible_figure`,
  `observed_figure_children_include_axes`,
  `observed_axes_children_are_plot_objects`,
  `observed_leaf_has_no_children`, `observed_array_input_gives_cell_column`,
  `observed_invalid_handles_error`, `unresolved_choice_argument_handling`,
  `registered_through_dispatcher`.
- `pcolor_tests`: `observed_c_only_data`, `observed_axes_argument`,
  `observed_vector_coordinates_kept_as_given`,
  `observed_matrix_coordinates_kept_as_given`,
  `observed_draws_into_current_axes`, `observed_view_from_above`,
  `observed_face_color_flat_and_edge_choice`,
  `observed_size_mismatch_errors`, `observed_two_numeric_arguments_error`,
  `unresolved_choice_unobserved_forms`, `registered_through_dispatcher`.
- `surfl_tests`: `observed_z_only`,
  `observed_vector_and_matrix_coordinates_expand_to_grids`,
  `observed_cdata_is_not_z`, `observed_view`,
  `observed_face_color_flat_and_edge_choice`, `observed_two_row_z_errors`,
  `observed_x_length_mismatch_errors`,
  `observed_single_char_argument_errors`,
  `unresolved_choice_unobserved_forms`,
  `unresolved_choice_flat_surface_uniform_reflectance`,
  `registered_through_dispatcher`.

Rendering is disabled in tests (`ensure_plot_test_env`); tests assert
object types, properties and stored plot data, not pixels.

## Verification commands

`cargo test -p runmat-runtime --lib -- findall_tests ishghandle_tests allchild_tests pcolor_tests surfl_tests` ·
`cargo test -p runmat-runtime --lib -- builtins::plotting` ·
`rustfmt --edition 2021 --check <files>` ·
`cargo clippy -p runmat-runtime --lib --tests`
