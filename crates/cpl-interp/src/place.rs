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
                    let target = self.part_mut(&mv.target, span)?;
                    return match target {
                        Value::List(items) => Ok(Rc::make_mut(items).cell_mut(mv.lo + i * mv.step)),
                        Value::Array(arr) => {
                            Ok(&mut Rc::make_mut(arr).as_mut_values()[mv.lo + i * mv.step])
                        }
                        Value::Moved => trap(span, cpl_diag::msgs::interp::LENT.at(&[])),
                        _ => bug(span, "mutable view target is not a list or array"),
                    };
                }
                let holder = self.resolve(base, indices)?;
                cell_mut(holder, indices, exprs.len(), span)
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
                part.steps
                    .push(Step::Index(indices.by_ref().take(exprs.len()).collect()));
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
        let mut value = if part.global {
            &mut self.globals[part.at]
        } else {
            &mut self.stack[part.at]
        };
        for step in &part.steps {
            value = match step {
                Step::Field(index) => field_mut(value, *index, span)?,
                Step::Some => inside_mut(value, span)?,
                Step::Index(indices) => cell_mut(value, &mut indices.iter(), indices.len(), span)?,
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
                    match &array.data {
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
