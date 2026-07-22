# Tasks — 028-eps-function

Order is normative (test-first). Every task cites requirements.

- [x] **T1** Confirm the RunMat single-scalar representation and the bare-
  identifier resolution order: inspect `class_name_for_value`
  (`introspection/class.rs`) — a 1×1 `NumericDType::F32` tensor reports
  `single`; inspect `runmat-hir` `lower_expr_semantic` — a bare identifier
  that is not a variable/constant dispatches to a zero-arg builtin call.
  [FR-028-02, FR-028-06] (no code changes)
- [x] **T2** Author observed (normative) tests: bare `eps` → exact double
  `f64::EPSILON`; `eps('single')` → exact `f32::EPSILON`, class `single`,
  size `1×1`. [FR-028-01, FR-028-02; SC-028-1]
- [x] **T3** Author documented-choice (`unresolved_choice_*`) and error tests:
  `eps('double')`, case-insensitive class, `eps(x)` ulp (1, 2, negative, 0,
  Inf/NaN), double array shape, single array class, complex magnitude,
  complex tensor, invalid class, too-many-args, integer input.
  [FR-028-03, FR-028-04, FR-028-05]
- [x] **T4** Author registry/resolution test proving `eps` is a builtin and
  not a constant (bare-`eps` evidence at the level the HIR consults).
  [FR-028-06; SC-028-2]
- [x] **T5** Implement `eps.rs` (descriptor, GPU/fusion metadata specs,
  `#[runtime_builtin]`, zero-or-one arg dispatch, `nextUp` spacing, single-
  class result) + `mod` entry in `math/elementwise/mod.rs`.
  [FR-028-01..05]
- [x] **T6** Remove **only** the `register_constant!("eps", …)` entry from
  `constants/mod.rs` (leave all other constants); add explanatory comment.
  [FR-028-06]
- [x] **T7** Verify bare `eps` end-to-end through the VM (temporary
  `runmat-core` integration test, run then removed); record the outcome.
  [SC-028-2]
- [x] **T8** Run verification commands (`cargo check`, targeted + `eps`-wide
  `cargo test`, `cargo fmt --all -- --check`, `cargo clippy`); record output
  in `validation.md`. [SC-028-1..3]
- [x] **T9** Traceability table in `validation.md` (claim → FR → test →
  result), plus the bare-`eps` resolution evidence and carried unresolved
  questions. [SC-028-3; Principle VI]
