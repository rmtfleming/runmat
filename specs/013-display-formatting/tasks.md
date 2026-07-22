# Tasks — 013-display-formatting

Order is normative (test-first). Every task cites requirements. No task may
start before Gate 3 approval.

- [x] **T1** Confirm substrate (no code changes): `Value` variants and
  helpers used by the formatter (`format_number` default-Short rendering of
  `42.0`; `char_row_to_string`; `canonical_dims`); confirm console capture
  via `reset_thread_buffer`/`take_thread_buffer` matches the quad.rs/fzero.rs
  precedent; confirm `io/mod.rs` flat `pub mod` registration pattern.
  [FR-013-01, FR-013-02]
- [x] **T2** Author failing normative tests in `io/display.rs`'s test module
  (red step; compile gated by T4) asserting byte-exact strings from the pure
  formatter:
  - `display_scalar_text`: `format_display_text(Num(42.0)) == "    42\n\n"`
    [FR-013-03]
  - `display_vector_text`: 1x3 tensor `[1 2 3]` →
    `"     1     2     3\n\n"` [FR-013-04]
  - `display_char_text`: 1x2 char `'hi'` → `"hi\n"` [FR-013-05]
  [SC-013-1]
- [x] **T3** Author failing `unresolved_choice`-tagged tests (documented
  choices, non-normative): second argument rejected with clear error
  (`display(X, name)` pending in export); no `name =` header ever emitted;
  delegation rendering for one non-observed kind (e.g. struct) is stable.
  [FR-013-06]
- [x] **T4** Implement `format_display_text` (pure, returns full text with
  trailing newlines) + `display_builtin` (descriptor, GPU/fusion sink specs,
  `gather_if_needed_async`, `record_console_output`, 0x0 sink return,
  `suppress_auto_output`) + `pub mod display;` registration; make
  `format_for_disp` `pub(crate)` only if the delegation path needs it.
  [FR-013-01..06]
- [x] **T5** Console-capture integration tests (one per observed case):
  `reset_thread_buffer()` → `block_on(display_builtin(...))` →
  `take_thread_buffer()` joined text equals the observed string exactly;
  plus GPU-tensor gather smoke test mirroring `disp_accepts_gpu_tensor`.
  [FR-013-02, FR-013-03..05; SC-013-1, SC-013-2]
- [x] **T6** Run verification commands (`cargo fmt --all -- --check`,
  `cargo check -p runmat-runtime`, `RUST_TEST_THREADS=1 cargo test -p
  runmat-runtime display`, and `cargo test -p runmat-runtime disp` to prove
  no disp regression); record actual output in `validation.md`.
  [SC-013-1, SC-013-2, SC-013-4]
- [x] **T7** Traceability table in `validation.md` (claim → FR → test →
  result), including `unresolved_choice` tags and the carried unresolved ids
  (`display.q-side-effect`, `display.q-overload`, `display.q-return`).
  [SC-013-3; Principle VI]
