//! `cpl stress`.

use crate::judge::{declared, Judge, Launch};
use crate::program::Program;
use crate::run::{save, stress_with_cold, Options, Source};
use crate::spec::InputSpec;
use cpl_diag::Span;
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const USAGE: &str = "\
usage: cpl stress <file.cp> [options]

Runs the solution and a brute on random inputs made from the file's
`read()` types, stops at the first case they disagree on, shrinks it, and
saves it to <file>.tests/ as a regression test.

The brute, the checker and the interactor are the file's own
`@alt(program)`, `@check` and `@interactor` fns unless an option names a
program: a .cpp file (built with g++), a .py file, or an executable.

options:
  --brute PROG       the slow, obviously right solution to compare with
  --fast PROG        run this instead of building the file
  --gen PROG         make inputs with `PROG <seed> <size>` instead of from the
                     file's `@gen fn` or its types; such a case is not shrunk
  --checker PROG     judge with `PROG <input> <output> <answer>`; exit 0 accepts
  --interactor PROG  judge by talking to `PROG <input>`; exit 0 accepts; with a
                     `# interactor: spawn` comment, `PROG <input> -- <solution>`
                     runs the solution itself
  --cases N          how many cases to run (default 1000)
  --time T           stop starting cases after T (`30s`, `500ms`, `2m`); with
                     no --cases, run cases until then
  --size N           the largest size (n, q) a case uses (default 8)
  --seed N           the first seed (default 1)
  --time-limit MS    per run, in milliseconds (default 5000)
  --jobs N           cases run at once (default: one per core)
  --trace            on a failure, run the solution and the brute on the
                     shrunk case in the debug interpreter and save the two
                     traces side by side as <file>.tests/stress-<seed>.trace.html
                     (the solution's alone, when a `@check` is the judge)
  --explain          on a failure, run the solution and its `@alt(program)` fn on
                     the shrunk case in the debug interpreter and name the
                     first value they give a variable differently, with its line
  --format json      print the result as JSON, for an editor
";

/// One checkpoint both the solution and the brute reached on the failing
/// case: a `debug(...)` each of them ran.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Stage {
    pub name: String,
    /// 1-based line in the solution.
    pub line: u32,
    pub fast: String,
    pub brute: String,
}

/// The solution and its brute run side by side on one input.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Traced {
    pub stages: Vec<Stage>,
    /// The two traces as one self-contained page.
    pub html: String,
    /// With `--explain`, the first value the two give a variable differently.
    pub divergence: Option<cpl_interp::trace::Divergence>,
}

/// Runs a `.cp` file and its `@alt(program)` fn on an input in the debug
/// interpreter. `cpl` passes its own; this crate does not link one.
/// The `bool` asks for [`Traced::divergence`].
/// Traces the file on a failing input: the input, `--explain`, and whether
/// the solution ran out of time on it.
pub type Tracer<'a> = &'a dyn Fn(&Path, &str, bool, bool) -> Result<Traced, String>;

struct Args {
    file: PathBuf,
    brute: Option<PathBuf>,
    fast: Option<PathBuf>,
    gen: Option<PathBuf>,
    checker: Option<PathBuf>,
    interactor: Option<PathBuf>,
    options: Options,
    json: bool,
    trace: bool,
    explain: bool,
}

fn parse(args: &[String]) -> Result<Args, String> {
    let raw = crate::args::Args::parse(
        "stress",
        args,
        &[
            "--brute",
            "--fast",
            "--gen",
            "--checker",
            "--interactor",
            "--cases",
            "--time",
            "--size",
            "--max-size",
            "--seed",
            "--time-limit",
            "--jobs",
            "--threads",
            "--format",
        ],
        &["--trace", "--explain"],
    )?;
    let file = match raw.positional.as_slice() {
        [path] => PathBuf::from(path),
        [] => return Err("`stress` needs a file: `cpl stress d.cp`".to_string()),
        [_, extra, ..] => return Err(format!("`stress` takes one file; `{extra}` is a second")),
    };
    let mut options = Options::default();
    if let Some(n) = raw.number("--cases")? {
        options.cases = n as usize;
    }
    if let Some(text) = raw.value("--time") {
        options.budget = Some(budget(text)?);
        if raw.value("--cases").is_none() {
            options.cases = usize::MAX;
        }
    }
    if let Some(n) = raw.number("--size")?.or(raw.number("--max-size")?) {
        options.size = i128::from(n.max(1));
    }
    if let Some(n) = raw.number("--seed")? {
        options.seed = n;
    }
    if let Some(n) = raw.number("--time-limit")? {
        options.limit = Duration::from_millis(n);
    }
    if let Some(n) = raw.number("--jobs")?.or(raw.number("--threads")?) {
        options.jobs = n.max(1) as usize;
    }
    let json = match raw.value("--format") {
        None | Some("text") => false,
        Some("json") => true,
        Some(other) => return Err(format!("`--format` is text or json, not `{other}`")),
    };
    let checker = raw.value("--checker").map(PathBuf::from);
    let interactor = raw.value("--interactor").map(PathBuf::from);
    if checker.is_some() && interactor.is_some() {
        return Err("a problem has a checker or an interactor, not both".to_string());
    }
    Ok(Args {
        file,
        brute: raw.value("--brute").map(PathBuf::from),
        fast: raw.value("--fast").map(PathBuf::from),
        gen: raw.value("--gen").map(PathBuf::from),
        checker,
        interactor,
        options,
        json,
        trace: raw.has("--trace"),
        explain: raw.has("--explain"),
    })
}

/// The judge for a problem: the checker or interactor passed on the
/// command line, or token comparison. A file that declares a judge is
/// never compared token by token: that would call right answers wrong.
pub fn select_judge(
    src: &str,
    name: &str,
    checker: Option<&Path>,
    interactor: Option<&Path>,
) -> Result<Judge, String> {
    let resolve = |path: &Path| Program::resolve(path).map_err(|e| e.to_string());
    match (checker, interactor, declared(src).first()) {
        (Some(path), _, _) => resolve(path).map(Judge::Checker),
        (_, Some(path), _) => {
            resolve(path).map(|program| Judge::Interactor(program, Launch::declared(path)))
        }
        // The interactor in the file is run by `cpl judge`.
        (None, None, Some(judge)) if judge.kind == "interactor" => {
            let cpl = std::env::current_exe().map_err(|e| {
                format!(
                    "can't find `cpl` to run the interactor `{}`: {e}",
                    judge.name
                )
            })?;
            Ok(Judge::Interactor(
                Program {
                    name: format!("{name} (`{}`)", judge.name),
                    argv: vec![
                        cpl.display().to_string(),
                        "judge".to_string(),
                        name.to_string(),
                    ],
                    env: Vec::new(),
                },
                Launch::Stdio,
            ))
        }
        // The checker in the file is run by `cpl judge` too: it is
        // given the input, the output and the answer.
        (None, None, Some(judge)) => {
            let cpl = std::env::current_exe().map_err(|e| {
                format!("can't find `cpl` to run the checker `{}`: {e}", judge.name)
            })?;
            Ok(Judge::Checker(Program {
                name: format!("{name} (`{}`)", judge.name),
                argv: vec![
                    cpl.display().to_string(),
                    "judge".to_string(),
                    name.to_string(),
                ],
                env: Vec::new(),
            }))
        }
        (None, None, None) => Ok(Judge::Tokens),
    }
}

/// The program to run for `file`: `fast` when given, else the file
/// itself, built.
pub fn solution(file: &Path, fast: Option<&Path>) -> Result<Program, String> {
    Program::resolve(fast.unwrap_or(file)).map_err(|e| e.to_string())
}

/// The file with each `@alt(f) fn g` put in place of `fn f`: `g` takes
/// `f`'s name, `f` steps aside and the decorator is blanked. Lines stay
/// where they were, so `debug(...)` lines still pair up. `None` when the
/// file pairs no fn with a brute.
pub fn swap_in_part_brutes(src: &str) -> Option<String> {
    swap_in_part_brutes_of(src, None)
}

/// Each `(part, alt)` the file pairs, `@alt(part) fn alt`, in file order.
pub fn part_alts(src: &str) -> Vec<(String, String)> {
    pairs(src)
        .into_iter()
        .map(|(_, alt, part)| {
            (
                part.to_string(),
                src[alt.start as usize..alt.end as usize].to_string(),
            )
        })
        .collect()
}

/// The spans of each part pair's decorator and alt fn name, and the part.
fn pairs(src: &str) -> Vec<(Span, Span, &str)> {
    let parsed = cpl_syntax::parse_file(src);
    let mut fns: HashMap<&str, Span> = HashMap::new();
    let mut pairs = Vec::new();
    for (base, item) in &parsed.items {
        for stmt in &item.stmts {
            let cpl_syntax::ast::StmtKind::Fn(decl) = &stmt.kind else {
                continue;
            };
            fns.insert(decl.name.text.as_str(), decl.name.span.shift(*base));
            for decorator in stmt
                .decorators
                .iter()
                .filter(|d| !d.directive && matches!(d.name.text.as_str(), "brute" | "alt"))
            {
                if let Some(arg) = decorator.args.first().filter(|a| a.name.is_none()) {
                    if let cpl_syntax::ast::ExprKind::Name(part) = &arg.value.kind {
                        if part != "main" && part != "program" {
                            let part =
                                &src[arg.value.span.shift(*base).start as usize..][..part.len()];
                            pairs.push((
                                decorator.span.shift(*base),
                                decl.name.span.shift(*base),
                                part,
                            ));
                        }
                    }
                }
            }
        }
    }
    pairs.retain(|(_, _, part)| fns.contains_key(part));
    pairs
}

/// [`swap_in_part_brutes`], swapping only `only`'s alt where it is given.
pub(crate) fn swap_in_part_brutes_of(src: &str, only: Option<&str>) -> Option<String> {
    let mut fns: HashMap<&str, Span> = HashMap::new();
    for (base, item) in &cpl_syntax::parse_file(src).items {
        for stmt in &item.stmts {
            if let cpl_syntax::ast::StmtKind::Fn(decl) = &stmt.kind {
                fns.insert(
                    &src[decl.name.span.shift(*base).start as usize..][..decl.name.text.len()],
                    decl.name.span.shift(*base),
                );
            }
        }
    }
    let pairs = pairs(src)
        .into_iter()
        .filter(|(_, _, part)| only.is_none_or(|only| only == *part));
    let tokens = cpl_syntax::lex(src).tokens;
    let mut edits: Vec<(Span, String)> = Vec::new();
    for (decorator, brute, part) in pairs {
        let Some(&fast) = fns.get(part) else { continue };
        let blank = src[decorator.start as usize..decorator.end as usize]
            .chars()
            .map(|c| if c == '\n' { c } else { ' ' })
            .collect();
        edits.push((decorator, blank));
        edits.push((fast, format!("{part}_cpl_fast")));
        // The brute's own name is gone once it is renamed, so a brute that
        // recurses, or a caller of it, must use the name it takes.
        let name = &src[brute.start as usize..brute.end as usize];
        for token in tokens
            .iter()
            .filter(|t| t.kind == cpl_syntax::TokenKind::Ident)
        {
            if &src[token.span.start as usize..token.span.end as usize] == name {
                edits.push((token.span, part.to_string()));
            }
        }
    }
    if edits.is_empty() {
        return None;
    }
    edits.sort_by_key(|(span, _)| std::cmp::Reverse(span.start));
    let mut swapped = src.to_string();
    for (span, text) in edits {
        swapped.replace_range(span.start as usize..span.end as usize, &text);
    }
    Some(swapped)
}

/// Builds `swapped`, the file with its `@alt(f)` fns in place, from
/// beside `file` so its imports resolve the same way.
pub(crate) fn build_part_brute(file: &Path, name: &str, swapped: &str) -> Result<Program, String> {
    let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or("brute");
    // Written only to interpret it with no C++ compiler, so its name and
    // its C++ are the same on every run and
    // the build cache keeps it.
    let path = file.with_file_name(format!(".{stem}.brute.cp"));
    let mut program = Program::resolve_source(&path, swapped).map_err(|e| e.to_string())?;
    program.name = format!("{name} with its @alt fns swapped in");
    Ok(program)
}

use cpl_diag::Shown;

/// `1st`, `2nd`, `3rd`, `11th`.
/// Where the solution and its alt part ways, as `cpl stress` and
/// `cpl locate` print it.
pub fn print_divergence(stdout: &mut dyn Write, d: &cpl_interp::trace::Divergence) {
    let ordinal = ordinal(d.nth);
    if let (true, Some((line, held))) = (d.stuck, &d.fast) {
        let _ = writeln!(
            stdout,
            "  never finishes: `{}` at line {line} is back where it was every {}, at {held}",
            d.name,
            match d.nth {
                1 => "step".to_string(),
                n => format!("{n} steps"),
            }
        );
        for (line, why) in &d.blame {
            let _ = writeln!(stdout, "    look at line {line}: {why}");
        }
        return;
    }
    if let (Some((line, answer)), None, "answer") = (&d.fast, &d.brute, d.name.as_str()) {
        let _ = writeln!(
            stdout,
            "  the judge rejected the run after its last line:\n    solution `{answer}` at line {line}"
        );
        for (line, why) in &d.blame {
            let _ = writeln!(stdout, "    look at line {line}: {why}");
        }
        return;
    }
    let _ = match d.name.split_once('(').filter(|_| d.call) {
        Some((f, _)) => writeln!(
            stdout,
            "  first wrong call: `{}`, the {ordinal} call of `{f}`",
            d.name
        ),
        None => writeln!(
            stdout,
            "  first differing value: `{}`, its {ordinal} value",
            d.name
        ),
    };
    for (who, side) in [("solution", &d.fast), ("brute", &d.brute)] {
        let _ = match side {
            Some((line, value)) => {
                writeln!(stdout, "    {who:<8} {} = {value} at line {line}", d.name)
            }
            None => writeln!(
                stdout,
                "    {who:<8} never gives `{}` a {ordinal} value",
                d.name
            ),
        };
    }
    for (line, why) in &d.blame {
        let _ = writeln!(stdout, "    look at line {line}: {why}");
    }
    if !d.inside.is_empty() {
        let _ = writeln!(stdout, "    the solution's last steps in it:");
        for (line, locals) in &d.inside {
            let _ = writeln!(stdout, "      line {line}: {locals}");
        }
    }
}

fn ordinal(n: usize) -> String {
    let suffix = match (n % 10, n % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}

/// Whether a wrong answer is the toolchain's: a debug run of the file
/// neither traps nor differs from the brute, and the built C++ does
/// (stress/U29).
/// The judge profile the cases were built for, unless `--fast` gave a
/// program stress didn't build.
fn build_stamp(args: &Args, src: &str) -> Option<String> {
    args.fast.is_none().then_some(())?;
    cpl_judge::resolve(&args.file, src, None)
        .ok()
        .map(|profile| profile.stamp())
}

fn toolchain_bug(args: &Args, failure: &crate::run::Failure) -> bool {
    matches!(failure.verdict, crate::Verdict::Wrong(_))
        && args.fast.is_none()
        && crate::explain::runtime_error(&args.file, &failure.input).is_none()
        && crate::explain::interpreted_output(&args.file, &failure.input).is_some_and(|out| {
            crate::judge::toolchain_differs(&out, &failure.expected, &failure.got)
        })
}

fn indent(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut out: Vec<String> = lines
        .iter()
        .take(12)
        .map(|line| format!("    {line}"))
        .collect();
    if lines.len() > 12 {
        out.push(format!("    ... ({} more lines)", lines.len() - 12));
    }
    out.join("\n")
}

/// Runs `cpl stress`. Returns the exit code: 0 when every case agreed, 1
/// when one did not or the run could not start, 2 for a usage error.
pub fn run(args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    run_with(args, stdout, stderr, None)
}

/// [`run`], with the debug interpreter to trace a failure in.
pub fn run_with(
    args: &[String],
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    tracer: Option<Tracer>,
) -> u8 {
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        let _ = write!(stdout, "{USAGE}");
        return 0;
    }
    let args = match parse(args) {
        Ok(args) => args,
        Err(message) => {
            let _ = write!(stderr, "cpl: {message}\n\n{USAGE}");
            return 2;
        }
    };
    let mut fail = |message: String| {
        let _ = writeln!(stderr, "error: {message}");
        1
    };
    let name = args.file.shown();
    let src = match std::fs::read_to_string(&args.file) {
        Ok(src) => src,
        Err(e) => return fail(format!("can't read {name}: {e}")),
    };

    let resolve = |path: &Path| Program::resolve(path).map_err(|e| e.to_string());
    let judge = match select_judge(
        &src,
        &name,
        args.checker.as_deref(),
        args.interactor.as_deref(),
    ) {
        Ok(judge) => judge,
        Err(message) => return fail(message),
    };
    // The file, analyzed once for its own brute, judge, `@cache` and `@gen`.
    let analysis = match args.file.extension().and_then(|e| e.to_str()) {
        Some("cp") => {
            let mut db = cpl_check::Session::new();
            db.set_file(&name, src.as_str());
            db.analyze(&name)
        }
        _ => None,
    };
    // Each input a bound lint names, with the rest generated, runs before
    // the generated cases, since small cases never reach the bounds (use
    // case stress/U28): what the lint is, where a failing one is saved,
    // and the input or why there is none.
    let lines = cpl_diag::LineIndex::new(&src);
    let witnesses: Vec<(String, PathBuf, Result<String, String>)> = (analysis.as_deref())
        .filter(|_| args.fast.is_none())
        .map(|analysis| cpl_lint::lint_analysis(analysis, &cpl_lint::Config::default()))
        .unwrap_or_default()
        .iter()
        .filter(|d| !d.witness.is_empty())
        .map(|d| {
            let line = lines.line_col(d.span.start).line + 1;
            let path =
                (args.file.with_extension("tests")).join(format!("lint-{}-{line}.in", d.code.0));
            let input = InputSpec::witness_case(&src, &d.witness).map(|case| case.render());
            (format!("{} at line {line}", d.code.0), path, input)
        })
        .collect();
    // The file's own `@alt(program)`, unless `--brute` names another.
    let own_brute = match &args.brute {
        None => analysis.as_deref().and_then(cpl_ir::brute),
        Some(_) => None,
    };
    // The file's own judge has to build before it can rule on a case.
    if args.checker.is_none()
        && args.interactor.is_none()
        && args.file.extension().is_some_and(|e| e == "cp")
    {
        let broken = analysis.as_deref().and_then(|analysis| {
            let lowered = match (cpl_ir::checker(analysis), cpl_ir::interactor(analysis)) {
                (Some((checker, _)), _) => cpl_ir::lower_checker(analysis, &checker),
                (None, Some(judge)) => cpl_ir::lower_judge(analysis, &judge),
                (None, None) => return None,
            };
            let lines = cpl_diag::LineIndex::new(&src);
            lowered.err().map(|diagnostics| {
                diagnostics
                    .iter()
                    .map(|d| cpl_diag::render(d, &name, &src, &lines))
                    .collect::<String>()
            })
        });
        if let Some(errors) = broken {
            let _ = write!(stderr, "{errors}");
            return 1;
        }
    }
    // `@alt(f) fn g`: the brute is the whole file with `g` in `f`'s place.
    let part_brute = match (
        &args.brute,
        &own_brute,
        args.file.extension().and_then(|e| e.to_str()),
    ) {
        (None, None, Some("cp")) => swap_in_part_brutes(&src),
        _ => None,
    };
    let has_cached = args.fast.is_none()
        && args.file.extension().is_some_and(|e| e == "cp")
        && src.contains("@cache")
        && analysis.as_deref().is_some_and(cpl_ir::has_cached_fields);
    // With nothing to judge an answer by, a case still fails on a runtime
    // error or the time limit.
    let unjudged = args.brute.is_none()
        && own_brute.is_none()
        && part_brute.is_none()
        && !has_cached
        && judge == Judge::Tokens;
    // The file's own `@gen` fn, unless `--gen` names another.
    let own_gen = match &args.gen {
        None => analysis.as_deref().and_then(cpl_ir::generator),
        Some(_) => None,
    };
    let source = match (&args.gen, &own_gen) {
        (Some(path), _) => match resolve(path) {
            Ok(program) => Source::Generator(program),
            Err(message) => return fail(message),
        },
        (None, Some((fn_name, _))) => {
            let size = i64::try_from(args.options.size).unwrap_or(i64::MAX);
            match Program::interpreted_gen(&args.file, size) {
                Ok(Some(program)) => Source::Interpreted {
                    program: Box::new(program),
                    name: name.clone(),
                    lines: cpl_diag::LineIndex::new(&src),
                },
                Ok(None) => match Program::resolve_gen(&args.file, fn_name, size) {
                    Ok(program) => Source::InFile(
                        program,
                        Program::lowered_gen(&args.file, size).ok().map(|program| {
                            Box::new(crate::InterpChoices {
                                program,
                                name: format!("{name} (`{fn_name}`)"),
                                lines: cpl_diag::LineIndex::new(&src),
                                stack_limit: None,
                                limit: args.options.limit,
                            })
                        }),
                    ),
                    Err(e) => return fail(e.to_string()),
                },
                Err(e) => return fail(e.to_string()),
            }
        }
        (None, None) => match InputSpec::from_source(&src) {
            Ok(spec) if spec.is_empty() => {
                return fail(format!(
                    "{name} has no typed `read()` to make inputs from; write `let n: 1..=100 = read()`, a `@gen fn` that prints an input, or pass `--gen <program>`"
                ))
            }
            Ok(spec) => Source::Typed(spec),
            Err(error) => {
                let line = cpl_diag::LineIndex::new(&src).line_of(error.span.start) + 1;
                return fail(format!("{name}:{line}: {}", error.message));
            }
        },
    };
    // The two builds are the whole wait before the first case, so they
    // run side by side.
    let (fast, brute, cold) = std::thread::scope(|scope| {
        let brute = scope.spawn(|| match (&args.brute, &own_brute) {
            (Some(path), _) => resolve(path).map(Some),
            (None, Some((fn_name, _))) => Program::resolve_brute(&args.file, fn_name)
                .map(Some)
                .map_err(|e| e.to_string()),
            (None, None) => match &part_brute {
                Some(swapped) => build_part_brute(&args.file, &name, swapped).map(Some),
                None => Ok(None),
            },
        });
        let cold = scope.spawn(|| {
            if has_cached {
                Program::resolve_cold_cache(&args.file)
                    .map(Some)
                    .map_err(|e| e.to_string())
            } else {
                Ok(None)
            }
        });
        let fast = solution(&args.file, args.fast.as_deref());
        (
            fast,
            brute.join().expect("a build does not panic"),
            cold.join().expect("a build does not panic"),
        )
    });
    let fast = match fast {
        Ok(program) => program,
        Err(message) => return fail(message),
    };
    let brute = match brute {
        Ok(brute) => brute,
        Err(message) => return fail(message),
    };
    let cold = match cold {
        Ok(cold) => cold,
        Err(message) => return fail(message),
    };

    // The cases saved before run first (use case stress/U15), then the
    // bound lints' inputs.
    let mut first = saved_cases(&args.file);
    let from_files = first.len();
    first.extend(
        (witnesses.iter())
            .filter_map(|(_, path, input)| Some((0, path.clone(), input.clone().ok()?))),
    );
    let options = Options {
        saved: (first.iter())
            .map(|(seed, _, input)| (*seed, input.clone()))
            .collect(),
        ..args.options.clone()
    };
    let report = match stress_with_cold(
        &source,
        &fast,
        brute.as_ref(),
        cold.as_ref(),
        &judge,
        &options,
    ) {
        Ok(report) => report,
        Err(error) => return fail(error.to_string()),
    };
    // A case the brute, checker or interactor fails on is no test of the
    // solution: it isn't saved.
    let helper_failed = |failure: &crate::run::Failure| {
        matches!(
            failure.verdict,
            crate::Verdict::BruteError(_) | crate::Verdict::JudgeError(..)
        )
    };
    let saved = (report.failure.as_ref())
        .filter(|failure| !helper_failed(failure))
        .and_then(|failure| match failure.saved {
            true => {
                let (_, path, input) = &first[failure.case - 1];
                let kept = failure.case <= from_files
                    || path.exists()
                    || (path.parent()).is_some_and(|dir| std::fs::create_dir_all(dir).is_ok())
                        && std::fs::write(path, input).is_ok();
                kept.then(|| path.clone())
            }
            false => save(&args.file, failure).ok(),
        });
    // The failing case again, both sides in the debug interpreter: the
    // `debug(...)`s they share, and with `--trace` the two traces.
    let in_file = own_brute.is_some()
        || part_brute.is_some()
        || (args.brute.is_none() && judge != Judge::Tokens);
    // Not a case the solution never finished: run again, it would only be stopped again.
    let finished =
        |failure: &crate::run::Failure| !matches!(failure.verdict, crate::Verdict::TimeLimit);
    let traced = match (&report.failure, tracer) {
        (Some(failure), Some(tracer))
            if args.fast.is_none()
                && in_file
                && !helper_failed(failure)
                && (args.explain || finished(failure)) =>
        {
            Some(tracer(
                &args.file,
                &failure.input,
                args.explain,
                !finished(failure),
            ))
        }
        _ => None,
    };
    let (stages, html) = match &traced {
        Some(Ok(traced)) => (traced.stages.as_slice(), Some(traced.html.as_str())),
        _ => (&[][..], None),
    };
    let explained = match (&report.failure, &traced) {
        (Some(failure), _) if helper_failed(failure) => None,
        (Some(_), Some(Ok(traced)))
            if (part_brute.is_some()
                || (args.explain && matches!(judge, Judge::Interactor(..))))
                && traced.divergence.is_some() =>
        {
            Some(Ok(traced.divergence.clone()))
        }
        (Some(_), Some(Ok(traced)))
            if args.explain && (own_brute.is_some() || part_brute.is_some()) =>
        {
            Some(Ok(traced.divergence.clone()))
        }
        (Some(_), Some(Err(why))) if args.explain => Some(Err(why.clone())),
        (Some(_), _) if args.explain => Some(Err(
            "`--explain` runs the file against its own `@alt` fns".to_string(),
        )),
        _ => None,
    };
    let mut trace_note = None;
    let trace_path = match (args.trace, &report.failure, html) {
        (true, Some(failure), Some(html)) => {
            let path = args
                .file
                .with_extension("tests")
                .join(format!("stress-{}.trace.html", failure.seed));
            let written = std::fs::create_dir_all(path.parent().unwrap_or(Path::new(".")))
                .and_then(|()| std::fs::write(&path, html));
            match written {
                Ok(()) => Some(path),
                Err(e) => {
                    trace_note = Some(format!("the trace could not be saved: {e}"));
                    None
                }
            }
        }
        (true, Some(_), None) => {
            trace_note = Some(match &traced {
                Some(Err(why)) => format!("no trace: {why}"),
                _ => "no trace: `--trace` runs the file itself, against its own `@alt(program)` fn or under its own `@check`".to_string(),
            });
            None
        }
        _ => None,
    };
    // Where a debug run of the brute stops: its own `@alt(program)` fn, or
    // the file with its `@alt(f)` fns swapped in.
    let brute_trap = |input: &str| match (&own_brute, &part_brute) {
        _ if args.fast.is_some() => None,
        (Some(_), _) => crate::explain::trap_report(&args.file, input, true),
        (None, Some(swapped)) => {
            crate::explain::trap_report_in(&args.file, swapped, input, false, None)
        }
        (None, None) => None,
    };
    // The trap a debug run stops at, in the shape `cpl check --format json` gives.
    let trap_json = (report.failure.as_ref())
        .filter(|_| args.json && args.fast.is_none())
        .and_then(|failure| match failure.verdict {
            crate::Verdict::RuntimeError(_) => {
                crate::explain::trap_report(&args.file, &failure.input, false).map(|t| t.json())
            }
            crate::Verdict::BruteError(_) => brute_trap(&failure.input).map(|t| t.json()),
            crate::Verdict::JudgeError("interactor", _) => match &judge {
                Judge::Interactor(interactor, _) => crate::judge::interactor_trap_json(
                    &fast,
                    interactor,
                    &failure.input,
                    args.options.limit,
                )
                .and_then(|json| cpl_json::parse(&json).ok()),
                _ => None,
            },
            crate::Verdict::JudgeError("checker", _) => {
                std::fs::read_to_string(&args.file).ok().and_then(|text| {
                    crate::explain::checker_trap_in(
                        &args.file,
                        &text,
                        &failure.input,
                        &failure.got,
                        None,
                    )
                    .map(|t| t.json())
                })
            }
            _ => None,
        });
    if args.json {
        let toolchain_bug = (report.failure.as_ref()).is_some_and(|f| toolchain_bug(&args, f));
        let result = crate::StressResult {
            build: build_stamp(&args, &src),
            saved: saved.as_deref().map(|path| path.shown()),
            trace: trace_path.as_deref().map(|path| path.shown()),
            stages: stages.to_vec(),
            explained: explained.clone(),
            trap: trap_json,
            toolchain_bug,
            report: report.clone(),
        };
        let _ = write!(stdout, "{}", result.render());
        return report.failure.is_some() as u8;
    }
    let judged_by = match &judge {
        Judge::Tokens if unjudged => "checker: none; only runtime errors and the time limit fail a case, and an `@alt(program)` fn would check the answers".to_string(),
        Judge::Tokens => "checker: none".to_string(),
        Judge::Checker(program) => format!("checker: {}", program.name),
        Judge::Interactor(program, _) => format!("interactor: {}", program.name),
    };
    let _ = writeln!(
        stdout,
        "  brute: {}      {judged_by}",
        brute.as_ref().map_or("none", |b| b.name.as_str())
    );
    // The build the cases ran, so a judge's verdict can be laid beside it
    // (use case stress/U29).
    if let Some(stamp) = build_stamp(&args, &src) {
        let _ = writeln!(stdout, "  build: {stamp}; checked, as `cpl run`");
    }
    let Some(failure) = &report.failure else {
        let skipped = match report.skipped {
            0 => String::new(),
            n => format!(" ({n} seeds made no valid case)"),
        };
        let ran = report.cases - report.skipped;
        let skipped = match args.options.budget {
            Some(budget) if report.cases < args.options.cases => match budget.as_millis() {
                ms if ms % 1000 == 0 => format!("{skipped} in {}s", ms / 1000),
                ms => format!("{skipped} in {ms}ms"),
            },
            _ => skipped,
        };
        let again = match report.saved.min(from_files) {
            0 => String::new(),
            1 => "the saved case passes; ".to_string(),
            n => format!("the {n} saved cases pass; "),
        } + &match report.saved.saturating_sub(from_files) {
            0 => String::new(),
            1 => "the input `cpl lint` names at the bounds passes; ".to_string(),
            n => format!("the {n} inputs `cpl lint` names at the bounds pass; "),
        };
        let _ = match unjudged {
            true => writeln!(
                stdout,
                "  {again}{ran} cases ran with no runtime error or time limit{skipped}"
            ),
            false => writeln!(stdout, "  {again}{ran} cases agree{skipped}"),
        };
        let tried = report.reached.describe();
        if !tried.is_empty() {
            let _ = writeln!(stdout, "  tried: {tried}");
        }
        // A bound lint's input stress could not make is pointed at.
        let unmade: Vec<String> = (witnesses.iter())
            .filter_map(|(lint, _, input)| Some(format!("{lint}: {}", input.as_ref().err()?)))
            .collect();
        if !unmade.is_empty() {
            let _ = writeln!(
                stdout,
                "  cases reach size {}; `cpl lint` names inputs at the bounds that fail, which stress can't make ({})",
                args.options.size,
                unmade.join("; ")
            );
        }
        if report.discarded > 0 {
            let inputs = if report.discarded == 1 {
                "1 input"
            } else {
                &format!("{} inputs", report.discarded)
            };
            let _ = writeln!(
                stdout,
                "  {inputs} that stopped the program thrown away; the rest were made with {}",
                report.narrowed.join("; ")
            );
            let _ = writeln!(
                stdout,
                "  write the range in the program to test with it, like `let n: 1..=100 = read()`"
            );
        }
        return 0;
    };
    let _ = match (&saved, failure.saved) {
        (Some(path), true) if failure.case > from_files => writeln!(
            stdout,
            "  FAIL on the input `cpl lint` names at the bounds, saved as {}: {}",
            path.shown(),
            failure.verdict.code()
        ),
        (Some(path), true) => writeln!(
            stdout,
            "  FAIL on the saved case {}: {}",
            path.shown(),
            failure.verdict.code()
        ),
        _ => writeln!(
            stdout,
            "  FAIL on case {} (seed {}): {}",
            failure.case,
            failure.seed,
            failure.verdict.code()
        ),
    };
    if let Some((from, to)) = failure.shrunk {
        let _ = writeln!(stdout, "  shrunk from {from} numbers to {to}:");
    }
    let _ = writeln!(stdout, "{}", indent(&failure.input));
    if let Some(described) = &failure.described {
        let _ = writeln!(stdout, "  that is: {described}");
    }
    // A wrong answer or a crash of the built C++ can stop the debug
    // interpreter at its cause: an overflow, an index out of range.
    let trapped = (args.fast.is_none()
        && matches!(
            failure.verdict,
            crate::Verdict::Wrong(_) | crate::Verdict::RuntimeError(_)
        ))
    .then(|| crate::explain::runtime_error(&args.file, &failure.input))
    .flatten();
    match &failure.verdict {
        // A checker alone rules on the output: there is no answer to show.
        crate::Verdict::Wrong(why) if failure.expected.is_empty() => {
            let _ = writeln!(stdout, "  got:\n{}\n  {why}", indent(&failure.got));
        }
        crate::Verdict::Wrong(why) => {
            let _ = writeln!(
                stdout,
                "  expected:\n{}\n  got:\n{}\n  {why}",
                indent(&failure.expected),
                indent(&failure.got)
            );
            if let Some(explained) = &trapped {
                let _ = writeln!(
                    stdout,
                    "  a debug run of the solution stops:\n{}",
                    indent(explained.trim_end())
                );
            } else if toolchain_bug(&args, failure) {
                let _ = writeln!(
                    stdout,
                    "  toolchain bug: the debug interpreter prints the expected answer on this case\n  and the built C++ prints something else, so this is no WA of yours\n  `cpl report {}` writes the bug report",
                    args.file.shown()
                );
            }
        }
        crate::Verdict::BruteError(why) => {
            let brute_name = brute.as_ref().map_or("brute", |b| b.name.as_str());
            let trap = brute_trap(&failure.input).map(|t| t.text());
            let _ = match trap {
                Some(trap) => writeln!(
                    stdout,
                    "  the brute {brute_name} failed, so there is nothing to compare with:\n{}",
                    indent(trap.trim_end())
                ),
                None => writeln!(
                    stdout,
                    "  the brute {brute_name} failed, so there is nothing to compare with: {why}"
                ),
            };
            let _ = writeln!(
                stdout,
                "  fix the brute, or narrow the input it is meant for, then `cpl stress` again"
            );
        }
        crate::Verdict::JudgeError(kind, trap) => {
            let _ = writeln!(
                stdout,
                "  the {kind} failed, so the run can't be judged:\n{}",
                indent(trap.trim_end())
            );
            let _ = writeln!(stdout, "  fix the {kind}, then `cpl stress` again");
        }
        crate::Verdict::RuntimeError(why) => match &trapped {
            Some(explained) => {
                let _ = writeln!(
                    stdout,
                    "  the solution failed:\n{}",
                    indent(explained.trim_end())
                );
            }
            None => {
                let _ = writeln!(stdout, "  the solution failed: {why}");
            }
        },
        _ => {
            let _ = writeln!(stdout, "  the solution ran past the time limit");
        }
    }
    if !stages.is_empty() {
        let _ = writeln!(stdout, "  debug(...) both ran, solution | brute:");
        let first = stages.iter().position(|stage| stage.fast != stage.brute);
        for (index, stage) in stages
            .iter()
            .enumerate()
            .take(first.map_or(12, |first| first + 1))
        {
            let mark = if stage.fast == stage.brute { " " } else { "!" };
            let _ = writeln!(
                stdout,
                "  {mark} line {}: {}: {} | {}",
                stage.line, stage.name, stage.fast, stage.brute
            );
            if Some(index) == first {
                let _ = writeln!(stdout, "    ^ the first one that differs");
            }
        }
    }
    match &explained {
        // The trap above is the explanation, and where it stopped the
        // solution gave no more values.
        Some(Ok(Some(d))) if trapped.is_some() && d.fast.is_none() => {}
        Some(Ok(None)) if trapped.is_some() => {}
        Some(Ok(Some(d))) => print_divergence(stdout, d),
        Some(Ok(None)) => {
            let _ = writeln!(
                stdout,
                "  first differing value: none; no variable both name takes different values"
            );
        }
        Some(Err(why)) => {
            let _ = writeln!(stderr, "warning: no explanation: {why}");
        }
        None => {}
    }
    if let Some(path) = &trace_path {
        let _ = writeln!(stdout, "  trace {}", path.shown());
    }
    if let Some(note) = &trace_note {
        let _ = writeln!(stderr, "warning: {note}");
    }
    match saved {
        Some(path) => {
            if !failure.saved {
                let _ = writeln!(stdout, "  saved {}", path.shown());
            }
            let file = args.file.shown();
            if !part_alts(&src).is_empty() && explained.is_none() {
                let _ = writeln!(
                    stdout,
                    "  next: `cpl locate {file}` names the first part whose alt disagrees"
                );
            } else if trace_path.is_none() {
                let _ = writeln!(
                    stdout,
                    "  next: `cpl run {file} --trace` steps through this case"
                );
            }
            let _ = writeln!(
                stdout,
                "  after a fix: `cpl test {file}` reruns it, then `cpl stress {file}` again"
            );
        }
        None if !helper_failed(failure) => {
            let _ = writeln!(
                stderr,
                "warning: the case could not be saved next to {name}"
            );
        }
        None => {}
    }
    1
}

/// The cases `cpl stress` saved for `file` before, by seed.
/// `--time`'s value: `30s`, `500ms`, `2m`, or seconds alone.
fn budget(text: &str) -> Result<Duration, String> {
    let (number, unit) = text.split_at(
        text.find(|c: char| !c.is_ascii_digit())
            .unwrap_or(text.len()),
    );
    let bad = || format!("`--time` is a duration like `30s`, `500ms` or `2m`, not `{text}`");
    let number: u64 = number.parse().map_err(|_| bad())?;
    match unit {
        "ms" => Ok(Duration::from_millis(number)),
        "" | "s" => Ok(Duration::from_secs(number)),
        "m" => Ok(Duration::from_secs(number * 60)),
        _ => Err(bad()),
    }
}

fn saved_cases(file: &Path) -> Vec<(u64, PathBuf, String)> {
    let Ok(dir) = std::fs::read_dir(file.with_extension("tests")) else {
        return Vec::new();
    };
    let mut saved: Vec<(u64, PathBuf, String)> = (dir.flatten())
        .filter_map(|entry| {
            let name = entry.file_name().into_string().ok()?;
            let seed = name.strip_prefix("stress-")?.strip_suffix(".in")?;
            Some((
                seed.parse().ok()?,
                entry.path(),
                std::fs::read_to_string(entry.path()).ok()?,
            ))
        })
        .collect();
    saved.sort();
    saved
}

#[cfg(test)]
mod tests {
    use super::swap_in_part_brutes;
    use crate::judge::declared;

    #[test]
    fn a_recursive_brute_calls_itself_by_the_name_it_takes() {
        let src = "fn ways(n: Int) -> Int = n\n@alt(ways)\nfn slow(n: Int) -> Int = if n <= 1: 1 else: slow(n - 1) + slow(n - 2)\nprint(ways(read()))\n";
        let swapped = swap_in_part_brutes(src).expect("a pair");
        assert_eq!(
            swapped,
            "fn ways_cpl_fast(n: Int) -> Int = n\n          \nfn ways(n: Int) -> Int = if n <= 1: 1 else: ways(n - 1) + ways(n - 2)\nprint(ways(read()))\n"
        );
    }

    #[test]
    fn alt_and_check_decorators_work_like_brute_and_checker() {
        let part = "fn ways(n: Int) -> Int = n\n@alt(ways, n: ..=8)\nfn slow(n: Int) -> Int = n + 1\nprint(ways(read()))\n";
        let swapped = swap_in_part_brutes(part).expect("an alt pair");
        assert!(swapped.contains("fn ways_cpl_fast("), "{swapped}");
        assert!(
            swapped.contains("fn ways(n: Int) -> Int = n + 1"),
            "{swapped}"
        );

        let whole = "@alt(program, n: ..=10)\nfn brute():\n    print(0)\n";
        assert!(swap_in_part_brutes(whole).is_none());

        let checked = "@check\nfn ok():\n    0\n";
        let judges = declared(checked);
        assert_eq!(judges.len(), 1);
        assert_eq!(judges[0].kind, "checker");
        assert_eq!(judges[0].name, "ok");
    }
}
