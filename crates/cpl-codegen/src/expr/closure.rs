//! Closures: captures, recursion slots and the lambda C++ they emit.

use super::*;

impl<'p> Emitter<'p> {
    pub(crate) fn emit_closure(&mut self, cid: usize, skip_captures: bool) -> Code {
        let def = &self.program.closures[cid];
        let rec_slot = self.rec_slot.take();
        // Prepare inner names for the closure's locals.
        let mut inner_taken = self.taken.clone();
        let mut inner_names = Vec::with_capacity(def.locals.len());
        let mut inner_fresh = 0usize;
        for local in &def.locals {
            let base = if local.name.is_empty() {
                let mut n;
                loop {
                    n = format!("c{cid}_{inner_fresh}");
                    inner_fresh += 1;
                    if !inner_taken.contains(&n) && !self.top_names.contains(&n) {
                        break;
                    }
                }
                n
            } else {
                let b = ident(&local.name);
                let mut n = b.clone();
                let mut k = 2;
                while inner_taken.contains(&n) || self.top_names.contains(&n) {
                    n = format!("{b}_{k}");
                    k += 1;
                }
                n
            };
            inner_taken.insert(base.clone());
            inner_names.push(base);
        }

        // Build C++17 capture list from enclosing names/globals.
        let mut cap_parts = Vec::new();
        let mut global_ref_aliases = Vec::new();
        let borrow_immutable = self.borrow_closures.contains(&cid);
        if !skip_captures {
            for cap in &def.captures {
                let inner = &inner_names[cap.inner];
                let non_scalar = !def.locals[cap.inner].ty.is_scalar();
                if !cap.is_global && Some(cap.outer) == rec_slot {
                    continue;
                }
                if cap.is_global {
                    let gname = ident(&self.program.globals[cap.outer].name);
                    if cap.by_ref
                        || (borrow_immutable
                            && non_scalar
                            && self.const_globals.contains(&cap.outer))
                    {
                        let constant = if self.const_globals.contains(&cap.outer) {
                            "const "
                        } else {
                            ""
                        };
                        global_ref_aliases.push(format!("{constant}auto& {inner} = {gname};"));
                    } else {
                        self.note_capture_copy(
                            cid,
                            non_scalar,
                            &self.program.globals[cap.outer].name,
                        );
                        cap_parts.push(format!("{inner} = {gname}"));
                    }
                } else {
                    let outer = &self.names[cap.outer];
                    if cap.by_ref || (borrow_immutable && non_scalar) {
                        cap_parts.push(format!("&{inner} = {outer}"));
                    } else {
                        cap_parts.push(format!("{inner} = {outer}"));
                        let name = self.locals()[cap.outer].name.clone();
                        self.note_capture_copy(cid, non_scalar, &name);
                    }
                }
            }
        }

        // Parameters.
        let mut params = Vec::new();
        for cap in &def.captures {
            if !cap.is_global && Some(cap.outer) == rec_slot {
                params.push(("auto&".to_string(), inner_names[cap.inner].clone()));
            }
        }
        for &slot in &def.params {
            let ty = &def.locals[slot].ty;
            let cpp = self.value_ty(ty);
            let cpp = if def.muts.contains(&slot) {
                format!("{cpp}&")
            } else if ty.is_scalar() || self.caches_on_read(ty) {
                cpp
            } else {
                format!("const {cpp}&")
            };
            params.push((cpp, inner_names[slot].clone()));
        }

        let ret_cpp = self.ty(&def.ret);
        let pre_visible = def.params.len() + def.captures.len();
        let inner_hoisted = crate::scope::hoisted(&def.body, pre_visible);

        // Save enclosing emitter state and emit body.
        let saved_closure = self.closure_id.replace(cid);
        let saved_names = std::mem::replace(&mut self.names, inner_names);
        let saved_hoisted = std::mem::replace(&mut self.hoisted, inner_hoisted);
        let saved_in_setup = std::mem::replace(&mut self.in_setup, false);
        let saved_bodies = std::mem::take(&mut self.bodies);
        let inner_spent = crate::scope::spent(self.program, &def.body);
        let saved_spent = std::mem::replace(&mut self.spent, inner_spent);
        let saved_lent = std::mem::take(&mut self.lent);
        let saved_taken = std::mem::replace(&mut self.taken, inner_taken);
        let saved_fresh = std::mem::replace(&mut self.fresh, inner_fresh);
        let saved_brk = std::mem::take(&mut self.break_targets);
        let saved_cont = std::mem::take(&mut self.continue_targets);

        walk::deep_stmts(&def.body, &mut |stmt| match stmt {
            Stmt::Break(Some(id)) => {
                self.break_targets.insert(*id);
            }
            Stmt::Continue(Some(id)) => {
                self.continue_targets.insert(*id);
            }
            _ => {}
        });

        let mut text = String::new();
        self.indent += 1;
        for alias in global_ref_aliases {
            text.push_str(&format!("{}{alias}\n", pad(self.indent)));
        }
        // Declare any hoisted locals inside the closure.
        for slot in self.hoisted.clone() {
            if def.params.contains(&slot)
                || def.captures.iter().any(|c| c.inner == slot)
                || def.locals[slot].ty == Ty::Unit
            {
                continue;
            }
            let ty = &def.locals[slot].ty;
            text.push_str(&format!(
                "{}{} {}{};\n",
                pad(self.indent),
                self.ty(ty),
                self.names[slot],
                self.zero_init(ty)
            ));
        }
        self.block(&def.body, &mut text);
        self.indent -= 1;
        text.push_str(&pad(self.indent));
        text.push('}');
        // A parameter the body never names is left unnamed, as `_` is
        // in the source: g++ warns about a named one.
        let names = |name: &str| {
            let word = |c: char| c.is_ascii_alphanumeric() || c == '_';
            text.match_indices(name).any(|(at, _)| {
                !text[..at].ends_with(word) && !text[at + name.len()..].starts_with(word)
            })
        };
        let param_parts: Vec<String> = params
            .iter()
            .map(|(cpp, name)| {
                if names(name) {
                    format!("{cpp} {name}")
                } else {
                    cpp.clone()
                }
            })
            .collect();
        let text = format!(
            "[{}]({}) mutable -> {ret_cpp} {{\n{text}",
            cap_parts.join(", "),
            param_parts.join(", ")
        );

        // Restore enclosing emitter state.
        self.closure_id = saved_closure;
        self.names = saved_names;
        self.hoisted = saved_hoisted;
        self.in_setup = saved_in_setup;
        self.bodies = saved_bodies;
        self.spent = saved_spent;
        self.lent = saved_lent;
        self.taken = saved_taken;
        self.fresh = saved_fresh;
        self.break_targets = saved_brk;
        self.continue_targets = saved_cont;

        Code::primary(text)
    }
}

impl Emitter<'_> {
    /// The C++ type of a value that is stored or passed: a `()` held in a
    /// tuple, a list or a parameter is a `cpl::Unit`, since C++ has no
    /// `void` value.
    pub(crate) fn value_ty(&self, ty: &Ty) -> String {
        match ty {
            Ty::Unit => {
                self.niche_types.set(self.niche_types.get() | 1 << 7);
                "cpl::Unit".into()
            }
            _ => self.ty(ty),
        }
    }

    /// `e` where a value is stored or passed. A `()` there is a
    /// `cpl::Unit`, made after whatever `e` runs.
    pub(crate) fn stored(&mut self, e: &Expr) -> String {
        let text = self.expr(e).text;
        self.as_stored(e, text)
    }

    pub(crate) fn as_stored(&self, e: &Expr, text: String) -> String {
        if e.ty != Ty::Unit {
            return text;
        }
        self.niche_types.set(self.niche_types.get() | 1 << 7);
        match e.kind {
            ExprKind::Index(..) | ExprKind::Field(..) => text,
            _ if text == "(void)0" || text == "void()" => "cpl::Unit()".into(),
            ExprKind::Unit | ExprKind::Tuple(_) | ExprKind::Local(_) | ExprKind::Global(_) => {
                "cpl::Unit()".into()
            }
            _ => format!("({text}, cpl::Unit())"),
        }
    }
}
