# Validation Report — 040-spy (spec 0.3.0) — 2026-10-29

Environment: Linux x86_64; `HDF5_DIR=~/anaconda3`; test binaries run with
anaconda's `lib/` on `LD_LIBRARY_PATH` through a cargo runner wrapper. The
target directory was shared with three other agents. MATLAB was not
invoked.

## Freshness (T0)

In `../matlab-interface-spec` after `git fetch -q --all`: `HEAD` =
`origin/main` = `8614c5b396851cdac1d1eb58f1cf5f7fa7f8909e`;
`git diff --name-only 8614c5b HEAD -- specifications/spy` and the same
against `origin/main` were empty; `git status --short` was clean. The
import is current.

## Test-first evidence (T4)

The tests were compiled with the module registered and **no builtin
present**:
`cargo test -p runmat-runtime --lib -- spy_tests exportgraphics_tests` →
**0 passed, 30 failed** (14 `spy_tests`, 16 `exportgraphics_tests`). Every
failure came from the builtin being absent: either a panic on
`Undefined function: spy` / `…: exportgraphics` (15) or an identifier
assertion whose left side was `RunMat:UndefinedFunction` (14). The one
remaining failure, `observed_plots_adds_axes_children`, failed on its
precondition, which used axes `Children` (see D-040-4). That precondition
was later rewritten.

(An earlier attempt on the same day failed to compile because of another
agent's in-progress edit to `table/builtins/variable_editing.rs`. It was
retried unchanged once that file compiled.)

## Commands and results (T7)

- `cargo test -p runmat-runtime --lib -- spy_tests exportgraphics_tests`:
  **30 passed, 0 failed** (14 `spy_tests`):
  - `observed_plots_adds_axes_children`
  - `observed_output_request_errors`
  - `observed_points_one_marker_only_line`
  - `observed_axes_limits_and_equal_units`
  - `observed_nz_label`
  - `observed_all_zero_gives_one_nan_line`
  - `observed_linespec_sets_marker_and_colour`
  - `observed_markersize_sets_marker_size`
  - `observed_dense_logical_char_inputs`
  - `unresolved_choice_complex_input_unsupported`
  - `unresolved_choice_unobserved_forms_unsupported`
  - `rejects_non_numeric_inputs`
  - `rejects_nd_and_missing_input`
  - `rejects_invalid_style_arguments`
- Plotting regression:
  `cargo test -p runmat-runtime --lib -- builtins::plotting::print builtins::plotting::figure_persistence builtins::plotting::get builtins::plotting::findobj`
  → **45 passed, 0 failed**. Before any change, `builtins::plotting::print`
  alone gave 7 passed.
- `cargo clippy -p runmat-runtime --lib --tests --message-format short`:
  exit 0, **no warnings or errors** anywhere in the crate, including
  `ops/spy.rs`.
- `rustfmt --edition 2021 --check` on `ops/spy.rs`, `ops/exportgraphics.rs`
  and `ops/mod.rs`: clean. `cargo fmt --all` was not run.

## Traceability (claim → FR → test → result) (T8)

| Claim (0.3.0) | FR | Test(s) | Result |
|---|---|---|---|
| spy.void-plot | FR-040-01 | `observed_plots_adds_axes_children` (no output; a line appears) | PASS (proxy via findobj, D-040-4) |
| spy.error-output-arg | FR-040-01 | `observed_output_request_errors` | PASS (`RunMat:spy:TooManyOutputs`, D-040-3) |
| spy.one-line | FR-040-02 | `observed_points_one_marker_only_line`, `observed_all_zero_gives_one_nan_line`, `observed_linespec_sets_marker_and_colour` | PASS |
| spy.positions | FR-040-03 | `observed_points_one_marker_only_line`, `observed_dense_logical_char_inputs` | PASS |
| spy.axes (XLim, YLim, equal data units) | FR-040-04 | `observed_axes_limits_and_equal_units` (points, tall, all-zero), `observed_dense_logical_char_inputs` (char limits) | PASS |
| spy.axes (YDir 'reverse') | FR-040-04 | none | **NOT IMPLEMENTED** (D-040-1) |
| spy.nz-label | FR-040-05 | `observed_nz_label` (3, 2, 0) | PASS |
| spy.default-marker | FR-040-06 | `observed_points_one_marker_only_line`, `observed_markersize_sets_marker_size` | PASS (`'.'` passed to plot; read back as a non-`none` marker, D-040-2) |
| spy.linespec | FR-040-07 | `observed_linespec_sets_marker_and_colour` (`'r*'` → `*`, red; `'g+'` → `+`, green) | PASS |
| spy.markersize | FR-040-08 | `observed_markersize_sets_marker_size` (20), `observed_linespec_sets_marker_and_colour` (12) | PASS |
| spy.all-zero | FR-040-09 | `observed_all_zero_gives_one_nan_line` | PASS |
| spy.input-types | FR-040-10 | `observed_dense_logical_char_inputs` (dense, logical, char); sparse in the other observed tests | PASS |
| spy.default-style-values (non-normative) | FR-040-11 | not asserted; RunMat defaults kept | n/a |
| spy.q-large | FR-040-12 | `unresolved_choice_complex_input_unsupported` | PASS |
| U-040-2 (unobserved forms) | FR-040-13 | `unresolved_choice_unobserved_forms_unsupported` | PASS |
| documented choice: validation | FR-040-14 | `rejects_non_numeric_inputs`, `rejects_nd_and_missing_input`, `rejects_invalid_style_arguments` | PASS |

Every FR has at least one test, and every test maps to an FR.

## Deviations

- **D-040-1: `YDir 'reverse'` is not implemented.** RunMat axes have no
  axis-direction model. Upstream accepts `axis ij` as a no-op, and there is
  no `YDir` property. Reversing the data instead (`YData = -row`) would
  break spy.positions, so row 1 is drawn at the bottom. Implementing this
  needs an axis-direction feature in upstream plotting core and renderer,
  which is outside this feature's scope.
- **D-040-2: `Marker` reads `'o'`.** RunMat's line model maps the `.`
  token to a point-sized circle (`MarkerKind::Point` →
  `PlotMarkerStyle::Circle`), and `get(h,'Marker')` reports `'o'`. spy
  passes `'.'`. The tests only assert that the line has markers.
- **D-040-3: identifier.** The observed `MATLAB:TooManyOutputs` becomes
  `RunMat:spy:TooManyOutputs`, following RunMat's convention (as `display`
  and `diary` do) and batch C2 decision D2.
- **D-040-4: Children proxy.** RunMat's axes `Children` property always
  lists six fixed objects (title, subtitle, three labels, legend) and never
  plot lines. The observation's proxy, "Children became non-empty", is
  therefore checked as "a line object appears", using
  `findobj(gca,'Type','line')`.

## Finding F-1 (upstream, not fixed, outside scope)

In this build, every builtin checked (`spy`, `plot`, `find`, `gca`,
`display`) has **two** entries in the legacy builtin inventory (a debug
probe found `builtin_functions()` count 2, canonical bindings 0). When a
call through `call_builtin_async` fails, the dispatcher runs the second
entry as a retry. During that retry the requested output count is no
longer visible: it reads `None`, and a nested `find` asked for 2 outputs
saw 1. The error finally reported can come from the retry rather than the
real cause.

- Consequences here: spy evaluates `find` directly
  (`array::indexing::find::evaluate`) instead of through the dispatcher,
  and the error-path tests call `spy_builtin` directly. This is the same
  pattern as upstream's `display` test, using `push_output_count(Some(1))`
  for the output-request case. The normative success paths go through
  `call_builtin_async`.
- The svds agent's debug output, seen in the shared target directory,
  shows the same retry pattern. This should be raised upstream.

## Not verified headlessly

- How the plot looks when rendered is not checked: tests run with
  interactive rendering disabled and assert graphics-object properties
  only.
- spy was not run end to end through the MATLAB-syntax VM; only the
  runtime dispatcher and direct calls were used.
