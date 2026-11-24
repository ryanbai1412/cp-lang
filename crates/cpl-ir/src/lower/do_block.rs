//! A `do` block's value: statements run in a frame of their own, then give.
//! The frame is opened and closed by plain fns, so each `body` closure
//! instantiates only the call between them (one copy of the frame code in
//! cpl.wasm, not one per caller).

use super::*;

impl<'a> Lowerer<'a> {
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
        let frame = self.cx.dos.pop().expect("pushed by open_do");
        self.mk(
            ExprKind::Do(Block { stmts: out }),
            frame.ty.unwrap_or(Ty::Error),
            span,
        )
    }
}
