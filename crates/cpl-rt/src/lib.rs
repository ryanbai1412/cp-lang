//! The intrinsics table (decision 573, spec R12.14): every primitive a
//! library body names with `@intrinsic("id")`, with its signature, its cost,
//! the interpreter's implementation and the C++ it emits. The compiler
//! matches [`PrimOp`] values, never the names libraries give them.

pub mod cpp;

pub use cpl_prim::PrimOp;
/// The frozen old pipeline's name for [`PrimOp`]; goes with it at the switch.
pub use cpl_prim::PrimOp as Prim;

/// A scalar the interpreter hands to, and gets back from, a primitive.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Scalar {
    Int(i64),
    I128(i128),
    U64(u64),
    F64(f64),
}

impl Scalar {
    /// The 64 bits of the scalar; an integer's as C++'s `(uint64_t)` cast reads them.
    pub fn bits(self) -> u64 {
        match self {
            Scalar::Int(v) => v as u64,
            Scalar::I128(v) => v as u64,
            Scalar::U64(v) => v,
            Scalar::F64(v) => v.to_bits(),
        }
    }

    /// The number as an `F64`, as C++'s `(double)` cast reads it.
    pub fn f64(self) -> f64 {
        match self {
            Scalar::Int(v) => v as f64,
            Scalar::I128(v) => v as f64,
            Scalar::U64(v) => v as f64,
            Scalar::F64(v) => v,
        }
    }
}

/// Why a primitive traps; the interpreter words it from its registry
/// row, so this crate carries no message text.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Fault {
    /// `abs(Int.MIN)`: the result does not fit in an `Int`.
    Overflow,
    /// `abs(I128.MIN)`: the result does not fit in an `I128`.
    WideOverflow,
    /// `bit_ceil` above 2^62: the result does not fit in an `Int`.
    CeilOverflow,
    /// The rounded float, outside `Int`.
    NotInt(f64),
}

/// A primitive's result, or why it traps.
pub type Ran = Result<Scalar, Fault>;

/// The C++ of a primitive for an argument type.
pub struct Cpp {
    /// The first argument's types this form is for, `|` between them.
    pub ty: &'static str,
    /// The expression: `$0`, `$1`, ... are the arguments, each read once.
    pub code: &'static str,
    /// The runtime helper the code needs, included once per bundle.
    pub need: Option<&'static str>,
}

const fn cpp(ty: &'static str, code: &'static str, need: Option<&'static str>) -> Cpp {
    Cpp { ty, code, need }
}

const TO_INT: &str = "narrow";

/// What each primitive takes, costs, computes and emits; one row of the table per [`PrimOp`].
pub trait Row {
    fn cost(self) -> &'static str;
    fn explain(self) -> &'static str;
    fn forms(self) -> &'static [Cpp];
    fn run(self, a: Scalar) -> Ran;
    fn form(self, ty: &str) -> &'static Cpp;
    fn cpp(self, ty: &str, args: &[String]) -> String;
}

impl Row for PrimOp {
    /// The cost as `cpl explain` and docs show it (R09 form).
    fn cost(self) -> &'static str {
        "\\(O(1)\\)"
    }

    /// What `cpl explain` and the trace say the primitive does.
    fn explain(self) -> &'static str {
        match self {
            PrimOp::Popcount => "the number of 1 bits, one `popcnt`",
            PrimOp::Abs => "the absolute value; an integer traps where it has none",
            PrimOp::Sqrt => "the square root as an `F64`",
            PrimOp::Floor => "rounds down to an `Int`; traps outside `Int`",
            PrimOp::Ceil => "rounds up to an `Int`; traps outside `Int`",
            PrimOp::Round => {
                "rounds to the nearest `Int`, halves away from zero; traps outside `Int`"
            }
            PrimOp::Trunc => "rounds toward zero to an `Int`; traps outside `Int`",
            PrimOp::Ctz => "the 0 bits below the lowest 1 bit, 64 for 0; one `tzcnt` with bmi",
            PrimOp::Clz => "the 0 bits above the highest 1 bit, 64 for 0; one `lzcnt` with lzcnt",
            PrimOp::BitWidth => "the bits needed to write the word, 0 for 0",
            PrimOp::BitCeil => "the smallest power of two at least the value; traps above 2^62",
            PrimOp::Acos => "the angle in `0..=pi` whose cosine it is",
            PrimOp::Asin => "the angle in `-pi/2..=pi/2` whose sine it is",
            PrimOp::Atan2 => "the angle of the point `(x, y)` in `-pi..=pi`, from `y` and `x`",
            PrimOp::Cos => "the cosine of an angle in radians",
            PrimOp::Sin => "the sine of an angle in radians",
            PrimOp::Tan => "the tangent of an angle in radians",
            PrimOp::Hypot => "the length of `(x, y)`, without overflow in the squares",
            PrimOp::Exp => "`e` to the power of the value",
            PrimOp::Ln => "the natural logarithm",
        }
    }

    /// The C++ forms, by first argument type; the last is for any type (`*`).
    fn forms(self) -> &'static [Cpp] {
        match self {
            PrimOp::Popcount => {
                const F: &[Cpp] = &[cpp("*", "(ll)__builtin_popcountll((uint64_t)($0))", None)];
                F
            }
            PrimOp::Abs => {
                const F: &[Cpp] = &[
                    cpp("F64", "std::fabs($0)", None),
                    cpp("I128", "cpl::abs128($0)", Some("abs128")),
                    cpp("*", "std::abs((ll)($0))", None),
                ];
                F
            }
            PrimOp::Sqrt => {
                const F: &[Cpp] = &[
                    cpp("F64", "std::sqrt($0)", None),
                    cpp("*", "std::sqrt((double)($0))", None),
                ];
                F
            }
            PrimOp::Floor => {
                const F: &[Cpp] = &[cpp("*", "cpl::to_int(std::floor($0))", Some(TO_INT))];
                F
            }
            PrimOp::Ceil => {
                const F: &[Cpp] = &[cpp("*", "cpl::to_int(std::ceil($0))", Some(TO_INT))];
                F
            }
            PrimOp::Round => {
                const F: &[Cpp] = &[cpp("*", "cpl::to_int(std::round($0))", Some(TO_INT))];
                F
            }
            PrimOp::Trunc => {
                const F: &[Cpp] = &[cpp("*", "cpl::to_int($0)", Some(TO_INT))];
                F
            }
            PrimOp::Ctz => {
                const F: &[Cpp] = &[cpp("*", "cpl::ctz($0)", Some("bits"))];
                F
            }
            PrimOp::Clz => {
                const F: &[Cpp] = &[cpp("*", "cpl::clz($0)", Some("bits"))];
                F
            }
            PrimOp::BitWidth => {
                const F: &[Cpp] = &[cpp("*", "cpl::bit_width($0)", Some("bits"))];
                F
            }
            PrimOp::BitCeil => {
                const F: &[Cpp] = &[cpp("*", "cpl::bit_ceil($0)", Some("bits"))];
                F
            }
            PrimOp::Acos => {
                const F: &[Cpp] = &[cpp("*", "std::acos($0)", None)];
                F
            }
            PrimOp::Asin => {
                const F: &[Cpp] = &[cpp("*", "std::asin($0)", None)];
                F
            }
            PrimOp::Atan2 => {
                const F: &[Cpp] = &[cpp("*", "std::atan2($0, $1)", None)];
                F
            }
            PrimOp::Cos => {
                const F: &[Cpp] = &[cpp("*", "std::cos($0)", None)];
                F
            }
            PrimOp::Sin => {
                const F: &[Cpp] = &[cpp("*", "std::sin($0)", None)];
                F
            }
            PrimOp::Tan => {
                const F: &[Cpp] = &[cpp("*", "std::tan($0)", None)];
                F
            }
            PrimOp::Hypot => {
                const F: &[Cpp] = &[cpp("*", "std::hypot($0, $1)", None)];
                F
            }
            PrimOp::Exp => {
                const F: &[Cpp] = &[cpp("*", "std::exp($0)", None)];
                F
            }
            PrimOp::Ln => {
                const F: &[Cpp] = &[cpp("*", "std::log($0)", None)];
                F
            }
        }
    }

    /// Runs a primitive on its first argument, as the interpreter does. A
    /// float row (`sqrt`, `sin`, `atan2`, ..) gives its argument as the `F64`
    /// it reads as: the interpreter applies the fn, C++ calls `std::` itself,
    /// and the row's domain is its library's to check.
    fn run(self, a: Scalar) -> Ran {
        let w = a.bits();
        let x = a.f64();
        let n = match self {
            PrimOp::Popcount => w.count_ones().into(),
            PrimOp::Ctz => w.trailing_zeros().into(),
            PrimOp::Clz => w.leading_zeros().into(),
            PrimOp::BitWidth => (64 - w.leading_zeros()).into(),
            PrimOp::Floor | PrimOp::Ceil | PrimOp::Round | PrimOp::Trunc => {
                let y = match self {
                    PrimOp::Floor => x.floor(),
                    PrimOp::Ceil => x.ceil(),
                    PrimOp::Round => x.round(),
                    _ => x.trunc(),
                };
                if y.is_nan() || y.abs() >= 9223372036854775808.0 {
                    return Err(Fault::NotInt(y));
                }
                y as i64
            }
            PrimOp::BitCeil => match w as i64 {
                v if v > 1 << 62 => return Err(Fault::CeilOverflow),
                v if v <= 1 => 1,
                _ => 1 << (64 - (w - 1).leading_zeros()),
            },
            PrimOp::Abs => match a {
                Scalar::Int(v) => v.checked_abs().ok_or(Fault::Overflow)?,
                Scalar::I128(v) => {
                    return v.checked_abs().map(Scalar::I128).ok_or(Fault::WideOverflow)
                }
                Scalar::F64(v) => return Ok(Scalar::F64(v.abs())),
                other => return Ok(other),
            },
            _ => return Ok(Scalar::F64(x)),
        };
        Ok(Scalar::Int(n))
    }

    /// The C++ form for a first argument of type `ty`.
    fn form(self, ty: &str) -> &'static Cpp {
        let forms = self.forms();
        (forms.iter())
            .find(|c| c.ty == "*" || c.ty.split('|').any(|t| t == ty))
            .unwrap_or(&forms[forms.len() - 1])
    }

    /// The C++ of a call whose first argument is a `ty`, given each argument's C++.
    fn cpp(self, ty: &str, args: &[String]) -> String {
        let mut out = String::new();
        let mut rest = self.form(ty).code;
        while let Some(at) = rest.find('$') {
            out.push_str(&rest[..at]);
            let digits = rest[at + 1..]
                .bytes()
                .take_while(u8::is_ascii_digit)
                .count();
            let i: usize = rest[at + 1..at + 1 + digits].parse().unwrap_or(0);
            out.push_str(args.get(i).map_or("", String::as_str));
            rest = &rest[at + 1 + digits..];
        }
        out.push_str(rest);
        out
    }
}

/// A primitive's parameter types and result type, the signature its `@intrinsic` fn must have.
pub fn sig(p: PrimOp) -> (&'static [&'static str], &'static str) {
    (p.params(), p.result())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Growing the table needs a decision entry (decision 573): change this
    /// count in the commit that adds it.
    #[test]
    fn table_count() {
        assert_eq!(PrimOp::ALL.len(), 20);
    }

    #[test]
    fn each_entry_sits_at_its_number_and_reads_each_argument_once() {
        for (i, e) in PrimOp::ALL.into_iter().enumerate() {
            assert_eq!(e as usize, i, "{}", e.id());
            assert_eq!(PrimOp::from_id(e.id()), Some(e));
            for form in e.forms() {
                for k in 0..e.params().len() {
                    assert_eq!(
                        form.code.matches(&format!("${k}")).count(),
                        1,
                        "{} ${k}",
                        e.id()
                    );
                }
            }
            assert_eq!(e.forms().last().map(|c| c.ty), Some("*"), "{}", e.id());
        }
    }

    #[test]
    fn popcount_counts_the_bits_cpp_counts() {
        let p = PrimOp::Popcount;
        assert_eq!(p.run(Scalar::Int(-1)), Ok(Scalar::Int(64)));
        assert_eq!(p.run(Scalar::U64(0b1011)), Ok(Scalar::Int(3)));
        assert_eq!(
            p.cpp("Int", &["x".into()]),
            "(ll)__builtin_popcountll((uint64_t)(x))"
        );
    }

    #[test]
    fn bits_count_as_cpp_counts() {
        assert_eq!(PrimOp::Ctz.run(Scalar::Int(0)), Ok(Scalar::Int(64)));
        assert_eq!(PrimOp::Ctz.run(Scalar::Int(8)), Ok(Scalar::Int(3)));
        assert_eq!(PrimOp::Clz.run(Scalar::Int(-1)), Ok(Scalar::Int(0)));
        assert_eq!(PrimOp::Clz.run(Scalar::U64(1)), Ok(Scalar::Int(63)));
        assert_eq!(PrimOp::BitWidth.run(Scalar::Int(5)), Ok(Scalar::Int(3)));
        assert_eq!(PrimOp::BitCeil.run(Scalar::Int(5)), Ok(Scalar::Int(8)));
        assert_eq!(PrimOp::BitCeil.run(Scalar::Int(-3)), Ok(Scalar::Int(1)));
        assert_eq!(
            PrimOp::BitCeil.run(Scalar::Int(1 << 62)),
            Ok(Scalar::Int(1 << 62))
        );
        assert_eq!(
            PrimOp::BitCeil.run(Scalar::Int((1 << 62) + 1)),
            Err(Fault::CeilOverflow)
        );
        assert_eq!(PrimOp::Ctz.cpp("U64", &["x".into()]), "cpl::ctz(x)");
    }

    #[test]
    fn a_float_row_gives_its_argument_as_an_f64() {
        for p in [PrimOp::Sqrt, PrimOp::Ln, PrimOp::Acos, PrimOp::Atan2] {
            assert_eq!(p.run(Scalar::Int(-3)), Ok(Scalar::F64(-3.0)), "{p:?}");
        }
        assert_eq!(PrimOp::Exp.run(Scalar::F64(0.5)), Ok(Scalar::F64(0.5)));
    }

    #[test]
    fn math_traps_where_cpp_fails() {
        assert_eq!(PrimOp::Abs.run(Scalar::Int(i64::MIN)), Err(Fault::Overflow));
        let edge = 9223372036854775808.0;
        assert_eq!(
            PrimOp::Trunc.run(Scalar::F64(edge)),
            Err(Fault::NotInt(edge))
        );
        assert_eq!(
            PrimOp::Trunc.run(Scalar::F64(-edge)),
            Err(Fault::NotInt(-edge))
        );
        let below = -9223372036854774784.0;
        assert_eq!(
            PrimOp::Trunc.run(Scalar::F64(below)),
            Ok(Scalar::Int(below as i64))
        );
        assert_eq!(
            PrimOp::Abs.run(Scalar::I128(i128::MIN)),
            Err(Fault::WideOverflow)
        );
        assert_eq!(PrimOp::Abs.run(Scalar::I128(-5)), Ok(Scalar::I128(5)));
        assert_eq!(PrimOp::Abs.cpp("F64", &["x".into()]), "std::fabs(x)");
        assert_eq!(PrimOp::Abs.cpp("Int", &["x".into()]), "std::abs((ll)(x))");
        assert_eq!(PrimOp::Round.run(Scalar::F64(-2.5)), Ok(Scalar::Int(-3)));
        assert_eq!(PrimOp::Trunc.run(Scalar::F64(-2.5)), Ok(Scalar::Int(-2)));
        assert!(matches!(
            PrimOp::Floor.run(Scalar::F64(f64::NAN)),
            Err(Fault::NotInt(_))
        ));
    }
}
