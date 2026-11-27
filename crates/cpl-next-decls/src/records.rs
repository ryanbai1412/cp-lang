//! Records `(lo: Int, hi: V)` (R02.5.2). Each list of field names, in order, is one generic core
//! struct `(lo:, hi:)<A, B>`, so two record types are equal exactly when their field names, order
//! and types are, and fields, constructors, patterns and derived `Eq` and `Ord` are a struct's.
use cpl_diag::Span;
use cpl_syntax::ast::{self, ExprKind, FnDecl, StmtKind, TypeKind};
use cpl_syntax::visit::{self, Visitor};
use std::collections::{BTreeSet, HashMap};

/// The struct a record with these fields is: `(lo:, hi:)`.
pub fn name<S: AsRef<str>>(fields: &[S]) -> String {
    let fs: Vec<String> = fields.iter().map(|f| format!("{}:", f.as_ref())).collect();
    format!("({})", fs.join(", "))
}

/// The mark a record's synthesized struct carries, which no source can write (E0107).
pub const RECORD: &str = "record";

/// Each fn with a last `..xs` parameter, by name, with the labels of its other parameters: the
/// arguments of a call by any other names are the record `xs` is (R06.1.5).
pub fn variadics(mods: &[crate::modules::Module]) -> HashMap<String, Vec<String>> {
    let mut out: HashMap<String, Vec<String>> = HashMap::new();
    for crate::modules::Item { id: _, stmts } in mods.iter().flat_map(|m| m.items.iter()) {
        for s in stmts {
            let StmtKind::Fn(f) = &s.kind else {
                continue;
            };
            if f.params.last().is_some_and(|p| p.variadic) {
                let labels = f
                    .params
                    .iter()
                    .map(|p| &p.label.as_ref().unwrap_or(&p.name).text);
                out.entry(f.name.text.clone())
                    .or_default()
                    .extend(labels.cloned());
            }
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
    let mut v = Shapes(BTreeSet::new(), variadics);
    for crate::modules::Item { id: _, stmts } in mods.iter().flat_map(|m| m.items.iter()) {
        stmts.iter().for_each(|s| visit::walk_stmt(s, &mut v));
    }
    v.0.into_iter().collect()
}

/// `pub struct (lo:, hi:)<A, B>: pub lo: A, pub hi: B` for each shape.
pub fn structs(shapes: &[Vec<String>], span: Span) -> Vec<ast::Stmt> {
    let nm = |text: String| ast::Name { text, span };
    (shapes.iter())
        .filter(|fs| fs.len() <= 26)
        .map(|fs| {
            // Type parameters are single capitals, as the derived impls' bounds expect.
            let generics: Vec<ast::Name> = (b'A'..)
                .take(fs.len())
                .map(|c| nm((c as char).to_string()))
                .collect();
            let fields = (fs.iter().zip(&generics))
                .map(|(f, g)| ast::FieldDecl {
                    name: nm(f.clone()),
                    ty: ast::Type {
                        kind: TypeKind::Named {
                            name: g.clone(),
                            args: Vec::new(),
                        },
                        span,
                    },
                    default: None,
                    // A record's fields are assignable through a `var` that holds it (R02.1.1).
                    mutable: true,
                    cached: false,
                    bits: false,
                    public: true,
                })
                .collect();
            ast::Stmt {
                kind: StmtKind::Struct(ast::StructDecl {
                    name: nm(name(fs)),
                    generic_defaults: vec![None; fs.len()],
                    generics,
                    bounds: Vec::new(),
                    fields,
                    methods: Vec::new(),
                }),
                span,
                decorators: vec![ast::Decorator {
                    directive: false,
                    name: nm(RECORD.to_string()),
                    args: Vec::new(),
                    span,
                }],
                public: true,
            }
        })
        .collect()
}

struct Shapes<'v>(BTreeSet<Vec<String>>, &'v HashMap<String, Vec<String>>);

impl Shapes<'_> {
    fn ty(&mut self, t: &ast::Type) {
        match &t.kind {
            TypeKind::Record(fs) => {
                self.0
                    .insert(fs.iter().map(|(n, _)| n.text.clone()).collect());
                fs.iter().for_each(|(_, t)| self.ty(t));
            }
            TypeKind::Row(fs) => fs.iter().for_each(|(_, t)| self.ty(t)),
            TypeKind::Named { args: ts, .. } | TypeKind::Tuple(ts) => {
                ts.iter().for_each(|t| self.ty(t))
            }
            TypeKind::Array { index, elem } => {
                index.iter().for_each(|t| self.ty(t));
                self.ty(elem);
            }
            TypeKind::Fn { params, ret } => {
                params.iter().for_each(|t| self.ty(t));
                self.ty(ret);
            }
            TypeKind::Optional(t) | TypeKind::Mut(t) | TypeKind::Yields(t) => self.ty(t),
            _ => {}
        }
    }
    fn fn_decl(&mut self, f: &FnDecl) {
        let params = f.params.iter().filter_map(|p| p.ty.as_ref());
        for t in params.chain(&f.ret).chain(f.lends.as_ref().map(|l| &l.ty)) {
            self.ty(t);
        }
    }
}

impl Visitor for Shapes<'_> {
    fn stmt(&mut self, s: &ast::Stmt) {
        match &s.kind {
            StmtKind::Let { ty: Some(t), .. } | StmtKind::Const { ty: Some(t), .. } => self.ty(t),
            StmtKind::Fn(f) => self.fn_decl(f),
            StmtKind::Struct(sd) => sd.fields.iter().for_each(|f| self.ty(&f.ty)),
            StmtKind::TypeAlias { ty, .. } => self.ty(ty),
            StmtKind::Impl { target, .. } => self.ty(target),
            _ => {}
        }
    }
    fn expr(&mut self, e: &ast::Expr) {
        match &e.kind {
            ExprKind::Record(fs) => {
                self.0
                    .insert(fs.iter().map(|(n, _)| n.text.clone()).collect());
            }
            ExprKind::Call { callee, args } => {
                let ExprKind::Name(n) = &callee.kind else {
                    return;
                };
                let Some(fixed) = self.1.get(n) else {
                    return;
                };
                let named: Vec<String> = (args.iter())
                    .filter_map(|a| a.name.as_ref().map(|n| &n.text))
                    .filter(|n| !fixed.contains(n))
                    .cloned()
                    .collect();
                if !named.is_empty() {
                    self.0.insert(named);
                }
            }
            ExprKind::Generic { args, .. } => args.iter().for_each(|t| self.ty(t)),
            ExprKind::Table { domains, .. } => domains.iter().for_each(|t| self.ty(t)),
            ExprKind::Lambda {
                param_tys, ret_ty, ..
            } => param_tys
                .iter()
                .flatten()
                .chain(ret_ty)
                .for_each(|t| self.ty(t)),
            _ => {}
        }
    }
}

/// The `@positions` trait, in whichever module declares it, and the names of its members marked
/// `dom_count`, `dom_nth` and `dom_ord`: what a domain the compiler declares implements.
pub struct Positions {
    pub tr: String,
    pub count: String,
    pub nth: String,
    pub ord: String,
}

pub fn positions<'a>(stmts: impl IntoIterator<Item = &'a ast::Stmt>) -> Option<Positions> {
    let marked =
        (stmts.into_iter()).find(|s| s.decorators.iter().any(|d| d.name.text == "positions"));
    let Some(StmtKind::Trait { name: tr, body, .. }) = marked.map(|s| &s.kind) else {
        return None;
    };
    let member = |id: &str| {
        body.stmts.iter().find_map(|s| match &s.kind {
            StmtKind::Fn(f) => match &f.body {
                ast::FnBody::Expr(e) if matches!(&e.kind, ExprKind::Intrinsic(i) if i == id) => {
                    Some(f.name.text.clone())
                }
                _ => None,
            },
            _ => None,
        })
    };
    Some(Positions {
        tr: tr.text.clone(),
        count: member("dom_count")?,
        nth: member("dom_nth")?,
        ord: member("dom_ord")?,
    })
}

/// A record of domains is the product domain over its fields, walked row-major with the last
/// field fastest, as core's tuples are (R04.2.10): per shape, `impl<A: Domain, ..> Domain for
/// (lo: A, ..)` giving the members the `@positions` trait marks `dom_count`, `dom_nth` and
/// `dom_ord`.
pub fn domain_impls<'a>(
    stmts: impl IntoIterator<Item = &'a ast::Stmt>,
    shapes: &[Vec<String>],
    span: Span,
) -> Vec<(String, ast::Stmt)> {
    let Some(Positions {
        tr,
        count,
        nth,
        ord,
    }) = positions(stmts)
    else {
        return Vec::new();
    };
    let tr = ast::Name { text: tr, span };
    let nm = |text: &str| ast::Name {
        text: text.to_string(),
        span,
    };
    let named = |text: &str| ast::Type {
        kind: TypeKind::Named {
            name: nm(text),
            args: Vec::new(),
        },
        span,
    };
    let ex = |kind| ast::Expr { kind, span };
    let field = |base: ast::Expr, f: &str| {
        ex(ExprKind::Field {
            base: Box::new(base),
            name: nm(f),
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
    let size = |g: &str| field(ex(ExprKind::Name(g.to_string())), &count);
    let param = |n: &str, ty: Option<ast::Type>| ast::Param {
        mutable: false,
        variadic: false,
        lazy: false,
        named_only: false,
        label: None,
        name: nm(n),
        ty,
        default: None,
        span,
    };
    let get = || ast::Decorator {
        directive: false,
        name: nm(crate::decls::GETTER),
        args: Vec::new(),
        span,
    };
    let method = |getter: bool, name: &str, params, ret, body| ast::Stmt {
        kind: StmtKind::Fn(FnDecl {
            name: nm(name),
            generics: Vec::new(),
            bounds: Vec::new(),
            params,
            ret: Some(ret),
            lends: None,
            uses: Vec::new(),
            body: ast::FnBody::Expr(body),
            sig_span: span,
        }),
        span,
        decorators: if getter { vec![get()] } else { Vec::new() },
        public: false,
    };
    let mut out = Vec::new();
    for fs in shapes.iter().filter(|fs| !fs.is_empty() && fs.len() <= 26) {
        let gs: Vec<String> = (b'A'..)
            .take(fs.len())
            .map(|c| (c as char).to_string())
            .collect();
        let record = ast::Type {
            kind: TypeKind::Record(fs.iter().zip(&gs).map(|(f, g)| (nm(f), named(g))).collect()),
            span,
        };
        let total = (gs.iter().map(|g| size(g))).reduce(|a, b| bin(ast::BinOp::Mul, a, b));
        // Field `i` is `k / (the sizes after it) % its size`.
        let parts = (fs.iter().enumerate()).map(|(i, f)| {
            let after =
                (gs[i + 1..].iter().map(|g| size(g))).reduce(|a, b| bin(ast::BinOp::Mul, a, b));
            let k = ex(ExprKind::Name("k".to_string()));
            let k = after.map_or(k.clone(), |a| bin(ast::BinOp::Div, k, a));
            let k = if i == 0 {
                k
            } else {
                bin(ast::BinOp::Rem, k, size(&gs[i]))
            };
            let at = ex(ExprKind::Call {
                callee: Box::new(field(ex(ExprKind::Name(gs[i].clone())), &nth)),
                args: vec![ast::Arg {
                    name: None,
                    mutable: false,
                    value: k,
                }],
            });
            (nm(f), at)
        });
        let parts = ex(ExprKind::Record(parts.collect()));
        let ord_of = |f: &str| field(field(ex(ExprKind::Name("self".to_string())), f), &ord);
        let rank = (fs.iter().zip(&gs).skip(1)).fold(ord_of(&fs[0]), |o, (f, g)| {
            bin(ast::BinOp::Add, bin(ast::BinOp::Mul, o, size(g)), ord_of(f))
        });
        let Some(total) = total else { continue };
        let body = vec![
            method(true, &count, Vec::new(), named("Int"), total),
            method(
                false,
                &nth,
                vec![param("k", Some(named("Int")))],
                record.clone(),
                parts,
            ),
            method(true, &ord, vec![param("self", None)], named("Int"), rank),
        ];
        let imp = ast::Stmt {
            kind: StmtKind::Impl {
                trait_name: Some(named(&tr.text)),
                target: record,
                bounds: (gs.iter())
                    .map(|g| ast::Bound {
                        name: nm(g),
                        traits: vec![named(&tr.text)],
                    })
                    .collect(),
                body: ast::Block { stmts: body, span },
            },
            span,
            decorators: Vec::new(),
            public: false,
        };
        out.push((name(fs), imp));
    }
    out
}
