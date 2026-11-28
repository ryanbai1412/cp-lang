//! Declarations the compiler writes for a program: an enum's structs and the `Eq` and `Ord` a
//! struct derives from its fields.
use crate::decls::unnamed;
use crate::decls::{fills, Role};
use crate::diag::Msg;
use crate::records::marker;
use cpl_next_ir::ids::ItemId;
use cpl_next_ir::unbuilt::Unbuilt;
use cpl_syntax::ast::{self, FnBody, StmtKind, TypeKind};
use cpl_syntax::decorators::Mark;
use std::collections::HashMap;

/// An enum's tag errors (R02.7.2): a variant whose token another one before it has (E0407), and
/// an enum-level `@tags(..)`, which is removed: each variant writes its own `@tag` (decision
/// 459). With one plain tag per variant, the help is the rewrite.
pub(crate) fn enum_tags(item: &ItemId, s: &ast::Stmt, e: &ast::EnumDecl) -> Vec<Msg> {
    let mut out: Vec<Msg> = (e.variants.iter().enumerate())
        .filter_map(|(i, v)| {
            let t = token(v);
            let first = e.variants[..i].iter().find(|w| token(w) == t)?;
            let what = format!(
                "`{}` reads and prints as `{t}`, as `{}` does: tags are distinct within an enum",
                v.name.text, first.name.text
            );
            let m = Msg::verdict(cpl_diag::codes::DUPLICATE_VARIANT_TAG, what);
            Some(Msg::at(item.at(v.name.span), m))
        })
        .collect();
    let level = || -> Option<Msg> {
        let x = s.decorators.iter().find(|x| x.mark == Some(Mark::Tags))?;
        let text = cpl_diag::msgs::check::ENUM_LEVEL_TAGS_WAS.say(&[]);
        let mut m = Msg::at(
            item.at(x.span),
            Msg::verdict(cpl_diag::codes::REMOVED_SYNTAX, text),
        );
        let tag = |a: &ast::Arg| match &a.value.kind {
            ast::ExprKind::Str(p) => match p.as_slice() {
                [ast::StrPiece::Text(t)] => Some(format!("{t:?}")),
                _ => None,
            },
            _ => None,
        };
        let each = (x.args.len() == e.variants.len())
            .then(|| x.args.iter().zip(&e.variants))
            .into_iter()
            .flatten()
            .map(|(a, v)| Some(format!("@tag({}) {}", tag(a)?, v.name.text)))
            .collect::<Option<Vec<_>>>()
            .filter(|w| !w.is_empty());
        m.help = each.map(|w| cpl_diag::habits::ENUM_TAGS.say(&[&e.name.text, &w.join(" | ")]));
        Some(m)
    };
    out.extend(level());
    out
}

/// An enum as structs: its own, with no fields (the parser makes its methods an `impl`), then one
/// per variant holding that variant's payload, named `Enum.Variant`, which no source name can be
/// (R02.7).
pub(crate) fn enum_structs(
    s: &ast::Stmt,
    e: &ast::EnumDecl,
) -> (Vec<ast::Stmt>, Vec<Msg>, Vec<i64>) {
    let mut errors = Vec::new();
    let mut values = Vec::new();
    let one = |name: String, fields, methods, decorators| ast::Stmt {
        kind: StmtKind::Struct(ast::StructDecl {
            name: ast::Name {
                text: name,
                span: e.name.span,
            },
            generics: e.generics.clone(),
            generic_defaults: vec![None; e.generics.len()],
            bounds: Vec::new(),
            fields,
            methods,
        }),
        span: s.span,
        decorators,
        public: s.public,
    };
    let mut out = vec![one(
        e.name.text.clone(),
        Vec::new(),
        Vec::new(),
        s.decorators.clone(),
    )];
    for (i, v) in e.variants.iter().enumerate() {
        let value = match &v.value {
            None => Some(i as i64),
            Some(_) if !v.fields.is_empty() => None,
            Some(x) => x.int_literal(),
        };
        if value.is_none() {
            errors.push(
                Unbuilt::VariantValue {
                    name: v.name.text.clone(),
                }
                .into(),
            );
        }
        values.push(value.unwrap_or(i as i64));
        let fields = (v.fields.iter().enumerate())
            .map(|(i, (n, t))| ast::FieldDecl {
                name: n.clone().unwrap_or_else(|| ast::Name {
                    text: i.to_string(),
                    span: t.span,
                }),
                ty: t.clone(),
                default: None,
                mutable: false,
                cached: false,
                bits: false,
                public: true,
            })
            .collect();
        out.push(one(
            format!("{}.{}", e.name.text, v.name.text),
            fields,
            Vec::new(),
            Vec::new(),
        ));
    }
    (out, errors, values)
}

enum Derive<'a> {
    Cmp(ast::CmpOp, &'a str),
    /// A core trait marked `@derive`: its one method builds a struct from that method on each field.
    Each(&'a ast::FnDecl),
    /// The `@role(derive_fields)` trait: each method is that method on the `@role(pack)` of the
    /// fields.
    Fields(&'a str, &'a [ast::FnDecl]),
    /// A `@derive` method making a `Self`, for an enum: the variant its first parameter's
    /// `variant([tags])` picks, each field read by that parameter's `read<F>()` (R10.2.1).
    Tags(&'a ast::FnDecl, &'a [ast::Variant]),
    /// The same for an enum: each method is that method on the variant's name (R10.3.5).
    Names(&'a [ast::FnDecl], &'a [ast::Variant]),
}

/// A core trait every struct gets from its fields, and how.
pub(crate) enum Derivable {
    /// `@derive`, with its one method.
    Each(Box<ast::FnDecl>),
    /// `@role(derive_fields)`, with the `@role(pack)` struct and the trait's methods.
    Fields(String, Vec<ast::FnDecl>),
}

/// Per struct a module declares, the impls it gets structurally (R08.1.6), made after every written
/// impl and only where none is: `Eq` field by field, `Ord` lexicographic in declaration order, and
/// `Clone` cloning each field (R05.2.2).
pub(crate) fn module_derived(
    items: &[crate::modules::Item],
    each: &[(String, Derivable)],
) -> Vec<(ItemId, String, ast::Stmt, bool)> {
    let mut out = Vec::new();
    for crate::modules::Item { id: item, stmts } in items {
        for s in stmts {
            if let StmtKind::Enum(e) = &s.kind {
                let sd = ast::StructDecl {
                    name: e.name.clone(),
                    generics: e.generics.clone(),
                    generic_defaults: vec![None; e.generics.len()],
                    bounds: Vec::new(),
                    fields: Vec::new(),
                    methods: Vec::new(),
                };
                for (tr, how) in each {
                    let derive = match how {
                        Derivable::Fields(_, fns) => Derive::Names(fns, &e.variants),
                        Derivable::Each(m) if made(m) => Derive::Tags(m, &e.variants),
                        Derivable::Each(_) => continue,
                    };
                    let caches = copies_caches(&derive);
                    let derived = derived_impl(s, &sd, derive, tr);
                    out.push((item.clone(), sd.name.text.clone(), derived, caches));
                }
            }
            let StmtKind::Struct(sd) = &s.kind else {
                continue;
            };
            // The traits `==` and `<` call, and `.clone()`: the derivable ones.
            for op in [ast::CmpOp::Eq, ast::CmpOp::Lt] {
                let Some((tr, method)) = op.trait_method() else {
                    continue;
                };
                let derived = derived_impl(s, sd, Derive::Cmp(op, method), tr);
                out.push((item.clone(), sd.name.text.clone(), derived, false));
            }
            for (tr, how) in each {
                let derive = match how {
                    // A made value needs every field, and a derive gives no `@cache` field
                    // (R05.10.13): a struct with one makes itself by hand.
                    Derivable::Each(m) if made(m) && sd.fields.iter().any(|f| f.cached) => {
                        continue;
                    }
                    Derivable::Each(m) => Derive::Each(m),
                    Derivable::Fields(pack, fns) => Derive::Fields(pack, fns),
                };
                let caches = copies_caches(&derive);
                let derived = derived_impl(s, sd, derive, tr);
                out.push((item.clone(), sd.name.text.clone(), derived, caches));
            }
        }
    }
    out
}

/// The token a variant reads and prints as: its `@tag`'s literal, else its name (R02.7.2).
fn token(v: &ast::Variant) -> String {
    use ast::ExprKind;
    match v.tag.as_ref().map(|t| &t.kind) {
        Some(ExprKind::Str(pieces)) => (pieces.iter())
            .map(|p| match p {
                ast::StrPiece::Text(t) => t.clone(),
                _ => String::new(),
            })
            .collect(),
        Some(ExprKind::Char(c)) => c.to_string(),
        Some(_) => (v.tag.as_ref().and_then(|t| t.int_literal()))
            .map_or_else(|| v.name.text.clone(), |k| k.to_string()),
        None => v.name.text.clone(),
    }
}

/// A `@derive` method making a `Self` from its parameters, not from a `self`: its first one has a
/// written type, as `Read`'s `read(mut input: Input)`, where a trait's receiver has none.
/// Whether the derived impl reads the `@cache` fields: a `@cache` field is not part of the value
/// a derive compares or shows (R05.10.13); a clone copies it too.
fn copies_caches(derive: &Derive) -> bool {
    matches!(derive, Derive::Each(m) if !made(m))
}

fn made(m: &ast::FnDecl) -> bool {
    m.params.first().is_some_and(|p| p.ty.is_some())
}

/// `impl<T: Eq> Eq for S<T>: fn eq(a, b) = a.x == b.x and ..`, `Ord`'s `lt` over the fields in
/// order, `fn clone(a) = S(x: a.x.clone(), ..)`, or `fn show(a) = Pack(a.x, Pack(a.y, ())).show()`,
/// written at the struct. Its bounds are what its fields' impls ask, which only decls knows.
fn derived_impl(s: &ast::Stmt, sd: &ast::StructDecl, derive: Derive, tr: &str) -> ast::Stmt {
    use ast::{BinOp, CmpOp, ExprKind};
    let span = s.span;
    let name = |text: &str| ast::Name {
        text: text.to_string(),
        span,
    };
    let ty = |text: &str, args| ast::Type {
        kind: TypeKind::Named {
            name: name(text),
            args,
        },
        span,
    };
    let ex = |kind| ast::Expr { kind, span };
    let field_of = |base: ast::Expr, f: &str| {
        ex(ExprKind::Field {
            base: Box::new(base),
            name: name(f),
        })
    };
    let field = |side: &str, f: &str| field_of(ex(ExprKind::Name(side.to_string())), f);
    let cmp = |op, f: &str| {
        ex(ExprKind::Compare {
            first: Box::new(field("a", f)),
            rest: vec![(op, field("b", f))],
        })
    };
    let bin = |op, lhs, rhs| {
        ex(ExprKind::Binary {
            op,
            op_span: span,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        })
    };
    let clone = copies_caches(&derive);
    let compared: Vec<&ast::FieldDecl> = sd.fields.iter().filter(|f| clone || !f.cached).collect();
    let fields: Vec<&str> = compared.iter().map(|f| f.name.text.as_str()).collect();
    let stmt = |kind| ast::Stmt {
        kind,
        span,
        decorators: Vec::new(),
        public: false,
    };
    let call = |callee, args| {
        ex(ExprKind::Call {
            callee: Box::new(callee),
            args,
        })
    };
    // `input.read<F>()`: a field of type `F` read by the method's first parameter.
    let made_from = |m: &ast::FnDecl, t: &ast::Type| {
        call(
            ex(ExprKind::Generic {
                base: Box::new(field(&m.params[0].name.text, &m.name.text)),
                args: vec![t.clone()],
            }),
            Vec::new(),
        )
    };
    let body = match derive {
        Derive::Tags(m, variants) => {
            let tags = (variants.iter())
                .map(|v| ex(ExprKind::Str(vec![ast::StrPiece::Text(token(v))])))
                .collect();
            let picked = call(
                field(&m.params[0].name.text, "variant"),
                vec![ast::Arg {
                    name: None,
                    mutable: false,
                    value: ex(ExprKind::List(tags)),
                }],
            );
            let last = variants.len().saturating_sub(1);
            ex(ExprKind::Match {
                scrutinee: Box::new(picked),
                arms: (variants.iter().enumerate())
                    .map(|(i, v)| {
                        let made = field(&sd.name.text, &v.name.text);
                        let made = match v.fields.is_empty() {
                            true => made,
                            false => call(
                                made,
                                (v.fields.iter())
                                    .map(|(_, t)| ast::Arg {
                                        name: None,
                                        mutable: false,
                                        value: made_from(m, t),
                                    })
                                    .collect(),
                            ),
                        };
                        let pat = match i == last {
                            true => ast::PatternKind::Wildcard,
                            false => {
                                ast::PatternKind::Literal(Box::new(ex(ExprKind::Int(i as i128))))
                            }
                        };
                        ast::Arm {
                            pat: ast::Pattern { kind: pat, span },
                            guard: None,
                            body: ast::Block {
                                stmts: vec![stmt(StmtKind::Expr(made))],
                                span,
                            },
                        }
                    })
                    .collect(),
            })
        }
        Derive::Names(_, variants) => ex(ExprKind::Match {
            scrutinee: Box::new(ex(ExprKind::Name("a".to_string()))),
            arms: (variants.iter())
                .map(|v| ast::Arm {
                    pat: ast::Pattern {
                        kind: ast::PatternKind::Variant {
                            path: vec![sd.name.clone(), v.name.clone()],
                            args: (v.fields.iter())
                                .map(|_| ast::Pattern {
                                    kind: ast::PatternKind::Wildcard,
                                    span,
                                })
                                .collect(),
                        },
                        span,
                    },
                    guard: None,
                    body: ast::Block {
                        stmts: vec![stmt(StmtKind::Expr(ex(ExprKind::Str(vec![
                            ast::StrPiece::Text(token(v)),
                        ]))))],
                        span,
                    },
                })
                .collect(),
        }),
        Derive::Fields(pack, _) => {
            (fields.iter().rev()).fold(ex(ExprKind::Tuple(Vec::new())), |rest, f| {
                let arg = |value| ast::Arg {
                    name: None,
                    mutable: false,
                    value,
                };
                call(
                    ex(ExprKind::Name(pack.to_string())),
                    vec![arg(field("a", f)), arg(rest)],
                )
            })
        }
        Derive::Each(m) => {
            // `a.f.clone()`; or, where the method makes a `Self` from its first parameter, as
            // `Read`'s does from its `Input`, that parameter's own `read<F>()` at each field's type.
            let value = |f: &ast::FieldDecl| match made(m) {
                true => made_from(m, &f.ty),
                false => call(field_of(field("a", &f.name.text), &m.name.text), Vec::new()),
            };
            let args = (compared.iter())
                .map(|f| ast::Arg {
                    name: Some(f.name.clone()),
                    mutable: false,
                    value: value(f),
                })
                .collect();
            // Named with its parameters, so a struct overloaded by arity makes its own (R02.11.4).
            let base = Box::new(ex(ExprKind::Name(sd.name.text.clone())));
            let callee = match sd.generics.is_empty() {
                true => base,
                false => Box::new(ex(ExprKind::Generic {
                    base,
                    args: sd
                        .generics
                        .iter()
                        .map(|g| ty(&g.text, Vec::new()))
                        .collect(),
                })),
            };
            ex(ExprKind::Call { callee, args })
        }
        Derive::Cmp(CmpOp::Eq, _) => (fields.iter().map(|f| cmp(CmpOp::Eq, f)))
            .reduce(|a, b| bin(BinOp::And, a, b))
            .unwrap_or_else(|| ex(ExprKind::Bool(true))),
        Derive::Cmp(..) => (fields.iter().rev())
            .fold(None, |rest, f| {
                Some(match rest {
                    None => cmp(CmpOp::Lt, f),
                    Some(r) => bin(
                        BinOp::Or,
                        cmp(CmpOp::Lt, f),
                        bin(BinOp::And, cmp(CmpOp::Eq, f), r),
                    ),
                })
            })
            .unwrap_or_else(|| ex(ExprKind::Bool(false))),
    };
    let param = |n: &str| ast::Param {
        mutable: false,
        variadic: false,
        lazy: false,
        named_only: false,
        label: None,
        name: name(n),
        ty: None,
        default: None,
        span,
    };
    let f = |method: &str, params, body| ast::FnDecl {
        name: name(method),
        generics: Vec::new(),
        bounds: Vec::new(),
        params,
        ret: None,
        lends: None,
        uses: Vec::new(),
        body: FnBody::Expr(body),
        sig_span: span,
    };
    let fns = match derive {
        Derive::Each(m) | Derive::Tags(m, _) if made(m) => {
            let params = (m.params.iter()).map(|p| ast::Param {
                ty: None,
                default: None,
                span,
                ..p.clone()
            });
            vec![f(&m.name.text, params.collect(), body)]
        }
        Derive::Each(m) | Derive::Tags(m, _) => vec![f(&m.name.text, vec![param("a")], body)],
        Derive::Cmp(_, m) => vec![f(m, vec![param("a"), param("b")], body)],
        // Each method passes its other parameters on as they came.
        Derive::Fields(_, methods) | Derive::Names(methods, _) => (methods.iter())
            .map(|m| {
                let rest = m.params.iter().skip(1);
                let args = (rest.clone())
                    .map(|p| ast::Arg {
                        name: None,
                        mutable: p.mutable,
                        value: ex(ExprKind::Name(p.name.text.clone())),
                    })
                    .collect();
                let callee = ex(ExprKind::Field {
                    base: Box::new(body.clone()),
                    name: name(&m.name.text),
                });
                let params = std::iter::once(param("a")).chain(rest.map(|p| ast::Param {
                    ty: None,
                    default: None,
                    span,
                    ..p.clone()
                }));
                f(&m.name.text, params.collect(), call(callee, args))
            })
            .collect(),
    };
    stmt(StmtKind::Impl {
        trait_name: Some(ty(tr, Vec::new())),
        target: ty(
            &sd.name.text,
            sd.generics
                .iter()
                .map(|g| ty(&g.text, Vec::new()))
                .collect(),
        ),
        // Every parameter is declared, so a name longer than a letter is one too.
        bounds: (sd.generics.iter())
            .map(|g| ast::Bound {
                name: g.clone(),
                traits: Vec::new(),
            })
            .collect(),
        body: ast::Block {
            stmts: fns.into_iter().map(|f| stmt(StmtKind::Fn(f))).collect(),
            span,
        },
    })
}

/// The traits marked `@derive`, with their one method, and the `@role(derive_fields)` trait,
/// with its methods and the `@role(pack)` struct: every struct whose fields all have one gets it field by
/// field (R08.1.6).
pub(crate) fn derive_each<'a, I: Iterator<Item = &'a ast::Stmt>>(
    stmts: impl Fn() -> I,
) -> Vec<(String, Derivable)> {
    let marked = |s: &ast::Stmt, m: Mark| s.mark(m).is_some();
    let pack = stmts().find_map(|s| match &s.kind {
        StmtKind::Struct(sd) if fills(s, Role::Pack) => Some(sd.name.text.clone()),
        _ => None,
    });
    stmts()
        .filter_map(|s| {
            let StmtKind::Trait { name, body, .. } = &s.kind else {
                return None;
            };
            let fns = body.stmts.iter().filter_map(|m| match &m.kind {
                StmtKind::Fn(f) => Some(f.clone()),
                _ => None,
            });
            let how = match (&body.stmts[..], &pack) {
                ([_], _) if marked(s, Mark::Derive) => {
                    Derivable::Each(Box::new(fns.clone().next()?))
                }
                (_, Some(p)) if fills(s, Role::DeriveFields) => {
                    Derivable::Fields(p.clone(), fns.collect())
                }
                _ => return None,
            };
            Some((name.text.clone(), how))
        })
        .collect()
}

/// `@get fn name(self) = self.name`: how a struct meets a bodiless `@get` member its impl leaves
/// out (R12.11.3).
pub(crate) fn field_getter(field: &str, span: cpl_diag::Span) -> ast::Stmt {
    let name = |text: &str| ast::Name {
        text: text.to_string(),
        span,
    };
    let ex = |kind| ast::Expr { kind, span };
    let f = ast::FnDecl {
        name: name(field),
        generics: Vec::new(),
        bounds: Vec::new(),
        params: vec![ast::Param {
            mutable: false,
            variadic: false,
            lazy: false,
            named_only: false,
            label: None,
            name: name("self"),
            ty: None,
            default: None,
            span,
        }],
        ret: None,
        lends: None,
        uses: Vec::new(),
        body: FnBody::Expr(ex(ast::ExprKind::Field {
            base: Box::new(ex(ast::ExprKind::Name("self".to_string()))),
            name: name(field),
        })),
        sig_span: span,
    };
    ast::Stmt {
        kind: StmtKind::Fn(f),
        span,
        decorators: vec![marker(Mark::Get, span)],
        public: false,
    }
}

/// An impl's `let count = e`: the self-less getter `@get fn count() = e` that meets its trait's
/// member of that name (R04.2.26).
pub(crate) fn impl_value(m: &ast::Stmt) -> Option<ast::Stmt> {
    let StmtKind::Let {
        pat,
        ty,
        init: Some(init),
        ..
    } = &m.kind
    else {
        return None;
    };
    let mut get = field_getter(pat.let_name()?, m.span);
    if let StmtKind::Fn(f) = &mut get.kind {
        f.params.clear();
        f.ret = ty.clone();
        f.body = FnBody::Expr(init.clone());
    }
    Some(get)
}

/// `@get fn name(self) -> T` with no body: a field requirement, met by a field `name` (R08.1.7).
pub(crate) fn field_requirement(field: &str, ty: ast::Type, span: cpl_diag::Span) -> ast::Stmt {
    let mut get = field_getter(field, span);
    if let StmtKind::Fn(f) = &mut get.kind {
        f.body = FnBody::None;
        f.ret = Some(ty);
    }
    get
}

/// `trait HasW: w: Int` as the member it means, `@get fn w(self) -> Int` (R08.1.7).
pub(crate) fn trait_fields(s: &mut ast::Stmt) {
    let StmtKind::Trait { body, .. } = &mut s.kind else {
        return;
    };
    for m in &mut body.stmts {
        if let StmtKind::Let {
            pat:
                ast::Pattern {
                    kind: ast::PatternKind::Name(name),
                    ..
                },
            ty: Some(ty),
            init: None,
            ..
        } = &m.kind
        {
            *m = field_requirement(name, ty.clone(), m.span);
        }
    }
}

/// Per record shape with a field for each member of a trait made only of bodiless `@get` members,
/// a bodiless impl of it, written with the trait: `impl<V, A> Target<V> for (to: V, w: A)`, its
/// members then met by the fields (R12.11.3). A written impl replaces it.
pub(crate) fn field_trait_impls(
    items: &[crate::modules::Item],
    shapes: &[Vec<String>],
) -> Vec<(ItemId, String, ast::Stmt)> {
    let mut out = Vec::new();
    for (item, s) in items
        .iter()
        .flat_map(|it| it.stmts.iter().map(move |s| (&it.id, s)))
    {
        let StmtKind::Trait {
            name,
            generics,
            supers,
            body,
            alias: false,
            bounds,
            ..
        } = &s.kind
        else {
            continue;
        };
        if !supers.is_empty() || !bounds.is_empty() || body.stmts.is_empty() {
            continue;
        }
        fn member(m: &ast::Stmt) -> Option<(&str, &ast::Type)> {
            match &m.kind {
                StmtKind::Fn(f)
                    if matches!(f.body, FnBody::None)
                        && f.params.len() <= 1
                        && m.mark(Mark::Get).is_some() =>
                {
                    (f.ret.as_ref())
                        .filter(|t| writable(t))
                        .map(|t| (f.name.text.as_str(), t))
                }
                _ => None,
            }
        }
        let Some(members) = body.stmts.iter().map(member).collect::<Option<Vec<_>>>() else {
            continue;
        };
        let mut taken: Vec<String> = generics.iter().map(|g| g.text.clone()).collect();
        for (_, t) in &members {
            unnamed(t, &|_| None, &mut taken);
        }
        let fresh: Vec<String> = (b'A'..=b'Z')
            .map(|c| (c as char).to_string())
            .filter(|c| !taken.contains(c))
            .collect();
        let span = s.span;
        let nm = |t: &str| ast::Name {
            text: t.to_string(),
            span,
        };
        let named = |t: &str| ast::Type {
            kind: TypeKind::Named {
                name: nm(t),
                args: Vec::new(),
            },
            span,
        };
        for fs in shapes {
            if !members.iter().all(|(m, _)| fs.iter().any(|f| f == m)) {
                continue;
            }
            let mut params: Vec<String> = generics.iter().map(|g| g.text.clone()).collect();
            let mut extra = fresh.iter();
            let mut fields = Vec::new();
            for f in fs {
                match members.iter().find(|(m, _)| m == f) {
                    Some((_, t)) => fields.push((nm(f), (*t).clone())),
                    None => {
                        let Some(g) = extra.next() else { break };
                        params.push(g.clone());
                        fields.push((nm(f), named(g)));
                    }
                }
            }
            if fields.len() != fs.len() {
                continue;
            }
            let kind = StmtKind::Impl {
                trait_name: Some(ast::Type {
                    kind: TypeKind::Named {
                        name: name.clone(),
                        args: generics.iter().map(|g| named(&g.text)).collect(),
                    },
                    span,
                }),
                target: ast::Type {
                    kind: TypeKind::Record(fields),
                    span,
                },
                bounds: (params.iter())
                    .map(|p| ast::Bound {
                        name: nm(p),
                        traits: Vec::new(),
                    })
                    .collect(),
                body: ast::Block {
                    stmts: Vec::new(),
                    span,
                },
            };
            let imp = ast::Stmt {
                kind,
                span,
                decorators: Vec::new(),
                public: false,
            };
            out.push((item.clone(), crate::records::name(fs), imp));
        }
    }
    out
}

/// A field type an impl can write as it is: no `Self`, no associated type.
fn writable(t: &ast::Type) -> bool {
    match &t.kind {
        TypeKind::Named { name, args } => {
            !name.text.starts_with("Self") && !name.text.contains('.') && args.iter().all(writable)
        }
        TypeKind::Optional(e) => writable(e),
        TypeKind::Tuple(ts) => ts.iter().all(writable),
        TypeKind::Array { index, elem } => index.iter().chain([&**elem]).all(writable),
        _ => false,
    }
}

/// The `@role(convert)` trait with its one method, and the `@role(variant_check)` fn: what a
/// payload-free enum's conversions are written with.
pub(crate) fn convert_marked<'a, I: Iterator<Item = &'a ast::Stmt>>(
    stmts: impl Fn() -> I,
) -> Option<(String, String, String)> {
    let (tr, method) =
        (stmts().filter(|s| fills(s, Role::Convert))).find_map(|s| match &s.kind {
            StmtKind::Trait { name, body, .. } => match &body.stmts[..] {
                [ast::Stmt {
                    kind: StmtKind::Fn(f),
                    ..
                }] => Some((name.text.clone(), f.name.text.clone())),
                _ => None,
            },
            _ => None,
        })?;
    let check = (stmts().filter(|s| fills(s, Role::VariantCheck))).find_map(|s| match &s.kind {
        StmtKind::Fn(f) => Some(f.name.text.clone()),
        _ => None,
    })?;
    Some((tr, method, check))
}

/// Per payload-free enum a module declares, `Int(e)` and `E(x)` (R02.7): `impl Into<Int> for E`,
/// a `match` giving each variant's value, and `impl Into<E> for Int`, which checks `x` is a
/// variant's value and gives that variant. With the `@role(positions)` trait, the enum is also a
/// domain whose positions are its variants in declaration order (R02.7, R04.2.22).
pub(crate) fn enum_conversions(
    items: &[crate::modules::Item],
    (tr, method, check): &(String, String, String),
    positions: Option<&crate::records::Positions>,
) -> Vec<(ItemId, String, ast::Stmt)> {
    use ast::{CmpOp, ExprKind};
    let mut out = Vec::new();
    for crate::modules::Item { id: item, stmts } in items {
        for s in stmts {
            let StmtKind::Enum(e) = &s.kind else {
                continue;
            };
            let plain = (e.variants.iter()).all(|v| v.fields.is_empty() && v.tag.is_none());
            if !e.generics.is_empty() || e.variants.is_empty() || !plain {
                continue;
            }
            let values: Option<Vec<i64>> = (e.variants.iter().enumerate())
                .map(|(i, v)| match &v.value {
                    None => Some(i as i64),
                    Some(x) => x.int_literal(),
                })
                .collect();
            let Some(values) = values else {
                continue;
            };
            let span = s.span;
            let name = |text: &str| ast::Name {
                text: text.to_string(),
                span,
            };
            let ty = |text: &str, args| ast::Type {
                kind: TypeKind::Named {
                    name: name(text),
                    args,
                },
                span,
            };
            let ex = |kind| ast::Expr { kind, span };
            let stmt = |kind| ast::Stmt {
                kind,
                span,
                decorators: Vec::new(),
                public: false,
            };
            let block = |stmts: Vec<ast::Expr>| ast::Block {
                stmts: stmts.into_iter().map(|e| stmt(StmtKind::Expr(e))).collect(),
                span,
            };
            let this = || ex(ExprKind::Name("self".to_string()));
            let int = |k: i64| ex(ExprKind::Int(k as i128));
            let is = |k: i64| {
                ex(ExprKind::Compare {
                    first: Box::new(this()),
                    rest: vec![(CmpOp::Eq, int(k))],
                })
            };
            let variant = |v: &ast::Name| {
                ex(ExprKind::Field {
                    base: Box::new(ex(ExprKind::Name(e.name.text.clone()))),
                    name: v.clone(),
                })
            };
            let conversion = |to: ast::Type, from: ast::Type, body| {
                let f = ast::FnDecl {
                    name: name(method),
                    generics: Vec::new(),
                    bounds: Vec::new(),
                    params: vec![ast::Param {
                        mutable: false,
                        variadic: false,
                        lazy: false,
                        named_only: false,
                        label: None,
                        name: name("self"),
                        ty: None,
                        default: None,
                        span,
                    }],
                    ret: Some(to.clone()),
                    lends: None,
                    uses: Vec::new(),
                    body,
                    sig_span: span,
                };
                stmt(StmtKind::Impl {
                    trait_name: Some(ty(tr, vec![to])),
                    target: from,
                    bounds: Vec::new(),
                    body: ast::Block {
                        stmts: vec![stmt(StmtKind::Fn(f))],
                        span,
                    },
                })
            };
            let arms = (e.variants.iter().zip(&values))
                .map(|(v, k)| ast::Arm {
                    pat: ast::Pattern {
                        kind: ast::PatternKind::Variant {
                            path: vec![v.name.clone()],
                            args: Vec::new(),
                        },
                        span,
                    },
                    guard: None,
                    body: block(vec![int(*k)]),
                })
                .collect();
            let value = ex(ExprKind::Match {
                scrutinee: Box::new(this()),
                arms,
            });
            let known = (values.iter().map(|k| is(*k)))
                .reduce(|a, b| {
                    ex(ExprKind::Binary {
                        op: ast::BinOp::Or,
                        op_span: span,
                        lhs: Box::new(a),
                        rhs: Box::new(b),
                    })
                })
                .unwrap_or_else(|| ex(ExprKind::Bool(false)));
            let checked = ex(ExprKind::Call {
                callee: Box::new(ex(ExprKind::Name(check.clone()))),
                args: vec![ast::Arg {
                    name: None,
                    mutable: false,
                    value: known,
                }],
            });
            let last = e.variants.len() - 1;
            let pick = match last {
                0 => variant(&e.variants[0].name),
                _ => ex(ExprKind::If {
                    branches: (e.variants[..last].iter().zip(&values))
                        .map(|(v, k)| (ast::Cond::Expr(is(*k)), block(vec![variant(&v.name)])))
                        .collect(),
                    otherwise: Some(block(vec![variant(&e.variants[last].name)])),
                }),
            };
            let enum_ty = || ty(&e.name.text, Vec::new());
            let int_ty = || ty("Int", Vec::new());
            let into_int = conversion(int_ty(), enum_ty(), FnBody::Expr(value));
            let into_enum = conversion(
                enum_ty(),
                int_ty(),
                FnBody::Block(block(vec![checked, pick])),
            );
            out.push((item.clone(), e.name.text.clone(), into_int));
            out.push((item.clone(), e.name.text.clone(), into_enum));
            let Some(p) = positions else {
                continue;
            };
            let n = e.variants.len() as i64;
            let k = || ex(ExprKind::Name("k".to_string()));
            let cmp = |a, op, b| {
                ex(ExprKind::Compare {
                    first: Box::new(a),
                    rest: vec![(op, b)],
                })
            };
            let in_range = ex(ExprKind::Binary {
                op: ast::BinOp::And,
                op_span: span,
                lhs: Box::new(cmp(int(0), CmpOp::Le, k())),
                rhs: Box::new(cmp(k(), CmpOp::Lt, int(n))),
            });
            let checked = ex(ExprKind::Call {
                callee: Box::new(ex(ExprKind::Name(check.clone()))),
                args: vec![ast::Arg {
                    name: None,
                    mutable: false,
                    value: in_range,
                }],
            });
            let nth = match last {
                0 => variant(&e.variants[0].name),
                _ => ex(ExprKind::If {
                    branches: (e.variants[..last].iter().enumerate())
                        .map(|(i, v)| {
                            let at = cmp(k(), CmpOp::Eq, int(i as i64));
                            (ast::Cond::Expr(at), block(vec![variant(&v.name)]))
                        })
                        .collect(),
                    otherwise: Some(block(vec![variant(&e.variants[last].name)])),
                }),
            };
            let ords = (e.variants.iter().enumerate())
                .map(|(i, v)| ast::Arm {
                    pat: ast::Pattern {
                        kind: ast::PatternKind::Variant {
                            path: vec![v.name.clone()],
                            args: Vec::new(),
                        },
                        span,
                    },
                    guard: None,
                    body: block(vec![int(i as i64)]),
                })
                .collect();
            let ord = ex(ExprKind::Match {
                scrutinee: Box::new(this()),
                arms: ords,
            });
            let param = |n: &str, ty| ast::Param {
                mutable: false,
                variadic: false,
                lazy: false,
                named_only: false,
                label: None,
                name: name(n),
                ty,
                default: None,
                span,
            };
            let member = |getter: bool, n: &str, params, ret, body| ast::Stmt {
                kind: StmtKind::Fn(ast::FnDecl {
                    name: name(n),
                    generics: Vec::new(),
                    bounds: Vec::new(),
                    params,
                    ret: Some(ret),
                    lends: None,
                    uses: Vec::new(),
                    body,
                    sig_span: span,
                }),
                span,
                decorators: (getter.then(|| marker(Mark::Get, span)))
                    .into_iter()
                    .collect(),
                public: false,
            };
            let body = vec![
                member(true, &p.count, Vec::new(), int_ty(), FnBody::Expr(int(n))),
                member(
                    false,
                    &p.nth,
                    vec![param("k", Some(int_ty()))],
                    enum_ty(),
                    FnBody::Block(block(vec![checked, nth])),
                ),
                member(
                    true,
                    &p.ord,
                    vec![param("self", None)],
                    int_ty(),
                    FnBody::Expr(ord),
                ),
            ];
            let domain = stmt(StmtKind::Impl {
                trait_name: Some(ty(&p.tr, Vec::new())),
                target: enum_ty(),
                bounds: Vec::new(),
                body: ast::Block { stmts: body, span },
            });
            out.push((item.clone(), e.name.text.clone(), domain));
            // An enum's variants are in declaration order (R04.2.31).
            if let Some(o) = &p.order {
                let ordered = stmt(StmtKind::Impl {
                    trait_name: Some(ty(o, Vec::new())),
                    target: enum_ty(),
                    bounds: Vec::new(),
                    body: ast::Block {
                        stmts: Vec::new(),
                        span,
                    },
                });
                out.push((item.clone(), e.name.text.clone(), ordered));
            }
        }
    }
    out
}

/// The structs marked `@memo_table(word)`, by word: the module that declares the table a
/// `@memo(word)` fn keeps, and its name there. Any module may declare one (R06.4.1).
pub(crate) fn memo_tables(
    mods: &[crate::modules::Module],
) -> impl Iterator<Item = (String, (usize, String))> + '_ {
    (mods.iter().enumerate())
        .flat_map(|(mi, m)| {
            m.items
                .iter()
                .flat_map(move |it| it.stmts.iter().map(move |s| (mi, s)))
        })
        .filter_map(|(mi, s)| match &s.kind {
            StmtKind::Struct(sd) => (s.decorators.iter())
                .find(|x| x.mark == Some(Mark::MemoTable))
                .and_then(memo_word)
                .map(|w| (w, (mi, sd.name.text.clone()))),
            _ => None,
        })
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

fn memo_names(f: &ast::FnDecl) -> (String, String) {
    let n = &f.name.text;
    (format!("{n}.memo"), format!("{n}.body"))
}

/// A `@memo` fn's table type `T<K, R>` and its making, `T<K, R>()`. `K` is the one parameter's
/// type, else the tuple of them.
fn memo_new(f: &ast::FnDecl, span: cpl_diag::Span, table: &str) -> (ast::Type, ast::Expr) {
    let f = &*jagged(f).0;
    let name = |text: &str| ast::Name {
        text: text.to_string(),
        span,
    };
    let tys: Vec<ast::Type> = f.params.iter().filter_map(|p| p.ty.clone()).collect();
    let key = match &tys[..] {
        [t] => t.clone(),
        _ => ast::Type {
            kind: TypeKind::Tuple(tys),
            span,
        },
    };
    let error = ast::Type {
        kind: TypeKind::Error,
        span,
    };
    let args = vec![key, f.ret.clone().unwrap_or(error)];
    let t = ast::Type {
        kind: TypeKind::Named {
            name: name(table),
            args: args.clone(),
        },
        span,
    };
    let ex = |kind| ast::Expr { kind, span };
    let make = ex(ast::ExprKind::Call {
        callee: Box::new(ex(ast::ExprKind::Generic {
            base: Box::new(ex(ast::ExprKind::Name(table.to_string()))),
            args,
        })),
        args: Vec::new(),
    });
    (t, make)
}

/// A `@memo` fn's table, `var f.memo: T<K, R> = T<K, R>()`, a statement of the block that
/// declares `f`, so the cache lives as long as it does (R06.4.1).
pub(crate) fn memo_var(f: &ast::FnDecl, span: cpl_diag::Span, table: &str) -> ast::Stmt {
    let (t, make) = memo_new(f, span, table);
    ast::Stmt {
        kind: StmtKind::Let {
            mutable: true,
            pat: ast::Pattern {
                kind: ast::PatternKind::Name(memo_names(f).0),
                span,
            },
            ty: Some(t),
            init: Some(make),
            otherwise: None,
            guard: None,
        },
        span,
        decorators: Vec::new(),
        public: false,
    }
}

/// A `@memo` fn as its body, `fn f.body(ps) -> R`, `f` itself, which answers from the table when
/// it has the key and else runs the body and keeps its result, and `f.clear()`, which empties the
/// table. The body's own calls of `f` go through the table.
pub(crate) fn memo_fns(f: &ast::FnDecl, s: &ast::Stmt, table: &str) -> [ast::Stmt; 3] {
    let span = f.sig_span;
    let ex = |kind| ast::Expr { kind, span };
    let var = |n: &str| ex(ast::ExprKind::Name(n.to_string()));
    let arg = |value| ast::Arg {
        name: None,
        mutable: false,
        value,
    };
    let stmt = |kind| ast::Stmt {
        kind,
        span,
        decorators: Vec::new(),
        public: false,
    };
    let pat = |n: &str| ast::Pattern {
        kind: ast::PatternKind::Name(n.to_string()),
        span,
    };
    let call = |callee, args| {
        ex(ast::ExprKind::Call {
            callee: Box::new(callee),
            args,
        })
    };
    let method = |m: &str| {
        ex(ast::ExprKind::Field {
            base: Box::new(var(&memo_names(f).0)),
            name: ast::Name {
                text: m.to_string(),
                span,
            },
        })
    };
    let (_, body_name) = memo_names(f);
    let ps: Vec<&str> = f.params.iter().map(|p| p.name.text.as_str()).collect();
    let key = || match &ps[..] {
        [p] => var(p),
        _ => ex(ast::ExprKind::Tuple(ps.iter().map(|p| var(p)).collect())),
    };
    let (hit, got) = ("memo.hit", "memo.got");
    let look = ast::ExprKind::If {
        branches: vec![(
            ast::Cond::Let {
                pat: pat(hit),
                value: call(method("get"), vec![arg(key())]),
                and: None,
            },
            ast::Block {
                stmts: vec![stmt(StmtKind::Return(Some(var(hit))))],
                span,
            },
        )],
        otherwise: None,
    };
    let run = call(var(&body_name), ps.iter().map(|p| arg(var(p))).collect());
    let wrapper = ast::FnDecl {
        body: FnBody::Block(ast::Block {
            stmts: vec![
                stmt(StmtKind::Expr(ex(look))),
                stmt(StmtKind::Let {
                    mutable: false,
                    pat: pat(got),
                    ty: None,
                    init: Some(run),
                    otherwise: None,
                    guard: None,
                }),
                stmt(StmtKind::Expr(call(
                    method("set"),
                    vec![arg(key()), arg(var(got))],
                ))),
                stmt(StmtKind::Expr(var(got))),
            ],
            span,
        }),
        ..f.clone()
    };
    // Its arguments were checked where the wrapper was called.
    let body = ast::FnDecl {
        name: ast::Name {
            text: body_name,
            span: f.name.span,
        },
        params: jagged(f).0.params.clone(),
        ..f.clone()
    };
    let (_, make) = memo_new(f, span, table);
    let clear = ast::FnDecl {
        name: ast::Name {
            text: memo_clear_name(&f.name.text),
            span: f.name.span,
        },
        generics: Vec::new(),
        bounds: Vec::new(),
        params: Vec::new(),
        ret: None,
        lends: None,
        body: FnBody::Block(ast::Block {
            stmts: vec![stmt(StmtKind::Assign {
                target: var(&memo_names(f).0),
                op: ast::AssignOp::Set,
                value: make,
            })],
            span,
        }),
        ..f.clone()
    };
    let others = (s.decorators.iter())
        .filter(|x| x.mark != Some(Mark::Memo))
        .cloned()
        .collect();
    [
        // Its own span start, apart from `f`'s, which names a block's nested fn.
        ast::Stmt {
            span: f.name.span,
            ..stmt(StmtKind::Fn(body))
        },
        ast::Stmt {
            kind: StmtKind::Fn(wrapper),
            span,
            decorators: others,
            public: s.public,
        },
        // Its own span start too: one past the name's, which `f.body`'s has.
        ast::Stmt {
            span: cpl_diag::Span::new(f.name.span.start + 1, f.name.span.end),
            public: s.public,
            ..stmt(StmtKind::Fn(clear))
        },
    ]
}

/// A block's statements with its `@memo` fns written out, and where each fn's table now sits.
pub type MemoBlock<'s> = (Vec<ast::Stmt>, Vec<(usize, &'s ast::FnDecl)>);

/// The name `@memo` fn `f`'s clear is declared under, written `f.clear()` (R06.4.1).
pub fn memo_clear_name(f: &str) -> String {
    format!("{f}.clear")
}
/// A block's statements with each `@memo` fn written out as its table, a `var` of the block, and
/// its fns, with where each table's `var` now sits and the fn it is for; `None` when it has none
/// (R06.4.1).
pub fn memo_block<'s>(
    stmts: &'s [ast::Stmt],
    tables: &dyn Fn(&str) -> bool,
) -> Option<MemoBlock<'s>> {
    stmts
        .iter()
        .any(|s| memo_table(s, tables).is_some())
        .then(|| {
            let (mut out, mut at) = (Vec::new(), Vec::new());
            for s in stmts {
                match memo_table(s, tables) {
                    Some((f, t)) => {
                        at.push((out.len(), f));
                        out.push(memo_var(f, f.sig_span, &t));
                        out.extend(memo_fns(f, s, &t));
                    }
                    None => out.push(s.clone()),
                }
            }
            (out, at)
        })
}

/// `fn c(n: 0..=30, k: 0..=n)`: a range parameter whose ends name earlier parameters, so a
/// `@memo(array)` table over it is jagged (R06.4.1). Its type is the range those ends reach over
/// the earlier parameters' ranges, `0..=30`; a call checks its argument into the written range,
/// where an ordinary range parameter's is checked (R04.3.4).
pub fn jagged(f: &ast::FnDecl) -> Jagged<'_> {
    use ast::{BinOp, ExprKind};
    fn reach(e: &ast::Expr, known: &HashMap<&str, (i64, i64)>) -> Option<(i64, i64)> {
        match &e.kind {
            ExprKind::Paren(x) => reach(x, known),
            ExprKind::Name(n) => known.get(n.as_str()).copied(),
            ExprKind::Binary { op, lhs, rhs, .. } if matches!(op, BinOp::Add | BinOp::Sub) => {
                let ((a, b), k) = (reach(lhs, known)?, rhs.int_literal()?);
                let k = if *op == BinOp::Sub {
                    k.checked_neg()?
                } else {
                    k
                };
                Some((a.checked_add(k)?, b.checked_add(k)?))
            }
            _ => e.int_literal().map(|v| (v, v)),
        }
    }
    let mut known = HashMap::new();
    let (mut params, mut written) = (f.params.clone(), Vec::new());
    for p in &mut params {
        let Some(TypeKind::Range { lo, hi, inclusive }) = p.ty.as_ref().map(|t| &t.kind) else {
            continue;
        };
        if let Some(r) = crate::decls::static_range(lo, hi, *inclusive) {
            known.insert(p.name.text.as_str(), r);
            continue;
        }
        let ends = |e: &Option<Box<ast::Expr>>| e.as_deref().map(|e| reach(e, &known));
        let (Some(Some((a, _))), Some(Some((_, b)))) = (ends(lo), ends(hi)) else {
            continue;
        };
        let Some(b) = b.checked_sub(i64::from(!*inclusive)) else {
            continue;
        };
        let span = p.name.span;
        let int = |v: i64| {
            Some(Box::new(ast::Expr {
                kind: ExprKind::Int(i128::from(v)),
                span,
            }))
        };
        let wide = ast::Type {
            kind: TypeKind::Range {
                lo: int(a),
                hi: int(b),
                inclusive: true,
            },
            span,
        };
        if let Some(t) = p.ty.replace(wide) {
            written.push((p.name.text.clone(), t));
        }
        known.insert(p.name.text.as_str(), (a, b));
    }
    if written.is_empty() {
        return (std::borrow::Cow::Borrowed(f), written);
    }
    let f = ast::FnDecl {
        params,
        ..f.clone()
    };
    (std::borrow::Cow::Owned(f), written)
}

/// A fn with each jagged parameter's type widened, and each one's written type by its name.
pub type Jagged<'f> = (std::borrow::Cow<'f, ast::FnDecl>, Vec<(String, ast::Type)>);
