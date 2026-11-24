//! Counters over generated C++ that the copy and `std::function` ratchets
//! (`crates/cpl-cli/tests/read_only_copies.rs`) and the explain accuracy
//! test read: what the C++ itself says is copied or called indirectly,
//! independent of what codegen reports. They read text, never the IR, so a
//! codegen bug can't hide from them; `cpp_counts_tests.rs` pins each shape.

/// `cpp` as codegen wrote it before it was fitted to its size bar
/// (`compact.rs`): each line spaced and indented the way codegen writes
/// it, and the library names a fitted file shortens written in full, so a
/// count reads the same of a fitted file as of its `--full` build.
pub fn unfitted(cpp: &str) -> std::borrow::Cow<'_, str> {
    if !cpp.lines().any(|l| l == "using ll=long long;") {
        return cpp.into();
    }
    let mut long: Longs = Longs::new();
    for line in cpp.lines() {
        if let Some((short, name)) = line
            .strip_prefix("template<class...T>using ")
            .and_then(|r| r.strip_suffix("<T...>;"))
            .and_then(|r| r.split_once('='))
        {
            long.insert(short, name.to_string());
        } else if let Some(name) = line.strip_prefix("using std::").and_then(|r| r.strip_suffix(';')) {
            long.insert(name, format!("std::{name}"));
        } else if let Some((short, name)) = line
            .strip_prefix("using ")
            .and_then(|r| r.strip_suffix(';'))
            .and_then(|r| r.split_once('='))
            .filter(|(short, _)| *short != "ll")
        {
            long.insert(short, name.to_string());
        }
    }
    let lines: Vec<(&str, Option<Vec<String>>)> = cpp
        .lines()
        .filter(|line| {
            !(line.starts_with("template<class...T>using ")
                || line.starts_with("using std::")
                || *line == "namespace cpl{}using namespace cpl;"
                || line.strip_prefix("using ").is_some_and(|r| r.contains('=') && !r.starts_with("ll=")))
        })
        .map(|line| (line, words(line, &long)))
        .collect();
    let runtime = runtime_longs(lines.iter().filter_map(|(_, w)| w.as_deref()));
    let mut out = String::with_capacity(cpp.len() * 5 / 4);
    // What each open brace opened: a block, a namespace, whose body codegen
    // does not indent, or the runtime's namespace.
    let mut open: Vec<u8> = Vec::new();
    for (line, words) in lines {
        let Some(words) = words else {
            out += line;
            out.push('\n');
            continue;
        };
        let in_cpl = open.contains(&2);
        let mut full: Vec<String> = Vec::with_capacity(words.len());
        for w in words {
            let scoped = full.last().is_some_and(|l| matches!(l.as_str(), "::" | "." | "->"));
            match runtime.get(w.as_str()) {
                Some(name) if !in_cpl && !scoped => {
                    full.extend(["cpl".into(), "::".into(), name.to_string()])
                }
                Some(name) => full.push(name.to_string()),
                None => full.push(w),
            }
        }
        let closes = full.iter().take_while(|w| w.as_str() == "}").count().min(1);
        let depth = open[..open.len().saturating_sub(closes)].iter().filter(|n| **n == 0).count();
        out += &"  ".repeat(depth);
        out += &spaced(&full);
        out.push('\n');
        let namespace = match (full.first(), full.get(1)) {
            (Some(n), Some(c)) if n == "namespace" => 1 + u8::from(c == "cpl"),
            _ => 0,
        };
        for w in &full {
            match w.as_str() {
                "{" => open.push(namespace),
                "}" => {
                    open.pop();
                }
                _ => {}
            }
        }
    }
    out.into()
}

/// The library name each short name of a fitted file stands for.
type Longs<'a> = std::collections::BTreeMap<&'a str, String>;

/// The words of a line of code, the two characters of an operator as one
/// and each shortened library name in full; `None` for a preprocessor line
/// or a comment.
fn words(line: &str, long: &Longs) -> Option<Vec<String>> {
    let tokens = crate::compact::tokens(line).filter(|_| !line.starts_with(['#', '/']))?;
    let mut out: Vec<String> = Vec::new();
    let mut end = 0;
    for &(space, t) in &tokens {
        let at = t.as_ptr() as usize - line.as_ptr() as usize;
        let glued = !space && at == end;
        end = at + t.len();
        if let Some(l) = out.last_mut().filter(|l| {
            glued && l.len() == 1 && t.len() == 1 && crate::compact::JOINED.contains(&format!("{l}{t}").as_str())
        }) {
            l.push_str(t);
            continue;
        }
        // A short name stands for its long one everywhere, even after a
        // trailing `->`; a library name is a member's own after `.`.
        let member = out.last().is_some_and(|l| matches!(l.as_str(), "::" | "." | "->"));
        match long.get(t).filter(|_| !member || is_short(t)) {
            Some(name) => out.extend(name.split("::").enumerate().flat_map(|(i, part)| {
                (i > 0).then(|| "::".to_string()).into_iter().chain([part.to_string()])
            })),
            None => out.push(t.to_string()),
        }
    }
    Some(out)
}

/// A name fitting gave: `Q0`, `Qa`, `Q1z`.
fn is_short(w: &str) -> bool {
    w.strip_prefix('Q').is_some_and(|r| {
        !r.is_empty() && r.bytes().all(|c| c.is_ascii_digit() || c.is_ascii_lowercase())
    })
}

/// The runtime name each short name of a fitted file stands for, read off
/// the runtime's own lines: a line of the file that differs from exactly
/// one runtime line only where that line writes a runtime name gives each
/// of those names its short one.
fn runtime_longs<'a>(
    lines: impl Iterator<Item = &'a [String]>,
) -> std::collections::BTreeMap<String, &'static str> {
    use std::collections::{BTreeMap, HashMap};
    type Shapes = HashMap<String, Option<Vec<String>>>;
    static SHAPES: std::sync::OnceLock<Shapes> = std::sync::OnceLock::new();
    let names = crate::compact::runtime_names();
    let shape = |words: &[String], hole: &dyn Fn(&str) -> bool| -> String {
        let holes = words.iter().map(|w| if hole(w) { "\u{1}" } else { w.as_str() });
        holes.collect::<Vec<_>>().join(" ")
    };
    let runtime = |w: &str| names.contains(w);
    let shapes = SHAPES.get_or_init(|| {
        let mut shapes = Shapes::new();
        for piece in crate::runtime::pieces() {
            let code = crate::compact::compact(piece.code, false, &BTreeMap::new());
            for words in code.lines().filter_map(|l| words(l, &Longs::new())) {
                if !words.iter().any(|w| runtime(w)) {
                    continue;
                }
                let key = shape(&words, &runtime);
                let words = Some(words);
                shapes
                    .entry(key)
                    .and_modify(|seen| {
                        if *seen != words {
                            *seen = None;
                        }
                    })
                    .or_insert(words);
            }
        }
        shapes
    });
    let short = is_short;
    let mut longs: BTreeMap<&str, Option<&'static str>> = BTreeMap::new();
    for words in lines {
        if !words.iter().any(|w| short(w)) {
            continue;
        }
        let Some(Some(full)) = shapes.get(&shape(words, &|w| short(w) || runtime(w))) else {
            continue;
        };
        for (w, long) in words.iter().zip(full) {
            let Some(&long) = names.get(long.as_str()).filter(|_| short(w)) else {
                continue;
            };
            longs
                .entry(w.as_str())
                .and_modify(|seen| {
                    if *seen != Some(long) {
                        *seen = None;
                    }
                })
                .or_insert(Some(long));
        }
    }
    longs.into_iter().filter_map(|(short, long)| Some((short.to_string(), long?))).collect()
}

/// `words` written out with the spaces codegen puts between them.
fn spaced(words: &[String]) -> String {
    const SPACED: &[&str] = &[
        "=", "==", "!=", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "<=", ">=", "&&", "||",
        "?", ":",
    ];
    const BINARY: &[&str] = &["+", "-", "*", "/", "%", "|", "^", "<<"];
    let word = |w: &str| w.starts_with(|c: char| c.is_alphanumeric() || c == '_' || c == '"' || c == '\'');
    let value = |w: &str| word(w) && !matches!(w, "return" | "case" | "else") || matches!(w, ")" | "]");
    let mut out = String::new();
    let mut arrow = false;
    for (i, w) in words.iter().enumerate() {
        let (w, prev) = (w.as_str(), i.checked_sub(1).map(|p| words[p].as_str()));
        let last = i + 1 == words.len();
        let gap = prev.is_some_and(|p| {
            crate::compact::needs_space(p, w)
                || matches!(p, "," | ";")
                || SPACED.contains(&p)
                || SPACED.contains(&w)
                || matches!(p, "&" | "*") && word(w) && i >= 2 && (value(&words[i - 2]) || words[i - 2].ends_with('>'))
                || BINARY.contains(&w) && value(p)
                || BINARY.contains(&p) && i >= 2 && value(&words[i - 2])
                || w == "->" && p == ")"
                || p == "->" && arrow
                || w == "{" && (p == ")" || last || arrow)
                || p == "}" && word(w)
                || p.ends_with('>') && word(w)
                || w == "(" && matches!(p, "if" | "for" | "while" | "switch")
        });
        if w == "->" {
            arrow = prev == Some(")");
        }
        if gap {
            out.push(' ');
        }
        out += w;
    }
    out
}

/// The C++ container types a copy of which is deep.
pub const CONTAINERS: [&str; 6] = [
    "std::vector<",
    "std::map<",
    "std::set<",
    "std::unordered_map<",
    "std::deque<",
    "std::string",
];

/// A name or a place on one: `xs`, `self.rows`, `cpl::at(rows, i)`.
pub fn is_place(text: &str) -> bool {
    let text = text.trim();
    let name = |t: &str| {
        !t.is_empty()
            && t.split('.').all(|p| {
                p.starts_with(|c: char| c.is_alphabetic() || c == '_')
                    && p.chars().all(|c| c.is_alphanumeric() || c == '_')
            })
    };
    name(text) || (text.starts_with("cpl::at(") && text.ends_with(')') && !text[8..].contains("::"))
}

/// A place inside a value, `self.id` or `cpl::at(rows, i)`: returning it
/// from a fn that returns by value copies it, where a returned local moves.
pub(crate) fn is_part(text: &str) -> bool {
    if let Some((base, index)) = text.strip_suffix(']').and_then(|t| t.split_once('[')) {
        return is_part(base) && !base.starts_with("cpl::") && is_place(index);
    }
    is_place(text) && (text.contains('.') || text.starts_with("cpl::at("))
}

/// The type a line opening a fn or lambda body returns: `inline T f(..) {`,
/// `T f(..) {` or `[&]() -> T {`.
pub(crate) fn returned_type(line: &str) -> Option<&str> {
    let body = line.strip_suffix('{')?.trim_end();
    if let Some((_, ret)) = body.rsplit_once(") -> ") {
        return Some(ret.trim());
    }
    let head = body.strip_prefix("inline ").unwrap_or(body);
    let head = head.strip_prefix("static ").unwrap_or(head);
    let (sig, _) = head.split_once('(')?;
    let keyword = [
        "if", "for", "while", "switch", "else", "return", "struct", "do",
    ];
    let (ret, name) = sig.trim_end().rsplit_once([' ', '&', '*'])?;
    let ret = &sig[..ret.len() + 1];
    let first = ret.split([' ', '(']).next().unwrap_or("");
    (!keyword.contains(&first) && !name.is_empty() && !name.contains('>')).then(|| ret.trim())
}

/// The C++ types a local that shadows a container name may have.
const SCALARS: [&str; 9] = [
    "int",
    "ll",
    "bool",
    "double",
    "char",
    "auto",
    "unsigned",
    "__int128",
    "cpl::OptPos",
];

/// The name a scalar local declares, `int f = 0;` or `for (int f = lo; ..`,
/// which a lambda then captures as a scalar, not the container named `f`
/// further out.
pub fn scalar_decl(line: &str) -> Option<&str> {
    let decl = line.strip_prefix("for (").unwrap_or(line);
    let (ty, rest) = decl.split_once(' ')?;
    if !SCALARS.contains(&ty) {
        return None;
    }
    let name = rest.split([' ', '=', ';']).next()?;
    (!name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_')).then_some(name)
}

/// Every deep copy of a container the C++ spells out: a container local
/// initialised from a place (`std::vector<ll> row = cpl::at(rows, i);`), a
/// tuple of containers built from a pair's fields, and a by-value lambda
/// capture of a name declared as a container.
/// `std::optional<std::vector<ll>>(a)`: a container copied into an
/// optional, as a `?T` argument used to be. Counted apart from
/// [`container_copies`], under its own `// optional_copies:` header.
pub fn optional_copies(cpp: &str) -> usize {
    let cpp = &*unfitted(cpp);
    cpp.lines()
        .map(|l| optional_copies_on(l.trim(), &is_container))
        .sum()
}

pub fn is_container(ty: &str) -> bool {
    CONTAINERS.iter().any(|c| ty.starts_with(c))
}

pub(crate) fn optional_copies_on(line: &str, container: &dyn Fn(&str) -> bool) -> usize {
    let mut count = 0;
    let mut rest = line;
    while let Some(at) = rest.find("std::optional<") {
        rest = &rest[at + "std::optional<".len()..];
        let mut depth = 1;
        let Some(end) = rest.char_indices().find_map(|(i, c)| {
            depth += match c {
                '<' => 1,
                '>' => -1,
                _ => 0,
            };
            (depth == 0).then_some(i)
        }) else {
            break;
        };
        let (held, after) = (&rest[..end], &rest[end + 1..]);
        let arg = after
            .strip_prefix('(')
            .and_then(|a| a.find(')').map(|close| &a[..close]));
        if container(held) && arg.is_some_and(is_place) {
            count += 1;
        }
    }
    count
}

pub fn container_copies(cpp: &str) -> usize {
    let cpp = &*unfitted(cpp);
    let container = is_container;
    let mut held = std::collections::BTreeSet::new();
    let mut count = 0;
    // The fns and lambdas open at each line: the brace depth each body
    // opened at, and whether it returns a container by value.
    let mut bodies: Vec<(i64, bool)> = Vec::new();
    let mut depth = 0i64;
    // The fns that return a container by `const&`: a by-value return of
    // a call to one copies what it lends.
    let mut lending = std::collections::BTreeSet::new();
    for line in cpp.lines() {
        let line = line.trim();
        if let Some(ret) = returned_type(line) {
            bodies.push((depth, container(ret) && !ret.ends_with(['&', '*'])));
            if container(ret.trim_start_matches("const ")) && ret.ends_with('&') {
                lending.extend(
                    line.split_once('(')
                        .and_then(|(sig, _)| sig.rsplit([' ', '&']).next()),
                );
            }
        }
        depth += line.matches('{').count() as i64 - line.matches('}').count() as i64;
        let returns_container = bodies.last().is_some_and(|b| b.1);
        while bodies.last().is_some_and(|b| b.0 >= depth) {
            bodies.pop();
        }
        if returns_container
            && line
                .strip_prefix("return ")
                .and_then(|r| r.strip_suffix(';'))
                .is_some_and(|r| {
                    is_part(r) || r.split_once('(').is_some_and(|(f, _)| lending.contains(f))
                })
        {
            count += 1;
        }
        let decl = line.strip_prefix("const ").unwrap_or(line);
        if let Some(name) = scalar_decl(decl) {
            held.remove(name);
        }
        if container(decl) {
            // `T name = place;`, `const T& name`, `T name(...)`: the name is
            // the last word before `=`, `(`, `;`, `,` or `)` at depth 0.
            let (mut depth, mut end) = (0i32, decl.len());
            for (i, c) in decl.char_indices() {
                match c {
                    '<' => depth += 1,
                    '>' => depth -= 1,
                    '=' | '(' | ';' | ',' | ')' | '{' if depth == 0 => {
                        end = i;
                        break;
                    }
                    _ => {}
                }
            }
            let head = decl[..end].trim_end();
            let name = head.rsplit([' ', '&']).next().unwrap_or("");
            if !name.is_empty() && name.len() < head.len() {
                held.insert(name.to_string());
                let by_value = !head[..head.len() - name.len()]
                    .trim_end()
                    .ends_with(['&', '*']);
                let rhs = decl[end..]
                    .strip_prefix('=')
                    .and_then(|r| r.strip_suffix(';'));
                if by_value && rhs.is_some_and(is_place) {
                    count += 1;
                }
            }
        }
        if line.starts_with("std::tuple<")
            && CONTAINERS.iter().any(|c| line.contains(c))
            && line.contains(".first, ")
            && line.contains(".second)")
        {
            count += 1;
        }
        if line.contains("cpl_f(") {
            count += tuples_of_places(line);
        }
        let Some(open) = line.find("= [") else {
            continue;
        };
        let Some(close) = line[open..].find("](") else {
            continue;
        };
        count += line[open + 3..open + close]
            .split(", ")
            .filter(|cap| !cap.starts_with('&'))
            .filter_map(|cap| cap.split_once(" = "))
            .filter(|(_, outer)| held.contains(outer.trim()))
            .count();
    }
    count
}

/// Each `std::tuple<std::vector<int>, ..>{xs[i], xs[j]}` on a line: a tuple
/// that holds a container, built from places, copies them. Counted where a
/// `cpl_each_` form yields one, which can hand the places as references.
fn tuples_of_places(line: &str) -> usize {
    let mut count = 0;
    let mut rest = line;
    while let Some(at) = rest.find("std::tuple<") {
        rest = &rest[at + "std::tuple<".len()..];
        let mut depth = 1;
        let Some(end) = rest.char_indices().find_map(|(i, c)| {
            depth += match c {
                '<' => 1,
                '>' => -1,
                _ => 0,
            };
            (depth == 0).then_some(i)
        }) else {
            break;
        };
        let (held, after) = (&rest[..end], &rest[end + 1..]);
        let Some(inner) = after.strip_prefix('{') else {
            continue;
        };
        let mut depth = 1;
        let Some(close) = inner.char_indices().find_map(|(i, c)| {
            depth += match c {
                '{' | '(' => 1,
                '}' | ')' => -1,
                _ => 0,
            };
            (depth == 0).then_some(i)
        }) else {
            continue;
        };
        let args = &inner[..close];
        let indexed = |a: &str| {
            a.strip_suffix(']')
                .and_then(|t| t.split_once('['))
                .is_some_and(|(base, i)| is_place(base) && is_place(i))
        };
        let tys = top_level(held);
        let args = top_level(args);
        count += usize::from(
            tys.len() == args.len()
                && tys
                    .iter()
                    .zip(&args)
                    .any(|(t, a)| is_container(t) && (is_place(a) || is_part(a) || indexed(a))),
        );
    }
    count
}

/// `a, std::map<ll, ll>, c` split at its top-level commas.
fn top_level(list: &str) -> Vec<&str> {
    let (mut out, mut depth, mut from) = (Vec::new(), 0i32, 0);
    for (i, c) in list.char_indices() {
        match c {
            '<' | '(' | '[' | '{' => depth += 1,
            '>' | ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                out.push(list[from..i].trim());
                from = i + 1;
            }
            _ => {}
        }
    }
    out.push(list[from..].trim());
    out
}

/// Each `std::function` the C++ declares: a fn value whose callee codegen
/// could not pin down, called indirectly.
///
/// A big program's C++ may shorten the type to an alias
/// (`template<class...T>using Qy=std::function<T...>;`): the alias line
/// declares nothing, and each use of the alias counts.
pub fn std_functions(cpp: &str) -> usize {
    let cpp = &*unfitted(cpp);
    let code: Vec<&str> = cpp
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect();
    let aliases: Vec<&str> = code.iter().filter_map(|l| function_alias(l)).collect();
    code.iter()
        .map(|l| match function_alias(l) {
            Some(_) => 0,
            None => {
                l.matches("std::function<").count()
                    + aliases.iter().map(|a| uses_of(l, a)).sum::<usize>()
            }
        })
        .sum()
}

/// `Qy` for `template<class...T>using Qy=std::function<T...>;` or `using Qy = std::function<..>;`.
pub(crate) fn function_alias(line: &str) -> Option<&str> {
    let (_, rest) = line.split_once("using ")?;
    let (name, ty) = rest.split_once('=')?;
    let name = name.trim();
    let ident = !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_');
    (ident && ty.trim_start().starts_with("std::function<")).then_some(name)
}

/// How many times `name<` appears in `line` as a whole identifier.
pub(crate) fn uses_of(line: &str, name: &str) -> usize {
    let pat = format!("{name}<");
    line.match_indices(&pat)
        .filter(|(at, _)| {
            !line[..*at]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_alphanumeric() || c == '_' || c == ':')
        })
        .count()
}

/// `std::optional<std::vector<..>> t = cpl::map_get(m, k)`: the entry
/// copied out of the map to be read.
pub fn map_get_copies(cpp: &str) -> usize {
    let cpp = &*unfitted(cpp);
    cpp.lines()
        .filter(|l| l.contains("std::optional<std::vector<") && l.contains("= cpl::map_get("))
        .count()
}

/// `[&]() -> std::vector<..> { D_len = ..; return xs; }()`: a block that
/// sizes a domain from a list, then gives the list by value.
pub fn block_copies(cpp: &str) -> usize {
    let cpp = &*unfitted(cpp);
    let lines: Vec<&str> = cpp.lines().collect();
    (0..lines.len())
        .filter(|&i| lines[i].contains("[&]() -> std::vector<") && lines[i].ends_with('{'))
        .filter(|&i| {
            let lens = lines[i + 1..]
                .iter()
                .take_while(|l| {
                    l.trim()
                        .split(" = ")
                        .next()
                        .is_some_and(|lhs| lhs.ends_with("_len"))
                })
                .count();
            lens > 0
                && lines.get(i + 1 + lens).is_some_and(|l| {
                    let l = l.trim();
                    l.strip_prefix("return ")
                        .and_then(|r| r.strip_suffix(';'))
                        .is_some_and(|v| v.chars().all(|c| c.is_alphanumeric() || c == '_'))
                })
        })
        .count()
}

/// `std::vector<ll>(cpl::map_or(m, k, d))`: a copy of an entry, made to
/// be changed.
pub fn entry_copies(cpp: &str) -> usize {
    let cpp = &*unfitted(cpp);
    cpp.lines().filter(|l| l.contains(">(cpl::map_or(")).count()
}

/// The items of an argument or parameter list at depth 0.
pub(crate) fn items(list: &str) -> Vec<&str> {
    let (mut out, mut depth, mut start) = (Vec::new(), 0i32, 0);
    for (i, c) in list.char_indices() {
        match c {
            '(' | '<' | '[' | '{' => depth += 1,
            ')' | '>' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                out.push(list[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    if !list[start..].trim().is_empty() {
        out.push(list[start..].trim());
    }
    out
}

/// The structs whose fields hold no container or `std::function`, directly
/// or through another struct: passing one by value copies a few words.
pub fn flat_structs(lines: &[&str]) -> std::collections::BTreeSet<String> {
    let mut fields = Vec::new();
    let mut rest = lines;
    while let Some((line, tail)) = rest.split_first() {
        rest = tail;
        let Some(name) = line
            .strip_prefix("struct ")
            .and_then(|l| l.strip_suffix(" {"))
        else {
            continue;
        };
        let body: Vec<&str> = tail
            .iter()
            .take_while(|l| **l != "};")
            .map(|l| l.trim())
            .filter(|l| !l.starts_with("auto ") && !l.starts_with("bool operator"))
            .collect();
        fields.push((name.to_string(), body));
    }
    let mut flat = std::collections::BTreeSet::new();
    loop {
        let more: Vec<String> = fields
            .iter()
            .filter(|(name, _)| !flat.contains(name))
            .filter(|(_, body)| {
                body.iter().all(|f| {
                    let decl = f.split(" = ").next().unwrap_or(f);
                    let ty = decl.rsplit_once(' ').map_or(decl, |(ty, _)| ty);
                    !CONTAINERS.iter().any(|c| f.contains(c))
                        && !f.contains("std::function")
                        && (!ty.starts_with(|c: char| c.is_ascii_uppercase()) || flat.contains(ty))
                })
            })
            .map(|(name, _)| name.clone())
            .collect();
        if more.is_empty() {
            return flat;
        }
        flat.extend(more);
    }
}

/// `inline H wrap(std::vector<ll> xs) { return H{std::move(xs)}; }` called
/// as `wrap(a)`: a fn takes a heap parameter by value to move it into the
/// struct it returns, and a call hands it a named place, which is copied.
/// A lent field points to its value instead (R05.4.4, R13.4).
pub fn sunk_copies(cpp: &str) -> usize {
    let cpp = &*unfitted(cpp);
    let lines: Vec<&str> = cpp.lines().collect();
    let flat = flat_structs(&lines);
    let head = |l: &str| -> Option<(String, String)> {
        let l = l.strip_prefix("inline ").unwrap_or(l).strip_suffix(") {")?;
        let (sig, params) = l.split_once('(')?;
        let name = sig.rsplit(' ').next()?;
        let named = sig.contains(' ') && name.chars().all(|c| c.is_alphanumeric() || c == '_');
        named.then(|| (name.to_string(), params.to_string()))
    };
    let mut sunk = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let Some((name, params)) = head(line) else {
            continue;
        };
        let body = lines[i..].iter().take_while(|l| **l != "}");
        let returns: Vec<&str> = body
            .map(|l| l.trim())
            .filter(|l| l.starts_with("return ") && l.contains('{'))
            .collect();
        for (k, param) in items(&params).into_iter().enumerate() {
            let Some((ty, p)) = param.rsplit_once(' ') else {
                continue;
            };
            let by_value = !ty.ends_with('&') && !ty.starts_with("[[") && !ty.starts_with('F');
            let heap = is_container(ty)
                || ty.starts_with(|c: char| c.is_ascii_uppercase()) && !flat.contains(ty);
            let moved = format!("std::move({p})");
            if by_value && heap && returns.iter().any(|r| r.contains(&moved)) {
                sunk.push((name.clone(), k));
            }
        }
    }
    let mut count = 0;
    let code = |l: &&&str| head(l).is_none() && !l.trim_start().starts_with("//");
    for line in lines.iter().filter(code) {
        for (name, k) in &sunk {
            let call = format!("{name}(");
            for (at, _) in line.match_indices(&call) {
                let before = line[..at].chars().next_back();
                if before.is_some_and(|c| c.is_alphanumeric() || c == '_') {
                    continue;
                }
                let rest = &line[at + call.len()..];
                let mut depth = 1;
                let end = rest.char_indices().find(|&(_, c)| {
                    depth += match c {
                        '(' => 1,
                        ')' => -1,
                        _ => 0,
                    };
                    depth == 0
                });
                let args = end.map_or(Vec::new(), |(e, _)| items(&rest[..e]));
                count += usize::from(args.get(*k).is_some_and(|a| is_place(a)));
            }
        }
    }
    count
}

/// `std::vector<..> f(const S& self) {` then `return self.xs;`: a fn, like a
/// `@get` of a stored list, that copies out a part of what it was lent
/// instead of lending it on.
pub fn lent_part_copies(cpp: &str) -> usize {
    let cpp = &*unfitted(cpp);
    let lines: Vec<&str> = cpp.lines().collect();
    lines
        .windows(2)
        .filter(|w| {
            let sig = w[0].trim();
            let sig = sig.strip_prefix("inline ").unwrap_or(sig);
            let Some(part) = w[1].trim().strip_prefix("return ") else {
                return false;
            };
            let name: String = part
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            is_container(sig)
                && sig.ends_with(") {")
                && !name.is_empty()
                && part[name.len()..].starts_with(['.', '['])
                && part.ends_with(';')
                && [format!("& {name},"), format!("& {name})")]
                    .iter()
                    .any(|p| {
                        sig.find(p.as_str())
                            .is_some_and(|at| sig[..at].contains("const "))
                    })
        })
        .count()
}

/// The value of the header line `// <key>: N` (0 without one), or why the
/// header is malformed. The header is the file's leading `//` lines.
pub fn declared(text: &str, key: &str) -> Result<usize, String> {
    let mut found = None;
    for line in text.lines() {
        let Some(rest) = line.trim_start().strip_prefix("//") else {
            break;
        };
        let Some(v) = rest
            .trim_start()
            .strip_prefix(key)
            .and_then(|r| r.strip_prefix(':'))
        else {
            continue;
        };
        let n = v
            .trim()
            .parse::<usize>()
            .ok()
            .filter(|&n| n > 0)
            .ok_or_else(|| format!("`//{rest}`: write a count above 0, or drop the line for 0"))?;
        if found.replace(n).is_some() {
            return Err(format!("two `// {key}:` lines"));
        }
    }
    Ok(found.unwrap_or(0))
}
