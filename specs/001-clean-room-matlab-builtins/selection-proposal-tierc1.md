# Selection Proposal (FR-010) — Batch C1 (tables) — 2026-10-29

Candidates: the four COBRA-backlog table functions whose approved specs were
imported in Tier B batches 1–2 but **BLOCKED** at the time for lack of a
RunMat table type (B-010-1): `addvars`, `removevars` (Tier B batch 1,
pinned `e46587a`) and `movevars`, `outerjoin` (Tier B batch 2, pinned
`4445294`). This is batch R1 ("ready to implement") of the revised backlog in
`matlab-interface-spec/docs/prioritization/cobra-runmat-gap.md` (2026-10-29).

COBRA usage (source files): `removevars` 9, `addvars` 8, `movevars` 7,
`outerjoin` 5 (total 29).

## Blocker B-010-1 is resolved

This work was started on `feature/clean-room-batch-c1-tables` (based on
`feature/parallel-impls-0-6-0` @ `88e53d9e0`) and moved to
`feature/clean-room-matlab-builtins` after merge `57867ce4c`. That base which merges upstream `main`
(v0.6.2) and `RM-1064-remaining`. Upstream now provides a full `table` /
`timetable` subsystem under `crates/runmat-runtime/src/builtins/table/`,
including a column-level API that these features can build on without
touching the table representation:

- `table_variable_names_from_object`, `table_variables` — read names and
  columns in order;
- `table_from_columns_like` — rebuild a table that keeps the source's
  properties (row names, metadata);
- `rename_table_variables`, `make_unique_variable_names`,
  `validate_variable_names` — name handling;
- `object/rows.rs`, `object/selectors.rs` — row selection (for joins).

New builtins would go in a new hand-written `table/builtins/` child module
(`table/builtins.rs` is not a generated parent), so this batch adds nothing
to the generated-module maintenance burden.

## Import status

All four imports are current: `interface.yaml`, `behaviour.md` and
`provenance.yaml` are byte-identical to `matlab-interface-spec` HEAD
(`a4796fb`). Each `observations/*.json` differs only in its `notes` string
(source commit `1b01912` removed a stale "SKELETON run:" label). No case,
value or claim changed, so no re-import is required; optionally refresh the
pins to `1b01912` for tidiness (same precedent as Tier B batch 3).

## Proposed grouping

| # | Feature | Function(s) | Substrate | Scope | Status |
|---|---|---|---|---|---|
| F34 | 034-table-variable-editing | `addvars`, `removevars`, `movevars` | table column API | Small–medium | **Decision D1** |
| F35 | 035-outerjoin | `outerjoin` | table column + row API | Medium | **Decision D2** |

F34 is one tightly coupled family (Principle VII): all three take a table,
edit its ordered variable list, and rebuild it with the same row count and
properties.

## What the approved specs actually confirm

The specs are thin. Each confirms one basic form:

- `addvars(T, var)` appends one variable; class `table`; rows preserved
  (`[2 1]`→`[2 2]`).
- `removevars(T, name)` removes one named variable; rows preserved
  (`[1 3]`→`[1 2]`).
- `movevars(T, name, 'Before', location)` moves a variable before another;
  observed order `c,a,b`; counts preserved.
- `outerjoin(Tleft, Tright)` of two 2-row tables sharing a key gives a
  3-row, 4-variable table. Join semantics and fill values are explicitly
  **not** asserted.

Unresolved in the approved specs (claim ids): `addvars.q-position`
('Before'/'After'), `addvars.q-names` (NewVariableNames),
`removevars.q-index` (numeric index / logical mask),
`removevars.q-multiple` (several variables), `movevars.q-after` ('After'),
`movevars.q-index` (numeric location), `outerjoin.q-keys`
('Keys'/'LeftKeys'/'RightKeys'), `outerjoin.q-type` (left/right/full and fill
values).

These unresolved options are likely what COBRA call sites rely on.

## D1 — F34: how to treat the unresolved options

- **(A) Confirmed forms only; unresolved options raise a clear RunMat error**
  (e.g. `RunMat:addvars:UnsupportedOption`, "not yet specified"). This is
  strictly constitution-pure (Principles I/IV), the same treatment
  `inputParser` gave `addOptional`. COBRA call sites that pass
  'Before'/'After', NewVariableNames, index lists or several names fail
  loudly rather than silently diverging. In parallel, request that the
  specification side deepen the four specs (observe the q-* claims), then
  widen F34 in a follow-up feature.
- **(B) Confirmed forms normative, plus unresolved options as documented
  independent choices** (e.g. accept 'Before'/'After' by name or index,
  NewVariableNames, removal by name list / index / mask). This gives more
  COBRA coverage now, but the behaviour is not observation-backed. Each
  choice would be tagged `documented_choice` in spec, tests and docs and
  could diverge from MATLAB.

Recommendation: **(A)** plus a spec-deepening request. The options are cheap
to observe on the specification side, and an observation-backed widening
avoids shipping guesses into a compatibility layer.

## D2 — F35: whether to implement `outerjoin` now

The only normative claims are "output is a table" and "3×4 for this input".
A correct implementation needs choices the spec leaves open: which variables
are keys by default, how the output names duplicated key variables, the row
order, and the fill values for unmatched rows. All of these are user-visible.

- **(A) Defer F35** until the specification side observes
  `outerjoin.q-keys` / `outerjoin.q-type` (plus output naming and row order).
  Keep it BLOCKED, with blocker B-035-1 "join semantics unspecified".
- **(B) Implement now** as a full outer join on all common-named variables,
  with output naming, row order and fill values as documented choices that
  only the 3×4 shape check constrains.

Recommendation: **(A) defer**. Five COBRA files use `outerjoin`, and output
naming and fill values would almost certainly be relied on downstream, so a
guess is worse than a clear "unsupported".

## Proposed order

F34 (B1→B5) now; F35 after the specification side deepens `outerjoin`.

## Decision — RECORDED 2026-10-29 (maintainer, interactive gate)

- [x] Grouping/order **APPROVED**.
- [x] **D1: option A.** F34 implements only the confirmed forms; the
  unresolved options (`addvars.q-position`, `addvars.q-names`,
  `removevars.q-index`, `removevars.q-multiple`, `movevars.q-after`,
  `movevars.q-index`) raise an explicit RunMat "not yet specified" error.
  Request spec-side observation of these claims for a follow-up widening.
- [x] **D2: option A.** `outerjoin` is **DEFERRED**, blocker B-035-1 (join
  semantics unspecified: keys, type, output naming, row order, fill).
- [x] **Run scope: Gates 1–3 pre-approved in bulk** for F34 (same mode as
  Tier B batches 1–3). Run B1→B5 straight through; stop before any commit,
  push or PR (publication is a separate approval).
- [ ] Import pin refresh to `1b01912`: not requested (notes-label only).
