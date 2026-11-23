//! Where a run runs besides a thread of its own: on the caller's thread,
//! or on one it gives up on after a time limit.

use crate::{run_io, run_with, Io, Limits, Stop, Trap, OUT_OF_TIME};
use cpl_ir::Program;
use std::io::Write;

/// [`run`] on the calling thread, for a host that cannot start one. The
/// caller says how much of its stack the program's calls may take, since
/// only it knows how much there is.
pub fn run_on_this_thread(
    program: &Program,
    input: &[u8],
    output: &mut (dyn Write + Send),
    limits: Limits,
) -> Result<(), Trap> {
    match run_io(program, Io::fixed(input), output, limits, None, false).trap {
        Some(trap) => Err(trap),
        None => Ok(()),
    }
}

/// [`run_with`] on the calling thread, for a host that cannot start one.
pub fn run_with_on_this_thread(
    program: &Program,
    io: Io,
    output: &mut (dyn Write + Send),
    limits: Limits,
) -> Result<(), Trap> {
    match run_io(program, io, output, limits, None, false).trap {
        Some(trap) => Err(trap),
        None => Ok(()),
    }
}

/// [`crate::run`] ended after `limit`: `None` when the program was still
/// running then. The run stops itself (see [`Stop`]), so nothing is left
/// behind.
pub fn run_for(
    program: &Program,
    input: &[u8],
    limit: std::time::Duration,
) -> Option<(Vec<u8>, Result<(), Trap>)> {
    let mut out = Vec::new();
    let io = Io::fixed(input).until(Stop::after(limit));
    match run_with(program, io, &mut out) {
        Err(trap) if trap.message == OUT_OF_TIME => None,
        result => Some((out, result)),
    }
}
