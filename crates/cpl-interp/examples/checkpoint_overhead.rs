//! The cost of checkpoints on the untraced run, against no checkpoints:
//! `cargo run --release -p cpl-interp --example checkpoint_overhead`.

#[path = "../tests/common/mod.rs"]
mod common;
use common::program;
use cpl_interp::checkpoint::record_on_this_thread;
use cpl_interp::{observe_on_this_thread, Io, Limits};

const LIMITS: Limits = Limits {
    stack_bytes: 1 << 20,
};

/// A loop in main, and a run that is mostly calls, two deep.
const SOURCES: [(&str, &str); 3] = [
    (
        "loop",
        "var s = 0\nfor i in 0..10000000:\n    s = (s + i * i) % 1000003\nprint(s)\n",
    ),
    (
        "calls",
        "fn g(x: Int) -> Int = (x * x + 1) % 1009\n\
         fn f(n: Int) -> Int:\n    var s = 0\n    for i in 0..n:\n        s = (s + g(i)) % 1000003\n    return s\n\
         var t = 0\nfor r in 0..1000:\n    t = (t + f(5000)) % 1000003\nprint(t)\n",
    ),
    (
        "seqs",
        "var a: List<Int> = []\nfor i in 0..10000:\n    a.push(i * 37 % 101)\n\
         var s = 0\nfor r in 0..500:\n    for x in a:\n        s = (s + x * r) % 1000003\n    for x in a.rev:\n        s = (s * 3 + x) % 1000003\nprint(s)\n",
    ),
];

fn main() {
    for (name, source) in SOURCES {
        measure(name, source);
    }
}

fn measure(name: &str, source: &str) {
    let program = program(source);
    let best = |every: Option<u64>| {
        (0..5)
            .map(|_| {
                let started = std::time::Instant::now();
                let mut out = Vec::new();
                let taken = match every {
                    Some(every) => record_on_this_thread(&program, b"", &mut out, LIMITS, every)
                        .1
                        .len(),
                    None => {
                        observe_on_this_thread(&program, Io::fixed(b""), &mut out, LIMITS, None);
                        0
                    }
                };
                (started.elapsed().as_secs_f64() * 1e3, taken)
            })
            .fold((f64::MAX, 0), |a, b| if b.0 < a.0 { b } else { a })
    };
    let (plain, _) = best(None);
    for every in [1 << 12, 1 << 16, 1 << 20] {
        let (ms, taken) = best(Some(every));
        println!(
            "{name}, every {every}: {ms:.1} ms against {plain:.1} ms untraced ({:.3}x), {taken} checkpoints",
            ms / plain
        );
    }
}
