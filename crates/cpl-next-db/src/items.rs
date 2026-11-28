//! The item queries (handbook/design/next/incremental.md): a file's text is parsed once and cut
//! into its top-level items, each keyed by an `ItemId` (module path, name, and which item of
//! that name it is). `item(ItemId)` is the item's green node and the hash of its text, with no
//! offsets in either, so an edit to one item changes only that item's answer; where the item
//! sits in its file is `item_at(ItemId)`, the only answer a line added above it changes.
use crate::db::{Db, DbError, Input, Query};
use cpl_contract::{Definition, FileId};
use cpl_next_ir::ids::{ItemId, SourceId};
use cpl_next_syntax::ast::{self, AstNode};
use cpl_next_syntax::{GreenNode, SyntaxNode};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// A file's text, as the snapshot holds it.
pub struct Source;
impl Input for Source {
    type Key = FileId;
    type Value = Arc<str>;
}

/// The module path the loader gives a file.
pub struct ModulePath;
impl Input for ModulePath {
    type Key = FileId;
    type Value = Arc<str>;
}

/// The file a module path names.
pub struct ModuleFile;
impl Input for ModuleFile {
    type Key = Arc<str>;
    type Value = FileId;
}

/// The name an item's id carries: what it declares, an operator fn's operator, an impl's head
/// (`impl Ord for Pair`), or none for a block of top-level statements.
fn item_name(item: &ast::Item) -> String {
    if let Some(t) = item.name() {
        return t.text().to_string();
    }
    let words = |n: SyntaxNode| n.text().split_whitespace().collect::<Vec<_>>().join(" ");
    match item.decl() {
        Some(ast::Decl::Impl(d)) => match (d.trait_ref(), d.self_ty()) {
            (Some(t), Some(s)) => format!("impl {} for {}", words(t), words(s)),
            (None, Some(s)) => format!("impl {}", words(s)),
            _ => "impl".to_string(),
        },
        Some(ast::Decl::Fn(f)) => {
            (f.op_name().and_then(|o| o.op())).map_or_else(String::new, |t| t.text().to_string())
        }
        _ => String::new(),
    }
}

/// Each item of `items` with its id, and after an inline `mod m:` the items of its body, named
/// `m.name`, as their own items of the file. Items of one name are told apart by how many of
/// that name come before them.
fn flatten(
    items: impl Iterator<Item = ast::Item>,
    module: &Arc<str>,
    path: &str,
    out: &mut Vec<(ItemId, SyntaxNode)>,
) {
    for it in items {
        let name: Arc<str> = match path {
            "" => item_name(&it).into(),
            p => format!("{p}.{}", item_name(&it)).into(),
        };
        let nth = out.iter().filter(|(i, _)| i.name == name).count() as u32;
        let id = ItemId {
            module: module.clone(),
            name: name.clone(),
            nth,
        };
        out.push((id, it.syntax().clone()));
        if let Some(ast::Decl::Mod(m)) = it.decl() {
            flatten(m.items(), module, &name, out);
        }
    }
}

/// `parse(FileId)`: the file's green tree.
pub struct Parse;
impl Query for Parse {
    type Key = FileId;
    type Value = Option<GreenNode>;
    const NAME: &'static str = "parse";
    fn compute(db: &Db, file: &FileId) -> Result<Self::Value, DbError> {
        let text = db.input::<Source>(file)?;
        Ok(text.map(|t| cpl_next_syntax::parse(&t).root.green().clone()))
    }
}

/// The items of `file`, each with its id, in file order.
fn item_nodes(db: &Db, file: &FileId) -> Result<Vec<(ItemId, SyntaxNode)>, DbError> {
    let Some(green) = db.get::<Parse>(file)? else {
        return Ok(Vec::new());
    };
    let module = db
        .input::<ModulePath>(file)?
        .unwrap_or_else(|| file.path().into());
    let root = SyntaxNode::new_root(green);
    let mut out = Vec::new();
    flatten(
        root.children().filter_map(ast::Item::cast),
        &module,
        "",
        &mut out,
    );
    Ok(out)
}

/// `items(FileId)`: each item's id, in file order.
pub struct Items;
impl Query for Items {
    type Key = FileId;
    type Value = Vec<ItemId>;
    const NAME: &'static str = "items";
    fn compute(db: &Db, file: &FileId) -> Result<Self::Value, DbError> {
        Ok(item_nodes(db, file)?
            .into_iter()
            .map(|(id, _)| id)
            .collect())
    }
}

/// One item's syntax, offset-free.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Item {
    pub green: GreenNode,
    /// The hash of the item's text: what a stored answer for it is keyed by.
    pub hash: u64,
}

/// `id`'s node, if its module's file holds it. Reads `items` first, so an edit that leaves
/// the file's ids as they were reaches only the items whose text changed.
fn node(db: &Db, id: &ItemId) -> Result<Option<SyntaxNode>, DbError> {
    let Some(file) = db.input::<ModuleFile>(&id.module)? else {
        return Ok(None);
    };
    if !db.get::<Items>(&file)?.contains(id) {
        return Ok(None);
    }
    Ok(item_nodes(db, &file)?
        .into_iter()
        .find(|(i, _)| i == id)
        .map(|(_, n)| n))
}

/// `item(ItemId)`: the item's green node and its text's hash.
pub struct ItemOf;
impl Query for ItemOf {
    type Key = ItemId;
    type Value = Option<Item>;
    const NAME: &'static str = "item";
    fn compute(db: &Db, id: &ItemId) -> Result<Self::Value, DbError> {
        Ok(node(db, id)?.map(|n| {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            n.text().hash(&mut h);
            Item {
                green: n.green().clone(),
                hash: h.finish(),
            }
        }))
    }
}

/// `item_at(ItemId)`: the byte the item starts at in its file, which the item's spans count
/// from.
pub struct ItemAt;
impl Query for ItemAt {
    type Key = ItemId;
    type Value = Option<u32>;
    const NAME: &'static str = "item_at";
    fn compute(db: &Db, id: &ItemId) -> Result<Self::Value, DbError> {
        Ok(node(db, id)?.map(|n| n.span().start))
    }
}

/// `source_of(SourceId)`: where a site is in its file, the site's span counted from its item's
/// start. The only answer that turns a site into a file position, so a line added above the
/// item moves this and nothing the site's events say.
pub struct SourceOf;
impl Query for SourceOf {
    type Key = SourceId;
    type Value = Option<Definition>;
    const NAME: &'static str = "source_of";
    fn compute(db: &Db, site: &SourceId) -> Result<Self::Value, DbError> {
        let Some(at) = db.get::<ItemAt>(&site.item)? else {
            return Ok(None);
        };
        let Some(file) = db.input::<ModuleFile>(&site.item.module)? else {
            return Ok(None);
        };
        let span = cpl_diag::Span {
            start: at + site.span.start,
            end: at + site.span.end,
        };
        Ok(Some(Definition {
            path: file.path().into(),
            span,
        }))
    }
}

#[cfg(test)]
mod tests;
