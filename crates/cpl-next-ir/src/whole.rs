//! What link reads of the program as a whole beyond its fns and their set adds (`SetAdds`), which lowering hands on from
//! the checker so link reads no typed tree.
use crate::msg::Msg;
use crate::ty::*;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum VarRef {
    Local(LocalId),
    Global(LocalId),
}

/// A domain's place in `Whole::domains`: what a `Ty::Dom` names.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct DomId(pub usize);

/// A global's place in `Whole::globals`: what a `Place::Global` names.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct GlobalId(pub usize);

/// One of the two run-time values a domain carries.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum DomValue {
    /// Its size.
    Len(DomId),
    /// The integer of its first position.
    Lo(DomId),
}

/// A domain: the variables that hold its size and the integer of its first position.
#[derive(Clone, Debug)]
pub struct Dom {
    pub name: String,
    pub len: VarRef,
    pub lo: VarRef,
    /// The fn it is declared in; `None` at top level.
    pub owner: Option<FnId>,
}

/// A global's name, and whether it is a top-level binding of the file rather than one inside
/// a block.
#[derive(Clone, Debug)]
pub struct Global {
    pub name: String,
    pub top: bool,
}

/// The facts of the program as a whole that link reads beside its fns (contract 4).
#[derive(Clone, Debug, Default)]
pub struct Whole {
    /// By `DomId`.
    pub domains: Vec<Dom>,
    /// By `GlobalId`.
    pub globals: Vec<Global>,
    /// The diagnostics of each library fn reached only through a trait, each naming the layer
    /// that found it: link says them if an instance runs the fn.
    pub deferred: HashMap<FnId, Vec<Msg>>,
    /// The fns written in the user's program: where event points report entering and leaving.
    pub program_fns: HashSet<FnId>,
}

impl Whole {
    pub fn dom(&self, d: DomId) -> &Dom {
        &self.domains[d.0]
    }
    pub fn global(&self, g: GlobalId) -> Option<&Global> {
        self.globals.get(g.0)
    }
    /// Every run-time value of every domain.
    pub fn dom_values(&self) -> impl Iterator<Item = DomValue> {
        (0..self.domains.len()).flat_map(|d| [DomValue::Len(DomId(d)), DomValue::Lo(DomId(d))])
    }
    /// The variable that holds `v`, and the fn its domain is declared in.
    pub fn dom_var(&self, v: DomValue) -> (VarRef, Option<FnId>) {
        match v {
            DomValue::Len(d) => (self.dom(d).len, self.dom(d).owner),
            DomValue::Lo(d) => (self.dom(d).lo, self.dom(d).owner),
        }
    }
}
