//! From the front end's syntax trees to a typed [`Program`].
//!
//! The front end's checker is deliberately forgiving: what it can't type is
//! `Unknown` and passes. A back end can't run `Unknown`, so this pass types
//! every expression for real. What it can't type, or what the back ends
//! can't run yet, is an error here, with the same rendering as any other
//! diagnostic, and never a wrong program.

mod call;
mod expr;
mod stmt;

use crate::*;
use cpl_check::Analysis;
use cpl_diag::{codes, Diagnostic, Span};
use cpl_syntax::ast;
use std::collections::HashMap;

/// Lowers a file that the front end reported no errors for.
pub fn lower(analysis: &Analysis) -> Result<Program, Vec<Diagnostic>> {
    let mut lw = Lowerer::new(analysis);
    lw.collect();
    lw.signatures();
    lw.main();
    for id in 0..lw.sigs.len() {
        lw.function(id);
    }
    if !lw.diags.is_empty() {
        lw.diags.sort_by_key(|d| (d.span.start, d.span.end));
        lw.diags.dedup();
        return Err(lw.diags);
    }
    Ok(Program {
        structs: lw.structs.iter().map(|s| s.def.clone()).collect(),
        enums: lw.enums,
        domains: lw.domains,
        globals: lw.globals,
        fns: lw.bodies.into_iter().map(|f| f.expect("lowered")).collect(),
        main: lw.main_body,
        main_locals: lw.main_locals,
        multitest: lw.multitest,
    })
}

#[derive(Clone, Copy, Debug)]
enum Sym {
    Global(GlobalId),
    Fn(FnId),
    Struct(StructId),
    Enum(EnumId),
    Variant(EnumId, usize),
    Domain(DomainId),
    Alias(usize),
}

struct StructInfo<'a> {
    def: StructDef,
    decl: &'a ast::StructDecl,
    offset: u32,
    methods: HashMap<String, FnId>,
}

#[derive(Clone)]
struct SigParam<'a> {
    name: String,
    ty: Ty,
    mutable: bool,
    default: Option<&'a ast::Expr>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Pending,
    InProgress,
    Done,
}

struct FnSig<'a> {
    name: String,
    /// Without `self`.
    params: Vec<SigParam<'a>>,
    /// `None` until the body has been read, for a fn without `->`.
    ret: Option<Ty>,
    /// The struct whose method this is, and whether it takes `self`
    /// (`Some(mutable)`).
    owner: Option<StructId>,
    receiver: Option<bool>,
    decl: &'a ast::FnDecl,
    offset: u32,
    state: State,
}

/// What a block is for, which decides what happens to its last expression.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Tail {
    Discard,
    Return,
    Give,
}

struct DoFrame {
    ty: Option<Ty>,
    /// Loops opened before the block: `break` can't reach them.
    loops: usize,
}

#[derive(Default)]
struct FnCx {
    locals: Vec<Local>,
    scopes: Vec<HashMap<String, Slot>>,
    ret: Option<Ty>,
    /// `self`'s slot and struct inside a method.
    receiver: Option<(Slot, StructId)>,
    loops: Vec<(Option<String>, LoopId)>,
    dos: Vec<DoFrame>,
    is_main: bool,
}

struct Lowerer<'a> {
    analysis: &'a Analysis,
    diags: Vec<Diagnostic>,
    syms: HashMap<String, Sym>,
    structs: Vec<StructInfo<'a>>,
    enums: Vec<EnumDef>,
    domains: Vec<String>,
    aliases: Vec<(&'a ast::Type, u32)>,
    globals: Vec<Global>,
    sigs: Vec<FnSig<'a>>,
    bodies: Vec<Option<Function>>,
    main_body: Block,
    main_locals: Vec<Local>,
    multitest: bool,
    cx: FnCx,
    /// Start of the item being lowered; its spans are relative to it.
    offset: u32,
    next_loop: LoopId,
}

impl<'a> Lowerer<'a> {
    fn new(analysis: &'a Analysis) -> Lowerer<'a> {
        Lowerer {
            analysis,
            diags: Vec::new(),
            syms: HashMap::new(),
            structs: Vec::new(),
            enums: Vec::new(),
            domains: Vec::new(),
            aliases: Vec::new(),
            globals: Vec::new(),
            sigs: Vec::new(),
            bodies: Vec::new(),
            main_body: Block::default(),
            main_locals: Vec::new(),
            multitest: false,
            cx: FnCx::default(),
            offset: 0,
            next_loop: 0,
        }
    }

    // ----- reporting -----

    fn span(&self, span: Span) -> Span {
        span.shift(self.offset)
    }

    fn error(&mut self, code: cpl_diag::Code, span: Span, message: impl Into<String>) {
        let span = self.span(span);
        self.diags.push(Diagnostic::error(code, span, message));
    }

    fn help(&mut self, help: impl Into<String>) {
        if let Some(last) = self.diags.last_mut() {
            last.help = Some(help.into());
        }
    }

    /// Reports something the language has but the back ends don't yet.
    fn unsupported(&mut self, span: Span, what: &str) {
        self.error(
            codes::UNSUPPORTED,
            span,
            format!("`cpl run` and `cpl build` can't handle {what} yet"),
        );
    }

    fn mismatch(&mut self, span: Span, want: &Ty, found: &Ty) {
        let message = format!(
            "expected `{}`, found `{}`",
            self.show(want),
            self.show(found)
        );
        self.error(codes::TYPE_MISMATCH, span, message);
    }

    fn show(&self, ty: &Ty) -> String {
        let structs: Vec<StructDef> = self.structs.iter().map(|s| s.def.clone()).collect();
        crate::show::ty_name(ty, &self.domains, &structs, &self.enums)
    }

    fn err_expr(&self, span: Span) -> Expr {
        Expr {
            kind: ExprKind::Error,
            ty: Ty::Error,
            span: self.span(span),
        }
    }

    // ----- declarations -----

    fn items(&self) -> impl Iterator<Item = (u32, &'a ast::Stmt)> {
        let analysis: &'a Analysis = self.analysis;
        analysis.items.iter().flat_map(|item| {
            let offset = item.offset;
            item.syntax.parsed.stmts.iter().map(move |s| (offset, s))
        })
    }

    /// First pass: every top-level name that is a type or a fn, so order
    /// in the file doesn't matter for them.
    fn collect(&mut self) {
        for (offset, stmt) in self.items() {
            self.offset = offset;
            for decorator in &stmt.decorators {
                match decorator.name.text.as_str() {
                    "multitest" => self.multitest = true,
                    "get" | "brute" | "test" | "solution" | "" => {}
                    other => self.unsupported(decorator.span, &format!("the `@{other}` decorator")),
                }
            }
            match &stmt.kind {
                ast::StmtKind::Struct(decl) => {
                    let id = self.structs.len();
                    self.syms.insert(decl.name.text.clone(), Sym::Struct(id));
                    if !decl.generics.is_empty() {
                        self.unsupported(decl.name.span, "generic structs");
                    }
                    self.structs.push(StructInfo {
                        def: StructDef {
                            name: decl.name.text.clone(),
                            fields: Vec::new(),
                        },
                        decl,
                        offset,
                        methods: HashMap::new(),
                    });
                    for method in &decl.methods {
                        if let ast::StmtKind::Fn(f) = &method.kind {
                            let fn_id = self.declare_fn(f, offset, Some(id));
                            self.structs[id].methods.insert(f.name.text.clone(), fn_id);
                        }
                    }
                }
                ast::StmtKind::Enum(decl) => {
                    let id = self.enums.len();
                    self.syms.insert(decl.name.text.clone(), Sym::Enum(id));
                    if !decl.generics.is_empty() {
                        self.unsupported(decl.name.span, "generic enums");
                    }
                    if !decl.methods.is_empty() {
                        self.unsupported(decl.name.span, "methods on enums");
                    }
                    for (tag, variant) in decl.variants.iter().enumerate() {
                        self.syms
                            .insert(variant.name.text.clone(), Sym::Variant(id, tag));
                    }
                    self.enums.push(EnumDef {
                        name: decl.name.text.clone(),
                        variants: Vec::new(),
                    });
                }
                ast::StmtKind::Domain { name, .. } => {
                    self.syms
                        .insert(name.text.clone(), Sym::Domain(self.domains.len()));
                    self.domains.push(name.text.clone());
                }
                ast::StmtKind::TypeAlias { name, generics, ty } => {
                    if !generics.is_empty() {
                        self.unsupported(name.span, "generic type aliases");
                    }
                    self.syms
                        .insert(name.text.clone(), Sym::Alias(self.aliases.len()));
                    self.aliases.push((ty, offset));
                }
                ast::StmtKind::Fn(decl) => {
                    let id = self.declare_fn(decl, offset, None);
                    self.syms.insert(decl.name.text.clone(), Sym::Fn(id));
                }
                ast::StmtKind::Trait { name, .. } => self.unsupported(name.span, "traits"),
                ast::StmtKind::Impl { target, .. } => {
                    self.unsupported(target.span, "`impl` blocks")
                }
                ast::StmtKind::Use { .. } => {
                    self.unsupported(stmt.span, "`use`: modules are not loaded")
                }
                _ => {}
            }
        }
    }

    fn declare_fn(&mut self, decl: &'a ast::FnDecl, offset: u32, owner: Option<StructId>) -> FnId {
        let name = match owner {
            Some(id) => format!("{}.{}", self.structs[id].def.name, decl.name.text),
            None => decl.name.text.clone(),
        };
        self.sigs.push(FnSig {
            name,
            params: Vec::new(),
            ret: None,
            owner,
            receiver: None,
            decl,
            offset,
            state: State::Pending,
        });
        self.bodies.push(None);
        self.sigs.len() - 1
    }

    /// Second pass: field, payload and parameter types.
    fn signatures(&mut self) {
        for id in 0..self.structs.len() {
            let (decl, offset) = (self.structs[id].decl, self.structs[id].offset);
            self.offset = offset;
            let fields = decl
                .fields
                .iter()
                .map(|f| (f.name.text.clone(), self.resolve_ty(&f.ty)))
                .collect();
            self.structs[id].def.fields = fields;
        }
        let mut enum_id = 0;
        for (offset, stmt) in self.items() {
            self.offset = offset;
            if let ast::StmtKind::Enum(decl) = &stmt.kind {
                let variants = decl
                    .variants
                    .iter()
                    .map(|v| {
                        let fields = v.fields.iter().map(|(_, t)| self.resolve_ty(t)).collect();
                        (v.name.text.clone(), fields)
                    })
                    .collect();
                self.enums[enum_id].variants = variants;
                enum_id += 1;
            }
        }
        for id in 0..self.sigs.len() {
            let decl = self.sigs[id].decl;
            self.offset = self.sigs[id].offset;
            if !decl.generics.is_empty() {
                self.unsupported(decl.name.span, "generic fns");
            }
            let mut params = Vec::new();
            for (i, param) in decl.params.iter().enumerate() {
                if i == 0 && param.name.text == "self" && self.sigs[id].owner.is_some() {
                    self.sigs[id].receiver = Some(param.mutable);
                    continue;
                }
                if param.variadic {
                    self.unsupported(param.span, "variadic parameters");
                }
                let ty = match &param.ty {
                    Some(ty) => self.resolve_ty(ty),
                    None => {
                        self.error(
                            codes::UNKNOWN_TYPE,
                            param.span,
                            format!("parameter `{}` needs a type", param.name.text),
                        );
                        self.help(format!(
                            "write `{}: Int`, or the type it has",
                            param.name.text
                        ));
                        Ty::Error
                    }
                };
                params.push(SigParam {
                    name: param.name.text.clone(),
                    ty,
                    mutable: param.mutable,
                    default: param.default.as_ref(),
                });
            }
            self.sigs[id].params = params;
            self.sigs[id].ret = match (&decl.ret, &decl.body) {
                (
                    Some(ast::Type {
                        kind: ast::TypeKind::Yields(_),
                        span,
                    }),
                    _,
                ) => {
                    self.unsupported(*span, "generators");
                    Some(Ty::Error)
                }
                (Some(ty), _) => Some(self.resolve_ty(ty)),
                (None, ast::FnBody::Block(block)) => match block.stmts.last() {
                    Some(ast::Stmt {
                        kind: ast::StmtKind::Expr(_),
                        ..
                    }) => None,
                    _ => Some(Ty::Unit),
                },
                (None, _) => None,
            };
        }
    }

    // ----- types -----

    fn resolve_ty(&mut self, ty: &ast::Type) -> Ty {
        self.resolve(ty, false).ty()
    }

    /// The type `ty` names. With `values`, also the lengths and ranges
    /// written in it, lowered where the type stands.
    fn resolve(&mut self, ty: &ast::Type, values: bool) -> Spec {
        match &ty.kind {
            ast::TypeKind::Named { name, args } => self.named(name, args, values),
            ast::TypeKind::Array { index, elem } => {
                let dims: Vec<Option<DimSpec>> =
                    index.iter().map(|d| self.dim(d, values)).collect();
                let elem = self.resolve(elem, values);
                match dims.into_iter().collect::<Option<Vec<_>>>() {
                    Some(dims) => Spec::Array {
                        dims,
                        elem: Box::new(elem),
                    },
                    None => Spec::Plain(Ty::Error),
                }
            }
            ast::TypeKind::Optional(inner) => Spec::Opt(Box::new(self.resolve(inner, values))),
            ast::TypeKind::Tuple(items) if items.is_empty() => Spec::Plain(Ty::Unit),
            ast::TypeKind::Tuple(items) => {
                let specs: Vec<Spec> = items.iter().map(|t| self.resolve(t, values)).collect();
                Spec::Fields(Ty::Tuple(specs.iter().map(Spec::ty).collect()), specs)
            }
            ast::TypeKind::Record(fields) => {
                let specs: Vec<Spec> = fields
                    .iter()
                    .map(|(_, t)| self.resolve(t, values))
                    .collect();
                let ty = Ty::Record(
                    fields
                        .iter()
                        .zip(&specs)
                        .map(|((name, _), spec)| (name.text.clone(), spec.ty()))
                        .collect(),
                );
                Spec::Fields(ty, specs)
            }
            ast::TypeKind::Range { lo, hi, inclusive } => {
                if !values {
                    return Spec::Plain(Ty::Int);
                }
                let lo = self.expect(lo, &Ty::Int);
                let mut hi = self.expect(hi, &Ty::Int);
                if !inclusive {
                    hi = self.int_add(hi, -1);
                }
                Spec::Int {
                    lo: Some(Box::new(lo)),
                    hi: Some(Box::new(hi)),
                    text: self.source(ty.span),
                }
            }
            ast::TypeKind::OwnDomain { name, .. } => {
                self.unsupported(name.span, "arrays that declare their own domain");
                Spec::Plain(Ty::Error)
            }
            ast::TypeKind::Fn { .. } => {
                self.unsupported(ty.span, "fn types");
                Spec::Plain(Ty::Error)
            }
            ast::TypeKind::Value(_) => {
                self.unsupported(ty.span, "values as type arguments");
                Spec::Plain(Ty::Error)
            }
            ast::TypeKind::NewDomain => {
                self.unsupported(ty.span, "fns that return a fresh domain");
                Spec::Plain(Ty::Error)
            }
            ast::TypeKind::Yields(_) => {
                self.unsupported(ty.span, "generators");
                Spec::Plain(Ty::Error)
            }
            ast::TypeKind::Error => Spec::Plain(Ty::Error),
        }
    }

    fn named(&mut self, name: &ast::Name, args: &[ast::Type], values: bool) -> Spec {
        let arg = |lw: &mut Self, i: usize| match args.get(i) {
            Some(ty) => lw.resolve_ty(ty),
            None => {
                lw.error(
                    codes::UNKNOWN_TYPE,
                    name.span,
                    format!(
                        "`{}` needs its element type: `{}<Int>`",
                        name.text, name.text
                    ),
                );
                Ty::Error
            }
        };
        let ty = match name.text.as_str() {
            "Int" | "I64" => Ty::Int,
            "Float" | "F64" => Ty::Float,
            "Bool" => Ty::Bool,
            "Char" => Ty::Char,
            "Str" => Ty::Str,
            "List" => Ty::list(arg(self, 0)),
            "Set" => Ty::Set(Box::new(arg(self, 0))),
            "Map" => {
                let key = arg(self, 0);
                Ty::Map(Box::new(key), Box::new(arg(self, 1)))
            }
            "I32" | "U8" | "U32" | "U64" | "I128" => {
                self.unsupported(name.span, &format!("the fixed-width type `{}`", name.text));
                Ty::Error
            }
            _ => match self.syms.get(&name.text).copied() {
                Some(Sym::Struct(id)) if values => return self.struct_spec(id),
                Some(Sym::Struct(id)) => Ty::Struct(id),
                Some(Sym::Enum(id)) => Ty::Enum(id),
                Some(Sym::Domain(id)) => Ty::Pos(id),
                Some(Sym::Alias(id)) => {
                    let (ty, offset) = self.aliases[id];
                    let saved = std::mem::replace(&mut self.offset, offset);
                    let spec = self.resolve(ty, values);
                    self.offset = saved;
                    return spec;
                }
                _ => {
                    self.error(
                        codes::UNKNOWN_TYPE,
                        name.span,
                        format!("`{}` is not a type `cpl run` knows", name.text),
                    );
                    Ty::Error
                }
            },
        };
        Spec::Plain(ty)
    }

    /// A struct's fields as they are read from input: with their ranges.
    fn struct_spec(&mut self, id: StructId) -> Spec {
        let (decl, offset) = (self.structs[id].decl, self.structs[id].offset);
        let saved = std::mem::replace(&mut self.offset, offset);
        let fields = decl
            .fields
            .iter()
            .map(|f| self.resolve(&f.ty, true))
            .collect();
        self.offset = saved;
        Spec::Fields(Ty::Struct(id), fields)
    }

    /// One dimension of an array type: a domain, its bounds, or a count.
    fn dim(&mut self, ty: &ast::Type, values: bool) -> Option<DimSpec> {
        if let ast::TypeKind::Named { name, args } = &ty.kind {
            if args.is_empty() {
                if let Some(Sym::Domain(id)) = self.syms.get(&name.text).copied() {
                    return Some(DimSpec::Pos(id));
                }
                // `[V.bounds]Int` reaches here as the name `V.bounds`
                // when the parser reads a dotted type name.
                if let Some(domain) = name.text.strip_suffix(".bounds") {
                    if let Some(Sym::Domain(id)) = self.syms.get(domain).copied() {
                        return Some(DimSpec::Bound(id));
                    }
                }
            }
        }
        // Anything else is a length: `[n]Int`, `[2 * n]Int`, `[0..n]Int`.
        let text = self.source(ty.span);
        if let Some(domain) = text.strip_suffix(".bounds") {
            if let Some(Sym::Domain(id)) = self.syms.get(domain.trim()).copied() {
                return Some(DimSpec::Bound(id));
            }
        }
        match &ty.kind {
            ast::TypeKind::Range { lo, hi, inclusive } => {
                if !matches!(lo.kind, ast::ExprKind::Int(0)) {
                    self.unsupported(lo.span, "array dimensions that don't start at 0");
                    return None;
                }
                if !values {
                    return Some(DimSpec::Int(Box::new(self.int_lit(0, ty.span))));
                }
                let mut len = self.expect(hi, &Ty::Int);
                if *inclusive {
                    len = self.int_add(len, 1);
                }
                Some(DimSpec::Int(Box::new(len)))
            }
            ast::TypeKind::Named { name, args } if args.is_empty() => {
                if !values {
                    return Some(DimSpec::Int(Box::new(self.int_lit(0, ty.span))));
                }
                let expr = ast::Expr {
                    kind: ast::ExprKind::Name(name.text.clone()),
                    span: name.span,
                };
                Some(DimSpec::Int(Box::new(self.expect(&expr, &Ty::Int))))
            }
            _ => {
                self.error(
                    codes::UNKNOWN_TYPE,
                    ty.span,
                    "an array is indexed by a domain or a length: `[V]Int`, `[n]Int`",
                );
                None
            }
        }
    }

    /// The source text of a span of the current item.
    fn source(&self, span: Span) -> String {
        let span = self.span(span);
        self.analysis
            .text
            .get(span.start as usize..span.end as usize)
            .unwrap_or("")
            .to_string()
    }

    fn line(&self, span: Span) -> u32 {
        self.analysis.lines.line_of(self.span(span).start) + 1
    }

    // ----- scopes -----

    fn new_slot(&mut self, name: &str, ty: Ty) -> Slot {
        self.cx.locals.push(Local {
            name: name.to_string(),
            ty,
        });
        self.cx.locals.len() - 1
    }

    /// A local the user didn't name.
    fn temp(&mut self, ty: Ty) -> Slot {
        self.new_slot("", ty)
    }

    fn local(&self, slot: Slot, span: Span) -> Expr {
        Expr {
            kind: ExprKind::Local(slot),
            ty: self.cx.locals[slot].ty.clone(),
            span: self.span(span),
        }
    }

    fn lookup(&self, name: &str) -> Option<Slot> {
        self.cx
            .scopes
            .iter()
            .rev()
            .find_map(|s| s.get(name).copied())
    }

    /// Declares `name` with the value `init`: a global at the top level of
    /// the file, a local anywhere else.
    fn declare(&mut self, name: &str, init: Expr, out: &mut Vec<Stmt>) {
        if self.cx.is_main && self.cx.scopes.len() == 1 {
            let id = self.globals.len();
            self.globals.push(Global {
                name: name.to_string(),
                ty: init.ty.clone(),
            });
            self.syms.insert(name.to_string(), Sym::Global(id));
            let place = Place {
                kind: PlaceKind::Global(id),
                ty: init.ty.clone(),
                span: init.span,
            };
            out.push(Stmt::Assign(place, AssignOp::Set, init));
        } else {
            let slot = self.new_slot(name, init.ty.clone());
            self.cx
                .scopes
                .last_mut()
                .expect("a scope")
                .insert(name.to_string(), slot);
            out.push(Stmt::Let(slot, Some(init)));
        }
    }

    // ----- fns -----

    fn main(&mut self) {
        self.cx = FnCx {
            is_main: true,
            ret: Some(Ty::Unit),
            scopes: vec![HashMap::new()],
            ..FnCx::default()
        };
        let mut out = Vec::new();
        for (offset, stmt) in self.items() {
            self.offset = offset;
            self.stmt(stmt, &mut out);
        }
        let cx = std::mem::take(&mut self.cx);
        self.main_body = Block { stmts: out };
        self.main_locals = cx.locals;
    }

    /// Lowers the body of fn `id`, once.
    fn function(&mut self, id: FnId) {
        if self.sigs[id].state != State::Pending {
            return;
        }
        self.sigs[id].state = State::InProgress;
        let decl = self.sigs[id].decl;
        let saved_offset = std::mem::replace(&mut self.offset, self.sigs[id].offset);
        let mut cx = FnCx {
            ret: self.sigs[id].ret.clone(),
            scopes: vec![HashMap::new()],
            ..FnCx::default()
        };
        let mut params = Vec::new();
        if let (Some(mutable), Some(owner)) = (self.sigs[id].receiver, self.sigs[id].owner) {
            cx.locals.push(Local {
                name: "self".into(),
                ty: Ty::Struct(owner),
            });
            cx.scopes[0].insert("self".into(), 0);
            cx.receiver = Some((0, owner));
            params.push(ParamDef { mutable });
        }
        for param in &self.sigs[id].params {
            cx.scopes[0].insert(param.name.clone(), cx.locals.len());
            cx.locals.push(Local {
                name: param.name.clone(),
                ty: param.ty.clone(),
            });
            params.push(ParamDef {
                mutable: param.mutable,
            });
        }
        let saved_cx = std::mem::replace(&mut self.cx, cx);
        let mut out = Vec::new();
        match &decl.body {
            ast::FnBody::Block(block) => {
                let body = self.block(block, Tail::Return);
                out = body.stmts;
            }
            ast::FnBody::Expr(expr) => self.tail(expr, Tail::Return, &mut out),
            ast::FnBody::None => self.unsupported(decl.name.span, "a fn without a body"),
        }
        let ret = self.cx.ret.clone().unwrap_or(Ty::Unit);
        if ret != Ty::Unit && !matches!(out.last(), Some(Stmt::Return(_))) {
            out.push(Stmt::Trap(
                format!("fn `{}` ended without a value", decl.name.text),
                self.span(decl.name.span),
            ));
        }
        let cx = std::mem::replace(&mut self.cx, saved_cx);
        self.offset = saved_offset;
        self.sigs[id].ret = Some(ret.clone());
        self.sigs[id].state = State::Done;
        self.bodies[id] = Some(Function {
            name: self.sigs[id].name.clone(),
            params,
            ret,
            locals: cx.locals,
            body: Block { stmts: out },
        });
    }

    /// The return type of fn `id`, reading its body now if it has no `->`.
    fn ret_of(&mut self, id: FnId, span: Span) -> Ty {
        if let Some(ty) = &self.sigs[id].ret {
            return ty.clone();
        }
        if self.sigs[id].state == State::InProgress {
            let name = self.sigs[id].decl.name.text.clone();
            self.error(
                codes::UNKNOWN_TYPE,
                span,
                format!("`{name}` calls itself, so it needs its return type written"),
            );
            self.help(format!(
                "add `-> Int`, or the type it returns, to `fn {name}`"
            ));
            return Ty::Error;
        }
        self.function(id);
        self.sigs[id].ret.clone().unwrap_or(Ty::Error)
    }

    // ----- small builders -----

    fn int_lit(&self, value: i64, span: Span) -> Expr {
        Expr {
            kind: ExprKind::Int(value),
            ty: Ty::Int,
            span: self.span(span),
        }
    }

    /// `e + by`, folded when `e` is a literal.
    fn int_add(&self, e: Expr, by: i64) -> Expr {
        if let ExprKind::Int(v) = e.kind {
            if let Some(sum) = v.checked_add(by) {
                return Expr {
                    kind: ExprKind::Int(sum),
                    ..e
                };
            }
        }
        let span = e.span;
        let rhs = Expr {
            kind: ExprKind::Int(by),
            ty: Ty::Int,
            span,
        };
        Expr {
            kind: ExprKind::Binary(BinOp::Add, Box::new(e), Box::new(rhs)),
            ty: Ty::Int,
            span,
        }
    }

    fn mk(&self, kind: ExprKind, ty: Ty, span: Span) -> Expr {
        Expr {
            kind,
            ty,
            span: self.span(span),
        }
    }

    /// `e` as a `want`, converting where the language does it silently:
    /// an `Int` where a `Float` is wanted, a value where an optional is.
    fn coerce(&mut self, e: Expr, want: &Ty, span: Span) -> Expr {
        match self.try_coerce(e, want) {
            Ok(e) => e,
            Err(e) => {
                self.mismatch(span, want, &e.ty);
                self.err_expr(span)
            }
        }
    }

    fn try_coerce(&mut self, e: Expr, want: &Ty) -> Result<Expr, Expr> {
        if e.ty == *want || e.ty == Ty::Error || *want == Ty::Error {
            return Ok(e);
        }
        let span = e.span;
        match (want, &e.ty) {
            (Ty::Float, Ty::Int) => Ok(match e.kind {
                ExprKind::Int(v) => Expr {
                    kind: ExprKind::Float(v as f64),
                    ty: Ty::Float,
                    span,
                },
                _ => Expr {
                    kind: ExprKind::Cast(Cast::IntToFloat, Box::new(e)),
                    ty: Ty::Float,
                    span,
                },
            }),
            (Ty::Opt(inner), _) if !matches!(e.kind, ExprKind::None) => {
                let inner = self.try_coerce(e, inner)?;
                Ok(Expr {
                    kind: ExprKind::Some(Box::new(inner)),
                    ty: want.clone(),
                    span,
                })
            }
            (Ty::Record(fields), Ty::Tuple(items))
                if fields.len() == items.len()
                    && fields.iter().zip(items).all(|((_, f), i)| f == i) =>
            {
                Ok(Expr {
                    ty: want.clone(),
                    ..e
                })
            }
            _ => Err(e),
        }
    }
}
