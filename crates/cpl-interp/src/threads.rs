//! Where a run runs besides a thread of its own: on the caller's thread,
//! or on one it gives up on after a time limit.

use crate::{run, run_io, Io, Limits, Trap};
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

/// [`run`] that gives up after `limit`: `None` when the program was still
/// running. The run goes on, left behind on a thread of its own, so only a
/// process that ends soon after should give up on one.
pub fn run_for(
    program: &Program,
    input: &[u8],
    limit: std::time::Duration,
) -> Option<(Vec<u8>, Result<(), Trap>)> {
    let (program, input) = (program.clone(), input.to_vec());
    let (done, ran) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut out = Vec::new();
        let result = run(&program, &input, &mut out);
        let _ = done.send((out, result));
    });
    ran.recv_timeout(limit).ok()
}
