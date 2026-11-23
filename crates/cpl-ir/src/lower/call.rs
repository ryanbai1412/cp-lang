//! Calls, methods and members: the user's fns and the ones the language
//! provides.

use super::*;

mod collect;
mod field;
mod fold;
mod prelude;
mod seq;
mod user;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Reduce {
    Sum,
    Count,
    All,
    Any,
    Min,
    Max,
}

impl<'a> Lowerer<'a> {
    /// `g.items` in a `for`, when `g` names a grid: one loop over the
    /// cells, with a counter per axis stepped at the top of each round, so
    /// no list of pairs is built. The counters are set up in `out`.
    pub(super) fn grid_items_head(
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
    pub(super) fn grid_items_bind(
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
    fn item_at(&mut self, items: &Items, k: Slot, span: Span) -> (Expr, Expr) {
        let k = self.local(k, span);
        if let Some(lo) = &items.lo {
            let at = ExprKind::Binary(BinOp::Add, Box::new(lo.clone()), Box::new(k));
            let at = self.mk(at, Ty::Int, span);
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
    pub(super) fn items_bind(
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
    fn find_first(
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
    pub(super) fn is_test(&mut self, e: &ast::Expr) -> bool {
        matches!(e.kind, ast::ExprKind::Lambda { .. })
            || matches!(self.probe_ty_silent(e), Some(Ty::Fn(..)))
    }

    /// Lowers `xs.first_ge(val, [by:])`, `xs.first_gt(val, [by:])`, `xs.last_le(val, [by:])`,
    /// `xs.last_lt(val, [by:])`, or `xs.find(val)` on any `Indexed` sequence.
    fn seq_search_method(
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
    pub(super) fn method_call(
        &mut self,
        fn_id: FnId,
        receiver: Option<(&ast::Expr, Expr)>,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        if self.sigs[fn_id].ret.is_none() {
            self.resolve_fn_sig(fn_id, None);
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
            (Some(true), Some((base, _))) => Some(Arg::Mut(self.place(base))),
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
    fn talk(&mut self, name: &ast::Name, args: &[ast::Arg], hint: Option<&Ty>, span: Span) -> Expr {
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

    /// `base.name(args)`.
    fn method(
        &mut self,
        whole: &ast::Expr,
        base: &ast::Expr,
        name: &ast::Name,
        args: &[ast::Arg],
        hint: Option<&Ty>,
    ) -> Expr {
        let span = whole.span;
        // `x.inv()`: a `@get` method reads the same with empty parentheses.
        if args.is_empty() {
            if let Some(Ty::Struct(id)) = self.probe_ty_silent(base) {
                if self.structs[id].getters.contains(&name.text) {
                    return self.field(whole, base, name, hint);
                }
            }
        }
        // `opt.map(f)` / `opt.filter(p)` on `?T`.
        if let ([arg], op @ ("map" | "filter")) = (args, name.text.as_str()) {
            if arg.name.is_none() && matches!(self.probe_ty_silent(base), Some(Ty::Opt(_))) {
                return self.lower_opt_map_or_filter(base, &arg.value, op == "map", span);
            }
        }
        // `a.inverse()`: partial injection `[D]?E -> [E]?D` or bijection `[D]E -> [E]D`.
        if args.is_empty() && name.text == "inverse" {
            if let Some(Ty::Array { dims, .. }) = self.probe_ty_silent(base) {
                if dims.len() == 1 {
                    return self.lower_array_inverse(base, span);
                }
            }
        }
        let is_seq_like = |ty: &Ty| ty.is_seq() || matches!(ty, Ty::Set(_) | Ty::HashSet(_));
        let base_is_seq = self
            .probe_ty_silent(base)
            .as_ref()
            .is_some_and(&is_seq_like)
            || matches!(self.sym(base), Some(Sym::Domain(_)));
        // `xs.map(x => f(x))` returns `[D...]U` on fixed arrays `[D...]T`
        // (preserving index domains), and `List<U>` across every other sequence.
        if let ([func], "map") = (args, name.text.as_str()) {
            if func.name.is_none() && !func.mutable {
                if matches!(self.probe_ty_silent(base), Some(Ty::Array { .. })) {
                    return self.lower_array_map(base, &func.value, span);
                }
                if base_is_seq {
                    return self.lower_map(base, &func.value, span);
                }
            }
        }
        if name.text == "keyed" {
            if let Some(keyed) = self.lower_keyed(base, args, span) {
                return keyed;
            }
        }
        // `xs.filter(x => cond)` returns a `List<T>` of elements satisfying `cond`.
        if let ([test], "filter") = (args, name.text.as_str()) {
            if base_is_seq {
                return self.lower_filter(base, &test.value, span);
            }
        }
        // `xs.partition(x => cond)` returns `(List<T>, List<T>)`.
        if let ([test], "partition") = (args, name.text.as_str()) {
            if base_is_seq {
                return self.lower_partition(base, &test.value, span);
            }
        }
        // `xs.first(x => cond)` / `xs.last(x => cond)` returns `?T`.
        if let ([test], "first") = (args, name.text.as_str()) {
            if test.name.is_none() && base_is_seq {
                return self.lower_first_or_last_where(base, &test.value, true, span);
            }
        }
        if let ([test], "last") = (args, name.text.as_str()) {
            if test.name.is_none() && base_is_seq && self.is_test(&test.value) {
                return self.lower_first_or_last_where(base, &test.value, false, span);
            }
        }
        // `xs.reach(while: (a, b) => ..., cyclic: Bool = false)` returns `List<Int>`.
        if name.text == "reach" && base_is_seq {
            return self.lower_reach(base, args, span);
        }
        // `D.sort(...)`: sort all positions of domain `D` into a `List<D>`.
        if name.text == "sort" {
            if let Some(Sym::Domain(id)) = self.sym(base) {
                let list = self.positions(id, span);
                let list_ty = list.ty.clone();
                let elem = Ty::Pos(id);
                return self.do_expr(Some(&list_ty), span, |lw, out| {
                    let slot = lw.temp(list_ty.clone());
                    out.push(Stmt::Let(slot, Some(list)));
                    let place = Place {
                        kind: PlaceKind::Local(slot),
                        ty: list_ty.clone(),
                        span: lw.span(span),
                    };
                    let sort_expr = lw.sort_place(place, &elem, args, span);
                    out.push(Stmt::Expr(sort_expr));
                    out.push(Stmt::Give(lw.local(slot, span)));
                });
            }
        }
        // `V.all.without(v)`: lift `Range<V>` to `Bitset<V>` and call `without(v)`.
        if name.text == "without" && args.len() == 1 {
            if let Some(Ty::Range(of)) = self.probe_ty_silent(base) {
                if let Ty::Pos(dom) = of.as_ref() {
                    let dom = *dom;
                    if let Some(sid) = self.instantiate_bitset(dom, span) {
                        let base_expr = self.expr(base, Some(&Ty::Struct(sid)));
                        let bs = self.coerce(base_expr, &Ty::Struct(sid), base.span);
                        if let Some(&without_fn) = self.structs[sid].methods.get("without") {
                            self.queue_fn(without_fn);
                            let v = self.expect(&args[0].value, &Ty::Pos(dom));
                            return self.mk(
                                ExprKind::Call(without_fn, vec![Arg::Value(bs), Arg::Value(v)]),
                                Ty::Struct(sid),
                                span,
                            );
                        }
                    }
                }
            }
        }
        // A fold over a sequence reads as its method: `xs.sum()` is
        // `sum(xs)`, `xs.count(x => x > 2)` is `count(xs, x => x > 2)`.
        let seq_ty = self.probe_ty_silent(base);
        if self.is_sorted_range_expr(base)
            || matches!(self.sym(base), Some(Sym::Domain(_)))
            || matches!(
                seq_ty,
                Some(
                    Ty::List(_)
                        | Ty::ListView(_, _)
                        | Ty::Keyed(_, _)
                        | Ty::MutView(_)
                        | Ty::Array { .. }
                        | Ty::Range(_)
                        | Ty::Set(_)
                        | Ty::HashSet(_)
                        | Ty::Str
                        | Ty::StrView
                )
            )
        {
            let seq = ast::Arg {
                name: None,
                mutable: false,
                value: base.clone(),
            };
            match (args, name.text.as_str()) {
                ([], "sum") => return self.reduce(Reduce::Sum, base, span),
                ([op], "fold") if op.name.is_none() && !op.mutable => {
                    return self.fold_seq(base, op, false, name.span, span);
                }
                (_, "sorted")
                    if !matches!(
                        seq_ty,
                        Some(
                            Ty::List(_)
                                | Ty::ListView(_, _)
                                | Ty::Keyed(_, _)
                                | Ty::MutView(_)
                                | Ty::Array { .. }
                        )
                    ) =>
                {
                    let value = match self.walked(base) {
                        Some(values) => self.expr(&values, None),
                        None => self.expr(base, None),
                    };
                    return if args.is_empty() {
                        self.sorted(value, span)
                    } else {
                        self.sorted_by(value, args, span)
                    };
                }
                ([], "min" | "max") if matches!(seq_ty, Some(Ty::ListView(..))) => {
                    return self.min_max_reduce(name.text == "max", base, true, span);
                }
                ([], "min" | "max") if !matches!(seq_ty, Some(Ty::Set(_))) => {
                    return self.min_max(name.text == "max", &[seq], span);
                }
                ([by_arg], "min" | "max")
                    if by_arg.name.as_ref().is_some_and(|n| n.text == "by") && !by_arg.mutable =>
                {
                    return self.min_max_by(name.text == "max", base, &by_arg.value, span);
                }
                ([test], "first") => {
                    if let Some(walk) = self.tested_walk("first", &[seq, test.clone()], true) {
                        return self.lower_first(&walk, span);
                    }
                }
                ([test], "count" | "all" | "any") => {
                    if let Some(walk) = self.tested_walk(&name.text, &[seq, test.clone()], false) {
                        let kind = match name.text.as_str() {
                            "count" => Reduce::Count,
                            "all" => Reduce::All,
                            _ => Reduce::Any,
                        };
                        return self.reduce(kind, &walk, span);
                    }
                }
                _ => {}
            }
        }
        // `sol.send(x)` and `sol.recv()` in an `@interactor`.
        if self.judge && matches!(name.text.as_str(), "send" | "recv") {
            if let ast::ExprKind::Name(n) = &base.kind {
                if self
                    .lookup(n)
                    .is_some_and(|slot| self.cx.locals[slot].ty == Ty::Unit)
                {
                    return self.talk(name, args, hint, span);
                }
            }
        }
        // `[V].filled(x)`, `[n].filled(x)`: an array with every cell `x`.
        if let ast::ExprKind::List(index) = &base.kind {
            if name.text == "filled" && !index.is_empty() {
                return self.array_filled(index, args, hint, span);
            }
        }
        // `List.filled(n, val)`
        if let ast::ExprKind::Name(n) = &base.kind {
            if n == "List" && self.lookup(n).is_none() && name.text == "filled" {
                let Some(exprs) = self.positional_any("filled", args, 2, span) else {
                    return self.err_expr(span);
                };
                let count = self.expect(exprs[0], &Ty::Int);
                let val_hint = match hint {
                    Some(Ty::List(e)) => Some(&**e),
                    _ => None,
                };
                // Under a known element type the value is one of it:
                // `let a: List<Z> = List.filled(n, 0)` holds `Z(0)`.
                let val = match val_hint {
                    Some(elem) if !matches!(elem, Ty::Error) => {
                        let elem = elem.clone();
                        self.expect(exprs[1], &elem)
                    }
                    _ => self.expr(exprs[1], None),
                };
                let list_ty = Ty::list(val.ty.clone());
                return self.mk(
                    ExprKind::Builtin(Builtin::ListFilled, vec![count, val]),
                    list_ty,
                    span,
                );
            }
            if n == "List" && self.lookup(n).is_none() {
                return self.list_static_typo(name, span);
            }
        }
        // `D.nth(k)` where `D` is bound to `V.cuts`.
        if name.text == "nth" {
            if let ast::ExprKind::Name(n) = &base.kind {
                if self.lookup(n).is_none() {
                    if let Some(GenArg::Ty(Ty::Bound(id))) = self.gen_env.get(n).cloned() {
                        if let Some(val) = self.one_arg("nth", args, &Ty::Int, span) {
                            return self.mk(
                                ExprKind::Cast(Cast::Trust, Box::new(val)),
                                Ty::Bound(id),
                                span,
                            );
                        }
                        return self.err_expr(span);
                    }
                    // `D` bound to `Int` by `Range<Int>` arguments: positions are the `Int`s.
                    if let Some(GenArg::Ty(Ty::Int)) = self.gen_env.get(n) {
                        return self
                            .one_arg("nth", args, &Ty::Int, span)
                            .unwrap_or_else(|| self.err_expr(span));
                    }
                }
            }
        }
        // `D.cuts.nth(k)`
        if let ast::ExprKind::Field {
            base: dbase,
            name: fname,
        } = &base.kind
        {
            if matches!(fname.text.as_str(), "bounds" | "cuts")
                && matches!(name.text.as_str(), "nth" | "at")
            {
                let dom_id = if let Some(Sym::Domain(id)) = self.sym(dbase) {
                    Some(id)
                } else if let ast::ExprKind::Name(n) = &dbase.kind {
                    if self.lookup(n).is_none() {
                        match self.gen_env.get(n) {
                            Some(GenArg::Domain(id)) | Some(GenArg::Ty(Ty::Bound(id))) => Some(*id),
                            _ => None,
                        }
                    } else {
                        None
                    }
                } else {
                    let dv = self.expr(dbase, None);
                    match dv.kind {
                        ExprKind::DomainVal(id) => Some(id),
                        _ => match dv.ty {
                            Ty::Pos(id) | Ty::Bound(id) => Some(id),
                            Ty::DomainVal => self.gen_env.values().find_map(|ga| match ga {
                                GenArg::Domain(id) => Some(*id),
                                _ => None,
                            }),
                            _ => None,
                        },
                    }
                };
                if let Some(id) = dom_id {
                    if name.text == "at" {
                        return match self.one_arg("at", args, &Ty::Int, span) {
                            Some(k) => self.domain_cut(id, k, false, span),
                            None => self.err_expr(span),
                        };
                    }
                    if let Some(val) = self.one_arg("nth", args, &Ty::Int, span) {
                        return self.mk(
                            ExprKind::Cast(Cast::Trust, Box::new(val)),
                            Ty::Bound(id),
                            span,
                        );
                    }
                    return self.err_expr(span);
                }
            }
        }
        // `GenericStruct<T>.method(...)` (e.g. `SegTree<V, Int>.from(...)`, `Graph<V, Int>.directed(...)`).
        if let ast::ExprKind::Generic {
            base: gbase,
            args: gtypes,
        } = &base.kind
        {
            if let ast::ExprKind::Name(gname) = &gbase.kind {
                if gname == "Subset" && gtypes.len() == 1 && name.text == "single" {
                    if let Ty::Pos(d) = self.resolve_ty(&gtypes[0]) {
                        if let Some(el) = self.one_arg("single", args, &Ty::Pos(d), span) {
                            let sub = self.subset_domain(d);
                            let shifted = self.mk(
                                ExprKind::Binary(
                                    BinOp::Shl,
                                    Box::new(self.int_lit(1, span)),
                                    Box::new(el),
                                ),
                                Ty::Int,
                                span,
                            );
                            return self.mk(
                                ExprKind::Cast(Cast::Trust, Box::new(shifted)),
                                Ty::Pos(sub),
                                span,
                            );
                        }
                    }
                }
                let lookup = prefix_shape_struct(gname, gtypes.len()).unwrap_or(gname);
                if let Some(Sym::GenStruct(gid)) = self.get_sym(lookup) {
                    if let Some(call) = self.list_form_call(gid, name, gtypes, args, span) {
                        return call;
                    }
                    if let Some(call) = self.list_form_call(gid, name, gtypes, args, span) {
                        return call;
                    }
                    let gargs = self.resolve_gen_args_for_struct(gid, gtypes);
                    let sid = self.instantiate_struct(gid, gargs);
                    if let Some(&fn_id) = self.structs[sid].methods.get(&name.text) {
                        return self.method_call(fn_id, None, args, span);
                    }
                    if let Some(&gfid) = self.structs[sid].gen_methods.get(&name.text) {
                        return self.call_gen_fn(gfid, Vec::new(), None, args, span);
                    }
                    if self.structs[sid].index_domain.is_some() {
                        return self.library_domain_seq(base, sid, name, Some(args), span);
                    }
                    self.error(
                        codes::UNKNOWN_FIELD,
                        name.span,
                        format!("`{}` has no fn `{}`", self.structs[sid].def.name, name.text),
                    );
                    return self.err_expr(span);
                }
            }
        }
        if let Some(e) = self.builtin_type_call(base, name, args, span) {
            return e;
        }
        // `Tr.m(x)`: the `m` of `x`'s `impl Tr`.
        if let (Some(Sym::Trait(tid)), Some((first, rest))) = (self.sym(base), args.split_first()) {
            if first.name.is_none() {
                if let Some(Ty::Struct(sid)) = self.probe_ty_silent(&first.value) {
                    let key = (self.traits[tid].name.clone(), name.text.clone());
                    if let Some(&fn_id) = self.structs[sid].trait_methods.get(&key) {
                        let value = self.expr(&first.value, None);
                        return self.method_call(fn_id, Some((&first.value, value)), rest, span);
                    }
                }
            }
        }
        match self.sym(base) {
            Some(Sym::Mod(target_mod)) => {
                let sym = self.mod_syms[target_mod].get(&name.text).copied();
                return match sym {
                    Some(Sym::Fn(id)) => self.user_call(id, None, args, span),
                    Some(Sym::GenFn(gfid)) => self.call_gen_fn(gfid, Vec::new(), None, args, span),
                    Some(Sym::Struct(id)) => self.construct_or_new(id, args, span),
                    Some(Sym::GenStruct(gid)) => {
                        self.call_gen_struct(gid, Vec::new(), args, hint, span)
                    }
                    Some(Sym::Variant(id, tag)) => self.variant(id, tag, args, span),
                    _ => {
                        self.error(
                            codes::UNKNOWN_FIELD,
                            name.span,
                            format!("module has no fn `{}`", name.text),
                        );
                        self.err_expr(span)
                    }
                };
            }
            Some(Sym::Struct(id)) => {
                if let Some(&fn_id) = self.structs[id].methods.get(&name.text) {
                    let call = self.method_call(fn_id, None, args, span);
                    return match self.structs[id].index_domain {
                        Some(d) if name.text == "nth" => self.checked_nth(d, call, span),
                        _ => call,
                    };
                }
                if let Some(&gfid) = self.structs[id].gen_methods.get(&name.text) {
                    return self.call_gen_fn(gfid, Vec::new(), None, args, span);
                }
                if self.structs[id].index_domain.is_some() {
                    return self.library_domain_seq(base, id, name, Some(args), span);
                }
                self.error(
                    codes::UNKNOWN_FIELD,
                    name.span,
                    format!("`{}` has no fn `{}`", self.structs[id].def.name, name.text),
                );
                return self.err_expr(span);
            }
            Some(Sym::GenStruct(gid)) => {
                // Static method on a generic struct without explicit type args (`Order.by(...)`, `Rooted.weighted(...)`, `Monoid.new(...)`)
                let decl = self.gen_structs[gid].decl;
                let tpl_mod = self.gen_structs[gid].mod_id;
                let gen_names: Vec<String> = decl.generics.iter().map(|g| g.text.clone()).collect();
                if let Some(call) = self.list_form_call(gid, name, &[], args, span) {
                    return call;
                }
                let mut inferred: GenEnv = HashMap::new();
                if let Some(Ty::Struct(hsid)) = hint {
                    if let Some((hgid, hargs)) = &self.structs[*hsid].origin_gen {
                        if *hgid == gid {
                            for (g, a) in gen_names.iter().zip(hargs) {
                                inferred.insert(g.clone(), a.clone());
                            }
                        }
                    }
                }
                if inferred.len() < gen_names.len() {
                    if let Some(mdecl) = decl.methods.iter().find_map(|m| match &m.kind {
                        ast::StmtKind::Fn(f) if f.name.text == name.text => Some(f),
                        _ => None,
                    }) {
                        let mut all_gen_names = gen_names.clone();
                        for g in &mdecl.generics {
                            if !all_gen_names.contains(&g.text) {
                                all_gen_names.push(g.text.clone());
                            }
                        }
                        for (p, arg) in mdecl.params.iter().zip(args) {
                            if !matches!(arg.value.kind, ast::ExprKind::Lambda { .. }) {
                                if let Some(pty) = &p.ty {
                                    self.bind_range_domain(
                                        pty,
                                        &arg.value,
                                        &all_gen_names,
                                        &mut inferred,
                                        span,
                                        None,
                                    );
                                    let dlen = self.diags.len();
                                    let probed = self.expr(&arg.value, None);
                                    self.diags.truncate(dlen);
                                    if probed.ty != Ty::Error {
                                        self.unify_ast_ty(
                                            pty,
                                            &probed.ty,
                                            tpl_mod,
                                            &all_gen_names,
                                            &mut inferred,
                                        );
                                    }
                                }
                            }
                        }
                        self.unify_bounds(&mdecl.bounds, tpl_mod, &all_gen_names, &mut inferred);
                        self.unify_bounds(&decl.bounds, tpl_mod, &all_gen_names, &mut inferred);
                    }
                }
                let final_args: Vec<GenArg> = gen_names
                    .iter()
                    .map(|g| inferred.get(g).cloned().unwrap_or(GenArg::Ty(Ty::Int)))
                    .collect();
                let sid = self.instantiate_struct(gid, final_args);
                if let Some(&fn_id) = self.structs[sid].methods.get(&name.text) {
                    return self.method_call(fn_id, None, args, span);
                }
                if let Some(&gfid) = self.structs[sid].gen_methods.get(&name.text) {
                    return self.call_gen_fn(gfid, Vec::new(), None, args, span);
                }
                self.error(
                    codes::UNKNOWN_FIELD,
                    name.span,
                    format!("`{}` has no fn `{}`", decl.name.text, name.text),
                );
                return self.err_expr(span);
            }
            Some(Sym::Alias(aid)) => {
                let (m, ty_ast, offset) = self.aliases[aid];
                let sm = std::mem::replace(&mut self.cur_mod, m);
                let so = std::mem::replace(&mut self.offset, offset);
                let resolved = self.resolve_ty(ty_ast);
                self.cur_mod = sm;
                self.offset = so;
                if let Ty::Struct(id) = resolved {
                    if let Some(&fn_id) = self.structs[id].methods.get(&name.text) {
                        return self.method_call(fn_id, None, args, span);
                    }
                    if let Some(&gfid) = self.structs[id].gen_methods.get(&name.text) {
                        return self.call_gen_fn(gfid, Vec::new(), None, args, span);
                    }
                }
            }
            Some(Sym::Domain(id))
                if self
                    .sum_enums
                    .get(&id)
                    .is_some_and(|info| info.variants.iter().any(|(v, _)| *v == name.text)) =>
            {
                let info = self.sum_enums[&id].clone();
                let tag = info
                    .variants
                    .iter()
                    .position(|(v, _)| *v == name.text)
                    .unwrap();
                return self.variant(info.enum_id, tag, args, span);
            }
            Some(Sym::Enum(id)) => {
                return match self.enums[id]
                    .variants
                    .iter()
                    .position(|(v, _)| *v == name.text)
                {
                    Some(tag) => self.variant(id, tag, args, span),
                    None if self.unit_enum(id) => self.enum_seq(base, id, name, Some(args), span),
                    None => {
                        let message =
                            format!("`{}` has no variant `{}`", self.enums[id].name, name.text);
                        self.error(codes::UNKNOWN_FIELD, name.span, message);
                        self.err_expr(span)
                    }
                };
            }
            Some(Sym::GenEnum(gid)) => {
                let id = match hint {
                    Some(Ty::Enum(eid)) => Some(*eid),
                    _ => {
                        let decl = self.gen_enums[gid].decl;
                        let tpl_mod = self.gen_enums[gid].mod_id;
                        let gen_names: Vec<String> =
                            decl.generics.iter().map(|g| g.text.clone()).collect();
                        let mut inferred: GenEnv = HashMap::new();
                        if let Some(variant) =
                            decl.variants.iter().find(|v| v.name.text == name.text)
                        {
                            for (f, arg) in variant.fields.iter().zip(args) {
                                let dlen = self.diags.len();
                                let probed = self.expr(&arg.value, None);
                                self.diags.truncate(dlen);
                                if probed.ty != Ty::Error {
                                    self.unify_ast_ty(
                                        &f.1,
                                        &probed.ty,
                                        tpl_mod,
                                        &gen_names,
                                        &mut inferred,
                                    );
                                }
                            }
                        }
                        if inferred.len() == gen_names.len() {
                            let gargs: Vec<GenArg> =
                                gen_names.iter().map(|g| inferred[g].clone()).collect();
                            Some(self.instantiate_enum(gid, gargs))
                        } else {
                            None
                        }
                    }
                };
                if let Some(id) = id {
                    return match self.enums[id]
                        .variants
                        .iter()
                        .position(|(v, _)| *v == name.text)
                    {
                        Some(tag) => self.variant(id, tag, args, span),
                        None => {
                            let message =
                                format!("`{}` has no variant `{}`", self.enums[id].name, name.text);
                            self.error(codes::UNKNOWN_FIELD, name.span, message);
                            self.err_expr(span)
                        }
                    };
                }
            }
            Some(Sym::Domain(id)) => match name.text.as_str() {
                "cuts" => {
                    return match self.one_arg("cuts", args, &Ty::Int, span) {
                        Some(k) => self.domain_cut(id, k, true, span),
                        None => self.err_expr(span),
                    };
                }
                "wrap" => {
                    return match self.one_arg("wrap", args, &Ty::Int, span) {
                        Some(label) => self.domain_wrap(id, label, span),
                        None => self.err_expr(span),
                    };
                }
                "at" => {
                    return match self.one_arg("at", args, &Ty::Int, span) {
                        Some(label) => self.domain_try_label(id, label, span),
                        None => self.err_expr(span),
                    };
                }
                "nth" => {
                    return match self.one_arg(&name.text, args, &Ty::Int, span) {
                        Some(value) => self.mk(
                            ExprKind::Cast(Cast::FromOrd(id), Box::new(value)),
                            Ty::Pos(id),
                            span,
                        ),
                        None => self.err_expr(span),
                    };
                }
                // `V.from_idx(i)` is the inverse of `v.to_idx`, and `none`
                // outside the domain.
                "try_at" | "from_idx" => {
                    return match self.one_arg(&name.text, args, &Ty::Int, span) {
                        Some(value) => self.mk(
                            ExprKind::Builtin(Builtin::PosTryOrd(id), vec![value]),
                            Ty::opt(Ty::Pos(id)),
                            span,
                        ),
                        None => self.err_expr(span),
                    };
                }
                "after" => {
                    return match self.one_arg("after", args, &Ty::Pos(id), span) {
                        Some(pos) => {
                            let lo = self.int_add(self.cast(Cast::Ord, pos, Ty::Int), 1);
                            let hi = self.mk(ExprKind::DomainLen(id), Ty::Int, span);
                            self.mk(
                                ExprKind::Range {
                                    lo: Box::new(lo),
                                    hi: Box::new(hi),
                                },
                                Ty::Range(Box::new(Ty::Pos(id))),
                                span,
                            )
                        }
                        None => self.err_expr(span),
                    };
                }
                "before" => {
                    return match self.one_arg("before", args, &Ty::Pos(id), span) {
                        Some(pos) => {
                            let lo = self.int_lit(0, span);
                            let hi = self.cast(Cast::Ord, pos, Ty::Int);
                            self.mk(
                                ExprKind::Range {
                                    lo: Box::new(lo),
                                    hi: Box::new(hi),
                                },
                                Ty::Range(Box::new(Ty::Pos(id))),
                                span,
                            )
                        }
                        None => self.err_expr(span),
                    };
                }
                "pairs" if args.is_empty() => {
                    return self.domain_pairs_list(id, span);
                }
                _ => {}
            },
            _ => {}
        }
        // Check `lends` projection read before lowering `base` as a plain value.
        if let Some(proj) = self.find_projection_on(base, &name.text) {
            return self.lower_projection_read(proj, base, args, span);
        }
        let field = ast::ExprKind::Field {
            base: Box::new(base.clone()),
            name: name.clone(),
        };
        let op = self.detect_bare_op(&ast::Expr { kind: field, span });
        if let Some(op) = op.or_else(|| self.field_op(base, &name.text)) {
            if let Some(e) = self.bare_op_call(op, name.span, args, span, hint) {
                return e;
            }
        }
        let value = self.expr(base, None);
        let ty = value.ty.clone();
        let method = name.text.as_str();
        let value = match self.call_field(value, name, args, span, hint) {
            Ok(call) => return call,
            Err(value) => value,
        };
        if let Ty::Struct(id) = &ty {
            if method == "get" {
                if let Some(key_ty) = self.counter_key.get(id).cloned() {
                    let Some(key) = self.one_arg(method, args, &key_ty, span) else {
                        return self.err_expr(span);
                    };
                    let map_ty = self.structs[*id].def.fields[0].1.clone();
                    let map_expr = self.mk(ExprKind::Field(Box::new(value), 0), map_ty, span);
                    let found = self.mk(
                        ExprKind::Builtin(Builtin::MapGet, vec![map_expr, key]),
                        Ty::opt(Ty::Int),
                        span,
                    );
                    let zero = self.int_lit(0, span);
                    return self.mk(
                        ExprKind::Binary(BinOp::Coalesce, Box::new(found), Box::new(zero)),
                        Ty::Int,
                        span,
                    );
                }
            }
            if let Some(fn_id) = self.structs[*id].methods.get(method).copied() {
                return self.method_call(fn_id, Some((base, value)), args, span);
            }
            if let Some(gfid) = self.structs[*id].gen_methods.get(method).copied() {
                let is_mut = self.gen_fns[gfid]
                    .decl
                    .params
                    .first()
                    .is_some_and(|p| p.name.text == "self" && p.mutable)
                    || self.structs[*id]
                        .decl
                        .is_some_and(|sd| super::cache::grows_cache(sd, self.gen_fns[gfid].decl));
                let recv = if is_mut {
                    Arg::Mut(self.place(base))
                } else {
                    Arg::Value(value)
                };
                self.call_hint = hint.cloned();
                return self.call_gen_fn(gfid, Vec::new(), Some(recv), args, span);
            }
        }
        // Members that read like fields can be called too: `xs.len()`.
        if args.is_empty()
            && matches!(
                method,
                "len" | "is_empty" | "first" | "last" | "items" | "keys" | "values"
            )
            && !matches!(ty, Ty::Struct(_))
        {
            return self.field(whole, base, name, hint);
        }
        let elem = match &ty {
            Ty::List(elem)
            | Ty::ListView(elem, _)
            | Ty::Keyed(elem, _)
            | Ty::MutView(elem)
            | Ty::Set(elem)
            | Ty::HashSet(elem)
            | Ty::Range(elem) => (**elem).clone(),
            Ty::Array { elem, .. } => (**elem).clone(),
            Ty::Map(key, _) | Ty::HashMap(key, _) => (**key).clone(),
            Ty::Str | Ty::StrView => Ty::Char,
            _ => Ty::Error,
        };
        let mutate = |lw: &mut Self, op: Mutation, args: Vec<Expr>, result: Ty| {
            let place = lw.place(base);
            lw.mk(ExprKind::Mutate(op, Box::new(place), args), result, span)
        };
        match (&ty, method) {
            (Ty::Pos(d), "with") if self.subset_base(*d).is_some() => {
                let base_d = self.subset_base(*d).unwrap();
                let Some(v) = self.one_arg(method, args, &Ty::Pos(base_d), span) else {
                    return self.err_expr(span);
                };
                let s_ord = self.cast(Cast::Ord, value, Ty::Int);
                let v_ord = self.cast(Cast::Ord, v, Ty::Int);
                let one = self.int_lit(1, span);
                let bit = self.mk(
                    ExprKind::Binary(BinOp::Shl, Box::new(one), Box::new(v_ord)),
                    Ty::Int,
                    span,
                );
                let res = self.mk(
                    ExprKind::Binary(BinOp::BitOr, Box::new(s_ord), Box::new(bit)),
                    Ty::Int,
                    span,
                );
                self.cast(Cast::Trust, res, Ty::Pos(*d))
            }
            // `s.set(v, on)` is `s.with(v)` when `on` holds and `s.without(v)`
            // when it does not.
            (Ty::Pos(d), "set") if self.subset_base(*d).is_some() => {
                let base_d = self.subset_base(*d).unwrap();
                let Some(exprs) = self.positional_any(method, args, 2, span) else {
                    return self.err_expr(span);
                };
                let v = self.expect(exprs[0], &Ty::Pos(base_d));
                let on = self.expect(exprs[1], &Ty::Bool);
                let s_ord = self.cast(Cast::Ord, value, Ty::Int);
                let v_ord = self.cast(Cast::Ord, v, Ty::Int);
                let one = self.int_lit(1, span);
                let bit = self.mk(
                    ExprKind::Binary(BinOp::Shl, Box::new(one), Box::new(v_ord)),
                    Ty::Int,
                    span,
                );
                let with = self.mk(
                    ExprKind::Binary(BinOp::BitOr, Box::new(s_ord.clone()), Box::new(bit.clone())),
                    Ty::Int,
                    span,
                );
                let mask = self.mk(ExprKind::Unary(UnOp::BitNot, Box::new(bit)), Ty::Int, span);
                let without = self.mk(
                    ExprKind::Binary(BinOp::BitAnd, Box::new(s_ord), Box::new(mask)),
                    Ty::Int,
                    span,
                );
                // One side runs, so `s` and `v` are each worked out once.
                let res = self.mk(
                    ExprKind::If(Box::new(on), Box::new(with), Box::new(without)),
                    Ty::Int,
                    span,
                );
                self.cast(Cast::Trust, res, Ty::Pos(*d))
            }
            (Ty::Pos(d), "without") if self.subset_base(*d).is_some() => {
                let base_d = self.subset_base(*d).unwrap();
                let Some(v) = self.one_arg(method, args, &Ty::Pos(base_d), span) else {
                    return self.err_expr(span);
                };
                let s_ord = self.cast(Cast::Ord, value, Ty::Int);
                let v_ord = self.cast(Cast::Ord, v, Ty::Int);
                let one = self.int_lit(1, span);
                let bit = self.mk(
                    ExprKind::Binary(BinOp::Shl, Box::new(one), Box::new(v_ord)),
                    Ty::Int,
                    span,
                );
                let mask = self.mk(ExprKind::Unary(UnOp::BitNot, Box::new(bit)), Ty::Int, span);
                let res = self.mk(
                    ExprKind::Binary(BinOp::BitAnd, Box::new(s_ord), Box::new(mask)),
                    Ty::Int,
                    span,
                );
                self.cast(Cast::Trust, res, Ty::Pos(*d))
            }
            (Ty::Error, _) => {
                let reported = self.diags.len();
                for arg in args {
                    self.expr(&arg.value, None);
                }
                self.diags.truncate(reported);
                self.err_expr(span)
            }
            (Ty::Pos(d), "l") => {
                let d = *d;
                match self.one_arg("l", args, &Ty::Int, span) {
                    Some(k) => self.mk(
                        ExprKind::Builtin(Builtin::BoundStepL(d), vec![value, k]),
                        Ty::Bound(d),
                        span,
                    ),
                    None => self.err_expr(span),
                }
            }
            (Ty::Pos(d), "r") => {
                let d = *d;
                match self.one_arg("r", args, &Ty::Int, span) {
                    Some(k) => self.mk(
                        ExprKind::Builtin(Builtin::BoundStepR(d), vec![value, k]),
                        Ty::Bound(d),
                        span,
                    ),
                    None => self.err_expr(span),
                }
            }
            (Ty::Pos(d), "offset" | "ahead") => {
                let d = *d;
                match self.one_arg(method, args, &Ty::Int, span) {
                    Some(k) => self.mk(
                        ExprKind::Builtin(Builtin::PosOffset(d), vec![value, k]),
                        Ty::opt(Ty::Pos(d)),
                        span,
                    ),
                    None => self.err_expr(span),
                }
            }
            (Ty::Pos(d), "back") => {
                let d = *d;
                match self.one_arg("back", args, &Ty::Int, span) {
                    Some(k) => {
                        let neg_k = self.mk(ExprKind::Unary(UnOp::Neg, Box::new(k)), Ty::Int, span);
                        self.mk(
                            ExprKind::Builtin(Builtin::PosOffset(d), vec![value, neg_k]),
                            Ty::opt(Ty::Pos(d)),
                            span,
                        )
                    }
                    None => self.err_expr(span),
                }
            }
            (Ty::Pos(d), "cyc") => {
                let d = *d;
                match self.one_arg("cyc", args, &Ty::Int, span) {
                    Some(k) => self.mk(
                        ExprKind::Builtin(Builtin::PosCyc(d), vec![value, k]),
                        Ty::Pos(d),
                        span,
                    ),
                    None => self.err_expr(span),
                }
            }
            (Ty::Str, "join") => {
                let Some(exprs) = self.positional_any("join", args, 1, span) else {
                    return self.err_expr(span);
                };
                let items = self.expr(exprs[0], None);
                self.join(value, items, None, exprs[0].span)
            }
            (ty, "get") if ty.is_indexed() => {
                let elem = ty.seq_elem().unwrap();
                let opt = Ty::opt(elem);
                match self.one_arg("get", args, &Ty::Int, span) {
                    Some(at) => self.mk(
                        ExprKind::Builtin(Builtin::SeqGet, vec![value, at]),
                        opt,
                        span,
                    ),
                    None => self.err_expr(span),
                }
            }
            (ty, "at") if Self::keyed_view(ty) => {
                let elem = ty.seq_elem().unwrap();
                match self.one_arg("at", args, &Ty::Int, span) {
                    Some(at) => {
                        let view = self.hold_view(value, span);
                        self.key_index(view, at, elem, span)
                    }
                    None => self.err_expr(span),
                }
            }
            (ty, "nth")
                if ty.is_indexed()
                    && matches!(
                        ty,
                        Ty::List(_) | Ty::ListView(_, _) | Ty::Keyed(_, _) | Ty::Str | Ty::StrView
                    ) =>
            {
                let elem = ty.seq_elem().unwrap();
                match self.one_arg("nth", args, &Ty::Int, span) {
                    Some(at) => self.mk(ExprKind::Index(Box::new(value), vec![at]), elem, span),
                    None => self.err_expr(span),
                }
            }
            (ty, "at") if ty.is_indexed() => {
                let elem = ty.seq_elem().unwrap();
                match self.one_arg("at", args, &ty.key_ty(), span) {
                    Some(at) => {
                        let indices = self.vouched(ty, vec![at]);
                        self.mk(ExprKind::Index(Box::new(value), indices), elem, span)
                    }
                    None => self.err_expr(span),
                }
            }
            (ty, "set_at") if ty.is_indexed_mut() => {
                let elem = ty.seq_elem().unwrap();
                let ty_owned = ty.clone();
                match self.positional_any(method, args, 2, span) {
                    Some(exprs) => {
                        let at = self.expect(exprs[0], &Ty::Int);
                        let val = self.expect(exprs[1], &elem);
                        let val_span = exprs[1].span;
                        self.do_expr(Some(&Ty::Unit), span, |lw, out| {
                            let place = lw.place(base);
                            let val = match lw
                                .place_range(&place)
                                .and_then(|r| r.elem().and_then(|e| e.scalar()).cloned())
                            {
                                Some(b) => lw.check_range(val, &b, val_span),
                                None => val,
                            };
                            let indices = lw.vouched(&ty_owned, vec![at]);
                            let cell = Place {
                                kind: PlaceKind::Index(Box::new(place), indices),
                                ty: elem,
                                span: lw.span(span),
                            };
                            out.push(Stmt::Assign(cell, AssignOp::Set, val));
                            out.push(Stmt::Give(lw.mk(ExprKind::Unit, Ty::Unit, span)));
                        })
                    }
                    None => self.err_expr(span),
                }
            }
            (ty, "swap_idx") if ty.is_indexed_mut() => {
                match self.positional_any(method, args, 2, span) {
                    Some(exprs) => {
                        let a = self.expect(exprs[0], &Ty::Int);
                        let b = self.expect(exprs[1], &Ty::Int);
                        mutate(self, Mutation::Swap, vec![a, b], Ty::Unit)
                    }
                    None => self.err_expr(span),
                }
            }
            (Ty::List(_) | Ty::Str, "push") => match self.one_arg(method, args, &elem, span) {
                Some(arg) => {
                    let place = self.place(base);
                    let arg = match self
                        .place_range(&place)
                        .and_then(|r| r.elem().and_then(|e| e.scalar()).cloned())
                    {
                        Some(b) => self.check_range(arg, &b, span),
                        None => arg,
                    };
                    self.mk(
                        ExprKind::Mutate(Mutation::Push, Box::new(place), vec![arg]),
                        Ty::Unit,
                        span,
                    )
                }
                None => self.err_expr(span),
            },
            (Ty::List(_), "push_id") => match self.one_arg(method, args, &elem, span) {
                // The index the new element lands on.
                Some(arg) => self.do_expr(Some(&Ty::Int), span, |lw, out| {
                    let place = lw.place(base);
                    let place = lw.settle_indexes(place, out);
                    let arg = match lw
                        .place_range(&place)
                        .and_then(|r| r.elem().and_then(|e| e.scalar()).cloned())
                    {
                        Some(b) => lw.check_range(arg, &b, span),
                        None => arg,
                    };
                    let held = lw.temp(arg.ty.clone());
                    out.push(Stmt::Let(held, Some(arg)));
                    let id = lw.temp(Ty::Int);
                    let list = super::stmt::place_to_expr(&place);
                    let len = lw.mk(ExprKind::Builtin(Builtin::Len, vec![list]), Ty::Int, span);
                    out.push(Stmt::Let(id, Some(len)));
                    let arg = lw.local(held, span);
                    let push = lw.mk(
                        ExprKind::Mutate(Mutation::Push, Box::new(place), vec![arg]),
                        Ty::Unit,
                        span,
                    );
                    out.push(Stmt::Expr(push));
                    let id = lw.local(id, span);
                    out.push(Stmt::Give(id));
                }),
                None => self.err_expr(span),
            },
            (
                Ty::List(_) | Ty::Str | Ty::Set(_) | Ty::HashSet(_) | Ty::Map(..) | Ty::HashMap(..),
                "take",
            ) if args.is_empty() => match self.positional_any(method, args, 0, span) {
                Some(_) => mutate(self, Mutation::Take, Vec::new(), ty.clone()),
                None => self.err_expr(span),
            },
            (Ty::Set(_) | Ty::Map(..), "pop_first" | "pop_last") => {
                let found = match &ty {
                    Ty::Map(key, val) => Ty::Tuple(vec![(**key).clone(), (**val).clone()]),
                    _ => elem.clone(),
                };
                let op = if method == "pop_first" {
                    Mutation::PopFirst
                } else {
                    Mutation::PopLast
                };
                match self.positional_any(method, args, 0, span) {
                    Some(_) => mutate(self, op, Vec::new(), Ty::opt(found)),
                    None => self.err_expr(span),
                }
            }
            (Ty::List(_) | Ty::Str, "pop") => match self.positional_any(method, args, 0, span) {
                Some(_) => {
                    if matches!(elem, Ty::Opt(_)) {
                        mutate(self, Mutation::PopVal, Vec::new(), elem)
                    } else {
                        mutate(self, Mutation::Pop, Vec::new(), Ty::opt(elem))
                    }
                }
                None => self.err_expr(span),
            },
            (Ty::List(_) | Ty::MutView(_) | Ty::Array { .. } | Ty::Str, "swap") => {
                match self.positional_any(method, args, 2, span) {
                    Some(exprs) => {
                        let idx_ty = match &ty {
                            Ty::Array { dims, .. } if dims.len() == 1 => match dims[0] {
                                Dim::Pos(d) => Ty::Pos(d),
                                Dim::Bound(d) => Ty::Bound(d),
                                Dim::Enum(e) => Ty::Enum(e),
                                Dim::Sym(_) | Dim::Int | Dim::IntFrom(_) => Ty::Int,
                            },
                            _ => Ty::Int,
                        };
                        let a = self.expect(exprs[0], &idx_ty);
                        let b = self.expect(exprs[1], &idx_ty);
                        let a = if matches!(a.ty, Ty::Pos(_) | Ty::Bound(_)) {
                            self.cast(Cast::Ord, a, Ty::Int)
                        } else {
                            a
                        };
                        let b = if matches!(b.ty, Ty::Pos(_) | Ty::Bound(_)) {
                            self.cast(Cast::Ord, b, Ty::Int)
                        } else {
                            b
                        };
                        mutate(self, Mutation::Swap, vec![a, b], Ty::Unit)
                    }
                    None => self.err_expr(span),
                }
            }
            (Ty::List(_) | Ty::Str, "resize") => match self.positional_any(method, args, 2, span) {
                Some(exprs) => {
                    let n = self.expect(exprs[0], &Ty::Int);
                    let v = self.expect(exprs[1], &elem);
                    let place = self.place(base);
                    let v = match self
                        .place_range(&place)
                        .and_then(|r| r.elem().and_then(|e| e.scalar()).cloned())
                    {
                        Some(b) => self.check_range(v, &b, exprs[1].span),
                        None => v,
                    };
                    self.mk(
                        ExprKind::Mutate(Mutation::Resize, Box::new(place), vec![n, v]),
                        Ty::Unit,
                        span,
                    )
                }
                None => self.err_expr(span),
            },
            (Ty::List(_) | Ty::Str, "truncate") => match self.one_arg(method, args, &Ty::Int, span)
            {
                Some(n) => mutate(self, Mutation::Truncate, vec![n], Ty::Unit),
                None => self.err_expr(span),
            },
            (Ty::List(_), "reserve") => match self.one_arg(method, args, &Ty::Int, span) {
                Some(n) => mutate(self, Mutation::Reserve, vec![n], Ty::Unit),
                None => self.err_expr(span),
            },
            (Ty::List(_), "extend") => match self.positional_any(method, args, 1, span) {
                Some(exprs) => {
                    let other = self.expect(exprs[0], &ty);
                    mutate(self, Mutation::Extend, vec![other], Ty::Unit)
                }
                None => self.err_expr(span),
            },
            (Ty::List(_) | Ty::MutView(_) | Ty::Array { .. } | Ty::Str, "fill") => {
                match self.one_arg(method, args, &elem, span) {
                    Some(v) => {
                        let place = self.place(base);
                        let v = match self
                            .place_range(&place)
                            .and_then(|r| r.elem().and_then(|e| e.scalar()).cloned())
                        {
                            Some(b) => self.check_range(v, &b, span),
                            None => v,
                        };
                        self.mk(
                            ExprKind::Mutate(Mutation::Fill, Box::new(place), vec![v]),
                            Ty::Unit,
                            span,
                        )
                    }
                    None => self.err_expr(span),
                }
            }
            (Ty::Str | Ty::StrView, "to_str") => match self.positional_any(method, args, 0, span) {
                Some(_) => self.mk(
                    ExprKind::Builtin(Builtin::ToStr, vec![value]),
                    Ty::Str,
                    span,
                ),
                None => self.err_expr(span),
            },
            (Ty::List(e) | Ty::ListView(e, _) | Ty::Keyed(e, _) | Ty::MutView(e), "to_list") => {
                let out_ty = Ty::list((**e).clone());
                match self.positional_any(method, args, 0, span) {
                    Some(_) => self.mk(
                        ExprKind::Builtin(Builtin::ToList, vec![value]),
                        out_ty,
                        span,
                    ),
                    None => self.err_expr(span),
                }
            }
            (Ty::StrView, "copy" | "clone") => match self.positional_any(method, args, 0, span) {
                Some(_) => self.mk(
                    ExprKind::Builtin(Builtin::ToStr, vec![value]),
                    Ty::Str,
                    span,
                ),
                None => self.err_expr(span),
            },
            (Ty::ListView(e, _) | Ty::Keyed(e, _) | Ty::MutView(e), "copy" | "clone") => {
                let out_ty = Ty::list((**e).clone());
                match self.positional_any(method, args, 0, span) {
                    Some(_) => self.mk(
                        ExprKind::Builtin(Builtin::ToList, vec![value]),
                        out_ty,
                        span,
                    ),
                    None => self.err_expr(span),
                }
            }
            (
                Ty::List(_)
                | Ty::Array { .. }
                | Ty::Map(..)
                | Ty::HashMap(..)
                | Ty::Set(_)
                | Ty::HashSet(_)
                | Ty::Str,
                "copy" | "clone",
            ) => match self.positional_any(method, args, 0, span) {
                Some(_) => self.mk(
                    ExprKind::Builtin(Builtin::Copy, vec![value]),
                    ty.clone(),
                    span,
                ),
                None => self.err_expr(span),
            },
            (Ty::Set(_) | Ty::HashSet(_), "insert" | "add") => {
                match self.one_arg(method, args, &elem, span) {
                    Some(arg) => mutate(self, Mutation::Insert, vec![arg], Ty::Bool),
                    None => self.err_expr(span),
                }
            }
            // `m.remove(k)` gives the value that was there, or `none`.
            (Ty::Map(_, val) | Ty::HashMap(_, val), "remove") => {
                let found_ty = Ty::Opt(val.clone());
                match self.one_arg(method, args, &elem, span) {
                    Some(arg) => self.do_expr(Some(&found_ty), span, |lw, out| {
                        let place = lw.place(base);
                        let place = lw.settle_indexes(place, out);
                        let key = lw.temp(arg.ty.clone());
                        out.push(Stmt::Let(key, Some(arg)));
                        let map = super::stmt::place_to_expr(&place);
                        let key_expr = lw.local(key, span);
                        let found = lw.mk(
                            ExprKind::Builtin(Builtin::MapGet, vec![map, key_expr.clone()]),
                            found_ty.clone(),
                            span,
                        );
                        let held = lw.temp(found_ty.clone());
                        out.push(Stmt::Let(held, Some(found)));
                        let remove = lw.mk(
                            ExprKind::Mutate(Mutation::Remove, Box::new(place), vec![key_expr]),
                            Ty::Bool,
                            span,
                        );
                        // Whether it was there is already known.
                        out.push(Stmt::If(remove, Block::default(), Block::default()));
                        let held = lw.local(held, span);
                        out.push(Stmt::Give(held));
                    }),
                    None => self.err_expr(span),
                }
            }
            (Ty::Set(_) | Ty::HashSet(_), "remove") => {
                match self.one_arg(method, args, &elem, span) {
                    Some(arg) => mutate(self, Mutation::Remove, vec![arg], Ty::Bool),
                    None => self.err_expr(span),
                }
            }
            (Ty::Set(_) | Ty::HashSet(_), "set") => {
                let mut key_ast = None;
                let mut on_ast = None;
                let mut ok = args.len() == 2;
                for a in args {
                    match a.name.as_ref().map(|n| n.text.as_str()) {
                        None if key_ast.is_none() => key_ast = Some(&a.value),
                        Some("on") | None if on_ast.is_none() => on_ast = Some(&a.value),
                        _ => ok = false,
                    }
                }
                match (ok, key_ast, on_ast) {
                    (true, Some(k_ast), Some(b_ast)) => {
                        let k_val = self.expect(k_ast, &elem);
                        let b_val = self.expect(b_ast, &Ty::Bool);
                        self.do_expr(Some(&Ty::Unit), span, |lw, out| {
                            let place = lw.place(base);
                            let place = lw.settle_indexes(place, out);
                            let k_slot = lw.temp(k_val.ty.clone());
                            out.push(Stmt::Let(k_slot, Some(k_val)));
                            let k_ref = lw.local(k_slot, span);
                            let ins = lw.mk(
                                ExprKind::Mutate(
                                    Mutation::Insert,
                                    Box::new(place.clone()),
                                    vec![k_ref.clone()],
                                ),
                                Ty::Bool,
                                span,
                            );
                            let rem = lw.mk(
                                ExprKind::Mutate(Mutation::Remove, Box::new(place), vec![k_ref]),
                                Ty::Bool,
                                span,
                            );
                            out.push(Stmt::If(
                                b_val,
                                Block {
                                    stmts: vec![Stmt::Expr(ins)],
                                },
                                Block {
                                    stmts: vec![Stmt::Expr(rem)],
                                },
                            ));
                            out.push(Stmt::Give(lw.mk(ExprKind::Unit, Ty::Unit, span)));
                        })
                    }
                    _ => self.err_expr(span),
                }
            }
            (Ty::Set(_), "drain") => {
                let mut after_ast = None;
                let mut before_ast = None;
                let mut while_ast = None;
                let mut ok = true;
                for a in args {
                    match a.name.as_ref().map(|n| n.text.as_str()) {
                        Some("after") if after_ast.is_none() => after_ast = Some(&a.value),
                        Some("before") if before_ast.is_none() => before_ast = Some(&a.value),
                        Some("while") if while_ast.is_none() => while_ast = Some(&a.value),
                        Some(other) => {
                            self.error(
                                codes::UNKNOWN_ARG_NAME,
                                a.name.as_ref().unwrap().span,
                                format!("`drain` has no parameter named `{other}`"),
                            );
                            ok = false;
                        }
                        None => ok = false,
                    }
                }
                let Some(while_ast) = while_ast.filter(|_| ok) else {
                    return self.err_expr(span);
                };
                let opt_elem = Ty::opt(elem.clone());
                let after_val = after_ast.map(|a| self.expect(a, &opt_elem));
                let before_val = before_ast.map(|b| self.expect(b, &opt_elem));
                let pred_ty = Ty::Fn(vec![elem.clone()], Box::new(Ty::Bool));
                let pred_val = self.expect(while_ast, &pred_ty);
                let out_ty = Ty::list(elem.clone());
                self.do_expr(Some(&out_ty), span, |lw, out| {
                    let place = lw.place(base);
                    let place = lw.settle_indexes(place, out);
                    let pred_slot = lw.temp(pred_ty.clone());
                    out.push(Stmt::Let(pred_slot, Some(pred_val)));
                    let out_slot = lw.temp(out_ty.clone());
                    out.push(Stmt::Let(
                        out_slot,
                        Some(lw.mk(ExprKind::Tuple(Vec::new()), out_ty.clone(), span)),
                    ));
                    let after_slot = after_val.map(|v| {
                        let s = lw.temp(opt_elem.clone());
                        out.push(Stmt::Let(s, Some(v)));
                        s
                    });
                    let before_slot = before_val.map(|v| {
                        let s = lw.temp(opt_elem.clone());
                        out.push(Stmt::Let(s, Some(v)));
                        s
                    });
                    if let (Some(a_s), Some(b_s)) = (after_slot, before_slot) {
                        let both = lw.mk(
                            ExprKind::Binary(
                                BinOp::And,
                                Box::new(lw.mk(
                                    ExprKind::IsSome(Box::new(lw.local(a_s, span))),
                                    Ty::Bool,
                                    span,
                                )),
                                Box::new(lw.mk(
                                    ExprKind::IsSome(Box::new(lw.local(b_s, span))),
                                    Ty::Bool,
                                    span,
                                )),
                            ),
                            Ty::Bool,
                            span,
                        );
                        let cond =
                            lw.mk(ExprKind::Unary(UnOp::Not, Box::new(both)), Ty::Bool, span);
                        out.push(Stmt::Assert {
                            cond,
                            message: Some(lw.mk(
                                ExprKind::Str(
                                    "drain takes either after: or before:, not both".into(),
                                ),
                                Ty::Str,
                                span,
                            )),
                            text: "not (after != None and before != None)".into(),
                            line: lw.line(span),
                            span,
                            kept: false,
                        });
                    }
                    let build_loop = |lw: &mut Lowerer<'_>, next_opt: Expr| -> Stmt {
                        let lid = lw.next_loop;
                        lw.next_loop += 1;
                        let opt_slot = lw.temp(opt_elem.clone());
                        let k_slot = lw.temp(elem.clone());
                        let is_some = lw.mk(
                            ExprKind::IsSome(Box::new(lw.local(opt_slot, span))),
                            Ty::Bool,
                            span,
                        );
                        let not_some = lw.mk(
                            ExprKind::Unary(UnOp::Not, Box::new(is_some)),
                            Ty::Bool,
                            span,
                        );
                        let unwrapped = lw.mk(
                            ExprKind::Unwrap(Box::new(lw.local(opt_slot, span)), false),
                            elem.clone(),
                            span,
                        );
                        let ok_call = lw.mk(
                            ExprKind::CallExpr(
                                Box::new(lw.local(pred_slot, span)),
                                vec![Arg::Value(lw.local(k_slot, span))],
                            ),
                            Ty::Bool,
                            span,
                        );
                        let not_ok = lw.mk(
                            ExprKind::Unary(UnOp::Not, Box::new(ok_call)),
                            Ty::Bool,
                            span,
                        );
                        let rem = lw.mk(
                            ExprKind::Mutate(
                                Mutation::Remove,
                                Box::new(place.clone()),
                                vec![lw.local(k_slot, span)],
                            ),
                            Ty::Bool,
                            span,
                        );
                        let out_place = Place {
                            kind: PlaceKind::Local(out_slot),
                            ty: out_ty.clone(),
                            span,
                        };
                        let push = lw.mk(
                            ExprKind::Mutate(
                                Mutation::Push,
                                Box::new(out_place),
                                vec![lw.local(k_slot, span)],
                            ),
                            Ty::Unit,
                            span,
                        );
                        let body = vec![
                            Stmt::Let(opt_slot, Some(next_opt)),
                            Stmt::If(
                                not_some,
                                Block {
                                    stmts: vec![Stmt::Break(Some(lid))],
                                },
                                Block { stmts: Vec::new() },
                            ),
                            Stmt::Let(k_slot, Some(unwrapped)),
                            Stmt::If(
                                not_ok,
                                Block {
                                    stmts: vec![Stmt::Break(Some(lid))],
                                },
                                Block { stmts: Vec::new() },
                            ),
                            Stmt::Expr(rem),
                            Stmt::Expr(push),
                        ];
                        Stmt::While(
                            lid,
                            lw.mk(ExprKind::Bool(true), Ty::Bool, span),
                            Block { stmts: body },
                        )
                    };
                    let set_read = super::stmt::place_to_expr(&place);
                    let first_loop = build_loop(
                        lw,
                        lw.mk(
                            ExprKind::Builtin(Builtin::SetMin, vec![set_read.clone()]),
                            opt_elem.clone(),
                            span,
                        ),
                    );
                    let after_branch = match after_slot {
                        Some(a_s) => {
                            let a_val = lw.mk(
                                ExprKind::Unwrap(Box::new(lw.local(a_s, span)), false),
                                elem.clone(),
                                span,
                            );
                            let after_loop = build_loop(
                                lw,
                                lw.mk(
                                    ExprKind::Builtin(
                                        Builtin::FirstGt,
                                        vec![set_read.clone(), a_val],
                                    ),
                                    opt_elem.clone(),
                                    span,
                                ),
                            );
                            Stmt::If(
                                lw.mk(
                                    ExprKind::IsSome(Box::new(lw.local(a_s, span))),
                                    Ty::Bool,
                                    span,
                                ),
                                Block {
                                    stmts: vec![after_loop],
                                },
                                Block {
                                    stmts: vec![first_loop],
                                },
                            )
                        }
                        None => first_loop,
                    };
                    let top_stmt = match before_slot {
                        Some(b_s) => {
                            let b_val = lw.mk(
                                ExprKind::Unwrap(Box::new(lw.local(b_s, span)), false),
                                elem.clone(),
                                span,
                            );
                            let before_loop = build_loop(
                                lw,
                                lw.mk(
                                    ExprKind::Builtin(Builtin::LastLt, vec![set_read, b_val]),
                                    opt_elem.clone(),
                                    span,
                                ),
                            );
                            Stmt::If(
                                lw.mk(
                                    ExprKind::IsSome(Box::new(lw.local(b_s, span))),
                                    Ty::Bool,
                                    span,
                                ),
                                Block {
                                    stmts: vec![before_loop],
                                },
                                Block {
                                    stmts: vec![after_branch],
                                },
                            )
                        }
                        None => after_branch,
                    };
                    out.push(top_stmt);
                    out.push(Stmt::Give(lw.local(out_slot, span)));
                })
            }
            (
                Ty::List(_) | Ty::Set(_) | Ty::HashSet(_) | Ty::Map(..) | Ty::HashMap(..) | Ty::Str,
                "clear",
            ) => match self.positional_any(method, args, 0, span) {
                Some(_) => mutate(self, Mutation::Clear, Vec::new(), Ty::Unit),
                None => self.err_expr(span),
            },
            (Ty::List(_) | Ty::MutView(_) | Ty::Array { .. } | Ty::Str, "reverse") => {
                match self.positional_any(method, args, 0, span) {
                    Some(_) => mutate(self, Mutation::Reverse, Vec::new(), Ty::Unit),
                    None => self.err_expr(span),
                }
            }
            (Ty::List(_) | Ty::MutView(_) | Ty::Array { .. } | Ty::Str, "sort") => {
                self.sort(base, &elem, args, span)
            }
            (ty, "sorted") if ty.is_seq() || matches!(ty, Ty::Set(_) | Ty::HashSet(_)) => {
                if args.is_empty() {
                    self.sorted(value, span)
                } else {
                    self.sorted_by(value, args, span)
                }
            }
            (Ty::Str | Ty::StrView, "reversed") => match self.positional_any(method, args, 0, span)
            {
                Some(_) => {
                    let rev = self.mk(
                        ExprKind::Builtin(Builtin::Reversed, vec![value]),
                        Ty::StrView,
                        span,
                    );
                    self.mk(ExprKind::Builtin(Builtin::ToStr, vec![rev]), Ty::Str, span)
                }
                None => self.err_expr(span),
            },
            (Ty::List(e) | Ty::ListView(e, _) | Ty::Keyed(e, _) | Ty::MutView(e), "reversed") => {
                let out_ty = Ty::list((**e).clone());
                let view_ty = Ty::ListView(e.clone(), Dim::Int);
                match self.positional_any(method, args, 0, span) {
                    Some(_) => {
                        let rev = self.mk(
                            ExprKind::Builtin(Builtin::Reversed, vec![value]),
                            view_ty,
                            span,
                        );
                        self.mk(ExprKind::Builtin(Builtin::ToList, vec![rev]), out_ty, span)
                    }
                    None => self.err_expr(span),
                }
            }
            (Ty::Array { dims, elem: e }, "reversed") if dims.len() == 1 => {
                let out_ty = Ty::list((**e).clone());
                let view_ty = Ty::ListView(e.clone(), Dim::Int);
                match self.positional_any(method, args, 0, span) {
                    Some(_) => {
                        let rev = self.mk(
                            ExprKind::Builtin(Builtin::Reversed, vec![value]),
                            view_ty,
                            span,
                        );
                        self.mk(ExprKind::Builtin(Builtin::ToList, vec![rev]), out_ty, span)
                    }
                    None => self.err_expr(span),
                }
            }
            (
                Ty::List(_)
                | Ty::ListView(_, _)
                | Ty::Keyed(_, _)
                | Ty::MutView(_)
                | Ty::Set(_)
                | Ty::HashSet(_)
                | Ty::Map(..)
                | Ty::HashMap(..)
                | Ty::Array { .. }
                | Ty::Str
                | Ty::StrView
                | Ty::Range(_),
                "contains",
            ) => match self.one_arg(method, args, &elem, span) {
                Some(arg) => self.mk(
                    ExprKind::Builtin(Builtin::Contains, vec![value, arg]),
                    Ty::Bool,
                    span,
                ),
                None => self.err_expr(span),
            },
            // R04b.1.6: `z.rekey(s.keys)` hands a table computed on offsets back
            // in the caller's keys, O(1).
            (Ty::List(_) | Ty::ListView(_, _) | Ty::Keyed(_, _) | Ty::MutView(_), "rekey")
            | (Ty::Array { .. }, "rekey")
                if args.len() == 1 && ty.seq_key().is_some() =>
            {
                let Some(exprs) = self.positional_any(method, args, 1, span) else {
                    return self.err_expr(span);
                };
                let keys = self.expr(exprs[0], None);
                let dim = match &keys.ty {
                    Ty::Range(k) => match **k {
                        Ty::Pos(d) | Ty::Bound(d) => Dim::Pos(d),
                        _ => Dim::Int,
                    },
                    _ => return self.err_expr(span),
                };
                // A view must not outlive its cells, so a temporary gets a slot.
                let value = if matches!(value.kind, ExprKind::Local(_)) {
                    value
                } else {
                    let slot = self.new_slot("cells", value.ty.clone());
                    self.cx.pending.push(Stmt::Let(slot, Some(value)));
                    self.local(slot, span)
                };
                let res = if matches!(ty, Ty::List(_) | Ty::Array { .. }) {
                    Ty::Keyed(Box::new(elem.clone()), dim)
                } else {
                    Ty::ListView(Box::new(elem.clone()), dim)
                };
                self.do_expr(Some(&res.clone()), span, |lw, out| {
                    let keys = lw.hold(keys, out);
                    let lo = lw.mk(
                        ExprKind::Builtin(Builtin::RangeLo, vec![keys.clone()]),
                        Ty::Int,
                        span,
                    );
                    let hi = lw.mk(
                        ExprKind::Builtin(Builtin::RangeHi, vec![keys]),
                        Ty::Int,
                        span,
                    );
                    let view = lw.mk(
                        ExprKind::Builtin(Builtin::Rekey, vec![value, lo, hi]),
                        res,
                        span,
                    );
                    out.push(Stmt::Give(view));
                })
            }
            // How many cells equal `x`: `[1, 2, 1].count(1)` is 2,
            // `s[0..k].count('R')` scans the view directly without copying.
            (
                Ty::List(_)
                | Ty::ListView(_, _)
                | Ty::Keyed(_, _)
                | Ty::MutView(_)
                | Ty::Array { .. },
                "count",
            )
            | (Ty::Str | Ty::StrView, "count")
                if args.len() == 1
                    && (matches!(
                        &ty,
                        Ty::List(_)
                            | Ty::ListView(_, _)
                            | Ty::Keyed(_, _)
                            | Ty::MutView(_)
                            | Ty::Array { .. }
                    ) || self.probe_ty_silent(&args[0].value) == Some(Ty::Char)) =>
            {
                match self.one_arg(method, args, &elem, span) {
                    Some(arg) => self.do_expr(Some(&Ty::Int), span, |lw, out| {
                        let (seq, arg) = (lw.hold(value, out), lw.hold(arg, out));
                        let (acc, each) = (lw.temp(Ty::Int), lw.temp(elem.clone()));
                        out.push(Stmt::Let(acc, Some(lw.int_lit(0, span))));
                        let same = lw.mk(
                            ExprKind::Cmp(CmpOp::Eq, Box::new(lw.local(each, span)), Box::new(arg)),
                            Ty::Bool,
                            span,
                        );
                        let place = Place {
                            kind: PlaceKind::Local(acc),
                            ty: Ty::Int,
                            span: lw.span(span),
                        };
                        let inc =
                            Stmt::Assign(place, AssignOp::Binary(BinOp::Add), lw.int_lit(1, span));
                        let body = Block {
                            stmts: vec![Stmt::If(
                                same,
                                Block { stmts: vec![inc] },
                                Block::default(),
                            )],
                        };
                        let lid = lw.fresh_loop();
                        out.push(Stmt::For(lid, each, Iter::Seq(seq, false), body));
                        out.push(Stmt::Give(lw.local(acc, span)));
                    }),
                    None => self.err_expr(span),
                }
            }
            (Ty::Map(_, val) | Ty::HashMap(_, val), "get") => {
                match self.one_arg(method, args, &elem, span) {
                    Some(arg) => self.mk(
                        ExprKind::Builtin(Builtin::MapGet, vec![value, arg]),
                        Ty::Opt(val.clone()),
                        span,
                    ),
                    None => self.err_expr(span),
                }
            }
            (Ty::Set(key) | Ty::Map(key, _), "first_ge" | "first_gt" | "last_le" | "last_lt") => {
                let op = match method {
                    "first_ge" => Builtin::FirstGe,
                    "first_gt" => Builtin::FirstGt,
                    "last_le" => Builtin::LastLe,
                    _ => Builtin::LastLt,
                };
                let found = match &ty {
                    Ty::Map(key, val) => Ty::Tuple(vec![(**key).clone(), (**val).clone()]),
                    _ => (**key).clone(),
                };
                match self.one_arg(method, args, key, span) {
                    Some(bound) => self.mk(
                        ExprKind::Builtin(op, vec![value, bound]),
                        Ty::opt(found),
                        span,
                    ),
                    None => self.err_expr(span),
                }
            }
            (Ty::Set(_), "min" | "max") => match self.positional_any(method, args, 0, span) {
                Some(_) => {
                    let op = if method == "min" {
                        Builtin::SetMin
                    } else {
                        Builtin::SetMax
                    };
                    self.mk(ExprKind::Builtin(op, vec![value]), Ty::opt(elem), span)
                }
                None => self.err_expr(span),
            },
            (Ty::DomainVal | Ty::Pos(_), "nth" | "at" | "from_idx") => {
                let Some(idx_expr) = self.one_arg(method, args, &Ty::Int, span) else {
                    return self.err_expr(span);
                };
                let dom_id = match value.kind {
                    ExprKind::DomainVal(d) => Some(d),
                    _ => match value.ty {
                        Ty::Pos(d) => Some(d),
                        _ => self.gen_env.values().find_map(|ga| match ga {
                            GenArg::Domain(d) => Some(*d),
                            _ => None,
                        }),
                    },
                };
                match dom_id {
                    Some(d) if method == "at" => self.domain_try_label(d, idx_expr, span),
                    Some(d) => self.mk(
                        ExprKind::Cast(Cast::Trust, Box::new(idx_expr)),
                        Ty::Pos(d),
                        span,
                    ),
                    None => self.err_expr(span),
                }
            }
            (Ty::DomainVal, "windows") => {
                let Some(k_expr) = self.one_arg(method, args, &Ty::Int, span) else {
                    return self.err_expr(span);
                };
                let dom_id = match value.kind {
                    ExprKind::DomainVal(d) => Some(d),
                    _ => None,
                };
                let range_ty = match dom_id {
                    Some(d) => Ty::Range(Box::new(Ty::Pos(d))),
                    None => Ty::Range(Box::new(Ty::Int)),
                };
                let list_ty = Ty::list(range_ty.clone());
                let list_ty_hint = list_ty.clone();
                self.do_expr(Some(&list_ty_hint), span, |lw, out| {
                    let k_slot = lw.temp(Ty::Int);
                    out.push(Stmt::Let(k_slot, Some(k_expr)));
                    let k_ref = lw.local(k_slot, span);
                    let n_expr = match dom_id {
                        Some(d) => lw.mk(ExprKind::DomainLen(d), Ty::Int, span),
                        None => lw.mk(ExprKind::Builtin(Builtin::Len, vec![value]), Ty::Int, span),
                    };
                    let n_slot = lw.temp(Ty::Int);
                    out.push(Stmt::Let(n_slot, Some(n_expr)));
                    let n_ref = lw.local(n_slot, span);
                    let acc_slot = lw.temp(list_ty.clone());
                    out.push(Stmt::Let(
                        acc_slot,
                        Some(lw.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span)),
                    ));
                    let lim = lw.mk(
                        ExprKind::Binary(BinOp::Sub, Box::new(n_ref), Box::new(k_ref.clone())),
                        Ty::Int,
                        span,
                    );
                    let hi_loop = lw.int_add(lim, 1);
                    let i_slot = lw.temp(Ty::Int);
                    let i_ref = lw.local(i_slot, span);
                    let end_e = lw.mk(
                        ExprKind::Binary(BinOp::Add, Box::new(i_ref.clone()), Box::new(k_ref)),
                        Ty::Int,
                        span,
                    );
                    let rng = lw.mk(
                        ExprKind::Range {
                            lo: Box::new(i_ref),
                            hi: Box::new(end_e),
                        },
                        range_ty,
                        span,
                    );
                    let push = lw.mk(
                        ExprKind::Mutate(
                            Mutation::Push,
                            Box::new(Place {
                                kind: PlaceKind::Local(acc_slot),
                                ty: list_ty.clone(),
                                span,
                            }),
                            vec![rng],
                        ),
                        Ty::Unit,
                        span,
                    );
                    let lid = lw.next_loop;
                    lw.next_loop += 1;
                    out.push(Stmt::For(
                        lid,
                        i_slot,
                        Iter::Range {
                            lo: lw.int_lit(0, span),
                            hi: hi_loop,
                            step: None,
                            rev: false,
                            closed: false,
                        },
                        Block {
                            stmts: vec![Stmt::Expr(push)],
                        },
                    ));
                    out.push(Stmt::Give(lw.local(acc_slot, span)));
                })
            }
            // `xs.find(x => x > 3)` is the first element the test holds
            // for; `xs.find(3)` is where the value 3 is. Handled uniformly across
            // every `Indexed` sequence (`List`, `ListView`, 1D `[D]T`, `Str`, `StrView`, `Range`).
            (ty, "find" | "rfind")
                if ty.is_indexed() && args.len() == 1 && self.is_test(&args[0].value) =>
            {
                let elem = ty.seq_elem().unwrap();
                self.find_first(value, elem, &args[0].value, method == "rfind", span)
            }
            (ty, "find" | "rfind")
                if matches!(ty, Ty::Str | Ty::StrView)
                    && args.len() == 1
                    && args[0].name.is_none()
                    && matches!(
                        self.probe_ty_silent(&args[0].value),
                        Some(Ty::Str | Ty::StrView)
                    ) =>
            {
                let pat = self.expr(&args[0].value, None);
                let rev = method == "rfind";
                let out_ty = Ty::opt(Ty::Int);
                let out_ty_hint = out_ty.clone();
                self.do_expr(Some(&out_ty_hint), span, |lw, out| {
                    let s_slot = lw.temp(value.ty.clone());
                    out.push(Stmt::Let(s_slot, Some(value)));
                    let s_ref = lw.local(s_slot, span);
                    let p_slot = lw.temp(pat.ty.clone());
                    out.push(Stmt::Let(p_slot, Some(pat)));
                    let p_ref = lw.local(p_slot, span);
                    let n_slot = lw.temp(Ty::Int);
                    let s_len = lw.mk(
                        ExprKind::Builtin(Builtin::Len, vec![s_ref.clone()]),
                        Ty::Int,
                        span,
                    );
                    out.push(Stmt::Let(n_slot, Some(s_len)));
                    let m_slot = lw.temp(Ty::Int);
                    let p_len = lw.mk(
                        ExprKind::Builtin(Builtin::Len, vec![p_ref.clone()]),
                        Ty::Int,
                        span,
                    );
                    out.push(Stmt::Let(m_slot, Some(p_len)));
                    let res_slot = lw.temp(out_ty.clone());
                    out.push(Stmt::Let(
                        res_slot,
                        Some(lw.mk(ExprKind::None, out_ty.clone(), span)),
                    ));
                    let diff = lw.mk(
                        ExprKind::Binary(
                            BinOp::Sub,
                            Box::new(lw.local(n_slot, span)),
                            Box::new(lw.local(m_slot, span)),
                        ),
                        Ty::Int,
                        span,
                    );
                    let hi = lw.int_add(diff, 1);
                    let i_slot = lw.temp(Ty::Int);
                    let i_ref = lw.local(i_slot, span);
                    let ok_slot = lw.temp(Ty::Bool);
                    let j_slot = lw.temp(Ty::Int);
                    let j_ref = lw.local(j_slot, span);
                    let ij = lw.mk(
                        ExprKind::Binary(
                            BinOp::Add,
                            Box::new(i_ref.clone()),
                            Box::new(j_ref.clone()),
                        ),
                        Ty::Int,
                        span,
                    );
                    let sc = lw.mk(ExprKind::Index(Box::new(s_ref), vec![ij]), Ty::Char, span);
                    let pc = lw.mk(
                        ExprKind::Index(Box::new(p_ref), vec![j_ref]),
                        Ty::Char,
                        span,
                    );
                    let ne = lw.mk(
                        ExprKind::Cmp(CmpOp::Ne, Box::new(sc), Box::new(pc)),
                        Ty::Bool,
                        span,
                    );
                    let inner_lid = lw.next_loop;
                    lw.next_loop += 1;
                    let outer_lid = lw.next_loop;
                    lw.next_loop += 1;
                    out.push(Stmt::For(
                        outer_lid,
                        i_slot,
                        Iter::Range {
                            lo: lw.int_lit(0, span),
                            hi,
                            step: None,
                            rev,
                            closed: false,
                        },
                        Block {
                            stmts: vec![
                                Stmt::Let(
                                    ok_slot,
                                    Some(lw.mk(ExprKind::Bool(true), Ty::Bool, span)),
                                ),
                                Stmt::For(
                                    inner_lid,
                                    j_slot,
                                    Iter::Range {
                                        lo: lw.int_lit(0, span),
                                        hi: lw.local(m_slot, span),
                                        step: None,
                                        rev: false,
                                        closed: false,
                                    },
                                    Block {
                                        stmts: vec![Stmt::If(
                                            ne,
                                            Block {
                                                stmts: vec![
                                                    Stmt::Assign(
                                                        Place {
                                                            kind: PlaceKind::Local(ok_slot),
                                                            ty: Ty::Bool,
                                                            span,
                                                        },
                                                        AssignOp::Set,
                                                        lw.mk(
                                                            ExprKind::Bool(false),
                                                            Ty::Bool,
                                                            span,
                                                        ),
                                                    ),
                                                    Stmt::Break(None),
                                                ],
                                            },
                                            Block::default(),
                                        )],
                                    },
                                ),
                                Stmt::If(
                                    lw.local(ok_slot, span),
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
                                                    ExprKind::Some(Box::new(i_ref)),
                                                    out_ty.clone(),
                                                    span,
                                                ),
                                            ),
                                            Stmt::Break(None),
                                        ],
                                    },
                                    Block::default(),
                                ),
                            ],
                        },
                    ));
                    out.push(Stmt::Give(lw.local(res_slot, span)));
                })
            }
            (ty, "first_ge" | "first_gt" | "last_le" | "last_lt" | "find" | "rfind")
                if ty.is_indexed() =>
            {
                let dom = match &ty {
                    Ty::Array { dims, .. } if dims.len() == 1 => match dims[0] {
                        Dim::Pos(d) => Some(d),
                        _ => None,
                    },
                    _ => None,
                };
                let elem = ty.seq_elem().unwrap();
                self.seq_search_method(method, value, elem, dom, args, span)
            }
            (Ty::Fn(_, _), "out") => self.call_fn_val(value, args, span),
            // `Neighbours` for a table of neighbour lists: `adj.out(u)` is
            // `adj[u]`, as it is `adj(u)` for a fn.
            (Ty::Array { dims, elem }, "out")
                if dims.len() == 1 && args.len() == 1 && matches!(**elem, Ty::List(_)) =>
            {
                let at = ast::Expr {
                    kind: ast::ExprKind::Index {
                        base: Box::new(base.clone()),
                        indices: vec![args[0].value.clone()],
                    },
                    span,
                };
                self.expr(&at, None)
            }
            // `r.nth(k)` on a range of positions is `r[k]`, checked against `r`.
            (Ty::Range(inner), "nth") if matches!(**inner, Ty::Pos(_)) && args.len() == 1 => {
                let at = ast::Expr {
                    kind: ast::ExprKind::Index {
                        base: Box::new(base.clone()),
                        indices: vec![args[0].value.clone()],
                    },
                    span,
                };
                self.expr(&at, None)
            }
            // `r.nth(k)` on an Int range: its `k`-th value, `r.start + k` (R04b.5.9).
            (Ty::Range(inner), "nth") if **inner == Ty::Int && args.len() == 1 => {
                let Some(k) = self.one_arg("nth", args, &Ty::Int, span) else {
                    return self.err_expr(span);
                };
                let lo = self.mk(
                    ExprKind::Builtin(Builtin::RangeLo, vec![value]),
                    Ty::Int,
                    span,
                );
                self.mk(
                    ExprKind::Binary(BinOp::Add, Box::new(lo), Box::new(k)),
                    Ty::Int,
                    span,
                )
            }
            // `(lo..hi).clamp(D)`: the part of an Int range that lies in
            // `D`, as a range of D's positions; empty when none does.
            (Ty::Range(inner), "clamp") if **inner == Ty::Int && args.len() == 1 => {
                let domain = match &args[0].value.kind {
                    ast::ExprKind::Name(name) if self.lookup(name).is_none() => {
                        match (self.gen_env.get(name.as_str()), self.get_sym(name)) {
                            (Some(GenArg::Domain(id)), _) => Some(*id),
                            (_, Some(Sym::Domain(id))) => Some(id),
                            _ => None,
                        }
                    }
                    _ => None,
                };
                let Some(d) = domain else {
                    self.error(
                        codes::TYPE_MISMATCH,
                        args[0].value.span,
                        "`clamp` takes a domain",
                    );
                    self.help("write the domain the range is cut to: `(l..=r).clamp(V)`");
                    return self.err_expr(span);
                };
                let range_ty = Ty::Range(Box::new(Ty::Pos(d)));
                self.do_expr(Some(&range_ty.clone()), span, |lw, out| {
                    let range = lw.new_slot("range", value.ty.clone());
                    out.push(Stmt::Let(range, Some(value)));
                    // An end of the range as an ordinal of `D`, kept in 0..=len.
                    let end = |lw: &mut Self, which| {
                        let range = lw.local(range, span);
                        let at = lw.mk(ExprKind::Builtin(which, vec![range]), Ty::Int, span);
                        let first = lw.mk(ExprKind::DomainLo(d), Ty::Int, span);
                        let ord = lw.mk(
                            ExprKind::Binary(BinOp::Sub, Box::new(at), Box::new(first)),
                            Ty::Int,
                            span,
                        );
                        let len = lw.mk(ExprKind::DomainLen(d), Ty::Int, span);
                        let zero = lw.int_lit(0, span);
                        let capped = lw.mk(
                            ExprKind::Builtin(Builtin::Min, vec![ord, len]),
                            Ty::Int,
                            span,
                        );
                        lw.mk(
                            ExprKind::Builtin(Builtin::Max, vec![capped, zero]),
                            Ty::Int,
                            span,
                        )
                    };
                    let lo = lw.new_slot("lo", Ty::Int);
                    let from = end(lw, Builtin::RangeLo);
                    out.push(Stmt::Let(lo, Some(from)));
                    let to = end(lw, Builtin::RangeHi);
                    // An empty range stays empty: its end is not before its start.
                    let lo_again = lw.local(lo, span);
                    let hi = lw.mk(
                        ExprKind::Builtin(Builtin::Max, vec![to, lo_again]),
                        Ty::Int,
                        span,
                    );
                    let lo = lw.local(lo, span);
                    let cut = lw.mk(
                        ExprKind::Range {
                            lo: Box::new(lo),
                            hi: Box::new(hi),
                        },
                        range_ty.clone(),
                        span,
                    );
                    out.push(Stmt::Give(cut));
                })
            }
            // `r.drop(k)` / `r.take(k)` on `Range<Int>` or `Range<D>`: sub-range.
            (Ty::Range(inner), "drop" | "take" | "drop_last" | "take_last")
                if **inner != Ty::I128 =>
            {
                // The new end is `shift` in from `lo` or from `hi`, and replaces `lo` or `hi`.
                let (from_hi, cut_lo) = match method {
                    "drop" => (false, true),
                    "take" => (false, false),
                    "drop_last" => (true, false),
                    _ => (true, true),
                };
                let range_ty = value.ty.clone();
                let Some(k) = self.one_arg(method, args, &Ty::Int, span) else {
                    return self.err_expr(span);
                };
                self.do_expr(Some(&range_ty), span, |lw, out| {
                    let range = lw.new_slot("range", range_ty.clone());
                    out.push(Stmt::Let(range, Some(value)));
                    let lo_s = lw.new_slot("lo", Ty::Int);
                    let hi_s = lw.new_slot("hi", Ty::Int);
                    let r1 = lw.local(range, span);
                    let r2 = lw.local(range, span);
                    out.push(Stmt::Let(
                        lo_s,
                        Some(lw.mk(ExprKind::Builtin(Builtin::RangeLo, vec![r1]), Ty::Int, span)),
                    ));
                    out.push(Stmt::Let(
                        hi_s,
                        Some(lw.mk(ExprKind::Builtin(Builtin::RangeHi, vec![r2]), Ty::Int, span)),
                    ));
                    let k_s = lw.new_slot("k", Ty::Int);
                    let zero = lw.mk(ExprKind::Int(0), Ty::Int, span);
                    let k_clamped = lw.mk(
                        ExprKind::Builtin(Builtin::Max, vec![k, zero]),
                        Ty::Int,
                        span,
                    );
                    out.push(Stmt::Let(k_s, Some(k_clamped)));
                    let diff = lw.mk(
                        ExprKind::Binary(
                            BinOp::Sub,
                            Box::new(lw.local(hi_s, span)),
                            Box::new(lw.local(lo_s, span)),
                        ),
                        Ty::Int,
                        span,
                    );
                    let zero2 = lw.mk(ExprKind::Int(0), Ty::Int, span);
                    let avail = lw.mk(
                        ExprKind::Builtin(Builtin::Max, vec![diff, zero2]),
                        Ty::Int,
                        span,
                    );
                    let shift = lw.mk(
                        ExprKind::Builtin(Builtin::Min, vec![avail, lw.local(k_s, span)]),
                        Ty::Int,
                        span,
                    );
                    let (end, op) = if from_hi {
                        (hi_s, BinOp::Sub)
                    } else {
                        (lo_s, BinOp::Add)
                    };
                    let point = lw.mk(
                        ExprKind::Binary(op, Box::new(lw.local(end, span)), Box::new(shift)),
                        Ty::Int,
                        span,
                    );
                    let (new_lo, new_hi) = if cut_lo {
                        (point, lw.local(hi_s, span))
                    } else {
                        (lw.local(lo_s, span), point)
                    };
                    let res = lw.mk(
                        ExprKind::Range {
                            lo: Box::new(new_lo),
                            hi: Box::new(new_hi),
                        },
                        range_ty.clone(),
                        span,
                    );
                    out.push(Stmt::Give(res));
                })
            }
            // `(lo..hi).step(k)`: lo, lo + k, ... below hi, as a list.
            (Ty::Range(inner), "step") if **inner == Ty::Int && args.len() == 1 => {
                let by = self.expr(&args[0].value, Some(&Ty::Int));
                let by = self.coerce(by, &Ty::Int, span);
                let list_ty = Ty::list(Ty::Int);
                self.do_expr(Some(&list_ty.clone()), span, |lw, out| {
                    let range = lw.new_slot("range", value.ty.clone());
                    out.push(Stmt::Let(range, Some(value)));
                    let acc = lw.new_slot("steps", list_ty.clone());
                    let empty = lw.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span);
                    out.push(Stmt::Let(acc, Some(empty)));
                    let i = lw.new_slot("i", Ty::Int);
                    let bound = |lw: &mut Self, which| {
                        let range = lw.local(range, span);
                        lw.mk(ExprKind::Builtin(which, vec![range]), Ty::Int, span)
                    };
                    let (lo, hi) = (bound(lw, Builtin::RangeLo), bound(lw, Builtin::RangeHi));
                    let item = lw.local(i, span);
                    let push = lw.mk(
                        ExprKind::Mutate(
                            Mutation::Push,
                            Box::new(Place {
                                kind: PlaceKind::Local(acc),
                                ty: list_ty.clone(),
                                span: lw.span(span),
                            }),
                            vec![item],
                        ),
                        Ty::Unit,
                        span,
                    );
                    let lid = lw.next_loop;
                    lw.next_loop += 1;
                    out.push(Stmt::For(
                        lid,
                        i,
                        Iter::Range {
                            lo,
                            hi,
                            step: Some(by),
                            rev: false,
                            closed: false,
                        },
                        Block {
                            stmts: vec![Stmt::Expr(push)],
                        },
                    ));
                    out.push(Stmt::Give(lw.local(acc, span)));
                })
            }
            // `xs.last(k)`: the element `k` back from the last one, so
            // `xs.last(1)` is the one before the last; `none` when there
            // are not that many.
            (ty, "last") if ty.is_indexed() && args.len() == 1 => {
                let elem = ty.seq_elem().unwrap();
                // An array of positions is read at the position that far in.
                let of = match &ty {
                    Ty::Array { dims, .. } => match dims[0] {
                        Dim::Pos(d) => Some(d),
                        _ => None,
                    },
                    _ => None,
                };
                let back = self.expect(&args[0].value, &Ty::Int);
                self.do_expr(Some(&Ty::opt(elem.clone())), span, |lw, out| {
                    // A list that is named is read where it is; one that is
                    // computed is kept for the two reads.
                    let list = if matches!(value.kind, ExprKind::Local(_) | ExprKind::Global(_)) {
                        value
                    } else {
                        let kept = lw.temp(value.ty.clone());
                        out.push(Stmt::Let(kept, Some(value)));
                        lw.local(kept, span)
                    };
                    let k = lw.temp(Ty::Int);
                    out.push(Stmt::Let(k, Some(back)));
                    let len = lw.mk(
                        ExprKind::Builtin(Builtin::Len, vec![list.clone()]),
                        Ty::Int,
                        span,
                    );
                    let last = lw.int_add(len, -1);
                    let at = lw.temp(Ty::Int);
                    let from_end = lw.mk(
                        ExprKind::Binary(BinOp::Sub, Box::new(last), Box::new(lw.local(k, span))),
                        Ty::Int,
                        span,
                    );
                    out.push(Stmt::Let(at, Some(from_end)));
                    let cmp = |lw: &mut Self, slot| {
                        let zero = lw.int_lit(0, span);
                        lw.mk(
                            ExprKind::Cmp(
                                CmpOp::Ge,
                                Box::new(lw.local(slot, span)),
                                Box::new(zero),
                            ),
                            Ty::Bool,
                            span,
                        )
                    };
                    let (ahead, inside) = (cmp(lw, k), cmp(lw, at));
                    let there = lw.mk(
                        ExprKind::Binary(BinOp::And, Box::new(ahead), Box::new(inside)),
                        Ty::Bool,
                        span,
                    );
                    let cell = match of {
                        Some(d) => lw.mk(
                            ExprKind::Cast(Cast::FromOrd(d), Box::new(lw.local(at, span))),
                            Ty::Pos(d),
                            span,
                        ),
                        None => lw.local(at, span),
                    };
                    let item = lw.mk(
                        ExprKind::Index(Box::new(list), vec![cell]),
                        elem.clone(),
                        span,
                    );
                    let opt = Ty::opt(elem);
                    let some = lw.mk(ExprKind::Some(Box::new(item)), opt.clone(), span);
                    let none = lw.mk(ExprKind::None, opt.clone(), span);
                    out.push(Stmt::Give(lw.mk(
                        ExprKind::If(Box::new(there), Box::new(some), Box::new(none)),
                        opt,
                        span,
                    )));
                })
            }
            (Ty::Str | Ty::StrView, "rev") if args.is_empty() => {
                let value = self.hoist_view_base(value, span);
                self.mk(
                    ExprKind::Builtin(Builtin::Reversed, vec![value]),
                    Ty::StrView,
                    span,
                )
            }
            (Ty::List(e) | Ty::ListView(e, _) | Ty::Keyed(e, _) | Ty::MutView(e), "rev")
                if args.is_empty() =>
            {
                let elem = (**e).clone();
                let value = self.hoist_view_base(value, span);
                self.mk(
                    ExprKind::Builtin(Builtin::Reversed, vec![value]),
                    Ty::ListView(Box::new(elem), Dim::Int),
                    span,
                )
            }
            (Ty::Range(_), "rev") if args.is_empty() => self.reversed_range(value, span),
            // `r.halves()`: split `r` into `(left, right)` with `left.len == (r.len + 1) / 2`
            // and `right.len == r.len / 2` (both non-empty whenever `r.len >= 2`).
            (Ty::Range(_), "halves") if args.is_empty() => {
                let r_ty = value.ty.clone();
                let tup_ty = Ty::Tuple(vec![r_ty.clone(), r_ty.clone()]);
                self.do_expr(Some(&tup_ty.clone()), span, |lw, out| {
                    let r_slot = lw.temp(r_ty.clone());
                    out.push(Stmt::Let(r_slot, Some(value)));
                    let r_ref = lw.local(r_slot, span);
                    let lo_slot = lw.temp(Ty::Int);
                    let hi_slot = lw.temp(Ty::Int);
                    let lo_expr = lw.mk(
                        ExprKind::Builtin(Builtin::RangeLo, vec![r_ref.clone()]),
                        Ty::Int,
                        span,
                    );
                    let raw_hi = lw.mk(
                        ExprKind::Builtin(Builtin::RangeHi, vec![r_ref]),
                        Ty::Int,
                        span,
                    );
                    out.push(Stmt::Let(lo_slot, Some(lo_expr)));
                    let hi_expr = lw.mk(
                        ExprKind::Builtin(Builtin::Max, vec![raw_hi, lw.local(lo_slot, span)]),
                        Ty::Int,
                        span,
                    );
                    out.push(Stmt::Let(hi_slot, Some(hi_expr)));
                    let diff = lw.mk(
                        ExprKind::Binary(
                            BinOp::Sub,
                            Box::new(lw.local(hi_slot, span)),
                            Box::new(lw.local(lo_slot, span)),
                        ),
                        Ty::Int,
                        span,
                    );
                    let left_len = lw.mk(
                        ExprKind::Binary(
                            BinOp::Div,
                            Box::new(lw.int_add(diff, 1)),
                            Box::new(lw.int_lit(2, span)),
                        ),
                        Ty::Int,
                        span,
                    );
                    let mid_slot = lw.temp(Ty::Int);
                    let mid_expr = lw.mk(
                        ExprKind::Binary(
                            BinOp::Add,
                            Box::new(lw.local(lo_slot, span)),
                            Box::new(left_len),
                        ),
                        Ty::Int,
                        span,
                    );
                    out.push(Stmt::Let(mid_slot, Some(mid_expr)));
                    let left = lw.mk(
                        ExprKind::Range {
                            lo: Box::new(lw.local(lo_slot, span)),
                            hi: Box::new(lw.local(mid_slot, span)),
                        },
                        r_ty.clone(),
                        span,
                    );
                    let right = lw.mk(
                        ExprKind::Range {
                            lo: Box::new(lw.local(mid_slot, span)),
                            hi: Box::new(lw.local(hi_slot, span)),
                        },
                        r_ty.clone(),
                        span,
                    );
                    let pair = lw.mk(ExprKind::Tuple(vec![left, right]), tup_ty.clone(), span);
                    out.push(Stmt::Give(pair));
                })
            }
            // `r.find(i => ..)`: the first of the range the test holds
            // for, `none` when there is none. `(0..).find(..)` has no end
            // and stops there.
            (Ty::Range(elem), "find" | "rfind") if args.len() == 1 => {
                let elem = (**elem).clone();
                self.find_first(value, elem, &args[0].value, method == "rfind", span)
            }
            (Ty::Array { dims, elem }, "rev") if dims.len() == 1 && args.is_empty() => {
                let elem = (**elem).clone();
                let value = self.hoist_view_base(value, span);
                self.mk(
                    ExprKind::Builtin(Builtin::Reversed, vec![value]),
                    Ty::ListView(Box::new(elem), Dim::Int),
                    span,
                )
            }
            // A sequence walked backwards is something to loop over. As a
            // value it would be the sequence unchanged, which is not what
            // `.rev()` says.
            (
                Ty::Array { .. } | Ty::Set(_) | Ty::Map(..) | Ty::HashSet(_) | Ty::HashMap(..),
                "rev",
            ) if args.is_empty() => {
                let message = format!(
                    "a reversed `{}` is something to loop over, not a value: \
                     `for x in xs.rev`, or `[x for x in xs.rev]` for the list",
                    self.show(&ty)
                );
                self.error(codes::UNSUPPORTED, name.span, message);
                self.err_expr(span)
            }
            (Ty::DomainVal, "mask") if args.len() == 1 && args[0].name.is_none() => {
                if let ExprKind::DomainVal(d) = value.kind {
                    let arr_ty = Ty::Array {
                        dims: vec![Dim::Pos(d)],
                        elem: Box::new(Ty::Bool),
                    };
                    let list = self.collect_iter_to_list(&args[0].value, span);
                    let elem_ty = match &list.ty {
                        Ty::List(e) => (**e).clone(),
                        _ => Ty::Pos(d),
                    };
                    self.do_expr(Some(&arr_ty), span, |lw, out| {
                        let m_slot = lw.temp(arr_ty.clone());
                        let false_val = lw.mk(ExprKind::Bool(false), Ty::Bool, span);
                        let fill = lw.mk(
                            ExprKind::Fill(vec![DimSpec::Pos(d)], Some(Box::new(false_val))),
                            arr_ty.clone(),
                            span,
                        );
                        out.push(Stmt::Let(m_slot, Some(fill)));
                        let x_slot = lw.temp(elem_ty.clone());
                        let x_val = lw.coerce(lw.local(x_slot, span), &Ty::Pos(d), span);
                        let place = Place {
                            kind: PlaceKind::Index(
                                Box::new(Place {
                                    kind: PlaceKind::Local(m_slot),
                                    ty: arr_ty.clone(),
                                    span,
                                }),
                                vec![x_val],
                            ),
                            ty: Ty::Bool,
                            span,
                        };
                        let lid = lw.next_loop;
                        lw.next_loop += 1;
                        out.push(Stmt::For(
                            lid,
                            x_slot,
                            Iter::Seq(list, false),
                            Block {
                                stmts: vec![Stmt::Assign(
                                    place,
                                    AssignOp::Set,
                                    lw.mk(ExprKind::Bool(true), Ty::Bool, span),
                                )],
                            },
                        ));
                        out.push(Stmt::Give(lw.local(m_slot, span)));
                    })
                } else {
                    self.err_expr(span)
                }
            }
            // `v.next(k)`, `v.prev(k)`: step k positions, `None` past an end (R04.8.1).
            (Ty::Pos(d), "next" | "prev") if args.len() == 1 => {
                let d = *d;
                let op = if method == "next" {
                    BinOp::Add
                } else {
                    BinOp::Sub
                };
                let Some(k) = self.one_arg(method, args, &Ty::Int, span) else {
                    return self.err_expr(span);
                };
                let ord = self.cast(Cast::Ord, value, Ty::Int);
                let at = self.mk(
                    ExprKind::Binary(op, Box::new(ord), Box::new(k)),
                    Ty::Int,
                    span,
                );
                self.mk(
                    ExprKind::Builtin(Builtin::PosTryOrd(d), vec![at]),
                    Ty::opt(Ty::Pos(d)),
                    span,
                )
            }
            // `Reversible.rev`: a value that is not a sequence and has no
            // `rev` of its own is its own reverse.
            (_, "rev") if args.is_empty() => value,
            _ => {
                let prim = self
                    .prim_methods
                    .iter()
                    .find(|(t, m, _)| *t == ty && m == method)
                    .map(|p| p.2);
                if let Some(fid) = prim.or_else(|| self.optional_impl_method(&ty, method)) {
                    let mut all_args = vec![ast::Arg {
                        name: None,
                        mutable: false,
                        value: base.clone(),
                    }];
                    all_args.extend(args.iter().cloned());
                    return self.user_call(fid, None, &all_args, span);
                }
                // A domain is the sequence of its positions: `Cell.find(p)` is
                // `Cell.all.find(p)`.
                if ty == Ty::DomainVal && method != "all" {
                    let all = ast::Expr {
                        kind: ast::ExprKind::Field {
                            base: Box::new(base.clone()),
                            name: ast::Name {
                                text: "all".to_string(),
                                span: base.span,
                            },
                        },
                        span: base.span,
                    };
                    return self.method(whole, &all, name, args, hint);
                }
                if self.number_max_call(&ty, base, method, args, span) {
                    return self.err_expr(span);
                }
                // UFCS free function with `self` first parameter (e.g. `p.step(d)`, `p.ray(d)`).
                if let Some(res) = self.try_ufcs_call(method, base, args, span) {
                    return res;
                }
                let message = format!(
                    "{} has no method `{method}` that `cpl run` knows",
                    self.a_ty(&ty)
                );
                self.error(codes::UNKNOWN_FIELD, name.span, message);
                self.member_typo(&ty, method, name.span);
                self.foreign_call_fix(&ty, base, method, span);
                self.slice_call_fix(&ty, base, method, args, span);
                // Without the method's signature the arguments (and the
                // lambdas' parameters) have no types to check against.
                let reported = self.diags.len();
                for arg in args {
                    self.expr(&arg.value, None);
                }
                self.diags.truncate(reported);
                self.err_expr(span)
            }
        }
    }
}

/// Turns the `break` that ends an `all`/`any` early into one that leaves
/// the outermost loop of the comprehension.
fn retarget_break(stmts: &mut [Stmt], outer: LoopId, depth: usize) {
    for stmt in stmts {
        match stmt {
            Stmt::For(_, _, _, body) => retarget_break(&mut body.stmts, outer, depth + 1),
            Stmt::If(_, then, _) => retarget_break(&mut then.stmts, outer, depth),
            Stmt::Break(target @ None) if depth > 1 => *target = Some(outer),
            _ => {}
        }
    }
}

/// Whether `e` names somewhere a value lives: a variable, a field or an
/// element of one.
pub(super) fn is_place(e: &ast::Expr) -> bool {
    match &e.kind {
        ast::ExprKind::Name(_) | ast::ExprKind::SelfValue => true,
        ast::ExprKind::Paren(inner) => is_place(inner),
        ast::ExprKind::Field { base, .. } | ast::ExprKind::Index { base, .. } => is_place(base),
        _ => false,
    }
}

/// Makes every `?T` of `spec` read `absent` as `none`; false when it has no
/// `?T`.
pub(super) fn reads_none_as(spec: &mut Spec, absent: &Expr) -> bool {
    match spec {
        Spec::Plain(_) | Spec::Int { .. } | Spec::Call(_) | Spec::Parse(..) | Spec::Line => false,
        Spec::Fields(_, fields) => {
            let mut any = false;
            for field in fields {
                any |= reads_none_as(field, absent);
            }
            any
        }
        Spec::Array { elem, .. } | Spec::List(elem) => reads_none_as(elem, absent),
        Spec::Opt(_, slot) => {
            *slot = Some(Box::new(absent.clone()));
            true
        }
    }
}

/// What `for .. in g.items` goes through on a grid: `total` cells, and a
/// counter per axis that says which one the loop is at.
pub(super) struct GridItems {
    grid: Expr,
    dims: Vec<Dim>,
    elem: Ty,
    lens: Vec<Expr>,
    total: Expr,
    counters: Vec<Slot>,
}

impl GridItems {
    pub(super) fn range(&self) -> Iter {
        let lo = Expr {
            kind: ExprKind::Int(0),
            ty: Ty::Int,
            span: self.total.span,
        };
        Iter::Range {
            lo,
            hi: self.total.clone(),
            step: None,
            rev: false,
            closed: false,
        }
    }
}

pub(super) struct MapProj {
    map: Expr,
    pair_ty: Ty,
    elem_ty: Ty,
    idx: usize,
    rev: bool,
}

impl MapProj {
    pub(super) fn iter(&self) -> Iter {
        Iter::Seq(self.map.clone(), self.rev)
    }
}

/// What `xs.items` goes through: each of the `len` cells of `seq`, with
/// its index. An `Int` index starts at `first`.
pub(super) struct Items {
    seq: Expr,
    len: Expr,
    index: Ty,
    first: i64,
    /// The view names its cells by source keys, not walk positions.
    keyed: bool,
    elem: Ty,
    /// A range's first element: its cells are `lo + k`, not `seq[k]`.
    lo: Option<Expr>,
}

impl Items {
    pub(super) fn range(&self) -> Iter {
        let lo = Expr {
            kind: ExprKind::Int(0),
            ty: Ty::Int,
            span: self.len.span,
        };
        Iter::Range {
            lo,
            hi: self.len.clone(),
            step: None,
            rev: false,
            closed: false,
        }
    }
}

impl<'a> Lowerer<'a> {
    pub(super) fn sorted_container_key(&self, sid: StructId) -> Option<Ty> {
        let info = &self.structs[sid];
        if !info.def.name.starts_with("SortedBag") && !info.def.name.starts_with("SortedSet") {
            return None;
        }
        let (_, gargs) = info.origin_gen.as_ref()?;
        match gargs.first()? {
            GenArg::Ty(k) => Some(k.clone()),
            GenArg::Domain(d) => Some(Ty::Pos(*d)),
            _ => None,
        }
    }

    pub(super) fn is_sorted_range_expr(&mut self, mut e: &ast::Expr) -> bool {
        loop {
            match &e.kind {
                ast::ExprKind::Paren(inner) => e = inner,
                ast::ExprKind::Name(n) => {
                    return self
                        .lookup(n)
                        .is_some_and(|slot| self.cx.sorted_views.contains_key(&slot));
                }
                ast::ExprKind::Field { base, name } if name.text == "rev" => e = base,
                ast::ExprKind::Call { callee, args } if args.len() == 1 => {
                    if let ast::ExprKind::Field { base, name } = &callee.kind {
                        if name.text == "range" {
                            return match self.probe_ty_silent(base) {
                                Some(Ty::Struct(sid)) => self.sorted_container_key(sid).is_some(),
                                Some(Ty::Set(_) | Ty::Map(_, _)) => true,
                                _ => false,
                            };
                        }
                    }
                    return false;
                }
                _ => return false,
            }
        }
    }

    fn sorted_domain_bounds(
        &mut self,
        d: DomainId,
        lo_ord: Expr,
        hi_ord: Expr,
        span: Span,
        out: &mut Vec<Stmt>,
    ) -> (Expr, Expr, Expr, Expr, Option<Expr>) {
        let lo_s = self.temp(Ty::Int);
        out.push(Stmt::Let(lo_s, Some(lo_ord)));
        let hi_s = self.temp(Ty::Int);
        out.push(Stmt::Let(hi_s, Some(hi_ord)));
        let dlen_s = self.temp(Ty::Int);
        let dlen = self.mk(ExprKind::DomainLen(d), Ty::Int, span);
        out.push(Stmt::Let(dlen_s, Some(dlen)));

        let lo_l = self.local(lo_s, span);
        let hi_l = self.local(hi_s, span);
        let dlen_l = self.local(dlen_s, span);
        let zero = self.int_lit(0, span);

        let c1 = self.mk(
            ExprKind::Cmp(CmpOp::Ge, Box::new(lo_l.clone()), Box::new(hi_l.clone())),
            Ty::Bool,
            span,
        );
        let c2 = self.mk(
            ExprKind::Cmp(CmpOp::Ge, Box::new(lo_l.clone()), Box::new(dlen_l.clone())),
            Ty::Bool,
            span,
        );
        let c3 = self.mk(
            ExprKind::Cmp(CmpOp::Le, Box::new(hi_l.clone()), Box::new(zero.clone())),
            Ty::Bool,
            span,
        );
        let c12 = self.mk(
            ExprKind::Binary(BinOp::Or, Box::new(c1), Box::new(c2)),
            Ty::Bool,
            span,
        );
        let empty = self.mk(
            ExprKind::Binary(BinOp::Or, Box::new(c12), Box::new(c3)),
            Ty::Bool,
            span,
        );
        let empty_s = self.temp(Ty::Bool);
        out.push(Stmt::Let(empty_s, Some(empty)));
        let empty_l = self.local(empty_s, span);
        let not_empty = self.mk(
            ExprKind::Unary(UnOp::Not, Box::new(empty_l.clone())),
            Ty::Bool,
            span,
        );

        let opt_k = Ty::opt(Ty::Pos(d));
        let lo_gt_zero = self.mk(
            ExprKind::Cmp(CmpOp::Gt, Box::new(lo_l.clone()), Box::new(zero)),
            Ty::Bool,
            span,
        );
        let has_lo = self.mk(
            ExprKind::Binary(
                BinOp::And,
                Box::new(not_empty.clone()),
                Box::new(lo_gt_zero),
            ),
            Ty::Bool,
            span,
        );
        let lo_pos = self.cast(Cast::Trust, lo_l, Ty::Pos(d));
        let lo_some = self.mk(ExprKind::Some(Box::new(lo_pos)), opt_k.clone(), span);
        let lo_none = self.mk(ExprKind::None, opt_k.clone(), span);
        let lo_expr = self.mk(
            ExprKind::If(Box::new(has_lo), Box::new(lo_some), Box::new(lo_none)),
            opt_k.clone(),
            span,
        );

        let hi_lt_len = self.mk(
            ExprKind::Cmp(CmpOp::Lt, Box::new(hi_l.clone()), Box::new(dlen_l)),
            Ty::Bool,
            span,
        );
        let has_hi = self.mk(
            ExprKind::Binary(BinOp::And, Box::new(not_empty), Box::new(hi_lt_len)),
            Ty::Bool,
            span,
        );
        let hi_pos = self.cast(Cast::Trust, hi_l, Ty::Pos(d));
        let hi_some = self.mk(ExprKind::Some(Box::new(hi_pos)), opt_k.clone(), span);
        let hi_none = self.mk(ExprKind::None, opt_k.clone(), span);
        let hi_expr = self.mk(
            ExprKind::If(Box::new(has_hi), Box::new(hi_some), Box::new(hi_none)),
            opt_k,
            span,
        );

        let lo_inc = self.mk(ExprKind::Bool(true), Ty::Bool, span);
        let hi_inc = self.mk(ExprKind::Bool(false), Ty::Bool, span);
        (lo_expr, lo_inc, hi_expr, hi_inc, Some(empty_l))
    }

    pub(super) fn sorted_view_from_expr(
        &mut self,
        e: &ast::Expr,
        out: &mut Vec<Stmt>,
    ) -> Option<SortedView> {
        let mut cur = e;
        let mut flip = false;
        loop {
            match &cur.kind {
                ast::ExprKind::Paren(inner) => cur = inner,
                ast::ExprKind::Field { base, name } if name.text == "rev" => {
                    flip = !flip;
                    cur = base;
                }
                ast::ExprKind::Name(n) => {
                    let slot = self.lookup(n)?;
                    let mut v = self.cx.sorted_views.get(&slot)?.clone();
                    if flip {
                        v.rev = !v.rev;
                    }
                    return Some(v);
                }
                ast::ExprKind::Call { callee, args } if args.len() == 1 => {
                    let ast::ExprKind::Field { base, name } = &callee.kind else {
                        return None;
                    };
                    if name.text != "range" {
                        return None;
                    }
                    let (kind, key_ty, elem_ty) = match self.probe_ty_silent(base) {
                        Some(Ty::Struct(sid)) => {
                            let k = self.sorted_container_key(sid)?;
                            (SortedViewKind::Bag(sid), k.clone(), k)
                        }
                        Some(Ty::Set(k)) => (SortedViewKind::Set, (*k).clone(), *k),
                        Some(Ty::Map(k, v)) => {
                            let kt = (*k).clone();
                            let et = Ty::Tuple(vec![*k, *v]);
                            (SortedViewKind::Map, kt, et)
                        }
                        _ => return None,
                    };
                    let span = cur.span;
                    let raw_place = self.held_place(base, out);
                    let place = self.settle_indexes(raw_place, out);
                    let arg = &args[0].value;
                    let opt_k = Ty::opt(key_ty.clone());

                    let (lo_expr, lo_inc_expr, hi_expr, hi_inc_expr, empty_expr) = match &arg.kind {
                        ast::ExprKind::Range {
                            lo,
                            hi,
                            inclusive,
                            step: None,
                        } => {
                            let use_domain_ords = if let Ty::Pos(d) = key_ty {
                                let lo_is_bound = lo
                                    .as_deref()
                                    .and_then(|x| self.probe_ty_silent(x))
                                    .is_some_and(|t| t == Ty::Bound(d));
                                let hi_is_bound = hi
                                    .as_deref()
                                    .and_then(|x| self.probe_ty_silent(x))
                                    .is_some_and(|t| t == Ty::Bound(d));
                                lo_is_bound || hi_is_bound
                            } else {
                                false
                            };
                            if use_domain_ords {
                                let Ty::Pos(d) = key_ty else { unreachable!() };
                                let lo_ord = match lo.as_deref() {
                                    Some(l) => {
                                        let le = self.expr(l, None);
                                        self.cast(Cast::Ord, le, Ty::Int)
                                    }
                                    None => self.int_lit(0, span),
                                };
                                let hi_ord = match hi.as_deref() {
                                    Some(h) => {
                                        let he = self.expr(h, None);
                                        let is_pos = he.ty == Ty::Pos(d);
                                        let ho = self.cast(Cast::Ord, he, Ty::Int);
                                        if is_pos && *inclusive {
                                            self.int_add(ho, 1)
                                        } else {
                                            ho
                                        }
                                    }
                                    None => self.mk(ExprKind::DomainLen(d), Ty::Int, span),
                                };
                                self.sorted_domain_bounds(d, lo_ord, hi_ord, span, out)
                            } else {
                                let lo_e = match lo.as_deref() {
                                    Some(l) => {
                                        let lv = self.expect(l, &key_ty);
                                        self.mk(ExprKind::Some(Box::new(lv)), opt_k.clone(), span)
                                    }
                                    None => self.mk(ExprKind::None, opt_k.clone(), span),
                                };
                                let hi_e = match hi.as_deref() {
                                    Some(h) => {
                                        let hv = self.expect(h, &key_ty);
                                        self.mk(ExprKind::Some(Box::new(hv)), opt_k.clone(), span)
                                    }
                                    None => self.mk(ExprKind::None, opt_k.clone(), span),
                                };
                                (
                                    lo_e,
                                    self.mk(ExprKind::Bool(true), Ty::Bool, span),
                                    hi_e,
                                    self.mk(ExprKind::Bool(*inclusive), Ty::Bool, span),
                                    None,
                                )
                            }
                        }
                        _ => {
                            let want_r = Ty::Range(Box::new(key_ty.clone()));
                            let rv = self.expr(arg, Some(&want_r));
                            let r_slot = self.temp(rv.ty.clone());
                            out.push(Stmt::Let(r_slot, Some(rv)));
                            let r_l = self.local(r_slot, span);
                            let r_lo = self.mk(
                                ExprKind::Builtin(Builtin::RangeLo, vec![r_l.clone()]),
                                Ty::Int,
                                span,
                            );
                            let r_hi = self.mk(
                                ExprKind::Builtin(Builtin::RangeHi, vec![r_l]),
                                Ty::Int,
                                span,
                            );
                            if let Ty::Pos(d) = key_ty {
                                self.sorted_domain_bounds(d, r_lo, r_hi, span, out)
                            } else {
                                let r_lo = self.coerce(r_lo, &key_ty, span);
                                let r_hi = self.coerce(r_hi, &key_ty, span);
                                (
                                    self.mk(ExprKind::Some(Box::new(r_lo)), opt_k.clone(), span),
                                    self.mk(ExprKind::Bool(true), Ty::Bool, span),
                                    self.mk(ExprKind::Some(Box::new(r_hi)), opt_k.clone(), span),
                                    self.mk(ExprKind::Bool(false), Ty::Bool, span),
                                    None,
                                )
                            }
                        }
                    };

                    let lo_s = self.temp(opt_k.clone());
                    out.push(Stmt::Let(lo_s, Some(lo_expr)));
                    let lo_inc_s = self.temp(Ty::Bool);
                    out.push(Stmt::Let(lo_inc_s, Some(lo_inc_expr)));
                    let hi_s = self.temp(opt_k);
                    out.push(Stmt::Let(hi_s, Some(hi_expr)));
                    let hi_inc_s = self.temp(Ty::Bool);
                    out.push(Stmt::Let(hi_inc_s, Some(hi_inc_expr)));
                    let empty_s = empty_expr.map(|ee| {
                        let s = self.temp(Ty::Bool);
                        out.push(Stmt::Let(s, Some(ee)));
                        s
                    });
                    return Some(SortedView {
                        base: place,
                        kind,
                        key_ty,
                        elem_ty,
                        lo: lo_s,
                        lo_inc: lo_inc_s,
                        hi: hi_s,
                        hi_inc: hi_inc_s,
                        empty: empty_s,
                        rev: flip,
                    });
                }
                _ => return None,
            }
        }
    }

    pub(super) fn call_sorted_view(
        &mut self,
        view: &SortedView,
        sid: StructId,
        method: &str,
        ret_ty: Ty,
        empty_default: Expr,
        span: Span,
    ) -> Expr {
        let Some(fn_id) = self.struct_op_fn(sid, method) else {
            return empty_default;
        };
        let args = vec![
            Arg::Value(super::stmt::place_to_expr(&view.base)),
            Arg::Value(self.local(view.lo, span)),
            Arg::Value(self.local(view.lo_inc, span)),
            Arg::Value(self.local(view.hi, span)),
            Arg::Value(self.local(view.hi_inc, span)),
        ];
        let call = self.mk(ExprKind::Call(fn_id, args), ret_ty.clone(), span);
        if let Some(emp) = view.empty {
            let cond = self.local(emp, span);
            self.mk(
                ExprKind::If(Box::new(cond), Box::new(empty_default), Box::new(call)),
                ret_ty,
                span,
            )
        } else {
            call
        }
    }

    fn tree_view_key(&mut self, view: &SortedView, val: Expr, span: Span) -> Expr {
        match view.kind {
            SortedViewKind::Map => {
                self.mk(ExprKind::Field(Box::new(val), 0), view.key_ty.clone(), span)
            }
            _ => val,
        }
    }

    /// Filter an optional candidate `cand` against the upper bound (`hi`, `hi_inc`)
    /// when `check_hi`, or the lower bound (`lo`, `lo_inc`) when `!check_hi`.
    fn tree_view_clip(
        &mut self,
        view: &SortedView,
        cand: Expr,
        check_hi: bool,
        span: Span,
    ) -> Expr {
        let opt_e = Ty::opt(view.elem_ty.clone());
        let bound_slot = if check_hi { view.hi } else { view.lo };
        let inc_slot = if check_hi { view.hi_inc } else { view.lo_inc };
        self.do_expr(Some(&opt_e), span, |lw, out| {
            let cs = lw.temp(opt_e.clone());
            out.push(Stmt::Let(cs, Some(cand)));
            let c_ref = lw.local(cs, span);
            let c_some = lw.mk(ExprKind::IsSome(Box::new(c_ref.clone())), Ty::Bool, span);
            let b_ref = lw.local(bound_slot, span);
            let b_some = lw.mk(ExprKind::IsSome(Box::new(b_ref.clone())), Ty::Bool, span);
            let both = lw.mk(
                ExprKind::Binary(BinOp::And, Box::new(c_some), Box::new(b_some)),
                Ty::Bool,
                span,
            );
            let elem_val = lw.mk(
                ExprKind::Unwrap(Box::new(c_ref.clone()), false),
                view.elem_ty.clone(),
                span,
            );
            let k_val = lw.tree_view_key(view, elem_val, span);
            let b_val = lw.mk(
                ExprKind::Unwrap(Box::new(b_ref), false),
                view.key_ty.clone(),
                span,
            );
            let (strict_op, inc_op) = if check_hi {
                (CmpOp::Lt, CmpOp::Le)
            } else {
                (CmpOp::Gt, CmpOp::Ge)
            };
            let inc_cmp = lw.mk(
                ExprKind::Cmp(inc_op, Box::new(k_val.clone()), Box::new(b_val.clone())),
                Ty::Bool,
                span,
            );
            let strict_cmp = lw.mk(
                ExprKind::Cmp(strict_op, Box::new(k_val), Box::new(b_val)),
                Ty::Bool,
                span,
            );
            let in_bounds = lw.mk(
                ExprKind::If(
                    Box::new(lw.local(inc_slot, span)),
                    Box::new(inc_cmp),
                    Box::new(strict_cmp),
                ),
                Ty::Bool,
                span,
            );
            let keep = lw.mk(
                ExprKind::If(
                    Box::new(in_bounds),
                    Box::new(c_ref.clone()),
                    Box::new(lw.mk(ExprKind::None, opt_e.clone(), span)),
                ),
                opt_e.clone(),
                span,
            );
            let res = lw.mk(
                ExprKind::If(Box::new(both), Box::new(keep), Box::new(c_ref)),
                opt_e.clone(),
                span,
            );
            out.push(Stmt::Give(res));
        })
    }

    fn tree_view_end(&mut self, view: &SortedView, want_last: bool, span: Span) -> Expr {
        let opt_e = Ty::opt(view.elem_ty.clone());
        let none_e = self.mk(ExprKind::None, opt_e.clone(), span);
        let body = self.do_expr(Some(&opt_e), span, |lw, out| {
            let base_e = super::stmt::place_to_expr(&view.base);
            let (start_slot, start_inc, end_op, ge_op, gt_op, check_hi) = if want_last {
                (
                    view.hi,
                    view.hi_inc,
                    Builtin::SetMax,
                    Builtin::LastLe,
                    Builtin::LastLt,
                    false,
                )
            } else {
                (
                    view.lo,
                    view.lo_inc,
                    Builtin::SetMin,
                    Builtin::FirstGe,
                    Builtin::FirstGt,
                    true,
                )
            };
            let s_ref = lw.local(start_slot, span);
            let s_some = lw.mk(ExprKind::IsSome(Box::new(s_ref.clone())), Ty::Bool, span);
            let s_val = lw.mk(
                ExprKind::Unwrap(Box::new(s_ref), false),
                view.key_ty.clone(),
                span,
            );
            let call_ge = lw.mk(
                ExprKind::Builtin(ge_op, vec![base_e.clone(), s_val.clone()]),
                opt_e.clone(),
                span,
            );
            let call_gt = lw.mk(
                ExprKind::Builtin(gt_op, vec![base_e.clone(), s_val]),
                opt_e.clone(),
                span,
            );
            let bounded = lw.mk(
                ExprKind::If(
                    Box::new(lw.local(start_inc, span)),
                    Box::new(call_ge),
                    Box::new(call_gt),
                ),
                opt_e.clone(),
                span,
            );
            let open = lw.mk(ExprKind::Builtin(end_op, vec![base_e]), opt_e.clone(), span);
            let cand = lw.mk(
                ExprKind::If(Box::new(s_some), Box::new(bounded), Box::new(open)),
                opt_e.clone(),
                span,
            );
            let clipped = lw.tree_view_clip(view, cand, check_hi, span);
            out.push(Stmt::Give(clipped));
        });
        if let Some(emp) = view.empty {
            self.mk(
                ExprKind::If(
                    Box::new(self.local(emp, span)),
                    Box::new(none_e),
                    Box::new(body),
                ),
                opt_e,
                span,
            )
        } else {
            body
        }
    }

    fn tree_view_collect(&mut self, view: &SortedView, span: Span) -> Expr {
        let list_ty = Ty::List(Box::new(view.elem_ty.clone()));
        let opt_e = Ty::opt(view.elem_ty.clone());
        self.do_expr(Some(&list_ty), span, |lw, out| {
            let out_s = lw.temp(list_ty.clone());
            out.push(Stmt::Let(
                out_s,
                Some(lw.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span)),
            ));
            let cur_s = lw.temp(opt_e.clone());
            let first = lw.tree_view_end(view, view.rev, span);
            out.push(Stmt::Let(cur_s, Some(first)));
            let cur_ref = lw.local(cur_s, span);
            let cond = lw.mk(ExprKind::IsSome(Box::new(cur_ref.clone())), Ty::Bool, span);
            let v_s = lw.temp(view.elem_ty.clone());
            let v_init = lw.mk(
                ExprKind::Unwrap(Box::new(cur_ref), false),
                view.elem_ty.clone(),
                span,
            );
            let v_ref = lw.local(v_s, span);
            let push_stmt = Stmt::Expr(lw.mk(
                ExprKind::Mutate(
                    Mutation::Push,
                    Box::new(Place {
                        kind: PlaceKind::Local(out_s),
                        ty: list_ty.clone(),
                        span: lw.span(span),
                    }),
                    vec![v_ref.clone()],
                ),
                Ty::Unit,
                span,
            ));
            let k_val = lw.tree_view_key(view, v_ref, span);
            let step_op = if view.rev {
                Builtin::LastLt
            } else {
                Builtin::FirstGt
            };
            let next_raw = lw.mk(
                ExprKind::Builtin(step_op, vec![super::stmt::place_to_expr(&view.base), k_val]),
                opt_e.clone(),
                span,
            );
            let next_clipped = lw.tree_view_clip(view, next_raw, !view.rev, span);
            let step_stmt = Stmt::Assign(
                Place {
                    kind: PlaceKind::Local(cur_s),
                    ty: opt_e.clone(),
                    span: lw.span(span),
                },
                AssignOp::Set,
                next_clipped,
            );
            let lid = lw.next_loop;
            lw.next_loop += 1;
            out.push(Stmt::While(
                lid,
                cond,
                Block {
                    stmts: vec![Stmt::Let(v_s, Some(v_init)), push_stmt, step_stmt],
                },
            ));
            out.push(Stmt::Give(lw.local(out_s, span)));
        })
    }

    fn sorted_view_field(&mut self, base: &ast::Expr, field: &str, span: Span) -> Expr {
        let mut res_ty = Ty::Error;
        let mut e = self.do_expr(None, span, |lw, out| {
            let Some(view) = lw.sorted_view_from_expr(base, out) else {
                out.push(Stmt::Give(lw.err_expr(span)));
                return;
            };
            let opt_e = Ty::opt(view.elem_ty.clone());
            let res = match view.kind {
                SortedViewKind::Bag(sid) => match field {
                    "first" => {
                        let m = if view.rev {
                            "range_last"
                        } else {
                            "range_first"
                        };
                        let d = lw.mk(ExprKind::None, opt_e.clone(), span);
                        lw.call_sorted_view(&view, sid, m, opt_e, d, span)
                    }
                    "last" => {
                        let m = if view.rev {
                            "range_first"
                        } else {
                            "range_last"
                        };
                        let d = lw.mk(ExprKind::None, opt_e.clone(), span);
                        lw.call_sorted_view(&view, sid, m, opt_e, d, span)
                    }
                    "len" => {
                        let d = lw.int_lit(0, span);
                        lw.call_sorted_view(&view, sid, "range_len", Ty::Int, d, span)
                    }
                    "is_empty" => {
                        let d = lw.int_lit(0, span);
                        let len = lw.call_sorted_view(&view, sid, "range_len", Ty::Int, d, span);
                        let zero = lw.int_lit(0, span);
                        lw.mk(
                            ExprKind::Cmp(CmpOp::Eq, Box::new(len), Box::new(zero)),
                            Ty::Bool,
                            span,
                        )
                    }
                    _ => lw.err_expr(span),
                },
                SortedViewKind::Set | SortedViewKind::Map => match field {
                    "first" => lw.tree_view_end(&view, view.rev, span),
                    "last" => lw.tree_view_end(&view, !view.rev, span),
                    "is_empty" => {
                        let first = lw.tree_view_end(&view, false, span);
                        let some = lw.mk(ExprKind::IsSome(Box::new(first)), Ty::Bool, span);
                        lw.mk(ExprKind::Unary(UnOp::Not, Box::new(some)), Ty::Bool, span)
                    }
                    _ => lw.err_expr(span),
                },
            };
            res_ty = res.ty.clone();
            out.push(Stmt::Give(res));
        });
        e.ty = res_ty;
        e
    }

    pub(super) fn sorted_view_iter(&mut self, e: &ast::Expr, span: Span) -> Option<(Iter, Ty)> {
        if !self.is_sorted_range_expr(e) {
            return None;
        }
        // Zero-allocation direct iterator walk for `Set.range(r)` / `Map.range(r)`
        // when `r` is a syntactic range literal (`a..b`, `a..=b`, `a..`, `..b`, `..=b`, `..`).
        let mut cur = e;
        let mut rev = false;
        while let ast::ExprKind::Field { base, name } = &cur.kind {
            if name.text == "rev" {
                rev = !rev;
                cur = base;
            } else {
                break;
            }
        }
        if let ast::ExprKind::Call { callee, args } = &cur.kind {
            if let ast::ExprKind::Field { base, name } = &callee.kind {
                if name.text == "range" && args.len() == 1 {
                    let base_ty = self.probe_ty_silent(base);
                    if let Some((key_ty, elem_ty)) = match base_ty {
                        Some(Ty::Set(k)) => Some(((*k).clone(), *k)),
                        Some(Ty::Map(k, v)) => {
                            let kt = *k;
                            let et = Ty::Tuple(vec![kt.clone(), *v]);
                            Some((kt, et))
                        }
                        _ => None,
                    } {
                        if let ast::ExprKind::Range {
                            lo, hi, inclusive, ..
                        } = &args[0].value.kind
                        {
                            let container = self.expr(base, None);
                            let lo = lo.as_ref().map(|x| (self.expect(x, &key_ty), true));
                            let hi = hi.as_ref().map(|x| (self.expect(x, &key_ty), *inclusive));
                            return Some((
                                Iter::TreeRange {
                                    container,
                                    lo,
                                    hi,
                                    rev,
                                },
                                elem_ty,
                            ));
                        }
                    }
                }
            }
        }
        let mut elem_ty = Ty::Error;
        let mut seq_ty = Ty::Error;
        let mut seq = self.do_expr(None, span, |lw, out| {
            let Some(view) = lw.sorted_view_from_expr(e, out) else {
                out.push(Stmt::Give(lw.err_expr(span)));
                return;
            };
            elem_ty = view.elem_ty.clone();
            let list_ty = Ty::List(Box::new(view.elem_ty.clone()));
            let res = match view.kind {
                SortedViewKind::Bag(sid) => {
                    let m = if view.rev {
                        "range_each_rev"
                    } else {
                        "range_each"
                    };
                    let empty_list = lw.mk(ExprKind::Tuple(Vec::new()), list_ty.clone(), span);
                    lw.call_sorted_view(&view, sid, m, list_ty.clone(), empty_list, span)
                }
                SortedViewKind::Set | SortedViewKind::Map => lw.tree_view_collect(&view, span),
            };
            seq_ty = list_ty;
            out.push(Stmt::Give(res));
        });
        seq.ty = seq_ty;
        Some((Iter::Seq(seq, false), elem_ty))
    }
}
