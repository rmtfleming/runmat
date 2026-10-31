# Feature Specification: text parsing — `strread`, `asManyOfPattern`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`057-text-parsing`; export `SPECIFY_FEATURE=057-text-parsing`)

**Created**: 2026-10-30

**Status**: Batches N3 + N4, feature F57. Gates G1–G3 pre-approved by the
maintainer (`specs/001-clean-room-matlab-builtins/selection-proposal-n3n4.md`).

**Input**: Approved exports `strread` **0.2.0** (N3, source `1156c97`) and
`asManyOfPattern` **0.2.0** (N4, source `fc9f804`); see `provenance.md`.

Family rationale (Principle VII): two COBRA-backlog text-parsing builtins.
`strread` splits text into typed fields; `asManyOfPattern` builds a
repetition pattern for the string-pattern functions. They share no state.
Each has its own file, tests and requirement block.

## Decisions

- **D-057-1 Error identifier namespace.** Observed `MATLAB:x:y`
  identifiers are raised as `RunMat:x:y` (maintainer rule):
  `RunMat:textio:dataread:SecondInputFilenameOrString`,
  `RunMat:textio:dataread:TroubleReading`,
  `RunMat:pattern:InvalidLimitRange`,
  `RunMat:pattern:MustBeNonnegativeInteger`,
  `RunMat:pattern:MustBeTextOrPattern`. The `strread` identifiers name
  another function (`dataread`); whether they must be reproduced is open on
  the spec side (`strread.q-error-ids`). RunMat follows the observation and
  flags the question to the maintainer. Identifiers that were not observed
  use `RunMat:<fn>:<name>`.
- **D-057-2 `strread` has its own tokenizer.** The brief asked to reuse the
  `textread` parser (spec 052) only through its public items. It has none:
  `parse_format`, `tokens`, `read_fields` are private and raise
  `RunMat:textread:*` identifiers, and `read_fields` rejects uneven records,
  which `strread` must accept (`two-out-uneven`). Making them `pub(crate)`
  would not be enough, so `textread.rs` is not edited. `strread` has a
  ~150-line tokenizer of its own written for this feature.
- **D-057-3 `pattern` representation and missing upstream functions.**
  RunMat represents a `pattern` as a host object of class `pattern` holding
  a regular expression (`strings::core::compat::pattern_object` /
  `pattern_regex`, used by `pattern`, `digitsPattern`, `lettersPattern`,
  `matches`, `contains`, …). `asManyOfPattern` wraps that regex in a greedy
  quantifier. RunMat has no `extract`, no `alphanumericsPattern` and no `|`
  operator on patterns. The observed cases that use them are reproduced at
  regex level in the tests: a test helper performs `extract`
  (leftmost-first, non-overlapping matches of the pattern's regex), and
  `alphanumericsPattern | "_"` is replaced by
  `regexpPattern("[\p{Alphabetic}\p{N}]+|_")`. These are upstream gaps
  (deviation, see `validation.md`), not changes made here.
- **D-057-4 Unobserved points** raise `RunMat:<fn>:UnsupportedOption` or are
  labelled documented choices below. Neither kind is presented as MATLAB
  behaviour.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Read fields from text (Priority: P1)

**Acceptance Scenarios** (normative, one per observed case):

1. `strread('1 2 3')` → double row `[1 2 3]` (`default`).
   [strread.signature-primary, strread.default]
2. `strread('ab cd ef','%s')` → 3×1 cell `{'ab';'cd';'ef'}`;
   `strread(num2str([10 2 300]),'%s')` → `{'10';'2';'300'}` (`s`,
   `num2str-form`). [strread.s]
3. `strread('4 5 6','%d')` → `[4;5;6]`; `strread('1.5 2.25','%f')` →
   `[1.5;2.25]`; `strread('7 8','%d')` → `[7;8]` (`d`, `f`, `orientation`).
   [strread.numeric]
4. `[a,b] = strread('x!y!z!w','%s%s','delimiter','!')` →
   `a = {'x';'z'}`, `b = {'y';'w'}`; `'p 1 q 2'` with `'%s%d'` →
   `{'p';'q'}`, `[1;2]`; `'1 2 3'` with `'%d%d'` → `[1;3]`, `2`
   (`two-out`, `two-out-mixed`, `two-out-uneven`). [strread.cyclic]
5. `strread('a,b,c','%s','delimiter',',')` → `{'a';'b';'c'}`;
   `strread('a b, c d','%s','delimiter',',')` → `{'a b';'c d'}`
   (`delimiter-comma`, `delimiter-spaces`). [strread.delimiter]
6. `strread('','%s')` → 0×1 cell (`empty`). [strread.empty]
7. `strread('1 2')` → `[1 2]` with no warning (`warn`).
   [strread.no-warning]
8. `strread(5)` raises `RunMat:textio:dataread:SecondInputFilenameOrString`
   (`err-numeric`). [strread.error-numeric]
9. `strread('a b','%d')` raises `RunMat:textio:dataread:TroubleReading`
   (`err-bad-format`). [strread.error-bad-format]

### User Story 2 - Repetition patterns (Priority: P1)

**Acceptance Scenarios** (normative; `extract` and `alphanumericsPattern |
"_"` per D-057-3):

1. `class(asManyOfPattern(alphanumericsPattern | "_", 1))` is `'pattern'`
   (`class`). [asManyOfPattern.signature-primary, asManyOfPattern.class]
2. With `P = asManyOfPattern(alphanumericsPattern | "_", 1)`:
   `extract("ab_1 cd-e", P)` → `["ab_1";"cd";"e"]`; on `"--"` → 0×1;
   `matches(["ab_1" "ab 1" ""], P)` → `[true false false]`;
   `contains(["--" "-a-"], P)` → `[false true]` (`cobra-extract`,
   `min1-no-empty`, `matches`, `contains`). [asManyOfPattern.cobra-pattern]
3. `extract("ab cd", asManyOfPattern(lettersPattern(1)))` → `["ab";"cd"]`
   (`default-min`). [asManyOfPattern.greedy]
4. `extract("1 22 333 4444", asManyOfPattern(digitsPattern(1), 2, 3))` →
   `["22";"333";"444"]` (`min-max`). [asManyOfPattern.min-max]
5. With min 2, `"a aa aaa"` gives `["aa";"aaa"]` for string `"a"` and char
   `'a'` (`string-input-min`, `char-input`).
   [asManyOfPattern.text-pattern]
6. `asManyOfPattern("a",3,2)` raises `RunMat:pattern:InvalidLimitRange`;
   `asManyOfPattern("a",-1)` raises
   `RunMat:pattern:MustBeNonnegativeInteger`; `asManyOfPattern(5)` raises
   `RunMat:pattern:MustBeTextOrPattern` (`err-min-gt-max`, `err-negative`,
   `err-numeric`). [asManyOfPattern.error-*]
7. *(non-normative)* `extract("aaab", asManyOfPattern("a"))` and
   `extract("ab cd", asManyOfPattern("b"))` (`string-input`,
   `default-min-string-letters`). [asManyOfPattern.empty-matches]

## Requirements *(mandatory)*

### Functional Requirements — `strread` (normative)

- **FR-057-01** `A = strread(str)` MUST be accepted and return the
  whitespace-separated numbers as a 1×n double row
  [strread.signature-primary, strread.default].
- **FR-057-02** `%s` returns a column cell of char rows, one per field
  [strread.s].
- **FR-057-03** `%d` and `%f` return double columns [strread.numeric].
- **FR-057-04** With k conversions, field i (0-based) goes to output
  `mod(i,k)+1`; outputs may have different lengths [strread.cyclic].
- **FR-057-05** With `'delimiter', d`, fields are split only at `d`; inner
  spaces are kept and the whitespace around a field is not part of it
  [strread.delimiter].
- **FR-057-06** Empty text with `'%s'` gives a 0×1 cell [strread.empty].
- **FR-057-07** Numeric `str` MUST raise
  `RunMat:textio:dataread:SecondInputFilenameOrString`
  [strread.error-numeric].
- **FR-057-08** A field that cannot be read with a numeric conversion MUST
  raise `RunMat:textio:dataread:TroubleReading` [strread.error-bad-format].
- **FR-057-09** `strread('1 2')` emits no warning [strread.no-warning].

### Functional Requirements — `asManyOfPattern` (normative)

- **FR-057-13** `pat = asManyOfPattern(p)`, `(p, min)` and `(p, min, max)`
  MUST be accepted and return an object of class `pattern`
  [asManyOfPattern.signature-primary, asManyOfPattern.class].
- **FR-057-14** With `min` given, at least `min` repetitions are required:
  the COBRA pattern with min 1 extracts `["ab_1";"cd";"e"]`, nothing from
  `"--"`, matches `[true false false]` and contains `[false true]` on the
  observed inputs [asManyOfPattern.cobra-pattern].
- **FR-057-16** Repetitions are taken greedily; with no `min`, zero or more
  repetitions [asManyOfPattern.greedy].
- **FR-057-17** With `min` and `max`, between `min` and `max` repetitions
  are taken, as many as possible [asManyOfPattern.min-max].
- **FR-057-18** Text `p` (string scalar or char row) is accepted and
  matched literally [asManyOfPattern.text-pattern].
- **FR-057-19** `min` greater than `max` MUST raise
  `RunMat:pattern:InvalidLimitRange` [asManyOfPattern.error-min-gt-max].
- **FR-057-21** A negative count MUST raise
  `RunMat:pattern:MustBeNonnegativeInteger`
  [asManyOfPattern.error-negative].
- **FR-057-22** Numeric `p` MUST raise `RunMat:pattern:MustBeTextOrPattern`
  [asManyOfPattern.error-numeric].

(FR-057-10..12, 15 and 20 are not used.)

### Functional Requirements (documented choices and unresolved points)

- **FR-057-25** (documented choice, strread) Empty text with no format gives
  a 1×0 double; with `%d`/`%f` a 0×1 double. The no-format form parses
  fields like `%f`, and an unreadable field raises `TroubleReading`.
- **FR-057-26** (documented choice, strread, `strread.q-more-options`)
  - The format may contain only `%s`, `%d`, `%f`, optionally separated by
    whitespace. Other conversions, `%*…`, widths and literal text raise
    `RunMat:strread:UnsupportedOption`.
  - The only option is `'delimiter'` (name matched case-insensitively), with
    a single non-whitespace character. Other options, repeat counts,
    multi-character or whitespace delimiters and empty fields between
    delimiters raise `UnsupportedOption`.
  - `%d` accepts an optionally signed digit string. A field that is a number
    but not an integer (e.g. `1.5`) raises `UnsupportedOption`; a field that
    is not a number raises `TroubleReading` (FR-057-08). `%f` accepts what
    Rust's `f64` parser accepts (including `Inf`/`NaN`).
- **FR-057-27** (documented choice, asManyOfPattern,
  `asManyOfPattern.q-empty-match`) The pattern is the regex `(?:p)*`,
  `(?:p){min,}` or `(?:p){min,max}`. Where empty matches appear is decided
  by the consuming function's regex engine (the `regex` crate:
  leftmost-first, non-overlapping, an empty match directly after a previous
  match is skipped). For the two non-normative observed cases this gives
  the observed outputs, but RunMat does not claim it is MATLAB's rule.
- **FR-057-28** (documented choice, asManyOfPattern) Counts may be any real
  numeric scalar (double, single or integer class) with a non-negative
  integer value. Non-integer, NaN and `Inf` `min` raise
  `MustBeNonnegativeInteger`, and so does a negative `max`. `max = Inf`
  means no upper bound. `min = max = 0` is allowed.
- **FR-057-29** (documented choice, both) Arity and outputs:
  - `strread` with no input raises `RunMat:strread:InvalidInput`, and so
    does an option name without a value. More outputs than conversions (or
    more than one with no format) raise `RunMat:strread:TooManyOutputs`;
    fewer outputs return the first ones.
  - `asManyOfPattern` takes one to three inputs (else
    `RunMat:asManyOfPattern:InvalidInput`) and returns one output (more →
    `RunMat:asManyOfPattern:TooManyOutputs`).
- **FR-057-30** (documented choice, both) Unobserved classes raise
  `RunMat:<fn>:UnsupportedOption`:
  - `strread`: `str` that is not a char row (string scalars, logical,
    cell, multi-row char); a format or option that is not a char row.
  - `asManyOfPattern`: `p` that is logical, a cell or a multi-element
    string array; counts that are not numeric scalars.
  gpuArray inputs are gathered first.
- **FR-057-31** (unresolved, `*.q-warning-coverage`) Neither function emits
  a warning.

## Success Criteria *(mandatory)*

- **SC-057-1**: all observed cases reproduce: 15 for `strread`; for
  `asManyOfPattern` the 12 normative cases (with the D-057-3 stand-ins)
  plus the 2 non-normative cases as a labelled documented-choice test.
- **SC-057-2**: both builtins are registered through `#[runtime_builtin]`,
  and `cargo test -p runmat-runtime --lib -- strread_tests
  as_many_of_pattern_tests` passes.
- **SC-057-3**: every FR maps to at least one test (`validation.md`).

## Unresolved behaviour (returned to the specification side)

| id | disposition |
|---|---|
| strread.q-warning-coverage, asManyOfPattern.q-warning-coverage | no warning emitted (FR-057-31) |
| strread.q-more-options | `UnsupportedOption` for everything unobserved (FR-057-26) |
| strread.q-error-ids | observed identifiers reproduced in the RunMat namespace (D-057-1); flagged to maintainer |
| asManyOfPattern.q-empty-match | labelled documented choice FR-057-27 (regex-engine iteration); not claimed as MATLAB behaviour |
