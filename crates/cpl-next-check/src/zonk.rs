//! Replacing solved variables in the finished tree.

use super::*;

impl<'a> Cx<'a> {
    pub(super) fn zonk_arg(&mut self, a: &mut TArg) {
        match a {
            TArg::Value(e) | TArg::Mut(e) => self.zonk_expr(e),
        }
    }
    pub(super) fn zonk_stmt(&mut self, s: &mut TStmt) {
        match s {
            TStmt::Let(_, e) | TStmt::Expr(e) | TStmt::Assert(e, ..) => self.zonk_expr(e),
            TStmt::Assign(a, b) | TStmt::While(a, b) => {
                self.zonk_expr(a);
                self.zonk_expr(b);
            }
            TStmt::For(_, a, b, c) => {
                self.zonk_expr(a);
                self.zonk_expr(b);
                self.zonk_expr(c);
            }
            TStmt::Return(e) => {
                if let Some(e) = e {
                    self.zonk_expr(e)
                }
            }
            TStmt::Break | TStmt::Continue | TStmt::Item(_) | TStmt::Checks(_) => {}
            TStmt::Cases(count, body) => {
                if let Some(c) = count {
                    self.zonk_expr(c)
                }
                body.iter_mut().for_each(|s| self.zonk_stmt(s));
            }
            TStmt::Yield(e) | TStmt::LendOut(e) => self.zonk_expr(e),
            TStmt::Region {
                callee,
                recv,
                args,
                body,
                ..
            } => {
                match callee {
                    Callee::Fn(f) => *f = self.zonk_fnref(f),
                    Callee::Trait(tr, _) => {
                        tr.args = tr.args.iter().map(|a| self.zonk(a)).collect()
                    }
                    Callee::Value(c) => self.zonk_expr(c),
                }
                if let Some(r) = recv {
                    self.zonk_arg(r);
                }
                args.iter_mut().for_each(|a| self.zonk_arg(a));
                self.zonk_expr(body);
            }
        }
    }
    pub(super) fn zonk_expr(&mut self, e: &mut TExpr) {
        e.ty = self.zonk(&e.ty);
        match &mut e.kind {
            TK::Field(b, _)
            | TK::Payload(b, _)
            | TK::Tag(b)
            | TK::Variant(_, b)
            | TK::Inject(b)
            | TK::Widen(b)
            | TK::Owned(b)
            | TK::Pack(b)
            | TK::Elem(b)
            | TK::Not(b)
            | TK::CharOf(b)
            | TK::CodeOf(b)
            | TK::Neg(b)
            | TK::ToU64(b)
            | TK::ToI128(b)
            | TK::DomOrd(b)
            | TK::Ordinal(b) => self.zonk_expr(b),
            TK::DomLen(t) | TK::DomCounted(t) => *t = self.zonk(t),
            TK::HasBound(t, r) => {
                *t = self.zonk(t);
                for a in &mut r.args {
                    *a = self.zonk(a);
                }
            }
            TK::DomFrom(t, b, _) | TK::CutOrd(t, b) | TK::DomNth(t, b) | TK::DomHas(t, b) => {
                *t = self.zonk(t);
                self.zonk_expr(b);
            }
            TK::Call { f, recv, args } => {
                let open = (0..f.targs.len()).find(|&i| {
                    matches!(self.shallow(&f.targs[i]), Ty::Var(_))
                        && self.only_in_result(f.decl, i)
                });
                if let Some(i) = open {
                    let at = self.b.item.at(e.span);
                    self.b.unsolved.push((at, f.decl, i));
                }
                *f = self.zonk_fnref(f);
                if !f.targs.is_empty() {
                    let at = self.b.item.at(e.span);
                    (self.generic_calls).push((self.b.decl, at, f.decl, f.targs.clone()));
                }
                if let Some(r) = recv {
                    self.zonk_arg(r);
                }
                args.iter_mut().for_each(|a| self.zonk_arg(a));
            }
            TK::CallTrait {
                tr,
                recv,
                self_ty,
                targs,
                sets,
                args,
                ..
            } => {
                tr.args = tr.args.iter().map(|a| self.zonk(a)).collect();
                **self_ty = self.zonk(self_ty);
                *targs = targs.iter().map(|a| self.zonk(a)).collect();
                *sets = sets.iter().map(|s| self.zonk_set(s)).collect();
                if let Some(r) = recv {
                    self.zonk_arg(r);
                }
                args.iter_mut().for_each(|a| self.zonk_arg(a));
            }
            TK::CallValue { callee, args } => {
                self.zonk_expr(callee);
                args.iter_mut().for_each(|a| self.zonk_arg(a));
            }
            TK::Struct(fs) => fs.iter_mut().for_each(|a| self.zonk_expr(a)),
            TK::Lambda { body, .. } => self.zonk_expr(body),
            TK::FnItem(f) => *f = self.zonk_fnref(f),
            TK::Bin(_, a, b) | TK::Cmp(_, a, b) | TK::Slot(a, b) => {
                self.zonk_expr(a);
                self.zonk_expr(b);
            }
            TK::If(a, b, c) => {
                self.zonk_expr(a);
                self.zonk_expr(b);
                self.zonk_expr(c);
            }
            TK::Block(ss, v) => {
                ss.iter_mut().for_each(|s| self.zonk_stmt(s));
                if let Some(v) = v {
                    self.zonk_expr(v)
                }
            }
            TK::Int(_)
            | TK::Float(_)
            | TK::Bool(_)
            | TK::Unit
            | TK::Unset
            | TK::Text(_)
            | TK::Var(_)
            | TK::ReadInt
            | TK::Itself(_)
            | TK::Error => {}
        }
    }
}
