//! A loop's label and its `else:` (R01.7.2): a flag per such loop, which a `break` that leaves
//! it sets. The `else` runs when the flag is still clear; a labelled break from inside inner
//! loops leaves each of them in turn, so their `else`s are skipped too.
use super::*;

pub(super) struct Frame {
    label: Option<String>,
    flag: VarRef,
    /// `Cx::loops` inside the loop.
    depth: u32,
    /// How many `break`s leave it.
    hits: u32,
}

impl Cx<'_> {
    pub(super) fn framed_loop(&mut self, s: &ast::Stmt, out: &mut Vec<TStmt>) {
        let span = s.span;
        let mut inner = s.clone();
        let (label, else_body) = match &mut inner.kind {
            StmtKind::For {
                label, else_body, ..
            } => (label.take().map(|l| l.text), else_body.take()),
            StmtKind::While { else_body, .. } => (None, else_body.take()),
            _ => (None, None),
        };
        let framed = label.is_some() || else_body.is_some();
        if framed {
            let flag = self.bind(&format!("$broke{}", span.start), Ty::Bool);
            out.push(TStmt::Let(flag, boolean(false, span)));
            self.loop_frames.push(Frame {
                label,
                flag,
                depth: self.loops + 1,
                hits: 0,
            });
        }
        let before: Vec<u32> = self.loop_frames.iter().map(|f| f.hits).collect();
        let was = self.framed.replace(span.start);
        self.stmt(&inner, out);
        self.framed = was;
        let own = if framed { self.loop_frames.pop() } else { None };
        // A labelled break from in here to a loop around it leaves the loop this sits in too.
        let left: Vec<VarRef> = (self.loop_frames.iter().zip(&before))
            .filter(|(f, b)| f.hits > **b)
            .map(|(f, _)| f.flag)
            .collect();
        for flag in left {
            let then = TExpr {
                kind: TK::Block(vec![TStmt::Break], None),
                ty: Ty::Unit,
                span,
            };
            out.push(TStmt::Expr(when(read(flag, span), then, span)));
        }
        if let (Some(f), Some(e)) = (own, else_body) {
            let b = self.block(&e, None);
            let clear = TExpr {
                kind: TK::Not(Box::new(read(f.flag, span))),
                ty: Ty::Bool,
                span,
            };
            out.push(TStmt::Expr(when(clear, b, span)));
        }
    }
    /// `break`, or `break 'name` to the loop of that label around it.
    pub(super) fn break_to(&mut self, label: Option<&ast::Name>, span: Span, out: &mut Vec<TStmt>) {
        let at = match label {
            None => (self.loop_frames.last())
                .filter(|f| f.depth == self.loops)
                .map(|_| self.loop_frames.len() - 1),
            Some(l) => {
                let at =
                    (self.loop_frames.iter()).rposition(|f| f.label.as_deref() == Some(&l.text));
                if at.is_none() {
                    let code = cpl_diag::codes::UNRESOLVED_NAME;
                    let text = format!("no loop around this `break` is labelled `{}`", l.text);
                    return self.err(span, Msg::verdict(code, text));
                }
                at
            }
        };
        if let Some(i) = at {
            self.loop_frames[i].hits += 1;
            let flag = self.loop_frames[i].flag;
            out.push(TStmt::Assign(read(flag, span), boolean(true, span)));
        }
        out.push(TStmt::Break);
    }
}

fn boolean(b: bool, span: Span) -> TExpr {
    TExpr {
        kind: TK::Bool(b),
        ty: Ty::Bool,
        span,
    }
}
fn read(v: VarRef, span: Span) -> TExpr {
    TExpr {
        kind: TK::Var(v),
        ty: Ty::Bool,
        span,
    }
}
/// `if cond: then`, with no value.
fn when(cond: TExpr, then: TExpr, span: Span) -> TExpr {
    let none = TExpr {
        kind: TK::Block(Vec::new(), None),
        ty: Ty::Unit,
        span,
    };
    TExpr {
        kind: TK::If(Box::new(cond), Box::new(then), Box::new(none)),
        ty: Ty::Unit,
        span,
    }
}
