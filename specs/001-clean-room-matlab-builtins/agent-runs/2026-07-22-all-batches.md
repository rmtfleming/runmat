# Agent Run Ledger — 2026-07-22 — "Do all batches now" (B1–B5, features 005–010)

**Instruction**: maintainer directed all remaining batches proceed now
(recorded as blanket gate approval for features 005–009 through
implementation and validation; publication still excluded).

## Execution

- Features 005–009 assigned to five parallel implementation agents, each
  bound to the constitution, the imported approved specs, and the batch-1
  files as canonical templates:
  - 005-struct-cell-access: `fields`, `num2cell`
  - 006-string-search: `matches`, `strmatch`, `strtok`
  - 007-numeric-string-conversion: `int2str`, `str2num`
  - 008-date-formatting: `datestr`
  - 009-sparse-construction: `full`, `speye`, `spdiags`
- 010-table-conversion: **BLOCKED, not implemented** — RunMat has no table
  value type (B-010-1); feature bootstrapped with provenance and blocker
  documentation only. Surfaced per Principle VIII instead of inventing a
  table type as a side effect.

## Validation gate for agent output

Main session re-runs (not trusts): cargo check, module-filtered tests,
cargo fmt --check, clippy scan across all new modules before recording
results here.

## Outcome (verified by main session, not agent-reported)

- All five agents completed green. Main-session re-verification:
  `cargo check -p runmat-runtime` clean; **132/132 tests pass** across all
  features 002–009 (every observed export case reproduced exactly);
  `cargo fmt --all -- --check` clean; clippy clean after one main-session
  fix (type alias in `spdiags.rs` for a type-complexity denial; sparse tests
  re-run green afterwards).
- Implemented totals: 15 builtins, 16 new implementation files (incl.
  `math/sparse/mod.rs`), 10 registration-line edits to existing mod.rs /
  type_resolvers files. No other existing behaviour touched.
- Feature docs + validation.md present for 005–009 (agent-authored, format
  per 002); 010 blocked-documented.
- Notable findings recorded: `eval` delegation from runtime-internal calls
  is a dead path (VM-only interception) — str2num ships a self-contained
  expression parser, `str2num.q-eval` tracks it; generated WASM registry
  needs `scripts/regenerate-wasm-registry.sh` before the next wasm build
  (native unaffected); a `sparse` builtin already exists at
  `array/creation/sparse.rs` (batch-1 orientation note corrected).
- No commit, no push, no PR, no MATLAB invocation. matlab-interface-spec
  untouched.
