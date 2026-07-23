//! Small helpers vendored from the original clean-room branch so the parallel
//! implementations are self-contained (their original sibling modules were not
//! relocated here). Behaviour identical to the source definitions.
use runmat_builtins::Value;
use std::path::Path;

pub(crate) fn size_from_f64(raw: f64, name: &str) -> Result<usize, String> {
    if !raw.is_finite() || raw < 0.0 || raw.fract() != 0.0 {
        return Err(format!("{name}: size must be a non-negative integer"));
    }
    Ok(raw as usize)
}

pub(crate) fn scalar_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Num(n) => Some(*n),
        Value::Int(i) => Some(i.to_f64()),
        Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        Value::Tensor(t) if t.data.len() == 1 => Some(t.data[0]),
        _ => None,
    }
}

pub(crate) fn is_rooted_path(path: &Path) -> bool {
    path.is_absolute() || path.has_root()
}
