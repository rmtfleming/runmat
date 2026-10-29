# Provenance Record — 034-table-variable-editing

| Field | `addvars` | `removevars` | `movevars` |
|---|---|---|---|
| Source repository | `matlab-interface-spec` (local) | same | same |
| Approved specification | `addvars` 0.2.0, `status: approved` | `removevars` 0.2.0, `status: approved` | `movevars` 0.2.0, `status: approved` |
| Specification revision (import pin) | **`e46587ae1b78237e0f43cd55f678de01a28495a4`** (Tier B batch 1, IMPORT-MANIFEST Batch 3) | **`e46587a`** (Tier B batch 1) | **`4445294`** (Tier B batch 2, IMPORT-MANIFEST Batch 4) |
| Review status | approved 2026-07-22 (source `docs/review/2026-07-22-tier-b-batch1-human-loop.md`) | same | approved 2026-07-22 (source `docs/review/2026-07-22-tier-b-batch2-human-loop.md`) |
| Import date | 2026-07-22 | 2026-07-22 | 2026-07-22 |
| Re-verified | 2026-10-29 against source HEAD `a4796fb`: interface/behaviour/provenance byte-identical; observations differ only in a cosmetic `notes` label (source `1b01912`), with no change to case data, values or claims | same | same |
| Applicable release | as recorded in the import (see the imported `provenance.yaml`) | same | same |

## Relied-upon claims

| claim_id | category | normative | used by |
|---|---|---|---|
| addvars.signature-primary | public-interface-fact | yes | FR-034-01 |
| addvars.output-class | black-box-observation | yes | FR-034-01 |
| addvars.appends-variable | black-box-observation | yes | FR-034-01 |
| removevars.signature-primary | public-interface-fact | yes | FR-034-02 |
| removevars.output-class | black-box-observation | yes | FR-034-02 |
| removevars.removes-variable | black-box-observation | yes | FR-034-02 |
| movevars.signature-primary | public-interface-fact | yes | FR-034-03 |
| movevars.moves-variable | black-box-observation | yes | FR-034-03 |

Backing observations (imported `observations/*.json`):

- `addvars` case `append`: one-variable 2-row table plus a 2-row column →
  class `table`, size `[2,2]`.
- `removevars` case `by-name`: 1×3 table `a,b,c`, remove `'b'` → class
  `table`, size `[1,2]`.
- `movevars` cases `before` / `order`: 1×3 table `a,b,c`, move `'c'`
  `'Before'` `'a'` → size `[1,3]`; joined `VariableNames` `'c,a,b'`.

The three `signature-primary` ids above name the confirmed invocation form in each
`interface.yaml` (`invocation_forms[*].status: confirmed`). They are
public-interface facts, not observations.

## Unresolved questions carried (returned to specification side)

| id | question | disposition in this feature |
|---|---|---|
| addvars.q-position | 'Before'/'After' options | `RunMat:addvars:UnsupportedOption` |
| addvars.q-names | NewVariableNames | `RunMat:addvars:UnsupportedOption` |
| removevars.q-index | numeric index / logical mask | `RunMat:removevars:UnsupportedOption` |
| removevars.q-multiple | multiple variables | `RunMat:removevars:UnsupportedOption` |
| movevars.q-after | 'After' form | `RunMat:movevars:UnsupportedOption` |
| movevars.q-index | numeric location | `RunMat:movevars:UnsupportedOption` |

Local unresolved items U-034-1…4 are listed in `spec.md`. Per FR-010
decision D1, all unresolved items are requested from the specification side
so the feature can be widened later.

## Source categories and independence

- Only the eight claims above are normative.
- Default variable naming, height validation, text equivalence, keyword
  case-insensitivity, metadata preservation, unknown-name and non-table
  errors, and timetable rejection are **independently derived documented
  choices**, based on RunMat's existing table subsystem conventions. They
  are tagged in tests (`unresolved_choice_*`, `rejects_*`) and are not
  asserted as MATLAB-conformant.
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked from this repository.
