//! A layer fn written against a view runs on a map as well as on the program's declarations.
use super::*;
use cpl_next_ir::ty::SetTy;

/// A view as a test writes it: one fn's summary in a map.
struct OneSummary(HashMap<(FnId, usize), SetTy>);

impl Summaries for OneSummary {
    fn adds(&self, f: FnId, slot: usize) -> Option<&SetTy> {
        self.0.get(&(f, slot))
    }
    fn global_set(&self, _: GlobalId, _: usize) -> Option<&SetTy> {
        None
    }
    fn added_at(&self, _: usize) -> Option<&(FnRef, usize)> {
        None
    }
}

/// What a caller reads of a callee: whether its body adds to a set parameter.
fn adds_any(s: &dyn Summaries, f: FnId, slots: usize) -> bool {
    (0..slots).any(|i| s.adds(f, i).is_some())
}

#[test]
fn a_summary_view_is_a_map_or_the_checkers_adds() {
    let f = FnId(3);
    let one = OneSummary(HashMap::from([((f, 1), SetTy::Var(0))]));
    assert!(adds_any(&one, f, 2));
    assert!(!adds_any(&one, FnId(4), 2));
    let mut adds = SetAdds::default();
    adds.params.insert((f, 1), SetTy::Var(0));
    assert!(adds_any(&adds, f, 2));
}

#[test]
fn the_declarations_are_the_signatures_view() {
    let mut d = Decls::default();
    let f = d.syms.intern("f");
    let names = HashMap::from([(f, Named::Fn(FnId(0)))]);
    d.views.push(std::sync::Arc::new(names));
    let sigs: &dyn Signatures = &d;
    assert!(sigs
        .impls()
        .ranked_impls(TraitId(0))
        .map(|(k, _)| k)
        .collect::<Vec<_>>()
        .is_empty());
    assert_eq!(d.named(ModId(0), "f"), Some(Named::Fn(FnId(0))));
    assert_eq!(d.named(ModId(0), "g"), None);
}

#[test]
fn lowered_fns_are_the_generics_view() {
    let fns: HashMap<FuncKey, Func<Ty>> = HashMap::new();
    let g: &dyn Generics = &fns;
    assert!(g.generic(&FuncKey::Main).is_none());
}
