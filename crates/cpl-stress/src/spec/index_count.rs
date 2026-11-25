//! How many values a std `impl Index` type over a domain has, read off its
//! declaration: `Pair<V>`'s `let count = V.count * (V.count - 1) / 2`, with
//! the domain's count put in for `V.count` and each module fn it calls
//! (`Subset<B>`'s `subset_count(B.count)`) replaced by its result expression.

use super::*;

pub(super) fn std_index_count(name: &str, n: &Expr) -> Option<Expr> {
    let (_, parsed) = std_file(cpl_facade::Std.module_of(name)?)?;
    let stmts = || parsed.items.iter().flat_map(|(_, item)| &item.stmts);
    let param = stmts().find_map(|s| match &s.kind {
        StmtKind::Struct(decl) if decl.name.text == name => decl.generics.first(),
        _ => None,
    })?;
    let named = |ty: &Type, want: &str| matches!(&ty.kind, TypeKind::Named { name, .. } if name.text == want);
    let body = stmts().find_map(|s| match &s.kind {
        StmtKind::Impl {
            trait_name: Some(t),
            target,
            body,
            ..
        } if named(t, "Index") && named(target, name) => Some(body),
        _ => None,
    })?;
    let count = body.stmts.iter().find_map(|s| match &s.kind {
        StmtKind::Let {
            pat,
            init: Some(init),
            ..
        } if matches!(&pat.kind, PatternKind::Name(c) if c == "count") => Some(init),
        _ => None,
    })?;
    let fns = |f: &str| {
        stmts().find_map(|s| match &s.kind {
            StmtKind::Fn(decl) if decl.name.text == f => Some(decl),
            _ => None,
        })
    };
    let at = Inline {
        param: &param.text,
        n,
        fns: &fns,
    };
    at.expr(count, &HashMap::new(), 0)
}

struct Inline<'a> {
    param: &'a str,
    n: &'a Expr,
    fns: &'a dyn Fn(&str) -> Option<&'static cpl_syntax::ast::FnDecl>,
}

impl Inline<'_> {
    fn expr(&self, e: &Expr, env: &HashMap<&str, Expr>, depth: u32) -> Option<Expr> {
        match &e.kind {
            ExprKind::Int(_) => Some(e.clone()),
            ExprKind::Name(x) => env.get(x.as_str()).cloned(),
            ExprKind::Paren(inner) => self.expr(inner, env, depth),
            ExprKind::Field { base, name } if name.text == "count" => {
                matches!(&base.kind, ExprKind::Name(p) if p == self.param).then(|| self.n.clone())
            }
            ExprKind::Unary { expr, .. } => {
                let mut out = e.clone();
                if let ExprKind::Unary { expr: put, .. } = &mut out.kind {
                    **put = self.expr(expr, env, depth)?;
                }
                Some(out)
            }
            ExprKind::Binary { lhs, rhs, .. } => {
                let mut out = e.clone();
                if let ExprKind::Binary { lhs: l, rhs: r, .. } = &mut out.kind {
                    **l = self.expr(lhs, env, depth)?;
                    **r = self.expr(rhs, env, depth)?;
                }
                Some(out)
            }
            ExprKind::Call { callee, args } if depth < 8 => {
                let ExprKind::Name(f) = &callee.kind else {
                    return None;
                };
                let decl = (self.fns)(f)?;
                let tail: &Expr = match &decl.body {
                    FnBody::Expr(x) => x,
                    FnBody::Block(b) => match &b.stmts.last()?.kind {
                        StmtKind::Expr(x) => x,
                        _ => return None,
                    },
                    FnBody::None => return None,
                };
                let mut inner = HashMap::new();
                for (p, a) in decl.params.iter().zip(args) {
                    inner.insert(p.name.text.as_str(), self.expr(&a.value, env, depth)?);
                }
                self.expr(tail, &inner, depth + 1)
            }
            _ => None,
        }
    }
}
