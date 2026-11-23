//! C++ spellings that CP Lang names: a bit test (`manual_bit_test`), a
//! descending sort by a negated key (`sort_desc_by_negation`), the last
//! element by `len - 1` (`last_via_len`), the `i < j` pair loop
//! (`manual_pair_loop`), the `% 10` / `/= 10` digit loop
//! (`manual_digit_loop`), and a hand-written fast power or Euclid
//! (`hand_rolled_pow`, `hand_rolled_gcd`).

use crate::walk::{unparen, walk_expr, walk_stmt, Visitor};
use crate::{
    File, Finding, Lint, HAND_ROLLED_GCD, HAND_ROLLED_POW, LAST_VIA_LEN, MANUAL_BIT_TEST,
    MANUAL_DIGIT_LOOP, MANUAL_PAIR_LOOP, SORT_DESC_BY_NEGATION,
};
use cpl_diag::{Diagnostic, Fix, Span};
use cpl_syntax::ast::*;

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let mut v = Spelled {
        src: file.src,
        base: 0,
        targets: Vec::new(),
        bit_fns: Vec::new(),
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}

struct Spelled<'a> {
    src: &'a str,
    base: u32,
    /// Places written by an assignment, which `.last` cannot replace.
    targets: Vec<Span>,
    /// The bodies of functions named `bit`, which define the bit test.
    bit_fns: Vec<Span>,
    out: Vec<Finding>,
}

impl Spelled<'_> {
    fn text(&self, e: &Expr) -> &str {
        e.span.shift(self.base).text(self.src)
    }

    /// `label` is the place whose type `typed.rs` checks.
    fn push(
        &mut self,
        lint: &'static Lint,
        span: Span,
        (msg, help): (String, String),
        label: Option<(Span, &str)>,
        fix: Option<(Span, String)>,
    ) {
        let mut diag = Diagnostic::warning(lint.code, span.shift(self.base), msg).with_help(help);
        if let Some((at, what)) = label {
            diag = diag.with_label(at.shift(self.base), what);
        }
        if let Some((at, with)) = fix {
            diag.fixes.push(Fix {
                title: format!("use `{with}`"),
                edits: vec![(at.shift(self.base), with)],
            });
        }
        self.out.push(Finding { lint, diag });
    }

    /// `x` as a method receiver: bare when it is a place or a call.
    fn receiver(&self, e: &Expr) -> String {
        let text = self.text(e);
        match e.kind {
            ExprKind::Name(_)
            | ExprKind::Field { .. }
            | ExprKind::Index { .. }
            | ExprKind::Call { .. }
            | ExprKind::Paren(_)
            | ExprKind::SelfValue => text.to_string(),
            _ => format!("({text})"),
        }
    }

    fn bit_test(&mut self, expr: &Expr) {
        let ExprKind::Compare { first, rest } = &expr.kind else {
            return;
        };
        let [(op @ (CmpOp::Eq | CmpOp::Ne), rhs)] = rest.as_slice() else {
            return;
        };
        let ExprKind::Int(want) = rhs.kind else {
            return;
        };
        let ExprKind::Binary {
            op: BinOp::BitAnd,
            lhs,
            rhs: mask,
            ..
        } = &unparen(first).kind
        else {
            return;
        };
        let is_one = |e: &Expr| matches!(unparen(e).kind, ExprKind::Int(1));
        let by_shift = if is_one(mask) {
            shifted(lhs)
        } else if is_one(lhs) {
            shifted(mask)
        } else {
            None
        };
        let by_mask = one_shl(mask)
            .map(|k| (&**lhs, k))
            .or_else(|| one_shl(lhs).map(|k| (&**mask, k)));
        let (x, k, set) = match (by_shift, by_mask, want) {
            (Some((x, k)), _, 1) => (x, k, *op == CmpOp::Eq),
            (Some((x, k)), _, 0) | (None, Some((x, k)), 0) => (x, k, *op == CmpOp::Ne),
            _ => return,
        };
        let at = expr.span.shift(self.base);
        if self
            .bit_fns
            .iter()
            .any(|f| f.start <= at.start && at.end <= f.end)
        {
            return;
        }
        let call = format!("{}.bit({})", self.receiver(unparen(x)), self.text(k));
        let fixed = if set { call } else { format!("not {call}") };
        self.push(
            &MANUAL_BIT_TEST,
            expr.span,
            (
                format!("`{}` tests one bit by hand", self.text(expr)),
                format!("write `{fixed}`; `bit` reads bit `i` of an `Int`"),
            ),
            Some((unparen(x).span, "the tested value")),
            Some((expr.span, fixed)),
        );
    }
}

fn shifted(e: &Expr) -> Option<(&Expr, &Expr)> {
    match &unparen(e).kind {
        ExprKind::Binary {
            op: BinOp::Shr,
            lhs,
            rhs,
            ..
        } => Some((lhs, rhs)),
        _ => None,
    }
}

/// `1 << k`, giving `k`.
fn one_shl(e: &Expr) -> Option<&Expr> {
    match &unparen(e).kind {
        ExprKind::Binary {
            op: BinOp::Shl,
            lhs,
            rhs,
            ..
        } if matches!(lhs.kind, ExprKind::Int(1)) => Some(rhs),
        _ => None,
    }
}

fn name_of(e: &Expr) -> Option<&str> {
    match &unparen(e).kind {
        ExprKind::Name(n) => Some(n),
        _ => None,
    }
}

fn pat_name(p: &Pattern) -> Option<&str> {
    match &p.kind {
        PatternKind::Name(n) => Some(n),
        _ => None,
    }
}

/// Every statement and expression under `stmts`, nested ones included.
struct Each<S: FnMut(&Stmt), E: FnMut(&Expr)> {
    stmt: S,
    expr: E,
}

impl<S: FnMut(&Stmt), E: FnMut(&Expr)> Visitor for Each<S, E> {
    fn stmt(&mut self, stmt: &Stmt) {
        (self.stmt)(stmt)
    }
    fn expr(&mut self, expr: &Expr) {
        (self.expr)(expr)
    }
}

fn each(stmts: &[Stmt], stmt: impl FnMut(&Stmt), expr: impl FnMut(&Expr)) {
    let mut v = Each { stmt, expr };
    for s in stmts {
        walk_stmt(&mut v, s);
    }
}

fn body_stmts(decl: &FnDecl) -> Vec<Stmt> {
    match &decl.body {
        FnBody::Block(block) => block.stmts.clone(),
        FnBody::Expr(e) => vec![Stmt {
            kind: StmtKind::Expr(e.clone()),
            span: e.span,
            decorators: Vec::new(),
            public: false,
        }],
        FnBody::None => Vec::new(),
    }
}

/// `x /= 2`, `x >>= 1`, `x = x / 2` or `x = x >> 1`.
fn halves(stmt: &Stmt, x: &str) -> bool {
    let StmtKind::Assign { target, op, value } = &stmt.kind else {
        return false;
    };
    if name_of(target) != Some(x) {
        return false;
    }
    let by = |op: BinOp, v: &Expr| {
        matches!(
            (op, &v.kind),
            (BinOp::Div, ExprKind::Int(2)) | (BinOp::Shr, ExprKind::Int(1))
        )
    };
    match (op, &unparen(value).kind) {
        (AssignOp::Binary(op), _) => by(*op, value),
        (AssignOp::Set, ExprKind::Binary { op, lhs, rhs, .. }) => {
            name_of(lhs) == Some(x) && by(*op, rhs)
        }
        _ => false,
    }
}

/// `b = b * b % m`, `b = b * b` or `b *= b`.
fn squares(stmt: &Stmt) -> bool {
    let StmtKind::Assign { target, op, value } = &stmt.kind else {
        return false;
    };
    let Some(b) = name_of(target) else {
        return false;
    };
    if matches!(op, AssignOp::Binary(BinOp::Mul)) {
        return name_of(value) == Some(b);
    }
    let mut found = false;
    let mut v = Each {
        stmt: |_: &Stmt| {},
        expr: |e: &Expr| {
            if let ExprKind::Binary {
                op: BinOp::Mul,
                lhs,
                rhs,
                ..
            } = &e.kind
            {
                found |= name_of(lhs) == Some(b) && name_of(rhs) == Some(b);
            }
        },
    };
    walk_expr(&mut v, value);
    matches!(op, AssignOp::Set) && found
}

impl Spelled<'_> {
    fn sort_desc(&mut self, expr: &Expr) {
        let ExprKind::Call { callee, args } = &expr.kind else {
            return;
        };
        let ExprKind::Field { name: method, .. } = &callee.kind else {
            return;
        };
        if method.text != "sort" && method.text != "sorted"
            || args
                .iter()
                .any(|a| a.name.as_ref().is_some_and(|n| n.text == "desc"))
        {
            return;
        }
        for arg in args {
            let Some(name) = arg.name.as_ref().filter(|n| n.text == "by") else {
                continue;
            };
            let ExprKind::Lambda { params, body, .. } = &arg.value.kind else {
                continue;
            };
            let [param] = params.as_slice() else {
                continue;
            };
            let ExprKind::Unary {
                op: UnOp::Neg,
                expr: negated,
            } = &unparen(body).kind
            else {
                continue;
            };
            let plain = pat_name(param).is_some() && name_of(negated) == pat_name(param);
            let span = Span::new(name.span.start, arg.value.span.end);
            let key = format!("{} => {}", self.text_pat(param), self.text(negated));
            let (help, fix) = if plain {
                (
                    "write `desc: true` instead of negating the key".to_string(),
                    Some((span, "desc: true".to_string())),
                )
            } else {
                (format!("write `by: {key}, desc: true`; negating the key overflows on the smallest value"), None)
            };
            self.push(
                &SORT_DESC_BY_NEGATION,
                span,
                (
                    format!("`.{}` sorts descending by negating the key", method.text),
                    help,
                ),
                None,
                fix,
            );
        }
    }

    fn text_pat(&self, p: &Pattern) -> &str {
        p.span.shift(self.base).text(self.src)
    }

    fn last_via_len(&mut self, expr: &Expr) {
        let ExprKind::Index { base, indices } = &expr.kind else {
            return;
        };
        let [i] = indices.as_slice() else {
            return;
        };
        let ExprKind::Binary {
            op: BinOp::Sub,
            lhs,
            rhs,
            ..
        } = &unparen(i).kind
        else {
            return;
        };
        let ExprKind::Field { base: of, name } = &unparen(lhs).kind else {
            return;
        };
        if name.text != "len"
            || !matches!(rhs.kind, ExprKind::Int(1))
            || self.text(of) != self.text(base)
            || self.targets.contains(&expr.span.shift(self.base))
        {
            return;
        }
        let fixed = format!("{}.last!", self.receiver(base));
        self.push(
            &LAST_VIA_LEN,
            expr.span,
            (
                format!("`{}` reads the last element by its index", self.text(expr)),
                format!("write `{fixed}`; `.last` is the last element, and `.last(k)` the one `k` before it"),
            ),
            Some((base.span, "the list")),
            Some((expr.span, fixed)),
        );
    }

    fn pair_loop(&mut self, outer: &Stmt) {
        let StmtKind::For {
            pat, iter, body, ..
        } = &outer.kind
        else {
            return;
        };
        let (Some(i), ExprKind::Range { hi: Some(hi), .. }) = (pat_name(pat), &iter.kind) else {
            return;
        };
        for inner in &body.stmts {
            let StmtKind::For {
                pat: jpat,
                iter: jiter,
                ..
            } = &inner.kind
            else {
                continue;
            };
            let ExprKind::Range {
                lo: Some(lo),
                hi: Some(jhi),
                ..
            } = &jiter.kind
            else {
                continue;
            };
            let ExprKind::Binary {
                op: BinOp::Add,
                lhs,
                rhs,
                ..
            } = &unparen(lo).kind
            else {
                continue;
            };
            if name_of(lhs) != Some(i)
                || !matches!(rhs.kind, ExprKind::Int(1))
                || self.text(jhi) != self.text(hi)
                || pat_name(jpat).is_none()
            {
                continue;
            }
            let StmtKind::For { body: jbody, .. } = &inner.kind else {
                continue;
            };
            let j = self.text_pat(jpat);
            let Some(seq) = self.indexes_only(&body.stmts, inner, &jbody.stmts, [i, j]) else {
                continue;
            };
            self.push(
                &MANUAL_PAIR_LOOP,
                Span::new(inner.span.start, jiter.span.end),
                (
                    format!("`for {j} in {}` walks each pair `{i} < {j}` by hand", self.text(jiter)),
                    format!("write `for (x, y) in {seq}.pairs()`; the body reads `{seq}[{i}]` and `{seq}[{j}]` only"),
                ),
                None,
                None,
            );
        }
    }

    /// The one sequence `outer` (less `inner`'s header) and `inner_body`
    /// index by `names`, when they use the names for nothing else.
    fn indexes_only(
        &self,
        outer: &[Stmt],
        inner: &Stmt,
        inner_body: &[Stmt],
        names: [&str; 2],
    ) -> Option<String> {
        let (mut uses, mut indexed, mut seqs) = (Vec::new(), Vec::new(), Vec::new());
        let mut visit = |e: &Expr| match &e.kind {
            ExprKind::Name(n) if names.contains(&n.as_str()) => uses.push(e.span),
            ExprKind::Index { base, indices } => {
                if let [idx] = indices.as_slice() {
                    if name_of(idx).is_some_and(|n| names.contains(&n)) {
                        indexed.push(idx.span);
                        seqs.push(base.span.shift(self.base).text(self.src).to_string());
                    }
                }
            }
            _ => {}
        };
        for s in outer {
            if std::ptr::eq(s, inner) {
                each(inner_body, |_| {}, &mut visit);
            } else {
                each(std::slice::from_ref(s), |_| {}, &mut visit);
            }
        }
        seqs.dedup();
        let only = uses
            .iter()
            .all(|u| indexed.iter().any(|x| x.start <= u.start && u.end <= x.end));
        match seqs.as_slice() {
            [seq] if only => Some(seq.clone()),
            _ => None,
        }
    }

    fn digit_loop(&mut self, stmt: &Stmt) {
        let StmtKind::While {
            cond: Cond::Expr(cond),
            body,
            ..
        } = &stmt.kind
        else {
            return;
        };
        let ExprKind::Compare { first, rest } = &cond.kind else {
            return;
        };
        let (Some(y), [(CmpOp::Gt | CmpOp::Ne, zero)]) = (name_of(first), rest.as_slice()) else {
            return;
        };
        if !matches!(zero.kind, ExprKind::Int(0)) {
            return;
        }
        let base = body.stmts.iter().find_map(|s| {
            let StmtKind::Assign { target, op, value } = &s.kind else {
                return None;
            };
            if name_of(target) != Some(y) {
                return None;
            }
            let b = match (op, &unparen(value).kind) {
                (AssignOp::Binary(BinOp::Div), ExprKind::Int(b)) => *b,
                (
                    AssignOp::Set,
                    ExprKind::Binary {
                        op: BinOp::Div,
                        lhs,
                        rhs,
                        ..
                    },
                ) if name_of(lhs) == Some(y) => match rhs.kind {
                    ExprKind::Int(b) => b,
                    _ => return None,
                },
                _ => return None,
            };
            (3..=36).contains(&b).then_some(b)
        });
        let Some(b) = base else {
            return;
        };
        let call = if b == 10 {
            format!("{y}.digits()")
        } else {
            format!("{y}.digits(base: {b})")
        };
        self.push(
            &MANUAL_DIGIT_LOOP,
            Span::new(stmt.span.start, cond.span.end),
            (
                format!("this loop takes `{y}` apart digit by digit"),
                format!("write `for d in {call}` (lowest digit first), or `{call}.sum()` for the digit sum"),
            ),
            None,
            None,
        );
    }

    fn hand_rolled(&mut self, decl: &FnDecl) {
        let stmts = body_stmts(decl);
        let mut pow = false;
        each(
            &stmts,
            |s| {
                if let StmtKind::While {
                    cond: Cond::Expr(cond),
                    body,
                    ..
                } = &s.kind
                {
                    let x = match &cond.kind {
                        ExprKind::Compare { first, .. } => name_of(first),
                        _ => None,
                    };
                    pow |= x.is_some_and(|x| body.stmts.iter().any(|s| halves(s, x)))
                        && body.stmts.iter().any(squares);
                }
            },
            |_| {},
        );
        if pow {
            self.push(
                &HAND_ROLLED_POW,
                decl.name.span,
                (
                    format!("`{}` is fast exponentiation by squaring, written by hand", decl.name.text),
                    "call `pow_mod(b, e, m)` from `std.math`; with a fixed modulus, `Mod<P>` makes it `b ** e`".to_string(),
                ),
                None,
                None,
            );
        }
        let params: Vec<&str> = decl.params.iter().map(|p| p.name.text.as_str()).collect();
        let [a, b] = params.as_slice() else {
            return;
        };
        let mut euclid = false;
        each(
            &stmts,
            |_| {},
            |e| {
                let ExprKind::Call { callee, args } = &e.kind else {
                    return;
                };
                let [first, second] = args.as_slice() else {
                    return;
                };
                let rem = match &unparen(&second.value).kind {
                    ExprKind::Binary {
                        op: BinOp::Rem,
                        lhs,
                        rhs,
                        ..
                    } => name_of(lhs) == Some(a) && name_of(rhs) == Some(b),
                    _ => false,
                };
                euclid |= name_of(callee) == Some(&decl.name.text)
                    && name_of(&first.value) == Some(b)
                    && rem;
            },
        );
        if euclid {
            self.push(
                &HAND_ROLLED_GCD,
                decl.name.span,
                (
                    format!("`{}` is Euclid's algorithm, written by hand", decl.name.text),
                    format!("call `gcd({a}, {b})` from `std.math`; a fold over a list is `xs.fold(gcd, 0)`"),
                ),
                None,
                None,
            );
        }
    }
}

impl Visitor for Spelled<'_> {
    fn stmt(&mut self, stmt: &Stmt) {
        if let StmtKind::Assign { target, .. } = &stmt.kind {
            self.targets.push(target.span.shift(self.base));
        }
        if let StmtKind::Fn(decl) = &stmt.kind {
            if decl.name.text == "bit" {
                self.bit_fns.push(stmt.span.shift(self.base));
            }
            self.hand_rolled(decl);
        }
        self.pair_loop(stmt);
        self.digit_loop(stmt);
    }

    fn expr(&mut self, expr: &Expr) {
        self.bit_test(expr);
        self.sort_desc(expr);
        self.last_via_len(expr);
    }
}
