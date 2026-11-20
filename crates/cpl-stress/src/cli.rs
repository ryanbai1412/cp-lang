//! `cpl stress`.

use crate::judge::{declared, Judge};
use crate::program::Program;
use crate::run::{save, stress, Options, Report, Source};
use crate::spec::InputSpec;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const USAGE: &str = "\
usage: cpl stress <file.cp> --brute <program> [options]

Runs the solution and a brute on random inputs made from the file's
`read()` types, stops at the first case they disagree on, shrinks it, and
saves it to <file>.tests/ as a regression test.

A program is a .cpp file (built with g++), a .py file, or an executable.

options:
  --brute PROG       the slow, obviously right solution to compare with
  --fast PROG        the built solution (default: <file>.cpp next to the source)
  --gen PROG         make inputs with `PROG <seed> <size>` instead of from types
  --checker PROG     judge with `PROG <input> <output> <answer>`; exit 0 accepts
  --interactor PROG  judge by talking to `PROG <input>`; exit 0 accepts
  --cases N          how many cases to run (default 1000)
  --size N           the largest size (n, q) a case uses (default 8)
  --seed N           the first seed (default 1)
  --time-limit MS    per run, in milliseconds (default 5000)
  --jobs N           cases run at once (default: one per core)
  --format json      print the result as JSON, for an editor
";

struct Args {
    file: PathBuf,
    brute: Option<PathBuf>,
    fast: Option<PathBuf>,
    gen: Option<PathBuf>,
    checker: Option<PathBuf>,
    interactor: Option<PathBuf>,
    options: Options,
    json: bool,
}

fn parse(args: &[String]) -> Result<Args, String> {
    let mut file = None;
    let mut parsed = Args {
        file: PathBuf::new(),
        brute: None,
        fast: None,
        gen: None,
        checker: None,
        interactor: None,
        options: Options::default(),
        json: false,
    };
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        let mut value = || {
            iter.next()
                .cloned()
                .ok_or_else(|| format!("`{arg}` needs a value"))
        };
        let number = |text: String| {
            text.parse::<u64>()
                .map_err(|_| format!("`{arg}` takes a whole number, not `{text}`"))
        };
        match arg.as_str() {
            "--brute" => parsed.brute = Some(PathBuf::from(value()?)),
            "--fast" => parsed.fast = Some(PathBuf::from(value()?)),
            "--gen" => parsed.gen = Some(PathBuf::from(value()?)),
            "--checker" => parsed.checker = Some(PathBuf::from(value()?)),
            "--interactor" => parsed.interactor = Some(PathBuf::from(value()?)),
            "--cases" => parsed.options.cases = number(value()?)? as usize,
            "--size" => parsed.options.size = i128::from(number(value()?)?.max(1)),
            "--seed" => parsed.options.seed = number(value()?)?,
            "--time-limit" => parsed.options.limit = Duration::from_millis(number(value()?)?),
            "--jobs" => parsed.options.jobs = number(value()?)?.max(1) as usize,
            "--format" => match value()?.as_str() {
                "json" => parsed.json = true,
                "text" => parsed.json = false,
                other => return Err(format!("`--format` is text or json, not `{other}`")),
            },
            flag if flag.starts_with('-') => return Err(format!("unknown option `{flag}`")),
            path if file.is_none() => file = Some(PathBuf::from(path)),
            extra => return Err(format!("`stress` takes one file; `{extra}` is a second")),
        }
    }
    parsed.file = file.ok_or("`stress` needs a file: `cpl stress d.cp --brute brute.cpp`")?;
    if parsed.checker.is_some() && parsed.interactor.is_some() {
        return Err("a problem has a checker or an interactor, not both".to_string());
    }
    Ok(parsed)
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
        (None, None, Some(judge)) => Err(format!(
            "{name} declares the {kind} `{fn_name}` (line {line}), which cpl can't build yet; pass it built with `--{kind} <program>`",
            kind = judge.kind,
            fn_name = judge.name,
            line = judge.line
        )),
        (None, None, None) => Ok(Judge::Tokens),
    }
}

/// The program to run for `file`: `fast` when given, else the `.cpp`
/// built next to it, else the file itself.
pub fn solution(file: &Path, fast: Option<&Path>) -> Result<Program, String> {
    let path = fast.map(Path::to_path_buf).unwrap_or_else(|| {
        let built = file.with_extension("cpp");
        if built.is_file() {
            built
        } else {
            file.to_path_buf()
        }
    });
    Program::resolve(&path).map_err(|e| e.to_string())
}

/// `text` as a JSON string, quotes included.
pub fn json_string(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The report in the shape the editor reads (`editors/vscode/README.md`).
fn json(report: &Report, saved: Option<&Path>) -> String {
    let Some(failure) = &report.failure else {
        return format!("{{\"status\":\"ok\",\"cases\":{}}}\n", report.cases);
    };
    format!(
        "{{\"status\":\"fail\",\"cases\":{},\"counterexample\":{{\"case\":{},\"seed\":{},\"input\":{},\"raw_input\":{},\"expected\":{},\"got\":{},\"verdict\":{},\"stages\":[],\"saved\":{}}}}}\n",
        report.cases,
        failure.case,
        failure.seed,
        json_string(failure.described.as_deref().unwrap_or(failure.input.trim_end())),
        json_string(&failure.input),
        json_string(&failure.expected),
        json_string(&failure.got),
        json_string(failure.verdict.code()),
        saved.map_or("null".to_string(), |path| json_string(&path.display().to_string())),
    )
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
    if args.brute.is_none() && judge == Judge::Tokens {
        return fail(format!(
            "there is nothing to compare {name} with: pass `--brute <program>`, or a `--checker` or `--interactor` that can judge an output on its own"
        ));
    }
    let source = match &args.gen {
        Some(path) => match resolve(path) {
            Ok(program) => Source::Generator(program),
            Err(message) => return fail(message),
        },
        None => match InputSpec::from_source(&src) {
            Ok(spec) if spec.fields.is_empty() => {
                return fail(format!(
                    "{name} has no typed `read()` to make inputs from; write `let n: 1..=100 = read()`, or pass `--gen <program>`"
                ))
            }
            Ok(spec) => Source::Typed(spec),
            Err(error) => {
                let line = cpl_diag::LineIndex::new(&src).line_of(error.span.start) + 1;
                return fail(format!("{name}:{line}: {}", error.message));
            }
        },
    };
    let fast = match solution(&args.file, args.fast.as_deref()) {
        Ok(program) => program,
        Err(message) => return fail(message),
    };
    let brute = match args.brute.as_deref().map(resolve).transpose() {
        Ok(brute) => brute,
        Err(message) => return fail(message),
    };

    let report = match stress(&source, &fast, brute.as_ref(), &judge, &args.options) {
        Ok(report) => report,
        Err(error) => return fail(error.to_string()),
    };
    let saved = report
        .failure
        .as_ref()
        .and_then(|failure| save(&args.file, failure).ok());
    if args.json {
        let _ = write!(stdout, "{}", json(&report, saved.as_deref()));
        return report.failure.is_some() as u8;
    }
    let judged_by = match &judge {
        Judge::Tokens => "none".to_string(),
        Judge::Checker(program) | Judge::Interactor(program) => program.name.clone(),
    };
    let _ = writeln!(
        stdout,
        "  brute: {}      checker: {judged_by}",
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
        crate::Verdict::Wrong(why) => {
            let _ = writeln!(
                stdout,
                "  expected:\n{}\n  got:\n{}\n  {why}",
                indent(&failure.expected),
                indent(&failure.got)
            );
        }
        crate::Verdict::RuntimeError(why) => {
            let _ = writeln!(stdout, "  the solution failed: {why}");
        }
        _ => {
            let _ = writeln!(stdout, "  the solution ran past the time limit");
        }
    }
    match saved {
        Some(path) => {
            let _ = writeln!(stdout, "  saved {}", path.display());
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
