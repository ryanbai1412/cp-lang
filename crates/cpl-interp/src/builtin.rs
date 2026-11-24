//! The builtins on the values of their arguments, which both engines share.

use crate::observe::Observer;
use crate::value::*;
use crate::*;

impl<'p, 'o, O: Observer> Interp<'p, 'o, O> {
    /// The builtin `op` of `e`, on `values`, the values of `args`.
    pub(crate) fn builtin_values(
        &mut self,
        op: Builtin,
        args: &'p [Expr],
        e: &'p Expr,
        values: Vec<Value>,
    ) -> Res<Value> {
        let span = e.span;
        let mut values = values.into_iter();
        let first = values.next().unwrap_or_default();
        let optional = |value: Option<Value>| Value::Opt(OptVal::from_option(value));
        Ok(match op {
            Builtin::WrapAdd | Builtin::WrapSub | Builtin::WrapMul => {
                let (a, b) = (first.int(), values.next().unwrap_or_default().int());
                Value::Int(match op {
                    Builtin::WrapAdd => a.wrapping_add(b),
                    Builtin::WrapSub => a.wrapping_sub(b),
                    _ => a.wrapping_mul(b),
                })
            }
            Builtin::Min => {
                let mut best = first;
                for v in values {
                    if self.cmp_ty_val(&e.ty, &v, &best, span)?.is_lt() {
                        best = v;
                    }
                }
                best
            }
            Builtin::Max => {
                let mut best = first;
                for v in values {
                    if self.cmp_ty_val(&e.ty, &v, &best, span)?.is_gt() {
                        best = v;
                    }
                }
                best
            }
            Builtin::Abs => match first {
                Value::Int(v) => match v.checked_abs() {
                    Some(v) => Value::Int(v),
                    None => {
                        return trap(
                            span,
                            format!(
                                "integer overflow: `abs({v})` does not fit in an `Int` (64 bits)"
                            ),
                        )
                    }
                },
                Value::I128(v) => match v.get().checked_abs() {
                    Some(v) => Value::I128(v.into()),
                    None => {
                        return trap(
                            span,
                            format!(
                                "integer overflow: `abs({v})` does not fit in an `I128` (128 bits)"
                            ),
                        )
                    }
                },
                Value::Float(v) => Value::Float(v.abs()),
                other => other,
            },
            Builtin::Sqrt
            | Builtin::Atan2
            | Builtin::Sin
            | Builtin::Cos
            | Builtin::Tan
            | Builtin::Asin
            | Builtin::Acos
            | Builtin::Exp
            | Builtin::Ln
            | Builtin::Hypot => float_fn::eval(op, first.float(), values.next(), span)?,
            Builtin::Flush => {
                self.write_out(true);
                Value::Unit
            }
            Builtin::Floor | Builtin::Ceil | Builtin::Round => {
                let v = match op {
                    Builtin::Floor => first.float().floor(),
                    Builtin::Ceil => first.float().ceil(),
                    _ => first.float().round(),
                };
                float_int(v, span)?
            }
            Builtin::CountQuery => {
                self.queries += 1;
                let limit = first.int();
                if self.queries > limit {
                    return trap(
                        span,
                        format!(
                            "query limit exceeded: this is query {}, and the limit is {limit}",
                            self.queries
                        ),
                    );
                }
                Value::Unit
            }
            Builtin::AtEof => Value::Bool(self.input.at_eof()),
            Builtin::SwapInput => {
                match &mut self.peer {
                    Some(peer) => std::mem::swap(&mut self.input, peer),
                    None => return trap(span, "there is no solution to talk to: an `@interactor` runs under `cpl run` or `cpl stress`"),
                }
                Value::Unit
            }
            Builtin::Popcount => Value::Int(i64::from(bits64(&first).count_ones())),
            Builtin::Ctz => Value::Int(i64::from(bits64(&first).trailing_zeros())),
            Builtin::Clz => Value::Int(i64::from(bits64(&first).leading_zeros())),
            Builtin::AxisLen => match (&first, values.next()) {
                (Value::Array(array), Some(Value::Int(axis))) => {
                    Value::Int(array.dims[axis as usize])
                }
                _ => Value::Int(0),
            },
            Builtin::BitWidth => {
                let u = bits64(&first);
                Value::Int(i64::from(64 - u.leading_zeros()))
            }
            Builtin::BitCeil => {
                let x = first.int();
                if x <= 1 {
                    Value::Int(1)
                } else {
                    let shift = 64 - ((x - 1) as u64).leading_zeros();
                    if shift > 62 {
                        return trap(span, format!("integer overflow: `bit_ceil({x})` does not fit in an `Int` (64 bits)"));
                    }
                    Value::Int(1 << shift)
                }
            }
            Builtin::Rand => {
                let lo = first.int();
                let hi = values.next().unwrap_or_default().int();
                if lo > hi {
                    return trap(span, format!("rand({lo}, {hi}): lo > hi"));
                }
                let span_u = (hi as i128 - lo as i128 + 1) as u128;
                let choices = self.choices.as_deref();
                let off = draw_below(choices, &mut self.rng_state, span_u, span.start) as i128;
                Value::Int((lo as i128 + off) as i64)
            }
            Builtin::Copy => first,
            Builtin::ListFilled => {
                let n = first.int();
                let val = values.next().unwrap_or_default();
                if n < 0 {
                    return trap(span, format!("List.filled with negative length {n}"));
                }
                Value::List(Rc::new(vec![val; n as usize]))
            }
            Builtin::Join => {
                let text = |value: Value| match value {
                    Value::Str(bytes) => bytes,
                    Value::StrView(view) => Rc::new(view.to_vec()),
                    _ => Rc::new(Vec::new()),
                };
                let sep = text(first);
                let seq = values.next().unwrap_or_default();
                let none_text = values.next().map(text);
                let elem = match &args[1].ty {
                    Ty::List(elem)
                    | Ty::ListView(elem, _)
                    | Ty::Keyed(elem, _)
                    | Ty::MutView(elem, _)
                    | Ty::Array { elem, .. } => (**elem).clone(),
                    _ => Ty::Unit,
                };
                let owned;
                let items: &[Value] = match &seq {
                    Value::List(items) => items,
                    Value::ListView(view) => {
                        owned = view.to_vec();
                        &owned
                    }
                    Value::MutView(mv) => {
                        owned = self.mut_view_to_vec(mv, span)?;
                        &owned
                    }
                    Value::Array(array) => {
                        owned = array.to_values();
                        &owned
                    }
                    _ => &[],
                };
                let mut buf = Vec::new();
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        buf.extend_from_slice(&sep);
                    }
                    match (item, &none_text) {
                        (Value::Opt(OptVal::None), Some(text)) => buf.extend_from_slice(text),
                        _ => self.show_val(&mut buf, item, &elem, cpl_ir::FLOAT_DIGITS, span)?,
                    }
                }
                Value::Str(Rc::new(buf))
            }
            Builtin::SeqGet => {
                let at = values.next().unwrap_or_default().int();
                let item = match &first {
                    Value::List(items) => {
                        usize::try_from(at).ok().and_then(|i| items.get(i)).cloned()
                    }
                    Value::ListView(view) => {
                        usize::try_from(at).ok().and_then(|i| view.get(i)).cloned()
                    }
                    Value::MutView(mv) => match usize::try_from(at).ok().filter(|i| *i < mv.len) {
                        Some(i) => Some(self.mut_view_get(mv, i, span)?),
                        None => None,
                    },
                    Value::Str(bytes) => usize::try_from(at)
                        .ok()
                        .and_then(|i| bytes.get(i))
                        .map(|b| Value::Char(*b)),
                    Value::StrView(view) => usize::try_from(at)
                        .ok()
                        .and_then(|i| view.get(i))
                        .map(Value::Char),
                    Value::Array(array) if array.dims.len() == 1 => (at >= 0 && at < array.dims[0])
                        .then(|| array.get_unchecked_val(at as usize)),
                    Value::Range(lo, hi) => {
                        let len = hi.saturating_sub(*lo).max(0);
                        (at >= 0 && at < len).then_some(Value::Int(lo + at))
                    }
                    _ => None,
                };
                optional(item)
            }
            Builtin::ShowStr => {
                let mut buf = Vec::new();
                let arg_ty = args.first().map(|a| &a.ty).unwrap_or(&Ty::Unit);
                self.show_val(&mut buf, &first, arg_ty, cpl_ir::FLOAT_DIGITS, span)?;
                Value::Str(Rc::new(buf))
            }
            Builtin::ParseInt => match first {
                Value::Str(bytes) => {
                    let s = std::str::from_utf8(&bytes).unwrap_or("").trim();
                    match s.parse::<i64>() {
                        Ok(v) => Value::Int(v),
                        Err(_) => return trap(span, format!("`{s}` is not an Int")),
                    }
                }
                Value::StrView(view) => {
                    let bytes = view.to_vec();
                    let s = std::str::from_utf8(&bytes).unwrap_or("").trim();
                    match s.parse::<i64>() {
                        Ok(v) => Value::Int(v),
                        Err(_) => return trap(span, format!("`{s}` is not an Int")),
                    }
                }
                other => other,
            },
            Builtin::StrToList => match first {
                Value::Str(bytes) => {
                    Value::List(Rc::new(bytes.iter().copied().map(Value::Char).collect()))
                }
                Value::StrView(view) => Value::List(Rc::new(
                    view.to_vec().into_iter().map(Value::Char).collect(),
                )),
                other => other,
            },
            Builtin::ListToStr => match first {
                Value::List(items) => {
                    let bytes: Vec<u8> = items
                        .iter()
                        .map(|v| match v {
                            Value::Char(c) => *c,
                            _ => 0,
                        })
                        .collect();
                    Value::Str(Rc::new(bytes))
                }
                Value::ListView(view) => {
                    let bytes: Vec<u8> = (0..view.len)
                        .map(|i| match view.get(i) {
                            Some(Value::Char(c)) => *c,
                            _ => 0,
                        })
                        .collect();
                    Value::Str(Rc::new(bytes))
                }
                Value::MutView(mv) => {
                    let bytes: Vec<u8> = self
                        .mut_view_to_vec(&mv, span)?
                        .into_iter()
                        .map(|v| match v {
                            Value::Char(c) => c,
                            _ => 0,
                        })
                        .collect();
                    Value::Str(Rc::new(bytes))
                }
                other => other,
            },
            Builtin::ArrayToList => match first {
                Value::Array(arr) => Value::List(Rc::new(arr.to_values())),
                other => other,
            },
            Builtin::ListToArray => match first {
                Value::List(items) => Value::Array(Rc::new(Array::from_values(
                    vec![items.len() as i64],
                    items.to_vec(),
                ))),
                Value::ListView(view) => {
                    let items = view.to_vec();
                    Value::Array(Rc::new(Array::from_values(vec![items.len() as i64], items)))
                }
                Value::MutView(mv) => {
                    let items = self.mut_view_to_vec(&mv, span)?;
                    Value::Array(Rc::new(Array::from_values(vec![items.len() as i64], items)))
                }
                other => other,
            },
            Builtin::Len => Value::Int(match &first {
                Value::Array(array) => array.len() as i64,
                Value::List(items) => items.len() as i64,
                Value::ListView(view) => view.len as i64,
                Value::MutView(mv) => mv.len as i64,
                Value::Str(bytes) => bytes.len() as i64,
                Value::StrView(view) => view.len as i64,
                Value::Map(entries) => entries.len() as i64,
                Value::Set(items) => items.len() as i64,
                Value::Range(lo, hi) => {
                    if hi <= lo {
                        0
                    } else {
                        match hi.checked_sub(*lo) {
                            Some(d) => d,
                            None => {
                                return trap(
                                    span,
                                    format!("integer overflow: range `{lo}..{hi}` length does not fit in an `Int` (64 bits)"),
                                );
                            }
                        }
                    }
                }
                Value::Range128(r) => {
                    let (lo, hi) = (&r.0, &r.1);
                    if hi <= lo {
                        0
                    } else {
                        match hi.checked_sub(*lo).and_then(|d| i64::try_from(d).ok()) {
                            Some(d) => d,
                            None => {
                                return trap(
                                    span,
                                    format!("integer overflow: range `{lo}..{hi}` length does not fit in an `Int` (64 bits)"),
                                );
                            }
                        }
                    }
                }
                Value::DomainVal(_, len) => *len,
                _ => 0,
            }),
            Builtin::First | Builtin::Last => {
                optional(self.seq_end(&first, matches!(op, Builtin::Last), span)?)
            }
            Builtin::FirstVal | Builtin::LastVal => {
                match self.seq_end(&first, matches!(op, Builtin::LastVal), span)? {
                    Some(v) => v,
                    None => return trap(span, format!("the {} is empty", seq_noun(&first))),
                }
            }
            Builtin::Contains => {
                let needle = values.next().unwrap_or_default();
                let key_needle = args
                    .get(1)
                    .map(|a| value::strip_cached(&needle, &a.ty, self.program))
                    .unwrap_or_else(|| needle.clone());
                Value::Bool(match &first {
                    Value::List(items) => items.contains(&needle),
                    Value::ListView(view) => view.contains(&needle),
                    Value::MutView(mv) => self.mut_view_to_vec(mv, span)?.contains(&needle),
                    Value::Array(array) => array.contains(&needle),
                    Value::Set(items) => items.contains(&key_needle),
                    Value::Map(entries) => entries.contains_key(&key_needle),
                    Value::Str(bytes) => match &needle {
                        Value::Char(c) => bytes.contains(c),
                        Value::Str(part) => {
                            part.is_empty() || bytes.windows(part.len()).any(|at| at == &part[..])
                        }
                        Value::StrView(part) => {
                            let part = part.to_vec();
                            part.is_empty() || bytes.windows(part.len()).any(|at| at == &part[..])
                        }
                        _ => false,
                    },
                    Value::StrView(view) => match &needle {
                        Value::Char(c) => view.contains_byte(*c),
                        Value::Str(part) => {
                            let hay = view.to_vec();
                            part.is_empty() || hay.windows(part.len()).any(|at| at == &part[..])
                        }
                        Value::StrView(part) => {
                            let hay = view.to_vec();
                            let part = part.to_vec();
                            part.is_empty() || hay.windows(part.len()).any(|at| at == &part[..])
                        }
                        _ => false,
                    },
                    Value::Range(lo, hi) => {
                        let x = needle.int();
                        x >= *lo && x < *hi
                    }
                    Value::Range128(r) => {
                        let (lo, hi) = (&r.0, &r.1);
                        let x = needle.i128();
                        x >= *lo && x < *hi
                    }
                    _ => false,
                })
            }
            Builtin::MapGet => {
                let key = values.next().unwrap_or_default();
                let key = args
                    .get(1)
                    .map(|a| value::strip_cached(&key, &a.ty, self.program))
                    .unwrap_or(key);
                match &first {
                    Value::Map(entries) => optional(entries.get(&key).cloned()),
                    _ => Value::Opt(OptVal::None),
                }
            }
            Builtin::FirstGe | Builtin::FirstGt | Builtin::LastLe | Builtin::LastLt => {
                use std::ops::Bound::{Excluded, Included, Unbounded};
                let key = values.next().unwrap_or_default();
                let key = args
                    .get(1)
                    .map(|a| value::strip_cached(&key, &a.ty, self.program))
                    .unwrap_or(key);
                let range = match op {
                    Builtin::FirstGe => (Included(key), Unbounded),
                    Builtin::FirstGt => (Excluded(key), Unbounded),
                    Builtin::LastLe => (Unbounded, Included(key)),
                    _ => (Unbounded, Excluded(key)),
                };
                let first_in = matches!(op, Builtin::FirstGe | Builtin::FirstGt);
                match &first {
                    Value::Set(items) => {
                        let mut found = items.range(range);
                        optional(
                            if first_in {
                                found.next()
                            } else {
                                found.next_back()
                            }
                            .cloned(),
                        )
                    }
                    Value::Map(entries) => {
                        let mut found = entries.range(range);
                        let entry = if first_in {
                            found.next()
                        } else {
                            found.next_back()
                        };
                        optional(
                            entry.map(|(key, val)| Value::tuple(vec![key.clone(), val.clone()])),
                        )
                    }
                    _ => Value::Opt(OptVal::None),
                }
            }
            Builtin::SetMin => match &first {
                Value::Set(items) => optional(items.first().cloned()),
                Value::Map(entries) => optional(
                    entries
                        .iter()
                        .next()
                        .map(|(k, v)| Value::tuple(vec![k.clone(), v.clone()])),
                ),
                _ => Value::Opt(OptVal::None),
            },
            Builtin::SetMax => match &first {
                Value::Set(items) => optional(items.last().cloned()),
                Value::Map(entries) => optional(
                    entries
                        .iter()
                        .next_back()
                        .map(|(k, v)| Value::tuple(vec![k.clone(), v.clone()])),
                ),
                _ => Value::Opt(OptVal::None),
            },
            Builtin::Sorted => {
                let elem_ty = match &e.ty {
                    Ty::List(elem) => (**elem).clone(),
                    _ => Ty::Int,
                };
                match first {
                    Value::List(items) => {
                        let v = Rc::try_unwrap(items).unwrap_or_else(|r| (*r).clone());
                        let sorted = self.merge_sort_ty(v, &elem_ty, false, span)?;
                        Value::List(Rc::new(sorted))
                    }
                    Value::ListView(view) => {
                        let sorted = self.merge_sort_ty(view.to_vec(), &elem_ty, false, span)?;
                        Value::List(Rc::new(sorted))
                    }
                    Value::MutView(mv) => {
                        let sorted = self.merge_sort_ty(
                            self.mut_view_to_vec(&mv, span)?,
                            &elem_ty,
                            false,
                            span,
                        )?;
                        Value::List(Rc::new(sorted))
                    }
                    Value::Array(arr) => {
                        let sorted = self.merge_sort_ty(arr.to_values(), &elem_ty, false, span)?;
                        Value::List(Rc::new(sorted))
                    }
                    Value::Set(items) => {
                        let sorted = self.merge_sort_ty(
                            items.iter().cloned().collect(),
                            &elem_ty,
                            false,
                            span,
                        )?;
                        Value::List(Rc::new(sorted))
                    }
                    other => other,
                }
            }
            Builtin::Reach => {
                let pred = values.next().unwrap_or_default();
                let cyclic = values.next().unwrap_or_default().bool();
                let elems: Vec<Value> = match first {
                    Value::List(items) => (*items).clone(),
                    Value::ListView(view) => view.to_vec(),
                    Value::MutView(mv) => self.mut_view_to_vec(&mv, span)?,
                    Value::Array(arr) => arr.to_values(),
                    Value::Str(bytes) => bytes.iter().copied().map(Value::Char).collect(),
                    Value::StrView(view) => view.to_vec().into_iter().map(Value::Char).collect(),
                    _ => return trap(span, LENT),
                };
                let m = elems.len();
                let mut out = Vec::with_capacity(m);
                let mut j: usize = 0;
                for i in 0..m {
                    if j < i + 1 {
                        j = i + 1;
                    }
                    if j > i + 1 {
                        let a = if cyclic { (i + 1) % m } else { i + 1 };
                        let b = if cyclic { (j - 1) % m } else { j - 1 };
                        let check_res = self.quietly(|run| -> Res<bool> {
                            let ok_a = run
                                .call_value(
                                    pred.clone(),
                                    vec![elems[i].clone(), elems[a].clone()],
                                    span,
                                )?
                                .bool();
                            if !ok_a {
                                return Ok(false);
                            }
                            if b != a {
                                return Ok(run
                                    .call_value(
                                        pred.clone(),
                                        vec![elems[i].clone(), elems[b].clone()],
                                        span,
                                    )?
                                    .bool());
                            }
                            Ok(true)
                        });
                        if !check_res? {
                            return trap(
                                span,
                                "reach: `while` predicate is not monotone (reach dropped by more than one)",
                            );
                        }
                    }
                    let lim = if cyclic { i + m } else { m };
                    while j < lim {
                        let idx = if cyclic { j % m } else { j };
                        let ok = self
                            .call_value(
                                pred.clone(),
                                vec![elems[i].clone(), elems[idx].clone()],
                                span,
                            )?
                            .bool();
                        if !ok {
                            break;
                        }
                        j += 1;
                    }
                    out.push(Value::Int((j - i - 1) as i64));
                }
                Value::List(Rc::new(out))
            }
            Builtin::Reversed => match first {
                Value::List(items) => Value::ListView(Rc::new(ListViewVal::whole(items, true))),
                Value::ListView(view) => Value::ListView(Rc::new(view.reversed())),
                Value::MutView(mv) => Value::ListView(Rc::new(ListViewVal::whole(
                    Rc::new(self.mut_view_to_vec(&mv, span)?),
                    true,
                ))),
                Value::Str(bytes) => Value::StrView(Rc::new(StrViewVal::whole(bytes, true))),
                Value::StrView(view) => Value::StrView(Rc::new(view.reversed())),
                Value::Array(arr) => {
                    Value::ListView(Rc::new(ListViewVal::whole(Rc::new(arr.to_values()), true)))
                }
                other => other,
            },
            Builtin::StrSlice => {
                let lo = values.next().unwrap_or_default().int();
                let hi = values.next().unwrap_or_default().int();
                let (view, len) = match first {
                    Value::Str(bytes) => {
                        let n = bytes.len();
                        (StrViewVal::whole(bytes, false), n)
                    }
                    Value::StrView(v) => {
                        let n = v.len;
                        ((*v).clone(), n)
                    }
                    _ => return trap(span, "slice expects a string"),
                };
                if lo < 0 || hi < lo || hi as usize > len {
                    return trap(
                        span,
                        format!("slice {lo}..{hi} is outside a string of length {len}"),
                    );
                }
                Value::StrView(Rc::new(view.slice(lo as usize, hi as usize)))
            }
            Builtin::KeyLo => Value::Int(match &first {
                Value::ListView(v) => v.first_key(),
                Value::StrView(v) => v.lo as i64,
                Value::MutView(v) => v.lo as i64,
                _ => 0,
            }),
            Builtin::CutAfter | Builtin::CutBefore => {
                let key = values.next().unwrap_or_default().int();
                let (lo, len, rev) = match &first {
                    Value::ListView(v) => (v.first_key(), v.len as i64, v.rev),
                    Value::StrView(v) => (v.lo as i64, v.len as i64, v.rev),
                    Value::MutView(v) => (v.lo as i64, v.len as i64, false),
                    Value::Array(a) => (0, a.len() as i64, false),
                    Value::List(items) => (0, items.len() as i64, false),
                    Value::Str(bytes) => (0, bytes.len() as i64, false),
                    _ => return trap(span, "cuts need a sequence"),
                };
                if key < lo || key >= lo + len {
                    return trap(
                        span,
                        format!("key {key} is not one of the view's keys {lo}..{}", lo + len),
                    );
                }
                Value::Int(if (op == Builtin::CutAfter) != rev {
                    key + 1
                } else {
                    key
                })
            }
            Builtin::KeyPos | Builtin::PosKey => {
                let at = values.next().unwrap_or_default().int();
                let (lo, len, rev) = match &first {
                    Value::ListView(v) => (v.first_key(), v.len as i64, v.rev),
                    Value::StrView(v) => (v.lo as i64, v.len as i64, v.rev),
                    Value::MutView(v) => (v.lo as i64, v.len as i64, false),
                    Value::Range(lo, hi) => (*lo, hi.saturating_sub(*lo).max(0), false),
                    _ => return trap(span, "keys need a slice"),
                };
                Value::Int(if op == Builtin::KeyPos {
                    let i = at - lo;
                    if i < 0 || i >= len {
                        return trap(
                            span,
                            format!(
                                "key {at} is not one of the view's keys {lo}..{}: `.nth(k)` reads by position",
                                lo + len
                            ),
                        );
                    } else if rev {
                        len - 1 - i
                    } else {
                        i
                    }
                } else {
                    lo + if rev { len - 1 - at } else { at }
                })
            }
            Builtin::Rekey => {
                let lo = values.next().unwrap_or_default().int();
                let hi = values.next().unwrap_or_default().int();
                let mut view = match first {
                    Value::List(items) => ListViewVal::whole(items, false),
                    Value::Array(a) if a.dims.len() == 1 => {
                        ListViewVal::whole(Rc::new(a.to_values()), false)
                    }
                    Value::ListView(v) => (*v).clone(),
                    _ => return trap(span, "rekey needs a list"),
                };
                if hi - lo != view.len as i64 {
                    return trap(
                        span,
                        format!(
                            "rekey needs as many keys as cells: {lo}..{hi} for {} cells",
                            view.len
                        ),
                    );
                }
                view.shift = lo - view.lo as i64;
                Value::ListView(Rc::new(view))
            }
            Builtin::KeySlice => {
                let lo = values.next().unwrap_or_default().int();
                let hi = values.next().unwrap_or_default().int();
                let first = match first {
                    Value::List(items) => {
                        Value::ListView(Rc::new(ListViewVal::whole(items, false)))
                    }
                    Value::Array(a) if a.dims.len() == 1 => {
                        Value::ListView(Rc::new(ListViewVal::whole(Rc::new(a.to_values()), false)))
                    }
                    Value::MutView(mv) => {
                        let items = self.mut_view_to_vec(&mv, span)?;
                        let mut view = ListViewVal::whole(Rc::new(items), false);
                        view.shift = mv.lo as i64;
                        Value::ListView(Rc::new(view))
                    }
                    other => other,
                };
                let (first_key, len) = match &first {
                    Value::ListView(v) => (v.first_key(), v.len as i64),
                    Value::StrView(v) => (v.lo as i64, v.len as i64),
                    _ => return trap(span, "keys need a slice"),
                };
                if lo < first_key || hi < lo || hi > first_key + len {
                    return trap(
                        span,
                        format!(
                            "slice {lo}..{hi} is outside the keys {first_key}..{}",
                            first_key + len
                        ),
                    );
                }
                let rel = (lo - first_key) as usize..(hi - first_key) as usize;
                match first {
                    Value::ListView(v) => Value::ListView(Rc::new(v.slice_storage(rel))),
                    Value::StrView(v) => Value::StrView(Rc::new(v.slice_storage(rel))),
                    _ => unreachable!(),
                }
            }
            Builtin::ListSlice => {
                let lo = values.next().unwrap_or_default().int();
                let hi = values.next().unwrap_or_default().int();
                let (view, len) = match first {
                    Value::List(items) => {
                        let n = items.len();
                        (ListViewVal::whole(items, false), n)
                    }
                    Value::ListView(v) => {
                        let n = v.len;
                        ((*v).clone(), n)
                    }
                    Value::MutView(mv) => {
                        let items = Rc::new(self.mut_view_to_vec(&mv, span)?);
                        let n = items.len();
                        (ListViewVal::whole(items, false), n)
                    }
                    _ => return trap(span, "slice expects a list"),
                };
                if lo < 0 || hi < lo || hi as usize > len {
                    return trap(
                        span,
                        format!("slice {lo}..{hi} is outside a list of length {len}"),
                    );
                }
                Value::ListView(Rc::new(view.slice(lo as usize, hi as usize)))
            }
            Builtin::ToStr => match first {
                Value::Str(bytes) => Value::Str(Rc::new((*bytes).clone())),
                Value::StrView(view) => Value::Str(Rc::new(view.to_vec())),
                other => other,
            },
            Builtin::ToList => match first {
                Value::List(items) => Value::List(Rc::new((*items).clone())),
                Value::ListView(view) => Value::List(Rc::new(view.to_vec())),
                Value::MutView(mv) => Value::List(Rc::new(self.mut_view_to_vec(&mv, span)?)),
                other => other,
            },
            Builtin::StrConcat => {
                let rhs = values.next().unwrap_or_default();
                let mut out = match first {
                    Value::Str(b) => (*b).clone(),
                    Value::StrView(v) => v.to_vec(),
                    _ => Vec::new(),
                };
                match rhs {
                    Value::Str(b) => out.extend_from_slice(&b),
                    Value::StrView(v) => {
                        if !v.rev {
                            out.extend_from_slice(&v.base[v.lo..v.lo + v.len]);
                        } else {
                            out.reserve(v.len);
                            for i in 0..v.len {
                                out.push(v.get(i).unwrap());
                            }
                        }
                    }
                    _ => {}
                }
                Value::Str(Rc::new(out))
            }
            Builtin::ListConcat => {
                let rhs = values.next().unwrap_or_default();
                let mut out = match first {
                    Value::List(a) => (*a).clone(),
                    Value::ListView(v) => v.to_vec(),
                    Value::MutView(mv) => self.mut_view_to_vec(&mv, span)?,
                    Value::Array(arr) => arr.to_values(),
                    _ => Vec::new(),
                };
                match rhs {
                    Value::List(b) => out.extend_from_slice(&b),
                    Value::ListView(v) => {
                        out.reserve(v.len);
                        for i in 0..v.len {
                            out.push(v.get(i).unwrap().clone());
                        }
                    }
                    Value::MutView(mv) => out.extend(self.mut_view_to_vec(&mv, span)?),
                    Value::Array(arr) => out.extend(arr.to_values()),
                    _ => {}
                }
                Value::List(Rc::new(out))
            }
            Builtin::PosNext => {
                let len = values.next().unwrap_or_default().int();
                let next = first.int() + 1;
                optional((next < len).then_some(Value::Int(next)))
            }
            Builtin::PosPrev => {
                let ord = first.int();
                optional((ord > 0).then_some(Value::Int(ord - 1)))
            }
            Builtin::PosTryOrd(d) => {
                let (_, len) = self.domains[d];
                let ord = first.int();
                optional((ord >= 0 && ord < len).then_some(Value::Int(ord)))
            }
            Builtin::SymIdx => {
                let a = first.int();
                let b = values.next().unwrap_or_default().int();
                let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
                Value::Int(hi * (hi + 1) / 2 + lo)
            }
            Builtin::PosOffset(d) => {
                let (_, len) = self.domains[d];
                let ord = first.int();
                let delta = values.next().unwrap_or_default().int();
                let target = ord.saturating_add(delta);
                optional((target >= 0 && target < len).then_some(Value::Int(target)))
            }
            Builtin::PosCyc(d) => {
                let (_, len) = self.domains[d];
                if len <= 0 {
                    return trap(span, "cyc on empty domain");
                }
                let ord = first.int();
                let delta = values.next().unwrap_or_default().int();
                Value::Int((ord + delta).rem_euclid(len))
            }
            Builtin::BoundStepL(d) => {
                let (_, len) = self.domains[d];
                let ord = first.int();
                let k = values.next().unwrap_or_default().int();
                let b = ord + 1 - k;
                if b < 0 || b > len {
                    return trap(span, format!("window of {k} leaves domain (0..={len})"));
                }
                Value::Int(b)
            }
            Builtin::BoundStepR(d) => {
                let (_, len) = self.domains[d];
                let ord = first.int();
                let k = values.next().unwrap_or_default().int();
                let b = ord + k;
                if b < 0 || b > len {
                    return trap(span, format!("window of {k} leaves domain (0..={len})"));
                }
                Value::Int(b)
            }
            Builtin::BoundLeft(_d) => {
                let ord = first.int();
                optional((ord > 0).then_some(Value::Int(ord - 1)))
            }
            Builtin::BoundRight(d) => {
                let (_, len) = self.domains[d];
                let ord = first.int();
                optional((ord < len).then_some(Value::Int(ord)))
            }
            Builtin::RangeLo => match first {
                Value::Range(lo, _) => Value::Int(lo),
                Value::Range128(r) => Value::I128(r.0.into()),
                Value::RangeF(lo, _) => Value::Float(lo),
                _ => Value::Int(0),
            },
            Builtin::RangeHi => match first {
                Value::Range(_, hi) => Value::Int(hi),
                Value::Range128(r) => Value::I128(r.1.into()),
                Value::RangeF(_, hi) => Value::Float(hi),
                _ => Value::Int(0),
            },
            Builtin::RangeSum => {
                let by = values.next().map(|v| v.int()).unwrap_or(1);
                if by <= 0 {
                    return trap(
                        span,
                        format!("a range steps forward: `by {by}` is not positive"),
                    );
                }
                let Value::Range(lo, hi) = first else {
                    return Ok(Value::Int(0));
                };
                match closed_form::range_sum(lo, hi, by) {
                    Some(sum) => Value::Int(sum),
                    None => {
                        let why =
                            format!("the `sum` of `{lo}..{hi}` does not fit in an `Int` (64 bits)");
                        return trap(span, format!("integer overflow: {why}"));
                    }
                }
            }
            Builtin::RangeRev => match first {
                Value::Range(lo, hi) => {
                    let mut out = Vec::with_capacity(hi.saturating_sub(lo).max(0) as usize);
                    let mut v = hi - 1;
                    while v >= lo {
                        out.push(Value::Int(v));
                        if v == lo {
                            break;
                        }
                        v -= 1;
                    }
                    Value::List(Rc::new(out))
                }
                Value::Range128(r) => {
                    let (lo, hi) = *r;
                    let mut out = Vec::with_capacity(hi.saturating_sub(lo).max(0) as usize);
                    let mut v = hi - 1;
                    while v >= lo {
                        out.push(Value::I128(v.into()));
                        if v == lo {
                            break;
                        }
                        v -= 1;
                    }
                    Value::List(Rc::new(out))
                }
                _ => Value::List(Rc::new(Vec::new())),
            },
        })
    }
}
