# Implementation Plan — 020-strncmpi

## Constitution Check

- I Specification-first: requirements cite approved claims from the
  `strncmpi` 0.2.0 export — PASS
- II Clean-room: sources = imported export + RunMat code only; no MATLAB
  invocation anywhere in the workflow — PASS
- III Independence: implementation derived from RunMat's own `strncmp`
  (prefix walk) and `strcmpi` (lowercase fold) machinery; no MATLAB
  internals inferred — PASS
- IV Provenance: `provenance.md` complete with commit pin
  `e46587ae1b78237e0f43cd55f678de01a28495a4` — PASS
- V Test-first: observed-case tests authored with the module before the
  behaviour is finalised; every reported result comes from an actual run —
  PASS
- VI Traceability: claims → FRs → tests table produced in `validation.md` —
  PASS
- VII Minimal scope: one builtin, no refactoring of shared utilities — PASS
- VIII Human gates: Gates 1–3 approved for Tier B batch 1; implementation
  authorized; publication (commit/push/PR) NOT authorized — PASS
- IX Compatibility: follows `strings/core/strncmp.rs` exemplar (descriptor,
  GPU/fusion specs, macro registration, inline tests) — PASS
- X Licensing: no external implementation sources used — PASS

## Affected crates and files

| Item | Path |
|---|---|
| `strncmpi` builtin | `crates/runmat-runtime/src/builtins/strings/core/strncmpi.rs` (new) |
| Registration | add `pub mod strncmpi;` to `crates/runmat-runtime/src/builtins/strings/core/mod.rs` |
| Docs metadata | descriptor + `#[runtime_builtin]` doc fields (feeds `docs/builtins` tooling) |

Exemplar: `strings/core/strncmp.rs` (structure, input decoding, prefix
walk, broadcasting, error family, GPU/fusion specs, test shapes).
Case-folding exemplar: `strings/core/strcmpi.rs` (lowercase comparison).

## Design decisions

- Mirror `strncmp.rs` exactly: gather GPU operands
  (`gather_if_needed_async`, `accel = "sink"`), parse the prefix length with
  the same numeric/scalar rules, decode both text arguments through
  `TextCollection::from_argument`, broadcast shapes with
  `broadcast_shapes`/`broadcast_index`, and assemble the output with
  `logical_result` (scalar → `Value::Bool`, otherwise
  `Value::LogicalArray`).
- Case-insensitive comparison: per-character walk as in `strncmp`'s
  `prefix_equal`, but characters compare equal when identical or when their
  Unicode lowercase expansions are equal (`char::to_lowercase`), the same
  fold `strcmpi` applies via `str::to_lowercase`.
- Length rule copied verbatim from `strncmp`: `n = 0` → true; per-character
  mismatch within `n` → false; one string exhausted while the other
  continues within `n` → false; both exhausted simultaneously before `n` →
  true.
- Missing string elements: false when `n > 0`, true when `n = 0` (as
  `strncmp`).
- Errors: `RunMat:strncmpi:InvalidInput`, `RunMat:strncmpi:ShapeMismatch`,
  `RunMat:strncmpi:InvalidPrefixLength`, `RunMat:strncmpi:InternalError`
  (mirror of the `strncmp` family with the `RM.STRNCMPI.*` codes).
- GPU spec: `GpuOpKind::Custom("string-prefix-compare-fold")`, gather
  immediately, no provider hooks; fusion spec: not fusible (logical host
  results). Type resolver: `logical_text_match_type`.

## Test plan (inline `#[cfg(test)]`, per house convention)

Normative tier `observed_*` (each reproduces one export case exactly and
cites claims/FRs in comments):
- `observed_match_first_three_characters_ignoring_case_true`
- `observed_prefix_equal_first_two_characters_true`
- `observed_no_match_first_character_false`

Unresolved-choice tier `unresolved_choice_*` (documented independent
choices; non-normative): mismatch within prefix; shorter string within
prefix; both strings exhausted before `n`; `n = 0`; logical/tensor/bool
prefix-length scalars; char-array rows; cell-array broadcast
[strncmpi.q-arrays]; string-array broadcast [strncmpi.q-arrays]; missing
strings (`n > 0` and `n = 0`); shape-mismatch, invalid-length,
negative-length and invalid-input errors.

Infrastructure: wgpu-gated GPU prefix-length gather test; type-resolver
check (`logical_text_match_type` → `Type::Bool`).

## Risks & rollback

- Risk: concurrent sibling features touch `strings/core/mod.rs` — the
  change is a single added line; conflicts resolve trivially. Compile
  errors outside this feature's files are left to their owning feature.
- Risk: Unicode multi-char lowercase expansions (e.g. 'İ') differ from
  MATLAB — unobserved; covered by the documented-choice tier only.
- Rollback: delete `strncmpi.rs` and the one `mod.rs` line (no other file
  is modified).

## Verification commands

`cargo check -p runmat-runtime` ·
`RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::strings::core::strncmpi` ·
`cargo fmt` then `cargo fmt --all -- --check`
