//! `items(module)`: what a module declares, item by item, read from its syntax and no body
//! (handbook/design/next/incremental.md). An edit inside a body, or a line added above an
//! item, leaves the answer equal, so nothing that reads it is redone.
use crate::modules::Module;
use cpl_next_ir::ids::ItemId;
use cpl_next_syntax::ast;

/// An item, by its path, and the declaration it holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Head {
    pub id: ItemId,
    pub decls: Vec<Decl>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decl {
    pub kind: Kind,
    /// The name it declares in its module; `None` for an impl, a `use` and the like.
    pub name: Option<String>,
    pub public: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Fn,
    Struct,
    Enum,
    Trait,
    Impl,
    Alias,
    Const,
    Context,
    Domain,
    Use,
    Mod,
    /// A top-level statement: a `let`, an expression, an assignment.
    Stmt,
}

impl Head {
    /// The names its `pub` declarations export.
    pub fn exports(&self) -> impl Iterator<Item = &str> {
        (self.decls.iter())
            .filter(|d| d.public)
            .filter_map(|d| d.name.as_deref())
    }
}

pub fn items(m: &Module) -> Vec<Head> {
    (m.items.iter())
        .map(|it| Head {
            id: it.id.clone(),
            decls: vec![decl(&it.item)],
        })
        .collect()
}

fn decl(item: &ast::Item) -> Decl {
    use ast::Decl as D;
    let kind = match item.decl() {
        Some(D::Fn(_) | D::Extern(_)) => Kind::Fn,
        Some(D::Struct(_)) => Kind::Struct,
        Some(D::Enum(_)) => Kind::Enum,
        Some(D::Trait(_)) => Kind::Trait,
        Some(D::Impl(_)) => Kind::Impl,
        Some(D::TypeAlias(_)) => Kind::Alias,
        Some(D::Const(_)) => Kind::Const,
        Some(D::Context(_)) => Kind::Context,
        Some(D::Domain(_)) => Kind::Domain,
        Some(D::Use(_)) => Kind::Use,
        Some(D::Mod(_)) => Kind::Mod,
        None => Kind::Stmt,
    };
    Decl {
        kind,
        name: item.name().map(|t| t.text().to_string()),
        public: item.is_pub(),
    }
}
