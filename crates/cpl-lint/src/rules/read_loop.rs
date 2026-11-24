//! A list filled by a loop that reads one item per step (`read_loop`): one
//! typed `read()` takes every item and checks its bounds.

use crate::facts::Facts;
use crate::std_fn;
use crate::walk::{unparen, Visitor};
use crate::{is_std, File, Finding, READ_LOOP};
use cpl_check::Ty;
use cpl_diag::Span;
use cpl_syntax::ast::*;

use super::shapes::{name, squash};
use cpl_diag::msgs::lint_rules;

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let mut v = ReadLoop {
        src: file.src,
        facts: file.facts,
        base: 0,
        out: Vec::new(),
    };
    let top: Vec<_> = file.top_level().collect();
    v.scan(&top);
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}

struct ReadLoop<'a> {
    src: &'a str,
    facts: Option<&'a Facts>,
    base: u32,
    out: Vec<Finding>,
}

/// `read()`.
pub(crate) fn is_read(e: &Expr) -> bool {
    matches!(&unparen(e).kind, ExprKind::Call { callee, args } if args.is_empty() && name(callee) == Some("read"))
}

/// `xs.push(e)`: `xs` and `e`.
fn push<'s>(stmt: &'s Stmt, facts: Option<&Facts>, base: u32) -> Option<(&'s str, &'s Expr)> {
    let StmtKind::Expr(e) = &stmt.kind else {
        return None;
    };
    let ExprKind::Call { callee, args } = &e.kind else {
        return None;
    };
    let ExprKind::Field { base: list, .. } = &callee.kind else {
        return None;
    };
    match args.as_slice() {
        [arg] if arg.name.is_none() && is_std(facts, e.span.shift(base), std_fn::PUSH) => {
            Some((name(list)?, &arg.value))
        }
        _ => None,
    }
}

impl ReadLoop<'_> {
    fn text(&self, base: u32, span: Span) -> &str {
        span.shift(base).text(self.src)
    }

    /// The item type the loop reads, from its `let` or from `List<T>`.
    fn item(&self, base: u32, body: &Block, list_ty: Option<&Ty>) -> Option<(String, String)> {
        let from_list = || Some(list_ty?.held_elem()?.to_string());
        match body.stmts.as_slice() {
            [s] => {
                let (xs, v) = push(s, self.facts, base)?;
                is_read(v).then(|| Some((xs.to_string(), from_list()?)))?
            }
            [read, s] => {
                let StmtKind::Let {
                    pat,
                    ty,
                    init: Some(init),
                    otherwise: None,
                    guard: None,
                    ..
                } = &read.kind
                else {
                    return None;
                };
                let (xs, v) = push(s, self.facts, base)?;
                if !is_read(init)
                    || squash(self.text(base, v.span)) != squash(self.text(base, pat.span))
                {
                    return None;
                }
                let t = match ty {
                    Some(t) => self.text(base, t.span).to_string(),
                    None => from_list()?,
                };
                Some((xs.to_string(), t))
            }
            _ => None,
        }
    }

    fn scan(&mut self, stmts: &[(u32, &Stmt)]) {
        for pair in stmts.windows(2) {
            let [(d_base, decl), (f_base, f)] = pair else {
                continue;
            };
            let StmtKind::Let {
                mutable: true,
                pat,
                ty,
                init: Some(init),
                ..
            } = &decl.kind
            else {
                continue;
            };
            let (PatternKind::Name(xs), ExprKind::List(items)) = (&pat.kind, &unparen(init).kind)
            else {
                continue;
            };
            let StmtKind::For {
                pat: each,
                iter,
                body,
                else_body: None,
                ..
            } = &f.kind
            else {
                continue;
            };
            let (true, true, Some((pushed, t))) = (
                items.is_empty(),
                matches!(each.kind, PatternKind::Wildcard),
                self.item(
                    *f_base,
                    body,
                    ty.as_ref()
                        .and(self.facts)
                        .and_then(|f| f.place_type(pat.span.shift(*d_base))),
                ),
            ) else {
                continue;
            };
            let range = self.text(*f_base, iter.span).to_string();
            if pushed != xs.as_str() || !matches!(unparen(iter).kind, ExprKind::Range { .. }) {
                continue;
            }
            let span = Span::new(decl.span.start + d_base, iter.span.end + f_base);
            let diag = lint_rules::READ_LOOP.warning(span, &[&xs, &range])
            .with_help(format!(
                "write `let {xs}: [{range}]{t} = read()`: one read takes every item and checks its bounds (`var` if it changes later)"
            ));
            self.out.push(Finding {
                lint: &READ_LOOP,
                diag,
            });
        }
    }
}

impl Visitor for ReadLoop<'_> {
    fn stmts(&mut self, stmts: &[Stmt]) {
        let base = self.base;
        let stmts: Vec<_> = stmts.iter().map(|s| (base, s)).collect();
        self.scan(&stmts);
    }
}
