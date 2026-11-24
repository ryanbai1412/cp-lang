//! Whether a call's `mut` args are safe to take out of their places for the
//! call: no fn value passed beside them can reach those places meanwhile.

use super::Compiler;
use cpl_ir::{
    Arg, Block, ClosureId, Expr, ExprKind, FnId, GlobalId, Place, PlaceKind, Slot, Stmt, Ty,
};

impl<'p> Compiler<'p, '_> {
    pub(super) fn mut_fn_args_alias_free(&self, fn_id: FnId, args: &'p [Arg]) -> bool {
        let f = &self.program.fns[fn_id];
        let (mut globals, mut locals) = (Vec::new(), Vec::new());
        for arg in args {
            if let Arg::Mut(place) = arg {
                match root(place) {
                    Root::Global(g) => globals.push(g),
                    Root::Local(s) => locals.push(s),
                }
            }
        }
        let mut reach = Reach {
            program: self.program,
            globals: &globals,
            seen: Vec::new(),
        };
        for (i, arg) in args.iter().enumerate() {
            if !matches!(f.locals[i].ty, Ty::Fn(..)) {
                continue;
            }
            let Arg::Value(Expr {
                kind: ExprKind::Closure(cid),
                ..
            }) = arg
            else {
                return false;
            };
            let lends_capture = (self.program.closures[*cid].captures.iter()).any(|c| {
                c.by_ref
                    && match c.is_global {
                        true => globals.contains(&(c.outer as GlobalId)),
                        false => locals.contains(&c.outer),
                    }
            });
            if lends_capture || reach.closure(*cid) {
                return false;
            }
        }
        true
    }
}

impl<'p> Compiler<'p, '_> {
    /// Whether a call must lend its args: a fn value passed may reach a place
    /// the call takes out, its `mut` args or a slot this frame holds lent.
    #[inline(never)]
    pub(super) fn call_must_lend(&self, fn_id: FnId, args: &'p [Arg]) -> bool {
        let f = &self.program.fns[fn_id];
        let has_fn_param = f.locals[..f.params.len()]
            .iter()
            .any(|l| matches!(l.ty, Ty::Fn(..)));
        let has_mut = f.params.iter().any(|p| p.mutable);
        has_mut && has_fn_param && !self.mut_fn_args_alias_free(fn_id, args)
            || self.fn_args_reach_held(f, args)
    }

    /// Whether a fn value among `args` may reach a slot this frame holds
    /// lent: the callee would call it while the lender's place is empty.
    fn fn_args_reach_held(&self, f: &cpl_ir::Function, args: &[Arg]) -> bool {
        if self.held.is_empty() {
            return false;
        }
        let held = |s: &Slot| self.held.contains(&(*s as u16));
        (args.iter().zip(&f.locals)).any(|(arg, param)| {
            if !param.ty.has_fn(self.program) {
                return false;
            }
            match arg {
                Arg::Value(Expr {
                    kind: ExprKind::Closure(cid),
                    ..
                }) => (self.program.closures[*cid].captures.iter())
                    .any(|c| c.by_ref && !c.is_global && held(&c.outer)),
                Arg::Value(_) => {
                    self.ref_all || (self.ref_slots.iter()).any(|&s| held(&(s as Slot)))
                }
                Arg::Mut(_) => false,
            }
        })
    }
}

enum Root {
    Global(GlobalId),
    Local(Slot),
}

fn root(p: &Place) -> Root {
    match &p.kind {
        PlaceKind::Global(g) => Root::Global(*g),
        PlaceKind::Local(s) => Root::Local(*s),
        PlaceKind::Index(base, _) | PlaceKind::Field(base, _) | PlaceKind::Some(base) => root(base),
    }
}

/// Whether code may touch one of `globals`, through the fns it calls too.
struct Reach<'a> {
    program: &'a cpl_ir::Program,
    globals: &'a [GlobalId],
    seen: Vec<FnId>,
}

impl Reach<'_> {
    fn closure(&mut self, cid: ClosureId) -> bool {
        let program = self.program;
        !self.globals.is_empty() && self.block(&program.closures[cid].body)
    }

    fn call(&mut self, fn_id: FnId) -> bool {
        if self.seen.contains(&fn_id) {
            return false;
        }
        self.seen.push(fn_id);
        let program = self.program;
        self.block(&program.fns[fn_id].body)
    }

    fn place(&mut self, p: &Place) -> bool {
        match &p.kind {
            PlaceKind::Global(g) => self.globals.contains(g),
            PlaceKind::Local(_) => false,
            PlaceKind::Index(base, idxs) => self.place(base) || idxs.iter().any(|e| self.expr(e)),
            PlaceKind::Field(base, _) | PlaceKind::Some(base) => self.place(base),
        }
    }

    fn expr(&mut self, e: &Expr) -> bool {
        match &e.kind {
            ExprKind::Unit
            | ExprKind::Int(_)
            | ExprKind::Float(_)
            | ExprKind::Bool(_)
            | ExprKind::Char(_)
            | ExprKind::Str(_)
            | ExprKind::None
            | ExprKind::Local(_)
            | ExprKind::DomainVal(_)
            | ExprKind::DomainLen(_)
            | ExprKind::DomainLo(_)
            | ExprKind::Error => false,
            ExprKind::Global(g) => self.globals.contains(g),
            ExprKind::Some(x)
            | ExprKind::Unary(_, x)
            | ExprKind::Cast(_, x)
            | ExprKind::Field(x, _)
            | ExprKind::IsVariant(x, _)
            | ExprKind::Payload(x, _, _)
            | ExprKind::IsSome(x)
            | ExprKind::Unwrap(x, _) => self.expr(x),
            ExprKind::Binary(_, a, b)
            | ExprKind::Cmp(_, a, b)
            | ExprKind::Range { lo: a, hi: b } => self.expr(a) || self.expr(b),
            ExprKind::RangeCheck { value, lo, hi, .. } => {
                self.expr(value)
                    || lo.as_deref().is_some_and(|x| self.expr(x))
                    || hi.as_deref().is_some_and(|x| self.expr(x))
            }
            ExprKind::If(c, t, f) => self.expr(c) || self.expr(t) || self.expr(f),
            ExprKind::Index(b, idxs) => self.expr(b) || idxs.iter().any(|x| self.expr(x)),
            ExprKind::Tuple(xs)
            | ExprKind::Variant(_, xs)
            | ExprKind::Builtin(_, xs)
            | ExprKind::Intrinsic(_, xs) => xs.iter().any(|x| self.expr(x)),
            ExprKind::Do(blk) => self.block(blk),
            ExprKind::Call(fn_id, args) => {
                args.iter().any(|a| match a {
                    Arg::Value(x) => self.expr(x),
                    Arg::Mut(p) => self.place(p),
                }) || self.call(*fn_id)
            }
            ExprKind::Closure(cid) => self.closure(*cid),
            ExprKind::CallExpr(..)
            | ExprKind::Mutate(..)
            | ExprKind::Sort(..)
            | ExprKind::Read(..)
            | ExprKind::ReadFrom(..)
            | ExprKind::Print(..)
            | ExprKind::Debug(..)
            | ExprKind::Format(..)
            | ExprKind::Fill(..)
            | ExprKind::MutSlice(..)
            | ExprKind::MutLine(..) => true,
        }
    }

    fn block(&mut self, b: &Block) -> bool {
        b.stmts.iter().any(|s| match s {
            Stmt::Let(_, init) => init.as_ref().is_some_and(|e| self.expr(e)),
            Stmt::Assign(place, _, rhs) => self.place(place) || self.expr(rhs),
            Stmt::Expr(e) | Stmt::Give(e) => self.expr(e),
            Stmt::Return(e) => e.as_ref().is_some_and(|e| self.expr(e)),
            Stmt::If(c, t, f) => self.expr(c) || self.block(t) || self.block(f),
            Stmt::While(_, c, body) => self.expr(c) || self.block(body),
            Stmt::Assert { cond, message, .. } => {
                self.expr(cond) || message.as_ref().is_some_and(|e| self.expr(e))
            }
            Stmt::Break(_) | Stmt::Continue(_) | Stmt::Trap(..) => false,
            Stmt::For(..) | Stmt::DefDomain { .. } | Stmt::DefDomainFromVal { .. } => true,
        })
    }
}
