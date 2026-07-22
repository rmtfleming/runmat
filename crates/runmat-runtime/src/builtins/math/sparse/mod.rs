//! Sparse construction and conversion builtins for RunMat.
//!
//! Clean-room provenance: specs/009-sparse-construction (approved Tier A
//! exports `full` 0.2.0, `speye` 0.2.0, `spdiags` 0.2.0).

pub(crate) mod full;
pub(crate) mod spdiags;
pub(crate) mod speye;

use runmat_builtins::Value;

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
