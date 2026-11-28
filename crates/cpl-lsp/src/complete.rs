//! Completion, signature help and parameter-name hints.
//!
//! Types come from the checker: every name and field it resolved left a
//! [`HoverEntry`] with its type, even in an item that does not parse to
//! the end (`xs.` is a field access with an empty name). Signatures and
//! doc comments come from the declarations, through [`Outline`].

use crate::outline::{Entry, Kind, Outline, Param};
use cpl_facade::today::{
    builtin_members, prelude_names, Analysis, AnalyzedItem, BuiltinMember, DeclKind, FsLoader,
    HoverEntry, Loader, MemberKind, PreludeKind, Receiver, Ty,
};
use cpl_json::{obj, Value};
use cpl_syntax::ast::{BinOp, Expr, ExprKind, Stmt, StmtKind};
use cpl_syntax::visit::{walk_stmt, Visitor};
use cpl_syntax::TokenKind;
use std::collections::BTreeMap;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

mod signature;
use signature::{active_param, named_args, open_call, signature_of};
pub use signature::{callee_start, parameter_hints, signature_help};

/// Every `std/` module but the prelude.
const STD_MODULES: &[&str] = include!(concat!(env!("OUT_DIR"), "/std_modules.rs"));

/// The standard library's outlines, read once.
#[derive(Default)]
pub struct StdLib {
    /// What the documents are checked against, for the std in it.
    given: cpl_facade::Snapshot,
    modules: HashMap<String, Option<Arc<Outline>>>,
    prelude: Option<Arc<Vec<(String, String)>>>,
    files: HashMap<String, (Arc<str>, Arc<Outline>)>,
}

impl StdLib {
    /// The outlines of the std `snapshot` is checked against.
    pub fn of(snapshot: &cpl_facade::Snapshot) -> StdLib {
        StdLib {
            given: cpl_facade::Snapshot::default().with_std_of(snapshot),
            ..StdLib::default()
        }
    }

    /// `std.<name>`, as the file at `from` would load it.
    pub fn module(&mut self, from: &str, name: &str) -> Option<Arc<Outline>> {
        self.modules
            .entry(name.to_string())
            .or_insert_with(|| {
                let path: Vec<String> = std::iter::once("std")
                    .chain(name.split('.'))
                    .map(str::to_string)
                    .collect();
                if let Some(text) = self.given.std_text(&name.replace('.', "/")) {
                    return Some(Arc::new(Outline::of_source(text)));
                }
                if let Some((_, text)) = FsLoader.load(from, &path) {
                    return Some(Arc::new(Outline::of_source(&text)));
                }
                let (len, _, text) = self.given.std_outline(&path)?;
                let mut outline = Outline::of_source(text);
                let marks = outline.entries.iter_mut().map(|e| (None, e));
                let owned = outline.impls.iter_mut().flat_map(|(target, entries)| {
                    entries.iter_mut().map(|e| (Some(target.clone()), e))
                });
                for (owner, entry) in marks.chain(owned) {
                    for member in &mut entry.members {
                        member.origin = Some((name.to_string(), Some(entry.name.clone())));
                    }
                    entry.origin = Some((name.to_string(), owner));
                }
                (len == path.len()).then(|| Arc::new(outline))
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

/// The docs of a completion item from a std module the host has given
/// since it was offered: `data` names the module, and the type of a member.
pub fn resolve(item: &Value, std: &mut StdLib) -> Option<String> {
    let module = item.at(&["data", "module"]).as_str()?;
    let label = item.at(&["label"]).as_str()?;
    let outline = std.module("", module)?;
    let entries: Vec<&Entry> = match item.at(&["data", "owner"]).as_str() {
        Some(owner) => outline
            .find(owner)
            .map(|e| e.members.iter())
            .into_iter()
            .flatten()
            .chain(
                outline
                    .impls
                    .iter()
                    .filter(|(target, _)| target == owner)
                    .flat_map(|(_, entries)| entries),
            )
            .collect(),
        None => outline.entries.iter().collect(),
    };
    let entry = entries.into_iter().find(|e| e.name == label)?;
    (entry.origin.is_none() && !entry.doc.is_empty()).then(|| entry.doc.clone())
}

/// What one request looks at: a file's analysis and the cursor in it.
pub struct Cursor<'a> {
    pub uri: &'a str,
    pub analysis: &'a Analysis,
    pub offset: u32,
    /// The checker's answer to whether `receiver.name` picks an extension fn,
    /// for each name (`Session::method_fits`).
    pub fit: Option<&'a MethodFits<'a>>,
}

/// `Session::method_fits` for the cursor's file.
pub type MethodFits<'a> = dyn Fn(&Ty, &[&str]) -> Vec<Option<bool>> + 'a;

fn item_at(analysis: &Analysis, offset: u32) -> Option<&AnalyzedItem> {
    analysis
        .items
        .iter()
        .rev()
        .find(|item| item.offset <= offset)
}

pub(crate) fn outline_of(analysis: &Analysis) -> Outline {
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
    if head.ends_with("@!") {
        return cpl_next_syntax::grammar::DIRECTIVES
            .iter()
            .map(|(name, detail)| item_json(&plain(name, Kind::Keyword, (*detail).into(), ""), 0))
            .collect();
    }
    if head.ends_with('@') {
        return cpl_next_syntax::grammar::DECORATORS
            .iter()
            .map(|(name, detail)| item_json(&plain(name, Kind::Keyword, (*detail).into(), ""), 0))
            .collect();
    }
    if let Some(args) = lint_args(line) {
        return lint_completion(args);
    }
    match trigger {
        Some('(' | ',') => return argument_names(cursor, std, offset),
        Some('{' | '@' | '!') => return Vec::new(),
        _ => {}
    }
    let expected = expected_type(cursor, std, offset - word.len());
    let wants_type = type_position(line, head);
    let wants_pattern = pattern_position(&text[..offset - word.len()]);
    let in_test = in_test_fn(cursor);
    let rank = |entry: &Entry, sort: u8| match &expected {
        _ if wants_pattern && entry.kind == Kind::Variant => 0,
        _ if in_test && entry.kind == Kind::Keyword && entry.name == "assert" => 0,
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
            .map(|(entry, sort)| member_json(&entry, rank(&entry, sort)))
            .collect();
    }
    let mut items = argument_names(cursor, std, offset);
    items.extend(scope_completion(cursor, std, word, &rank));
    items
}

/// `xs.sorted(b|`: each parameter of the open call that no argument gives
/// yet, offered as `name:`, the named-only ones first.

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
/// The start of a match arm: the line so far is only a pattern's start
/// (blank, or after `|`), under a `match ..:` header.
fn pattern_position(before: &str) -> bool {
    let (above, line) = before.rsplit_once('\n').unwrap_or(("", before));
    let head = line.trim_start();
    if !(head.is_empty() || head.trim_end().ends_with('|')) || head.contains("=>") {
        return false;
    }
    let indent = line.len() - head.len();
    for prev in above.lines().rev().filter(|l| !l.trim().is_empty()) {
        let prev_head = prev.trim_start();
        let prev_indent = prev.len() - prev_head.len();
        if prev_indent < indent {
            let header = prev_head.trim_end();
            return (header.starts_with("match ") || header.contains(" match "))
                && header.ends_with(':');
        }
        if prev_indent == indent && !prev_head.contains("=>") {
            return false;
        }
    }
    false
}

/// Inside a top-level fn marked `@test`.
fn in_test_fn(cursor: &Cursor) -> bool {
    item_at(cursor.analysis, cursor.offset).is_some_and(|item| {
        item.syntax.parsed.stmts.iter().any(|stmt| {
            matches!(stmt.kind, cpl_syntax::ast::StmtKind::Fn(_))
                && stmt.decorators.iter().any(|d| d.name.text == "test")
        })
    })
}

fn type_position(line: &str, head: &str) -> bool {
    let head = head.trim_end_matches('?').trim_end();
    let last = cpl_syntax::lex(head).tokens.into_iter().rev().find(|t| {
        !matches!(
            t.kind,
            TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof
        )
    });
    if last.is_some_and(|t| t.kind == TokenKind::Arrow) {
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
            let ty = sig.params.get(index)?.ty.as_str();
            // `cmp: ?((T, T) -> Bool) = None` takes the fn itself.
            let ty = ty.strip_prefix('?').map_or(ty, |inner| {
                (inner.strip_prefix('(').and_then(|t| t.strip_suffix(')')))
                    .filter(|t| t.contains("->"))
                    .unwrap_or(inner)
            });
            return Some(ty.to_string()).filter(|t| !t.is_empty());
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
        if let Some((_, ty, _)) = locals(cursor)
            .into_iter()
            .find(|(name, ty, _)| name == lhs && !ty.has_vague())
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
        Kind::Fn | Kind::Method | Kind::Getter if entry.ret().is_some() => {
            entry.ret().map(str::to_string)
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
            entry.ret().unwrap_or(""),
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
        ("detail".to_string(), detail_of(entry).into()),
        (
            "sortText".to_string(),
            format!("{sort}{}", entry.name).into(),
        ),
    ]);
    if let Some((module, owner)) = &entry.origin {
        let mut data = BTreeMap::from([("module".to_string(), module.as_str().into())]);
        data.extend(
            owner
                .iter()
                .map(|owner| ("owner".to_string(), owner.as_str().into())),
        );
        members.insert("data".to_string(), Value::Object(data));
    }
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

/// A member after `.`: within its group, the members std itself calls
/// most come first, so `len` and `map` lead a list's long tail.
fn member_json(entry: &Entry, sort: u8) -> Value {
    let mut item = item_json(entry, sort);
    let rare = 9999 - cpl_facade::Std.calls(&entry.name).min(9999);
    if let Value::Object(members) = &mut item {
        let text = format!("{sort}{rare:04}{}", entry.name);
        members.insert("sortText".to_string(), text.into());
    }
    item
}

/// An entry's completion detail. A struct built through its own `new` is
/// called by its name with `new`'s parameters, so its detail shows that call.
fn detail_of(entry: &Entry) -> String {
    let ctor = (entry.kind == Kind::Struct)
        .then(|| {
            entry
                .members
                .iter()
                .find(|m| m.name == "new" && !m.instance && m.public)
        })
        .flatten()
        .and_then(|new| {
            new.detail
                .split_once("new")
                .map(|(_, rest)| rest.to_string())
        });
    match ctor {
        Some(call) => format!("{}\n{}{call}", entry.detail, entry.name),
        None => entry.detail.clone(),
    }
}

fn plain(name: &str, kind: Kind, detail: String, doc: &str) -> Entry {
    Entry {
        name: name.to_string(),
        ty: String::new(),
        kind,
        public: true,
        detail,
        doc: doc.to_string(),
        params: Vec::new(),
        instance: true,
        members: Vec::new(),
        origin: None,
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
            .filter(|e| e.public && !cpl_facade::today::is_list_form_hook(&e.name))
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
        .or_else(|| declared_type(cursor, name))
}

/// What `let name = init` or `for name in iter` holds when the checker left
/// `name` untyped: `t.at.filter(p).collect()` is a list whatever the
/// context will want.
fn declared_type(cursor: &Cursor, name: &str) -> Option<Ty> {
    cursor.analysis.items.iter().find_map(|item| {
        let decl = item
            .check
            .hovers
            .iter()
            .find(|e| e.name == name && e.decl == Some(e.span))?;
        let mut finder = Declared {
            at: decl.span,
            found: None,
        };
        for stmt in &item.syntax.parsed.stmts {
            walk_stmt(stmt, &mut finder);
        }
        match finder.found? {
            (init, false) => type_of(item, &init),
            (iter, true) => type_of(item, &iter)?.seq_elem(),
        }
    })
}

/// The value bound at `at`, and whether a `for` walks it.
struct Declared {
    at: cpl_diag::Span,
    found: Option<(Expr, bool)>,
}

impl Visitor for Declared {
    fn stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Let {
                pat,
                init: Some(init),
                ty: None,
                ..
            } if pat.span == self.at => self.found = Some((init.clone(), false)),
            StmtKind::For { pat, iter, .. } if pat.span == self.at => {
                self.found = Some((iter.clone(), true))
            }
            _ => {}
        }
    }
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

/// What calling `callee` gives.
fn call_type(item: &AnalyzedItem, callee: &Expr) -> Option<Ty> {
    match type_of(item, callee)? {
        Ty::Fn { ret, .. } => Some(*ret),
        Ty::TypeName(name) => Some(Ty::Named {
            name,
            args: Vec::new(),
        }),
        _ => None,
    }
}

/// What the checker found `expr` to be, from the types it left behind.
fn type_of(item: &AnalyzedItem, expr: &Expr) -> Option<Ty> {
    let compound = matches!(
        expr.kind,
        ExprKind::Call { .. }
            | ExprKind::Index { .. }
            | ExprKind::Paren(_)
            | ExprKind::Range { .. }
            | ExprKind::Unwrap(_)
            | ExprKind::Try(_)
    );
    let typed = (item.check.exprs.types.iter().rev()).find(|(span, _)| *span == expr.span);
    if let Some((_, ty)) = typed.filter(|_| compound) {
        if !matches!(ty, Ty::Error | Ty::Unknown) {
            return Some(ty.clone());
        }
    }
    let ty = match &expr.kind {
        ExprKind::Paren(inner) => return type_of(item, inner),
        ExprKind::Binary {
            op: BinOp::Coalesce,
            lhs,
            rhs,
            ..
        } => match type_of(item, lhs) {
            Some(Ty::Optional(inner)) => *inner,
            _ => type_of(item, rhs)?,
        },
        ExprKind::Call { callee, args } if args.is_empty() => match &callee.kind {
            ExprKind::Field { base, name } if name.text == "collect" => {
                match type_of(item, base)? {
                    ty @ Ty::Array { .. } => ty,
                    ty => Ty::list(ty.seq_elem()?),
                }
            }
            _ => call_type(item, callee)?,
        },
        ExprKind::Str(_) => Ty::Str,
        ExprKind::Int(_) => Ty::Int,
        ExprKind::Bool(_) => Ty::Bool,
        ExprKind::Char(_) => Ty::Char,
        ExprKind::Float(_) => Ty::Float,
        ExprKind::Tuple(parts) => Ty::Tuple(
            parts
                .iter()
                .map(|p| type_of(item, p))
                .collect::<Option<_>>()?,
        ),
        ExprKind::List(elems) => Ty::list(type_of(item, elems.first()?)?),
        ExprKind::Comprehension { body, .. } => Ty::list(type_of(item, body)?),
        ExprKind::Field { name, .. } => hover_at(item, name.span)?.clone(),
        ExprKind::Generic { base, .. } => return type_of(item, base),
        ExprKind::Call { callee, .. } => call_type(item, callee)?,
        ExprKind::Index { base, .. } => type_of(item, base)?.index_value()?,
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
    // A domain named as a value walks its positions.
    let ty = match ty {
        Ty::TypeName(name) if is_domain(cursor.analysis, name) => &Ty::Domain(name.clone()),
        ty => ty,
    };
    let entries = std_methods(cursor, std);
    let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
    let fits = cursor.fit.map_or_else(Vec::new, |fit| fit(ty, &names));
    for (entry, fit) in entries.into_iter().zip(fits) {
        // A `fn f<T>(self: T)` a `use` names is everyone's member; unnamed, it would crowd every list.
        // It sorts after the members whose receiver is a type of their own or meets a bound.
        let sort = match fit {
            Some(false) => 1,
            Some(true) if cursor.analysis.imports().contains_key(&entry.name) => 2,
            _ => continue,
        };
        if seen.insert(entry.name.clone()) {
            out.push((entry, sort));
        }
    }
    out
}

/// The hover for a member the checker records no fact for, `s.runs` in
/// `s.runs()`: the declaration the receiver's members name, rendered as
/// completion and signature help show it.
pub fn member_hover(cursor: &Cursor, std: &mut StdLib) -> Option<cpl_facade::Hover> {
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
    let cost = cpl_next_syntax::doc::Facts::parse(&entry.doc)
        .cost
        .map(|c| cpl_diag::plain::text(&c))
        .filter(|c| cpl_next_syntax::doc::big_o(c).is_some());
    Some(cpl_facade::Hover {
        typed: None,
        signature: false,
        span: cpl_diag::Span {
            start: start as u32,
            end: end as u32,
        },
        text: match &cost {
            Some(cost) => format!("{}\ncost: {cost}", entry.detail),
            None => entry.detail,
        },
        doc: (!entry.doc.is_empty()).then_some(entry.doc),
        cost,
    })
}

/// The top-level `fn name(self: ..)` of every std module: a program that
/// writes `.name` gets the fn without a `use`, so all of them are members.
/// A library's such fns are members once a `use` names them.
fn std_methods(cursor: &Cursor, std: &mut StdLib) -> Vec<Entry> {
    let mut out = Vec::new();
    let modules = cursor.analysis.modules();
    for (local, import) in cursor.analysis.imports() {
        let Some(module) = modules.get(import.module) else {
            continue;
        };
        let outline = std.outline_of(&module.path, module.analysis.text.clone());
        if let Some(entry) = outline.find(&import.name) {
            if entry.kind == Kind::Fn && entry.public {
                let mut entry = entry.clone();
                entry.name = local.clone();
                entry.kind = Kind::Method;
                entry.instance = true;
                out.push(entry);
            }
        }
    }
    for (module, name) in cpl_facade::Std.methods() {
        let Some(outline) = std.module(cursor.uri, &module) else {
            continue;
        };
        if let Some(entry) = outline.find(name) {
            if entry.kind == Kind::Fn && entry.public {
                let mut entry = entry.clone();
                entry.kind = Kind::Method;
                entry.instance = true;
                out.push(entry);
            }
        }
    }
    out
}

/// The type the `@role(range_literal)` fn returns: what `a..b` builds, whose members
/// the checker of today gives as `RANGE_MEMBERS`.
fn range_type() -> Option<&'static str> {
    use cpl_syntax::ast::{StmtKind, TypeKind};
    static FOUND: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    let found = FOUND.get_or_init(|| {
        let std = cpl_facade::Std;
        let marked = std
            .outlines()
            .iter()
            .filter(|(_, o)| o.contains("@range_literal"));
        let texts = marked.filter_map(|(m, _)| std.text(m));
        texts
            .flat_map(|t| cpl_syntax::parse_file(t).items)
            .find_map(|(_, item)| {
                item.stmts.into_iter().find_map(|s| match s.kind {
                    StmtKind::Fn(f)
                        if s.decorators.iter().any(|d| d.name.text == "range_literal") =>
                    {
                        match f.ret?.kind {
                            TypeKind::Named { name, .. } => Some(name.text),
                            _ => None,
                        }
                    }
                    _ => None,
                })
            })
    });
    found.as_deref()
}

fn own_members_of_ty(cursor: &Cursor, std: &mut StdLib, ty: &Ty) -> Vec<(Entry, u8)> {
    let builtin = |members: &[(&str, &str)], ty: String| {
        members
            .iter()
            .map(|(name, doc)| (plain(name, Kind::Getter, ty.clone(), doc), 0))
            .collect()
    };
    match ty {
        Ty::Named { name, .. } if Some(name.as_str()) == range_type() => {
            builtin(cpl_facade::today::RANGE_MEMBERS, ty.to_string())
        }
        Ty::Named { name, args } if builtin_receiver(ty).is_none() => {
            type_members(cursor, std, name, args, args.len(), true)
        }
        Ty::TypeName(name) if is_domain(cursor.analysis, name) => {
            builtin(cpl_facade::today::DOMAIN_MEMBERS, format!("domain {name}"))
        }
        Ty::TypeName(name) => {
            let arity = written_arity(&cursor.analysis.text[..cursor.offset as usize]);
            // A module's second `Prefix` is keyed `Prefix'1` (R02.11.4).
            let key = cpl_facade::today::arity_key(name, arity);
            let keyed = if arity > 0 {
                type_members(cursor, std, &key, &[], arity, false)
            } else {
                Vec::new()
            };
            if keyed.is_empty() {
                type_members(cursor, std, name, &[], arity, false)
            } else {
                keyed
            }
        }
        Ty::Record(fields) => fields
            .iter()
            .map(|(name, ty)| (plain(name, Kind::Field, format!("{name}: {ty}"), ""), 0))
            .collect(),
        Ty::Tuple(items) => items
            .iter()
            .enumerate()
            .map(|(i, ty)| (plain(&i.to_string(), Kind::Field, ty.to_string(), ""), 0))
            .collect(),
        Ty::Named { .. } | Ty::Str | Ty::Array { .. } => {
            builtin_receiver(ty).map_or_else(Vec::new, |(receiver, bindings)| {
                builtin_members(receiver)
                    .map(|member| (builtin_entry(member, &bindings), 0))
                    .collect()
            })
        }
        // The checker's own members of an optional, beside std's `self: ?T` fns.
        Ty::Optional(inner) => cpl_facade::today::OPTIONAL_MEMBERS
            .iter()
            .map(|(name, sig, doc)| {
                let detail = format!("fn {}", sig.replace('T', &inner.to_string()));
                (plain(name, Kind::Method, detail, doc), 0)
            })
            .collect(),
        Ty::Domain(domain) => builtin(
            cpl_facade::today::DOMAIN_MEMBERS,
            format!("domain {domain}"),
        ),
        Ty::Pos(domain) => builtin(
            cpl_facade::today::POS_MEMBERS,
            format!("position in {domain}"),
        ),
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
    let mut out = type_members(cursor, std, name, &[], 0, true);
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
            let names: &[&str] = if matches!(receiver, Receiver::Map | Receiver::HashMap) {
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
    // `; after: ?T = none, while ok: ..`: from the `;` on, options passed
    // by name, each by its label (`while`), not its inner name (`ok`).
    let mut named_only = false;
    let params = cpl_facade::today::split_builtin_params(member.params)
        .into_iter()
        .map(|param| {
            if let Some(rest) = param.strip_prefix(';') {
                named_only = true;
                return (rest.trim_start(), named_only);
            }
            (param, named_only)
        })
        .map(|(param, named_only)| {
            let (name, ty) = param.split_once(':').unwrap_or((param, ""));
            let (ty, default) = ty.split_once(" = ").unwrap_or((ty, ""));
            let mut words = name.split_whitespace();
            Param {
                name: words.next().unwrap_or("").to_string(),
                binding: words.next().unwrap_or("").to_string(),
                ty: substitute(ty.trim(), bindings),
                default: default.trim().to_string(),
                named_only,
            }
        })
        .collect();
    Entry {
        name: member.name.to_string(),
        ty: substitute(member.ret, bindings),
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
        origin: None,
    }
}

/// How many type arguments the type before the `.` ending `before` is
/// written with: 1 for `Prefix<Int>.fil` and `Prefix<Int>.filling(`, 0 for `Prefix.fil`.
fn written_arity(before: &str) -> usize {
    // The cursor is after the `.`, in the member's name or in its call.
    let at = before.trim_end_matches(|c: char| c.is_alphanumeric() || c == '_' || c == '(');
    let Some(at) = at.strip_suffix('.').filter(|t| t.ends_with('>')) else {
        return 0;
    };
    let (mut depth, mut commas) = (0, 0);
    for c in at.chars().rev() {
        match c {
            '>' | ')' | ']' => depth += 1,
            '<' | '(' | '[' if depth == 1 && c == '<' => return commas + 1,
            '<' | '(' | '[' => depth -= 1,
            ',' if depth == 1 => commas += 1,
            _ => {}
        }
    }
    0
}

fn type_members(
    cursor: &Cursor,
    std: &mut StdLib,
    name: &str,
    args: &[Ty],
    arity: usize,
    instance: bool,
) -> Vec<(Entry, u8)> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for (index, outline) in outlines_for(cursor, std, name).into_iter().enumerate() {
        // The first outline is this file's; any other is another module's,
        // where only what it offers its users is shown.
        let foreign = index > 0;
        let offered = |m: &Entry| !foreign || outline.offers(name, m);
        // `Prefix<Int>` is the one of the types named `Prefix` taking one argument.
        let def = (outline.entries.iter())
            .filter(|e| e.name == name)
            .find(|e| generics_of(&e.detail).len() == arity)
            .or_else(|| outline.find(name));
        let Some(def) = def else {
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
        // `pub jump: [V]V` beside `pub fn jump(self, ..)`: the call is what's offered.
        let methods: HashSet<&str> = (outline.members_of_entry(Some(def), name))
            .filter(|m| m.instance && m.kind != Kind::Field)
            .map(|m| m.name.as_str())
            .collect();
        for member in outline.members_of_entry(Some(def), name) {
            if member.kind == Kind::Field && methods.contains(member.name.as_str()) {
                continue;
            }
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
    for (name, ty, keyword) in locals(cursor) {
        // A local the checker could not type is still in scope.
        let (kind, detail) = match keyword {
            "domain" => (Kind::Domain, format!("domain {name}")),
            "type" => (Kind::Alias, format!("type {name}")),
            "field" => (Kind::Field, format!("{name}: {ty}")),
            _ if ty.has_vague() => (Kind::Value, name.clone()),
            _ => (Kind::Value, format!("{name}: {ty}")),
        };
        add(&plain(&name, kind, detail, ""), 0, &mut items);
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
    let mut imports: Vec<_> = analysis.imports().iter().collect();
    cpl_diag::sort_by_str_key(&mut imports, |(name, _)| name);
    for (name, import) in imports {
        if let Some(outline) = module_outline(cursor, std, import.module) {
            if let Some(entry) = outline.find(&import.name) {
                let mut entry = entry.clone();
                entry.name = name.clone();
                add(&entry, 2, &mut items);
            }
        }
    }
    let mut aliases: Vec<&String> = analysis.aliases().keys().collect();
    cpl_diag::sort_by_str_key(&mut aliases, |alias| alias);
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
        for (name, module) in cpl_facade::Std.names() {
            if !name.starts_with(word)
                || seen.contains(*name)
                || cpl_facade::today::is_list_form_hook(name)
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
            let (span, new_text) = cpl_diag::import_edit(text, module, name);
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
/// Also its comprehensions, whose `for` names are seen in the body before them,
/// each fn's generics (seen in its signature too) and the struct methods, which
/// see the struct's fields without `self.`.
#[derive(Default)]
struct FnSpans {
    fns: Vec<cpl_diag::Span>,
    comprehensions: Vec<cpl_diag::Span>,
    generics: Vec<(cpl_diag::Span, String)>,
    methods: Vec<cpl_diag::Span>,
}

impl Visitor for FnSpans {
    fn stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Fn(f) => {
                self.fns.push(stmt.span);
                let names = f.generics.iter().map(|g| (stmt.span, g.text.clone()));
                self.generics.extend(names);
            }
            StmtKind::Struct(s) => self.methods.extend(s.methods.iter().map(|m| m.span)),
            _ => {}
        }
    }
    fn expr(&mut self, expr: &Expr) {
        if let ExprKind::Comprehension { .. } = expr.kind {
            self.comprehensions.push(expr.span);
        }
    }
}

/// The locals in scope at the cursor, nearest last, with their types.
fn locals(cursor: &Cursor) -> Vec<(String, Ty, &'static str)> {
    let Some(item) = item_at(cursor.analysis, cursor.offset) else {
        return Vec::new();
    };
    let rel = cursor.offset - item.offset;
    let mut fns = FnSpans::default();
    for stmt in &item.syntax.parsed.stmts {
        walk_stmt(stmt, &mut fns);
    }
    let within = |s: &cpl_diag::Span| s.start <= rel && rel <= s.end;
    let in_same_comprehension = |at: u32| {
        fns.comprehensions
            .iter()
            .any(|c| c.start <= at && at <= c.end && c.start <= rel && rel <= c.end)
    };
    let innermost = |at: u32| {
        fns.fns
            .iter()
            .filter(|s| s.start <= at && at <= s.end)
            .min_by_key(|s| s.end - s.start)
            .copied()
    };
    let here = innermost(rel);
    let mut out: Vec<(String, Ty, &'static str)> = Vec::new();
    for entry in &item.check.hovers {
        let declares = entry.decl == Some(entry.span)
            && matches!(
                entry.keyword,
                "let" | "var" | "param" | "for" | "fn" | "domain"
            );
        let ahead = entry.span.end >= rel
            && !(entry.keyword == "for" && in_same_comprehension(entry.span.start));
        if !declares || ahead {
            continue;
        }
        // A nested fn's name is inside its own span; its scope is the fn around it.
        let owner = match entry.keyword {
            "fn" => {
                let mut around: Vec<_> = fns
                    .fns
                    .iter()
                    .filter(|s| s.start <= entry.span.start && entry.span.start <= s.end)
                    .collect();
                cpl_diag::sort_by_key(&mut around, |s| (s.end - s.start) as u64);
                around.get(1).map(|outer| **outer)
            }
            _ => innermost(entry.span.start),
        };
        if owner.is_some_and(|o| here.is_none_or(|h| !(o.start <= h.start && h.end <= o.end))) {
            continue;
        }
        out.retain(|(name, _, _)| *name != entry.name);
        out.push((entry.name.clone(), entry.ty.clone(), entry.keyword));
    }
    if fns.methods.iter().any(within) {
        for entry in &item.check.hovers {
            if entry.keyword == "field" && entry.decl == Some(entry.span) {
                out.insert(0, (entry.name.clone(), entry.ty.clone(), "field"));
            }
        }
    }
    for (_, name) in fns.generics.iter().filter(|(span, _)| within(span)) {
        out.insert(0, (name.clone(), Ty::TypeName(name.clone()), "type"));
    }
    out.reverse();
    out
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
