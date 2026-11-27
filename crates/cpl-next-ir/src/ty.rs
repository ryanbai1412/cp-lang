//! Contract "facts/types": one type representation for every layer.
//! Var forms exist only inside the checker; `zonk` removes them.

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
    Bool,
    /// An ASCII character, `0..=127`, one byte.
    Char,
    /// `0..=2^64-1`, wrapping (R02.1).
    U64,
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
    /// A closure's own type: a named fn or a lambda. Zero or more captures.
    Clo(Box<CloRef>),
    Param(usize),
    /// A domain declared in a body: `domain V = 0..n`. Its size is a run-time value.
    Dom(usize),
    Var(usize),
    Error,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum SetTy {
    Var(usize),
    /// The i-th implicit set parameter of the enclosing fn or struct.
    Param(usize),
    /// "Whatever set f<..> returns": resolved per instance.
    RetOf(Box<FnRef>, usize),
    /// In a signature: the k-th function type of the result. Callers see `RetOf(f, k)`.
    RetSlot(usize),
    Closed(Vec<Member>),
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
            Ty::Ref(e) => Ty::Ref(Box::new(self.ty(e))),
            Ty::Lent(e) => Ty::Lent(Box::new(self.ty(e))),
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
            SetTy::RetOf(f, k) => SetTy::RetOf(Box::new(self.fnref(f)), *k),
            SetTy::RetSlot(k) => SetTy::RetSlot(*k),
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
        FnRef {
            decl: f.decl,
            targs: f.targs.iter().map(|t| self.ty(t)).collect(),
            sets: f.sets.iter().map(|s| self.set(s)).collect(),
        }
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
pub fn map_slots(t: &Ty, f: &dyn Fn(usize) -> SetTy) -> Ty {
    let set = |s: &SetTy| match s {
        SetTy::RetSlot(k) => f(*k),
        other => other.clone(),
    };
    match t {
        Ty::Raw(e) => Ty::Raw(Box::new(map_slots(e, f))),
        Ty::Ref(e) => Ty::Ref(Box::new(map_slots(e, f))),
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
