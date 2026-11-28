//! Assignment, one path for `=`, `op=`, `f=` and `min=`/`max=` (R01.6.4), and `min=`/`max=`
//! as a condition (R02.4.6).

use super::call::CallArg;
use super::*;

impl<'a> Cx<'a> {
    /// A write to the place `target` is, `step` the statement on the place: through the cell
    /// an index lends (R12.4.6), inside the region a lending call opens (`t.at(i) = v`), or on
    /// the place itself.
    pub(super) fn write(
        &mut self,
        target: &next::Expr,
        span: Span,
        out: &mut Vec<TStmt>,
        step: impl FnOnce(&mut Self, TExpr) -> TStmt,
    ) {
        match super::index::unparen(target.clone()) {
            next::Expr::Index(ix) => {
                let Some((root, keys)) = self.hoist_indices(&ix, out) else {
                    return;
                };
                let inner = move |cx: &mut Self, cell: TExpr, out: &mut Vec<TStmt>| {
                    let s = step(cx, cell);
                    out.push(s)
                };
                self.through_cell(root, keys, span, out, Box::new(inner));
            }
            next::Expr::Call(_) => {
                // `t.at(i) = v`: `at` lends a place; the write runs while it is lent.
                let at = self.at(target);
                self.b.allow_region = true;
                let call = self.expr(target, None);
                self.b.allow_region = false;
                let TK::Call { f, recv, args } = call.kind else {
                    return;
                };
                let Some((true, elem)) = self.items.f(f.decl).region.clone() else {
                    let code = cpl_diag::codes::TYPE_MISMATCH;
                    let text = "this call does not lend a place, so it cannot be assigned";
                    return self.err(span, Msg::verdict(code, text));
                };
                let elem = Subst {
                    tys: &f.targs,
                    sets: &f.sets,
                }
                .ty(&elem);
                self.b.scopes.push(HashMap::new());
                let var = self.temp("$lent", elem.clone());
                let lent = TExpr {
                    kind: TK::Var(var),
                    ty: elem,
                    span: at,
                };
                let s = step(self, lent);
                self.b.scopes.pop();
                let body = TExpr {
                    kind: TK::Block(vec![s], None),
                    ty: Ty::Unit,
                    span,
                };
                out.push(TStmt::Region {
                    callee: Callee::Fn(f),
                    recv,
                    args,
                    var,
                    lend: true,
                    body,
                });
            }
            _ => {
                let place = self.assign_place(target);
                let s = step(self, place);
                out.push(s);
            }
        }
    }
    /// `place = v` with `v` already read: `[]=` on an indexed container that has one (spec 12,
    /// W8), else the write through the cell.
    fn set_place(&mut self, target: &next::Expr, v: TExpr, span: Span, out: &mut Vec<TStmt>) {
        if let next::Expr::Index(ix) = super::index::unparen(target.clone()) {
            let Some((root, keys)) = self.hoist_indices(&ix, out) else {
                return;
            };
            let held = v.clone();
            let assign = move |cx: &mut Self, cell: TExpr, out: &mut Vec<TStmt>| {
                let ty = cell.ty.clone();
                let v = cx.coerce(held, &ty);
                out.push(TStmt::Assign(cell, v))
            };
            return self.cell_set(root, keys, v, span, out, Box::new(assign));
        }
        self.write(target, span, out, |cx, p| {
            let ty = p.ty.clone();
            let v = cx.coerce(v, &ty);
            TStmt::Assign(p, v)
        })
    }
    /// `p = v`, `p op= v` and `(a, b) = (b, a)` (R01.6.4): the indices, then the value, are
    /// read once, before the place is written.
    pub(super) fn assign(&mut self, a: &next::AssignStmt, out: &mut Vec<TStmt>) {
        let span = self.at(a);
        let (Some(target), Some(op), Some(value)) = (a.target(), a.op(), a.value()) else {
            return;
        };
        let index = match super::index::unparen(target.clone()) {
            next::Expr::Tuple(t) if matches!(op, next::AssignOp::Set) => {
                return self.assign_parts(&t, &value, span, out);
            }
            next::Expr::Index(ix) => Some(ix),
            _ => None,
        };
        let Some(ix) = index else {
            let target_at = self.at(&target);
            return self.write(&target, span, out, |cx, p| {
                let v = match &op {
                    next::AssignOp::Fn(_) => cx.expr(&value, None),
                    _ => cx.expr(&value, Some(&p.ty)),
                };
                let v = match &op {
                    next::AssignOp::Set => v,
                    how => cx.combine(how, p.clone(), v, target_at),
                };
                let ty = p.ty.clone();
                let v = cx.coerce(v, &ty);
                TStmt::Assign(p, v)
            });
        };
        let Some((root, keys)) = self.hoist_indices(&ix, out) else {
            return;
        };
        // The cell's type, so `out[v] = j..i` of cuts is a piece (R04.4.4).
        let cell = match &keys[..] {
            [_] => self.cell_of(&root.ty),
            _ => None,
        };
        // The value is read whole before the cell is written (R01.6.4).
        let want = cell
            .as_ref()
            .filter(|_| !matches!(op, next::AssignOp::Fn(_)));
        let v = self.hoist(&value, (want, true), "$rhs", out);
        match op {
            next::AssignOp::Set => {
                let held = v.clone();
                let assign = move |cx: &mut Self, cell: TExpr, out: &mut Vec<TStmt>| {
                    let ty = cell.ty.clone();
                    let v = cx.coerce(held, &ty);
                    out.push(TStmt::Assign(cell, v))
                };
                self.cell_set(root, keys, v, span, out, Box::new(assign))
            }
            how => {
                let target_at = self.at(&target);
                let inner = move |cx: &mut Self, cell: TExpr, out: &mut Vec<TStmt>| {
                    let r = cx.combine(&how, cell.clone(), v, target_at);
                    let ty = cell.ty.clone();
                    let r = cx.coerce(r, &ty);
                    out.push(TStmt::Assign(cell, r))
                };
                self.through_cell(root, keys, span, out, Box::new(inner))
            }
        }
    }
    /// The new value of `old` under a compound assignment: `p op= e` is `p op e`, and `p f= e`
    /// is `f(p, e)` with `f` any two-argument fn in scope (R01.6.4).
    fn combine(&mut self, how: &next::AssignOp, old: TExpr, v: TExpr, at: Span) -> TExpr {
        match how {
            next::AssignOp::Set => v,
            next::AssignOp::Op(b) => self.binary(*b, old, v, at),
            next::AssignOp::Fn(f) => {
                let want = old.ty.clone();
                let args = [
                    CallArg::held(None, old, at),
                    CallArg::held(None, v.clone(), v.span),
                ];
                self.call_named(f.text(), &[], &args, at, Some(&want))
            }
        }
    }
    /// `(a, b) = v`: the right side is built first, then each place takes its part. A written
    /// tuple's parts are each read into a copy of their own before any place is written, so
    /// no part borrows a place the writes change.
    fn assign_parts(
        &mut self,
        places: &next::TupleExpr,
        value: &next::Expr,
        span: Span,
        out: &mut Vec<TStmt>,
    ) {
        let places: Vec<next::Expr> = places.elems().collect();
        let held = |cx: &mut Self, k: usize, v: TExpr, out: &mut Vec<TStmt>| {
            let (ty, at) = (v.ty.clone(), v.span);
            let var = cx.temp(&format!("$parts{}_{k}", span.start), ty.clone());
            let kind = TK::Owned(Box::new(v));
            out.push(TStmt::Let(
                var,
                TExpr {
                    kind,
                    ty: ty.clone(),
                    span: at,
                },
            ));
            TExpr {
                kind: TK::Var(var),
                ty,
                span: at,
            }
        };
        let parts: Vec<TExpr> = match super::index::unparen(value.clone()) {
            next::Expr::Tuple(t) if t.elems().count() == places.len() => {
                let mut parts = Vec::new();
                for (k, (e, place)) in t.elems().zip(&places).enumerate() {
                    let want = self.named_place_ty(place);
                    let v = self.expr(&e, want.as_ref());
                    parts.push(held(self, k, v, out));
                }
                parts
            }
            _ => {
                let v = self.expr(value, None);
                let whole = held(self, places.len(), v, out);
                let tys = match self.field_tys(&whole.ty) {
                    Some((_, tys)) if tys.len() == places.len() => tys,
                    _ => {
                        if !self.mentions_error(&whole.ty) {
                            let (t, n) = (self.show(&whole.ty), places.len());
                            let code = cpl_diag::codes::TYPE_MISMATCH;
                            let text = format!("these {n} places take the parts of `{t}`, which has another number");
                            self.err(span, Msg::verdict(code, text));
                        }
                        return;
                    }
                };
                (tys.into_iter().enumerate())
                    .map(|(k, ty)| TExpr {
                        kind: TK::Field(Box::new(whole.clone()), k),
                        ty,
                        span: whole.span,
                    })
                    .collect()
            }
        };
        for (place, v) in places.iter().zip(parts) {
            let at = self.at(place);
            self.set_place(place, v, at, out);
        }
    }
    /// The type of a place written as a name, a local or a field of `self`: what its part of
    /// `(lines, left) = ([None], [0])` is built as, before any place is written.
    fn named_place_ty(&mut self, place: &next::Expr) -> Option<Ty> {
        let n = super::member::bare_name(place)?;
        if let Some((_, ty)) = self.lookup(&n) {
            return Some(ty);
        }
        let i = self.self_field(&n)?;
        let s = self.self_expr(Span::new(0, 0))?;
        self.field_tys(&s.ty)?.1.get(i).cloned()
    }
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
                    let sd = self.items.s(sid);
                    cache |= sd.cached.get(*i).copied().unwrap_or(false);
                    if !cache && !sd.var_fields.get(*i).copied().unwrap_or(true) {
                        let (st, f) = (
                            sd.name.clone(),
                            self.items.text(sd.fields[*i].0).to_string(),
                        );
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
        let tmp = self.temp("$keep", place.ty.clone());
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
        let t = self.temp("$kept", Ty::Bool);
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
