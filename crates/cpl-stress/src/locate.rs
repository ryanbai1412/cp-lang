//! `cpl locate`: on a failing input, the first part whose alt disagrees.

use crate::cli::{build_part_brute, part_alts, swap_in_part_brutes_of};
use crate::program::{Outcome, Program};
use cpl_diag::Shown;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const USAGE: &str = "\
usage: cpl locate <file.cp> [input] [--json]

Runs the file on the input, then again with each `@alt(f)` fn put in place
of `f`, one part at a time, and names the first part whose alt prints
something else. The input is a file; without one it is the newest
<file>.tests/stress-*.in, the case `cpl stress` saved.

options:
  --json   print one JSON object: the `input`, the `part` and its `alt`
           (both null when every part agrees), and what the program and
           the program with that alt printed (`output`, `alt_output`),
           and the first wrong call of the part as `cpl stress --json`
           gives it under `explain` (`divergence`, null when none)

Exit status: 0 when every part agrees, 1 when one disagrees or the file
does not build, 2 on a usage error.
";

const LIMIT: Duration = Duration::from_secs(10);

/// Where `file` and its alt for one part first part ways on an input.
#[derive(Debug, PartialEq, Eq)]
pub struct Located {
    pub part: String,
    pub alt: String,
    pub output: String,
    pub alt_output: String,
}

pub fn run(args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    run_with(args, stdout, stderr, None)
}

/// [`run`], naming the wrong call of the part with `tracer` when given.
pub fn run_with(
    args: &[String],
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    tracer: Option<crate::cli::Tracer>,
) -> u8 {
    let json = args.iter().any(|a| a == "--json");
    let rest: Vec<&str> = args
        .iter()
        .map(String::as_str)
        .filter(|a| *a != "--json")
        .collect();
    if let Some(flag) = rest.iter().find(|a| a.starts_with('-')) {
        let message = cpl_diag::usage::no_option("locate", flag);
        let _ = write!(stderr, "cpl: {message}\n\n{USAGE}");
        return 2;
    }
    let (file, input) = match rest.as_slice() {
        [file] => (Path::new(file), None),
        [file, input] => (Path::new(file), Some(PathBuf::from(input))),
        _ => {
            let _ = write!(stderr, "cpl: `locate` takes a file and an input\n\n{USAGE}");
            return 2;
        }
    };
    let Some(input) = input.or_else(|| newest_saved_case(file)) else {
        let _ = writeln!(
            stderr,
            "cpl: no input: pass one, or run `cpl stress {}` to save a failing case",
            file.shown()
        );
        return 1;
    };
    let text = match std::fs::read_to_string(&input) {
        Ok(text) => text,
        Err(error) => {
            let _ = writeln!(stderr, "cpl: can't read {}: {error}", input.shown());
            return 1;
        }
    };
    let located = match locate(file, &text) {
        Ok(located) => located,
        Err(message) => {
            let _ = writeln!(stderr, "cpl: {message}");
            return 1;
        }
    };
    let shown = input.shown();
    let divergence = || {
        let traced = tracer.and_then(|trace| trace(file, &text, false).ok());
        traced.and_then(|traced| traced.divergence)
    };
    if json {
        let field = |v: Option<&String>| v.map_or("null".into(), |v| cpl_json::escape_string(v));
        let _ = writeln!(
            stdout,
            "{{\"input\":{},\"part\":{},\"alt\":{},\"output\":{},\"alt_output\":{},\"divergence\":{}}}",
            cpl_json::escape_string(&shown),
            field(located.as_ref().map(|l| &l.part)),
            field(located.as_ref().map(|l| &l.alt)),
            field(located.as_ref().map(|l| &l.output)),
            field(located.as_ref().map(|l| &l.alt_output)),
            (located.as_ref())
                .and_then(|_| divergence())
                .map_or("null".into(), |d| crate::cli::divergence_json(&d)),
        );
    } else {
        match &located {
            None => {
                let _ = writeln!(stdout, "every part agrees with its alt on {shown}");
            }
            Some(l) => {
                let (line, ours, theirs) = first_difference(&l.output, &l.alt_output);
                let _ = writeln!(
                    stdout,
                    "`{}` disagrees with its alt `{}` on {shown}: line {line} is {ours:?}, with `{}` swapped in it is {theirs:?}",
                    l.part, l.alt, l.alt
                );
                if let Some(d) = divergence() {
                    crate::cli::print_divergence(stdout, &d);
                }
            }
        }
    }
    u8::from(located.is_some())
}

/// The first `@alt(part)` whose swap changes what `file` does on `input`;
/// `None` when every part agrees.
pub fn locate(file: &Path, input: &str) -> Result<Option<Located>, String> {
    let src =
        std::fs::read_to_string(file).map_err(|e| format!("can't read {}: {e}", file.shown()))?;
    let alts = part_alts(&src);
    if alts.is_empty() {
        return Err(format!(
            "{} has no `@alt(f)` fn to swap in: write one beside the part you doubt",
            file.shown()
        ));
    }
    let name = file.shown();
    let solution = Program::resolve(file).map_err(|e| e.to_string())?;
    let ran = outcome(&solution, input)?;
    for (part, alt) in alts {
        let Some(swapped) = swap_in_part_brutes_of(&src, Some(&part)) else {
            continue;
        };
        let program = build_part_brute(file, &name, &swapped)?;
        let theirs = outcome(&program, input)?;
        if theirs.stdout != ran.stdout || theirs.exit != ran.exit {
            return Ok(Some(Located {
                part,
                alt,
                output: ran.stdout,
                alt_output: theirs.stdout,
            }));
        }
    }
    Ok(None)
}

fn outcome(program: &Program, input: &str) -> Result<Outcome, String> {
    program
        .run(input, LIMIT)
        .map_err(|e| format!("can't run {}: {e}", program.name))
}

/// The 1-based number of the first line where `a` and `b` differ, and each
/// side's line there (empty past its end).
fn first_difference<'a>(a: &'a str, b: &'a str) -> (usize, &'a str, &'a str) {
    let (mut la, mut lb) = (a.lines(), b.lines());
    let mut line = 1;
    loop {
        match (la.next(), lb.next()) {
            (x, y) if x != y => return (line, x.unwrap_or(""), y.unwrap_or("")),
            (None, None) => return (line, "", ""),
            _ => line += 1,
        }
    }
}

/// The newest `<file>.tests/stress-*.in`.
fn newest_saved_case(file: &Path) -> Option<PathBuf> {
    std::fs::read_dir(file.with_extension("tests"))
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension().is_some_and(|e| e == "in")
                && path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("stress-"))
        })
        .max_by_key(|path| std::fs::metadata(path).and_then(|m| m.modified()).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLANTED: &str = concat!(
        "fn double(n: Int) -> Int = n * 2\n",
        "@alt(double)\n",
        "fn double_slow(n: Int) -> Int = n + n\n",
        "fn square(n: Int) -> Int = if n == 3: 10 else: n * n\n",
        "@alt(square)\n",
        "fn square_slow(n: Int) -> Int = n * n\n",
        "let n: Int = read()\n",
        "print(double(n))\n",
        "print(square(n))\n",
    );

    fn file(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("cpl-locate-{}-{name}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join(format!("{name}.cp"));
        std::fs::write(&file, PLANTED).unwrap();
        file
    }

    #[test]
    fn names_the_first_part_whose_alt_disagrees() {
        let file = file("planted");
        let located = locate(&file, "3\n").unwrap().expect("square disagrees");
        assert_eq!(
            (located.part.as_str(), located.alt.as_str()),
            ("square", "square_slow")
        );
        assert_eq!(
            (located.output.as_str(), located.alt_output.as_str()),
            ("6\n10\n", "6\n9\n")
        );
        assert_eq!(locate(&file, "4\n").unwrap(), None);
    }

    #[test]
    fn reads_the_newest_saved_case_and_says_it_in_json() {
        let file = file("saved");
        let tests = file.with_extension("tests");
        std::fs::create_dir_all(&tests).unwrap();
        std::fs::write(tests.join("stress-7.in"), "3\n").unwrap();
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = run(
            &[file.display().to_string(), "--json".into()],
            &mut out,
            &mut err,
        );
        let out = String::from_utf8(out).unwrap();
        assert_eq!(code, 1, "{}", String::from_utf8_lossy(&err));
        let json = cpl_json::parse(&out).expect("JSON");
        assert_eq!(json.at(&["part"]).as_str(), Some("square"));
        assert_eq!(json.at(&["alt_output"]).as_str(), Some("6\n9\n"));
        assert!(json
            .at(&["input"])
            .as_str()
            .unwrap()
            .ends_with("stress-7.in"));
    }

    #[test]
    fn first_difference_counts_lines_from_one() {
        assert_eq!(first_difference("a\nb\n", "a\nc\n"), (2, "b", "c"));
        assert_eq!(first_difference("a\n", "a\nb\n"), (2, "", "b"));
    }
}
