//! A read-only `[V]T` parameter is a template parameter (R07.5.11): a view
//! keyed by `V`, such as a slice, gets its own copy of the fn that takes the
//! view as it is, so its keys stay its source's.

use super::*;

impl<'a> Lowerer<'a> {
    /// `id`, or its copy taking each read-only `[V]T` argument that is a
    /// view over `V` as that view.
    pub(super) fn view_param_fn(&mut self, id: FnId, args: &[ast::Arg]) -> FnId {
        let params = self.sigs[id].params.clone();
        let mut given: Vec<Option<&ast::Arg>> = vec![None; params.len()];
        let mut next = 0;
        for arg in args {
            let at = match &arg.name {
                Some(label) => params.iter().position(|p| p.label == label.text),
                None => {
                    while next < given.len() && given[next].is_some() {
                        next += 1;
                    }
                    (next < given.len()).then_some(next)
                }
            };
            if let Some(at) = at {
                given[at] = Some(arg);
            }
        }
        let mut takes = Vec::new();
        for (at, (param, arg)) in params.iter().zip(given).enumerate() {
            let Some(arg) = arg.filter(|a| !a.mutable && !param.mutable) else {
                continue;
            };
            let Ty::Array { dims, elem } = &param.ty else {
                continue;
            };
            let [Dim::Pos(d)] = dims.as_slice() else {
                continue;
            };
            if self.list_domains.contains(d) {
                continue;
            }
            let view = self.probe_ty_silent(&arg.value);
            if let Some(Ty::ListView(e, Dim::Pos(k)) | Ty::Keyed(e, Dim::Pos(k))) = &view {
                if k == d && e == elem {
                    takes.push((at, view.clone().unwrap_or(Ty::Error)));
                }
            }
        }
        if takes.is_empty() {
            return id;
        }
        let key = (id, takes);
        if let Some(&copy) = self.view_fns.get(&key) {
            return copy;
        }
        let sig = &self.sigs[id];
        let (decl, mod_id, offset, owner) = (sig.decl, sig.mod_id, sig.offset, sig.owner);
        let copy = self.declare_fn_in(decl, mod_id, offset, owner, sig.env.clone());
        self.sigs[copy].name = format!("{}_{copy}", self.sigs[copy].name);
        self.resolve_fn_sig(copy, None);
        for (at, view) in &key.1 {
            self.sigs[copy].params[*at].ty = view.clone();
        }
        self.view_fns.insert(key, copy);
        copy
    }

    /// `arg` where a `[K]T` of a generic fn is made from a list: a view
    /// keyed by `Int` stays a view, so `K` starts at its first key, and a
    /// `[K]T` already over `K` is passed as it is, keeping its keys (R04b.3.1).
    pub(super) fn expect_keeping_view(&mut self, arg: &ast::Expr, list: Ty, param: &Ty) -> Expr {
        let want = match self.probe_ty_silent(arg) {
            Some(view @ (Ty::ListView(_, Dim::Int) | Ty::Keyed(_, Dim::Int)))
                if view.seq_elem() == list.seq_elem() =>
            {
                view
            }
            Some(same) if same == *param => same,
            _ => list,
        };
        self.expect(arg, &want)
    }

    /// The first key of a view: where it starts in its source (R04b.1.1).
    pub(super) fn view_first_key(&mut self, value: &Expr, span: Span) -> Expr {
        let known = match &value.kind {
            ExprKind::Local(slot) => self.cx.view_lo.contains_key(slot),
            ExprKind::Global(gid) => self.global_view_lo.contains_key(gid),
            _ => false,
        };
        match &value.kind {
            // A view's keys are where it starts in its source, which a
            // parameter or temporary only knows at run time (R04b.1.1).
            ExprKind::Local(_) | ExprKind::Global(_) if !known && Self::keyed_view(&value.ty) => {
                self.key_lo(value, span)
            }
            ExprKind::Builtin(Builtin::ListSlice | Builtin::StrSlice, args) => args
                .get(1)
                .cloned()
                .unwrap_or_else(|| self.int_lit(0, span)),
            ExprKind::Local(slot) => self
                .cx
                .view_lo
                .get(slot)
                .copied()
                .map(|v| self.int_lit(v, span))
                .unwrap_or_else(|| self.int_lit(0, span)),
            ExprKind::Global(gid) => self
                .global_view_lo
                .get(gid)
                .copied()
                .map(|v| self.int_lit(v, span))
                .unwrap_or_else(|| self.int_lit(0, span)),
            _ => self.int_lit(0, span),
        }
    }

    /// What a list given for `[D]T` does to `D` (see `size_domain`), and
    /// the list: a view's `D` starts at its first key, and the array is a
    /// copy of it.
    pub(super) fn size_for_list(
        &mut self,
        d: DomainId,
        len: Expr,
        list: Expr,
        span: Span,
    ) -> (Stmt, Expr) {
        let first = self.range_domains.get(&d).copied().unwrap_or(0);
        let mut size = self.size_domain(d, first, len, span);
        let (Ty::ListView(elem, _) | Ty::Keyed(elem, _)) = &list.ty else {
            return (size, list);
        };
        let ty = Ty::List(elem.clone());
        if let Stmt::DefDomain { lo, .. } = &mut size {
            *lo = self.view_first_key(&list, span);
        }
        (
            size,
            self.mk(ExprKind::Builtin(Builtin::ToList, vec![list]), ty, span),
        )
    }
}
