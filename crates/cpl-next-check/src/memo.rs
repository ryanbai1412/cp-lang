//! A `@memo` fn declared in a block.

use super::*;

impl Cx<'_> {
    /// A block's `@memo(word)` fn `fd`, `rest` the statements after it: its table `f.memo`, its
    /// body as `f.body`, `f` itself, which answers from the table and else runs the body and
    /// keeps its result, and `f.clear()`. Lower builds `f` and `f.clear` from the `Memo` row; the
    /// body's own calls of `f` go through the table. A fn the table cannot key, or whose word
    /// names no `@memo_table`, is an ordinary nested fn, as decls says why (R06.4.1).
    pub(super) fn memo_fn(&mut self, fd: &next::FnDecl, rest: &[next::Stmt], out: &mut Vec<TStmt>) {
        let Some(name) = fd.name().map(|t| t.text().to_string()) else {
            return;
        };
        let Some((word, table)) = self.memo_table(fd) else {
            self.nested_fn(fd, &name, out);
            return;
        };
        let (key, value) = self.nested_sig(fd, out);
        let n_sets = self.d.s(table).n_sets;
        let sets: Vec<SetTy> = (0..n_sets).map(|_| self.new_set()).collect();
        let key_ty = match &key[..] {
            [k] => k.clone(),
            _ => match self.d.tuple(key.len()) {
                Some(t) => Ty::Struct(t, key.clone(), Vec::new()),
                None => Ty::Error,
            },
        };
        let table_v = self.bind(
            &format!("{name}.memo"),
            Ty::Struct(table, vec![key_ty, value.clone()], sets),
        );
        let fn_ty = Ty::Fn(key.clone(), Box::new(value.clone()), self.new_set());
        let f = self.bind(&name, fn_ty);
        let clear_ty = Ty::Fn(Vec::new(), Box::new(Ty::Unit), self.new_set());
        let clear = self.bind(&cpl_next_decls::memo_clear_name(&name), clear_ty);
        let Some(body) = self.nested_fn(fd, &format!("{name}.body"), out) else {
            return;
        };
        self.memos.push(Memo {
            f,
            body,
            table: table_v,
            clear,
            key,
            value,
            word,
        });
        self.b.memo_clears.insert(f, clear);
        self.memo_writes(fd, rest);
    }

    /// The `@memo(word)` on `fd` and the struct its word's `@memo_table` marks, when `fd` is a
    /// fn the table can key: written parameters, none `mut` or variadic, a written result, no
    /// type parameters and no lend.
    fn memo_table(&mut self, fd: &next::FnDecl) -> Option<(Sym, StructId)> {
        use cpl_next_syntax::grammar::Decorator;
        let x = fd
            .decorators()
            .find(|x| x.name().and_then(|n| Decorator::of(n.text())) == Some(Decorator::Memo))?;
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
        match self.d.named(self.b.module, &cpl_next_decls::memo_key(&w))? {
            decls::Named::Struct(s) => Some((self.sym(&w), s)),
            _ => None,
        }
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
