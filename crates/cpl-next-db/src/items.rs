//! The item queries (handbook/design/next/incremental.md): a file's text is parsed once and cut
//! into its top-level items, each keyed by an `ItemId` (module path, name, and which item of
//! that name it is). `item(ItemId)` is the item's green node and the hash of its text, with no
//! offsets in either, so an edit to one item changes only that item's answer; where the item
//! sits in its file is `item_at(ItemId)`, the only answer a line added above it changes.
use crate::db::{Db, DbError, Input, Query};
use cpl_contract::{Definition, FileId};
use cpl_diag::Diagnostic;
use cpl_next_ir::ids::{ItemId, SourceId};
use cpl_next_syntax::ast::{self, AstNode};
use cpl_next_syntax::{GreenNode, SyntaxNode};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::Arc;

/// A file's text, as the snapshot holds it.
pub struct Source;
impl Input for Source {
    type Key = FileId;
    type Value = Arc<str>;
}

/// The libraries root file `root` reads beside std: each `[libs]` name of the `cp.toml` at or
/// above it and the directory it names, a nearer one first.
pub struct Libraries;
impl Input for Libraries {
    type Key = FileId;
    type Value = Arc<[(Arc<str>, PathBuf)]>;
}

/// The extension of a source file.
pub const SOURCE_EXT: &str = "cp";

/// `module_file(root, module)`: the file root file `root` reads library module `module` from:
/// `kit.m` is `m.cp` in the directory its `[libs]` call `kit`. Whether the file is there is
/// `Source`'s answer, not this one's.
pub struct ModuleFile;
impl Query for ModuleFile {
    type Key = (FileId, Arc<str>);
    type Value = Option<FileId>;
    const NAME: &'static str = "module_file";
    fn compute(db: &Db, (root, module): &Self::Key) -> Result<Self::Value, DbError> {
        let Some((lib, rest)) = module.split_once('.') else {
            return Ok(None);
        };
        let libs = db.input::<Libraries>(root)?.unwrap_or_default();
        Ok((libs.iter().find(|(n, _)| &**n == lib)).map(|(_, dir)| {
            let path = dir.join(format!("{}.{SOURCE_EXT}", rest.replace('.', "/")));
            FileId::new(&path.display().to_string())
        }))
    }
}

/// The name an item's id carries: what it declares, an operator fn's operator, an impl's head
/// (`impl Ord for Pair`), or none for a block of top-level statements.
fn item_name(item: &ast::Item) -> String {
    if let Some(t) = item.name() {
        return t.text().to_string();
    }
    let words = |n: &SyntaxNode| n.text().split_whitespace().collect::<Vec<_>>().join(" ");
    match item.decl() {
        Some(ast::Decl::Impl(d)) => match (d.trait_ref(), d.self_ty()) {
            (Some(t), Some(s)) => format!("impl {} for {}", words(t.syntax()), words(s.syntax())),
            (None, Some(s)) => format!("impl {}", words(s.syntax())),
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
    (file, module): &(FileId, Arc<str>),
    path: &str,
    nths: &mut HashMap<Arc<str>, u32>,
    out: &mut Vec<(ItemId, SyntaxNode)>,
) {
    for it in items {
        let name: Arc<str> = match path {
            "" => item_name(&it).into(),
            p => format!("{p}.{}", item_name(&it)).into(),
        };
        let nth = nths.entry(name.clone()).or_default();
        let id = ItemId {
            module: module.clone(),
            name: name.clone(),
            nth: *nth,
            file: file.clone(),
        };
        *nth += 1;
        out.push((id, it.syntax().clone()));
        if let Some(ast::Decl::Mod(m)) = it.decl() {
            flatten(m.items(), &(file.clone(), module.clone()), &name, nths, out);
        }
    }
}

/// A file's syntax: its green tree and what was wrong with its text, in file positions.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Parsed {
    pub green: GreenNode,
    pub diagnostics: Arc<[Diagnostic]>,
}

/// `parse(FileId)`: the file's green tree and its syntax errors.
pub struct Parse;
impl Query for Parse {
    type Key = FileId;
    type Value = Option<Parsed>;
    const NAME: &'static str = "parse";
    fn compute(db: &Db, file: &FileId) -> Result<Self::Value, DbError> {
        let text = db.input::<Source>(file)?;
        Ok(text.map(|t| {
            let file = cpl_next_syntax::parse(&t);
            Parsed {
                green: file.root.green().clone(),
                diagnostics: file.diagnostics.into(),
            }
        }))
    }
}

/// The items of file `of.0` read as module `of.1`, each with its id, in file order.
fn item_nodes(db: &Db, of: &(FileId, Arc<str>)) -> Result<Vec<(ItemId, SyntaxNode)>, DbError> {
    let Some(Parsed { green, .. }) = db.get::<Parse>(&of.0)? else {
        return Ok(Vec::new());
    };
    let root = SyntaxNode::new_root(green);
    let mut out = Vec::new();
    flatten(
        root.children().filter_map(ast::Item::cast),
        of,
        "",
        &mut HashMap::new(),
        &mut out,
    );
    Ok(out)
}

/// A file's items cut once: each one's id, green node and start, in file order, and where
/// each id is among them.
#[derive(Clone, Debug)]
pub struct FileNodes {
    nodes: Arc<[(ItemId, GreenNode, u32)]>,
    index: Arc<HashMap<ItemId, usize>>,
}

impl PartialEq for FileNodes {
    fn eq(&self, other: &Self) -> bool {
        self.nodes == other.nodes
    }
}
impl Eq for FileNodes {}

/// `item_nodes(FileId, module)`: the file's items, cut once, so `item` and `item_at` each find
/// one item by its id without cutting the file again.
pub struct ItemNodes;
impl Query for ItemNodes {
    type Key = (FileId, Arc<str>);
    type Value = FileNodes;
    const NAME: &'static str = "item_nodes";
    fn compute(db: &Db, of: &Self::Key) -> Result<Self::Value, DbError> {
        let nodes: Arc<[_]> = (item_nodes(db, of)?.into_iter())
            .map(|(id, n)| (id, n.green().clone(), n.span().start))
            .collect();
        let index = (nodes.iter().enumerate())
            .map(|(k, (id, ..))| (id.clone(), k))
            .collect();
        Ok(FileNodes {
            nodes,
            index: Arc::new(index),
        })
    }
}

/// `items(FileId, module)`: each item's id, in file order, of the file read as the module. One
/// file a session reads as two modules (a program, and a library another program uses) is
/// two modules' items.
pub struct Items;
impl Query for Items {
    type Key = (FileId, Arc<str>);
    type Value = Vec<ItemId>;
    const NAME: &'static str = "items";
    fn compute(db: &Db, of: &Self::Key) -> Result<Self::Value, DbError> {
        Ok((db.get::<ItemNodes>(of)?.nodes.iter())
            .map(|(id, ..)| id.clone())
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

/// `id`'s node and start, if its file holds it.
fn node(db: &Db, id: &ItemId) -> Result<Option<(GreenNode, u32)>, DbError> {
    let file = db.get::<ItemNodes>(&(id.file.clone(), id.module.clone()))?;
    let found = file.index.get(id).map(|&k| &file.nodes[k]);
    Ok(found.map(|(_, n, at)| (n.clone(), *at)))
}

/// `item(ItemId)`: the item's green node and its text's hash.
pub struct ItemOf;
impl Query for ItemOf {
    type Key = ItemId;
    type Value = Option<Item>;
    const NAME: &'static str = "item";
    fn compute(db: &Db, id: &ItemId) -> Result<Self::Value, DbError> {
        Ok(node(db, id)?.map(|(green, _)| {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            SyntaxNode::new_root(green.clone()).text().hash(&mut h);
            Item {
                green,
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
        Ok(node(db, id)?.map(|(_, at)| at))
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
        let mut span = site.span;
        span.start += at;
        span.end += at;
        Ok(Some(Definition {
            path: site.item.file.path().into(),
            span,
        }))
    }
}

#[cfg(test)]
mod tests;
