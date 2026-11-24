//! Arithmetic and shifts: operators, widths and the checks each build keeps.

use super::*;
use crate::span::Reduce;

impl<'p> Emitter<'p> {
    /// How far a shift goes, held to `0..width` in every build: outside
    /// it C++ gives garbage. An amount proven inside the range needs no test.
    pub(super) fn shift_amount(&mut self, b: &Code, by: &Expr, width: i64) -> Code {
        if self.shift_fits(&b.text, width) || self.facts.within(self.facts_home(), by, width) {
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
                // A map's entry is pointed at, so only a found value is copied, once.
                let entry = match &l.kind {
                    ExprKind::Builtin(Builtin::MapGet, got) if !keeps_opt => Some(got),
                    _ => None,
                };
                self.lambda(&ret, |this, out| {
                    let held = match entry {
                        Some(got) => {
                            this.need("map_lend");
                            let (m, k) = (this.expr(&got[0]).text, this.expr(&got[1]).text);
                            format!("const auto {{}} = cpl::map_lend({m}, {k});")
                        }
                        None => format!("{} {{}} = {};", this.ty(&l.ty), this.expr(l).text),
                    };
                    let name = this.temp();
                    out.push_str(&format!(
                        "{}{}\n",
                        pad(this.indent),
                        held.replacen("{}", &name, 1)
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
            BinOp::Rem if e.ty == Ty::Int => match self.facts.reduce(self.facts_home(), l, r) {
                Some(how) => self.reduced_rem(how, l, r),
                None => self.sequence(&[l, r], &e.ty, |this, codes| {
                    this.arith(op, l, r, &codes[0], &codes[1])
                }),
            },
            _ => self.sequence(&[l, r], &e.ty, |this, codes| {
                this.arith(op, l, r, &codes[0], &codes[1])
            }),
        }
    }

    /// Whether the ordinal `ord` is proved in `0..d.count`: `d.wrap(x)` is.
    pub(crate) fn ord_inside(&self, d: usize, ord: &Expr) -> bool {
        let len = Expr {
            kind: ExprKind::DomainLen(d),
            ty: Ty::Int,
            span: ord.span,
        };
        self.facts.counts_below(self.facts_home(), ord, &len)
            || self.facts.reduce(self.facts_home(), ord, &len) == Some(Reduce::Keep)
    }

    /// `l % r` where `l`'s bounds against `r > 0` leave at most one lap
    /// (`span::reduce`): no division, at most one compare.
    fn reduced_rem(&mut self, how: Reduce, l: &Expr, r: &Expr) -> Code {
        let pure = crate::stmt::same_path(r, r);
        match how {
            Reduce::Keep if pure => return self.expr(l),
            // `(t + n) % n` with `t` in `[0, n)`: `t`.
            Reduce::SubOnce if pure => {
                if let ExprKind::Binary(BinOp::Add, t, m) = &l.kind {
                    if crate::span::same_count(m, r) {
                        return self.expr(t);
                    }
                }
            }
            _ => {}
        }
        self.sequence(&[l, r], &Ty::Int, |this, codes| {
            let (a, b) = (&codes[0], &codes[1]);
            match how {
                Reduce::Keep => a.clone(),
                Reduce::SubOnce => infix(a, "-", b, ADD),
                Reduce::SubIf | Reduce::AddIf => {
                    let helper = if how == Reduce::SubIf {
                        "sub_if"
                    } else {
                        "add_if"
                    };
                    this.need("lap");
                    Code::primary(format!("cpl::{helper}({}, {})", a.text, b.text))
                }
            }
        })
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
        if l.ty == Ty::Int && matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul) {
            let checked = !both_narrow && !self.facts.fits(self.facts_home(), op, l, r);
            self.note_arith(l.span, r.span, checked);
        }
        match (&l.ty, op) {
            (Ty::Int, BinOp::Add | BinOp::Sub | BinOp::Mul)
                if !both_narrow && !self.facts.fits(self.facts_home(), op, l, r) =>
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
            // Two `Int`s widened to `I128` multiply within 2**126.
            (Ty::I128, BinOp::Mul) if widened_int(l) && widened_int(r) => infix(a, "*", b, MUL),
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
            (Ty::Int, BinOp::Shl) if !self.facts.fits(self.facts_home(), op, l, r) => {
                let b = self.shift_amount(b, r, 64);
                self.need("ishl");
                Code::primary(format!("cpl::ishl({}, {})", a.text, b.text))
            }
            (Ty::Int, BinOp::Shl) => {
                let b = self.shift_amount(b, r, 64);
                Code::new(format!("{} << {}", widen(a).at(UNARY), b.at(UNARY)), SHIFT)
            }
            (Ty::Int, BinOp::Shr) => {
                let b = self.shift_amount(b, r, 64);
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
                let b = self.shift_amount(b, r, if l.ty == Ty::I128 { 128 } else { 64 });
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

fn widened_int(e: &Expr) -> bool {
    matches!(&e.kind, cpl_ir::ExprKind::Cast(cpl_ir::Cast::IntToI128, _))
}
