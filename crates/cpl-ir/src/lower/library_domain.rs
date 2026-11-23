//! Library domains in IR: a struct with `impl Index` is a domain whose
//! positions are its values, indexed through `ord` and walked through `nth`.

use super::call::Reduce;
use super::*;

impl<'a> Lowerer<'a> {
    /// `S.nth(k)` from user code: `k` must lie in `0..S.count`. Walks call the
    /// impl's `nth` directly, unchecked.
    pub(super) fn checked_nth(&mut self, dom: DomainId, call: Expr, span: Span) -> Expr {
        let Expr {
            kind: ExprKind::Call(fid, mut args),
            ty,
            ..
        } = call
        else {
            return call;
        };
        if let Some(Arg::Value(k)) = args.first_mut() {
            let value = std::mem::replace(k, self.int_lit(0, span));
            *k = self.ord_in_range(dom, value);
        }
        self.mk(ExprKind::Call(fid, args), ty, span)
    }

    /// `k`, trapping unless it is an ordinal of the domain, `0..T.count`:
    /// the same check `V.at(k)` makes.
    fn ord_in_range(&mut self, dom: DomainId, k: Expr) -> Expr {
        let pos = self.cast(Cast::FromOrd(dom), k, Ty::Pos(dom));
        self.cast(Cast::Ord, pos, Ty::Int)
    }
}

impl<'a> Lowerer<'a> {
    /// `T.first`, `T.rev`, `T.find(p)`, …: an `impl Index` type is the `Seq`
    /// of its walk (R04.2.20). `first` and `last` call `nth` once, a query
    /// runs over the walk, and what is left reads off `[x for x in T]`.
    pub(super) fn library_domain_seq(
        &mut self,
        base: &ast::Expr,
        sid: StructId,
        name: &ast::Name,
        args: Option<&[ast::Arg]>,
        span: Span,
    ) -> Expr {
        let ends = matches!(name.text.as_str(), "first" | "last") && args.is_none();
        if let (true, Some(dom), Some(&nth)) = (
            ends,
            self.structs[sid].index_domain,
            self.structs[sid].methods.get("nth"),
        ) {
            let opt_ty = Ty::opt(Ty::Struct(sid));
            let len = self.mk(ExprKind::DomainLen(dom), Ty::Int, span);
            let k = match name.text.as_str() {
                "first" => self.int_lit(0, span),
                _ => self.int_add(len.clone(), -1),
            };
            let zero = self.int_lit(0, span);
            let empty = self.mk(
                ExprKind::Cmp(CmpOp::Eq, Box::new(len), Box::new(zero)),
                Ty::Bool,
                span,
            );
            let item = self.mk(
                ExprKind::Call(nth, vec![Arg::Value(k)]),
                Ty::Struct(sid),
                span,
            );
            let some = self.mk(ExprKind::Some(Box::new(item)), opt_ty.clone(), span);
            let none = self.mk(ExprKind::None, opt_ty.clone(), span);
            return self.mk(
                ExprKind::If(Box::new(empty), Box::new(none), Box::new(some)),
                opt_ty,
                span,
            );
        }
        if let Some(query) = args.and_then(|args| self.walk_query(base, name, args, span)) {
            return query;
        }
        let field = ast::Expr {
            kind: ast::ExprKind::Field {
                base: Box::new(ast::Expr::walk_list(base)),
                name: name.clone(),
            },
            span,
        };
        let Some(args) = args else {
            return self.expr(&field, None);
        };
        let call = ast::ExprKind::Call {
            callee: Box::new(field),
            args: args.to_vec(),
        };
        self.expr(&ast::Expr { kind: call, span }, None)
    }
}

impl<'a> Lowerer<'a> {
    /// The `impl Index` struct the type expression `e` names: `T`, an alias of it, a generic's
    /// argument, or an instance `Sub<V>`.
    pub(super) fn index_struct_of(&mut self, e: &ast::Expr) -> Option<StructId> {
        match &e.kind {
            ast::ExprKind::Name(name) if self.lookup(name).is_none() => {
                let Some(Sym::Alias(aid)) = self.get_sym(name) else {
                    return self.index_struct(name);
                };
                let (m, ty_ast, offset) = self.aliases[aid];
                let sm = std::mem::replace(&mut self.cur_mod, m);
                let so = std::mem::replace(&mut self.offset, offset);
                let resolved = self.resolve_ty(ty_ast);
                self.cur_mod = sm;
                self.offset = so;
                match resolved {
                    Ty::Struct(sid) => self.structs[sid].index_domain.is_some().then_some(sid),
                    _ => None,
                }
            }
            ast::ExprKind::Generic { base, args } => {
                let ast::ExprKind::Name(name) = &base.kind else {
                    return None;
                };
                let Some(Sym::GenStruct(gid)) = self.get_sym(name) else {
                    return None;
                };
                let gargs = self.resolve_gen_args_for_struct(gid, args);
                let sid = self.instantiate_struct(gid, gargs);
                self.structs[sid].index_domain.is_some().then_some(sid)
            }
            _ => None,
        }
    }

    /// The `impl Index` struct `name` names, directly or as a generic's argument.
    pub(super) fn index_struct(&self, name: &str) -> Option<StructId> {
        let sid = match self.gen_env.get(name) {
            Some(GenArg::Ty(Ty::Struct(sid))) => *sid,
            Some(_) => return None,
            None => match self.get_sym(name) {
                Some(Sym::Struct(sid)) => sid,
                _ => return None,
            },
        };
        self.structs[sid].index_domain.is_some().then_some(sid)
    }
}

impl<'a> Lowerer<'a> {
    /// The `nth` of the `impl Index` type whose positions `dim` runs over.
    pub(super) fn library_walk_nth(&self, dim: &DimSpec) -> Option<FnId> {
        let DimSpec::Pos(d) = dim else { return None };
        self.structs
            .iter()
            .find(|s| s.index_domain == Some(*d))
            .and_then(|s| s.methods.get("nth").copied())
    }

    /// The walk of a library domain: `nth(k)` for `k` in `0..count`.
    pub(super) fn walk(&mut self, dom: DomainId, nth: FnId, span: Span) -> Iter {
        Iter::Walk {
            k: self.temp(Ty::Int),
            n: self.temp(Ty::Int),
            len: self.mk(ExprKind::DomainLen(dom), Ty::Int, span),
            nth,
            recv: None,
            rev: false,
        }
    }

    /// `T.find(p)`, `T.count(p)`, `T.min(by: f)`, …: the `Seq` query as one
    /// loop over the walk, with no list of the domain's values.
    fn walk_query(
        &mut self,
        base: &ast::Expr,
        name: &ast::Name,
        args: &[ast::Arg],
        span: Span,
    ) -> Option<Expr> {
        let seq = ast::Arg {
            name: None,
            mutable: false,
            value: base.clone(),
        };
        let method = name.text.as_str();
        let positional = |a: &ast::Arg| a.name.is_none() && !a.mutable;
        Some(match args {
            [test] if positional(test) && self.is_test(&test.value) => match method {
                "find" | "first" => self.lower_first_or_last_where(base, &test.value, true, span),
                "rfind" | "last" => self.lower_first_or_last_where(base, &test.value, false, span),
                "count" | "all" | "any" => {
                    let kind = match method {
                        "count" => Reduce::Count,
                        "all" => Reduce::All,
                        _ => Reduce::Any,
                    };
                    let walk = self.tested_walk(method, &[seq, test.clone()], false)?;
                    self.reduce(kind, &walk, span)
                }
                "filter" => self.lower_filter(base, &test.value, span),
                "partition" => self.lower_partition(base, &test.value, span),
                _ => return None,
            },
            [f] if positional(f) && method == "map" => self.lower_map(base, &f.value, span),
            [] if method == "sum" => self.reduce(Reduce::Sum, base, span),
            [] if matches!(method, "min" | "max") => self.min_max(method == "max", &[seq], span),
            [by] if matches!(method, "min" | "max")
                && by.name.as_ref().is_some_and(|n| n.text == "by")
                && !by.mutable =>
            {
                self.min_max_by(method == "max", base, &by.value, span)
            }
            _ => return None,
        })
    }

    /// Expands each `Iter::Walk` in `block` into the loop it stands for, so
    /// no backend sees one: `n = len`, then `k` over `0..n` (from `n - 1`
    /// down when reversed), binding the item to `nth(k)`.
    pub(super) fn expand_walks(&self, block: &mut Block) {
        walk::blocks_mut(block, &mut |b| {
            if !b
                .stmts
                .iter()
                .any(|st| matches!(st, Stmt::For(_, _, Iter::Walk { .. }, _)))
            {
                return;
            }
            let mut stmts = Vec::with_capacity(b.stmts.len() + 1);
            for stmt in std::mem::take(&mut b.stmts) {
                let Stmt::For(
                    lid,
                    item,
                    Iter::Walk {
                        k,
                        n,
                        len,
                        nth,
                        recv,
                        rev,
                    },
                    mut body,
                ) = stmt
                else {
                    stmts.push(stmt);
                    continue;
                };
                let span = len.span;
                let at = |slot: Slot| Expr {
                    kind: ExprKind::Local(slot),
                    ty: Ty::Int,
                    span,
                };
                let ty = self.sigs[nth].ret.clone().unwrap_or(Ty::Unit);
                let value = Expr {
                    kind: ExprKind::Call(
                        nth,
                        recv.map(Arg::Value)
                            .into_iter()
                            .chain([Arg::Value(at(k))])
                            .collect(),
                    ),
                    ty,
                    span,
                };
                body.stmts.insert(0, Stmt::Let(item, Some(value)));
                stmts.push(Stmt::Let(n, Some(len)));
                let ords = Iter::Range {
                    lo: self.int_lit(0, span),
                    hi: at(n),
                    step: None,
                    rev,
                    closed: false,
                };
                stmts.push(Stmt::For(lid, k, ords, body));
            }
            b.stmts = stmts;
        });
    }

    /// `B.subsets` is `Subset<B>` (spec/lib/search-dp.md), std's `impl Index` struct.
    pub(super) fn subsets_alias(&mut self, e: &ast::Expr) -> Option<ast::Expr> {
        let ast::ExprKind::Field { base, name } = &e.kind else {
            return None;
        };
        let ast::ExprKind::Name(n) = &base.kind else {
            return None;
        };
        if name.text != "subsets"
            || self.lookup(n).is_some()
            || self.resolve_domain_name(n).is_none()
        {
            return None;
        }
        let span = base.span;
        let arg = ast::Type {
            kind: ast::TypeKind::Named {
                name: ast::Name {
                    text: n.clone(),
                    span,
                },
                args: Vec::new(),
            },
            span,
        };
        Some(ast::Expr {
            kind: ast::ExprKind::Generic {
                base: Box::new(ast::Expr {
                    kind: ast::ExprKind::Name("Subset".into()),
                    span,
                }),
                args: vec![arg],
            },
            span: e.span,
        })
    }

    /// `Subset<B>`: std's struct over the domain `d`.
    pub(super) fn subset_ty(&mut self, d: DomainId, span: Span) -> Ty {
        match self.get_sym("Subset") {
            Some(Sym::GenStruct(gid)) => {
                Ty::Struct(self.instantiate_struct(gid, vec![GenArg::Domain(d)]))
            }
            _ => {
                self.error(
                    codes::UNKNOWN_TYPE,
                    span,
                    "`Subset` is not in scope".to_string(),
                );
                Ty::Error
            }
        }
    }
}

impl Lowerer<'_> {
    /// Sizes the library domain `id` to `len`: after every statement that
    /// sets a domain or variable `len` reads, or once before `main` when it
    /// reads none.
    pub(super) fn size_library_domain(&mut self, id: DomainId, len: Expr) {
        let size = LibrarySize::Count(len, self.asked_at.at);
        if library_size_inputs(&size).is_empty() {
            let mut stmts = Vec::new();
            self.push_library_size(id, &size, &mut stmts);
            self.preamble_stmts.extend(stmts);
        } else {
            self.library_sizes.insert(id, size);
        }
    }

    /// Sizes each library domain that reads `input`, which a statement just
    /// set, and in turn those built on them. A size waits for
    /// the other inputs its block sets, so a later one is never read unset.
    pub(super) fn library_sizing(
        &self,
        input: SizeInput,
        pending: &dyn Fn(SizeInput) -> bool,
        out: &mut Vec<Stmt>,
    ) {
        for (&id, size) in &self.library_sizes {
            let inputs = library_size_inputs(size);
            if inputs.contains(&input) && !inputs.iter().any(|&i| pending(i)) {
                self.push_library_size(id, size, out);
                self.library_sizing(SizeInput::Domain(id), pending, out);
            }
        }
    }

    pub(super) fn push_library_size(&self, id: DomainId, size: &LibrarySize, out: &mut Vec<Stmt>) {
        let LibrarySize::Count(len, span) = size;
        let span = *span;
        let int = |kind| Expr {
            kind,
            ty: Ty::Int,
            span,
        };
        let len = len.clone();
        out.push(Stmt::DefDomain {
            id,
            lo: int(ExprKind::Int(0)),
            len,
        });
    }

    /// Puts each library domain's sizing after every definition, in
    /// `block`, of a domain it reads.
    /// Inserts the sizing of each library domain after the statements that
    /// set what it reads. `locals` is whether this body's slots are the
    /// ones top-level sizes read: true for `main` and the per-test setup.
    pub(super) fn size_library_domains(&self, block: &mut Block, locals: bool) {
        if self.library_sizes.is_empty() {
            return;
        }
        walk::blocks_mut(block, &mut |b| {
            let sets: Vec<Option<SizeInput>> = b
                .stmts
                .iter()
                .map(|st| size_input_set(st, locals))
                .collect();
            let mut seen = HashSet::new();
            let mut stmts = Vec::with_capacity(b.stmts.len());
            for (stmt, set) in std::mem::take(&mut b.stmts).into_iter().zip(&sets) {
                stmts.push(stmt);
                if let Some(input) = *set {
                    seen.insert(input);
                    let pending = |i: SizeInput| !seen.contains(&i) && sets.contains(&Some(i));
                    self.library_sizing(input, &pending, &mut stmts);
                }
            }
            b.stmts = stmts;
        });
    }
}

/// The domains a library domain's size reads.
fn library_size_inputs(size: &LibrarySize) -> Vec<SizeInput> {
    match size {
        LibrarySize::Count(len, _) => {
            let mut inputs = Vec::new();
            walk::deep_expr(len, &mut |e| {
                let input = match e.kind {
                    ExprKind::DomainLen(d) | ExprKind::DomainLo(d) | ExprKind::DomainVal(d) => {
                        SizeInput::Domain(d)
                    }
                    ExprKind::Local(slot) => SizeInput::Local(slot),
                    ExprKind::Global(g) => SizeInput::Global(g),
                    _ => return,
                };
                if !inputs.contains(&input) {
                    inputs.push(input);
                }
            });
            inputs
        }
    }
}

/// What `stmt` sets that a library domain's size may read.
fn size_input_set(stmt: &Stmt, locals: bool) -> Option<SizeInput> {
    match stmt {
        Stmt::DefDomain { id, .. } | Stmt::DefDomainFromVal { id, .. } => {
            Some(SizeInput::Domain(*id))
        }
        Stmt::Let(slot, Some(_)) if locals => Some(SizeInput::Local(*slot)),
        Stmt::Assign(place, ..) => match place.kind {
            PlaceKind::Local(slot) if locals => Some(SizeInput::Local(slot)),
            PlaceKind::Global(g) => Some(SizeInput::Global(g)),
            _ => None,
        },
        _ => None,
    }
}
