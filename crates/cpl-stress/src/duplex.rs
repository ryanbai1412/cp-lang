//! An interactive case in-process, on one thread: the solution against the
//! file's `@interactor`, as the browser runs it. The interactor runs once.
//! Each time it waits on the solution with nothing left to hear, the
//! solution runs again from its start on all the interactor has said, up
//! to where it waits for more. Both are deterministic, so each run repeats
//! the last one's output and goes further.

use crate::case::{Side, Transcript};
use cpl_interp::{Io, Limits, Stop, Trap};
use std::io::Write;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

/// How an interactive run went.
pub struct Spoken {
    /// The solution's output, and its trap if it stopped at one.
    pub solution: Result<String, (String, Trap)>,
    /// The interactor's trap: it rejected the run, or failed itself.
    pub interactor: Option<Trap>,
    pub transcript: Transcript,
    /// Everything the interactor sent: the solution's whole input.
    pub sent: Vec<u8>,
}

/// What the two sides have said so far.
#[derive(Default)]
struct Wire {
    /// Everything the interactor printed: the solution's input.
    sent: Vec<u8>,
    /// How much of the solution's output the interactor has heard.
    heard: Vec<u8>,
    transcript: Transcript,
    /// Each side's last line, until it ends.
    partial: [Vec<u8>; 2],
}

impl Wire {
    fn say(&mut self, side: Side, bytes: &[u8]) {
        let partial = &mut self.partial[side as usize];
        partial.extend_from_slice(bytes);
        if let Some(end) = partial.iter().rposition(|b| *b == b'\n') {
            let lines: Vec<u8> = partial.drain(..=end).collect();
            self.transcript.say(side, &String::from_utf8_lossy(&lines));
        }
    }

    fn finish(mut self) -> Transcript {
        for side in [Side::Solution, Side::Interactor] {
            let rest = std::mem::take(&mut self.partial[side as usize]);
            self.transcript.say(side, &String::from_utf8_lossy(&rest));
        }
        self.transcript
    }
}

#[derive(Default)]
struct Shared<T>(Arc<Mutex<T>>);

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Shared(Arc::clone(&self.0))
    }
}

impl<T> Shared<T> {
    fn lock(&self) -> MutexGuard<'_, T> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Write for Shared<Wire> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let mut wire = self.lock();
        wire.sent.extend_from_slice(bytes);
        wire.say(Side::Interactor, bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Write for Shared<Vec<u8>> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.lock().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Runs `program` on `io`: on this thread with `stack` bytes of it, or on a
/// thread of its own.
fn run(
    program: &cpl_ir::Program,
    io: Io,
    out: &mut (dyn Write + Send),
    stack: Option<u32>,
) -> Result<(), Trap> {
    match stack {
        Some(bytes) => cpl_interp::run_with_on_this_thread(
            program,
            io,
            out,
            Limits {
                stack_bytes: bytes as usize,
            },
        ),
        None => cpl_interp::run_with(program, io, out),
    }
}

/// How one run of the solution on `sent` went.
struct Heard {
    /// What it printed before it waited for more than `sent`.
    out: Vec<u8>,
    /// `None` while it still waits for more; else how it ended.
    ended: Option<Result<(), Trap>>,
}

/// The solution from its start on `sent`. Unless `last`, it is stopped when
/// it waits for more: the interactor has more to say.
fn solution_on(
    solution: &cpl_ir::Program,
    sent: &[u8],
    last: bool,
    limit: Duration,
    stack: Option<u32>,
) -> Heard {
    let out = Shared::<Vec<u8>>::default();
    let waited = Shared::<Option<usize>>::default();
    let stop = Stop::after(limit);
    let mut given = false;
    let more = {
        let (out, waited, stop) = (out.clone(), waited.clone(), stop.clone());
        let sent = sent.to_vec();
        Box::new(move || {
            if !given && !sent.is_empty() {
                given = true;
                return Some(sent.clone());
            }
            if !last {
                waited.lock().get_or_insert(out.lock().len());
                stop.cancel();
            }
            None
        })
    };
    let result = run(
        solution,
        Io::live(more).until(stop),
        &mut out.clone(),
        stack,
    );
    let mut out = std::mem::take(&mut *out.lock());
    let waited = *waited.lock();
    match waited {
        Some(at) => {
            out.truncate(at);
            Heard { out, ended: None }
        }
        None => Heard {
            out,
            ended: Some(result),
        },
    }
}

#[cfg(test)]
thread_local! {
    /// How many times [`talk`] ran an interactor on this thread.
    pub(crate) static INTERACTOR_RUNS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// The solution against `interactor` on the hidden test `input`. The
/// solution has `limit` for each of its runs; the interactor has
/// [`crate::case::judge_limit`]. With `stack`, both run on this thread,
/// each with half of it.
pub fn talk(
    solution: &cpl_ir::Program,
    interactor: &cpl_ir::Program,
    input: &[u8],
    limit: Duration,
    stack: Option<u32>,
) -> Spoken {
    #[cfg(test)]
    INTERACTOR_RUNS.with(|runs| runs.set(runs.get() + 1));
    let stack = stack.map(|bytes| bytes / 2);
    let wire = Shared::<Wire>::default();
    let ended = Shared::<Option<Result<(), Trap>>>::default();
    let hear = {
        let (wire, ended) = (wire.clone(), ended.clone());
        Box::new(move || {
            if ended.lock().is_some() {
                return None;
            }
            let sent = wire.lock().sent.clone();
            let heard = solution_on(solution, &sent, false, limit, stack);
            if heard.ended.is_some() {
                *ended.lock() = heard.ended;
            }
            let mut wire = wire.lock();
            let new = heard
                .out
                .get(wire.heard.len()..)
                .unwrap_or_default()
                .to_vec();
            if new.is_empty() {
                // Both wait on the other, or the solution has ended.
                return None;
            }
            wire.heard.extend_from_slice(&new);
            wire.say(Side::Solution, &new);
            Some(new)
        })
    };
    let io = Io::judge(input, hear).until(Stop::after(crate::case::judge_limit(limit)));
    let interactor = run(interactor, io, &mut wire.clone(), stack).err();
    // The interactor is done: the solution reads the end of its input.
    let last = ended.lock().take();
    let (out, result) = match last {
        Some(result) => (wire.lock().heard.clone(), result),
        None => {
            let sent = wire.lock().sent.clone();
            let heard = solution_on(solution, &sent, true, limit, stack);
            let mut wire = wire.lock();
            let new = heard
                .out
                .get(wire.heard.len()..)
                .unwrap_or_default()
                .to_vec();
            wire.say(Side::Solution, &new);
            (heard.out, heard.ended.unwrap_or(Ok(())))
        }
    };
    let out = String::from_utf8_lossy(&out).into_owned();
    let wire = std::mem::take(&mut *wire.lock());
    Spoken {
        solution: match result {
            Ok(()) => Ok(out),
            Err(trap) => Err((out, trap)),
        },
        interactor,
        sent: wire.sent.clone(),
        transcript: wire.finish(),
    }
}
