//! How one item's check reaches the rest of the file: names, imports,
//! effects and replays it reads, each recorded as a fingerprinted dependency.

use super::*;

impl FileCtx<'_> {
    pub(super) fn record(&mut self, key: &str, fingerprint: u64) {
        if let Some(frame) = self.frames.last_mut() {
            if !frame.iter().any(|(k, _)| k == key) {
                frame.push((key.to_string(), fingerprint));
            }
        }
    }

    /// The current fingerprint of a dependency key.
    pub(super) fn fingerprint(&mut self, key: &str) -> u64 {
        match key {
            DEP_NAMES => self.scope.names_fp,
            DEP_IMPORTS => self.scope.imports_fp,
            DEP_LENT => self.replays_fingerprint(),
            key if key.starts_with(fields::DEP_FIELD) => self.field_fp(key),
            key => match (key.strip_prefix(DEP_TYPE), key.strip_prefix(DEP_EFFECTS)) {
                (Some(name), _) => self.type_def(name).1,
                (_, Some(name)) => self.effects(name).1,
                _ => self.def(key).1,
            },
        }
    }

    pub(super) fn deps_hold(&mut self, deps: &[(String, u64)]) -> bool {
        deps.iter()
            .all(|(key, fingerprint)| self.fingerprint(key) == *fingerprint)
    }

    /// The definition of a top-level name and its fingerprint (0 if the
    /// file does not define it).
    pub(super) fn def(&mut self, name: &str) -> (Option<Arc<Def>>, u64) {
        let Some(&index) = self.scope.names.get(name) else {
            return self.imported(name);
        };
        if self.in_progress.iter().any(|n| n == name) {
            return (None, 0);
        }
        let item = self.items[index].clone();
        let revision = self.db.revision;
        let base = crate::prelude::overload_part(name).map_or(rebind::base(name), |(n, _)| n);
        let key = (self.file, item.hash, base.to_string());
        if let Some(memo) = self.db.defs.get_mut(&key) {
            memo.last_used = revision;
            if memo.verified_at == revision {
                return (Some(memo.def.clone()), memo.fingerprint);
            }
            let deps = memo.deps.clone();
            self.in_progress.push(name.to_string());
            let holds = self.deps_hold(&deps);
            self.in_progress.pop();
            if holds {
                let memo = self.db.defs.get_mut(&key).unwrap();
                memo.verified_at = revision;
                return (Some(memo.def.clone()), memo.fingerprint);
            }
        }

        // One statement can define several names (`let (a, b) = ...`, an
        // enum and its variants); they are computed and stored together.
        self.in_progress.push(name.to_string());
        self.frames.push(Vec::new());
        let mut defs = Vec::new();
        {
            let mut checker = Checker::new(self, &item.text, Vec::new());
            for stmt in &item.parsed.stmts {
                let found = checker.defs_of(stmt);
                if head_is_poisoned(&item.text, stmt, &item.parsed.poisoned) {
                    let annotation_broken = match &stmt.kind {
                        ast::StmtKind::Let { ty: Some(ty), .. } => item
                            .parsed
                            .poisoned
                            .iter()
                            .any(|p| p.start < ty.span.end && p.end > ty.span.start),
                        _ => crate::def::guessed(stmt),
                    } || head_leaves_open(&item.text, stmt);
                    defs.extend(
                        found
                            .into_iter()
                            .map(|(name, def)| (name, def.recovered(annotation_broken))),
                    );
                } else {
                    defs.extend(found);
                }
            }
        }
        let deps = self.frames.pop().unwrap();
        self.in_progress.pop();
        self.db.stats.defs_computed += 1;

        let mut found = (None, 0);
        let mut stored: Vec<String> = Vec::new();
        for (def_name, def) in defs {
            // A name being worked out further out was cut short here, and
            // that computation stores its own whole def.
            if stored.contains(&def_name) || self.in_progress.contains(&def_name) {
                continue;
            }
            let fingerprint = def.fingerprint();
            let def = Arc::new(def);
            if def_name == base {
                found = (Some(def.clone()), fingerprint);
            }
            stored.push(def_name.clone());
            self.db.defs.insert(
                (self.file, item.hash, def_name),
                DefMemo {
                    def,
                    fingerprint,
                    deps: deps.clone(),
                    verified_at: revision,
                    last_used: revision,
                },
            );
        }
        found
    }

    /// The definition of a name this file does not define: one it imports
    /// (`Monoid` after `use std.algebra.{Monoid}`, `fl.Graph` after `use
    /// lib.flow as fl`), or an impl in a module it reaches.
    pub(super) fn imported(&mut self, name: &str) -> (Option<Arc<Def>>, u64) {
        let scope = self.scope.clone();
        if let Some(import) = scope.imports.get(name) {
            return self.module_def(&scope.modules[import.module], &import.name);
        }
        if let Some((alias, member)) = name.split_once('.') {
            return match scope.aliases.get(alias) {
                Some(&index) => {
                    let module = &scope.modules[index];
                    let public = module.analysis.public_names();
                    if public.iter().any(|n| n == member) {
                        self.module_def(module, member)
                    } else {
                        (None, 0)
                    }
                }
                None => (None, 0),
            };
        }
        if impl_parts(name).is_some() {
            return self.reached(name);
        }
        (None, 0)
    }

    /// The definition behind a type's name: this file's own or imported one, else one in a
    /// module it reaches (a library's value can come back without importing its type).
    pub(super) fn type_def(&mut self, name: &str) -> (Option<Arc<Def>>, u64) {
        match self.def(name) {
            (None, _) => self.reached(name),
            found => found,
        }
    }

    pub(super) fn module_def(&mut self, module: &Module, name: &str) -> (Option<Arc<Def>>, u64) {
        match self.db.lookup_def(&module.path, &module.analysis, name) {
            (Some(def), fingerprint) => (Some(def), fingerprint),
            (None, _) => (None, 0),
        }
    }

    /// The earlier top-level statements whose effect reaches item `current`: views still named
    /// at or after it, and asserts none of whose names was assigned since. Usually empty.
    pub(super) fn replays(&mut self, current: usize) -> Vec<Replay> {
        let items = self.items.clone();
        if matches!(
            items[current].parsed.stmts.first().map(|s| &s.kind),
            Some(
                StmtKind::Fn(_)
                    | StmtKind::Struct(_)
                    | StmtKind::Enum(_)
                    | StmtKind::Trait { .. }
                    | StmtKind::Impl { .. }
            )
        ) {
            return Vec::new();
        }
        if !items[..current].iter().any(|item| item.flow != Flow::None) {
            return Vec::new();
        }
        let names = self.names.get_or_insert_with(|| {
            items
                .iter()
                .map(|item| item_names(&item.parsed.stmts))
                .collect()
        });
        let mut carried = crate::fx::HashSet::default(); // views a closure carries here
        for k in (0..current).rev() {
            // `let d = o!` lends what `o` lends, so `o`'s loan lives while `d` does.
            if let Flow::View(name) | Flow::LentVar(name) = &items[k].flow {
                let live = carried.contains(name)
                    || (current..items.len()).any(|j| names[j].0.contains(name));
                if let (true, [stmt]) = (live, &items[k].parsed.stmts[..]) {
                    if let Some(src) = crate::check::lends::unwrapped_name(stmt) {
                        carried.insert(src.to_string());
                    }
                }
            }
            if let Flow::Capture(name) = &items[k].flow {
                if carried.contains(name)
                    || (current..items.len()).any(|j| names[j].0.contains(name))
                {
                    carried.extend(names[k].0.iter().cloned());
                }
            }
        }
        let mut out = Vec::new();
        let offsets = &self.offsets;
        for (index, item) in items[..current].iter().enumerate() {
            let back = offsets.get(current).zip(offsets.get(index));
            let back = back.map_or(0, |(c, i)| c - i);
            let replay = |live_after, written| Replay {
                item: item.clone(),
                live_after,
                written,
                back,
            };
            match &item.flow {
                Flow::None => {}
                Flow::View(name) | Flow::Capture(name) | Flow::LentVar(name) => {
                    let mentioned = |k: usize| names[k].0.contains(name);
                    let live_after =
                        carried.contains(name) || (current + 1..items.len()).any(mentioned);
                    if !(live_after || mentioned(current)) {
                        continue;
                    }
                    // A later `let` of the same name is another value.
                    if (index + 1..current).any(|k| items[k].decls.iter().any(|d| d.name == *name))
                    {
                        continue;
                    }
                    // A later store into the same `var` replaces the value, and its loan.
                    let restored =
                        |k: usize| matches!(&items[k].flow, Flow::LentVar(n) if n == name);
                    if matches!(item.flow, Flow::LentVar(_)) && (index + 1..current).any(restored) {
                        continue;
                    }
                    let written = (current..items.len()).any(|k| names[k].1.contains(name));
                    out.push(replay(live_after, written));
                }
                Flow::Known(name) => {
                    let redeclared = (index + 1..current)
                        .any(|k| items[k].decls.iter().any(|d| d.name == *name));
                    if names[current].0.contains(name) && !redeclared {
                        out.push(replay(false, false));
                    }
                }
                Flow::Assert(about) => {
                    let assigned = (index + 1..current)
                        .any(|k| names[k].1.iter().any(|written| about.contains(written)));
                    if !assigned {
                        out.push(replay(false, false));
                    }
                }
                Flow::Move(srcs) => {
                    let any_live = srcs.iter().any(|src| {
                        names[current].0.contains(src)
                            && !(index + 1..current).any(|k| {
                                names[k].1.contains(src)
                                    || items[k].decls.iter().any(|d| d.name == *src)
                            })
                    });
                    if any_live {
                        out.push(replay(false, false));
                    }
                }
            }
        }
        out
    }

    /// What the body of this file's fn `name` touches by itself: the top-level `var`s it
    /// changes and reads, and the fns of this file it calls. `None` if there is no such fn.
    pub(super) fn own_effects(&mut self, name: &str) -> Option<Arc<Effects>> {
        if let Some(known) = self.own_effects.get(name) {
            return known.clone();
        }
        let items = self.items.clone();
        let vars = self.vars.get_or_insert_with(|| {
            let mutable = |item: &&Arc<ItemSyntax>| {
                item.parsed.stmts.iter().any(|stmt| {
                    matches!(
                        stmt.kind,
                        StmtKind::Let { mutable: true, .. }
                            | StmtKind::Let {
                                init: Some(cpl_syntax::ast::Expr {
                                    kind: cpl_syntax::ast::ExprKind::Mut(_),
                                    ..
                                }),
                                ..
                            }
                    )
                })
            };
            items
                .iter()
                .filter(mutable)
                .flat_map(|item| item.decls.iter().map(|decl| decl.name.clone()))
                .collect()
        });
        let own = self.scope.names.get(name).and_then(|&index| {
            items[index]
                .parsed
                .stmts
                .iter()
                .find_map(|stmt| match &stmt.kind {
                    StmtKind::Fn(f) if f.name.text == name => Some(own_effects(f)),
                    StmtKind::Let {
                        pat:
                            cpl_syntax::ast::Pattern {
                                kind: cpl_syntax::ast::PatternKind::Name(n),
                                ..
                            },
                        init:
                            Some(cpl_syntax::ast::Expr {
                                kind: cpl_syntax::ast::ExprKind::Lambda { params, body, .. },
                                ..
                            }),
                        ..
                    } if n == name => Some(crate::check::lends::lambda_effects(params, body)),
                    _ => None,
                })
        });
        let own = own.map(|mut own| {
            own.writes.retain(|n| vars.contains(n));
            own.reads.retain(|n| vars.contains(n));
            let scope = &self.scope;
            own.calls.retain(|n| {
                scope.names.get(n).is_some_and(|&index| {
                    items[index]
                        .decls
                        .iter()
                        .any(|d| d.name == *n && d.kind == DeclKind::Fn)
                })
            });
            Arc::new(own)
        });
        self.own_effects.insert(name.to_string(), own.clone());
        own
    }

    /// What calling this file's fn `name` touches outside itself, counting the fns it calls, and
    /// the fingerprint of that. A fn from a module touches nothing here.
    pub(super) fn effects(&mut self, name: &str) -> (Arc<Effects>, u64) {
        if let Some(known) = self.effects.get(name) {
            return known.clone();
        }
        let mut all = Effects::default();
        // Each fn is asked once: a fn it calls answers from what it
        // already worked out. A fn that calls itself through others, or a
        // very long chain of calls, is followed with a list instead.
        let nested = self.effects_of.iter().any(|n| n == name) || self.effects_of.len() > 200;
        if nested {
            let mut todo = vec![name.to_string()];
            let mut seen: Vec<String> = Vec::new();
            while let Some(next) = todo.pop() {
                if seen.contains(&next) {
                    continue;
                }
                if let Some((known, _)) = self.effects.get(&next) {
                    all.absorb(known);
                } else if let Some(own) = self.own_effects(&next) {
                    all.absorb(&own);
                    todo.extend(own.calls.iter().cloned());
                }
                seen.push(next);
            }
        } else if let Some(own) = self.own_effects(name) {
            all.absorb(&own);
            self.effects_of.push(name.to_string());
            for callee in &own.calls {
                let (theirs, _) = self.effects(callee);
                all.absorb(&theirs);
            }
            self.effects_of.pop();
        }
        all.writes.sort();
        all.reads.sort();
        let fingerprint = hash_str(&format!("{:?}{:?}", all.writes, all.reads)) | 2;
        let out = (Arc::new(all), fingerprint);
        self.effects.insert(name.to_string(), out.clone());
        out
    }

    pub(super) fn replays_fingerprint(&mut self) -> u64 {
        let Some(current) = self.current else {
            return 2;
        };
        let replays = self.replays(current);
        if replays.is_empty() {
            return 2;
        }
        let text: Vec<String> = replays
            .iter()
            // Only a move's span reaches back into its item.
            .map(|r| (r, matches!(r.item.flow, Flow::Move(_)) as u32 * r.back))
            .map(|(r, back)| format!("{}:{}:{}:{back}", r.item.hash, r.live_after, r.written))
            .collect();
        hash_str(&text.join("\n")) | 2
    }

    pub(super) fn check_item(&mut self, index: usize) -> Arc<ItemCheck> {
        self.current = Some(index);
        let item = self.items[index].clone();
        let revision = self.db.revision;
        let key = (self.file, item.hash);
        if let Some(memo) = self.db.checks.get_mut(&key) {
            memo.last_used = revision;
            let result = memo.result.clone();
            if memo.verified_at == revision {
                return result;
            }
            let deps = memo.deps.clone();
            if self.deps_hold(&deps) {
                self.db.checks.get_mut(&key).unwrap().verified_at = revision;
                self.db.stats.items_reused += 1;
                return result;
            }
        }
        self.frames.push(Vec::new());
        let checked = {
            let mut checker = Checker::new(self, &item.text, item.parsed.poisoned.clone());
            checker.item(&item.parsed.stmts);
            checker.finish()
        };
        let deps = self.frames.pop().unwrap();
        self.db.stats.items_checked += 1;
        let result = Arc::new(ItemCheck {
            diagnostics: checked.diagnostics,
            hovers: checked.hovers,
            refs: checked.refs,
            resolved: checked.resolved,
            ambiguous: checked.ambiguous,
            typed_indexes: checked.typed_indexes,
            fixed_io: checked.fixed_io,
            io_costs: checked.io_costs,
            exprs: checked.exprs,
        });
        self.db.checks.insert(
            key,
            CheckMemo {
                result: result.clone(),
                deps,
                verified_at: revision,
                last_used: revision,
            },
        );
        result
    }
}
