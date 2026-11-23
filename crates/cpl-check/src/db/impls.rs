//! Every impl in a file's reach, indexed once by the type it is for.

use super::Scope;
use crate::def::impl_parts;
use crate::fx::HashMap;

/// Each impl's name and trait (`None` for an inherent impl), by the type it
/// is for, sorted and without duplicates.
pub(super) type ImplNames = HashMap<String, Vec<(String, Option<String>)>>;

pub(super) fn impls_by_target(scope: &Scope) -> ImplNames {
    let reached = scope
        .modules
        .iter()
        .flat_map(|module| module.analysis.scope.names.keys());
    let mut by_target = ImplNames::default();
    for name in scope.names.keys().chain(reached) {
        if let Some((trait_name, of)) = impl_parts(name) {
            by_target
                .entry(of.to_string())
                .or_default()
                .push((name.clone(), trait_name.map(str::to_string)));
        }
    }
    for names in by_target.values_mut() {
        names.sort();
        names.dedup();
    }
    by_target
}
