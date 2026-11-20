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
                    (Some(Ty::Pos(d)), _) | (_, Some(Ty::Pos(d))) => Ty::Pos(d.clone()),
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
                match base_ty {
                    Ty::TypeName(name) => Ty::Named { name, args },
                    other => other,
                }
            }
            ExprKind::Tuple(items) if items.is_empty() => Ty::Unit,
            ExprKind::Tuple(items) => Ty::Tuple(items.iter().map(|i| self.infer(i)).collect()),
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
                for (cond, body) in branches {
                    self.push_scope();
                    self.cond(cond);
                    let ty = self.block(body);
                    self.pop_scope();
                    result.get_or_insert(ty);
                }
                match otherwise {
                    Some(body) => {
                        let ty = self.block(body);
                        match result {
                            Some(first) if first.is_vague() => ty,
                            Some(first) => first.widen(),
                            None => ty,
                        }
                    }
                    None => Ty::Unit,
                }
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
            Cond::Let { pat, value } => {
                let ty = match self.infer(value) {
                    Ty::Optional(inner) => *inner,
                    Ty::Error => Ty::Error,
                    _ => Ty::Unknown,
                };
                self.bind(pat, &ty, false, "let");
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
                name: type_name, ..
            } => match self.env.global(type_name).as_deref() {
                Some(Def::Struct { fields, .. }) => fields
                    .iter()
                    .find(|f| f.name == name.text)
                    .map(|f| f.ty.clone()),
                _ => None,
            },
            _ => None,
        }
    }

    fn field(&mut self, base: &Expr, name: &ast::Name) -> Ty {
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
                name: type_name, ..
            } => match self.env.global(type_name).as_deref() {
                Some(Def::Struct {
                    fields, methods, ..
                }) => Some((
                    fields
                        .iter()
                        .find(|f| f.name == name.text)
                        .map(|f| f.ty.clone()),
                    fields.iter().map(|f| f.name.clone()).collect(),
                    methods.clone(),
                )),
                Some(Def::Enum { methods, .. }) => Some((None, Vec::new(), methods.clone())),
                _ => None,
            },
            // The members of a domain that give positions back.
            Ty::Domain(domain) => {
                return match name.text.as_str() {
                    "first" | "last" => Ty::Pos(domain.clone()),
                    "rev" => Ty::Domain(domain.clone()),
                    "bounds" => Ty::Domain(format!("{domain}.bounds")),
                    _ => Ty::Unknown,
                }
            }
            Ty::Pos(_) if name.text == "ord" => return Ty::Int,
            // `Dir.L`: a variant named through its enum.
            Ty::TypeName(type_name) => match self.env.global(type_name).as_deref() {
                Some(Def::Enum { variants, methods }) => {
                    if let Some((_, fields)) = variants.iter().find(|(v, _)| *v == name.text) {
                        let def = Def::Variant {
                            of: type_name.clone(),
                            fields: fields.clone(),
                        };
                        return def.value_ty(&name.text);
                    }
                    Some((None, Vec::new(), methods.clone()))
                }
                // `Graph.read()`: a static fn of the struct.
                Some(Def::Struct { methods, .. }) => Some((None, Vec::new(), methods.clone())),
                _ => None,
            },
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
            Some(ty) => ty,
            None => match methods.iter().find(|(m, _)| *m == name.text) {
                Some((_, ty)) => ty.clone(),
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
                    if let Some(diag) = self.error(codes::WRONG_DOMAIN, index_expr.span, message) {
                        diag.help = Some(if got.is_int() {
                            format!("convert it with `{want}({index_text})`, or write `{base_text}.raw[{index_text}]` to index by number")
                        } else {
                            format!("index with {}", want.with_article())
                        });
                    }
                }
                if slice {
                    Ty::Unknown
                } else {
                    (**elem).clone()
                }
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
