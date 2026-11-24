//! Keyed sequences: `keyed()`, and `map` over a view, keep the keys they
//! are read at (R04a.9.6, R12.4.2).

use super::*;

impl<'a> Lowerer<'a> {
    /// The element of a mapped sequence's source that `m[indices]` reads:
    /// a view is read by its source keys (R04a.9.4).
    pub(super) fn mapped_elem(&mut self, base: Expr, indices: &[ast::Expr], span: Span) -> Expr {
        let base = if Self::keyed_view(&base.ty) && indices.len() == 1 {
            self.hold_view(base, span)
        } else {
            base
        };
        let (mut idx_exprs, elem_ty) = self.index_args(&base.ty, indices, span);
        if matches!(elem_ty, Ty::Error) {
            return self.err_expr(span);
        }
        if Self::keyed_view(&base.ty) && idx_exprs.len() == 1 {
            let key = idx_exprs.remove(0);
            return self.key_index(base, key, elem_ty, span);
        }
        self.mk_index(base, idx_exprs, elem_ty, span)
    }

    /// `s.keyed()` / `s.keyed(keys)` on a list or view: the same cells, keyed
    /// in walk order from the first key of `keys` (`0..` by default), O(1)
    /// (R12.4.2, R04a.9.6). `None` leaves `by:` and other sequences to the
    /// std definition.
    pub(super) fn lower_keyed(
        &mut self,
        base: &ast::Expr,
        args: &[ast::Arg],
        span: Span,
    ) -> Option<Expr> {
        let keys = match args {
            [] => None,
            [arg] if !arg.mutable && arg.name.as_ref().is_none_or(|n| n.text == "keys") => {
                Some(&arg.value)
            }
            _ => return None,
        };
        let elem = match self.probe_ty_silent(base)? {
            Ty::List(elem) | Ty::ListView(elem, _) | Ty::Keyed(elem, _) | Ty::MutView(elem, _) => {
                *elem
            }
            _ => return None,
        };
        let dim = match keys {
            None => Dim::Int,
            Some(keys) => match self.probe_ty_silent(keys)? {
                Ty::Range(k) => match *k {
                    Ty::Pos(d) | Ty::Bound(d) => Dim::Pos(d),
                    Ty::Int => Dim::Int,
                    _ => return None,
                },
                _ => return None,
            },
        };
        let mut seq = self.expr(base, None);
        // A reversed view keys its cells from the far end; the counter follows the walk (R04a.9.6).
        if matches!(seq.kind, ExprKind::Builtin(Builtin::Reversed, _)) {
            seq = self.mk_builtin(Builtin::ToList, [seq], Ty::list(elem.clone()), span);
        }
        let res = if matches!(seq.ty, Ty::List(_)) {
            Ty::Keyed(Box::new(elem), dim)
        } else {
            Ty::ListView(Box::new(elem), dim)
        };
        // A view must not outlive its cells, so a temporary gets a slot.
        let seq = if matches!(seq.kind, ExprKind::Local(_) | ExprKind::Global(_)) {
            seq
        } else {
            let slot = self.new_slot("cells", seq.ty.clone());
            self.cx.pending.push_let(slot, seq);
            self.local(slot, span)
        };
        let lo = match keys {
            None => self.int_lit(0, span),
            Some(keys) => {
                let keys = self.expr(keys, None);
                let keys = self.hold_view(keys, span);
                self.mk_builtin(Builtin::RangeLo, [keys], Ty::Int, span)
            }
        };
        let lo = self.hold_view(lo, span);
        let len = self.len_of(seq.clone(), span);
        let hi = self.int_sum(lo.clone(), len);
        Some(self.mk_builtin(Builtin::Rekey, [seq, lo, hi], res, span))
    }

    /// Whether `x[k]` names a key: a view keeps its source's keys (R04b.1),
    /// and a range's keys are its own elements (R04.4.31).
    pub(super) fn keyed_index(ty: &Ty) -> bool {
        Self::keyed_view(ty) || matches!(ty, Ty::Range(of) if **of != Ty::I128)
    }

    /// The domain `rest` that `prefix` holds: `D` of `let xs = compress(all)`, `t.Idx` of a rebase.
    pub(super) fn domain_of_value(&mut self, prefix: &str, rest: &str) -> Option<DomainId> {
        let owner = ast::Expr {
            kind: ast::ExprKind::Name(prefix.to_string()),
            span: Span::point(0),
        };
        let ty = self.probe_ty_silent(&owner)?;
        let Ty::Struct(sid) = ty else {
            return None;
        };
        if self.per_value_domain(sid, rest) {
            let value = self.expr(&owner, None);
            return self.value_domain(value, rest, owner.span);
        }
        match self.structs[sid].env.get(rest) {
            Some(GenArg::Domain(id)) | Some(GenArg::Ty(Ty::Pos(id))) => Some(*id),
            _ => self.structs[sid].owned_domains.get(rest).copied(),
        }
    }

    /// `owner.member` for a named struct value whose domain `member` is
    /// sized from its fields.
    pub(super) fn value_domain_named(
        &mut self,
        owner: &ast::Expr,
        member: &str,
    ) -> Option<DomainId> {
        if !matches!(owner.kind, ast::ExprKind::Name(_)) {
            return None;
        }
        let Ty::Struct(sid) = self.probe_ty_silent(owner)? else {
            return None;
        };
        if !self.per_value_domain(sid, member) {
            return None;
        }
        let value = self.expr(owner, None);
        self.value_domain(value, member, owner.span)
    }

    /// A position of `w.Idx` where the struct's own `Idx` is wanted, or back:
    /// the checker already kept two values' domains apart (E0301).
    pub(super) fn same_value_domain(&self, want: &Ty, got: &Ty) -> bool {
        let shared = |d: DomainId| {
            self.value_domain_ids
                .iter()
                .find(|(_, &copy)| copy == d)
                .map_or(d, |((_, owned), _)| *owned)
        };
        matches!((want, got), (Ty::Pos(w), Ty::Pos(g)) if shared(*w) == shared(*g))
    }

    /// Whether the domain `member` of struct `sid` is sized from its fields.
    pub(super) fn per_value_domain(&self, sid: StructId, member: &str) -> bool {
        let Some(&shared) = self.structs[sid].owned_domains.get(member) else {
            return false;
        };
        self.structs[sid]
            .owned_domain_inits
            .iter()
            .any(|i| i.id == shared && i.per_value)
    }

    /// `w.Idx` for a struct value `w` whose domain `Idx` is sized from its
    /// fields: a domain of `w`'s own, set from `w` each time it is named
    /// (R04b.1.7).
    pub(super) fn value_domain(
        &mut self,
        owner: Expr,
        member: &str,
        span: Span,
    ) -> Option<DomainId> {
        let Ty::Struct(sid) = owner.ty else {
            return None;
        };
        let shared = *self.structs[sid].owned_domains.get(member)?;
        let init = self.structs[sid]
            .owned_domain_inits
            .iter()
            .find(|i| i.id == shared && i.per_value)?
            .clone();
        let (key, kind) = match owner.kind {
            ExprKind::Local(slot) => ((false, slot), PlaceKind::Local(slot)),
            ExprKind::Global(id) => ((true, id), PlaceKind::Global(id)),
            _ => {
                let slot = self.temp(owner.ty.clone());
                self.cx.pending.push_let(slot, owner.clone());
                ((false, slot), PlaceKind::Local(slot))
            }
        };
        let id = match self.value_domain_ids.get(&(key, shared)) {
            Some(&id) => id,
            None => {
                let id = self.domains.len();
                let name = format!("{}_v{id}", self.domains[shared]);
                self.domains.push(name);
                self.value_domain_ids.insert((key, shared), id);
                id
            }
        };
        let place = Place {
            kind,
            ty: owner.ty.clone(),
            span,
        };
        // The fields are the only names the definition sees besides globals.
        let scopes = std::mem::take(&mut self.cx.scopes);
        let inline = self.cx.inline_self.replace((place, sid));
        let module = std::mem::replace(&mut self.cur_mod, init.mod_id);
        let offset = std::mem::replace(&mut self.offset, init.offset);
        let mut out = Vec::new();
        self.init_domain_id(id, init.def, &mut out);
        self.cur_mod = module;
        self.offset = offset;
        self.cx.inline_self = inline;
        self.cx.scopes = scopes;
        self.cx.pending.extend(out);
        Some(id)
    }

    /// A method's receiver sets the domains its struct sizes from fields, so
    /// the call's keys are checked against that value's: `mk()[1]`.
    pub(super) fn receiver_domains(&mut self, value: Expr, span: Span) -> Expr {
        let Ty::Struct(sid) = value.ty else {
            return value;
        };
        let inits: Vec<_> = self.structs[sid]
            .owned_domain_inits
            .iter()
            .filter(|i| i.per_value)
            .cloned()
            .collect();
        if inits.is_empty() {
            return value;
        }
        let (kind, value) = match value.kind {
            ExprKind::Local(slot) => (PlaceKind::Local(slot), value),
            ExprKind::Global(id) => (PlaceKind::Global(id), value),
            _ => {
                let slot = self.temp(value.ty.clone());
                self.cx.pending.push_let(slot, value);
                (PlaceKind::Local(slot), self.local(slot, span))
            }
        };
        let place = Place {
            kind,
            ty: value.ty.clone(),
            span,
        };
        let scopes = std::mem::take(&mut self.cx.scopes);
        let inline = self.cx.inline_self.replace((place, sid));
        let mut out = Vec::new();
        for init in inits {
            let module = std::mem::replace(&mut self.cur_mod, init.mod_id);
            let offset = std::mem::replace(&mut self.offset, init.offset);
            self.init_domain_id(init.id, init.def, &mut out);
            self.cur_mod = module;
            self.offset = offset;
        }
        self.cx.inline_self = inline;
        self.cx.scopes = scopes;
        self.cx.pending.extend(out);
        value
    }

    /// `seq.map(f)` as a list, keyed like `seq` when `seq` is a view:
    /// `a[1..3].map(f)[2]` is `f(a[2])` (R04a.9.4).
    pub(super) fn map_seq(
        &mut self,
        seq: Expr,
        elem_ty: Ty,
        func_ast: &ast::Expr,
        span: Span,
    ) -> Expr {
        let dim = match &seq.ty {
            Ty::ListView(_, dim) | Ty::Keyed(_, dim) => *dim,
            Ty::StrView => Dim::Int,
            _ => return self.lower_map_iter(Iter::Seq(seq, false), elem_ty, func_ast, span),
        };
        let seq = self.hold_view(seq, span);
        let lo = self.key_lo(&seq, span);
        let len = self.len_of(seq.clone(), span);
        let hi = self.int_sum(lo.clone(), len);
        let mapped = self.lower_map_iter(Iter::Seq(seq, false), elem_ty, func_ast, span);
        let Ty::List(out_elem) = mapped.ty.clone() else {
            return mapped;
        };
        let cells = self.new_slot("cells", mapped.ty.clone());
        self.cx.pending.push_let(cells, mapped);
        let cells = self.local(cells, span);
        self.mk_builtin(
            Builtin::Rekey,
            [cells, lo, hi],
            Ty::Keyed(out_elem, dim),
            span,
        )
    }

    /// The key of one row or column of a grid: the axis it runs along,
    /// when that axis is a domain (R04b.1.12).
    pub(super) fn axis_key(&self, whole: &[usize], dims: &[Dim]) -> Option<Dim> {
        let [axis] = whole else { return None };
        Some(dims[*axis]).filter(|d| !matches!(d, Dim::Int | Dim::IntFrom(_) | Dim::Sym(_)))
    }

    /// `g[r]` or `g[.., c]` on a 2D grid whose axes are domains: a view of the grid's
    /// cells, or a mutable one when the binding is written through, as a slice's is
    /// (R05.3.3). `None` for any other piece, which is collected.
    pub(super) fn grid_line(
        &mut self,
        base: &ast::Expr,
        grid: &Expr,
        indices: &[ast::Expr],
        whole: &[usize],
        span: Span,
    ) -> Option<Expr> {
        let Ty::Array { dims, elem } = &grid.ty else {
            return None;
        };
        let (&[axis], &[d0, d1]) = (whole, dims.as_slice()) else {
            return None;
        };
        let elem: &Ty = elem;
        let dim = self.axis_key(whole, dims)?;
        let ast::ExprKind::Range {
            lo: None,
            hi: None,
            step: None,
            ..
        } = &indices[axis].kind
        else {
            return None;
        };
        let fixed = &indices[1 - axis];
        let want = match [d0, d1][1 - axis] {
            Dim::Pos(d) => Ty::Pos(d),
            Dim::Bound(d) => Ty::Bound(d),
            _ => return None,
        };
        // The grid is lowered before the index, as `g[r]` reads them.
        let grid = if self.mut_slice_inits.contains(&self.span(span)) && self.is_mutable_place(base)
        {
            Err(self.place(base))
        } else {
            Ok(self.hoist_view_base(grid.clone(), span))
        };
        let at = self.expr(fixed, Some(&want));
        let at = self.coerce(at, &want, fixed.span);
        let at = self.cast(Cast::Ord, at, Ty::Int);
        Some(match grid {
            Err(place) => self.mk(
                ExprKind::MutLine(Box::new(place), Box::new(at), axis),
                Ty::MutView(Box::new(elem.clone()), dim),
                span,
            ),
            Ok(grid) => {
                let axis = self.int_lit(axis as i64, span);
                self.mk_builtin(
                    Builtin::GridLine,
                    [grid, at, axis],
                    Ty::Keyed(Box::new(elem.clone()), dim),
                    span,
                )
            }
        })
    }

    /// The cells of one row or column of a grid, collected into `cells`,
    /// keyed by [`Self::axis_key`] from the position `first` holds: a row
    /// keeps the column domain.
    pub(super) fn keyed_axis(
        &mut self,
        cells: Expr,
        whole: &[usize],
        dims: &[Dim],
        first: Option<Slot>,
        span: Span,
    ) -> Expr {
        let (Some(dim), Some(first), Ty::List(elem)) =
            (self.axis_key(whole, dims), first, cells.ty.clone())
        else {
            return cells;
        };
        let ty = Ty::Keyed(elem, dim);
        self.do_expr(Some(&ty.clone()), span, |lw, out| {
            out.insert(0, Stmt::Let(first, None));
            let held = lw.temp(cells.ty.clone());
            out.push_let(held, cells);
            let (cells, lo) = (lw.local(held, span), lw.local(first, span));
            let len = lw.len_of(cells.clone(), span);
            let hi = lw.int_sum(lo.clone(), len);
            let keyed = lw.mk_builtin(Builtin::Rekey, [cells, lo, hi], ty, span);
            out.push_give(keyed);
        })
    }

    /// `first = value` on a local.
    pub(super) fn set_local(&mut self, first: Slot, value: Expr, span: Span) -> Stmt {
        let ty = self.cx.locals[first].ty.clone();
        let place = Place {
            kind: PlaceKind::Local(first),
            ty,
            span: self.span(span),
        };
        Stmt::Assign(place, AssignOp::Set, value)
    }

    /// `s.keys` on a list, view or string: its keys in walk order, `lo..lo + len`.
    pub(super) fn seq_keys(&mut self, value: Expr, span: Span) -> Expr {
        let value = if Self::keyed_view(&value.ty) {
            self.hold_view(value, span)
        } else {
            value
        };
        let lo = self.view_first_key(&value, span);
        let key = value.ty.key_ty();
        let len = self.len_of(value, span);
        let hi = self.mk_binary(BinOp::Add, lo.clone(), len, Ty::Int, span);
        let (lo, hi) = (Box::new(lo), Box::new(hi));
        self.mk(ExprKind::Range { lo, hi }, Ty::range(key), span)
    }

    /// `a.rev` over a list or view, whose `.keys` are `a`'s walked backwards (R07.4.7).
    pub(super) fn reversed_list(value: &Expr) -> bool {
        matches!(&value.kind, ExprKind::Builtin(Builtin::Reversed, args)
            if matches!(&args[0].ty, Ty::List(_) | Ty::ListView(..) | Ty::MutView(..))
                || matches!(&args[0].ty, Ty::Array { dims, .. } if dims.len() == 1))
    }

    /// `value` walks backwards: `a.rev` over a list or view that walks
    /// forwards, or a name bound to one.
    pub(super) fn is_reversed(&self, value: &Expr) -> bool {
        match &value.kind {
            ExprKind::Builtin(Builtin::Reversed, args) if Self::reversed_list(value) => {
                !self.is_reversed(&args[0])
            }
            _ => self.named_reversed(value),
        }
    }

    /// The sequence the reversed `value` walks backwards.
    pub(super) fn reversed_source(&mut self, value: Expr, span: Span) -> Expr {
        match value.kind {
            ExprKind::Builtin(Builtin::Reversed, mut args) if !self.is_reversed(&args[0]) => {
                args.remove(0)
            }
            _ => {
                let ty = value.ty.clone();
                self.mk_builtin(Builtin::Reversed, [value], ty, span)
            }
        }
    }

    pub(super) fn reversed_keys(&mut self, value: Expr, span: Span) -> Expr {
        let value = match value.kind {
            ExprKind::Builtin(Builtin::Reversed, mut args) => args.remove(0),
            _ => value,
        };
        let key = value.ty.key_ty();
        let keys = self.seq_keys(value, span);
        self.reversed_range(keys, key, span)
    }

    /// `s.filter(p)` written as a call: its source and its test.
    pub(super) fn filter_call(e: &ast::Expr) -> Option<(&ast::Expr, &ast::Expr)> {
        let ast::ExprKind::Call { callee, args } = &e.kind else {
            return None;
        };
        let ast::ExprKind::Field { base, name } = &callee.kind else {
            return None;
        };
        match args.as_slice() {
            [test] if name.text == "filter" && test.name.is_none() => Some((base, &test.value)),
            _ => None,
        }
    }

    /// `s.filter(p).items` / `.keys`: keys survive `filter` (R07.3.2), so this walks
    /// `s.items` and keeps each pair whose element passes, rather than counting
    /// the kept elements from 0.
    pub(super) fn filtered_items(
        &mut self,
        base: &ast::Expr,
        name: &ast::Name,
        span: Span,
    ) -> Option<Expr> {
        let keys = match name.text.as_str() {
            "items" => false,
            "keys" => true,
            _ => return None,
        };
        let (src, test) = Self::filter_call(base)?;
        if !matches!(
            self.checked_ty(src),
            Some(
                Ty::List(_) | Ty::ListView(..) | Ty::Keyed(..) | Ty::MutView(..) | Ty::Array { .. }
            ) | Some(Ty::Str | Ty::StrView)
        ) {
            return None;
        }
        let items = ast::Expr {
            kind: ast::ExprKind::Field {
                base: Box::new(src.clone()),
                name: ast::Name {
                    text: "items".into(),
                    span: name.span,
                },
            },
            span: src.span,
        };
        let (iter, pair_ty) = self.iter(&items);
        let Ty::Tuple(parts) = &pair_ty else {
            return Some(self.err_expr(span));
        };
        let (key_ty, elem_ty) = (parts[0].clone(), parts[1].clone());
        let pred = self.expr_as(test, &Ty::Fn(vec![elem_ty.clone()], Box::new(Ty::Bool)));
        let out_ty = if keys {
            key_ty.clone()
        } else {
            pair_ty.clone()
        };
        Some(self.collected(out_ty, span, |lw, out, push| {
            let pred = lw.hold(pred, out);
            let item = lw.temp(pair_ty);
            let lid = lw.fresh_loop();
            let pair = lw.local(item, span);
            let elem = lw.mk_field(pair.clone(), 1, elem_ty, span);
            let call = ExprKind::CallExpr(Box::new(pred), vec![Arg::Value(elem)]);
            let keep = lw.mk(call, Ty::Bool, span);
            let kept = if keys {
                lw.mk_field(pair, 0, key_ty, span)
            } else {
                pair
            };
            let stmts = vec![push(lw, kept)];
            let body = vec![Stmt::If(keep, Block { stmts }, Block::default())];
            out.push(Stmt::For(lid, item, iter, Block { stmts: body }));
        }))
    }
}

/// A domain a struct body declares, with where its definition is read.
#[derive(Clone)]
pub(super) struct OwnedDomainInit<'a> {
    pub id: DomainId,
    pub def: &'a ast::Expr,
    pub offset: u32,
    pub mod_id: ModId,
    /// Sized from the fields (`domain Idx = 0..src.len`): each value has its own.
    pub per_value: bool,
}
