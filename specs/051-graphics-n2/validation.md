# Validation Report — 051-graphics-n2 (`hsv`, `alpha`, `tiledlayout` 0.2.0) — 2026-10-30

Environment: Linux x86_64; `HDF5_DIR=~/anaconda3`; test binaries run
through the shared cargo runner wrapper. The target directory was shared
with other N2 agents; two build attempts were blocked by other agents'
in-progress files (`graph/domain/addnode.rs`, `object/resolve.rs`) and were
retried unchanged. MATLAB was not invoked.

## Freshness (T0 and finish)

In `../matlab-interface-spec` after `git fetch -q origin`, at start and at
finish: `HEAD` = `origin/main` = `1156c97` (batch N3, after the `8f72edb`
pin). `git log 8f72edb..origin/main` and `git log 8f72edb..HEAD` for
`specifications/{hsv,alpha,tiledlayout}` are empty, and `git status
--short` for those directories is clean. The twelve imported files match
`SHA256SUMS`. The import is current.

## Test-first evidence (T4)

Tests were compiled with the modules registered and **stub builtins**
(each returned an error; `add_tile`/`layout_property` likewise):
`cargo test -p runmat-runtime --lib -- hsv_tests alpha_tests tiledlayout_tests`
→ **2 passed, 32 failed**. The two passes were
`hsv_tests::descriptor_lists_both_forms_and_error_identifiers` (descriptor
metadata was written with the stub) and
`hsv_tests::rejects_non_scalar_text_extra_and_huge_lengths` (the stub
happened to return `RunMat:hsv:InvalidInput`). Every other test failed on
the missing behaviour.

## Commands and results (T8)

- `cargo test -p runmat-runtime --lib -- hsv_tests alpha_tests tiledlayout_tests`
  → **35 passed, 0 failed** (9 `hsv_tests`, 13 `alpha_tests`,
  13 `tiledlayout_tests`). The first passing run had 34 tests;
  `deviation_properties_read_through_field_access` was added afterwards and
  the final run is 35/35.
- Surrounding suite `cargo test -p runmat-runtime --lib -- builtins::plotting`
  → **1099 passed, 1 failed**. The failure is the known pre-existing
  `contourf::tests::contourf_accepts_target_axes_and_does_not_alias_integer_z_as_a_handle`
  (axes-handle form), unrelated to this feature.
- `cargo clippy -p runmat-runtime --lib --tests --message-format short`:
  **no findings in this feature's files** (two lints found along the way,
  `manual_is_multiple_of` and `cloned_ref_to_slice_refs` in
  `tiledlayout.rs`, were fixed). The command still exits 101 because of
  one lint in another agent's in-progress
  `builtins/containers/map/conformance_tests.rs:303`, outside this
  feature.
- `rustfmt --edition 2021 --check` on `ops/alpha.rs`, `ops/tiledlayout.rs`,
  `ops/colormap_arrays.rs`, `ops/mod.rs`: clean. `cargo fmt --all` was not
  run.

## Traceability (claim → FR → test → result) (T9)

| Claim (0.2.0) | FR | Test(s) | Result |
|---|---|---|---|
| hsv.signature-primary, hsv.output-class | FR-051-01 | `observed_six_four_one_values_and_class` | PASS |
| hsv.values | FR-051-01 | `observed_six_four_one_values_and_class` | PASS |
| hsv.range | FR-051-01 | `observed_range_of_hsv_64` | PASS |
| hsv.zero | FR-051-02 | `observed_zero_is_0_by_3` | PASS |
| hsv.default | FR-051-03 | `observed_default_size_without_figure_is_256` | PASS |
| hsv.error-negative, hsv.error-fraction | FR-051-04 | `observed_negative_and_fraction_errors` | PASS |
| hsv.q-default-with-figure (unresolved) | FR-051-05 | `unresolved_choice_default_length_follows_open_figure_colormap` | PASS |
| documented choice | FR-051-06 | `unresolved_choice_integer_classes_and_nonfinite_lengths`, `rejects_non_scalar_text_extra_and_huge_lengths` | PASS |
| (scope / metadata) | FR-051-07 | `descriptor_lists_both_forms_and_error_identifiers` | PASS |
| alpha.signature-primary, alpha.scalar | FR-051-10 | `observed_scalar_sets_surface_face_alpha`, `observed_scalar_sets_patch_face_alpha`, `observed_object_form_sets_given_surface` | PASS (1e-6 tolerance, D-051-1) |
| alpha.keywords (`clear`, `opaque`) | FR-051-11 | `observed_clear_and_opaque_keywords` | PASS |
| alpha.edge-untouched | FR-051-12 | `observed_edge_alpha_untouched` | PASS (on a patch, D-051-3) |
| alpha.line-only | FR-051-13 | `observed_line_only_axes_completes` | PASS |
| alpha.no-output, alpha.error-output-arg | FR-051-14 | `observed_output_request_errors` | PASS (`RunMat:alpha:TooManyOutputs`, D-051-5) |
| alpha.error-bad-keyword | FR-051-15 | `observed_unknown_keyword_errors` | PASS |
| alpha.keywords (`flat`, `interp`) | FR-051-16 | `deviation_flat_and_interp_unsupported` | **NOT IMPLEMENTED** (D-051-2; raises `UnsupportedOption`) |
| alpha.q-more-forms (unresolved) | FR-051-17 | `unresolved_choice_mixed_axes_and_axes_target`, `unresolved_choice_keywords_case_insensitive_and_string_scalar`, `unresolved_choice_unobserved_values_and_targets_unsupported`, `rejects_missing_value_and_invalid_handle` | PASS |
| tiledlayout.signature-primary, tiledlayout.output-class | FR-051-20 | `observed_grid_layout_properties`, `observed_flow_layout_and_growth`, `observed_spacing_and_padding_stored` | PASS |
| tiledlayout.grid | FR-051-21 | `observed_grid_layout_properties` | PASS |
| tiledlayout.tiles | FR-051-22 | `observed_tiles_add_one_child_each` | PASS (through internal `add_tile`, D-051-4) |
| tiledlayout.flow | FR-051-23 | `observed_flow_layout_and_growth` | PASS (growth through `add_tile`, D-051-4) |
| tiledlayout.options | FR-051-24 | `observed_spacing_and_padding_stored` | PASS |
| tiledlayout.parent | FR-051-25 | `observed_figure_parent` | PASS (`Parent` is numeric, D-051-6) |
| tiledlayout.error-one-arg, -zero, -spacing | FR-051-26 | `observed_single_numeric_argument_errors`, `observed_zero_grid_dimension_errors`, `observed_invalid_tile_spacing_errors` | PASS |
| tiledlayout.q-flow-growth (unresolved) + documented choices | FR-051-27 | `unresolved_choice_flow_growth_rule`, `unresolved_choice_option_values_and_names` | PASS |
| unobserved forms / validation | FR-051-28 | `unresolved_choice_unobserved_forms_unsupported`, `rejects_malformed_inputs` | PASS |
| (storage model) | D-051-7 | `deviation_properties_read_through_field_access` (`load_member`, the `t.GridSize` path) | PASS |

Every FR has at least one test, and every test maps to an FR or deviation.

## Deviations

- **D-051-1: `FaceAlpha` precision.** Upstream surfaces and patches store
  `FaceAlpha` as `f32`, so `alpha(s, 0.3)` reads back `0.30000001192…`
  rather than the double `0.3` the observation shows. Tests compare with a
  1e-6 tolerance. Fixing it needs an `f64` field in `runmat_plot`.
- **D-051-2: `'flat'` / `'interp'`.** Confirmed to store those keywords in
  `FaceAlpha`; RunMat has no keyword or alpha-data storage for `FaceAlpha`
  (a numeric `f32`), so `alpha` raises `RunMat:alpha:UnsupportedOption`
  and leaves objects unchanged.
- **D-051-3: `EdgeAlpha` on surfaces.** RunMat surfaces expose no
  `EdgeAlpha`; the "EdgeAlpha untouched" rule is checked on a patch.
- **D-051-4: no `nexttile`.** No approved `nexttile` specification exists
  in N2, so there is no public builtin and MATLAB code cannot add tiles.
  The tile model (`add_tile`) is implemented and tested internally. A
  `nexttile` spec is suggested for the specification side.
- **D-051-5: identifier.** The observed `MATLAB:TooManyOutputs` becomes
  `RunMat:alpha:TooManyOutputs` (convention of `spy`/`exportgraphics`).
- **D-051-6: `Parent` class.** RunMat figures are numeric handles, so
  `t.Parent` is the figure number (class `double`), not a figure object.
- **D-051-7: `get`/`set` on layouts.** Properties are read with member
  access (`t.GridSize`). `get(t, …)`/`set(t, …)` are not wired, because
  that needs upstream changes to `get.rs`/`set.rs`. The public `getfield`
  builtin is gated for handle objects by RunMat's
  `GetfieldObjectFamilyExtension` compatibility setting (upstream policy).
- **D-051-8: layout rendering.** `TileSpacing` and `Padding` are stored
  only; tiles use the subplot grid's own spacing.

## Not verified headlessly

- How transparency looks when rendered, and the visual tile layout, are not
  checked: tests run with rendering disabled and assert properties only.
- No end-to-end run through the MATLAB-syntax VM. `t.GridSize` was checked
  through the runtime's `object::resolve::load_member`, which is the path
  the VM uses for member access.
