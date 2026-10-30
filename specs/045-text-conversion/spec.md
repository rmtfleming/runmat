# Feature Specification: text conversion — `regexptranslate`, `dec2base`, `validatestring`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`045-text-conversion`; export `SPECIFY_FEATURE=045-text-conversion`)

**Created**: 2026-10-30

**Status**: Batch N1, feature F45. Gates G1–G3 pre-approved by the maintainer
(`specs/001-clean-room-matlab-builtins/selection-proposal-n1.md`, "Start N1,
gates pre-approved").

**Input**: Approved N1 exports `regexptranslate`, `dec2base` and
`validatestring`, all **0.2.0** (see `provenance.md`).

Family rationale (Principle VII): three small text-producing helpers from
the same approved batch. Each one takes text or numbers and returns text,
and none of them shares state with the others. They are grouped only to keep
the N1 feature count small. Each has its own file, tests and requirement
block.

## Decisions

- **D-045-1 Error identifier namespace.** Observed `MATLAB:x:y` identifiers
  are raised as `RunMat:x:y` (maintainer N1 rule). This gives
  `RunMat:REGEXP:invalidOptions`, `RunMat:dec2base:FirstArg` and
  `RunMat:dec2base:SecondArg`. For `validatestring`, RunMat follows the
  `validateattributes` convention (spec 038, decision D2 = B):
  `RunMat:validatestring:<observed suffix>`. funcName and varName change
  only the message, never the identifier. Identifiers that were not observed
  (`NotEnoughInputs`, `TooManyInputs`, `UnsupportedOption`) use
  `RunMat:<fn>:<name>`. **Flag for maintainer review:** the literal mapping
  keeps the upper-case `REGEXP` component.
- **D-045-2 `dec2base` char bases (flagged open question).** A char base
  raises `RunMat:dec2base:UnsupportedOption`, with one exception. A
  two-character char row raises `RunMat:dec2base:SecondArg`, because the
  observed case `err-symbols` (`'ab'`) is normative. String bases also raise
  `UnsupportedOption`.
- **D-045-3 Unobserved points** raise `RunMat:<fn>:UnsupportedOption` or are
  labelled documented choices below. Neither kind is presented as MATLAB
  behaviour.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Turn literal text or wildcards into a regex (Priority: P1)

**Acceptance Scenarios** (normative, one per observed case):

1. `regexptranslate('escape','a.b')` → char `'a\.b'` 1×4 (`escape-dot`).
   [regexptranslate.signature-primary, regexptranslate.escape]
2. `regexptranslate('escape','$^*+?()[]{}|\')` → each character prefixed
   with `\`, 1×26 (`escape-specials`). [regexptranslate.escape]
3. `regexptranslate('escape','abc')` → `'abc'` (`escape-plain`).
   [regexptranslate.escape]
4. `regexptranslate('escape','-<>#/ ,:=!@')` → `'\-<>#/ ,:=!@'` 1×12
   (`escape-others`). [regexptranslate.escape]
5. Tab and newline are left unchanged, 1×5 (`escape-whitespace`).
   [regexptranslate.escape-whitespace]
6. `regexptranslate('wildcard', …)`: `'*.m'` → `'.*\.m'`; `'a?c'` → `'a.c'`;
   `'a.b'` → `'a\.b'` (`wildcard-star`, `wildcard-question`,
   `wildcard-dot`). [regexptranslate.wildcard]
7. `regexp('file.m', ['^' regexptranslate('wildcard','*.m') '$'], 'once')`
   is non-empty (`wildcard-use`). [regexptranslate.wildcard-use]
8. String scalar in → string scalar out. Cell in → cell of the same size,
   translated element by element. `''` → 0×0 char (`string-input`,
   `cell-input`, `empty`). [regexptranslate.types]
9. `regexptranslate('nope','a')` raises `RunMat:REGEXP:invalidOptions`
   (`err-op`). [regexptranslate.error-op]

### User Story 2 - Write integers in another base (Priority: P1)

1. `dec2base(23,2)` → `'10111'`; `(23,3)` → `'212'`; `(255,16)` → `'FF'`;
   `(35,36)` → `'Z'`; `(uint8(200),16)` → `'C8'`, all char rows (`base2`,
   `base3`, `base16`, `base36-z`, `uint8`). [dec2base.signature-primary,
   dec2base.output-class, dec2base.digits, dec2base.integer-class]
2. `dec2base(0,7)` → `'0'` (`zero`). [dec2base.zero]
3. `dec2base(5,2,8)` → `'00000101'`; `dec2base(255,2,3)` → `'11111111'`
   (`pad`, `pad-short`). [dec2base.padding]
4. `dec2base([1;10;100],10)` → `['001';'010';'100']` 3×3;
   `dec2base([1 2 3],2)` → `['01';'10';'11']` 3×2;
   `dec2base([35 36],36)` → `['0Z';'10']` 2×2 (`vector`, `row-vector`,
   `base36`). [dec2base.vector, dec2base.output-class]
5. `dec2base(2^52,16)` → `'10000000000000'` 1×14 (`large`). [dec2base.large]
6. `dec2base(-1,2)` and `dec2base(1.5,2)` raise `RunMat:dec2base:FirstArg`
   (`err-negative`, `err-fraction`). [dec2base.error-negative]
7. `dec2base(5,1)`, `dec2base(5,37)` and `dec2base(5,'ab')` raise
   `RunMat:dec2base:SecondArg` (`err-base1`, `err-base37`, `err-symbols`).
   [dec2base.error-base1]

### User Story 3 - Validate a text choice (Priority: P1)

1. `validatestring('apple',{'apple','banana'})` → `'apple'` (`exact`).
   [validatestring.signature-primary, validatestring.exact]
2. `'ban'` → `'banana'` (`prefix`). [validatestring.prefix]
3. `'APP'` → `'apple'` (`case`). [validatestring.case-insensitive]
4. `'app'` and `'ap'` with `{'app','apple'}` → `'app'` (`exact-over-prefix`,
   `prefix-of-several-nested`). [validatestring.nested-options]
5. `"ban"` with a cellstr list → char `'banana'`. `"ap"` with
   `["apple" "banana"]` → string `"apple"` (`string-input`, `string-list`).
   [validatestring.output-type]
6. `'b'` with `{'banana','berry'}` raises
   `RunMat:validatestring:ambiguousStringChoice` (`err-ambiguous`).
   [validatestring.error-ambiguous]
7. `'kiwi'`, `''` and `5` raise `RunMat:validatestring:unrecognizedStringChoice`
   (`err-nomatch`, `err-empty-input`, `err-nonchar`).
   [validatestring.error-nomatch]
8. `('kiwi',{…},'myfun','fruit')` raises
   `RunMat:validatestring:unrecognizedStringChoice`, and the message names
   `myfun` and `fruit` (`err-funcname`). [validatestring.error-funcname]
9. `("ban",{"apple","banana"})` raises
   `RunMat:validatestring:invalidStringList` (`err-cell-of-strings`).
   [validatestring.error-cell-of-strings]

## Requirements *(mandatory)*

### Functional Requirements — `regexptranslate` (normative)

- **FR-045-01** `p = regexptranslate(op, s)` MUST be accepted
  [regexptranslate.signature-primary].
- **FR-045-02** `'escape'` MUST put a backslash before each of
  `. $ ^ * + ? ( ) [ ] { } | \ -`. It MUST leave letters and
  `< > # / space , : = ! @` unchanged [regexptranslate.escape], and also tab
  and newline [regexptranslate.escape-whitespace].
- **FR-045-03** `'wildcard'` MUST map `*` → `.*`, `?` → `.` and `.` → `\.`
  [regexptranslate.wildcard]. The result, anchored, MUST work as a pattern
  for RunMat's `regexp` [regexptranslate.wildcard-use].
- **FR-045-04** A char row gives char, a string scalar gives a string
  scalar, and a cell gives a same-size cell translated element by element.
  Empty char text gives a 0×0 char [regexptranslate.types].
- **FR-045-05** An unrecognised op MUST raise `RunMat:REGEXP:invalidOptions`
  [regexptranslate.error-op].

### Functional Requirements — `dec2base` (normative)

- **FR-045-06** `str = dec2base(d, base)` and `dec2base(d, base, n)` MUST be
  accepted and return char [dec2base.signature-primary,
  dec2base.output-class].
- **FR-045-07** Digits are `0-9` then upper-case `A-Z` [dec2base.digits].
  0 gives `'0'` [dec2base.zero].
- **FR-045-08** `n` left-pads with `'0'` to at least `n` digits and never
  truncates [dec2base.padding].
- **FR-045-09** Several values give one row per element, all left-padded
  with `'0'` to a common width, for both row and column input
  [dec2base.vector].
- **FR-045-10** Integer-class `d` is accepted [dec2base.integer-class]. The
  double `2^52` converts exactly [dec2base.large].
- **FR-045-11** Negative or non-integer `d` MUST raise
  `RunMat:dec2base:FirstArg` [dec2base.error-negative]. A base of 1 or 37
  (outside 2–36), or the two-character char `'ab'`, MUST raise
  `RunMat:dec2base:SecondArg` [dec2base.error-base1].

### Functional Requirements — `validatestring` (normative)

- **FR-045-12** `str = validatestring(txt, validStrings)` and the 4-argument
  form with `funcName, varName` MUST be accepted
  [validatestring.signature-primary, validatestring.error-funcname].
- **FR-045-13** Matching is case-insensitive and by prefix. An exact match,
  or a unique prefix match, returns the option as spelled in the list
  [validatestring.exact, validatestring.prefix,
  validatestring.case-insensitive].
- **FR-045-14** When several options match and one of them is a prefix of
  all the others, that option (the shortest) is returned
  [validatestring.nested-options]. Otherwise the call MUST raise
  `RunMat:validatestring:ambiguousStringChoice`
  [validatestring.error-ambiguous].
- **FR-045-15** The result is char for a cellstr list (for char or string
  `txt` alike) and a string scalar for a string-array list
  [validatestring.output-type].
- **FR-045-16** No match, empty text or numeric text MUST raise
  `RunMat:validatestring:unrecognizedStringChoice`
  [validatestring.error-nomatch]. With funcName given, the identifier is
  the same and funcName/varName appear in the message
  [validatestring.error-funcname] (D-045-1).
- **FR-045-17** A cell list holding string scalars MUST raise
  `RunMat:validatestring:invalidStringList`
  [validatestring.error-cell-of-strings].

### Functional Requirements (documented choices and unresolved points)

- **FR-045-18** (documented choice, regexptranslate)
  - Arity other than 2 raises `RunMat:regexptranslate:NotEnoughInputs` or
    `…:TooManyInputs`.
  - `op` must be a char row or string scalar.
  - An op that equals `escape`/`wildcard` only when case is ignored (e.g.
    `'ESCAPE'`) raises `RunMat:regexptranslate:UnsupportedOption`
    (regexptranslate.q-other-ops). Any other op text raises
    `RunMat:REGEXP:invalidOptions`, extending the observed `'nope'` case.
    A non-text op raises `UnsupportedOption`.
- **FR-045-19** (documented choice, regexptranslate)
  - In `'escape'`, characters outside the observed lists (digits, `_`, `~`,
    `%`, `'`, `"`, `;`, `&`, non-ASCII, …) are copied unchanged.
  - In `'wildcard'`, every character other than `* ? .` is copied unchanged.
- **FR-045-20** (documented choice, regexptranslate)
  - Any empty char (0×0 or 1×0) gives 0×0 char. A cell element `''` gives
    0×0 char.
  - A multi-element string array is translated element by element, keeping
    its shape.
  - Unsupported `s`, raising `RunMat:regexptranslate:UnsupportedOption`:
    multi-row char, cell elements that are not char rows, and any other
    class.
- **FR-045-21** (documented choice, dec2base)
  - Arity other than 2–3 raises `RunMat:dec2base:NotEnoughInputs` or
    `…:TooManyInputs`.
  - `d` is checked before `base`, and `base` before `n`.
  - `d` may be any shape. Elements are taken in column-major order, one row
    each (the vector rule, extended).
  - Empty `d` gives 0×0 char.
  - NaN and ±Inf raise `FirstArg`, since they are not non-negative integers.
  - Logical, single, complex, char, string and other classes for `d` raise
    `RunMat:dec2base:UnsupportedOption`.
- **FR-045-22** (documented choice, dec2base) `base` may be a double or
  integer-class scalar. A non-integer, non-scalar or out-of-range numeric
  base raises `SecondArg`. A logical base raises `UnsupportedOption`. Char
  and string bases follow D-045-2.
- **FR-045-23** (unresolved, dec2base.q-large-values) Values above `2^52`, of
  either class, raise `RunMat:dec2base:UnsupportedOption`.
- **FR-045-24** (documented choice, dec2base) `n` must be a non-negative
  integer-valued real scalar (double or integer class). Anything else
  raises `RunMat:dec2base:UnsupportedOption`, since no identifier was
  observed.
- **FR-045-25** (documented choice, validatestring)
  - Fewer than 2 arguments raise `RunMat:validatestring:NotEnoughInputs`.
  - The 3-argument form (funcName only) is accepted.
  - funcName and varName must be text, otherwise `UnsupportedOption`.
- **FR-045-26** (unresolved, validatestring.q-more-args) More than 4
  arguments raise `RunMat:validatestring:UnsupportedOption`.
- **FR-045-27** (documented choice, validatestring)
  - The list is checked before the text.
  - Lists that are neither a cell of char rows nor a string array raise
    `invalidStringList`. This covers a cell containing any non-char-row
    element, a char array and numeric values.
  - A string scalar list counts as a one-element string array, so the
    result is a string.
  - An empty cell or empty string array matches nothing, so it raises
    `unrecognizedStringChoice`.
  - Non-text `txt` of any class raises `unrecognizedStringChoice`, as do a
    multi-row char and a multi-element string array (the `err-nonchar`
    rule, extended).
  - When options tie (for example duplicates, or options equal up to case),
    the first listed option wins.
- **FR-045-28** (unresolved, `*.q-warning-coverage` for all three) None of
  the three functions emits a warning.

## Success Criteria *(mandatory)*

- **SC-045-1**: all 43 observed cases reproduce (class, size, value or
  error identifier): 13 for regexptranslate, 17 for dec2base and 13 for
  validatestring.
- **SC-045-2**: the three builtins are registered through
  `#[runtime_builtin]`, and `cargo test -p runmat-runtime --lib --
  regexptranslate_tests dec2base_tests validatestring_tests` passes.
- **SC-045-3**: every FR maps to at least one test (`validation.md`).

## Unresolved behaviour (returned to the specification side)

| id | disposition |
|---|---|
| regexptranslate.q-warning-coverage | no warning emitted (FR-045-28) |
| regexptranslate.q-other-ops | case variants of the two ops → `UnsupportedOption`; other text → `invalidOptions` (documented extension of `err-op`) (FR-045-18) |
| dec2base.q-warning-coverage | no warning emitted |
| dec2base.q-large-values | values > 2^52 → `UnsupportedOption` (FR-045-23) |
| dec2base.q-char-base (flagged) | char bases → `UnsupportedOption`, apart from the observed two-character `'ab'` → `SecondArg` (D-045-2) |
| validatestring.q-warning-coverage | no warning emitted |
| validatestring.q-more-args | > 4 arguments → `UnsupportedOption` (FR-045-26) |
