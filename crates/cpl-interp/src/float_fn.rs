//! The `F64` intrinsics: `sqrt`, `atan2`, `hypot`, `exp`, `ln` and the trigonometric fns.

use super::*;

/// `op` applied to `x` (and `y`, for `atan2` and `hypot`), trapping where the result would be NaN.
pub(crate) fn eval(op: Builtin, x: f64, y: Option<Value>, span: Span) -> Res<Value> {
    Ok(Value::Float(match op {
        Builtin::Atan2 => x.atan2(y.unwrap_or_default().float()),
        Builtin::Hypot => x.hypot(y.unwrap_or_default().float()),
        Builtin::Exp => x.exp(),
        Builtin::Ln if x < 0.0 => {
            return trap(span, cpl_diag::msgs::interp::LN_NEGATIVE_NUMBER.at(&[&x]))
        }
        Builtin::Ln => x.ln(),
        Builtin::Sin => x.sin(),
        Builtin::Cos => x.cos(),
        Builtin::Tan => x.tan(),
        Builtin::Asin | Builtin::Acos if !(-1.0..=1.0).contains(&x) => {
            let name = if op == Builtin::Asin { "asin" } else { "acos" };
            return trap(span, cpl_diag::msgs::interp::OUTSIDE_1_1.at(&[&name, &x]));
        }
        Builtin::Asin => x.asin(),
        _ => x.acos(),
    }))
}
