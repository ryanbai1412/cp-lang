//! Trace, profile and replay hear one event stream: on one program, the
//! trace's steps at a statement are the profile's hits there, and replaying
//! a recorded call gives back what the trace recorded of it.

mod common;
use common::program;
use cpl_interp::replay::{replay, Rerun};
use cpl_interp::trace::TraceConfig;
use cpl_interp::{run_observed, run_profiled, Io};

const SOURCE: &str = "\
var total = 0

fn add(x: Int) -> Int:
    total += x
    total

for i in 0..4:
    print(add(i))
";

fn config() -> TraceConfig {
    TraceConfig {
        source: SOURCE.to_string(),
        ..Default::default()
    }
}

fn line_of(at: u32) -> u32 {
    1 + SOURCE[..at as usize].matches('\n').count() as u32
}

#[test]
fn trace_profile_and_replay_agree() {
    let p = program(SOURCE);
    let mut traced_out = Vec::new();
    let traced = run_observed(&p, Io::fixed(b""), &mut traced_out, Some(config()));
    assert!(traced.trap.is_none(), "{:?}", traced.trap);
    let recording = traced.recording.expect("a traced run records");
    let mut profiled_out = Vec::new();
    let profiled = run_profiled(&p, Io::fixed(b""), &mut profiled_out);
    assert!(profiled.trap.is_none(), "{:?}", profiled.trap);
    let profile = profiled.profile.expect("a profiled run profiles");
    assert_eq!(traced_out, profiled_out);
    assert_eq!(String::from_utf8_lossy(&traced_out), "0\n1\n3\n6\n");

    let hits = |line: u32| -> u64 {
        (profile.sites.iter())
            .filter(|(span, _)| line_of(span.start) == line)
            .map(|(_, site)| site.hits)
            .sum()
    };
    let steps: Vec<u32> = recording
        .steps(0..recording.len())
        .map(|s| s.line)
        .collect();
    for &line in &steps {
        assert!(
            hits(line) > 0,
            "line {line} has a trace step and no profile hit"
        );
    }
    let at_write = steps.iter().filter(|&&line| line == 4).count() as u64;
    assert_eq!(
        (at_write, hits(4)),
        (4, 4),
        "`total += x`: trace steps, profile hits"
    );

    let call = (recording.calls.iter())
        .find(|c| c.name == "add" && c.ordinal == 3)
        .expect("the trace records the third call of add");
    assert_eq!(call.returned.as_deref(), Some("3"));
    let replayed = replay(
        Rerun {
            recorded: &recording,
            original: &p,
            edited: &p,
            input: b"",
            call,
        },
        config(),
    );
    assert!(replayed.trap.is_none() && replayed.fallback.is_none());
    let again = replayed.replayed.expect("the call ran again");
    assert_eq!(
        (again.args, again.returned),
        (call.args.clone(), call.returned.clone())
    );
    assert_eq!(replayed.before.returned, replayed.after.returned);
    assert!(replayed.change.is_none(), "{:?}", replayed.change);
}
