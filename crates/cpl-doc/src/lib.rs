//! `cpl doc`: documentation from `///` comments (`plan/45-dev-tools.md`).
//!
//! ```text
//! /// The sum of `a[l..r]` in O(1), after O(n) to build.
//! ///
//! /// ```
//! /// let p = Prefix([3, 1, 4])
//! /// print(p.sum(0, 2))
//! /// ```
//! @cost("O(1)")
//! fn sum(self, l: Int, r: Int) -> Int
//! ```
//!
//! [`extract`] reads one source file into a [`Module`]: every declaration
//! with its signature, its doc comment, its `@cost`, the line it is on and
//! the examples in its comment. [`html::render`] turns modules into a site,
//! [`json::render`] into data for another generator, and [`examples::check`]
//! is what `cpl doc --test` runs.

mod highlight;
mod markdown;
mod model;

pub mod cli;
pub mod examples;
pub mod html;
pub mod links;
pub mod plain;

pub mod json {
    use crate::{links, Example, Item, Module};
    use cpl_json::escape_string;

    pub(crate) fn string(text: &str) -> String {
        escape_string(text)
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

    fn link(target: &links::Target) -> String {
        format!(
            "{{\"text\":{},\"path\":{},\"module\":{},\"anchor\":{}}}",
            string(&target.text),
            string(&target.path),
            string(&target.module),
            string(&target.anchor)
        )
    }

    /// `owner` scopes its intra-doc links: the type it is a member of.
    fn item(modules: &[Module], m: usize, owner: Option<&str>, item: &Item) -> String {
        let scope = owner.or((!item.members.is_empty()).then_some(item.name.as_str()));
        let links = links::targets(modules, m, scope, &item.doc);
        format!(
            "{{\"kind\":{},\"name\":{},\"signature\":{},\"doc\":{},\"summary\":{},\"cost\":{},\"line\":{},\"examples\":{},\"links\":{},\"members\":{}}}",
            string(item.kind.as_str()),
            string(&item.name),
            string(&item.signature),
            string(&item.doc),
            string(item.summary()),
            item.cost.as_deref().map_or("null".to_string(), string),
            item.line,
            list(&item.examples, example),
            list(&links, link),
            list(&item.members, |member| self::item(modules, m, Some(&item.name), member)),
        )
    }

    /// Every module as one JSON document.
    pub fn render(modules: &[Module]) -> String {
        let all = modules;
        let indexed: Vec<usize> = (0..modules.len()).collect();
        let modules = list(&indexed, |&m| {
            let module = &all[m];
            format!(
                "{{\"name\":{},\"path\":{},\"doc\":{},\"examples\":{},\"links\":{},\"items\":{}}}",
                string(&module.name),
                string(&module.path),
                string(&module.doc),
                list(&module.examples, example),
                list(&links::targets(all, m, None, &module.doc), link),
                list(&module.items, |i| item(all, m, None, i)),
            )
        });
        format!("{{\"modules\":{modules}}}\n")
    }
}

pub use model::{extract, Example, Item, Kind, Module};

/// Name of this crate.
pub const NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests;
