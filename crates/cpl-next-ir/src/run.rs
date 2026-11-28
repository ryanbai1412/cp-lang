//! What a run of a program comes to, the same for every engine: its output, its events and
//! how it stopped (contract 7).
use crate::ids::SourceId;
use crate::ir::{EventKind, EventPoint, Func, FuncKey, Site};
use crate::layout::Layouts;
use crate::ops::{Fault, Input};
use crate::prog::Program;
use std::collections::HashMap;

pub struct Trap {
    pub fault: Fault,
    /// The innermost site of the user's file that was running: its `Program::origins` number,
    /// 0 for none.
    pub origin: u32,
    pub msg: String,
    /// The calls it left, innermost first: each fn and the origin running in it. Only the
    /// reference interpreter and the engine fill it: generated C++ keeps no frames (decisions.md).
    pub calls: Vec<(usize, u32)>,
}
impl Trap {
    /// The trap's code in the diagnostic registry, or `ops::INTERNAL`.
    pub fn code(&self) -> &'static str {
        self.fault.code()
    }
}
/// What a run is given (contract 0's `RunInput`): its input and its limits.
pub struct RunInput {
    pub io: Io,
    pub limits: Limits,
}
/// Where a run's input comes from (contract 0).
pub enum Io {
    /// All of it, given before the run.
    Fixed(Vec<u8>),
    /// Read as the program reads: a terminal or a pipe.
    Stream(Box<dyn std::io::Read + Send>),
}
impl Io {
    /// The program's input, read as it asks.
    pub fn input(self) -> Input {
        match self {
            Io::Fixed(b) => Input::new(&b),
            Io::Stream(r) => Input::streamed(r),
        }
    }
}
/// The stack a run's calls take against `Limits::stack`, by one rule on every backend: a call
/// takes its raw instance's frame words (`Layouts::frame_words`), whichever form runs, and
/// two more, the return address and frame link a native call keeps.
#[derive(Clone, Default)]
pub struct Stack {
    limit: u64,
    /// The globals' words, below every frame.
    base: u64,
    cost: Vec<u64>,
    /// The words taken up to and including each call running, outermost first.
    at: Vec<u64>,
}
impl Stack {
    pub fn new(p: &Program, limits: Limits) -> Stack {
        let mut lays = Layouts::default();
        let cost = (p.insts.iter()).map(|f| lays.frame_words(p, f)).collect();
        let base = lays.globals_words(p);
        Stack {
            limit: limits.stack,
            base,
            cost,
            at: Vec::new(),
        }
    }
    /// This stack, empty, for a run under `limits`.
    pub fn with(&self, limits: Limits) -> Stack {
        Stack {
            limit: limits.stack,
            ..self.clone()
        }
    }
    /// Enters a call of `f` beside `calls` calls running, unless the stack runs out. The calls
    /// past `calls` have returned, so no backend says when a call leaves.
    pub fn enter(&mut self, f: usize, calls: usize) -> Result<(), Fault> {
        self.at.truncate(calls);
        let words = self.at.last().copied().unwrap_or(self.base) + self.cost[f];
        if (words + 2 * (calls as u64 + 1)).saturating_mul(8) > self.limit {
            return Err(Fault::StackExhausted);
        }
        self.at.push(words);
        Ok(())
    }
}
/// What a run may use (contract 0's `Limits`).
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// Bytes of stack. A call takes its frame's words (`layout::Layouts::frame_words`) and two
    /// more, the return address and frame link a native call keeps.
    pub stack: u64,
}
impl Default for Limits {
    /// The generic judge's: a stack as large as its 256 MB of memory.
    fn default() -> Limits {
        Limits { stack: 256 << 20 }
    }
}
impl RunInput {
    /// A run of all of `input`, under the default limits.
    pub fn fixed(input: &str) -> RunInput {
        RunInput {
            io: Io::Fixed(input.as_bytes().to_vec()),
            limits: Limits::default(),
        }
    }
}
/// An event a run reported: its point's kind, `func`'s `sites[site]`, and its values shown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fired {
    pub kind: EventKind,
    pub func: u32,
    pub site: Site,
    pub vals: Vec<String>,
}

impl Fired {
    /// `p` firing with its values shown.
    pub fn of<T>(p: &EventPoint<T>, vals: Vec<String>) -> Fired {
        let (kind, func, site) = (p.kind.clone(), p.func, p.site);
        Fired {
            kind,
            func,
            site,
            vals,
        }
    }
    /// Its trace line: `ev enter f 3`.
    pub fn line<T>(&self, funcs: &[Func<T>]) -> String {
        let mut s = format!("ev {}", self.kind.label(&funcs[self.func as usize].name));
        for v in &self.vals {
            s.push(' ');
            s.push_str(v);
        }
        s
    }
}

/// Where an event is in a run (contract 8): each call it is inside, as the fn and which of
/// its caller's calls it was, its site, and which time that site ran inside this call. Two
/// runs line up by it, not by position in the stream.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct At {
    pub call: Vec<(u32, u32)>,
    pub site: SourceId,
    pub nth: u32,
}

/// Each event's `At`, from the stream alone: an `Enter` opens a call and its `Exit` closes it.
pub fn at<T>(events: &[Fired], funcs: &[Func<T>]) -> Vec<At> {
    struct Call {
        path: Vec<(u32, u32)>,
        calls: u32,
        seen: HashMap<(u32, Site), u32>,
    }
    let open = |path| Call {
        path,
        calls: 0,
        seen: HashMap::new(),
    };
    let mut stack = vec![open(Vec::new())];
    let mut out = Vec::with_capacity(events.len());
    for e in events {
        if e.kind == EventKind::Enter {
            if let Some(caller) = stack.last_mut() {
                let mut path = caller.path.clone();
                path.push((e.func, caller.calls));
                caller.calls += 1;
                stack.push(open(path));
            }
        }
        let Some(call) = stack.last_mut() else {
            continue;
        };
        let nth = call.seen.entry((e.func, e.site)).or_insert(0);
        out.push(At {
            call: call.path.clone(),
            site: funcs[e.func as usize].sites[e.site as usize].clone(),
            nth: *nth,
        });
        *nth += 1;
        if e.kind == EventKind::Exit && stack.len() > 1 {
            stack.pop();
        }
    }
    out
}

pub struct Outcome {
    pub events: Vec<Fired>,
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
            origin: 0,
            calls: Vec::new(),
            msg: fault.message().into(),
        }
    }
}

/// A trap's call stack in the user's terms, from every call it left (`calls`: the fn and the
/// origin running in it, innermost first), each origin shown by `line`. Std's fns are left out; past 32 calls, only the 16
/// innermost and the 8 outermost are named.
pub fn call_stack<T>(
    funcs: &[Func<T>],
    calls: impl IntoIterator<Item = (usize, u32)>,
    line: impl Fn(u32) -> u32,
) -> Vec<String> {
    let shown: Vec<String> = calls
        .into_iter()
        .filter_map(|(f, o)| {
            let g = funcs.get(f)?;
            (g.line != 0 || matches!(g.key, FuncKey::Main))
                .then(|| format!("{} (line {})", g.shown, line(o)))
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
