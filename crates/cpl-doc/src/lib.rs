//! `cpl doc`: documentation from `///` comments (`plan/45-dev-tools.md`).
//!
//! ```text
//! /// The sum of `a[l..r]` in O(1), after O(n) to build.
//! ///
//! /// ```
//! /// let p = Prefix([3, 1, 4])
//! /// print(p.sum(0, 2))
//! /// ```
//! @cost("O(1)")
//! fn sum(self, l: Int, r: Int) -> Int
//! ```
//!
//! [`extract`] reads one source file into a [`Module`]: every declaration
//! with its signature, its doc comment, its `@cost`, the line it is on and
//! the examples in its comment. [`html::render`] turns modules into a site,
//! [`json::render`] into data for another generator, and [`examples::check`]
//! is what `cpl doc --test` runs.

mod highlight;
mod markdown;
mod model;

pub mod cli;
pub mod examples;
pub mod html;
pub mod json;

pub use model::{extract, Example, Item, Kind, Module};

/// Name of this crate.
pub const NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests;
