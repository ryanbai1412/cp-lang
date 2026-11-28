//! What link reads of the program as a whole beyond its fns and their set adds (`SetAdds`), which lowering hands on from
//! the checker so link reads no typed tree.
use crate::ty::*;
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum VarRef {
    Local(LocalId),
    Global(LocalId),
}

pub use crate::ty::{DomId, RangeId};

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
    /// It is declared by a count alone, so `io.base` shifts it (R10.2.2).
    pub counted: bool,
    /// The traits its impl rows are of, `Domain` and `Ordered` (S11): what link reads for it.
    pub impls: Vec<TraitId>,
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
    pub domains: BTreeMap<DomId, Dom>,
    /// By `GlobalId`.
    pub globals: Vec<Global>,
    /// The fns written in the user's program: where event points report entering and leaving.
    pub program_fns: HashSet<FnId>,
    /// For a call of a trait's fn's `k`-th context copy, the copy of each impl's fn it runs
    /// (R09.1.3): `(k, impl fn)` to that fn's copy.
    pub ctx_impls: HashMap<(u32, FnId), u32>,
}

impl Whole {
    pub fn dom(&self, d: DomId) -> &Dom {
        &self.domains[&d]
    }
    pub fn global(&self, g: GlobalId) -> Option<&Global> {
        self.globals.get(g.0)
    }
}
