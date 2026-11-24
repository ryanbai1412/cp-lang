//! Run-time values.
//!
//! Collections are values: assigning one copies it. They are shared
//! behind `Rc` and copied on the first write, so the copy the language
//! promises costs nothing until someone changes one side.

use cpl_ir::{sep_of, ClosureId, Dim, Program, Sep, Ty};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub(crate) enum CaptureVal {
    Val(Value),
    StackRef(usize),
    GlobalRef(usize),
    PartRef(Rc<Part>),
}

/// One step from a value to a value inside it.
#[derive(Clone, Debug)]
pub(crate) enum Step {
    Field(usize),
    /// Into an optional that holds a value.
    Some,
    /// A cell, by the indices it was reached with.
    Index(Vec<Value>),
}

/// A part of a variable: the variable, on the stack or (`global`) among
/// the globals, and the steps from it to the part.
#[derive(Clone, Debug)]
pub(crate) struct Part {
    pub global: bool,
    pub at: usize,
    pub steps: Vec<Step>,
}

/// Unboxed optional for `Int`/`Pos`/`Bound` (the common case for domain steps
/// like `i - 1`), falling back to `Rc<Value>` for compound values.
#[derive(Clone, Debug)]
pub(crate) enum OptVal {
    None,
    Int(i64),
    Boxed(Rc<Value>),
}

impl OptVal {
    pub fn some(value: Value) -> OptVal {
        match value {
            Value::Int(v) => OptVal::Int(v),
            other => OptVal::Boxed(Rc::new(other)),
        }
    }

    pub fn from_option(value: Option<Value>) -> OptVal {
        match value {
            Some(v) => OptVal::some(v),
            None => OptVal::None,
        }
    }

    pub fn is_some(&self) -> bool {
        !matches!(self, OptVal::None)
    }

    pub fn unwrap_owned(self) -> Option<Value> {
        match self {
            OptVal::None => None,
            OptVal::Int(v) => Some(Value::Int(v)),
            OptVal::Boxed(rc) => Some(Rc::try_unwrap(rc).unwrap_or_else(|r| (*r).clone())),
        }
    }
}

impl Ord for OptVal {
    fn cmp(&self, other: &OptVal) -> Ordering {
        match (self, other) {
            (OptVal::None, OptVal::None) => Ordering::Equal,
            (OptVal::None, _) => Ordering::Less,
            (_, OptVal::None) => Ordering::Greater,
            (OptVal::Int(a), OptVal::Int(b)) => a.cmp(b),
            (OptVal::Int(a), OptVal::Boxed(b)) => Value::Int(*a).cmp(b),
            (OptVal::Boxed(a), OptVal::Int(b)) => a.as_ref().cmp(&Value::Int(*b)),
            (OptVal::Boxed(a), OptVal::Boxed(b)) => a.cmp(b),
        }
    }
}

impl PartialOrd for OptVal {
    fn partial_cmp(&self, other: &OptVal) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for OptVal {
    fn eq(&self, other: &OptVal) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for OptVal {}

/// Unboxed 1-field `Int` tuple/struct (e.g. `Mod<P>`, newtypes), falling back
/// to `Rc<Vec<Value>>` for multi-field or non-`Int` tuples/structs.
#[derive(Clone, Debug)]
pub(crate) enum TupleVal {
    OneInt(i64),
    Boxed(Rc<Vec<Value>>),
}

impl TupleVal {
    pub fn from_vec(fields: Vec<Value>) -> TupleVal {
        if fields.len() == 1 {
            if let Value::Int(v) = fields[0] {
                return TupleVal::OneInt(v);
            }
        }
        TupleVal::Boxed(Rc::new(fields))
    }

    pub fn get(&self, index: usize) -> Option<Value> {
        match self {
            TupleVal::OneInt(v) if index == 0 => Some(Value::Int(*v)),
            TupleVal::OneInt(_) => None,
            TupleVal::Boxed(fields) => fields.get(index).cloned(),
        }
    }

    pub fn as_slice<'a>(&'a self, scratch: &'a mut [Value; 1]) -> &'a [Value] {
        match self {
            TupleVal::OneInt(v) => {
                scratch[0] = Value::Int(*v);
                scratch
            }
            TupleVal::Boxed(fields) => fields.as_slice(),
        }
    }

    pub fn as_mut_vec(&mut self) -> &mut Vec<Value> {
        if let TupleVal::OneInt(v) = *self {
            *self = TupleVal::Boxed(Rc::new(vec![Value::Int(v)]));
        }
        match self {
            TupleVal::Boxed(fields) => Rc::make_mut(fields),
            TupleVal::OneInt(_) => unreachable!(),
        }
    }
}

impl Ord for TupleVal {
    fn cmp(&self, other: &TupleVal) -> Ordering {
        match (self, other) {
            (TupleVal::OneInt(a), TupleVal::OneInt(b)) => a.cmp(b),
            (TupleVal::OneInt(a), TupleVal::Boxed(b)) => {
                [Value::Int(*a)].as_slice().cmp(b.as_slice())
            }
            (TupleVal::Boxed(a), TupleVal::OneInt(b)) => {
                a.as_slice().cmp([Value::Int(*b)].as_slice())
            }
            (TupleVal::Boxed(a), TupleVal::Boxed(b)) => a.cmp(b),
        }
    }
}

impl PartialOrd for TupleVal {
    fn partial_cmp(&self, other: &TupleVal) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for TupleVal {
    fn eq(&self, other: &TupleVal) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for TupleVal {}

#[derive(Clone, Debug, Default)]
pub(crate) enum Value {
    #[default]
    Unit,
    /// Integers, and positions and bounds as their ordinals.
    Int(i64),
    I128(i128),
    U64(u64),
    Float(f64),
    Bool(bool),
    Char(u8),
    Str(Rc<Vec<u8>>),
    /// O(1) read-only view (slice or reverse) into a `Str`.
    StrView(Rc<StrViewVal>),
    Opt(OptVal),
    /// A half-open range `(lo, hi)`.
    Range(i64, i64),
    /// A half-open 128-bit integer range `(lo, hi)`.
    Range128(i128, i128),
    /// An interval of reals `(lo, hi)`: searched, never walked.
    RangeF(f64, f64),
    /// A runtime domain handle `(lo, len)`.
    DomainVal(i64, i64),
    /// A closure value referencing `Program::closures[id]` with captured environment.
    Closure(ClosureId, Rc<Vec<CaptureVal>>),
    /// A closure's slot for a variable it captured by reference: where
    /// the variable lives, on the stack or (`true`) among the globals.
    Alias(bool, usize),
    /// The slot of a `mut` parameter given a field or a cell: it names
    /// the part, so the callee and whatever it calls see one value.
    Part(Rc<Part>),
    /// A tuple, record or struct: its fields.
    Tuple(TupleVal),
    Enum(u32, Rc<Vec<Value>>),
    Array(Rc<Array>),
    List(Rc<Vec<Value>>),
    /// O(1) read-only view (slice or reverse) into a `List<T>`.
    ListView(Rc<ListViewVal>),
    /// O(1) fixed-length mutable slice view into a `List<T>` or 1D `[D]T`.
    MutView(Rc<MutViewVal>),
    Map(Rc<BTreeMap<Value, Value>>),
    Set(Rc<BTreeSet<Value>>),
    /// Lent to a call as a `mut` argument until the call returns.
    Moved,
}

/// O(1) read-only slice/reverse view into a `Str`'s byte buffer.
#[derive(Clone, Debug)]
pub(crate) struct StrViewVal {
    pub base: Rc<Vec<u8>>,
    pub lo: usize,
    pub len: usize,
    pub rev: bool,
}

impl StrViewVal {
    pub fn whole(base: Rc<Vec<u8>>, rev: bool) -> StrViewVal {
        let len = base.len();
        StrViewVal {
            base,
            lo: 0,
            len,
            rev,
        }
    }

    #[inline]
    pub fn get(&self, idx: usize) -> Option<u8> {
        if idx >= self.len {
            return None;
        }
        let pos = if self.rev {
            self.lo + self.len - 1 - idx
        } else {
            self.lo + idx
        };
        Some(self.base[pos])
    }

    /// The cells `rel` counted in storage order from `lo`, whatever the walk direction.
    pub fn slice_storage(&self, rel: std::ops::Range<usize>) -> StrViewVal {
        StrViewVal {
            base: Rc::clone(&self.base),
            lo: self.lo + rel.start,
            len: rel.len(),
            rev: self.rev,
        }
    }

    pub fn slice(&self, lo: usize, hi: usize) -> StrViewVal {
        let n = hi.saturating_sub(lo);
        let new_lo = if self.rev {
            self.lo + self.len - hi
        } else {
            self.lo + lo
        };
        StrViewVal {
            base: Rc::clone(&self.base),
            lo: new_lo,
            len: n,
            rev: self.rev,
        }
    }

    pub fn reversed(&self) -> StrViewVal {
        StrViewVal {
            base: Rc::clone(&self.base),
            lo: self.lo,
            len: self.len,
            rev: !self.rev,
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let slice = &self.base[self.lo..self.lo + self.len];
        if self.rev {
            slice.iter().rev().copied().collect()
        } else {
            slice.to_vec()
        }
    }

    pub fn cmp_bytes(&self, other: &[u8]) -> Ordering {
        if !self.rev {
            return self.base[self.lo..self.lo + self.len].cmp(other);
        }
        let n = self.len.min(other.len());
        for i in 0..n {
            let ord = self.base[self.lo + self.len - 1 - i].cmp(&other[i]);
            if ord != Ordering::Equal {
                return ord;
            }
        }
        self.len.cmp(&other.len())
    }

    pub fn cmp_view(&self, other: &StrViewVal) -> Ordering {
        if !self.rev && !other.rev {
            return self.base[self.lo..self.lo + self.len]
                .cmp(&other.base[other.lo..other.lo + other.len]);
        }
        let n = self.len.min(other.len);
        for i in 0..n {
            let a = self.get(i).unwrap();
            let b = other.get(i).unwrap();
            let ord = a.cmp(&b);
            if ord != Ordering::Equal {
                return ord;
            }
        }
        self.len.cmp(&other.len)
    }

    pub fn contains_byte(&self, b: u8) -> bool {
        self.base[self.lo..self.lo + self.len].contains(&b)
    }
}

/// O(1) fixed-length mutable slice view into a `List<T>` or 1D `[D]T`.
#[derive(Clone, Debug)]
pub(crate) struct MutViewVal {
    pub target: Part,
    pub lo: usize,
    pub len: usize,
}

/// O(1) read-only slice/reverse view into a `List<T>`'s backing `Vec`.
#[derive(Clone, Debug)]
pub(crate) struct ListViewVal {
    pub base: Rc<Vec<Value>>,
    pub lo: usize,
    pub len: usize,
    pub rev: bool,
    /// Key of storage cell `i` is `i + shift`; nonzero only after `rekey`.
    pub shift: i64,
}

impl ListViewVal {
    pub fn first_key(&self) -> i64 {
        self.lo as i64 + self.shift
    }

    pub fn whole(base: Rc<Vec<Value>>, rev: bool) -> ListViewVal {
        let len = base.len();
        ListViewVal {
            base,
            lo: 0,
            len,
            rev,
            shift: 0,
        }
    }

    #[inline]
    pub fn get(&self, idx: usize) -> Option<&Value> {
        if idx >= self.len {
            return None;
        }
        let pos = if self.rev {
            self.lo + self.len - 1 - idx
        } else {
            self.lo + idx
        };
        Some(&self.base[pos])
    }

    /// The cells `rel` counted in storage order from `lo`, whatever the walk direction.
    pub fn slice_storage(&self, rel: std::ops::Range<usize>) -> ListViewVal {
        ListViewVal {
            base: Rc::clone(&self.base),
            lo: self.lo + rel.start,
            len: rel.len(),
            rev: self.rev,
            shift: self.shift,
        }
    }

    pub fn slice(&self, lo: usize, hi: usize) -> ListViewVal {
        let n = hi.saturating_sub(lo);
        let new_lo = if self.rev {
            self.lo + self.len - hi
        } else {
            self.lo + lo
        };
        ListViewVal {
            base: Rc::clone(&self.base),
            lo: new_lo,
            len: n,
            rev: self.rev,
            shift: self.shift,
        }
    }

    pub fn reversed(&self) -> ListViewVal {
        ListViewVal {
            base: Rc::clone(&self.base),
            lo: self.lo,
            len: self.len,
            rev: !self.rev,
            shift: self.shift,
        }
    }

    pub fn to_vec(&self) -> Vec<Value> {
        let slice = &self.base[self.lo..self.lo + self.len];
        if self.rev {
            slice.iter().rev().cloned().collect()
        } else {
            slice.to_vec()
        }
    }

    pub fn cmp_slice(&self, other: &[Value]) -> Ordering {
        if !self.rev {
            return self.base[self.lo..self.lo + self.len].cmp(other);
        }
        let n = self.len.min(other.len());
        for i in 0..n {
            let ord = self.base[self.lo + self.len - 1 - i].cmp(&other[i]);
            if ord != Ordering::Equal {
                return ord;
            }
        }
        self.len.cmp(&other.len())
    }

    pub fn cmp_view(&self, other: &ListViewVal) -> Ordering {
        if !self.rev && !other.rev {
            return self.base[self.lo..self.lo + self.len]
                .cmp(&other.base[other.lo..other.lo + other.len]);
        }
        let n = self.len.min(other.len);
        for i in 0..n {
            let ord = self.get(i).unwrap().cmp(other.get(i).unwrap());
            if ord != Ordering::Equal {
                return ord;
            }
        }
        self.len.cmp(&other.len)
    }

    pub fn contains(&self, needle: &Value) -> bool {
        self.base[self.lo..self.lo + self.len].contains(needle)
    }
}

/// Kind of each scalar field inside a packed [`ArrayData::SmallIntTuple`].
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

fn try_unbox_small_tuple(v: &Value) -> Option<(u8, [SmallFieldKind; 4], [i64; 4])> {
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
fn box_small_tuple(stride: u8, kinds: &[SmallFieldKind; 4], slice: &[i64]) -> Value {
    let n = stride as usize;
    let mut fs = Vec::with_capacity(n);
    for i in 0..n {
        fs.push(kinds[i].box_val(slice[i]));
    }
    Value::Tuple(TupleVal::Boxed(Rc::new(fs)))
}

/// Typed flat backing storage for [`Array`] cells so primitive arrays
/// (`Int`, `Pos`, `Bound`, 1-field `Int` structs like `Mod<P>`, `Bool`,
/// `Char`, `Float`, and 2..=4-field scalar tuples/structs) avoid 24-byte
/// `Value` tagging and `Rc`/`Clone`/`Drop` overhead.
#[derive(Clone, Debug)]
pub(crate) enum ArrayData {
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

#[derive(Clone, Debug)]
pub(crate) struct Array {
    /// Length of each dimension.
    pub dims: Vec<i64>,
    /// Cells in row-major order.
    pub data: ArrayData,
}

impl Array {
    pub fn filled(dims: Vec<i64>, fill: Value) -> Array {
        let cells = dims.iter().product::<i64>() as usize;
        let data = match fill {
            Value::Int(v) => ArrayData::Int(vec![v; cells]),
            Value::Tuple(TupleVal::OneInt(v)) => ArrayData::OneIntStruct(vec![v; cells]),
            Value::Opt(OptVal::None) => ArrayData::OptInt(vec![None; cells]),
            Value::Opt(OptVal::Int(v)) => ArrayData::OptInt(vec![Some(v); cells]),
            Value::Bool(b) => ArrayData::Bool(vec![u8::from(b); cells]),
            Value::Char(c) => ArrayData::Char(vec![c; cells]),
            Value::Float(f) => ArrayData::Float(vec![f; cells]),
            other => {
                if let Some((stride, kinds, vals)) = try_unbox_small_tuple(&other) {
                    let s = stride as usize;
                    let mut data = Vec::with_capacity(cells * s);
                    for _ in 0..cells {
                        data.extend_from_slice(&vals[..s]);
                    }
                    ArrayData::SmallIntTuple {
                        stride,
                        kinds,
                        data,
                    }
                } else {
                    ArrayData::Values(vec![other; cells])
                }
            }
        };
        Array { dims, data }
    }

    pub fn empty(dims: Vec<i64>) -> Array {
        Array {
            dims,
            data: ArrayData::Values(Vec::new()),
        }
    }

    pub fn from_values(dims: Vec<i64>, values: Vec<Value>) -> Array {
        let data = match values.first() {
            Some(Value::Int(_)) if values.iter().all(|v| matches!(v, Value::Int(_))) => {
                ArrayData::Int(values.into_iter().map(|v| v.int()).collect())
            }
            Some(Value::Tuple(TupleVal::OneInt(_)))
                if values
                    .iter()
                    .all(|v| matches!(v, Value::Tuple(TupleVal::OneInt(_)))) =>
            {
                ArrayData::OneIntStruct(
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
                ArrayData::Bool(values.into_iter().map(|v| u8::from(v.bool())).collect())
            }
            Some(Value::Char(_)) if values.iter().all(|v| matches!(v, Value::Char(_))) => {
                ArrayData::Char(
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
                ArrayData::Float(values.into_iter().map(|v| v.float()).collect())
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
                    ArrayData::SmallIntTuple {
                        stride,
                        kinds,
                        data,
                    }
                } else {
                    ArrayData::Values(values)
                }
            }
            _ => ArrayData::Values(values),
        };
        Array { dims, data }
    }

    pub fn len(&self) -> usize {
        match &self.data {
            ArrayData::Int(v) | ArrayData::OneIntStruct(v) => v.len(),
            ArrayData::OptInt(v) => v.len(),
            ArrayData::Bool(v) | ArrayData::Char(v) => v.len(),
            ArrayData::Float(v) => v.len(),
            ArrayData::SmallIntTuple { stride, data, .. } => data.len() / (*stride as usize),
            ArrayData::Values(v) => v.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn get_int(&self, idx: usize) -> Option<i64> {
        match &self.data {
            ArrayData::Int(v) => v.get(idx).copied(),
            _ => match self.get(idx)? {
                Value::Int(v) => Some(v),
                _ => None,
            },
        }
    }

    pub fn get(&self, idx: usize) -> Option<Value> {
        match &self.data {
            ArrayData::Int(v) => v.get(idx).copied().map(Value::Int),
            ArrayData::OneIntStruct(v) => v
                .get(idx)
                .copied()
                .map(|x| Value::Tuple(TupleVal::OneInt(x))),
            ArrayData::OptInt(v) => v.get(idx).copied().map(|o| match o {
                Some(x) => Value::Opt(OptVal::Int(x)),
                None => Value::Opt(OptVal::None),
            }),
            ArrayData::Bool(v) => v.get(idx).copied().map(|b| Value::Bool(b != 0)),
            ArrayData::Char(v) => v.get(idx).copied().map(Value::Char),
            ArrayData::Float(v) => v.get(idx).copied().map(Value::Float),
            ArrayData::SmallIntTuple {
                stride,
                kinds,
                data,
            } => {
                let s = *stride as usize;
                let off = idx.checked_mul(s)?;
                let slice = data.get(off..off + s)?;
                Some(box_small_tuple(*stride, kinds, slice))
            }
            ArrayData::Values(v) => v.get(idx).cloned(),
        }
    }

    #[inline]
    pub fn get_unchecked_val(&self, idx: usize) -> Value {
        match &self.data {
            ArrayData::Int(v) => Value::Int(v[idx]),
            ArrayData::OneIntStruct(v) => Value::Tuple(TupleVal::OneInt(v[idx])),
            ArrayData::OptInt(v) => match v[idx] {
                Some(x) => Value::Opt(OptVal::Int(x)),
                None => Value::Opt(OptVal::None),
            },
            ArrayData::Bool(v) => Value::Bool(v[idx] != 0),
            ArrayData::Char(v) => Value::Char(v[idx]),
            ArrayData::Float(v) => Value::Float(v[idx]),
            ArrayData::SmallIntTuple {
                stride,
                kinds,
                data,
            } => {
                let s = *stride as usize;
                let off = idx * s;
                box_small_tuple(*stride, kinds, &data[off..off + s])
            }
            ArrayData::Values(v) => v[idx].clone(),
        }
    }

    #[inline]
    pub fn get_small_tuple(&self, idx: usize, len: usize, out: &mut [i64]) {
        match &self.data {
            ArrayData::SmallIntTuple { stride, data, .. } if *stride as usize == len => {
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
        match &mut self.data {
            ArrayData::SmallIntTuple { stride, data, .. } if *stride as usize == len => {
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
        match (&mut self.data, val) {
            (ArrayData::Int(v), Value::Int(x)) => v[idx] = x,
            (ArrayData::OneIntStruct(v), Value::Tuple(TupleVal::OneInt(x))) => v[idx] = x,
            (ArrayData::OptInt(v), Value::Opt(OptVal::None)) => v[idx] = None,
            (ArrayData::OptInt(v), Value::Opt(OptVal::Int(x))) => v[idx] = Some(x),
            (ArrayData::Bool(v), Value::Bool(b)) => v[idx] = u8::from(b),
            (ArrayData::Char(v), Value::Char(c)) => v[idx] = c,
            (ArrayData::Float(v), Value::Float(f)) => v[idx] = f,
            (
                ArrayData::SmallIntTuple {
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
            (ArrayData::Values(v), other) => v[idx] = other,
            (_, other) => {
                self.as_mut_values()[idx] = other;
            }
        }
    }

    pub fn first(&self) -> Option<Value> {
        self.get(0)
    }

    pub fn last(&self) -> Option<Value> {
        self.len().checked_sub(1).and_then(|i| self.get(i))
    }

    pub fn contains(&self, needle: &Value) -> bool {
        match (&self.data, needle) {
            (ArrayData::Int(v), Value::Int(x)) => v.contains(x),
            (ArrayData::OneIntStruct(v), Value::Tuple(TupleVal::OneInt(x))) => v.contains(x),
            (ArrayData::OptInt(v), Value::Opt(OptVal::None)) => v.contains(&None),
            (ArrayData::OptInt(v), Value::Opt(OptVal::Int(x))) => v.contains(&Some(*x)),
            (ArrayData::Bool(v), Value::Bool(b)) => v.contains(&u8::from(*b)),
            (ArrayData::Char(v), Value::Char(c)) => v.contains(c),
            (ArrayData::Float(v), Value::Float(f)) => {
                v.iter().any(|x| x.total_cmp(f) == Ordering::Equal)
            }
            (ArrayData::Values(v), _) => v.contains(needle),
            _ => (0..self.len()).any(|i| &self.get_unchecked_val(i) == needle),
        }
    }

    pub fn reverse(&mut self) {
        match &mut self.data {
            ArrayData::Int(v) | ArrayData::OneIntStruct(v) => v.reverse(),
            ArrayData::OptInt(v) => v.reverse(),
            ArrayData::Bool(v) | ArrayData::Char(v) => v.reverse(),
            ArrayData::Float(v) => v.reverse(),
            ArrayData::SmallIntTuple { stride, data, .. } => {
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
            ArrayData::Values(v) => v.reverse(),
        }
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        match &mut self.data {
            ArrayData::Int(v) | ArrayData::OneIntStruct(v) => v.swap(i, j),
            ArrayData::OptInt(v) => v.swap(i, j),
            ArrayData::Bool(v) | ArrayData::Char(v) => v.swap(i, j),
            ArrayData::Float(v) => v.swap(i, j),
            ArrayData::SmallIntTuple { stride, data, .. } => {
                let s = *stride as usize;
                let a = i * s;
                let b = j * s;
                for k in 0..s {
                    data.swap(a + k, b + k);
                }
            }
            ArrayData::Values(v) => v.swap(i, j),
        }
    }

    pub fn fill(&mut self, val: Value) {
        match (&mut self.data, val) {
            (ArrayData::Int(v), Value::Int(x)) => v.fill(x),
            (ArrayData::OneIntStruct(v), Value::Tuple(TupleVal::OneInt(x))) => v.fill(x),
            (ArrayData::OptInt(v), Value::Opt(OptVal::None)) => v.fill(None),
            (ArrayData::OptInt(v), Value::Opt(OptVal::Int(x))) => v.fill(Some(x)),
            (ArrayData::Bool(v), Value::Bool(b)) => v.fill(u8::from(b)),
            (ArrayData::Char(v), Value::Char(c)) => v.fill(c),
            (ArrayData::Float(v), Value::Float(f)) => v.fill(f),
            (
                ArrayData::SmallIntTuple {
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
            (ArrayData::Values(v), other) => v.fill(other),
            (_, other) => {
                self.as_mut_values().fill(other);
            }
        }
    }

    pub fn to_values(&self) -> Vec<Value> {
        match &self.data {
            ArrayData::Int(v) => v.iter().copied().map(Value::Int).collect(),
            ArrayData::OneIntStruct(v) => v
                .iter()
                .copied()
                .map(|x| Value::Tuple(TupleVal::OneInt(x)))
                .collect(),
            ArrayData::OptInt(v) => v
                .iter()
                .copied()
                .map(|o| match o {
                    Some(x) => Value::Opt(OptVal::Int(x)),
                    None => Value::Opt(OptVal::None),
                })
                .collect(),
            ArrayData::Bool(v) => v.iter().copied().map(|b| Value::Bool(b != 0)).collect(),
            ArrayData::Char(v) => v.iter().copied().map(Value::Char).collect(),
            ArrayData::Float(v) => v.iter().copied().map(Value::Float).collect(),
            ArrayData::SmallIntTuple {
                stride,
                kinds,
                data,
            } => {
                let s = *stride as usize;
                data.chunks_exact(s)
                    .map(|chunk| box_small_tuple(*stride, kinds, chunk))
                    .collect()
            }
            ArrayData::Values(v) => v.clone(),
        }
    }

    pub fn into_values(self) -> Vec<Value> {
        match self.data {
            ArrayData::Values(v) => v,
            _ => self.to_values(),
        }
    }

    pub fn as_mut_values(&mut self) -> &mut Vec<Value> {
        if !matches!(self.data, ArrayData::Values(_)) {
            let vals = self.to_values();
            self.data = ArrayData::Values(vals);
        }
        match &mut self.data {
            ArrayData::Values(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn cmp_data(&self, other: &Array) -> Ordering {
        match (&self.data, &other.data) {
            (ArrayData::Int(a), ArrayData::Int(b))
            | (ArrayData::OneIntStruct(a), ArrayData::OneIntStruct(b)) => a.cmp(b),
            (ArrayData::Bool(a), ArrayData::Bool(b)) | (ArrayData::Char(a), ArrayData::Char(b)) => {
                a.cmp(b)
            }
            (ArrayData::Float(a), ArrayData::Float(b)) => {
                for (x, y) in a.iter().zip(b.iter()) {
                    let ord = x.total_cmp(y);
                    if ord != Ordering::Equal {
                        return ord;
                    }
                }
                a.len().cmp(&b.len())
            }
            (ArrayData::Values(a), ArrayData::Values(b)) => a.cmp(b),
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

impl Value {
    pub fn int(&self) -> i64 {
        match self {
            Value::Int(v) => *v,
            _ => 0,
        }
    }

    pub fn i128(&self) -> i128 {
        match self {
            Value::I128(v) => *v,
            _ => 0,
        }
    }

    pub fn u64(&self) -> u64 {
        match self {
            Value::U64(v) => *v,
            _ => 0,
        }
    }

    pub fn float(&self) -> f64 {
        match self {
            Value::Float(v) => *v,
            _ => 0.0,
        }
    }

    pub fn bool(&self) -> bool {
        matches!(self, Value::Bool(true))
    }

    pub fn some(value: Value) -> Value {
        Value::Opt(OptVal::some(value))
    }

    pub fn tuple(fields: Vec<Value>) -> Value {
        Value::Tuple(TupleVal::from_vec(fields))
    }

    fn rank(&self) -> u8 {
        match self {
            Value::Unit => 0,
            Value::Int(_) => 1,
            Value::Float(_) => 2,
            Value::Bool(_) => 3,
            Value::Char(_) => 4,
            Value::Str(_) | Value::StrView(_) => 5,
            Value::Opt(_) => 6,
            Value::Range(..) => 7,
            Value::DomainVal(..) => 8,
            Value::Closure(..) | Value::Alias(..) | Value::Part(_) => 9,
            Value::Tuple(_) => 10,
            Value::Enum(..) => 11,
            Value::Array(_) => 12,
            Value::List(_) | Value::ListView(_) | Value::MutView(_) => 13,
            Value::Map(_) => 14,
            Value::Set(_) => 15,
            Value::Moved => 16,
            Value::I128(_) => 17,
            Value::U64(_) => 18,
            Value::Range128(..) => 19,
            Value::RangeF(..) => 20,
        }
    }
}

/// The order `<` uses, which is also the order of sets and map keys:
/// numbers by value, strings by byte, composites field by field, `none`
/// before any value. It matches what the emitted C++ compares.
impl Ord for Value {
    fn cmp(&self, other: &Value) -> Ordering {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a.cmp(b),
            (Value::I128(a), Value::I128(b)) => a.cmp(b),
            (Value::U64(a), Value::U64(b)) => a.cmp(b),
            (Value::Float(a), Value::Float(b)) => {
                a.partial_cmp(b).unwrap_or_else(|| a.total_cmp(b))
            }
            (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
            (Value::Char(a), Value::Char(b)) => a.cmp(b),
            (Value::Str(a), Value::Str(b)) => a.cmp(b),
            (Value::StrView(a), Value::StrView(b)) => a.cmp_view(b),
            (Value::StrView(a), Value::Str(b)) => a.cmp_bytes(b),
            (Value::Str(a), Value::StrView(b)) => b.cmp_bytes(a).reverse(),
            (Value::Opt(a), Value::Opt(b)) => a.cmp(b),
            (Value::Range(a1, a2), Value::Range(b1, b2)) => (a1, a2).cmp(&(b1, b2)),
            (Value::Range128(a1, a2), Value::Range128(b1, b2)) => (a1, a2).cmp(&(b1, b2)),
            (Value::RangeF(a1, a2), Value::RangeF(b1, b2)) => {
                a1.total_cmp(b1).then_with(|| a2.total_cmp(b2))
            }
            (Value::DomainVal(a1, a2), Value::DomainVal(b1, b2)) => (a1, a2).cmp(&(b1, b2)),
            (Value::Closure(a, _), Value::Closure(b, _)) => a.cmp(b),
            (Value::Tuple(a), Value::Tuple(b)) => a.cmp(b),
            (Value::List(a), Value::List(b)) => a.cmp(b),
            (Value::ListView(a), Value::ListView(b)) => a.cmp_view(b),
            (Value::ListView(a), Value::List(b)) => a.cmp_slice(b),
            (Value::List(a), Value::ListView(b)) => b.cmp_slice(a).reverse(),
            (Value::Enum(ta, a), Value::Enum(tb, b)) => ta.cmp(tb).then_with(|| a.cmp(b)),
            (Value::Array(a), Value::Array(b)) => a.cmp_data(b),
            (Value::Map(a), Value::Map(b)) => a.cmp(b),
            (Value::Set(a), Value::Set(b)) => a.cmp(b),
            _ => self.rank().cmp(&other.rank()),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Value) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Value {}

/// What a variable of type `ty` holds before anything is stored in it.
pub(crate) fn zero(ty: &Ty, program: &Program) -> Value {
    match ty {
        Ty::Unit | Ty::Error => Value::Unit,
        Ty::Int | Ty::Pos(_) | Ty::Bound(_) => Value::Int(0),
        Ty::I128 => Value::I128(0),
        Ty::U64 => Value::U64(0),
        Ty::Float => Value::Float(0.0),
        Ty::Bool => Value::Bool(false),
        Ty::Char => Value::Char(0),
        Ty::Str => Value::Str(Rc::new(Vec::new())),
        Ty::StrView => Value::StrView(Rc::new(StrViewVal::whole(Rc::new(Vec::new()), false))),
        Ty::Opt(_) => Value::Opt(OptVal::None),
        Ty::Range(inner) if **inner == Ty::I128 => Value::Range128(0, 0),
        Ty::Range(_) => Value::Range(0, 0),
        Ty::DomainVal => Value::DomainVal(0, 0),
        Ty::Fn(..) | Ty::Mut(_) => Value::Unit,
        Ty::Tuple(items) => Value::tuple(items.iter().map(|t| zero(t, program)).collect()),
        Ty::Record(fields) => Value::tuple(fields.iter().map(|(_, t)| zero(t, program)).collect()),
        Ty::Struct(id) => Value::tuple(
            program.structs[*id]
                .fields
                .iter()
                .map(|(_, t)| zero(t, program))
                .collect(),
        ),
        Ty::Enum(id) => {
            let payload = program.enums[*id]
                .variants
                .first()
                .map(|(_, fields)| fields.iter().map(|t| zero(t, program)).collect())
                .unwrap_or_default();
            Value::Enum(0, Rc::new(payload))
        }
        Ty::Array { dims, .. } => Value::Array(Rc::new(Array::empty(vec![0; dims.len()]))),
        Ty::List(_) => Value::List(Rc::new(Vec::new())),
        Ty::ListView(_, _) | Ty::Keyed(_, _) => {
            Value::ListView(Rc::new(ListViewVal::whole(Rc::new(Vec::new()), false)))
        }
        Ty::MutView(..) => Value::Unit,
        Ty::Map(..) | Ty::HashMap(..) => Value::Map(Rc::new(BTreeMap::new())),
        Ty::Set(_) | Ty::HashSet(_) => Value::Set(Rc::new(BTreeSet::new())),
    }
}

pub(crate) fn ty_has_cached(ty: &Ty, program: &Program) -> bool {
    match ty {
        Ty::Struct(id) => {
            let s = &program.structs[*id];
            !s.cached_fields.is_empty() || s.fields.iter().any(|(_, t)| ty_has_cached(t, program))
        }
        Ty::Tuple(ts) => ts.iter().any(|t| ty_has_cached(t, program)),
        Ty::Record(fs) => fs.iter().any(|(_, t)| ty_has_cached(t, program)),
        Ty::Opt(t) | Ty::List(t) | Ty::Set(t) | Ty::HashSet(t) => ty_has_cached(t, program),
        Ty::Array { elem, .. } => ty_has_cached(elem, program),
        Ty::Map(k, v) | Ty::HashMap(k, v) => ty_has_cached(k, program) || ty_has_cached(v, program),
        Ty::Enum(id) => program.enums[*id]
            .variants
            .iter()
            .any(|(_, fs)| fs.iter().any(|t| ty_has_cached(t, program))),
        _ => false,
    }
}

pub(crate) fn strip_cached(val: &Value, ty: &Ty, program: &Program) -> Value {
    if !ty_has_cached(ty, program) {
        return val.clone();
    }
    match (val, ty) {
        (Value::Tuple(fields), Ty::Struct(id)) => {
            let s = &program.structs[*id];
            let mut scratch = [Value::Unit];
            let slice = fields.as_slice(&mut scratch);
            let out: Vec<Value> = s
                .fields
                .iter()
                .zip(slice.iter())
                .enumerate()
                .map(|(i, ((_, ft), v))| {
                    if s.cached_fields.contains(&i) {
                        zero(ft, program)
                    } else {
                        strip_cached(v, ft, program)
                    }
                })
                .collect();
            Value::tuple(out)
        }
        (Value::Tuple(fields), Ty::Tuple(ts)) => {
            let mut scratch = [Value::Unit];
            let slice = fields.as_slice(&mut scratch);
            Value::tuple(
                ts.iter()
                    .zip(slice.iter())
                    .map(|(t, v)| strip_cached(v, t, program))
                    .collect(),
            )
        }
        (Value::Tuple(fields), Ty::Record(fs)) => {
            let mut scratch = [Value::Unit];
            let slice = fields.as_slice(&mut scratch);
            Value::tuple(
                fs.iter()
                    .zip(slice.iter())
                    .map(|((_, t), v)| strip_cached(v, t, program))
                    .collect(),
            )
        }
        (Value::Opt(OptVal::Boxed(inner)), Ty::Opt(t)) => {
            Value::Opt(OptVal::Boxed(Rc::new(strip_cached(inner, t, program))))
        }
        (Value::List(items), Ty::List(t)) => Value::List(Rc::new(
            items.iter().map(|v| strip_cached(v, t, program)).collect(),
        )),
        _ => val.clone(),
    }
}

/// A float with `digits` after the point, as C's `%.*f` writes it.
pub(crate) fn show_float(out: &mut Vec<u8>, value: f64, digits: u32) {
    use std::io::Write;
    if value.is_nan() {
        out.extend_from_slice(if value.is_sign_negative() {
            b"-nan"
        } else {
            b"nan"
        });
    } else {
        let _ = write!(out, "{:.*}", digits as usize, value);
    }
}

/// Writes `value` the way `print` shows a `ty`. The rules are in
/// `cpl_ir::show`; the C++ back end emits the same ones.
pub(crate) fn show(
    out: &mut Vec<u8>,
    value: &Value,
    ty: &Ty,
    digits: u32,
    program: &Program,
    domains: &[(i64, i64)],
) {
    use std::io::Write;
    let rec =
        |out: &mut Vec<u8>, value: &Value, ty: &Ty| show(out, value, ty, digits, program, domains);
    match (value, ty) {
        (Value::Int(v), Ty::Pos(d) | Ty::Bound(d)) => {
            let _ = write!(out, "{}", v + domains[*d].0);
        }
        (Value::Int(v), _) => {
            let _ = write!(out, "{v}");
        }
        (Value::I128(v), _) => {
            let _ = write!(out, "{v}");
        }
        (Value::U64(v), _) => {
            let _ = write!(out, "{v}");
        }
        (Value::Float(v), _) => show_float(out, *v, digits),
        (Value::Bool(v), _) => out.extend_from_slice(if *v { b"YES" } else { b"NO" }),
        (Value::Char(c), _) => out.push(*c),
        (Value::Str(s), _) => out.extend_from_slice(s),
        (Value::StrView(v), _) => {
            if !v.rev {
                out.extend_from_slice(&v.base[v.lo..v.lo + v.len]);
            } else {
                for i in 0..v.len {
                    out.push(v.get(i).unwrap());
                }
            }
        }
        (Value::Opt(OptVal::None), _) => out.extend_from_slice(b"none"),
        (Value::Opt(OptVal::Int(v)), Ty::Opt(inner_ty)) => rec(out, &Value::Int(*v), inner_ty),
        (Value::Opt(OptVal::Boxed(inner)), Ty::Opt(inner_ty)) => rec(out, inner, inner_ty),
        (Value::Tuple(fields), _) => {
            let mut scratch = [Value::Unit];
            let fields = fields.as_slice(&mut scratch);
            let (tys, cached): (Vec<Ty>, Option<&std::collections::BTreeSet<usize>>) = match ty {
                Ty::Struct(id) => (
                    program.structs[*id]
                        .fields
                        .iter()
                        .map(|(_, t)| t.clone())
                        .collect(),
                    Some(&program.structs[*id].cached_fields),
                ),
                other => (other.fields().unwrap_or_default(), None),
            };
            let mut shown = 0usize;
            for (i, (field, field_ty)) in fields.iter().zip(&tys).enumerate() {
                if cached.is_some_and(|c| c.contains(&i)) {
                    continue;
                }
                if shown > 0 {
                    out.push(b' ');
                }
                rec(out, field, field_ty);
                shown += 1;
            }
        }
        (Value::Enum(tag, payload), Ty::Enum(id)) => {
            let tys = &program.enums[*id].variants[*tag as usize].1;
            out.extend_from_slice(program.enums[*id].token(*tag as usize).as_bytes());
            for (field, field_ty) in payload.iter().zip(tys) {
                out.push(b' ');
                rec(out, field, field_ty);
            }
        }
        (Value::Array(array), Ty::Array { dims, elem }) => {
            let sep = sep_of(elem, &program.structs);
            // The last dimension is a row; rows are one per line.
            let row = match dims.last() {
                Some(
                    Dim::Pos(_)
                    | Dim::Bound(_)
                    | Dim::Sym(_)
                    | Dim::Enum(_)
                    | Dim::Int
                    | Dim::IntFrom(_),
                ) if dims.len() > 1 => array.dims.last().copied().unwrap_or(0).max(1) as usize,
                _ => usize::MAX,
            };
            for i in 0..array.len() {
                if i > 0 {
                    if i % row == 0 {
                        out.push(b'\n');
                    } else {
                        out.extend_from_slice(sep.bytes());
                    }
                }
                let cell = array.get_unchecked_val(i);
                rec(out, &cell, elem);
            }
        }
        (Value::List(items), Ty::List(elem) | Ty::MutView(elem, _)) => {
            let sep = sep_of(elem, &program.structs);
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.extend_from_slice(sep.bytes());
                }
                rec(out, item, elem);
            }
        }
        (Value::ListView(view), Ty::ListView(elem, _) | Ty::Keyed(elem, _)) => {
            let sep = sep_of(elem, &program.structs);
            for i in 0..view.len {
                if i > 0 {
                    out.extend_from_slice(sep.bytes());
                }
                rec(out, view.get(i).unwrap(), elem);
            }
        }
        (Value::Set(items), Ty::Set(elem) | Ty::HashSet(elem)) => {
            let sep = match sep_of(elem, &program.structs) {
                Sep::None => Sep::Space,
                sep => sep,
            };
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.extend_from_slice(sep.bytes());
                }
                rec(out, item, elem);
            }
        }
        (Value::Map(entries), Ty::Map(key_ty, value_ty) | Ty::HashMap(key_ty, value_ty)) => {
            for (i, (key, value)) in entries.iter().enumerate() {
                if i > 0 {
                    out.push(b'\n');
                }
                rec(out, key, key_ty);
                out.push(b' ');
                rec(out, value, value_ty);
            }
        }
        // A range is a `Seq`: it shows its elements (R10.3.1).
        (Value::Range128(lo, hi), Ty::Range(_)) => {
            for i in *lo..*hi {
                if i > *lo {
                    out.push(b' ');
                }
                let _ = write!(out, "{i}");
            }
        }
        (Value::Range(lo, hi), Ty::Range(inner)) => {
            let dlo = match **inner {
                Ty::Pos(d) => domains[d].0,
                _ => 0,
            };
            for i in *lo..*hi {
                if i > *lo {
                    out.push(b' ');
                }
                let _ = write!(out, "{}", i + dlo);
            }
        }
        _ => {}
    }
}
