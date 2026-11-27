//! What a run of a program comes to, the same for every engine: its output, its events and
//! how it stopped (contract 7).
use crate::ir::{Func, FuncKey, Site};
use crate::ops::Fault;

pub struct Trap {
    pub fault: Fault,
    /// The line of the user's file that was running (`ir::Site`).
    pub line: u32,
    pub msg: String,
    /// The user's calls it left, innermost first, each with the line running in it. Only the
    /// reference interpreter and the engine fill it: generated C++ keeps no frames (decisions.md).
    pub stack: Vec<String>,
}
impl Trap {
    /// The trap's code in the diagnostic registry, or `ops::INTERNAL`.
    pub fn code(&self) -> &'static str {
        self.fault.code()
    }
}
pub struct Outcome {
    pub events: Vec<String>,
    pub output: String,
    pub trap: Option<Trap>,
}

impl Trap {
    /// A fault reading input token `t`, which a message past `Int` names.
    pub fn at_token(fault: Fault, t: Option<&str>) -> Trap {
        match (fault, t) {
            (Fault::IntPastInt, Some(t)) => Trap {
                msg: format!("`{t}` does not fit in an `Int`"),
                ..fault.into()
            },
            _ => fault.into(),
        }
    }
}

impl From<Fault> for Trap {
    fn from(fault: Fault) -> Trap {
        Trap {
            fault,
            line: 0,
            stack: Vec::new(),
            msg: fault.message().into(),
        }
    }
}

/// A trap's call stack in the user's terms, from every call it left (`calls`: the fn and the
/// line running in it, innermost first). Std's fns are left out; past 32 calls, only the 16
/// innermost and the 8 outermost are named.
pub fn call_stack<T>(
    funcs: &[Func<T>],
    calls: impl IntoIterator<Item = (usize, Site)>,
) -> Vec<String> {
    let shown: Vec<String> = calls
        .into_iter()
        .filter_map(|(f, line)| {
            let g = funcs.get(f)?;
            (g.line != 0 || matches!(g.key, FuncKey::Main))
                .then(|| format!("{} (line {line})", g.shown))
        })
        .collect();
    let n = shown.len();
    if n <= 32 {
        return shown;
    }
    let mut out = shown[..16].to_vec();
    out.push(format!("... {} frames omitted ...", n - 24));
    out.extend_from_slice(&shown[n - 8..]);
    out
}
