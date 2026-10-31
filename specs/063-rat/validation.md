# Validation Report — 063-rat (spec 0.2.0) — 2026-10-31

Environment: Linux x86_64; `HDF5_DIR=~/anaconda3`; test binaries run
through the cargo runner wrapper; shared target directory. MATLAB was not
invoked.

## Freshness check (T0, T7)

In `../matlab-interface-spec`, after `git fetch -q origin` (rc 0), at start
and finish: `HEAD` = `origin/main` = `1c91aa1` (the pin);
`git log 1c91aa1..origin/main -- specifications/rat experiments/rat`
empty; `git status --short` for both empty; `diff -rq` of the imported copy
against the source tree reports no difference. The imported copy is
current.

## Test-first evidence (T2–T4)

`rat.rs` was first written with `rat` registered as a stub (returning
`RunMat:rat:Internal` "not yet implemented") and the complete test module
(built together with feature 062's stubs).
`cargo test -p runmat-runtime --lib -- graph_n5 rat_tests` →
**0 passed, 38 failed, 2 ignored** (13 of the failures are this feature's;
the ignored tests belong to 062).

## Commands and results (T6)

- `cargo test -p runmat-runtime --lib -- graph_n5 rat_tests` after the
  implementation: **38 passed, 0 failed, 2 ignored** (this feature: 13
  passed).
- `cargo test -p runmat-runtime --lib -- builtins::math::rounding`:
  **129 passed, 0 failed**. No regressions.
- `rustfmt --edition 2021 --check crates/runmat-runtime/src/builtins/math/rounding/rat.rs`:
  clean.
- `cargo clippy -p runmat-runtime --lib --tests --message-format short`:
  see "Clippy" below.

## Clippy

`cargo clippy -p runmat-runtime --lib --tests --message-format short`: the
first run stopped on a finding in another agent's in-progress file
(`plotting/ops/rectangle.rs`, lib target; no lib finding in this feature).
The second run reported one test-code finding here
(`rat.rs`: `&[z.clone()]` → `std::slice::from_ref(&z)`) and one in
feature 062; both were fixed. The final run **finished with no findings**
(lib and tests), and the feature tests were re-run (38 passed, 2 ignored,
together with 062).

## Traceability (claim → FR → test → result) (T7)

| Claim / basis | FR | Test(s) | Result |
|---|---|---|---|
| rat.signature-primary, rat.simple | FR-063-01 | `observed_simple_values` | PASS |
| rat.default-tol | FR-063-02 | `observed_default_tolerance` (`355/113`, error `2.667641894049666e-07` exactly; `6667/10000`) | PASS |
| rat.tol | FR-063-03 | `observed_explicit_tolerance` (`22/7`, error `0.0012644892673496777` exactly) | PASS |
| rat.elementwise | FR-063-04 | `observed_elementwise_and_empty` | PASS |
| rat.class | FR-063-05 | `observed_simple_values`, `observed_single_class` | PASS |
| rat.nonfinite | FR-063-06 | `observed_nonfinite` | PASS |
| rat.text | FR-063-07 | `observed_text_form` | PASS |
| rat.error-char (D-063-1) | FR-063-08 | `rejects_char_and_integer` | PASS |
| rat.complex (non-normative; D-063-3) | — | `supported_complex_as_observed` | PASS |
| U-063-1, U-063-2 | FR-063-09 | `unresolved_choice_expansion_and_default_tolerance` | PASS |
| U-063-3, U-063-5, U-063-6 | FR-063-09 | `unresolved_choice_inputs_and_outputs` | PASS |
| U-063-4 | FR-063-09 | `unresolved_choice_complex_layout`, `unresolved_choice_inputs_and_outputs` (non-finite rows, empty) | PASS |
| rat.q-warning-coverage | FR-063-10 | no warning path exists | not tested (warnings) |

Every FR maps to at least one test; every test cites a claim or a
documented choice.

## Unresolved-question dispositions

| id | disposition |
|---|---|
| rat.q-default-tol | U-063-2 (`1e-6·|x|` per element; reproduces all observed cases) |
| rat.q-complex | supported as observed (D-063-3), layout U-063-4 |
| rat.q-error-ids | `RunMat:norm:inputType` (D-063-1) |
| rat.q-warning-coverage | no warnings emitted |

## Deviations

None.

## Notes

- `pub(crate) mod rat;` added to the hand-written
  `math/rounding/mod.rs` (D-063-2). No generated parent `mod.rs` edited.
- Integer-capability metadata was not declared.
- No MathWorks material was consulted; MATLAB was not invoked.
  Independent legal review MAY still be required.
