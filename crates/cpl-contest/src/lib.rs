//! The contest workflow around one problem file: `cpl new`, `cpl test`,
//! `cpl bench` and `cpl watch` (`plan/40-tooling.md`).

mod args;
pub mod bench;
pub mod corpus_bench;
pub mod new;
pub mod problem;
pub mod test;
pub mod watch;
pub mod zero_cost;

/// Name of this crate.
pub const NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests;
