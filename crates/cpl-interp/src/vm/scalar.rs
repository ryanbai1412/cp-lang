use crate::value::{OptVal, SmallFieldKind, TupleVal, Value};
use cpl_ir::{Program, Ty};

/// How an unboxed `i64` register maps back to a [`Value`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ScalarKind {
    Int,
    U64,
    Bool,
    Char,
    OneIntStruct,
    /// A struct of one `U64` field (`Subset<B>`), held as its bits.
    OneU64Struct,
}

impl ScalarKind {
    pub fn of_ty(ty: &Ty, program: &Program) -> Option<ScalarKind> {
        match ty {
            Ty::Int | Ty::Pos(_) | Ty::Bound(_) => Some(ScalarKind::Int),
            Ty::U64 => Some(ScalarKind::U64),
            Ty::Bool => Some(ScalarKind::Bool),
            Ty::Char => Some(ScalarKind::Char),
            Ty::Struct(id) => {
                let s = &program.structs[*id];
                if s.fields.len() == 1
                    && matches!(s.fields[0].1, Ty::Int | Ty::Pos(_) | Ty::Bound(_))
                {
                    Some(ScalarKind::OneIntStruct)
                } else if s.fields.len() == 1 && s.fields[0].1 == Ty::U64 {
                    Some(ScalarKind::OneU64Struct)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn of_opt_ty(ty: &Ty, program: &Program) -> Option<ScalarKind> {
        match ty {
            Ty::Opt(inner) => ScalarKind::of_ty(inner, program),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn box_val(self, raw: i64) -> Value {
        match self {
            ScalarKind::Int => Value::Int(raw),
            ScalarKind::U64 => Value::U64(raw as u64),
            ScalarKind::Bool => Value::Bool(raw != 0),
            ScalarKind::Char => Value::Char(raw as u8),
            ScalarKind::OneIntStruct => Value::Tuple(TupleVal::OneInt(raw)),
            ScalarKind::OneU64Struct => {
                Value::Tuple(TupleVal::Boxed(crate::value::Shared::new(vec![
                    Value::U64(raw as u64),
                ])))
            }
        }
    }

    #[inline(always)]
    pub fn box_opt(self, raw: i64, some: i64) -> Value {
        if some == 0 {
            Value::Opt(OptVal::None)
        } else if self == ScalarKind::Int {
            Value::Opt(OptVal::Int(raw))
        } else {
            Value::Opt(OptVal::some(self.box_val(raw)))
        }
    }

    #[inline(always)]
    pub fn unbox_val(val: &Value) -> i64 {
        match val {
            Value::Int(v) => *v,
            Value::U64(v) => *v as i64,
            Value::Bool(b) => i64::from(*b),
            Value::Char(c) => i64::from(*c),
            Value::Tuple(TupleVal::OneInt(v)) => *v,
            Value::Tuple(TupleVal::Boxed(fs)) if fs.len() == 1 => Self::unbox_val(&fs[0]),
            _ => 0,
        }
    }

    #[inline(always)]
    pub fn unbox_opt(val: &Value) -> (i64, i64) {
        match val {
            Value::Opt(OptVal::None) => (0, 0),
            Value::Opt(OptVal::Int(v)) => (*v, 1),
            Value::Opt(OptVal::Boxed(rc)) => (ScalarKind::unbox_val(rc), 1),
            _ => (0, 0),
        }
    }
}

/// The words and [`crate::value::shape`] of a tuple of 2-5 Int-register
/// scalars, nested tuples of them flattened in order.
pub(super) fn small_tuple_shape(ty: &Ty, program: &Program) -> Option<(u8, u64)> {
    let (shape, _, words) = scalar_tuple(ty, program)?;
    (2..=5).contains(&words).then_some((words as u8, shape))
}

/// The shape, nodes and words of the tuple `ty` of scalars or such tuples.
fn scalar_tuple(ty: &Ty, program: &Program) -> Option<(u64, usize, usize)> {
    let field_tys: Vec<&Ty> = match ty {
        Ty::Tuple(fs) => fs.iter().collect(),
        Ty::Record(fs) => fs.iter().map(|(_, t)| t).collect(),
        Ty::Struct(id) => program.structs[*id].fields.iter().map(|(_, t)| t).collect(),
        _ => return None,
    };
    let mut fields = Vec::with_capacity(field_tys.len());
    let mut words = 0;
    for t in field_tys {
        let kind = match ScalarKind::of_ty(t, program) {
            Some(ScalarKind::Int) => SmallFieldKind::Int,
            Some(ScalarKind::OneIntStruct) => SmallFieldKind::OneIntStruct,
            Some(ScalarKind::Bool) => SmallFieldKind::Bool,
            Some(ScalarKind::Char) => SmallFieldKind::Char,
            Some(ScalarKind::U64 | ScalarKind::OneU64Struct) => return None,
            None => {
                let (shape, nodes, w) = scalar_tuple(t, program).filter(|t| t.2 >= 2)?;
                fields.push((shape, nodes));
                words += w;
                continue;
            }
        };
        fields.push((kind as u64, 1));
        words += 1;
    }
    let (shape, nodes) = crate::value::shape::tuple(&fields)?;
    (words <= 5).then_some((shape, nodes, words))
}
