# Selection Proposal (FR-010) — Tier A batch 1 — 2026-07-22

Candidates: the 20 approved exports imported under
`specs/imports/matlab-interface-spec/` (manifest: IMPORT-MANIFEST.md).
Eligibility per FR-002: (a)–(c),(e) satisfied for all 20; (d) satisfied only
by interim content pin — see blocker B-001-REVISED.

None of the 20 exists in RunMat today (verified against
`crates/runmat-runtime/src/builtins/` sources and
`docs/builtins/reference/` catalogue, 481 entries). All are genuinely new.

## Proposed feature grouping (Principle VII: coherent, tightly coupled families)

| # | Feature (proposed) | Functions | Primary category / substrate | Scope estimate |
|---|---|---|---|---|
| F1 | type-predicates | `iscell`, `isstruct`, `isfile` | logical / introspection; `isfile` touches `runmat-filesystem` | Small — value-kind checks + fs stat |
| F2 | struct-cell-access | `fields`, `num2cell` | structs / cells | Small–medium |
| F3 | string-search | `matches`, `strmatch`, `strtok` | strings | Medium |
| F4 | numeric-string-conversion | `int2str`, `str2num` | strings / numeric | Medium (`str2num` expression semantics need care) |
| F5 | date-formatting | `datestr` | datetime | Medium |
| F6 | path-parts | `fileparts` | io / fs paths | Small |
| F7 | sparse-construction | `full`, `speye`, `spdiags` | `Value::SparseTensor` (CSC) exists in `runmat-builtins` | Medium–large |
| F8 | table-conversion | `array2table`, `cell2table`, `table2array`, `table2cell` | table + cells | Medium–large |
| F9 | minmax-bounds | `bounds` | stats / reduction | Small |

## Proposed order

F1 → F6 → F9 → F2 → F3 → F4 → F5 → F7 → F8.
Rationale: F1/F6/F9 are the smallest coherent features and exercise the full
gated pipeline end-to-end early; sparse (F7) and table (F8) carry the most
architectural surface and benefit from established precedent.

## Blockers

- **B-001-REVISED**: source repo still uncommitted; provenance pinned by
  content hash (interim). Human decision required: accept interim pin, or
  commit/tag the source repository first and upgrade the pin.

## Decision (to be recorded at gate)

- [ ] Grouping approved / amended
- [ ] Order approved / amended
- [ ] First feature authorised for B1 (requirements)
