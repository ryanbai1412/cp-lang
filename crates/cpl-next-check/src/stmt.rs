//! Statements, scopes, ranges and reads.

use super::call::CallArg;
use super::place::{place_binding, place_held, InPlace};
use super::*;
use cpl_next_ir::unbuilt::Unbuilt;
use cpl_next_syntax::SyntaxToken;

impl<'a> Cx<'a> {
    /// The innermost scope.
    pub(super) fn scope(&mut self) -> &mut HashMap<String, usize> {
        if self.b.scopes.is_empty() {
            self.b.scopes.push(HashMap::new());
        }
        let last = self.b.scopes.len() - 1;
        &mut self.b.scopes[last]
    }
    pub(super) fn add_local(&mut self, name: &str, ty: Ty) -> LocalId {
        self.b.locals.push(Local {
            name: name.to_string(),
            ty,
            top: false,
            mutable: false,
            fixed: false,
            let_at: None,
            synthetic: false,
        });
        let id = self.b.locals.len() - 1;
        self.scope().insert(name.to_string(), id);
        id
    }
    /// A new binding: a global at top level, a local in a fn.
    pub(super) fn bind(&mut self, name: &str, ty: Ty) -> VarRef {
        if self.b.decl.is_none() && self.b.scopes.len() == 1 {
            let local = Local {
                name: name.to_string(),
                ty,
                top: true,
                mutable: false,
                fixed: false,
                let_at: None,
                synthetic: false,
            };
            // A top-level value (R02.10.3) fills the global made for it, found by item and Sym.
            let sym = self.b.syms.of(&self.d.syms, name);
            let item = &self.b.item;
            let top = self
                .d
                .top_values
                .iter()
                .position(|t| t.item == *item && Some(t.name) == sym);
            let g = match top {
                Some(i) => {
                    self.globals[self.top_slots[i]] = local;
                    self.top_slots[i]
                }
                None => {
                    self.globals.push(local);
                    self.globals.len() - 1
                }
            };
            self.global_names.insert(name.to_string(), g);
            VarRef::Global(g)
        } else if self.b.decl.is_none() {
            // A binding inside a top-level block lives in the globals too, under its scope.
            self.globals.push(Local {
                name: name.to_string(),
                ty,
                top: false,
                mutable: false,
                fixed: false,
                let_at: None,
                synthetic: false,
            });
            let id = self.globals.len() - 1;
            self.scope().insert(name.to_string(), id);
            VarRef::Global(id)
        } else {
            VarRef::Local(self.add_local(name, ty))
        }
    }
    /// A slot the checker made and holds by its id: no name in scope reaches it. `role` is
    /// only what a trace shows; storage with no name is a temporary to ownership.
    pub(super) fn slot(&mut self, ty: Ty, role: &str) -> VarRef {
        let local = Local {
            name: role.to_string(),
            ty,
            top: self.b.decl.is_none() && self.b.scopes.len() == 1,
            mutable: false,
            fixed: false,
            let_at: None,
            synthetic: true,
        };
        match self.b.decl {
            None => {
                self.globals.push(local);
                VarRef::Global(self.globals.len() - 1)
            }
            Some(_) => {
                self.b.locals.push(local);
                VarRef::Local(self.b.locals.len() - 1)
            }
        }
    }
    pub(super) fn lookup(&self, name: &str) -> Option<(VarRef, Ty)> {
        for (depth, s) in self.b.scopes.iter().enumerate().rev() {
            if let Some(id) = s.get(name) {
                return Some(if self.b.decl.is_none() {
                    let _ = depth;
                    (VarRef::Global(*id), self.globals[*id].ty.clone())
                } else {
                    (VarRef::Local(*id), self.b.locals[*id].ty.clone())
                });
            }
        }
        // A field of `self` is nearer than a top-level binding, and only the program sees its own.
        if self.self_field(name).is_some() || self.b.module != self.d.program {
            return None;
        }
        self.global_names
            .get(name)
            .map(|id| (VarRef::Global(*id), self.globals[*id].ty.clone()))
    }
    pub(super) fn self_expr(&self, span: Span) -> Option<TExpr> {
        let id = self.b.self_local?;
        Some(TExpr {
            kind: TK::Var(VarRef::Local(id)),
            ty: self.b.locals[id].ty.clone(),
            span,
        })
    }

    /// `for pats in v`: a value walks by its `recipe`, a bound's walk, or its `each` (else
    /// `iter`); each item is bound by `pats` around `body`.
    fn walk_value(
        &mut self,
        mut call: TExpr,
        pats: &[next::Pattern],
        body: impl FnOnce(&mut Self, TExpr, &mut Vec<TStmt>),
        span: Span,
        out: &mut Vec<TStmt>,
    ) {
        if let Some(walk) = self.recipe_walk(&call) {
            let each = |cx: &mut Self, elem: Ty| cx.walk_each(pats, elem, body, span);
            return self.value_walk(walk, call, each, out);
        }
        // `for x in g.out(u)` through a bound on `g`'s type parameter; a bound's value
        // (`s.keys`) walks as the value it returns.
        let yields = |tr: &TraitRef, m: &str| {
            self.d
                .member(&self.d.t(tr.tr).methods, m)
                .is_some_and(|m| self.d.f(m).region.is_some())
        };
        call = match call.kind {
            TK::CallTrait {
                tr,
                method,
                recv: Some(recv),
                args,
                ..
            } if yields(&tr, &method) => {
                return self.trait_walk((tr, method, recv, args), pats, body, span, out);
            }
            kind => TExpr { kind, ..call },
        };
        if !matches!(&call.kind, TK::Call { f, .. } if self.d.f(f.decl).region.is_some()) {
            // A type without its own `each` walks through its `Seq` impl (R07.1.1).
            let rt = self.shallow(&call.ty);
            if let Ty::Param(p) = rt {
                return self.bound_walk(p, call, pats, body, span, out);
            }
            self.b.allow_region = true;
            let own = (self.d.iteration_own_walk()).filter(|(n, _)| {
                self.d
                    .sym(n)
                    .and_then(|sym| self.impls().lookup_member(&rt, sym))
                    .is_some()
            });
            let walk = (own.or_else(|| self.d.iteration_walk()))
                .map_or_else(String::new, |(n, _)| n.to_string());
            call = self.method_call(call, &walk, vec![], span, false);
            self.b.allow_region = false;
        }
        let TK::Call { f, recv, args } = call.kind else {
            return;
        };
        let Some((false, elem)) = self.d.f(f.decl).region.clone() else {
            return self.err(
                span,
                Msg::verdict(
                    cpl_diag::codes::TYPE_MISMATCH,
                    "this cannot be walked with `for`",
                ),
            );
        };
        let elem = self.result_of(&f, &elem);
        let (var, body) = self.walk_each(pats, elem, body, span);
        out.push(TStmt::Region {
            callee: Callee::Fn(f),
            recv,
            args,
            var,
            lend: false,
            body,
        });
    }
    /// `for x in g.out(u)` with `out` a generator of a bound on `g`'s type parameter: the
    /// impl each instance picks runs the body.
    fn trait_walk(
        &mut self,
        (tr, method, recv, args): (TraitRef, String, Box<TArg>, Vec<TArg>),
        pats: &[next::Pattern],
        body: impl FnOnce(&mut Self, TExpr, &mut Vec<TStmt>),
        span: Span,
        out: &mut Vec<TStmt>,
    ) {
        let Some(m) = self.d.member(&self.d.t(tr.tr).methods, &method) else {
            return self.err(
                span,
                Msg::internal("a bound's walk names no fn of its trait"),
            );
        };
        let Some((false, elem)) = self.d.f(m).region.clone() else {
            return self.err(
                span,
                Msg::verdict(
                    cpl_diag::codes::TYPE_MISMATCH,
                    "this cannot be walked with `for`",
                ),
            );
        };
        let (TArg::Value(r) | TArg::Mut(r)) = recv.as_ref();
        let mut tys = vec![r.ty.clone()];
        tys.extend(tr.args.iter().cloned());
        let elem = Subst {
            tys: &tys,
            sets: &[],
        }
        .ty(&elem);
        let (var, body) = self.walk_each(pats, elem, body, span);
        out.push(TStmt::Region {
            callee: Callee::Trait(tr, method),
            recv: Some(recv),
            args,
            var,
            lend: false,
            body,
        });
    }
    /// `for x in s` with `s` of a type parameter: its bound's `iter` walks it, and each
    /// instance picks the impl (R07.1.1).
    fn bound_walk(
        &mut self,
        p: usize,
        recv: TExpr,
        pats: &[next::Pattern],
        body: impl FnOnce(&mut Self, TExpr, &mut Vec<TStmt>),
        span: Span,
        out: &mut Vec<TStmt>,
    ) {
        let walk = self.d.iteration_walk().map(|(n, m)| (n.to_string(), m));
        let found = (self.bound_traits(p).into_iter())
            .find(|tr| Some(tr.tr) == self.d.roles.iteration)
            .zip(walk);
        let Some((tr, (walk, m))) = found else {
            let shown = self.show(&recv.ty);
            return self.err(
                span,
                Msg::verdict(
                    cpl_diag::codes::UNKNOWN_FIELD,
                    format!("`{shown}` is not bound by the iteration role's trait (R11.6.1)"),
                ),
            );
        };
        let Some((false, elem)) = self.d.f(m).region.clone() else {
            return self.err(
                span,
                Msg::verdict(
                    cpl_diag::codes::TYPE_MISMATCH,
                    "this cannot be walked with `for`",
                ),
            );
        };
        self.reach(m);
        let mut tys = vec![recv.ty.clone()];
        tys.extend(tr.args.iter().cloned());
        let elem = Subst {
            tys: &tys,
            sets: &[],
        }
        .ty(&elem);
        let (var, body) = self.walk_each(pats, elem, body, span);
        out.push(TStmt::Region {
            callee: Callee::Trait(tr.clone(), walk),
            recv: Some(Box::new(TArg::Value(recv))),
            args: Vec::new(),
            var,
            lend: false,
            body,
        });
    }
    /// One walk's body over items of type `elem`: a scope binding the item by `pats`, then
    /// `body`. Gives the item's variable and the checked body.
    pub(super) fn walk_each(
        &mut self,
        pats: &[next::Pattern],
        elem: Ty,
        body: impl FnOnce(&mut Self, TExpr, &mut Vec<TStmt>),
        span: Span,
    ) -> (VarRef, TExpr) {
        self.b.scopes.push(HashMap::new());
        let (var, open) = self.bind_each(pats, elem.clone(), span);
        let item = TExpr {
            kind: TK::Var(var),
            ty: elem,
            span,
        };
        let b = self.scoped_block(span, |cx, out| {
            out.extend(open);
            body(cx, item, out);
            None
        });
        self.b.scopes.pop();
        (var, b)
    }
    /// The variable a walk binds each item to, and what opens a destructured one: a name
    /// binds the item, `_` a slot nothing reads, and a tuple or record pattern, or several
    /// patterns (a table's `r, c`), a slot whose parts the body opens with.
    fn bind_each(&mut self, pats: &[next::Pattern], elem: Ty, span: Span) -> (VarRef, Vec<TStmt>) {
        if let [next::Pattern::Path(p)] = pats {
            if let Some(n) = super::pattern::bind_name(p) {
                let var = self.bind(n.text(), elem);
                return (var, Vec::new());
            }
        }
        if let [] | [next::Pattern::Wildcard(_)] = pats {
            return (self.temp("$each", elem), Vec::new());
        }
        let var = self.temp("$item", elem.clone());
        let item = TExpr {
            kind: TK::Var(var),
            ty: elem.clone(),
            span,
        };
        let mut open = Vec::new();
        match pats {
            [p] => self.let_pattern(p, false, item, &mut open),
            ps => {
                let tys = self.field_tys(&elem).map(|(_, t)| t).unwrap_or_default();
                if tys.len() != ps.len() && !self.mentions_error(&elem) {
                    let (k, n, t) = (ps.len(), tys.len(), self.show(&elem));
                    let what = format!("these {k} patterns name the parts of `{t}`, which has {n}");
                    self.err(span, Msg::verdict(cpl_diag::codes::TYPE_MISMATCH, what));
                }
                for (k, p) in ps.iter().enumerate() {
                    let ty = tys.get(k).cloned().unwrap_or(Ty::Error);
                    let kind = TK::Field(Box::new(item.clone()), k);
                    self.let_pattern(p, false, TExpr { kind, ty, span }, &mut open);
                }
            }
        }
        (var, open)
    }
    /// What a block does around its statements: a scope of their own, the checks and
    /// contexts they set kept to it, and its value, what `fill` gives (`None` for none). A
    /// value whose type mentions a domain declared inside leaves neither as the block's value
    /// nor in a binding made before it (R02.9.2).
    pub(super) fn scoped_block(
        &mut self,
        span: Span,
        fill: impl FnOnce(&mut Self, &mut Vec<TStmt>) -> Option<TExpr>,
    ) -> TExpr {
        self.b.scopes.push(HashMap::new());
        let mut out = Vec::new();
        let outer = self.b.group.len();
        let (doms_at, locals_at) = (self.b.block_doms.len(), self.b.locals.len());
        let globals_at = self.globals.len();
        let checks_kept = self.checks_kept;
        let frames = self.b.frames.len();
        let value = fill(self, &mut out);
        self.checks_kept = checks_kept;
        self.b.frames.truncate(frames);
        self.captures_set(outer, &out);
        self.b.group.truncate(outer);
        self.b.scopes.pop();
        let ty = value
            .as_ref()
            .map(|v: &TExpr| v.ty.clone())
            .unwrap_or(Ty::Unit);
        let mine = self.b.block_doms.split_off(doms_at);
        if !mine.is_empty() {
            let local = |t: &Ty| {
                let mut ds = Vec::new();
                super::domain::doms_in(t, &mut ds);
                ds.into_iter().find(|d| mine.contains(d))
            };
            let mut found = local(&self.zonk(&ty)).map(|d| (d, "as the block's value".to_string()));
            let before = self.b.locals[..locals_at]
                .iter()
                .chain(&self.globals[..globals_at]);
            for l in before {
                if found.is_some() {
                    break;
                }
                found = local(&self.zonk(&l.ty))
                    .map(|d| (d, format!("in `{}`, made before it", l.name)));
            }
            if let Some((d, how)) = found {
                let msg = cpl_diag::msgs::check::DOMAIN_ESCAPES_A_VALUE_OF_FRESH;
                let text = msg.say(&[&self.domains[d].name, &how]);
                self.err(span, Msg::verdict(cpl_diag::codes::DOMAIN_ESCAPES, text));
            }
        }
        TExpr {
            kind: TK::Block(out, value.map(Box::new)),
            ty,
            span,
        }
    }
    pub(super) fn block(&mut self, b: &next::Block, want: Option<&Ty>) -> TExpr {
        let entries: Vec<next::BlockEntry> = b.entries().collect();
        let span = self.at(b);
        self.scoped_block(span, |cx, out| cx.entries(&entries, want, true, out))
    }
    /// `{ .. }` written where a value goes: its entries in a scope of their own.
    pub(super) fn block_expr(&mut self, b: &next::BlockExpr, want: Option<&Ty>) -> TExpr {
        let entries: Vec<next::BlockEntry> = b.entries().collect();
        let span = self.at(b);
        let valued = !b.ends_with_semi();
        self.scoped_block(span, |cx, out| cx.entries(&entries, want, valued, out))
    }
    /// `stmts` then `tail`, in a scope of their own: a fn's `= e` body is
    /// `tail_block(&[], Some(e), ..)`. With no `tail`, a last expression statement is the value.
    pub(super) fn tail_block(
        &mut self,
        stmts: &[next::Stmt],
        tail: Option<&next::Expr>,
        want: Option<&Ty>,
    ) -> TExpr {
        let spans: Vec<Span> = (stmts.iter().map(|s| self.at(s)))
            .chain(tail.map(|e| self.at(e)))
            .collect();
        let span = match (spans.first(), spans.last()) {
            (Some(a), Some(b)) => Span::new(a.start, b.end),
            _ => Span::new(0, 0),
        };
        let entries: Vec<next::BlockEntry> =
            stmts.iter().cloned().map(next::BlockEntry::Stmt).collect();
        self.scoped_block(span, |cx, out| {
            let v = cx.entries(&entries, want, tail.is_none(), out);
            match tail {
                Some(e) => cx.tail_value(e, want, out),
                None => v,
            }
        })
    }
    /// A block's entries in order, the last expression its value when `valued`.
    fn entries(
        &mut self,
        entries: &[next::BlockEntry],
        want: Option<&Ty>,
        valued: bool,
        out: &mut Vec<TStmt>,
    ) -> Option<TExpr> {
        self.declare_nested(entries);
        let stmts_after = |i: usize| -> Vec<next::Stmt> {
            (entries[i + 1..].iter())
                .filter_map(|e| match e {
                    next::BlockEntry::Stmt(s) => Some(s.clone()),
                    next::BlockEntry::Decl(_) => None,
                })
                .collect()
        };
        let mut placed = 0;
        for (i, e) in entries.iter().enumerate() {
            if i < placed {
                continue;
            }
            let s = match e {
                next::BlockEntry::Decl(next::Decl::Fn(f)) if is_memo(f) => {
                    self.memo_fn(f, &stmts_after(i), out);
                    continue;
                }
                next::BlockEntry::Decl(next::Decl::Fn(f)) => {
                    let name = f.name().map_or_else(String::new, |t| t.text().to_string());
                    self.nested_fn(f, &name, out);
                    continue;
                }
                next::BlockEntry::Decl(next::Decl::Domain(d)) => {
                    self.domain_stmt(d, out);
                    continue;
                }
                next::BlockEntry::Decl(d) => {
                    self.local_decl(d);
                    continue;
                }
                next::BlockEntry::Stmt(s) => s,
            };
            // A bare `with` line sets a context for the rest of its block (R09.1.1).
            if let next::Stmt::With(w) = s {
                if w.body().is_none() && i > 0 {
                    let msg = Msg::verdict(
                        cpl_diag::codes::WITH_NOT_FIRST,
                        "a bare `with` line comes first in its block",
                    );
                    self.err(self.at(s), msg);
                    continue;
                }
            }
            // `let p = mut a[i]` names the cell: the statements through `p`'s last use run while
            // `a`'s `IndexedMut` lends it, `p` that cell (R05.3.4). A block value the lend would
            // hold is refused below with the bare `mut`.
            let unit = want.is_none_or(|w| self.shallow(w) == Ty::Unit);
            let named = place_binding(s)
                .map(|n| (place_held(&n.0, &entries[i + 1..]), n))
                .filter(|(held, _)| unit || i + 1 + held < entries.len());
            if let Some((held, named)) = named {
                let rest = entries[i + 1..i + 1 + held].to_vec();
                let span = self.at(s);
                let inside: InPlace<'_> = Box::new(move |this, body| {
                    let v = this.scoped_block(span, |cx, out| cx.entries(&rest, None, false, out));
                    body.push(TStmt::Expr(v));
                });
                self.place_rest(named, span, inside, out);
                placed = i + 1 + held;
                continue;
            }
            if valued && i + 1 == entries.len() {
                if let Some(e) = (match s {
                    next::Stmt::Expr(x) => Some(x),
                    _ => None,
                })
                .and_then(next::ExprStmt::expr)
                {
                    return self.tail_value(&e, want, out);
                }
            }
            self.stmt(s, out);
        }
        None
    }
    /// A block's last expression: its value, against `want`. One that never gives a value is a
    /// statement, as `return` is.
    fn tail_value(
        &mut self,
        e: &next::Expr,
        want: Option<&Ty>,
        out: &mut Vec<TStmt>,
    ) -> Option<TExpr> {
        if e.leaves() {
            self.expr_stmt(e, out);
            return None;
        }
        // `a[i].push(x)` last in a block that gives no value writes the cell, as the same line
        // anywhere else in the block does.
        let unit = want.is_none_or(|w| self.shallow(w) == Ty::Unit);
        if unit && self.cell_method(e, out) {
            return None;
        }
        let v = self.expr(e, want);
        Some(match want {
            Some(w) if self.shallow(w) != Ty::Unit => self.coerce(v, w),
            _ => v,
        })
    }
    /// A declaration written in a block that a block does not hold: what its name meant outside
    /// is shadowed, so its uses are the error, not another declaration's.
    fn local_decl(&mut self, d: &next::Decl) {
        let span = self.at(d);
        let name = decl_name(d);
        if let Some(n) = &name {
            self.b.refused_names.insert(n.clone());
            if self.b.decl.is_some() {
                self.bind(n, Ty::Error);
            }
        }
        self.err(
            span,
            Unbuilt::Statement {
                kind: super::kind_name(d),
            },
        );
    }
    /// `inside`, the statements after `let p = mut a[i]` through `p`'s last use, run while
    /// `a`'s `IndexedMut` lends cell `i`, with `p` naming that cell (R05.3.4).
    pub(super) fn place_rest(
        &mut self,
        (n, place): (String, next::Expr),
        at: Span,
        inside: InPlace<'_>,
        out: &mut Vec<TStmt>,
    ) {
        let next::Expr::Index(ix) = super::index::unparen(place) else {
            return;
        };
        let Some((root, keys)) = self.hoist_indices(&ix, out) else {
            return;
        };
        self.through_cell(
            root,
            keys,
            at,
            out,
            Box::new(move |this, cell, body| {
                let TK::Var(VarRef::Local(id) | VarRef::Global(id)) = cell.kind else {
                    return;
                };
                this.scope().insert(n, id);
                inside(this, body);
            }),
        );
    }
    /// Top-level items, each from its own item, with what a `let p = mut a[i]` among them
    /// holds run inside its lend.
    pub(super) fn items_placed(&mut self, items: &[(ItemId, next::Item)], out: &mut Vec<TStmt>) {
        let mut k = 0;
        while let Some((off, s)) = items.get(k) {
            self.b.item = off.clone();
            out.push(TStmt::Item(off.clone()));
            k += 1;
            let Some(named) = s.stmt().as_ref().and_then(place_binding) else {
                self.top_item(s, None, out);
                continue;
            };
            let rest: Vec<next::Item> = items[k..].iter().map(|(_, s)| s.clone()).collect();
            let held = place_held(&named.0, &rest);
            let inner = items[k..k + held].to_vec();
            let at = self.at(s);
            let inside: InPlace<'_> = Box::new(move |this, body| {
                this.b.scopes.push(HashMap::new());
                this.items_placed(&inner, body);
                this.b.scopes.pop();
            });
            self.place_rest(named, at, inside, out);
            k += held;
        }
    }
    /// A condition, in a scope of its own that holds the names its `let`s bind: the caller
    /// checks what the condition guards, then pops the scope.
    pub(super) fn cond(&mut self, c: &next::Condition) -> TExpr {
        self.b.scopes.push(HashMap::new());
        let parts: Vec<next::CondPart> = c.parts().collect();
        let span = self.at(c);
        self.cond_in(&parts, span)
    }
    /// An expression a condition tests: a `Bool`, never an integer or optional read as one (R02.11.1).
    pub(super) fn bool_cond(&mut self, e: &next::Expr) -> TExpr {
        if self.question_in_cond(e) {
            return Self::error_node(self.at(e));
        }
        let v = self.expr(e, Some(&Ty::Bool));
        let t = self.shallow(&v.ty);
        if matches!(t, Ty::Bool | Ty::Var(_) | Ty::Error) || self.coercion(&t, &Ty::Bool).is_some()
        {
            return self.coerce(v, &Ty::Bool);
        }
        let code = cpl_diag::codes::CONDITION_NOT_BOOL;
        let shown = self.show(&t);
        self.err(
            v.span,
            Msg::verdict(code, format!("this condition is a `{shown}`, not a `Bool`")),
        );
        TExpr {
            kind: TK::Error,
            ty: Ty::Bool,
            span: v.span,
        }
    }
    /// `parts`, each tested once those before it hold.
    fn cond_in(&mut self, parts: &[next::CondPart], span: Span) -> TExpr {
        let mk = |kind, span| TExpr {
            kind,
            ty: Ty::Bool,
            span,
        };
        let Some((part, more)) = parts.split_first() else {
            return mk(TK::Bool(true), span);
        };
        let t = match part {
            // `let p = e`: holds when `e` matches `p`, whose names then name its parts; on an
            // optional `e`, a pattern of no optional matches the value it has (R01.7.1).
            next::CondPart::Let(l) => {
                let (Some(pat), Some(value)) = (l.pattern(), l.value()) else {
                    return TExpr {
                        kind: TK::Error,
                        ty: Ty::Bool,
                        span,
                    };
                };
                let span = self.at(&value);
                let v = self.expr(&value, None);
                let mk = |kind| TExpr {
                    kind,
                    ty: Ty::Bool,
                    span,
                };
                let inner = (self.optional_of(&v.ty)).filter(|_| !self.optional_pattern(&pat));
                if inner.is_none() && Self::name_pattern(&pat) {
                    if !self.mentions_error(&v.ty) {
                        let t = self.show(&v.ty);
                        self.err(span, Unbuilt::IfLet { ty: t });
                    }
                    // Its names are the error, so their uses say nothing more.
                    self.bind_unknown(&pat, false, &mut Vec::new());
                    return TExpr {
                        kind: TK::Error,
                        ty: Ty::Error,
                        span,
                    };
                }
                let (hold, read) = self.held(v);
                let mut binds = Vec::new();
                let test = match inner {
                    Some(inner) => {
                        let flag = mk(TK::Field(Box::new(read.clone()), 0));
                        let val = TExpr {
                            kind: TK::Field(Box::new(read), 1),
                            ty: inner,
                            span,
                        };
                        let rest = self.refute(&pat, &val, false, &mut binds);
                        Self::both(Some(flag), rest)
                    }
                    None => self.refute(&pat, &read, false, &mut binds),
                };
                let rest = self.cond_in(more, span);
                let then = mk(TK::Block(binds, Some(Box::new(rest))));
                let body = match test {
                    Some(t) => mk(TK::If(
                        Box::new(t),
                        Box::new(then),
                        Box::new(mk(TK::Bool(false))),
                    )),
                    None => then,
                };
                mk(TK::Block(hold, Some(Box::new(body))))
            }
            next::CondPart::Expr(e) => self.bool_cond(e),
            next::CondPart::Bound(b) => {
                let at = self.at(b);
                self.has_bound(b, at)
            }
        };
        if more.is_empty() {
            return t;
        }
        let rest = self.cond_in(more, span);
        let span = t.span;
        mk(
            TK::If(
                Box::new(t),
                Box::new(rest),
                Box::new(mk(TK::Bool(false), span)),
            ),
            span,
        )
    }
    /// `if cond:` around what `then` checks, with the names the condition binds in scope.
    pub(super) fn if_then(
        &mut self,
        cond: &next::Condition,
        then: impl FnOnce(&mut Self, &mut Vec<TStmt>),
        at: Span,
        out: &mut Vec<TStmt>,
    ) {
        let test = self.cond(cond);
        let mut body = Vec::new();
        then(self, &mut body);
        self.b.scopes.pop();
        let unit = |kind| TExpr {
            kind,
            ty: Ty::Unit,
            span: at,
        };
        let then = unit(TK::Block(body, None));
        let none = unit(TK::Block(Vec::new(), None));
        out.push(TStmt::Expr(unit(TK::If(
            Box::new(test),
            Box::new(then),
            Box::new(none),
        ))));
    }
    /// The body of a match arm, a lambda, a table or an `if` value's branch.
    pub(super) fn arm_body(&mut self, b: &next::ArmBody, want: Option<&Ty>) -> TExpr {
        match b {
            next::ArmBody::Block(b) => self.block(b, want),
            next::ArmBody::Expr(e) => self.expr(e, want),
            next::ArmBody::Stmt(s) => self.tail_block(std::slice::from_ref(s), None, want),
        }
    }
    /// `let p = v`: `v` bound to the names `p` gives (R01.6.3); `at` is the statement, which a
    /// pattern that may not match is refused at.
    pub(super) fn let_pattern(
        &mut self,
        p: &next::Pattern,
        mutable: bool,
        v: TExpr,
        out: &mut Vec<TStmt>,
    ) {
        let at = self.at(p);
        self.let_pattern_at(p, mutable, v, at, out)
    }
    fn let_pattern_at(
        &mut self,
        p: &next::Pattern,
        mutable: bool,
        v: TExpr,
        at: Span,
        out: &mut Vec<TStmt>,
    ) {
        let span = self.at(p);
        let refused = |cx: &mut Self| {
            let code = cpl_diag::codes::EXPECTED_PATTERN;
            let text = "this pattern may not match; give the `let` an `else` that leaves";
            cx.err(at, Msg::verdict(code, text));
        };
        match p {
            next::Pattern::Paren(q) => match q.pattern() {
                Some(q) => self.let_pattern_at(&q, mutable, v, at, out),
                None => out.push(TStmt::Expr(v)),
            },
            // `let _ = e` runs `e` for its effects and drops its value.
            next::Pattern::Wildcard(_) => out.push(TStmt::Expr(v)),
            next::Pattern::Path(q) => {
                if let Some(n) = super::pattern::bind_name(q) {
                    // `var x` in a pattern binds a name that may change.
                    let ty = v.ty.clone();
                    self.let_bind(n.text(), mutable || q.is_var(), ty, v, span, out);
                    return;
                }
                let segs: Vec<_> = q.segments().collect();
                match (&segs[..], q.args()) {
                    ([n], None) if !super::expr::none_word(n) => {
                        let code = cpl_diag::codes::VALUE_NAME_CASE;
                        let text = format!("a value's name starts lowercase, not `{}`", n.text());
                        self.err(span, Msg::verdict(code, text));
                    }
                    _ => refused(self),
                }
            }
            next::Pattern::Literal(_) | next::Pattern::Or(_) => refused(self),
            // A record pattern without `..` names every field (R02.5.5).
            next::Pattern::Record(r) => {
                let parts = r.fields().map(|f| (f.name(), f.pattern())).collect();
                self.let_parts(p, parts, !r.has_rest(), mutable, v, at, out)
            }
            next::Pattern::Tuple(t) => {
                let parts = t.patterns().map(|q| (None, Some(q))).collect();
                self.let_parts(p, parts, false, mutable, v, at, out)
            }
        }
    }
    /// `let (a, b) = v` or `let {x, y: q} = v`: `v` held once, then each part bound to its
    /// field. A tuple pattern reads a record's fields in order (R02.5.5): `let (tree, used) =
    /// kruskal(..)` of `-> (tree: Rooted<V, _, _>, used: [E]Bool)`.
    #[allow(clippy::too_many_arguments)]
    fn let_parts(
        &mut self,
        p: &next::Pattern,
        parts: Vec<(Option<SyntaxToken>, Option<next::Pattern>)>,
        exact: bool,
        mutable: bool,
        v: TExpr,
        at: Span,
        out: &mut Vec<TStmt>,
    ) {
        let span = self.at(p);
        let held = format!("$parts{}", span.start);
        let whole = self.temp(&held, v.ty.clone());
        let read = TExpr {
            kind: TK::Var(whole),
            ty: v.ty.clone(),
            span,
        };
        out.push(TStmt::Let(whole, v));
        let Some((s, tys)) = self.field_tys(&read.ty) else {
            if !self.mentions_error(&read.ty) {
                self.err(span, Unbuilt::LetPattern);
            }
            return self.bind_unknown(p, mutable, out);
        };
        let names = self.d.field_names(s);
        let record = self.d.s(s).record;
        if exact && record && names.len() != parts.len() {
            let (n, have) = (parts.len(), names.len());
            let code = cpl_diag::codes::TYPE_MISMATCH;
            let text =
                format!("this names {n} of the record's {have} fields; write `..` for the rest");
            self.err(span, Msg::verdict(code, text));
        }
        for (k, (label, q)) in parts.into_iter().enumerate() {
            let i = match &label {
                Some(l) => match names.iter().position(|n| n == l.text()) {
                    Some(i) => i,
                    None => {
                        let t = self.show(&read.ty);
                        let code = cpl_diag::codes::TYPE_MISMATCH;
                        let text = format!("`{t}` has no field `{}`", l.text());
                        self.err(l.span(), Msg::verdict(code, text));
                        if let Some(q) = &q {
                            self.bind_unknown(q, mutable, out);
                        }
                        continue;
                    }
                },
                None => k,
            };
            let Some(t) = tys.get(i).cloned() else {
                let code = cpl_diag::codes::TYPE_MISMATCH;
                let text = format!(
                    "this pattern has {} parts, more than the value's {}",
                    k + 1,
                    tys.len()
                );
                self.err(span, Msg::verdict(code, text));
                break;
            };
            let part_at = q.as_ref().map_or(span, |q| self.at(q));
            let part = TExpr {
                kind: TK::Field(Box::new(read.clone()), i),
                ty: t,
                span: part_at,
            };
            let bound = match &q {
                Some(next::Pattern::Path(x)) => super::pattern::bind_name(x),
                Some(_) => None,
                None => label.clone(),
            };
            match &q {
                Some(q) => self.let_pattern_at(q, mutable, part, at, out),
                None => {
                    let l = label
                        .as_ref()
                        .expect("a record field without a pattern names it");
                    let ty = part.ty.clone();
                    self.let_bind(l.text(), mutable, ty, part, l.span(), out);
                }
            }
            // `tree.Pre` is the domain the whole value owns as its `tree.Pre`.
            if let (Some(n), Some(field)) = (bound, names.get(i)) {
                let field = self.b.syms.intern(&self.d.syms, field);
                let part = self.b.syms.intern(&self.d.syms, n.text());
                let held = self.b.syms.intern(&self.d.syms, &held);
                self.hand_down(held, field, part);
            }
        }
    }
    /// `name`, bound to `v` of type `ty` (what is declared is the binding's type even when
    /// the value is not one), with what a `let` knows of its value.
    pub(super) fn let_bind(
        &mut self,
        name: &str,
        mutable: bool,
        ty: Ty,
        v: TExpr,
        span: Span,
        out: &mut Vec<TStmt>,
    ) -> VarRef {
        self.memory_peak(name, &v.ty, span);
        let var = self.bind(name, ty);
        if !mutable && self.is_none_value(&v) {
            self.none_lets.insert(var);
        }
        if let (false, Some(r)) = (mutable, super::unify::known_range(&self.b.let_ranges, &v)) {
            self.b.let_ranges.insert(var, r);
        }
        let let_at = (!mutable).then(|| self.let_keyword(span)).flatten();
        match var {
            VarRef::Local(i) => {
                (self.b.locals[i].mutable, self.b.locals[i].fixed) = (mutable, !mutable);
                self.b.locals[i].let_at = let_at;
            }
            VarRef::Global(i) => {
                (self.globals[i].mutable, self.globals[i].fixed) = (mutable, !mutable);
                self.globals[i].let_at = let_at;
            }
        }
        let fresh = self.fresh_args(&v.ty);
        out.push(TStmt::Let(var, v));
        self.own_fresh(name, var, fresh, span, out);
        var
    }
    /// `let name = value`, as an item of the program writes it.
    pub(super) fn let_name(
        &mut self,
        name: &str,
        value: &next::Expr,
        span: Span,
        out: &mut Vec<TStmt>,
    ) {
        let v = self.expr(value, None);
        let ty = v.ty.clone();
        let var = self.let_bind(name, false, ty, v, span, out);
        if let Some(keys) = super::index::literal_cut(value) {
            self.view_keys.insert(var, keys);
        }
    }
    /// `let p: T = v`, `var p = v`, `let p = v else: leave` and `var a: T` (R01.6).
    fn let_stmt(&mut self, l: &next::LetStmt, out: &mut Vec<TStmt>) {
        let at = self.at(l);
        let mutable = l.is_var();
        let pats: Vec<next::Pattern> = l.patterns().collect();
        let [pat] = &pats[..] else {
            self.err(at, Unbuilt::LetPattern);
            return pats.iter().for_each(|p| self.bind_unknown(p, mutable, out));
        };
        let Some(init) = l.value() else {
            // `var a: T` with no value: the parser has said so, and `a` is still declared, so
            // what names it below is checked as if the value were there.
            let t = match l.ty() {
                Some(t) => self.body_ty(&t, out),
                None => Ty::Error,
            };
            match pat {
                next::Pattern::Path(p) => match super::pattern::bind_name(p) {
                    Some(n) => {
                        self.bind(n.text(), t);
                    }
                    None => self.bind_unknown(pat, mutable, out),
                },
                _ => self.bind_unknown(pat, mutable, out),
            }
            return;
        };
        if let Some(e) = l.else_clause() {
            return self.let_else(pat, mutable, l.ty().as_ref(), &init, &e, at, out);
        }
        let name = match pat {
            next::Pattern::Path(p) => super::pattern::bind_name(p),
            _ => None,
        };
        // `let (a, b): (T, U) = read()` reads each part in order, so a later part's type may
        // name an earlier part (R10.2.3).
        if let next::Pattern::Tuple(t) = pat {
            let ps: Vec<next::Pattern> = t.patterns().collect();
            if let Some((n, targ, args)) = self.per_part_call(&init) {
                let written = match (l.ty(), targ) {
                    (Some(next::Type::Tuple(ts)), None) => Some(ts),
                    (None, Some(next::Type::Tuple(ts))) => Some(ts),
                    _ => None,
                };
                let tys: Vec<next::Type> = written
                    .map(|ts| ts.elems().into_iter().map(|(_, t)| t).collect())
                    .unwrap_or_default();
                if !tys.is_empty() && tys.len() == ps.len() {
                    let span = self.at(&init);
                    for (p, t) in ps.iter().zip(tys) {
                        let t = self.body_ty(&t, out);
                        let args: Vec<CallArg> =
                            args.iter().map(|a| CallArg::of(self, a)).collect();
                        let v = self.call_named(&n, &[], &args, span, Some(&t));
                        let v = self.coerce(v, &t);
                        self.let_pattern_at(p, mutable, v, at, out);
                    }
                    return;
                }
            }
        }
        let declared = l.ty().map(|t| self.body_ty(&t, out));
        let v = self.let_value(
            &init,
            declared.as_ref(),
            mutable,
            name.as_ref().map(|n| n.text()),
            at,
        );
        match (name, pat) {
            (Some(n), next::Pattern::Path(p)) => {
                let ty = declared.unwrap_or_else(|| v.ty.clone());
                let var = self.let_bind(n.text(), mutable || p.is_var(), ty, v, at, out);
                if let Some(keys) = super::index::literal_cut(&init) {
                    self.view_keys.insert(var, keys);
                }
            }
            _ => self.let_pattern_at(pat, mutable, v, at, out),
        }
    }
    /// A `let`'s value: checked against what is declared, a declared array of a literal its
    /// fill, and a `var` holding a function given a set so it can be reassigned.
    fn let_value(
        &mut self,
        init: &next::Expr,
        declared: Option<&Ty>,
        mutable: bool,
        name: Option<&str>,
        span: Span,
    ) -> TExpr {
        // A literal is never an array, so where one is declared it is the fill: its cell's
        // type, as a range of cuts filling `[V]Range<V>` is a piece (R04.4.4).
        let literal = match super::index::unparen(init.clone()) {
            next::Expr::Literal(_) | next::Expr::Range(_) => true,
            next::Expr::Name(n) => n.ident().is_some_and(|t| super::expr::none_word(&t)),
            _ => false,
        };
        let mut cell = None;
        while let (Some(Ty::Struct(d, ts, _)), Some(arr)) =
            (cell.as_ref().or(declared), self.array_id())
        {
            if !literal || *d != arr {
                break;
            }
            cell = Some(ts[1].clone());
        }
        if let (false, Some(n), next::Expr::Lambda(_)) = (mutable, name, init) {
            self.b.recur = Some(n.to_string());
        }
        let v = self.expr(init, cell.as_ref().or(declared));
        self.b.recur = None;
        // `var a: [V]Int = 0` fills every slot: it is `Array.filled(0)`.
        let fill = match (declared, self.array_id()) {
            (Some(array @ Ty::Struct(d, ts, _)), Some(arr))
                if *d == arr
                    && (!matches!(self.shallow(&v.ty), Ty::Struct(..))
                        || self.fits(&v.ty, &ts[1])) =>
            {
                Some(array.clone())
            }
            _ => None,
        };
        let v = match fill {
            Some(array) => self.fill(v, array, span),
            None => v,
        };
        match declared {
            Some(t) => self.coerce(v, t),
            // A `var` holding a function can be reassigned, so it gets a set; a `let` keeps the
            // closure's own type.
            None if mutable => match self.shallow(&v.ty) {
                Ty::Clo(c) => {
                    let (ps, r) = self.clo_sig(&c);
                    let slot = Ty::Fn(ps, Box::new(r), self.new_set());
                    self.coerce(v, &slot)
                }
                Ty::Fn(ps, r, set) if !matches!(self.shallow_set(&set), SetTy::Var(_)) => {
                    let slot = Ty::Fn(ps, r, self.new_set());
                    self.coerce(v, &slot)
                }
                _ => v,
            },
            None => v,
        }
    }
    /// `let p = x else: leave`: the block runs when `x` does not match `p`, whose names are
    /// bound after it (R01.6.2).
    #[allow(clippy::too_many_arguments)]
    fn let_else(
        &mut self,
        pat: &next::Pattern,
        mutable: bool,
        ty: Option<&next::Type>,
        init: &next::Expr,
        e: &next::ElseClause,
        at: Span,
        out: &mut Vec<TStmt>,
    ) {
        let body = e.body();
        if !body.as_ref().is_some_and(|b| b.leaves()) {
            let code = cpl_diag::codes::ELSE_FALLS_THROUGH;
            let text = "this `else` block can reach its end; end it with `return`, `break`, `continue` or a trap";
            self.err(at, Msg::verdict(code, text));
        }
        let otherwise = |cx: &mut Self| match &body {
            Some(b) => cx.arm_body(b, None),
            None => TExpr {
                kind: TK::Block(Vec::new(), None),
                ty: Ty::Unit,
                span: at,
            },
        };
        let unit = TExpr {
            kind: TK::Unit,
            ty: Ty::Unit,
            span: at,
        };
        let declared = ty.map(|t| self.body_ty(t, out));
        let name = match pat {
            next::Pattern::Path(p) => super::pattern::bind_name(p).map(|n| (n, p.is_var())),
            _ => None,
        };
        let Some((name, var)) = name else {
            let v = self.expr(init, declared.as_ref());
            let v = match &declared {
                Some(t) => self.coerce(v, t),
                None => v,
            };
            let (hold, read) = self.held(v);
            out.extend(hold);
            let none = otherwise(self);
            let mut binds = Vec::new();
            if let Some(test) = self.refute(pat, &read, mutable, &mut binds) {
                out.push(TStmt::Expr(TExpr {
                    kind: TK::If(Box::new(test), Box::new(unit), Box::new(none)),
                    ty: Ty::Unit,
                    span: at,
                }));
            }
            return out.extend(binds);
        };
        let mutable = mutable || var;
        let want = declared.map(|t| self.optional_ty(t));
        let v = self.expr(init, want.as_ref());
        let v = match &want {
            Some(w) => self.coerce(v, w),
            None => v,
        };
        let Some(inner) = self.optional_of(&v.ty) else {
            if !self.mentions_error(&v.ty) {
                let t = self.show(&v.ty);
                self.err(self.at(init), Unbuilt::LetElse { ty: t });
            }
            // Its name is the error, so its uses say nothing more.
            otherwise(self);
            self.bind(name.text(), Ty::Error);
            return;
        };
        let (hold, flag, val) = self.opened(v, inner.clone());
        out.extend(hold);
        let none = otherwise(self);
        out.push(TStmt::Expr(TExpr {
            kind: TK::If(Box::new(flag), Box::new(unit), Box::new(none)),
            ty: Ty::Unit,
            span: at,
        }));
        let var = self.bind(name.text(), inner);
        match var {
            VarRef::Local(i) => {
                (self.b.locals[i].mutable, self.b.locals[i].fixed) = (mutable, !mutable)
            }
            VarRef::Global(i) => {
                (self.globals[i].mutable, self.globals[i].fixed) = (mutable, !mutable)
            }
        }
        out.push(TStmt::Let(var, val));
    }
    /// `f(..)` or `f<T>(..)` for a fn whose parameters all have defaults and that gives its one
    /// type parameter, as `read(base: 1)`: its name, the type argument if written, and the
    /// arguments. Bound to a tuple pattern with a tuple type it is called once per part, with
    /// the same arguments, so a later part's type may name an earlier part (R10.2.3).
    pub(super) fn per_part_call(
        &self,
        e: &next::Expr,
    ) -> Option<(String, Option<next::Type>, Vec<next::Arg>)> {
        let next::Expr::Call(c) = e else {
            return None;
        };
        let (base, targ) = match c.callee()? {
            next::Expr::Generic(g) => {
                let ts = super::member::written_types(&g);
                match &ts[..] {
                    [t] => (g.base()?, Some(t.clone())),
                    _ => return None,
                }
            }
            callee => (callee, None),
        };
        let n = super::member::bare_name(&base)?;
        let f = self
            .d
            .fn_named(self.b.module, &n)
            .filter(|_| self.declared(&n))?;
        let fd = self.d.f(f);
        let shape = fd.params.iter().all(|p| p.default.is_some()) && fd.generics.len() == 1;
        let args = c.args().map(|a| a.args().collect()).unwrap_or_default();
        (shape && fd.ret == Ty::Param(0)).then_some((n, targ, args))
    }
    /// A `const` of the program: a `let` its value fixes (R01.6.1); `known` the value it has
    /// when it runs above the program, written in as its literal.
    pub(super) fn const_stmt(
        &mut self,
        c: &next::ConstDecl,
        known: Option<ops::Scalar>,
        out: &mut Vec<TStmt>,
    ) {
        let span = self.at(c);
        let (Some(name), Some(value)) = (c.name(), c.value()) else {
            return;
        };
        let declared = c.ty().map(|t| self.body_ty(&t, out));
        let v = match known {
            Some(ops::Scalar::Int(k)) => TExpr {
                kind: TK::Int(k),
                ty: Ty::Int,
                span: self.at(&value),
            },
            _ => self.expr(&value, declared.as_ref()),
        };
        let v = match &declared {
            Some(t) => self.coerce(v, t),
            None => v,
        };
        let ty = declared.unwrap_or_else(|| v.ty.clone());
        let var = self.let_bind(name.text(), false, ty, v, span, out);
        if let Some(keys) = super::index::literal_cut(&value) {
            self.view_keys.insert(var, keys);
        }
    }
    /// An expression run for its effects.
    pub(super) fn expr_stmt(&mut self, e: &next::Expr, out: &mut Vec<TStmt>) {
        if self.dropped_question(e) || self.cell_method(e, out) {
            return;
        }
        let v = self.expr(e, None);
        out.push(TStmt::Expr(v));
    }
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
                let Some((true, elem)) = self.d.f(f.decl).region.clone() else {
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
    fn assign(&mut self, a: &next::AssignStmt, out: &mut Vec<TStmt>) {
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
                for (k, e) in t.elems().enumerate() {
                    let v = self.expr(&e, None);
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
    /// `for pats in iter: body` over a value already checked (a table's `r, c`, a
    /// comprehension's clause).
    pub(super) fn for_over(
        &mut self,
        pats: &[next::Pattern],
        iter: TExpr,
        body: impl FnOnce(&mut Self, &mut Vec<TStmt>),
        at: Span,
        out: &mut Vec<TStmt>,
    ) {
        self.walk_value(iter, pats, |cx, _, out| body(cx, out), at, out)
    }
    /// `for pat in iter: body`: a declared domain's ordinals, a closed range of `Int`s, or a
    /// value walked through its `Seq` (R06.3).
    pub(super) fn for_loop(
        &mut self,
        pat: &next::Pattern,
        iter: &next::Expr,
        body: impl FnOnce(&mut Self, &mut Vec<TStmt>),
        at: Span,
        out: &mut Vec<TStmt>,
    ) {
        let iter_at = self.at(iter);
        let it = super::index::unparen(iter.clone());
        // A bare `..` names no domain to walk (R04.4.11).
        if let next::Expr::Range(r) = &it {
            if r.lo().is_none() && r.hi().is_none() && r.step().is_none() {
                let text = cpl_diag::msgs::check::BARE_MEANS_ALL_DIMENSION.say(&[]);
                let help = "name what to walk: `for x in D` or `for i in 0..n`".to_string();
                let msg = Msg::verdict(cpl_diag::codes::EXPECTED_EXPR, text);
                return self.err_help(iter_at, msg, Some(help));
            }
        }
        // A name, or `_`: an item nothing reads.
        let bound: Option<Option<SyntaxToken>> = match pat {
            next::Pattern::Path(p) => super::pattern::bind_name(p).map(Some),
            next::Pattern::Wildcard(_) => Some(None),
            _ => None,
        };
        let Some(name) = bound else {
            return self.walk_iter(pat, &it, body, at, out);
        };
        let counter = |cx: &mut Self, ty: Ty| match &name {
            Some(n) => cx.bind(n.text(), ty),
            None => cx.temp("$each", ty),
        };
        // A declared domain's positions are its ordinals; any other walks its `Seq`.
        if let next::Expr::Name(dn) = &it {
            let ordinals = (dn.ident())
                .and_then(|t| self.domain_named(t.text()))
                .filter(|d| matches!(d, Ty::Dom(_)));
            if let Some(dty) = ordinals {
                let lo = TExpr {
                    kind: TK::Int(0),
                    ty: Ty::Int,
                    span: at,
                };
                let hi = TExpr {
                    kind: TK::DomLen(dty.clone()),
                    ty: Ty::Int,
                    span: at,
                };
                self.b.scopes.push(HashMap::new());
                let var = counter(self, dty);
                let b = self.scoped_block(at, |cx, out| {
                    body(cx, out);
                    None
                });
                self.b.scopes.pop();
                return out.push(TStmt::For(var, lo, hi, Box::new(b)));
            }
        }
        let next::Expr::Range(r) = &it else {
            return self.walk_iter(pat, &it, body, at, out);
        };
        let (Some(lo), Some(hi), None) = (r.lo(), r.hi(), r.step()) else {
            return self.walk_iter(pat, &it, body, at, out);
        };
        let inclusive = r.is_inclusive();
        let first = self.expr(&lo, Some(&Ty::Int));
        if self.ord_ends(&first.ty, None).is_some() {
            let r = self.range_from(first, &hi, inclusive, None, None, iter_at);
            return self.walk_value(
                r,
                std::slice::from_ref(pat),
                |cx, _, out| body(cx, out),
                at,
                out,
            );
        }
        let lo = self.coerce(first, &Ty::Int);
        let hi_at = self.at(&hi);
        let hi = self.expr(&hi, Some(&Ty::Int));
        let hi = self.coerce(hi, &Ty::Int);
        // `lo..=hi` is `lo..hi+1` here, so it fails at the largest Int where the language does not.
        let hi = match inclusive {
            true => {
                let one = TExpr {
                    kind: TK::Int(1),
                    ty: Ty::Int,
                    span: hi_at,
                };
                self.binary(BinOp::Add, hi, one, hi_at)
            }
            false => hi,
        };
        self.b.scopes.push(HashMap::new());
        let var = counter(self, Ty::Int);
        let b = self.scoped_block(at, |cx, out| {
            body(cx, out);
            None
        });
        self.b.scopes.pop();
        out.push(TStmt::For(var, lo, hi, Box::new(b)));
    }
    /// `for pat in xs` through the type's own walk: `gen(args)`, `xs.getter`, or `xs`'s `Seq`.
    fn walk_iter(
        &mut self,
        pat: &next::Pattern,
        iter: &next::Expr,
        body: impl FnOnce(&mut Self, &mut Vec<TStmt>),
        at: Span,
        out: &mut Vec<TStmt>,
    ) {
        self.b.allow_region = matches!(iter, next::Expr::Call(_) | next::Expr::Field(_));
        let call = self.expr(iter, None);
        self.b.allow_region = false;
        self.walk_value(
            call,
            std::slice::from_ref(pat),
            |cx, _, out| body(cx, out),
            at,
            out,
        )
    }
    /// `yield from xs` is `for x in xs: yield x`, over any `Seq` (R06.3.6).
    pub(super) fn yield_all(&mut self, seq: &next::Expr, span: Span, out: &mut Vec<TStmt>) {
        let Some((false, t)) = self.b.region.clone() else {
            let code = cpl_diag::codes::TYPE_MISMATCH;
            return self.err(
                span,
                Msg::verdict(code, "`yield` outside a fn that `yields`"),
            );
        };
        let it = super::index::unparen(seq.clone());
        self.b.allow_region = matches!(it, next::Expr::Call(_) | next::Expr::Field(_));
        let call = self.expr(&it, None);
        self.b.allow_region = false;
        self.b.loops += 1;
        let each = move |cx: &mut Self, item: TExpr, out: &mut Vec<TStmt>| {
            let v = cx.coerce(item, &t);
            out.push(TStmt::Yield(v))
        };
        self.walk_value(call, &[], each, span, out);
        self.b.loops -= 1;
    }
    /// `lend place`: a lender's one lend of its place (R05.6); in a read lend, the place's
    /// value is returned (R05.6.3).
    pub(super) fn lend_stmt(&mut self, place: &next::Expr, span: Span, out: &mut Vec<TStmt>) {
        match self.b.region.clone() {
            // Every path through a lender reaches one `lend`; inside a loop it could run twice.
            Some((true, _)) if self.b.loops > 0 => {
                let code = cpl_diag::codes::BAD_PROJECTION;
                let text = "`lend` inside a loop: a lender lends one place once";
                self.err(span, Msg::verdict(code, text))
            }
            Some((true, t)) => {
                // `lend mut s[k]` of a cell that is no slot lends on the cell `s`'s `IndexMut`
                // lends (R12.4.6).
                if let next::Expr::Index(ix) = super::index::unparen(place.clone()) {
                    let seen = self.errors.len();
                    let probe = self.expr_as_typed(place, None);
                    self.errors.truncate(seen);
                    if !matches!(probe.kind, TK::Slot(..)) {
                        let Some((root, keys)) = self.hoist_indices(&ix, out) else {
                            return;
                        };
                        let inner = move |cx: &mut Self, cell: TExpr, out: &mut Vec<TStmt>| {
                            cx.lend_out(cell, &t, span, out)
                        };
                        return self.through_cell(root, keys, span, out, Box::new(inner));
                    }
                }
                let p = self.place(place);
                self.lend_out(p, &t, span, out);
            }
            None if (self.b.decl).is_some_and(|f| self.d.f(f).lends.is_some()) => {
                let ret = self.b.ret.clone();
                let v = self.expr(place, Some(&ret));
                let v = self.coerce(v, &ret);
                out.push(TStmt::Return(Some(v)));
            }
            _ => {
                let code = cpl_diag::codes::BAD_PROJECTION;
                self.err(span, Msg::verdict(code, "`lend` outside a fn that `lends`"))
            }
        }
    }
    pub(super) fn stmt(&mut self, s: &next::Stmt, out: &mut Vec<TStmt>) {
        let (mark, at) = (self.b.list_doms.len(), out.len());
        let rhs = match s {
            next::Stmt::Let(l) if l.else_clause().is_none() => l.value(),
            next::Stmt::Assign(a) => a.value(),
            next::Stmt::Return(r) => r.value(),
            _ => None,
        };
        let extent = rhs.and_then(|e| {
            e.bare_try()
                .map(|_| self.at(&super::index::unparen(e.clone())))
        });
        let outer = std::mem::replace(&mut self.b.rhs_extent, extent);
        self.stmt_kind(s, out);
        self.b.rhs_extent = outer;
        let mut pre = Vec::new();
        for (d, n, span) in self.b.list_doms.split_off(mark) {
            if !matches!(self.shallow(&d), Ty::Var(_)) {
                continue;
            }
            let int = |k: usize| TExpr {
                kind: TK::Int(k as _),
                ty: Ty::Int,
                span,
            };
            let (lo, end, within) = self.typed_ends(int(0), int(n), false);
            let i = self.declare_anonymous(lo, end, within, span, &mut pre);
            self.unify(&d, &Ty::Dom(i));
        }
        out.splice(at..at, pre);
    }
    fn stmt_kind(&mut self, s: &next::Stmt, out: &mut Vec<TStmt>) {
        let span = self.at(s);
        let refused = |cx: &mut Self| {
            let kind = crate::kind_name(s);
            cx.err(span, Unbuilt::Statement { kind })
        };
        match s {
            next::Stmt::Let(l) => self.let_stmt(l, out),
            next::Stmt::Expr(e) => {
                if let Some(e) = e.expr() {
                    self.expr_stmt(&e, out)
                }
            }
            next::Stmt::Assign(a) => self.assign(a, out),
            next::Stmt::For(_) | next::Stmt::While(_) if self.b.framed != Some(span.start) => {
                self.framed_loop(s, out)
            }
            next::Stmt::While(w) => {
                let (Some(c), Some(body)) = (w.cond(), w.body()) else {
                    return;
                };
                let c = self.cond(&c);
                self.b.loops += 1;
                let b = self.block(&body, None);
                self.b.loops -= 1;
                self.b.scopes.pop();
                out.push(TStmt::While(c, b));
            }
            next::Stmt::For(f) => {
                let clauses: Vec<next::ForClause> = f.clauses().collect();
                let ([c], Some(body)) = (&clauses[..], f.body()) else {
                    return self.err(span, Unbuilt::ForForm);
                };
                let (Some(pat), Some(iter)) = (c.pattern(), c.iter()) else {
                    return;
                };
                if c.filters().next().is_some() {
                    return self.err(span, Unbuilt::ForForm);
                }
                self.b.loops += 1;
                self.for_loop(&pat, &iter, |cx, out| cx.block_into(&body, out), span, out);
                self.b.loops -= 1;
            }
            next::Stmt::Repeat(r) => {
                let (Some(count), Some(body)) = (r.count(), r.body()) else {
                    return;
                };
                let lo = TExpr {
                    kind: TK::Int(0),
                    ty: Ty::Int,
                    span,
                };
                let hi = self.expr(&count, Some(&Ty::Int));
                let hi = self.coerce(hi, &Ty::Int);
                self.b.loops += 1;
                self.b.scopes.push(HashMap::new());
                let var = self.temp("$each", Ty::Int);
                let b = self.block(&body, None);
                self.b.scopes.pop();
                self.b.loops -= 1;
                out.push(TStmt::For(var, lo, hi, Box::new(b)));
            }
            next::Stmt::Return(r) => {
                let ret = self.b.ret.clone();
                let v = r.value().map(|e| {
                    let v = self.expr(&e, Some(&ret));
                    self.coerce(v, &ret)
                });
                out.push(TStmt::Return(v));
            }
            next::Stmt::Break(b) => {
                let label = b.label().map(|l| l.text().to_string());
                self.break_to(label.as_deref(), span, out)
            }
            next::Stmt::Continue(c) if c.label().is_none() => out.push(TStmt::Continue),
            next::Stmt::Yield(y) => {
                let Some(v) = y.value() else {
                    return;
                };
                if y.is_from() {
                    return self.yield_all(&v, span, out);
                }
                match self.b.region.clone() {
                    Some((false, t)) => {
                        let v = self.expr(&v, Some(&t));
                        let v = self.coerce(v, &t);
                        out.push(TStmt::Yield(v));
                    }
                    _ => {
                        let code = cpl_diag::codes::TYPE_MISMATCH;
                        self.err(
                            span,
                            Msg::verdict(code, "`yield` outside a fn that `yields`"),
                        )
                    }
                }
            }
            next::Stmt::Lend(l) => {
                if let Some(p) = l.place() {
                    self.lend_stmt(&p, span, out)
                }
            }
            next::Stmt::Assert(a) => {
                let Some(c) = a.cond() else {
                    return;
                };
                let c = self.bool_cond(&c);
                let msg = (a.message())
                    .and_then(|m| assert_text(&m))
                    .unwrap_or_else(|| "assertion failed".to_string());
                out.push(TStmt::Assert(c, msg, span, CheckKind::Assert));
            }
            next::Stmt::If(i) => {
                let e = self.if_stmt(i);
                out.push(TStmt::Expr(e));
            }
            next::Stmt::With(w) => self.with_stmt(w, out),
            // `unchecked:` turns every check off in its block, in every build (R09.4.3).
            next::Stmt::Unchecked(u) => {
                if let Some(body) = u.body() {
                    let b = self.block_keeping(Checks::NONE, &body, span);
                    out.push(TStmt::Expr(b));
                }
            }
            next::Stmt::Continue(_) | next::Stmt::Trusted(_) => refused(self),
        }
    }
    /// `body`'s statements and value, checked into `out` as one block.
    fn block_into(&mut self, body: &next::Block, out: &mut Vec<TStmt>) {
        let b = self.block(body, None);
        out.push(TStmt::Expr(b));
    }
    /// `with checks(..)`, `with ctx(f: v)` and several at once: a block's own, or a bare line
    /// the program's, for the rest of the file (R09.1.1).
    fn with_stmt(&mut self, w: &next::WithStmt, out: &mut Vec<TStmt>) {
        let span = self.at(w);
        let (mut kept, mut frames) = (None, Vec::new());
        for c in w.contexts() {
            let (name, args) = match &c {
                next::Expr::Call(call) => (
                    call.callee().and_then(|e| super::member::bare_name(&e)),
                    call.args().map(|a| a.args().collect()).unwrap_or_default(),
                ),
                e => (super::member::bare_name(e), Vec::new()),
            };
            let Some(name) = name else {
                return self.err(
                    span,
                    Unbuilt::Statement {
                        kind: crate::kind_name(w),
                    },
                );
            };
            if name == "checks" {
                match self.checks(&args, span, kept.unwrap_or(self.checks_kept)) {
                    Some(k) => kept = Some(k),
                    None => return,
                }
                continue;
            }
            match self.ctx_frames(&name, &args) {
                Some(fs) => frames.extend(fs),
                None => {
                    return self.err(
                        span,
                        Unbuilt::Statement {
                            kind: crate::kind_name(w),
                        },
                    )
                }
            }
        }
        let Some(body) = w.body() else {
            if let Some(k) = kept {
                self.checks_kept = k;
                out.push(TStmt::Checks(k));
            }
            return self.b.frames.extend(frames);
        };
        let n = self.b.frames.len();
        self.b.frames.extend(frames);
        let b = match kept {
            Some(k) => self.block_keeping(k, &body, span),
            None => self.block(&body, None),
        };
        self.b.frames.truncate(n);
        out.push(TStmt::Expr(b));
    }
    /// The checks kept under `with checks(...)` here (R09.4): each field keeps its check on or
    /// turns it off, from the checks kept around it.
    fn checks(&mut self, args: &[next::Arg], span: Span, from: Checks) -> Option<Checks> {
        let mut kept = from;
        for a in args {
            let on = match a
                .value()
                .as_ref()
                .and_then(super::member::bare_name)
                .as_deref()
            {
                Some("on") => true,
                Some("off") => false,
                _ => {
                    self.err(span, Unbuilt::ChecksValue);
                    return None;
                }
            };
            match a.label().and_then(|n| CheckName::named(n.text())) {
                Some(c) => kept = kept.with(c, on),
                None => {
                    self.err(span, Unbuilt::ChecksField);
                    return None;
                }
            }
        }
        Some(kept)
    }
    /// `body` checked and lowered with `kept` its checks: the block opens by saying so.
    fn block_keeping(&mut self, kept: Checks, body: &next::Block, span: Span) -> TExpr {
        let outer = std::mem::replace(&mut self.checks_kept, kept);
        let b = self.block(body, None);
        self.checks_kept = outer;
        TExpr {
            kind: TK::Block(vec![TStmt::Checks(kept), TStmt::Expr(b)], None),
            ty: Ty::Unit,
            span,
        }
    }
    /// `with ctx(f: v, ..)`: the fields it sets, each to a constant (R09.1.2); `None` when `ctx`
    /// names no context.
    fn ctx_frames(&mut self, ctx: &str, args: &[next::Arg]) -> Option<Vec<super::CtxFrame>> {
        let d = self.d;
        let Some(decls::Named::Context(id)) = d.named(self.b.module, ctx) else {
            return None;
        };
        let c = &d.contexts[id.0];
        let mut fs = Vec::new();
        for a in args {
            let Some(value) = a.value() else {
                continue;
            };
            let at = self.at(&value);
            let field = a.label().map(|n| n.text().to_string()).unwrap_or_default();
            if !c.fields.iter().any(|x| self.d.syms.text(x.name) == field) {
                let code = cpl_diag::codes::UNKNOWN_ARG_NAME;
                let msg =
                    Msg::verdict(code, format!("context `{}` has no field `{field}`", c.name));
                self.err(at, msg);
            } else if self.const_value(&value).is_none() {
                let code = cpl_diag::codes::CONTEXT_NOT_CONSTANT;
                let msg = Msg::verdict(code, format!("`{}.{field}` is set to a constant", c.name));
                self.err(at, msg);
            } else {
                fs.push(super::CtxFrame {
                    ctx: id,
                    field,
                    value,
                });
            }
        }
        Some(fs)
    }
    pub(super) fn body_ty(&mut self, t: &next::Type, out: &mut Vec<TStmt>) -> Ty {
        self.body_ty_open(t, &[], out)
    }
    /// `t` read with `holes` as type parameters past the fn's own, each then a fresh variable
    /// for the checker to infer.
    pub(super) fn body_ty_open(
        &mut self,
        t: &next::Type,
        holes: &[String],
        out: &mut Vec<TStmt>,
    ) -> Ty {
        let span = self.at(t);
        let axes = self.name_ranges(t, out);
        let mut generics = self.b.generics.clone();
        for h in holes {
            let h = self.b.syms.intern(&self.d.syms, h);
            generics.push(h);
        }
        let (ty, errs) = decls::body_type(
            self.d,
            self.d,
            self.b.module,
            &generics,
            &self.b.doms,
            &|n| self.owned_named(n),
            &axes,
            t,
        );
        for e in errs {
            self.err(span, e);
        }
        let ty = self.fresh_sets(&ty);
        if holes.is_empty() {
            return ty;
        }
        let tys: Vec<Ty> = (0..self.b.generics.len())
            .map(Ty::Param)
            .chain(holes.iter().map(|_| self.new_ty()))
            .collect();
        Subst {
            tys: &tys,
            sets: &[],
        }
        .ty(&ty)
    }
    /// A domain's size is a hidden variable next to its declaration.
    pub(super) fn array_id(&self) -> Option<StructId> {
        self.d.sugar.array
    }
    /// `Array.filled(v)` through the trait marked `@literal("[;]")`: `var a: [V]Int = 0` (R01.6.1).
    fn fill(&mut self, v: TExpr, array: Ty, span: Span) -> TExpr {
        let Ty::Struct(_, ts, _) = &array else {
            return v;
        };
        let elem = ts[1].clone();
        // `[V][W]Int = 0` fills each row with a filled `[W]Int`.
        let v = match self.shallow(&elem) {
            Ty::Struct(d, ..) if Some(d) == self.array_id() && !self.fits(&v.ty, &elem) => {
                self.fill(v, elem.clone(), span)
            }
            _ => v,
        };
        let v = self.coerce(v, &elem);
        let tr = self.d.literal_trait("[;]");
        let fns = tr.map(|tr| self.d.trait_fns(tr)).unwrap_or_default();
        match (tr, fns.first()) {
            (Some(tr), Some(&(n, m))) => {
                let n = self.d.text(n);
                self.reach(m);
                let tr = decls::TraitRef {
                    tr,
                    args: vec![elem],
                };
                let o = (array.clone(), tr.clone(), span, Origin::Bound);
                self.b.obligations.push(o);
                let kind = TK::CallTrait {
                    ctx: self.trait_ctx(tr.tr, n),
                    tr,
                    method: n.to_string(),
                    recv: None,
                    self_ty: Box::new(array.clone()),
                    targs: Box::new([]),
                    args: vec![TArg::Value(v)],
                };
                TExpr {
                    kind,
                    ty: array,
                    span,
                }
            }
            _ => {
                self.err(
                    span,
                    Msg::internal("no trait is marked `@literal(\"[;]\")`"),
                );
                v
            }
        }
    }
    /// E0904: an array the input alone can fill past the judge's memory limit (R03.1.9). Only
    /// one-domain arrays of fixed-width cells are measured; nothing else is claimed.
    fn memory_peak(&mut self, name: &str, t: &Ty, span: Span) {
        let (Some(judge), Some(arr)) = (&self.judge, self.array_id()) else {
            return;
        };
        let Ty::Struct(a, ts, _) = self.shallow(t) else {
            return;
        };
        let (Some(key), Some(elem)) = (ts.first().map(|d| self.shallow(d)), ts.get(1)) else {
            return;
        };
        let (lo, hi) = match key {
            Ty::Dom(i) => self.domains[i].within,
            Ty::Range(lo, hi) => (Some(lo), Some(hi)),
            _ => return,
        };
        let width: u128 = match self.shallow(elem) {
            Ty::I128 => 16,
            Ty::Int | Ty::U64 | Ty::Dom(_) => 8,
            Ty::Bool | Ty::Char => 1,
            _ => return,
        };
        let (Some(lo), Some(hi)) = (lo, hi) else {
            return;
        };
        let cells = u128::try_from(hi - lo + 1).unwrap_or(0);
        let (bytes, limit) = (cells * width, u128::from(judge.memory_mb) << 20);
        if a == arr && bytes > limit {
            let (mb, judge) = (judge.memory_mb, judge.name.clone());
            self.err(
                span,
                Msg::verdict(cpl_diag::codes::MEMORY_OVER_PROFILE, format!("`{name}` takes {} MB at the input bounds, over the {mb} MB limit on {judge}", bytes >> 20)),
            );
        }
    }
    /// Whether `v` is input: a value an `@input` fn gives, or a part of one (R10.2.2).
    pub(super) fn is_input(&self, v: &TExpr) -> bool {
        match &v.kind {
            TK::Call { f, .. } => self.d.input_fns.contains(&f.decl),
            TK::Field(b, _) => matches!(&b.kind, TK::Var(t) if self.b.input_parts.contains(t)),
            _ => false,
        }
    }
    /// `v`, after a check that it is between the two ends, each inclusive.
    pub(super) fn checked_between(
        &mut self,
        v: TExpr,
        lo: Option<TExpr>,
        hi: Option<TExpr>,
        span: Span,
    ) -> TExpr {
        let kind = if self.is_input(&v) {
            CheckKind::ReadRange
        } else {
            CheckKind::RangeStore
        };
        self.checked_as(v, (lo, hi), span, kind)
    }
    /// `v`, trapping as `kind` unless it lies within `lo..=hi`.
    pub(super) fn checked_as(
        &mut self,
        v: TExpr,
        (lo, hi): (Option<TExpr>, Option<TExpr>),
        span: Span,
        kind: CheckKind,
    ) -> TExpr {
        self.b.scopes.push(HashMap::new());
        let tmp = self.bind("$ranged", Ty::Int);
        let var = TExpr {
            kind: TK::Var(tmp),
            ty: Ty::Int,
            span,
        };
        let mut stmts = vec![TStmt::Let(tmp, v)];
        for (bound, op) in [(lo, CmpOp::Ge), (hi, CmpOp::Le)] {
            if let Some(b) = bound {
                let c = TExpr {
                    kind: TK::Cmp(op, Box::new(var.clone()), Box::new(b)),
                    ty: Ty::Bool,
                    span,
                };
                stmts.push(TStmt::Assert(
                    c,
                    "value outside its range".to_string(),
                    span,
                    kind,
                ));
            }
        }
        self.b.scopes.pop();
        TExpr {
            kind: TK::Block(stmts, Some(Box::new(var))),
            ty: Ty::Int,
            span,
        }
    }
    /// `i` as a position of the domain `dty`; a literal that is never one is E0306 (R04.4.25).
    pub(super) fn position(&mut self, dty: Ty, i: TExpr, kind: CheckKind) -> TExpr {
        let span = i.span;
        let within = match &dty {
            Ty::Dom(d) => Some((self.domains[*d].within, self.domains[*d].name.clone())),
            Ty::Range(lo, hi) => Some(((Some(*lo), Some(*hi)), self.show(&dty))),
            _ => None,
        };
        if let (Some(((least, most), name)), TK::Int(k)) = (within, &i.kind) {
            if least.is_some_and(|l| *k < l) || most.is_some_and(|m| *k > m) {
                self.err(
                    span,
                    Msg::verdict(
                        cpl_diag::codes::RANGE_STORE_CONST,
                        format!("`{k}` is never a position of `{name}`"),
                    ),
                );
            }
        }
        TExpr {
            kind: TK::DomFrom(dty.clone(), Box::new(i), kind),
            ty: dty,
            span,
        }
    }
    /// The Int `i` as a cut of the domain `dty`, the count of positions before it; a literal
    /// that is never one is E0306 (R04.4.25).
    pub(super) fn cut_ordinal(&mut self, dty: Ty, i: TExpr) -> TExpr {
        let span = i.span;
        let within = match &dty {
            Ty::Dom(d) => Some((self.domains[*d].within, self.domains[*d].name.clone())),
            Ty::Range(lo, hi) => Some(((Some(*lo), Some(*hi)), self.show(&dty))),
            _ => None,
        };
        if let (Some(((least, most), name)), TK::Int(k)) = (within, &i.kind) {
            if least.is_some_and(|l| *k < l) || most.is_some_and(|m| *k > m.saturating_add(1)) {
                self.err(
                    span,
                    Msg::verdict(
                        cpl_diag::codes::RANGE_STORE_CONST,
                        format!("`{k}` is never a cut of `{name}`"),
                    ),
                );
            }
        }
        TExpr {
            kind: TK::CutOrd(dty, Box::new(i)),
            ty: Ty::Int,
            span,
        }
    }
}

/// Whether an arm's body leaves (returns, breaks, continues or traps) rather than giving a value.
pub(super) fn arm_leaves(b: &next::ArmBody) -> bool {
    b.leaves()
}

/// Whether `f` is a `@memo` fn: its table and clear are made around its body (R06.4.1).
pub(super) fn is_memo(f: &next::FnDecl) -> bool {
    (f.decorators()).any(|x| {
        x.name()
            .and_then(|n| cpl_next_syntax::grammar::Decorator::of(n.text()))
            == Some(cpl_next_syntax::grammar::Decorator::Memo)
    })
}

/// The name a declaration written in a block gives, when it gives one.
fn decl_name(d: &next::Decl) -> Option<String> {
    use next::Decl as D;
    let n = match d {
        D::Fn(x) => x.name(),
        D::Struct(x) => x.name(),
        D::Enum(x) => x.name(),
        D::Trait(x) => x.name(),
        D::TypeAlias(x) => x.name(),
        D::Domain(x) => x.name(),
        D::Const(x) => x.name(),
        D::Context(x) => x.name(),
        D::Mod(x) => x.name(),
        D::Impl(_) | D::Extern(_) | D::Use(_) => None,
    };
    n.map(|t| t.text().to_string())
}

/// The text of an `assert`'s message, a string literal's text parts.
fn assert_text(m: &next::Expr) -> Option<String> {
    let next::Expr::Literal(l) = m else {
        return None;
    };
    let tok = l.token()?;
    if !matches!(tok.kind(), cpl_next_syntax::TokenKind::Str) {
        return None;
    }
    let text = tok.text();
    let parts = cpl_next_syntax::literal::string_parts(text);
    let text_of = |p: &cpl_next_syntax::literal::StrPart| match p {
        cpl_next_syntax::literal::StrPart::Text(s) => {
            cpl_next_syntax::literal::unescape(&text[s.start as usize..s.end as usize])
        }
        cpl_next_syntax::literal::StrPart::Expr(_) => String::new(),
    };
    Some(parts.iter().map(text_of).collect())
}
