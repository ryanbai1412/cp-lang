//! `i + 1` and other `?I` values used where a position of `I` is wanted.

use super::*;
use cpl_syntax::ast::{Expr, ExprKind, Name};

impl Checker<'_> {
    /// R11.1.4: E0207 when `base.name` reads a member another module declares
    /// without `pub`; a C++ name (`push_back`) points at the public member it means.
    pub(crate) fn private_member(&mut self, base_ty: &Ty, name: &Name) -> bool {
        let Ty::Named { name: owner, .. } = base_ty else {
            return false;
        };
        if !self.env.is_private_field(owner, &name.text) {
            return false;
        }
        let public = self.public_foreign_twin(owner, &name.text);
        let message = format!(
            "`{}` is private to the module that declares `{owner}`",
            name.text
        );
        if let Some(diag) = self.error(codes::PRIVATE_NAME, name.span, message) {
            if let Some(own) = public {
                diag.help = Some(format!("`{owner}`'s public member is `{own}`"));
                diag.fix(format!("write `{own}`"), name.span, own);
            } else {
                diag.help = Some(format!(
                    "mark it `pub` where `{owner}` declares it (R11.1.4)"
                ));
            }
        }
        true
    }

    /// `push` for `push_back` when `owner` has a public `push`.
    pub(super) fn public_foreign_twin(&mut self, owner: &str, name: &str) -> Option<&'static str> {
        let own = crate::builtins::foreign_own(name)?;
        let members = self.members_of(owner)?;
        let has = members.members.iter().any(|(m, _)| m == own);
        (has && !self.env.is_private_field(owner, own)).then_some(own)
    }

    /// The fix for an unknown method: its `_by` twin (`max_by`), or the free
    /// function it names (`mask.popcount()` is `popcount(mask)`).
    pub(super) fn method_twin(
        &self,
        base: &Expr,
        name: &Name,
        base_ty: &Ty,
    ) -> Option<(String, cpl_diag::Fix)> {
        if let Some(found) = super::advice::index_of(self.src, base.span, name, base_ty) {
            return Some(found);
        }
        let twin = super::advice::by_twin(self.src, &name.text, name.span);
        let free = crate::builtins::prelude_names().iter().any(|(f, kind)| {
            *f == name.text
                && matches!(kind, crate::PreludeKind::Value | crate::PreludeKind::Std(_))
        });
        let takes_value = !["read", "flush", "rand", "some"]
            .iter()
            .any(|r| name.text.starts_with(r));
        if twin.is_some() || !free || !takes_value {
            return twin;
        }
        let recv = self.quote(base.span);
        let after = &self.src[name.span.end as usize..];
        let (to, call) = match after.strip_prefix('(') {
            Some(rest) if rest.trim_start().starts_with(')') => {
                let close = name.span.end + 1 + rest.find(')')? as u32 + 1;
                (close, format!("{}({recv})", name.text))
            }
            Some(_) => (name.span.end + 1, format!("{}({recv}, ", name.text)),
            None => (name.span.end, format!("{}({recv})", name.text)),
        };
        let shown = if call.ends_with(", ") {
            format!("{call}..)")
        } else {
            call.clone()
        };
        let help = format!(
            "`{}` is a function, not a method: write `{shown}`",
            name.text
        );
        let fix = cpl_diag::Fix {
            title: format!("write `{shown}`"),
            edits: vec![(cpl_diag::Span::new(base.span.start, to), call)],
        };
        Some((help, fix))
    }

    /// R07.1.7: parentheses mark work, so `xs.sorted` reads as O(1) but isn't.
    pub(super) fn missing_parens(
        &mut self,
        expr: &Expr,
        base: &Expr,
        name: &Name,
        owner: &str,
        ty: &Ty,
    ) {
        if name.text.is_empty() {
            return;
        }
        let Some(member) = crate::builtins::builtin_member(owner, &name.text) else {
            return self.uncalled_method(expr, base, name, owner, ty);
        };
        if member.kind != crate::builtins::MemberKind::Method
            || member.needs_arguments()
            || member.ret.is_empty()
        {
            return;
        }
        if super::advice::called_next(self.src.get(expr.span.end as usize..).unwrap_or("")) {
            return;
        }
        let shown = format!("{}.{}", self.quote(base.span), name.text);
        let message = format!("`{shown}` does work, but without `()` it reads like an O(1) field");
        self.sink.push(
            Diagnostic::warning(codes::PARENS_MEAN_WORK, expr.span, message)
                .with_help(format!("write `{shown}()`: `()` marks a call that does work, and only O(1) getters read as fields")),
        );
    }

    /// R07.9.2: `s.fold(f)` with a plain fn has no identity, so it is `?T`.
    pub(super) fn fold_needs_identity(&mut self, init: &Expr, declared: &Ty) {
        if matches!(declared, Ty::Optional(_) | Ty::Error) {
            return;
        }
        let ExprKind::Call { callee, args } = &crate::check::lends::unparen(init).kind else {
            return;
        };
        let ExprKind::Field { name, .. } = &callee.kind else {
            return;
        };
        let [arg] = args.as_slice() else {
            return;
        };
        if name.text != "fold"
            || arg.name.is_some()
            || !matches!(arg.value.kind, ExprKind::Lambda { .. })
            || matches!(self.quote(arg.value.span).as_str(), "+" | "*" | "^")
        {
            return;
        }
        let shown = self.quote(init.span);
        let message = format!(
            "`{shown}` is {}: a fold by a lambda has no identity, so an empty sequence gives `None`",
            Ty::Optional(Box::new(declared.clone())).with_article()
        );
        if let Some(diag) = self.error(codes::EMPTY_FOLD_NO_IDENTITY, init.span, message) {
            diag.help = Some(format!("unwrap: `{shown}!`, or `?? d` for a default"));
            diag.fix("unwrap it with `!`", Span::point(init.span.end), "!");
        }
    }

    /// `xs.copy()`, the spelling before `xs.clone()` (R05, decision in spec/core/05-ownership.md).
    pub(super) fn old_copy_spelling(&mut self, base: &Ty, name: &Name) -> bool {
        let owned = matches!(base, Ty::Str | Ty::Array { .. })
            || matches!(base, Ty::Named { name, .. } if matches!(name.as_str(), "List" | "Map" | "Set"));
        if name.text != "copy" || !owned {
            return false;
        }
        let message = "`.copy()` is spelled `.clone()` since 0.3".to_string();
        if let Some(diag) = self.error(codes::REMOVED_SYNTAX, name.span, message) {
            diag.help = Some(
                "an owned copy is `.clone()`; `cpl migrate` rewrites every `.copy()` in a folder"
                    .into(),
            );
            diag.fix("write `clone`", name.span, "clone");
        }
        true
    }

    pub(super) fn optional_compare(
        &mut self,
        op_str: &str,
        (left_expr, left): (&Expr, &Ty),
        (right_expr, right): (&Expr, &Ty),
    ) {
        let unwrap = |c: &Self, e: &Expr, ty: &Ty| match (&e.kind, ty) {
            (_, ty) if !matches!(ty, Ty::Optional(_)) => c.quote(e.span),
            (ExprKind::Name(_) | ExprKind::Field { .. }, _) => format!("{}!", c.quote(e.span)),
            _ => format!("({})!", c.quote(e.span)),
        };
        let (at, ty) = match left {
            Ty::Optional(_) => (left_expr, left),
            _ => (right_expr, right),
        };
        let message = format!(
            "`{}` is {}: optional values can't be ordered with `{op_str}`",
            self.quote(at.span),
            ty.with_article()
        );
        let (l, r) = (
            unwrap(self, left_expr, left),
            unwrap(self, right_expr, right),
        );
        if let Some(diag) = self.error(codes::OPTIONAL_COMPARE, at.span, message) {
            diag.help = Some(format!(
                "`None` has no order: unwrap first, `{l} {op_str} {r}`, or give a default with `??`"
            ));
        }
    }

    pub(super) fn with_not_first(&mut self, span: Span) {
        let message =
            "a bare `with` line comes first in its block; indent what it covers under `with ...:`";
        if let Some(diag) = self.error(codes::WITH_NOT_FIRST, span, message.to_string()) {
            diag.help = Some(super::advice::WITH_FIRST_HELP.into());
        }
    }

    /// Whether the step `e`, a `?I`, is known to stay inside `I`: a test
    /// or a loop around it orders the name it steps from against another
    /// position of `I` ([`cpl_syntax::order`]).
    pub(super) fn step_stays(&self, e: &Expr, domain: &Ty) -> bool {
        matches!(domain, Ty::Pos(_))
            && self.steps.bounds(e, 0).into_iter().any(|name| {
                match cpl_syntax::order::end_domain(name) {
                    // `i > D.first`: an end of the domain itself.
                    Some(of) => matches!(domain, Ty::Pos(d) if d == of),
                    None => self.local(name).is_some_and(|local| local.ty == *domain),
                }
            })
    }

    /// `i + 1` used where its domain is wanted: the message for indexing
    /// with it. A step is a `?I`, `none` when it leaves `I`.
    pub(super) fn unhandled_step(&self, got: &Ty, want: &Ty, at: &Expr) -> Option<String> {
        let Ty::Optional(inner) = got else {
            return None;
        };
        // `let u = xs.pop()`: a name holds the `?V`, it did not step.
        let may = match at.kind {
            ExprKind::Name(_) => "may be `None`".into(),
            _ => format!("may step outside {inner}"),
        };
        (matches!(**inner, Ty::Pos(_)) && inner.fits(want)).then_some(may)
    }

    pub(super) fn step_help(&self, at: &Expr) -> String {
        let t = self.quote(at.span);
        let (default, unwrap) = match at.kind {
            ExprKind::Name(_) => (format!("a default with `{t} ?? d`"), format!("{t}!")),
            _ => ("the edge a value with `??`".into(), format!("({t})!")),
        };
        format!("bind it with `if let j = {t}`, give {default}, or assert with `{unwrap}`")
    }
}
