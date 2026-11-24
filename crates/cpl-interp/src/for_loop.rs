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
            Iter::Range {
                lo,
                hi,
                step,
                rev,
                closed,
            } => {
                let (lo_i64, hi_i64) = (self.eval(lo)?.int(), self.eval(hi)?.int());
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
            Iter::TreeRange {
                container,
                lo,
                hi,
                rev,
            } => {
                let held = self.eval(container)?;
                let mut bound = |b: &'p Option<(Expr, bool)>| -> Res<_> {
                    Ok(match b {
                        Some((e, closed)) => Some((self.eval(e)?, *closed)),
                        None => None,
                    })
                };
                let (lo, hi) = (bound(lo)?, bound(hi)?);
                let items = self.tree_range_items(&held, lo, hi, span)?;
                if *rev {
                    for item in items.into_iter().rev() {
                        round!(item);
                    }
                } else {
                    for item in items {
                        round!(item);
                    }
                }
            }
            Iter::Seq(seq, false) if self.is_fed(id, seq, body).is_some() => {
                let (ExprKind::Call(callee, args), Some((_, acc))) =
                    (&seq.kind, self.is_fed(id, seq, body))
                else {
                    return Err(interp_failed(seq.span));
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
                self.call_feeding(*callee, args, seq.span, Some(fed))?;
                if let Some(flow) = self.fed_left.take() {
                    return Ok(flow);
                }
            }
            Iter::RangeVal(seq, rev) | Iter::Seq(seq, rev) => {
                // The loop walks the value as it was when the loop began.
                let held = self.eval(seq)?;
                let items = self.seq_items(&held, span)?;
                for k in rounds(items.len(), *rev) {
                    round!(items[k].clone());
                }
            }
        }
        Ok(Flow::Next)
    }

    /// The items a `for` loop over sequence `held` walks, in order: a
    /// string's are its chars, a map's its `(key, value)` pairs.
    pub(crate) fn seq_items(&self, held: &Value, span: Span) -> Res<Vec<Value>> {
        if let Some(cells) = held.cells() {
            return Ok(cells.to_values());
        }
        Ok(match held {
            Value::ListView(v) => (0..v.len)
                .map(|k| v.base.get_unchecked_val(v.pos(k)))
                .collect(),
            Value::MutView(mv) => self.mut_view_to_vec(mv, span)?,
            Value::Str(bytes) => bytes.iter().map(|&c| Value::Char(c)).collect(),
            Value::StrView(v) => (0..v.len).map(|k| Value::Char(v.base[v.pos(k)])).collect(),
            Value::Set(items) => items.iter().cloned().collect(),
            Value::Map(entries) => (entries.iter())
                .map(|(k, v)| Value::tuple(vec![k.clone(), v.clone()]))
                .collect(),
            Value::Range(lo, hi) => (*lo..*hi).map(Value::Int).collect(),
            Value::Range128(r) => (r.0..r.1).map(|v| Value::I128(v.into())).collect(),
            _ => return Err(interp_failed(span)),
        })
    }

    /// The items of sorted set or map `held` between bounds, each a value
    /// and whether it is closed, in order: a map's are `(key, value)` pairs.
    pub(crate) fn tree_range_items(
        &mut self,
        held: &Value,
        lo: Option<(Value, bool)>,
        hi: Option<(Value, bool)>,
        span: Span,
    ) -> Res<Vec<Value>> {
        use std::ops::Bound;
        let [lo, hi] = [lo, hi].map(|b| match b {
            Some((v, true)) => Bound::Included(v),
            Some((v, false)) => Bound::Excluded(v),
            None => Bound::Unbounded,
        });
        let empty = match (&lo, &hi) {
            (Bound::Included(a), Bound::Included(b)) => a > b,
            (Bound::Included(a) | Bound::Excluded(a), Bound::Included(b) | Bound::Excluded(b)) => {
                a >= b
            }
            _ => false,
        };
        // Only a B-tree's bounds can be ordered structurally; a sorted set finds its own.
        let empty = empty
            && !matches!(held, Value::Set(s) if s.is_sorted_by_user())
            && !matches!(held, Value::Map(m) if m.is_sorted_by_user());
        let mut cmp = |t: &Ty, a: &Value, b: &Value| self.less_ty_val(t, a, b, span);
        Ok(match held {
            _ if empty => vec![],
            Value::Set(items) => items.range(lo, hi, &mut cmp)?.cloned().collect(),
            Value::Map(entries) => (entries.range(lo, hi, &mut cmp)?)
                .map(|(k, v)| Value::tuple(vec![k.clone(), v.clone()]))
                .collect(),
            _ => return Err(interp_failed(span)),
        })
    }
}

/// The indices a `for` over `n` items walks, backward when `rev`.
fn rounds(n: usize, rev: bool) -> impl Iterator<Item = usize> {
    (0..n).map(move |i| if rev { n - 1 - i } else { i })
}
