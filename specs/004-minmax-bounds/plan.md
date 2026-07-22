# Implementation Plan — 004-minmax-bounds

## Constitution Check
PASS profile as 002/003 (single builtin; test-first; reuses existing
reduction machinery per Principle IX; interim-pin waiver applies).

## Affected crates and files

| Item | Path |
|---|---|
| `bounds` | `crates/runmat-runtime/src/builtins/math/reduction/bounds.rs` (new) |
| Registration | `math/reduction/mod.rs` module entry |

Exemplars: `math/reduction/max.rs` and `min.rs` (dim handling, output modes,
GPU/fusion specs, `output_list_with_padding`).

## Design decisions

- Delegate elementwise work to the same host/GPU reduction paths `min`/`max`
  use (call their internal helpers if exported, else mirror their kernel
  invocation) — no new numeric algorithm (Principle III/IX).
- Signatures: `bounds(A)`, `bounds(A, dim)`. No option strings (FR-004-05).
- Output mode `ByRequestedOutputCount`: nargout 1 → lo; 2 → (lo, hi).
- GPU inputs: follow `min`/`max` provider hooks; if their spec gathers, we
  gather — consistency over novelty.

## Test plan

Normative: vector → scalar 1; matrix default → [1 2] (1×2); dim=2 → [1;3]
(2×1); class double. Summary-derived: hi values/shapes for the same cases.
Unresolved-tracking: NaN input (documents inherited kernel behaviour),
empty input, int32 input.

## Risks & rollback

- Risk: `min`/`max` helpers not factored for reuse — mitigation: minimal
  local wrapper calling both reductions; no refactor of existing files
  beyond a possible `pub(crate)` visibility change (flagged in review if
  needed).
- Rollback: remove file + mod entry (+ revert any visibility line).

## Verification

`cargo fmt --all -- --check` · `cargo check -p runmat-runtime` ·
`RUST_TEST_THREADS=1 cargo test -p runmat-runtime bounds`
