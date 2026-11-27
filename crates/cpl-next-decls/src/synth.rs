//! Declarations the compiler writes for a program: an enum's structs and the `Eq` and `Ord` a
//! struct derives from its fields.
use crate::decls::{constant_int, unnamed, GETTER};
use crate::diag::Msg;
use cpl_next_ir::ids::ItemId;
use cpl_next_ir::unbuilt::Unbuilt;
use cpl_syntax::ast::{self, FnBody, StmtKind, TypeKind};
use std::collections::HashMap;

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
        if v.tag.is_some() {
            errors.push(
                Unbuilt::TaggedVariant {
                    name: v.name.text.clone(),
                }
                .into(),
            );
        }
        let value = match &v.value {
            None => Some(i as i64),
            Some(_) if !v.fields.is_empty() => None,
            Some(x) => constant_int(x),
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

enum Derive {
    Cmp(ast::CmpOp),
    /// A core trait marked `@derive`: its one method builds a struct from that method on each field.
    Each,
}

/// Per struct a module declares, the impls it gets structurally (R08.1.6), made after every written
/// impl and only where none is: `Eq` field by field, `Ord` lexicographic in declaration order, and
/// `Clone` cloning each field (R05.2.2).
pub(crate) fn module_derived(
    items: &[crate::modules::Item],
    each: &[(String, String)],
) -> Vec<(ItemId, String, ast::Stmt)> {
    let mut out = Vec::new();
    for crate::modules::Item { id: item, stmts } in items {
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
                out.push((item.clone(), sd.name.text.clone(), derived));
            }
            for (tr, method) in each {
                let derived = derived_impl(s, sd, Derive::Each, tr, method);
                out.push((item.clone(), sd.name.text.clone(), derived));
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

/// The traits marked `@derive`, with their one method: every struct whose fields all have
/// one gets it field by field (R08.1.6).
pub(crate) fn derive_each<'a>(stmts: impl Iterator<Item = &'a ast::Stmt>) -> Vec<(String, String)> {
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
            out.push((item.clone(), crate::records::name(fs), imp));
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

/// The `@convert` trait with its one method, and the fn marked `@variant_check`: what a
/// payload-free enum's conversions are written with.
pub(crate) fn convert_marked<'a, I: Iterator<Item = &'a ast::Stmt>>(
    stmts: impl Fn() -> I,
) -> Option<(String, String, String)> {
    let marked = |s: &ast::Stmt, n: &str| s.decorators.iter().any(|d| d.name.text == n);
    let (tr, method) = (stmts().filter(|s| marked(s, "convert"))).find_map(|s| match &s.kind {
        StmtKind::Trait { name, body, .. } => match &body.stmts[..] {
            [ast::Stmt {
                kind: StmtKind::Fn(f),
                ..
            }] => Some((name.text.clone(), f.name.text.clone())),
            _ => None,
        },
        _ => None,
    })?;
    let check = (stmts().filter(|s| marked(s, "variant_check"))).find_map(|s| match &s.kind {
        StmtKind::Fn(f) => Some(f.name.text.clone()),
        _ => None,
    })?;
    Some((tr, method, check))
}

/// Per payload-free enum a module declares, `Int(e)` and `E(x)` (R02.7): `impl Into<Int> for E`,
/// a `match` giving each variant's value, and `impl Into<E> for Int`, which checks `x` is a
/// variant's value and gives that variant.
pub(crate) fn enum_conversions(
    items: &[crate::modules::Item],
    (tr, method, check): &(String, String, String),
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
                    Some(x) => constant_int(x),
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
        }
    }
    out
}

/// The structs marked `@memo_table(word)`, by word: the table a `@memo(word)` fn keeps. Any
/// module may declare one (R06.4.1).
pub(crate) fn memo_tables(mods: &[crate::modules::Module]) -> HashMap<String, String> {
    (mods
        .iter()
        .flat_map(|m| m.items.iter())
        .flat_map(|it| &it.stmts))
    .filter_map(|s| match &s.kind {
        StmtKind::Struct(sd) => (s.decorators.iter())
            .find(|x| x.name.text == "memo_table")
            .and_then(memo_word)
            .map(|w| (w, sd.name.text.clone())),
        _ => None,
    })
    .collect()
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
pub(crate) fn memo_table<'s, 'a>(
    s: &'s ast::Stmt,
    tables: &'a HashMap<String, String>,
) -> Option<(&'s ast::FnDecl, &'a String)> {
    let StmtKind::Fn(f) = &s.kind else {
        return None;
    };
    let x = s.decorators.iter().find(|x| x.name.text == "memo")?;
    let plain = |p: &ast::Param| p.ty.is_some() && !p.mutable && !p.variadic && !p.lazy;
    let typed = f.ret.is_some() && !f.params.is_empty() && f.params.iter().all(plain);
    (typed && f.generics.is_empty() && f.lends.is_none()).then_some(())?;
    Some((f, tables.get(&memo_word(x)?)?))
}

fn memo_names(f: &ast::FnDecl) -> (String, String) {
    let n = &f.name.text;
    (format!("{n}.memo"), format!("{n}.body"))
}

/// A `@memo` fn's table, `var f.memo: T<K, R> = T<K, R>()`, a statement of the block that
/// declares `f`, so the cache lives as long as it does (R06.4.1). `K` is the one parameter's
/// type, else the tuple of them.
pub(crate) fn memo_var(f: &ast::FnDecl, span: cpl_diag::Span, table: &str) -> ast::Stmt {
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
    let args = vec![
        key,
        f.ret.clone().unwrap_or(ast::Type {
            kind: TypeKind::Error,
            span,
        }),
    ];
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

/// A `@memo` fn as its body, `fn f.body(ps) -> R`, and `f` itself, which answers from the table
/// when it has the key and else runs the body and keeps its result. The body's own calls of `f`
/// go through the table.
pub(crate) fn memo_fns(f: &ast::FnDecl, s: &ast::Stmt) -> [ast::Stmt; 2] {
    let span = s.span;
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
    let body = ast::FnDecl {
        name: ast::Name {
            text: body_name,
            span: f.name.span,
        },
        ..f.clone()
    };
    let others = (s.decorators.iter())
        .filter(|x| x.name.text != "memo")
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
    ]
}

/// A block's statements with its `@memo` fns written out, and where each fn's table now sits.
pub type MemoBlock<'s> = (Vec<ast::Stmt>, Vec<(usize, &'s ast::FnDecl)>);

/// A block's statements with each `@memo` fn written out as its table, a `var` of the block, and
/// its fns, with where each table's `var` now sits and the fn it is for; `None` when it has none
/// (R06.4.1).
pub fn memo_block<'s>(
    stmts: &'s [ast::Stmt],
    tables: &HashMap<String, String>,
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
                        out.push(memo_var(f, s.span, t));
                        out.extend(memo_fns(f, s));
                    }
                    None => out.push(s.clone()),
                }
            }
            (out, at)
        })
}
