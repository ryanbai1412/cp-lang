//! Completion, signature help and parameter-name hints.
//!
//! Types come from the checker: every name and field it resolved left a
//! [`HoverEntry`] with its type, even in an item that does not parse to
//! the end (`xs.` is a field access with an empty name). Signatures and
//! doc comments come from the declarations, through [`Outline`].

use crate::outline::{Entry, Kind, Outline, Param};
use cpl_check::{
    builtin_members, prelude_names, Analysis, AnalyzedItem, BuiltinMember, DeclKind, FsLoader,
    HoverEntry, Loader, MemberKind, PreludeKind, Receiver, Ty,
};
use cpl_json::{obj, Value};
use cpl_syntax::ast::{Expr, ExprKind, Stmt, StmtKind};
use cpl_syntax::visit::{walk_stmt, Visitor};
use std::collections::BTreeMap;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Every `spec/std/` module but the prelude.
const STD_MODULES: &[&str] = include!(concat!(env!("OUT_DIR"), "/std_modules.rs"));

/// The members every domain has (`V.first`) and every position has
/// (`v.ord`), which the checker types without a declaration to read.
/// The tests check each against the checker.
pub(crate) const DOMAIN_MEMBERS: &[(&str, &str)] = &[
    ("first", "the first position"),
    ("last", "the last position"),
    ("rev", "the positions in reverse order"),
    ("cuts", "the boundaries between and around positions"),
];
pub(crate) const POS_MEMBERS: &[(&str, &str)] = &[
    ("ord", "the 0-based rank of the position"),
    ("to_idx", "the position as a raw 0-based index"),
    ("idx", "the position as a raw 0-based index"),
    ("label", "the value the position stands for"),
    ("l", "the boundary before the position"),
    ("r", "the boundary after the position"),
    ("before", "the boundary before the position"),
    ("after", "the boundary after the position"),
];

pub(crate) const RANGE_MEMBERS: &[(&str, &str)] = &[
    ("start", "the lower endpoint of the range"),
    ("end", "the exclusive upper endpoint of the range"),
    ("first", "the first element, or `none` when empty"),
    ("last", "the last element, or `none` when empty"),
    ("len", "the number of elements in the range"),
    ("is_empty", "true when the range holds no elements"),
    ("rev", "the range in reverse order"),
];

/// The standard library's outlines, read once.
#[derive(Default)]
pub struct StdLib {
    modules: HashMap<String, Option<Arc<Outline>>>,
    prelude: Option<Arc<Vec<(String, String)>>>,
}

impl StdLib {
    /// `std.<name>`, as the file at `from` would load it.
    pub fn module(&mut self, from: &str, name: &str) -> Option<Arc<Outline>> {
        self.modules
            .entry(name.to_string())
            .or_insert_with(|| {
                let path: Vec<String> = std::iter::once("std")
                    .chain(name.split('.'))
                    .map(str::to_string)
                    .collect();
                let (_, text) = FsLoader.load(from, &path)?;
                Some(Arc::new(Outline::of_source(&text)))
            })
            .clone()
    }

    /// The names `spec/std/prelude.cp` brings in, with the module of each.
    pub fn prelude(&mut self, from: &str) -> Arc<Vec<(String, String)>> {
        if let Some(prelude) = &self.prelude {
            return prelude.clone();
        }
        let names: Vec<(String, String)> = self
            .module(from, "prelude")
            .map(|outline| {
                outline
                    .uses
                    .iter()
                    .filter(|(path, _)| path.first().is_some_and(|p| p == "std"))
                    .flat_map(|(path, names)| {
                        let module = path[1..].join(".");
                        names.iter().map(move |n| (module.clone(), n.clone()))
                    })
                    .collect()
            })
            .unwrap_or_default();
        let names = Arc::new(names);
        self.prelude = Some(names.clone());
        names
    }
}

/// What one request looks at: a file's analysis and the cursor in it.
pub struct Cursor<'a> {
    pub uri: &'a str,
    pub analysis: &'a Analysis,
    pub offset: u32,
}

fn item_at(analysis: &Analysis, offset: u32) -> Option<&AnalyzedItem> {
    analysis
        .items
        .iter()
        .rev()
        .find(|item| item.offset <= offset)
}

fn outline_of(analysis: &Analysis) -> Outline {
    Outline::of_items(
        &analysis.text,
        analysis
            .items
            .iter()
            .map(|item| (item.offset, &item.syntax.parsed.stmts[..])),
    )
}

fn is_ident(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// The identifier ending at `end`.
fn ident_before(text: &str, end: usize) -> &str {
    let start = text[..end]
        .rfind(|c: char| !is_ident(c))
        .map_or(0, |i| i + 1);
    &text[start..end]
}

// ---------------------------------------------------------------- completion

pub fn completion(cursor: &Cursor, std: &mut StdLib) -> Vec<Value> {
    let text = &*cursor.analysis.text;
    let offset = (cursor.offset as usize).min(text.len());
    let line = &text[text[..offset].rfind('\n').map_or(0, |i| i + 1)..offset];
    if in_comment_or_string(line) {
        return Vec::new();
    }
    let word = ident_before(line, line.len());
    let head = &line[..line.len() - word.len()];
    if let Some(path) = line.trim_start().strip_prefix("use ") {
        return use_completion(cursor, std, path, word);
    }
    let expected = expected_type(cursor, std, offset - word.len());
    let rank = |entry: &Entry, sort: u8| match &expected {
        Some(want)
            if type_of_entry(entry)
                .or_else(|| inferred_type(cursor.analysis, entry))
                .is_some_and(|ty| same_type(&ty, want)) =>
        {
            0
        }
        _ => sort + 1,
    };
    if head.ends_with('.') && !head.ends_with("..") {
        let dot = (offset - word.len() - 1) as u32;
        return dot_completion(cursor, std, dot)
            .into_iter()
            .map(|(entry, sort)| item_json(&entry, rank(&entry, sort)))
            .collect();
    }
    scope_completion(cursor, std, word, &rank)
}

/// The type the expression starting at `at` should have: the parameter it
/// is an argument for, the written type of the `let` it initializes, or the
/// type of the variable it is assigned to.
fn expected_type(cursor: &Cursor, std: &mut StdLib, at: usize) -> Option<String> {
    let text = &*cursor.analysis.text;
    let item = item_at(cursor.analysis, cursor.offset);
    let line_start = text[..at].rfind('\n').map_or(0, |i| i + 1);
    let call = item.and_then(|item| Some((item, open_call(text, item.offset as usize, at)?)));
    let start = call.map_or(line_start, |(_, (.., arg))| arg.max(line_start));
    // After `r.` the expected type is the whole `r.member`'s.
    let before = text[start..at]
        .trim_end_matches(|c| is_ident(c) || c == '.')
        .trim();
    if let Some((item, (paren, commas, arg_start))) = call {
        let callee_end = text[..paren].trim_end().len();
        let name = ident_before(text, callee_end);
        if !name.is_empty() && !before.contains(['(', '[', '{']) {
            let sig = signature_of(cursor, std, item, callee_end, name)?;
            let given = named_args(&text[paren + 1..arg_start]);
            let index = active_param(&sig, &given, text[arg_start..at].trim_start(), commas);
            let ty = &sig.params.get(index)?.ty;
            return Some(ty.clone()).filter(|t| !t.is_empty());
        }
    }
    let lhs = before.strip_suffix('=')?;
    if lhs.ends_with(['=', '!', '<', '>']) {
        return None;
    }
    let lhs = lhs
        .trim_end_matches(['+', '-', '*', '/', '%', '^', '|', '&'])
        .trim();
    let decl = lhs
        .strip_prefix("let ")
        .or_else(|| lhs.strip_prefix("var "));
    if let Some((_, ty)) = decl.and_then(|d| d.split_once(':')) {
        return Some(ty.trim().to_string());
    }
    if decl.is_none() && !lhs.is_empty() && lhs.chars().all(is_ident) {
        if let Some((_, ty)) = locals(cursor).into_iter().find(|(name, _)| name == lhs) {
            return Some(ty.to_string());
        }
        let entry = outline_of(cursor.analysis).find(lhs)?.clone();
        return type_of_entry(&entry).or_else(|| inferred_type(cursor.analysis, &entry));
    }
    None
}

/// What the entry gives when used as a value: a variable's or field's
/// type, or what a fn returns.
fn type_of_entry(entry: &Entry) -> Option<String> {
    let detail = entry.detail.as_str();
    match entry.kind {
        Kind::Fn | Kind::Method | Kind::Getter if detail.contains("->") => {
            detail.rsplit_once("->").map(|(_, r)| r.trim().to_string())
        }
        Kind::Field | Kind::Value | Kind::Const | Kind::Getter => {
            let rest = detail
                .trim_start_matches("pub ")
                .trim_start_matches("let ")
                .trim_start_matches("var ")
                .trim_start_matches("const ");
            let (_, ty) = rest.split_once(':')?;
            Some(ty.split(" = ").next()?.trim().to_string())
        }
        _ => None,
    }
}

/// The checked type of a top-level `let` or `var` written without one.
fn inferred_type(analysis: &Analysis, entry: &Entry) -> Option<String> {
    if entry.kind != Kind::Value {
        return None;
    }
    analysis.items.iter().find_map(|item| {
        item.check
            .hovers
            .iter()
            .find(|e| {
                e.decl == Some(e.span) && e.name == entry.name && matches!(e.keyword, "let" | "var")
            })
            .filter(|e| !e.ty.has_vague())
            .map(|e| e.ty.to_string())
    })
}

fn same_type(a: &str, b: &str) -> bool {
    a.split_whitespace().eq(b.split_whitespace())
}

/// A line-comment, or an odd number of quotes, before the cursor.
fn in_comment_or_string(line: &str) -> bool {
    let mut in_string = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' if in_string => {
                chars.next();
            }
            '"' => in_string = !in_string,
            '/' if !in_string && chars.peek() == Some(&'/') => return true,
            _ => {}
        }
    }
    in_string
}

fn item_json(entry: &Entry, sort: u8) -> Value {
    let mut members = BTreeMap::from([
        ("label".to_string(), entry.name.as_str().into()),
        ("kind".to_string(), entry.kind.completion_kind().into()),
        ("detail".to_string(), entry.detail.as_str().into()),
        (
            "sortText".to_string(),
            format!("{sort}{}", entry.name).into(),
        ),
    ]);
    if !entry.doc.is_empty() {
        members.insert(
            "documentation".to_string(),
            obj([
                ("kind", "markdown".into()),
                ("value", entry.doc.as_str().into()),
            ]),
        );
    }
    Value::Object(members)
}

fn plain(name: &str, kind: Kind, detail: String, doc: &str) -> Entry {
    Entry {
        name: name.to_string(),
        kind,
        public: true,
        detail,
        doc: doc.to_string(),
        params: Vec::new(),
        instance: true,
        members: Vec::new(),
    }
}

fn use_completion(cursor: &Cursor, std: &mut StdLib, path: &str, word: &str) -> Vec<Value> {
    let path = path.trim_start();
    // `use std.seq.{zip, sc`: the public names of `std.seq`.
    if let Some((module, _)) = path.split_once(".{") {
        let Some(module) = module.strip_prefix("std.") else {
            return Vec::new();
        };
        let Some(outline) = std.module(cursor.uri, module) else {
            return Vec::new();
        };
        return outline
            .entries
            .iter()
            .filter(|e| e.public && !cpl_check::is_list_form_hook(&e.name))
            .map(|e| item_json(e, 0))
            .collect();
    }
    let typed = &path[..path.len() - word.len()];
    match typed {
        "" => vec![item_json(
            &plain("std", Kind::Module, "the standard library".into(), ""),
            0,
        )],
        "std." => STD_MODULES
            .iter()
            .map(|name| {
                let doc = std
                    .module(cursor.uri, name)
                    .map(|o| o.doc.clone())
                    .unwrap_or_default();
                let summary = doc.lines().next().unwrap_or("").to_string();
                item_json(&plain(name, Kind::Module, summary, &doc), 0)
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// The types of the declarations a module exports, by the module's outline.
fn module_outline(cursor: &Cursor, std: &mut StdLib, index: usize) -> Option<Arc<Outline>> {
    let module = cursor.analysis.modules().get(index)?;
    match module.name.strip_prefix("std.") {
        Some(name) => std.module(cursor.uri, name),
        None => Some(Arc::new(outline_of(&module.analysis))),
    }
}

/// The members to offer after the `.` at `dot`, each with its sort group.
fn dot_completion(cursor: &Cursor, std: &mut StdLib, dot: u32) -> Vec<(Entry, u8)> {
    let analysis = cursor.analysis;
    let receiver = ident_before(&analysis.text, dot as usize);
    if let Some(&index) = analysis.aliases().get(receiver) {
        if local_type(cursor, receiver, dot).is_none() {
            let Some(outline) = module_outline(cursor, std, index) else {
                return Vec::new();
            };
            return outline
                .entries
                .iter()
                .filter(|e| e.public)
                .map(|e| (e.clone(), 0))
                .collect();
        }
    }
    match receiver_type(cursor, dot) {
        Some(ty) => members_of_ty(cursor, std, &ty),
        None => Vec::new(),
    }
}

/// The type of the expression that ends just before the `.` at `dot`.
pub fn receiver_type(cursor: &Cursor, dot: u32) -> Option<Ty> {
    let item = item_at(cursor.analysis, dot)?;
    let rel = dot - item.offset;
    let mut finder = FieldBase {
        dot: rel,
        base: None,
    };
    for stmt in &item.syntax.parsed.stmts {
        walk_stmt(stmt, &mut finder);
    }
    if let Some(base) = &finder.base {
        if let Some(ty) = type_of(item, base) {
            return Some(ty);
        }
    }
    let name = ident_before(&cursor.analysis.text, dot as usize);
    local_type(cursor, name, dot)
}

/// The latest declaration of `name` at or before `at` in its item, else at top level.
fn local_type(cursor: &Cursor, name: &str, at: u32) -> Option<Ty> {
    if name.is_empty() {
        return None;
    }
    let named = |entry: &&HoverEntry| entry.name == name && !entry.ty.has_vague();
    let item = item_at(cursor.analysis, at)?;
    let rel = at - item.offset;
    item.check
        .hovers
        .iter()
        .filter(named)
        .filter(|e| e.span.end <= rel)
        .max_by_key(|e| e.span.end)
        .or_else(|| {
            cursor
                .analysis
                .items
                .iter()
                .flat_map(|i| i.check.hovers.iter())
                .find(|e| named(e) && e.decl == Some(e.span))
        })
        .map(|e| e.ty.clone())
}

struct FieldBase {
    dot: u32,
    base: Option<Expr>,
}

impl Visitor for FieldBase {
    fn expr(&mut self, expr: &Expr) {
        if let ExprKind::Field { base, name } = &expr.kind {
            if base.span.end == self.dot && name.span.start > self.dot {
                self.base = Some((**base).clone());
            }
        }
    }
}

fn hover_at(item: &AnalyzedItem, span: cpl_diag::Span) -> Option<&Ty> {
    item.check
        .hovers
        .iter()
        .rev()
        .find(|e| e.span == span)
        .map(|e| &e.ty)
}

/// What the checker found `expr` to be, from the types it left behind.
fn type_of(item: &AnalyzedItem, expr: &Expr) -> Option<Ty> {
    if let Some((_, ty)) = item
        .check
        .shapes
        .iter()
        .rev()
        .find(|(span, _)| *span == expr.span)
    {
        return (!matches!(ty, Ty::Error | Ty::Unknown)).then(|| ty.clone());
    }
    let ty = match &expr.kind {
        ExprKind::Paren(inner) => return type_of(item, inner),
        ExprKind::Field { name, .. } => hover_at(item, name.span)?.clone(),
        ExprKind::Generic { base, .. } => return type_of(item, base),
        ExprKind::Call { callee, .. } => match type_of(item, callee)? {
            Ty::Fn { ret, .. } => *ret,
            Ty::TypeName(name) => Ty::Named {
                name,
                args: Vec::new(),
            },
            _ => return None,
        },
        ExprKind::Index { base, .. } => match type_of(item, base)? {
            Ty::Array { elem, .. } => *elem,
            Ty::Named { name, args } if name == "List" && args.len() == 1 => args[0].clone(),
            Ty::Named { name, args } if name == "Map" && args.len() == 2 => args[1].clone(),
            _ => return None,
        },
        ExprKind::Unwrap(inner) | ExprKind::Try(inner) => match type_of(item, inner)? {
            Ty::Optional(inner) => *inner,
            _ => return None,
        },
        _ => hover_at(item, expr.span)?.clone(),
    };
    (!matches!(ty, Ty::Error | Ty::Unknown)).then_some(ty)
}

/// Every outline a type name could be declared in, nearest first.
fn outlines_for(cursor: &Cursor, std: &mut StdLib, type_name: &str) -> Vec<Arc<Outline>> {
    let mut outlines = vec![Arc::new(outline_of(cursor.analysis))];
    for index in 0..cursor.analysis.modules().len() {
        outlines.extend(module_outline(cursor, std, index));
    }
    for (module, name) in std.prelude(cursor.uri).iter() {
        if name == type_name {
            outlines.extend(std.module(cursor.uri, module));
        }
    }
    outlines
}

/// `T` in a signature of `SegTree<D, T>`, written as the type given for it.
fn substitute(text: &str, bindings: &[(String, String)]) -> String {
    if bindings.is_empty() {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut word = String::new();
    let flush = |word: &mut String, out: &mut String| {
        match bindings.iter().find(|(g, _)| g == word) {
            Some((_, ty)) => out.push_str(ty),
            None => out.push_str(word),
        }
        word.clear();
    };
    for c in text.chars() {
        if is_ident(c) {
            word.push(c);
        } else {
            flush(&mut word, &mut out);
            out.push(c);
        }
    }
    flush(&mut word, &mut out);
    out
}

/// `struct SegTree<D, T>(...)`: `D` and `T`.
fn generics_of(header: &str) -> Vec<String> {
    let Some(start) = header.find('<') else {
        return Vec::new();
    };
    let mut depth = 0;
    let mut end = header.len();
    for (i, c) in header[start..].char_indices() {
        match c {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 {
                    end = start + i;
                    break;
                }
            }
            _ => {}
        }
    }
    header[start + 1..end]
        .split(',')
        .map(|g| g.split(':').next().unwrap_or("").trim().to_string())
        .collect()
}

fn members_of_ty(cursor: &Cursor, std: &mut StdLib, ty: &Ty) -> Vec<(Entry, u8)> {
    let builtin = |members: &[(&str, &str)], ty: String| {
        members
            .iter()
            .map(|(name, doc)| (plain(name, Kind::Getter, ty.clone(), doc), 0))
            .collect()
    };
    match ty {
        Ty::Named { name, .. } if name == "Range" => builtin(RANGE_MEMBERS, ty.to_string()),
        Ty::Named { name, args } if builtin_receiver(ty).is_none() => {
            type_members(cursor, std, name, args, true)
        }
        Ty::TypeName(name) if is_domain(cursor.analysis, name) => {
            builtin(DOMAIN_MEMBERS, format!("domain {name}"))
        }
        Ty::TypeName(name) => type_members(cursor, std, name, &[], false),
        Ty::Record(fields) => fields
            .iter()
            .map(|(name, ty)| (plain(name, Kind::Field, format!("{name}: {ty}"), ""), 0))
            .collect(),
        Ty::Tuple(items) => items
            .iter()
            .enumerate()
            .map(|(i, ty)| (plain(&i.to_string(), Kind::Field, ty.to_string(), ""), 0))
            .collect(),
        Ty::Named { .. } | Ty::Str | Ty::Array { .. } if builtin_receiver(ty).is_some() => {
            let (receiver, bindings) = builtin_receiver(ty).expect("checked by the guard");
            builtin_members(receiver)
                .map(|member| (builtin_entry(member, &bindings), 0))
                .collect()
        }
        Ty::Domain(domain) => builtin(DOMAIN_MEMBERS, format!("domain {domain}")),
        Ty::Pos(domain) => builtin(POS_MEMBERS, format!("position in {domain}")),
        _ => Vec::new(),
    }
}

/// A domain named where a type goes, as in `read<[V.drop(1)]Int>()`.
fn is_domain(analysis: &Analysis, name: &str) -> bool {
    analysis
        .items
        .iter()
        .flat_map(|item| &item.syntax.decls)
        .any(|decl| decl.name == name && decl.kind == DeclKind::Domain)
}

/// The built-in type a value of `ty` is, with what its catalog's `T`, `K`,
/// `V` and `D` stand for here.
fn builtin_receiver(ty: &Ty) -> Option<(Receiver, Vec<(String, String)>)> {
    let bind = |names: &[&str], args: &[&Ty]| -> Vec<(String, String)> {
        names
            .iter()
            .zip(args)
            .map(|(name, ty)| (name.to_string(), ty.to_string()))
            .collect()
    };
    Some(match ty {
        Ty::Str => (Receiver::Str, bind(&["T"], &[&Ty::Char])),
        Ty::Array { index, elem } => {
            let index = index
                .iter()
                .map(Ty::to_string)
                .collect::<Vec<_>>()
                .join("][");
            let mut bindings = bind(&["T"], &[elem]);
            bindings.push(("D".to_string(), index));
            (Receiver::Array, bindings)
        }
        Ty::Named { name, args } => {
            let receiver = *Receiver::ALL.iter().find(|r| r.name() == name)?;
            let args: Vec<&Ty> = args.iter().collect();
            let names: &[&str] = if receiver == Receiver::Map {
                &["K", "V"]
            } else {
                &["T"]
            };
            (receiver, bind(names, &args))
        }
        _ => return None,
    })
}

/// A catalog row as a completion entry, its types written for this value.
fn builtin_entry(member: &BuiltinMember, bindings: &[(String, String)]) -> Entry {
    let params = split_params(member.params)
        .into_iter()
        .map(|param| {
            let (name, ty) = param.split_once(':').unwrap_or((param, ""));
            Param {
                name: name.trim().to_string(),
                ty: substitute(ty.trim(), bindings),
                default: String::new(),
                named_only: false,
            }
        })
        .collect();
    Entry {
        name: member.name.to_string(),
        kind: match member.kind {
            MemberKind::Get => Kind::Getter,
            MemberKind::Method => Kind::Method,
        },
        public: true,
        detail: substitute(&member.signature(), bindings),
        doc: member.doc.to_string(),
        params,
        instance: true,
        members: Vec::new(),
    }
}

/// `i: Int, f: (Int, Int) -> Int` split at its top-level commas.
fn split_params(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let (mut depth, mut start) = (0i32, 0);
    for (i, c) in text.char_indices() {
        match c {
            '(' | '[' | '<' | '{' => depth += 1,
            ')' | ']' | '>' | '}' if !text[..i].ends_with('-') => depth -= 1,
            ',' if depth == 0 => {
                out.push(text[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    if !text[start..].trim().is_empty() {
        out.push(text[start..].trim());
    }
    out
}

fn type_members(
    cursor: &Cursor,
    std: &mut StdLib,
    name: &str,
    args: &[Ty],
    instance: bool,
) -> Vec<(Entry, u8)> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for outline in outlines_for(cursor, std, name) {
        let Some(def) = outline.find(name) else {
            if outline.impls.iter().all(|(target, _)| target != name) {
                continue;
            }
            for member in outline.members_of(name) {
                if member.instance == instance && seen.insert(member.name.clone()) {
                    out.push((member.clone(), 1));
                }
            }
            continue;
        };
        let generics = generics_of(&def.detail);
        let bindings: Vec<(String, String)> = if generics.len() == args.len() {
            generics
                .into_iter()
                .zip(args.iter().map(Ty::to_string))
                .collect()
        } else {
            Vec::new()
        };
        for member in outline.members_of(name) {
            let wanted = match member.kind {
                Kind::Variant | Kind::Domain | Kind::Const => !instance,
                _ => member.instance == instance,
            };
            if !wanted || !seen.insert(member.name.clone()) {
                continue;
            }
            let mut member = member.clone();
            member.detail = substitute(&member.detail, &bindings);
            for param in &mut member.params {
                param.ty = substitute(&param.ty, &bindings);
            }
            let sort = match member.kind {
                Kind::Field | Kind::Getter => 0,
                _ => 1,
            };
            out.push((member, sort));
        }
        if !out.is_empty() {
            break;
        }
    }
    out
}

fn scope_completion(
    cursor: &Cursor,
    std: &mut StdLib,
    word: &str,
    rank: &dyn Fn(&Entry, u8) -> u8,
) -> Vec<Value> {
    let analysis = cursor.analysis;
    let mut seen = HashSet::new();
    let mut items = Vec::new();
    let mut add = |entry: &Entry, sort: u8, items: &mut Vec<Value>| {
        if entry.name != word && seen.insert(entry.name.clone()) {
            items.push(item_json(entry, rank(entry, sort)));
        }
    };
    for (name, ty) in locals(cursor) {
        add(
            &plain(&name, Kind::Value, format!("{name}: {ty}"), ""),
            0,
            &mut items,
        );
    }
    let own = outline_of(analysis);
    for entry in &own.entries {
        add(entry, 1, &mut items);
    }
    let mut imports: Vec<(&String, usize)> = analysis
        .imports()
        .iter()
        .map(|(name, import)| (name, import.module))
        .collect();
    imports.sort();
    for (name, module) in imports {
        if let Some(outline) = module_outline(cursor, std, module) {
            let import = &analysis.imports()[name];
            if let Some(entry) = outline.find(&import.name) {
                let mut entry = entry.clone();
                entry.name = name.clone();
                add(&entry, 2, &mut items);
            }
        }
    }
    let mut aliases: Vec<&String> = analysis.aliases().keys().collect();
    aliases.sort();
    for alias in aliases {
        let item = plain(alias, Kind::Module, format!("module {alias}"), "");
        add(&item, 2, &mut items);
    }
    for (module, name) in std.prelude(cursor.uri).iter() {
        if let Some(entry) = std
            .module(cursor.uri, module)
            .and_then(|o| o.find(name).cloned())
        {
            add(&entry, 3, &mut items);
        }
    }
    // The compiler's own names; the `spec/std/prelude.cp` ones came above
    // with their signatures and docs.
    for (name, kind) in prelude_names() {
        let (kind, detail) = match kind {
            PreludeKind::Type => (Kind::Struct, "built-in type"),
            PreludeKind::Trait => (Kind::Trait, "built-in trait"),
            PreludeKind::Value => (Kind::Fn, "built-in"),
            PreludeKind::Std(_) => continue,
        };
        add(&plain(name, kind, detail.into(), ""), 3, &mut items);
    }
    for keyword in cpl_syntax::KEYWORDS {
        add(
            &plain(keyword, Kind::Keyword, "keyword".into(), ""),
            4,
            &mut items,
        );
    }
    // A stdlib name the file doesn't import yet comes with its `use`.
    if word.len() >= 2 {
        let text = &cursor.analysis.text;
        for (name, module) in cpl_check::std_names() {
            if !name.starts_with(word) || seen.contains(*name) || cpl_check::is_list_form_hook(name)
            {
                continue;
            }
            let Some(entry) = std
                .module(cursor.uri, module)
                .and_then(|o| o.find(name).cloned())
            else {
                continue;
            };
            if !seen.insert(entry.name.clone()) {
                continue;
            }
            let (span, new_text) = cpl_check::import_edit(text, module, name);
            let mut item = item_json(&entry, rank(&entry, 4));
            if let Value::Object(members) = &mut item {
                members.insert(
                    "labelDetails".to_string(),
                    obj([("description", format!("std.{module}").into())]),
                );
                members.insert(
                    "additionalTextEdits".to_string(),
                    vec![obj([
                        (
                            "range",
                            crate::server::range_json(span, text, &cursor.analysis.lines),
                        ),
                        ("newText", new_text.into()),
                    ])]
                    .into(),
                );
            }
            items.push(item);
        }
    }
    items
}

/// The fn bodies of an item: a local declared in one is seen only in it.
struct FnSpans(Vec<cpl_diag::Span>);

impl Visitor for FnSpans {
    fn stmt(&mut self, stmt: &Stmt) {
        if let StmtKind::Fn(_) = stmt.kind {
            self.0.push(stmt.span);
        }
    }
}

/// The locals in scope at the cursor, nearest last, with their types.
fn locals(cursor: &Cursor) -> Vec<(String, Ty)> {
    let Some(item) = item_at(cursor.analysis, cursor.offset) else {
        return Vec::new();
    };
    let rel = cursor.offset - item.offset;
    let mut fns = FnSpans(Vec::new());
    for stmt in &item.syntax.parsed.stmts {
        walk_stmt(stmt, &mut fns);
    }
    let innermost = |at: u32| {
        fns.0
            .iter()
            .filter(|s| s.start <= at && at <= s.end)
            .min_by_key(|s| s.end - s.start)
            .copied()
    };
    let here = innermost(rel);
    let mut out: Vec<(String, Ty)> = Vec::new();
    for entry in &item.check.hovers {
        let declares = entry.decl == Some(entry.span)
            && matches!(entry.keyword, "let" | "var" | "param" | "for");
        if !declares || entry.span.end >= rel || entry.ty.has_vague() {
            continue;
        }
        let owner = innermost(entry.span.start);
        if owner.is_some_and(|o| here.is_none_or(|h| !(o.start <= h.start && h.end <= o.end))) {
            continue;
        }
        out.retain(|(name, _)| *name != entry.name);
        out.push((entry.name.clone(), entry.ty.clone()));
    }
    out.reverse();
    out
}

// ------------------------------------------------------------ signature help

/// The innermost unclosed `(` before the cursor and how many of its
/// arguments are already finished.
fn open_call(text: &str, from: usize, to: usize) -> Option<(usize, usize, usize)> {
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
struct SigParam {
    name: String,
    ty: String,
    variadic: bool,
    default: String,
    named_only: bool,
}

/// One signature to show: its parameters' names and types as text.
struct Signature {
    name: String,
    params: Vec<SigParam>,
    ret: String,
    doc: String,
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
fn active_param(sig: &Signature, given: &[&str], arg: &str, commas: usize) -> usize {
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
fn named_args(args: &str) -> Vec<&str> {
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

fn signature_of(
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
    let doc = declared.as_ref().map_or(String::new(), |e| e.doc.clone());
    match checked {
        Some(Ty::Fn { params, ret }) => Some(Signature {
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
        }),
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
                    entry
                        .detail
                        .rsplit_once("->")
                        .map_or(String::new(), |(_, r)| r.trim().to_string()),
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
