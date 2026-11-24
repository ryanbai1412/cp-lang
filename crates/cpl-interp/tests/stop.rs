//! A run ends when its `Stop` goes off, on every path the interpreter takes.

mod common;
use common::program;
use cpl_interp::{Io, Stop, CANCELLED, OUT_OF_TIME};
use std::time::{Duration, Instant};

fn ends_at(source: &str, stop: Stop, traced: bool) -> Option<String> {
    let started = Instant::now();
    let config = traced.then(|| cpl_interp::trace::TraceConfig {
        source: source.to_string(),
        ..Default::default()
    });
    let mut out = Vec::new();
    let observed = cpl_interp::run_observed(
        &program(source),
        Io::fixed(b"").until(stop),
        &mut out,
        config,
    );
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "{:?}",
        started.elapsed()
    );
    observed.trap.map(|trap| trap.message)
}

const NEVER_ENDS: [&str; 5] = [
    // A loop the VM runs, with nothing to record.
    "var i = 0\nwhile i >= 0:\n    i = (i + 1) % 2\nprint(i)\n",
    "var s = 0\nfor i in 0..=1e18:\n    s ^= i\nprint(s)\n",
    // Recursion that never deepens the stack past its first frames.
    "fn f(n: Int) -> Int:\n    var k = n\n    while true:\n        k = g(k)\n    k\nfn g(n: Int) -> Int = (n + 1) % 7\nprint(f(0))\n",
    // A comprehension over a range too long to walk.
    "print([x % 3 for x in 0..1e18 if x < 0].len)\n",
    // A generic std fold over a huge range.
    "let r = 1..=1e18\nprint(r.count(x => x < 0))\n",
];

#[test]
fn a_run_past_its_deadline_traps_traced_or_not() {
    for source in NEVER_ENDS {
        for traced in [false, true] {
            let trap = ends_at(source, Stop::after(Duration::from_millis(100)), traced);
            assert_eq!(
                trap.as_deref(),
                Some(OUT_OF_TIME),
                "traced {traced}: {source}"
            );
        }
    }
}

#[test]
fn a_cancel_from_another_thread_ends_the_run() {
    let stop = Stop::new();
    let canceller = stop.clone();
    std::thread::spawn(move || canceller.cancel());
    assert_eq!(
        ends_at(NEVER_ENDS[0], stop, false).as_deref(),
        Some(CANCELLED)
    );
}

#[test]
fn a_stop_that_never_goes_off_changes_nothing() {
    let source = "var s = 0\nfor i in 0..100000:\n    s += i\nprint(s)\n";
    assert_eq!(
        ends_at(source, Stop::after(Duration::from_secs(60)), false),
        None
    );
    let mut out = Vec::new();
    let io = Io::fixed(b"").until(Stop::new());
    cpl_interp::run_with(&program(source), io, &mut out).unwrap();
    assert_eq!(out, b"4999950000\n");
}

#[test]
fn run_for_stops_the_run_it_gives_up_on() {
    let program = program(NEVER_ENDS[0]);
    assert!(cpl_interp::run_for(&program, b"", Duration::from_millis(100)).is_none());
    let done = cpl_interp::run_for(&program_of_one(), b"", Duration::from_secs(10));
    assert_eq!(
        done.map(|(out, ran)| (out, ran.is_ok())),
        Some((b"1\n".to_vec(), true))
    );
}

fn program_of_one() -> cpl_ir::Program {
    program("print(1)\n")
}

#[test]
fn a_run_past_its_round_budget_traps_traced_or_not() {
    // The browser's clock: a budget of loop rounds and calls.
    for traced in [false, true] {
        for source in NEVER_ENDS {
            let trap = ends_at(source, Stop::after_rounds(1_000_000), traced);
            assert_eq!(
                trap.as_deref(),
                Some(OUT_OF_TIME),
                "traced {traced}: {source}"
            );
        }
        let small = "var s = 0\nfor i in 0..1000:\n    s += i\nprint(s)\n";
        assert_eq!(ends_at(small, Stop::after_rounds(1_000_000), traced), None);
    }
}
