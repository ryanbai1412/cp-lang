//! Typed cell storage shared by arrays and lists: cells of one scalar kind
//! are stored packed, as the emitted C++ stores them, and boxed into a
//! [`Value`] only when read.

use super::{shape, OptVal, TupleVal, Value};
use std::cmp::Ordering;
use std::rc::Rc;

/// Kind of each scalar field inside a packed [`Cells::Rows`] row.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum SmallFieldKind {
    Int,
    OneIntStruct,
    Bool,
    Char,
}

impl SmallFieldKind {
    #[inline]
    pub fn try_unbox(v: &Value) -> Option<(Self, i64)> {
        match v {
            Value::Int(x) => Some((Self::Int, *x)),
            Value::Tuple(TupleVal::OneInt(x)) => Some((Self::OneIntStruct, *x)),
            Value::Bool(b) => Some((Self::Bool, i64::from(*b))),
            Value::Char(c) => Some((Self::Char, i64::from(*c))),
            _ => None,
        }
    }

    pub(super) fn of_bits(bits: i64) -> Self {
        match bits {
            0 => Self::Int,
            1 => Self::OneIntStruct,
            2 => Self::Bool,
            _ => Self::Char,
        }
    }

    #[inline]
    pub fn box_val(self, v: i64) -> Value {
        match self {
            Self::Int => Value::Int(v),
            Self::OneIntStruct => Value::Tuple(TupleVal::OneInt(v)),
            Self::Bool => Value::Bool(v != 0),
            Self::Char => Value::Char(v as u8),
        }
    }
}

/// An enum value whose payload is at most three scalar fields, packed as
/// a [`Cells::Rows`] row keeps it: its tag, arity and field kinds in the first
/// word, its fields in the rest.
#[inline]
pub(super) fn try_unbox_small_enum(v: &Value) -> Option<[i64; 4]> {
    let Value::Enum(tag, payload) = v else {
        return None;
    };
    if payload.len() > 3 {
        return None;
    }
    let mut word = [0i64; 4];
    let mut head = i64::from(*tag) | (payload.len() as i64) << 32;
    for (i, f) in payload.iter().enumerate() {
        let (k, x) = SmallFieldKind::try_unbox(f)?;
        head |= (k as i64) << (34 + 2 * i);
        word[i + 1] = x;
    }
    word[0] = head;
    Some(word)
}

#[inline]
pub(super) fn box_small_enum(word: &[i64]) -> Value {
    let head = word[0];
    let arity = ((head >> 32) & 3) as usize;
    let mut payload = Vec::with_capacity(arity);
    for i in 0..arity {
        payload.push(SmallFieldKind::of_bits((head >> (34 + 2 * i)) & 3).box_val(word[i + 1]));
    }
    Value::Enum(head as u32, Rc::new(payload))
}

/// Typed flat backing storage for array and list cells so primitive ones
/// (`Int`, `Pos`, `Bound`, 1-field `Int` structs like `Mod<P>`, `Bool`,
/// `Char`, `Float`, and tuples, structs and small enums of them) avoid 24-byte
/// `Value` tagging and `Rc`/`Clone`/`Drop` overhead.
#[derive(Clone)]
pub(crate) enum Cells {
    Int(Vec<i64>),
    OneIntStruct(Vec<i64>),
    OptInt(Vec<Option<i64>>),
    Bool(Vec<u8>),
    Char(Vec<u8>),
    /// Each float as [`float_key`], so `<` and `==` on cells are the integers'.
    Float(Vec<i64>),
    /// Rows of one shape of tuples, small enums and an optional over
    /// scalars, `stride` words each; see [`shape`].
    Rows {
        stride: u8,
        shape: u64,
        data: Vec<i64>,
    },
    Values(Vec<Value>),
}

/// The `i64` whose order is `f64::total_cmp`'s; its own inverse.
#[inline]
fn float_key(b: i64) -> i64 {
    b ^ (((b >> 63) as u64) >> 1) as i64
}

#[inline]
fn key_of(f: f64) -> i64 {
    float_key(f.to_bits() as i64)
}

#[inline]
fn float_of(k: i64) -> Value {
    Value::Float(f64::from_bits(float_key(k) as u64))
}

impl Cells {
    pub fn filled(cells: usize, fill: Value) -> Cells {
        let mut out = Cells::one(fill.clone());
        out.resize(cells, fill);
        out
    }

    /// One cell, packed by its kind.
    fn one(v: Value) -> Cells {
        let mut out = match &v {
            Value::Int(_) => Cells::Int(Vec::new()),
            Value::Tuple(TupleVal::OneInt(_)) => Cells::OneIntStruct(Vec::new()),
            Value::Opt(OptVal::None | OptVal::Int(_)) => Cells::OptInt(Vec::new()),
            Value::Bool(_) => Cells::Bool(Vec::new()),
            Value::Char(_) => Cells::Char(Vec::new()),
            Value::Float(_) => Cells::Float(Vec::new()),
            other => match shape::of(other) {
                Some((shape, stride)) => Cells::Rows {
                    stride,
                    shape,
                    data: Vec::new(),
                },
                None => return Cells::Values(vec![v]),
            },
        };
        out.push(v);
        out
    }

    /// Packs `values` by their element kind when they share one.
    pub fn from_values(values: Vec<Value>) -> Cells {
        let mut cells = Cells::default();
        let n = values.len();
        for (i, v) in values.into_iter().enumerate() {
            cells.push(v);
            if i == 0 {
                cells.reserve(n - 1);
            }
        }
        cells
    }

    #[cfg_attr(not(target_family = "wasm"), inline)]
    pub fn len(&self) -> usize {
        match self {
            Cells::Int(v) | Cells::OneIntStruct(v) | Cells::Float(v) => v.len(),
            Cells::OptInt(v) => v.len(),
            Cells::Bool(v) | Cells::Char(v) => v.len(),
            Cells::Rows { stride, data, .. } => data.len() / (*stride as usize),
            Cells::Values(v) => v.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn get_int(&self, idx: usize) -> Option<i64> {
        match self {
            Cells::Int(v) => v.get(idx).copied(),
            _ => match self.get(idx)? {
                Value::Int(v) => Some(v),
                _ => None,
            },
        }
    }

    pub fn get(&self, idx: usize) -> Option<Value> {
        (idx < self.len()).then(|| self.get_unchecked_val(idx))
    }

    #[inline]
    pub fn get_unchecked_val(&self, idx: usize) -> Value {
        match self {
            Cells::Int(v) => Value::Int(v[idx]),
            Cells::OneIntStruct(v) => Value::Tuple(TupleVal::OneInt(v[idx])),
            Cells::OptInt(v) => match v[idx] {
                Some(x) => Value::Opt(OptVal::Int(x)),
                None => Value::Opt(OptVal::None),
            },
            Cells::Bool(v) => Value::Bool(v[idx] != 0),
            Cells::Char(v) => Value::Char(v[idx]),
            Cells::Float(v) => float_of(v[idx]),
            Cells::Rows {
                stride,
                shape,
                data,
            } => {
                let s = *stride as usize;
                shape::unpack(*shape, &data[idx * s..idx * s + s])
            }
            Cells::Values(v) => v[idx].clone(),
        }
    }

    /// The words of a store of tuples whose fields are a scalar word each,
    /// and the fields of each row.
    pub fn flat_rows(&self) -> Option<(usize, &[i64])> {
        match self {
            Cells::Rows { shape, data, .. } => Some((shape::flat_len(*shape)?, data)),
            _ => None,
        }
    }

    /// [`Cells::flat_rows`], to write.
    pub fn flat_rows_mut(&mut self) -> Option<(usize, &mut [i64])> {
        match self {
            Cells::Rows { shape, data, .. } => Some((shape::flat_len(*shape)?, data)),
            _ => None,
        }
    }

    #[inline]
    pub fn get_small_tuple(&self, idx: usize, len: usize, out: &mut [i64]) {
        match self {
            Cells::Rows {
                stride,
                shape,
                data,
            } if *stride as usize == len && shape::flat_len(*shape) == Some(len) => {
                let off = idx * len;
                out[..len].copy_from_slice(&data[off..off + len]);
            }
            _ => {
                if let Some(Value::Tuple(TupleVal::Boxed(fs))) = self.get(idx) {
                    for (i, f) in fs.iter().take(len).enumerate() {
                        out[i] = SmallFieldKind::try_unbox(f).map_or(0, |(_, x)| x);
                    }
                }
            }
        }
    }

    #[inline]
    pub fn set_small_tuple(
        &mut self,
        idx: usize,
        len: usize,
        vals: &[i64],
        kinds: &[SmallFieldKind; 4],
    ) {
        match &mut *self {
            Cells::Rows { shape, data, .. } if *shape == shape::flat(kinds, len) => {
                let off = idx * len;
                data[off..off + len].copy_from_slice(&vals[..len]);
            }
            _ => {
                let v = shape::unpack(shape::flat(kinds, len), &vals[..len]);
                self.set(idx, v);
            }
        }
    }

    #[inline]
    pub fn set(&mut self, idx: usize, val: Value) {
        match (&mut *self, val) {
            (Cells::Int(v), Value::Int(x)) => v[idx] = x,
            (Cells::OneIntStruct(v), Value::Tuple(TupleVal::OneInt(x))) => v[idx] = x,
            (Cells::OptInt(v), Value::Opt(OptVal::None)) => v[idx] = None,
            (Cells::OptInt(v), Value::Opt(OptVal::Int(x))) => v[idx] = Some(x),
            (Cells::Bool(v), Value::Bool(b)) => v[idx] = u8::from(b),
            (Cells::Char(v), Value::Char(c)) => v[idx] = c,
            (Cells::Float(v), Value::Float(f)) => v[idx] = key_of(f),
            (
                Cells::Rows {
                    stride,
                    shape,
                    data,
                },
                other,
            ) => {
                let s = *stride as usize;
                if !shape::pack(*shape, &other, &mut data[idx * s..idx * s + s]) {
                    self.as_mut_values()[idx] = other;
                }
            }
            (Cells::Values(v), other) => v[idx] = other,
            (_, other) => {
                if self.rows_from_nones(&other) {
                    self.set(idx, other);
                } else {
                    self.as_mut_values()[idx] = other;
                }
            }
        }
    }

    pub fn contains(&self, needle: &Value) -> bool {
        match self {
            Cells::Values(v) => v.contains(needle),
            _ => (0..self.len()).any(|i| &self.get_unchecked_val(i) == needle),
        }
    }

    pub fn reverse(&mut self) {
        self.reverse_range(0, self.len());
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        match &mut *self {
            Cells::Int(v) | Cells::OneIntStruct(v) | Cells::Float(v) => v.swap(i, j),
            Cells::OptInt(v) => v.swap(i, j),
            Cells::Bool(v) | Cells::Char(v) => v.swap(i, j),
            Cells::Rows { stride, data, .. } => {
                let s = *stride as usize;
                let a = i * s;
                let b = j * s;
                for k in 0..s {
                    data.swap(a + k, b + k);
                }
            }
            Cells::Values(v) => v.swap(i, j),
        }
    }

    pub fn fill(&mut self, val: Value) {
        self.fill_range(0, self.len(), val);
    }

    pub fn to_values(&self) -> Vec<Value> {
        match self {
            Cells::Values(v) => v.clone(),
            _ => (0..self.len()).map(|i| self.get_unchecked_val(i)).collect(),
        }
    }

    pub fn into_values(self) -> Vec<Value> {
        match self {
            Cells::Values(v) => v,
            other => other.to_values(),
        }
    }

    pub fn as_mut_values(&mut self) -> &mut Vec<Value> {
        if !matches!(self, Cells::Values(_)) {
            let vals = self.to_values();
            *self = Cells::Values(vals);
        }
        match &mut *self {
            Cells::Values(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn cmp_data(&self, other: &Cells) -> Ordering {
        match (self, other) {
            (Cells::Int(a), Cells::Int(b))
            | (Cells::OneIntStruct(a), Cells::OneIntStruct(b))
            | (Cells::Float(a), Cells::Float(b)) => a.cmp(b),
            (Cells::Values(a), Cells::Values(b)) => a.cmp(b),
            _ => {
                let n = self.len().min(other.len());
                for i in 0..n {
                    let ord = self.get_unchecked_val(i).cmp(&other.get_unchecked_val(i));
                    if ord != Ordering::Equal {
                        return ord;
                    }
                }
                self.len().cmp(&other.len())
            }
        }
    }
}

/// The list operations: a push of a cell of the store's kind
/// stays packed, an empty store takes the kind of its first cell, and any
/// other cell turns the store back into boxed values.
impl Cells {
    pub fn push(&mut self, val: Value) {
        match (&mut *self, val) {
            (Cells::Int(v), Value::Int(x)) => v.push(x),
            (Cells::OneIntStruct(v), Value::Tuple(TupleVal::OneInt(x))) => v.push(x),
            (Cells::OptInt(v), Value::Opt(OptVal::None)) => v.push(None),
            (Cells::OptInt(v), Value::Opt(OptVal::Int(x))) => v.push(Some(x)),
            (Cells::Bool(v), Value::Bool(b)) => v.push(u8::from(b)),
            (Cells::Char(v), Value::Char(c)) => v.push(c),
            (Cells::Float(v), Value::Float(f)) => v.push(key_of(f)),
            (Cells::Values(v), other) if v.is_empty() => {
                let cap = v.capacity();
                *self = Cells::one(other);
                self.reserve(cap.saturating_sub(1));
            }
            (Cells::Values(v), other) => v.push(other),
            (
                Cells::Rows {
                    stride,
                    shape,
                    data,
                },
                other,
            ) => {
                let at = data.len();
                data.resize(at + *stride as usize, 0);
                if !shape::pack(*shape, &other, &mut data[at..]) {
                    data.truncate(at);
                    self.as_mut_values().push(other);
                }
            }
            (_, other) => {
                if self.rows_from_nones(&other) {
                    self.push(other);
                } else {
                    self.as_mut_values().push(other);
                }
            }
        }
    }

    /// Turns a store of only `None`s into rows of the shape of `like`, an
    /// optional that a store of optional `Int`s cannot hold.
    fn rows_from_nones(&mut self, like: &Value) -> bool {
        let Cells::OptInt(v) = self else {
            return false;
        };
        if !matches!(like, Value::Opt(_)) || v.iter().any(Option::is_some) {
            return false;
        }
        let Some((shape, stride)) = shape::of(like) else {
            return false;
        };
        let data = vec![0; v.len() * stride as usize];
        *self = Cells::Rows {
            stride,
            shape,
            data,
        };
        true
    }

    pub fn reserve(&mut self, extra: usize) {
        match self {
            Cells::Int(v) | Cells::OneIntStruct(v) | Cells::Float(v) => v.reserve(extra),
            Cells::OptInt(v) => v.reserve(extra),
            Cells::Bool(v) | Cells::Char(v) => v.reserve(extra),
            Cells::Rows { stride, data, .. } => data.reserve(extra * *stride as usize),
            Cells::Values(v) => v.reserve(extra),
        }
    }

    pub fn truncate(&mut self, len: usize) {
        match self {
            Cells::Int(v) | Cells::OneIntStruct(v) | Cells::Float(v) => v.truncate(len),
            Cells::OptInt(v) => v.truncate(len),
            Cells::Bool(v) | Cells::Char(v) => v.truncate(len),
            Cells::Rows { stride, data, .. } => data.truncate(len * *stride as usize),
            Cells::Values(v) => v.truncate(len),
        }
    }

    pub fn clear(&mut self) {
        self.truncate(0);
    }

    pub fn pop(&mut self) -> Option<Value> {
        let last = self.len().checked_sub(1)?;
        let val = self.get_unchecked_val(last);
        self.truncate(last);
        Some(val)
    }

    /// `val` at `at`, the tail moved up one cell in one block move.
    pub fn insert(&mut self, at: usize, val: Value) {
        self.push(val);
        self.rotate_tail(at, true);
    }

    /// The cell at `at`, the tail moved down one cell in one block move.
    pub fn remove(&mut self, at: usize) -> Value {
        let val = self.get_unchecked_val(at);
        self.rotate_tail(at, false);
        self.truncate(self.len() - 1);
        val
    }

    fn rotate_tail(&mut self, at: usize, up: bool) {
        fn turn<T>(s: &mut [T], by: usize, up: bool) {
            if up {
                s.rotate_right(by);
            } else {
                s.rotate_left(by);
            }
        }
        match self {
            Cells::Int(v) | Cells::OneIntStruct(v) | Cells::Float(v) => turn(&mut v[at..], 1, up),
            Cells::Rows { stride, data, .. } => {
                let s = *stride as usize;
                turn(&mut data[at * s..], s, up);
            }
            Cells::Values(v) => turn(&mut v[at..], 1, up),
            Cells::OptInt(_) | Cells::Bool(_) | Cells::Char(_) => {
                let n = self.len();
                for k in at + 1..n {
                    let k = if up { at + n - k } else { k };
                    self.swap(k - 1, k);
                }
            }
        }
    }

    pub fn extend(&mut self, vals: impl IntoIterator<Item = Value>) {
        for v in vals {
            self.push(v);
        }
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = Value> + ExactSizeIterator + '_ {
        (0..self.len()).map(move |i| self.get_unchecked_val(i))
    }
}

impl std::fmt::Debug for Cells {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_values().fmt(f)
    }
}

impl Default for Cells {
    fn default() -> Cells {
        Cells::Values(Vec::new())
    }
}

impl From<Vec<Value>> for Cells {
    fn from(values: Vec<Value>) -> Cells {
        Cells::from_values(values)
    }
}

impl FromIterator<Value> for Cells {
    fn from_iter<I: IntoIterator<Item = Value>>(iter: I) -> Cells {
        let mut cells = Cells::default();
        cells.extend(iter);
        cells
    }
}

impl Ord for Cells {
    fn cmp(&self, other: &Cells) -> Ordering {
        self.cmp_data(other)
    }
}

impl PartialOrd for Cells {
    fn partial_cmp(&self, other: &Cells) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Cells {
    fn eq(&self, other: &Cells) -> bool {
        self.len() == other.len() && self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Cells {}

impl Cells {
    /// Cell `idx` as a stored value, which only a store of boxed values holds.
    pub fn stored(&self, idx: usize) -> Option<&Value> {
        match self {
            Cells::Values(v) => v.get(idx),
            _ => None,
        }
    }

    /// A place for cell `idx`, boxing the store first.
    pub fn cell_mut(&mut self, idx: usize) -> &mut Value {
        &mut self.as_mut_values()[idx]
    }

    pub fn values_in(&self, lo: usize, hi: usize) -> Vec<Value> {
        (lo..hi).map(|i| self.get_unchecked_val(i)).collect()
    }

    /// The cells as values, without a copy when `cells` is not shared.
    pub fn unwrap_values(cells: Rc<Cells>) -> Vec<Value> {
        Rc::try_unwrap(cells).map_or_else(|r| r.to_values(), Cells::into_values)
    }

    pub fn resize(&mut self, len: usize, fill: Value) {
        if len <= self.len() {
            self.truncate(len);
            return;
        }
        self.reserve(len - self.len());
        while self.len() < len {
            self.push(fill.clone());
        }
    }

    pub fn reverse_range(&mut self, lo: usize, hi: usize) {
        let (mut a, mut b) = (lo, hi);
        while a + 1 < b {
            b -= 1;
            self.swap(a, b);
            a += 1;
        }
    }

    pub fn fill_range(&mut self, lo: usize, hi: usize, val: Value) {
        for i in lo..hi {
            self.set(i, val.clone());
        }
    }

    /// Sorts in the order `<` uses on the cells.
    pub fn sort_unstable(&mut self) {
        match self {
            Cells::Int(v) | Cells::OneIntStruct(v) | Cells::Float(v) => v.sort_unstable(),
            Cells::OptInt(v) => v.sort_unstable(),
            Cells::Bool(v) | Cells::Char(v) => v.sort_unstable(),
            Cells::Values(v) => crate::sort::sort_values(v),
            Cells::Rows { shape, .. } if *shape & shape::UNORDERED != 0 => {
                let mut vals = self.to_values();
                crate::sort::sort_values(&mut vals);
                *self = Cells::from_values(vals);
            }
            // Every field kind packs into `i64` in its own order, so a tuple's
            // `<` is its row's.
            Cells::Rows { stride, data, .. } => {
                let k = *stride as usize;
                let row = |i: usize| &data[i * k..i * k + k];
                let Ok(order) =
                    crate::sort::stable_order(data.len() / k, &mut |a, b| Ok(row(a) < row(b)))
                else {
                    return;
                };
                *data = order.iter().flat_map(|&i| row(i)).copied().collect();
            }
        }
    }
}
