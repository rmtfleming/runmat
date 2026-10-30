# Implementation Plan — 040-spy (spec 0.3.0)

## Constitution Check

- I Specification-first: FR-040-01..10 cite the eleven normative 0.3.0
  claims; FR-040-11..14 are tagged unresolved or documented choice;
  deviations D-040-1..3 are explicit. PASS.
- II Clean-room: inputs are the imported export plus RunMat code only;
  MATLAB not invoked. PASS.
- III Independence: `spy` is composed from RunMat's own `find`, `plot`,
  `xlim`, `ylim`, `daspect` and `xlabel` builtins. PASS.
- IV Provenance: `provenance.md` pinned to `8614c5b`, freshness checked.
  PASS.
- V Test-first: tests are written, compiled and run (failing) before the
  builtin exists. PASS (see validation.md).
- VII Minimal scope: one new file plus two lines in the hand-written
  `plotting/ops/mod.rs`; no upstream plotting edits. PASS.
- IX Compatibility: `#[runtime_builtin]` with a descriptor, `sink = true`,
  `suppress_auto_output = true`, no-output return
  `Value::OutputList(vec![])` as `hold`; no integer-capability metadata.
  PASS.

## Affected files

| Item | Path |
|---|---|
| Builtin (new) | `crates/runmat-runtime/src/builtins/plotting/ops/spy.rs` |
| Registration | `#[path = "spy.rs"] pub(crate) mod spy;` in `crates/runmat-runtime/src/builtins/plotting/ops/mod.rs` |

`plotting/mod.rs` is `@generated` and is not edited.
`builtin_path = "crate::builtins::plotting::ops::spy"`.

## Design

1. Output count > 0 → `TooManyOutputs`.
2. Parse style arguments (`rest`): see FR-040-07/08/13/14. Build a plot
   spec string `colour + marker` (marker defaults to `.`), and an optional
   `MarkerSize` name-value pair.
3. Gather `A`. Classify: real numeric / logical / sparse real-or-logical /
   char (converted to its code points) → OK; complex (dense or sparse) →
   `UnsupportedOption`; everything else → `InvalidInput`. More than two
   dimensions → `InvalidInput`.
4. `[i, j] = find(A)` through the dispatcher (2 outputs). Empty → `NaN`.
5. `plot(j', i', spec[, 'MarkerSize', ms])`; then `xlim([0 n+1])`,
   `ylim([0 m+1])`, `daspect([1 1 1])`, `xlabel(sprintf('nz = %d', nnz))`,
   all through the dispatcher on the current axes.
6. Return `Value::OutputList(vec![])`.
- Errors: `RunMat:spy:{InvalidInput,UnsupportedOption,TooManyOutputs,Internal}`.

## Test plan (inline `#[cfg(test)] mod spy_tests`)

Observed: `observed_plots_adds_axes_children`,
`observed_output_request_errors`, `observed_points_one_marker_only_line`,
`observed_axes_limits_and_equal_units`, `observed_nz_label`,
`observed_all_zero_gives_one_nan_line`,
`observed_linespec_sets_marker_and_colour`,
`observed_markersize_sets_marker_size`,
`observed_dense_logical_char_inputs`. Unresolved:
`unresolved_choice_complex_input_unsupported`,
`unresolved_choice_unobserved_forms_unsupported`. Validation:
`rejects_non_numeric_inputs`, `rejects_nd_and_missing_input`,
`rejects_invalid_style_arguments`.

## Verification commands

`cargo test -p runmat-runtime --lib -- spy_tests` ·
`cargo test -p runmat-runtime --lib -- builtins::plotting::print` ·
`rustfmt --edition 2021 --check <files>` ·
`cargo clippy -p runmat-runtime --lib --tests --message-format short | grep ops/spy.rs`
