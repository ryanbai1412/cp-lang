//! The fns that cross to the judge's grader, and the types they cross in
//! (R10.1.3, R10.1.4).

use super::*;

impl<'a> Lowerer<'a> {
    /// The fns that cross to the judge's grader: those it supplies (no
    /// body) and those it calls (`@grader fn` with a body), each checked
    /// to take and give only what a grader can.
    pub(super) fn grader_fns(&mut self, analysis: &'a Analysis) -> (Vec<FnId>, Vec<FnId>) {
        let entries: Vec<&ast::FnDecl> = analysis
            .items
            .iter()
            .flat_map(|item| &item.syntax.parsed.stmts)
            .filter_map(|stmt| match &stmt.kind {
                ast::StmtKind::Fn(decl)
                    if stmt.decorators.iter().any(|d| d.name.text == "grader") =>
                {
                    Some(decl)
                }
                _ => None,
            })
            .collect();
        let (mut judge_fns, mut entry_fns) = (Vec::new(), Vec::new());
        for id in 0..self.sigs.len() {
            let Some(f) = &self.bodies[id] else { continue };
            let decl = self.sigs[id].decl;
            let top_level = self.sigs[id].mod_id == 0 && self.sigs[id].owner.is_none();
            if matches!(decl.body, ast::FnBody::None) {
                judge_fns.push(id);
            } else if top_level
                && (entries.iter().any(|entry| std::ptr::eq(*entry, decl))
                    || self
                        .grader_wrappers
                        .iter()
                        .any(|(_, entry)| std::ptr::eq(*entry, decl)))
            {
                entry_fns.push(id);
            } else {
                continue;
            }
            let name = &decl.name.text;
            let changed = f.params.iter().position(|p| p.mutable);
            let tys: Vec<Ty> = f
                .locals
                .iter()
                .take(f.params.len())
                .map(|l| l.ty.clone())
                .collect();
            let ret = f.ret.clone();
            let saved = (self.cur_mod, self.offset);
            (self.cur_mod, self.offset) = (self.sigs[id].mod_id, self.sigs[id].offset);
            if let Some(at) = changed {
                let param = decl.params[at].name.text.clone();
                self.error(
                    codes::UNSUPPORTED,
                    decl.params[at].name.span,
                    format!("the judge's grader can't lend `{param}` to `{name}` to change"),
                );
                self.help(format!("take `{param}` by value and return what changed"));
            } else {
                let (params, result) = self.sig_ranges.get(&id).cloned().unwrap_or_default();
                let at = |i: usize| params.get(i).and_then(Option::as_ref);
                let sides: Result<Vec<JudgeTy>, Uncrossable> = (tys.iter().enumerate())
                    .map(|(i, ty)| judge_ty(ty, at(i)))
                    .collect();
                match sides.and_then(|sides| Ok((sides, judge_ty(&ret, result.as_ref())?))) {
                    Ok((sides, ret)) => self.judge_tys.push((id, sides, ret)),
                    Err(Uncrossable::Type(ty)) => {
                        let shown = self.show(&ty);
                        self.error(
                            codes::UNSUPPORTED,
                            decl.name.span,
                            format!("a `{shown}` can't cross to the judge's grader in `{name}`"),
                        );
                        self.help("a `@grader fn` takes and gives `Int`, `F64`, `Bool`, `Char`, `Str`, and lists and tuples of them");
                    }
                    Err(Uncrossable::Open(open)) => {
                        self.error(
                            codes::UNSUPPORTED,
                            decl.name.span,
                            format!("`{open}` can't cross to the judge's grader in `{name}`: no C++ integer holds a range with an open end"),
                        );
                        self.help("close the range, as in `0..=1e18`, or write `Int` for the grader's `int`");
                    }
                }
            }
            (self.cur_mod, self.offset) = saved;
        }
        // Every `@grader fn` the grader calls is built, or the build fails.
        let built = |decl: &ast::FnDecl| {
            let entry = (self.grader_wrappers.iter())
                .find(|(source, _)| std::ptr::eq(*source, decl))
                .map_or(decl, |(_, entry)| *entry);
            entry_fns
                .iter()
                .any(|&id| std::ptr::eq(self.sigs[id].decl, entry))
        };
        let dropped: Vec<&ast::FnDecl> = (entries.iter().copied())
            .filter(|decl| !matches!(decl.body, ast::FnBody::None) && !built(decl))
            .collect();
        for decl in dropped {
            let name = &decl.name.text;
            self.error(
                codes::UNSUPPORTED,
                decl.name.span,
                format!("`{name}` can't be built for the judge's grader"),
            );
            self.help("a `@grader fn` is a top-level fn of the program's own file");
        }
        (judge_fns, entry_fns)
    }
}

/// The C++ entry the grader calls for `decl`, a `@grader fn` over its own
/// domains: each `[C]T` it takes is a `List<T>`, each `C` it gives an `Int`,
/// and its body calls `inner`, the fn itself, which binds `C` to the list's
/// length.
pub(super) fn entry_over_lists(decl: &ast::FnDecl, inner: &str) -> ast::FnDecl {
    let own: Vec<&str> = decl.generics.iter().map(|g| g.text.as_str()).collect();
    let at = decl.name.span;
    let name = |text: &str| ast::Expr {
        kind: ast::ExprKind::Name(text.to_string()),
        span: at,
    };
    let args = (decl.params.iter())
        .map(|p| ast::Arg {
            name: None,
            mutable: false,
            value: name(&p.name.text),
        })
        .collect();
    ast::FnDecl {
        name: decl.name.clone(),
        generics: Vec::new(),
        bounds: Vec::new(),
        params: (decl.params.iter())
            .map(|p| ast::Param {
                ty: p.ty.as_ref().map(|t| over_lists(t, &own)),
                ..p.clone()
            })
            .collect(),
        ret: decl.ret.as_ref().map(|t| over_lists(t, &own)),
        lends: None,
        uses: decl.uses.clone(),
        body: ast::FnBody::Expr(ast::Expr {
            kind: ast::ExprKind::Call {
                callee: Box::new(name(inner)),
                args,
            },
            span: at,
        }),
        sig_span: decl.sig_span,
    }
}

fn over_lists(t: &ast::Type, own: &[&str]) -> ast::Type {
    let is_own = |t: &ast::Type| {
        matches!(&t.kind, ast::TypeKind::Named { name, args }
            if args.is_empty() && own.contains(&name.text.as_str()))
    };
    let named = |text: &str, args| ast::TypeKind::Named {
        name: ast::Name {
            text: text.to_string(),
            span: t.span,
        },
        args,
    };
    let kind = match &t.kind {
        _ if is_own(t) => named("Int", Vec::new()),
        ast::TypeKind::Array { index, elem } if index.len() == 1 && is_own(&index[0]) => {
            named("List", vec![over_lists(elem, own)])
        }
        ast::TypeKind::Array { index, elem } => ast::TypeKind::Array {
            index: index.clone(),
            elem: Box::new(over_lists(elem, own)),
        },
        ast::TypeKind::Named { name, args } => ast::TypeKind::Named {
            name: name.clone(),
            args: args.iter().map(|a| over_lists(a, own)).collect(),
        },
        ast::TypeKind::Tuple(ts) => {
            ast::TypeKind::Tuple(ts.iter().map(|a| over_lists(a, own)).collect())
        }
        ast::TypeKind::Record(fs) => ast::TypeKind::Record(
            fs.iter()
                .map(|(n, a)| (n.clone(), over_lists(a, own)))
                .collect(),
        ),
        ast::TypeKind::Optional(a) => ast::TypeKind::Optional(Box::new(over_lists(a, own))),
        other => other.clone(),
    };
    ast::Type { kind, span: t.span }
}
