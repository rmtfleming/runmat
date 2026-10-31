# Selection Proposal (FR-010) — Batches N3 + N4 — 2026-10-30

Maintainer instruction: "implement N3 and N4".

Candidates: N3 (IMPORT-MANIFEST Batch 11, source `1156c97`; `throw` was
already implemented in N2 F49) and N4 (Batch 13, source `fc9f804`). All
0.2.0 `status: approved`. Freshness (2026-10-30): spec repo `HEAD` =
`origin/main` = `fc9f804`, no drafts in `specifications/`, and no N3 spec
changed since import.

Coverage check (this branch, which includes upstream `main`,
`RM-1064-remaining` and `fin/local-runtime-handoff-2026-09-24`; no other
upstream branch carries builtin work): none of the 27 names is implemented,
apart from `addCause`, which exists only as the N2 stub raising
`RunMat:addCause:UnsupportedOption` (D-049-1, documented only).

## Grouping (one coherent family per feature, Principle VII)

| # | Feature | Functions | Placement |
|---|---|---|---|
| F53 | 053-environment | `isunix`, `ismac`, `isMATLABReleaseOlderThan`, `chdir`, `date` | introspection / io/repl_fs / timing |
| F54 | 054-types-structs-text | `isfloat`, `isnat`, `struct2cell`, `strvcat` | logical/tests, datetime, structs, strings |
| F55 | 055-numeric-n3n4 | `sec`, `orth`, `condest` | math/elementwise/trigonometry, math/linalg |
| F56 | 056-table-variables | `renamevars`, `rows2vars`, `varfun`, `convertvars` | table/builtins |
| F57 | 057-text-parsing | `strread`, `asManyOfPattern` | strings / io |
| F58 | 058-graphics-n3n4 | `findall`, `ishghandle`, `allchild`, `pcolor`, `surfl` | plotting/ops |
| F59 | 059-graph-n3n4 | `isomorphism`, `allpaths`, `inedges` | graph/domain |
| F60 | 060-exception-causes | `addCause` (N4 spec) | diagnostics + core MException (see below) |

Six parallel agents: F53+F54, F55+F57, F56, F58, F59, F60.

## Flagged risks and non-normative claims

| Item | Proposed disposition |
|---|---|
| `addCause` causes (D-049-1) | needs a cause list on `runmat_value::MException`, the value codec and error propagation — maintainer decision |
| isunix/ismac/isMATLABReleaseOlderThan observed only on GLNXA64/R2026a | report the host platform; release comparison against `MATLAB_COMPATIBILITY_RELEASE` |
| orth basis, isomorphism permutation, allpaths order, condest value | non-normative: assert structural properties only |
| asManyOfPattern empty-match outputs | open question: UnsupportedOption or labelled choice |
| pcolor/surfl default EdgeColor | non-normative: documented choice |
| Generated parent `mod.rs` files | hand-added lines listed in the parallel_implementations README |

## Decision — RECORDED 2026-10-30 (maintainer)

- [x] Scope: all 27 functions, F53–F60 (maintainer: "implement N3 and N4").
- [x] F60: implement cause support (maintainer: "Implement cause
  support"), superseding the D-049-1 "document only" decision.
- [x] Gates G1–G3 pre-approved. Stop before commit or push; local commits
  only.
- [x] Other upstream gaps remain documented only.
