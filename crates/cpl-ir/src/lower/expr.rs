//! Expressions.

use super::*;

/// One `for` of a comprehension while its body is being lowered.
pub(super) struct LoopFrame {
    id: LoopId,
    slot: Slot,
    iter: Iter,
    /// Statements that take the loop variable's pattern apart.
    pre: Vec<Stmt>,
    cond: Option<Expr>,
}

impl<'a> Lowerer<'a> {
    /// Lowers `e` where a `want` is needed.
    pub(super) fn expect(&mut self, e: &ast::Expr, want: &Ty) -> Expr {
        let value = self.expr(e, Some(want));
        self.coerce(value, want, e.span)
    }

    /// Lowers `e`. `hint` is the type the context would like, which only
    /// the forms that can't tell on their own look at: `none`, `[]`,
    /// `read()`, and number literals.
    pub(super) fn expr(&mut self, e: &ast::Expr, hint: Option<&Ty>) -> Expr {
        // Where an optional is wanted, a plain value is typed as the inside.
        let inner_hint = match hint {
            Some(Ty::Opt(inner)) => Some(&**inner),
            other => other,
        };
        let span = e.span;
        match &e.kind {
            ast::ExprKind::Int(value) => match i64::try_from(*value) {
                Ok(value) if inner_hint == Some(&Ty::Float) => {
                    self.mk(ExprKind::Float(value as f64), Ty::Float, span)
                }
                Ok(value) => self.mk(ExprKind::Int(value), Ty::Int, span),
                Err(_) => {
                    self.error(
                        codes::BAD_NUMBER,
                        span,
                        "this number does not fit in an `Int` (64 bits)",
                    );
                    self.err_expr(span)
                }
            },
            ast::ExprKind::Float(text) => {
                let clean: String = text.chars().filter(|c| *c != '_').collect();
                match clean.parse::<f64>() {
                    Ok(value) => self.mk(ExprKind::Float(value), Ty::Float, span),
                    Err(_) => {
                        self.error(codes::BAD_NUMBER, span, "this is not a number");
                        self.err_expr(span)
                    }
                }
            }
            ast::ExprKind::Str(pieces) => self.string(pieces, span),
            ast::ExprKind::Char(c) => match self.ascii(&c.to_string(), span) {
                Some(bytes) => self.mk(ExprKind::Char(bytes[0]), Ty::Char, span),
                None => self.err_expr(span),
            },
            ast::ExprKind::Bool(value) => self.mk(ExprKind::Bool(*value), Ty::Bool, span),
            ast::ExprKind::None => match hint {
                Some(ty @ Ty::Opt(_)) => self.mk(ExprKind::None, ty.clone(), span),
                Some(Ty::Error) => self.err_expr(span),
                _ => {
                    self.error(
                        codes::TYPE_MISMATCH,
                        span,
                        "can't tell which optional this `none` is",
                    );
                    self.help("say the type where it is stored: `let best: ?Int = none`");
                    self.err_expr(span)
                }
            },
            ast::ExprKind::Name(name) => self.name(name, span),
            ast::ExprKind::SelfValue => match self.cx.receiver {
                Some((slot, _)) => self.local(slot, span),
                None => {
                    self.error(codes::UNRESOLVED_NAME, span, "`self` outside a method");
                    self.err_expr(span)
                }
            },
            ast::ExprKind::Paren(inner) => self.expr(inner, hint),
            ast::ExprKind::Unary { op, expr } => self.unary(*op, expr, inner_hint, span),
            ast::ExprKind::Binary { op, lhs, rhs, .. } => self.binary(*op, lhs, rhs, span),
            ast::ExprKind::Compare { first, rest } => self.compare(first, rest, span),
            ast::ExprKind::Range { .. } => {
                self.unsupported(span, "a range kept as a value");
                self.err_expr(span)
            }
            ast::ExprKind::Call { callee, args } => self.call(e, callee, args, hint),
            ast::ExprKind::Index { base, indices } => {
                let base = self.expr(base, None);
                let (indices, ty) = self.index_args(&base.ty, indices, span);
                self.mk(ExprKind::Index(Box::new(base), indices), ty, span)
            }
            ast::ExprKind::Field { base, name } => self.field(e, base, name),
            ast::ExprKind::Generic { .. } => {
                self.unsupported(span, "explicit type arguments here");
                self.err_expr(span)
            }
            ast::ExprKind::Tuple(items) => {
                let hints = inner_hint
                    .and_then(Ty::fields)
                    .filter(|f| f.len() == items.len());
                let values: Vec<Expr> = items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| match &hints {
                        Some(hints) => self.expect(item, &hints[i]),
                        None => self.expr(item, None),
                    })
                    .collect();
                let ty = match (inner_hint, &hints) {
                    (Some(ty @ (Ty::Tuple(_) | Ty::Record(_))), Some(_)) => ty.clone(),
                    _ => Ty::Tuple(values.iter().map(|v| v.ty.clone()).collect()),
                };
                self.mk(ExprKind::Tuple(values), ty, span)
            }
            ast::ExprKind::Record(fields) => {
                let hints = match inner_hint {
                    Some(Ty::Record(want))
                        if want.len() == fields.len()
                            && want.iter().zip(fields).all(|((w, _), (f, _))| *w == f.text) =>
                    {
                        Some(want.clone())
                    }
                    _ => None,
                };
                let values: Vec<Expr> = fields
                    .iter()
                    .enumerate()
                    .map(|(i, (_, value))| match &hints {
                        Some(hints) => self.expect(value, &hints[i].1),
                        None => self.expr(value, None),
                    })
                    .collect();
                let ty = Ty::Record(
                    fields
                        .iter()
                        .zip(&values)
                        .map(|((name, _), value)| (name.text.clone(), value.ty.clone()))
                        .collect(),
                );
                self.mk(ExprKind::Tuple(values), ty, span)
            }
            ast::ExprKind::List(items) => self.list(items, inner_hint, span),
            ast::ExprKind::Comprehension { body, clauses } => {
                self.comprehension(body, clauses, inner_hint, span)
            }
            ast::ExprKind::Table {
                domains,
                params,
                body,
            } => self.table(domains, params, body, inner_hint, span),
            ast::ExprKind::Lambda { .. } => {
                self.unsupported(span, "a fn kept as a value");
                self.err_expr(span)
            }
            ast::ExprKind::Block(block) => self.do_expr(hint, span, |lw, out| {
                out.extend(lw.block(block, Tail::Give).stmts);
            }),
            ast::ExprKind::If {
                branches,
                otherwise,
            } => {
                if let Some(value) = self.ternary(branches, otherwise.as_ref(), hint) {
                    return value;
                }
                self.do_expr(hint, span, |lw, out| lw.tail(e, Tail::Give, out))
            }
            ast::ExprKind::Match { .. } => {
                self.do_expr(hint, span, |lw, out| lw.tail(e, Tail::Give, out))
            }
            ast::ExprKind::With { .. } => {
                self.unsupported(span, "`with`");
                self.err_expr(span)
            }
            ast::ExprKind::Try(_) => {
                self.unsupported(span, "postfix `?`");
                self.err_expr(span)
            }
            ast::ExprKind::Unwrap(inner) => {
                let value = self.expr(inner, None);
                match value.ty.clone() {
                    Ty::Opt(inside) => {
                        self.mk(ExprKind::Unwrap(Box::new(value), true), *inside, span)
                    }
                    Ty::Error => value,
                    other => {
                        let message = format!(
                            "`!` takes the value out of an optional, but this is a `{}`",
                            self.show(&other)
                        );
                        self.error(codes::TYPE_MISMATCH, span, message);
                        self.err_expr(span)
                    }
                }
            }
            ast::ExprKind::Missing | ast::ExprKind::Error => self.err_expr(span),
        }
    }

    /// A block with a value. The first value given decides the type when
    /// the context doesn't.
    pub(super) fn do_expr(
        &mut self,
        hint: Option<&Ty>,
        span: Span,
        body: impl FnOnce(&mut Self, &mut Vec<Stmt>),
    ) -> Expr {
        self.cx.dos.push(DoFrame {
            ty: hint.cloned(),
            loops: self.cx.loops.len(),
        });
        let mut out = Vec::new();
        body(self, &mut out);
        let frame = self.cx.dos.pop().expect("pushed above");
        self.mk(
            ExprKind::Do(Block { stmts: out }),
            frame.ty.unwrap_or(Ty::Error),
            span,
        )
    }

    /// An `if` whose branches are single expressions is a conditional
    /// expression: `if a < b: b - a else: a - b`.
    fn ternary(
        &mut self,
        branches: &[(ast::Cond, ast::Block)],
        otherwise: Option<&ast::Block>,
        hint: Option<&Ty>,
    ) -> Option<Expr> {
        fn single(block: &ast::Block) -> Option<&ast::Expr> {
            match block.stmts.as_slice() {
                [ast::Stmt {
                    kind: ast::StmtKind::Expr(e),
                    ..
                }] if !matches!(
                    e.kind,
                    ast::ExprKind::If { .. }
                        | ast::ExprKind::Match { .. }
                        | ast::ExprKind::Block(_)
                ) =>
                {
                    Some(e)
                }
                _ => None,
            }
        }
        let otherwise = single(otherwise?)?;
        let mut arms = Vec::new();
        for (cond, block) in branches {
            let ast::Cond::Expr(cond) = cond else {
                return None;
            };
            arms.push((cond, single(block)?));
        }
        let mut lowered = Vec::new();
        let mut ty = hint.cloned();
        for (cond, value) in arms {
            let cond = self.expect(cond, &Ty::Bool);
            let value = self.expr(value, ty.as_ref());
            if ty.is_none() {
                ty = Some(value.ty.clone());
            }
            let at = value.span;
            lowered.push((cond, value, at));
        }
        let mut result = self.expr(otherwise, ty.as_ref());
        // The branches agree on one type: the hint, or the widest of them.
        let mut ty = ty.unwrap_or(Ty::Error);
        for candidate in lowered.iter().map(|(_, v, _)| &v.ty).chain([&result.ty]) {
            if *candidate == Ty::Float && ty == Ty::Int {
                ty = Ty::Float;
            }
            if let Ty::Opt(inner) = candidate {
                if **inner == ty {
                    ty = candidate.clone();
                }
            }
        }
        result = self.coerce(result, &ty, otherwise.span);
        for (cond, value, span) in lowered.into_iter().rev() {
            let value = match self.try_coerce(value, &ty) {
                Ok(value) => value,
                Err(value) => {
                    let message = format!(
                        "the branches of this `if` have different types: `{}` and `{}`",
                        self.show(&ty),
                        self.show(&value.ty)
                    );
                    self.diags
                        .push(Diagnostic::error(codes::TYPE_MISMATCH, span, message));
                    Expr {
                        kind: ExprKind::Error,
                        ty: Ty::Error,
                        span,
                    }
                }
            };
            let span = cond.span;
            result = Expr {
                kind: ExprKind::If(Box::new(cond), Box::new(value), Box::new(result)),
                ty: ty.clone(),
                span,
            };
        }
        Some(result)
    }

    fn name(&mut self, name: &str, span: Span) -> Expr {
        if let Some(slot) = self.lookup(name) {
            return self.local(slot, span);
        }
        if let Some((slot, owner)) = self.cx.receiver {
            if let Some((index, ty)) = self.field_of(&Ty::Struct(owner), name) {
                let base = self.local(slot, span);
                return self.mk(ExprKind::Field(Box::new(base), index), ty, span);
            }
        }
        match self.syms.get(name).copied() {
            Some(Sym::Global(id)) => {
                return self.mk(ExprKind::Global(id), self.globals[id].ty.clone(), span)
            }
            Some(Sym::Variant(id, tag)) => {
                if self.enums[id].variants[tag].1.is_empty() {
                    return self.mk(ExprKind::Variant(tag, Vec::new()), Ty::Enum(id), span);
                }
                let message = format!("the variant `{name}` holds values: write `{name}(...)`");
                self.error(codes::WRONG_ARG_COUNT, span, message);
            }
            Some(Sym::Fn(_)) => self.unsupported(span, "a fn kept as a value"),
            Some(Sym::Struct(_) | Sym::Enum(_) | Sym::Domain(_) | Sym::Alias(_)) => {
                let message = format!("`{name}` is a type, and a value is needed here");
                self.error(codes::TYPE_MISMATCH, span, message);
            }
            None if name == "INF" => return self.mk(ExprKind::Int(INF), Ty::Int, span),
            None => {
                let message = format!("`cpl run` doesn't know the name `{name}` here");
                self.error(codes::UNRESOLVED_NAME, span, message);
                self.help("a top-level `let` is visible only after the line that sets it");
            }
        }
        self.err_expr(span)
    }

    /// The bytes of literal text. `Char` is a byte and `Str` is bytes, so
    /// text outside ASCII has no value.
    fn ascii(&mut self, text: &str, span: Span) -> Option<Vec<u8>> {
        match text.chars().find(|c| !c.is_ascii()) {
            None => Some(text.as_bytes().to_vec()),
            Some(c) => {
                let message =
                    format!("`{c}` is not ASCII: a `Char` is one byte and a `Str` is a byte array");
                self.error(codes::NON_ASCII, span, message);
                self.help("only ASCII can be written in a literal; other bytes come from input");
                None
            }
        }
    }

    fn string(&mut self, pieces: &[ast::StrPiece], span: Span) -> Expr {
        let mut out: Vec<FmtPiece> = Vec::new();
        for piece in pieces {
            match piece {
                ast::StrPiece::Text(text) => {
                    let Some(bytes) = self.ascii(text, span) else {
                        return self.err_expr(span);
                    };
                    // `{{` and `}}` are how a brace is written.
                    let mut plain = Vec::with_capacity(bytes.len());
                    let mut i = 0;
                    while i < bytes.len() {
                        plain.push(bytes[i]);
                        let doubled =
                            matches!(bytes[i], b'{' | b'}') && bytes.get(i + 1) == Some(&bytes[i]);
                        i += if doubled { 2 } else { 1 };
                    }
                    match out.last_mut() {
                        Some(FmtPiece::Text(prev)) => prev.extend(plain),
                        _ => out.push(FmtPiece::Text(plain)),
                    }
                }
                ast::StrPiece::Expr(e) => {
                    let value = self.expr(e, None);
                    if let Some(arg) = self.fmt_arg(value, 6, e.span) {
                        out.push(FmtPiece::Value(arg));
                    }
                }
            }
        }
        match out.as_slice() {
            [] => self.mk(ExprKind::Str(Vec::new()), Ty::Str, span),
            [FmtPiece::Text(text)] => self.mk(ExprKind::Str(text.clone()), Ty::Str, span),
            _ => self.mk(ExprKind::Format(out), Ty::Str, span),
        }
    }

    /// A value to print. Everything prints except nothing.
    pub(super) fn fmt_arg(&mut self, value: Expr, digits: u32, span: Span) -> Option<FmtArg> {
        if value.ty == Ty::Unit {
            self.error(codes::TYPE_MISMATCH, span, "this has no value to print");
            return None;
        }
        Some(FmtArg { value, digits })
    }

    /// A `Str` or a `Char` as a `Str`, for `+`.
    pub(super) fn as_str(&mut self, value: Expr, span: Span) -> Expr {
        match value.ty {
            Ty::Str | Ty::Error => value,
            Ty::Char => {
                let at = value.span;
                Expr {
                    kind: ExprKind::Format(vec![FmtPiece::Value(FmtArg { value, digits: 6 })]),
                    ty: Ty::Str,
                    span: at,
                }
            }
            _ => {
                self.mismatch(span, &Ty::Str, &value.ty);
                self.err_expr(span)
            }
        }
    }

    fn unary(&mut self, op: UnOp, operand: &ast::Expr, hint: Option<&Ty>, span: Span) -> Expr {
        match op {
            UnOp::Neg => {
                let value = self.expr(operand, hint);
                match (&value.kind, &value.ty) {
                    (ExprKind::Int(v), _) => self.mk(ExprKind::Int(-*v), Ty::Int, span),
                    (ExprKind::Float(v), _) => self.mk(ExprKind::Float(-*v), Ty::Float, span),
                    (_, Ty::Int | Ty::Float) => {
                        let ty = value.ty.clone();
                        self.mk(ExprKind::Unary(op, Box::new(value)), ty, span)
                    }
                    (_, Ty::Error) => value,
                    _ => {
                        let message =
                            format!("`-` can't be applied to a `{}`", self.show(&value.ty));
                        self.error(codes::BAD_OPERANDS, span, message);
                        self.err_expr(span)
                    }
                }
            }
            UnOp::Not => {
                let value = self.expect(operand, &Ty::Bool);
                self.mk(ExprKind::Unary(op, Box::new(value)), Ty::Bool, span)
            }
            UnOp::BitNot => {
                let value = self.expect(operand, &Ty::Int);
                self.mk(ExprKind::Unary(op, Box::new(value)), Ty::Int, span)
            }
        }
    }

    pub(super) fn cast(&self, cast: Cast, value: Expr, ty: Ty) -> Expr {
        let span = value.span;
        Expr {
            kind: ExprKind::Cast(cast, Box::new(value)),
            ty,
            span,
        }
    }

    fn binary(&mut self, op: BinOp, lhs: &ast::Expr, rhs: &ast::Expr, span: Span) -> Expr {
        use BinOp::*;
        match op {
            And | Or => {
                let l = self.expect(lhs, &Ty::Bool);
                let r = self.expect(rhs, &Ty::Bool);
                return self.mk(
                    ExprKind::Binary(op, Box::new(l), Box::new(r)),
                    Ty::Bool,
                    span,
                );
            }
            Coalesce => {
                let l = self.expr(lhs, None);
                let Ty::Opt(inner) = l.ty.clone() else {
                    if l.ty != Ty::Error {
                        let message = format!(
                            "`??` gives a default for an optional, but the left side is a `{}`",
                            self.show(&l.ty)
                        );
                        self.error(codes::BAD_OPERANDS, span, message);
                    }
                    self.expr(rhs, None);
                    return self.err_expr(span);
                };
                let r = self.expr(rhs, Some(&l.ty));
                // `a ?? b ?? 0`: an optional default keeps the result optional.
                let ty = if r.ty == l.ty { l.ty.clone() } else { *inner };
                let r = self.coerce(r, &ty, rhs.span);
                return self.mk(ExprKind::Binary(op, Box::new(l), Box::new(r)), ty, span);
            }
            _ => {}
        }
        let l = self.expr(lhs, None);
        let r = self.expr(rhs, None);
        let arith = matches!(op, Add | Sub | Mul | Div | Pow);
        let (l, r, ty) = match (&l.ty, &r.ty) {
            (Ty::Error, _) | (_, Ty::Error) => return self.err_expr(span),
            (Ty::Int, Ty::Int) => (l, r, Ty::Int),
            (Ty::Float, Ty::Float) if arith => (l, r, Ty::Float),
            (Ty::Float, Ty::Int) if arith => {
                let r = self.coerce(r, &Ty::Float, rhs.span);
                (l, r, Ty::Float)
            }
            (Ty::Int, Ty::Float) if arith => {
                let l = self.coerce(l, &Ty::Float, lhs.span);
                (l, r, Ty::Float)
            }
            (Ty::Bool, Ty::Bool) if matches!(op, BitAnd | BitOr | BitXor) => (l, r, Ty::Bool),
            (Ty::Str | Ty::Char, Ty::Str) | (Ty::Str, Ty::Char) if op == Add => {
                let l = self.as_str(l, lhs.span);
                let r = self.as_str(r, rhs.span);
                (l, r, Ty::Str)
            }
            (Ty::Char, Ty::Char) if op == Sub => {
                let l = self.cast(Cast::CharToInt, l, Ty::Int);
                let r = self.cast(Cast::CharToInt, r, Ty::Int);
                (l, r, Ty::Int)
            }
            (Ty::Char, Ty::Int) if matches!(op, Add | Sub) => {
                let l = self.cast(Cast::CharToInt, l, Ty::Int);
                let sum = self.mk(
                    ExprKind::Binary(op, Box::new(l), Box::new(r)),
                    Ty::Int,
                    span,
                );
                return self.cast(Cast::IntToChar, sum, Ty::Char);
            }
            (Ty::Pos(a), Ty::Pos(b)) | (Ty::Bound(a), Ty::Bound(b)) if a == b && op == Sub => {
                let l = self.cast(Cast::Ord, l, Ty::Int);
                let r = self.cast(Cast::Ord, r, Ty::Int);
                (l, r, Ty::Int)
            }
            (Ty::Pos(_), Ty::Int) if matches!(op, Add | Sub) => {
                self.error(
                    codes::POSITION_ARITHMETIC,
                    span,
                    "a position plus a number has no single meaning",
                );
                self.help("`v.next` and `v.prev` step by one and are `none` at the ends");
                return self.err_expr(span);
            }
            _ => {
                let message = format!(
                    "`{}` can't be applied to `{}` and `{}`",
                    op.text(),
                    self.show(&l.ty),
                    self.show(&r.ty)
                );
                self.error(codes::BAD_OPERANDS, span, message);
                return self.err_expr(span);
            }
        };
        self.mk(ExprKind::Binary(op, Box::new(l), Box::new(r)), ty, span)
    }

    /// Whether `<` means something for the type.
    pub(super) fn ordered(&self, ty: &Ty) -> bool {
        match ty {
            Ty::Int | Ty::Float | Ty::Bool | Ty::Char | Ty::Str | Ty::Pos(_) | Ty::Bound(_) => true,
            Ty::Tuple(items) => items.iter().all(|t| self.ordered(t)),
            Ty::Record(fields) => fields.iter().all(|(_, t)| self.ordered(t)),
            Ty::List(elem) => self.ordered(elem),
            Ty::Struct(id) => {
                let fields: Vec<Ty> = self.structs[*id]
                    .def
                    .fields
                    .iter()
                    .map(|(_, t)| t.clone())
                    .collect();
                fields.iter().all(|t| self.ordered(t))
            }
            Ty::Error => true,
            _ => false,
        }
    }

    fn compare(&mut self, first: &ast::Expr, rest: &[(CmpOp, ast::Expr)], span: Span) -> Expr {
        if let [(op @ (CmpOp::In | CmpOp::NotIn), container)] = rest {
            let found = self.contains(first, container, span);
            return if *op == CmpOp::In {
                found
            } else {
                self.mk(ExprKind::Unary(UnOp::Not, Box::new(found)), Ty::Bool, span)
            };
        }
        // `none == x` needs the other side's type first.
        let mut operands: Vec<(Expr, Span)> = Vec::new();
        let asts: Vec<&ast::Expr> = std::iter::once(first)
            .chain(rest.iter().map(|(_, e)| e))
            .collect();
        for (i, e) in asts.iter().enumerate() {
            let value = if matches!(e.kind, ast::ExprKind::None) {
                match (operands.last(), asts.get(i + 1)) {
                    (Some((prev, _)), _) => {
                        let hint = Ty::opt(strip_opt(&prev.ty));
                        self.expr(e, Some(&hint))
                    }
                    (None, Some(next)) => {
                        let next = self.expr(next, None);
                        let hint = Ty::opt(strip_opt(&next.ty));
                        self.expr(e, Some(&hint))
                    }
                    _ => self.expr(e, None),
                }
            } else {
                self.expr(e, None)
            };
            operands.push((value, e.span));
        }
        if rest.len() == 1 {
            let (r, r_span) = operands.pop().expect("two operands");
            let (l, l_span) = operands.pop().expect("two operands");
            return self.cmp(rest[0].0, l, l_span, r, r_span, span);
        }
        // `lo <= x < hi`: each operand is evaluated once, left to right.
        let mut stmts = Vec::new();
        let held: Vec<(Expr, Span)> = operands
            .into_iter()
            .map(|(value, at)| {
                let atom = matches!(
                    value.kind,
                    ExprKind::Local(_)
                        | ExprKind::Global(_)
                        | ExprKind::Int(_)
                        | ExprKind::Char(_)
                        | ExprKind::Float(_)
                );
                if atom {
                    (value, at)
                } else {
                    (self.hold(value, &mut stmts), at)
                }
            })
            .collect();
        let mut all: Option<Expr> = None;
        for (i, (op, _)) in rest.iter().enumerate() {
            let (l, l_span) = held[i].clone();
            let (r, r_span) = held[i + 1].clone();
            let one = self.cmp(*op, l, l_span, r, r_span, span);
            all = Some(match all {
                None => one,
                Some(prev) => self.mk(
                    ExprKind::Binary(BinOp::And, Box::new(prev), Box::new(one)),
                    Ty::Bool,
                    span,
                ),
            });
        }
        let all = all.expect("a comparison has an operator");
        if stmts.is_empty() {
            return all;
        }
        stmts.push(Stmt::Give(all));
        self.mk(ExprKind::Do(Block { stmts }), Ty::Bool, span)
    }

    fn cmp(&mut self, op: CmpOp, l: Expr, l_span: Span, r: Expr, r_span: Span, span: Span) -> Expr {
        if l.ty == Ty::Error || r.ty == Ty::Error {
            return self.err_expr(span);
        }
        match op {
            CmpOp::In | CmpOp::NotIn => {
                self.unsupported(span, "`in` inside a chain of comparisons");
                return self.err_expr(span);
            }
            CmpOp::Order => {
                self.unsupported(span, "`<=>`");
                return self.err_expr(span);
            }
            _ => {}
        }
        // Bring both sides to one type.
        let (l, r) = if l.ty == r.ty {
            (l, r)
        } else {
            let want = match (&l.ty, &r.ty) {
                (Ty::Float, Ty::Int) | (Ty::Int, Ty::Float) => Ty::Float,
                (Ty::Opt(_), _) => l.ty.clone(),
                (_, Ty::Opt(_)) => r.ty.clone(),
                _ => l.ty.clone(),
            };
            let l = match self.try_coerce(l, &want) {
                Ok(l) => l,
                Err(l) => {
                    let message = format!(
                        "can't compare a `{}` with a `{}`",
                        self.show(&l.ty),
                        self.show(&r.ty)
                    );
                    let code = match (&l.ty, &r.ty) {
                        (Ty::Pos(_), Ty::Pos(_)) => codes::WRONG_DOMAIN,
                        _ => codes::TYPE_MISMATCH,
                    };
                    self.error(code, span, message);
                    return self.err_expr(span);
                }
            };
            let r = match self.try_coerce(r, &want) {
                Ok(r) => r,
                Err(r) => {
                    let message = format!(
                        "can't compare a `{}` with a `{}`",
                        self.show(&l.ty),
                        self.show(&r.ty)
                    );
                    let code = match (&l.ty, &r.ty) {
                        (Ty::Pos(_), Ty::Pos(_)) => codes::WRONG_DOMAIN,
                        _ => codes::TYPE_MISMATCH,
                    };
                    self.error(code, span, message);
                    return self.err_expr(span);
                }
            };
            (l, r)
        };
        let _ = (l_span, r_span);
        if !matches!(op, CmpOp::Eq | CmpOp::Ne) {
            if matches!(l.ty, Ty::Opt(_)) {
                self.error(
                    codes::OPTIONAL_COMPARE,
                    span,
                    "an optional has no order: `none` is neither small nor large",
                );
                self.help("give it a value first: `(x ?? INF) < y`");
                return self.err_expr(span);
            }
            if !self.ordered(&l.ty) {
                let message = format!("a `{}` has no order to compare by", self.show(&l.ty));
                self.error(codes::BAD_OPERANDS, span, message);
                return self.err_expr(span);
            }
        }
        self.mk(ExprKind::Cmp(op, Box::new(l), Box::new(r)), Ty::Bool, span)
    }

    /// `x in c`: a range, a domain, or anything that holds values.
    fn contains(&mut self, item: &ast::Expr, container: &ast::Expr, span: Span) -> Expr {
        let mut inner = container;
        while let ast::ExprKind::Paren(e) = &inner.kind {
            inner = e;
        }
        let between = |lw: &mut Self, value: Expr, lo: Expr, hi: Expr| {
            // `lo <= x and x < hi`, with `x` evaluated once.
            let mut stmts = Vec::new();
            let value = lw.hold(value, &mut stmts);
            let ge = lw.mk(
                ExprKind::Cmp(CmpOp::Le, Box::new(lo), Box::new(value.clone())),
                Ty::Bool,
                span,
            );
            let lt = lw.mk(
                ExprKind::Cmp(CmpOp::Lt, Box::new(value), Box::new(hi)),
                Ty::Bool,
                span,
            );
            let both = lw.mk(
                ExprKind::Binary(BinOp::And, Box::new(ge), Box::new(lt)),
                Ty::Bool,
                span,
            );
            if stmts.is_empty() {
                return both;
            }
            stmts.push(Stmt::Give(both));
            lw.mk(ExprKind::Do(Block { stmts }), Ty::Bool, span)
        };
        match &inner.kind {
            ast::ExprKind::Range {
                lo: Some(lo),
                hi: Some(hi),
                inclusive,
                step: None,
            } => {
                let value = self.expect(item, &Ty::Int);
                let lo = self.expect(lo, &Ty::Int);
                let mut hi = self.expect(hi, &Ty::Int);
                if *inclusive {
                    hi = self.int_add(hi, 1);
                }
                between(self, value, lo, hi)
            }
            ast::ExprKind::Name(name)
                if self.lookup(name).is_none()
                    && matches!(self.syms.get(name), Some(Sym::Domain(_))) =>
            {
                let Some(Sym::Domain(id)) = self.syms.get(name).copied() else {
                    unreachable!("matched above")
                };
                let value = self.expect(item, &Ty::Int);
                let lo = self.mk(ExprKind::DomainLo(id), Ty::Int, span);
                let len = self.mk(ExprKind::DomainLen(id), Ty::Int, span);
                let hi = self.mk(
                    ExprKind::Binary(BinOp::Add, Box::new(lo.clone()), Box::new(len)),
                    Ty::Int,
                    span,
                );
                between(self, value, lo, hi)
            }
            _ => {
                let holder = self.expr(container, None);
                let elem = match &holder.ty {
                    Ty::List(elem) | Ty::Set(elem) => (**elem).clone(),
                    Ty::Array { elem, .. } => (**elem).clone(),
                    Ty::Map(key, _) => (**key).clone(),
                    Ty::Str => Ty::Char,
                    Ty::Error => return self.err_expr(span),
                    other => {
                        let message = format!("`in` can't look inside a `{}`", self.show(other));
                        self.error(codes::BAD_OPERANDS, span, message);
                        return self.err_expr(span);
                    }
                };
                let value = self.expect(item, &elem);
                self.mk(
                    ExprKind::Builtin(Builtin::Contains, vec![holder, value]),
                    Ty::Bool,
                    span,
                )
            }
        }
    }

    /// The indices of `base[...]` and the type of the element.
    pub(super) fn index_args(
        &mut self,
        base: &Ty,
        indices: &[ast::Expr],
        span: Span,
    ) -> (Vec<Expr>, Ty) {
        let one = |lw: &mut Self, want: &Ty| -> Vec<Expr> {
            if indices.len() != 1 {
                let message = format!("this takes 1 index, but {} were given", indices.len());
                lw.error(codes::WRONG_ARG_COUNT, span, message);
                return vec![lw.err_expr(span)];
            }
            vec![lw.expect(&indices[0], want)]
        };
        match base {
            Ty::Array { dims, elem } => {
                if dims.len() != indices.len() {
                    let message = format!(
                        "a `{}` takes {} indices, but {} were given",
                        self.show(base),
                        dims.len(),
                        indices.len()
                    );
                    self.error(codes::WRONG_ARG_COUNT, span, message);
                    if dims.len() > 1 && indices.len() == 1 {
                        self.help("a grid is indexed `g[r, c]`, never `g[r][c]`");
                    }
                    return (vec![self.err_expr(span)], (**elem).clone());
                }
                let mut out = Vec::new();
                for (dim, index) in dims.iter().zip(indices) {
                    let want = match dim {
                        Dim::Pos(d) => Ty::Pos(*d),
                        Dim::Bound(d) => Ty::Bound(*d),
                        Dim::Int => Ty::Int,
                    };
                    let value = self.expr(index, Some(&want));
                    if value.ty != want && value.ty != Ty::Error {
                        let message = format!(
                            "this is indexed by `{}`, but the index is a `{}`",
                            self.show(&want),
                            self.show(&value.ty)
                        );
                        self.error(codes::WRONG_DOMAIN, index.span, message);
                    }
                    out.push(value);
                }
                (out, (**elem).clone())
            }
            Ty::List(elem) => (one(self, &Ty::Int), (**elem).clone()),
            Ty::Str => (one(self, &Ty::Int), Ty::Char),
            Ty::Map(key, value) => (one(self, key), (**value).clone()),
            Ty::Error => {
                for index in indices {
                    self.expr(index, None);
                }
                (vec![self.err_expr(span)], Ty::Error)
            }
            other => {
                let message = format!("a `{}` can't be indexed", self.show(other));
                self.error(codes::NOT_INDEXABLE, span, message);
                (vec![self.err_expr(span)], Ty::Error)
            }
        }
    }

    fn list(&mut self, items: &[ast::Expr], hint: Option<&Ty>, span: Span) -> Expr {
        match hint {
            Some(ty @ (Ty::List(elem) | Ty::Set(elem))) => {
                let values = items.iter().map(|item| self.expect(item, elem)).collect();
                self.mk(ExprKind::Tuple(values), ty.clone(), span)
            }
            Some(ty @ Ty::Map(..)) if items.is_empty() => {
                self.mk(ExprKind::Tuple(Vec::new()), ty.clone(), span)
            }
            _ => {
                let Some((first, rest)) = items.split_first() else {
                    self.error(
                        codes::TYPE_MISMATCH,
                        span,
                        "can't tell what this empty list holds",
                    );
                    self.help("say the type where it is stored: `var xs: List<Int> = []`");
                    return self.err_expr(span);
                };
                let first = self.expr(first, None);
                let elem = first.ty.clone();
                let mut values = vec![first];
                for item in rest {
                    values.push(self.expect(item, &elem));
                }
                self.mk(ExprKind::Tuple(values), Ty::list(elem), span)
            }
        }
    }

    /// What a `for` walks over, and the type of each value.
    pub(super) fn iter(&mut self, e: &ast::Expr) -> (Iter, Ty) {
        let span = e.span;
        let error = |lw: &mut Self| {
            let zero = lw.int_lit(0, span);
            (
                Iter::Range {
                    lo: zero.clone(),
                    hi: zero,
                    step: None,
                    rev: false,
                },
                Ty::Error,
            )
        };
        match &e.kind {
            ast::ExprKind::Paren(inner) => self.iter(inner),
            ast::ExprKind::Range {
                lo: Some(lo),
                hi: Some(hi),
                inclusive,
                step,
            } => {
                let lo = self.expr(lo, None);
                let ty = lo.ty.clone();
                if !matches!(ty, Ty::Int | Ty::Pos(_) | Ty::Bound(_) | Ty::Error) {
                    let message = format!(
                        "a range runs over numbers or positions, not `{}`",
                        self.show(&ty)
                    );
                    self.error(codes::TYPE_MISMATCH, span, message);
                    return error(self);
                }
                let mut hi = self.expect(hi, &ty);
                let mut lo = lo;
                if ty != Ty::Int {
                    lo = self.cast(Cast::Ord, lo, Ty::Int);
                    hi = self.cast(Cast::Ord, hi, Ty::Int);
                }
                if *inclusive {
                    hi = self.int_add(hi, 1);
                }
                let step = step.as_ref().map(|s| self.expect(s, &Ty::Int));
                (
                    Iter::Range {
                        lo,
                        hi,
                        step,
                        rev: false,
                    },
                    ty,
                )
            }
            ast::ExprKind::Range { .. } => {
                self.unsupported(span, "a range without both ends in a `for`");
                error(self)
            }
            ast::ExprKind::Name(name)
                if self.lookup(name).is_none()
                    && matches!(self.syms.get(name), Some(Sym::Domain(_))) =>
            {
                let Some(Sym::Domain(id)) = self.syms.get(name).copied() else {
                    unreachable!("matched above")
                };
                let lo = self.int_lit(0, span);
                let hi = self.mk(ExprKind::DomainLen(id), Ty::Int, span);
                (
                    Iter::Range {
                        lo,
                        hi,
                        step: None,
                        rev: false,
                    },
                    Ty::Pos(id),
                )
            }
            ast::ExprKind::Field { base, name } if name.text == "rev" => {
                let (iter, ty) = self.iter(base);
                let iter = match iter {
                    Iter::Range { lo, hi, step, rev } => Iter::Range {
                        lo,
                        hi,
                        step,
                        rev: !rev,
                    },
                    Iter::Seq(value, rev) => Iter::Seq(value, !rev),
                };
                (iter, ty)
            }
            ast::ExprKind::Field { base, name } if name.text == "bounds" => {
                let domain = match &base.kind {
                    ast::ExprKind::Name(n) if self.lookup(n).is_none() => match self.syms.get(n) {
                        Some(Sym::Domain(id)) => Some(*id),
                        _ => None,
                    },
                    _ => None,
                };
                let Some(id) = domain else {
                    self.error(
                        codes::UNKNOWN_FIELD,
                        name.span,
                        "`.bounds` belongs to a domain: `V.bounds`",
                    );
                    return error(self);
                };
                let lo = self.int_lit(0, span);
                let len = self.mk(ExprKind::DomainLen(id), Ty::Int, span);
                let hi = self.int_add(len, 1);
                (
                    Iter::Range {
                        lo,
                        hi,
                        step: None,
                        rev: false,
                    },
                    Ty::Bound(id),
                )
            }
            _ => {
                let value = self.expr(e, None);
                let elem = match &value.ty {
                    Ty::Array { elem, .. } | Ty::List(elem) | Ty::Set(elem) => (**elem).clone(),
                    Ty::Str => Ty::Char,
                    Ty::Map(key, val) => Ty::Tuple(vec![(**key).clone(), (**val).clone()]),
                    Ty::Error => return error(self),
                    Ty::Int => {
                        self.error(
                            codes::TYPE_MISMATCH,
                            span,
                            "a `for` can't walk over an `Int`",
                        );
                        self.help("count with a range: `for i in 0..n`");
                        return error(self);
                    }
                    other => {
                        let message = format!("a `for` can't walk over a `{}`", self.show(other));
                        self.error(codes::TYPE_MISMATCH, span, message);
                        return error(self);
                    }
                };
                (Iter::Seq(value, false), elem)
            }
        }
    }

    /// Opens the loops of a comprehension. Its body is lowered next, with
    /// the loop variables in scope, and [`Self::close_loops`] wraps it.
    pub(super) fn open_loops(&mut self, clauses: &[ast::Clause]) -> Vec<LoopFrame> {
        let mut frames = Vec::new();
        for clause in clauses {
            let (iter, elem) = self.iter(&clause.iter);
            let id = self.next_loop;
            self.next_loop += 1;
            self.cx.scopes.push(HashMap::new());
            let mut pre = Vec::new();
            let slot = self.loop_var(&clause.pat, elem, &mut pre);
            let cond = clause.cond.as_ref().map(|c| self.expect(c, &Ty::Bool));
            frames.push(LoopFrame {
                id,
                slot,
                iter,
                pre,
                cond,
            });
        }
        frames
    }

    /// One loop over the values of `seq`, shaped like a comprehension's.
    pub(super) fn open_seq(&mut self, seq: Expr, elem: Ty) -> (Vec<LoopFrame>, Expr) {
        let id = self.next_loop;
        self.next_loop += 1;
        self.cx.scopes.push(HashMap::new());
        let span = seq.span;
        let slot = self.temp(elem.clone());
        let value = Expr {
            kind: ExprKind::Local(slot),
            ty: elem,
            span,
        };
        let frame = LoopFrame {
            id,
            slot,
            iter: Iter::Seq(seq, false),
            pre: Vec::new(),
            cond: None,
        };
        (vec![frame], value)
    }

    /// Wraps `inner` in the loops; gives the statements and the id of the
    /// outermost loop, for an early exit.
    pub(super) fn close_loops(
        &mut self,
        frames: Vec<LoopFrame>,
        inner: Vec<Stmt>,
    ) -> (Vec<Stmt>, LoopId) {
        let outer = frames.first().map_or(0, |f| f.id);
        let mut stmts = inner;
        for frame in frames.into_iter().rev() {
            self.cx.scopes.pop();
            let mut body = frame.pre;
            match frame.cond {
                Some(cond) => body.push(Stmt::If(cond, Block { stmts }, Block::default())),
                None => body.extend(stmts),
            }
            stmts = vec![Stmt::For(
                frame.id,
                frame.slot,
                frame.iter,
                Block { stmts: body },
            )];
        }
        (stmts, outer)
    }

    /// `[body for x in xs if cond]`: a list, or a set where one is wanted.
    fn comprehension(
        &mut self,
        body: &ast::Expr,
        clauses: &[ast::Clause],
        hint: Option<&Ty>,
        span: Span,
    ) -> Expr {
        let frames = self.open_loops(clauses);
        let (elem_hint, is_set) = match hint {
            Some(Ty::List(elem)) => (Some((**elem).clone()), false),
            Some(Ty::Set(elem)) => (Some((**elem).clone()), true),
            _ => (None, false),
        };
        let value = match &elem_hint {
            Some(elem) => self.expect(body, elem),
            None => self.expr(body, None),
        };
        let ty = if is_set {
            Ty::Set(Box::new(value.ty.clone()))
        } else {
            Ty::list(value.ty.clone())
        };
        let acc = self.temp(ty.clone());
        let at = self.span(span);
        let place = Place {
            kind: PlaceKind::Local(acc),
            ty: ty.clone(),
            span: at,
        };
        let op = if is_set {
            Mutation::Insert
        } else {
            Mutation::Push
        };
        let result_ty = if is_set { Ty::Bool } else { Ty::Unit };
        let add = Expr {
            kind: ExprKind::Mutate(op, Box::new(place), vec![value]),
            ty: result_ty,
            span: at,
        };
        let (loops, _) = self.close_loops(frames, vec![Stmt::Expr(add)]);
        let mut stmts = vec![Stmt::Let(
            acc,
            Some(self.mk(ExprKind::Tuple(Vec::new()), ty.clone(), span)),
        )];
        stmts.extend(loops);
        stmts.push(Stmt::Give(self.local(acc, span)));
        self.mk(ExprKind::Do(Block { stmts }), ty, span)
    }

    /// `[deg[u] * 2 for u in V]` where a `[V]T` is declared builds the
    /// array, like the table `[V] u => deg[u] * 2`.
    pub(super) fn comprehension_table(
        &mut self,
        init: &ast::Expr,
        dims: &[DimSpec],
        elem: &Ty,
    ) -> Option<Expr> {
        let ast::ExprKind::Comprehension { body, clauses } = &init.kind else {
            return None;
        };
        let ([clause], [DimSpec::Pos(dim)]) = (clauses.as_slice(), dims) else {
            return None;
        };
        let ast::ExprKind::Name(domain) = &clause.iter.kind else {
            return None;
        };
        if clause.cond.is_some() || self.lookup(domain).is_some() {
            return None;
        }
        match self.syms.get(domain) {
            Some(Sym::Domain(id)) if id == dim => {}
            _ => return None,
        }
        let pats = std::slice::from_ref(&clause.pat);
        Some(self.build_table(dims.to_vec(), pats, body, Some(elem), init.span))
    }

    fn table(
        &mut self,
        domains: &[ast::Type],
        params: &[ast::Pattern],
        body: &ast::Expr,
        hint: Option<&Ty>,
        span: Span,
    ) -> Expr {
        let dims: Option<Vec<DimSpec>> = domains.iter().map(|d| self.dim(d, true)).collect();
        let Some(dims) = dims else {
            return self.err_expr(span);
        };
        let elem = match hint {
            Some(Ty::Array { elem, .. }) => Some((**elem).clone()),
            _ => None,
        };
        self.build_table(dims, params, body, elem.as_ref(), span)
    }

    fn build_table(
        &mut self,
        dims: Vec<DimSpec>,
        params: &[ast::Pattern],
        body: &ast::Expr,
        elem: Option<&Ty>,
        span: Span,
    ) -> Expr {
        if params.len() != dims.len() {
            let message = format!(
                "this table runs over {} domains but names {} positions",
                dims.len(),
                params.len()
            );
            self.error(codes::WRONG_ARG_COUNT, span, message);
            return self.err_expr(span);
        }
        let at = self.span(span);
        let mut stmts = Vec::new();
        // A length is used to allocate and again to loop: hold it.
        let dims: Vec<DimSpec> = dims
            .into_iter()
            .map(|dim| match dim {
                DimSpec::Int(len) => DimSpec::Int(Box::new(self.hold(*len, &mut stmts))),
                other => other,
            })
            .collect();
        self.cx.scopes.push(HashMap::new());
        let mut loops = Vec::new();
        let mut indices = Vec::new();
        for (dim, param) in dims.iter().zip(params) {
            let (ty, hi) = match dim {
                DimSpec::Pos(d) => (Ty::Pos(*d), self.mk(ExprKind::DomainLen(*d), Ty::Int, span)),
                DimSpec::Bound(d) => {
                    let len = self.mk(ExprKind::DomainLen(*d), Ty::Int, span);
                    (Ty::Bound(*d), self.int_add(len, 1))
                }
                DimSpec::Int(len) => (Ty::Int, (**len).clone()),
            };
            let mut pre = Vec::new();
            let slot = self.loop_var(param, ty.clone(), &mut pre);
            let id = self.next_loop;
            self.next_loop += 1;
            let iter = Iter::Range {
                lo: self.int_lit(0, span),
                hi,
                step: None,
                rev: false,
            };
            indices.push(Expr {
                kind: ExprKind::Local(slot),
                ty,
                span: at,
            });
            loops.push((id, slot, iter, pre));
        }
        let value = match elem {
            Some(elem) => self.expect(body, elem),
            None => self.expr(body, None),
        };
        self.cx.scopes.pop();
        let ty = Ty::Array {
            dims: dims.iter().map(DimSpec::dim).collect(),
            elem: Box::new(value.ty.clone()),
        };
        let acc = self.temp(ty.clone());
        let cell = Place {
            kind: PlaceKind::Index(
                Box::new(Place {
                    kind: PlaceKind::Local(acc),
                    ty: ty.clone(),
                    span: at,
                }),
                indices,
            ),
            ty: value.ty.clone(),
            span: at,
        };
        let mut inner = vec![Stmt::Assign(cell, AssignOp::Set, value)];
        for (id, slot, iter, mut pre) in loops.into_iter().rev() {
            pre.extend(inner);
            inner = vec![Stmt::For(id, slot, iter, Block { stmts: pre })];
        }
        stmts.push(Stmt::Let(
            acc,
            Some(self.mk(ExprKind::Fill(dims, None), ty.clone(), span)),
        ));
        stmts.extend(inner);
        stmts.push(Stmt::Give(self.local(acc, span)));
        self.mk(ExprKind::Do(Block { stmts }), ty, span)
    }
}

fn strip_opt(ty: &Ty) -> Ty {
    match ty {
        Ty::Opt(inner) => (**inner).clone(),
        other => other.clone(),
    }
}
