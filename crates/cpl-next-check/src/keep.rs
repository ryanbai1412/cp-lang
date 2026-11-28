//! `x min= e` and `x max= e`, as a statement and as a condition (R01.6.4, R02.4.6).
use super::*;

impl Cx<'_> {
    /// The place an assignment writes, after checking that it may be written.
    pub(super) fn assign_place(&mut self, target: &next::Expr) -> TExpr {
        let span = self.at(target);
        let place = self.place(target);
        // A place that is the error was said where it was made.
        if self.is_error(&place.ty) {
            return place;
        }
        let mut root = &place;
        let mut cache = false;
        while let TK::Field(b, i) = &root.kind {
            if let Ty::Struct(sid, ..) = self.shallow(&b.ty) {
                {
                    let sd = self.d.s(sid);
                    cache |= sd.cached.get(*i).copied().unwrap_or(false);
                    if !cache && !sd.var_fields.get(*i).copied().unwrap_or(true) {
                        let (st, f) = (sd.name.clone(), self.d.text(sd.fields[*i].0).to_string());
                        self.err(
                            span,
                            Msg::verdict(
                                cpl_diag::codes::ASSIGN_FIXED_FIELD,
                                format!(
                                    "field `{st}.{f}` is fixed (not `var`) and cannot be assigned"
                                ),
                            ),
                        );
                    }
                }
            }
            root = b;
        }
        if let Some(name) = self.let_root(&place).filter(|_| !cache) {
            let msg = Msg::verdict(
                cpl_diag::codes::ASSIGN_TO_LET,
                format!("`{name}` is bound by `let` and cannot be assigned; declare it with `var`"),
            );
            self.err(span, self.with_var_fix(&place, &name, msg));
        }
        place
    }

    /// `place min= value` (`max=` when `max`) as a `Bool`: true when the place strictly
    /// improved. A `None` place takes a present value; a `None` value changes nothing.
    pub(super) fn keep(
        &mut self,
        place: TExpr,
        max: bool,
        value: &next::Expr,
        span: Span,
    ) -> TExpr {
        let mk = |kind, ty| TExpr { kind, ty, span };
        let op = if max { CmpOp::Gt } else { CmpOp::Lt };
        let v = self.expr(value, Some(&place.ty));
        let v = self.coerce(v, &place.ty);
        let tmp = self.bind("$keep", place.ty.clone());
        let read = mk(TK::Var(tmp), place.ty.clone());
        let better = match self.optional_of(&place.ty) {
            Some(inner) => {
                let part = |e: &TExpr, i, ty| mk(TK::Field(Box::new(e.clone()), i), ty);
                let (has, had) = (part(&read, 0, Ty::Bool), part(&place, 0, Ty::Bool));
                let (new, old) = (part(&read, 1, inner.clone()), part(&place, 1, inner));
                let lt = self.ordered(op, new, old, span);
                let over = TK::If(
                    Box::new(had),
                    Box::new(lt),
                    Box::new(mk(TK::Bool(true), Ty::Bool)),
                );
                let no = mk(TK::Bool(false), Ty::Bool);
                mk(
                    TK::If(Box::new(has), Box::new(mk(over, Ty::Bool)), Box::new(no)),
                    Ty::Bool,
                )
            }
            None => self.ordered(op, read.clone(), place.clone(), span),
        };
        let yes = Some(Box::new(mk(TK::Bool(true), Ty::Bool)));
        let set = mk(TK::Block(vec![TStmt::Assign(place, read)], yes), Ty::Bool);
        let no = mk(TK::Bool(false), Ty::Bool);
        let pick = mk(
            TK::If(Box::new(better), Box::new(set), Box::new(no)),
            Ty::Bool,
        );
        mk(
            TK::Block(vec![TStmt::Let(tmp, v)], Some(Box::new(pick))),
            Ty::Bool,
        )
    }

    /// `place min= value` where a value is wanted: the statement, checked as any assignment is,
    /// leaves its `Bool` in a new variable.
    pub(super) fn keep_expr(
        &mut self,
        target: &next::Expr,
        max: bool,
        value: &next::Expr,
        span: Span,
    ) -> TExpr {
        let mk = |kind, ty| TExpr { kind, ty, span };
        let t = self.bind("$kept", Ty::Bool);
        let mut out = vec![TStmt::Let(t, mk(TK::Bool(false), Ty::Bool))];
        self.b.kept = Some(t);
        self.write(target, span, &mut out, |cx, place| {
            let k = cx.keep(place, max, value, span);
            Self::kept(cx.b.kept, k)
        });
        self.b.kept = None;
        mk(
            TK::Block(out, Some(Box::new(mk(TK::Var(t), Ty::Bool)))),
            Ty::Bool,
        )
    }

    /// A checked `min=` / `max=` as a statement: kept in `into` when it is a condition.
    pub(super) fn kept(into: Option<VarRef>, k: TExpr) -> TStmt {
        match into {
            Some(t) => {
                let span = k.span;
                TStmt::Assign(
                    TExpr {
                        kind: TK::Var(t),
                        ty: Ty::Bool,
                        span,
                    },
                    k,
                )
            }
            None => TStmt::Expr(k),
        }
    }
}
