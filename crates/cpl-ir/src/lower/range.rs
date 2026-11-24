use super::*;

/// Inclusive integer bounds `[lo, hi]` declared on a range-typed place
/// (scalar variable, struct field, or array/list element).
#[derive(Clone, Debug)]
pub(super) struct RangeBounds {
    pub lo: Option<Expr>,
    pub hi: Option<Expr>,
    pub lo_const: Option<i64>,
    pub hi_const: Option<i64>,
    /// The least `lo` and greatest `hi` can be when they are runtime
    /// values of proven range: `0..n` with `n: 1..=300_000` stays below
    /// 300_000, so its values fit 32 bits.
    pub lo_min: Option<i64>,
    pub hi_max: Option<i64>,
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
    /// A map whose keys or values carry `RangeInfo` (e.g. `HashMap<0..N, 0..=1e6>`).
    Map(Option<Box<RangeInfo>>, Option<Box<RangeInfo>>),
    /// A key of `V.ranges`: a range that is non-empty by its type, so its
    /// `first` and `last` are positions, not optionals (lib/search-dp.md).
    Piece,
}

impl RangeBounds {
    /// Bounds known only as limits, `lo..=hi`: enough for a storage width.
    pub(super) fn spanning(lo: i64, hi: i64) -> RangeBounds {
        RangeBounds {
            lo: None,
            hi: None,
            lo_const: None,
            hi_const: None,
            lo_min: Some(lo),
            hi_max: Some(hi),
            text: format!("{lo}..={hi}"),
        }
    }

    /// `??lo..hi` inside an `int32_t`: two values below the range are the two `none`s.
    pub(super) fn narrow_opt_opt_layout(&self) -> Option<NarrowLayout> {
        let (lo_min, hi_max) = self.limits()?;
        (lo_min > i64::from(i32::MIN) + 1 && hi_max <= i64::from(i32::MAX))
            .then_some(NarrowLayout::OptOptI32)
    }

    pub(super) fn narrow_opt_layout(&self) -> Option<NarrowLayout> {
        let (lo_min, hi_max) = self.limits()?;
        (lo_min > i64::from(i32::MIN) && hi_max <= i64::from(i32::MAX))
            .then_some(NarrowLayout::OptI32)
    }

    pub(super) fn limits(&self) -> Option<(i64, i64)> {
        let lo_min = self
            .lo_const
            .or(self.lo_min)
            .or_else(|| self.lo.as_ref().and_then(expr_limits).map(|(lo, _)| lo))?;
        let hi_max = self
            .hi_const
            .or(self.hi_max)
            .or_else(|| self.hi.as_ref().and_then(expr_limits).map(|(_, hi)| hi))?;
        (lo_min <= hi_max).then_some((lo_min, hi_max))
    }

    pub(super) fn narrow_width(&self) -> Option<IntWidth> {
        let lo_min = self
            .lo_const
            .or(self.lo_min)
            .or_else(|| self.lo.as_ref().and_then(expr_limits).map(|(lo, _)| lo))?;
        let hi_max = self
            .hi_const
            .or(self.hi_max)
            .or_else(|| self.hi.as_ref().and_then(expr_limits).map(|(_, hi)| hi))?;
        IntWidth::of(lo_min, hi_max)
    }
}

/// `ty` as the judge's grader holds it, from its declared range `info`
/// (R10.1.4): `int` unless the range is past it, so a value never
/// narrows on the way; `Err` is the spelling of a range with an open end.
/// Why a declared type can't cross to the judge's grader.
pub(super) enum Uncrossable {
    /// A range with an open end, as spelled.
    Open(String),
    /// A type the grader has no spelling for.
    Type(Ty),
}

/// The grader's side of `ty`, declared with ranges `info`.
pub(super) fn judge_ty(ty: &Ty, info: Option<&RangeInfo>) -> Result<JudgeTy, Uncrossable> {
    side(ty, info).map_err(|why| match why {
        Uncrossable::Type(_) => Uncrossable::Type(ty.clone()),
        open => open,
    })
}

fn side(ty: &Ty, info: Option<&RangeInfo>) -> Result<JudgeTy, Uncrossable> {
    match (ty, info) {
        (Ty::Int, Some(RangeInfo::Scalar(b))) => {
            let open = |e: &Option<Expr>, c: Option<i64>| e.is_none() && c.is_none();
            if open(&b.lo, b.lo_const) || open(&b.hi, b.hi_const) {
                return Err(Uncrossable::Open(b.text.clone()));
            }
            let int = i64::from(i32::MIN)..=i64::from(i32::MAX);
            Ok(match b.limits() {
                Some((lo, hi)) if !int.contains(&lo) || !int.contains(&hi) => JudgeTy::Long,
                _ => JudgeTy::Int,
            })
        }
        (Ty::Int, _) => Ok(JudgeTy::Int),
        (Ty::Unit, _) => Ok(JudgeTy::Void),
        (Ty::Float, _) => Ok(JudgeTy::Double),
        (Ty::Bool, _) => Ok(JudgeTy::Bool),
        (Ty::Char, _) => Ok(JudgeTy::Char),
        (Ty::Str, _) => Ok(JudgeTy::Str),
        (Ty::List(elem), info) if **elem != Ty::Unit => {
            let inner = match info {
                Some(RangeInfo::Elem(e)) => Some(&**e),
                _ => None,
            };
            Ok(JudgeTy::List(Box::new(side(elem, inner)?)))
        }
        (Ty::Tuple(items), info) if !items.contains(&Ty::Unit) => {
            let at = |i: usize| match info {
                Some(RangeInfo::Tuple(infos)) => infos.get(i).and_then(Option::as_ref),
                _ => None,
            };
            let items = items.iter().enumerate().map(|(i, t)| side(t, at(i)));
            Ok(JudgeTy::Tuple(items.collect::<Result<_, _>>()?))
        }
        (ty, _) => Err(Uncrossable::Type(ty.clone())),
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
    /// A range holding the values of both `self` and `other`: what a list
    /// built from two lists' elements knows of its own.
    pub(super) fn join(&self, other: &RangeInfo) -> Option<RangeInfo> {
        match (self, other) {
            (RangeInfo::Scalar(a), RangeInfo::Scalar(b)) => {
                let ((alo, ahi), (blo, bhi)) = (a.limits()?, b.limits()?);
                Some(RangeInfo::Scalar(RangeBounds::spanning(
                    alo.min(blo),
                    ahi.max(bhi),
                )))
            }
            (RangeInfo::Opt(a), RangeInfo::Opt(b)) => Some(RangeInfo::Opt(Box::new(a.join(b)?))),
            (RangeInfo::Elem(a), RangeInfo::Elem(b)) => Some(RangeInfo::Elem(Box::new(a.join(b)?))),
            (RangeInfo::Tuple(a), RangeInfo::Tuple(b)) if a.len() == b.len() => {
                let items: Vec<Option<RangeInfo>> = a
                    .iter()
                    .zip(b)
                    .map(|(x, y)| x.as_ref().zip(y.as_ref()).and_then(|(x, y)| x.join(y)))
                    .collect();
                items
                    .iter()
                    .any(Option::is_some)
                    .then_some(RangeInfo::Tuple(items))
            }
            _ => None,
        }
    }

    /// A scalar range as the [`Spec::Int`] the IR carries.
    pub(super) fn spec<L>(&self) -> Option<Spec<L>> {
        let b = self.scalar()?;
        Some(Spec::Int {
            lo: b.lo.clone().map(Box::new),
            hi: b.hi.clone().map(Box::new),
            text: b.text.clone(),
        })
    }

    /// A sequence's cell range and how many indexes reach it: `[V][W]0..5`
    /// is `(2, 0..5)`.
    pub(super) fn cell_spec(&self) -> Option<(u8, Spec)> {
        let mut info = self.elem()?;
        let mut depth = 1u8;
        while let Some(inner) = info.elem() {
            info = inner;
            depth += 1;
        }
        Some((depth, info.spec()?))
    }

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
            (RangeInfo::Opt(inner), Ty::Opt(t)) if **t == Ty::opt(Ty::Int) => {
                match inner.as_ref() {
                    RangeInfo::Opt(held) => held.scalar()?.narrow_opt_opt_layout(),
                    _ => None,
                }
            }
            (RangeInfo::Tuple(items), Ty::Tuple(_) | Ty::Record(_)) => {
                let tys: Vec<&Ty> = match ty {
                    Ty::Tuple(ts) => ts.iter().collect(),
                    Ty::Record(fs) => fs.iter().map(|(_, t)| t).collect(),
                    _ => return None,
                };
                if tys.is_empty() || items.len() != tys.len() {
                    return None;
                }
                let mut fields = Vec::with_capacity(tys.len());
                for (item, t) in items.iter().zip(tys) {
                    let width = (*t == Ty::Int)
                        .then(|| item.as_ref()?.scalar()?.narrow_width())
                        .flatten();
                    fields.push(match width {
                        Some(w) => FieldLayout::Int(w),
                        None if t.is_scalar() && *t != Ty::Unit => FieldLayout::Kept(t.clone()),
                        None => return None,
                    });
                }
                fields
                    .iter()
                    .any(|f| matches!(f, FieldLayout::Int(_)))
                    .then_some(NarrowLayout::Tuple(fields))
            }
            (RangeInfo::Map(..), Ty::Map(..) | Ty::HashMap(..)) => {
                Some(NarrowLayout::Seq(Box::new(self.narrow_elem_layout(ty)?)))
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
        if let (RangeInfo::Map(k, v), Ty::Map(kt, vt) | Ty::HashMap(kt, vt)) = (self, container_ty)
        {
            let width = |info: &Option<Box<RangeInfo>>, ty: &Ty| match (info.as_deref(), ty) {
                (Some(RangeInfo::Scalar(b)), Ty::Int) => b.narrow_width(),
                _ => None,
            };
            let (k, v) = (width(k, kt), width(v, vt));
            return (k.is_some() || v.is_some()).then_some(NarrowLayout::Map(k, v));
        }
        let elem_ty = match container_ty {
            Ty::Array { elem, .. }
            | Ty::List(elem)
            | Ty::ListView(elem, _)
            | Ty::Keyed(elem, _) => elem.as_ref(),
            _ => return None,
        };
        self.elem()?.narrow_value_layout(elem_ty)
    }

    pub(super) fn from_spec(spec: &ReadSpec) -> Option<Self> {
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
                    lo_min: None,
                    hi_max: None,
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
        Ty::Array { .. }
            | Ty::List(_)
            | Ty::ListView(_, _)
            | Ty::Keyed(_, _)
            | Ty::Map(..)
            | Ty::HashMap(..)
    )
}

impl<'a> Lowerer<'a> {
    pub(super) fn narrow_range_width(&self, ty: &ast::Type) -> Option<IntWidth> {
        self.static_range_limits(ty)
            .and_then(|(lo, hi)| IntWidth::of(lo, hi))
    }

    /// The ends of a range type known while lowering, through a local alias.
    fn static_range_limits(&self, ty: &ast::Type) -> Option<(i64, i64)> {
        // `type Sign = -1..=1`: a field of the alias narrows as its range would.
        if let ast::TypeKind::Named { name, args } = &ty.kind {
            // A generic parameter bound to a range keeps its bounds in the instance.
            if let (true, Some(GenArg::Range(Some(lo), Some(hi)))) =
                (args.is_empty(), self.gen_env.get(&name.text))
            {
                return Some((*lo, *hi));
            }
            return self
                .local_alias_target(ty)
                .and_then(|aliased| self.static_range_limits(aliased));
        }
        let ast::TypeKind::Range {
            lo: Some(lo),
            hi: Some(hi),
            inclusive,
        } = &ty.kind
        else {
            return None;
        };
        let (lo_min, _) = self.range_bound_limits(lo)?;
        let (_, hi_raw) = self.range_bound_limits(hi)?;
        let hi_max = if *inclusive {
            hi_raw
        } else {
            hi_raw.checked_sub(1)?
        };
        Some((lo_min, hi_max))
    }

    /// The ends of the range a `?range` type holds: an alias of one, or a
    /// generic parameter bound to one.
    fn static_opt_range_limits(&self, ty: &ast::Type) -> Option<(i64, i64)> {
        match &ty.kind {
            ast::TypeKind::Optional(inner) => self.static_range_limits(inner),
            ast::TypeKind::Named { name, args } if args.is_empty() => {
                if let Some(GenArg::OptRange(Some(lo), Some(hi))) = self.gen_env.get(&name.text) {
                    return Some((*lo, *hi));
                }
                self.local_alias_target(ty)
                    .and_then(|aliased| self.static_opt_range_limits(aliased))
            }
            _ => None,
        }
    }

    /// The ends of `decl`'s scalar ranged `Int` fields known while lowering.
    pub(super) fn int_field_ranges(
        &self,
        decl: &ast::StructDecl,
    ) -> std::collections::BTreeMap<usize, (i64, i64)> {
        let mut ranges = std::collections::BTreeMap::new();
        ranges.extend(
            (decl.fields.iter().enumerate())
                .filter(|(_, f)| match &f.ty.kind {
                    ast::TypeKind::Range { .. } => true,
                    ast::TypeKind::Named { args, .. } => args.is_empty(),
                    _ => false,
                })
                .filter_map(|(at, f)| Some((at, self.static_range_limits(&f.ty)?))),
        );
        ranges
    }

    /// The ranged `Int`, `?Int` and 1D ranged-`Int` array/list fields of `decl` whose bounds fit in 32 bits.
    pub(super) fn narrow_int_fields(
        &self,
        decl: &ast::StructDecl,
    ) -> std::collections::BTreeMap<usize, IntWidth> {
        let mut narrow = std::collections::BTreeMap::new();
        for (at, field) in decl.fields.iter().enumerate() {
            let width = match &field.ty.kind {
                ast::TypeKind::Range { .. } => self.narrow_range_width(&field.ty),
                ast::TypeKind::Named { args, .. } if args.is_empty() => {
                    self.narrow_range_width(&field.ty).or_else(|| {
                        (self.static_opt_range_limits(&field.ty))
                            .filter(|&(lo, hi)| {
                                lo > i64::from(i32::MIN) && hi <= i64::from(i32::MAX)
                            })
                            .map(|_| IntWidth::I32)
                    })
                }
                ast::TypeKind::Array { index, elem } if index.len() == 1 => {
                    self.narrow_range_width(elem)
                }
                ast::TypeKind::Named { name, args }
                    if (name.text == "List" || name.text == "ListView") && args.len() == 1 =>
                {
                    self.narrow_range_width(&args[0])
                }
                // `?lo..hi` inside an `int32_t` above INT32_MIN, which is `none`;
                // `??lo..hi` above INT32_MIN + 1, the inner `none`.
                ast::TypeKind::Optional(inner) => (self.static_range_limits(inner))
                    .filter(|&(lo, hi)| lo > i64::from(i32::MIN) && hi <= i64::from(i32::MAX))
                    .or_else(|| {
                        self.static_opt_range_limits(inner).filter(|&(lo, hi)| {
                            lo > i64::from(i32::MIN) + 1 && hi <= i64::from(i32::MAX)
                        })
                    })
                    .map(|_| IntWidth::I32),
                _ => None,
            };
            if let Some(w) = width {
                narrow.insert(at, w);
            }
        }
        narrow
    }

    pub(super) fn record_narrow_locals(
        &mut self,
        scope: ScopeId,
        locals: &[Local],
        ranges: &HashMap<Slot, RangeInfo>,
    ) {
        for (&slot, info) in ranges {
            if let Some(cell) = info.cell_spec() {
                self.cell_ranges.insert(ElemLoc::Local(scope, slot), cell);
            }
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
                let lo_min = lo_expr
                    .as_ref()
                    .and_then(|e| self.bound_limits(e))
                    .map(|l| l.0);
                let hi_max = hi_expr
                    .as_ref()
                    .and_then(|e| self.bound_limits(e))
                    .map(|l| l.1);
                Some(RangeInfo::Scalar(RangeBounds {
                    lo: lo_expr,
                    hi: hi_expr,
                    lo_const,
                    hi_const,
                    lo_min,
                    hi_max,
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
                if args.len() == 2 && self.get_sym(&name.text).is_none() && {
                    let saved = self.diags.len();
                    let resolved = self.resolve_ty(ty);
                    self.diags.truncate(saved);
                    matches!(resolved, Ty::Map(..) | Ty::HashMap(..))
                } {
                    let k = self.range_info_from_ast(&args[0]).map(Box::new);
                    let v = self.range_info_from_ast(&args[1]).map(Box::new);
                    return (k.is_some() || v.is_some()).then_some(RangeInfo::Map(k, v));
                }
                if args.is_empty() {
                    if let Some(GenArg::Range(lo, hi)) = self.gen_env.get(&name.text).cloned() {
                        return Some(RangeInfo::Scalar(self.const_bounds(lo, hi, ty.span)));
                    }
                    if let Some(GenArg::OptRange(lo, hi)) = self.gen_env.get(&name.text).cloned() {
                        let held = RangeInfo::Scalar(self.const_bounds(lo, hi, ty.span));
                        return Some(RangeInfo::Opt(Box::new(held)));
                    }
                    if let Some(Sym::Alias(aid)) = self.get_sym(&name.text) {
                        return self.in_alias(aid, |lw, ty| lw.range_info_from_ast(ty));
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
        self.expr_as(e, &Ty::Int)
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
                    // A field typed by an alias from another module, such as
                    // `n: Len`, gets the bounds its checked stores hold.
                    if let RangeInfo::Scalar(RangeBounds {
                        lo_min: Some(lo),
                        hi_max: Some(hi),
                        ..
                    }) = info
                    {
                        (self.structs[id].def.int_ranges)
                            .entry(idx)
                            .or_insert((lo, hi));
                    }
                    let ranges = &mut self.structs[id].field_ranges;
                    if ranges.len() <= idx {
                        ranges.resize(idx + 1, None);
                    }
                    ranges[idx] = Some(info);
                }
            }
        }
        self.cx = saved_cx;
        self.outer_cxs = saved_outer;
    }

    fn field_range(&self, id: StructId, idx: usize) -> Option<&RangeInfo> {
        self.structs.get(id)?.field_ranges.get(idx)?.as_ref()
    }

    /// Looks up `RangeInfo` for field `idx` of struct `id`, falling back to
    /// resolving the field's AST type in the current scope for dynamic bounds
    /// (e.g. `struct Pt(r: 1..=n, c: 1..=m)`).
    pub(super) fn struct_field_range(&mut self, id: StructId, idx: usize) -> Option<RangeInfo> {
        if let Some(info) = self.field_range(id, idx) {
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

    /// The range of fn `id`'s param declared `ty`, read in `id`'s own
    /// module, when its ends are constants and so mean the same at the call.
    /// The range a parameter's declared cells hold (`xs: List<-5..=5>`),
    /// when their bounds are constant.
    pub(super) fn const_param_cells(&mut self, id: FnId, ty: &ast::Type) -> Option<RangeInfo> {
        let ast::TypeKind::Named { name, args } = &ty.kind else {
            return None;
        };
        if name.text != "List" || args.len() != 1 {
            return None;
        }
        match self.const_param_cells(id, &args[0]) {
            Some(inner) => Some(RangeInfo::Elem(Box::new(inner))),
            None => self
                .const_param_range(id, &args[0])
                .map(|b| RangeInfo::Elem(Box::new(RangeInfo::Scalar(b)))),
        }
    }

    /// Checks each cell of a list literal against the range its destination
    /// declares (`first([x])` for `xs: List<-5..=5>`): a literal's cells are
    /// stored as given, so each is checked as a scalar would be.
    pub(super) fn check_fresh_cells(
        &mut self,
        value: Expr,
        info: &RangeInfo,
        written: &ast::Expr,
    ) -> Expr {
        let RangeInfo::Elem(inner) = info else {
            return value;
        };
        let cells = match &written.kind {
            ast::ExprKind::List(cells) => &cells[..],
            _ => &[],
        };
        match value.kind {
            ExprKind::Tuple(items)
                if matches!(value.ty, Ty::List(_)) && items.len() == cells.len() =>
            {
                let items = items
                    .into_iter()
                    .zip(cells)
                    .map(|(item, cell)| match inner.as_ref() {
                        RangeInfo::Scalar(b) => self.check_range(item, b, cell.span),
                        nested => self.check_fresh_cells(item, nested, cell),
                    })
                    .collect();
                Expr {
                    kind: ExprKind::Tuple(items),
                    ..value
                }
            }
            kind => Expr { kind, ..value },
        }
    }

    pub(super) fn const_param_range(&mut self, id: FnId, ty: &ast::Type) -> Option<RangeBounds> {
        let saved_m = std::mem::replace(&mut self.cur_mod, self.sigs[id].mod_id);
        let saved_off = std::mem::replace(&mut self.offset, self.sigs[id].offset);
        let saved_env = std::mem::replace(&mut self.gen_env, self.sigs[id].env.clone());
        let saved_diags = self.diags.len();
        let info = self.range_info_from_ast(ty);
        let had_err = self.diags.len() > saved_diags;
        self.diags.truncate(saved_diags);
        self.gen_env = saved_env;
        self.cur_mod = saved_m;
        self.offset = saved_off;
        let b = info.filter(|_| !had_err)?.scalar()?.clone();
        let fixed =
            (b.lo.is_none() || b.lo_const.is_some()) && (b.hi.is_none() || b.hi_const.is_some());
        fixed.then_some(b)
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
                RangeInfo::Map(_, value) => value.map(|v| *v),
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

    /// The least and greatest a range bound can be: a constant, or a
    /// local or global of proven range, through `+`, `-` and negation.
    pub(super) fn bound_limits(&self, e: &Expr) -> Option<(i64, i64)> {
        match &e.kind {
            ExprKind::Local(slot) => match self.expr_range(e) {
                Some(r) => r.scalar()?.limits(),
                None => self.cx.let_bounds.get(slot).copied(),
            },
            ExprKind::Global(gid) => match self.expr_range(e) {
                Some(r) => r.scalar()?.limits(),
                None => self.global_bounds.get(gid).copied(),
            },
            ExprKind::Unary(UnOp::Neg, inner) => {
                let (lo, hi) = self.bound_limits(inner)?;
                Some((hi.checked_neg()?, lo.checked_neg()?))
            }
            ExprKind::Binary(BinOp::Add, a, b) => {
                let ((alo, ahi), (blo, bhi)) = (self.bound_limits(a)?, self.bound_limits(b)?);
                Some((alo.checked_add(blo)?, ahi.checked_add(bhi)?))
            }
            ExprKind::Binary(BinOp::Sub, a, b) => {
                let ((alo, ahi), (blo, bhi)) = (self.bound_limits(a)?, self.bound_limits(b)?);
                Some((alo.checked_sub(bhi)?, ahi.checked_sub(blo)?))
            }
            ExprKind::RangeCheck { value, .. } => self.bound_limits(value),
            _ => match self.expr_const_bounds(e) {
                (Some(lo), Some(hi)) => Some((lo, hi)),
                _ => expr_limits(e),
            },
        }
    }

    /// The values an `Int` loop over `lo..hi` gives its variable.
    pub(super) fn loop_var_range(&self, head: &Head) -> Option<RangeInfo> {
        let Head::Iter(Iter::Range { lo, hi, closed, .. }) = head else {
            return None;
        };
        let (lo_min, _) = self.bound_limits(lo)?;
        let (_, hi_max) = self.bound_limits(hi)?;
        let hi_max = if *closed {
            hi_max
        } else {
            hi_max.checked_sub(1)?
        };
        (lo_min <= hi_max).then(|| RangeInfo::Scalar(RangeBounds::spanning(lo_min, hi_max)))
    }

    /// Looks up `RangeInfo` for a value expression (used to prove sub-range sources).
    pub(super) fn expr_range(&self, e: &Expr) -> Option<RangeInfo> {
        match &e.kind {
            ExprKind::Local(slot) => self.cx.local_ranges.get(slot).cloned(),
            ExprKind::Global(gid) => self.global_ranges.get(gid).cloned(),
            ExprKind::Field(base, idx) => {
                if let Ty::Struct(sid) = base.ty {
                    return self.field_range(sid, *idx).cloned();
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
                Builtin::ListSlice
                | Builtin::KeySlice
                | Builtin::Rekey
                | Builtin::GridLine
                | Builtin::ToList,
                args,
            ) => args.first().and_then(|a| self.expr_range(a)),
            ExprKind::Builtin(Builtin::Sorted, args) if matches!(e.ty, Ty::List(_)) => {
                args.first().and_then(|a| self.expr_range(a))
            }
            ExprKind::Tuple(items) if matches!(e.ty, Ty::Tuple(_) | Ty::Record(_)) => {
                let items: Vec<Option<RangeInfo>> =
                    items.iter().map(|i| self.expr_range(i)).collect();
                items
                    .iter()
                    .any(Option::is_some)
                    .then_some(RangeInfo::Tuple(items))
            }
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
            // A position or bound is at most its domain's length.
            ExprKind::Cast(Cast::Ord, _) => (Some(0), Some(i64::from(i32::MAX))),
            ExprKind::Binary(BinOp::Rem, _, rhs) => {
                if let (Some(lo), Some(hi)) = self.expr_const_bounds(rhs) {
                    if lo == hi && lo > 0 {
                        return (Some(0), Some(lo - 1));
                    }
                }
                (None, None)
            }
            // Non-negative operands: no bit above the larger one's top bit.
            ExprKind::Binary(BinOp::BitXor | BinOp::BitOr, a, b) => {
                match (self.expr_const_bounds(a), self.expr_const_bounds(b)) {
                    ((Some(alo), Some(ahi)), (Some(blo), Some(bhi))) if alo >= 0 && blo >= 0 => {
                        let top = ahi.max(bhi);
                        (
                            Some(0),
                            Some(u64::MAX.checked_shr(top.leading_zeros()).unwrap_or(0) as i64),
                        )
                    }
                    _ => (None, None),
                }
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

    /// The bounds of a range a type parameter is bound to: `V = 0..=10`.
    pub(super) fn const_bounds(
        &mut self,
        lo: Option<i64>,
        hi: Option<i64>,
        span: Span,
    ) -> RangeBounds {
        RangeBounds {
            lo: lo.map(|v| self.int_lit(v, span)),
            hi: hi.map(|v| self.int_lit(v, span)),
            lo_const: lo,
            hi_const: hi,
            lo_min: lo,
            hi_max: hi,
            text: format!(
                "{}..={}",
                lo.map_or(String::new(), |v| v.to_string()),
                hi.map_or(String::new(), |v| v.to_string())
            ),
        }
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
                arg: false,
            },
            Ty::Int,
            span,
        )
    }

    /// [`Self::check_range`] for an argument of a fn's range param, a check
    /// every build that keeps traps keeps.
    pub(super) fn check_arg(&mut self, value: Expr, bounds: &RangeBounds, span: Span) -> Expr {
        let mut checked = self.check_range(value, bounds, span);
        if let ExprKind::RangeCheck { arg, .. } = &mut checked.kind {
            *arg = true;
        }
        checked
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
        let value = self.keys_like_place(&place, op, value, span);
        if matches!(op, AssignOp::Max | AssignOp::Min) && self.user_ordered(&place.ty) {
            self.keep_by_order(place, op == AssignOp::Max, value, span, out);
            return;
        }
        if let (Ty::Opt(_), AssignOp::Set, ExprKind::Some(_)) = (&place.ty, op, &value.kind) {
            if let Some(RangeInfo::Opt(inner)) = self.place_range(&place) {
                if let (
                    RangeInfo::Scalar(bounds),
                    Expr {
                        kind: ExprKind::Some(x),
                        ty,
                        span: at,
                    },
                ) = (*inner, &value)
                {
                    let (ty, at) = (ty.clone(), *at);
                    let x = self.check_range((**x).clone(), &bounds, span);
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
            // `m[k] += d` starts an absent key at zero and `m[k] min= v` at `v`, so write first
            // and check what it holds.
            AssignOp::Binary(_) | AssignOp::Max | AssignOp::Min if matches!(&place.kind, PlaceKind::Index(base, _) if matches!(base.ty, Ty::Map(..) | Ty::HashMap(..))) =>
            {
                let held = self.check_range(super::stmt::place_to_expr(&place), &bounds, span);
                out.push(Stmt::Assign(place, op, value));
                if matches!(held.kind, ExprKind::RangeCheck { .. }) {
                    out.push(Stmt::Expr(held));
                }
            }
            AssignOp::Binary(bin) => {
                let cur = super::stmt::place_to_expr(&place);
                let next = self.mk_binary(bin, cur, value, Ty::Int, span);
                let checked = self.check_range(next, &bounds, span);
                out.push(Stmt::Assign(place, AssignOp::Set, checked));
            }
            AssignOp::Max | AssignOp::Min => {
                if self.proven_in_range(&value, &bounds) {
                    out.push(Stmt::Assign(place, op, value));
                    return;
                }
                let slot = self.temp(Ty::Int);
                out.push_let(slot, value);
                let rhs = self.local(slot, span);
                let cur = super::stmt::place_to_expr(&place);
                let cmp = if op == AssignOp::Max {
                    CmpOp::Gt
                } else {
                    CmpOp::Lt
                };
                let cond = self.mk_cmp(cmp, rhs.clone(), cur, Ty::Bool, span);
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
    /// The cell of the non-empty range `piece` in a `[V.ranges]T`; an empty range traps, and so
    /// does a `Range<Int>` reaching outside `V`.
    pub(super) fn piece_idx(&mut self, piece: Expr, d: DomainId, span: Span) -> Expr {
        let ty = piece.ty.clone();
        let of_ints = matches!(&ty, Ty::Range(inner) if **inner == Ty::Int);
        // A `V.ranges` key is non-empty by its type, so its cell needs no check.
        let proven = !of_ints && matches!(self.expr_range(&piece), Some(RangeInfo::Piece));
        let sym = self.do_expr(Some(&Ty::Int), span, |lw, out| {
            // The ends straight from a range literal or a local, with no copy of the range.
            let (lo_e, hi_e) = match piece.kind {
                ExprKind::Range { lo, hi } if lo.ty == Ty::Int && hi.ty == Ty::Int => (*lo, *hi),
                kind => {
                    let mut value = Expr { kind, ..piece };
                    if !matches!(value.kind, ExprKind::Local(_)) {
                        let r = lw.new_slot("piece", ty);
                        out.push_let(r, value);
                        value = lw.local(r, span);
                    }
                    let lo = lw.mk_builtin(Builtin::RangeLo, [value.clone()], Ty::Int, span);
                    (lo, lw.mk_builtin(Builtin::RangeHi, [value], Ty::Int, span))
                }
            };
            let (lo, hi) = (lw.new_slot("lo", Ty::Int), lw.new_slot("hi", Ty::Int));
            out.push_let(lo, lo_e);
            out.push_let(hi, hi_e);
            if !proven {
                let (l, h) = (lw.local(lo, span), lw.local(hi, span));
                let empty = lw.mk_cmp(CmpOp::Le, h, l, Ty::Bool, span);
                let trap = Stmt::Trap(
                    "an empty range has no cell in `V.ranges`".into(),
                    lw.span(span),
                );
                out.push(Stmt::If(
                    empty,
                    Block { stmts: vec![trap] },
                    Block::default(),
                ));
            }
            if of_ints {
                let (l, h) = (lw.local(lo, span), lw.local(hi, span));
                let zero = lw.int_lit(0, span);
                let n = lw.mk(ExprKind::DomainLen(d), Ty::Int, span);
                let below = lw.mk_cmp(CmpOp::Lt, l, zero, Ty::Bool, span);
                let above = lw.mk_cmp(CmpOp::Gt, h, n, Ty::Bool, span);
                let outside = lw.mk_binary(BinOp::Or, below, above, Ty::Bool, span);
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
            let idx = lw.mk_builtin(Builtin::TriIdx, [first, last], Ty::Int, span);
            out.push_give(idx);
        });
        self.cast(Cast::Raw, sym, Ty::Int)
    }
}

impl<'a> Lowerer<'a> {
    /// Whether `ty`'s order calls a user `Ord` somewhere inside it.
    pub(super) fn user_ordered(&self, ty: &Ty) -> bool {
        match ty {
            Ty::Struct(id) => {
                let def = &self.structs[*id].def;
                def.ord.is_some() || def.fields.iter().any(|(_, t)| self.user_ordered(t))
            }
            Ty::Tuple(items) => items.iter().any(|t| self.user_ordered(t)),
            Ty::Record(fields) => fields.iter().any(|(_, t)| self.user_ordered(t)),
            Ty::List(elem) | Ty::Opt(elem) => self.user_ordered(elem),
            _ => false,
        }
    }

    /// `place max= value` on a type ordered by a user `Ord`: the order is a
    /// call, so it is `if value > place: place = value`, as the C++ reads it.
    fn keep_by_order(
        &mut self,
        place: Place,
        is_max: bool,
        value: Expr,
        span: Span,
        out: &mut Vec<Stmt>,
    ) {
        let slot = self.temp(place.ty.clone());
        out.push_let(slot, value);
        let held = self.local(slot, span);
        let op = if is_max { CmpOp::Gt } else { CmpOp::Lt };
        let current = super::stmt::place_to_expr(&place);
        let better = self.mk_cmp(op, held.clone(), current, Ty::Bool, span);
        let set = Stmt::Assign(place, AssignOp::Set, held);
        out.push(Stmt::If(
            better,
            Block { stmts: vec![set] },
            Block::default(),
        ));
    }
}

impl<'a> Lowerer<'a> {
    /// `(lo..hi).step(k)` written out walks as a counted loop, not a built list.
    pub(super) fn stepped_range_iter(&mut self, e: &ast::Expr) -> Option<(Iter, Ty)> {
        if let ast::ExprKind::Call { callee, args } = &e.kind {
            if let (ast::ExprKind::Field { base, name }, [k]) = (&callee.kind, args.as_slice()) {
                let mut range = &**base;
                while let ast::ExprKind::Paren(inner) = &range.kind {
                    range = inner;
                }
                if name.text == "step"
                    && k.name.is_none()
                    && !k.mutable
                    && matches!(
                        range.kind,
                        ast::ExprKind::Range {
                            lo: Some(_),
                            hi: Some(_),
                            step: None,
                            ..
                        }
                    )
                    && self.probe_is(
                        range,
                        |t| matches!(t, Ty::Range(inner) if **inner == Ty::Int),
                    )
                {
                    let (
                        Head::Iter(Iter::Range {
                            lo,
                            hi,
                            step: None,
                            rev: false,
                            closed,
                        }),
                        ty,
                    ) = self.iter(range)
                    else {
                        return None;
                    };
                    let step = self.expr_as(&k.value, &Ty::Int);
                    return Some((
                        Iter::Range {
                            lo,
                            hi,
                            step: Some(step),
                            rev: false,
                            closed,
                        },
                        ty,
                    ));
                }
            }
        }
        None
    }
}

/// Slices `a[lo..hi]` made from their bounds: views of lists, strings and arrays, sub-ranges of ranges.
impl<'a> Lowerer<'a> {
    /// `base[lo..hi]` in key terms on a view and positions otherwise.
    pub(super) fn slice_builtin(&mut self, base: Expr, lo: Expr, hi: Expr, span: Span) -> Expr {
        let keyed = Self::keyed_view(&base.ty);
        let (op, ty) = match &base.ty {
            Ty::Str | Ty::StrView => (
                if keyed {
                    Builtin::KeySlice
                } else {
                    Builtin::StrSlice
                },
                Ty::StrView,
            ),
            Ty::List(e) | Ty::ListView(e, _) | Ty::Keyed(e, _) | Ty::MutView(e, _) => (
                if keyed {
                    Builtin::KeySlice
                } else {
                    Builtin::ListSlice
                },
                Ty::ListView(e.clone(), base.ty.seq_key().unwrap_or(Dim::Int)),
            ),
            // R04b.1.2: a slice of an array keeps its positions as keys.
            Ty::Array { elem, dims } => (Builtin::KeySlice, Ty::ListView(elem.clone(), dims[0])),
            Ty::Error => return self.err_expr(span),
            other => {
                let shown = self.show(other);
                self.report(
                    span,
                    cpl_diag::msgs::ir::BAD_OPERANDS_CAN_T_SLICE_A,
                    &[&shown],
                );
                return self.err_expr(span);
            }
        };
        self.mk_builtin(op, [base, lo, hi], ty, span)
    }

    /// `r[lo..hi]` of a `Range`: the keys `lo..hi`, which must lie inside `r`.
    pub(super) fn slice_range(
        &mut self,
        base: Expr,
        lo: Option<&ast::Expr>,
        hi: Option<&ast::Expr>,
        inclusive: bool,
        span: Span,
    ) -> Expr {
        let range_ty = base.ty.clone();
        let elem = range_ty.seq_elem().unwrap_or(Ty::Int);
        self.do_expr(Some(&range_ty), span, |lw, out| {
            let src_slot = lw.temp(range_ty.clone());
            out.push_let(src_slot, base);
            let src = lw.local(src_slot, span);
            let src_lo = lw.mk_builtin(Builtin::RangeLo, [src.clone()], Ty::Int, span);
            let src_hi = lw.mk_builtin(Builtin::RangeHi, [src], Ty::Int, span);
            let key = |lw: &mut Self, e: &ast::Expr, is_end: bool| -> Expr {
                let v = lw.expr(e, Some(&elem));
                let v = match v.ty {
                    Ty::Int => v,
                    Ty::Pos(_) => lw.cast(Cast::Ord, v, Ty::Int),
                    _ => lw.coerce(v, &Ty::Int, e.span),
                };
                lw.int_add(v, i64::from(is_end && inclusive))
            };
            let lo_e = match lo {
                Some(l) => key(lw, l, false),
                None => src_lo.clone(),
            };
            let hi_e = match hi {
                Some(h) => key(lw, h, true),
                None => src_hi.clone(),
            };
            let (lo_slot, hi_slot) = (lw.temp(Ty::Int), lw.temp(Ty::Int));
            out.push_let(lo_slot, lo_e);
            out.push_let(hi_slot, hi_e);
            let (lo_e, hi_e) = (lw.local(lo_slot, span), lw.local(hi_slot, span));
            let cmp = |lw: &mut Self, op: CmpOp, l: &Expr, r: &Expr| {
                lw.mk_cmp(op, l.clone(), r.clone(), Ty::Bool, span)
            };
            let or =
                |lw: &mut Self, l: Expr, r: Expr| lw.mk_binary(BinOp::Or, l, r, Ty::Bool, span);
            let below = cmp(lw, CmpOp::Lt, &lo_e, &src_lo);
            let reversed = cmp(lw, CmpOp::Lt, &hi_e, &lo_e);
            let beyond = cmp(lw, CmpOp::Gt, &hi_e, &src_hi);
            let bad = or(lw, below, reversed);
            let bad = or(lw, bad, beyond);
            let message =
                "a slice `r[lo..hi]` of a range needs r.first <= lo <= hi <= r.end".to_string();
            out.push(Stmt::If(
                bad,
                Block {
                    stmts: vec![Stmt::Trap(message, lw.span(span))],
                },
                Block::default(),
            ));
            let res = lw.mk(
                ExprKind::Range {
                    lo: Box::new(lo_e),
                    hi: Box::new(hi_e),
                },
                range_ty.clone(),
                span,
            );
            out.push_give(res);
        })
    }

    /// `lo..hi by k` kept as a value is the std view `(lo..hi).step(k)`: it
    /// reverses, and its `len`, `first` and `last` are \(O(1)\) (R01.8.4).
    pub(super) fn stepped_range_val(&mut self, e: &ast::Expr, hint: Option<&Ty>) -> Expr {
        let ast::ExprKind::Range {
            lo,
            hi,
            inclusive,
            step: Some(k),
        } = &e.kind
        else {
            return self.err_expr(e.span);
        };
        let range = ast::ExprKind::Range {
            lo: lo.clone(),
            hi: hi.clone(),
            inclusive: *inclusive,
            step: None,
        };
        let name = ast::Name {
            text: "step".into(),
            span: k.span,
        };
        let base = Box::new(ast::Expr {
            kind: range,
            span: e.span,
        });
        let callee = ast::Expr {
            kind: ast::ExprKind::Field { base, name },
            span: e.span,
        };
        let arg = ast::Arg {
            name: None,
            mutable: false,
            value: (**k).clone(),
        };
        let kind = ast::ExprKind::Call {
            callee: Box::new(callee),
            args: vec![arg],
        };
        self.expr(&ast::Expr { kind, span: e.span }, hint)
    }
}
