//! The item queries (handbook/design/next/incremental.md): a file's text is parsed once and cut
//! into its top-level items, each keyed by an `ItemId` (module path, name, and which item of
//! that name it is). `item(ItemId)` is the item's green node and the hash of its text, with no
//! offsets in either, so an edit to one item changes only that item's answer; where the item
//! sits in its file is `item_at(ItemId)`, the only answer a line added above it changes.
use crate::db::{Db, DbError, Input, Query};
use cpl_contract::FileId;
use cpl_next_ir::ids::ItemId;
use cpl_next_syntax::{GreenNode, NodeKind, SyntaxNode};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
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

/// How an item's declared name is read from its node: the syntax tree's `ast::Item::name()`.
/// An item that declares none (a block of top-level statements) has none.
pub trait Names: Send + Sync + 'static {
    fn name(item: &SyntaxNode) -> Option<String>;
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

/// The top-level item nodes of `file`, in order.
fn item_nodes(db: &Db, file: &FileId) -> Result<Vec<SyntaxNode>, DbError> {
    let Some(green) = db.get::<Parse>(file)? else {
        return Ok(Vec::new());
    };
    let root = SyntaxNode::new_root(green);
    Ok(root
        .children()
        .filter(|n| n.kind() == NodeKind::Item)
        .collect())
}

/// `items(FileId)`: each top-level item's id, in file order. Items of one name are told apart
/// by how many of that name come before them.
pub struct Items<N>(PhantomData<N>);
impl<N: Names> Query for Items<N> {
    type Key = FileId;
    type Value = Vec<ItemId>;
    const NAME: &'static str = "items";
    fn compute(db: &Db, file: &FileId) -> Result<Self::Value, DbError> {
        let module = db
            .input::<ModulePath>(file)?
            .unwrap_or_else(|| file.path().into());
        let mut ids: Vec<ItemId> = Vec::new();
        for n in item_nodes(db, file)? {
            let name: Arc<str> = N::name(&n).unwrap_or_default().into();
            let nth = ids.iter().filter(|i| i.name == name).count() as u32;
            ids.push(ItemId {
                module: module.clone(),
                name,
                nth,
            });
        }
        Ok(ids)
    }
}

/// One item's syntax, offset-free.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Item {
    pub green: GreenNode,
    /// The hash of the item's text: what a stored answer for it is keyed by.
    pub hash: u64,
}

/// `id`'s node, if its module's file holds it.
fn node<N: Names>(db: &Db, id: &ItemId) -> Result<Option<SyntaxNode>, DbError> {
    let Some(file) = db.input::<ModuleFile>(&id.module)? else {
        return Ok(None);
    };
    let at = db.get::<Items<N>>(&file)?.iter().position(|i| i == id);
    Ok(at.and_then(|k| item_nodes(db, &file).ok()?.into_iter().nth(k)))
}

/// `item(ItemId)`: the item's green node and its text's hash.
pub struct ItemOf<N>(PhantomData<N>);
impl<N: Names> Query for ItemOf<N> {
    type Key = ItemId;
    type Value = Option<Item>;
    const NAME: &'static str = "item";
    fn compute(db: &Db, id: &ItemId) -> Result<Self::Value, DbError> {
        Ok(node::<N>(db, id)?.map(|n| {
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
pub struct ItemAt<N>(PhantomData<N>);
impl<N: Names> Query for ItemAt<N> {
    type Key = ItemId;
    type Value = Option<u32>;
    const NAME: &'static str = "item_at";
    fn compute(db: &Db, id: &ItemId) -> Result<Self::Value, DbError> {
        Ok(node::<N>(db, id)?.map(|n| n.span().start))
    }
}

#[cfg(test)]
mod tests;
