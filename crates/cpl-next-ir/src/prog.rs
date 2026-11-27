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
    Error,
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
    pub main: usize,
    pub globals: Vec<CTy>,
    pub structs: HashMap<CTy, Vec<CTy>>,
    /// An enum's payload struct per variant: its value is one of them, with a tag saying which.
    pub sums: HashMap<CTy, Vec<CTy>>,
    /// Each type whose representation is an earlier type's, mapped to that type: link's
    /// canonicalization decides it once, and no backend tells the two apart.
    pub shares: HashMap<CTy, CTy>,
    pub errors: Vec<Msg>,
}
impl Program {
    /// The type whose representation a value of `t` has.
    pub fn rep<'a>(&'a self, t: &'a CTy) -> &'a CTy {
        self.shares.get(t).unwrap_or(t)
    }
    /// What a value of `t` holds: a struct's fields, or an enum's payloads.
    pub fn parts(&self, t: &CTy) -> Vec<CTy> {
        parts(&self.structs, &self.sums, t)
    }
    /// Whether a value of type `t` holds storage (`stores`).
    pub fn stores(&self, t: &CTy) -> bool {
        stores(
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

/// R1: whether a value of type `t` holds storage, a `Raw` handle in it or in a part: what a
/// copy counts and a drop releases. Ownership places drops by it and a backend counts a copy by
/// it, so the two agree on which values own. Past a depth of 16, or for a closure with no
/// instance (link stopped), it says yes.
pub fn stores(
    parts: &dyn Fn(&CTy) -> Vec<CTy>,
    captures: &dyn Fn(&InstKey) -> Option<Vec<CTy>>,
    t: &CTy,
    depth: usize,
) -> bool {
    let at = |t: &CTy| stores(parts, captures, t, depth + 1);
    match t {
        CTy::Raw(_) => true,
        _ if depth >= 16 => true,
        CTy::Lent(e) => at(e),
        CTy::Struct(..) => parts(t).iter().any(at),
        CTy::Clo(k) => captures(k).is_none_or(|cs| cs.iter().any(at)),
        CTy::Fn(_, _, ms) => (ms.iter()).any(|k| captures(k).is_none_or(|cs| cs.iter().any(at))),
        _ => false,
    }
}

/// Spec R1: a value with no heap part, copied when passed; `parts` gives a struct's fields.
pub fn plain(parts: &dyn Fn(&CTy) -> Vec<CTy>, t: &CTy, depth: usize) -> bool {
    match t {
        CTy::Int
        | CTy::Bool
        | CTy::Char
        | CTy::U64
        | CTy::F64
        | CTy::Unit
        | CTy::Dom(_)
        | CTy::Range(..) => true,
        CTy::Struct(..) if depth < 8 => parts(t).iter().all(|f| plain(parts, f, depth + 1)),
        _ => false,
    }
}
