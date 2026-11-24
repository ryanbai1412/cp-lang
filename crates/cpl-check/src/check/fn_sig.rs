//! A fn's signature: its written types, and for a fn in a trait impl the
//! unwritten ones read off the trait's declaration with `Self` and the trait's
//! arguments put in, for a user trait and a core one alike.

use super::*;
use crate::ty::Param;
use cpl_diag::msgs;
use cpl_syntax::ast;

impl Checker<'_> {
    /// The signature of a fn. `self_ty` is the receiver's type in a method;
    /// the `self` parameter is not part of the signature callers see.
    pub(crate) fn fn_sig(&mut self, f: &ast::FnDecl, self_ty: Option<&Ty>) -> Ty {
        let outer = self.generics.len();
        self.generics
            .extend(f.generics.iter().map(|g| g.text.clone()));
        self.in_fn += 1;
        // A parameter's type can use the ones before it:
        // `fn valid(n: Int, seq: [0..n]Int)`. So can the result's.
        self.push_scope();
        let declared = self.impl_trait_sig(&f.name.text);
        let mut params = Vec::new();
        for (i, p) in f
            .params
            .iter()
            .filter(|p| !(self_ty.is_some() && p.name.text == "self"))
            .enumerate()
        {
            let ty = match &p.ty {
                Some(ty) => self.resolve_type(ty),
                None => (declared.as_ref())
                    .and_then(|(params, _)| params.get(i).cloned())
                    .unwrap_or(Ty::Unknown),
            };
            if p.variadic && !matches!(ty, Ty::Param(_) | Ty::Error) {
                if let Some(diag) = self.report(
                    p.span,
                    msgs::check::VARIADIC_NOT_PARAM,
                    &[&p.name.text, &ty],
                ) {
                    diag.help = Some(format!(
                        "write `fn {}<Ts>(..{}: Ts)`, or take a `List<{ty}>` and pass `[a, b, c]`",
                        f.name.text, p.name.text
                    ));
                }
            }
            if p.lazy
                && !matches!(&ty, Ty::Fn { params, .. } if params.is_empty())
                && ty != Ty::Error
            {
                if let Some(diag) = self.report(
                    p.span,
                    msgs::check::LAZY_NOT_FN,
                    &[&p.name.text, &p.name.text, &ty],
                ) {
                    diag.help = Some(format!("write `lazy {}: () -> {ty}`", p.name.text));
                }
            }
            let seen = Local {
                ty: ty.clone(),
                mutable: p.mutable,
                span: p.name.span,
                keyword: "param",
            };
            self.scopes.innermost().insert(p.name.text.clone(), seen);
            if p.ty.as_ref().is_some_and(Self::names_range_keys) {
                self.lends.range_keys.insert(p.name.span);
            }
            params.push(Param {
                name: p.label_text().to_string(),
                ty,
                mutable: p.mutable || self.note_opt_mut(p),
                has_default: p.default.is_some(),
                named_only: p.named_only,
                variadic: p.variadic,
                lazy: p.lazy,
            });
        }
        let ret = match (&f.ret, &f.lends, &f.body) {
            (Some(ty), _, _) => self.resolve_type(ty),
            // A projection's caller works with the value it lends.
            (None, Some(lends), _) => self.resolve_type(&lends.ty),
            (None, None, ast::FnBody::Expr(_)) => declared.map_or(Ty::Unknown, |(_, ret)| ret),
            (None, None, _) => Ty::Unit,
        };
        self.scopes.pop();
        self.in_fn -= 1;
        self.generics.truncate(outer);
        Ty::Fn {
            params,
            ret: Box::new(ret),
        }
    }

    /// Reads an impl's trait and its arguments for the fns in its body:
    /// `impl Mul<Int> for Pt` is `Mul` with `[Int]`. Returns the outer one.
    pub(crate) fn enter_impl_trait(
        &mut self,
        trait_ty: Option<&ast::Type>,
    ) -> Option<(String, Vec<Ty>)> {
        let read = trait_ty.and_then(|t| match &t.kind {
            ast::TypeKind::Named { name, args } => Some((
                name.text.clone(),
                args.iter()
                    .map(|a| self.quietly(|c| c.resolve_type(a)))
                    .collect(),
            )),
            _ => None,
        });
        std::mem::replace(&mut self.impl_trait, read)
    }

    /// The signature the trait of the impl being read declares for its fn
    /// `method`, with `Self` and the trait's arguments put in (a missing
    /// argument is `Self`). An impl fn's unwritten types are read from it:
    /// `fn add(a, b)` in `impl Add for Pt` takes two `Pt`, and `fn mul(a, k)`
    /// in `impl Mul<Int> for Pt` a `Pt` and an `Int`.
    fn impl_trait_sig(&mut self, method: &str) -> Option<(Vec<Ty>, Ty)> {
        let (trait_name, args) = self.impl_trait.clone()?;
        let self_ty = self.self_ty.clone()?;
        let (generics, sig) = match self.global(&trait_name).as_deref() {
            Some(Def::Trait {
                generics, methods, ..
            }) => (
                generics.clone(),
                methods.iter().find(|m| m.name == method)?.ty.clone(),
            ),
            _ => {
                let (generics, body) = prelude::core_trait_decl(&trait_name)?;
                let generics: Vec<String> = generics.iter().map(|g| g.text.clone()).collect();
                let outer = self.generics.len();
                self.generics.push("Self".into());
                self.generics.extend(generics.iter().cloned());
                let outer_trait = self.impl_trait.take();
                let (sigs, ..) = self.quietly(|c| c.method_sigs(body, &Ty::Param("Self".into())));
                self.impl_trait = outer_trait;
                self.generics.truncate(outer);
                let sig = sigs.into_iter().find(|(n, _)| n == method)?.1;
                (generics, sig)
            }
        };
        let mut on = crate::ty::Bindings::default();
        on.insert("Self".into(), self_ty.clone());
        for (i, g) in generics.into_iter().enumerate() {
            on.insert(g, args.get(i).cloned().unwrap_or_else(|| self_ty.clone()));
        }
        let Ty::Fn { params, ret } = sig else {
            return None;
        };
        let generics = &self.generics;
        let known = |t: Ty| {
            if Self::names_foreign_param(&t, generics) {
                Ty::Unknown
            } else {
                t
            }
        };
        Some((
            params.iter().map(|p| known(p.ty.subst(&on))).collect(),
            known(ret.subst(&on)),
        ))
    }

    /// Whether `ty` names a type parameter not in scope here, such as a
    /// trait fn's own `<U>`.
    fn names_foreign_param(ty: &Ty, generics: &[String]) -> bool {
        let mut foreign = false;
        ty.map(&mut |t| {
            if let Ty::Param(n) = t {
                foreign |= !generics.contains(n);
            }
            None
        });
        foreign
    }
}
