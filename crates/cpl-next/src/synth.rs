//! Declarations the compiler writes for a program: an enum's structs and the `Eq` and `Ord` a
//! struct derives from its fields.
use crate::decls::{constant_int, unnamed, GETTER};
use cpl_syntax::ast::{self, FnBody, StmtKind, TypeKind};
use std::collections::HashMap;

/// An enum as structs: its own, with no fields (the parser makes its methods an `impl`), then one
/// per variant holding that variant's payload, named `Enum.Variant`, which no source name can be
/// (R02.7).
pub(crate) fn enum_structs(
    s: &ast::Stmt,
    e: &ast::EnumDecl,
) -> (Vec<ast::Stmt>, Vec<String>, Vec<i64>) {
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
        if v.tag.is_some() {
            errors.push(format!("slice: a tagged variant `{}`", v.name.text));
        }
        let value = match &v.value {
            None => Some(i as i64),
            Some(_) if !v.fields.is_empty() => None,
            Some(x) => constant_int(x),
        };
        if value.is_none() {
            errors.push(format!("slice: the value of variant `{}`", v.name.text));
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

enum Derive {
    Cmp(ast::CmpOp),
    /// A core trait marked `@derive`: its one method builds a struct from that method on each field.
    Each,
}

/// Per struct a module declares, the impls it gets structurally (R08.1.6), made after every written
/// impl and only where none is: `Eq` field by field, `Ord` lexicographic in declaration order, and
/// `Clone` cloning each field (R05.2.2).
pub(crate) fn module_derived(
    items: &[(u32, Vec<ast::Stmt>)],
    each: &[(String, String)],
) -> Vec<(u32, String, ast::Stmt)> {
    let mut out = Vec::new();
    for (item, stmts) in items {
        for s in stmts {
            let StmtKind::Struct(sd) = &s.kind else {
                continue;
            };
            // The traits `==` and `<` call, and `.clone()`: the derivable ones.
            for op in [ast::CmpOp::Eq, ast::CmpOp::Lt] {
                let Some((tr, method)) = op.trait_method() else {
                    continue;
                };
                let derived = derived_impl(s, sd, Derive::Cmp(op), tr, method);
                out.push((*item, sd.name.text.clone(), derived));
            }
            for (tr, method) in each {
                let derived = derived_impl(s, sd, Derive::Each, tr, method);
                out.push((*item, sd.name.text.clone(), derived));
            }
        }
    }
    out
}

/// `impl<T: Eq> Eq for S<T>: fn eq(a, b) = a.x == b.x and ..`, `Ord`'s `lt` over the fields in
/// order, or `fn clone(a) = S(x: a.x.clone(), ..)`, bounding the parameters the fields mention and
/// written at the struct.
fn derived_impl(
    s: &ast::Stmt,
    sd: &ast::StructDecl,
    derive: Derive,
    tr: &str,
    method: &str,
) -> ast::Stmt {
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
    let field = |side: &str, f: &str| {
        ex(ExprKind::Field {
            base: Box::new(ex(ExprKind::Name(side.to_string()))),
            name: name(f),
        })
    };
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
    // A `@cache` field is not part of the value it compares (R05.10.13); a clone copies it too.
    let clone = matches!(derive, Derive::Each);
    let compared: Vec<&ast::FieldDecl> = sd.fields.iter().filter(|f| clone || !f.cached).collect();
    let fields: Vec<&str> = compared.iter().map(|f| f.name.text.as_str()).collect();
    let body = match derive {
        Derive::Each => {
            let args = (fields.iter())
                .map(|f| ast::Arg {
                    name: Some(name(f)),
                    mutable: false,
                    value: ex(ExprKind::Call {
                        callee: Box::new(ex(ExprKind::Field {
                            base: Box::new(field("a", f)),
                            name: name(method),
                        })),
                        args: Vec::new(),
                    }),
                })
                .collect();
            ex(ExprKind::Call {
                callee: Box::new(ex(ExprKind::Name(sd.name.text.clone()))),
                args,
            })
        }
        Derive::Cmp(CmpOp::Eq) => (fields.iter().map(|f| cmp(CmpOp::Eq, f)))
            .reduce(|a, b| bin(BinOp::And, a, b))
            .unwrap_or_else(|| ex(ExprKind::Bool(true))),
        Derive::Cmp(_) => (fields.iter().rev())
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
    let mut mentioned = Vec::new();
    for f in &compared {
        unnamed(&f.ty, &HashMap::new(), &mut mentioned);
    }
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
    let f = ast::FnDecl {
        name: name(method),
        generics: Vec::new(),
        bounds: Vec::new(),
        params: match derive {
            Derive::Each => vec![param("a")],
            Derive::Cmp(_) => vec![param("a"), param("b")],
        },
        ret: None,
        lends: None,
        uses: Vec::new(),
        body: FnBody::Expr(body),
        sig_span: span,
    };
    let stmt = |kind| ast::Stmt {
        kind,
        span,
        decorators: Vec::new(),
        public: false,
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
        bounds: (sd.generics.iter())
            .filter(|g| mentioned.contains(&g.text))
            .map(|g| ast::Bound {
                name: g.clone(),
                traits: vec![ty(tr, Vec::new())],
            })
            .collect(),
        body: ast::Block {
            stmts: vec![stmt(StmtKind::Fn(f))],
            span,
        },
    })
}

/// The core traits marked `@derive`, with their one method: every struct whose fields all have
/// one gets it field by field (R08.1.6).
pub(crate) fn derive_each(core: &[(u32, Vec<ast::Stmt>)]) -> Vec<(String, String)> {
    let stmts = core.iter().flat_map(|(_, ss)| ss);
    let marked = stmts.filter(|s| s.decorators.iter().any(|d| d.name.text == "derive"));
    marked
        .filter_map(|s| match &s.kind {
            StmtKind::Trait { name, body, .. } => match &body.stmts[..] {
                [ast::Stmt {
                    kind: StmtKind::Fn(f),
                    ..
                }] => Some((name.text.clone(), f.name.text.clone())),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

/// `@get fn name(self) = self.name`: how a struct meets a bodiless `@get` member its impl leaves
/// out (R12.11.3).
pub(crate) fn field_getter(field: &str, at: &ast::Stmt) -> ast::Stmt {
    let span = at.span;
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
        decorators: vec![ast::Decorator {
            directive: false,
            name: name(GETTER),
            args: Vec::new(),
            span,
        }],
        public: false,
    }
}

/// Per record shape with a field for each member of a trait made only of bodiless `@get` members,
/// a bodiless impl of it, written with the trait: `impl<V, A> Target<V> for (to: V, w: A)`, its
/// members then met by the fields (R12.11.3). A written impl replaces it.
pub(crate) fn field_trait_impls(
    items: &[(u32, Vec<ast::Stmt>)],
    shapes: &[Vec<String>],
) -> Vec<(u32, String, ast::Stmt)> {
    let mut out = Vec::new();
    for (item, s) in items
        .iter()
        .flat_map(|(i, ss)| ss.iter().map(move |s| (i, s)))
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
                        && m.decorators.iter().any(|d| d.name.text == GETTER) =>
                {
                    (f.ret.as_ref())
                        .filter(|t| plain(t))
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
            unnamed(t, &HashMap::new(), &mut taken);
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
            out.push((*item, crate::records::name(fs), imp));
        }
    }
    out
}

/// A field type an impl can write as it is: no `Self`, no associated type.
fn plain(t: &ast::Type) -> bool {
    match &t.kind {
        TypeKind::Named { name, args } => {
            !name.text.starts_with("Self") && !name.text.contains('.') && args.iter().all(plain)
        }
        TypeKind::Optional(e) => plain(e),
        TypeKind::Tuple(ts) => ts.iter().all(plain),
        TypeKind::Array { index, elem } => index.iter().chain([&**elem]).all(plain),
        _ => false,
    }
}
