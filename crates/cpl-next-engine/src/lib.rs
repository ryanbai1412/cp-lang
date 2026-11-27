//! Contract 7's fast engine: canonical IR compiled to code over registers and flat memory.
//! A value is a run of 64-bit words laid out by its type (`layout`); a frame is a window of
//! one stack of words; storage is a counted heap object, shared until written, as the
//! reference interpreter's is. Operations mean what `ops` says, so the engine, the reference
//! interpreter and the C++ agree by construction; its output, events and trap are the
//! reference interpreter's, which `tests/programs.rs` checks on every case.
mod compile;
mod exec;
mod layout;
mod peep;

use cpl_next_ir::ir::Func;
use cpl_next_ir::prog::{CTy, Program};
pub use cpl_next_ir::run::Outcome;

/// Runs canonical IR on `input`.
pub fn run(p: &Program, funcs: &[Func<CTy>], input: &str) -> Outcome {
    let code = compile::compile(p, funcs);
    exec::run(&code, funcs, input)
}
