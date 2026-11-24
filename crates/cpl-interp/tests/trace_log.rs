//! The recording is a log of changes with a snapshot now and then: any
//! step rebuilt from it is the step the run took.

mod common;
mod counting;
mod timing;
use common::program;
use cpl_interp::trace::{Recording, TraceConfig};
use std::time::Instant;
use timing::ROUNDS;

fn record(source: &str, input: &str, config: TraceConfig) -> Recording {
    let config = TraceConfig {
        source: source.into(),
        record_frames: true,
        ..config
    };
    let mut out = Vec::new();
    let observed = cpl_interp::run_observed(
        &program(source),
        cpl_interp::Io::fixed(input.as_bytes()),
        &mut out,
        Some(config),
    );
    assert!(observed.trap.is_none());
    observed.recording.expect("it was traced")
}

const CALLS: &str = "\
var xs: List<Int> = []
fn sum_below(k: Int) -> Int:
    var s = 0
    for j in 0..k:
        s += j
    return s
for i in 0..900:
    xs.push(sum_below(i % 5))
    if i % 7 == 0:
        print(i)
";

#[test]
fn any_step_rebuilt_alone_is_the_step_the_run_took() {
    let rec = record(CALLS, "", TraceConfig::default());
    assert!(
        rec.len() > 3000,
        "{} steps: more than a few snapshots apart",
        rec.len()
    );
    let walked: Vec<String> = rec.steps(0..rec.len()).map(|s| format!("{s:?}")).collect();
    for k in (0..rec.len()).rev() {
        assert_eq!(format!("{:?}", rec.step(k).unwrap()), walked[k], "step {k}");
    }
    let end = rec.step(rec.len() - 1).unwrap();
    let stdout = &end
        .values
        .iter()
        .find(|(var, _)| rec.vars[*var].id == "stdout")
        .unwrap()
        .1;
    assert!(
        stdout.starts_with("[\"0\",\"7\",") && stdout.ends_with("\"896\"]"),
        "{stdout}"
    );
    assert!(end.frames.is_empty() || end.frames == [("main".to_string(), end.line)]);
    let inside = rec
        .steps(0..rec.len())
        .find(|s| s.frames.len() == 2)
        .expect("a step inside the call");
    assert_eq!(inside.frames[0].0, "sum_below");
}

const MIXED: &str = "\
fn count(k: Int) -> Int:
    var s = 0
    for j in 0..k:
        s += j * 3000000000000
    return s
fn listed(k: Int) -> Int:
    var t = 0
    var ys: List<Int> = []
    for j in 0..k:
        ys.push(j)
        t += j
    return t
for i in 0..300:
    print(count(i % 4) + listed(i % 3))
";

/// Locals that are all `Int`s are kept raw; a step with any other local
/// keeps their JSON, and either reads back the same from any step.
#[test]
fn raw_and_json_locals_read_back_from_any_step() {
    let rec = record(MIXED, "", TraceConfig::default());
    let walked: Vec<String> = rec.steps(0..rec.len()).map(|s| format!("{s:?}")).collect();
    for k in (0..rec.len()).rev() {
        assert_eq!(format!("{:?}", rec.step(k).unwrap()), walked[k], "step {k}");
    }
    let locals: Vec<String> = rec.steps(0..rec.len()).map(|s| s.locals).collect();
    assert!(
        locals
            .iter()
            .any(|l| l == "{\"k\":3,\"s\":9000000000000,\"j\":2}"),
        "{locals:?}"
    );
    assert!(
        locals.iter().any(|l| l.contains("\"ys\":[0,1]")),
        "{locals:?}"
    );
}

const ONE_LOCAL: &str = "\
fn walk(n: Int) -> Int:
    var a = 1
    var b = 2
    var c = 3
    for j in 0..n:
        c += j % 5
    return a + b + c
print(walk(5000))
";

/// A step that changes one of its `Int` locals logs that change alone,
/// and any step, the last ones kept included, reads back the same.
#[test]
fn a_step_changing_one_int_local_logs_only_that_local() {
    let rec = record(ONE_LOCAL, "", TraceConfig::default());
    let per_step = rec.bytes() as f64 / rec.len() as f64;
    assert!(
        per_step < 20.0,
        "{per_step:.1} bytes a step over {} steps",
        rec.len()
    );
    let walked: Vec<String> = rec.steps(0..rec.len()).map(|s| format!("{s:?}")).collect();
    for k in (0..rec.len()).step_by(97).chain([rec.len() - 1]) {
        assert_eq!(format!("{:?}", rec.step(k).unwrap()), walked[k], "step {k}");
    }
    assert!(
        walked
            .iter()
            .any(|s| s.contains(r#"{\"n\":5000,\"a\":1,\"b\":2,\"c\":13,\"j\":4}"#)),
        "{:?}",
        &walked[..12]
    );
    let last = record(
        ONE_LOCAL,
        "",
        TraceConfig {
            last_steps: Some(1000),
            ..TraceConfig::default()
        },
    );
    let kept: Vec<String> = last
        .steps(0..last.len())
        .map(|s| format!("{s:?}"))
        .collect();
    assert_eq!(kept.last(), walked.last());
    assert_eq!(kept[kept.len() - 500..], walked[walked.len() - 500..]);
}

const GRID: &str = "\
var g: [0..4, 0..4]Int = 0
for i in 0..4:
    g[i, 3 - i] = i + 1
print(g[0, 3])
";

/// A write to a cell of a grid focuses the step on that cell.
#[test]
fn a_grid_write_focuses_its_cell() {
    let rec = record(GRID, "", TraceConfig::default());
    let g = rec
        .vars
        .iter()
        .position(|v| v.id == "g")
        .expect("g is traced");
    let focused: Vec<(usize, Vec<usize>)> =
        rec.steps(0..rec.len()).filter_map(|s| s.focus).collect();
    assert_eq!(
        focused,
        (0..4).map(|i| (g, vec![i, 3 - i])).collect::<Vec<_>>()
    );
}

const CELLS: &str = "\
var a: [0..8]Int = 0
for i in 0..2000:
    a[i % 8] += i % 3
print(a[0])
";

#[test]
fn array_cells_rebuilt_from_single_changes_hold_every_write() {
    let rec = record(CELLS, "", TraceConfig::default());
    let a = rec
        .vars
        .iter()
        .position(|v| v.id == "a")
        .expect("a is traced");
    let value = |k: usize| {
        rec.step(k)
            .unwrap()
            .values
            .iter()
            .find(|(var, _)| *var == a)
            .map(|(_, v)| v.to_string())
    };
    let show = |a: &[usize; 8]| format!("[{}]", a.map(|x| x.to_string()).join(","));
    let mut cells = [0; 8];
    let mut states = vec![show(&cells)];
    for i in 0..2000 {
        cells[i % 8] += i % 3;
        if i % 3 != 0 {
            states.push(show(&cells));
        }
    }
    let mut seen: Vec<String> = Vec::new();
    for k in 0..rec.len() {
        if let Some(got) = value(k) {
            if seen.last() != Some(&got) {
                seen.push(got);
            }
        }
    }
    assert_eq!(seen, states);
    for k in 1..rec.len() {
        let (before, now) = (value(k - 1), value(k));
        let (Some(was), Some(is)) = (&before, &now) else {
            continue;
        };
        if was != is {
            let focus = rec.step(k).unwrap().focus.expect("a write shows its cell");
            assert_eq!(focus.0, a, "step {k}");
            let cell = |v: &str| {
                v.trim_matches(['[', ']'])
                    .split(',')
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            };
            let (was, is) = (cell(was), cell(is));
            let changed: Vec<usize> = (0..8).filter(|&i| was[i] != is[i]).collect();
            assert_eq!(focus.1, changed, "step {k}");
        }
    }
    assert_eq!(seen.last().unwrap(), "[249,250,251,249,250,251,249,250]");
    let tail = record(
        CELLS,
        "",
        TraceConfig {
            last_steps: Some(1500),
            ..TraceConfig::default()
        },
    );
    let skipped = rec.len() - tail.len();
    for (k, step) in tail.steps(0..tail.len()).enumerate() {
        assert_eq!(
            format!("{step:?}"),
            format!("{:?}", rec.step(skipped + k).unwrap()),
            "step {k}"
        );
    }
}

#[test]
fn the_last_steps_kept_are_the_last_steps_of_the_whole_run() {
    let whole = record(CALLS, "", TraceConfig::default());
    let tail = record(
        CALLS,
        "",
        TraceConfig {
            last_steps: Some(1500),
            ..TraceConfig::default()
        },
    );
    assert!(tail.truncated);
    assert_eq!(tail.len(), 1500);
    let skipped = whole.len() - tail.len();
    for (k, step) in tail.steps(0..tail.len()).enumerate() {
        assert_eq!(
            format!("{step:?}"),
            format!("{:?}", whole.step(skipped + k).unwrap()),
            "step {k}"
        );
    }
}

#[test]
fn each_test_case_prints_anew() {
    let source = "@!multitest\nlet n: Int = read()\nprint(n)\nprint(n + 1)\n";
    let rec = record(source, "2\n5\n8\n", TraceConfig::default());
    let stdout = |k: usize| {
        let step = rec.step(k).unwrap();
        step.values
            .iter()
            .find(|(var, _)| rec.vars[*var].id == "stdout")
            .unwrap()
            .1
            .to_string()
    };
    assert_eq!(stdout(rec.len() - 1), "[\"8\",\"9\"]");
    let first = (0..rec.len()).find(|&k| stdout(k) == "[\"5\",\"6\"]");
    assert!(first.is_some(), "the first case printed its two lines");
}

const WIDE: &str = "\
var x = 0
var a: [0..4]Int = 0
for i in 0..3000:
    x = (i - 1500) * 3000000000000 + i
    a[i % 4] = x * (i % 2) + i
print(x, a[0])
";

/// `Int`s kept raw, in and past 32 bits and past what a JSON number holds
/// exactly, read back as the run wrote them, from any step.
#[test]
fn raw_ints_read_back_as_written() {
    let rec = record(WIDE, "", TraceConfig::default());
    let var = |name: &str| rec.vars.iter().position(|v| v.id == name).unwrap();
    let (x, a) = (var("x"), var("a"));
    let value = |k: usize, var: usize| {
        rec.step(k)
            .unwrap()
            .values
            .iter()
            .find(|(v, _)| *v == var)
            .map(|(_, v)| v.to_string())
    };
    let json = |v: i64| {
        if v.unsigned_abs() > 9_007_199_254_740_991 {
            format!("\"{v}\"")
        } else {
            v.to_string()
        }
    };
    let mut xs = Vec::new();
    for k in 0..rec.len() {
        if let Some(got) = value(k, x) {
            if xs.last() != Some(&got) {
                xs.push(got);
            }
        }
    }
    let want: Vec<String> = std::iter::once("0".to_string())
        .chain((0..3000i64).map(|i| json((i - 1500) * 3_000_000_000_000 + i)))
        .collect();
    assert_eq!(xs, want);
    let last = (2996..3000i64).map(|i| json(((i - 1500) * 3_000_000_000_000 + i) * (i % 2) + i));
    let cells: Vec<String> = last.collect();
    assert_eq!(
        value(rec.len() - 1, a).unwrap(),
        format!("[{}]", cells.join(","))
    );
    let walked: Vec<String> = rec.steps(0..rec.len()).map(|s| format!("{s:?}")).collect();
    for k in (0..rec.len()).rev().step_by(7) {
        assert_eq!(format!("{:?}", rec.step(k).unwrap()), walked[k], "step {k}");
    }
}

/// Rounds of the debugging demos, each running them [`DEMO_RUNS`] times.
const DEMO_ROUNDS: usize = 20;
const DEMO_RUNS: usize = 100;

/// The modes whose recording is still over the 1.2x budget, each with a
/// ceiling on its slowdown that only goes down: debt-ratchet's
/// `trace_budget_gap` counts the tenths they allow over 1.2x. Owner:
/// VM-trace, who deletes an entry once its mode fits the budget.
const OVER_BUDGET: &[(&str, f64)] = &[
    ("scalar", 7.0),
    ("array", 7.0),
    ("knapsack", 1.8),
    ("shortest_path", 2.6),
    ("shows", 1.7),
];

/// How far a noisy neighbour can move a slowdown (see
/// `a_busy_machine_does_not_decide_the_trace_budget`).
const NOISE: f64 = 1.15;

/// Holds mode `name`, `slower` times the untraced run, to the 1.2x budget
/// or, while it is over that, to its [`OVER_BUDGET`] ceiling. An entry is
/// due for deletion only when it fits with [`NOISE`] to spare, so a quiet
/// machine can't demand a deletion that a busy one then fails.
fn check_budget(over: &mut Vec<String>, name: &str, slower: f64) {
    match OVER_BUDGET.iter().find(|(mode, _)| *mode == name) {
        Some(&(_, ceiling)) if slower > ceiling => over.push(format!(
            "{name}: tracing made the run {slower:.2}x slower (ceiling {ceiling}x, budget 1.2x)"
        )),
        Some(_) if slower * NOISE <= 1.2 => over.push(format!(
            "{name}: fits the 1.2x budget now: delete its OVER_BUDGET entry"
        )),
        Some(_) => eprintln!("{name}: {slower:.2}x, still over the 1.2x budget"),
        None if slower > 1.2 => {
            over.push(format!("{name}: tracing made the run {slower:.2}x slower"))
        }
        None => {}
    }
}

/// The budgets of a debug session at 10^7 steps: stepping back answers
/// in under 50 ms, recording makes the run at most 1.2x slower, and the
/// recording takes a few hundred MB. A wall-clock test (scripts/ci.py
/// WALL_CLOCK_TESTS): `scripts/check.sh --timing` (ci-slow.yml) reports it;
/// `cargo test --release -p cpl-interp --test trace_log -- --nocapture`.
#[test]
fn ten_million_steps_fit_the_debugging_budgets() {
    let mut over = Vec::new();
    for (name, source) in [
        (
            "scalar",
            "var x = 0\nfor i in 0..10000000:\n    x += i % 7\nprint(x)\n",
        ),
        (
            "array",
            "var a: [0..1000]Int = 0\nfor i in 0..10000000:\n    a[i % 1000] += 1\nprint(a[0])\n",
        ),
    ] {
        let program = program(source);
        let mut out = Vec::new();
        let run = |config: Option<&TraceConfig>| timing::run(&program, b"", config);
        // Hooks on, nothing kept: the traced VM under a filter that matches no fn.
        let observed = TraceConfig {
            source: source.into(),
            only_fn: Some("not_a_fn".into()),
            ..TraceConfig::default()
        };
        let config = TraceConfig {
            source: source.into(),
            record_frames: true,
            max_steps: Some(10_000_000),
            ..TraceConfig::default()
        };
        let times = timing::rounds(
            ROUNDS,
            &mut [&mut || run(None), &mut || run(Some(&observed)), &mut || {
                run(Some(&config))
            }],
        );
        let (plain, walked, traced) = (times.time(0), times.time(1), times.time(2));
        // The last 20,000 steps, traced again from a checkpoint: what time travel to the end costs.
        let started = Instant::now();
        let tail = TraceConfig {
            source: source.into(),
            record_frames: true,
            ..TraceConfig::default()
        };
        let tail = cpl_interp::trace::resumed_tail(&program, b"", &mut Vec::new(), tail, 20_000);
        let windowed = started.elapsed();
        assert_eq!(tail.recording.unwrap().len(), 20_000);
        out.clear();
        let (observed, usage) = counting::measure(|here| {
            here.observe(&program, cpl_interp::Io::fixed(b""), &mut out, Some(config))
        });
        let rec = observed.recording.unwrap();
        let held = usage.held;
        let mut back = std::time::Duration::ZERO;
        for k in [
            rec.len() - 1,
            rec.len() - 2,
            rec.len() / 2,
            1023,
            1024,
            1025,
        ] {
            let one = Instant::now();
            rec.step(k).unwrap();
            back = back.max(one.elapsed());
        }
        let mb = held as f64 / 1e6;
        let (slower, recording) = (times.slower(2, 0), times.slower(2, 1));
        eprintln!(
            "{name}: {} steps; untraced {plain:?}, observed {walked:?}, recorded {traced:?}: {slower:.2}x the untraced run, {recording:.2}x the observed one; {mb:.0} MB; step back at most {back:?}; the last 20,000 steps from a checkpoint {windowed:?}, {:.2}x the untraced run",
            rec.len(),
            windowed.as_secs_f64() / plain.as_secs_f64()
        );
        assert_eq!(rec.len(), 10_000_000);
        if back.as_millis() >= 50 {
            over.push(format!("{name}: step back took {back:?}"));
        }
        check_budget(&mut over, name, slower);
        if mb >= 500.0 {
            over.push(format!("{name}: the recording takes {mb:.0} MB"));
        }
    }
    // The debugging demos on the inputs `cpl stress` pins, each run many
    // times a round, since one run takes microseconds.
    let demos = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugging/");
    for (name, input) in [
        ("knapsack", "1 2\n1 1\n"),
        ("shortest_path", "2 2\n1 2 2\n1 2 1\n"),
        ("shows", "3\n0 2\n0 1\n1 1\n"),
    ] {
        let source = std::fs::read_to_string(format!("{demos}{name}.cp")).unwrap();
        let program = program(&source);
        let config = TraceConfig {
            source: source.clone(),
            record_frames: true,
            ..TraceConfig::default()
        };
        let runs = |trace: bool| {
            for _ in 0..DEMO_RUNS {
                timing::run(&program, input.as_bytes(), trace.then_some(&config));
            }
        };
        let modes = timing::rounds(DEMO_ROUNDS, &mut [&mut || runs(false), &mut || runs(true)]);
        let (plain, traced, slower) = (modes.time(0), modes.time(1), modes.slower(1, 0));
        eprintln!(
            "{name}: {DEMO_RUNS} runs; untraced {plain:?}, recorded {traced:?}: \
             {slower:.2}x the untraced run"
        );
        check_budget(&mut over, name, slower);
    }
    assert!(over.is_empty(), "over budget: {over:?}");
}

/// The measurement under the budgets above: a noisy neighbour on every core
/// moves the array slowdown (at 3*10^6 steps) by at most 15%, about what two
/// quiet runs differ by, so it can flip a verdict only that near the ceiling, and a recording that got
/// 2x slower is still caught.
/// A wall-clock test (scripts/ci.py WALL_CLOCK_TESTS).
#[test]
fn a_busy_machine_does_not_decide_the_trace_budget() {
    let source =
        "var a: [0..1000]Int = 0\nfor i in 0..3000000:\n    a[i % 1000] += 1\nprint(a[0])\n";
    let program = program(source);
    let config = TraceConfig {
        source: source.into(),
        record_frames: true,
        max_steps: Some(3_000_000),
        ..TraceConfig::default()
    };
    // Untraced, traced, and traced with every run done twice: a 2x regression.
    let slowdowns = || {
        let run = |config: Option<&TraceConfig>| timing::run(&program, b"", config);
        let times = timing::rounds(
            ROUNDS,
            &mut [&mut || run(None), &mut || run(Some(&config)), &mut || {
                (0..2).for_each(|_| run(Some(&config)))
            }],
        );
        let slower = |k: usize| times.slower(k, 0);
        (slower(1), slower(2))
    };
    let verdict = |slower: f64| {
        let mut over = Vec::new();
        check_budget(&mut over, "array", slower);
        over.is_empty()
    };
    let (quiet, _) = slowdowns();
    let (loaded, regressed) = timing::beside_neighbours(slowdowns);
    eprintln!(
        "array: {quiet:.2}x quiet, {loaded:.2}x loaded, {regressed:.2}x loaded and 2x slower"
    );
    assert!(
        (loaded / quiet - 1.0).abs() <= 0.15,
        "a busy machine moved the slowdown from {quiet:.2}x to {loaded:.2}x"
    );
    assert!(
        regressed / loaded >= 1.5 && !verdict(regressed),
        "a 2x slower recording measured {regressed:.2}x ({loaded:.2}x before) and passed"
    );
}

#[test]
fn a_step_inside_an_imported_fn_is_shown_at_the_call_running_it() {
    // `MinHeap.push` is lowered once, with the span of its first call:
    // the second call's writes are still shown at the second call.
    let source = "\
use std.heap.{MinHeap}
var heap = MinHeap<Int>()
heap.push(3)
let k: Int = read()
heap.push(k)
print(heap.len)
";
    let recording = record(source, "5", TraceConfig::default());
    let lines: Vec<u32> = (0..recording.len())
        .filter_map(|step| recording.line(step))
        .collect();
    assert!(lines.contains(&3), "{lines:?}");
    assert!(lines.contains(&5), "{lines:?}");
}

#[test]
fn frames_name_a_generic_method_as_the_source_does() {
    let source = "\
struct Box<T>:
    x: T
    fn get(self) -> T:
        let y = x
        return y
let b = Box(x: 3)
print(b.get())
";
    let rec = record(source, "", TraceConfig::default());
    let inside = rec
        .steps(0..rec.len())
        .find(|s| s.frames.len() == 2)
        .expect("a step inside the call");
    assert_eq!(inside.frames[0].0, "Box.get");
}

#[test]
fn a_trap_in_a_calls_first_statement_ends_on_its_locals() {
    let source = "\
fn get(a: List<Int>, j: Int) -> Int:
    return a[j]
let k = 2
print(get([0, 0, 0], k + 1))
";
    let config = TraceConfig {
        source: source.into(),
        record_frames: true,
        ..TraceConfig::default()
    };
    let mut out = Vec::new();
    let io = cpl_interp::Io::fixed(b"");
    let observed = cpl_interp::run_observed(&program(source), io, &mut out, Some(config));
    assert!(observed.trap.is_some());
    let rec = observed.recording.expect("recorded");
    let end = rec.step(rec.len() - 1).unwrap();
    assert!(end.label.starts_with("failed: "), "{}", end.label);
    assert_eq!(end.frames.len(), 2, "{:?}", end.frames);
    assert_eq!(end.frames[0].0, "get");
    assert!(end.locals.contains("\"j\""), "{}", end.locals);
}

#[test]
fn a_window_of_a_run_holds_the_steps_the_whole_recording_holds_there() {
    let source = "\
fn bump(x: Int) -> Int:
    let y = x * 2
    return y + 1

var s = 0
var a: [0..5]Int = 0
let (n, m) = (3, 4)
for i in 0..40:
    let (p, q) = (i + n, i + m)
    a[p % 5] += q
    s += bump(i)
    a[i % 5] += s
    print(s)
";
    let program = program(source);
    let config = TraceConfig {
        source: source.into(),
        record_frames: true,
        ..TraceConfig::default()
    };
    let io = || cpl_interp::Io::fixed(b"");
    let whole = cpl_interp::run_observed(&program, io(), &mut Vec::new(), Some(config.clone()));
    let whole = whole.recording.unwrap();
    let tail = cpl_interp::trace::tail(&program, b"", config.clone(), 25);
    let tail = tail.recording.unwrap();
    assert_eq!(tail.len(), 25);
    assert_eq!(tail.skipped, whole.len() - 25);
    // Variables are numbered in the order a recording first meets them.
    let named = |rec: &Recording, k: usize| {
        let step = rec.step(k).unwrap();
        let mut values: Vec<_> = (step.values.iter())
            .map(|(var, value)| (rec.vars[*var].name.clone(), value.to_string()))
            .collect();
        values.sort();
        (step.line, values, step.locals, step.frames)
    };
    for k in 0..25 {
        assert_eq!(
            named(&whole, tail.skipped + k),
            named(&tail, k),
            "step {k} of the window"
        );
    }
}

#[test]
fn a_traced_run_that_never_ends_stops_at_its_time_limit() {
    // One that writes, one the tree-walker runs (its condition calls),
    // one that writes nothing.
    for source in [
        "var i = 0\nwhile i >= 0:\n    i = (i + 1) % 2\nprint(i)\n",
        "fn f(x: Int) -> Int = x\nvar i = 0\nwhile f(i) >= 0:\n    i = i\nprint(i)\n",
        "var i = 0\nwhile i >= 0:\n    if i < 0: i = 1\nprint(i)\n",
    ] {
        stops_at_its_time_limit(source);
    }
}

fn stops_at_its_time_limit(source: &str) {
    let config = TraceConfig {
        source: source.to_string(),
        ..TraceConfig::default()
    };
    let started = Instant::now();
    let mut out = Vec::new();
    let observed = cpl_interp::run_observed(
        &program(source),
        cpl_interp::Io::fixed(b"").until(cpl_interp::Stop::after(
            std::time::Duration::from_millis(200),
        )),
        &mut out,
        Some(config),
    );
    let trap = observed.trap.map(|trap| trap.into_message());
    assert_eq!(trap.as_deref(), Some(cpl_interp::trace::OUT_OF_TIME.text()));
    assert!(started.elapsed().as_secs() < 5, "{:?}", started.elapsed());
}

const PUSHES: &str = "\
var xs: List<Int> = []
var bs: List<Bool> = []
for i in 0..300:
    xs.push(i * 3)
    bs.push(i % 2 == 0)
print(xs[299])
";

#[test]
fn a_list_rebuilt_from_its_pushes_holds_every_item_pushed() {
    let rec = record(PUSHES, "", TraceConfig::default());
    let var = |name: &str| rec.vars.iter().position(|v| v.id == name).expect("traced");
    let (xs, bs) = (var("xs"), var("bs"));
    let value = |k: usize, var: usize| {
        rec.step(k)
            .unwrap()
            .values
            .iter()
            .find(|(v, _)| *v == var)
            .map(|(_, v)| v.to_string())
    };
    let ints = |n: usize| {
        format!(
            "[{}]",
            (0..n)
                .map(|i| (i * 3).to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
    };
    let bools = |n: usize| {
        format!(
            "[{}]",
            (0..n)
                .map(|i| (i % 2 == 0).to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
    };
    let mut seen = (0, 0);
    for k in 0..rec.len() {
        if let Some(got) = value(k, xs) {
            while ints(seen.0) != got {
                seen.0 += 1;
                assert!(seen.0 <= 300, "step {k}: xs = {got}");
            }
        }
        if let Some(got) = value(k, bs) {
            while bools(seen.1) != got {
                seen.1 += 1;
                assert!(seen.1 <= 300, "step {k}: bs = {got}");
            }
        }
    }
    assert_eq!(seen, (300, 300));
}

#[test]
fn an_untraced_run_keeps_the_last_values_a_traced_one_does() {
    let failing = "var x = 5\nvar a: [0..3]Int = 0\nvar xs: List<Int> = []\nfor i in 0..400:\n    xs.push(i)\na[1] = x\nassert x < 0\n";
    for source in [CALLS, CELLS, PUSHES, failing] {
        let lives = |traced: bool| {
            let program = program(source);
            let io = cpl_interp::Io::fixed(b"");
            let observed = match traced {
                true => cpl_interp::run_observed(
                    &program,
                    io,
                    &mut Vec::new(),
                    Some(TraceConfig {
                        source: source.into(),
                        ..TraceConfig::default()
                    }),
                ),
                false => cpl_interp::run_observed(
                    &program,
                    io.with_last_values(source),
                    &mut Vec::new(),
                    None,
                ),
            };
            let rec = observed.recording.expect("kept");
            rec.lives
                .iter()
                .map(|live| format!("{} {} {}", live.line, live.name, live.value))
                .collect::<Vec<_>>()
        };
        let traced = lives(true);
        assert!(!traced.is_empty(), "{source}");
        assert_eq!(lives(false), traced, "{source}");
    }
}

/// The gate under the budgets above, at 10^6 steps so every run has it.
/// The bounds are today's cost with room for a busy machine and only go
/// down, toward the 1.2x the budgets ask for: the hooks with nothing kept
/// cost 2.1x an untraced run, recording every step 7x (10.6x unoptimized).
#[test]
fn tracing_stays_cheap_enough_to_leave_on() {
    let source = "var x = 0\nfor i in 0..1000000:\n    x += i % 7\nprint(x)\n";
    let program = program(source);
    let run = |config: Option<&TraceConfig>| timing::run(&program, b"", config);
    let hooked = TraceConfig {
        source: source.into(),
        only_fn: Some("not_a_fn".into()),
        ..TraceConfig::default()
    };
    let kept = TraceConfig {
        source: source.into(),
        record_frames: true,
        max_steps: Some(1_000_000),
        ..TraceConfig::default()
    };
    let times = timing::rounds(
        ROUNDS,
        &mut [&mut || run(None), &mut || run(Some(&hooked)), &mut || {
            run(Some(&kept))
        }],
    );
    let slower = |k: usize| times.slower(k, 0);
    let (observed, recorded) = (slower(1), slower(2));
    eprintln!("observed {observed:.2}x, recorded {recorded:.2}x an untraced run");
    let (hooks, recording) = if cfg!(debug_assertions) {
        (2.6, 13.0)
    } else {
        (2.6, 8.5)
    };
    assert!(
        observed <= hooks,
        "the hooks alone made the run {observed:.2}x slower (bound {hooks}x)"
    );
    assert!(
        recorded <= recording,
        "recording made the run {recorded:.2}x slower (bound {recording}x)"
    );
}
