//! Lists a program writes, literals and comprehensions (R01.8.1), and map literals (R01.4.5).
use super::*;
use cpl_next_decls::decls::Role;

/// A list being filled: the local holding it, its element type, and the `@literal("[]")`
/// trait and fn it is pushed onto through.
#[derive(Clone)]
pub(super) struct Filling {
    read: TExpr,
    elem: Ty,
    tr: decls::TraitRef,
    push: String,
}

impl Cx<'_> {
    /// A new empty list in local `name`, the `@literal("[]")` trait's first fn with room for
    /// `room`, of the type `want` names when it has that trait and else a `List` of the element
    /// type it gives (R01.8.8). A literal's local is `$list`, which says its length is the
    /// pushes that follow.
    fn new_list(
        &mut self,
        name: &str,
        room: usize,
        want: Option<&Ty>,
        span: Span,
    ) -> Option<(Filling, TStmt)> {
        let list = self.list_struct(span)?;
        let tr = (self.syms.get("[]")).and_then(|op| self.rows.literal_trait(op));
        let fns = tr.map(|tr| self.methods.trait_fns(tr)).unwrap_or_default();
        let (Some(tr), [(new, nf), (push, pf), ..]) = (tr, &fns[..]) else {
            self.err(
                span,
                Msg::internal("no trait marked `@literal(\"[]\")` with two fns"),
            );
            return None;
        };
        let (new, push) = (
            self.items.text(*new).to_string(),
            self.items.text(*push).to_string(),
        );
        self.reach(*nf);
        self.reach(*pf);
        let own = match want.map(|w| self.shallow(w)) {
            Some(t @ Ty::Struct(s, ..))
                if s != list
                    && Some(s) != self.array_id()
                    && (self.impls().trait_impls(tr))
                        .any(|(_, im)| matches!(&im.for_, Ty::Struct(f, ..) if *f == s)) =>
            {
                Some(t)
            }
            _ => None,
        };
        let elem = match want.map(|w| self.shallow(w)) {
            Some(Ty::Struct(d, ts, _)) if d == list => ts[0].clone(),
            // A literal where `[K]T` is expected becomes one, so its elements are `T`s; when `T`
            // is a list the literal may be that `T` filling every cell (R04.8.10), so it waits.
            Some(Ty::Struct(d, ts, _))
                if Some(d) == self.array_id()
                    && !matches!(ts.last().map(|t| self.shallow(t)), Some(Ty::Struct(l, ..)) if l == list) =>
            {
                ts.last().cloned().unwrap_or_else(|| self.new_ty())
            }
            _ => self.new_ty(),
        };
        let elem = if own.is_some() { self.new_ty() } else { elem };
        let lty = own.unwrap_or_else(|| Ty::Struct(list, vec![elem.clone()], Vec::new()));
        let tr = decls::TraitRef {
            tr,
            args: vec![elem.clone()],
        };
        (self.b.obligations).push((lty.clone(), tr.clone(), span, Origin::Bound));
        let tmp = self.temp(name, lty.clone());
        let read = TExpr {
            kind: TK::Var(tmp),
            ty: lty.clone(),
            span,
        };
        let call = TK::CallTrait {
            ctx: self.trait_ctx(tr.tr, &new),
            tr: tr.clone(),
            method: new,
            recv: None,
            self_ty: Box::new(lty.clone()),
            targs: Box::new([]),
            sets: Box::default(),
            args: vec![TArg::Value(TExpr {
                kind: TK::Int(room as i64),
                ty: Ty::Int,
                span,
            })],
        };
        let init = TStmt::Let(
            tmp,
            TExpr {
                kind: call,
                ty: lty,
                span,
            },
        );
        Some((
            Filling {
                read,
                elem,
                tr,
                push,
            },
            init,
        ))
    }

    /// `x` pushed onto the list being filled.
    fn push_onto(&mut self, l: &Filling, x: &next::Expr, span: Span) -> TExpr {
        let v = self.expr(x, Some(&l.elem));
        self.push_value(l, v, span)
    }
    /// The checked `v` pushed onto the list being filled.
    fn push_value(&mut self, l: &Filling, v: TExpr, span: Span) -> TExpr {
        // Every element flows into the list, so its fns share one set, not the first one's.
        if matches!(self.shallow(&l.elem), Ty::Var(_)) {
            let j = self.join_ty(&v.ty);
            self.unify(&l.elem, &j);
        }
        let v = self.coerce(v, &l.elem);
        TExpr {
            kind: TK::CallTrait {
                ctx: self.trait_ctx(l.tr.tr, &l.push),
                tr: l.tr.clone(),
                method: l.push.clone(),
                recv: Some(Box::new(TArg::Mut(l.read.clone()))),
                self_ty: Box::new(l.read.ty.clone()),
                targs: Box::new([]),
                sets: Box::default(),
                args: vec![TArg::Value(v)],
            },
            ty: Ty::Unit,
            span,
        }
    }

    /// `[a, b]`: `with_room(2)` and a `push` per element (R01.8.8).
    pub(super) fn list_literal(
        &mut self,
        xs: &[next::Expr],
        want: Option<&Ty>,
        span: Span,
    ) -> TExpr {
        let cells = self.cells_wanted(xs, want);
        let want = cells.as_ref().or(want);
        self.b.scopes.push(HashMap::new());
        let e = match self.new_list("$list", xs.len(), want, span) {
            Some((l, init)) => {
                let mut stmts = vec![init];
                for x in xs {
                    let push = self.push_onto(&l, x, span);
                    stmts.push(TStmt::Expr(push));
                }
                filled(stmts, l, span)
            }
            None => error(span),
        };
        self.b.scopes.pop();
        e
    }

    /// A list of the checked `vs`, as a literal of them is.
    pub(super) fn list_of(&mut self, vs: Vec<TExpr>, span: Span) -> TExpr {
        match self.new_list("$list", vs.len(), None, span) {
            Some((l, init)) => {
                let mut stmts = vec![init];
                for v in vs {
                    let push = self.push_value(&l, v, span);
                    stmts.push(TStmt::Expr(push));
                }
                filled(stmts, l, span)
            }
            None => error(span),
        }
    }

    /// Where `[K]List<E>` is wanted a literal is one list per cell, unless each element could be
    /// an `E`, making it one `List<E>` that fills every cell (R04.8.10).
    fn cells_wanted(&self, xs: &[next::Expr], want: Option<&Ty>) -> Option<Ty> {
        let (list, array) = (self.role_structs.role_struct(Role::List)?, self.array_id()?);
        let Ty::Struct(a, ts, _) = self.shallow(want?) else {
            return None;
        };
        let cell = ts.last().filter(|_| a == array)?.clone();
        let Ty::Struct(l, es, _) = self.shallow(&cell) else {
            return None;
        };
        let fill = xs
            .iter()
            .all(|x| es.first().is_none_or(|e| self.could_be(x, e)));
        (l == list && !fill).then(|| Ty::Struct(list, vec![cell], Vec::new()))
    }
    /// Whether `x` could be a `t`: a list literal is one only when `t` is a list or an array.
    fn could_be(&self, x: &next::Expr, t: &Ty) -> bool {
        let mut x = x.clone();
        while let next::Expr::Paren(p) = &x {
            match p.inner() {
                Some(inner) => x = inner,
                None => return true,
            }
        }
        let next::Expr::List(ys) = &x else {
            return true;
        };
        match self.shallow(t) {
            Ty::Struct(d, ts, _)
                if Some(d) == self.role_structs.role_struct(Role::List)
                    || Some(d) == self.array_id() =>
            {
                (ys.elems()).all(|y| ts.last().is_none_or(|e| self.could_be(&y, e)))
            }
            Ty::Struct(..) | Ty::Int | Ty::Dom(_) => false,
            _ => true,
        }
    }

    /// `[e for p in s if c, q in t]` and the generator `(e for ..)`: a new list, then the loop
    /// its clauses write, outermost first, each condition an `if` around the rest, pushing `e`
    /// innermost (R01.8.1).
    pub(super) fn comp_of(
        &mut self,
        body: Option<next::Expr>,
        clauses: Vec<next::CompClause>,
        want: Option<&Ty>,
        span: Span,
    ) -> TExpr {
        let Some(body) = body else {
            return error(span);
        };
        self.b.scopes.push(HashMap::new());
        let e = match self.new_list("$comp", 0, want, span) {
            Some((l, init)) => {
                let mut stmts = vec![init];
                self.comp_clauses(&l, &body, &clauses, span, &mut stmts);
                filled(stmts, l, span)
            }
            None => error(span),
        };
        self.b.scopes.pop();
        e
    }
    /// A table's fill, `[D] k => e`: a new list, then `body` pushed for each element of the
    /// checked `walk`, its parts bound by `pats`, as a comprehension's one clause would.
    pub(super) fn fill_over(
        &mut self,
        pats: &[next::Pattern],
        walk: TExpr,
        body: &next::ArmBody,
        want: Option<&Ty>,
        span: Span,
    ) -> TExpr {
        self.b.scopes.push(HashMap::new());
        let e = match self.new_list("$fill", 0, want, span) {
            Some((l, init)) => {
                let mut stmts = vec![init];
                let each = |cx: &mut Self, out: &mut Vec<TStmt>| {
                    let v = cx.arm_body(body, Some(&l.elem));
                    let push = cx.push_value(&l, v, span);
                    out.push(TStmt::Expr(push));
                };
                self.for_over(pats, walk, each, span, &mut stmts);
                filled(stmts, l, span)
            }
            None => error(span),
        };
        self.b.scopes.pop();
        e
    }
    /// The loop of `clauses`, outermost first, whose innermost statement pushes `body` onto
    /// the list being filled.
    fn comp_clauses(
        &mut self,
        l: &Filling,
        body: &next::Expr,
        clauses: &[next::CompClause],
        span: Span,
        out: &mut Vec<TStmt>,
    ) {
        let Some((c, rest)) = clauses.split_first() else {
            let push = self.push_onto(l, body, span);
            out.push(TStmt::Expr(push));
            return;
        };
        let (Some(pat), Some(iter)) = (c.pattern(), c.iter()) else {
            return;
        };
        let conds: Vec<next::Condition> = c.filters().collect();
        self.for_loop(
            &pat,
            &iter,
            |cx, out| cx.comp_filters(l, &conds, body, rest, span, out),
            span,
            out,
        );
    }
    /// A clause's conditions, each an `if` around the rest.
    fn comp_filters(
        &mut self,
        l: &Filling,
        conds: &[next::Condition],
        body: &next::Expr,
        rest: &[next::CompClause],
        span: Span,
        out: &mut Vec<TStmt>,
    ) {
        match conds.split_first() {
            None => self.comp_clauses(l, body, rest, span, out),
            Some((c, more)) => self.if_then(
                c,
                |cx, out| cx.comp_filters(l, more, body, rest, span, out),
                span,
                out,
            ),
        }
    }
}

fn filled(stmts: Vec<TStmt>, l: Filling, span: Span) -> TExpr {
    let ty = l.read.ty.clone();
    TExpr {
        kind: TK::Block(stmts, Some(Box::new(l.read))),
        ty,
        span,
    }
}

fn error(span: Span) -> TExpr {
    TExpr {
        kind: TK::Error,
        ty: Ty::Error,
        span,
    }
}

impl Cx<'_> {
    /// `[k: v, ..]`: a new map, of the struct `want` names when it has a `new()` and else of the
    /// `@role(map)` one, and each entry written through its `IndexSet` (R01.4.5).
    pub(super) fn map_literal(
        &mut self,
        m: &next::MapExpr,
        want: Option<&Ty>,
        span: Span,
    ) -> TExpr {
        // A struct a map literal can start empty: a `new()`, or a default for every field.
        let empty = |d: StructId| {
            let sd = self.items.s(d);
            match sd.ctor {
                Some(c) => self.items.f(c).params.is_empty(),
                None => sd.defaults.iter().all(Option::is_some),
            }
        };
        let (sid, targs, sets) = match want.map(|w| self.shallow(w)) {
            Some(Ty::Struct(d, ts, ss)) if empty(d) => (d, ts, ss),
            _ => match self
                .role_structs
                .role_struct(Role::Map)
                .filter(|&d| empty(d))
            {
                Some(d) => {
                    let sd = self.items.s(d);
                    let (n, m) = (sd.generics.len(), sd.n_sets);
                    let targs = (0..n).map(|_| self.new_ty()).collect();
                    (d, targs, (0..m).map(|_| self.new_set()).collect())
                }
                None => {
                    self.err(span, Msg::internal("no `@role(map)` struct starts empty"));
                    return Self::error_node(span);
                }
            },
        };
        let ty = Ty::Struct(sid, targs.clone(), sets.clone());
        let mk = |kind, ty| TExpr { kind, ty, span };
        let new = match self.items.s(sid).ctor {
            Some(c) => {
                let f = self.fnref(c, targs, sets);
                mk(
                    TK::Call {
                        f,
                        recv: None,
                        args: vec![],
                    },
                    ty.clone(),
                )
            }
            None => self.construct(sid, targs, sets, &[], span, None),
        };
        self.b.scopes.push(HashMap::new());
        let var = self.temp("$map", ty.clone());
        match var {
            VarRef::Local(l) => self.b.locals[l].mutable = true,
            VarRef::Global(g) => self.globals[g].mutable = true,
        }
        let mut stmts = vec![TStmt::Let(var, new)];
        for entry in m.entries() {
            let (Some(k), Some(v)) = (entry.key(), entry.value()) else {
                continue;
            };
            let recv = mk(TK::Var(var), ty.clone());
            let Some(set) = self.index_set(recv, &k, &v, span) else {
                let t = self.show(&ty);
                let text = format!("a map literal builds a `{t}`, which has no `[]=` write");
                self.err(span, Msg::verdict(cpl_diag::codes::TYPE_MISMATCH, text));
                break;
            };
            stmts.push(TStmt::Expr(set));
        }
        self.b.scopes.pop();
        mk(
            TK::Block(stmts, Some(Box::new(mk(TK::Var(var), ty.clone())))),
            ty,
        )
    }
}
