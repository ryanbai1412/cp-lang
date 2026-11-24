//! Closure captures: a local threaded down from an enclosing `FnCx`, through every
//! nested closure between, with a captured bag range view taking its parts along.

use super::*;

impl Lowerer<'_> {
    /// Captures a local variable from an enclosing `FnCx` (possibly across multiple nested closures).
    pub(super) fn capture_from_outer(&mut self, name: &str) -> Option<Slot> {
        if !self.cx.is_closure {
            return None;
        }
        // Check if already captured in current closure.
        if let Some(&slot) = self.cx.captured_names.get(name) {
            return Some(slot);
        }
        // Find the innermost enclosing FnCx that defines `name` (either directly or as a field of `self`).
        let found_level = self
            .outer_cxs
            .iter()
            .rposition(|cx| cx.scopes.iter().rev().any(|s| s.contains_key(name)))?;
        let orig_slot = self.outer_cxs[found_level]
            .scopes
            .iter()
            .rev()
            .find_map(|s| s.get(name).copied())?;
        let by_ref = self.outer_cxs[found_level].mut_slots.contains(&orig_slot);
        let slots = self.thread_capture(found_level, orig_slot, name, by_ref);
        let inner = *slots.last().expect("one slot per level");
        if let Some(view) = self.outer_cxs[found_level]
            .sorted_views
            .get(&orig_slot)
            .cloned()
        {
            self.capture_sorted_view(found_level, name, view, &slots);
        }
        let prev = if slots.len() >= 2 {
            slots[slots.len() - 2]
        } else {
            orig_slot
        };
        if let Some(rinfo) = self
            .outer_cxs
            .last()
            .and_then(|ocx| ocx.local_ranges.get(&prev).cloned())
        {
            self.cx.local_ranges.insert(inner, rinfo);
        }
        Some(inner)
    }

    /// Threads the outer slot `orig` of the FnCx at `found_level` down to the current
    /// closure under `key`, returning its slot at each level below `found_level`.
    fn thread_capture(
        &mut self,
        found_level: usize,
        orig: Slot,
        key: &str,
        by_ref: bool,
    ) -> Vec<Slot> {
        let ty = self.outer_cxs[found_level].locals[orig].ty.clone();
        let mut slots = Vec::new();
        let mut cur = orig;
        let depth = self.outer_cxs.len();
        for level in (found_level + 1)..=depth {
            let cx = if level == depth {
                &mut self.cx
            } else {
                &mut self.outer_cxs[level]
            };
            if let Some(&existing) = cx.captured_names.get(key) {
                cur = existing;
            } else {
                let inner = cx.locals.len();
                cx.locals.push(Local {
                    name: key.to_string(),
                    ty: ty.clone(),
                });
                cx.captures.push(Capture {
                    outer: cur,
                    inner,
                    is_global: false,
                    by_ref,
                });
                cx.captured_names.insert(key.to_string(), inner);
                if by_ref {
                    cx.mut_slots.insert(inner);
                }
                cur = inner;
            }
            slots.push(cur);
        }
        slots
    }

    /// A captured `b.range(r)` view is the same view in the closure: its bag is captured by
    /// reference (the checker keeps it lent while the closure lives) and its bounds by value.
    fn capture_sorted_view(
        &mut self,
        found_level: usize,
        name: &str,
        view: SortedView,
        slots: &[Slot],
    ) {
        let base = match &view.base.kind {
            PlaceKind::Local(root) => {
                Some(self.thread_capture(found_level, *root, &format!("{name}#base"), true))
            }
            PlaceKind::Global(_) => None,
            _ => return,
        };
        let part = |lw: &mut Self, slot: Slot, what: &str| {
            lw.thread_capture(found_level, slot, &format!("{name}#{what}"), false)
        };
        let lo = part(self, view.lo, "lo");
        let lo_inc = part(self, view.lo_inc, "lo_inc");
        let hi = part(self, view.hi, "hi");
        let hi_inc = part(self, view.hi_inc, "hi_inc");
        let empty = view.empty.map(|e| part(self, e, "empty"));
        let depth = self.outer_cxs.len();
        for (k, &slot) in slots.iter().enumerate() {
            let mut v = view.clone();
            if let Some(base) = &base {
                v.base.kind = PlaceKind::Local(base[k]);
            }
            (v.lo, v.lo_inc, v.hi, v.hi_inc) = (lo[k], lo_inc[k], hi[k], hi_inc[k]);
            v.empty = empty.as_ref().map(|e| e[k]);
            let level = found_level + 1 + k;
            let cx = if level == depth {
                &mut self.cx
            } else {
                &mut self.outer_cxs[level]
            };
            // The view itself is never a C++ value, only its parts are.
            cx.captures.retain(|c| c.inner != slot);
            cx.sorted_views.insert(slot, v);
        }
    }
}
