//! `cpl stress`: random inputs against a brute, shrunk to the smallest
//! case that still fails (`plan/40-tooling.md`, sections 7b and 8).

pub mod args;
pub mod calls;
pub mod case;
pub mod choices;
pub mod cli;
pub mod duplex;
pub mod engine;
pub mod explain;
pub mod gen;
pub mod judge;
pub mod locate;
pub mod program;
pub mod report;
pub mod rng;
pub mod run;
pub mod shrink;
pub mod spec;
pub mod traced;

pub use case::{judge_case, Failed, Ran, Ruling, Side, Sides, Talked, Transcript, Wrong};
pub use engine::{
    stress_source, ChoiceSource, Engine, InterpChoices, InterpEngine, ProcessChoices, ProcessEngine,
};
pub use gen::{generate, make, Case, Pick, Size, TreeShape, Value};
pub use judge::{Judge, Judged, Launch, Verdict};
pub use program::{missing_compiler, BuildError, Exit, Outcome, Program};
pub use report::StressResult;
pub use rng::Rng;
pub use shrink::{fit, shrink};
pub use spec::{InputSpec, Shape, SpecError};
pub use traced::STUCK_STEPS;

/// Name of this crate.
pub const NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests;
