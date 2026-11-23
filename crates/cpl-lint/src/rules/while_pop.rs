//! `while_not_empty_pop` (L0591): `while not q.is_empty:` whose body starts
//! `let x = q.pop()!` tests and unwraps one fact twice; `while let x = q.pop():`
//! says it once and stops on the same `None`.

use crate::walk::Visitor;
use crate::{File, Finding, COALESCE_TEST_THEN_UNWRAP, WHILE_NOT_EMPTY_POP};
use cpl_diag::{Diagnostic, Fix, Span};
use cpl_syntax::ast::{BinOp, CmpOp, Cond, Expr, ExprKind, Stmt, StmtKind};

pub(crate) fn check(file: &File) -> Vec<Finding> {
    struct Loops<'a> {
        src: &'a str,
        base: u32,
        out: Vec<Finding>,
    }
    impl Visitor for Loops<'_> {
        fn stmt(&mut self, stmt: &Stmt) {
            self.visit(stmt);
        }
    }
    impl Loops<'_> {
        fn text(&self, span: Span) -> &str {
            span.shift(self.base).text(self.src)
        }
        fn visit(&mut self, stmt: &Stmt) {
            let StmtKind::While {
                cond: Cond::Expr(cond),
                body,
                else_body: None,
            } = &stmt.kind
            else {
                return;
            };
            let Some(queue) = nonempty_test(self.text(cond.span)) else {
                return;
            };
            let [first, _, ..] = body.stmts.as_slice() else {
                return;
            };
            let StmtKind::Let {
                mutable: false,
                pat,
                ty: None,
                init: Some(init),
                otherwise: None,
                guard: None,
            } = &first.kind
            else {
                return;
            };
            let Some((popped, pop)) = unwrapped_pop(init) else {
                return;
            };
            if self.text(popped.span) != queue {
                return;
            }
            let (src, base) = (self.src, self.base as usize);
            let (start, end) = (
                base + first.span.start as usize,
                base + first.span.end as usize,
            );
            let line = src[..start].rfind('\n').map_or(0, |i| i + 1);
            if !src[line..start].trim().is_empty() || !src[end..].starts_with('\n') {
                return;
            }
            let p = self.text(pat.span);
            let take = format!("let {p} = {queue}.{pop}()");
            let mut diag = Diagnostic::warning(
                WHILE_NOT_EMPTY_POP.code,
                Span::new(stmt.span.start, first.span.end).shift(self.base),
                format!("`{queue}` is tested for empty, then `{queue}.{pop}()!` unwraps"),
            )
            .with_label(first.span.shift(self.base), "the pop the test guards")
            .with_help(format!(
                "write `while {take}:`; the loop stops at the same `None`, with no test and unwrap to keep in step"
            ));
            diag.fixes.push(Fix {
                title: format!("use `while {take}:`"),
                edits: vec![
                    (cond.span.shift(self.base), take),
                    (Span::new(line as u32, end as u32 + 1), String::new()),
                ],
            });
            self.out.push(Finding {
                lint: &WHILE_NOT_EMPTY_POP,
                diag,
            });
        }
    }
    let mut v = Loops {
        src: file.src,
        base: 0,
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}

/// The `q` of `not q.is_empty`, `q.len > 0` or `q.len != 0`, a name or field path.
fn nonempty_test(cond: &str) -> Option<&str> {
    let cond = cond.trim();
    let queue = cond
        .strip_prefix("not ")
        .and_then(|c| c.trim().strip_suffix(".is_empty"))
        .or_else(|| {
            cond.strip_suffix("> 0")
                .or_else(|| cond.strip_suffix("!= 0"))
                .and_then(|c| c.trim_end().strip_suffix(".len"))
        })?;
    let path = !queue.is_empty()
        && queue
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '.');
    path.then_some(queue)
}

/// `q.pop()!` (or `pop_front`, `pop_first`, ...): the queue and the method.
/// Any no-argument method qualifies: the loop guards it with `not q.is_empty`,
/// so it is one whose `None` means empty, the contract `while let` relies on.
fn unwrapped_pop(init: &Expr) -> Option<(&Expr, &str)> {
    let ExprKind::Unwrap(call) = &init.kind else {
        return None;
    };
    let ExprKind::Call { callee, args } = &call.kind else {
        return None;
    };
    let ExprKind::Field { base, name } = &callee.kind else {
        return None;
    };
    args.is_empty()
        .then_some((base.as_ref(), name.text.as_str()))
}

/// `coalesce_test_then_unwrap` (L0595): `while (e ?? c) > y:` whose body starts
/// `let w = e!` gives `e` a stand-in for `None` and then asserts it was never
/// `None`; `while let w = e, w > y:` binds it once and stops on `None`.
pub(crate) fn coalesce(file: &File) -> Vec<Finding> {
    struct Tests<'a> {
        src: &'a str,
        base: u32,
        out: Vec<Finding>,
    }
    impl Visitor for Tests<'_> {
        fn stmt(&mut self, stmt: &Stmt) {
            if let StmtKind::While {
                cond: Cond::Expr(cond),
                body,
                ..
            } = &stmt.kind
            {
                self.visit("while", stmt.span, cond, &body.stmts);
            }
        }
        fn expr(&mut self, expr: &Expr) {
            let ExprKind::If { branches, .. } = &expr.kind else {
                return;
            };
            for (i, (cond, body)) in branches.iter().enumerate() {
                if let Cond::Expr(cond) = cond {
                    let kw = if i == 0 { "if" } else { "elif" };
                    let start = if i == 0 {
                        expr.span.start
                    } else {
                        cond.span.start
                    };
                    self.visit(kw, Span::new(start, cond.span.end), cond, &body.stmts);
                }
            }
        }
    }
    impl Tests<'_> {
        fn text(&self, span: Span) -> &str {
            span.shift(self.base).text(self.src)
        }
        fn visit(&mut self, kw: &str, whole: Span, cond: &Expr, body: &[Stmt]) {
            let ExprKind::Compare { first, rest } = &cond.kind else {
                return;
            };
            let [(op, rhs)] = rest.as_slice() else {
                return;
            };
            if matches!(op, CmpOp::In | CmpOp::NotIn | CmpOp::Order) {
                return;
            }
            let ExprKind::Paren(inner) = &first.kind else {
                return;
            };
            let ExprKind::Binary {
                op: BinOp::Coalesce,
                lhs: opt,
                rhs: stand_in,
                ..
            } = &inner.kind
            else {
                return;
            };
            let [head, _, ..] = body else {
                return;
            };
            let StmtKind::Let {
                mutable: false,
                pat,
                ty: None,
                init: Some(init),
                otherwise: None,
                guard: None,
            } = &head.kind
            else {
                return;
            };
            let ExprKind::Unwrap(unwrapped) = &init.kind else {
                return;
            };
            let e = self.text(opt.span);
            // A call could answer differently the second time it runs.
            if e.contains('(') || self.text(unwrapped.span) != e {
                return;
            }
            let p = self.text(pat.span);
            if p.is_empty() || !p.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return;
            }
            let (src, base) = (self.src, self.base as usize);
            let (start, end) = (
                base + head.span.start as usize,
                base + head.span.end as usize,
            );
            let line = src[..start].rfind('\n').map_or(0, |i| i + 1);
            if !src[line..start].trim().is_empty() || !src[end..].starts_with('\n') {
                return;
            }
            let c = self.text(stand_in.span);
            let tail = self.text(Span::new(first.span.end, rhs.span.end));
            let header = format!("let {p} = {e}, {p}{tail}");
            let mut diag = Diagnostic::warning(
                COALESCE_TEST_THEN_UNWRAP.code,
                Span::new(whole.start, head.span.end).shift(self.base),
                format!("`{e}` is tested with the stand-in `?? {c}`, then `{e}!` unwraps it"),
            )
            .with_label(head.span.shift(self.base), "the unwrap the test guards")
            .with_help(format!(
                "write `{kw} {header}:`; it compares the real value and skips on `None`, with no stand-in to pick and no `!` to keep in step"
            ));
            diag.fixes.push(Fix {
                title: format!("use `{kw} {header}:`"),
                edits: vec![
                    (cond.span.shift(self.base), header),
                    (Span::new(line as u32, end as u32 + 1), String::new()),
                ],
            });
            self.out.push(Finding {
                lint: &COALESCE_TEST_THEN_UNWRAP,
                diag,
            });
        }
    }
    let mut v = Tests {
        src: file.src,
        base: 0,
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}
