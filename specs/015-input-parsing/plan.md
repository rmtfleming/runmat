# Implementation Plan — 015-input-parsing

## Constitution Check

- I Specification-first: all normative FRs cite approved claims
  (`inputParser.signature-primary`, `.results-fields`, `.using-defaults`);
  everything else is explicitly unresolved — PASS
- II Clean-room: sources = imported export + RunMat code only; no MATLAB
  invocation anywhere in this feature (including tests) — PASS
- III Independence: design derived from RunMat's own object machinery
  (`containers.Map` precedent); no MATLAB-internal imitation — PASS
- IV Provenance: `provenance.md` complete with the authoritative batch-2
  commit pin `b054f3ad809ceee28529a4dd7a6e28d2b6b434ef` — PASS (Gate-1
  blocker of the interim pin era does not apply)
- V Test-first: tasks T2–T3 (failing tests) precede T4–T6 (bodies) — PASS
- VI Traceability: FR→test mapping enforced by SC-015-3 and T8 — PASS
- VII Minimal scope: one built-in class, four builtins, no refactoring of
  existing machinery — PASS
- VIII Human gates: this plan stops at Gate 2; no implementation before
  Gate 3 approval — PASS
- IX RunMat compatibility: uses `runtime_builtin` macro, existing class
  registry, existing value kinds; no new architecture — PASS
- X Licensing: no external implementation sources used — PASS

## Object-system maturity assessment (MAIN DELIVERABLE)

### VERDICT: (a) IMPLEMENTABLE NOW — no prerequisite feature required.

Every capability a stateful, handle-like `inputParser` needs already exists
in-tree and is exercised by a shipped builtin class, **`containers.Map`**
(`crates/runmat-runtime/src/builtins/containers/map/containers.map.rs`),
which is the concrete exemplar this feature follows. Evidence, capability by
capability:

1. **Object/handle value kinds exist.**
   `crates/runmat-builtins/src/lib.rs`: `ObjectInstance { class_name,
   properties: HashMap<String, Value> }` (line ~2513) and `HandleRef {
   class_name, target: GcHandle, valid }` (line ~2089), surfaced as
   `Value::Object` / `Value::HandleObject`, both GC-traced (`impl Trace`,
   lines ~2143–2155).

2. **A class can register named methods that resolve to builtins.**
   `runmat_builtins::register_class(ClassDef { name, parent, properties,
   methods })` (`crates/runmat-builtins/src/lib.rs:2660`), with `MethodDef
   { name, function_name, access, … }`. Concrete existing example:
   `ensure_containers_map_class_registered()`
   (`containers.map.rs:530–582`) registers methods `keys`, `values`,
   `isKey`, `remove` whose `function_name`s are dotted builtin names
   (`"containers.Map.keys"` etc.), each defined via
   `#[runtime_builtin(name = "containers.Map.keys")]` and siblings
   (`containers.map.rs:889–:990`). Registration is lazy, per-thread
   (`CONTAINERS_MAP_CLASS_REGISTERED` flag), performed inside the
   constructor builtin — inputParser copies this exactly.

3. **Method-call syntax `p.parse(5)` dispatches to registered methods.**
   VM path: `call_method_or_member_index_named_with_outputs`
   (`crates/runmat-vm/src/call/closures.rs:548`) →
   `call_member_index_on_object_like` (`closures.rs:226`) →
   `runmat_builtins::lookup_method(class, name)` → dispatch to
   `MethodDef.function_name` with the receiver prepended
   (`closures.rs:240–261`). The runtime-side equivalents are the
   `call_method` builtin
   (`crates/runmat-runtime/src/builtins/introspection/call_method.rs`,
   `dispatch_call_method` → `dispatch_object_external_member`,
   `crates/runmat-runtime/src/lib.rs:293`) and
   `__runmat_call_bound_method__` (`call_bound_method.rs:117–121`), with
   `getmethod` (`getmethod.rs:105–151`) producing bound closures for
   `Value::Object`/`Value::HandleObject` receivers.

4. **Function-call syntax `parse(p, 5)` also reaches the method.**
   `try_call_registered_instance_method`
   (`crates/runmat-runtime/src/dispatcher.rs:335–406`): first argument's
   class (`Value::HandleObject(handle) => handle.class_name`) →
   `lookup_method` → invokes `method.function_name` builtin. So FR-015-07
   is satisfied by existing machinery.

5. **Property reads `p.Results` / `p.UsingDefaults` work on handles with
   zero new machinery**, provided the class does NOT register a member
   `subsref` override: VM `load_member`
   (`crates/runmat-vm/src/object/resolve.rs:209–231`,
   `Value::HandleObject` arm) falls through to the `getfield` builtin →
   `get_handle_field`
   (`crates/runmat-runtime/src/builtins/structs/core/getfield.rs:948,
   1038–1052`) → `runmat_gc::gc_clone_value(&handle.target)` → reads
   `ObjectInstance.properties`. Nested `p.Results.x` is then an ordinary
   struct field read. (This is why the design below stores `Results` and
   `UsingDefaults` as plain data properties and registers NO
   subsref/subsasgn for `inputParser` — unlike `containers.Map`, which
   needs paren-indexing overloads.)

6. **State mutation through a handle (the crux for `parse`) is proven in
   two in-tree forms.**
   - Write-back through the GC target: `assign_into_handle`
     (`crates/runmat-runtime/src/builtins/structs/core/setfield.rs:1031–
     1080`) mutates the underlying `Value::Object` in place via
     `runmat_gc::gc_with_value_mut(&handle.target, …)`
     (`crates/runmat-gc/src/lib.rs:793`). Because every alias of the
     handle shares the same `GcHandle` target, mutations are visible to
     all aliases — exactly MATLAB handle semantics.
   - Side-registry keyed by an `id` property: `containers.Map`'s
     `MAP_REGISTRY` + `with_store_mut` (`containers.map.rs:1258–1337`).
   **Answer to "does method dispatch allow mutating the underlying
   object?": yes.** Method builtins receive the receiver as
   `Value::HandleObject` (a cheap clone of the `HandleRef`), and mutate
   shared state through `gc_with_value_mut` on `handle.target` — the
   receiver being passed "by value" is irrelevant because the state lives
   behind the GC handle, not in the `HandleRef` copy.
   **Chosen story for inputParser: GC write-back (`gc_with_value_mut`),
   not the side registry.** All parser state (registered names, defaults,
   Results, UsingDefaults) is representable as plain `Value`s, so it can
   live directly in `ObjectInstance.properties`; `containers.Map` only
   needs its registry because `MapStore` holds non-`Value` typed-key
   state. This also gives GC tracing of defaults for free
   (`ObjectInstance::trace`).

7. **`class(p)` returns the class name.** `class.rs:104–110`
   (`crates/runmat-runtime/src/builtins/introspection/class.rs`):
   `Value::HandleObject(handle)` → `handle.class_name` → `'inputParser'`
   as required by FR-015-01. Handle lifecycle helpers exist
   (`is_handle_valid`, `HANDLE_VALID_FLAG_PROPERTY`,
   `crates/runmat-runtime/src/lib.rs:53–81`; `isvalid` builtin).

8. **Constructor plumbing exists.** Either construct directly (allocate
   `ObjectInstance`, `runmat_gc::gc_allocate`, wrap in `HandleRef` — the
   `new_handle_object_builtin` pattern, `crates/runmat-runtime/src/lib.rs:
   496–505`, and `allocate_handle`, `containers.map.rs:1258–1301`), or via
   `create_class_object` (`lib.rs:828`), which auto-wraps in a handle when
   the class chain reaches a `handle` parent. We construct directly in the
   `inputParser` builtin, like `containers.Map` does.

Known machinery caveats (accepted, none blocking):
- The class registry is thread-local; registration must be lazy in the
  constructor (containers.Map's flag pattern) so every thread that
  constructs a parser registers the class. Function-syntax dispatch and
  property access only occur after construction, so ordering is safe.
- `call_member_index_on_object_like` routes zero-argument method calls
  through a member-`subsref` override when one exists
  (`closures.rs:234–238`); since inputParser registers no subsref, `p.parse()`
  (zero-arg, defaults-only — case `param-default`) dispatches normally.
- Internal schema properties stored on the object (see below) are
  technically visible to `fieldnames`/`getfield`; unobserved surface,
  documented as a known cosmetic leak, prefixed to avoid collision.

## Design

New file `crates/runmat-runtime/src/builtins/introspection/input_parser.rs`
(the introspection category already hosts the function-argument/objects
machinery: `inputname.rs`, `arity_check.rs`, `call_method.rs`; a dedicated
category is not warranted for one class). Registered via a `mod` line in
`introspection/mod.rs`. Builtins (all `#[runtime_builtin]`, descriptor-
documented per `docs/builtins/authoring.md`):

| Builtin name | Role |
|---|---|
| `inputParser` | constructor: lazy `register_class` (methods below; no subsref/subsasgn; no `handle` parent needed since we wrap explicitly), allocate `ObjectInstance` with default properties, `gc_allocate`, return `Value::HandleObject` |
| `inputParser.addRequired` | append name to ordered required list (receiver-first args; mutate via `gc_with_value_mut`) |
| `inputParser.addParameter` | append name+default to parameter lists |
| `inputParser.parse` | bind positionals in registration order, then name-value pairs (exact-match names); write `Results` (struct) and `UsingDefaults` (row cell of defaulted parameter names) |

`ClassDef.methods`: `addRequired`, `addParameter`, `parse` →
`function_name`s above (public, non-static). `addOptional` is deliberately
NOT registered (FR-015-08).

Object property layout (all plain `Value`s):
- `Results`: struct (empty at construction)
- `UsingDefaults`: cell 1×0 at construction
- `__ip_required__`: cell 1×N of char names (ordered)
- `__ip_param_names__` / `__ip_param_defaults__`: parallel 1×M cells

Arity/typing rules (unresolved-choice errors, FR-015-08): `addRequired`
takes exactly one name; `addParameter` exactly name+default; a validator
third argument → explicit `RunMat:inputParser:ValidatorsUnsupported` error;
non-char/string names rejected; `parse` errors on missing required values,
surplus positionals, dangling or unknown parameter names.

## Test plan (inline `#[cfg(test)]`, pure in-process, no side effects)

Direct builtin-level calls (containers.Map test style, `containers.map.rs`
tests ~line 2040) plus dispatch-path coverage; no filesystem, no network,
no MATLAB.

- Normative (FR/claim-cited, one per observed case):
  `construct-class` (`class` builtin on constructor result → char 1×11
  'inputParser'); `results-class`; `required-value`; `param-value`;
  `param-default`; `using-defaults` (cell, 1×1).
- Dispatch coverage: same chain through `call_method` builtin
  (method-call path) and through `parse(p, …)` via
  `try_call_registered_instance_method` (function-call path); one tagged
  `dual_syntax` per FR-015-07.
- Statefulness (FR-015-06): registrations then parse on the same handle
  value, no reassignment.
- `unresolved_choice`-tagged: alias mutation visibility (`q = p`),
  UsingDefaults element content ('tol' char), pre-parse property values,
  validator-argument error, unknown-parameter error, missing-required
  error, case-sensitive name matching.

## Risks

- Highest-risk feature of batch 3 (selection proposal F15): first
  clean-room builtin *class*; mitigation = strict adherence to the
  containers.Map exemplar and the narrow observed surface.
- Thread-local class registry: a parser handle crossing threads would not
  find the class registered on the other thread. Same pre-existing
  limitation as containers.Map; not widened by this feature; noted for the
  spec side.
- Ordering guarantees of `StructValue`/field display are HashMap-based;
  Results field order is unobserved, so no requirement is taken on it.
- Registering a plain (undotted) builtin named `inputParser` shadows
  nothing (verified: no existing builtin/function of that name in-tree).

## Rollback

Delete `crates/runmat-runtime/src/builtins/introspection/input_parser.rs`
and its single `mod` line in `introspection/mod.rs`. No existing file is
otherwise modified; no persisted state, no on-disk artefacts.

## Verification commands

`cargo fmt --all -- --check` · `cargo check -p runmat-runtime` ·
`RUST_TEST_THREADS=1 cargo test -p runmat-runtime input_parser`
