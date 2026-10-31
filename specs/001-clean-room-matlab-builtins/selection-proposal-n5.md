# Selection Proposal (FR-010) — Batch N5 — 2026-10-31

Maintainer instruction: "then import N5 and implement it".

Candidates: N5, imported as IMPORT-MANIFEST Batch 14 (source `1c91aa1`, all
0.2.0 `status: approved`). Freshness (2026-10-31): spec repo `HEAD` =
`origin/main` = `1c91aa1`, no drafts in `specifications/`, and no N3 or N4
spec changed since import (`c3305a5` touches `tools/` only).

Coverage check (this branch and every upstream branch with commits not
on it: `main`, `0-6-2`, `RM-1064-integrate-terminal-25ab`,
`continue-meshing-work`, `end-july-work`,
`fin/local-runtime-handoff-2026-09-24`; the rest are dependabot or docs
branches): none of the 11 names is registered.

## Grouping (one coherent family per feature, Principle VII)

| # | Feature | Functions | Placement |
|---|---|---|---|
| F61 | 061-sparse-linalg-n5 | `lsqminnorm`, `sprandn`, `sprank` | math/linalg/solve, array/creation/sparse |
| F62 | 062-graph-n5 | `outedges`, `reordernodes` | graph/domain |
| F63 | 063-rat | `rat` | math/elementwise |
| F64 | 064-file-io-n5 | `fscanf`, `urlwrite` | io/filetext, io/http |
| F65 | 065-graphics-n5 | `fplot`, `rectangle`, `validatecolor` | plotting/ops |

Four parallel agents: F61, F62+F63, F64, F65.

## Flagged risks and open questions (spec §8.2)

| Item | Disposition |
|---|---|
| `urlwrite.q-network` (https unobserved) | maintainer: "Use webread transport" — http(s) through `io/http/transport.rs`, a labelled choice, not observed behaviour |
| `lsqminnorm.q-tol-rule` | follow the observed case (column-pivoted QR truncated at the numerical rank); labelled inference |
| `sprandn.q-values` | structure, class, density bound and seeded repeatability only; values not asserted |
| `rat.q-complex` | complex input supported as observed |
| `fscanf.q-mixed` | character-code result reproduced as observed |
| `fplot.q-theme`, `rectangle.q-theme` | default colours/LineWidth are documented choices |
| `q-error-ids` naming other functions | raised as observed, under the `RunMat:` prefix |
| Generated parent `mod.rs` files | hand-added lines listed in the parallel_implementations README |

## Decision — RECORDED 2026-10-31 (maintainer)

- [x] Scope: all 11 functions, F61–F65 ("import N5 and implement it").
- [x] Gates G1–G3 pre-approved ("Pre-approve G1–G3"). Local commits only.
- [x] `urlwrite` http(s): "Use webread transport".
- [x] Other upstream gaps remain documented only.
