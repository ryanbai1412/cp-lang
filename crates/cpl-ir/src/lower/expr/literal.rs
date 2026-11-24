//! Literals: maps, lists, strings and the bytes of literal text.

use super::*;
use cpl_diag::msgs;

impl<'a> Lowerer<'a> {
    pub(in crate::lower) fn map_lit(
        &mut self,
        entries: &[(ast::Expr, ast::Expr)],
        hint: Option<&Ty>,
        span: Span,
    ) -> Expr {
        let (kh, vh, hashed) = match hint {
            Some(Ty::Map(k, v)) => (Some((**k).clone()), Some((**v).clone()), false),
            Some(Ty::HashMap(k, v)) => (Some((**k).clone()), Some((**v).clone()), true),
            _ => (None, None, false),
        };
        let mut pairs = Vec::with_capacity(entries.len());
        let mut kt = kh;
        let mut vt = vh;
        for (k, v) in entries {
            let ke = match &kt {
                Some(t) => self.expect(k, t),
                None => {
                    let e = self.expr(k, None);
                    kt = Some(e.ty.clone());
                    e
                }
            };
            let ve = match &vt {
                Some(t) => self.expect(v, t),
                None => {
                    let e = self.expr(v, None);
                    vt = Some(e.ty.clone());
                    e
                }
            };
            pairs.push((ke, ve));
        }
        let k_ty = kt.unwrap_or(Ty::Int);
        let v_ty = vt.unwrap_or(Ty::Int);
        let map_ty = if hashed {
            Ty::HashMap(Box::new(k_ty), Box::new(v_ty.clone()))
        } else {
            Ty::Map(Box::new(k_ty), Box::new(v_ty.clone()))
        };
        self.do_expr(Some(&map_ty), span, |lw, out| {
            let slot = lw.temp(map_ty.clone());
            out.push_let(
                slot,
                lw.mk(ExprKind::Tuple(Vec::new()), map_ty.clone(), span),
            );
            for (k, v) in pairs {
                let place = Place {
                    kind: PlaceKind::Index(
                        Box::new(Place {
                            kind: PlaceKind::Local(slot),
                            ty: map_ty.clone(),
                            span: lw.span(span),
                        }),
                        vec![k],
                    ),
                    ty: v_ty.clone(),
                    span: lw.span(span),
                };
                out.push(Stmt::Assign(place, AssignOp::Set, v));
            }
            let res = lw.local(slot, span);
            out.push_give(res);
        })
    }

    /// The bytes of literal text. `Char` is a byte and `Str` is bytes, so
    /// text outside ASCII has no value.
    pub(in crate::lower) fn ascii(&mut self, text: &str, span: Span) -> Option<Vec<u8>> {
        match text.chars().find(|c| !c.is_ascii()) {
            None => Some(text.as_bytes().to_vec()),
            Some(c) => {
                let message = cpl_diag::msgs::ir::NOT_ASCII_CHAR_ONE.at(&[&c]);
                self.report_said(span, message);
                self.help("only ASCII can be written in a literal; other bytes come from input");
                None
            }
        }
    }

    pub(in crate::lower) fn string(&mut self, pieces: &[ast::StrPiece], span: Span) -> Expr {
        let mut out: Vec<FmtPiece> = Vec::new();
        for piece in pieces {
            match piece {
                ast::StrPiece::Text(text) => {
                    let Some(bytes) = self.ascii(text, span) else {
                        return self.err_expr(span);
                    };
                    // `{{` and `}}` are how a brace is written.
                    let mut plain = Vec::with_capacity(bytes.len());
                    let mut i = 0;
                    while i < bytes.len() {
                        plain.push(bytes[i]);
                        let doubled =
                            matches!(bytes[i], b'{' | b'}') && bytes.get(i + 1) == Some(&bytes[i]);
                        i += if doubled { 2 } else { 1 };
                    }
                    match out.last_mut() {
                        Some(FmtPiece::Text(prev)) => prev.extend(plain),
                        _ => out.push(FmtPiece::Text(plain)),
                    }
                }
                ast::StrPiece::Expr(e) => {
                    let value = self.expr(e, None);
                    if let Some(arg) = self.fmt_arg(value, crate::FLOAT_DIGITS, e.span) {
                        out.push(FmtPiece::Value(arg));
                    }
                }
                ast::StrPiece::Spec(e, spec) => {
                    let value = self.expr(e, None);
                    let fixed = match spec.kind {
                        ast::FmtKind::Fixed(places) => Some(places),
                        _ => None,
                    };
                    let written = spec.to_string();
                    let help = "use `:.6f` for an F64, `:04` / `:x` / `:b` for an Int";
                    let (value, kind) = match (&value.ty, fixed, spec.kind) {
                        (Ty::Error, _, _) => continue,
                        (Ty::Float, Some(places), _) => (value, NumKind::Fixed(places)),
                        (Ty::Int, Some(places), _) => {
                            let at = value.span;
                            let cast = ExprKind::Cast(Cast::IntToFloat, Box::new(value));
                            (self.mk(cast, Ty::Float, at), NumKind::Fixed(places))
                        }
                        (Ty::Int, None, ast::FmtKind::Hex) => (value, NumKind::Hex),
                        (Ty::Int, None, ast::FmtKind::Bin) => (value, NumKind::Bin),
                        (Ty::Int, None, _) => (value, NumKind::Dec),
                        (Ty::Float, None, _) => {
                            let message = cpl_diag::msgs::ir::DOES_NOT_SAY_HOW.at(&[&written]);
                            self.report_said(e.span, message);
                            self.help(help);
                            continue;
                        }
                        (ty, _, _) => {
                            self.report(
                                e.span,
                                msgs::ir::TYPE_MISMATCH_IS_NOT_A_NUMBER,
                                &[&written, &self.show(ty)],
                            );
                            self.help(help);
                            continue;
                        }
                    };
                    let fmt = NumFmt {
                        zero: spec.zero,
                        width: spec.width,
                        kind,
                    };
                    out.push(FmtPiece::Number(value, fmt));
                }
            }
        }
        match out.as_slice() {
            [] => self.mk(ExprKind::Str(Vec::new()), Ty::Str, span),
            [FmtPiece::Text(text)] => self.mk(ExprKind::Str(text.clone()), Ty::Str, span),
            _ => self.mk(ExprKind::Format(out), Ty::Str, span),
        }
    }

    /// A `Str` or a `Char` as a `Str`, for `+`.
    pub(in crate::lower) fn as_str(&mut self, value: Expr, span: Span) -> Expr {
        match value.ty {
            Ty::Str | Ty::Error => value,
            Ty::Char => {
                let at = value.span;
                Expr {
                    kind: ExprKind::Format(vec![FmtPiece::Value(FmtArg {
                        value,
                        digits: crate::FLOAT_DIGITS,
                    })]),
                    ty: Ty::Str,
                    span: at,
                }
            }
            _ => {
                self.mismatch(span, &Ty::Str, &value.ty);
                self.err_expr(span)
            }
        }
    }

    pub(crate) fn list(&mut self, items: &[ast::Expr], hint: Option<&Ty>, span: Span) -> Expr {
        // `BeatsSegTree([5, 1, 4])`: a literal given for a `[D]T` whose `D`
        // stands for a list's positions is that list, and sets `D`'s size.
        if let Some(want @ Ty::Array { dims, elem }) = hint {
            if matches!(dims[..], [Dim::Pos(d)] if self.list_domains.contains(&d)) {
                let list_ty = Ty::list(self.plain_ty(elem));
                let list = self.list(items, Some(&list_ty), span);
                return self.coerce(list, want, span);
            }
        }
        match hint {
            Some(ty @ (Ty::List(elem) | Ty::Set(elem) | Ty::HashSet(elem))) => {
                let values = items.iter().map(|item| self.expect(item, elem)).collect();
                self.mk(ExprKind::Tuple(values), ty.clone(), span)
            }
            // `let primes: [Small]Int = [2, 3, 5]`: one value per position,
            // which is checked when the domain's size is known: at run time
            // (or statically for a plain enum domain `[Side]T`).
            Some(ty @ Ty::Array { dims, elem })
                if matches!(
                    dims[..],
                    [Dim::Pos(_) | Dim::Bound(_) | Dim::Enum(_) | Dim::Int]
                ) =>
            {
                let (dim, len) = match dims[0] {
                    Dim::Pos(d) => (
                        DimSpec::Pos(d),
                        self.mk(ExprKind::DomainLen(d), Ty::Int, span),
                    ),
                    Dim::Bound(d) => {
                        let len = self.mk(ExprKind::DomainLen(d), Ty::Int, span);
                        let one = self.mk(ExprKind::Int(1), Ty::Int, span);
                        let gaps = ExprKind::Binary(BinOp::Add, Box::new(len), Box::new(one));
                        (DimSpec::Bound(d), self.mk(gaps, Ty::Int, span))
                    }
                    Dim::Enum(e) => {
                        let n = self.enums[e].variants.len() as i64;
                        (DimSpec::Enum(e), self.mk(ExprKind::Int(n), Ty::Int, span))
                    }
                    Dim::Int => {
                        let n = self.mk(ExprKind::Int(items.len() as i64), Ty::Int, span);
                        (DimSpec::Int(Box::new(n.clone())), n)
                    }
                    Dim::Sym(_) | Dim::IntFrom(_) => unreachable!("matched above"),
                };
                // `domain V = 0..3` writes its count, so `[V]Int = [0, 2]` is
                // as wrong as `[0..3]Int = [0, 2]` (R04.8.10).
                let cells = match dims[0] {
                    Dim::Pos(d) => self
                        .domain_lens
                        .get(&d)
                        .filter(|(l, h)| l == h)
                        .map(|l| l.0),
                    Dim::Bound(d) => self
                        .domain_lens
                        .get(&d)
                        .filter(|(l, h)| l == h)
                        .map(|l| l.0 + 1),
                    _ => None,
                };
                if let Some(cells) = cells.filter(|&c| c != items.len() as i64) {
                    self.report(
                        span,
                        msgs::ir::TYPE_MISMATCH_THIS_ARRAY_HAS_BUT,
                        &[
                            &crate::lower::stmt::plural(cells, "cell"),
                            &crate::lower::stmt::plural(items.len() as i64, "value"),
                        ],
                    );
                    return self.err_expr(span);
                }
                let index_ty = match dims[0] {
                    Dim::Bound(d) => Ty::Bound(d),
                    Dim::Pos(d) => Ty::Pos(d),
                    Dim::Enum(e) => Ty::Enum(e),
                    Dim::Sym(_) | Dim::Int | Dim::IntFrom(_) => Ty::Int,
                };
                let values: Vec<Expr> = items.iter().map(|item| self.expect(item, elem)).collect();
                let ty = ty.clone();
                let elem = (**elem).clone();
                self.do_expr(Some(&ty), span, |lw, out| {
                    let count = lw.mk(ExprKind::Int(values.len() as i64), Ty::Int, span);
                    let wrong = lw.mk_cmp(CmpOp::Ne, len, count, Ty::Bool, span);
                    let trap = Stmt::Trap(
                        "an array literal needs one value per position".to_string(),
                        lw.span(span),
                    );
                    out.push(Stmt::If(
                        wrong,
                        Block { stmts: vec![trap] },
                        Block::default(),
                    ));
                    let slot = lw.temp(ty.clone());
                    let blank = lw.mk(ExprKind::Fill(vec![dim], None), ty.clone(), span);
                    out.push_let(slot, blank);
                    for (i, value) in values.into_iter().enumerate() {
                        let at = lw.mk(ExprKind::Int(i as i64), index_ty.clone(), span);
                        let whole = Place {
                            kind: PlaceKind::Local(slot),
                            ty: ty.clone(),
                            span,
                        };
                        let cell = Place {
                            kind: PlaceKind::Index(Box::new(whole), vec![at]),
                            ty: elem.clone(),
                            span,
                        };
                        out.push(Stmt::Assign(cell, AssignOp::Set, value));
                    }
                    out.push_give(lw.local(slot, span));
                })
            }
            Some(ty @ (Ty::Map(..) | Ty::HashMap(..))) if items.is_empty() => {
                self.mk(ExprKind::Tuple(Vec::new()), ty.clone(), span)
            }
            _ => {
                let Some((first, rest)) = items.split_first() else {
                    self.report(span, msgs::ir::TYPE_MISMATCH_CAN_T_TELL_WHAT_2, &[]);
                    self.help("say the type where it is stored: `var xs: List<Int> = []`");
                    return self.err_expr(span);
                };
                let has_none = items
                    .iter()
                    .any(|it| matches!(it.kind, ast::ExprKind::None));
                if has_none {
                    let non_none = items
                        .iter()
                        .find(|it| !matches!(it.kind, ast::ExprKind::None));
                    if let Some(sample) = non_none {
                        let sample_e = self.expr(sample, None);
                        let elem = match &sample_e.ty {
                            Ty::Opt(_) => sample_e.ty.clone(),
                            other => Ty::opt(other.clone()),
                        };
                        let values = items.iter().map(|item| self.expect(item, &elem)).collect();
                        return self.mk(ExprKind::Tuple(values), Ty::list(elem), span);
                    }
                }
                let first = self.expr(first, None);
                let elem = first.ty.clone();
                let mut values = vec![first];
                for item in rest {
                    values.push(self.expect(item, &elem));
                }
                self.mk(ExprKind::Tuple(values), Ty::list(elem), span)
            }
        }
    }
}
