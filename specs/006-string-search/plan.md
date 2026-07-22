# Implementation Plan — 006-string-search

## Constitution Check

- I Specification-first: requirements cite approved claims — PASS
- II Clean-room: sources = imported export + RunMat code only; no MATLAB — PASS
- III Independence: algorithms are plain equality/prefix/token scans over
  RunMat's own text representations — PASS
- IV Provenance: `provenance.md` complete; interim pin waiver recorded — PASS
- V Test-first: observed-case tests authored with the builtin bodies gated
  red-first (tasks T1–T3 before T4–T6) — PASS
- VII Minimal scope: three tightly coupled string-search builtins, no
  refactoring — PASS
- IX Compatibility: follows `strings/search/` conventions (`contains.rs`,
  `startswith.rs`, `text_utils.rs`) and the `fileparts.rs` multi-output
  pattern — PASS

## Affected crates and files

| Item | Path |
|---|---|
| `matches` | `crates/runmat-runtime/src/builtins/strings/search/matches.rs` (new) |
| `strmatch` | `crates/runmat-runtime/src/builtins/strings/search/strmatch.rs` (new) |
| `strtok` | `crates/runmat-runtime/src/builtins/strings/search/strtok.rs` (new) |
| Registration | add `mod` entries in `strings/search/mod.rs` |
| Docs metadata | descriptor + `#[runtime_builtin]` doc fields (feeds `docs/builtins` tooling) |

Exemplars: `strings/search/contains.rs` (text-collection input handling,
IgnoreCase name-value parsing, logical result assembly, descriptor/GPU/fusion
spec layout), `strings/search/strfind.rs` (numeric index outputs),
`io/repl_fs/fileparts.rs` (multi-output via
`BuiltinOutputMode::ByRequestedOutputCount` + `crate::output_count`),
`logical/tests/iscell.rs` (provenance doc comment and test-tier comments).

## Design decisions

- `matches`: reuse `TextCollection`/`parse_ignore_case`/`logical_result`
  from `text_utils.rs`. Result shape = shape of the first argument; each
  subject element is compared for equality against every pattern element
  (`any` membership), lowercasing both sides when IgnoreCase is set. The
  observed rule covers scalar first arguments only; the element-wise
  generalisation is a documented choice (FR-006-03). Missing strings compare
  false. Type resolver: existing `logical_text_match_type`.
- `strmatch`: pattern restricted to a single text (char row vector, string
  scalar, 1-element string array); search array through
  `TextCollection::from_argument` (cellstr, char matrix rows, string
  arrays). Prefix mode uses `str::starts_with`; `'exact'` uses equality.
  Output `Value::Tensor` (double) with shape `[n, 1]` including `[0, 1]`
  for no match — matching the observed 2×1/1×1/0×1 sizes. Only the `'exact'`
  flag is accepted (other flags unobserved → error, documented choice). No
  legacy deprecation warning (strmatch.q-deprecated unresolved).
- `strtok`: input restricted to char row vector / string scalar (mirrors
  `fileparts`). `split_token` skips leading delimiters, collects the token,
  and returns the remainder starting at the terminating delimiter
  (strtok.q-remainder documented choice). Default delimiters:
  `char::is_whitespace`. Two outputs `[token, remain]` via
  `BuiltinOutputMode::ByRequestedOutputCount` + `crate::output_count`
  (fileparts pattern); single-output calls return only the token.
- All three: `GPU_SPEC`/`FUSION_SPEC` consts registered with
  `builtin_path`; host-only execution (text never resides on the GPU;
  `matches` gathers defensively like `contains`).

## Test plan (inline `#[cfg(test)]`, per house convention)

Normative (cite FR/claims in comments; `observed_*` names):
- matches: cases `in-set`, `scalar`, `ignore-case` → logical 1×1 true.
- strmatch: cases `prefix` → [1;2] (2×1), `exact` → 1 (1×1), `no-match` →
  0×1 empty double.
- strtok: cases `default` → 'hello' (1×5), `custom-delim` → 'a' (1×1),
  `leading-space` → 'lead' (1×4).
Summary-derived (tagged `summary_derived`): strtok two-output remainder
(`[tok, rem]` via `push_output_count(Some(2))` and the `split_token`
helper).
Unresolved-tracking (tagged `unresolved_choice`): matches array-first-arg
shape, default case sensitivity, missing strings, cell patterns; strmatch
char-matrix and string-array inputs, exact-vs-prefix distinction; strtok
empty/all-delimiter input, string input, non-space whitespace.
Error paths: invalid input types, invalid options/flags/delimiters.

## Risks & rollback

- Risk: `matches` result-shape semantics differ from broadcast semantics of
  `contains` — kept as a documented choice pinned by the observed scalar
  cases; revisit when the specification side observes array first
  arguments.
- Risk: WASM build needs registry regeneration — handled by macro
  `builtin_path`; verify with `cargo check -p runmat-runtime`.
- Rollback: delete the three new files + `mod.rs` entries (no existing file
  is modified beyond `mod.rs` line additions).

## Verification commands

`cargo fmt --all -- --check` · `cargo check -p runmat-runtime` ·
`RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib --
builtins::strings::search::matches builtins::strings::search::strmatch
builtins::strings::search::strtok`
