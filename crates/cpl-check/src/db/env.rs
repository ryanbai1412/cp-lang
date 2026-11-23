//! What the checker asks a file's database while it checks one item: the
//! definitions, effects and impls behind the names it meets.

use super::*;

impl Env for FileCtx<'_> {
    fn origin(&mut self, name: &str) -> Option<String> {
        if self.scope.names.contains_key(name) {
            return None;
        }
        let module = match self.scope.imports.get(name) {
            Some(import) => import.module,
            None => *self.scope.aliases.get(name.split_once('.')?.0)?,
        };
        Some(self.scope.modules[module].name.clone())
    }

    fn global(&mut self, name: &str) -> Option<Arc<Def>> {
        let (def, fingerprint) = self.def(name);
        self.record(name, fingerprint);
        def
    }

    fn effects(&mut self, name: &str) -> Arc<Effects> {
        let (effects, fingerprint) = self.effects(name);
        self.record(&format!("{DEP_EFFECTS}{name}"), fingerprint);
        effects
    }

    fn field_decl(&mut self, ty: &str, field: &str) -> Option<Span> {
        self.field_decl_at(ty, field)
    }

    fn lent_before(&mut self) -> Vec<Replay> {
        let fingerprint = self.replays_fingerprint();
        self.record(DEP_LENT, fingerprint);
        match self.current {
            Some(current) if fingerprint != 2 => self.replays(current),
            _ => Vec::new(),
        }
    }

    fn type_def(&mut self, name: &str) -> Option<Arc<Def>> {
        let (def, fingerprint) = self.type_def(name);
        self.record(&format!("{DEP_TYPE}{name}"), fingerprint);
        def
    }

    fn is_module(&mut self, alias: &str) -> bool {
        self.record(DEP_IMPORTS, self.scope.imports_fp);
        self.scope.aliases.contains_key(alias)
    }

    fn is_private_member(&mut self, alias: &str, member: &str) -> bool {
        self.record(DEP_IMPORTS, self.scope.imports_fp);
        let Some(&idx) = self.scope.aliases.get(alias) else {
            return false;
        };
        let module = &self.scope.modules[idx];
        module.analysis.scope.names.contains_key(member)
            && !module.analysis.public_names().iter().any(|n| n == member)
    }

    fn is_private_field(&mut self, ty: &str, member: &str) -> bool {
        self.member_private(ty, member)
    }

    fn impls(&mut self, target: &str) -> Vec<(Option<String>, Arc<Def>)> {
        self.record(DEP_NAMES, self.scope.names_fp);
        self.record(DEP_IMPORTS, self.scope.imports_fp);
        let scope = self.scope.clone();
        let by_target = self
            .impl_names
            .get_or_insert_with(|| impls_by_target(&scope));
        let names = by_target.get(target).cloned().unwrap_or_default();
        names
            .into_iter()
            .filter_map(|(name, trait_name)| Some((trait_name, self.global(&name)?)))
            .collect()
    }

    fn has_imports(&mut self) -> bool {
        self.record(DEP_IMPORTS, self.scope.imports_fp);
        self.scope.has_imports
    }

    fn names(&mut self) -> Vec<String> {
        self.record(DEP_NAMES, self.scope.names_fp);
        self.record(DEP_IMPORTS, self.scope.imports_fp);
        let mut names: Vec<String> = self
            .scope
            .names
            .keys()
            .chain(self.scope.imports.keys())
            .chain(self.scope.aliases.keys())
            .filter(|name| impl_parts(name).is_none())
            .cloned()
            .collect();
        names.sort();
        names
    }

    fn declared(&mut self) -> Vec<String> {
        self.record(DEP_NAMES, self.scope.names_fp);
        let names = self.scope.names.keys();
        names
            .filter(|name| impl_parts(name).is_none())
            .cloned()
            .collect()
    }
}
