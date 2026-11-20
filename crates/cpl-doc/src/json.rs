//! Modules as JSON, for a site generator that has its own templates.
//!
//! ```json
//! {"modules": [{"name": "prefix", "path": "lib/prefix.cp", "doc": "...",
//!   "items": [{"kind": "fn", "name": "sum", "signature": "fn sum(...)",
//!     "doc": "...", "summary": "...", "cost": "O(1)", "line": 12,
//!     "examples": [{"code": "...", "line": 8}], "members": []}]}]}
//! ```

use crate::{Example, Item, Module};

/// `text` as a JSON string, quotes included.
pub(crate) fn string(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn list<T>(items: &[T], one: impl Fn(&T) -> String) -> String {
    format!("[{}]", items.iter().map(one).collect::<Vec<_>>().join(","))
}

fn example(example: &Example) -> String {
    format!(
        "{{\"code\":{},\"line\":{}}}",
        string(&example.code),
        example.line
    )
}

fn item(item: &Item) -> String {
    format!(
        "{{\"kind\":{},\"name\":{},\"signature\":{},\"doc\":{},\"summary\":{},\"cost\":{},\"line\":{},\"examples\":{},\"members\":{}}}",
        string(item.kind.as_str()),
        string(&item.name),
        string(&item.signature),
        string(&item.doc),
        string(item.summary()),
        item.cost.as_deref().map_or("null".to_string(), string),
        item.line,
        list(&item.examples, example),
        list(&item.members, self::item),
    )
}

/// Every module as one JSON document.
pub fn render(modules: &[Module]) -> String {
    let modules = list(modules, |module| {
        format!(
            "{{\"name\":{},\"path\":{},\"doc\":{},\"examples\":{},\"items\":{}}}",
            string(&module.name),
            string(&module.path),
            string(&module.doc),
            list(&module.examples, example),
            list(&module.items, item),
        )
    });
    format!("{{\"modules\":{modules}}}\n")
}
