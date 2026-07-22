# Feature Specification: Table Conversion — `array2table`, `cell2table`, `table2array`, `table2cell`

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id: `010-table-conversion`)

**Created**: 2026-07-22

**Status**: **BLOCKED — architectural prerequisite missing (B-010-1)**

**Input**: Approved Tier A exports `array2table`, `cell2table`,
`table2array`, `table2cell` (all 0.2.0; see `provenance.md`).

## Blocker B-010-1: RunMat has no table data type

The observed behaviour of all four functions produces or consumes values of
class `table` (e.g. `array2table([1 2 3; 4 5 6])` ⇒ `<table [2 3]>`).
RunMat's runtime value model (`runmat_builtins::Value`) has **no table
variant**, and `crates/runmat-runtime/src/builtins/table/` is an empty stub
(only `mod.rs`). Implementing these converters therefore requires first
introducing a table type — value representation, display, indexing,
variable-name metadata — which is a separate architectural feature well
beyond this feature's minimal scope (Principle VII) and must not be smuggled
in as a side effect (Principle IX).

Per constitution Principle VIII, this is surfaced rather than silently
resolved. The feature is bootstrapped, provenance is recorded, and
implementation is blocked.

## Recommended path

1. A dedicated RunMat feature (outside this Tier A batch) designs the table
   value type against RunMat's architecture — driven by RunMat's own
   conventions, not by imitating MATLAB internals.
2. Once a table type exists, this feature returns to planning (B2) with the
   four converters as thin construct/extract adapters over it.
3. The specification side could meanwhile deepen the exports (currently 1–2
   observation cases per function; per-variable class behaviour, VariableNames
   defaults and dimension rules are largely unobserved).

## Requirements (recorded, not yet implementable)

- FR-010-01..04: one requirement per converter mirroring its observed cases
  (shapes: array2table [2 3]→2x3 table; cell2table {2x2}→2x2 table;
  table2array round-trip → original numeric matrix; table2cell round-trip →
  original cell), each citing the claims in the respective provenance.yaml.
- 'VariableNames' name-value pairs observed for array2table/cell2table MUST
  be supported when implemented.

## Unresolved behaviour

Everything beyond the six observed cases, including default variable names,
heterogeneous column typing rules, and round-trip type preservation.
