//! Walks newcomers spell by hand where std names them: the submasks of a
//! mask (`submask_loop`), a ternary search over integers
//! (`ternary_search_by_hand`) and the sort order of positions by value
//! (`argsort_loop`).

use crate::walk::Visitor;
use crate::{File, Finding, ARGSORT_LOOP, SUBMASK_LOOP, TERNARY_SEARCH_BY_HAND};
use cpl_diag::Diagnostic;
use cpl_syntax::ast::*;

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let mut v = HandSearch {
        src: file.src,
        base: 0,
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}

struct HandSearch<'a> {
    src: &'a str,
    base: u32,
    out: Vec<Finding>,
}

fn squash(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

fn ident(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

/// `(x[i],i)` at the start of `s`, squashed: `(x, i)`.
fn value_index(s: &str) -> Option<(&str, &str)> {
    let (x, rest) = s.strip_prefix('(')?.split_once('[')?;
    let (i, rest) = rest.split_once(']')?;
    let ok = ident(x) && ident(i) && rest.starts_with(&format!(",{i})"));
    ok.then_some((x, i))
}

impl HandSearch<'_> {
    /// The pairs sorted by `sorted`: `(x, i, range)` when `recv` is a list of
    /// `(x[i], i)` built by a comprehension or a push loop over `range`.
    fn argsort(&self, recv: &str) -> Option<(String, String, String)> {
        let src = squash(self.src);
        let comp = |c: &str| {
            let (x, i) = value_index(c.strip_prefix('[')?)?;
            let rest = c.split_once(&format!("for{i}in"))?.1;
            let range = rest.split([']', ':']).next()?;
            Some((x.to_string(), i.to_string(), range.to_string()))
        };
        if recv.starts_with("[(") {
            return comp(recv);
        }
        if !ident(recv) {
            return None;
        }
        if let Some(at) = src.find(&format!("{recv}=[(")) {
            return comp(&src[at + recv.len() + 1..]);
        }
        let at = src.find(&format!("{recv}.push(("))?;
        let (x, i) = value_index(&src[at + recv.len() + 6..])?;
        let head = &src[..at];
        let start = head.rfind(&format!("for{i}in"))? + i.len() + 5;
        let range = head[start..].split(':').next()?;
        Some((x.to_string(), i.to_string(), range.to_string()))
    }
}

impl Visitor for HandSearch<'_> {
    fn expr(&mut self, expr: &Expr) {
        let span = expr.span.shift(self.base);
        let text = squash(span.text(self.src));
        let Some(recv) = text.strip_suffix(".sorted()") else {
            return;
        };
        let Some((x, i, range)) = self.argsort(recv) else {
            return;
        };
        let diag = Diagnostic::warning(
            ARGSORT_LOOP.code,
            span,
            format!("pairs `({x}[{i}], {i})` sorted to get the positions in order of `{x}`"),
        )
        .with_help(format!(
            "sort the positions by value: `({range}).sorted(by: {i} => {x}[{i}])`; the sort is stable, so equal values keep their position order as the pairs did, and `{x}[{i}]` is read where it is needed"
        ));
        self.out.push(Finding {
            lint: &ARGSORT_LOOP,
            diag,
        });
    }

    fn stmt(&mut self, stmt: &Stmt) {
        let StmtKind::While {
            cond: Cond::Expr(cond),
            body,
            else_body: None,
        } = &stmt.kind
        else {
            return;
        };
        let span = stmt.span.shift(self.base);
        let text = |s: cpl_diag::Span| squash(s.shift(self.base).text(self.src));
        let c = text(cond.span);
        let Some(last) = body.stmts.last() else {
            return;
        };
        if let Some(s) = c.strip_suffix(">0").or_else(|| c.strip_suffix("!=0")) {
            let step = text(last.span);
            let mask = step.strip_prefix(&format!("{s}=({s}-1)&"));
            if let Some(m) = mask.filter(|m| ident(s) && ident(m)) {
                let diag = Diagnostic::warning(
                    SUBMASK_LOOP.code,
                    span,
                    format!("`{s} = ({s} - 1) & {m}` walks the submasks of `{m}` by hand"),
                )
                .with_help(format!(
                    "write `for {s} in submasks({m}):` (`use std.math.{{submasks}}`): it yields every submask of `{m}`, largest first, and `0` last, so skip `0` there if the loop must not see it"
                ));
                self.out.push(Finding {
                    lint: &SUBMASK_LOOP,
                    diag,
                });
            }
            return;
        }
        let whole = text(body.span);
        if whole.matches(")/3").count() >= 2 && body.stmts.len() >= 3 {
            let diag = Diagnostic::warning(
                TERNARY_SEARCH_BY_HAND.code,
                span,
                "a ternary search over integers written by hand".to_string(),
            )
            .with_help(
                "write `ternary_min(lo..hi, f)!` or `ternary_max(lo..hi, f)!` (`std.search`): golden-section search, about \\(\\log_\\varphi n\\) calls of `f`, with no stopping rule or final scan to get wrong",
            );
            self.out.push(Finding {
                lint: &TERNARY_SEARCH_BY_HAND,
                diag,
            });
        }
    }
}
