//! `test_count_loop` (L0593): the file's first read is a count `t`, and a
//! `for _ in 0..t:` holding the rest of the program runs each case by hand.
//! The `@!multitest` divider reads the count and runs what is below it per
//! case, unindented, with the same per-case state.
//!
//! A loop whose one read is a query answered from a table built before the
//! count (`let pre = Prefix(..)`, then `for _ in 0..q:` reading `(l, r)`)
//! is a query loop, not test cases, and is left alone.

use crate::rules::read_loop::is_read;
use crate::walk::{walk_stmt, Scope, Visitor};
use crate::{File, Finding, TEST_COUNT_LOOP};
use cpl_diag::{Diagnostic, Fix, Span};
use cpl_syntax::ast::{Block, Expr, PatternKind, Stmt, StmtKind};

pub(crate) fn check(file: &File) -> Vec<Finding> {
    if file.src.contains("@!multitest") {
        return Vec::new();
    }
    let tops: Vec<(u32, &Stmt)> = file
        .items
        .iter()
        .flat_map(|(base, item)| item.stmts.iter().map(move |s| (*base, s)))
        .collect();
    let Some(k) = tops.iter().position(|(_, s)| reads(s)) else {
        return Vec::new();
    };
    let ((base, count), Some(&(loop_base, lp))) = (tops[k], tops.get(k + 1)) else {
        return Vec::new();
    };
    let declarations_after = tops[k + 2..].iter().all(|(_, s)| {
        matches!(
            s.kind,
            StmtKind::Fn(_)
                | StmtKind::Struct(_)
                | StmtKind::Enum(_)
                | StmtKind::TypeAlias { .. }
                | StmtKind::Trait { .. }
                | StmtKind::Impl { .. }
        )
    });
    if !declarations_after || !count.decorators.is_empty() || !lp.decorators.is_empty() {
        return Vec::new();
    }
    let src = file.src;
    let StmtKind::Let {
        mutable: false,
        pat,
        init: Some(init),
        otherwise: None,
        ..
    } = &count.kind
    else {
        return Vec::new();
    };
    let PatternKind::Name(t) = &pat.kind else {
        return Vec::new();
    };
    let StmtKind::For {
        label: None,
        pat: each,
        iter,
        body,
        else_body: None,
    } = &lp.kind
    else {
        return Vec::new();
    };
    let iter_text = iter.span.shift(loop_base).text(src);
    if !is_read(init)
        || !matches!(each.kind, PatternKind::Wildcard)
        || (iter_text != format!("0..{t}") && iter_text != format!("1..={t}"))
        || !runs_alone(body)
    {
        return Vec::new();
    }
    let (let_at, loop_at) = (count.span.shift(base), lp.span.shift(loop_base));
    let between = &src[let_at.end as usize..loop_at.start as usize];
    let loop_text = loop_at.text(src);
    let Some(header_end) = loop_text.find('\n') else {
        return Vec::new();
    };
    let body_text = &loop_text[header_end + 1..];
    let words = || body_text.split(|c: char| !c.is_alphanumeric() && c != '_');
    let built: Vec<String> = tops[..k]
        .iter()
        .filter_map(|(_, s)| bound_name(s))
        .collect();
    let answers_queries = body.stmts.iter().filter(|s| reads(s)).count() == 1
        && body.stmts.first().is_some_and(reads)
        && words().any(|w| built.iter().any(|b| b == w));
    if !between.trim().is_empty()
        || body_text.contains("\"\"\"")
        || words().any(|w| w == t)
        || answers_queries
    {
        return Vec::new();
    }
    let indent = &body_text[..body_text.len() - body_text.trim_start().len()];
    let lines: Vec<&str> = body_text
        .split('\n')
        .map(|l| l.strip_prefix(indent).unwrap_or(l.trim_start()))
        .collect();
    let replacement = format!("@!multitest\n{}", lines.join("\n"));
    let span = Span::new(let_at.start, loop_at.end);
    let mut diag = Diagnostic::warning(
        TEST_COUNT_LOOP.code,
        Span::new(let_at.start, loop_at.start + header_end as u32),
        format!("`for _ in {iter_text}:` runs the test cases by hand"),
    )
    .with_help(format!(
        "write `@!multitest` in place of `let {t} = read()` and the loop, and unindent its body; \
         the divider reads the count, and code above it runs once"
    ));
    diag.fixes.push(Fix {
        title: "use `@!multitest`".into(),
        edits: vec![(span, replacement)],
    });
    vec![Finding {
        lint: &TEST_COUNT_LOOP,
        diag,
    }]
}

/// The name a top-level `let`, `var` or `const` binds.
fn bound_name(stmt: &Stmt) -> Option<String> {
    match &stmt.kind {
        StmtKind::Let { pat, .. } => match &pat.kind {
            PatternKind::Name(n) => Some(n.to_string()),
            _ => None,
        },
        StmtKind::Const { name, .. } => Some(name.text.clone()),
        _ => None,
    }
}

/// Whether `stmt` reads input anywhere.
fn reads(stmt: &Stmt) -> bool {
    struct Reads(bool);
    impl Visitor for Reads {
        fn expr(&mut self, e: &Expr) {
            self.0 |= is_read(e);
        }
    }
    let mut v = Reads(false);
    walk_stmt(&mut v, stmt);
    v.0
}

/// Whether `body` means the same as one case: no `return` (which would end
/// every case, not this one) and no `break` / `continue` of the case loop.
fn runs_alone(body: &Block) -> bool {
    #[derive(Default)]
    struct Escapes {
        fns: u32,
        loops: u32,
        found: bool,
    }
    impl Visitor for Escapes {
        fn stmt(&mut self, s: &Stmt) {
            self.found |= match &s.kind {
                StmtKind::Return(_) => self.fns == 0,
                StmtKind::Break(label) | StmtKind::Continue(label) => {
                    label.is_some() || self.loops == 0
                }
                _ => false,
            };
        }
        fn enter(&mut self, scope: Scope) {
            match scope {
                Scope::Fn => self.fns += 1,
                Scope::Loop => self.loops += 1,
                Scope::Block => {}
            }
        }
        fn exit(&mut self, scope: Scope) {
            match scope {
                Scope::Fn => self.fns -= 1,
                Scope::Loop => self.loops -= 1,
                Scope::Block => {}
            }
        }
    }
    let mut v = Escapes::default();
    for s in &body.stmts {
        walk_stmt(&mut v, s);
    }
    !v.found
}
