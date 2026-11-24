//! A `do` block's value: statements run in a frame of their own, then give.
//! The frame is opened and closed by plain fns, so each `body` closure
//! instantiates only the call between them (one copy of the frame code in
//! cpl.wasm, not one per caller).

use super::*;

impl<'a> Lowerer<'a> {
    /// `e` as an operand that runs again or not at all (a `while` test, the
    /// right of `and`, `or` and `??`): what it needs first runs with it,
    /// each time it is worked out, and not once ahead of the statement.
    pub(super) fn worked_out_here(
        &mut self,
        span: Span,
        lower: impl FnOnce(&mut Self) -> Expr,
    ) -> Expr {
        let outer = std::mem::take(&mut self.cx.pending);
        let value = lower(self);
        let needs = std::mem::replace(&mut self.cx.pending, outer);
        if needs.is_empty() || value.ty == Ty::Error {
            self.cx.pending.extend(needs);
            return value;
        }
        let ty = value.ty.clone();
        self.do_expr(Some(&ty), span, |_, out| {
            out.extend(needs);
            out.push_give(value);
        })
    }

    pub(super) fn do_expr(
        &mut self,
        hint: Option<&Ty>,
        span: Span,
        body: impl FnOnce(&mut Self, &mut Vec<Stmt>),
    ) -> Expr {
        self.open_do(hint);
        let mut out = Vec::new();
        body(self, &mut out);
        self.close_do(out, span)
    }

    fn open_do(&mut self, hint: Option<&Ty>) {
        self.cx.dos.push(DoFrame {
            ty: hint.cloned(),
            loops: self.cx.loops.len(),
            open: false,
        });
    }

    fn close_do(&mut self, out: Vec<Stmt>, span: Span) -> Expr {
        let ty = self.cx.dos.pop().and_then(|frame| frame.ty);
        self.mk(
            ExprKind::Do(Block { stmts: out }),
            ty.unwrap_or(Ty::Error),
            span,
        )
    }
}
