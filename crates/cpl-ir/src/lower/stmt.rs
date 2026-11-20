//! Statements, blocks and patterns.

use super::*;

impl<'a> Lowerer<'a> {
    pub(super) fn block(&mut self, block: &ast::Block, tail: Tail) -> Block {
        self.cx.scopes.push(HashMap::new());
        let mut out = Vec::new();
        match block.stmts.split_last() {
            None if tail == Tail::Give => self.needs_value(block.span),
            None => {}
            Some((last, init)) => {
                for stmt in init {
                    self.stmt(stmt, &mut out);
                }
                match &last.kind {
                    ast::StmtKind::Expr(e) if tail != Tail::Discard => self.tail(e, tail, &mut out),
                    _ => {
                        self.stmt(last, &mut out);
                        if tail == Tail::Give {
                            self.needs_value(last.span);
                        }
                    }
                }
            }
        }
        self.cx.scopes.pop();
        Block { stmts: out }
    }

    fn needs_value(&mut self, span: Span) {
        self.error(
            codes::TYPE_MISMATCH,
            span,
            "this block is used as a value, so it has to end with an expression",
        );
    }

    /// Lowers `e` where its value goes to `tail`. `if`, `match` and inline
    /// blocks pass the destination on to their branches, so a fn that ends
    /// in an `if` returns from each branch.
    pub(super) fn tail(&mut self, e: &ast::Expr, tail: Tail, out: &mut Vec<Stmt>) {
        match &e.kind {
            ast::ExprKind::Paren(inner)
                if matches!(
                    inner.kind,
                    ast::ExprKind::If { .. }
                        | ast::ExprKind::Match { .. }
                        | ast::ExprKind::Block(_)
                ) =>
            {
                self.tail(inner, tail, out)
            }
            ast::ExprKind::If {
                branches,
                otherwise,
            } => match (otherwise, tail) {
                (None, Tail::Give) => {
                    self.error(
                        codes::TYPE_MISMATCH,
                        e.span,
                        "an `if` used as a value needs an `else`",
                    );
                }
                // Without an `else` there is no value on one path: a
                // statement, and the fn's value comes later.
                (None, _) => self.if_chain(branches, None, Tail::Discard, out),
                (Some(otherwise), _) => self.if_chain(branches, Some(otherwise), tail, out),
            },
            ast::ExprKind::Match { scrutinee, arms } => {
                self.match_stmt(e, scrutinee, arms, tail, out)
            }
            ast::ExprKind::Block(block) => out.extend(self.block(block, tail).stmts),
            _ => match tail {
                Tail::Discard => {
                    let value = self.expr(e, None);
                    out.push(Stmt::Expr(value));
                }
                Tail::Return => {
                    let hint = self.cx.ret.clone();
                    let value = self.expr(e, hint.as_ref());
                    match hint {
                        Some(Ty::Unit) => out.push(Stmt::Expr(value)),
                        Some(ty) => {
                            let value = self.coerce(value, &ty, e.span);
                            out.push(Stmt::Return(Some(value)));
                        }
                        None => {
                            self.cx.ret = Some(value.ty.clone());
                            if value.ty == Ty::Unit {
                                out.push(Stmt::Expr(value));
                            } else {
                                out.push(Stmt::Return(Some(value)));
                            }
                        }
                    }
                }
                Tail::Give => {
                    let hint = self.cx.dos.last().and_then(|frame| frame.ty.clone());
                    let mut value = self.expr(e, hint.as_ref());
                    match hint {
                        Some(ty) => value = self.coerce(value, &ty, e.span),
                        None => {
                            if let Some(frame) = self.cx.dos.last_mut() {
                                frame.ty = Some(value.ty.clone());
                            }
                        }
                    }
                    out.push(Stmt::Give(value));
                }
            },
        }
    }

    fn at_top_level(&self) -> bool {
        self.cx.is_main && self.cx.scopes.len() == 1
    }

    pub(super) fn stmt(&mut self, stmt: &ast::Stmt, out: &mut Vec<Stmt>) {
        match &stmt.kind {
            ast::StmtKind::Let { pat, ty, init, .. } => {
                if let Some(value) = self.let_value(ty.as_ref(), init.as_ref(), stmt.span) {
                    self.bind(pat, value, out);
                }
            }
            ast::StmtKind::Const { name, ty, init } => {
                if let Some(value) = self.let_value(ty.as_ref(), init.as_ref(), stmt.span) {
                    self.declare(&name.text, value, out);
                }
            }
            ast::StmtKind::Fn(decl) if !self.at_top_level() => {
                self.unsupported(decl.name.span, "a fn inside another fn or block")
            }
            ast::StmtKind::Struct(_)
            | ast::StmtKind::Enum(_)
            | ast::StmtKind::TypeAlias { .. }
            | ast::StmtKind::Trait { .. }
            | ast::StmtKind::Impl { .. }
                if !self.at_top_level() =>
            {
                self.unsupported(stmt.span, "a type declared inside a fn or block")
            }
            // Top-level declarations were handled by `collect`.
            ast::StmtKind::Fn(_)
            | ast::StmtKind::Struct(_)
            | ast::StmtKind::Enum(_)
            | ast::StmtKind::TypeAlias { .. }
            | ast::StmtKind::Trait { .. }
            | ast::StmtKind::Impl { .. }
            | ast::StmtKind::Use { .. } => {}
            ast::StmtKind::Domain { name, def } => self.domain(name, def, out),
            ast::StmtKind::Expr(e) => self.tail(e, Tail::Discard, out),
            ast::StmtKind::Assign { target, op, value } => self.assign(target, *op, value, out),
            ast::StmtKind::While { cond, body } => self.while_stmt(cond, body, out),
            ast::StmtKind::For {
                label,
                pat,
                iter,
                body,
            } => {
                let (iter, elem) = self.iter(iter);
                let id = self.open_loop(label.as_ref().map(|l| l.text.clone()));
                self.cx.scopes.push(HashMap::new());
                let mut stmts = Vec::new();
                let slot = self.loop_var(pat, elem, &mut stmts);
                stmts.extend(self.block(body, Tail::Discard).stmts);
                self.cx.scopes.pop();
                self.cx.loops.pop();
                out.push(Stmt::For(id, slot, iter, Block { stmts }));
            }
            ast::StmtKind::Return(value) => {
                if !self.cx.dos.is_empty() {
                    self.unsupported(stmt.span, "`return` inside a block that is used as a value");
                    return;
                }
                match value {
                    None => {
                        match &self.cx.ret {
                            None => self.cx.ret = Some(Ty::Unit),
                            Some(Ty::Unit | Ty::Error) => {}
                            Some(ty) => {
                                let ty = ty.clone();
                                self.mismatch(stmt.span, &ty, &Ty::Unit);
                            }
                        }
                        out.push(Stmt::Return(None));
                    }
                    Some(e) => {
                        let hint = self.cx.ret.clone();
                        let mut value = self.expr(e, hint.as_ref());
                        match hint {
                            Some(ty) => value = self.coerce(value, &ty, e.span),
                            None => self.cx.ret = Some(value.ty.clone()),
                        }
                        out.push(Stmt::Return(Some(value)));
                    }
                }
            }
            ast::StmtKind::Break(label) | ast::StmtKind::Continue(label) => {
                let is_break = matches!(stmt.kind, ast::StmtKind::Break(_));
                let word = if is_break { "break" } else { "continue" };
                let found = match label {
                    None => self.cx.loops.len().checked_sub(1),
                    Some(label) => self
                        .cx
                        .loops
                        .iter()
                        .rposition(|(name, _)| name.as_deref() == Some(label.text.as_str())),
                };
                let Some(index) = found else {
                    let message = match label {
                        Some(label) => format!("no enclosing loop is labeled `{}`", label.text),
                        None => format!("`{word}` outside a loop"),
                    };
                    self.error(codes::UNRESOLVED_NAME, stmt.span, message);
                    return;
                };
                if self.cx.dos.last().is_some_and(|frame| index < frame.loops) {
                    self.unsupported(
                        stmt.span,
                        &format!("`{word}` out of a block that is used as a value"),
                    );
                    return;
                }
                let target = (index + 1 != self.cx.loops.len()).then(|| self.cx.loops[index].1);
                out.push(if is_break {
                    Stmt::Break(target)
                } else {
                    Stmt::Continue(target)
                });
            }
            ast::StmtKind::Yield { .. } => self.unsupported(stmt.span, "generators"),
            ast::StmtKind::Assert { cond, message } => {
                let text = self.source(cond.span);
                let line = self.line(cond.span);
                let span = self.span(stmt.span);
                let cond = self.expect(cond, &Ty::Bool);
                let message = message.as_ref().map(|m| self.expect(m, &Ty::Str));
                out.push(Stmt::Assert {
                    cond,
                    message,
                    text,
                    line,
                    span,
                });
            }
            ast::StmtKind::Unchecked(block) => out.extend(self.block(block, Tail::Discard).stmts),
            ast::StmtKind::Error => {}
        }
    }

    fn open_loop(&mut self, label: Option<String>) -> LoopId {
        let id = self.next_loop;
        self.next_loop += 1;
        self.cx.loops.push((label, id));
        id
    }

    /// The slot a `for` fills each round. A pattern other than a name is
    /// taken apart at the top of the body.
    pub(super) fn loop_var(&mut self, pat: &ast::Pattern, elem: Ty, body: &mut Vec<Stmt>) -> Slot {
        match &pat.kind {
            ast::PatternKind::Name(name) => {
                let slot = self.new_slot(name, elem);
                self.cx
                    .scopes
                    .last_mut()
                    .expect("a scope")
                    .insert(name.clone(), slot);
                slot
            }
            ast::PatternKind::Wildcard => self.temp(elem),
            _ => {
                let slot = self.temp(elem);
                let value = self.local(slot, pat.span);
                self.bind(pat, value, body);
                slot
            }
        }
    }

    fn domain(&mut self, name: &ast::Name, def: &ast::Expr, out: &mut Vec<Stmt>) {
        if !self.at_top_level() {
            self.unsupported(name.span, "a domain declared inside a fn or block");
            return;
        }
        let Some(Sym::Domain(id)) = self.syms.get(&name.text).copied() else {
            return;
        };
        let ast::ExprKind::Range {
            lo: Some(lo),
            hi: Some(hi),
            inclusive,
            step: None,
        } = &def.kind
        else {
            self.unsupported(def.span, "a domain that is not a range such as `1..=n`");
            return;
        };
        let lo = self.expect(lo, &Ty::Int);
        let hi = self.expect(hi, &Ty::Int);
        // The size is `hi - lo`, plus one when the end is included. `lo`
        // is used twice, so it goes through a temporary.
        let slot = self.temp(Ty::Int);
        out.push(Stmt::Let(slot, Some(lo)));
        let lo = self.local(slot, def.span);
        let len = self.mk(
            ExprKind::Binary(BinOp::Sub, Box::new(hi), Box::new(lo.clone())),
            Ty::Int,
            def.span,
        );
        let len = if *inclusive {
            self.int_add(len, 1)
        } else {
            len
        };
        out.push(Stmt::DefDomain { id, lo, len });
    }

    /// The value a `let` binds: its initializer, read or allocated the way
    /// the declared type says.
    fn let_value(
        &mut self,
        ty: Option<&ast::Type>,
        init: Option<&ast::Expr>,
        span: Span,
    ) -> Option<Expr> {
        let Some(ty) = ty else {
            let Some(init) = init else {
                self.error(
                    codes::UNKNOWN_TYPE,
                    span,
                    "a variable needs a type or a value",
                );
                return None;
            };
            return Some(self.expr(init, None));
        };
        let spec = self.resolve(ty, true);
        let want = spec.ty();
        let Some(init) = init else {
            return Some(self.zero(&spec, span));
        };
        if is_read_call(init) {
            return Some(self.read(spec, init.span));
        }
        if let Spec::Array { dims, elem } = &spec {
            let elem_ty = elem.ty();
            if is_literal(init) {
                let value = self.expect(init, &elem_ty);
                return Some(self.mk(
                    ExprKind::Fill(dims.clone(), Some(Box::new(value))),
                    want,
                    init.span,
                ));
            }
            if let Some(table) = self.comprehension_table(init, dims, &elem_ty) {
                return Some(table);
            }
            let value = self.expr(init, Some(&want));
            if value.ty == want || value.ty == Ty::Error {
                return Some(value);
            }
            return Some(match self.try_coerce(value, &elem_ty) {
                Ok(value) => self.mk(
                    ExprKind::Fill(dims.clone(), Some(Box::new(value))),
                    want,
                    init.span,
                ),
                Err(value) => {
                    self.mismatch(init.span, &want, &value.ty);
                    self.err_expr(init.span)
                }
            });
        }
        Some(self.expect(init, &want))
    }

    /// The value a variable declared without one starts with.
    fn zero(&mut self, spec: &Spec, span: Span) -> Expr {
        let ty = spec.ty();
        let kind = match spec {
            Spec::Array { dims, .. } => ExprKind::Fill(dims.clone(), None),
            _ => match &ty {
                Ty::Int => ExprKind::Int(0),
                Ty::Float => ExprKind::Float(0.0),
                Ty::Bool => ExprKind::Bool(false),
                Ty::Str => ExprKind::Str(Vec::new()),
                Ty::Opt(_) => ExprKind::None,
                Ty::List(_) | Ty::Map(..) | Ty::Set(_) => ExprKind::Tuple(Vec::new()),
                Ty::Error => ExprKind::Error,
                _ => {
                    let message = format!(
                        "a `{}` has no starting value: give this variable one",
                        self.show(&ty)
                    );
                    self.error(codes::UNKNOWN_TYPE, span, message);
                    ExprKind::Error
                }
            },
        };
        self.mk(kind, ty, span)
    }

    /// Binds the names in `pat` to `value`.
    pub(super) fn bind(&mut self, pat: &ast::Pattern, value: Expr, out: &mut Vec<Stmt>) {
        match &pat.kind {
            ast::PatternKind::Name(name) => self.declare(name, value, out),
            ast::PatternKind::Wildcard => out.push(Stmt::Expr(value)),
            ast::PatternKind::Tuple(pats) => {
                let Some(fields) = value.ty.fields() else {
                    if value.ty != Ty::Error {
                        let message = format!(
                            "a `{}` can't be taken apart into {} names",
                            self.show(&value.ty),
                            pats.len()
                        );
                        self.error(codes::TYPE_MISMATCH, pat.span, message);
                    }
                    return;
                };
                if fields.len() != pats.len() {
                    let message = format!(
                        "this pattern has {} names, but the value has {} fields",
                        pats.len(),
                        fields.len()
                    );
                    self.error(codes::TYPE_MISMATCH, pat.span, message);
                    return;
                }
                let base = self.hold(value, out);
                for (i, (p, ty)) in pats.iter().zip(fields).enumerate() {
                    let field = Expr {
                        kind: ExprKind::Field(Box::new(base.clone()), i),
                        ty,
                        span: base.span,
                    };
                    self.bind(p, field, out);
                }
            }
            ast::PatternKind::Error => {}
            _ => self.unsupported(pat.span, "this pattern outside a `match`"),
        }
    }

    /// `value` as something cheap to mention twice: itself if it is a
    /// variable, else a temporary that holds it.
    pub(super) fn hold(&mut self, value: Expr, out: &mut Vec<Stmt>) -> Expr {
        if matches!(value.kind, ExprKind::Local(_) | ExprKind::Global(_)) {
            return value;
        }
        let span = value.span;
        let slot = self.temp(value.ty.clone());
        let ty = value.ty.clone();
        out.push(Stmt::Let(slot, Some(value)));
        Expr {
            kind: ExprKind::Local(slot),
            ty,
            span,
        }
    }

    fn assign(
        &mut self,
        target: &ast::Expr,
        op: ast::AssignOp,
        value: &ast::Expr,
        out: &mut Vec<Stmt>,
    ) {
        if let (ast::ExprKind::Tuple(targets), ast::AssignOp::Set) = (&target.kind, op) {
            // `(a, b) = (b, a)`: the right side is whole before any store.
            let value = self.expr(value, None);
            let Some(fields) = value.ty.fields().filter(|f| f.len() == targets.len()) else {
                if value.ty != Ty::Error {
                    let message = format!(
                        "{} variables can't take a `{}`",
                        targets.len(),
                        self.show(&value.ty)
                    );
                    self.error(codes::TYPE_MISMATCH, target.span, message);
                }
                return;
            };
            let slot = self.temp(value.ty.clone());
            let base = Expr {
                kind: ExprKind::Local(slot),
                ty: value.ty.clone(),
                span: value.span,
            };
            out.push(Stmt::Let(slot, Some(value)));
            for (i, (t, ty)) in targets.iter().zip(fields).enumerate() {
                let place = self.place(t);
                let field = Expr {
                    kind: ExprKind::Field(Box::new(base.clone()), i),
                    ty,
                    span: base.span,
                };
                let field = self.coerce(field, &place.ty, t.span);
                out.push(Stmt::Assign(place, AssignOp::Set, field));
            }
            return;
        }
        let place = self.place(target);
        let (op, value) = match op {
            ast::AssignOp::Set => (AssignOp::Set, self.expect(value, &place.ty)),
            ast::AssignOp::Max | ast::AssignOp::Min => {
                if !matches!(place.ty, Ty::Error) && !self.ordered(&place.ty) {
                    let message = format!(
                        "`max=` and `min=` need values with an order, not `{}`",
                        self.show(&place.ty)
                    );
                    self.error(codes::BAD_OPERANDS, target.span, message);
                }
                let op = if op == ast::AssignOp::Max {
                    AssignOp::Max
                } else {
                    AssignOp::Min
                };
                (op, self.expect(value, &place.ty))
            }
            ast::AssignOp::Binary(bin) => {
                use BinOp::*;
                let value = match (&place.ty, bin) {
                    (
                        Ty::Int,
                        Add | Sub | Mul | Div | Rem | Pow | BitAnd | BitOr | BitXor | Shl | Shr,
                    ) => self.expect(value, &Ty::Int),
                    (Ty::Float, Add | Sub | Mul | Div) => self.expect(value, &Ty::Float),
                    (Ty::Bool, BitAnd | BitOr | BitXor) => self.expect(value, &Ty::Bool),
                    (Ty::Str, Add) => {
                        let value = self.expr(value, None);
                        self.as_str(value, target.span)
                    }
                    (Ty::Error, _) => self.expr(value, None),
                    _ => {
                        let message = format!(
                            "`{}=` can't be applied to a `{}`",
                            bin.text(),
                            self.show(&place.ty)
                        );
                        self.error(codes::BAD_OPERANDS, target.span, message);
                        self.expr(value, None)
                    }
                };
                (AssignOp::Binary(bin), value)
            }
        };
        out.push(Stmt::Assign(place, op, value));
    }

    /// `e` as something to store into.
    pub(super) fn place(&mut self, e: &ast::Expr) -> Place {
        let span = self.span(e.span);
        let error = Place {
            kind: PlaceKind::Local(0),
            ty: Ty::Error,
            span,
        };
        match &e.kind {
            ast::ExprKind::Paren(inner) => self.place(inner),
            ast::ExprKind::Name(_) | ast::ExprKind::SelfValue => {
                let value = self.expr(e, None);
                match value.kind {
                    ExprKind::Local(slot) => Place {
                        kind: PlaceKind::Local(slot),
                        ty: value.ty,
                        span,
                    },
                    ExprKind::Global(id) => Place {
                        kind: PlaceKind::Global(id),
                        ty: value.ty,
                        span,
                    },
                    // A bare field name inside a method.
                    ExprKind::Field(base, index) => match base.kind {
                        ExprKind::Local(slot) => Place {
                            kind: PlaceKind::Field(
                                Box::new(Place {
                                    kind: PlaceKind::Local(slot),
                                    ty: base.ty,
                                    span,
                                }),
                                index,
                            ),
                            ty: value.ty,
                            span,
                        },
                        _ => error,
                    },
                    ExprKind::Error => error,
                    _ => {
                        self.error(
                            codes::ASSIGN_TO_LET,
                            e.span,
                            "this name can't be assigned to",
                        );
                        error
                    }
                }
            }
            ast::ExprKind::Index { base, indices } => {
                let base = self.place(base);
                let (indices, ty) = self.index_args(&base.ty, indices, e.span);
                Place {
                    kind: PlaceKind::Index(Box::new(base), indices),
                    ty,
                    span,
                }
            }
            ast::ExprKind::Field { base, name } => {
                let base = self.place(base);
                match self.field_of(&base.ty, &name.text) {
                    Some((index, ty)) => Place {
                        kind: PlaceKind::Field(Box::new(base), index),
                        ty,
                        span,
                    },
                    None => {
                        if base.ty != Ty::Error {
                            let message = format!(
                                "`{}` has no field `{}` to assign to",
                                self.show(&base.ty),
                                name.text
                            );
                            self.error(codes::UNKNOWN_FIELD, name.span, message);
                        }
                        error
                    }
                }
            }
            _ => {
                self.error(
                    codes::ASSIGN_TO_LET,
                    e.span,
                    "only a variable, a field, or an element can be assigned to",
                );
                error
            }
        }
    }

    /// A stored field of a struct, record or tuple (`t.0`).
    pub(super) fn field_of(&self, ty: &Ty, name: &str) -> Option<(usize, Ty)> {
        match ty {
            Ty::Struct(id) => {
                let fields = &self.structs[*id].def.fields;
                let index = fields.iter().position(|(f, _)| f == name)?;
                Some((index, fields[index].1.clone()))
            }
            Ty::Record(fields) => {
                let index = fields.iter().position(|(f, _)| f == name)?;
                Some((index, fields[index].1.clone()))
            }
            Ty::Tuple(items) => {
                let index: usize = name.parse().ok()?;
                Some((index, items.get(index)?.clone()))
            }
            _ => None,
        }
    }

    fn if_chain(
        &mut self,
        branches: &[(ast::Cond, ast::Block)],
        otherwise: Option<&ast::Block>,
        tail: Tail,
        out: &mut Vec<Stmt>,
    ) {
        let Some(((cond, body), rest)) = branches.split_first() else {
            if let Some(block) = otherwise {
                out.extend(self.block(block, tail).stmts);
            }
            return;
        };
        match cond {
            ast::Cond::Expr(cond) => {
                let cond = self.expect(cond, &Ty::Bool);
                let then = self.block(body, tail);
                let mut els = Vec::new();
                self.if_chain(rest, otherwise, tail, &mut els);
                out.push(Stmt::If(cond, then, Block { stmts: els }));
            }
            ast::Cond::Let { pat, value } => {
                let Some((present, inner)) = self.optional(value, "if let", out) else {
                    return;
                };
                self.cx.scopes.push(HashMap::new());
                let mut then = Vec::new();
                self.bind(pat, inner, &mut then);
                then.extend(self.block(body, tail).stmts);
                self.cx.scopes.pop();
                let mut els = Vec::new();
                self.if_chain(rest, otherwise, tail, &mut els);
                out.push(Stmt::If(
                    present,
                    Block { stmts: then },
                    Block { stmts: els },
                ));
            }
            ast::Cond::Const(cond) => self.unsupported(cond.span, "`if const`"),
        }
    }

    /// Evaluates the optional `value` into a temporary; gives the test
    /// that it is present and the value inside.
    fn optional(
        &mut self,
        value: &ast::Expr,
        what: &str,
        out: &mut Vec<Stmt>,
    ) -> Option<(Expr, Expr)> {
        let value_expr = self.expr(value, None);
        let Ty::Opt(inner) = value_expr.ty.clone() else {
            if value_expr.ty != Ty::Error {
                let message = format!(
                    "`{what}` takes the value out of an optional, but this is a `{}`",
                    self.show(&value_expr.ty)
                );
                self.error(codes::TYPE_MISMATCH, value.span, message);
            }
            return None;
        };
        let span = value_expr.span;
        let slot = self.temp(value_expr.ty.clone());
        let held = Expr {
            kind: ExprKind::Local(slot),
            ty: value_expr.ty.clone(),
            span,
        };
        out.push(Stmt::Let(slot, Some(value_expr)));
        let present = Expr {
            kind: ExprKind::IsSome(Box::new(held.clone())),
            ty: Ty::Bool,
            span,
        };
        let inside = Expr {
            kind: ExprKind::Unwrap(Box::new(held), false),
            ty: *inner,
            span,
        };
        Some((present, inside))
    }

    fn while_stmt(&mut self, cond: &ast::Cond, body: &ast::Block, out: &mut Vec<Stmt>) {
        match cond {
            ast::Cond::Expr(cond) => {
                let cond = self.expect(cond, &Ty::Bool);
                let id = self.open_loop(None);
                let body = self.block(body, Tail::Discard);
                self.cx.loops.pop();
                out.push(Stmt::While(id, cond, body));
            }
            ast::Cond::Let { pat, value } => {
                // `while let x = next():` is a loop that leaves when the
                // optional is empty.
                let id = self.open_loop(None);
                self.cx.scopes.push(HashMap::new());
                let mut stmts = Vec::new();
                if let Some((present, inner)) = self.optional(value, "while let", &mut stmts) {
                    let span = present.span;
                    let absent = Expr {
                        kind: ExprKind::Unary(UnOp::Not, Box::new(present)),
                        ty: Ty::Bool,
                        span,
                    };
                    stmts.push(Stmt::If(
                        absent,
                        Block {
                            stmts: vec![Stmt::Break(None)],
                        },
                        Block::default(),
                    ));
                    self.bind(pat, inner, &mut stmts);
                    stmts.extend(self.block(body, Tail::Discard).stmts);
                }
                self.cx.scopes.pop();
                self.cx.loops.pop();
                let always = self.mk(ExprKind::Bool(true), Ty::Bool, value.span);
                out.push(Stmt::While(id, always, Block { stmts }));
            }
            ast::Cond::Const(cond) => self.unsupported(cond.span, "`while const`"),
        }
    }

    pub(super) fn match_stmt(
        &mut self,
        whole: &ast::Expr,
        scrutinee: &ast::Expr,
        arms: &[ast::Arm],
        tail: Tail,
        out: &mut Vec<Stmt>,
    ) {
        let value = self.expr(scrutinee, None);
        let value = self.hold(value, out);
        let span = self.span(whole.span);
        let trap = Stmt::Trap("no arm of this `match` fits the value".into(), span);
        if arms.iter().all(|arm| arm.guard.is_none()) {
            let mut lowered = Vec::new();
            for arm in arms {
                self.cx.scopes.push(HashMap::new());
                let mut body = Vec::new();
                let test = self.pattern(&arm.pat, &value, &mut body);
                body.extend(self.block(&arm.body, tail).stmts);
                self.cx.scopes.pop();
                lowered.push((test, body));
            }
            let mut chain = vec![trap];
            for (test, body) in lowered.into_iter().rev() {
                chain = match test {
                    None => body,
                    Some(test) => vec![Stmt::If(
                        test,
                        Block { stmts: body },
                        Block { stmts: chain },
                    )],
                };
            }
            out.extend(chain);
            return;
        }
        // With guards an arm can fit the pattern and still decline, so the
        // arms are tried in turn until one has run.
        let done = self.temp(Ty::Bool);
        let done_expr = Expr {
            kind: ExprKind::Local(done),
            ty: Ty::Bool,
            span,
        };
        let not_done = Expr {
            kind: ExprKind::Unary(UnOp::Not, Box::new(done_expr.clone())),
            ty: Ty::Bool,
            span,
        };
        let flag = |value: bool| Expr {
            kind: ExprKind::Bool(value),
            ty: Ty::Bool,
            span,
        };
        out.push(Stmt::Let(done, Some(flag(false))));
        for arm in arms {
            self.cx.scopes.push(HashMap::new());
            let mut body = Vec::new();
            let test = self.pattern(&arm.pat, &value, &mut body);
            let guard = arm.guard.as_ref().map(|g| self.expect(g, &Ty::Bool));
            let mut run = vec![Stmt::Assign(
                Place {
                    kind: PlaceKind::Local(done),
                    ty: Ty::Bool,
                    span,
                },
                AssignOp::Set,
                flag(true),
            )];
            run.extend(self.block(&arm.body, tail).stmts);
            self.cx.scopes.pop();
            match guard {
                Some(guard) => body.push(Stmt::If(guard, Block { stmts: run }, Block::default())),
                None => body.extend(run),
            }
            let test = match test {
                None => not_done.clone(),
                Some(test) => Expr {
                    kind: ExprKind::Binary(BinOp::And, Box::new(not_done.clone()), Box::new(test)),
                    ty: Ty::Bool,
                    span,
                },
            };
            out.push(Stmt::If(test, Block { stmts: body }, Block::default()));
        }
        if tail == Tail::Discard {
            out.push(Stmt::If(
                not_done,
                Block { stmts: vec![trap] },
                Block::default(),
            ));
        } else {
            out.push(trap);
        }
    }

    /// The test that `value` fits `pat` (`None` when it always does), with
    /// the statements that bind the pattern's names once it has.
    fn pattern(&mut self, pat: &ast::Pattern, value: &Expr, binds: &mut Vec<Stmt>) -> Option<Expr> {
        let span = self.span(pat.span);
        let and = |a: Option<Expr>, b: Option<Expr>| match (a, b) {
            (Some(a), Some(b)) => Some(Expr {
                kind: ExprKind::Binary(BinOp::And, Box::new(a), Box::new(b)),
                ty: Ty::Bool,
                span,
            }),
            (a, b) => a.or(b),
        };
        match &pat.kind {
            ast::PatternKind::Wildcard | ast::PatternKind::Error => None,
            ast::PatternKind::Name(name) => {
                self.declare(name, value.clone(), binds);
                None
            }
            ast::PatternKind::Literal(lit) => {
                if let (ast::ExprKind::None, Ty::Opt(_)) = (&lit.kind, &value.ty) {
                    let present = Expr {
                        kind: ExprKind::IsSome(Box::new(value.clone())),
                        ty: Ty::Bool,
                        span,
                    };
                    return Some(Expr {
                        kind: ExprKind::Unary(UnOp::Not, Box::new(present)),
                        ty: Ty::Bool,
                        span,
                    });
                }
                let lit = self.expect(lit, &value.ty);
                Some(Expr {
                    kind: ExprKind::Cmp(CmpOp::Eq, Box::new(value.clone()), Box::new(lit)),
                    ty: Ty::Bool,
                    span,
                })
            }
            ast::PatternKind::Variant { path, args } => {
                let name = path.last().expect("a variant pattern has a name");
                let Ty::Enum(id) = value.ty else {
                    if value.ty != Ty::Error {
                        let message = format!(
                            "`{}` is a variant, but the value matched is a `{}`",
                            name.text,
                            self.show(&value.ty)
                        );
                        self.error(codes::TYPE_MISMATCH, pat.span, message);
                    }
                    return None;
                };
                let variants = &self.enums[id].variants;
                let Some(tag) = variants.iter().position(|(v, _)| *v == name.text) else {
                    let message =
                        format!("`{}` has no variant `{}`", self.enums[id].name, name.text);
                    self.error(codes::UNKNOWN_FIELD, name.span, message);
                    return None;
                };
                let fields = variants[tag].1.clone();
                if fields.len() != args.len() {
                    let message = format!(
                        "`{}` holds {} values, but this pattern names {}",
                        name.text,
                        fields.len(),
                        args.len()
                    );
                    self.error(codes::WRONG_ARG_COUNT, pat.span, message);
                    return None;
                }
                let mut test = Some(Expr {
                    kind: ExprKind::IsVariant(Box::new(value.clone()), tag),
                    ty: Ty::Bool,
                    span,
                });
                for (i, (arg, ty)) in args.iter().zip(fields).enumerate() {
                    let payload = Expr {
                        kind: ExprKind::Payload(Box::new(value.clone()), tag, i),
                        ty,
                        span,
                    };
                    test = and(test, self.pattern(arg, &payload, binds));
                }
                test
            }
            ast::PatternKind::Tuple(pats) => {
                let Some(fields) = value.ty.fields().filter(|f| f.len() == pats.len()) else {
                    if value.ty != Ty::Error {
                        let message = format!(
                            "a pattern of {} parts can't match a `{}`",
                            pats.len(),
                            self.show(&value.ty)
                        );
                        self.error(codes::TYPE_MISMATCH, pat.span, message);
                    }
                    return None;
                };
                let mut test = None;
                for (i, (p, ty)) in pats.iter().zip(fields).enumerate() {
                    let field = Expr {
                        kind: ExprKind::Field(Box::new(value.clone()), i),
                        ty,
                        span,
                    };
                    test = and(test, self.pattern(p, &field, binds));
                }
                test
            }
            ast::PatternKind::Or(alts) => {
                let mut test: Option<Expr> = None;
                let mut always = false;
                for alt in alts {
                    let mut names = Vec::new();
                    let one = self.pattern(alt, value, &mut names);
                    if !names.is_empty() {
                        self.unsupported(alt.span, "names bound inside a `|` pattern");
                    }
                    match (one, test.take()) {
                        (None, _) => always = true,
                        (Some(one), None) => test = Some(one),
                        (Some(one), Some(prev)) => {
                            test = Some(Expr {
                                kind: ExprKind::Binary(BinOp::Or, Box::new(prev), Box::new(one)),
                                ty: Ty::Bool,
                                span,
                            })
                        }
                    }
                }
                if always {
                    None
                } else {
                    test
                }
            }
        }
    }
}

/// `read()` with nothing in it.
pub(super) fn is_read_call(e: &ast::Expr) -> bool {
    match &e.kind {
        ast::ExprKind::Call { callee, args } => {
            args.is_empty() && matches!(&callee.kind, ast::ExprKind::Name(n) if n == "read")
        }
        _ => false,
    }
}

/// A literal: in `var a: [V]Int = 0` it is the value of every cell.
fn is_literal(e: &ast::Expr) -> bool {
    match &e.kind {
        ast::ExprKind::Int(_)
        | ast::ExprKind::Float(_)
        | ast::ExprKind::Char(_)
        | ast::ExprKind::Bool(_)
        | ast::ExprKind::None => true,
        ast::ExprKind::Str(pieces) => pieces.iter().all(|p| matches!(p, ast::StrPiece::Text(_))),
        ast::ExprKind::Unary { expr, .. } | ast::ExprKind::Paren(expr) => is_literal(expr),
        ast::ExprKind::Tuple(items) => items.iter().all(is_literal),
        ast::ExprKind::List(items) => items.is_empty(),
        _ => false,
    }
}
