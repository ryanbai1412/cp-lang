//! Walks newcomers spell by hand where std names them: the submasks of a
//! mask (`submask_loop`), a ternary search over integers
//! (`ternary_search_by_hand`), a palindrome test (`palindrome_by_hand`),
//! nearest smaller or greater positions (`monotone_stack_by_hand`) and the
//! sort order of positions by value
//! (`argsort_loop`).

use crate::walk::Visitor;
use crate::{
    File, Finding, ARGSORT_LOOP, MONOTONE_STACK_BY_HAND, PALINDROME_BY_HAND, SUBMASK_LOOP,
    TERNARY_SEARCH_BY_HAND,
};
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

/// The std call for `st.len > 0 and a[st.last!] < a[i]` (squashed), the pop
/// condition of a monotone stack of positions into `a`, with what it finds,
/// the stack and the current position.
fn monotone_pop(c: &str) -> Option<(String, &'static str, &str, &str)> {
    let (nonempty, cmp) = c.split_once("and")?;
    let st = nonempty
        .strip_suffix(".len>0")
        .or_else(|| nonempty.strip_suffix(".len!=0"))
        .or_else(|| nonempty.strip_prefix("not")?.strip_suffix(".is_empty"))?;
    let (a, rest) = cmp.split_once(&format!("[{st}.last!]"))?;
    let (op, rhs) = ["<=", ">=", "<", ">"]
        .iter()
        .find_map(|op| Some((*op, rest.strip_prefix(op)?)))?;
    let i = rhs.strip_prefix(&format!("{a}["))?.strip_suffix(']')?;
    if !(ident(st) && ident(a) && ident(i)) {
        return None;
    }
    let (call, what) = match op {
        "<" => ("next_smaller(by: x => -x)", "greater"),
        "<=" => (
            "next_smaller(by: x => -x, strict: false)",
            "greater or equal",
        ),
        ">" => ("next_smaller()", "smaller"),
        _ => ("next_smaller(strict: false)", "smaller or equal"),
    };
    Some((format!("{a}.{call}"), what, st, i))
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

impl HandSearch<'_> {
    /// `span` widened to a `var ok = true` (or `false`) on the line above: the
    /// flag the loop sets is part of the same test.
    fn with_flag(&self, span: cpl_diag::Span) -> cpl_diag::Span {
        let head = &self.src[..span.start as usize];
        let above = head.trim_end_matches([' ', '\t']);
        let above = above.strip_suffix('\n').unwrap_or(above);
        let line_start = above.rfind('\n').map_or(0, |i| i + 1);
        let line = above[line_start..].split("//").next().unwrap_or("").trim();
        let flag =
            line.starts_with("var ") && (line.ends_with("= true") || line.ends_with("= false"));
        if !flag {
            return span;
        }
        let indent = above[line_start..].len() - above[line_start..].trim_start().len();
        cpl_diag::Span::new((line_start + indent) as u32, span.end)
    }

    /// `for i in 0..n / 2:` comparing `s[i]` with `s[n - 1 - i]`.
    fn palindrome(&mut self, stmt: &Stmt, pat: &Pattern, iter: &Expr, body: &Block) {
        let text = |s: cpl_diag::Span| squash(s.shift(self.base).text(self.src));
        let i = text(pat.span);
        let Some(n) = text(iter.span)
            .strip_prefix("0..")
            .and_then(|r| r.strip_suffix("/2"))
            .map(str::to_string)
        else {
            return;
        };
        let whole = text(body.span);
        let mirror = format!("[{n}-1-{i}]");
        let Some(s) = whole.match_indices(&mirror).find_map(|(at, _)| {
            let before = &whole[..at];
            let s = before
                .rsplit(|c: char| !(c.is_alphanumeric() || c == '_'))
                .next()?;
            let before = &before[..before.len() - s.len()];
            ["!=", "=="]
                .iter()
                .any(|op| before.ends_with(&format!("{s}[{i}]{op}")))
                .then(|| s.to_string())
        }) else {
            return;
        };
        if !ident(&i) || !ident(&s) {
            return;
        }
        let diag = Diagnostic::warning(
            PALINDROME_BY_HAND.code,
            self.with_flag(stmt.span.shift(self.base)),
            format!("`{s}[{i}]` against `{s}[{n} - 1 - {i}]` tests whether `{s}` is a palindrome"),
        )
        .with_help(format!(
            "write `is_palindrome({s})` (`use std.strings.{{is_palindrome}}`): a `Bool`, with no mirrored index or half-length bound to get wrong"
        ));
        self.out.push(Finding {
            lint: &PALINDROME_BY_HAND,
            diag,
        });
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
        if let StmtKind::For {
            pat, iter, body, ..
        } = &stmt.kind
        {
            self.palindrome(stmt, pat, iter, body);
        }
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
        let records = |st: &str, i: &str| {
            let mut wrote = false;
            let all = body.stmts.iter().all(|b| {
                let t = text(b.span);
                let popped = t.strip_prefix("let").and_then(|t| {
                    let (j, rhs) = t.split_once('=')?;
                    (ident(j) && (rhs == format!("{st}.last!") || rhs == format!("{st}.pop()!")))
                        .then_some(())
                });
                let answer = t.strip_suffix(&format!("={i}")).is_some_and(|lhs| {
                    lhs.ends_with(']') && lhs.split_once('[').is_some_and(|(ans, _)| ident(ans))
                });
                wrote |= answer;
                popped.is_some() || answer || t == format!("{st}.pop()")
            });
            all && wrote
        };
        if let Some((call, what)) = monotone_pop(&c)
            .filter(|(_, _, st, i)| records(st, i))
            .map(|(c, w, _, _)| (c, w))
        {
            let diag = Diagnostic::warning(
                MONOTONE_STACK_BY_HAND.code,
                span,
                "a monotone stack written by hand".to_string(),
            )
            .with_help(format!(
                "write `{call}` (`std.seq`): each position's nearest later position with a {what} value, as a `[D]?D` with `None` where there is none; `prev_smaller` gives the nearest earlier one"
            ));
            self.out.push(Finding {
                lint: &MONOTONE_STACK_BY_HAND,
                diag,
            });
            return;
        }
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
