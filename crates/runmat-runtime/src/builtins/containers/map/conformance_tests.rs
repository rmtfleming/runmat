//! Feature 048 (containers.Map conformance): spec-derived tests for `keys`,
//! `isKey` and `remove` (approved exports 0.2.0, matlab-interface-spec
//! `8f72edb`). Tests exercise the public dispatch routes — the function-call
//! form `keys(M)` (instance-method dispatch by name), the qualified method
//! name, and the member forms `M.keys` / `M.keys()`.

use crate::sequence::ResolveValueSequence;
use futures::executor::block_on;
use runmat_value::{CellArray, CharArray, IntValue, Tensor, Value};

const NO_KEY_TO_REMOVE: &str = "RunMat:Containers:Map:NoKeyToRemove";

fn call(name: &str, args: &[Value]) -> crate::BuiltinResult<Value> {
    block_on(crate::call_builtin_async(name, args))
}

fn ch(text: &str) -> Value {
    Value::CharArray(CharArray::new_row(text))
}

fn row_cell(values: Vec<Value>) -> Value {
    let n = values.len();
    Value::Cell(CellArray::new(values, 1, n).expect("cell"))
}

fn char_map(keys: &[&str], values: &[f64]) -> Value {
    let keys = row_cell(keys.iter().map(|k| ch(k)).collect());
    let values = row_cell(values.iter().map(|v| Value::Num(*v)).collect());
    call("containers.Map", &[keys, values]).expect("char map")
}

/// containers.Map([3 1 2], {'x','y','z'})
fn numeric_map() -> Value {
    let keys = Value::Tensor(Tensor::new(vec![3.0, 1.0, 2.0], vec![1, 3]).expect("tensor"));
    let values = row_cell(vec![ch("x"), ch("y"), ch("z")]);
    call("containers.Map", &[keys, values]).expect("numeric map")
}

/// containers.Map({'b','a','c'}, {1, 2, 3})
fn bac_map() -> Value {
    char_map(&["b", "a", "c"], &[1.0, 2.0, 3.0])
}

fn cell_of(value: &Value) -> &CellArray {
    match value {
        Value::Cell(cell) => cell,
        other => panic!("expected cell, got {other:?}"),
    }
}

fn char_entries(value: &Value) -> Vec<String> {
    let cell = cell_of(value);
    cell.data
        .iter()
        .map(|entry| match entry {
            Value::CharArray(ca) => ca.data.iter().collect(),
            other => panic!("expected char key, got {other:?}"),
        })
        .collect()
}

fn num_entries(value: &Value) -> Vec<f64> {
    let cell = cell_of(value);
    cell.data
        .iter()
        .map(|entry| match entry {
            Value::Num(n) => *n,
            other => panic!("expected double, got {other:?}"),
        })
        .collect()
}

fn count(map: &Value) -> u64 {
    match block_on(crate::object::resolve::load_member(
        map.clone(),
        "Count".to_string(),
        false,
        None,
    ))
    .expect("Count")
    {
        Value::Int(IntValue::U64(n)) => n,
        Value::Num(n) => n as u64,
        other => panic!("unexpected Count {other:?}"),
    }
}

fn logical_column_major(value: &Value) -> (Vec<usize>, Vec<bool>) {
    match value {
        Value::Bool(b) => (vec![1, 1], vec![*b]),
        Value::LogicalArray(arr) => (
            arr.shape.clone(),
            arr.data.iter().map(|b| *b != 0).collect(),
        ),
        other => panic!("expected logical, got {other:?}"),
    }
}

fn reset_last_warning() {
    crate::warnings::with_policy(|policy| policy.reset());
}

fn last_warning_id() -> Option<String> {
    crate::warnings::with_policy(|policy| policy.last_warning()).map(|w| w.identifier)
}

fn class_of(value: &Value) -> String {
    match call("class", std::slice::from_ref(value)).expect("class") {
        Value::CharArray(ca) => ca.data.iter().collect(),
        Value::String(s) => s,
        other => panic!("unexpected class {other:?}"),
    }
}

// ---------------------------------------------------------------- keys

#[test]
fn observed_keys_char_sorted_row_cell() {
    // keys.signature-primary, keys.output-class, keys.sorted (case char)
    let keys = call("keys", &[bac_map()]).expect("keys");
    let cell = cell_of(&keys);
    assert_eq!((cell.rows, cell.cols), (1, 3));
    assert_eq!(char_entries(&keys), vec!["a", "b", "c"]);
}

#[test]
fn observed_keys_numeric_sorted() {
    // keys.output-class, keys.sorted (case numeric)
    let keys = call("keys", &[numeric_map()]).expect("keys");
    let cell = cell_of(&keys);
    assert_eq!((cell.rows, cell.cols), (1, 3));
    assert_eq!(num_entries(&keys), vec![1.0, 2.0, 3.0]);
}

#[test]
fn observed_keys_empty_map_is_1x0() {
    // keys.output-class (case empty)
    let map = call("containers.Map", &[]).expect("map");
    let keys = call("keys", &[map]).expect("keys");
    let cell = cell_of(&keys);
    assert_eq!(cell.shape, vec![1, 0]);
    assert!(cell.data.is_empty());
}

#[test]
fn observed_keys_case_order_uppercase_first() {
    // keys.sorted (case case-order)
    let map = char_map(&["b", "B", "a", "A"], &[1.0, 2.0, 3.0, 4.0]);
    let keys = call("keys", &[map]).expect("keys");
    assert_eq!(char_entries(&keys), vec!["A", "B", "a", "b"]);
}

#[test]
fn observed_values_aligned_with_keys() {
    // keys.aligned-with-values (case values-aligned)
    let map = bac_map();
    let keys = call("keys", std::slice::from_ref(&map)).expect("keys");
    let values = call("values", &[map]).expect("values");
    assert_eq!(char_entries(&keys), vec!["a", "b", "c"]);
    assert_eq!(num_entries(&values), vec![2.0, 1.0, 3.0]);
}

#[test]
fn observed_keys_method_syntax_forms() {
    // keys.method-syntax (cases method-syntax, char): M.keys, M.keys() and the
    // qualified method name give the same result as keys(M).
    let map = bac_map();
    let member = block_on(crate::object::resolve::load_member(
        map.clone(),
        "keys".to_string(),
        false,
        None,
    ))
    .expect("M.keys");
    assert_eq!(char_entries(&member), vec!["a", "b", "c"]);

    let mut seq = block_on(
        crate::object::dispatch::call_method_or_member_index_named_with_outputs(
            map.clone(),
            "keys".to_string(),
            Vec::new(),
            1,
            None,
        ),
    )
    .expect("M.keys()")
    .resolve(
        runmat_types::SequenceUse::RequireSingle,
        crate::sequence::SequenceResolutionContext::default(),
    )
    .expect("single");
    let called = seq.pop().expect("value");
    assert_eq!(char_entries(&called), vec!["a", "b", "c"]);

    let qualified = call("containers.Map.keys", &[map]).expect("qualified");
    assert_eq!(char_entries(&qualified), vec!["a", "b", "c"]);
}

#[test]
fn unresolved_choice_keys_non_map_errors() {
    // keys.non-map (non-normative, dispatch level): a struct input errors.
    // RunMat's identifier is not asserted beyond being an error.
    let mut st = runmat_value::StructValue::new();
    st.fields.insert("a".to_string(), Value::Num(1.0));
    assert!(call("keys", &[Value::Struct(st)]).is_err());
}

// ---------------------------------------------------------------- isKey

#[test]
fn observed_iskey_present_absent_case() {
    // isKey.signature-primary, isKey.output-class, isKey.membership
    let map = bac_map();
    assert_eq!(
        call("isKey", &[map.clone(), ch("a")]).expect("present"),
        Value::Bool(true)
    );
    assert_eq!(
        call("isKey", &[map.clone(), ch("z")]).expect("absent"),
        Value::Bool(false)
    );
    assert_eq!(
        call("isKey", &[map, ch("A")]).expect("case"),
        Value::Bool(false)
    );
}

#[test]
fn observed_iskey_cell_query_same_shape() {
    // isKey.cell-query (case cell-query): {'a','z';'c','b'} -> [true false; true true]
    let query = CellArray::new(vec![ch("a"), ch("z"), ch("c"), ch("b")], 2, 2).expect("cell");
    assert_eq!(query.get(0, 1).expect("(1,2)"), ch("z"));
    let result = call("isKey", &[bac_map(), Value::Cell(query)]).expect("isKey");
    let (shape, data) = logical_column_major(&result);
    assert_eq!(shape, vec![2, 2]);
    // column-major [ (1,1) (2,1) (1,2) (2,2) ] = [true true false true]
    assert_eq!(data, vec![true, true, false, true]);
}

#[test]
fn observed_iskey_numeric_and_numeric_cell() {
    // isKey.numeric (cases numeric, numeric-cell)
    let map = numeric_map();
    assert_eq!(
        call("isKey", &[map.clone(), Value::Num(2.0)]).expect("2"),
        Value::Bool(true)
    );
    assert_eq!(
        call("isKey", &[map.clone(), Value::Num(5.0)]).expect("5"),
        Value::Bool(false)
    );
    let query = row_cell(vec![Value::Num(1.0), Value::Num(4.0)]);
    let (shape, data) = logical_column_major(&call("isKey", &[map, query]).expect("cell"));
    assert_eq!(shape, vec![1, 2]);
    assert_eq!(data, vec![true, false]);
}

#[test]
fn observed_iskey_lenient_type_mismatch_and_empty_map() {
    // isKey.lenient (cases type-mismatch, empty-map)
    assert_eq!(
        call("isKey", &[numeric_map(), ch("a")]).expect("type-mismatch"),
        Value::Bool(false)
    );
    let empty = call("containers.Map", &[]).expect("map");
    assert_eq!(
        call("isKey", &[empty, ch("a")]).expect("empty-map"),
        Value::Bool(false)
    );
}

#[test]
fn observed_iskey_string_scalar_key() {
    // isKey.string-key (case string-key)
    assert_eq!(
        call("isKey", &[bac_map(), Value::String("a".to_string())]).expect("string"),
        Value::Bool(true)
    );
}

#[test]
fn unresolved_choice_iskey_mismatch_inside_cell_is_false() {
    // U-048-3: a class-mismatched element inside a cell query is false too.
    let query = row_cell(vec![Value::Num(1.0), ch("a")]);
    let (_, data) = logical_column_major(&call("isKey", &[numeric_map(), query]).expect("cell"));
    assert_eq!(data, vec![true, false]);
}

// ---------------------------------------------------------------- remove

fn remove_case(map: &Value, key_set: Value) -> (Value, Option<String>) {
    reset_last_warning();
    let returned = call("remove", &[map.clone(), key_set]).expect("remove");
    (returned, last_warning_id())
}

#[test]
fn observed_remove_one_key_handle_semantics() {
    // remove.signature-primary, remove.removes, remove.handle, remove.returns-map (case one)
    let map = bac_map();
    let (returned, warning) = remove_case(&map, ch("a"));
    assert_eq!(
        char_entries(&call("keys", std::slice::from_ref(&returned)).unwrap()),
        vec!["b", "c"]
    );
    assert_eq!(
        char_entries(&call("keys", std::slice::from_ref(&map)).unwrap()),
        vec!["b", "c"]
    );
    assert_eq!(count(&map), 2);
    assert_eq!(class_of(&returned), "containers.Map");
    assert_eq!(warning, None);
}

#[test]
fn observed_remove_several_numeric_all() {
    // remove.removes, remove.handle (cases several, numeric, all)
    let map = bac_map();
    let (returned, warning) = remove_case(&map, row_cell(vec![ch("a"), ch("c")]));
    assert_eq!(char_entries(&call("keys", &[returned]).unwrap()), vec!["b"]);
    assert_eq!(count(&map), 1);
    assert_eq!(warning, None);

    let map = numeric_map();
    let (returned, warning) = remove_case(&map, Value::Num(1.0));
    assert_eq!(
        num_entries(&call("keys", &[returned]).unwrap()),
        vec![2.0, 3.0]
    );
    assert_eq!(
        num_entries(&call("keys", std::slice::from_ref(&map)).unwrap()),
        vec![2.0, 3.0]
    );
    assert_eq!(count(&map), 2);
    assert_eq!(warning, None);

    let map = bac_map();
    let (returned, warning) = remove_case(&map, row_cell(vec![ch("a"), ch("b"), ch("c")]));
    assert!(cell_of(&call("keys", &[returned]).unwrap()).data.is_empty());
    assert_eq!(count(&map), 0);
    assert_eq!(warning, None);
}

#[test]
fn observed_remove_absent_key_warns_and_completes() {
    // remove.warning-absent, remove.returns-map (case absent)
    let map = bac_map();
    let (returned, warning) = remove_case(&map, ch("z"));
    assert_eq!(class_of(&returned), "containers.Map");
    assert_eq!(
        char_entries(&call("keys", std::slice::from_ref(&map)).unwrap()),
        vec!["a", "b", "c"]
    );
    assert_eq!(count(&map), 3);
    assert_eq!(warning.as_deref(), Some(NO_KEY_TO_REMOVE));
}

#[test]
fn observed_remove_partly_absent_removes_present() {
    // remove.present-removed-with-absent, remove.warning-absent (case partly-absent)
    let map = bac_map();
    let (_, warning) = remove_case(&map, row_cell(vec![ch("a"), ch("z")]));
    assert_eq!(
        char_entries(&call("keys", std::slice::from_ref(&map)).unwrap()),
        vec!["b", "c"]
    );
    assert_eq!(count(&map), 2);
    assert_eq!(warning.as_deref(), Some(NO_KEY_TO_REMOVE));
}

#[test]
fn observed_remove_type_mismatch_warns() {
    // remove.warning-absent (case type-mismatch)
    let map = numeric_map();
    let (_, warning) = remove_case(&map, ch("a"));
    assert_eq!(
        num_entries(&call("keys", std::slice::from_ref(&map)).unwrap()),
        vec![1.0, 2.0, 3.0]
    );
    assert_eq!(count(&map), 3);
    assert_eq!(warning.as_deref(), Some(NO_KEY_TO_REMOVE));
}

#[test]
#[ignore = "D-048-1: the dispatcher re-invokes the method after an error and reports internal storage not found"]
fn deviation_remove_warning_error_mode_through_dispatcher() {
    // U-048-2: the warning goes through RunMat's warning policy, so
    // warning('error', id) turns it into an error with the same identifier.
    let map = bac_map();
    reset_last_warning();
    crate::warnings::with_policy(|policy| {
        policy.set_identifier_mode(NO_KEY_TO_REMOVE, crate::warnings::WarningMode::Error)
    });
    let result = call("remove", &[map.clone(), ch("z")]);
    reset_last_warning();
    let err = result.expect_err("as error");
    assert_eq!(err.identifier(), Some(NO_KEY_TO_REMOVE), "{err:?}");
    assert_eq!(count(&map), 3);
}
