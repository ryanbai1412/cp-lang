//! Which optionals fit in one scalar.
//!
//! `?V` is a `std::optional<V>`: the value and a flag beside it, 16 bytes
//! for an `Int`. Where some value of the scalar is never a `V`, that
//! value can say `none` instead, and the optional is the size of what it
//! holds. Three cases are decided by the type alone: a position is an
//! ordinal from 0, so -1 is free; a `Bool` is 0 or 1 in a byte; a `Char`
//! is a byte in sixteen bits.
//!
//! An `Int` has no value to spare, so `?Int` stays a `std::optional`
//! unless the program shows one: a table of `?Int` is packed, with
//! `Int.MIN` as `none`, when everything ever stored in it is proven not
//! to be `Int.MIN`. The proof is the small analysis below. It gives each
//! variable a bound on every `Int` held anywhere inside it, by following
//! each store in the program until nothing changes, and it trusts what
//! `cpl run` checks: declared input ranges, and that arithmetic does not
//! overflow. A table that fails it, or that is used as a whole value
//! anywhere (passed, returned, compared, looped over), stays as it was.

use crate::names::Niches;
use crate::ranges::Var;
use crate::walk;
use cpl_ir::{
    Arg, AssignOp, BinOp, Block, Builtin, Cast, Expr, ExprKind, Iter, Mutation, Place, PlaceKind,
    Program, SortBy, Spec, Stmt, Ty, UnOp,
};
use std::collections::{BTreeMap, BTreeSet};

/// True when the inside of a `?Bool` or `?Char` is taken as a place
/// somewhere in `block`: `(o ??= x) = y`.
fn small_place(block: &Block, bools: &mut bool, chars: &mut bool) {
    let mut place = |mut place: &Place| loop {
        match &place.kind {
            PlaceKind::Local(_) | PlaceKind::Global(_) => return,
            PlaceKind::Some(base) => {
                *bools &= base.ty != Ty::opt(Ty::Bool);
                *chars &= base.ty != Ty::opt(Ty::Char);
                place = base;
            }
            PlaceKind::Index(base, _) | PlaceKind::Field(base, _) => place = base,
        }
    };
    walk::deep_stmts(block, &mut |stmt| {
        if let Stmt::Assign(target, ..) = stmt {
            place(target);
        }
    });
    walk::deep_exprs(block, &mut |e| match &e.kind {
        ExprKind::Call(_, args) => {
            for arg in args {
                if let Arg::Mut(target) = arg {
                    place(target);
                }
            }
        }
        ExprKind::Mutate(_, target, _) | ExprKind::Sort(target, _) => place(target),
        _ => {}
    });
}

/// Whether `?Bool` and `?Char` are packed in this program.
pub(crate) fn small(program: &Program) -> Niches {
    let (mut bools, mut chars) = (true, true);
    let bodies = [&program.setup, &program.main]
        .into_iter()
        .chain(program.fns.iter().map(|f| &f.body))
        .chain(program.closures.iter().map(|c| &c.body));
    for body in bodies {
        small_place(body, &mut bools, &mut chars);
    }
    Niches { bools, chars }
}

/// What is known of every `Int` inside a value, weakest last.
type Level = u8;
/// There is no `Int` in it (yet).
const EMPTY: Level = 0;
/// Each is `>= 0`.
const NON_NEG: Level = 1;
/// Each is above `Int.MIN`.
const NOT_MIN: Level = 2;
const ANY: Level = 3;

/// Whose locals a body's slots are.
#[derive(Clone, Copy)]
enum Home {
    Main,
    Fn(usize),
    /// A closure's locals are not followed: they read as [`ANY`].
    Closure,
}

/// True for the types whose values are only ever built in ways this
/// analysis follows.
fn followed(ty: &Ty) -> bool {
    match ty {
        Ty::Unit
        | Ty::Int
        | Ty::Float
        | Ty::Bool
        | Ty::Char
        | Ty::Str
        | Ty::Pos(_)
        | Ty::Bound(_) => true,
        Ty::Opt(inner) | Ty::List(inner) => followed(inner),
        Ty::Array { elem, .. } => followed(elem),
        Ty::Tuple(items) => items.iter().all(followed),
        Ty::Record(fields) => fields.iter().all(|(_, t)| followed(t)),
        _ => false,
    }
}

/// True when a value of this type can hold an `Int`.
fn has_int(ty: &Ty) -> bool {
    match ty {
        Ty::Unit | Ty::Float | Ty::Bool | Ty::Char | Ty::Str | Ty::Pos(_) | Ty::Bound(_) => false,
        Ty::Opt(inner) | Ty::List(inner) => has_int(inner),
        Ty::Array { elem, .. } => has_int(elem),
        Ty::Tuple(items) => items.iter().any(has_int),
        Ty::Record(fields) => fields.iter().any(|(_, t)| has_int(t)),
        _ => true,
    }
}

/// A table of `?Int`: the only shape that is packed.
fn is_table(ty: &Ty) -> bool {
    let cell = Ty::opt(Ty::Int);
    match ty {
        Ty::List(elem) | Ty::Array { elem, .. } => **elem == cell,
        _ => false,
    }
}

fn is_var(e: &Expr) -> bool {
    matches!(e.kind, ExprKind::Local(_) | ExprKind::Global(_))
}

/// True when evaluating `e` only computes a value. An index that runs
/// code (a call, a `pop`, a block) is moved out of line together with the
/// table it indexes, as a whole value.
fn only_computes(e: &Expr) -> bool {
    let mut pure = true;
    walk::deep_expr(e, &mut |inner| {
        pure &= !matches!(
            inner.kind,
            ExprKind::Call(..)
                | ExprKind::CallExpr(..)
                | ExprKind::Mutate(..)
                | ExprKind::Sort(..)
                | ExprKind::Do(_)
                | ExprKind::Closure(_)
                | ExprKind::Read(_)
                | ExprKind::Print(_)
        );
    });
    pure
}

/// A value a packed table can be created from: `[]`, or a fill.
fn makes_table(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Fill(..) => true,
        ExprKind::Tuple(items) => items.is_empty(),
        _ => false,
    }
}

fn literal(v: i64) -> Level {
    match v {
        0.. => NON_NEG,
        i64::MIN => ANY,
        _ => NOT_MIN,
    }
}

/// `a op b` on `Int`s that does not overflow.
fn arith(op: BinOp, a: Level, b: Level) -> Level {
    let both = a.max(b) <= NON_NEG;
    match op {
        BinOp::Add if both => NON_NEG,
        // One side can only raise the other.
        BinOp::Add if a.min(b) <= NON_NEG && a.max(b) <= NOT_MIN => NOT_MIN,
        // At least `0 - Int.MAX`.
        BinOp::Sub if both => NOT_MIN,
        BinOp::Mul | BinOp::Pow | BinOp::Div | BinOp::BitOr | BinOp::BitXor if both => NON_NEG,
        // Euclidean: never negative.
        BinOp::Rem => NON_NEG,
        BinOp::BitAnd if a.min(b) <= NON_NEG => NON_NEG,
        BinOp::Shl | BinOp::Shr if a <= NON_NEG => NON_NEG,
        _ => ANY,
    }
}

struct Tables<'p> {
    program: &'p Program,
    level: BTreeMap<Var, Level>,
    /// Variables used in a way a packed table can't be.
    whole: BTreeSet<Var>,
    changed: bool,
}

impl<'p> Tables<'p> {
    fn local(&self, home: Home, slot: usize) -> Option<Var> {
        match home {
            Home::Main => Some((None, false, slot)),
            Home::Fn(id) => Some((Some(id), false, slot)),
            Home::Closure => None,
        }
    }

    fn var(&self, home: Home, e: &Expr) -> Option<Var> {
        match e.kind {
            ExprKind::Local(slot) => self.local(home, slot),
            ExprKind::Global(id) => Some((None, true, id)),
            _ => None,
        }
    }

    fn root(&self, home: Home, place: &Place) -> Option<Var> {
        match walk::root(place) {
            (true, id) => Some((None, true, id)),
            (false, slot) => self.local(home, slot),
        }
    }

    fn var_ty(&self, (fn_id, global, index): Var) -> &'p Ty {
        let program = self.program;
        match (global, fn_id) {
            (true, _) => &program.globals[index].ty,
            (false, Some(id)) => &program.fns[id].locals[index].ty,
            (false, None) => &program.main_locals[index].ty,
        }
    }

    fn raise(&mut self, var: Option<Var>, to: Level) {
        if let Some(var) = var {
            let level = self.level.entry(var).or_insert(EMPTY);
            if to > *level {
                *level = to;
                self.changed = true;
            }
        }
    }

    fn uses_whole(&mut self, var: Option<Var>) {
        if let Some(var) = var {
            self.whole.insert(var);
        }
    }

    fn read(&self, var: Option<Var>) -> Level {
        match var {
            Some(var) if followed(self.var_ty(var)) => {
                self.level.get(&var).copied().unwrap_or(EMPTY)
            }
            _ => ANY,
        }
    }

    fn spec(&self, home: Home, spec: &Spec) -> Level {
        match spec {
            Spec::Plain(ty) if has_int(ty) => ANY,
            Spec::Plain(_) => EMPTY,
            // Input is at least the low end it is declared with.
            Spec::Int { lo: Some(lo), .. } => self.val(home, lo),
            Spec::Int { lo: None, .. } => ANY,
            Spec::Fields(_, fields) => fields
                .iter()
                .map(|f| self.spec(home, f))
                .max()
                .unwrap_or(EMPTY),
            Spec::Array { elem, .. } => self.spec(home, elem),
            Spec::Opt(inner) => self.spec(home, inner),
        }
    }

    /// A bound on every `Int` in the value of `e`.
    fn val(&self, home: Home, e: &Expr) -> Level {
        if !has_int(&e.ty) {
            return EMPTY;
        }
        let of = |e: &Expr| self.val(home, e);
        let all = |es: &[Expr]| es.iter().map(of).max().unwrap_or(EMPTY);
        match &e.kind {
            ExprKind::Int(v) => literal(*v),
            ExprKind::None => EMPTY,
            ExprKind::Local(_) | ExprKind::Global(_) => self.read(self.var(home, e)),
            // A part holds no `Int` the whole does not.
            ExprKind::Some(inner)
            | ExprKind::Unwrap(inner, _)
            | ExprKind::Field(inner, _)
            | ExprKind::Index(inner, _) => of(inner),
            ExprKind::Mutate(Mutation::Pop | Mutation::PopVal | Mutation::Take, place, _) => {
                self.read(self.root(home, place))
            }
            ExprKind::Tuple(items) => all(items),
            ExprKind::If(_, a, b) => of(a).max(of(b)),
            ExprKind::Binary(BinOp::Coalesce, a, b) => of(a).max(of(b)),
            ExprKind::Binary(op, a, b) if e.ty == Ty::Int => arith(*op, of(a), of(b)),
            // `-x` is `Int.MIN` only for an `x` that is not an `Int`.
            ExprKind::Unary(UnOp::Neg, inner) if e.ty == Ty::Int && of(inner) <= NOT_MIN => NOT_MIN,
            ExprKind::Cast(Cast::Ord | Cast::CharToInt, _) | ExprKind::DomainLen(_) => NON_NEG,
            ExprKind::Fill(_, value) => value.as_deref().map(of).unwrap_or(NON_NEG),
            ExprKind::Read(spec) => self.spec(home, spec),
            ExprKind::Builtin(op, args) => match op {
                Builtin::Len
                | Builtin::Abs
                | Builtin::Isqrt
                | Builtin::Popcount
                | Builtin::Ctz
                | Builtin::Clz
                | Builtin::BitWidth
                | Builtin::BitCeil => NON_NEG,
                // The largest is no smaller than any of them. (An optional
                // argument may be skipped, so this is for `Int`s only.)
                Builtin::Max if args.iter().all(|a| a.ty == Ty::Int) => {
                    args.iter().map(of).min().unwrap_or(ANY)
                }
                // At least `lo`.
                Builtin::Rand => args.first().map(of).unwrap_or(ANY),
                // One of the arguments, or a part of one.
                Builtin::Min
                | Builtin::Max
                | Builtin::First
                | Builtin::Last
                | Builtin::FirstVal
                | Builtin::LastVal
                | Builtin::Sorted
                | Builtin::Reversed
                | Builtin::Copy => all(args),
                _ => ANY,
            },
            _ => ANY,
        }
    }

    fn block(&mut self, home: Home, block: &'p Block) {
        for stmt in &block.stmts {
            self.stmt(home, stmt);
        }
    }

    fn stmt(&mut self, home: Home, stmt: &'p Stmt) {
        match stmt {
            Stmt::Let(slot, init) => {
                let var = self.local(home, *slot);
                match init {
                    // The type's zero.
                    None => self.raise(var, NON_NEG),
                    Some(init) => {
                        self.raise(var, self.val(home, init));
                        if !makes_table(init) {
                            self.uses_whole(var);
                        }
                        self.expr(home, init);
                    }
                }
            }
            Stmt::Assign(place, op, value) => {
                let root = self.root(home, place);
                let stored = self.val(home, value);
                let level = match op {
                    // `max=` and `min=` keep one of the two.
                    AssignOp::Set | AssignOp::Max | AssignOp::Min => stored,
                    AssignOp::Binary(op) if place.ty == Ty::Int => {
                        arith(*op, self.read(root), stored)
                    }
                    AssignOp::Binary(_) => ANY,
                };
                self.raise(root, level);
                let set = *op == AssignOp::Set;
                let fits = match &place.kind {
                    PlaceKind::Local(_) | PlaceKind::Global(_) => set && makes_table(value),
                    PlaceKind::Index(base, indices) => {
                        set && place.ty == Ty::opt(Ty::Int)
                            && indices.iter().all(only_computes)
                            && matches!(base.kind, PlaceKind::Local(_) | PlaceKind::Global(_))
                    }
                    _ => false,
                };
                if !fits {
                    self.uses_whole(root);
                }
                walk::place_exprs(place, &mut |e| self.expr(home, e));
                self.expr(home, value);
            }
            Stmt::Expr(e) | Stmt::Give(e) | Stmt::Return(Some(e)) => self.expr(home, e),
            Stmt::If(cond, then, otherwise) => {
                self.expr(home, cond);
                self.block(home, then);
                self.block(home, otherwise);
            }
            Stmt::While(_, cond, body) => {
                self.expr(home, cond);
                self.block(home, body);
            }
            Stmt::For(_, slot, iter, body) => {
                let var = self.local(home, *slot);
                match iter {
                    Iter::Range { lo, hi, step, .. } => {
                        // Counting up from `lo`, in either order.
                        let up = step.as_ref().is_none_or(|s| self.val(home, s) <= NON_NEG);
                        let level = if up { self.val(home, lo) } else { ANY };
                        self.raise(var, level);
                        self.expr(home, lo);
                        self.expr(home, hi);
                        if let Some(step) = step {
                            self.expr(home, step);
                        }
                    }
                    Iter::RangeVal(e, _) => {
                        self.raise(var, ANY);
                        self.expr(home, e);
                    }
                    Iter::Seq(e, _) => {
                        self.raise(var, self.val(home, e));
                        self.expr(home, e);
                    }
                    Iter::Subsets(e) => {
                        self.raise(var, NON_NEG);
                        self.expr(home, e);
                    }
                }
                self.block(home, body);
            }
            Stmt::Assert { cond, message, .. } => {
                self.expr(home, cond);
                if let Some(message) = message {
                    self.expr(home, message);
                }
            }
            Stmt::DefDomain { lo, len, .. } => {
                self.expr(home, lo);
                self.expr(home, len);
            }
            Stmt::DefDomainFromVal { val, .. } => self.expr(home, val),
            Stmt::Return(None) | Stmt::Break(_) | Stmt::Continue(_) | Stmt::Trap(..) => {}
        }
    }

    /// The stores inside `e`, and every use of a variable that is not a
    /// read of one cell or of the length.
    fn expr(&mut self, home: Home, e: &'p Expr) {
        match &e.kind {
            ExprKind::Local(_) | ExprKind::Global(_) => self.uses_whole(self.var(home, e)),
            ExprKind::Index(base, indices)
                if is_var(base)
                    && e.ty == Ty::opt(Ty::Int)
                    && indices.iter().all(only_computes) =>
            {
                for index in indices {
                    self.expr(home, index);
                }
            }
            ExprKind::Builtin(Builtin::Len, args) if args.len() == 1 && is_var(&args[0]) => {}
            ExprKind::Do(block) => self.block(home, block),
            ExprKind::Closure(id) => {
                for capture in &self.program.closures[*id].captures {
                    let var = match capture.is_global {
                        true => Some((None, true, capture.outer)),
                        false => self.local(home, capture.outer),
                    };
                    self.uses_whole(var);
                    if capture.by_ref {
                        self.raise(var, ANY);
                    }
                }
            }
            ExprKind::Call(_, args) => {
                for arg in args {
                    match arg {
                        Arg::Value(value) => self.expr(home, value),
                        // The callee stores what it likes.
                        Arg::Mut(place) => {
                            let root = self.root(home, place);
                            self.uses_whole(root);
                            self.raise(root, ANY);
                            walk::place_exprs(place, &mut |e| self.expr(home, e));
                        }
                    }
                }
            }
            ExprKind::Mutate(op, place, args) => {
                let root = self.root(home, place);
                let level = match op {
                    Mutation::Pop
                    | Mutation::PopVal
                    | Mutation::Remove
                    | Mutation::Clear
                    | Mutation::Reverse
                    | Mutation::Swap
                    | Mutation::Truncate
                    | Mutation::Shuffle
                    | Mutation::Take => EMPTY,
                    Mutation::Push | Mutation::Extend | Mutation::Fill | Mutation::Insert => args
                        .iter()
                        .map(|a| self.val(home, a))
                        .max()
                        .unwrap_or(EMPTY),
                    // New cells are the given value, or the type's zero.
                    Mutation::Resize => args.get(1).map_or(NON_NEG, |a| self.val(home, a)),
                };
                self.raise(root, level);
                let whole = matches!(place.kind, PlaceKind::Local(_) | PlaceKind::Global(_));
                if !(whole && matches!(op, Mutation::Push | Mutation::Clear)) {
                    self.uses_whole(root);
                }
                walk::place_exprs(place, &mut |e| self.expr(home, e));
                for arg in args {
                    self.expr(home, arg);
                }
            }
            ExprKind::Sort(place, by) => {
                let root = self.root(home, place);
                self.uses_whole(root);
                walk::place_exprs(place, &mut |e| self.expr(home, e));
                match by {
                    SortBy::Natural => {}
                    SortBy::Key(slot, key) => {
                        self.raise(self.local(home, *slot), self.read(root));
                        self.expr(home, key);
                    }
                    SortBy::Fn { func, .. } => self.expr(home, func),
                }
            }
            _ => walk::children(e, &mut |child| self.expr(home, child)),
        }
    }
}

/// The `?Int` tables to hold as `cpl::OptInt`: `Int.MIN` is never stored
/// in them, and they are only ever read and written a cell at a time.
pub(crate) fn packed_tables(program: &Program) -> BTreeSet<Var> {
    let mut tables = Tables {
        program,
        level: BTreeMap::new(),
        whole: BTreeSet::new(),
        changed: true,
    };
    // A parameter's value comes from each caller.
    for (id, f) in program.fns.iter().enumerate() {
        for slot in 0..f.params.len() {
            tables.level.insert((Some(id), false, slot), ANY);
        }
    }
    while tables.changed {
        tables.changed = false;
        tables.block(Home::Main, &program.setup);
        tables.block(Home::Main, &program.main);
        for (id, f) in program.fns.iter().enumerate() {
            tables.block(Home::Fn(id), &f.body);
        }
        for closure in &program.closures {
            tables.block(Home::Closure, &closure.body);
        }
    }
    let globals = (0..program.globals.len()).map(|id| (None, true, id));
    let top = (0..program.main_locals.len()).map(|slot| (None, false, slot));
    let locals = program.fns.iter().enumerate().flat_map(|(id, f)| {
        (f.params.len()..f.locals.len()).map(move |slot| (Some(id), false, slot))
    });
    globals
        .chain(top)
        .chain(locals)
        .filter(|var| {
            is_table(tables.var_ty(*var))
                && tables.level.get(var).copied().unwrap_or(EMPTY) <= NOT_MIN
                && !tables.whole.contains(var)
        })
        .collect()
}
