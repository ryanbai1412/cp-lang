//! Places: storage an expression names, and `let p = mut a[i]` bindings.

use super::*;

impl Cx<'_> {
    /// An expression that names storage: a variable, or a field of one.
    /// The name of the `let` a place lies in, if it lies in one.
    /// The `let` keyword on the line of `near`, at or before it: where a `let` binding at
    /// `near` was written.
    pub(super) fn let_keyword(&self, near: Span) -> Option<cpl_next_ir::ids::SourceId> {
        let at = self.b.item.at(near);
        let text = self.src.text_of(&at);
        let abs = text.span(&at);
        let end = (abs.start as usize + 3).min(text.text.len());
        let head = text.text.get(..end)?;
        let line = head.rfind('\n').map_or(0, |i| i + 1);
        let found = line + head[line..].rfind("let")?;
        let word = |c: Option<char>| c.is_none_or(|c| !c.is_alphanumeric() && c != '_');
        let bounded =
            word(head[..found].chars().last()) && word(text.text[found + 3..].chars().next());
        let rel = (near.start + found as u32).checked_sub(abs.start)?;
        bounded.then(|| self.b.item.at(Span::new(rel, rel + 3)))
    }

    /// The `let` that bound the root of `place`, when a `let` did.
    pub(super) fn let_root_at(&self, place: &TExpr) -> Option<cpl_next_ir::ids::SourceId> {
        let mut root = place;
        while let TK::Field(b, _) = &root.kind {
            root = b;
        }
        let TK::Var(v) = &root.kind else { return None };
        match v {
            VarRef::Local(i) => self.b.locals[*i].let_at.clone(),
            VarRef::Global(i) => self.globals[*i].let_at.clone(),
        }
    }

    /// `msg` about writing the `let` binding `name` that is the root of `place`, pointing at
    /// the `let` and offering to write `var` there.
    pub(super) fn with_var_fix(&self, place: &TExpr, name: &str, mut msg: Msg) -> Msg {
        if let Some(at) = self.let_root_at(place) {
            msg.labels.push((at.clone(), "declared here".into()));
            msg.fixes.push(cpl_next_ir::msg::Fix {
                title: format!("declare `{name}` with `var`"),
                edits: vec![(at, "var".into())],
            });
        }
        msg
    }

    pub(super) fn let_root(&self, place: &TExpr) -> Option<String> {
        let mut root = place;
        while let TK::Field(b, _) = &root.kind {
            root = b;
        }
        let TK::Var(v) = &root.kind else { return None };
        let (fixed, name) = match v {
            VarRef::Local(i) => (self.b.locals[*i].fixed, &self.b.locals[*i].name),
            VarRef::Global(i) => (self.globals[*i].fixed, &self.globals[*i].name),
        };
        (fixed && !name.starts_with('$')).then(|| name.clone())
    }

    /// `lend mut place` in a lender of `t`s: `place` is what its caller's body runs on.
    pub(super) fn lend_out(&mut self, place: &ast::Expr, t: &Ty, span: Span, out: &mut Vec<TStmt>) {
        let p = self.place(place);
        if !self.unify(&p.ty, t) {
            self.err(
                span,
                Msg::verdict(
                    cpl_diag::codes::TYPE_MISMATCH,
                    format!("expected `{}`, found `{}`", self.show(t), self.show(&p.ty)),
                ),
            );
        }
        out.push(TStmt::LendOut(p));
    }

    pub(super) fn place(&mut self, e: &ast::Expr) -> TExpr {
        let v = self.expr_as_typed(e, None);
        if !matches!(
            v.kind,
            TK::Var(_) | TK::Field(..) | TK::Slot(..) | TK::Error
        ) {
            self.err(
                e.span,
                Msg::verdict(
                    cpl_diag::codes::TYPE_MISMATCH,
                    "this is not a place that can be assigned or lent",
                ),
            );
            return v;
        }
        // A field or cell is a place only in storage a name holds: `mk().f = v` writes nothing.
        let mut root = &v;
        while let TK::Field(b, _) | TK::Slot(b, _) | TK::Payload(b, _) = &root.kind {
            root = b;
        }
        if !matches!(root.kind, TK::Var(_) | TK::Error) {
            self.err(
                e.span,
                Msg::verdict(
                    cpl_diag::codes::ASSIGN_TO_LET,
                    "only a variable, a field, or an element can be assigned to",
                ),
            );
        }
        v
    }
}

/// `let p = mut a[i]`: the name and the indexed place it names.
/// What runs inside a place binding's lend.
pub(super) type InPlace<'a> = Box<dyn FnOnce(&mut Cx<'_>, &mut Vec<TStmt>) + 'a>;

pub(super) fn place_binding(s: &ast::Stmt) -> Option<(String, &ast::Expr)> {
    let StmtKind::Let {
        pat,
        ty: None,
        init: Some(init),
        mutable: false,
        otherwise: None,
        ..
    } = &s.kind
    else {
        return None;
    };
    let (PatternKind::Name(n), ExprKind::Mut(place)) = (&pat.kind, &init.kind) else {
        return None;
    };
    let place = super::index::unparen(place);
    matches!(&place.kind, ExprKind::Index { indices, .. } if indices.len() == 1)
        .then(|| (n.clone(), place))
}

/// How many of `rest` hold the place `p` names: through the last one that mentions `p`, or a
/// name one of those binds, so every name bound inside the lend is seen only inside it.
pub(super) fn place_held(p: &str, rest: &[ast::Stmt]) -> usize {
    let mut names = vec![p.to_string()];
    let mut held = 0;
    loop {
        let seen: Vec<&str> = names.iter().map(String::as_str).collect();
        let upto = (rest.iter())
            .rposition(|s| cpl_syntax::visit::stmt_mentions_any(s, &seen))
            .map_or(0, |k| k + 1);
        if upto == held {
            return held;
        }
        for s in &rest[held..upto] {
            if let StmtKind::Let { pat, .. } = &s.kind {
                cpl_syntax::visit::pattern_names(pat, &mut names);
            }
        }
        held = upto;
    }
}
