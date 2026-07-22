# Feature Specification: Input Parsing — `inputParser`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id: `015-input-parsing`; export `SPECIFY_FEATURE=015-input-parsing`)

**Created**: 2026-07-22

**Status**: Batch 3 (F15) — B1 requirements authored; combined review pending

**Input**: Approved Tier A export `inputParser` 0.2.0 (see `provenance.md`;
commit pin `b054f3ad…34ef` via `dist/runmat-export-2026-07-22`, batch 2).

**Selection context**: F15 in
`specs/001-clean-room-matlab-builtins/selection-proposal-batch3.md` —
"Large / highest risk. Stateful handle-like object with methods (addRequired,
addParameter, parse) and properties (Results struct, UsingDefaults cell).
Object-system maturity must be assessed in the plan; may surface a
prerequisite feature." The maturity assessment and its verdict live in
`plan.md`.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Construct a parser object (Priority: P1)

A RunMat user writes `p = inputParser` and receives a parser object whose
class is `inputParser`, ready to have argument definitions registered on it.

**Independent Test**: construct a parser, call `class` on it, compare value,
class and shape against the observed case.

**Acceptance Scenarios** (observed, normative):

1. **Given** no prior state, **When** `class(inputParser)` is evaluated,
   **Then** the result is char, 1×11, `'inputParser'`.
   [inputParser.signature-primary, inputParser.results-fields; case
   `construct-class`]

### User Story 2 - Parse a required positional argument (Priority: P1)

The user registers a required argument named `x`, parses the input `5`, and
reads the parsed value back from the `Results` property.

**Acceptance Scenarios** (observed, normative):

1. **Given** a parser with `addRequired` called for `'x'`, **When** `parse`
   is called with input `5`, **Then** `class(p.Results)` is `'struct'`
   (char, 1×6). [inputParser.results-fields; case `results-class`]
2. **Given** the same sequence, **Then** `p.Results.x` is double, 1×1,
   value `5`. [inputParser.results-fields; case `required-value`]

### User Story 3 - Parse a name-value parameter with a default (Priority: P1)

The user registers a parameter `tol` with default `0.1`; supplying
`'tol', 0.2` at parse time overrides the default, omitting it keeps the
default, and `UsingDefaults` reports what was defaulted.

**Acceptance Scenarios** (observed, normative):

1. **Given** a parser with `addParameter('tol', 0.1)`, **When** `parse` is
   called with `'tol', 0.2`, **Then** `p.Results.tol` is double, 1×1, value
   `0.2` (i.e. the double nearest 0.2; observation renders it
   `0.20000000000000001`). [inputParser.results-fields; case `param-value`]
2. **Given** the same registration, **When** `parse` is called with no
   inputs, **Then** `p.Results.tol` is double, 1×1, value `0.1`.
   [inputParser.results-fields; case `param-default`]
3. **Given** the same no-input parse, **Then** `p.UsingDefaults` is a cell
   array of size 1×1 listing the parameter left at its default.
   [inputParser.using-defaults; cases `using-defaults`, `param-default`]

## Requirements *(mandatory)*

### Functional Requirements

- **FR-015-01** `p = inputParser` MUST be accepted with no inputs and
  produce a parser object; `class(p)` MUST return char 1×11 `'inputParser'`.
  [inputParser.signature-primary, inputParser.results-fields]
- **FR-015-02** The parser MUST support method `addRequired(p, name)`
  registering a required positional argument, and method `parse(p, value)`
  binding it, such that afterwards `p.Results` is a struct
  (`class` → `'struct'`) with field `name` holding the parsed value
  (observed: `x` → double 1×1 `5`). [inputParser.results-fields]
- **FR-015-03** The parser MUST support method
  `addParameter(p, name, default)` registering a name-value parameter, such
  that `parse(p, name, value)` yields `p.Results.(name)` equal to the
  supplied value (observed: `tol`, default `0.1`, supplied `0.2` → double
  1×1 `0.2`). [inputParser.results-fields]
- **FR-015-04** When a registered parameter is not supplied to `parse`,
  `p.Results.(name)` MUST hold the registered default (observed: double 1×1
  `0.1`). [inputParser.results-fields]
- **FR-015-05** `p.UsingDefaults` MUST be a cell array listing the
  parameters left at their default value; with exactly one such parameter it
  MUST be 1×1. [inputParser.using-defaults]
- **FR-015-06** Registrations made by `addRequired`/`addParameter` MUST be
  visible to a subsequent `parse` on the same parser variable, and parse
  results MUST be visible to subsequent property reads, without any
  reassignment of the variable — this statefulness is required to reproduce
  the observed call chains. [inputParser.results-fields,
  inputParser.using-defaults — the cases are multi-step chains on one
  parser]
- **FR-015-07** Methods MUST be invocable in method-call syntax
  (`p.parse(5)`) and function-call syntax (`parse(p, 5)`). *(The export does
  not record which syntax the experiment used; supporting both is RunMat's
  own generic member-dispatch semantics, not asserted as a MATLAB-conformance
  claim. Tests for whichever form the normative comparison uses are
  normative; the other form is tagged `dual_syntax`.)*
- **FR-015-08** Behaviour with no approved source MUST NOT be invented:
  every input form outside the observed chains MUST either raise an explicit
  RunMat error or follow a documented independent choice listed under
  Unresolved behaviour, and MUST NOT be asserted as MATLAB-conformant. This
  covers at minimum: validation functions (third argument to
  `addRequired`/`addParameter`), `addOptional`, `KeepUnmatched`,
  `StructExpand`, `CaseSensitive`, the `Unmatched` property, duplicate or
  unknown parameter names, and all error/warning conditions.
  [carried unresolved: inputParser.q-class-semantics, inputParser.q-methods]

### Key Entities

- Parser object: `Value::HandleObject(HandleRef)` whose GC target is a
  `Value::Object(ObjectInstance)` with class name `inputParser` (see
  plan.md for the grounded machinery assessment).
- `Results`: `Value::Struct` exposed as an ordinary property read.
- `UsingDefaults`: `Value::Cell`, row-shaped.

## Success Criteria *(mandatory)*

- **SC-015-1**: Every observed case in the export reproduces exactly (value,
  class, size) in normative tests: `construct-class`, `results-class`,
  `required-value`, `param-value`, `param-default`, `using-defaults`.
- **SC-015-2**: Constructor and method builtins registered and discoverable
  (`builtin_function_by_name` for `inputParser`, `inputParser.addRequired`,
  `inputParser.addParameter`, `inputParser.parse`); the `inputParser` class
  is registered with those methods (`runmat_builtins::get_class`); `cargo
  test -p runmat-runtime input_parser` passes.
- **SC-015-3**: Traceability: each FR cites ≥1 claim id (or is explicitly
  tagged as unresolved/independent choice); each normative test cites ≥1 FR.
- **SC-015-4**: No test invokes MATLAB, touches the network, or writes
  outside the test process (pure in-process; Constitution II, and the F15
  sandbox note: "No external side effects; risk is architectural").

## Unresolved behaviour (non-normative; implemented by documented choice)

- **Handle vs value semantics beyond the observed chains**
  (`inputParser.q-class-semantics`): the interface text "(handle)" is
  `status: pending`. Choice: implement as a RunMat handle object
  (`Value::HandleObject`), because (a) it is the only design under which the
  observed multi-step chains work without reassignment, and (b) it follows
  the in-tree `containers.Map` precedent. Aliasing behaviour (`q = p`
  observing `p`'s later mutations) follows from this choice and is tagged
  `unresolved_choice` in tests, never asserted as MATLAB-conformant.
- **`UsingDefaults` element content/type**: the observation records only
  class `cell` and size 1×1; behaviour.md's approved wording says defaulted
  parameters "are listed". Choice: each element is the registered parameter
  name as a char row vector. Element-type conformance is tagged
  `unresolved_choice`.
- **Pre-parse property values**: `Results`/`UsingDefaults` before any
  `parse` call are unobserved. Choice: `Results` initialises to an empty
  1×1 struct with no fields; `UsingDefaults` to a 1×0 cell.
- **`parse` with wrong arity / unknown parameter name / missing required
  value**: unobserved. Choice: raise RunMat-identified errors
  (`RunMat:inputParser:...`); messages are RunMat's own wording.
- **Validation functions, `addOptional`, `KeepUnmatched`, `StructExpand`,
  `CaseSensitive`, `Unmatched`**: not implemented. A third argument to
  `addRequired`/`addParameter` raises an explicit "not supported" RunMat
  error rather than being silently ignored; `addOptional` and the
  configuration properties are absent (undefined method/field errors).
  Parameter-name matching is exact (case-sensitive) by documented choice.
- **`p.parse` without parentheses invoking the method**: unobserved and out
  of scope (RunMat member-load semantics apply).

## Assumptions

- RunMat's existing object machinery suffices — grounded in plan.md's
  maturity assessment (verdict: implementable now, `containers.Map`
  exemplar). No parser/HIR/VM changes are anticipated.
- Observed doubles `0.20000000000000001` / `0.10000000000000001` are the
  17-digit renderings of the IEEE-754 doubles nearest 0.2 / 0.1; tests
  compare against the literals `0.2` / `0.1`.
