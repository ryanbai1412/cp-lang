//! `impl Show` inside a compound value: a tuple, record, struct, optional or
//! list holding a type with a `show` fn prints that part through the fn,
//! as the generated C++ does.

use crate::observe::Observer;
use crate::value::{OptVal, Value};
use crate::*;

impl<'p, 'o, O: Observer> Interp<'p, 'o, O> {
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
}
