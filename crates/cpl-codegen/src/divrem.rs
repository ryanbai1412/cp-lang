//! Euclidean `/` and `%` on `Int` as C++: C++'s own operator where no
//! operand is negative, a shift or mask for a power-of-two divisor (floor
//! and Euclidean agree when the divisor is positive), else a runtime helper.

use crate::emit::{Code, Emitter, BIT, MUL, SHIFT, UNARY};
use crate::expr::infix;
use crate::ranges::Sign;
use cpl_ir::{Expr, ExprKind};

/// `k`'s exponent when `k` is a positive power of two.
fn pow2(e: &Expr) -> Option<u32> {
    match e.kind {
        ExprKind::Int(k) if k > 0 && k & (k - 1) == 0 => Some(k.trailing_zeros()),
        _ => None,
    }
}

impl<'p> Emitter<'p> {
    pub(crate) fn divrem(
        &mut self,
        div: bool,
        l: &Expr,
        r: &Expr,
        a: &Code,
        b: &Code,
        both_narrow: bool,
    ) -> Code {
        let dividend = self.facts.sign(self.facts_home(), l);
        let divisor = self.facts.sign(self.facts_home(), r);
        if dividend != Sign::Unknown && divisor == Sign::Positive {
            // Nothing is negative: C++'s own operator agrees, and in 32
            // bits when both fit, where division is several times faster.
            let sym = if div { "/" } else { "%" };
            if !both_narrow
                && !matches!(r.kind, ExprKind::Int(_))
                && self.facts.fits_u32(self.facts_home(), l)
                && self.facts.fits_u32(self.facts_home(), r)
            {
                let int = self.ty(&cpl_ir::Ty::Int);
                return Code::primary(format!(
                    "{int}(uint32_t({}) {sym} uint32_t({}))",
                    a.text, b.text
                ));
            }
            let mut code = infix(a, sym, b, MUL);
            code.narrow = both_narrow;
            return code;
        }
        if let Some(shift) = pow2(r) {
            return match (div, shift) {
                (true, 0) => a.clone(),
                (false, 0) => Code::primary("0"),
                (true, _) => Code {
                    narrow: a.narrow,
                    ..Code::new(format!("{} >> {shift}", a.at(UNARY)), SHIFT)
                },
                (false, _) => Code {
                    narrow: both_narrow,
                    ..Code::new(format!("{} & {}", a.at(MUL), (1i64 << shift) - 1), BIT)
                },
            };
        }
        let helper = match (divisor == Sign::Positive, div) {
            (true, true) => "divp",
            (true, false) => "modp",
            (false, true) => "ediv",
            (false, false) => "emod",
        };
        self.need(helper);
        Code::primary(format!("cpl::{helper}({}, {})", a.text, b.text))
    }

    /// `p /= k` or `p %= k` for a power-of-two `k`, written as a shift or mask.
    pub(crate) fn pow2_assign(
        &mut self,
        p: &str,
        div: bool,
        value: &Expr,
        out: &mut String,
    ) -> bool {
        let Some(shift) = pow2(value) else {
            return false;
        };
        match (div, shift) {
            (true, 0) => {}
            (true, _) => self.line(out, &format!("{p} >>= {shift};")),
            (false, 0) => self.line(out, &format!("{p} = 0;")),
            (false, _) => self.line(out, &format!("{p} &= {};", (1i64 << shift) - 1)),
        }
        true
    }
}
