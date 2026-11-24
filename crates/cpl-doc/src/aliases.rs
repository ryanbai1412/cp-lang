//! What a contestant arriving from C++, Python, Rust or the AtCoder Library
//! searches for. The left-hand names of `spec/lib/inventory.md` are the one
//! alias table: `lower_bound` finds `first_ge`.

use crate::Module;
use std::sync::OnceLock;

/// The table, compiled in so `cpl doc` needs no checkout.
pub const INVENTORY: &str = include_str!("../../../spec/lib/inventory.md");

const LIBS: [&str; 4] = ["C++", "Python", "Rust", "ACL"];

/// One operation of the table.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Row<'a> {
    /// The 1-based line it is on.
    pub line: usize,
    pub lib: &'a str,
    /// What it is called there, arguments dropped: `lower_bound`, `math.comb`.
    pub from: Vec<String>,
    /// The std names its CP Lang spelling uses: `first_ge`, `Heap`.
    pub names: Vec<String>,
}

/// The rows of an inventory table.
pub fn rows(text: &str) -> Vec<Row<'_>> {
    let row = |(n, line): (usize, &str)| {
        let inner = line.trim().strip_prefix('|')?.strip_suffix('|')?;
        let cells: Vec<&str> = inner.split('|').map(str::trim).collect();
        let [lib, from, to] = cells[..] else {
            return None;
        };
        Some(Row {
            line: n + 1,
            lib: LIBS.into_iter().find(|l| *l == lib)?,
            from: code(from).map(unapplied).collect(),
            names: code(to).flat_map(named).collect(),
        })
    };
    text.lines().enumerate().filter_map(row).collect()
}

fn code(cell: &str) -> impl Iterator<Item = &str> {
    cell.split('`').skip(1).step_by(2)
}

/// `pow(a, b, m)` -> `pow`, `iter().rev()` -> `iter.rev`.
fn unapplied(span: &str) -> String {
    let mut depth = 0;
    let mut out = String::new();
    for c in span.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out
}

/// The names a spelling uses: a member after `.`, a fn before `(`, or a
/// type. Parameters and operators (`SegTree(op, a, act: f)`, `scan(+)`)
/// are not names.
fn named(span: &str) -> Vec<String> {
    let bytes = span.as_bytes();
    let word = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if !(bytes[i].is_ascii_alphabetic() || bytes[i] == b'_') || (i > 0 && word(bytes[i - 1])) {
            i += 1;
            continue;
        }
        let start = i;
        while i < bytes.len() && word(bytes[i]) {
            i += 1;
        }
        let upper = bytes[start].is_ascii_uppercase();
        let member = start > 0 && bytes[start - 1] == b'.';
        if upper || member || bytes.get(i) == Some(&b'(') {
            out.push(span[start..i].to_string());
        }
    }
    out
}

/// What the table says std's `name` is called elsewhere, in table order.
pub fn of(name: &str) -> &'static [String] {
    static BY_NAME: OnceLock<Vec<(String, Vec<String>)>> = OnceLock::new();
    let table = BY_NAME.get_or_init(|| {
        let mut out: Vec<(String, Vec<String>)> = Vec::new();
        for row in rows(INVENTORY) {
            for name in &row.names {
                let at = match out.iter().position(|(n, _)| n == name) {
                    Some(at) => at,
                    None => {
                        out.push((name.clone(), Vec::new()));
                        out.len() - 1
                    }
                };
                let known = &mut out[at].1;
                for from in &row.from {
                    if from != name && !known.contains(from) {
                        known.push(from.clone());
                    }
                }
            }
        }
        out
    });
    let base = name.split('<').next().unwrap_or(name);
    table
        .iter()
        .find(|(n, _)| n == base)
        .map_or(&[], |(_, known)| known.as_slice())
}

/// Gives every item of std its names elsewhere.
pub fn attach(modules: &mut [Module]) {
    fn walk(items: &mut [crate::Item]) {
        for item in items {
            item.aliases = of(&item.name).to_vec();
            walk(&mut item.members);
        }
    }
    for module in modules {
        walk(&mut module.items);
    }
}
