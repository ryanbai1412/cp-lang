//! The keys of a `Set` or a `Map`. A key type whose order is the structural one keeps them in
//! a B-tree ordered by `Value`'s own `Ord`; a key type with a user `Ord` anywhere in it keeps them
//! in a list sorted by that `Ord`, which runs user code, so every search there takes the one
//! compare function (`Interp::cmp_ty_val`) the rest of the interpreter orders with.

use super::Value;
use crate::Res;
use cpl_ir::Ty;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Bound;

/// A walk over keys in order, from either end, that knows its length.
pub(crate) trait Walk: DoubleEndedIterator + ExactSizeIterator {}
impl<I: DoubleEndedIterator + ExactSizeIterator> Walk for I {}

/// Compares two keys of `ty` by its `Ord`.
pub(crate) type Cmp<'a> = dyn FnMut(&Ty, &Value, &Value) -> Res<Ordering> + 'a;

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

/// Where `key` sits in `keys`, sorted by `ty`'s `Ord`: `Ok(i)` holds an equal key at `i`,
/// `Err(i)` is where it would go.
fn search(ty: &Ty, keys: &[Value], key: &Value, cmp: &mut Cmp) -> Res<Result<usize, usize>> {
    let (mut lo, mut hi) = (0, keys.len());
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        match cmp(ty, &keys[mid], key)? {
            Ordering::Less => lo = mid + 1,
            Ordering::Greater => hi = mid,
            Ordering::Equal => return Ok(Ok(mid)),
        }
    }
    Ok(Err(lo))
}

/// The position `edge` names next to `key`, if there is one.
fn edge_at(ty: &Ty, keys: &[Value], key: &Value, edge: Edge, cmp: &mut Cmp) -> Res<Option<usize>> {
    let at = search(ty, keys, key, cmp)?;
    let i = match (edge, at) {
        (Edge::FirstGe | Edge::LastLe, Ok(i)) => Some(i),
        (Edge::FirstGt, Ok(i)) => Some(i + 1),
        (Edge::FirstGe | Edge::FirstGt, Err(i)) => Some(i),
        (Edge::LastLe, Err(i)) | (Edge::LastLt, Ok(i) | Err(i)) => i.checked_sub(1),
    };
    Ok(i.filter(|&i| i < keys.len()))
}

/// The positions from `lo` to `hi` in `keys`, sorted by `ty`'s `Ord`.
fn between(
    ty: &Ty,
    keys: &[Value],
    lo: &Bound<Value>,
    hi: &Bound<Value>,
    cmp: &mut Cmp,
) -> Res<(usize, usize)> {
    let a = match lo {
        Bound::Unbounded => 0,
        Bound::Included(k) => search(ty, keys, k, cmp)?.unwrap_or_else(|i| i),
        Bound::Excluded(k) => search(ty, keys, k, cmp)?.map_or_else(|i| i, |i| i + 1),
    };
    let b = match hi {
        Bound::Unbounded => keys.len(),
        Bound::Included(k) => search(ty, keys, k, cmp)?.map_or_else(|i| i, |i| i + 1),
        Bound::Excluded(k) => search(ty, keys, k, cmp)?.unwrap_or_else(|i| i),
    };
    Ok((a, b.max(a)))
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

#[derive(Clone, Debug)]
pub enum KeySet {
    Tree(BTreeSet<Value>),
    /// Sorted by the key type's user `Ord`.
    Sorted(Ty, Vec<Value>),
}

impl Default for KeySet {
    fn default() -> KeySet {
        KeySet::Tree(BTreeSet::new())
    }
}

impl KeySet {
    /// An empty set over keys of `ty`.
    pub(crate) fn of(ty: &Ty, program: &cpl_ir::Program) -> KeySet {
        if user_ordered(ty, program) {
            KeySet::Sorted(ty.clone(), Vec::new())
        } else {
            KeySet::default()
        }
    }

    /// An empty set that orders its keys as this one does.
    pub(crate) fn empty_like(&self) -> KeySet {
        match self {
            KeySet::Tree(_) => KeySet::default(),
            KeySet::Sorted(ty, _) => KeySet::Sorted(ty.clone(), Vec::new()),
        }
    }

    pub(crate) fn is_sorted_by_user(&self) -> bool {
        matches!(self, KeySet::Sorted(..))
    }

    pub(crate) fn len(&self) -> usize {
        match self {
            KeySet::Tree(t) => t.len(),
            KeySet::Sorted(_, v) => v.len(),
        }
    }

    pub(crate) fn iter(&self) -> Box<dyn Walk<Item = &Value> + '_> {
        match self {
            KeySet::Tree(t) => Box::new(t.iter()),
            KeySet::Sorted(_, v) => Box::new(v.iter()),
        }
    }

    pub(crate) fn first(&self) -> Option<&Value> {
        match self {
            KeySet::Tree(t) => t.first(),
            KeySet::Sorted(_, v) => v.first(),
        }
    }

    pub(crate) fn last(&self) -> Option<&Value> {
        match self {
            KeySet::Tree(t) => t.last(),
            KeySet::Sorted(_, v) => v.last(),
        }
    }

    pub(crate) fn pop_first(&mut self) -> Option<Value> {
        match self {
            KeySet::Tree(t) => t.pop_first(),
            KeySet::Sorted(_, v) => (!v.is_empty()).then(|| v.remove(0)),
        }
    }

    pub(crate) fn pop_last(&mut self) -> Option<Value> {
        match self {
            KeySet::Tree(t) => t.pop_last(),
            KeySet::Sorted(_, v) => v.pop(),
        }
    }

    pub(crate) fn clear(&mut self) {
        match self {
            KeySet::Tree(t) => t.clear(),
            KeySet::Sorted(_, v) => v.clear(),
        }
    }

    pub(crate) fn contains(&self, key: &Value, cmp: &mut Cmp) -> Res<bool> {
        match self {
            KeySet::Tree(t) => Ok(t.contains(key)),
            KeySet::Sorted(ty, v) => Ok(search(ty, v, key, cmp)?.is_ok()),
        }
    }

    /// Adds `key`; whether it was new.
    pub(crate) fn insert(&mut self, key: Value, cmp: &mut Cmp) -> Res<bool> {
        match self {
            KeySet::Tree(t) => Ok(t.insert(key)),
            KeySet::Sorted(ty, v) => Ok(match search(ty, v, &key, cmp)? {
                Ok(_) => false,
                Err(i) => {
                    v.insert(i, key);
                    true
                }
            }),
        }
    }

    /// Takes `key` out; whether it was there.
    pub(crate) fn remove(&mut self, key: &Value, cmp: &mut Cmp) -> Res<bool> {
        match self {
            KeySet::Tree(t) => Ok(t.remove(key)),
            KeySet::Sorted(ty, v) => Ok(match search(ty, v, key, cmp)? {
                Ok(i) => {
                    v.remove(i);
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
        cmp: &mut Cmp,
    ) -> Res<Box<dyn DoubleEndedIterator<Item = &Value> + '_>> {
        match self {
            KeySet::Tree(t) => Ok(Box::new(t.range((lo, hi)))),
            KeySet::Sorted(ty, v) => {
                let (a, b) = between(ty, v, &lo, &hi, cmp)?;
                Ok(Box::new(v[a..b].iter()))
            }
        }
    }

    /// The key `edge` names next to `key`.
    pub(crate) fn edge(&self, key: Value, edge: Edge, cmp: &mut Cmp) -> Res<Option<&Value>> {
        match self {
            KeySet::Tree(t) => {
                let mut found = t.range(edge.range(key));
                Ok(if edge.first() {
                    found.next()
                } else {
                    found.next_back()
                })
            }
            KeySet::Sorted(ty, v) => Ok(edge_at(ty, v, &key, edge, cmp)?.map(|i| &v[i])),
        }
    }
}

#[derive(Clone, Debug)]
pub enum KeyMap {
    Tree(BTreeMap<Value, Value>),
    /// Keys sorted by the key type's user `Ord`, each beside its value.
    Sorted(Ty, Vec<Value>, Vec<Value>),
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
            KeyMap::Sorted(ty.clone(), Vec::new(), Vec::new())
        } else {
            KeyMap::default()
        }
    }

    pub(crate) fn empty_like(&self) -> KeyMap {
        match self {
            KeyMap::Tree(_) => KeyMap::default(),
            KeyMap::Sorted(ty, ..) => KeyMap::Sorted(ty.clone(), Vec::new(), Vec::new()),
        }
    }

    pub(crate) fn is_sorted_by_user(&self) -> bool {
        matches!(self, KeyMap::Sorted(..))
    }

    pub(crate) fn len(&self) -> usize {
        match self {
            KeyMap::Tree(t) => t.len(),
            KeyMap::Sorted(_, k, _) => k.len(),
        }
    }

    pub(crate) fn iter(&self) -> Box<dyn Walk<Item = (&Value, &Value)> + '_> {
        match self {
            KeyMap::Tree(t) => Box::new(t.iter()),
            KeyMap::Sorted(_, k, v) => Box::new(k.iter().zip(v.iter())),
        }
    }

    pub(crate) fn pop_first(&mut self) -> Option<(Value, Value)> {
        match self {
            KeyMap::Tree(t) => t.pop_first(),
            KeyMap::Sorted(_, k, v) => (!k.is_empty()).then(|| (k.remove(0), v.remove(0))),
        }
    }

    pub(crate) fn pop_last(&mut self) -> Option<(Value, Value)> {
        match self {
            KeyMap::Tree(t) => t.pop_last(),
            KeyMap::Sorted(_, k, v) => k.pop().zip(v.pop()),
        }
    }

    pub(crate) fn clear(&mut self) {
        match self {
            KeyMap::Tree(t) => t.clear(),
            KeyMap::Sorted(_, k, v) => {
                k.clear();
                v.clear();
            }
        }
    }

    pub(crate) fn contains_key(&self, key: &Value, cmp: &mut Cmp) -> Res<bool> {
        Ok(self.get(key, cmp)?.is_some())
    }

    pub(crate) fn get(&self, key: &Value, cmp: &mut Cmp) -> Res<Option<&Value>> {
        match self {
            KeyMap::Tree(t) => Ok(t.get(key)),
            KeyMap::Sorted(ty, k, v) => Ok(search(ty, k, key, cmp)?.ok().map(|i| &v[i])),
        }
    }

    pub(crate) fn get_mut(&mut self, key: &Value, cmp: &mut Cmp) -> Res<Option<&mut Value>> {
        match self {
            KeyMap::Tree(t) => Ok(t.get_mut(key)),
            KeyMap::Sorted(ty, k, v) => Ok(search(ty, k, key, cmp)?.ok().map(|i| &mut v[i])),
        }
    }

    /// Sets `key` to `value`; the value it had.
    pub(crate) fn insert(&mut self, key: Value, value: Value, cmp: &mut Cmp) -> Res<Option<Value>> {
        match self {
            KeyMap::Tree(t) => Ok(t.insert(key, value)),
            KeyMap::Sorted(ty, k, v) => Ok(match search(ty, k, &key, cmp)? {
                Ok(i) => Some(std::mem::replace(&mut v[i], value)),
                Err(i) => {
                    k.insert(i, key);
                    v.insert(i, value);
                    None
                }
            }),
        }
    }

    pub(crate) fn remove(&mut self, key: &Value, cmp: &mut Cmp) -> Res<Option<Value>> {
        match self {
            KeyMap::Tree(t) => Ok(t.remove(key)),
            KeyMap::Sorted(ty, k, v) => Ok(match search(ty, k, key, cmp)? {
                Ok(i) => {
                    k.remove(i);
                    Some(v.remove(i))
                }
                Err(_) => None,
            }),
        }
    }

    /// The entries from `lo` to `hi`, in key order.
    pub(crate) fn range(
        &self,
        lo: Bound<Value>,
        hi: Bound<Value>,
        cmp: &mut Cmp,
    ) -> Res<Box<dyn DoubleEndedIterator<Item = (&Value, &Value)> + '_>> {
        match self {
            KeyMap::Tree(t) => Ok(Box::new(t.range((lo, hi)))),
            KeyMap::Sorted(ty, k, v) => {
                let (a, b) = between(ty, k, &lo, &hi, cmp)?;
                Ok(Box::new(k[a..b].iter().zip(v[a..b].iter())))
            }
        }
    }

    /// The entry `edge` names next to `key`.
    pub(crate) fn edge(
        &self,
        key: Value,
        edge: Edge,
        cmp: &mut Cmp,
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
            KeyMap::Sorted(ty, k, v) => {
                Ok(edge_at(ty, k, &key, edge, cmp)?.map(|i| (&k[i], &v[i])))
            }
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
pub(crate) fn tree_only(span: cpl_diag::Span) -> impl FnMut(&Ty, &Value, &Value) -> Res<Ordering> {
    move |_, _, _| {
        crate::trap(
            span,
            cpl_diag::msgs::interp::INTERNAL_THIS_VALUE_CAN_BE.at(&[]),
        )
    }
}
