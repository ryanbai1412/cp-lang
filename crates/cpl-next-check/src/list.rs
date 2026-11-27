//! Lists a program writes: literals and comprehensions (R01.8.1).
use super::*;

/// A list being filled: the local holding it, its element type and its `push`.
#[derive(Clone)]
pub(super) struct Filling {
    read: TExpr,
    elem: Ty,
    push: FnId,
}

impl Cx<'_> {
    /// A new empty list in local `name`, `List.new()`, of the element type `want` gives. A
    /// literal's local is `$list`, which says its length is the pushes that follow.
    fn new_list(&mut self, name: &str, want: Option<&Ty>, span: Span) -> Option<(Filling, TStmt)> {
        let list = self.list_struct(span)?;
        let ls = self.d.s(list);
        let elem = match want.map(|w| self.shallow(w)) {
            Some(Ty::Struct(d, ts, _)) if d == list => ts[0].clone(),
            // A literal where `[K]T` is expected becomes one, so its elements are `T`s; when `T`
            // is a list the literal may be that `T` filling every cell (R04.8.10), so it waits.
            Some(Ty::Struct(d, ts, _))
                if Some(d) == self.array_id()
                    && !matches!(ts.last().map(|t| self.shallow(t)), Some(Ty::Struct(l, ..)) if l == list) =>
            {
                ts.last().cloned().unwrap_or_else(|| self.new_ty())
            }
            _ => self.new_ty(),
        };
        let lty = Ty::Struct(list, vec![elem.clone()], Vec::new());
        let Some(ctor) = ls.ctor else {
            self.err(span, "internal: `List` declares no `new`");
            return None;
        };
        let new = self.fnref(ctor, vec![elem.clone()], Vec::new());
        let push = ls.methods["push"];
        let tmp = self.bind(name, lty.clone());
        let read = TExpr {
            kind: TK::Var(tmp),
            ty: lty.clone(),
            span,
        };
        let call = TK::Call {
            f: new,
            recv: None,
            args: vec![],
        };
        let init = TStmt::Let(
            tmp,
            TExpr {
                kind: call,
                ty: lty,
                span,
            },
        );
        Some((Filling { read, elem, push }, init))
    }

    /// `x` pushed onto the list being filled.
    fn push_onto(&mut self, l: &Filling, x: &ast::Expr, span: Span) -> TExpr {
        let v = self.expr(x, Some(&l.elem));
        let v = self.coerce(v, &l.elem);
        let f = self.fnref(l.push, vec![l.elem.clone()], Vec::new());
        TExpr {
            kind: TK::Call {
                f,
                recv: Some(Box::new(TArg::Mut(l.read.clone()))),
                args: vec![TArg::Value(v)],
            },
            ty: Ty::Unit,
            span,
        }
    }

    /// `[a, b]`: `List.new()` and a `push` per element.
    pub(super) fn list_literal(
        &mut self,
        xs: &[ast::Expr],
        want: Option<&Ty>,
        span: Span,
    ) -> TExpr {
        self.scopes.push(HashMap::new());
        let e = match self.new_list("$list", want, span) {
            Some((l, init)) => {
                let mut stmts = vec![init];
                for x in xs {
                    let push = self.push_onto(&l, x, span);
                    stmts.push(TStmt::Expr(push));
                }
                filled(stmts, l, span)
            }
            None => error(span),
        };
        self.scopes.pop();
        e
    }

    /// `[e for p in s if c, q in t]`: a new list, then the loop its clauses write, outermost
    /// first, each condition an `if` around the rest, pushing `e` innermost (R01.8.1). With no
    /// clauses left it is that innermost push, onto the list being filled.
    pub(super) fn comprehension(
        &mut self,
        body: &ast::Expr,
        clauses: &[ast::Clause],
        want: Option<&Ty>,
        span: Span,
    ) -> TExpr {
        if clauses.is_empty() {
            return match self.filling.last().cloned() {
                Some(l) => self.push_onto(&l, body, span),
                None => error(span),
            };
        }
        self.scopes.push(HashMap::new());
        let e = match self.new_list("$comp", want, span) {
            Some((l, init)) => {
                let mut stmts = vec![init];
                self.filling.push(l);
                self.stmt(&comprehension_loop(body, clauses, span), &mut stmts);
                let l = self.filling.pop();
                l.map_or_else(|| error(span), |l| filled(stmts, l, span))
            }
            None => error(span),
        };
        self.scopes.pop();
        e
    }
}

fn filled(stmts: Vec<TStmt>, l: Filling, span: Span) -> TExpr {
    let ty = l.read.ty.clone();
    TExpr {
        kind: TK::Block(stmts, Some(Box::new(l.read))),
        ty,
        span,
    }
}

fn error(span: Span) -> TExpr {
    TExpr {
        kind: TK::Error,
        ty: Ty::Error,
        span,
    }
}

/// The loop of a comprehension's clauses, its innermost statement the comprehension with none.
fn comprehension_loop(body: &ast::Expr, clauses: &[ast::Clause], span: Span) -> ast::Stmt {
    let expr = |kind| ast::Expr { kind, span };
    let stmt = |kind| ast::Stmt {
        kind,
        span,
        decorators: Vec::new(),
        public: false,
    };
    let block = |s: ast::Stmt| ast::Block {
        stmts: vec![s],
        span,
    };
    let mut inner = stmt(ast::StmtKind::Expr(expr(ExprKind::Comprehension {
        body: Box::new(body.clone()),
        clauses: Vec::new(),
    })));
    for c in clauses.iter().rev() {
        for cond in c.conds.iter().rev() {
            inner = stmt(ast::StmtKind::Expr(expr(ExprKind::If {
                branches: vec![(cond.clone(), block(inner))],
                otherwise: None,
            })));
        }
        // `[0 for _ in xs]`: the loop binds each element to a name nothing reads.
        let pat = match &c.pat.kind {
            PatternKind::Wildcard => ast::Pattern {
                kind: PatternKind::Name("$each".into()),
                span: c.pat.span,
            },
            _ => c.pat.clone(),
        };
        inner = stmt(ast::StmtKind::For {
            label: None,
            pat,
            iter: c.iter.clone(),
            body: block(inner),
            else_body: None,
        });
    }
    inner
}
