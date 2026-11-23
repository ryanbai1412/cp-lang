//! `impl Show` inside a compound value: a tuple, record, struct, optional or
//! list holding a type with a `show` fn prints that part through the fn,
//! as the generated C++ does. `call_direct` runs any such hook (`Eq`, `Ord`,
//! `Show`).

use crate::observe::Observer;
use crate::value::{OptVal, TupleVal, Value};
use crate::*;

impl<'p, 'o, O: Observer> Interp<'p, 'o, O> {
    /// Stops the run before a call would overflow the interpreter's own
    /// stack: the program's recursion is the interpreter's.
    pub(crate) fn call_direct(&mut self, id: FnId, values: Vec<Value>, span: Span) -> Res<Value> {
        let f = &self.program.fns[id];
        self.stack_left(span, Some(&f.name))?;
        let base = self.stack.len();
        self.stack.extend(values);
        self.stack.resize(base + f.locals.len(), Value::Unit);
        let saved = std::mem::replace(&mut self.base, base);
        self.frame_locals.push(&f.locals);
        self.call_frames.push((f.name.as_str(), span));
        let dom_mark = self.saved_domains.len();
        for &d in &self.fn_guarded[id] {
            self.saved_domains.push((d, self.domains[d]));
        }
        #[cfg(not(target_arch = "wasm32"))]
        let flow = match self.vm.clone() {
            Some(vm) if self.on_vm(id) => self.run_fn_vm_waited(&vm, id, span, saved, dom_mark),
            _ => self.block(&f.body),
        };
        #[cfg(target_arch = "wasm32")]
        let flow = self.block(&f.body);
        while self.saved_domains.len() > dom_mark {
            let (id, prev) = self.saved_domains.pop().unwrap();
            self.domains[id] = prev;
        }
        self.frame_locals.pop();
        self.base = saved;
        let flow = flow?;
        self.call_frames.pop();
        self.stack.truncate(base);
        Ok(match flow {
            Flow::Return(value) => value,
            _ => Value::Unit,
        })
    }

    /// Whether fn `id` runs on the VM, which keeps a one-`Int` struct such
    /// as `Mod<P>` as the `Int`.
    fn on_vm(&self, id: FnId) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        return (self.vm.as_ref()).is_some_and(|vm| vm.fns.get(id).is_some_and(Option::is_some));
        #[cfg(target_arch = "wasm32")]
        return false;
    }

    /// `value` with each part a `show` fn prints replaced by its text, typed
    /// to match; `None` when no part of `ty` has a `show` fn.
    pub(crate) fn shown_parts(
        &mut self,
        value: &Value,
        ty: &Ty,
        span: Span,
    ) -> Res<Option<(Value, Ty)>> {
        if !self.has_show_hook(ty, &mut Vec::new()) {
            return Ok(None);
        }
        self.show_parts(value, ty, span).map(Some)
    }

    fn show_parts(&mut self, value: &Value, ty: &Ty, span: Span) -> Res<(Value, Ty)> {
        if let Some(show_fn) = self.program.show_fn(ty) {
            return Ok((
                self.call_direct(show_fn, vec![value.clone()], span)?,
                Ty::Str,
            ));
        }
        let fields: Option<Vec<Ty>> = match ty {
            Ty::Struct(id) => Some(
                self.program.structs[*id]
                    .fields
                    .iter()
                    .map(|(_, t)| t.clone())
                    .collect(),
            ),
            Ty::Tuple(_) | Ty::Record(_) => ty.fields(),
            _ => None,
        };
        match (value, ty, fields) {
            (Value::Tuple(vals), _, Some(tys)) => {
                let mut scratch = [Value::Unit];
                let vals = vals.as_slice(&mut scratch).to_vec();
                let cached = match ty {
                    Ty::Struct(id) => self.program.structs[*id].cached_fields.clone(),
                    _ => Default::default(),
                };
                let (mut out, mut out_tys) = (Vec::new(), Vec::new());
                for (i, (v, t)) in vals.iter().zip(&tys).enumerate() {
                    if !cached.contains(&i) {
                        let (v, t) = self.show_parts(v, t, span)?;
                        out.push(v);
                        out_tys.push(t);
                    }
                }
                Ok((Value::tuple(out), Ty::Tuple(out_tys)))
            }
            (Value::Opt(OptVal::Boxed(inner)), Ty::Opt(t), _) => {
                let (v, t) = self.show_parts(inner, t, span)?;
                Ok((Value::Opt(OptVal::Boxed(Rc::new(v))), Ty::Opt(Box::new(t))))
            }
            (Value::List(items), Ty::List(t), _) => {
                let mut out = Vec::with_capacity(items.len());
                let mut elem = (**t).clone();
                for v in items.iter() {
                    let (v, t) = self.show_parts(v, t, span)?;
                    out.push(v);
                    elem = t;
                }
                Ok((Value::List(Rc::new(out)), Ty::List(Box::new(elem))))
            }
            _ => Ok((value.clone(), ty.clone())),
        }
    }

    fn has_show_hook(&self, ty: &Ty, seen: &mut Vec<usize>) -> bool {
        if self.program.show_fn(ty).is_some() {
            return true;
        }
        match ty {
            Ty::Struct(id) if !seen.contains(id) => {
                seen.push(*id);
                let fields = &self.program.structs[*id].fields;
                fields.iter().any(|(_, t)| self.has_show_hook(t, seen))
            }
            Ty::Tuple(ts) => ts.iter().any(|t| self.has_show_hook(t, seen)),
            Ty::Record(fs) => fs.iter().any(|(_, t)| self.has_show_hook(t, seen)),
            Ty::Opt(t) | Ty::List(t) => self.has_show_hook(t, seen),
            _ => false,
        }
    }

    /// Runs the `show` fn of each value the trace is about to show whose
    /// type has an `impl Show`, so the trace shows it as the program would
    /// print it. Each value's text is kept, and only new values run a fn.
    pub(crate) fn fill_shown(&mut self) {
        let program = self.program;
        let shows = |f: &Option<FnId>| f.is_some();
        if !program.structs.iter().any(|s| shows(&s.show_fn))
            && !program.enums.iter().any(|e| shows(&e.show_fn))
        {
            return;
        }
        let mut wanted = Vec::new();
        for (id, global) in program.globals.iter().enumerate() {
            if self.holds_show(&global.ty) {
                wanted_shows(program, &self.globals[id], &global.ty, &mut wanted);
            }
        }
        let tys: Vec<&Ty> = self.named_locals().map(|(_, ty, _)| ty).collect();
        let holds: Vec<bool> = tys.iter().map(|ty| self.holds_show(ty)).collect();
        if holds.contains(&true) {
            for ((_, ty, value), _) in self.named_locals().zip(&holds).filter(|(_, h)| **h) {
                wanted_shows(program, value, ty, &mut wanted);
            }
        }
        wanted.retain(|key| !self.trace_shown.contains_key(key));
        if wanted.is_empty() {
            return;
        }
        if self.trace_shown.len() + wanted.len() > 1 << 14 {
            self.trace_shown.clear();
        }
        let tracer = self.tracer.take();
        let (stack, frames) = (self.stack.len(), self.call_frames.len());
        for (f, value) in wanted {
            let arg = match value {
                Value::Int(x) if !self.on_vm(f) => Value::Tuple(TupleVal::OneInt(x)),
                ref other => other.clone(),
            };
            match self.call_direct(f, vec![arg], Span::default()) {
                Ok(Value::Str(bytes)) => {
                    let text = String::from_utf8_lossy(&bytes).into_owned();
                    self.trace_shown.insert((f, value), text);
                }
                _ => {
                    self.stack.truncate(stack);
                    self.call_frames.truncate(frames);
                }
            }
        }
        self.tracer = tracer;
    }

    fn holds_show(&mut self, ty: &Ty) -> bool {
        if let Some(&holds) = self.trace_show_tys.get(ty) {
            return holds;
        }
        let holds = holds_show(self.program, ty, &mut Vec::new());
        self.trace_show_tys.insert(ty.clone(), holds);
        holds
    }
}

/// Whether a value of `ty` can hold one whose type has a `show` fn.
fn holds_show(program: &Program, ty: &Ty, seen: &mut Vec<Ty>) -> bool {
    if program.show_fn(ty).is_some() {
        return true;
    }
    if seen.contains(ty) {
        return false;
    }
    seen.push(ty.clone());
    match ty {
        Ty::Struct(id) => program.structs[*id]
            .fields
            .iter()
            .any(|(_, t)| holds_show(program, t, seen)),
        Ty::Enum(id) => (program.enums[*id].variants.iter())
            .any(|(_, tys)| tys.iter().any(|t| holds_show(program, t, seen))),
        Ty::Tuple(ts) => ts.iter().any(|t| holds_show(program, t, seen)),
        Ty::Record(fs) => fs.iter().any(|(_, t)| holds_show(program, t, seen)),
        Ty::Opt(t) | Ty::List(t) | Ty::Set(t) | Ty::HashSet(t) | Ty::Array { elem: t, .. } => {
            holds_show(program, t, seen)
        }
        Ty::Map(k, v) | Ty::HashMap(k, v) => {
            holds_show(program, k, seen) || holds_show(program, v, seen)
        }
        _ => false,
    }
}

/// Each part of `value` whose type has a `show` fn, with that fn, in the
/// order the trace shows them. A long array is shown by its count, not
/// its cells, so they are skipped.
fn wanted_shows(program: &Program, value: &Value, ty: &Ty, out: &mut Vec<(FnId, Value)>) {
    if let Some(f) = program.show_fn(ty) {
        out.push((f, value.clone()));
        return;
    }
    let mut each = |items: &mut dyn Iterator<Item = (&Value, &Ty)>| {
        for (item, ty) in items {
            wanted_shows(program, item, ty, out);
        }
    };
    match (value, ty) {
        (Value::Opt(OptVal::Boxed(inner)), Ty::Opt(t)) => {
            each(&mut std::iter::once((&**inner, &**t)))
        }
        (Value::Tuple(fields), _) => {
            let tys: Vec<Ty> = match ty {
                Ty::Struct(id) => program.structs[*id]
                    .fields
                    .iter()
                    .map(|(_, t)| t.clone())
                    .collect(),
                other => other.fields().unwrap_or_default(),
            };
            let mut scratch = [Value::Unit];
            each(&mut fields.as_slice(&mut scratch).iter().zip(&tys));
        }
        (Value::Enum(tag, payload), Ty::Enum(id)) => {
            let tys = &program.enums[*id].variants[*tag as usize].1;
            each(&mut payload.iter().zip(tys));
        }
        (Value::Array(array), Ty::Array { elem, .. }) if array.len() <= 200 => {
            each(&mut array.to_values().iter().map(|v| (v, &**elem)));
        }
        (Value::List(items), Ty::List(elem)) if items.len() <= 200 => {
            each(&mut items.iter().map(|v| (v, &**elem)));
        }
        (Value::Set(items), Ty::Set(elem) | Ty::HashSet(elem)) if items.len() <= 200 => {
            each(&mut items.iter().map(|v| (v, &**elem)));
        }
        (Value::Map(entries), Ty::Map(k, v) | Ty::HashMap(k, v)) if entries.len() <= 200 => {
            each(&mut entries.iter().flat_map(|(a, b)| [(a, &**k), (b, &**v)]));
        }
        _ => {}
    }
}
