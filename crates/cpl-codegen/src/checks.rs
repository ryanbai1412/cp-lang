//! What a judge build checks at run time: [`Checks`], and the runtime
//! rewritten for a build that keeps no traps.

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
}

/// Each trap in the runtime pieces, and what a build without traps does
/// instead.
const TRAPS: &[(&str, &str)] = &[
    (
        r#"{ cpl::flush(); fputs("division by zero\n", stderr); exit(3); }"#,
        "return 0;",
    ),
    (
        r#"if ((unsigned long long)i >= v.size()) fail("index outside the list"); "#,
        "",
    ),
    (
        r#"if ((unsigned long long)i >= (unsigned long long)K) fail("index outside the list"); "#,
        "",
    ),
    (
        r#"if ((unsigned long long)i >= s.size()) fail("index outside the string"); "#,
        "",
    ),
    (r#"if (bad) fail("index outside the array"); "#, ""),
];

/// `code`, a runtime piece, without its index and `/ 0` traps.
pub(crate) fn untrapped(code: &str) -> String {
    TRAPS
        .iter()
        .fold(code.to_string(), |code, (trap, instead)| {
            code.replace(trap, instead)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_trap_is_in_the_runtime() {
        for (trap, _) in TRAPS {
            assert!(
                crate::runtime::PIECES.iter().any(|p| p.code.contains(trap)),
                "no piece has {trap}"
            );
        }
    }

    fn cpp(checks: Checks) -> String {
        let source = concat!(
            "let n: Int = read()\n",
            "let xs = [1, 2, 3]\n",
            "assert n != 7\n",
            "print(xs[n] + 10 / n + 10 % n)\n",
        );
        let mut db = cpl_check::Database::new();
        db.set_file("t.cp", source);
        let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
        let target = crate::Target {
            checks,
            ..crate::Target::default()
        };
        crate::emit_for(&program, "t.cp", None, &target)
    }

    #[test]
    fn each_level_keeps_its_traps() {
        let traps = [
            "division by zero",
            "index outside the list",
            "assertion failed",
        ];
        let kept = cpp(Checks::Traps);
        for trap in traps {
            assert!(kept.contains(trap), "{trap}\n{kept}");
        }
        assert!(!kept.contains("#define LOCAL"));
        let none = cpp(Checks::None);
        for trap in traps {
            assert!(!none.contains(trap), "{trap}\n{none}");
        }
        assert!(cpp(Checks::All).contains("#define LOCAL\n"));
    }

    #[test]
    fn names_round_trip() {
        for name in Checks::NAMES {
            assert_eq!(Checks::parse(name).map(Checks::name), Some(name));
        }
        assert_eq!(Checks::parse("some"), None);
    }
}
