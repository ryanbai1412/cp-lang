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
        let name = &self.rebound(name);
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

    fn decl_order(&mut self, name: &str) -> Option<usize> {
        self.record(DEP_NAMES, self.scope.names_fp);
        self.scope.names.get(name).copied()
    }

    fn declares_type(&mut self, name: &str) -> bool {
        let Some(k) = self.decl_order(name) else {
            return false;
        };
        let first = self.items[k].parsed.stmts.first().map(|s| &s.kind);
        matches!(first, Some(StmtKind::Struct(_) | StmtKind::Enum(_)))
    }

    fn declares_alias(&mut self, name: &str) -> bool {
        let Some(k) = self.decl_order(name) else {
            return false;
        };
        let first = self.items[k].parsed.stmts.first().map(|s| &s.kind);
        matches!(first, Some(StmtKind::TypeAlias { .. }))
    }

    fn alias_rhs(&mut self, name: &str) -> Option<cpl_syntax::ast::Type> {
        let k = self.decl_order(name)?;
        match self.items[k].parsed.stmts.first().map(|s| &s.kind) {
            Some(StmtKind::TypeAlias { ty, .. }) => Some(ty.clone()),
            _ => None,
        }
    }

    fn outer_global(&mut self, name: &str) -> Option<Arc<Def>> {
        let (def, fingerprint) = self.imported(name);
        self.record(name, fingerprint);
        def
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

    fn variant_enums(&mut self, name: &str) -> Vec<String> {
        let mut enums = Vec::new();
        for item in self.items.clone().iter() {
            let mut owner = None;
            for decl in &item.decls {
                match decl.kind {
                    DeclKind::Enum => {
                        // Any enum gaining or losing the variant changes the answer.
                        let (_, fingerprint) = self.type_def(&decl.name);
                        self.record(&format!("{DEP_TYPE}{}", decl.name), fingerprint);
                        owner = Some(decl.name.clone());
                    }
                    DeclKind::Variant if decl.name == name => enums.extend(owner.clone()),
                    _ => {}
                }
            }
        }
        enums
    }

    fn method_keys(&mut self, name: &str) -> Vec<String> {
        self.record(DEP_IMPORTS, self.scope.imports_fp);
        let prefix = crate::prelude::method_key(name, "");
        let mut keys: Vec<String> = (self.scope.imports.keys())
            .filter(|k| k.starts_with(&prefix))
            .cloned()
            .collect();
        keys.sort();
        keys
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
            .filter(|name| impl_parts(name).is_none() && !super::rebind::is_version(name))
            .cloned()
            .collect();
        names.sort();
        names
    }

    fn declared(&mut self) -> Vec<String> {
        self.record(DEP_NAMES, self.scope.names_fp);
        let names = self.scope.names.keys();
        names
            .filter(|name| impl_parts(name).is_none() && !super::rebind::is_version(name))
            .cloned()
            .collect()
    }
}
