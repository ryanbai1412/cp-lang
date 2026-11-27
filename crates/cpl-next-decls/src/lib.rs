//! The new pipeline's first layer (contract 2): the modules a program loads, placed in one
//! offset space, and their declarations: what each name means, without looking inside a
//! body. Every later layer reads what it gives; it reads none of them.

use cpl_next_ir::ty;
mod cycles;
pub mod decls;
pub mod diag;
pub mod modules;
pub mod plain;
pub mod records;
mod synth;

/// The name of a syntax node's kind, for saying which one is not implemented.
pub fn kind_name(kind: &impl std::fmt::Debug) -> String {
    let text = format!("{kind:?}");
    let end = text
        .find(|c: char| !c.is_alphanumeric())
        .unwrap_or(text.len());
    text[..end].to_string()
}
