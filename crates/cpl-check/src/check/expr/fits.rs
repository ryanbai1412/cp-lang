//! Whether a type fits where another is wanted, and the diagnostics when it does not.

use super::*;
use cpl_diag::msgs;

impl<'a> Checker<'a> {
    /// A struct value seen as a struct with fewer fields, when one is
    /// wanted: a `Group<Int>` has the `id` and `op` of a `Monoid<Int>`, so
    /// it is one (decision 14.1). Anything else is returned as it is.
    pub(crate) fn narrowed(&mut self, got: &Ty, want: &Ty) -> Ty {
        let (
            Ty::Named {
                name: got_name,
                args: got_args,
            },
            Ty::Named {
                name: want_name, ..
            },
        ) = (got, want)
        else {
            return got.clone();
        };
        if got_name == want_name {
            return got.clone();
        }
        let (Some(got_members), Some(want_members)) =
            (self.members_of(got_name), self.members_of(want_name))
        else {
            return got.clone();
        };
        if want_members.fields.is_empty() {
            return got.clone();
        }
        let got_bindings = got_members.bindings(got_args);
        let mut bindings = crate::ty::Bindings::default();
        for want_field in &want_members.fields {
            let Some(got_field) = got_members
                .fields
                .iter()
                .find(|f| f.name == want_field.name)
            else {
                return got.clone();
            };
            let got_ty = got_field.ty.subst(&got_bindings);
            if !self.fits(&got_ty, &want_field.ty) {
                return got.clone();
            }
            want_field.ty.unify(&got_ty, &mut bindings);
        }
        Ty::Named {
            name: want_name.clone(),
            args: want_members
                .generics
                .iter()
                .map(|g| bindings.get(g).cloned().unwrap_or(Ty::Unknown))
                .collect(),
        }
    }

    /// `got.fits(want)`, with an `Int` fitting a number struct, cut-domain
    /// values/ranges (§124), and product tuples/records fitting their product domain.
    pub(crate) fn fits(&mut self, got: &Ty, want: &Ty) -> bool {
        if got.fits_in(want, &mut |name| self.is_number(name)) {
            return true;
        }
        match (got, want) {
            (Ty::Named { name, args }, Ty::Pos(dom))
                if name == "Subset"
                    && args.len() == 1
                    && matches!(&args[0], Ty::Pos(d) | Ty::Domain(d) if dom.strip_suffix(".subsets") == Some(d.as_str())) =>
            {
                return true;
            }
            (Ty::Pos(dom), Ty::Named { name, args })
                if name == "Subset"
                    && args.len() == 1
                    && matches!(&args[0], Ty::Pos(d) | Ty::Domain(d) if dom.strip_suffix(".subsets") == Some(d.as_str())) =>
            {
                return true;
            }
            // §124: integer literals and checked arithmetic fit `D.bounds`.
            (a, Ty::Pos(dom)) if a.is_int() && self.bounds_base_domain(dom).is_some() => {
                return true;
            }
            // §124: half-open cut range `Range<D.bounds>` fits `Range<D>`.
            (Ty::Named { name: an, args: aa }, Ty::Named { name: bn, args: ba })
                if an == "Range" && bn == "Range" && aa.len() == 1 && ba.len() == 1 =>
            {
                if let (Ty::Pos(cut), Ty::Pos(dom)) = (&aa[0], &ba[0]) {
                    if self.bounds_base_domain(cut).as_deref() == Some(dom.as_str()) {
                        return true;
                    }
                }
                if matches!(&aa[0], Ty::Int | Ty::Range { .. }) && matches!(&ba[0], Ty::Pos(_)) {
                    return true;
                }
            }
            (Ty::Array { index: a, elem: ae }, Ty::Array { index: b, elem: be })
                if a.len() == 1 && b.len() > 1 =>
            {
                if let Ty::Pos(dom) = &a[0] {
                    if let Some(factors) = self.product_factors(dom) {
                        return factors.len() == b.len()
                            && factors.iter().zip(b).all(|((_, f), w)| self.fits(f, w))
                            && self.fits(ae, be);
                    }
                }
            }
            (Ty::Array { index: a, elem: ae }, Ty::Array { index: b, elem: be })
                if b.len() == 1 && a.len() > 1 =>
            {
                if let Ty::Pos(dom) = &b[0] {
                    if let Some(factors) = self.product_factors(dom) {
                        return factors.len() == a.len()
                            && a.iter().zip(&factors).all(|(g, (_, w))| self.fits(g, w))
                            && self.fits(ae, be);
                    }
                }
            }
            (Ty::Named { name: an, args: aa }, Ty::Named { name: bn, args: ba })
                if an == "List" && bn == "List" && aa.len() == 1 && ba.len() == 1 =>
            {
                return self.fits(&aa[0], &ba[0]);
            }
            (Ty::Tuple(items), Ty::Pos(dom)) => {
                if let Some(factors) = self.product_factors(dom) {
                    return items.len() == factors.len()
                        && items
                            .iter()
                            .zip(&factors)
                            .all(|(g, (_, w))| self.fits(g, w));
                }
            }
            (Ty::Record(fields), Ty::Pos(dom)) => {
                if let Some(factors) = self.product_factors(dom) {
                    return fields.len() == factors.len()
                        && fields.iter().zip(&factors).all(|((gn, g), (wn, w))| {
                            wn.as_ref().is_none_or(|wn| wn == gn) && self.fits(g, w)
                        });
                }
            }
            (Ty::Tuple(a), Ty::Tuple(b)) if a.len() == b.len() => {
                return a.iter().zip(b).all(|(g, w)| self.fits(g, w));
            }
            (_, Ty::Optional(inner)) => {
                return self.fits(got, inner);
            }
            _ => {}
        }
        false
    }

    /// A list literal written where a 1D array is wanted is that array's
    /// cells, also inside a tuple literal or as the value of a `match` or
    /// `if` arm: `([p[1], p[0]], [q, r, s])` is a `([2]Site, [3]Site)`.
    pub(super) fn literal_shape_fits(&mut self, at: &Expr, got: &Ty, want: &Ty) -> bool {
        let tail = |block: &ast::Block| match block.stmts.last().map(|s| &s.kind) {
            Some(ast::StmtKind::Expr(e)) => Some(e.clone()),
            _ => None,
        };
        let part = |c: &mut Self, item: &Expr, g: &Ty, w: &Ty| {
            c.fits(g, w) || c.literal_position(item, w) || c.literal_shape_fits(item, g, w)
        };
        let list_elem = |ty: &Ty| match ty {
            Ty::Named { name, args } if name == "List" && args.len() == 1 => Some(args[0].clone()),
            Ty::Array { index, elem } if index.len() == 1 => Some((**elem).clone()),
            _ => None,
        };
        match (&at.kind, got, want) {
            (ExprKind::List(items), Ty::Named { name, args }, _)
                if name == "List" && list_elem(want).is_some() =>
            {
                let w = list_elem(want).unwrap_or(Ty::Error);
                match args.first() {
                    Some(g) => items.iter().all(|item| part(self, item, g, &w)),
                    None => true,
                }
            }
            (ExprKind::Tuple(items), Ty::Tuple(gots), Ty::Tuple(wants))
                if items.len() == gots.len() && gots.len() == wants.len() =>
            {
                items
                    .iter()
                    .zip(gots.iter().zip(wants))
                    .all(|(item, (g, w))| part(self, item, g, w))
            }
            (ExprKind::Record(items), Ty::Record(gots), Ty::Record(wants))
                if items.len() == gots.len() && gots.len() == wants.len() =>
            {
                items.iter().all(|(name, item)| {
                    let g = gots.iter().find(|(n, _)| *n == name.text).map(|(_, t)| t);
                    let w = wants.iter().find(|(n, _)| *n == name.text).map(|(_, t)| t);
                    matches!((g, w), (Some(g), Some(w)) if part(self, item, g, w))
                })
            }
            (ExprKind::Match { arms, .. }, _, _) => {
                let tails: Vec<_> = arms.iter().map(|arm| tail(&arm.body)).collect();
                tails.iter().all(|t| {
                    t.as_ref()
                        .is_some_and(|e| self.literal_shape_fits(e, got, want))
                })
            }
            (
                ExprKind::If {
                    branches,
                    otherwise,
                },
                _,
                _,
            ) => {
                let mut tails: Vec<_> = branches.iter().map(|(_, b)| tail(b)).collect();
                tails.push(otherwise.as_ref().and_then(tail));
                tails.iter().all(|t| {
                    t.as_ref()
                        .is_some_and(|e| self.literal_shape_fits(e, got, want))
                })
            }
            (_, _, Ty::Optional(inner)) => self.literal_shape_fits(at, got, inner),
            _ => false,
        }
    }

    pub(crate) fn expect_fits_by(
        &mut self,
        got: &Ty,
        want: &Ty,
        at: &Expr,
        message: &mut dyn FnMut(&Self, &Ty, &Ty) -> String,
    ) -> bool {
        // `none` fills whatever is declared, and code that already has an
        // error is not reported again.
        let got = &self.read_narrowed(got, want, at);
        // §124: `j..=i` over cuts never converts to `Range<D>`.
        if let ExprKind::Range {
            hi: Some(hi),
            inclusive: true,
            ..
        } = &at.kind
        {
            let inner_want = match want {
                Ty::Optional(inner) => inner.as_ref(),
                other => other,
            };
            if let (Ty::Named { name: an, args: aa }, Ty::Named { name: bn, args: ba }) =
                (got, inner_want)
            {
                if an == "Range" && bn == "Range" && aa.len() == 1 && ba.len() == 1 {
                    if let (Ty::Pos(cut), Ty::Pos(dom)) = (&aa[0], &ba[0]) {
                        if self.bounds_base_domain(cut).as_deref() == Some(dom.as_str()) {
                            if let Some(diag) = self.report(
                                at.span,
                                msgs::check::WRONG_DOMAIN_ENDS_AT_A_POSITION
                                    .at(&[&self.quote(hi.span)]),
                            ) {
                                diag.help = Some(
                                    "use half-open `..` for the positions between two cuts"
                                        .to_string(),
                                );
                            }
                            return false;
                        }
                    }
                }
            }
        }
        if let Ty::Range {
            lo: want_lo,
            hi: want_hi,
            ..
        } = want
        {
            let (vlo, vhi) = self.bounds(at);
            let out_low = matches!((vhi, want_lo), (Some(vh), Some(wl)) if vh < *wl);
            let out_high = matches!((vlo, want_hi), (Some(vl), Some(wh)) if vl > *wh);
            if out_low || out_high {
                self.report(
                    at.span,
                    msgs::check::RANGE_STORE_CONST_IS_OUT_OF_RANGE
                        .at(&[&self.quote(at.span), &want]),
                );
                return false;
            }
        }

        if self.fits(got, want)
            || matches!(at.kind, ExprKind::None)
            || got.has_error()
            || want.has_error()
            || self.expected_variants(at, want)
        {
            return true;
        }
        // A list literal `[e0, e1, ...]` fits a 1D `[D]T` array when element
        // types fit (and for a plain enum domain `D`, when the length equals |D|).
        if let ExprKind::List(items) = &at.kind {
            let inner_want = match want {
                Ty::Optional(inner) => inner.as_ref(),
                other => other,
            };
            if let (Ty::Named { name, args }, Ty::Array { index, elem }) = (got, inner_want) {
                if name == "List" && index.len() == 1 {
                    let elem_ok = args.first().is_none_or(|g| self.fits(g, elem));
                    match &index[0] {
                        Ty::Named { name: ename, .. } => {
                            if let Some(Def::Enum { variants, .. }) =
                                self.type_def(ename).as_deref().cloned()
                            {
                                if variants.iter().all(|(_, fs)| fs.is_empty()) {
                                    if items.len() != variants.len() {
                                        self.report(
                                            at.span,
                                            msgs::check::TYPE_MISMATCH_HOLDS_ELEMENTS_ONE_PER.at(
                                                &[
                                                    &inner_want,
                                                    &variants.len(),
                                                    &ename,
                                                    &items.len(),
                                                ],
                                            ),
                                        );
                                        return false;
                                    }
                                    if elem_ok {
                                        return true;
                                    }
                                }
                            }
                        }
                        Ty::Pos(_) if elem_ok => return true,
                        _ => {}
                    }
                }
            }
        }
        if self.literal_position(at, want) || self.literal_shape_fits(at, got, want) {
            return true;
        }
        if let Some(may) = self.unhandled_step(got, want, at) {
            let text = format!(
                "expected {want}, got {got}: `{}` {may}",
                self.quote(at.span)
            );
            let help = self.step_help(at);
            let origin = self.origin(at, &format!("declared as {}", got.with_article()));
            // R04.8.4: a `?V` where a `V` is wanted is the wrong type, not the wrong domain.
            let code = { codes::TYPE_MISMATCH };
            if let Some(diag) = self.error(code, at.span, text) {
                diag.help = Some(help);
                diag.labels.extend(origin);
            }
            return false;
        }
        if matches!(got, Ty::Domain(_)) && !matches!(want, Ty::Domain(_)) {
            self.planned(at.span, &cpl_syntax::planned::DOMAIN_VALUE);
            return false;
        }
        let text = message(self, got, want);
        let is_empty_fold = matches!(got, Ty::Optional(_))
            && !matches!(want, Ty::Optional(_))
            && matches!(
                &at.kind,
                ExprKind::Call { callee, args }
                    if args.len() == 1
                        && matches!(&callee.kind, ExprKind::Field { name, .. } if name.text == "fold")
            );
        let code = if is_empty_fold {
            codes::EMPTY_FOLD_NO_IDENTITY
        } else if matches!((got, want), (Ty::Pos(_), Ty::Pos(_))) {
            codes::WRONG_DOMAIN
        } else {
            codes::TYPE_MISMATCH
        };
        let quoted = self.quote(at.span);
        let origin = self.origin(at, &format!("declared as {}", got.with_article()));
        let struct_of_tuple = self.struct_of_tuple(got, want, at);
        if let Some(diag) = self.error(code, at.span, text) {
            diag.labels.extend(origin);
            if let (true, Ty::Pos(domain)) = (got.is_int(), want) {
                diag.help = Some(format!("convert it with `{domain}({quoted})`"));
                diag.fixes.push(convert_to(domain, at.span));
            } else if let Some(help) = rename_fields(got, want) {
                diag.help = Some(help);
            } else if let Some((help, fix)) = struct_of_tuple {
                diag.help = Some(help);
                diag.fixes.extend(fix);
            }
        }
        self.char_literal_fix(got, want, at)
    }

    /// R08.1.6: a tuple is not a struct, even one with its fields' types.
    /// The help builds the struct, and the fix does when that reads the
    /// tuple once: a literal or a name.
    fn struct_of_tuple(
        &mut self,
        got: &Ty,
        want: &Ty,
        at: &Expr,
    ) -> Option<(String, Option<cpl_diag::Fix>)> {
        let (Ty::Tuple(items), Ty::Named { name, .. }) = (got, want) else {
            return None;
        };
        let def = self.type_def(name);
        let Some(Def::Struct { fields, .. }) = def.as_deref() else {
            return None;
        };
        if fields.len() != items.len() {
            return None;
        }
        let quoted = self.quote(at.span);
        let built = match &at.kind {
            ExprKind::Tuple(_) => format!("{name}{quoted}"),
            ExprKind::Name(_) => {
                let parts: Vec<String> =
                    (0..items.len()).map(|i| format!("{quoted}.{i}")).collect();
                format!("{name}({})", parts.join(", "))
            }
            _ => {
                return Some((
                    format!(
                        "a tuple is not a `{name}`: build a `{name}(..)` where `{quoted}` is made"
                    ),
                    None,
                ));
            }
        };
        let fix = cpl_diag::Fix {
            title: format!("build a `{name}`"),
            edits: vec![(at.span, built.clone())],
        };
        Some((
            format!("a tuple is not a `{name}`: build one, `{built}`"),
            Some(fix),
        ))
    }
}

/// `(a: V, b: V)` given where `(u: V, v: V)` is wanted, inside the same
/// arrays, lists or optionals: the help names the fields to rename.
pub(crate) fn rename_fields(got: &Ty, want: &Ty) -> Option<String> {
    match (got, want) {
        (Ty::Array { elem: got, .. }, Ty::Array { elem: want, .. })
        | (Ty::Optional(got), Ty::Optional(want)) => rename_fields(got, want),
        (
            Ty::Named { name, args: got },
            Ty::Named {
                name: wanted,
                args: want,
            },
        ) if name == wanted && got.len() == want.len() => got
            .iter()
            .zip(want)
            .find_map(|(got, want)| rename_fields(got, want)),
        (Ty::Record(got), Ty::Record(want)) if got.len() >= want.len() => {
            let has = |fields: &[(String, Ty)], name: &str| fields.iter().any(|(n, _)| n == name);
            let missing: Vec<&str> = want
                .iter()
                .map(|(n, _)| n.as_str())
                .filter(|n| !has(got, n))
                .collect();
            let spare: Vec<&str> = got
                .iter()
                .map(|(n, _)| n.as_str())
                .filter(|n| !has(want, n))
                .collect();
            if missing.is_empty() || spare.len() < missing.len() {
                return None;
            }
            let pairs: Vec<String> = spare
                .iter()
                .zip(&missing)
                .map(|(from, to)| format!("`{from}` to `{to}`"))
                .collect();
            Some(format!(
                "the fields are read by name: rename {}",
                pairs.join(", ")
            ))
        }
        _ => None,
    }
}

impl Checker<'_> {
    /// `domain Cut = Pos.cuts` names the cuts of `Pos`: as a range end it
    /// is `Pos.cuts`.
    pub(super) fn unalias_bounds(&mut self, ty: Ty) -> Ty {
        if let Ty::Pos(d) = &ty {
            if !d.ends_with(".cuts") {
                if let Some(Def::Domain(info)) = self.type_def(d).as_deref() {
                    let text = info.text.trim();
                    if let Some(base) = text.strip_suffix(".cuts") {
                        if !base.is_empty() && base.chars().all(|c| c.is_alphanumeric() || c == '_')
                        {
                            return Ty::Pos(text.to_string());
                        }
                    }
                }
            }
        }
        ty
    }

    /// An integer literal written where a position or cut of an integer
    /// domain is expected (or fills an array of them) is that position.
    pub(super) fn literal_position(&mut self, at: &Expr, want: &Ty) -> bool {
        // R04.4.26: a literal `j..i` where a `Range<V>` is wanted is the piece between them.
        if let (
            ExprKind::Range {
                lo, hi, step: None, ..
            },
            Ty::Named { name, args },
        ) = (&at.kind, want)
        {
            let [elem @ Ty::Pos(_)] = args.as_slice() else {
                return false;
            };
            if name != "Range" || (lo.is_none() && hi.is_none()) {
                return false;
            }
            let elem = elem.clone();
            return [lo, hi]
                .into_iter()
                .flatten()
                .all(|end| self.literal_position(end, &elem));
        }
        if let Some(fits) = self.literal_float(at, want) {
            return fits;
        }
        let Some(domain) = want.literal_domain() else {
            return false;
        };
        let base = domain.strip_suffix(".cuts").unwrap_or(&domain).to_string();
        if self
            .local(&base)
            .is_some_and(|l| matches!(l.ty, Ty::Domain(_)))
        {
            return !self.local_product_domains.contains_key(&base);
        }
        let text = match self.type_def(&base).as_deref() {
            Some(Def::Domain(info)) if info.factors.is_empty() => info.text.clone(),
            _ => return false,
        };
        self.literal_in_domain(at, &base, &text, domain.ends_with(".cuts"));
        true
    }

    /// R04.2.18: a literal outside a static domain bound is an error.
    pub(super) fn literal_in_domain(&mut self, at: &Expr, domain: &str, text: &str, cut: bool) {
        let ExprKind::Int(v) = at.kind else {
            return;
        };
        let bounds = text.split_once("..").and_then(|(lo, hi)| {
            let (hi, inclusive) = match hi.strip_prefix('=') {
                Some(hi) => (hi, true),
                None => (hi, false),
            };
            let lo: i128 = lo.trim().parse().ok()?;
            let hi: i128 = hi.trim().parse().ok()?;
            Some((lo, if inclusive { hi } else { hi - 1 }))
        });
        let Some((lo, hi)) = bounds else {
            return;
        };
        let (lo, hi) = if cut { (0, hi - lo + 1) } else { (lo, hi) };
        if (lo..=hi).contains(&v) {
            return;
        }
        let what = if cut {
            format!("{domain}.cuts")
        } else {
            domain.to_string()
        };
        if let Some(diag) = self.report(
            at.span,
            msgs::check::RANGE_STORE_CONST_IS_OUTSIDE_WHICH_IS.at(&[&v, &what, &lo, &hi]),
        ) {
            diag.help = Some(format!("use a literal in {lo}..={hi}"));
        }
    }
}

/// Wraps the number at `span` in `domain(...)`, the checked conversion.
pub(super) fn convert_to(domain: &str, span: Span) -> cpl_diag::Fix {
    cpl_diag::Fix {
        title: format!("convert with `{domain}(...)`"),
        edits: vec![
            (Span::point(span.start), format!("{domain}(")),
            (Span::point(span.end), ")".into()),
        ],
    }
}
