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
pub use cpl_diag::plain;

pub mod json {
    use crate::{links, Example, Facts, Item, Module};
    use cpl_json::{obj, Value, Writer};

    fn list<T>(w: &mut Writer, items: &[T], mut one: impl FnMut(&mut Writer, &T)) {
        w.array();
        for item in items {
            one(w, item);
        }
        w.end();
    }

    fn opt(text: &Option<String>) -> Value {
        text.as_deref().map_or(Value::Null, Value::from)
    }

    fn example(w: &mut Writer, example: &Example) {
        let (setup, expr, value) = example.claim();
        let rest = [
            ("block", Value::Bool(example.block)),
            ("expect", opt(&example.expect)),
            ("expr", expr.into()),
            (
                "setup",
                Value::Array(setup.into_iter().map(Value::from).collect()),
            ),
            ("stdin", opt(&example.stdin)),
            ("text", example.text.as_str().into()),
            ("value", opt(&value)),
        ];
        w.object().key("code").opt_str(example.code.as_deref());
        w.key("line").int(example.line);
        for (key, value) in &rest {
            w.key(key).value(value);
        }
        w.end();
    }

    fn facts(facts: &Facts) -> Value {
        let [cost, traps, empty, ties, order] = Facts::LABELS;
        obj([
            (cost, opt(&facts.cost)),
            (traps, opt(&facts.traps)),
            (empty, opt(&facts.empty)),
            (ties, opt(&facts.ties)),
            (order, opt(&facts.order)),
        ])
    }

    fn link(w: &mut Writer, target: &links::Target) {
        w.object().key("text").str(&target.text);
        w.key("path").str(&target.path);
        w.key("module").str(&target.module);
        w.key("anchor").str(&target.anchor).end();
    }

    /// `owner` scopes its intra-doc links: the type it is a member of.
    fn item(w: &mut Writer, modules: &[Module], m: usize, owner: Option<&str>, item: &Item) {
        let scope = owner.or((!item.members.is_empty()).then_some(item.name.as_str()));
        let links = links::targets(modules, m, scope, &item.doc);
        w.object().key("kind").str(item.kind.as_str());
        w.key("name").str(&item.name);
        w.key("signature").str(&item.signature);
        w.key("doc").str(&item.doc);
        w.key("summary").str(item.summary());
        w.key("cost").opt_str(item.cost.as_deref());
        w.key("facts").value(&facts(&item.facts));
        w.key("prose").str(&item.prose);
        w.key("line").int(item.line);
        w.key("examples");
        list(w, &item.examples, example);
        w.key("links");
        list(w, &links, link);
        w.key("members");
        list(w, &item.members, |w, member| {
            self::item(w, modules, m, Some(&item.name), member)
        });
        w.end();
    }

    /// Every module as one JSON document.
    pub fn render(modules: &[Module]) -> String {
        let indexed: Vec<usize> = (0..modules.len()).collect();
        let mut out = cpl_json::write_with(|w| {
            w.object().key("modules");
            list(w, &indexed, |w, &m| {
                let module = &modules[m];
                w.object().key("name").str(&module.name);
                w.key("path").str(&module.path);
                w.key("doc").str(&module.doc);
                w.key("examples");
                list(w, &module.examples, example);
                w.key("links");
                list(w, &links::targets(modules, m, None, &module.doc), link);
                w.key("items");
                list(w, &module.items, |w, i| item(w, modules, m, None, i));
                w.end();
            });
            w.end();
        });
        out.push('\n');
        out
    }
}

pub use model::{builtins, extract, DocText, Example, Facts, Item, Kind, Module};

/// Name of this crate.
pub const NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests;
