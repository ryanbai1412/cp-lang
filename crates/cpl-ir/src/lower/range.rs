use super::*;

/// Inclusive integer bounds `[lo, hi]` declared on a range-typed place
/// (scalar variable, struct field, or array/list element).
#[derive(Clone, Debug)]
pub(super) struct RangeBounds {
    pub lo: Option<Expr>,
    pub hi: Option<Expr>,
    pub lo_const: Option<i64>,
    pub hi_const: Option<i64>,
    pub text: String,
}

/// Range metadata attached to a local, global, or struct field.
#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)]
pub(super) enum RangeInfo {
    /// A scalar range-typed `Int` (e.g. `0..=1_000_000`, `0..P`).
    Scalar(RangeBounds),
    /// An optional whose value carries `RangeInfo` (e.g. `?(0..2_147_483_648)`).
    Opt(Box<RangeInfo>),
    /// An array or list whose elements carry `RangeInfo` (e.g. `[V]0..=1_000_000`, `List<0..=100>`).
    Elem(Box<RangeInfo>),
    /// A tuple or record whose fields may carry `RangeInfo`.
    Tuple(Vec<Option<RangeInfo>>),
    /// A key of `V.ranges`: a range that is non-empty by its type, so its
    /// `first` and `last` are positions, not optionals (lib/search-dp.md).
    Piece,
}

impl RangeBounds {
    pub(super) fn narrow_opt_layout(&self) -> Option<NarrowLayout> {
        let (lo_min, hi_max) = self.limits()?;
        (lo_min > i64::from(i32::MIN) && hi_max <= i64::from(i32::MAX))
            .then_some(NarrowLayout::OptI32)
    }

    fn limits(&self) -> Option<(i64, i64)> {
        let lo_min = self
            .lo_const
            .or_else(|| self.lo.as_ref().and_then(expr_limits).map(|(lo, _)| lo))?;
        let hi_max = self
            .hi_const
            .or_else(|| self.hi.as_ref().and_then(expr_limits).map(|(_, hi)| hi))?;
        (lo_min <= hi_max).then_some((lo_min, hi_max))
    }

    pub(super) fn narrow_width(&self) -> Option<IntWidth> {
        let lo_min = self
            .lo_const
            .or_else(|| self.lo.as_ref().and_then(expr_limits).map(|(lo, _)| lo))?;
        let hi_max = self
            .hi_const
            .or_else(|| self.hi.as_ref().and_then(expr_limits).map(|(_, hi)| hi))?;
        IntWidth::of(lo_min, hi_max)
    }
}

/// The bounds of `e` when it is `x`, `x + c` or `x - c` for a constant `c`
/// and `x` lies in `within`.
fn offset_of(e: &Expr, x: &Expr, within: (i64, i64)) -> Option<(i64, i64)> {
    let (lo, hi) = within;
    let c = match &e.kind {
        _ if same_expr(e, x) => 0,
        ExprKind::Binary(BinOp::Add, a, b) if same_expr(a, x) => const_int_expr(b)?,
        ExprKind::Binary(BinOp::Sub, a, b) if same_expr(a, x) => {
            const_int_expr(b)?.checked_neg()?
        }
        _ => return None,
    };
    (lo <= hi).then_some((lo.checked_add(c)?, hi.checked_add(c)?))
}

fn expr_limits(e: &Expr) -> Option<(i64, i64)> {
    match &e.kind {
        ExprKind::Int(v) => Some((*v, *v)),
        ExprKind::DomainLen(_) => Some((0, i64::from(i32::MAX))),
        ExprKind::Unary(UnOp::Neg, inner) => {
            let (lo, hi) = expr_limits(inner)?;
            Some((hi.checked_neg()?, lo.checked_neg()?))
        }
        ExprKind::Binary(BinOp::Add, a, b) => {
            let (alo, ahi) = expr_limits(a)?;
            let (blo, bhi) = expr_limits(b)?;
            Some((alo.checked_add(blo)?, ahi.checked_add(bhi)?))
        }
        ExprKind::Binary(BinOp::Sub, a, b) => {
            let (alo, ahi) = expr_limits(a)?;
            let (blo, bhi) = expr_limits(b)?;
            Some((alo.checked_sub(bhi)?, ahi.checked_sub(blo)?))
        }
        ExprKind::RangeCheck { value, .. } => expr_limits(value),
        _ => None,
    }
}

impl RangeInfo {
    pub(super) fn scalar(&self) -> Option<&RangeBounds> {
        match self {
            RangeInfo::Scalar(b) => Some(b),
            _ => None,
        }
    }

    pub(super) fn elem(&self) -> Option<&RangeInfo> {
        match self {
            RangeInfo::Elem(inner) => Some(inner),
            _ => None,
        }
    }

    pub(super) fn narrow_value_layout(&self, ty: &Ty) -> Option<NarrowLayout> {
        match (self, ty) {
            (RangeInfo::Scalar(b), Ty::Int) => b.narrow_width().map(NarrowLayout::Int),
            (RangeInfo::Opt(inner), Ty::Opt(t)) if **t == Ty::Int => {
                inner.scalar()?.narrow_opt_layout()
            }
            (RangeInfo::Tuple(items), Ty::Tuple(ts))
                if !ts.is_empty() && items.len() == ts.len() =>
            {
                let mut widths = Vec::with_capacity(ts.len());
                for (item, t) in items.iter().zip(ts) {
                    if *t != Ty::Int {
                        return None;
                    }
                    let w = item.as_ref()?.scalar()?.narrow_width()?;
                    widths.push(w);
                }
                Some(NarrowLayout::Tuple(widths))
            }
            (RangeInfo::Tuple(items), Ty::Record(fs))
                if !fs.is_empty() && items.len() == fs.len() =>
            {
                let mut widths = Vec::with_capacity(fs.len());
                for (item, (_, t)) in items.iter().zip(fs) {
                    if *t != Ty::Int {
                        return None;
                    }
                    let w = item.as_ref()?.scalar()?.narrow_width()?;
                    widths.push(w);
                }
                Some(NarrowLayout::Tuple(widths))
            }
            (
                RangeInfo::Elem(_),
                Ty::Array { .. } | Ty::List(_) | Ty::ListView(_, _) | Ty::Keyed(_, _),
            ) => self
                .narrow_elem_layout(ty)
                .map(|l| NarrowLayout::Seq(Box::new(l))),
            _ => None,
        }
    }

    pub(super) fn narrow_elem_layout(&self, container_ty: &Ty) -> Option<NarrowLayout> {
        let elem_ty = match container_ty {
            Ty::Array { elem, .. }
            | Ty::List(elem)
            | Ty::ListView(elem, _)
            | Ty::Keyed(elem, _) => elem.as_ref(),
            _ => return None,
        };
        self.elem()?.narrow_value_layout(elem_ty)
    }

    pub(super) fn from_spec(spec: &Spec) -> Option<Self> {
        match spec {
            Spec::Int { lo, hi, text } => {
                let lo_expr = lo.as_deref().cloned();
                let hi_expr = hi.as_deref().cloned();
                let lo_const = lo_expr.as_ref().and_then(const_int_expr);
                let hi_const = hi_expr.as_ref().and_then(const_int_expr);
                Some(RangeInfo::Scalar(RangeBounds {
                    lo: lo_expr,
                    hi: hi_expr,
                    lo_const,
                    hi_const,
                    text: text.clone(),
                }))
            }
            Spec::Array { elem, .. } => {
                RangeInfo::from_spec(elem).map(|e| RangeInfo::Elem(Box::new(e)))
            }
            Spec::Fields(_, fields) => {
                let items: Vec<Option<RangeInfo>> =
                    fields.iter().map(RangeInfo::from_spec).collect();
                items
                    .iter()
                    .any(Option::is_some)
                    .then_some(RangeInfo::Tuple(items))
            }
            _ => None,
        }
    }
}

pub(super) fn const_int_expr(e: &Expr) -> Option<i64> {
    match &e.kind {
        ExprKind::Int(v) => Some(*v),
        ExprKind::Unary(UnOp::Neg, inner) => const_int_expr(inner).and_then(i64::checked_neg),
        ExprKind::Binary(BinOp::Add, a, b) => const_int_expr(a)?.checked_add(const_int_expr(b)?),
        ExprKind::Binary(BinOp::Sub, a, b) => const_int_expr(a)?.checked_sub(const_int_expr(b)?),
        ExprKind::Binary(BinOp::Mul, a, b) => const_int_expr(a)?.checked_mul(const_int_expr(b)?),
        _ => None,
    }
}

/// Structural equality on simple bound expressions, ignoring source spans.
fn same_expr(a: &Expr, b: &Expr) -> bool {
    match (&a.kind, &b.kind) {
        (ExprKind::Int(x), ExprKind::Int(y)) => x == y,
        (ExprKind::Global(x), ExprKind::Global(y)) => x == y,
        (ExprKind::Local(x), ExprKind::Local(y)) => x == y,
        (ExprKind::DomainLen(x), ExprKind::DomainLen(y)) => x == y,
        (ExprKind::Field(ab, af), ExprKind::Field(bb, bf)) => af == bf && same_expr(ab, bb),
        (ExprKind::Unary(ao, ax), ExprKind::Unary(bo, bx)) => ao == bo && same_expr(ax, bx),
        (ExprKind::Binary(ao, al, ar), ExprKind::Binary(bo, bl, br)) => {
            ao == bo && same_expr(al, bl) && same_expr(ar, br)
        }
        _ => false,
    }
}

/// True when `hi_expr` is `divisor - 1` (either statically or as `Binary(Add, divisor, -1)` /
/// `Binary(Sub, divisor, 1)`), matching Euclidean `x % divisor` against `0..divisor`.
fn is_minus_one_of(hi_expr: &Expr, divisor: &Expr) -> bool {
    if let (Some(h), Some(d)) = (const_int_expr(hi_expr), const_int_expr(divisor)) {
        return d > 0 && h == d - 1;
    }
    match &hi_expr.kind {
        ExprKind::Binary(BinOp::Add, lhs, rhs) => {
            (same_expr(lhs, divisor) && matches!(rhs.kind, ExprKind::Int(-1)))
                || (same_expr(rhs, divisor) && matches!(lhs.kind, ExprKind::Int(-1)))
        }
        ExprKind::Binary(BinOp::Sub, lhs, rhs) => {
            same_expr(lhs, divisor) && matches!(rhs.kind, ExprKind::Int(1))
        }
        _ => false,
    }
}

pub(super) fn is_narrowable_elem_ty(ty: &Ty) -> bool {
    matches!(
        ty,
        Ty::Array { .. } | Ty::List(_) | Ty::ListView(_, _) | Ty::Keyed(_, _)
    )
}

impl<'a> Lowerer<'a> {
    pub(super) fn record_narrow_locals(
        &mut self,
        scope: ScopeId,
        locals: &[Local],
        ranges: &HashMap<Slot, RangeInfo>,
    ) {
        for (&slot, info) in ranges {
            if let Some(l) = locals.get(slot).filter(|l| is_narrowable_elem_ty(&l.ty)) {
                if let Some(layout) = info.narrow_elem_layout(&l.ty) {
                    self.narrow_elems
                        .insert(ElemLoc::Local(scope, slot), layout);
                }
            }
        }
    }

    /// Extracts `RangeInfo` from an AST type, following type aliases and handling
    /// `lo..hi`, `[dims]T`, `List<T>`, tuples, and records.
    pub(super) fn range_info_from_ast(&mut self, ty: &ast::Type) -> Option<RangeInfo> {
        match &ty.kind {
            ast::TypeKind::Range { lo, hi, inclusive } => {
                let lo_expr = lo.as_deref().map(|e| self.resolve_bound_expr(e));
                let hi_expr = hi.as_deref().map(|e| {
                    let mut v = self.resolve_bound_expr(e);
                    if !inclusive {
                        v = self.int_add(v, -1);
                    }
                    v
                });
                let lo_const = lo_expr.as_ref().and_then(const_int_expr);
                let hi_const = hi_expr.as_ref().and_then(const_int_expr);
                Some(RangeInfo::Scalar(RangeBounds {
                    lo: lo_expr,
                    hi: hi_expr,
                    lo_const,
                    hi_const,
                    text: self.source(ty.span),
                }))
            }
            ast::TypeKind::Array { elem, .. } | ast::TypeKind::OwnDomain { elem, .. } => self
                .range_info_from_ast(elem)
                .map(|e| RangeInfo::Elem(Box::new(e))),
            ast::TypeKind::Named { name, args } => {
                if name.text.ends_with(".ranges") && args.is_empty() {
                    return Some(RangeInfo::Piece);
                }
                if name.text == "List" && args.len() == 1 && self.get_sym(&name.text).is_none() {
                    return self
                        .range_info_from_ast(&args[0])
                        .map(|e| RangeInfo::Elem(Box::new(e)));
                }
                if args.is_empty() {
                    if let Some(GenArg::Range(lo_c, hi_c)) = self.gen_env.get(&name.text).cloned() {
                        let lo_expr = lo_c.map(|v| self.int_lit(v, ty.span));
                        let hi_expr = hi_c.map(|v| self.int_lit(v, ty.span));
                        return Some(RangeInfo::Scalar(RangeBounds {
                            lo: lo_expr,
                            hi: hi_expr,
                            lo_const: lo_c,
                            hi_const: hi_c,
                            text: format!(
                                "{}..={}",
                                lo_c.map_or(String::new(), |v| v.to_string()),
                                hi_c.map_or(String::new(), |v| v.to_string())
                            ),
                        }));
                    }
                    if let Some(Sym::Alias(aid)) = self.get_sym(&name.text) {
                        let (m, target, off) = self.aliases[aid];
                        let saved_m = std::mem::replace(&mut self.cur_mod, m);
                        let saved_off = std::mem::replace(&mut self.offset, off);
                        let res = self.range_info_from_ast(target);
                        self.cur_mod = saved_m;
                        self.offset = saved_off;
                        return res;
                    }
                }
                None
            }
            ast::TypeKind::Optional(inner) => self
                .range_info_from_ast(inner)
                .map(|r| RangeInfo::Opt(Box::new(r))),
            ast::TypeKind::Tuple(items) => {
                let parts: Vec<Option<RangeInfo>> =
                    items.iter().map(|t| self.range_info_from_ast(t)).collect();
                parts
                    .iter()
                    .any(Option::is_some)
                    .then_some(RangeInfo::Tuple(parts))
            }
            ast::TypeKind::Record(fields) => {
                let parts: Vec<Option<RangeInfo>> = fields
                    .iter()
                    .map(|(_, t)| self.range_info_from_ast(t))
                    .collect();
                parts
                    .iter()
                    .any(Option::is_some)
                    .then_some(RangeInfo::Tuple(parts))
            }
            _ => None,
        }
    }

    /// Resolves a range bound expression without emitting errors if a dynamic name
    /// is not yet in scope (used when pre-scanning struct definitions).
    fn resolve_bound_expr(&mut self, e: &ast::Expr) -> Expr {
        if let Some(v) = self.eval_const_int(e) {
            return self.int_lit(v, e.span);
        }
        self.expect(e, &Ty::Int)
    }

    /// Resolves static/gen-env range metadata for every field of struct `id`.
    /// Fields whose bounds depend on runtime locals/globals not yet declared are
    /// resolved lazily when constructed or assigned.
    pub(super) fn collect_struct_field_ranges(&mut self, id: StructId, decl: &ast::StructDecl) {
        let saved_cx = std::mem::take(&mut self.cx);
        let saved_outer = std::mem::take(&mut self.outer_cxs);
        for (idx, field) in decl.fields.iter().enumerate() {
            let saved_diags = self.diags.len();
            let info = self.range_info_from_ast(&field.ty);
            let had_err = self.diags.len() > saved_diags;
            self.diags.truncate(saved_diags);
            if !had_err {
                if let Some(info) = info {
                    if let Some((_, field_ty)) = self.structs[id].def.fields.get(idx) {
                        if let Some(layout) = info.narrow_elem_layout(field_ty) {
                            self.narrow_elems.insert(ElemLoc::Field(id, idx), layout);
                        }
                    }
                    self.struct_field_ranges.insert((id, idx), info);
                }
            }
        }
        self.cx = saved_cx;
        self.outer_cxs = saved_outer;
    }

    /// Looks up `RangeInfo` for field `idx` of struct `id`, falling back to
    /// resolving the field's AST type in the current scope for dynamic bounds
    /// (e.g. `struct Pt(r: 1..=n, c: 1..=m)`).
    pub(super) fn struct_field_range(&mut self, id: StructId, idx: usize) -> Option<RangeInfo> {
        if let Some(info) = self.struct_field_ranges.get(&(id, idx)) {
            return Some(info.clone());
        }
        let decl = self.structs.get(id)?.decl?;
        let field = decl.fields.get(idx)?;
        let m = self.structs[id].mod_id;
        let off = self.structs[id].offset;
        let saved_m = std::mem::replace(&mut self.cur_mod, m);
        let saved_off = std::mem::replace(&mut self.offset, off);
        let saved_env = std::mem::replace(&mut self.gen_env, self.structs[id].env.clone());
        let saved_diags = self.diags.len();
        let info = self.range_info_from_ast(&field.ty);
        let had_err = self.diags.len() > saved_diags;
        self.diags.truncate(saved_diags);
        self.gen_env = saved_env;
        self.cur_mod = saved_m;
        self.offset = saved_off;
        if had_err {
            None
        } else {
            info
        }
    }

    /// Records `RangeInfo` for every binding introduced by `pat`.
    pub(super) fn bind_range_info(&mut self, pat: &ast::Pattern, info: &RangeInfo) {
        match (&pat.kind, info) {
            (ast::PatternKind::Name(name), _) => {
                if self.cx.is_main && self.cx.scopes.len() == 1 {
                    if let Some(Sym::Global(gid)) = self.mod_syms[self.cur_mod].get(name).copied() {
                        self.global_ranges.insert(gid, info.clone());
                    }
                } else if let Some(slot) = self.lookup(name) {
                    self.cx.local_ranges.insert(slot, info.clone());
                }
            }
            (ast::PatternKind::Variant { path, args }, _) if path.len() == 1 && args.is_empty() => {
                let name = &path[0].text;
                if self.cx.is_main && self.cx.scopes.len() == 1 {
                    if let Some(Sym::Global(gid)) = self.mod_syms[self.cur_mod].get(name).copied() {
                        self.global_ranges.insert(gid, info.clone());
                    }
                } else if let Some(slot) = self.lookup(name) {
                    self.cx.local_ranges.insert(slot, info.clone());
                }
            }
            (ast::PatternKind::Tuple(pats), RangeInfo::Tuple(infos)) => {
                for (p, sub) in pats.iter().zip(infos) {
                    if let Some(sub) = sub {
                        self.bind_range_info(p, sub);
                    }
                }
            }
            (ast::PatternKind::Tuple(pats), RangeInfo::Scalar(_) | RangeInfo::Elem(_)) => {
                // Shared annotation `var x, y: 0..=100 = (5, 5)`: applies to each binding.
                for p in pats {
                    self.bind_range_info(p, info);
                }
            }
            _ => {}
        }
    }

    /// Looks up the `RangeInfo` of a `Place` (scalar var, global, struct field,
    /// array/list element, or tuple field).
    pub(super) fn place_range(&mut self, place: &Place) -> Option<RangeInfo> {
        match &place.kind {
            PlaceKind::Local(slot) => self.cx.local_ranges.get(slot).cloned(),
            PlaceKind::Global(gid) => self.global_ranges.get(gid).cloned(),
            PlaceKind::Field(base, idx) => {
                if let Ty::Struct(sid) = base.ty {
                    return self.struct_field_range(sid, *idx);
                }
                match self.place_range(base)? {
                    RangeInfo::Tuple(items) => items.get(*idx).cloned().flatten(),
                    _ => None,
                }
            }
            PlaceKind::Index(base, _) => match self.place_range(base)? {
                RangeInfo::Elem(inner) => Some(*inner),
                _ => None,
            },
            PlaceKind::Some(_) => None,
        }
    }

    /// Looks up scalar `RangeBounds` for a `Place` of type `Ty::Int`.
    pub(super) fn place_scalar_range(&mut self, place: &Place) -> Option<RangeBounds> {
        if place.ty != Ty::Int {
            return None;
        }
        match self.place_range(place)? {
            RangeInfo::Scalar(b) => Some(b),
            _ => None,
        }
    }

    /// Looks up `RangeInfo` for a value expression (used to prove sub-range sources).
    pub(super) fn expr_range(&self, e: &Expr) -> Option<RangeInfo> {
        match &e.kind {
            ExprKind::Local(slot) => self.cx.local_ranges.get(slot).cloned(),
            ExprKind::Global(gid) => self.global_ranges.get(gid).cloned(),
            ExprKind::Field(base, idx) => {
                if let Ty::Struct(sid) = base.ty {
                    return self.struct_field_ranges.get(&(sid, *idx)).cloned();
                }
                match self.expr_range(base)? {
                    RangeInfo::Tuple(items) => items.get(*idx).cloned().flatten(),
                    _ => None,
                }
            }
            ExprKind::Index(base, _) => match self.expr_range(base)? {
                RangeInfo::Elem(inner) => Some(*inner),
                _ => None,
            },
            ExprKind::Builtin(
                Builtin::ListSlice | Builtin::KeySlice | Builtin::Rekey | Builtin::ToList,
                args,
            ) => args.first().and_then(|a| self.expr_range(a)),
            ExprKind::Do(blk) => match blk.stmts.last() {
                Some(Stmt::Give(e)) => self.expr_range(e),
                _ => None,
            },
            _ => None,
        }
    }

    /// The constant bounds a local `let` bound to `value` keeps (see `FnCx::let_bounds`):
    /// none when it is mutable, typed or global (`fixed`), or not an `Int`.
    pub(super) fn let_bounds(&self, fixed: bool, value: &Expr) -> Option<(i64, i64)> {
        if fixed || value.ty != Ty::Int {
            return None;
        }
        let (lo, hi) = self.expr_const_bounds(value);
        lo.zip(hi)
    }

    /// Records `bounds` for the local just bound to `name`.
    pub(super) fn note_let_bounds(&mut self, bounds: Option<(i64, i64)>, name: Option<&str>) {
        if let (Some(b), Some(slot)) = (bounds, name.and_then(|n| self.lookup(n))) {
            self.cx.let_bounds.insert(slot, b);
        }
    }

    /// Computes sound static inclusive bounds `(lo, hi)` for `e` when provable.
    fn expr_const_bounds(&self, e: &Expr) -> (Option<i64>, Option<i64>) {
        match &e.kind {
            ExprKind::Int(v) => (Some(*v), Some(*v)),
            ExprKind::Unary(UnOp::Neg, inner) => {
                let (lo, hi) = self.expr_const_bounds(inner);
                (hi.and_then(i64::checked_neg), lo.and_then(i64::checked_neg))
            }
            ExprKind::Local(_)
            | ExprKind::Global(_)
            | ExprKind::Field(..)
            | ExprKind::Index(..) => {
                if let ExprKind::Global(gid) = &e.kind {
                    if let Some(&v) = self.const_ints.get(&self.globals[*gid].name) {
                        return (Some(v), Some(v));
                    }
                }
                match self.expr_range(e) {
                    Some(RangeInfo::Scalar(b)) => (b.lo_const, b.hi_const),
                    _ => match &e.kind {
                        ExprKind::Local(slot) => self
                            .cx
                            .let_bounds
                            .get(slot)
                            .map_or((None, None), |&(lo, hi)| (Some(lo), Some(hi))),
                        _ => (None, None),
                    },
                }
            }
            ExprKind::Read(spec) => match spec.as_ref() {
                Spec::Int { lo, hi, .. } => (
                    lo.as_deref().and_then(const_int_expr),
                    hi.as_deref().and_then(const_int_expr),
                ),
                _ => (None, None),
            },
            ExprKind::RangeCheck { lo, hi, .. } => (
                lo.as_deref().and_then(const_int_expr),
                hi.as_deref().and_then(const_int_expr),
            ),
            ExprKind::Binary(BinOp::Rem, _, rhs) => {
                if let (Some(lo), Some(hi)) = self.expr_const_bounds(rhs) {
                    if lo == hi && lo > 0 {
                        return (Some(0), Some(lo - 1));
                    }
                }
                (None, None)
            }
            ExprKind::Binary(BinOp::BitAnd, a, b) => {
                if let Some(m) = const_int_expr(a).filter(|&v| v >= 0) {
                    return (Some(0), Some(m));
                }
                if let Some(m) = const_int_expr(b).filter(|&v| v >= 0) {
                    return (Some(0), Some(m));
                }
                (None, None)
            }
            ExprKind::Binary(BinOp::Add, a, b) => {
                let (alo, ahi) = self.expr_const_bounds(a);
                let (blo, bhi) = self.expr_const_bounds(b);
                (
                    alo.and_then(|x| blo.and_then(|y| x.checked_add(y))),
                    ahi.and_then(|x| bhi.and_then(|y| x.checked_add(y))),
                )
            }
            ExprKind::Binary(BinOp::Sub, a, b) => {
                let (alo, ahi) = self.expr_const_bounds(a);
                let (blo, bhi) = self.expr_const_bounds(b);
                (
                    alo.and_then(|x| bhi.and_then(|y| x.checked_sub(y))),
                    ahi.and_then(|x| blo.and_then(|y| x.checked_sub(y))),
                )
            }
            ExprKind::If(cond, then_e, else_e) => {
                let (mut tlo, mut thi) = self.expr_const_bounds(then_e);
                let (mut elo, mut ehi) = self.expr_const_bounds(else_e);
                if let ExprKind::Cmp(op, lhs, rhs) = &cond.kind {
                    // A branch that is the compared value plus a constant, under the
                    // bound the condition (or its negation) puts on that value.
                    if let (Some(k), (Some(lo), Some(hi))) =
                        (const_int_expr(rhs), self.expr_const_bounds(lhs))
                    {
                        let below = |k: i64| (lo, hi.min(k));
                        let above = |k: i64| (lo.max(k), hi);
                        let split = match op {
                            CmpOp::Lt => Some((below(k.saturating_sub(1)), above(k))),
                            CmpOp::Le => Some((below(k), above(k.saturating_add(1)))),
                            CmpOp::Gt => Some((above(k.saturating_add(1)), below(k))),
                            CmpOp::Ge => Some((above(k), below(k.saturating_sub(1)))),
                            _ => None,
                        };
                        if let Some((yes, no)) = split {
                            if let Some((l, h)) = offset_of(then_e, lhs, yes) {
                                (tlo, thi) = (Some(l), Some(h));
                            }
                            if let Some((l, h)) = offset_of(else_e, lhs, no) {
                                (elo, ehi) = (Some(l), Some(h));
                            }
                        }
                    }
                    match op {
                        CmpOp::Ge => {
                            if let Some(k) = const_int_expr(rhs) {
                                // `if x >= k: x - k else: x`
                                if let ExprKind::Binary(BinOp::Sub, a, b) = &then_e.kind {
                                    if a.as_ref() == lhs.as_ref() && const_int_expr(b) == Some(k) {
                                        let (_, xhi) = self.expr_const_bounds(lhs);
                                        tlo = Some(0);
                                        thi = xhi.and_then(|h| h.checked_sub(k));
                                    }
                                }
                                if else_e.as_ref() == lhs.as_ref() {
                                    ehi = Some(ehi.map_or(k - 1, |h| h.min(k - 1)));
                                }
                            } else {
                                // `if a >= b: a - b else: a - b + P`
                                if let ExprKind::Binary(BinOp::Sub, ta, tb) = &then_e.kind {
                                    if ta.as_ref() == lhs.as_ref() && tb.as_ref() == rhs.as_ref() {
                                        tlo = Some(tlo.map_or(0, |l| l.max(0)));
                                    }
                                }
                                if let ExprKind::Binary(BinOp::Add, esub, ep) = &else_e.kind {
                                    if let (ExprKind::Binary(BinOp::Sub, ea, eb), Some(p)) =
                                        (&esub.kind, const_int_expr(ep))
                                    {
                                        if ea.as_ref() == lhs.as_ref()
                                            && eb.as_ref() == rhs.as_ref()
                                        {
                                            ehi = Some(ehi.map_or(p - 1, |h| h.min(p - 1)));
                                        }
                                    }
                                }
                            }
                        }
                        CmpOp::Eq => {
                            // `if a == 0: 0 else: P - a` when `a >= 0`
                            if const_int_expr(rhs) == Some(0) {
                                if let ExprKind::Binary(BinOp::Sub, ep, ea) = &else_e.kind {
                                    if ea.as_ref() == lhs.as_ref() {
                                        if let (Some(p), (Some(alo), _)) =
                                            (const_int_expr(ep), self.expr_const_bounds(lhs))
                                        {
                                            if alo >= 0 {
                                                ehi = Some(ehi.map_or(p - 1, |h| h.min(p - 1)));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                (
                    tlo.zip(elo).map(|(a, b)| a.min(b)),
                    thi.zip(ehi).map(|(a, b)| a.max(b)),
                )
            }
            ExprKind::Builtin(Builtin::Min, args) if args.len() == 2 => {
                let (alo, ahi) = self.expr_const_bounds(&args[0]);
                let (blo, bhi) = self.expr_const_bounds(&args[1]);
                (
                    alo.zip(blo).map(|(a, b)| a.min(b)),
                    match (ahi, bhi) {
                        (Some(a), Some(b)) => Some(a.min(b)),
                        (Some(a), None) | (None, Some(a)) => Some(a),
                        (None, None) => None,
                    },
                )
            }
            ExprKind::Builtin(Builtin::Max, args) if args.len() == 2 => {
                let (alo, ahi) = self.expr_const_bounds(&args[0]);
                let (blo, bhi) = self.expr_const_bounds(&args[1]);
                (
                    match (alo, blo) {
                        (Some(a), Some(b)) => Some(a.max(b)),
                        (Some(a), None) | (None, Some(a)) => Some(a),
                        (None, None) => None,
                    },
                    ahi.zip(bhi).map(|(a, b)| a.max(b)),
                )
            }
            ExprKind::Cast(Cast::I128ToInt | Cast::U64ToInt, inner) => {
                if let ExprKind::Binary(BinOp::Rem, _, rhs) = &inner.kind {
                    if let ExprKind::Cast(Cast::IntToI128 | Cast::IntToU64, div) = &rhs.kind {
                        if let Some(p) = const_int_expr(div) {
                            if p > 0 {
                                return (Some(0), Some(p - 1));
                            }
                        }
                    }
                }
                (None, None)
            }
            _ => (None, None),
        }
    }

    /// True when `value` is statically guaranteed to lie inside `dst`.
    pub(super) fn proven_in_range(&self, value: &Expr, dst: &RangeBounds) -> bool {
        if dst.lo.is_none() && dst.hi.is_none() {
            return true;
        }
        // 1. Constant-bounds inclusion.
        let (vlo, vhi) = self.expr_const_bounds(value);
        let lo_ok = match (&dst.lo, dst.lo_const) {
            (None, _) => true,
            (Some(_), Some(dlo)) => vlo.is_some_and(|v| v >= dlo),
            (Some(_), None) => false,
        };
        let hi_ok = match (&dst.hi, dst.hi_const) {
            (None, _) => true,
            (Some(_), Some(dhi)) => vhi.is_some_and(|v| v <= dhi),
            (Some(_), None) => false,
        };
        if lo_ok && hi_ok {
            return true;
        }
        // 2. Dynamic-bounds equality or Euclidean `% P` into `0..P`.
        match &value.kind {
            ExprKind::Read(spec) => {
                if let Spec::Int { lo, hi, .. } = spec.as_ref() {
                    let lo_eq = dst.lo.is_none()
                        || lo
                            .as_deref()
                            .zip(dst.lo.as_ref())
                            .is_some_and(|(a, b)| same_expr(a, b));
                    let hi_eq = dst.hi.is_none()
                        || hi
                            .as_deref()
                            .zip(dst.hi.as_ref())
                            .is_some_and(|(a, b)| same_expr(a, b));
                    if lo_eq && hi_eq {
                        return true;
                    }
                }
            }
            ExprKind::RangeCheck { lo, hi, .. } => {
                let lo_eq = dst.lo.is_none()
                    || lo
                        .as_deref()
                        .zip(dst.lo.as_ref())
                        .is_some_and(|(a, b)| same_expr(a, b));
                let hi_eq = dst.hi.is_none()
                    || hi
                        .as_deref()
                        .zip(dst.hi.as_ref())
                        .is_some_and(|(a, b)| same_expr(a, b));
                if lo_eq && hi_eq {
                    return true;
                }
            }
            ExprKind::Local(_)
            | ExprKind::Global(_)
            | ExprKind::Field(..)
            | ExprKind::Index(..) => {
                if let Some(RangeInfo::Scalar(src)) = self.expr_range(value) {
                    let lo_eq = dst.lo.is_none()
                        || src
                            .lo
                            .as_ref()
                            .zip(dst.lo.as_ref())
                            .is_some_and(|(a, b)| same_expr(a, b))
                        || (src.lo_const.is_some()
                            && dst.lo_const.is_some()
                            && src.lo_const >= dst.lo_const);
                    let hi_eq = dst.hi.is_none()
                        || src
                            .hi
                            .as_ref()
                            .zip(dst.hi.as_ref())
                            .is_some_and(|(a, b)| same_expr(a, b))
                        || (src.hi_const.is_some()
                            && dst.hi_const.is_some()
                            && src.hi_const <= dst.hi_const);
                    if lo_eq && hi_eq {
                        return true;
                    }
                }
            }
            ExprKind::Binary(BinOp::Rem, _, rhs) => {
                if dst.lo_const.is_some_and(|l| l <= 0)
                    && dst.hi.as_ref().is_some_and(|h| is_minus_one_of(h, rhs))
                {
                    return true;
                }
            }
            ExprKind::Cast(Cast::I128ToInt | Cast::U64ToInt, inner) => {
                if let ExprKind::Binary(BinOp::Rem, _, rhs) = &inner.kind {
                    if let ExprKind::Cast(Cast::IntToI128 | Cast::IntToU64, div) = &rhs.kind {
                        if dst.lo_const.is_some_and(|l| l <= 0)
                            && dst.hi.as_ref().is_some_and(|h| is_minus_one_of(h, div))
                        {
                            return true;
                        }
                    }
                }
            }
            ExprKind::If(cond, t, e) => {
                if self.proven_in_range(t, dst) && self.proven_in_range(e, dst) {
                    return true;
                }
                // Runtime-modulus `Mod<m>` branch-refined `+`, `-`, and unary `-`:
                // e.g. `if a + b >= m { a + b - m } else { a + b }`,
                //      `if a >= b { a - b } else { a - b + m }`,
                //      `if a == 0 { 0 } else { m - a }` where `a, b in 0..m`.
                if dst.lo_const.is_some_and(|l| l <= 0) {
                    if let Some(dhi) = dst.hi.as_ref() {
                        match &cond.kind {
                            ExprKind::Cmp(CmpOp::Ge, lhs, rhs) => {
                                // `if a + b >= m { a + b - m } else { a + b }`
                                if is_minus_one_of(dhi, rhs)
                                    && same_expr(lhs, e)
                                    && matches!(
                                        &t.kind,
                                        ExprKind::Binary(BinOp::Sub, tl, tr)
                                            if same_expr(tl, lhs) && same_expr(tr, rhs)
                                    )
                                {
                                    if let ExprKind::Binary(BinOp::Add, a, b) = &lhs.kind {
                                        if self.proven_in_range(a, dst)
                                            && self.proven_in_range(b, dst)
                                        {
                                            return true;
                                        }
                                    }
                                }
                                // `if a >= b { a - b } else { a - b + m }`
                                if self.proven_in_range(lhs, dst)
                                    && self.proven_in_range(rhs, dst)
                                    && matches!(
                                        &t.kind,
                                        ExprKind::Binary(BinOp::Sub, tl, tr)
                                            if same_expr(tl, lhs) && same_expr(tr, rhs)
                                    )
                                    && matches!(
                                        &e.kind,
                                        ExprKind::Binary(BinOp::Add, el, er)
                                            if same_expr(el, t) && is_minus_one_of(dhi, er)
                                    )
                                {
                                    return true;
                                }
                            }
                            ExprKind::Cmp(CmpOp::Eq, lhs, rhs)
                                if matches!(rhs.kind, ExprKind::Int(0))
                                    && matches!(t.kind, ExprKind::Int(0)) =>
                            {
                                // `if a == 0 { 0 } else { m - a }`
                                if self.proven_in_range(lhs, dst)
                                    && matches!(
                                        &e.kind,
                                        ExprKind::Binary(BinOp::Sub, el, er)
                                            if is_minus_one_of(dhi, el) && same_expr(er, lhs)
                                    )
                                {
                                    return true;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
        false
    }

    /// Wraps `value` in `ExprKind::RangeCheck` unless statically proven inside `bounds`.
    pub(super) fn check_range(&mut self, value: Expr, bounds: &RangeBounds, span: Span) -> Expr {
        if value.ty != Ty::Int {
            return value;
        }
        if self.proven_in_range(&value, bounds) {
            return value;
        }
        self.mk(
            ExprKind::RangeCheck {
                value: Box::new(value),
                lo: bounds.lo.clone().map(Box::new),
                hi: bounds.hi.clone().map(Box::new),
                text: bounds.text.clone(),
            },
            Ty::Int,
            span,
        )
    }

    /// Emits `Stmt::Assign(place, op, value)`, validating range bounds when `place`
    /// is range-typed. Handles plain `=`, compound `+=`/`-=`/etc., and `max=`/`min=`.
    pub(super) fn emit_assign(
        &mut self,
        place: Place,
        op: AssignOp,
        value: Expr,
        span: Span,
        out: &mut Vec<Stmt>,
    ) {
        if let (Ty::Opt(_), AssignOp::Set, ExprKind::Some(_)) = (&place.ty, op, &value.kind) {
            if let Some(RangeInfo::Opt(inner)) = self.place_range(&place) {
                if let RangeInfo::Scalar(bounds) = *inner {
                    let (ty, at) = (value.ty.clone(), value.span);
                    let ExprKind::Some(x) = value.kind else {
                        unreachable!()
                    };
                    let x = self.check_range(*x, &bounds, span);
                    out.push(Stmt::Assign(
                        place,
                        op,
                        Expr {
                            kind: ExprKind::Some(Box::new(x)),
                            ty,
                            span: at,
                        },
                    ));
                    return;
                }
            }
        }
        let Some(bounds) = self.place_scalar_range(&place) else {
            out.push(Stmt::Assign(place, op, value));
            return;
        };
        match op {
            AssignOp::Set => {
                let checked = self.check_range(value, &bounds, span);
                out.push(Stmt::Assign(place, AssignOp::Set, checked));
            }
            AssignOp::Binary(bin) => {
                let cur = super::stmt::place_to_expr(&place);
                let next = self.mk(
                    ExprKind::Binary(bin, Box::new(cur), Box::new(value)),
                    Ty::Int,
                    span,
                );
                let checked = self.check_range(next, &bounds, span);
                out.push(Stmt::Assign(place, AssignOp::Set, checked));
            }
            AssignOp::Max | AssignOp::Min => {
                if self.proven_in_range(&value, &bounds) {
                    out.push(Stmt::Assign(place, op, value));
                    return;
                }
                let slot = self.temp(Ty::Int);
                out.push(Stmt::Let(slot, Some(value)));
                let rhs = self.local(slot, span);
                let cur = super::stmt::place_to_expr(&place);
                let cmp = if op == AssignOp::Max {
                    CmpOp::Gt
                } else {
                    CmpOp::Lt
                };
                let cond = self.mk(
                    ExprKind::Cmp(cmp, Box::new(rhs.clone()), Box::new(cur)),
                    Ty::Bool,
                    span,
                );
                let checked = self.check_range(rhs, &bounds, span);
                out.push(Stmt::If(
                    cond,
                    Block {
                        stmts: vec![Stmt::Assign(place, AssignOp::Set, checked)],
                    },
                    Block::default(),
                ));
            }
        }
    }
}

impl<'a> Lowerer<'a> {
    /// `s.set(b, on)` on a `Subset<B>` writes `s` (`set(mut self, ...)`).
    pub(super) fn subset_set(&mut self, e: &ast::Expr, out: &mut Vec<Stmt>) -> bool {
        let ast::ExprKind::Call { callee, .. } = &e.kind else {
            return false;
        };
        let ast::ExprKind::Field { base, name } = &callee.kind else {
            return false;
        };
        if name.text != "set" {
            return false;
        }
        let Some(Ty::Pos(d)) = self.probe_ty_silent(base) else {
            return false;
        };
        if self.subset_base(d).is_none() {
            return false;
        }
        let value = self.expr(e, None);
        let place = self.place(base);
        out.push(Stmt::Assign(place, AssignOp::Set, value));
        true
    }

    /// The cell of the non-empty range `piece` in a `[V.ranges]T`; an empty range traps, and so
    /// does a `Range<Int>` reaching outside `V`.
    pub(super) fn piece_idx(&mut self, piece: Expr, d: DomainId, span: Span) -> Expr {
        let ty = piece.ty.clone();
        let of_ints = matches!(&ty, Ty::Range(inner) if **inner == Ty::Int);
        let sym = self.do_expr(Some(&Ty::Int), span, |lw, out| {
            let r = lw.new_slot("piece", ty);
            out.push(Stmt::Let(r, Some(piece)));
            let (lo, hi) = (lw.new_slot("lo", Ty::Int), lw.new_slot("hi", Ty::Int));
            for (slot, end) in [(lo, Builtin::RangeLo), (hi, Builtin::RangeHi)] {
                let value = lw.local(r, span);
                let value = lw.mk(ExprKind::Builtin(end, vec![value]), Ty::Int, span);
                out.push(Stmt::Let(slot, Some(value)));
            }
            let (l, h) = (lw.local(lo, span), lw.local(hi, span));
            let empty = lw.mk(
                ExprKind::Cmp(CmpOp::Le, Box::new(h), Box::new(l)),
                Ty::Bool,
                span,
            );
            let trap = Stmt::Trap(
                "an empty range has no cell in `V.ranges`".into(),
                lw.span(span),
            );
            out.push(Stmt::If(
                empty,
                Block { stmts: vec![trap] },
                Block::default(),
            ));
            if of_ints {
                let (l, h) = (lw.local(lo, span), lw.local(hi, span));
                let zero = lw.int_lit(0, span);
                let n = lw.mk(ExprKind::DomainLen(d), Ty::Int, span);
                let below = lw.mk(
                    ExprKind::Cmp(CmpOp::Lt, Box::new(l), Box::new(zero)),
                    Ty::Bool,
                    span,
                );
                let above = lw.mk(
                    ExprKind::Cmp(CmpOp::Gt, Box::new(h), Box::new(n)),
                    Ty::Bool,
                    span,
                );
                let outside = lw.mk(
                    ExprKind::Binary(BinOp::Or, Box::new(below), Box::new(above)),
                    Ty::Bool,
                    span,
                );
                let trap = Stmt::Trap(
                    "the range reaches outside the domain of `V.ranges`".into(),
                    lw.span(span),
                );
                out.push(Stmt::If(
                    outside,
                    Block { stmts: vec![trap] },
                    Block::default(),
                ));
            }
            let last = lw.local(hi, span);
            let last = lw.int_add(last, -1);
            let first = lw.local(lo, span);
            let idx = lw.mk(
                ExprKind::Builtin(Builtin::SymIdx, vec![first, last]),
                Ty::Int,
                span,
            );
            out.push(Stmt::Give(idx));
        });
        self.cast(Cast::Raw, sym, Ty::Int)
    }
}
