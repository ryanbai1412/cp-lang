//! Facts about the build a program is checked for, passed down by the pipeline as plain data.

/// The judge a program is built for, as far as checking reads it (R03.1.9).
#[derive(Clone, Debug)]
pub struct Judge {
    pub name: String,
    pub memory_mb: u64,
}

/// One check's name in `with checks(..)`, `release_checks:` and diagnostics (R09.3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CheckName {
    Bounds,
    Ranges,
    Overflow,
    Unwrap,
    Div,
    Asserts,
    Reads,
}

impl CheckName {
    /// Each check by the name the source writes, the one list (R09.3).
    pub const ALL: [(&'static str, CheckName); 7] = [
        ("bounds", CheckName::Bounds),
        ("ranges", CheckName::Ranges),
        ("overflow", CheckName::Overflow),
        ("unwrap", CheckName::Unwrap),
        ("div", CheckName::Div),
        ("asserts", CheckName::Asserts),
        ("reads", CheckName::Reads),
    ];
    pub fn named(text: &str) -> Option<CheckName> {
        (CheckName::ALL.iter()).find_map(|&(n, c)| (n == text).then_some(c))
    }
}

/// Which checks a build keeps (contract 0): a checked build keeps all, release none but those
/// a `with checks(..: on)` or the judge's `release_checks:` keeps (R09.4).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Checks(u8);

impl Checks {
    pub const CHECKED: Checks = Checks(0x7f);
    /// No check: release before `with checks(..: on)`, and `unchecked:` in every build.
    pub const NONE: Checks = Checks(0);
    pub fn keeps(self, c: CheckName) -> bool {
        self.0 & (1 << c as u8) != 0
    }
    pub fn with(self, c: CheckName, on: bool) -> Checks {
        match on {
            true => Checks(self.0 | 1 << c as u8),
            false => Checks(self.0 & !(1 << c as u8)),
        }
    }
}

/// What a build is for, as checking reads it: its judge and the checks it keeps (contract 0).
#[derive(Clone, Debug)]
pub struct Build {
    pub judge: Option<Judge>,
    pub checks: Checks,
}
