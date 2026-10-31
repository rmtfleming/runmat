//! Clean-room `sprank` (structural rank).
//!
//! Clean-room feature `specs/061-sparse-linalg-n5/`. Normative behaviour
//! comes only from the approved export `sprank` 0.2.0 (see that feature's
//! `provenance.md`): the result depends only on the nonzero pattern and
//! equals, for every observed case, the largest number of nonzeros with no
//! two in the same row or column; zero and empty matrices give 0; full,
//! sparse logical and sparse complex inputs are accepted; the result is a
//! double scalar; and the error identifiers (raised as `RunMat:*`, D-061-1).
//!
//! Algorithm: the size of a maximum matching of the bipartite column/row
//! graph of the pattern, computed with the Hopcroft–Karp algorithm
//! (Hopcroft & Karp, SIAM J. Comput. 2(4), 1973), with an iterative
//! depth-first search so that large COBRA matrices do not exhaust the stack.

use runmat_builtins::{
    BuiltinCompletionPolicy, BuiltinDescriptor, BuiltinErrorDescriptor, BuiltinOutputMode,
    BuiltinParamArity, BuiltinParamDescriptor, BuiltinParamType, BuiltinSignatureDescriptor,
};
use runmat_macros::runtime_builtin;
use runmat_value::{NumericDType, Value};

use crate::builtins::common::map_control_flow_with_builtin;
use crate::{build_runtime_error, gather_if_needed_async, BuiltinResult, RuntimeError};

const NAME: &str = "sprank";

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
    "Matrix (sparse or full; double, single, logical or complex).",
);
const OUT_R: BuiltinParamDescriptor = param(
    "r",
    BuiltinParamArity::Required,
    "Structural rank (double scalar).",
);

const SPRANK_INPUTS: [BuiltinParamDescriptor; 1] = [IN_A];
const SPRANK_OUTPUTS: [BuiltinParamDescriptor; 1] = [OUT_R];

const SPRANK_SIGNATURES: [BuiltinSignatureDescriptor; 1] = [BuiltinSignatureDescriptor {
    label: "r = sprank(A)",
    inputs: &SPRANK_INPUTS,
    outputs: &SPRANK_OUTPUTS,
}];

const MUST_BE_FLOAT_OR_LOGICAL: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SPRANK.MUST_BE_FLOAT_OR_LOGICAL",
    identifier: Some("RunMat:math:mustBeFloatOrLogical"),
    when: "A is char or a cell array.",
    message: "sprank: A must be a floating-point or logical matrix",
};

const MINRHS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SPRANK.MINRHS",
    identifier: Some("RunMat:minrhs"),
    when: "No input is given.",
    message: "sprank: not enough input arguments",
};

const INVALID_INPUT: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SPRANK.INVALID_INPUT",
    identifier: Some("RunMat:sprank:InvalidInput"),
    when: "A is not 2-D.",
    message: "sprank: A must be a 2-D matrix",
};

const UNSUPPORTED_OPTION: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SPRANK.UNSUPPORTED_OPTION",
    identifier: Some("RunMat:sprank:UnsupportedOption"),
    when: "Unobserved inputs: extra arguments, integer, string and other classes.",
    message: "sprank: this form is not yet specified",
};

const TOO_MANY_OUTPUTS: BuiltinErrorDescriptor = BuiltinErrorDescriptor {
    code: "RM.SPRANK.TOO_MANY_OUTPUTS",
    identifier: Some("RunMat:sprank:TooManyOutputs"),
    when: "More than one output was requested.",
    message: "sprank: too many output arguments",
};

const SPRANK_ERRORS: [BuiltinErrorDescriptor; 5] = [
    MUST_BE_FLOAT_OR_LOGICAL,
    MINRHS,
    INVALID_INPUT,
    UNSUPPORTED_OPTION,
    TOO_MANY_OUTPUTS,
];

pub const SPRANK_DESCRIPTOR: BuiltinDescriptor = BuiltinDescriptor {
    signatures: &SPRANK_SIGNATURES,
    output_mode: BuiltinOutputMode::Fixed,
    completion_policy: BuiltinCompletionPolicy::Public,
    errors: &SPRANK_ERRORS,
};

fn sprank_error(
    descriptor: &'static BuiltinErrorDescriptor,
    detail: impl Into<String>,
) -> RuntimeError {
    let mut builder = build_runtime_error(format!("{NAME}: {}", detail.into())).with_builtin(NAME);
    if let Some(identifier) = descriptor.identifier {
        builder = builder.with_identifier(identifier);
    }
    builder.build()
}

#[runtime_builtin(
    name = "sprank",
    category = "math/sparse",
    summary = "Structural rank of a matrix (size of a maximum matching of its pattern).",
    keywords = "sprank,structural rank,sparse,matching,dmperm",
    accel = "cpu",
    descriptor(crate::builtins::math::sparse::sprank::SPRANK_DESCRIPTOR),
    builtin_path = "crate::builtins::math::sparse::sprank"
)]
pub(crate) async fn sprank_builtin(args: Vec<Value>) -> BuiltinResult<Value> {
    let out_count = crate::output_count::current_output_count();
    // FR-061-27
    if out_count.is_some_and(|count| count > 1) {
        return Err(sprank_error(
            &TOO_MANY_OUTPUTS,
            "only one output is supported",
        ));
    }
    // FR-061-25 (sprank.error-no-arg)
    if args.is_empty() {
        return Err(sprank_error(&MINRHS, "expected sprank(A)"));
    }
    if args.len() > 1 {
        return Err(sprank_error(
            &UNSUPPORTED_OPTION,
            "arguments after A are not yet specified",
        ));
    }
    // FR-061-28: gpuArray input is gathered.
    let a = gather_if_needed_async(&args[0])
        .await
        .map_err(|err| map_control_flow_with_builtin(err, NAME))?;
    let (rows, _cols, columns) = pattern(&a)?;
    let r = Value::Num(maximum_matching(rows, &columns) as f64);
    match out_count {
        None => Ok(r),
        Some(0) => Ok(Value::OutputList(Vec::new())),
        Some(_) => Ok(Value::OutputList(vec![r])),
    }
}

/// Row indices of the stored nonzeros of each column.
type Columns = Vec<Vec<usize>>;

fn dims(shape: &[usize]) -> BuiltinResult<(usize, usize)> {
    if shape.iter().skip(2).any(|&dim| dim != 1) {
        return Err(sprank_error(&INVALID_INPUT, "A must be a 2-D matrix"));
    }
    Ok((
        shape.first().copied().unwrap_or(1),
        shape.get(1).copied().unwrap_or(1),
    ))
}

/// Column lists of a column-major dense array, with `nonzero(index)` telling
/// whether an element counts (NaN counts, FR-061-26).
fn dense_columns(rows: usize, cols: usize, nonzero: impl Fn(usize) -> bool) -> Columns {
    (0..cols)
        .map(|c| (0..rows).filter(|&r| nonzero(r + c * rows)).collect())
        .collect()
}

fn unsupported(detail: &str) -> RuntimeError {
    sprank_error(&UNSUPPORTED_OPTION, detail)
}

/// FR-061-19, FR-061-22, FR-061-24, FR-061-26: the nonzero pattern of `A`.
fn pattern(a: &Value) -> BuiltinResult<(usize, usize, Columns)> {
    match a {
        Value::SparseTensor(s) => {
            if s.integer_storage().is_some() {
                return Err(unsupported("integer A is not yet specified"));
            }
            let columns = (0..s.cols)
                .map(|c| s.row_indices[s.col_ptrs[c]..s.col_ptrs[c + 1]].to_vec())
                .collect();
            Ok((s.rows, s.cols, columns))
        }
        Value::Tensor(t) => {
            if t.integer_storage().is_some()
                || !matches!(t.numeric_dtype(), NumericDType::F64 | NumericDType::F32)
            {
                return Err(unsupported("integer A is not yet specified"));
            }
            let (rows, cols) = dims(&t.shape)?;
            let data = t.materialize_f64();
            Ok((rows, cols, dense_columns(rows, cols, |i| data[i] != 0.0)))
        }
        Value::ComplexTensor(t) => {
            if t.integer_storage().is_some() {
                return Err(unsupported("integer A is not yet specified"));
            }
            let (rows, cols) = dims(&t.shape)?;
            let data = t.materialize_f64();
            Ok((
                rows,
                cols,
                dense_columns(rows, cols, |i| data[i].0 != 0.0 || data[i].1 != 0.0),
            ))
        }
        Value::LogicalArray(l) => {
            let (rows, cols) = dims(&l.shape)?;
            Ok((rows, cols, dense_columns(rows, cols, |i| l.data[i] != 0)))
        }
        Value::Num(v) => Ok((1, 1, vec![if *v != 0.0 { vec![0] } else { vec![] }])),
        Value::Bool(b) => Ok((1, 1, vec![if *b { vec![0] } else { vec![] }])),
        Value::Complex(re, im) => Ok((
            1,
            1,
            vec![if *re != 0.0 || *im != 0.0 {
                vec![0]
            } else {
                vec![]
            }],
        )),
        // FR-061-24 (sprank.error-char)
        Value::CharArray(_) | Value::Cell(_) => Err(sprank_error(
            &MUST_BE_FLOAT_OR_LOGICAL,
            "A must be a floating-point or logical matrix",
        )),
        _ => Err(unsupported(
            "A must be a double, single, logical or complex matrix",
        )),
    }
}

/// Size of a maximum matching between columns (left) and rows (right),
/// Hopcroft–Karp with an iterative DFS.
fn maximum_matching(rows: usize, columns: &Columns) -> usize {
    const NONE: usize = usize::MAX;
    const INF: usize = usize::MAX;
    let cols = columns.len();
    let mut match_col = vec![NONE; cols];
    let mut match_row = vec![NONE; rows];
    let mut dist = vec![INF; cols];
    let mut size = 0usize;
    loop {
        // BFS layering from the free columns.
        let mut queue = Vec::with_capacity(cols);
        for c in 0..cols {
            if match_col[c] == NONE {
                dist[c] = 0;
                queue.push(c);
            } else {
                dist[c] = INF;
            }
        }
        let mut found = false;
        let mut head = 0;
        while head < queue.len() {
            let c = queue[head];
            head += 1;
            for &r in &columns[c] {
                let next = match_row[r];
                if next == NONE {
                    found = true;
                } else if dist[next] == INF {
                    dist[next] = dist[c] + 1;
                    queue.push(next);
                }
            }
        }
        if !found {
            break;
        }
        // DFS along the layers, one augmenting path per free column.
        let mut cursor = vec![0usize; cols];
        for start in 0..cols {
            if match_col[start] != NONE {
                continue;
            }
            let mut stack = vec![start];
            while let Some(&c) = stack.last() {
                if cursor[c] < columns[c].len() {
                    let r = columns[c][cursor[c]];
                    cursor[c] += 1;
                    let next = match_row[r];
                    if next == NONE {
                        // Augment: each column on the stack takes the row
                        // that leads to its successor.
                        let mut row = r;
                        while let Some(col) = stack.pop() {
                            let previous = match_col[col];
                            match_col[col] = row;
                            match_row[row] = col;
                            row = previous;
                        }
                        size += 1;
                        break;
                    } else if dist[next] != INF && dist[next] == dist[c] + 1 {
                        stack.push(next);
                    }
                } else {
                    dist[c] = INF;
                    stack.pop();
                }
            }
        }
    }
    size
}

#[cfg(test)]
mod sprank_tests {
    use super::{maximum_matching, sprank_builtin};
    use crate::BuiltinResult;
    use futures::executor::block_on;
    use runmat_value::{
        CellArray, CharArray, ComplexTensor, IntegerStorage, LogicalArray, SparseTensor, Tensor,
        Value,
    };

    fn call(args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async("sprank", args))
    }

    fn sprank(a: Value) -> f64 {
        match call(&[a]).unwrap() {
            Value::Num(v) => v,
            other => panic!("expected a double scalar, got {other:?}"),
        }
    }

    /// Sparse matrix from a row-major dense literal.
    fn sparse(rows: usize, cols: usize, row_major: &[f64]) -> Value {
        let mut col_ptrs = vec![0];
        let mut row_idx = Vec::new();
        let mut values = Vec::new();
        for c in 0..cols {
            for r in 0..rows {
                let v = row_major[r * cols + c];
                if v != 0.0 {
                    row_idx.push(r);
                    values.push(v);
                }
            }
            col_ptrs.push(row_idx.len());
        }
        Value::SparseTensor(SparseTensor::new(rows, cols, col_ptrs, row_idx, values).unwrap())
    }

    fn identifier(result: BuiltinResult<Value>) -> String {
        match result {
            Ok(value) => panic!("expected an error, got {value:?}"),
            Err(err) => err.identifier().unwrap_or("<none>").to_string(),
        }
    }

    const EYE3: [f64; 9] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];

    // --- Observed (normative): one test per observed case --------------------

    /// case `identity` — sprank.signature-primary, sprank.matching, sprank.class
    #[test]
    fn observed_identity() {
        assert_eq!(sprank(sparse(3, 3, &EYE3)), 3.0);
    }

    /// case `cancelling` — sprank.structural: sprank 2 while rank is 1.
    #[test]
    fn observed_cancelling() {
        let a = sparse(2, 2, &[1.0; 4]);
        assert_eq!(sprank(a.clone()), 2.0);
        let full = block_on(crate::call_builtin_async("full", &[a])).unwrap();
        match block_on(crate::call_builtin_async("rank", &[full])).unwrap() {
            Value::Num(v) => assert_eq!(v, 1.0),
            other => panic!("unexpected rank {other:?}"),
        }
    }

    /// case `zero-row` — sprank.matching
    #[test]
    fn observed_zero_row() {
        let a = sparse(3, 3, &[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]);
        assert_eq!(sprank(a), 2.0);
    }

    /// case `matching-limited` — sprank.matching
    #[test]
    fn observed_matching_limited() {
        let a = sparse(3, 3, &[1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0]);
        assert_eq!(sprank(a), 2.0);
    }

    /// case `wide` — sprank.matching
    #[test]
    fn observed_wide() {
        assert_eq!(sprank(sparse(2, 3, &[1.0, 1.0, 0.0, 0.0, 1.0, 1.0])), 2.0);
    }

    /// case `tall` — sprank.matching
    #[test]
    fn observed_tall() {
        let a = sparse(4, 2, &[1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0]);
        assert_eq!(sprank(a), 2.0);
    }

    /// case `zero-matrix` — sprank.zero-empty
    #[test]
    fn observed_zero_matrix() {
        assert_eq!(sprank(Value::SparseTensor(SparseTensor::zeros(3, 3))), 0.0);
    }

    /// case `empty` — sprank.zero-empty
    #[test]
    fn observed_empty() {
        assert_eq!(sprank(Value::SparseTensor(SparseTensor::zeros(0, 0))), 0.0);
    }

    /// case `dense` — sprank.inputs, sprank.class
    #[test]
    fn observed_dense() {
        let a = Value::Tensor(Tensor::new(vec![1.0, 0.0, 0.0, 1.0], vec![2, 2]).unwrap());
        assert_eq!(sprank(a), 2.0);
    }

    /// case `logical` — sprank.inputs (sparse logical)
    #[test]
    fn observed_logical() {
        let s = SparseTensor::new_logical(2, 2, vec![0, 1, 2], vec![0, 1]).unwrap();
        assert_eq!(sprank(Value::SparseTensor(s)), 2.0);
    }

    /// case `complex` — sprank.inputs (sparse complex)
    #[test]
    fn observed_complex() {
        let s = SparseTensor::new_complex(2, 2, vec![0, 1, 2], vec![0, 1], vec![(0.0, 1.0); 2])
            .unwrap();
        assert!(s.is_complex());
        assert_eq!(sprank(Value::SparseTensor(s)), 2.0);
    }

    /// case `err-char` — sprank.error-char
    #[test]
    fn observed_err_char() {
        assert_eq!(
            identifier(call(&[Value::CharArray(CharArray::new_row("ab"))])),
            "RunMat:math:mustBeFloatOrLogical"
        );
    }

    /// case `err-cell` — sprank.error-char
    #[test]
    fn observed_err_cell() {
        let cell = Value::Cell(CellArray::new(vec![Value::Num(1.0)], 1, 1).unwrap());
        assert_eq!(
            identifier(call(&[cell])),
            "RunMat:math:mustBeFloatOrLogical"
        );
    }

    /// case `err-no-arg` — sprank.error-no-arg
    #[test]
    fn observed_err_no_arg() {
        assert_eq!(identifier(call(&[])), "RunMat:minrhs");
    }

    // --- Documented choices and unresolved points ----------------------------

    /// FR-061-26: full logical, complex, single and scalar inputs; NaN counts
    /// as a nonzero; trailing singleton dimensions are 2-D.
    #[test]
    fn documented_choice_input_forms() {
        let logical = LogicalArray::new(vec![1, 1, 0, 0], vec![2, 2]).unwrap();
        assert_eq!(sprank(Value::LogicalArray(logical)), 1.0);
        let z = ComplexTensor::new(
            vec![(0.0, 1.0), (0.0, 0.0), (0.0, 0.0), (2.0, 0.0)],
            vec![2, 2],
        )
        .unwrap();
        assert_eq!(sprank(Value::ComplexTensor(z)), 2.0);
        let single = Tensor::from_f32(vec![0.0, 1.0, 1.0, 0.0], vec![2, 2]).unwrap();
        assert_eq!(sprank(Value::Tensor(single)), 2.0);
        assert_eq!(sprank(Value::Num(0.0)), 0.0);
        assert_eq!(sprank(Value::Num(5.0)), 1.0);
        assert_eq!(sprank(Value::Bool(true)), 1.0);
        assert_eq!(sprank(Value::Complex(0.0, 2.0)), 1.0);
        assert_eq!(sprank(Value::Num(f64::NAN)), 1.0);
        let trailing = Tensor::new(vec![1.0, 0.0, 0.0, 1.0], vec![2, 2, 1]).unwrap();
        assert_eq!(sprank(Value::Tensor(trailing)), 2.0);
        assert_eq!(sprank(Value::SparseTensor(SparseTensor::zeros(0, 5))), 0.0);
    }

    /// FR-061-27: unobserved inputs and output counts.
    #[test]
    fn documented_choice_unsupported_and_outputs() {
        const U: &str = "RunMat:sprank:UnsupportedOption";
        let int = Tensor::new_integer(IntegerStorage::I8(vec![1, 0, 0, 1]), vec![2, 2]).unwrap();
        assert_eq!(identifier(call(&[Value::Tensor(int)])), U);
        assert_eq!(identifier(call(&[Value::String("ab".into())])), U);
        assert_eq!(identifier(call(&[Value::Num(1.0), Value::Num(1.0)])), U);
        let nd = Tensor::new(vec![1.0; 8], vec![2, 2, 2]).unwrap();
        assert_eq!(
            identifier(call(&[Value::Tensor(nd)])),
            "RunMat:sprank:InvalidInput"
        );
        let _guard = crate::output_count::push_output_count(Some(2));
        let err = block_on(sprank_builtin(vec![Value::Num(1.0)])).unwrap_err();
        assert_eq!(err.identifier(), Some("RunMat:sprank:TooManyOutputs"));
    }

    /// The matching is maximum on patterns where a greedy choice fails, and
    /// on a large banded pattern (iterative DFS, no recursion).
    #[test]
    fn documented_choice_matching_algorithm() {
        // Greedy would match column 0 to row 0 and then strand column 1.
        let columns = vec![vec![0, 1], vec![0], vec![1, 2]];
        assert_eq!(maximum_matching(3, &columns), 3);
        // Lower-bidiagonal 200000 x 200000 pattern: structural rank n.
        let n = 200_000;
        let columns: Vec<Vec<usize>> = (0..n)
            .map(|c| if c + 1 < n { vec![c + 1, c] } else { vec![c] })
            .collect();
        assert_eq!(maximum_matching(n, &columns), n);
        // A row of ones over a column of ones has structural rank 1.
        assert_eq!(maximum_matching(1, &vec![vec![0]; 4]), 1);
    }

    /// `sprank.q-warning-coverage` (unresolved): no warning is emitted.
    #[test]
    fn unresolved_no_warning() {
        assert!(call(&[Value::Num(2.0)]).is_ok());
    }
}
