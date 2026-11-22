//! `cpl stress`: random inputs against a brute, shrunk to the smallest
//! case that still fails (`plan/40-tooling.md`, sections 7b and 8).

pub mod args;
pub mod choices;
pub mod cli;
pub mod engine;
pub mod explain;
pub mod gen;
pub mod judge;
pub mod program;
pub mod rng;
pub mod run;
pub mod shrink;
pub mod spec;

pub use engine::{
    stress_source, ChoiceSource, Engine, InterpChoices, InterpEngine, ProcessChoices,
    ProcessEngine, SourceStressResult, Wrong,
};
pub use gen::{generate, make, Case, Pick, Size, TreeShape, Value};
pub use judge::{Judge, Judged, Verdict};
pub use program::{BuildError, Exit, Outcome, Program};
pub use rng::Rng;
pub use shrink::{fit, shrink};
pub use spec::{InputSpec, Shape, SpecError};

/// Name of this crate.
pub const NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests;
