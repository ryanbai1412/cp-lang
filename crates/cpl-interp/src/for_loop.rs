//! Running a `for` loop over each kind of iterator.

use super::*;

impl<'p, 'o, O: Observer> Interp<'p, 'o, O> {
    pub(super) fn for_loop(
        &mut self,
        id: LoopId,
        slot: usize,
        iter: &'p Iter,
        body: &'p Block,
    ) -> Res<Flow> {
        let base = self.base;
        let span = match iter {
            Iter::Walk { .. } => unreachable!("Iter::Walk is expanded before codegen"),
            Iter::Range { lo: e, .. }
            | Iter::RangeVal(e, _)
            | Iter::Seq(e, _)
            | Iter::TreeRange { container: e, .. } => e.span,
        };
        macro_rules! round {
            ($value:expr) => {{
                self.stack[base + slot] = $value;
                self.trace_local(slot, span);
                self.trace_assigned_local(slot, span);
                match self.block(body)? {
                    Flow::Next | Flow::Continue(None) => {}
                    Flow::Break(None) => break,
                    Flow::Break(Some(target)) if target == id => break,
                    Flow::Continue(Some(target)) if target == id => {}
                    other => return Ok(other),
                }
            }};
        }
        let is_enum_slot =
            matches!(self.locals().get(slot), Some(l) if matches!(l.ty, Ty::Enum(_)));
        let wrap = |v: i64| {
            if is_enum_slot {
                Value::Enum(v as u32, Rc::new(Vec::new()))
            } else {
                Value::Int(v)
            }
        };
        match iter {
            Iter::Walk { .. } => unreachable!("Iter::Walk is expanded before codegen"),
            Iter::Range {
                lo,
                hi,
                step,
                rev,
                closed,
            } => {
                let (lo_i64, hi_i64) = match self.ckpt.range_from.take() {
                    Some(from) => from,
                    None => (self.eval(lo)?.int(), self.eval(hi)?.int()),
                };
                if step.is_none() {
                    if *closed {
                        if lo_i64 <= hi_i64 {
                            if *rev {
                                let mut v = hi_i64;
                                loop {
                                    round!(wrap(v));
                                    if v == lo_i64 {
                                        break;
                                    }
                                    v -= 1;
                                }
                            } else {
                                let mut v = lo_i64;
                                loop {
                                    round!(wrap(v));
                                    if v == hi_i64 {
                                        break;
                                    }
                                    v += 1;
                                }
                            }
                        }
                    } else if lo_i64 < hi_i64 {
                        if *rev {
                            let mut v = hi_i64 - 1;
                            loop {
                                round!(wrap(v));
                                if v == lo_i64 {
                                    break;
                                }
                                v -= 1;
                            }
                        } else {
                            let mut v = lo_i64;
                            while v < hi_i64 {
                                round!(wrap(v));
                                v += 1;
                            }
                        }
                    }
                    return Ok(Flow::Next);
                }
                let lo = lo_i64 as i128;
                // One past the last value, which an `Int` may not hold.
                let hi = hi_i64 as i128 + i128::from(*closed);
                let step = match step {
                    Some(step) => {
                        let value = self.eval(step)?.int();
                        if value <= 0 {
                            return trap(
                                step.span,
                                cpl_diag::msgs::interp::RANGE_STEPS_FORWARD_BY_NOT.at(&[&value]),
                            );
                        }
                        value
                    }
                    None => 1,
                };
                if lo >= hi {
                    return Ok(Flow::Next);
                }
                // The values are lo, lo + step, ..., below hi.
                let step = step as i128;
                let count = (hi - lo + step - 1) / step;
                if *rev {
                    let mut k = count - 1;
                    loop {
                        if k < 0 {
                            break;
                        }
                        round!(Value::Int((lo + k * step) as i64));
                        k -= 1;
                    }
                } else {
                    let mut k = 0;
                    loop {
                        if k >= count {
                            break;
                        }
                        round!(Value::Int((lo + k * step) as i64));
                        k += 1;
                    }
                }
            }
            Iter::RangeVal(expr, rev) => match self.eval(expr)? {
                Value::Range(lo, hi) if lo < hi => {
                    if *rev {
                        let mut v = hi - 1;
                        while v >= lo {
                            round!(Value::Int(v));
                            if v == lo {
                                break;
                            }
                            v -= 1;
                        }
                    } else {
                        let mut v = lo;
                        while v < hi {
                            round!(Value::Int(v));
                            v += 1;
                        }
                    }
                }
                Value::Range128(r) if r.0 < r.1 => {
                    let (lo, hi) = *r;
                    if *rev {
                        let mut v = hi - 1;
                        while v >= lo {
                            round!(Value::I128(v.into()));
                            if v == lo {
                                break;
                            }
                            v -= 1;
                        }
                    } else {
                        let mut v = lo;
                        while v < hi {
                            round!(Value::I128(v.into()));
                            v += 1;
                        }
                    }
                }
                _ => {}
            },
            Iter::TreeRange {
                container,
                lo,
                hi,
                rev,
            } => {
                use std::ops::Bound;
                let held = self.eval(container)?;
                let lo_b = match lo {
                    Some((e, true)) => Bound::Included(self.eval(e)?),
                    Some((e, false)) => Bound::Excluded(self.eval(e)?),
                    None => Bound::Unbounded,
                };
                let hi_b = match hi {
                    Some((e, true)) => Bound::Included(self.eval(e)?),
                    Some((e, false)) => Bound::Excluded(self.eval(e)?),
                    None => Bound::Unbounded,
                };
                let empty = match (&lo_b, &hi_b) {
                    (Bound::Included(a), Bound::Included(b)) => a > b,
                    (Bound::Included(a), Bound::Excluded(b))
                    | (Bound::Excluded(a), Bound::Included(b))
                    | (Bound::Excluded(a), Bound::Excluded(b)) => a >= b,
                    _ => false,
                };
                if !empty {
                    match &held {
                        Value::Set(items) => {
                            let range = items.range((lo_b, hi_b));
                            if *rev {
                                for item in range.rev() {
                                    round!(item.clone());
                                }
                            } else {
                                for item in range {
                                    round!(item.clone());
                                }
                            }
                        }
                        Value::Map(entries) => {
                            let range = entries.range((lo_b, hi_b));
                            if *rev {
                                for (k, v) in range.rev() {
                                    round!(Value::tuple(vec![k.clone(), v.clone()]));
                                }
                            } else {
                                for (k, v) in range {
                                    round!(Value::tuple(vec![k.clone(), v.clone()]));
                                }
                            }
                        }
                        _ => unreachable!("TreeRange only lowers on Set/Map"),
                    }
                }
            }
            Iter::Seq(seq, false) if self.is_fed(id, seq, body).is_some() => {
                let (ExprKind::Call(callee, args), Some((_, acc))) =
                    (&seq.kind, self.is_fed(id, seq, body))
                else {
                    unreachable!("a fed loop is over a call");
                };
                let fed = Fed {
                    callee: 0,
                    acc,
                    caller: base,
                    id,
                    slot,
                    body,
                    left: None,
                };
                if !(self.ckpt.waits() && self.resume_fed(seq, &fed)?) {
                    self.call_feeding(*callee, args, seq.span, Some(fed))?;
                }
                if let Some(flow) = self.fed_left.take() {
                    return Ok(flow);
                }
            }
            Iter::Seq(seq, rev) => {
                // The loop walks the value as it was when the loop began.
                let (held, from) = self.seq_held(seq)?;
                match &held {
                    Value::List(items) => {
                        let n = items.len();
                        for k in checkpoint::rounds(n, *rev, from) {
                            round!(items.get_unchecked_val(k));
                        }
                    }
                    Value::ListView(view) => {
                        let n = view.len;
                        for k in checkpoint::rounds(n, *rev, from) {
                            round!(view.base.get_unchecked_val(view.pos(k)));
                        }
                    }
                    Value::MutView(mv) => {
                        let items = self.mut_view_to_vec(mv, span)?;
                        let n = items.len();
                        for k in checkpoint::rounds(n, *rev, from) {
                            round!(items[k].clone());
                        }
                    }
                    Value::Array(array) => {
                        let n = array.len();
                        for k in checkpoint::rounds(n, *rev, from) {
                            round!(array.get_unchecked_val(k));
                        }
                    }
                    Value::Str(bytes) => {
                        let n = bytes.len();
                        for k in checkpoint::rounds(n, *rev, from) {
                            round!(Value::Char(bytes[k]));
                        }
                    }
                    Value::StrView(view) => {
                        let n = view.len;
                        for k in checkpoint::rounds(n, *rev, from) {
                            round!(Value::Char(view.base[view.pos(k)]));
                        }
                    }
                    Value::Set(items) => {
                        if *rev {
                            for item in items.iter().rev() {
                                round!(item.clone());
                            }
                        } else {
                            for item in items.iter() {
                                round!(item.clone());
                            }
                        }
                    }
                    Value::Map(entries) => {
                        if *rev {
                            for (key, value) in entries.iter().rev() {
                                round!(Value::tuple(vec![key.clone(), value.clone()]));
                            }
                        } else {
                            for (key, value) in entries.iter() {
                                round!(Value::tuple(vec![key.clone(), value.clone()]));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(Flow::Next)
    }
}
