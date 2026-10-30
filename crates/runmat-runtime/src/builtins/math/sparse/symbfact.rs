//! Clean-room `symbfact` (feature `specs/042-sparse-orderings/`).
//!
//! Normative behaviour comes only from the approved export `symbfact` 0.2.0:
//! the column counts of the predicted Cholesky factor (the nonzeros in each
//! row of the upper-triangular factor pattern `R`), optionally with the tree
//! height, parent vector, postorder and `R`; `'col'` analyses `A'*A`.
//! Algorithm: symbolic factorization by merging child structures along the
//! elimination tree (George & Liu 1981; Liu 1990). Only the upper triangle of
//! a square `A` is used, consistent with `etree` (documented choice).

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{SparseTensor, Value};

use super::symbolic::{
    children_of, etree_col, etree_upper, gather, height, one_based, option_text, parent_values,
    postorder, read_pattern, row_vector, symbolic_error, upper_columns, Pattern,
};
use crate::{BuiltinResult, RuntimeError};

const NAME: &str = "symbfact";

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
    "'col' to analyse A'*A.",
);
const OUT_COUNT: BuiltinParamDescriptor = param(
    "count",
    BuiltinParamArity::Required,
    "1-by-n column counts of the predicted Cholesky factor.",
);
const OUTPUTS_ALL: [BuiltinParamDescriptor; 5] = [
    OUT_COUNT,
    param(
        "h",
        BuiltinParamArity::Required,
        "Height of the elimination tree.",
    ),
    param(
        "parent",
        BuiltinParamArity::Required,
        "Elimination tree parent vector.",
    ),
    param(
        "post",
        BuiltinParamArity::Required,
        "Postorder permutation.",
    ),
    param(
        "R",
        BuiltinParamArity::Required,
        "Sparse upper-triangular pattern of the factor (ones).",
    ),
];

const INPUTS_A: [BuiltinParamDescriptor; 1] = [IN_A];
const INPUTS_A_TYPE: [BuiltinParamDescriptor; 2] = [IN_A, IN_TYPE];
const OUTPUTS_COUNT: [BuiltinParamDescriptor; 1] = [OUT_COUNT];

const SYMBFACT_SIGNATURES: [BuiltinSignatureDescriptor; 4] = [
    BuiltinSignatureDescriptor {
        label: "count = symbfact(A)",
        inputs: &INPUTS_A,
        outputs: &OUTPUTS_COUNT,
    },
    BuiltinSignatureDescriptor {
        label: "count = symbfact(A, 'col')",
        inputs: &INPUTS_A_TYPE,
        outputs: &OUTPUTS_COUNT,
    },
    BuiltinSignatureDescriptor {
        label: "[count, h, parent, post, R] = symbfact(A)",
        inputs: &INPUTS_A,
        outputs: &OUTPUTS_ALL,
    },
    BuiltinSignatureDescriptor {
        label: "[count, h, parent, post, R] = symbfact(A, 'col')",
        inputs: &INPUTS_A_TYPE,
        outputs: &OUTPUTS_ALL,
    },
];

const INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SYMBFACT.INVALID_INPUT",
    identifier: Some("RunMat:symbfact:InvalidInput"),
    when: "A is not square and 'col' is not given, or A is not a 2-D numeric or logical matrix.",
    message: "symbfact: invalid input",
};

const UNSUPPORTED_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SYMBFACT.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:symbfact:UnsupportedOption"),
    when: "An option other than 'col', or more arguments (symbfact.q-more-options).",
    message: "symbfact: option not yet specified",
};

const TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SYMBFACT.TOO_MANY_OUTPUTS",
    identifier: Some("RunMat:symbfact:TooManyOutputs"),
    when: "More than five outputs are requested.",
    message: "symbfact: at most five outputs are supported",
};

const INTERNAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SYMBFACT.INTERNAL",
    identifier: Some("RunMat:symbfact:Internal"),
    when: "The runtime cannot assemble the result.",
    message: "symbfact: internal runtime failure",
};

const SYMBFACT_ERRORS: [BuiltinErrorDescriptor; 4] = [
    INVALID_INPUT,
    UNSUPPORTED_OPTION,
    TOO_MANY_OUTPUTS,
    INTERNAL,
];

pub const SYMBFACT_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &SYMBFACT_SIGNATURES,
    output_mode: BuiltinOutputMode::ByRequestedOutputCount,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &SYMBFACT_ERRORS,
};

fn symbfact_error(descriptor: &BuiltinErrorDescriptor, detail: impl AsRef<str>) -> RuntimeError {
    symbolic_error(
        NAME,
        descriptor.identifier.expect("identifier"),
        detail.as_ref(),
    )
}

#[runtime_builtin(
    name = "symbfact",
    category = "math/sparse",
    summary = "Symbolic Cholesky factorization: predicted column counts, tree height, elimination tree, postorder and factor pattern.",
    keywords = "symbfact,symbolic factorization,cholesky,column counts,elimination tree,sparse",
    accel = "cpu",
    descriptor(crate::builtins::math::sparse::symbfact::SYMBFACT_DESCRIPTOR),
    builtin_path = "crate::builtins::math::sparse::symbfact"
)]
async fn symbfact_builtin(a: Value, rest: Vec<Value>) -> BuiltinResult<Value> {
    let out_count = crate::output_count::current_output_count();
    if out_count.is_some_and(|count| count > 5) {
        return Err(symbfact_error(
            &TOO_MANY_OUTPUTS,
            "at most five outputs are supported",
        ));
    }
    let column_mode = match rest.as_slice() {
        [] => false,
        // symbfact.col; anything else is symbfact.q-more-options.
        [option] => {
            if option_text(&gather(option).await?).as_deref() == Some("col") {
                true
            } else {
                return Err(symbfact_error(
                    &UNSUPPORTED_OPTION,
                    "only the 'col' option is supported (symbfact.q-more-options)",
                ));
            }
        }
        _ => {
            return Err(symbfact_error(
                &UNSUPPORTED_OPTION,
                "only the 'col' option is supported (symbfact.q-more-options)",
            ))
        }
    };
    let a = gather(&a).await?;
    let pattern = read_pattern(&a).map_err(|e| symbfact_error(&INVALID_INPUT, e))?;
    if !column_mode && pattern.rows != pattern.cols {
        // symbfact.error-nonsquare
        return Err(symbfact_error(
            &INVALID_INPUT,
            "A must be square unless 'col' is given",
        ));
    }
    let (parent, below) = if column_mode {
        (etree_col(&pattern), ata_below(&pattern))
    } else {
        let upper = upper_columns(&pattern);
        let mut below = vec![Vec::new(); pattern.cols];
        for (k, rows) in upper.iter().enumerate() {
            for &i in rows {
                below[i].push(k);
            }
        }
        (etree_upper(&upper), below)
    };
    let structure = factor_structure(&parent, &below);

    let internal = |e: String| symbfact_error(&INTERNAL, e);
    let counts: Vec<f64> = structure.iter().map(|s| s.len() as f64).collect();
    let mut outputs = vec![row_vector(counts).map_err(internal)?];
    let wanted = out_count.unwrap_or(1);
    if wanted >= 2 {
        outputs.push(Value::Num(height(&parent) as f64));
    }
    if wanted >= 3 {
        outputs.push(row_vector(parent_values(&parent)).map_err(internal)?);
    }
    if wanted >= 4 {
        outputs.push(row_vector(one_based(&postorder(&parent))).map_err(internal)?);
    }
    if wanted >= 5 {
        outputs.push(factor_pattern(&structure).map_err(internal)?);
    }
    match out_count {
        None => Ok(outputs.swap_remove(0)),
        Some(count) => {
            outputs.truncate(count);
            Ok(Value::OutputList(outputs))
        }
    }
}

/// For each column `i` of `A'*A`, the columns `j > i` sharing a row with it.
fn ata_below(pattern: &Pattern) -> Vec<Vec<usize>> {
    let mut cols_in_row: Vec<Vec<usize>> = vec![Vec::new(); pattern.rows];
    for (c, rows) in pattern.columns.iter().enumerate() {
        for &r in rows {
            cols_in_row[r].push(c);
        }
    }
    let mut below = vec![Vec::new(); pattern.cols];
    for cols in &cols_in_row {
        // Columns are pushed in increasing order, so `cols` is sorted.
        for (x, &i) in cols.iter().enumerate() {
            below[i].extend(cols[x + 1..].iter().copied().filter(|&j| j != i));
        }
    }
    below
}

/// Row structure of each column of `L` (diagonal included, sorted):
/// `{j} ∪ below(j) ∪ ⋃_{children c} (Struct(c) \ {c})`.
fn factor_structure(parent: &[Option<usize>], below: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let n = parent.len();
    let children = children_of(parent);
    let mut mark = vec![usize::MAX; n];
    let mut structure: Vec<Vec<usize>> = Vec::with_capacity(n);
    for j in 0..n {
        let mut s = vec![j];
        mark[j] = j;
        let inherited = children[j]
            .iter()
            .flat_map(|&c| structure[c].iter().copied().filter(move |&i| i != c));
        for i in below[j].iter().copied().chain(inherited) {
            if mark[i] != j {
                mark[i] = j;
                s.push(i);
            }
        }
        s.sort_unstable();
        structure.push(s);
    }
    structure
}

/// `R = L'` as an n×n sparse double of ones: column `i` of `R` holds the rows
/// `j <= i` with `i ∈ Struct(L(:, j))`.
fn factor_pattern(structure: &[Vec<usize>]) -> Result<Value, String> {
    let n = structure.len();
    let mut rows_in_col: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (j, s) in structure.iter().enumerate() {
        for &i in s {
            rows_in_col[i].push(j);
        }
    }
    let mut col_ptrs = Vec::with_capacity(n + 1);
    col_ptrs.push(0usize);
    let mut row_indices = Vec::new();
    for rows in rows_in_col {
        row_indices.extend(rows);
        col_ptrs.push(row_indices.len());
    }
    let values = vec![1.0; row_indices.len()];
    SparseTensor::new(n, n, col_ptrs, row_indices, values).map(Value::SparseTensor)
}

#[cfg(test)]
mod symbfact_tests {
    use runmat_value::{CellArray, SparseTensor, Value};

    use super::super::symbolic::test_support::*;

    fn symbfact(args: &[Value]) -> Value {
        assert_no_warnings(|| call("symbfact", args)).expect("symbfact")
    }

    fn symbfact5(args: &[Value]) -> Vec<Value> {
        let outputs = assert_no_warnings(|| call_outputs("symbfact", args, 5)).expect("symbfact");
        assert_eq!(outputs.len(), 5);
        outputs
    }

    /// (rows, cols, row-major 0/1 pattern, stored values) of a sparse double.
    fn sparse_pattern(value: &Value) -> (usize, usize, Vec<f64>, Vec<f64>) {
        let Value::SparseTensor(s) = value else {
            panic!("expected a sparse matrix, got {value:?}");
        };
        assert!(!s.is_logical() && !s.is_complex(), "expected sparse double");
        let values = s.as_f64_slice().expect("sparse double values").to_vec();
        let mut pattern = vec![0.0; s.rows * s.cols];
        for c in 0..s.cols {
            for idx in s.col_ptrs[c]..s.col_ptrs[c + 1] {
                pattern[s.row_indices[idx] * s.cols + c] = 1.0;
            }
        }
        (s.rows, s.cols, pattern, values)
    }

    fn scalar(value: &Value) -> f64 {
        let (shape, data) = double_array(value);
        assert_eq!(shape, vec![1, 1]);
        data[0]
    }

    // --- Observed (normative): one test per observed case --------------------

    /// case `tridiagonal` — symbfact.signature-primary, symbfact.output-class,
    /// symbfact.counts
    #[test]
    fn observed_tridiagonal() {
        assert_row(&symbfact(&[sparse(4, 4, &TRIDIAG)]), &[2.0, 2.0, 2.0, 1.0]);
    }

    /// case `arrow` — symbfact.counts (fill-in)
    #[test]
    fn observed_arrow() {
        assert_row(&symbfact(&[sparse(4, 4, &ARROW)]), &[4.0, 3.0, 2.0, 1.0]);
    }

    /// case `arrow-reversed` — symbfact.counts (no fill-in)
    #[test]
    fn observed_arrow_reversed() {
        assert_row(
            &symbfact(&[sparse(4, 4, &ARROW_REVERSED)]),
            &[2.0, 2.0, 2.0, 1.0],
        );
    }

    /// case `diag` — symbfact.counts
    #[test]
    fn observed_diag() {
        assert_row(&symbfact(&[speye(3)]), &[1.0, 1.0, 1.0]);
    }

    /// case `outputs` — symbfact.outputs, symbfact.counts
    #[test]
    fn observed_outputs() {
        let out = symbfact5(&[sparse(4, 4, &ARROW)]);
        assert_row(&out[0], &[4.0, 3.0, 2.0, 1.0]);
        assert_eq!(scalar(&out[1]), 4.0);
        assert_row(&out[2], &[2.0, 3.0, 4.0, 0.0]);
        assert_row(&out[3], &[1.0, 2.0, 3.0, 4.0]);
        let (rows, cols, pattern, values) = sparse_pattern(&out[4]);
        assert_eq!((rows, cols), (4, 4));
        assert_eq!(pattern, triangle(4, &[1.0; 16], true));
        assert!(values.iter().all(|&v| v == 1.0));
    }

    /// case `col` — symbfact.col
    #[test]
    fn observed_col() {
        assert_row(&symbfact(&[sparse(3, 2, &COL32), text("col")]), &[2.0, 1.0]);
    }

    /// case `dense-input` — symbfact.dense-input
    #[test]
    fn observed_dense_input() {
        assert_row(&symbfact(&[dense(4, 4, &TRIDIAG)]), &[2.0, 2.0, 2.0, 1.0]);
    }

    /// case `class` — symbfact.output-class, symbfact.counts
    #[test]
    fn observed_class() {
        assert_row(
            &symbfact(&[sparse(5, 5, &CORNER5)]),
            &[3.0, 3.0, 3.0, 2.0, 1.0],
        );
    }

    /// case `err-nonsquare` — symbfact.error-nonsquare
    #[test]
    fn observed_nonsquare_errors() {
        assert_eq!(
            identifier(call("symbfact", &[sparse(3, 2, &COL32)])),
            "RunMat:symbfact:InvalidInput"
        );
    }

    // --- Documented choices ---------------------------------------------------

    /// Checks all five outputs against the Boolean-elimination oracle for an
    /// n-node problem with the given off-diagonal edges.
    fn check_against_oracle(out: &[Value], n: usize, edges: &[(usize, usize)]) {
        let filled = filled_pattern(n, edges);
        let counts: Vec<f64> = (0..n)
            .map(|k| (k..n).filter(|&i| filled[i * n + k]).count() as f64)
            .collect();
        let parent = oracle_parent(n, &filled);
        assert_row(&out[0], &counts);
        assert_row(&out[2], &parent);
        assert_postorder(&parent, &assert_permutation(&out[3], n));
        // Height: nodes on the longest leaf-to-root path.
        let height = (0..n)
            .map(|v| {
                let (mut x, mut depth) = (v, 1);
                while parent[x] != 0.0 {
                    x = parent[x] as usize - 1;
                    depth += 1;
                }
                depth
            })
            .max()
            .unwrap_or(0) as f64;
        assert_eq!(scalar(&out[1]), height);
        // R = L': R(k, i) set iff filled(i, k), i >= k.
        let (rows, cols, pattern, values) = sparse_pattern(&out[4]);
        assert_eq!((rows, cols), (n, n));
        for k in 0..n {
            for i in 0..n {
                let expected = i >= k && filled[i * n + k];
                assert_eq!(pattern[k * n + i] == 1.0, expected, "R({k},{i})");
            }
        }
        assert!(values.iter().all(|&v| v == 1.0));
        // FR-042-11: count(j) is the number of nonzeros in row j of R.
        for (k, &count) in counts.iter().enumerate() {
            let row_nnz = pattern[k * n..(k + 1) * n].iter().sum::<f64>();
            assert_eq!(row_nnz, count);
        }
    }

    /// FR-042-11/12/19: random square patterns against the oracle; parent and
    /// post equal those of etree; dense equals sparse.
    #[test]
    fn documented_choice_random_patterns_match_oracle() {
        for (n, seed) in [(5, 31), (11, 32), (24, 33)] {
            let pattern = random_pattern(n, n, 15, seed);
            let out = symbfact5(&[sparse(n, n, &pattern)]);
            check_against_oracle(&out, n, &upper_edges(n, &pattern));
            let et = call_outputs("etree", &[sparse(n, n, &pattern)], 2).expect("etree");
            assert_eq!(out[2], et[0]);
            assert_eq!(out[3], et[1]);
            assert_eq!(out[0], symbfact(&[dense(n, n, &pattern)]));
        }
    }

    /// FR-042-14: 'col' on random rectangular patterns against the A'*A
    /// oracle.
    #[test]
    fn documented_choice_col_matches_ata_oracle() {
        for (rows, cols, seed) in [(3, 2, 40), (8, 5, 41), (6, 12, 42), (20, 15, 43)] {
            let pattern = random_pattern(rows, cols, 20, seed);
            let out = symbfact5(&[sparse(rows, cols, &pattern), text("col")]);
            check_against_oracle(&out, cols, &ata_edges(rows, cols, &pattern));
        }
    }

    /// FR-042-19: h is 1 for a diagonal matrix; a 0x0 input gives empty rows,
    /// h = 0 and a 0x0 sparse R.
    #[test]
    fn documented_choice_height_and_empty() {
        let out = symbfact5(&[speye(3)]);
        assert_eq!(scalar(&out[1]), 1.0);
        let out = symbfact5(&[Value::SparseTensor(SparseTensor::zeros(0, 0))]);
        assert_row(&out[0], &[]);
        assert_eq!(scalar(&out[1]), 0.0);
        assert_row(&out[2], &[]);
        assert_row(&out[3], &[]);
        let (rows, cols, _, _) = sparse_pattern(&out[4]);
        assert_eq!((rows, cols), (0, 0));
    }

    /// FR-042-21: fewer outputs return the leading ones; more than five raise
    /// TooManyOutputs.
    #[test]
    fn documented_choice_output_counts() {
        let two = call_outputs("symbfact", &[sparse(4, 4, &ARROW)], 2).expect("symbfact");
        assert_eq!(two.len(), 2);
        assert_row(&two[0], &[4.0, 3.0, 2.0, 1.0]);
        assert_eq!(scalar(&two[1]), 4.0);
        let result = call_direct("symbfact", &[sparse(4, 4, &ARROW)], 6);
        assert_eq!(identifier(result), "RunMat:symbfact:TooManyOutputs");
    }

    /// FR-042-18 (option text forms) and FR-042-20 (input classes).
    #[test]
    fn documented_choice_option_text_and_inputs() {
        assert_row(
            &symbfact(&[sparse(3, 2, &COL32), Value::String("COL".into())]),
            &[2.0, 1.0],
        );
        assert_row(&symbfact(&[Value::Num(4.0)]), &[1.0]);
        let cell = Value::Cell(CellArray::new(vec![Value::Num(1.0)], 1, 1).unwrap());
        assert_eq!(
            identifier(call("symbfact", &[cell])),
            "RunMat:symbfact:InvalidInput"
        );
    }

    // --- Unresolved (symbfact.q-more-options) ---------------------------------

    /// FR-042-18: options other than 'col' (including 'sym') and extra
    /// arguments are unsupported.
    #[test]
    fn unresolved_other_options_unsupported() {
        let a = sparse(4, 4, &TRIDIAG);
        for args in [
            vec![a.clone(), text("sym")],
            vec![a.clone(), text("lo")],
            vec![a.clone(), Value::Num(1.0)],
            vec![a.clone(), text("col"), text("lower")],
        ] {
            assert_eq!(
                identifier(call("symbfact", &args)),
                "RunMat:symbfact:UnsupportedOption",
                "{args:?}"
            );
        }
    }
}
