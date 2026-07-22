# Feature Specification: Environment Identity — `version`, `ver`, `verLessThan`, `license`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`033-environment-identity`; export `SPECIFY_FEATURE=033-environment-identity`)

**Created**: 2026-07-22

**Status**: Tier B batch 3 — implementation authorised (RunMat-honest identity
gate decision recorded below and in `provenance.md`)

**Input**: Approved Tier B exports `version` 0.2.0, `ver` 0.2.0,
`verLessThan` 0.2.0, `license` 0.2.0 (see `provenance.md`; import pinned at
source commit `1b019121bb747e63d69a3eb005534cb60669e5f4`).

## Gate decision — RunMat-honest identity (BINDING)

These four builtins report the identity of the running system. The approved
exports captured **MATLAB's own** identity on the specification-side
installation:

| function | MATLAB observed value | class |
|---|---|---|
| `version` | `'26.1.0.3276743 (R2026a) Update 3'` | char |
| `version('-release')` | `'2026a'` | char |
| `ver('MATLAB')` | 1×1 struct describing the MATLAB product | struct |
| `verLessThan('matlab','99.0')` | `true` | logical |
| `verLessThan('matlab','1.0')` | `false` | logical |
| `license('test','MATLAB')` | `1` | double |

Those values are **MATLAB's factual identity**, documented on the
specification side. **RunMat is not MATLAB.** Under Constitution Principle II
(clean-room separation) and the honesty requirement, RunMat MUST NOT return
MATLAB's observed identity strings/values as its own normative behaviour —
doing so would be impersonation and factually false.

Therefore this feature splits every claim into two parts:

- **Contract-claim (honored):** the observable signature and output
  **class/shape** (char row vector; a `-release` substring; a struct; a 1×1
  logical from a semantic version comparison; a double scalar). These are
  reproduced faithfully.
- **MATLAB-value-claim (deliberately NOT asserted):** the specific MATLAB
  strings/booleans/numbers above. RunMat returns its **own** honest identity
  instead. These MATLAB values are recorded as **non-normative** and are never
  asserted as satisfied by RunMat.

RunMat's own identity, used throughout:

- Version: `env!("CARGO_PKG_VERSION")` in `runmat-runtime` = **`0.5.6`**.
- Release tag (documented choice): **`R2026a-compat`** — signals the MATLAB
  release RunMat targets for compatibility; deliberately NOT MATLAB's `2026a`.
- Product name: **`RunMat`**; release date **`2026-06-28`** (RunMat's own
  v0.5.6 release date).
- Licensing: RunMat is **open-source with no license manager** → every feature
  is unconditionally **available**.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Query the running version (Priority: P1)

RunMat users calling `version` get a char row vector naming **RunMat's** own
version; `version('-release')` gives RunMat's release tag.

**Acceptance Scenarios** (contract, normative):

1. **Given** no argument, **When** `version` is called, **Then** the result is
   a char row vector whose text contains RunMat's crate version `0.5.6` and
   self-identifies as RunMat (e.g. `'0.5.6 (RunMat)'`). Class char.
   [version.output-class contract; version.version-strings **shape** only]
2. **Given** the option `'-release'`, **When** `version('-release')` is called,
   **Then** the result is a char row vector holding RunMat's release tag
   `'R2026a-compat'` (NOT MATLAB's `'2026a'`). Class char.
   [version.output-class contract]

### User Story 2 - Describe the running product (Priority: P1)

`ver` / `ver('RunMat')` returns a struct describing RunMat.

**Acceptance Scenarios** (contract, normative):

1. **Given** no argument (or `'RunMat'`), **When** `ver` is called, **Then**
   the result is class **struct** with fields `Name`='RunMat', `Version`=`0.5.6`,
   `Release`=`R2026a-compat`, `Date`=`2026-06-28`.
   [ver.output-class contract; ver.product-struct **shape** only]
2. **Given** `'MATLAB'`, **When** `ver('MATLAB')` is called, **Then** the
   result is an **empty struct array** (RunMat is not MATLAB). *(Documented
   choice; MATLAB's 1×1 MATLAB-product struct is NON-NORMATIVE and NOT
   reproduced.)*

### User Story 3 - Compare the running version (Priority: P1)

`verLessThan(toolbox, verstr)` returns a 1×1 logical: true iff **RunMat's**
version is older than `verstr`.

**Acceptance Scenarios** (contract, normative):

1. **Given** `('matlab','99.0')`, **When** called, **Then** the result is a 1×1
   logical `true` (0.5.6 < 99.0). [verLessThan.output-class contract;
   verLessThan.logical-compare **shape/semantics** with RunMat's value]
2. **Given** `('matlab','1.0')`, **When** called, **Then** the result is a 1×1
   logical **`true`** (0.5.6 < 1.0). *This DIVERGES from MATLAB's observed
   `false`, which reflected MATLAB's version 26.x. MATLAB's value is
   NON-NORMATIVE and NOT asserted; RunMat asserts its own honest comparison.*

### User Story 4 - Query licensing (Priority: P2)

`license('test', feature)` returns a double.

**Acceptance Scenarios** (contract, normative):

1. **Given** `('test', feature)` for any feature, **When** called, **Then** the
   result is the double `1` (available) — RunMat is open-source, so every
   feature is available. Class double, 1×1. [license.output-class contract;
   license.test-numeric: `1`==available honored as RunMat's own truth]

## Requirements *(mandatory)*

### Functional Requirements

- **FR-033-01** `version` MUST accept the zero-argument form `v = version`
  [version.signature-primary] and return class **char** [version.output-class]
  as a row vector holding **RunMat's own** version, embedding
  `env!("CARGO_PKG_VERSION")` (`0.5.6`). MATLAB's observed value
  `'26.1.0.3276743 (R2026a) Update 3'` is NON-NORMATIVE and MUST NOT be
  returned.
- **FR-033-02** `version('-release')` MUST be accepted and return a char row
  vector holding RunMat's release tag `'R2026a-compat'` (documented choice).
  MATLAB's `'2026a'` is NON-NORMATIVE. Option matching is case-insensitive over
  char/string scalars; any other option raises `RunMat:version:InvalidOption`;
  more than one argument raises `RunMat:version:TooManyInputs` (documented
  choices).
- **FR-033-03** `ver` (no argument) and `ver('RunMat')` MUST return class
  **struct** [ver.output-class] describing RunMat with fields `Name`,
  `Version`, `Release`, `Date` set to RunMat's own identity. The MATLAB
  product struct and its field set are unresolved (ver.q-fields,
  ver.q-install-dependent); the field choice here is RunMat's documented
  choice, honoring only the struct **class** contract.
- **FR-033-04** `ver('MATLAB')` (and any other unknown product) MUST return an
  **empty struct array** (documented choice: RunMat is not MATLAB). A MATLAB
  product entry MUST NOT be fabricated. Non-text product raises
  `RunMat:ver:InvalidProduct`; more than one argument raises
  `RunMat:ver:TooManyInputs`.
- **FR-033-05** `verLessThan(toolbox, verstr)` MUST accept two text arguments
  [verLessThan.signature-primary] and return class **logical**
  [verLessThan.output-class], a 1×1 scalar computed by a semantic version
  comparison (split on `'.'`, numeric compare) of **RunMat's** version
  (`0.5.6`) against `verstr`. The specific true/false values follow from
  RunMat's version; MATLAB's observed `true`/`false` values are NON-NORMATIVE.
  `toolbox` is a documented mapping: every component name (including
  `'matlab'`) maps to RunMat's single version. Missing/non-text arguments raise
  `RunMat:verLessThan:InvalidInput`; a non-numeric version string raises
  `RunMat:verLessThan:InvalidVersion`.
- **FR-033-06** `license('test', feature)` MUST return class **double**
  [license.output-class] scalar `1` for any feature [license.test-numeric:
  `1`==available], honored as RunMat's own truth (open-source → available), not
  as a copy of MATLAB's licensing state.
- **FR-033-07** The remaining `license` forms are documented choices
  (license.q-other-actions, license.q-value): `license` (no argument) returns
  RunMat's identity string `'RunMat'` (char); `license('inuse')` returns an
  empty struct array; `license('checkout'|'checkin', feature)` returns double
  `1`; unsupported usage raises `RunMat:license:InvalidUsage`.

### Key Entities

- `version`/`version('-release')`, `license` no-arg → `Value::CharArray` row.
- `ver`/`ver('RunMat')` → `Value::Struct`; `ver('MATLAB')`, `license('inuse')`
  → empty struct array (RunMat's `Value::Cell` 0×1 representation, as `dir`).
- `verLessThan` → `Value::Bool` (1×1 logical).
- `license('test',…)` → `Value::Num` (double scalar).
- Identity source: `env!("CARGO_PKG_VERSION")` compile-time constant; no
  process, MATLAB, or license-manager query.

## Success Criteria *(mandatory)*

- **SC-033-1**: Every builtin honors the observable **contract** (output
  class/shape) with RunMat's own values, proven by `contract_*` tests; NO test
  asserts a MATLAB observed identity value as RunMat behaviour.
- **SC-033-2**: All four builtins registered and discoverable via
  `builtin_function_by_name`; `cargo test -p runmat-runtime` passes for their
  test modules.
- **SC-033-3**: Traceability: each FR cites ≥1 claim id (contract) or is a
  documented choice; each normative test cites ≥1 FR; the identity divergence
  is explicit in spec and validation.

## Unresolved behaviour (non-normative; implemented by documented choice)

- **Identity divergence (whole feature):** MATLAB's observed identity values
  (version string, release, MATLAB product struct, verLessThan booleans,
  license `1`-as-MATLAB) are documented on the spec side but deliberately NOT
  reproduced. RunMat returns its own honest identity.
- `version.q-values`: exact strings are release-dependent → RunMat uses its own
  version + release tag.
- `ver.q-fields`, `ver.q-install-dependent`: field names/contents unresolved →
  RunMat's documented `Name/Version/Release/Date`. `ver` no-arg returns a
  single RunMat struct (RunMat has one "product").
- `verLessThan.q-deprecated`: any deprecation vs `isMATLABReleaseOlderThan`
  not asserted.
- `license.q-other-actions`, `license.q-value`: no-arg, `'inuse'`,
  `'checkout'`/`'checkin'` are documented choices.

## Assumptions

- RunMat's value model already provides `CharArray`, `StructValue`,
  `CellArray`, `Value::Bool`, `Value::Num`; no new runtime types required.
- `env!("CARGO_PKG_VERSION")` (`0.5.6`) is RunMat's authoritative version.
