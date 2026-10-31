# Feature Specification: `rat` — rational approximation (N5)

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`063-rat`; export `SPECIFY_FEATURE=063-rat`)

**Created**: 2026-10-31

**Status**: Batch N5 (F63). Gates G1–G3 pre-approved by the maintainer
(`specs/001-clean-room-matlab-builtins/selection-proposal-n5.md`).

**Input**: Approved export `rat` 0.2.0 (N5, `1c91aa1`, IMPORT-MANIFEST
Batch 14) (see `provenance.md`).

## Decisions

- **D-063-1 Error identifier namespace.** Observed `MATLAB:x:y`
  identifiers are raised as `RunMat:x:y` (maintainer rule). The observed
  char/integer error therefore raises `RunMat:norm:inputType`, which names
  another function; the maintainer disposition of `q-error-ids` is "raised
  as observed, under the `RunMat:` prefix" (`rat.q-error-ids`).
  Identifiers that were not observed (`InvalidInput`, `UnsupportedOption`,
  `TooManyOutputs`, `Internal`) use `RunMat:rat:<name>`.
- **D-063-2 Placement.** `rat` lives in `math/rounding/` (hand-written
  `mod.rs`; one `pub(crate) mod rat;` line). The selection proposal named
  `math/elementwise`; `rounding` is preferred because `rat` is built on
  nearest-integer rounding and its parent `mod.rs` is hand-written (no
  generated-parent edit needed).
- **D-063-3 Complex input** is supported as observed (maintainer
  disposition of `rat.q-complex`); the claim `rat.complex` stays
  non-normative and its tests are labelled `supported_*`.
- **D-063-4 Default tolerance.** Not determined by the export
  (`rat.q-default-tol`). RunMat uses the documented choice U-063-2, which
  reproduces every observed result (`pi` → `355/113`, `0.6667` →
  `6667/10000`, all simple cases).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Numerators and denominators (Priority: P1)

`[N, D] = rat(X)` (COBRA form, `D` used) returns elementwise numerators and
denominators with `N./D` approximating `X`; `[N, D] = rat(X, tol)` uses a
tolerance.

**Acceptance Scenarios** (normative, one per observed case):

1. `0.75` → `3/4`; `1/3` → `1/3`; `-0.5` → `-1/2` (sign on `N`, `D`
   positive); `5` → `5/1`; `0` → `0/1` (`quarter`, `third`, `negative`,
   `integer`, `zero`). [rat.signature-primary, rat.simple]
2. `pi` → `355/113` with `|N/D - pi| = 2.667641894049666e-07`; `0.6667` →
   `6667/10000` (`pi-default`, `coeff-like`). [rat.default-tol]
3. `rat(pi, 1e-2)` → `22/7` (`pi-tol`). [rat.tol]
4. `[0.5 0.25; 1.5 2]` → `N = [1 1; 3 2]`, `D = [2 4; 2 1]`; `[]` → empty
   outputs (`matrix`, `empty`). [rat.elementwise]
5. `N`, `D` are double for double input and single for single input
   (`quarter`, `single`). [rat.class]
6. `[Inf -Inf]` → `N = [1 -1]`, `D = [0 0]`; `NaN` → `0/0` (`inf`, `nan`).
   [rat.nonfinite]
7. char or integer `X` raises `RunMat:norm:inputType` (`err-char`,
   `err-int`). [rat.error-char]

### User Story 2 - Text expansion (Priority: P1)

`s = rat(X)` returns a char matrix with one continued-fraction row per
element.

**Acceptance Scenarios** (normative):

1. `rat(0.75)` → `'1 + 1/(-4)'` (1×10); `rat([0.5 1.5])` → 2×10 rows
   `'1 + 1/(-2)'`, `'2 + 1/(-2)'` (`text-form`, `text-form-vector`).
   [rat.text]

### User Story 3 - Complex input (Priority: P2, non-normative)

Supported as observed (D-063-3): `[N, D] = rat(1 + 2i)` → `N = 1+2i`,
`D = 1`; `s = rat(1 + 2i)` → 3×8 rows `'1       '`, `' +i* ...'`,
`'2       '` (`complex-two-out`, `complex-text-form`). [rat.complex
(non-normative)]

### User Story 4 - Unresolved forms fail clearly (Priority: P2)

Unobserved forms raise `RunMat:rat:UnsupportedOption` / `InvalidInput` or
follow a documented choice (FR-063-09), tagged `unresolved_choice_*`.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-063-01** `[N, D] = rat(X)` MUST return `N`, `D` with `N./D`
  approximating `X` elementwise, `D > 0` for finite input and the sign on
  `N`; integers give `D = 1`, `0` gives `0/1` [rat.signature-primary,
  rat.simple].
- **FR-063-02** With the default tolerance `pi` MUST give `355/113` and
  `0.6667` MUST give `6667/10000` [rat.default-tol].
- **FR-063-03** `rat(X, tol)` MUST accept a tolerance; `rat(pi, 1e-2)` MUST
  give `22/7` [rat.tol].
- **FR-063-04** Arrays MUST be handled elementwise with the shape of `X`;
  empty `X` MUST give empty outputs of the same shape [rat.elementwise].
- **FR-063-05** `N` and `D` MUST be double for double `X` and single for
  single `X` [rat.class].
- **FR-063-06** `Inf`/`-Inf` MUST give `N = ±1`, `D = 0`; `NaN` MUST give
  `N = 0`, `D = 0` [rat.nonfinite].
- **FR-063-07** For real `X`, one output MUST give a char matrix with one
  row per element (column-major element order), of the form
  `'1 + 1/(-4)'` for `0.75`, rows blank-padded to a common width
  [rat.text].
- **FR-063-08** char or integer-class `X` MUST raise
  `RunMat:norm:inputType` [rat.error-char; D-063-1].
- **FR-063-09** (documented choices, local ids; not MATLAB requirements)
  - **U-063-1** Algorithm: continued fraction with nearest-integer terms
    (`round`, halves away from zero): `a_k = round(y_k)`,
    `y_{k+1} = 1/(y_k - a_k)`, convergents by the standard recurrence;
    stop at the first convergent with `|x - N/D| <= tol`, when the
    remainder is exactly zero, when the next term or convergent would not
    be finite, or after 64 terms. Signs are normalised so `D >= 0`.
  - **U-063-2** Default tolerance: `1e-6 * |x|` for each element
    (relative, per element). An explicit `tol` is an absolute tolerance
    applied to every element.
  - **U-063-3** `tol` must be a real numeric scalar `>= 0` (not NaN);
    otherwise `RunMat:rat:InvalidInput`. `tol = 0` asks for the exact
    binary value where the term limit allows.
  - **U-063-4** Text form: an expansion `a0, a1, …, ak` is written
    `a0 + 1/(a1 + 1/(… + 1/(ak)…))` (integers without decimals, `-0`
    written `0`); `Inf`, `-Inf`, `NaN` are written as such; empty `X` gives
    a 0×0 char. Complex elements (D-063-3): `N`, `D` from the real and
    imaginary parts expanded separately over the common denominator
    `D = lcm(D_re, D_im)` (`D = 0` when either part is non-finite, the
    non-finite part keeping its `±1`/`0` numerator and the finite part 0);
    each complex element gives three text rows: real-part expansion,
    `' +i* ...'`, imaginary-part expansion. Single complex input gives
    single outputs.
  - **U-063-5** Output count: none requested or one → char matrix; two →
    `[N, D]`; zero → empty output list; more than two →
    `RunMat:rat:TooManyOutputs`.
  - **U-063-6** No input → `RunMat:rat:InvalidInput`; more than two inputs,
    and `X` of any other class (logical, string, cell, struct, sparse,
    objects) → `RunMat:rat:UnsupportedOption`. gpuArray input is gathered.
- **FR-063-10** (unresolved, returned to the specification side)
  `rat.q-warning-coverage`: no warnings are emitted. `rat.q-default-tol`:
  U-063-2. `rat.q-complex`: D-063-3. `rat.q-error-ids`: D-063-1.

## Success Criteria *(mandatory)*

- **SC-063-1** Every observed case reproduces in a test (normative cases in
  `observed_*`/`rejects_*`, complex cases in `supported_*`).
- **SC-063-2** `rat` is registered through `#[runtime_builtin]`;
  `cargo test -p runmat-runtime --lib -- rat_tests` passes.
- **SC-063-3** Every FR maps to at least one test (validation.md).
- **SC-063-4** `cargo test -p runmat-runtime --lib -- builtins::math::rounding`
  shows no regressions.

## Unresolved behaviour (non-normative)

| id | disposition |
|---|---|
| rat.q-default-tol | U-063-2 (reproduces both observed default-tolerance results) |
| rat.q-complex | supported as observed (D-063-3, U-063-4) |
| rat.q-error-ids | raised as observed under `RunMat:` (D-063-1) |
| rat.q-warning-coverage | no warnings emitted |

## Known deviations

None.
