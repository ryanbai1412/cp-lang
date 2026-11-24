//! Graph and window algorithms written out by hand: a three-colour DFS for
//! a directed cycle (`cycle_by_colors`), a monotone deque for each window's
//! minimum or maximum (`sliding_min_by_hand`), and depths stepped from the
//! parent's (`depth_by_hand`).

use crate::std_fn;
use crate::walk::{unparen, Visitor};
use crate::{File, Finding, CYCLE_BY_COLORS, DEPTH_BY_HAND, SLIDING_MIN_BY_HAND};
use cpl_diag::Span;
use cpl_syntax::ast::*;

use super::shapes::{compare, index1, int, name, squash};
use super::Warn;
use cpl_diag::msgs::lint_rules;

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let mut v = HandGraph {
        src: file.src,
        base: 0,
        structs: file
            .top_level()
            .filter_map(|(base, s)| match &s.kind {
                StmtKind::Struct(decl) => Some((
                    s.span.shift(base),
                    decl.fields.iter().map(|f| f.name.text.clone()).collect(),
                )),
                _ => None,
            })
            .collect(),
        compared: Vec::new(),
        depths: Vec::new(),
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.base = 0;
    for (t, at) in std::mem::take(&mut v.depths) {
        if v.compared.contains(&t) {
            v.warn_at(
                &DEPTH_BY_HAND,
                at,
&lint_rules::DEPTH_BY_HAND, &[&t],
                "`Rooted.from_parents(par)` (`par: [V]?V`, `None` at the root) or `Rooted.new(adj, root)` knows `t.depth(v)`, `t.parent(v)` and `t.lca(u, v)`".to_string(),
            );
        }
    }
    v.out
}

struct HandGraph<'a> {
    src: &'a str,
    base: u32,
    /// Each struct and its fields: a depth table that is a struct's own
    /// field (std's `Rooted`, `JumpTree`) is how that type answers depths.
    structs: Vec<(Span, Vec<String>)>,
    /// Tables read at two places in one comparison or difference,
    /// `depth[u] > depth[v]`, `depth[u] - depth[a]`: the depths feed a walk
    /// up the tree or a distance, which `Rooted` answers. Depths read only
    /// as values are one pass by hand, cheaper than building the tree.
    compared: Vec<String>,
    /// Each table stepped from the parent's, and where.
    depths: Vec<(String, Span)>,
    out: Vec<Finding>,
}

/// `t[i] = c` with a name index and an integer: the table, the index, `c`.
fn mark(stmt: &Stmt) -> Option<(&str, &str, i128)> {
    let StmtKind::Assign {
        target,
        op: AssignOp::Set,
        value,
    } = &stmt.kind
    else {
        return None;
    };
    let (t, i) = index1(target)?;
    Some((t, name(i)?, int(value)?))
}

/// `xs.method(args)` as a statement: the receiver, the method, the args.
fn method_call<'e>(stmt: &'e Stmt, method: &str) -> Option<(&'e str, &'e [Arg])> {
    let StmtKind::Expr(e) = &stmt.kind else {
        return None;
    };
    let ExprKind::Call { callee, args } = &e.kind else {
        return None;
    };
    match &callee.kind {
        ExprKind::Field { base, name: m } if m.text == method => Some((name(base)?, args)),
        _ => None,
    }
}

fn conjuncts<'e>(e: &'e Expr, out: &mut Vec<&'e Expr>) {
    match &unparen(e).kind {
        ExprKind::Binary {
            op: BinOp::And,
            lhs,
            rhs,
            ..
        } => {
            conjuncts(lhs, out);
            conjuncts(rhs, out);
        }
        _ => out.push(e),
    }
}

/// The queue a `while` pops, the test it pops under, and whether that test is
/// negated: `while t: q.pop()`, or `while ..:` with `if t: break` then `q.pop()`.
fn pop_test(s: &Stmt) -> Option<(&str, &Expr, bool)> {
    let StmtKind::While {
        cond: Cond::Expr(cond),
        body,
        ..
    } = &s.kind
    else {
        return None;
    };
    match body.stmts.as_slice() {
        [pop] => Some((method_call(pop, std_fn::POP)?.0, cond, false)),
        [stop, pop] => {
            let StmtKind::Expr(e) = &stop.kind else {
                return None;
            };
            let ExprKind::If {
                branches,
                otherwise: None,
            } = &e.kind
            else {
                return None;
            };
            let [(Cond::Expr(test), then)] = branches.as_slice() else {
                return None;
            };
            matches!(then.stmts.as_slice(), [b] if matches!(b.kind, StmtKind::Break(None)))
                .then_some(())?;
            Some((method_call(pop, std_fn::POP)?.0, test, true))
        }
        _ => None,
    }
}

/// The starts of `word` in `text` as a whole identifier.
fn starts_of<'t>(text: &'t str, word: &'t str) -> impl Iterator<Item = usize> + 't {
    text.match_indices(word).map(|(i, _)| i).filter(move |&i| {
        !text[..i]
            .chars()
            .next_back()
            .is_some_and(super::shapes::ident_char)
    })
}

/// Whether `text` has `word` as a whole identifier followed by `next`.
fn has_followed(text: &str, word: &str, next: &str) -> bool {
    starts_of(text, word).any(|i| text[i + word.len()..].trim_start().starts_with(next))
}

/// Whether `text` compares an element of `t` with `== c`.
fn tests_colour(text: &str, t: &str, c: i128) -> bool {
    let c = c.to_string();
    starts_of(text, t).any(|i| {
        let rest = &text[i + t.len()..];
        if !rest.starts_with('[') {
            return false;
        }
        let mut depth = 0;
        for (j, ch) in rest.char_indices() {
            match ch {
                '[' => depth += 1,
                ']' => {
                    depth -= 1;
                    if depth == 0 {
                        let Some(rest) = rest[j + 1..].trim_start().strip_prefix("==") else {
                            return false;
                        };
                        let rest = rest.trim_start();
                        return rest.starts_with(&c)
                            && !rest[c.len()..]
                                .chars()
                                .next()
                                .is_some_and(super::shapes::ident_char);
                    }
                }
                _ => {}
            }
        }
        false
    })
}

impl super::Warn for HandGraph<'_> {
    fn sink(&mut self) -> (&mut Vec<Finding>, u32) {
        (&mut self.out, self.base)
    }
}

impl HandGraph<'_> {
    fn text(&self, span: Span) -> &str {
        span.shift(self.base).text(self.src)
    }

    /// `fn dfs(u): c[u] = 1; ... c[v] == 1 ...; c[u] = 2`.
    fn colours(&mut self, decl: &FnDecl) {
        let FnBody::Block(body) = &decl.body else {
            return;
        };
        let Some(u) = decl.params.first().map(|p| p.name.text.as_str()) else {
            return;
        };
        let (Some(first), Some(last)) = (body.stmts.first(), body.stmts.last()) else {
            return;
        };
        let (Some((t, i, open)), Some((t2, i2, done))) = (mark(first), mark(last)) else {
            return;
        };
        if t != t2 || i != u || i2 != u || open == done {
            return;
        }
        if !tests_colour(self.text(body.span), t, open) {
            return;
        }
        let f = &decl.name.text;
        self.warn_at(
            &CYCLE_BY_COLORS,
            decl.sig_span,
&lint_rules::CYCLE_BY_COLORS, &[&f, &t, &open, &done],
            "`find_cycle(g)` (`std.graph`) returns one directed cycle or `None`, without recursion: `find_cycle(g) != None` asks whether there is one, and `topo(g)` gives an order when there is none".to_string(),
        );
    }

    /// `for i ..: while .. q[q.len - 1] >= a[i]: q.pop()` (or `if a[q.last!] < a[i]: break`
    /// then `q.pop()`) then `q.push(i)`, with the window's start (`i - k`) tested in the
    /// same loop.
    fn deque(&mut self, stmt: &Stmt) {
        let StmtKind::For { pat, body, .. } = &stmt.kind else {
            return;
        };
        let PatternKind::Name(i) = &pat.kind else {
            return;
        };
        let Some((at, (q, test, negated))) =
            (body.stmts.iter().enumerate()).find_map(|(at, s)| Some((at, pop_test(s)?)))
        else {
            return;
        };
        let pushes_i = body
            .stmts
            .get(at + 1)
            .and_then(|s| method_call(s, std_fn::PUSH))
            .is_some_and(|(r, args)| {
                r == q && matches!(args, [a] if name(&a.value) == Some(i.as_str()))
            });
        if !pushes_i {
            return;
        }
        // The test compares the back (or a key of it) with the item pushed (or its key):
        // a turn over two entries, `cross(h.last! - h[h.len - 2], ..)`, is no window's.
        let back = |e: &Expr| {
            let t = squash(self.text(e.span));
            let rest = t
                .replace(&format!("{q}[{q}.len-1]"), "")
                .replace(&format!("{q}.last"), "");
            rest.len() < t.len() && starts_of(&rest, q).next().is_none()
        };
        let item = |e: &Expr| {
            let t = squash(self.text(e.span));
            starts_of(&t, i).next().is_some() && starts_of(&t, q).next().is_none()
        };
        let mut parts = Vec::new();
        match negated {
            false => conjuncts(test, &mut parts),
            true => parts.push(test),
        }
        let Some(keeps_min) = parts.iter().find_map(|c| {
            let (l, op, r) = compare(c)?;
            let drops_larger = match op {
                CmpOp::Ge | CmpOp::Gt => true,
                CmpOp::Le | CmpOp::Lt => false,
                _ => return None,
            };
            if back(l) && item(r) {
                Some(drops_larger != negated)
            } else if back(r) && item(l) {
                Some(drops_larger == negated)
            } else {
                None
            }
        }) else {
            return;
        };
        if !has_followed(self.text(body.span), i, "-") {
            return;
        }
        let m = if keeps_min { std_fn::MIN } else { std_fn::MAX };
        self.warn_at(
            &SLIDING_MIN_BY_HAND,
            stmt.span,
&lint_rules::SLIDING_MIN_BY_HAND, &[&q, &m],
            format!("`var w = Window({m})` (`std.window`) keeps that deque: `w.push(x)` as an item enters, `w.pop()` as the oldest leaves, and `w.all!` is the window's {m}"),
        );
    }

    /// `d[v] = d[p] + 1` or `d.push(d[p] + 1)` on a table named for depth.
    fn depth(&mut self, stmt: &Stmt) {
        let (t, value) = match &stmt.kind {
            StmtKind::Assign {
                target,
                op: AssignOp::Set,
                value,
            } => match index1(target) {
                Some((t, _)) => (t, value),
                None => return,
            },
            _ => match method_call(stmt, std_fn::PUSH) {
                Some((t, [a])) => (t, &a.value),
                _ => return,
            },
        };
        let lower = t.to_ascii_lowercase();
        if !["dep", std_fn::LEVEL, "lvl"]
            .iter()
            .any(|w| lower.contains(w))
        {
            return;
        }
        let ExprKind::Binary {
            op: BinOp::Add,
            lhs,
            rhs,
            ..
        } = &unparen(value).kind
        else {
            return;
        };
        if int(rhs) != Some(1) || index1(lhs).map(|(b, _)| b) != Some(t) {
            return;
        }
        let at = stmt.span.shift(self.base);
        if self.structs.iter().any(|(s, fields)| {
            s.start <= at.start && at.end <= s.end && fields.iter().any(|f| f == t)
        }) {
            return;
        }
        self.depths.push((t.to_string(), at));
    }
}

impl Visitor for HandGraph<'_> {
    fn stmt(&mut self, stmt: &Stmt) {
        if let StmtKind::Fn(decl) = &stmt.kind {
            self.colours(decl);
        }
        self.deque(stmt);
        self.depth(stmt);
    }
    fn expr(&mut self, expr: &Expr) {
        let pair = match &unparen(expr).kind {
            ExprKind::Binary {
                op: BinOp::Sub,
                lhs,
                rhs,
                ..
            } => Some((&**lhs, &**rhs)),
            _ => compare(expr).map(|(a, _, b)| (a, b)),
        };
        if let Some((a, b)) = pair {
            if let (Some((x, _)), Some((y, _))) = (index1(a), index1(b)) {
                if x == y {
                    self.compared.push(x.to_string());
                }
            }
        }
    }
}
