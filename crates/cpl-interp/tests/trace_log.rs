//! The recording is a log of changes with a snapshot now and then: any
//! step rebuilt from it is the step the run took.

use cpl_interp::trace::{Recording, TraceConfig};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

/// Counts the bytes in use, so the budget is what the recording holds,
/// not what it estimates.
struct Counting;

static IN_USE: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        IN_USE.fetch_add(layout.size(), Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        IN_USE.fetch_sub(layout.size(), Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        IN_USE.fetch_add(size, Ordering::Relaxed);
        IN_USE.fetch_sub(layout.size(), Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, size) }
    }
}

#[global_allocator]
static COUNTING: Counting = Counting;

fn program(source: &str) -> cpl_ir::Program {
    let mut db = cpl_check::Database::new();
    db.set_file("t.cp", source);
    let analysis = db.analyze("t.cp").expect("the file was just set");
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    cpl_ir::lower(&analysis).expect("lowers")
}

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

/// The budgets of a debug session at 10^7 steps: stepping back answers
/// in under 50 ms, recording makes the run at most 1.2x slower, and the
/// recording takes a few hundred MB. `cargo test --release -p cpl-interp
/// --test trace_log -- --ignored --nocapture`.
#[test]
#[ignore = "takes a minute in release; run by hand"]
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
        let started = Instant::now();
        let mut out = Vec::new();
        cpl_interp::run_observed(&program, cpl_interp::Io::fixed(b""), &mut out, None);
        let plain = started.elapsed();
        // The tracer runs on the tree-walking engine: the same run, observed but not recorded.
        let observed = TraceConfig {
            source: source.into(),
            only_fn: Some("not_a_fn".into()),
            ..TraceConfig::default()
        };
        let started = Instant::now();
        cpl_interp::run_observed(
            &program,
            cpl_interp::Io::fixed(b""),
            &mut out,
            Some(observed),
        );
        let walked = started.elapsed();
        let config = TraceConfig {
            source: source.into(),
            record_frames: true,
            max_steps: Some(10_000_000),
            ..TraceConfig::default()
        };
        out.clear();
        let before = IN_USE.load(Ordering::Relaxed);
        let started = Instant::now();
        let observed =
            cpl_interp::run_observed(&program, cpl_interp::Io::fixed(b""), &mut out, Some(config));
        let traced = started.elapsed();
        let rec = observed.recording.unwrap();
        let held = IN_USE.load(Ordering::Relaxed) - before;
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
        let slower = traced.as_secs_f64() / plain.as_secs_f64();
        let recording = traced.as_secs_f64() / walked.as_secs_f64();
        eprintln!(
            "{name}: {} steps; untraced {plain:?}, observed {walked:?}, recorded {traced:?}: {slower:.2}x the untraced run, {recording:.2}x the observed one; {mb:.0} MB; step back at most {back:?}",
            rec.len()
        );
        assert_eq!(rec.len(), 10_000_000);
        if back.as_millis() >= 50 {
            over.push(format!("{name}: step back took {back:?}"));
        }
        if slower > 1.2 {
            over.push(format!("{name}: tracing made the run {slower:.2}x slower"));
        }
        if mb >= 500.0 {
            over.push(format!("{name}: the recording takes {mb:.0} MB"));
        }
    }
    // The debugging demos on the inputs `cpl stress` pins, each run many
    // times, since one run takes microseconds.
    let demos = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/debugging/");
    for (name, input) in [
        ("knapsack", "1 2\n1 1\n"),
        ("shortest_path", "2 2\n1 2 2\n1 2 1\n"),
        ("shows", "3\n0 2\n0 1\n1 1\n"),
    ] {
        let source = std::fs::read_to_string(format!("{demos}{name}.cp")).unwrap();
        let program = program(&source);
        let mut out = Vec::new();
        let mut time = |trace: bool| {
            let started = Instant::now();
            for _ in 0..2000 {
                let config = trace.then(|| TraceConfig {
                    source: source.clone(),
                    record_frames: true,
                    ..TraceConfig::default()
                });
                let io = cpl_interp::Io::fixed(input.as_bytes());
                cpl_interp::run_observed(&program, io, &mut out, config);
            }
            started.elapsed()
        };
        let (plain, traced) = (time(false), time(true));
        let slower = traced.as_secs_f64() / plain.as_secs_f64();
        eprintln!(
            "{name}: 2000 runs; untraced {plain:?}, recorded {traced:?}: \
             {slower:.2}x the untraced run"
        );
        if slower > 1.2 {
            over.push(format!("{name}: tracing made the run {slower:.2}x slower"));
        }
    }
    assert!(over.is_empty(), "over budget: {over:?}");
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
