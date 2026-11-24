//! The keys of a `Set` or a `Map`. A key type whose order is the structural one keeps them in
//! a B-tree ordered by `Value`'s own `Ord`; a key type with a user `Ord` anywhere in it keeps them
//! in sorted runs ordered by that `Ord`, which runs user code, so every search there takes the one
//! compare the rest of the interpreter orders with (`Interp::less_ty_val`).

use super::Value;
use crate::Res;
use cpl_ir::Ty;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::ops::Bound;

/// A walk over keys in order, from either end, that knows its length.
pub(crate) trait Walk: DoubleEndedIterator + ExactSizeIterator {}
impl<I: DoubleEndedIterator + ExactSizeIterator> Walk for I {}

/// Whether a key of `ty` orders before another by its `Ord`.
pub(crate) type Less<'a> = dyn FnMut(&Ty, &Value, &Value) -> Res<bool> + 'a;

/// Where a bound search starts: `first_ge`, `first_gt`, `last_le`, `last_lt`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Edge {
    FirstGe,
    FirstGt,
    LastLe,
    LastLt,
}

impl Edge {
    pub(crate) fn of_mode(mode: u8) -> Edge {
        match mode {
            0 => Edge::FirstGe,
            1 => Edge::FirstGt,
            2 => Edge::LastLe,
            _ => Edge::LastLt,
        }
    }

    fn range(self, key: Value) -> (Bound<Value>, Bound<Value>) {
        match self {
            Edge::FirstGe => (Bound::Included(key), Bound::Unbounded),
            Edge::FirstGt => (Bound::Excluded(key), Bound::Unbounded),
            Edge::LastLe => (Bound::Unbounded, Bound::Included(key)),
            Edge::LastLt => (Bound::Unbounded, Bound::Excluded(key)),
        }
    }

    fn first(self) -> bool {
        matches!(self, Edge::FirstGe | Edge::FirstGt)
    }
}

/// A key type that orders by a user `Ord` somewhere inside it.
pub(crate) fn user_ordered(ty: &Ty, program: &cpl_ir::Program) -> bool {
    match ty {
        Ty::Struct(id) => {
            let s = &program.structs[*id];
            s.ord.is_some() || s.fields.iter().any(|(_, t)| user_ordered(t, program))
        }
        Ty::Tuple(ts) => ts.iter().any(|t| user_ordered(t, program)),
        Ty::Record(fs) => fs.iter().any(|(_, t)| user_ordered(t, program)),
        Ty::List(t) | Ty::Opt(t) => user_ordered(t, program),
        Ty::Array { elem, .. } => user_ordered(elem, program),
        _ => false,
    }
}

/// An entry of a sorted set or map: the key it is ordered by.
pub(crate) trait Entry: Clone {
    fn key(&self) -> &Value;
}

impl Entry for Value {
    fn key(&self) -> &Value {
        self
    }
}

impl Entry for (Value, Value) {
    fn key(&self) -> &Value {
        &self.0
    }
}

/// The longest a run grows before it splits in two.
const RUN: usize = 512;

/// A run and an offset in it; `(runs.len(), 0)` is the end.
type Pos = (usize, usize);

/// Entries sorted by key in runs of at most `RUN`, so an insert or a removal moves one run, not
/// the whole set, and a search takes \(O(\log n)\) compares, one per step.
#[derive(Clone, Debug)]
pub struct Runs<T> {
    runs: Vec<Vec<T>>,
    len: usize,
}

impl<T> Default for Runs<T> {
    fn default() -> Runs<T> {
        Runs {
            runs: Vec::new(),
            len: 0,
        }
    }
}

impl<T: Entry> Runs<T> {
    fn end(&self) -> Pos {
        (self.runs.len(), 0)
    }

    fn get(&self, (r, o): Pos) -> Option<&T> {
        self.runs.get(r)?.get(o)
    }

    fn next(&self, (r, o): Pos) -> Pos {
        if o + 1 < self.runs[r].len() {
            (r, o + 1)
        } else {
            (r + 1, 0)
        }
    }

    fn prev(&self, (r, o): Pos) -> Option<Pos> {
        if o > 0 {
            Some((r, o - 1))
        } else {
            let r = r.checked_sub(1)?;
            Some((r, self.runs[r].len() - 1))
        }
    }

    /// How many entries come before `p`.
    fn index(&self, (r, o): Pos) -> usize {
        self.runs[..r].iter().map(Vec::len).sum::<usize>() + o
    }

    /// The first position whose key isn't less than `key`.
    fn lower_bound(&self, ty: &Ty, key: &Value, less: &mut Less) -> Res<Pos> {
        let (mut lo, mut hi) = (0, self.runs.len());
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            let Some(last) = self.runs[mid].last() else {
                hi = mid;
                continue;
            };
            if less(ty, last.key(), key)? {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        let Some(run) = self.runs.get(lo) else {
            return Ok(self.end());
        };
        let (mut a, mut b) = (0, run.len());
        while a < b {
            let mid = a + (b - a) / 2;
            if less(ty, run[mid].key(), key)? {
                a = mid + 1;
            } else {
                b = mid;
            }
        }
        Ok((lo, a))
    }

    /// `Ok` at the entry whose key equals `key`, else `Err` where it would go.
    fn find(&self, ty: &Ty, key: &Value, less: &mut Less) -> Res<Result<Pos, Pos>> {
        let at = self.lower_bound(ty, key, less)?;
        Ok(match self.get(at) {
            Some(x) if !less(ty, key, x.key())? => Ok(at),
            _ => Err(at),
        })
    }

    /// The position `edge` names next to `key`.
    fn edge(&self, ty: &Ty, key: &Value, edge: Edge, less: &mut Less) -> Res<Option<Pos>> {
        let at = self.find(ty, key, less)?;
        let p = match (edge, at) {
            (Edge::FirstGe | Edge::LastLe, Ok(p)) | (Edge::FirstGe | Edge::FirstGt, Err(p)) => {
                Some(p)
            }
            (Edge::FirstGt, Ok(p)) => Some(self.next(p)),
            (Edge::LastLe, Err(p)) | (Edge::LastLt, Ok(p) | Err(p)) => self.prev(p),
        };
        Ok(p.filter(|&p| self.get(p).is_some()))
    }

    /// The positions from `lo` to `hi`.
    fn between(
        &self,
        ty: &Ty,
        lo: &Bound<Value>,
        hi: &Bound<Value>,
        less: &mut Less,
    ) -> Res<(Pos, Pos)> {
        let after = |s: &Self, k: &Value, less: &mut Less| -> Res<Pos> {
            Ok(match s.find(ty, k, less)? {
                Ok(p) => s.next(p),
                Err(p) => p,
            })
        };
        let a = match lo {
            Bound::Unbounded => (0, 0),
            Bound::Included(k) => self.lower_bound(ty, k, less)?,
            Bound::Excluded(k) => after(self, k, less)?,
        };
        let b = match hi {
            Bound::Unbounded => self.end(),
            Bound::Included(k) => after(self, k, less)?,
            Bound::Excluded(k) => self.lower_bound(ty, k, less)?,
        };
        Ok((a, b.max(a)))
    }

    fn insert_at(&mut self, (r, o): Pos, x: T) {
        self.len += 1;
        if self.runs.is_empty() {
            self.runs.push(vec![x]);
            return;
        }
        let (r, o) = if r == self.runs.len() {
            (r - 1, self.runs[r - 1].len())
        } else {
            (r, o)
        };
        self.runs[r].insert(o, x);
        if self.runs[r].len() > RUN {
            let tail = self.runs[r].split_off(RUN / 2);
            self.runs.insert(r + 1, tail);
        }
    }

    fn remove_at(&mut self, (r, o): Pos) -> T {
        self.len -= 1;
        let x = self.runs[r].remove(o);
        if self.runs[r].is_empty() {
            self.runs.remove(r);
        }
        x
    }

    fn walk(&self, a: Pos, b: Pos) -> RunWalk<'_, T> {
        let len = if a < b {
            self.index(b) - self.index(a)
        } else {
            0
        };
        RunWalk {
            runs: &self.runs,
            front: a,
            back: b,
            len,
        }
    }

    fn first(&self) -> Option<&T> {
        self.runs.first()?.first()
    }

    fn last(&self) -> Option<&T> {
        self.runs.last()?.last()
    }

    fn pop_first(&mut self) -> Option<T> {
        (self.len > 0).then(|| self.remove_at((0, 0)))
    }

    fn pop_last(&mut self) -> Option<T> {
        let r = self.runs.len().checked_sub(1)?;
        Some(self.remove_at((r, self.runs[r].len() - 1)))
    }

    fn clear(&mut self) {
        self.runs.clear();
        self.len = 0;
    }
}

/// The entries of runs from `front` up to `back`.
pub struct RunWalk<'a, T> {
    runs: &'a [Vec<T>],
    front: Pos,
    back: Pos,
    len: usize,
}

impl<'a, T> Iterator for RunWalk<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        if self.len == 0 {
            return None;
        }
        let (r, o) = self.front;
        let x = &self.runs[r][o];
        self.front = if o + 1 < self.runs[r].len() {
            (r, o + 1)
        } else {
            (r + 1, 0)
        };
        self.len -= 1;
        Some(x)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<T> DoubleEndedIterator for RunWalk<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        let (mut r, mut o) = self.back;
        if o == 0 {
            r -= 1;
            o = self.runs[r].len();
        }
        o -= 1;
        self.back = (r, o);
        self.len -= 1;
        Some(&self.runs[r][o])
    }
}

impl<T> ExactSizeIterator for RunWalk<'_, T> {}

#[derive(Clone, Debug)]
pub enum KeySet {
    /// Keys mapped to `Unit`, so a set shares the map's B-tree code.
    Tree(BTreeMap<Value, Value>),
    /// Sorted by the key type's user `Ord`.
    Sorted(Ty, Runs<Value>),
}

impl Default for KeySet {
    fn default() -> KeySet {
        KeySet::Tree(BTreeMap::new())
    }
}

impl KeySet {
    /// An empty set over keys of `ty`.
    pub(crate) fn of(ty: &Ty, program: &cpl_ir::Program) -> KeySet {
        if user_ordered(ty, program) {
            KeySet::Sorted(ty.clone(), Runs::default())
        } else {
            KeySet::default()
        }
    }

    /// An empty set that orders its keys as this one does.
    pub(crate) fn empty_like(&self) -> KeySet {
        match self {
            KeySet::Tree(_) => KeySet::default(),
            KeySet::Sorted(ty, _) => KeySet::Sorted(ty.clone(), Runs::default()),
        }
    }

    pub(crate) fn is_sorted_by_user(&self) -> bool {
        matches!(self, KeySet::Sorted(..))
    }

    pub(crate) fn len(&self) -> usize {
        match self {
            KeySet::Tree(t) => t.len(),
            KeySet::Sorted(_, v) => v.len,
        }
    }

    pub(crate) fn iter(&self) -> Box<dyn Walk<Item = &Value> + '_> {
        match self {
            KeySet::Tree(t) => Box::new(t.keys()),
            KeySet::Sorted(_, v) => Box::new(v.walk((0, 0), v.end())),
        }
    }

    pub(crate) fn first(&self) -> Option<&Value> {
        match self {
            KeySet::Tree(t) => t.first_key_value().map(|(k, _)| k),
            KeySet::Sorted(_, v) => v.first(),
        }
    }

    pub(crate) fn last(&self) -> Option<&Value> {
        match self {
            KeySet::Tree(t) => t.last_key_value().map(|(k, _)| k),
            KeySet::Sorted(_, v) => v.last(),
        }
    }

    pub(crate) fn pop_first(&mut self) -> Option<Value> {
        match self {
            KeySet::Tree(t) => t.pop_first().map(|(k, _)| k),
            KeySet::Sorted(_, v) => v.pop_first(),
        }
    }

    pub(crate) fn pop_last(&mut self) -> Option<Value> {
        match self {
            KeySet::Tree(t) => t.pop_last().map(|(k, _)| k),
            KeySet::Sorted(_, v) => v.pop_last(),
        }
    }

    pub(crate) fn clear(&mut self) {
        match self {
            KeySet::Tree(t) => t.clear(),
            KeySet::Sorted(_, v) => v.clear(),
        }
    }

    pub(crate) fn contains(&self, key: &Value, less: &mut Less) -> Res<bool> {
        match self {
            KeySet::Tree(t) => Ok(t.contains_key(key)),
            KeySet::Sorted(ty, v) => Ok(v.find(ty, key, less)?.is_ok()),
        }
    }

    /// Adds `key`; whether it was new.
    pub(crate) fn insert(&mut self, key: Value, less: &mut Less) -> Res<bool> {
        match self {
            KeySet::Tree(t) => Ok(t.insert(key, Value::Unit).is_none()),
            KeySet::Sorted(ty, v) => Ok(match v.find(ty, &key, less)? {
                Ok(_) => false,
                Err(p) => {
                    v.insert_at(p, key);
                    true
                }
            }),
        }
    }

    /// Takes `key` out; whether it was there.
    pub(crate) fn remove(&mut self, key: &Value, less: &mut Less) -> Res<bool> {
        match self {
            KeySet::Tree(t) => Ok(t.remove(key).is_some()),
            KeySet::Sorted(ty, v) => Ok(match v.find(ty, key, less)? {
                Ok(p) => {
                    v.remove_at(p);
                    true
                }
                Err(_) => false,
            }),
        }
    }

    /// The keys from `lo` to `hi`, in order.
    pub(crate) fn range(
        &self,
        lo: Bound<Value>,
        hi: Bound<Value>,
        less: &mut Less,
    ) -> Res<Box<dyn DoubleEndedIterator<Item = &Value> + '_>> {
        match self {
            KeySet::Tree(t) => Ok(Box::new(t.range((lo, hi)).map(|(k, _)| k))),
            KeySet::Sorted(ty, v) => {
                let (a, b) = v.between(ty, &lo, &hi, less)?;
                Ok(Box::new(v.walk(a, b)))
            }
        }
    }

    /// The key `edge` names next to `key`.
    pub(crate) fn edge(&self, key: Value, edge: Edge, less: &mut Less) -> Res<Option<&Value>> {
        match self {
            KeySet::Tree(t) => {
                let mut found = t.range(edge.range(key));
                Ok(if edge.first() {
                    found.next()
                } else {
                    found.next_back()
                }
                .map(|(k, _)| k))
            }
            KeySet::Sorted(ty, v) => Ok(v.edge(ty, &key, edge, less)?.and_then(|p| v.get(p))),
        }
    }
}

#[derive(Clone, Debug)]
pub enum KeyMap {
    Tree(BTreeMap<Value, Value>),
    /// Entries sorted by the key type's user `Ord`.
    Sorted(Ty, Runs<(Value, Value)>),
}

impl Default for KeyMap {
    fn default() -> KeyMap {
        KeyMap::Tree(BTreeMap::new())
    }
}

impl KeyMap {
    /// An empty map over keys of `ty`.
    pub(crate) fn of(ty: &Ty, program: &cpl_ir::Program) -> KeyMap {
        if user_ordered(ty, program) {
            KeyMap::Sorted(ty.clone(), Runs::default())
        } else {
            KeyMap::default()
        }
    }

    pub(crate) fn empty_like(&self) -> KeyMap {
        match self {
            KeyMap::Tree(_) => KeyMap::default(),
            KeyMap::Sorted(ty, _) => KeyMap::Sorted(ty.clone(), Runs::default()),
        }
    }

    pub(crate) fn is_sorted_by_user(&self) -> bool {
        matches!(self, KeyMap::Sorted(..))
    }

    pub(crate) fn len(&self) -> usize {
        match self {
            KeyMap::Tree(t) => t.len(),
            KeyMap::Sorted(_, v) => v.len,
        }
    }

    pub(crate) fn iter(&self) -> Box<dyn Walk<Item = (&Value, &Value)> + '_> {
        match self {
            KeyMap::Tree(t) => Box::new(t.iter()),
            KeyMap::Sorted(_, v) => Box::new(v.walk((0, 0), v.end()).map(|(k, x)| (k, x))),
        }
    }

    pub(crate) fn pop_first(&mut self) -> Option<(Value, Value)> {
        match self {
            KeyMap::Tree(t) => t.pop_first(),
            KeyMap::Sorted(_, v) => v.pop_first(),
        }
    }

    pub(crate) fn pop_last(&mut self) -> Option<(Value, Value)> {
        match self {
            KeyMap::Tree(t) => t.pop_last(),
            KeyMap::Sorted(_, v) => v.pop_last(),
        }
    }

    pub(crate) fn clear(&mut self) {
        match self {
            KeyMap::Tree(t) => t.clear(),
            KeyMap::Sorted(_, v) => v.clear(),
        }
    }

    pub(crate) fn contains_key(&self, key: &Value, less: &mut Less) -> Res<bool> {
        Ok(self.get(key, less)?.is_some())
    }

    pub(crate) fn get(&self, key: &Value, less: &mut Less) -> Res<Option<&Value>> {
        match self {
            KeyMap::Tree(t) => Ok(t.get(key)),
            KeyMap::Sorted(ty, v) => Ok(v
                .find(ty, key, less)?
                .ok()
                .and_then(|p| v.get(p))
                .map(|e| &e.1)),
        }
    }

    pub(crate) fn get_mut(&mut self, key: &Value, less: &mut Less) -> Res<Option<&mut Value>> {
        match self {
            KeyMap::Tree(t) => Ok(t.get_mut(key)),
            KeyMap::Sorted(ty, v) => Ok(match v.find(ty, key, less)? {
                Ok((r, o)) => Some(&mut v.runs[r][o].1),
                Err(_) => None,
            }),
        }
    }

    /// Sets `key` to `value`; the value it had.
    pub(crate) fn insert(
        &mut self,
        key: Value,
        value: Value,
        less: &mut Less,
    ) -> Res<Option<Value>> {
        match self {
            KeyMap::Tree(t) => Ok(t.insert(key, value)),
            KeyMap::Sorted(ty, v) => Ok(match v.find(ty, &key, less)? {
                Ok((r, o)) => Some(std::mem::replace(&mut v.runs[r][o].1, value)),
                Err(p) => {
                    v.insert_at(p, (key, value));
                    None
                }
            }),
        }
    }

    pub(crate) fn remove(&mut self, key: &Value, less: &mut Less) -> Res<Option<Value>> {
        match self {
            KeyMap::Tree(t) => Ok(t.remove(key)),
            KeyMap::Sorted(ty, v) => Ok(match v.find(ty, key, less)? {
                Ok(p) => Some(v.remove_at(p).1),
                Err(_) => None,
            }),
        }
    }

    /// The entries from `lo` to `hi`, in key order.
    pub(crate) fn range(
        &self,
        lo: Bound<Value>,
        hi: Bound<Value>,
        less: &mut Less,
    ) -> Res<Box<dyn DoubleEndedIterator<Item = (&Value, &Value)> + '_>> {
        match self {
            KeyMap::Tree(t) => Ok(Box::new(t.range((lo, hi)))),
            KeyMap::Sorted(ty, v) => {
                let (a, b) = v.between(ty, &lo, &hi, less)?;
                Ok(Box::new(v.walk(a, b).map(|(k, x)| (k, x))))
            }
        }
    }

    /// The entry `edge` names next to `key`.
    pub(crate) fn edge(
        &self,
        key: Value,
        edge: Edge,
        less: &mut Less,
    ) -> Res<Option<(&Value, &Value)>> {
        match self {
            KeyMap::Tree(t) => {
                let mut found = t.range(edge.range(key));
                Ok(if edge.first() {
                    found.next()
                } else {
                    found.next_back()
                })
            }
            KeyMap::Sorted(ty, v) => Ok(v
                .edge(ty, &key, edge, less)?
                .and_then(|p| v.get(p))
                .map(|(k, x)| (k, x))),
        }
    }
}

macro_rules! by_contents {
    ($t:ty) => {
        impl PartialEq for $t {
            fn eq(&self, other: &$t) -> bool {
                self.cmp(other) == Ordering::Equal
            }
        }
        impl Eq for $t {}
        impl PartialOrd for $t {
            fn partial_cmp(&self, other: &$t) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }
        impl Ord for $t {
            fn cmp(&self, other: &$t) -> Ordering {
                self.iter().cmp(other.iter())
            }
        }
    };
}
by_contents!(KeySet);
by_contents!(KeyMap);

/// The compare a site passes that the compiler only reaches with B-tree keys (the VM's own
/// ops, emitted for key types without a user `Ord`): a sorted set there is a compiler bug,
/// never a silent structural order.
pub(crate) fn tree_only(span: cpl_diag::Span) -> impl FnMut(&Ty, &Value, &Value) -> Res<bool> {
    move |_, _, _| crate::bug(span, "this value can't be changed that way")
}
