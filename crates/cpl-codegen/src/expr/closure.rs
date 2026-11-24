//! Closures: captures, recursion slots and the lambda C++ they emit.

use super::*;

/// A closure's lambda C++ in pieces, so a closure type can reuse its body.
pub(crate) struct ClosureText {
    captures: Vec<String>,
    params: Vec<String>,
    ret: String,
    body: String,
    /// The inner name of each capture, in `captures` order.
    captured: Vec<String>,
}

impl<'p> Emitter<'p> {
    pub(crate) fn emit_closure(&mut self, cid: usize, skip_captures: bool) -> Code {
        if !skip_captures && self.devirt.functor(cid) {
            return self.functor_value(cid);
        }
        if !skip_captures {
            if let Some(code) = self.bound_value(cid) {
                return code;
            }
        }
        let t = self.closure_text(cid, skip_captures);
        Code::primary(format!(
            "[{}]({}) mutable -> {} {{\n{}",
            t.captures.join(", "),
            t.params.join(", "),
            t.ret,
            t.body
        ))
    }

    /// `cpl::bind_env([](env&, ..) {..}, {a, b})`: a capturing closure whose
    /// signature only ever holds closures with captures of these types, so
    /// its fn value is a plain function beside its captures. A closure that
    /// is only called where it is made lends its captures instead.
    fn bound_value(&mut self, cid: usize) -> Option<Code> {
        let def = &self.program.closures[cid];
        if def.captures.is_empty()
            || !def.muts.is_empty()
            || self.rec_slot.is_some()
            || self.borrow_closures.contains(&cid)
        {
            return None;
        }
        let args: Vec<Ty> = def
            .params
            .iter()
            .map(|&p| def.locals[p].ty.clone())
            .collect();
        let env = crate::names::fn_bound(self.program, &args, &def.ret)?;
        let fn_ty = self.ty(&Ty::Fn(args, Box::new(def.ret.clone())));
        if !fn_ty.starts_with("cpl::Bound<") {
            return None;
        }
        let outers: Vec<String> = def
            .captures
            .iter()
            .map(|cap| self.names[cap.outer].clone())
            .collect();
        let env_ty = format!(
            "cpl::Env<{}>",
            env.iter()
                .map(|t| self.value_ty(t))
                .collect::<Vec<_>>()
                .join(", ")
        );
        self.need("bound");
        self.written.bound.insert(cid);
        let t = self.closure_text(cid, true);
        let aliases: String = t
            .captured
            .iter()
            .enumerate()
            .map(|(k, name)| {
                format!(
                    "{}[[maybe_unused]] auto& {name} = cpl::at<{k}>(cpl_env);\n",
                    pad(self.indent + 1)
                )
            })
            .collect();
        let params: Vec<String> = std::iter::once(format!("{env_ty}& cpl_env"))
            .chain(t.params)
            .collect();
        Some(Code::primary(format!(
            "cpl::bind_env([]({}) -> {} {{\n{aliases}{}, {env_ty}{{{}}})",
            params.join(", "),
            t.ret,
            t.body,
            outers.join(", ")
        )))
    }

    /// `cpl_clN{a, b}`: closure `cid` built as its own type, each capture
    /// copied as the lambda's `[a = a]` would.
    fn functor_value(&mut self, cid: usize) -> Code {
        let def = &self.program.closures[cid];
        let parts: Vec<String> = def
            .captures
            .iter()
            .map(|cap| self.names[cap.outer].clone())
            .collect();
        Code::primary(format!(
            "{}{{{}}}",
            crate::devirt::functor_name(cid),
            parts.join(", ")
        ))
    }

    /// `struct cpl_clN { .. }`: one mutable member per capture, as a
    /// `mutable` lambda holds them, and a call operator defined later.
    pub(crate) fn functor_decl(&self, cid: usize) -> String {
        let def = &self.program.closures[cid];
        let mut out = format!("struct {} {{\n", crate::devirt::functor_name(cid));
        for (k, cap) in def.captures.iter().enumerate() {
            let ty = &def.locals[cap.inner].ty;
            out.push_str(&format!(
                "    mutable {} c{k}{};\n",
                self.ty(ty),
                self.zero_init(ty)
            ));
        }
        let params: Vec<String> = def
            .params
            .iter()
            .map(|&slot| self.closure_param_ty(cid, slot))
            .collect();
        out.push_str(&format!(
            "    {} operator()({}) const;\n}};\n",
            self.ty(&def.ret),
            params.join(", ")
        ));
        out
    }

    /// The call operator of `cpl_clN`: the lambda's body, each capture
    /// named by a reference to its member.
    pub(crate) fn functor_body(&mut self, cid: usize) -> String {
        let t = self.closure_text(cid, true);
        let aliases: String = t
            .captured
            .iter()
            .enumerate()
            .map(|(k, name)| {
                format!(
                    "{}[[maybe_unused]] auto& {name} = c{k};\n",
                    pad(self.indent + 1)
                )
            })
            .collect();
        format!(
            "inline {} {}::operator()({}) const {{\n{aliases}{}\n",
            t.ret,
            crate::devirt::functor_name(cid),
            t.params.join(", "),
            t.body
        )
    }

    fn closure_param_ty(&self, cid: usize, slot: usize) -> String {
        let def = &self.program.closures[cid];
        let ty = &def.locals[slot].ty;
        let cpp = self.value_ty(ty);
        if def.muts.contains(&slot) {
            format!("{cpp}&")
        } else if ty.is_scalar() || self.caches_on_read(ty) {
            cpp
        } else {
            format!("const {cpp}&")
        }
    }

    fn closure_text(&mut self, cid: usize, skip_captures: bool) -> ClosureText {
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
                let heap = crate::lend::holds_heap(self.program, &def.locals[cap.inner].ty);
                if !cap.is_global && Some(cap.outer) == rec_slot {
                    continue;
                }
                if cap.is_global {
                    let gname = crate::names::global_ident(self.program, cap.outer);
                    // A closure that lives only through its call reads in
                    // place a global it does not write itself.
                    let scoped = self.call_scoped.contains(&cid)
                        && self
                            .uses
                            .table
                            .closure_access(self.program, cid)
                            .is_some_and(|a| !a.writes.contains(&(true, cap.outer)));
                    let lent = borrow_immutable
                        && non_scalar
                        && (self.const_globals.contains(&cap.outer) || scoped);
                    if cap.by_ref || lent {
                        let constant = if lent || self.const_globals.contains(&cap.outer) {
                            "const "
                        } else {
                            ""
                        };
                        global_ref_aliases.push(format!("{constant}auto& {inner} = {gname};"));
                    } else {
                        let name = &self.program.globals[cap.outer].name;
                        self.note_capture_copy(cid, heap, name, crate::lend::KEPT);
                        cap_parts.push(format!("{inner} = {gname}"));
                    }
                } else {
                    let outer = &self.names[cap.outer];
                    let owns = self.capture_owns.get(&(cid, cap.outer)).copied();
                    if cap.by_ref || (borrow_immutable && non_scalar && owns.is_none()) {
                        cap_parts.push(format!("&{inner} = {outer}"));
                    } else {
                        cap_parts.push(format!("{inner} = {outer}"));
                        let name = self.locals()[cap.outer].name.clone();
                        self.note_capture_copy(cid, heap, &name, owns.unwrap_or(crate::lend::KEPT));
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
            params.push((self.closure_param_ty(cid, slot), inner_names[slot].clone()));
        }
        let captured: Vec<String> = def
            .captures
            .iter()
            .map(|c| inner_names[c.inner].clone())
            .collect();

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
        let (breaks, continues) = crate::uses::loops::labeled_jumps(&def.body);
        let saved_brk = std::mem::replace(&mut self.break_targets, breaks);
        let saved_cont = std::mem::replace(&mut self.continue_targets, continues);

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
        // A local fn can re-enter (recursion, or through a callee), so a
        // runtime domain it declares is saved and restored per frame.
        for d in self.program.fn_guarded_domains(&def.ret, &def.body) {
            self.need("dom_guard");
            let dn = crate::names::dom_ident(self.program, d);
            let g = self.temp();
            text.push_str(&format!(
                "{}cpl::DomGuard {g}({dn}_lo, {dn}_len);\n",
                pad(self.indent)
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

        ClosureText {
            captures: cap_parts,
            params: param_parts,
            ret: ret_cpp,
            body: text,
            captured,
        }
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
