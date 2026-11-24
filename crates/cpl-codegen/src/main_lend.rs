//! A top-level `let` only `main` reads is a local of `solve()`. When it
//! names a heap part of a variable nothing changes while it lives
//! (`let r = rows[1]`), the C++ binds a reference there instead of
//! declaring the variable up front and copying into it (R05.3.5).

use crate::emit::Emitter;
use crate::walk;
use cpl_ir::{AssignOp, Expr, ExprKind, Place, PlaceKind, Stmt, Ty};

impl Emitter<'_> {
    /// Whether the global `id`, a local of `solve()`, is set only by one
    /// statement of `main` whose value can be lent.
    pub(crate) fn lends_main_let(&self, id: usize) -> bool {
        let stmts = &self.program.main.stmts;
        let sets = |stmt: &Stmt| matches!(stmt, Stmt::Assign(p, AssignOp::Set, _) if matches!(p.kind, PlaceKind::Global(g) if g == id));
        let Some(k) = stmts.iter().position(sets) else {
            return false;
        };
        let Stmt::Assign(_, _, init) = &stmts[k] else {
            return false;
        };
        let viewed = matches!(
            init.kind,
            ExprKind::Field(..)
                | ExprKind::Payload(..)
                | ExprKind::Index(..)
                | ExprKind::Unwrap(..)
        );
        if !viewed || init.ty.is_scalar() || matches!(init.ty, Ty::Fn(..)) {
            return false;
        }
        let Some((global, root)) = crate::stmt::place_root(init) else {
            return false;
        };
        let rest = &stmts[k + 1..];
        let tail = self.tail(rest);
        let t = &tail.touch;
        let calls_write = t
            .calls
            .iter()
            .any(|&f| global && self.uses.effects(f).writes.contains(&root));
        !t.opaque
            && !calls_write
            && !t.writes.contains(&(true, id))
            && !t.writes.contains(&(global, root))
            && !(if global {
                reads_global_whole(rest, root)
            } else {
                tail.whole.contains(&root)
            })
    }

    /// `let r = rows[1]` at the top of `main`, for a global
    /// [`Emitter::lends_main_let`] chose: a reference into the place.
    pub(crate) fn let_lent_main(&mut self, place: &Place, value: &Expr, out: &mut String) -> bool {
        let PlaceKind::Global(id) = place.kind else {
            return false;
        };
        if !self.lent_main.contains(&id) || self.fn_id.is_some() || self.closure_id.is_some() {
            return false;
        }
        let cpp = self.ty_for_loc(&self.program.globals[id].ty, cpl_ir::ElemLoc::Global(id));
        let name = self.place(place);
        let text = self.expr(value).text;
        self.line(out, &format!("const {cpp}& {name} = {text};"));
        true
    }
}

/// Whether `stmts` read the global `root` whole, not only through an
/// index, field or length: the C++ may move it then.
fn reads_global_whole(stmts: &[Stmt], root: usize) -> bool {
    let block = cpl_ir::Block {
        stmts: stmts.to_vec(),
    };
    let global = |e: &Expr| matches!(e.kind, ExprKind::Global(g) if g == root);
    let (mut uses, mut parts) = (0, 0);
    walk::deep_exprs(&block, &mut |e| match &e.kind {
        ExprKind::Global(g) if *g == root => uses += 1,
        ExprKind::Index(base, _)
        | ExprKind::Field(base, _)
        | ExprKind::Payload(base, ..)
        | ExprKind::IsVariant(base, _)
            if global(base) =>
        {
            parts += 1
        }
        ExprKind::Builtin(cpl_ir::Builtin::Len, args) if args.len() == 1 && global(&args[0]) => {
            parts += 1
        }
        _ => {}
    });
    walk::deep_stmts(&block, &mut |stmt| {
        if let Stmt::For(_, _, cpl_ir::Iter::Seq(seq, _), _) = stmt {
            parts += usize::from(global(seq));
        }
    });
    uses > parts
}
