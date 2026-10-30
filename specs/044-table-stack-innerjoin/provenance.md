# Provenance Record — 044-table-stack-innerjoin

| Field | `stack` | `innerjoin` |
|---|---|---|
| Source repository | `matlab-interface-spec` (local) | same |
| Approved specification | `stack` 0.2.0, `status: approved` | `innerjoin` 0.2.0, `status: approved` |
| Specification revision (import pin) | **`c65e2382fa2420416439f8342333af77dcaee5b3`**, IMPORT-MANIFEST Batch 9 (N1) | same |
| Review status | approved 2026-10-29 after human-loop review (source `docs/review/2026-10-29-n1-new-specs-human-loop.md`, "Approve all 14") | same |
| Selection | F44 in `specs/001-clean-room-matlab-builtins/selection-proposal-n1.md` (maintainer decision recorded 2026-10-30) | same |
| Freshness check | 2026-10-30, at start and at finish (see `validation.md`) | same |
| Applicable release | as recorded in the imported `interface.yaml` / `provenance.yaml` | same |

## Relied-upon claims (all `source_category: black-box-observation`, `normative: true`, `status: confirmed`)

Each id is verified against the imported
`specs/imports/matlab-interface-spec/specifications/{stack,innerjoin}/provenance.yaml`.

| claim_id | used by | backing cases |
|---|---|---|
| stack.signature-primary | FR-044-01 | basic |
| stack.layout | FR-044-01 | basic, by-index, three-vars |
| stack.default-names | FR-044-02 | basic, indicator, three-vars |
| stack.indicator | FR-044-03 | basic, indicator |
| stack.by-index | FR-044-04 | by-index, basic |
| stack.options | FR-044-05 | new-names, constant |
| stack.single-var | FR-044-06 | one-var |
| stack.error-unknown | FR-044-07 | err-unknown |
| stack.error-class-mix | FR-044-07 | err-class-mix |
| innerjoin.signature-primary | FR-044-10 | basic |
| innerjoin.output-class | FR-044-10 | basic, no-match |
| innerjoin.rows | FR-044-11 | basic |
| innerjoin.row-order | FR-044-11 | basic, unsorted-left, unsorted-both |
| innerjoin.single-key | FR-044-12 | basic, keys |
| innerjoin.keys | FR-044-13 | keys, leftright-keys |
| innerjoin.duplicates | FR-044-14 | dup-left |
| innerjoin.name-suffix | FR-044-15 | same-nonkey, same-nonkey-named |
| innerjoin.no-match | FR-044-16 | no-match |
| innerjoin.left-variables | FR-044-17 | left-variables |
| innerjoin.error-no-common | FR-044-18 | err-no-common |

The second invocation form of each (`Name, Value`) is a `status: confirmed`
public-interface fact in `interface.yaml`, backed by the cases cited above.
All 20 observation cases are reproduced as tests. Error cases record only an
identifier; RunMat raises the same identifier tail in the `RunMat:`
namespace and does not reproduce message text.

## Unresolved questions carried (returned to specification side)

| id | disposition |
|---|---|
| stack.q-warning-coverage | no warnings emitted; nothing asserted (FR-044-22) |
| stack.q-second-output | `RunMat:stack:UnsupportedOption` (FR-044-20) |
| stack.q-groups | `RunMat:stack:UnsupportedOption` (FR-044-21) |
| innerjoin.q-warning-coverage | no warnings emitted; nothing asserted (FR-044-22) |
| innerjoin.q-name-suffix-context | documented choice, same call-site mechanism as F35 (FR-044-34); flagged for human review |
| innerjoin.q-index-outputs | `RunMat:innerjoin:UnsupportedOption` (FR-044-23) |
| innerjoin.q-many-to-many | `RunMat:innerjoin:UnsupportedOption` (FR-044-24) |

Local items U-044-1…11 are listed in `spec.md` (FR-044-25..36).

## Source categories and independence

- Normative: the 20 claims above (black-box observations) and the confirmed
  invocation forms (public-interface facts).
- Independently derived: `stack` is a wide-to-long unpivot (row `i` of the
  input, stacked variable `v` → output row `i·k + v`); `innerjoin` is a
  textbook equi-join (ordered map from right key tuples to rows, stable sort
  of matches by key tuple). Neither is presented as an observation.
- The innerjoin join helpers (option parsing, key atoms, call-site suffixes,
  name uniquifying) are copied from RunMat's own clean-room F35
  `table/builtins/joins.rs`, whose items are private to that file. The
  duplication is deliberate so that `joins.rs` is not edited.
- The caller-name mechanism reuses RunMat's call-site recording
  (`crate::callsite`), which `display`, `inputname` and F35 also use. It is
  RunMat architecture, not derived from MATLAB.
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked. No external implementation code was used, so there
  is no third-party licence to record. Independent legal review MAY still be
  required.
