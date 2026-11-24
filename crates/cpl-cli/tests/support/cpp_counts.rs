//! Counters over generated C++ that the copy ratchet and the explain
//! accuracy test share: what the C++ itself says is copied or called
//! indirectly, independent of what codegen reports.

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
fn is_part(text: &str) -> bool {
    if let Some((base, index)) = text.strip_suffix(']').and_then(|t| t.split_once('[')) {
        return is_part(base) && !base.starts_with("cpl::") && is_place(index);
    }
    is_place(text) && (text.contains('.') || text.starts_with("cpl::at("))
}

/// The type a line opening a fn or lambda body returns: `inline T f(..) {`,
/// `T f(..) {` or `[&]() -> T {`.
fn returned_type(line: &str) -> Option<&str> {
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
fn scalar_decl(line: &str) -> Option<&str> {
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
/// [`deep_copies`], under its own `// optional_copies:` header.
pub fn optional_copies(cpp: &str) -> usize {
    cpp.lines()
        .map(|l| optional_copies_on(l.trim(), &is_container))
        .sum()
}

pub fn is_container(ty: &str) -> bool {
    CONTAINERS.iter().any(|c| ty.starts_with(c))
}

fn optional_copies_on(line: &str, container: &dyn Fn(&str) -> bool) -> usize {
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

pub fn deep_copies(cpp: &str) -> usize {
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
fn function_alias(line: &str) -> Option<&str> {
    let (_, rest) = line.split_once("using ")?;
    let (name, ty) = rest.split_once('=')?;
    let name = name.trim();
    let ident = !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_');
    (ident && ty.trim_start().starts_with("std::function<")).then_some(name)
}

/// How many times `name<` appears in `line` as a whole identifier.
fn uses_of(line: &str, name: &str) -> usize {
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
