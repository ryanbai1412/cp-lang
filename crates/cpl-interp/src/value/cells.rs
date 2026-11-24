//! Typed cell storage shared by arrays and lists: cells of one scalar kind
//! are stored packed, as the emitted C++ stores them, and boxed into a
//! [`Value`] only when read.

use super::{OptVal, TupleVal, Value};
use std::cmp::Ordering;
use std::rc::Rc;

/// Kind of each scalar field inside a packed [`Cells::SmallIntTuple`].
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

pub(crate) fn try_unbox_small_tuple(v: &Value) -> Option<(u8, [SmallFieldKind; 4], [i64; 4])> {
    let Value::Tuple(TupleVal::Boxed(fs)) = v else {
        return None;
    };
    let len = fs.len();
    if !(2..=4).contains(&len) {
        return None;
    }
    let mut kinds = [SmallFieldKind::Int; 4];
    let mut vals = [0i64; 4];
    for (i, f) in fs.iter().enumerate() {
        let (k, x) = SmallFieldKind::try_unbox(f)?;
        kinds[i] = k;
        vals[i] = x;
    }
    Some((len as u8, kinds, vals))
}

#[inline]
pub(crate) fn box_small_tuple(stride: u8, kinds: &[SmallFieldKind; 4], slice: &[i64]) -> Value {
    let n = stride as usize;
    let mut fs = Vec::with_capacity(n);
    for i in 0..n {
        fs.push(kinds[i].box_val(slice[i]));
    }
    Value::Tuple(TupleVal::Boxed(Rc::new(fs)))
}

/// Typed flat backing storage for array and list cells so primitive ones
/// (`Int`, `Pos`, `Bound`, 1-field `Int` structs like `Mod<P>`, `Bool`,
/// `Char`, `Float`, and 2..=4-field scalar tuples/structs) avoid 24-byte
/// `Value` tagging and `Rc`/`Clone`/`Drop` overhead.
#[derive(Clone, Debug)]
pub(crate) enum Cells {
    Int(Vec<i64>),
    OneIntStruct(Vec<i64>),
    OptInt(Vec<Option<i64>>),
    Bool(Vec<u8>),
    Char(Vec<u8>),
    Float(Vec<f64>),
    SmallIntTuple {
        stride: u8,
        kinds: [SmallFieldKind; 4],
        data: Vec<i64>,
    },
    Values(Vec<Value>),
}

impl Cells {
    pub fn filled(cells: usize, fill: Value) -> Cells {
        match fill {
            Value::Int(v) => Cells::Int(vec![v; cells]),
            Value::Tuple(TupleVal::OneInt(v)) => Cells::OneIntStruct(vec![v; cells]),
            Value::Opt(OptVal::None) => Cells::OptInt(vec![None; cells]),
            Value::Opt(OptVal::Int(v)) => Cells::OptInt(vec![Some(v); cells]),
            Value::Bool(b) => Cells::Bool(vec![u8::from(b); cells]),
            Value::Char(c) => Cells::Char(vec![c; cells]),
            Value::Float(f) => Cells::Float(vec![f; cells]),
            other => {
                if let Some((stride, kinds, vals)) = try_unbox_small_tuple(&other) {
                    let s = stride as usize;
                    let mut data = Vec::with_capacity(cells * s);
                    for _ in 0..cells {
                        data.extend_from_slice(&vals[..s]);
                    }
                    Cells::SmallIntTuple {
                        stride,
                        kinds,
                        data,
                    }
                } else {
                    Cells::Values(vec![other; cells])
                }
            }
        }
    }

    /// Packs `values` by their element kind when they share one.
    pub fn from_values(values: Vec<Value>) -> Cells {
        match values.first() {
            Some(Value::Int(_)) if values.iter().all(|v| matches!(v, Value::Int(_))) => {
                Cells::Int(values.into_iter().map(|v| v.int()).collect())
            }
            Some(Value::Tuple(TupleVal::OneInt(_)))
                if values
                    .iter()
                    .all(|v| matches!(v, Value::Tuple(TupleVal::OneInt(_)))) =>
            {
                Cells::OneIntStruct(
                    values
                        .into_iter()
                        .map(|v| match v {
                            Value::Tuple(TupleVal::OneInt(x)) => x,
                            _ => unreachable!(),
                        })
                        .collect(),
                )
            }
            Some(Value::Bool(_)) if values.iter().all(|v| matches!(v, Value::Bool(_))) => {
                Cells::Bool(values.into_iter().map(|v| u8::from(v.bool())).collect())
            }
            Some(Value::Char(_)) if values.iter().all(|v| matches!(v, Value::Char(_))) => {
                Cells::Char(
                    values
                        .into_iter()
                        .map(|v| match v {
                            Value::Char(c) => c,
                            _ => unreachable!(),
                        })
                        .collect(),
                )
            }
            Some(Value::Float(_)) if values.iter().all(|v| matches!(v, Value::Float(_))) => {
                Cells::Float(values.into_iter().map(|v| v.float()).collect())
            }
            Some(first) if try_unbox_small_tuple(first).is_some() => {
                let (stride, kinds, _) = try_unbox_small_tuple(first).unwrap();
                let s = stride as usize;
                let mut data = Vec::with_capacity(values.len() * s);
                let mut ok = true;
                for v in &values {
                    match try_unbox_small_tuple(v) {
                        Some((st, ks, vals)) if st == stride && ks[..s] == kinds[..s] => {
                            data.extend_from_slice(&vals[..s]);
                        }
                        _ => {
                            ok = false;
                            break;
                        }
                    }
                }
                if ok {
                    Cells::SmallIntTuple {
                        stride,
                        kinds,
                        data,
                    }
                } else {
                    Cells::Values(values)
                }
            }
            _ => Cells::Values(values),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Cells::Int(v) | Cells::OneIntStruct(v) => v.len(),
            Cells::OptInt(v) => v.len(),
            Cells::Bool(v) | Cells::Char(v) => v.len(),
            Cells::Float(v) => v.len(),
            Cells::SmallIntTuple { stride, data, .. } => data.len() / (*stride as usize),
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
        match self {
            Cells::Int(v) => v.get(idx).copied().map(Value::Int),
            Cells::OneIntStruct(v) => v
                .get(idx)
                .copied()
                .map(|x| Value::Tuple(TupleVal::OneInt(x))),
            Cells::OptInt(v) => v.get(idx).copied().map(|o| match o {
                Some(x) => Value::Opt(OptVal::Int(x)),
                None => Value::Opt(OptVal::None),
            }),
            Cells::Bool(v) => v.get(idx).copied().map(|b| Value::Bool(b != 0)),
            Cells::Char(v) => v.get(idx).copied().map(Value::Char),
            Cells::Float(v) => v.get(idx).copied().map(Value::Float),
            Cells::SmallIntTuple {
                stride,
                kinds,
                data,
            } => {
                let s = *stride as usize;
                let off = idx.checked_mul(s)?;
                let slice = data.get(off..off + s)?;
                Some(box_small_tuple(*stride, kinds, slice))
            }
            Cells::Values(v) => v.get(idx).cloned(),
        }
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
            Cells::Float(v) => Value::Float(v[idx]),
            Cells::SmallIntTuple {
                stride,
                kinds,
                data,
            } => {
                let s = *stride as usize;
                let off = idx * s;
                box_small_tuple(*stride, kinds, &data[off..off + s])
            }
            Cells::Values(v) => v[idx].clone(),
        }
    }

    #[inline]
    pub fn get_small_tuple(&self, idx: usize, len: usize, out: &mut [i64]) {
        match self {
            Cells::SmallIntTuple { stride, data, .. } if *stride as usize == len => {
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
            Cells::SmallIntTuple { stride, data, .. } if *stride as usize == len => {
                let off = idx * len;
                data[off..off + len].copy_from_slice(&vals[..len]);
            }
            _ => {
                let v = box_small_tuple(len as u8, kinds, &vals[..len]);
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
            (Cells::Float(v), Value::Float(f)) => v[idx] = f,
            (
                Cells::SmallIntTuple {
                    stride,
                    kinds,
                    data,
                },
                other,
            ) => {
                let s = *stride as usize;
                if let Some((st, ks, vals)) = try_unbox_small_tuple(&other) {
                    if st == *stride && ks[..s] == kinds[..s] {
                        let off = idx * s;
                        data[off..off + s].copy_from_slice(&vals[..s]);
                        return;
                    }
                }
                self.as_mut_values()[idx] = other;
            }
            (Cells::Values(v), other) => v[idx] = other,
            (_, other) => {
                self.as_mut_values()[idx] = other;
            }
        }
    }

    pub fn contains(&self, needle: &Value) -> bool {
        match (self, needle) {
            (Cells::Int(v), Value::Int(x)) => v.contains(x),
            (Cells::OneIntStruct(v), Value::Tuple(TupleVal::OneInt(x))) => v.contains(x),
            (Cells::OptInt(v), Value::Opt(OptVal::None)) => v.contains(&None),
            (Cells::OptInt(v), Value::Opt(OptVal::Int(x))) => v.contains(&Some(*x)),
            (Cells::Bool(v), Value::Bool(b)) => v.contains(&u8::from(*b)),
            (Cells::Char(v), Value::Char(c)) => v.contains(c),
            (Cells::Float(v), Value::Float(f)) => {
                v.iter().any(|x| x.total_cmp(f) == Ordering::Equal)
            }
            (Cells::Values(v), _) => v.contains(needle),
            _ => (0..self.len()).any(|i| &self.get_unchecked_val(i) == needle),
        }
    }

    pub fn reverse(&mut self) {
        match &mut *self {
            Cells::Int(v) | Cells::OneIntStruct(v) => v.reverse(),
            Cells::OptInt(v) => v.reverse(),
            Cells::Bool(v) | Cells::Char(v) => v.reverse(),
            Cells::Float(v) => v.reverse(),
            Cells::SmallIntTuple { stride, data, .. } => {
                let s = *stride as usize;
                let n = data.len() / s;
                for i in 0..n / 2 {
                    let a = i * s;
                    let b = (n - 1 - i) * s;
                    for k in 0..s {
                        data.swap(a + k, b + k);
                    }
                }
            }
            Cells::Values(v) => v.reverse(),
        }
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        match &mut *self {
            Cells::Int(v) | Cells::OneIntStruct(v) => v.swap(i, j),
            Cells::OptInt(v) => v.swap(i, j),
            Cells::Bool(v) | Cells::Char(v) => v.swap(i, j),
            Cells::Float(v) => v.swap(i, j),
            Cells::SmallIntTuple { stride, data, .. } => {
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
        match (&mut *self, val) {
            (Cells::Int(v), Value::Int(x)) => v.fill(x),
            (Cells::OneIntStruct(v), Value::Tuple(TupleVal::OneInt(x))) => v.fill(x),
            (Cells::OptInt(v), Value::Opt(OptVal::None)) => v.fill(None),
            (Cells::OptInt(v), Value::Opt(OptVal::Int(x))) => v.fill(Some(x)),
            (Cells::Bool(v), Value::Bool(b)) => v.fill(u8::from(b)),
            (Cells::Char(v), Value::Char(c)) => v.fill(c),
            (Cells::Float(v), Value::Float(f)) => v.fill(f),
            (
                Cells::SmallIntTuple {
                    stride,
                    kinds,
                    data,
                },
                other,
            ) => {
                let s = *stride as usize;
                if let Some((st, ks, vals)) = try_unbox_small_tuple(&other) {
                    if st == *stride && ks[..s] == kinds[..s] {
                        for chunk in data.chunks_exact_mut(s) {
                            chunk.copy_from_slice(&vals[..s]);
                        }
                        return;
                    }
                }
                self.as_mut_values().fill(other);
            }
            (Cells::Values(v), other) => v.fill(other),
            (_, other) => {
                self.as_mut_values().fill(other);
            }
        }
    }

    pub fn to_values(&self) -> Vec<Value> {
        match self {
            Cells::Int(v) => v.iter().copied().map(Value::Int).collect(),
            Cells::OneIntStruct(v) => v
                .iter()
                .copied()
                .map(|x| Value::Tuple(TupleVal::OneInt(x)))
                .collect(),
            Cells::OptInt(v) => v
                .iter()
                .copied()
                .map(|o| match o {
                    Some(x) => Value::Opt(OptVal::Int(x)),
                    None => Value::Opt(OptVal::None),
                })
                .collect(),
            Cells::Bool(v) => v.iter().copied().map(|b| Value::Bool(b != 0)).collect(),
            Cells::Char(v) => v.iter().copied().map(Value::Char).collect(),
            Cells::Float(v) => v.iter().copied().map(Value::Float).collect(),
            Cells::SmallIntTuple {
                stride,
                kinds,
                data,
            } => {
                let s = *stride as usize;
                data.chunks_exact(s)
                    .map(|chunk| box_small_tuple(*stride, kinds, chunk))
                    .collect()
            }
            Cells::Values(v) => v.clone(),
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
            (Cells::Int(a), Cells::Int(b)) | (Cells::OneIntStruct(a), Cells::OneIntStruct(b)) => {
                a.cmp(b)
            }
            (Cells::Bool(a), Cells::Bool(b)) | (Cells::Char(a), Cells::Char(b)) => a.cmp(b),
            (Cells::Float(a), Cells::Float(b)) => {
                for (x, y) in a.iter().zip(b.iter()) {
                    let ord = x.total_cmp(y);
                    if ord != Ordering::Equal {
                        return ord;
                    }
                }
                a.len().cmp(&b.len())
            }
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
