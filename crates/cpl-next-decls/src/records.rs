//! Records `(lo: Int, hi: V)` (R02.5.2). Each list of field names, in order, is one generic core
//! struct `(lo:, hi:)<A, B>`, so two record types are equal exactly when their field names, order
//! and types are, and fields, constructors, patterns and derived `Eq` and `Ord` are a struct's.
use crate::attrs::Decorator;
use crate::decls::forms::{
    AssocForm, AttrForm, BodyForm, BoundForm, FieldForm, FnForm, ImplForm, ParamForm, StructForm,
    TraitForm, TyForm,
};
use crate::decls::{form_fills, Role};
use crate::sym::{Sym, Syms};
use cpl_diag::Span;
use cpl_next_syntax::ast::{self as next, AstNode as _};
use std::collections::{BTreeSet, HashMap};

/// The struct a record with these fields is: `(lo:, hi:)`.
pub fn name<S: AsRef<str>>(fields: &[S]) -> String {
    let fs: Vec<String> = fields.iter().map(|f| format!("{}:", f.as_ref())).collect();
    format!("({})", fs.join(", "))
}

/// The trait a row bound `R: (u: V, w: W, ..)` is: `(u:, w:, ..)`, met by every record with
/// those fields (R08.4.1).
pub fn row_name<S: AsRef<str>>(fields: &[S]) -> String {
    let fs: Vec<String> = fields
        .iter()
        .map(|f| format!("{}:, ", f.as_ref()))
        .collect();
    format!("({}..)", fs.concat())
}

/// Every row bound any module writes, by its field names.
pub fn rows(mods: &[crate::modules::Module]) -> Vec<Vec<String>> {
    let mut out = BTreeSet::new();
    for t in all_of::<next::RecordType>(mods) {
        let fs: Vec<_> = t.fields().collect();
        if fs.iter().any(|f| f.is_rest() && f.ty().is_none()) {
            out.insert(field_names(&fs));
        }
    }
    out.into_iter().collect()
}

/// Each node of kind `N` anywhere in `mods`' items, bodies too.
fn all_of<N: next::AstNode>(mods: &[crate::modules::Module]) -> Vec<N> {
    (mods.iter().flat_map(|m| m.items.iter()))
        .flat_map(|it| {
            it.item
                .syntax()
                .descendants()
                .filter_map(N::cast)
                .collect::<Vec<_>>()
        })
        .collect()
}

/// A record type's written field names, in order; a rest and a `domain C` binder have none.
fn field_names(fs: &[next::RecordTypeField]) -> Vec<String> {
    (fs.iter().filter(|f| !f.is_rest()))
        .filter_map(|f| f.name())
        .map(|n| n.text().to_string())
        .collect()
}

fn token_texts(ts: &[cpl_next_syntax::tree::SyntaxToken]) -> Vec<String> {
    ts.iter().map(|t| t.text().to_string()).collect()
}

/// `pub trait (u:, w:, ..)<A, B>: @get fn u(self) -> A; @get fn w(self) -> B` for each row: a
/// trait of `@get` members only, so each record shape with those fields meets it by its fields.
pub fn row_traits(rows: &[Vec<String>], syms: &mut Syms, span: Span) -> Vec<TraitForm> {
    (rows.iter())
        .filter(|fs| fs.len() <= 26)
        .map(|fs| {
            let generics = capitals(syms, fs.len());
            let methods = (fs.iter().enumerate())
                .map(|(i, f)| {
                    let f = syms.intern(f);
                    crate::synth::field_requirement(f, TyForm::Param(i), span, syms)
                })
                .collect();
            TraitForm {
                name: syms.intern(&row_name(fs)),
                public: true,
                attrs: Vec::new(),
                record: true,
                alias: false,
                generic_defaults: vec![None; fs.len()],
                generics,
                supers: Vec::new(),
                assoc: Vec::new(),
                methods,
                span,
            }
        })
        .collect()
}

/// `A`, `B`, ..: a made declaration's type parameters, single capitals as the derived impls'
/// bounds expect.
fn capitals(syms: &mut Syms, n: usize) -> Vec<Sym> {
    (b'A'..)
        .take(n)
        .map(|c| syms.intern(&(c as char).to_string()))
        .collect()
}

/// Each fn with a last `..xs` parameter, by name, with the labels of its other parameters: the
/// arguments of a call by any other names are the record `xs` is (R06.1.5).
pub fn variadics(mods: &[crate::modules::Module]) -> HashMap<String, Vec<String>> {
    let mut out: HashMap<String, Vec<String>> = HashMap::new();
    for it in mods.iter().flat_map(|m| m.items.iter()) {
        let Some(next::Decl::Fn(f)) = it.item.decl() else {
            continue;
        };
        let ps: Vec<next::Param> = f
            .params()
            .iter()
            .flat_map(|l| l.params().collect::<Vec<_>>())
            .collect();
        if ps.last().is_some_and(|p| p.is_variadic()) {
            let labels = (ps.iter()).filter_map(|p| p.label().or_else(|| p.name()));
            let name = f.name().map_or(String::new(), |n| n.text().to_string());
            (out.entry(name).or_default()).extend(labels.map(|t| t.text().to_string()));
        }
    }
    out
}

/// Every record shape any module writes, as a type, a literal or the named arguments of a
/// call to a fn in `variadics`.
pub fn shapes(
    mods: &[crate::modules::Module],
    variadics: &HashMap<String, Vec<String>>,
) -> Vec<Vec<String>> {
    let mut out = BTreeSet::new();
    for t in all_of::<next::RecordType>(mods) {
        let fs: Vec<_> = t.fields().collect();
        if !fs.iter().any(|f| f.is_rest()) {
            out.insert(field_names(&fs));
        }
    }
    for e in all_of::<next::RecordExpr>(mods) {
        if e.rest().is_none() {
            out.insert(
                e.fields()
                    .filter_map(|f| f.name())
                    .map(|n| n.text().to_string())
                    .collect(),
            );
        }
    }
    // The named arguments of a call to a variadic fn past its own labels are the record its
    // `..xs` is (R06.1.5).
    for c in all_of::<next::CallExpr>(mods) {
        let Some(next::Expr::Name(n)) = c.callee() else {
            continue;
        };
        let Some(fixed) = n.ident().and_then(|n| variadics.get(n.text())) else {
            continue;
        };
        let named: Vec<String> = (c.args().iter().flat_map(|a| a.args().collect::<Vec<_>>()))
            .filter_map(|a| a.label())
            .map(|l| l.text().to_string())
            .filter(|l| !fixed.contains(l))
            .collect();
        if !named.is_empty() {
            out.insert(named);
        }
    }
    out.into_iter().collect()
}

/// A `without` form, `(to: _, e: _, ..R without (u, v))`: the fields it adds, then those it drops.
pub type Form = (Vec<String>, Vec<String>);

/// Every `without` form any module writes, in a type or a value.
pub fn forms(mods: &[crate::modules::Module]) -> Vec<Form> {
    let mut out = BTreeSet::new();
    for t in all_of::<next::RecordType>(mods) {
        let fs: Vec<_> = t.fields().collect();
        if let Some(rest) = fs.iter().find(|f| f.is_rest() && f.ty().is_some()) {
            out.insert((field_names(&fs), token_texts(&rest.removed())));
        }
    }
    for e in all_of::<next::RecordExpr>(mods)
        .into_iter()
        .filter(|e| !e.is_update())
    {
        if let Some(rest) = e.rest() {
            let added = e
                .fields()
                .filter_map(|f| f.name())
                .map(|n| n.text().to_string())
                .collect();
            out.insert((added, token_texts(&rest.removed())));
        }
    }
    out.into_iter().collect()
}

/// The trait a form is, `(to:, e:, ..without u, v)`: met by each record with the fields it drops
/// and none it adds (R08.4.1).
pub fn without_name((prefix, removed): &Form) -> String {
    let fs: Vec<String> = prefix.iter().map(|f| format!("{f}:, ")).collect();
    format!("({}..without {})", fs.concat(), removed.join(", "))
}

/// The form's trait's associated type, the record it makes, and its member that makes it.
pub fn without_member((prefix, removed): &Form) -> String {
    format!("without({}; {})", prefix.join(", "), removed.join(", "))
}

/// The fields a form makes of a record with fields `fs`, its added ones first; none when the
/// record lacks a dropped field or has an added one, or nothing is left.
pub fn without_shape((prefix, removed): &Form, fs: &[String]) -> Option<Vec<String>> {
    let rest: Vec<&String> = fs.iter().filter(|f| !removed.contains(f)).collect();
    let fits = removed.iter().all(|r| fs.contains(r)) && !rest.iter().any(|f| prefix.contains(f));
    let out: Vec<String> = prefix.iter().chain(rest).cloned().collect();
    (fits && !out.is_empty()).then_some(out)
}

/// The shapes the forms make of `shapes` and of what they make, past `shapes`, sorted.
pub fn derived(forms: &[Form], shapes: &[Vec<String>]) -> Vec<Vec<String>> {
    let mut all: BTreeSet<Vec<String>> = shapes.iter().cloned().collect();
    let mut made = BTreeSet::new();
    let mut fresh: Vec<Vec<String>> = shapes.to_vec();
    // A form drops fields it does not add back, so each round's shapes are smaller.
    while !fresh.is_empty() {
        let next: Vec<Vec<String>> = (fresh.iter())
            .flat_map(|fs| forms.iter().filter_map(move |f| without_shape(f, fs)))
            .filter(|s| all.insert(s.clone()))
            .collect();
        made.extend(next.iter().cloned());
        fresh = next;
    }
    made.into_iter().collect()
}

/// `pub trait (to:, e:, ..without u, v)<P0, P1>: type R; fn R(self, to: P0, e: P1) -> Self.R`
/// for each form, `R` its `without_member`.
pub fn without_traits(forms: &[Form], syms: &mut Syms, span: Span) -> Vec<TraitForm> {
    (forms.iter())
        .map(|form| {
            let m = syms.intern(&without_member(form));
            let this = syms.intern("self");
            let self_param = |name: Sym, ty: Option<TyForm>| ParamForm {
                name,
                label: name,
                ty,
                is_self: name == this,
                mutable: false,
                named_only: false,
                variadic: false,
                default: None,
                span,
                name_span: span,
            };
            let mut params = vec![self_param(this, None)];
            for (i, f) in form.0.iter().enumerate() {
                params.push(self_param(syms.intern(f), Some(TyForm::Param(i))));
            }
            let build = FnForm {
                name: m,
                public: true,
                attrs: Vec::new(),
                generics: Vec::new(),
                bounds: Vec::new(),
                params,
                ret: Some(TyForm::SelfAssoc(m)),
                yields: false,
                lends: None,
                uses: Vec::new(),
                body: BodyForm::None,
                span,
            };
            let n = form.0.len();
            TraitForm {
                name: syms.intern(&without_name(form)),
                public: true,
                attrs: Vec::new(),
                record: true,
                alias: false,
                generics: (0..n).map(|i| syms.intern(&format!("P{i}"))).collect(),
                generic_defaults: vec![None; n],
                supers: Vec::new(),
                assoc: vec![AssocForm {
                    name: m,
                    bounds: Vec::new(),
                    default: None,
                }],
                methods: vec![build],
                span,
            }
        })
        .collect()
}

/// Per form and record shape it fits, its impl: `impl<P0, F0, F1> W<P0> for (u: F0, w: F1)`
/// with `type R = (to: P0, w: F1)` and `fn R(self, to: P0) -> .. = (to: to, w: self.w)`.
pub fn without_impls(
    forms: &[Form],
    shapes: &[Vec<String>],
    syms: &mut Syms,
    span: Span,
) -> Vec<(String, ImplForm)> {
    let mut out = Vec::new();
    let this = syms.intern("self");
    for form in forms {
        let m = syms.intern(&without_member(form));
        let tr = syms.intern(&without_name(form));
        let np = form.0.len();
        for fs in shapes {
            let Some(made) = without_shape(form, fs) else {
                continue;
            };
            // The impl's parameters: `P0..` for the form's added fields, then `F0..` for the
            // record's own.
            let mut generics: Vec<Sym> = (0..np).map(|i| syms.intern(&format!("P{i}"))).collect();
            generics.extend((0..fs.len()).map(|i| syms.intern(&format!("F{i}"))));
            let ty_of = |f: &String| match form.0.iter().position(|p| p == f) {
                Some(k) => TyForm::Param(k),
                None => TyForm::Param(np + fs.iter().position(|x| x == f).unwrap_or(0)),
            };
            let made_ty = TyForm::Record(made.iter().map(|f| (syms.intern(f), ty_of(f))).collect());
            let param = |name: Sym, ty: Option<TyForm>| ParamForm {
                name,
                label: name,
                ty,
                is_self: name == this,
                mutable: false,
                named_only: false,
                variadic: false,
                default: None,
                span,
                name_span: span,
            };
            let mut params = vec![param(this, None)];
            for (k, f) in form.0.iter().enumerate() {
                params.push(param(syms.intern(f), Some(TyForm::Param(k))));
            }
            let added = form.0.iter().map(|f| syms.intern(f)).collect();
            let kept = (made.iter())
                .filter(|f| !form.0.contains(f))
                .map(|f| syms.intern(f))
                .collect();
            let build = FnForm {
                name: m,
                public: true,
                attrs: Vec::new(),
                generics: Vec::new(),
                bounds: Vec::new(),
                params,
                ret: Some(made_ty.clone()),
                yields: false,
                lends: None,
                uses: Vec::new(),
                body: BodyForm::Synth(crate::synth::SynthBody::Without { added, kept }),
                span,
            };
            let imp = ImplForm {
                attrs: Vec::new(),
                generics,
                bounds: Vec::new(),
                trait_: Some(TyForm::Named(tr, (0..np).map(TyForm::Param).collect())),
                self_ty: TyForm::Record(
                    (fs.iter().enumerate())
                        .map(|(k, f)| (syms.intern(f), TyForm::Param(np + k)))
                        .collect(),
                ),
                assoc: vec![(m, span, made_ty)],
                methods: vec![build],
                lets: Vec::new(),
                derived: false,
                span,
            };
            out.push((name(fs), imp));
        }
    }
    out
}

/// `pub struct (lo:, hi:)<A, B>: pub lo: A, pub hi: B` for each shape.
pub fn structs(shapes: &[Vec<String>], syms: &mut Syms, span: Span) -> Vec<StructForm> {
    (shapes.iter())
        .filter(|fs| fs.len() <= 26)
        .map(|fs| {
            let generics = capitals(syms, fs.len());
            let fields = (fs.iter().enumerate())
                .map(|(i, f)| FieldForm {
                    name: syms.intern(f),
                    ty: TyForm::Param(i),
                    public: true,
                    // A record's fields are assignable through a `var` that holds it (R02.1.1).
                    var: true,
                    attrs: Vec::new(),
                    default: None,
                    span,
                })
                .collect();
            StructForm {
                name: syms.intern(&name(fs)),
                public: true,
                attrs: Vec::new(),
                record: true,
                generic_defaults: vec![None; fs.len()],
                generics,
                bounds: Vec::new(),
                fields,
                methods: Vec::new(),
                lets: Vec::new(),
                consts: Vec::new(),
                span,
            }
        })
        .collect()
}

/// The `@role(positions)` trait, in whichever module declares it, and the names of its members marked
/// `dom_count`, `dom_nth` and `dom_ord`: what a domain the compiler declares implements, with
/// the `@role(order)` trait, whose members all have bodies.
pub struct Positions {
    /// The `@role(positions)` trait.
    pub tr: Sym,
    /// The `@role(order)` trait.
    pub order: Option<Sym>,
    pub count: Sym,
    pub nth: Sym,
    pub ord: Sym,
}

/// The positions trait's members, by the intrinsic each one's body is (R04.2.11).
pub fn positions(forms: &[&crate::decls::forms::Form]) -> Option<Positions> {
    use crate::decls::forms::{intrinsic_name, BodyForm};
    use cpl_next_syntax::ast::{Expr, FnBody};
    let (mut marks, mut order) = (None, None);
    for f in forms {
        if let crate::decls::forms::Form::Trait(t) = f {
            if form_fills(&t.attrs, Role::Positions) {
                marks = Some(t);
            }
            if form_fills(&t.attrs, Role::Order) {
                order = Some(t.name);
            }
        }
    }
    let t = marks?;
    let member = |id: &str| {
        t.methods.iter().find_map(|m| match &m.body {
            BodyForm::Written(FnBody::Expr(Expr::Intrinsic(i)))
                if intrinsic_name(i).as_deref() == Some(id) =>
            {
                Some(m.name)
            }
            _ => None,
        })
    };
    Some(Positions {
        tr: t.name,
        order,
        count: member("dom_count")?,
        nth: member("dom_nth")?,
        ord: member("dom_ord")?,
    })
}

/// A record of domains is the product domain over its fields, walked row-major with the last
/// field fastest, as core's tuples are (R04.2.10): per shape, `impl<A: Domain, ..> Domain for
/// (lo: A, ..)` giving the members the `@role(positions)` trait marks `dom_count`, `dom_nth` and
/// `dom_ord`.
pub fn domain_impls(
    positions: Option<&Positions>,
    shapes: &[Vec<String>],
    syms: &mut Syms,
    span: Span,
) -> Vec<(String, ImplForm)> {
    use crate::synth::SynthBody;
    let Some(p) = positions else {
        return Vec::new();
    };
    let (count, nth, ord) = (p.count, p.nth, p.ord);
    let tr = p.tr;
    let this = syms.intern("self");
    let int = syms.intern("Int");
    let k = syms.intern("k");
    let mut out = Vec::new();
    for fs in shapes.iter().filter(|fs| !fs.is_empty() && fs.len() <= 26) {
        let generics = capitals(syms, fs.len());
        let fields: Vec<Sym> = fs.iter().map(|f| syms.intern(f)).collect();
        let record = TyForm::Record(
            (fields.iter().enumerate())
                .map(|(i, f)| (*f, TyForm::Param(i)))
                .collect(),
        );
        let param = |name: Sym, ty: Option<TyForm>| ParamForm {
            name,
            label: name,
            ty,
            is_self: name == this,
            mutable: false,
            named_only: false,
            variadic: false,
            default: None,
            span,
            name_span: span,
        };
        let method =
            |getter: bool, name: Sym, params: Vec<ParamForm>, ret: TyForm, body: SynthBody| {
                FnForm {
                    name,
                    public: false,
                    attrs: match getter {
                        true => vec![AttrForm {
                            attr: Decorator::Get,
                            node: None,
                        }],
                        false => Vec::new(),
                    },
                    generics: Vec::new(),
                    bounds: Vec::new(),
                    params,
                    ret: Some(ret),
                    yields: false,
                    lends: None,
                    uses: Vec::new(),
                    body: BodyForm::Synth(body),
                    span,
                }
            };
        let methods = vec![
            method(
                true,
                count,
                Vec::new(),
                TyForm::Named(int, Vec::new()),
                SynthBody::ProductCount { count, n: fs.len() },
            ),
            method(
                false,
                nth,
                vec![param(k, Some(TyForm::Named(int, Vec::new())))],
                record.clone(),
                SynthBody::ProductNth {
                    nth,
                    count,
                    fields: fields.clone(),
                },
            ),
            method(
                true,
                ord,
                vec![param(this, None)],
                TyForm::Named(int, Vec::new()),
                SynthBody::ProductOrd {
                    ord,
                    count,
                    fields: fields.clone(),
                },
            ),
        ];
        // `impl<A: tr, ..> tr for (lo: A, ..)`.
        let imp = |tr: Sym, methods: Vec<FnForm>| ImplForm {
            attrs: Vec::new(),
            generics: generics.clone(),
            bounds: (generics.iter())
                .map(|g| BoundForm {
                    param: *g,
                    tr: TyForm::Named(tr, Vec::new()),
                })
                .collect(),
            trait_: Some(TyForm::Named(tr, Vec::new())),
            self_ty: record.clone(),
            assoc: Vec::new(),
            methods,
            lets: Vec::new(),
            derived: false,
            span,
        };
        // A record of ordered domains is ordered, as a tuple of them is (R04.2.31).
        if let Some(o) = &p.order {
            let o = *o;
            out.push((name(fs), imp(o, Vec::new())));
        }
        out.push((name(fs), imp(tr, methods)));
    }
    out
}
