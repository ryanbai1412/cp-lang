//! The largest and smallest value an expression can take, from the input
//! bounds the program declares.
//!
//! `let n: 1..=2e5 = read()` says `n` is at most 200000, `const K = 20`
//! says `K` is 20, and `domain V = 1..=n` then has at most 200000 values.
//! Anything this cannot see through is `None`, and the caller stays quiet.

use crate::File;
use cpl_syntax::ast::{BinOp, Expr, ExprKind, PatternKind, StmtKind, Type, TypeKind, UnOp};
use std::collections::HashMap;

/// An inclusive range of integers.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Interval {
    pub lo: i128,
    pub hi: i128,
}

impl Interval {
    /// How many integers it holds; 0 when it is empty.
    pub(crate) fn count(self) -> u128 {
        if self.hi < self.lo {
            0
        } else {
            (self.hi - self.lo) as u128 + 1
        }
    }
}

/// What the top level of a file says about its names.
#[derive(Default, Clone)]
pub(crate) struct Bounds {
    values: HashMap<String, Interval>,
    /// The number of values of each domain, at most.
    domains: HashMap<String, u128>,
    aliases: HashMap<String, Type>,
}

impl Bounds {
    pub(crate) fn of(file: &File) -> Bounds {
        let mut bounds = Bounds::default();
        // In source order: a bound can only use what is declared above it.
        for (_, stmt) in file.top_level() {
            match &stmt.kind {
                StmtKind::Const {
                    name,
                    init: Some(init),
                    ..
                } => {
                    if let Some(value) = bounds.expr(init) {
                        bounds.values.insert(name.text.clone(), value);
                    }
                }
                StmtKind::Let {
                    pat, ty: Some(ty), ..
                } => {
                    if let (PatternKind::Name(name), Some(range)) = (&pat.kind, bounds.range(ty)) {
                        bounds.values.insert(name.clone(), range);
                    }
                }
                StmtKind::Let {
                    mutable: false,
                    pat,
                    ty: None,
                    init: Some(init),
                    ..
                } => {
                    if let (PatternKind::Name(name), Some(value)) = (&pat.kind, bounds.expr(init)) {
                        bounds.values.insert(name.clone(), value);
                    }
                }
                StmtKind::Domain { name, def } => {
                    let size = match &def.kind {
                        ExprKind::Range {
                            lo: Some(lo),
                            hi: Some(hi),
                            inclusive,
                            step: None,
                        } => bounds.span(lo, hi, *inclusive).map(Interval::count),
                        _ => None,
                    };
                    if let Some(size) = size {
                        bounds.domains.insert(name.text.clone(), size);
                    }
                }
                StmtKind::TypeAlias { name, generics, ty } if generics.is_empty() => {
                    bounds.aliases.insert(name.text.clone(), ty.clone());
                }
                _ => {}
            }
        }
        bounds
    }

    /// From here on `name` is in `value`, or unknown: a local that
    /// shadows a bounded name hides its bound.
    pub(crate) fn bind(&mut self, name: &str, value: Option<Interval>) {
        match value {
            Some(value) => self.values.insert(name.to_string(), value),
            None => self.values.remove(name),
        };
    }

    /// The values `lo..hi` or `lo..=hi` can cover.
    pub(crate) fn span(&self, lo: &Expr, hi: &Expr, inclusive: bool) -> Option<Interval> {
        let (lo, hi) = (self.expr(lo)?, self.expr(hi)?);
        Some(Interval {
            lo: lo.lo,
            hi: if inclusive {
                hi.hi
            } else {
                hi.hi.checked_sub(1)?
            },
        })
    }

    pub(crate) fn range(&self, ty: &Type) -> Option<Interval> {
        match &ty.kind {
            TypeKind::Range {
                lo: Some(lo),
                hi: Some(hi),
                inclusive,
            } => self.span(lo, hi, *inclusive),
            _ => None,
        }
    }

    /// The interval an expression's value lies in.
    pub(crate) fn expr(&self, expr: &Expr) -> Option<Interval> {
        match &expr.kind {
            ExprKind::Int(value) => Some(Interval {
                lo: *value,
                hi: *value,
            }),
            ExprKind::Name(name) => self.values.get(name).copied(),
            ExprKind::Paren(inner) => self.expr(inner),
            ExprKind::Unary {
                op: UnOp::Neg,
                expr,
            } => {
                let inner = self.expr(expr)?;
                Some(Interval {
                    lo: inner.hi.checked_neg()?,
                    hi: inner.lo.checked_neg()?,
                })
            }
            ExprKind::Binary { op, lhs, rhs, .. } => {
                let (a, b) = (self.expr(lhs)?, self.expr(rhs)?);
                match op {
                    BinOp::Add => Some(Interval {
                        lo: a.lo.checked_add(b.lo)?,
                        hi: a.hi.checked_add(b.hi)?,
                    }),
                    BinOp::Sub => Some(Interval {
                        lo: a.lo.checked_sub(b.hi)?,
                        hi: a.hi.checked_sub(b.lo)?,
                    }),
                    BinOp::Mul => {
                        let corners = [
                            a.lo.checked_mul(b.lo)?,
                            a.lo.checked_mul(b.hi)?,
                            a.hi.checked_mul(b.lo)?,
                            a.hi.checked_mul(b.hi)?,
                        ];
                        Some(Interval {
                            lo: *corners.iter().min()?,
                            hi: *corners.iter().max()?,
                        })
                    }
                    BinOp::Div if b.lo > 0 => {
                        let corners = [
                            a.lo.checked_div_euclid(b.lo)?,
                            a.lo.checked_div_euclid(b.hi)?,
                            a.hi.checked_div_euclid(b.lo)?,
                            a.hi.checked_div_euclid(b.hi)?,
                        ];
                        Some(Interval {
                            lo: *corners.iter().min()?,
                            hi: *corners.iter().max()?,
                        })
                    }
                    BinOp::Rem if b.lo > 0 => Some(if a.lo >= 0 && a.hi < b.lo {
                        a
                    } else {
                        let hi = b.hi - 1;
                        Interval {
                            lo: 0,
                            hi: if a.lo >= 0 { hi.min(a.hi) } else { hi },
                        }
                    }),
                    BinOp::Pow if b.lo == b.hi && b.lo >= 0 => {
                        let k = u32::try_from(b.lo).ok()?;
                        let (lo, hi) = (a.lo.checked_pow(k)?, a.hi.checked_pow(k)?);
                        Some(if k % 2 == 1 || a.lo >= 0 {
                            Interval { lo, hi }
                        } else if a.hi <= 0 {
                            Interval { lo: hi, hi: lo }
                        } else {
                            Interval { lo: 0, hi: lo.max(hi) }
                        })
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// How many values a type has, at most.
    pub(crate) fn type_size(&self, ty: &Type) -> Option<u128> {
        match &ty.kind {
            TypeKind::Range { .. } => self.range(ty).map(Interval::count),
            TypeKind::Named { name, args } if args.is_empty() => match name.text.as_str() {
                "Bool" => Some(2),
                "Char" | "U8" => Some(256),
                other => self
                    .domains
                    .get(other)
                    .copied()
                    .or_else(|| self.type_size(self.aliases.get(other)?)),
            },
            TypeKind::Optional(inner) => self.type_size(inner)?.checked_add(1),
            TypeKind::Tuple(parts) => parts.iter().try_fold(1u128, |total, part| {
                total.checked_mul(self.type_size(part)?)
            }),
            TypeKind::Record(fields) => fields.iter().try_fold(1u128, |total, (_, ty)| {
                total.checked_mul(self.type_size(ty)?)
            }),
            _ => None,
        }
    }
}
