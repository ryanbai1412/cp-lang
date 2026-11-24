//! Range values: `lo..hi` written where a `Range<V>` is wanted, and the
//! `?` operator on positions and optionals.

use super::*;
use cpl_diag::msgs;

impl<'a> Lowerer<'a> {
    /// `j..i` with integer literal ends where a `Range<V>` is wanted: the
    /// piece between the cuts they label (R04.4.26, R04.8.8).
    pub(in crate::lower) fn literal_piece(
        &mut self,
        lo: Option<&ast::Expr>,
        hi: Option<&ast::Expr>,
        inclusive: bool,
        expected: Option<&Ty>,
        span: Span,
    ) -> Option<Expr> {
        let Some(Ty::Range(of)) = expected else {
            return None;
        };
        let Ty::Pos(d) = **of else {
            return None;
        };
        let literal =
            |e: Option<&ast::Expr>| e.is_none_or(|e| matches!(e.kind, ast::ExprKind::Int(_)));
        if (lo.is_none() && hi.is_none()) || !literal(lo) || !literal(hi) {
            return None;
        }
        let ord = |lw: &mut Self, e: &ast::Expr| {
            let label = lw.expr(e, None);
            let first = lw.domain_first_label(d, span);
            lw.mk_binary(BinOp::Sub, label, first, Ty::Int, span)
        };
        let lo = match lo {
            Some(e) => ord(self, e),
            None => self.int_lit(0, span),
        };
        let hi = match hi {
            Some(e) if inclusive => {
                let at = ord(self, e);
                self.int_add(at, 1)
            }
            Some(e) => ord(self, e),
            None => self.mk(ExprKind::DomainLen(d), Ty::Int, span),
        };
        Some(self.mk(
            ExprKind::Range {
                lo: Box::new(lo),
                hi: Box::new(hi),
            },
            Ty::range(Ty::Pos(d)),
            span,
        ))
    }

    pub(in crate::lower) fn range_val(
        &mut self,
        lo: Option<&ast::Expr>,
        hi: Option<&ast::Expr>,
        inclusive: bool,
        expected: Option<&Ty>,
        span: Span,
    ) -> Expr {
        if let Some(piece) = self.literal_piece(lo, hi, inclusive, expected, span) {
            return piece;
        }
        let lo_e = lo.map(|e| self.expr(e, None));
        let hi_e = hi.map(|e| self.expr(e, None));
        match (lo_e, hi_e) {
            (Some(l), Some(r)) => match (&l.ty, &r.ty) {
                (Ty::Int, Ty::Int) => {
                    let hi = if inclusive { self.int_add(r, 1) } else { r };
                    self.mk(
                        ExprKind::Range {
                            lo: Box::new(l),
                            hi: Box::new(hi),
                        },
                        Ty::range(Ty::Int),
                        span,
                    )
                }
                (Ty::I128, Ty::I128) | (Ty::I128, Ty::Int) | (Ty::Int, Ty::I128) => {
                    let l = self.coerce(l, &Ty::I128, span);
                    let r = self.coerce(r, &Ty::I128, span);
                    let hi = if inclusive {
                        let one = self.coerce(self.int_lit(1, span), &Ty::I128, span);
                        self.mk_binary(BinOp::Add, r, one, Ty::I128, span)
                    } else {
                        r
                    };
                    self.mk(
                        ExprKind::Range {
                            lo: Box::new(l),
                            hi: Box::new(hi),
                        },
                        Ty::range(Ty::I128),
                        span,
                    )
                }
                (Ty::Pos(d), Ty::Pos(d2)) if d == d2 => {
                    let d = *d;
                    let lo_ord = self.cast(Cast::Ord, l, Ty::Int);
                    let hi_ord = self.cast(Cast::Ord, r, Ty::Int);
                    let hi_ord = if inclusive {
                        self.int_add(hi_ord, 1)
                    } else {
                        hi_ord
                    };
                    self.mk(
                        ExprKind::Range {
                            lo: Box::new(lo_ord),
                            hi: Box::new(hi_ord),
                        },
                        Ty::range(Ty::Pos(d)),
                        span,
                    )
                }
                (Ty::Bound(d), Ty::Bound(d2)) if d == d2 => {
                    let d = *d;
                    let convert_to_pos = !inclusive
                        && matches!(
                            expected,
                            Some(Ty::Range(inner)) if matches!(inner.as_ref(), Ty::Pos(ed) if *ed == d)
                        );
                    let lo_ord = self.cast(Cast::Ord, l, Ty::Int);
                    let hi_ord = self.cast(Cast::Ord, r, Ty::Int);
                    let hi_ord = if !convert_to_pos && inclusive {
                        self.int_add(hi_ord, 1)
                    } else {
                        hi_ord
                    };
                    let elem = if convert_to_pos {
                        Ty::Pos(d)
                    } else {
                        Ty::Bound(d)
                    };
                    self.mk(
                        ExprKind::Range {
                            lo: Box::new(lo_ord),
                            hi: Box::new(hi_ord),
                        },
                        Ty::range(elem),
                        span,
                    )
                }
                // `lo..hi` with `F64` ends is an interval: it is searched, never walked.
                (Ty::Float, Ty::Float) if !inclusive => self.mk(
                    ExprKind::Range {
                        lo: Box::new(l),
                        hi: Box::new(r),
                    },
                    Ty::range(Ty::Float),
                    span,
                ),
                (Ty::Float, Ty::Float) => {
                    let message = "an interval of `F64`s is written `lo..hi`, not `..=`";
                    self.error(codes::TYPE_MISMATCH, span, message.to_string());
                    self.help("a real interval has no last element, so `..` and `..=` are the same set to a search");
                    self.err_expr(span)
                }
                (Ty::Error, _) | (_, Ty::Error) => self.err_expr(span),
                (Ty::U64, _) | (_, Ty::U64) => {
                    self.unsupported(span, "a range of `U64` values");
                    self.help("loop over an `Int` or `I128` range and narrow inside: `U64(x)`");
                    self.err_expr(span)
                }
                _ => {
                    self.bad_range_ends((lo, hi), (&l.ty, &r.ty), inclusive, span);
                    self.err_expr(span)
                }
            },
            (Some(l), None) => match l.ty {
                Ty::Pos(d) | Ty::Bound(d) => {
                    let want_bound =
                        matches!(l.ty, Ty::Bound(_)) && Self::cut_range_wanted(expected, d);
                    let lo_ord = self.cast(Cast::Ord, l, Ty::Int);
                    let len = self.mk(ExprKind::DomainLen(d), Ty::Int, span);
                    let hi_ord = if want_bound {
                        self.int_add(len, 1)
                    } else {
                        len
                    };
                    let elem = if want_bound { Ty::Bound(d) } else { Ty::Pos(d) };
                    self.mk(
                        ExprKind::Range {
                            lo: Box::new(lo_ord),
                            hi: Box::new(hi_ord),
                        },
                        Ty::range(elem),
                        span,
                    )
                }
                // `(0..).find(..)`: a range with no end stops when the
                // search does. It runs to `Int.MAX`.
                Ty::Int => {
                    let hi = self.int_lit(i64::MAX, span);
                    self.mk(
                        ExprKind::Range {
                            lo: Box::new(l),
                            hi: Box::new(hi),
                        },
                        Ty::range(Ty::Int),
                        span,
                    )
                }
                _ => {
                    self.unsupported(span, "an unbounded range value");
                    self.err_expr(span)
                }
            },
            (None, Some(r)) => match r.ty {
                Ty::Int => {
                    let lo = self.int_lit(0, span);
                    let hi = if inclusive { self.int_add(r, 1) } else { r };
                    self.mk(
                        ExprKind::Range {
                            lo: Box::new(lo),
                            hi: Box::new(hi),
                        },
                        Ty::range(Ty::Int),
                        span,
                    )
                }
                Ty::Pos(d) => {
                    let lo_ord = self.int_lit(0, span);
                    let hi_ord = self.cast(Cast::Ord, r, Ty::Int);
                    let hi_ord = if inclusive {
                        self.int_add(hi_ord, 1)
                    } else {
                        hi_ord
                    };
                    self.mk(
                        ExprKind::Range {
                            lo: Box::new(lo_ord),
                            hi: Box::new(hi_ord),
                        },
                        Ty::range(Ty::Pos(d)),
                        span,
                    )
                }
                Ty::Bound(d) => {
                    let want_bound = Self::cut_range_wanted(expected, d);
                    let lo_ord = self.int_lit(0, span);
                    let hi_ord = self.cast(Cast::Ord, r, Ty::Int);
                    let hi_ord = if want_bound && inclusive {
                        self.int_add(hi_ord, 1)
                    } else {
                        hi_ord
                    };
                    let elem = if want_bound { Ty::Bound(d) } else { Ty::Pos(d) };
                    self.mk(
                        ExprKind::Range {
                            lo: Box::new(lo_ord),
                            hi: Box::new(hi_ord),
                        },
                        Ty::range(elem),
                        span,
                    )
                }
                _ => {
                    self.unsupported(span, "an unbounded range value");
                    self.err_expr(span)
                }
            },
            (None, None) => {
                self.unsupported(span, "a range with neither end");
                self.err_expr(span)
            }
        }
    }

    /// `(d? + 1) ?? -1`: a `?` on the left of `??` makes the whole left
    /// side `None`, and the default is taken. (R02.4.7)
    pub(in crate::lower) fn try_extent(&mut self, lhs: &ast::Expr, span: Span) -> Expr {
        let lid = self.next_loop;
        self.next_loop += 1;
        let outer = self.cx.try_extent.replace(lid);
        let pending = std::mem::take(&mut self.cx.pending);
        let value = self.expr_maybe_opt_reduce(lhs);
        let mut body = std::mem::replace(&mut self.cx.pending, pending);
        self.cx.try_extent = outer;
        let (ty, value) = match value.ty {
            Ty::Error => return value,
            Ty::Opt(_) => (value.ty.clone(), value),
            ref t => {
                let ty = Ty::opt(t.clone());
                (ty.clone(), self.mk_some(value, ty, span))
            }
        };
        self.do_expr(Some(&ty), span, |lw, out| {
            let slot = lw.temp(ty.clone());
            out.push(Stmt::Let(
                slot,
                Some(lw.mk(ExprKind::None, ty.clone(), span)),
            ));
            let place = Place {
                kind: PlaceKind::Local(slot),
                ty: ty.clone(),
                span: lw.span(span),
            };
            body.push(Stmt::Assign(place, AssignOp::Set, value));
            body.push(Stmt::Break(Some(lid)));
            let yes = lw.mk(ExprKind::Bool(true), Ty::Bool, span);
            out.push(Stmt::While(lid, yes, Block { stmts: body }));
            out.push(Stmt::Give(lw.local(slot, span)));
        })
    }

    pub(in crate::lower) fn try_expr(&mut self, inner: &ast::Expr, span: Span) -> Expr {
        self.cx.as_written = narrow::asks_optional(inner);
        let opt = self.expr(inner, None);
        self.cx.as_written = false;
        let Ty::Opt(inner_ty) = opt.ty.clone() else {
            if opt.ty != Ty::Error {
                self.report(
                    span,
                    msgs::ir::TYPE_MISMATCH_NEEDS_AN_OPTIONAL_FOUND.at(&[&self.show(&opt.ty)]),
                );
            }
            return self.err_expr(span);
        };
        if let Some(lid) = self.cx.try_extent {
            // Hoisted ahead of the left side of `??`, inside its loop: a
            // `break` from a value block would not reach the loop in C++.
            let slot = self.temp(opt.ty.clone());
            self.cx.pending.push(Stmt::Let(slot, Some(opt)));
            let held = self.local(slot, span);
            let is_some = self.mk_is_some(held.clone(), Ty::Bool, span);
            let is_none = self.mk_unary(UnOp::Not, is_some, Ty::Bool, span);
            let leave = Block {
                stmts: vec![Stmt::Break(Some(lid))],
            };
            self.cx
                .pending
                .push(Stmt::If(is_none, leave, Block::default()));
            return self.mk_unwrap(held, false, *inner_ty, span);
        }
        let ret_ty = self
            .cx
            .ret
            .clone()
            .unwrap_or_else(|| Ty::opt((*inner_ty).clone()));
        self.do_expr(Some(&inner_ty), span, |lw, out| {
            let slot = lw.temp(opt.ty.clone());
            out.push(Stmt::Let(slot, Some(opt)));
            let held = lw.local(slot, span);
            let is_some = lw.mk_is_some(held.clone(), Ty::Bool, span);
            let is_none = lw.mk_unary(UnOp::Not, is_some, Ty::Bool, span);
            let ret_none = lw.mk(ExprKind::None, ret_ty.clone(), span);
            out.push(Stmt::If(
                is_none,
                Block {
                    stmts: vec![Stmt::Return(Some(ret_none))],
                },
                Block::default(),
            ));
            let val = lw.mk_unwrap(held, false, (*inner_ty).clone(), span);
            out.push(Stmt::Give(val));
        })
    }
}
