//! What a run of a program comes to, the same for every engine: its output, its events and
//! how it stopped (contract 7).
use crate::ops::Fault;

pub struct Trap {
    pub fault: Fault,
    /// The line of the user's file that was running (`ir::Site`).
    pub line: u32,
    pub msg: String,
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
            msg: fault.message().into(),
        }
    }
}
