# Tasks — 015-input-parsing

Order is normative (test-first, Constitution V). Every task cites
requirements. Verdict from plan.md: implementable now on existing object
machinery (containers.Map exemplar) — this is the implementable-path task
list; no prerequisite feature is raised.

- [x] **T1** Re-verify the exemplar path before writing code (no code
  changes): `containers.Map` class registration
  (`containers.map.rs:530–582`), handle allocation (`:1258–1301`), method
  dispatch (`runmat-vm/src/call/closures.rs:226–261`,
  `dispatcher.rs:335–406`), handle mutation
  (`setfield.rs:1031–1080` / `gc_with_value_mut`), and property reads via
  `get_handle_field` (`getfield.rs:1038–1052`). Confirm no `inputParser`
  name collision (`grep` crates — none as of planning). [FR-015-01..07]
- [x] **T2** Author failing normative tests in the new file's
  `#[cfg(test)]` module, one per observed case, each citing FR + claim id:
  `construct-class` (char 1×11 'inputParser'), `results-class` (struct),
  `required-value` (double 1×1, 5), `param-value` (0.2), `param-default`
  (0.1), `using-defaults` (cell 1×1). Include the statefulness chain (no
  reassignment) and both dispatch syntaxes (`call_method` path; direct
  `inputParser.parse` builtin call receiver-first), extra syntax tagged
  `dual_syntax`. (Red step: builtin bodies stubbed/absent; compile gated
  by T4–T6.) [FR-015-01..07; SC-015-1]
- [x] **T3** Author failing `unresolved_choice`-tagged tests: validator
  third argument → `RunMat:inputParser:ValidatorsUnsupported`; unknown
  parameter name at parse → error; missing required value → error;
  case-sensitive name matching; pre-parse `Results` (empty struct) and
  `UsingDefaults` (1×0 cell); alias mutation visibility (`q = p`);
  UsingDefaults element content 'tol' char row. [FR-015-05, FR-015-06,
  FR-015-08]
- [x] **T4** Implement the `inputParser` constructor builtin in
  `crates/runmat-runtime/src/builtins/introspection/input_parser.rs` +
  `mod` entry in `introspection/mod.rs`: lazy per-thread
  `register_class` (methods addRequired/addParameter/parse; NO
  subsref/subsasgn; `addOptional` not registered), object property
  defaults (`Results` empty struct, `UsingDefaults` 1×0 cell, internal
  `__ip_*` schema cells), `gc_allocate` + `HandleRef` return, descriptor
  metadata per `docs/builtins/authoring.md`. [FR-015-01, FR-015-08]
- [x] **T5** Implement `inputParser.addRequired` and
  `inputParser.addParameter` builtins: receiver validation
  (HandleObject of class `inputParser`, valid), name extraction
  (char/string scalar), arity rules incl. validator rejection, schema
  append via `gc_with_value_mut`. [FR-015-02, FR-015-03, FR-015-06,
  FR-015-08]
- [x] **T6** Implement `inputParser.parse`: bind positionals in
  registration order, then exact-match name-value pairs; error rules per
  spec Unresolved choices; write `Results` struct and `UsingDefaults` row
  cell of defaulted parameter names via `gc_with_value_mut`.
  [FR-015-02..06, FR-015-08]
- [x] **T7** Run verification commands (`cargo fmt --all -- --check`,
  `cargo check -p runmat-runtime`, `RUST_TEST_THREADS=1 cargo test -p
  runmat-runtime input_parser`); record real output in `validation.md`
  (no fabricated results — Constitution V). [SC-015-1, SC-015-2, SC-015-4]
- [x] **T8** Traceability table in `validation.md` (claim → FR → test →
  result), including the unresolved-question register
  (`inputParser.q-class-semantics`, `inputParser.q-methods`) and the list
  of `unresolved_choice`/`dual_syntax`-tagged tests. [SC-015-3;
  Principle VI]
