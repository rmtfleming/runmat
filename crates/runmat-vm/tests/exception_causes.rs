//! Spec 060 (`specs/060-exception-causes/`): exception causes through the
//! interpreter's `try`/`catch` [addCause.thrown, throw.cause,
//! addCause.method-syntax, addCause.append].

#[path = "support/mod.rs"]
mod test_helpers;

use runmat_value::Value;
use test_helpers::execute_source;

fn has_num(vars: &[Value], expected: f64) -> bool {
    vars.iter()
        .any(|v| matches!(v, Value::Num(n) if (*n - expected).abs() < 1e-9))
}

fn caught_with_causes<'a>(
    vars: &'a [Value],
    identifier: &str,
) -> Option<&'a [runmat_value::MException]> {
    vars.iter().find_map(|v| match v {
        Value::MException(me) if me.identifier == identifier => Some(me.causes.as_slice()),
        _ => None,
    })
}

#[test]
fn thrown_causes_reach_catch() {
    // addCause.thrown / throw.cause: try throw(addCause(outer, c1)); catch x; end
    let vars = execute_source(
        "outer = MException('n4:outer', 'o'); c1 = MException('n4:c1', 'c'); \
         try; throw(addCause(outer, c1)); catch x; n = 1000 + numel(x.cause); end",
    )
    .unwrap();
    assert!(has_num(&vars, 1001.0), "numel(x.cause) should be 1");
    let causes = vars
        .iter()
        .find_map(|v| match v {
            Value::MException(me) if me.identifier == "n4:outer" && !me.causes.is_empty() => {
                Some(me.causes.clone())
            }
            _ => None,
        })
        .expect("caught exception with causes");
    assert_eq!(causes.len(), 1);
    assert_eq!(causes[0].identifier, "n4:c1");
}

#[test]
fn method_syntax_and_value_semantics() {
    // addCause.method-syntax, addCause.value-semantics, addCause.append
    let vars = execute_source(
        "e = MException('n4:o', 'o'); addCause(e, MException('n4:x', 'x')); \
         n0 = 2000 + numel(e.cause); \
         e = e.addCause(MException('n4:c', 'c')); \
         e = addCause(e, MException('n4:c2', 'c')); \
         n2 = 3000 + numel(e.cause);",
    )
    .unwrap();
    assert!(has_num(&vars, 2000.0), "value semantics: 0 causes");
    assert!(has_num(&vars, 3002.0), "two causes after two adds");
    let causes = caught_with_causes(&vars, "n4:o").expect("e");
    let ids: Vec<&str> = causes.iter().map(|c| c.identifier.as_str()).collect();
    assert_eq!(ids, vec!["n4:c", "n4:c2"]);
}

#[test]
fn caught_error_as_cause_then_rethrown() {
    // addCause.caught-cause; U-060-2 (rethrow keeps causes)
    let vars = execute_source(
        "try; error('n4:raised', 'r'); catch ME; e = addCause(MException('n4:outer', 'o'), ME); end; \
         try; rethrow(e); catch y; n = 4000 + numel(y.cause); end",
    )
    .unwrap();
    assert!(has_num(&vars, 4001.0), "rethrown exception keeps its cause");
}

#[test]
fn method_throw_carries_causes() {
    // throw.caught (case dot) with addCause.thrown: e.throw() through the VM
    let vars = execute_source(
        "e = addCause(MException('n4:dot', 'd'), MException('n4:c', 'c')); \
         try; e.throw(); catch y; n = 5000 + numel(y.cause); end",
    )
    .unwrap();
    assert!(has_num(&vars, 5001.0), "e.throw() keeps the cause");
}
