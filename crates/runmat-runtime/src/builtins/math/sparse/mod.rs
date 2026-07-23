//! Sparse construction and conversion builtins for RunMat.
//!
//! Clean-room provenance: specs/009-sparse-construction (approved Tier A
//! exports `full` 0.2.0, `speye` 0.2.0, `spdiags` 0.2.0) and
//! specs/017-nonzeros (approved Tier B export `nonzeros` 0.2.0).

pub(crate) mod colamd;
pub(crate) mod dissect;
pub(crate) mod spalloc;

use runmat_builtins::Value;

/// (rows, cols, coordinates) where each coordinate is a (row, col) pair of a
/// structurally stored entry.
pub(crate) type MatrixPattern = (usize, usize, Vec<(usize, usize)>);

/// Largest f64 value that can be cast to usize without overflow.
pub(crate) fn max_usize_cast_value() -> f64 {
    if usize::BITS <= f64::MANTISSA_DIGITS {
        usize::MAX as f64
    } else {
        f64::from_bits((usize::MAX as f64).to_bits() - 1)
    }
}

/// Parse one dimension size from a raw numeric value. Returns a plain message
/// on failure; callers wrap it in their builtin-specific error.
pub(crate) fn size_from_f64(raw: f64, name: &str) -> Result<usize, String> {
    if !raw.is_finite() || raw < 0.0 || raw.fract() != 0.0 {
        return Err(format!("{name} must be a nonnegative integer"));
    }
    if raw > max_usize_cast_value() {
        return Err(format!("{name} exceeds the maximum supported size"));
    }
    Ok(raw as usize)
}

/// Extract an f64 from scalar-like values (Num, Int, Bool, 1-element tensor).
pub(crate) fn scalar_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Num(n) => Some(*n),
        Value::Int(i) => Some(i.to_f64()),
        Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        Value::Tensor(t) if t.data.len() == 1 => Some(t.data[0]),
        _ => None,
    }
}

/// Read the structural pattern of a 2-D matrix value as `(rows, cols, coords)`,
/// where `coords` lists the zero-based `(row, col)` of every structurally
/// nonzero entry. Used by the sparse-ordering builtins (`colamd`, `dissect`)
/// which operate on the sparsity pattern regardless of the numeric values.
///
/// Sparse (CSC) storage is enumerated directly; dense/logical storage is read
/// column-major. `NaN` counts as a nonzero (it is a stored value). Scalar-like
/// numeric values are treated as 1×1 matrices. Non-matrix inputs and N-D
/// tensors return a plain error message for the caller to wrap.
pub(crate) fn matrix_pattern(value: &Value) -> Result<MatrixPattern, String> {
    match value {
        Value::SparseTensor(sparse) => {
            let mut coords = Vec::with_capacity(sparse.values.len());
            for col in 0..sparse.cols {
                for idx in sparse.col_ptrs[col]..sparse.col_ptrs[col + 1] {
                    coords.push((sparse.row_indices[idx], col));
                }
            }
            Ok((sparse.rows, sparse.cols, coords))
        }
        Value::Tensor(t) => {
            reject_nd(&t.shape)?;
            let (rows, cols) = (t.rows, t.cols);
            let mut coords = Vec::new();
            for col in 0..cols {
                for row in 0..rows {
                    let v = t.data[row + col * rows];
                    if v != 0.0 || v.is_nan() {
                        coords.push((row, col));
                    }
                }
            }
            Ok((rows, cols, coords))
        }
        Value::LogicalArray(logical) => {
            reject_nd(&logical.shape)?;
            let rows = logical.shape.first().copied().unwrap_or(0);
            let cols = logical.shape.get(1).copied().unwrap_or(1);
            let mut coords = Vec::new();
            for col in 0..cols {
                for row in 0..rows {
                    if logical.data[row + col * rows] != 0 {
                        coords.push((row, col));
                    }
                }
            }
            Ok((rows, cols, coords))
        }
        // Scalar-like numeric values are 1×1 matrices.
        other => match scalar_f64(other) {
            Some(v) => {
                let coords = if v != 0.0 || v.is_nan() {
                    vec![(0, 0)]
                } else {
                    Vec::new()
                };
                Ok((1, 1, coords))
            }
            None => Err(format!(
                "input must be a 2-D numeric matrix (full or sparse), got {other:?}"
            )),
        },
    }
}

/// Reject arrays with more than two dimensions (a trailing singleton dimension
/// is allowed and treated as 2-D).
fn reject_nd(shape: &[usize]) -> Result<(), String> {
    if shape.len() > 2 && shape[2..].iter().product::<usize>() > 1 {
        return Err("input must be a 2-D matrix (N-D arrays are not supported)".to_string());
    }
    Ok(())
}
