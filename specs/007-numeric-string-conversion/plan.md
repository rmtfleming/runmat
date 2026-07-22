# Implementation Plan — 007-numeric-string-conversion

## Constitution Check

- I Specification-first: requirements cite approved claims — PASS
- II Clean-room: sources = imported export + RunMat code only; no MATLAB — PASS
- III Independence: rounding via Rust `f64::round`; layout derived from
  observed strings; `str2num` grammar is an original recursive-descent
  evaluator; full-language text delegates to RunMat's own `eval` — PASS
- IV Provenance: `provenance.md` complete; interim pin waiver recorded — PASS
- V Test-first: observed-case tests authored with the builtin bodies
  (tasks T2–T3 before/with T4–T5; red step gated by compile) — PASS
- VII Minimal scope: one tightly coupled conversion pair, no refactoring —
  PASS
- IX Compatibility: follows `strings/core/` conventions (`num2str`,
  `str2double` exemplars); `cargo fmt` clean — PASS

## Affected crates and files

| Item | Path |
|---|---|
| `int2str` | `crates/runmat-runtime/src/builtins/strings/core/int2str.rs` (new) |
| `str2num` | `crates/runmat-runtime/src/builtins/strings/core/str2num.rs` (new) |
| Registration | add `mod` entries in `strings/core/mod.rs` |
| Docs metadata | descriptor + `#[runtime_builtin]` doc fields (feeds `docs/builtins` tooling) |

Exemplars: `strings/core/num2str.rs` (descriptor/GPU spec/error builders,
column formatting, inline test module), `strings/core/str2double.rs`
(text-input handling, conversion GPU/fusion notes),
`logical/tests/iscell.rs` (clean-room file structure and test tiers).

## Design decisions

- `int2str`: gather GPU inputs, extract real numeric data (Num/Int/Bool/
  Tensor/LogicalArray; >2-D, complex and non-numeric rejected with
  `RunMat:int2str:InvalidInput` — documented choices on unobserved input
  classes). Round with `f64::round` (ties away from zero, matching observed
  `-2.5 -> -3`, `5.5 -> 6`). Layout derived from observed strings:
  per-column width = widest entry, columns right-aligned and joined by two
  spaces, rows of the char matrix = input rows (observed 1×7 and 2×4
  extents follow). Output `Value::CharArray`; type_resolver
  `string_scalar_type`; output mode `Fixed`.
- `str2num`: accept char row vectors (including `''`) and string scalars;
  reject other classes (`RunMat:str2num:InvalidInput`, documented choice).
  Evaluation order: (1) self-contained numeric grammar — scalar arithmetic
  expressions (`+ - * /`, right-associative `^`, unary signs, parentheses,
  decimal/scientific literals, `Inf`/`NaN` words) and bracketed
  row/matrix concatenation (rows split on `;`/newline, elements on commas
  or whitespace with an attached-sign heuristic); (2) delegation of
  out-of-grammar text to the registered `eval` builtin via
  `call_builtin_async_with_outputs("eval", …, 1)` — the runtime-side
  `eval` dispatch requires a VM workspace frame
  (`RunMat:DynamicWorkspaceRequiresVm` otherwise), so failures map to the
  documented empty result; (3) any failure or non-numeric evaluation
  result → 0×0 empty double (`str2num.q-eval` documented choice).
  Scalars return `Value::Num`, other shapes `Value::Tensor`
  (column-major); type_resolver `unknown_type` (result shape depends on
  runtime text).
- Both: `GPU_SPEC` (Custom("conversion"), GatherImmediately) and
  `FUSION_SPEC` (not fusable) registered per house convention.

## Test plan (inline `#[cfg(test)]`, per house convention)

Normative (cite FR/claims in test comments):
- int2str: observed cases scalar `3.7 -> '4'`, negative `-2.5 -> '-3'`,
  vector `[1.2 3.8 5.5] -> '1  4  6'` (1×7), matrix
  `[1.1 2.9; 3.5 4.4] -> ['1  3';'4  4']` (2×4).
- str2num: observed cases `'42' -> 42`, `'[1 2 3]' -> 1×3`,
  `'[1 2; 3 4]' -> 2×2`, `'2+3' -> 5`.
Summary-derived (tagged `summary_derived`): str2num unparsable → empty;
arithmetic precedence/grouping.
Unresolved-tracking (tagged `unresolved_choice`): int2str multi-width
alignment, NaN/Inf, empty input, integer/logical inputs, non-numeric
rejection; str2num comma separators, sign/whitespace tokenisation, `[]`,
inconsistent rows, identifier expressions without a VM frame, non-text
rejection, string-scalar acceptance.
Infrastructure: GPU gather round-trip (int2str), type-resolver checks.

## Risks & rollback

- Risk: `eval` delegation is VM-gated, so out-of-grammar text yields empty
  rather than full-language evaluation — documented under
  `str2num.q-eval`; revisit if the runtime gains a VM-eval bridge.
- Risk: WASM registry regeneration required before the next wasm release
  (`scripts/regenerate-wasm-registry.sh`); native builds unaffected.
- Rollback: delete the two new files + the two `mod` lines in
  `strings/core/mod.rs` (no other existing file is modified).

## Verification commands

`cargo fmt --all -- --check` · `cargo check -p runmat-runtime` ·
`RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::strings::core::int2str builtins::strings::core::str2num`
