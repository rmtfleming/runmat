# Implementation Plan — 018-dec2bin

## Constitution Check

- I Specification-first: requirements cite approved claims — PASS
- II Clean-room: sources = imported export + RunMat code only; no MATLAB — PASS
- III Independence: standard binary formatting + house patterns; no MATLAB
  internals inferred — PASS
- IV Provenance: `provenance.md` complete with commit pin
  `e46587ae1b78237e0f43cd55f678de01a28495a4` — PASS
- V Test-first: observed cases authored as tests before the conversion body
  (tasks T2 before T3) — PASS
- VII Minimal scope: one builtin, no refactoring — PASS
- IX Compatibility: follows `strings/core` conventions (`int2str` exemplar) —
  PASS

## Affected crates and files

| Item | Path |
|---|---|
| `dec2bin` | `crates/runmat-runtime/src/builtins/strings/core/dec2bin.rs` (new) |
| Registration | add `pub mod dec2bin;` to `crates/runmat-runtime/src/builtins/strings/core/mod.rs` |
| Docs metadata | descriptor + `#[runtime_builtin]` doc fields (feeds `docs/builtins` tooling) |

Exemplar: `strings/core/int2str.rs` (descriptor, GPU/fusion spec, error
builders, gather-then-format flow, inline test module). Optional-argument
handling follows `strings/core/num2str.rs` (`rest: Vec<Value>`).

## Design decisions

- Signatures: `str = dec2bin(D)` and `str = dec2bin(D, n)` (two descriptor
  signatures; the macro function takes `(Value, Vec<Value>)`).
- Element extraction mirrors `int2str`: `Num`/`Int`/`Bool`/`Tensor`/
  `LogicalArray` accepted; each element must be finite; non-integers round to
  nearest (ties away from zero, documented choice); negatives and values
  above 2^53 error (`RunMat:dec2bin:InvalidInput`).
- Width: absent → 0 (natural width). Must be a finite nonnegative real
  numeric scalar; rounded to nearest integer; otherwise
  `RunMat:dec2bin:InvalidWidth`. More than two arguments → same identifier.
- Formatting: binary digits via Rust `{:b}` (no leading zeros; `0` → `"0"`),
  then left `'0'`-padding to `max(requested, widest natural, 1)`. One row
  per element in column-major element order; empty input → 0×0 char array
  (documented choices for dec2bin.q-vector).
- GPU: `gather_if_needed_async` before formatting; GPU spec residency
  `GatherImmediately`, op kind `Custom("conversion")`; fusion spec: not
  eligible (same as `int2str`).
- Output mode `Fixed`, type_resolver `string_scalar_type`.

## Test plan (inline `#[cfg(test)]`, per house convention)

Normative (cite FR/claims in test comments), tier `observed_*`:
- scalar 5 → '101' (1×3); padded 5,8 → '00000101' (1×8); zero → '0' (1×1);
  width-below-natural 5,2 → '101' (1×3).
Unresolved-tracking, tier `unresolved_choice_*`:
- vector rows share common width; width pads every row; negative input
  errors; non-integer rounds; non-finite errors; empty → 0×0; integer and
  logical inputs convert; invalid width arguments error; non-numeric input
  errors.
Infrastructure: GPU roundtrip via test provider; registry discoverability;
type-resolver check.

## Risks & rollback

- Risk: width-argument arity handling differs from macro expectations —
  mitigated by following `num2str`'s `rest: Vec<Value>` pattern; verified by
  `cargo check -p runmat-runtime`.
- Risk: char matrix layout (row-major `CharArray.data`) — matches
  `int2str`'s assembly and is covered by shape assertions in tests.
- Rollback: delete `dec2bin.rs` + the one `mod.rs` line (no other file is
  modified).

## Verification commands

`cargo fmt --all -- --check` · `cargo check -p runmat-runtime` ·
`RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::strings::core::dec2bin`
