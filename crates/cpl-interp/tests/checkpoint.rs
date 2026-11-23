//! A run started again from a checkpoint does what the whole run did from
//! there on: the same output, and the same trace, step for step.

mod common;
use common::program;
use cpl_interp::checkpoint::{record_on_this_thread, resume_on_this_thread, Checkpoint};
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
