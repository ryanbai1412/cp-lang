//! Taste lints: code that works but reads like translated C++
//! (`plan/PORTING.md`, "Taste checklist"). Each names the construct the
//! language has for the idea, and rewrites to it where the rewrite cannot
//! change what the program does.

use crate::walk::{self, Bind, Visitor};
use crate::{
    File, Finding, BY_TWIN, DISCARDED_RESULT, DOMAIN_SPELLING, FREE_FOLD, HAND_ROLLED_RANGE_TREE,
    INDEX_ONLY_LOOP, LOOP_REDUCTION, NONE_CHECK_UNWRAP, PREFIX_TWIN, PUSH_LOOP, REPEATED_LITERAL,
    SEARCH_LOOP, SYMMETRIC_WRITE, TREE_ORDER,
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
    rule.findings.extend(hand_rolled_range_trees(file));
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
        self.taste.none_check_unwrap(expr);
        self.taste.free_fold(expr);
        self.taste.by_twin(expr);
        self.taste.tree_order(expr);
        self.taste.domain_spelling(expr);
        self.taste.prefix_twin(expr);
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
        let (cond, only) = match &only.kind {
            StmtKind::Expr(Expr {
                kind:
                    ExprKind::If {
                        branches,
                        otherwise: None,
                    },
                ..
            }) => match &branches[..] {
                [(Cond::Expr(cond), then)] => match &then.stmts[..] {
                    [only] => (Some(cond), only),
                    _ => return,
                },
                _ => return,
            },
            _ => (None, only),
        };
        match &only.kind {
            StmtKind::Assign { target, op, value }
                if matches!(&target.kind, ExprKind::Name(n) if n == name) =>
            {
                self.reduce(name, init, pat, iter, cond, op, value, decl, next)
            }
            StmtKind::Expr(e) => {
                if let Some(value) = pushed_onto(e, name) {
                    self.collect(name, init, pat, iter, cond, value, decl, next)
                }
            }
            _ => {}
        }
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
        if mentions(v, name) || c.is_some_and(|c| mentions(c, name)) || mentions(&it, name) {
            return;
        }
        let filter = c.map(|c| format!(" if {c}")).unwrap_or_default();
        let comp = format!("[{v} for {p} in {it}{filter}]");
        let text = if items.is_empty() { comp.clone() } else { format!("{i} + {comp}") };
        let mut diag = Diagnostic::warning(
            PUSH_LOOP.code,
            self.pair_span(decl.span, next.span),
            format!("a loop that only pushes onto `{name}` is a comprehension"),
        )
        .with_help(format!("write `{text}`"));
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
        let (ExprKind::Name(x), [(CmpOp::Ne, Expr { kind: ExprKind::None, .. })]) = (&first.kind, &rest[..]) else {
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
            format!("`{x} != none` and then `{x}!`: the check is said twice"),
        )
        .with_help(format!(
            "bind it once: `if let v = {x} and ..:`, or compare the option itself (`{x} == some(..)`, `{x} ?? d`)"
        ));
        self.push(&NONE_CHECK_UNWRAP, diag);
    }

    /// `sum(xs)`, `count(xs, p)`, `sorted(xs, by: k)`, `sort(mut xs)`:
    /// a fold or sort of one sequence, which reads as its method.
    fn free_fold(&mut self, expr: &Expr) {
        let ExprKind::Call { callee, args } = &expr.kind else {
            return;
        };
        // `xs.find(x => ..)` is the element a test holds for: `xs.first(..)`,
        // where `xs.find(x)` is the index of a value.
        if let (ExprKind::Field { name: method, .. }, [test]) = (&callee.kind, &args[..]) {
            if method.text == "find" && test.name.is_none() && matches!(test.value.kind, ExprKind::Lambda { .. }) {
                let mut diag = Diagnostic::warning(
                    FREE_FOLD.code,
                    method.span,
                    "the element a test holds for is `first`; `find` is the index of a value",
                )
                .with_help("write `.first(..)`");
                diag.fixes.push(Fix {
                    title: "use `.first`".to_string(),
                    edits: vec![(method.span, "first".to_string())],
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
                        || (args.len() == 2 && args[1].name.as_ref().map(|n| n.text.as_str()) == Some("by")))
            }
            "count" | "all" | "any" | "first" => args.len() == 2 && positional && is_test(&args[1].value),
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
        let recv = if postfix(&seq.value) { recv.to_string() } else { format!("({recv})") };
        let rest = match (args.get(1), args.last()) {
            (Some(second), Some(last)) => {
                let start = second.name.as_ref().map_or(second.value.span.start, |n| n.span.start);
                self.text(Span { start, end: last.value.span.end })
            }
            _ => "",
        };
        let call = format!("{recv}.{name}({rest})");
        let mut diag = Diagnostic::warning(
            FREE_FOLD.code,
            expr.span,
            format!("`{name}` of one sequence reads as its method"),
        )
        .with_help(format!("write `{call}`"));
        diag.fixes.push(Fix {
            title: format!("use `.{name}`"),
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
        .with_help(format!("write `.{to}`"));
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
                    Span { start: key.value.span.start, end: key.value.span.start },
                    "by: ".to_string(),
                ));
            }
            let mut diag = Diagnostic::warning(
                BY_TWIN.code,
                method.span,
                format!("`{}` is replaced by `.{base}(.., by:)`", method.text),
            )
            .with_help(format!("use `.{base}(.., by:)`"));
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
            let recv = if postfix(&seq.value) { recv.to_string() } else { format!("({recv})") };
            format!("{recv}.{base}({prefix}by: {key_text})")
        };
        let mut diag = Diagnostic::warning(
            BY_TWIN.code,
            expr.span,
            format!("`{name}` is replaced by `{base}` with `by:`"),
        )
        .with_help(format!("write `{call}`"));
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
                        "pass the predicate to `.first(p)` positionally, without `where:`",
                    )
                    .with_help("use `.first(p)` to match `xs.first(p)`");
                    diag.fixes.push(Fix {
                        title: "drop `where:`".into(),
                        edits: vec![(label_span, String::new())],
                    });
                    self.push(&DOMAIN_SPELLING, diag);
                }
            }
        }
    }

    /// `prefix1(a)`, `prefix2(a)`, `prefix_n(a)` -> `prefix(a)` (prelude overload).
    fn prefix_twin(&mut self, expr: &Expr) {
        let ExprKind::Call { callee, args } = &expr.kind else {
            return;
        };
        if args.len() != 1 || args[0].name.is_some() || args[0].mutable {
            return;
        }
        let ExprKind::Name(name) = &callee.kind else {
            return;
        };
        if !matches!(name.as_str(), "prefix1" | "prefix2" | "prefix_n") {
            return;
        }
        if self.declared.contains("prefix") || self.declared.contains(name.as_str()) {
            return;
        }
        let mut diag = Diagnostic::warning(
            PREFIX_TWIN.code,
            callee.span,
            format!("use `prefix(...)` instead of `{name}(...)`"),
        )
        .with_help("`prefix(a)` handles `List<T>`, `[D]T`, and `[R, C]T` automatically");
        diag.fixes.push(Fix {
            title: "rename to `prefix`".into(),
            edits: vec![(callee.span, "prefix".into())],
        });
        self.push(&PREFIX_TWIN, diag);
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

/// `xs.push(e)`: `e`.
fn pushed_onto<'e>(e: &'e Expr, name: &str) -> Option<&'e Expr> {
    let ExprKind::Call { callee, args } = &e.kind else {
        return None;
    };
    let ExprKind::Field { base, name: method } = &callee.kind else {
        return None;
    };
    match (&base.kind, &args[..]) {
        (ExprKind::Name(n), [arg]) if n == name && method.text == "push" && arg.name.is_none() => Some(&arg.value),
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
        /// The types of `let x: T = read()`: each is the statement's bound
        /// for its own input, so equal bounds there are separate facts.
        input_types: Vec<Span>,
    }
    impl Visitor for Ints {
        fn stmt(&mut self, stmt: &Stmt) {
            if let StmtKind::Let {
                ty: Some(ty),
                init: Some(init),
                ..
            } = &stmt.kind
            {
                if let ExprKind::Call { callee, .. } = &init.kind {
                    if matches!(&callee.kind, ExprKind::Name(n) if n == "read") {
                        self.input_types.push(ty.span.shift(self.base));
                    }
                }
            }
        }
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
        input_types: Vec::new(),
    };
    file.walk(&mut ints, |i, base| i.base = base);
    let input_types = ints.input_types;
    for spans in ints.seen.values_mut() {
        spans.retain(|s| !input_types.iter().any(|t| t.start <= s.start && s.end <= t.end));
    }
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

/// Flags hand-rolled Fenwick loops (`p += p & -p` / `p -= p & -p` with table
/// indexing) and hand-rolled iterative or recursive segment trees (`2 * p` /
/// `2 * p + 1` with halving or range-guarded midpoint recursion).
fn hand_rolled_range_trees(file: &File) -> Vec<Finding> {
    struct Scan {
        base: u32,
        findings: Vec<Finding>,
    }

    impl Scan {
        fn warn(&mut self, span: Span, msg: &str, help: &str) {
            self.findings.push(Finding {
                lint: &HAND_ROLLED_RANGE_TREE,
                diag: Diagnostic::warning(HAND_ROLLED_RANGE_TREE.code, span.shift(self.base), msg)
                    .with_help(help),
            });
        }
    }

    impl Visitor for Scan {
        fn stmt(&mut self, stmt: &Stmt) {
            match &stmt.kind {
                StmtKind::While { body, .. } => {
                    if let Some(var) = fenwick_step_var(body) {
                        if body_indexes_by(body, &var) {
                            self.warn(
                                stmt.span,
                                "hand-rolled Fenwick loop",
                                "use `Fenwick`, `DualFenwick`, or `RangeFenwick` from `std.fenwick`",
                            );
                        }
                    } else if is_iterative_segtree_loop(body) {
                        self.warn(
                            stmt.span,
                            "hand-rolled segment tree loop",
                            "use `SegTree` (`std.segtree`) or `LazySegTree` / `DualSegTree` (`std.lazy`)",
                        );
                    }
                }
                StmtKind::Fn(decl) => {
                    if let FnBody::Block(body) = &decl.body {
                        if is_recursive_segtree_fn(decl, body) {
                            self.warn(
                                decl.name.span,
                                "hand-rolled recursive segment tree",
                                "use `SegTree` (`std.segtree`) or `LazySegTree` / `DualSegTree` (`std.lazy`)",
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    }

    let mut scan = Scan {
        base: 0,
        findings: Vec::new(),
    };
    file.walk(&mut scan, |s, base| s.base = base);
    scan.findings
}

fn expr_name(e: &Expr) -> Option<&str> {
    match &walk::unparen(e).kind {
        ExprKind::Name(s) => Some(s.as_str()),
        _ => None,
    }
}

fn is_int_lit(e: &Expr, want: i128) -> bool {
    matches!(walk::unparen(e).kind, ExprKind::Int(v) if v == want)
}

/// Matches `v & -v` or `-v & v`.
fn is_lowbit(e: &Expr, var: &str) -> bool {
    let ExprKind::Binary {
        op: BinOp::BitAnd,
        lhs,
        rhs,
        ..
    } = &walk::unparen(e).kind
    else {
        return false;
    };
    let is_neg = |x: &Expr| {
        matches!(
            &walk::unparen(x).kind,
            ExprKind::Unary { op: UnOp::Neg, expr } if expr_name(expr) == Some(var)
        )
    };
    (expr_name(lhs) == Some(var) && is_neg(rhs)) || (expr_name(rhs) == Some(var) && is_neg(lhs))
}

/// Matches `v | (v + 1)` or `(v & (v + 1)) - 1` (0-indexed Fenwick step).
fn is_zero_indexed_bit_step(e: &Expr, var: &str) -> bool {
    let is_v_plus_1 = |x: &Expr| {
        matches!(
            &walk::unparen(x).kind,
            ExprKind::Binary { op: BinOp::Add, lhs, rhs, .. }
                if (expr_name(lhs) == Some(var) && is_int_lit(rhs, 1))
                    || (expr_name(rhs) == Some(var) && is_int_lit(lhs, 1))
        )
    };
    match &walk::unparen(e).kind {
        ExprKind::Binary {
            op: BinOp::BitOr,
            lhs,
            rhs,
            ..
        } => {
            (expr_name(lhs) == Some(var) && is_v_plus_1(rhs))
                || (expr_name(rhs) == Some(var) && is_v_plus_1(lhs))
        }
        ExprKind::Binary {
            op: BinOp::Sub,
            lhs,
            rhs,
            ..
        } if is_int_lit(rhs, 1) => matches!(
            &walk::unparen(lhs).kind,
            ExprKind::Binary { op: BinOp::BitAnd, lhs: a, rhs: b, .. }
                if (expr_name(a) == Some(var) && is_v_plus_1(b))
                    || (expr_name(b) == Some(var) && is_v_plus_1(a))
        ),
        _ => false,
    }
}

/// Returns the stepped variable if `body` updates a variable with a BIT step.
fn fenwick_step_var(body: &Block) -> Option<String> {
    for stmt in &body.stmts {
        if let StmtKind::Assign { target, op, value } = &stmt.kind {
            let Some(var) = expr_name(target) else {
                continue;
            };
            match op {
                AssignOp::Binary(BinOp::Add | BinOp::Sub) if is_lowbit(value, var) => {
                    return Some(var.to_string());
                }
                AssignOp::Set => {
                    if let ExprKind::Binary {
                        op: BinOp::Add | BinOp::Sub,
                        lhs,
                        rhs,
                        ..
                    } = &walk::unparen(value).kind
                    {
                        if expr_name(lhs) == Some(var) && is_lowbit(rhs, var) {
                            return Some(var.to_string());
                        }
                    }
                    if is_zero_indexed_bit_step(value, var) {
                        return Some(var.to_string());
                    }
                }
                _ => {}
            }
        }
    }
    None
}

fn walk_block(v: &mut dyn Visitor, block: &Block) {
    for s in &block.stmts {
        walk::walk_stmt(v, s);
    }
}

fn expr_mentions(e: &Expr, var: &str) -> bool {
    struct Find<'a>(&'a str, bool);
    impl Visitor for Find<'_> {
        fn expr(&mut self, expr: &Expr) {
            if let ExprKind::Name(s) = &expr.kind {
                if s == self.0 {
                    self.1 = true;
                }
            }
        }
    }
    let mut f = Find(var, false);
    walk::walk_expr(&mut f, e);
    f.1
}

fn body_indexes_by(body: &Block, var: &str) -> bool {
    struct Find<'a>(&'a str, bool);
    impl Visitor for Find<'_> {
        fn expr(&mut self, expr: &Expr) {
            if self.1 {
                return;
            }
            if let ExprKind::Index { indices, .. } = &expr.kind {
                if indices.iter().any(|ix| expr_mentions(ix, self.0)) {
                    self.1 = true;
                }
            }
        }
    }
    let mut f = Find(var, false);
    walk_block(&mut f, body);
    f.1
}

/// Matches `2 * v`, `v * 2`, or `v << 1`.
fn is_double(e: &Expr, var: &str) -> bool {
    match &walk::unparen(e).kind {
        ExprKind::Binary {
            op: BinOp::Mul,
            lhs,
            rhs,
            ..
        } => {
            (expr_name(lhs) == Some(var) && is_int_lit(rhs, 2))
                || (expr_name(rhs) == Some(var) && is_int_lit(lhs, 2))
        }
        ExprKind::Binary {
            op: BinOp::Shl,
            lhs,
            rhs,
            ..
        } => expr_name(lhs) == Some(var) && is_int_lit(rhs, 1),
        _ => false,
    }
}

/// Matches `2 * v + 1`, `v * 2 + 1`, `(v << 1) + 1`, or `(v << 1) | 1`.
fn is_double_plus_one(e: &Expr, var: &str) -> bool {
    match &walk::unparen(e).kind {
        ExprKind::Binary {
            op: BinOp::Add | BinOp::BitOr,
            lhs,
            rhs,
            ..
        } => {
            (is_double(lhs, var) && is_int_lit(rhs, 1))
                || (is_double(rhs, var) && is_int_lit(lhs, 1))
        }
        _ => false,
    }
}

fn is_halve_assign(stmt: &Stmt) -> Option<String> {
    let StmtKind::Assign { target, op, value } = &stmt.kind else {
        return None;
    };
    let var = expr_name(target)?;
    let halved = match op {
        AssignOp::Binary(BinOp::Shr) => is_int_lit(value, 1),
        AssignOp::Binary(BinOp::Div) => is_int_lit(value, 2),
        AssignOp::Set => match &walk::unparen(value).kind {
            ExprKind::Binary {
                op: BinOp::Shr,
                lhs,
                rhs,
                ..
            } => expr_name(lhs) == Some(var) && is_int_lit(rhs, 1),
            ExprKind::Binary {
                op: BinOp::Div,
                lhs,
                rhs,
                ..
            } => expr_name(lhs) == Some(var) && is_int_lit(rhs, 2),
            _ => false,
        },
        _ => false,
    };
    halved.then(|| var.to_string())
}

/// Bottom-up segment-tree loop: halves a node index (`p >>= 1` / `p /= 2`)
/// while indexing children (`2 * p` and `2 * p + 1`), OR halves two endpoints
/// (`l >>= 1`, `r >>= 1`) while testing parity (`& 1`) and indexing by them.
fn is_iterative_segtree_loop(body: &Block) -> bool {
    let mut halved = Vec::new();
    for stmt in &body.stmts {
        if let Some(v) = is_halve_assign(stmt) {
            if !halved.contains(&v) {
                halved.push(v);
            }
        }
    }
    if halved.is_empty() {
        return false;
    }
    struct Checks<'a> {
        vars: &'a [String],
        has_left: bool,
        has_right: bool,
        has_parity: bool,
        has_index: bool,
    }
    impl Visitor for Checks<'_> {
        fn expr(&mut self, expr: &Expr) {
            if let ExprKind::Index { indices, .. } = &expr.kind {
                for ix in indices {
                    for v in self.vars {
                        if is_double(ix, v) {
                            self.has_left = true;
                        }
                        if is_double_plus_one(ix, v) {
                            self.has_right = true;
                        }
                        if expr_mentions(ix, v) {
                            self.has_index = true;
                        }
                    }
                }
            }
            if let ExprKind::Binary {
                op: BinOp::BitAnd,
                lhs,
                rhs,
                ..
            } = &expr.kind
            {
                for v in self.vars {
                    if (expr_name(lhs) == Some(v.as_str()) && is_int_lit(rhs, 1))
                        || (expr_name(rhs) == Some(v.as_str()) && is_int_lit(lhs, 1))
                    {
                        self.has_parity = true;
                    }
                }
            }
        }
    }
    let mut c = Checks {
        vars: &halved,
        has_left: false,
        has_right: false,
        has_parity: false,
        has_index: false,
    };
    walk_block(&mut c, body);
    (c.has_left && c.has_right) || (halved.len() >= 2 && c.has_parity && c.has_index)
}

/// Top-down recursive segment-tree function: computes a midpoint `(lo + hi) / 2`,
/// recurses on `2 * node` and `2 * node + 1`, and guards a query/update range
/// or branches on a point index (`pos < mid` / `qr <= lo or hi <= ql`).
fn is_recursive_segtree_fn(decl: &FnDecl, body: &Block) -> bool {
    if decl.params.len() < 3 {
        return false;
    }
    let param_names: Vec<&str> = decl.params.iter().map(|p| p.name.text.as_str()).collect();
    struct Check<'a> {
        fn_name: &'a str,
        params: &'a [&'a str],
        has_mid: bool,
        has_left_rec: bool,
        has_right_rec: bool,
        has_range_or_point_guard: bool,
    }
    impl Visitor for Check<'_> {
        fn expr(&mut self, expr: &Expr) {
            match &expr.kind {
                ExprKind::If { branches, .. } => {
                    for (cond, _) in branches {
                        if let Cond::Expr(e) = cond {
                            match &walk::unparen(e).kind {
                                ExprKind::Binary {
                                    op: BinOp::Or | BinOp::And,
                                    lhs,
                                    rhs,
                                    ..
                                } if matches!(walk::unparen(lhs).kind, ExprKind::Compare { .. })
                                    && matches!(
                                        walk::unparen(rhs).kind,
                                        ExprKind::Compare { .. }
                                    ) =>
                                {
                                    self.has_range_or_point_guard = true;
                                }
                                ExprKind::Compare { first, rest } if rest.len() == 1 => {
                                    let (op, rhs) = &rest[0];
                                    if matches!(op, CmpOp::Lt | CmpOp::Le | CmpOp::Gt | CmpOp::Ge) {
                                        let l = expr_name(first);
                                        let r = expr_name(rhs);
                                        if matches!(l, Some("mid") | Some("m"))
                                            || matches!(r, Some("mid") | Some("m"))
                                        {
                                            self.has_range_or_point_guard = true;
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                ExprKind::Binary {
                    op: BinOp::Div,
                    lhs,
                    rhs,
                    ..
                } if is_int_lit(rhs, 2)
                    && matches!(
                        walk::unparen(lhs).kind,
                        ExprKind::Binary { op: BinOp::Add, .. }
                    ) =>
                {
                    self.has_mid = true;
                }
                ExprKind::Binary {
                    op: BinOp::Shr,
                    lhs,
                    rhs,
                    ..
                } if is_int_lit(rhs, 1)
                    && matches!(
                        walk::unparen(lhs).kind,
                        ExprKind::Binary { op: BinOp::Add, .. }
                    ) =>
                {
                    self.has_mid = true;
                }
                ExprKind::Call { callee, args } if expr_name(callee) == Some(self.fn_name) => {
                    for arg in args {
                        for p in self.params {
                            if is_double(&arg.value, p) {
                                self.has_left_rec = true;
                            }
                            if is_double_plus_one(&arg.value, p) {
                                self.has_right_rec = true;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    let mut c = Check {
        fn_name: decl.name.text.as_str(),
        params: &param_names,
        has_mid: false,
        has_left_rec: false,
        has_right_rec: false,
        has_range_or_point_guard: false,
    };
    walk_block(&mut c, body);
    c.has_mid && c.has_left_rec && c.has_right_rec && c.has_range_or_point_guard
}
