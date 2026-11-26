//! Declarations the compiler writes for a program: an enum's structs and the `Eq` and `Ord` a
//! struct derives from its fields.
use crate::decls::{constant_int, unnamed};
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

/// Per struct a module declares, the impls it gets structurally (R08.1.6), made after every written
/// impl and only where none is: `Eq` field by field and `Ord` lexicographic in declaration order.
pub(crate) fn module_derived(items: &[(u32, Vec<ast::Stmt>)]) -> Vec<(u32, String, ast::Stmt)> {
    let mut out = Vec::new();
    for (item, stmts) in items {
        for s in stmts {
            let StmtKind::Struct(sd) = &s.kind else {
                continue;
            };
            // The traits `==` and `<` call: the derivable ones.
            for op in [ast::CmpOp::Eq, ast::CmpOp::Lt] {
                let Some((tr, method)) = op.trait_method() else {
                    continue;
                };
                let derived = derived_impl(s, sd, op, tr, method);
                out.push((*item, sd.name.text.clone(), derived));
            }
        }
    }
    out
}

/// `impl<T: Eq> Eq for S<T>: fn eq(a, b) = a.x == b.x and ..`, or `Ord`'s `lt` over the fields in
/// order, bounding the parameters the fields mention and written at the struct.
fn derived_impl(
    s: &ast::Stmt,
    sd: &ast::StructDecl,
    op: ast::CmpOp,
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
    // A `@cache` field is not part of the value (R05.10.13).
    let compared: Vec<&ast::FieldDecl> = sd.fields.iter().filter(|f| !f.cached).collect();
    let fields: Vec<&str> = compared.iter().map(|f| f.name.text.as_str()).collect();
    let body = match op {
        CmpOp::Eq => (fields.iter().map(|f| cmp(CmpOp::Eq, f)))
            .reduce(|a, b| bin(BinOp::And, a, b))
            .unwrap_or_else(|| ex(ExprKind::Bool(true))),
        _ => (fields.iter().rev())
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
        params: vec![param("a"), param("b")],
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
