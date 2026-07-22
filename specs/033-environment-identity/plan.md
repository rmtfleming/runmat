# Implementation Plan — 033-environment-identity

## Constitution Check

- I Specification-first: contract requirements cite approved claims; RunMat
  identity values recorded as documented choices — PASS
- II Clean-room (NON-NEGOTIABLE): sources = imported export + RunMat code only;
  no MATLAB invoked; **MATLAB's observed identity values are deliberately NOT
  reproduced** (honesty) — PASS
- III Independence: version comparison is an independent numeric algorithm;
  identity taken from RunMat's own crate metadata — PASS
- IV Provenance: `provenance.md` complete; commit pin
  `1b019121bb747e63d69a3eb005534cb60669e5f4`; claims split into
  contract-claim (honored) vs MATLAB-value-claim (not asserted) — PASS
- V Test-first: acceptance criteria authored as `contract_*` tests with the
  builtin files; identity divergence encoded in tests — PASS
- VI Traceability: FR ↔ claim ↔ test mapping in `validation.md` — PASS
- VII Minimal scope: one tightly coupled family (environment identity), no
  refactoring — PASS
- IX Compatibility: follows `introspection/` conventions (descriptor, macro
  registration, GPU/fusion metadata, inline tests) exactly like `computer` — PASS
- X Licensing: no MathWorks material; `license` builtin reports RunMat's own
  open-source availability, not a copy of MATLAB licensing state — PASS

## Affected crates and files

| Item | Path |
|---|---|
| `version` | `crates/runmat-runtime/src/builtins/introspection/version.rs` (new) |
| `ver` | `crates/runmat-runtime/src/builtins/introspection/ver.rs` (new) |
| `verLessThan` | `crates/runmat-runtime/src/builtins/introspection/verLessThan.rs` (new; module `ver_less_than` via `#[path]`, mirroring `getReport.rs`) |
| `license` | `crates/runmat-runtime/src/builtins/introspection/license.rs` (new) |
| Registration | add `pub mod license;`, `pub mod ver;`, `#[path="verLessThan.rs"] pub mod ver_less_than;`, `pub mod version;` to `introspection/mod.rs` |

Category `introspection`: these report runtime identity, exactly like the
sibling `computer` builtin (specs/019-computer), which is the primary exemplar
(descriptor + GPU/fusion spec + `args: Vec<Value>` dispatch + tiered inline
tests + `option_text` helper).

Module-naming note: `verLessThan.rs` keeps the export's filename; the Rust
module is `ver_less_than` (snake_case, no `non_snake_case` lint) via
`#[path = "verLessThan.rs"]`, the established RunMat convention (`getReport.rs`
→ `get_report`). `builtin_path` in the macro uses the module path
`crate::builtins::introspection::ver_less_than`.

## Design decisions

- **Shared identity constants** live in `version.rs`: `RUNMAT_VERSION`
  (`env!("CARGO_PKG_VERSION")` = `0.5.6`), `RUNMAT_RELEASE` (`R2026a-compat`),
  and the reusable `option_text` text-extraction helper. `ver`/`verLessThan`/
  `license` import from there to keep identity single-sourced and honest.
- `version`: no-arg → `'0.5.6 (RunMat)'` (mirrors the observed
  number+parenthetical **shape** with RunMat content); `'-release'` →
  `R2026a-compat`; other option → `RunMat:version:InvalidOption`; >1 arg →
  `RunMat:version:TooManyInputs`. Output `Value::CharArray`, resolver
  `Type::String`.
- `ver`: no-arg / `'RunMat'` → `Value::Struct{Name,Version,Release,Date}`;
  `'MATLAB'`/unknown → empty struct array (`CellArray::new(vec![],0,1)`,
  RunMat's `dir`-style empty representation) — **no fabricated MATLAB entry**;
  non-text → `RunMat:ver:InvalidProduct`; >1 arg →
  `RunMat:ver:TooManyInputs`. Resolver `Type::Struct`.
- `verLessThan`: independent `parse_version` (leading digits per dotted
  component) + `version_less_than` (zero-padded component compare) over
  RunMat's `0.5.6`; `toolbox` is a documented mapping (every name → RunMat's
  single version); output `Value::Bool`, resolver `Type::Bool`. **MATLAB's
  observed booleans are not asserted** — `verLessThan('matlab','1.0')` is
  `true` for RunMat (diverges from MATLAB's `false`).
- `license`: `('test',feature)` → `Value::Num(1.0)` (open-source availability,
  the contract value); no-arg → `'RunMat'` (char); `'inuse'` → empty struct
  array; `'checkout'|'checkin'` → `1`; else → `RunMat:license:InvalidUsage`.
  Resolver `Type::Num`.
- GPU/fusion metadata: host-only identity queries; `GpuOpKind::Custom
  ("introspection")`, empty precision list, `GatherImmediately`, not fusible;
  no `accel` tag (mirrors `computer`).
- Output mode `Fixed` (single output) for all four.

## Test plan (inline `#[cfg(test)]`, per house convention)

Tiers (deliberately NOT `observed_*`, which would imply MATLAB-value
conformance):

- `contract_*` — assert the observable contract (class/shape) **with RunMat's
  own values**: `version` char contains `0.5.6`; `version('-release')` char ==
  `R2026a-compat` (and `!= '2026a'`); `ver` struct with RunMat fields;
  `verLessThan` 1×1 logical with RunMat-correct results incl. the `1.0`
  divergence test; `license('test',…)` double `1`.
- `documented_choice_*` — RunMat-specific choices (non-normative): option
  case-insensitivity, error identifiers, `ver('MATLAB')` empty, toolbox-name
  independence, pure comparator semantics, `license` no-arg/`inuse`/`checkout`.
- Registration/metadata: `*_is_registered`, `descriptor_and_spec_metadata_
  consistent`, `*_type_is_*`.

Honesty guards are asserted (`!text.contains("R2026a) Update")`,
`Version != '26.1.0.3276743 …'`, `release != '2026a'`).

## Risks & rollback

- Risk: concurrent Tier B siblings touch `introspection/mod.rs` — changes here
  are additive `pub mod` lines; trivially mergeable. (Handled: siblings added
  `methods`/`properties` during this work; merged cleanly.)
- Risk: sibling compile error blocks the build — per programme rule, waited and
  re-ran rather than editing another feature's file (`what.rs` cleared).
- Rollback: delete the four new files and the `mod` lines (no other file
  changed).

## Verification commands

`cargo check -p runmat-runtime` ·
`RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::introspection::ver builtins::introspection::license`
· `cargo fmt` then `cargo fmt --all -- --check` ·
`cargo clippy -p runmat-runtime --lib`
