# Tasks — 034-table-variable-editing

Order is normative (test-first). Every task cites requirements.

## Batch C1 (0.2.0) — done, committed in `69a30fe2c`

- [x] T1–T8: confirmed forms only (D1 = A); see git history for the
  0.2.0-era task list.

## Widening W1 (0.3.0)

- [x] **W1** Freshness check of the three specs against source
  `5d595da` (HEAD, origin/main, working tree); read the 0.3.0 interface,
  behaviour, provenance and observations; verify every claim id cited in
  `spec.md`/`provenance.md`. [Principle IV]
- [x] **W2** Investigate caller variable names (`inputname`,
  `crate::callsite`) for addvars.default-name-input. Result: the VM records
  a builtin callsite with argument spans, so the name is reachable. [FR-034-04, FR-034-14]
- [x] **W3** Investigate whether the table subsystem can hold an N×0
  table (removevars.remove-all). Result: it cannot (height derives from the
  first variable), so record deviation D-034-1. [FR-034-10]
- [x] **W4** Rewrite the test module: observed tests for every 0.3.0 claim
  group, unresolved, documented-choice and deviation tests. Run them against
  the 0.2.0-era code and record the failures in `validation.md`.
  [FR-034-01..16]
- [x] **W5** Implement: error kinds/descriptors (FR-034-11), selections,
  options and placement, the widened addvars/removevars/movevars, and the
  updated signatures. [FR-034-01..16]
- [x] **W6** Run the verification commands; update `validation.md`
  (traceability, counts, clippy, table suite, freshness re-check).
  [SC-034-1..3]
