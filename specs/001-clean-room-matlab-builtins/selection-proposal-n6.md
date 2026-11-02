# Selection Proposal (FR-010) — Batch N6 — 2026-11-02

Candidates: N6, imported as IMPORT-MANIFEST Batch 15 (source `e364206`, all
0.2.0 `status: approved`). Freshness (2026-11-02): spec repo `HEAD` =
`origin/main` = `e364206`; no N1–N5 spec changed since import.

Coverage check (this branch @ `87095ae0c`, which includes upstream `main`
and every builtin-carrying upstream branch): none of `subgraph`,
`findnode`, `normest`, `lsqnonneg`, `web` is registered.

## Grouping (Principle VII)

| # | Feature | Functions | Placement |
|---|---|---|---|
| F66 | 066-graph-n6 | `subgraph`, `findnode` | graph/domain |
| F67 | 067-numeric-n6 | `normest`, `lsqnonneg` | math/linalg/solve, math/optim |
| F68 | 068-web | `web` | io/http or io/repl_fs (environment) |

Two parallel agents: F66, F67+F68.

## Open questions (spec §8.2) and dispositions

| Item | Disposition |
|---|---|
| `normest.q-exactness`, `q-default-tol` | accuracy only: tests compare with `norm(full(S))` within tolerance; iteration count not asserted |
| `lsqnonneg` `output.algorithm`/`message`, `q-nonunique` | fields present; values are documented choices; tests assert KKT optimality and objective, not a particular minimiser where it is not unique |
| `subgraph.q-tie-order` | edge table sorted by renumbered `EndNodes` (normative); tie order a documented choice |
| `web` `q-network`, `q-display`, `q-handle`, `q-headless`, `q-stat` | observed `file://` outputs reproduced; launching a browser goes through an injectable launcher (never in tests); headless behaviour a labelled choice |
| `q-error-ids` naming other functions | raised as observed under `RunMat:` |

## Decision — RECORDED 2026-11-02 (maintainer)

- [x] Scope: all five ("Import and implement").
- [x] Gates G1–G3 pre-approved, as for N2–N5. Commit locally only.
