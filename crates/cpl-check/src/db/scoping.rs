//! Where a definition is found and when it runs: an inline module names its
//! siblings (`use a.f` inside `mod b:`, R11.1.3, R11.2.2), and a `domain`
//! without compile-time bounds runs where it is written, like a `let`
//! (R11.1.1).

use super::*;
use cpl_syntax::const_int;

impl Database {
    /// The path and text of `parts` seen from `from`: a sibling inline
    /// module first, then whatever the loader finds.
    pub(super) fn find_module(&self, from: &str, parts: &[String]) -> Option<(String, String)> {
        if let ([name], Some((file, _))) = (parts, from.rsplit_once("::mod::")) {
            let path = format!("{file}::mod::{name}");
            if let Some(state) = self.files.get(&path) {
                return Some((path, state.text.to_string()));
            }
        }
        if let Some(found) = self.loader.load(from, parts) {
            return Some(found);
        }
        // The loader has the libraries; std is this check's own.
        let (_, rest) = parts.split_first().filter(|(std, _)| *std == "std")?;
        let name = rest.join("/");
        let path = format!("std/{name}.cp");
        if let Some(text) = self.std.text(&name) {
            return Some((path, text.to_string()));
        }
        if !self.outlining {
            return None;
        }
        let text = crate::modules::outline_of(&name)?;
        Some((path, text.to_string()))
    }

    /// Writes down that the file being checked needs the std module `name`.
    pub(super) fn need_std(&mut self, name: String) {
        let root = self.analyzing.first();
        if let Some(file) = root.and_then(|root| self.files.get_mut(root)) {
            if let Err(at) = file.std_needs.binary_search(&name) {
                file.std_needs.insert(at, name);
            }
        }
    }

    /// The std modules the last check of the file `path` asked for and
    /// was not given, by name (`"sparse"`), sorted.
    pub fn std_needs(&self, path: &str) -> Vec<String> {
        (self.files.get(path)).map_or(Vec::new(), |file| file.std_needs.clone())
    }

    /// Checks against `std` from now on; true when that is a change, and
    /// every analysis is then forgotten, since any may have read std.
    pub fn set_std(&mut self, std: &std::sync::Arc<crate::GivenStd>) -> bool {
        if std::sync::Arc::ptr_eq(&self.std, std) || self.std == *std {
            return false;
        }
        self.std = std.clone();
        self.reload_modules();
        true
    }

    /// The module a `use` of `parts` loads, and the name in it when `lone`
    /// (`use std.math.gcd`: the last name is a name in the module before
    /// it). A std module not given yet loads as its outline meanwhile.
    pub(super) fn import_module<'a>(
        &mut self,
        from: &str,
        parts: &[String],
        whole: Span,
        lone: Option<&'a ast::Name>,
        diags: &mut Vec<Diagnostic>,
    ) -> (Option<Module>, Option<&'a ast::Name>) {
        let found = self.load_module(from, parts, whole, diags);
        if found.is_some() || !diags.is_empty() {
            return (found, None);
        }
        if let (Some(_), [module @ .., _]) = (lone, parts) {
            if let Some(found) = (!module.is_empty())
                .then(|| self.load_module(from, module, whole, diags))
                .flatten()
            {
                return (Some(found), lone);
            }
            if !diags.is_empty() {
                return (None, None);
            }
        }
        let lacked = self.std.outline(parts);
        if let Some((len, ..)) = &lacked {
            self.need_std(parts[1..*len].join("/"));
        }
        diags.push(crate::modules::missing(parts, whole, lacked.is_some()));
        let Some((len, path, text)) = lacked else {
            return (None, None);
        };
        if len < parts.len() && lone.is_none() {
            return (None, None);
        }
        self.store(&path, text.into(), true);
        let Some(analysis) = self.analyze_as(&path, true) else {
            return (None, None);
        };
        let name = parts[..len].join(".");
        let module = Module {
            name,
            path,
            analysis,
        };
        (Some(module), lone.filter(|_| len < parts.len()))
    }

    /// Forgets every analysis, for when what the loader finds changed
    /// (a std module arrived): the next one loads its modules again.
    pub fn reload_modules(&mut self) {
        for file in self.files.values_mut() {
            file.forget();
        }
    }
}

/// True when item `k` declares a domain whose bounds are not compile-time
/// integers, so it runs where it is written; a constant one is hoisted.
pub(super) fn in_place(items: &[Arc<ItemSyntax>], scope: &Scope, k: usize) -> bool {
    items[k].parsed.stmts.iter().any(|stmt| match &stmt.kind {
        StmtKind::Domain { def, .. } => {
            !const_int::constant_domain(def, &|n| const_value(items, scope, n, 0))
        }
        _ => false,
    })
}

/// The value of the top-level integer `const` called `name`.
fn const_value(items: &[Arc<ItemSyntax>], scope: &Scope, name: &str, depth: usize) -> Option<i64> {
    let &at = scope.names.get(name).filter(|_| depth < 64)?;
    items[at]
        .parsed
        .stmts
        .iter()
        .find_map(|stmt| match &stmt.kind {
            StmtKind::Const {
                name: n,
                generics,
                init: Some(init),
                ..
            } if n.text == name && generics.is_empty() => {
                const_int::eval(init, &|m| const_value(items, scope, m, depth + 1))
            }
            _ => None,
        })
}

/// R11.3.4: each `pub fn f(self: ..)` of a library module is a method its
/// receivers find, under its `method_key`, whatever else the file calls `f`.
pub(super) fn add_methods(scope: &mut Scope, index: usize) {
    let module = &scope.modules[index];
    if module.name.starts_with("std.") {
        return;
    }
    let mut found = Vec::new();
    for item in &module.analysis.items {
        for stmt in &item.syntax.parsed.stmts {
            let StmtKind::Fn(f) = &stmt.kind else {
                continue;
            };
            let public = item
                .syntax
                .decls
                .iter()
                .any(|d| d.public && d.name == f.name.text);
            if public && f.params.first().is_some_and(|p| p.name.text == "self") {
                let key = crate::prelude::method_key(&f.name.text, &module.name);
                found.push((key, f.name.text.clone()));
            }
        }
    }
    for (key, name) in found {
        add_overloads(scope, index, &name);
        scope.imports.entry(key).or_insert(Import {
            module: index,
            name,
        });
    }
}

/// The later `fn name(self: ..)`s of the module at `index`, each under
/// its own key beside the first: `bsearch of std.range#2` (R08.1.4).
pub(super) fn add_overloads(scope: &mut Scope, index: usize, name: &str) {
    let module = &scope.modules[index];
    let found: Vec<(String, String)> = (2..)
        .map(|k| (k, crate::prelude::overload_key(name, k)))
        .take_while(|(_, key)| module.analysis.scope.names.contains_key(key))
        .map(|(k, key)| {
            (
                crate::prelude::method_key(name, &format!("{}#{k}", module.name)),
                key,
            )
        })
        .collect();
    for (key, name) in found {
        scope.imports.entry(key).or_insert(Import {
            module: index,
            name,
        });
    }
}

/// What the library's `name` of `module` is imported under: its
/// `method_key` beside a name the file declares, or another module's of
/// the name (seq's `xs.fold_from(init, f)` beside the listed free
/// `std.algebra.fold`), so a call picks among them (R11.3.4).
pub(super) fn import_key(
    scope: &Scope,
    module: &[cpl_syntax::ast::Name],
    name: &str,
    method: bool,
    own: bool,
) -> String {
    let path: Vec<&str> = module.iter().map(|n| n.text.as_str()).collect();
    let path = path.join(".");
    let taken =
        method && (scope.imports.get(name)).is_some_and(|i| scope.modules[i.module].name != path);
    crate::prelude::import_key(module, name, method && (own || taken))
}
