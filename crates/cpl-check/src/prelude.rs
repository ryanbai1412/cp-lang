//! Names every file can use without a `use`: the prelude of
//! `plan/40-tooling.md` (`print`, `read`, `min`, `max`, `abs`, `List`,
//! `Map`, `Set`, `Int`, `Bool`, `Char`, `Str`, `INF`), the reductions
//! over sequences that `plan/12-domains-and-sequences.md` writes bare
//! (`sum`, `count`, `all`, `any`, `find`, `first`, `last`, `argmax`,
//! `argmin`), and the intrinsics the libraries are written on top of
//! (`rand`, `sort`, `swap`, `popcount`, ...).
//!
//! Everything else, `std` included, is cp source and resolves through
//! `use`. The `std` names a program has without writing one are listed in
//! `std/prelude.cp`, which [`names`] reads.

use std::sync::OnceLock;

/// The std modules a scan reads, by name with their text: the
/// `modules::std_index` outlines, or in a test the whole files.
pub(crate) type Std<'a> = &'a [(&'static str, &'static str)];

use cpl_diag::Span;
use cpl_syntax::ast;

use crate::ty::{Param, Ty};

/// What `std/prelude.cp` lists: each name, and the module its `use`
/// takes it from (`std.mod`, `Mod`). The compiler has no list of its own.
pub fn names() -> &'static [(Vec<ast::Name>, ast::Name)] {
    static NAMES: OnceLock<Vec<(Vec<ast::Name>, ast::Name)>> = OnceLock::new();
    NAMES.get_or_init(|| {
        // A `CPL_STD_DIR` without a prelude gives programs no prelude names.
        let Some(text) = crate::modules::bundled_std(&["prelude".to_string()]) else {
            return Vec::new();
        };
        let mut found = Vec::new();
        for (_, item) in cpl_syntax::parse_file(text).items {
            for stmt in item.stmts {
                if let ast::StmtKind::Use { path, names, .. } = stmt.kind {
                    // The spans are of the prelude's text, not the file's.
                    let bare = |name: &ast::Name| ast::Name {
                        text: name.text.clone(),
                        span: cpl_diag::Span::new(0, 0),
                    };
                    let module: Vec<ast::Name> = path.iter().map(bare).collect();
                    found.extend(names.iter().map(|name| (module.clone(), bare(name))));
                }
            }
        }
        found
    })
}

/// Words of the language over a walk (`argmax(f(v) for v in V)`) that,
/// given a whole list or array, are the library's method of that name:
/// `argmax(a)` is `a.argmax`. Writing the word brings the method in.
pub const SEQUENCE_FORMS: [&str; 2] = ["argmax", "argmin"];

/// The name a method of `module` is imported under beside another of
/// its name, one per module: `find of std.strings`. The space keeps it
/// out of reach of what a program can write.
pub fn method_key(name: &str, module: &str) -> String {
    format!("{name} of {module}")
}

/// The key of a struct that shares its name with another of a different
/// arity, after the file's first (R02.11.4): `Prefix'3` for `Prefix<R, C, T>`.
pub fn arity_key(name: &str, arity: usize) -> String {
    format!("{name}'{arity}")
}

/// The name a type is written with: `Prefix` for the key `Prefix'3`.
pub fn written_name(key: &str) -> &str {
    key.split('\'').next().unwrap_or(key)
}

/// The key of the `k`th `fn name(self: ..)` of one module, after the
/// first (R08.1.4): `bsearch of #2`.
pub fn overload_key(name: &str, k: usize) -> String {
    method_key(name, &format!("#{k}"))
}

/// The name and `#k` of an overload's key, `bsearch of #2` or the
/// imported `bsearch of std.range#2`.
pub fn overload_part(key: &str) -> Option<(&str, &str)> {
    let (name, module) = key.split_once(" of ")?;
    Some((name, &module[module.find('#')?..]))
}

/// What the method `name` of the std `module` is imported under: its
/// `method_key` beside the file's own `name` or another module's.
pub fn import_key(module: &[ast::Name], name: &str, own: bool) -> String {
    if own || shared_method(name) {
        let path: Vec<&str> = module.iter().map(|n| n.text.as_str()).collect();
        method_key(name, &path.join("."))
    } else {
        name.to_string()
    }
}

/// Whether two modules of the standard library give a method `name`:
/// each is then imported under its `method_key`, and a call picks the
/// one its receiver fits (R11.3.4).
pub fn shared_method(name: &str) -> bool {
    methods().iter().filter(|(_, n)| n.text == name).count() > 1
}

/// Every fn, method and getter name the active standard library declares where a program
/// can call it: `pub`, or in a trait or impl body. A struct's fields and a module's private
/// helpers are not members of the built-in types (R02.6.4). A member of a built-in type that
/// the checker can't type and no module declares is a spec gap.
pub fn spec_member_names() -> &'static crate::fx::HashSet<String> {
    static NAMES: OnceLock<crate::fx::HashSet<String>> = OnceLock::new();
    NAMES.get_or_init(|| member_names(crate::modules::std_index()))
}

fn member_names(std: Std) -> crate::fx::HashSet<String> {
    let ident = |s: &str| -> String {
        s.chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect()
    };
    let mut names = crate::fx::HashSet::default();
    for (_, text) in std {
        let mut shared_body = false;
        for line in text.lines() {
            let line = line.split('#').next().unwrap_or("");
            let code = line.trim_start();
            if code.is_empty() || code.starts_with("//") {
                continue;
            }
            let top = code.len() == line.len();
            if top {
                let item = code.strip_prefix("pub ").unwrap_or(code);
                shared_body = item.starts_with("trait ") || item.starts_with("impl");
            }
            if let Some(at) = code.find("fn ") {
                let public = code[..at].split_whitespace().any(|w| w == "pub");
                if public || (shared_body && !top) {
                    names.insert(ident(&code[at + 3..]));
                }
            }
        }
    }
    names
}

/// The member names std declares only inside `struct` bodies: a view's
/// `src`, a stack's `push`. A built-in type such as `Range` gets none of
/// them, though the name is on `spec_member_names`.
pub fn spec_struct_only_member_names() -> &'static crate::fx::HashSet<String> {
    static NAMES: OnceLock<crate::fx::HashSet<String>> = OnceLock::new();
    NAMES.get_or_init(|| struct_only_member_names(crate::modules::std_index()))
}

fn struct_only_member_names(std: Std) -> crate::fx::HashSet<String> {
    let ident = |s: &str| -> String {
        s.chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect()
    };
    let mut in_struct = crate::fx::HashSet::default();
    let mut elsewhere = crate::fx::HashSet::default();
    for (_, text) in std {
        let mut struct_body = false;
        for line in text.lines() {
            let code = line.split('#').next().unwrap_or("");
            let trimmed = code.trim_start();
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }
            let top = trimmed.len() == code.len();
            if top {
                let item = trimmed.strip_prefix("pub ").unwrap_or(trimmed);
                struct_body = item.starts_with("struct ");
            }
            if let Some(at) = trimmed.find("fn ") {
                let name = ident(&trimmed[at + 3..]);
                if struct_body && !top {
                    in_struct.insert(name);
                } else {
                    elsewhere.insert(name);
                }
            }
        }
    }
    in_struct.retain(|n| !elsewhere.contains(n));
    in_struct
}

/// The names of the top-level `fn name(self: S, ..)` methods: all a value
/// of no declared struct, such as a generator, can be given.
pub fn spec_receiver_method_names() -> &'static crate::fx::HashSet<String> {
    static NAMES: OnceLock<crate::fx::HashSet<String>> = OnceLock::new();
    NAMES.get_or_init(|| receiver_method_names(crate::modules::std_index()))
}

fn receiver_method_names(std: Std) -> crate::fx::HashSet<String> {
    std.iter()
        .flat_map(|(_, text)| scan_top_level_methods(text))
        .map(|(n, _)| n.to_string())
        .collect()
}

/// The `self` each top-level std `fn name(self: T, ..)` takes, by name:
/// `Some(head)` for a named type (`Range`, `List`, `[` for an array) and
/// `None` for one of the fn's own type parameters, which takes any receiver.
pub fn std_self_heads() -> &'static crate::names::NameMap<Vec<Option<String>>> {
    static HEADS: OnceLock<crate::names::NameMap<Vec<Option<String>>>> = OnceLock::new();
    HEADS.get_or_init(|| self_heads(crate::modules::std_index()))
}

fn self_heads(std: Std) -> crate::names::NameMap<Vec<Option<String>>> {
    let mut heads: crate::names::NameMap<Vec<Option<String>>> = Default::default();
    for (_, text) in std {
        for line in text.lines() {
            let line = line.strip_prefix("@get ").unwrap_or(line);
            let Some(rest) = line.strip_prefix("pub fn ") else {
                continue;
            };
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            let rest = &rest[name.len()..];
            let (generics, rest) = match rest.strip_prefix('<') {
                Some(g) => g.split_once(">(").unwrap_or(("", "")),
                None => ("", rest.strip_prefix('(').unwrap_or("")),
            };
            let rest = rest.strip_prefix("mut ").unwrap_or(rest);
            let Some(ty) = rest.strip_prefix("self: ") else {
                continue;
            };
            let own_param = |t: &str| {
                let t = t.trim();
                generics
                    .split(',')
                    .any(|g| g.split(':').next().unwrap_or("").trim() == t)
            };
            // `[I]E`: `[` takes any list; `[=` has an index or element
            // of a set type (`[P]Range<D>`, `[Pair<V>]T`), which a list
            // of anything may not be.
            let head: String = match ty.strip_prefix('[') {
                Some(arr) => {
                    let (index, rest) = arr.split_once(']').unwrap_or((arr, ""));
                    let elem: String = rest
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_')
                        .collect();
                    if own_param(index) && own_param(&elem) {
                        "[".into()
                    } else {
                        "[=".into()
                    }
                }
                None => ty
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect(),
            };
            let own = generics
                .split(',')
                .any(|g| g.split(':').next().unwrap_or("").trim() == head);
            heads.entry(name).or_default().push((!own).then_some(head));
        }
    }
    heads
}

/// Whether `name` is a top-level std method that does work and needs
/// nothing but `self`: every `pub fn name(self: S, ..)` gives the rest
/// defaults and is not `@get`, so reading one without `()` hides work
/// (R07.1.7). Only the modules that declare a fn `name` are parsed, each
/// once, so a check pays for the modules its file reads.
pub fn is_work_method(name: &str) -> bool {
    static MODULES: OnceLock<Vec<OnceLock<SelfMethods>>> = OnceLock::new();
    let std = crate::modules::std_index();
    let modules = MODULES.get_or_init(|| std.iter().map(|_| OnceLock::new()).collect());
    let mut work = false;
    for ((_, text), parsed) in std.iter().zip(modules) {
        if !text.contains(&format!("fn {name}")) || !text.contains("self:") {
            continue;
        }
        let parsed = parsed.get_or_init(|| self_methods_in(text));
        if parsed.other.contains(name) {
            return false;
        }
        work |= parsed.work.contains(name);
    }
    work
}

/// A module's `pub fn name(self, ..)` names: `work` when the fn is no
/// getter and gives every other parameter a default.
#[derive(Default)]
struct SelfMethods {
    work: crate::fx::HashSet<String>,
    other: crate::fx::HashSet<String>,
}

fn self_methods_in(text: &str) -> SelfMethods {
    let mut found = SelfMethods::default();
    for (_, item) in cpl_syntax::parse_file(text).items {
        for stmt in &item.stmts {
            let ast::StmtKind::Fn(f) = &stmt.kind else {
                continue;
            };
            if !stmt.public || !f.params.first().is_some_and(|p| p.name.text == "self") {
                continue;
            }
            let getter = stmt.decorators.iter().any(|d| d.name.text == "get");
            match !getter && f.params[1..].iter().all(|p| p.default.is_some()) {
                true => found.work.insert(f.name.text.clone()),
                false => found.other.insert(f.name.text.clone()),
            };
        }
    }
    found
}

/// The top-level std methods that are only ever `@get`: every
/// `pub fn name(self: S, ..)` of that name is a getter, so `g.rows()` is
/// E0304 as `xs.len()` is (R06.4.2).
pub fn getter_methods() -> &'static crate::fx::HashSet<String> {
    static NAMES: OnceLock<crate::fx::HashSet<String>> = OnceLock::new();
    NAMES.get_or_init(|| getter_methods_in(crate::modules::std_index()))
}

fn getter_methods_in(std: Std) -> crate::fx::HashSet<String> {
    let mut get = crate::fx::HashSet::default();
    let mut other = crate::fx::HashSet::default();
    for (_, text) in std {
        if !text.contains("self:") {
            continue;
        }
        for (_, item) in cpl_syntax::parse_file(text).items {
            for stmt in &item.stmts {
                let ast::StmtKind::Fn(f) = &stmt.kind else {
                    continue;
                };
                if !stmt.public || !f.params.first().is_some_and(|p| p.name.text == "self") {
                    continue;
                }
                match stmt.decorators.iter().any(|d| d.name.text == "get") {
                    true => get.insert(f.name.text.clone()),
                    false => other.insert(f.name.text.clone()),
                };
            }
        }
    }
    get.retain(|n| !other.contains(n));
    get
}

/// The methods the standard library adds to the types of the language:
/// each `pub fn name(self: T, ..)` at the top of a module, with that
/// module. A program that writes `.name` gets the fn without a `use`,
/// as it gets the methods a type is declared with. A method name does
/// not stand in the way of a program's own names, so these are not on
/// the prelude's list. A name two modules give is there once for each.
pub fn methods() -> &'static [(Vec<ast::Name>, ast::Name)] {
    static METHODS: OnceLock<Vec<(Vec<ast::Name>, ast::Name)>> = OnceLock::new();
    METHODS.get_or_init(|| methods_in(crate::modules::std_index()))
}

fn methods_in(std: Std) -> Vec<(Vec<ast::Name>, ast::Name)> {
    let bare = |text: &str| ast::Name {
        text: text.to_string(),
        span: cpl_diag::Span::new(0, 0),
    };
    let mut found: Vec<(Vec<ast::Name>, ast::Name, bool)> = Vec::new();
    for (module, text) in std {
        if *module == "prelude" || !text.contains("self:") {
            continue;
        }
        let path: Vec<ast::Name> = std::iter::once("std")
            .chain(module.split('/'))
            .map(bare)
            .collect();
        for (name, is_todo) in scan_top_level_methods(text) {
            found.push((path.clone(), bare(name), is_todo));
        }
    }
    // A `todo` body gives way to a module that has the method.
    let done = |name: &str| found.iter().any(|(_, n, t)| n.text == name && !t);
    found
        .iter()
        .filter(|(_, name, is_todo)| !is_todo || !done(&name.text))
        .map(|(path, name, _)| (path.clone(), name.clone()))
        .collect()
}

/// Scans top-level `[@dec ...] pub [@dec ...] fn <name>[<...>]([mut] self: ...)`
/// headers without parsing function bodies. Returns `(name, is_todo)`.
fn scan_top_level_methods(text: &str) -> Vec<(&str, bool)> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut offset = 0;
    while offset < bytes.len() {
        let rest = &text[offset..];
        // Top-level declarations start at column 0 with `pub` or `@decorator`.
        if rest.starts_with("pub ") || rest.starts_with('@') {
            let mut cur = rest;
            let mut is_pub = false;
            loop {
                cur = cur.trim_start_matches([' ', '\t']);
                if let Some(after_pub) = cur.strip_prefix("pub ") {
                    is_pub = true;
                    cur = after_pub;
                    continue;
                }
                if let Some(dec) = cur.strip_prefix('@') {
                    let dec_len = dec
                        .bytes()
                        .take_while(|b| b.is_ascii_alphanumeric() || *b == b'_')
                        .count();
                    cur = &dec[dec_len..];
                    if cur.starts_with('(') {
                        if let Some(close) = cur.find(')') {
                            cur = &cur[close + 1..];
                        }
                    }
                    continue;
                }
                break;
            }
            if is_pub {
                if let Some(after_pub_fn) = cur.strip_prefix("fn ") {
                    let name_len = after_pub_fn
                        .bytes()
                        .take_while(|b| b.is_ascii_alphanumeric() || *b == b'_')
                        .count();
                    if name_len > 0 {
                        let name = &after_pub_fn[..name_len];
                        let mut tail = after_pub_fn[name_len..].trim_start();
                        if tail.starts_with('<') {
                            let mut depth = 0usize;
                            let mut end = 0;
                            for (idx, b) in tail.bytes().enumerate() {
                                match b {
                                    b'<' => depth += 1,
                                    b'>' => {
                                        depth -= 1;
                                        if depth == 0 {
                                            end = idx + 1;
                                            break;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            if end > 0 {
                                tail = tail[end..].trim_start();
                            }
                        }
                        if let Some(mut params) = tail.strip_prefix('(') {
                            params = params.trim_start();
                            if let Some(after_mut) = params.strip_prefix("mut") {
                                if after_mut.starts_with(|c: char| c.is_whitespace()) {
                                    params = after_mut.trim_start();
                                }
                            }
                            if let Some(after_self) = params.strip_prefix("self") {
                                if after_self.trim_start().starts_with(':') {
                                    let line_end = rest.find('\n').unwrap_or(rest.len());
                                    let line = rest[..line_end].trim_end();
                                    let is_todo = line.ends_with("= todo");
                                    out.push((name, is_todo));
                                }
                            }
                        }
                    }
                }
            }
        }
        match bytes[offset..].iter().position(|&b| b == b'\n') {
            Some(nl) => offset += nl + 1,
            None => break,
        }
    }
    out
}

/// The words of the language that are written with a library fn: a
/// program that uses the word gets the fn, as if it had named it.
/// `prefix(a)` dispatches by what `a` is (`[D]T`, `[R, C]T`, or `List<T>`).
pub const BUILT_ON: &[(&str, &str)] = &[("subsets", "Subset")];

/// `BUILT_ON` and the words std states on a type, `@word(prefix)` above
/// `pub struct Prefix`: `prefix(a)` is that type's constructor, `Prefix(a)`.
pub fn built_on() -> &'static [(&'static str, &'static str)] {
    static ALL: OnceLock<Vec<(&'static str, &'static str)>> = OnceLock::new();
    ALL.get_or_init(|| {
        let mut all = BUILT_ON.to_vec();
        let leak = |s: &str| &*Box::leak(s.to_string().into_boxed_str());
        for (_, text) in crate::modules::std_index() {
            if !text.contains("@word(") {
                continue;
            }
            for (_, item) in cpl_syntax::parse_file(text).items {
                for stmt in &item.stmts {
                    let ast::StmtKind::Struct(d) = &stmt.kind else {
                        continue;
                    };
                    let words = stmt.decorators.iter().filter(|d| d.name.text == "word");
                    for arg in words.flat_map(|d| &d.args) {
                        if let ast::ExprKind::Name(word) = &arg.value.kind {
                            all.push((leak(word), leak(&d.name.text)));
                        }
                    }
                }
            }
        }
        all
    })
}

/// The type whose constructor the word `name` is, if std built one on it.
pub fn word_type(name: &str) -> Option<&'static str> {
    built_on()
        .iter()
        .find(|(word, _)| *word == name)
        .map(|(_, on)| *on)
}

/// The name a std item has now for one it was written with before, as
/// `@was(Prefix1, PrefixN)` above `pub struct Prefix` states. A word built
/// on that type was written in snake case: `prefix_n` for `PrefixN`.
pub fn renamed(name: &str) -> Option<&'static str> {
    static WAS: OnceLock<crate::names::NameMap<&'static str>> = OnceLock::new();
    let was = WAS.get_or_init(|| {
        let mut was = crate::names::NameMap::default();
        let leak = |s: &str| &*Box::leak(s.to_string().into_boxed_str());
        for (_, text) in crate::modules::std_index() {
            if !text.contains("@was(") {
                continue;
            }
            for (_, item) in cpl_syntax::parse_file(text).items {
                for stmt in &item.stmts {
                    let now = match &stmt.kind {
                        ast::StmtKind::Struct(d) => &d.name.text,
                        ast::StmtKind::Enum(d) => &d.name.text,
                        ast::StmtKind::Fn(f) => &f.name.text,
                        _ => continue,
                    };
                    let olds = stmt.decorators.iter().filter(|d| d.name.text == "was");
                    for arg in olds.flat_map(|d| &d.args) {
                        if let ast::ExprKind::Name(old) = &arg.value.kind {
                            was.insert(old.clone(), leak(now));
                        }
                    }
                }
            }
        }
        let snake = |s: &str| {
            let mut out = String::new();
            for (i, c) in s.chars().enumerate() {
                if c.is_uppercase() && i > 0 {
                    out.push('_');
                }
                out.extend(c.to_lowercase());
            }
            out
        };
        let types: Vec<(String, &str)> = was.iter().map(|(o, n)| (o.clone(), *n)).collect();
        for &(word, on) in built_on() {
            for (old, _) in types.iter().filter(|(_, now)| *now == on) {
                was.insert(snake(old), word);
            }
        }
        was
    });
    was.get(name).copied()
}

/// Internal shape specializations brought in from a stdlib module when a
/// file uses a word, without spending prelude slots or exposing internal
/// names to users.
pub const INTERNAL_BUILT_ON: &[(&str, &str, &str)] = &[("gen", "strings", "_gen_str")];

/// The types a prelude type's public signatures name from its own std
/// module, followed through theirs: `Prefix.filling(order: FillOrder) ->
/// FillPrefix` gives `FillPrefix` and `FillOrder`. A program that gets the
/// type gets these with it, so what its methods take and return can be named.
pub fn companions(name: &str) -> &'static [&'static str] {
    static ALL: OnceLock<crate::names::NameMap<Vec<&'static str>>> = OnceLock::new();
    let all = ALL.get_or_init(|| {
        let mut all = crate::names::NameMap::default();
        for (module, ty) in names() {
            if !ty.text.starts_with(char::is_uppercase) || module.len() < 2 {
                continue;
            }
            let module: Vec<&str> = module[1..].iter().map(|n| n.text.as_str()).collect();
            let Some(&(_, text)) = crate::modules::std_index()
                .iter()
                .find(|(m, _)| *m == module.join("/"))
            else {
                continue;
            };
            let found = signature_types(text, &ty.text);
            all.insert(ty.text.clone(), found);
        }
        all
    });
    all.get(name).map_or(&[], |v| v.as_slice())
}

fn signature_types(text: &'static str, ty: &str) -> Vec<&'static str> {
    let word = |s: &'static str| s.split(|c: char| !c.is_alphanumeric() && c != '_').next();
    let types: Vec<&str> = text
        .lines()
        .filter_map(|l| {
            ["pub struct ", "pub enum "]
                .iter()
                .find_map(|k| l.strip_prefix(k))
        })
        .filter_map(word)
        .collect();
    let mut found: Vec<&'static str> = vec![];
    let mut todo = vec![ty.to_string()];
    while let Some(of) = todo.pop() {
        let mut inside = false;
        for line in text.lines() {
            if !line.starts_with(char::is_whitespace) && !line.is_empty() {
                inside = line.strip_prefix("pub struct ").and_then(word) == Some(of.as_str());
            } else if inside && line.trim_start().starts_with("pub fn ") {
                for id in line.split(|c: char| !c.is_alphanumeric() && c != '_') {
                    if let Some(&t) = types.iter().find(|&&t| t == id) {
                        if t != ty && !found.contains(&t) {
                            found.push(t);
                            todo.push(t.to_string());
                        }
                    }
                }
            }
        }
    }
    found
}

/// The std module and struct of each shape specialization a family name
/// stands for, as `Prefix<T>` for `_PrefixList<T>`.
pub fn std_family(name: &str) -> impl Iterator<Item = (&'static str, &'static str)> + '_ {
    INTERNAL_BUILT_ON
        .iter()
        .filter(move |(word, ..)| *word == name)
        .map(|&(_, module, sym)| (module, sym))
}

/// `sqrt`, `atan2`: an F64 from any number, an `Int` variable included.
fn float_fn(arity: usize) -> Ty {
    fn_of(arity, Ty::Unknown, Ty::Float)
}

/// A fn taking `arity` arguments of type `param`.
fn fn_of(arity: usize, param: Ty, ret: Ty) -> Ty {
    Ty::Fn {
        params: (0..arity)
            .map(|_| Param {
                name: String::new(),
                ty: param.clone(),
                mutable: false,
                has_default: false,
                named_only: false,
                variadic: false,
                lazy: false,
            })
            .collect(),
        ret: Box::new(ret),
    }
}

/// The type of a prelude value, or `None` if `name` is not one. Fns that
/// are generic over what they are given are `Unknown`, which accepts any
/// call. `VALUE_NAMES` is the one list of them.
pub fn value(name: &str) -> Option<Ty> {
    Some(match name {
        "INF" => Ty::Int,
        "cbrt" | "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "exp" | "ln" | "log2"
        | "log10" => float_fn(1),
        "atan2" | "hypot" => float_fn(2),
        // `ask("?", x)` prints a query, flushes, and reads the judge's
        // answer; `Subset<D>` is the subsets of a domain. Both are lowered
        // by `cpl-ir`.
        "ask" | "Subset" => Ty::Unknown,
        _ if TYPE_NAMES.contains(&name) => Ty::TypeName(name.into()),
        _ if VALUE_NAMES.contains(&name) => Ty::Unknown,
        _ => return None,
    })
}

/// The doc of a prelude value the compiler defines, for a call with `args`
/// arguments or, with `None`, the value itself: `min(xs)` walks `xs` where
/// `min(a, b)` compares two values. `fixed` says the checker found the
/// values a `print` or `read` handles are of a fixed size.
pub fn value_doc(name: &str, args: Option<usize>, fixed: bool) -> Option<&'static str> {
    Some(match (name, args) {
        ("print", _) if fixed => "Writes its arguments and a newline, in O(1) for numbers, characters and tuples of them.\n\ncost: O(1)",
        ("print", _) => "Writes its arguments and a newline, lists space-separated and grids by row, in time linear in what it writes.",
        ("read", _) if fixed => "Reads the next value of the type it is bound to, in O(1) for a number, a character or a tuple of them.\n\ncost: O(1)",
        ("read", _) => "Reads the next value of the type it is bound to, in time linear in what it reads.",
        ("ask", _) if fixed => "Writes a query of its arguments, flushes and reads the judge's reply, in O(1) for numbers, characters and tuples of them, not counting the judge's own time.\n\ncost: O(1)",
        ("ask", _) => "Writes a query of its arguments, flushes and reads the judge's reply, in time linear in what it writes and reads, not counting the judge's own time.",
        ("Int" | "I128" | "U64" | "F64" | "Char" | "Bool", Some(1)) if fixed => "Converts a number, character or `Bool` to this type, in O(1).\n\ncost: O(1)",
        ("gen", _) => "A random value of its type, or of its range, within the type's contract, in time linear in the value's size.",
        ("rand", Some(0 | 2)) => "A random `Int`, from `lo` to `hi` when given, in O(1).\n\ncost: O(1)",
        ("min", Some(1)) => "The smallest element of a list, an array, a slice `xs[a..b]` or a generator, in O(n) for n elements.\n\ncost: O(n)",
        ("max", Some(1)) => "The largest element of a list, an array, a slice `xs[a..b]` or a generator, in O(n) for n elements.\n\ncost: O(n)",
        ("min", _) => "The smallest of its arguments, in O(1) per argument; `min(xs)` gives the smallest element of `xs`.\n\ncost: O(1)",
        ("max", _) => "The largest of its arguments, in O(1) per argument; `max(xs)` gives the largest element of `xs`.\n\ncost: O(1)",
        ("swap", _) => "Exchanges the values of two `mut` places, in O(1) whatever their size.\n\ncost: O(1)",
        ("some", _) => "The value as a present `?T`, in O(1).\n\ncost: O(1)",
        _ => return None,
    })
}

/// A prelude type by name.
pub fn ty(name: &str) -> Option<Ty> {
    Some(match name {
        "Int" => Ty::Int,
        "F64" => Ty::Float,
        "Bool" => Ty::Bool,
        "Char" => Ty::Char,
        "Str" => Ty::Str,
        "I128" => Ty::I128,
        "U64" => Ty::U64,
        "List" | "Map" | "HashMap" | "Set" | "HashSet" | "Iter" | "Seq" | "Indexed" | "Opt"
        | "Range" | "Cut" | "Sym" => Ty::Named {
            name: name.into(),
            args: Vec::new(),
        },
        // The solution as an `@interactor` sees it: `sol.recv()` reads what
        // it printed, `sol.send(x)` answers it.
        "Interactor" => Ty::Unknown,
        // The subsets of a domain, lowered by `cpl-ir` as a bitmask.
        "Subset" => Ty::Unknown,
        _ => return None,
    })
}

/// A prelude trait: what `T: Ord` and `impl Add for V` may name.
pub fn is_trait(name: &str) -> bool {
    TRAIT_NAMES.contains(&name)
}

pub const TYPE_NAMES: &[&str] = &[
    "List",
    "Map",
    "Set",
    "Int",
    "I128",
    "U64",
    "Bool",
    "Char",
    "Str",
    "F64",
    "Iter",
    "Opt",
    "Range",
    "Cut",
    "HashMap",
    "HashSet",
    "Interactor",
    "Subset",
    "Sym",
    "Seq",
    "Indexed",
];

/// The core traits std is written on (`spec/core/`).
pub const TRAIT_NAMES: &[&str] = &[
    "Index",
    "Seq",
    "Indexed",
    "IndexedMut",
    "Keyed",
    "Sized",
    "Bidir",
    "Contains",
    "Ord",
    "Eq",
    "Show",
    "Into",
    "Read",
    "Gen",
    "Shrink",
    "Add",
    "Sub",
    "Mul",
    "Div",
    "Neg",
    "Rem",
    "Pow",
    "Zero",
    "One",
    "Bounded",
    "Num",
    "Copy",
    "Bits",
    "BitAnd",
    "BitOr",
    "BitXor",
    "Domain",
];

pub const VALUE_NAMES: &[&str] = &[
    "print",
    "read",
    "read_line",
    "debug",
    "read_output",
    "gen",
    "min",
    "max",
    "sum",
    "count",
    "all",
    "any",
    "find",
    "first",
    "last",
    "argmax",
    "argmin",
    "sort",
    "sorted",
    "swap",
    "copy",
    "rand",
    "some",
    "laws_hold",
    "atan2",
    "sin",
    "cos",
    "tan",
    "asin",
    "acos",
    "exp",
    "ln",
    "hypot",
    "flush",
    "len",
    "counts",
    "prefix",
    "take",
    "columns",
    "List",
    "Map",
    "Set",
    "HashSet",
    "Int",
    "I128",
    "U64",
    "Bool",
    "Char",
    "Str",
    "F64",
    "Iter",
    "Opt",
    "Range",
    "Cut",
];

/// The standard library module that declares the public name `name`, for
/// the hint on a name that was used without its `use`.
/// The module whose top-level `pub fn name(self: ..)` takes a receiver
/// that is (`on_str`) or isn't a `Str`, for `.name` written without its
/// `use`: `split` on a list is `std.seq`'s, on a string `std.strings`'.
pub fn method_module(name: &str, on_str: bool) -> Option<String> {
    method_module_in(crate::modules::std_index(), name, on_str)
}

fn method_module_in(std: Std, name: &str, on_str: bool) -> Option<String> {
    let mut generic = None;
    for (module, text) in std {
        for line in text.lines() {
            let Some(rest) = line
                .strip_prefix("pub fn ")
                .and_then(|r| r.strip_prefix(name))
            else {
                continue;
            };
            let Some((_, of)) = rest
                .split_once("(self: ")
                .or_else(|| rest.split_once("(mut self: "))
            else {
                continue;
            };
            if !rest.starts_with(['<', '(']) {
                continue;
            }
            let module = module.replace('/', ".");
            match (
                of.starts_with("Str") && !of[3..].starts_with(char::is_alphanumeric),
                on_str,
            ) {
                (true, true) => return Some(module),
                (true, false) => {}
                (false, _) => {
                    generic.get_or_insert(module);
                }
            }
        }
    }
    generic
}

/// How many type arguments a built-in or std type takes, as in `Map<K, V>`.
pub fn type_arity(name: &str) -> Option<usize> {
    type_arity_in(crate::modules::std_index(), name)
}

fn type_arity_in(std: Std, name: &str) -> Option<usize> {
    match name {
        "Map" | "HashMap" => return Some(2),
        "List" | "Set" | "HashSet" => return Some(1),
        _ if TYPE_NAMES.contains(&name) => return Some(0),
        _ => {}
    }
    std.iter().find_map(|(_, text)| {
        text.lines().find_map(|line| {
            let rest = ["pub struct ", "pub enum ", "pub trait "]
                .iter()
                .find_map(|k| line.strip_prefix(k))?
                .strip_prefix(name)?;
            match rest.strip_prefix('<') {
                Some(params) => Some(params.split('>').next()?.split(',').count()),
                None => (!rest.starts_with(|c: char| c.is_alphanumeric() || c == '_')).then_some(0),
            }
        })
    })
}

pub fn std_module(name: &str) -> Option<&'static str> {
    std_names()
        .binary_search_by(|(known, _)| (*known).cmp(name))
        .ok()
        .map(|at| std_names()[at].1)
}

/// The edit that brings `name` in from `std.<module>`: into the braces of
/// the file's `use std.<module>.{..}` when it has one, else as a line after
/// its last top-level `use`, else above its first line of code.
pub fn import_edit(text: &str, module: &str, name: &str) -> (Span, String) {
    let head = format!("use std.{module}.{{");
    let mut after_uses = None;
    let mut first_code = None;
    let mut at = 0;
    for line in text.split_inclusive('\n') {
        let end = at + line.len();
        let body = line.trim_end();
        if let Some(close) = body.strip_prefix(&head).and_then(|_| body.rfind('}')) {
            let inside = body[head.len()..close].trim();
            let edit = if inside.is_empty() {
                name.to_string()
            } else {
                format!(", {name}")
            };
            let close = (at + close) as u32;
            return (Span::point(close), edit);
        }
        if body.starts_with("use ") {
            after_uses = Some(end);
        } else if first_code.is_none()
            && !body.trim_start().is_empty()
            && !body.trim_start().starts_with("//")
        {
            first_code = Some(at);
        }
        at = end;
    }
    let line = format!("use std.{module}.{{{name}}}\n");
    match (after_uses, first_code) {
        (Some(end), _) if !text[..end].ends_with('\n') => {
            (Span::point(end as u32), format!("\n{line}"))
        }
        (Some(end), _) => (Span::point(end as u32), line),
        (None, Some(start)) => (Span::point(start as u32), line),
        (None, None) => (Span::point(text.len() as u32), line),
    }
}

/// Every public name of the standard library and its module, sorted by
/// name, read off the outlines (`modules::std_index`), which keep every
/// top-level line: nothing embeds a table of them.
pub fn std_names() -> &'static [(&'static str, &'static str)] {
    static NAMES: std::sync::OnceLock<Vec<(&'static str, &'static str)>> =
        std::sync::OnceLock::new();
    NAMES.get_or_init(|| {
        let leak = |s: String| &*Box::leak(s.into_boxed_str());
        public_names(crate::modules::std_index().iter().copied())
            .into_iter()
            .map(|(name, module)| (leak(name), leak(module)))
            .collect()
    })
}

/// Every public name that exactly one module declares, with that module,
/// sorted by name: what the hint for a missing `use` reads. It is read off
/// the files, so a new `pub fn` needs no second edit.
pub(crate) fn public_names<'a>(
    modules: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Vec<(String, String)> {
    let mut found: std::collections::BTreeMap<String, Vec<&str>> = Default::default();
    for (module, text) in modules {
        for line in text.lines() {
            let mut words = line.split_whitespace();
            let public = !line.starts_with(' ') && words.next() == Some("pub");
            let declares = matches!(
                words.next(),
                Some("fn" | "struct" | "enum" | "trait" | "type" | "const" | "domain")
            );
            if !public || !declares {
                continue;
            }
            let name: String = words
                .next()
                .unwrap_or_default()
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.starts_with('_') {
                found.entry(name).or_default().push(module);
            }
        }
    }
    found
        .into_iter()
        .filter_map(|(name, modules)| match modules.as_slice() {
            [module] => Some((name, module.to_string())),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A module's private helper is no member a program can name, so it
    /// never hides a typo on a built-in type (`v.nxt` beside a std `fn nxt`).
    #[test]
    fn member_names_skip_private_helpers() {
        let text = "pub struct S:\n    pub fn get(self) -> Int = 0\n    fn nxt(self) -> Int = 1\nfn helper() -> Int = 2\npub trait T:\n    fn need(self) -> Int\n";
        let names = member_names(&[("m", text)]);
        assert!(names.contains("get") && names.contains("need"), "{names:?}");
        assert!(
            !names.contains("nxt") && !names.contains("helper"),
            "{names:?}"
        );
    }

    /// Every scan of std's declarations reads the outlines a browser build
    /// carries; on every std name it answers as it does from the files.
    #[test]
    fn each_std_scan_reads_the_same_off_the_outlines_as_off_the_files() {
        let (files, index) = (crate::modules::std_modules(), crate::modules::std_index());
        let names: Vec<&str> = files.iter().map(|(name, _)| *name).collect();
        assert_eq!(
            names,
            index.iter().map(|(name, _)| *name).collect::<Vec<_>>()
        );
        assert!(
            index.iter().map(|(_, t)| t.len()).sum::<usize>() * 4
                < files.iter().map(|(_, t)| t.len()).sum()
        );
        assert_eq!(member_names(files), member_names(index));
        assert_eq!(
            struct_only_member_names(files),
            struct_only_member_names(index)
        );
        assert_eq!(receiver_method_names(files), receiver_method_names(index));
        assert_eq!(self_heads(files), self_heads(index));
        let work = |std: Std| -> Vec<(Vec<String>, Vec<String>)> {
            let sorted = |set: &crate::fx::HashSet<String>| {
                let mut v: Vec<String> = set.iter().cloned().collect();
                v.sort();
                v
            };
            std.iter()
                .map(|(_, text)| self_methods_in(text))
                .map(|m| (sorted(&m.work), sorted(&m.other)))
                .collect()
        };
        assert_eq!(work(files), work(index));
        assert_eq!(methods_in(files), methods_in(index));
        assert!(is_work_method("sum") && !is_work_method("rev"));
        let types = files
            .iter()
            .flat_map(|(_, t)| t.lines())
            .filter_map(|line| {
                let rest = ["pub struct ", "pub enum ", "pub trait "]
                    .iter()
                    .find_map(|k| line.strip_prefix(k))?;
                rest.split(|c: char| !c.is_alphanumeric() && c != '_')
                    .next()
            });
        let names = member_names(files);
        for word in names
            .iter()
            .map(String::as_str)
            .chain(types)
            .chain(["List", "no_such"])
        {
            for on_str in [false, true] {
                assert_eq!(
                    method_module_in(files, word, on_str),
                    method_module_in(index, word, on_str),
                    "{word}"
                );
            }
            assert_eq!(
                type_arity_in(files, word),
                type_arity_in(index, word),
                "{word}"
            );
        }
    }

    /// The outlines `build.rs` writes are the ones read off the std text it
    /// put in. A dev build reads std/ from disk and is not rerun by a std
    /// edit, so the disk may be newer than the built outlines.
    #[test]
    fn the_built_outlines_are_the_outlines_of_std() {
        let read: Vec<(&str, String)> = crate::modules::BUNDLED
            .iter()
            .map(|(name, text)| (*name, crate::std_files::outline(text)))
            .collect();
        let built: Vec<(&str, String)> = crate::modules::BUILT_INDEX
            .iter()
            .map(|(n, t)| (*n, t.to_string()))
            .collect();
        assert_eq!(read, built);
    }

    /// A module `build.rs` borrows by `#[path]` reruns it when edited, or the outlines go stale.
    #[test]
    fn the_build_script_reruns_on_every_module_it_borrows() {
        let build = include_str!("../build.rs");
        for line in build.lines() {
            let Some(path) = line.trim().strip_prefix("#[path = \"") else {
                continue;
            };
            let path = path.trim_end_matches("\"]");
            let rerun = format!("cargo:rerun-if-changed={path}");
            assert!(
                build.contains(&rerun),
                "build.rs borrows {path} without `{rerun}`"
            );
        }
    }

    #[test]
    fn fast_method_scan_matches_full_parse_on_bundled_stdlib() {
        for (module, text) in crate::modules::std_modules() {
            if *module == "prelude" || !text.contains("self:") {
                continue;
            }
            let mut parsed = Vec::new();
            for (_, item) in cpl_syntax::parse_file(text).items {
                for stmt in item.stmts {
                    if let ast::StmtKind::Fn(decl) = &stmt.kind {
                        let on_a_type = decl
                            .params
                            .first()
                            .is_some_and(|p| p.name.text == "self" && p.ty.is_some());
                        if stmt.public && on_a_type {
                            parsed.push(decl.name.text.clone());
                        }
                    }
                }
            }
            let scanned: Vec<String> = scan_top_level_methods(text)
                .into_iter()
                .map(|(n, _)| n.to_string())
                .collect();
            assert_eq!(scanned, parsed, "method scan mismatch in std.{module}");
        }
    }
}

/// A library type that is a domain over another: `Pair<V>`, each two
/// positions of `V`, or `Subset<B>`, each set of them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetDomain {
    Pairs,
    Subsets,
}

/// The set domain a library type name stands for.
pub fn set_domain(name: &str) -> Option<SetDomain> {
    match name {
        "Pair" => Some(SetDomain::Pairs),
        "Subset" => Some(SetDomain::Subsets),
        _ => None,
    }
}
