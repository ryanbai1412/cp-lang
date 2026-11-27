//! What link reads of the program as a whole beyond its fns and their set adds (`SetAdds`), which lowering hands on from
//! the checker so link reads no typed tree.
use crate::msg::Msg;
use crate::ty::*;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum VarRef {
    Local(LocalId),
    Global(LocalId),
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

#[derive(Clone, Debug, Default)]
pub struct Whole {
    pub domains: Vec<Dom>,
    pub globals: Vec<Global>,
    /// The errors of each library fn reached only through a trait, said if an instance runs it.
    pub deferred: HashMap<FnId, Vec<Msg>>,
}
