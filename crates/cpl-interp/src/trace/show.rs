//! How the trace and `debug` show a value: as JSON or for a person.

use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Mode {
    /// For the trace: arrays are JSON arrays, `none` is `null`.
    Json,
    /// For a person: `[0, 3, none]`, `(u: 1, v: 2)`.
    Text,
}

/// A product domain's position as the tuple of parts it names, with that
/// tuple's type: `(R(2), C(0))` for position 6 of `(R, C)` over 3 columns.
fn product_parts(ord: i64, d: cpl_ir::DomainId, view: &View) -> (Value, Ty) {
    let parts = view.program.products[d].as_deref().unwrap_or_default();
    let mut rest = ord;
    let mut values = vec![Value::Unit; parts.len()];
    for (value, (_, ty)) in values.iter_mut().zip(parts).rev() {
        let len = match ty {
            Ty::Pos(f) => view.domains.get(*f).map_or(1, |&(_, len)| len),
            Ty::Enum(e) => view.program.enums[*e].variants.len() as i64,
            _ => 2,
        };
        let part = rest.rem_euclid(len.max(1));
        rest = rest.div_euclid(len.max(1));
        *value = match ty {
            Ty::Enum(_) => Value::Enum(part as u32, std::rc::Rc::new(Vec::new())),
            Ty::Bool => Value::Bool(part != 0),
            _ => Value::Int(part),
        };
    }
    let names: Option<Vec<String>> = parts.iter().map(|(name, _)| name.clone()).collect();
    let tys = parts.iter().map(|(_, ty)| ty.clone());
    let ty = match names {
        Some(names) => Ty::Record(names.into_iter().zip(tys).collect()),
        None => Ty::Tuple(tys.collect()),
    };
    (Value::Tuple(crate::value::TupleVal::from_vec(values)), ty)
}

/// The label of a position or cut, or the number itself.
fn label(value: i64, ty: &Ty, view: &View) -> i64 {
    match ty {
        Ty::Pos(d) | Ty::Bound(d) => value + view.domains[*d].0,
        _ => value,
    }
}

pub(super) fn field_names(ty: &Ty, program: &Program) -> Option<Vec<String>> {
    match ty {
        Ty::Record(fields) => Some(fields.iter().map(|(name, _)| name.clone()).collect()),
        Ty::Struct(id) => Some(
            program.structs[*id]
                .fields
                .iter()
                .map(|(name, _)| name.clone())
                .collect(),
        ),
        _ => None,
    }
}

pub(super) fn field_tys(ty: &Ty, program: &Program) -> Vec<Ty> {
    match ty {
        Ty::Struct(id) => program.structs[*id]
            .fields
            .iter()
            .map(|(_, ty)| ty.clone())
            .collect(),
        other => other.fields().unwrap_or_default(),
    }
}

/// A value's `show` fn and the value, to the text the fn gave.
pub(crate) type ShowTexts = BTreeMap<(cpl_ir::FnId, Value), String>;

/// `value` as `debug(...)` shows it.
pub(crate) fn text_of(value: &Value, ty: &Ty, view: &View) -> String {
    let mut out = String::new();
    render(&mut out, value, ty, view, Mode::Text);
    out
}

/// `value` as a JSON value.
pub(crate) fn json_of(value: &Value, ty: &Ty, view: &View) -> String {
    let mut out = String::new();
    if cells(value) > MAX_CELLS {
        string(&mut out, &format!("{} values", cells(value)));
    } else {
        render(&mut out, value, ty, view, Mode::Json);
    }
    out
}

/// Writes `value` of type `ty`.
pub(super) fn render(out: &mut String, value: &Value, ty: &Ty, view: &View, mode: Mode) {
    let json = mode == Mode::Json;
    let sep = if json { "," } else { ", " };
    let list = |out: &mut String,
                items: &mut dyn Iterator<Item = (&Value, &Ty)>,
                open: char,
                close: char| {
        out.push(open);
        for (i, (item, ty)) in items.enumerate() {
            if i > 0 {
                out.push_str(sep);
            }
            render(out, item, ty, view, mode);
        }
        out.push(close);
    };
    if let Some(text) = (view.program.show_fn(ty)).and_then(|f| view.shown.get(&(f, value.clone())))
    {
        if json {
            string(out, text);
        } else {
            out.push_str(text);
        }
        return;
    }
    match (value, ty) {
        // A `U64` the VM keeps as an `Int` shows as one the tree-walker kept.
        (Value::Int(v), Ty::U64) => string(out, &(*v as u64).to_string()),
        (Value::Int(v), Ty::Pos(d))
            if view.program.products.get(*d).is_some_and(Option::is_some) =>
        {
            let (parts, ty) = product_parts(*v, *d, view);
            render(out, &parts, &ty, view, mode)
        }
        (Value::Int(v), _) => {
            let v = label(*v, ty, view);
            // Past 2^53 a JSON number is no longer exact.
            if json && v.unsigned_abs() > 9_007_199_254_740_991 {
                string(out, &v.to_string());
            } else {
                let _ = write!(out, "{v}");
            }
        }
        // Wider than a JSON number holds exactly: as text.
        (Value::I128(v), _) => string(out, &v.to_string()),
        (Value::U64(v), _) => string(out, &v.to_string()),
        (Value::Float(v), _) => {
            if v.is_finite() {
                let text = format!("{v}");
                out.push_str(&text);
                if json && !text.contains(['.', 'e']) {
                    out.push_str(".0");
                }
            } else if json {
                string(out, &format!("{v}"));
            } else {
                out.push_str(&format!("{v}"));
            }
        }
        (Value::Bool(v), _) => out.push_str(if *v { "true" } else { "false" }),
        (Value::Char(c), _) => {
            if json {
                string(out, &(*c as char).to_string());
            } else {
                out.push_str(&format!("'{}'", *c as char));
            }
        }
        (Value::Str(bytes), _) => string(out, &bytes_text(bytes)),
        (Value::Opt(OptVal::None), _) => out.push_str(if json { "null" } else { "none" }),
        (Value::Opt(OptVal::Int(v)), Ty::Opt(inner_ty)) => {
            render(out, &Value::Int(*v), inner_ty, view, mode)
        }
        (Value::Opt(OptVal::Boxed(inner)), Ty::Opt(inner_ty)) => {
            render(out, inner, inner_ty, view, mode)
        }
        (Value::Range(lo, hi), _) => {
            let text = format!("{lo}..{hi}");
            if json {
                string(out, &text);
            } else {
                out.push_str(&text);
            }
        }
        (Value::RangeF(lo, hi), _) => {
            let text = format!("{lo:?}..{hi:?}");
            if json {
                string(out, &text);
            } else {
                out.push_str(&text);
            }
        }
        (Value::Range128(lo, hi), _) => {
            let text = format!("{lo}..{hi}");
            if json {
                string(out, &text);
            } else {
                out.push_str(&text);
            }
        }
        (Value::Tuple(fields), _) => {
            let mut scratch = [Value::Unit];
            let fields = fields.as_slice(&mut scratch);
            let tys = field_tys(ty, view.program);
            match field_names(ty, view.program) {
                Some(names) => {
                    if let (Ty::Struct(id), false) = (ty, json) {
                        let name = &view.program.structs[*id].name;
                        out.push_str(name.split(['<', '_']).next().unwrap_or(name));
                    }
                    out.push(if json { '{' } else { '(' });
                    for (i, ((field, ty), name)) in fields.iter().zip(&tys).zip(&names).enumerate()
                    {
                        if i > 0 {
                            out.push_str(sep);
                        }
                        if json {
                            string(out, name);
                            out.push(':');
                        } else {
                            out.push_str(name);
                            out.push_str(": ");
                        }
                        render(out, field, ty, view, mode);
                    }
                    out.push(if json { '}' } else { ')' });
                }
                None if json => list(out, &mut fields.iter().zip(&tys), '[', ']'),
                None => list(out, &mut fields.iter().zip(&tys), '(', ')'),
            }
        }
        (Value::Enum(tag, payload), Ty::Enum(id)) => {
            let (name, tys) = &view.program.enums[*id].variants[*tag as usize];
            if payload.is_empty() {
                if json {
                    string(out, name);
                } else {
                    out.push_str(name);
                }
            } else if json {
                out.push('{');
                string(out, name);
                out.push(':');
                list(out, &mut payload.iter().zip(tys), '[', ']');
                out.push('}');
            } else {
                out.push_str(name);
                list(out, &mut payload.iter().zip(tys), '(', ')');
            }
        }
        (Value::Array(array), Ty::Array { elem, .. }) => {
            // Rows of rows, the last dimension innermost.
            fn rows(
                out: &mut String,
                data: &[Value],
                dims: &[i64],
                elem: &Ty,
                view: &View,
                mode: Mode,
            ) {
                let sep = if mode == Mode::Json { "," } else { ", " };
                out.push('[');
                match dims {
                    [] | [_] => {
                        for (i, cell) in data.iter().enumerate() {
                            if i > 0 {
                                out.push_str(sep);
                            }
                            render(out, cell, elem, view, mode);
                        }
                    }
                    [rows_n, rest @ ..] => {
                        let width = data.len() / (*rows_n).max(1) as usize;
                        for (i, row) in data.chunks(width.max(1)).enumerate() {
                            if i > 0 {
                                out.push_str(sep);
                            }
                            rows(out, row, rest, elem, view, mode);
                        }
                    }
                }
                out.push(']');
            }
            let vals = array.to_values();
            rows(out, &vals, &array.dims, elem, view, mode);
        }
        (Value::List(items), Ty::List(elem)) => {
            list(out, &mut items.iter().map(|item| (item, &**elem)), '[', ']')
        }
        (Value::Set(items), Ty::Set(elem) | Ty::HashSet(elem)) => {
            let (open, close) = if json { ('[', ']') } else { ('{', '}') };
            list(
                out,
                &mut items.iter().map(|item| (item, &**elem)),
                open,
                close,
            )
        }
        (Value::Map(entries), Ty::Map(key, val) | Ty::HashMap(key, val)) => {
            out.push(if json { '[' } else { '{' });
            for (i, (k, v)) in entries.iter().enumerate() {
                if i > 0 {
                    out.push_str(sep);
                }
                if json {
                    out.push('[');
                    render(out, k, key, view, mode);
                    out.push(',');
                    render(out, v, val, view, mode);
                    out.push(']');
                } else {
                    render(out, k, key, view, mode);
                    out.push_str(": ");
                    render(out, v, val, view, mode);
                }
            }
            out.push(if json { ']' } else { '}' });
        }
        _ if json => string(out, "…"),
        _ => out.push('…'),
    }
}
