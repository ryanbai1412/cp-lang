//! Records `(lo: Int, hi: V)` (R02.5.2). Each list of field names, in order, is one generic core
//! struct `(lo:, hi:)<A, B>`, so two record types are equal exactly when their field names, order
//! and types are, and fields, constructors, patterns and derived `Eq` and `Ord` are a struct's.
use cpl_diag::Span;
use cpl_syntax::ast::{self, ExprKind, FnDecl, StmtKind, TypeKind};
use cpl_syntax::visit::{self, Visitor};
use std::collections::BTreeSet;

/// The struct a record with these fields is: `(lo:, hi:)`.
pub fn name<S: AsRef<str>>(fields: &[S]) -> String {
    let fs: Vec<String> = fields.iter().map(|f| format!("{}:", f.as_ref())).collect();
    format!("({})", fs.join(", "))
}

pub fn is_record(name: &str) -> bool {
    name.starts_with('(')
}

/// Every record shape any module writes, as a type or a literal.
pub fn shapes(mods: &[crate::modules::Module]) -> Vec<Vec<String>> {
    let mut v = Shapes(BTreeSet::new());
    for (_, stmts) in mods.iter().flat_map(|m| &m.items) {
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
                decorators: Vec::new(),
                public: true,
            }
        })
        .collect()
}

struct Shapes(BTreeSet<Vec<String>>);

impl Shapes {
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

impl Visitor for Shapes {
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
