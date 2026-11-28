//! Contract "facts/types": one type representation for every layer.
//! Var forms exist only inside the checker; `zonk` removes them.
use crate::prog::Holds;

/// A fn declaration: an index into `Decls::fns`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct FnId(pub usize);
/// A struct declaration: an index into `Decls::structs`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct StructId(pub usize);
/// A trait declaration: an index into `Decls::traits`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct TraitId(pub usize);
pub type LocalId = usize;
/// A lambda is named by the unit it sits in and its span there.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct LambdaId(pub usize);

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum Ty {
    Int,
    /// `lo..=hi` with constant ends: an `Int` that a slot of this type is promised to hold. A
    /// value read from the slot is an `Int`; an `Int` stored into it is checked (R03.1.4).
    Range(i64, i64),
    /// A range written in a body with an end that is a run-time value: `1..=n`. Each one written
    /// is a type of its own; its ends are values held where it is written (`check::RangeInfo`).
    RangeOf(usize),
    /// `_..=_` as an impl's type: every range type and never `Int`. It binds the impl's
    /// parameter `i`, which its body names `Self`, to the range type it is for.
    AnyRange(usize),
    /// `lo..=D.m + k` in a declaration: a range whose end is a static getter `m` of `Domain`, of
    /// a domain type, give or take a constant (`dep: [V]0..=V.count`). A store checks it against `D.m` where
    /// it is made, so it keeps the end a struct's arguments give it (R03.1.4).
    RangeTo(Box<Ty>, FnId, i64, i64),
    /// `lo..hi` with an end that is a value parameter (`0..P` of `Mod<P>`, R03.5.1). Built only
    /// through [`Ty::range_ends`], which gives `Range` once both ends are singletons, so a fully
    /// substituted type never holds it.
    RangeEnds {
        lo: End,
        hi: End,
        inclusive: bool,
    },
    Bool,
    /// An ASCII character, `0..=127`, one byte.
    Char,
    /// `0..=2^64-1`, wrapping (R02.1).
    U64,
    /// A signed 128-bit integer, for products past `Int`; overflow traps (R02.1, R03.4).
    I128,
    /// An IEEE double (R02.1).
    F64,
    Unit,
    Raw(Box<Ty>),
    /// A closure's view of a `var` of the fn around it. Made by lowering only.
    Ref(Box<Ty>),
    /// A heap value a struct slot holds lent from the caller (R05.5.3) instead of owning it.
    /// Made by lowering only; a plain type in it is just that type.
    Lent(Box<Ty>),
    Struct(StructId, Vec<Ty>, Vec<SetTy>),
    /// A stored function value: one of the closures in the set.
    Fn(Vec<Ty>, Box<Ty>, SetTy),
    /// A `mut` parameter of a function type, `(mut S, T) -> ()`: the caller lends the place
    /// (R01.8.1, R05.4.1). Only ever a parameter of a `Fn` or of a closure's signature.
    Mut(Box<Ty>),
    /// A closure's own type: a named fn or a lambda. Zero or more captures.
    Clo(Box<CloRef>),
    Param(usize),
    /// A domain declared in a body: `domain V = 0..n`. Its size is a run-time value.
    Dom(usize),
    Var(usize),
    Error,
}
/// An end of a [`Ty::RangeEnds`]: a constant, or a type that names one value (a parameter whose
/// argument is a singleton range, `Mod<7>`'s `7..=7`).
#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum End {
    Lit(i64),
    Of(Box<Ty>),
}

impl End {
    fn value(&self) -> Option<i64> {
        match self {
            End::Lit(k) => Some(*k),
            End::Of(t) => t.singleton(),
        }
    }
}

impl Ty {
    /// The one value a singleton range type holds: `7` of `Mod<7>`'s argument `7..=7`.
    pub fn singleton(&self) -> Option<i64> {
        match self {
            Ty::Range(a, b) if a == b => Some(*a),
            _ => None,
        }
    }

    /// The range `lo..hi` (`..=` when `inclusive`), as `Range` when both ends are known values.
    pub fn range_ends(lo: End, hi: End, inclusive: bool) -> Ty {
        let hi_at = hi
            .value()
            .and_then(|b| if inclusive { Some(b) } else { b.checked_sub(1) });
        match (lo.value(), hi_at) {
            (Some(a), Some(b)) => Ty::Range(a, b),
            _ => Ty::RangeEnds { lo, hi, inclusive },
        }
    }

    /// R1 before instancing (`prog::holds` after it): what a value of this type holds, where
    /// `fields` gives a struct's field types. A fn value holds an address, as it captures heap
    /// values by reference (R05.10.1-2); a type parameter could be a heap type.
    pub fn holds(&self, fields: &dyn Fn(&Ty) -> Vec<Ty>, depth: usize) -> Holds {
        match self {
            Ty::Int
            | Ty::Range(..)
            | Ty::RangeOf(_)
            | Ty::RangeTo(..)
            | Ty::RangeEnds { .. }
            | Ty::Bool
            | Ty::Char
            | Ty::U64
            | Ty::I128
            | Ty::F64
            | Ty::Unit
            | Ty::Dom(_) => Holds::Nothing,
            Ty::Fn(..) | Ty::Clo(_) | Ty::Ref(_) | Ty::Lent(_) => Holds::Address,
            Ty::Struct(..) if depth < 8 => (fields(self).iter())
                .map(|f| f.holds(fields, depth + 1))
                .max()
                .unwrap_or(Holds::Nothing),
            _ => Holds::Storage,
        }
    }
    /// R1: using a value of this type copies it rather than moving it (`holds`).
    pub fn copied(&self, fields: &dyn Fn(&Ty) -> Vec<Ty>) -> bool {
        self.holds(fields, 0) != Holds::Storage
    }
    /// The value a slot of this type holds: a lent mark applies to slots, not values.
    pub fn unlent(&self) -> &Ty {
        match self {
            Ty::Lent(t) => t,
            t => t,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SetTy {
    Var(usize),
    /// The i-th implicit set parameter of the enclosing fn or struct.
    Param(usize),
    /// "Whatever the fn a call reaches adds to its `slot`-th set argument": `call_adds[i]` is
    /// that call and slot (`SetAdds`), a fn's result slots after its own sets. Resolved per instance.
    AddedAt(usize),
    /// In a signature: the k-th function type of the result. A call passes its set for it after
    /// the fn's own, so the body sees `Param(n_sets + k)`.
    RetSlot(usize),
    /// The `k`-th fn-value set in global `g`'s type, the top level's own: what a body puts in it
    /// is `SetAdds::globals`, so checking one body changes no other body's types. Closed at link.
    Global(usize, usize),
    Closed(Vec<Member>),
}

/// The closures each fn's body adds to its set parameters, and each call (`AddedAt`) that
/// takes what its callee adds: lowering hands them from the checker to link.
#[derive(Clone, Debug, Default)]
pub struct SetAdds {
    /// What `f`'s body adds to its `slot`-th set.
    pub params: std::collections::HashMap<(FnId, usize), SetTy>,
    /// `AddedAt(i)`'s call and slot.
    pub calls: Vec<(FnRef, usize)>,
    /// Whether `AddedAt(i)`'s callee may add anything there; one that cannot adds nothing.
    pub live: Vec<bool>,
    /// What the program puts in `Global(g, k)`.
    pub globals: std::collections::HashMap<(usize, usize), SetTy>,
}

/// As derived, but a set parameter prints as `SetParam(i)`, apart from a type's `Param(i)`, so
/// the parameters a fn's IR names can be read off its text.
impl std::fmt::Debug for SetTy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SetTy::Var(v) => f.debug_tuple("Var").field(v).finish(),
            SetTy::Param(i) => f.debug_tuple("SetParam").field(i).finish(),
            SetTy::AddedAt(c) => f.debug_tuple("AddedAt").field(c).finish(),
            SetTy::RetSlot(k) => f.debug_tuple("RetSlot").field(k).finish(),
            SetTy::Global(g, k) => f.debug_tuple("Global").field(g).field(k).finish(),
            SetTy::Closed(ms) => f.debug_tuple("Closed").field(ms).finish(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum Member {
    Clo(CloRef),
    Sub(SetTy),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct FnRef {
    pub decl: FnId,
    pub targs: Vec<Ty>,
    pub sets: Vec<SetTy>,
    /// Which copy of a `uses` fn: 0 is the one under its contexts' defaults; `k` the one checked
    /// under the `k`-th distinct set of context values a call gave it (R09.1.4).
    pub ctx: u32,
}

impl FnRef {
    /// The same fn, its type and set arguments each mapped.
    pub fn map(&self, ty: impl Fn(&Ty) -> Ty, set: impl Fn(&SetTy) -> SetTy) -> FnRef {
        FnRef {
            decl: self.decl,
            targs: self.targs.iter().map(ty).collect(),
            sets: self.sets.iter().map(set).collect(),
            ctx: self.ctx,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum CloRef {
    Item(FnRef),
    /// A lambda, with the type and set arguments of the fn it sits in.
    Lambda(LambdaId, Vec<Ty>, Vec<SetTy>),
}

/// Substitution of type and set parameters.
pub struct Subst<'a> {
    pub tys: &'a [Ty],
    pub sets: &'a [SetTy],
}

impl Subst<'_> {
    pub fn ty(&self, t: &Ty) -> Ty {
        match t {
            Ty::Param(i) => self.tys.get(*i).cloned().unwrap_or(Ty::Error),
            Ty::Raw(e) => Ty::Raw(Box::new(self.ty(e))),
            Ty::RangeTo(d, m, lo, k) => Ty::RangeTo(Box::new(self.ty(d)), *m, *lo, *k),
            Ty::RangeEnds { lo, hi, inclusive } => {
                let end = |e: &End| match e {
                    End::Lit(k) => End::Lit(*k),
                    End::Of(t) => End::Of(Box::new(self.ty(t))),
                };
                Ty::range_ends(end(lo), end(hi), *inclusive)
            }
            Ty::Ref(e) => Ty::Ref(Box::new(self.ty(e))),
            Ty::Lent(e) => Ty::Lent(Box::new(self.ty(e))),
            Ty::Mut(e) => Ty::Mut(Box::new(self.ty(e))),
            Ty::Struct(d, ts, ss) => Ty::Struct(
                *d,
                ts.iter().map(|t| self.ty(t)).collect(),
                ss.iter().map(|s| self.set(s)).collect(),
            ),
            Ty::Fn(ps, r, s) => Ty::Fn(
                ps.iter().map(|t| self.ty(t)).collect(),
                Box::new(self.ty(r)),
                self.set(s),
            ),
            Ty::Clo(c) => Ty::Clo(Box::new(self.clo(c))),
            other => other.clone(),
        }
    }
    pub fn set(&self, s: &SetTy) -> SetTy {
        match s {
            SetTy::Param(i) => self.sets.get(*i).cloned().unwrap_or(SetTy::Closed(vec![])),
            SetTy::RetSlot(k) => SetTy::RetSlot(*k),
            // Its call is resolved where it sits, under that fn's own parameters.
            SetTy::AddedAt(c) => SetTy::AddedAt(*c),
            // The top level's own, the same under any fn's parameters.
            SetTy::Global(g, k) => SetTy::Global(*g, *k),
            SetTy::Closed(ms) => SetTy::Closed(
                ms.iter()
                    .map(|m| match m {
                        Member::Clo(c) => Member::Clo(self.clo(c)),
                        Member::Sub(s) => Member::Sub(self.set(s)),
                    })
                    .collect(),
            ),
            SetTy::Var(v) => SetTy::Var(*v),
        }
    }
    pub fn fnref(&self, f: &FnRef) -> FnRef {
        f.map(|t| self.ty(t), |s| self.set(s))
    }
    pub fn clo(&self, c: &CloRef) -> CloRef {
        match c {
            CloRef::Item(f) => CloRef::Item(self.fnref(f)),
            CloRef::Lambda(id, ts, ss) => CloRef::Lambda(
                *id,
                ts.iter().map(|t| self.ty(t)).collect(),
                ss.iter().map(|s| self.set(s)).collect(),
            ),
        }
    }
}

/// Replaces each result slot of a signature's result type.
/// Every type parameter `t` mentions.
pub fn params_in(t: &Ty, out: &mut std::collections::BTreeSet<usize>) {
    match t {
        Ty::Param(i) => {
            out.insert(*i);
        }
        Ty::Raw(e) | Ty::Ref(e) | Ty::Lent(e) | Ty::Mut(e) | Ty::RangeTo(e, ..) => {
            params_in(e, out)
        }
        Ty::RangeEnds { lo, hi, .. } => [lo, hi].into_iter().for_each(|e| match e {
            End::Of(t) => params_in(t, out),
            End::Lit(_) => {}
        }),
        Ty::Struct(_, ts, _) => ts.iter().for_each(|t| params_in(t, out)),
        Ty::Fn(ps, r, _) => ps.iter().chain([&**r]).for_each(|t| params_in(t, out)),
        _ => {}
    }
}

pub fn map_slots(t: &Ty, f: &dyn Fn(usize) -> SetTy) -> Ty {
    let set = |s: &SetTy| match s {
        SetTy::RetSlot(k) => f(*k),
        other => other.clone(),
    };
    match t {
        Ty::Raw(e) => Ty::Raw(Box::new(map_slots(e, f))),
        Ty::Ref(e) => Ty::Ref(Box::new(map_slots(e, f))),
        Ty::Mut(e) => Ty::Mut(Box::new(map_slots(e, f))),
        Ty::Struct(d, ts, ss) => Ty::Struct(
            *d,
            ts.iter().map(|t| map_slots(t, f)).collect(),
            ss.iter().map(set).collect(),
        ),
        Ty::Fn(ps, r, s) => Ty::Fn(
            ps.iter().map(|t| map_slots(t, f)).collect(),
            Box::new(map_slots(r, f)),
            set(s),
        ),
        other => other.clone(),
    }
}

impl Ty {
    /// The core type `p` is.
    pub fn of_prim(p: cpl_contract::Prim) -> Ty {
        use cpl_contract::Prim;
        match p {
            Prim::Int => Ty::Int,
            Prim::Bool => Ty::Bool,
            Prim::Char => Ty::Char,
            Prim::U64 => Ty::U64,
            Prim::I128 => Ty::I128,
            Prim::F64 => Ty::F64,
        }
    }

    /// The core type this is, if it is one: `Int` for an integer of any range.
    pub fn prim(&self) -> Option<cpl_contract::Prim> {
        use cpl_contract::Prim;
        Some(match self {
            Ty::Int
            | Ty::Range(..)
            | Ty::RangeOf(_)
            | Ty::AnyRange(_)
            | Ty::RangeTo(..)
            | Ty::RangeEnds { .. } => Prim::Int,
            Ty::Bool => Prim::Bool,
            Ty::Char => Prim::Char,
            Ty::U64 => Prim::U64,
            Ty::I128 => Prim::I128,
            Ty::F64 => Prim::F64,
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fn_value_holds_an_address_and_is_copied() {
        let fields = |t: &Ty| match t {
            Ty::Struct(StructId(0), ..) => {
                vec![Ty::Int, Ty::Fn(vec![], Box::new(Ty::Int), SetTy::Var(0))]
            }
            Ty::Struct(..) => vec![Ty::Raw(Box::new(Ty::Int))],
            _ => vec![],
        };
        let with_fn = Ty::Struct(StructId(0), vec![], vec![]);
        let with_raw = Ty::Struct(StructId(1), vec![], vec![]);
        assert_eq!(Ty::Int.holds(&fields, 0), Holds::Nothing);
        assert_eq!(with_fn.holds(&fields, 0), Holds::Address);
        assert!(with_fn.copied(&fields));
        assert_eq!(with_raw.holds(&fields, 0), Holds::Storage);
        assert!(!with_raw.copied(&fields) && !Ty::Param(0).copied(&fields));
    }
}
