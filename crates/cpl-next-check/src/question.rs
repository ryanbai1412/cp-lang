//! Postfix `x?` and its extent (R02.4.7): an `x?` reads `x`'s value, and when `x` is `None`
//! the nearest enclosing extent is `None`. `?` is never control flow.
use super::*;

/// An extent being checked: the scope depth it opened at, and each `?` operand in it, held.
pub(super) struct Extent {
    depth: usize,
    held: Vec<(VarRef, TExpr)>,
}

impl Cx<'_> {
    /// `e` as an extent wanting `want`: an optional, `None` when any `?` in it found `None`.
    pub(super) fn extent(&mut self, e: &next::Expr, want: Option<&Ty>) -> TExpr {
        let inner = want.map(|w| self.optional_of(w).unwrap_or_else(|| w.clone()));
        let depth = self.b.scopes.len();
        self.b.tries.push(Extent {
            depth,
            held: Vec::new(),
        });
        let v = self.expr(e, inner.as_ref());
        let held = self.b.tries.pop().map(|x| x.held).unwrap_or_default();
        if held.is_empty() || self.shallow(&v.ty) == Ty::Error {
            return v;
        }
        let span = v.span;
        let mk = |kind, ty| TExpr { kind, ty, span };
        let (opt, value) = match self.optional_of(&v.ty) {
            Some(_) => (v.ty.clone(), v),
            None => {
                let opt = self.optional_ty(v.ty.clone());
                (opt.clone(), self.some(v, opt))
            }
        };
        let inner = self.optional_of(&opt).unwrap_or(Ty::Error);
        let none = vec![mk(TK::Bool(false), Ty::Bool), mk(TK::Unset, inner)];
        held.into_iter().rev().fold(value, |acc, (t, x)| {
            let read = mk(TK::Var(t), x.ty.clone());
            let has = mk(TK::Field(Box::new(read), 0), Ty::Bool);
            let no = mk(TK::Struct(none.clone()), opt.clone());
            let pick = mk(
                TK::If(Box::new(has), Box::new(acc), Box::new(no)),
                opt.clone(),
            );
            mk(
                TK::Block(vec![TStmt::Let(t, x)], Some(Box::new(pick))),
                opt.clone(),
            )
        })
    }

    /// `x?` inside an extent: `x`'s value, held so the extent can test it first.
    pub(super) fn question(&mut self, x: &next::Expr, span: Span) -> TExpr {
        let open = self.b.tries.last().map(|t| t.depth);
        if open != Some(self.b.scopes.len()) {
            // A `?` with no extent of its own around it, or under a name the extent can't see.
            let kind = "Try".to_string();
            self.err(span, Unbuilt::Expression { kind });
            return Self::error_node(span);
        }
        let v = self.expr(x, None);
        let Some(inner) = self.optional_of(&v.ty) else {
            if self.shallow(&v.ty) != Ty::Error {
                let t = self.show(&v.ty);
                let text = cpl_diag::msgs::ir::TYPE_MISMATCH_NEEDS_AN_OPTIONAL_FOUND.say(&[&t]);
                self.err(span, Msg::verdict(cpl_diag::codes::TYPE_MISMATCH, text));
            }
            return Self::error_node(span);
        };
        self.b.scopes.push(HashMap::new());
        let t = self.bind("$q", v.ty.clone());
        self.b.scopes.pop();
        let read = TExpr {
            kind: TK::Var(t),
            ty: v.ty.clone(),
            span,
        };
        if let Some(open) = self.b.tries.last_mut() {
            open.held.push((t, v));
        }
        TExpr {
            kind: TK::Field(Box::new(read), 1),
            ty: inner,
            span,
        }
    }

    /// A `?` a condition tests directly gives it no value (R02.4.7).
    pub(super) fn question_in_cond(&mut self, e: &next::Expr) -> bool {
        let Some(at) = e.bare_try() else { return false };
        let span = self.at(e);
        let text = cpl_diag::msgs::check::CANNOT_USED_DIRECTLY_CONDITION.say(&[]);
        let mut msg = Msg::verdict(cpl_diag::codes::QUESTION_IN_CONDITION, text);
        msg.fixes.push(cpl_next_ir::msg::Fix {
            title: "treat `None` as false".to_string(),
            edits: vec![(
                self.b.item.at(span),
                format!("({}) ?? false", self.text(span)),
            )],
        });
        self.err(at, msg);
        true
    }

    /// `x?` as a statement: its result, the only thing `?` makes, is dropped (R02.4.7).
    pub(super) fn dropped_question(&mut self, e: &next::Expr) -> bool {
        if !matches!(e, next::Expr::Try(_)) {
            return false;
        }
        let span = self.at(e);
        let q = self.text(span).to_string();
        let text = cpl_diag::msgs::check::DROPPED_QUESTION_THE_RESULT_OF_IS.say(&[&q]);
        self.err(span, Msg::verdict(cpl_diag::codes::DROPPED_QUESTION, text));
        true
    }
}
