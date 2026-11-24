//! Fused views: `xs.map(f)`, `xs.filter(p)` and `xs.items` lower to a list one
//! loop builds ([`crate::fuse::producer`]). Where `first`, `last` or `len`
//! reads that list, the read runs at each `push` and the list is never built;
//! a `for`, a comprehension, `any`, `all` and `sum` take the producer's loop
//! the same way in `crate::fuse`. The list is built only when it is a value:
//! a list stored, returned or indexed is a `let` of the producer, never the
//! operand of a read. The interpreter builds the list, calling `f` on every
//! element in order, so `first` stops at the first push only when the rest of
//! the loop can neither trap, print nor write past its own locals.

use std::borrow::Cow;

use cpl_ir::{
    AssignOp, Block, Builtin, Expr, ExprKind, Iter, LoopId, Mutation, Place, PlaceKind, Program,
    Stmt,
};

use crate::emit::{Code, Emitter};
use crate::fuse::{held_producer, FusedPush, Producer};
use crate::uses::effects::{pure_call, pure_fns, pure_path, trap_free_stmts, writes_own};
use crate::uses::split::each_value;
use crate::Checks;

/// What a read of a fused view keeps of its elements.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Read {
    First,
    Last,
    Len,
}

impl Read {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Read::First => "first",
            Read::Last => "last",
            Read::Len => "len",
        }
    }
}

/// A list or array built on the spot by one loop: the `let`s the loop
/// reads, run before it, and the loop, whose body pushes each element.
pub(crate) struct View<'e> {
    pub held: &'e [Stmt],
    vec: usize,
    id: &'e LoopId,
    slot: &'e usize,
    iter: &'e Iter,
    body: Cow<'e, Block>,
}

impl View<'_> {
    pub(crate) fn producer(&self) -> Producer<'_> {
        Producer {
            vec: &self.vec,
            id: self.id,
            slot: self.slot,
            iter: self.iter,
            body: &self.body,
        }
    }
}

/// `a.map(f)` on a 1-D array: `let out = [..]; for k in 0..n { .. out[k] = v }; give out`,
/// with `out[k] = v` read as a push of `v`, since `k` walks the cells in order.
fn filled(stmts: &[Stmt]) -> Option<View<'_>> {
    let [held @ .., Stmt::Let(out, Some(init)), Stmt::For(id, slot, iter, body), Stmt::Give(given)] =
        stmts
    else {
        return None;
    };
    let Iter::Range {
        lo,
        step: None,
        rev: false,
        closed: false,
        ..
    } = iter
    else {
        return None;
    };
    let (Some((Stmt::Assign(dst, AssignOp::Set, v), rest)), ExprKind::Fill(..)) =
        (body.stmts.split_last(), &init.kind)
    else {
        return None;
    };
    let PlaceKind::Index(whole, keys) = &dst.kind else {
        return None;
    };
    let at_k = matches!(keys.as_slice(), [k] if matches!(k.kind, ExprKind::Local(s) if s == *slot));
    let ok = at_k
        && matches!(lo.kind, ExprKind::Int(0))
        && matches!(whole.kind, PlaceKind::Local(w) if w == *out)
        && matches!(given.kind, ExprKind::Local(g) if g == *out)
        && held.iter().all(|s| matches!(s, Stmt::Let(_, Some(_))));
    if !ok {
        return None;
    }
    let place = Place {
        kind: PlaceKind::Local(*out),
        ty: init.ty.clone(),
        span: dst.span,
    };
    let push = Expr {
        kind: ExprKind::Mutate(Mutation::Push, Box::new(place), vec![v.clone()]),
        ty: cpl_ir::Ty::Unit,
        span: v.span,
    };
    let mut stmts = rest.to_vec();
    stmts.push(Stmt::Expr(push));
    let body = Block { stmts };
    let mut iter_uses = false;
    crate::fuse::iter_exprs(iter, &mut |e| {
        iter_uses |= crate::fuse::expr_mentions_local(e, *out)
    });
    (!iter_uses && crate::fuse::only_pushed_in_block(*out, &body)).then_some(View {
        held,
        vec: *out,
        id,
        slot,
        iter,
        body: Cow::Owned(body),
    })
}

/// `e` as a read of a list or array built on the spot, and its [`View`].
pub(crate) fn fused_read(e: &Expr) -> Option<(Read, View<'_>)> {
    let ExprKind::Builtin(op, args) = &e.kind else {
        return None;
    };
    let read = match op {
        Builtin::First => Read::First,
        Builtin::Last => Read::Last,
        Builtin::Len => Read::Len,
        _ => return None,
    };
    let [seq] = args.as_slice() else {
        return None;
    };
    let ExprKind::Do(block) = &seq.kind else {
        return None;
    };
    let view = match held_producer(&block.stmts) {
        Some((held, p)) => View {
            held,
            vec: *p.vec,
            id: p.id,
            slot: p.slot,
            iter: p.iter,
            body: Cow::Borrowed(p.body),
        },
        None => filled(&block.stmts)?,
    };
    Some((read, view))
}

/// `base[k]` for the loop's own counter `k` over `0..base.len`, which
/// `.items` reads: always in range.
fn own_index(p: &Producer<'_>, e: &Expr) -> bool {
    let (
        ExprKind::Index(base, idxs),
        Iter::Range {
            lo,
            hi,
            step: None,
            rev: false,
            closed: false,
        },
    ) = (&e.kind, p.iter)
    else {
        return false;
    };
    let same = |a: &Expr| match (&a.kind, &base.kind) {
        (ExprKind::Local(x), ExprKind::Local(y)) | (ExprKind::Global(x), ExprKind::Global(y)) => {
            x == y
        }
        _ => false,
    };
    matches!(idxs.as_slice(), [k] if matches!(k.kind, ExprKind::Local(s) if s == *p.slot))
        && matches!(lo.kind, ExprKind::Int(0))
        && matches!(&hi.kind, ExprKind::Builtin(Builtin::Len, a) if matches!(a.as_slice(), [b] if same(b)))
        && pure_path(base)
}

/// True when the producer's loop can stop at a push without the rest of it
/// being missed: it can neither trap nor write past its own locals. A build
/// that checks overflow (`Checks::All`, as `cpl run` and `cpl test` check)
/// runs it to the end, since any `+` there may trap.
/// `call_ok` says which calls count as pure ([`crate::uses::effects::pure_call`]).
pub(crate) fn stops_early(
    view: &View<'_>,
    checks: Checks,
    call_ok: &dyn Fn(&Expr) -> bool,
) -> bool {
    let p = view.producer();
    let p = &p;
    // A call of a held fn value (`a.map(f)` holds `f`) is a call of what it holds.
    let held_call = |e: &Expr| {
        let ExprKind::CallExpr(callee, args) = &e.kind else {
            return false;
        };
        let ExprKind::Local(s) = callee.kind else {
            return false;
        };
        view.held.iter().any(|stmt| match stmt {
            Stmt::Let(h, Some(init)) if *h == s => call_ok(&Expr {
                kind: ExprKind::CallExpr(Box::new(init.clone()), args.clone()),
                ..e.clone()
            }),
            _ => false,
        })
    };
    // An argument that is an own index (`f(a[k])`) is read as the counter, which cannot trap.
    let unindexed = |e: &Expr| match &e.kind {
        ExprKind::CallExpr(callee, args) => {
            let arg = |a: &cpl_ir::Arg| match a {
                cpl_ir::Arg::Value(v) if own_index(p, v) => cpl_ir::Arg::Value(Expr {
                    kind: ExprKind::Local(*p.slot),
                    ..v.clone()
                }),
                a => a.clone(),
            };
            Expr {
                kind: ExprKind::CallExpr(callee.clone(), args.iter().map(arg).collect()),
                ..e.clone()
            }
        }
        _ => e.clone(),
    };
    let ok = |e: &Expr| {
        let e = unindexed(e);
        own_index(p, &e) || call_ok(&e) || held_call(&e)
    };
    checks != Checks::All
        && trap_free_stmts(&p.body.stmts, *p.vec, checks != Checks::None, &ok)
        && writes_own(p.body)
}

/// Each fused read in `program`: its span, what it reads, and whether it
/// stops at the first element. `cpl explain` lists these.
pub(crate) fn fused_reads(program: &Program, checks: Checks) -> Vec<(cpl_diag::Span, Read, bool)> {
    let traps = checks != Checks::None;
    let devirt = crate::devirt::Devirt::collect(program);
    let pure = pure_fns(program, &devirt, traps);
    let call_ok = |e: &Expr| pure_call(program, &devirt, &pure, None, traps, e);
    let mut found = Vec::new();
    let mut visit = |block: &Block| {
        each_value(block, &mut |e| {
            if let Some((read, view)) = fused_read(e) {
                let stops = read == Read::First && stops_early(&view, checks, &call_ok);
                found.push((e.span, read, stops));
            }
        });
    };
    program.fns.iter().for_each(|f| visit(&f.body));
    program.closures.iter().for_each(|c| visit(&c.body));
    visit(&program.setup);
    visit(&program.main);
    found
}

impl Emitter<'_> {
    /// `e`, a `first`, `last` or `len` of a list built on the spot, as the
    /// producer's loop keeping what the read needs at each push.
    pub(crate) fn fused_view(&mut self, e: &Expr) -> Option<Code> {
        let (read, view) = fused_read(e)?;
        let p = view.producer();
        if self.hoisted.contains(p.vec) {
            return None;
        }
        let ExprKind::Builtin(_, args) = &e.kind else {
            return None;
        };
        let checks = self.target.checks;
        let traps = checks != Checks::None;
        let call_ok =
            |e: &Expr| pure_call(self.program, &self.devirt, &self.sink_pure, None, traps, e);
        let stops = read == Read::First && stops_early(&view, checks, &call_ok);
        let stop = stops.then(|| self.temp());
        let r = self.temp();
        let put = self.temp();
        let (decl, keep) = match read {
            Read::Len => (format!("ll {r} = 0;"), format!("++{r};")),
            Read::First | Read::Last => {
                self.need("first");
                let list = self.ty(&args[0].ty);
                let decl = format!("decltype(cpl::first(std::declval<const {list}&>())) {r};");
                let keep = if read == Read::First && stop.is_none() {
                    format!("if (!{r}) {r}.emplace(cpl_item);")
                } else {
                    format!("{r}.emplace(cpl_item);")
                };
                (decl, keep)
            }
        };
        let param = if read == Read::Len { "" } else { " cpl_item" };
        let mut out =
            format!("[&]() {{ {decl} auto {put} = [&](const auto&{param}) {{ {keep} }};\n");
        self.fused_pushes.push((
            *p.vec,
            FusedPush {
                var: String::new(),
                cpp: String::new(),
                first: Vec::new(),
                body: std::rc::Rc::new(Block { stmts: Vec::new() }),
                slot: None,
                sink: Some(put),
                stop: stop.clone(),
            },
        ));
        self.indent += 1;
        if view.held.is_empty() {
            self.for_stmt(*p.id, *p.slot, p.iter, p.body, &mut out);
        } else {
            // The held `let`s run first, emitted as a block so each sees the loop after it.
            let mut stmts = view.held.to_vec();
            stmts.push(Stmt::For(*p.id, *p.slot, p.iter.clone(), p.body.clone()));
            self.block(&Block { stmts }, &mut out);
        }
        self.indent -= 1;
        self.fused_pushes.pop();
        let pad = crate::emit::pad(self.indent);
        if let Some(label) = &stop {
            out.push_str(&format!("{pad}{label}:;\n"));
        }
        out.push_str(&format!("{pad}return {r}; }}()"));
        let code = Code::primary(out);
        Some(match read {
            Read::Len => code,
            Read::First | Read::Last => self.packed_result(&e.ty, code),
        })
    }
}
