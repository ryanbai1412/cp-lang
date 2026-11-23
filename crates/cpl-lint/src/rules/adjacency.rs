//! `manual_adjacency_list` (L0537): C++'s `vector<vector<int>> adj(n + 1)`
//! filled by `adj[u].push_back(v)` as each edge is read. `std.graph`'s
//! `Graph.undirected(es)` / `Graph.directed(es)` build it from the edge list
//! read in one go, over a vertex domain.

use crate::walk::{walk_stmt, Visitor};
use crate::{File, Finding, MANUAL_ADJACENCY_LIST};
use cpl_diag::{Diagnostic, Span};
use cpl_syntax::ast::{Expr, ExprKind, PatternKind, Stmt, StmtKind, TypeKind};

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
                                && matches!(&args[..], [a] if is_list(a))
                                && is_empty_list(body)
                        }
                        (TypeKind::Array { elem, .. }, _) => is_list(elem) && is_empty_list(init),
                        _ => false,
                    };
                    if lists {
                        self.empty.push((name.clone(), stmt.span.shift(self.base)));
                    }
                }
                StmtKind::For { body, .. } => {
                    let reads = body.stmts.iter().any(
                        |s| matches!(&s.kind, StmtKind::Let { init: Some(e), .. } if is_read_call(e)),
                    );
                    if !reads {
                        return;
                    }
                    struct Pushes<'a>(&'a mut Vec<String>);
                    impl Visitor for Pushes<'_> {
                        fn expr(&mut self, expr: &Expr) {
                            let ExprKind::Call { callee, .. } = &expr.kind else {
                                return;
                            };
                            let ExprKind::Field { base, name } = &callee.kind else {
                                return;
                            };
                            if name.text != "push" {
                                return;
                            }
                            if let ExprKind::Index { base, .. } = &base.kind {
                                if let ExprKind::Name(list) = &base.kind {
                                    self.0.push(list.clone());
                                }
                            }
                        }
                    }
                    for s in &body.stmts {
                        walk_stmt(&mut Pushes(&mut self.pushed_per_edge), s);
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

fn is_list(ty: &cpl_syntax::ast::Type) -> bool {
    matches!(&ty.kind, TypeKind::Named { name, .. } if name.text == "List")
}

fn is_empty_list(e: &Expr) -> bool {
    matches!(&e.kind, ExprKind::List(xs) if xs.is_empty())
}

fn is_read_call(e: &Expr) -> bool {
    matches!(&e.kind, ExprKind::Call { callee, .. } if matches!(&callee.kind, ExprKind::Name(n) if n == "read"))
}
