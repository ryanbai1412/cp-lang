//! What is known about the sign of integers, so that Euclidean `/` and
//! `%` cost nothing where C++'s own would give the same answer.
//!
//! C++ rounds a quotient toward zero and the language rounds so that the
//! remainder is never negative. The two agree when the dividend is not
//! negative and the divisor is positive, and that is what this proves:
//! from literals, from the declared ranges of input (`let n: 1..=2e5 =
//! read()`), from loop bounds, and from what arithmetic keeps. A variable
//! counts only if it is stored exactly once, so a later assignment can't
//! undo the fact.

use cpl_ir::{
    Arg, BinOp, Block, Builtin, Cast, Expr, ExprKind, Iter, Place, PlaceKind, Program, Spec, Stmt,
    Ty,
};
use std::collections::BTreeMap;

/// A variable: a global, or a local of one fn (`None` is the top level).
pub(crate) type Var = (Option<usize>, bool, usize);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Sign {
    Unknown,
    /// `>= 0`.
    NonNeg,
    /// `> 0`.
    Positive,
}

#[derive(Default)]
pub(crate) struct Facts<'p> {
    /// How many times each variable is stored to, in any way.
    stores: BTreeMap<Var, u32>,
    /// The value a variable starts with, or the spec it is read with.
    init: BTreeMap<Var, &'p Expr>,
    /// Loop variables: the low end of the range they run over.
    loop_lo: BTreeMap<Var, &'p Expr>,
    /// Each domain's length expression, when defined once.
    domain_len: BTreeMap<usize, Option<(Option<usize>, &'p Expr)>>,
}

fn var(fn_id: Option<usize>, kind: &PlaceKind) -> Option<Var> {
    match kind {
        PlaceKind::Local(slot) => Some((fn_id, false, *slot)),
        PlaceKind::Global(id) => Some((None, true, *id)),
        PlaceKind::Index(base, _) | PlaceKind::Field(base, _) | PlaceKind::Some(base) => {
            var(fn_id, &base.kind)
        }
    }
}

impl<'p> Facts<'p> {
    pub fn collect(program: &'p Program) -> Facts<'p> {
        let mut facts = Facts::default();
        facts.block(None, &program.setup);
        facts.block(None, &program.main);
        for (id, f) in program.fns.iter().enumerate() {
            // A parameter's value comes from each caller: unknown.
            for slot in 0..f.params.len() {
                *facts.stores.entry((Some(id), false, slot)).or_default() += 2;
            }
            facts.block(Some(id), &f.body);
        }
        facts
    }

    /// The stores the body of one fn makes, on its own.
    pub fn of_fn(id: usize, body: &'p Block) -> Facts<'p> {
        let mut facts = Facts::default();
        facts.block(Some(id), body);
        facts
    }

    fn store(&mut self, fn_id: Option<usize>, place: &Place) {
        if let Some(v) = var(fn_id, &place.kind) {
            *self.stores.entry(v).or_default() += 1;
        }
    }

    fn block(&mut self, fn_id: Option<usize>, block: &'p Block) {
        for stmt in &block.stmts {
            match stmt {
                Stmt::Let(slot, init) => {
                    let v = (fn_id, false, *slot);
                    *self.stores.entry(v).or_default() += 1;
                    if let Some(init) = init {
                        self.init.insert(v, init);
                        self.expr(fn_id, init);
                    }
                }
                Stmt::Assign(place, _, value) => {
                    self.store(fn_id, place);
                    if let (PlaceKind::Global(_) | PlaceKind::Local(_), Some(v)) =
                        (&place.kind, var(fn_id, &place.kind))
                    {
                        self.init.entry(v).or_insert(value);
                    }
                    self.place(fn_id, place);
                    self.expr(fn_id, value);
                }
                Stmt::Expr(e) | Stmt::Give(e) | Stmt::Return(Some(e)) => self.expr(fn_id, e),
                Stmt::If(cond, then, otherwise) => {
                    self.expr(fn_id, cond);
                    self.block(fn_id, then);
                    self.block(fn_id, otherwise);
                }
                Stmt::While(_, cond, body) => {
                    self.expr(fn_id, cond);
                    self.block(fn_id, body);
                }
                Stmt::For(_, slot, iter, body) => {
                    let v = (fn_id, false, *slot);
                    *self.stores.entry(v).or_default() += 1;
                    match iter {
                        Iter::Range { lo, hi, step, .. } => {
                            self.loop_lo.insert(v, lo);
                            self.expr(fn_id, lo);
                            self.expr(fn_id, hi);
                            if let Some(step) = step {
                                self.expr(fn_id, step);
                            }
                        }
                        Iter::RangeVal(e, _) | Iter::Seq(e, _) | Iter::Subsets(e) | Iter::Members(e) => {
                            self.expr(fn_id, e)
                        }
                        Iter::TreeRange { container, lo, hi, .. } => {
                            self.expr(fn_id, container);
                            if let Some((e, _)) = lo {
                                self.expr(fn_id, e);
                            }
                            if let Some((e, _)) = hi {
                                self.expr(fn_id, e);
                            }
                        }
                    }
                    self.block(fn_id, body);
                }
                Stmt::Assert { cond, message, .. } => {
                    self.expr(fn_id, cond);
                    if let Some(message) = message {
                        self.expr(fn_id, message);
                    }
                }
                Stmt::DefDomain { id, lo, len, .. } => {
                    self.domain_len
                        .entry(*id)
                        .and_modify(|slot| *slot = None)
                        .or_insert(Some((fn_id, len)));
                    self.expr(fn_id, lo);
                    self.expr(fn_id, len);
                }
                Stmt::DefDomainFromVal { val, .. } => self.expr(fn_id, val),
                Stmt::Return(None) | Stmt::Break(_) | Stmt::Continue(_) | Stmt::Trap(..) => {}
            }
        }
    }

    fn place(&mut self, fn_id: Option<usize>, place: &'p Place) {
        match &place.kind {
            PlaceKind::Local(_) | PlaceKind::Global(_) => {}
            PlaceKind::Field(base, _) | PlaceKind::Some(base) => self.place(fn_id, base),
            PlaceKind::Index(base, indices) => {
                self.place(fn_id, base);
                for index in indices {
                    self.expr(fn_id, index);
                }
            }
        }
    }

    /// Finds the stores and blocks inside an expression.
    fn expr(&mut self, fn_id: Option<usize>, e: &'p Expr) {
        match &e.kind {
            ExprKind::Do(block) => self.block(fn_id, block),
            ExprKind::Call(_, args) | ExprKind::CallExpr(_, args) => {
                if let ExprKind::CallExpr(callee, _) = &e.kind {
                    self.expr(fn_id, callee);
                }
                for arg in args {
                    match arg {
                        Arg::Value(value) => self.expr(fn_id, value),
                        Arg::Mut(place) => {
                            self.store(fn_id, place);
                            self.place(fn_id, place);
                        }
                    }
                }
            }
            ExprKind::Mutate(_, place, args) => {
                self.store(fn_id, place);
                self.place(fn_id, place);
                for arg in args {
                    self.expr(fn_id, arg);
                }
            }
            ExprKind::Sort(place, sort_by, desc) => {
                self.store(fn_id, place);
                self.place(fn_id, place);
                if let Some(desc) = desc {
                    self.expr(fn_id, desc);
                }
                match sort_by {
                    cpl_ir::SortBy::Natural => {}
                    cpl_ir::SortBy::Key(slot, key) => {
                        *self.stores.entry((fn_id, false, *slot)).or_default() += 2;
                        self.expr(fn_id, key);
                    }
                    cpl_ir::SortBy::Fn { func, .. } => self.expr(fn_id, func),
                }
            }
            _ => crate::walk::children(e, &mut |child| self.expr(fn_id, child)),
        }
    }

    /// How many times the variable is stored to. A parameter starts at two.
    pub fn stores(&self, v: Var) -> u32 {
        self.stores.get(&v).copied().unwrap_or(0)
    }

    /// A static upper bound on `domain`'s length when provable from literals
    /// and declared input ranges (`let n: 5..=300 = read()`).
    pub fn domain_max_len(&self, id: usize) -> Option<i64> {
        let (fn_id, len) = self.domain_len.get(&id).copied().flatten()?;
        self.upper_bound(fn_id, len, 0)
    }

    fn upper_bound(&self, fn_id: Option<usize>, e: &Expr, depth: u8) -> Option<i64> {
        if depth > 8 {
            return None;
        }
        match &e.kind {
            ExprKind::Int(n) => Some(*n),
            ExprKind::Read(spec) => match spec.as_ref() {
                Spec::Int { hi: Some(hi), .. } => self.upper_bound(fn_id, hi, depth + 1),
                _ => None,
            },
            ExprKind::Local(slot) => {
                let v = (fn_id, false, *slot);
                if self.stores(v) == 1 {
                    self.init
                        .get(&v)
                        .and_then(|init| self.upper_bound(fn_id, init, depth + 1))
                } else {
                    None
                }
            }
            ExprKind::Global(id) => {
                let v = (None, true, *id);
                if self.stores(v) == 1 {
                    self.init
                        .get(&v)
                        .and_then(|init| self.upper_bound(None, init, depth + 1))
                } else {
                    None
                }
            }
            ExprKind::Binary(BinOp::Add, l, r) => self
                .upper_bound(fn_id, l, depth + 1)?
                .checked_add(self.upper_bound(fn_id, r, depth + 1)?),
            ExprKind::Binary(BinOp::Sub, l, r) => {
                let l_hi = self.upper_bound(fn_id, l, depth + 1)?;
                if let ExprKind::Int(c) = r.kind {
                    l_hi.checked_sub(c)
                } else if self.sign(fn_id, r) != Sign::Unknown {
                    Some(l_hi)
                } else {
                    None
                }
            }
            ExprKind::Builtin(Builtin::Max, args) if !args.is_empty() => {
                let mut hi = i64::MIN;
                for arg in args {
                    hi = hi.max(self.upper_bound(fn_id, arg, depth + 1)?);
                }
                Some(hi)
            }
            ExprKind::If(_, then, otherwise) => Some(
                self.upper_bound(fn_id, then, depth + 1)?
                    .max(self.upper_bound(fn_id, otherwise, depth + 1)?),
            ),
            ExprKind::Cast(_, inner) => self.upper_bound(fn_id, inner, depth + 1),
            _ => None,
        }
    }

    /// The value of the variable's first store.
    pub fn init(&self, v: Var) -> Option<&'p Expr> {
        self.init.get(&v).copied()
    }

    /// True when the variable is stored once, so its first value is its
    /// only value.
    pub fn stable(&self, v: Var) -> bool {
        self.stores.get(&v).copied().unwrap_or(0) <= 1
    }

    /// The declared range of the value `e` reads, when `e` is a path into
    /// a variable that was read from input and never changed.
    fn spec(&self, fn_id: Option<usize>, e: &Expr) -> Option<&'p Spec> {
        match &e.kind {
            ExprKind::Local(_) | ExprKind::Global(_) => {
                let v = match &e.kind {
                    ExprKind::Local(slot) => (fn_id, false, *slot),
                    ExprKind::Global(id) => (None, true, *id),
                    _ => return None,
                };
                if !self.stable(v) {
                    return None;
                }
                match &self.init.get(&v)?.kind {
                    ExprKind::Read(spec) => Some(spec),
                    _ => None,
                }
            }
            ExprKind::Index(base, _) => match self.spec(fn_id, base)? {
                Spec::Array { elem, .. } => Some(elem),
                _ => None,
            },
            ExprKind::Field(base, index) => match self.spec(fn_id, base)? {
                Spec::Fields(_, fields) => fields.get(*index),
                _ => None,
            },
            _ => None,
        }
    }

    /// What is known about the sign of the `Int` expression `e`, which
    /// appears in fn `fn_id`.
    pub fn sign(&self, fn_id: Option<usize>, e: &Expr) -> Sign {
        self.sign_at(fn_id, e, 0)
    }

    fn sign_at(&self, fn_id: Option<usize>, e: &Expr, depth: u32) -> Sign {
        use Sign::*;
        if depth > 16 || e.ty != Ty::Int {
            return Unknown;
        }
        let of = |e: &Expr| self.sign_at(fn_id, e, depth + 1);
        let both = |a: Sign, b: Sign| match (a, b) {
            (Unknown, _) | (_, Unknown) => Unknown,
            (Positive, Positive) => Positive,
            _ => NonNeg,
        };
        let literal = |v: i64| match v {
            v if v > 0 => Positive,
            0 => NonNeg,
            _ => Unknown,
        };
        match &e.kind {
            ExprKind::Int(v) => literal(*v),
            ExprKind::Local(_)
            | ExprKind::Global(_)
            | ExprKind::Index(..)
            | ExprKind::Field(..) => {
                if let Some(Spec::Int { lo: Some(lo), .. }) = self.spec(fn_id, e) {
                    if let ExprKind::Int(lo) = lo.kind {
                        return literal(lo);
                    }
                }
                let v = match &e.kind {
                    ExprKind::Local(slot) => (fn_id, false, *slot),
                    ExprKind::Global(id) => (None, true, *id),
                    _ => return Unknown,
                };
                if !self.stable(v) {
                    return Unknown;
                }
                // A global is set by top-level code, a local by its own fn.
                let home = if v.1 { None } else { fn_id };
                if let Some(lo) = self.loop_lo.get(&v) {
                    return self.sign_at(home, lo, depth + 1);
                }
                match self.init.get(&v) {
                    Some(init) => self.sign_at(home, init, depth + 1),
                    None => Unknown,
                }
            }
            ExprKind::Binary(op, l, r) => match op {
                BinOp::Add => match (of(l), of(r)) {
                    (Unknown, _) | (_, Unknown) => Unknown,
                    (NonNeg, NonNeg) => NonNeg,
                    _ => Positive,
                },
                BinOp::Mul | BinOp::Pow => both(of(l), of(r)),
                // Euclidean: never negative, whatever the operands.
                BinOp::Rem => NonNeg,
                BinOp::Div | BinOp::Shr | BinOp::BitOr | BinOp::BitXor => {
                    match both(of(l), of(r)) {
                        Unknown => Unknown,
                        _ => NonNeg,
                    }
                }
                BinOp::BitAnd => match (of(l), of(r)) {
                    (Unknown, Unknown) => Unknown,
                    _ => NonNeg,
                },
                // The interpreter rejects a shift that overflows.
                BinOp::Shl => of(l),
                _ => Unknown,
            },
            ExprKind::Builtin(Builtin::Len | Builtin::AxisLen, _) => NonNeg,
            ExprKind::Builtin(Builtin::Abs, _) => NonNeg,
            ExprKind::Builtin(Builtin::Min, args) => {
                args.iter().map(of).reduce(both).unwrap_or(Unknown)
            }
            ExprKind::Builtin(Builtin::Max, args) => {
                let signs: Vec<Sign> = args.iter().map(of).collect();
                if signs.contains(&Positive) {
                    Positive
                } else if signs.contains(&NonNeg) {
                    NonNeg
                } else {
                    Unknown
                }
            }
            ExprKind::Cast(Cast::CharToInt | Cast::Ord, _) | ExprKind::DomainLen(_) => NonNeg,
            ExprKind::If(_, a, b) => match both(of(a), of(b)) {
                Unknown => Unknown,
                sign => sign,
            },
            _ => Unknown,
        }
    }
}
