# Provenance Record — 050-numeric-conversion

| Field | `nthroot` | `reallog` | `cast` |
|---|---|---|---|
| Source repository | `matlab-interface-spec` (local) | same | same |
| Approved specification | `nthroot` **0.2.0**, `status: approved` | `reallog` **0.2.0**, `status: approved` | `cast` **0.2.0**, `status: approved` |
| Observations | 14 cases | 12 cases | 16 cases |

| Field | Value (all three) |
|---|---|
| Specification revision (import pin) | **`8f72edb119511f95884fb63f550e6c88682c327b`** (IMPORT-MANIFEST Batch 10, N2) |
| Review status | approved 2026-10-30 (source `docs/review/2026-10-30-n2-new-specs-human-loop.md`, "Approve all 14"; `cast(x,'like',complex)` left open for review) |
| Import date | 2026-10-30 |
| Selection | maintainer, 2026-10-30, `specs/001-clean-room-matlab-builtins/selection-proposal-n2.md` (F50; G1–G3 pre-approved) |
| Freshness check (start) | 2026-10-30: in `../matlab-interface-spec`, after `git fetch -q origin`, HEAD = origin/main = `1156c97`. `git diff --name-only 8f72edb HEAD` and the same against `origin/main`, both over `specifications/{nthroot,reallog,cast}`, returned nothing; `git status --short` for those paths was clean. |
| Freshness check (finish) | recorded in `validation.md` |
| Applicable release | as recorded in each imported `provenance.yaml` |
| Imported files | `specs/imports/matlab-interface-spec/specifications/{nthroot,reallog,cast}/{interface.yaml,behaviour.md,provenance.yaml,observations/*_observations.json}` |

## Relied-upon claims

All are `source_category: black-box-observation`, `normative: true`,
`status: confirmed`. Every id below exists in the imported
`provenance.yaml` (checked with `grep claim_id`, see `validation.md`).

| claim_id | used by |
|---|---|
| nthroot.signature-primary | FR-050-01 |
| nthroot.output-class | FR-050-02 |
| nthroot.negative-odd | FR-050-03 |
| nthroot.even-root | FR-050-03 |
| nthroot.elementwise-n | FR-050-04 |
| nthroot.specials | FR-050-05 |
| nthroot.n-values | FR-050-06 |
| nthroot.exact | FR-050-07 |
| nthroot.error-even-negative | FR-050-08 |
| nthroot.error-complex | FR-050-09 |
| nthroot.error-size | FR-050-10 |
| reallog.signature-primary | FR-050-11 |
| reallog.values | FR-050-11 |
| reallog.specials | FR-050-12 |
| reallog.shape-class | FR-050-13 |
| reallog.error-negative | FR-050-14 |
| reallog.error-complex | FR-050-14 |
| reallog.error-int | FR-050-14 |
| cast.signature-primary | FR-050-15 |
| cast.rounding | FR-050-16 |
| cast.saturation | FR-050-16 |
| cast.nan-inf | FR-050-16 |
| cast.other-classes | FR-050-17 |
| cast.like | FR-050-15, FR-050-18 (complexity part: D-050-3) |
| cast.shape | FR-050-19 |
| cast.error-bad-type | FR-050-20 |
| cast.error-cell | FR-050-20 |

## Unresolved questions carried

`nthroot.q-warning-coverage`, `reallog.q-warning-coverage`,
`cast.q-warning-coverage`, and `cast.q-like-complexity` (flagged for human
review on the spec side). `spec.md` ("Unresolved behaviour") records how
each is handled.

**Reported back to the specification side:** for case `like-complex` the
claim `cast.like` (and `behaviour.md`) states a *real* double result, while
the observation record carries `isreal: false`. RunMat follows the claim
text (D-050-3) and tests only the class and value.

## Source categories and independence

- FR-050-01..20 come from black-box observations. The observed identifiers
  are in the `MATLAB:` namespace, and RunMat raises them in its own
  namespace (D-050-1).
- FR-050-21..30 are **independently derived documented choices** or
  explicit `UnsupportedOption` dispositions. Tests tag them
  `documented_choice_*` / `unresolved_*`.
- Algorithms, all written for this feature or reused from RunMat:
  - nthroot: `|x|^(1/n)` followed by one Newton step for `y^n = |x|`
    (textbook root refinement), sign restored for odd integer `n`.
  - reallog: elementwise `f64::ln` after a domain check.
  - cast: class-name validation, then delegation to RunMat's existing
    conversion builtins (`double`, `single`, `int8`…`uint64`, `logical`,
    `char`) (D-050-2).
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked from this repository. Independent legal review may
  still be required.
