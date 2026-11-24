//! The cp language server (`cpl lsp`).
//!
//! It speaks the Language Server Protocol over stdio and answers from the
//! same incremental [`cpl_check::Session`] as `cpl check`, `cpl run` and
//! `cpl build`. It publishes diagnostics as you type and serves hover,
//! go-to-definition, the document outline, quick fixes, formatting,
//! type-aware completion (`xs.`, names in scope, `use std.`), signature
//! help, inlay hints, find-references and rename. Semantic tokens come with a later phase
//! (`plan/60-editor.md`).

mod complete;
pub mod explain;
pub mod fix;
mod imports;
mod outline;
mod rename;
mod server;
pub mod syntax_hover;
pub mod transport;
mod uri;

pub use server::Server;
pub use transport::{read_message, serve, write_message};

/// Name of this crate.
pub const NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests;
