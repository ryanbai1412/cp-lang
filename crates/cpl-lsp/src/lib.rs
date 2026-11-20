//! The cp language server (`cpl lsp`).
//!
//! It speaks the Language Server Protocol over stdio and answers from the
//! same incremental [`cpl_check::Database`] as `cpl check`, `cpl run` and
//! `cpl build`. Today it publishes diagnostics as you type and serves
//! hover, go-to-definition, the document outline, and quick fixes.
//! Completion, semantic tokens and formatting come with later phases
//! (`plan/60-editor.md`).

pub mod json;
mod server;
mod transport;

pub use server::Server;
pub use transport::{read_message, serve, write_message};

/// Name of this crate.
pub const NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests;
