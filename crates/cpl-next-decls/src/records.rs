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
    let mut v = Rows(BTreeSet::new());
    for crate::modules::Item { id: _, stmts } in mods.iter().flat_map(|m| m.items.iter()) {
        stmts.iter().for_each(|s| visit::walk_stmt(s, &mut v));
    }
    v.0.into_iter().collect()
}

struct Rows(BTreeSet<Vec<String>>);

impl Visitor for Rows {
    fn stmt(&mut self, s: &ast::Stmt) {
        let bounds = match &s.kind {
            StmtKind::Fn(f) => &f.bounds,
            StmtKind::Struct(sd) => &sd.bounds,
            StmtKind::Impl { bounds, .. } | StmtKind::Trait { bounds, .. } => bounds,
            _ => return,
        };
        for t in bounds.iter().flat_map(|b| &b.traits) {
            if let TypeKind::Row(fs) = &t.kind {
                self.0
                    .insert(fs.iter().map(|(n, _)| n.text.clone()).collect());
            }
        }
    }
}

/// `pub trait (u:, w:, ..)<A, B>: @get fn u(self) -> A; @get fn w(self) -> B` for each row: a
/// trait of `@get` members only, so each record shape with those fields meets it by its fields.
pub fn row_traits(rows: &[Vec<String>], span: Span) -> Vec<ast::Stmt> {
    let nm = |text: String| ast::Name { text, span };
    (rows.iter())
        .filter(|fs| fs.len() <= 26)
        .map(|fs| {
            let generics: Vec<ast::Name> = (b'A'..)
                .take(fs.len())
                .map(|c| nm((c as char).to_string()))
                .collect();
            let getters = (fs.iter().zip(&generics)).map(|(f, g)| {
                let mut get = crate::synth::field_getter(f, span);
                if let StmtKind::Fn(fd) = &mut get.kind {
                    fd.body = ast::FnBody::None;
                    fd.ret = Some(ast::Type {
                        kind: TypeKind::Named {
                            name: g.clone(),
                            args: Vec::new(),
                        },
                        span,
                    });
                }
                get.public = true;
                get
            });
            ast::Stmt {
                kind: StmtKind::Trait {
                    name: nm(row_name(fs)),
                    generic_defaults: vec![None; fs.len()],
                    generics: generics.clone(),
                    supers: Vec::new(),
                    body: ast::Block {
                        stmts: getters.collect(),
                        span,
                    },
                    alias: false,
                    bounds: Vec::new(),
                },
                span,
                decorators: vec![ast::Decorator {
                    directive: false,
                    name: ast::Name {
                        text: RECORD.to_string(),
                        span,
                    },
                    args: Vec::new(),
                    span,
                }],
                public: true,
            }
        })
        .collect()
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
    let mut v = Shapes(BTreeSet::new(), variadics, BTreeSet::new());
    for crate::modules::Item { id: _, stmts } in mods.iter().flat_map(|m| m.items.iter()) {
        stmts.iter().for_each(|s| visit::walk_stmt(s, &mut v));
    }
    v.0.into_iter().collect()
}

/// A `without` form, `(to: _, e: _, ..R without (u, v))`: the fields it adds, then those it drops.
pub type Form = (Vec<String>, Vec<String>);

/// Every `without` form any module writes, in a type or a value.
pub fn forms(mods: &[crate::modules::Module]) -> Vec<Form> {
    let none = HashMap::new();
    let mut v = Shapes(BTreeSet::new(), &none, BTreeSet::new());
    for crate::modules::Item { id: _, stmts } in mods.iter().flat_map(|m| m.items.iter()) {
        stmts.iter().for_each(|s| visit::walk_stmt(s, &mut v));
    }
    v.2.into_iter().collect()
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

fn gen_names(prefix: &str, n: usize) -> Vec<String> {
    (0..n).map(|k| format!("{prefix}{k}")).collect()
}

fn self_param(n: &str, ty: Option<ast::Type>, span: Span) -> ast::Param {
    ast::Param {
        mutable: false,
        variadic: false,
        lazy: false,
        named_only: false,
        label: None,
        name: ast::Name {
            text: n.to_string(),
            span,
        },
        ty,
        default: None,
        span,
    }
}

fn member_fn(
    name: &str,
    params: Vec<ast::Param>,
    ret: ast::Type,
    body: ast::FnBody,
    span: Span,
) -> ast::Stmt {
    ast::Stmt {
        kind: StmtKind::Fn(FnDecl {
            name: ast::Name {
                text: name.to_string(),
                span,
            },
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
        decorators: Vec::new(),
        public: true,
    }
}

/// `pub trait (to:, e:, ..without u, v)<P0, P1>: type R; fn R(self, to: P0, e: P1) -> Self.R`
/// for each form, `R` its `without_member`.
pub fn without_traits(forms: &[Form], span: Span) -> Vec<ast::Stmt> {
    let nm = |text: &str| ast::Name {
        text: text.to_string(),
        span,
    };
    (forms.iter())
        .map(|form| {
            let m = without_member(form);
            let ps = gen_names("P", form.0.len());
            let mut params = vec![self_param("self", None, span)];
            params.extend(
                (form.0.iter().zip(&ps))
                    .map(|(f, p)| self_param(f, Some(ast::Type::named(p, span)), span)),
            );
            let assoc = ast::Stmt {
                kind: StmtKind::TypeAlias {
                    name: nm(&m),
                    generics: Vec::new(),
                    ty: ast::Type {
                        kind: TypeKind::Error,
                        span,
                    },
                    bounds: Vec::new(),
                },
                span,
                decorators: Vec::new(),
                public: true,
            };
            let ret = ast::Type::named(&format!("Self.{m}"), span);
            let build = member_fn(&m, params, ret, ast::FnBody::None, span);
            ast::Stmt {
                kind: StmtKind::Trait {
                    name: nm(&without_name(form)),
                    generic_defaults: vec![None; ps.len()],
                    generics: ps.iter().map(|p| nm(p)).collect(),
                    supers: Vec::new(),
                    body: ast::Block {
                        stmts: vec![assoc, build],
                        span,
                    },
                    alias: false,
                    bounds: Vec::new(),
                },
                span,
                decorators: vec![ast::Decorator {
                    directive: false,
                    name: nm(RECORD),
                    args: Vec::new(),
                    span,
                }],
                public: true,
            }
        })
        .collect()
}

/// Per form and record shape it fits, its impl: `impl<P0, F0, F1> W<P0> for (u: F0, w: F1)`
/// with `type R = (to: P0, w: F1)` and `fn R(self, to: P0) -> .. = (to: to, w: self.w)`.
pub fn without_impls(
    forms: &[Form],
    shapes: &[Vec<String>],
    span: Span,
) -> Vec<(String, ast::Stmt)> {
    let nm = |text: &str| ast::Name {
        text: text.to_string(),
        span,
    };
    let ex = |kind| ast::Expr { kind, span };
    let mut out = Vec::new();
    for form in forms {
        let m = without_member(form);
        let ps = gen_names("P", form.0.len());
        for fs in shapes {
            let Some(made) = without_shape(form, fs) else {
                continue;
            };
            let fts = gen_names("F", fs.len());
            let ty_of = |f: &String| match form.0.iter().position(|p| p == f) {
                Some(k) => ast::Type::named(&ps[k], span),
                None => ast::Type::named(&fts[fs.iter().position(|x| x == f).unwrap_or(0)], span),
            };
            let made_ty = ast::Type {
                kind: TypeKind::Record(made.iter().map(|f| (nm(f), ty_of(f))).collect()),
                span,
            };
            let value = |f: &String| match form.0.contains(f) {
                true => ex(ExprKind::Name(f.clone())),
                false => ex(ExprKind::Field {
                    base: Box::new(ex(ExprKind::Name("self".to_string()))),
                    name: nm(f),
                }),
            };
            let body = ex(ExprKind::Record(
                made.iter().map(|f| (nm(f), value(f))).collect(),
            ));
            let mut params = vec![self_param("self", None, span)];
            params.extend(
                (form.0.iter().zip(&ps))
                    .map(|(f, p)| self_param(f, Some(ast::Type::named(p, span)), span)),
            );
            let assoc = ast::Stmt {
                kind: StmtKind::TypeAlias {
                    name: nm(&m),
                    generics: Vec::new(),
                    ty: made_ty.clone(),
                    bounds: Vec::new(),
                },
                span,
                decorators: Vec::new(),
                public: true,
            };
            let build = member_fn(&m, params, made_ty, ast::FnBody::Expr(body), span);
            let kind = StmtKind::Impl {
                trait_name: Some(ast::Type {
                    kind: TypeKind::Named {
                        name: nm(&without_name(form)),
                        args: ps.iter().map(|p| ast::Type::named(p, span)).collect(),
                    },
                    span,
                }),
                target: ast::Type {
                    kind: TypeKind::Record(
                        (fs.iter().zip(&fts))
                            .map(|(f, t)| (nm(f), ast::Type::named(t, span)))
                            .collect(),
                    ),
                    span,
                },
                bounds: (ps.iter().chain(&fts))
                    .map(|p| ast::Bound {
                        name: nm(p),
                        traits: Vec::new(),
                    })
                    .collect(),
                body: ast::Block {
                    stmts: vec![assoc, build],
                    span,
                },
            };
            let imp = ast::Stmt {
                kind,
                span,
                decorators: Vec::new(),
                public: false,
            };
            out.push((name(fs), imp));
        }
    }
    out
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

struct Shapes<'v>(
    BTreeSet<Vec<String>>,
    &'v HashMap<String, Vec<String>>,
    BTreeSet<Form>,
);

fn texts(ns: &[ast::Name]) -> Vec<String> {
    ns.iter().map(|n| n.text.clone()).collect()
}

impl Shapes<'_> {
    fn ty(&mut self, t: &ast::Type) {
        match &t.kind {
            TypeKind::Record(fs) => {
                self.0
                    .insert(fs.iter().map(|(n, _)| n.text.clone()).collect());
                fs.iter().for_each(|(_, t)| self.ty(t));
            }
            TypeKind::Row(fs) => fs.iter().for_each(|(_, t)| self.ty(t)),
            TypeKind::RowWithout {
                prefix,
                base,
                removed,
            } => {
                let added = prefix.iter().map(|(n, _)| n.text.clone()).collect();
                self.2.insert((added, texts(removed)));
                prefix.iter().for_each(|(_, t)| self.ty(t));
                self.ty(base);
            }
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
            ExprKind::Without {
                prefix, removed, ..
            } => {
                let added = prefix.iter().map(|(n, _)| n.text.clone()).collect();
                self.2.insert((added, texts(removed)));
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
