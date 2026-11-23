//! The `F64` intrinsics: `sqrt`, `atan2` and the trigonometric fns.

use super::*;

/// `op` applied to `x` (and `y`, for `atan2`), trapping where the result would be NaN.
pub(crate) fn eval(op: Builtin, x: f64, y: Option<Value>, span: Span) -> Res<Value> {
    Ok(Value::Float(match op {
        Builtin::Sqrt if x < 0.0 => return trap(span, format!("sqrt({x}) of a negative number")),
        Builtin::Sqrt => x.sqrt(),
        Builtin::Atan2 => x.atan2(y.unwrap_or_default().float()),
        Builtin::Sin => x.sin(),
        Builtin::Cos => x.cos(),
        Builtin::Tan => x.tan(),
        Builtin::Asin | Builtin::Acos if !(-1.0..=1.0).contains(&x) => {
            let name = if op == Builtin::Asin { "asin" } else { "acos" };
            return trap(span, format!("{name}({x}) is outside -1..=1"));
        }
        Builtin::Asin => x.asin(),
        _ => x.acos(),
    }))
}
