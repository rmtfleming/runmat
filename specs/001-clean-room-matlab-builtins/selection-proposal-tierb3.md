# Selection Proposal (FR-010) — Tier B batch 3 — 2026-07-22

Candidates: 12 approved specs imported from source HEAD `1b01912` (approved at
`67eaf83`; see IMPORT-MANIFEST.md batch-5). Environment / introspection / time
functions. Two carry genuine design decisions (F28, F33) surfaced below per
Principle VIII rather than silently resolved.

## Proposed grouping

| # | Feature | Function(s) | Substrate | Scope | Status |
|---|---|---|---|---|---|
| F28 | 028-eps-function | `eps` | constants (COLLISION — see below) | Small | **Decision** |
| F29 | 029-time-conversion | `etime`, `seconds` | datetime / duration | Small–medium | Ready |
| F30 | 030-accumarray | `accumarray` | array ops | Medium | Ready |
| F31 | 031-error-report | `getReport` | Value::MException | Small | Ready |
| F32 | 032-object-introspection | `properties`, `methods`, `what` | ClassDef/ObjectInstance + filesystem | Medium | Ready (values object/fs-specific — see note) |
| F33 | 033-environment-identity | `version`, `ver`, `verLessThan`, `license` | RunMat identity | Small | **Decision** |

## Ready features (F29–F32)

- `etime([2020 1 1 0 0 1],[2020 1 1 0 0 0])`→1 (seconds between date vectors);
  `seconds(5)`→duration. Deterministic.
- `accumarray([1;2;1],[10;20;30])`→`[40;20]` (accumulate by subscript).
  Deterministic.
- `getReport(MException('x:y','oops'))`→`'oops'` (report text). MException
  value kind exists.
- `properties(obj)`/`methods(obj)` return a cellstr column of the object's
  property / method names; `what(dir)` returns a struct listing folder
  contents. **Note (like colamd/dissect):** the observed counts (9 properties,
  19 methods) are for *MATLAB's* inputParser; RunMat's inputParser has its own
  (fewer) members. Normative claim = returns a cellstr column of the actual
  object's members; the specific list/count is NOT asserted MATLAB-conformant.

## F28 decision — `eps` constant vs function

RunMat already registers `eps` as a **constant** (`register_constant!("eps",
…)` in `builtins/constants/mod.rs`) returning double machine epsilon; bare
`eps` works. The spec needs the **function** form `eps('single')`→single
epsilon (and MATLAB's `eps(x)` = distance to next float, though only bare and
'single' are observed). A name cannot be both a constant and a function.
Options:

- **(A) Replace the constant with a function** `eps` that returns double eps
  with no args, single eps for `'single'`, and (documented choice) `eps(x)`
  ulp distance. Bare `eps` still evaluates correctly (zero-arg call). Cleanest
  MATLAB alignment; modifies one existing registration (flagged, minimal).
- **(B) Leave the constant, skip the function** — defer `eps('single')` until
  a constants-vs-functions convention exists. No collision risk; incomplete.

Recommendation: **(A)** — it is the honest MATLAB-compatible behaviour and the
change is contained (one constant → one builtin). Requires verifying bare
`eps` still resolves as a zero-arg call in RunMat's VM.

## F33 decision — environment identity (RunMat is not MATLAB)

`version`→`'26.1.0.3276743 (R2026a) Update 3'`, `ver('MATLAB')`→version
struct, `verLessThan('matlab','99.0')`→true, `license('test','MATLAB')`→1.
These observed MATLAB's **own identity**. RunMat returning these verbatim
would be impersonation and factually false (Principle II/III, and honesty).
The observable *shape* of each can be honored with RunMat's own truth:

- **`version`**: return RunMat's version string (e.g. from
  `CARGO_PKG_VERSION`), optionally with a MATLAB-compatibility token.
  `version('-release')` → a RunMat-defined release tag or a configured
  compat level. NOT MATLAB's `'2026a'`.
- **`ver`**: struct with Name='RunMat' (or a compat entry), Version=RunMat's,
  Release/Date RunMat's. NOT a MATLAB struct.
- **`verLessThan`**: compares against whatever `version` reports (consistent
  with the above).
- **`license`**: RunMat is open-source with no license manager;
  `license('test',…)`→1 (available) is honest and useful; `license` /
  `license('inuse')` return RunMat-appropriate values.

Options: **(A)** implement all four with RunMat-honest identity (recommended);
**(B)** implement only `license` (open-source → available) now and defer
version/ver/verLessThan pending a decision on a MATLAB-compat version scheme;
**(C)** defer F33 entirely.

Whatever is chosen, tests assert the *shape/class/contract* (e.g.
verLessThan returns logical; version returns a char version string) and
RunMat's own values — never MATLAB's observed strings as normative.

## Proposed order

F31 → F29 → F30 → F32 → F28 → F33 (ready first; decisions last).

## Decision — RECORDED 2026-07-22 (maintainer, interactive gate)

- [x] Grouping/order **APPROVED**.
- [x] **F29–F32 authorised** (etime, seconds, accumarray, getReport,
  properties, methods, what).
- [x] **F28 `eps`: option A** — replace the constant with a builtin function
  (no args → double eps, `'single'` → single eps, `eps(x)` → ulp distance as
  documented choice); verify bare `eps` still resolves as a zero-arg call.
- [x] **F33 identity: option A** — implement all four with RunMat-honest
  identity (version→RunMat's version, ver→RunMat struct, verLessThan→compares
  RunMat's version, license('test')→1 open-source). Assert contract/shape and
  RunMat's own values; MATLAB's observed strings are NEVER normative.
- [x] properties/methods/colamd-style value-non-assertion framing
  acknowledged.
