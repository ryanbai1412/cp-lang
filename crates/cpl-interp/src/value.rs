//! Run-time values.
//!
//! Collections are values: assigning one copies it. They are shared
//! behind `Rc` and copied on the first write, so the copy the language
//! promises costs nothing until someone changes one side.

use cpl_ir::{sep_of, ClosureId, Dim, Program, Sep, Ty};
use std::cmp::Ordering;
use std::rc::Rc;

mod cells;
pub(crate) mod keys;
pub use keys::{KeyMap, KeySet};
mod recycle;
mod shape;
pub(crate) use cells::{Cells, SmallFieldKind};
pub(crate) use recycle::Shared;

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
    Boxed(Shared<Vec<Value>>),
}

impl TupleVal {
    pub fn from_vec(fields: Vec<Value>) -> TupleVal {
        if fields.len() == 1 {
            if let Value::Int(v) = fields[0] {
                return TupleVal::OneInt(v);
            }
        }
        TupleVal::Boxed(crate::value::Shared::new(fields))
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
            *self = TupleVal::Boxed(crate::value::Shared::new(vec![Value::Int(v)]));
        }
        match self {
            TupleVal::Boxed(fields) => crate::value::Shared::make_mut(fields),
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

/// An `I128` kept at 8-byte alignment: a bare `i128` would make every
/// [`Value`] 16-byte aligned and twice the size.
#[derive(Clone, Copy)]
#[repr(C, packed(8))]
pub(crate) struct Wide(pub i128);

impl Wide {
    #[inline(always)]
    pub fn get(self) -> i128 {
        self.0
    }
}

impl From<i128> for Wide {
    #[inline(always)]
    fn from(v: i128) -> Wide {
        Wide(v)
    }
}

impl std::fmt::Debug for Wide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.get(), f)
    }
}

impl std::fmt::Display for Wide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.get(), f)
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) enum Value {
    #[default]
    Unit,
    /// Integers, and positions and bounds as their ordinals.
    Int(i64),
    I128(Wide),
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
    Range128(Rc<(i128, i128)>),
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
    List(Rc<Cells>),
    /// O(1) read-only view (slice or reverse) into a `List<T>`.
    ListView(Shared<ListViewVal>),
    /// O(1) fixed-length mutable slice view into a `List<T>` or 1D `[D]T`.
    MutView(Rc<MutViewVal>),
    Map(Rc<KeyMap>),
    Set(Rc<KeySet>),
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

    /// Where walk position `idx` (below `len`) sits in `base`.
    #[inline]
    pub fn pos(&self, idx: usize) -> usize {
        if self.rev {
            self.lo + self.len - 1 - idx
        } else {
            self.lo + idx
        }
    }

    #[inline]
    pub fn get(&self, idx: usize) -> Option<u8> {
        (idx < self.len).then(|| self.base[self.pos(idx)])
    }

    /// The bytes in walk order.
    pub fn iter(&self) -> impl Iterator<Item = u8> + '_ {
        (0..self.len).map(move |i| self.base[self.pos(i)])
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
        self.iter().cmp(other.iter())
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
    /// Cell `i` is target cell `lo + i * step`: 1 but on a grid's column.
    pub step: usize,
    /// The key of cell 0: `lo` on a list's slice, 0 on a grid's row or column.
    pub key0: i64,
}

/// O(1) read-only slice/reverse view into a `List<T>`'s backing `Vec`.
#[derive(Clone, Debug)]
pub(crate) struct ListViewVal {
    pub base: Rc<Cells>,
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

    pub fn whole(base: Rc<Cells>, rev: bool) -> ListViewVal {
        let len = base.len();
        ListViewVal {
            base,
            lo: 0,
            len,
            rev,
            shift: 0,
        }
    }

    /// Where walk position `idx` (below `len`) sits in `base`.
    #[inline]
    pub fn pos(&self, idx: usize) -> usize {
        if self.rev {
            self.lo + self.len - 1 - idx
        } else {
            self.lo + idx
        }
    }

    #[inline]
    pub fn get(&self, idx: usize) -> Option<Value> {
        (idx < self.len).then(|| self.base.get_unchecked_val(self.pos(idx)))
    }

    /// The cells in walk order.
    pub fn iter(&self) -> impl Iterator<Item = Value> + '_ {
        (0..self.len).map(move |i| self.base.get_unchecked_val(self.pos(i)))
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
        self.iter().collect()
    }

    pub fn cmp_slice(&self, other: &Cells) -> Ordering {
        self.iter().cmp(other.iter())
    }

    pub fn cmp_view(&self, other: &ListViewVal) -> Ordering {
        self.iter().cmp(other.iter())
    }

    pub fn contains(&self, needle: &Value) -> bool {
        self.iter().any(|v| &v == needle)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Array {
    /// Length of each dimension.
    pub dims: Vec<i64>,
    /// Cells in row-major order.
    pub data: Cells,
}

impl Array {
    pub fn filled(dims: Vec<i64>, fill: Value) -> Array {
        let cells = dims.iter().product::<i64>() as usize;
        let data = Cells::filled(cells, fill);
        Array { dims, data }
    }

    pub fn empty(dims: Vec<i64>) -> Array {
        let data = Cells::Values(Vec::new());
        Array { dims, data }
    }

    pub fn from_values(dims: Vec<i64>, values: Vec<Value>) -> Array {
        let data = Cells::from_values(values);
        Array { dims, data }
    }
}

impl std::ops::Deref for Array {
    type Target = Cells;
    fn deref(&self) -> &Cells {
        &self.data
    }
}

impl std::ops::DerefMut for Array {
    fn deref_mut(&mut self) -> &mut Cells {
        &mut self.data
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
            Value::I128(v) => v.get(),
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

    pub fn range128(lo: i128, hi: i128) -> Value {
        Value::Range128(Rc::new((lo, hi)))
    }

    pub fn some(value: Value) -> Value {
        Value::Opt(OptVal::some(value))
    }

    pub fn list(items: Vec<Value>) -> Value {
        Value::List(Rc::new(Cells::from(items)))
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
            (Value::I128(a), Value::I128(b)) => a.get().cmp(&b.get()),
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
            (Value::Range128(a), Value::Range128(b)) => a.cmp(b),
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
        Ty::I128 => Value::I128(0.into()),
        Ty::U64 => Value::U64(0),
        Ty::Float => Value::Float(0.0),
        Ty::Bool => Value::Bool(false),
        Ty::Char => Value::Char(0),
        Ty::Str => Value::Str(Rc::new(Vec::new())),
        Ty::StrView => Value::StrView(Rc::new(StrViewVal::whole(Rc::new(Vec::new()), false))),
        Ty::Opt(_) => Value::Opt(OptVal::None),
        Ty::Range(inner) if **inner == Ty::I128 => Value::range128(0, 0),
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
        Ty::List(_) => Value::List(Rc::new(crate::value::Cells::from(Vec::new()))),
        Ty::ListView(_, _) | Ty::Keyed(_, _) => Value::ListView(crate::value::Shared::new(
            ListViewVal::whole(Rc::new(crate::value::Cells::from(Vec::new())), false),
        )),
        Ty::MutView(..) => Value::Unit,
        Ty::Map(k, _) => Value::Map(Rc::new(KeyMap::of(k, program))),
        Ty::HashMap(..) => Value::Map(Rc::default()),
        Ty::Set(k) => Value::Set(Rc::new(KeySet::of(k, program))),
        Ty::HashSet(_) => Value::Set(Rc::default()),
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
            items.iter().map(|v| strip_cached(&v, t, program)).collect(),
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
                out.extend(v.iter());
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
                rec(out, &item, elem);
            }
        }
        (Value::ListView(view), Ty::ListView(elem, _) | Ty::Keyed(elem, _)) => {
            let sep = sep_of(elem, &program.structs);
            for (i, item) in view.iter().enumerate() {
                if i > 0 {
                    out.extend_from_slice(sep.bytes());
                }
                rec(out, &item, elem);
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
        (Value::Range128(r), Ty::Range(_)) => {
            let (lo, hi) = (r.0, r.1);
            for i in lo..hi {
                if i > lo {
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
