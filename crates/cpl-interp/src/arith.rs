//! The tree-walker's arithmetic on values: what each operator gives, and
//! what it traps on or, in a release run, does instead (R09.3).

use super::*;

pub(super) fn binary(op: BinOp, l: Value, r: Value, span: Span) -> Res<Value> {
    Ok(match (l, r) {
        (Value::Int(a), Value::Int(b)) => {
            let result = match op {
                BinOp::Add => a.checked_add(b),
                BinOp::Sub => a.checked_sub(b),
                BinOp::Mul => a.checked_mul(b),
                // Euclidean: the remainder is never negative, whatever the
                // signs, and the quotient is the one that goes with it.
                BinOp::Div | BinOp::Rem if b == 0 => {
                    return release::div_zero(
                        span,
                        cpl_diag::msgs::interp::DIV_BY_ZERO_OP.at(&[&a, &(op.text())]),
                    )
                    .map(Value::Int)
                }
                BinOp::Div => a.checked_div_euclid(b),
                BinOp::Rem => a.checked_rem_euclid(b),
                BinOp::Pow => match u32::try_from(b) {
                    Ok(exp) => a.checked_pow(exp),
                    Err(_) if b < 0 => {
                        return trap(
                            span,
                            cpl_diag::msgs::interp::NEEDS_POWER_THAT_NOT_NEGATIVE.at(&[&b]),
                        )
                    }
                    Err(_) => match a {
                        0 | 1 => Some(a),
                        -1 => Some(if b % 2 == 0 { 1 } else { -1 }),
                        _ => None,
                    },
                },
                BinOp::BitAnd => Some(a & b),
                BinOp::BitOr => Some(a | b),
                BinOp::BitXor => Some(a ^ b),
                BinOp::Shl | BinOp::Shr if !(0..64).contains(&b) => {
                    return trap(
                        span,
                        cpl_diag::msgs::interp::SHIFT_BY_OUTSIDE_0_64.at(&[&b]),
                    )
                }
                BinOp::Shl => {
                    if a < 0 {
                        return trap(span, cpl_diag::msgs::interp::NEGATIVE_NUMBER.at(&[&a]));
                    }
                    Some(a << b).filter(|shifted| *shifted >= 0 && shifted >> b == a)
                }
                BinOp::Shr => Some(a >> b),
                BinOp::And | BinOp::Or | BinOp::Coalesce => None,
            };
            match result.filter(|v| *v != i64::MIN) {
                Some(value) => Value::Int(value),
                None => {
                    let why = cpl_diag::msgs::interp::INT_OVERFLOW_BINOP.at(&[&a, &op.text(), &b]);
                    return release::overflow(span, why, release::wrapping(op, a, b))
                        .map(Value::Int);
                }
            }
        }
        (Value::I128(Wide(a)), Value::I128(Wide(b))) => {
            let result = match op {
                BinOp::Add => a.checked_add(b),
                BinOp::Sub => a.checked_sub(b),
                BinOp::Mul => a.checked_mul(b),
                BinOp::Div | BinOp::Rem if b == 0 => {
                    return trap(
                        span,
                        cpl_diag::msgs::interp::DIV_BY_ZERO_OP.at(&[&a, &(op.text())]),
                    )
                }
                BinOp::Div => a.checked_div_euclid(b),
                BinOp::Rem => a.checked_rem_euclid(b),
                BinOp::Pow => match u32::try_from(b) {
                    Ok(exp) => a.checked_pow(exp),
                    Err(_) if b < 0 => {
                        return trap(
                            span,
                            cpl_diag::msgs::interp::NEEDS_POWER_THAT_NOT_NEGATIVE.at(&[&b]),
                        )
                    }
                    Err(_) => match a {
                        0 | 1 => Some(a),
                        -1 => Some(if b % 2 == 0 { 1 } else { -1 }),
                        _ => None,
                    },
                },
                BinOp::BitAnd => Some(a & b),
                BinOp::BitOr => Some(a | b),
                BinOp::BitXor => Some(a ^ b),
                _ => None,
            };
            match result {
                Some(value) => Value::I128(value.into()),
                None => {
                    return trap(
                        span,
                        cpl_diag::msgs::interp::I128_OVERFLOW_BINOP.at(&[&a, &(op.text()), &b]),
                    )
                }
            }
        }
        (Value::I128(Wide(a)), Value::Int(b)) => {
            if !(0..128).contains(&b) {
                return trap(
                    span,
                    cpl_diag::msgs::interp::SHIFT_BY_OUTSIDE_0_128.at(&[&b]),
                );
            }
            match op {
                BinOp::Shl if a < 0 => {
                    return trap(span, cpl_diag::msgs::interp::NEGATIVE_NUMBER.at(&[&a]))
                }
                BinOp::Shl => {
                    match Some(a << b).filter(|shifted| *shifted >= 0 && shifted >> b == a) {
                        Some(value) => Value::I128(value.into()),
                        None => {
                            return trap(
                                span,
                                cpl_diag::msgs::interp::I128_OVERFLOW_BINOP.at(&[&a, &"<<", &b]),
                            )
                        }
                    }
                }
                _ => Value::I128((a >> b).into()),
            }
        }
        // A `U64` is the 64 bits: what does not fit is dropped.
        (Value::U64(a), Value::U64(b)) => Value::U64(match op {
            BinOp::Add => a.wrapping_add(b),
            BinOp::Sub => a.wrapping_sub(b),
            BinOp::Mul => a.wrapping_mul(b),
            BinOp::Div | BinOp::Rem if b == 0 => {
                return release::div_zero(
                    span,
                    cpl_diag::msgs::interp::DIV_BY_ZERO_OP.at(&[&a, &(op.text())]),
                )
                .map(Value::U64)
            }
            BinOp::Div => a / b,
            BinOp::Rem => a % b,
            BinOp::Pow => {
                let (mut base, mut exp, mut result) = (a, b, 1u64);
                while exp > 0 {
                    if exp & 1 == 1 {
                        result = result.wrapping_mul(base);
                    }
                    base = base.wrapping_mul(base);
                    exp >>= 1;
                }
                result
            }
            BinOp::BitAnd => a & b,
            BinOp::BitOr => a | b,
            BinOp::BitXor => a ^ b,
            _ => a,
        }),
        (Value::U64(a), Value::Int(b)) => {
            if !(0..64).contains(&b) {
                return trap(
                    span,
                    cpl_diag::msgs::interp::SHIFT_BY_OUTSIDE_0_64.at(&[&b]),
                );
            }
            Value::U64(if op == BinOp::Shl { a << b } else { a >> b })
        }
        (Value::Float(a), Value::Float(b)) => Value::Float(match op {
            BinOp::Add => a + b,
            BinOp::Sub => a - b,
            BinOp::Mul => a * b,
            BinOp::Div => a / b,
            BinOp::Pow => a.powf(b),
            _ => return bug(span, "this operator has no float form"),
        }),
        (Value::Bool(a), Value::Bool(b)) => Value::Bool(match op {
            BinOp::BitAnd => a & b,
            BinOp::BitOr => a | b,
            _ => a ^ b,
        }),
        (Value::Str(mut a), Value::Str(b)) => {
            Rc::make_mut(&mut a).extend_from_slice(&b);
            Value::Str(a)
        }
        _ => return bug(span, "operands of different kinds"),
    })
}

/// `old op= new`.
pub(super) fn combine(op: AssignOp, old: Value, new: Value, span: Span) -> Res<Value> {
    Ok(match op {
        AssignOp::Set => new,
        AssignOp::Max => {
            if new > old {
                new
            } else {
                old
            }
        }
        AssignOp::Min => {
            if new < old {
                new
            } else {
                old
            }
        }
        AssignOp::Binary(op) => binary(op, old, new, span)?,
    })
}
