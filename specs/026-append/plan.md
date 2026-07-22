# Implementation Plan — 026-append

## Constitution Check

- I Specification-first: normative requirements (FR-026-01..03) cite approved
  claims `append.signature-primary`, `append.concatenates-text` — PASS
- II Clean-room: sources = imported `append` export + RunMat code only; no
  MATLAB invoked — PASS
- III Independence: implementation derived from RunMat's `Value` model and the
  shared broadcast helpers; no MATLAB internals imitated — PASS
- IV Provenance: `provenance.md` complete; revision pinned to source commit
  `4445294` — PASS
- V Test-first: acceptance criteria and tests authored alongside the builtin;
  observed cases encoded as normative tests, unobserved as tagged choices — PASS
- VI Traceability: FR ↔ claim ↔ test mapping in `validation.md` — PASS
- VII Minimal scope: one builtin (`append`); no unrelated refactoring; cellstr
  deliberately out of scope — PASS
- IX Compatibility: follows the `strcat` exemplar (descriptor, GPU/fusion spec,
  `#[runtime_builtin]` macro, inline test module); uses RunMat `Value` — PASS
- X Licensing: no external implementation code imported — PASS

## Affected crates and files

| Item | Path |
|---|---|
| `append` builtin | `crates/runmat-runtime/src/builtins/strings/transform/append.rs` (new) |
| Registration | `mod append;` in `crates/runmat-runtime/src/builtins/strings/transform/mod.rs` |
| Docs metadata | descriptor + `#[runtime_builtin]` doc fields (feeds `docs/builtins` tooling) |

Closest exemplar: `strings/transform/strcat.rs` (string/char concatenation,
variadic, text-extraction and output-class handling, GPU/fusion specs, inline
tests). `append` is a simplified sibling: two operand classes instead of three
(no cell), and **no** trailing-space trimming.

## Design decisions

- Variadic signature `append(str1, str2, ...)`; internal handler takes
  `rest: Vec<Value>`. Zero args → `RunMat:append:NotEnoughInputs`.
- `TextOperand::from_value` accepts `Value::String`, `Value::StringArray`,
  `Value::CharArray`; every other kind → `RunMat:append:InvalidInput`
  (documented choice). GPU tensors are gathered first (`gather_if_needed_async`)
  and then fail the type check, matching `strcat`.
- Char arrays are normalised to one text piece per row, shape `[rows, 1]`,
  **without** trimming trailing whitespace (the key difference from `strcat`).
- Output class: `OutputKind` starts `Char`; any string operand promotes it to
  `String`. String result that is scalar → `Value::String`; non-scalar →
  `Value::StringArray`; all-char → `Value::CharArray` row(s).
- Concatenation reuses `broadcast_shapes` / `broadcast_index` / `compute_strides`
  from `builtins/common/broadcast` (MATLAB implicit expansion, column-major),
  giving element-wise string-array support for free.
- `output_mode = Fixed`; `type_resolver = text_concat_type` (reused: string →
  `Type::String`, otherwise `Unknown`, since RunMat has no distinct char Type).
  `accel = "sink"` with host-only GPU and fusion specs (string transforms are
  not GPU/fusion eligible).
- Missing-string sentinels are **not** special-cased (plain text concatenation);
  unobserved for `append`, documented choice.

## Test plan (inline `#[cfg(test)]`, per house convention)

Observed (normative, names prefixed `observed_`):
- `observed_two_strings_return_string_scalar` — `append("a","b") → "ab"`.
- `observed_three_strings_return_string_scalar` — `append("x","y","z") → "xyz"`.
- `observed_two_chars_return_char_row` — `append('a','b') → 'ab'` `[1 2]`.

Documented choices (non-normative, names prefixed `unresolved_choice_`):
- trailing whitespace preserved on char inputs;
- mixed char+string → string;
- single string argument returns itself;
- string array element-wise concatenation (implicit expansion);
- empty string array propagates empty shape;
- zero args / non-text input / mismatched sizes → the respective errors.

Plus a type-resolver assertion and a `wgpu`-gated GPU-operand type-error test
(mirrors `strcat`).

## Risks & rollback

- Risk: concurrent sibling features (Tier B batch 2) share the crate; a compile
  error outside `append.rs` blocks the build. Mitigation: wait and re-run, do
  not edit sibling files.
- Risk: char broadcast semantics for multi-row char inputs are unobserved.
  Mitigation: mirror `strcat`'s row-wise model; assert only scalar observed
  cases as normative.
- Rollback: delete `append.rs` and the single `mod append;` line — no existing
  file is otherwise modified.

## Verification commands

`cargo check -p runmat-runtime` ·
`RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::strings::transform::append` ·
`cargo fmt` then `cargo fmt --all -- --check`
