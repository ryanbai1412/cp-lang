//! Signature help and parameter-name inlay hints.

use super::*;

// ------------------------------------------------------------ signature help

/// The innermost unclosed `(` before the cursor and how many of its
/// arguments are already finished.
pub(super) fn open_call(text: &str, from: usize, to: usize) -> Option<(usize, usize, usize)> {
    // (bracket, its offset, commas at its level, start of the current argument)
    let mut stack: Vec<(u8, usize, usize, usize)> = Vec::new();
    let bytes = text.as_bytes();
    let mut i = from;
    while i < to {
        match bytes[i] {
            b'"' => {
                i += 1;
                while i < to && bytes[i] != b'"' {
                    i += if bytes[i] == b'\\' { 2 } else { 1 };
                }
            }
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                while i < to && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b @ (b'(' | b'[' | b'{') => stack.push((b, i, 0, i + 1)),
            b')' | b']' | b'}' => {
                stack.pop();
            }
            b',' => {
                if let Some(top) = stack.last_mut() {
                    top.2 += 1;
                    top.3 = i + 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    let &(_, at, commas, arg) = stack.iter().rev().find(|(b, ..)| *b == b'(')?;
    let top_is_paren = stack.last().is_some_and(|(b, ..)| *b == b'(');
    Some((
        at,
        if top_is_paren { commas } else { 0 },
        if top_is_paren { arg } else { to },
    ))
}

/// One parameter as signature help shows it, its parts as text.
pub(super) struct SigParam {
    pub(super) name: String,
    pub(super) ty: String,
    pub(super) variadic: bool,
    pub(super) default: String,
    pub(super) named_only: bool,
}

/// One signature to show: its parameters' names and types as text.
pub(super) struct Signature {
    pub(super) name: String,
    pub(super) params: Vec<SigParam>,
    pub(super) ret: String,
    pub(super) doc: String,
}

pub fn signature_help(cursor: &Cursor, std: &mut StdLib) -> Option<Value> {
    let text = &*cursor.analysis.text;
    let offset = (cursor.offset as usize).min(text.len());
    let item = item_at(cursor.analysis, cursor.offset)?;
    let (paren, commas, arg_start) = open_call(text, item.offset as usize, offset)?;
    let callee_end = text[..paren].trim_end().len();
    let name = ident_before(text, callee_end);
    if name.is_empty() {
        return None;
    }
    let sig = signature_of(cursor, std, item, callee_end, name)?;
    // `f(a, by: k`: the argument named, else the one by position.
    let given = named_args(&text[paren + 1..arg_start]);
    let active = active_param(&sig, &given, text[arg_start..offset].trim_start(), commas);

    let mut label = format!("{}(", sig.name);
    let mut parameters = Vec::new();
    for (i, param) in sig.params.iter().enumerate() {
        let first_named = param.named_only && (i == 0 || !sig.params[i - 1].named_only);
        if first_named {
            label.push_str(if i > 0 { "; " } else { ";" });
        } else if i > 0 {
            label.push_str(", ");
        }
        let start = utf16_len(&label);
        label.push_str(if param.variadic { "..." } else { "" });
        label.push_str(&param.name);
        if !param.ty.is_empty() {
            label.push_str(": ");
            label.push_str(&param.ty);
        }
        if !param.default.is_empty() {
            label.push_str(" = ");
            label.push_str(&param.default);
        }
        parameters.push(obj([(
            "label",
            vec![start.into(), utf16_len(&label).into()].into(),
        )]));
    }
    label.push(')');
    if !sig.ret.is_empty() && sig.ret != "()" {
        label.push_str(" -> ");
        label.push_str(&sig.ret);
    }
    let mut signature = BTreeMap::from([
        ("label".to_string(), label.into()),
        ("parameters".to_string(), Value::Array(parameters)),
        ("activeParameter".to_string(), (active as u32).into()),
    ]);
    if !sig.doc.is_empty() {
        signature.insert(
            "documentation".to_string(),
            obj([("kind", "markdown".into()), ("value", sig.doc.into())]),
        );
    }
    Some(obj([
        ("signatures", vec![Value::Object(signature)].into()),
        ("activeSignature", 0.into()),
        ("activeParameter", (active as u32).into()),
    ]))
}

/// The parameter an argument is for: `f(a, by: k`'s is `by`, else the
/// positional one at its place, the last when that one is variadic.
pub(super) fn active_param(sig: &Signature, given: &[&str], arg: &str, commas: usize) -> usize {
    let named = arg
        .split_once(':')
        .filter(|(n, _)| !n.trim().is_empty() && n.trim().chars().all(is_ident))
        .and_then(|(n, _)| sig.params.iter().position(|p| p.name == n.trim()));
    if let Some(index) = named {
        return index;
    }
    let positional: Vec<usize> = (0..sig.params.len())
        .filter(|&i| !sig.params[i].named_only)
        .collect();
    match positional.get(commas) {
        Some(&index) => index,
        None => match positional.last() {
            Some(&last) if sig.params[last].variadic => last,
            // Past the positional ones, the next option not given yet.
            _ => (0..sig.params.len())
                .find(|&i| {
                    sig.params[i].named_only && !given.iter().any(|g| *g == sig.params[i].name)
                })
                .unwrap_or(sig.params.len().max(commas)),
        },
    }
}

/// The names of the named arguments in `args`, the text before the current one.
pub(super) fn named_args(args: &str) -> Vec<&str> {
    let mut names = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;
    for (i, c) in args.char_indices() {
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                if let Some((name, _)) = args[start..i].split_once(':') {
                    let name = name.trim();
                    if !name.is_empty() && name.chars().all(is_ident) {
                        names.push(name);
                    }
                }
                start = i + 1;
            }
            _ => {}
        }
    }
    names
}

fn utf16_len(s: &str) -> u32 {
    s.encode_utf16().count() as u32
}

pub(super) fn signature_of(
    cursor: &Cursor,
    std: &mut StdLib,
    item: &AnalyzedItem,
    callee_end: usize,
    name: &str,
) -> Option<Signature> {
    let text = &*cursor.analysis.text;
    let start = (callee_end - name.len()) as u32;
    let method = text[..start as usize].ends_with('.');
    // The declaration, for its doc comment and its parameters as written.
    let declared: Option<Entry> = if method {
        receiver_type(cursor, start - 1).and_then(|ty| {
            members_of_ty(cursor, std, &ty)
                .into_iter()
                .map(|(e, _)| e)
                .find(|e| e.name == name)
        })
    } else {
        scope_entry(cursor, std, name)
    };
    let rel = callee_end as u32 - item.offset;
    let checked = item
        .check
        .hovers
        .iter()
        .rev()
        .find(|e| e.span.end == rel && e.name == name)
        .map(|e| e.ty.clone());
    // A method neither the checker resolved nor the receiver's type offers
    // (another module's internals).
    if method && declared.is_none() && !matches!(checked, Some(Ty::Fn { .. })) {
        return None;
    }
    let doc = declared.as_ref().map_or(String::new(), |e| e.doc.clone());
    match checked {
        // A library fn called as a method is checked as its free form, whose
        // instantiated types lose the bounds and `yields`: the declaration, when
        // the receiver's type offers one, wins.
        Some(Ty::Fn { params, ret })
            if !(method && declared.is_some() && params.iter().any(|p| p.name == "self")) =>
        {
            Some(Signature {
                name: name.to_string(),
                params: params
                    .iter()
                    .filter(|p| p.name != "self")
                    .map(|p| {
                        let written = declared
                            .as_ref()
                            .and_then(|e| e.params.iter().find(|w| w.name == p.name));
                        let default = match written {
                            Some(w) if !w.default.is_empty() => w.default.clone(),
                            _ if p.has_default => "...".to_string(),
                            _ => String::new(),
                        };
                        SigParam {
                            name: p.name.clone(),
                            ty: p.ty.to_string(),
                            variadic: p.variadic,
                            default,
                            named_only: p.named_only,
                        }
                    })
                    .collect(),
                ret: ret.to_string(),
                doc,
            })
        }
        _ => {
            let entry = declared?;
            let (params, ret) = match entry.kind {
                Kind::Fn | Kind::Method => (
                    entry
                        .params
                        .iter()
                        .map(|p| SigParam {
                            name: p.name.clone(),
                            ty: p.ty.clone(),
                            variadic: false,
                            default: p.default.clone(),
                            named_only: p.named_only,
                        })
                        .collect(),
                    ret_of(&entry.detail).unwrap_or("").to_string(),
                ),
                // A struct called as its constructor takes its fields.
                Kind::Struct => (
                    entry
                        .members
                        .iter()
                        .filter(|m| m.kind == Kind::Field)
                        .map(|m| {
                            let ty = m.detail.split_once(": ").map_or("", |(_, t)| t);
                            SigParam {
                                name: m.name.clone(),
                                ty: ty.to_string(),
                                variadic: false,
                                default: String::new(),
                                named_only: false,
                            }
                        })
                        .collect(),
                    entry.name.clone(),
                ),
                _ => return None,
            };
            Some(Signature {
                name: name.to_string(),
                params,
                ret,
                doc: entry.doc,
            })
        }
    }
}

/// The declaration a bare name refers to: this file's, an import's or the prelude's.
fn scope_entry(cursor: &Cursor, std: &mut StdLib, name: &str) -> Option<Entry> {
    if let Some(entry) = outline_of(cursor.analysis).find(name) {
        return Some(entry.clone());
    }
    if let Some(import) = cursor.analysis.imports().get(name) {
        let module = import.module;
        let imported = import.name.clone();
        return module_outline(cursor, std, module)?
            .find(&imported)
            .cloned();
    }
    let prelude = std.prelude(cursor.uri);
    let (module, _) = prelude.iter().find(|(_, n)| n == name)?;
    std.module(cursor.uri, module)?.find(name).cloned()
}

// ---------------------------------------------------------- parameter hints

/// `name:` before each literal argument of a call to a fn with several
/// parameters: `pow(2, 10)` reads as `pow(base: 2, exp: 10)`.
pub fn parameter_hints(analysis: &Analysis) -> Vec<(u32, String)> {
    let mut hints = Vec::new();
    for item in &analysis.items {
        let mut calls = Calls {
            item,
            hints: Vec::new(),
        };
        for stmt in &item.syntax.parsed.stmts {
            walk_stmt(stmt, &mut calls);
        }
        hints.extend(
            calls
                .hints
                .into_iter()
                .map(|(at, label)| (item.offset + at, label)),
        );
    }
    hints
}

struct Calls<'a> {
    item: &'a AnalyzedItem,
    hints: Vec<(u32, String)>,
}

fn is_literal(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Int(_)
        | ExprKind::Float(_)
        | ExprKind::Str(_)
        | ExprKind::Char(_)
        | ExprKind::Bool(_)
        | ExprKind::None => true,
        ExprKind::Unary { expr, .. } => is_literal(expr),
        _ => false,
    }
}

impl Visitor for Calls<'_> {
    fn expr(&mut self, expr: &Expr) {
        let ExprKind::Call { callee, args } = &expr.kind else {
            return;
        };
        let Some(Ty::Fn { params, .. }) = type_of(self.item, callee) else {
            return;
        };
        let params: Vec<_> = params.iter().filter(|p| p.name != "self").collect();
        if params.len() < 2 {
            return;
        }
        for (arg, param) in args.iter().zip(&params) {
            if arg.name.is_some() || param.variadic {
                break;
            }
            if is_literal(&arg.value) && !param.name.is_empty() && !param.name.starts_with('_') {
                self.hints
                    .push((arg.value.span.start, format!("{}:", param.name)));
            }
        }
    }
}
