//! The largest and smallest value an expression can take, from the input
//! bounds the program declares.
//!
//! `let n: 1..=2e5 = read()` says `n` is at most 200000, `const K = 20`
//! says `K` is 20, and `domain V = 1..=n` then has at most 200000 values.
//! Anything this cannot see through is `None`, and the caller stays quiet.

use crate::File;
use cpl_syntax::ast::{BinOp, Expr, ExprKind, PatternKind, StmtKind, Type, TypeKind, UnOp};
use std::collections::{HashMap, HashSet};

/// An inclusive range of integers.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Interval {
    pub lo: i128,
    pub hi: i128,
}

impl Interval {
    pub(crate) fn hull(self, other: Interval) -> Interval {
        Interval {
            lo: self.lo.min(other.lo),
            hi: self.hi.max(other.hi),
        }
    }

    /// How many integers it holds; 0 when it is empty.
    pub(crate) fn count(self) -> u128 {
        if self.hi < self.lo {
            0
        } else {
            (self.hi - self.lo) as u128 + 1
        }
    }
}

/// What the top level of a file says about its names.
#[derive(Default, Clone)]
pub(crate) struct Bounds {
    values: HashMap<String, Interval>,
    /// The number of values of each domain, at most.
    domains: HashMap<String, u128>,
    aliases: HashMap<String, Type>,
    /// Fns the program declares, which hide the prelude's `min`, `max`, `abs`.
    fns: HashSet<String>,
    /// How each name the prover can reason about got its value.
    defs: HashMap<String, Def>,
    /// How many places in the file bind each value name.
    bindings: HashMap<String, u32>,
}

/// A name the prover can be told about: an input of a declared range, or an
/// immutable `let` of arithmetic over such names, as prover text.
#[derive(Clone)]
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
enum Def {
    Input(Interval),
    Let { text: String, names: Vec<String> },
}

/// `expr` as fully parenthesized prover text and the names it reads, when
/// it is only integers, names and arithmetic.
fn prover_text(expr: &Expr, names: &mut Vec<String>) -> Option<String> {
    Some(match &expr.kind {
        ExprKind::Int(value) => value.to_string(),
        ExprKind::Name(name) => {
            names.push(name.clone());
            name.clone()
        }
        ExprKind::Paren(inner) => prover_text(inner, names)?,
        ExprKind::Unary {
            op: UnOp::Neg,
            expr,
        } => format!("(-{})", prover_text(expr, names)?),
        ExprKind::Binary { op, lhs, rhs, .. } => {
            let op = match op {
                BinOp::Add => "+",
                BinOp::Sub => "-",
                BinOp::Mul => "*",
                BinOp::Div => "/",
                BinOp::Rem => "%",
                BinOp::Pow => "**",
                _ => return None,
            };
            format!("({} {op} {})", prover_text(lhs, names)?, prover_text(rhs, names)?)
        }
        _ => return None,
    })
}

impl Bounds {
    pub(crate) fn of(file: &File) -> Bounds {
        let mut bounds = Bounds::default();
        bounds.count_bindings(file);
        // In source order: a bound can only use what is declared above it.
        for (_, stmt) in file.top_level() {
            match &stmt.kind {
                StmtKind::Const {
                    name,
                    init: Some(init),
                    ..
                } => {
                    if let Some(value) = bounds.expr(init) {
                        bounds.values.insert(name.text.clone(), value);
                    }
                }
                StmtKind::Let {
                    pat, ty: Some(ty), ..
                } => {
                    if let (PatternKind::Name(name), Some(range)) = (&pat.kind, bounds.range(ty)) {
                        bounds.input(name, range);
                    }
                }
                StmtKind::Let {
                    mutable: false,
                    pat,
                    ty: None,
                    init: Some(init),
                    ..
                } => {
                    if let (PatternKind::Name(name), Some(value)) = (&pat.kind, bounds.expr(init)) {
                        bounds.values.insert(name.clone(), value);
                        bounds.define(name, init);
                    }
                }
                StmtKind::Domain { name, def } => {
                    let size = match &def.kind {
                        ExprKind::Range {
                            lo: Some(lo),
                            hi: Some(hi),
                            inclusive,
                            step: None,
                        } => bounds.span(lo, hi, *inclusive).map(Interval::count),
                        _ => None,
                    };
                    if let Some(size) = size {
                        bounds.domains.insert(name.text.clone(), size);
                    }
                }
                StmtKind::Fn(decl) => bounds.declare_fn(&decl.name.text),
                StmtKind::Use {
                    path,
                    names,
                    glob,
                    alias,
                } => {
                    if *glob {
                        for name in ["min", "max", "abs"] {
                            bounds.declare_fn(name);
                        }
                    }
                    for name in path.last().into_iter().chain(names).chain(alias) {
                        bounds.declare_fn(&name.text);
                    }
                }
                StmtKind::TypeAlias { name, generics, ty } if generics.is_empty() => {
                    bounds.aliases.insert(name.text.clone(), ty.clone());
                }
                _ => {}
            }
        }
        bounds
    }

    /// From here on `name` is in `value`, or unknown: a local that
    /// shadows a bounded name hides its bound.
    fn count_bindings(&mut self, file: &File) {
        use cpl_syntax::ast::{Pattern, Stmt};
        use cpl_syntax::visit::{walk_stmt, Visitor};
        struct Count<'a>(&'a mut HashMap<String, u32>);
        impl Count<'_> {
            fn add(&mut self, name: &str) {
                *self.0.entry(name.to_string()).or_default() += 1;
            }
            fn names(&mut self, pat: &Pattern) {
                match &pat.kind {
                    PatternKind::Name(n) => self.add(n),
                    PatternKind::Domain(n) => self.add(&n.text),
                    PatternKind::Tuple(parts) | PatternKind::Or(parts) => {
                        parts.iter().for_each(|p| self.names(p))
                    }
                    PatternKind::Variant { args, .. } => args.iter().for_each(|p| self.names(p)),
                    PatternKind::Wildcard | PatternKind::Literal(_) | PatternKind::Error => {}
                }
            }
        }
        impl Visitor for Count<'_> {
            fn stmt(&mut self, stmt: &Stmt) {
                match &stmt.kind {
                    StmtKind::Fn(decl) => decl.params.iter().for_each(|p| self.add(&p.name.text)),
                    StmtKind::Const { name, .. } => self.add(&name.text),
                    _ => {}
                }
            }
            fn pattern(&mut self, pat: &Pattern) {
                self.names(pat);
            }
        }
        let mut count = Count(&mut self.bindings);
        file.top_level().for_each(|(_, s)| walk_stmt(s, &mut count));
    }

    /// Whether one place in the file binds `name`, so every use of it
    /// that resolves at all reads that binding.
    pub(crate) fn bound_once(&self, name: &str) -> bool {
        self.bindings.get(name) == Some(&1)
    }

    pub(crate) fn bind(&mut self, name: &str, value: Option<Interval>) {
        self.defs.remove(name);
        match value {
            Some(value) => self.values.insert(name.to_string(), value),
            None => self.values.remove(name),
        };
    }

    /// From here on `name` is an input anywhere in `range`.
    pub(crate) fn input(&mut self, name: &str, range: Interval) {
        self.bind(name, Some(range));
        self.defs.insert(name.to_string(), Def::Input(range));
    }

    /// `name` was just bound to the value of `init`, which the prover can
    /// follow when it is arithmetic over names it can follow.
    pub(crate) fn define(&mut self, name: &str, init: &Expr) {
        let mut names = Vec::new();
        if let Some(text) = prover_text(init, &mut names) {
            if names.iter().all(|n| self.defs.contains_key(n)) {
                self.defs.insert(name.to_string(), Def::Let { text, names });
            }
        }
    }

    /// Inputs in their declared ranges for which `expr` itself leaves
    /// `Int`, found and checked by the prover, as `n = 55110`.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn overflow(&self, expr: &Expr) -> Option<String> {
        fn tell(
            bounds: &Bounds,
            name: &str,
            ctx: &mut cpl_prover::Context,
            told: &mut HashSet<String>,
        ) -> Option<()> {
            if !told.insert(name.to_string()) {
                return Some(());
            }
            match bounds.defs.get(name)? {
                Def::Input(range) => ctx.var(name, &format!("{}..={}", range.lo, range.hi)).ok(),
                Def::Let { text, names } => {
                    for n in names {
                        tell(bounds, n, ctx, told)?;
                    }
                    ctx.define(name, text).ok()
                }
            }
        }
        let mut names = Vec::new();
        let text = prover_text(expr, &mut names)?;
        let mut ctx = cpl_prover::Context::new();
        let mut told = HashSet::new();
        for name in &names {
            tell(self, name, &mut ctx, &mut told)?;
        }
        let whole = cpl_prover::parse::term(&text).ok()?.to_string();
        let findings = ctx.check_overflow(&text).ok()?;
        let finding = findings.into_iter().find(|f| f.expr == whole)?;
        finding.verdict.counterexample().map(|c| c.to_string())
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn overflow(&self, _expr: &Expr) -> Option<String> {
        None
    }

    /// The interval a bounded name is in.
    pub(crate) fn expr_name(&self, name: &str) -> Option<Interval> {
        self.values.get(name).copied()
    }

    /// The values `lo..hi` or `lo..=hi` can cover.
    pub(crate) fn span(&self, lo: &Expr, hi: &Expr, inclusive: bool) -> Option<Interval> {
        let (lo, hi) = (self.expr(lo)?, self.expr(hi)?);
        Some(Interval {
            lo: lo.lo,
            hi: if inclusive {
                hi.hi
            } else {
                hi.hi.checked_sub(1)?
            },
        })
    }

    pub(crate) fn range(&self, ty: &Type) -> Option<Interval> {
        match &ty.kind {
            TypeKind::Range {
                lo: Some(lo),
                hi: Some(hi),
                inclusive,
            } => self.span(lo, hi, *inclusive),
            _ => None,
        }
    }

    /// The interval an expression's value lies in.
    pub(crate) fn expr(&self, expr: &Expr) -> Option<Interval> {
        match &expr.kind {
            ExprKind::Int(value) => Some(Interval {
                lo: *value,
                hi: *value,
            }),
            ExprKind::Name(name) => self.values.get(name).copied(),
            ExprKind::Paren(inner) => self.expr(inner),
            ExprKind::Unary {
                op: UnOp::Neg,
                expr,
            } => {
                let inner = self.expr(expr)?;
                Some(Interval {
                    lo: inner.hi.checked_neg()?,
                    hi: inner.lo.checked_neg()?,
                })
            }
            ExprKind::Binary { op, lhs, rhs, .. } => {
                Self::combine(*op, self.expr(lhs)?, self.expr(rhs)?)
            }
            ExprKind::Call { callee, args } => {
                let ExprKind::Name(name) = &callee.kind else {
                    return None;
                };
                if self.fns.contains(name) || args.iter().any(|a| a.name.is_some() || a.mutable) {
                    return None;
                }
                let values = args
                    .iter()
                    .map(|a| self.expr(&a.value))
                    .collect::<Option<Vec<_>>>()?;
                match (name.as_str(), values.as_slice()) {
                    ("abs", [x]) => Some(if x.lo >= 0 {
                        *x
                    } else if x.hi <= 0 {
                        Interval {
                            lo: x.hi.checked_neg()?,
                            hi: x.lo.checked_neg()?,
                        }
                    } else {
                        Interval {
                            lo: 0,
                            hi: x.lo.checked_neg()?.max(x.hi),
                        }
                    }),
                    ("max", [first, rest @ ..]) if !rest.is_empty() => {
                        Some(rest.iter().fold(*first, |a, b| Interval {
                            lo: a.lo.max(b.lo),
                            hi: a.hi.max(b.hi),
                        }))
                    }
                    ("min", [first, rest @ ..]) if !rest.is_empty() => {
                        Some(rest.iter().fold(*first, |a, b| Interval {
                            lo: a.lo.min(b.lo),
                            hi: a.hi.min(b.hi),
                        }))
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// The values `a op b` can take.
    pub(crate) fn combine(op: BinOp, a: Interval, b: Interval) -> Option<Interval> {
        match op {
            BinOp::Add => Some(Interval {
                lo: a.lo.checked_add(b.lo)?,
                hi: a.hi.checked_add(b.hi)?,
            }),
            BinOp::Sub => Some(Interval {
                lo: a.lo.checked_sub(b.hi)?,
                hi: a.hi.checked_sub(b.lo)?,
            }),
            BinOp::Mul => {
                let corners = [
                    a.lo.checked_mul(b.lo)?,
                    a.lo.checked_mul(b.hi)?,
                    a.hi.checked_mul(b.lo)?,
                    a.hi.checked_mul(b.hi)?,
                ];
                Some(Interval {
                    lo: *corners.iter().min()?,
                    hi: *corners.iter().max()?,
                })
            }
            BinOp::Div if b.lo > 0 => {
                let corners = [
                    a.lo.checked_div_euclid(b.lo)?,
                    a.lo.checked_div_euclid(b.hi)?,
                    a.hi.checked_div_euclid(b.lo)?,
                    a.hi.checked_div_euclid(b.hi)?,
                ];
                Some(Interval {
                    lo: *corners.iter().min()?,
                    hi: *corners.iter().max()?,
                })
            }
            BinOp::Rem if b.lo > 0 => Some(if a.lo >= 0 && a.hi < b.lo {
                a
            } else {
                let hi = b.hi - 1;
                Interval {
                    lo: 0,
                    hi: if a.lo >= 0 { hi.min(a.hi) } else { hi },
                }
            }),
            BinOp::Pow if b.lo == b.hi && b.lo >= 0 => {
                let k = u32::try_from(b.lo).ok()?;
                let (lo, hi) = (a.lo.checked_pow(k)?, a.hi.checked_pow(k)?);
                Some(if k % 2 == 1 || a.lo >= 0 {
                    Interval { lo, hi }
                } else if a.hi <= 0 {
                    Interval { lo: hi, hi: lo }
                } else {
                    Interval { lo: 0, hi: lo.max(hi) }
                })
            }
            _ => None,
        }
    }

    /// From here on `name` is a fn of the program's own, not the prelude's.
    pub(crate) fn declare_fn(&mut self, name: &str) {
        self.fns.insert(name.to_string());
    }

    /// How many values a type has, at most.
    pub(crate) fn type_size(&self, ty: &Type) -> Option<u128> {
        match &ty.kind {
            TypeKind::Range { .. } => self.range(ty).map(Interval::count),
            TypeKind::Named { name, args } if args.is_empty() => match name.text.as_str() {
                "Bool" => Some(2),
                "Char" | "U8" => Some(256),
                other => self
                    .domains
                    .get(other)
                    .copied()
                    .or_else(|| self.type_size(self.aliases.get(other)?)),
            },
            TypeKind::Optional(inner) => self.type_size(inner)?.checked_add(1),
            TypeKind::Tuple(parts) => parts.iter().try_fold(1u128, |total, part| {
                total.checked_mul(self.type_size(part)?)
            }),
            TypeKind::Record(fields) => fields.iter().try_fold(1u128, |total, (_, ty)| {
                total.checked_mul(self.type_size(ty)?)
            }),
            _ => None,
        }
    }
}
