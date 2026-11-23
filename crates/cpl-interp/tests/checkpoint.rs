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
    for (source, input) in [(NESTED, &b""[..]), (CALLS, &b""[..]), (READS, &input[..])] {
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
    for source in [NESTED, CALLS] {
        let (program, _, taken) = record(source, b"");
        let mut out = Vec::new();
        let whole = observe_on_this_thread(
            &program,
            Io::fixed(b""),
            &mut out,
            LIMITS,
            Some(config(source)),
        )
        .recording
        .unwrap();
        assert!(!whole.truncated);
        for from in [&taken[0], &taken[taken.len() / 2], &taken[taken.len() - 1]] {
            let mut rest = Vec::new();
            let part =
                resume_on_this_thread(&program, b"", from, &mut rest, LIMITS, Some(config(source)))
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
    let program = program(LONG);
    let timeline = cpl_interp::trace::Timeline::record_on_this_thread(
        &program,
        b"",
        &config(LONG),
        LIMITS,
        64,
    );
    let starts = timeline.starts();
    assert!(starts.len() > 8 && starts.len() <= 64, "{starts:?}");
    assert!(starts.windows(2).all(|w| w[0] < w[1]), "{starts:?}");
    assert!(
        starts[0] < timeline.steps / 8,
        "{starts:?} of {}",
        timeline.steps
    );
    assert!(
        *starts.last().unwrap() > timeline.steps / 2,
        "{starts:?} of {}",
        timeline.steps
    );
}
