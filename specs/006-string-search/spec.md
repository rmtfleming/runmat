# Feature Specification: String Search — `matches`, `strmatch`, `strtok`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id: `006-string-search`; export `SPECIFY_FEATURE=006-string-search`)

**Created**: 2026-07-22

**Status**: Implemented (see `validation.md`)

**Input**: Approved Tier A exports `matches` 0.2.0, `strmatch` 0.2.0,
`strtok` 0.2.0 (see `provenance.md`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Test text equality against a pattern set (Priority: P1)

RunMat users calling `matches(str, pat)` get a logical result that is true
when the text equals the pattern or any element of a set of patterns, with an
optional `'IgnoreCase'` name-value pair, matching the approved observed
behaviour.

**Independent Test**: run each observed case from the export through the
builtin and compare value, class and shape.

**Acceptance Scenarios** (from observed results, normative):

1. **Given** the string scalar `"hello"` and the pattern set
   `["hello","world"]`, **When** `matches` is called, **Then** the result is
   logical 1×1 `true`. [matches.signature-primary, matches.output-class,
   matches.scalar-membership]
2. **Given** char vectors `'abc'` and `'abc'`, **When** `matches` is called,
   **Then** the result is logical 1×1 `true`. [matches.output-class,
   matches.scalar-membership]
3. **Given** `"Cat"` and `"cat"` with `'IgnoreCase', true`, **When**
   `matches` is called, **Then** the result is logical 1×1 `true`.
   [matches.output-class, matches.scalar-membership]

### User Story 2 - Find text rows by prefix or exact match (Priority: P2)

`strmatch(str, arr)` returns a double column vector of the 1-based indices of
the entries of `arr` that begin with `str`; the `'exact'` flag requires full
equality; no match yields a 0×1 empty double.

**Acceptance Scenarios** (from observed results, normative):

1. **Given** `'ap'` and `{'apple';'apricot';'banana'}`, **When** `strmatch`
   is called, **Then** the result is double 2×1 `[1;2]`.
   [strmatch.signature-primary, strmatch.output-class,
   strmatch.index-vector]
2. **Given** `'apple'` and `{'apple';'apple pie'}` with the `'exact'` flag,
   **When** `strmatch` is called, **Then** the result is double 1×1 `1`.
   [strmatch.output-class, strmatch.index-vector]
3. **Given** `'zz'` and `{'apple';'banana'}`, **When** `strmatch` is called,
   **Then** the result is a double 0×1 empty. [strmatch.output-class,
   strmatch.index-vector]

### User Story 3 - Extract the first delimited token (Priority: P2)

`strtok(str)` returns the first whitespace-delimited token as char, skipping
leading delimiters; `strtok(str, delimiters)` uses a supplied delimiter set;
a second output returns the remainder.

**Acceptance Scenarios** (from observed results, normative):

1. **Given** `'hello world'`, **When** `strtok` is called, **Then** the first
   output is char 1×5 `'hello'`. [strtok.signature-primary,
   strtok.output-class, strtok.leading-token]
2. **Given** `'a,b,c'` and delimiter `','`, **When** `strtok` is called,
   **Then** the first output is char 1×1 `'a'`. [strtok.output-class,
   strtok.leading-token]
3. **Given** `'   lead trail'`, **When** `strtok` is called, **Then** the
   first output is char 1×4 `'lead'` (leading delimiters skipped).
   [strtok.output-class, strtok.leading-token]
4. **Given** `'hello world'` and two requested outputs, **When**
   `[tok, rem] = strtok(...)` is called, **Then** `rem` holds the remainder
   of the text. *(Summary-derived: the approved summary states a second
   output returns the remainder; no backing observation case records its
   value — see Unresolved. Test tagged `summary_derived`.)*

## Requirements *(mandatory)*

### Functional Requirements

- **FR-006-01** `matches` MUST accept two text arguments
  (`tf = matches(str, pat)`) [matches.signature-primary] and return class
  logical [matches.output-class]; a scalar first argument yields a 1×1
  result that is `true` exactly when it equals any of the given patterns
  [matches.scalar-membership].
- **FR-006-02** `matches` MUST accept the `'IgnoreCase', true` name-value
  pair enabling case-insensitive equality [matches.scalar-membership,
  matches.output-class].
- **FR-006-03** Inputs not covered by the export (array first arguments,
  case-sensitivity default, missing strings, non-text inputs, positional
  option forms) MUST be handled by documented independent choice (see
  Unresolved behaviour) and MUST NOT be asserted as MATLAB-conformant.
- **FR-006-04** `strmatch` MUST accept a text pattern and a text array
  (`idx = strmatch(str, arr)`) [strmatch.signature-primary] and return class
  double [strmatch.output-class] as a column vector of 1-based indices of
  the entries beginning with the pattern; no match MUST yield a 0×1 empty
  double [strmatch.index-vector].
- **FR-006-05** `strmatch` MUST accept the `'exact'` flag as a third
  argument, requiring full equality instead of prefix matching
  [strmatch.index-vector, strmatch.output-class].
- **FR-006-06** `strmatch` inputs not covered by the export (char-matrix and
  string-array search arrays, other flag values, non-text inputs) MUST be
  handled by documented independent choice and MUST NOT be asserted as
  MATLAB-conformant.
- **FR-006-07** `strtok` MUST accept one text argument
  (`token = strtok(str)`) [strtok.signature-primary] and return class char
  for char input [strtok.output-class]; the first output MUST be the leading
  token with whitespace as the default delimiter set and leading delimiters
  skipped [strtok.leading-token].
- **FR-006-08** `strtok` MUST accept a second argument supplying the
  delimiter set (`strtok(str, delimiters)`) [strtok.leading-token,
  strtok.output-class].
- **FR-006-09** (summary-derived) `strtok` SHOULD return a second output
  `remain` holding the remainder of the text, per the approved summary; the
  exact delimiter handling of the remainder is not observed and is
  implemented as a documented choice (remainder starts at the delimiter that
  terminated the token).
- **FR-006-10** `strtok` behaviour not covered by the export (empty input,
  all-delimiter input, string inputs, the precise default whitespace set)
  MUST be handled by documented independent choice and MUST NOT be asserted
  as MATLAB-conformant.

### Key Entities

- `matches` result: `Value::Bool` (1×1) or `Value::LogicalArray` (RunMat's
  logical array) shaped like the first argument.
- `strmatch` result: `Value::Tensor` (double) column vector, including the
  0×1 empty case.
- `strtok` results: `Value::CharArray` for char input, `Value::String` for
  string input; multi-output delivery through RunMat's
  `BuiltinOutputMode::ByRequestedOutputCount` / `output_count` machinery.

## Success Criteria *(mandatory)*

- **SC-006-1**: Every observed case in the three exports reproduces exactly
  (value, class, size) in normative tests.
- **SC-006-2**: All three builtins registered under
  `crates/runmat-runtime/src/builtins/strings/search/` and discoverable via
  the builtin registry; `cargo test -p runmat-runtime` passes for their test
  modules.
- **SC-006-3**: Traceability: each FR cites ≥1 claim id; each normative test
  cites ≥1 FR.

## Unresolved behaviour (non-normative; implemented by documented choice)

- `matches.q-release`: introduction release unrecorded; applicable release
  noted as R2026a only.
- `matches.q-string-vs-char`: string/char differences unconfirmed. Choice:
  both accepted through the shared `TextCollection` text handling used by
  `contains`/`startsWith`.
- `matches` array first argument: unobserved. Choice: result is a logical
  array shaped like the first argument, each element tested for equality
  against the whole pattern set (element-wise generalisation of the observed
  scalar membership rule, consistent with `contains.rs` input handling).
- `matches` default case sensitivity: only the `'IgnoreCase', true` form is
  observed. Choice: case-sensitive comparison by default.
- `strmatch.q-deprecated`: whether `strmatch` warns as legacy is unrecorded.
  Choice: no warning emitted.
- `strmatch.q-shape`: orientation confirmed column (observed sizes 2×1, 1×1,
  0×1); n-D arrays unobserved — linear (column-major) element order used.
- `strmatch` flags other than `'exact'`: unobserved. Choice: rejected with
  an error; the flag is matched case-insensitively.
- `strtok.q-remainder`: remainder delimiter handling unobserved. Choice:
  remainder starts at (and includes) the delimiter that terminated the
  token (FR-006-09 tagged summary-derived).
- `strtok.q-empty`: empty/all-delimiter input unobserved. Choice: empty
  token (0×0 char) and empty remainder.
- `strtok` default whitespace set: only `' '` observed. Choice: Rust
  `char::is_whitespace`.
- `strtok` string inputs: unobserved. Choice: accepted; outputs are string
  scalars (mirrors `fileparts`).

## Assumptions

- RunMat's existing text handling (`strings/search/text_utils.rs`,
  `strings/common.rs`) provides the char/string/cellstr conversions; no new
  runtime types required.
- Multi-output delivery for `strtok` reuses the `output_count` machinery
  used by `fileparts`.
