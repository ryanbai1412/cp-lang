//! Counters over generated C++ that the copy and `std::function` ratchets
//! (`crates/cpl-cli/tests/read_only_copies.rs`) and the explain accuracy
//! test read: what the C++ itself says is copied or called indirectly,
//! independent of what codegen reports. They read text, never the IR, so a
//! codegen bug can't hide from them; `cpp_counts_tests.rs` pins each shape.

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
        } else if !line.contains(".first, ") && decl.starts_with("std::tuple<") {
            count += tuple_decl_copies(decl);
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

/// Copies in a by-value `std::tuple<..> t = place;` that holds a
/// container, grid or array, and each such item of `std::tuple<..> t = T{a.x, b};`
/// read from a place. Mirrors tuple_copies in
/// scripts/specgate/explain_copies.py.
fn tuple_decl_copies(decl: &str) -> usize {
    let rest = &decl["std::tuple<".len()..];
    let mut depth = 1;
    let Some(end) = rest.char_indices().find_map(|(i, c)| {
        depth += match c {
            '<' => 1,
            '>' => -1,
            _ => 0,
        };
        (depth == 0).then_some(i)
    }) else {
        return 0;
    };
    let tys = top_level(&rest[..end]);
    let Some((name, init)) = rest[end + 1..].split_once('=') else {
        return 0;
    };
    let init = init.trim().trim_end_matches(';').trim();
    if !is_place(name) || name.contains('.') {
        return 0;
    }
    let place = |a: &str| is_place(a) || place_text(a);
    let heap =
        |t: &str| is_container(t) || t.starts_with("cpl::Grid<") || t.starts_with("std::array<");
    if place(init) {
        return usize::from(tys.iter().any(|t| heap(t)));
    }
    let Some(items) = init
        .find('{')
        .filter(|_| init.ends_with('}'))
        .map(|at| &init[at + 1..init.len() - 1])
    else {
        return 0;
    };
    tys.iter()
        .zip(top_level(items))
        .filter(|(t, a)| heap(t) && place(a))
        .count()
}

/// A name with parts, `a.b[i].c`, as the explain gate's counter reads one.
fn place_text(text: &str) -> bool {
    let ident = |t: &str| {
        t.find(|c: char| !(c.is_alphanumeric() || c == '_'))
            .unwrap_or(t.len())
    };
    let text = text.trim();
    let mut at = ident(text);
    if at == 0 || text.starts_with(|c: char| c.is_ascii_digit()) {
        return false;
    }
    while at < text.len() {
        let rest = &text[at..];
        at += match rest.as_bytes()[0] {
            b'.' if ident(&rest[1..]) > 0 => 1 + ident(&rest[1..]),
            b'[' => match rest.find(']') {
                Some(close) if close > 1 && !rest[1..close].contains('[') => close + 1,
                _ => return false,
            },
            _ => return false,
        };
    }
    true
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
    cpp.lines()
        .filter(|l| l.contains("std::optional<std::vector<") && l.contains("= cpl::map_get("))
        .count()
}

/// `[&]() -> std::vector<..> { D_len = ..; return xs; }()`: a block that
/// sizes a domain from a list, then gives the list by value.
pub fn block_copies(cpp: &str) -> usize {
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
