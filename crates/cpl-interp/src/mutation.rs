//! What a mutation does to the value it changes: shared by the tree-walker
//! and the VM, which find the value each their own way.

use crate::value::{self, OptVal, Value};
use crate::{trap, Res, LENT};
use cpl_diag::Span;
use cpl_ir::{Mutation, Program, Ty};
use std::rc::Rc;

/// `op` on `cell`, a value of type `ty` that is not a mutable view, with
/// arguments `arg` and `arg2` (`Value::Unit` when absent).
pub(crate) fn mutate_cell(
    op: Mutation,
    cell: &mut Value,
    ty: &Ty,
    arg: Value,
    arg2: Value,
    program: &Program,
    span: Span,
) -> Res<Value> {
    Ok(match (op, cell) {
        (_, Value::Moved) => return trap(span, LENT),
        (Mutation::Push, Value::List(items)) => {
            Rc::make_mut(items).push(arg);
            Value::Unit
        }
        (Mutation::Take, cell) => {
            let empty = match &*cell {
                Value::List(_) => Value::List(Rc::default()),
                Value::Str(_) => Value::Str(Rc::default()),
                Value::Map(_) => Value::Map(Rc::default()),
                Value::Set(_) => Value::Set(Rc::default()),
                // What is left behind in `swap` is written over.
                other => other.clone(),
            };
            std::mem::replace(cell, empty)
        }
        (Mutation::Push, Value::Str(bytes)) => {
            if let Value::Char(c) = arg {
                Rc::make_mut(bytes).push(c);
            }
            Value::Unit
        }
        (Mutation::Pop, Value::List(items)) => {
            Value::Opt(OptVal::from_option(Rc::make_mut(items).pop()))
        }
        (Mutation::Pop, Value::Str(bytes)) => Value::Opt(OptVal::from_option(
            Rc::make_mut(bytes).pop().map(Value::Char),
        )),
        (Mutation::PopFirst, Value::Set(items)) => {
            Value::Opt(OptVal::from_option(Rc::make_mut(items).pop_first()))
        }
        (Mutation::PopLast, Value::Set(items)) => {
            Value::Opt(OptVal::from_option(Rc::make_mut(items).pop_last()))
        }
        (Mutation::PopFirst | Mutation::PopLast, Value::Map(entries)) => {
            let entries = Rc::make_mut(entries);
            let found = if op == Mutation::PopFirst {
                entries.pop_first()
            } else {
                entries.pop_last()
            };
            Value::Opt(OptVal::from_option(
                found.map(|(k, v)| Value::tuple(vec![k, v])),
            ))
        }
        (Mutation::PopVal, Value::List(items)) => match Rc::make_mut(items).pop() {
            Some(v) => v,
            None => return trap(span, "pop from an empty list"),
        },
        (Mutation::Insert, Value::Set(items)) => {
            let k = match ty {
                Ty::Set(kt) | Ty::HashSet(kt) => value::strip_cached(&arg, kt, program),
                _ => arg,
            };
            Value::Bool(Rc::make_mut(items).insert(k))
        }
        (Mutation::Remove, Value::Set(items)) => {
            let k = match ty {
                Ty::Set(kt) | Ty::HashSet(kt) => value::strip_cached(&arg, kt, program),
                _ => arg,
            };
            Value::Bool(Rc::make_mut(items).remove(&k))
        }
        (Mutation::Remove, Value::Map(entries)) => {
            let k = match ty {
                Ty::Map(kt, _) | Ty::HashMap(kt, _) => value::strip_cached(&arg, kt, program),
                _ => arg,
            };
            Value::Bool(Rc::make_mut(entries).remove(&k).is_some())
        }
        (Mutation::Clear, Value::List(items)) => {
            Rc::make_mut(items).clear();
            Value::Unit
        }
        (Mutation::Clear, Value::Set(items)) => {
            Rc::make_mut(items).clear();
            Value::Unit
        }
        (Mutation::Clear, Value::Map(entries)) => {
            Rc::make_mut(entries).clear();
            Value::Unit
        }
        (Mutation::Clear, Value::Str(bytes)) => {
            Rc::make_mut(bytes).clear();
            Value::Unit
        }
        (Mutation::Reverse, Value::List(items)) => {
            Rc::make_mut(items).reverse();
            Value::Unit
        }
        (Mutation::Reverse, Value::Array(arr)) => {
            Rc::make_mut(arr).reverse();
            Value::Unit
        }
        (Mutation::Reverse, Value::Str(bytes)) => {
            Rc::make_mut(bytes).reverse();
            Value::Unit
        }
        (Mutation::Swap, Value::List(items)) => {
            let (i, j) = (arg.int(), arg2.int());
            let v = Rc::make_mut(items);
            let len = v.len();
            match (usize::try_from(i).ok(), usize::try_from(j).ok()) {
                (Some(i), Some(j)) if i < len && j < len => v.swap(i, j),
                _ => {
                    return trap(
                        span,
                        format!("swap({i}, {j}) is outside a list of length {len}"),
                    )
                }
            }
            Value::Unit
        }
        (Mutation::Swap, Value::Array(arr)) => {
            let (i, j) = (arg.int(), arg2.int());
            let a = Rc::make_mut(arr);
            let len = a.len();
            match (usize::try_from(i).ok(), usize::try_from(j).ok()) {
                (Some(i), Some(j)) if i < len && j < len => a.swap(i, j),
                _ => {
                    return trap(
                        span,
                        format!("swap({i}, {j}) is outside an array of length {len}"),
                    )
                }
            }
            Value::Unit
        }
        (Mutation::Swap, Value::Str(bytes)) => {
            let (i, j) = (arg.int(), arg2.int());
            let s = Rc::make_mut(bytes);
            let len = s.len();
            match (usize::try_from(i).ok(), usize::try_from(j).ok()) {
                (Some(i), Some(j)) if i < len && j < len => s.swap(i, j),
                _ => {
                    return trap(
                        span,
                        format!("swap({i}, {j}) is outside a list of length {len}"),
                    )
                }
            }
            Value::Unit
        }
        (Mutation::Resize, Value::List(items)) => {
            let n = arg.int();
            if n < 0 {
                return trap(span, format!("resize to negative length {n}"));
            }
            Rc::make_mut(items).resize(n as usize, arg2);
            Value::Unit
        }
        (Mutation::Resize, Value::Str(bytes)) => {
            let n = arg.int();
            if n < 0 {
                return trap(span, format!("resize to negative length {n}"));
            }
            let fill = match arg2 {
                Value::Char(c) => c,
                _ => 0,
            };
            Rc::make_mut(bytes).resize(n as usize, fill);
            Value::Unit
        }
        (Mutation::Truncate, Value::List(items)) => {
            let n = arg.int();
            if n < 0 {
                return trap(span, format!("truncate to negative length {n}"));
            }
            Rc::make_mut(items).truncate(n as usize);
            Value::Unit
        }
        (Mutation::Truncate, Value::Str(bytes)) => {
            let n = arg.int();
            if n < 0 {
                return trap(span, format!("truncate to negative length {n}"));
            }
            Rc::make_mut(bytes).truncate(n as usize);
            Value::Unit
        }
        (Mutation::Reserve, Value::List(items)) => {
            let n = arg.int();
            if n < 0 {
                return trap(span, format!("reserve to negative capacity {n}"));
            }
            let v = Rc::make_mut(items);
            let extra = (n as usize).saturating_sub(v.len());
            v.reserve(extra);
            Value::Unit
        }
        (Mutation::Extend, Value::List(items)) => {
            let v = Rc::make_mut(items);
            match arg {
                Value::List(other) => v.extend(other.iter().cloned()),
                Value::Array(arr) => v.extend((0..arr.len()).map(|i| arr.get_unchecked_val(i))),
                Value::Str(bytes) => v.extend(bytes.iter().copied().map(Value::Char)),
                Value::Set(set) => v.extend(set.iter().cloned()),
                _ => {}
            }
            Value::Unit
        }
        (Mutation::Fill, Value::Array(arr)) => {
            let a = Rc::make_mut(arr);
            a.fill(arg);
            Value::Unit
        }
        (Mutation::Fill, Value::List(items)) => {
            Rc::make_mut(items).fill(arg);
            Value::Unit
        }
        (Mutation::Fill, Value::Str(bytes)) => {
            let fill = match arg {
                Value::Char(c) => c,
                _ => 0,
            };
            Rc::make_mut(bytes).fill(fill);
            Value::Unit
        }
        _ => return trap(span, "internal: this value can't be changed that way"),
    })
}
