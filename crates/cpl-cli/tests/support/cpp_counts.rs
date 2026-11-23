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

/// Every deep copy of a container the C++ spells out: a container local
/// initialised from a place (`std::vector<ll> row = cpl::at(rows, i);`), a
/// tuple of containers built from a pair's fields, and a by-value lambda
/// capture of a name declared as a container.
pub fn deep_copies(cpp: &str) -> usize {
    let container = |ty: &str| CONTAINERS.iter().any(|c| ty.starts_with(c));
    let mut held = std::collections::BTreeSet::new();
    let mut count = 0;
    for line in cpp.lines() {
        let line = line.trim();
        let decl = line.strip_prefix("const ").unwrap_or(line);
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
                let by_value = !head[..head.len() - name.len()].trim_end().ends_with('&');
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
    }
    for line in cpp.lines() {
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
pub fn std_functions(cpp: &str) -> usize {
    cpp.lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .map(|l| l.matches("std::function<").count())
        .sum()
}
