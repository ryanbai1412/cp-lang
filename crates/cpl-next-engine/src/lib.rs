//! Contract 7's fast engine: canonical IR compiled to code over registers and flat memory.
//! A value is a run of 64-bit words laid out by its type (`layout`); a frame is a window of
//! one stack of words; storage is a counted heap object, shared until written, as the
//! reference interpreter's is. Operations mean what `ops` says, so the engine, the reference
//! interpreter and the C++ agree by construction; its output, events and trap are the
//! reference interpreter's, which `tests/programs.rs` checks on every case.
mod compile;
mod exec;
use cpl_next_ir::layout;
mod peep;

use cpl_next_ir::ir::Func;
use cpl_next_ir::prog::{CTy, Program};
pub use cpl_next_ir::run::{Io, Limits, Outcome, RunInput, Trap};

/// Runs canonical IR on `input`, under its limits.
pub fn run(p: &Program, funcs: &[Func<CTy>], input: RunInput) -> Outcome {
    let code = compile::compile(p, funcs);
    exec::run(&code, input)
}

/// Runs canonical IR on `input`, then releases its globals if it ended: what it did, and how
/// many storage objects are still held, which is 0 when the drops ownership placed (contract
/// 07) release every value it made.
pub fn run_held(p: &Program, funcs: &[Func<CTy>], input: RunInput) -> (Outcome, usize) {
    let code = compile::compile(p, funcs);
    exec::run_held(&code, input)
}

/// How running a program for a value stopped short of it.
pub enum Stop {
    Trap(Trap),
    /// It took more steps than it was given.
    Budget,
    /// It reads the input, so its value is decided at run time.
    Input,
}

/// Runs canonical IR with no input for at most `steps` steps, each fn entered and each jump
/// taken back being one: the words each global of `gs` holds when it ends. Compile-time evaluation
/// runs here, so a constant means what running the program would make it.
pub fn eval(
    p: &Program,
    funcs: &[Func<CTy>],
    gs: &[usize],
    steps: u64,
) -> Result<Vec<Vec<u64>>, Stop> {
    let code = compile::compile(p, funcs);
    exec::eval(&code, gs, steps)
}
