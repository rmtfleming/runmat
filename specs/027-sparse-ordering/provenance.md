# Provenance Record — 027-sparse-ordering

| Field | Value |
|---|---|
| Source repository | `matlab-interface-spec` (local; no remote) |
| Approved specification identifiers | `colamd` 0.2.0, `dissect` 0.2.0 — both `status: approved` |
| Specification revision (pin) | Source commit **`4445294`** ("Approve Tier B batch 2 after human-loop review"); Tier B batch 2. See `specs/imports/matlab-interface-spec/IMPORT-MANIFEST.md`, Batch 4 section (byte-identical between `4445294` and source HEAD `37087db`; SHA256SUMS regenerated over all 50 specifications). |
| Review status | Approved 2026-07-22 by repository maintainer (source `docs/review/2026-07-22-tier-b-batch2-human-loop.md`, "all 12 approved"; `colamd`/`dissect` had no spec-side corrections in that batch). |
| Import date | 2026-07-22 |
| Applicable release | R2026a (26.1.0.3276743, GLNXA64) |

## Relied-upon claims (from the approved exports)

| claim_id | category | normative | used by |
|---|---|---|---|
| colamd.signature-primary | public-interface-fact | yes | FR-027-01 |
| colamd.output-class | black-box-observation | yes | FR-027-02 |
| colamd.permutation-vector | black-box-observation | yes | FR-027-02 |
| dissect.signature-primary | public-interface-fact | yes | FR-027-01 |
| dissect.output-class | black-box-observation | yes | FR-027-03 |
| dissect.permutation-vector | black-box-observation | yes | FR-027-03 |

Backing observations:
`specs/imports/matlab-interface-spec/specifications/colamd/observations/colamd_observations.json`
(case `basic`) and
`.../dissect/observations/dissect_observations.json` (case `basic`) — one
independently constructed case each on R2026a (GLNXA64).

## External algorithm sources (Constitution Principle X)

The observed permutation values are algorithm-dependent and NOT normative
(`colamd.q-values`, `dissect.q-values`). The orderings are implemented from an
**independent** fill-reducing heuristic derived from published algorithm
literature — NOT from MathWorks, SuiteSparse COLAMD, or METIS. No MathWorks or
SuiteSparse/METIS source code, documentation prose, or documentation examples
were consulted.

| function | heuristic | public source(s) | licence of source |
|---|---|---|---|
| `colamd` | minimum-degree ordering on the column-intersection graph (pattern of `Aᵀ·A`) | Tinney & Walker, "Direct solutions of sparse network equations by optimally ordered triangular factorization," Proc. IEEE 55(11):1801–1809, 1967; George & Liu, "Computer Solution of Large Sparse Positive Definite Systems," Prentice-Hall, 1981 | published algorithm description (journal / textbook) |
| `dissect` | recursive nested dissection via level-structure graph bisection with a pseudo-peripheral root | George, "Nested Dissection of a Regular Finite Element Mesh," SIAM J. Numer. Anal. 10(2):345–363, 1973; George & Liu 1981 (level structures); Gibbs, Poole & Stockmeyer, "An algorithm for reducing the bandwidth and profile of a sparse matrix," SIAM J. Numer. Anal. 13(2):236–250, 1976 (pseudo-peripheral vertex) | published algorithm description (journal / textbook) |

These implementations are original Rust code derived from the published
algorithm descriptions; no third-party source code was transferred. The
`colamd` implementation is an exact minimum-degree elimination on an explicitly
formed intersection graph — deliberately distinct from SuiteSparse COLAMD's
approximate-external-degree / supernode-absorption scheme. The `dissect`
implementation uses a whole BFS level as the vertex separator — deliberately
distinct from MATLAB's METIS-based multilevel nested dissection. Independent
legal review MAY still be required; this record does not substitute for legal
advice (Principle X).

## Unresolved questions carried

- `colamd.q-permutation`, `dissect.q-permutation` — orientation/permutation
  confirmation (partially satisfied by the normative valid-permutation test).
- `colamd.q-values`, `dissect.q-values` — **the specific permutation values are
  algorithm-dependent and are carried as non-normative**; this feature does not
  assert them (see `spec.md`, `validation.md`).

## Notes

- Source `provenance.yaml` author for this batch: "clean-room agent (Claude
  Opus 4.8)"; approval by the repository maintainer (per the import manifest).
- No MATLAB was invoked in this repository at any point (Principle II).
