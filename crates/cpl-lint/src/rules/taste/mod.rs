//! Taste lints: code that works but reads like translated C++
//! (`plan/PORTING.md`, "Taste checklist"). Each names the construct the
//! language has for the idea, and rewrites to it where the rewrite cannot
//! change what the program does.

use super::shapes;
use crate::facts::Facts;
use crate::std_fn;
use crate::walk::{self, Visitor};
use crate::Calls;
use crate::{
    File, Finding, DISCARDED_RESULT, DOMAIN_SPELLING, HAND_ROLLED_ANSWER_SEARCH, PUSH_LOOP,
    SEARCH_LOOP,
};
use cpl_diag::msgs;
use cpl_diag::{Diagnostic, Fix, Span};
use cpl_syntax::ast::*;
use std::collections::HashSet;

mod forms;
mod search;
mod stdlib_algo;
mod stmts;
mod text;
use forms::{AccLoop, Form};
use search::*;
use stdlib_algo::*;
use stmts::*;
use text::*;

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let mut rule = Taste {
        src: file.src,
        facts: file.facts,
        base: 0,
        next_base: 0,
        valued_fns: HashSet::new(),
        declared: HashSet::new(),
        idents: idents(file.src),
        asks: asks(file),
        unchecked: unchecked_spans(file),
        findings: Vec::new(),
        fn_spans: Vec::new(),
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
    struct FnSpans<'s> {
        out: &'s mut Vec<(String, u32, u32)>,
        base: u32,
    }
    impl Visitor for FnSpans<'_> {
        fn stmt(&mut self, stmt: &Stmt) {
            if let StmtKind::Fn(decl) = &stmt.kind {
                let at = stmt.span.shift(self.base);
                self.out.push((decl.name.text.clone(), at.start, at.end));
            }
        }
    }
    let mut spans = FnSpans {
        out: &mut rule.fn_spans,
        base: 0,
    };
    file.walk(&mut spans, |v, base| v.base = base);
    let mut blocks = Blocks { taste: &mut rule };
    file.walk(&mut blocks, |b, base| b.taste.base = base);
    rule.findings
}

fn is_int_lit(e: &Expr, want: i128) -> bool {
    matches!(walk::unparen(e).kind, ExprKind::Int(v) if v == want)
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
        self.taste.domain_spelling(expr);
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
    facts: Option<&'a Facts>,
    /// The `unchecked:` blocks, as absolute offsets: hand-tuned on purpose.
    unchecked: Vec<(u32, u32)>,
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
    /// Whether the file calls std's `ask`: an interactive program.
    asks: bool,
    findings: Vec<Finding>,
    /// Every fn's name and absolute span: a reduction's own body is the
    /// loop it names.
    fn_spans: Vec<(String, u32, u32)>,
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
    fn fold(
        &self,
        init: &Expr,
        next: &Stmt,
        call: &str,
        new_init: String,
    ) -> Option<Vec<(Span, String)>> {
        // A `?` would leave the comprehension, not the fn.
        if self.declared.contains(call) || has_try(next) {
            return None;
        }
        let src = self.src.as_bytes();
        let delta = self.next_base - self.base;
        let mut end = next.span.end + delta;
        while end > next.span.start + delta
            && src[(end + self.base) as usize - 1].is_ascii_whitespace()
        {
            end -= 1;
        }
        Some(vec![
            (init.span, new_init),
            (
                Span {
                    start: init.span.end,
                    end,
                },
                String::new(),
            ),
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
            self.answer_search(stmts, i);
            if let Some((next_base, next)) = stmts.get(i + 1) {
                self.next_base = *next_base;
                let Some(acc) = AccLoop::of(stmt, next, Calls::new(self.facts, self.next_base))
                else {
                    continue;
                };
                for form in acc.forms() {
                    match form {
                        Form::Search => self.search(&acc, stmt, next),
                        Form::Build { cond, value } => self.collect(&acc, cond, value, stmt, next),
                    }
                }
            }
        }
    }

    /// `while lo < hi` halving on a predicate of `mid` (L0520). The fix
    /// folds `var (lo, hi) = (a, b)` and the loop into one `let lo = ..`.
    fn answer_search(&mut self, stmts: &[(u32, &Stmt)], i: usize) {
        let (base, stmt) = stmts[i];
        let Some(found) = halving_loop(stmt) else {
            return;
        };
        let start = base + stmt.span.start;
        // An interactive search's queries are counted: `first_true` asks at
        // the ends too.
        if self.unchecked.iter().any(|&(a, b)| a <= start && start < b)
            || self.asks
            || self.idents.contains("flush")
        {
            return;
        }
        let at = |b: u32, span: Span| &self.src[(b + span.start) as usize..(b + span.end) as usize];
        let pred_text = at(base, found.pred.span);
        if mentions(pred_text, found.lo) || mentions(pred_text, found.hi) || has_try(stmt) {
            return;
        }
        let call = if found.up {
            std_fn::LAST_TRUE
        } else {
            std_fn::FIRST_TRUE
        };
        let mut diag = msgs::lint::ANSWER_SEARCH
            .warning(stmt.span, &[&call])
            .with_help(format!(
            "`let {lo} = (..).{call}({mid} => ..)`: the predicate must be monotone along the range",
            lo = found.lo,
            mid = found.mid
        ));
        let mut call_name = call.to_string();
        if let Some((first, a, b, second)) = bounds_decl(stmts, i, found.lo, found.hi) {
            let later: Vec<&str> = stmts[i + 1..].iter().map(|&(b, s)| at(b, s.span)).collect();
            let simple = |e: &Expr, at_base: u32| {
                !at(at_base, e.span).contains('(') || matches!(e.kind, ExprKind::Paren(_))
            };
            let (fb, fs) = stmts[first];
            // `var lo = ..` and `var hi = ..` at top level are two items, each with its own base.
            let sb = stmts[second].0;
            if !later.iter().any(|t| mentions(t, found.hi)) && simple(b, sb) && simple(a, fb) {
                let a_text = at(fb, a.span);
                let b_text = at(sb, b.span);
                let pred = if found.keep {
                    pred_text.to_string()
                } else {
                    negated(found.pred, |e| at(base, e.span))
                        .unwrap_or_else(|| format!("not ({pred_text})"))
                };
                let after_a = match walk::unparen(a).kind {
                    ExprKind::Int(v) => (v + 1).to_string(),
                    _ => format!("{a_text} + 1"),
                };
                let fallback = |e: &Expr, t: &str| match walk::unparen(e).kind {
                    ExprKind::Binary { .. } => format!("({t})"),
                    _ => t.to_string(),
                };
                let (a_or, b_or) = (fallback(a, a_text), fallback(b, b_text));
                let reassigned = later.iter().any(|t| assigns(t, found.lo));
                let keyword = if reassigned { "var" } else { "let" };
                let sorted = (!found.up && is_int_lit(a, 0))
                    .then(|| {
                        sorted_lookup(found.pred, found.mid, found.keep, b_text, |e| {
                            at(base, e.span)
                        })
                    })
                    .flatten();
                let text = if let Some(lookup) = sorted {
                    call_name = lookup.0;
                    format!("{keyword} {} = {} ?? {b_or}", found.lo, lookup.1)
                } else if found.up {
                    format!(
                        "{keyword} {} = ({after_a}..={b_text}).last_true({} => {pred}) ?? {a_or}",
                        found.lo, found.mid
                    )
                } else {
                    format!(
                        "{keyword} {} = ({a_text}..{b_text}).first_true({} => {pred}) ?? {b_or}",
                        found.lo, found.mid
                    )
                };
                // The note after the bounds (`var (lo, hi) = (0, n)  // ..`) stays.
                let decls =
                    &self.src[(fb + fs.span.start) as usize..(base + stmt.span.start) as usize];
                let notes: Vec<&str> = decls
                    .lines()
                    .filter_map(|line| line.find("//").map(|at| &line[at + 2..]))
                    .collect();
                let text = if notes.is_empty() {
                    text
                } else {
                    format!("{text}  //{}", notes.join(";"))
                };
                // Spans relative to the first declaration's offset: a
                // top-level statement has its own.
                let end = trim_end(self.src, base, stmt.span) + base - fb;
                let span = Span {
                    start: fs.span.start,
                    end,
                };
                diag.span = Span {
                    start: stmt.span.start + base - fb,
                    end: stmt.span.end + base - fb,
                };
                diag.fixes.push(Fix {
                    title: format!("use `{call_name}`"),
                    edits: vec![(span, text)],
                });
                self.base = fb;
                self.push(&HAND_ROLLED_ANSWER_SEARCH, diag);
                self.base = base;
                return;
            }
        }
        self.push(&HAND_ROLLED_ANSWER_SEARCH, diag);
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
        let diag = msgs::lint::DISCARDED_RESULT
            .warning(stmt.span, &[&name])
            .with_help(format!(
                "use the result, or if `{name}` is called for what it does, make it return nothing"
            ));
        self.push(&DISCARDED_RESULT, diag);
    }

    /// `var x = None` then a `for` that sets `x` and breaks: `first(..)`;
    /// with `false`/`true` and `x = true`/`false`: `any(..)`/`all(..)`.
    fn search(&mut self, acc: &AccLoop, decl: &Stmt, next: &Stmt) {
        let AccLoop {
            name,
            init,
            pat,
            iter,
            body,
            ..
        } = *acc;
        let init_text = self.text(init.span);
        let int_lit = |t: &str| {
            let t = t.strip_prefix('-').unwrap_or(t);
            !t.is_empty() && t.chars().all(|c| c.is_ascii_digit() || c == '_')
        };
        let sentinel = int_lit(init_text);
        if !matches!(init_text, "None" | "none" | "false" | "true") && !sentinel {
            return;
        }
        let exact = self.exact_search(name, init_text, pat, iter, body);
        // A loop that also assigns other state (`psum += ..`, `tbl[r] = m`)
        // is a walk with an early exit, not a search one call can say.
        if exact.is_none() && !only_sets(&body.stmts, name) {
            return;
        }
        // A sentinel start is a search only in the exact `if c: x = e; break` shape.
        if sentinel && exact.is_none() {
            return;
        }
        let (call, fix) = match exact {
            Some((call, text)) => (call, Some(text)),
            None => (
                match init_text {
                    "None" | "none" => std_fn::FIRST,
                    "false" => std_fn::ANY,
                    _ => std_fn::ALL,
                },
                None,
            ),
        };
        let mut diag = (if breaks_after_setting(&body.stmts, name) { &msgs::lint::SEARCH_LOOP_BREAKS } else { &msgs::lint::SEARCH_LOOP_FIRST }).warning(self.pair_span(decl.span, next.span), &[&name])
        .with_help(if let Some(text) = &fix {
            let keep = if sentinel {
                format!(", or keep the `?Int` `first` returns instead of the sentinel `{init_text}`")
            } else {
                String::new()
            };
            format!("write `{text}`; one call says the search, with no flag to set or `break` to forget{keep}")
        } else if sentinel {
            format!(
                "say the search: `first(.. for .. in .. if ..) ?? {init_text}`, or keep the `?Int` it returns instead of the sentinel `{init_text}`"
            )
        } else if matches!(init_text, "None" | "none") {
            "say the search: `first(.. for .. in .. if ..)`, or `.count(..)` to count".to_string()
        } else {
            format!("say the search: `xs.{call}(x => ..)`, or `.count(..)` to count")
        });
        // `let first = first(..)` would read itself.
        let fix = fix.filter(|_| name != call);
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
        let (cond, value) =
            forms::write_once(&body.stmts, name).or_else(|| forms::set_then_break(body, name))?;
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
            ("None" | "none", _) => (std_fn::FIRST, format!("first({v} for {p} in {it} if {c})")),
            (lit, _)
                if forms::write_once(&body.stmts, name).is_none()
                    && lit
                        .trim_start_matches('-')
                        .chars()
                        .all(|ch| ch.is_ascii_digit() || ch == '_') =>
            {
                (
                    std_fn::FIRST,
                    format!("first({v} for {p} in {it} if {c}) ?? {lit}"),
                )
            }
            ("false", "true") => (std_fn::ANY, format!("{}.any({p} => {c})", receiver(&it))),
            ("true", "false") => {
                let holds =
                    negated(cond, |e| self.ntext(e.span)).unwrap_or_else(|| format!("not ({c})"));
                (
                    std_fn::ALL,
                    format!("{}.all({p} => {holds})", receiver(&it)),
                )
            }
            _ => return None,
        })
    }

    /// `var xs = [..]` then a loop that only pushes onto `xs`.
    fn collect(
        &mut self,
        acc: &AccLoop,
        cond: Option<&Expr>,
        value: &Expr,
        decl: &Stmt,
        next: &Stmt,
    ) {
        let AccLoop {
            name,
            init,
            pat,
            iter,
            ..
        } = *acc;
        let ExprKind::List(items) = &init.kind else {
            return;
        };
        let (p, it, v, i) = (
            self.ntext(pat.span),
            iter_text(iter, self.ntext(iter.span)),
            self.ntext(value.span),
            self.text(init.span),
        );
        let c = cond.map(|c| self.ntext(c.span));
        if shapes::is_read(value) {
            return;
        }
        if mentions(v, name) || c.is_some_and(|c| mentions(c, name)) || mentions(&it, name) {
            return;
        }
        let unused = p.chars().all(|ch| ch.is_alphanumeric() || ch == '_')
            && !mentions(v, p)
            && !c.is_some_and(|c| mentions(c, p));
        let p = if unused { "_" } else { p };
        let filter = c.map(|c| format!(" if {c}")).unwrap_or_default();
        let reversed = c.is_none().then(|| forms::reversed(p, &it, v)).flatten();
        let comp = reversed
            .clone()
            .unwrap_or_else(|| format!("[{v} for {p} in {it}{filter}]"));
        let text = if items.is_empty() {
            comp.clone()
        } else {
            format!("{i} + {comp}")
        };
        let mut diag = msgs::lint::PUSH_LOOP.warning(self.pair_span(decl.span, next.span), &[&name])
        .with_help(if reversed.is_some() {
            format!("write `{text}`: it is the sequence reversed, with no `- 1 -` index to get wrong")
        } else {
            format!(
                "write `{text}`; the comprehension says what each item is, so no push is missed or doubled"
            )
        });
        if let Some(edits) = self.fold(init, next, "", text) {
            diag.fixes.push(Fix {
                title: "use a comprehension".into(),
                edits,
            });
        }
        self.push(&PUSH_LOOP, diag);
    }

    /// Canonical domain/sequence spellings:
    /// - `D.sorted(...)` -> `D.sort(...)`
    /// - `xs.first(where: p)` -> `xs.first(p)`
    fn domain_spelling(&mut self, expr: &Expr) {
        let ExprKind::Call { callee, args } = &expr.kind else {
            return;
        };
        let ExprKind::Field { base, name: method } = &callee.kind else {
            return;
        };
        let calls = Calls::new(self.facts, self.base);
        if calls.is(expr, std_fn::SORTED) {
            if let ExprKind::Name(n) = &base.kind {
                let domain = (self.facts).and_then(|f| f.type_of(base.span.shift(self.base)));
                if matches!(
                    domain,
                    Some(cpl_check::Ty::Domain(_) | cpl_check::Ty::TypeName(_))
                ) {
                    let mut diag = msgs::lint::DOMAIN_SORTED
                        .warning(method.span, &[&n, &n])
                        .with_help("domains sort with `.sort(by:)`, matching `xs.sort(by:)`");
                    diag.fixes.push(Fix {
                        title: "rename to `sort`".into(),
                        edits: vec![(method.span, std_fn::SORT.into())],
                    });
                    self.push(&DOMAIN_SPELLING, diag);
                    return;
                }
            }
        }
        if calls.unresolved(expr) == Some(std_fn::FIRST) && args.len() == 1 {
            if let Some(arg_name) = &args[0].name {
                if arg_name.text == "where" {
                    let label_span = Span {
                        start: arg_name.span.start,
                        end: args[0].value.span.start,
                    };
                    let mut diag = msgs::lint::FIND_WHERE
                        .warning(arg_name.span, &[])
                        .with_help(
                        "`first` is the first element; `find(p)` is the first one `p` holds for",
                    );
                    diag.fixes.push(Fix {
                        title: "use `.find(p)`".into(),
                        edits: vec![
                            (method.span, std_fn::FIND.into()),
                            (label_span, String::new()),
                        ],
                    });
                    self.push(&DOMAIN_SPELLING, diag);
                }
            }
        }
    }
}

/// `a < b` as `a >= b`, and so on for one comparison; `not x` as `x`.
fn negated<'e>(pred: &'e Expr, text: impl Fn(&'e Expr) -> &'e str) -> Option<String> {
    let pred = walk::unparen(pred);
    if let ExprKind::Unary {
        op: UnOp::Not,
        expr,
    } = &pred.kind
    {
        return Some(text(expr).to_string());
    }
    let ExprKind::Compare { first, rest } = &pred.kind else {
        return None;
    };
    let [(op, rhs)] = rest.as_slice() else {
        return None;
    };
    let flipped = super::shapes::negated_op(*op)?;
    Some(format!("{} {flipped} {}", text(first), text(rhs)))
}

/// Every `unchecked:` block in the file, as absolute offsets.
fn unchecked_spans(file: &File) -> Vec<(u32, u32)> {
    struct Scan {
        base: u32,
        spans: Vec<(u32, u32)>,
    }
    impl Visitor for Scan {
        fn stmt(&mut self, stmt: &Stmt) {
            if let StmtKind::Unchecked(_) = &stmt.kind {
                self.spans
                    .push((self.base + stmt.span.start, self.base + stmt.span.end));
            }
        }
    }
    let mut scan = Scan {
        base: 0,
        spans: Vec::new(),
    };
    file.walk(&mut scan, |s, base| s.base = base);
    scan.spans
}

/// `text` assigns to `name` (`name = `, `name += `, ...).
fn assigns(text: &str, name: &str) -> bool {
    let mut rest = text;
    while let Some(at) = rest.find(name) {
        let before = rest[..at].chars().next_back();
        let after = rest[at + name.len()..].trim_start();
        let word = !before.is_some_and(|c| c.is_alphanumeric() || c == '_')
            && !rest[at + name.len()..].starts_with(|c: char| c.is_alphanumeric() || c == '_');
        if word {
            let op = after.split_whitespace().next().unwrap_or("");
            if op == "=" || (op.ends_with('=') && !matches!(op, "==" | "<=" | ">=" | "!=")) {
                return true;
            }
        }
        rest = &rest[at + name.len()..];
    }
    false
}

/// The end of `span` (relative to `base`) without trailing whitespace.
fn trim_end(src: &str, base: u32, span: Span) -> u32 {
    let bytes = src.as_bytes();
    let mut end = span.end;
    while end > span.start && bytes[(base + end) as usize - 1].is_ascii_whitespace() {
        end -= 1;
    }
    end
}

/// Whether `file` calls std's `ask`, by the checker's callees.
fn asks(file: &File) -> bool {
    struct Asks<'a> {
        facts: Option<&'a crate::Facts>,
        base: u32,
        found: bool,
    }
    impl crate::walk::Visitor for Asks<'_> {
        fn expr(&mut self, e: &Expr) {
            if matches!(e.kind, ExprKind::Call { .. }) {
                self.found |= Calls::new(self.facts, self.base).is(e, std_fn::ASK);
            }
        }
    }
    let mut v = Asks {
        facts: file.facts,
        base: 0,
        found: false,
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.found
}
