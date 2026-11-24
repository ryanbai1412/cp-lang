//! Calls: feeding values into loops, and calling functions with their arguments.

use super::*;

impl<'p, 'o, O: Observer> Interp<'p, 'o, O> {
    /// The fn and its list when the loop `id` runs its body as the fn
    /// yields, as the C++ does.
    pub(super) fn is_fed(&mut self, id: LoopId, seq: &Expr, body: &Block) -> Option<(FnId, usize)> {
        if self.in_closure
            || self
                .fed_loops
                .last()
                .is_some_and(|fed| fed.callee == self.base)
        {
            return None;
        }
        if let Some(known) = self.fed.get(&id) {
            return *known;
        }
        let found = self.feeds.feeds(id, seq, body);
        self.fed.insert(id, found);
        found
    }

    /// One value of the `yields` fn that is running goes to the body of
    /// the loop it feeds. An `Err` with `left` set means the body left
    /// the loop, and the fn stops where it is.
    pub(super) fn feed(&mut self, value: Value, span: Span) -> Res<Value> {
        let fed = self.fed_loops.last().expect("asked by the caller");
        let (callee, caller, slot, id, body) = (fed.callee, fed.caller, fed.slot, fed.id, fed.body);
        let locals = self.frame_locals.pop();
        self.base = caller;
        self.stack[caller + slot] = value;
        self.fed_bodies += 1;
        self.feeding();
        if let Some(tracer) = self.tracer.as_mut() {
            tracer.enter_fed_body();
        }
        let flow = match self.fed_body_vm(id, caller) {
            Some(flow) => flow,
            None => self.block(body),
        };
        if let Some(tracer) = self.tracer.as_mut() {
            tracer.leave_fed_body();
        }
        self.fed_bodies -= 1;
        let updated = self.stack[caller + slot].clone();
        self.base = callee;
        self.frame_locals.extend(locals);
        let left = match flow? {
            Flow::Next | Flow::Continue(None) => return Ok(updated),
            Flow::Continue(Some(target)) if target == id => return Ok(updated),
            Flow::Break(None) => Flow::Next,
            Flow::Break(Some(target)) if target == id => Flow::Next,
            other => other,
        };
        self.fed_loops.last_mut().expect("still the innermost").left = Some(left);
        Err(Box::new(Trap {
            message: String::new(),
            code: cpl_diag::codes::RUNTIME,
            span,
            call_stack: Vec::new(),
            case: None,
            notes: Vec::new(),
        }))
    }

    pub(super) fn call(&mut self, id: FnId, args: &'p [Arg], span: Span) -> Res<Value> {
        self.call_feeding(id, args, span, None)
    }

    /// Calls a fn; with `fed`, each value it yields runs that loop's body.
    pub(super) fn call_feeding(
        &mut self,
        id: FnId,
        args: &'p [Arg],
        span: Span,
        fed: Option<Fed<'p>>,
    ) -> Res<Value> {
        let f = &self.program.fns[id];
        if !self.program.judge_fns.is_empty() && self.program.judge_fns.contains(&id) {
            let cpp = self.program.cpp_bodies.iter().any(|(c, _)| *c == id);
            let said = if cpp {
                &cpl_diag::msgs::interp::CPP_ONLY
            } else {
                &cpl_diag::msgs::interp::NO_GRADER
            };
            return trap(span, said.at(&[]));
        }
        self.stack_left(span, Some(&f.name))?;
        let base = match args {
            [] => {
                let base = self.stack.len();
                self.stack.resize(base + f.locals.len(), Value::Unit);
                base
            }
            [Arg::Value(e0)] => {
                let v0 = self.eval(e0)?;
                let base = self.stack.len();
                self.stack.push(v0);
                if f.locals.len() > 1 {
                    self.stack.resize(base + f.locals.len(), Value::Unit);
                }
                base
            }
            [Arg::Value(e0), Arg::Value(e1)] => {
                let v0 = self.eval(e0)?;
                let v1 = self.eval(e1)?;
                let base = self.stack.len();
                self.stack.push(v0);
                self.stack.push(v1);
                if f.locals.len() > 2 {
                    self.stack.resize(base + f.locals.len(), Value::Unit);
                }
                base
            }
            [Arg::Value(e0), Arg::Value(e1), Arg::Value(e2)] => {
                let v0 = self.eval(e0)?;
                let v1 = self.eval(e1)?;
                let v2 = self.eval(e2)?;
                let base = self.stack.len();
                self.stack.push(v0);
                self.stack.push(v1);
                self.stack.push(v2);
                if f.locals.len() > 3 {
                    self.stack.resize(base + f.locals.len(), Value::Unit);
                }
                base
            }
            _ => {
                let mut values = vec![Value::Unit; args.len()];
                // Evaluate by-value arguments first (two-phase borrowing), so patterns
                // like `a.union(a.shifted(w))` can read `a` while preparing the call.
                for (i, arg) in args.iter().enumerate() {
                    if let Arg::Value(e) = arg {
                        values[i] = self.eval(e)?;
                    }
                }
                // A `mut` argument names the caller's variable or the part of it:
                // the callee, and a fn value it runs (`() => ids.len`), read and
                // write the one value, as in the C++, where it is a reference.
                for (i, arg) in args.iter().enumerate() {
                    if let Arg::Mut(place) = arg {
                        values[i] = self.lend(place)?;
                    }
                }
                let base = self.stack.len();
                self.stack.extend(values);
                self.stack.resize(base + f.locals.len(), Value::Unit);
                base
            }
        };
        self.call_at(id, base, span, fed)
    }

    /// Runs fn `id` in the frame at `base`, which holds its arguments.
    pub(crate) fn call_at(
        &mut self,
        id: FnId,
        base: usize,
        span: Span,
        fed: Option<Fed<'p>>,
    ) -> Res<Value> {
        let f = &self.program.fns[id];
        let saved = std::mem::replace(&mut self.base, base);
        self.frame_locals.push(&f.locals);
        self.call_frames.push((f.name.as_str(), span));
        let entered = self.tracer.as_mut().map(|tracer| tracer.enter_fn(&f.name));
        if self.observed() {
            let params: Vec<&str> = f.locals[..f.params.len()]
                .iter()
                .map(|l| l.name.as_str())
                .collect();
            self.traced(|tracer, view| {
                tracer.enter_call(&f.name, || trace::args_json(view, &params))
            });
        }
        let feeding = fed.is_some();
        if let Some(fed) = fed {
            self.fed_loops.push(Fed {
                callee: base,
                ..fed
            });
        }
        let in_closure = std::mem::replace(&mut self.in_closure, false);
        let dom_mark = self.saved_domains.len();
        for &d in &self.fn_guarded[id] {
            self.saved_domains.push((d, self.domains[d]));
        }
        // A traced chunk leaves a `yield` to the tree-walker, which feeds the loop.
        let fed_ok = !feeding || self.vm.as_ref().is_some_and(|c| c.main.traced);
        let mut flow = if fed_ok {
            if let Some(compiled) = self.vm.clone() {
                if compiled.fns.get(id).is_some_and(Option::is_some) {
                    self.run_fn_vm_waited(&compiled, id, span, saved, dom_mark)
                } else {
                    self.fell_back(|| format!("fn:{}(uncompiled)", f.name), false);
                    self.block(&f.body)
                }
            } else {
                self.block(&f.body)
            }
        } else {
            let why = if feeding { "feeding" } else { "part" };
            self.fell_back(|| format!("fn:{}({why})", f.name), false);
            self.block(&f.body)
        };
        while self.saved_domains.len() > dom_mark {
            let (id, prev) = self.saved_domains.pop().unwrap();
            self.domains[id] = prev;
        }
        self.in_closure = in_closure;
        if let (Some(entered), Some(tracer)) = (entered, &mut self.tracer) {
            tracer.leave_fn(entered);
        }
        if self.observed() {
            self.traced(|tracer, view| {
                let returned = match &flow {
                    Ok(Flow::Return(value)) if tracer.keeps_call() && f.ret != Ty::Unit => {
                        Some(trace::json_of(value, &f.ret, view))
                    }
                    _ => None,
                };
                tracer.leave_call(returned);
            });
            flow = flow.and_then(|flow| self.unstopped(span).map(|()| flow));
        }
        if flow.is_err() {
            self.keep_trap_frame();
        }
        self.frame_locals.pop();
        self.base = saved;
        if feeding {
            // The body left the loop: the fn stopped at that `yield`.
            if let Some(left) = self.fed_loops.pop().and_then(|fed| fed.left) {
                self.fed_left = Some(left).filter(|left| !matches!(left, Flow::Next));
                flow = Ok(Flow::Next);
            }
        }
        let flow = flow?;
        self.call_frames.pop();
        self.stack.truncate(base);
        self.after_call(id, span);
        Ok(match flow {
            Flow::Return(value) => value,
            _ => Value::Unit,
        })
    }
}
