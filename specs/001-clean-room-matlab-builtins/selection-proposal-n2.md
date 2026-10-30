# Selection Proposal (FR-010) — Batch N2 — 2026-10-30

Candidates: the 14 specs imported as IMPORT-MANIFEST Batch 10 (source
`8f72edb`, all 0.2.0 `status: approved`). Freshness (2026-10-30): spec repo
`HEAD` = `origin/main` = `8f72edb`, no change to any N2 spec since import, no
drafts in the working tree.

Coverage check (this branch, which contains upstream `main` `282910ffb` and
`RM-1064-remaining`; no newer upstream branch carries builtin work):

- **Absent** (11): `alpha`, `hsv`, `tiledlayout`, `lastwarn`, `nthroot`,
  `textread`, `MException` (the `Value::MException` type exists, with no
  constructor builtin), `reallog`, `addnode`, `cast`, `categories` (the
  `categorical` constructor exists in `table/builtins/constructors.rs`).
  `lastwarn` can read the existing `warnings/policy.rs` `last_warning` state.
- **Already present as methods** (3): `keys`, `isKey`, `remove` are
  registered as `containers.Map.keys/isKey/remove` in
  `containers/map/containers.map.rs` (upstream code). The specs cover both
  `keys(M)` and `M.keys`, so this feature is **conformance work on upstream
  code**: spec-derived tests first, then minimal fixes for any divergence.

## Grouping (one coherent family per feature, Principle VII)

| # | Feature | Functions | Placement |
|---|---|---|---|
| F48 | 048-containers-map-conformance | `keys`, `isKey`, `remove` | tests next to `containers.map.rs`; fixes in place |
| F49 | 049-exceptions-warnings | `MException` (+ `throw`, `addCause`), `lastwarn` | `diagnostics/` |
| F50 | 050-numeric-conversion | `nthroot`, `reallog`, `cast` | `math/elementwise/` |
| F51 | 051-graphics-n2 | `hsv`, `alpha`, `tiledlayout` | `plotting/ops/` (`hsv` via the colormap macro list) |
| F52 | 052-data-n2 | `addnode`, `categories`, `textread` | `graph/domain/`, `table/`, `io/` |

Four parallel agents: F48+F49, F50, F51, F52.

## Flagged risks (existing policy: explicit `RunMat:<fn>:UnsupportedOption`
or a labelled documented choice, never asserted as MATLAB behaviour)

| Risk | Proposed disposition |
|---|---|
| F48 changes upstream-owned code | minimal diffs only; any behaviour change is listed in validation.md |
| `tiledlayout` layout model (`'flow'`, `nexttile` absent from N2) | grid via the existing subplot machinery; unsupported forms raise UnsupportedOption |
| `alpha` on renderers without transparency | store the property; rendering gaps documented as upstream gaps |
| `textread` format coverage | the observed conversions only; others UnsupportedOption |
| Generated parent `mod.rs` files | hand-added lines listed in the parallel_implementations README |

## Decision — RECORDED 2026-10-30 (maintainer)

- [x] Scope: all five features F48–F52 (maintainer: "All 5 features").
- [x] Gates G1–G3 pre-approved (maintainer: "Pre-approve G1–G3"). Stop
  before commit or push; local commits only, as for N1.
- [x] Upstream gaps remain documented only.
