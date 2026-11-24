//! Compile-time reflection: `for f in fields(x):`, the values of
//! `fields(..)`, and the records a `with` context sets.

use super::*;
use cpl_diag::msgs;

impl Checker<'_> {
    /// The values of `fields(a, b, ..)` in a `for` head: the compile-time
    /// loop over a struct's or a tuple's fields (decision 28.1).
    pub(crate) fn reflected_values<'e>(
        &mut self,
        iter: &'e Expr,
    ) -> Option<&'e [cpl_syntax::ast::Arg]> {
        let ExprKind::Call { callee, args } = &iter.kind else {
            return None;
        };
        let ExprKind::Name(name) = &callee.kind else {
            return None;
        };
        (name == "fields"
            && self.local(name).is_none()
            && self.global(name).is_none()
            && !args.is_empty()
            && args.iter().all(|a| a.name.is_none()))
        .then_some(args.as_slice())
    }

    /// The fields of a value of type `ty`, by name, in declaration order;
    /// `Ok(None)` when the type is not known until the code is instantiated.
    pub(super) fn variant_payloads(&mut self, ty: &Ty) -> Option<Vec<Vec<Ty>>> {
        let Ty::Named { name, .. } = ty else {
            return None;
        };
        match self.global(name).as_deref() {
            Some(Def::Enum { variants, .. }) => {
                Some(variants.iter().map(|(_, fs)| fs.clone()).collect())
            }
            _ => None,
        }
    }

    pub(crate) fn reflect(&mut self, ty: &Ty) -> Result<Option<Vec<(String, Ty)>>, ()> {
        match ty {
            Ty::Tuple(items) => Ok(Some(
                items
                    .iter()
                    .enumerate()
                    .map(|(i, t)| (i.to_string(), t.clone()))
                    .collect(),
            )),
            Ty::Record(fields) => Ok(Some(fields.clone())),
            Ty::Named { name, args } => {
                let Some(members) = self.members_of(name) else {
                    return Ok(None);
                };
                if members.fields.is_empty() {
                    return Err(());
                }
                let bindings = members.bindings(args);
                let fields: Vec<_> = members
                    .fields
                    .iter()
                    .map(|f| (f.name.clone(), f.ty.subst(&bindings)))
                    .collect();
                Ok(Some(
                    fields
                        .into_iter()
                        .map(|(n, t)| (n, self.settle(&t)))
                        .collect(),
                ))
            }
            Ty::Param(_) | Ty::Unknown | Ty::Error => Ok(None),
            ty if ty.is_vague() => Ok(None),
            _ => Err(()),
        }
    }

    /// `for f in fields(x):` is checked once per field, with `f` of that
    /// field's type, as the back ends unroll it; inside a generic body the
    /// field types are not known yet and `f` is checked once, untyped.
    pub(super) fn fields_loop(
        &mut self,
        pat: &Pattern,
        iter: &Expr,
        body: &cpl_syntax::ast::Block,
    ) {
        let values = self.reflected_values(iter).unwrap_or_default();
        let mut variant_shapes: Option<Vec<Vec<Ty>>> = None;
        let mut shapes = Vec::new();
        let mut first: Option<Ty> = None;
        for v in values {
            let ty = self.infer(&v.value);
            let mismatched = first.as_ref().filter(|first| {
                !first.is_vague()
                    && !ty.is_vague()
                    && **first != ty
                    && (variant_shapes.is_some() || self.variant_payloads(&ty).is_some())
            });
            if let Some(first) = mismatched {
                self.report(
                    v.value.span,
                    msgs::check::TYPE_MISMATCH_FIELDS_WALKS_VALUES_OF,
                    &[&ty, &first],
                );
            }
            if first.is_none() {
                first = Some(ty.clone());
            }
            if let Some(variants) = self.variant_payloads(&ty) {
                variant_shapes = Some(variants);
                shapes.push(None);
                continue;
            }
            match self.reflect(&ty) {
                // Values of other types walk along when they have as many fields.
                Ok(Some(fields))
                    if shapes
                        .iter()
                        .flatten()
                        .next()
                        .is_some_and(|f: &Vec<(String, Ty)>| f.len() != fields.len()) =>
                {
                    self.report(
                        v.value.span,
                        msgs::check::TYPE_MISMATCH_FIELDS_WALKS_VALUES_WITH,
                        &[&ty, &fields.len()],
                    );
                    shapes.push(None);
                }
                Ok(fields) => shapes.push(fields),
                Err(()) => {
                    if let Some(d) = self.report(
                        v.value.span,
                        msgs::check::TYPE_MISMATCH_FIELDS_WALKS_THE_FIELDS,
                        &[&ty],
                    ) {
                        d.help =
                            Some("pass a struct or tuple value: `for f in fields(p):`".to_string());
                    }
                    shapes.push(None);
                }
            }
        }
        if let Some(stmt) = loop_exit(&body.stmts) {
            self.report(stmt, msgs::check::UNSUPPORTED_A_FIELDS_LOOP_RUNS, &[]);
        }
        let elem = |i: Option<usize>, shapes: &[Option<Vec<(String, Ty)>>]| -> Ty {
            let one = |s: &Option<Vec<(String, Ty)>>| match (i, s) {
                (Some(i), Some(f)) => f.get(i).map(|(_, t)| t.clone()).unwrap_or(Ty::Unknown),
                _ => Ty::Unknown,
            };
            if shapes.len() == 1 {
                one(&shapes[0])
            } else {
                Ty::Tuple(shapes.iter().map(one).collect())
            }
        };
        // An enum value walks the payload of the variant it is: the body is
        // checked for each variant's fields.
        if let Some(variants) = variant_shapes {
            let n = shapes.len();
            for fields in variants {
                for ty in fields {
                    let ty = if n == 1 { ty } else { Ty::Tuple(vec![ty; n]) };
                    self.push_scope();
                    self.bind(pat, &ty, false, "for");
                    self.block(body);
                    self.pop_scope();
                }
            }
            return;
        }
        let count = shapes.first().and_then(|s| s.as_ref().map(Vec::len));
        let passes: Vec<Option<usize>> = match count {
            Some(n) => (0..n).map(Some).collect(),
            None => vec![None],
        };
        for i in passes {
            let ty = elem(i, &shapes);
            self.push_scope();
            self.bind(pat, &ty, false, "for");
            self.block(body);
            self.pop_scope();
        }
    }

    pub(super) fn context_record_fields(&mut self, name: &str) -> Option<Vec<(String, Ty)>> {
        if let Some(local) = self.local(name) {
            if local.keyword == "context" {
                if let Ty::Record(fields) = &local.ty {
                    return Some(fields.clone());
                }
            }
        }
        if let Some(Def::Value {
            keyword: "context",
            ty: Ty::Record(fields),
            ..
        }) = self.global(name).as_deref()
        {
            return Some(fields.clone());
        }
        if name == "io" {
            return Some(vec![
                (
                    "base".into(),
                    Ty::Range {
                        lo: Some(0),
                        hi: Some(1),
                        text: "0..=1".into(),
                    },
                ),
                ("sep".into(), Ty::Str),
                ("answers".into(), Ty::Tuple(vec![Ty::Str, Ty::Str])),
                ("precision".into(), Ty::Optional(Box::new(Ty::Int))),
            ]);
        }
        None
    }

    pub(super) fn check_with(&mut self, ctx: &ast::Name, args: &[ast::Arg]) {
        if ctx.text == "checks" {
            const CHECK_FIELDS: &[&str] = &[
                "ranges", "overflow", "bounds", "unwrap", "div", "asserts", "reads",
            ];
            for arg in args {
                let Some(name) = &arg.name else {
                    self.report(
                        arg.value.span,
                        msgs::check::TYPE_MISMATCH_ARGUMENTS_TO_WITH_CHECKS,
                        &[],
                    );
                    continue;
                };
                if !CHECK_FIELDS.contains(&name.text.as_str()) {
                    self.report(
                        name.span,
                        msgs::check::UNKNOWN_FIELD_CONTEXT_CHECKS_HAS_NO,
                        &[&name.text],
                    );
                }
                if !matches!(&arg.value.kind, ExprKind::Name(v) if matches!(v.as_str(), "on" | "off"))
                {
                    self.report(
                        arg.value.span,
                        msgs::check::TYPE_MISMATCH_EXPECTED_ON_OR_OFF,
                        &[],
                    );
                }
            }
            return;
        }
        if ctx.text == "lint" {
            for arg in args {
                if arg.name.is_none() {
                    self.report(
                        arg.value.span,
                        msgs::check::TYPE_MISMATCH_ARGUMENTS_TO_WITH_LINT,
                        &[],
                    );
                    continue;
                }
                if !matches!(
                    &arg.value.kind,
                    ExprKind::Name(v) if matches!(v.as_str(), "allow" | "warn" | "deny")
                ) {
                    self.report(
                        arg.value.span,
                        msgs::check::TYPE_MISMATCH_EXPECTED_ALLOW_WARN_OR,
                        &[],
                    );
                }
            }
            return;
        }
        let Some(fields) = self.context_record_fields(&ctx.text) else {
            self.report(
                ctx.span,
                msgs::check::UNRESOLVED_NAME_UNKNOWN_CONTEXT,
                &[&ctx.text],
            );
            for arg in args {
                self.infer(&arg.value);
            }
            return;
        };
        for arg in args {
            if !context_constant(&arg.value) {
                if let Some(diag) = self.report(
                    arg.value.span,
                    msgs::check::CONTEXT_NOT_CONSTANT_A_CONTEXT_VALUE_IS,
                    &[],
                ) {
                    diag.help = Some(
                        "write the value itself here: a number, character, string or `None`".into(),
                    );
                }
            }
            let Some(name) = &arg.name else {
                self.report(
                    arg.value.span,
                    msgs::check::TYPE_MISMATCH_ARGUMENTS_TO_WITH_MUST,
                    &[&ctx.text],
                );
                self.infer(&arg.value);
                continue;
            };
            let Some((_, expected)) = fields.iter().find(|(n, _)| n == &name.text) else {
                self.report(
                    name.span,
                    msgs::check::UNKNOWN_FIELD_CONTEXT_HAS_NO_FIELD,
                    &[&ctx.text, &name.text],
                );
                self.infer(&arg.value);
                continue;
            };
            let got = self.infer(&arg.value);
            if !matches!(expected, Ty::Error | Ty::Unknown)
                && !matches!(got, Ty::Error | Ty::Unknown)
            {
                let target = match expected {
                    Ty::Optional(inner)
                        if !matches!(got, Ty::Optional(_))
                            && !matches!(arg.value.kind, ExprKind::None) =>
                    {
                        inner.as_ref()
                    }
                    other => other,
                };
                if let Ty::Range { lo, hi, .. } = target {
                    if let ExprKind::Int(n) = arg.value.kind {
                        let in_lo = lo.is_none_or(|l| n >= l);
                        let in_hi = hi.is_none_or(|h| n <= h);
                        if !in_lo || !in_hi {
                            self.report(
                                arg.value.span,
                                msgs::check::TYPE_MISMATCH_EXPECTED_GOT,
                                &[&expected, &n],
                            );
                        }
                        continue;
                    }
                }
                self.expect_fits(&got, target, &arg.value, &|_, got, want| {
                    cpl_diag::msgs::check::TYPE_MISMATCH_EXPECTED_GOT.at(&[&want, &got])
                });
            }
        }
    }
}

/// A literal, a level such as `off`, or a tuple of them. (R09.1.2)
pub(super) fn context_constant(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Int(_)
        | ExprKind::Float(_)
        | ExprKind::Char(_)
        | ExprKind::Bool(_)
        | ExprKind::None => true,
        ExprKind::Str(pieces) => pieces
            .iter()
            .all(|p| matches!(p, cpl_syntax::ast::StrPiece::Text(_))),
        ExprKind::Name(name) => matches!(name.as_str(), "on" | "off" | "allow" | "warn" | "deny"),
        ExprKind::Unary { expr, .. } => context_constant(expr),
        ExprKind::Tuple(items) => items.iter().all(context_constant),
        _ => false,
    }
}
