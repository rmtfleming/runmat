//! Shared symbolic-analysis helpers for the clean-room sparse builtins `amd`,
//! `etree` and `symbfact` (feature `specs/042-sparse-orderings/`).
//!
//! All three builtins work on the sparsity pattern only. Algorithms are
//! textbook ones, cited in `specs/042-sparse-orderings/provenance.md`:
//! Liu's elimination tree with path compression (and the column elimination
//! tree of `A'*A` without forming `A'*A`), and a depth-first postorder.

use runmat_value::{Tensor, Value};

use crate::builtins::common::{gpu_helpers, tensor};
use crate::{build_runtime_error, BuiltinResult, RuntimeError};

/// Sparsity pattern of a 2-D matrix: for each column, the sorted zero-based
/// row indices of its structural nonzeros.
pub(crate) struct Pattern {
    pub rows: usize,
    pub cols: usize,
    pub columns: Vec<Vec<usize>>,
}

/// Build a `RunMat:` error for one of the sparse symbolic builtins.
pub(crate) fn symbolic_error(name: &str, identifier: &str, detail: &str) -> RuntimeError {
    build_runtime_error(format!("{name}: {detail}"))
        .with_builtin(name)
        .with_identifier(identifier)
        .build()
}

/// Gather a possibly GPU-resident value to the host.
pub(crate) async fn gather(value: &Value) -> BuiltinResult<Value> {
    gpu_helpers::gather_value_async(value).await
}

/// 2-D (rows, cols) of a shape; trailing singleton dimensions are allowed.
fn dims(shape: &[usize]) -> Result<(usize, usize), String> {
    if shape.len() > 2 && shape[2..].iter().product::<usize>() != 1 {
        return Err("input must be a 2-D matrix (N-D arrays are not supported)".to_string());
    }
    Ok((
        shape.first().copied().unwrap_or(1),
        shape.get(1).copied().unwrap_or(1),
    ))
}

/// Pattern from dense column-major data and an "is nonzero" predicate.
fn dense_pattern(rows: usize, cols: usize, nonzero: impl Fn(usize) -> bool) -> Pattern {
    let columns = (0..cols)
        .map(|c| (0..rows).filter(|&r| nonzero(r + c * rows)).collect())
        .collect();
    Pattern {
        rows,
        cols,
        columns,
    }
}

/// Read the sparsity pattern of a matrix value (FR-042-20). Sparse values
/// (any storage) contribute their stored entries; dense numeric, logical and
/// complex arrays contribute their nonzero elements (`NaN` is nonzero);
/// numeric, logical and complex scalars are 1×1. Anything else, and N-D
/// arrays, give an error message for the caller to wrap.
pub(crate) fn read_pattern(value: &Value) -> Result<Pattern, String> {
    match value {
        Value::SparseTensor(s) => {
            let columns = (0..s.cols)
                .map(|c| {
                    (s.col_ptrs[c]..s.col_ptrs[c + 1])
                        .map(|k| s.row_indices[k])
                        .collect()
                })
                .collect();
            Ok(Pattern {
                rows: s.rows,
                cols: s.cols,
                columns,
            })
        }
        Value::Tensor(t) => {
            let (rows, cols) = dims(&t.shape)?;
            let data = t.materialize_f64();
            Ok(dense_pattern(rows, cols, |i| data[i] != 0.0))
        }
        Value::LogicalArray(l) => {
            let (rows, cols) = dims(&l.shape)?;
            let data = l.data.as_slice();
            Ok(dense_pattern(rows, cols, |i| data[i] != 0))
        }
        Value::ComplexTensor(t) => {
            let (rows, cols) = dims(&t.shape)?;
            let data = t.materialize_f64();
            Ok(dense_pattern(rows, cols, |i| {
                data[i].0 != 0.0 || data[i].1 != 0.0
            }))
        }
        Value::Num(x) => Ok(dense_pattern(1, 1, |_| *x != 0.0)),
        Value::Int(i) => Ok(dense_pattern(1, 1, |_| i.to_f64() != 0.0)),
        Value::Bool(b) => Ok(dense_pattern(1, 1, |_| *b)),
        Value::Complex(re, im) => Ok(dense_pattern(1, 1, |_| *re != 0.0 || *im != 0.0)),
        other => Err(format!(
            "input must be a 2-D numeric or logical matrix (full or sparse), got {}",
            crate::builtins::introspection::class::class_name_for_value(other)
        )),
    }
}

/// For each column `k`, the rows `i < k` stored in the upper triangle
/// (FR-042-07: only the upper triangle is used).
pub(crate) fn upper_columns(pattern: &Pattern) -> Vec<Vec<usize>> {
    pattern
        .columns
        .iter()
        .enumerate()
        .map(|(k, rows)| rows.iter().copied().filter(|&i| i < k).collect())
        .collect()
}

/// Walk from `start` to the root of its current subtree, compressing the path
/// onto `k` and linking the old root under `k` (Liu 1990).
fn link(start: usize, k: usize, ancestor: &mut [Option<usize>], parent: &mut [Option<usize>]) {
    let mut node = Some(start);
    while let Some(i) = node {
        if i >= k {
            break;
        }
        let next = ancestor[i];
        ancestor[i] = Some(k);
        if next.is_none() {
            parent[i] = Some(k);
        }
        node = next;
    }
}

/// Elimination tree of the symmetric pattern given by its upper triangle.
pub(crate) fn etree_upper(upper: &[Vec<usize>]) -> Vec<Option<usize>> {
    let n = upper.len();
    let mut parent = vec![None; n];
    let mut ancestor = vec![None; n];
    for (k, rows) in upper.iter().enumerate() {
        for &i in rows {
            link(i, k, &mut ancestor, &mut parent);
        }
    }
    parent
}

/// Column elimination tree: the elimination tree of `A'*A`, computed without
/// forming `A'*A` by linking each column to the previous column that shares
/// a row (Davis 2006).
pub(crate) fn etree_col(pattern: &Pattern) -> Vec<Option<usize>> {
    let n = pattern.cols;
    let mut parent = vec![None; n];
    let mut ancestor = vec![None; n];
    let mut prev: Vec<Option<usize>> = vec![None; pattern.rows];
    for (k, rows) in pattern.columns.iter().enumerate() {
        for &r in rows {
            if let Some(i) = prev[r] {
                link(i, k, &mut ancestor, &mut parent);
            }
            prev[r] = Some(k);
        }
    }
    parent
}

/// Depth-first postorder of a forest (FR-042-17, documented choice): roots,
/// and the children of each node, are visited in increasing index order.
pub(crate) fn postorder(parent: &[Option<usize>]) -> Vec<usize> {
    let n = parent.len();
    let children = children_of(parent);
    let mut order = Vec::with_capacity(n);
    let mut stack: Vec<(usize, usize)> = Vec::new();
    for root in (0..n).filter(|&j| parent[j].is_none()) {
        stack.push((root, 0));
        while let Some((node, next)) = stack.pop() {
            if let Some(&child) = children[node].get(next) {
                stack.push((node, next + 1));
                stack.push((child, 0));
            } else {
                order.push(node);
            }
        }
    }
    order
}

/// Children of every node, in increasing index order.
pub(crate) fn children_of(parent: &[Option<usize>]) -> Vec<Vec<usize>> {
    let mut children = vec![Vec::new(); parent.len()];
    for (j, p) in parent.iter().enumerate() {
        if let Some(p) = p {
            children[*p].push(j);
        }
    }
    children
}

/// Height of the forest: nodes on the longest leaf-to-root path (FR-042-19).
/// Parents always have larger indices than their children.
pub(crate) fn height(parent: &[Option<usize>]) -> usize {
    let n = parent.len();
    let mut depth = vec![0usize; n];
    for j in (0..n).rev() {
        depth[j] = parent[j].map_or(1, |p| depth[p] + 1);
    }
    depth.into_iter().max().unwrap_or(0)
}

/// One-based parent vector, `0` for roots.
pub(crate) fn parent_values(parent: &[Option<usize>]) -> Vec<f64> {
    parent
        .iter()
        .map(|p| p.map_or(0.0, |p| (p + 1) as f64))
        .collect()
}

/// One-based values of zero-based indices.
pub(crate) fn one_based(indices: &[usize]) -> Vec<f64> {
    indices.iter().map(|&i| (i + 1) as f64).collect()
}

/// A 1×n double row vector (a 1×1 result is a scalar double).
pub(crate) fn row_vector(values: Vec<f64>) -> Result<Value, String> {
    let n = values.len();
    Tensor::new(values, vec![1, n]).map(tensor::tensor_into_value)
}

/// Option text: a char row or a string scalar, lower-cased.
pub(crate) fn option_text(value: &Value) -> Option<String> {
    match value {
        Value::CharArray(chars) if chars.rows <= 1 => {
            Some(chars.data.iter().collect::<String>().to_lowercase())
        }
        Value::String(text) => Some(text.to_lowercase()),
        Value::StringArray(array) if array.data.len() == 1 => Some(array.data[0].to_lowercase()),
        _ => None,
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    //! Test-only helpers shared by `amd_tests`, `etree_tests` and
    //! `symbfact_tests`.

    use futures::executor::block_on;
    use runmat_value::{CharArray, NumericDType, SparseTensor, Tensor, Value};

    use crate::BuiltinResult;

    /// `sparse([2 1 0 0; 1 2 1 0; 0 1 2 1; 0 0 1 2])` (row-major literal).
    pub const TRIDIAG: [f64; 16] = [
        2.0, 1.0, 0.0, 0.0, 1.0, 2.0, 1.0, 0.0, 0.0, 1.0, 2.0, 1.0, 0.0, 0.0, 1.0, 2.0,
    ];
    /// `sparse([4 1 1 1; 1 4 0 0; 1 0 4 0; 1 0 0 4])`.
    pub const ARROW: [f64; 16] = [
        4.0, 1.0, 1.0, 1.0, 1.0, 4.0, 0.0, 0.0, 1.0, 0.0, 4.0, 0.0, 1.0, 0.0, 0.0, 4.0,
    ];
    /// `sparse([4 0 0 1; 0 4 0 1; 0 0 4 1; 1 1 1 4])`.
    pub const ARROW_REVERSED: [f64; 16] = [
        4.0, 0.0, 0.0, 1.0, 0.0, 4.0, 0.0, 1.0, 0.0, 0.0, 4.0, 1.0, 1.0, 1.0, 1.0, 4.0,
    ];
    /// `sparse([4 1 0 0 1; 1 4 1 0 0; 0 1 4 1 0; 0 0 1 4 1; 1 0 0 1 4])`.
    pub const CORNER5: [f64; 25] = [
        4.0, 1.0, 0.0, 0.0, 1.0, 1.0, 4.0, 1.0, 0.0, 0.0, 0.0, 1.0, 4.0, 1.0, 0.0, 0.0, 0.0, 1.0,
        4.0, 1.0, 1.0, 0.0, 0.0, 1.0, 4.0,
    ];
    /// `sparse([1 0 1 0; 0 1 1 0; 1 1 1 1; 0 0 1 1])`.
    pub const BRANCHING: [f64; 16] = [
        1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0,
    ];
    /// `sparse([1 0; 1 1; 0 1])` (3×2).
    pub const COL32: [f64; 6] = [1.0, 0.0, 1.0, 1.0, 0.0, 1.0];

    /// Column-major copy of a row-major literal.
    fn column_major(rows: usize, cols: usize, row_major: &[f64]) -> Vec<f64> {
        assert_eq!(row_major.len(), rows * cols);
        let mut out = vec![0.0; rows * cols];
        for r in 0..rows {
            for c in 0..cols {
                out[r + c * rows] = row_major[r * cols + c];
            }
        }
        out
    }

    /// A sparse double matrix from a row-major literal (zeros not stored).
    pub fn sparse(rows: usize, cols: usize, row_major: &[f64]) -> Value {
        let data = column_major(rows, cols, row_major);
        let mut col_ptrs = vec![0usize];
        let mut row_indices = Vec::new();
        let mut values = Vec::new();
        for c in 0..cols {
            for r in 0..rows {
                let v = data[r + c * rows];
                if v != 0.0 {
                    row_indices.push(r);
                    values.push(v);
                }
            }
            col_ptrs.push(row_indices.len());
        }
        Value::SparseTensor(SparseTensor::new(rows, cols, col_ptrs, row_indices, values).unwrap())
    }

    /// A full double matrix from a row-major literal.
    pub fn dense(rows: usize, cols: usize, row_major: &[f64]) -> Value {
        Value::Tensor(Tensor::new(column_major(rows, cols, row_major), vec![rows, cols]).unwrap())
    }

    /// `speye(n)`.
    pub fn speye(n: usize) -> Value {
        let mut eye = vec![0.0; n * n];
        for i in 0..n {
            eye[i * n + i] = 1.0;
        }
        sparse(n, n, &eye)
    }

    /// Keep only the upper (`upper = true`) or lower triangle of a row-major
    /// square literal, diagonal included.
    pub fn triangle(n: usize, row_major: &[f64], upper: bool) -> Vec<f64> {
        let mut out = row_major.to_vec();
        for r in 0..n {
            for c in 0..n {
                if (upper && r > c) || (!upper && r < c) {
                    out[r * n + c] = 0.0;
                }
            }
        }
        out
    }

    pub fn text(s: &str) -> Value {
        Value::CharArray(CharArray::new_row(s))
    }

    pub fn call(name: &str, args: &[Value]) -> BuiltinResult<Value> {
        block_on(crate::call_builtin_async(name, args))
    }

    pub fn call_outputs(name: &str, args: &[Value], outputs: usize) -> BuiltinResult<Vec<Value>> {
        match block_on(crate::call_builtin_async_with_outputs(name, args, outputs))? {
            Value::OutputList(list) => Ok(list),
            other => Ok(vec![other]),
        }
    }

    /// Call the registered implementation directly with a requested output
    /// count. Used for too-many-outputs checks, because the dispatcher
    /// re-invokes a failed builtin without the output count (see
    /// specs/039-svds/validation.md, deviations).
    pub fn call_direct(name: &str, args: &[Value], outputs: usize) -> BuiltinResult<Value> {
        let builtin = runmat_builtins::builtin_function_by_name(name)
            .unwrap_or_else(|| panic!("UndefinedFunction: {name} is not registered"));
        let _guard = crate::output_count::push_output_count(Some(outputs));
        block_on((builtin.implementation)(args))
    }

    pub fn identifier(result: BuiltinResult<Value>) -> String {
        match result {
            Ok(value) => panic!("expected an error, got {value:?}"),
            Err(err) => err.identifier().unwrap_or("<none>").to_string(),
        }
    }

    /// (shape, values) of a dense class-double result; panics on anything
    /// else (sparse, single, integer, logical).
    pub fn double_array(value: &Value) -> (Vec<usize>, Vec<f64>) {
        match value {
            Value::Num(n) => (vec![1, 1], vec![*n]),
            Value::Tensor(t) => {
                assert!(t.integer_storage().is_none(), "expected double storage");
                assert_eq!(
                    t.numeric_dtype(),
                    NumericDType::F64,
                    "expected class double"
                );
                (t.shape.clone(), t.materialize_f64())
            }
            other => panic!("expected a full double array, got {other:?}"),
        }
    }

    /// Asserts a full 1×n double row vector equal to `expected`.
    pub fn assert_row(value: &Value, expected: &[f64]) {
        let (shape, data) = double_array(value);
        assert_eq!(shape, vec![1, expected.len()], "1×n row, got {value:?}");
        assert_eq!(data, expected);
    }

    /// Asserts a full 1×n double row vector that is a permutation of 1..n and
    /// returns it as one-based indices.
    pub fn assert_permutation(value: &Value, n: usize) -> Vec<usize> {
        let (shape, data) = double_array(value);
        assert_eq!(shape, vec![1, n], "1×{n} row, got {value:?}");
        let mut seen = vec![false; n];
        let mut out = Vec::with_capacity(n);
        for &raw in &data {
            assert_eq!(raw.fract(), 0.0, "integer entries: {data:?}");
            let idx = raw as usize;
            assert!((1..=n).contains(&idx), "{data:?} is not a permutation");
            assert!(!seen[idx - 1], "{data:?} is not a permutation");
            seen[idx - 1] = true;
            out.push(idx);
        }
        out
    }

    pub fn assert_no_warnings<T>(f: impl FnOnce() -> T) -> T {
        crate::warning_store::reset();
        let out = f();
        let warnings = crate::warning_store::take_all();
        assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");
        out
    }

    /// Deterministic pseudo-random row-major 0/1 pattern (LCG) with roughly
    /// `percent`% nonzeros.
    pub fn random_pattern(rows: usize, cols: usize, percent: u64, seed: u64) -> Vec<f64> {
        let mut state = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (0..rows * cols)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                if (state >> 33) % 100 < percent {
                    1.0
                } else {
                    0.0
                }
            })
            .collect()
    }

    /// Independent oracle: Boolean Gaussian elimination on the symmetric
    /// pattern whose off-diagonal edges are `edges` (pairs `i < j`). Returns
    /// the filled pattern `F` (row-major, n×n, symmetric, diagonal set):
    /// `F(i,k)` for `i >= k` is the structure of `L(i,k)`.
    pub fn filled_pattern(n: usize, edges: &[(usize, usize)]) -> Vec<bool> {
        let mut f = vec![false; n * n];
        for i in 0..n {
            f[i * n + i] = true;
        }
        for &(i, j) in edges {
            f[i * n + j] = true;
            f[j * n + i] = true;
        }
        for k in 0..n {
            let below: Vec<usize> = ((k + 1)..n).filter(|&i| f[i * n + k]).collect();
            for &i in &below {
                for &j in &below {
                    f[i * n + j] = true;
                }
            }
        }
        f
    }

    /// Edges `(i, j)`, `i < j`, of the upper triangle of a row-major square
    /// literal.
    pub fn upper_edges(n: usize, row_major: &[f64]) -> Vec<(usize, usize)> {
        let mut edges = Vec::new();
        for i in 0..n {
            for j in (i + 1)..n {
                if row_major[i * n + j] != 0.0 {
                    edges.push((i, j));
                }
            }
        }
        edges
    }

    /// Edges `(i, j)`, `i < j`, of the pattern of `A'*A` for a row-major
    /// rows×cols literal.
    pub fn ata_edges(rows: usize, cols: usize, row_major: &[f64]) -> Vec<(usize, usize)> {
        let mut edges = Vec::new();
        for i in 0..cols {
            for j in (i + 1)..cols {
                if (0..rows)
                    .any(|r| row_major[r * cols + i] != 0.0 && row_major[r * cols + j] != 0.0)
                {
                    edges.push((i, j));
                }
            }
        }
        edges
    }

    /// Oracle parent vector (one-based, 0 for roots): the first off-diagonal
    /// nonzero below the diagonal in each column of the filled pattern.
    pub fn oracle_parent(n: usize, filled: &[bool]) -> Vec<f64> {
        (0..n)
            .map(|k| {
                ((k + 1)..n)
                    .find(|&i| filled[i * n + k])
                    .map_or(0.0, |i| (i + 1) as f64)
            })
            .collect()
    }

    /// Asserts that `post` (one-based) is a postorder of the forest `parent`
    /// (one-based, 0 for roots): every subtree occupies a contiguous block of
    /// positions that ends at its root.
    pub fn assert_postorder(parent: &[f64], post: &[usize]) {
        let n = parent.len();
        assert_eq!(post.len(), n);
        let mut position = vec![0usize; n];
        for (pos, &node) in post.iter().enumerate() {
            position[node - 1] = pos;
        }
        for root in 0..n {
            // Collect the subtree of `root`.
            let members: Vec<usize> = (0..n)
                .filter(|&v| {
                    let mut x = v;
                    loop {
                        if x == root {
                            return true;
                        }
                        let p = parent[x] as usize;
                        if p == 0 {
                            return false;
                        }
                        x = p - 1;
                    }
                })
                .collect();
            let end = position[root];
            assert!(end + 1 >= members.len(), "subtree of {root} too large");
            let start = end + 1 - members.len();
            for v in members {
                assert!(
                    (start..=end).contains(&position[v]),
                    "{post:?} is not a postorder of {parent:?}"
                );
            }
        }
    }
}
