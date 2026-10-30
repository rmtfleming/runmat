# Implementation Plan — 051-graphics-n2 (`hsv`, `alpha`, `tiledlayout` 0.2.0)

## Constitution Check

- I Specification-first: FR-051-01..04, 10..15, 20..26 cite the normative
  0.2.0 claims; FR-051-05..07, 16..17, 27..28 are tagged documented choice,
  scope or deviation. PASS.
- II Clean-room: inputs are the imported exports plus RunMat code only;
  MATLAB not invoked. PASS.
- III Independence: `hsv` uses the textbook HSV-to-RGB formula; `alpha`
  composes RunMat's `set` on surface/patch handles; `tiledlayout` reuses
  RunMat's handle-object pattern (`pan`, `datacursormode`) and subplot grid.
  PASS.
- IV Provenance: `provenance.md` pinned to `8f72edb`; freshness checked.
  PASS.
- V Test-first: tests written and run against stubs before the
  implementation (validation.md). PASS.
- VII Minimal scope: two new files, one sibling builtin in
  `colormap_arrays.rs`, four lines in the hand-written `ops/mod.rs`. No
  upstream subsystem refactor. PASS.
- IX Compatibility: `#[runtime_builtin]` with descriptors; `alpha` uses
  `sink = true`, `suppress_auto_output = true` and returns
  `Value::OutputList(vec![])` like `spy`; no integer-capability metadata.
  PASS.

## Affected files

| Item | Path |
|---|---|
| `hsv` (sibling of the colormap family) | `crates/runmat-runtime/src/builtins/plotting/ops/colormap_arrays.rs` |
| `alpha` (new) | `crates/runmat-runtime/src/builtins/plotting/ops/alpha.rs` |
| `tiledlayout` (new) | `crates/runmat-runtime/src/builtins/plotting/ops/tiledlayout.rs` |
| Registration | `#[path]` module lines in the hand-written `crates/runmat-runtime/src/builtins/plotting/ops/mod.rs` |

`plotting/mod.rs` is `@generated` and is not edited. `colormap_arrays` is
declared there already, so `hsv` needs no new module line.

## Design

### hsv

1. Arguments: none → `current_colormap_length()` (256 with no figure);
   one → length validation (FR-051-04, FR-051-06); more → `InvalidInput`.
2. Row `i` = RGB of hue `i/n`: `h6 = 6*i/n`, sector `floor(h6)`, fraction
   `f`; sectors give `[1 f 0] [1-f 1 0] [0 1 f] [0 1-f 1] [f 0 1] [1 0 1-f]`.
3. Return an n-by-3 double `Tensor` (column-major).
- Errors: `RunMat:validators:{mustBeNonnegative,mustBeInteger}`,
  `RunMat:hsv:InvalidInput`.

### alpha

1. Output count > 0 → `TooManyOutputs`.
2. 0 args → `InvalidInput`; >2 → `UnsupportedOption`.
3. Parse value: text → `clear` 0, `opaque` 1, `flat`/`interp`
   `UnsupportedOption`, other `UnknownAlphaValue`; numeric → real finite
   scalar in [0, 1] else `UnsupportedOption`.
4. Target: 1 arg → current figure's current axes; 2 args →
   `resolve_plot_handle(obj)`: Axes → that axes; Surface/Patch child → that
   object; other kind → `UnsupportedOption`; invalid → `InvalidInput`.
5. For an axes, collect `plot_child_handles_for_axes` whose snapshot is
   Surface or Patch; for each, `set(h, 'FaceAlpha', v)` through
   `set_builtin` (upstream validation and storage).
6. Return `Value::OutputList(vec![])`.

### tiledlayout

1. Gather arguments. Leading parent: three leading numeric scalars, or a
   numeric scalar followed by `'flow'` → first is the parent figure
   (`resolve_plot_handle` must give a Figure, else `InvalidInput`).
2. Arrangement: `'flow'` → flow, `GridSize [1 1]`; other text →
   `UnsupportedOption`; numeric `m` then numeric `n` → fixed (positive
   integers else `InvalidGridSize`); `m` alone → `notEnoughInputs`; no
   arguments → `notEnoughInputs`.
3. Options: pairs of (text name, value); names `TileSpacing`/`Padding`
   (case-insensitive) else `UnsupportedOption`; unpaired or non-text name
   → `InvalidInput`; values as FR-051-24/27 else `InvalidEnumValue`.
4. Build `ObjectInstance` of class `matlab.graphics.layout.TiledChartLayout`
   with `GridSize`, `TileArrangement`, `TileSpacing`, `Padding`, `Children`
   (0-by-1), `Parent` (figure number); allocate in the GC and return a
   `Value::HandleObject`, as `pan.rs` does.
5. `add_tile(layout)`: k = children + 1; fixed: k > m*n →
   `UnsupportedOption`; flow: recompute `GridSize` (FR-051-27). Select the
   parent figure, `configure_subplot(rows, cols, k-1)`, append
   `encode_axes_handle(fig, k-1)` to `Children`, write back to the object.
- Errors: `RunMat:narginchk:notEnoughInputs`,
  `RunMat:tiledchartlayout:InvalidGridSize`,
  `RunMat:datatypes:InvalidEnumValue`,
  `RunMat:tiledlayout:{InvalidInput,UnsupportedOption,Internal}`.

## Test plan

- `hsv_tests` (in `colormap_arrays.rs`): `observed_six_four_one_values_and_class`,
  `observed_range_of_hsv_64`, `observed_zero_is_0_by_3`,
  `observed_default_size_without_figure_is_256`,
  `observed_negative_and_fraction_errors`,
  `unresolved_choice_default_length_follows_open_figure_colormap`,
  `unresolved_choice_integer_classes_and_nonfinite_lengths`,
  `rejects_non_scalar_text_extra_and_huge_lengths`,
  `descriptor_lists_both_forms_and_error_identifiers`.
- `alpha_tests`: `observed_scalar_sets_surface_face_alpha`,
  `observed_scalar_sets_patch_face_alpha`,
  `observed_object_form_sets_given_surface`,
  `observed_clear_and_opaque_keywords`, `observed_edge_alpha_untouched`,
  `observed_line_only_axes_completes`, `observed_output_request_errors`,
  `observed_unknown_keyword_errors`, `deviation_flat_and_interp_unsupported`,
  `unresolved_choice_mixed_axes_and_axes_target`,
  `unresolved_choice_keywords_case_insensitive_and_string_scalar`,
  `unresolved_choice_unobserved_values_and_targets_unsupported`,
  `rejects_missing_value_and_invalid_handle`.
- `tiledlayout_tests`: `observed_grid_layout_properties`,
  `observed_tiles_add_one_child_each`, `observed_flow_layout_and_growth`,
  `observed_spacing_and_padding_stored`, `observed_figure_parent`,
  `observed_single_numeric_argument_errors`,
  `observed_zero_grid_dimension_errors`,
  `observed_invalid_tile_spacing_errors`, `unresolved_choice_flow_growth_rule`,
  `unresolved_choice_option_values_and_names`,
  `unresolved_choice_unobserved_forms_unsupported`, `rejects_malformed_inputs`,
  `deviation_properties_read_through_field_access`.

## Verification commands

`cargo test -p runmat-runtime --lib -- hsv_tests alpha_tests tiledlayout_tests` ·
`cargo test -p runmat-runtime --lib -- builtins::plotting` ·
`rustfmt --edition 2021 --check <files>` ·
`cargo clippy -p runmat-runtime --lib --tests`
