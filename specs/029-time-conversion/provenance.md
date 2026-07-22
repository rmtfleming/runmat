# Provenance Record — 029-time-conversion

| Field | Value |
|---|---|
| Source repository | `matlab-interface-spec` (local import under `specs/imports/`) |
| Approved specification identifiers | `etime` 0.2.0, `seconds` 0.2.0 — both `status: approved` |
| Specification revision | Source commit **`1b019121bb747e63d69a3eb005534cb60669e5f4`** ("Clean up stale SKELETON labels"); Tier B batch 3, approved at commit `67eaf83`, pinned to `1b01912` at import (see `specs/imports/matlab-interface-spec/IMPORT-MANIFEST.md`, "Batch 5 — Tier B batch 3") |
| Review status | Approved 2026-07-22 (source `docs/review/2026-07-22-tier-b-batch3-human-loop.md`; all 12 batch-3 specs approved) |
| Import date | 2026-07-22 |
| Applicable release | R2026a (26.1.0.3276743, GLNXA64) |

## Relied-upon claims — `etime`

| claim_id | category | normative |
|---|---|---|
| etime.signature-primary | public-interface-fact | yes |
| etime.output-class | black-box-observation | yes |
| etime.elapsed-seconds | black-box-observation | yes |

Backing observation: `observations/etime_observations.json`, case
`one-second` — `etime([2020 1 1 0 0 1], [2020 1 1 0 0 0])` → `1` (double,
1×1).

## Relied-upon claims — `seconds`

| claim_id | category | normative |
|---|---|---|
| seconds.signature-primary | public-interface-fact | yes |
| seconds.output-class | black-box-observation | yes |
| seconds.to-duration | black-box-observation | yes |

Backing observation: `observations/seconds_observations.json`, case
`to-duration` — `seconds(5)` → `<duration [1 1]>` (class duration, 1×1).

## Unresolved questions carried

- `etime.q-deprecated` — deprecation notice (unobserved).
- `etime.q-negative` — result when `t2` precedes `t1` (unobserved).
- `seconds.q-from-duration` — the duration-to-number form `seconds(d)`
  (unobserved).

The observed `seconds(5)` value is opaque (`<duration [1 1]>`): the internal
magnitude (5 seconds) is the documented semantic meaning of the function and
the display `Format` is an unobserved documented choice. Both are carried as
non-normative independent choices, not asserted as MATLAB-conformant beyond
class and size.

## Clean-room / licensing note

Implementation uses only the imported export artefacts, RunMat's existing
`duration` module, and an independent public-domain civil-to-days algorithm
(Howard Hinnant's chrono-compatible date algorithms). No MathWorks source,
documentation prose, or examples were consulted; MATLAB was not invoked.
Independent legal review may still be required (Constitution Principle X).
