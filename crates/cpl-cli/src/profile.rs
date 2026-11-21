//! `cpl profile` and `cpl cover`: where a run spends its time, and which
//! lines and branches a set of inputs never reaches. Both read the same
//! per-statement tally, from the interpreter or from the C++ built with a
//! counter at each statement.

use crate::{compile, Compiled, EXIT_ERRORS, EXIT_FAILED, EXIT_USAGE};
use cpl_diag::{LineIndex, Span};
use cpl_interp::profile::Site;
use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::path::{Path, PathBuf};

pub const PROFILE_USAGE: &str = "\
usage: cpl profile <file.cp> [input] [options]

Runs the program on the input file, or standard input, and shows each line
with how many times it ran and the share of the time spent there. Time in
library code is the time of the line that called it. The program's own
output is not shown.

options:
  --native         profile the C++ of `cpl build`, built with a counter at
                   each statement (with the flags of the file's judge, on its stack); the counts are the
                   interpreter's, the times are the judge's
  --format json    one JSON object: the lines with their hits and nanoseconds
";

pub const COVER_USAGE: &str = "\
usage: cpl cover <file.cp> [input]... [options]

Runs the program on every input and lists the lines no input reached and
each `if` that went only one way. A directory stands for its `.in` files;
no input at all means the cases in `<file>.tests/`.

options:
  --native         run the C++ of `cpl build` instead of the interpreter
  --format json    one JSON object: covered and uncovered lines, branches
";

struct Options<'a> {
    file: &'a str,
    inputs: Vec<&'a str>,
    native: bool,
    json: bool,
}

fn options<'a>(rest: &[&'a str], usage: &str, stdout: &mut dyn Write) -> Result<Options<'a>, u8> {
    let mut options = Options {
        file: "",
        inputs: Vec::new(),
        native: false,
        json: false,
    };
    let mut files = Vec::new();
    let mut args = rest.iter();
    while let Some(arg) = args.next() {
        match *arg {
            "-h" | "--help" => {
                let _ = write!(stdout, "{usage}");
                return Err(0);
            }
            "--native" => options.native = true,
            "--format" => match args.next() {
                Some(&"json") => options.json = true,
                Some(&"text") => options.json = false,
                _ => return Err(EXIT_USAGE),
            },
            flag if flag.starts_with('-') && flag != "-" => return Err(EXIT_USAGE),
            file => files.push(file),
        }
    }
    let Some((file, inputs)) = files.split_first() else {
        return Err(EXIT_USAGE);
    };
    options.file = file;
    options.inputs = inputs.to_vec();
    Ok(options)
}

/// One run's tally, by span, and how it ended when not well.
struct Run {
    sites: HashMap<Span, Site>,
    failure: Option<String>,
}

fn run(file: &str, compiled: &Compiled, input: &[u8], native: bool) -> Result<Run, String> {
    if native {
        return run_native(file, compiled, input);
    }
    let mut sink = Vec::new();
    let observed =
        cpl_interp::run_profiled(&compiled.program, cpl_interp::Io::fixed(input), &mut sink);
    Ok(Run {
        sites: observed
            .profile
            .unwrap_or_default()
            .sites
            .into_iter()
            .collect(),
        failure: observed.trap.map(|trap| trap.message),
    })
}

/// The program built as the file's judge builds it, with the judge's flags
/// this compiler accepts, and run on the judge's stack.
fn run_native(file: &str, compiled: &Compiled, input: &[u8]) -> Result<Run, String> {
    let judge = cpl_judge::resolve(Path::new(file), &compiled.text, None)?;
    let target = cpl_codegen::Target {
        stamp: Some(judge.stamp()),
        thread_stack: judge.thread_stack,
        windows: judge.os == cpl_judge::Os::Windows,
    };
    let (cpp, spans) = cpl_codegen::emit_profiled(&compiled.program, "profile", &target);
    let dir = std::env::temp_dir().join(format!("cpl-profile-{}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|e| format!("can't make `{}`: {e}", dir.display()))?;
    let (source, binary, tally) = (
        dir.join("profile.cpp"),
        dir.join("profile"),
        dir.join("profile.txt"),
    );
    std::fs::write(&source, cpp).map_err(|e| format!("can't write `{}`: {e}", source.display()))?;
    let host = judge.host();
    let compiler = std::env::var("CXX").unwrap_or(host.compiler);
    let built = std::process::Command::new(&compiler)
        .args(&host.flags)
        .arg("-o")
        .arg(&binary)
        .arg(&source)
        .output()
        .map_err(|e| {
            format!("`--native` needs a C++ compiler, and `{compiler}` did not start: {e}")
        })?;
    if !built.status.success() {
        let errors = String::from_utf8_lossy(&built.stderr);
        return Err(format!(
            "{compiler} could not build the profiled C++:\n{}",
            errors.lines().take(20).collect::<Vec<_>>().join("\n")
        ));
    }
    let _ = std::fs::remove_file(&tally);
    let argv = judge.on_stack(binary.display().to_string());
    let mut child = std::process::Command::new(&argv[0])
        .args(&argv[1..])
        .env("CPL_PROFILE", &tally)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("can't run the profiled build: {e}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(input);
    }
    let output = child
        .wait_with_output()
        .map_err(|e| format!("can't run the profiled build: {e}"))?;
    let failure = (!output.status.success()).then(|| {
        let said = String::from_utf8_lossy(&output.stderr).trim().to_string();
        match output.status.code() {
            Some(code) if !said.is_empty() => format!("{said} (exit status {code})"),
            Some(code) => format!("exit status {code}"),
            None => "the program crashed".to_string(),
        }
    });
    let text = std::fs::read_to_string(&tally).unwrap_or_default();
    let _ = std::fs::remove_dir_all(&dir);
    let mut sites: HashMap<Span, Site> = HashMap::new();
    for line in text.lines() {
        let numbers: Vec<u64> = line.split(' ').filter_map(|n| n.parse().ok()).collect();
        if let [id, hits, nanos] = numbers[..] {
            if let Some(span) = spans.get(id as usize) {
                let site = sites.entry(*span).or_default();
                site.hits += hits;
                site.nanos += nanos;
            }
        }
    }
    Ok(Run { sites, failure })
}

/// A line's tally: the most any of its statements ran, and their time.
#[derive(Clone, Copy, Default)]
struct Line {
    hits: u64,
    nanos: u64,
}

fn by_line(index: &LineIndex, sites: &HashMap<Span, Site>) -> BTreeMap<u32, Line> {
    let mut lines: BTreeMap<u32, Line> = BTreeMap::new();
    for (span, site) in sites {
        let line = lines.entry(index.line_of(span.start) + 1).or_default();
        line.hits = line.hits.max(site.hits);
        line.nanos += site.nanos;
    }
    lines
}

fn read_input(path: Option<&str>) -> Result<Vec<u8>, String> {
    match path {
        None | Some("-") => {
            let mut input = Vec::new();
            std::io::Read::read_to_end(&mut std::io::stdin(), &mut input)
                .map_err(|e| format!("can't read standard input: {e}"))?;
            Ok(input)
        }
        Some(path) => std::fs::read(path).map_err(|e| format!("can't read `{path}`: {e}")),
    }
}

fn json_string(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn millis(nanos: u64) -> String {
    match nanos {
        0..1_000 => format!("{nanos} ns"),
        1_000..1_000_000 => format!("{:.1} us", nanos as f64 / 1e3),
        _ => format!("{:.1} ms", nanos as f64 / 1e6),
    }
}

/// `cpl profile`.
pub fn profile(rest: &[&str], stdout: &mut dyn Write, stderr: &mut String) -> u8 {
    let options = match options(rest, PROFILE_USAGE, stdout) {
        Ok(options) => options,
        Err(0) => return 0,
        Err(code) => {
            *stderr += &format!(
                "cpl: `profile` takes a file and an input, or standard input\n\n{PROFILE_USAGE}"
            );
            return code;
        }
    };
    if options.inputs.len() > 1 {
        *stderr += &format!("cpl: `profile` runs one input\n\n{PROFILE_USAGE}");
        return EXIT_USAGE;
    }
    let compiled = match compile(options.file, stderr) {
        Ok(compiled) => compiled,
        Err(code) => return code,
    };
    let input = match read_input(options.inputs.first().copied()) {
        Ok(input) => input,
        Err(message) => {
            *stderr += &format!("cpl: {message}\n");
            return EXIT_USAGE;
        }
    };
    let run = match run(options.file, &compiled, &input, options.native) {
        Ok(run) => run,
        Err(message) => {
            *stderr += &format!("cpl: {message}\n");
            return EXIT_ERRORS;
        }
    };
    let index = LineIndex::new(&compiled.text);
    let lines = by_line(&index, &run.sites);
    let total: u64 = lines.values().map(|line| line.nanos).sum();
    let engine = if options.native {
        "native"
    } else {
        "interpreter"
    };
    if options.json {
        let rows: Vec<String> = lines
            .iter()
            .map(|(line, tally)| {
                format!(
                    "{{\"line\":{line},\"hits\":{},\"nanos\":{}}}",
                    tally.hits, tally.nanos
                )
            })
            .collect();
        let failure = run.failure.as_deref().map_or("null".into(), json_string);
        let _ = writeln!(
            stdout,
            "{{\"file\":{},\"engine\":\"{engine}\",\"nanos\":{total},\"failure\":{failure},\"lines\":[{}]}}",
            json_string(options.file),
            rows.join(",")
        );
    } else {
        let _ = writeln!(
            stdout,
            "profile of {} ({engine}): {}",
            options.file,
            millis(total)
        );
        let _ = writeln!(
            stdout,
            "{:>5} {:>12} {:>10} {:>4}",
            "line", "hits", "time", "%"
        );
        for (number, text) in compiled.text.lines().enumerate() {
            let number = number as u32 + 1;
            match lines.get(&number) {
                Some(line) => {
                    let share = (line.nanos * 100).checked_div(total).unwrap_or(0);
                    let _ = writeln!(
                        stdout,
                        "{number:>5} {:>12} {:>10} {share:>3}%  {text}",
                        line.hits,
                        millis(line.nanos)
                    );
                }
                None => {
                    let _ = writeln!(
                        stdout,
                        "{number:>5} {:>12} {:>10} {:>4}  {text}",
                        "", "", ""
                    );
                }
            }
        }
    }
    match run.failure {
        Some(message) => {
            *stderr += &format!("cpl: the run failed: {message}\n");
            EXIT_FAILED
        }
        None => 0,
    }
}

/// The inputs `cpl cover` runs: the files named, a directory's `.in`
/// files, or the problem's cases.
fn cover_inputs(file: &str, inputs: &[&str]) -> Vec<PathBuf> {
    if inputs.is_empty() {
        return cpl_contest::problem::cases(Path::new(file))
            .into_iter()
            .map(|case| case.input)
            .collect();
    }
    let mut paths = Vec::new();
    for input in inputs {
        let path = PathBuf::from(input);
        match std::fs::read_dir(&path) {
            Ok(entries) => {
                let mut found: Vec<PathBuf> = entries
                    .filter_map(|entry| entry.ok().map(|e| e.path()))
                    .filter(|path| path.extension().is_some_and(|e| e == "in"))
                    .collect();
                found.sort();
                paths.extend(found);
            }
            Err(_) => paths.push(path),
        }
    }
    paths
}

/// An `if` that a run reached but sent only one way.
struct Branch {
    line: u32,
    taken: bool,
}

/// The `if`s that went one way: from how often each began and how often
/// its first statement began.
fn one_way(compiled: &Compiled, index: &LineIndex, sites: &HashMap<Span, Site>) -> Vec<Branch> {
    let counted = cpl_ir::sites::user_stmts(&compiled.program);
    let hits = |stmt: &cpl_ir::Stmt| {
        stmt.span()
            .and_then(|span| sites.get(&span))
            .map_or(0, |site| site.hits)
    };
    let mut found = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let program = &compiled.program;
    let mut blocks: Vec<&cpl_ir::Block> = vec![&program.setup, &program.main];
    blocks.extend(program.fns.iter().map(|f| &f.body));
    blocks.extend(program.closures.iter().map(|c| &c.body));
    for block in blocks {
        cpl_ir::walk::deep_stmts(block, &mut |stmt| {
            let cpl_ir::Stmt::If(_, then, _) = stmt else {
                return;
            };
            let (Some(span), Some(first)) = (stmt.span(), then.stmts.first()) else {
                return;
            };
            if !counted.contains(&(stmt as *const cpl_ir::Stmt))
                || first.span().is_none()
                || !seen.insert(span)
            {
                return;
            }
            let (ran, entered) = (hits(stmt), hits(first));
            if ran > 0 && (entered == 0 || entered == ran) {
                found.push(Branch {
                    line: index.line_of(span.start) + 1,
                    taken: entered > 0,
                });
            }
        });
    }
    found.sort_by_key(|branch| branch.line);
    found
}

/// `cpl cover`.
pub fn cover(rest: &[&str], stdout: &mut dyn Write, stderr: &mut String) -> u8 {
    let options = match options(rest, COVER_USAGE, stdout) {
        Ok(options) => options,
        Err(0) => return 0,
        Err(code) => {
            *stderr += &format!("cpl: `cover` takes a file and its inputs\n\n{COVER_USAGE}");
            return code;
        }
    };
    let compiled = match compile(options.file, stderr) {
        Ok(compiled) => compiled,
        Err(code) => return code,
    };
    let inputs = cover_inputs(options.file, &options.inputs);
    if inputs.is_empty() {
        *stderr += &format!(
            "cpl: no inputs: name them, or put cases in `{}`\n",
            cpl_contest::problem::tests_dir(Path::new(options.file)).display()
        );
        return EXIT_USAGE;
    }
    let mut sites: HashMap<Span, Site> = HashMap::new();
    let mut failed = Vec::new();
    for input in &inputs {
        let bytes = match std::fs::read(input) {
            Ok(bytes) => bytes,
            Err(e) => {
                *stderr += &format!("cpl: can't read `{}`: {e}\n", input.display());
                return EXIT_USAGE;
            }
        };
        let run = match run(options.file, &compiled, &bytes, options.native) {
            Ok(run) => run,
            Err(message) => {
                *stderr += &format!("cpl: {message}\n");
                return EXIT_ERRORS;
            }
        };
        if let Some(message) = run.failure {
            failed.push(format!("{}: {message}", input.display()));
        }
        for (span, site) in run.sites {
            let total = sites.entry(span).or_default();
            total.hits += site.hits;
            total.nanos += site.nanos;
        }
    }
    let index = LineIndex::new(&compiled.text);
    let mut lines: BTreeMap<u32, bool> = BTreeMap::new();
    for span in cpl_ir::sites::user_spans(&compiled.program) {
        lines.entry(index.line_of(span.start) + 1).or_default();
    }
    for (number, line) in by_line(&index, &sites) {
        if line.hits > 0 {
            lines.insert(number, true);
        }
    }
    let covered = lines.values().filter(|hit| **hit).count();
    let uncovered: Vec<u32> = lines
        .iter()
        .filter(|(_, hit)| !**hit)
        .map(|(line, _)| *line)
        .collect();
    let branches = one_way(&compiled, &index, &sites);
    let text: Vec<&str> = compiled.text.lines().collect();
    let source = |line: u32| text.get(line as usize - 1).map_or("", |text| text.trim());
    if options.json {
        let list = |lines: &[u32]| {
            lines
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(",")
        };
        let branches: Vec<String> = branches
            .iter()
            .map(|b| format!("{{\"line\":{},\"always\":{}}}", b.line, b.taken))
            .collect();
        let _ = writeln!(
            stdout,
            "{{\"file\":{},\"inputs\":{},\"lines\":{},\"covered\":{covered},\"uncovered\":[{}],\"one_way\":[{}]}}",
            json_string(options.file),
            inputs.len(),
            lines.len(),
            list(&uncovered),
            branches.join(",")
        );
    } else {
        let percent = if lines.is_empty() {
            100
        } else {
            covered * 100 / lines.len()
        };
        let _ = writeln!(
            stdout,
            "{}: {covered} of {} lines covered ({percent}%) by {} input{}",
            options.file,
            lines.len(),
            inputs.len(),
            if inputs.len() == 1 { "" } else { "s" }
        );
        for line in &uncovered {
            let _ = writeln!(stdout, "  line {line:>4}  never ran     {}", source(*line));
        }
        for branch in &branches {
            let way = if branch.taken {
                "always true "
            } else {
                "always false"
            };
            let _ = writeln!(
                stdout,
                "  line {:>4}  {way}  {}",
                branch.line,
                source(branch.line)
            );
        }
    }
    for failure in &failed {
        *stderr += &format!("cpl: failed on {failure}\n");
    }
    0
}
