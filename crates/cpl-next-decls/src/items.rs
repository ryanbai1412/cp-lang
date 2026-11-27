//! `items(module)`: what a module declares, item by item, read from its syntax and no body
//! (handbook/design/next/incremental.md). An edit inside a body, or a line added above an
//! item, leaves the answer equal, so nothing that reads it is redone.
use crate::modules::Module;
use cpl_next_ir::ids::ItemId;
use cpl_syntax::ast::{Stmt, StmtKind};

/// An item, by its path, and each declaration it holds: lines a blank line does not part are
/// one item, so `fn floor`, `fn ceil` and `fn round` on three lines are one.
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
    Law,
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
            decls: it.stmts.iter().map(decl).collect(),
        })
        .collect()
}

fn decl(s: &Stmt) -> Decl {
    let kind = match &s.kind {
        StmtKind::Fn(_) => Kind::Fn,
        StmtKind::Struct(_) => Kind::Struct,
        StmtKind::Enum(_) => Kind::Enum,
        StmtKind::Trait { .. } => Kind::Trait,
        StmtKind::Impl { .. } => Kind::Impl,
        StmtKind::TypeAlias { .. } => Kind::Alias,
        StmtKind::Const { .. } => Kind::Const,
        StmtKind::Context { .. } => Kind::Context,
        StmtKind::Domain { .. } => Kind::Domain,
        StmtKind::Law { .. } => Kind::Law,
        StmtKind::Use { .. } => Kind::Use,
        StmtKind::Mod { .. } => Kind::Mod,
        _ => Kind::Stmt,
    };
    Decl {
        kind,
        name: crate::decls::item_name(s).map(str::to_string),
        public: s.public,
    }
}
