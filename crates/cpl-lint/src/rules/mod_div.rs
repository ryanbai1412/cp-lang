//! `loop_invariant_mod_div`: dividing a `Mod<P>` by a loop-invariant value
//! inside a loop computes an O(log P) modular inverse on every iteration.
//!
//! Hoisting `let inv_d = d.inv` before the loop replaces each O(log P)
//! division with an O(1) multiplication.

use crate::walk::unparen;
use crate::{File, Finding, LOOP_INVARIANT_MOD_DIV};
use cpl_diag::{Diagnostic, Fix, Span};
use cpl_syntax::ast::{
    AssignOp, BinOp, Block, Cond, Expr, ExprKind, FnBody, Pattern, PatternKind, Stmt, StmtKind,
    Type, TypeKind, UnOp,
};
use std::collections::{HashMap, HashSet};

/// Tracks `Mod<P>` type aliases and bindings within a lexical scope.
#[derive(Clone, Default)]
struct Mods {
    mod_types: HashSet<String>,
    names: HashMap<String, String>,
    seqs: HashMap<String, String>,
    fns: HashMap<String, String>,
    fields: HashMap<String, String>,
}

impl Mods {
    fn new() -> Self {
        let mut m = Self::default();
        m.mod_types.insert("Mod".to_string());
        m
    }

    fn mod_type_name(&self, ty: &Type, src: &str, base: u32) -> Option<String> {
        match &ty.kind {
            TypeKind::Named { name, args } if self.mod_types.contains(&name.text) => {
                if args.is_empty() {
                    Some(name.text.clone())
                } else {
                    let s = ty.span.shift(base);
                    let text = &src[s.start as usize..s.end as usize];
                    if text.contains('<') && !text.ends_with('>') {
                        Some(format!("{text}>"))
                    } else {
                        Some(text.to_string())
                    }
                }
            }
            _ => None,
        }
    }

    fn mod_seq_elem_type(&self, ty: &Type, src: &str, base: u32) -> Option<String> {
        match &ty.kind {
            TypeKind::Named { name, args } if name.text == "List" && args.len() == 1 => {
                self.mod_type_name(&args[0], src, base)
            }
            TypeKind::Array { elem, .. } => self.mod_type_name(elem, src, base),
            _ => None,
        }
    }

    fn mod_ty_of(&self, expr: &Expr, src: &str, base: u32) -> Option<String> {
        match &unparen(expr).kind {
            ExprKind::Name(name) => self.names.get(name).cloned(),
            ExprKind::Index { base: arr, .. } => match &unparen(arr).kind {
                ExprKind::Name(name) => self.seqs.get(name).cloned(),
                _ => None,
            },
            ExprKind::Unwrap(inner)
            | ExprKind::Unary {
                op: UnOp::Neg,
                expr: inner,
            } => self.mod_ty_of(inner, src, base),
            ExprKind::Binary {
                op: BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div,
                lhs,
                rhs,
                ..
            } => self
                .mod_ty_of(lhs, src, base)
                .or_else(|| self.mod_ty_of(rhs, src, base)),
            ExprKind::Field { base: recv, name } => {
                if matches!(name.text.as_str(), "inv" | "zero" | "one") {
                    if let ExprKind::Name(t) = &unparen(recv).kind {
                        if self.mod_types.contains(t) {
                            return Some(t.clone());
                        }
                    }
                    return self.mod_ty_of(recv, src, base);
                }
                self.fields.get(&name.text).cloned()
            }
            ExprKind::Call { callee, .. } => match &unparen(callee).kind {
                ExprKind::Name(name) if self.mod_types.contains(name) => Some(name.clone()),
                ExprKind::Name(name) => self.fns.get(name).cloned(),
                ExprKind::Generic { base: g, .. } => {
                    if let ExprKind::Name(name) = &unparen(g).kind {
                        if self.mod_types.contains(name) {
                            let s = callee.span.shift(base);
                            let text = &src[s.start as usize..s.end as usize];
                            if text.contains('<') && !text.ends_with('>') {
                                return Some(format!("{text}>"));
                            }
                            return Some(text.to_string());
                        }
                    }
                    None
                }
                ExprKind::Field { base: recv, name }
                    if matches!(name.text.as_str(), "new" | "pow" | "inv") =>
                {
                    if let ExprKind::Name(t) = &unparen(recv).kind {
                        if self.mod_types.contains(t) {
                            return Some(t.clone());
                        }
                    }
                    self.mod_ty_of(recv, src, base)
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn mod_seq_of(&self, expr: &Expr, src: &str, base: u32) -> Option<String> {
        match &unparen(expr).kind {
            ExprKind::List(items) => items.first().and_then(|e| self.mod_ty_of(e, src, base)),
            ExprKind::Comprehension { body, .. } => self.mod_ty_of(body, src, base),
            ExprKind::Call { callee, args } => {
                if let ExprKind::Field { base: recv, name } = &unparen(callee).kind {
                    if matches!(&unparen(recv).kind, ExprKind::Name(n) if n == "List")
                        && name.text == "filled"
                        && args.len() == 2
                    {
                        return self.mod_ty_of(&args[1].value, src, base);
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn record_stmt_binding(&mut self, stmt: &Stmt, src: &str, base: u32) {
        match &stmt.kind {
            StmtKind::TypeAlias { name, ty, .. } => {
                if self.mod_type_name(ty, src, base).is_some() {
                    self.mod_types.insert(name.text.clone());
                }
            }
            StmtKind::Let { pat, ty, init, .. } => {
                if let PatternKind::Name(name) = &pat.kind {
                    let mty = ty
                        .as_ref()
                        .and_then(|t| self.mod_type_name(t, src, base))
                        .or_else(|| {
                            ty.is_none()
                                .then(|| init.as_ref().and_then(|e| self.mod_ty_of(e, src, base)))
                                .flatten()
                        });
                    let seq = ty
                        .as_ref()
                        .and_then(|t| self.mod_seq_elem_type(t, src, base))
                        .or_else(|| {
                            ty.is_none()
                                .then(|| init.as_ref().and_then(|e| self.mod_seq_of(e, src, base)))
                                .flatten()
                        });
                    match mty {
                        Some(m) => {
                            self.names.insert(name.clone(), m);
                            self.seqs.remove(name);
                        }
                        None => {
                            self.names.remove(name);
                            match seq {
                                Some(s) => {
                                    self.seqs.insert(name.clone(), s);
                                }
                                None => {
                                    self.seqs.remove(name);
                                }
                            }
                        }
                    }
                }
            }
            StmtKind::Const { name, ty, init, .. } => {
                let mty = ty
                    .as_ref()
                    .and_then(|t| self.mod_type_name(t, src, base))
                    .or_else(|| init.as_ref().and_then(|e| self.mod_ty_of(e, src, base)));
                if let Some(m) = mty {
                    self.names.insert(name.text.clone(), m);
                }
            }
            _ => {}
        }
    }
}

fn collect_all_names_block(block: &Block, out: &mut HashSet<String>) {
    for stmt in &block.stmts {
        collect_all_names_stmt(stmt, out);
    }
}

fn collect_all_names_stmt(stmt: &Stmt, out: &mut HashSet<String>) {
    match &stmt.kind {
        StmtKind::Let { pat, .. } => collect_pat_names(pat, out),
        StmtKind::Const { name, .. } | StmtKind::TypeAlias { name, .. } => {
            out.insert(name.text.clone());
        }
        StmtKind::Fn(decl) => {
            out.insert(decl.name.text.clone());
            for p in &decl.params {
                out.insert(p.name.text.clone());
            }
            if let FnBody::Block(b) = &decl.body {
                collect_all_names_block(b, out);
            }
        }
        StmtKind::For { pat, body, .. } => {
            collect_pat_names(pat, out);
            collect_all_names_block(body, out);
        }
        StmtKind::While { body, .. } | StmtKind::Unchecked(body) => {
            collect_all_names_block(body, out);
        }
        StmtKind::Impl { body, .. } | StmtKind::Trait { body, .. } => {
            collect_all_names_block(body, out);
        }
        _ => {}
    }
}

fn collect_loop_writes(
    pat: Option<&Pattern>,
    cond: Option<&Cond>,
    body: &Block,
) -> HashSet<String> {
    let mut out = HashSet::new();
    if let Some(p) = pat {
        collect_pat_names(p, &mut out);
    }
    if let Some(c) = cond {
        collect_cond_writes(c, &mut out);
    }
    collect_block_writes(body, &mut out);
    out
}

fn collect_pat_names(pat: &Pattern, out: &mut HashSet<String>) {
    match &pat.kind {
        PatternKind::Name(n) => {
            out.insert(n.clone());
        }
        PatternKind::Tuple(ps) => {
            for p in ps {
                collect_pat_names(p, out);
            }
        }
        PatternKind::Variant { args, .. } => {
            for p in args {
                collect_pat_names(p, out);
            }
        }
        PatternKind::Domain(n) => {
            out.insert(n.text.clone());
        }
        _ => {}
    }
}

fn collect_target_names(expr: &Expr, out: &mut HashSet<String>) {
    match &unparen(expr).kind {
        ExprKind::Name(n) => {
            out.insert(n.clone());
        }
        ExprKind::Field { base, .. } | ExprKind::Index { base, .. } => {
            collect_target_names(base, out);
        }
        ExprKind::Tuple(es) => {
            for e in es {
                collect_target_names(e, out);
            }
        }
        _ => {}
    }
}

fn collect_cond_writes(cond: &Cond, out: &mut HashSet<String>) {
    match cond {
        Cond::Expr(e) => collect_expr_writes(e, out),
        Cond::Let { pat, value, and } => {
            collect_pat_names(pat, out);
            collect_expr_writes(value, out);
            if let Some(rest) = and {
                collect_cond_writes(rest, out);
            }
        }
        Cond::Guard { test, and } => {
            collect_expr_writes(test, out);
            collect_cond_writes(and, out);
        }
        Cond::Const(e) => collect_expr_writes(e, out),
    }
}

fn collect_block_writes(block: &Block, out: &mut HashSet<String>) {
    for stmt in &block.stmts {
        collect_stmt_writes(stmt, out);
    }
}

fn collect_stmt_writes(stmt: &Stmt, out: &mut HashSet<String>) {
    match &stmt.kind {
        StmtKind::Let {
            pat,
            init,
            otherwise,
            ..
        } => {
            collect_pat_names(pat, out);
            if let Some(e) = init {
                collect_expr_writes(e, out);
            }
            if let Some(b) = otherwise {
                collect_block_writes(b, out);
            }
        }
        StmtKind::Assign { target, value, .. } => {
            collect_target_names(target, out);
            collect_expr_writes(target, out);
            collect_expr_writes(value, out);
        }
        StmtKind::For {
            pat, iter, body, ..
        } => {
            collect_pat_names(pat, out);
            collect_expr_writes(iter, out);
            collect_block_writes(body, out);
        }
        StmtKind::While {
            cond,
            body,
            else_body,
        } => {
            collect_cond_writes(cond, out);
            collect_block_writes(body, out);
            if let Some(eb) = else_body {
                collect_block_writes(eb, out);
            }
        }
        StmtKind::Expr(e) | StmtKind::Return(Some(e)) => collect_expr_writes(e, out),
        StmtKind::Unchecked(b) => collect_block_writes(b, out),
        StmtKind::Fn(decl) => {
            out.insert(decl.name.text.clone());
            match &decl.body {
                FnBody::Block(b) => collect_block_writes(b, out),
                FnBody::Expr(e) => collect_expr_writes(e, out),
                FnBody::None => {}
            }
        }
        _ => {}
    }
}

fn collect_expr_writes(expr: &Expr, out: &mut HashSet<String>) {
    match &expr.kind {
        ExprKind::Call { callee, args } => {
            collect_expr_writes(callee, out);
            for arg in args {
                if arg.mutable {
                    collect_target_names(&arg.value, out);
                }
                collect_expr_writes(&arg.value, out);
            }
        }
        ExprKind::Binary { lhs, rhs, .. } => {
            collect_expr_writes(lhs, out);
            collect_expr_writes(rhs, out);
        }
        ExprKind::Unary { expr: e, .. }
        | ExprKind::Paren(e)
        | ExprKind::Unwrap(e)
        | ExprKind::Field { base: e, .. } => collect_expr_writes(e, out),
        ExprKind::Index { base, indices } => {
            collect_expr_writes(base, out);
            for idx in indices {
                collect_expr_writes(idx, out);
            }
        }
        ExprKind::If {
            branches,
            otherwise,
        } => {
            for (c, b) in branches {
                collect_cond_writes(c, out);
                collect_block_writes(b, out);
            }
            if let Some(b) = otherwise {
                collect_block_writes(b, out);
            }
        }
        ExprKind::Match { scrutinee, arms } => {
            collect_expr_writes(scrutinee, out);
            for arm in arms {
                collect_pat_names(&arm.pat, out);
                if let Some(g) = &arm.guard {
                    collect_expr_writes(g, out);
                }
                collect_block_writes(&arm.body, out);
            }
        }
        ExprKind::Block(b) => collect_block_writes(b, out),
        ExprKind::Tuple(es) | ExprKind::List(es) => {
            for e in es {
                collect_expr_writes(e, out);
            }
        }
        _ => {}
    }
}

fn is_pure_divisor(expr: &Expr, mods: &Mods, top: bool) -> bool {
    match &unparen(expr).kind {
        ExprKind::Int(v) => !top || (*v != 0 && *v != 1),
        ExprKind::Name(_) => true,
        ExprKind::Unary {
            op: UnOp::Neg,
            expr: inner,
        } => is_pure_divisor(inner, mods, false),
        ExprKind::Binary {
            op: BinOp::Add | BinOp::Sub | BinOp::Mul,
            lhs,
            rhs,
            ..
        } => is_pure_divisor(lhs, mods, false) && is_pure_divisor(rhs, mods, false),
        ExprKind::Field { base, .. } => is_pure_divisor(base, mods, false),
        ExprKind::Call { callee, args } => {
            let is_mod_ctor = match &unparen(callee).kind {
                ExprKind::Name(n) => mods.mod_types.contains(n),
                ExprKind::Generic { base, .. } => {
                    matches!(&unparen(base).kind, ExprKind::Name(n) if mods.mod_types.contains(n))
                }
                _ => false,
            };
            if !is_mod_ctor || args.len() != 1 || args[0].mutable || args[0].name.is_some() {
                return false;
            }
            if top && matches!(unparen(&args[0].value).kind, ExprKind::Int(0 | 1)) {
                return false;
            }
            is_pure_divisor(&args[0].value, mods, false)
        }
        _ => false,
    }
}

fn is_nonzero_literal(expr: &Expr, mods: &Mods) -> bool {
    match &unparen(expr).kind {
        ExprKind::Int(v) => *v != 0,
        ExprKind::Call { callee, args } if args.len() == 1 => {
            let is_mod_ctor = match &unparen(callee).kind {
                ExprKind::Name(n) => mods.mod_types.contains(n),
                ExprKind::Generic { base, .. } => {
                    matches!(&unparen(base).kind, ExprKind::Name(n) if mods.mod_types.contains(n))
                }
                _ => false,
            };
            is_mod_ctor && matches!(unparen(&args[0].value).kind, ExprKind::Int(v) if v != 0)
        }
        _ => false,
    }
}

fn collect_expr_names(expr: &Expr, out: &mut HashSet<String>) {
    match &unparen(expr).kind {
        ExprKind::Name(n) => {
            out.insert(n.clone());
        }
        ExprKind::Unary { expr: e, .. } | ExprKind::Field { base: e, .. } => {
            collect_expr_names(e, out);
        }
        ExprKind::Binary { lhs, rhs, .. } => {
            collect_expr_names(lhs, out);
            collect_expr_names(rhs, out);
        }
        ExprKind::Call { args, .. } => {
            for a in args {
                collect_expr_names(&a.value, out);
            }
        }
        _ => {}
    }
}

struct LoopFrame {
    stmt_span: Span,
    writes: HashSet<String>,
}

struct Hit {
    diag_span: Span,
    hoist_span: Span,
    replace_span: Span,
    is_assign: bool,
    rhs_text: String,
    inv_expr: String,
    base_name: String,
    fixable: bool,
}

struct Scanner<'a> {
    src: &'a str,
    base: u32,
    mods: Mods,
    loops: Vec<LoopFrame>,
    branch_depth: usize,
    hits: Vec<Hit>,
}

impl<'a> Scanner<'a> {
    fn slice(&self, span: Span) -> &'a str {
        let s = span.shift(self.base);
        &self.src[s.start as usize..s.end as usize]
    }

    fn scan_block(&mut self, block: &Block) {
        let saved = self.mods.clone();
        for stmt in &block.stmts {
            self.scan_stmt(stmt);
        }
        self.mods = saved;
    }

    fn scan_stmt(&mut self, stmt: &Stmt) {
        if stmt.decorators.iter().any(|d| d.name.text == "test") {
            return;
        }
        match &stmt.kind {
            StmtKind::TypeAlias { .. } | StmtKind::Const { .. } => {
                self.mods.record_stmt_binding(stmt, self.src, self.base);
            }
            StmtKind::Let {
                init,
                otherwise,
                guard,
                ..
            } => {
                if let Some(e) = init {
                    self.scan_expr(e);
                }
                if let Some(b) = otherwise {
                    self.branch_depth += 1;
                    self.scan_block(b);
                    self.branch_depth -= 1;
                }
                if let Some(g) = guard {
                    self.scan_expr(g);
                }
                self.mods.record_stmt_binding(stmt, self.src, self.base);
            }
            StmtKind::For {
                pat, iter, body, ..
            } => {
                self.scan_expr(iter);
                let writes = collect_loop_writes(Some(pat), None, body);
                self.loops.push(LoopFrame {
                    stmt_span: stmt.span.shift(self.base),
                    writes,
                });
                let saved = self.mods.clone();
                if let PatternKind::Name(n) = &pat.kind {
                    if let Some(elem) =
                        self.mods
                            .mod_seq_of(iter, self.src, self.base)
                            .or_else(|| match &unparen(iter).kind {
                                ExprKind::Name(arr) => self.mods.seqs.get(arr).cloned(),
                                _ => None,
                            })
                    {
                        self.mods.names.insert(n.clone(), elem);
                    } else {
                        self.mods.names.remove(n);
                        self.mods.seqs.remove(n);
                    }
                }
                for s in &body.stmts {
                    self.scan_stmt(s);
                }
                self.mods = saved;
                self.loops.pop();
            }
            StmtKind::While { cond, body, .. } => {
                self.branch_depth += 1;
                self.scan_cond(cond);
                self.branch_depth -= 1;
                let writes = collect_loop_writes(None, Some(cond), body);
                self.loops.push(LoopFrame {
                    stmt_span: stmt.span.shift(self.base),
                    writes,
                });
                self.scan_block(body);
                self.loops.pop();
            }
            StmtKind::Assign {
                target,
                op: AssignOp::Binary(BinOp::Div),
                value,
            } => {
                self.scan_expr(target);
                self.scan_expr(value);
                self.check_div(stmt.span, target, value, true);
            }
            StmtKind::Assign { target, value, .. } => {
                self.scan_expr(target);
                self.scan_expr(value);
            }
            StmtKind::Expr(e) | StmtKind::Return(Some(e)) => self.scan_expr(e),
            StmtKind::Assert { cond, message } => {
                self.scan_expr(cond);
                if let Some(m) = message {
                    self.scan_expr(m);
                }
            }
            StmtKind::Yield { value, .. } => self.scan_expr(value),
            StmtKind::Unchecked(b) => self.scan_block(b),
            StmtKind::Fn(decl) => {
                let saved_loops = std::mem::take(&mut self.loops);
                let saved_br = self.branch_depth;
                let saved_mods = self.mods.clone();
                self.branch_depth = 0;
                for param in &decl.params {
                    self.mods.names.remove(&param.name.text);
                    self.mods.seqs.remove(&param.name.text);
                    if let Some(ty) = &param.ty {
                        if let Some(m) = self.mods.mod_type_name(ty, self.src, self.base) {
                            self.mods.names.insert(param.name.text.clone(), m);
                        } else if let Some(m) = self.mods.mod_seq_elem_type(ty, self.src, self.base)
                        {
                            self.mods.seqs.insert(param.name.text.clone(), m);
                        }
                    }
                }
                match &decl.body {
                    FnBody::Block(b) => {
                        for s in &b.stmts {
                            self.scan_stmt(s);
                        }
                    }
                    FnBody::Expr(e) => self.scan_expr(e),
                    FnBody::None => {}
                }
                self.mods = saved_mods;
                self.loops = saved_loops;
                self.branch_depth = saved_br;
            }
            StmtKind::Impl { body, .. } | StmtKind::Trait { body, .. } => {
                let saved = std::mem::take(&mut self.loops);
                self.scan_block(body);
                self.loops = saved;
            }
            _ => {}
        }
    }

    fn scan_cond(&mut self, cond: &Cond) {
        match cond {
            Cond::Expr(e) => self.scan_expr(e),
            Cond::Let { value, and, .. } => {
                self.scan_expr(value);
                if let Some(rest) = and {
                    self.branch_depth += 1;
                    self.scan_cond(rest);
                    self.branch_depth -= 1;
                }
            }
            Cond::Guard { test, and } => {
                self.scan_expr(test);
                self.branch_depth += 1;
                self.scan_cond(and);
                self.branch_depth -= 1;
            }
            Cond::Const(e) => self.scan_expr(e),
        }
    }

    fn scan_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Binary {
                op: BinOp::Div,
                lhs,
                rhs,
                ..
            } => {
                self.scan_expr(lhs);
                self.scan_expr(rhs);
                self.check_div(expr.span, lhs, rhs, false);
            }
            ExprKind::Binary {
                op: BinOp::And | BinOp::Or,
                lhs,
                rhs,
                ..
            } => {
                self.scan_expr(lhs);
                self.branch_depth += 1;
                self.scan_expr(rhs);
                self.branch_depth -= 1;
            }
            ExprKind::Binary { lhs, rhs, .. } => {
                self.scan_expr(lhs);
                self.scan_expr(rhs);
            }
            ExprKind::Unary { expr: e, .. }
            | ExprKind::Paren(e)
            | ExprKind::Unwrap(e)
            | ExprKind::Field { base: e, .. } => self.scan_expr(e),
            ExprKind::Index { base, indices } => {
                self.scan_expr(base);
                for i in indices {
                    self.scan_expr(i);
                }
            }
            ExprKind::Call { callee, args } => {
                self.scan_expr(callee);
                for a in args {
                    self.scan_expr(&a.value);
                }
            }
            ExprKind::If {
                branches,
                otherwise,
            } => {
                for (i, (c, b)) in branches.iter().enumerate() {
                    if i > 0 {
                        self.branch_depth += 1;
                    }
                    self.scan_cond(c);
                    if i > 0 {
                        self.branch_depth -= 1;
                    }
                    self.branch_depth += 1;
                    self.scan_block(b);
                    self.branch_depth -= 1;
                }
                if let Some(b) = otherwise {
                    self.branch_depth += 1;
                    self.scan_block(b);
                    self.branch_depth -= 1;
                }
            }
            ExprKind::Match { scrutinee, arms } => {
                self.scan_expr(scrutinee);
                for arm in arms {
                    self.branch_depth += 1;
                    if let Some(g) = &arm.guard {
                        self.scan_expr(g);
                    }
                    self.scan_block(&arm.body);
                    self.branch_depth -= 1;
                }
            }
            ExprKind::Block(b) => self.scan_block(b),
            ExprKind::Tuple(es) | ExprKind::List(es) => {
                for e in es {
                    self.scan_expr(e);
                }
            }
            ExprKind::Comprehension { .. } => {}
            ExprKind::Lambda { body, .. } => {
                let saved = std::mem::take(&mut self.loops);
                self.scan_expr(body);
                self.loops = saved;
            }
            _ => {}
        }
    }

    fn check_div(&mut self, full_span: Span, lhs: &Expr, rhs: &Expr, is_assign: bool) {
        if self.loops.is_empty() {
            return;
        }
        let lhs_mod = self.mods.mod_ty_of(lhs, self.src, self.base);
        let rhs_mod = self.mods.mod_ty_of(rhs, self.src, self.base);
        let Some(mty) = lhs_mod.or_else(|| rhs_mod.clone()) else {
            return;
        };
        if !is_pure_divisor(rhs, &self.mods, true) {
            return;
        }
        let mut names = HashSet::new();
        collect_expr_names(rhs, &mut names);
        let mut hoist_idx = None;
        for idx in (0..self.loops.len()).rev() {
            if names.iter().any(|n| self.loops[idx].writes.contains(n)) {
                break;
            }
            hoist_idx = Some(idx);
        }
        let Some(idx) = hoist_idx else {
            return;
        };
        let hoist_span = self.loops[idx].stmt_span;
        let rhs_text = self.slice(rhs.span).trim().to_string();
        let inv_expr = if rhs_mod.is_some() {
            match &unparen(rhs).kind {
                ExprKind::Name(_) | ExprKind::Field { .. } | ExprKind::Call { .. } => {
                    format!("{rhs_text}.inv()")
                }
                _ => format!("({rhs_text}).inv()"),
            }
        } else {
            format!("{mty}({rhs_text}).inv()")
        };
        let base_name = match &unparen(rhs).kind {
            ExprKind::Name(n) => format!("inv_{n}"),
            ExprKind::Int(v) if *v > 0 => format!("inv_{v}"),
            ExprKind::Call { args, .. }
                if args.len() == 1
                    && matches!(unparen(&args[0].value).kind, ExprKind::Int(v) if v > 0) =>
            {
                if let ExprKind::Int(v) = unparen(&args[0].value).kind {
                    format!("inv_{v}")
                } else {
                    "inv_div".to_string()
                }
            }
            _ => "inv_div".to_string(),
        };
        let replace_span = Span::new(self.base + lhs.span.end, self.base + full_span.end);
        let fixable = line_indent(self.src, hoist_span.start).is_some()
            && (self.branch_depth == 0 || is_nonzero_literal(rhs, &self.mods));
        self.hits.push(Hit {
            diag_span: full_span.shift(self.base),
            hoist_span,
            replace_span,
            is_assign,
            rhs_text,
            inv_expr,
            base_name,
            fixable,
        });
    }
}

fn line_indent(src: &str, offset: u32) -> Option<&str> {
    let pos = offset as usize;
    let line_start = src[..pos].rfind('\n').map_or(0, |i| i + 1);
    let prefix = &src[line_start..pos];
    if prefix.bytes().all(|b| b == b' ' || b == b'\t') {
        Some(prefix)
    } else {
        None
    }
}

fn fresh_name(base: &str, used: &mut HashSet<String>) -> String {
    if used.insert(base.to_string()) {
        return base.to_string();
    }
    for k in 2.. {
        let cand = format!("{base}_{k}");
        if used.insert(cand.clone()) {
            return cand;
        }
    }
    unreachable!()
}

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let mut top_mods = Mods::new();
    let mut all_names = HashSet::new();

    // Pass 1: top-level type aliases, struct fields, and fn return types.
    for (base, item) in file.items {
        for stmt in &item.stmts {
            collect_all_names_stmt(stmt, &mut all_names);
            match &stmt.kind {
                StmtKind::TypeAlias { name, ty, .. } => {
                    if top_mods.mod_type_name(ty, file.src, *base).is_some() {
                        top_mods.mod_types.insert(name.text.clone());
                    }
                }
                StmtKind::Struct(decl) => {
                    for field in &decl.fields {
                        if let Some(m) = top_mods.mod_type_name(&field.ty, file.src, *base) {
                            top_mods.fields.insert(field.name.text.clone(), m);
                        }
                    }
                }
                StmtKind::Fn(decl) => {
                    if let Some(m) = decl
                        .ret
                        .as_ref()
                        .and_then(|t| top_mods.mod_type_name(t, file.src, *base))
                    {
                        top_mods.fns.insert(decl.name.text.clone(), m);
                    }
                }
                _ => {}
            }
        }
    }

    let mut scanner = Scanner {
        src: file.src,
        base: 0,
        mods: top_mods,
        loops: Vec::new(),
        branch_depth: 0,
        hits: Vec::new(),
    };
    for (base, item) in file.items {
        scanner.base = *base;
        for stmt in &item.stmts {
            scanner.scan_stmt(stmt);
        }
    }

    struct HoistGroup {
        var_name: String,
        sites: Vec<(Span, bool)>,
    }

    let mut used = all_names;
    let mut hoisted: HashMap<(u32, String), HoistGroup> = HashMap::new();
    let mut first_idx: HashMap<(u32, String), usize> = HashMap::new();
    for (i, hit) in scanner.hits.iter().enumerate() {
        if hit.fixable {
            let key = (hit.hoist_span.start, hit.inv_expr.clone());
            first_idx.entry(key.clone()).or_insert(i);
            let entry = hoisted.entry(key).or_insert_with(|| HoistGroup {
                var_name: fresh_name(&hit.base_name, &mut used),
                sites: Vec::new(),
            });
            entry.sites.push((hit.replace_span, hit.is_assign));
        }
    }

    let mut findings = Vec::new();
    for (i, hit) in scanner.hits.iter().enumerate() {
        let mut diag = Diagnostic::warning(
            LOOP_INVARIANT_MOD_DIV.code,
            hit.diag_span,
            format!(
                "loop-invariant `Mod` divisor `{}` computes an O(log P) inverse every iteration",
                hit.rhs_text
            ),
        )
        .with_help(format!(
            "hoist `let {} = {}` before the loop and multiply instead: one O(log P) inverse in place of one per iteration",
            hit.base_name, hit.inv_expr
        ));
        if hit.fixable {
            let key = (hit.hoist_span.start, hit.inv_expr.clone());
            if first_idx.get(&key) == Some(&i) {
                if let (Some(group), Some(indent)) = (
                    hoisted.get(&key),
                    line_indent(file.src, hit.hoist_span.start),
                ) {
                    let var_name = &group.var_name;
                    let mut edits = Vec::with_capacity(group.sites.len() + 1);
                    edits.push((
                        Span::new(hit.hoist_span.start, hit.hoist_span.start),
                        format!("let {var_name} = {}\n{indent}", hit.inv_expr),
                    ));
                    for &(span, is_assign) in &group.sites {
                        let replacement = if is_assign {
                            format!(" *= {var_name}")
                        } else {
                            format!(" * {var_name}")
                        };
                        edits.push((span, replacement));
                    }
                    diag.fixes.push(Fix {
                        title: format!("hoist `let {var_name} = {}` before the loop", hit.inv_expr),
                        edits,
                    });
                }
            }
        }
        findings.push(Finding {
            lint: &LOOP_INVARIANT_MOD_DIV,
            diag,
        });
    }
    findings
}
