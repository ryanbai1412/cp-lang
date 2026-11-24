//! `cpl bench`: the solution on the largest inputs the problem allows.
//!
//! Sample tests are small; a judge's are not. The input types say how
//! large an input can be, so the worst cases can be made without writing
//! a generator: every size at its maximum, with values that are random,
//! all equal, sorted, reversed or alternating, and edge lists shaped as a
//! path and a star.

use crate::problem::{source_path, Limits};
use crate::test::ready;
use cpl_diag::Shown;
use cpl_json::escape_string as json_string;
use cpl_stress::args::Args;
use cpl_stress::{fit, make, Case, Exit, InputSpec, Pick, Program, Rng, Shape, TreeShape, Value};
use std::io::Write;
use std::time::Duration;

pub const USAGE: &str = "\
usage: cpl bench <problem> [options]
       cpl bench --zero-cost [dir] [--reps N] [--max-ratio R] [--format json]
       cpl bench --corpus [--runs N] [--ports a,b] [--out PATH] [--no-write]

Runs the solution on maximum-size inputs made from the problem's `read()`
types, in the shapes that break slow or deep solutions, and compares time
and memory with the problem's limits.

options:
  --zero-cost        run the zero-cost pairs (abstract.cp against direct.cp)
  --corpus           benchmark corpus/problems/* against the original C++
                     (p50/p95 + perf.cp)
  --fast PROG        run this instead of building the problem
  --time-limit MS    default: the problem's `// limits:` line, or 2000
  --memory-limit MB  default: the problem's `// limits:` line, or 256
  --seed N           the seed of the random shape (default 1)
  --format json      print the result as JSON
";

/// How the values of a worst case are chosen. Sizes are always maximal.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WorstShape {
    Random,
    AllMin,
    AllMax,
    Ascending,
    Descending,
    Alternating,
    /// Edge lists as one long chain: deep recursion.
    Path,
    /// Edge lists around one centre: one huge adjacency list.
    Star,
}

impl WorstShape {
    pub const ALL: [WorstShape; 8] = [
        WorstShape::Random,
        WorstShape::AllMin,
        WorstShape::AllMax,
        WorstShape::Ascending,
        WorstShape::Descending,
        WorstShape::Alternating,
        WorstShape::Path,
        WorstShape::Star,
    ];

    pub fn name(self) -> &'static str {
        match self {
            WorstShape::Random => "random",
            WorstShape::AllMin => "all smallest",
            WorstShape::AllMax => "all largest",
            WorstShape::Ascending => "ascending",
            WorstShape::Descending => "descending",
            WorstShape::Alternating => "alternating",
            WorstShape::Path => "path",
            WorstShape::Star => "star",
        }
    }
}

struct Worst {
    shape: WorstShape,
    rng: Rng,
    /// How many values have been drawn.
    drawn: i128,
    /// Remaining element budget across nested size picks so problems with
    /// `sum(s) <= 2e5` (e.g. `m = 5e4` queries each declaring `s: 2..=1e5`)
    /// cannot multiply into billions of elements and OOM.
    size_budget: i128,
}

const WORST_SIZE_BUDGET: i128 = 250_000;

impl Pick for Worst {
    fn pick(&mut self, lo: i128, hi: i128, is_size: bool) -> i128 {
        if is_size {
            if self.drawn == 0 && self.size_budget == WORST_SIZE_BUDGET {
                let chosen = hi.min(WORST_SIZE_BUDGET).max(lo);
                self.size_budget = (self.size_budget - chosen.max(0)).max(0);
                return chosen;
            }
            if self.size_budget <= 0 {
                return lo;
            }
            let chosen = hi.min(self.size_budget).max(lo);
            self.size_budget = (self.size_budget - chosen.max(0)).max(0);
            return chosen;
        }
        let index = self.drawn;
        self.drawn += 1;
        match self.shape {
            WorstShape::Random => self.rng.range(lo, hi),
            WorstShape::AllMin | WorstShape::Path | WorstShape::Star => lo,
            WorstShape::AllMax => hi,
            WorstShape::Ascending => lo + index.min(hi - lo),
            WorstShape::Descending => hi - index.min(hi - lo),
            WorstShape::Alternating if index % 2 == 0 => lo,
            WorstShape::Alternating => hi,
        }
    }

    fn tree(&mut self) -> TreeShape {
        match self.shape {
            WorstShape::Path => TreeShape::Path,
            WorstShape::Star => TreeShape::Star,
            _ => TreeShape::Random,
        }
    }
}

/// True for `[..](a, b)`: a list of pairs, which is how edges are read.
fn is_edge_list(shape: &Shape) -> bool {
    matches!(shape, Shape::Array { elem, .. }
        if matches!(&**elem, Shape::Tuple { parts, .. }
            if parts.len() >= 2 && parts.iter().all(|p| matches!(p, Shape::Int { .. }))))
}

/// Maximum nested array/sequence depth in an `InputSpec`, used to scale
/// `Worst::size_budget` so 2D/3D tables (`[N][N]Int` with unbounded `n: Int`)
/// stay within ~250k total elements instead of allocating `N^2` and OOMing.
fn shape_array_depth(shape: &Shape) -> u32 {
    match shape {
        Shape::Array { elem, .. } => 1 + shape_array_depth(elem),
        Shape::Tuple { parts, .. } => parts.iter().map(shape_array_depth).max().unwrap_or(0),
        _ => 0,
    }
}

fn spec_size_budget(spec: &InputSpec) -> i128 {
    let depth = spec
        .fields
        .iter()
        .map(|f| shape_array_depth(&f.shape))
        .max()
        .unwrap_or(1);
    match depth {
        0 | 1 => i128::MAX,
        2 => 500,
        _ => 64,
    }
}

/// The worst case of one shape. `None` when the spec has nothing that
/// shape applies to, or the case cannot be made.
pub fn worst_case(spec: &InputSpec, shape: WorstShape, seed: u64) -> Option<Case> {
    let budget = spec_size_budget(spec);
    let mut pick = Worst {
        shape,
        rng: Rng::new(seed),
        drawn: 0,
        size_budget: budget,
    };
    let mut case = make(spec, &mut pick)?;
    if !matches!(shape, WorstShape::Path | WorstShape::Star) {
        return Some(case);
    }
    // A tree, and the tree a connected graph starts from, are made in
    // the shape already.
    let mut any = spec
        .fields
        .iter()
        .any(|field| matches!(field.shape, Shape::Tree { .. } | Shape::Graph { .. }));
    for (field, value) in spec.fields.iter().zip(&mut case.values) {
        let (true, Value::Seq(edges)) = (is_edge_list(&field.shape), value) else {
            continue;
        };
        any = true;
        for (index, edge) in edges.iter_mut().enumerate() {
            let Value::Tuple(ends) = edge else { continue };
            // Every end starts at its lowest value: vertex 1 (or 0).
            if let [Value::Int(u), Value::Int(v), ..] = ends.as_mut_slice() {
                let index = index as i128;
                if shape == WorstShape::Path {
                    *u += index;
                }
                *v += index + 1;
            }
        }
    }
    // Fitting clamps an end that ran past the last vertex.
    any.then(|| fit(spec, &case)).flatten()
}

/// One benchmark run.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Run {
    pub shape: WorstShape,
    /// How many numbers the input has.
    pub numbers: usize,
    pub elapsed: Duration,
    pub peak_kb: Option<u64>,
    /// `ok`, `TLE`, `MLE` or `RE`.
    pub status: &'static str,
    /// Why a run failed (`RE`): the last line it printed to standard error,
    /// or the signal that stopped it.
    pub error: Option<String>,
}

/// Runs the solution on every worst case the spec has.
pub fn bench(
    spec: &InputSpec,
    solution: &Program,
    limits: Limits,
    seed: u64,
) -> std::io::Result<Vec<Run>> {
    let Limits { time, memory_kb } = limits;
    let mut runs = Vec::new();
    for shape in WorstShape::ALL {
        let Some(case) = worst_case(spec, shape, seed) else {
            continue;
        };
        let input = case.render();
        // Twice the limit, to say how far over a slow solution is.
        let outcome = solution.run(&input, time * 2)?;
        let status = match outcome.exit {
            Exit::TimedOut => "TLE",
            _ if outcome.peak_kb.is_some_and(|kb| kb > memory_kb) => "MLE",
            Exit::Code(0) if outcome.elapsed > time => "TLE",
            Exit::Code(0) => "ok",
            Exit::Code(_) | Exit::Signal => "RE",
        };
        let error = (status == "RE").then(|| crash_reason(&outcome));
        runs.push(Run {
            shape,
            numbers: case.weight().0,
            elapsed: outcome.elapsed,
            peak_kb: outcome.peak_kb,
            status,
            error,
        });
    }
    Ok(runs)
}

/// Why a run that did not exit cleanly failed.
fn crash_reason(outcome: &cpl_stress::Outcome) -> String {
    let last = outcome
        .stderr
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .map(str::trim);
    match (&outcome.exit, last) {
        (_, Some(line)) => line.to_string(),
        (Exit::Code(code), None) => format!("exit code {code}"),
        _ => "killed by a signal (often a stack overflow from deep recursion)".to_string(),
    }
}

fn megabytes(kb: Option<u64>) -> String {
    kb.map_or("?".to_string(), |kb| format!("{}", kb.div_ceil(1024)))
}

pub fn render(runs: &[Run], limits: Limits) -> String {
    let mut out = String::new();
    for run in runs {
        out.push_str(&format!(
            "  {:<13} {:>9} numbers  {:<3} {:>6} ms  {:>5} MB\n",
            run.shape.name(),
            run.numbers,
            run.status,
            run.elapsed.as_millis(),
            megabytes(run.peak_kb)
        ));
    }
    let slowest = runs.iter().max_by_key(|run| run.elapsed);
    let largest = runs.iter().filter_map(|run| run.peak_kb).max();
    if let Some(slowest) = slowest {
        let limit = limits.time.as_millis().max(1);
        out.push_str(&format!(
            "  slowest: {} at {} ms, {}% of the {limit} ms limit\n",
            slowest.shape.name(),
            slowest.elapsed.as_millis(),
            slowest.elapsed.as_millis() * 100 / limit
        ));
    }
    if let Some(largest) = largest {
        out.push_str(&format!(
            "  memory: {} MB of {} MB\n",
            megabytes(Some(largest)),
            limits.memory_kb / 1024
        ));
    }
    let mut crashes: Vec<(&str, &str)> = Vec::new();
    for run in runs.iter().filter(|run| run.status == "RE") {
        let why = run.error.as_deref().unwrap_or("no reason given");
        if !crashes.iter().any(|(_, seen)| *seen == why) {
            crashes.push((run.shape.name(), why));
        }
    }
    for (shape, why) in &crashes {
        out.push_str(&format!("  runtime error on {shape}: {why}\n"));
    }
    let crashed = runs.iter().filter(|run| run.status == "RE").count();
    let over = runs
        .iter()
        .filter(|run| run.status != "ok" && run.status != "RE")
        .count();
    let shapes = |n: usize| {
        if n == 1 {
            "1 shape".to_string()
        } else {
            format!("{n} shapes")
        }
    };
    out.push_str(&match (over, crashed) {
        (0, 0) => "  within limits on every shape\n".to_string(),
        (0, c) => format!("  {} crashed\n", shapes(c)),
        (o, 0) => format!(
            "  {} over the limits\n",
            if o == 1 {
                "1 shape is".to_string()
            } else {
                format!("{o} shapes are")
            }
        ),
        (o, c) => format!("  {} over the limits, {} crashed\n", shapes(o), shapes(c)),
    });
    out
}

fn json(runs: &[Run], limits: Limits) -> String {
    let rows: Vec<String> = runs
        .iter()
        .map(|run| {
            format!(
                "{{\"shape\":{},\"number_count\":{},\"status\":{},\"time_ms\":{},\"peak_kb\":{},\"error\":{}}}",
                json_string(run.shape.name()),
                run.numbers,
                json_string(run.status),
                run.elapsed.as_millis(),
                run.peak_kb.map_or("null".to_string(), |kb| kb.to_string()),
                run.error.as_deref().map_or("null".to_string(), json_string)
            )
        })
        .collect();
    format!(
        "{{\"status\":{},\"time_limit_ms\":{},\"memory_limit_kb\":{},\"runs\":[{}]}}\n",
        json_string(if runs.iter().all(|run| run.status == "ok") {
            "ok"
        } else {
            "fail"
        }),
        limits.time.as_millis(),
        limits.memory_kb,
        rows.join(",")
    )
}

/// Runs `cpl bench`. Returns the exit code: 0 when every shape is within
/// the limits.
pub fn run(args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        let _ = write!(stdout, "{USAGE}");
        return 0;
    }
    if args.iter().any(|arg| arg == "--zero-cost") {
        return crate::zero_cost::run(args, stdout, stderr);
    }
    if args.iter().any(|arg| arg == "--corpus") {
        return crate::corpus_bench::run(args, stdout, stderr);
    }
    let parsed = Args::parse(
        "bench",
        args,
        &[
            "--fast",
            "--time-limit",
            "--memory-limit",
            "--seed",
            "--format",
        ],
        &[],
    )
    .and_then(|args| {
        let source = args.file("bench").map(source_path)?;
        let as_json = match args.value("--format") {
            None | Some("text") => false,
            Some("json") => true,
            Some(other) => return Err(format!("`--format` is text or json, not `{other}`")),
        };
        let seed = args.number("--seed")?.unwrap_or(1);
        Ok((args, source, as_json, seed))
    });
    let (args, source, as_json, seed) = match parsed {
        Ok(parsed) => parsed,
        Err(message) => {
            let _ = write!(stderr, "cpl: {message}\n\n{USAGE}");
            return 2;
        }
    };
    let name = source.shown();
    let result = std::fs::read_to_string(&source)
        .map_err(|e| format!("can't read {name}: {e}"))
        .and_then(|src| {
            let spec = InputSpec::from_source(&src).map_err(|error| {
                let line = src[..error.span.start as usize].matches('\n').count() + 1;
                format!("{name}:{line}: {}", error.message.replace("pass a generator with `--gen`", "`cpl bench` needs typed input"))
            })?;
            if spec.is_empty() {
                return Err(format!(
                    "{name} has no typed `read()` to make inputs from; write `let n: 1..=2e5 = read()`"
                ));
            }
            let ready = ready(&source, &src, &args)?;
            let runs = bench(&spec, &ready.solution, ready.limits, seed).map_err(|e| format!("can't run {}: {e}", ready.solution.name))?;
            if runs.is_empty() {
                return Err(format!(
                    "no maximum-size input could be made for {name}: a range is empty or an array would have more than {} elements",
                    cpl_stress::gen::MAX_LEN
                ));
            }
            Ok((runs, ready.limits))
        });
    match result {
        Ok((runs, limits)) => {
            let text = if as_json {
                json(&runs, limits)
            } else {
                render(&runs, limits)
            };
            let _ = write!(stdout, "{text}");
            runs.iter().any(|run| run.status != "ok") as u8
        }
        Err(message) => {
            let _ = writeln!(stderr, "cpl: {message}");
            1
        }
    }
}
