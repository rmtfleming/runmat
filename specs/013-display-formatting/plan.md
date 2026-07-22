# Implementation Plan — 013-display-formatting

## Constitution Check

- I Specification-first: all normative FRs cite approved claims
  (`display.signature-primary`, `display.formatted-text`) — PASS
- II Clean-room: sources = batch-2 packaged export + RunMat code only; no
  MATLAB invocation anywhere (tests capture output via RunMat's own console
  buffer, not evalc) — PASS
- III Independence: formatting derived from the observed byte strings and
  RunMat's own helpers; no MATLAB internals imitated — PASS
- IV Provenance: `provenance.md` complete with authoritative commit pin
  `b054f3ad…34ef` (batch 2) — PASS (no waiver needed)
- V Test-first: exact-string tests authored before the builtin body (T2–T3
  before T4–T5) — PASS
- VI Traceability: FR ↔ test mapping enforced in tasks; validation.md will
  carry the table — PASS
- VII Minimal scope: one builtin, one new file + registration line; the two
  known `disp` divergences are surfaced as findings, NOT fixed globally —
  PASS
- VIII Human gates: this plan stops at Gate 2; no implementation before
  Gate 3 approval — PASS
- IX Compatibility: follows `io/disp.rs` conventions (sink, gather, macro
  registration, descriptor) — PASS
- X Licensing: no external code sources used — PASS

## Affected crates and files

| Item | Path |
|---|---|
| `display` builtin | `crates/runmat-runtime/src/builtins/io/display.rs` (new) |
| Registration | `crates/runmat-runtime/src/builtins/io/mod.rs` — add `pub mod display;` (module list is flat `pub mod` lines; the `runtime_builtin` macro with `builtin_path = "crate::builtins::io::display"` handles registry/inventory, as in `disp.rs`) |
| Helper visibility (only if delegation is used) | `io/disp.rs`: `fn format_for_disp` → `pub(crate) fn` (no behavioural change) |
| Type resolver | reuse `crate::builtins::io::type_resolvers::disp_type` (sink returning tensor type), as `disp` does |
| Docs metadata | descriptor + `#[runtime_builtin]` doc fields (feeds `docs/builtins` tooling) |

Exemplar: `io/disp.rs` (descriptor, GPU/fusion sink specs, error builder,
`gather_if_needed_async`, inline test module).

## How the exact observed strings are produced

`display` gets its own small, pure formatter — it does NOT reuse `disp`'s
table renderer for numerics, because `disp` demonstrably diverges from the
observed `display` output (see Risks):

- `pub(crate) fn format_display_text(value: &Value) -> String` returns the
  COMPLETE emitted text, including every trailing `\n`. This is the unit
  under test; byte-equality with the observed strings is asserted directly
  on the `String`, no console involved.
- Rendering rules (normative cases first, generalisations are documented
  choices per spec Unresolved):
  - Real numeric scalar (`Value::Num`, `Value::Int`, 1x1 tensor): field =
    `runmat_builtins::format_number` (default `FormatMode::Short` renders
    `42.0` → `"42"`), right-aligned to width ≥ 6 → `"    42"`; append
    `"\n\n"` (content line + blank line). Produces exactly `"    42\n\n"`.
  - Real numeric 2-D tensor: per matrix row, concatenate width-6 (minimum)
    right-aligned fields with NO separator between fields; column width =
    max(6, widest field in the column); newline per row; then one blank
    line. Produces exactly `"     1     2     3\n\n"` for `[1 2 3]`.
  - Char array (`Value::CharArray`): one line per row via the existing
    `strings::common::char_row_to_string`; no quotes, no indent, single
    trailing `\n`, NO blank line. Produces exactly `"hi\n"`.
  - Everything else (struct, cell, string array, complex, logical, sparse,
    objects, empties): delegate to `disp`'s `format_for_disp` lines joined
    with `\n`, plus trailing `"\n\n"` for non-char-like output (documented
    choice, `unresolved_choice`-tagged tests only; alternatively the plan's
    fallback is acceptable to simplify at Gate 2 review to “delegate +
    single \n”, since nothing here is normative).
- Builtin body (mirrors `disp_builtin`): reject a second argument with a
  descriptor-declared error (`RM.DISPLAY.ARG_UNSUPPORTED` — the two-argument
  form is `pending` in the export, spec Unresolved); gather GPU values via
  `gather_if_needed_async`; compute `format_display_text`; write it with
  `crate::console::record_console_output(ConsoleStream::Stdout, text)` —
  NOT `record_console_line`, so the recorded bytes are exactly the
  formatter's bytes (`record_console_line` appends `\n` only when missing,
  which would be a silent no-op here but blurs the byte-equality argument);
  return the 0x0 sink placeholder with `suppress_auto_output = true`.

## How tests capture output in-process (no evalc, no MATLAB)

Two layers, both already-proven patterns in this repo:

1. **Pure-formatter tests (primary, normative)**: assert
   `format_display_text(&value) == "    42\n\n"` etc. No console, no
   threading concerns; byte-exact by construction.
2. **Console-capture integration tests (one per observed case)**: the
   thread-local console buffer in `crates/runmat-runtime/src/console.rs` —
   `reset_thread_buffer()`, invoke `display_builtin(...)` via
   `futures::executor::block_on`, then `take_thread_buffer()` and join the
   entries' `text` fields; assert the joined text equals the observed
   string. Precedent: `builtins/math/optim/quad.rs`
   (`quad_trace_records_rows`) and `fzero.rs` use exactly this mechanism.
   Run with `RUST_TEST_THREADS=1` (house convention) or rely on the buffer
   being thread-local (it is — `runmat_thread_local!`), which isolates
   parallel tests.

## Risks — formatter divergence findings (surfaced, not silently patched)

These are pre-existing observations about `disp`'s renderer discovered while
planning; `display` works around them locally. They are findings for the
gate, NOT changes this feature makes:

1. **Inter-column separator**: `disp`'s `format_table` inserts a two-space
   separator between columns (`io/disp.rs`, `format_table`: `if c > 0 {
   line.push_str("  ") }`), so RunMat `disp([1 2 3])` renders
   `"     1       2       3"`. The observed `display` vector text is
   contiguous width-6 fields: `"     1     2     3"`. `disp`'s own unit test
   `numeric_matrix_right_aligned` locks in the separator behaviour.
   If MATLAB's `disp` matches the observed `display` layout (plausible but
   NOT covered by any approved export), RunMat's `disp` numeric layout
   diverges from MATLAB — recommend requesting a spec-side `disp`
   observation; out of scope here.
2. **Scalar indent**: RunMat `disp(42)` renders `"42"` (scalar path bypasses
   the table renderer, no indent). Observed `display(42)` text is
   `"    42"`. Same remark as (1) — likely a `disp` divergence too, needs a
   spec-side observation to confirm; `display` implements its own indent.
3. **Trailing blank line**: `disp` emits none (and `record_console_line`
   appends only a single `\n`). Observed `display` numeric texts end
   `"\n\n"`, the char text `"\n"`. `display` encodes this per-kind in its
   formatter.
4. **Spacing state**: the trailing blank line corresponds to MATLAB's
   default (loose) spacing; RunMat's `format` builtin accepts
   `compact`/`loose` as explicit no-ops (`io/format.rs` ~line 97). `display`
   hard-codes the observed default; if spacing state is ever implemented,
   `display` (and `disp`) must consult it — recorded in spec Unresolved.
5. **Non-default numeric formats**: `format_number` is thread-local-state
   dependent; tests must not assume another test changed `FormatMode`.
   Tests set/reset nothing and rely on the default; the `format` builtin's
   own tests already guard their state.

## Rollback

Delete `io/display.rs`, remove the `pub mod display;` line, revert the
`format_for_disp` visibility tweak (if taken). No other file is touched.

## Verification commands

`cargo fmt --all -- --check` · `cargo check -p runmat-runtime` ·
`RUST_TEST_THREADS=1 cargo test -p runmat-runtime display`
(disp's module must also still pass: `cargo test -p runmat-runtime disp`)
