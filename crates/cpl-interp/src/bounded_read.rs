//! Reads checked against bounds the type gives: an `Int` in a range, and a
//! `Str(len: …) of "…"` (R02.3.2).

use super::*;

impl<'p, 'o, O: Observer> Interp<'p, 'o, O> {
    pub(crate) fn read_bounded(
        &mut self,
        spec: &'p Spec,
        fail: impl Fn(input::InputError) -> Box<Trap>,
        given: &[(&'p Expr, Value)],
    ) -> Res<Value> {
        Ok(match spec {
            Spec::Int { lo, hi, text } => {
                let (lo, hi) = (
                    self.bound(lo, cpl_ir::INT_MIN, given)?,
                    self.bound(hi, i64::MAX, given)?,
                );
                Value::Int(self.input.int_in(lo, hi, text).map_err(fail)?)
            }
            Spec::Str {
                lo,
                hi,
                alphabet,
                text,
            } => {
                let len = (self.bound(lo, 0, given)?, self.bound(hi, i64::MAX, given)?);
                let word = self.input.word_in(len, alphabet.as_deref(), text);
                Value::Str(Rc::new(word.map_err(fail)?))
            }
            _ => unreachable!("only a bounded `Int` or `Str` is read here"),
        })
    }

    fn bound(
        &mut self,
        bound: &'p Option<Box<Expr>>,
        or: i64,
        given: &[(&'p Expr, Value)],
    ) -> Res<i64> {
        Ok(match bound {
            Some(bound) => self.given(bound, given)?.int(),
            None => or,
        })
    }
}
