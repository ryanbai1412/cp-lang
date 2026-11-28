//! Declarations the compiler writes for a program: an enum's structs and the `Eq` and `Ord` a
//! struct derives from its fields.
use crate::attrs::Decorator;
use crate::decls::forms::{
    AttrForm, BodyForm, FnForm, Form, ImplForm, ParamForm, TraitForm, TyForm,
};
use crate::decls::literal_int;
use crate::decls::unnamed;
use crate::decls::{fills, form_fills, Role};
use crate::diag::Msg;
use crate::records::marker;
use crate::sym::Sym;
use cpl_next_ir::ids::ItemId;
use cpl_next_ir::ty::StructId;
use cpl_next_ir::unbuilt::Unbuilt;
use cpl_next_syntax::ast::{self as next, AstNode as _};
use cpl_syntax::ast::{self, FnBody, StmtKind, TypeKind};
use cpl_syntax::decorators::Mark;
use std::collections::HashMap;

/// An enum's tag errors (R02.7.2): a variant whose token another one before it has (E0407), and
/// an enum-level `@tags(..)`, which is removed: each variant writes its own `@tag` (decision
/// 459). With one plain tag per variant, the help is the rewrite.
pub(crate) fn enum_tags(
    item: &ItemId,
    it: &next::Item,
    e: &crate::decls::forms::EnumForm,
    syms: &crate::sym::Syms,
) -> Vec<Msg> {
    let tokens: Vec<String> = e.variants.iter().map(|v| token(v, syms)).collect();
    let mut out: Vec<Msg> = (e.variants.iter().enumerate())
        .filter_map(|(i, v)| {
            let t = &tokens[i];
            let first = (0..i).find(|&w| &tokens[w] == t)?;
            let what = format!(
                "`{}` reads and prints as `{t}`, as `{}` does: tags are distinct within an enum",
                syms.text(v.name),
                syms.text(e.variants[first].name)
            );
            let m = Msg::verdict(cpl_diag::codes::DUPLICATE_VARIANT_TAG, what);
            Some(Msg::at(item.at(v.name_span), m))
        })
        .collect();
    let level = || -> Option<Msg> {
        let tags = |x: &next::Decorator| {
            x.name().and_then(|n| Decorator::of(n.text())) == Some(Decorator::Tags)
        };
        let x = it.decorators().into_iter().find(tags)?;
        let text = cpl_diag::msgs::check::ENUM_LEVEL_TAGS_WAS.say(&[]);
        let mut m = Msg::at(
            item.at(x.syntax().span()),
            Msg::verdict(cpl_diag::codes::REMOVED_SYNTAX, text),
        );
        let tag = |a: next::Arg| match a.value()? {
            next::Expr::Literal(l) => {
                let t = l.token()?;
                let text = t.text();
                let parts = cpl_next_syntax::literal::string_parts(text);
                match (t.kind(), parts.as_slice()) {
                    (
                        cpl_next_syntax::TokenKind::Str,
                        [cpl_next_syntax::literal::StrPart::Text(p)],
                    ) => Some(text[p.start as usize..p.end as usize].to_string()),
                    _ => None,
                }
            }
            _ => None,
        };
        let args: Vec<next::Arg> = x.args().map_or(Vec::new(), |a| a.args().collect());
        let each = (args.len() == e.variants.len())
            .then(|| args.into_iter().zip(&e.variants))
            .into_iter()
            .flatten()
            .map(|(a, v)| Some(format!("@tag({:?}) {}", tag(a)?, syms.text(v.name))))
            .collect::<Option<Vec<_>>>()
            .filter(|w| !w.is_empty());
        let name = syms.text(e.name).to_string();
        m.help = each.map(|w| cpl_diag::habits::ENUM_TAGS.say(&[&name, &w.join(" | ")]));
        Some(m)
    };
    out.extend(level());
    out
}

/// An enum as structs: its own, with no fields (the parser makes its methods an `impl`), then one
/// per variant holding that variant's payload, named `Enum.Variant`, which no source name can be
/// (R02.7).
pub(crate) fn enum_structs(
    e: &crate::decls::forms::EnumForm,
    syms: &mut crate::sym::Syms,
) -> Vec<crate::decls::forms::StructForm> {
    use crate::decls::forms::{FieldForm, StructForm};
    let one = |name: Sym, public: bool, attrs: Vec<AttrForm>, fields: Vec<FieldForm>| StructForm {
        name,
        public,
        attrs,
        record: false,
        generics: e.generics.clone(),
        generic_defaults: vec![None; e.generics.len()],
        bounds: Vec::new(),
        fields,
        methods: Vec::new(),
        lets: Vec::new(),
        consts: Vec::new(),
        span: e.span,
    };
    let mut out = vec![one(e.name, e.public, e.attrs.clone(), Vec::new())];
    for v in &e.variants {
        let name = syms.intern(&format!("{}.{}", syms.text(e.name), syms.text(v.name)));
        let fields = (v.fields.iter())
            .map(|f| FieldForm {
                public: true,
                ..f.clone()
            })
            .collect();
        out.push(one(name, e.public, Vec::new(), fields));
    }
    out
}

/// A core trait every struct gets from its fields, and how.
pub(crate) enum Derivable {
    /// `@derive`, with its one method.
    Each(Box<FnForm>),
    /// `@role(derive_fields)`, with the `@role(pack)` struct and the trait's methods.
    Fields(Sym, Vec<FnForm>),
}

/// Per struct a module declares, the impls it gets structurally (R08.1.6), made after every written
/// impl and only where none is: `Eq` field by field, `Ord` lexicographic in declaration order, and
/// `Clone` cloning each field (R05.2.2).
pub(crate) fn module_derived(
    items: &[(ItemId, Form)],
    each: &[(Sym, Derivable)],
    cmp: &[(Sym, Sym, bool)],
    syms: &mut crate::sym::Syms,
) -> Vec<(ItemId, String, ImplForm, bool)> {
    let mut out = Vec::new();
    let a = syms.intern("a");
    let b = syms.intern("b");
    for (item, form) in items {
        match form {
            Form::Enum(e) => {
                for (tr, how) in each {
                    let fns = match how {
                        Derivable::Fields(_, fns) => {
                            let variants: Vec<(Sym, String)> = e
                                .variants
                                .iter()
                                .map(|v| (v.name, token(v, syms)))
                                .collect();
                            passing(fns, a, |m| SynthBody::VariantName {
                                method: m,
                                variants: variants.clone(),
                            })
                        }
                        Derivable::Each(m) if made(m) => {
                            let variants = (e.variants.iter())
                                .map(|v| (v.name, token(v, syms), v.fields.len()))
                                .collect();
                            vec![made_fn(
                                m,
                                SynthBody::VariantByTag {
                                    method: m.name,
                                    variants,
                                },
                            )]
                        }
                        Derivable::Each(_) => continue,
                    };
                    let imp = derived_impl(e.name, &e.generics, *tr, fns, e.span);
                    out.push((item.clone(), syms.text(e.name).to_string(), imp, false));
                }
            }
            Form::Struct(sd) => {
                let cached = |f: &&crate::decls::forms::FieldForm| {
                    f.attrs.iter().any(|x| x.attr == Decorator::Cache)
                };
                let all: Vec<Sym> = sd.fields.iter().map(|f| f.name).collect();
                let uncached: Vec<Sym> = sd
                    .fields
                    .iter()
                    .filter(|f| !cached(f))
                    .map(|f| f.name)
                    .collect();
                let name = syms.text(sd.name).to_string();
                // The traits `==` and `<` call: the comparisons a struct derives.
                for &(tr, method, eq) in cmp {
                    let body = match eq {
                        true => SynthBody::EqFields(uncached.clone()),
                        false => SynthBody::LtFields(uncached.clone()),
                    };
                    let f = synth_fn(
                        method,
                        vec![plain(a, sd.span), plain(b, sd.span)],
                        body,
                        sd.span,
                    );
                    out.push((
                        item.clone(),
                        name.clone(),
                        derived_impl(sd.name, &sd.generics, tr, vec![f], sd.span),
                        false,
                    ));
                }
                for (tr, how) in each {
                    let (fns, caches) = match how {
                        // A made value needs every field, and a derive gives no `@cache` field
                        // (R05.10.13): a struct with one makes itself by hand.
                        Derivable::Each(m) if made(m) && sd.fields.iter().any(|f| cached(&f)) => {
                            continue
                        }
                        Derivable::Each(m) if made(m) => (
                            vec![made_fn(
                                m,
                                SynthBody::MadeEach {
                                    method: m.name,
                                    fields: uncached.clone(),
                                },
                            )],
                            false,
                        ),
                        // Only a derive that rebuilds its value copies the `@cache` fields.
                        Derivable::Each(m) => {
                            let body = SynthBody::EachField {
                                method: m.name,
                                fields: all.clone(),
                            };
                            (
                                vec![synth_fn(m.name, vec![plain(a, sd.span)], body, sd.span)],
                                true,
                            )
                        }
                        Derivable::Fields(_, fns) => (
                            passing(fns, a, |m| SynthBody::PackFields {
                                method: m,
                                fields: uncached.clone(),
                            }),
                            false,
                        ),
                    };
                    let imp = derived_impl(sd.name, &sd.generics, *tr, fns, sd.span);
                    out.push((item.clone(), name.clone(), imp, caches));
                }
            }
            _ => {}
        }
    }
    out
}

/// A made fn: one of the language's own, with its signature from the trait it meets.
fn synth_fn(name: Sym, params: Vec<ParamForm>, body: SynthBody, span: cpl_diag::Span) -> FnForm {
    FnForm {
        name,
        public: false,
        attrs: Vec::new(),
        generics: Vec::new(),
        bounds: Vec::new(),
        params,
        ret: None,
        yields: false,
        lends: None,
        uses: Vec::new(),
        body: BodyForm::Synth(body),
        span,
    }
}

fn plain(name: Sym, span: cpl_diag::Span) -> ParamForm {
    ParamForm {
        name,
        label: name,
        ty: None,
        is_self: false,
        mutable: false,
        named_only: false,
        variadic: false,
        default: None,
        span,
        name_span: span,
    }
}

/// A parameter as the trait's method declares it, its type and default left to the trait.
fn untyped(p: &ParamForm) -> ParamForm {
    ParamForm {
        ty: None,
        default: None,
        ..p.clone()
    }
}

/// A `@derive` method making a `Self`: its parameters as the trait declares them.
fn made_fn(m: &FnForm, body: SynthBody) -> FnForm {
    synth_fn(m.name, m.params.iter().map(untyped).collect(), body, m.span)
}

/// Each method of a `@derive_fields` trait, passing its other parameters on as they came.
fn passing(fns: &[FnForm], a: Sym, body: impl Fn(Sym) -> SynthBody) -> Vec<FnForm> {
    (fns.iter())
        .map(|m| {
            let params =
                std::iter::once(plain(a, m.span)).chain(m.params.iter().skip(1).map(untyped));
            synth_fn(m.name, params.collect(), body(m.name), m.span)
        })
        .collect()
}

fn derived_impl(
    ty: Sym,
    generics: &[Sym],
    tr: Sym,
    methods: Vec<FnForm>,
    span: cpl_diag::Span,
) -> ImplForm {
    ImplForm {
        attrs: Vec::new(),
        generics: generics.to_vec(),
        bounds: Vec::new(),
        trait_: Some(TyForm::Named(tr, Vec::new())),
        self_ty: TyForm::Named(ty, (0..generics.len()).map(TyForm::Param).collect()),
        assoc: Vec::new(),
        methods,
        lets: Vec::new(),
        derived: true,
        span,
    }
}

/// The traits marked `@derive`, with their one method, and the `@role(derive_fields)` trait,
/// with its methods and the `@role(pack)` struct: every struct whose fields all have one gets it field by
/// field (R08.1.6).
pub(crate) fn derive_each(forms: &[&Form]) -> Vec<(Sym, Derivable)> {
    let pack = forms.iter().find_map(|f| match f {
        Form::Struct(sd) if form_fills(&sd.attrs, Role::Pack) => Some(sd.name),
        _ => None,
    });
    let derive = |t: &TraitForm| t.attrs.iter().any(|a| a.attr == Decorator::Derive);
    forms
        .iter()
        .filter_map(|f| {
            let Form::Trait(t) = f else {
                return None;
            };
            let how = match (&t.methods[..], pack) {
                ([m], _) if derive(t) && t.assoc.is_empty() => Derivable::Each(Box::new(m.clone())),
                (_, Some(p)) if form_fills(&t.attrs, Role::DeriveFields) => {
                    Derivable::Fields(p, t.methods.clone())
                }
                _ => return None,
            };
            Some((t.name, how))
        })
        .collect()
}

/// `@get fn name(self) = self.name`: how a struct meets a bodiless `@get` member its impl leaves
/// out (R12.11.3).
pub(crate) fn field_getter(
    field: Sym,
    span: cpl_diag::Span,
    syms: &mut crate::sym::Syms,
) -> FnForm {
    let this = syms.intern("self");
    FnForm {
        name: field,
        public: false,
        attrs: vec![AttrForm {
            attr: Decorator::Get,
            node: None,
        }],
        generics: Vec::new(),
        bounds: Vec::new(),
        params: vec![ParamForm {
            name: this,
            label: this,
            ty: None,
            is_self: true,
            mutable: false,
            named_only: false,
            variadic: false,
            default: None,
            span,
            name_span: span,
        }],
        ret: None,
        yields: false,
        lends: None,
        uses: Vec::new(),
        body: BodyForm::Synth(SynthBody::Getter(field)),
        span,
    }
}

/// An impl's `let count = e`: the self-less getter `@get fn count() = e` that meets its trait's
/// member of that name (R04.2.26).
pub(crate) fn value_fn(l: &crate::decls::forms::LetForm) -> Option<FnForm> {
    Some(FnForm {
        name: l.name,
        public: false,
        attrs: vec![AttrForm {
            attr: Decorator::Get,
            node: None,
        }],
        generics: Vec::new(),
        bounds: Vec::new(),
        params: Vec::new(),
        ret: l.ty.clone(),
        yields: false,
        lends: None,
        uses: Vec::new(),
        body: BodyForm::Value(l.value.clone()?),
        span: l.span,
    })
}

/// `@get fn name(self) -> T` with no body: a field requirement, met by a field `name` (R08.1.7).
pub(crate) fn field_requirement(
    field: Sym,
    ty: TyForm,
    span: cpl_diag::Span,
    syms: &mut crate::sym::Syms,
) -> FnForm {
    let this = syms.intern("self");
    FnForm {
        name: field,
        public: true,
        attrs: vec![AttrForm {
            attr: Decorator::Get,
            node: None,
        }],
        generics: Vec::new(),
        bounds: Vec::new(),
        params: vec![ParamForm {
            name: this,
            label: this,
            ty: None,
            is_self: true,
            mutable: false,
            named_only: false,
            variadic: false,
            default: None,
            span,
            name_span: span,
        }],
        ret: Some(ty),
        yields: false,
        lends: None,
        uses: Vec::new(),
        body: BodyForm::None,
        span,
    }
}

/// Per record shape with a field for each member of a trait made only of bodiless `@get` members,
/// a bodiless impl of it, written with the trait: `impl<V, A> Target<V> for (to: V, w: A)`, its
/// members then met by the fields (R12.11.3). A written impl replaces it.
pub(crate) fn field_trait_impls(
    items: &[(ItemId, Form)],
    shapes: &[Vec<String>],
    syms: &mut crate::sym::Syms,
) -> Vec<(ItemId, String, ImplForm)> {
    let mut out = Vec::new();
    for (item, form) in items {
        let Form::Trait(t) = form else {
            continue;
        };
        if t.alias || !t.supers.is_empty() || t.methods.is_empty() {
            continue;
        }
        let getter = |f: &FnForm| f.attrs.iter().any(|a| a.attr == Decorator::Get);
        let member = |f: &FnForm| match (&f.body, &f.ret) {
            (BodyForm::None, Some(TyForm::Written(ty)))
                if f.params.len() <= 1 && getter(f) && writable(ty) =>
            {
                Some((f.name, ty.clone()))
            }
            _ => None,
        };
        let Some(members) = t.methods.iter().map(member).collect::<Option<Vec<_>>>() else {
            continue;
        };
        let mut taken: Vec<String> = t
            .generics
            .iter()
            .map(|g| syms.text(*g).to_string())
            .collect();
        for (_, ty) in &members {
            let names = ty.syntax().descendants().filter_map(next::PathType::cast);
            taken.extend(names.map(|p| crate::cycles::path_text(&p)));
        }
        let fresh: Vec<String> = (b'A'..=b'Z')
            .map(|c| (c as char).to_string())
            .filter(|c| !taken.contains(c))
            .collect();
        let own = t.generics.len();
        for fs in shapes {
            let names: Vec<Sym> = fs.iter().map(|f| syms.intern(f)).collect();
            if !members.iter().all(|(m, _)| names.contains(m)) {
                continue;
            }
            let mut generics = t.generics.clone();
            let mut extra = fresh.iter();
            let mut fields = Vec::new();
            for f in &names {
                match members.iter().find(|(m, _)| m == f) {
                    Some((_, ty)) => fields.push((*f, TyForm::Written(ty.clone()))),
                    None => {
                        let Some(g) = extra.next() else { break };
                        generics.push(syms.intern(g));
                        fields.push((*f, TyForm::Param(generics.len() - 1)));
                    }
                }
            }
            if fields.len() != fs.len() {
                continue;
            }
            let imp = ImplForm {
                attrs: Vec::new(),
                generics,
                bounds: Vec::new(),
                trait_: Some(TyForm::Named(t.name, (0..own).map(TyForm::Param).collect())),
                self_ty: TyForm::Record(fields),
                assoc: Vec::new(),
                methods: Vec::new(),
                lets: Vec::new(),
                derived: false,
                span: t.span,
            };
            out.push((item.clone(), crate::records::name(fs), imp));
        }
    }
    out
}

/// A field type an impl can write as it is: no `Self`, no associated type.
/// Whether a member's type can be a field's as written: no `Self` and no path through a module.
fn writable(t: &next::Type) -> bool {
    use cpl_next_syntax::TokenKind;
    let types = |args: Vec<next::GenericArg>| {
        args.into_iter().all(|a| match a {
            next::GenericArg::Type(t) => writable(&t),
            next::GenericArg::Param(_) => false,
        })
    };
    match t {
        next::Type::Path(p) => {
            let segs: Vec<_> = p.segments().collect();
            segs.len() == 1
                && segs[0].kind() != TokenKind::SelfKw
                && types(p.generic_args().map_or(Vec::new(), |a| a.args().collect()))
        }
        next::Type::Optional(o) => o.inner().is_some_and(|t| writable(&t)),
        next::Type::Tuple(ts) => ts.elems().iter().all(|(_, t)| writable(t)),
        next::Type::Array(a) => {
            types(a.dims().map_or(Vec::new(), |d| d.args().collect()))
                && a.elem().is_some_and(|t| writable(&t))
        }
        _ => false,
    }
}

/// The `@role(convert)` trait with its one method, and the `@role(variant_check)` fn: what a
/// payload-free enum's conversions are written with.
pub(crate) fn convert_marked(forms: &[&Form]) -> Option<(Sym, Sym, Sym)> {
    let (tr, method) = forms.iter().find_map(|f| match f {
        Form::Trait(t) if form_fills(&t.attrs, Role::Convert) => match &t.methods[..] {
            [m] => Some((t.name, m.name)),
            _ => None,
        },
        _ => None,
    })?;
    let check = forms.iter().find_map(|f| match f {
        Form::Fn(f) if form_fills(&f.attrs, Role::VariantCheck) => Some(f.name),
        _ => None,
    })?;
    Some((tr, method, check))
}

/// Per payload-free enum a module declares, `Int(e)` and `E(x)` (R02.7): `impl Into<Int> for E`,
/// a `match` giving each variant's value, and `impl Into<E> for Int`, which checks `x` is a
/// variant's value and gives that variant. With the `@role(positions)` trait, the enum is also a
/// domain whose positions are its variants in declaration order (R02.7, R04.2.22).
pub(crate) fn enum_conversions(
    items: &[(ItemId, Form)],
    (tr, method, check): (Sym, Sym, Sym),
    positions: Option<&crate::records::Positions>,
    syms: &mut crate::sym::Syms,
) -> Vec<(ItemId, String, ImplForm)> {
    let mut out = Vec::new();
    let this = syms.intern("self");
    let k = syms.intern("k");
    let int = TyForm::Named(syms.intern("Int"), Vec::new());
    for (item, form) in items {
        let Form::Enum(e) = form else {
            continue;
        };
        let plain = (e.variants.iter())
            .all(|v| v.fields.is_empty() && v.attrs.iter().all(|a| a.attr != Decorator::Tag));
        if !e.generics.is_empty() || e.variants.is_empty() || !plain {
            continue;
        }
        let span = e.span;
        let variants: Vec<Sym> = e.variants.iter().map(|v| v.name).collect();
        let enum_ty = TyForm::Named(e.name, Vec::new());
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
        let func =
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
        let imp = |tr: TyForm, for_: TyForm, methods: Vec<FnForm>| ImplForm {
            attrs: Vec::new(),
            generics: Vec::new(),
            bounds: Vec::new(),
            trait_: Some(tr),
            self_ty: for_,
            assoc: Vec::new(),
            methods,
            lets: Vec::new(),
            derived: false,
            span,
        };
        let name = syms.text(e.name).to_string();
        let to_int = func(
            false,
            method,
            vec![param(this, None)],
            int.clone(),
            SynthBody::EnumToInt(variants.clone()),
        );
        let to_enum = func(
            false,
            method,
            vec![param(this, None)],
            enum_ty.clone(),
            SynthBody::IntToEnum {
                check,
                variants: variants.clone(),
            },
        );
        out.push((
            item.clone(),
            name.clone(),
            imp(
                TyForm::Named(tr, vec![int.clone()]),
                enum_ty.clone(),
                vec![to_int],
            ),
        ));
        out.push((
            item.clone(),
            name.clone(),
            imp(
                TyForm::Named(tr, vec![enum_ty.clone()]),
                int.clone(),
                vec![to_enum],
            ),
        ));
        let Some(p) = positions else {
            continue;
        };
        let (count, nth, ord) = (p.count, p.nth, p.ord);
        let methods = vec![
            func(
                true,
                count,
                Vec::new(),
                int.clone(),
                SynthBody::EnumCount(variants.len()),
            ),
            func(
                false,
                nth,
                vec![param(k, Some(int.clone()))],
                enum_ty.clone(),
                SynthBody::IntToEnum {
                    check,
                    variants: variants.clone(),
                },
            ),
            func(
                true,
                ord,
                vec![param(this, None)],
                int.clone(),
                SynthBody::EnumToInt(variants.clone()),
            ),
        ];
        let domain = p.tr;
        out.push((
            item.clone(),
            name.clone(),
            imp(TyForm::Named(domain, Vec::new()), enum_ty.clone(), methods),
        ));
        // An enum's variants are in declaration order (R04.2.31).
        if let Some(o) = &p.order {
            let o = *o;
            out.push((
                item.clone(),
                name.clone(),
                imp(TyForm::Named(o, Vec::new()), enum_ty.clone(), Vec::new()),
            ));
        }
    }
    out
}

/// The structs marked `@memo_table(word)`, by word: the module that declares the table a
/// `@memo(word)` fn keeps, and its name there. Any module may declare one (R06.4.1).
pub(crate) fn memo_tables(
    forms: &[&[(ItemId, Form)]],
    syms: &mut crate::sym::Syms,
) -> Vec<(Sym, (usize, Sym))> {
    let mut out = Vec::new();
    for (mi, items) in forms.iter().enumerate() {
        for (_, form) in items.iter() {
            let Form::Struct(sd) = form else {
                continue;
            };
            let word = sd
                .attrs
                .iter()
                .find(|a| a.attr == Decorator::MemoTable)
                .and_then(|a| attr_word(a, syms));
            out.extend(word.map(|w| (w, (mi, sd.name))));
        }
    }
    out
}

/// The one bare word a `@memo(word)` or `@memo_table(word)` names.
fn attr_word(x: &AttrForm, syms: &mut crate::sym::Syms) -> Option<Sym> {
    let args: Vec<next::Arg> = x.node.as_ref()?.args()?.args().collect();
    match &args[..] {
        [a] => match a.value()? {
            next::Expr::Name(n) => Some(syms.intern(n.ident()?.text())),
            _ => None,
        },
        _ => None,
    }
}

/// The name every module sees `@memo(word)`'s table under, whatever it imports: none a program
/// can write, so the expansion finds the marked declaration and nothing in scope.
pub fn memo_key(word: &str) -> String {
    format!("$memo({word})")
}

/// A `@memo` that does not spell its storage: the table is spelled, never picked (R06.4.1).
pub fn unspelled_memo(x: &ast::Decorator) -> Option<cpl_next_ir::msg::Msg> {
    (x.mark == Some(Mark::Memo) && x.args.is_empty()).then(|| {
        cpl_next_ir::msg::Msg::verdict(
            cpl_diag::codes::BAD_MEMO,
            "`@memo` spells its table: write `@memo(array)` or `@memo(map)`",
        )
    })
}

fn memo_word(x: &ast::Decorator) -> Option<String> {
    match &x.args[..] {
        [a] => match &a.value.kind {
            ast::ExprKind::Name(w) => Some(w.clone()),
            _ => None,
        },
        _ => None,
    }
}

/// The table a `@memo(word)` fn keeps: one some struct is marked with, for a fn whose
/// parameters and result are written.
pub(crate) fn memo_table<'s>(
    s: &'s ast::Stmt,
    tables: &dyn Fn(&str) -> bool,
) -> Option<(&'s ast::FnDecl, String)> {
    let StmtKind::Fn(f) = &s.kind else {
        return None;
    };
    let x = s.decorators.iter().find(|x| x.mark == Some(Mark::Memo))?;
    let plain = |p: &ast::Param| p.ty.is_some() && !p.mutable && !p.variadic && !p.lazy;
    let typed = f.ret.is_some() && !f.params.is_empty() && f.params.iter().all(plain);
    (typed && f.generics.is_empty() && f.lends.is_none()).then_some(())?;
    let w = memo_word(x).filter(|w| tables(w))?;
    Some((f, memo_key(&w)))
}

/// The name `@memo` fn `f`'s clear is declared under, written `f.clear()` (R06.4.1).
pub fn memo_clear_name(f: &str) -> String {
    format!("{f}.clear")
}

/// Each parameter's reach when it is jagged, with its range as written: `k: 0..=n` of `n: 0..=9`
/// reaches `0..=9` (R06.4.1). `None` for every other parameter. A fn's parameters and a nested
/// fn's read it alike.
pub fn jagged(params: &[(String, Option<next::Type>)]) -> Vec<Option<((i64, i64), next::Type)>> {
    use cpl_next_syntax::TokenKind;
    use next::Expr;
    fn reach(e: &Expr, known: &HashMap<String, (i64, i64)>) -> Option<(i64, i64)> {
        match e {
            Expr::Paren(p) => reach(&p.inner()?, known),
            Expr::Name(n) => known.get(n.ident()?.text()).copied(),
            Expr::Bin(b) if matches!(b.op()?.kind(), TokenKind::Plus | TokenKind::Minus) => {
                let ((a, c), k) = (reach(&b.lhs()?, known)?, literal_int(&b.rhs()?)?);
                let k = match b.op()?.kind() {
                    TokenKind::Minus => k.checked_neg()?,
                    _ => k,
                };
                Some((a.checked_add(k)?, c.checked_add(k)?))
            }
            _ => literal_int(e).map(|v| (v, v)),
        }
    }
    let mut known = HashMap::new();
    let mut out = Vec::new();
    for (name, ty) in params {
        let range = match ty {
            Some(next::Type::Range(r)) => match r.expr() {
                Some(Expr::Range(re)) => Some(re),
                _ => None,
            },
            _ => None,
        };
        let Some(re) = range else {
            out.push(None);
            continue;
        };
        let (lo, hi, inclusive) = (re.lo(), re.hi(), re.is_inclusive());
        let lit = |e: &Option<Expr>, open: i64| match e {
            Some(e) => literal_int(e),
            None => Some(open),
        };
        if let (Some(a), Some(b)) = (lit(&lo, i64::MIN + 1), lit(&hi, i64::MAX)) {
            let b = match (inclusive, &hi) {
                (false, Some(_)) => b.checked_sub(1),
                _ => Some(b),
            };
            if let Some(b) = b {
                known.insert(name.clone(), (a, b));
            }
            out.push(None);
            continue;
        }
        let ends = |e: &Option<Expr>| e.as_ref().map(|e| reach(e, &known));
        let (Some(Some((a, _))), Some(Some((_, b)))) = (ends(&lo), ends(&hi)) else {
            out.push(None);
            continue;
        };
        let Some(b) = b.checked_sub(i64::from(!inclusive)) else {
            out.push(None);
            continue;
        };
        known.insert(name.clone(), (a, b));
        out.push(ty.clone().map(|t| ((a, b), t)));
    }
    out
}

/// A fn with each jagged parameter's type widened, and each one's written type by its name.

/// A made fn's body: what lower builds its IR from, by variant, with no syntax in between. Each
/// is one shape the language derives or desugars; check reads its signature, never a body.
#[derive(Clone, Debug)]
pub enum SynthBody {
    /// `a.x == b.x and ..` over the compared fields, through the `==` operator's trait (R08.1.6).
    EqFields(Vec<Sym>),
    /// `a.x < b.x or (a.x == b.x and ..)`, lexicographic in declaration order (R08.1.6).
    LtFields(Vec<Sym>),
    /// `S(x: a.x.m(), ..)`: a `@derive` trait's one method on each field, `clone` among them
    /// (R08.1.6, R05.2.2).
    EachField { method: Sym, fields: Vec<Sym> },
    /// `S(x: p.m<F>(), ..)`: a `@derive` method making a `Self` from its first parameter, each
    /// field read at its own type, as `Read`'s `read(mut input)` (R10.2.1).
    MadeEach { method: Sym, fields: Vec<Sym> },
    /// `Pack(a.x, Pack(a.y, ())).m(rest..)`: a `@derive_fields` method on the fields packed into
    /// the `@role(pack)` struct, read from `Roles` (R08.1.6).
    PackFields { method: Sym, fields: Vec<Sym> },
    /// An enum read by its tags: the variant `p.variant([tags])` picks, its payload read field by
    /// field (R10.2.1).
    VariantByTag {
        method: Sym,
        variants: Vec<(Sym, String, usize)>,
    },
    /// A `@derive_fields` method on an enum: that method on the variant's token (R10.3.5).
    VariantName {
        method: Sym,
        variants: Vec<(Sym, String)>,
    },
    /// `@get fn name(self) = self.name`: a bodiless `@get` member met by the field of its name
    /// (R12.11.3, R08.1.7).
    Getter(Sym),
    /// A `without` form's member on a record it fits: `fn R(self, to: P0) = (to: to, w: self.w)`,
    /// its added fields from its parameters, then the kept ones from `self` (R08.4.1).
    Without { added: Vec<Sym>, kept: Vec<Sym> },
    /// A record of domains as the product domain, row-major with the last field fastest
    /// (R04.2.10): its size, `A.count * B.count * ..`, through the `@role(positions)` trait's
    /// count member.
    ProductCount { count: Sym, n: usize },
    /// Position `k` of the product: field `i` is `A_i.nth(k / (the sizes after it) % its size)`.
    ProductNth {
        nth: Sym,
        count: Sym,
        fields: Vec<Sym>,
    },
    /// The rank of a position: `((self.a.ord * B.count) + self.b.ord) * ..`.
    ProductOrd {
        ord: Sym,
        count: Sym,
        fields: Vec<Sym>,
    },
    /// `Int(e)` and a payload-free enum's rank as a domain: the variant's position in
    /// declaration order (R02.7, R04.2.22).
    EnumToInt(Vec<Sym>),
    /// `E(x)` and the enum's `nth(k)`: `x` checked through the `@role(variant_check)` fn to be
    /// in `0..n`, then the variant at that position (R02.7, R04.2.22).
    IntToEnum { check: Sym, variants: Vec<Sym> },
    /// A payload-free enum's size as a domain: its number of variants (R04.2.22).
    EnumCount(usize),
}

/// Whether a `@derive` method makes a `Self` from its first parameter, as `Read`'s does.
fn made(m: &FnForm) -> bool {
    m.params.first().is_some_and(|p| p.ty.is_some())
}

/// The token a variant reads and prints as: its `@tag`'s literal, else its name (R02.7.2).
fn token(v: &crate::decls::forms::VariantForm, syms: &crate::sym::Syms) -> String {
    use cpl_next_syntax::literal::{int_value, string_parts, StrPart};
    use cpl_next_syntax::TokenKind;
    let tag = v
        .attrs
        .iter()
        .find(|a| a.attr == Decorator::Tag)
        .and_then(|a| a.node.as_ref());
    let lit = tag
        .and_then(|x| x.args()?.args().next()?.value())
        .and_then(|e| match e {
            next::Expr::Literal(l) => l.token(),
            _ => None,
        });
    let Some(t) = lit else {
        return syms.text(v.name).to_string();
    };
    let text = t.text();
    match t.kind() {
        TokenKind::Str => (string_parts(text).into_iter())
            .filter_map(|p| match p {
                StrPart::Text(s) => Some(&text[s.start as usize..s.end as usize]),
                StrPart::Expr(_) => None,
            })
            .collect(),
        TokenKind::Char => text.trim_matches('\'').to_string(),
        TokenKind::Int => {
            int_value(text).map_or_else(|| syms.text(v.name).to_string(), |k| k.to_string())
        }
        _ => syms.text(v.name).to_string(),
    }
}
