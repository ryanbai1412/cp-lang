//! A run started again from a checkpoint does what the whole run did from
//! there on: the same output, and the same trace, step for step.

mod common;
use common::program;
use cpl_interp::checkpoint::{
    record_last_on_this_thread, record_on_this_thread, resume_on_this_thread, Checkpoint,
};
use cpl_interp::trace::{Recording, TraceConfig};
use cpl_interp::{observe_on_this_thread, Io, Limits};

const LIMITS: Limits = Limits {
    stack_bytes: 1 << 20,
};

const NESTED: &str = "var s = 0
var t = 1
for i in 0..3000:
    var k = i % 7
    while k > 0:
        s = (s * 31 + k + i) % 1000003
        k -= 1
    if i % 500 == 0:
        print(s)
    t = (t * 3 + s) % 998244353
print(s)
print(t)
";

const CALLS: &str = "fn f(x: Int) -> Int = (x * x + 1) % 1009
var s = 0
var i = 0
while i < 20000:
    s = (s + f(i)) % 1000003
    if i % 4000 == 0:
        print(s)
    i += 1
print(s)
";

const READS: &str = "let n: Int = read()
var s = 0
for i in 0..n:
    let a: Int = read()
    s = (s + a * rand(0, 5)) % 1000003
    if i % 3000 == 0:
        print(s)
print(s)
";

/// Most of the run is inside calls, three deep, each called partway
/// through an expression.
const DEEP: &str = "fn walk(n: Int, d: Int) -> Int:
    var s = 0
    var i = 0
    while i < n:
        s = (s * 7 + i + d) % 1000003
        i += 1
    if d == 0:
        return s
    return (s + walk(n, d - 1) * 3) % 1000003
var t = 0
for r in 0..4:
    t = (t + 2 * walk(3000, 3)) % 1000003
    print(t)
";

const SHARED: &str = "fn walk(n: Int, d: Int) -> Int:
    var s = 0
    for i in 0..n:
        s = (s * 7 + i + d) % 1000003
    return s
fn tree(n: Int, d: Int) -> Int:
    if d == 0:
        return walk(n, d)
    return (tree(n, d - 1) + tree(n / 2, d - 1) * 3) % 1000003
var t = 0
for r in 0..3:
    t = (t + walk(2000, r) * walk(3000, r + 1) + walk(1000, 2)) % 1000003
    print(walk(1500, r) % 1000, walk(700, r))
    t = (t + walk(900, r) + rand(0, 3) + walk(1100, r)) % 1000003
print(t, tree(900, 3))
";

const SEQ: &str = "fn total(a: List<Int>, d: Int) -> Int:
    var s = 0
    for x in a:
        s = (s * 7 + x + d) % 1000003
    return s
var a: List<Int> = []
for i in 0..3000:
    a.push(i * 37 % 101)
let word = \"abcabdacab\"
var t = 0
for w in [3, 1, 4, 1, 5]:
    t = (t + total(a, w)) % 1000003
    for c in word:
        if c == 'a':
            t += 1
for x in a.rev:
    t = (t * 3 + x) % 1000003
print(t)
";

const CLOSURE: &str = "fn apply(f: (Int) -> Int, n: Int) -> Int:
    var s = 0
    for i in 0..n:
        s = (s + f(i)) % 1000003
    return s
fn solve(m: Int) -> Int:
    let d = m % 5
    fn walk(n: Int) -> Int:
        var s = 0
        for i in 0..n:
            s = (s * 7 + i + d) % 1000003
        return s
    let step = (n: Int) -> Int =>
        var s = 1
        for i in 0..n:
            s = (s * 3 + i + m) % 1000003
        s
    var t = 0
    for r in 0..12:
        t = (t + walk(3000) + step(2000 + r)) % 1000003
    return t
let k = 3
print(apply(x => x * k + 1, 40000))
print(solve(7))
";

const MULTITEST_RAND: &str = "@!multitest
let n: Int = read()
var s = 0
for i in 0..n:
    s = (s + i * i + rand(0, 3)) % 1000003
print(s)
";

const MULTITEST_INPUT: &[u8] = b"3\n4000\n5000\n6000\n";

fn reads_input() -> Vec<u8> {
    let mut input = b"12000\n".to_vec();
    for i in 0..12000 {
        input.extend_from_slice(format!("{}\n", i * 7919 % 10007).as_bytes());
    }
    input
}

fn record(source: &str, input: &[u8]) -> (cpl_ir::Program, Vec<u8>, Vec<Checkpoint>) {
    let program = program(source);
    let mut out = Vec::new();
    let (observed, taken) = record_on_this_thread(&program, input, &mut out, LIMITS, 1);
    assert!(observed.trap.is_none(), "{:?}", observed.trap);
    (program, out, taken)
}

fn config(source: &str) -> TraceConfig {
    TraceConfig {
        source: source.to_string(),
        max_steps: Some(1 << 22),
        ..Default::default()
    }
}

fn step_text(recording: &Recording, step: usize) -> String {
    let step = recording.step(step).unwrap();
    format!(
        "{} {} {} {}",
        step.line, step.label, step.stage, step.locals
    )
}

#[test]
fn a_run_from_a_checkpoint_prints_the_rest_of_the_output() {
    let input = reads_input();
    for (source, input) in [
        (NESTED, &b""[..]),
        (CALLS, &b""[..]),
        (READS, &input[..]),
        (DEEP, &b""[..]),
        (SHARED, &b""[..]),
        (SEQ, &b""[..]),
        (CLOSURE, &b""[..]),
        (MULTITEST_RAND, MULTITEST_INPUT),
    ] {
        let (program, whole, taken) = record(source, input);
        assert!(taken.len() >= 2, "{} checkpoints: {source}", taken.len());
        for from in &taken {
            let mut rest = Vec::new();
            let observed = resume_on_this_thread(&program, input, from, &mut rest, LIMITS, None);
            assert!(observed.trap.is_none(), "{:?}", observed.trap);
            let printed = from.printed as usize;
            assert_eq!(
                String::from_utf8_lossy(&rest),
                String::from_utf8_lossy(&whole[printed..]),
                "from tick {}: {source}",
                from.ticks
            );
        }
    }
}

#[test]
fn a_traced_run_from_a_checkpoint_records_the_whole_trace_from_there() {
    for (source, input) in [
        (NESTED, &b""[..]),
        (CALLS, &b""[..]),
        (DEEP, &b""[..]),
        (SHARED, &b""[..]),
        (SEQ, &b""[..]),
        (CLOSURE, &b""[..]),
        (MULTITEST_RAND, MULTITEST_INPUT),
    ] {
        let (program, _, taken) = record(source, input);
        let mut out = Vec::new();
        let whole = observe_on_this_thread(
            &program,
            Io::fixed(input),
            &mut out,
            LIMITS,
            Some(config(source)),
        )
        .recording
        .unwrap();
        assert!(!whole.truncated);
        for from in [&taken[0], &taken[taken.len() / 2], &taken[taken.len() - 1]] {
            let mut rest = Vec::new();
            let part = resume_on_this_thread(
                &program,
                input,
                from,
                &mut rest,
                LIMITS,
                Some(config(source)),
            )
            .recording
            .unwrap();
            let skip = whole.len() - part.len();
            assert!(
                !part.is_empty() && skip > 0,
                "{} of {}",
                part.len(),
                whole.len()
            );
            for step in 0..part.len() {
                assert_eq!(
                    step_text(&part, step),
                    step_text(&whole, skip + step),
                    "step {step} from tick {}: {source}",
                    from.ticks
                );
            }
        }
    }
}

const MULTITEST: &str = "@!multitest\nlet n: Int = read()\nvar s = 0\nfor i in 0..n:\n    s = (s + i * i) % 1009\nprint(s)\n";

const ONE_CALL: &str = "fn walk(n: Int) -> Int:\n    var s = 0\n    for i in 0..n:\n        s = (s + i) % 1009\n    return s\nprint(walk(30000))\n";

#[test]
fn the_tail_from_a_checkpoint_is_the_whole_runs_last_steps() {
    let input = reads_input();
    for (source, input, steps) in [
        (NESTED, &b""[..], 1000),
        (CALLS, &b""[..], 5000),
        (READS, &input[..], 700),
        (NESTED, &b""[..], 1 << 22),
        (MULTITEST, &b"2\n3000\n4000\n"[..], 900),
        (ONE_CALL, &b""[..], 400),
        (DEEP, &b""[..], 3000),
        (MULTITEST_RAND, MULTITEST_INPUT, 2000),
        (SHARED, &b""[..], 3000),
        (SEQ, &b""[..], 3000),
        (CLOSURE, &b""[..], 3000),
        (NESTED, &b""[..], 1),
    ] {
        let program = program(source);
        let keep = TraceConfig {
            last_steps: Some(steps),
            ..config(source)
        };
        let mut whole_out = Vec::new();
        let io = Io::fixed(input);
        let whole =
            observe_on_this_thread(&program, io, &mut whole_out, LIMITS, Some(keep.clone()));
        let whole = whole.recording.unwrap();
        let mut out = Vec::new();
        let tail =
            cpl_interp::trace::resumed_tail_every(&program, input, &mut out, keep, steps, 64);
        assert!(tail.trap.is_none(), "{:?}", tail.trap);
        assert_eq!(
            String::from_utf8_lossy(&out),
            String::from_utf8_lossy(&whole_out)
        );
        let tail = tail.recording.unwrap();
        assert_eq!(tail.len(), whole.len(), "{steps} steps of {source}");
        for step in 0..tail.len() {
            assert_eq!(
                step_text(&tail, step),
                step_text(&whole, step),
                "step {step} of {source}"
            );
        }
    }
}

#[test]
fn a_run_keeping_the_last_checkpoints_keeps_the_whole_runs_last_ones() {
    let source = "var s = 0\nfor i in 0..100000:\n    s = (s + i * i) % 1009\n    if i % 20000 == 0:\n        print(s)\nprint(s)\n";
    let (program, out, all) = record(source, b"");
    assert!(all.len() > 3, "only {} checkpoints", all.len());
    let mut kept_out = Vec::new();
    let (observed, kept) = record_last_on_this_thread(&program, b"", &mut kept_out, LIMITS, 1, 3);
    assert!(observed.trap.is_none(), "{:?}", observed.trap);
    assert_eq!(kept_out, out);
    let ticks = |cs: &[Checkpoint]| cs.iter().map(|c| c.ticks).collect::<Vec<_>>();
    assert_eq!(ticks(&kept), ticks(&all[all.len() - 3..]));
}

const LONG: &str = "var s = 0\nvar t = 1\nfor i in 0..300000:\n    s = (s + i * i) % 1009\n    if i % 7 == 0:\n        t = (t * 3 + s) % 1000003\nprint(s)\nprint(t)\n";

#[test]
fn a_window_from_a_checkpoint_is_the_whole_runs_window() {
    let input = reads_input();
    for (source, input) in [
        (NESTED, &b""[..]),
        (CALLS, &b""[..]),
        (READS, &input[..]),
        (MULTITEST, &b"2\n3000\n4000\n"[..]),
        (ONE_CALL, &b""[..]),
        (LONG, &b""[..]),
        (DEEP, &b""[..]),
        (SHARED, &b""[..]),
        (SEQ, &b""[..]),
        (CLOSURE, &b""[..]),
        (MULTITEST_RAND, MULTITEST_INPUT),
    ] {
        let program = program(source);
        let replay = cpl_interp::trace::Replay::start_every(
            program.clone(),
            input.to_vec(),
            config(source),
            64,
        );
        for from in [0, 1, 37, 4999, 20_000, 333_333, 1_000_000, usize::MAX / 2] {
            let window = TraceConfig {
                from_step: Some(from),
                max_steps: Some(300),
                ..config(source)
            };
            let io = Io::fixed(input);
            let whole =
                observe_on_this_thread(&program, io, &mut Vec::new(), LIMITS, Some(window.clone()));
            let whole = whole.recording.unwrap();
            let replayed = replay.window(window);
            assert!(replayed.trap.is_none(), "{:?}", replayed.trap);
            let replayed = replayed.recording.unwrap();
            let at = format!("from {from} of {source}");
            assert_eq!(replayed.skipped, whole.skipped, "{at}");
            assert_eq!(replayed.truncated, whole.truncated, "{at}");
            assert_eq!(replayed.len(), whole.len(), "{at}");
            for step in 0..whole.len() {
                assert_eq!(
                    step_text(&replayed, step),
                    step_text(&whole, step),
                    "step {step} {at}"
                );
            }
        }
    }
}

#[test]
fn a_timeline_keeps_checkpoints_across_the_whole_run() {
    for source in [LONG, DEEP, ONE_CALL, SHARED, SEQ, CLOSURE] {
        let program = program(source);
        let timeline = cpl_interp::trace::Timeline::record_on_this_thread(
            &program,
            b"",
            &config(source),
            LIMITS,
            64,
        );
        let starts = timeline.starts();
        let of = format!("{starts:?} of {} in {source}", timeline.steps);
        assert!(starts.len() > 4 && starts.len() <= 64, "{of}");
        assert!(starts.windows(2).all(|w| w[0] < w[1]), "{of}");
        assert!(starts[0] < timeline.steps / 4, "{of}");
        assert!(*starts.last().unwrap() > timeline.steps * 7 / 8, "{of}");
        let gap = starts.windows(2).map(|w| w[1] - w[0]).max().unwrap();
        assert!(gap < timeline.steps / 4, "{of}");
        if source == LONG {
            assert!(starts.len() > 8 && starts[0] < timeline.steps / 8, "{of}");
        }
    }
}

#[test]
fn a_counted_runs_checkpoints_resume_to_the_rest_of_the_output() {
    for source in [ONE_CALL, DEEP, SHARED, SEQ, CLOSURE] {
        let (program, whole, _) = record(source, b"");
        let (counted, taken) = cpl_interp::checkpoint::count_on_this_thread(
            &program,
            b"",
            LIMITS,
            config(source),
            64,
            64,
        );
        assert!(counted.trap.is_none(), "{:?}", counted.trap);
        assert!(taken.iter().any(|c| c.depth() > 0), "{source}");
        for from in &taken {
            let mut rest = Vec::new();
            let observed = resume_on_this_thread(&program, b"", from, &mut rest, LIMITS, None);
            assert!(observed.trap.is_none(), "{:?}", observed.trap);
            assert_eq!(
                String::from_utf8_lossy(&rest),
                String::from_utf8_lossy(&whole[from.printed as usize..]),
                "from step {}: {source}",
                from.steps
            );
        }
    }
}

#[test]
fn a_window_stops_the_run_once_it_is_full() {
    let source = "var s = 0\nfor i in 0..100000:\n    s = (s + i) % 1009\nassert s < 0\n";
    let program = program(source);
    let whole = observe_on_this_thread(
        &program,
        Io::fixed(b""),
        &mut Vec::new(),
        LIMITS,
        Some(config(source)),
    );
    assert!(whole.trap.is_some(), "the whole run fails its assert");
    let replay = cpl_interp::trace::Replay::start_every(program, Vec::new(), config(source), 64);
    for from in [10, 5000, 50_000] {
        let window = replay.window(TraceConfig {
            from_step: Some(from),
            max_steps: Some(50),
            ..config(source)
        });
        assert!(window.trap.is_none(), "from {from}: {:?}", window.trap);
        let rec = window.recording.unwrap();
        assert!(rec.truncated, "from {from}");
        assert_eq!((rec.skipped, rec.len()), (from, 50));
    }
}

#[test]
fn a_checkpoint_can_stand_inside_calls_and_test_cases() {
    let (_, _, taken) = record(DEEP, b"");
    let depths: std::collections::BTreeSet<usize> = taken.iter().map(Checkpoint::depth).collect();
    assert_eq!(depths, (1..=4).collect(), "{depths:?}");
    let (_, _, taken) = record(MULTITEST_RAND, MULTITEST_INPUT);
    let cases: std::collections::BTreeSet<u64> =
        taken.iter().filter_map(Checkpoint::case).collect();
    assert!(cases.len() >= 2, "{cases:?}");
}

#[test]
fn a_windowed_trace_is_the_full_traces_window_and_the_whole_run() {
    let failing = "var s = 0\nfor i in 0..100000:\n    s = (s + i) % 1009\n    if i % 30000 == 0:\n        print(s)\nassert s < 0\n";
    for source in [NESTED, CALLS, ONE_CALL, LONG, failing] {
        for from in [None, Some(0), Some(4999), Some(1_000_000)] {
            let program = program(source);
            let config = TraceConfig {
                from_step: from,
                max_steps: Some(300),
                ..config(source)
            };
            let mut full_out = Vec::new();
            let full = cpl_interp::run_observed(
                &program,
                Io::fixed(b""),
                &mut full_out,
                Some(config.clone()),
            );
            let mut out = Vec::new();
            let windowed = cpl_interp::trace::windowed(&program, b"", &mut out, config);
            let at = format!("from {from:?} of {source}");
            assert_eq!(out, full_out, "{at}");
            assert_eq!(
                windowed.trap.map(|t| t.message),
                full.trap.map(|t| t.message),
                "{at}"
            );
            let (rec, whole) = (windowed.recording.unwrap(), full.recording.unwrap());
            assert_eq!(
                (rec.skipped, rec.truncated, rec.len()),
                (whole.skipped, whole.truncated, whole.len()),
                "{at}"
            );
            for step in 0..whole.len() {
                assert_eq!(
                    step_text(&rec, step),
                    step_text(&whole, step),
                    "{at}, step {step}"
                );
            }
        }
    }
}

const CELLS: &str = "var a: [0..50]Int = 0
var best = 0
fn bump(x: Int) -> Int:
    a[x % 50] += x
    return a[x % 50] % 1009
for i in 0..4000:
    a[i % 50] += 1
    let v = bump(i)
    if v > best:
        best = v
print(best)
";

#[test]
fn a_window_from_step_n_is_the_whole_traces_steps_from_n() {
    let input = reads_input();
    for (source, input) in [
        (NESTED, &b""[..]),
        (CALLS, &b""[..]),
        (READS, &input[..]),
        (DEEP, &b""[..]),
        (ONE_CALL, &b""[..]),
        (SHARED, &b""[..]),
        (CELLS, &b""[..]),
        (MULTITEST_RAND, MULTITEST_INPUT),
    ] {
        let program = program(source);
        let whole = observe_on_this_thread(
            &program,
            Io::fixed(input),
            &mut Vec::new(),
            LIMITS,
            Some(config(source)),
        );
        let whole = whole.recording.unwrap();
        for from in [
            1,
            2,
            37,
            1000,
            whole.len() / 2,
            whole.len().saturating_sub(5),
        ] {
            let window = TraceConfig {
                from_step: Some(from),
                max_steps: Some(200),
                ..config(source)
            };
            let part = observe_on_this_thread(
                &program,
                Io::fixed(input),
                &mut Vec::new(),
                LIMITS,
                Some(window),
            );
            let part = part.recording.unwrap();
            let at = format!("from {from} of {source}");
            assert_eq!(part.skipped, from.min(whole.len()), "{at}");
            for step in 0..part.len().min(whole.len().saturating_sub(from)) {
                assert_eq!(
                    step_text(&part, step),
                    step_text(&whole, from + step),
                    "{at}, step {step}"
                );
            }
        }
    }
}
