//! What a walk lends (R05.9.1): `for x in g(a, ..)` or `for x in a.g(..)`
//! over a generator or lazy view lends what its arguments lend, receiver
//! included, through any chain of adapters, for the whole walk. A call
//! that gives owned storage (a list, a set, a number) lends nothing.

use super::*;

impl Checker<'_> {
    /// Lends the places a walk over the call `iter` reads until the loop
    /// `stmt` ends; `false` when `iter` is no generator or view call.
    pub(super) fn hold_walk(&mut self, stmt: &Stmt, iter: &Expr) -> bool {
        let mut sources = self.walk_sources(iter);
        if sources.is_empty() {
            // A walk over a plain element (`for x in a[i]`) is the loop's own place lend.
            sources = (self.extra_holds(iter).into_iter())
                .filter(|(kind, ..)| *kind != "element")
                .map(|(_, place, _, force_mut)| (place, force_mut))
                .collect();
        }
        if sources.is_empty() {
            return false;
        }
        for (place, force_mut) in sources {
            if force_mut || place.root_mutable {
                self.lends.holds.push(Hold {
                    name: String::new(),
                    root: place.root,
                    steps: place.steps,
                    mutable: force_mut,
                    at: iter.span,
                    until: stmt.span,
                    in_loop: true,
                    is_capture: false,
                });
            }
        }
        true
    }

    /// `for x in lo..hi` knows `x >= lo` and `x < hi` (`<=` for `..=`)
    /// in its body, so `a[k] = a[l].take()` under `for l in 0..k` is two
    /// disjoint parts.
    pub(super) fn add_loop_facts(&mut self, stmt: &Stmt) {
        let StmtKind::For { pat, iter, .. } = &stmt.kind else {
            return;
        };
        let (
            ast::PatternKind::Name(x),
            ExprKind::Range {
                lo,
                hi,
                inclusive,
                step: None,
            },
        ) = (&pat.kind, &unparen(iter).kind)
        else {
            return;
        };
        let bounds = [
            (lo, CmpOp::Ge),
            (hi, if *inclusive { CmpOp::Le } else { CmpOp::Lt }),
        ];
        for (bound, op) in bounds {
            let Some(bound) = bound.as_deref().filter(|b| is_pure(b)) else {
                continue;
            };
            let mut names = names_in(bound);
            names.push(x.clone());
            self.lends.facts.push(Fact {
                lhs: x.clone(),
                op,
                rhs: show(bound),
                names,
            });
        }
    }

    /// The places the walk over `iter` keeps lent, each with whether the
    /// lend is `mut`. Empty when `iter` is no generator or view call.
    pub(super) fn walk_sources(&mut self, iter: &Expr) -> Vec<(Place, bool)> {
        let e = unparen(iter);
        let (receiver, args): (Option<&Expr>, &[Arg]) = match &e.kind {
            ExprKind::Call { callee, args } => match &unparen(callee).kind {
                ExprKind::Field { base, .. } => (Some(&**base), args),
                _ => (None, args),
            },
            ExprKind::Field { base, .. } => (Some(&**base), &[]),
            _ => return Vec::new(),
        };
        if !self.reads_its_sources(e) {
            return Vec::new();
        }
        let mut out = Vec::new();
        let sources = receiver
            .map(|r| (r, false))
            .into_iter()
            .chain(args.iter().map(|a| (&a.value, a.mutable)));
        for (value, is_mut) in sources {
            let nested = self.walk_sources(value);
            if !nested.is_empty() {
                out.extend(nested);
                continue;
            }
            let ty = self.quietly(|c| c.infer(value));
            if is_mut || self.is_heap_type(&ty) {
                if let Some(mut place) = self.place(value) {
                    place.steps.clear();
                    out.push((place, is_mut));
                }
            }
        }
        out
    }

    /// Whether the value of the call or getter `e` reads its arguments as
    /// it is walked: its fn is `lends` or a generator, or its declared
    /// result wraps the type of a parameter (`Filtered<S>`, `Slice<S>` of
    /// `self: S`), so it holds that argument rather than storage of its own.
    fn reads_its_sources(&mut self, e: &Expr) -> bool {
        let callee = match &e.kind {
            ExprKind::Call { callee, .. } => unparen(callee),
            _ => e,
        };
        let def = match &callee.kind {
            ExprKind::Name(n) => self.global(n),
            ExprKind::Generic { base, .. } => match &unparen(base).kind {
                ExprKind::Name(n) => self.global(n),
                _ => None,
            },
            ExprKind::Field { name, .. } => self.chosen_method(name),
            _ => None,
        };
        let Some(def) = def else {
            return false;
        };
        let Def::Fn {
            ty: Ty::Fn { params, ret },
            lends,
            ..
        } = &*def
        else {
            return false;
        };
        if *lends || matches!(&**ret, Ty::Named { name, .. } if name == "Gen") {
            return true;
        }
        // A result of the parameter's own type (`clone(self: S) -> S`) is a
        // copy; one that wraps it is a view of it.
        params.iter().any(|p| match &p.ty {
            Ty::Param(t) => **ret != p.ty && mentions(ret, t),
            Ty::Named { name, args } if args.is_empty() => **ret != p.ty && mentions(ret, name),
            _ => false,
        })
    }
}

/// Whether `ty` names the type parameter `param` anywhere inside.
fn mentions(ty: &Ty, param: &str) -> bool {
    match ty {
        Ty::Param(p) => p == param,
        Ty::Named { name, args } => name == param || args.iter().any(|a| mentions(a, param)),
        Ty::Array { index, elem } => {
            index.iter().any(|i| mentions(i, param)) || mentions(elem, param)
        }
        Ty::Optional(inner) => mentions(inner, param),
        Ty::Tuple(items) => items.iter().any(|i| mentions(i, param)),
        Ty::Record(fields) => fields.iter().any(|(_, t)| mentions(t, param)),
        _ => false,
    }
}
