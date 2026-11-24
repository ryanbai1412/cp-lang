//! Assignment: storing into places, compound assignment, and mutating calls.

use super::*;

pub(super) fn field_mut(holder: &mut Value, index: usize, span: Span) -> Res<&mut Value> {
    match holder {
        Value::Tuple(fields) => Ok(&mut fields.as_mut_vec()[index]),
        Value::Moved => trap(span, LENT),
        _ => trap(span, "internal: field of a value without fields"),
    }
}

pub(super) fn inside_mut(holder: &mut Value, span: Span) -> Res<&mut Value> {
    match holder {
        Value::Opt(opt) => {
            if let OptVal::Int(v) = *opt {
                *opt = OptVal::Boxed(Rc::new(Value::Int(v)));
            }
            match opt {
                OptVal::Boxed(inner) => Ok(Rc::make_mut(inner)),
                OptVal::None => trap(span, "there is no value here: it is `None`"),
                OptVal::Int(_) => unreachable!(),
            }
        }
        Value::Moved => trap(span, LENT),
        _ => trap(span, "there is no value here: it is `None`"),
    }
}

/// The cell of `holder` at the next `count` of `indices`.
pub(super) fn cell_mut<'s>(
    holder: &'s mut Value,
    indices: &mut std::slice::Iter<'_, Value>,
    count: usize,
    span: Span,
) -> Res<&'s mut Value> {
    match holder {
        Value::Array(array) => {
            let at = array_offset(&array.dims, indices.by_ref().take(count), span)?;
            Ok(&mut Rc::make_mut(array).as_mut_values()[at])
        }
        Value::List(items) => {
            let index = indices.next().map_or(0, Value::int);
            let len = items.len();
            match usize::try_from(index).ok().filter(|i| *i < len) {
                Some(i) => Ok(Rc::make_mut(items).cell_mut(i)),
                None => trap(
                    span,
                    format!("index {index} is outside a list of length {len}"),
                ),
            }
        }
        Value::Map(entries) => {
            let key = indices.next().cloned().unwrap_or_default();
            match Rc::make_mut(entries).get_mut(&key) {
                Some(value) => Ok(value),
                None => trap(
                    span,
                    "this key is not in the map; store with `m[k] = v` first",
                ),
            }
        }
        Value::Moved => trap(span, LENT),
        _ => trap(span, "internal: index of a value that has no elements"),
    }
}

impl<'p, 'o, O: Observer> Interp<'p, 'o, O> {
    /// Fast-path assignment when tracing is disabled: avoids allocating an
    /// index `Vec` for direct locals/globals and 1D/2D array or list writes.
    pub(super) fn assign_untraced(
        &mut self,
        place: &'p Place,
        op: AssignOp,
        value: &'p Expr,
    ) -> Res<()> {
        match &place.kind {
            PlaceKind::Local(slot) => {
                let value = self.eval(value)?;
                let at = self.base + slot;
                if !matches!(self.stack[at], Value::Alias(..) | Value::Part(..)) {
                    let cell = &mut self.stack[at];
                    if op == AssignOp::Set {
                        *cell = value;
                    } else {
                        let old = std::mem::take(cell);
                        if matches!(old, Value::Moved) {
                            *cell = old;
                            return trap(place.span, LENT);
                        }
                        *cell = combine(op, old, value, place.span)?;
                    }
                    return Ok(());
                }
                let cell = self.resolve(place, &mut [].iter())?;
                if op == AssignOp::Set {
                    *cell = value;
                } else {
                    let old = std::mem::take(cell);
                    if matches!(old, Value::Moved) {
                        *cell = old;
                        return trap(place.span, LENT);
                    }
                    *cell = combine(op, old, value, place.span)?;
                }
                return Ok(());
            }
            PlaceKind::Global(id) => {
                let value = self.eval(value)?;
                let cell = &mut self.globals[*id];
                if op == AssignOp::Set {
                    *cell = value;
                } else {
                    let old = std::mem::take(cell);
                    if matches!(old, Value::Moved) {
                        *cell = old;
                        return trap(place.span, LENT);
                    }
                    *cell = combine(op, old, value, place.span)?;
                }
                return Ok(());
            }
            PlaceKind::Index(base, exprs)
                if exprs.len() <= 2
                    && !matches!(base.ty, Ty::Str | Ty::Map(..) | Ty::HashMap(..))
                    && matches!(base.kind, PlaceKind::Local(_) | PlaceKind::Global(_)) =>
            {
                let value = self.eval(value)?;
                let n = exprs.len();
                let i0 = if n >= 1 {
                    self.eval(&exprs[0])?.int()
                } else {
                    0
                };
                let i1 = if n == 2 {
                    self.eval(&exprs[1])?.int()
                } else {
                    0
                };
                let holder: &mut Value = match &base.kind {
                    PlaceKind::Global(id) => &mut self.globals[*id],
                    PlaceKind::Local(slot) => {
                        let at = self.base + slot;
                        match &self.stack[at] {
                            Value::Alias(true, id) => {
                                let id = *id;
                                &mut self.globals[id]
                            }
                            Value::Alias(false, target) => {
                                let target = *target;
                                &mut self.stack[target]
                            }
                            Value::Part(part) => {
                                let part = Rc::clone(part);
                                self.part_mut(&part, base.span)?
                            }
                            _ => &mut self.stack[at],
                        }
                    }
                    _ => unreachable!(),
                };
                let span = place.span;
                let cell: &mut Value = match holder {
                    Value::Array(array) if array.dims.len() == 1 && n == 1 => {
                        let len = array.dims[0];
                        if i0 < 0 || i0 >= len {
                            return trap(
                                span,
                                format!("index {i0} is outside an array dimension of length {len}"),
                            );
                        }
                        let at = i0 as usize;
                        let arr = Rc::make_mut(array);
                        if op == AssignOp::Set {
                            arr.set(at, value);
                            return Ok(());
                        }
                        let old = arr.get_unchecked_val(at);
                        let next = combine(op, old, value, span)?;
                        arr.set(at, next);
                        return Ok(());
                    }
                    Value::Array(array) if array.dims.len() == 2 && n == 2 => {
                        let l0 = array.dims[0];
                        if i0 < 0 || i0 >= l0 {
                            return trap(
                                span,
                                format!("index {i0} is outside an array dimension of length {l0}"),
                            );
                        }
                        let l1 = array.dims[1];
                        if i1 < 0 || i1 >= l1 {
                            return trap(
                                span,
                                format!("index {i1} is outside an array dimension of length {l1}"),
                            );
                        }
                        let at = (i0 * l1 + i1) as usize;
                        let arr = Rc::make_mut(array);
                        if op == AssignOp::Set {
                            arr.set(at, value);
                            return Ok(());
                        }
                        let old = arr.get_unchecked_val(at);
                        let next = combine(op, old, value, span)?;
                        arr.set(at, next);
                        return Ok(());
                    }
                    Value::List(items) if n == 1 => {
                        let len = items.len();
                        match usize::try_from(i0).ok().filter(|i| *i < len) {
                            Some(i) => Rc::make_mut(items).cell_mut(i),
                            None => {
                                return trap(
                                    span,
                                    format!("index {i0} is outside a list of length {len}"),
                                )
                            }
                        }
                    }
                    _ => {
                        let buf = [Value::Int(i0), Value::Int(i1)];
                        self.resolve(place, &mut buf[..n].iter())?
                    }
                };
                if op == AssignOp::Set {
                    *cell = value;
                } else {
                    let old = std::mem::take(cell);
                    if matches!(old, Value::Moved) {
                        *cell = old;
                        return trap(span, LENT);
                    }
                    *cell = combine(op, old, value, span)?;
                }
                return Ok(());
            }
            _ => {}
        }
        let _ = self.assign(place, op, value)?;
        Ok(())
    }

    /// Stores into `place`, and gives the indices it was reached by.
    pub(super) fn assign(
        &mut self,
        place: &'p Place,
        op: AssignOp,
        value: &'p Expr,
    ) -> Res<Vec<Value>> {
        // As in C++17: the right side first, then the place.
        let value = self.eval(value)?;
        let mut indices = Vec::new();
        self.indices(place, &mut indices)?;
        // Storing into a string or under a new map key has no cell to
        // point at, so those two go through their container.
        if let PlaceKind::Index(base, _) = &place.kind {
            if matches!(base.ty, Ty::Str | Ty::Map(..) | Ty::HashMap(..)) {
                let (key, rest) = indices.split_last().expect("an index");
                let program = self.program;
                let holder = self.resolve(base, &mut rest.iter())?;
                match holder {
                    Value::Str(bytes) => {
                        let index = key.int();
                        let len = bytes.len();
                        let Some(i) = usize::try_from(index).ok().filter(|i| *i < len) else {
                            return trap(
                                place.span,
                                format!("index {index} is outside a string of length {len}"),
                            );
                        };
                        let old = Value::Char(bytes[i]);
                        let new = combine(op, old, value, place.span)?;
                        if let Value::Char(c) = new {
                            Rc::make_mut(bytes)[i] = c;
                        }
                        return Ok(indices);
                    }
                    Value::Map(entries) if op == AssignOp::Set => {
                        let k = match &base.ty {
                            Ty::Map(kt, _) | Ty::HashMap(kt, _) => {
                                value::strip_cached(key, kt, program)
                            }
                            _ => key.clone(),
                        };
                        Rc::make_mut(entries).insert(k, value);
                        return Ok(indices);
                    }
                    Value::Map(entries) => {
                        let k = match &base.ty {
                            Ty::Map(kt, _) | Ty::HashMap(kt, _) => {
                                value::strip_cached(key, kt, program)
                            }
                            _ => key.clone(),
                        };
                        let map = Rc::make_mut(entries);
                        let next =
                            if let Some(old) = map.get(&k).cloned() {
                                combine(op, old, value, place.span)?
                            } else {
                                match op {
                                    AssignOp::Max
                                    | AssignOp::Min
                                    | AssignOp::Binary(
                                        BinOp::Add
                                        | BinOp::Mul
                                        | BinOp::BitOr
                                        | BinOp::BitXor
                                        | BinOp::BitAnd,
                                    ) => value,
                                    AssignOp::Binary(BinOp::Sub) => {
                                        combine(op, zero(&place.ty, program), value, place.span)?
                                    }
                                    _ => return trap(
                                        place.span,
                                        "this key is not in the map; store with `m[k] = v` first",
                                    ),
                                }
                            };
                        map.insert(k, next);
                        return Ok(indices);
                    }
                    Value::Moved => return trap(place.span, LENT),
                    _ => {}
                }
            }
        }
        let cell = self.resolve(place, &mut indices.iter())?;
        if op == AssignOp::Set {
            *cell = value;
        } else {
            let old = std::mem::take(cell);
            if matches!(old, Value::Moved) {
                *cell = old;
                return trap(place.span, LENT);
            }
            *cell = combine(op, old, value, place.span)?;
        }
        Ok(indices)
    }

    pub(super) fn mutate(
        &mut self,
        op: Mutation,
        place: &'p Place,
        args: &'p [Expr],
        span: Span,
    ) -> Res<Value> {
        // As in C++17: what is called on comes before the arguments.
        let mut indices = Vec::new();
        self.indices(place, &mut indices)?;
        let mut arg_vals = Vec::with_capacity(args.len());
        for a in args {
            arg_vals.push(self.eval(a)?);
        }
        // A `yield` of a fn that feeds a loop: the body runs on the value.
        if let (PlaceKind::Local(slot), Some(fed)) = (&place.kind, self.fed_loops.last()) {
            if fed.callee == self.base && fed.acc == *slot {
                match (op, arg_vals.pop()) {
                    (Mutation::Push, Some(value)) => {
                        let updated = self.feed(value, span)?;
                        if let Some(Expr {
                            kind: ExprKind::Local(item_slot),
                            ..
                        }) = args.first()
                        {
                            let at = self.base + *item_slot;
                            let lent = match &self.stack[at] {
                                Value::Alias(global, to) => Some(Part {
                                    global: *global,
                                    at: *to,
                                    steps: Vec::new(),
                                }),
                                Value::Part(part) => Some(Part::clone(part)),
                                _ => None,
                            };
                            match lent {
                                Some(part) => *self.part_mut(&part, span)? = updated,
                                None => self.stack[at] = updated,
                            }
                        }
                    }
                    (Mutation::Extend, Some(Value::List(items))) => {
                        for item in items.iter() {
                            self.feed(item.clone(), span)?;
                        }
                    }
                    (Mutation::Extend, Some(Value::Array(array))) => {
                        for i in 0..array.len() {
                            self.feed(array.get_unchecked_val(i), span)?;
                        }
                    }
                    _ => {
                        return trap(
                            span,
                            "internal: a `yields` fn did something else to its list",
                        )
                    }
                }
                return Ok(Value::Unit);
            }
        }
        let arg = arg_vals.first().cloned().unwrap_or(Value::Unit);
        let arg2 = arg_vals.get(1).cloned().unwrap_or(Value::Unit);
        if op == Mutation::Shuffle {
            let mut rng_state = self.rng_state;
            let choices = self.choices.clone();
            let mut below = |n: usize| {
                draw_below(choices.as_deref(), &mut rng_state, n as u128, span.start) as usize
            };
            let mv_opt = match self.resolve(place, &mut indices.iter())? {
                Value::MutView(mv) => Some(Rc::clone(mv)),
                _ => None,
            };
            let res = if let Some(mv) = mv_opt {
                match self.part_mut(&mv.target, span)? {
                    Value::List(items) => {
                        let v = Rc::make_mut(items);
                        for i in (1..mv.len).rev() {
                            let j = below(i + 1);
                            v.swap(mv.lo + i, mv.lo + j);
                        }
                        Value::Unit
                    }
                    Value::Array(arr) => {
                        let a = Rc::make_mut(arr);
                        for i in (1..mv.len).rev() {
                            let j = below(i + 1);
                            a.swap(mv.lo + i, mv.lo + j);
                        }
                        Value::Unit
                    }
                    Value::Moved => return trap(span, LENT),
                    _ => return trap(span, "internal: this value can't be shuffled"),
                }
            } else {
                let cell = self.resolve(place, &mut indices.iter())?;
                match cell {
                    Value::Moved => return trap(span, LENT),
                    Value::List(items) => {
                        let v = Rc::make_mut(items);
                        for i in (1..v.len()).rev() {
                            let j = below(i + 1);
                            v.swap(i, j);
                        }
                        Value::Unit
                    }
                    Value::Array(arr) => {
                        let a = Rc::make_mut(arr);
                        let n = a.len();
                        for i in (1..n).rev() {
                            let j = below(i + 1);
                            a.swap(i, j);
                        }
                        Value::Unit
                    }
                    _ => return trap(span, "internal: this value can't be shuffled"),
                }
            };
            self.rng_state = rng_state;
            return Ok(res);
        }
        let mv_opt = match self.resolve(place, &mut indices.iter())? {
            Value::MutView(mv) => Some(Rc::clone(mv)),
            _ => None,
        };
        if let Some(mv) = mv_opt {
            let target = self.part_mut(&mv.target, span)?;
            return Ok(match (op, target) {
                (_, Value::Moved) => return trap(span, LENT),
                (Mutation::Reverse, Value::List(items)) => {
                    Rc::make_mut(items).reverse_range(mv.lo, mv.lo + mv.len);
                    Value::Unit
                }
                (Mutation::Reverse, Value::Array(arr)) => {
                    let a = Rc::make_mut(arr);
                    for i in 0..mv.len / 2 {
                        a.swap(mv.lo + i, mv.lo + mv.len - 1 - i);
                    }
                    Value::Unit
                }
                (Mutation::Swap, Value::List(items)) => {
                    let (i, j) = (arg.int(), arg2.int());
                    let v = Rc::make_mut(items);
                    match (usize::try_from(i).ok(), usize::try_from(j).ok()) {
                        (Some(i), Some(j)) if i < mv.len && j < mv.len => {
                            v.swap(mv.lo + i, mv.lo + j);
                        }
                        _ => {
                            return trap(
                                span,
                                format!("swap({i}, {j}) is outside a list of length {}", mv.len),
                            )
                        }
                    }
                    Value::Unit
                }
                (Mutation::Swap, Value::Array(arr)) => {
                    let (i, j) = (arg.int(), arg2.int());
                    let a = Rc::make_mut(arr);
                    match (usize::try_from(i).ok(), usize::try_from(j).ok()) {
                        (Some(i), Some(j)) if i < mv.len && j < mv.len => {
                            a.swap(mv.lo + i, mv.lo + j);
                        }
                        _ => {
                            return trap(
                                span,
                                format!("swap({i}, {j}) is outside an array of length {}", mv.len),
                            )
                        }
                    }
                    Value::Unit
                }
                (Mutation::Fill, Value::List(items)) => {
                    Rc::make_mut(items).fill_range(mv.lo, mv.lo + mv.len, arg);
                    Value::Unit
                }
                (Mutation::Fill, Value::Array(arr)) => {
                    let a = Rc::make_mut(arr);
                    for i in 0..mv.len {
                        a.set(mv.lo + i, arg.clone());
                    }
                    Value::Unit
                }
                _ => return trap(span, "internal: this value can't be changed that way"),
            });
        }
        let program = self.program;
        let cell = self.resolve(place, &mut indices.iter())?;
        mutation::mutate_cell(op, cell, &place.ty, arg, arg2, program, span)
    }
}
