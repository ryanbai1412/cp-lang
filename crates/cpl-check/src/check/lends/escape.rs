//! Second-class lends: a lent slice or row never outlives its block, a
//! write through a projection needs `lends mut`, and `lend` appears only
//! in a projection's body.

use super::*;
use cpl_diag::msgs;

impl<'a> Checker<'a> {
    /// Reports a lent slice or row put where it would outlive its block.
    /// `what` is the thing that would keep it: "`return`", "this
    /// assignment".
    pub(crate) fn no_escape(&mut self, e: &Expr, what: &str) {
        let Some(kind) = self.view_kind(e) else {
            return;
        };
        let src = show(e);
        let ro = self.is_readonly_view(e);
        if let Some(diag) = self.report(
            e.span,
            msgs::check::LENT_ESCAPES_WOULD_KEEP_THE_LENT,
            &[&what, &kind, &src],
        ) {
            diag.help = Some(if ro {
                format!(
                    "copy it into an owned value with `{src}.to_str()` / `{src}.to_list()`, or keep the range and re-slice where it is used"
                )
            } else {
                format!(
                    "keep the range or the row index (a plain value) and re-project where it is used, or keep a copy: `{src}.clone()`"
                )
            });
        }
    }

    /// True when a parameter of type `ty` is passed by lend rather than copied.
    pub(super) fn is_lent_param_ty(&mut self, ty: &Ty) -> bool {
        matches!(ty, Ty::Str | Ty::Fn { .. }) || self.is_heap_type(ty)
    }

    /// Assigning a lent param or heap `for`-loop variable into an outer `var`
    /// is E0603 (R05.7.4 / R05.7.10 / R05.9.2).
    pub(crate) fn no_escape_assign(&mut self, value: &Expr, target_depth: usize) {
        let e = unparen(value);
        let ExprKind::Name(src) = &e.kind else {
            return;
        };
        let Some(local) = self.local(src).cloned() else {
            return;
        };
        let src_depth = self.var_depth(src);
        if local.keyword == "param" && self.is_lent_param_ty(&local.ty) && target_depth < src_depth
        {
            self.report(
                e.span,
                msgs::check::LENT_ESCAPES_LENT_PARAMETER_CANNOT_BE,
                &[&src],
            );
        } else if local.keyword == "for" && self.is_heap_type(&local.ty) && target_depth < src_depth
        {
            self.report(e.span, msgs::check::LENT_ESCAPES_LOOP_ITEM_IS_LENT, &[&src]);
        }
    }

    /// The arguments of a call that stores them must not be lent views.
    pub(crate) fn no_escape_args(&mut self, callee: &Expr, callee_ty: &Ty, args: &[Arg]) {
        let mut receiver_is_mut_param = false;
        let mut moves_in = false;
        let what = match (&callee.kind, callee_ty) {
            (ExprKind::Field { base, name }, _) if STORES.contains(&name.text.as_str()) => {
                // A container stores its `.push`/`.insert`/`.set`/`.add` argument when
                // the method takes one of the type's own parameters, `push(x: T)`;
                // `Trie<A>.add(word: Seq<A>)` only reads the sequence it is given.
                let base_ty = self.quietly(|c| c.infer(base));
                let stores_elem = match &base_ty {
                    Ty::Named { name: ty_name, .. } => {
                        matches!(
                            ty_name.as_str(),
                            "List" | "Set" | "HashSet" | "Map" | "HashMap"
                        ) || self.method_takes_type_param(ty_name, &name.text)
                    }
                    _ => true,
                };
                if !stores_elem {
                    return;
                }
                moves_in = matches!(base_ty, Ty::Named { .. });
                if let ExprKind::Name(b) = &unparen(base).kind {
                    if let Some(local) = self.local(b) {
                        receiver_is_mut_param = local.keyword == "param" && local.mutable;
                    }
                }
                format!("`.{}(...)`", name.text)
            }
            (_, Ty::TypeName(name) | Ty::Named { name, .. }) => {
                if matches!(name.as_str(), "List" | "Str") {
                    return;
                }
                format!("storing it in a `{name}`")
            }
            _ => return,
        };
        for arg in args {
            self.escape_unless_kept(&arg.value, &what, Keep::In(callee));
            if receiver_is_mut_param {
                if let ExprKind::Name(src) = &unparen(&arg.value).kind {
                    if let Some(local) = self.local(src).cloned() {
                        if local.keyword == "param" && self.is_lent_param_ty(&local.ty) {
                            self.report(
                                arg.value.span,
                                msgs::check::LENT_ESCAPES_CANNOT_STORE_LENT_PARAMETER,
                                &[&what, &src],
                            );
                        }
                    }
                }
            }
        }
        // Put in a container, an owned heap value moves in (R05.2.2).
        if moves_in {
            self.move_ctor_args(args);
        }
    }

    /// A write through `target` needs `lends mut` at every projection on
    /// the way, and a `var` under a slice.
    pub(crate) fn writable(&mut self, target: &Expr) {
        // Direct range-place updates (`a[l..r] = 0`, `a[l..r] min= b[l..r]`)
        // write into the underlying `var` list/array; views (`let v = a[l..r]`,
        // `a[l..r][0]`, `a.rev`, `s[l..r]`) remain read-only.
        let direct_list_range = matches!(
            &target.kind,
            ExprKind::Index { indices, .. }
                if indices.iter().any(|i| matches!(i.kind, ExprKind::Range { .. }))
        ) && Self::root_name(target).is_some_and(|(name, _)| {
            let ty = self.local(name).map(|l| l.ty.clone()).or_else(|| {
                match self.global(name).as_deref() {
                    Some(Def::Value { ty, .. }) => Some(ty.clone()),
                    _ => None,
                }
            });
            matches!(ty, Some(Ty::Named { name, .. }) if name == "List")
        });
        let mut e = target;
        let mut first = true;
        loop {
            if !(first && direct_list_range) && self.is_readonly_view(e) {
                let kind = self.view_kind(e).unwrap_or("view");
                if let Some(diag) = self.report(
                    target.span,
                    msgs::check::ASSIGN_TO_LET_IS_A_READ_ONLY,
                    &[&show(e), &kind],
                ) {
                    diag.help = Some(
                        "copy it into an owned value first with `.to_str()` or `.to_list()`".into(),
                    );
                }
                return;
            }
            first = false;
            e = match &e.kind {
                ExprKind::Paren(inner) | ExprKind::Unwrap(inner) => inner,
                ExprKind::Field { base, .. } | ExprKind::Index { base, .. } => base,
                ExprKind::Call { callee, .. } => {
                    let ExprKind::Field { base, .. } = &callee.kind else {
                        return;
                    };
                    if let Some(call) = self.lends.proj_calls.get(&e.span).cloned() {
                        if !call.mutable {
                            if let Some(diag) = self.report(
                                e.span,
                                msgs::check::ASSIGN_TO_LET_IS_LENT_READ_ONLY,
                                &[&show(e), &call.who, &call.lent],
                            ) {
                                diag.help = Some(format!(
                                    "declare it `lends mut {}` to allow writes through it",
                                    call.lent
                                ));
                            }
                            return;
                        }
                    }
                    base
                }
                _ => return,
            };
        }
    }

    /// `a[l..=r].sort()` changes `a`: it must be a `var`.
    pub(crate) fn mutates_view(&mut self, call: &Expr, callee: &Expr) {
        let ExprKind::Field { base, name } = &callee.kind else {
            return;
        };
        let ExprKind::Call { args, .. } = &call.kind else {
            return;
        };
        let base_ty = self.quietly(|c| c.infer(base));
        let own = self.own_mutator(&base_ty, &name.text);
        let by_row = self.row_mutator(&base_ty, &name.text, args);
        if !own && !by_row {
            return;
        }
        if name.text == "sort" {
            if let ExprKind::Name(n) = &base.kind {
                if matches!(self.local(n).map(|b| &b.ty), Some(Ty::Domain(_)))
                    || matches!(self.global(n).as_deref(), Some(Def::Domain { .. }))
                {
                    return;
                }
            }
        }
        // `xs.push(x)` changes `xs` as an assignment to it would.
        self.check_access(base, true);
        // A type's own `mut self` method says why its receiver can't change.
        if own && !by_row {
            return;
        }
        let viewed = self.view_kind(base).is_some();
        if let Some(kind) = self.view_kind(base) {
            if self.is_readonly_view(base) {
                if let Some(diag) = self.report(
                    base.span,
                    msgs::check::ASSIGN_TO_LET_IS_A_READ_ONLY_2,
                    &[&show(base), &kind, &show(call)],
                ) {
                    diag.help = Some(
                        "copy it into an owned value first with `.to_str()` or `.to_list()`".into(),
                    );
                }
                return;
            }
            if kind == "slice" && LENGTH_CHANGING_MUTATORS.contains(&name.text.as_str()) {
                if let Some(diag) = self.report(
                    base.span,
                    msgs::check::ASSIGN_TO_LET_IS_A_FIXED_LENGTH,
                    &[&show(base), &show(call)],
                ) {
                    diag.help = Some("copy it into an owned list first with `.to_list()`".into());
                }
                return;
            }
        }
        let Some(place) = self.place(base) else {
            return;
        };
        // A `let`-bound lend was checked where it was bound.
        if place.root_mutable || place.via.is_some() {
            return;
        }
        let root = place.root;
        if self.cached_fields.contains(&root) {
            return;
        }
        let decl = self.let_decl(&root);
        let message = cpl_diag::msgs::check::NOT_A_VAR_CANT_CHANGE.at(&[&root, &show(call)]);
        // R07.11.1: an in-place method needs a `var` source.
        let code = if viewed {
            codes::ASSIGN_TO_LET
        } else {
            codes::MUT_OF_IMMUTABLE
        };
        if let Some(diag) = self.report_as(code, base.span, message) {
            super::declare_with_var(diag, &root, decl);
        }
    }

    /// Checks the shape of `fn name(self, ..) lends [mut] T`: one `lend`
    /// of a part of `self`, as a statement of the body itself. What
    /// follows it is the write-back, which runs when the caller is done
    /// with the place and so needs `mut self`.
    pub(crate) fn projection_shape(&mut self, f: &ast::FnDecl, fields: &[String]) {
        let Some(lends) = &f.lends else {
            return;
        };
        // A static fn that lends is a `lends` fn like a free one (R05.5.2).
        let receiver = f.params.first().filter(|p| p.name.text == "self");
        if receiver.is_none() {
            return;
        }
        let name = &f.name.text;
        let stmts: &[Stmt] = match &f.body {
            ast::FnBody::Block(block) => &block.stmts,
            _ => &[],
        };
        let mut found = stmts
            .iter()
            .enumerate()
            .filter_map(|(at, stmt)| match &stmt.kind {
                StmtKind::Lend { mutable, place } => {
                    Some((stmts.get(at + 1), Some(stmt.span), *mutable, place))
                }
                _ => None,
            });
        // `fn lo(mut self) -> lends mut Int = self.xs[0]` lends its body as declared.
        let single = match &f.body {
            ast::FnBody::Expr(place) => Some((None, None, lends.mutable, place)),
            _ => found.next().filter(|_| found.next().is_none()),
        };
        let Some((after, lend_at, mutable, place)) = single else {
            if let Some(diag) = self.report(
                f.name.span,
                msgs::check::BAD_PROJECTION_IS_DECLARED_LENDS_SO,
                &[&name],
            ) {
                diag.help = Some(
                    "give the body one `lend <place>`, the part of `self` it lends, as a statement of its own"
                        .into(),
                );
            }
            self.lends.lend_reported = true;
            return;
        };
        if let Some(after) = after {
            if receiver.is_some_and(|p| !p.mutable) {
                let message = cpl_diag::msgs::check::CODE_AFTER_LEND_CHANGES.at(&[&name]);
                self.report_said(after.span, message);
            }
        }
        self.lends.lend_at = lend_at;
        let span = lend_at.unwrap_or(place.span);
        if mutable && !lends.mutable {
            let message = cpl_diag::msgs::check::DECLARED_LENDS_WITHOUT_MUT.at(&[&name]);
            if let Some(diag) = self.report_said(span, message) {
                diag.help = Some("declare it `lends mut`".into());
            }
        } else if !mutable && lends.mutable {
            self.report(
                span,
                msgs::check::BAD_PROJECTION_IS_DECLARED_LENDS_MUT,
                &[&name, &show(place)],
            );
        }
        let reaches = |e: &Expr| {
            root_name(e).is_some_and(|root| root == "self" || fields.iter().any(|f| f == root))
        };
        // `SortedRange(self, ..)`: a view built around `self` lends `self`.
        let lends_self = |e: &Expr| {
            reaches(e)
                || matches!(&e.kind, ExprKind::Call { callee, args }
                    if matches!(callee.kind, ExprKind::Name(_)) && args.iter().any(|a| reaches(&a.value)))
        };
        let mut foreign = Vec::new();
        super::foreign_lends(place, &lends_self, &mut foreign);
        for place in foreign {
            self.report(
                place.span,
                msgs::check::BAD_PROJECTION_A_PROJECTION_MAY_ONLY,
                &[&show(place)],
            );
        }
    }

    /// `lend` anywhere but in the body of a projection, as one of its own
    /// statements.
    pub(crate) fn lend_in_place(&mut self, stmt: &Stmt) {
        if self.lends.lend_at != Some(stmt.span) && !self.lends.lend_reported {
            self.report(
                stmt.span,
                cpl_diag::msgs::check::LEND_STATEMENT_PROJECTION_S,
                &[],
            );
        }
    }
}
