//! `cpl stress`.

use crate::judge::{declared, Judge};
use crate::program::Program;
use crate::run::{save, stress_with_cold, Options, Report, Source};
use crate::spec::InputSpec;
use cpl_diag::Span;
use cpl_interp::trace::Divergence;
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
  --interactor PROG  judge by talking to `PROG <input>`; exit 0 accepts
  --cases N          how many cases to run (default 1000)
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
pub type Tracer<'a> = &'a dyn Fn(&Path, &str, bool) -> Result<Traced, String>;

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
        args,
        &[
            "--brute",
            "--fast",
            "--gen",
            "--checker",
            "--interactor",
            "--cases",
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
        (_, Some(path), _) => resolve(path).map(Judge::Interactor),
        // The interactor in the file is run by `cpl judge`.
        (None, None, Some(judge)) if judge.kind == "interactor" => {
            let cpl = std::env::current_exe().map_err(|e| {
                format!(
                    "can't find `cpl` to run the interactor `{}`: {e}",
                    judge.name
                )
            })?;
            Ok(Judge::Interactor(Program {
                name: format!("{name} (`{}`)", judge.name),
                argv: vec![
                    cpl.display().to_string(),
                    "judge".to_string(),
                    name.to_string(),
                ],
                env: Vec::new(),
            }))
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

/// The file with each `@brute(f) fn g` put in place of `fn f`: `g` takes
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

use cpl_json::escape_string as json_string;

/// The report in the shape the editor reads (`editors/vscode/README.md`).
pub fn report_json(
    report: &Report,
    saved: Option<&Path>,
    stages: &[Stage],
    trace: Option<&Path>,
    explained: Option<&Result<Option<Divergence>, String>>,
) -> String {
    let Some(failure) = &report.failure else {
        return format!("{{\"status\":\"ok\",\"cases\":{}}}\n", report.cases);
    };
    let stages: Vec<String> = stages
        .iter()
        .map(|stage| {
            format!(
                "{{\"name\":{},\"line\":{},\"fast\":{},\"brute\":{}}}",
                json_string(&stage.name),
                stage.line,
                json_string(&stage.fast),
                json_string(&stage.brute)
            )
        })
        .collect();
    let path = |path: Option<&Path>| {
        path.map_or("null".to_string(), |path| {
            json_string(&path.display().to_string())
        })
    };
    let side = |side: &Option<(u32, String)>| {
        side.as_ref().map_or("null".to_string(), |(line, value)| {
            format!("{{\"line\":{line},\"value\":{}}}", json_string(value))
        })
    };
    let explain = match explained {
        None => String::new(),
        Some(Ok(None)) => ",\"explain\":{\"divergence\":null}".to_string(),
        Some(Ok(Some(d))) => format!(
            ",\"explain\":{{\"divergence\":{{\"name\":{},\"nth\":{},\"fast\":{},\"brute\":{}{}}}}}",
            json_string(&d.name),
            d.nth,
            side(&d.fast),
            side(&d.brute),
            if d.inside.is_empty() {
                String::new()
            } else {
                let steps: Vec<_> = (d.inside.iter())
                    .map(|(line, locals)| {
                        format!("{{\"line\":{line},\"locals\":{}}}", json_string(locals))
                    })
                    .collect();
                format!(",\"inside\":[{}]", steps.join(","))
            }
        ),
        Some(Err(why)) => format!(",\"explain\":{{\"error\":{}}}", json_string(why)),
    };
    format!(
        "{{\"status\":\"fail\",\"cases\":{},\"counterexample\":{{\"case\":{},\"seed\":{},\"input\":{},\"raw_input\":{},\"expected\":{},\"got\":{},\"verdict\":{},\"stages\":[{}],\"saved\":{},\"trace\":{}{explain}}}}}\n",
        report.cases,
        failure.case,
        failure.seed,
        json_string(failure.described.as_deref().unwrap_or(failure.input.trim_end())),
        json_string(&failure.input),
        // A Windows build writes CRLF; the report reads the same on every OS.
        json_string(&failure.expected),
        json_string(&failure.got),
        json_string(failure.verdict.code()),
        stages.join(","),
        path(saved),
        path(trace),
    )
}

/// `1st`, `2nd`, `3rd`, `11th`.
/// Where the solution and its alt part ways, as `cpl stress` and
/// `cpl locate` print it.
pub fn print_divergence(stdout: &mut dyn Write, d: &cpl_interp::trace::Divergence) {
    let ordinal = ordinal(d.nth);
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
            let _ = write!(stderr, "error: {message}\n\n{USAGE}");
            return 2;
        }
    };
    let mut fail = |message: String| {
        let _ = writeln!(stderr, "error: {message}");
        1
    };
    let name = args.file.display().to_string();
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
    // The file's own `@alt(program)`, unless `--brute` names another.
    let own_brute = match (&args.brute, args.file.extension().and_then(|e| e.to_str())) {
        (None, Some("cp")) => {
            let mut db = cpl_check::Database::new();
            db.set_file(&name, src.as_str());
            db.analyze(&name)
                .and_then(|analysis| cpl_ir::brute(&analysis))
        }
        _ => None,
    };
    // The file's own judge has to build before it can rule on a case.
    if args.checker.is_none()
        && args.interactor.is_none()
        && args.file.extension().is_some_and(|e| e == "cp")
    {
        let mut db = cpl_check::Database::new();
        db.set_file(&name, src.as_str());
        let broken = db.analyze(&name).and_then(|analysis| {
            let lowered = match (cpl_ir::checker(&analysis), cpl_ir::interactor(&analysis)) {
                (Some((checker, _)), _) => cpl_ir::lower_checker(&analysis, &checker),
                (None, Some(judge)) => cpl_ir::lower_judge(&analysis, &judge),
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
    // `@brute(f) fn g`: the brute is the whole file with `g` in `f`'s place.
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
        && {
            let mut db = cpl_check::Database::new();
            db.set_file(&name, src.as_str());
            db.analyze(&name)
                .is_some_and(|analysis| cpl_ir::has_cached_fields(&analysis))
        };
    if args.brute.is_none()
        && own_brute.is_none()
        && part_brute.is_none()
        && !has_cached
        && judge == Judge::Tokens
    {
        return fail(format!(
            "there is nothing to compare {name} with: write an `@alt(program)` fn in it, pass `--brute <program>`, or give it a `@check` or an `@interactor` that can judge an output on its own"
        ));
    }
    // The file's own `@gen` fn, unless `--gen` names another.
    let own_gen = match (&args.gen, args.file.extension().and_then(|e| e.to_str())) {
        (None, Some("cp")) => {
            let mut db = cpl_check::Database::new();
            db.set_file(&name, src.as_str());
            db.analyze(&name)
                .and_then(|analysis| cpl_ir::generator(&analysis))
        }
        _ => None,
    };
    let source = match (&args.gen, &own_gen) {
        (Some(path), _) => match resolve(path) {
            Ok(program) => Source::Generator(program),
            Err(message) => return fail(message),
        },
        (None, Some((fn_name, _))) => {
            let size = i64::try_from(args.options.size).unwrap_or(i64::MAX);
            match Program::resolve_gen(&args.file, fn_name, size) {
                Ok(program) => Source::InFile(program),
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

    let report = match stress_with_cold(
        &source,
        &fast,
        brute.as_ref(),
        cold.as_ref(),
        &judge,
        &args.options,
    ) {
        Ok(report) => report,
        Err(error) => return fail(error.to_string()),
    };
    let saved = report
        .failure
        .as_ref()
        .and_then(|failure| save(&args.file, failure).ok());
    // The failing case again, both sides in the debug interpreter: the
    // `debug(...)`s they share, and with `--trace` the two traces.
    let in_file = own_brute.is_some()
        || part_brute.is_some()
        || (args.brute.is_none() && judge != Judge::Tokens);
    // Not a case the solution never finished: run again, it would only be stopped again.
    let finished =
        |failure: &crate::run::Failure| !matches!(failure.verdict, crate::Verdict::TimeLimit);
    let traced = match (&report.failure, tracer) {
        (Some(failure), Some(tracer)) if args.fast.is_none() && in_file && finished(failure) => {
            Some(tracer(&args.file, &failure.input, args.explain))
        }
        _ => None,
    };
    let (stages, html) = match &traced {
        Some(Ok(traced)) => (traced.stages.as_slice(), Some(traced.html.as_str())),
        _ => (&[][..], None),
    };
    let explained = match (&report.failure, &traced) {
        (Some(_), Some(Ok(traced))) if part_brute.is_some() && traced.divergence.is_some() => {
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
    if args.json {
        let _ = write!(
            stdout,
            "{}",
            report_json(
                &report,
                saved.as_deref(),
                stages,
                trace_path.as_deref(),
                explained.as_ref()
            )
        );
        return report.failure.is_some() as u8;
    }
    let judged_by = match &judge {
        Judge::Tokens => "checker: none".to_string(),
        Judge::Checker(program) => format!("checker: {}", program.name),
        Judge::Interactor(program) => format!("interactor: {}", program.name),
    };
    let _ = writeln!(
        stdout,
        "  brute: {}      {judged_by}",
        brute.as_ref().map_or("none", |b| b.name.as_str())
    );
    let Some(failure) = &report.failure else {
        let skipped = match report.skipped {
            0 => String::new(),
            n => format!(" ({n} seeds made no valid case)"),
        };
        let _ = writeln!(
            stdout,
            "  {} cases agree{skipped}",
            report.cases - report.skipped
        );
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
    let _ = writeln!(
        stdout,
        "  FAIL on case {} (seed {}): {}",
        failure.case,
        failure.seed,
        failure.verdict.code()
    );
    if let Some((from, to)) = failure.shrunk {
        let _ = writeln!(stdout, "  shrunk from {from} numbers to {to}:");
    }
    let _ = writeln!(stdout, "{}", indent(&failure.input));
    if let Some(described) = &failure.described {
        let _ = writeln!(stdout, "  that is: {described}");
    }
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
        }
        crate::Verdict::RuntimeError(why) => {
            match args
                .fast
                .is_none()
                .then(|| crate::explain::runtime_error(&args.file, &failure.input))
                .flatten()
            {
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
            }
        }
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
        let _ = writeln!(stdout, "  trace {}", path.display());
    }
    if let Some(note) = &trace_note {
        let _ = writeln!(stderr, "warning: {note}");
    }
    match saved {
        Some(path) => {
            let _ = writeln!(stdout, "  saved {}", path.display());
            if !part_alts(&src).is_empty() && explained.is_none() {
                let _ = writeln!(
                    stdout,
                    "  next: `cpl locate {}` names the first part whose alt disagrees",
                    args.file.display()
                );
            }
        }
        None => {
            let _ = writeln!(
                stderr,
                "warning: the case could not be saved next to {name}"
            );
        }
    }
    1
}

#[cfg(test)]
mod tests {
    use super::swap_in_part_brutes;
    use crate::judge::declared;

    #[test]
    fn a_recursive_brute_calls_itself_by_the_name_it_takes() {
        let src = "fn ways(n: Int) -> Int = n\n@brute(ways)\nfn slow(n: Int) -> Int = if n <= 1: 1 else: slow(n - 1) + slow(n - 2)\nprint(ways(read()))\n";
        let swapped = swap_in_part_brutes(src).expect("a pair");
        assert_eq!(
            swapped,
            "fn ways_cpl_fast(n: Int) -> Int = n\n            \nfn ways(n: Int) -> Int = if n <= 1: 1 else: ways(n - 1) + ways(n - 2)\nprint(ways(read()))\n"
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
