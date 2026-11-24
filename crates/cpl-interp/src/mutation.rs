//! What a mutation does to the value it changes: shared by the tree-walker
//! and the VM, which find the value each their own way.

use crate::value::keys::Less;
use crate::value::{self, OptVal, Value};
use crate::{bug, trap, Res};
use cpl_diag::Span;
use cpl_ir::{Mutation, Program, Ty};
use std::rc::Rc;

/// `op` on `cell`, a value of type `ty` that is not a mutable view, with
/// arguments `arg` and `arg2` (`Value::Unit` when absent).
pub(crate) fn mutate_cell(
    op: Mutation,
    cell: &mut Value,
    ty: &Ty,
    (arg, arg2): (Value, Value),
    program: &Program,
    span: Span,
    cmp: &mut Less,
) -> Res<Value> {
    Ok(match (op, cell) {
        (_, Value::Moved) => return trap(span, cpl_diag::msgs::interp::LENT.at(&[])),
        (Mutation::Push, Value::List(items)) => {
            Rc::make_mut(items).push(arg);
            Value::Unit
        }
        (Mutation::Take, cell) => {
            let empty = match &*cell {
                Value::List(_) => Value::List(Rc::default()),
                Value::Str(_) => Value::Str(Rc::default()),
                Value::Map(m) => Value::Map(Rc::new(m.empty_like())),
                Value::Set(s) => Value::Set(Rc::new(s.empty_like())),
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
            None => return trap(span, cpl_diag::msgs::interp::POP_FROM_EMPTY_LIST.at(&[])),
        },
        (Mutation::Insert, Value::Set(items)) => {
            let k = match ty {
                Ty::Set(kt) | Ty::HashSet(kt) => value::strip_cached(&arg, kt, program),
                _ => arg,
            };
            Value::Bool(Rc::make_mut(items).insert(k, cmp)?)
        }
        (Mutation::Remove, Value::Set(items)) => {
            let k = match ty {
                Ty::Set(kt) | Ty::HashSet(kt) => value::strip_cached(&arg, kt, program),
                _ => arg,
            };
            Value::Bool(Rc::make_mut(items).remove(&k, cmp)?)
        }
        (Mutation::Remove, Value::Map(entries)) => {
            let k = match ty {
                Ty::Map(kt, _) | Ty::HashMap(kt, _) => value::strip_cached(&arg, kt, program),
                _ => arg,
            };
            Value::Bool(Rc::make_mut(entries).remove(&k, cmp)?.is_some())
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
        (Mutation::Reverse, cells @ (Value::List(_) | Value::Array(_))) => {
            if let Some(cells) = cells.cells_mut() {
                cells.reverse();
            }
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
                        cpl_diag::msgs::interp::SWAP_OUTSIDE_LIST_LENGTH.at(&[&i, &j, &len]),
                    )
                }
            }
            Value::Unit
        }
        (Mutation::Insert, Value::List(items)) => {
            let i = arg.int();
            let v = Rc::make_mut(items);
            match usize::try_from(i).ok().filter(|&at| at <= v.len()) {
                Some(at) => v.insert(at, arg2),
                None => {
                    let len = v.len();
                    return trap(
                        span,
                        cpl_diag::msgs::interp::INSERT_OUTSIDE_LIST_LENGTH.at(&[&i, &len]),
                    );
                }
            }
            Value::Unit
        }
        (Mutation::Remove, Value::List(items)) => {
            let i = arg.int();
            let v = Rc::make_mut(items);
            match usize::try_from(i).ok().filter(|&at| at < v.len()) {
                Some(at) => v.remove(at),
                None => {
                    let len = v.len();
                    return trap(
                        span,
                        cpl_diag::msgs::interp::REMOVE_OUTSIDE_LIST_LENGTH.at(&[&i, &len]),
                    );
                }
            }
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
                        cpl_diag::msgs::interp::SWAP_OUTSIDE_ARRAY_LENGTH.at(&[&i, &j, &len]),
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
                        cpl_diag::msgs::interp::SWAP_OUTSIDE_LIST_LENGTH.at(&[&i, &j, &len]),
                    )
                }
            }
            Value::Unit
        }
        (Mutation::Resize, Value::List(items)) => {
            let n = arg.int();
            if n < 0 {
                return trap(
                    span,
                    cpl_diag::msgs::interp::RESIZE_TO_NEGATIVE_LENGTH.at(&[&n]),
                );
            }
            Rc::make_mut(items).resize(n as usize, arg2);
            Value::Unit
        }
        (Mutation::Resize, Value::Str(bytes)) => {
            let n = arg.int();
            if n < 0 {
                return trap(
                    span,
                    cpl_diag::msgs::interp::RESIZE_TO_NEGATIVE_LENGTH.at(&[&n]),
                );
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
                return trap(
                    span,
                    cpl_diag::msgs::interp::TRUNCATE_TO_NEGATIVE_LENGTH.at(&[&n]),
                );
            }
            Rc::make_mut(items).truncate(n as usize);
            Value::Unit
        }
        (Mutation::Truncate, Value::Str(bytes)) => {
            let n = arg.int();
            if n < 0 {
                return trap(
                    span,
                    cpl_diag::msgs::interp::TRUNCATE_TO_NEGATIVE_LENGTH.at(&[&n]),
                );
            }
            Rc::make_mut(bytes).truncate(n as usize);
            Value::Unit
        }
        (Mutation::Reserve, Value::List(items)) => {
            let n = arg.int();
            if n < 0 {
                return trap(
                    span,
                    cpl_diag::msgs::interp::RESERVE_TO_NEGATIVE_CAPACITY.at(&[&n]),
                );
            }
            let v = Rc::make_mut(items);
            let extra = (n as usize).saturating_sub(v.len());
            v.reserve(extra);
            Value::Unit
        }
        // A map is a tree here: it has no capacity to reserve.
        (Mutation::Reserve, Value::Map(_)) => {
            let n = arg.int();
            if n < 0 {
                return trap(
                    span,
                    cpl_diag::msgs::interp::RESERVE_TO_NEGATIVE_CAPACITY.at(&[&n]),
                );
            }
            Value::Unit
        }
        (Mutation::Extend, Value::List(items)) => {
            let v = Rc::make_mut(items);
            match arg {
                _ if let Some(cells) = arg.cells() => v.extend(cells.iter()),
                Value::Str(bytes) => v.extend(bytes.iter().copied().map(Value::Char)),
                Value::Set(set) => v.extend(set.iter().cloned()),
                _ => {}
            }
            Value::Unit
        }
        (Mutation::Fill, cells @ (Value::List(_) | Value::Array(_))) => {
            if let Some(cells) = cells.cells_mut() {
                cells.fill(arg);
            }
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
        _ => return bug(span, "this value can't be changed that way"),
    })
}
