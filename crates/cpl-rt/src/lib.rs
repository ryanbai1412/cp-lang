//! The intrinsics table (decision 573, spec R12.14): every primitive a
//! library body names with `@intrinsic("id")`, with its signature, its cost,
//! the interpreter's implementation and the C++ it emits. The compiler
//! matches [`Prim`] values, never the names libraries give them.

/// One primitive of the table. Its index in [`Prim::ALL`] is its stable number.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum Prim {
    Popcount,
    Abs,
    Sqrt,
    Floor,
    Ceil,
    Round,
    Trunc,
    Ctz,
    Clz,
    BitWidth,
    BitCeil,
}

/// A type std declares with `struct T<..> = @intrinsic("id")`: storage the
/// compiler supplies (R12.14.6).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Store {
    HashMap,
    HashSet,
}

impl Store {
    /// Each type's name, as `@intrinsic("...")` writes it, at its stable number.
    pub const IDS: [&'static str; 2] = ["hash_map", "hash_set"];

    /// The name `@intrinsic("...")` writes.
    pub fn id(self) -> &'static str {
        Store::IDS[self as usize]
    }
}

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
    /// The argument below zero (`sqrt`).
    Negative(f64),
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

impl Prim {
    /// The table: each primitive, at its stable number.
    pub const ALL: [Prim; 11] = [
        Prim::Popcount,
        Prim::Abs,
        Prim::Sqrt,
        Prim::Floor,
        Prim::Ceil,
        Prim::Round,
        Prim::Trunc,
        Prim::Ctz,
        Prim::Clz,
        Prim::BitWidth,
        Prim::BitCeil,
    ];

    /// Each primitive's name, as `@intrinsic("...")` writes it.
    pub const IDS: [&'static str; 11] = [
        "popcount",
        "abs",
        "sqrt",
        "floor",
        "ceil",
        "round",
        "trunc",
        "ctz",
        "clz",
        "bit_width",
        "bit_ceil",
    ];

    /// The primitive `@intrinsic(id)` names.
    pub fn from_id(id: &str) -> Option<Prim> {
        let at = Prim::IDS.iter().position(|i| *i == id)?;
        Prim::ALL.get(at).copied()
    }

    /// The name `@intrinsic("...")` writes.
    pub fn id(self) -> &'static str {
        Prim::IDS[self as usize]
    }

    /// The parameter types, `|` between the types one accepts.
    pub fn params(self) -> &'static [&'static str] {
        const PARAMS: [&[&str]; 11] = [
            &["Int|U64"],
            &["Int|I128|F64"],
            &["Int|F64"],
            &["F64"],
            &["F64"],
            &["F64"],
            &["F64"],
            &["Int|U64"],
            &["Int|U64"],
            &["Int|U64"],
            &["Int"],
        ];
        PARAMS[self as usize]
    }

    /// The result type, `|` between the types it may be.
    pub fn result(self) -> &'static str {
        const RESULTS: [&str; 11] = [
            "Int",
            "Int|I128|F64",
            "F64",
            "Int",
            "Int",
            "Int",
            "Int",
            "Int",
            "Int",
            "Int",
            "Int",
        ];
        RESULTS[self as usize]
    }

    /// The cost as `cpl explain` and docs show it (R09 form).
    pub fn cost(self) -> &'static str {
        "\\(O(1)\\)"
    }

    /// What `cpl explain` and the trace say the primitive does.
    pub fn explain(self) -> &'static str {
        match self {
            Prim::Popcount => "the number of 1 bits, one `popcnt`",
            Prim::Abs => "the absolute value; an integer traps where it has none",
            Prim::Sqrt => "the square root as an `F64`; traps below zero",
            Prim::Floor => "rounds down to an `Int`; traps outside `Int`",
            Prim::Ceil => "rounds up to an `Int`; traps outside `Int`",
            Prim::Round => {
                "rounds to the nearest `Int`, halves away from zero; traps outside `Int`"
            }
            Prim::Trunc => "rounds toward zero to an `Int`; traps outside `Int`",
            Prim::Ctz => "the 0 bits below the lowest 1 bit, 64 for 0; one `tzcnt` with bmi",
            Prim::Clz => "the 0 bits above the highest 1 bit, 64 for 0; one `lzcnt` with lzcnt",
            Prim::BitWidth => "the bits needed to write the word, 0 for 0",
            Prim::BitCeil => "the smallest power of two at least the value; traps above 2^62",
        }
    }

    /// The C++ forms, by first argument type; the last is for any type (`*`).
    pub fn forms(self) -> &'static [Cpp] {
        match self {
            Prim::Popcount => {
                const F: &[Cpp] = &[cpp("*", "(ll)__builtin_popcountll((uint64_t)($0))", None)];
                F
            }
            Prim::Abs => {
                const F: &[Cpp] = &[
                    cpp("F64", "std::fabs($0)", None),
                    cpp("I128", "cpl::abs128($0)", Some("abs128")),
                    cpp("*", "std::abs((ll)($0))", None),
                ];
                F
            }
            Prim::Sqrt => {
                const F: &[Cpp] = &[
                    cpp("F64", "cpl::fsqrt($0)", Some("fsqrt")),
                    cpp("*", "cpl::fsqrt((double)($0))", Some("fsqrt")),
                ];
                F
            }
            Prim::Floor => {
                const F: &[Cpp] = &[cpp("*", "cpl::to_int(std::floor($0))", Some(TO_INT))];
                F
            }
            Prim::Ceil => {
                const F: &[Cpp] = &[cpp("*", "cpl::to_int(std::ceil($0))", Some(TO_INT))];
                F
            }
            Prim::Round => {
                const F: &[Cpp] = &[cpp("*", "cpl::to_int(std::round($0))", Some(TO_INT))];
                F
            }
            Prim::Trunc => {
                const F: &[Cpp] = &[cpp("*", "cpl::to_int($0)", Some(TO_INT))];
                F
            }
            Prim::Ctz => {
                const F: &[Cpp] = &[cpp("*", "cpl::ctz($0)", Some("bits"))];
                F
            }
            Prim::Clz => {
                const F: &[Cpp] = &[cpp("*", "cpl::clz($0)", Some("bits"))];
                F
            }
            Prim::BitWidth => {
                const F: &[Cpp] = &[cpp("*", "cpl::bit_width($0)", Some("bits"))];
                F
            }
            Prim::BitCeil => {
                const F: &[Cpp] = &[cpp("*", "cpl::bit_ceil($0)", Some("bits"))];
                F
            }
        }
    }

    /// Runs a primitive of one scalar, as the interpreter does.
    pub fn run(self, a: Scalar) -> Ran {
        let w = a.bits();
        let x = a.f64();
        let n = match self {
            Prim::Popcount => w.count_ones().into(),
            Prim::Ctz => w.trailing_zeros().into(),
            Prim::Clz => w.leading_zeros().into(),
            Prim::BitWidth => (64 - w.leading_zeros()).into(),
            Prim::Floor | Prim::Ceil | Prim::Round | Prim::Trunc => {
                let y = match self {
                    Prim::Floor => x.floor(),
                    Prim::Ceil => x.ceil(),
                    Prim::Round => x.round(),
                    _ => x.trunc(),
                };
                if y.is_nan() || y.abs() >= 9223372036854775808.0 {
                    return Err(Fault::NotInt(y));
                }
                y as i64
            }
            Prim::Sqrt if x < 0.0 => return Err(Fault::Negative(x)),
            Prim::Sqrt => return Ok(Scalar::F64(x.sqrt())),
            Prim::BitCeil => match w as i64 {
                v if v > 1 << 62 => return Err(Fault::CeilOverflow),
                v if v <= 1 => 1,
                _ => 1 << (64 - (w - 1).leading_zeros()),
            },
            Prim::Abs => match a {
                Scalar::Int(v) => v.checked_abs().ok_or(Fault::Overflow)?,
                Scalar::I128(v) => {
                    return v.checked_abs().map(Scalar::I128).ok_or(Fault::WideOverflow)
                }
                Scalar::F64(v) => return Ok(Scalar::F64(v.abs())),
                other => return Ok(other),
            },
        };
        Ok(Scalar::Int(n))
    }

    /// The C++ form for a first argument of type `ty`.
    pub fn form(self, ty: &str) -> &'static Cpp {
        let forms = self.forms();
        (forms.iter())
            .find(|c| c.ty == "*" || c.ty.split('|').any(|t| t == ty))
            .unwrap_or(&forms[forms.len() - 1])
    }

    /// The C++ of a call whose first argument is a `ty`, given each argument's C++.
    pub fn cpp(self, ty: &str, args: &[String]) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Growing the table needs a decision entry (decision 573): change this
    /// count in the commit that adds it.
    #[test]
    fn table_count() {
        assert_eq!(Prim::ALL.len(), 11);
    }

    #[test]
    fn each_entry_sits_at_its_number_and_reads_each_argument_once() {
        for (i, e) in Prim::ALL.into_iter().enumerate() {
            assert_eq!(e as usize, i, "{}", e.id());
            assert_eq!(Prim::from_id(e.id()), Some(e));
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
        let p = Prim::Popcount;
        assert_eq!(p.run(Scalar::Int(-1)), Ok(Scalar::Int(64)));
        assert_eq!(p.run(Scalar::U64(0b1011)), Ok(Scalar::Int(3)));
        assert_eq!(
            p.cpp("Int", &["x".into()]),
            "(ll)__builtin_popcountll((uint64_t)(x))"
        );
    }

    #[test]
    fn bits_count_as_cpp_counts() {
        assert_eq!(Prim::Ctz.run(Scalar::Int(0)), Ok(Scalar::Int(64)));
        assert_eq!(Prim::Ctz.run(Scalar::Int(8)), Ok(Scalar::Int(3)));
        assert_eq!(Prim::Clz.run(Scalar::Int(-1)), Ok(Scalar::Int(0)));
        assert_eq!(Prim::Clz.run(Scalar::U64(1)), Ok(Scalar::Int(63)));
        assert_eq!(Prim::BitWidth.run(Scalar::Int(5)), Ok(Scalar::Int(3)));
        assert_eq!(Prim::BitCeil.run(Scalar::Int(5)), Ok(Scalar::Int(8)));
        assert_eq!(Prim::BitCeil.run(Scalar::Int(-3)), Ok(Scalar::Int(1)));
        assert_eq!(
            Prim::BitCeil.run(Scalar::Int(1 << 62)),
            Ok(Scalar::Int(1 << 62))
        );
        assert_eq!(
            Prim::BitCeil.run(Scalar::Int((1 << 62) + 1)),
            Err(Fault::CeilOverflow)
        );
        assert_eq!(Prim::Ctz.cpp("U64", &["x".into()]), "cpl::ctz(x)");
    }

    #[test]
    fn math_traps_where_cpp_fails() {
        assert_eq!(Prim::Abs.run(Scalar::Int(i64::MIN)), Err(Fault::Overflow));
        let edge = 9223372036854775808.0;
        assert_eq!(Prim::Trunc.run(Scalar::F64(edge)), Err(Fault::NotInt(edge)));
        assert_eq!(
            Prim::Trunc.run(Scalar::F64(-edge)),
            Err(Fault::NotInt(-edge))
        );
        let below = -9223372036854774784.0;
        assert_eq!(
            Prim::Trunc.run(Scalar::F64(below)),
            Ok(Scalar::Int(below as i64))
        );
        assert_eq!(
            Prim::Abs.run(Scalar::I128(i128::MIN)),
            Err(Fault::WideOverflow)
        );
        assert_eq!(Prim::Abs.run(Scalar::I128(-5)), Ok(Scalar::I128(5)));
        assert_eq!(Prim::Abs.cpp("F64", &["x".into()]), "std::fabs(x)");
        assert_eq!(Prim::Abs.cpp("Int", &["x".into()]), "std::abs((ll)(x))");
        assert_eq!(Prim::Sqrt.run(Scalar::Int(-1)), Err(Fault::Negative(-1.0)));
        assert_eq!(Prim::Sqrt.run(Scalar::Int(9)), Ok(Scalar::F64(3.0)));
        assert_eq!(Prim::Round.run(Scalar::F64(-2.5)), Ok(Scalar::Int(-3)));
        assert_eq!(Prim::Trunc.run(Scalar::F64(-2.5)), Ok(Scalar::Int(-2)));
        assert!(matches!(
            Prim::Floor.run(Scalar::F64(f64::NAN)),
            Err(Fault::NotInt(_))
        ));
    }
}
