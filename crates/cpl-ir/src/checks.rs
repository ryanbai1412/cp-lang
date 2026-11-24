//! What a build checks at run time (R09.3): [`Checks`], the one table the
//! code generator's judge builds and the interpreter's release runs read.

/// The runtime checks a judge build keeps (`cpl build --checks`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Checks {
    /// No index, `/ 0` or `assert` traps: what hand-written C++ checks. A
    /// zero divisor gives 0, so the program still has no undefined
    /// behaviour there; an index out of range is left unchecked.
    None,
    /// Index, `/ 0` and `assert` stop the program with their message.
    #[default]
    Traps,
    /// [`Checks::Traps`], plus range stores, overflow and the other checks
    /// of the local build (`-DLOCAL`).
    All,
}

impl Checks {
    /// What every local run checks, as the debug interpreter does: `cpl
    /// test`, `cpl stress` and their builds of brutes and generators. A
    /// program that traps under `cpl run` traps under each of them.
    pub const LOCAL: Checks = Checks::All;

    pub const NAMES: [&'static str; 3] = ["none", "traps", "all"];

    pub fn parse(name: &str) -> Option<Checks> {
        Some(match name {
            "none" => Checks::None,
            "traps" => Checks::Traps,
            "all" => Checks::All,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Checks::None => "none",
            Checks::Traps => "traps",
            Checks::All => "all",
        }
    }

    /// Whether a build at this level traps on `check`. Where it does not,
    /// the operation does what release does (R09.3): overflow wraps
    /// (R09.3.4), a zero divisor gives 0 (R09.3.6), an assert is skipped
    /// (R09.3.7). The code generator and the interpreter both read this.
    pub fn keeps(self, check: Check) -> bool {
        match self {
            Checks::None => false,
            Checks::Traps => matches!(check, Check::Bounds | Check::Div | Check::Asserts),
            Checks::All => true,
        }
    }
}

/// One row of the checks table (R09.3), by its `with checks(...)` name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Check {
    /// An index, `D(x)` or a raw index outside its bounds.
    Bounds,
    /// A value stored into a range type it does not fit.
    Ranges,
    /// An `Int` result that does not fit.
    Overflow,
    /// `x!` on `None`.
    Unwrap,
    /// A zero divisor, a negative shift, `sqrt` of a negative.
    Div,
    /// `assert c`.
    Asserts,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_level_keeps_its_rows() {
        let rows = [
            Check::Bounds,
            Check::Ranges,
            Check::Overflow,
            Check::Unwrap,
            Check::Div,
            Check::Asserts,
        ];
        assert!(rows.iter().all(|&c| Checks::All.keeps(c)));
        assert!(rows.iter().all(|&c| !Checks::None.keeps(c)));
        let traps: Vec<_> = rows
            .into_iter()
            .filter(|&c| Checks::Traps.keeps(c))
            .collect();
        assert_eq!(traps, [Check::Bounds, Check::Div, Check::Asserts]);
    }
}
