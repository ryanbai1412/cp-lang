//! The incremental front end of the cp language.
//!
//! [`Database`] is the one front end behind `cpl check`, `cpl run`,
//! `cpl build` and `cpl lsp`. Give it file texts with
//! [`Database::set_file`] and ask for [`Database::analyze`],
//! [`Database::hover`], [`Database::definition`] or
//! [`Database::symbols`]; it re-does only the work an edit invalidated.
//! See `db.rs` for how.
//!
//! What the checker covers today: name resolution, types of bindings, fns,
//! structs, enums and domains, strict domain indexing, call arity and
//! argument types, `mut` at declarations and call sites, and assignment to
//! immutable bindings. Borrows, value ranges and overflow, generics and
//! traits are later phases; where the checker does not know, it says
//! nothing.

mod builtins;
mod check;
mod db;
mod def;
mod hover;
mod modules;
mod prelude;
mod ty;

pub use builtins::{builtin_members, prelude_names, BuiltinMember, MemberKind, PreludeKind, Receiver, BUILTIN_MEMBERS};
pub use check::{GlobalRef, HoverEntry};
pub use db::{Analysis, AnalyzedItem, Database, Decl, Import, ItemCheck, ItemSyntax, Module, Stats};
pub use def::{DeclKind, Def, DomainInfo};
pub use hover::{Definition, Hover, Symbol};
pub use modules::{FsLoader, Loader};
pub use ty::{Param, Ty};

/// Name of this crate.
pub const NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests;
