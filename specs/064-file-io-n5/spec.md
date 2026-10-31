# Feature Specification: file I/O — `fscanf`, `urlwrite`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`064-file-io-n5`; export `SPECIFY_FEATURE=064-file-io-n5`)

**Created**: 2026-10-31

**Status**: Batch N5, feature F64. Gates G1–G3 pre-approved by the
maintainer (`specs/001-clean-room-matlab-builtins/selection-proposal-n5.md`).

**Input**: Approved exports `fscanf` **0.2.0** and `urlwrite` **0.2.0**
(N5, source `1c91aa1`); see `provenance.md`.

Family rationale (Principle VII): two COBRA-backlog file I/O builtins.
`fscanf` reads formatted data from a file opened with `fopen`; `urlwrite`
saves the content at a URL to a file. They share no state. Each has its own
file, tests and requirement block.

## Decisions

- **D-064-1 Error identifier namespace.** Observed `MATLAB:x:y`
  identifiers are raised as `RunMat:x:y` (maintainer rule):
  `RunMat:FileIO:InvalidFid`, `RunMat:minrhs`,
  `RunMat:urlwrite:FileNotFound`, `RunMat:urlwrite:InvalidOutputLocation`,
  `RunMat:urlwrite:InvalidUrl`. Identifiers that were not observed use
  `RunMat:<fn>:<name>`.
- **D-064-2 `fscanf` reuses the `sscanf` scanner.** The conversion parser
  and the single-conversion scanner of `sscanf`
  (`strings/core/compat.rs`: `parse_scan_format`, `scan_one`,
  `skip_whitespace`, and the types `ScanToken`, `ScanKind`, `IntegerRadix`,
  `ScanValue`) were made `pub(crate)`; their bodies are unchanged.
  `sscanf_scan` itself is not reused: it counts values rather than
  conversions, always returns double, and does not implement the `[m Inf]`
  shape, all of which `fscanf` needs (`s-skip`, `mixed-d-s`, `c-all`,
  `size-matrix`). `fscanf.rs` has its own read loop and result shaping.
  The file handle comes from the `fopen` registry
  (`io/filetext/registry.rs`), as for `fgetl`/`fread`.
- **D-064-3 `fscanf.q-mixed`** (maintainer: "character-code result
  reproduced as observed"). Mixed numeric and `%s` conversions return a
  double column holding the character codes (FR-064-07).
- **D-064-4 `urlwrite.q-network`** (maintainer: "Use webread transport").
  http/https URLs are fetched with the shared transport
  (`io/http/transport.rs`), as `urlread` and `webread` do. This is a
  **labelled documented choice, not observed behaviour** (FR-064-24). Its
  tests use a loopback listener on 127.0.0.1 and never contact the network.
- **D-064-5 Unobserved points** raise `RunMat:<fn>:UnsupportedOption` or
  are labelled documented choices below. Neither kind is presented as
  MATLAB behaviour.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Read a text file with a format (Priority: P1)

**Acceptance Scenarios** (normative, one per observed case; file contents
in brackets):

1. `[A,c] = fscanf(fid,'%c')` on `['ab c' newline 'de']` → 1×7 char equal
   to the text, `c = 7`; on `['xy' newline]` → 1×3 char, `c = 3` (`c-all`,
   `c-trailing-newline`). [fscanf.signature-primary, fscanf.c]
2. `'%s'` on the `c-all` text → `'abcde'` (1×5), `c = 3` (`s-skip`).
   [fscanf.s]
3. `'%d'` on `'1 2 3'` → `[1;2;3]`; `'%f'` on `['1.5 -2' newline '3e2']` →
   `[1.5;-2;300]`; `'%x'` on `'1f a'` → `[31;10]`, `c = 2` (`d-ints`,
   `f-reals`, `hex`). [fscanf.numeric]
4. `fscanf(fid,'%f',[2 Inf])` on `'1 2 3 4 5 6'` → `[1 3 5;2 4 6]`,
   `c = 6`; `fscanf(fid,'%d',2)` on `'1 2 3'` → `[1;2]`, `c = 2`
   (`size-matrix`, `size-scalar`). [fscanf.size]
5. `'%d,'` on `'1,2,3'` → `[1;2;3]`; `'%d'` on `'4 5 x 6'` → `[4;5]`,
   `c = 2`; on `'abc'` → 0×0 double, `c = 0` (`literal`, `partial`,
   `mismatch`). [fscanf.literal-stop]
6. Empty file: `'%c'` → 0×0 char, `'%d'` → 0×0 double, `c = 0`
   (`empty-c`, `empty-d`). [fscanf.empty]
7. `'%d%s'` on `'7 ab'` → `[7;97;98]` (double), `c = 2` (`mixed-d-s`).
   [fscanf.mixed]
8. `fscanf(-1,'%d')` raises `RunMat:FileIO:InvalidFid` (`err-bad-fid`).
   [fscanf.error-bad-fid]
9. `fscanf(fid)` raises `RunMat:minrhs` (`err-no-format`).
   [fscanf.error-no-format]

### User Story 2 - Save a URL to a file (Priority: P1)

**Acceptance Scenarios** (normative; `src` holds `'abc 123'`):

1. `f = urlwrite(['file://' src], dst)` writes `dst` with `'abc 123'` and
   returns `f` equal to `dst` (char), no warning; with two outputs
   `status = 1` (double) (`file-url`, `file-url-status`).
   [urlwrite.signature-primary, urlwrite.success]
2. `[f,status] = urlwrite(['file://' missing], dst)` and
   `[f,status] = urlwrite('nosuchprotocol://x', dst)` do not error: `f` is
   empty char, `status = 0`, `dst` is not created (`missing-two-out`,
   `protocol-two-out`). [urlwrite.status-failure]
3. With one output, a missing file:// source raises
   `RunMat:urlwrite:FileNotFound` (`err-missing-one-out`).
   [urlwrite.error-missing-one-out]
4. A destination in a folder that does not exist raises
   `RunMat:urlwrite:InvalidOutputLocation` with one or two outputs
   (`err-bad-dir`, `err-bad-dir-two-out`). [urlwrite.error-bad-dir]
5. With one output, `urlwrite('nosuchprotocol://x', dst)` raises
   `RunMat:urlwrite:InvalidUrl` (`err-protocol`). [urlwrite.error-protocol]
6. `urlwrite('file:///tmp')` raises `RunMat:minrhs` (`err-one-arg`).
   [urlwrite.error-one-arg]

## Requirements *(mandatory)*

### Functional Requirements — `fscanf` (normative)

- **FR-064-01** `A = fscanf(fid, format)` MUST be accepted. `'%c'` returns
  every character, including spaces and newlines, as a 1×n char row, and
  `count` is n [fscanf.signature-primary, fscanf.c].
- **FR-064-02** `'%s'` skips whitespace and concatenates the words into one
  char row; `count` is the number of words [fscanf.s].
- **FR-064-03** `%d`, `%f` and `%x` return a double column; whitespace
  (including line breaks) between values is skipped [fscanf.numeric].
- **FR-064-04** A size `[m Inf]` fills an m-row matrix column by column; a
  scalar size n reads at most n values [fscanf.size].
- **FR-064-05** Literal format text is matched; reading stops at the first
  failed conversion or literal, keeping the values read so far; no value
  gives a 0×0 double with `count` 0 [fscanf.literal-stop].
- **FR-064-06** An empty file gives 0×0 char for `'%c'` and 0×0 double for
  `'%d'`, `count` 0 [fscanf.empty].
- **FR-064-07** Mixed numeric and `%s` conversions return a double column
  with the character codes of the text; `count` counts conversions, not
  values [fscanf.mixed] (D-064-3).
- **FR-064-08** An identifier that is not an open file MUST raise
  `RunMat:FileIO:InvalidFid` [fscanf.error-bad-fid].
- **FR-064-09** A call without a format MUST raise `RunMat:minrhs`
  [fscanf.error-no-format].
- **FR-064-10** `[A, count] = fscanf(...)` returns `count` as a double
  scalar [fscanf.signature-primary; all count-bearing cases].

### Functional Requirements — `urlwrite` (normative)

- **FR-064-11** `f = urlwrite(url, filename)` and `[f, status] = ...` MUST
  be accepted. For a readable `file://` URL the destination receives the
  same bytes, `f` is the char destination path as given, `status` is double
  1, and no warning is issued [urlwrite.signature-primary,
  urlwrite.success].
- **FR-064-12** With two outputs, a missing `file://` source or an unknown
  protocol does not error: `f` is empty char, `status` is 0 and no
  destination file is created [urlwrite.status-failure].
- **FR-064-13** With one output, a missing `file://` source MUST raise
  `RunMat:urlwrite:FileNotFound` [urlwrite.error-missing-one-out].
- **FR-064-14** A destination folder that does not exist MUST raise
  `RunMat:urlwrite:InvalidOutputLocation` with one or two outputs
  [urlwrite.error-bad-dir].
- **FR-064-15** With one output, an unknown protocol MUST raise
  `RunMat:urlwrite:InvalidUrl` [urlwrite.error-protocol].
- **FR-064-16** A call with only the URL MUST raise `RunMat:minrhs`
  [urlwrite.error-one-arg].

### Functional Requirements (documented choices and unresolved points)

- **FR-064-20** (documented choice, `fscanf`) Conversions: besides the
  observed `%c %s %d %f %x`, the shared scanner's `%e %g %i %u %o`, field
  widths and `%*` (suppressed, not counted) are accepted. `%l…` (64-bit
  integer output), conversions the scanner does not know (e.g. `%q`,
  `%[…]`), an empty format and a format without conversions raise
  `RunMat:fscanf:UnsupportedOption`. The result is char when every
  non-suppressed conversion is `%c` or `%s`, otherwise double. A `%Nc`
  conversion counts once.
- **FR-064-21** (documented choice, `fscanf`) Size: `Inf` means no limit;
  `[m Inf]` with a count that is not a multiple of m is padded with zeros
  to m×ceil(k/m); `[m n]` reads at most m·n values and is shaped the same
  way; a size with a char result shapes the char array the same way (a
  scalar n gives a 1×k row). A size that is not a nonnegative integer
  scalar, `Inf`, or a 2-element `[m n]` (n may be `Inf`, m ≥ 1 then) raises
  `RunMat:fscanf:InvalidInput`.
- **FR-064-22** (documented choice, `fscanf`, `fscanf.q-encoding`) The text
  is the rest of the file from the current position, decoded with the
  encoding given to `fopen` (shared `decode_bytes`). After the call the
  file position is just after the last consumed character (end of file if
  the text was consumed or the offset cannot be mapped back to bytes), and
  the end-of-file flag used by `feof` is set when the end was reached.
- **FR-064-23** (documented choice, `fscanf`) Arguments: fewer than two
  inputs raise `RunMat:minrhs`; more than three raise
  `UnsupportedOption`; more than two outputs raise
  `RunMat:fscanf:TooManyOutputs`. The identifiers 0, 1, 2 (standard
  streams) raise `UnsupportedOption`. A non-numeric, non-integer or
  negative identifier, an identifier not in the `fopen` registry and a file
  not opened for reading raise `RunMat:FileIO:InvalidFid`. A format that is
  not a char row or string scalar raises `RunMat:fscanf:InvalidInput`.
  Read/decode failures raise `RunMat:fscanf:ReadFailed`.
- **FR-064-24** (documented choice, `urlwrite`, `urlwrite.q-network`,
  D-064-4 — **not observed behaviour**) `http://` and `https://` URLs are
  fetched with a GET through the shared transport (60 s timeout) and the
  body bytes are written unchanged. A transport failure (connect, timeout,
  non-2xx status) gives `f = ''`, `status = 0` with two outputs, and raises
  `RunMat:urlwrite:ConnectionFailed` with one output.
- **FR-064-25** (documented choice, `urlwrite`, `urlwrite.q-options`) More
  than two inputs raise `RunMat:urlwrite:UnsupportedOption`; more than two
  outputs raise `RunMat:urlwrite:TooManyOutputs`; a URL or file name that
  is not a char row or string scalar, or an empty file name, raises
  `RunMat:urlwrite:InvalidInput`. URL text that cannot be parsed is
  treated like an unknown protocol. The destination folder is checked
  before the source is read. A destination that cannot be written (e.g. a
  folder) raises `InvalidOutputLocation` with one or two outputs. A
  `file://` URL that cannot be mapped to a path is treated as a missing
  file. A relative file name is resolved against the current folder and
  returned as given.
- **FR-064-26** (unresolved, `*.q-warning-coverage`) Neither function
  emits a warning.

## Success Criteria *(mandatory)*

- **SC-064-1**: all observed cases reproduce: 16 for `fscanf`, 9 for
  `urlwrite`.
- **SC-064-2**: both builtins are registered through `#[runtime_builtin]`,
  and `cargo test -p runmat-runtime --lib -- fscanf_tests urlwrite_tests`
  passes without network access.
- **SC-064-3**: every FR maps to at least one test (`validation.md`).

## Unresolved behaviour (returned to the specification side)

| id | disposition |
|---|---|
| fscanf.q-warning-coverage, urlwrite.q-warning-coverage | no warning emitted (FR-064-26) |
| fscanf.q-mixed | reproduced as observed (maintainer; FR-064-07, D-064-3) |
| fscanf.q-encoding | fopen encoding used; labelled choice FR-064-22 |
| urlwrite.q-network | shared transport, labelled choice FR-064-24 (maintainer, D-064-4) |
| urlwrite.q-options | `UnsupportedOption` (FR-064-25) |
