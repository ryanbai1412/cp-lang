//! Whole-row assignments: `g[r, ..] max= g[v, ..]`, `a[l..r] = b[..] + x`,
//! `a[l..r] += x`, a cell at a time.

use super::*;

impl<'a> Lowerer<'a> {
    /// True for an array of one dimension, or a list, of values with an
    /// order: what `a max= b` goes through cell by cell.
    pub(in crate::lower) fn is_row(&self, ty: &Ty) -> bool {
        match ty {
            Ty::Array { dims, elem } if dims.len() == 1 => {
                !matches!(dims[0], Dim::Bound(_)) && self.ordered(elem)
            }
            Ty::List(elem) => self.ordered(elem),
            _ => false,
        }
    }

    /// `g[r, ..]`, `g[.., c]`, `g[r, lo..hi]`, or `a[..]` and `a[lo..hi]`
    /// of an array of one index: the array, its indexes and the axis the
    /// range runs along.
    pub(in crate::lower) fn grid_row<'e>(
        &mut self,
        e: &'e ast::Expr,
    ) -> Option<(&'e ast::Expr, &'e [ast::Expr], usize)> {
        let ast::ExprKind::Index { base, indices } = &e.kind else {
            return None;
        };
        let ranges: Vec<usize> = (0..indices.len())
            .filter(|&k| crate::lower::expr::is_whole(&indices[k]))
            .collect();
        let [axis] = ranges.as_slice() else {
            return None;
        };
        match self.probe_ty_silent(base)? {
            Ty::Array { dims, .. } if dims.len() == indices.len() => Some((base, indices, *axis)),
            Ty::List(_) if indices.len() == 1 => Some((base, indices, *axis)),
            _ => None,
        }
    }

    /// The axes and the cell type of what [`Self::grid_row`] found: a list
    /// is one axis counted from 0.
    pub(in crate::lower) fn axes_of(ty: &Ty) -> Option<(Vec<Dim>, Ty)> {
        match ty {
            Ty::Array { dims, elem } => Some((dims.clone(), (**elem).clone())),
            Ty::List(elem) => Some((vec![Dim::Int], (**elem).clone())),
            _ => None,
        }
    }

    /// The indexes of the cell the loop is at, `at` standing where the
    /// range was written.
    pub(in crate::lower) fn axis_cell(
        &mut self,
        rows: &Ty,
        indices: &[ast::Expr],
        at: Slot,
        span: Span,
    ) -> Vec<Expr> {
        if matches!(rows, Ty::List(_)) {
            return vec![self.local(at, span)];
        }
        self.whole_axes = vec![at];
        let (cell, _) = self.index_args(rows, indices, span);
        self.whole_axes.clear();
        cell
    }

    /// How many cells `rows` has along `axis`.
    pub(in crate::lower) fn axis_len(&mut self, rows: Expr, axis: usize, span: Span) -> Expr {
        let one = match &rows.ty {
            Ty::Array { dims, .. } => dims.len() == 1,
            _ => true,
        };
        if one {
            return self.len_of(rows, span);
        }
        let axis = self.int_lit(axis as i64, span);
        self.mk_builtin(Builtin::AxisLen, [rows, axis], Ty::Int, span)
    }

    /// `g[r, ..] max= g[v, ..]`: every cell of one row or column of a grid
    /// takes the larger of the two, where it is, in one loop. The other
    /// side is a row of a grid too, or any row of the same cells. When
    /// `asked`, gives the slot that says whether any cell changed.
    pub(in crate::lower) fn grid_keep(
        &mut self,
        target: &ast::Expr,
        max: Option<bool>,
        value: &ast::Expr,
        asked: bool,
        out: &mut Vec<Stmt>,
    ) -> Option<Slot> {
        let span = target.span;
        let word = match max {
            Some(true) => "max=",
            Some(false) => "min=",
            None => "=",
        };
        let (base, indices, axis) = self.grid_row(target)?;
        let place = self.place(base);
        let place = self.settle_indexes(place, out);
        if let PlaceKind::Local(s) = root_place(&place) {
            self.cx.mut_slots.insert(s);
        }
        let (dims, elem) = Self::axes_of(&place.ty)?;
        if max.is_some() && !self.ordered(&elem) {
            let message = format!(
                "`{word}` needs cells that have an order, and these are `{}`",
                self.show(&elem)
            );
            self.error(codes::TYPE_MISMATCH, span, message);
            return None;
        }
        let slot_ty = |dim: Dim| match dim {
            Dim::Pos(d) => Ty::Pos(d),
            Dim::Bound(d) => Ty::Bound(d),
            Dim::Enum(e) => Ty::Enum(e),
            Dim::Sym(_) | Dim::Int | Dim::IntFrom(_) => Ty::Int,
        };
        let changed = asked.then(|| {
            let changed = self.temp(Ty::Bool);
            out.push(Stmt::Let(
                changed,
                Some(self.mk(ExprKind::Bool(false), Ty::Bool, span)),
            ));
            changed
        });
        // The cell of the target the loop is at.
        let at = self.temp(slot_ty(dims[axis]));
        let cell = self.axis_cell(&place.ty, indices, at, span);
        let cell: Vec<Expr> = cell
            .into_iter()
            .enumerate()
            .map(|(k, index)| {
                if k == axis {
                    index
                } else {
                    self.hold(index, out)
                }
            })
            .collect();
        let grid = place_to_expr(&place);
        let len = self.axis_len(grid, axis, span);
        let (lo, hi) = self.axis_part(&indices[axis], dims[axis], len, out);
        let (lo, hi) = (self.hold(lo, out), self.hold(hi, out));
        let count = self.mk_binary(BinOp::Sub, hi.clone(), lo.clone(), Ty::Int, span);
        let cell = Place {
            kind: PlaceKind::Index(Box::new(place), cell),
            ty: elem.clone(),
            span: self.span(span),
        };
        // The cell of the other side that goes with it.
        let mut body = Vec::new();
        let counter = self.cast(Cast::Trust, self.local(at, span), Ty::Int);
        let offset = self.mk_binary(BinOp::Sub, counter, lo.clone(), Ty::Int, span);
        let (theirs, their_count) = match self.grid_row(value) {
            Some((other, their_indices, their_axis)) => {
                let other = self.expr(other, None);
                let other = self.hold(other, out);
                let Some((their_dims, their_elem)) = Self::axes_of(&other.ty) else {
                    return changed;
                };
                if their_elem != elem {
                    let message = format!(
                        "`{word}` goes cell by cell: these are `{}`, and the other side has `{}`",
                        self.show(&elem),
                        self.show(&their_elem)
                    );
                    self.error(codes::TYPE_MISMATCH, value.span, message);
                    return changed;
                }
                let their_at = self.temp(slot_ty(their_dims[their_axis]));
                let their_cell = self.axis_cell(&other.ty, their_indices, their_at, value.span);
                let their_cell: Vec<Expr> = their_cell
                    .into_iter()
                    .enumerate()
                    .map(|(k, index)| {
                        if k == their_axis {
                            index
                        } else {
                            self.hold(index, out)
                        }
                    })
                    .collect();
                let their_len = self.axis_len(other.clone(), their_axis, span);
                let (their_lo, their_hi) = self.axis_part(
                    &their_indices[their_axis],
                    their_dims[their_axis],
                    their_len,
                    out,
                );
                let (their_lo, their_hi) = (self.hold(their_lo, out), self.hold(their_hi, out));
                let step = self.mk_binary(BinOp::Add, their_lo.clone(), offset, Ty::Int, span);
                let step = self.cast(Cast::Trust, step, slot_ty(their_dims[their_axis]));
                body.push(Stmt::Let(their_at, Some(step)));
                let their_count = self.mk_binary(BinOp::Sub, their_hi, their_lo, Ty::Int, span);
                (
                    self.mk_index(other, their_cell, elem.clone(), value.span),
                    Some(their_count),
                )
            }
            // `a[l..r] max= x`: the one value against every cell.
            None if self.one_cell(value) => {
                let one = self.expect(value, &elem);
                (self.hold(one, out), None)
            }
            None => {
                let other = self.expr(value, Some(&Ty::list(elem.clone())));
                let dim = match &other.ty {
                    Ty::Array { dims, elem: theirs } if dims.len() == 1 && **theirs == elem => {
                        dims[0]
                    }
                    Ty::List(theirs) if **theirs == elem => Dim::Int,
                    Ty::Error => return changed,
                    _ => {
                        let message = format!(
                            "`{word}` goes cell by cell: this side is a row of `{}`, and the other is `{}`",
                            self.show(&elem),
                            self.show(&other.ty)
                        );
                        self.error(codes::TYPE_MISMATCH, value.span, message);
                        return changed;
                    }
                };
                let other = self.hold(other, out);
                let their_count = self.len_of(other.clone(), span);
                let index = self.cast(Cast::Trust, offset, slot_ty(dim));
                (
                    self.mk_index(other, vec![index], elem.clone(), value.span),
                    Some(their_count),
                )
            }
        };
        if let Some(their_count) = their_count {
            let differ = self.mk_cmp(CmpOp::Ne, their_count, count, Ty::Bool, span);
            let trap = Stmt::Trap(
                format!("`a {word} b` needs two rows of one length"),
                self.span(span),
            );
            out.push(Stmt::If(
                differ,
                Block { stmts: vec![trap] },
                Block::default(),
            ));
        }
        let Some(max) = max else {
            body.push(Stmt::Assign(cell, AssignOp::Set, theirs));
            let iter = Iter::Range {
                lo,
                hi,
                step: None,
                rev: false,
                closed: false,
            };
            let lid = self.fresh_loop();
            out.push(Stmt::For(lid, at, iter, Block { stmts: body }));
            return changed;
        };
        let theirs = self.hold(theirs, &mut body);
        let better = self.mk_cmp(
            if max { CmpOp::Gt } else { CmpOp::Lt },
            theirs.clone(),
            place_to_expr(&cell),
            Ty::Bool,
            span,
        );
        let mut take = vec![Stmt::Assign(cell, AssignOp::Set, theirs)];
        if let Some(changed) = changed {
            let flag = Place {
                kind: PlaceKind::Local(changed),
                ty: Ty::Bool,
                span: self.span(span),
            };
            take.push(Stmt::Assign(
                flag,
                AssignOp::Set,
                self.mk(ExprKind::Bool(true), Ty::Bool, span),
            ));
        }
        body.push(Stmt::If(better, Block { stmts: take }, Block::default()));
        let iter = Iter::Range {
            lo,
            hi,
            step: None,
            rev: false,
            closed: false,
        };
        let lid = self.fresh_loop();
        out.push(Stmt::For(lid, at, iter, Block { stmts: body }));
        changed
    }

    /// True when `value` is one cell's worth and not a row: `a[l..r] = 0`
    /// fills, `a[l..r] = b` copies. A value that has no type until it is
    /// told one (`none`) is one cell's; a list written out is a row.
    pub(in crate::lower) fn one_cell(&mut self, value: &ast::Expr) -> bool {
        match self.probe_ty_silent(value) {
            Some(ty) => !matches!(ty, Ty::List(_) | Ty::Array { .. }),
            None => !matches!(
                value.kind,
                ast::ExprKind::List(_) | ast::ExprKind::Comprehension { .. }
            ),
        }
    }

    /// `a[l..r] = b[l2..r2] + x` (and `x + b[..]`, with `-`, `*`, `&`, `|`,
    /// `^`): each cell of the part on the left takes the cell of the part
    /// on the right worked with the one value, in one loop and no copy.
    /// The two parts have one length. When they lie over each other in
    /// one array the loop runs from the end that reads every cell before
    /// it is stored to, so the result is that of working out the right
    /// side first.
    ///
    /// `a[l..r] op= b[l2..r2]` (`=`, `min=`, `max=`, `+=`, `-=`, `*=`, `&=`,
    /// `|=`, `^=`) is the same loop with no value: each cell of the left
    /// part is stored to, or worked, with the cell of the right part.
    pub(in crate::lower) fn part_map(
        &mut self,
        target: &ast::Expr,
        store: ast::AssignOp,
        source: &ast::Expr,
        with: Option<(BinOp, &ast::Expr, bool)>,
        out: &mut Vec<Stmt>,
    ) {
        let span = target.span;
        let (Some((base, indices, axis)), Some((from, from_indices, from_axis))) =
            (self.grid_row(target), self.grid_row(source))
        else {
            return;
        };
        let place = self.place(base);
        let place = self.settle_indexes(place, out);
        if let PlaceKind::Local(s) = root_place(&place) {
            self.cx.mut_slots.insert(s);
        }
        // The other array is read where it is when it has a name.
        let rows = match self.try_place(from) {
            Some(theirs) => {
                let theirs = self.settle_indexes(theirs, out);
                place_to_expr(&theirs)
            }
            None => {
                let theirs = self.expr(from, None);
                self.hold(theirs, out)
            }
        };
        let (Some((dims, elem)), Some((from_dims, from_elem))) =
            (Self::axes_of(&place.ty), Self::axes_of(&rows.ty))
        else {
            return;
        };
        let number = matches!(elem, Ty::Int | Ty::I128 | Ty::U64 | Ty::Float);
        let plain = with.is_none() && !matches!(store, ast::AssignOp::Binary(_));
        if elem != from_elem || !(number || plain) {
            if elem != Ty::Error && from_elem != Ty::Error {
                self.unsupported(span, "a part worked with one value, cell for cell, on cells that are not one kind of number");
            }
            return;
        }
        let index_ty = |dim: Dim| match dim {
            Dim::Pos(d) => Ty::Pos(d),
            Dim::Bound(d) => Ty::Bound(d),
            Dim::Enum(e) => Ty::Enum(e),
            Dim::Sym(_) | Dim::Int | Dim::IntFrom(_) => Ty::Int,
        };
        let (at, from_at) = (
            self.temp(index_ty(dims[axis])),
            self.temp(index_ty(from_dims[from_axis])),
        );
        let cell = self.axis_cell(&place.ty, indices, at, span);
        let cell: Vec<Expr> = cell
            .into_iter()
            .enumerate()
            .map(|(k, index)| {
                if k == axis {
                    index
                } else {
                    self.hold(index, out)
                }
            })
            .collect();
        let from_cell = self.axis_cell(&rows.ty, from_indices, from_at, span);
        let from_cell: Vec<Expr> = from_cell
            .into_iter()
            .enumerate()
            .map(|(k, index)| {
                if k == from_axis {
                    index
                } else {
                    self.hold(index, out)
                }
            })
            .collect();
        let len = self.axis_len(place_to_expr(&place), axis, span);
        let (lo, hi) = self.axis_part(&indices[axis], dims[axis], len, out);
        let from_len = self.axis_len(rows.clone(), from_axis, span);
        let (from_lo, from_hi) = self.axis_part(
            &from_indices[from_axis],
            from_dims[from_axis],
            from_len,
            out,
        );
        let sub = |lw: &Self, a: Expr, b: Expr| lw.mk_binary(BinOp::Sub, a, b, Ty::Int, span);
        let count = sub(self, hi, lo.clone());
        let count = self.hold(count, out);
        let theirs = sub(self, from_hi, from_lo.clone());
        let differ = self.mk_cmp(CmpOp::Ne, theirs, count.clone(), Ty::Bool, span);
        let written = match (store, with) {
            (_, Some((bin, ..))) => format!("a[l..r] = b[l2..r2] {} x", bin.text()),
            (ast::AssignOp::Set, None) => "a[l..r] = b[l2..r2]".to_string(),
            (ast::AssignOp::Max, None) => "a[l..r] max= b[l2..r2]".to_string(),
            (ast::AssignOp::Min, None) => "a[l..r] min= b[l2..r2]".to_string(),
            (ast::AssignOp::Binary(bin), None) => format!("a[l..r] {}= b[l2..r2]", bin.text()),
        };
        let trap = Stmt::Trap(
            format!("`{written}` needs two parts of one length"),
            self.span(span),
        );
        out.push(Stmt::If(
            differ,
            Block { stmts: vec![trap] },
            Block::default(),
        ));
        let with = with.map(|(bin, scalar, first)| {
            let value = self.expect(scalar, &elem);
            (bin, self.hold(value, out), first)
        });
        let k = self.temp(Ty::Int);
        let step = |lw: &Self, slot: Slot, first: &Expr, dim: Dim| {
            let sum = lw.mk_binary(BinOp::Add, first.clone(), lw.local(k, span), Ty::Int, span);
            let sum = match index_ty(dim) {
                Ty::Int => sum,
                ty => lw.cast(Cast::Trust, sum, ty),
            };
            Stmt::Let(slot, Some(sum))
        };
        let read = self.mk_index(rows, from_cell, elem.clone(), span);
        let worked = match with {
            Some((bin, value, first)) => {
                let (left, right) = if first { (value, read) } else { (read, value) };
                self.mk_binary(bin, left, right, elem.clone(), span)
            }
            None => read,
        };
        let cell = Place {
            kind: PlaceKind::Index(Box::new(place), cell),
            ty: elem,
            span: self.span(span),
        };
        let mut body = vec![
            step(self, at, &lo, dims[axis]),
            step(self, from_at, &from_lo, from_dims[from_axis]),
        ];
        self.emit_assign_place(cell, store, worked, span, &mut body);
        let run = |lw: &mut Self, rev: bool| {
            let iter = Iter::Range {
                lo: lw.int_lit(0, span),
                hi: count.clone(),
                step: None,
                rev,
                closed: false,
            };
            let lid = lw.fresh_loop();
            Block {
                stmts: vec![Stmt::For(
                    lid,
                    k,
                    iter,
                    Block {
                        stmts: body.clone(),
                    },
                )],
            }
        };
        let (forward, backward) = (run(self, false), run(self, true));
        let later = self.mk_cmp(CmpOp::Gt, lo, from_lo, Ty::Bool, span);
        out.push(Stmt::If(later, backward, forward));
    }

    /// `g[r, lo..hi] = 0`, `a[l..r] += x`, `g[.., c] min= x`: every cell of
    /// one part of a row or column takes the one value, or is worked
    /// with it, where it is, in one loop.
    pub(in crate::lower) fn grid_fill(
        &mut self,
        target: &ast::Expr,
        op: ast::AssignOp,
        value: &ast::Expr,
        out: &mut Vec<Stmt>,
    ) {
        let span = target.span;
        let Some((base, indices, axis)) = self.grid_row(target) else {
            return;
        };
        let place = self.place(base);
        let place = self.settle_indexes(place, out);
        if let PlaceKind::Local(s) = root_place(&place) {
            self.cx.mut_slots.insert(s);
        }
        let Some((dims, elem)) = Self::axes_of(&place.ty) else {
            return;
        };
        let at = self.temp(match dims[axis] {
            Dim::Pos(d) => Ty::Pos(d),
            Dim::Bound(d) => Ty::Bound(d),
            Dim::Enum(e) => Ty::Enum(e),
            Dim::Sym(_) | Dim::Int | Dim::IntFrom(_) => Ty::Int,
        });
        let cell = self.axis_cell(&place.ty, indices, at, span);
        let cell: Vec<Expr> = cell
            .into_iter()
            .enumerate()
            .map(|(k, index)| {
                if k == axis {
                    index
                } else {
                    self.hold(index, out)
                }
            })
            .collect();
        let grid = place_to_expr(&place);
        let len = self.axis_len(grid, axis, span);
        let (lo, hi) = self.axis_part(&indices[axis], dims[axis], len, out);
        let (lo, hi) = (self.hold(lo, out), self.hold(hi, out));
        let cell = Place {
            kind: PlaceKind::Index(Box::new(place), cell),
            ty: elem.clone(),
            span: self.span(span),
        };
        let value = self.expect(value, &elem);
        let value = self.hold(value, out);
        let iter = Iter::Range {
            lo,
            hi,
            step: None,
            rev: false,
            closed: false,
        };
        let lid = self.fresh_loop();
        let mut body = Vec::new();
        self.emit_assign_place(cell, op, value, span, &mut body);
        out.push(Stmt::For(lid, at, iter, Block { stmts: body }));
    }

    /// `a max= b` on two rows: every cell of `a` takes the larger of the
    /// two, in place. When `asked`, gives the slot that says whether any
    /// cell changed.
    pub(in crate::lower) fn row_keep(
        &mut self,
        place: Place,
        max: bool,
        value: Expr,
        asked: bool,
        span: Span,
        out: &mut Vec<Stmt>,
    ) -> Option<Slot> {
        let changed = asked.then(|| {
            let changed = self.temp(Ty::Bool);
            out.push(Stmt::Let(
                changed,
                Some(self.mk(ExprKind::Bool(false), Ty::Bool, span)),
            ));
            changed
        });
        let value = self.coerce(value, &place.ty, span);
        if value.ty != place.ty {
            return changed;
        }
        let (dim, elem) = match &place.ty {
            Ty::Array { dims, elem } => (dims[0], (**elem).clone()),
            Ty::List(elem) => (Dim::Int, (**elem).clone()),
            _ => unreachable!("the caller asked `is_row`"),
        };
        // A variable is read where it is; anything else is worked out once.
        let other = match value.kind {
            ExprKind::Local(_) | ExprKind::Global(_) => value,
            _ => self.hold(value, out),
        };
        let row = place_to_expr(&place);
        let len = match dim {
            Dim::Pos(d) => self.mk(ExprKind::DomainLen(d), Ty::Int, span),
            _ => {
                let len = self.len_of(row.clone(), span);
                let len = self.hold(len, out);
                let theirs = self.len_of(other.clone(), span);
                let same = self.mk_cmp(CmpOp::Ne, theirs, len.clone(), Ty::Bool, span);
                let word = if max { "max=" } else { "min=" };
                let trap = Stmt::Trap(
                    format!("`a {word} b` needs two rows of one length"),
                    self.span(span),
                );
                out.push(Stmt::If(
                    same,
                    Block { stmts: vec![trap] },
                    Block::default(),
                ));
                len
            }
        };
        let k = self.temp(Ty::Int);
        let at = match dim {
            // The loop keeps `k` under the domain's length.
            Dim::Pos(d) => self.mk(
                ExprKind::Cast(Cast::Trust, Box::new(self.local(k, span))),
                Ty::Pos(d),
                span,
            ),
            _ => self.local(k, span),
        };
        let cell = Place {
            kind: PlaceKind::Index(Box::new(place), vec![at.clone()]),
            ty: elem.clone(),
            span: self.span(span),
        };
        let theirs = self.mk_index(other, vec![at], elem, span);
        let better = self.mk_cmp(
            if max { CmpOp::Gt } else { CmpOp::Lt },
            theirs.clone(),
            place_to_expr(&cell),
            Ty::Bool,
            span,
        );
        let mut take = vec![Stmt::Assign(cell, AssignOp::Set, theirs)];
        if let Some(changed) = changed {
            let flag = Place {
                kind: PlaceKind::Local(changed),
                ty: Ty::Bool,
                span: self.span(span),
            };
            take.push(Stmt::Assign(
                flag,
                AssignOp::Set,
                self.mk(ExprKind::Bool(true), Ty::Bool, span),
            ));
        }
        let body = Block {
            stmts: vec![Stmt::If(better, Block { stmts: take }, Block::default())],
        };
        let iter = Iter::Range {
            lo: self.int_lit(0, span),
            hi: len,
            step: None,
            rev: false,
            closed: false,
        };
        let lid = self.fresh_loop();
        out.push(Stmt::For(lid, k, iter, body));
        changed
    }
}
