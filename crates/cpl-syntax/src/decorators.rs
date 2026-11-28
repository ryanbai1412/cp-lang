//! The closed set of decorators (spec/core/06-control-functions.md, 6.4):
//! the parser rejects any other `@name`, so no later pass meets one. The
//! table is `cpl_next_syntax::grammar::Decorator`, shared with cpl-next.

pub use cpl_next_syntax::grammar::{Decorator as Mark, DECORATORS, DIRECTIVES};

/// Old spellings the parser or checker answers with their own E0503 fix.
pub use cpl_next_syntax::grammar::REMOVED_DECORATORS as REMOVED;

/// Whether `@name` is a decorator, live or removed with its own fix.
pub fn is_known(name: &str) -> bool {
    Mark::of(name).is_some()
}

/// The marker on the struct whose values are a domain's cuts (`D.cuts`).
pub const CUTS: &str = "cuts";
