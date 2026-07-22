# Feature Specification: `accumarray` — subscript accumulation

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`030-accumarray`; export `SPECIFY_FEATURE=030-accumarray`)

**Created**: 2026-07-22

**Status**: Batch 5 (Tier B batch 3) — review complete, implemented

**Input**: Approved Tier B export `accumarray` 0.2.0 (see `provenance.md`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Accumulate values by subscript (Priority: P1)

RunMat users calling `A = accumarray(subs, vals)` get a double column vector
in which every value is summed into the position given by its subscript,
matching the approved observed behaviour.

**Independent Test**: run the single observed case from the export
(`accumarray([1;2;1],[10;20;30])`) through the builtin and compare value,
class and shape against `[40;20]`, `double`, `[2,1]`.

**Acceptance Scenarios** (from observed results, normative):

1. **Given** subscripts `[1;2;1]` and values `[10;20;30]`, **When**
   `accumarray` is called with the two-argument form, **Then** the result is a
   `double` column vector `[40;20]` of size `[2,1]`: index 1 receives
   `10 + 30 = 40`, index 2 receives `20`.
   [accumarray.sum-by-subscript, accumarray.output-class,
   accumarray.signature-primary]

### User Story 2 - Shape and fill the output explicitly (Priority: P2)

Users may pass an output size `sz` and/or a `fillval` to control the length of
`A` and the value placed in positions that receive no subscript. These forms
are unobserved in the approved export and are provided as documented
independent choices, not asserted as MATLAB-conformant.

**Acceptance Scenarios** (documented choice, non-normative):

1. **Given** `accumarray([1;2;1],[10;20;30],[4 1])`, **When** called, **Then**
   the result is `[40;20;0;0]` (size `[4,1]`), the trailing positions taking
   the default fill `0`. *(unresolved: accumarray.q-size)*
2. **Given** `accumarray([1;3],[5;7],[4 1],[],-1)`, **When** called, **Then**
   the result is `[5;-1;7;-1]`; `-1` fills only positions with no subscript.
   *(unresolved: accumarray.q-size)*

## Requirements *(mandatory)*

### Functional Requirements

- **FR-030-01** `accumarray` MUST accept the two-argument form
  `A = accumarray(subs, vals)` [accumarray.signature-primary] and return a
  `double` [accumarray.output-class] column vector of length `max(subs)`,
  summing all `vals` entries that share a subscript, with unindexed positions
  filled with `0` [accumarray.sum-by-subscript]. The observed case
  `accumarray([1;2;1],[10;20;30]) => [40;20]` MUST reproduce exactly.
- **FR-030-02** Subscripts MUST be validated as real positive integers; any
  non-finite, non-integer, or `< 1` subscript MUST raise a stable error
  (`RunMat:accumarray:InvalidSubscripts`). (Public-interface fact / documented
  independent choice; not directly observed.)
- **FR-030-03** `vals` MUST either be a scalar (broadcast to every subscript)
  or match the number of subscripts; a mismatched length MUST raise
  `RunMat:accumarray:SizeMismatch`. (Documented independent choice; scalar
  broadcast is unobserved.)
- **FR-030-04** (documented choice, unobserved — accumarray.q-size) The
  optional `sz` argument MUST set the output length for the single-subscript
  `[m 1]` column form; empty `[]` selects `max(subs)`; an `sz` smaller than
  `max(subs)`, or one describing more than one output column, MUST raise
  `RunMat:accumarray:InvalidSize`.
- **FR-030-05** (documented choice, unobserved — accumarray.q-size) The
  optional `fillval` argument MUST replace the value of positions that receive
  no subscript (default `0`), without altering accumulated positions.
- **FR-030-06** (unresolved — accumarray.q-func) A custom accumulation
  function (function handle in the `fun` argument) MUST be rejected clearly
  (`RunMat:accumarray:CustomFunctionUnsupported`); only the default sum
  (`[]`) is accepted.
- **FR-030-07** (unresolved — accumarray.q-size) The multi-column
  (N-D output) subscript form MUST be rejected clearly
  (`RunMat:accumarray:MultiColumnUnsupported`), and the sparse-output
  (`issparse`) option MUST be rejected clearly
  (`RunMat:accumarray:SparseOutputUnsupported`).
- **FR-030-08** GPU-resident inputs MUST be gathered to the host before
  accumulation (host-only scatter); logical and integer inputs are coerced to
  double like RunMat's other array constructors.

### Key Entities

- Result: `Value::Tensor` of shape `[m,1]` (double), collapsing to
  `Value::Num` when `m == 1` per RunMat's 1×1 convention.
- Accumulation: host scatter-add over a `Vec<f64>` accumulator with a parallel
  `touched` mask so `fillval` applies only to empty positions.

## Success Criteria *(mandatory)*

- **SC-030-1**: The single observed case reproduces exactly (value `[40;20]`,
  class `double`, size `[2,1]`) in a normative test.
- **SC-030-2**: `accumarray` is registered and discoverable via the
  `#[runtime_builtin]` inventory macro; `cargo test -p runmat-runtime --lib --
  accumarray` passes.
- **SC-030-3**: Traceability: each FR cites ≥1 claim id or is explicitly
  tagged documented-choice/unresolved; each observed test cites ≥1 claim.

## Unresolved behaviour (non-normative; carried or implemented by choice)

- `accumarray.q-func`: the custom accumulation-function form. Choice: reject
  non-empty `fun`; only default sum supported. Carried to the specification
  side.
- `accumarray.q-size`: the size and fill-value options. Choice: implement
  `sz`/`fillval` for the single-subscript `[m 1]` column form as documented
  independent choices; reject multi-column (N-D) subscripts and the
  `issparse`/sparse-output option. Carried to the specification side.
- Empty `subs`: yields an empty `[0,1]` double (documented choice; unobserved).

## Assumptions

- The approved export confirms only the two-argument column sum form (one
  observation case `sum`). Everything beyond it is unobserved.
- RunMat's `Value`/`Tensor` model already supports column vectors and GPU
  gather; no new runtime types are required.
