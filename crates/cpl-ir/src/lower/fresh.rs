//! Fns whose result carries a domain their body makes: each call gets its own.

use super::*;

impl<'a> Lowerer<'a> {
    /// The domains made at or after `dom_start` that `ty` mentions, each
    /// once, in the order it first mentions them.
    pub(super) fn fresh_domains_in(
        &self,
        ty: &Ty,
        dom_start: usize,
        seen_structs: &mut HashSet<StructId>,
        seen_enums: &mut HashSet<EnumId>,
        out: &mut Vec<DomainId>,
    ) {
        let note = |d: DomainId, out: &mut Vec<DomainId>| {
            if d >= dom_start && !out.contains(&d) {
                out.push(d);
            }
        };
        match ty {
            Ty::Pos(d) | Ty::Bound(d) => note(*d, out),
            Ty::Array { dims, elem } => {
                for dim in dims {
                    if let Dim::Pos(d) | Dim::Bound(d) | Dim::Sym(d) = dim {
                        note(*d, out);
                    }
                }
                self.fresh_domains_in(elem, dom_start, seen_structs, seen_enums, out);
            }
            Ty::Range(t)
            | Ty::Opt(t)
            | Ty::List(t)
            | Ty::ListView(t, _)
            | Ty::Keyed(t, _)
            | Ty::MutView(t)
            | Ty::Set(t)
            | Ty::HashSet(t)
            | Ty::Mut(t) => self.fresh_domains_in(t, dom_start, seen_structs, seen_enums, out),
            Ty::Map(k, v) | Ty::HashMap(k, v) => {
                self.fresh_domains_in(k, dom_start, seen_structs, seen_enums, out);
                self.fresh_domains_in(v, dom_start, seen_structs, seen_enums, out);
            }
            Ty::Tuple(ts) => {
                for t in ts {
                    self.fresh_domains_in(t, dom_start, seen_structs, seen_enums, out);
                }
            }
            Ty::Record(fs) => {
                for (_, t) in fs {
                    self.fresh_domains_in(t, dom_start, seen_structs, seen_enums, out);
                }
            }
            Ty::Fn(ps, r) => {
                for t in ps {
                    self.fresh_domains_in(t, dom_start, seen_structs, seen_enums, out);
                }
                self.fresh_domains_in(r, dom_start, seen_structs, seen_enums, out);
            }
            Ty::Struct(sid) => {
                if !seen_structs.insert(*sid) {
                    return;
                }
                let info = &self.structs[*sid];
                if let Some((_, gargs)) = &info.origin_gen {
                    for a in gargs {
                        match a {
                            GenArg::Domain(d) => note(*d, out),
                            GenArg::Ty(t) => {
                                self.fresh_domains_in(t, dom_start, seen_structs, seen_enums, out)
                            }
                            _ => {}
                        }
                    }
                }
                for (_, t) in &info.def.fields {
                    self.fresh_domains_in(t, dom_start, seen_structs, seen_enums, out);
                }
            }
            Ty::Enum(eid) => {
                if !seen_enums.insert(*eid) {
                    return;
                }
                for (_, fields) in &self.enums[*eid].variants {
                    for t in fields {
                        self.fresh_domains_in(t, dom_start, seen_structs, seen_enums, out);
                    }
                }
            }
            _ => {}
        }
    }

    pub(super) fn clone_fresh_domain_fn(&mut self, orig: FnId) -> FnId {
        let decl = self.sigs[orig].decl;
        let mod_id = self.sigs[orig].mod_id;
        let offset = self.sigs[orig].offset;
        let owner = self.sigs[orig].owner;
        let env = self.sigs[orig].env.clone();
        let new_id = self.declare_fn_in(decl, mod_id, offset, owner, env);
        self.sigs[new_id].name = format!("{}_{new_id}", self.sigs[new_id].name);
        self.resolve_fn_sig(new_id, None);
        new_id
    }
}
