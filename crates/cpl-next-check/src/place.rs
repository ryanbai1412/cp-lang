//! Places: storage an expression names, and `let p = mut a[i]` bindings.

use super::*;
use cpl_next_syntax::visit;

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
        let local = match v {
            VarRef::Local(i) => &self.b.locals[*i],
            VarRef::Global(i) => &self.globals[*i],
        };
        (local.fixed && !local.synthetic).then(|| local.name.clone())
    }

    /// `lend mut place` in a lender of `t`s: `place` is what its caller's body runs on.
    pub(super) fn lend_out(
        &mut self,
        place: &next::Expr,
        t: &Ty,
        span: Span,
        out: &mut Vec<TStmt>,
    ) {
        let p = self.place(place);
        self.lend_out_value(p, t, span, out);
    }

    /// `lend mut` of the checked place `p`, as `lend_out` and a lent cell (`lend mut s[k]`) do.
    pub(super) fn lend_out_value(&mut self, p: TExpr, t: &Ty, span: Span, out: &mut Vec<TStmt>) {
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

    pub(super) fn place(&mut self, e: &next::Expr) -> TExpr {
        let span = self.at(e);
        let v = self.expr_as_typed(e, None);
        if !matches!(
            v.kind,
            TK::Var(_) | TK::Field(..) | TK::Slot(..) | TK::Error
        ) {
            self.err(
                span,
                Msg::verdict(
                    cpl_diag::codes::TYPE_MISMATCH,
                    "this is not a place that can be assigned or lent",
                ),
            );
        }
        v
    }
}

/// `let p = mut a[i]`: the name and the indexed place it names.
/// What runs inside a place binding's lend.
pub(super) type InPlace<'a> = Box<dyn FnOnce(&mut Cx<'_>, &mut Vec<TStmt>) + 'a>;

pub(super) fn place_binding(s: &next::Stmt) -> Option<(String, next::Expr)> {
    let next::Stmt::Let(l) = s else {
        return None;
    };
    if l.ty().is_some() || l.is_var() || l.else_clause().is_some() {
        return None;
    }
    let mut pats = l.patterns();
    let (Some(p), None) = (pats.next(), pats.next()) else {
        return None;
    };
    let name = super::plain_name(&p)?;
    let Some(next::Expr::Mut(m)) = l.value() else {
        return None;
    };
    let place = super::index::unparen(m.inner()?);
    matches!(&place, next::Expr::Index(i) if i.indices().count() == 1).then(|| (name, place))
}

/// How many of `rest` hold the place `p` names: through the last one that mentions `p`, or a
/// name one of those binds, so every name bound inside the lend is seen only inside it.
pub(super) fn place_held<N: AstNode>(p: &str, rest: &[N]) -> usize {
    let mut names = vec![p.to_string()];
    let mut held = 0;
    loop {
        let seen: Vec<&str> = names.iter().map(String::as_str).collect();
        let upto = (rest.iter())
            .rposition(|s| visit::mentions_any(s.syntax(), &seen))
            .map_or(0, |k| k + 1);
        if upto == held {
            return held;
        }
        for s in &rest[held..upto] {
            let n = s.syntax();
            let lets = next::LetStmt::cast(n.clone())
                .or_else(|| n.children().find_map(next::LetStmt::cast));
            for pat in lets.iter().flat_map(next::LetStmt::patterns) {
                visit::pattern_names(&pat, &mut names);
            }
        }
        held = upto;
    }
}
