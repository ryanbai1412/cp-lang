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
    File, Finding, COPY_SORT_WRITEBACK, DISCARDED_RESULT, DOMAIN_SPELLING, FREE_FOLD,
    HAND_ROLLED_ANSWER_SEARCH, LOOP_REDUCTION, PUSH_LOOP, SEARCH_LOOP,
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

/// Hands every statement sequence, the top level's too, to [`Taste::sequence`].
struct Blocks<'a, 'b> {
    taste: &'b mut Taste<'a>,
}

impl Visitor for Blocks<'_, '_> {
    fn stmts(&mut self, stmts: &[(u32, &Stmt)]) {
        let base = self.taste.base;
        let all: Vec<(u32, &Stmt)> = stmts.iter().map(|&(b, s)| (base + b, s)).collect();
        self.taste.sequence(&all);
        self.taste.base = base;
    }
    fn expr(&mut self, expr: &Expr) {
        self.taste.free_fold(expr);
        self.taste.domain_spelling(expr);
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

    fn sequence(&mut self, stmts: &[(u32, &Stmt)]) {
        for (i, (base, stmt)) in stmts.iter().enumerate() {
            self.base = *base;
            self.next_base = *base;
            self.discarded(stmt);
            self.answer_search(stmts, i);
            self.copy_mutate_writeback(stmts, i);
            if let Some((next_base, next)) = stmts.get(i + 1) {
                self.next_base = *next_base;
                let Some(acc) = AccLoop::of(stmt, next, Calls::new(self.facts, self.next_base))
                else {
                    continue;
                };
                for form in acc.forms() {
                    match form {
                        Form::Search => self.search(&acc, stmt, next),
                        Form::Fold { cond, op, value } => {
                            self.reduce(&acc, cond, op, value, stmt, next)
                        }
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
        if self.unchecked.iter().any(|&(a, b)| a <= start && start < b) || self.asks {
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

    /// Copy-mutate-writeback on a slice (L0522):
    ///   `let sub = xs[l..r].sorted(..); for i in l..r: xs[i] = sub[i - l]`
    /// or
    ///   `var sub = xs[l..r].to_list(); sub.sort(..); for i in 0..sub.len: xs[l + i] = sub[i]`
    /// rewrites to `xs[l..r].sort(..)` when `sub` is not read afterward.
    fn copy_mutate_writeback(&mut self, stmts: &[(u32, &Stmt)], i: usize) {
        let (first_base, first_stmt) = stmts[i];
        let StmtKind::Let {
            pat,
            init: Some(init),
            otherwise: None,
            ..
        } = &first_stmt.kind
        else {
            return;
        };
        let PatternKind::Name(sub_name) = &pat.kind else {
            return;
        };
        let at = |b: u32, span: Span| {
            self.src[(b + span.start) as usize..(b + span.end) as usize].trim()
        };

        let Some(found) = self.match_copy_mutate_pair(stmts, i, sub_name, first_base, init) else {
            return;
        };
        let Some(&(wb_base, wb_stmt)) = stmts.get(found.wb_idx) else {
            return;
        };
        if !is_slice_writeback_loop(
            self.src,
            wb_base,
            wb_stmt,
            &found.target,
            sub_name,
            &found.range,
            found.lo.as_deref(),
        ) {
            return;
        }
        let replacement = format!(
            "{}[{}].{}({})",
            found.target, found.range, found.method, found.args
        );
        let end = trim_end(self.src, wb_base, wb_stmt.span) + wb_base - first_base;
        let span = Span {
            start: first_stmt.span.start,
            end,
        };
        let mut diag = msgs::lint::COPY_SORT_WRITEBACK
            .warning(span, &[&found.target, &found.range, &replacement])
            .with_help(format!("mutate the slice in place: `{replacement}`"));

        let sub_used_after = stmts[found.wb_idx + 1..]
            .iter()
            .any(|&(b, s)| mentions(at(b, s.span), sub_name));
        if !sub_used_after {
            diag.fixes.push(Fix {
                title: format!("use `{replacement}`"),
                edits: vec![(span, replacement)],
            });
        }
        self.base = first_base;
        self.push(&COPY_SORT_WRITEBACK, diag);
    }

    fn match_copy_mutate_pair(
        &self,
        stmts: &[(u32, &Stmt)],
        i: usize,
        sub_name: &str,
        first_base: u32,
        init: &Expr,
    ) -> Option<CopyMutate> {
        let at = |b: u32, span: Span| {
            self.src[(b + span.start) as usize..(b + span.end) as usize].trim()
        };

        // Form 1 (2 statements): `let/var sub = xs[l..r].sorted(..)` / `.reversed()` / `.rotated_left(..)` / `.rotated_right(..)`
        if let ExprKind::Call { callee, args, .. } = &init.kind {
            if let ExprKind::Field { base, .. } = &callee.kind {
                if let Some((target, range, lo)) =
                    extract_contiguous_slice(self.src, first_base, base)
                {
                    let calls = Calls::new(self.facts, first_base);
                    let in_place = calls.name(init).and_then(in_place_twin);
                    if let Some(method) = in_place {
                        let args =
                            call_args_text(self.src, first_base, init.span, callee.span, args);
                        return Some(CopyMutate {
                            target,
                            range,
                            lo,
                            method,
                            args,
                            wb_idx: i + 1,
                        });
                    }
                    // Form 2a (3 statements): `var sub = xs[l..r].to_list()` + `sub.sort(..)` + writeback
                    if calls.is(init, std_fn::TO_LIST) && args.is_empty() {
                        if let Some((method, args)) = match_in_place_mut_stmt(
                            self.src,
                            stmts.get(i + 1)?,
                            sub_name,
                            self.facts,
                        ) {
                            return Some(CopyMutate {
                                target,
                                range,
                                lo,
                                method: method.to_string(),
                                args,
                                wb_idx: i + 2,
                            });
                        }
                    }
                }
            }
        }

        // Form 2b (3 statements): `var sub = [xs[k] for k in l..r]` + `sub.sort(..)` + writeback
        if let ExprKind::Comprehension { body, clauses } = &init.kind {
            if clauses.len() == 1 {
                let Clause { pat, iter, conds } = &clauses[0];
                if conds.is_empty() {
                    if let PatternKind::Name(var) = &pat.kind {
                        if let ExprKind::Range {
                            lo,
                            hi: _,
                            inclusive: _,
                            step: None,
                        } = &iter.kind
                        {
                            if let ExprKind::Index { base, indices } = &body.kind {
                                if indices.len() == 1
                                    && matches!(&indices[0].kind, ExprKind::Name(n) if n == var)
                                {
                                    let target = at(first_base, base.span).to_string();
                                    let range = at(first_base, iter.span).to_string();
                                    let lo =
                                        lo.as_ref().map(|e| at(first_base, e.span).to_string());
                                    if let Some((method, args)) = match_in_place_mut_stmt(
                                        self.src,
                                        stmts.get(i + 1)?,
                                        sub_name,
                                        self.facts,
                                    ) {
                                        return Some(CopyMutate {
                                            target,
                                            range,
                                            lo,
                                            method: method.to_string(),
                                            args,
                                            wb_idx: i + 2,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Form 3 (4 statements): `var sub: List<T> = []` + `for k in l..r: sub.push(xs[k])` +
        // `sub.sort(..)` + writeback
        if matches!(&init.kind, ExprKind::List(items) if items.is_empty()) {
            let &(loop_base, loop_stmt) = stmts.get(i + 1)?;
            let StmtKind::For {
                pat, iter, body, ..
            } = &loop_stmt.kind
            else {
                return None;
            };
            let PatternKind::Name(var) = &pat.kind else {
                return None;
            };
            let ExprKind::Range { lo, step: None, .. } = &iter.kind else {
                return None;
            };
            let [push] = body.stmts.as_slice() else {
                return None;
            };
            let StmtKind::Expr(call) = &push.kind else {
                return None;
            };
            let (base, ix) = plain_index(pushed_onto(
                call,
                sub_name,
                Calls::new(self.facts, loop_base),
            )?)?;
            if shapes::name(ix) != Some(var.as_str()) || base == sub_name {
                return None;
            }
            let (method, args) =
                match_in_place_mut_stmt(self.src, stmts.get(i + 2)?, sub_name, self.facts)?;
            return Some(CopyMutate {
                target: base.to_string(),
                range: at(loop_base, iter.span).to_string(),
                lo: lo.as_ref().map(|e| at(loop_base, e.span).to_string()),
                method: method.to_string(),
                args,
                wb_idx: i + 3,
            });
        }

        None
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

    /// `sum(xs)`, `count(xs, p)`, `sorted(xs, by: k)`, `sort(mut xs)`:
    /// a fold or sort of one sequence, which reads as its method.
    fn free_fold(&mut self, expr: &Expr) {
        let ExprKind::Call { callee, args } = &expr.kind else {
            return;
        };
        if !matches!(callee.kind, ExprKind::Name(_)) {
            return;
        }
        let Some(name) = Calls::new(self.facts, self.base).name(expr) else {
            return;
        };
        let Some(seq) = args.first() else {
            return;
        };
        if self.declared.contains(name)
            || seq.name.is_some()
            || matches!(seq.value.kind, ExprKind::Comprehension { .. })
        {
            return;
        }
        let positional = args.iter().all(|a| a.name.is_none() && !a.mutable);
        let fits = match name {
            std_fn::SUM | std_fn::PROD => args.len() == 1 && !seq.mutable,
            std_fn::MIN | std_fn::MAX => {
                !seq.mutable
                    && (args.len() == 1
                        || (args.len() == 2
                            && args[1].name.as_ref().map(|n| n.text.as_str()) == Some("by")))
            }
            std_fn::COUNT | std_fn::ALL | std_fn::ANY | std_fn::FIRST => {
                args.len() == 2 && positional && is_test(&args[1].value)
            }
            std_fn::SORTED => !seq.mutable && args[1..].iter().all(|a| a.name.is_some()),
            std_fn::SORT => seq.mutable && args[1..].iter().all(|a| a.name.is_some()),
            _ => false,
        };
        // `sorted(V, by: k)` walks a domain, not a value; `min([a, b, c])`
        // is a list literal rather than a named sequence.
        let skip = match &seq.value.kind {
            ExprKind::Name(n) => n.starts_with(char::is_uppercase),
            ExprKind::List(_) => matches!(name, std_fn::MIN | std_fn::MAX) && args.len() == 1,
            _ => false,
        };
        if !fits || skip {
            return;
        }
        let recv = self.text(seq.value.span);
        let recv = if postfix(&seq.value) {
            recv.to_string()
        } else {
            format!("({recv})")
        };
        let rest = match (args.get(1), args.last()) {
            (Some(second), Some(last)) => {
                let start = second
                    .name
                    .as_ref()
                    .map_or(second.value.span.start, |n| n.span.start);
                self.text(Span {
                    start,
                    end: last.value.span.end,
                })
            }
            _ => "",
        };
        // The method form of `first(xs, p)` is `xs.find(p)` (decisions.md §182).
        let method = match name {
            std_fn::FIRST => std_fn::FIND,
            other => other,
        };
        let call = format!("{recv}.{method}({rest})");
        let mut diag = msgs::lint::FREE_FOLD.warning(expr.span, &[&name])
        .with_help(format!("write `{call}`; the method follows the sequence, so a chain like `xs.filter(..).sum()` reads in order"));
        diag.fixes.push(Fix {
            title: format!("use `.{method}`"),
            edits: vec![(expr.span, call)],
        });
        self.push(&FREE_FOLD, diag);
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
                    Some(cpl_facade::today::Ty::Domain(_) | cpl_facade::today::Ty::TypeName(_))
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

    fn reduce(
        &mut self,
        acc: &AccLoop,
        cond: Option<&Expr>,
        op: &AssignOp,
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
        // `x ?? return None` must be a whole right side (E0501), so a
        // generator can't hold it.
        if leaves(value) || cond.is_some_and(leaves) {
            return;
        }
        let (p, it, v, i) = (
            self.ntext(pat.span),
            iter_text(iter, self.ntext(iter.span)),
            self.ntext(value.span),
            self.text(init.span),
        );
        if let (AssignOp::Set, ExprKind::Call { callee, args }) = (op, &value.kind) {
            let (f, x) = (self.ntext(callee.span), self.ntext(args[1].value.span));
            // A fn parameter `f` is the fold being written, not one to call.
            let param = [format!("({f}:"), format!(", {f}:")];
            if cond.is_some()
                || mentions(x, name)
                || mentions(&it, name)
                || param.iter().any(|p| self.src.contains(p.as_str()))
            {
                return;
            }
            let seq = sequence(&it, p, x, None);
            // `fold(f, ..)` hands `f` each element as is; when `f` takes
            // another type (`?Pos` for a `Pos`), the call widened it, so it
            // stays in a lambda.
            // An untyped lambda can't tell the start value's type, so it
            // writes both; it stays quiet on an optional parameter when the
            // element type is not written.
            let elem = it.trim();
            let folded = match (param_type(self.src, f, 0), param_type(self.src, f, 1)) {
                (Some(acc), Some(t)) if t != elem && is_type_name(elem) => {
                    format!("({name}: {acc}, {p}: {elem}) => {v}")
                }
                (_, Some(t)) if t.starts_with('?') && !is_type_name(elem) => return,
                _ => f.to_string(),
            };
            let diag = msgs::lint::LOOP_FOLD.warning(self.pair_span(decl.span, next.span), &[&name, &f])
            .with_help(format!(
                "write `{seq}.fold_from({i}, {folded})`; one call says what the loop computes, with the start value first"
            ));
            self.push(&LOOP_REDUCTION, diag);
            return;
        }
        let c = cond.map(|c| self.ntext(c.span));
        if mentions(v, name) || c.is_some_and(|c| mentions(c, name)) || mentions(&it, name) {
            return;
        }
        let filter = c.map(|c| format!(" if {c}")).unwrap_or_default();
        // The generator a `.count(..)` falls back to where the checker says
        // the loop walks something that is not a `Seq` (a `Map`).
        let mut generator = None;
        let (call, text) = match op {
            // Decision 53: one sequence's count is the method; a pattern
            // that binds more than one name keeps the generator.
            AssignOp::Binary(BinOp::Add) if v == "1" && c.is_some() && i == "0" => {
                let c = c.unwrap_or_default();
                let one_name = !p.is_empty()
                    && p != "_"
                    && p.chars().all(|ch| ch.is_alphanumeric() || ch == '_');
                // A lambda can't write what it captures; a generator can.
                let writes = cond.is_some_and(writes_place);
                (
                    std_fn::COUNT,
                    Some(if one_name && !writes {
                        generator = Some(format!("count({c} for {p} in {it})"));
                        format!("{}.count({p} => {c})", receiver(&it))
                    } else {
                        format!("count({c} for {p} in {it})")
                    }),
                )
            }
            AssignOp::Binary(BinOp::Add) if !numeric(i) => return,
            AssignOp::Binary(BinOp::Add) if i == "0" && c.is_none() && v == p => {
                (std_fn::SUM, Some(format!("{}.sum()", receiver(&it))))
            }
            AssignOp::Binary(BinOp::Add) => (
                std_fn::SUM,
                Some(if i == "0" {
                    format!("sum({v} for {p} in {it}{filter})")
                } else {
                    format!("{i} + sum({v} for {p} in {it}{filter})")
                }),
            ),
            AssignOp::Max => (std_fn::MAX, None),
            AssignOp::Min => (std_fn::MIN, None),
            _ => return,
        };
        // `let sum = sum(..)` names itself.
        if name == call {
            return;
        }
        let at = decl.span.shift(self.base).start;
        if self
            .fn_spans
            .iter()
            .any(|(f, start, end)| f == call && *start <= at && at < *end)
        {
            return;
        }
        let mut diag = msgs::lint::LOOP_ACCUMULATE.warning(self.pair_span(decl.span, next.span), &[&name, &call])
        .with_help(match &text {
            Some(text) => format!(
                "write `{text}`; one call says what the loop computes, with no starting value to get wrong"
            ),
            None if sentinel(i) => {
                let seq = sequence(&it, p, v, c);
                format!("write `{seq}.{call}() ?? {i}`")
            }
            None => {
                let seq = sequence(&it, p, v, c);
                format!("write `{seq}.{call}()`, or `{call}({i}, {seq}.{call}() ?? {i})` if `{i}` can win or the loop can run zero times")
            }
        });
        if let Some(generator) = generator {
            let delta = self.next_base - self.base;
            let iter = Span {
                start: iter.span.start + delta,
                end: iter.span.end + delta,
            };
            diag = diag.with_label(iter, generator);
        }
        if let Some(edits) = text.and_then(|text| self.fold(init, next, call, text)) {
            diag.fixes.push(Fix {
                title: format!("use `{call}`"),
                edits,
            });
        }
        self.push(&LOOP_REDUCTION, diag);
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

/// The written type of parameter `n` of the file's `fn f`, if it declares one.
fn param_type<'s>(src: &'s str, f: &str, n: usize) -> Option<&'s str> {
    let at = src.find(&format!("fn {f}("))? + f.len() + 4;
    let (mut depth, mut start, mut params) = (0usize, at, Vec::new());
    for (i, c) in src[at..].char_indices() {
        match c {
            '(' | '[' | '<' => depth += 1,
            ')' if depth == 0 => {
                params.push(&src[start..at + i]);
                break;
            }
            ')' | ']' | '>' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                params.push(&src[start..at + i]);
                start = at + i + 1;
            }
            _ => {}
        }
    }
    let ty = params.get(n)?.split_once(':')?.1;
    Some(ty.split('=').next().unwrap_or(ty).trim())
}

fn is_type_name(s: &str) -> bool {
    s.starts_with(|c: char| c.is_ascii_uppercase())
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Whether `e` holds a `return`, `break` or `continue` (`x ?? break`).
fn leaves(e: &Expr) -> bool {
    struct Leaves(bool);
    impl cpl_syntax::visit::Visitor for Leaves {
        fn stmt(&mut self, s: &Stmt) {
            self.0 |= matches!(
                s.kind,
                StmtKind::Return(_) | StmtKind::Break(_) | StmtKind::Continue(_)
            );
        }
    }
    let mut v = Leaves(false);
    cpl_syntax::visit::walk_expr(e, &mut v);
    v.0
}

/// The in-place twin std declares for a value form: `sorted` is `sort`,
/// `rotated_left` is `rotate_left`, read off std's `self` fns.
fn in_place_twin(name: &str) -> Option<String> {
    let std = cpl_facade::Std;
    std.has_method(name).then_some(())?;
    let (word, rest) = name
        .split_once('_')
        .map_or((name, None), |(w, r)| (w, Some(r)));
    [word.strip_suffix('d'), word.strip_suffix("ed")]
        .into_iter()
        .flatten()
        .map(|w| rest.map_or_else(|| w.to_string(), |r| format!("{w}_{r}")))
        .find(|stem| std.has_method(stem))
}

/// Whether `file` is interactive: it calls std's `ask` or `flush`, by the checker's callees.
fn asks(file: &File) -> bool {
    struct Asks<'a> {
        facts: Option<&'a crate::Facts>,
        base: u32,
        found: bool,
    }
    impl crate::walk::Visitor for Asks<'_> {
        fn expr(&mut self, e: &Expr) {
            if matches!(e.kind, ExprKind::Call { .. }) {
                let calls = Calls::new(self.facts, self.base);
                self.found |= calls.is(e, std_fn::ASK) || calls.is(e, std_fn::FLUSH);
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
