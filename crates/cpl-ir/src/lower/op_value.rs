//! A bare operator inside a tuple or record (`each(+, max)`, `(n: +, hi: max)`)
//! is a value of its own empty type, one per operator, so the operator stays
//! known through generics and `fields(..)`: `same(m, +)` reads it off the type,
//! as it does for a marker struct (R06.1.5).

use super::mono::BareOp;
use super::*;

impl<'a> Lowerer<'a> {
    /// Records `impl Add<R>` (and the other operators) and `impl Into<T>`
    /// by the type they take or give, so several can coexist on one struct:
    /// `a + b` picks by `b`'s type, `Int(x)`/`F64(x)` by `T` (decision 390).
    pub(super) fn register_op_impl(
        &mut self,
        sid: StructId,
        fn_id: FnId,
        trait_name: Option<&str>,
        method: &str,
        rhs: Option<Ty>,
    ) -> bool {
        let binop = matches!(
            (trait_name, method),
            (Some("Add"), "add")
                | (Some("Sub"), "sub")
                | (Some("Mul"), "mul")
                | (Some("Div"), "div")
                | (Some("Rem"), "rem")
        );
        let into = matches!((trait_name, method), (Some("Into"), "into")) && rhs.is_some();
        if !binop && !into {
            return false;
        }
        let rhs_resolved = rhs.clone().unwrap_or(Ty::Struct(sid));
        let idx_n = self.structs[sid].op_methods.len();
        if into {
            self.sigs[fn_id].name = format!("{}_into{idx_n}", self.sigs[fn_id].name);
        } else if rhs.is_some() && rhs_resolved != Ty::Struct(sid) {
            self.sigs[fn_id].name = format!("{}_op{idx_n}", self.sigs[fn_id].name);
        } else {
            self.structs[sid].methods.insert(method.to_string(), fn_id);
        }
        let entry = (method.to_string(), rhs_resolved, fn_id);
        self.structs[sid].op_methods.push(entry);
        true
    }

    /// `Int(x)` or `F64(x)` on a struct: its `impl Into<Int>` or
    /// `impl Into<F64>`, else E0302 naming the impl to write.
    pub(super) fn convert_by_impl(&mut self, to: &str, value: Expr, span: Span) -> Expr {
        let want = match to {
            "Int" => Some(Ty::Int),
            "F64" => Some(Ty::Float),
            _ => None,
        };
        let into = match (&value.ty, want) {
            (Ty::Struct(sid), Some(want)) => self.structs[*sid]
                .op_methods
                .iter()
                .find(|(m, t, _)| m == "into" && *t == want)
                .map(|(_, t, id)| (*id, t.clone())),
            _ => None,
        };
        let Some((id, ty)) = into else {
            self.bad_conversion(to, &value.ty, span);
            return self.err_expr(span);
        };
        self.queue_fn(id);
        self.mk(ExprKind::Call(id, vec![Arg::Value(value)]), ty, span)
    }

    /// The empty struct that stands for `op` as a value.
    fn op_type(&mut self, op: BareOp) -> StructId {
        if let Some(&(_, id)) = self.op_types.iter().find(|(o, _)| *o == op) {
            return id;
        }
        let id = self.push_synth_struct(format!("Op_{op:?}"), Vec::new());
        self.op_types.push((op, id));
        id
    }

    /// A call of a field that holds a function (`m.op(a, b)`, `self.lt(a, b)`)
    /// or a bare operator (`(count: +).count(a, b)`); `Err` gives `value` back.
    pub(super) fn call_field(
        &mut self,
        value: Expr,
        name: &ast::Name,
        args: &[ast::Arg],
        span: Span,
        hint: Option<&Ty>,
    ) -> Result<Expr, Expr> {
        let Some((index, fty)) = self.field_of(&value.ty, &name.text) else {
            return Err(value);
        };
        if matches!(fty, Ty::Fn(..)) {
            let fv = self.mk(ExprKind::Field(Box::new(value), index), fty, span);
            return Ok(self.call_fn_val(fv, args, span));
        }
        match self.op_of_type(&fty) {
            Some(op) => self
                .bare_op_call(op, name.span, args, span, hint)
                .ok_or(value),
            None => Err(value),
        }
    }

    /// The operator a value of type `ty` stands for.
    pub(super) fn op_of_type(&self, ty: &Ty) -> Option<BareOp> {
        let Ty::Struct(id) = ty else { return None };
        self.op_types
            .iter()
            .find(|(_, s)| s == id)
            .map(|(op, _)| *op)
    }

    /// A component of a tuple or record literal: a bare operator with no fn
    /// type asked for is its operator value; anything else lowers as usual.
    pub(super) fn component(&mut self, item: &ast::Expr, hint: Option<&Ty>) -> Expr {
        let bare = matches!(
            &item.kind,
            ast::ExprKind::Lambda { .. } | ast::ExprKind::Name(_)
        );
        let wants_op = match hint {
            None | Some(Ty::Unit) => true,
            Some(h) => self.op_of_type(h).is_some(),
        };
        if bare && wants_op {
            if let Some(op) = self.detect_bare_op(item) {
                let id = self.op_type(op);
                return self.mk(ExprKind::Tuple(Vec::new()), Ty::Struct(id), item.span);
            }
        }
        match hint {
            Some(h) => self.expect(item, h),
            None => self.expr(item, None),
        }
    }

    /// The operator `e` names: a bare operator, or a value of an operator type.
    pub(super) fn op_identity(&mut self, e: &ast::Expr) -> Option<BareOp> {
        if let Some(op) = self.detect_bare_op(e) {
            return Some(op);
        }
        let ty = self.probe_ty_silent(e)?;
        self.op_of_type(&ty)
    }

    /// The operator a record field holds (`ops.count` of `(count: +, n: 2)`),
    /// so `ops.count(a, b)` calls it. Only probed once an operator value exists.
    pub(super) fn field_op(&mut self, base: &ast::Expr, field: &str) -> Option<BareOp> {
        if self.op_types.is_empty() {
            return None;
        }
        let Some(Ty::Record(fields)) = self.probe_ty_silent(base) else {
            return None;
        };
        let (_, ty) = fields.iter().find(|(name, _)| name == field)?;
        self.op_of_type(ty)
    }
}

impl Lowerer<'_> {
    /// The builtin for an `Int` `+`, `-` or `*` written in an `unchecked:`
    /// or `with checks(overflow: off)` block, where it wraps in every build
    /// (R09.4.2, R09.4.3).
    pub(super) fn wrapping(&self, op: BinOp, ty: &Ty) -> Option<Builtin> {
        if (self.unchecked == 0 && self.overflow_checks_off == 0) || *ty != Ty::Int {
            return None;
        }
        match op {
            BinOp::Add => Some(Builtin::WrapAdd),
            BinOp::Sub => Some(Builtin::WrapSub),
            BinOp::Mul => Some(Builtin::WrapMul),
            _ => None,
        }
    }

    /// `place op= value` once `value` is typed: where `op` wraps, it is
    /// `place = place op value` with the wrapping builtin, so `x += 1` and
    /// `x = x + 1` wrap alike.
    pub(super) fn compound_step(
        &mut self,
        place: &Place,
        op: BinOp,
        value: Expr,
        span: Span,
    ) -> (AssignOp, Expr) {
        match self.wrapping(op, &place.ty) {
            Some(wrap) if value.ty == place.ty => {
                let read = super::stmt::place_to_expr(place);
                let step = ExprKind::Builtin(wrap, vec![read, value]);
                (AssignOp::Set, self.mk(step, place.ty.clone(), span))
            }
            _ => (AssignOp::Binary(op), value),
        }
    }
}

impl Lowerer<'_> {
    /// Recognizes a bare operator expression (`+`, `*`, `^`, `|`, `&`, `max`, `min`, `gcd`, `lcm`)
    /// or a local variable bound to a `BareOp` generic parameter.
    pub(super) fn detect_bare_op(&self, e: &ast::Expr) -> Option<BareOp> {
        match &e.kind {
            ast::ExprKind::Paren(inner) => self.detect_bare_op(inner),
            ast::ExprKind::Unwrap(inner) => match &inner.kind {
                ast::ExprKind::Name(n) => self.cx.op_locals.get(n).copied(),
                _ => None,
            },
            ast::ExprKind::Name(n) => {
                if let Some(&op) = self.cx.op_locals.get(n) {
                    return Some(op);
                }
                if self.lookup(n).is_some() {
                    return None;
                }
                match n.as_str() {
                    "max" => Some(BareOp::Max),
                    "min" => Some(BareOp::Min),
                    "gcd" => Some(BareOp::Gcd),
                    "lcm" => Some(BareOp::Lcm),
                    _ => None,
                }
            }
            ast::ExprKind::Lambda { params, body, .. } if params.len() == 2 => {
                let ast::PatternKind::Name(p0) = &params[0].kind else {
                    return None;
                };
                let ast::PatternKind::Name(p1) = &params[1].kind else {
                    return None;
                };
                if let ast::ExprKind::Binary { op, lhs, rhs, .. } = &body.kind {
                    let is_direct = matches!(&lhs.kind, ast::ExprKind::Name(a) if a == p0)
                        && matches!(&rhs.kind, ast::ExprKind::Name(b) if b == p1);
                    if is_direct {
                        return match op {
                            ast::BinOp::Add => Some(BareOp::Add),
                            ast::BinOp::Mul => Some(BareOp::Mul),
                            ast::BinOp::BitXor => Some(BareOp::BitXor),
                            ast::BinOp::BitOr => Some(BareOp::BitOr),
                            ast::BinOp::BitAnd => Some(BareOp::BitAnd),
                            ast::BinOp::And => Some(BareOp::And),
                            ast::BinOp::Or => Some(BareOp::Or),
                            _ => None,
                        };
                    }
                }
                None
            }
            ast::ExprKind::Field { base, name: field } => {
                let ast::ExprKind::Name(bname) = &base.kind else {
                    return None;
                };
                let bty = if let Some(slot) = self.lookup(bname) {
                    self.cx.locals[slot].ty.clone()
                } else if let Some(Sym::Global(gid)) = self.get_sym(bname) {
                    self.globals[gid].ty.clone()
                } else {
                    return None;
                };
                let Ty::Struct(sid) = bty else {
                    return None;
                };
                let (gid, gargs) = self.structs[sid].origin_gen.as_ref()?;
                let decl = self.gen_structs[*gid].decl;
                let fdecl = decl.fields.iter().find(|f| f.name.text == field.text)?;
                let ast::TypeKind::Named { name, args } = &fdecl.ty.kind else {
                    return None;
                };
                if !args.is_empty() {
                    return None;
                }
                let pos = decl.generics.iter().position(|g| g.text == name.text)?;
                match gargs.get(pos) {
                    Some(GenArg::Op(op)) => Some(*op),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}
