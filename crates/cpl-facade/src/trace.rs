//! What tools read from a run's recording, in the facade's own forms (contract 8).

/// A checkpoint both runs reached, with what each showed there.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Stage {
    pub name: String,
    /// 1-based line in the solution.
    pub line: u32,
    pub fast: String,
    pub brute: String,
}

impl From<cpl_interp::trace::Stage> for Stage {
    fn from(d: cpl_interp::trace::Stage) -> Stage {
        Stage {
            name: d.name,
            line: d.line,
            fast: d.fast,
            brute: d.brute,
        }
    }
}

/// Where a solution and its brute first give a variable they both name
/// different values: its `nth` value (1-based, a repeat of the value
/// before not counted), with the line and the value on each side, `None`
/// on a side that never gives it an `nth` value.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Divergence {
    pub name: String,
    pub nth: usize,
    pub fast: Option<(u32, String)>,
    pub brute: Option<(u32, String)>,
    /// The solution's own last steps in a wrong call, not those of the
    /// calls it makes: each line and its locals, as `k: 1, s: 0`.
    pub inside: Vec<(u32, String)>,
    /// The lines to look at, each with why: where a wrong call made its
    /// value, where what that line reads last changed, or the declaration
    /// of a variable an earlier test left a value in.
    pub blame: Vec<(u32, String)>,
    /// `name` is a call, `f(k: 1)`, and `nth` which call of `f` it is.
    pub call: bool,
    /// The solution never finishes: `name` is the loop it goes round in,
    /// `fast` that loop's line and the state it keeps coming back to, and
    /// `nth` how many steps one time round takes.
    pub stuck: bool,
}

impl From<cpl_interp::trace::Divergence> for Divergence {
    fn from(d: cpl_interp::trace::Divergence) -> Divergence {
        Divergence {
            name: d.name,
            nth: d.nth,
            fast: d.fast,
            brute: d.brute,
            inside: d.inside,
            blame: d.blame,
            call: d.call,
            stuck: d.stuck,
        }
    }
}
