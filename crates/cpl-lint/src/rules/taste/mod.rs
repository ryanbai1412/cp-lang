//! Taste lints: code that works but reads like translated C++
//! (`plan/PORTING.md`, "Taste checklist"). Each names the construct the
//! language has for the idea, and rewrites to it where the rewrite cannot
//! change what the program does.

use crate::walk::{self, Bind, Visitor};
use crate::{
    File, Finding, BY_TWIN, COPY_SORT_WRITEBACK, DISCARDED_RESULT, DOMAIN_SPELLING, FREE_FOLD,
    HAND_ROLLED_ANSWER_SEARCH, HAND_ROLLED_BINARY_LIFTING, HAND_ROLLED_DSU, HAND_ROLLED_PREFIX_SUM,
    HAND_ROLLED_RANGE_TREE, HAND_ROLLED_SIEVE, HAND_ROLLED_STRING_ALGO, INDEX_ONLY_LOOP,
    LOOP_REDUCTION, NONE_CHECK_UNWRAP, PREFIX_TWIN, PUSH_LOOP, SEARCH_LOOP, SORTED_RANGE_METHOD,
    SYMMETRIC_WRITE, TREE_ORDER,
};
use cpl_diag::{Diagnostic, Fix, Span};
use cpl_syntax::ast::*;
use std::collections::{HashMap, HashSet};

mod forms;
mod range_tree;
mod search;
mod stdlib_algo;
mod string_algo;
mod text;
use forms::{AccLoop, Form};
use range_tree::*;
use search::*;
use stdlib_algo::*;
use string_algo::*;
use text::*;

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let mut rule = Taste {
        src: file.src,
        base: 0,
        next_base: 0,
        valued_fns: HashSet::new(),
        declared: HashSet::new(),
        sorted_containers: HashMap::new(),
        idents: idents(file.src),
        unchecked: unchecked_spans(file),
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
    struct Declared<'s>(&'s mut HashSet<String>, &'s mut HashMap<String, bool>);
    impl Visitor for Declared<'_> {
        fn stmt(&mut self, stmt: &Stmt) {
            if let StmtKind::Fn(decl) = &stmt.kind {
                self.0.insert(decl.name.text.clone());
            }
            if let StmtKind::Let { pat, ty, init, .. } = &stmt.kind {
                if let PatternKind::Name(vname) = &pat.kind {
                    if let Some(is_int) = sorted_container_binding(ty.as_ref(), init.as_ref()) {
                        self.1.insert(vname.clone(), is_int);
                    }
                }
            }
        }
    }
    file.walk(
        &mut Declared(&mut rule.declared, &mut rule.sorted_containers),
        |_, _| {},
    );
    let mut blocks = Blocks { taste: &mut rule };
    file.walk(&mut blocks, |b, base| b.taste.base = base);
    rule.findings.extend(hand_rolled_range_trees(file));
    rule.findings.extend(hand_rolled_string_algos(file));
    rule.findings.extend(hand_rolled_stdlib_algos(file));
    rule.findings
}

/// Hands every nested statement list to [`Taste::block`].
struct Blocks<'a, 'b> {
    taste: &'b mut Taste<'a>,
}

impl Visitor for Blocks<'_, '_> {
    fn stmt(&mut self, stmt: &Stmt) {
        self.taste.prefix_use_twin(stmt);
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
    fn ty(&mut self, ty: &cpl_syntax::ast::Type) {
        self.taste.prefix_type_twin(ty);
    }
    fn expr(&mut self, expr: &Expr) {
        self.taste.none_check_unwrap(expr);
        self.taste.free_fold(expr);
        self.taste.by_twin(expr);
        self.taste.tree_order(expr);
        self.taste.domain_spelling(expr);
        self.taste.prefix_twin(expr);
        self.taste.sorted_range_method(expr);
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
    /// Variables known to hold `SortedBag<K>` or `SortedSet<K>` (`bool` = integer key).
    sorted_containers: HashMap<String, bool>,
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
            self.index_only(stmt);
            self.answer_search(stmts, i);
            self.prefix_sum(stmts, i);
            self.copy_mutate_writeback(stmts, i);
            if let Some((next_base, next)) = stmts.get(i + 1) {
                self.next_base = *next_base;
                self.symmetric(stmt, next);
                let Some(acc) = AccLoop::of(stmt, next) else {
                    continue;
                };
                for form in acc.forms() {
                    let AccLoop {
                        name,
                        init,
                        pat,
                        iter,
                        ..
                    } = acc;
                    match form {
                        Form::Search => self.search(&acc, stmt, next),
                        Form::Fold { cond, op, value } => {
                            self.reduce(name, init, pat, iter, cond, op, value, stmt, next)
                        }
                        Form::Build { cond, value } => {
                            self.collect(name, init, pat, iter, cond, value, stmt, next)
                        }
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
            || self.idents.contains("ask")
            || self.idents.contains("flush")
        {
            return;
        }
        let at = |b: u32, span: Span| &self.src[(b + span.start) as usize..(b + span.end) as usize];
        let pred_text = at(base, found.pred.span);
        if mentions(pred_text, found.lo) || mentions(pred_text, found.hi) || has_try(stmt) {
            return;
        }
        let call = if found.up { "last_true" } else { "first_true" };
        let mut diag = Diagnostic::warning(
            HAND_ROLLED_ANSWER_SEARCH.code,
            stmt.span,
            format!("a hand-rolled binary search: this is `{call}`"),
        )
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

    /// Hand-rolled 1D/2D prefix-sum loop (L0521). When the preceding statement
    /// declares `var p` as zero-initialized and every later reference in the
    /// same block is a read `p[e]`, `cpl fix` folds the declaration, loop, and
    /// reads into `let p = Prefix(...)` and `p.prefix(e)`.
    fn prefix_sum(&mut self, stmts: &[(u32, &Stmt)], i: usize) {
        let (base, stmt) = stmts[i];
        let Some(found) = prefix_sum_loop(stmt) else {
            self.running_total(base, stmt);
            return;
        };
        let at = |b: u32, span: Span| &self.src[(b + span.start) as usize..(b + span.end) as usize];
        let rhs_text = at(base, found.rhs.span);
        if mentions(rhs_text, found.arr) || has_try(stmt) {
            return;
        }
        let mut diag = Diagnostic::warning(
            HAND_ROLLED_PREFIX_SUM.code,
            stmt.span,
            format!("a hand-rolled prefix sum into `{}`: use `Prefix(...)`", found.arr),
        )
        .with_help(format!(
            "`let {} = Prefix(..)` builds the prefix table in O(n) with O(1) `.fold(range)` and `.prefix(cut)`",
            found.arr
        ));
        if let Some((decl_idx, decl_base, decl_stmt)) =
            prefix_zero_decl(self.src, stmts, i, found.arr, &found.kind)
        {
            if let Some(reads) = collect_pure_index_reads(&stmts[i + 1..], found.arr) {
                let expr = match &found.kind {
                    PrefixLoopKind::Domain1d { var, domain } => {
                        if let Some((src_arr, ix)) = plain_index(found.rhs) {
                            if expr_name(ix) == Some(var) && src_arr != found.arr {
                                format!("Prefix({src_arr})")
                            } else {
                                format!("Prefix([{domain}] {var} => {rhs_text})")
                            }
                        } else {
                            format!("Prefix([{domain}] {var} => {rhs_text})")
                        }
                    }
                    PrefixLoopKind::RangeZero { var, hi } => {
                        let hi_text = at(base, hi.span);
                        if let Some((src_arr, ix)) = plain_index(found.rhs) {
                            if expr_name(ix) == Some(var)
                                && src_arr != found.arr
                                && hi_text == format!("{src_arr}.len")
                            {
                                format!("Prefix({src_arr})")
                            } else {
                                format!("Prefix([{rhs_text} for {var} in 0..{hi_text}])")
                            }
                        } else {
                            format!("Prefix([{rhs_text} for {var} in 0..{hi_text}])")
                        }
                    }
                    PrefixLoopKind::RangeOne | PrefixLoopKind::Grid2d => String::new(),
                };
                if !expr.is_empty() && decl_idx + 1 == i {
                    let end = trim_end(self.src, base, stmt.span) + base - decl_base;
                    let mut edits = vec![(
                        Span {
                            start: decl_stmt.span.start,
                            end,
                        },
                        format!("let {} = {expr}", found.arr),
                    )];
                    for (rb, idx_expr_span, ix_span) in reads {
                        let ix_text = at(rb, ix_span);
                        edits.push((
                            Span {
                                start: idx_expr_span.start + rb - decl_base,
                                end: idx_expr_span.end + rb - decl_base,
                            },
                            format!("{}.prefix({ix_text})", found.arr),
                        ));
                    }
                    diag.span = Span {
                        start: stmt.span.start + base - decl_base,
                        end: stmt.span.end + base - decl_base,
                    };
                    diag.fixes.push(Fix {
                        title: "use `Prefix(...)`".to_string(),
                        edits,
                    });
                    self.base = decl_base;
                    self.push(&HAND_ROLLED_PREFIX_SUM, diag);
                    self.base = base;
                    return;
                }
            }
        }
        self.push(&HAND_ROLLED_PREFIX_SUM, diag);
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
        let mut diag = Diagnostic::warning(
            COPY_SORT_WRITEBACK.code,
            span,
            format!(
                "copies `{}[{}]`, mutates it, and writes it back: use `{replacement}`",
                found.target, found.range
            ),
        )
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
            if let ExprKind::Field { base, name } = &callee.kind {
                if let Some((target, range, lo)) =
                    extract_contiguous_slice(self.src, first_base, base)
                {
                    let in_place = match name.text.as_str() {
                        "sorted" => Some("sort"),
                        "reversed" => Some("reverse"),
                        "rotated_left" => Some("rotate_left"),
                        "rotated_right" => Some("rotate_right"),
                        _ => None,
                    };
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
                    if name.text == "to_list" && args.is_empty() {
                        if let Some((method, args)) =
                            match_in_place_mut_stmt(self.src, stmts.get(i + 1)?, sub_name)
                        {
                            return Some(CopyMutate {
                                target,
                                range,
                                lo,
                                method,
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
                                    ) {
                                        return Some(CopyMutate {
                                            target,
                                            range,
                                            lo,
                                            method,
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
            let (base, ix) = plain_index(pushed_onto(call, sub_name)?)?;
            if expr_name(ix) != Some(var.as_str()) || base == sub_name {
                return None;
            }
            let (method, args) = match_in_place_mut_stmt(self.src, stmts.get(i + 2)?, sub_name)?;
            return Some(CopyMutate {
                target: base.to_string(),
                range: at(loop_base, iter.span).to_string(),
                lo: lo.as_ref().map(|e| at(loop_base, e.span).to_string()),
                method,
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
            (StmtKind::Expr(ea), StmtKind::Expr(eb)) => match (
                self.push_call(ea, self.base),
                self.push_call(eb, self.next_base),
            ) {
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
            "keep it in one place: build the graph with `Graph.undirected(..)`, or loop over both orders",
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
        let ExprKind::Index {
            base: list,
            indices,
        } = &base.kind
        else {
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

    /// `var x = None` then a `for` that sets `x` and breaks: `first(..)`;
    /// with `false`/`true` and `x = true`/`false`: `any(..)`/`all(..)`.
    fn search(&mut self, acc: &AccLoop, decl: &Stmt, next: &Stmt) {
        let AccLoop {
            name,
            init,
            pat,
            iter,
            body,
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
                    "None" | "none" => "first",
                    "false" => "any",
                    _ => "all",
                },
                None,
            ),
        };
        let mut diag = Diagnostic::warning(
            SEARCH_LOOP.code,
            self.pair_span(decl.span, next.span),
            if breaks_after_setting(&body.stmts, name) {
                format!("a search loop that sets `{name}` and breaks")
            } else {
                format!("a search loop: only the first write to `{name}` counts")
            },
        )
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
            ("None" | "none", _) => ("first", format!("first({v} for {p} in {it} if {c})")),
            (lit, _)
                if forms::write_once(&body.stmts, name).is_none()
                    && lit
                        .trim_start_matches('-')
                        .chars()
                        .all(|ch| ch.is_ascii_digit() || ch == '_') =>
            {
                (
                    "first",
                    format!("first({v} for {p} in {it} if {c}) ?? {lit}"),
                )
            }
            ("false", "true") => ("any", format!("{}.any({p} => {c})", receiver(&it))),
            ("true", "false") => {
                let holds =
                    negated(cond, |e| self.ntext(e.span)).unwrap_or_else(|| format!("not ({c})"));
                ("all", format!("{}.all({p} => {holds})", receiver(&it)))
            }
            _ => return None,
        })
    }

    /// `var xs = [..]` then a loop that only pushes onto `xs`.
    #[allow(clippy::too_many_arguments)]
    fn collect(
        &mut self,
        name: &str,
        init: &Expr,
        pat: &Pattern,
        iter: &Expr,
        cond: Option<&Expr>,
        value: &Expr,
        decl: &Stmt,
        next: &Stmt,
    ) {
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
        if super::read_loop::is_read(value) {
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
        let mut diag = Diagnostic::warning(
            PUSH_LOOP.code,
            self.pair_span(decl.span, next.span),
            format!("a loop that only pushes onto `{name}` is a comprehension"),
        )
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

    /// `x != none and .. x! ..`: the test and the unwrap say one thing twice.
    fn none_check_unwrap(&mut self, expr: &Expr) {
        let ExprKind::Binary {
            op: BinOp::And,
            lhs,
            rhs,
            ..
        } = &expr.kind
        else {
            return;
        };
        let ExprKind::Compare { first, rest } = &lhs.kind else {
            return;
        };
        let (
            ExprKind::Name(x),
            [(
                CmpOp::Ne,
                Expr {
                    kind: ExprKind::None,
                    ..
                },
            )],
        ) = (&first.kind, &rest[..])
        else {
            return;
        };
        struct Unwraps<'n>(&'n str, bool);
        impl Visitor for Unwraps<'_> {
            fn expr(&mut self, expr: &Expr) {
                if let ExprKind::Unwrap(inner) = &expr.kind {
                    self.1 |= matches!(&inner.kind, ExprKind::Name(n) if n == self.0);
                }
            }
        }
        let mut unwraps = Unwraps(x, false);
        walk::walk_expr(&mut unwraps, rhs);
        if !unwraps.1 {
            return;
        }
        let diag = Diagnostic::warning(
            NONE_CHECK_UNWRAP.code,
            expr.span,
            format!("`{x} != None` and then `{x}!`: the check is said twice"),
        )
        .with_help(format!(
            "bind it once: `if let v = {x} and ..:`, or compare the option itself (`{x} == Some(..)`, `{x} ?? d`)"
        ));
        self.push(&NONE_CHECK_UNWRAP, diag);
    }

    /// `sum(xs)`, `count(xs, p)`, `sorted(xs, by: k)`, `sort(mut xs)`:
    /// a fold or sort of one sequence, which reads as its method.
    fn free_fold(&mut self, expr: &Expr) {
        let ExprKind::Call { callee, args } = &expr.kind else {
            return;
        };
        // `xs.first` is the first element; the element a test holds for is
        // `xs.find(x => ..)` (decisions.md §182).
        if let (ExprKind::Field { name: method, .. }, [test]) = (&callee.kind, &args[..]) {
            if method.text == "first"
                && test.name.is_none()
                && matches!(test.value.kind, ExprKind::Lambda { .. })
            {
                let mut diag = Diagnostic::warning(
                    FREE_FOLD.code,
                    method.span,
                    "the element a test holds for is `find`; `first` is the first element",
                )
                .with_help("write `.find(..)`");
                diag.fixes.push(Fix {
                    title: "use `.find`".to_string(),
                    edits: vec![(method.span, "find".to_string())],
                });
                self.push(&FREE_FOLD, diag);
            }
            return;
        }
        let ExprKind::Name(name) = &callee.kind else {
            return;
        };
        let Some(seq) = args.first() else {
            return;
        };
        if self.declared.contains(name.as_str())
            || seq.name.is_some()
            || matches!(seq.value.kind, ExprKind::Comprehension { .. })
        {
            return;
        }
        let positional = args.iter().all(|a| a.name.is_none() && !a.mutable);
        let fits = match name.as_str() {
            "sum" | "prod" => args.len() == 1 && !seq.mutable,
            "min" | "max" => {
                !seq.mutable
                    && (args.len() == 1
                        || (args.len() == 2
                            && args[1].name.as_ref().map(|n| n.text.as_str()) == Some("by")))
            }
            "count" | "all" | "any" | "first" => {
                args.len() == 2 && positional && is_test(&args[1].value)
            }
            "sorted" => !seq.mutable && args[1..].iter().all(|a| a.name.is_some()),
            "sort" => seq.mutable && args[1..].iter().all(|a| a.name.is_some()),
            _ => false,
        };
        // `sorted(V, by: k)` walks a domain, not a value; `min([a, b, c])`
        // is a list literal rather than a named sequence.
        let skip = match &seq.value.kind {
            ExprKind::Name(n) => n.starts_with(char::is_uppercase),
            ExprKind::List(_) => matches!(name.as_str(), "min" | "max") && args.len() == 1,
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
        let method = if name == "first" {
            "find"
        } else {
            name.as_str()
        };
        let call = format!("{recv}.{method}({rest})");
        let mut diag = Diagnostic::warning(
            FREE_FOLD.code,
            expr.span,
            format!("`{name}` of one sequence reads as its method"),
        )
        .with_help(format!("write `{call}`; the method follows the sequence, so a chain like `xs.filter(..).sum()` reads in order"));
        diag.fixes.push(Fix {
            title: format!("use `.{method}`"),
            edits: vec![(expr.span, call)],
        });
        self.push(&FREE_FOLD, diag);
    }

    /// `t.preorder` / `t.postorder` of a `Rooted` tree: the preorder array `t.at`,
    /// reversed for children first.
    fn tree_order(&mut self, expr: &Expr) {
        let ExprKind::Field { name, .. } = &expr.kind else {
            return;
        };
        let to = match name.text.as_str() {
            "preorder" => "at",
            "postorder" | "post_order" => "at.rev",
            _ => return,
        };
        let mut diag = Diagnostic::warning(
            TREE_ORDER.code,
            name.span,
            format!("`{}` is the preorder array `{to}`", name.text),
        )
        .with_help(format!("write `.{to}`; the tree already keeps that order, so there is no array to build or keep in step"));
        diag.fixes.push(Fix {
            title: format!("use `.{to}`"),
            edits: vec![(name.span, to.to_string())],
        });
        self.push(&TREE_ORDER, diag);
    }

    /// `_by` twin (`min_by`, `max_by`, `argmax_by`, `runs_by`, `first_ge_by`, ...):
    /// rewrite to the base function with a `by:` option.
    fn by_twin(&mut self, expr: &Expr) {
        let ExprKind::Call { callee, args } = &expr.kind else {
            return;
        };
        let twin_info = |name: &str| -> Option<(&'static str, usize, bool)> {
            // (base_name, extra_args_before_key, takes_mut_seq_in_free_form)
            match name {
                "min_by" => Some(("min", 0, false)),
                "max_by" => Some(("max", 0, false)),
                "argmax_by" => Some(("argmax", 0, false)),
                "argmin_by" => Some(("argmin", 0, false)),
                "sort_by" => Some(("sort", 0, true)),
                "sorted_by" => Some(("sorted", 0, false)),
                "is_sorted_by" => Some(("is_sorted", 0, false)),
                "runs_by" => Some(("runs", 0, false)),
                "run_ranges_by" => Some(("run_ranges", 0, false)),
                "group_by" => Some(("group", 0, false)),
                "first_ge_by" => Some(("first_ge", 1, false)),
                "last_le_by" => Some(("last_le", 1, false)),
                "first_gt_by" => Some(("first_gt", 1, false)),
                "last_lt_by" => Some(("last_lt", 1, false)),
                _ => None,
            }
        };

        // Method form: `xs.<twin>(...args, [by:] k)` -> `xs.<base>(...args, by: k)`
        if let ExprKind::Field { name: method, .. } = &callee.kind {
            let Some((base, extra, _)) = twin_info(method.text.as_str()) else {
                return;
            };
            if args.len() != extra + 1 {
                return;
            }
            let key = &args[extra];
            let key_is_by = key.name.as_ref().map(|n| n.text.as_str()) == Some("by");
            if key.name.is_some() && !key_is_by {
                return;
            }
            let mut edits = vec![(method.span, base.to_string())];
            if !key_is_by {
                edits.push((
                    Span {
                        start: key.value.span.start,
                        end: key.value.span.start,
                    },
                    "by: ".to_string(),
                ));
            }
            let mut diag = Diagnostic::warning(
                BY_TWIN.code,
                method.span,
                format!("`{}` is replaced by `.{base}(.., by:)`", method.text),
            )
            .with_help(format!("use `.{base}(.., by:)`; options are named arguments, so there is one `{base}` and no `_by` twin to remember"));
            diag.fixes.push(Fix {
                title: format!("use `.{base}(.., by:)`"),
                edits,
            });
            self.push(&BY_TWIN, diag);
            return;
        }

        // Free form: `<twin>(seq, ...args, [by:] k)`
        let ExprKind::Name(name) = &callee.kind else {
            return;
        };
        let Some((base, extra, wants_mut)) = twin_info(name.as_str()) else {
            return;
        };
        if self.declared.contains(name.as_str()) || args.len() != extra + 2 {
            return;
        }
        let seq = &args[0];
        if seq.name.is_some() || seq.mutable != wants_mut {
            return;
        }
        let key = &args[extra + 1];
        let key_is_by = key.name.as_ref().map(|n| n.text.as_str()) == Some("by");
        if key.name.is_some() && !key_is_by {
            return;
        }
        let key_text = self.text(key.value.span);
        let prefix = if extra == 1 {
            format!("{}, ", self.text(args[1].value.span))
        } else {
            String::new()
        };
        let keep_free = extra == 0
            && matches!(base, "min" | "max" | "sorted")
            && match &seq.value.kind {
                ExprKind::Name(n) => n.starts_with(char::is_uppercase),
                ExprKind::Comprehension { .. } => true,
                _ => false,
            };
        let call = if keep_free {
            format!("{base}({}, by: {key_text})", self.text(seq.value.span))
        } else {
            let recv = self.text(seq.value.span);
            let recv = if postfix(&seq.value) {
                recv.to_string()
            } else {
                format!("({recv})")
            };
            format!("{recv}.{base}({prefix}by: {key_text})")
        };
        let mut diag = Diagnostic::warning(
            BY_TWIN.code,
            expr.span,
            format!("`{name}` is replaced by `{base}` with `by:`"),
        )
        .with_help(format!("write `{call}`; options are named arguments, so there is one `{base}` and no `_by` twin to remember"));
        diag.fixes.push(Fix {
            title: format!("use `{base}(.., by:)`"),
            edits: vec![(expr.span, call)],
        });
        self.push(&BY_TWIN, diag);
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
        if method.text == "sorted" {
            if let ExprKind::Name(n) = &base.kind {
                if n.chars().next().is_some_and(char::is_uppercase) {
                    let mut diag = Diagnostic::warning(
                        DOMAIN_SPELLING.code,
                        method.span,
                        format!("use `{n}.sort(...)` instead of `{n}.sorted(...)`"),
                    )
                    .with_help("domains sort with `.sort(by:)`, matching `xs.sort(by:)`");
                    diag.fixes.push(Fix {
                        title: "rename to `sort`".into(),
                        edits: vec![(method.span, "sort".into())],
                    });
                    self.push(&DOMAIN_SPELLING, diag);
                    return;
                }
            }
        }
        if method.text == "first" && args.len() == 1 {
            if let Some(arg_name) = &args[0].name {
                if arg_name.text == "where" {
                    let label_span = Span {
                        start: arg_name.span.start,
                        end: args[0].value.span.start,
                    };
                    let mut diag = Diagnostic::warning(
                        DOMAIN_SPELLING.code,
                        arg_name.span,
                        "a predicate search is `.find(p)`, without `where:`",
                    )
                    .with_help(
                        "`first` is the first element; `find(p)` is the first one `p` holds for",
                    );
                    diag.fixes.push(Fix {
                        title: "use `.find(p)`".into(),
                        edits: vec![(method.span, "find".into()), (label_span, String::new())],
                    });
                    self.push(&DOMAIN_SPELLING, diag);
                }
            }
        }
    }

    /// `prefix1(a)`, `prefix2(a)`, `prefix_n(a)` -> `Prefix(a)`,
    /// `Prefix1`/`Prefix2`/`PrefixN` -> `Prefix`, `Diff1`/`Diff2` -> `Diff`.
    fn prefix_twin(&mut self, expr: &Expr) {
        let (target_span, name, replacement) = match &expr.kind {
            ExprKind::Call { callee, args } => {
                if let ExprKind::Name(name) = &callee.kind {
                    let one_arg = args.len() == 1 && args[0].name.is_none() && !args[0].mutable;
                    if matches!(name.as_str(), "Prefix1" | "Prefix2" | "PrefixN")
                        || (one_arg && matches!(name.as_str(), "prefix1" | "prefix2" | "prefix_n"))
                    {
                        (callee.span, name.as_str(), "Prefix")
                    } else if matches!(name.as_str(), "Diff1" | "Diff2") {
                        (callee.span, name.as_str(), "Diff")
                    } else {
                        return;
                    }
                } else {
                    return;
                }
            }
            ExprKind::Generic { base, .. } => {
                if let ExprKind::Name(name) = &base.kind {
                    if matches!(name.as_str(), "Prefix1" | "Prefix2" | "PrefixN") {
                        (base.span, name.as_str(), "Prefix")
                    } else if matches!(name.as_str(), "Diff1" | "Diff2") {
                        (base.span, name.as_str(), "Diff")
                    } else {
                        return;
                    }
                } else {
                    return;
                }
            }
            _ => return,
        };
        if self.declared.contains(replacement) || self.declared.contains(name) {
            return;
        }
        let mut diag = Diagnostic::warning(
            PREFIX_TWIN.code,
            target_span,
            format!("use `{replacement}` instead of `{name}`"),
        )
        .with_help(
            "`Prefix(a)` and `Diff` dispatch by shape across `[D]T`, `[R, C]T`, and `List<T>`",
        );
        diag.fixes.push(Fix {
            title: format!("rename to `{replacement}`"),
            edits: vec![(target_span, replacement.into())],
        });
        self.push(&PREFIX_TWIN, diag);
    }

    /// Type annotations `Prefix1`/`Prefix2`/`PrefixN` -> `Prefix`, `Diff1`/`Diff2` -> `Diff`.
    fn prefix_type_twin(&mut self, ty: &cpl_syntax::ast::Type) {
        let TypeKind::Named { name, .. } = &ty.kind else {
            return;
        };
        let replacement = match name.text.as_str() {
            "Prefix1" | "Prefix2" | "PrefixN" => "Prefix",
            "Diff1" | "Diff2" => "Diff",
            _ => return,
        };
        if self.declared.contains(replacement) || self.declared.contains(name.text.as_str()) {
            return;
        }
        let mut diag = Diagnostic::warning(
            PREFIX_TWIN.code,
            name.span,
            format!("use `{replacement}` instead of `{}`", name.text),
        )
        .with_help("`Prefix` and `Diff` dispatch by shape across `[D]T`, `[R, C]T`, and `List<T>`");
        diag.fixes.push(Fix {
            title: format!("rename to `{replacement}`"),
            edits: vec![(name.span, replacement.into())],
        });
        self.push(&PREFIX_TWIN, diag);
    }

    /// `use std.prefix.{prefix1, Prefix2, Diff1, Diff2}` or `use std.prefix.Diff1` -> drop/rename.
    fn prefix_use_twin(&mut self, stmt: &Stmt) {
        let StmtKind::Use { path, names, .. } = &stmt.kind else {
            return;
        };
        if path.len() == 3 && names.is_empty() && path[0].text == "std" && path[1].text == "prefix"
        {
            let replacement = match path[2].text.as_str() {
                "prefix1" | "prefix2" | "prefix_n" => String::new(),
                "Prefix1" | "Prefix2" | "PrefixN" => "use std.prefix.Prefix".into(),
                "Diff1" | "Diff2" => "use std.prefix.Diff".into(),
                _ => return,
            };
            let mut diag = Diagnostic::warning(
                PREFIX_TWIN.code,
                stmt.span,
                "removed `std.prefix` arity-suffixed import",
            )
            .with_help("`prefix`, `Prefix`, and `Diff` are in the prelude and dispatch by shape");
            diag.fixes.push(Fix {
                title: "migrate `std.prefix` import".into(),
                edits: vec![(stmt.span, replacement)],
            });
            self.push(&PREFIX_TWIN, diag);
            return;
        }
        let is_std_prefix = path.len() == 2 && path[0].text == "std" && path[1].text == "prefix";
        if !is_std_prefix || names.is_empty() {
            return;
        }
        let has_old = names.iter().any(|n| {
            matches!(
                n.text.as_str(),
                "prefix1"
                    | "prefix2"
                    | "prefix_n"
                    | "Prefix1"
                    | "Prefix2"
                    | "PrefixN"
                    | "Diff1"
                    | "Diff2"
            )
        });
        if !has_old {
            return;
        }
        let mut kept: Vec<String> = Vec::new();
        for n in names {
            let mapped = match n.text.as_str() {
                "prefix1" | "prefix2" | "prefix_n" => continue,
                "Prefix1" | "Prefix2" | "PrefixN" => "Prefix",
                "Diff1" | "Diff2" => "Diff",
                other => other,
            };
            if !kept.iter().any(|k| k == mapped) {
                kept.push(mapped.to_string());
            }
        }
        let replacement = if kept.is_empty() {
            String::new()
        } else {
            format!("use std.prefix.{{{}}}", kept.join(", "))
        };
        let mut diag = Diagnostic::warning(
            PREFIX_TWIN.code,
            stmt.span,
            "removed `std.prefix` arity-suffixed import",
        )
        .with_help("`prefix`, `Prefix`, and `Diff` are in the prelude and dispatch by shape");
        diag.fixes.push(Fix {
            title: "migrate `std.prefix` import".into(),
            edits: vec![(stmt.span, replacement)],
        });
        self.push(&PREFIX_TWIN, diag);
    }

    /// `s.first_ge(k)` / `first_gt(k)` / `last_le(k)` / `last_lt(k)` / `count_range(lo, hi)`
    /// on `SortedBag`/`SortedSet` -> `s.range(...)`.
    fn sorted_range_method(&mut self, expr: &Expr) {
        let ExprKind::Call { callee, args } = &expr.kind else {
            return;
        };
        let ExprKind::Field { base, name } = &callee.kind else {
            return;
        };
        if !matches!(
            name.text.as_str(),
            "first_ge" | "first_gt" | "last_le" | "last_lt" | "count_range"
        ) {
            return;
        }
        let is_int_key = match &base.kind {
            ExprKind::Name(v) => match self.sorted_containers.get(v) {
                Some(&b) => b,
                None if name.text == "count_range" => true,
                None => return,
            },
            _ if name.text == "count_range" => true,
            _ => return,
        };
        let replacement = match (name.text.as_str(), args.as_slice()) {
            ("first_ge", [a]) if a.name.is_none() => {
                format!("range({}..).first", self.text(a.value.span))
            }
            ("last_le", [a]) if a.name.is_none() => {
                format!("range(..={}).last", self.text(a.value.span))
            }
            ("last_lt", [a]) if a.name.is_none() => {
                format!("range(..{}).last", self.text(a.value.span))
            }
            ("first_gt", [a]) if a.name.is_none() => {
                let q = self.text(a.value.span);
                if is_int_key {
                    format!("range({q} + 1..).first")
                } else {
                    format!("first_true(x => x > {q})")
                }
            }
            ("count_range", [lo, hi]) if lo.name.is_none() && hi.name.is_none() => format!(
                "range({}..{}).len",
                self.text(lo.value.span),
                self.text(hi.value.span)
            ),
            _ => return,
        };
        let replace_span = Span {
            start: name.span.start,
            end: expr.span.end,
        };
        let mut diag = Diagnostic::warning(
            SORTED_RANGE_METHOD.code,
            expr.span,
            format!("use `.{replacement}` instead of `.{}`", name.text),
        )
        .with_help(format!(
            "`cpl fix` rewrites `.{}` to `.{replacement}`",
            name.text
        ));
        diag.fixes.push(Fix {
            title: format!("rewrite `.{}` to `.{replacement}`", name.text),
            edits: vec![(replace_span, replacement)],
        });
        self.push(&SORTED_RANGE_METHOD, diag);
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
            let diag = Diagnostic::warning(
                LOOP_REDUCTION.code,
                self.pair_span(decl.span, next.span),
                format!("a loop that only folds `{name}` through `{f}` is one `fold`"),
            )
            .with_help(format!(
                "write `{seq}.fold({f}, {i})`; one call says what the loop computes, with the start value next to it"
            ));
            self.push(&LOOP_REDUCTION, diag);
            return;
        }
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
            AssignOp::Binary(BinOp::Add) if i == "0" && c.is_none() && v == p => {
                ("sum", Some(format!("{}.sum()", receiver(&it))))
            }
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
        let inside = |span: Span| {
            uses.reads
                .iter()
                .any(|r| r.start <= span.start && span.end <= r.end)
        };
        if !uses.ok || uses.reads.is_empty() || !uses.names.iter().all(|s| inside(*s)) {
            return;
        }
        let mut diag = Diagnostic::warning(
            INDEX_ONLY_LOOP.code,
            pat.span.to(iter.span),
            format!("`{index}` is only used as `{list}[{index}]`"),
        )
        .with_help(format!(
            "iterate the values: `for x in {list}:`; with no index there is nothing to get wrong"
        ));
        if let Some(elem) = list
            .strip_suffix('s')
            .filter(|e| e.len() > 1 && !self.idents.contains(e))
        {
            let mut edits = vec![(pat.span, elem.to_string()), (iter.span, list.to_string())];
            edits.extend(uses.reads.iter().map(|r| (*r, elem.to_string())));
            diag.help = Some(format!(
                "iterate the values: `for {elem} in {list}:`; with no index there is nothing to get wrong"
            ));
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
    let flipped = match op {
        CmpOp::Lt => ">=",
        CmpOp::Le => ">",
        CmpOp::Gt => "<=",
        CmpOp::Ge => "<",
        CmpOp::Eq => "!=",
        CmpOp::Ne => "==",
        _ => return None,
    };
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

/// `stmts` assign no place but `name`: calls (`print`, `seen.add(v)`) may
/// run, but no other variable or table moves.
fn only_sets(stmts: &[Stmt], name: &str) -> bool {
    stmts.iter().all(|s| match &s.kind {
        StmtKind::Assign { target, .. } => {
            matches!(&target.kind, ExprKind::Name(n) if n == name)
        }
        StmtKind::Break(None) | StmtKind::Continue(None) => true,
        StmtKind::Let { .. } => true,
        StmtKind::Expr(Expr {
            kind: ExprKind::If {
                branches,
                otherwise,
            },
            ..
        }) => {
            branches.iter().all(|(_, b)| only_sets(&b.stmts, name))
                && otherwise.as_ref().is_none_or(|b| only_sets(&b.stmts, name))
        }
        StmtKind::Expr(_) => true,
        _ => false,
    })
}

/// Some block under `stmts` sets `name` and then breaks.
fn breaks_after_setting(stmts: &[Stmt], name: &str) -> bool {
    let sets = |s: &Stmt| {
        matches!(&s.kind, StmtKind::Assign { target, op: AssignOp::Set, .. }
            if matches!(&target.kind, ExprKind::Name(n) if n == name))
    };
    stmts
        .windows(2)
        .any(|w| sets(&w[0]) && matches!(w[1].kind, StmtKind::Break(None)))
        || stmts.iter().any(|s| match &s.kind {
            StmtKind::Expr(Expr {
                kind:
                    ExprKind::If {
                        branches,
                        otherwise,
                    },
                ..
            }) => {
                branches
                    .iter()
                    .any(|(_, b)| breaks_after_setting(&b.stmts, name))
                    || otherwise
                        .as_ref()
                        .is_some_and(|b| breaks_after_setting(&b.stmts, name))
            }
            StmtKind::Unchecked(b) => breaks_after_setting(&b.stmts, name),
            _ => false,
        })
}

/// `xs.push(e)`: `e`.
fn pushed_onto<'e>(e: &'e Expr, name: &str) -> Option<&'e Expr> {
    let ExprKind::Call { callee, args } = &e.kind else {
        return None;
    };
    let ExprKind::Field { base, name: method } = &callee.kind else {
        return None;
    };
    match (&base.kind, &args[..]) {
        (ExprKind::Name(n), [arg]) if n == name && method.text == "push" && arg.name.is_none() => {
            Some(&arg.value)
        }
        _ => None,
    }
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

/// A test passed to a fold: a one-parameter lambda or a name.
fn is_test(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Lambda { params, .. } => params.len() == 1,
        ExprKind::Name(_) | ExprKind::Field { .. } => true,
        _ => false,
    }
}

/// Whether `.m()` binds to all of `e` without parentheses.
fn postfix(e: &Expr) -> bool {
    matches!(
        e.kind,
        ExprKind::Name(_)
            | ExprKind::Field { .. }
            | ExprKind::Call { .. }
            | ExprKind::Index { .. }
            | ExprKind::Paren(_)
            | ExprKind::List(_)
            | ExprKind::Tuple(_)
            | ExprKind::Str(_)
    )
}
