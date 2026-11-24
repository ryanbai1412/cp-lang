//! Assignments: stores to places, rekeyed locals and counted globals.

use super::*;

impl<'p> Emitter<'p> {
    /// The `+`/`-` of `global += k`, an `Int` global stepped by a
    /// literal, which a guard can add as `0` instead of branching around.
    /// The local list a returned `list.rekey(keys)` owns and reads for the
    /// last time: a local of this fn, or a parameter it takes by value, that
    /// the keys do not read.
    pub(super) fn rekeyed_local(&self, value: &Expr) -> Option<usize> {
        let ExprKind::Builtin(Builtin::Rekey, args) = &value.kind else {
            return None;
        };
        let ExprKind::Local(local) = args.first()?.kind else {
            return None;
        };
        let id = self.fn_id?;
        let owned = local >= self.program.fns[id].params.len() || self.sunk[id].contains(&local);
        let read_again = args[1..].iter().any(|arg| {
            let mut used = false;
            crate::walk::deep_expr(arg, &mut |e| {
                used |= matches!(e.kind, ExprKind::Local(s) if s == local);
            });
            used
        });
        (owned && !read_again && matches!(value.ty, Ty::Keyed(..))).then_some(local)
    }

    pub(super) fn counted_global(
        &self,
        place: &Place,
        op: AssignOp,
        value: &Expr,
    ) -> Option<&'static str> {
        let PlaceKind::Global(id) = &place.kind else {
            return None;
        };
        let sym = match op {
            AssignOp::Binary(BinOp::Add) => "+",
            AssignOp::Binary(BinOp::Sub) => "-",
            _ => return None,
        };
        (place.ty == Ty::Int
            && matches!(value.kind, ExprKind::Int(1))
            && self.facts.unit_step((None, true, *id))
            && !self.const_globals.contains(id)
            && !self.dropped_store(place, op))
        .then_some(sym)
    }

    /// Whether `v op= value` on an `Int` variable is proved to stay an
    /// `Int`, as `v op value` would be.
    fn store_fits(&self, place: &Place, op: BinOp, value: &Expr) -> bool {
        let kind = match place.kind {
            PlaceKind::Local(slot) => ExprKind::Local(slot),
            PlaceKind::Global(id) => ExprKind::Global(id),
            _ => return false,
        };
        let read = Expr {
            kind,
            ty: place.ty.clone(),
            span: place.span,
        };
        place.ty == Ty::Int && self.facts.fits(self.facts_home(), op, &read, value)
    }

    pub(super) fn assign(&mut self, place: &Place, op: AssignOp, value: &Expr, out: &mut String) {
        if self.dropped_store(place, op) {
            self.dropped_value(value, out);
            return;
        }
        if let Some(base) = self.absent_field_base(place) {
            if !crate::uses::effects::safe_place(base) {
                let held = self.place(base);
                self.line(out, &format!("(void){held};"));
            }
            self.dropped_value(value, out);
            return;
        }
        if let (AssignOp::Set, PlaceKind::Global(id)) = (op, &place.kind) {
            if self.const_globals.contains(id) {
                return;
            }
        }
        self.mark_moved_entry(place, op, value);
        let marks = self.one_walk_entry(place, op, value, out);
        self.store(place, op, value, out);
        if let Some((found, pinned)) = marks {
            self.found.truncate(found);
            self.pinned.truncate(pinned);
        }
    }

    /// `m[k] = f(m.get(k) ?? d)` or `m[k] = f(m[k])` would walk the map
    /// twice, to read the entry and to store it. When nothing else in the
    /// value reaches `m` or changes `k`, and the value cannot leave early,
    /// the entry is found (or put at `d`) once and read and stored through
    /// a reference. Gives back the marks to undo once the store is written.
    fn one_walk_entry(
        &mut self,
        place: &Place,
        op: AssignOp,
        value: &Expr,
        out: &mut String,
    ) -> Option<(usize, usize)> {
        let (AssignOp::Set, PlaceKind::Index(base, keys)) = (op, &place.kind) else {
            return None;
        };
        let ([key], PlaceKind::Local(_) | PlaceKind::Global(_), Ty::Map(..) | Ty::HashMap(..)) =
            (keys.as_slice(), &base.kind, &base.ty)
        else {
            return None;
        };
        if self.found_cell(base, keys).is_some()
            || self.map_slot(key).is_some()
            || !crate::uses::effects::pure_path(key)
        {
            return None;
        }
        let root = walk::root(base);
        let same = |m: &Expr, k: &Expr| {
            matches!(m.kind, ExprKind::Local(_) | ExprKind::Global(_))
                && walk::path_root(m) == Some(root)
                && (same_path(k, key) || self.key_temp_of(key, k))
        };
        let (mut read, mut copies, mut mentions, mut exits) = (None, Vec::new(), 0, false);
        walk::deep_expr(value, &mut |e| match &e.kind {
            ExprKind::Local(s) if root == (false, *s) => mentions += 1,
            ExprKind::Global(g) if root == (true, *g) => mentions += 1,
            ExprKind::Binary(BinOp::Coalesce, got, d) => {
                if let Some((m, k)) = crate::lend::entry_read(got) {
                    if same(m, k) {
                        read = Some((e, m, Some(&**d)));
                    }
                }
            }
            ExprKind::Index(m, ks) if matches!(m.ty, Ty::Map(..) | Ty::HashMap(..)) => {
                if let [k] = ks.as_slice() {
                    if same(m, k) {
                        read = Some((e, m, None));
                    }
                }
            }
            ExprKind::Builtin(Builtin::Copy, of) if of.len() == 1 => {
                copies.push((e as *const Expr, &of[0] as *const Expr))
            }
            ExprKind::Do(block) => walk::deep_stmts(block, &mut |s| {
                exits |= matches!(s, Stmt::Return(_) | Stmt::Break(_) | Stmt::Continue(_))
            }),
            _ => {}
        });
        let (at, map, d) = read?;
        let touch = Touch::of_expr(value);
        let key_kept = walk::path_root(key).is_some_and(|r| !touch.writes.contains(&r));
        // A `mut` parameter may alias a global or a caller's local, and a
        // call may grow the map behind the reference, so the value writes
        // nothing but its own locals, through any call.
        let deep = self
            .uses
            .table
            .access(self.program, None, &|_, _| None, value);
        let own = |w: &(bool, usize)| !w.0 && deep.binds.contains(&w.1);
        let reaches = deep.opaque || !deep.writes.iter().all(own);
        if mentions != 1 || exits || !key_kept || touch.writes.contains(&root) || reaches {
            return None;
        }
        let traps = self.target.checks != crate::Checks::None;
        if let Some(d) = d {
            let call_ok = |e: &Expr| {
                crate::uses::effects::pure_call(
                    self.program,
                    &self.devirt,
                    &self.sink_pure,
                    self.fn_id,
                    traps,
                    e,
                )
            };
            if !crate::uses::effects::trap_free_with(d, traps, &call_ok) {
                return None;
            }
        }
        let (m, k, cell) = (self.place(base), self.expr(key).text, self.temp());
        let find = match d {
            None => format!("{m}.at({k})"),
            Some(d) => {
                let d = self.expr(d).text;
                let hashed = match &base.ty {
                    Ty::Map(k, _) => self.program.unordered_maps.contains(k),
                    _ => true,
                };
                let put = if hashed { "hash_put_or" } else { "map_put_or" };
                self.need(put);
                format!("cpl::{put}({m}, {k}, [&] {{ return {d}; }})")
            }
        };
        self.line(out, &format!("auto& {cell} = {find};"));
        let marks = (self.found.len(), self.pinned.len());
        let at = at as *const Expr;
        let copy = copies.iter().find(|c| c.1 == at).map(|c| c.0);
        let pin = match copy {
            Some(c) if self.moved_entry.remove(&(c as usize)) => (c, format!("std::move({cell})")),
            _ if self.moved_entry.remove(&(at as usize)) => (at, format!("std::move({cell})")),
            _ => (at, cell.clone()),
        };
        self.pinned.push(pin);
        self.found.push((map.clone(), key.clone(), cell));
        Some(marks)
    }

    fn store(&mut self, place: &Place, op: AssignOp, value: &Expr, out: &mut String) {
        let whole = value;
        let value = self.flatten(value, out);
        if let (AssignOp::Set, PlaceKind::Global(g)) = (op, &place.kind) {
            if self.devirt.main_auto_global(*g) && self.fn_id.is_none() && self.closure_id.is_none()
            {
                let name = self.place(place);
                let text = self.expr(value).text;
                self.line(out, &format!("auto {name} = {text};"));
                return;
            }
        }
        if matches!(op, AssignOp::Set) && self.let_lent_main(place, value, out) {
            return;
        }
        if matches!(op, AssignOp::Set) && self.store_pinned(place, value, out) {
            return;
        }
        if let (AssignOp::Set, PlaceKind::Local(_) | PlaceKind::Global(_)) = (op, &place.kind) {
            if matches!((&value.kind, &value.ty), (ExprKind::Tuple(items), Ty::List(_)) if items.is_empty())
                || matches!(&value.kind, ExprKind::Str(bytes) if bytes.is_empty())
            {
                let name = self.place(place);
                self.line(out, &format!("{name}.clear();"));
                return;
            }
        }
        if let (AssignOp::Set, PlaceKind::Local(_) | PlaceKind::Global(_), ExprKind::Read(spec)) =
            (op, &place.kind, &value.kind)
        {
            match &**spec {
                Spec::Array { dims, elem } => {
                    let name = self.place(place);
                    let l = self.place_elem_layout(place).cloned();
                    self.read_array_with_layout(&name, false, dims, elem, l.as_ref(), out);
                    return;
                }
                Spec::Plain(Ty::Str) => {
                    self.need("read_str");
                    let name = self.place(place);
                    self.line(out, &format!("cpl::read_str_into({name});"));
                    return;
                }
                _ => {}
            }
        }
        let place_fixed = self.place_fixed_len(place);
        let place_l = self.place_elem_layout(place).cloned();
        let mut v = if op == AssignOp::Set && place_fixed.is_some() {
            self.expr_with_fixed_len(value, place_fixed)
        } else if op == AssignOp::Set && place_l.is_some() {
            self.expr_with_elem_layout(value, place_l.as_ref())
        } else {
            self.expr(value)
        };
        if let Some(cell) = (op == AssignOp::Set)
            .then(|| self.carried_store(place))
            .flatten()
        {
            self.line(out, &format!("{cell} = {};", v.text));
            v.text = cell;
        }
        if let PlaceKind::Index(base, _) = &place.kind {
            if op == AssignOp::Set {
                if let Some(l @ (cpl_ir::NarrowLayout::Int(_) | cpl_ir::NarrowLayout::Tuple(_))) =
                    self.place_elem_layout(base)
                {
                    v.text = self.narrow_cell_value(&v.text, l);
                }
            }
        }
        // A value built by its own block (a comprehension), or a block's
        // local read for the last time, is not read again: it moves.
        let last = std::mem::take(&mut self.moves_last);
        if let (AssignOp::Set, ExprKind::Local(built)) = (op, &value.kind) {
            let made_here = last && std::ptr::eq(value, whole)
                || flat(whole).is_some_and(|(stmts, _)| {
                    stmts
                        .iter()
                        .any(|stmt| matches!(stmt, Stmt::Let(slot, _) if slot == built))
                });
            if made_here
                && !self.ref_bound.contains(built)
                && !value.ty.is_scalar()
                && !matches!(value.ty, Ty::Fn(..))
            {
                v.text = format!("std::move({})", v.text);
            }
        }
        if let (AssignOp::Set, ExprKind::Some(inner)) = (op, &value.kind) {
            if last && std::ptr::eq(value, whole) {
                if let Some(text) = self.some_moved(inner, &value.ty) {
                    v.text = text;
                }
            }
        }
        // Handle map/hashmap fold-on-missing-key (`max=`, `min=`, `*=`, `+=`, `-=`, `|=`, `^=`, `&=`).
        if let PlaceKind::Index(base, idx) = &place.kind {
            if matches!(base.ty, Ty::Map(..) | Ty::HashMap(..))
                && op != AssignOp::Set
                && self.found_cell(base, idx).is_none()
            {
                let b = self.place(base);
                let k = self.expr(&idx[0]).text;
                let at = self.map_slot(&idx[0]);
                let cell = match &at {
                    Some(at) => format!("cpl::map_slot_put({b}, {at}, {k})"),
                    None => format!("{b}[{k}]"),
                };
                if let Some(cpl_ir::NarrowLayout::Map(_, Some(w))) =
                    self.place_elem_layout(base).cloned()
                {
                    if self.narrow_map_fold(out, op, (&b, &k, &cell), &v.text, w.cpp()) {
                        if let Some(at) = at.filter(|_| {
                            matches!(
                                op,
                                AssignOp::Max
                                    | AssignOp::Min
                                    | AssignOp::Binary(BinOp::Mul | BinOp::BitAnd)
                            )
                        }) {
                            self.line(out, &format!("{at} = cpl::map_slot({b}, {k});"));
                        }
                        return;
                    }
                }
                match op {
                    AssignOp::Max => {
                        self.need("map_fold");
                        self.line(out, &format!("cpl::map_max({b}, {k}, {});", v.text));
                        if let Some(at) = &at {
                            self.line(out, &format!("{at} = cpl::map_slot({b}, {k});"));
                        }
                        return;
                    }
                    AssignOp::Min => {
                        self.need("map_fold");
                        self.line(out, &format!("cpl::map_min({b}, {k}, {});", v.text));
                        if let Some(at) = &at {
                            self.line(out, &format!("{at} = cpl::map_slot({b}, {k});"));
                        }
                        return;
                    }
                    AssignOp::Binary(BinOp::Mul) if matches!(place.ty, Ty::Int | Ty::I128) => {
                        let f = self.overflow_checked(&place.ty, BinOp::Mul);
                        let it = self.temp();
                        self.line(
                            out,
                            &format!(
                                "if (auto {it} = {b}.find({k}); {it} == {b}.end()) {b}.emplace({k}, {}); \
                                 else {it}->second = {f}({it}->second, {});",
                                v.text, v.text
                            ),
                        );
                        if let Some(at) = &at {
                            self.line(out, &format!("{at} = cpl::map_slot({b}, {k});"));
                        }
                        return;
                    }
                    AssignOp::Binary(BinOp::Mul) => {
                        self.need("map_fold");
                        self.line(out, &format!("cpl::map_mul({b}, {k}, {});", v.text));
                        if let Some(at) = &at {
                            self.line(out, &format!("{at} = cpl::map_slot({b}, {k});"));
                        }
                        return;
                    }
                    AssignOp::Binary(BinOp::BitAnd) => {
                        self.need("map_fold");
                        self.line(out, &format!("cpl::map_and({b}, {k}, {});", v.text));
                        if let Some(at) = &at {
                            self.line(out, &format!("{at} = cpl::map_slot({b}, {k});"));
                        }
                        return;
                    }
                    AssignOp::Binary(BinOp::Add | BinOp::Sub | BinOp::BitOr | BinOp::BitXor) => {
                        let sym = match op {
                            AssignOp::Binary(BinOp::Sub) => "-",
                            AssignOp::Binary(BinOp::BitOr) => "|",
                            AssignOp::Binary(BinOp::BitXor) => "^",
                            _ => "+",
                        };
                        let checked = matches!(place.ty, Ty::Int | Ty::I128)
                            && matches!(op, AssignOp::Binary(BinOp::Add | BinOp::Sub));
                        if checked {
                            let bop = if sym == "-" { BinOp::Sub } else { BinOp::Add };
                            let f = self.overflow_checked(&place.ty, bop);
                            let (tmp, r) = (self.temp(), self.temp());
                            self.line(out, "{");
                            self.indent += 1;
                            self.line(out, &format!("auto {tmp} = {};", v.text));
                            self.line(out, &format!("auto& {r} = {cell};"));
                            self.line(out, &format!("{r} = {f}({r}, {tmp});"));
                            self.indent -= 1;
                            self.line(out, "}");
                        } else if matches!(base.ty, Ty::HashMap(..))
                            && expr_may_alias_root(value, walk::root(base))
                        {
                            let tmp = self.temp();
                            self.line(out, &format!("auto {tmp} = {};", v.text));
                            self.line(out, &format!("{cell} {sym}= {tmp};"));
                        } else {
                            self.line(out, &format!("{cell} {sym}= {};", v.text));
                        }
                        return;
                    }
                    _ => {}
                }
            }
        }
        let into_map = op == AssignOp::Set
            && matches!(&place.kind, PlaceKind::Index(base, _) if matches!(base.ty, Ty::Map(..) | Ty::HashMap(..)));
        let hash_alias = op == AssignOp::Set
            && matches!(
                &place.kind,
                PlaceKind::Index(base, _)
                    if matches!(base.ty, Ty::HashMap(..))
                        && expr_may_alias_root(value, walk::root(base))
            );
        let p = self.place_at(place, into_map);
        let ty = self.ty(&place.ty);
        let narrow = match &place.kind {
            PlaceKind::Index(base, _) => match self.place_elem_layout(base) {
                Some(cpl_ir::NarrowLayout::Map(_, Some(w))) => Some(w.cpp()),
                _ => None,
            },
            _ => None,
        };
        if narrow.is_some() {
            self.need("fit");
        }
        // `p = f(p, v)` names the place twice; a narrow map cell takes the wide result through `cpl::fit`.
        let store = |p: &str, new: String| match narrow {
            Some(w) => format!("{p} = cpl::fit<{w}>({new});"),
            None => format!("{p} = {new};"),
        };
        let twice = |this: &mut Self, out: &mut String, make: &dyn Fn(&str, &str) -> String| {
            if this.place_is_plain(place) {
                this.line(out, &store(&p, make(&p, &v.text)));
            } else {
                let (value_name, r) = (this.temp(), this.temp());
                this.line(out, "{");
                this.indent += 1;
                this.line(out, &format!("{ty} {value_name} = {};", v.text));
                this.line(out, &format!("auto& {r} = {p};"));
                this.line(out, &store(&r, make(&r, &value_name)));
                this.indent -= 1;
                this.line(out, "}");
            }
        };
        match op {
            AssignOp::Set if hash_alias => {
                let tmp = self.temp();
                self.note_set_copy(value, &v.text);
                self.line(out, &format!("auto {tmp} = {};", v.text));
                self.line(out, &format!("{p} = std::move({tmp});"));
            }
            AssignOp::Set => {
                let text = match self.place_elem_layout(place).cloned() {
                    Some(l @ cpl_ir::NarrowLayout::Map(..)) => {
                        self.expr_with_elem_layout(value, Some(&l)).text
                    }
                    _ => v.text,
                };
                let v = self.as_stored(value, text);
                self.note_set_copy(value, &v);
                self.line(out, &format!("{p} = {v};"))
            }
            AssignOp::Max => twice(self, out, &|p, v| format!("std::max<{ty}>({p}, {v})")),
            AssignOp::Min => twice(self, out, &|p, v| format!("std::min<{ty}>({p}, {v})")),
            AssignOp::Binary(bop) => match (&place.ty, bop) {
                (Ty::Int, BinOp::Div | BinOp::Rem) => {
                    if narrow.is_none() && self.pow2_assign(&p, bop == BinOp::Div, value, out) {
                        return;
                    }
                    let positive =
                        self.facts.sign(self.facts_home(), value) == crate::ranges::Sign::Positive;
                    let helper = match (positive, bop == BinOp::Div) {
                        (true, true) => "divp",
                        (true, false) => "modp",
                        (false, true) => "ediv",
                        (false, false) => "emod",
                    };
                    self.need(helper);
                    twice(self, out, &|p, v| format!("cpl::{helper}({p}, {v})"));
                }
                (Ty::Int, BinOp::Pow) => {
                    self.need("ipow");
                    twice(self, out, &|p, v| format!("cpl::ipow({p}, {v})"));
                }
                (Ty::I128, BinOp::Div | BinOp::Rem) => {
                    let helper = if bop == BinOp::Div {
                        "ediv128"
                    } else {
                        "emod128"
                    };
                    self.need(helper);
                    twice(self, out, &|p, v| format!("cpl::{helper}({p}, {v})"));
                }
                (Ty::I128 | Ty::U64, BinOp::Pow) => {
                    self.need("wpow");
                    twice(self, out, &|p, v| format!("cpl::wpow({p}, {v})"));
                }
                (Ty::Float, BinOp::Pow) => twice(self, out, &|p, v| format!("std::pow({p}, {v})")),
                (Ty::Int | Ty::I128, BinOp::Add | BinOp::Sub | BinOp::Mul)
                    if !self.store_fits(place, bop, value) =>
                {
                    let f = self.overflow_checked(&place.ty, bop);
                    twice(self, out, &|p, v| format!("{f}({p}, {v})"));
                }
                (Ty::Int, BinOp::Shl) if self.facts.within(self.facts_home(), value, 64) => {
                    self.need("ishl");
                    twice(self, out, &|p, v| format!("cpl::ishl({p}, {v})"));
                }
                (Ty::Int, BinOp::Shl) => {
                    self.need("ishl");
                    self.need("shamt");
                    twice(self, out, &|p, v| {
                        format!("cpl::ishl({p}, cpl::shamt({v}, 64))")
                    });
                }
                _ => {
                    let sym = match bop {
                        BinOp::Sub => "-",
                        BinOp::Mul => "*",
                        BinOp::Div => "/",
                        BinOp::Rem => "%",
                        BinOp::BitAnd => "&",
                        BinOp::BitOr => "|",
                        BinOp::BitXor => "^",
                        BinOp::Shl => "<<",
                        BinOp::Shr => ">>",
                        _ => "+",
                    };
                    if narrow.is_some() {
                        twice(self, out, &|p, v| format!("{p} {sym} {v}"));
                    } else {
                        self.line(out, &format!("{p} {sym}= {};", v.text));
                    }
                }
            },
        }
    }

    /// `m[k] op= v` into a map whose values are stored narrow: the op runs
    /// on `ll` and `cpl::fit` stores it back, so a result outside the
    /// value's range traps under `LOCAL` instead of wrapping into range.
    /// `max=`, `min=`, `*=` and `&=` on a missing key store `v`, as
    /// `map_fold` does. False when `op` has no fold here.
    fn narrow_map_fold(
        &mut self,
        out: &mut String,
        op: AssignOp,
        (b, k, cell): (&str, &str, &str),
        v: &str,
        w: &str,
    ) -> bool {
        let wide = match op {
            AssignOp::Max => "std::max<ll>(@, #)".to_string(),
            AssignOp::Min => "std::min<ll>(@, #)".to_string(),
            AssignOp::Binary(bop @ (BinOp::Add | BinOp::Sub | BinOp::Mul)) => {
                format!("{}(@, #)", self.overflow_checked(&Ty::Int, bop))
            }
            AssignOp::Binary(BinOp::BitAnd) => "@ & #".to_string(),
            AssignOp::Binary(BinOp::BitOr) => "@ | #".to_string(),
            AssignOp::Binary(BinOp::BitXor) => "@ ^ #".to_string(),
            _ => return false,
        };
        self.need("fit");
        let (x, it) = (self.temp(), self.temp());
        let wide = |old: &str| wide.replace('@', &format!("ll({old})")).replace('#', &x);
        let set = if matches!(
            op,
            AssignOp::Max | AssignOp::Min | AssignOp::Binary(BinOp::Mul | BinOp::BitAnd)
        ) {
            format!(
                "if (auto {it} = {b}.find({k}); {it} == {b}.end()) {b}.emplace({k}, cpl::fit<{w}>({x})); \
                 else {it}->second = cpl::fit<{w}>({});",
                wide(&format!("{it}->second"))
            )
        } else {
            format!("auto& {it} = {cell}; {it} = cpl::fit<{w}>({});", wide(&it))
        };
        self.line(out, &format!("{{ ll {x} = {v}; {set} }}"));
        true
    }
}
