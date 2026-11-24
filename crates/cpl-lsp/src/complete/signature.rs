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
    /// `ok` in `while ok: (T) -> Bool`, shown as the declaration writes it.
    pub(super) binding: String,
}

/// One signature to show: its parameters' names and types as text.
pub(super) struct Signature {
    pub(super) name: String,
    pub(super) params: Vec<SigParam>,
    pub(super) ret: String,
    pub(super) doc: String,
}

/// Where the name of the call the cursor is in starts.
pub fn callee_start(cursor: &Cursor) -> Option<u32> {
    let text = &*cursor.analysis.text;
    let offset = (cursor.offset as usize).min(text.len());
    let item = item_at(cursor.analysis, cursor.offset)?;
    let (paren, _, _) = open_call(text, item.offset as usize, offset)?;
    let callee_end = text[..paren].trim_end().len();
    Some((callee_end - ident_before(text, callee_end).len()) as u32)
}

/// `doc` is what hover shows for the callee, so the two agree.
pub fn signature_help(cursor: &Cursor, std: &mut StdLib, doc: Option<String>) -> Option<Value> {
    let text = &*cursor.analysis.text;
    let offset = (cursor.offset as usize).min(text.len());
    let item = item_at(cursor.analysis, cursor.offset)?;
    let (paren, commas, arg_start) = open_call(text, item.offset as usize, offset)?;
    let callee_end = text[..paren].trim_end().len();
    let name = ident_before(text, callee_end);
    if name.is_empty() {
        return None;
    }
    let mut sig = signature_of(cursor, std, item, callee_end, name)?;
    if let Some(doc) = doc {
        sig.doc = doc;
    }
    // `f(a, by: k`: the argument named, else the one by position.
    let given = named_args(&text[paren + 1..arg_start]);
    let arg = text[arg_start..offset].trim_start();
    let (sigs, chosen) = overloads_fitting(cursor, std, callee_end, name, sig, &given, arg);
    let mut active = 0;
    let signatures: Vec<Value> = (sigs.iter().enumerate())
        .map(|(i, sig)| {
            let (value, at) = signature_json(sig, active_param(sig, &given, arg, commas));
            if i == chosen {
                active = at;
            }
            value
        })
        .collect();
    Some(obj([
        ("signatures", signatures.into()),
        ("activeSignature", (chosen as u32).into()),
        ("activeParameter", (active as u32).into()),
    ]))
}

/// R06.1.6: every overload of the name, and which one the labels fit.
pub(super) fn overloads_fitting(
    cursor: &Cursor,
    std: &mut StdLib,
    callee_end: usize,
    name: &str,
    sig: Signature,
    given: &[&str],
    arg: &str,
) -> (Vec<Signature>, usize) {
    let overloads = overloads(cursor, std, callee_end, name);
    let sigs = if overloads.len() > 1 {
        (overloads.into_iter())
            .map(|e| Signature {
                name: name.to_string(),
                params: e.params.iter().map(written_param).collect(),
                ret: e.ret().unwrap_or("").to_string(),
                doc: e.doc,
            })
            .collect()
    } else {
        vec![sig]
    };
    // The argument being typed counts once its label is written (`cmp: |`).
    let typing = (arg.split_once(':'))
        .map(|(l, _)| l.trim())
        .filter(|l| !l.is_empty() && l.chars().all(|c| c.is_alphanumeric() || c == '_'));
    let labels: Vec<&str> = given.iter().copied().chain(typing).collect();
    let fits = |s: &Signature| {
        let named = |n: &str| s.params.iter().any(|p| p.name == n);
        (s.params.iter())
            .filter(|p| p.named_only && p.default.is_empty())
            .all(|p| labels.contains(&p.name.as_str()))
            && labels.iter().all(|g| named(g))
    };
    let chosen = sigs.iter().position(fits).unwrap_or(0);
    (sigs, chosen)
}

/// The fns a call's name may mean: its overloads (R06.1.6), as declared.
fn overloads(cursor: &Cursor, std: &mut StdLib, callee_end: usize, name: &str) -> Vec<Entry> {
    let start = (callee_end - name.len()) as u32;
    let is_fn = |e: &Entry| e.name == name && matches!(e.kind, Kind::Fn | Kind::Method);
    let found: Vec<Entry> = if cursor.analysis.text[..start as usize].ends_with('.') {
        receiver_type(cursor, start - 1).map_or(Vec::new(), |ty| {
            (members_of_ty(cursor, std, &ty).into_iter())
                .map(|(e, _)| e)
                .filter(is_fn)
                .collect()
        })
    } else {
        (outline_of(cursor.analysis).entries.iter())
            .filter(|e| is_fn(e))
            .cloned()
            .collect()
    };
    if found.len() > 1 {
        return found;
    }
    // A std name's overloads live in its module, which lists each once.
    let modules: Vec<_> = (std.prelude(cursor.uri).iter())
        .filter(|(_, n)| n == name)
        .map(|(m, _)| m.clone())
        .collect();
    let from_std: Vec<Entry> = (modules.iter())
        .filter_map(|m| std.module(cursor.uri, m))
        .flat_map(|o| o.entries.iter().filter(|e| is_fn(e)).cloned().collect::<Vec<_>>())
        .collect();
    match from_std.len() > 1 {
        true => from_std,
        false => found,
    }
}

/// One signature's LSP form, and the parameter `active` is.
fn signature_json(sig: &Signature, active: usize) -> (Value, usize) {
    let mut label = format!("{}(", sig.name);
    let mut parameters = Vec::new();
    for (i, param) in sig.params.iter().enumerate() {
        let first_named = param.named_only && (i == 0 || !sig.params[i - 1].named_only);
        if first_named {
            label.push_str("; ");
        } else if i > 0 {
            label.push_str(", ");
        }
        let start = utf16_len(&label);
        label.push_str(if param.variadic { "..." } else { "" });
        label.push_str(&param.name);
        if !param.binding.is_empty() {
            label.push(' ');
            label.push_str(&param.binding);
        }
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
            obj([
                ("kind", "markdown".into()),
                ("value", sig.doc.clone().into()),
            ]),
        );
    }
    (Value::Object(signature), active)
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
    // A struct with its own `new` is called by its name with `new`'s parameters.
    let new = (declared.as_ref())
        .filter(|e| !method && e.kind == Kind::Struct)
        .and_then(|e| (e.members.iter()).find(|m| m.name == "new" && !m.instance && m.public));
    if let Some(new) = new {
        return Some(Signature {
            name: name.to_string(),
            params: new.params.iter().map(written_param).collect(),
            ret: new.ret().unwrap_or("").to_string(),
            doc,
        });
    }
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
                            binding: written.map_or(String::new(), |w| w.binding.clone()),
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
                    entry.params.iter().map(written_param).collect(),
                    entry.ret().unwrap_or("").to_string(),
                ),
                // A struct called as its constructor takes its fields.
                Kind::Struct => (
                    entry
                        .members
                        .iter()
                        .filter(|m| m.kind == Kind::Field)
                        .map(|m| SigParam {
                            name: m.name.clone(),
                            ty: m.ty.clone(),
                            variadic: false,
                            default: String::new(),
                            named_only: false,
                            binding: String::new(),
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

/// A parameter as its declaration writes it.
fn written_param(p: &Param) -> SigParam {
    SigParam {
        name: p.name.clone(),
        ty: p.ty.clone(),
        variadic: false,
        default: p.default.clone(),
        named_only: p.named_only,
        binding: p.binding.clone(),
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
