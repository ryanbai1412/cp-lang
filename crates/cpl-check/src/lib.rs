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
mod fx;
mod hover;
mod modules;
mod nav;
mod prelude;
mod std_files;
mod ty;

pub use builtins::{
    builtin_member, builtin_members, char_literal, foreign_iter_call, foreign_member,
    foreign_str_fn, prelude_names, BuiltinMember, MemberKind, PreludeKind, Receiver,
    BUILTIN_MEMBERS,
};
pub use check::{closest, GlobalRef, HoverEntry, TypedIndex};
pub use db::{
    Analysis, AnalyzedItem, Database, Decl, Import, ItemCheck, ItemSyntax, Module, Stats,
};
pub use def::{DeclKind, Def, DomainInfo};
pub use hover::{Definition, Hover, Representation, Symbol};
pub use modules::{bundled_std, dev_std_problem, file_path, home_dir, FsLoader, Loader};
pub use prelude::{import_edit, methods as std_methods, std_method, std_module, std_names};
pub use ty::{human_bytes, Param, Ty};

/// A `<fn>_of_pairs`/`<fn>_of_triples` fn: what `T.fn(list)` resolves to, never a name
/// anyone types, so completion and the editor tables leave it out.
pub fn is_list_form_hook(name: &str) -> bool {
    name.ends_with("_of_pairs") || name.ends_with("_of_triples")
}

/// Name of this crate.
pub const NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_messages;
#[cfg(test)]
mod tests_mutation;
