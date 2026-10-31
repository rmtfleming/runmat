# Provenance Record — 049-exceptions-warnings

| Field | `MException` | `lastwarn` | `throw` |
|---|---|---|---|
| Source repository | `matlab-interface-spec` (local) | same | same |
| Approved specification | `MException` **0.2.0**, `status: approved` | `lastwarn` **0.2.0**, approved | `throw` **0.2.0**, approved |
| Specification revision (import pin) | **`8f72edb119511f95884fb63f550e6c88682c327b`** (IMPORT-MANIFEST Batch 10, N2) | same | **`1156c975db1f393b58252a44e04333a130211398`** (IMPORT-MANIFEST Batch 11, N3) |
| Review status | approved 2026-10-30 (`docs/review/2026-10-30-n2-new-specs-human-loop.md`) | same | approved 2026-10-30 (`docs/review/2026-10-30-n3-new-specs-human-loop.md`) |
| Import date | 2026-10-30 | 2026-10-30 | 2026-10-30 |
| Freshness check | 2026-10-30, at start and at finish: after `git fetch -q origin`, `git log <pin>..HEAD` and `<pin>..origin/main` over each spec dir were empty (spec repo `HEAD` = `origin/main` = `1156c97`); `git status --short` clean (see validation.md) | same | same |
| Applicable release | R2026a, as recorded in the imported `provenance.yaml` | same | same |

## Relied-upon claims

All ids verified present in the imported 0.2.0 `provenance.yaml` files
(`status: confirmed`, `source_category: black-box-observation`). Normative
unless marked.

| claim_id | used by |
|---|---|
| MException.signature-primary | FR-049-01 |
| MException.output-class | FR-049-01 |
| MException.properties | FR-049-01 |
| MException.identifier-form | FR-049-02 |
| MException.error-bad-id | FR-049-03 |
| MException.error-no-message | FR-049-03 |
| MException.throw-catch | FR-049-04 |
| MException.rethrow | FR-049-05 |
| MException.addcause | FR-049-06 (met by F60; D-049-1 closed) |
| throw.signature-primary | FR-049-04 |
| throw.identifier | FR-049-04 |
| throw.caught | FR-049-04 |
| throw.formatted | FR-049-04 |
| throw.no-id | FR-049-04, U-049-1 |
| throw.rethrow | FR-049-05 |
| throw.cause | FR-049-06 (met by F60; D-049-1 closed) |
| throw.non-mexception (**non-normative**) | U-049-2 |
| lastwarn.signature-primary | FR-049-07 |
| lastwarn.after-warning | FR-049-07 |
| lastwarn.set | FR-049-08 |
| lastwarn.set-return | FR-049-09 |
| lastwarn.disabled | FR-049-10 |
| lastwarn.no-id | FR-049-11 |
| lastwarn.empty-class | FR-049-12 |

Error identifiers (programme rule: observed `MATLAB:x:y` → `RunMat:x:y`):

| observed (export) | raised by RunMat |
|---|---|
| `MATLAB:MException:badIdentifier` | `RunMat:MException:badIdentifier` |
| `MATLAB:minrhs` | `RunMat:minrhs` |
| `MATLAB:UndefinedFunction` (throw, non-normative) | `RunMat:throw:UndefinedFunction` |
| thrown user identifiers (`n3:demo`, `t:h`, `''`) | unchanged (they are the exception's own identifier) |

## Unresolved questions carried (returned to specification side)

| id | disposition in this feature |
|---|---|
| MException.q-stack | not asserted; new exceptions have an empty stack |
| throw.q-stack | not asserted |
| MException.q-warning-coverage, throw.q-warning-coverage, lastwarn.q-warning-coverage | no warnings emitted by these builtins |

## Source categories and independence

- Only the claims above are relied upon (23 normative, 1 non-normative); 2
  normative claims are not met (D-049-1) and are reported, not hidden.
- The implementation uses RunMat's own `Value::MException`, `RuntimeError`,
  `format_variadic` and warning policy. Nothing is derived from MATLAB
  internals.
- U-049-1 … U-049-6 are choices, tagged `unresolved_choice_*` in tests.
- No MathWorks source, documentation prose or examples were consulted.
  MATLAB was not invoked from this repository. Independent legal review MAY
  still be required.
