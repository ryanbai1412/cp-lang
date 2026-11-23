//! Walk items, searches and method-call dispatch: `for (k, x) in xs.items`,
//! `xs.find(..)`, `x.m(..)` on a user type.

use super::*;

impl<'a> Lowerer<'a> {
    /// `g.items` in a `for`, when `g` names a grid: one loop over the
    /// cells, with a counter per axis stepped at the top of each round, so
    /// no list of pairs is built. The counters are set up in `out`.
    pub(in crate::lower) fn grid_items_head(
        &mut self,
        e: &ast::Expr,
        out: &mut Vec<Stmt>,
    ) -> Option<GridItems> {
        let ast::ExprKind::Field { base, name } = &e.kind else {
            return None;
        };
        if name.text != "items" || !is_place(base) {
            return None;
        }
        match self.probe_ty_silent(base) {
            Some(Ty::Array { dims, .. })
                if dims.len() >= 2 && !dims.iter().any(|d| matches!(d, Dim::Bound(_))) => {}
            _ => return None,
        }
        let span = e.span;
        let grid = self.expr(base, None);
        let Ty::Array { dims, elem } = grid.ty.clone() else {
            return None;
        };
        let mut lens = Vec::new();
        let mut total = self.int_lit(1, span);
        for axis in 0..dims.len() {
            let len = self.mk(
                ExprKind::Builtin(
                    Builtin::AxisLen,
                    vec![grid.clone(), self.int_lit(axis as i64, span)],
                ),
                Ty::Int,
                span,
            );
            let len = self.hold(len, out);
            total = self.mk(
                ExprKind::Binary(BinOp::Mul, Box::new(total), Box::new(len.clone())),
                Ty::Int,
                span,
            );
            lens.push(len);
        }
        let total = self.hold(total, out);
        // The last counter starts one before the first cell.
        let last = dims.len() - 1;
        let counters: Vec<Slot> = (0..dims.len())
            .map(|axis| {
                let k = self.temp(Ty::Int);
                let start = self.int_lit(if axis == last { -1 } else { 0 }, span);
                out.push(Stmt::Let(k, Some(start)));
                k
            })
            .collect();
        Some(GridItems {
            grid,
            dims,
            elem: *elem,
            lens,
            total,
            counters,
        })
    }

    /// Steps the counters of `for .. in g.items` and binds its pattern;
    /// gives the slot the loop itself counts in.
    pub(in crate::lower) fn grid_items_bind(
        &mut self,
        pat: &ast::Pattern,
        items: &GridItems,
        body: &mut Vec<Stmt>,
    ) -> Slot {
        let span = pat.span;
        let place = |lw: &Self, k: Slot| Place {
            kind: PlaceKind::Local(k),
            ty: Ty::Int,
            span: lw.span(span),
        };
        // `c += 1`, and at the end of the row `c = 0` and the same one axis up.
        let mut step: Vec<Stmt> = Vec::new();
        for (axis, &k) in items.counters.iter().enumerate() {
            let bump = Stmt::Assign(
                place(self, k),
                AssignOp::Binary(BinOp::Add),
                self.int_lit(1, span),
            );
            if axis == 0 {
                step = vec![bump];
                continue;
            }
            let full = self.mk(
                ExprKind::Cmp(
                    CmpOp::Eq,
                    Box::new(self.local(k, span)),
                    Box::new(items.lens[axis].clone()),
                ),
                Ty::Bool,
                span,
            );
            let mut wrap = vec![Stmt::Assign(
                place(self, k),
                AssignOp::Set,
                self.int_lit(0, span),
            )];
            wrap.extend(step);
            step = vec![
                bump,
                Stmt::If(full, Block { stmts: wrap }, Block::default()),
            ];
        }
        body.extend(step);
        let mut named = Vec::new();
        let mut stored = Vec::new();
        for (dim, &k) in items.dims.iter().zip(&items.counters) {
            let k = self.local(k, span);
            match *dim {
                Dim::Pos(d) => {
                    // The counter stays under the axis's length.
                    let at = self.mk(ExprKind::Cast(Cast::Trust, Box::new(k)), Ty::Pos(d), span);
                    named.push(at.clone());
                    stored.push(at);
                }
                Dim::IntFrom(first) => {
                    named.push(self.int_add(k.clone(), first));
                    stored.push(k);
                }
                _ => {
                    named.push(k.clone());
                    stored.push(k);
                }
            }
        }
        let at_ty = Ty::Tuple(named.iter().map(|e| e.ty.clone()).collect());
        let at = self.mk(ExprKind::Tuple(named), at_ty.clone(), span);
        let cell = self.mk(
            ExprKind::Index(Box::new(items.grid.clone()), stored),
            items.elem.clone(),
            span,
        );
        match &pat.kind {
            ast::PatternKind::Tuple(parts) if parts.len() == 2 => {
                self.bind(&parts[0], at, body);
                self.bind(&parts[1], cell, body);
            }
            _ => {
                let ty = Ty::Tuple(vec![at_ty, items.elem.clone()]);
                let both = self.mk(ExprKind::Tuple(vec![at, cell]), ty, span);
                self.bind(pat, both, body);
            }
        }
        self.temp(Ty::Int)
    }

    /// The index and the element `items` has at the counter `k`.
    pub(in crate::lower) fn item_at(&mut self, items: &Items, k: Slot, span: Span) -> (Expr, Expr) {
        let k = self.local(k, span);
        if let Some(lo) = &items.lo {
            let at = ExprKind::Binary(BinOp::Add, Box::new(lo.clone()), Box::new(k));
            let at = self.mk(at, Ty::Int, span);
            let at = match items.index {
                Ty::Pos(d) => self.mk(
                    ExprKind::Cast(Cast::FromOrd(d), Box::new(at)),
                    Ty::Pos(d),
                    span,
                ),
                _ => at,
            };
            return (at.clone(), at);
        }
        let (at, stored) = match items.index {
            Ty::Pos(d) if items.keyed => {
                let key = self.mk(
                    ExprKind::Builtin(Builtin::PosKey, vec![items.seq.clone(), k.clone()]),
                    Ty::Int,
                    span,
                );
                (self.cast(Cast::Trust, key, Ty::Pos(d)), k)
            }
            Ty::Pos(d) => {
                let at = self.mk(
                    ExprKind::Cast(Cast::FromOrd(d), Box::new(k)),
                    Ty::Pos(d),
                    span,
                );
                (at.clone(), at)
            }
            _ if items.keyed => {
                let at = self.mk(
                    ExprKind::Builtin(Builtin::PosKey, vec![items.seq.clone(), k.clone()]),
                    Ty::Int,
                    span,
                );
                (at, k)
            }
            _ if items.first == 0 => (k.clone(), k),
            _ => (self.int_add(k.clone(), items.first), k),
        };
        let cell = self.mk(
            ExprKind::Index(Box::new(items.seq.clone()), vec![stored]),
            items.elem.clone(),
            span,
        );
        (at, cell)
    }

    /// Binds the pattern of `for (i, x) in xs.items` and gives the
    /// loop's counter.
    pub(in crate::lower) fn items_bind(
        &mut self,
        pat: &ast::Pattern,
        items: &Items,
        body: &mut Vec<Stmt>,
    ) -> Slot {
        let k = self.temp(Ty::Int);
        let (at, cell) = self.item_at(items, k, pat.span);
        match &pat.kind {
            ast::PatternKind::Tuple(parts) if parts.len() == 2 => {
                self.bind(&parts[0], at, body);
                self.bind(&parts[1], cell, body);
            }
            _ => {
                let ty = Ty::Tuple(vec![items.index.clone(), items.elem.clone()]);
                let both = self.mk(ExprKind::Tuple(vec![at, cell]), ty, pat.span);
                self.bind(pat, both, body);
            }
        }
        k
    }

    /// `xs.find(x => ..)`: the first of a range, list or array the test
    /// holds for, `none` when there is none.
    pub(in crate::lower) fn find_first(
        &mut self,
        value: Expr,
        elem: Ty,
        test: &ast::Expr,
        rev: bool,
        span: Span,
    ) -> Expr {
        let test_ty = Ty::Fn(vec![elem.clone()], Box::new(Ty::Bool));
        let test = self.expect(test, &test_ty);
        let found_ty = Ty::opt(elem.clone());
        self.do_expr(Some(&found_ty), span, |lw, out| {
            let holds = lw.temp(test_ty);
            out.push(Stmt::Let(holds, Some(test)));
            let found = lw.temp(found_ty.clone());
            out.push(Stmt::Let(
                found,
                Some(lw.mk(ExprKind::None, found_ty.clone(), span)),
            ));
            let item = lw.temp(elem.clone());
            let lid = lw.fresh_loop();
            let asked = vec![Arg::Value(lw.local(item, span))];
            let fits = lw.mk(
                ExprKind::CallExpr(Box::new(lw.local(holds, span)), asked),
                Ty::Bool,
                span,
            );
            let some = lw.mk(
                ExprKind::Some(Box::new(lw.local(item, span))),
                found_ty.clone(),
                span,
            );
            let place = Place {
                kind: PlaceKind::Local(found),
                ty: found_ty.clone(),
                span: lw.span(span),
            };
            let hit = Block {
                stmts: vec![Stmt::Assign(place, AssignOp::Set, some), Stmt::Break(None)],
            };
            let body = Block {
                stmts: vec![Stmt::If(fits, hit, Block::default())],
            };
            let iter = match value.kind {
                ExprKind::Range { lo, hi } => Iter::Range {
                    lo: *lo,
                    hi: *hi,
                    step: None,
                    rev,
                    closed: false,
                },
                _ if matches!(value.ty, Ty::Range(_)) => Iter::RangeVal(value, rev),
                _ => Iter::Seq(value, rev),
            };
            out.push(Stmt::For(lid, item, iter, body));
            out.push(Stmt::Give(lw.local(found, span)));
        })
    }

    /// Whether `e` is a test to search with (`x => x > 3`, or a fn value)
    /// rather than a value to search for.
    pub(in crate::lower) fn is_test(&mut self, e: &ast::Expr) -> bool {
        matches!(e.kind, ast::ExprKind::Lambda { .. })
            || matches!(self.probe_ty_silent(e), Some(Ty::Fn(..)))
    }

    /// Lowers `xs.first_ge(val, [by:])`, `xs.first_gt(val, [by:])`, `xs.last_le(val, [by:])`,
    /// `xs.last_lt(val, [by:])`, or `xs.find(val)` on any `Indexed` sequence.
    pub(in crate::lower) fn seq_search_method(
        &mut self,
        method: &str,
        container: Expr,
        elem: Ty,
        dom: Option<DomainId>,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let by_key: Option<&ast::Expr> = if method != "find" && method != "rfind" {
            match args {
                [first, second]
                    if first.name.is_none()
                        && !first.mutable
                        && second.name.as_ref().is_some_and(|n| n.text == "by")
                        && !second.mutable =>
                {
                    Some(&second.value)
                }
                _ => None,
            }
        } else {
            None
        };
        let target = if by_key.is_some() {
            let t = self.expr(&args[0].value, None);
            if t.ty == Ty::Error {
                return self.err_expr(span);
            }
            t
        } else {
            let Some(t) = self.one_arg(method, args, &elem, span) else {
                return self.err_expr(span);
            };
            t
        };
        let target_ty = target.ty.clone();
        let idx_ty = match dom {
            Some(d) => Ty::Pos(d),
            None => Ty::Int,
        };
        let out_ty = Ty::opt(idx_ty.clone());
        let out_ty_hint = out_ty.clone();
        self.do_expr(Some(&out_ty_hint), span, |lw, out| {
            // `vals.first_ge(x)` searches `vals` where it is: holding a
            // field would copy the whole list on every call.
            let c_ref = lw.hold(container, out);
            let t_slot = lw.temp(target_ty.clone());
            out.push(Stmt::Let(t_slot, Some(target)));
            let t_ref = lw.local(t_slot, span);
            let n_expr = match dom {
                Some(d) => lw.mk(ExprKind::DomainLen(d), Ty::Int, span),
                None => lw.len_of(c_ref.clone(), span),
            };
            let n_slot = lw.temp(Ty::Int);
            out.push(Stmt::Let(n_slot, Some(n_expr)));
            let n_ref = lw.local(n_slot, span);
            let make_idx = |lw: &mut Self, i: Expr| -> Expr {
                match dom {
                    Some(d) => lw.mk(
                        ExprKind::Cast(Cast::FromOrd(d), Box::new(i)),
                        Ty::Pos(d),
                        span,
                    ),
                    None => i,
                }
            };
            if method == "find" || method == "rfind" {
                let res_slot = lw.temp(out_ty.clone());
                out.push(Stmt::Let(
                    res_slot,
                    Some(lw.mk(ExprKind::None, out_ty.clone(), span)),
                ));
                let i_slot = lw.temp(Ty::Int);
                let i_ref = lw.local(i_slot, span);
                let pos_e = make_idx(lw, i_ref);
                let cur = lw.mk(
                    ExprKind::Index(Box::new(c_ref), vec![pos_e.clone()]),
                    elem,
                    span,
                );
                let eq = lw.mk(
                    ExprKind::Cmp(CmpOp::Eq, Box::new(cur), Box::new(t_ref)),
                    Ty::Bool,
                    span,
                );
                let lid = lw.next_loop;
                lw.next_loop += 1;
                out.push(Stmt::For(
                    lid,
                    i_slot,
                    Iter::Range {
                        lo: lw.int_lit(0, span),
                        hi: n_ref,
                        step: None,
                        rev: method == "rfind",
                        closed: false,
                    },
                    Block {
                        stmts: vec![Stmt::If(
                            eq,
                            Block {
                                stmts: vec![
                                    Stmt::Assign(
                                        Place {
                                            kind: PlaceKind::Local(res_slot),
                                            ty: out_ty.clone(),
                                            span,
                                        },
                                        AssignOp::Set,
                                        lw.mk(
                                            ExprKind::Some(Box::new(pos_e)),
                                            out_ty.clone(),
                                            span,
                                        ),
                                    ),
                                    Stmt::Break(None),
                                ],
                            },
                            Block::default(),
                        )],
                    },
                ));
                out.push(Stmt::Give(lw.local(res_slot, span)));
            } else {
                // Binary search:
                // - `first_ge` / `last_lt`: partition at first `i` with `c[i] >= t`.
                // - `first_gt` / `last_le`: partition at first `i` with `c[i] > t`.
                let cmp_ge = matches!(method, "first_ge" | "last_lt");
                let is_first = matches!(method, "first_ge" | "first_gt");
                let lo_slot = lw.temp(Ty::Int);
                let hi_slot = lw.temp(Ty::Int);
                out.push(Stmt::Let(lo_slot, Some(lw.int_lit(0, span))));
                out.push(Stmt::Let(hi_slot, Some(n_ref.clone())));
                let lo_ref = lw.local(lo_slot, span);
                let hi_ref = lw.local(hi_slot, span);
                let cond = lw.mk(
                    ExprKind::Cmp(
                        CmpOp::Lt,
                        Box::new(lo_ref.clone()),
                        Box::new(hi_ref.clone()),
                    ),
                    Ty::Bool,
                    span,
                );
                let sum = lw.mk(
                    ExprKind::Binary(
                        BinOp::Add,
                        Box::new(lo_ref.clone()),
                        Box::new(hi_ref.clone()),
                    ),
                    Ty::Int,
                    span,
                );
                let mid = lw.mk(
                    ExprKind::Binary(BinOp::Div, Box::new(sum), Box::new(lw.int_lit(2, span))),
                    Ty::Int,
                    span,
                );
                let mid_slot = lw.temp(Ty::Int);
                let mid_ref = lw.local(mid_slot, span);
                let mid_pos = make_idx(lw, mid_ref.clone());
                let cur = lw.mk(ExprKind::Index(Box::new(c_ref), vec![mid_pos]), elem, span);
                let mut loop_stmts = vec![Stmt::Let(mid_slot, Some(mid))];
                let cur_val = if let Some(key_ast) = by_key {
                    let Some(k) = lw.apply_key(key_ast, cur, &mut loop_stmts, span) else {
                        out.push(Stmt::Give(lw.err_expr(span)));
                        return;
                    };
                    k
                } else {
                    cur
                };
                let t_cmp = lw.coerce(t_ref, &cur_val.ty, args[0].value.span);
                let go_left = if cmp_ge {
                    lw.mk(
                        ExprKind::Cmp(CmpOp::Ge, Box::new(cur_val), Box::new(t_cmp)),
                        Ty::Bool,
                        span,
                    )
                } else {
                    lw.mk(
                        ExprKind::Cmp(CmpOp::Gt, Box::new(cur_val), Box::new(t_cmp)),
                        Ty::Bool,
                        span,
                    )
                };
                loop_stmts.push(Stmt::If(
                    go_left,
                    Block {
                        stmts: vec![Stmt::Assign(
                            Place {
                                kind: PlaceKind::Local(hi_slot),
                                ty: Ty::Int,
                                span,
                            },
                            AssignOp::Set,
                            mid_ref.clone(),
                        )],
                    },
                    Block {
                        stmts: vec![Stmt::Assign(
                            Place {
                                kind: PlaceKind::Local(lo_slot),
                                ty: Ty::Int,
                                span,
                            },
                            AssignOp::Set,
                            lw.int_add(mid_ref, 1),
                        )],
                    },
                ));
                let lid = lw.next_loop;
                lw.next_loop += 1;
                out.push(Stmt::While(lid, cond, Block { stmts: loop_stmts }));
                let res = if is_first {
                    let ok = lw.mk(
                        ExprKind::Cmp(CmpOp::Lt, Box::new(lo_ref.clone()), Box::new(n_ref)),
                        Ty::Bool,
                        span,
                    );
                    let pos = make_idx(lw, lo_ref);
                    lw.mk(
                        ExprKind::If(
                            Box::new(ok),
                            Box::new(lw.mk(ExprKind::Some(Box::new(pos)), out_ty.clone(), span)),
                            Box::new(lw.mk(ExprKind::None, out_ty.clone(), span)),
                        ),
                        out_ty,
                        span,
                    )
                } else {
                    let ok = lw.mk(
                        ExprKind::Cmp(
                            CmpOp::Gt,
                            Box::new(lo_ref.clone()),
                            Box::new(lw.int_lit(0, span)),
                        ),
                        Ty::Bool,
                        span,
                    );
                    let pos = make_idx(lw, lw.int_add(lo_ref, -1));
                    lw.mk(
                        ExprKind::If(
                            Box::new(ok),
                            Box::new(lw.mk(ExprKind::Some(Box::new(pos)), out_ty.clone(), span)),
                            Box::new(lw.mk(ExprKind::None, out_ty.clone(), span)),
                        ),
                        out_ty,
                        span,
                    )
                };
                out.push(Stmt::Give(res));
            }
        })
    }

    /// Calls method `fn_id`. With a receiver, `self` is the caller's own
    /// variable when the method changes it.
    pub(in crate::lower) fn method_call(
        &mut self,
        fn_id: FnId,
        receiver: Option<(&ast::Expr, Expr)>,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        if !self.ensure_sig(fn_id, Some(span)) {
            return self.err_expr(span);
        }
        let name = self.sigs[fn_id].decl.name.text.clone();
        let declared_mut = self.sigs[fn_id]
            .decl
            .params
            .first()
            .is_some_and(|p| p.mutable);
        let receiver = match (self.sigs[fn_id].receiver, receiver) {
            // A method that grows a cache, called on a value with no home
            // (`Squares([]).at(4)`): the value is held for the call.
            (Some(true), Some((base, value))) if !declared_mut && !is_place(base) => {
                let mut ty = Ty::Unit;
                let mut call = self.do_expr(None, span, |lw, out| {
                    let held = lw.temp(value.ty.clone());
                    let place = Place {
                        kind: PlaceKind::Local(held),
                        ty: value.ty.clone(),
                        span,
                    };
                    out.push(Stmt::Let(held, Some(value)));
                    let call = lw.user_call(fn_id, Some(Arg::Mut(place)), args, span);
                    ty = call.ty.clone();
                    out.push(if ty == Ty::Unit {
                        Stmt::Expr(call)
                    } else {
                        Stmt::Give(call)
                    });
                });
                call.ty = ty;
                return call;
            }
            (Some(true), Some((base, _))) => Some(Arg::Mut(self.mut_receiver(base, &name))),
            (Some(false), Some((_, value))) => Some(Arg::Value(value)),
            (None, None) => None,
            (Some(_), None) => {
                let message = format!("`{name}` is called on a value: `x.{name}(...)`");
                self.error(codes::WRONG_ARG_COUNT, span, message);
                return self.err_expr(span);
            }
            (None, Some(_)) => {
                let message = format!("`{name}` takes no `self`: call it on the type");
                self.error(codes::WRONG_ARG_COUNT, span, message);
                return self.err_expr(span);
            }
        };
        self.user_call(fn_id, receiver, args, span)
    }

    /// The judge's side of the conversation: `sol.send(a, b)` is a line
    /// the solution reads, and `sol.recv()` is the next thing it printed.
    pub(in crate::lower) fn talk(
        &mut self,
        name: &ast::Name,
        args: &[ast::Arg],
        hint: Option<&Ty>,
        span: Span,
    ) -> Expr {
        if name.text == "send" {
            let values: Vec<(Expr, Span)> = args
                .iter()
                .map(|arg| (self.expr(&arg.value, None), arg.value.span))
                .collect();
            let values: Vec<FmtArg> = values
                .into_iter()
                .filter_map(|(value, at)| self.fmt_arg(value, crate::FLOAT_DIGITS, at))
                .collect();
            return self.mk(ExprKind::Print(values, None), Ty::Unit, span);
        }
        if !args.is_empty() {
            self.error(codes::WRONG_ARG_COUNT, span, "`recv` takes no arguments");
            return self.err_expr(span);
        }
        let Some(want) = hint.filter(|want| **want != Ty::Unit) else {
            self.error(codes::TYPE_MISMATCH, span, "`recv` can't tell what to read");
            self.help("say what the solution printed: `let x: Int = sol.recv()`");
            return self.err_expr(span);
        };
        let Some(spec) = self.spec_of(want) else {
            let message = format!("`recv` can't tell how much a `{}` is", self.show(want));
            self.error(codes::TYPE_MISMATCH, span, message);
            return self.err_expr(span);
        };
        let want = want.clone();
        self.do_expr(Some(&want), span, |lw, out| {
            let swap = lw.mk(
                ExprKind::Builtin(Builtin::SwapInput, Vec::new()),
                Ty::Unit,
                span,
            );
            out.push(Stmt::Expr(swap.clone()));
            let heard = lw.read(spec, span);
            let slot = lw.temp(heard.ty.clone());
            out.push(Stmt::Let(slot, Some(heard)));
            out.push(Stmt::Expr(swap));
            out.push(Stmt::Give(lw.local(slot, span)));
        })
    }
}
