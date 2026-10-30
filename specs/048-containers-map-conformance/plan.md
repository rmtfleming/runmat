# Implementation Plan — 048-containers-map-conformance (specs 0.2.0)

## Constitution Check

- I Specification-first: FR-048-01..12 cite the 18 approved normative
  claims; U-048-1..3 are labelled documented choices; FR-048-14 carries the
  unresolved questions. PASS.
- II Clean-room: inputs are the imported exports and the upstream RunMat
  map code; MATLAB not invoked. PASS.
- III Independence: fixes are local control-flow changes in upstream code.
  PASS.
- IV Provenance: `provenance.md` pinned to `8f72edb`, freshness-checked,
  claim ids verified against the imported `provenance.yaml`. PASS.
- V Test-first: conformance tests written and run against the unchanged
  upstream code; the failing run is recorded in `validation.md` before any
  fix. PASS.
- VII Minimal scope: one new test file, a `#[cfg(test)]` module line in the
  hand-written `containers/map/mod.rs`, and minimal edits inside
  `containers.map.rs` (no restructuring); every upstream behaviour change is
  listed in `validation.md`. PASS.
- IX Compatibility: existing `#[runtime_builtin]` registrations, warning
  emission through `crate::warnings::emit`. No `integer_capabilities`
  changes. PASS.

## Affected files

| Item | Path |
|---|---|
| Conformance tests (new) | `crates/runmat-runtime/src/builtins/containers/map/conformance_tests.rs` |
| Test module registration | `crates/runmat-runtime/src/builtins/containers/map/mod.rs` (hand-written; `#[cfg(test)] #[path] mod map_conformance_tests;`) |
| Minimal fixes (upstream) | `crates/runmat-runtime/src/builtins/containers/map/containers.map.rs` |

## Design

- Tests call the public routes: `call_builtin_async("keys"|"isKey"|"remove",
  …)` (registered-instance-method dispatch), `"containers.Map.keys"`
  (qualified), `object::resolve::load_member(M, "keys")` (`M.keys`) and
  `object::dispatch::call_method_or_member_index_named_with_outputs`
  (`M.keys()`). Warnings are read from `warnings::with_policy(..last_warning)`.
- Expected divergences (to be confirmed by the failing run):
  - `isKey` with a char key on a numeric map errors (FR-048-08) → a new
    helper `key_category_mismatch` (text vs numeric) counts as not present
    (U-048-3); numeric-class mismatches keep upstream's error.
  - `isKey` with a 2-D cell: upstream iterates the cell's row-major payload
    but builds a column-major `LogicalArray` (FR-048-06) → iterate the cell
    in column-major order.
  - `remove` of an absent key raises `RunMat:containers.Map:MissingKey`
    (FR-048-12) → skip absent/mismatched keys, remove the rest, then emit one
    `RunMat:Containers:Map:NoKeyToRemove` warning (U-048-2).
  - `M.keys` member form: `containers.Map.subsref` rejects the member
    `keys` as an unknown property → return `keys(M)` for the member `keys`.

## Test plan (`map_conformance_tests`)

- `observed_*`: every observed case of the three exports.
- `unresolved_choice_*`: U-048-1 (non-map), U-048-2 (warning in error
  mode), U-048-3 (mismatch inside a cell).

## Risks & rollback

- The `remove` change alters upstream behaviour (error → warning); the
  upstream unit test `remove_missing_key_returns_error` asserts the old
  behaviour and is updated to FR-048-12 (reported).
- Rollback: revert the edits in `containers.map.rs`, delete
  `conformance_tests.rs` and its `mod` line.

## Verification commands

```
export HDF5_DIR=/home/rfleming/anaconda3
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=<scratchpad>/run.sh
cargo test -p runmat-runtime --lib -- map_conformance_tests
cargo test -p runmat-runtime --lib -- builtins::containers
rustfmt --edition 2021 --check crates/runmat-runtime/src/builtins/containers/map/{conformance_tests.rs,containers.map.rs,mod.rs}
cargo clippy -p runmat-runtime --lib --tests --message-format short 2>&1 | grep containers
```

## Analysis (B3)

Cross-artifact check of spec.md, provenance.md, plan.md, tasks.md:

- Every normative claim id in the three imported `provenance.yaml` files is
  cited by exactly one FR (18/18); the only non-normative claim
  (keys.non-map) backs U-048-1 only.
- Every FR (01..14) is covered by a planned test group; every task in
  tasks.md cites an FR; no orphan tasks.
- Identifier mapping (`MATLAB:Containers:Map:NoKeyToRemove` →
  `RunMat:Containers:Map:NoKeyToRemove`) is consistent in spec, provenance
  and tests.
- No inconsistencies found; no unresolved question is presented as
  normative.
