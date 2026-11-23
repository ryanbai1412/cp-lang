//! Calls to function values on the tree-walker: a closure's frame, its
//! captures and the `mut` arguments it borrows.

use crate::observe::Observer;
use crate::value::{CaptureVal, Value};
use crate::*;

impl<'p, 'o, O: Observer> Interp<'p, 'o, O> {
    /// The call `e`, of the function value `callee` on `args`.
    pub(crate) fn call_expr(
        &mut self,
        e: &'p Expr,
        callee: &'p Expr,
        args: &'p [Arg],
        span: Span,
    ) -> Res<Value> {
        if self.ckpt.waits() {
            if let Some(value) = self.resume_pending(e)? {
                return Ok(value);
            }
        }
        let fv = self.eval(callee)?;
        if !self.observed()
            && self.profiler.is_none()
            && args.len() <= 4
            && args.iter().all(|a| matches!(a, Arg::Value(_)))
        {
            if let Value::Closure(cid, _) = &fv {
                if let Some(fast) = self.fast_closures[*cid].clone() {
                    let mut evaled = [Value::Unit, Value::Unit, Value::Unit, Value::Unit];
                    for (slot, arg) in evaled.iter_mut().zip(args) {
                        let Arg::Value(e) = arg else { unreachable!() };
                        *slot = self.eval(e)?;
                    }
                    // A parameter may appear more than once in the body (`x => x + x`).
                    let arg = |i: usize| evaled[i].clone();
                    return match fast {
                        FastClosure::MinMax { is_max, a, b, ty } => {
                            let (va, vb) = (arg(a), arg(b));
                            let ord = self.cmp_ty_val(&ty, &vb, &va, span)?;
                            let take_b = if is_max { ord.is_gt() } else { ord.is_lt() };
                            Ok(if take_b { vb } else { va })
                        }
                        FastClosure::Bin { op, a, b } => binary(op, arg(a), arg(b), span),
                        FastClosure::IntAddMul { x, f, w } => {
                            let prod = binary(BinOp::Mul, arg(f), arg(w), span)?;
                            binary(BinOp::Add, arg(x), prod, span)
                        }
                        FastClosure::IntCmpConst { op, a, c } => {
                            let va = arg(a).int();
                            Ok(Value::Bool(match op {
                                CmpOp::Eq => va == c,
                                CmpOp::Ne => va != c,
                                CmpOp::Lt => va < c,
                                CmpOp::Le => va <= c,
                                CmpOp::Gt => va > c,
                                CmpOp::Ge => va >= c,
                                _ => false,
                            }))
                        }
                        FastClosure::Identity(a) => Ok(arg(a)),
                    };
                }
            }
        }
        let mut vals = vec![Value::Unit; args.len()];
        for (i, arg) in args.iter().enumerate() {
            if let Arg::Value(e) = arg {
                vals[i] = self.eval(e)?;
            }
        }
        // `mut` arguments are lent for the call, as to a
        // top-level fn.
        let mut lent: Vec<(usize, &'p Place, Vec<Value>)> = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            if let Arg::Mut(place) = arg {
                let mut indices = Vec::new();
                self.indices(place, &mut indices)?;
                let cell = self.resolve(place, &mut indices.iter())?;
                let value = std::mem::replace(cell, Value::Moved);
                if matches!(value, Value::Moved) {
                    return trap(place.span, LENT);
                }
                vals[i] = value;
                lent.push((i, place, indices));
            }
        }
        let at: Vec<usize> = lent.iter().map(|(i, ..)| *i).collect();
        let (value, back) = self.call_lending(fv, vals, &at, span)?;
        for ((_, place, indices), value) in lent.into_iter().zip(back) {
            *self.resolve(place, &mut indices.iter())? = value;
        }
        self.keep_called(e, &value);
        Ok(value)
    }

    pub(crate) fn call_value(&mut self, fv: Value, args: Vec<Value>, span: Span) -> Res<Value> {
        Ok(self.call_lending(fv, args, &[], span)?.0)
    }

    /// Calls a fn value. The arguments at `lent` are `mut`: what the fn
    /// left in them comes back, in that order, next to its result.
    pub(crate) fn call_lending(
        &mut self,
        fv: Value,
        args: Vec<Value>,
        lent: &[usize],
        span: Span,
    ) -> Res<(Value, Vec<Value>)> {
        let Value::Closure(cid, caps) = fv else {
            return trap(span, "internal: tried to call a non-function value");
        };
        let def = &self.program.closures[cid];
        self.stack_left(span, None)?;
        let base = self.stack.len();
        self.stack.resize(base + def.locals.len(), Value::Unit);
        for (slot, val) in def.params.iter().zip(args) {
            self.stack[base + slot] = val;
        }
        for (cap, cval) in def.captures.iter().zip(caps.iter()) {
            let v = match cval {
                CaptureVal::Val(v) => v.clone(),
                // Captured by reference: the slot names the variable, so
                // every call, however deep, reads and writes the one value.
                CaptureVal::StackRef(at) => Value::Alias(false, *at),
                CaptureVal::GlobalRef(id) => Value::Alias(true, *id),
                CaptureVal::PartRef(part) => Value::Part(Rc::clone(part)),
            };
            self.stack[base + cap.inner] = v;
        }
        let saved = std::mem::replace(&mut self.base, base);
        self.frame_locals.push(&def.locals);
        self.call_frames.push(("<closure>", span));
        let in_closure = std::mem::replace(&mut self.in_closure, true);
        let dom_mark = self.saved_domains.len();
        for &d in &self.closure_guarded[cid] {
            self.saved_domains.push((d, self.domains[d]));
        }
        let flow = match self.vm.clone() {
            Some(compiled) if lent.is_empty() && compiled.closures[cid].is_some() => self
                .run_closure_vm_from_stack(&compiled, cid)
                .map(Flow::Return),
            _ => self.block(&def.body),
        };
        while self.saved_domains.len() > dom_mark {
            let (id, prev) = self.saved_domains.pop().unwrap();
            self.domains[id] = prev;
        }
        self.in_closure = in_closure;
        self.frame_locals.pop();
        self.base = saved;
        let flow = flow?;
        self.call_frames.pop();
        if let Some(tracer) = self.tracer.as_mut() {
            tracer.globals_may_change();
        }
        let back = lent
            .iter()
            .map(|i| std::mem::take(&mut self.stack[base + def.params[*i]]))
            .collect();
        self.stack.truncate(base);
        let value = match flow {
            Flow::Return(v) | Flow::Give(v) => v,
            _ => Value::Unit,
        };
        Ok((value, back))
    }

    /// A `mut` argument: the caller's variable `place`, or the part of it,
    /// which the callee and a fn value it runs read and write as the one
    /// value, as in the C++, where it is a reference.
    pub(crate) fn lend(&mut self, place: &'p Place) -> Res<Value> {
        let mut indices = Vec::new();
        self.indices(place, &mut indices)?;
        self.lend_at(place, indices)
    }

    /// [`Self::lend`], at `indices` already worked out.
    pub(crate) fn lend_at(&mut self, place: &'p Place, indices: Vec<Value>) -> Res<Value> {
        let part = self.part_of(place, &mut indices.into_iter());
        if matches!(self.part_mut(&part, place.span)?, Value::Moved) {
            return trap(place.span, LENT);
        }
        Ok(match part.steps.is_empty() {
            true => Value::Alias(part.global, part.at),
            false => Value::Part(Rc::new(part)),
        })
    }

    /// Whether a fn value among fn `id`'s arguments, now on the stack, may
    /// read or write a variable lent to the call. A closure that captures
    /// nothing by reference, touches no lent global, and calls nothing
    /// cannot.
    pub(crate) fn fn_args_may_see_lent(&self, id: cpl_ir::FnId) -> bool {
        let f = &self.program.fns[id];
        let params = self.base..self.base + f.params.len();
        let lent: Vec<usize> = (self.stack[params].iter())
            .filter_map(|v| {
                let (mut global, mut at) = match v {
                    Value::Alias(global, at) => (*global, *at),
                    Value::Part(part) => (part.global, part.at),
                    _ => return None,
                };
                while let Value::Alias(g, a) = match global {
                    true => &self.globals[at],
                    false => &self.stack[at],
                } {
                    (global, at) = (*g, *a);
                }
                global.then_some(at)
            })
            .collect();
        let touches = |place: &Place| match cpl_ir::walk::root(place) {
            (true, g) => lent.contains(&g),
            _ => false,
        };
        (f.locals.iter().take(f.params.len()).enumerate())
            .filter(|(_, l)| matches!(l.ty, Ty::Fn(..)))
            .any(|(i, _)| match &self.stack[self.base + i] {
                Value::Closure(cid, _) => {
                    let def = &self.program.closures[*cid];
                    let mut sees = (def.captures.iter())
                        .any(|c| c.by_ref || (c.is_global && lent.contains(&c.outer)));
                    cpl_ir::walk::deep_exprs(&def.body, &mut |e| {
                        sees |= match &e.kind {
                            ExprKind::Call(..) | ExprKind::CallExpr(..) | ExprKind::Closure(..) => {
                                true
                            }
                            ExprKind::Global(g) => lent.contains(g),
                            ExprKind::Mutate(_, place, _) => touches(place),
                            _ => false,
                        }
                    });
                    cpl_ir::walk::deep_stmts(&def.body, &mut |stmt| {
                        sees |= matches!(stmt, Stmt::Assign(place, ..) if touches(place))
                    });
                    sees
                }
                _ => true,
            })
    }
}
