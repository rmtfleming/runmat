# Feature Specification: environment — `isunix`, `ismac`, `isMATLABReleaseOlderThan`, `chdir`, `date`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`053-environment`; export `SPECIFY_FEATURE=053-environment`)

**Created**: 2026-10-30

**Status**: Batches N3+N4, feature F53. Gates G1–G3 pre-approved by the
maintainer (`specs/001-clean-room-matlab-builtins/selection-proposal-n3n4.md`).

**Input**: Approved exports `chdir` and `date` **0.2.0** (N3, IMPORT-MANIFEST
Batch 11, `1156c97`) and `isunix`, `ismac`, `isMATLABReleaseOlderThan`
**0.2.0** (N4, IMPORT-MANIFEST Batch 13, `fc9f804`). See `provenance.md`.

Family rationale (Principle VII): five small queries of, or changes to, the
running environment (host platform, compatibility release, current folder,
current date). They share no state. Each has its own requirement block and
test module.

## Decisions

- **D-053-1 Error identifier namespace.** Observed `MATLAB:x:y` identifiers
  are raised as `RunMat:x:y`: `RunMat:maxrhs`, `RunMat:minrhs`,
  `RunMat:isMATLABReleaseOlderThan:invalidReleaseInput`,
  `RunMat:validators:mustBeTextScalar`, `RunMat:cd:NonExistentFolder`,
  `RunMat:string` and `RunMat:TooManyInputs`. Identifiers that were not
  observed (`UnsupportedOption`, `TooManyOutputs`, `InvalidInput`,
  `Internal`) use `RunMat:<fn>:<name>`.
- **D-053-2 Host platform.** `isunix` and `ismac` were observed only on a
  GLNXA64 (Linux) oracle. RunMat reports the platform it was compiled for
  (`cfg!(unix)`, `cfg!(target_os = "macos")`). On Linux this reproduces every
  observation. The values on other platforms are a documented choice
  (FR-053-22), not MATLAB behaviour.
- **D-053-3 Running release.** `isMATLABReleaseOlderThan` compares with
  RunMat's compatibility pin `crate::compatibility::MATLAB_COMPATIBILITY_RELEASE`
  (currently `R2026a`, the release that was observed). The same pin is used
  by `version('-release')` and `verLessThan`. When the pin moves, the
  observed truth table moves with it, as the spec says (results depend on the
  running release).
- **D-053-4 `chdir` reuses `cd`.** After its own argument checks
  (FR-053-16, FR-053-17), `chdir` calls RunMat's `cd` implementation
  (`builtins/io/repl_fs/cd`), which changes the folder and returns the
  previous one. Path expansion and the folder value format are those of `cd`.
- **D-053-5 `date` reuses `clock`.** `date` calls RunMat's `clock` builtin
  once and formats its year, month and day fields. So the date agrees with
  `clock` by construction (time zone and day boundary included).
- **D-053-6 Unobserved points** raise `RunMat:<fn>:UnsupportedOption` or are
  labelled documented choices below. Neither kind is presented as MATLAB
  behaviour.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Platform tests (Priority: P1)

**Acceptance Scenarios** (normative on a Linux host, one per observed case):

1. `isunix` → logical `true` 1×1 (`value`). [isunix.signature-primary,
   isunix.value]
2. `isunix` true and `ismac` false together (`consistency`; `ispc` is not in
   scope). [isunix.value]
3. `isunix && ~ismac` → `true` (`cobra-linux-test`). [isunix.cobra-test]
4. `isunix(1)` raises `RunMat:maxrhs` (`err-arg`). [isunix.error-arg]
5. `ismac` → logical `false` 1×1 (`value`). [ismac.signature-primary,
   ismac.value]
6. `ismac(1)` raises `RunMat:maxrhs` (`err-arg`). [ismac.error-arg]

### User Story 2 - Release comparison (Priority: P1)

With the compatibility pin at `R2026a`:

1. `'R2022a'` → false (`older`); `'R2006a'` → false (`much-older`);
   `'R2026a'` → false (`same`). [isMATLABReleaseOlderThan.signature-primary,
   isMATLABReleaseOlderThan.comparison]
2. `'R2026b'` → true (`later-b`); `'R2030a'` → true (`later-year`).
   [isMATLABReleaseOlderThan.comparison]
3. `"R2027a"` (string) → true (`string`); `'r2022a'` → false (`lowercase`).
   [isMATLABReleaseOlderThan.text, isMATLABReleaseOlderThan.comparison]
4. `'2022a'` raises `RunMat:isMATLABReleaseOlderThan:invalidReleaseInput`
   (`err-bad-name`). [isMATLABReleaseOlderThan.error-bad-name]
5. `2022` raises `RunMat:validators:mustBeTextScalar` (`err-numeric`).
   [isMATLABReleaseOlderThan.error-numeric]
6. No input raises `RunMat:minrhs` (`err-none`).
   [isMATLABReleaseOlderThan.error-none]

All results are logical 1×1.

### User Story 3 - Change folder (Priority: P1)

Tests use a fresh temporary folder in place of `tempdir`, and restore the
start folder.

1. `chdir(T)` makes `T` the current folder (`changes`); `chdir(string(T))`
   does the same (`string`). [chdir.signature-primary, chdir.changes]
2. `chdir(T)` and `chdir([T filesep])` land in the same folder
   (`trailing-sep`). [chdir.trailing-sep]
3. `chdir(T); chdir('..')` moves to the parent of `T` (`relative`).
   [chdir.relative]
4. `old = chdir(T)` from a folder other than `T` returns a 1×n char row equal
   to the previous folder, and the current folder is then `T`
   (`output-old`). [chdir.output-old]
5. `d = chdir` returns a 1×n char row equal to `pwd` (`no-arg-output`).
   [chdir.no-arg]
6. `chdir(fullfile(T,'no_such_folder_n3'))` raises
   `RunMat:cd:NonExistentFolder` (`err-missing`). [chdir.error-missing]
7. `chdir(5)` raises `RunMat:string` (`err-numeric`). [chdir.error-numeric]

### User Story 4 - Current date (Priority: P1)

The date changes, so tests check form and agreement with `clock`, never a
fixed date.

1. `d = date` is a 1×11 char row (`class-shape`). [date.signature-primary,
   date.output-class]
2. `d` matches `^\d{2}-[A-Z][a-z]{2}-\d{4}$`, with a two-character day field
   (`pattern`, `leading-zero`). [date.format]
3. Day, month (index in Jan…Dec) and year agree with `clock`
   (`agrees-clock`). [date.agrees-clock]
4. `date(1)` raises `RunMat:TooManyInputs` (`err-arg`). [date.error-arg]

## Requirements *(mandatory)*

### Functional Requirements — `isunix`, `ismac` (normative)

- **FR-053-01** `tf = isunix` MUST be accepted and return a logical scalar,
  `true` on a Linux host [isunix.signature-primary, isunix.value].
- **FR-053-02** `tf = ismac` MUST be accepted and return a logical scalar,
  `false` on a Linux host [ismac.signature-primary, ismac.value].
- **FR-053-03** On a Linux host `isunix && ~ismac` MUST be `true`
  [isunix.cobra-test].
- **FR-053-04** Any input argument MUST raise `RunMat:maxrhs`
  [isunix.error-arg, ismac.error-arg].

### Functional Requirements — `isMATLABReleaseOlderThan` (normative)

- **FR-053-05** `tf = isMATLABReleaseOlderThan(release)` MUST be accepted
  and return a logical scalar [isMATLABReleaseOlderThan.signature-primary].
- **FR-053-06** The result is `true` exactly when the running release is
  older than `release`. With the running release `R2026a`: `R2022a`,
  `R2006a`, `R2026a` → false; `R2026b`, `R2027a`, `R2030a` → true
  [isMATLABReleaseOlderThan.comparison].
- **FR-053-07** `release` may be a char row or a string scalar, and a
  lower-case leading `r` is accepted [isMATLABReleaseOlderThan.text].
- **FR-053-08** Text that is not a release name (e.g. `'2022a'`) MUST raise
  `RunMat:isMATLABReleaseOlderThan:invalidReleaseInput`
  [isMATLABReleaseOlderThan.error-bad-name].
- **FR-053-09** A numeric input MUST raise
  `RunMat:validators:mustBeTextScalar`
  [isMATLABReleaseOlderThan.error-numeric].
- **FR-053-10** No input MUST raise `RunMat:minrhs`
  [isMATLABReleaseOlderThan.error-none].

### Functional Requirements — `chdir` (normative)

- **FR-053-11** `chdir(folder)` MUST make `folder` the current folder, for a
  char row or a string scalar [chdir.signature-primary, chdir.changes].
- **FR-053-12** A trailing file separator on `folder` MUST NOT change the
  target [chdir.trailing-sep].
- **FR-053-13** A relative `folder` (`'..'`) is resolved from the current
  folder [chdir.relative].
- **FR-053-14** `old = chdir(folder)` MUST return the previous current
  folder as a 1×n char row, and still change folder [chdir.output-old].
- **FR-053-15** `d = chdir` MUST return the current folder as a 1×n char
  row equal to `pwd`, without changing it [chdir.no-arg].
- **FR-053-16** A folder that does not exist MUST raise
  `RunMat:cd:NonExistentFolder` [chdir.error-missing].
- **FR-053-17** A numeric `folder` MUST raise `RunMat:string`
  [chdir.error-numeric].

### Functional Requirements — `date` (normative)

- **FR-053-18** `d = date` MUST return a 1×11 char row
  [date.signature-primary, date.output-class].
- **FR-053-19** `d` MUST have the form `DD-Mmm-YYYY`: two day digits,
  hyphen, a three-letter month with a capital first letter, hyphen, four
  year digits [date.format].
- **FR-053-20** Day, month and year MUST agree with `clock`; October is
  `Oct` [date.agrees-clock].
- **FR-053-21** Any input argument MUST raise `RunMat:TooManyInputs`
  [date.error-arg].

### Functional Requirements (documented choices and unresolved points)

- **FR-053-22** (documented choice, `isunix.q-platform`, `ismac.q-platform`)
  `isunix` is `true` on every Unix-family host RunMat is compiled for
  (`cfg!(unix)`: Linux, macOS, BSD) and `false` elsewhere (Windows,
  wasm32). `ismac` is `true` only on macOS (`target_os = "macos"`). Only
  the Linux values are observed.
- **FR-053-23** (documented choice, all five) More than one requested output
  raises `RunMat:<fn>:TooManyOutputs`. `isMATLABReleaseOlderThan` with more
  than one input raises `RunMat:isMATLABReleaseOlderThan:UnsupportedOption`
  (`isMATLABReleaseOlderThan.q-release`: such calls were not exercised).
  `chdir` with more than one input raises `RunMat:chdir:InvalidInput`.
- **FR-053-24** (documented choice, isMATLABReleaseOlderThan) Release name
  grammar: `R` or `r`, exactly four digits, then `a` or `b` (either case),
  nothing else (no spaces, no update suffix). Any other text, including
  empty text, raises `invalidReleaseInput`. Non-text inputs (numeric,
  logical, cell, struct, …), char matrices with more than one row and string
  arrays with more than one element raise
  `RunMat:validators:mustBeTextScalar`. Releases compare by year, then
  `a` < `b`.
- **FR-053-25** (documented choice, chdir) Every non-text `folder` (logical,
  cell, struct, …) raises `RunMat:string`, like the observed numeric case.
  Empty text raises `cd`'s `RunMat:cd:EmptyFolder`. A path to an existing
  file that is not a folder raises `RunMat:cd:NonExistentFolder`. Other
  failures to enter an existing folder keep `cd`'s
  `RunMat:cd:ChangeFailed`. A `chdir` call with no input and no output
  returns the current folder like `d = chdir` (`chdir.q-display`: the
  display form was not observed).
- **FR-053-26** (documented choice, date) The day is always zero-padded to
  two digits (`date.q-zero-pad`), month names are the English abbreviations
  `Jan`…`Dec` for every month and locale (`date.q-other-months`,
  `date.q-locale`), and the date is the local date reported by `clock`.
- **FR-053-27** (unresolved, `*.q-warning-coverage` for all five) None of
  the five functions emits a warning.

## Success Criteria *(mandatory)*

- **SC-053-1**: all 29 observed cases reproduce on a Linux host (class,
  size, value or error identifier): isunix 4, ismac 2,
  isMATLABReleaseOlderThan 10, chdir 8, date 5.
- **SC-053-2**: the five builtins are registered through `#[runtime_builtin]`
  and `cargo test -p runmat-runtime --lib -- isunix_tests ismac_tests
  is_matlab_release_older_than_tests chdir_tests date_tests` passes.
- **SC-053-3**: every FR maps to at least one test (`validation.md`).

## Unresolved behaviour (returned to the specification side)

| id | disposition |
|---|---|
| isunix.q-platform, ismac.q-platform | host platform reported (FR-053-22, documented choice) |
| isMATLABReleaseOlderThan.q-release | compared with the compatibility pin (D-053-3); more than one input → `UnsupportedOption` (FR-053-23) |
| chdir.q-display | returns the folder like `d = chdir` (FR-053-25) |
| date.q-zero-pad, date.q-other-months, date.q-locale | zero-padded day, English abbreviations (FR-053-26) |
| *.q-warning-coverage (5) | no warning emitted (FR-053-27) |
