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

pub(super) fn small_tuple_shape(ty: &Ty, program: &Program) -> Option<(u8, [SmallFieldKind; 4])> {
    let field_tys: Vec<&Ty> = match ty {
        Ty::Tuple(fs) => fs.iter().collect(),
        Ty::Struct(id) => program.structs[*id].fields.iter().map(|(_, t)| t).collect(),
        _ => return None,
    };
    let len = field_tys.len();
    if !(2..=4).contains(&len) {
        return None;
    }
    let mut kinds = [SmallFieldKind::Int; 4];
    for (i, t) in field_tys.iter().enumerate() {
        let sk = ScalarKind::of_ty(t, program)?;
        kinds[i] = match sk {
            ScalarKind::Int => SmallFieldKind::Int,
            ScalarKind::OneIntStruct => SmallFieldKind::OneIntStruct,
            ScalarKind::Bool => SmallFieldKind::Bool,
            ScalarKind::Char => SmallFieldKind::Char,
            ScalarKind::U64 => return None,
        };
    }
    Some((len as u8, kinds))
}
