//! Contract 5's output: the instances of a whole program and the concrete types they use.
use crate::ir::*;
use crate::msg::Msg;
use crate::ty::*;
use std::collections::HashMap;

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum CTy {
    Int,
    Bool,
    Char,
    U64,
    I128,
    F64,
    Unit,
    Raw(Box<CTy>),
    Ref(Box<CTy>),
    /// A slot that points at a heap value it does not own (`Ty::Lent`).
    Lent(Box<CTy>),
    Struct(StructId, Vec<CTy>, Vec<CSet>),
    /// A function value: its parameter and result types as the checker gave them, and one of
    /// these closures. The order is the tag.
    Fn(Vec<CTy>, Box<CTy>, CSet),
    /// One closure: its captures.
    Clo(Box<InstKey>),
    /// A domain declared in a body. Its values are positions; its size is a run-time value.
    Dom(usize),
    /// `lo..=hi` as a type argument: `MemoArray<0..=90, V>` is an instance of its own, whose
    /// `K.count` is 91. A value of it is an `Int`.
    Range(i64, i64),
    /// `lo..=D.m + k` as a type argument, `m` the `Domain` getter `Ty::RangeTo` names:
    /// `Tree<V>`'s `[0..V.count - 1]` axis. Its count is `D`'s plus `k - lo + 1`; a value of it
    /// is an `Int`.
    RangeTo(Box<CTy>, FnId, i64, i64),
    Error,
}
impl CTy {
    /// The value a slot of this type holds: a lent mark applies to slots, not values.
    pub fn unlent(&self) -> &CTy {
        match self {
            CTy::Lent(t) => t,
            t => t,
        }
    }
}
pub type CSet = Vec<InstKey>;
#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct InstKey {
    pub func: FuncKey,
    pub targs: Vec<CTy>,
    pub sets: Vec<CSet>,
}

pub struct Program {
    pub insts: Vec<Func<CTy>>,
    pub index: HashMap<InstKey, usize>,
    /// The trace role's instance that shows a value of each type an event can show (contract 8).
    pub shown: HashMap<CTy, InstKey>,
    pub main: usize,
    pub globals: Vec<CTy>,
    pub structs: HashMap<CTy, Vec<CTy>>,
    /// An enum's payload struct per variant: its value is one of them, with a tag saying which.
    pub sums: HashMap<CTy, Vec<CTy>>,
    /// Each type whose representation is an earlier type's, mapped to that type: link's
    /// canonicalization decides it once, and no backend tells the two apart.
    pub shares: HashMap<CTy, CTy>,
    /// The sites of the user's file a trap can name, once each: origin `o` is `origins[o - 1]`,
    /// and 0 is none (`Func::origins`).
    pub origins: Vec<crate::ids::SourceId>,
    pub errors: Vec<Msg>,
}
impl Program {
    /// Where origin `o` is; `None` for 0.
    pub fn origin(&self, o: u32) -> Option<&crate::ids::SourceId> {
        self.origins.get((o as usize).checked_sub(1)?)
    }
    /// The type whose representation a value of `t` has.
    pub fn rep<'a>(&'a self, t: &'a CTy) -> &'a CTy {
        self.shares.get(t).unwrap_or(t)
    }
    /// What a value of `t` holds: a struct's fields, or an enum's payloads.
    pub fn parts(&self, t: &CTy) -> Vec<CTy> {
        parts(&self.structs, &self.sums, t)
    }
    /// What a value of type `t` holds (`holds`).
    pub fn holds(&self, t: &CTy) -> Holds {
        holds(
            &|s| self.parts(s),
            &|k| {
                (self.index.get(k))
                    .and_then(|&i| self.insts.get(i))
                    .map(|_| self.captures(k))
            },
            t,
            0,
        )
    }
    /// Whether a value of type `t` holds storage: what a copy counts and a drop releases.
    pub fn stores(&self, t: &CTy) -> bool {
        self.holds(t) == Holds::Storage
    }
    /// R1: whether a value of type `t` is plain: it holds neither storage nor an address, so
    /// using it lends nothing. A view, or a closure over a heap local, is copied but lends its
    /// source (R05.10.1-2), so ownership asks this, not `copied`, for what a value borrows.
    pub fn plain(&self, t: &CTy) -> bool {
        self.holds(t) == Holds::Nothing
    }
    /// R1: whether using a value of type `t` copies it rather than moving it: it owns no
    /// storage, so a fn value that captures by reference is copied (R05.10.1-2).
    pub fn copied(&self, t: &CTy) -> bool {
        self.holds(t) != Holds::Storage
    }
    /// The types of what closure `k` captures.
    pub fn captures(&self, k: &InstKey) -> Vec<CTy> {
        let f = &self.insts[self.index[k]];
        f.params[..f.n_captures]
            .iter()
            .map(|(l, _)| f.locals[*l].clone())
            .collect()
    }
}

/// A struct's fields, then a sum's payloads.
pub fn parts(structs: &HashMap<CTy, Vec<CTy>>, sums: &HashMap<CTy, Vec<CTy>>, t: &CTy) -> Vec<CTy> {
    let mut out = structs.get(t).cloned().unwrap_or_default();
    out.extend(sums.get(t).into_iter().flatten().cloned());
    out
}

/// What a value holds, the one answer to "is it plain" and "does it hold storage" (R1).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Holds {
    /// No part of it lives on the heap or points at another value: it is plain, and a copy of
    /// it never affects the original.
    Nothing,
    /// It points at a value another frame owns (a `Ref` or a lent slot), and owns no storage:
    /// a closure with only lent captures releases nothing, as its captures are the caller's.
    Address,
    /// A `Raw` handle in it or in a part: what a copy counts and a drop releases.
    Storage,
}

/// R1: what a value of type `t` holds. A fn value holds what its closures capture. Ownership
/// places drops and copies by it and a backend counts a copy by it, so all of them agree on
/// which values are plain and which own. Past a depth of 16, or for a closure with no instance
/// yet (link has not made it, or stopped), it says storage.
pub fn holds(
    parts: &dyn Fn(&CTy) -> Vec<CTy>,
    captures: &dyn Fn(&InstKey) -> Option<Vec<CTy>>,
    t: &CTy,
    depth: usize,
) -> Holds {
    let at = |t: &CTy| holds(parts, captures, t, depth + 1);
    let most = |ts: &[CTy]| ts.iter().map(at).max().unwrap_or(Holds::Nothing);
    let clo = |k: &InstKey| captures(k).map_or(Holds::Storage, |cs| most(&cs));
    match t {
        CTy::Raw(_) => Holds::Storage,
        _ if depth >= 16 => Holds::Storage,
        CTy::Int
        | CTy::Bool
        | CTy::Char
        | CTy::U64
        | CTy::I128
        | CTy::F64
        | CTy::Unit
        | CTy::Dom(_)
        | CTy::Range(..)
        | CTy::RangeTo(..) => Holds::Nothing,
        CTy::Ref(_) | CTy::Lent(_) | CTy::Error => Holds::Address,
        CTy::Struct(..) => most(&parts(t)),
        CTy::Clo(k) => clo(k),
        CTy::Fn(_, _, ms) => ms.iter().map(clo).max().unwrap_or(Holds::Nothing),
    }
}

/// Whether a value of type `t` holds storage (`holds`).
pub fn stores(
    parts: &dyn Fn(&CTy) -> Vec<CTy>,
    captures: &dyn Fn(&InstKey) -> Option<Vec<CTy>>,
    t: &CTy,
) -> bool {
    holds(parts, captures, t, 0) == Holds::Storage
}

/// R1: whether a value of type `t` is plain (`holds`).
pub fn plain(
    parts: &dyn Fn(&CTy) -> Vec<CTy>,
    captures: &dyn Fn(&InstKey) -> Option<Vec<CTy>>,
    t: &CTy,
) -> bool {
    holds(parts, captures, t, 0) == Holds::Nothing
}

/// R1: the type of the slot a value of `t` is lent in, in an instance: a plain value lent is copied,
/// as nothing it holds can be kept past the lend. A fn value is plain when what it captures is.
pub fn lent_slot(
    t: CTy,
    parts: &dyn Fn(&CTy) -> Vec<CTy>,
    captures: &dyn Fn(&InstKey) -> Option<Vec<CTy>>,
) -> CTy {
    match plain(parts, captures, &t) {
        true => t,
        false => CTy::Lent(Box::new(t)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(n: &str) -> InstKey {
        InstKey {
            func: FuncKey::Main,
            targs: vec![CTy::Range(0, n.len() as i64)],
            sets: vec![],
        }
    }

    /// A fn value is plain exactly when what it captures is (R1), and holds storage exactly when
    /// a capture does: one answer for ownership's copies and drops and the engine's counts.
    #[test]
    fn a_fn_value_holds_what_it_captures() {
        let (ints, list, unmade) = (key("a"), key("ab"), key("abc"));
        let caps = |k: &InstKey| match k.targs[0] {
            CTy::Range(_, 1) => Some(vec![CTy::Int, CTy::Bool]),
            CTy::Range(_, 2) => Some(vec![CTy::Int, CTy::Raw(Box::new(CTy::Int))]),
            _ => None,
        };
        let parts = |_: &CTy| Vec::new();
        let at = |t: &CTy| holds(&parts, &caps, t, 0);
        let clo = |k: &InstKey| CTy::Clo(Box::new(k.clone()));
        assert_eq!(at(&clo(&ints)), Holds::Nothing);
        assert_eq!(at(&clo(&list)), Holds::Storage);
        assert_eq!(at(&clo(&unmade)), Holds::Storage);
        let both = CTy::Fn(vec![], Box::new(CTy::Int), vec![ints.clone(), list]);
        assert!(stores(&parts, &caps, &both) && !plain(&parts, &caps, &both));
        let one = CTy::Fn(vec![], Box::new(CTy::Int), vec![ints]);
        assert!(plain(&parts, &caps, &one) && !stores(&parts, &caps, &one));
        assert_eq!(at(&CTy::Ref(Box::new(CTy::Int))), Holds::Address);
    }

    /// A view holds its source's address: copied, not moved, yet not plain, as it lends it.
    #[test]
    fn a_view_is_copied_but_lends() {
        let caps = |_: &InstKey| None;
        let view = CTy::Struct(StructId(0), vec![], vec![]);
        let parts = |t: &CTy| match t {
            CTy::Struct(..) => vec![CTy::Ref(Box::new(CTy::Raw(Box::new(CTy::Int)))), CTy::Int],
            _ => Vec::new(),
        };
        assert_eq!(holds(&parts, &caps, &view, 0), Holds::Address);
        assert!(!plain(&parts, &caps, &view) && !stores(&parts, &caps, &view));
    }
}

#[cfg(test)]
mod lent_slot_tests {
    use super::*;
    use crate::ir::FuncKey;

    /// A fn value lent is copied when what it captures is plain, and lent in a slot when it
    /// captures an address or storage (R05.10.1-2): its real captures decide, not a stub.
    #[test]
    fn a_fn_value_lent_is_copied_when_its_captures_are_plain() {
        let key = |n: i64| InstKey {
            func: FuncKey::Main,
            targs: vec![CTy::Range(0, n)],
            sets: vec![],
        };
        let caps = |k: &InstKey| match k.targs[0] {
            CTy::Range(0, 0) => Some(vec![CTy::Int, CTy::Bool]),
            _ => Some(vec![CTy::Ref(Box::new(CTy::Int))]),
        };
        let none = |_: &CTy| Vec::new();
        let plain = CTy::Clo(Box::new(key(0)));
        let lending = CTy::Clo(Box::new(key(1)));
        assert_eq!(lent_slot(plain.clone(), &none, &caps), plain);
        assert_eq!(
            lent_slot(lending.clone(), &none, &caps),
            CTy::Lent(Box::new(lending))
        );
    }
}
