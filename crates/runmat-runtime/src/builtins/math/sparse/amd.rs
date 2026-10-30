//! Clean-room `amd` (feature `specs/042-sparse-orderings/`).
//!
//! Normative behaviour comes only from the approved export `amd` 0.2.0 (see
//! that feature's `provenance.md`): a 1×n double permutation of 1:n, full and
//! sparse input agree, 0×0 → 1×0, 1×1 → 1, non-square → `RunMat:square`.
//! The specific ordering is NOT normative (`amd.specific-orderings`,
//! `amd.q-ordering`). RunMat uses an exact minimum-degree ordering of the
//! graph of `A + A'` (Tinney & Walker 1967; George & Liu 1981), ties to the
//! smallest index. It is not the approximate-minimum-degree algorithm of
//! Amestoy, Davis and Duff and does not try to reproduce its orderings.

use std::collections::{BTreeSet, HashSet};

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::Value;

use super::symbolic::{gather, one_based, read_pattern, row_vector, symbolic_error};
use crate::{BuiltinResult, RuntimeError};

const NAME: &str = "amd";

const AMD_INPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "A",
    ty: BuiltinParamType::Any,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "Square matrix, sparse or full.",
}];

const AMD_OUTPUTS: [BuiltinParamDescriptor; 1] = [BuiltinParamDescriptor {
    name: "p",
    ty: BuiltinParamType::NumericArray,
    arity: BuiltinParamArity::Required,
    default: None,
    description: "1-by-n double row vector, a permutation of 1:n.",
}];

const AMD_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "p = amd(A)",
    inputs: &AMD_INPUTS,
    outputs: &AMD_OUTPUTS,
}];

const SQUARE: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.AMD.SQUARE",
    identifier: Some("RunMat:square"),
    when: "A is not square.",
    message: "amd: A must be square",
};

const INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.AMD.INVALID_INPUT",
    identifier: Some("RunMat:amd:InvalidInput"),
    when: "A is not a 2-D numeric or logical matrix.",
    message: "amd: A must be a 2-D numeric or logical matrix",
};

const UNSUPPORTED_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.AMD.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:amd:UnsupportedOption"),
    when: "Arguments after A (not yet specified for RunMat, amd.q-more-args).",
    message: "amd: arguments after A are not yet specified",
};

const TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.AMD.TOO_MANY_OUTPUTS",
    identifier: Some("RunMat:amd:TooManyOutputs"),
    when: "More than one output is requested.",
    message: "amd: at most one output is supported",
};

const INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.AMD.INTERNAL",
    identifier: Some("RunMat:amd:Internal"),
    when: "The runtime cannot assemble the result.",
    message: "amd: internal runtime failure",
};

const AMD_ERRORS: [BuiltinErrorDescriptor; 5] = [
    SQUARE,
    INVALID_INPUT,
    UNSUPPORTED_OPTION,
    TOO_MANY_OUTPUTS,
    INTERNAL,
];

pub const AMD_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &AMD_SIGNATURES,
    output_mode: BuiltinOutputMode::ByRequestedOutputCount,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &AMD_ERRORS,
};

fn amd_error(descriptor: &BuiltinErrorDescriptor, detail: impl AsRef<str>) -> RuntimeError {
    symbolic_error(
        NAME,
        descriptor.identifier.expect("identifier"),
        detail.as_ref(),
    )
}

#[runtime_builtin(
    name = "amd",
    category = "math/sparse",
    summary = "Fill-reducing symmetric permutation of a square matrix (minimum-degree ordering).",
    keywords = "amd,minimum degree,ordering,permutation,sparse,fill-in,cholesky",
    accel = "cpu",
    descriptor(crate::builtins::math::sparse::amd::AMD_DESCRIPTOR),
    builtin_path = "crate::builtins::math::sparse::amd"
)]
async fn amd_builtin(a: Value, rest: Vec<Value>) -> BuiltinResult<Value> {
    let out_count = crate::output_count::current_output_count();
    if out_count.is_some_and(|count| count > 1) {
        return Err(amd_error(
            &TOO_MANY_OUTPUTS,
            "at most one output is supported",
        ));
    }
    if !rest.is_empty() {
        // amd.q-more-args
        return Err(amd_error(
            &UNSUPPORTED_OPTION,
            "arguments after A are not yet specified (amd.q-more-args)",
        ));
    }
    let a = gather(&a).await?;
    let pattern = read_pattern(&a).map_err(|e| amd_error(&INVALID_INPUT, e))?;
    if pattern.rows != pattern.cols {
        // amd.error-nonsquare
        return Err(amd_error(&SQUARE, "A must be square"));
    }
    // amd.permutation, amd.dense-input: the order depends only on the pattern.
    let order = minimum_degree_order(pattern.cols, &pattern.columns);
    let p = row_vector(one_based(&order)).map_err(|e| amd_error(&INTERNAL, e))?;
    match out_count {
        Some(0) => Ok(Value::OutputList(Vec::new())),
        Some(_) => Ok(Value::OutputList(vec![p])),
        None => Ok(p),
    }
}

/// Exact minimum-degree elimination order of the graph of `A + A'` (diagonal
/// ignored). The vertex of smallest current degree is eliminated first (ties
/// → smallest index) and its neighbours become a clique.
fn minimum_degree_order(n: usize, columns: &[Vec<usize>]) -> Vec<usize> {
    let mut adjacency: Vec<HashSet<usize>> = vec![HashSet::new(); n];
    for (c, rows) in columns.iter().enumerate() {
        for &r in rows {
            if r != c {
                adjacency[r].insert(c);
                adjacency[c].insert(r);
            }
        }
    }
    let mut queue: BTreeSet<(usize, usize)> = (0..n).map(|v| (adjacency[v].len(), v)).collect();
    let mut order = Vec::with_capacity(n);
    while let Some((_, v)) = queue.pop_first() {
        order.push(v);
        let mut neighbours: Vec<usize> = adjacency[v].drain().collect();
        neighbours.sort_unstable();
        for &u in &neighbours {
            queue.remove(&(adjacency[u].len(), u));
            adjacency[u].remove(&v);
        }
        for &u in &neighbours {
            for &w in &neighbours {
                if u != w {
                    adjacency[u].insert(w);
                }
            }
        }
        for &u in &neighbours {
            queue.insert((adjacency[u].len(), u));
        }
    }
    order
}

#[cfg(test)]
mod amd_tests {
    use runmat_value::{
        CellArray, CharArray, ComplexTensor, LogicalArray, SparseTensor, Tensor, Value,
    };

    use super::super::symbolic::test_support::*;

    fn amd(args: &[Value]) -> Value {
        assert_no_warnings(|| call("amd", args)).expect("amd")
    }

    // --- Observed (normative): one test per observed case --------------------

    /// case `perm` — amd.signature-primary, amd.output-class, amd.permutation
    #[test]
    fn observed_perm() {
        let p = amd(&[sparse(5, 5, &CORNER5)]);
        assert_permutation(&p, 5);
    }

    /// case `arrow` — amd.permutation
    #[test]
    fn observed_arrow() {
        let p = amd(&[sparse(4, 4, &ARROW)]);
        assert_permutation(&p, 4);
    }

    /// case `dense-input` — amd.dense-input
    #[test]
    fn observed_dense_input_matches_sparse() {
        let from_sparse = amd(&[sparse(4, 4, &ARROW)]);
        let from_dense = amd(&[dense(4, 4, &ARROW)]);
        let p = assert_permutation(&from_dense, 4);
        assert_eq!(p, assert_permutation(&from_sparse, 4));
    }

    /// case `class` — amd.output-class
    #[test]
    fn observed_class() {
        let p = amd(&[sparse(4, 4, &TRIDIAG)]);
        assert_permutation(&p, 4);
    }

    /// case `diag` — amd.output-class, amd.permutation
    #[test]
    fn observed_diag() {
        let p = amd(&[speye(3)]);
        assert_permutation(&p, 3);
    }

    /// case `empty` — amd.trivial-sizes
    #[test]
    fn observed_empty() {
        let p = amd(&[Value::SparseTensor(SparseTensor::zeros(0, 0))]);
        assert_row(&p, &[]);
    }

    /// case `scalar` — amd.trivial-sizes
    #[test]
    fn observed_scalar() {
        let p = amd(&[sparse(1, 1, &[5.0])]);
        assert_row(&p, &[1.0]);
    }

    /// case `err-nonsquare` — amd.error-nonsquare
    #[test]
    fn observed_nonsquare_errors() {
        let a = sparse(2, 3, &[1.0; 6]);
        assert_eq!(identifier(call("amd", &[a])), "RunMat:square");
    }

    // --- Documented choices (FR-042-15, 20, 21) ------------------------------

    /// FR-042-15 (amd.q-ordering; amd.specific-orderings is non-normative):
    /// RunMat's own minimum-degree orderings for the observed inputs, recorded
    /// for traceability only. They are NOT required to match the reference.
    #[test]
    fn documented_choice_orderings_on_observed_inputs() {
        let perm = assert_permutation(&amd(&[sparse(5, 5, &CORNER5)]), 5);
        let arrow = assert_permutation(&amd(&[sparse(4, 4, &ARROW)]), 4);
        let diag = assert_permutation(&amd(&[speye(3)]), 3);
        let tridiag = assert_permutation(&amd(&[sparse(4, 4, &TRIDIAG)]), 4);
        assert_eq!(perm, vec![1, 2, 3, 4, 5]);
        assert_eq!(arrow, vec![2, 3, 1, 4]);
        assert_eq!(diag, vec![1, 2, 3]);
        assert_eq!(tridiag, vec![1, 2, 3, 4]);
    }

    /// FR-042-02/03 on larger, unsymmetric patterns: always a permutation, and
    /// full equals sparse.
    #[test]
    fn documented_choice_larger_patterns_dense_equals_sparse() {
        for (n, seed) in [(8, 1), (15, 2), (30, 3)] {
            let pattern = random_pattern(n, n, 20, seed);
            let p_sparse = assert_permutation(&amd(&[sparse(n, n, &pattern)]), n);
            let p_dense = assert_permutation(&amd(&[dense(n, n, &pattern)]), n);
            assert_eq!(p_sparse, p_dense, "n = {n}");
        }
    }

    /// FR-042-15: the ordering is fill-reducing on the arrow matrix: the dense
    /// node (1) is not eliminated first.
    #[test]
    fn documented_choice_arrow_dense_node_not_first() {
        let p = assert_permutation(&amd(&[sparse(4, 4, &ARROW)]), 4);
        assert_ne!(p[0], 1);
    }

    /// FR-042-20: logical, single, complex, scalar and dense inputs are read as
    /// patterns; text, cell and N-D values raise InvalidInput.
    #[test]
    fn documented_choice_input_classes() {
        let logical = Value::LogicalArray(LogicalArray::new(vec![1, 1, 0, 1], vec![2, 2]).unwrap());
        assert_permutation(&amd(&[logical]), 2);
        let single = Value::Tensor(Tensor::from_f32(vec![1.0, 1.0, 1.0, 1.0], vec![2, 2]).unwrap());
        assert_permutation(&amd(&[single]), 2);
        let complex = Value::ComplexTensor(
            ComplexTensor::new(
                vec![(1.0, 0.0), (0.0, 1.0), (0.0, 1.0), (1.0, 0.0)],
                vec![2, 2],
            )
            .unwrap(),
        );
        assert_permutation(&amd(&[complex]), 2);
        assert_row(&amd(&[Value::Num(3.0)]), &[1.0]);
        for bad in [
            Value::CharArray(CharArray::new_row("ab")),
            Value::Cell(CellArray::new(vec![Value::Num(1.0)], 1, 1).unwrap()),
            Value::Tensor(Tensor::new(vec![1.0; 8], vec![2, 2, 2]).unwrap()),
        ] {
            assert_eq!(
                identifier(call("amd", std::slice::from_ref(&bad))),
                "RunMat:amd:InvalidInput",
                "{bad:?}"
            );
        }
    }

    /// FR-042-21: more than one output raises TooManyOutputs.
    #[test]
    fn documented_choice_too_many_outputs() {
        let result = call_direct("amd", &[sparse(4, 4, &ARROW)], 2);
        assert_eq!(identifier(result), "RunMat:amd:TooManyOutputs");
    }

    // --- Unresolved (amd.q-more-args) ----------------------------------------

    /// FR-042-16: any argument after A raises UnsupportedOption.
    #[test]
    fn unresolved_more_args_unsupported() {
        for extra in [Value::Num(10.0), text("opts")] {
            assert_eq!(
                identifier(call("amd", &[sparse(4, 4, &ARROW), extra.clone()])),
                "RunMat:amd:UnsupportedOption",
                "{extra:?}"
            );
        }
    }
}
