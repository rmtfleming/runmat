# Provenance Record — 033-environment-identity

| Field | Value |
|---|---|
| Source repository | `matlab-interface-spec` (local; no remote) |
| Approved specification identifiers | `version` 0.2.0, `ver` 0.2.0, `verLessThan` 0.2.0, `license` 0.2.0 — all `status: approved` |
| Specification revision | Commit pin: `1b019121bb747e63d69a3eb005534cb60669e5f4` (Tier B batch 3; see `specs/imports/matlab-interface-spec/IMPORT-MANIFEST.md`, batch-5 section) |
| Review status | Approved 2026-07-22 by repository maintainer (Tier B batch 3 human-loop review) |
| Import date | 2026-07-22 |
| Applicable release | R2026a (GLNXA64) — observed on the specification side |

## Identity divergence (BINDING — read with `spec.md` gate decision)

The four approved exports captured **MATLAB's own** environment identity
(`version` string, `-release`, the MATLAB `ver` product struct, `verLessThan`
booleans, `license('test')` == `1`). Those are MATLAB's factual values,
correctly documented on the specification side. **RunMat is not MATLAB.** Under
Constitution Principle II and the honesty requirement, RunMat deliberately does
**not reproduce** MATLAB's identity values; it returns its own honest identity
(version `0.5.6`, product `RunMat`, release tag `R2026a-compat`, open-source
licensing → available). The MATLAB values are recorded here as **non-normative
context**, never as satisfied behaviour.

## Claims table — contract vs MATLAB-value

| claim_id | category | contract-claim (honored) | MATLAB-value-claim (NOT asserted) |
|---|---|---|---|
| version.signature-primary | public-interface-fact | `v = version` accepted | — |
| version.output-class | black-box-observation | output class **char** row vector | — |
| version.version-strings | black-box-observation | **shape**: char row; a `-release` substring form | value `'26.1.0.3276743 (R2026a) Update 3'` / `'2026a'` — **NOT asserted** |
| ver.signature-primary | public-interface-fact | `s = ver(product)` accepted | — |
| ver.output-class | black-box-observation | output class **struct** | — |
| ver.product-struct | black-box-observation | **shape**: a struct is returned | MATLAB-product struct contents — **NOT asserted** |
| verLessThan.signature-primary | public-interface-fact | `tf = verLessThan(toolbox, version)` accepted | — |
| verLessThan.output-class | black-box-observation | output class **logical**, 1×1 | — |
| verLessThan.logical-compare | black-box-observation | **semantics**: 1×1 logical from a version comparison | values `true`/`false` for MATLAB 26.x — **NOT asserted** (RunMat 0.5.6 gives its own results) |
| license.signature-primary | public-interface-fact | `license('test', feature)` accepted | — |
| license.output-class | black-box-observation | output class **double**, 1×1 | — |
| license.test-numeric | black-box-observation | **value**: `1` == "available" (honored as RunMat's own open-source truth) | `1` as MATLAB's *licensing state* — not a copy |

Categories per Constitution Principle IV: `public-interface-fact` and
`black-box-observation` are the two source categories relied on. RunMat's
identity values (version `0.5.6`, `R2026a-compat`, `RunMat`, `2026-06-28`,
open-source availability) are **independently derived inferences / documented
choices**, not observations, and are labelled as such in `spec.md` and the
builtin tests (`documented_choice_*`).

## Unresolved questions carried

- `version.q-values` — exact strings release-dependent (RunMat uses its own).
- `ver.q-fields`, `ver.q-install-dependent` — field names/contents unresolved;
  RunMat's `Name/Version/Release/Date` is a documented choice.
- `verLessThan.q-deprecated` — deprecation vs `isMATLABReleaseOlderThan` not
  confirmed; not asserted.
- `license.q-other-actions`, `license.q-value` — no-arg / `inuse` / `checkout`
  forms unresolved; documented choices only.
