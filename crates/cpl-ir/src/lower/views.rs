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
}
