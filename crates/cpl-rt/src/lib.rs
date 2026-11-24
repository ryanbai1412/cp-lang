//! The intrinsics table (decision 573, spec R12.14): every primitive a
//! library body names with `@intrinsic("id")`, with its signature, its cost,
//! the interpreter's implementation and the C++ it emits. The compiler
//! matches [`Prim`] values, never the names libraries give them.

/// One primitive of the table. Its index in [`TABLE`] is its stable number.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum Prim {
    Popcount,
    Abs,
    Sqrt,
    Floor,
    Ceil,
    Round,
    Trunc,
}

/// A scalar the interpreter hands to, and gets back from, a primitive.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Scalar {
    Int(i64),
    I128(i128),
    U64(u64),
    F64(f64),
    Bool(bool),
}

impl Scalar {
    /// The 64 bits of an integer, as C++'s `(uint64_t)` cast reads them.
    pub fn bits(self) -> u64 {
        match self {
            Scalar::Int(v) => v as u64,
            Scalar::I128(v) => v as u64,
            Scalar::U64(v) => v,
            Scalar::F64(v) => v as u64,
            Scalar::Bool(b) => u64::from(b),
        }
    }

    /// The number as an `F64`, as C++'s `(double)` cast reads it.
    pub fn f64(self) -> f64 {
        match self {
            Scalar::Int(v) => v as f64,
            Scalar::I128(v) => v as f64,
            Scalar::U64(v) => v as f64,
            Scalar::F64(v) => v,
            Scalar::Bool(b) => f64::from(u8::from(b)),
        }
    }
}

/// A primitive's result, or the message of the trap it raises.
pub type Ran = Result<Scalar, String>;

/// How the interpreter runs a primitive.
#[derive(Clone, Copy)]
pub enum Vm {
    /// A function of scalar arguments; `Err` is a trap.
    Scalar(fn(&[Scalar]) -> Ran),
}

/// The C++ of a primitive for an argument type.
pub struct Cpp {
    /// The first argument's types this form is for, `|` between them.
    pub ty: &'static str,
    /// The expression: `$0`, `$1`, ... are the arguments, each read once.
    pub code: &'static str,
    /// The runtime helper the code needs, included once per bundle.
    pub need: Option<&'static str>,
}

/// A primitive: what a library names, what it takes, what it costs, and
/// how each engine runs it.
pub struct Entry {
    pub prim: Prim,
    /// The name `@intrinsic("...")` writes.
    pub id: &'static str,
    /// The parameter types, `|` between the types one accepts, and the result.
    pub params: &'static [&'static str],
    pub result: &'static str,
    /// The cost as `cpl explain` and docs show it (R09 form).
    pub cost: &'static str,
    /// What `cpl explain` and the trace say the primitive does.
    pub explain: &'static str,
    pub vm: Vm,
    pub cpp: &'static [Cpp],
}

const fn cpp(ty: &'static str, code: &'static str, need: Option<&'static str>) -> Cpp {
    Cpp { ty, code, need }
}

/// An `F64` rounded to an `Int`, trapping where C++'s `cpl::to_int` fails.
fn to_int(v: f64) -> Ran {
    if v > -9223372036854775808.0 && v < 9223372036854775808.0 {
        Ok(Scalar::Int(v as i64))
    } else {
        Err(format!("{v} does not fit in an `Int`"))
    }
}

fn abs(a: &[Scalar]) -> Ran {
    let over = |v: &dyn std::fmt::Display, ty, bits| {
        format!("integer overflow: `abs({v})` does not fit in an `{ty}` ({bits} bits)")
    };
    match a[0] {
        Scalar::Int(v) => v
            .checked_abs()
            .map(Scalar::Int)
            .ok_or_else(|| over(&v, "Int", 64)),
        Scalar::I128(v) => (v.checked_abs().map(Scalar::I128)).ok_or_else(|| over(&v, "I128", 128)),
        Scalar::F64(v) => Ok(Scalar::F64(v.abs())),
        other => Ok(other),
    }
}

fn sqrt(a: &[Scalar]) -> Ran {
    match a[0].f64() {
        x if x < 0.0 => Err(format!("sqrt({x}) of a negative number")),
        x => Ok(Scalar::F64(x.sqrt())),
    }
}

const TO_INT: &str = "narrow";

pub static TABLE: &[Entry] = &[
    Entry {
        prim: Prim::Popcount,
        id: "popcount",
        params: &["Int|U64"],
        result: "Int",
        cost: "\\(O(1)\\)",
        explain: "the number of 1 bits, one `popcnt`",
        vm: Vm::Scalar(|a| Ok(Scalar::Int(i64::from(a[0].bits().count_ones())))),
        cpp: &[cpp("*", "(ll)__builtin_popcountll((uint64_t)($0))", None)],
    },
    Entry {
        prim: Prim::Abs,
        id: "abs",
        params: &["Int|I128|F64"],
        result: "Int|I128|F64",
        cost: "\\(O(1)\\)",
        explain: "the absolute value; an integer traps where it has none",
        vm: Vm::Scalar(abs),
        cpp: &[
            cpp("F64", "std::fabs($0)", None),
            cpp("I128", "cpl::abs128($0)", Some("abs128")),
            cpp("*", "std::abs((ll)($0))", None),
        ],
    },
    Entry {
        prim: Prim::Sqrt,
        id: "sqrt",
        params: &["Int|F64"],
        result: "F64",
        cost: "\\(O(1)\\)",
        explain: "the square root as an `F64`; traps below zero",
        vm: Vm::Scalar(sqrt),
        cpp: &[
            cpp("F64", "cpl::fsqrt($0)", Some("fsqrt")),
            cpp("*", "cpl::fsqrt((double)($0))", Some("fsqrt")),
        ],
    },
    Entry {
        prim: Prim::Floor,
        id: "floor",
        params: &["F64"],
        result: "Int",
        cost: "\\(O(1)\\)",
        explain: "rounds down to an `Int`; traps outside `Int`",
        vm: Vm::Scalar(|a| to_int(a[0].f64().floor())),
        cpp: &[cpp("*", "cpl::to_int(std::floor($0))", Some(TO_INT))],
    },
    Entry {
        prim: Prim::Ceil,
        id: "ceil",
        params: &["F64"],
        result: "Int",
        cost: "\\(O(1)\\)",
        explain: "rounds up to an `Int`; traps outside `Int`",
        vm: Vm::Scalar(|a| to_int(a[0].f64().ceil())),
        cpp: &[cpp("*", "cpl::to_int(std::ceil($0))", Some(TO_INT))],
    },
    Entry {
        prim: Prim::Round,
        id: "round",
        params: &["F64"],
        result: "Int",
        cost: "\\(O(1)\\)",
        explain: "rounds to the nearest `Int`, halves away from zero; traps outside `Int`",
        vm: Vm::Scalar(|a| to_int(a[0].f64().round())),
        cpp: &[cpp("*", "cpl::to_int(std::round($0))", Some(TO_INT))],
    },
    Entry {
        prim: Prim::Trunc,
        id: "trunc",
        params: &["F64"],
        result: "Int",
        cost: "\\(O(1)\\)",
        explain: "rounds toward zero to an `Int`; traps outside `Int`",
        vm: Vm::Scalar(|a| to_int(a[0].f64().trunc())),
        cpp: &[cpp("*", "cpl::to_int($0)", Some(TO_INT))],
    },
];

impl Prim {
    pub fn entry(self) -> &'static Entry {
        &TABLE[self as usize]
    }

    /// The primitive `@intrinsic(id)` names.
    pub fn from_id(id: &str) -> Option<Prim> {
        TABLE.iter().find(|e| e.id == id).map(|e| e.prim)
    }

    pub fn id(self) -> &'static str {
        self.entry().id
    }

    /// Runs a scalar primitive, as the interpreter does.
    pub fn run(self, args: &[Scalar]) -> Ran {
        match self.entry().vm {
            Vm::Scalar(f) => f(args),
        }
    }

    /// The C++ form for a first argument of type `ty`.
    pub fn form(self, ty: &str) -> &'static Cpp {
        let forms = self.entry().cpp;
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
        assert_eq!(TABLE.len(), 7);
    }

    #[test]
    fn each_entry_sits_at_its_number_and_reads_each_argument_once() {
        for (i, e) in TABLE.iter().enumerate() {
            assert_eq!(e.prim as usize, i, "{}", e.id);
            assert_eq!(Prim::from_id(e.id), Some(e.prim));
            for form in e.cpp {
                for k in 0..e.params.len() {
                    assert_eq!(
                        form.code.matches(&format!("${k}")).count(),
                        1,
                        "{} ${k}",
                        e.id
                    );
                }
            }
            assert_eq!(e.cpp.last().map(|c| c.ty), Some("*"), "{}", e.id);
        }
    }

    #[test]
    fn popcount_counts_the_bits_cpp_counts() {
        let p = Prim::Popcount;
        assert_eq!(p.run(&[Scalar::Int(-1)]), Ok(Scalar::Int(64)));
        assert_eq!(p.run(&[Scalar::U64(0b1011)]), Ok(Scalar::Int(3)));
        assert_eq!(
            p.cpp("Int", &["x".into()]),
            "(ll)__builtin_popcountll((uint64_t)(x))"
        );
    }

    #[test]
    fn math_traps_where_cpp_fails() {
        assert!(Prim::Abs.run(&[Scalar::Int(i64::MIN)]).is_err());
        assert_eq!(Prim::Abs.run(&[Scalar::I128(-5)]), Ok(Scalar::I128(5)));
        assert_eq!(Prim::Abs.cpp("F64", &["x".into()]), "std::fabs(x)");
        assert_eq!(Prim::Abs.cpp("Int", &["x".into()]), "std::abs((ll)(x))");
        assert!(Prim::Sqrt.run(&[Scalar::Int(-1)]).is_err());
        assert_eq!(Prim::Sqrt.run(&[Scalar::Int(9)]), Ok(Scalar::F64(3.0)));
        assert_eq!(Prim::Round.run(&[Scalar::F64(-2.5)]), Ok(Scalar::Int(-3)));
        assert_eq!(Prim::Trunc.run(&[Scalar::F64(-2.5)]), Ok(Scalar::Int(-2)));
        assert!(Prim::Floor.run(&[Scalar::F64(f64::NAN)]).is_err());
    }
}
