# Tasks — 056-table-variables (specs 0.2.0)

Order is normative (test-first). Every task cites requirements. Only work
actually done is ticked.

- [x] **T0** Freshness check of the source repository for the four specs
  (fetch; log/diff against pins `1156c97`, `fc9f804`, HEAD and
  `origin/main`; status; diff against imports). [provenance]
- [x] **T1** Inspect the substrate: table object model and properties,
  F34 selection helpers, `MappedCallable` (cellfun), `call_builtin_async`
  (F50 cast), `crate::warnings::emit`, `crate::output_count`. No code
  changes. [FR-056-01..53]
- [x] **T2** Write the shared support file and the four builtin files with
  registered stubs (descriptors; bodies returning an `Internal` "not yet
  implemented" error) and the observed normative tests. [FR-056-01..33;
  SC-056-1]
- [x] **T3** Write the `rejects_*`, `unresolved_*` and
  `documented_choice_*` tests. [FR-056-40..53]
- [x] **T4** Register the modules, run the four test modules against the
  stubs and record the failing run in `validation.md`.
- [x] **T5** Implement `renamevars`. [FR-056-01..04, 40..44]
- [x] **T6** Implement `rows2vars`. [FR-056-10..17, 40, 41, 45..48]
- [x] **T7** Implement `varfun`. [FR-056-20..23, 40, 41, 49..51]
- [x] **T8** Implement `convertvars`. [FR-056-30..33, 40..42, 52, 53]
- [x] **T9** Run feature tests, the `builtins::table` suite, rustfmt and
  clippy; record output in `validation.md`. [SC-056-1, SC-056-2]
- [x] **T10** Traceability table (claim → FR → test → result), documented
  choices and deviations in `validation.md`; final freshness check.
  [SC-056-3; Principle VI]
