//! An interactive problem run against the `@interactor` in its own file.
//!
//! `cpl run a.cp` runs the solution and the judge side by side: what one
//! prints is what the other reads. The input of `cpl run` is the hidden
//! test, which only the judge's `read()` sees. `cpl judge a.cp <input>`
//! is the judge alone, talking to a solution on its standard input and
//! output the way `cpl stress --interactor` expects.

use std::collections::VecDeque;
use std::io::Write;
use std::sync::{Arc, Condvar, Mutex};

/// The two directions of a conversation. A side that would wait for the
/// other while the other waits for it is told the input ended, so a
/// broken protocol is an error and never a hang.
#[derive(Default)]
struct Wire {
    state: Mutex<State>,
    changed: Condvar,
}

#[derive(Default)]
struct State {
    /// What each side has yet to read.
    inbox: [VecDeque<Vec<u8>>; 2],
    /// A side that will print nothing more.
    done: [bool; 2],
    waiting: [bool; 2],
}

/// One side's end of the wire: it reads `inbox[side]`.
struct End {
    wire: Arc<Wire>,
    side: usize,
}

impl End {
    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.wire.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// The next bytes the other side printed; `None` when there will be none.
    fn recv(&self) -> Option<Vec<u8>> {
        let other = 1 - self.side;
        let mut state = self.lock();
        loop {
            if let Some(chunk) = state.inbox[self.side].pop_front() {
                state.waiting[self.side] = false;
                return Some(chunk);
            }
            // The other side is done, or waits for this one with nothing
            // left to read.
            if state.done[other] || (state.waiting[other] && state.inbox[other].is_empty()) {
                state.waiting[self.side] = false;
                state.done[self.side] = true;
                self.wire.changed.notify_all();
                return None;
            }
            state.waiting[self.side] = true;
            self.wire.changed.notify_all();
            state = self
                .wire
                .changed
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    fn close(&self) {
        self.lock().done[self.side] = true;
        self.wire.changed.notify_all();
    }
}

/// What a side prints: it goes to the other side, and to `echo` too.
struct Speaker<'a> {
    end: &'a End,
    echo: Option<&'a mut (dyn Write + Send)>,
    lines: usize,
}

impl Write for Speaker<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if let Some(echo) = &mut self.echo {
            echo.write_all(bytes)?;
        }
        self.lines += bytes.iter().filter(|b| **b == b'\n').count();
        let other = 1 - self.end.side;
        self.end.lock().inbox[other].push_back(bytes.to_vec());
        self.end.wire.changed.notify_all();
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match &mut self.echo {
            Some(echo) => echo.flush(),
            None => Ok(()),
        }
    }
}

/// How a run against the interactor ended.
pub struct Ruling {
    /// Why the solution stopped, if it failed.
    pub solution: Option<cpl_interp::Trap>,
    /// Why the judge rejected the run, if it did.
    pub judge: Option<cpl_interp::Trap>,
    /// Lines the solution printed.
    pub lines: usize,
}

/// Runs `solution` against `judge` on the hidden test `input`. What the
/// solution prints is echoed to `stdout`.
pub fn run(
    solution: &cpl_ir::Program,
    judge: &cpl_ir::Program,
    input: &[u8],
    stdout: &mut (dyn Write + Send),
) -> Ruling {
    const SOLUTION: usize = 0;
    const JUDGE: usize = 1;
    let wire = Arc::new(Wire::default());
    let end = |side| End {
        wire: Arc::clone(&wire),
        side,
    };
    let (solution_end, judge_end) = (end(SOLUTION), end(JUDGE));
    std::thread::scope(|scope| {
        let judging = scope.spawn(|| {
            let mut says = Speaker {
                end: &judge_end,
                echo: None,
                lines: 0,
            };
            let io = cpl_interp::Io::judge(input, Box::new(|| judge_end.recv()));
            let result = cpl_interp::run_with(judge, io, &mut says);
            judge_end.close();
            result.err()
        });
        let mut says = Speaker {
            end: &solution_end,
            echo: Some(stdout),
            lines: 0,
        };
        let io = cpl_interp::Io::live(Box::new(|| solution_end.recv()));
        let result = cpl_interp::run_with(solution, io, &mut says);
        solution_end.close();
        let judge = judging.join().unwrap_or(None);
        Ruling {
            solution: result.err(),
            judge,
            lines: says.lines,
        }
    })
}

/// [`run`], recording the solution's side: its trace and what its
/// `debug(...)`s showed. A failure on either side is the run's.
pub fn run_traced(
    solution: &cpl_ir::Program,
    judge: &cpl_ir::Program,
    input: &[u8],
    stdout: &mut (dyn Write + Send),
    config: cpl_interp::trace::TraceConfig,
) -> cpl_interp::Observed {
    let wire = Arc::new(Wire::default());
    let end = |side| End {
        wire: Arc::clone(&wire),
        side,
    };
    let (solution_end, judge_end) = (end(0), end(1));
    std::thread::scope(|scope| {
        let judging = scope.spawn(|| {
            let mut says = Speaker {
                end: &judge_end,
                echo: None,
                lines: 0,
            };
            let io = cpl_interp::Io::judge(input, Box::new(|| judge_end.recv()));
            let result = cpl_interp::run_with(judge, io, &mut says);
            judge_end.close();
            result.err()
        });
        let mut says = Speaker {
            end: &solution_end,
            echo: Some(stdout),
            lines: 0,
        };
        let io = cpl_interp::Io::live(Box::new(|| solution_end.recv()));
        let mut observed = cpl_interp::run_observed(solution, io, &mut says, Some(config));
        solution_end.close();
        if let (None, Some(trap)) = (&observed.trap, judging.join().unwrap_or(None)) {
            observed.trap = Some(trap);
        }
        observed
    })
}

/// `cpl judge <file> <input>`: the file's `@interactor` as a judge
/// program. It reads the hidden test from `<input>`, talks to a solution
/// on standard input and output, and exits 0 to accept.
pub fn judge_alone(program: &cpl_ir::Program, input: &[u8]) -> Result<(), cpl_interp::Trap> {
    let heard = Box::new(|| {
        let mut line = Vec::new();
        match std::io::BufRead::read_until(&mut std::io::stdin().lock(), b'\n', &mut line) {
            Ok(n) if n > 0 => Some(line),
            _ => None,
        }
    });
    let mut stdout = std::io::stdout();
    cpl_interp::run_with(program, cpl_interp::Io::judge(input, heard), &mut stdout)
}
