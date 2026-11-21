//! Taste lints: code that works but reads like translated C++
//! (`plan/PORTING.md`, "Taste checklist"). Each names the construct the
//! language has for the idea, and rewrites to it where the rewrite cannot
//! change what the program does.

use crate::walk::{self, Bind, Visitor};
use crate::{
    File, Finding, DISCARDED_RESULT, INDEX_ONLY_LOOP, LOOP_REDUCTION, REPEATED_LITERAL,
    SEARCH_LOOP, SYMMETRIC_WRITE,
};
use cpl_diag::{Diagnostic, Fix, Span};
use cpl_syntax::ast::*;
use std::collections::{HashMap, HashSet};

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let mut rule = Taste {
        src: file.src,
        base: 0,
        next_base: 0,
        valued_fns: HashSet::new(),
        declared: HashSet::new(),
        idents: idents(file.src),
        findings: Vec::new(),
    };
    for (_, stmt) in file.top_level() {
        if let StmtKind::Fn(decl) = &stmt.kind {
            if decl.ret.is_some() {
                rule.valued_fns.insert(decl.name.text.clone());
            }
        }
    }
    let top: Vec<(u32, &Stmt)> = file.top_level().collect();
    rule.sequence(&top);
    struct Declared<'s>(&'s mut HashSet<String>);
    impl Visitor for Declared<'_> {
        fn stmt(&mut self, stmt: &Stmt) {
            if let StmtKind::Fn(decl) = &stmt.kind {
                self.0.insert(decl.name.text.clone());
            }
        }
    }
    file.walk(&mut Declared(&mut rule.declared), |_, _| {});
    let mut blocks = Blocks { taste: &mut rule };
    file.walk(&mut blocks, |b, base| b.taste.base = base);
    rule.findings.extend(repeated_literals(file));
    rule.findings
}

/// Hands every nested statement list to [`Taste::block`].
struct Blocks<'a, 'b> {
    taste: &'b mut Taste<'a>,
}

impl Visitor for Blocks<'_, '_> {
    fn stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Fn(FnDecl {
                body: FnBody::Block(body),
                ..
            })
            | StmtKind::While { body, .. }
            | StmtKind::For { body, .. }
            | StmtKind::Unchecked(body) => self.taste.block(&body.stmts),
            StmtKind::Let {
                otherwise: Some(body),
                ..
            } => self.taste.block(&body.stmts),
            _ => {}
        }
    }
    fn expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Block(body) => self.taste.block(&body.stmts),
            ExprKind::If {
                branches,
                otherwise,
            } => {
                for (_, body) in branches {
                    self.taste.block(&body.stmts);
                }
                if let Some(body) = otherwise {
                    self.taste.block(&body.stmts);
                }
            }
            _ => {}
        }
    }
}

struct Taste<'a> {
    src: &'a str,
    base: u32,
    /// The offset of the statement after the one being looked at: each
    /// top-level statement has its own.
    next_base: u32,
    /// Top-level fns declared to return a value.
    valued_fns: HashSet<String>,
    /// Every fn and method the file declares: a rewrite to `count(..)`
    /// would call the file's own `count`.
    declared: HashSet<String>,
    /// Every identifier-like word in the file, for picking fresh names.
    idents: HashSet<&'a str>,
    findings: Vec<Finding>,
}

impl<'a> Taste<'a> {
    fn text(&self, span: Span) -> &'a str {
        span.shift(self.base).text(self.src)
    }

    /// The text of a span of the next statement.
    fn ntext(&self, span: Span) -> &'a str {
        span.shift(self.next_base).text(self.src)
    }

    /// From the start of `a` to the end of the next statement `b`.
    fn pair_span(&self, a: Span, b: Span) -> Span {
        Span {
            start: a.start,
            end: b.end + self.next_base - self.base,
        }
    }

    /// The edits that make `decl` start at `init` and drop the loop after it.
    fn fold(&self, init: &Expr, next: &Stmt, call: &str, new_init: String) -> Option<Vec<(Span, String)>> {
        // A `?` would leave the comprehension, not the fn.
        if self.declared.contains(call) || has_try(next) {
            return None;
        }
        let src = self.src.as_bytes();
        let delta = self.next_base - self.base;
        let mut end = next.span.end + delta;
        while end > next.span.start + delta && src[(end + self.base) as usize - 1].is_ascii_whitespace() {
            end -= 1;
        }
        Some(vec![
            (init.span, new_init),
            (Span { start: init.span.end, end }, String::new()),
        ])
    }

    fn push(&mut self, lint: &'static crate::Lint, diag: Diagnostic) {
        self.findings.push(Finding {
            lint,
            diag: diag.shift(self.base),
        });
    }

    fn block(&mut self, stmts: &[Stmt]) {
        let base = self.base;
        let all: Vec<(u32, &Stmt)> = stmts.iter().map(|s| (base, s)).collect();
        self.sequence(&all);
        self.base = base;
    }

    fn sequence(&mut self, stmts: &[(u32, &Stmt)]) {
        for (i, (base, stmt)) in stmts.iter().enumerate() {
            self.base = *base;
            self.next_base = *base;
            self.discarded(stmt);
            self.index_only(stmt);
            if let Some((next_base, next)) = stmts.get(i + 1) {
                self.next_base = *next_base;
                self.symmetric(stmt, next);
                self.search(stmt, next);
                self.reduction(stmt, next);
            }
        }
    }

    /// `let _ = f(..)` where `f` is a fn of this file that returns a value.
    fn discarded(&mut self, stmt: &Stmt) {
        let StmtKind::Let {
            pat,
            init: Some(init),
            ty: None,
            ..
        } = &stmt.kind
        else {
            return;
        };
        let ExprKind::Call { callee, .. } = &walk::unparen(init).kind else {
            return;
        };
        let ExprKind::Name(name) = &callee.kind else {
            return;
        };
        if !matches!(pat.kind, PatternKind::Wildcard) || !self.valued_fns.contains(name) {
            return;
        }
        let diag = Diagnostic::warning(
            DISCARDED_RESULT.code,
            stmt.span,
            format!("`let _ =` throws away what `{name}` returns"),
        )
        .with_help(format!(
            "use the result, or if `{name}` is called for what it does, make it return nothing"
        ));
        self.push(&DISCARDED_RESULT, diag);
    }

    /// `a[u, v] = e` then `a[v, u] = e`; `g[u].push(v)` then `g[v].push(u)`.
    fn symmetric(&mut self, a: &Stmt, b: &Stmt) {
        let mirrored = match (&a.kind, &b.kind) {
            (
                StmtKind::Assign {
                    target: ta,
                    op: oa,
                    value: va,
                },
                StmtKind::Assign {
                    target: tb,
                    op: ob,
                    value: vb,
                },
            ) => {
                oa == ob
                    && self.text(va.span) == self.ntext(vb.span)
                    && match (&ta.kind, &tb.kind) {
                        (
                            ExprKind::Index {
                                base: ba,
                                indices: ia,
                            },
                            ExprKind::Index {
                                base: bb,
                                indices: ib,
                            },
                        ) => {
                            ia.len() == 2
                                && ib.len() == 2
                                && self.text(ba.span) == self.ntext(bb.span)
                                && self.text(ia[0].span) != self.text(ia[1].span)
                                && self.text(ia[0].span) == self.ntext(ib[1].span)
                                && self.text(ia[1].span) == self.ntext(ib[0].span)
                        }
                        _ => false,
                    }
            }
            (StmtKind::Expr(ea), StmtKind::Expr(eb)) => match (self.push_call(ea, self.base), self.push_call(eb, self.next_base)) {
                (Some((la, ua, va, ra)), Some((lb, ub, vb, rb))) => {
                    la == lb && ua == vb && va == ub && ua != va && ra == rb
                }
                _ => false,
            },
            _ => false,
        };
        if !mirrored {
            return;
        }
        let diag = Diagnostic::warning(
            SYMMETRIC_WRITE.code,
            self.pair_span(a.span, b.span),
            "one fact written twice, once for each order",
        )
        .with_help(
            "keep it in one place: build the graph with `undirected(..)`, or loop over both orders",
        );
        self.push(&SYMMETRIC_WRITE, diag);
    }

    /// `g[u].push(v)` or `g[u].push((v, w..))` as (list, u, v, rest).
    fn push_call(&self, e: &Expr, base: u32) -> Option<(&'a str, &'a str, &'a str, Vec<&'a str>)> {
        let text = |span: Span| span.shift(base).text(self.src);
        let ExprKind::Call { callee, args } = &e.kind else {
            return None;
        };
        let ExprKind::Field { base, name } = &callee.kind else {
            return None;
        };
        let ExprKind::Index { base: list, indices } = &base.kind else {
            return None;
        };
        if name.text != "push" || indices.len() != 1 || args.len() != 1 {
            return None;
        }
        let (v, rest) = match &walk::unparen(&args[0].value).kind {
            ExprKind::Tuple(parts) if !parts.is_empty() => (
                text(parts[0].span),
                parts[1..].iter().map(|p| text(p.span)).collect(),
            ),
            _ => (text(args[0].value.span), Vec::new()),
        };
        Some((text(list.span), text(indices[0].span), v, rest))
    }

    /// `var x = none` then a `for` that sets `x` and breaks: `first(..)`;
    /// with `false`/`true` and `x = true`/`false`: `any(..)`/`all(..)`.
    fn search(&mut self, decl: &Stmt, next: &Stmt) {
        let Some((name, init)) = var_decl(decl) else {
            return;
        };
        let StmtKind::For {
            label: None,
            pat,
            iter,
            body,
        } = &next.kind
        else {
            return;
        };
        let init_text = self.text(init.span);
        if !matches!(init_text, "none" | "false" | "true") || !breaks_after_setting(&body.stmts, name) {
            return;
        }
        let (call, fix) = match self.exact_search(name, init_text, pat, iter, body) {
            Some((call, text)) => (call, Some(text)),
            None => (
                match init_text {
                    "none" => "first",
                    "false" => "any",
                    _ => "all",
                },
                None,
            ),
        };
        let mut diag = Diagnostic::warning(
            SEARCH_LOOP.code,
            self.pair_span(decl.span, next.span),
            format!("a search loop that sets `{name}` and breaks"),
        )
        .with_help(format!(
            "say the search: `{call}(.. for .. in ..)`, or `find`/`count` for their questions"
        ));
        if let Some(edits) = fix.and_then(|text| self.fold(init, next, call, text)) {
            diag.fixes.push(Fix {
                title: format!("use `{call}`"),
                edits,
            });
        }
        self.push(&SEARCH_LOOP, diag);
    }

    /// The rewrite of the loop that is only `if c: x = e; break`.
    fn exact_search(
        &self,
        name: &str,
        init: &str,
        pat: &Pattern,
        iter: &Expr,
        body: &Block,
    ) -> Option<(&'static str, String)> {
        let [Stmt {
            kind: StmtKind::Expr(e),
            ..
        }] = &body.stmts[..]
        else {
            return None;
        };
        let ExprKind::If {
            branches,
            otherwise: None,
        } = &e.kind
        else {
            return None;
        };
        let [(Cond::Expr(cond), then)] = &branches[..] else {
            return None;
        };
        let [Stmt {
            kind:
                StmtKind::Assign {
                    target,
                    op: AssignOp::Set,
                    value,
                },
            ..
        }, Stmt {
            kind: StmtKind::Break(None),
            ..
        }] = &then.stmts[..]
        else {
            return None;
        };
        if !matches!(&target.kind, ExprKind::Name(n) if n == name) {
            return None;
        }
        let (p, it, c, v) = (
            self.ntext(pat.span),
            iter_text(iter, self.ntext(iter.span)),
            self.ntext(cond.span),
            self.ntext(value.span),
        );
        if mentions(c, name) || mentions(v, name) {
            return None;
        }
        Some(match (init, v) {
            ("none", _) => ("first", format!("first({v} for {p} in {it} if {c})")),
            ("false", "true") => ("any", format!("any({c} for {p} in {it})")),
            ("true", "false") => ("all", format!("all(not ({c}) for {p} in {it})")),
            _ => return None,
        })
    }

    /// `var s = 0` then `for x in xs: s += e`: one `sum`, `count`, `max`
    /// or `min`.
    fn reduction(&mut self, decl: &Stmt, next: &Stmt) {
        let Some((name, init)) = var_decl(decl) else {
            return;
        };
        let StmtKind::For {
            label: None,
            pat,
            iter,
            body,
        } = &next.kind
        else {
            return;
        };
        let [only] = &body.stmts[..] else {
            return;
        };
        let (cond, op, value) = match &only.kind {
            StmtKind::Assign { target, op, value }
                if matches!(&target.kind, ExprKind::Name(n) if n == name) =>
            {
                (None, op, value)
            }
            StmtKind::Expr(Expr {
                kind:
                    ExprKind::If {
                        branches,
                        otherwise: None,
                    },
                ..
            }) => match &branches[..] {
                [(Cond::Expr(cond), then)] => match &then.stmts[..] {
                    [Stmt {
                        kind: StmtKind::Assign { target, op, value },
                        ..
                    }] if matches!(&target.kind, ExprKind::Name(n) if n == name) => {
                        (Some(cond), op, value)
                    }
                    _ => return,
                },
                _ => return,
            },
            _ => return,
        };
        self.reduce(name, init, pat, iter, cond, op, value, decl, next);
    }

    #[allow(clippy::too_many_arguments)]
    fn reduce(
        &mut self,
        name: &str,
        init: &Expr,
        pat: &Pattern,
        iter: &Expr,
        cond: Option<&Expr>,
        op: &AssignOp,
        value: &Expr,
        decl: &Stmt,
        next: &Stmt,
    ) {
        let (p, it, v, i) = (
            self.ntext(pat.span),
            iter_text(iter, self.ntext(iter.span)),
            self.ntext(value.span),
            self.text(init.span),
        );
        let c = cond.map(|c| self.ntext(c.span));
        if mentions(v, name) || c.is_some_and(|c| mentions(c, name)) || mentions(&it, name) {
            return;
        }
        let filter = c.map(|c| format!(" if {c}")).unwrap_or_default();
        let (call, text) = match op {
            AssignOp::Binary(BinOp::Add) if v == "1" && c.is_some() && i == "0" => (
                "count",
                Some(format!("count({} for {p} in {it})", c.unwrap_or_default())),
            ),
            AssignOp::Binary(BinOp::Add) if !numeric(i) => return,
            AssignOp::Binary(BinOp::Add) => (
                "sum",
                Some(if i == "0" {
                    format!("sum({v} for {p} in {it}{filter})")
                } else {
                    format!("{i} + sum({v} for {p} in {it}{filter})")
                }),
            ),
            AssignOp::Max => ("max", None),
            AssignOp::Min => ("min", None),
            _ => return,
        };
        let mut diag = Diagnostic::warning(
            LOOP_REDUCTION.code,
            self.pair_span(decl.span, next.span),
            format!("a loop that only accumulates `{name}` is one `{call}`"),
        )
        .with_help(format!("write `{call}({v} for {p} in {it}{filter})`"));
        if let Some(edits) = text.and_then(|text| self.fold(init, next, call, text)) {
            diag.fixes.push(Fix {
                title: format!("use `{call}`"),
                edits,
            });
        }
        self.push(&LOOP_REDUCTION, diag);
    }

    /// `for i in 0..xs.len` whose body uses `i` only as `xs[i]`, reading it.
    fn index_only(&mut self, stmt: &Stmt) {
        let StmtKind::For {
            pat, iter, body, ..
        } = &stmt.kind
        else {
            return;
        };
        let PatternKind::Name(index) = &pat.kind else {
            return;
        };
        let Some(list) = indexed_list(iter) else {
            return;
        };
        let mut uses = Uses {
            index,
            list,
            reads: Vec::new(),
            names: Vec::new(),
            ok: true,
        };
        for inner in &body.stmts {
            walk::walk_stmt(&mut uses, inner);
        }
        let inside = |span: Span| uses.reads.iter().any(|r| r.start <= span.start && span.end <= r.end);
        if !uses.ok || uses.reads.is_empty() || !uses.names.iter().all(|s| inside(*s)) {
            return;
        }
        let mut diag = Diagnostic::warning(
            INDEX_ONLY_LOOP.code,
            pat.span.to(iter.span),
            format!("`{index}` is only used as `{list}[{index}]`"),
        )
        .with_help(format!("iterate the values: `for x in {list}:`"));
        if let Some(elem) = list
            .strip_suffix('s')
            .filter(|e| e.len() > 1 && !self.idents.contains(e))
        {
            let mut edits = vec![(pat.span, elem.to_string()), (iter.span, list.to_string())];
            edits.extend(uses.reads.iter().map(|r| (*r, elem.to_string())));
            diag.help = Some(format!("iterate the values: `for {elem} in {list}:`"));
            diag.fixes.push(Fix {
                title: format!("iterate `{list}`"),
                edits,
            });
        }
        self.push(&INDEX_ONLY_LOOP, diag);
    }
}

/// The name `xs` of `0..xs.len` or `xs.indices`.
fn indexed_list(iter: &Expr) -> Option<&str> {
    fn len_of<'e>(e: &'e Expr, field: &str) -> Option<&'e str> {
        match &e.kind {
            ExprKind::Field { base, name } if name.text == field => match &base.kind {
                ExprKind::Name(list) => Some(list.as_str()),
                _ => None,
            },
            _ => None,
        }
    }
    match &walk::unparen(iter).kind {
        ExprKind::Range {
            lo: Some(lo),
            hi: Some(hi),
            inclusive: false,
            step: None,
        } if matches!(lo.kind, ExprKind::Int(0)) => len_of(hi, "len"),
        _ => len_of(iter, "indices"),
    }
}

/// The uses of an index and its list in a loop body.
struct Uses<'n> {
    index: &'n str,
    list: &'n str,
    /// Each `xs[i]` that is read.
    reads: Vec<Span>,
    /// Each mention of `i` or `xs`.
    names: Vec<Span>,
    ok: bool,
}

impl Visitor for Uses<'_> {
    fn stmt(&mut self, stmt: &Stmt) {
        if let StmtKind::Assign { target, .. } = &stmt.kind {
            if walk::root_name(target) == Some(self.list) {
                self.ok = false;
            }
        }
    }
    fn expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Index { base, indices } => {
                if matches!(&base.kind, ExprKind::Name(n) if n == self.list)
                    && matches!(&indices[..], [Expr { kind: ExprKind::Name(i), .. }] if i == self.index)
                {
                    self.reads.push(expr.span);
                }
            }
            ExprKind::Name(n) if n == self.index || n == self.list => self.names.push(expr.span),
            _ => {}
        }
    }
    fn bind(&mut self, name: &str, _span: Span, _how: Bind) {
        if name == self.index || name == self.list {
            self.ok = false;
        }
    }
}

/// `var x = e` with a plain name.
fn var_decl(stmt: &Stmt) -> Option<(&str, &Expr)> {
    match &stmt.kind {
        StmtKind::Let {
            mutable: true,
            pat:
                Pattern {
                    kind: PatternKind::Name(name),
                    ..
                },
            init: Some(init),
            otherwise: None,
            guard: None,
            ..
        } => Some((name, init)),
        _ => None,
    }
}

/// Some block under `stmts` sets `name` and then breaks.
fn breaks_after_setting(stmts: &[Stmt], name: &str) -> bool {
    let sets = |s: &Stmt| {
        matches!(&s.kind, StmtKind::Assign { target, op: AssignOp::Set, .. }
            if matches!(&target.kind, ExprKind::Name(n) if n == name))
    };
    stmts.windows(2).any(|w| sets(&w[0]) && matches!(w[1].kind, StmtKind::Break(None)))
        || stmts.iter().any(|s| match &s.kind {
            StmtKind::Expr(Expr {
                kind:
                    ExprKind::If {
                        branches,
                        otherwise,
                    },
                ..
            }) => {
                branches.iter().any(|(_, b)| breaks_after_setting(&b.stmts, name))
                    || otherwise.as_ref().is_some_and(|b| breaks_after_setting(&b.stmts, name))
            }
            StmtKind::Unchecked(b) => breaks_after_setting(&b.stmts, name),
            _ => false,
        })
}

/// `stmt` holds a `?`.
fn has_try(stmt: &Stmt) -> bool {
    struct Tries(bool);
    impl Visitor for Tries {
        fn expr(&mut self, expr: &Expr) {
            self.0 |= matches!(expr.kind, ExprKind::Try(_));
        }
    }
    let mut tries = Tries(false);
    walk::walk_stmt(&mut tries, stmt);
    tries.0
}

/// `0..` as `(0..)`, so a following `if` is not read as its end.
fn iter_text<'s>(iter: &Expr, text: &'s str) -> std::borrow::Cow<'s, str> {
    match &iter.kind {
        ExprKind::Range { hi: None, .. } => format!("({text})").into(),
        _ => text.into(),
    }
}

/// An initial value `sum` can start from: a number or a `.zero`.
fn numeric(init: &str) -> bool {
    init.ends_with(".zero") || init.trim_start_matches('-').starts_with(|c: char| c.is_ascii_digit())
}

/// `text` mentions the identifier `name`.
fn mentions(text: &str, name: &str) -> bool {
    words(text).any(|w| w == name)
}

fn words(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| !(c.is_alphanumeric() || c == '_')).filter(|w| !w.is_empty())
}

fn idents(src: &str) -> HashSet<&str> {
    words(src).collect()
}

/// Big literals: a bound or modulus written more than once is two facts
/// that must agree.
fn repeated_literals(file: &File) -> Vec<Finding> {
    struct Ints {
        base: u32,
        seen: HashMap<i128, Vec<Span>>,
    }
    impl Visitor for Ints {
        fn expr(&mut self, expr: &Expr) {
            if let ExprKind::Int(v) = expr.kind {
                if v >= 1000 {
                    self.seen.entry(v).or_default().push(expr.span.shift(self.base));
                }
            }
        }
    }
    let mut ints = Ints {
        base: 0,
        seen: HashMap::new(),
    };
    file.walk(&mut ints, |i, base| i.base = base);
    let lines = cpl_diag::LineIndex::new(file.src);
    let mut out = Vec::new();
    for (value, mut spans) in ints.seen {
        // `-1e9..=1e9` is one fact: count lines, not occurrences.
        spans.sort_by_key(|s| s.start);
        spans.dedup_by_key(|s| lines.line_col(s.start).line);
        let need = if value >= 1_000_000 { 2 } else { 3 };
        if spans.len() < need {
            continue;
        }
        let line = lines.line_col(spans[0].start).line + 1;
        let span = spans[1];
        let text = span.text(file.src);
        let diag = Diagnostic::warning(
            REPEATED_LITERAL.code,
            span,
            format!(
                "`{text}` is written on {} lines; it is first on line {line}",
                spans.len()
            ),
        )
        .with_help(format!(
            "name it once (`const P = {text}`, a domain, or a type alias) and use the name"
        ));
        out.push(Finding {
            lint: &REPEATED_LITERAL,
            diag,
        });
    }
    out
}
