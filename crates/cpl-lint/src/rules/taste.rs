//! Taste lints: code that works but reads like translated C++
//! (`plan/PORTING.md`, "Taste checklist"). Each names the construct the
//! language has for the idea, and rewrites to it where the rewrite cannot
//! change what the program does.

use crate::walk::{self, Bind, Visitor};
use crate::{
    File, Finding, BY_TWIN, DISCARDED_RESULT, DOMAIN_SPELLING, FREE_FOLD,
    HAND_ROLLED_BINARY_LIFTING, HAND_ROLLED_DSU, HAND_ROLLED_RANGE_TREE, HAND_ROLLED_SIEVE,
    HAND_ROLLED_STRING_ALGO, INDEX_ONLY_LOOP, LOOP_REDUCTION, NONE_CHECK_UNWRAP, PREFIX_TWIN,
    PUSH_LOOP, REPEATED_LITERAL, SEARCH_LOOP, SORTED_RANGE_METHOD, SYMMETRIC_WRITE, TREE_ORDER,
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
        sorted_containers: HashMap::new(),
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
    file.walk(&mut Declared(&mut rule.declared, &mut rule.sorted_containers), |_, _| {});
    let mut blocks = Blocks { taste: &mut rule };
    file.walk(&mut blocks, |b, base| b.taste.base = base);
    rule.findings.extend(repeated_literals(file));
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

    /// `prefix1(a)`, `prefix2(a)`, `prefix_n(a)` -> `prefix(a)`,
    /// `Prefix1`/`Prefix2`/`PrefixN` -> `Prefix`, `Diff1`/`Diff2` -> `Diff`.
    fn prefix_twin(&mut self, expr: &Expr) {
        let (target_span, name, replacement) = match &expr.kind {
            ExprKind::Call { callee, args } => {
                if let ExprKind::Name(name) = &callee.kind {
                    if args.len() == 1
                        && args[0].name.is_none()
                        && !args[0].mutable
                        && matches!(
                            name.as_str(),
                            "prefix1" | "prefix2" | "prefix_n" | "Prefix1" | "Prefix2" | "PrefixN"
                        )
                    {
                        (callee.span, name.as_str(), "prefix")
                    } else if matches!(name.as_str(), "Prefix1" | "Prefix2" | "PrefixN") {
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
        .with_help("`prefix(a)`, `Prefix`, and `Diff` dispatch by shape across `[D]T`, `[R, C]T`, and `List<T>`");
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
        if path.len() == 3 && names.is_empty() && path[0].text == "std" && path[1].text == "prefix" {
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
                "prefix1" | "prefix2" | "prefix_n" | "Prefix1" | "Prefix2" | "PrefixN" | "Diff1" | "Diff2"
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
            ("first_ge", [a]) if a.name.is_none() => format!("range({}..).first", self.text(a.value.span)),
            ("last_le", [a]) if a.name.is_none() => format!("range(..={}).last", self.text(a.value.span)),
            ("last_lt", [a]) if a.name.is_none() => format!("range(..{}).last", self.text(a.value.span)),
            ("first_gt", [a]) if a.name.is_none() => {
                let q = self.text(a.value.span);
                if is_int_key {
                    format!("range({q} + 1..).first")
                } else {
                    format!("first_where(x => x > {q})")
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
        .with_help(format!("`cpl fix` rewrites `.{}` to `.{replacement}`", name.text));
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
                StmtKind::For { pat, body, .. } => {
                    if let PatternKind::Name(var) = &pat.kind {
                        if is_segtree_build_pull_loop(var, body) {
                            self.warn(
                                stmt.span,
                                "hand-rolled segment tree pull loop",
                                "use `SegTree` (`std.segtree`) or `LazySegTree` / `DualSegTree` (`std.lazy`)",
                            );
                        }
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
fn sorted_container_binding(ty: Option<&Type>, init: Option<&Expr>) -> Option<bool> {
    let key_is_int = |args: &[Type]| match args.first() {
        Some(Type {
            kind: TypeKind::Named { name, .. },
            ..
        }) => matches!(name.text.as_str(), "Int" | "I128" | "U64"),
        None => true,
        _ => false,
    };
    if let Some(Type {
        kind: TypeKind::Named { name, args },
        ..
    }) = ty
    {
        if matches!(name.text.as_str(), "SortedBag" | "SortedSet" | "OrderedSet") {
            return Some(key_is_int(args));
        }
    }
    let init = init?;
    let ExprKind::Call { callee, .. } = &walk::unparen(init).kind else {
        return None;
    };
    let head = match &callee.kind {
        ExprKind::Field { base, .. } => base.as_ref(),
        _ => callee.as_ref(),
    };
    match &head.kind {
        ExprKind::Generic { base, args } => {
            if let ExprKind::Name(n) = &base.kind {
                if matches!(n.as_str(), "SortedBag" | "SortedSet" | "OrderedSet") {
                    return Some(key_is_int(args));
                }
            }
            None
        }
        ExprKind::Name(n) if matches!(n.as_str(), "SortedBag" | "SortedSet" | "OrderedSet") => {
            Some(true)
        }
        _ => None,
    }
}

/// Flags hand-rolled KMP prefix-function, Z-function, polynomial rolling-hash,
/// and trie insertion loops that duplicate `std.strings`.
fn hand_rolled_string_algos(file: &File) -> Vec<Finding> {
    struct Scan {
        base: u32,
        findings: Vec<Finding>,
    }

    impl Scan {
        fn warn(&mut self, span: Span, msg: &str, help: &str) {
            self.findings.push(Finding {
                lint: &HAND_ROLLED_STRING_ALGO,
                diag: Diagnostic::warning(
                    HAND_ROLLED_STRING_ALGO.code,
                    span.shift(self.base),
                    msg,
                )
                .with_help(help),
            });
        }
    }

    impl Visitor for Scan {
        fn stmt(&mut self, stmt: &Stmt) {
            match &stmt.kind {
                StmtKind::For { pat, body, .. } => {
                    let var = match &pat.kind {
                        PatternKind::Name(n) => Some(n.as_str()),
                        _ => None,
                    };
                    if let Some(v) = var {
                        if is_kmp_prefix_loop(v, body) {
                            self.warn(
                                stmt.span,
                                "hand-rolled prefix-function (KMP) loop",
                                "use `prefix_function` or `find_all` from `std.strings`",
                            );
                            return;
                        }
                        if is_z_function_loop(v, body) {
                            self.warn(
                                stmt.span,
                                "hand-rolled Z-function loop",
                                "use `z_function` from `std.strings`",
                            );
                            return;
                        }
                        if is_poly_hash_loop(v, body) {
                            self.warn(
                                stmt.span,
                                "hand-rolled polynomial rolling hash",
                                "use `RollingHash` or `Hash2D` from `std.strings`",
                            );
                            return;
                        }
                    }
                    if is_trie_insert_loop(body) {
                        self.warn(
                            stmt.span,
                            "hand-rolled trie loop",
                            "use `Trie` or `AhoCorasick` from `std.strings`",
                        );
                    }
                }
                StmtKind::While { body, .. } if is_trie_insert_loop(body) => {
                    self.warn(
                        stmt.span,
                        "hand-rolled trie loop",
                        "use `Trie` or `AhoCorasick` from `std.strings`",
                    );
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

/// Extracts `(base_name, index_expr)` from `arr[idx]` or `arr.at(idx)`.
fn seq_index_parts(e: &Expr) -> Option<(&str, &Expr)> {
    match &walk::unparen(e).kind {
        ExprKind::Index { base, indices } if indices.len() == 1 => {
            Some((expr_name(base)?, &indices[0]))
        }
        ExprKind::Call { callee, args } if args.len() == 1 => {
            if let ExprKind::Field { base, name } = &walk::unparen(callee).kind {
                if name.text == "at" {
                    return Some((expr_name(base)?, &args[0].value));
                }
            }
            None
        }
        _ => None,
    }
}

/// Extracts `(lhs, op, rhs)` from a single binary comparison `lhs op rhs`.
fn single_cmp(e: &Expr) -> Option<(&Expr, CmpOp, &Expr)> {
    if let ExprKind::Compare { first, rest } = &walk::unparen(e).kind {
        if rest.len() == 1 {
            return Some((first, rest[0].0, &rest[0].1));
        }
    }
    None
}

/// Matches `var - 1`.
fn is_var_minus_one(e: &Expr, var: &str) -> bool {
    matches!(
        &walk::unparen(e).kind,
        ExprKind::Binary { op: BinOp::Sub, lhs, rhs, .. }
            if expr_name(lhs) == Some(var) && is_int_lit(rhs, 1)
    )
}

/// Matches `var + 1` or `1 + var`.
fn is_var_plus_one(e: &Expr, var: &str) -> bool {
    matches!(
        &walk::unparen(e).kind,
        ExprKind::Binary { op: BinOp::Add, lhs, rhs, .. }
            if (expr_name(lhs) == Some(var) && is_int_lit(rhs, 1))
                || (expr_name(rhs) == Some(var) && is_int_lit(lhs, 1))
    )
}

/// Collects all top-level `and`-conjuncts of a condition.
fn collect_and<'a>(e: &'a Expr, out: &mut Vec<&'a Expr>) {
    match &walk::unparen(e).kind {
        ExprKind::Binary {
            op: BinOp::And,
            lhs,
            rhs,
            ..
        } => {
            collect_and(lhs, out);
            collect_and(rhs, out);
        }
        _ => out.push(walk::unparen(e)),
    }
}

fn cond_expr(cond: &Cond) -> Option<&Expr> {
    match cond {
        Cond::Expr(e) => Some(e),
        _ => None,
    }
}

/// Matches a KMP prefix-function loop: contains a `while j > 0 and s[...] != s[...]`
/// whose body steps `j = pi[j - 1]`, and writes `pi[i] = ...` in the loop body.
fn is_kmp_prefix_loop(for_var: &str, body: &Block) -> bool {
    struct Check<'a> {
        for_var: &'a str,
        kmp_tables: Vec<String>,
        written_tables: Vec<String>,
    }
    impl Visitor for Check<'_> {
        fn stmt(&mut self, stmt: &Stmt) {
            match &stmt.kind {
                StmtKind::While { cond, body } => {
                    let Some(cond_e) = cond_expr(cond) else {
                        return;
                    };
                    let mut fallback: Option<(String, String)> = None;
                    for s in &body.stmts {
                        if let StmtKind::Assign {
                            target,
                            op: AssignOp::Set,
                            value,
                        } = &s.kind
                        {
                            if let Some(j) = expr_name(target) {
                                if let Some((pi, idx)) = seq_index_parts(value) {
                                    if is_var_minus_one(idx, j) {
                                        fallback = Some((j.to_string(), pi.to_string()));
                                    }
                                }
                            }
                        }
                    }
                    if let Some((j, pi)) = fallback {
                        let mut parts = Vec::new();
                        collect_and(cond_e, &mut parts);
                        let has_gt0 = parts.iter().any(|p| {
                            matches!(
                                single_cmp(p),
                                Some((lhs, CmpOp::Gt, rhs))
                                    if expr_name(lhs) == Some(j.as_str()) && is_int_lit(rhs, 0)
                            ) || matches!(
                                single_cmp(p),
                                Some((lhs, CmpOp::Lt, rhs))
                                    if is_int_lit(lhs, 0) && expr_name(rhs) == Some(j.as_str())
                            )
                        });
                        let has_seq_ne = parts.iter().any(|p| {
                            matches!(
                                single_cmp(p),
                                Some((lhs, CmpOp::Ne, rhs))
                                    if matches!(
                                        (seq_index_parts(lhs), seq_index_parts(rhs)),
                                        (Some((s1, _)), Some((s2, _))) if s1 == s2
                                    )
                            )
                        });
                        if has_gt0 && has_seq_ne {
                            self.kmp_tables.push(pi);
                        }
                    }
                }
                StmtKind::Assign {
                    target,
                    op: AssignOp::Set,
                    ..
                } => {
                    if let Some((arr, idx)) = seq_index_parts(target) {
                        if expr_name(idx) == Some(self.for_var) {
                            self.written_tables.push(arr.to_string());
                        }
                    }
                }
                _ => {}
            }
        }
    }
    let mut c = Check {
        for_var,
        kmp_tables: Vec::new(),
        written_tables: Vec::new(),
    };
    walk_block(&mut c, body);
    c.kmp_tables.iter().any(|t| c.written_tables.contains(t))
}

/// Checks if two expressions have the same simple shape (`var` or `arr[var]`).
fn same_simple_expr(a: &Expr, b: &Expr) -> bool {
    let a = walk::unparen(a);
    let b = walk::unparen(b);
    match (&a.kind, &b.kind) {
        (ExprKind::Name(x), ExprKind::Name(y)) => x == y,
        (
            ExprKind::Index {
                base: t1,
                indices: i1,
            },
            ExprKind::Index {
                base: t2,
                indices: i2,
            },
        ) if i1.len() == 1 && i2.len() == 1 => {
            expr_name(t1) == expr_name(t2)
                && expr_name(t1).is_some()
                && same_simple_expr(&i1[0], &i2[0])
        }
        _ => false,
    }
}

/// Matches `for_var + k` or `k + for_var`.
fn is_for_plus_expr(e: &Expr, for_var: &str, k: &Expr) -> bool {
    matches!(
        &walk::unparen(e).kind,
        ExprKind::Binary { op: BinOp::Add, lhs, rhs, .. }
            if (expr_name(lhs) == Some(for_var) && same_simple_expr(rhs, k))
                || (expr_name(rhs) == Some(for_var) && same_simple_expr(lhs, k))
    )
}

/// Matches a Z-function extension loop: `while ... and s[k] == s[i + k]: k += 1`.
fn is_z_function_loop(for_var: &str, body: &Block) -> bool {
    struct Check<'a> {
        for_var: &'a str,
        found: bool,
    }
    impl Visitor for Check<'_> {
        fn stmt(&mut self, stmt: &Stmt) {
            if self.found {
                return;
            }
            if let StmtKind::While { cond, body } = &stmt.kind {
                let Some(cond_e) = cond_expr(cond) else {
                    return;
                };
                let mut parts = Vec::new();
                collect_and(cond_e, &mut parts);
                for p in parts {
                    if let Some((lhs, CmpOp::Eq, rhs)) = single_cmp(p) {
                        if let (Some((s1, i1)), Some((s2, i2))) =
                            (seq_index_parts(lhs), seq_index_parts(rhs))
                        {
                            if s1 != s2 {
                                continue;
                            }
                            let k = if is_for_plus_expr(i2, self.for_var, i1) {
                                Some(i1)
                            } else if is_for_plus_expr(i1, self.for_var, i2) {
                                Some(i2)
                            } else {
                                None
                            };
                            if let Some(k) = k {
                                let inc = body.stmts.iter().any(|s| match &s.kind {
                                    StmtKind::Assign {
                                        target,
                                        op: AssignOp::Binary(BinOp::Add),
                                        value,
                                    } => same_simple_expr(target, k) && is_int_lit(value, 1),
                                    StmtKind::Assign {
                                        target,
                                        op: AssignOp::Set,
                                        value,
                                    } => {
                                        same_simple_expr(target, k)
                                            && matches!(
                                                &walk::unparen(value).kind,
                                                ExprKind::Binary { op: BinOp::Add, lhs, rhs, .. }
                                                    if (same_simple_expr(lhs, k) && is_int_lit(rhs, 1))
                                                        || (same_simple_expr(rhs, k) && is_int_lit(lhs, 1))
                                            )
                                    }
                                    _ => false,
                                });
                                if inc {
                                    self.found = true;
                                    return;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    let mut c = Check {
        for_var,
        found: false,
    };
    walk_block(&mut c, body);
    c.found
}

/// Unwraps an outer `% mod` if present, returning `(inner, has_mod)`.
fn strip_mod(e: &Expr) -> (&Expr, bool) {
    let u = walk::unparen(e);
    if let ExprKind::Binary {
        op: BinOp::Rem,
        lhs,
        ..
    } = &u.kind
    {
        (walk::unparen(lhs), true)
    } else {
        (u, false)
    }
}

fn is_small_radix(e: &Expr) -> bool {
    matches!(
        walk::unparen(e).kind,
        ExprKind::Int(2 | 8 | 10 | 16 | 26 | 100)
    )
}

/// Checks if an expression looks like a character/byte code (`Int(s[i])`, `ch.ord`, `'a'` offset).
fn expr_looks_like_char_code(e: &Expr) -> bool {
    struct Check {
        found: bool,
    }
    impl Visitor for Check {
        fn expr(&mut self, e: &Expr) {
            match &e.kind {
                ExprKind::Char(_) => self.found = true,
                ExprKind::Field { name, .. } if name.text == "ord" => self.found = true,
                ExprKind::Call { callee, args }
                    if expr_name(callee) == Some("Int")
                        && args.len() == 1
                        && matches!(
                            walk::unparen(&args[0].value).kind,
                            ExprKind::Index { .. } | ExprKind::Name(_)
                        ) =>
                {
                    self.found = true;
                }
                _ => {}
            }
        }
    }
    let mut c = Check { found: false };
    walk::walk_expr(&mut c, e);
    c.found
}

/// Matches a polynomial rolling-hash recurrence inside a `for` loop:
/// `h[i + 1] = (h[i] * B + term) % M` (or `h = (h * B + term) % M`) with either
/// a matching power recurrence `pw[i + 1] = (pw[i] * B) % M` in the same loop or
/// a char-code term with a non-decimal base.
fn is_poly_hash_loop(for_var: &str, body: &Block) -> bool {
    let mut hash_bases: Vec<(String, bool, bool)> = Vec::new();
    let mut pow_bases: Vec<String> = Vec::new();

    let base_key = |e: &Expr| -> Option<String> {
        let u = walk::unparen(e);
        if is_small_radix(u) {
            return None;
        }
        match &u.kind {
            ExprKind::Name(n) => Some(n.clone()),
            ExprKind::Int(v) if *v >= 27 => Some(v.to_string()),
            _ => None,
        }
    };

    for s in &body.stmts {
        let StmtKind::Assign {
            target,
            op: AssignOp::Set,
            value,
        } = &s.kind
        else {
            continue;
        };
        let (inner, has_mod) = strip_mod(value);
        let prev_matches = |cand: &Expr| -> bool {
            if let Some((arr, idx)) = seq_index_parts(target) {
                if let Some((arr2, idx2)) = seq_index_parts(cand) {
                    if arr != arr2 {
                        return false;
                    }
                    return (is_var_plus_one(idx, for_var) && expr_name(idx2) == Some(for_var))
                        || (expr_name(idx) == Some(for_var) && is_var_minus_one(idx2, for_var));
                }
                false
            } else if let Some(h) = expr_name(target) {
                expr_name(cand) == Some(h)
            } else {
                false
            }
        };

        match &inner.kind {
            ExprKind::Binary {
                op: BinOp::Mul,
                lhs,
                rhs,
                ..
            } => {
                if prev_matches(lhs) {
                    if let Some(b) = base_key(rhs) {
                        pow_bases.push(b);
                    }
                } else if prev_matches(rhs) {
                    if let Some(b) = base_key(lhs) {
                        pow_bases.push(b);
                    }
                }
            }
            ExprKind::Binary {
                op: BinOp::Add,
                lhs,
                rhs,
                ..
            } => {
                let mut check_pair = |mul_side: &Expr, term_side: &Expr| {
                    if let ExprKind::Binary {
                        op: BinOp::Mul,
                        lhs: ml,
                        rhs: mr,
                        ..
                    } = &walk::unparen(mul_side).kind
                    {
                        let base = if prev_matches(ml) {
                            base_key(mr)
                        } else if prev_matches(mr) {
                            base_key(ml)
                        } else {
                            None
                        };
                        if let Some(b) = base {
                            hash_bases.push((b, has_mod, expr_looks_like_char_code(term_side)));
                        }
                    }
                };
                check_pair(lhs, rhs);
                check_pair(rhs, lhs);
            }
            _ => {}
        }
    }

    hash_bases
        .iter()
        .any(|(b, has_mod, char_term)| pow_bases.contains(b) || (*has_mod && *char_term))
}

/// Checks if an expression looks like a trie child-table lookup on `cur`:
/// `ch[cur][c]`, `ch[cur, c]`, or `nodes[cur].next[c]`.
/// Returns `(table_key, cursor_var)`.
fn trie_child_slot(e: &Expr) -> Option<(String, String)> {
    let ExprKind::Index { base, indices } = &walk::unparen(e).kind else {
        return None;
    };
    if indices.len() == 2 {
        let tbl = expr_name(base)?;
        let cur = expr_name(&indices[0])?;
        return Some((tbl.to_string(), cur.to_string()));
    }
    if indices.len() != 1 {
        return None;
    }
    match &walk::unparen(base).kind {
        ExprKind::Index {
            base: outer,
            indices: outer_idx,
        } if outer_idx.len() == 1 => {
            let tbl = expr_name(outer)?;
            let cur = expr_name(&outer_idx[0])?;
            Some((tbl.to_string(), cur.to_string()))
        }
        ExprKind::Field {
            base: node_expr,
            name,
        } => {
            let (arr, idx) = seq_index_parts(node_expr)?;
            let cur = expr_name(idx)?;
            Some((format!("{arr}.{}", name.text), cur.to_string()))
        }
        _ => None,
    }
}

/// Matches a hand-rolled trie insertion loop: allocates a missing child slot
/// (`nodes[cur].kid[c] ??= ...` or `if ch[cur][c] == -1 / 0 / none: ch[cur][c] = ...`)
/// and advances `cur = ...` from that same child table.
fn is_trie_insert_loop(body: &Block) -> bool {
    struct Check {
        alloc_slots: Vec<(String, String)>,
        advance_vars: Vec<String>,
    }
    impl Visitor for Check {
        fn stmt(&mut self, stmt: &Stmt) {
            match &stmt.kind {
                StmtKind::Assign {
                    target,
                    op: AssignOp::Binary(BinOp::Coalesce),
                    ..
                } => {
                    if let Some(slot) = trie_child_slot(target) {
                        self.alloc_slots.push(slot);
                    }
                }
                StmtKind::Assign {
                    target,
                    op: AssignOp::Set,
                    value,
                } => {
                    if let Some(cur) = expr_name(target) {
                        let inner = match &walk::unparen(value).kind {
                            ExprKind::Unwrap(e) => e,
                            _ => value,
                        };
                        if let Some((_, c2)) = trie_child_slot(inner) {
                            if c2 == cur {
                                self.advance_vars.push(cur.to_string());
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        fn expr(&mut self, expr: &Expr) {
            if let ExprKind::If { branches, .. } = &expr.kind {
                for (cond, then_b) in branches {
                    let Some(cond_e) = cond_expr(cond) else {
                        continue;
                    };
                    let slot = match single_cmp(cond_e) {
                        Some((lhs, CmpOp::Eq, rhs))
                            if matches!(
                                walk::unparen(rhs).kind,
                                ExprKind::Int(0)
                                    | ExprKind::None
                                    | ExprKind::Unary { op: UnOp::Neg, .. }
                            ) =>
                        {
                            trie_child_slot(lhs)
                        }
                        _ => None,
                    };
                    if let Some(slot) = slot {
                        let writes_slot = then_b.stmts.iter().any(|s| {
                            matches!(
                                &s.kind,
                                StmtKind::Assign { target, op: AssignOp::Set, .. }
                                    if trie_child_slot(target) == Some(slot.clone())
                            )
                        });
                        if writes_slot {
                            self.alloc_slots.push(slot);
                        }
                    }
                }
            }
        }
    }
    let mut c = Check {
        alloc_slots: Vec::new(),
        advance_vars: Vec::new(),
    };
    walk_block(&mut c, body);
    c.alloc_slots
        .iter()
        .any(|(_, cur)| c.advance_vars.contains(cur))
}

/// Flags hand-rolled union-find (L0517), prime/SPF sieve (L0518), and
/// binary-lifting tables (L0519) that duplicate `std.dsu`, `std.math`, and
/// `std.tree`.
fn hand_rolled_stdlib_algos(file: &File) -> Vec<Finding> {
    struct Scan {
        base: u32,
        findings: Vec<Finding>,
    }

    impl Scan {
        fn warn(&mut self, lint: &'static crate::Lint, span: Span, msg: &str, help: &str) {
            self.findings.push(Finding {
                lint,
                diag: Diagnostic::warning(lint.code, span.shift(self.base), msg).with_help(help),
            });
        }
    }

    impl Visitor for Scan {
        fn stmt(&mut self, stmt: &Stmt) {
            match &stmt.kind {
                StmtKind::While { cond, body } if is_dsu_path_halving_loop(cond, body) => {
                    self.warn(
                        &HAND_ROLLED_DSU,
                        stmt.span,
                        "hand-rolled union-find (DSU) path compression",
                        "use `Dsu`, `DsuOf`, `ParityDsu`, or `PotentialDsu` from `std.dsu`",
                    );
                }
                StmtKind::Fn(decl) => {
                    if let FnBody::Block(body) = &decl.body {
                        if is_dsu_recursive_or_two_pass_fn(decl, body) {
                            self.warn(
                                &HAND_ROLLED_DSU,
                                stmt.span,
                                "hand-rolled union-find (DSU) `find` function",
                                "use `Dsu`, `DsuOf`, `ParityDsu`, or `PotentialDsu` from `std.dsu`",
                            );
                        }
                    }
                }
                StmtKind::For { pat, body, .. } => {
                    if let PatternKind::Name(var) = &pat.kind {
                        if is_sieve_loop(var, body) {
                            self.warn(
                                &HAND_ROLLED_SIEVE,
                                stmt.span,
                                "hand-rolled prime/SPF sieve loop",
                                "use `Sieve` or `sieve_multiplicative` from `std.math`",
                            );
                        }
                        if is_binary_lifting_loop(var, body) {
                            self.warn(
                                &HAND_ROLLED_BINARY_LIFTING,
                                stmt.span,
                                "hand-rolled binary lifting table",
                                "use `Rooted.lca` / `Rooted.kth_ancestor` or `JumpTree` from `std.tree`",
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

/// Unwraps a 1-D index `arr[ix]` and returns `(arr_name, ix)`.
fn plain_index(e: &Expr) -> Option<(&str, &Expr)> {
    let ExprKind::Index { base, indices } = &walk::unparen(e).kind else {
        return None;
    };
    let [ix] = indices.as_slice() else {
        return None;
    };
    Some((expr_name(base)?, ix))
}

/// Matches `arr[x] != x` (or `x != arr[x]`), returning `(arr, x)`.
fn dsu_not_root_cond(cond: &Cond) -> Option<(&str, &str)> {
    let e = cond_expr(cond)?;
    let (lhs, CmpOp::Ne, rhs) = single_cmp(e)? else {
        return None;
    };
    if let (Some((arr, ix)), Some(v)) = (plain_index(lhs), expr_name(rhs)) {
        if expr_name(ix) == Some(v) {
            return Some((arr, v));
        }
    }
    if let (Some(v), Some((arr, ix))) = (expr_name(lhs), plain_index(rhs)) {
        if expr_name(ix) == Some(v) {
            return Some((arr, v));
        }
    }
    None
}

/// `while up[x] != x:` containing `up[x] = up[up[x]]` (path halving).
fn is_dsu_path_halving_loop(cond: &Cond, body: &Block) -> bool {
    let Some((arr, var)) = dsu_not_root_cond(cond) else {
        return false;
    };
    struct Check<'a> {
        arr: &'a str,
        var: &'a str,
        halved: bool,
    }
    impl Visitor for Check<'_> {
        fn stmt(&mut self, stmt: &Stmt) {
            if let StmtKind::Assign {
                target,
                op: AssignOp::Set,
                value,
            } = &stmt.kind
            {
                if let Some((a1, ix1)) = plain_index(target) {
                    if a1 == self.arr && expr_name(ix1) == Some(self.var) {
                        if let Some((a2, ix2)) = plain_index(value) {
                            if a2 == self.arr {
                                if let Some((a3, ix3)) = plain_index(ix2) {
                                    if a3 == self.arr && expr_name(ix3) == Some(self.var) {
                                        self.halved = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    let mut c = Check {
        arr,
        var,
        halved: false,
    };
    walk_block(&mut c, body);
    c.halved
}

/// Recursive path-compressing DSU `find(x)` (`if up[x] == x: ...; up[x] = find(up[x])`)
/// or two-pass iterative DSU `find(v)` (chase `up[r]` to root, then compress `up[x] = r`).
fn is_dsu_recursive_or_two_pass_fn(decl: &FnDecl, body: &Block) -> bool {
    if decl.params.len() != 1 {
        return false;
    }
    let fn_name = decl.name.text.as_str();
    let param = decl.params[0].name.text.as_str();

    // Case 1: recursive `fn find(x)`: calls `find(up[x])` and writes `up[x] = ...`.
    struct Rec<'a> {
        fn_name: &'a str,
        param: &'a str,
        rec_arr: Option<String>,
        writes_arr: Vec<String>,
    }
    impl Visitor for Rec<'_> {
        fn expr(&mut self, expr: &Expr) {
            if let ExprKind::Call { callee, args } = &expr.kind {
                if expr_name(callee) == Some(self.fn_name) && args.len() == 1 {
                    if let Some((arr, ix)) = plain_index(&args[0].value) {
                        if expr_name(ix) == Some(self.param) {
                            self.rec_arr = Some(arr.to_string());
                        }
                    }
                }
            }
        }
        fn stmt(&mut self, stmt: &Stmt) {
            if let StmtKind::Assign {
                target,
                op: AssignOp::Set,
                ..
            } = &stmt.kind
            {
                if let Some((arr, ix)) = plain_index(target) {
                    if expr_name(ix) == Some(self.param) {
                        self.writes_arr.push(arr.to_string());
                    }
                }
            }
        }
    }
    let mut r = Rec {
        fn_name,
        param,
        rec_arr: None,
        writes_arr: Vec::new(),
    };
    walk_block(&mut r, body);
    if let Some(arr) = r.rec_arr {
        if r.writes_arr.iter().any(|w| w == &arr) {
            return true;
        }
    }

    // Case 2: two-pass iterative `find(v)`: two `while` loops over the same parent array
    // where the second writes `up[x] = r`.
    let mut chase_arrs: Vec<String> = Vec::new();
    let mut compress_arrs: Vec<String> = Vec::new();
    for stmt in &body.stmts {
        if let StmtKind::While { cond, body: wb } = &stmt.kind {
            let cond_arr = if let Some((arr, _)) = dsu_not_root_cond(cond) {
                Some(arr.to_string())
            } else if let Cond::Let { value, .. } = cond {
                plain_index(value).map(|(arr, _)| arr.to_string())
            } else {
                None
            };
            if let Some(arr) = cond_arr {
                let writes_arr = wb.stmts.iter().any(|s| {
                    matches!(
                        &s.kind,
                        StmtKind::Assign { target, op: AssignOp::Set, .. }
                            if plain_index(target).map(|(a, _)| a) == Some(arr.as_str())
                    )
                });
                if writes_arr {
                    compress_arrs.push(arr);
                } else {
                    chase_arrs.push(arr);
                }
            }
        }
    }
    compress_arrs.iter().any(|a| chase_arrs.contains(a))
}

/// Matches a prime/SPF sieve outer loop `for i in ...:`:
/// - Eratosthenes: initializes `j = i * i` (or `2 * i`), increments `j += i`,
///   and writes `arr[j] = false` / `arr[D(j)] = false` / `arr[j] = i` / `arr[j] = 0`.
/// - Linear (Euler) sieve: inner `for p in primes:` writing `arr[i * p] = p` (or `false`).
fn is_sieve_loop(outer_var: &str, body: &Block) -> bool {
    struct Check<'a> {
        outer: &'a str,
        multiple_vars: Vec<String>,
        stepped_vars: Vec<String>,
        marked_vars: Vec<String>,
        linear_sieve: bool,
    }
    fn is_mul_of_var(e: &Expr, var: &str) -> bool {
        let ExprKind::Binary {
            op: BinOp::Mul,
            lhs,
            rhs,
            ..
        } = &walk::unparen(e).kind
        else {
            return false;
        };
        (expr_name(lhs) == Some(var) && (expr_name(rhs) == Some(var) || is_int_lit(rhs, 2)))
            || (expr_name(rhs) == Some(var) && is_int_lit(lhs, 2))
    }
    fn unwrap_domain_wrap(e: &Expr) -> &Expr {
        let u = walk::unparen(e);
        if let ExprKind::Call { args, .. } = &u.kind {
            if args.len() == 1 {
                return walk::unparen(&args[0].value);
            }
        }
        u
    }
    impl Visitor for Check<'_> {
        fn stmt(&mut self, stmt: &Stmt) {
            match &stmt.kind {
                StmtKind::Let {
                    pat: Pattern {
                        kind: PatternKind::Name(v),
                        ..
                    },
                    init: Some(init),
                    ..
                } if is_mul_of_var(init, self.outer) => {
                    self.multiple_vars.push(v.clone());
                }
                StmtKind::Assign {
                    target,
                    op: AssignOp::Binary(BinOp::Add),
                    value,
                } => {
                    if let Some(v) = expr_name(target) {
                        if expr_name(value) == Some(self.outer) {
                            self.stepped_vars.push(v.to_string());
                        }
                    }
                }
                StmtKind::Assign {
                    target,
                    op: AssignOp::Set,
                    value,
                } => {
                    if let Some((_, ix)) = plain_index(target) {
                        let inner = unwrap_domain_wrap(ix);
                        if let Some(v) = expr_name(inner) {
                            let marks = matches!(
                                &walk::unparen(value).kind,
                                ExprKind::Bool(false) | ExprKind::Int(0)
                            ) || expr_name(value) == Some(self.outer);
                            if marks {
                                self.marked_vars.push(v.to_string());
                            }
                        }
                    }
                }
                StmtKind::For {
                    pat: Pattern {
                        kind: PatternKind::Name(p),
                        ..
                    },
                    body: inner_body,
                    ..
                } => {
                    for s in &inner_body.stmts {
                        if let StmtKind::Assign {
                            target,
                            op: AssignOp::Set,
                            value,
                        } = &s.kind
                        {
                            if let Some((_, ix)) = plain_index(target) {
                                let inner = unwrap_domain_wrap(ix);
                                if let ExprKind::Binary {
                                    op: BinOp::Mul,
                                    lhs,
                                    rhs,
                                    ..
                                } = &inner.kind
                                {
                                    let is_ip = (expr_name(lhs) == Some(self.outer)
                                        && expr_name(rhs) == Some(p.as_str()))
                                        || (expr_name(rhs) == Some(self.outer)
                                            && expr_name(lhs) == Some(p.as_str()));
                                    let writes_p = expr_name(value) == Some(p.as_str())
                                        || matches!(
                                            walk::unparen(value).kind,
                                            ExprKind::Bool(false)
                                        );
                                    if is_ip && writes_p {
                                        self.linear_sieve = true;
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    let mut c = Check {
        outer: outer_var,
        multiple_vars: Vec::new(),
        stepped_vars: Vec::new(),
        marked_vars: Vec::new(),
        linear_sieve: false,
    };
    walk_block(&mut c, body);
    c.linear_sieve
        || c.multiple_vars
            .iter()
            .any(|v| c.stepped_vars.contains(v) && c.marked_vars.contains(v))
}

/// Unwraps a 2-D index written either `tbl[a, b]` or `tbl[a][b]` into `(tbl, a, b)`.
fn index_2d(e: &Expr) -> Option<(&str, &Expr, &Expr)> {
    let ExprKind::Index { base, indices } = &walk::unparen(e).kind else {
        return None;
    };
    match indices.as_slice() {
        [a, b] => Some((expr_name(base)?, a, b)),
        [b] => {
            let (tbl, a) = plain_index(base)?;
            Some((tbl, a, b))
        }
        _ => None,
    }
}

/// Matches a binary lifting recurrence inside `for k in 1..lg:`:
/// `up[k, v] = up[k - 1, up[k - 1, v]]` or `up[k][v] = up[k - 1][up[k - 1][v]]`
/// (in either `(k, v)` or `(v, k)` index order).
fn is_binary_lifting_loop(level_var: &str, body: &Block) -> bool {
    struct Check<'a> {
        k: &'a str,
        found: bool,
    }
    impl Visitor for Check<'_> {
        fn stmt(&mut self, stmt: &Stmt) {
            if self.found {
                return;
            }
            let StmtKind::Assign {
                target,
                op: AssignOp::Set,
                value,
            } = &stmt.kind
            else {
                return;
            };
            let Some((tbl, la, lb)) = index_2d(target) else {
                return;
            };
            let Some((rtbl, ra, rb)) = index_2d(value) else {
                return;
            };
            if tbl != rtbl {
                return;
            }
            // Orientation 1: `up[k, v] = up[k - 1, up[k - 1, v]]`
            if expr_name(la) == Some(self.k) && is_var_minus_one(ra, self.k) {
                if let Some((itbl, ia, ib)) = index_2d(rb) {
                    if itbl == tbl && is_var_minus_one(ia, self.k) && expr_name(ib) == expr_name(lb)
                    {
                        self.found = true;
                        return;
                    }
                }
            }
            // Orientation 2: `up[v, k] = up[up[v, k - 1], k - 1]`
            if expr_name(lb) == Some(self.k) && is_var_minus_one(rb, self.k) {
                if let Some((itbl, ia, ib)) = index_2d(ra) {
                    if itbl == tbl && is_var_minus_one(ib, self.k) && expr_name(ia) == expr_name(la)
                    {
                        self.found = true;
                    }
                }
            }
        }
    }
    let mut c = Check {
        k: level_var,
        found: false,
    };
    walk_block(&mut c, body);
    c.found
}

/// Bottom-up segment-tree build/pull loop: `for i in (1..n).rev:` writing
/// `t[i] = ... t[2 * i] ... t[2 * i + 1]` on the same array `t`.
fn is_segtree_build_pull_loop(var: &str, body: &Block) -> bool {
    struct Check<'a> {
        var: &'a str,
        found: bool,
    }
    fn rhs_reads_both_children(e: &Expr, arr: &str, var: &str) -> bool {
        struct Reads<'a> {
            arr: &'a str,
            var: &'a str,
            left: bool,
            right: bool,
        }
        impl Visitor for Reads<'_> {
            fn expr(&mut self, expr: &Expr) {
                if let Some((a, ix)) = plain_index(expr) {
                    if a == self.arr {
                        if is_double(ix, self.var) {
                            self.left = true;
                        }
                        if is_double_plus_one(ix, self.var) {
                            self.right = true;
                        }
                    }
                }
            }
        }
        let mut r = Reads {
            arr,
            var,
            left: false,
            right: false,
        };
        walk::walk_expr(&mut r, e);
        r.left && r.right
    }
    impl Visitor for Check<'_> {
        fn stmt(&mut self, stmt: &Stmt) {
            if self.found {
                return;
            }
            if let StmtKind::Assign {
                target,
                op: AssignOp::Set,
                value,
            } = &stmt.kind
            {
                if let Some((arr, ix)) = plain_index(target) {
                    if expr_name(ix) == Some(self.var)
                        && rhs_reads_both_children(value, arr, self.var)
                    {
                        self.found = true;
                    }
                }
            }
        }
    }
    let mut c = Check { var, found: false };
    walk_block(&mut c, body);
    c.found
}
