# Tasks — 029-time-conversion

Order is normative (test-first). Every task cites requirements.

- [x] **T1** Read the approved exports (`etime`, `seconds`) and the RunMat
  `duration` module; confirm the `Value` construction path
  (`duration_object_from_days_tensor`) and the date-conversion approach.
  [FR-029-01, FR-029-02, FR-029-03] (no code changes)
- [x] **T2** Author normative + unresolved-choice tests for `etime` in
  `datetime/etime.rs` (observed one-second → `1`; negative, cross-day,
  cross-year, invalid-input tiers). [FR-029-01, FR-029-04, FR-029-05;
  SC-029-1]
- [x] **T3** Author normative + unresolved-choice tests for `seconds` in
  `duration/seconds.rs` (observed `seconds(5)` → 1×1 duration of 5 s; array
  input; invalid-input tiers). [FR-029-02, FR-029-03, FR-029-06; SC-029-1]
- [x] **T4** Implement `etime.rs` (independent `days_from_civil`, descriptor,
  GPU/fusion specs, macro, type_resolver) + `pub mod etime;` in
  `datetime/mod.rs`. [FR-029-01, FR-029-04, FR-029-05]
- [x] **T5** Implement `seconds.rs` (reuse `duration_object_from_days_tensor`,
  descriptor, GPU/fusion specs, macro) + `pub mod seconds;` in
  `duration/mod.rs`. [FR-029-02, FR-029-03, FR-029-06]
- [x] **T6** Run verification commands (`cargo check`, targeted tests,
  `cargo fmt --all -- --check`, `cargo clippy -p runmat-runtime --lib`);
  record output in `validation.md`. [SC-029-2]
- [x] **T7** Traceability table in `validation.md` (claim → FR → test →
  result). [SC-029-3; Principle VI]
