# Selection Proposal (FR-010) — Tier B batch 1 — 2026-07-22

Candidates: the 12 approved specifications imported from source commit
`e46587a` (see IMPORT-MANIFEST.md batch-3 section). FR-002 eligibility fully
satisfied (approved, commit-pinned, export-boundary artefacts). None exist in
RunMat today (verified). Substrate confirmed: `strncmp.rs`, `gamma.rs`,
`SparseTensor`.

## Proposed grouping

| # | Feature | Function(s) | Substrate | Scope | Status |
|---|---|---|---|---|---|
| F16 | 016-shape-storage-predicates | `isrow`, `iscolumn`, `issparse`, `iscellstr` | logical/tests; shape + storage + cell-content checks | Small | Ready |
| F17 | 017-nonzeros | `nonzeros` | SparseTensor + Tensor, column-major extract | Small | Ready |
| F18 | 018-dec2bin | `dec2bin` | strings/core (near int2str) | Small | Ready |
| F19 | 019-computer | `computer` | platform introspection (std::env::consts) | Small | Ready |
| F20 | 020-strncmpi | `strncmpi` | strings/core, mirror `strncmp.rs` | Small | Ready |
| F21 | 021-psi | `psi` | math special function (digamma; independent algorithm, alongside gamma.rs) | Small–medium | Ready |
| — | (table-blocked) | `addvars`, `removevars` | **No table type (B-010-1)** | — | BLOCKED |
| — | (decision needed) | `istable` | logical predicate whose observed true-case needs a table type | Small | See note |

## `istable` note (gate decision)

`istable` returns a logical scalar; observed `true` only for a `table` input,
`false` for numeric/cell. RunMat has no table type, so no constructible value
can ever be a table. `istable` is therefore **implementable now** as a
predicate that returns `false` for every current value kind — correct for all
inputs RunMat can construct — with a documented note that the observed
`true`-case is unreachable until a table type exists. Alternative: group it
with the blocked table cluster (010). Recommend implementing it now as
always-false (F16a) since it is correct and useful (guards in user code),
with the caveat recorded.

## Proposed order

F16 → F19 → F20 → F18 → F17 → F21 (ascending effort; psi last for its
special-function algorithm). `istable` folded into F16 pending the gate
decision above.

## Decision — RECORDED 2026-07-22 (maintainer, interactive gate)

- [x] Grouping and order **APPROVED**.
- [x] `istable`: **implement now as always-false**, folded into F16
  (correct for every constructible input; true-case unreachable until a
  table type exists — documented).
- [x] **All six ready features authorised** for implementation (F16 now
  covers isrow, iscolumn, issparse, iscellstr, istable).
- [x] `addvars`/`removevars` blocked status acknowledged — tracked with 010.
