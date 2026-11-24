//! Calls: feeding values into loops, and calling functions with their arguments.

use super::*;

impl<'p, 'o, O: Observer> Interp<'p, 'o, O> {
    /// The fn and its list when the loop `id` runs its body as the fn
    /// yields, as the C++ does.
    pub(super) fn is_fed(&mut self, id: LoopId, seq: &Expr, body: &Block) -> Option<(FnId, usize)> {
        if self.in_closure {
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
        let Some(at) = self.fed_by_here() else {
            return Err(interp_failed(span));
        };
        let fed = &self.fed_loops[at];
        let (callee, caller, slot, id, body) = (fed.callee, fed.caller, fed.slot, fed.id, fed.body);
        let locals = self.frame_locals.pop();
        let pushed = self.leave_library_frames(fed.frames);
        self.base = caller;
        self.stack[caller + slot] = value;
        self.fed_bodies += 1;
        self.feeding();
        if let Some(tracer) = self.tracer.as_mut() {
            tracer.on(crate::observe::Event::EnterFedBody);
        }
        let flow = match self.fed_body_vm(id, caller) {
            Some(flow) => flow,
            None => self.block(body),
        };
        if let Some(tracer) = self.tracer.as_mut() {
            tracer.on(crate::observe::Event::LeaveFedBody);
        }
        self.fed_bodies -= 1;
        self.call_frames.extend(pushed);
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
        match self.fed_loops.get_mut(at) {
            Some(fed) => fed.left = Some(left),
            None => return Err(interp_failed(span)),
        }
        Err(Box::new(Trap::said(
            span,
            cpl_diag::msgs::interp::WRITTEN.at(&[&""]),
        )))
    }

    /// The library calls on top of the callee's frame while a fed body
    /// runs: the body is its caller's code, so a step in it is not shown
    /// inside the `r.splits` the callee's own loop is fed by. Given back by
    /// extending `call_frames` once the body is done.
    fn leave_library_frames(&mut self, callee: usize) -> Vec<(&'p str, Span)> {
        if !self.observed() {
            return Vec::new();
        }
        let program = self.program;
        let library = |name: &str| {
            let id = program.fns.iter().position(|f| f.name == name);
            id.is_some_and(|id| program.library_fns.get(id).copied().unwrap_or(false))
        };
        let mut cut = self.call_frames.len();
        while cut > callee && library(self.call_frames[cut - 1].0) {
            cut -= 1;
        }
        self.call_frames.split_off(cut)
    }

    /// The loop the running fn feeds, if it feeds one. A fn feeding a
    /// loop may run a loop of its own fed by another fn, so its loop need
    /// not be the innermost.
    pub(crate) fn fed_by_here(&self) -> Option<usize> {
        self.fed_loops
            .iter()
            .rposition(|fed| fed.callee == self.base)
    }

    pub(super) fn call(&mut self, id: FnId, args: &'p [Arg], span: Span) -> Res<Value> {
        self.call_feeding(id, args, span, None)
    }

    /// Calls a fn; with `fed`, each value it yields runs that loop's body.
    /// The trap for a call to a judge's or `@cpp` fn: only the judge or
    /// the C++ build has its body.
    pub(crate) fn no_body<T>(&self, id: FnId, span: Span) -> Res<T> {
        let cpp = self.program.cpp_bodies.iter().any(|(c, _)| *c == id);
        let said = if cpp {
            &cpl_diag::msgs::interp::CPP_ONLY
        } else {
            &cpl_diag::msgs::interp::NO_GRADER
        };
        trap(span, said.at(&[]))
    }

    pub(super) fn call_feeding(
        &mut self,
        id: FnId,
        args: &'p [Arg],
        span: Span,
        fed: Option<Fed<'p>>,
    ) -> Res<Value> {
        let f = &self.program.fns[id];
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
        if !self.program.judge_fns.is_empty() && self.program.judge_fns.contains(&id) {
            return self.no_body(id, span);
        }
        let saved = std::mem::replace(&mut self.base, base);
        self.frame_locals.push(&f.locals);
        self.call_frames.push((f.name.as_str(), span));
        if let Some(tracer) = self.tracer.as_mut() {
            tracer.on(crate::observe::Event::EnterFn(&f.name));
        }
        if self.observed() {
            let params: Vec<&str> = f.locals[..f.params.len()]
                .iter()
                .map(|l| l.name.as_str())
                .collect();
            self.traced(|tracer, view| {
                tracer.on(crate::observe::Event::EnterCall {
                    name: &f.name,
                    args: &|| trace::args_json(view, &params),
                });
            });
        }
        let feeding = fed.is_some();
        if let Some(fed) = fed {
            self.fed_loops.push(Fed {
                callee: base,
                frames: self.call_frames.len(),
                ..fed
            });
        }
        let in_closure = std::mem::replace(&mut self.in_closure, false);
        let dom_mark =
            crate::save_domains(&mut self.saved_domains, &self.domains, &self.fn_guarded[id]);
        // A traced chunk leaves a `yield` to the tree-walker, which feeds the loop.
        let compiled = Rc::clone(&self.vm);
        let flow = if !feeding || compiled.main.traced {
            // Only a judge fn has no chunk, and it stopped above.
            self.run_fn_vm_waited(&compiled, id, span, saved, dom_mark)
        } else {
            let why = if feeding { "feeding" } else { "part" };
            self.fell_back(|| format!("fn:{}({why})", f.name), false);
            self.block(&f.body)
        };
        self.restore_domains(dom_mark);
        self.in_closure = in_closure;
        let flow = self.leave_frame(Some(f), span, saved, feeding, flow);
        let flow = flow?;
        self.call_frames.pop();
        self.stack.truncate(base);
        self.after_call(id, span);
        Ok(match flow {
            Flow::Return(value) => value,
            _ => Value::Unit,
        })
    }

    /// The end of a call to `f`, or to a closure when `f` is `None`, that
    /// `flow` ended: what the tracer sees of it, the frame a trap keeps,
    /// the caller's frame back at `saved`, and where a fed loop's body left.
    pub(crate) fn leave_frame(
        &mut self,
        f: Option<&'p cpl_ir::Function>,
        span: Span,
        saved: usize,
        feeding: bool,
        mut flow: Res<Flow>,
    ) -> Res<Flow> {
        if let Some(f) = f {
            if let Some(tracer) = &mut self.tracer {
                tracer.on(crate::observe::Event::LeaveFn);
            }
            if self.observed() {
                self.traced(|tracer, view| {
                    let returned = match &flow {
                        Ok(Flow::Return(value)) if tracer.keeps_call() && f.ret != Ty::Unit => {
                            Some(trace::json_of(value, &f.ret, view))
                        }
                        _ => None,
                    };
                    tracer.on(crate::observe::Event::LeaveCall(returned));
                });
                flow = flow.and_then(|flow| self.unstopped(span).map(|()| flow));
            }
            if flow.is_err() {
                self.keep_trap_frame();
            }
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
        flow
    }
}
