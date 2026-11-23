//! Input formats for `read()` (R10.2): which types have one, and
//! `read<List<T>>()`, a count then that many `T`s, so `[D]List<T>` reads
//! `D`-many count-prefixed rows (R10.2.3).

use super::*;

impl<'a> Lowerer<'a> {
    /// A count, then that many `each`, as a `List`.
    pub(super) fn counted_list(&mut self, each: Spec) -> Spec {
        let span = Span::default();
        let list = Ty::list(each.ty());
        let call = self.do_expr(Some(&list), span, |lw, out| {
            let place = |lw: &Self, slot, ty: &Ty| Place {
                kind: PlaceKind::Local(slot),
                ty: ty.clone(),
                span: lw.span(span),
            };
            let left = lw.temp(Ty::Int);
            let count = lw.mk(
                ExprKind::Read(Box::new(Spec::Plain(Ty::Int))),
                Ty::Int,
                span,
            );
            out.push(Stmt::Let(left, Some(count)));
            let acc = lw.temp(list.clone());
            let empty = lw.mk(ExprKind::Tuple(Vec::new()), list.clone(), span);
            out.push(Stmt::Let(acc, Some(empty)));
            let lid = lw.open_loop(None);
            let item = lw.read(each, span);
            let push =
                ExprKind::Mutate(Mutation::Push, Box::new(place(lw, acc, &list)), vec![item]);
            let one = lw.int_lit(1, span);
            let body = vec![
                Stmt::Expr(lw.mk(push, Ty::Unit, span)),
                Stmt::Assign(place(lw, left, &Ty::Int), AssignOp::Binary(BinOp::Sub), one),
            ];
            lw.cx.loops.pop();
            let (rest, zero) = (lw.local(left, span), lw.int_lit(0, span));
            let more = lw.mk(
                ExprKind::Cmp(CmpOp::Gt, Box::new(rest), Box::new(zero)),
                Ty::Bool,
                span,
            );
            out.push(Stmt::While(lid, more, Block { stmts: body }));
            out.push(Stmt::Give(lw.local(acc, span)));
        });
        Spec::Call(Box::new(call))
    }

    /// `Str(len: lo..=hi) of "01"`: the length's bounds, when `values`.
    pub(super) fn str_spec(&mut self, ty: &ast::Type, values: bool) -> Spec {
        let ast::TypeKind::ConstrainedStr { alphabet, len } = &ty.kind else {
            return Spec::Plain(Ty::Str);
        };
        let (lo, hi) = match len.as_deref().map(|len| self.resolve(len, values)) {
            Some(Spec::Int { lo, hi, .. }) => (lo, hi),
            _ => (None, None),
        };
        Spec::Str {
            lo,
            hi,
            alphabet: alphabet.clone(),
            text: self.source(ty.span),
        }
    }
}

/// The type inside `spec` that has no input format, if any.
pub(super) fn unreadable(spec: &Spec) -> Option<Ty> {
    match spec {
        Spec::Plain(ty) => match ty {
            Ty::Int
            | Ty::I128
            | Ty::U64
            | Ty::Float
            | Ty::Char
            | Ty::Str
            | Ty::Bool
            | Ty::Pos(_)
            | Ty::Error => None,
            Ty::Range(inner) if matches!(**inner, Ty::Int | Ty::Pos(_)) => None,
            other => Some(other.clone()),
        },
        Spec::Int { .. } | Spec::Str { .. } | Spec::Call(_) | Spec::Line => None,
        Spec::Parse(_, inner) => unreadable(inner),
        Spec::Fields(_, fields) => fields.iter().find_map(unreadable),
        Spec::Array { elem, .. } | Spec::List(elem) => unreadable(elem),
        Spec::Opt(inner, absent) => match (&**inner, absent) {
            (Spec::Plain(Ty::Int | Ty::Pos(_)) | Spec::Int { .. }, Some(_)) => None,
            _ => Some(spec.ty()),
        },
    }
}

impl<'a> Lowerer<'a> {
    /// The header an aside that reads input itself shares: when the solution
    /// reads its queries inside a loop and the fn reads them too, the
    /// top-level `let`s that read before that loop, if the fn uses a name
    /// they bind (`n`, `a`, `q`). Otherwise nothing: the fn reads it all.
    pub(super) fn shared_reads(
        &mut self,
        items: &[(u32, &'a ast::Stmt)],
        name: &str,
        gen: bool,
    ) -> usize {
        let reads = |this: &mut Self, (offset, stmt): &(u32, &ast::Stmt)| {
            this.offset = *offset;
            Self::reads_input(&this.source(stmt.span))
        };
        let item = |stmt: &ast::Stmt| {
            matches!(
                stmt.kind,
                ast::StmtKind::Fn(_) | ast::StmtKind::Struct(_) | ast::StmtKind::Impl { .. }
            )
        };
        if gen {
            return 0;
        }
        let end = (0..items.len())
            .find(|&i| {
                let stmt = items[i].1;
                !item(stmt)
                    && !matches!(stmt.kind, ast::StmtKind::Let { .. })
                    && reads(self, &items[i])
            })
            .unwrap_or(items.len());
        let Some(last) = (0..end)
            .rev()
            .find(|&i| !item(items[i].1) && reads(self, &items[i]))
        else {
            return 0;
        };
        let mut bound = HashSet::new();
        for (_, stmt) in &items[..=last] {
            if let ast::StmtKind::Let { pat, .. } = &stmt.kind {
                Self::pattern_names(pat, &mut bound);
            }
        }
        // `fn brute(): for _ in 0..q: let i = read()` reads per query and
        // shares `n`; `fn brute(): let n = read()` reads everything itself.
        let per_query = items.iter().any(|(offset, stmt)| {
            let ast::StmtKind::Fn(decl) = &stmt.kind else {
                return false;
            };
            let ast::FnBody::Block(body) = &decl.body else {
                return false;
            };
            self.offset = *offset;
            let uses = |w: &str| bound.contains(w);
            decl.name.text == name
                && self
                    .source(stmt.span)
                    .split(|c: char| !(c.is_alphanumeric() || c == '_'))
                    .any(uses)
                && body
                    .stmts
                    .iter()
                    .find(|s| Self::reads_input(&self.source(s.span)))
                    .is_some_and(|s| !matches!(s.kind, ast::StmtKind::Let { .. }))
        });
        let used = per_query;
        if used {
            last + 1
        } else {
            0
        }
    }
}
