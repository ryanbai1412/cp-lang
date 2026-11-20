//! Expression types.

use super::*;
use cpl_syntax::ast::{self, BinOp, Cond, Expr, ExprKind, StrPiece, UnOp};

fn range_of(elem: Ty) -> Ty {
    Ty::Named {
        name: "Range".into(),
        args: vec![elem],
    }
}

impl<'a> Checker<'a> {
    pub(crate) fn infer(&mut self, expr: &Expr) -> Ty {
        // An outermost place expression (`a[i]`, not the `a` inside it)
        // uses its place: a conflict with a live lend is reported once
        // its parts are known.
        let uses_place = self.enter_place(expr);
        let ty = self.infer_kind(expr);
        if uses_place {
            self.check_access(expr, false);
        }
        ty
    }

    fn infer_kind(&mut self, expr: &Expr) -> Ty {
        match &expr.kind {
            ExprKind::Int(_) => Ty::Int,
            ExprKind::Float(_) => Ty::Float,
            ExprKind::Char(_) => Ty::Char,
            ExprKind::Bool(_) => Ty::Bool,
            ExprKind::None => Ty::Unknown,
            ExprKind::Str(pieces) => {
                for piece in pieces {
                    if let StrPiece::Expr(inner) = piece {
                        self.infer(inner);
                    }
                }
                Ty::Str
            }
            ExprKind::Name(name) => self.name_ty(name, expr.span),
            ExprKind::SelfValue => self.name_ty("self", expr.span),
            ExprKind::Paren(inner) => self.infer(inner),
            ExprKind::Unary { op, expr: inner } => {
                let ty = self.infer(inner);
                match op {
                    UnOp::Not => {
                        self.require_bool(&ty, inner, "`not`");
                        Ty::Bool
                    }
                    UnOp::Neg | UnOp::BitNot if ty.is_numeric() => ty.widen(),
                    UnOp::Neg | UnOp::BitNot => {
                        self.bad_operand(&ty, inner, if *op == UnOp::Neg { "-" } else { "~" })
                    }
                }
            }
            ExprKind::Binary {
                op,
                op_span,
                lhs,
                rhs,
            } => self.binary(*op, *op_span, lhs, rhs),
            ExprKind::Compare { first, rest } => {
                let mut left = self.infer(first);
                let mut left_expr: &Expr = first;
                for (op, right_expr) in rest {
                    let right = self.infer(right_expr);
                    if !matches!(op, ast::CmpOp::In | ast::CmpOp::NotIn) {
                        self.comparable(&left, left_expr, &right, right_expr);
                    }
                    left = right;
                    left_expr = right_expr;
                }
                if rest.iter().any(|(op, _)| *op == ast::CmpOp::Order) {
                    return Ty::Unknown;
                }
                Ty::Bool
            }
            ExprKind::Range {
                lo,
                hi,
                inclusive,
                step,
            } => {
                let lo_ty = lo.as_ref().map(|e| self.infer(e));
                let hi_ty = hi.as_ref().map(|e| self.infer(e));
                if let Some(step) = step {
                    self.infer(step);
                }
                let elem = match (&lo_ty, &hi_ty) {
                    // Two boundaries enclose positions: `l.l..r.r` is a
                    // range of `V`, like `l..=r`.
                    (Some(Ty::Pos(d)), _) | (_, Some(Ty::Pos(d))) => {
                        Ty::Pos(d.strip_suffix(".bounds").unwrap_or(d).to_string())
                    }
                    (Some(a), Some(b)) if a.is_int() && b.is_int() => {
                        let lo_min = lo.as_ref().and_then(|e| self.bounds(e).0);
                        let hi_max = hi.as_ref().and_then(|e| self.bounds(e).1).map(|h| {
                            if *inclusive {
                                h
                            } else {
                                h - 1
                            }
                        });
                        Ty::Range {
                            lo: lo_min,
                            hi: hi_max,
                            text: self.text(expr.span).to_string(),
                        }
                    }
                    (Some(a), None) | (None, Some(a)) if a.is_int() => Ty::Int,
                    _ => Ty::Unknown,
                };
                range_of(elem)
            }
            ExprKind::Call { callee, args } => self.call(expr, callee, args),
            ExprKind::Index { base, indices } => self.index(expr, base, indices),
            ExprKind::Field { base, name } => self.field(base, name),
            ExprKind::Generic { base, args } => {
                let base_ty = self.infer(base);
                let args: Vec<Ty> = args.iter().map(|a| self.resolve_type(a)).collect();
                // `Sum<Int>`, `f::<Int>`: the declared type with its
                // parameters filled in.
                let declared =
                    match &base.kind {
                        ExprKind::Name(name) if self.local(name).is_none() => {
                            match self.env.global(name).as_deref() {
                                Some(
                                    Def::Value { ty, generics, .. } | Def::Fn { ty, generics },
                                ) if !generics.is_empty() => Some((ty.clone(), generics.clone())),
                                _ => None,
                            }
                        }
                        _ => None,
                    };
                match (base_ty, declared) {
                    // `scc::<V>(..)`: parameters after the ones given are
                    // read off the arguments.
                    (_, Some((ty, generics))) => {
                        if args.len() > generics.len() {
                            let message = format!(
                                "`{}` takes {}, but {} given",
                                self.quote(base.span),
                                plural(generics.len(), "type argument", "type arguments"),
                                if args.len() == 1 {
                                    "1 was".to_string()
                                } else {
                                    format!("{} were", args.len())
                                }
                            );
                            self.error(codes::WRONG_ARG_COUNT, expr.span, message);
                            return Ty::Error;
                        }
                        let bindings = generics.into_iter().zip(args).collect();
                        ty.subst(&bindings)
                    }
                    (Ty::TypeName(name), None) => Ty::Named { name, args },
                    (other, None) => other,
                }
            }
            ExprKind::Tuple(items) if items.is_empty() => Ty::Unit,
            ExprKind::Tuple(items) => {
                let tys = items.iter().map(|i| self.infer(i)).collect();
                for item in items {
                    self.no_escape(item, "storing it in a list or tuple");
                }
                Ty::Tuple(tys)
            }
            ExprKind::Record(fields) => Ty::Record(
                fields
                    .iter()
                    .map(|(name, value)| (name.text.clone(), self.infer(value)))
                    .collect(),
            ),
            ExprKind::List(items) => {
                let mut elem = Ty::Unknown;
                for (i, item) in items.iter().enumerate() {
                    let ty = self.infer(item);
                    if i == 0 {
                        elem = ty.widen();
                    }
                    self.no_escape(item, "storing it in a list or tuple");
                }
                if items.is_empty() {
                    Ty::Unknown
                } else {
                    Ty::list(elem)
                }
            }
            ExprKind::Comprehension { body, clauses } => {
                self.push_scope();
                for clause in clauses {
                    let iter_ty = self.infer(&clause.iter);
                    let elem = self.element_ty(&iter_ty);
                    self.bind(&clause.pat, &elem, false, "for");
                    for cond in &clause.conds {
                        self.cond(cond);
                    }
                }
                let body_ty = self.infer(body).widen();
                self.pop_scope();
                Ty::list(body_ty)
            }
            ExprKind::Table {
                domains,
                params,
                body,
            } => {
                let index: Vec<Ty> = domains.iter().map(|d| self.resolve_type(d)).collect();
                self.push_scope();
                for (i, param) in params.iter().enumerate() {
                    let ty = index.get(i).cloned().unwrap_or(Ty::Unknown);
                    self.bind(param, &ty, false, "param");
                }
                let elem = self.infer(body).widen();
                self.pop_scope();
                Ty::Array {
                    index,
                    elem: Box::new(elem),
                }
            }
            ExprKind::Lambda { params, body } => {
                self.push_scope();
                for param in params {
                    self.bind(param, &Ty::Unknown, false, "param");
                }
                // A lambda's `return` and trailing value belong to the lambda.
                self.fns.push((String::new(), Ty::Unknown, false));
                self.infer(body);
                self.fns.pop();
                self.pop_scope();
                Ty::Unknown
            }
            ExprKind::Block(block) => self.block(block),
            ExprKind::If {
                branches,
                otherwise,
            } => {
                let mut result = None;
                // Inside a branch its condition holds; after it, in the
                // branches that follow, it does not.
                let mark = self.lend_mark();
                for (cond, body) in branches {
                    self.push_scope();
                    self.cond(cond);
                    let branch = self.lend_mark();
                    self.add_cond_facts(cond, false);
                    let ty = self.block(body);
                    self.lend_reset(branch);
                    self.add_cond_facts(cond, true);
                    self.pop_scope();
                    result.get_or_insert(ty);
                }
                let leaves = |body: &ast::Block| {
                    matches!(
                        body.stmts.last().map(|s| &s.kind),
                        Some(
                            ast::StmtKind::Return(_)
                                | ast::StmtKind::Break(_)
                                | ast::StmtKind::Continue(_)
                        )
                    )
                };
                // `if i == j: return` leaves `i != j` for the rest of the
                // block.
                let after = match (branches.as_slice(), otherwise) {
                    ([(cond, body)], None) if leaves(body) => Some(cond),
                    _ => None,
                };
                let result_ty = match otherwise {
                    Some(body) => {
                        let ty = self.block(body);
                        match result {
                            Some(first) if first.is_vague() => ty,
                            Some(first) => first.widen(),
                            None => ty,
                        }
                    }
                    None => Ty::Unit,
                };
                self.lend_reset(mark);
                if let Some(cond) = after {
                    self.add_cond_facts(cond, true);
                }
                result_ty
            }
            ExprKind::Match { scrutinee, arms } => {
                let scrutinee_ty = self.infer(scrutinee);
                let mut result = Ty::Unknown;
                for (i, arm) in arms.iter().enumerate() {
                    self.push_scope();
                    self.bind(&arm.pat, &scrutinee_ty, false, "let");
                    if let Some(guard) = &arm.guard {
                        let ty = self.infer(guard);
                        self.require_bool(&ty, guard, "the guard");
                    }
                    let ty = self.block(&arm.body);
                    self.pop_scope();
                    if i == 0 {
                        result = ty.widen();
                    }
                }
                result
            }
            ExprKind::With { base, updates } => {
                let ty = self.infer(base);
                for (name, value) in updates {
                    let value_ty = self.infer(value);
                    if let Some(field_ty) = self.field_of(&ty, name) {
                        self.expect_fits(&value_ty, &field_ty, value, |c, got, want| {
                            format!(
                                "`{}` is {}, but `{}` is {}",
                                name.text,
                                want.with_article(),
                                c.quote(value.span),
                                got.with_article()
                            )
                        });
                    }
                }
                ty
            }
            ExprKind::Try(inner) | ExprKind::Unwrap(inner) => match self.infer(inner) {
                Ty::Optional(inner) => *inner,
                Ty::Error => Ty::Error,
                _ => Ty::Unknown,
            },
            ExprKind::Missing | ExprKind::Error => Ty::Error,
        }
    }

    pub(crate) fn cond(&mut self, cond: &Cond) {
        match cond {
            Cond::Expr(expr) => {
                let ty = self.infer(expr);
                self.require_bool(&ty, expr, "the condition");
            }
            Cond::Const(expr) => {
                self.infer(expr);
            }
            Cond::Let { pat, value, and } => {
                let ty = match self.infer(value) {
                    Ty::Optional(inner) => *inner,
                    Ty::Error => Ty::Error,
                    _ => Ty::Unknown,
                };
                self.bind(pat, &ty, false, "let");
                if let Some(and) = and {
                    let ty = self.infer(and);
                    self.require_bool(&ty, and, "the condition");
                }
            }
        }
    }

    /// Reports `expr` if it is known not to be a `Bool`.
    pub(crate) fn require_bool(&mut self, ty: &Ty, expr: &Expr, what: &str) {
        if ty.is_vague() || *ty == Ty::Bool {
            return;
        }
        if !(ty.is_numeric() || matches!(ty, Ty::Str | Ty::Char | Ty::Pos(_) | Ty::Optional(_))) {
            return;
        }
        let quoted = self.quote(expr.span);
        let message = format!(
            "{what} needs a Bool, but `{quoted}` is {}",
            ty.with_article()
        );
        if let Some(diag) = self.error(codes::CONDITION_NOT_BOOL, expr.span, message) {
            diag.help = Some(match ty {
                Ty::Optional(_) => format!("compare it: `{quoted} != none`"),
                t if t.is_numeric() => format!("compare it: `{quoted} != 0`"),
                _ => "write a comparison".to_string(),
            });
        }
    }

    fn bad_operand(&mut self, ty: &Ty, expr: &Expr, op: &str) -> Ty {
        if ty.is_vague() {
            return ty.clone();
        }
        if matches!(ty, Ty::Bool | Ty::Str) {
            let message = format!(
                "`{op}` needs a number, but `{}` is {}",
                self.quote(expr.span),
                ty.with_article()
            );
            self.error(codes::BAD_OPERANDS, expr.span, message);
            return Ty::Error;
        }
        Ty::Unknown
    }

    fn binary(&mut self, op: BinOp, op_span: Span, lhs: &Expr, rhs: &Expr) -> Ty {
        let left = self.infer(lhs);
        let right = self.infer(rhs);
        match op {
            BinOp::And | BinOp::Or => {
                self.require_bool(&left, lhs, &format!("`{}`", op.text()));
                self.require_bool(&right, rhs, &format!("`{}`", op.text()));
                Ty::Bool
            }
            BinOp::Coalesce => match left {
                Ty::Optional(inner) if !inner.is_vague() => *inner,
                Ty::Error => right,
                _ if right.is_vague() => Ty::Unknown,
                _ => right.widen(),
            },
            // `total + ` mid-edit: the half that is there says what the
            // whole will be.
            _ if matches!(rhs.kind, ExprKind::Missing) && left.is_numeric() => left.widen(),
            _ if self.operator_trait(op, &left).is_some() => {
                let (trait_name, owner) = self.operator_trait(op, &left).unwrap();
                let implemented = self
                    .env
                    .impls(&owner)
                    .iter()
                    .any(|(implemented, _)| implemented.as_deref() == Some(trait_name));
                if !implemented {
                    let message = format!(
                        "`{}` needs `impl {trait_name} for {owner}`, and there is none",
                        op.text()
                    );
                    if let Some(diag) = self.error(codes::MISSING_IMPL, op_span, message) {
                        diag.help = Some(format!(
                            "an operator on a struct is the trait's fn: write `impl {trait_name} for {owner}`"
                        ));
                    }
                    return Ty::Error;
                }
                left
            }
            _ if left == Ty::Error || right == Ty::Error => Ty::Error,
            _ if left.is_vague() || right.is_vague() => Ty::Unknown,
            _ if left.is_int() && right.is_int() => Ty::Int,
            _ if left.is_numeric() && right.is_numeric() => Ty::Float,
            BinOp::Add if left == Ty::Str && right == Ty::Str => Ty::Str,
            _ => {
                // Only the clear mistakes are reported: arithmetic on a Bool,
                // or a Str mixed with a number. Positions, chars and library
                // types have their own operators, which `std` will define.
                let primitive = |t: &Ty| t.is_numeric() || matches!(t, Ty::Bool | Ty::Str);
                let clear = primitive(&left) && primitive(&right);
                if clear {
                    let message = format!(
                        "`{}` can't combine {} and {}",
                        op.text(),
                        left.with_article(),
                        right.with_article()
                    );
                    if let Some(diag) = self.error(codes::BAD_OPERANDS, op_span, message) {
                        if left == Ty::Str || right == Ty::Str {
                            diag.help = Some("put the value in the string: `\"{x}\"`".into());
                        }
                    }
                    return Ty::Error;
                }
                Ty::Unknown
            }
        }
    }

    /// Reports a comparison between positions of two different domains.
    /// The trait an arithmetic operator asks of a struct on its left, and
    /// the struct's name: `a + b` with `a: Pt` is `Add` for `Pt`.
    fn operator_trait(&mut self, op: BinOp, left: &Ty) -> Option<(&'static str, String)> {
        let trait_name = match op {
            BinOp::Add => "Add",
            BinOp::Sub => "Sub",
            BinOp::Mul => "Mul",
            BinOp::Div => "Div",
            BinOp::Rem => "Rem",
            _ => return None,
        };
        let Ty::Named { name, .. } = left else {
            return None;
        };
        matches!(self.env.type_def(name).as_deref(), Some(Def::Struct { .. }))
            .then(|| (trait_name, name.clone()))
    }

    fn comparable(&mut self, left: &Ty, left_expr: &Expr, right: &Ty, right_expr: &Expr) {
        let (Ty::Pos(a), Ty::Pos(b)) = (left, right) else {
            return;
        };
        if a == b {
            return;
        }
        let message = format!(
            "`{}` is {} and `{}` is {}; positions of different domains can't be compared",
            self.quote(left_expr.span),
            left.with_article(),
            self.quote(right_expr.span),
            right.with_article()
        );
        self.error(
            codes::WRONG_DOMAIN,
            left_expr.span.to(right_expr.span),
            message,
        );
    }

    /// What a `for` over a value of type `ty` yields.
    pub(crate) fn element_ty(&mut self, ty: &Ty) -> Ty {
        match ty {
            Ty::Domain(name) => Ty::Pos(name.clone()),
            Ty::Array { elem, .. } => (**elem).clone(),
            Ty::Str => Ty::Char,
            Ty::Named { name, args }
                if matches!(name.as_str(), "List" | "Set" | "Range" | "Gen") && args.len() == 1 =>
            {
                args[0].clone()
            }
            Ty::Error => Ty::Error,
            _ => Ty::Unknown,
        }
    }

    /// The type of field `name` on a value of type `ty`, if `ty` is a
    /// struct or record that has it.
    fn field_of(&mut self, ty: &Ty, name: &ast::Name) -> Option<Ty> {
        match ty {
            Ty::Record(fields) => fields
                .iter()
                .find(|(n, _)| *n == name.text)
                .map(|(_, t)| t.clone()),
            Ty::Named {
                name: type_name,
                args,
            } => {
                let members = self.members_of(type_name)?;
                let bindings = members.bindings(args);
                members
                    .fields
                    .iter()
                    .find(|f| f.name == name.text)
                    .map(|f| self.settle(&f.ty.subst(&bindings)))
            }
            _ => None,
        }
    }

    /// A struct value seen as a struct with fewer fields, when one is
    /// wanted: a `Group<Int>` has the `id` and `op` of a `Monoid<Int>`, so
    /// it is one (decision 10.4). Anything else is returned as it is.
    pub(crate) fn narrowed(&mut self, got: &Ty, want: &Ty) -> Ty {
        let (
            Ty::Named {
                name: got_name,
                args: got_args,
            },
            Ty::Named {
                name: want_name, ..
            },
        ) = (got, want)
        else {
            return got.clone();
        };
        if got_name == want_name {
            return got.clone();
        }
        let (Some(got_members), Some(want_members)) =
            (self.members_of(got_name), self.members_of(want_name))
        else {
            return got.clone();
        };
        if want_members.fields.is_empty() {
            return got.clone();
        }
        let got_bindings = got_members.bindings(got_args);
        let mut bindings = crate::ty::Bindings::new();
        for want_field in &want_members.fields {
            let Some(got_field) = got_members
                .fields
                .iter()
                .find(|f| f.name == want_field.name)
            else {
                return got.clone();
            };
            let got_ty = got_field.ty.subst(&got_bindings);
            if !got_ty.fits(&want_field.ty) {
                return got.clone();
            }
            want_field.ty.unify(&got_ty, &mut bindings);
        }
        Ty::Named {
            name: want_name.clone(),
            args: want_members
                .generics
                .iter()
                .map(|g| bindings.get(g).cloned().unwrap_or(Ty::Unknown))
                .collect(),
        }
    }

    /// Everything `value.name` can reach on a struct or enum: its fields,
    /// its methods (a `@get` method reads as its value), what its body
    /// owns, and the same from each impl for it.
    pub(crate) fn members_of(&mut self, type_name: &str) -> Option<Members> {
        let mut out = Members::default();
        let add = |out: &mut Members, methods: &[(String, Ty)], getters: &[String]| {
            for (name, ty) in methods {
                let ty = match ty {
                    Ty::Fn { ret, .. } if getters.contains(name) => (**ret).clone(),
                    other => other.clone(),
                };
                out.members.push((name.clone(), ty));
            }
        };
        match self.env.type_def(type_name).as_deref() {
            Some(Def::Struct {
                generics,
                fields,
                methods,
                getters,
                owned,
                projections,
                ..
            }) => {
                out.generics = generics.clone();
                out.fields = fields.clone();
                add(&mut out, methods, getters);
                out.members.extend(owned.iter().cloned());
                out.projections.extend(projections.iter().cloned());
            }
            Some(Def::Enum { methods, .. }) => add(&mut out, methods, &[]),
            _ => return None,
        }
        for (trait_name, def) in self.env.impls(type_name) {
            // Every fn of the trait comes with the impl: the ones the
            // trait writes a body for, and the ones the impl must write
            // (one it leaves out is reported once, on the impl).
            if let Some(Def::Trait { methods, .. }) =
                trait_name.and_then(|t| self.env.type_def(&t)).as_deref()
            {
                for method in methods {
                    out.members.push((method.name.clone(), method.ty.clone()));
                }
            }
            if let Def::Impl {
                methods,
                getters,
                owned,
                projections,
            } = &*def
            {
                add(&mut out, methods, getters);
                out.members.extend(owned.iter().cloned());
                out.projections.extend(projections.iter().cloned());
            }
        }
        Some(out)
    }

    fn field(&mut self, base: &Expr, name: &ast::Name) -> Ty {
        // `treap.merge` after `use lib.treap`: a public name of a module.
        if let ExprKind::Name(alias) = &base.kind {
            if self.local(alias).is_none() && self.env.is_module(alias) {
                return match self.env.global(&format!("{alias}.{}", name.text)) {
                    Some(def) => def.value_ty(&name.text),
                    None if name.text.is_empty() => Ty::Error,
                    None => {
                        let message =
                            format!("module `{alias}` has no public name `{}`", name.text);
                        self.error(codes::UNRESOLVED_NAME, name.span, message);
                        Ty::Error
                    }
                };
            }
        }
        let base_ty = self.infer(base);
        if name.text.is_empty() {
            return Ty::Error;
        }
        let found = match &base_ty {
            Ty::Record(fields) => {
                let names: Vec<String> = fields.iter().map(|(n, _)| n.clone()).collect();
                Some((self.field_of(&base_ty, name), names, Vec::new()))
            }
            Ty::Tuple(items) => {
                return match name.text.parse::<usize>() {
                    Ok(i) if i < items.len() => items[i].clone(),
                    _ => Ty::Unknown,
                }
            }
            Ty::Named {
                name: type_name,
                args,
            } => self.members_of(type_name).map(|members| {
                let bindings = members.bindings(args);
                (
                    members
                        .fields
                        .iter()
                        .find(|f| f.name == name.text)
                        .map(|f| f.ty.subst(&bindings)),
                    members.fields.iter().map(|f| f.name.clone()).collect(),
                    members
                        .members
                        .into_iter()
                        .map(|(m, ty)| (m, ty.subst(&bindings)))
                        .collect::<Vec<_>>(),
                )
            }),
            // The members of a domain that give positions back.
            Ty::Domain(domain) => {
                return match name.text.as_str() {
                    "first" | "last" => Ty::Pos(domain.clone()),
                    "rev" => Ty::Domain(domain.clone()),
                    "bounds" => Ty::Domain(format!("{domain}.bounds")),
                    _ => Ty::Unknown,
                }
            }
            Ty::Pos(domain) => {
                return match name.text.as_str() {
                    "ord" | "to_idx" | "idx" | "label" => Ty::Int,
                    // The boundaries on either side of a position.
                    "l" | "r" | "before" | "after" if !domain.ends_with(".bounds") => {
                        Ty::Pos(format!("{domain}.bounds"))
                    }
                    _ => Ty::Unknown,
                };
            }
            // `Dir.L`: a variant named through its enum.
            Ty::TypeName(type_name) => {
                if let Some(Def::Enum { variants, .. }) = self.env.type_def(type_name).as_deref() {
                    if let Some((_, fields)) = variants.iter().find(|(v, _)| *v == name.text) {
                        let def = Def::Variant {
                            of: type_name.clone(),
                            fields: fields.clone(),
                        };
                        return def.value_ty(&name.text);
                    }
                }
                // `Graph.read()`, `Rooted.Pre`: what the type has without a value.
                self.members_of(type_name)
                    .map(|members| (None, Vec::new(), members.members))
            }
            _ => None,
        };
        let Some((field_ty, field_names, methods)) = found else {
            // Members of library types, arrays and domains (`xs.len`,
            // `V.rev`) come from `std`, which is not loaded yet.
            return if base_ty == Ty::Error {
                Ty::Error
            } else {
                Ty::Unknown
            };
        };
        let ty = match field_ty {
            Some(ty) => self.settle(&ty),
            None => match methods.iter().find(|(m, _)| *m == name.text) {
                Some((_, ty)) => self.settle(ty),
                None => {
                    let known = field_names
                        .iter()
                        .chain(methods.iter().map(|(m, _)| m))
                        .map(String::as_str);
                    let suggestion = closest(&name.text, known);
                    let message = format!("{} has no field or method `{}`", base_ty, name.text);
                    if let Some(diag) = self.error(codes::UNKNOWN_FIELD, name.span, message) {
                        match suggestion {
                            Some(s) => {
                                diag.help = Some(format!("did you mean `{s}`?"));
                                diag.fixes.push(cpl_diag::Fix {
                                    title: format!("change to `{s}`"),
                                    edits: vec![(name.span, s)],
                                });
                            }
                            None if !field_names.is_empty() => {
                                diag.help =
                                    Some(format!("its fields are {}", list_names(&field_names)));
                            }
                            None => {}
                        }
                    }
                    return Ty::Error;
                }
            },
        };
        self.hovers.push(HoverEntry {
            span: name.span,
            name: name.text.clone(),
            ty: ty.clone(),
            keyword: "field",
            decl: None,
        });
        ty
    }

    fn index(&mut self, expr: &Expr, base: &Expr, indices: &[Expr]) -> Ty {
        let base_ty = self.infer(base);
        let index_tys: Vec<Ty> = indices.iter().map(|i| self.infer(i)).collect();
        match &base_ty {
            // `g[p]` with `p: (Row, Col)`: `a[i, j]` is sugar for `a[(i, j)]`.
            Ty::Array { index, elem }
                if index.len() > 1
                    && indices.len() == 1
                    && (index_tys[0].is_vague()
                        || matches!(&index_tys[0], Ty::Tuple(items) if items.len() == index.len())) =>
            {
                let want = Ty::Tuple(index.clone());
                self.expect_fits(&index_tys[0], &want, &indices[0], |c, got, want| {
                    format!(
                        "`{}` is indexed by {}, but `{}` is {}",
                        c.quote(base.span),
                        want,
                        c.quote(indices[0].span),
                        got.with_article()
                    )
                });
                (**elem).clone()
            }
            Ty::Array { index, elem } => {
                if index.len() != indices.len() {
                    let wanted: Vec<String> = index.iter().map(Ty::to_string).collect();
                    let message = format!(
                        "`{}` takes {} ({}), but {} given",
                        self.quote(base.span),
                        plural(index.len(), "index", "indices"),
                        wanted.join(", "),
                        if indices.len() == 1 {
                            "1 was".to_string()
                        } else {
                            format!("{} were", indices.len())
                        },
                    );
                    if let Some(diag) = self.error(codes::NOT_INDEXABLE, expr.span, message) {
                        diag.help = Some(
                            "all indices go in one pair of brackets: `g[r, c]`, never `g[r][c]`"
                                .into(),
                        );
                    }
                    return Ty::Error;
                }
                let mut slice = false;
                for ((want, got), index_expr) in index.iter().zip(&index_tys).zip(indices) {
                    if matches!(got, Ty::Named { name, .. } if name == "Range") {
                        slice = true;
                        continue;
                    }
                    if got.fits(want) {
                        continue;
                    }
                    let message = format!(
                        "`{}` is indexed by {}, but `{}` is {}",
                        self.quote(base.span),
                        want,
                        self.quote(index_expr.span),
                        got.with_article()
                    );
                    let base_text = self.quote(base.span);
                    let index_text = self.quote(index_expr.span);
                    let boundary = match (got, want) {
                        (Ty::Pos(got), Ty::Pos(want)) => {
                            got.strip_suffix(".bounds") == Some(want)
                                || want.strip_suffix(".bounds") == Some(got)
                        }
                        _ => false,
                    };
                    if let Some(diag) = self.error(codes::WRONG_DOMAIN, index_expr.span, message) {
                        diag.help = Some(if boundary {
                            "a position and the boundaries around it are different indices; use `.l` / `.r` for the boundaries on either side of a position".to_string()
                        } else if got.is_int() {
                            format!("convert it with `{want}({index_text})`, or write `{base_text}.raw[{index_text}]` to index by number")
                        } else {
                            format!("index with {}", want.with_article())
                        });
                    }
                }
                if slice {
                    // A slice is its array seen through a range: lent,
                    // and indexed by the same positions.
                    self.note_view(expr, "slice");
                    base_ty.clone()
                } else {
                    (**elem).clone()
                }
            }
            // `xs[0..5]`: a part of a list or a string is one too.
            ty if (*ty == Ty::Str || matches!(ty, Ty::Named { name, .. } if name == "List"))
                && matches!(index_tys.as_slice(), [Ty::Named { name, .. }] if name == "Range") =>
            {
                ty.clone()
            }
            Ty::Named { name, args } if name == "List" && args.len() == 1 => args[0].clone(),
            Ty::Named { name, args } if name == "Map" && args.len() == 2 => args[1].clone(),
            Ty::Str => Ty::Char,
            ty if ty.is_numeric() || matches!(ty, Ty::Bool | Ty::Char | Ty::Fn { .. }) => {
                let message = format!(
                    "`{}` is {} and can't be indexed",
                    self.quote(base.span),
                    ty.with_article()
                );
                self.error(codes::NOT_INDEXABLE, base.span, message);
                Ty::Error
            }
            Ty::Error => Ty::Error,
            _ => Ty::Unknown,
        }
    }

    /// Reports with `message(self, got, want)` unless `got` fits `want`.
    pub(crate) fn expect_fits(
        &mut self,
        got: &Ty,
        want: &Ty,
        at: &Expr,
        message: impl FnOnce(&Self, &Ty, &Ty) -> String,
    ) -> bool {
        // `none` fills whatever is declared, and code that already has an
        // error is not reported again.
        let got = &self.narrowed(got, want);
        if got.fits(want)
            || matches!(at.kind, ExprKind::None)
            || got.has_error()
            || want.has_error()
        {
            return true;
        }
        let text = message(self, got, want);
        let code = if matches!((got, want), (Ty::Pos(_), _) | (_, Ty::Pos(_))) {
            codes::WRONG_DOMAIN
        } else {
            codes::TYPE_MISMATCH
        };
        let quoted = self.quote(at.span);
        if let Some(diag) = self.error(code, at.span, text) {
            if let (true, Ty::Pos(domain)) = (got.is_int(), want) {
                diag.help = Some(format!("convert it with `{domain}({quoted})`"));
            }
        }
        false
    }
}

/// What a struct or enum, with its impls, lets a value reach by name.
#[derive(Default)]
pub(crate) struct Members {
    generics: Vec<String>,
    pub(crate) fields: Vec<crate::ty::Param>,
    members: Vec<(String, Ty)>,
    pub(crate) projections: Vec<crate::def::Projection>,
}

impl Members {
    /// What each type parameter is for a value of the type with `args`.
    fn bindings(&self, args: &[Ty]) -> crate::ty::Bindings {
        if args.len() != self.generics.len() {
            return crate::ty::Bindings::new();
        }
        self.generics
            .iter()
            .cloned()
            .zip(args.iter().cloned())
            .collect()
    }
}

pub(crate) fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

pub(crate) fn list_names(names: &[String]) -> String {
    names
        .iter()
        .map(|n| format!("`{n}`"))
        .collect::<Vec<_>>()
        .join(", ")
}
