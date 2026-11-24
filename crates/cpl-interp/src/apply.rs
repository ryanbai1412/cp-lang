//! The part of evaluating an expression that both engines share: its
//! value from the values of its operands, which the tree-walker evaluates
//! with `eval` and the VM in registers.

use crate::observe::Observer;
use crate::value::{TupleVal, Value};
use crate::*;
use cpl_ir::OrdHook;

/// The operands of `e`, evaluated in order before [`Interp::apply`], when
/// every one of them is evaluated, once, as a plain value.
pub(crate) fn operands(e: &Expr) -> Vec<&Expr> {
    match &e.kind {
        ExprKind::Cmp(_, lhs, rhs) => vec![lhs, rhs],
        ExprKind::Unary(_, x)
        | ExprKind::Cast(_, x)
        | ExprKind::IsVariant(x, _)
        | ExprKind::Payload(x, ..) => vec![x],
        ExprKind::Tuple(items) | ExprKind::Variant(_, items) | ExprKind::Builtin(_, items) => {
            items.iter().collect()
        }
        ExprKind::Binary(op, lhs, rhs) if !lazy(*op) => vec![lhs, rhs],
        ExprKind::Read(spec) => spec_exprs(spec).unwrap_or_default(),
        ExprKind::Index(base, indices) => std::iter::once(&**base).chain(indices).collect(),
        _ => Vec::new(),
    }
}

/// The expressions in `spec`, each evaluated once before the read, or
/// `None` when it reads through a call, whose reads cannot be hoisted.
fn spec_exprs(spec: &Spec) -> Option<Vec<&Expr>> {
    let mut out = Vec::new();
    let mut todo = vec![spec];
    while let Some(spec) = todo.pop() {
        match spec {
            Spec::Plain(_) | Spec::Line => {}
            Spec::Int { lo, hi, .. } | Spec::Str { lo, hi, .. } => {
                out.extend(lo.iter().chain(hi).map(|x| &**x))
            }
            Spec::Fields(_, fields) => todo.extend(fields.iter().rev()),
            Spec::Array { dims, elem } => {
                for dim in dims {
                    if let DimSpec::Int(len) | DimSpec::IntFrom(_, len) = dim {
                        out.push(len);
                    }
                }
                todo.push(elem);
            }
            Spec::Opt(inner, absent) => {
                out.extend(absent.as_deref());
                todo.push(inner);
            }
            Spec::Call(_) | Spec::Parse(..) | Spec::List(_) => return None,
        }
    }
    Some(out)
}

/// Whether `e` is applied to its [`operands`] by [`Interp::apply`].
pub(crate) fn applies(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Binary(op, ..) => return !lazy(*op),
        ExprKind::Read(spec) => return spec_exprs(spec).is_some(),
        _ => {}
    }
    matches!(
        e.kind,
        ExprKind::Builtin(..)
            | ExprKind::Index(..)
            | ExprKind::Unary(..)
            | ExprKind::Cmp(..)
            | ExprKind::Cast(..)
            | ExprKind::Tuple(_)
            | ExprKind::Variant(..)
            | ExprKind::IsVariant(..)
            | ExprKind::Payload(..)
    )
}

/// Whether `op` evaluates its right operand only for some left ones.
fn lazy(op: BinOp) -> bool {
    matches!(op, BinOp::And | BinOp::Or | BinOp::Coalesce)
}

impl<'p, 'o, O: Observer> Interp<'p, 'o, O> {
    /// The value of `e` from `vals`, the values of its [`operands`].
    pub(crate) fn apply(&mut self, e: &'p Expr, vals: Vec<Value>) -> Res<Value> {
        let span = e.span;
        Ok(match &e.kind {
            ExprKind::Cmp(op, lhs, _) => {
                let mut vals = vals.into_iter();
                let (l, r) = (vals.next().unwrap(), vals.next().unwrap());
                let l = match l {
                    Value::MutView(mv) => Value::List(Rc::new(self.mut_view_to_vec(&mv, span)?)),
                    other => other,
                };
                let r = match r {
                    Value::MutView(mv) => Value::List(Rc::new(self.mut_view_to_vec(&mv, span)?)),
                    other => other,
                };
                match (&l, &r) {
                    (Value::Int(a), Value::Int(b)) => {
                        return Ok(Value::Bool(match op {
                            CmpOp::Eq => a == b,
                            CmpOp::Ne => a != b,
                            CmpOp::Lt => a < b,
                            CmpOp::Le => a <= b,
                            CmpOp::Gt => a > b,
                            CmpOp::Ge => a >= b,
                            _ => false,
                        }));
                    }
                    (Value::Char(a), Value::Char(b)) => {
                        return Ok(Value::Bool(match op {
                            CmpOp::Eq => a == b,
                            CmpOp::Ne => a != b,
                            CmpOp::Lt => a < b,
                            CmpOp::Le => a <= b,
                            CmpOp::Gt => a > b,
                            CmpOp::Ge => a >= b,
                            _ => false,
                        }));
                    }
                    (Value::Bool(a), Value::Bool(b)) => {
                        return Ok(Value::Bool(match op {
                            CmpOp::Eq => a == b,
                            CmpOp::Ne => a != b,
                            CmpOp::Lt => a < b,
                            CmpOp::Le => a <= b,
                            CmpOp::Gt => a > b,
                            CmpOp::Ge => a >= b,
                            _ => false,
                        }));
                    }
                    _ => {}
                }
                if matches!(op, CmpOp::Eq | CmpOp::Ne) {
                    let eq = self.eq_ty_val(&lhs.ty, &l, &r, span)?;
                    return Ok(Value::Bool(if *op == CmpOp::Eq { eq } else { !eq }));
                }
                if matches!(op, CmpOp::Lt | CmpOp::Le | CmpOp::Gt | CmpOp::Ge) {
                    // One `lt` or `cmp` call per comparison, as the C++ makes, so the
                    // profiler counts the same calls in both.
                    if let Ty::Struct(id) = &lhs.ty {
                        if let Some(hook) = self.program.structs[*id].ord {
                            let (a, b) = if matches!(op, CmpOp::Lt | CmpOp::Ge) {
                                (l, r)
                            } else {
                                (r, l)
                            };
                            let less = self.user_less(hook, a, b, span)?;
                            return Ok(Value::Bool(if matches!(op, CmpOp::Lt | CmpOp::Gt) {
                                less
                            } else {
                                !less
                            }));
                        }
                    }
                    // `a > b` is `b < a` and `a <= b` is `!(b < a)`, as in
                    // C++, so a user `lt` inside a tuple runs the same calls.
                    let flip = matches!(op, CmpOp::Gt | CmpOp::Le);
                    let (a, b) = if flip { (&r, &l) } else { (&l, &r) };
                    let less = self.cmp_ty_val(&lhs.ty, a, b, span)?.is_lt();
                    return Ok(Value::Bool(less != matches!(op, CmpOp::Le | CmpOp::Ge)));
                }
                Value::Bool(compare(*op, &l, &r))
            }
            ExprKind::Cast(cast, _) => {
                let value = vals.into_iter().next().unwrap();
                self.cast(*cast, value, span)?
            }
            ExprKind::Tuple(items) => {
                let mut values = vals;
                if items.len() == 1
                    && !matches!(
                        &e.ty,
                        Ty::List(_) | Ty::Set(_) | Ty::HashSet(_) | Ty::Map(..) | Ty::HashMap(..)
                    )
                {
                    let v = values.pop().unwrap();
                    if let Value::Int(x) = v {
                        return Ok(Value::Tuple(TupleVal::OneInt(x)));
                    }
                    return Ok(Value::Tuple(TupleVal::Boxed(Rc::new(vec![v]))));
                }
                match &e.ty {
                    Ty::List(_) => Value::List(Rc::new(values)),
                    Ty::Set(_) | Ty::HashSet(_) => {
                        Value::Set(Rc::new(values.into_iter().collect()))
                    }
                    Ty::Map(..) | Ty::HashMap(..) => Value::Map(Rc::default()),
                    _ => Value::tuple(values),
                }
            }
            ExprKind::Variant(tag, _) => Value::Enum(*tag as u32, Rc::new(vals)),
            ExprKind::IsVariant(_, tag) => match vals.into_iter().next().unwrap() {
                Value::Enum(actual, _) => Value::Bool(actual as usize == *tag),
                _ => Value::Bool(false),
            },
            ExprKind::Payload(_, tag, index) => match vals.into_iter().next().unwrap() {
                Value::Enum(actual, payload) if actual as usize == *tag => payload[*index].clone(),
                _ => return trap(span, "internal: payload of another variant"),
            },
            ExprKind::Unary(op, _) => match (op, vals.into_iter().next().unwrap()) {
                (UnOp::Neg, Value::Int(v)) => match v.checked_neg() {
                    Some(v) => Value::Int(v),
                    None => {
                        return trap(
                            span,
                            format!("integer overflow: `-{v}` does not fit in an `Int` (64 bits)"),
                        )
                    }
                },
                (UnOp::Neg, Value::I128(v)) => match v.checked_neg() {
                    Some(v) => Value::I128(v),
                    None => {
                        return trap(
                            span,
                            format!(
                                "integer overflow: `-{v}` does not fit in an `I128` (128 bits)"
                            ),
                        )
                    }
                },
                (UnOp::Neg, Value::U64(v)) => Value::U64(v.wrapping_neg()),
                (UnOp::Neg, Value::Float(v)) => Value::Float(-v),
                (UnOp::Not, Value::Bool(v)) => Value::Bool(!v),
                (UnOp::Pos, v) => v,
                (UnOp::BitNot, Value::Int(v)) => Value::Int(!v),
                (UnOp::BitNot, Value::I128(v)) => Value::I128(!v),
                (UnOp::BitNot, Value::U64(v)) => Value::U64(!v),
                (_, other) => other,
            },
            ExprKind::Read(spec) => {
                let given: Vec<_> = operands(e).into_iter().zip(vals).collect();
                self.read(spec, span, &given)?
            }
            ExprKind::Builtin(op, args) => self.builtin_values(*op, args, e, vals)?,
            ExprKind::Index(base, _) => self.index_value(&vals[0], &vals[1..], &base.ty, span)?,
            ExprKind::Binary(op, ..) => {
                let mut vals = vals.into_iter();
                let (l, r) = (vals.next().unwrap(), vals.next().unwrap());
                binary(*op, l, r, span)?
            }
            _ => return trap(span, "internal: no shared evaluation for this expression"),
        })
    }

    /// Whether `l < r` under a struct's own `Ord`, with one hook call:
    /// `lt(l, r)`, or `cmp(l, r) == Less`, as the C++ `operator<` makes.
    pub(crate) fn user_less(&mut self, hook: OrdHook, l: Value, r: Value, span: Span) -> Res<bool> {
        let (OrdHook::Lt(f) | OrdHook::Cmp(f)) = hook;
        let got = self.call_direct(f, vec![l, r], span)?;
        let OrdHook::Cmp(_) = hook else {
            return Ok(got.bool());
        };
        let (Value::Enum(tag, _), &Ty::Enum(e)) = (&got, &self.program.fns[f].ret) else {
            return trap(span, "`cmp` must return an `Ordering`");
        };
        Ok(self.program.enums[e].variants[*tag as usize].0 == "Less")
    }

    /// How a struct's own `Ord` orders `l` and `r`: built from `<` both
    /// ways, as C++ orders tuples and sorts, so both run the same calls.
    pub(crate) fn user_order(
        &mut self,
        hook: OrdHook,
        l: &Value,
        r: &Value,
        span: Span,
    ) -> Res<std::cmp::Ordering> {
        use std::cmp::Ordering;
        if self.user_less(hook, l.clone(), r.clone(), span)? {
            return Ok(Ordering::Less);
        }
        if self.user_less(hook, r.clone(), l.clone(), span)? {
            return Ok(Ordering::Greater);
        }
        Ok(Ordering::Equal)
    }

    pub(crate) fn less_ty_val(&mut self, ty: &Ty, l: &Value, r: &Value, span: Span) -> Res<bool> {
        if let Ty::Struct(id) = ty {
            if let Some(hook) = self.program.structs[*id].ord {
                return self.user_less(hook, l.clone(), r.clone(), span);
            }
        }
        Ok(self.cmp_ty_val(ty, l, r, span)?.is_lt())
    }
}
