//! The incremental front end of the cp language.
//!
//! [`Session`] is the one front end behind `cpl check`, `cpl run`,
//! `cpl build` and `cpl lsp`. Give it file texts with
//! [`Session::set_file`] and ask for [`Session::analysis`],
//! [`Session::facts`], [`Database::hover`], [`Database::definition`] or
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
mod facts;
mod fx;
mod hover;
mod labels;
mod modules;
mod nav;
mod prelude;
mod session;
mod std_files;
mod ty;

pub use builtins::{
    builtin_member, builtin_members, char_literal, foreign_iter_call, foreign_member,
    foreign_str_fn, list_end_help, prelude_names, split_params as split_builtin_params,
    BuiltinMember, BuiltinType, MemberKind, PreludeKind, Receiver, BUILTIN_MEMBERS, BUILTIN_TYPES,
    ITERATOR_POSITION_HELP,
};
pub use check::{closest, GlobalRef, HoverEntry, Resolved, TypedIndex};
pub use db::{
    Analysis, AnalyzedItem, Database, Decl, Import, ItemCheck, ItemSyntax, Module, Stats,
};
pub use def::{DeclKind, Def, DomainInfo};
pub use facts::{CallFact, Callee, CalleeKind, ExprFacts, Facts};
pub use hover::{Definition, Hover, Representation, Symbol};
pub use modules::{
    bundled_std, carry_only_embedded_std, configured_libraries, dev_std_problem, file_path,
    home_dir, provide_std, std_calls, std_from_disk, std_needed, std_needed_with, std_outline,
    take_std_needs, FsLoader, Loader,
};
pub use prelude::{
    import_edit, method_key, method_module, methods as std_methods, set_domain, std_module,
    std_names, std_self_heads, SetDomain,
};
pub use session::{open, Libraries, Revision, Session, SessionOptions, StdSource};
pub use ty::{human_bytes, Param, Ty};

/// A `<fn>_of_pairs`/`<fn>_of_triples` fn: what `T.fn(list)` resolves to, never a name
/// anyone types, so completion and the editor tables leave it out.
pub fn is_list_form_hook(name: &str) -> bool {
    name.ends_with("_of_pairs") || name.ends_with("_of_triples")
}

/// Name of this crate.
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Every std module this process checks against, by name (`"seq"`), with its text: the
/// ones `CPL_STD_DIR` (or a dev build's own std/) had when first read, else those built in.
pub fn loaded_std() -> &'static [(&'static str, &'static str)] {
    modules::modules()
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_messages;
#[cfg(test)]
mod tests_mutation;
