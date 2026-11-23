//! `manual_adjacency_list` (L0537): C++'s `vector<vector<int>> adj(n + 1)`
//! filled by `adj[u].push_back(v)` as each edge is read. `std.graph`'s
//! `Graph.undirected(es)` / `Graph.directed(es)` build it from the edge list
//! read in one go, over a vertex domain.

use crate::walk::{unparen, walk_pattern, walk_stmt, Bind, Visitor};
use crate::{File, Finding, MANUAL_ADJACENCY_LIST};
use cpl_diag::{Diagnostic, Span};
use cpl_syntax::ast::{BinOp, Expr, ExprKind, PatternKind, Stmt, StmtKind, TypeKind};

use super::lists::is_list_type;

const HELP: &str = "read the edges at once, `let es: [E](u: V, v: V) = read()` over `domain V = 1..=n` and `domain E = 0..m`, then build `Graph.undirected(es)` or `Graph.directed(es)` (`std.graph`): one compact block in input order, and every vertex index is proven";

pub(crate) fn check(file: &File) -> Vec<Finding> {
    struct Lists {
        base: u32,
        empty: Vec<(String, Span)>,
        pushed_per_edge: Vec<String>,
    }
    impl Visitor for Lists {
        fn stmt(&mut self, stmt: &Stmt) {
            match &stmt.kind {
                StmtKind::Let {
                    mutable: true,
                    pat,
                    ty: Some(ty),
                    init: Some(init),
                    ..
                } => {
                    let PatternKind::Name(name) = &pat.kind else {
                        return;
                    };
                    let lists = match (&ty.kind, &init.kind) {
                        (TypeKind::Named { name, args }, ExprKind::Comprehension { body, .. }) => {
                            name.text == "List"
                                && matches!(&args[..], [a] if is_list_type(a))
                                && is_empty_list(body)
                        }
                        (TypeKind::Array { elem, .. }, _) => {
                            is_list_type(elem) && is_empty_list(init)
                        }
                        _ => false,
                    };
                    if lists {
                        self.empty.push((name.clone(), stmt.span.shift(self.base)));
                    }
                }
                StmtKind::For { body, .. } => {
                    // The names each `let .. = read()` of the body binds: an edge
                    // reads both its ends in one statement.
                    let reads: Vec<Vec<String>> = body
                        .stmts
                        .iter()
                        .filter_map(|s| match &s.kind {
                            StmtKind::Let {
                                pat, init: Some(e), ..
                            } if is_read_call(e) => Some(bound(pat)),
                            _ => None,
                        })
                        .collect();
                    if reads.is_empty() {
                        return;
                    }
                    struct Pushes<'a>(&'a mut Vec<String>, &'a [Vec<String>]);
                    impl Visitor for Pushes<'_> {
                        fn expr(&mut self, expr: &Expr) {
                            let ExprKind::Call { callee, args } = &expr.kind else {
                                return;
                            };
                            let ExprKind::Field { base, name } = &callee.kind else {
                                return;
                            };
                            if name.text != "push" {
                                return;
                            }
                            if let ExprKind::Index { base, indices } = &base.kind {
                                let ends = match (&indices[..], &args[..]) {
                                    ([i], [a]) => vertex(i).zip(pushed_end(&a.value)),
                                    _ => None,
                                };
                                let Some((u, v)) = ends else {
                                    return;
                                };
                                let one_read = |names: &Vec<String>| {
                                    names.iter().any(|n| n == u) && names.iter().any(|n| n == v)
                                };
                                if !self.1.iter().any(one_read) {
                                    return;
                                }
                                if let ExprKind::Name(list) = &base.kind {
                                    self.0.push(list.clone());
                                }
                            }
                        }
                    }
                    for s in &body.stmts {
                        walk_stmt(&mut Pushes(&mut self.pushed_per_edge, &reads), s);
                    }
                }
                _ => {}
            }
        }
    }
    let mut v = Lists {
        base: 0,
        empty: Vec::new(),
        pushed_per_edge: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.empty
        .into_iter()
        .filter(|(name, _)| v.pushed_per_edge.contains(name))
        .map(|(name, span)| {
            let diag = Diagnostic::warning(
                MANUAL_ADJACENCY_LIST.code,
                span,
                format!("`{name}` is an adjacency list built one read edge at a time"),
            )
            .with_help(HELP);
            Finding {
                lint: &MANUAL_ADJACENCY_LIST,
                diag,
            }
        })
        .collect()
}

fn is_empty_list(e: &Expr) -> bool {
    matches!(&e.kind, ExprKind::List(xs) if xs.is_empty())
}

fn is_read_call(e: &Expr) -> bool {
    matches!(&e.kind, ExprKind::Call { callee, .. } if matches!(&callee.kind, ExprKind::Name(n) if n == "read"))
}

/// `u`, `u - 1` or `u + 1`: a read vertex used directly, by name. A bucket
/// keyed by a computed value (`bads[diag(i, x)]`) is not an adjacency list.
fn vertex(e: &Expr) -> Option<&str> {
    match &unparen(e).kind {
        ExprKind::Name(n) => Some(n),
        ExprKind::Binary {
            op: BinOp::Add | BinOp::Sub,
            lhs,
            rhs,
            ..
        } => match (&unparen(lhs).kind, &unparen(rhs).kind) {
            (ExprKind::Name(n), ExprKind::Int(_)) => Some(n),
            _ => None,
        },
        _ => None,
    }
}

/// The far end of a pushed edge: `v`, or the first of `(v, w)`.
fn pushed_end(e: &Expr) -> Option<&str> {
    match &unparen(e).kind {
        ExprKind::Tuple(parts) => parts.first().and_then(vertex),
        _ => vertex(e),
    }
}

/// The names `pat` binds.
fn bound(pat: &cpl_syntax::ast::Pattern) -> Vec<String> {
    struct Names(Vec<String>);
    impl Visitor for Names {
        fn bind(&mut self, name: &str, _span: Span, _how: Bind) {
            self.0.push(name.to_string());
        }
    }
    let mut names = Names(Vec::new());
    walk_pattern(&mut names, pat, Bind::Let);
    names.0
}
