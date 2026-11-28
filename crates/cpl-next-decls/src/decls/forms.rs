//! What a declaration says, as decls reads it: one form per kind, built from the typed tree for
//! a written item and directly by synth and records for a made one (R02.5.2, R08.1.6). The kind
//! builders and the type conversion read only forms, so a made declaration is never written as
//! syntax and parsed back.
use crate::attrs::Decorator;
use crate::sym::Sym;
use crate::synth::SynthBody;
use cpl_diag::Span;
use cpl_next_syntax::ast::{self, AstNode};

/// A type as a declaration writes it: the tree's own node for a written one; for a made one, the
/// parts the conversion of a written one reaches once its names resolve.
#[derive(Clone, Debug)]
pub enum TyForm {
    Written(ast::Type),
    /// The declaration's own type parameter, by index.
    Param(usize),
    /// `Self` of the trait or impl the declaration is in.
    SelfTy,
    /// `Self.R`: an associated type of the trait the declaration is in.
    SelfAssoc(Sym),
    /// A resolved name with its arguments: `Bool`, `Eq<T>`, a record shape's struct.
    Named(Sym, Vec<TyForm>),
    /// `(lo: A, hi: B)` by its field names.
    Record(Vec<(Sym, TyForm)>),
}

/// `T: Tr<A>`: the bounded parameter and the trait it must have.
#[derive(Clone, Debug)]
pub struct BoundForm {
    pub param: Sym,
    pub tr: TyForm,
}

/// A decorator read once, at its node: what it is and the node, for its arguments and span.
#[derive(Clone, Debug)]
pub struct AttrForm {
    pub attr: Decorator,
    pub node: Option<ast::Decorator>,
}

#[derive(Clone, Debug)]
pub enum BodyForm {
    Written(ast::FnBody),
    /// An impl's `let count = e`: the value its getter gives (R04.2.26).
    Value(ast::Expr),
    /// A made fn's body, built as IR by lower from its variant.
    Synth(SynthBody),
    /// A trait's member with no default.
    None,
}

#[derive(Clone, Debug)]
pub struct ParamForm {
    pub name: Sym,
    pub label: Sym,
    pub ty: Option<TyForm>,
    pub is_self: bool,
    pub mutable: bool,
    pub named_only: bool,
    pub variadic: bool,
    pub default: Option<ast::Expr>,
    pub span: Span,
    /// Where its name is written, for what is said of the parameter.
    pub name_span: Span,
}

#[derive(Clone, Debug)]
pub struct FnForm {
    pub name: Sym,
    pub public: bool,
    pub attrs: Vec<AttrForm>,
    pub generics: Vec<Sym>,
    pub bounds: Vec<BoundForm>,
    pub params: Vec<ParamForm>,
    pub ret: Option<TyForm>,
    pub yields: bool,
    /// `-> lends T` / `-> lends mut T`, with the parameters it borrows from.
    pub lends: Option<(bool, Vec<(Sym, Span)>)>,
    pub uses: Vec<Sym>,
    pub body: BodyForm,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct FieldForm {
    pub name: Sym,
    pub ty: TyForm,
    pub public: bool,
    pub var: bool,
    pub attrs: Vec<AttrForm>,
    pub default: Option<ast::Expr>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct StructForm {
    pub name: Sym,
    pub public: bool,
    pub attrs: Vec<AttrForm>,
    /// A record's struct or row trait, which the compiler declares; no source writes it.
    pub record: bool,
    pub generics: Vec<Sym>,
    pub generic_defaults: Vec<Option<TyForm>>,
    pub bounds: Vec<BoundForm>,
    pub fields: Vec<FieldForm>,
    pub methods: Vec<FnForm>,
    pub lets: Vec<LetForm>,
    pub consts: Vec<(Sym, Option<TyForm>, ast::Expr)>,
    pub span: Span,
}

/// A variant: its payload's fields and the `@tag` it reads and prints as (R02.7.2).
#[derive(Clone, Debug)]
pub struct VariantForm {
    pub name: Sym,
    pub name_span: Span,
    pub attrs: Vec<AttrForm>,
    pub fields: Vec<FieldForm>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct EnumForm {
    pub name: Sym,
    pub public: bool,
    pub attrs: Vec<AttrForm>,
    pub generics: Vec<Sym>,
    pub variants: Vec<VariantForm>,
    pub methods: Vec<FnForm>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct AssocForm {
    pub name: Sym,
    pub bounds: Vec<TyForm>,
    pub default: Option<TyForm>,
}

#[derive(Clone, Debug)]
pub struct TraitForm {
    pub name: Sym,
    pub public: bool,
    pub attrs: Vec<AttrForm>,
    /// A record's struct or row trait, which the compiler declares; no source writes it.
    pub record: bool,
    pub alias: bool,
    pub generics: Vec<Sym>,
    pub generic_defaults: Vec<Option<TyForm>>,
    pub supers: Vec<TyForm>,
    pub assoc: Vec<AssocForm>,
    pub methods: Vec<FnForm>,
    pub span: Span,
}

/// A struct's or impl's `let n = e` member: an impl's is the value its getter `n` gives
/// (R04.2.26).
#[derive(Clone, Debug)]
pub struct LetForm {
    pub name: Sym,
    pub span: Span,
    pub value: Option<ast::Expr>,
}

#[derive(Clone, Debug)]
pub struct ImplForm {
    pub attrs: Vec<AttrForm>,
    pub generics: Vec<Sym>,
    pub bounds: Vec<BoundForm>,
    pub trait_: Option<TyForm>,
    pub self_ty: TyForm,
    pub assoc: Vec<(Sym, TyForm)>,
    pub methods: Vec<FnForm>,
    pub lets: Vec<LetForm>,
    /// Made by a derive: replaced by a written impl of its trait for its type (R08.1.6).
    pub derived: bool,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct AliasForm {
    pub name: Sym,
    pub public: bool,
    pub attrs: Vec<AttrForm>,
    pub generics: Vec<Sym>,
    pub ty: TyForm,
    pub span: Span,
}

/// `use std.rand.Rng`, `use m.{a, b}`, `use m.*`, `use m as n` (R11.2.1).
#[derive(Clone, Debug)]
pub struct UseForm {
    pub public: bool,
    pub path: Vec<String>,
    pub names: Vec<String>,
    pub glob: bool,
    pub alias: Option<String>,
}

impl UseForm {
    pub fn read(public: bool, u: &ast::UseDecl) -> UseForm {
        UseForm {
            public,
            path: u.path().map(|t| t.text().to_string()).collect(),
            names: (u.names().iter())
                .flat_map(|n| n.names().map(|t| t.text().to_string()).collect::<Vec<_>>())
                .collect(),
            glob: u.is_glob(),
            alias: u.alias().map(|t| t.text().to_string()),
        }
    }
}

/// What one item declares, as its form.
#[derive(Clone, Debug)]
pub enum Form {
    Fn(FnForm),
    Struct(StructForm),
    Enum(EnumForm),
    Trait(TraitForm),
    Impl(ImplForm),
    Alias(AliasForm),
    Const {
        name: Sym,
        public: bool,
        ty: Option<TyForm>,
        value: Option<ast::Expr>,
    },
    Context {
        name: Sym,
        fields: Vec<FieldForm>,
    },
    Domain(ast::DomainDecl),
    Use(UseForm),
    /// A top-level statement: a `let`, an expression, an assignment.
    Stmt(ast::Stmt),
    /// A declaration decls reads no form of (an `extern`, a `mod`), or one the parser could
    /// not read.
    Other(ast::Item),
}

/// The id an intrinsic body names: `intrinsic("dom_count")` names `dom_count` (R12.14.1).
pub fn intrinsic_name(e: &ast::IntrinsicExpr) -> Option<String> {
    use cpl_next_syntax::literal::{string_parts, StrPart};
    let tok = e.id()?;
    let text = tok.text();
    Some(
        (string_parts(text).into_iter())
            .filter_map(|p| match p {
                StrPart::Text(s) => Some(&text[s.start as usize..s.end as usize]),
                StrPart::Expr(_) => None,
            })
            .collect(),
    )
}

/// Reads a written item into its form: each name interned once, here, at the token.
pub struct Reader<'a> {
    pub syms: &'a mut crate::sym::Syms,
}

impl Reader<'_> {
    fn sym(&mut self, t: Option<cpl_next_syntax::tree::SyntaxToken>) -> Sym {
        self.syms.intern(t.as_ref().map_or("", |t| t.text()))
    }

    fn ty(&mut self, t: Option<ast::Type>) -> Option<TyForm> {
        t.map(TyForm::Written)
    }

    pub fn generics(
        &mut self,
        g: Option<ast::GenericParams>,
    ) -> (Vec<Sym>, Vec<Option<TyForm>>, Vec<BoundForm>) {
        let (mut names, mut defaults, mut bounds) = (Vec::new(), Vec::new(), Vec::new());
        for p in g.iter().flat_map(|g| g.params().collect::<Vec<_>>()) {
            let name = self.sym(p.name());
            for t in p.bound().iter().flat_map(|b| b.types().collect::<Vec<_>>()) {
                bounds.push(BoundForm {
                    param: name,
                    tr: TyForm::Written(t),
                });
            }
            names.push(name);
            defaults.push(self.ty(p.default()));
        }
        (names, defaults, bounds)
    }

    pub fn where_bounds(&mut self, w: Option<ast::WhereClause>, out: &mut Vec<BoundForm>) {
        for p in w.iter().flat_map(|w| w.preds().collect::<Vec<_>>()) {
            let Some(ast::Type::Path(t)) = p.ty() else {
                continue;
            };
            let param = self.syms.intern(&crate::cycles::path_text(&t));
            for tr in p.bound().iter().flat_map(|b| b.types().collect::<Vec<_>>()) {
                out.push(BoundForm {
                    param,
                    tr: TyForm::Written(tr),
                });
            }
        }
    }

    pub fn param(&mut self, p: &ast::Param) -> ParamForm {
        let name = self.sym(p.name());
        let label = match p.label() {
            Some(l) => self.sym(Some(l)),
            None => name,
        };
        ParamForm {
            name,
            label,
            ty: self.ty(p.ty()),
            is_self: p.is_self(),
            mutable: p.is_mut(),
            named_only: p.is_named_only(),
            variadic: p.is_variadic(),
            default: p.default(),
            span: p.syntax().span(),
            name_span: p.name().map_or(p.syntax().span(), |n| n.span()),
        }
    }

    pub fn func(&mut self, f: &ast::FnDecl, attrs: Vec<AttrForm>) -> FnForm {
        let (generics, _, mut bounds) = self.generics(f.generics());
        self.where_bounds(f.where_clause(), &mut bounds);
        let params = (f
            .params()
            .iter()
            .flat_map(|l| l.params().collect::<Vec<_>>()))
        .map(|p| self.param(&p))
        .collect();
        let ret = f.ret();
        let lends = ret.as_ref().filter(|r| r.lends()).map(|r| {
            let srcs = r
                .lend_sources()
                .iter()
                .flat_map(|s| s.names().collect::<Vec<_>>())
                .collect::<Vec<_>>();
            (
                r.lends_mut(),
                srcs.into_iter()
                    .map(|t| (self.sym(Some(t.clone())), t.span()))
                    .collect(),
            )
        });
        let uses = (f
            .uses()
            .iter()
            .flat_map(|u| u.contexts().collect::<Vec<_>>()))
        .map(|t| self.sym(Some(t)))
        .collect();
        FnForm {
            name: self.sym(f.name()),
            public: f.is_pub(),
            attrs,
            generics,
            bounds,
            params,
            yields: ret.as_ref().is_some_and(|r| r.is_yields()),
            ret: self.ty(ret.and_then(|r| r.ty())),
            lends,
            uses,
            body: f.body().map_or(BodyForm::None, BodyForm::Written),
            span: f.signature_span(),
        }
    }

    /// Each decorator on a node as the attribute it is; the unknown and removed ones come back
    /// apart, for the item to report at their nodes.
    pub fn attrs(
        &mut self,
        ds: impl Iterator<Item = ast::Decorator>,
    ) -> (Vec<AttrForm>, Vec<ast::Decorator>) {
        let (mut known, mut other) = (Vec::new(), Vec::new());
        for x in ds {
            match x.name().and_then(|n| Decorator::live(n.text())) {
                Some(attr) => known.push(AttrForm {
                    attr,
                    node: Some(x),
                }),
                None => other.push(x),
            }
        }
        (known, other)
    }

    fn bound_types(&mut self, b: Option<ast::Bound>) -> Vec<TyForm> {
        b.iter()
            .flat_map(|b| b.types().collect::<Vec<_>>())
            .map(TyForm::Written)
            .collect()
    }

    pub fn field(&mut self, f: &ast::FieldDecl, attrs: Vec<AttrForm>) -> Option<FieldForm> {
        Some(FieldForm {
            name: self.sym(f.name()),
            ty: TyForm::Written(f.ty()?),
            public: f.is_pub(),
            var: f.is_var(),
            attrs,
            default: f.default(),
            span: f.syntax().span(),
        })
    }

    pub fn alias(&mut self, a: &ast::TypeAlias, attrs: Vec<AttrForm>) -> Option<AliasForm> {
        let (generics, ..) = self.generics(a.generics());
        Some(AliasForm {
            name: self.sym(a.name()),
            public: a.is_pub(),
            attrs,
            generics,
            ty: TyForm::Written(a.ty()?),
            span: a.syntax().span(),
        })
    }

    pub fn trait_(&mut self, t: &ast::TraitDecl, attrs: Vec<AttrForm>) -> TraitForm {
        let (generics, generic_defaults, _) = self.generics(t.generics());
        let members = t.members();
        let assoc = (members
            .iter()
            .flat_map(|m| m.assoc_types().collect::<Vec<_>>()))
        .map(|a| AssocForm {
            name: self.sym(a.name()),
            bounds: self.bound_types(a.bound()),
            default: a.ty().map(TyForm::Written),
        })
        .collect();
        let methods = (members.iter().flat_map(|m| m.fns().collect::<Vec<_>>()))
            .map(|f| {
                let (attrs, _) = self.attrs(f.decorators());
                self.func(&f, attrs)
            })
            .collect();
        // `trait HasW: w: Int` is the member it means, `@get fn w(self) -> Int` (R08.1.7).
        let mut methods: Vec<FnForm> = methods;
        for f in t.fields() {
            if let Some(ty) = f.ty() {
                let name = self.sym(f.name());
                let span = f.syntax().span();
                methods.push(crate::synth::field_requirement(
                    name,
                    TyForm::Written(ty),
                    span,
                    self.syms,
                ));
            }
        }
        let alias = t.alias().is_some();
        let supers = match t.alias() {
            Some(b) => self.bound_types(Some(b)),
            None => self.bound_types(t.supertraits()),
        };
        TraitForm {
            name: self.sym(t.name()),
            public: t.is_pub(),
            attrs,
            record: false,
            alias,
            generics,
            generic_defaults,
            supers,
            assoc,
            methods,
            span: t.syntax().span(),
        }
    }

    /// Each `let n = e` member of a struct or impl.
    fn lets(&mut self, members: &Option<ast::Members>) -> Vec<LetForm> {
        (members.iter().flat_map(|m| m.lets().collect::<Vec<_>>()))
            .filter_map(|l| {
                let ps: Vec<ast::Pattern> = l.patterns().collect();
                let [ast::Pattern::Path(p)] = &ps[..] else {
                    return None;
                };
                let segs: Vec<_> = p.segments().collect();
                let [n] = &segs[..] else {
                    return None;
                };
                Some(LetForm {
                    name: self.syms.intern(n.text()),
                    span: p.syntax().span(),
                    value: l.value(),
                })
            })
            .collect()
    }

    pub fn struct_(
        &mut self,
        sd: &ast::StructDecl,
        attrs: Vec<AttrForm>,
    ) -> (StructForm, Vec<ast::Decorator>) {
        let (generics, generic_defaults, bounds) = self.generics(sd.generics());
        let mut unknown = Vec::new();
        let fields = (sd.fields().iter())
            .filter_map(|f| {
                let (attrs, other) = self.attrs(f.decorators());
                unknown.extend(other);
                self.field(f, attrs)
            })
            .collect();
        let members = sd.members();
        let methods = (members.iter().flat_map(|m| m.fns().collect::<Vec<_>>()))
            .map(|f| {
                let (attrs, other) = self.attrs(f.decorators());
                unknown.extend(other);
                self.func(&f, attrs)
            })
            .collect();
        let consts = (members.iter().flat_map(|m| m.consts().collect::<Vec<_>>()))
            .filter_map(|c| Some((self.sym(c.name()), c.ty().map(TyForm::Written), c.value()?)))
            .collect();
        let form = StructForm {
            name: self.sym(sd.name()),
            public: sd.is_pub(),
            attrs,
            record: false,
            generics,
            generic_defaults,
            bounds,
            fields,
            methods,
            consts,
            lets: self.lets(&members),
            span: sd.syntax().span(),
        };
        (form, unknown)
    }

    pub fn impl_(
        &mut self,
        im: &ast::ImplDecl,
        attrs: Vec<AttrForm>,
    ) -> Option<(ImplForm, Vec<ast::Decorator>)> {
        let (generics, _, mut bounds) = self.generics(im.generics());
        self.where_bounds(im.where_clause(), &mut bounds);
        let trait_ = self.bound_types(im.trait_ref()).into_iter().next();
        let members = im.members();
        let assoc = (members
            .iter()
            .flat_map(|m| m.assoc_types().collect::<Vec<_>>()))
        .filter_map(|a| Some((self.sym(a.name()), TyForm::Written(a.ty()?))))
        .collect();
        let mut unknown = Vec::new();
        let methods = (members.iter().flat_map(|m| m.fns().collect::<Vec<_>>()))
            .map(|f| {
                let (attrs, other) = self.attrs(f.decorators());
                unknown.extend(other);
                self.func(&f, attrs)
            })
            .collect();
        let form = ImplForm {
            attrs,
            generics,
            bounds,
            trait_,
            self_ty: TyForm::Written(im.self_ty()?),
            assoc,
            methods,
            lets: self.lets(&members),
            derived: false,
            span: im.syntax().span(),
        };
        Some((form, unknown))
    }

    pub fn enum_(
        &mut self,
        e: &ast::EnumDecl,
        attrs: Vec<AttrForm>,
    ) -> (EnumForm, Vec<ast::Decorator>) {
        let (generics, ..) = self.generics(e.generics());
        let mut unknown = Vec::new();
        let variants = (e.variants().iter())
            .map(|v| {
                let (attrs, other) = self.attrs(v.decorators());
                unknown.extend(other);
                // A payload's unnamed fields are named by position (R02.7).
                let fields = (v.fields().collect::<Vec<_>>().iter().enumerate())
                    .filter_map(|(i, f)| {
                        let (attrs, other) = self.attrs(f.decorators());
                        unknown.extend(other);
                        let mut field = self.field(f, attrs)?;
                        if f.name().is_none() {
                            field.name = self.syms.intern(&i.to_string());
                        }
                        Some(field)
                    })
                    .collect();
                VariantForm {
                    name_span: v.name().map_or(v.syntax().span(), |n| n.span()),
                    name: self.sym(v.name()),
                    attrs,
                    fields,
                    span: v.syntax().span(),
                }
            })
            .collect();
        let members = e.members();
        let methods = (members.iter().flat_map(|m| m.fns().collect::<Vec<_>>()))
            .map(|f| {
                let (attrs, other) = self.attrs(f.decorators());
                unknown.extend(other);
                self.func(&f, attrs)
            })
            .collect();
        let form = EnumForm {
            name: self.sym(e.name()),
            public: e.is_pub(),
            attrs,
            generics,
            variants,
            methods,
            span: e.syntax().span(),
        };
        (form, unknown)
    }

    /// The item's form, and its decorators no attribute names, for the item to report.
    pub fn item(&mut self, it: &ast::Item) -> (Form, Vec<ast::Decorator>) {
        use ast::Decl as D;
        let (attrs, mut unknown) = self.attrs(it.decorators().into_iter());
        let form = match it.decl() {
            Some(D::Fn(f)) => Form::Fn(self.func(&f, attrs)),
            Some(D::Struct(sd)) => {
                let (f, u) = self.struct_(&sd, attrs);
                unknown.extend(u);
                Form::Struct(f)
            }
            Some(D::Enum(e)) => {
                let (f, u) = self.enum_(&e, attrs);
                unknown.extend(u);
                Form::Enum(f)
            }
            Some(D::Trait(t)) => Form::Trait(self.trait_(&t, attrs)),
            Some(D::Impl(im)) => match self.impl_(&im, attrs) {
                Some((f, u)) => {
                    unknown.extend(u);
                    Form::Impl(f)
                }
                None => Form::Other(it.clone()),
            },
            Some(D::TypeAlias(a)) => match self.alias(&a, attrs) {
                Some(f) => Form::Alias(f),
                None => Form::Other(it.clone()),
            },
            Some(D::Const(c)) => Form::Const {
                name: self.sym(c.name()),
                public: c.is_pub(),
                ty: c.ty().map(TyForm::Written),
                value: c.value(),
            },
            Some(D::Context(c)) => Form::Context {
                name: self.sym(c.name()),
                fields: (c.fields().iter())
                    .filter_map(|f| self.field(f, Vec::new()))
                    .collect(),
            },
            Some(D::Domain(dd)) => Form::Domain(dd),
            Some(D::Use(u)) => Form::Use(UseForm::read(it.is_pub(), &u)),
            Some(D::Mod(_) | D::Extern(_)) => Form::Other(it.clone()),
            None => match it.stmt() {
                Some(s) => Form::Stmt(s),
                None => Form::Other(it.clone()),
            },
        };
        (form, unknown)
    }
}

impl Form {
    /// The name it declares in its module; `None` for an impl, a `use` and the like.
    pub fn name(&self) -> Option<Sym> {
        match self {
            Form::Fn(f) => Some(f.name),
            Form::Struct(s) => Some(s.name),
            Form::Enum(e) => Some(e.name),
            Form::Trait(t) => Some(t.name),
            Form::Alias(a) => Some(a.name),
            Form::Const { name, .. } | Form::Context { name, .. } => Some(*name),
            Form::Impl(_) | Form::Domain(_) | Form::Use(_) | Form::Stmt(_) | Form::Other(_) => None,
        }
    }

    /// What kind of declaration it is, as a diagnostic says it.
    pub fn kind(&self) -> &'static str {
        match self {
            Form::Fn(_) => "fn",
            Form::Struct(_) => "struct",
            Form::Enum(_) => "enum",
            Form::Trait(_) => "trait",
            Form::Impl(_) => "impl",
            Form::Alias(_) => "type alias",
            Form::Const { .. } => "const",
            Form::Context { .. } => "context",
            Form::Domain(_) => "domain",
            Form::Use(_) => "use",
            Form::Stmt(_) => "statement",
            Form::Other(_) => "item",
        }
    }
}
