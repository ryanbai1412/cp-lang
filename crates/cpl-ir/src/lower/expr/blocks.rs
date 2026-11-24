//! Blocks with a value: `if` and `match` as values, conditional expressions,
//! and lifting a block that leaves the fn out of the middle of an expression.

use super::*;

impl<'a> Lowerer<'a> {
    /// A block with a value. The first value given decides the type when
    /// the context doesn't.
    /// `best min= d` as a `Bool`: the update of the statement, then whether
    /// the place holds something else than before. It runs where it is
    /// written, so not at all on the unused side of an `and`.
    pub(in crate::lower) fn keep(
        &mut self,
        target: &ast::Expr,
        max: bool,
        value: &ast::Expr,
        span: Span,
    ) -> Expr {
        // `none` on the right is skipped: nothing changes.
        if matches!(value.kind, ast::ExprKind::None) {
            return self.mk(ExprKind::Bool(false), Ty::Bool, span);
        }
        let op = if max {
            ast::AssignOp::Max
        } else {
            ast::AssignOp::Min
        };
        if self.grid_row(target).is_some() {
            return self.do_expr(Some(&Ty::Bool), span, |lw, out| {
                let changed = match lw.grid_keep(target, Some(max), value, true, out) {
                    Some(changed) => lw.local(changed, span),
                    None => lw.err_expr(span),
                };
                out.push(Stmt::Give(changed));
            });
        }
        self.do_expr(Some(&Ty::Bool), span, |lw, out| {
            // What the place needs first (the check of `o! min= d`) runs
            // here, and not ahead of the whole statement.
            let outer = std::mem::take(&mut lw.cx.pending);
            let place = lw.place(target);
            out.extend(std::mem::replace(&mut lw.cx.pending, outer));
            let place = lw.settle_indexes(place, out);
            let rhs = lw.expr(value, Some(&place.ty));
            if place.ty == Ty::Error {
                out.push(Stmt::Give(lw.err_expr(span)));
                return;
            }
            if lw.is_row(&place.ty) {
                let changed = lw.row_keep(place, max, rhs, true, target.span, out);
                let changed = match changed {
                    Some(changed) => lw.local(changed, span),
                    None => lw.err_expr(span),
                };
                out.push(Stmt::Give(changed));
                return;
            }
            let now = place_to_expr(&place);
            let old = lw.temp(place.ty.clone());
            // A key the map does not have yet takes the value.
            let had = match &place.kind {
                PlaceKind::Index(base, keys)
                    if matches!(base.ty, Ty::Map(..) | Ty::HashMap(..)) =>
                {
                    let has = lw.mk_builtin(
                        Builtin::Contains,
                        [place_to_expr(base), keys[0].clone()],
                        Ty::Bool,
                        span,
                    );
                    let slot = lw.temp(Ty::Bool);
                    out.push(Stmt::Let(slot, Some(has)));
                    Some(lw.local(slot, span))
                }
                _ => None,
            };
            match &had {
                Some(had) => {
                    let held = Place {
                        kind: PlaceKind::Local(old),
                        ty: place.ty.clone(),
                        span,
                    };
                    out.push(Stmt::Let(old, None));
                    out.push(Stmt::If(
                        had.clone(),
                        Block {
                            stmts: vec![Stmt::Assign(held, AssignOp::Set, now.clone())],
                        },
                        Block::default(),
                    ));
                }
                None => out.push(Stmt::Let(old, Some(now.clone()))),
            }
            lw.emit_assign_place(place, op, rhs, target.span, out);
            let before = lw.local(old, span);
            let mut changed = lw.mk_cmp(CmpOp::Ne, now, before, Ty::Bool, span);
            if let Some(had) = had {
                let fresh = lw.mk_unary(UnOp::Not, had, Ty::Bool, span);
                changed = lw.mk_binary(BinOp::Or, fresh, changed, Ty::Bool, span);
            }
            out.push(Stmt::Give(changed));
        })
    }

    /// A `match` or `if` that is the whole value of a statement and has
    /// an arm that leaves (`Skip => continue`): its arms run as statements
    /// ahead of the statement, each storing its value, so the arm that
    /// leaves is an ordinary `continue`.
    pub(in crate::lower) fn leaving_value(
        &mut self,
        e: &ast::Expr,
        hint: Option<&Ty>,
        span: Span,
    ) -> Expr {
        let loops = self.cx.dos.last().map_or(0, |frame| frame.loops);
        self.cx.dos.push(DoFrame {
            ty: hint.cloned(),
            loops,
            open: true,
        });
        let mut stmts = Vec::new();
        self.tail(e, Tail::Give, &mut stmts);
        let frame = self.cx.dos.pop().expect("pushed above");
        let ty = frame.ty.unwrap_or(Ty::Error);
        let kept = self.temp(ty.clone());
        let place = Place {
            kind: PlaceKind::Local(kept),
            ty,
            span: self.span(span),
        };
        store_gives(&mut stmts, &place);
        self.cx.pending.push(Stmt::Let(kept, None));
        self.cx.pending.extend(stmts);
        self.local(kept, span)
    }

    /// An `if` whose branches are single expressions is a conditional
    /// expression: `if a < b: b - a else: a - b`.
    pub(in crate::lower) fn ternary(
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
        let else_block = otherwise?;
        let otherwise = single(else_block)?;
        let mut arms = Vec::new();
        for (cond, block) in branches {
            let ast::Cond::Expr(cond) = cond else {
                return None;
            };
            arms.push((cond, single(block)?));
        }
        let mut lowered = Vec::new();
        let mut ty = hint.cloned();
        let first_needs_hint = ty.is_none()
            && arms.first().is_some_and(|(_, v)| {
                matches!(v.kind, ast::ExprKind::None)
                    || matches!(&v.kind, ast::ExprKind::List(items) if items.is_empty())
            });
        let mut pre_otherwise = None;
        let mark = self.cx.present.len();
        if first_needs_hint {
            for (at, (cond, _)) in branches.iter().enumerate() {
                let mutators = self.mutators.clone();
                let past =
                    narrow::past(cond, &branches[at + 1..], Some(else_block), &mut |f, x| {
                        mutators.writes(f, x)
                    });
                self.cx.present.extend(past.into_iter().map(String::from));
            }
            let other_val = self.expr(otherwise, None);
            self.cx.present.truncate(mark);
            if other_val.ty != Ty::Error {
                if matches!(arms[0].1.kind, ast::ExprKind::None) {
                    ty = Some(match &other_val.ty {
                        Ty::Opt(_) => other_val.ty.clone(),
                        t => Ty::opt(t.clone()),
                    });
                } else {
                    ty = Some(other_val.ty.clone());
                }
            }
            pre_otherwise = Some(other_val);
        }
        // In a branch its test holds; past it, the test has failed.
        for (at, (cond, value)) in arms.into_iter().enumerate() {
            let cond = self.expect(cond, &Ty::Bool);
            let (test, body) = &branches[at];
            let past = self.cx.present.len();
            let mutators = self.mutators.clone();
            let held = narrow::in_branch(test, body, &mut |f, x| mutators.writes(f, x));
            self.cx.present.extend(held.into_iter().map(String::from));
            let value = self.expr(value, ty.as_ref());
            self.cx.present.truncate(past);
            let mutators = self.mutators.clone();
            let failed = narrow::past(test, &branches[at + 1..], Some(else_block), &mut |f, x| {
                mutators.writes(f, x)
            });
            self.cx.present.extend(failed.into_iter().map(String::from));
            if ty.is_none() {
                ty = Some(value.ty.clone());
            }
            let at = value.span;
            lowered.push((cond, value, at));
        }
        let mut result = match pre_otherwise {
            Some(v) => v,
            None => self.expr(otherwise, ty.as_ref()),
        };
        self.cx.present.truncate(mark);
        // The branches agree on one type: the hint, or the widest of them.
        let mut ty = ty.unwrap_or(Ty::Error);
        for candidate in lowered.iter().map(|(_, v, _)| &v.ty).chain([&result.ty]) {
            if ty == Ty::Int && self.int_joins(candidate) {
                ty = candidate.clone();
            }
            if let (Ty::Range(c_inner), Ty::Range(t_inner)) = (candidate, &ty) {
                if matches!(
                    (c_inner.as_ref(), t_inner.as_ref()),
                    (Ty::Pos(d1), Ty::Bound(d2)) if d1 == d2
                ) {
                    ty = candidate.clone();
                }
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
                    let (then, other) = (self.show(&ty), self.show(&value.ty));
                    self.diags.push(
                        cpl_diag::msgs::check::IF_BRANCHES_DIFFER.error(span, &[&then, &other]),
                    );
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
}

impl Lowerer<'_> {
    /// `a + x?` cannot leave the fn from the middle of an expression in
    /// the C++, so the statements after `at` that hold such a value get
    /// it worked out ahead of them, operand by operand in the order
    /// written, and then use it.
    pub(in crate::lower) fn lift_leaving(&mut self, out: &mut Vec<Stmt>, at: usize) {
        let mut done = Vec::with_capacity(out.len() - at);
        for mut stmt in out.drain(at..) {
            match &mut stmt {
                // The cell stored to is found first, then the value.
                Stmt::Assign(place, _, e) => {
                    let mut at = &mut *place;
                    let mut indexes = Vec::new();
                    loop {
                        at = match &mut at.kind {
                            PlaceKind::Index(base, index) => {
                                indexes.push(index);
                                base
                            }
                            PlaceKind::Field(base, _) | PlaceKind::Some(base) => base,
                            PlaceKind::Local(_) | PlaceKind::Global(_) => break,
                        };
                    }
                    for index in indexes.into_iter().rev().flatten() {
                        self.lift_value(index, &mut done);
                    }
                    self.lift_value(e, &mut done);
                }
                // A local that is such a block is what the block stores to.
                Stmt::Let(slot, Some(e)) => {
                    let unit = Expr {
                        kind: ExprKind::Unit,
                        ty: Ty::Unit,
                        span: e.span,
                    };
                    let lifted = self.lifted(std::mem::replace(e, unit));
                    match lifted {
                        Expr {
                            kind: ExprKind::Do(mut block),
                            ty,
                            span,
                        } if block_leaves(&block) => {
                            let place = Place {
                                kind: PlaceKind::Local(*slot),
                                ty,
                                span,
                            };
                            store_gives(&mut block.stmts, &place);
                            done.push(Stmt::Let(*slot, None));
                            done.extend(block.stmts);
                            continue;
                        }
                        lifted => *e = lifted,
                    }
                }
                Stmt::Return(Some(e)) | Stmt::Give(e) => self.lift_value(e, &mut done),
                // A block run for what it does is its statements.
                Stmt::Expr(e) => {
                    self.lift_value(e, &mut done);
                    if matches!(e.kind, ExprKind::Unit) {
                        continue;
                    }
                }
                // A test and the ends of a range are read once, before
                // the statement.
                Stmt::If(cond, _, otherwise) => {
                    self.lift_value(cond, &mut done);
                    // An `elif` is an `if` alone in the `else`.
                    if matches!(otherwise.stmts.as_slice(), [Stmt::If(..)]) {
                        self.lift_leaving(&mut otherwise.stmts, 0);
                    }
                }
                Stmt::Assert { cond, .. } => self.lift_value(cond, &mut done),
                Stmt::For(_, _, Iter::Range { lo, hi, .. }, _) => {
                    self.lift_value(lo, &mut done);
                    self.lift_value(hi, &mut done);
                }
                Stmt::For(_, _, Iter::RangeVal(e, _) | Iter::Seq(e, _), _) => {
                    self.lift_value(e, &mut done);
                }
                _ => {}
            }
            done.push(stmt);
        }
        out.extend(done);
    }

    /// `value` with what leaves the fn worked out in `done` first.
    pub(in crate::lower) fn lift_value(&mut self, value: &mut Expr, done: &mut Vec<Stmt>) {
        let unit = Expr {
            kind: ExprKind::Unit,
            ty: Ty::Unit,
            span: value.span,
        };
        let lifted = self.lifted(std::mem::replace(value, unit));
        *value = if leaves(&lifted) {
            self.ahead(lifted, done)
        } else {
            lifted
        };
    }

    /// The block's statements go to `out`; what it gives is held and read.
    pub(in crate::lower) fn ahead(&mut self, e: Expr, out: &mut Vec<Stmt>) -> Expr {
        let Expr {
            kind: ExprKind::Do(mut block),
            ty,
            span,
        } = e
        else {
            return e;
        };
        if ty == Ty::Unit {
            run_gives(&mut block.stmts);
            out.extend(block.stmts);
            return Expr {
                kind: ExprKind::Unit,
                ty,
                span,
            };
        }
        let slot = self.temp(ty.clone());
        let place = Place {
            kind: PlaceKind::Local(slot),
            ty: ty.clone(),
            span,
        };
        store_gives(&mut block.stmts, &place);
        out.push(Stmt::Let(slot, None));
        out.extend(block.stmts);
        Expr {
            kind: ExprKind::Local(slot),
            ty,
            span,
        }
    }

    /// `e` with every leaving block among its operands brought to the
    /// front: the result is then one such block, or `e` as it was.
    pub(in crate::lower) fn lifted(&mut self, e: Expr) -> Expr {
        let Expr { kind, ty, span } = e;
        let mut parts: Vec<Expr> = Vec::new();
        // The operands that are always worked out, in order; `shape` puts
        // them back.
        let shape: Box<dyn FnOnce(Vec<Expr>) -> ExprKind> = match kind {
            ExprKind::Binary(op, l, r)
                if !matches!(op, BinOp::And | BinOp::Or | BinOp::Coalesce) =>
            {
                parts.extend([*l, *r]);
                Box::new(move |p| {
                    let Ok([l, r]) = <[Expr; 2]>::try_from(p) else {
                        unreachable!("two operands")
                    };
                    ExprKind::Binary(op, Box::new(l), Box::new(r))
                })
            }
            ExprKind::Cmp(op, l, r) => {
                parts.extend([*l, *r]);
                Box::new(move |p| {
                    let Ok([l, r]) = <[Expr; 2]>::try_from(p) else {
                        unreachable!("two operands")
                    };
                    ExprKind::Cmp(op, Box::new(l), Box::new(r))
                })
            }
            ExprKind::Unary(op, x) => {
                parts.push(*x);
                Box::new(move |mut p| ExprKind::Unary(op, Box::new(p.remove(0))))
            }
            ExprKind::Cast(cast, x) => {
                parts.push(*x);
                Box::new(move |mut p| ExprKind::Cast(cast, Box::new(p.remove(0))))
            }
            ExprKind::Some(x) => {
                parts.push(*x);
                Box::new(|mut p| ExprKind::Some(Box::new(p.remove(0))))
            }
            ExprKind::Unwrap(x, checked) => {
                parts.push(*x);
                Box::new(move |mut p| ExprKind::Unwrap(Box::new(p.remove(0)), checked))
            }
            ExprKind::RangeCheck {
                value,
                lo,
                hi,
                text,
            } => {
                let has_lo = lo.is_some();
                let has_hi = hi.is_some();
                parts.push(*value);
                if let Some(l) = lo {
                    parts.push(*l);
                }
                if let Some(h) = hi {
                    parts.push(*h);
                }
                Box::new(move |mut p| {
                    let value = Box::new(p.remove(0));
                    let lo = has_lo.then(|| Box::new(p.remove(0)));
                    let hi = has_hi.then(|| Box::new(p.remove(0)));
                    ExprKind::RangeCheck {
                        value,
                        lo,
                        hi,
                        text,
                    }
                })
            }
            ExprKind::Field(x, at) => {
                parts.push(*x);
                Box::new(move |mut p| ExprKind::Field(Box::new(p.remove(0)), at))
            }
            ExprKind::Tuple(items) => {
                parts = items;
                Box::new(ExprKind::Tuple)
            }
            ExprKind::Index(base, at) => {
                parts.push(*base);
                parts.extend(at);
                Box::new(|mut p| {
                    let base = p.remove(0);
                    ExprKind::Index(Box::new(base), p)
                })
            }
            ExprKind::Call(id, args) => {
                let mut places = Vec::new();
                for arg in args {
                    match arg {
                        Arg::Value(e) => {
                            parts.push(e);
                            places.push(None);
                        }
                        Arg::Mut(place) => places.push(Some(place)),
                    }
                }
                Box::new(move |p| {
                    let mut values = p.into_iter();
                    let args = places
                        .into_iter()
                        .map(|place| match place {
                            Some(place) => Arg::Mut(place),
                            None => Arg::Value(values.next().expect("one value per slot")),
                        })
                        .collect();
                    ExprKind::Call(id, args)
                })
            }
            // A block's own statements may hold values that leave.
            ExprKind::Do(mut block) => {
                self.lift_leaving(&mut block.stmts, 0);
                return Expr {
                    kind: ExprKind::Do(block),
                    ty,
                    span,
                };
            }
            ExprKind::Mutate(how, place, args) => {
                parts = args;
                Box::new(move |p| ExprKind::Mutate(how, place, p))
            }
            ExprKind::Print(args, end) => {
                let digits: Vec<_> = args.iter().map(|arg| arg.digits).collect();
                parts = args.into_iter().map(|arg| arg.value).collect();
                Box::new(move |p| {
                    let args = p
                        .into_iter()
                        .zip(digits)
                        .map(|(value, digits)| FmtArg { value, digits })
                        .collect();
                    ExprKind::Print(args, end)
                })
            }
            ExprKind::Builtin(builtin, args) => {
                parts = args;
                Box::new(move |p| ExprKind::Builtin(builtin, p))
            }
            kind => return Expr { kind, ty, span },
        };
        let mut parts: Vec<Expr> = parts.into_iter().map(|part| self.lifted(part)).collect();
        let Some(last) = parts.iter().rposition(leaves) else {
            return Expr {
                kind: shape(parts),
                ty,
                span,
            };
        };
        let mut stmts = Vec::new();
        for part in &mut parts[..=last] {
            if steady(part) {
                continue;
            }
            let taken = std::mem::replace(
                part,
                Expr {
                    kind: ExprKind::Unit,
                    ty: Ty::Unit,
                    span,
                },
            );
            *part = match leaves(&taken) {
                true => self.ahead(taken, &mut stmts),
                false => {
                    let slot = self.temp(taken.ty.clone());
                    let held = Expr {
                        kind: ExprKind::Local(slot),
                        ty: taken.ty.clone(),
                        span: taken.span,
                    };
                    stmts.push(Stmt::Let(slot, Some(taken)));
                    held
                }
            };
        }
        stmts.push(Stmt::Give(Expr {
            kind: shape(parts),
            ty: ty.clone(),
            span,
        }));
        Expr {
            kind: ExprKind::Do(Block { stmts }),
            ty,
            span,
        }
    }
}

pub(in crate::lower) fn block_leaves(block: &Block) -> bool {
    let mut found = false;
    crate::walk::deep_stmts(block, &mut |stmt| {
        found |= matches!(stmt, Stmt::Return(_) | Stmt::Break(..) | Stmt::Continue(..));
    });
    found
}

/// A name or a field of one: reading it runs nothing, so it is read
/// where it stands and not copied ahead.
pub(in crate::lower) fn steady(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Field(of, _) => steady(of),
        _ => crate::walk::is_simple(e),
    }
}

pub(in crate::lower) fn store_gives(stmts: &mut [Stmt], place: &Place) {
    for stmt in stmts {
        match stmt {
            Stmt::Give(value) => {
                let value = std::mem::replace(
                    value,
                    Expr {
                        kind: ExprKind::Unit,
                        ty: Ty::Unit,
                        span: place.span,
                    },
                );
                *stmt = Stmt::Assign(place.clone(), AssignOp::Set, value);
            }
            Stmt::If(_, then, otherwise) => {
                store_gives(&mut then.stmts, place);
                store_gives(&mut otherwise.stmts, place);
            }
            _ => {}
        }
    }
}

/// A block with nothing to give has its last value run where it gave it.
pub(in crate::lower) fn run_gives(stmts: &mut [Stmt]) {
    for stmt in stmts {
        match stmt {
            Stmt::Give(value) => {
                let unit = Expr {
                    kind: ExprKind::Unit,
                    ty: Ty::Unit,
                    span: value.span,
                };
                *stmt = Stmt::Expr(std::mem::replace(value, unit));
            }
            Stmt::If(_, then, otherwise) => {
                run_gives(&mut then.stmts);
                run_gives(&mut otherwise.stmts);
            }
            _ => {}
        }
    }
}
