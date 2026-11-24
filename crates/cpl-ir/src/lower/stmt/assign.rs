//! Assignment statements: `x = v`, compound `x op= v`, settling index
//! expressions, and storing into a place.

use super::*;
use cpl_diag::msgs;

impl<'a> Lowerer<'a> {
    /// `value` as something cheap to mention twice: itself if it is a
    /// variable, else a temporary that holds it.
    pub(in crate::lower) fn hold(&mut self, value: Expr, out: &mut Vec<Stmt>) -> Expr {
        if names_a_place(&value) {
            return value;
        }
        let span = value.span;
        let slot = self.temp(value.ty.clone());
        let ty = value.ty.clone();
        out.push_let(slot, value);
        Expr {
            kind: ExprKind::Local(slot),
            ty,
            span,
        }
    }

    pub(in crate::lower) fn assign(
        &mut self,
        target: &ast::Expr,
        op: ast::AssignOp,
        value: &ast::Expr,
        out: &mut Vec<Stmt>,
    ) {
        if self.grid_row(target).is_some() {
            // `a[l..r] -= b[l2..r2]`, `a[1..n] min= a[0..n - 1]`, `a[l..r] =
            // b[l2..r2]`: part against part, cell for cell.
            let cellwise = match op {
                ast::AssignOp::Set | ast::AssignOp::Max | ast::AssignOp::Min => true,
                ast::AssignOp::Binary(bin) => {
                    matches!(
                        bin,
                        BinOp::Add
                            | BinOp::Sub
                            | BinOp::Mul
                            | BinOp::BitAnd
                            | BinOp::BitOr
                            | BinOp::BitXor
                    )
                }
            };
            if cellwise && self.grid_row(value).is_some() {
                self.part_map(target, op, value, None, out);
                return;
            }
            if matches!(op, ast::AssignOp::Max | ast::AssignOp::Min) {
                self.grid_keep(target, Some(op == ast::AssignOp::Max), value, false, out);
                return;
            }
            // `a[l..r] = b[l2..r2] + x`: each cell worked out and stored.
            if let (
                ast::AssignOp::Set,
                ast::ExprKind::Binary {
                    op: bin, lhs, rhs, ..
                },
            ) = (op, &value.kind)
            {
                if matches!(
                    bin,
                    BinOp::Add
                        | BinOp::Sub
                        | BinOp::Mul
                        | BinOp::BitAnd
                        | BinOp::BitOr
                        | BinOp::BitXor
                ) {
                    if self.grid_row(lhs).is_some() && self.one_cell(rhs) {
                        self.part_map(target, op, lhs, Some((*bin, rhs, false)), out);
                        return;
                    }
                    if self.grid_row(rhs).is_some() && self.one_cell(lhs) {
                        self.part_map(target, op, rhs, Some((*bin, lhs, true)), out);
                        return;
                    }
                }
            }
            if self.one_cell(value) {
                self.grid_fill(target, op, value, out);
                return;
            }
            // `dp[k, ..] = row`: the row's cells, stored one by one.
            if op == ast::AssignOp::Set {
                self.grid_keep(target, None, value, false, out);
                return;
            }
        }
        if self.index_set(target, op, value, out) {
            return;
        }
        if let (ast::ExprKind::Tuple(targets), ast::AssignOp::Set) = (&target.kind, op) {
            // `(a[d], b[d]) = read()`: the places say what is read.
            let tried: Vec<Option<Place>> = targets.iter().map(|t| self.try_place(t)).collect();
            let wanted: Option<Vec<Ty>> = tried
                .iter()
                .map(|p| p.as_ref().map(|p| p.ty.clone()))
                .collect();
            // A store to a variable or field path runs no code, so a right-hand
            // item that is one too, of a variable no earlier store sets, may be
            // read at its own store, where codegen moves it if a later store
            // sets it whole: `(x, y) = (y, f(x, y))` holds only `f(x, y)`.
            let roots: Option<Vec<(bool, usize)>> = tried
                .iter()
                .map(|p| p.as_ref().and_then(path_place))
                .collect();
            // `(a, b) = (b, a)` of lists: every other right-hand item is held
            // whole before any store, in a temp of its own, so each store moves it in.
            let owning = wanted
                .as_ref()
                .is_some_and(|w| w.iter().any(|t| !t.is_scalar()));
            if let (ast::ExprKind::Tuple(items), true) = (&value.kind, owning) {
                if items.len() == targets.len() {
                    let mut held = Vec::new();
                    for (k, item) in items.iter().enumerate() {
                        let want = wanted.as_ref().map(|w| &w[k]);
                        let item = self.expr(item, want);
                        let unset = roots.as_ref().is_some_and(|roots| {
                            path_expr(&item).is_some_and(|root| !roots[..k].contains(&root))
                        });
                        if unset {
                            held.push(item);
                            continue;
                        }
                        let slot = self.temp(item.ty.clone());
                        held.push(Expr {
                            kind: ExprKind::Local(slot),
                            ty: item.ty.clone(),
                            span: item.span,
                        });
                        out.push(Stmt::Let(slot, Some(item)));
                    }
                    for (t, item) in targets.iter().zip(held) {
                        let place = self.place(t);
                        let item = self.coerce(item, &place.ty, t.span);
                        self.emit_assign(place, AssignOp::Set, item, t.span, out);
                    }
                    return;
                }
            }
            let wanted = wanted.map(Ty::Tuple);
            let value = self.expr(value, wanted.as_ref());
            let Some(fields) = value.ty.fields().filter(|f| f.len() == targets.len()) else {
                if value.ty != Ty::Error {
                    self.report(
                        target.span,
                        msgs::ir::TYPE_MISMATCH_VARIABLES_CAN_T_TAKE,
                        &[&targets.len(), &self.show(&value.ty)],
                    );
                }
                return;
            };
            let slot = self.temp(value.ty.clone());
            let base = Expr {
                kind: ExprKind::Local(slot),
                ty: value.ty.clone(),
                span: value.span,
            };
            out.push_let(slot, value);
            for (i, (t, ty)) in targets.iter().zip(fields).enumerate() {
                let place = self.place(t);
                let field = Expr {
                    kind: ExprKind::Field(Box::new(base.clone()), i),
                    ty,
                    span: base.span,
                };
                let field = self.coerce(field, &place.ty, t.span);
                self.emit_assign(place, AssignOp::Set, field, t.span, out);
            }
            return;
        }
        // Check for assignment through a `lends mut` projection (`m.at(i, j) = v`, `t.at(v) += 1`).
        if let ast::ExprKind::Call { callee, args } = &target.kind {
            if let ast::ExprKind::Field { base, name } = &callee.kind {
                if let Some(proj) = self.find_projection_on(base, &name.text) {
                    self.lower_projection_assign(proj, base, args, op, value, target.span, out);
                    return;
                }
            }
            if let ast::ExprKind::Name(fname) = &callee.kind {
                if self.lookup(fname).is_none() {
                    if let Some((offset, decl)) = self.free_lends_mut_fn(fname) {
                        self.lower_free_lend_assign(
                            decl,
                            offset,
                            args,
                            op,
                            value,
                            target.span,
                            out,
                        );
                        return;
                    }
                }
            }
        }
        let mut post = Vec::new();
        if let Some((through, alias)) = self.through_lend(target, out, &mut post) {
            self.assign(&through, op, value, out);
            out.append(&mut post);
            self.cx.mut_aliases.remove(&alias);
            return;
        }
        let place = self.place(target);
        // Every index of the target is worked out before the right side,
        // and the place is only reached for after both: `trie[grow()].ends
        // = true` and `trie[o].next ??= trie.push_id(Node())` are safe when
        // the list moves.
        let settled = out.len();
        let place = self.settle_indexes(place, out);
        // `a op= b` is `a = a op b` for every type that has `op`: a list,
        // a set, anything `a op b` is written for. The numbers, strings
        // and structs below have a shorter way that reads `a` once.
        if let ast::AssignOp::Binary(bin) = op {
            let short = bin == BinOp::Coalesce
                || matches!(
                    place.ty,
                    Ty::Int
                        | Ty::I128
                        | Ty::U64
                        | Ty::Float
                        | Ty::Bool
                        | Ty::Str
                        | Ty::Struct(_)
                        | Ty::Opt(_)
                        | Ty::Error
                );
            // `xs += ys` grows `xs` where it is; it is not a copy of both.
            if bin == BinOp::Add && matches!(place.ty, Ty::List(_)) {
                let more = self.expr_as(value, &place.ty);
                let extend = ExprKind::Mutate(Mutation::Extend, Box::new(place), vec![more]);
                out.push(Stmt::Expr(self.mk(extend, Ty::Unit, target.span)));
                return;
            }
            // An index that does work is done once, so `a` can't be
            // written out a second time.
            if !short && out.len() == settled {
                let whole = ast::Expr {
                    kind: ast::ExprKind::Binary {
                        op: bin,
                        op_span: target.span,
                        lhs: Box::new(target.clone()),
                        rhs: Box::new(value.clone()),
                    },
                    span: target.span,
                };
                // A step along a domain is checked: `cur += 2` stops the
                // program when it leaves the domain, as `(cur + 2)!` does.
                let rhs = if matches!(place.ty, Ty::Pos(_)) {
                    let stepped = self.expr(&whole, None);
                    if stepped.ty == Ty::opt(place.ty.clone()) {
                        let inside = ExprKind::Unwrap(Box::new(stepped), true);
                        self.mk(inside, place.ty.clone(), target.span)
                    } else {
                        self.coerce(stepped, &place.ty, target.span)
                    }
                } else {
                    self.expr_as(&whole, &place.ty)
                };
                out.push(Stmt::Assign(place, AssignOp::Set, rhs));
                return;
            }
        }
        let rhs = match op {
            ast::AssignOp::Set => self.expr_as(value, &place.ty),
            ast::AssignOp::Binary(BinOp::Coalesce) => match &place.ty {
                Ty::Opt(inner) => {
                    let inner = (**inner).clone();
                    // `(o ??= []).push(x)` goes on with what `o` holds.
                    self.filled
                        .insert((self.cur_mod, target.span.start, target.span.end));
                    self.expr_as(value, &inner)
                }
                Ty::Error => self.expr(value, None),
                // A map entry: `m[k] ??= []` holds what the map holds.
                entry => {
                    let entry = entry.clone();
                    self.expr_as(value, &entry)
                }
            },
            ast::AssignOp::Binary(BinOp::Add) if place.ty == Ty::Str => self.expr(value, None),
            _ => self.expr(value, Some(&place.ty)),
        };
        self.emit_assign_place(place, op, rhs, target.span, out);
    }

    /// Moves each index of `place` that does work (a call, a push) into a
    /// local declared in `out`, left to right, so the place can be named
    /// again without doing the work twice.
    pub(in crate::lower) fn settle_indexes(&mut self, place: Place, out: &mut Vec<Stmt>) -> Place {
        let Place { kind, ty, span } = place;
        let kind = match kind {
            PlaceKind::Index(base, indexes) => {
                let base = self.settle_indexes(*base, out);
                let indexes = indexes
                    .into_iter()
                    .map(|index| {
                        if settled(&index) || self.decoded_from(&index).is_some() {
                            return index;
                        }
                        let wrap = match &index.kind {
                            ExprKind::Cast(cast @ (Cast::Raw | Cast::Trust), _) => Some(*cast),
                            _ => None,
                        };
                        let slot = self.temp(index.ty.clone());
                        let ispan = index.span;
                        let ity = index.ty.clone();
                        let held = Expr {
                            kind: ExprKind::Local(slot),
                            ty: ity.clone(),
                            span: ispan,
                        };
                        out.push_let(slot, index);
                        match wrap {
                            Some(cast) => Expr {
                                kind: ExprKind::Cast(cast, Box::new(held)),
                                ty: ity,
                                span: ispan,
                            },
                            None => held,
                        }
                    })
                    .collect();
                PlaceKind::Index(Box::new(base), indexes)
            }
            PlaceKind::Field(base, field) => {
                PlaceKind::Field(Box::new(self.settle_indexes(*base, out)), field)
            }
            other => other,
        };
        Place { kind, ty, span }
    }

    /// `m.remove(k)` for its effect alone: the value that was there is
    /// not asked for, so it is not looked up. False when `e` is not that.
    pub(in crate::lower) fn drop_key(&mut self, e: &ast::Expr, out: &mut Vec<Stmt>) -> bool {
        let ast::ExprKind::Call { callee, args } = &e.kind else {
            return false;
        };
        let ast::ExprKind::Field { base, name } = &callee.kind else {
            return false;
        };
        let [ast::Arg {
            name: None,
            value: key,
            ..
        }] = args.as_slice()
        else {
            return false;
        };
        if name.text != "remove" {
            return false;
        }
        let Some(Ty::Map(key_ty, _) | Ty::HashMap(key_ty, _)) = self.probe_ty_silent(base) else {
            return false;
        };
        let place = self.place(base);
        let place = self.settle_indexes(place, out);
        if let PlaceKind::Local(s) = root_place(&place) {
            self.cx.mut_slots.insert(s);
        }
        let key = self.expr_as(key, &key_ty);
        let key = self.hold(key, out);
        let remove = self.mk_mutate(Mutation::Remove, place, vec![key], Ty::Bool, e.span);
        out.push(Stmt::Expr(remove));
        true
    }

    pub(in crate::lower) fn emit_assign_place(
        &mut self,
        place: Place,
        op: ast::AssignOp,
        value: Expr,
        span: Span,
        out: &mut Vec<Stmt>,
    ) {
        // Track local mutations for closure capture analysis.
        if let PlaceKind::Local(s) = root_place(&place) {
            self.cx.mut_slots.insert(s);
        }
        let (op, value) = match op {
            ast::AssignOp::Set => (AssignOp::Set, self.coerce(value, &place.ty, span)),
            ast::AssignOp::Max | ast::AssignOp::Min if self.is_row(&place.ty) => {
                self.row_keep(place, op == ast::AssignOp::Max, value, false, span, out);
                return;
            }
            ast::AssignOp::Max | ast::AssignOp::Min => {
                if let Ty::Opt(inner) = place.ty.clone() {
                    if !matches!(*inner, Ty::Error) && !self.ordered(&inner) {
                        self.report(
                            span,
                            msgs::ir::BAD_OPERANDS_MAX_AND_MIN_NEED,
                            &[&self.show(&place.ty)],
                        );
                        return;
                    }
                    let is_max = op == ast::AssignOp::Max;
                    let rhs = self.coerce(value, &place.ty, span);
                    if is_max && !self.user_ordered(&inner) {
                        // `none` sorts below every value, so `max=` is the max of that order.
                        out.push(Stmt::Assign(place, AssignOp::Max, rhs));
                        return;
                    }
                    let (r_some, r_val) = if let ExprKind::Some(x) = rhs.kind {
                        // `p min= Some(e)`: the value is there, so only `p` is tested.
                        let slot = self.temp((*inner).clone());
                        out.push_let(slot, *x);
                        (None, self.local(slot, span))
                    } else {
                        let r_slot = self.temp(place.ty.clone());
                        out.push_let(r_slot, rhs);
                        let r_expr = self.local(r_slot, span);
                        let r_some = self.mk_is_some(r_expr.clone(), Ty::Bool, span);
                        let r_val = self.mk_unwrap(r_expr, false, (*inner).clone(), span);
                        (Some(r_some), r_val)
                    };
                    let p_expr = place_to_expr(&place);
                    let p_some = self.mk_is_some(p_expr.clone(), Ty::Bool, span);
                    let p_none = self.mk_unary(UnOp::Not, p_some, Ty::Bool, span);
                    let p_val = self.mk_unwrap(p_expr, false, (*inner).clone(), span);
                    let cmp_op = if is_max { CmpOp::Gt } else { CmpOp::Lt };
                    let better = self.mk_cmp(cmp_op, r_val.clone(), p_val, Ty::Bool, span);
                    let should_set = self.mk_binary(BinOp::Or, p_none, better, Ty::Bool, span);
                    let set_stmt = Stmt::Assign(
                        place,
                        AssignOp::Set,
                        self.mk_some(r_val, Ty::Opt(inner), span),
                    );
                    let keep = Stmt::If(
                        should_set,
                        Block {
                            stmts: vec![set_stmt],
                        },
                        Block::default(),
                    );
                    out.push(match r_some {
                        Some(r_some) => {
                            Stmt::If(r_some, Block { stmts: vec![keep] }, Block::default())
                        }
                        None => keep,
                    });
                    return;
                }
                // `best min= cand` with a `?T` on the right skips `none`.
                if matches!(&value.ty, Ty::Opt(inner) if **inner == place.ty)
                    && self.ordered(&place.ty)
                {
                    let op = if op == ast::AssignOp::Max {
                        AssignOp::Max
                    } else {
                        AssignOp::Min
                    };
                    let slot = self.temp(value.ty.clone());
                    out.push_let(slot, value);
                    let held = self.local(slot, span);
                    let some = self.mk_is_some(held.clone(), Ty::Bool, span);
                    let inside = self.mk_unwrap(held, false, place.ty.clone(), span);
                    let mut branch = Vec::new();
                    self.emit_assign(place, op, inside, span, &mut branch);
                    out.push(Stmt::If(some, Block { stmts: branch }, Block::default()));
                    return;
                }
                if !matches!(place.ty, Ty::Error) && !self.ordered(&place.ty) {
                    let word = if op == ast::AssignOp::Max {
                        "max="
                    } else {
                        "min="
                    };
                    let message = match &place.ty {
                        Ty::Array { dims, .. } if dims.len() > 1 => {
                            cpl_diag::msgs::ir::WORKS_ONE_ROW_AT.at(&[&word, &word])
                        }
                        Ty::Array { elem, .. } | Ty::List(elem) if matches!(**elem, Ty::Opt(_)) => {
                            cpl_diag::msgs::ir::WHOLE_ROWS_NEEDS_CELLS.at(&[
                                &word,
                                &self.show(elem),
                                &word,
                            ])
                        }
                        _ => cpl_diag::msgs::ir::BAD_OPERANDS_MAX_AND_MIN_NEED
                            .at(&[&self.show(&place.ty)]),
                    };
                    self.report_said(span, message);
                }
                let op = if op == ast::AssignOp::Max {
                    AssignOp::Max
                } else {
                    AssignOp::Min
                };
                (op, self.coerce(value, &place.ty, span))
            }
            ast::AssignOp::Binary(bin) => {
                // Handle `+=`, `-=`, `*=`, `/=` on structs implementing `Add`/`Sub`/`Mul`/`Div`.
                if let Ty::Struct(sid) = place.ty {
                    let mname = match bin {
                        BinOp::Add => Some("add"),
                        BinOp::Sub => Some("sub"),
                        BinOp::Mul => Some("mul"),
                        BinOp::Div => Some("div"),
                        BinOp::BitOr => Some("bitor"),
                        BinOp::BitAnd => Some("bitand"),
                        BinOp::BitXor => Some("bitxor"),
                        _ => None,
                    };
                    // `z **= e` is `z = z.pow(e)`, as `z ** e` is.
                    let pow = match (bin, &value.ty) {
                        (BinOp::Pow, Ty::Int) => self.own_method(sid, "pow"),
                        _ => None,
                    };
                    if let Some(fn_id) = pow {
                        self.queue_fn(fn_id);
                        let ret = self.ret_of(fn_id, span);
                        let call = Expr {
                            kind: ExprKind::Call(
                                fn_id,
                                vec![Arg::Value(place_to_expr(&place)), Arg::Value(value)],
                            ),
                            ty: ret,
                            span: self.span(span),
                        };
                        let call = self.coerce(call, &place.ty, span);
                        out.push(Stmt::Assign(place, AssignOp::Set, call));
                        return;
                    }
                    if let Some(fn_id) = mname.and_then(|m| self.struct_op_fn(sid, m)) {
                        let lhs_expr = place_to_expr(&place);
                        let rhs_val = self.coerce(value, &place.ty, span);
                        let ret = self.ret_of(fn_id, span);
                        let call = Expr {
                            kind: ExprKind::Call(
                                fn_id,
                                vec![Arg::Value(lhs_expr), Arg::Value(rhs_val)],
                            ),
                            ty: ret,
                            span: self.span(span),
                        };
                        let call = self.coerce(call, &place.ty, span);
                        out.push(Stmt::Assign(place, AssignOp::Set, call));
                        return;
                    }
                }
                use BinOp::*;
                // `x ??= init`: `init` runs only when `x` is `none`.
                if bin == Coalesce {
                    // `m[k] ??= init`: a key the map does not have gets
                    // `init`.
                    if let PlaceKind::Index(base, keys) = &place.kind {
                        if matches!(base.ty, Ty::Map(..) | Ty::HashMap(..)) {
                            let init = self.coerce(value, &place.ty, span);
                            let has = self.mk_builtin(
                                Builtin::Contains,
                                [place_to_expr(base), keys[0].clone()],
                                Ty::Bool,
                                span,
                            );
                            let missing = self.mk_unary(UnOp::Not, has, Ty::Bool, span);
                            let slot = self.temp(place.ty.clone());
                            let fresh = self.local(slot, span);
                            out.push(Stmt::If(
                                missing,
                                Block {
                                    stmts: vec![
                                        Stmt::Let(slot, Some(init)),
                                        Stmt::Assign(place, AssignOp::Set, fresh),
                                    ],
                                },
                                Block::default(),
                            ));
                            return;
                        }
                    }
                    let Ty::Opt(inner) = place.ty.clone() else {
                        if place.ty != Ty::Error {
                            self.report(
                                span,
                                msgs::ir::BAD_OPERANDS_FILLS_IN_AN_OPTIONAL,
                                &[&self.show(&place.ty)],
                            );
                        }
                        return;
                    };
                    let init = self.coerce(value, &inner, span);
                    let held = place_to_expr(&place);
                    let some = self.mk_is_some(held, Ty::Bool, span);
                    let missing = self.mk_unary(UnOp::Not, some, Ty::Bool, span);
                    // The value is whole before the place is reached for.
                    let slot = self.temp((*inner).clone());
                    let fresh = self.local(slot, span);
                    let filled = self.mk_some(fresh, place.ty.clone(), span);
                    out.push(Stmt::If(
                        missing,
                        Block {
                            stmts: vec![
                                Stmt::Let(slot, Some(init)),
                                Stmt::Assign(place, AssignOp::Set, filled),
                            ],
                        },
                        Block::default(),
                    ));
                    return;
                }
                // `none` does not pass through `+=` and `-=`: a `none` that
                // stays `none` hides a table that was never filled in.
                if let (Ty::Opt(inner), Add | Sub) = (&place.ty, bin) {
                    if matches!(**inner, Ty::Int | Ty::Float) {
                        let help = format!(
                            "write `x = x {} y` when `none` should stay `none`, or `x min= y + w` for a transition",
                            bin.text()
                        );
                        self.report(
                            span,
                            msgs::ir::BAD_OPERANDS_CAN_T_BE_APPLIED_2,
                            &[&bin.text(), &self.show(&place.ty)],
                        );
                        self.help(help);
                        return;
                    }
                }
                // `s += c` appends in place; `s + c` would build a new string.
                if let (Ty::Str, Add, Ty::Char) = (&place.ty, bin, &value.ty) {
                    let push = ExprKind::Mutate(Mutation::Push, Box::new(place), vec![value]);
                    out.push(Stmt::Expr(self.mk(push, Ty::Unit, span)));
                    return;
                }
                let value = match (&place.ty, bin) {
                    (
                        Ty::Int,
                        Add | Sub | Mul | Div | Rem | Pow | BitAnd | BitOr | BitXor | Shl | Shr,
                    ) => self.coerce(value, &Ty::Int, span),
                    (Ty::I128 | Ty::U64, Shl | Shr) => self.coerce(value, &Ty::Int, span),
                    (
                        wide @ (Ty::I128 | Ty::U64),
                        Add | Sub | Mul | Div | Rem | Pow | BitAnd | BitOr | BitXor,
                    ) => self.coerce(value, &wide.clone(), span),
                    (Ty::Float, Add | Sub | Mul | Div) => self.coerce(value, &Ty::Float, span),
                    (Ty::Bool, BitAnd | BitOr | BitXor) => self.coerce(value, &Ty::Bool, span),
                    (Ty::Str, Add) => self.as_str(value, span),
                    (Ty::Error, _) => value,
                    _ => {
                        self.report(
                            span,
                            msgs::ir::BAD_OPERANDS_CAN_T_BE_APPLIED_2,
                            &[&bin.text(), &self.show(&place.ty)],
                        );
                        value
                    }
                };
                self.compound_step(&place, bin, value, span)
            }
        };
        self.emit_assign(place, op, value, span, out);
    }
}

/// The variable a place of variables and fields is part of; `None` for an
/// element, whose index may run code.
fn path_place(place: &Place) -> Option<(bool, usize)> {
    match &place.kind {
        PlaceKind::Local(slot) => Some((false, *slot)),
        PlaceKind::Global(id) => Some((true, *id)),
        PlaceKind::Field(base, _) => path_place(base),
        PlaceKind::Index(..) | PlaceKind::Some(_) => None,
    }
}

/// The variable an expression of variables and fields reads.
fn path_expr(e: &Expr) -> Option<(bool, usize)> {
    match &e.kind {
        ExprKind::Local(slot) => Some((false, *slot)),
        ExprKind::Global(id) => Some((true, *id)),
        ExprKind::Field(base, _) => path_expr(base),
        _ => None,
    }
}
