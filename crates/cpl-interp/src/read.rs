//! Reading a value of a spec from the input, which both engines share.

use crate::observe::Observer;
use crate::value::*;
use crate::*;

/// Evaluates an expression in a spec for [`Interp::read`].
pub(crate) type Run<'r, 'p, S> = dyn FnMut(&mut S, &'p Expr) -> Res<Value> + 'r;

impl<'p, 'o, O: Observer> Interp<'p, 'o, O> {
    /// The read `e`, with `text` the string a `read<T>(from: s)` reads.
    pub(crate) fn read_v(
        &mut self,
        e: &'p Expr,
        text: Option<Value>,
        run: &mut Run<'_, 'p, Self>,
    ) -> Res<Value> {
        match &e.kind {
            // The string is the whole input, and the read gives `Some` only
            // when it succeeds and reads all of it. (R10.2.4)
            ExprKind::ReadFrom(_, body) => {
                let text = text.unwrap_or(Value::Unit).into_bytes();
                let held = std::mem::replace(&mut self.input, Input::owned(text));
                let value = self.read_v(body, None, run).ok();
                let value = value.filter(|_| self.input.at_eof());
                self.input = held;
                Ok(Value::Opt(OptVal::from_option(value)))
            }
            ExprKind::Read(spec) => self.read(spec, e.span, run),
            _ => run(self, e),
        }
    }

    /// A value of `spec` from the input, with `run` evaluating each
    /// expression in it as the read reaches it.
    pub(crate) fn read(
        &mut self,
        spec: &'p Spec,
        span: Span,
        run: &mut Run<'_, 'p, Self>,
    ) -> Res<Value> {
        // A read that waits shows what was printed first, as the C++
        // does: a judge answers only what it has seen.
        if !self.out.is_empty() && self.input.starved() {
            self.write_out(true);
        }
        let fail = |error: input::InputError| -> Box<Trap> {
            let said = cpl_diag::msgs::interp::WRITTEN.at(&[&error.message]);
            Box::new(Trap::said(span, said.with_code(error.code)))
        };
        Ok(match spec {
            Spec::Line => Value::Str(Rc::new(self.input.line().map_err(fail)?)),
            Spec::List(never) => match *never {},
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
                        let message = cpl_diag::msgs::interp::NOT_BOOL_WHICH_READ_AS.at(&[&other]);
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
                        let message = cpl_diag::msgs::interp::NOT_IN_WHICH.at(&[
                            &label,
                            &name,
                            &(describe(lo, len)),
                        ]);
                        return Err(fail(self.input.fail(start, message)));
                    }
                    Value::Int(label - lo)
                }
                Ty::Range(inner) => match &**inner {
                    Ty::Int => {
                        let (lo, _) = self.input.int().map_err(fail)?;
                        let (hi, start) = self.input.int().map_err(fail)?;
                        if lo > hi {
                            let message =
                                cpl_diag::msgs::interp::RANGE_END_BEFORE_START.at(&[&hi, &lo]);
                            return Err(fail(self.input.fail(start, message)));
                        }
                        Value::Range(lo, hi.saturating_add(1))
                    }
                    Ty::Pos(d) => {
                        let (l_label, l_start) = self.input.int().map_err(fail)?;
                        let (lo, len) = self.domains[*d];
                        if l_label < lo || l_label - lo >= len {
                            let name = &self.program.domains[*d];
                            let message = cpl_diag::msgs::interp::NOT_IN_WHICH.at(&[
                                &l_label,
                                &name,
                                &(describe(lo, len)),
                            ]);
                            return Err(fail(self.input.fail(l_start, message)));
                        }
                        let (r_label, r_start) = self.input.int().map_err(fail)?;
                        if r_label < lo || r_label - lo >= len {
                            let name = &self.program.domains[*d];
                            let message = cpl_diag::msgs::interp::NOT_IN_WHICH.at(&[
                                &r_label,
                                &name,
                                &(describe(lo, len)),
                            ]);
                            return Err(fail(self.input.fail(r_start, message)));
                        }
                        if l_label > r_label {
                            let message = cpl_diag::msgs::interp::RANGE_END_BEFORE_START
                                .at(&[&r_label, &l_label]);
                            return Err(fail(self.input.fail(r_start, message)));
                        }
                        Value::Range(l_label - lo, r_label - lo + 1)
                    }
                    _ => Value::Unit,
                },
                _ => Value::Unit,
            },
            Spec::Int { .. } | Spec::Str { .. } => self.read_bounded(spec, fail, run)?,
            Spec::Call(call) => run(self, call)?,
            Spec::Fields(_, fields) => {
                let mut values = Vec::with_capacity(fields.len());
                for field in fields {
                    values.push(self.read(field, span, run)?);
                }
                Value::tuple(values)
            }
            Spec::Array { dims, elem } => {
                let dims = self.read_dims(dims, span, run)?;
                let cells = dims.iter().product::<i64>() as usize;
                // Each cell goes straight into the packed store, never into a
                // boxed list first.
                let mut data = Cells::default();
                for i in 0..cells {
                    data.push(self.read(elem, span, run)?);
                    if i == 0 {
                        data.reserve(cells - 1);
                    }
                }
                Value::Array(Rc::new(Array {
                    dims,
                    data: Rc::new(data),
                }))
            }
            Spec::Opt(inner, Some(absent)) => {
                let absent = run(self, absent)?.int();
                let (value, start) = self.input.int().map_err(fail)?;
                if value == absent {
                    return Ok(Value::Opt(OptVal::None));
                }
                // The number is already read; the inner type checks it.
                self.input.rewind(start);
                Value::some(self.read(inner, span, run)?)
            }
            // `read<?T>()`: `None` once the input has ended. (R10.2.10)
            Spec::Opt(inner, None) => {
                if self.input.at_eof() {
                    Value::Opt(OptVal::None)
                } else {
                    Value::some(self.read(inner, span, run)?)
                }
            }
        })
    }

    /// The lengths of `dims`, with `run` as in [`Interp::read`].
    pub(crate) fn read_dims(
        &mut self,
        dims: &'p [DimSpec],
        span: Span,
        run: &mut Run<'_, 'p, Self>,
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
                    let len = run(self, len)?.int();
                    if len < 0 {
                        return trap(
                            span,
                            cpl_diag::msgs::interp::ARRAY_CAN_HAVE_LENGTH.at(&[&len]),
                        );
                    }
                    len
                }
            };
            cells = match cells.checked_mul(len).filter(|c| *c <= 1 << 32) {
                Some(cells) => cells,
                None => {
                    return trap(
                        span,
                        cpl_diag::msgs::interp::THIS_ARRAY_TOO_LARGE_TO.at(&[]),
                    )
                }
            };
            out.push(len);
        }
        Ok(out)
    }
}
