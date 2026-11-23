//! Intra-doc links: [`up`], [`Rooted.up`] or [`Monoid`] in a doc comment
//! name an item (or [`text`](Rooted.up), naming it explicitly), resolved from the scope of the item documented, as
//! rustdoc does. Plain `up` stays code. A link that names no item is an
//! error, so a rename can't leave a dead reference behind.

use crate::Module;

/// Where a link goes: the module's page and the item's anchor on it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Target {
    /// The name as written between the backticks.
    pub text: String,
    /// The item path it names: `text`, or the target of [`text`](path).
    pub path: String,
    pub module: String,
    /// `sum`, or `Prefix.sum` for a member, as [`crate::Item::anchor`].
    pub anchor: String,
}

/// A link that names no item.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Unresolved {
    pub path: String,
    /// The item whose comment holds it, or the module's name.
    pub item: String,
    /// The 1-based line of that item's declaration.
    pub line: u32,
    pub text: String,
}

/// The `(text, path)` of each link in `doc` outside fenced code: `` [`name`] ``
/// names `name`, `` [`text`](path) `` names `path` when it is an item path,
/// not a URL.
pub fn spans(doc: &str) -> Vec<(&str, &str)> {
    let mut out = Vec::new();
    let mut fenced = false;
    for line in doc.lines() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        let mut rest = line;
        while let Some(at) = rest.find("[`") {
            let tail = &rest[at + 2..];
            let Some(end) = tail.find("`]") else { break };
            let name = &tail[..end];
            rest = &tail[end + 2..];
            if name.contains('`') {
                continue;
            }
            match rest.strip_prefix('(').and_then(|r| r.split_once(')')) {
                Some((path, after)) => {
                    rest = after;
                    if !path.is_empty()
                        && path
                            .chars()
                            .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
                    {
                        out.push((name, path));
                    }
                }
                None => out.push((name, name)),
            }
        }
    }
    out
}

/// Resolves `name` from the scope of an item in `modules[module]`:
/// `owner` is the type the item is a member of, or the item itself when
/// it has members. A plain name is looked up among the owner's members,
/// then the module's items, then every other module's; `Type.member`
/// names a member of a type anywhere.
pub fn resolve(
    modules: &[Module],
    module: usize,
    owner: Option<&str>,
    text: &str,
    path: &str,
) -> Option<Target> {
    let name = path.strip_suffix("()").unwrap_or(path);
    let order = || std::iter::once(module).chain((0..modules.len()).filter(move |&m| m != module));
    let target = |m: usize, anchor: String| Target {
        text: text.to_string(),
        path: path.to_string(),
        module: modules[m].name.clone(),
        anchor,
    };
    let has_member = |m: usize, ty: &str, member: &str| {
        modules[m]
            .items
            .iter()
            .filter(|i| i.name == ty)
            .any(|i| i.members.iter().any(|x| x.name == member))
    };
    if let Some((ty, member)) = name.split_once('.') {
        return order()
            .find(|&m| has_member(m, ty, member))
            .map(|m| target(m, format!("{ty}.{member}")));
    }
    if let Some(owner) = owner {
        if has_member(module, owner, name) {
            return Some(target(module, format!("{owner}.{name}")));
        }
    }
    order()
        .find(|&m| modules[m].items.iter().any(|i| i.name == name))
        .map(|m| target(m, name.to_string()))
}

/// The links of one doc comment that resolve.
pub fn targets(modules: &[Module], module: usize, owner: Option<&str>, doc: &str) -> Vec<Target> {
    spans(doc)
        .into_iter()
        .filter_map(|(text, path)| resolve(modules, module, owner, text, path))
        .collect()
}

/// Every link in every comment that names no item.
pub fn unresolved(modules: &[Module]) -> Vec<Unresolved> {
    let mut out = Vec::new();
    for (m, module) in modules.iter().enumerate() {
        let mut check = |owner: Option<&str>, doc: &str, item: String, line: u32| {
            for (text, path) in spans(doc) {
                if resolve(modules, m, owner, text, path).is_none() {
                    out.push(Unresolved {
                        path: module.path.clone(),
                        item: item.clone(),
                        line,
                        text: path.to_string(),
                    });
                }
            }
        };
        check(None, &module.doc, module.name.clone(), 1);
        for item in &module.items {
            let own = (!item.members.is_empty()).then_some(item.name.as_str());
            check(own, &item.doc, item.name.clone(), item.line);
            for member in &item.members {
                check(
                    Some(&item.name),
                    &member.doc,
                    member.anchor(Some(&item.name)),
                    member.line,
                );
            }
        }
    }
    out
}

/// `doc` with each resolved link pointing at `url(target)`: `` [`up`] `` and
/// `` [`text`](Rooted.up) `` become `` [`up`](url) ``. Others are left as written.
pub fn rewrite(doc: &str, targets: &[Target], url: impl Fn(&Target) -> String) -> String {
    let mut out = doc.to_string();
    for t in targets {
        let to = format!("[`{}`]({})", t.text, url(t));
        let explicit = format!("[`{}`]({})", t.text, t.path);
        if out.contains(&explicit) {
            out = out.replace(&explicit, &to);
        } else {
            let bare = format!("[`{}`]", t.text);
            let mut rest = out.as_str();
            let mut done = String::new();
            while let Some(at) = rest.find(&bare) {
                let after = &rest[at + bare.len()..];
                done.push_str(&rest[..at]);
                if after.starts_with('(') {
                    done.push_str(&bare);
                } else {
                    done.push_str(&to);
                }
                rest = after;
            }
            done.push_str(rest);
            out = done;
        }
    }
    out
}
