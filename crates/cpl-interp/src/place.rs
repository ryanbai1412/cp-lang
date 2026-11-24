//! Places: resolving a place to the part of a value it names, and reading or changing that part.

use super::*;

pub(super) fn leading_indices(place: &Place) -> usize {
    match &place.kind {
        PlaceKind::Global(_) | PlaceKind::Local(_) => 0,
        PlaceKind::Index(base, indices) => match &base.kind {
            PlaceKind::Global(_) | PlaceKind::Index(..)
                if leading_indices(base) == index_count(base) =>
            {
                leading_indices(base) + indices.len()
            }
            _ => leading_indices(base),
        },
        PlaceKind::Field(base, _) | PlaceKind::Some(base) => leading_indices(base),
    }
}

pub(super) fn index_count(place: &Place) -> usize {
    match &place.kind {
        PlaceKind::Global(_) | PlaceKind::Local(_) => 0,
        PlaceKind::Index(base, indices) => index_count(base) + indices.len(),
        PlaceKind::Field(base, _) | PlaceKind::Some(base) => index_count(base),
    }
}

impl<'p, 'o, O: Observer> Interp<'p, 'o, O> {
    /// Evaluates every index along `place`, outermost container first.
    pub(super) fn indices(&mut self, place: &'p Place, out: &mut Vec<Value>) -> Res<()> {
        match &place.kind {
            PlaceKind::Local(_) | PlaceKind::Global(_) => {}
            PlaceKind::Field(base, _) | PlaceKind::Some(base) => self.indices(base, out)?,
            PlaceKind::Index(base, indices) => {
                self.indices(base, out)?;
                for index in indices {
                    out.push(self.eval(index)?);
                }
            }
        }
        Ok(())
    }

    /// The storage `place` names, with its indices already evaluated.
    pub(super) fn resolve<'s>(
        &'s mut self,
        place: &Place,
        indices: &mut std::slice::Iter<'_, Value>,
    ) -> Res<&'s mut Value> {
        let span = place.span;
        match &place.kind {
            PlaceKind::Local(slot) => {
                let at = self.base + slot;
                let part = match &self.stack[at] {
                    Value::Alias(global, at) => Part {
                        global: *global,
                        at: *at,
                        steps: Vec::new(),
                    },
                    Value::Part(part) => return self.part_mut(&Rc::clone(part), span),
                    _ => Part {
                        global: false,
                        at,
                        steps: Vec::new(),
                    },
                };
                self.part_mut(&part, span)
            }
            PlaceKind::Global(id) => Ok(&mut self.globals[*id]),
            PlaceKind::Field(base, index) => field_mut(self.resolve(base, indices)?, *index, span),
            PlaceKind::Some(base) => inside_mut(self.resolve(base, indices)?, span),
            PlaceKind::Index(base, exprs) => {
                if matches!(base.ty, Ty::MutView(..)) {
                    let mv = match self.resolve(base, indices)? {
                        Value::MutView(mv) => Rc::clone(mv),
                        Value::Moved => return trap(span, cpl_diag::msgs::interp::LENT.at(&[])),
                        _ => return bug(span, "expected mutable view"),
                    };
                    let index = indices.next().map_or(0, Value::int);
                    let Some(i) = usize::try_from(index).ok().filter(|i| *i < mv.len) else {
                        return trap(
                            span,
                            cpl_diag::msgs::interp::INDEX_OUTSIDE_LIST_LENGTH
                                .at(&[&index, &mv.len]),
                        );
                    };
                    return self.mut_view_cell(&mv, i, span);
                }
                let program = self.program;
                let (Ty::Map(kt, _) | Ty::HashMap(kt, _)) = &base.ty else {
                    let holder = self.resolve(base, indices)?;
                    return cell_mut(holder, indices, exprs.len(), span);
                };
                if !value::keys::user_ordered(kt, program) {
                    let holder = self.resolve(base, indices)?;
                    return cell_mut(holder, indices, exprs.len(), span);
                }
                // The key's `Ord` runs user code, so the map is out of its place meanwhile.
                let mut again = indices.clone();
                let holder = self.resolve(base, indices)?;
                let held = std::mem::replace(holder, Value::Moved);
                let key = indices.next().cloned().unwrap_or_default();
                let key = value::strip_cached(&key, kt, program);
                let found = match &held {
                    Value::Map(m) => {
                        m.sorted_pos(&key, &mut |t, a, b| self.less_ty_val(t, a, b, span))
                    }
                    _ => Ok(None),
                };
                let holder = self.resolve(base, &mut again)?;
                *holder = held;
                let Value::Map(entries) = holder else {
                    return cell_mut(holder, &mut std::slice::from_ref(&key).iter(), 1, span);
                };
                match found?.and_then(|at| Rc::make_mut(entries).sorted_at_mut(at)) {
                    Some(value) => Ok(value),
                    None => trap(span, cpl_diag::msgs::interp::THIS_KEY_NOT_IN_MAP.at(&[])),
                }
            }
        }
    }

    /// The part of a variable `place` names, with its indices evaluated.
    pub(super) fn part_of(&self, place: &Place, indices: &mut std::vec::IntoIter<Value>) -> Part {
        let (base, step) = match &place.kind {
            PlaceKind::Global(id) => {
                return Part {
                    global: true,
                    at: *id,
                    steps: Vec::new(),
                }
            }
            PlaceKind::Local(slot) => {
                let at = self.base + slot;
                return match &self.stack[at] {
                    Value::Alias(global, at) => Part {
                        global: *global,
                        at: *at,
                        steps: Vec::new(),
                    },
                    Value::Part(part) => Part::clone(part),
                    _ => Part {
                        global: false,
                        at,
                        steps: Vec::new(),
                    },
                };
            }
            PlaceKind::Field(base, index) => (base, Step::Field(*index)),
            PlaceKind::Some(base) => (base, Step::Some),
            PlaceKind::Index(base, exprs) => {
                let mut part = self.part_of(base, indices);
                let step = match &base.ty {
                    Ty::Map(kt, _) | Ty::HashMap(kt, _)
                        if value::keys::user_ordered(kt, self.program) =>
                    {
                        let key = indices.next().unwrap_or_default();
                        Step::Key(value::strip_cached(&key, kt, self.program))
                    }
                    _ => Step::Index(indices.by_ref().take(exprs.len()).collect()),
                };
                part.steps.push(step);
                return part;
            }
        };
        let mut part = self.part_of(base, indices);
        part.steps.push(step);
        part
    }

    /// The storage of `part`.
    /// The caller's place a param is lent from.
    pub(crate) fn lender(&mut self, part: &Part) -> Res<&mut Value> {
        self.part_mut(part, Span::default())
    }

    pub(super) fn part_mut(&mut self, part: &Part, span: Span) -> Res<&mut Value> {
        if !part.steps.iter().any(|s| matches!(s, Step::Key(_))) {
            return self.walk_part(part, part.steps.len(), &[], span);
        }
        // A key's `Ord` runs user code, so each such map is out of its place while it searches.
        let mut found = vec![(0, 0); part.steps.len()];
        for (i, step) in part.steps.iter().enumerate() {
            let Step::Key(key) = step else { continue };
            let held = std::mem::replace(self.walk_part(part, i, &found, span)?, Value::Moved);
            let at = match &held {
                Value::Map(m) => m.sorted_pos(key, &mut |t, a, b| self.less_ty_val(t, a, b, span)),
                _ => Ok(None),
            };
            *self.walk_part(part, i, &found, span)? = held;
            match at? {
                Some(at) => found[i] = at,
                None => return trap(span, cpl_diag::msgs::interp::THIS_KEY_NOT_IN_MAP.at(&[])),
            }
        }
        self.walk_part(part, part.steps.len(), &found, span)
    }

    /// The storage of the first `upto` steps of `part`, with `found` the entry each `Key`
    /// step names.
    fn walk_part(
        &mut self,
        part: &Part,
        upto: usize,
        found: &[value::keys::Pos],
        span: Span,
    ) -> Res<&mut Value> {
        let mut value = if part.global {
            &mut self.globals[part.at]
        } else {
            &mut self.stack[part.at]
        };
        for (i, step) in part.steps[..upto].iter().enumerate() {
            value = match step {
                Step::Field(index) => field_mut(value, *index, span)?,
                Step::Some => inside_mut(value, span)?,
                Step::Index(indices) => cell_mut(value, &mut indices.iter(), indices.len(), span)?,
                Step::Key(key) if !matches!(value, Value::Map(m) if m.is_sorted_by_user()) => {
                    cell_mut(value, &mut std::slice::from_ref(key).iter(), 1, span)?
                }
                Step::Key(_) => match value {
                    Value::Map(m) => match Rc::make_mut(m).sorted_at_mut(found[i]) {
                        Some(v) => v,
                        None => return bug(span, "a sorted entry moved while lent"),
                    },
                    _ => return bug(span, "index of a value that has no elements"),
                },
            };
        }
        Ok(value)
    }

    /// The value of `part`, if it is still there.
    pub(super) fn part(&self, part: &Part) -> Option<&Value> {
        let mut value = if part.global {
            self.globals.get(part.at)?
        } else {
            self.stack.get(part.at)?
        };
        for step in &part.steps {
            value = match (step, value) {
                (Step::Field(index), Value::Tuple(TupleVal::Boxed(fields))) => {
                    fields.get(*index)?
                }
                (Step::Some, Value::Opt(OptVal::Boxed(inner))) => inner,
                (Step::Index(indices), Value::Array(array)) => {
                    let at = array_offset(&array.dims, indices.iter(), Span::new(0, 0)).ok()?;
                    match &*array.data {
                        value::Cells::Values(v) => v.get(at)?,
                        _ => return None,
                    }
                }
                (Step::Index(indices), Value::List(items)) => {
                    items.stored(usize::try_from(indices.first()?.int()).ok()?)?
                }
                // A sorted map's search runs user code, which a read-only peek can't.
                (Step::Index(indices), Value::Map(entries)) if !entries.is_sorted_by_user() => {
                    entries
                        .get(indices.first()?, &mut |_, a, b| Ok(a < b))
                        .ok()??
                }
                _ => return None,
            };
        }
        Some(value)
    }

    /// Cell `i` (below `mv.len`) of the view `mv`, where it writes.
    pub(super) fn mut_view_cell(
        &mut self,
        mv: &MutViewVal,
        i: usize,
        span: Span,
    ) -> Res<&mut Value> {
        match self.part_mut(&mv.target, span)? {
            Value::List(items) => Ok(Rc::make_mut(items).cell_mut(mv.lo + i * mv.step)),
            Value::Array(arr) => Ok(&mut Rc::make_mut(arr).as_mut_values()[mv.lo + i * mv.step]),
            Value::Moved => trap(span, cpl_diag::msgs::interp::LENT.at(&[])),
            _ => bug(span, "mutable view target is not a list or array"),
        }
    }

    pub(super) fn mut_view_get(&self, mv: &MutViewVal, idx: usize, span: Span) -> Res<Value> {
        if idx >= mv.len {
            return trap(
                span,
                cpl_diag::msgs::interp::INDEX_OUTSIDE_LIST_LENGTH.at(&[&idx, &mv.len]),
            );
        }
        match self.part(&mv.target) {
            Some(Value::List(items)) => Ok(items.get_unchecked_val(mv.lo + idx * mv.step)),
            Some(Value::Array(arr)) => Ok(arr.get_unchecked_val(mv.lo + idx * mv.step)),
            Some(Value::Moved) => trap(span, cpl_diag::msgs::interp::LENT.at(&[])),
            _ => bug(span, "mutable view target is not a list or array"),
        }
    }

    /// Sorts the list, array, string or view at `place`, its indices
    /// already evaluated.
    pub(crate) fn sort_place(
        &mut self,
        place: &'p Place,
        sort_by: &'p SortBy,
        desc: bool,
        indices: Vec<Value>,
        span: Span,
    ) -> Res<Value> {
        let cell = self.resolve(place, &mut indices.iter())?;
        // A view sorts where it writes, so it stays in its place; `dims`
        // is a list's or an array's, and neither is a string's.
        let (mut items_vec, dims, view) = match std::mem::replace(cell, Value::Moved) {
            Value::MutView(mv) => {
                *cell = Value::MutView(Rc::clone(&mv));
                (self.mut_view_to_vec(&mv, span)?, None, Some(mv))
            }
            Value::Str(bytes) => (bytes.iter().copied().map(Value::Char).collect(), None, None),
            seq => {
                let (items, dims) = crate::vm::seq_items(seq, span)?;
                (items, Some(dims), None)
            }
        };
        let elem_ty = match &place.ty {
            Ty::List(e) | Ty::MutView(e, _) | Ty::Array { elem: e, .. } => (**e).clone(),
            Ty::Str => Ty::Char,
            _ => Ty::Int,
        };
        match sort_by {
            SortBy::Natural => {
                items_vec = self.merge_sort_ty(items_vec, &elem_ty, desc, span)?;
            }
            SortBy::Key(slot, key) => {
                let base = self.base;
                let mut keys = Vec::with_capacity(items_vec.len());
                for item in &items_vec {
                    self.stack[base + slot] = item.clone();
                    keys.push(self.eval(key)?);
                }
                let order =
                    self.keyed_order(keys.len(), &|i| keys[i].clone(), &key.ty, desc, span)?;
                items_vec = crate::sort::permuted(items_vec, order);
            }
            SortBy::Fn { func, is_order } => {
                let fv = self.eval(func)?;
                if *is_order {
                    // Merge-sort using fallible comparator `fv(a, b) -> Bool`,
                    // trapping on strict-weak-order violations (matching `cpl::sort_cmp` under LOCAL).
                    items_vec = self.checked_merge_sort_by(items_vec, &fv, desc, span)?;
                } else {
                    let key_ty = match &func.ty {
                        Ty::Fn(_, r) => (**r).clone(),
                        _ => Ty::Int,
                    };
                    let mut keys = Vec::with_capacity(items_vec.len());
                    for item in &items_vec {
                        keys.push(self.call_value(fv.clone(), vec![item.clone()], span)?);
                    }
                    let order =
                        self.keyed_order(keys.len(), &|i| keys[i].clone(), &key_ty, desc, span)?;
                    items_vec = crate::sort::permuted(items_vec, order);
                }
            }
        }
        // A view may name a global.
        let shown_next = place.root_global().is_some() || view.is_some();
        let sorted = match (view, dims) {
            (Some(mv), _) => {
                for (k, item) in items_vec.into_iter().enumerate() {
                    *self.mut_view_cell(&mv, k, span)? = item;
                }
                None
            }
            (None, Some(dims)) => Some(crate::vm::seq_of_items(items_vec, dims)),
            (None, None) => Some(Value::Str(Rc::new(
                items_vec
                    .into_iter()
                    .map(|v| match v {
                        Value::Char(c) => c,
                        _ => 0,
                    })
                    .collect(),
            ))),
        };
        if let Some(sorted) = sorted {
            *self.resolve(place, &mut indices.iter())? = sorted;
        }
        // Not a step of its own, as before: the next step shows it.
        if let (true, Some(tracer)) = (shown_next, self.tracer.as_mut()) {
            tracer.on(crate::observe::Event::GlobalsMayChange);
        }
        Ok(Value::Unit)
    }

    /// The view `mut place[a..b]`, or with `line` line `a` of the grid at
    /// `place` along that axis (1: a row), the place's indices already
    /// evaluated.
    pub(crate) fn mut_view(
        &self,
        place: &Place,
        line: Option<usize>,
        a: i64,
        b: i64,
        indices: Vec<Value>,
        span: Span,
    ) -> Res<Value> {
        let part = self.part_of(place, &mut indices.into_iter());
        let (base_lo, base_len, step, key0, dims) = match self.part(&part) {
            Some(Value::Moved) => return trap(span, cpl_diag::msgs::interp::LENT.at(&[])),
            Some(Value::Array(arr)) if line.is_some() && arr.dims.len() == 2 => (
                0,
                0,
                1,
                0,
                Some((arr.dims[0] as usize, arr.dims[1] as usize)),
            ),
            _ if line.is_some() => return bug(span, "a grid line needs a 2D array"),
            Some(Value::List(items)) => (0usize, items.len(), 1, 0, None),
            Some(Value::Array(arr)) => (0usize, arr.len(), 1, 0, None),
            Some(Value::MutView(mv)) => (mv.lo, mv.len, mv.step, mv.key0, None),
            _ => {
                return trap(
                    span,
                    cpl_diag::msgs::interp::SLICE_EXPECTS_LIST_OR_ARRAY.at(&[]),
                )
            }
        };
        let target = match self.part(&part) {
            Some(Value::MutView(mv)) => mv.target.clone(),
            _ => part,
        };
        let (lo, len, step, key0) = match (dims, line) {
            (Some((rows, cols)), Some(axis)) => {
                let (fixed, len, step) = if axis == 1 {
                    (rows, cols, 1)
                } else {
                    (cols, rows, cols)
                };
                let Some(at) = usize::try_from(a).ok().filter(|at| *at < fixed) else {
                    return trap(
                        span,
                        cpl_diag::msgs::interp::INDEX_OUTSIDE_AXIS_LENGTH.at(&[&a, &fixed]),
                    );
                };
                (if axis == 1 { at * cols } else { at }, len, step, 0)
            }
            _ if a < 0 || b < a || b as usize > base_len => {
                return trap(
                    span,
                    cpl_diag::msgs::interp::SLICE_OUTSIDE_LIST_LENGTH.at(&[&a, &b, &base_len]),
                );
            }
            _ => (
                base_lo + a as usize * step,
                (b - a) as usize,
                step,
                key0 + a,
            ),
        };
        Ok(Value::MutView(Rc::new(MutViewVal {
            target,
            lo,
            len,
            step,
            key0,
        })))
    }

    pub(super) fn mut_view_to_vec(&self, mv: &MutViewVal, span: Span) -> Res<Vec<Value>> {
        match self.part(&mv.target) {
            Some(Value::List(items)) if mv.step == 1 => Ok(items.values_in(mv.lo, mv.lo + mv.len)),
            Some(Value::List(items)) => Ok((0..mv.len)
                .map(|i| items.get_unchecked_val(mv.lo + i * mv.step))
                .collect()),
            Some(Value::Array(arr)) => {
                let mut out = Vec::with_capacity(mv.len);
                for i in 0..mv.len {
                    out.push(arr.get_unchecked_val(mv.lo + i * mv.step));
                }
                Ok(out)
            }
            Some(Value::Moved) => trap(span, cpl_diag::msgs::interp::LENT.at(&[])),
            _ => bug(span, "mutable view target is not a list or array"),
        }
    }
}
