//! The value a `return` or a `Do` block's `give` hands out: each local
//! whose last use it is moves into it.

use crate::emit::Emitter;
use crate::uses::split::{each_sink, Sink};
use cpl_ir::{Block, Expr, ExprKind, Slot, Stmt, Ty};
use std::collections::BTreeSet;

/// The `let`s of a `Do` block written as a lambda: the lambda owns them,
/// so each one dies at every `give`.
pub(crate) fn block_lets(block: &Block) -> BTreeSet<Slot> {
    let mut lets = BTreeSet::new();
    each_sink(block, &mut |sink| {
        if let Sink::Let(slot, _) = sink {
            lets.insert(slot);
        }
    });
    lets
}

/// The variable a value is, or is a part of.
fn local_root(e: &Expr) -> Option<Slot> {
    match &e.kind {
        ExprKind::Local(s) => Some(*s),
        ExprKind::Field(base, _) | ExprKind::Index(base, _) => local_root(base),
        _ => None,
    }
}

impl Emitter<'_> {
    /// A tuple or struct literal handed out, or `Some` of one, with each
    /// item that dies there moved in.
    pub(super) fn handed_out(&mut self, value: &Expr, is_return: bool) -> Option<String> {
        let ExprKind::Some(inner) = &value.kind else {
            return self.handed_out_items(value, is_return);
        };
        let opt = self.ty(&value.ty);
        let laid_out = is_return
            && self
                .fn_id
                .is_some_and(|id| self.elem_layout(cpl_ir::ElemLoc::FnRet(id)).is_some());
        if laid_out || !opt.starts_with("std::optional<") {
            return None;
        }
        let items = self.handed_out_items(inner, is_return)?;
        Some(format!("{opt}({items})"))
    }

    fn handed_out_items(&mut self, value: &Expr, is_return: bool) -> Option<String> {
        let ExprKind::Tuple(items) = &value.kind else {
            return None;
        };
        if items.is_empty()
            || matches!(
                value.ty,
                Ty::Unit | Ty::Error | Ty::Map(..) | Ty::HashMap(..)
            )
        {
            return None;
        }
        let ty = self.ty(&value.ty);
        let struct_id = match value.ty {
            Ty::Struct(s) => Some(s),
            _ => None,
        };
        let dropped: Vec<bool> = (0..items.len())
            .map(|idx| struct_id.is_some_and(|s| self.devirt.dropped_field(s, idx)))
            .collect();
        let rendered: Vec<String> = items
            .iter()
            .enumerate()
            .filter(|(idx, _)| !dropped[*idx])
            .map(|(idx, item)| {
                if struct_id.is_some_and(|s| {
                    self.devirt.field_elidable(s, idx) && self.devirt.skippable_value(item)
                }) {
                    return "false".to_string();
                }
                if let Some(lent) = self.lent_item(struct_id, idx, item) {
                    return lent;
                }
                if let Some(sid) = struct_id {
                    if let Some(&k) = self.rows.get(&(sid, idx)) {
                        let cell = self
                            .ty(&item.ty)
                            .strip_prefix("std::vector<")
                            .and_then(|c| c.strip_suffix('>'))
                            .map(str::to_string);
                        if let Some(cell) = cell {
                            if let Some(row_init) = self.emit_row_init(item, sid, idx, &cell, k) {
                                return row_init;
                            }
                        }
                    }
                }
                let dies = self.handed_item_dies(items, idx, is_return);
                // A call item of a returned literal is the last use of the
                // locals it is given that no other item names.
                self.last_use_call = is_return
                    && self.closure_id.is_none()
                    && self.fn_id.is_some()
                    && self.call_item_alone(items, idx);
                let whole = crate::uses::copied(item).unwrap_or(item);
                if let (ExprKind::Local(slot), true) = (&whole.kind, dies) {
                    let moved = self.move_of(&item.ty, &self.names[*slot]);
                    return match struct_id {
                        Some(s) => self.narrow_field_store(s, idx, &moved),
                        None => moved,
                    };
                }
                if let Some(l) = struct_id
                    .and_then(|s| self.elem_layout(cpl_ir::ElemLoc::Field(s, idx)))
                    .cloned()
                {
                    let v = self.expr_with_elem_layout(item, Some(&l)).text;
                    self.last_use_call = false;
                    return self.moved_if(dies, &item.ty, v);
                }
                let v = self.expr(item).text;
                self.last_use_call = false;
                let v = self.moved_if(dies, &item.ty, v);
                match struct_id {
                    Some(s) => self.narrow_field_store(s, idx, &v),
                    None => v,
                }
            })
            .collect();
        Some(format!("{ty}{{{}}}", rendered.join(", ")))
    }

    /// Item `idx` is a call, and no other item names a local it is given.
    fn call_item_alone(&self, items: &[Expr], idx: usize) -> bool {
        let ExprKind::Call(_, args) = &items[idx].kind else {
            return false;
        };
        let given = args.iter().filter_map(|arg| match arg {
            cpl_ir::Arg::Value(Expr {
                kind: ExprKind::Local(s),
                ..
            }) => Some(*s),
            _ => None,
        });
        given.into_iter().all(|s| {
            items
                .iter()
                .enumerate()
                .all(|(j, other)| j == idx || !crate::uses::mentions_any(self.program, other, s))
        })
    }

    /// Whether item `idx` is a value the fn (for a `return`) or the `Do`
    /// block's lambda (for a `give`) owns and never reads again.
    fn handed_item_dies(&self, items: &[Expr], idx: usize, is_return: bool) -> bool {
        if is_return {
            // A copy of a local that dies here takes it.
            return match crate::uses::copied(&items[idx]) {
                Some(src) if matches!(src.kind, ExprKind::Local(_)) => {
                    let mut taken = items.to_vec();
                    taken[idx] = src.clone();
                    self.returned_item_dies(&taken, idx)
                }
                _ => self.returned_item_dies(items, idx),
            };
        }
        let item = &items[idx];
        let Some(slot) = local_root(item) else {
            return false;
        };
        self.give_owned
            .as_ref()
            .is_some_and(|owned| owned.contains(&slot))
            && !self.ref_bound.contains(&slot)
            && !item.ty.is_scalar()
            && !matches!(item.ty, Ty::Fn(..))
            && !items
                .iter()
                .enumerate()
                .any(|(j, other)| j != idx && crate::uses::mentions_any(self.program, other, slot))
    }
}

impl Emitter<'_> {
    /// The tuple a block written in line gives, `(t1, t2)` or `Some((t1,
    /// t2))`, takes each of the block's own `let`s it alone reads: the
    /// block is over, so nothing reads them again.
    pub(super) fn take_block_lets(&mut self, stmts: &[Stmt], value: &Expr) {
        let tuple = match &value.kind {
            ExprKind::Some(inner) => inner,
            _ => value,
        };
        let ExprKind::Tuple(items) = &tuple.kind else {
            return;
        };
        let lets: BTreeSet<Slot> = stmts
            .iter()
            .filter_map(|s| match s {
                Stmt::Let(slot, _) => Some(*slot),
                _ => None,
            })
            .collect();
        for (idx, item) in items.iter().enumerate() {
            let Some(slot) = local_root(item).filter(|s| lets.contains(s)) else {
                continue;
            };
            if !self.ref_bound.contains(&slot)
                && !item.ty.is_scalar()
                && !matches!(item.ty, Ty::Fn(..))
                && !items.iter().enumerate().any(|(j, other)| {
                    j != idx && crate::uses::mentions_any(self.program, other, slot)
                })
            {
                self.taken_items.insert(item as *const Expr);
            }
        }
    }
}
