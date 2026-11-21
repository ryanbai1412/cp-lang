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
}

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
    Opt(Option<Rc<Value>>),
    /// A half-open range `(lo, hi)`.
    Range(i64, i64),
    /// A runtime domain handle `(lo, len)`.
    DomainVal(i64, i64),
    /// A closure value referencing `Program::closures[id]` with captured environment.
    Closure(ClosureId, Rc<Vec<CaptureVal>>),
    /// A closure's slot for a variable it captured by reference: where
    /// the variable lives, on the stack or (`true`) among the globals.
    Alias(bool, usize),
    /// A tuple, record or struct: its fields.
    Tuple(Rc<Vec<Value>>),
    Enum(u32, Rc<Vec<Value>>),
    Array(Rc<Array>),
    List(Rc<Vec<Value>>),
    Map(Rc<BTreeMap<Value, Value>>),
    Set(Rc<BTreeSet<Value>>),
    /// Lent to a call as a `mut` argument until the call returns.
    Moved,
}

#[derive(Clone, Debug)]
pub(crate) struct Array {
    /// Length of each dimension.
    pub dims: Vec<i64>,
    /// Cells in row-major order.
    pub data: Vec<Value>,
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
        Value::Opt(Some(Rc::new(value)))
    }

    pub fn tuple(fields: Vec<Value>) -> Value {
        Value::Tuple(Rc::new(fields))
    }

    fn rank(&self) -> u8 {
        match self {
            Value::Unit => 0,
            Value::Int(_) => 1,
            Value::Float(_) => 2,
            Value::Bool(_) => 3,
            Value::Char(_) => 4,
            Value::Str(_) => 5,
            Value::Opt(_) => 6,
            Value::Range(..) => 7,
            Value::DomainVal(..) => 8,
            Value::Closure(..) | Value::Alias(..) => 9,
            Value::Tuple(_) => 10,
            Value::Enum(..) => 11,
            Value::Array(_) => 12,
            Value::List(_) => 13,
            Value::Map(_) => 14,
            Value::Set(_) => 15,
            Value::Moved => 16,
            Value::I128(_) => 17,
            Value::U64(_) => 18,
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
            (Value::Opt(a), Value::Opt(b)) => a.cmp(b),
            (Value::Range(a1, a2), Value::Range(b1, b2)) => (a1, a2).cmp(&(b1, b2)),
            (Value::DomainVal(a1, a2), Value::DomainVal(b1, b2)) => (a1, a2).cmp(&(b1, b2)),
            (Value::Closure(a, _), Value::Closure(b, _)) => a.cmp(b),
            (Value::Tuple(a), Value::Tuple(b)) | (Value::List(a), Value::List(b)) => a.cmp(b),
            (Value::Enum(ta, a), Value::Enum(tb, b)) => ta.cmp(tb).then_with(|| a.cmp(b)),
            (Value::Array(a), Value::Array(b)) => a.data.cmp(&b.data),
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
        Ty::Opt(_) => Value::Opt(None),
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
        Ty::Array { dims, .. } => Value::Array(Rc::new(Array {
            dims: vec![0; dims.len()],
            data: Vec::new(),
        })),
        Ty::List(_) => Value::List(Rc::new(Vec::new())),
        Ty::Map(..) | Ty::HashMap(..) => Value::Map(Rc::new(BTreeMap::new())),
        Ty::Set(_) | Ty::HashSet(_) => Value::Set(Rc::new(BTreeSet::new())),
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
        (Value::Opt(None), _) => out.extend_from_slice(b"none"),
        (Value::Opt(Some(inner)), Ty::Opt(inner_ty)) => rec(out, inner, inner_ty),
        (Value::Tuple(fields), _) => {
            let tys: Vec<Ty> = match ty {
                Ty::Struct(id) => program.structs[*id]
                    .fields
                    .iter()
                    .map(|(_, t)| t.clone())
                    .collect(),
                other => other.fields().unwrap_or_default(),
            };
            for (i, (field, field_ty)) in fields.iter().zip(&tys).enumerate() {
                if i > 0 {
                    out.push(b' ');
                }
                rec(out, field, field_ty);
            }
        }
        (Value::Enum(tag, payload), Ty::Enum(id)) => {
            let (name, tys) = &program.enums[*id].variants[*tag as usize];
            out.extend_from_slice(name.as_bytes());
            for (field, field_ty) in payload.iter().zip(tys) {
                out.push(b' ');
                rec(out, field, field_ty);
            }
        }
        (Value::Array(array), Ty::Array { dims, elem }) => {
            let sep = sep_of(elem, &program.structs);
            // The last dimension is a row; rows are one per line.
            let row = match dims.last() {
                Some(Dim::Pos(_) | Dim::Bound(_) | Dim::Int | Dim::IntFrom(_)) if dims.len() > 1 => {
                    array.dims.last().copied().unwrap_or(0).max(1) as usize
                }
                _ => usize::MAX,
            };
            for (i, cell) in array.data.iter().enumerate() {
                if i > 0 {
                    if i % row == 0 {
                        out.push(b'\n');
                    } else {
                        out.extend_from_slice(sep.bytes());
                    }
                }
                rec(out, cell, elem);
            }
        }
        (Value::List(items), Ty::List(elem)) => {
            let sep = sep_of(elem, &program.structs);
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.extend_from_slice(sep.bytes());
                }
                rec(out, item, elem);
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
        (Value::Range(lo, hi), Ty::Range(inner)) => {
            if let Ty::Pos(d) = **inner {
                let dlo = domains[d].0;
                let _ = write!(out, "{}..{}", lo + dlo, hi + dlo);
            } else {
                let _ = write!(out, "{lo}..{hi}");
            }
        }
        _ => {}
    }
}
