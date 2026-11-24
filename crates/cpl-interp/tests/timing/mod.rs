//! Slowdown ratios a busy machine cannot decide. A run is timed by the
//! less of its wall time and the CPU time of the threads running it: a
//! neighbour that has the core adds to the first and not the second, and
//! the second never counts other threads, of this test binary or any other.
//! The modes take turns (A B C, C B A, ...), so what load is left (a
//! neighbour on the other half of a core, in the caches) lands on all of
//! them alike, and each keeps its least time.

use cpl_interp::trace::TraceConfig;
use cpl_interp::{Io, Limits};
use cpl_ir::Program;
use std::time::{Duration, Instant};

/// Rounds of every mode: the least of these is each mode's time.
pub const ROUNDS: usize = 7;

/// The interpreter's stack, as `cpl_interp::run_observed` asks for it.
const STACK: usize = 8 << 30;

mod clock {
    /// The CPU time this thread has run, where the kernel says (Linux's
    /// schedstat, in nanoseconds); elsewhere none, and wall time decides.
    /// The kernel brings the count up to date when the thread yields, not
    /// when it is read, so it yields first.
    pub fn thread_cpu() -> Option<std::time::Duration> {
        std::thread::yield_now();
        let stat = std::fs::read_to_string("/proc/thread-self/schedstat").ok()?;
        let ns = stat.split_whitespace().next()?.parse().ok()?;
        Some(std::time::Duration::from_nanos(ns))
    }
}

std::thread_local! {
    /// CPU time the interpreter threads [`run`] started from this thread spent.
    static INTERPRETER_CPU: std::cell::Cell<Duration> =
        const { std::cell::Cell::new(Duration::ZERO) };
}

/// Runs `program` on `input` as `cpl_interp::run_observed` does, on a
/// thread with the interpreter's deep stack, and counts that thread's CPU
/// time toward this one's.
pub fn run(program: &Program, input: &[u8], trace: Option<&TraceConfig>) {
    let on_its_thread = || {
        let started = clock::thread_cpu();
        cpl_interp::observe_on_this_thread(
            program,
            Io::fixed(input),
            &mut Vec::new(),
            Limits {
                stack_bytes: STACK - STACK / 64,
            },
            trace.cloned(),
        );
        started
            .zip(clock::thread_cpu())
            .map(|(started, ended)| ended - started)
    };
    let spent = std::thread::scope(|scope| {
        std::thread::Builder::new()
            .stack_size(STACK)
            .spawn_scoped(scope, on_its_thread)
            .expect("the interpreter thread starts")
            .join()
            .expect("the run finishes")
    });
    INTERPRETER_CPU.set(INTERPRETER_CPU.get() + spent.unwrap_or_default());
}

/// This thread's CPU time and that of the interpreter threads it started.
fn cpu() -> Option<Duration> {
    clock::thread_cpu().map(|cpu| cpu + INTERPRETER_CPU.get())
}

/// How long `run` took: the less of wall and CPU time.
fn time(run: &mut dyn FnMut()) -> Duration {
    let before = cpu();
    let started = Instant::now();
    run();
    let wall = started.elapsed();
    match before.zip(cpu()) {
        Some((before, after)) => wall.min(after - before),
        None => wall,
    }
}

/// Every mode's time in every round: [`rounds`] measures them.
pub struct Rounds(Vec<Vec<Duration>>);

/// Times `modes` over `rounds` rounds, the modes taking turns.
pub fn rounds(rounds: usize, modes: &mut [&mut dyn FnMut()]) -> Rounds {
    Rounds(
        (0..rounds)
            .map(|round| {
                let mut times = vec![Duration::ZERO; modes.len()];
                for k in 0..modes.len() {
                    let k = if round % 2 == 0 {
                        k
                    } else {
                        modes.len() - 1 - k
                    };
                    times[k] = time(&mut *modes[k]);
                }
                times
            })
            .collect(),
    )
}

/// Times `modes` as [`rounds`] does, then [`ROUNDS`] more at a time, up
/// to [`MAX_ROUNDS`], while `holds` says no: a least time only goes down
/// with more rounds, so a busy machine gets more chances and a real
/// slowdown still fails.
pub fn rounds_until(modes: &mut [&mut dyn FnMut()], holds: impl Fn(&Rounds) -> bool) -> Rounds {
    let mut all = rounds(ROUNDS, modes);
    while !holds(&all) && all.0.len() < MAX_ROUNDS {
        all.0.extend(rounds(ROUNDS, modes).0);
    }
    all
}

/// The most rounds [`rounds_until`] runs.
pub const MAX_ROUNDS: usize = 4 * ROUNDS;

impl Rounds {
    /// How many rounds were timed.
    pub fn count(&self) -> usize {
        self.0.len()
    }

    /// Mode `k`'s least time, the round the load disturbed least.
    pub fn time(&self, k: usize) -> Duration {
        self.0.iter().map(|round| round[k]).min().unwrap()
    }

    /// How many times slower mode `k` ran than mode `than`, by their least
    /// times.
    pub fn slower(&self, k: usize, than: usize) -> f64 {
        self.time(k).as_secs_f64() / self.time(than).as_secs_f64()
    }
}

/// `measure` run beside a noisy neighbour: a thread per core, half
/// streaming through 64 MB each and half spinning on arithmetic.
pub fn beside_neighbours<T>(measure: impl FnOnce() -> T) -> T {
    use std::hint::black_box;
    use std::sync::atomic::{AtomicBool, Ordering};
    let done = AtomicBool::new(false);
    let cores = std::thread::available_parallelism().map_or(2, |n| n.get());
    let started = std::sync::Barrier::new(cores + 1);
    std::thread::scope(|scope| {
        for k in 0..cores {
            let (done, started) = (&done, &started);
            scope.spawn(move || {
                started.wait();
                let mut bytes = vec![0u8; if k % 2 == 0 { 64 << 20 } else { 0 }];
                let mut x = 1u64;
                while !done.load(Ordering::Relaxed) {
                    if bytes.is_empty() {
                        for _ in 0..1 << 16 {
                            x = black_box(x.wrapping_mul(6364136223846793005).wrapping_add(1));
                        }
                    } else {
                        bytes.fill(x as u8);
                        x = x.wrapping_add(
                            u64::from(black_box(bytes[x as usize % bytes.len()])) + 1,
                        );
                    }
                }
            });
        }
        started.wait();
        let measured = measure();
        done.store(true, Ordering::Relaxed);
        measured
    })
}
