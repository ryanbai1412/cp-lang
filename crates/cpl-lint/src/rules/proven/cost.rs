//! How many iterations each fn runs per call, for `crate::ProvenFn`.

use super::*;

/// Each fn's iterations per call: its own loops plus, for each call of a
/// fn declared in the file, how often the call runs times that fn's own
/// count. Recursion, or a call inside a lambda, makes the count unknown.
pub(crate) fn proven_fns(file: &File) -> Vec<crate::ProvenFn> {
    /// Iterations per call, and the same as a sum of products of loop
    /// counts (`None` when it would be long).
    type Cost = (Option<u128>, Option<Vec<Vec<Factor>>>);
    enum State {
        Todo,
        Busy,
        Done(Cost),
    }
    const MOST_TERMS: usize = 6;
    fn cost(
        i: usize,
        fns: &[FnCost],
        by_name: &std::collections::HashMap<&str, usize>,
        state: &mut [State],
    ) -> Cost {
        match &state[i] {
            State::Done(c) => return c.clone(),
            State::Busy => return (None, None),
            State::Todo => state[i] = State::Busy,
        }
        let mut total = fns[i].own;
        let mut terms = Some(fns[i].own_terms.clone());
        for call in &fns[i].calls {
            if let Some(&j) = by_name.get(call.callee.as_str()) {
                let (each, each_terms) = cost(j, fns, by_name, state);
                if each == Some(0) {
                    continue;
                }
                total = total
                    .zip(call.times.zip(each))
                    .and_then(|(t, (n, e))| t.checked_add(n.checked_mul(e)?));
                terms = terms.zip(call.chain.as_ref().zip(each_terms)).map(
                    |(mut terms, (chain, each))| {
                        terms.extend(
                            each.into_iter()
                                .map(|t| chain.iter().cloned().chain(t).collect()),
                        );
                        terms
                    },
                );
            }
        }
        let terms = terms.filter(|t| t.len() <= MOST_TERMS && total.is_some());
        state[i] = State::Done((total, terms.clone()));
        (total, terms)
    }
    let fns = proven(file).fns;
    let mut by_name = std::collections::HashMap::new();
    for (i, f) in fns.iter().enumerate() {
        by_name.entry(f.name.as_str()).or_insert(i);
    }
    struct Structs(std::collections::HashSet<String>);
    impl cpl_syntax::visit::Visitor for Structs {
        fn stmt(&mut self, stmt: &cpl_syntax::ast::Stmt) {
            if let cpl_syntax::ast::StmtKind::Struct(decl) = &stmt.kind {
                self.0.insert(decl.name.text.clone());
            }
        }
    }
    let mut structs = Structs(Default::default());
    for stmt in file.items.iter().flat_map(|(_, item)| &item.stmts) {
        cpl_syntax::visit::walk_stmt(stmt, &mut structs);
    }
    let mut state: Vec<State> = fns.iter().map(|_| State::Todo).collect();
    (0..fns.len())
        .map(|i| {
            let (steps, terms) = cost(i, &fns, &by_name, &mut state);
            let f = &fns[i];
            crate::ProvenFn {
                name: f.at,
                body: f.body,
                runs: f
                    .runs
                    .iter()
                    .map(|(span, times, chain)| crate::Run {
                        span: *span,
                        times: *times,
                        count: chain.as_ref().map(|c| render_count(file, c)),
                    })
                    .collect(),
                steps,
                count: terms.map(|t| render_sum(file, &t)),
                callees: f
                    .callees
                    .iter()
                    .filter(|(name, _)| {
                        name.as_ref().is_none_or(|n| {
                            !by_name.contains_key(n.as_str()) && !structs.0.contains(n)
                        })
                    })
                    .map(|(_, at)| *at)
                    .collect(),
                builds: f
                    .callees
                    .iter()
                    .filter(|(name, _)| name.as_ref().is_some_and(|n| structs.0.contains(n)))
                    .map(|(_, at)| *at)
                    .collect(),
                helpers: f
                    .calls
                    .iter()
                    .filter_map(|call| {
                        Some(crate::Helper {
                            at: call.at,
                            callee: fns[*by_name.get(call.callee.as_str())?].at,
                            times: call.times,
                            count: call
                                .chain
                                .as_ref()
                                .filter(|c| !c.is_empty())
                                .map(|c| render_count(file, c)),
                        })
                    })
                    .collect(),
            }
        })
        .collect()
}

/// `n * q + 2 * n`: each product rendered by `render_count`, equal ones
/// gathered with their multiplicity.
pub(super) fn render_sum(file: &File, terms: &[Vec<Factor>]) -> String {
    let mut seen: Vec<(String, usize)> = Vec::new();
    for t in terms {
        let text = render_count(file, t);
        match seen.iter_mut().find(|(s, _)| *s == text) {
            Some((_, k)) => *k += 1,
            None => seen.push((text, 1)),
        }
    }
    let parts: Vec<String> = seen
        .into_iter()
        .map(|(t, k)| match k {
            1 => t,
            _ if t.contains(' ') => format!("{k} * ({t})"),
            _ => format!("{k} * {t}"),
        })
        .collect();
    parts.join(" + ")
}

/// A part of a fn's body, how often it runs per call, and its loop-count
/// chain when that is known.
pub(super) type RunCost = ((u32, u32), Option<u128>, Option<Vec<Factor>>);

/// A call by name in a fn, how often it runs, and the counts of the loops
/// around it (`None` in a lambda, table or comprehension).
pub(super) struct Call {
    pub(super) callee: String,
    pub(super) at: u32,
    pub(super) times: Option<u128>,
    pub(super) chain: Option<Vec<Factor>>,
}

/// A fn's own loop iterations per call (`None` when a loop has no count)
/// and the fns it calls by name, each with how often the call runs.
pub(super) struct FnCost {
    pub(super) name: String,
    pub(super) at: (u32, u32),
    pub(super) body: (u32, u32),
    pub(super) runs: Vec<RunCost>,
    pub(super) own: Option<u128>,
    /// The loop-count chain of each of its own loops.
    pub(super) own_terms: Vec<Vec<Factor>>,
    pub(super) calls: Vec<Call>,
    pub(super) callees: Vec<(Option<String>, (u32, u32))>,
}

/// The calls by name with their offsets, what runs an unknown number of
/// times or not here (lambdas, tables, comprehensions, nested fns), the
/// nested fns, the loops, and every call's callee: its name for a call by
/// name (`print`), the member for a method call (`sorted`).
pub(super) type Calls = (
    Vec<(String, u32)>,
    Vec<(u32, u32)>,
    Vec<(u32, u32)>,
    Vec<(u32, u32)>,
    Vec<(Option<String>, (u32, u32))>,
);

pub(super) fn calls_in(block: &cpl_syntax::ast::Block) -> Calls {
    use cpl_syntax::ast::{Expr, ExprKind, Stmt, StmtKind};
    use cpl_syntax::visit::{walk_block, Visitor};
    #[allow(clippy::type_complexity)]
    struct Found(
        Vec<(String, u32)>,
        Vec<(u32, u32)>,
        Vec<(u32, u32)>,
        Vec<(u32, u32)>,
        Vec<(Option<String>, (u32, u32))>,
    );
    impl Visitor for Found {
        fn stmt(&mut self, stmt: &Stmt) {
            match stmt.kind {
                StmtKind::Fn(_) => {
                    self.1.push((stmt.span.start, stmt.span.end));
                    self.2.push((stmt.span.start, stmt.span.end));
                }
                StmtKind::For { .. } | StmtKind::While { .. } => {
                    self.3.push((stmt.span.start, stmt.span.end))
                }
                _ => {}
            }
        }
        fn expr(&mut self, expr: &Expr) {
            match &expr.kind {
                ExprKind::Call { callee, .. } => match &callee.kind {
                    ExprKind::Name(name) => {
                        self.0.push((name.clone(), expr.span.start));
                        self.4
                            .push((Some(name.clone()), (callee.span.start, callee.span.end)));
                    }
                    ExprKind::Field { name, .. } => {
                        self.4.push((None, (name.span.start, name.span.end)))
                    }
                    _ => self.4.push((None, (callee.span.start, callee.span.end))),
                },
                ExprKind::Lambda { .. }
                | ExprKind::Table { .. }
                | ExprKind::Comprehension { .. } => {
                    self.1.push((expr.span.start, expr.span.end));
                }
                _ => {}
            }
        }
    }
    let mut calls = Found(Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    walk_block(block, &mut calls);
    (calls.0, calls.1, calls.2, calls.3, calls.4)
}
