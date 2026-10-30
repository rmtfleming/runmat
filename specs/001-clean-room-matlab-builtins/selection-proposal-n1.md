# Selection Proposal (FR-010) — Batch N1 — 2026-10-30

Candidates: the 14 specs imported as IMPORT-MANIFEST Batch 9 (source
`c65e238`, all 0.2.0 `status: approved`; approval record
`docs/review/2026-10-29-n1-new-specs-human-loop.md`). None is implemented by
upstream RunMat (main + RM-1064) or by this branch. Freshness: no source
revision since `c65e238` (checked 2026-10-30).

## Grouping (one coherent family per feature, Principle VII)

| # | Feature | Functions | Placement (hand-written parent where possible) |
|---|---|---|---|
| F42 | 042-sparse-orderings | `amd`, `etree`, `symbfact` | next to the clean-room `colamd`/`dissect` in `math/sparse/` |
| F43 | 043-graph-editing | `rmnode`, `addedge`, `rmedge`, `laplacian` | `graph/domain/` (next to F36 `structure.rs`) |
| F44 | 044-table-stack-innerjoin | `stack`, `innerjoin` | `table/builtins/` (next to F35 `joins.rs`) |
| F45 | 045-text-conversion | `regexptranslate`, `dec2base`, `validatestring` | `strings/…` (generated parents: add a line and list it in the README) |
| F46 | 046-isprop | `isprop` | `introspection/` (next to the clean-room `properties`) |
| F47 | 047-balance | `balance` | `math/linalg/factor/` (next to `svds`) |

## Flagged open questions (spec side: open questions, not requirements)

These follow the established policy: an explicit `RunMat:<fn>:UnsupportedOption`
or a clearly labelled documented choice, never asserted as MATLAB behaviour.

| Question | Proposed disposition |
|---|---|
| rmnode ignoring unknown node IDs | documented choice decided from the spec's observations; if they don't settle it, `UnsupportedOption` |
| innerjoin output names from caller variables | same callsite approach as F35 `outerjoin` (caller names when available, otherwise the observed fallback) |
| exact `balance` values | non-normative: tests assert structural properties (similarity, permutation/scaling form, power-of-2 scaling where the spec claims it) rather than exact entries |
| exact `amd` ordering | non-normative: tests assert a valid permutation, plus any observed properties the spec makes normative |
| `dec2base` char bases | `UnsupportedOption` |

## Decision — RECORDED 2026-10-30 (maintainer)

- [x] Start N1, gates G1–G3 pre-approved (maintainer: "Start N1, gates
  pre-approved"). Stop before commit or push; maintainer chose local
  commits only.
- [x] Upstream gaps remain documented only.
- N2 (Batch 10) is imported only and not selected.
