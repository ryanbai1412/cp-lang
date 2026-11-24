//! Reading a value of a spec from the input, which both engines share.

use crate::observe::Observer;
use crate::value::*;
use crate::*;

impl<'p, 'o, O: Observer> Interp<'p, 'o, O> {
    /// The value of `e`: from `given` when the caller evaluated it.
    pub(crate) fn given(&mut self, e: &'p Expr, given: &[(&'p Expr, Value)]) -> Res<Value> {
        match given.iter().find(|(x, _)| std::ptr::eq(*x, e)) {
            Some((_, v)) => Ok(v.clone()),
            None => self.eval(e),
        }
    }

    /// A value of `spec` from the input, with `given` the values of the
    /// expressions in it the caller already evaluated.
    pub(crate) fn read(
        &mut self,
        spec: &'p Spec,
        span: Span,
        given: &[(&'p Expr, Value)],
    ) -> Res<Value> {
        // A read that waits shows what was printed first, as the C++
        // does: a judge answers only what it has seen.
        if !self.out.is_empty() && self.input.starved() {
            self.write_out(true);
        }
        let fail = |error: input::InputError| -> Box<Trap> {
            Box::new(Trap {
                message: error.message,
                span,
                call_stack: Vec::new(),
                case: None,
                notes: Vec::new(),
            })
        };
        Ok(match spec {
            Spec::Line => Value::Str(Rc::new(self.input.line().map_err(fail)?)),
            Spec::List(_) => unreachable!("lowering spells out a counted list read"),
            Spec::Plain(ty) => match ty {
                Ty::Int => Value::Int(self.input.int().map_err(fail)?.0),
                Ty::I128 => Value::I128(self.input.wide::<i128>("an `I128`").map_err(fail)?.into()),
                Ty::U64 => Value::U64(self.input.wide("a `U64`").map_err(fail)?),
                Ty::Float => Value::Float(self.input.float().map_err(fail)?),
                Ty::Char => Value::Char(self.input.char().map_err(fail)?),
                // A `Bool` is written `0` or `1`.
                Ty::Bool => match self.input.int().map_err(fail)? {
                    (bit @ (0 | 1), _) => Value::Bool(bit == 1),
                    (other, start) => {
                        let message = format!("{other} is not a `Bool`, which is read as 0 or 1");
                        return Err(fail(self.input.fail(start, message)));
                    }
                },
                Ty::Str => Value::Str(Rc::new(
                    self.input.token("a word").map_err(fail)?.0.to_vec(),
                )),
                Ty::Pos(d) => {
                    let (label, start) = self.input.int().map_err(fail)?;
                    let (lo, len) = self.domains[*d];
                    if label < lo || label - lo >= len {
                        let name = &self.program.domains[*d];
                        let message =
                            format!("{label} is not in `{name}`, which is {}", describe(lo, len));
                        return Err(fail(self.input.fail(start, message)));
                    }
                    Value::Int(label - lo)
                }
                Ty::Range(inner) => match &**inner {
                    Ty::Int => {
                        let (lo, _) = self.input.int().map_err(fail)?;
                        let (hi, start) = self.input.int().map_err(fail)?;
                        if lo > hi {
                            let message = format!("range end {hi} is before start {lo}");
                            return Err(fail(self.input.fail(start, message)));
                        }
                        Value::Range(lo, hi.saturating_add(1))
                    }
                    Ty::Pos(d) => {
                        let (l_label, l_start) = self.input.int().map_err(fail)?;
                        let (lo, len) = self.domains[*d];
                        if l_label < lo || l_label - lo >= len {
                            let name = &self.program.domains[*d];
                            let message = format!(
                                "{l_label} is not in `{name}`, which is {}",
                                describe(lo, len)
                            );
                            return Err(fail(self.input.fail(l_start, message)));
                        }
                        let (r_label, r_start) = self.input.int().map_err(fail)?;
                        if r_label < lo || r_label - lo >= len {
                            let name = &self.program.domains[*d];
                            let message = format!(
                                "{r_label} is not in `{name}`, which is {}",
                                describe(lo, len)
                            );
                            return Err(fail(self.input.fail(r_start, message)));
                        }
                        if l_label > r_label {
                            let message = format!("range end {r_label} is before start {l_label}");
                            return Err(fail(self.input.fail(r_start, message)));
                        }
                        Value::Range(l_label - lo, r_label - lo + 1)
                    }
                    _ => Value::Unit,
                },
                _ => Value::Unit,
            },
            Spec::Int { .. } | Spec::Str { .. } => self.read_bounded(spec, fail, given)?,
            Spec::Call(call) => self.given(call, given)?,
            Spec::Parse(text, inner) => {
                let text = match self.given(text, given)? {
                    Value::Str(bytes) => bytes.to_vec(),
                    _ => Vec::new(),
                };
                let held = std::mem::replace(&mut self.input, Input::owned(text));
                let value = self.read(inner, span, given);
                let whole = value.is_ok() && self.input.at_eof();
                self.input = held;
                match value {
                    Ok(value) if whole => Value::some(value),
                    _ => Value::Opt(OptVal::None),
                }
            }
            Spec::Fields(_, fields) => {
                let mut values = Vec::with_capacity(fields.len());
                for field in fields {
                    values.push(self.read(field, span, given)?);
                }
                Value::tuple(values)
            }
            Spec::Array { dims, elem } => {
                let dims = self.read_dims(dims, span, given)?;
                let cells = dims.iter().product::<i64>() as usize;
                let mut data = Vec::with_capacity(cells);
                for _ in 0..cells {
                    data.push(self.read(elem, span, given)?);
                }
                Value::Array(Rc::new(Array::from_values(dims, data)))
            }
            Spec::Opt(inner, Some(absent)) => {
                let absent = self.given(absent, given)?.int();
                let (value, start) = self.input.int().map_err(fail)?;
                if value == absent {
                    return Ok(Value::Opt(OptVal::None));
                }
                // The number is already read; the inner type checks it.
                self.input.rewind(start);
                Value::some(self.read(inner, span, given)?)
            }
            Spec::Opt(..) => Value::Opt(OptVal::None),
        })
    }

    /// The lengths of `dims`, with `given` as in [`Interp::read`].
    pub(crate) fn read_dims(
        &mut self,
        dims: &'p [DimSpec],
        span: Span,
        given: &[(&'p Expr, Value)],
    ) -> Res<Vec<i64>> {
        let mut out = Vec::with_capacity(dims.len());
        let mut cells: i64 = 1;
        for dim in dims {
            let len = match dim {
                DimSpec::Pos(d) => self.domains[*d].1,
                DimSpec::Bound(d) => self.domains[*d].1 + 1,
                DimSpec::Sym(d) => {
                    let n = self.domains[*d].1;
                    n * (n + 1) / 2
                }
                DimSpec::Enum(e) => self.program.enums[*e].variants.len() as i64,
                DimSpec::Int(len) | DimSpec::IntFrom(_, len) => {
                    let len = self.given(len, given)?.int();
                    if len < 0 {
                        return trap(span, format!("an array can't have length {len}"));
                    }
                    len
                }
            };
            cells = match cells.checked_mul(len).filter(|c| *c <= 1 << 32) {
                Some(cells) => cells,
                None => return trap(span, "this array is too large to allocate"),
            };
            out.push(len);
        }
        Ok(out)
    }
}
