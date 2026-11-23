//! Arithmetic and shifts: operators, widths and the checks each build keeps.

use super::*;

impl<'p> Emitter<'p> {
    /// How far a shift goes, held to `0..width` in every build: outside
    /// it C++ gives garbage. A literal inside the range needs no test.
    pub(super) fn shift_amount(&mut self, b: &Code, width: i64) -> Code {
        if self.shift_fits(&b.text, width) {
            return Code::new(b.text.clone(), b.prec);
        }
        self.need("shamt");
        Code::primary(format!("cpl::shamt({}, {width})", b.text))
    }

    pub(super) fn binary(&mut self, e: &Expr, op: BinOp, l: &Expr, r: &Expr) -> Code {
        // Two numbers written out whose result is past an `Int`
        // (`Int.MAX + 1`, the end of `..=Int.MAX`): `cpl run` stops there,
        // and so does this, instead of an overflow the compiler may use.
        if let (Ty::Int, ExprKind::Int(a), ExprKind::Int(b)) = (&e.ty, &l.kind, &r.kind) {
            let past = match op {
                BinOp::Add => a.checked_add(*b),
                BinOp::Sub => a.checked_sub(*b),
                BinOp::Mul => a.checked_mul(*b),
                _ => Some(0),
            }
            .is_none_or(|v| v == i64::MIN);
            if past {
                self.need("fail");
                return Code::primary(
                    "(cpl::fail(\"integer overflow: the result does not fit in an `Int` (64 bits)\"), 0LL)",
                );
            }
        }
        match op {
            BinOp::And => {
                let (a, b) = (self.expr(l), self.expr(r));
                infix(&a, "&&", &b, AND)
            }
            BinOp::Or => {
                let (a, b) = (self.expr(l), self.expr(r));
                // `a || b && c` is right but reads wrong: keep the parentheses.
                Code::new(
                    format!(
                        "{} || {}",
                        a.at(if a.prec == OR { OR } else { AND + 1 }),
                        b.at(AND + 1)
                    ),
                    OR,
                )
            }
            BinOp::Coalesce => {
                if self.read_in_place.remove(&(e as *const Expr as usize)) {
                    if let ExprKind::Builtin(Builtin::MapGet, got) = &l.kind {
                        self.need("map_or");
                        let (m, k) = (self.expr(&got[0]).text, self.expr(&got[1]).text);
                        let d = self.expr(r).text;
                        return Code::primary(format!("cpl::map_or({m}, {k}, {d})"));
                    }
                }
                // An optional default keeps the result optional: `a` itself.
                // A plain default gives what is inside `a`, which is still
                // an optional when `a` is an optional of optionals.
                let keeps_opt = l.ty == e.ty;
                if walk::path_root(l).is_some() {
                    let a = self.expr(l).at(PRIMARY);
                    let b = self.expr(r);
                    let value = if keeps_opt {
                        a.clone()
                    } else {
                        format!("*{a}")
                    };
                    return Code::new(format!("{a}.has_value() ? {value} : {}", b.at(COND)), COND);
                }
                let ret = self.ty(&e.ty);
                self.lambda(&ret, |this, out| {
                    let a = this.expr(l).text;
                    let name = this.temp();
                    out.push_str(&format!(
                        "{}{} {name} = {a};\n",
                        pad(this.indent),
                        this.ty(&l.ty)
                    ));
                    let b = this.expr(r);
                    let value = if keeps_opt {
                        name.clone()
                    } else {
                        format!("*{name}")
                    };
                    out.push_str(&format!(
                        "{}return {name}.has_value() ? {value} : {};\n",
                        pad(this.indent),
                        b.at(COND)
                    ));
                })
            }
            _ => self.sequence(&[l, r], &e.ty, |this, codes| {
                this.arith(op, l, r, &codes[0], &codes[1])
            }),
        }
    }

    /// The runtime fn for `+`, `-` or `*` on `Int`s or `I128`s not proved
    /// in range: it traps with the operands in a checked build (`-DLOCAL`)
    /// and wraps in a judge build, never overflowing C++'s signed integers
    /// (R09.3.4).
    pub(crate) fn overflow_checked(&mut self, ty: &Ty, op: BinOp) -> String {
        self.need("iarith");
        let name = match op {
            BinOp::Add => "add",
            BinOp::Sub => "sub",
            _ => "mul",
        };
        let wide = if *ty == Ty::I128 { "128" } else { "" };
        format!("cpl::i{name}{wide}")
    }

    /// `l op r` for operands already emitted as `a` and `b`.
    pub fn arith(&mut self, op: BinOp, l: &Expr, r: &Expr, a: &Code, b: &Code) -> Code {
        let both_narrow = a.narrow && b.narrow;
        match (&l.ty, op) {
            (Ty::Int, BinOp::Add | BinOp::Sub | BinOp::Mul)
                if !both_narrow && !self.facts.fits(self.fn_id, op, l, r) =>
            {
                let f = self.overflow_checked(&l.ty, op);
                Code::primary(format!("{f}({}, {})", a.text, b.text))
            }
            (Ty::Int, BinOp::Add | BinOp::Sub | BinOp::Mul) => {
                let (sym, prec) = match op {
                    BinOp::Add => ("+", ADD),
                    BinOp::Sub => ("-", ADD),
                    _ => ("*", MUL),
                };
                let a = if both_narrow { widen(a) } else { a.clone() };
                infix(&a, sym, b, prec)
            }
            (Ty::I128, BinOp::Add | BinOp::Sub | BinOp::Mul) => {
                let f = self.overflow_checked(&l.ty, op);
                Code::primary(format!("{f}({}, {})", a.text, b.text))
            }
            (Ty::Int, BinOp::Div | BinOp::Rem) => {
                self.divrem(op == BinOp::Div, l, r, a, b, both_narrow)
            }
            (Ty::Int, BinOp::Pow) => {
                self.need("ipow");
                Code::primary(format!("cpl::ipow({}, {})", a.text, b.text))
            }
            (Ty::Int, BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor) => {
                let sym = match op {
                    BinOp::BitAnd => "&",
                    BinOp::BitOr => "|",
                    _ => "^",
                };
                let mut code = Code::new(format!("{} {sym} {}", a.at(MUL), b.at(MUL)), BIT);
                code.narrow = both_narrow;
                code
            }
            (Ty::Int, BinOp::Shl) if !self.facts.fits(self.fn_id, op, l, r) => {
                let b = self.shift_amount(b, 64);
                self.need("ishl");
                Code::primary(format!("cpl::ishl({}, {})", a.text, b.text))
            }
            (Ty::Int, BinOp::Shl) => {
                let b = self.shift_amount(b, 64);
                Code::new(format!("{} << {}", widen(a).at(UNARY), b.at(UNARY)), SHIFT)
            }
            (Ty::Int, BinOp::Shr) => {
                let b = self.shift_amount(b, 64);
                let mut code = Code::new(format!("{} >> {}", a.at(UNARY), b.at(UNARY)), SHIFT);
                code.narrow = a.narrow;
                code
            }
            (Ty::I128, BinOp::Div | BinOp::Rem) => {
                let helper = if op == BinOp::Div {
                    "ediv128"
                } else {
                    "emod128"
                };
                self.need(helper);
                Code::primary(format!("cpl::{helper}({}, {})", a.text, b.text))
            }
            (Ty::U64, BinOp::Div | BinOp::Rem) => {
                let helper = if op == BinOp::Div { "udiv" } else { "umod" };
                self.need("udiv");
                Code::primary(format!("cpl::{helper}({}, {})", a.text, b.text))
            }
            (Ty::I128, BinOp::Shl) => {
                self.need("shl128");
                Code::primary(format!("cpl::shl128({}, {})", a.text, b.text))
            }
            (Ty::I128 | Ty::U64, BinOp::Pow) => {
                self.need("wpow");
                Code::primary(format!("cpl::wpow({}, {})", a.text, b.text))
            }
            (Ty::I128 | Ty::U64, BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor) => {
                let sym = match op {
                    BinOp::BitAnd => "&",
                    BinOp::BitOr => "|",
                    _ => "^",
                };
                Code::new(format!("{} {sym} {}", a.at(MUL), b.at(MUL)), BIT)
            }
            (Ty::I128 | Ty::U64, BinOp::Shl | BinOp::Shr) => {
                let sym = if op == BinOp::Shl { "<<" } else { ">>" };
                let b = self.shift_amount(b, if l.ty == Ty::I128 { 128 } else { 64 });
                Code::new(format!("{} {sym} {}", a.at(UNARY), b.at(UNARY)), SHIFT)
            }
            (Ty::Float, BinOp::Pow) => Code::primary(format!("std::pow({}, {})", a.text, b.text)),
            (Ty::Bool, BinOp::BitAnd) => {
                Code::primary(format!("bool({} & {})", a.at(BIT + 1), b.at(BIT + 1)))
            }
            (Ty::Bool, BinOp::BitOr) => {
                Code::primary(format!("bool({} | {})", a.at(BIT + 1), b.at(BIT + 1)))
            }
            (Ty::Bool, BinOp::BitXor) => Code::new(format!("{} != {}", a.at(REL), b.at(REL)), EQ),
            _ => {
                let (sym, prec) = match op {
                    BinOp::Sub => ("-", ADD),
                    BinOp::Mul => ("*", MUL),
                    BinOp::Div => ("/", MUL),
                    BinOp::Rem => ("%", MUL),
                    _ => ("+", ADD),
                };
                infix(a, sym, b, prec)
            }
        }
    }
}
