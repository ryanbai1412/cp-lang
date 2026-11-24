//! The intrinsics table (decision 573, spec R12.14): every primitive a
//! library body names with `@intrinsic("id")`, with its signature, its cost,
//! the interpreter's implementation and the C++ it emits. The compiler
//! matches [`Prim`] values, never the names libraries give them.

/// One primitive of the table. Its index in [`TABLE`] is its stable number.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum Prim {
    Popcount,
}

/// A scalar the interpreter hands to, and gets back from, a primitive.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Scalar {
    Int(i64),
    U64(u64),
    F64(f64),
    Bool(bool),
}

impl Scalar {
    /// The 64 bits of an integer, as C++'s `(uint64_t)` cast reads them.
    pub fn bits(self) -> u64 {
        match self {
            Scalar::Int(v) => v as u64,
            Scalar::U64(v) => v,
            Scalar::F64(v) => v as u64,
            Scalar::Bool(b) => u64::from(b),
        }
    }
}

/// How the interpreter runs a primitive.
#[derive(Clone, Copy)]
pub enum Vm {
    /// A pure function of scalar arguments.
    Scalar(fn(&[Scalar]) -> Scalar),
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
    /// The C++ expression: `$0`, `$1`, ... are the arguments, each read once.
    pub cpp: &'static str,
    /// The runtime header or helper the snippet needs, included once per bundle.
    pub need: Option<&'static str>,
}

pub static TABLE: &[Entry] = &[Entry {
    prim: Prim::Popcount,
    id: "popcount",
    params: &["Int|U64"],
    result: "Int",
    cost: "\\(O(1)\\)",
    explain: "the number of 1 bits, one `popcnt`",
    vm: Vm::Scalar(|a| Scalar::Int(i64::from(a[0].bits().count_ones()))),
    cpp: "(ll)__builtin_popcountll((uint64_t)($0))",
    need: None,
}];

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
    pub fn run(self, args: &[Scalar]) -> Scalar {
        match self.entry().vm {
            Vm::Scalar(f) => f(args),
        }
    }

    /// The C++ of a call, given each argument's C++.
    pub fn cpp(self, args: &[String]) -> String {
        let mut out = String::new();
        let mut rest = self.entry().cpp;
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
        assert_eq!(TABLE.len(), 1);
    }

    #[test]
    fn each_entry_sits_at_its_number_and_reads_each_argument_once() {
        for (i, e) in TABLE.iter().enumerate() {
            assert_eq!(e.prim as usize, i, "{}", e.id);
            assert_eq!(Prim::from_id(e.id), Some(e.prim));
            for k in 0..e.params.len() {
                assert_eq!(e.cpp.matches(&format!("${k}")).count(), 1, "{} ${k}", e.id);
            }
        }
    }

    #[test]
    fn popcount_counts_the_bits_cpp_counts() {
        let p = Prim::Popcount;
        assert_eq!(p.run(&[Scalar::Int(-1)]), Scalar::Int(64));
        assert_eq!(p.run(&[Scalar::U64(0b1011)]), Scalar::Int(3));
        assert_eq!(
            p.cpp(&["x".into()]),
            "(ll)__builtin_popcountll((uint64_t)(x))"
        );
    }
}
