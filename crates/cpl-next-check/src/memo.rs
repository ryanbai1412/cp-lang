//! A `@memo` fn, in a block or at the top level: the fn that answers from its table.

use super::lambda::{LamBody, LamParam};
use super::*;
use cpl_next_decls::views::RoleRows;
use cpl_next_syntax::grammar::RoleMember;

impl Cx<'_> {
    /// A `@memo(word)` fn `fd`, in a block or at the top level, `rest` the statements after it.
    /// It is, by R06.4.1's equivalence, these typed statements, each spanned at `fd`:
    ///
    /// ```text
    /// var $memo = T<K, R>()                      // T the struct `@memo_table(word)` marks
    /// fn $body(params) -> R: <fd's body>         // its calls of `f` are calls of the `f` below
    /// fn f(params) -> R:
    ///     let $got = MemoTable.get($memo, key)   // key: the one argument, or their tuple
    ///     if $got is Some: $got's value
    ///     else: let $r = $body(params); MemoTable.set(mut $memo, key, $r); $r
    /// fn f.clear(): $memo = T<K, R>()
    /// ```
    ///
    /// A fn the table cannot key, or whose word names no `@memo_table`, is an ordinary nested
    /// fn, as decls says why.
    pub(super) fn memo_fn(&mut self, fd: &next::FnDecl, rest: &[next::Stmt], out: &mut Vec<TStmt>) {
        let Some(name) = fd.name().map(|t| t.text().to_string()) else {
            return;
        };
        let top = self.b.decl.is_none() && self.b.scopes.len() == 1;
        // A top-level `@memo` is said by decls, where its fn is declared (R06.4.1).
        let unspelled = |x: &next::Decorator| x.args().is_none_or(|a| a.args().next().is_none());
        if let Some(x) = memo_mark(fd).filter(unspelled).filter(|_| !top) {
            let msg = Msg::verdict(
                cpl_diag::codes::BAD_MEMO,
                "`@memo` spells its table: write `@memo(array)` or `@memo(map)`",
            );
            let at = self.at(&x);
            self.err(at, msg);
        }
        let span = self.at(fd);
        // A block declared its fns before its statements; a top-level one is declared here, as
        // spans count from their own item's start.
        if top || !self.b.group.iter().any(|g| g.0 == span.start) {
            self.declare_nested(&[next::BlockEntry::Decl(next::Decl::Fn(fd.clone()))]);
        }
        let table = memo_table(fd, self.syms, self.names, self.b.module);
        let role = self.d.role_trait(decls::Role::MemoTable);
        let (Some(table), Some(tr)) = (table, role) else {
            self.nested_fn(fd, Some(&name), out);
            return;
        };
        let (key, value) = self.nested_sig(fd);
        let member = |m| self.d.role_member(m);
        let (get, set) = (member(RoleMember::MemoGet), member(RoleMember::MemoSet));
        let (Some((get_name, get)), Some((set_name, set))) = (get, set) else {
            self.nested_fn(fd, Some(&name), out);
            return;
        };
        let key_ty = match &key[..] {
            [k] => k.clone(),
            _ => match self.d.tuple(key.len()) {
                Some(t) => Ty::Struct(t, key.clone(), Vec::new()),
                None => Ty::Error,
            },
        };
        let targs = vec![key_ty.clone(), value.clone()];
        let n_sets = self.items.s(table).n_sets;
        let sets: Vec<SetTy> = (0..n_sets).map(|_| self.new_set()).collect();
        let table_ty = Ty::Struct(table, targs.clone(), sets.clone());
        let tref = decls::TraitRef {
            tr,
            args: targs.clone(),
        };
        // A table without the impl is said at its `@memo_table`, E0327, not again here.
        let rows = self.d.impls_by_trait.get(&tr).into_iter().flatten();
        let filled = rows.map(|k| &self.d.impls[*k].for_);
        if filled
            .into_iter()
            .any(|t| matches!(t, Ty::Struct(s, ..) if *s == table))
        {
            (self.b.obligations).push((
                table_ty.clone(),
                tref.clone(),
                self.b.item.at(span),
                Origin::Bound,
            ));
        }
        self.reach(get);
        self.reach(set);
        let mk = |kind, ty| TExpr { kind, ty, span };
        // The table: made once here, and again by `clear`.
        let fresh = self.construct(table, targs.clone(), sets.clone(), &[], span, None);
        let table_v = self.temp("$memo", table_ty.clone());
        match table_v {
            VarRef::Local(l) => self.b.locals[l].mutable = true,
            VarRef::Global(g) => self.globals[g].mutable = true,
        }
        out.push(TStmt::Let(table_v, fresh.clone()));
        // The body and `f` are one group: the body's `f` is the wrapper, which runs the body.
        let at = span.start;
        let Some(k) = self.b.group.iter().rposition(|g| g.0 == at) else {
            return;
        };
        let body_id = self.b.group[k].2;
        let wrap_id = self.new_lambda(LambdaInfo {
            params: key.clone(),
            ret: value.clone(),
            owner: self.b.decl,
            group: Vec::new(),
            yields: None,
            written: Vec::new(),
        });
        for id in [body_id, wrap_id] {
            self.lambdas[id.0].group = vec![body_id, wrap_id];
        }
        self.b.group.insert(k, (at, Some(name.clone()), wrap_id));
        let Some(body) = self.nested_fn(fd, None, out) else {
            return;
        };
        let body_ty = match body {
            VarRef::Local(k) => self.b.locals[k].ty.clone(),
            VarRef::Global(k) => self.globals[k].ty.clone(),
        };
        let read = move |v: VarRef, ty: Ty| mk(TK::Var(v), ty);
        let (get_name, set_name) = (self.syms.text(get_name), self.syms.text(set_name));
        let (get_name, set_name) = (get_name.to_string(), set_name.to_string());
        let (get_ctx, set_ctx) = (self.trait_ctx(tr, &get_name), self.trait_ctx(tr, &set_name));
        let got_ty = self.optional_ty(value.clone());
        let built = {
            let (table_ty, key_ty, value, tref) = (table_ty.clone(), key_ty, value.clone(), tref);
            move |cx: &mut Cx<'_>, ps: Vec<TExpr>, _: &Ty| -> TExpr {
                let key = match &ps[..] {
                    [k] => k.clone(),
                    _ => mk(TK::Struct(ps.clone()), key_ty.clone()),
                };
                let call = |method: &str, recv: TArg, args: Vec<TArg>, ctx: u32, ty: Ty| {
                    let kind = TK::CallTrait {
                        tr: tref.clone(),
                        method: method.to_string(),
                        recv: Some(Box::new(recv)),
                        self_ty: Box::new(table_ty.clone()),
                        targs: Box::new([]),
                        sets: Box::default(),
                        args,
                        ctx,
                    };
                    mk(kind, ty)
                };
                let table = read(table_v, table_ty.clone());
                let looked = call(
                    &get_name,
                    TArg::Value(table.clone()),
                    vec![TArg::Value(key.clone())],
                    get_ctx,
                    got_ty.clone(),
                );
                let got = cx.temp("$got", got_ty.clone());
                let r = cx.temp("$value", value.clone());
                let run = mk(
                    TK::CallValue {
                        callee: Box::new(mk(TK::Itself(body_id), body_ty.clone())),
                        args: ps.into_iter().map(TArg::Value).collect(),
                    },
                    value.clone(),
                );
                let keep = call(
                    &set_name,
                    TArg::Mut(table),
                    vec![TArg::Value(key), TArg::Value(read(r, value.clone()))],
                    set_ctx,
                    Ty::Unit,
                );
                let miss = mk(
                    TK::Block(
                        vec![TStmt::Let(r, run), TStmt::Expr(keep)],
                        Some(Box::new(read(r, value.clone()))),
                    ),
                    value.clone(),
                );
                let present = mk(TK::Field(Box::new(read(got, got_ty.clone())), 0), Ty::Bool);
                let held = mk(
                    TK::Field(Box::new(read(got, got_ty.clone())), 1),
                    value.clone(),
                );
                let pick = mk(
                    TK::If(Box::new(present), Box::new(held), Box::new(miss)),
                    value.clone(),
                );
                mk(
                    TK::Block(vec![TStmt::Let(got, looked)], Some(Box::new(pick))),
                    value,
                )
            }
        };
        let params = (key.iter())
            .map(|t| LamParam {
                pat: None,
                ty: Some(t.clone()),
                mutable: false,
            })
            .collect();
        self.b.nested = Some(wrap_id);
        self.b.recur = Some(name.clone());
        let wrapper = self.lambda_of(
            params,
            Some(value.clone()),
            LamBody::Built(Box::new(built)),
            None,
            span,
        );
        self.b.nested = None;
        self.b.recur = None;
        let f = self.let_bind(&name, false, wrapper.ty.clone(), wrapper, span, out);
        // `f.clear()`: the table made fresh, as it was before the first call.
        let place = mk(TK::Var(table_v), table_ty);
        let clear_body = mk(TK::Block(vec![TStmt::Assign(place, fresh)], None), Ty::Unit);
        let clear_fn = self.lambda_of(
            Vec::new(),
            Some(Ty::Unit),
            LamBody::Ready(clear_body),
            None,
            span,
        );
        let clear = self.temp(&cpl_next_decls::memo_clear_name(&name), clear_fn.ty.clone());
        out.push(TStmt::Let(clear, clear_fn));
        self.b.memo_clears.insert(f, clear);
        self.memo_writes(fd, rest);
    }

    /// E0311: a `@memo` fn's cache keeps results computed from what it captured, so nothing
    /// may write a captured `var` while the cache lives, to the end of its block (R06.4.1).
    pub(super) fn memo_writes(&mut self, f: &next::FnDecl, rest: &[next::Stmt]) {
        /// The name a place written through `e` starts from.
        fn root(e: &next::Expr) -> Option<String> {
            match e {
                next::Expr::Name(n) => n.ident().map(|t| t.text().to_string()),
                next::Expr::Field(x) => root(&x.base()?),
                next::Expr::Index(x) => root(&x.base()?),
                next::Expr::Paren(x) => root(&x.inner()?),
                _ => None,
            }
        }
        let body = f.body().map(|b| match b {
            next::FnBody::Block(b) => b.syntax().clone(),
            next::FnBody::Expr(e) => e.syntax().clone(),
        });
        let names: Vec<String> = (body.iter().flat_map(|b| b.descendants()))
            .filter_map(next::NameRef::cast)
            .filter_map(|n| n.ident())
            .map(|t| t.text().to_string())
            .collect();
        let params: Vec<String> = (f.params().iter().flat_map(next::ParamList::params))
            .filter_map(|p| p.name())
            .map(|t| t.text().to_string())
            .collect();
        let captured: std::collections::HashSet<String> = (names.into_iter())
            .filter(|n| !params.contains(n))
            .filter(|n| match self.lookup(n) {
                Some((VarRef::Local(l), _)) => self.b.locals[l].mutable,
                Some((VarRef::Global(g), _)) => self.globals[g].mutable,
                None => false,
            })
            .collect();
        if captured.is_empty() {
            return;
        }
        // Each place written below: an assignment's, a `min=`'s, or a `mut` argument's.
        let mut written: Vec<next::Expr> = Vec::new();
        for n in rest.iter().flat_map(|s| s.syntax().descendants()) {
            if let Some(a) = next::AssignStmt::cast(n.clone()) {
                written.extend(a.target());
            } else if let Some(k) = next::KeepExpr::cast(n.clone()) {
                written.extend(k.target());
            } else if let Some(c) = next::CallExpr::cast(n) {
                let args = c
                    .args()
                    .into_iter()
                    .flat_map(|l| l.args().collect::<Vec<_>>());
                written.extend(args.filter(next::Arg::is_mut).filter_map(|a| a.value()));
            }
        }
        let fname = f.name().map_or_else(String::new, |t| t.text().to_string());
        for e in written {
            let Some(n) = root(&e).filter(|n| captured.contains(n)) else {
                continue;
            };
            let text = format!(
                "`{n}` is written while the cache of `@memo` fn `{fname}`, which reads it, lives"
            );
            let span = self.at(&e);
            self.err(span, Msg::verdict(cpl_diag::codes::BAD_MEMO, text));
        }
    }
}

/// The `@memo(word)` on `fd` and the struct its word's `@memo_table` marks, when `fd` is a
/// fn the table can key: written parameters, none `mut` or variadic, a written result, no
/// type parameters and no lend.
fn memo_table(
    fd: &next::FnDecl,
    syms: &Syms,
    names: &dyn views::Names,
    module: ModId,
) -> Option<StructId> {
    let x = memo_mark(fd)?;
    let ps: Vec<next::Param> = fd.params()?.params().collect();
    let plain = |p: &next::Param| p.ty().is_some() && !p.is_mut() && !p.is_variadic();
    let ret = fd.ret()?;
    let typed = ret.ty().is_some() && !ret.lends() && !ps.is_empty() && ps.iter().all(plain);
    let generic = fd
        .generics()
        .is_some_and(|g| g.syntax().children().next().is_some());
    if !typed || generic {
        return None;
    }
    let args: Vec<next::Arg> = x.args()?.args().collect();
    let [a] = &args[..] else {
        return None;
    };
    let Some(next::Expr::Name(w)) = a.value() else {
        return None;
    };
    let w = w.ident()?.text().to_string();
    let key = syms.get(&cpl_next_decls::memo_key(&w))?;
    match names.named(module, key)? {
        decls::Named::Struct(s) => Some(s),
        _ => None,
    }
}

/// The `@memo` written on `fd`.
fn memo_mark(fd: &next::FnDecl) -> Option<next::Decorator> {
    use cpl_next_syntax::grammar::Decorator;
    fd.decorators()
        .find(|x| x.name().and_then(|n| Decorator::of(n.text())) == Some(Decorator::Memo))
}
