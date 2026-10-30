# Feature Specification: `addnode`, `categories`, `textread` — data builtins (N2)

**Feature Branch**: `feature/clean-room-matlab-builtins` (SpecKit id:
`052-data-n2`; export `SPECIFY_FEATURE=052-data-n2`)

**Created**: 2026-10-30

**Status**: Batch N2 (F52). Gates G1–G3 pre-approved by the maintainer
(`specs/001-clean-room-matlab-builtins/selection-proposal-n2.md`).

**Input**: Approved exports `addnode`, `categories`, `textread` 0.2.0 (see
`provenance.md`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Add nodes to a graph (Priority: P1)

`H = addnode(G, nodeCount)` and `H = addnode(G, nodeNames)` return a new
graph or digraph with nodes appended.

**Acceptance Scenarios** (normative):

1. `addnode(graph([1 2], [2 3]), 2)` has 5 nodes and the edges `[1 2; 2 3]`;
   with 0 it has 3 nodes and the same edges. [addnode.signature-primary,
   addnode.count]
2. `addnode(digraph(1, 2), 1)` is a `digraph` with 3 nodes and edge `[1 2]`;
   the count form gives a `graph` for a graph. [addnode.count,
   addnode.class]
3. `addnode(graph(), 3)` has 3 nodes and 0 edges (`EndNodes` 0×2).
   [addnode.count]
4. `addnode(graph({'a'}, {'b'}), {'c','d'})` has nodes a, b, c, d and edge
   a–b; with the char `'c'` it has nodes a, b, c. `addnode(graph(),
   {'p','q'})` has nodes p, q and no edges. [addnode.names]
5. `addnode(graph(1, 2), {'x'})` has node names Node1, Node2, x and edge
   Node1–Node2. [addnode.names-to-unnamed]
6. A name already used by a node raises `RunMat:addnode:NonUniqueNames`; a
   negative count raises `RunMat:addnode:InvalidNrNodes`.
   [addnode.error-dup-name, addnode.error-negative]

### User Story 2 - List categories (Priority: P1)

`C = categories(A)` returns the categories of a categorical array.

**Acceptance Scenarios** (normative):

1. `categories(categorical({'b','a','c','a'}))` is the 3×1 cell
   `{'a';'b';'c'}` of char vectors. [categories.signature-primary,
   categories.output-class, categories.sorted-default]
2. `{'b','B','a'}` gives `{'B';'a';'b'}`; the string array `["y" "x"]`
   gives `{'x';'y'}`. [categories.sorted-default]
3. An explicit list `{'c','b','a'}` with data `{'b','a'}` gives
   `{'c';'b';'a'}`; the ordinal `{'lo','hi'}` gives `{'lo';'hi'}`.
   [categories.explicit-order]
4. `categorical([3 1 2 1])` gives `{'1';'2';'3'}` (char). [categories.numeric]
5. `{'a', ''}` gives `{'a'}`. [categories.missing]
6. `categorical({})` gives a 0×0 cell. [categories.empty]

### User Story 3 - Read formatted text files (Priority: P1)

`[A, B, ...] = textread(filename, format, Name, Value)` reads one output per
non-skipped format field; `A = textread(filename)` reads a numeric matrix.

**Acceptance Scenarios** (normative):

1. File `1 2\n3 4\n` with `'%f %f'` gives `[1;3]` and `[2;4]` (double
   2×1); `7\n8\n9\n` with `'%f'` gives `[7;8;9]`.
   [textread.signature-primary, textread.columns]
2. `x 1\ny 2\n` with `'%s %d'` gives the 2×1 cell `{'x';'y'}` and the double
   `[1;2]`. [textread.types]
3. `h\n5\n6\n` with `'%f', 'headerlines', 1` gives `[5;6]`; `1,2\n3,4\n`
   with `'%f %f', 'delimiter', ','` gives `[1;3]` and `[2;4]`.
   [textread.options]
4. `x 1\ny 2\n` with `'%*s %d'` gives one output `[1;2]`. [textread.skip]
5. `textread(fn)` on `1 2\n3 4\n` gives `[1 2; 3 4]`.
   [textread.default-format]
6. An empty file with `'%f'` gives a 0×1 double. [textread.empty]
7. No warning is issued in these cases. [textread.no-warning]
8. A missing file raises `RunMat:textread:FileNotFound`.
   [textread.error-missing]

### User Story 4 - Unresolved forms fail clearly (Priority: P2)

Every form the exports do not observe either raises
`RunMat:<fn>:UnsupportedOption` or follows a documented choice (FR-052-17),
tagged `unresolved_choice_*` in tests.

## Requirements *(mandatory)*

### Functional Requirements

#### addnode

- **FR-052-01** `addnode(G, nodeCount)` MUST return a new object of the
  input class with `nodeCount` nodes appended and the edges unchanged; 0 adds
  none; this holds for the empty graph [addnode.signature-primary,
  addnode.count, addnode.class].
- **FR-052-02** `addnode(G, nodeNames)` with a char vector or a cellstr MUST
  append nodes with those names in order, the edges unchanged; on the empty
  graph this creates a named graph [addnode.names].
- **FR-052-03** Names added to a graph whose existing nodes are unnamed MUST
  name the existing nodes `Node1` … `NodeN` and append the new names
  [addnode.names-to-unnamed].
- **FR-052-04** A name already used by a node MUST raise
  `RunMat:addnode:NonUniqueNames` [addnode.error-dup-name]; a negative count
  MUST raise `RunMat:addnode:InvalidNrNodes` [addnode.error-negative].

#### categories

- **FR-052-05** `categories(A)` for a categorical `A` with at least one
  category MUST return an n×1 cell array of char row vectors, one per
  category, in the order the categorical stores them
  [categories.signature-primary, categories.output-class].
- **FR-052-06** For the observed text sources the order MUST be sorted (by
  character code: `B` before `a`), an explicit category list MUST be kept in
  the given order including unused categories, and numeric sources MUST give
  the decimal text of the values [categories.sorted-default,
  categories.explicit-order, categories.numeric]. (The order is fixed by
  RunMat's `categorical` constructor; `categories` reports it.)
- **FR-052-07** Empty text in the data MUST NOT be a category
  [categories.missing]; an empty categorical MUST give a 0×0 cell
  [categories.empty].

#### textread

- **FR-052-08** `textread(filename, format)` MUST read the file's
  whitespace-separated fields, assigning them to the format's fields in
  turn (cyclically), and return one column per non-skipped field
  [textread.signature-primary, textread.columns].
- **FR-052-09** `%f` and `%d` fields MUST give double columns; `%s` fields
  MUST give cell columns of char row vectors [textread.types].
- **FR-052-10** `'headerlines', n` MUST skip the first `n` lines;
  `'delimiter', ','` MUST split fields on commas (line ends still separate
  fields) [textread.options].
- **FR-052-11** `%*s` MUST read and discard a field without an output
  [textread.skip].
- **FR-052-12** `textread(filename)` MUST return the numbers as a double
  matrix with the file's row and column layout [textread.default-format].
- **FR-052-13** An empty file with `'%f'` MUST give a 0×1 double
  [textread.empty].
- **FR-052-14** No warning is issued [textread.no-warning].
- **FR-052-15** A file that does not exist MUST raise
  `RunMat:textread:FileNotFound` [textread.error-missing].

#### Common

- **FR-052-16** The builtins are registered through `#[runtime_builtin]`;
  `addnode` results are built with upstream's `graph_object`, so `G.Edges`,
  `G.Nodes`, `numnodes` and `numedges` stay consistent; the input is not
  modified.
- **FR-052-17** (documented choices, local ids; labelled, not asserted as
  MATLAB behaviour)
  - **U-052-1** Too few inputs raise `RunMat:<fn>:InvalidInput`; `addnode`
    and `categories` with more than one requested output raise
    `RunMat:<fn>:TooManyOutputs`.
  - **U-052-2** `addnode`: a first input that is not a graph/digraph raises
    `RunMat:addnode:UndefinedFunction`; more than two inputs, a numeric
    non-scalar, or any node input other than a numeric scalar, char vector,
    string scalar/array or cellstr raises `RunMat:addnode:UnsupportedOption`
    (`addnode.q-graph-variables`); a count that is NaN, infinite or not an
    integer raises `RunMat:addnode:InvalidNrNodes`; a count added to a graph
    with named nodes raises `RunMat:addnode:UnsupportedOption`; an empty
    name raises `RunMat:addnode:UnsupportedOption`; a name repeated within
    the new names, or equal to a generated `NodeK` name, raises
    `RunMat:addnode:NonUniqueNames`; string scalars/arrays are accepted as
    names like a cellstr.
  - **U-052-3** `categories`: a non-categorical input raises
    `RunMat:categories:UndefinedFunction` (matches the non-normative
    observation categories.non-categorical, dispatch-level in the export);
    more than one input raises `RunMat:categories:UnsupportedOption`; a
    categorical with no categories (empty or not) gives a 0×0 cell; a
    categorical with categories gives n×1 even when it has no elements.
  - **U-052-4** `textread` format grammar: only `%f`, `%d`, `%s` and `%*s`
    separated by optional whitespace are accepted; any other conversion,
    width/precision, other `%*` skip, literal text, or an empty format
    raises `RunMat:textread:UnsupportedOption` (`textread.q-more-formats`).
  - **U-052-5** `textread` data: a `%f` field accepts Rust's decimal float
    grammar (including `Inf`/`NaN` spellings); a `%d` field accepts an
    optionally signed digit string; anything else raises
    `RunMat:textread:UnsupportedOption`. A final record with fewer fields
    than the format raises `RunMat:textread:UnsupportedOption`. With a
    delimiter, fields are trimmed of surrounding whitespace and an empty
    field raises `RunMat:textread:UnsupportedOption`. `\r` is treated as
    whitespace; blank lines are skipped.
  - **U-052-6** `textread` options: names are matched case-insensitively;
    only `headerlines` (nonnegative integer scalar) and `delimiter` (a
    single character) are accepted; other names, a repeat-count argument, a
    multi-character delimiter, or options without a format raise
    `RunMat:textread:UnsupportedOption`; an odd name/value list or a
    malformed value raises `RunMat:textread:InvalidInput`.
  - **U-052-7** `textread(filename)`: whitespace-separated numbers per
    non-blank line; ragged rows or non-numeric text raise
    `RunMat:textread:UnsupportedOption`; an empty file gives a 0×0 double.
    An empty file with `%s` gives a 0×1 cell.
  - **U-052-8** `textread` outputs: in a context with no requested output
    count the first output is returned; when `k` outputs are requested and
    `k` is at most the number of non-skipped fields the first `k` are
    returned; more than that raises `RunMat:textread:TooManyOutputs`.
  - **U-052-9** `textread` file access: the path is used as given (relative
    to the current directory); any failure to read the file raises
    `RunMat:textread:FileNotFound` with the OS reason in the message; bytes
    that are not UTF-8 are decoded lossily.
- **FR-052-18** (unresolved, returned to the specification side)
  `addnode.q-warning-coverage`, `categories.q-warning-coverage`: the
  implementation emits no warnings. `addnode.q-graph-variables`: RunMat graph
  objects carry no node variables other than `Name`/`Index`; table node
  inputs raise UnsupportedOption (U-052-2). `textread.q-more-formats`:
  U-052-4/U-052-6.

### Key Entities

- `addnode`: input/output `Value::Object` of class `graph`/`digraph`, read
  with upstream `graph_data` and built with `graph_object`.
- `categories`: input `Value::Object` of class `categorical` (properties
  `Codes`, `Categories`); output `Value::Cell` of `Value::CharArray`.
- `textread`: input path (char/string); outputs `Value::Tensor` (double
  column or matrix) and `Value::Cell` columns, as `Value::OutputList` when an
  output count is requested.

## Success Criteria *(mandatory)*

- **SC-052-1** Every observed case of the three exports reproduces in a
  normative test.
- **SC-052-2** `cargo test -p runmat-runtime --lib -- addnode_tests
  categories_tests textread_tests` passes.
- **SC-052-3** Every FR maps to at least one test (validation.md).
- **SC-052-4** The graph, table and io suites show no regressions.

## Unresolved behaviour (non-normative)

| id | disposition |
|---|---|
| addnode.q-warning-coverage, categories.q-warning-coverage | no warnings emitted (FR-052-18) |
| addnode.q-graph-variables | U-052-2 (UnsupportedOption) |
| textread.q-more-formats | U-052-4, U-052-6 (UnsupportedOption) |
| U-052-1 … U-052-9 (local) | FR-052-17 |

## Known deviations

None expected. The addnode observations construct graphs only with
`graph(s, t)`, `digraph(s, t)` and `graph()`, whose edge order already
matches; D-036-1/D-043-1 (RunMat's `graph` constructor keeps edges in input
order and endpoints as given) and D-036-2 (`graph(s, t, w, n)` not accepted)
are upstream constructor gaps that do not affect these cases; `addnode`
never reorders edges.

## Assumptions

- Upstream's graph subsystem supplies `GraphData`, `graph_data`,
  `graph_object` and `parse_string_vector`.
- Upstream's `categorical` constructor and `categorical_categories`
  accessor supply the categories (FR-052-06/07 depend on the constructor's
  ordering and empty-text handling).
