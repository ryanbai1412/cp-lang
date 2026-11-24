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

/// Every `std/` module but the prelude.
const STD_MODULES: &[&str] = include!(concat!(env!("OUT_DIR"), "/std_modules.rs"));

/// The members every domain has (`V.first`) and every position has
/// (`v.ord`), which the checker types without a declaration to read.
/// The tests check each against the checker.
pub(crate) const DOMAIN_MEMBERS: &[(&str, &str)] = &[
    ("first", "the first position"),
    ("last", "the last position"),
    ("rev", "the positions in reverse order"),
    ("cuts", "the boundaries between and around positions"),
    ("at", "the position at a 0-based index, or `None` outside"),
    ("nth", "the position at a 0-based index"),
    ("wrap", "the position at an index taken modulo the size"),
    ("pairs", "every pair of positions `(u, v)` with `u < v`"),
    ("partition", "the positions split by a predicate"),
    ("filter", "the positions a predicate holds on"),
];
pub(crate) const POS_MEMBERS: &[(&str, &str)] = &[
    ("ord", "the 0-based rank of the position"),
    ("idx", "the position as a raw 0-based index"),
    ("label", "the value the position stands for"),
    ("l", "the boundary before the position"),
    ("r", "the boundary after the position"),
    ("before", "the boundary before the position"),
    ("after", "the boundary after the position"),
    ("prev", "the position before, or `None` at the first"),
    ("next", "the position after, or `None` at the last"),
];

pub(crate) const RANGE_MEMBERS: &[(&str, &str)] = &[
    ("start", "the lower endpoint of the range"),
    ("end", "the exclusive upper endpoint of the range"),
    ("first", "the first element, or `None` when empty"),
    ("last", "the last element, or `None` when empty"),
    ("len", "the number of elements in the range"),
    ("is_empty", "true when the range holds no elements"),
    ("rev", "the range in reverse order"),
];

/// The standard library's outlines, read once.
#[derive(Default)]
pub struct StdLib {
    modules: HashMap<String, Option<Arc<Outline>>>,
    prelude: Option<Arc<Vec<(String, String)>>>,
    files: HashMap<String, (Arc<str>, Arc<Outline>)>,
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

    /// The outline of the file at `path`, parsed again only when its text changes.
    pub fn outline_of(&mut self, path: &str, text: Arc<str>) -> Arc<Outline> {
        if let Some((seen, outline)) = self.files.get(path) {
            if Arc::ptr_eq(seen, &text) || *seen == text {
                return outline.clone();
            }
        }
        let outline = Arc::new(Outline::of_source(&text));
        self.files.insert(path.to_string(), (text, outline.clone()));
        outline
    }

    /// The names `std/prelude.cp` brings in, with the module of each.
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
        .char_indices()
        .rfind(|&(_, c)| !is_ident(c))
        .map_or(0, |(i, c)| i + c.len_utf8());
    &text[start..end]
}

// ---------------------------------------------------------------- completion

/// `trigger` is the character the client completes on without a typed
/// letter: `(` and `,` offer only the open call's argument names, `{` only
/// the items of a `use`, so neither pops up the whole scope.
pub fn completion(cursor: &Cursor, std: &mut StdLib, trigger: Option<char>) -> Vec<Value> {
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
    if head.ends_with('@') {
        return DECORATORS
            .iter()
            .map(|(name, detail)| item_json(&plain(name, Kind::Keyword, (*detail).into(), ""), 0))
            .collect();
    }
    if let Some(args) = lint_args(line) {
        return lint_completion(args);
    }
    match trigger {
        Some('(' | ',') => return argument_names(cursor, std, offset),
        Some('{') => return Vec::new(),
        _ => {}
    }
    let expected = expected_type(cursor, std, offset - word.len());
    let wants_type = type_position(line, head);
    let rank = |entry: &Entry, sort: u8| match &expected {
        _ if wants_type => match entry.kind {
            Kind::Struct | Kind::Enum | Kind::Domain | Kind::Trait | Kind::Alias => sort,
            _ => sort + 5,
        },
        Some(want) if fits(cursor.analysis, entry, want) => 0,
        _ => sort + 1,
    };
    if head.ends_with('.') && !head.ends_with("..") {
        let dot = (offset - word.len() - 1) as u32;
        return dot_completion(cursor, std, dot)
            .into_iter()
            .map(|(entry, sort)| item_json(&entry, rank(&entry, sort)))
            .collect();
    }
    let mut items = argument_names(cursor, std, offset);
    items.extend(scope_completion(cursor, std, word, &rank));
    items
}

/// `xs.sorted(b|`: each parameter of the open call that no argument gives
/// yet, offered as `name:`, the named-only ones first.
/// The closed set of decorators (spec/core/06-control-functions.md, 6.4).
const DECORATORS: &[(&str, &str)] = &[
    ("test", "tool role: a test fn `cpl test` runs"),
    (
        "alt",
        "tool role: `@alt(f, caps)`, another implementation of `f`",
    ),
    ("gen", "tool role: a stress generator"),
    ("check", "tool role: the checker of an output"),
    (
        "interactor",
        "tool role: the judge side of an interactive problem",
    ),
    ("memo", "`@memo(array)` / `@memo(map)`: cache a function"),
    ("get", "an O(1) read-only method called without parentheses"),
    ("cache", "on a struct field: interior mutability"),
    ("trusted", "`@trusted:` block: the unsafe tier"),
    ("cpp", "an item written in C++: the unsafe tier"),
    ("grader", "`@grader(..)`: a judge entry point"),
    (
        "laws",
        "`@laws(sample: n)`: stress sampling of a type's laws",
    ),
    (
        "tag",
        "`@tag(lit)` on a variant: the token it reads and prints as",
    ),
    (
        "droppable",
        "the result may be dropped without the unused-result lint",
    ),
    (
        "lint",
        "`@lint(name, from:, to:)` on a `pub fn`: a library's own lint",
    ),
];

/// The text after `with lint(` while the cursor is inside its parentheses.
fn lint_args(line: &str) -> Option<&str> {
    let at = line.rfind("lint(")?;
    let args = &line[at + "lint(".len()..];
    (!args.contains(')') && line[..at].trim_end().ends_with("with")).then_some(args)
}

/// In `with lint(..)`: the lints by name, then after `name:` the levels.
fn lint_completion(args: &str) -> Vec<Value> {
    if args.rsplit(',').next().unwrap_or("").contains(':') {
        return ["allow", "warn", "deny"]
            .iter()
            .map(|level| item_json(&plain(level, Kind::Keyword, "lint level".into(), ""), 0))
            .collect();
    }
    // `LINT_TITLES`, not `LINTS`: linking the whole table costs cpl.wasm 9 KB gzipped.
    cpl_lint::LINT_TITLES
        .iter()
        .map(|(name, title)| item_json(&plain(name, Kind::Const, (*title).into(), ""), 0))
        .collect()
}

fn argument_names(cursor: &Cursor, std: &mut StdLib, offset: usize) -> Vec<Value> {
    let text = &*cursor.analysis.text;
    let Some(item) = item_at(cursor.analysis, cursor.offset) else {
        return Vec::new();
    };
    let Some((paren, commas, arg_start)) = open_call(text, item.offset as usize, offset) else {
        return Vec::new();
    };
    if !text[arg_start..offset].trim_start().chars().all(is_ident) {
        return Vec::new();
    }
    let callee_end = text[..paren].trim_end().len();
    let name = ident_before(text, callee_end);
    if name.is_empty() {
        return Vec::new();
    }
    let Some(sig) = signature_of(cursor, std, item, callee_end, name) else {
        return Vec::new();
    };
    let given = named_args(&text[paren + 1..arg_start]);
    let mut by_position = commas.saturating_sub(given.len());
    let mut out = Vec::new();
    for param in &sig.params {
        if !param.named_only && by_position > 0 {
            by_position -= 1;
            continue;
        }
        if param.variadic || given.contains(&param.name.as_str()) {
            continue;
        }
        let mut detail = format!("{}: {}", param.name, param.ty);
        if !param.default.is_empty() {
            detail = format!("{detail} = {}", param.default);
        }
        let label = format!("{}:", param.name);
        let mut item = item_json(
            &plain(&label, Kind::Field, detail, ""),
            u8::from(!param.named_only),
        );
        if let Value::Object(members) = &mut item {
            let space = if text[..offset].ends_with(',') {
                " "
            } else {
                ""
            };
            members.insert("insertText".to_string(), format!("{space}{label} ").into());
        }
        out.push(item);
    }
    out
}

/// Whether `head`, the line before the word being typed, leaves the cursor
/// where a type goes: after a declaration's or parameter's `:`, after `->`,
/// in `<..>` type arguments, or after the `?` of an optional type.
fn type_position(line: &str, head: &str) -> bool {
    let head = head.trim_end_matches('?').trim_end();
    if head.ends_with("->") {
        return true;
    }
    if let Some(before) = head.strip_suffix('<') {
        return before.ends_with(is_ident);
    }
    if let Some(before) = head.strip_suffix(',') {
        // `Map<Int, |`: the innermost open bracket is a `<`.
        let mut depth = 0i32;
        for c in before.chars().rev() {
            match c {
                '>' | ')' | ']' => depth += 1,
                '(' | '[' if depth == 0 => return false,
                '<' if depth == 0 => return true,
                '<' | '(' | '[' => depth -= 1,
                _ => {}
            }
        }
        return false;
    }
    let Some(before) = head.strip_suffix(':') else {
        return false;
    };
    let code = line.trim_start();
    let decl = [
        "let ",
        "var ",
        "const ",
        "fn ",
        "pub fn ",
        "pub let ",
        "pub const ",
    ]
    .iter()
    .any(|kw| code.starts_with(kw));
    // A field of a struct: an indented `name:` alone on its line.
    let field = line.starts_with([' ', '\t']) && before.trim().chars().all(is_ident);
    (decl && !before.contains('=')) || (field && !before.trim().is_empty())
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
        if let Some((_, ty)) = locals(cursor)
            .into_iter()
            .find(|(name, ty)| name == lhs && !ty.has_vague())
        {
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
        Kind::Fn | Kind::Method | Kind::Getter if ret_of(detail).is_some() => {
            ret_of(detail).map(str::to_string)
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

/// Whether `entry` gives a value of type `want`: the same type, or where a
/// fn is wanted (`by: Int -> K`), a fn taking those parameters; a type
/// parameter like `K` takes anything.
fn fits(analysis: &Analysis, entry: &Entry, want: &str) -> bool {
    let ty = type_of_entry(entry).or_else(|| inferred_type(analysis, entry));
    if ty.as_deref().is_some_and(|ty| same_type(ty, want)) {
        return true;
    }
    let Some((want_params, want_ret)) = fn_type(want) else {
        return false;
    };
    let (params, ret) = match entry.kind {
        Kind::Fn => (
            entry.params.iter().map(|p| p.ty.as_str()).collect(),
            ret_of(&entry.detail).unwrap_or(""),
        ),
        _ => match ty.as_deref().and_then(fn_type) {
            Some(fn_ty) => fn_ty,
            None => return false,
        },
    };
    let loose = |have: &str, want: &str| {
        have.is_empty() || same_type(have, want) || {
            let mut chars = want.chars();
            want.len() <= 2
                && chars.next().is_some_and(|c| c.is_ascii_uppercase())
                && chars.all(|c| c.is_ascii_alphanumeric())
        }
    };
    params.len() == want_params.len()
        && params.iter().zip(&want_params).all(|(h, w)| loose(h, w))
        && loose(ret, want_ret)
}

/// `(A, B) -> R` as `([A, B], R)`; `A -> R` as `([A], R)`.
fn fn_type(ty: &str) -> Option<(Vec<&str>, &str)> {
    let bytes = ty.as_bytes();
    let mut depth = 0i32;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'-' if bytes.get(i + 1) == Some(&b'>') => {
                if depth == 0 {
                    let (from, to) = (ty[..i].trim(), ty[i + 2..].trim());
                    let params = match from.strip_prefix('(').and_then(|f| f.strip_suffix(')')) {
                        Some(inner) if !inner.trim().is_empty() => split_top(inner),
                        Some(_) => Vec::new(),
                        None => vec![from],
                    };
                    return Some((params, to));
                }
                i += 1;
            }
            b'(' | b'[' | b'{' | b'<' => depth += 1,
            b')' | b']' | b'}' | b'>' => depth -= 1,
            _ => {}
        }
        i += 1;
    }
    None
}

/// `inner` split at its top-level commas, each part trimmed.
fn split_top(inner: &str) -> Vec<&str> {
    let (mut parts, mut depth, mut start) = (Vec::new(), 0i32, 0);
    let bytes = inner.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' | b'[' | b'{' | b'<' => depth += 1,
            b'>' if i > 0 && bytes[i - 1] == b'-' => {}
            b')' | b']' | b'}' | b'>' => depth -= 1,
            b',' if depth == 0 => {
                parts.push(inner[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(inner[start..].trim());
    parts
}

fn same_type(a: &str, b: &str) -> bool {
    a.split_whitespace().eq(b.split_whitespace())
}

/// A line-comment, or an odd number of quotes, before the cursor.
fn in_comment_or_string(line: &str) -> bool {
    // Inside a string's `{..}` the cursor is in code again: `"{xs.|}"`.
    let (mut in_string, mut raw, mut depth) = (false, false, 0usize);
    let mut prev = ' ';
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if in_string && depth == 0 {
            match c {
                '\\' if !raw => {
                    chars.next();
                }
                '"' => in_string = false,
                '{' if !raw && chars.peek() == Some(&'{') => {
                    chars.next();
                }
                '{' if !raw => depth = 1,
                _ => {}
            }
        } else {
            match c {
                '"' if depth == 0 => {
                    in_string = true;
                    raw = prev == 'r';
                }
                '{' if depth > 0 => depth += 1,
                '}' if depth > 0 => depth -= 1,
                '/' if depth == 0 && chars.peek() == Some(&'/') => return true,
                _ => {}
            }
        }
        prev = c;
    }
    in_string && depth == 0
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
    let typed = &path[..path.len() - word.len()];
    // `use std.seq.{zip, sc` and `use std.seq.sc`: the public names of `std.seq`.
    let module = match path.split_once(".{") {
        Some((module, _)) => Some(module),
        None => typed.strip_suffix('.').filter(|m| m.contains('.')),
    };
    if let Some(module) = module {
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
        .map(|g| g.split([':', '=']).next().unwrap_or("").trim().to_string())
        .collect()
}

fn members_of_ty(cursor: &Cursor, std: &mut StdLib, ty: &Ty) -> Vec<(Entry, u8)> {
    let mut out = own_members_of_ty(cursor, std, ty);
    let mut seen: HashSet<String> = out.iter().map(|(e, _)| e.name.clone()).collect();
    // A domain named as a value walks its positions; a type with `each` is a Seq.
    let ty = match ty {
        Ty::TypeName(name) if is_domain(cursor.analysis, name) => &Ty::Domain(name.clone()),
        ty => ty,
    };
    let walks = seen.contains("each");
    for entry in std_methods(cursor, std) {
        if receives(&entry.detail, ty, walks) && seen.insert(entry.name.clone()) {
            out.push((entry, 1));
        }
    }
    out
}

/// The hover for a member the checker records no fact for, `s.runs` in
/// `s.runs()`: the declaration the receiver's members name, rendered as
/// completion and signature help show it.
pub fn member_hover(cursor: &Cursor, std: &mut StdLib) -> Option<cpl_check::Hover> {
    let text = &*cursor.analysis.text;
    let at = (cursor.offset as usize).min(text.len());
    let start = text[..at]
        .char_indices()
        .rev()
        .take_while(|&(_, c)| is_ident(c))
        .last()
        .map_or(at, |(i, _)| i);
    let end = at
        + text[at..]
            .chars()
            .take_while(|&c| is_ident(c))
            .map(char::len_utf8)
            .sum::<usize>();
    if start == end || !text[..start].ends_with('.') {
        return None;
    }
    let ty = receiver_type(cursor, start as u32 - 1)?;
    let entry = members_of_ty(cursor, std, &ty)
        .into_iter()
        .map(|(e, _)| e)
        .find(|e| e.name == text[start..end])?;
    Some(cpl_check::Hover {
        span: cpl_diag::Span {
            start: start as u32,
            end: end as u32,
        },
        // The doc's whole `cost:` line: `O(1) to make, O(n) to walk`, not just `O(1)`.
        text: match entry
            .doc
            .lines()
            .map(str::trim)
            .find(|l| l.starts_with("cost: O("))
        {
            Some(cost) => format!("{}\n{cost}", entry.detail),
            None => entry.detail,
        },
        doc: (!entry.doc.is_empty()).then_some(entry.doc),
    })
}

/// The top-level `fn name(self: ..)` of every std module: a program that
/// writes `.name` gets the fn without a `use`, so all of them are members.
fn std_methods(cursor: &Cursor, std: &mut StdLib) -> Vec<Entry> {
    let mut out = Vec::new();
    for (path, name) in cpl_check::std_methods() {
        let module: Vec<&str> = path[1..].iter().map(|n| n.text.as_str()).collect();
        let Some(outline) = std.module(cursor.uri, &module.join(".")) else {
            continue;
        };
        if let Some(entry) = outline.find(&name.text) {
            if entry.kind == Kind::Fn && entry.public && self_type(&entry.detail).is_some() {
                let mut entry = entry.clone();
                entry.kind = Kind::Method;
                entry.instance = true;
                out.push(entry);
            }
        }
    }
    out
}

/// `Int` in `fn f(g: Int -> Int) -> Int`: what follows the parameter list.
fn ret_of(detail: &str) -> Option<&str> {
    let open = detail.find('(')?;
    let mut depth = 0i32;
    for (i, c) in detail[open..].char_indices() {
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => {
                depth -= 1;
                if depth == 0 {
                    let rest = detail[open + i + 1..].trim_start();
                    return rest
                        .strip_prefix("->")
                        .map(|r| r.trim().trim_end_matches(':').trim());
                }
            }
            _ => {}
        }
    }
    None
}

/// `S` in `fn runs<S: Indexed, T>(self: S, ..)`.
fn self_type(detail: &str) -> Option<&str> {
    let (_, rest) = detail
        .split_once("(self: ")
        .or_else(|| detail.split_once("(mut self: "))?;
    let end = rest.find([',', ')']).unwrap_or(rest.len());
    Some(rest[..end].trim())
}

/// Whether `ty` can be the `self` of the fn whose signature is `detail`:
/// its generic's bound (`S: Indexed`) holds, or its type is `ty`'s.
fn receives(detail: &str, ty: &Ty, walks: bool) -> bool {
    let Some(self_ty) = self_type(detail) else {
        return false;
    };
    let generics = detail
        .split_once('<')
        .filter(|(head, _)| !head.contains('('))
        .and_then(|(_, rest)| rest.split_once(">("))
        .map_or("", |(g, _)| g);
    let bound = generics.split(',').find_map(|g| {
        let (name, bound) = g.split_once(':')?;
        (name.trim() == self_ty).then(|| bound.trim())
    });
    let is_generic = generics
        .split(',')
        .any(|g| g.split(':').next().unwrap_or("").trim() == self_ty);
    match bound {
        Some(bound) => bound.split('+').all(|b| match b.trim() {
            "Seq" => ty.is_seq() || walks,
            "Sized" => ty.is_sized(),
            "Bidir" => ty.is_bidir(),
            "Keyed" => ty.is_keyed(),
            "Indexed" => ty.is_indexed(),
            "IndexedMut" => ty.is_indexed_mut(),
            _ => false,
        }),
        None if is_generic => false,
        // `self: ?T`, `T` generic, takes every optional.
        None if matches!(ty, Ty::Optional(_))
            && self_ty.strip_prefix('?').is_some_and(|inner| {
                generics
                    .split(',')
                    .any(|g| g.split(':').next().unwrap_or("").trim() == inner)
            }) =>
        {
            true
        }
        None => {
            let name = self_ty.split('<').next().unwrap_or(self_ty);
            let ty = match ty {
                Ty::Range { .. } => &Ty::Int,
                ty => ty,
            };
            ty.to_string().split('<').next() == Some(name)
        }
    }
}

fn own_members_of_ty(cursor: &Cursor, std: &mut StdLib, ty: &Ty) -> Vec<(Entry, u8)> {
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
        Ty::Int | Ty::Range { .. } => scalar_members(cursor, std, "Int"),
        Ty::I128 => scalar_members(cursor, std, "I128"),
        Ty::U64 => scalar_members(cursor, std, "U64"),
        Ty::Float => scalar_members(cursor, std, "Float"),
        Ty::Bool => scalar_members(cursor, std, "Bool"),
        Ty::Char => scalar_members(cursor, std, "Char"),
        _ => Vec::new(),
    }
}

/// The methods of a built-in scalar: the `fn f(self: Int)` of this file and
/// its modules, and those the prelude brings in.
fn scalar_members(cursor: &Cursor, std: &mut StdLib, name: &str) -> Vec<(Entry, u8)> {
    let mut out = type_members(cursor, std, name, &[], true);
    let mut seen: HashSet<String> = out.iter().map(|(e, _)| e.name.clone()).collect();
    let prelude = std.prelude(cursor.uri);
    for (module, imported) in prelude.iter() {
        let Some(outline) = std.module(cursor.uri, module) else {
            continue;
        };
        for member in outline.members_of(name) {
            if member.instance
                && member.public
                && &member.name == imported
                && seen.insert(member.name.clone())
            {
                out.push((member.clone(), 1));
            }
        }
    }
    out
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
    let params = cpl_check::split_builtin_params(member.params)
        .into_iter()
        .map(|param| {
            let (name, ty) = param.split_once(':').unwrap_or((param, ""));
            let (ty, default) = ty.split_once(" = ").unwrap_or((ty, ""));
            Param {
                name: name.trim().to_string(),
                ty: substitute(ty.trim(), bindings),
                default: default.trim().to_string(),
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

fn type_members(
    cursor: &Cursor,
    std: &mut StdLib,
    name: &str,
    args: &[Ty],
    instance: bool,
) -> Vec<(Entry, u8)> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for (index, outline) in outlines_for(cursor, std, name).into_iter().enumerate() {
        // The first outline is this file's; any other is another module's,
        // where only what it offers its users is shown.
        let foreign = index > 0;
        let offered = |m: &Entry| !foreign || outline.offers(name, m);
        let Some(def) = outline.find(name) else {
            if outline.impls.iter().all(|(target, _)| target != name) {
                continue;
            }
            for member in outline.members_of(name) {
                if member.instance == instance
                    && offered(member)
                    && seen.insert(member.name.clone())
                {
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
            if !wanted || !offered(member) || !seen.insert(member.name.clone()) {
                continue;
            }
            let mut member = member.clone();
            member.detail = substitute(&member.detail, &bindings);
            for param in &mut member.params {
                param.ty = substitute(&param.ty, &bindings);
            }
            // A library type's own fields are its state; what it offers is
            // its methods, so they come first.
            let sort = match member.kind {
                Kind::Field if foreign => 2,
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
    // The word being typed is offered when it names something; only a word
    // that appears nowhere else (a new name) is left out.
    let new_word = whole_words(&analysis.text, word) <= 1;
    let mut add = |entry: &Entry, sort: u8, items: &mut Vec<Value>| {
        if !(new_word && entry.name == word) && seen.insert(entry.name.clone()) {
            items.push(item_json(entry, rank(entry, sort)));
        }
    };
    for (name, ty) in locals(cursor) {
        // A local the checker could not type is still in scope.
        let detail = match ty.has_vague() {
            true => name.clone(),
            false => format!("{name}: {ty}"),
        };
        add(&plain(&name, Kind::Value, detail, ""), 0, &mut items);
    }
    let own = outline_of(analysis);
    for entry in &own.entries {
        add(entry, 1, &mut items);
    }
    // A variant is in scope by its bare name: `enum Who = A | B`, then `A`.
    for entry in own.entries.iter().filter(|e| e.kind == Kind::Enum) {
        for variant in entry.members.iter().filter(|m| m.kind == Kind::Variant) {
            add(variant, 1, &mut items);
        }
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
    // The compiler's own names; the `std/prelude.cp` ones came above
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
    // `?T`'s variants are prelude names like any other (R01.3.6).
    for (name, detail) in [("Some", "Some(x: T) -> ?T"), ("None", "None: ?T")] {
        add(
            &plain(name, Kind::Variant, detail.into(), ""),
            3,
            &mut items,
        );
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

/// How many times `word` appears in `text` as a whole identifier.
fn whole_words(text: &str, word: &str) -> usize {
    if word.is_empty() {
        return 0;
    }
    text.match_indices(word)
        .filter(|&(at, _)| {
            let before = text[..at].chars().next_back();
            let after = text[at + word.len()..].chars().next();
            !before.is_some_and(is_ident) && !after.is_some_and(is_ident)
        })
        .count()
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
            && matches!(entry.keyword, "let" | "var" | "param" | "for" | "fn");
        if !declares || entry.span.end >= rel {
            continue;
        }
        // A nested fn's name is inside its own span; its scope is the fn around it.
        let owner = match entry.keyword {
            "fn" => {
                let mut around: Vec<_> = fns
                    .0
                    .iter()
                    .filter(|s| s.start <= entry.span.start && entry.span.start <= s.end)
                    .collect();
                around.sort_by_key(|s| s.end - s.start);
                match around.get(1) {
                    Some(outer) => Some(**outer),
                    None => continue,
                }
            }
            _ => innermost(entry.span.start),
        };
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

#[cfg(test)]
mod ident_tests {
    #[test]
    fn the_identifier_before_the_cursor_starts_after_a_multibyte_char() {
        assert_eq!(super::ident_before("é", 2), "");
        assert_eq!(super::ident_before("café x.ab", 10), "ab");
        assert_eq!(super::ident_before("éab", 4), "ab");
    }
}
