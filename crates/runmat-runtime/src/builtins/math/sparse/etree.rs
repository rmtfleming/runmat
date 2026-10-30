//! Clean-room `etree` (feature `specs/042-sparse-orderings/`).
//!
//! Normative behaviour comes only from the approved export `etree` 0.2.0:
//! a 1×n double parent vector (0 marks a root) of the elimination tree of the
//! upper triangle of `A`, or of `A'*A` with `'col'`, optionally with a
//! postorder. Algorithms: Liu's elimination tree with path compression and
//! the column elimination tree (see `symbolic.rs`). The sibling order of the
//! postorder is a documented choice (`etree.q-postorder-general`).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::Value;

use super::symbolic::{
    etree_col, etree_upper, gather, one_based, option_text, parent_values, postorder, read_pattern,
    row_vector, symbolic_error, upper_columns,
};
use crate::{BuiltinResult, RuntimeError};

const NAME: &str = "etree";

const fn param(
    name: &'static str,
    arity: BuiltinParamArity,
    description: &'static str,
) -> BuiltinParamDescriptor {
    BuiltinParamDescriptor {
        name,
        ty: BuiltinParamType::Any,
        arity,
        default: None,
        description,
    }
}

const IN_A: BuiltinParamDescriptor = param(
    "A",
    BuiltinParamArity::Required,
    "Sparse or full matrix; square unless 'col' is given.",
);
const IN_TYPE: BuiltinParamDescriptor = param(
    "type",
    BuiltinParamArity::Required,
    "'sym' (default) or 'col'.",
);
const OUT_P: BuiltinParamDescriptor = param(
    "p",
    BuiltinParamArity::Required,
    "1-by-n parent vector; 0 marks a root.",
);
const OUT_Q: BuiltinParamDescriptor = param(
    "q",
    BuiltinParamArity::Required,
    "1-by-n postorder permutation of the tree.",
);

const INPUTS_A: [BuiltinParamDescriptor; 1] = [IN_A];
const INPUTS_A_TYPE: [BuiltinParamDescriptor; 2] = [IN_A, IN_TYPE];
const OUTPUTS_P: [BuiltinParamDescriptor; 1] = [OUT_P];
const OUTPUTS_PQ: [BuiltinParamDescriptor; 2] = [OUT_P, OUT_Q];

const ETREE_SIGNATURES: [BuiltinSignatureDescriptor; 4] = [
    BuiltinSignatureDescriptor {
        label: "p = etree(A)",
        inputs: &INPUTS_A,
        outputs: &OUTPUTS_P,
    },
    BuiltinSignatureDescriptor {
        label: "p = etree(A, type)",
        inputs: &INPUTS_A_TYPE,
        outputs: &OUTPUTS_P,
    },
    BuiltinSignatureDescriptor {
        label: "[p, q] = etree(A)",
        inputs: &INPUTS_A,
        outputs: &OUTPUTS_PQ,
    },
    BuiltinSignatureDescriptor {
        label: "[p, q] = etree(A, type)",
        inputs: &INPUTS_A_TYPE,
        outputs: &OUTPUTS_PQ,
    },
];

const INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ETREE.INVALID_INPUT",
    identifier: Some("RunMat:etree:InvalidInput"),
    when: "A is not square and 'col' is not given, or A is not a 2-D numeric or logical matrix.",
    message: "etree: invalid input",
};

const UNSUPPORTED_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ETREE.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:etree:UnsupportedOption"),
    when: "An option other than 'sym' or 'col', or more than one option.",
    message: "etree: option not supported",
};

const TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ETREE.TOO_MANY_OUTPUTS",
    identifier: Some("RunMat:etree:TooManyOutputs"),
    when: "More than two outputs are requested.",
    message: "etree: at most two outputs are supported",
};

const INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.ETREE.INTERNAL",
    identifier: Some("RunMat:etree:Internal"),
    when: "The runtime cannot assemble the result.",
    message: "etree: internal runtime failure",
};

const ETREE_ERRORS: [BuiltinErrorDescriptor; 4] = [
    INVALID_INPUT,
    UNSUPPORTED_OPTION,
    TOO_MANY_OUTPUTS,
    INTERNAL,
];

pub const ETREE_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &ETREE_SIGNATURES,
    output_mode: BuiltinOutputMode::ByRequestedOutputCount,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &ETREE_ERRORS,
};

fn etree_error(descriptor: &BuiltinErrorDescriptor, detail: impl AsRef<str>) -> RuntimeError {
    symbolic_error(
        NAME,
        descriptor.identifier.expect("identifier"),
        detail.as_ref(),
    )
}

#[runtime_builtin(
    name = "etree",
    category = "math/sparse",
    summary = "Elimination tree of a sparse matrix (or of A'*A with 'col'), with optional postorder.",
    keywords = "etree,elimination tree,postorder,sparse,cholesky,symbolic",
    accel = "cpu",
    descriptor(crate::builtins::math::sparse::etree::ETREE_DESCRIPTOR),
    builtin_path = "crate::builtins::math::sparse::etree"
)]
async fn etree_builtin(a: Value, rest: Vec<Value>) -> BuiltinResult<Value> {
    let out_count = crate::output_count::current_output_count();
    if out_count.is_some_and(|count| count > 2) {
        return Err(etree_error(
            &TOO_MANY_OUTPUTS,
            "at most two outputs are supported",
        ));
    }
    let column_tree = match rest.as_slice() {
        [] => false,
        [option] => match option_text(&gather(option).await?).as_deref() {
            Some("sym") => false, // etree.sym-default
            Some("col") => true,  // etree.col
            _ => {
                return Err(etree_error(
                    &UNSUPPORTED_OPTION,
                    "the option must be 'sym' or 'col'",
                ))
            }
        },
        _ => {
            return Err(etree_error(
                &UNSUPPORTED_OPTION,
                "at most one option ('sym' or 'col') is supported",
            ))
        }
    };
    let a = gather(&a).await?;
    let pattern = read_pattern(&a).map_err(|e| etree_error(&INVALID_INPUT, e))?;
    let parent = if column_tree {
        etree_col(&pattern)
    } else {
        if pattern.rows != pattern.cols {
            // etree.error-nonsquare
            return Err(etree_error(
                &INVALID_INPUT,
                "A must be square unless 'col' is given",
            ));
        }
        // etree.upper-triangle
        etree_upper(&upper_columns(&pattern))
    };
    let internal = |e: String| etree_error(&INTERNAL, e);
    let p = row_vector(parent_values(&parent)).map_err(internal)?;
    match out_count {
        None => Ok(p),
        Some(0) => Ok(Value::OutputList(Vec::new())),
        Some(1) => Ok(Value::OutputList(vec![p])),
        Some(_) => {
            // etree.postorder
            let q = row_vector(one_based(&postorder(&parent))).map_err(internal)?;
            Ok(Value::OutputList(vec![p, q]))
        }
    }
}

#[cfg(test)]
mod etree_tests {
    use runmat_value::{CellArray, LogicalArray, SparseTensor, Tensor, Value};

    use super::super::symbolic::test_support::*;

    fn etree(args: &[Value]) -> Value {
        assert_no_warnings(|| call("etree", args)).expect("etree")
    }

    fn etree2(args: &[Value]) -> (Value, Value) {
        let mut outputs = assert_no_warnings(|| call_outputs("etree", args, 2)).expect("etree");
        assert_eq!(outputs.len(), 2);
        let q = outputs.pop().unwrap();
        (outputs.pop().unwrap(), q)
    }

    // --- Observed (normative): one test per observed case --------------------

    /// case `tridiagonal` — etree.signature-primary, etree.output-class,
    /// etree.parent-vector
    #[test]
    fn observed_tridiagonal() {
        assert_row(&etree(&[sparse(4, 4, &TRIDIAG)]), &[2.0, 3.0, 4.0, 0.0]);
    }

    /// case `arrow` — etree.parent-vector
    #[test]
    fn observed_arrow() {
        assert_row(&etree(&[sparse(4, 4, &ARROW)]), &[2.0, 3.0, 4.0, 0.0]);
    }

    /// case `diag` — etree.output-class, etree.parent-vector
    #[test]
    fn observed_diag() {
        assert_row(&etree(&[speye(3)]), &[0.0, 0.0, 0.0]);
    }

    /// case `pentadiag` — etree.output-class, etree.parent-vector
    #[test]
    fn observed_pentadiag() {
        assert_row(
            &etree(&[sparse(5, 5, &CORNER5)]),
            &[2.0, 3.0, 4.0, 5.0, 0.0],
        );
    }

    /// case `postorder` — etree.postorder
    #[test]
    fn observed_postorder() {
        let (p, q) = etree2(&[sparse(4, 4, &TRIDIAG)]);
        assert_row(&p, &[2.0, 3.0, 4.0, 0.0]);
        assert_row(&q, &[1.0, 2.0, 3.0, 4.0]);
    }

    /// case `postorder-arrow` — etree.postorder
    #[test]
    fn observed_postorder_arrow() {
        let (p, q) = etree2(&[sparse(4, 4, &ARROW)]);
        assert_row(&p, &[2.0, 3.0, 4.0, 0.0]);
        assert_row(&q, &[1.0, 2.0, 3.0, 4.0]);
    }

    /// case `col` — etree.col
    #[test]
    fn observed_col() {
        assert_row(&etree(&[sparse(3, 2, &COL32), text("col")]), &[2.0, 0.0]);
    }

    /// case `sym` — etree.sym-default
    #[test]
    fn observed_sym() {
        let a = sparse(4, 4, &TRIDIAG);
        let p = etree(&[a.clone(), text("sym")]);
        assert_row(&p, &[2.0, 3.0, 4.0, 0.0]);
        assert_eq!(p, etree(&[a]));
    }

    /// case `dense-input` — etree.dense-input
    #[test]
    fn observed_dense_input() {
        assert_row(&etree(&[dense(4, 4, &TRIDIAG)]), &[2.0, 3.0, 4.0, 0.0]);
    }

    /// case `upper-only` — etree.upper-triangle
    #[test]
    fn observed_upper_only() {
        let upper = triangle(4, &TRIDIAG, true);
        assert_row(&etree(&[sparse(4, 4, &upper)]), &[2.0, 3.0, 4.0, 0.0]);
    }

    /// case `lower-only` — etree.upper-triangle
    #[test]
    fn observed_lower_only() {
        let lower = triangle(4, &TRIDIAG, false);
        assert_row(&etree(&[sparse(4, 4, &lower)]), &[0.0, 0.0, 0.0, 0.0]);
    }

    /// case `postorder-branching` — etree.postorder, etree.branching
    #[test]
    fn observed_postorder_branching() {
        let (p, q) = etree2(&[sparse(4, 4, &BRANCHING)]);
        assert_row(&p, &[3.0, 3.0, 4.0, 0.0]);
        assert_row(&q, &[1.0, 2.0, 3.0, 4.0]);
    }

    /// case `empty` — etree.empty
    #[test]
    fn observed_empty() {
        assert_row(
            &etree(&[Value::SparseTensor(SparseTensor::zeros(0, 0))]),
            &[],
        );
    }

    /// case `err-nonsquare` — etree.error-nonsquare
    #[test]
    fn observed_nonsquare_errors() {
        assert_eq!(
            identifier(call("etree", &[sparse(3, 2, &COL32)])),
            "RunMat:etree:InvalidInput"
        );
    }

    // --- Documented choices ---------------------------------------------------

    /// FR-042-06/07: on random patterns the tree equals the oracle tree of the
    /// upper triangle (Boolean elimination), dense equals sparse, and the
    /// postorder is valid.
    #[test]
    fn documented_choice_random_patterns_match_oracle() {
        for (n, seed) in [(6, 11), (12, 12), (25, 13)] {
            let pattern = random_pattern(n, n, 15, seed);
            let filled = filled_pattern(n, &upper_edges(n, &pattern));
            let expected = oracle_parent(n, &filled);
            let (p, q) = etree2(&[sparse(n, n, &pattern)]);
            assert_row(&p, &expected);
            assert_row(&etree(&[dense(n, n, &pattern)]), &expected);
            assert_postorder(&expected, &assert_permutation(&q, n));
        }
    }

    /// FR-042-08: 'col' equals the tree of the A'*A pattern on random
    /// rectangular patterns (tall and wide).
    #[test]
    fn documented_choice_col_matches_ata_oracle() {
        for (rows, cols, seed) in [(9, 6, 21), (5, 10, 22), (20, 14, 23)] {
            let pattern = random_pattern(rows, cols, 20, seed);
            let filled = filled_pattern(cols, &ata_edges(rows, cols, &pattern));
            let expected = oracle_parent(cols, &filled);
            let (p, q) = etree2(&[sparse(rows, cols, &pattern), text("col")]);
            assert_row(&p, &expected);
            assert_postorder(&expected, &assert_permutation(&q, cols));
        }
    }

    /// FR-042-17 (etree.q-postorder-general): a forest whose postorder is not
    /// the identity. Roots and children are visited in increasing index order,
    /// so parent [3 0 0] gives [2 1 3].
    #[test]
    fn documented_choice_postorder_sibling_order() {
        let a = sparse(3, 3, &[1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0]);
        let (p, q) = etree2(&[a]);
        assert_row(&p, &[3.0, 0.0, 0.0]);
        assert_row(&q, &[2.0, 1.0, 3.0]);
        assert_postorder(&[3.0, 0.0, 0.0], &[2, 1, 3]);
    }

    /// FR-042-18: option text is case-insensitive and may be a string scalar.
    #[test]
    fn documented_choice_option_text_forms() {
        let a = sparse(3, 2, &COL32);
        assert_row(&etree(&[a.clone(), text("COL")]), &[2.0, 0.0]);
        assert_row(&etree(&[a, Value::String("col".into())]), &[2.0, 0.0]);
        assert_row(
            &etree(&[sparse(4, 4, &TRIDIAG), Value::String("Sym".into())]),
            &[2.0, 3.0, 4.0, 0.0],
        );
    }

    /// FR-042-08/10: 'col' on empty and zero-column inputs.
    #[test]
    fn documented_choice_col_empty_shapes() {
        let a = Value::SparseTensor(SparseTensor::zeros(3, 0));
        assert_row(&etree(&[a, text("col")]), &[]);
        let z = Value::SparseTensor(SparseTensor::zeros(0, 2));
        assert_row(&etree(&[z, text("col")]), &[0.0, 0.0]);
        let (p, q) = etree2(&[Value::SparseTensor(SparseTensor::zeros(0, 0))]);
        assert_row(&p, &[]);
        assert_row(&q, &[]);
    }

    /// FR-042-20: logical and scalar inputs are patterns; cell and N-D values
    /// raise InvalidInput.
    #[test]
    fn documented_choice_input_classes() {
        let logical = Value::LogicalArray(LogicalArray::new(vec![1, 0, 1, 1], vec![2, 2]).unwrap());
        assert_row(&etree(&[logical]), &[2.0, 0.0]);
        assert_row(&etree(&[Value::Num(2.0)]), &[0.0]);
        for bad in [
            Value::Cell(CellArray::new(vec![Value::Num(1.0)], 1, 1).unwrap()),
            Value::Tensor(Tensor::new(vec![1.0; 8], vec![2, 2, 2]).unwrap()),
        ] {
            assert_eq!(
                identifier(call("etree", std::slice::from_ref(&bad))),
                "RunMat:etree:InvalidInput",
                "{bad:?}"
            );
        }
    }

    /// FR-042-21: more than two outputs raise TooManyOutputs.
    #[test]
    fn documented_choice_too_many_outputs() {
        let result = call_direct("etree", &[sparse(4, 4, &TRIDIAG)], 3);
        assert_eq!(identifier(result), "RunMat:etree:TooManyOutputs");
    }

    /// FR-042-18: other option values and extra arguments are unsupported.
    #[test]
    fn documented_choice_other_options_unsupported() {
        let a = sparse(4, 4, &TRIDIAG);
        for args in [
            vec![a.clone(), text("row")],
            vec![a.clone(), Value::Num(1.0)],
            vec![a.clone(), text("col"), Value::Num(1.0)],
        ] {
            assert_eq!(
                identifier(call("etree", &args)),
                "RunMat:etree:UnsupportedOption",
                "{args:?}"
            );
        }
    }
}
