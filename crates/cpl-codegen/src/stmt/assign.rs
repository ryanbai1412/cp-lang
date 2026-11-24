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
            && matches!(value.kind, ExprKind::Int(_))
            && !self.const_globals.contains(id)
            && !self.dropped_store(place, op))
        .then_some(sym)
    }

    pub(super) fn assign(&mut self, place: &Place, op: AssignOp, value: &Expr, out: &mut String) {
        if self.dropped_store(place, op) {
            self.dropped_value(value, out);
            return;
        }
        if let (AssignOp::Set, PlaceKind::Global(id)) = (op, &place.kind) {
            if self.const_globals.contains(id) {
                return;
            }
        }
        self.mark_moved_entry(place, op, value);
        let whole = value;
        let value = self.flatten(value, out);
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
        // Handle map/hashmap fold-on-missing-key (`max=`, `min=`, `*=`, `+=`, `-=`, `|=`, `^=`, `&=`).
        if let PlaceKind::Index(base, idx) = &place.kind {
            if matches!(base.ty, Ty::Map(..) | Ty::HashMap(..)) && op != AssignOp::Set {
                let b = self.place(base);
                let k = self.expr(&idx[0]).text;
                match op {
                    AssignOp::Max => {
                        self.need("map_fold");
                        self.line(out, &format!("cpl::map_max({b}, {k}, {});", v.text));
                        return;
                    }
                    AssignOp::Min => {
                        self.need("map_fold");
                        self.line(out, &format!("cpl::map_min({b}, {k}, {});", v.text));
                        return;
                    }
                    AssignOp::Binary(BinOp::Mul) => {
                        self.need("map_fold");
                        self.line(out, &format!("cpl::map_mul({b}, {k}, {});", v.text));
                        return;
                    }
                    AssignOp::Binary(BinOp::BitAnd) => {
                        self.need("map_fold");
                        self.line(out, &format!("cpl::map_and({b}, {k}, {});", v.text));
                        return;
                    }
                    AssignOp::Binary(BinOp::Add | BinOp::Sub | BinOp::BitOr | BinOp::BitXor) => {
                        let sym = match op {
                            AssignOp::Binary(BinOp::Sub) => "-",
                            AssignOp::Binary(BinOp::BitOr) => "|",
                            AssignOp::Binary(BinOp::BitXor) => "^",
                            _ => "+",
                        };
                        if matches!(base.ty, Ty::HashMap(..))
                            && expr_may_alias_root(value, walk::root(base))
                        {
                            let tmp = self.temp();
                            self.line(out, &format!("auto {tmp} = {};", v.text));
                            self.line(out, &format!("{b}[{k}] {sym}= {tmp};"));
                        } else {
                            self.line(out, &format!("{b}[{k}] {sym}= {};", v.text));
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
        // `p = f(p, v)` names the place twice.
        let twice = |this: &mut Self, out: &mut String, make: &dyn Fn(&str, &str) -> String| {
            if this.place_is_plain(place) {
                this.line(out, &make(&p, &v.text));
            } else {
                let (value_name, r) = (this.temp(), this.temp());
                this.line(out, "{");
                this.indent += 1;
                this.line(out, &format!("{ty} {value_name} = {};", v.text));
                this.line(out, &format!("auto& {r} = {p};"));
                this.line(out, &make(&r, &value_name));
                this.indent -= 1;
                this.line(out, "}");
            }
        };
        match op {
            AssignOp::Set if hash_alias => {
                let tmp = self.temp();
                self.line(out, &format!("auto {tmp} = {};", v.text));
                self.line(out, &format!("{p} = std::move({tmp});"));
            }
            AssignOp::Set => {
                let v = self.as_stored(value, v.text);
                self.line(out, &format!("{p} = {v};"))
            }
            AssignOp::Max => twice(self, out, &|p, v| {
                format!("{p} = std::max<{ty}>({p}, {v});")
            }),
            AssignOp::Min => twice(self, out, &|p, v| {
                format!("{p} = std::min<{ty}>({p}, {v});")
            }),
            AssignOp::Binary(bop) => match (&place.ty, bop) {
                (Ty::Int, BinOp::Div | BinOp::Rem) => {
                    if self.pow2_assign(&p, bop == BinOp::Div, value, out) {
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
                    twice(self, out, &|p, v| format!("{p} = cpl::{helper}({p}, {v});"));
                }
                (Ty::Int, BinOp::Pow) => {
                    self.need("ipow");
                    twice(self, out, &|p, v| format!("{p} = cpl::ipow({p}, {v});"));
                }
                (Ty::I128, BinOp::Div | BinOp::Rem) => {
                    let helper = if bop == BinOp::Div {
                        "ediv128"
                    } else {
                        "emod128"
                    };
                    self.need(helper);
                    twice(self, out, &|p, v| format!("{p} = cpl::{helper}({p}, {v});"));
                }
                (Ty::I128 | Ty::U64, BinOp::Pow) => {
                    self.need("wpow");
                    twice(self, out, &|p, v| format!("{p} = cpl::wpow({p}, {v});"));
                }
                (Ty::Float, BinOp::Pow) => {
                    twice(self, out, &|p, v| format!("{p} = std::pow({p}, {v});"))
                }
                (Ty::Int | Ty::I128, BinOp::Add | BinOp::Sub | BinOp::Mul) => {
                    let f = self.overflow_checked(&place.ty, bop);
                    twice(self, out, &|p, v| format!("{p} = {f}({p}, {v});"));
                }
                (Ty::Int, BinOp::Shl) => {
                    self.need("ishl");
                    self.need("shamt");
                    twice(self, out, &|p, v| {
                        format!("{p} = cpl::ishl({p}, cpl::shamt({v}, 64));")
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
                    self.line(out, &format!("{p} {sym}= {};", v.text));
                }
            },
        }
    }
}
